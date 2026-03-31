//! Settings domain types.

use cortex_core::Timestamp;
use serde::{Deserialize, Serialize};

/// A setting value. Settings are stored as typed values that serialize to JSON.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SettingValue {
    /// A string value.
    String(String),
    /// A numeric value.
    Number(f64),
    /// A boolean value.
    Boolean(bool),
    /// A nested JSON object value.
    Object(serde_json::Value),
}

impl SettingValue {
    /// Parse a setting value from a raw string representation.
    ///
    /// Tries boolean first, then number, then falls back to string.
    /// Use `serde_json::from_str` for object values.
    pub fn from_str_raw(s: &str) -> Self {
        // Try boolean
        match s {
            "true" => return SettingValue::Boolean(true),
            "false" => return SettingValue::Boolean(false),
            _ => {}
        }

        // Try number
        if let Ok(n) = s.parse::<f64>() {
            return SettingValue::Number(n);
        }

        // Try JSON object/array
        if s.starts_with('{') || s.starts_with('[') {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(s) {
                return SettingValue::Object(val);
            }
        }

        // Fall back to plain string
        SettingValue::String(s.to_owned())
    }
}

/// A stored setting entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SettingEntry {
    /// The namespace this setting belongs to.
    pub namespace: String,
    /// The setting key within the namespace.
    pub key: String,
    /// The setting value.
    pub value: SettingValue,
    /// When the setting was last updated (ISO 8601).
    pub updated_at: Timestamp,
}

/// Metadata for a settings namespace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SettingNamespace {
    /// Namespace identifier (e.g. "ai.providers").
    pub name: String,
    /// Human-readable description of the namespace.
    pub description: String,
}

/// Well-known AI-related setting namespaces.
pub fn ai_setting_namespaces() -> &'static [SettingNamespace] {
    use std::sync::LazyLock;
    static NAMESPACES: LazyLock<Vec<SettingNamespace>> = LazyLock::new(|| {
        vec![
            SettingNamespace {
                name: "ai.providers".to_owned(),
                description: "AI provider credentials and configuration".to_owned(),
            },
            SettingNamespace {
                name: "ai.routing".to_owned(),
                description: "Model routing and fallback rules".to_owned(),
            },
            SettingNamespace {
                name: "ai.budget".to_owned(),
                description: "Usage budgets and rate limits".to_owned(),
            },
            SettingNamespace {
                name: "ai.safety".to_owned(),
                description: "Safety and content filtering rules".to_owned(),
            },
        ]
    });
    &NAMESPACES
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setting_value_string_roundtrip() {
        let val = SettingValue::String("hello world".to_owned());
        let json = serde_json::to_string(&val).unwrap();
        let de: SettingValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, de);
    }

    #[test]
    fn setting_value_number_roundtrip() {
        let val = SettingValue::Number(42.5);
        let json = serde_json::to_string(&val).unwrap();
        let de: SettingValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, de);
    }

    #[test]
    fn setting_value_boolean_roundtrip() {
        let val = SettingValue::Boolean(true);
        let json = serde_json::to_string(&val).unwrap();
        let de: SettingValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, de);
    }

    #[test]
    fn setting_value_object_roundtrip() {
        let obj = serde_json::json!({"key": "value", "nested": {"a": 1}});
        let val = SettingValue::Object(obj);
        let json = serde_json::to_string(&val).unwrap();
        let de: SettingValue = serde_json::from_str(&json).unwrap();
        assert_eq!(val, de);
    }

    #[test]
    fn setting_entry_construction() {
        let entry = SettingEntry {
            namespace: "ai.providers".to_owned(),
            key: "openai.api_key".to_owned(),
            value: SettingValue::String("sk-test".to_owned()),
            updated_at: "2026-03-30T12:00:00Z".to_owned(),
        };
        assert_eq!(entry.namespace, "ai.providers");
        assert_eq!(entry.key, "openai.api_key");
        assert_eq!(entry.value, SettingValue::String("sk-test".to_owned()));
    }

    #[test]
    fn setting_entry_serialization_roundtrip() {
        let entry = SettingEntry {
            namespace: "ai.routing".to_owned(),
            key: "default_provider".to_owned(),
            value: SettingValue::String("openai".to_owned()),
            updated_at: "2026-03-30T12:00:00Z".to_owned(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let de: SettingEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, de);
    }

    #[test]
    fn ai_namespaces_constant_exists() {
        let namespaces = ai_setting_namespaces();
        assert_eq!(namespaces.len(), 4);
        let names: Vec<&str> = namespaces.iter().map(|ns| ns.name.as_str()).collect();
        assert!(names.contains(&"ai.providers"));
        assert!(names.contains(&"ai.routing"));
        assert!(names.contains(&"ai.budget"));
        assert!(names.contains(&"ai.safety"));

        // Every namespace has a non-empty description
        for ns in namespaces {
            assert!(!ns.description.is_empty());
        }
    }

    #[test]
    fn from_str_raw_boolean() {
        assert_eq!(
            SettingValue::from_str_raw("true"),
            SettingValue::Boolean(true)
        );
        assert_eq!(
            SettingValue::from_str_raw("false"),
            SettingValue::Boolean(false)
        );
    }

    #[test]
    fn from_str_raw_number() {
        assert_eq!(SettingValue::from_str_raw("42"), SettingValue::Number(42.0));
        assert_eq!(
            SettingValue::from_str_raw("-2.78"),
            SettingValue::Number(-2.78)
        );
    }

    #[test]
    fn from_str_raw_string_fallback() {
        assert_eq!(
            SettingValue::from_str_raw("hello"),
            SettingValue::String("hello".to_owned())
        );
    }

    #[test]
    fn from_str_raw_json_object() {
        let val = SettingValue::from_str_raw("{\"a\":1}");
        match val {
            SettingValue::Object(_) => {}
            other => panic!("expected Object, got {:?}", other),
        }
    }

    #[test]
    fn setting_namespace_struct() {
        let ns = SettingNamespace {
            name: "test".to_owned(),
            description: "A test namespace".to_owned(),
        };
        assert_eq!(ns.name, "test");
        assert_eq!(ns.description, "A test namespace");
    }
}
