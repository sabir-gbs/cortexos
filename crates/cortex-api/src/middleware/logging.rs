//! Request logging middleware.
//!
//! Framework-agnostic request/response logging.
//! The HTTP layer calls [`RequestLog`] to record structured request info.

use std::time::Instant;

/// A logged HTTP request with timing and metadata.
#[derive(Debug, Clone)]
pub struct RequestLog {
    /// HTTP method (GET, POST, etc.).
    pub method: String,
    /// Request path.
    pub path: String,
    /// Optional authenticated user ID.
    pub user_id: Option<String>,
    /// Request ID for tracing.
    pub request_id: String,
    /// Wall-clock start time.
    pub start: Instant,
}

impl RequestLog {
    /// Create a new request log entry at the start of request processing.
    pub fn new(method: &str, path: &str, request_id: &str) -> Self {
        Self {
            method: method.to_string(),
            path: path.to_string(),
            user_id: None,
            request_id: request_id.to_string(),
            start: Instant::now(),
        }
    }

    /// Attach an authenticated user ID to this log entry.
    pub fn with_user_id(&mut self, user_id: String) {
        self.user_id = Some(user_id);
    }

    /// Record request completion and log the result.
    pub fn finish(&self, status: u16) {
        let elapsed = self.start.elapsed();
        let user = self.user_id.as_deref().unwrap_or("-");
        tracing::info!(
            method = %self.method,
            path = %self.path,
            status = status,
            elapsed_ms = elapsed.as_millis() as u64,
            request_id = %self.request_id,
            user_id = user,
            "request completed"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_log_new() {
        let log = RequestLog::new("GET", "/api/v1/health", "req-001");
        assert_eq!(log.method, "GET");
        assert_eq!(log.path, "/api/v1/health");
        assert_eq!(log.request_id, "req-001");
        assert!(log.user_id.is_none());
    }

    #[test]
    fn request_log_with_user_id() {
        let mut log = RequestLog::new("POST", "/api/v1/files", "req-002");
        log.with_user_id("user-123".to_string());
        assert_eq!(log.user_id.as_deref(), Some("user-123"));
    }

    #[test]
    fn request_log_finish_does_not_panic() {
        let log = RequestLog::new("GET", "/api/v1/health", "req-003");
        log.finish(200);
    }
}
