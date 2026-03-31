//! Policy domain types.

use cortex_core::{AppId, Timestamp, UserId};
use serde::{Deserialize, Serialize};

/// Kinds of permissions that can be granted to apps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionKind {
    /// General read access.
    Read,
    /// General write access.
    Write,
    /// Execute access.
    Execute,
    /// Administrative access.
    Admin,
    /// Permission to send chat messages to the AI.
    AiChat,
    /// Permission to invoke AI actions on behalf of the user.
    AiAction,
    /// Permission to read files.
    FilesRead,
    /// Permission to write / modify files.
    FilesWrite,
    /// Permission to read system or app settings.
    SettingsRead,
    /// Permission to modify system or app settings.
    SettingsWrite,
}

impl PermissionKind {
    /// Return the snake_case string representation used in serialization.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Execute => "execute",
            Self::Admin => "admin",
            Self::AiChat => "ai_chat",
            Self::AiAction => "ai_action",
            Self::FilesRead => "files_read",
            Self::FilesWrite => "files_write",
            Self::SettingsRead => "settings_read",
            Self::SettingsWrite => "settings_write",
        }
    }
}

impl std::fmt::Display for PermissionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Convert a PascalCase / camelCase / SCREAMING_SNAKE_CASE string to
/// lowercase snake_case.
///
/// Example inputs and outputs:
///   `"AiChat"`       -> `"ai_chat"`
///   `"FILES_WRITE"`  -> `"files_write"`
///   `"files_read"`   -> `"files_read"`
fn to_snake_case(input: &str) -> String {
    let trimmed = input.trim();
    let chars: Vec<char> = trimmed.chars().collect();
    let mut out = String::with_capacity(chars.len() + 4);
    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_ascii_uppercase() {
            let prev = if i > 0 { chars[i - 1] } else { '\0' };
            // Insert underscore before this uppercase if:
            // - there is a previous char AND
            // - the previous char is lowercase OR
            // - the previous char is uppercase but the next char is lowercase
            //   (handles "FILESWrite" -> "files_write" at the 'W' boundary)
            if i > 0 {
                let next = chars.get(i + 1).copied().unwrap_or('\0');
                if prev.is_ascii_lowercase()
                    || (prev.is_ascii_uppercase() && next.is_ascii_lowercase())
                {
                    out.push('_');
                }
            }
            out.push(ch.to_ascii_lowercase());
        } else if ch == '_' {
            // Collapse consecutive underscores and avoid leading underscore.
            if !out.ends_with('_') && !out.is_empty() {
                out.push('_');
            }
        } else {
            out.push(ch.to_ascii_lowercase());
        }
    }
    out
}

/// Parse a permission string into a [`PermissionKind`].
///
/// Accepts the same snake_case names used in JSON serialization (e.g.
/// `"files_read"`, `"ai_chat"`). Case-insensitive. Also accepts CamelCase
/// variants like `"AiChat"` or `"FilesRead"` and SCREAMING_SNAKE_CASE like
/// `"FILES_WRITE"`.
impl std::str::FromStr for PermissionKind {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match to_snake_case(s).as_str() {
            "read" => Ok(Self::Read),
            "write" => Ok(Self::Write),
            "execute" => Ok(Self::Execute),
            "admin" => Ok(Self::Admin),
            "ai_chat" => Ok(Self::AiChat),
            "ai_action" => Ok(Self::AiAction),
            "files_read" => Ok(Self::FilesRead),
            "files_write" => Ok(Self::FilesWrite),
            "settings_read" => Ok(Self::SettingsRead),
            "settings_write" => Ok(Self::SettingsWrite),
            other => Err(format!("unknown permission kind: {other}")),
        }
    }
}

/// A single permission entry, scoped to an app.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    /// The app this permission is scoped to.
    pub app_id: AppId,
    /// The permission kind.
    pub permission: PermissionKind,
}

