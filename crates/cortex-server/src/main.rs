//! CortexOS HTTP server.
//!
//! Axum-based HTTP server that exposes all API routes, WebSocket
//! connections, CORS middleware, auth middleware, and request logging.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::{HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use cortex_api::bus::CommandBus;
use cortex_api::middleware::cors::{cors_headers, CorsConfig};
use cortex_api::middleware::logging::RequestLog;
use cortex_api::routes;
use cortex_api::ws::{handle_client_frame, WsClientFrame, WsServerFrame, WsSession};
use cortex_config::AppConfig;
use cortex_core::SuccessResponse;
use futures_util::{SinkExt, StreamExt};
use percent_encoding::percent_decode_str;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Instant;
use tower_http::trace::TraceLayer;
use tracing::Level;

/// Shared application state, wrapped for axum.
pub type SharedState = Arc<cortex_api::AppState>;

/// Query params for the WebSocket handshake.
///
/// The `token` field is a transitional compatibility path. The canonical auth mechanism
/// per spec 03 is HttpOnly cookie-based sessions, which browsers send automatically on
/// the WebSocket upgrade request. The query-token fallback exists for non-browser clients
/// that cannot send cookies. Once all consumers migrate, this field should be removed.
#[derive(Debug, Deserialize)]
struct WsParams {
    #[allow(dead_code)]
    token: Option<String>,
}

/// Request body for moving a file.
#[derive(Debug, Deserialize)]
struct MoveFileBody {
    from: String,
    to: String,
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let config = AppConfig::load(None).unwrap_or_else(|e| {
        eprintln!("config error: {e}");
        std::process::exit(1);
    });

    let level: Level = config.log_level.parse().unwrap_or(Level::INFO);
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .init();

    std::fs::create_dir_all(&config.data_dir).unwrap_or_else(|e| {
        eprintln!("cannot create data dir '{}': {e}", config.data_dir);
        std::process::exit(1);
    });

    let pool = cortex_db::Pool::open(&config.database_url).unwrap_or_else(|e| {
        eprintln!("cannot open database '{}': {e}", config.database_url);
        std::process::exit(1);
    });

    cortex_db::run_migrations(&pool).unwrap_or_else(|e| {
        eprintln!("migration failed: {e}");
        std::process::exit(1);
    });

    let state = Arc::new(cortex_api::AppState::new(
        pool,
        &config.data_dir,
        config.session_ttl_secs,
    ));

    let cors_config = CorsConfig::default();

