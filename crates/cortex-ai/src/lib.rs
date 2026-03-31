//! CortexOS AI runtime -- provider registry, model routing, and chat API.
//!
//! Provides a provider-agnostic interface for AI completions and chat.
//! Provider-specific code is isolated behind the [`adapter::AiAdapter`] trait.
//! No provider is treated as special in core routing logic.
//!
//! # Crate layout
//!
//! - [`error`]   -- [`AiError`] enum with `is_retryable()` helper
//! - [`types`]   -- domain types (`ChatRequest`, `ChatResponse`, `TokenUsage`, ...)
//! - [`adapter`] -- [`AiAdapter`] trait that every provider backend implements
//! - [`service`] -- [`AiService`] trait that the rest of the OS calls into
//! - [`sqlite`]  -- [`SqliteAiService`] concrete implementation with routing

pub mod adapter;
pub mod error;
pub mod service;
pub mod sqlite;
pub mod types;

pub use adapter::AiAdapter;
pub use error::{AiError, Result};
pub use service::AiService;
pub use sqlite::{SqliteAiService, StubAdapter};
pub use types::{
    ChatMessage, ChatRequest, ChatResponse, ChatRole, ModelInfo, ProviderInfo, TimeoutCategory,
    TokenUsage,
};
