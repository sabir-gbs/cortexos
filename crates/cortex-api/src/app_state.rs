//! Application state composition.
//!
//! Holds all concrete service instances and wires them together.
//! This is the single place where the dependency graph is assembled.

use std::sync::Arc;

use crate::bus::CommandBus;

/// Application-wide shared state.
///
/// Contains every service instance needed by the API handlers.
/// Constructed once at server startup and shared via `Arc`.
pub struct AppState {
    /// Database connection pool (shared by all services).
    pub pool: cortex_db::Pool,

    /// Authentication service.
    pub auth: cortex_auth::SqliteAuthService,

    /// Policy/permission service.
    pub policy: cortex_policy::SqlitePolicyService,

    /// Settings service.
    pub settings: cortex_settings::SqliteSettingsService,

    /// Virtual filesystem service.
    pub files: cortex_files::SqliteFilesService,

    /// Search service.
    pub search: cortex_search::SqliteSearchService,

    /// Notification service.
    pub notify: cortex_notify::SqliteNotifyService,

    /// Admin/diagnostics service.
    pub admin: cortex_admin::SqliteAdminService,

    /// Audit logger.
    pub audit: cortex_observability::AuditLogger,

    /// Runtime/app lifecycle service.
    pub runtime: cortex_runtime::SqliteRuntimeService,

    /// AI service.
    pub ai: tokio::sync::Mutex<cortex_ai::SqliteAiService>,

    /// Window manager service.
    pub wm: cortex_wm::SqliteWindowManager,

    /// Command bus for pub/sub events.
    pub bus: Arc<CommandBus>,
}

impl AppState {
    /// Build a new `AppState` from a database pool and configuration.
    ///
    /// Creates all service instances, wires them to the same pool, and
    /// initialises the command bus. The caller is responsible for running
    /// migrations before calling this.
    pub fn new(pool: cortex_db::Pool, data_dir: &str, session_ttl_secs: u64) -> Self {
        let bus = Arc::new(CommandBus::new(pool.clone()));

        let auth = cortex_auth::SqliteAuthService::new(pool.clone(), session_ttl_secs);
        let policy = cortex_policy::SqlitePolicyService::new(pool.clone());
        let settings = cortex_settings::SqliteSettingsService::new(pool.clone());
        let search = cortex_search::SqliteSearchService::new(pool.clone());
        let notify = cortex_notify::SqliteNotifyService::new(pool.clone());
        let admin = cortex_admin::SqliteAdminService::new(pool.clone());
        let audit = cortex_observability::AuditLogger::new(pool.clone());
        let runtime = cortex_runtime::SqliteRuntimeService::new(pool.clone());
        let ai = cortex_ai::SqliteAiService::new(pool.clone());
        let wm = cortex_wm::SqliteWindowManager::new(pool.clone());
        wm.init_schema().expect("window manager schema init");

        // Files service needs a default owner. We use a well-known system
        // user ID for files created outside of a user context.
        let system_user = cortex_core::UserId(uuid::Uuid::nil());
        let files =
            cortex_files::SqliteFilesService::new(pool.clone(), data_dir.to_string(), system_user);

        Self {
            pool,
            auth,
            policy,
            settings,
            files,
            search,
            notify,
            admin,
            audit,
            runtime,
            ai: tokio::sync::Mutex::new(ai),
            wm,
            bus,
        }
    }
}
