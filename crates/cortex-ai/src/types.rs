//! AI runtime domain types.
//!
//! All types are serialisable so they can cross the FFI / API boundary.

use cortex_core::AiProvider;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Chat types
// ---------------------------------------------------------------------------

/// Chat message role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

/// A single message in a chat conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// The role of the message author.
    pub role: ChatRole,
    /// The text content of the message.
    pub content: String,
}

/// A chat completion request sent to an AI provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// The messages in the conversation so far.
    pub messages: Vec<ChatMessage>,
    /// Optional model override. `None` means "use the resolved default".
    pub model: Option<String>,
    /// Optional sampling temperature. `None` means provider default.
    pub temperature: Option<f64>,
    /// Optional cap on the number of generated tokens.
    pub max_tokens: Option<u32>,
}

/// Token usage statistics returned by a provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Tokens consumed by the prompt.
    pub prompt_tokens: u32,
    /// Tokens produced in the completion.
    pub completion_tokens: u32,
    /// Total tokens (prompt + completion).
    pub total_tokens: u32,
}

impl TokenUsage {
    /// Convenience constructor that computes `total_tokens` from the two parts.
    pub fn new(prompt_tokens: u32, completion_tokens: u32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }
}

/// A chat completion response returned from a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// The assistant's reply message.
    pub message: ChatMessage,
    /// Token usage statistics for this request.
    pub usage: TokenUsage,
    /// The model identifier that was actually used.
    pub model: String,
    /// The provider that handled the request.
    pub provider: AiProvider,
}

// ---------------------------------------------------------------------------
// Provider / model metadata
// ---------------------------------------------------------------------------

/// Information about a configured AI provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// The provider type.
    pub provider_type: AiProvider,
    /// Human-readable provider name for display.
    pub display_name: String,
    /// Whether the provider is currently enabled and available.
    pub is_enabled: bool,
    /// Priority for fallback ordering (lower = higher priority).
    pub priority: i32,
}

/// Information about an available AI model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// The model identifier (e.g. "gpt-4o", "claude-sonnet-4-6").
    pub id: String,
    /// Human-readable model name (e.g. "GPT-4o").
    pub display_name: String,
    /// The provider offering this model.
    pub provider: AiProvider,
}

// ---------------------------------------------------------------------------
// Timeout categories
// ---------------------------------------------------------------------------

/// Timeout presets for different classes of AI request.
///
/// Values come from spec 06, section 6.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeoutCategory {
    /// Quick lookups, short prompts -- 5 s.
    Fast,
    /// Normal chat completions -- 30 s.
    Standard,
    /// Long-form generation, code -- 120 s.
    Long,
    /// Streaming sessions -- 300 s.
    Streaming,
}

impl TimeoutCategory {
    /// Return the timeout duration in seconds for this category.
    pub const fn duration_secs(&self) -> u64 {
        match self {
            TimeoutCategory::Fast => 5,
            TimeoutCategory::Standard => 30,
            TimeoutCategory::Long => 120,
            TimeoutCategory::Streaming => 300,
        }
    }

    /// Return the timeout as a [`std::time::Duration`].
    pub const fn duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.duration_secs())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- ChatRole serialization ------------------------------------------------

    #[test]
    fn chat_role_serializes_to_snake_case() {
        let json = serde_json::to_string(&ChatRole::System).unwrap();
        assert_eq!(json, "\"system\"");

        let json = serde_json::to_string(&ChatRole::User).unwrap();
        assert_eq!(json, "\"user\"");

        let json = serde_json::to_string(&ChatRole::Assistant).unwrap();
        assert_eq!(json, "\"assistant\"");
    }

    #[test]
    fn chat_role_deserializes_from_snake_case() {
        let role: ChatRole = serde_json::from_str("\"system\"").unwrap();
        assert_eq!(role, ChatRole::System);

        let role: ChatRole = serde_json::from_str("\"user\"").unwrap();
        assert_eq!(role, ChatRole::User);

        let role: ChatRole = serde_json::from_str("\"assistant\"").unwrap();
        assert_eq!(role, ChatRole::Assistant);
    }

    #[test]
    fn chat_role_roundtrip() {
        for role in [ChatRole::System, ChatRole::User, ChatRole::Assistant] {
            let json = serde_json::to_string(&role).unwrap();
            let back: ChatRole = serde_json::from_str(&json).unwrap();
            assert_eq!(role, back);
        }
    }

    // -- ChatRequest construction ----------------------------------------------

    #[test]
    fn chat_request_minimal() {
        let req = ChatRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: "hello".into(),
            }],
            model: None,
            temperature: None,
            max_tokens: None,
        };
        assert_eq!(req.messages.len(), 1);
        assert!(req.model.is_none());
        assert!(req.temperature.is_none());
        assert!(req.max_tokens.is_none());
    }

    #[test]
    fn chat_request_full() {
        let req = ChatRequest {
            messages: vec![
                ChatMessage {
                    role: ChatRole::System,
                    content: "You are helpful.".into(),
                },
                ChatMessage {
                    role: ChatRole::User,
                    content: "Explain Rust.".into(),
                },
            ],
            model: Some("gpt-4o".into()),
            temperature: Some(0.7),
            max_tokens: Some(1024),
        };
        assert_eq!(req.messages.len(), 2);
        assert_eq!(req.model.as_deref(), Some("gpt-4o"));
        assert_eq!(req.temperature, Some(0.7));
        assert_eq!(req.max_tokens, Some(1024));
    }

    // -- TokenUsage math -------------------------------------------------------

    #[test]
    fn token_usage_new_sums_correctly() {
        let usage = TokenUsage::new(100, 50);
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn token_usage_zero() {
        let usage = TokenUsage::new(0, 0);
        assert_eq!(usage.total_tokens, 0);
    }

    #[test]
    fn token_usage_large_values() {
        let usage = TokenUsage::new(u32::MAX / 2, u32::MAX / 2);
        assert_eq!(usage.total_tokens, u32::MAX - 1);
    }

    // -- TimeoutCategory durations ----------------------------------------------

    #[test]
    fn timeout_category_durations() {
        assert_eq!(TimeoutCategory::Fast.duration_secs(), 5);
        assert_eq!(TimeoutCategory::Standard.duration_secs(), 30);
        assert_eq!(TimeoutCategory::Long.duration_secs(), 120);
        assert_eq!(TimeoutCategory::Streaming.duration_secs(), 300);
    }

    #[test]
    fn timeout_category_duration_objects() {
        assert_eq!(
            TimeoutCategory::Fast.duration(),
            std::time::Duration::from_secs(5)
        );
        assert_eq!(
            TimeoutCategory::Standard.duration(),
            std::time::Duration::from_secs(30)
        );
        assert_eq!(
            TimeoutCategory::Long.duration(),
            std::time::Duration::from_secs(120)
        );
        assert_eq!(
            TimeoutCategory::Streaming.duration(),
            std::time::Duration::from_secs(300)
        );
    }
}