/// A granted permission linking a user and app to a specific permission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionGrant {
    /// Unique grant identifier.
    pub grant_id: String,
    /// The user who holds the grant.
    pub user_id: UserId,
    /// The app the permission is granted to.
    pub app_id: AppId,
    /// The permission that was granted.
    pub permission: PermissionKind,
    /// ISO-8601 timestamp of when the grant was created.
    pub granted_at: Timestamp,
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- to_snake_case helper ---

    #[test]
    fn to_snake_case_conversions() {
        assert_eq!(to_snake_case("AiChat"), "ai_chat");
        assert_eq!(to_snake_case("FilesRead"), "files_read");
        assert_eq!(to_snake_case("FILES_WRITE"), "files_write");
        assert_eq!(to_snake_case("files_read"), "files_read");
        assert_eq!(to_snake_case("Read"), "read");
    }

    // --- PermissionKind serialization round-trip ---

    #[test]
    fn permission_kind_serde_roundtrip() {
        let kinds = [
            PermissionKind::Read,
            PermissionKind::Write,
            PermissionKind::Execute,
            PermissionKind::Admin,
            PermissionKind::AiChat,
            PermissionKind::AiAction,
            PermissionKind::FilesRead,
            PermissionKind::FilesWrite,
            PermissionKind::SettingsRead,
            PermissionKind::SettingsWrite,
        ];

        for kind in &kinds {
            let json = serde_json::to_string(kind).expect("serialize");
            let back: PermissionKind = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(*kind, back, "roundtrip failed for {kind:?}");
        }
    }

    #[test]
    fn permission_kind_serialized_format() {
        assert_eq!(
            serde_json::to_string(&PermissionKind::AiChat).unwrap(),
            r#""ai_chat""#
        );
        assert_eq!(
            serde_json::to_string(&PermissionKind::FilesRead).unwrap(),
            r#""files_read""#
        );
        assert_eq!(
            serde_json::to_string(&PermissionKind::SettingsWrite).unwrap(),
            r#""settings_write""#
        );
    }

    // --- PermissionKind from_str ---

    #[test]
    fn permission_kind_from_str_valid() {
        assert_eq!(
            "read".parse::<PermissionKind>().unwrap(),
            PermissionKind::Read
        );
        assert_eq!(
            "AiChat".parse::<PermissionKind>().unwrap(),
            PermissionKind::AiChat
        );
        assert_eq!(
            "FILES_WRITE".parse::<PermissionKind>().unwrap(),
            PermissionKind::FilesWrite
        );
        assert_eq!(
            " settings_read ".parse::<PermissionKind>().unwrap(),
            PermissionKind::SettingsRead
        );
    }

    #[test]
    fn permission_kind_from_str_invalid() {
        let err = "nope".parse::<PermissionKind>().unwrap_err();
        assert!(err.contains("unknown permission kind"));
    }

    // --- PermissionKind Display ---

    #[test]
    fn permission_kind_display() {
        assert_eq!(PermissionKind::AiChat.to_string(), "ai_chat");
        assert_eq!(PermissionKind::FilesRead.to_string(), "files_read");
    }

    // --- PermissionGrant construction ---

    #[test]
    fn permission_grant_construction() {
        let user_id = UserId(uuid::Uuid::nil());
        let app_id = AppId("com.example.app".into());
        let grant = PermissionGrant {
            grant_id: "grant-001".to_string(),
            user_id: user_id.clone(),
            app_id: app_id.clone(),
            permission: PermissionKind::AiChat,
            granted_at: "2025-06-15T10:30:00Z".to_string(),
        };

        assert_eq!(grant.grant_id, "grant-001");
        assert_eq!(grant.user_id, user_id);
        assert_eq!(grant.app_id, app_id);
        assert_eq!(grant.permission, PermissionKind::AiChat);
        assert_eq!(grant.granted_at, "2025-06-15T10:30:00Z");
    }

    #[test]
    fn permission_grant_serde_roundtrip() {
        let grant = PermissionGrant {
            grant_id: "grant-002".to_string(),
            user_id: UserId(uuid::Uuid::nil()),
            app_id: AppId("com.example.test".into()),
            permission: PermissionKind::FilesWrite,
            granted_at: "2025-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&grant).expect("serialize grant");
        let back: PermissionGrant = serde_json::from_str(&json).expect("deserialize grant");
        assert_eq!(grant, back);
    }

    // --- Error display messages ---

    #[test]
    fn error_display_messages() {
        use crate::error::PolicyError;

        let err = PolicyError::PermissionDenied {
            permission: "ai_chat".to_string(),
        };
        assert_eq!(err.to_string(), "permission denied: ai_chat");

        let err = PolicyError::PolicyViolation {
            reason: "app not allowed".to_string(),
        };
        assert_eq!(err.to_string(), "policy violation: app not allowed");

        let err = PolicyError::GrantNotFound;
        assert_eq!(err.to_string(), "grant not found");

        let err = PolicyError::Internal("db exploded".to_string());
        assert_eq!(err.to_string(), "internal policy error: db exploded");
    }
}
