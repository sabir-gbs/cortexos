//! SQLite-backed implementation of [`SettingsService`].

use std::future::Future;

use cortex_db::Pool;

use crate::error::{Result, SettingsError};
use crate::service::SettingsService;
use crate::types::{SettingEntry, SettingValue};

/// SQLite-backed settings service.
///
/// Stores settings in the `settings` table (created by migration 0005).
/// Values are serialized as JSON strings so that the [`SettingValue`] enum
/// round-trips cleanly through `serde_json`.
pub struct SqliteSettingsService {
    pool: Pool,
}

impl SqliteSettingsService {
    /// Create a new [`SqliteSettingsService`] backed by the given connection pool.
    ///
    /// The caller is responsible for ensuring that migrations have been applied
    /// (i.e. [`cortex_db::run_migrations`] has been called on the pool).
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

/// Convert a rusqlite error into a [`cortex_db::DbError::Query`].
///
/// `rusqlite::Error` does not implement `Send + Sync`, so we capture the
/// display representation as a string.
fn db_err(e: impl std::fmt::Display) -> cortex_db::DbError {
    cortex_db::DbError::Query(format!("{e}"))
}

impl SettingsService for SqliteSettingsService {
    fn get(&self, namespace: &str, key: &str) -> impl Future<Output = Result<SettingEntry>> + Send {
        let pool = self.pool.clone();
        let namespace = namespace.to_owned();
        let key = key.to_owned();

        async move {
            let (value_text, updated_at) = {
                let result = pool.read(|conn| {
                    let result = conn.query_row(
                        "SELECT value, updated_at FROM settings WHERE namespace = ?1 AND key = ?2",
                        [&namespace, &key],
                        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                    );

                    match result {
                        Ok(pair) => Ok(pair),
                        Err(rusqlite::Error::QueryReturnedNoRows) => {
                            Err(cortex_db::DbError::NotFound(format!("{namespace}:{key}")))
                        }
                        Err(e) => Err(db_err(e)),
                    }
                });

                match result {
                    Ok(v) => v,
                    Err(cortex_db::DbError::NotFound(msg)) => {
                        return Err(SettingsError::NotFound(msg));
                    }
                    Err(e) => {
                        return Err(SettingsError::Internal(
                            format!("database error: {e}").into(),
                        ));
                    }
                }
            };

            let value: SettingValue = serde_json::from_str(&value_text).map_err(|e| {
                SettingsError::Internal(format!("value deserialization error: {e}").into())
            })?;

            Ok(SettingEntry {
                namespace,
                key,
                value,
                updated_at,
            })
        }
    }

    fn set(
        &self,
        namespace: &str,
        key: &str,
        value: SettingValue,
    ) -> impl Future<Output = Result<()>> + Send {
        let pool = self.pool.clone();
        let namespace = namespace.to_owned();
        let key = key.to_owned();

        async move {
            let value_json = serde_json::to_string(&value).map_err(|e| {
                SettingsError::Internal(format!("value serialization error: {e}").into())
            })?;

            pool.write(move |conn| {
                conn.execute(
                    "INSERT OR REPLACE INTO settings (namespace, key, value) VALUES (?1, ?2, ?3)",
                    rusqlite::params![namespace, key, value_json],
                )
                .map_err(|e| cortex_db::DbError::Query(format!("insert failed: {e}")))?;
                Ok(())
            })
            .map_err(|e| SettingsError::Internal(format!("database error: {e}").into()))?;

            Ok(())
        }
    }

    fn delete(&self, namespace: &str, key: &str) -> impl Future<Output = Result<()>> + Send {
        let pool = self.pool.clone();
        let namespace = namespace.to_owned();
        let key = key.to_owned();

        async move {
            let namespace_for_err = namespace.clone();
            let key_for_err = key.clone();

            let rows = pool
                .write(move |conn| {
                    let affected = conn
                        .execute(
                            "DELETE FROM settings WHERE namespace = ?1 AND key = ?2",
                            rusqlite::params![namespace, key],
                        )
                        .map_err(|e| cortex_db::DbError::Query(format!("delete failed: {e}")))?;
                    Ok(affected)
                })
                .map_err(|e| SettingsError::Internal(format!("database error: {e}").into()))?;

            if rows == 0 {
                return Err(SettingsError::NotFound(format!(
                    "{namespace_for_err}:{key_for_err}"
                )));
            }

            Ok(())
        }
    }

