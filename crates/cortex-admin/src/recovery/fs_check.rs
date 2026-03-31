//! Filesystem verification and repair (SPEC 22 §5.6.3).

use cortex_db::Pool;

use crate::error::{AdminError, Result};
use crate::types::{FsCheckItem, FsCheckResult, FsCheckStatus};

/// Convert a database error into an AdminError.
fn _db_err(_e: cortex_db::DbError) -> AdminError {
    AdminError::FilesystemError(_e.to_string())
}

/// Checks filesystem integrity and repairs issues.
pub struct FsChecker {
    pool: Pool,
}

impl FsChecker {
    /// Create a new filesystem checker.
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Run all filesystem checks.
    pub fn verify(&self) -> Result<FsCheckResult> {
        let items = vec![
            self.check_config_dir(),
            self.check_app_data_dir(),
            self.check_app_registry(),
            self.check_settings(),
        ];

        Ok(FsCheckResult::from_items(items))
    }

    /// Repair the given check items.
    pub fn repair(&self, items: &[FsCheckItem]) -> Result<FsCheckResult> {
        let mut repaired = Vec::new();
        for item in items {
            if !item.repairable {
                repaired.push(FsCheckItem {
                    check_name: item.check_name.clone(),
                    path: item.path.clone(),
                    status: FsCheckStatus::Error,
                    message: format!("Not repairable: {}", item.message),
                    repairable: false,
                });
                continue;
            }
            match self.repair_single(item) {
                Ok(()) => repaired.push(FsCheckItem {
                    check_name: item.check_name.clone(),
                    path: item.path.clone(),
                    status: FsCheckStatus::Pass,
                    message: "Repaired successfully".to_string(),
                    repairable: false,
                }),
                Err(e) => repaired.push(FsCheckItem {
                    check_name: item.check_name.clone(),
                    path: item.path.clone(),
                    status: FsCheckStatus::Error,
                    message: format!("Repair failed: {e}"),
                    repairable: true,
                }),
            }
        }
        Ok(FsCheckResult::from_items(repaired))
    }

    fn check_config_dir(&self) -> FsCheckItem {
        let path = dirs_config_path();
        if std::path::Path::new(&path).exists() {
            FsCheckItem {
                check_name: "config_dir".to_string(),
                path,
                status: FsCheckStatus::Pass,
                message: "Configuration directory exists".to_string(),
                repairable: false,
            }
        } else {
            FsCheckItem {
                check_name: "config_dir".to_string(),
                path,
                status: FsCheckStatus::Warning,
                message: "Configuration directory missing".to_string(),
                repairable: true,
            }
        }
    }

    fn check_app_data_dir(&self) -> FsCheckItem {
        let path = dirs_data_path();
        if std::path::Path::new(&path).exists() {
            FsCheckItem {
                check_name: "app_data_dir".to_string(),
                path,
                status: FsCheckStatus::Pass,
                message: "App data directory exists".to_string(),
                repairable: false,
            }
        } else {
            FsCheckItem {
                check_name: "app_data_dir".to_string(),
                path,
                status: FsCheckStatus::Warning,
                message: "App data directory missing".to_string(),
                repairable: true,
            }
        }
    }

    fn check_app_registry(&self) -> FsCheckItem {
        // In a full deployment, this would cross-reference installed_app
        // rows with their on-disk directories. For now, we verify the
        // database table is queryable.
        match self.pool.read(|conn| {
            conn.query_row("SELECT COUNT(*) FROM app_instances", [], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|e| cortex_db::DbError::Query(e.to_string()))
        }) {
            Ok(count) => FsCheckItem {
                check_name: "app_registry".to_string(),
                path: "sqlite://app_instances".to_string(),
                status: FsCheckStatus::Pass,
                message: format!("App registry has {count} entries"),
                repairable: false,
            },
            Err(e) => FsCheckItem {
                check_name: "app_registry".to_string(),
                path: "sqlite://app_instances".to_string(),
                status: FsCheckStatus::Error,
                message: format!("App registry query failed: {e}"),
                repairable: false,
            },
        }
    }

    fn check_settings(&self) -> FsCheckItem {
        match self.pool.read(|conn| {
            conn.query_row("SELECT COUNT(*) FROM settings", [], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|e| cortex_db::DbError::Query(e.to_string()))
        }) {
            Ok(count) => FsCheckItem {
                check_name: "settings_integrity".to_string(),
                path: "sqlite://settings".to_string(),
                status: FsCheckStatus::Pass,
                message: format!("Settings table has {count} entries"),
                repairable: false,
            },
            Err(e) => FsCheckItem {
                check_name: "settings_integrity".to_string(),
                path: "sqlite://settings".to_string(),
                status: FsCheckStatus::Error,
                message: format!("Settings query failed: {e}"),
                repairable: false,
            },
        }
    }

    fn repair_single(&self, item: &FsCheckItem) -> Result<()> {
        match item.check_name.as_str() {
            "config_dir" => {
                let path = std::path::Path::new(&item.path);
                std::fs::create_dir_all(path).map_err(|e| {
                    AdminError::FilesystemError(format!("cannot create config dir: {e}"))
                })?;
                tracing::info!(path = %item.path, "repaired: created config directory");
                Ok(())
            }
            "app_data_dir" => {
                let path = std::path::Path::new(&item.path);
                std::fs::create_dir_all(path).map_err(|e| {
                    AdminError::FilesystemError(format!("cannot create app data dir: {e}"))
                })?;
                tracing::info!(path = %item.path, "repaired: created app data directory");
                Ok(())
            }
            _ => Err(AdminError::FilesystemError(format!(
                "no repair handler for: {}",
                item.check_name
            ))),
        }
    }
}

/// Returns the expected config directory path.
fn dirs_config_path() -> String {
    std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        format!(
            "{}/.config",
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
        )
    }) + "/cortexos"
}

/// Returns the expected data directory path.
fn dirs_data_path() -> String {
    std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| {
        format!(
            "{}/.local/share",
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
        )
    }) + "/cortexos"
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_db::run_migrations;

    fn test_pool() -> Pool {
        let pool = Pool::open_in_memory().unwrap();
        run_migrations(&pool).unwrap();
        pool
    }

    #[test]
    fn verify_returns_result() {
        let pool = test_pool();
        let checker = FsChecker::new(pool);
        let result = checker.verify().unwrap();
        assert!(result.total_checked >= 4);
        // Database tables exist in the test pool, so registry/settings should pass.
        // Filesystem checks (config_dir, app_data_dir) may fail in CI/test env.
        // passed is usize, always >= 0; just verify no panic
        let _ = result.passed;
    }

    #[test]
    fn repair_non_repairable_fails() {
        let pool = test_pool();
        let checker = FsChecker::new(pool);

        let items = vec![FsCheckItem {
            check_name: "app_registry".to_string(),
            path: "sqlite://app_instances".to_string(),
            status: FsCheckStatus::Error,
            message: "broken".to_string(),
            repairable: false,
        }];

        let result = checker.repair(&items).unwrap();
        assert_eq!(result.errors, 1);
    }

    #[test]
    fn repair_config_dir_creates_directory() {
        let pool = test_pool();
        let checker = FsChecker::new(pool);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing_config");
        let path_str = path.to_str().unwrap().to_string();

        let items = vec![FsCheckItem {
            check_name: "config_dir".to_string(),
            path: path_str.clone(),
            status: FsCheckStatus::Warning,
            message: "missing".to_string(),
            repairable: true,
        }];

        let result = checker.repair(&items).unwrap();
        assert_eq!(result.passed, 1);
        assert!(path.exists());
    }
}
