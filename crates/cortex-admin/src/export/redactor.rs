//! Sensitive value redaction for diagnostic exports (SPEC 22 §13, §21.5).
//!
//! Redacts API keys, passwords, tokens, secrets, auth headers, credentials,
//! and access tokens from JSON settings data. The key name is preserved;
//! the value is replaced with `"[REDACTED]"`.

use regex::Regex;

use crate::constants::REDACTED_VALUE;

/// Patterns that match sensitive key names in JSON. These are compiled once
/// and reused. Matches are case-insensitive.
fn redaction_patterns() -> Vec<Regex> {
    let patterns: &[&str] = &[
        // Matches "key_name": "sensitive_value" (quoted value)
        r#"(?i)"([^"]*(?:key|password|secret|token|auth|credential|api_key|apikey|access_token)[^"]*)"\s*:\s*"[^"]*""#,
        // Matches "key_name": unquoted_value (numbers, booleans, etc.)
        r#"(?i)"([^"]*(?:key|password|secret|token|auth|credential|api_key|apikey|access_token)[^"]*)"\s*:\s*\S+"#,
    ];
    patterns
        .iter()
        .map(|p| Regex::new(p).expect("invalid redaction regex"))
        .collect()
}

/// Redact sensitive values in a JSON string.
///
/// Keys matching common secret patterns have their values replaced with
/// `"[REDACTED]"`. The JSON structure is otherwise preserved.
///
/// # Examples
/// ```ignore
/// let input = r#"{"api_key": "sk-12345", "theme": "dark"}"#;
/// let output = redact_settings(input);
/// assert!(output.contains("\"api_key\": \"[REDACTED]\""));
/// assert!(output.contains("\"theme\": \"dark\""));
/// ```
pub fn redact_settings(settings_json: &str) -> String {
    let patterns = redaction_patterns();
    let mut result = settings_json.to_string();

    for pattern in patterns {
        result = pattern
            .replace_all(&result, |caps: &regex::Captures| {
                let key = &caps[1];
                format!("\"{key}\": \"{REDACTED_VALUE}\"")
            })
            .to_string();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_api_key() {
        let input = r#"{"api_key": "sk-abc123def456"}"#;
        let output = redact_settings(input);
        assert!(output.contains("\"api_key\": \"[REDACTED]\""));
        assert!(!output.contains("sk-abc123def456"));
    }

    #[test]
    fn redacts_password() {
        let input = r#"{"password": "hunter2", "username": "admin"}"#;
        let output = redact_settings(input);
        assert!(output.contains("\"password\": \"[REDACTED]\""));
        assert!(output.contains("\"username\": \"admin\""));
    }

    #[test]
    fn redacts_token() {
        let input = r#"{"access_token": "eyJhbGciOiJIUzI1NiJ9"}"#;
        let output = redact_settings(input);
        assert!(output.contains("\"access_token\": \"[REDACTED]\""));
    }

    #[test]
    fn redacts_secret_case_insensitive() {
        let input = r#"{"Secret": "my_secret_value"}"#;
        let output = redact_settings(input);
        assert!(output.contains("\"Secret\": \"[REDACTED]\""));
    }

    #[test]
    fn redacts_auth_header() {
        let input = r#"{"auth": "Bearer abc123"}"#;
        let output = redact_settings(input);
        assert!(output.contains("\"auth\": \"[REDACTED]\""));
    }

    #[test]
    fn redacts_credential() {
        let input = r#"{"credential": "user:pass"}"#;
        let output = redact_settings(input);
        assert!(output.contains("\"credential\": \"[REDACTED]\""));
    }

    #[test]
    fn preserves_non_sensitive_keys() {
        let input = r#"{"theme": "dark", "fontSize": 14, "language": "en"}"#;
        let output = redact_settings(input);
        assert_eq!(output, input);
    }

    #[test]
    fn redacts_multiple_keys() {
        let input =
            r#"{"api_key": "sk-123", "theme": "dark", "password": "pass", "username": "user"}"#;
        let output = redact_settings(input);
        assert!(output.contains("\"api_key\": \"[REDACTED]\""));
        assert!(output.contains("\"password\": \"[REDACTED]\""));
        assert!(output.contains("\"theme\": \"dark\""));
        assert!(output.contains("\"username\": \"user\""));
    }

    #[test]
    fn redacts_nested_json() {
        let input =
            r#"{"ai": {"provider": "openai", "api_key": "sk-test"}, "ui": {"theme": "light"}}"#;
        let output = redact_settings(input);
        assert!(output.contains("\"api_key\": \"[REDACTED]\""));
        assert!(output.contains("\"provider\": \"openai\""));
        assert!(output.contains("\"theme\": \"light\""));
    }

    #[test]
    fn handles_empty_json() {
        let input = "{}";
        let output = redact_settings(input);
        assert_eq!(output, "{}");
    }

    #[test]
    fn handles_plain_text() {
        let input = "No JSON here, just text";
        let output = redact_settings(input);
        assert_eq!(output, input);
    }

    #[test]
    fn redacts_key_in_compound_name() {
        let input = r#"{"my_api_key_here": "secret123"}"#;
        let output = redact_settings(input);
        assert!(output.contains("\"my_api_key_here\": \"[REDACTED]\""));
    }

    #[test]
    fn redacts_numerical_secret_value() {
        // Test the second pattern that matches non-quoted values.
        let input = r#"{"secret": 12345}"#;
        let output = redact_settings(input);
        // The second regex should match "secret": 12345}
        assert!(output.contains("[REDACTED]"));
        assert!(!output.contains("12345"));
    }
}
