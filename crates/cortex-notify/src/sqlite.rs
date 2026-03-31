//! SQLite-backed implementation of [`NotifyService`].

use cortex_core::{NotificationId, UserId};
use cortex_db::{DbError, Pool};

use crate::error::{NotifyError, Result};
use crate::service::NotifyService;
use crate::types::Notification;

/// SQLite-backed notification service.
pub struct SqliteNotifyService {
    pool: Pool,
}

impl SqliteNotifyService {
    /// Create a new `SqliteNotifyService` backed by the given connection pool.
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

/// Convert a [`DbError`] into a [`NotifyError`].
fn db_to_notify(e: DbError) -> NotifyError {
    match e {
        DbError::NotFound(msg) => NotifyError::NotFound(msg),
        _ => NotifyError::Internal,
    }
}

impl NotifyService for SqliteNotifyService {
    fn create(
        &self,
        notification: Notification,
    ) -> impl std::future::Future<Output = Result<Notification>> + Send {
        let pool = self.pool.clone();
        async move {
            let notification_id_str = notification.notification_id.0.to_string();
            let user_id_str = notification.user_id.0.to_string();
            let is_read_int: i32 = if notification.is_read { 1 } else { 0 };
            let dismissed_at_str: Option<&str> = notification.dismissed_at.as_deref();

            pool.write(|conn| {
                conn.execute(
                    "INSERT INTO notifications (notification_id, user_id, title, body, category, is_read, created_at, dismissed_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    rusqlite::params![
                        notification_id_str,
                        user_id_str,
                        notification.title,
                        notification.body,
                        notification.category,
                        is_read_int,
                        notification.created_at,
                        dismissed_at_str,
                    ],
                )
                .map_err(|e| DbError::Query(e.to_string()))?;
                Ok(())
            })
            .map_err(db_to_notify)?;

            Ok(notification)
        }
    }

    fn list(
        &self,
        user_id: &UserId,
        limit: Option<u32>,
    ) -> impl std::future::Future<Output = Result<Vec<Notification>>> + Send {
        let pool = self.pool.clone();
        let user_id_str = user_id.0.to_string();
        let limit = limit.unwrap_or(u32::MAX);
        async move {
            let sql = "SELECT notification_id, user_id, title, body, category, is_read, created_at, dismissed_at
                       FROM notifications
                       WHERE user_id = ?1
                       ORDER BY created_at DESC
                       LIMIT ?2";

            let notifications = pool
                .read(|conn| {
                    let mut stmt = conn
                        .prepare(sql)
                        .map_err(|e| DbError::Query(e.to_string()))?;
                    let rows = stmt
                        .query_map(rusqlite::params![user_id_str, limit], |row| {
                            let notification_id_str: String = row.get(0)?;
                            let uid_str: String = row.get(1)?;
                            let is_read_int: i32 = row.get(5)?;
                            let dismissed_at: Option<String> = row.get(7)?;

                            Ok(Notification {
                                notification_id: NotificationId(
                                    notification_id_str.parse::<uuid::Uuid>().map_err(|_| {
                                        rusqlite::Error::InvalidParameterName("bad uuid".into())
                                    })?,
                                ),
                                user_id: UserId(uid_str.parse::<uuid::Uuid>().map_err(|_| {
                                    rusqlite::Error::InvalidParameterName("bad uuid".into())
                                })?),
                                title: row.get(2)?,
                                body: row.get(3)?,
                                category: row.get(4)?,
                                is_read: is_read_int != 0,
                                created_at: row.get(6)?,
                                dismissed_at,
                            })
                        })
                        .map_err(|e| DbError::Query(e.to_string()))?;

                    let mut result = Vec::new();
                    for notif in rows {
                        result.push(notif.map_err(|e| DbError::Query(e.to_string()))?);
                    }
                    Ok(result)
                })
                .map_err(db_to_notify)?;

            Ok(notifications)
        }
    }

    fn mark_read(
        &self,
        id: &NotificationId,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        let pool = self.pool.clone();
        let id_str = id.0.to_string();
        async move {
            let affected = pool
                .write(|conn| {
                    let rows = conn
                        .execute(
                            "UPDATE notifications SET is_read = 1 WHERE notification_id = ?1",
                            rusqlite::params![id_str],
                        )
                        .map_err(|e| DbError::Query(e.to_string()))?;
                    Ok(rows)
                })
                .map_err(db_to_notify)?;

            if affected == 0 {
                return Err(NotifyError::NotFound(format!("notification {id_str}")));
            }
            Ok(())
        }
    }

