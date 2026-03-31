//! Provider adapter trait.
//!
//! Each AI provider (OpenAI, Anthropic, Google, Ollama, Zhipu, custom)
//! implements [`AiAdapter`] so the routing engine can treat them uniformly.
//! No provider SDK is ever imported here -- adapters live behind this trait.

use crate::error::Result;
use crate::types::{ChatRequest, ChatResponse, ModelInfo};

/// Provider-agnostic adapter that every AI backend must implement.
///
/// All methods are async and the trait is `Send + Sync` so adapters can be
/// stored in a registry and shared across tasks.
#[async_trait::async_trait]
pub trait AiAdapter: Send + Sync {
    /// Perform a non-streaming chat completion.
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;

    /// List the models available from this provider.
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;

    /// Return the adapter's unique name (e.g. "openai", "anthropic").
    fn name(&self) -> &str;
}