    let app = Router::new()
        // ── Health ────────────────────────────────
        .route("/api/v1/health", get(health_handler))
        // ── Auth ──────────────────────────────────
        .route("/api/v1/auth/login", post(auth_login_handler))
        .route("/api/v1/auth/logout", post(auth_logout_handler))
        .route("/api/v1/auth/me", get(auth_me_handler))
        .route("/api/v1/auth/profile", get(auth_profile_handler))
        .route("/api/v1/auth/profile", patch(auth_update_profile_handler))
        .route("/api/v1/auth/password", post(auth_change_password_handler))
        // ── Apps ──────────────────────────────────
        .route("/api/v1/apps/launch", post(apps_launch_handler))
        .route("/api/v1/apps/stop/{instance_id}", post(apps_stop_handler))
        .route(
            "/api/v1/apps/suspend/{instance_id}",
            post(apps_suspend_handler),
        )
        .route(
            "/api/v1/apps/resume/{instance_id}",
            post(apps_resume_handler),
        )
        .route("/api/v1/apps/state/{instance_id}", get(apps_state_handler))
        .route("/api/v1/apps/running", get(apps_running_handler))
        // ── Files ─────────────────────────────────
        .route("/api/v1/files/list", get(files_list_handler))
        .route("/api/v1/files/read", get(files_read_handler))
        .route("/api/v1/files/write", post(files_write_handler))
        .route("/api/v1/files/delete", delete(files_delete_handler))
        .route("/api/v1/files/move", post(files_move_handler))
        // REST-style file access (path-as-URL-segment, matches text-editor-app usage)
        .route("/api/v1/files/{*path}", get(files_read_rest_handler))
        .route("/api/v1/files/{*path}", put(files_write_rest_handler))
        // ── Settings ──────────────────────────────
        .route(
            "/api/v1/settings/{namespace}/{key}",
            get(settings_get_handler),
        )
        .route("/api/v1/settings", post(settings_set_handler))
        .route(
            "/api/v1/settings/{namespace}/{key}",
            delete(settings_delete_handler),
        )
        .route("/api/v1/settings", get(settings_list_handler))
        // ── Search ────────────────────────────────
        .route("/api/v1/search", get(search_handler))
        // ── Notifications ─────────────────────────
        .route("/api/v1/notifications", get(notifications_list_handler))
        .route("/api/v1/notifications", post(notifications_create_handler))
        .route(
            "/api/v1/notifications/{id}/read",
            post(notifications_mark_read_handler),
        )
        .route(
            "/api/v1/notifications/{id}/dismiss",
            post(notifications_dismiss_handler),
        )
        // ── Policy ────────────────────────────────
        .route("/api/v1/policy/check", post(policy_check_handler))
        .route("/api/v1/policy/grant", post(policy_grant_handler))
        .route(
            "/api/v1/policy/grant/{grant_id}",
            delete(policy_revoke_handler),
        )
        .route("/api/v1/policy/grants/{app_id}", get(policy_list_handler))
        // ── AI ────────────────────────────────────
        .route("/api/v1/ai/chat", post(ai_chat_handler))
        .route("/api/v1/ai/providers", get(ai_providers_handler))
        .route("/api/v1/ai/models/{provider}", get(ai_models_handler))
        // ── Window Manager ────────────────────────
        .route("/api/v1/wm/windows", post(wm_open_window_handler))
        .route(
            "/api/v1/wm/windows/{id}/close",
            post(wm_close_window_handler),
        )
        .route(
            "/api/v1/wm/windows/{id}/minimize",
            post(wm_minimize_handler),
        )
        .route(
            "/api/v1/wm/windows/{id}/maximize",
            post(wm_maximize_handler),
        )
        .route("/api/v1/wm/windows/{id}/restore", post(wm_restore_handler))
        .route("/api/v1/wm/windows/{id}/move", post(wm_move_handler))
        .route("/api/v1/wm/windows/{id}/resize", post(wm_resize_handler))
        .route("/api/v1/wm/windows/{id}/focus", post(wm_focus_handler))
        .route("/api/v1/wm/windows/{id}", get(wm_get_window_handler))
        .route("/api/v1/wm/windows", get(wm_list_windows_handler))
        .route("/api/v1/wm/workspaces", post(wm_create_workspace_handler))
        .route(
            "/api/v1/wm/workspaces/active",
            get(wm_active_workspace_handler),
        )
        .route(
            "/api/v1/wm/workspaces/{id}/switch",
            post(wm_switch_workspace_handler),
        )
        .route("/api/v1/wm/workspaces", get(wm_list_workspaces_handler))
        .route(
            "/api/v1/wm/workspaces/{id}",
            delete(wm_delete_workspace_handler),
        )
        // ── Admin ─────────────────────────────────
        .route("/api/v1/admin/health", get(admin_health_handler))
        .route("/api/v1/admin/diagnostics", get(admin_diagnostics_handler))
        .route("/api/v1/admin/backup", post(admin_backup_handler))
        // ── WebSocket ─────────────────────────────
        .route("/ws", get(ws_handler))
        // ── Middleware ─────────────────────────────
        .layer(middleware::from_fn_with_state(cors_config, cors_middleware))
        .layer(middleware::from_fn(logging_middleware))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("{}:{}", config.bind_addr, config.port);
    tracing::info!("CortexOS server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("cannot bind {addr}: {e}");
            std::process::exit(1);
        });

    axum::serve(listener, app).await.unwrap_or_else(|e| {
        eprintln!("server error: {e}");
        std::process::exit(1);
    });
}

// ── Middleware ─────────────────────────────────────────────────────────────────