    fn list_all(&self, namespace: &str) -> impl Future<Output = Result<Vec<SettingEntry>>> + Send {
        let pool = self.pool.clone();
        let namespace = namespace.to_owned();

        async move {
            let raw_entries = pool
                .read(move |conn| {
                    let mut stmt = conn
                        .prepare(
                            "SELECT namespace, key, value, updated_at FROM settings WHERE namespace = ?1",
                        )
                        .map_err(|e| {
                            cortex_db::DbError::Query(format!(
                                "prepare failed: {e}"
                            ))
                        })?;

                    let rows = stmt
                        .query_map(rusqlite::params![namespace], |row| {
                            let ns: String = row.get(0)?;
                            let key: String = row.get(1)?;
                            let value_text: String = row.get(2)?;
                            let updated_at: String = row.get(3)?;
                            Ok((ns, key, value_text, updated_at))
                        })
                        .map_err(|e| {
                            cortex_db::DbError::Query(format!(
                                "query_map failed: {e}"
                            ))
                        })?;

                    let mut out = Vec::new();
                    for row_result in rows {
                        let (ns, key, value_text, updated_at) =
                            row_result.map_err(|e| {
                                cortex_db::DbError::Query(format!(
                                    "row read failed: {e}"
                                ))
                            })?;
                        out.push((ns, key, value_text, updated_at));
                    }

                    Ok(out)
                })
                .map_err(|e| {
                    SettingsError::Internal(
                        format!("database error: {e}").into(),
                    )
                })?;

            let mut entries = Vec::with_capacity(raw_entries.len());
            for (ns, key, value_text, updated_at) in raw_entries {
                let value: SettingValue = serde_json::from_str(&value_text).map_err(|e| {
                    SettingsError::Internal(
                        format!("value deserialization error for {ns}:{key}: {e}").into(),
                    )
                })?;
                entries.push(SettingEntry {
                    namespace: ns,
                    key,
                    value,
                    updated_at,
                });
            }

            Ok(entries)
        }
    }
}

#[cfg(test)]
mod tests {
    use cortex_db::run_migrations;

    use super::*;
    use crate::types::SettingValue;

    fn setup() -> SqliteSettingsService {
        let pool = Pool::open_in_memory().unwrap();
        run_migrations(&pool).unwrap();
        SqliteSettingsService::new(pool)
    }

    #[tokio::test]
    async fn set_and_get_string_value() {
        let svc = setup();
        svc.set("test", "greeting", SettingValue::String("hello".to_owned()))
            .await
            .unwrap();

        let entry = svc.get("test", "greeting").await.unwrap();
        assert_eq!(entry.namespace, "test");
        assert_eq!(entry.key, "greeting");
        assert_eq!(entry.value, SettingValue::String("hello".to_owned()));
        assert!(!entry.updated_at.is_empty());
    }

    #[tokio::test]
    async fn set_and_get_number_value() {
        let svc = setup();
        svc.set("test", "count", SettingValue::Number(42.5))
            .await
            .unwrap();

        let entry = svc.get("test", "count").await.unwrap();
        assert_eq!(entry.value, SettingValue::Number(42.5));
    }

    #[tokio::test]
    async fn set_and_get_boolean_value() {
        let svc = setup();
        svc.set("test", "enabled", SettingValue::Boolean(true))
            .await
            .unwrap();

        let entry = svc.get("test", "enabled").await.unwrap();
        assert_eq!(entry.value, SettingValue::Boolean(true));
    }

    #[tokio::test]
    async fn set_and_get_object_value() {
        let svc = setup();
        let obj = serde_json::json!({"model": "gpt-4", "temperature": 0.7});
        svc.set("test", "config", SettingValue::Object(obj.clone()))
            .await
            .unwrap();

        let entry = svc.get("test", "config").await.unwrap();
        assert_eq!(entry.value, SettingValue::Object(obj));
    }

    #[tokio::test]
    async fn get_nonexistent_returns_not_found() {
        let svc = setup();
        let err = svc.get("missing", "key").await.unwrap_err();
        match err {
            SettingsError::NotFound(msg) => {
                assert!(msg.contains("missing:key"));
            }
            other => panic!("expected NotFound, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn delete_removes_entry() {
        let svc = setup();
        svc.set("test", "temp", SettingValue::String("value".to_owned()))
            .await
            .unwrap();

        svc.delete("test", "temp").await.unwrap();

        let err = svc.get("test", "temp").await.unwrap_err();
        match err {
            SettingsError::NotFound(_) => {}
            other => {
                panic!("expected NotFound after delete, got {:?}", other)
            }
        }
    }

    #[tokio::test]
    async fn delete_nonexistent_returns_not_found() {
        let svc = setup();
        let err = svc.delete("ghost", "key").await.unwrap_err();
        match err {
            SettingsError::NotFound(msg) => {
                assert!(msg.contains("ghost:key"));
            }
            other => panic!("expected NotFound, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn list_all_returns_all_entries_in_namespace() {
        let svc = setup();
        svc.set("ns", "a", SettingValue::String("alpha".to_owned()))
            .await
            .unwrap();
        svc.set("ns", "b", SettingValue::Number(99.0))
            .await
            .unwrap();
        svc.set("ns", "c", SettingValue::Boolean(false))
            .await
            .unwrap();
        // Insert into a different namespace to verify isolation.
        svc.set("other", "x", SettingValue::String("epsilon".to_owned()))
            .await
            .unwrap();

        let entries = svc.list_all("ns").await.unwrap();
        assert_eq!(entries.len(), 3);

        let keys: Vec<&str> = entries.iter().map(|e| e.key.as_str()).collect();
        assert!(keys.contains(&"a"));
        assert!(keys.contains(&"b"));
        assert!(keys.contains(&"c"));
        assert!(!keys.contains(&"x"));
    }

    #[tokio::test]
    async fn list_all_empty_namespace_returns_empty_vec() {
        let svc = setup();
        let entries = svc.list_all("nonexistent").await.unwrap();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn set_overwrites_existing_value() {
        let svc = setup();
        svc.set("test", "key", SettingValue::String("original".to_owned()))
            .await
            .unwrap();

        svc.set("test", "key", SettingValue::Number(7.0))
            .await
            .unwrap();

        let entry = svc.get("test", "key").await.unwrap();
        assert_eq!(entry.value, SettingValue::Number(7.0));
    }
}
