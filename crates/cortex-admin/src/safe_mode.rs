//! Safe mode detection and management (SPEC 22 §5.8, §6.3).
//!
//! Safe mode boots with only first-party system apps. It is activated by
//! a flag file at `$XDG_CONFIG_HOME/cortexos/safe_mode`.

use std::path::PathBuf;

use crate::error::{AdminError, Result};

/// Safe mode manager.
pub struct SafeMode {
    flag_path: PathBuf,
}

impl SafeMode {
    /// Create a new safe mode manager with the default flag file path.
    pub fn new() -> Self {
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                PathBuf::from(format!("{home}/.config"))
            });
        Self {
            flag_path: config_dir.join("cortexos").join("safe_mode"),
        }
    }

    /// Create a safe mode manager with a custom flag file path (for testing).
    pub fn with_flag_path(flag_path: PathBuf) -> Self {
        Self { flag_path }
    }

    /// Check if the system is currently in safe mode.
    pub fn is_safe_mode(&self) -> bool {
        self.flag_path.exists()
    }

    /// Enable safe mode by creating the flag file.
    pub fn enable(&self) -> Result<()> {
        if let Some(parent) = self.flag_path.parent() {
            std::fs::create_dir_all(parent).map_err(|_e| AdminError::Internal)?;
        }
        std::fs::write(&self.flag_path, "").map_err(|e| {
            tracing::error!(error = %e, path = %self.flag_path.display(), "failed to create safe mode flag");
            AdminError::Internal
        })?;
        tracing::warn!("safe mode flag created");
        Ok(())
    }

    /// Disable safe mode by removing the flag file.
    ///
    /// Called on normal boot, even if the previous boot was normal (SPEC 22 §6.3).
    pub fn disable(&self) -> Result<()> {
        if self.flag_path.exists() {
            std::fs::remove_file(&self.flag_path).map_err(|e| {
                tracing::error!(error = %e, path = %self.flag_path.display(), "failed to remove safe mode flag");
                AdminError::Internal
            })?;
            tracing::info!("safe mode flag removed");
        }
        Ok(())
    }

    /// Get the path to the safe mode flag file.
    pub fn flag_path(&self) -> &std::path::Path {
        &self.flag_path
    }

    /// Filter a list of app IDs to only first-party system apps.
    ///
    /// In safe mode, only these apps are loaded.
    pub fn filter_first_party_apps<'a>(&self, apps: &'a [String]) -> Vec<&'a String> {
        const FIRST_PARTY_SYSTEM_APPS: &[&str] = &[
            "com.cortexos.settings",
            "com.cortexos.terminal-lite",
            "com.cortexos.file-manager",
            "com.cortexos.admin",
        ];
        apps.iter()
            .filter(|app| FIRST_PARTY_SYSTEM_APPS.contains(&app.as_str()))
            .collect()
    }
}

impl Default for SafeMode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_safe_mode_false_when_no_flag() {
        let dir = tempfile::tempdir().unwrap();
        let sm = SafeMode::with_flag_path(dir.path().join("safe_mode"));
        assert!(!sm.is_safe_mode());
    }

    #[test]
    fn enable_creates_flag() {
        let dir = tempfile::tempdir().unwrap();
        let sm = SafeMode::with_flag_path(dir.path().join("safe_mode"));
        sm.enable().unwrap();
        assert!(sm.is_safe_mode());
        assert!(sm.flag_path().exists());
    }

    #[test]
    fn disable_removes_flag() {
        let dir = tempfile::tempdir().unwrap();
        let sm = SafeMode::with_flag_path(dir.path().join("safe_mode"));
        sm.enable().unwrap();
        assert!(sm.is_safe_mode());
        sm.disable().unwrap();
        assert!(!sm.is_safe_mode());
    }

    #[test]
    fn disable_is_noop_when_no_flag() {
        let dir = tempfile::tempdir().unwrap();
        let sm = SafeMode::with_flag_path(dir.path().join("safe_mode"));
        sm.disable().unwrap(); // Should not error.
        assert!(!sm.is_safe_mode());
    }

    #[test]
    fn filter_first_party_apps() {
        let dir = tempfile::tempdir().unwrap();
        let sm = SafeMode::with_flag_path(dir.path().join("safe_mode"));

        let apps = vec![
            "com.cortexos.settings".to_string(),
            "com.cortexos.file-manager".to_string(),
            "com.cortexos.calculator".to_string(),
            "com.example.third-party".to_string(),
            "com.cortexos.terminal-lite".to_string(),
        ];

        let filtered = sm.filter_first_party_apps(&apps);
        assert_eq!(filtered.len(), 3);
        assert!(filtered.contains(&&"com.cortexos.settings".to_string()));
        assert!(filtered.contains(&&"com.cortexos.file-manager".to_string()));
        assert!(filtered.contains(&&"com.cortexos.terminal-lite".to_string()));
        assert!(!filtered.contains(&&"com.cortexos.calculator".to_string()));
    }
}