async fn cors_middleware(
    State(config): State<CorsConfig>,
    req: axum::extract::Request,
    next: Next,
) -> Response {
    let origin = req
        .headers()
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("*")
        .to_string();

    let mut response = next.run(req).await;

    if let Some(headers) = cors_headers(&config, &origin) {
        let hdrs = response.headers_mut();
        hdrs.insert(
            "access-control-allow-origin",
            HeaderValue::from_str(&headers.access_control_allow_origin)
                .unwrap_or(HeaderValue::from_static("*")),
        );
        hdrs.insert(
            "access-control-allow-methods",
            HeaderValue::from_str(&headers.access_control_allow_methods)
                .unwrap_or(HeaderValue::from_static("GET, POST, PUT, DELETE, PATCH")),
        );
        hdrs.insert(
            "access-control-allow-headers",
            HeaderValue::from_str(&headers.access_control_allow_headers)
                .unwrap_or(HeaderValue::from_static("Content-Type, Authorization")),
        );
        hdrs.insert(
            "access-control-max-age",
            HeaderValue::from_str(&headers.access_control_max_age.to_string())
                .unwrap_or(HeaderValue::from_static("86400")),
        );
        if headers.access_control_allow_credentials {
            hdrs.insert(
                "access-control-allow-credentials",
                HeaderValue::from_static("true"),
            );
        }
    }

    response
}

async fn logging_middleware(req: axum::extract::Request, next: Next) -> Response {
    let method = req.method().clone().to_string();
    let path = req.uri().path().to_string();
    let request_id = uuid::Uuid::now_v7().to_string();
    let _start = Instant::now();

    let log = RequestLog::new(&method, &path, &request_id);
    let response = next.run(req).await;
    let status = response.status().as_u16();
    log.finish(status);
    response
}

// ── Error response helper ─────────────────────────────────────────────────────

fn error_response(status: StatusCode, message: &str) -> Response {
    let err = cortex_api::ApiError::Internal(message.to_string());
    (status, Json(err.to_cortex_error())).into_response()
}

fn to_response<T: serde::Serialize>(result: cortex_api::Result<SuccessResponse<T>>) -> Response {
    match result {
        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
        Err(e) => {
            let (status, cortex_err) = match &e {
                cortex_api::ApiError::Unauthorized(_) => {
                    (StatusCode::UNAUTHORIZED, e.to_cortex_error())
                }
                cortex_api::ApiError::Forbidden(_) => (StatusCode::FORBIDDEN, e.to_cortex_error()),
                cortex_api::ApiError::NotFound(_) => (StatusCode::NOT_FOUND, e.to_cortex_error()),
                cortex_api::ApiError::BadRequest(_) => {
                    (StatusCode::BAD_REQUEST, e.to_cortex_error())
                }
                cortex_api::ApiError::Internal(_) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, e.to_cortex_error())
                }
            };
            (status, Json(cortex_err)).into_response()
        }
    }
}

// ── Health ─────────────────────────────────────────────────────────────────────

async fn health_handler() -> Response {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
}

// ── Auth ───────────────────────────────────────────────────────────────────────

async fn auth_login_handler(
    State(state): State<SharedState>,
    Json(req): Json<cortex_auth::types::LoginRequest>,
) -> Response {
    let result = routes::auth::login(&state, req).await;
    match result {
        Ok(resp_data) => {
            let session_token = &resp_data.data.token;
            let cookie = session_cookie_header(session_token);
            let mut resp = (StatusCode::OK, Json(resp_data)).into_response();
            if let Ok(val) = axum::http::HeaderValue::from_str(&cookie) {
                resp.headers_mut().insert("set-cookie", val);
            }
            resp
        }
        Err(e) => to_response::<cortex_api::routes::auth::LoginResponse>(Err(e)),
    }
}

