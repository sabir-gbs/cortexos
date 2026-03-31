//! SQLite-backed implementation of [`RuntimeService`].

use std::future::Future;
use std::pin::Pin;

use cortex_core::{AppId, AppInstanceId, UserId};
use uuid::Uuid;

use crate::error::{Result, RuntimeError};
use crate::service::RuntimeService;
use crate::types::{AppInstance, AppState};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Deserialize an [`AppState`] from the database `state` column.
fn parse_state(raw: &str) -> cortex_db::Result<AppState> {
    let wrapped = format!("\"{raw}\"");
    serde_json::from_str(&wrapped)
        .map_err(|e| cortex_db::DbError::Query(format!("invalid state '{raw}': {e}")))
}

/// Serialize an [`AppState`] to a string suitable for the `state` column.
fn serialize_state(state: AppState) -> String {
    serde_json::to_string(&state)
        .expect("AppState serialization is infallible")
        .trim_matches('"')
        .to_string()
}

/// Convert a [`cortex_db::DbError`] into a [`RuntimeError`].
///
/// Maps `NotFound` variants to [`RuntimeError::AppNotFound`] and everything
/// else to [`RuntimeError::Internal`].
fn db_err(e: cortex_db::DbError) -> RuntimeError {
    match e {
        cortex_db::DbError::NotFound(msg) => RuntimeError::AppNotFound(msg),
        _ => RuntimeError::Internal,
    }
}

/// Raw column values read from an `app_instances` row.
type Row = (
    String,         // instance_id
    String,         // app_id
    String,         // user_id
    String,         // state
    Option<String>, // window_id
    Option<String>, // launched_at
    Option<String>, // stopped_at
);

/// Extract raw columns from a rusqlite row.
fn extract_row(row: &rusqlite::Row<'_>) -> std::result::Result<Row, rusqlite::Error> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
    ))
}

/// Convert raw row data into an [`AppInstance`].
fn row_to_instance(
    instance_id_str: String,
    app_id_str: String,
    user_id_str: String,
    state_str: String,
    window_id: Option<String>,
    launched_at: Option<String>,
    stopped_at: Option<String>,
) -> cortex_db::Result<AppInstance> {
    let instance_id = Uuid::parse_str(&instance_id_str)
        .map_err(|e| cortex_db::DbError::Query(format!("invalid instance_id uuid: {e}")))?;
    let user_id = Uuid::parse_str(&user_id_str)
        .map_err(|e| cortex_db::DbError::Query(format!("invalid user_id uuid: {e}")))?;
    let state = parse_state(&state_str)?;

    Ok(AppInstance {
        instance_id: AppInstanceId(instance_id),
        app_id: AppId(app_id_str),
        user_id: UserId(user_id),
        state,
        window_id,
        launched_at,
        stopped_at,
    })
}

// ---------------------------------------------------------------------------
// SqliteRuntimeService
// ---------------------------------------------------------------------------

/// SQLite-backed runtime service.
pub struct SqliteRuntimeService {
    pool: cortex_db::Pool,
}

impl SqliteRuntimeService {
    /// Create a new service backed by the given connection pool.
    pub fn new(pool: cortex_db::Pool) -> Self {
        Self { pool }
    }
}

// ---------------------------------------------------------------------------
// RuntimeService implementation
// ---------------------------------------------------------------------------