    fn dismiss(&self, id: &NotificationId) -> impl std::future::Future<Output = Result<()>> + Send {
        let pool = self.pool.clone();
        let id_str = id.0.to_string();
        async move {
            let affected = pool
                .write(|conn| {
                    let rows = conn
                        .execute(
                            "UPDATE notifications SET dismissed_at = datetime('now') WHERE notification_id = ?1",
                            rusqlite::params![id_str],
                        )
                        .map_err(|e| DbError::Query(e.to_string()))?;
                    Ok(rows)
                })
                .map_err(db_to_notify)?;

            if affected == 0 {
                return Err(NotifyError::NotFound(format!("notification {id_str}")));
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_db::run_migrations;
    use uuid::Uuid;

    /// Helper: create an in-memory pool with all migrations applied.
    fn setup_pool() -> Pool {
        let pool = Pool::open_in_memory().unwrap();
        run_migrations(&pool).unwrap();
        pool
    }

    /// Helper: insert a test user row so FK constraints are satisfied.
    fn insert_test_user(pool: &Pool, user_id: &UserId) {
        let uid = user_id.0.to_string();
        pool.write(|conn| {
            conn.execute(
                "INSERT OR IGNORE INTO users (user_id, username, password_hash) VALUES (?1, ?2, ?3)",
                rusqlite::params![uid, "testuser", "fakehash"],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;
            Ok(())
        })
        .unwrap();
    }

    /// Helper: build a notification with sensible defaults.
    fn make_notification(user_id: &UserId) -> Notification {
        Notification {
            notification_id: NotificationId(Uuid::now_v7()),
            user_id: user_id.clone(),
            title: "Test notification".to_string(),
            body: "Hello world".to_string(),
            category: "system".to_string(),
            is_read: false,
            created_at: "2026-03-30T12:00:00Z".to_string(),
            dismissed_at: None,
        }
    }

    #[tokio::test]
    async fn create_and_list_notifications() {
        let pool = setup_pool();
        let user_id = UserId(Uuid::now_v7());
        insert_test_user(&pool, &user_id);

        let svc = SqliteNotifyService::new(pool);
        let notif1 = Notification {
            created_at: "2026-03-30T12:00:00Z".to_string(),
            ..make_notification(&user_id)
        };
        let notif2 = Notification {
            notification_id: NotificationId(Uuid::now_v7()),
            title: "Second".to_string(),
            created_at: "2026-03-30T13:00:00Z".to_string(),
            ..make_notification(&user_id)
        };

        svc.create(notif1.clone()).await.unwrap();
        svc.create(notif2.clone()).await.unwrap();

        let listed = svc.list(&user_id, None).await.unwrap();
        assert_eq!(listed.len(), 2);
        // Ordered by created_at DESC so notif2 (13:00) comes first.
        assert_eq!(listed[0].notification_id, notif2.notification_id);
        assert_eq!(listed[1].notification_id, notif1.notification_id);
    }

    #[tokio::test]
    async fn list_with_limit() {
        let pool = setup_pool();
        let user_id = UserId(Uuid::now_v7());
        insert_test_user(&pool, &user_id);

        let svc = SqliteNotifyService::new(pool);

        for _ in 0..5 {
            let notif = make_notification(&user_id);
            svc.create(notif).await.unwrap();
        }

        let listed = svc.list(&user_id, Some(3)).await.unwrap();
        assert_eq!(listed.len(), 3);
    }

    #[tokio::test]
    async fn list_returns_empty_for_user_with_no_notifications() {
        let pool = setup_pool();
        let user_id = UserId(Uuid::now_v7());
        insert_test_user(&pool, &user_id);

        let other_user_id = UserId(Uuid::now_v7());
        insert_test_user(&pool, &other_user_id);

        let svc = SqliteNotifyService::new(pool);

        // Create a notification for user_id
        let notif = make_notification(&user_id);
        svc.create(notif).await.unwrap();

        // Listing for other_user_id should return empty
        let listed = svc.list(&other_user_id, None).await.unwrap();
        assert!(listed.is_empty());
    }

    #[tokio::test]
    async fn mark_read_sets_is_read_to_true() {
        let pool = setup_pool();
        let user_id = UserId(Uuid::now_v7());
        insert_test_user(&pool, &user_id);

        let svc = SqliteNotifyService::new(pool);
        let notif = make_notification(&user_id);
        let notif_id = notif.notification_id.clone();

        svc.create(notif).await.unwrap();
        svc.mark_read(&notif_id).await.unwrap();

        let listed = svc.list(&user_id, None).await.unwrap();
        assert_eq!(listed.len(), 1);
        assert!(listed[0].is_read);
    }

    #[tokio::test]
    async fn mark_read_nonexistent_returns_not_found() {
        let pool = setup_pool();
        let svc = SqliteNotifyService::new(pool);

        let fake_id = NotificationId(Uuid::now_v7());
        let result = svc.mark_read(&fake_id).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            NotifyError::NotFound(msg) => assert!(msg.contains("notification")),
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn dismiss_sets_dismissed_at() {
        let pool = setup_pool();
        let user_id = UserId(Uuid::now_v7());
        insert_test_user(&pool, &user_id);

        let svc = SqliteNotifyService::new(pool);
        let notif = make_notification(&user_id);
        let notif_id = notif.notification_id.clone();

        svc.create(notif).await.unwrap();
        svc.dismiss(&notif_id).await.unwrap();

        let listed = svc.list(&user_id, None).await.unwrap();
        assert_eq!(listed.len(), 1);
        assert!(listed[0].dismissed_at.is_some());
    }

    #[tokio::test]
    async fn dismiss_nonexistent_returns_not_found() {
        let pool = setup_pool();
        let svc = SqliteNotifyService::new(pool);

        let fake_id = NotificationId(Uuid::now_v7());
        let result = svc.dismiss(&fake_id).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            NotifyError::NotFound(msg) => assert!(msg.contains("notification")),
            other => panic!("expected NotFound, got {other:?}"),
        }
    }
}