async fn auth_logout_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let token = extract_cookie_token(&headers).or_else(|| extract_token(&headers));
    match token {
        Some(t) => {
            let resp = to_response(routes::auth::logout(&state, &t).await);
            let clear = clearing_cookie_header();
            if let Ok(val) = axum::http::HeaderValue::from_str(&clear) {
                let mut r = resp;
                r.headers_mut().insert("set-cookie", val);
                return r;
            }
            resp
        }
        None => error_response(StatusCode::UNAUTHORIZED, "missing auth token"),
    }
}

async fn auth_me_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let auth = authenticate(&state, &headers).await;
    match auth {
        Ok(ctx) => to_response(routes::auth::get_profile(&state, &ctx.user_id).await),
        Err(e) => e,
    }
}

async fn auth_profile_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let auth = authenticate(&state, &headers).await;
    match auth {
        Ok(ctx) => to_response(routes::auth::get_profile(&state, &ctx.user_id).await),
        Err(e) => e,
    }
}

async fn auth_update_profile_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<cortex_auth::types::ProfileUpdate>,
) -> Response {
    let auth = authenticate(&state, &headers).await;
    match auth {
        Ok(ctx) => to_response(routes::auth::update_profile(&state, &ctx.user_id, req).await),
        Err(e) => e,
    }
}

async fn auth_change_password_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<serde_json::Value>,
) -> Response {
    let auth = authenticate(&state, &headers).await;
    match auth {
        Ok(ctx) => {
            let current = req.get("current").and_then(|v| v.as_str()).unwrap_or("");
            let new = req.get("new").and_then(|v| v.as_str()).unwrap_or("");
            to_response(routes::auth::change_password(&state, &ctx.user_id, current, new).await)
        }
        Err(e) => e,
    }
}

// ── Apps ───────────────────────────────────────────────────────────────────────

async fn apps_launch_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<routes::apps::LaunchRequest>,
) -> Response {
    let auth = authenticate(&state, &headers).await;
    match auth {
        Ok(ctx) => to_response(routes::apps::launch(&state, &ctx.user_id, req).await),
        Err(e) => e,
    }
}

async fn apps_stop_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
    Path(instance_id): Path<String>,
) -> Response {
    let auth = authenticate(&state, &headers).await;
    match auth {
        Ok(_) => to_response(routes::apps::stop(&state, &instance_id).await),
        Err(e) => e,
    }
}

async fn apps_suspend_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
    Path(instance_id): Path<String>,
) -> Response {
    let auth = authenticate(&state, &headers).await;
    match auth {
        Ok(_) => to_response(routes::apps::suspend(&state, &instance_id).await),
        Err(e) => e,
    }
}

async fn apps_resume_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
    Path(instance_id): Path<String>,
) -> Response {
    let auth = authenticate(&state, &headers).await;
    match auth {
        Ok(_) => to_response(routes::apps::resume(&state, &instance_id).await),
        Err(e) => e,
    }
}

async fn apps_state_handler(
    State(state): State<SharedState>,
    Path(instance_id): Path<String>,
) -> Response {
    to_response(routes::apps::get_state(&state, &instance_id).await)
}

async fn apps_running_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let auth = authenticate(&state, &headers).await;
    match auth {
        Ok(ctx) => to_response(routes::apps::list_running(&state, &ctx.user_id).await),
        Err(e) => e,
    }
}

// ── Files ──────────────────────────────────────────────────────────────────────

async fn files_list_handler(
    State(state): State<SharedState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let path = params.get("path").map(|s| s.as_str()).unwrap_or("/");
    to_response(routes::files::list(&state, path).await)
}

async fn files_read_handler(
    State(state): State<SharedState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let path = params.get("path").map(|s| s.as_str()).unwrap_or("");
    to_response(routes::files::read(&state, path).await)
}

async fn files_write_handler(
    State(state): State<SharedState>,
    Json(req): Json<routes::files::WriteFileRequest>,
) -> Response {
    to_response(routes::files::write(&state, req).await)
}

async fn files_delete_handler(
    State(state): State<SharedState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let path = params.get("path").map(|s| s.as_str()).unwrap_or("");
    to_response(routes::files::delete(&state, path).await)
}

async fn files_move_handler(
    State(state): State<SharedState>,
    Json(req): Json<MoveFileBody>,
) -> Response {
    to_response(routes::files::move_file(&state, &req.from, &req.to).await)
}

