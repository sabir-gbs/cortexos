//! CORS middleware.
//!
//! Framework-agnostic CORS configuration and header generation.
//! The HTTP layer calls [`cors_headers`] to append the appropriate
//! headers to every response.

/// CORS configuration.
#[derive(Debug, Clone)]
pub struct CorsConfig {
    /// Comma-separated list of allowed origins. Use `"*"` for any origin.
    pub allowed_origins: Vec<String>,
    /// Comma-separated list of allowed HTTP methods.
    pub allowed_methods: Vec<String>,
    /// Comma-separated list of allowed request headers.
    pub allowed_headers: Vec<String>,
    /// Whether to expose these headers to the browser.
    pub expose_headers: Vec<String>,
    /// Whether credentials (cookies, auth headers) are allowed.
    pub allow_credentials: bool,
    /// Max age for preflight cache in seconds.
    pub max_age_secs: u32,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "PATCH".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "X-Request-Id".to_string(),
            ],
            expose_headers: vec!["X-Request-Id".to_string()],
            allow_credentials: false,
            max_age_secs: 86400,
        }
    }
}

impl CorsConfig {
    /// Create a new CORS config with the given allowed origins.
    pub fn new(allowed_origins: Vec<String>) -> Self {
        Self {
            allowed_origins,
            ..Default::default()
        }
    }

    /// Determine whether the given origin is allowed.
    pub fn is_origin_allowed(&self, origin: &str) -> bool {
        self.allowed_origins.iter().any(|o| o == "*" || o == origin)
    }
}

/// A set of CORS response headers to add to a response.
#[derive(Debug, Clone)]
pub struct CorsHeaders {
    pub access_control_allow_origin: String,
    pub access_control_allow_methods: String,
    pub access_control_allow_headers: String,
    pub access_control_expose_headers: String,
    pub access_control_allow_credentials: bool,
    pub access_control_max_age: u32,
}

/// Compute CORS headers for a request with the given origin.
///
/// Returns `None` if the origin is not allowed.
pub fn cors_headers(config: &CorsConfig, origin: &str) -> Option<CorsHeaders> {
    if !config.is_origin_allowed(origin) {
        return None;
    }

    Some(CorsHeaders {
        access_control_allow_origin: origin.to_string(),
        access_control_allow_methods: config.allowed_methods.join(", "),
        access_control_allow_headers: config.allowed_headers.join(", "),
        access_control_expose_headers: config.expose_headers.join(", "),
        access_control_allow_credentials: config.allow_credentials,
        access_control_max_age: config.max_age_secs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_allows_any_origin() {
        let config = CorsConfig::default();
        assert!(config.is_origin_allowed("https://example.com"));
        assert!(config.is_origin_allowed("http://localhost:3000"));
    }

    #[test]
    fn restricted_origins() {
        let config = CorsConfig::new(vec![
            "https://example.com".to_string(),
            "http://localhost:3000".to_string(),
        ]);
        assert!(config.is_origin_allowed("https://example.com"));
        assert!(!config.is_origin_allowed("https://evil.com"));
    }

    #[test]
    fn cors_headers_for_allowed_origin() {
        let config = CorsConfig::default();
        let headers = cors_headers(&config, "https://example.com").unwrap();
        assert_eq!(headers.access_control_allow_origin, "https://example.com");
        assert!(headers.access_control_allow_methods.contains("GET"));
        assert!(headers
            .access_control_allow_headers
            .contains("Authorization"));
    }

    #[test]
    fn cors_headers_rejects_disallowed_origin() {
        let config = CorsConfig::new(vec!["https://example.com".to_string()]);
        assert!(cors_headers(&config, "https://evil.com").is_none());
    }

    #[test]
    fn wildcard_allows_credentials_false() {
        let config = CorsConfig::default();
        let headers = cors_headers(&config, "*").unwrap();
        assert!(!headers.access_control_allow_credentials);
    }
}
