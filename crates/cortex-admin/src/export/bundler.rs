//! Diagnostic bundle creation (SPEC 22 §5.7, §10.4).

use crate::error::{AdminError, Result};
use crate::export::redactor::redact_settings;
use crate::types::{DiagnosticBundle, DiagnosticComponent, ExportOptions, SystemInfo};

/// Creates diagnostic export bundles.
pub struct DiagnosticBundler;

impl DiagnosticBundler {
    /// Create a new bundler.
    pub fn new() -> Self {
        Self
    }

    /// Create a diagnostic bundle from system data.
    ///
    /// In a full deployment this would write a ZIP file. For the crate-level
    /// implementation, we build the metadata structure and return it, and
    /// the caller can serialize and write it.
    pub fn create_bundle(
        &self,
        options: &ExportOptions,
        system_info: SystemInfo,
        settings_json: Option<&str>,
    ) -> Result<DiagnosticBundle> {
        // Validate: at least one component must be selected.
        let mut components = Vec::new();
        if options.include_logs {
            components.push(DiagnosticComponent::SystemLogs {
                hours: options.log_hours,
            });
        }
        if options.include_running_apps {
            components.push(DiagnosticComponent::RunningApps);
        }
        if options.include_permissions {
            components.push(DiagnosticComponent::PermissionGrants);
        }
        if options.include_ai_usage {
            components.push(DiagnosticComponent::AiUsageSummary);
        }
        if options.include_crash_logs {
            components.push(DiagnosticComponent::CrashLogs);
        }
        if options.include_settings {
            components.push(DiagnosticComponent::SystemSettings);
        }

        if components.is_empty() {
            return Err(AdminError::ExportFailed(
                "at least one component must be selected for export".to_string(),
            ));
        }

        // Validate log hours.
        if options.log_hours > crate::constants::MAX_EXPORT_LOG_HOURS {
            return Err(AdminError::ExportFailed(format!(
                "log_hours ({}) exceeds maximum ({})",
                options.log_hours,
                crate::constants::MAX_EXPORT_LOG_HOURS
            )));
        }

        // Redact settings if included.
        if let Some(json) = settings_json {
            if options.include_settings {
                let _redacted = redact_settings(json);
                // In a full deployment, we'd add the redacted JSON to the ZIP.
            }
        }

        // Estimate total size (placeholder logic).
        let total_size_bytes = self.estimate_size(options, settings_json);

        Ok(DiagnosticBundle {
            exported_at: chrono::Utc::now().to_rfc3339(),
            system_info,
            included_components: components,
            total_size_bytes,
        })
    }

    /// Estimate the size of the diagnostic bundle before creating it.
    pub fn estimate_size(&self, options: &ExportOptions, settings_json: Option<&str>) -> u64 {
        let mut size: u64 = 0;

        if options.include_logs {
            // Rough estimate: 1KB per log entry × entries per hour × hours.
            size += options.log_hours as u64 * 3600 * 1024;
        }
        if options.include_system_info {
            size += 1024; // Small JSON blob.
        }
        if options.include_running_apps {
            size += 50 * 256; // ~50 apps × 256 bytes each.
        }
        if options.include_permissions {
            size += 100 * 128; // ~100 permissions × 128 bytes each.
        }
        if options.include_ai_usage {
            size += 2048; // Summary statistics.
        }
        if options.include_crash_logs {
            size += 10 * 4096; // Up to 10 crash logs × 4KB each.
        }
        if options.include_settings {
            if let Some(json) = settings_json {
                size += json.len() as u64;
            } else {
                size += 4096;
            }
        }

        size
    }
}

impl Default for DiagnosticBundler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_system_info() -> SystemInfo {
        SystemInfo {
            os_version: "0.1.0".to_string(),
            kernel_version: "6.19.9".to_string(),
            hostname: "test-host".to_string(),
            architecture: "x86_64".to_string(),
            cpu_count: 8,
            total_memory_bytes: 16_000_000_000,
            uptime_seconds: 3600,
        }
    }

    #[test]
    fn create_bundle_with_defaults() {
        let bundler = DiagnosticBundler::new();
        let opts = ExportOptions::default();
        let bundle = bundler
            .create_bundle(&opts, test_system_info(), None)
            .unwrap();

        assert!(!bundle.exported_at.is_empty());
        assert_eq!(bundle.included_components.len(), 6); // all flags enabled; system_info is metadata, not a component
        assert!(bundle.total_size_bytes > 0);
    }

    #[test]
    fn create_bundle_rejects_empty_components() {
        let bundler = DiagnosticBundler::new();
        let opts = ExportOptions {
            include_logs: false,
            include_system_info: false,
            include_running_apps: false,
            include_permissions: false,
            include_ai_usage: false,
            include_crash_logs: false,
            include_settings: false,
            ..ExportOptions::default()
        };
        let result = bundler.create_bundle(&opts, test_system_info(), None);
        assert!(result.is_err());
        match result.unwrap_err() {
            AdminError::ExportFailed(msg) => {
                assert!(msg.contains("at least one component"));
            }
            other => panic!("expected ExportFailed, got: {other}"),
        }
    }

    #[test]
    fn create_bundle_rejects_excessive_log_hours() {
        let bundler = DiagnosticBundler::new();
        let opts = ExportOptions {
            log_hours: 200,
            ..ExportOptions::default()
        };
        let result = bundler.create_bundle(&opts, test_system_info(), None);
        assert!(result.is_err());
    }

    #[test]
    fn create_bundle_with_settings_redaction() {
        let bundler = DiagnosticBundler::new();
        let opts = ExportOptions {
            include_settings: true,
            include_logs: false,
            include_system_info: false,
            include_running_apps: false,
            include_permissions: false,
            include_ai_usage: false,
            include_crash_logs: false,
            ..ExportOptions::default()
        };
        let settings = r#"{"api_key": "sk-secret123", "theme": "dark"}"#;
        let bundle = bundler
            .create_bundle(&opts, test_system_info(), Some(settings))
            .unwrap();

        assert_eq!(bundle.included_components.len(), 1);
    }

    #[test]
    fn estimate_size_is_nonzero() {
        let bundler = DiagnosticBundler::new();
        let opts = ExportOptions::default();
        let size = bundler.estimate_size(&opts, None);
        assert!(size > 0);
    }

    #[test]
    fn estimate_size_reflects_log_hours() {
        let bundler = DiagnosticBundler::new();
        let opts_small = ExportOptions {
            log_hours: 1,
            ..ExportOptions::default()
        };
        let opts_large = ExportOptions {
            log_hours: 168,
            ..ExportOptions::default()
        };
        let small = bundler.estimate_size(&opts_small, None);
        let large = bundler.estimate_size(&opts_large, None);
        assert!(large > small);
    }

    #[test]
    fn estimate_size_includes_settings_json() {
        let bundler = DiagnosticBundler::new();
        let opts = ExportOptions {
            include_settings: true,
            include_logs: false,
            include_system_info: false,
            include_running_apps: false,
            include_permissions: false,
            include_ai_usage: false,
            include_crash_logs: false,
            ..ExportOptions::default()
        };
        let with_settings = bundler.estimate_size(
            &opts,
            Some("{\"key\": \"very_long_value_here_that_adds_bytes\"}"),
        );
        let without_settings = bundler.estimate_size(
            &ExportOptions {
                include_settings: false,
                ..opts.clone()
            },
            None,
        );
        assert!(with_settings > without_settings);
    }
}