// REST-style file handlers (path as URL segment, matches text-editor-app contract)

async fn files_read_rest_handler(
    State(state): State<SharedState>,
    Path(path): Path<String>,
) -> Response {
    let decoded = percent_decode_str(&path).decode_utf8_lossy().to_string();
    to_response(routes::files::read(&state, &decoded).await)
}

#[derive(Deserialize)]
struct WriteContentBody {
    content: String,
}

async fn files_write_rest_handler(
    State(state): State<SharedState>,
    Path(path): Path<String>,
    Json(body): Json<WriteContentBody>,
) -> Response {
    let decoded = percent_decode_str(&path).decode_utf8_lossy().to_string();
    let content_b64 = {
        use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
        BASE64.encode(body.content.as_bytes())
    };
    let req = routes::files::WriteFileRequest {
        path: decoded,
        content_base64: content_b64,
    };
    to_response(routes::files::write(&state, req).await)
}

// ── Settings ───────────────────────────────────────────────────────────────────

async fn settings_get_handler(
    State(state): State<SharedState>,
    Path((namespace, key)): Path<(String, String)>,
) -> Response {
    to_response(routes::settings::get(&state, &namespace, &key).await)
}

async fn settings_set_handler(
    State(state): State<SharedState>,
    Json(req): Json<routes::settings::SetSettingRequest>,
) -> Response {
    to_response(routes::settings::set(&state, req).await)
}

async fn settings_delete_handler(
    State(state): State<SharedState>,
    Path((namespace, key)): Path<(String, String)>,
) -> Response {
    to_response(routes::settings::delete(&state, &namespace, &key).await)
}

async fn settings_list_handler(
    State(state): State<SharedState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let namespace = params.get("namespace").map(|s| s.as_str()).unwrap_or("");
    to_response(routes::settings::list_all(&state, namespace).await)
}

// ── Search ─────────────────────────────────────────────────────────────────────

async fn search_handler(
    State(state): State<SharedState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let req = routes::search::SearchRequest {
        query: params
            .get("q")
            .map(|s| s.as_str())
            .unwrap_or("")
            .to_string(),
        limit: params.get("limit").and_then(|s| s.parse::<u32>().ok()),
    };
    to_response(routes::search::search(&state, req).await)
}

// ── Notifications ──────────────────────────────────────────────────────────────

async fn notifications_list_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let auth = authenticate(&state, &headers).await;
    match auth {
        Ok(ctx) => {
            to_response(routes::notifications::list(&state, &ctx.user_id.0.to_string()).await)
        }
        Err(e) => e,
    }
}

async fn notifications_create_handler(
    State(state): State<SharedState>,
    Json(req): Json<cortex_notify::Notification>,
) -> Response {
    to_response(routes::notifications::create(&state, req).await)
}

async fn notifications_mark_read_handler(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Response {
    to_response(routes::notifications::mark_read(&state, &id).await)
}

async fn notifications_dismiss_handler(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Response {
    to_response(routes::notifications::dismiss(&state, &id).await)
}

// ── Policy ─────────────────────────────────────────────────────────────────────

async fn policy_check_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<routes::policy::CheckRequest>,
) -> Response {
    let auth = authenticate(&state, &headers).await;
    match auth {
        Ok(ctx) => to_response(routes::policy::check_permission(&state, &ctx.user_id, req).await),
        Err(e) => e,
    }
}

async fn policy_grant_handler(
    State(state): State<SharedState>,
    Json(req): Json<routes::policy::GrantRequest>,
) -> Response {
    to_response(routes::policy::grant(&state, req).await)
}

async fn policy_revoke_handler(
    State(state): State<SharedState>,
    Path(grant_id): Path<String>,
) -> Response {
    to_response(routes::policy::revoke(&state, &grant_id).await)
}

async fn policy_list_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
    Path(app_id): Path<String>,
) -> Response {
    let auth = authenticate(&state, &headers).await;
    match auth {
        Ok(ctx) => to_response(routes::policy::list_grants(&state, &ctx.user_id, &app_id).await),
        Err(e) => e,
    }
}

