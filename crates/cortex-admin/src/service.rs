//! Admin service trait for diagnostics and recovery.

use crate::error::Result;
use crate::types::{DiagnosticReport, SystemHealth};
use cortex_core::Timestamp;

/// Admin service for system diagnostics, backup, and recovery.
///
/// All admin operations require admin-level permissions.
/// The admin service reads data from subsystem-owned stores,
/// it does NOT create its own shadow persistence.
pub trait AdminService: Send + Sync {
    /// Get the current system health report.
    fn health(&self) -> impl std::future::Future<Output = Result<SystemHealth>> + Send;

    /// Generate a full diagnostic report.
    fn diagnostics(&self) -> impl std::future::Future<Output = Result<DiagnosticReport>> + Send;

    /// Create a backup at the specified path.
    fn backup(&self, path: &str) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Restore from a backup at the specified path.
    fn restore(&self, path: &str) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Export user data for migration.
    fn export_data(
        &self,
        user_id: &str,
        path: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// List active sessions.
    fn list_sessions(&self) -> impl std::future::Future<Output = Result<Vec<SessionInfo>>> + Send;
}

/// Summary info about an active session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub user_id: String,
    pub created_at: Timestamp,
    pub last_active_at: Timestamp,
    pub ip_address: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_info_construction() {
        let info = SessionInfo {
            session_id: "sess-001".to_string(),
            user_id: "user-123".to_string(),
            created_at: "2026-03-30T10:00:00Z".to_string(),
            last_active_at: "2026-03-30T10:05:00Z".to_string(),
            ip_address: Some("192.168.1.1".to_string()),
        };
        assert_eq!(info.session_id, "sess-001");
        assert_eq!(info.user_id, "user-123");
        assert!(info.ip_address.is_some());
    }

    #[test]
    fn session_info_without_ip() {
        let info = SessionInfo {
            session_id: "sess-002".to_string(),
            user_id: "user-456".to_string(),
            created_at: "2026-03-30T11:00:00Z".to_string(),
            last_active_at: "2026-03-30T11:00:30Z".to_string(),
            ip_address: None,
        };
        assert!(info.ip_address.is_none());
    }

    #[test]
    fn session_info_serde_roundtrip() {
        let info = SessionInfo {
            session_id: "sess-003".to_string(),
            user_id: "user-789".to_string(),
            created_at: "2026-03-30T12:00:00Z".to_string(),
            last_active_at: "2026-03-30T12:01:00Z".to_string(),
            ip_address: Some("10.0.0.1".to_string()),
        };
        let json = serde_json::to_string(&info).unwrap();
        let parsed: SessionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info.session_id, parsed.session_id);
        assert_eq!(info.user_id, parsed.user_id);
        assert_eq!(info.created_at, parsed.created_at);
        assert_eq!(info.last_active_at, parsed.last_active_at);
        assert_eq!(info.ip_address, parsed.ip_address);
    }

    /// A minimal stub implementation used only in tests to verify the trait compiles.
    struct StubAdminService;

    impl AdminService for StubAdminService {
        async fn health(&self) -> Result<SystemHealth> {
            Ok(SystemHealth {
                status: cortex_observability::HealthStatus::Healthy,
                components: vec![],
                uptime_secs: 0,
            })
        }

        async fn diagnostics(&self) -> Result<DiagnosticReport> {
            Ok(DiagnosticReport {
                generated_at: "2026-03-30T00:00:00Z".to_string(),
                health: SystemHealth {
                    status: cortex_observability::HealthStatus::Healthy,
                    components: vec![],
                    uptime_secs: 0,
                },
                active_sessions: 0,
                running_apps: 0,
                db_size_bytes: 0,
            })
        }

        async fn backup(&self, _path: &str) -> Result<()> {
            Ok(())
        }

        async fn restore(&self, _path: &str) -> Result<()> {
            Ok(())
        }

        async fn export_data(&self, _user_id: &str, _path: &str) -> Result<()> {
            Ok(())
        }

        async fn list_sessions(&self) -> Result<Vec<SessionInfo>> {
            Ok(vec![])
        }
    }

    #[test]
    fn trait_can_be_implemented() {
        // Verify the concrete type implements the trait.
        // Note: `impl Future` return types make the trait non-dyn-compatible,
        // so we use a concrete reference here.
        fn assert_impl<T: AdminService>() {}
        assert_impl::<StubAdminService>();
    }

    #[tokio::test]
    async fn stub_health_returns_ok() {
        let svc = StubAdminService;
        let health = svc.health().await.unwrap();
        assert_eq!(health.status, cortex_observability::HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn stub_backup_succeeds() {
        let svc = StubAdminService;
        svc.backup("/tmp/test.bak").await.unwrap();
    }

    #[tokio::test]
    async fn stub_restore_succeeds() {
        let svc = StubAdminService;
        svc.restore("/tmp/test.bak").await.unwrap();
    }

    #[tokio::test]
    async fn stub_export_data_succeeds() {
        let svc = StubAdminService;
        svc.export_data("user-1", "/tmp/export.json").await.unwrap();
    }

    #[tokio::test]
    async fn stub_list_sessions_returns_empty() {
        let svc = StubAdminService;
        let sessions = svc.list_sessions().await.unwrap();
        assert!(sessions.is_empty());
    }
}
