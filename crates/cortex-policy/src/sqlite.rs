//! SQLite-backed implementation of [`PolicyService`].
//!
//! Persists permission grants in the `permission_grants` table managed by
//! migration 0004. Uses the shared [`cortex_db::Pool`] for all reads and
//! writes.

use cortex_core::{AppId, Timestamp, UserId};
use cortex_db::Pool;

use crate::error::{PolicyError, Result};
use crate::service::PolicyService;
use crate::types::{PermissionGrant, PermissionKind};

/// SQLite-backed policy service.
pub struct SqlitePolicyService {
    pool: Pool,
}

impl SqlitePolicyService {
    /// Create a new policy service backed by the given connection pool.
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

/// Convert a [`cortex_db::DbError`] into a [`PolicyError::Internal`].
fn db_err(e: cortex_db::DbError) -> PolicyError {
    PolicyError::Internal(e.to_string())
}

impl PolicyService for SqlitePolicyService {
    fn check_permission(
        &self,
        user_id: &UserId,
        app_id: &AppId,
        permission: &str,
    ) -> impl std::future::Future<Output = Result<bool>> + Send {
        let pool = self.pool.clone();
        let uid = user_id.0.to_string();
        let aid = app_id.0.clone();
        let perm = permission.to_string();

        async move {
            let has_permission = pool
                .read(move |conn| {
                    let count: i64 = conn
                        .query_row(
                            "SELECT COUNT(*) FROM permission_grants \
                             WHERE user_id = ?1 AND app_id = ?2 AND permission = ?3",
                            rusqlite::params![&uid, &aid, &perm],
                            |row| row.get(0),
                        )
                        .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                    Ok(count > 0)
                })
                .map_err(db_err)?;

            Ok(has_permission)
        }
    }

    fn grant(
        &self,
        grant: PermissionGrant,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        let pool = self.pool.clone();
        let grant_id = grant.grant_id;
        let uid = grant.user_id.0.to_string();
        let aid = grant.app_id.0;
        let perm = grant.permission.to_string();
        let granted_at = grant.granted_at;

        async move {
            pool.write(move |conn| {
                conn.execute(
                    "INSERT OR IGNORE INTO permission_grants \
                     (grant_id, user_id, app_id, permission, granted_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![&grant_id, &uid, &aid, &perm, &granted_at],
                )
                .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                Ok(())
            })
            .map_err(db_err)?;

            Ok(())
        }
    }

    fn revoke(&self, grant_id: &str) -> impl std::future::Future<Output = Result<()>> + Send {
        let pool = self.pool.clone();
        let gid = grant_id.to_string();

        async move {
            let rows = pool
                .write(move |conn| {
                    let affected = conn
                        .execute(
                            "DELETE FROM permission_grants WHERE grant_id = ?1",
                            rusqlite::params![&gid],
                        )
                        .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                    Ok(affected)
                })
                .map_err(db_err)?;

            if rows == 0 {
                return Err(PolicyError::GrantNotFound);
            }

            Ok(())
        }
    }