// ── AI ─────────────────────────────────────────────────────────────────────────

async fn ai_chat_handler(
    State(state): State<SharedState>,
    Json(req): Json<routes::ai::ChatRequestBody>,
) -> Response {
    to_response(routes::ai::chat(&state, req).await)
}

async fn ai_providers_handler(State(state): State<SharedState>) -> Response {
    to_response(routes::ai::list_providers(&state).await)
}

async fn ai_models_handler(
    State(state): State<SharedState>,
    Path(provider): Path<String>,
) -> Response {
    to_response(routes::ai::list_models(&state, Some(&provider)).await)
}

// ── Window Manager ─────────────────────────────────────────────────────────────

async fn wm_open_window_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<cortex_wm::types::OpenWindowRequest>,
) -> Response {
    let auth = authenticate(&state, &headers).await;
    match auth {
        Ok(ctx) => to_response(routes::wm::open_window(&state, &ctx.user_id, req).await),
        Err(e) => e,
    }
}

async fn wm_close_window_handler(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Response {
    to_response(routes::wm::close_window(&state, &id).await)
}

async fn wm_minimize_handler(State(state): State<SharedState>, Path(id): Path<String>) -> Response {
    to_response(routes::wm::minimize_window(&state, &id).await)
}

async fn wm_maximize_handler(State(state): State<SharedState>, Path(id): Path<String>) -> Response {
    to_response(routes::wm::maximize_window(&state, &id).await)
}

async fn wm_restore_handler(State(state): State<SharedState>, Path(id): Path<String>) -> Response {
    to_response(routes::wm::restore_window(&state, &id).await)
}

async fn wm_move_handler(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<cortex_wm::types::MoveWindowRequest>,
) -> Response {
    to_response(routes::wm::move_window(&state, &id, req).await)
}

async fn wm_resize_handler(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<cortex_wm::types::ResizeWindowRequest>,
) -> Response {
    to_response(routes::wm::resize_window(&state, &id, req).await)
}

async fn wm_focus_handler(State(state): State<SharedState>, Path(id): Path<String>) -> Response {
    to_response(routes::wm::focus_window(&state, &id).await)
}

async fn wm_get_window_handler(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Response {
    to_response(routes::wm::get_window(&state, &id).await)
}

async fn wm_list_windows_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let auth = authenticate(&state, &headers).await;
    let ws = params.get("workspace_id").map(|s| s.as_str()).unwrap_or("");
    match auth {
        Ok(ctx) => to_response(routes::wm::list_windows(&state, &ctx.user_id, ws).await),
        Err(e) => e,
    }
}

async fn wm_create_workspace_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<cortex_wm::types::CreateWorkspaceRequest>,
) -> Response {
    let auth = authenticate(&state, &headers).await;
    match auth {
        Ok(ctx) => to_response(routes::wm::create_workspace(&state, &ctx.user_id, req).await),
        Err(e) => e,
    }
}

async fn wm_active_workspace_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let auth = authenticate(&state, &headers).await;
    match auth {
        Ok(ctx) => to_response(routes::wm::get_active_workspace(&state, &ctx.user_id).await),
        Err(e) => e,
    }
}

async fn wm_switch_workspace_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let auth = authenticate(&state, &headers).await;
    match auth {
        Ok(ctx) => to_response(routes::wm::switch_workspace(&state, &ctx.user_id, &id).await),
        Err(e) => e,
    }
}

async fn wm_list_workspaces_handler(
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let auth = authenticate(&state, &headers).await;
    match auth {
        Ok(ctx) => to_response(routes::wm::list_workspaces(&state, &ctx.user_id).await),
        Err(e) => e,
    }
}

