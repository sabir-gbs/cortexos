//! AI service trait.
//!
//! [`AiService`] is the high-level interface that the rest of CortexOS calls
//! into. The routing engine resolves the correct provider/model based on
//! per-feature, per-app, and global settings before delegating to the
//! appropriate adapter.

use crate::error::Result;
use crate::types::{ChatRequest, ChatResponse, ModelInfo, ProviderInfo};

/// Top-level AI service trait.
///
/// All AI operations go through this trait. The routing engine resolves the
/// provider based on per-feature, per-app, and global settings, then
/// delegates to the matching adapter.
pub trait AiService: Send + Sync {
    /// Send a chat request and receive a complete response.
    ///
    /// The service resolves the provider/model internally using the
    /// five-level precedence from spec 06, section 6.3.
    fn chat(
        &self,
        request: ChatRequest,
    ) -> impl std::future::Future<Output = Result<ChatResponse>> + Send;

    /// List configured AI providers and their status.
    fn list_providers(&self)
        -> impl std::future::Future<Output = Result<Vec<ProviderInfo>>> + Send;

    /// List available AI models, optionally filtered by provider name.
    ///
    /// When `provider` is `Some`, only models from the named provider are
    /// returned. When `None`, models from all providers are returned.
    fn list_models(
        &self,
        provider: Option<&str>,
    ) -> impl std::future::Future<Output = Result<Vec<ModelInfo>>> + Send;
}
