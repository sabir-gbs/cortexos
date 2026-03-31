//! Shared domain types for all CortexOS crates.

use serde::{Deserialize, Serialize};

/// Supported AI providers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiProvider {
    OpenAI,
    Anthropic,
    Google,
    Ollama,
    Zhipu,
    Custom,
}

/// Model routing resolution result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedProvider {
    /// The selected AI provider.
    pub provider: AiProvider,
    /// The model identifier. Empty string means provider default.
    pub model: String,
}