async fn wm_delete_workspace_handler(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Response {
    to_response(routes::wm::delete_workspace(&state, &id).await)
}

// ── Admin ──────────────────────────────────────────────────────────────────────

async fn admin_health_handler(State(state): State<SharedState>) -> Response {
    to_response(routes::admin::health(&state).await)
}

async fn admin_diagnostics_handler(State(state): State<SharedState>) -> Response {
    to_response(routes::admin::diagnostics(&state).await)
}

async fn admin_backup_handler(
    State(state): State<SharedState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let path = params
        .get("path")
        .map(|s| s.as_str())
        .unwrap_or("./backup.db");
    to_response(routes::admin::backup(&state, path).await)
}

// ── WebSocket ──────────────────────────────────────────────────────────────────

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsParams>,
    State(state): State<SharedState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let token = params
        .token
        .as_deref()
        .map(|s| s.to_string())
        .or_else(|| extract_cookie_token(&headers));

    let session = match token {
        Some(t) => match cortex_api::middleware::auth::require_auth(&state, &t).await {
            Ok(ctx) => WsSession::for_user(ctx.user_id),
            Err(_) => WsSession::new(),
        },
        None => WsSession::new(),
    };

    ws.on_upgrade(move |socket| handle_ws_connection(socket, session, state.bus.clone()))
        .into_response()
}

async fn handle_ws_connection(socket: WebSocket, mut session: WsSession, bus: Arc<CommandBus>) {
    let (mut sender, mut receiver) = socket.split();

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let frame: std::result::Result<WsClientFrame, _> = serde_json::from_str(&text);

                match frame {
                    Ok(client_frame) => {
                        let responses = handle_client_frame(&mut session, client_frame, &bus);
                        for response in responses {
                            let json = serde_json::to_string(&response).unwrap_or_default();
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        let error = WsServerFrame::Error {
                            message: format!("invalid frame: {e}"),
                        };
                        let json = serde_json::to_string(&error).unwrap_or_default();
                        if sender.send(Message::Text(json.into())).await.is_err() {
                            return;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => break,
            _ => {}
        }
    }
}

// ── Auth helper ────────────────────────────────────────────────────────────────

fn extract_cookie_token(headers: &axum::http::HeaderMap) -> Option<String> {
    let cookie_header = headers.get("cookie")?.to_str().ok()?;
    cookie_header.split(';').find_map(|c| {
        let parts: Vec<&str> = c.trim().splitn(2, '=').collect();
        if parts.len() == 2 && parts[0] == routes::auth::SESSION_COOKIE_NAME {
            Some(parts[1].to_string())
        } else {
            None
        }
    })
}

/// Extract bearer token from the Authorization header.
///
/// This is a transitional compatibility path. The canonical auth mechanism per spec 03
/// is HttpOnly cookie-based sessions. Bearer tokens are accepted as a fallback for
/// SDK consumers and API-only clients that cannot use cookies. Once all consumers
/// migrate to cookie auth, this function should be removed.
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    let auth_header = headers.get("Authorization")?.to_str().ok()?;
    cortex_api::middleware::auth::extract_bearer_token(auth_header).map(|s| s.to_string())
}

fn session_cookie_header(session_id: &str) -> String {
    routes::auth::session_cookie_value(session_id)
}

fn clearing_cookie_header() -> String {
    format!(
        "{}=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0",
        routes::auth::SESSION_COOKIE_NAME
    )
}

/// Authenticate a request by extracting the session token.
///
/// Per spec 03 section 6.2, the canonical auth mechanism is cookie-based:
/// "Extract session token from cookie." The bearer-token fallback is a transitional
/// compatibility path for SDK/API consumers that cannot use cookies.
async fn authenticate(
    state: &cortex_api::AppState,
    headers: &axum::http::HeaderMap,
) -> std::result::Result<cortex_api::middleware::auth::AuthContext, Response> {
    let token = extract_cookie_token(headers).or_else(|| extract_token(headers));

    let token = match token {
        Some(t) => t,
        None => {
            return Err(error_response(
                StatusCode::UNAUTHORIZED,
                "missing auth token",
            ))
        }
    };

    cortex_api::middleware::auth::require_auth(state, &token)
        .await
        .map_err(|_| error_response(StatusCode::UNAUTHORIZED, "invalid or expired session"))
}
