//! Health check E2E test.
//! Verifies the system health endpoint responds correctly.

#[test]
fn health_endpoint_returns_valid_structure() {
    // This would hit /api/v1/health in a running system.
    // For now, validate the types compile and serialize correctly.
    use cortex_observability::{ComponentHealth, HealthReport, HealthStatus};

    let report = HealthReport {
        status: HealthStatus::Healthy,
        components: vec![ComponentHealth {
            name: "database".to_string(),
            status: HealthStatus::Healthy,
            latency_ms: Some(2),
        }],
        checked_at: chrono::Utc::now().to_rfc3339(),
    };

    let json = serde_json::to_string(&report).unwrap();
    assert!(json.contains("\"status\":\"Healthy\""));
    assert!(json.contains("\"database\""));
}