    fn list_grants(
        &self,
        user_id: &UserId,
        app_id: &AppId,
    ) -> impl std::future::Future<Output = Result<Vec<PermissionGrant>>> + Send {
        let pool = self.pool.clone();
        let uid = user_id.0.to_string();
        let aid = app_id.0.clone();

        async move {
            let grants = pool
                .read(move |conn| {
                    let mut stmt = conn
                        .prepare(
                            "SELECT grant_id, user_id, app_id, permission, granted_at \
                             FROM permission_grants \
                             WHERE user_id = ?1 AND app_id = ?2",
                        )
                        .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;

                    let rows = stmt
                        .query_map(rusqlite::params![&uid, &aid], |row| {
                            let grant_id: String = row.get(0)?;
                            let user_id_str: String = row.get(1)?;
                            let app_id_str: String = row.get(2)?;
                            let permission_str: String = row.get(3)?;
                            let granted_at: String = row.get(4)?;
                            Ok((
                                grant_id,
                                user_id_str,
                                app_id_str,
                                permission_str,
                                granted_at,
                            ))
                        })
                        .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;

                    let mut out = Vec::new();
                    for row_res in rows {
                        let (grant_id, user_id_str, app_id_str, permission_str, granted_at) =
                            row_res.map_err(|e| cortex_db::DbError::Query(e.to_string()))?;

                        let uid = uuid::Uuid::parse_str(&user_id_str)
                            .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                        let permission: PermissionKind =
                            permission_str.parse().map_err(cortex_db::DbError::Query)?;

                        out.push(PermissionGrant {
                            grant_id,
                            user_id: UserId(uid),
                            app_id: AppId(app_id_str),
                            permission,
                            granted_at: granted_at as Timestamp,
                        });
                    }
                    Ok(out)
                })
                .map_err(db_err)?;

            Ok(grants)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_db::migration::run_migrations;

    /// Helper: create an in-memory database with all migrations applied.
    fn setup() -> SqlitePolicyService {
        let pool = Pool::open_in_memory().expect("open in-memory db");
        run_migrations(&pool).expect("run migrations");
        SqlitePolicyService::new(pool)
    }

    /// Helper: insert a bare user row so FK constraints are satisfied.
    fn insert_user(pool: &Pool, user_id: &UserId) {
        pool.write(|conn| {
            conn.execute(
                "INSERT OR IGNORE INTO users (user_id, username, password_hash) \
                 VALUES (?1, ?2, ?3)",
                rusqlite::params![&user_id.0.to_string(), "testuser", "irrelevant"],
            )
            .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
            Ok(())
        })
        .expect("insert test user");
    }

    /// Helper: build a [`PermissionGrant`] with sensible defaults.
    fn make_grant(
        grant_id: &str,
        user_id: &UserId,
        app_id: &AppId,
        permission: PermissionKind,
    ) -> PermissionGrant {
        PermissionGrant {
            grant_id: grant_id.to_string(),
            user_id: user_id.clone(),
            app_id: app_id.clone(),
            permission,
            granted_at: "2025-06-15T10:30:00Z".to_string(),
        }
    }

    /// Produce a unique [`UserId`] for tests using a nil UUID variant.
    /// Each call appends a counter suffix to guarantee uniqueness.
    fn test_user_id() -> UserId {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        // Build a deterministic but unique UUID from the counter.
        let mut bytes = uuid::Uuid::nil().into_bytes();
        bytes[0..8].copy_from_slice(&n.to_le_bytes());
        UserId(uuid::Uuid::from_bytes(bytes))
    }

    #[tokio::test]
    async fn grant_then_check_returns_true() {
        let svc = setup();
        let uid = test_user_id();
        let aid = AppId("com.example.app".into());
        insert_user(&svc.pool, &uid);

        svc.grant(make_grant("g1", &uid, &aid, PermissionKind::AiChat))
            .await
            .expect("grant");

        let ok = svc
            .check_permission(&uid, &aid, "ai_chat")
            .await
            .expect("check");
        assert!(ok, "expected permission to be granted");
    }

    #[tokio::test]
    async fn check_permission_non_granted_returns_false() {
        let svc = setup();
        let uid = test_user_id();
        let aid = AppId("com.example.app".into());
        insert_user(&svc.pool, &uid);

        let ok = svc
            .check_permission(&uid, &aid, "files_read")
            .await
            .expect("check");
        assert!(!ok, "expected permission to be absent");
    }

    #[tokio::test]
    async fn revoke_removes_grant() {
        let svc = setup();
        let uid = test_user_id();
        let aid = AppId("com.example.app".into());
        insert_user(&svc.pool, &uid);

        svc.grant(make_grant("g2", &uid, &aid, PermissionKind::FilesRead))
            .await
            .expect("grant");

        // Confirm it is present
        assert!(svc
            .check_permission(&uid, &aid, "files_read")
            .await
            .expect("check before revoke"));

        svc.revoke("g2").await.expect("revoke");

        let ok = svc
            .check_permission(&uid, &aid, "files_read")
            .await
            .expect("check after revoke");
        assert!(!ok, "expected permission to be gone after revoke");
    }

    #[tokio::test]
    async fn revoke_nonexistent_returns_grant_not_found() {
        let svc = setup();
        let err = svc.revoke("does-not-exist").await.unwrap_err();
        assert!(
            matches!(err, PolicyError::GrantNotFound),
            "expected GrantNotFound, got {err:?}"
        );
    }

    #[tokio::test]
    async fn list_grants_returns_correct_grants() {
        let svc = setup();
        let uid = test_user_id();
        let aid = AppId("com.example.app".into());
        insert_user(&svc.pool, &uid);

        svc.grant(make_grant("g10", &uid, &aid, PermissionKind::Read))
            .await
            .expect("grant read");
        svc.grant(make_grant("g11", &uid, &aid, PermissionKind::Write))
            .await
            .expect("grant write");

        let grants = svc.list_grants(&uid, &aid).await.expect("list");
        assert_eq!(grants.len(), 2, "expected 2 grants");

        let perm_strings: Vec<String> = grants.iter().map(|g| g.permission.to_string()).collect();
        assert!(
            perm_strings.contains(&"read".to_string()),
            "expected 'read' in grants"
        );
        assert!(
            perm_strings.contains(&"write".to_string()),
            "expected 'write' in grants"
        );
    }

    #[tokio::test]
    async fn list_grants_empty_returns_empty_vec() {
        let svc = setup();
        let uid = test_user_id();
        let aid = AppId("com.example.app".into());
        insert_user(&svc.pool, &uid);

        let grants = svc.list_grants(&uid, &aid).await.expect("list");
        assert!(grants.is_empty(), "expected no grants");
    }

    #[tokio::test]
    async fn duplicate_grant_is_idempotent() {
        let svc = setup();
        let uid = test_user_id();
        let aid = AppId("com.example.app".into());
        insert_user(&svc.pool, &uid);

        svc.grant(make_grant("g20", &uid, &aid, PermissionKind::Admin))
            .await
            .expect("first grant");

        // Grant again with the same logical key but a different grant_id.
        // INSERT OR IGNORE swallows the duplicate UNIQUE violation.
        svc.grant(make_grant("g20-dup", &uid, &aid, PermissionKind::Admin))
            .await
            .expect("second grant");

        // There should still be exactly one grant for this permission.
        let grants = svc.list_grants(&uid, &aid).await.expect("list");
        assert_eq!(
            grants.len(),
            1,
            "expected exactly one grant after duplicate"
        );
        assert_eq!(grants[0].grant_id, "g20", "original grant should remain");
    }
}
