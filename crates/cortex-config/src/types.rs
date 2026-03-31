//! Configuration types.

use crate::error::{ConfigError, Result};

/// Top-level application configuration.
///
/// Constructed once at startup and shared via `Arc<AppConfig>` to all
/// services that need it. No service reads environment variables or
/// config files directly — everything goes through this struct.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// The host address to bind the HTTP server to.
    pub bind_addr: String,
    /// The port to listen on.
    pub port: u16,
    /// Path to the SQLite database file.
    pub database_url: String,
    /// Path to the data directory for file blobs and artifacts.
    pub data_dir: String,
    /// Log level filter (e.g., "info", "debug", "warn", "error", "trace").
    pub log_level: String,
    /// Session token expiry in seconds. Default: 86400 (24h).
    pub session_ttl_secs: u64,
    /// WebSocket ping interval in seconds. Default: 30.
    pub ws_ping_interval_secs: u64,
    /// Maximum file upload size in bytes. Default: 100 MiB.
    pub max_upload_bytes: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0".to_string(),
            port: 8080,
            database_url: "./data/cortex.db".to_string(),
            data_dir: "./data".to_string(),
            log_level: "info".to_string(),
            session_ttl_secs: 86400,
            ws_ping_interval_secs: 30,
            max_upload_bytes: 100 * 1024 * 1024,
        }
    }
}

impl AppConfig {
    /// Load configuration with deterministic precedence:
    /// 1. Built-in defaults
    /// 2. Config file (optional TOML at `path`)
    /// 3. Environment variables (highest priority)
    pub fn load(path: Option<&std::path::Path>) -> Result<Self> {
        let mut config = Self::default();

        // Layer 2: config file
        if let Some(p) = path {
            if p.exists() {
                let content = std::fs::read_to_string(p)?;
                let toml_val: toml::Value = content
                    .parse()
                    .map_err(|e| ConfigError::Validation(format!("invalid config file: {e}")))?;
                if let Some(table) = toml_val.as_table() {
                    if let Some(v) = table.get("bind_addr").and_then(|v| v.as_str()) {
                        config.bind_addr = v.to_string();
                    }
                    if let Some(v) = table.get("port").and_then(|v| v.as_integer()) {
                        config.port = v as u16;
                    }
                    if let Some(v) = table.get("database_url").and_then(|v| v.as_str()) {
                        config.database_url = v.to_string();
                    }
                    if let Some(v) = table.get("data_dir").and_then(|v| v.as_str()) {
                        config.data_dir = v.to_string();
                    }
                    if let Some(v) = table.get("log_level").and_then(|v| v.as_str()) {
                        config.log_level = v.to_string();
                    }
                    if let Some(v) = table.get("session_ttl_secs").and_then(|v| v.as_integer()) {
                        config.session_ttl_secs = v as u64;
                    }
                    if let Some(v) = table
                        .get("ws_ping_interval_secs")
                        .and_then(|v| v.as_integer())
                    {
                        config.ws_ping_interval_secs = v as u64;
                    }
                    if let Some(v) = table.get("max_upload_bytes").and_then(|v| v.as_integer()) {
                        config.max_upload_bytes = v as u64;
                    }
                }
            }
        }

        // Layer 3: environment variables override everything
        if let Ok(v) = std::env::var("CORTEX_BIND_ADDR") {
            config.bind_addr = v;
        }
        if let Ok(v) = std::env::var("CORTEX_PORT") {
            config.port = v
                .parse()
                .map_err(|_| ConfigError::Validation(format!("invalid CORTEX_PORT: {v}")))?;
        }
        if let Ok(v) = std::env::var("CORTEX_DATABASE_URL") {
            config.database_url = v;
        }
        if let Ok(v) = std::env::var("CORTEX_DATA_DIR") {
            config.data_dir = v;
        }
        if let Ok(v) = std::env::var("CORTEX_LOG_LEVEL") {
            config.log_level = v;
        }

        config.validate()?;
        Ok(config)
    }

    /// Validate the config values after loading.
    fn validate(&self) -> Result<()> {
        if self.port == 0 {
            return Err(ConfigError::Validation("port cannot be 0".to_string()));
        }
        if self.data_dir.is_empty() {
            return Err(ConfigError::Validation(
                "data_dir cannot be empty".to_string(),
            ));
        }
        if self.database_url.is_empty() {
            return Err(ConfigError::Validation(
                "database_url cannot be empty".to_string(),
            ));
        }
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.log_level.as_str()) {
            return Err(ConfigError::Validation(format!(
                "invalid log_level '{}', must be one of: {:?}",
                self.log_level, valid_levels
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn default_values() {
        let config = AppConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.session_ttl_secs, 86400);
        assert_eq!(config.ws_ping_interval_secs, 30);
        assert_eq!(config.max_upload_bytes, 100 * 1024 * 1024);
    }

    #[test]
    fn validation_rejects_bad_port() {
        let config = AppConfig {
            port: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn validation_rejects_empty_data_dir() {
        let config = AppConfig {
            data_dir: String::new(),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn validation_rejects_bad_log_level() {
        let config = AppConfig {
            log_level: "verbose".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn load_from_file() {
        let dir = std::env::temp_dir().join("cortex_config_test");
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("test.toml");
        std::fs::write(&path, "port = 9090\nlog_level = \"debug\"\n").unwrap();
        let config = AppConfig::load(Some(&path)).unwrap();
        assert_eq!(config.port, 9090);
        assert_eq!(config.log_level, "debug");
        // defaults preserved where not overridden
        assert_eq!(config.bind_addr, "0.0.0.0");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_without_file_uses_defaults() {
        let config = AppConfig::load(None).unwrap();
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn load_missing_file_uses_defaults() {
        let config =
            AppConfig::load(Some(std::path::Path::new("/nonexistent/config.toml"))).unwrap();
        assert_eq!(config.port, 8080);
    }
}