impl RuntimeService for SqliteRuntimeService {
    #[allow(refining_impl_trait)]
    fn launch(
        &self,
        app_id: &AppId,
        user_id: &UserId,
    ) -> Pin<Box<dyn Future<Output = Result<AppInstance>> + Send + '_>> {
        let app_id = app_id.clone();
        let user_id = user_id.clone();
        Box::pin(async move {
            let instance_id = AppInstanceId(Uuid::now_v7());

            // Insert the new instance with state 'launching'; then
            // transition to 'running' once the write succeeds.  The
            // database records the real server-side timestamp.
            let state = AppState::Starting;
            self.pool
                .write(|conn| {
                    conn.execute(
                        "INSERT INTO app_instances (instance_id, app_id, user_id, state, launched_at) \
                         VALUES (?1, ?2, ?3, ?4, datetime('now'))",
                        rusqlite::params![
                            instance_id.0.to_string(),
                            app_id.0,
                            user_id.0.to_string(),
                            serialize_state(state),
                        ],
                    )
                    .map_err(|e| cortex_db::DbError::Query(format!("insert app instance: {e}")))?;

                    // Transition to Running now that the row is persisted.
                    conn.execute(
                        "UPDATE app_instances SET state = ?1 WHERE instance_id = ?2",
                        rusqlite::params![
                            serialize_state(AppState::Running),
                            instance_id.0.to_string(),
                        ],
                    )
                    .map_err(|e| {
                        cortex_db::DbError::Query(format!("update instance to running: {e}"))
                    })?;
                    Ok(())
                })
                .map_err(|e| {
                    tracing::error!(target: "runtime", "failed to persist app instance: {e}");
                    RuntimeError::Internal
                })?;

            // Read back the instance to get the server-generated launched_at timestamp.
            let iid_str = instance_id.0.to_string();
            let instance = self
                .pool
                .read(|conn| {
                    let (iid, aid, uid, st, wid, lat, sat) = conn
                        .query_row(
                            "SELECT instance_id, app_id, user_id, state, window_id, launched_at, stopped_at \
                             FROM app_instances WHERE instance_id = ?1",
                            rusqlite::params![iid_str],
                            extract_row,
                        )
                        .map_err(|e| {
                            cortex_db::DbError::Query(format!(
                                "read back launched instance: {e}"
                            ))
                        })?;
                    row_to_instance(iid, aid, uid, st, wid, lat, sat)
                })
                .map_err(db_err)?;

            Ok(instance)
        })
    }

    #[allow(refining_impl_trait)]
    fn stop(
        &self,
        instance_id: &AppInstanceId,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        let iid_str = instance_id.0.to_string();
        Box::pin(async move {
            // Fetch current state
            let current_state: AppState = self
                .pool
                .read(|conn| {
                    let state_str: String = conn
                        .query_row(
                            "SELECT state FROM app_instances WHERE instance_id = ?1",
                            rusqlite::params![iid_str],
                            |row| row.get(0),
                        )
                        .map_err(|e| {
                            cortex_db::DbError::NotFound(format!("app instance not found: {e}"))
                        })?;
                    parse_state(&state_str)
                })
                .map_err(db_err)?;

            if current_state.is_terminal() {
                return Err(RuntimeError::NotRunning(iid_str.clone()));
            }

            // Transition to Stopping then Stopped
            let stopping = current_state
                .transition_to(AppState::Stopping)
                .map_err(|_| RuntimeError::NotRunning(iid_str.clone()))?;
            let stopped = stopping
                .transition_to(AppState::Stopped)
                .expect("Stopping -> Stopped is always valid");

            self.pool
                .write(|conn| {
                    conn.execute(
                        "UPDATE app_instances SET state = ?1, stopped_at = datetime('now') \
                         WHERE instance_id = ?2",
                        rusqlite::params![serialize_state(stopped), iid_str],
                    )
                    .map_err(|e| {
                        cortex_db::DbError::Query(format!("update instance to stopped: {e}"))
                    })?;
                    Ok(())
                })
                .map_err(|_| RuntimeError::Internal)?;

            Ok(())
        })
    }

    #[allow(refining_impl_trait)]
    fn get_state(
        &self,
        instance_id: &AppInstanceId,
    ) -> Pin<Box<dyn Future<Output = Result<AppState>> + Send + '_>> {
        let iid_str = instance_id.0.to_string();
        Box::pin(async move {
            self.pool
                .read(|conn| {
                    let state_str: String = conn
                        .query_row(
                            "SELECT state FROM app_instances WHERE instance_id = ?1",
                            rusqlite::params![iid_str],
                            |row| row.get(0),
                        )
                        .map_err(|e| {
                            cortex_db::DbError::NotFound(format!("app instance not found: {e}"))
                        })?;
                    parse_state(&state_str)
                })
                .map_err(db_err)
        })
    }

    #[allow(refining_impl_trait)]
    fn list_running(
        &self,
        user_id: &UserId,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<AppInstance>>> + Send + '_>> {
        let uid_str = user_id.0.to_string();
        Box::pin(async move {
            self.pool
                .read(|conn| {
                    let mut stmt = conn
                        .prepare(
                            "SELECT instance_id, app_id, user_id, state, window_id, launched_at, stopped_at \
                             FROM app_instances \
                             WHERE user_id = ?1 AND state NOT IN ('stopped', 'crashed')",
                        )
                        .map_err(|e| {
                            cortex_db::DbError::Query(format!(
                                "prepare list_running: {e}"
                            ))
                        })?;
                    let rows = stmt
                        .query_map(rusqlite::params![uid_str], extract_row)
                        .map_err(|e| {
                            cortex_db::DbError::Query(format!(
                                "query list_running: {e}"
                            ))
                        })?;
                    let mut instances = Vec::new();
                    for row in rows {
                        let (iid, aid, uid, st, wid, lat, sat) =
                            row.map_err(|e| {
                                cortex_db::DbError::Query(format!(
                                    "row mapping error: {e}"
                                ))
                            })?;
                        instances.push(row_to_instance(iid, aid, uid, st, wid, lat, sat)?);
                    }
                    Ok(instances)
                })
                .map_err(db_err)
        })
    }

    #[allow(refining_impl_trait)]
    fn suspend(
        &self,
        instance_id: &AppInstanceId,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        let iid_str = instance_id.0.to_string();
        Box::pin(async move {
            let current_state: AppState = self
                .pool
                .read(|conn| {
                    let state_str: String = conn
                        .query_row(
                            "SELECT state FROM app_instances WHERE instance_id = ?1",
                            rusqlite::params![iid_str],
                            |row| row.get(0),
                        )
                        .map_err(|e| {
                            cortex_db::DbError::NotFound(format!("app instance not found: {e}"))
                        })?;
                    parse_state(&state_str)
                })
                .map_err(db_err)?;

            let suspended = current_state
                .transition_to(AppState::Suspended)
                .map_err(|_| RuntimeError::NotRunning(iid_str.clone()))?;

            self.pool
                .write(|conn| {
                    conn.execute(
                        "UPDATE app_instances SET state = ?1 WHERE instance_id = ?2",
                        rusqlite::params![serialize_state(suspended), iid_str],
                    )
                    .map_err(|e| {
                        cortex_db::DbError::Query(format!("update instance to suspended: {e}"))
                    })?;
                    Ok(())
                })
                .map_err(|_| RuntimeError::Internal)?;

            Ok(())
        })
    }

    #[allow(refining_impl_trait)]
    fn resume(
        &self,
        instance_id: &AppInstanceId,
    ) -> Pin<Box<dyn Future<Output = Result<AppInstance>> + Send + '_>> {
        let iid_str = instance_id.0.to_string();
        Box::pin(async move {
            let current_state: AppState = self
                .pool
                .read(|conn| {
                    let state_str: String = conn
                        .query_row(
                            "SELECT state FROM app_instances WHERE instance_id = ?1",
                            rusqlite::params![iid_str],
                            |row| row.get(0),
                        )
                        .map_err(|e| {
                            cortex_db::DbError::NotFound(format!("app instance not found: {e}"))
                        })?;
                    parse_state(&state_str)
                })
                .map_err(db_err)?;

            let running = current_state
                .transition_to(AppState::Running)
                .map_err(|_| RuntimeError::NotRunning(iid_str.clone()))?;

            self.pool
                .write(|conn| {
                    conn.execute(
                        "UPDATE app_instances SET state = ?1 WHERE instance_id = ?2",
                        rusqlite::params![serialize_state(running), iid_str],
                    )
                    .map_err(|e| {
                        cortex_db::DbError::Query(format!("update instance to running: {e}"))
                    })?;
                    Ok(())
                })
                .map_err(|_| RuntimeError::Internal)?;

            // Read back the updated instance
            let instance = self
                .pool
                .read(|conn| {
                    let (iid, aid, uid, st, wid, lat, sat) = conn
                        .query_row(
                            "SELECT instance_id, app_id, user_id, state, window_id, launched_at, stopped_at \
                             FROM app_instances WHERE instance_id = ?1",
                            rusqlite::params![iid_str],
                            extract_row,
                        )
                        .map_err(|e| {
                            cortex_db::DbError::Query(format!(
                                "read back resumed instance: {e}"
                            ))
                        })?;
                    row_to_instance(iid, aid, uid, st, wid, lat, sat)
                })
                .map_err(db_err)?;

            Ok(instance)
        })
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::RuntimeError;

    /// Helper: create an in-memory database with migrations applied and a
    /// test user inserted for FK constraints.
    fn setup() -> SqliteRuntimeService {
        let pool = cortex_db::Pool::open_in_memory().unwrap();
        cortex_db::run_migrations(&pool).unwrap();

        // Insert a test user so the FK constraint on app_instances passes.
        pool.write(|conn| {
            conn.execute(
                "INSERT INTO users (user_id, username, display_name, password_hash) \
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![
                    "00000000-0000-7000-8000-000000000001",
                    "testuser",
                    "Test User",
                    "irrelevant",
                ],
            )
            .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
            Ok(())
        })
        .unwrap();

        SqliteRuntimeService::new(pool)
    }

    /// The user_id used by all tests.
    fn test_user_id() -> UserId {
        UserId(Uuid::parse_str("00000000-0000-7000-8000-000000000001").unwrap())
    }

    // -- launch_creates_running_instance ------------------------------------

    #[tokio::test]
    async fn launch_creates_running_instance() {
        let svc = setup();
        let app_id = AppId("com.cortexos.calculator".into());
        let user_id = test_user_id();

        let instance = svc.launch(&app_id, &user_id).await.unwrap();

        assert_eq!(instance.app_id.0, "com.cortexos.calculator");
        assert_eq!(instance.user_id, user_id);
        assert_eq!(instance.state, AppState::Running);
        assert!(instance.launched_at.is_some());
        assert!(instance.stopped_at.is_none());
    }

    // -- launch_and_list_running --------------------------------------------

    #[tokio::test]
    async fn launch_and_list_running() {
        let svc = setup();
        let user_id = test_user_id();

        svc.launch(&AppId("com.cortexos.calculator".into()), &user_id)
            .await
            .unwrap();
        svc.launch(&AppId("com.cortexos.notes".into()), &user_id)
            .await
            .unwrap();

        let running = svc.list_running(&user_id).await.unwrap();
        assert_eq!(running.len(), 2);
    }

    // -- stop_transitions_to_stopped ----------------------------------------

    #[tokio::test]
    async fn stop_transitions_to_stopped() {
        let svc = setup();
        let user_id = test_user_id();

        let instance = svc
            .launch(&AppId("com.cortexos.calculator".into()), &user_id)
            .await
            .unwrap();

        svc.stop(&instance.instance_id).await.unwrap();

        let state = svc.get_state(&instance.instance_id).await.unwrap();
        assert_eq!(state, AppState::Stopped);
    }

    // -- stop_nonexistent_returns_not_found ---------------------------------

    #[tokio::test]
    async fn stop_nonexistent_returns_not_found() {
        let svc = setup();
        let iid = AppInstanceId(Uuid::now_v7());

        let result = svc.stop(&iid).await;
        assert!(matches!(result, Err(RuntimeError::AppNotFound(_))));
    }

    // -- get_state_returns_current_state ------------------------------------

    #[tokio::test]
    async fn get_state_returns_current_state() {
        let svc = setup();
        let user_id = test_user_id();

        let instance = svc
            .launch(&AppId("com.cortexos.calculator".into()), &user_id)
            .await
            .unwrap();

        let state = svc.get_state(&instance.instance_id).await.unwrap();
        assert_eq!(state, AppState::Running);
    }

    // -- get_state_nonexistent_returns_not_found ----------------------------

    #[tokio::test]
    async fn get_state_nonexistent_returns_not_found() {
        let svc = setup();
        let iid = AppInstanceId(Uuid::now_v7());

        let result = svc.get_state(&iid).await;
        assert!(matches!(result, Err(RuntimeError::AppNotFound(_))));
    }

    // -- suspend_transitions_running_to_suspended ---------------------------

    #[tokio::test]
    async fn suspend_transitions_running_to_suspended() {
        let svc = setup();
        let user_id = test_user_id();

        let instance = svc
            .launch(&AppId("com.cortexos.calculator".into()), &user_id)
            .await
            .unwrap();

        svc.suspend(&instance.instance_id).await.unwrap();

        let state = svc.get_state(&instance.instance_id).await.unwrap();
        assert_eq!(state, AppState::Suspended);
    }

    // -- suspend_non_running_returns_error ----------------------------------

    #[tokio::test]
    async fn suspend_non_running_returns_error() {
        let svc = setup();
        let user_id = test_user_id();

        let instance = svc
            .launch(&AppId("com.cortexos.calculator".into()), &user_id)
            .await
            .unwrap();

        // Stop first, then try to suspend.
        svc.stop(&instance.instance_id).await.unwrap();

        let result = svc.suspend(&instance.instance_id).await;
        assert!(matches!(result, Err(RuntimeError::NotRunning(_))));
    }

    // -- resume_transitions_suspended_to_running ----------------------------

    #[tokio::test]
    async fn resume_transitions_suspended_to_running() {
        let svc = setup();
        let user_id = test_user_id();

        let instance = svc
            .launch(&AppId("com.cortexos.calculator".into()), &user_id)
            .await
            .unwrap();

        svc.suspend(&instance.instance_id).await.unwrap();
        let resumed = svc.resume(&instance.instance_id).await.unwrap();

        assert_eq!(resumed.state, AppState::Running);
        assert_eq!(resumed.instance_id, instance.instance_id);
    }

    // -- resume_non_suspended_returns_error ---------------------------------

    #[tokio::test]
    async fn resume_non_suspended_returns_error() {
        let svc = setup();
        let user_id = test_user_id();

        let instance = svc
            .launch(&AppId("com.cortexos.calculator".into()), &user_id)
            .await
            .unwrap();

        // Instance is Running, not Suspended -- resume should fail.
        let result = svc.resume(&instance.instance_id).await;
        assert!(matches!(result, Err(RuntimeError::NotRunning(_))));
    }

    // -- list_running_excludes_stopped_instances -----------------------------

    #[tokio::test]
    async fn list_running_excludes_stopped_instances() {
        let svc = setup();
        let user_id = test_user_id();

        let inst1 = svc
            .launch(&AppId("com.cortexos.calculator".into()), &user_id)
            .await
            .unwrap();
        let _inst2 = svc
            .launch(&AppId("com.cortexos.notes".into()), &user_id)
            .await
            .unwrap();

        // Stop one instance
        svc.stop(&inst1.instance_id).await.unwrap();

        let running = svc.list_running(&user_id).await.unwrap();
        assert_eq!(running.len(), 1);
        assert_eq!(running[0].app_id.0, "com.cortexos.notes");
    }
}
