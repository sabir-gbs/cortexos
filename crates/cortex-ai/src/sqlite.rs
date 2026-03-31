//! SQLite-backed AI service with adapter routing and chat history persistence.
//!
//! [`SqliteAiService`] is the concrete implementation of [`AiService`] that:
//! - Maintains a registry of [`AiAdapter`] instances (injected at construction)
//! - Reads provider configuration from the `ai_providers` table
//! - Routes chat requests to the highest-priority enabled adapter
//! - Persists chat messages to `ai_chat_history`
//!
//! [`StubAdapter`] provides canned responses for testing.

use cortex_core::AiProvider;
use cortex_db::types::Pool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::adapter::AiAdapter;
use crate::error::{AiError, Result};
use crate::service::AiService;
use crate::types::{ChatMessage, ChatRequest, ChatResponse, ChatRole, ModelInfo, ProviderInfo};

// ---------------------------------------------------------------------------
// Row helper types (not exported)
// ---------------------------------------------------------------------------

/// Row from the `ai_providers` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProviderRow {
    provider_id: String,
    provider_type: String,
    display_name: String,
    is_enabled: bool,
    priority: i32,
}

// ---------------------------------------------------------------------------
// StubAdapter
// ---------------------------------------------------------------------------

/// A test adapter that returns canned responses.
///
/// Useful in unit tests and integration tests where a real provider backend
/// is not available.
pub struct StubAdapter {
    provider: AiProvider,
    model_id: String,
    response_text: String,
}

impl StubAdapter {
    /// Create a new stub adapter.
    pub fn new(provider: AiProvider, model_id: &str, response_text: &str) -> Self {
        Self {
            provider,
            model_id: model_id.to_owned(),
            response_text: response_text.to_owned(),
        }
    }
}

#[async_trait::async_trait]
impl AiAdapter for StubAdapter {
    async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse> {
        Ok(ChatResponse {
            message: ChatMessage {
                role: ChatRole::Assistant,
                content: self.response_text.clone(),
            },
            usage: crate::types::TokenUsage::new(10, 20),
            model: self.model_id.clone(),
            provider: self.provider.clone(),
        })
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        Ok(vec![ModelInfo {
            id: self.model_id.clone(),
            display_name: self.model_id.clone(),
            provider: self.provider.clone(),
        }])
    }

    fn name(&self) -> &str {
        match self.provider {
            AiProvider::OpenAI => "openai",
            AiProvider::Anthropic => "anthropic",
            AiProvider::Google => "google",
            AiProvider::Ollama => "ollama",
            AiProvider::Zhipu => "zhipu",
            AiProvider::Custom => "custom",
        }
    }
}

// ---------------------------------------------------------------------------
// SqliteAiService
// ---------------------------------------------------------------------------

/// SQLite-backed implementation of [`AiService`].
pub struct SqliteAiService {
    pool: Pool,
    adapters: Vec<Box<dyn AiAdapter>>,
}

impl SqliteAiService {
    /// Create a new service with the given connection pool and no adapters.
    pub fn new(pool: Pool) -> Self {
        Self {
            pool,
            adapters: Vec::new(),
        }
    }

    /// Create a new service with pre-registered adapters.
    pub fn with_adapters(pool: Pool, adapters: Vec<Box<dyn AiAdapter>>) -> Self {
        Self { pool, adapters }
    }

    /// Register an adapter at runtime.
    pub fn register_adapter(&mut self, adapter: Box<dyn AiAdapter>) {
        self.adapters.push(adapter);
    }

    /// Persist a chat message to history.
    pub fn save_message(
        &self,
        user_id: &cortex_core::UserId,
        conversation_id: &str,
        message: &ChatMessage,
        model: &str,
        provider: &str,
        token_count: u32,
    ) -> Result<()> {
        let message_id = Uuid::now_v7().to_string();
        let user_id_str = user_id.0.to_string();
        let role_str = match message.role {
            ChatRole::System => "system",
            ChatRole::User => "user",
            ChatRole::Assistant => "assistant",
        };
        let content = message.content.clone();

        self.pool
            .write(|conn| {
                conn.execute(
                    "INSERT INTO ai_chat_history (message_id, user_id, conversation_id, role, content, model, provider, token_count)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    rusqlite::params![
                        message_id,
                        user_id_str,
                        conversation_id,
                        role_str,
                        content,
                        model,
                        provider,
                        token_count,
                    ],
                )
                .map_err(|e| cortex_db::DbError::Query(format!("insert chat message: {e}")))?;
                Ok(())
            })
            .map_err(|_| AiError::Internal)
    }

    /// Get chat history for a conversation.
    pub fn get_history(&self, conversation_id: &str) -> Result<Vec<ChatMessage>> {
        self.pool
            .read(|conn| {
                let mut stmt = conn
                    .prepare(
                        "SELECT role, content FROM ai_chat_history
                         WHERE conversation_id = ?1
                         ORDER BY created_at ASC",
                    )
                    .map_err(|e| cortex_db::DbError::Query(format!("prepare get_history: {e}")))?;

                let rows = stmt
                    .query_map(rusqlite::params![conversation_id], |row| {
                        let role_str: String = row.get(0)?;
                        let content: String = row.get(1)?;
                        Ok((role_str, content))
                    })
                    .map_err(|e| cortex_db::DbError::Query(format!("query get_history: {e}")))?;

                let mut messages = Vec::new();
                for row_result in rows {
                    let (role_str, content) =
                        row_result.map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                    let role = match role_str.as_str() {
                        "system" => ChatRole::System,
                        "user" => ChatRole::User,
                        "assistant" => ChatRole::Assistant,
                        other => {
                            return Err(cortex_db::DbError::Query(format!("unknown role: {other}")))
                        }
                    };
                    messages.push(ChatMessage { role, content });
                }
                Ok(messages)
            })
            .map_err(|_| AiError::Internal)
    }

    /// Configure a provider in the database.
    pub fn configure_provider(&self, provider: &ProviderInfo) -> Result<()> {
        // Serde serialises AiProvider as "openai", "anthropic", etc.
        let provider_type = serde_json::to_string(&provider.provider_type)
            .unwrap_or_else(|_| "\"custom\"".to_owned());
        let provider_id = serde_json::to_value(&provider.provider_type)
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_owned()))
            .unwrap_or_else(|| "custom".to_owned());
        let display_name = provider.display_name.clone();
        let is_enabled = provider.is_enabled as i32;
        let priority = provider.priority;

        self.pool
            .write(|conn| {
                conn.execute(
                    "INSERT INTO ai_providers (provider_id, provider_type, display_name, is_enabled, priority)
                     VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(provider_id) DO UPDATE SET
                         display_name = excluded.display_name,
                         is_enabled = excluded.is_enabled,
                         priority = excluded.priority",
                    rusqlite::params![provider_id, provider_type, display_name, is_enabled, priority],
                )
                .map_err(|e| cortex_db::DbError::Query(format!("configure provider: {e}")))?;
                Ok(())
            })
            .map_err(|_| AiError::Internal)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Read enabled providers from the database, ordered by priority (ascending).
    fn read_enabled_providers(&self) -> Result<Vec<ProviderRow>> {
        self.pool
            .read(|conn| {
                let mut stmt = conn
                    .prepare(
                        "SELECT provider_id, provider_type, display_name, is_enabled, priority
                         FROM ai_providers
                         WHERE is_enabled = 1
                         ORDER BY priority ASC",
                    )
                    .map_err(|e| cortex_db::DbError::Query(format!("prepare providers: {e}")))?;

                let rows = stmt
                    .query_map([], |row| {
                        Ok(ProviderRow {
                            provider_id: row.get(0)?,
                            provider_type: row.get(1)?,
                            display_name: row.get(2)?,
                            is_enabled: row.get::<_, i32>(3)? == 1,
                            priority: row.get(4)?,
                        })
                    })
                    .map_err(|e| cortex_db::DbError::Query(format!("query providers: {e}")))?;

                let mut providers = Vec::new();
                for row_result in rows {
                    providers
                        .push(row_result.map_err(|e| cortex_db::DbError::Query(e.to_string()))?);
                }
                Ok(providers)
            })
            .map_err(|_| AiError::Internal)
    }

    /// Find the best adapter for the given request.
    ///
    /// If a model is specified in the request, search adapters for one that
    /// lists that model. Otherwise, pick the adapter whose name matches the
    /// highest-priority enabled provider. Falls back to the first adapter if
    /// no database config exists.
    fn resolve_adapter(&self, request: &ChatRequest) -> Option<&dyn AiAdapter> {
        let providers = self.read_enabled_providers().unwrap_or_default();

        // If a specific model is requested, find an adapter that provides it.
        // We do a best-effort match: iterate adapters and pick the first that
        // either lists the model or whose name matches a provider row.
        if let Some(ref model) = request.model {
            for adapter in &self.adapters {
                // We cannot call async list_models here in a sync context.
                // Instead, do a simple name match: if the adapter name appears
                // in the model id or vice-versa, prefer it.
                if model.contains(adapter.name()) || adapter.name().contains(model.as_str()) {
                    return Some(adapter.as_ref());
                }
            }
            // Fallback: try each adapter -- check if model ID matches the
            // adapter's known naming pattern. For stubs the model_id is exact.
        }

        // Route by priority from database config.
        for provider_row in &providers {
            for adapter in &self.adapters {
                if adapter.name() == provider_row.provider_type {
                    return Some(adapter.as_ref());
                }
            }
        }

        // If no database config matches, use the first registered adapter.
        self.adapters.first().map(|a| a.as_ref())
    }
}

impl AiService for SqliteAiService {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let adapter = self
            .resolve_adapter(&request)
            .ok_or(AiError::NoProviderConfigured)?;

        let response = adapter.chat(request).await?;
        Ok(response)
    }

    async fn list_providers(&self) -> Result<Vec<ProviderInfo>> {
        self.pool
            .read(|conn| {
                let mut stmt = conn
                    .prepare(
                        "SELECT provider_id, provider_type, display_name, is_enabled, priority
                         FROM ai_providers
                         ORDER BY priority ASC",
                    )
                    .map_err(|e| {
                        cortex_db::DbError::Query(format!("prepare list_providers: {e}"))
                    })?;

                let rows = stmt
                    .query_map([], |row| {
                        let provider_type_str: String = row.get(1)?;
                        let display_name: String = row.get(2)?;
                        let is_enabled: bool = row.get::<_, i32>(3)? == 1;
                        let priority: i32 = row.get(4)?;
                        Ok((provider_type_str, display_name, is_enabled, priority))
                    })
                    .map_err(|e| cortex_db::DbError::Query(format!("query list_providers: {e}")))?;

                let mut providers = Vec::new();
                for row_result in rows {
                    let (provider_type_str, display_name, is_enabled, priority) =
                        row_result.map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                    let provider_type: AiProvider =
                        serde_json::from_str(&provider_type_str).unwrap_or(AiProvider::Custom);
                    providers.push(ProviderInfo {
                        provider_type,
                        display_name,
                        is_enabled,
                        priority,
                    });
                }
                Ok(providers)
            })
            .map_err(|_| AiError::Internal)
    }

    async fn list_models(&self, provider: Option<&str>) -> Result<Vec<ModelInfo>> {
        let mut all_models = Vec::new();
        for adapter in &self.adapters {
            if let Some(filter) = provider {
                if adapter.name() != filter {
                    continue;
                }
            }
            let models = adapter.list_models().await?;
            all_models.extend(models);
        }
        Ok(all_models)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_db::migration::run_migrations;

    /// Helper: create an in-memory pool with all migrations applied and a test
    /// user inserted (required for the FK on `ai_chat_history.user_id`).
    fn test_pool() -> Pool {
        let pool = Pool::open_in_memory().unwrap();
        run_migrations(&pool).unwrap();
        // Insert a test user so FK checks pass.
        pool.write(|conn| {
            conn.execute(
                "INSERT INTO users (user_id, username, password_hash) VALUES (?1, ?2, ?3)",
                rusqlite::params!["u-test-001", "testuser", "hash"],
            )
            .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
            Ok(())
        })
        .unwrap();
        pool
    }

    /// Helper: insert a user row and return the matching UserId.
    /// Tests that persist chat messages need a valid FK reference.
    fn insert_test_user(pool: &Pool) -> cortex_core::UserId {
        let uid = Uuid::now_v7();
        pool.write(|conn| {
            conn.execute(
                "INSERT OR IGNORE INTO users (user_id, username, password_hash) VALUES (?1, ?2, ?3)",
                rusqlite::params![uid.to_string(), format!("user-{uid}"), "hash"],
            )
            .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
            Ok(())
        })
        .unwrap();
        cortex_core::UserId(uid)
    }

    #[tokio::test]
    async fn stub_adapter_returns_canned_response() {
        let adapter = StubAdapter::new(AiProvider::OpenAI, "gpt-4o", "Hello from stub!");
        let request = ChatRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: "hi".into(),
            }],
            model: None,
            temperature: None,
            max_tokens: None,
        };
        let response = adapter.chat(request).await.unwrap();
        assert_eq!(response.message.content, "Hello from stub!");
        assert_eq!(response.message.role, ChatRole::Assistant);
        assert_eq!(response.model, "gpt-4o");
        assert_eq!(response.provider, AiProvider::OpenAI);
        assert_eq!(response.usage.prompt_tokens, 10);
        assert_eq!(response.usage.completion_tokens, 20);
        assert_eq!(response.usage.total_tokens, 30);
    }

    #[tokio::test]
    async fn configure_and_list_providers() {
        let pool = test_pool();
        let service = SqliteAiService::new(pool);

        service
            .configure_provider(&ProviderInfo {
                provider_type: AiProvider::OpenAI,
                display_name: "OpenAI".into(),
                is_enabled: true,
                priority: 0,
            })
            .unwrap();

        service
            .configure_provider(&ProviderInfo {
                provider_type: AiProvider::Anthropic,
                display_name: "Anthropic".into(),
                is_enabled: false,
                priority: 1,
            })
            .unwrap();

        let providers = service.list_providers().await.unwrap();
        assert_eq!(providers.len(), 2);
        // Ordered by priority ascending
        assert_eq!(providers[0].provider_type, AiProvider::OpenAI);
        assert_eq!(providers[0].display_name, "OpenAI");
        assert!(providers[0].is_enabled);
        assert_eq!(providers[1].provider_type, AiProvider::Anthropic);
        assert!(!providers[1].is_enabled);
    }

    #[tokio::test]
    async fn list_models_from_adapter() {
        let pool = test_pool();
        let adapter = StubAdapter::new(AiProvider::OpenAI, "gpt-4o", "hi");
        let service = SqliteAiService::with_adapters(pool, vec![Box::new(adapter)]);

        // All models
        let models = service.list_models(None).await.unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, "gpt-4o");

        // Filter by provider name
        let models = service.list_models(Some("openai")).await.unwrap();
        assert_eq!(models.len(), 1);

        // Non-matching filter
        let models = service.list_models(Some("anthropic")).await.unwrap();
        assert!(models.is_empty());
    }

    #[tokio::test]
    async fn chat_uses_highest_priority_adapter() {
        let pool = test_pool();
        let high = StubAdapter::new(AiProvider::OpenAI, "gpt-4o", "from-openai");
        let low = StubAdapter::new(AiProvider::Anthropic, "claude-3", "from-anthropic");

        let service = SqliteAiService::with_adapters(
            pool,
            vec![
                Box::new(high) as Box<dyn AiAdapter>,
                Box::new(low) as Box<dyn AiAdapter>,
            ],
        );

        // Configure openai with higher priority (lower number)
        service
            .configure_provider(&ProviderInfo {
                provider_type: AiProvider::OpenAI,
                display_name: "OpenAI".into(),
                is_enabled: true,
                priority: 0,
            })
            .unwrap();
        service
            .configure_provider(&ProviderInfo {
                provider_type: AiProvider::Anthropic,
                display_name: "Anthropic".into(),
                is_enabled: true,
                priority: 5,
            })
            .unwrap();

        let request = ChatRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: "hello".into(),
            }],
            model: None,
            temperature: None,
            max_tokens: None,
        };

        let response = service.chat(request).await.unwrap();
        assert_eq!(response.message.content, "from-openai");
    }

    #[tokio::test]
    async fn chat_returns_no_provider_when_none_configured() {
        let pool = test_pool();
        // No adapters, no providers configured
        let service = SqliteAiService::new(pool);

        let request = ChatRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: "hello".into(),
            }],
            model: None,
            temperature: None,
            max_tokens: None,
        };

        let result = service.chat(request).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AiError::NoProviderConfigured));
    }

    #[test]
    fn save_and_retrieve_chat_history() {
        let pool = test_pool();
        let user_id = insert_test_user(&pool);
        let service = SqliteAiService::new(pool);
        let conversation_id = Uuid::now_v7().to_string();

        let msg = ChatMessage {
            role: ChatRole::User,
            content: "What is Rust?".into(),
        };
        service
            .save_message(&user_id, &conversation_id, &msg, "gpt-4o", "openai", 15)
            .unwrap();

        let history = service.get_history(&conversation_id).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].role, ChatRole::User);
        assert_eq!(history[0].content, "What is Rust?");
    }

    #[test]
    fn chat_history_persists_multiple_messages() {
        let pool = test_pool();
        let user_id = insert_test_user(&pool);
        let service = SqliteAiService::new(pool);
        let conversation_id = Uuid::now_v7().to_string();

        // User message
        service
            .save_message(
                &user_id,
                &conversation_id,
                &ChatMessage {
                    role: ChatRole::User,
                    content: "Explain ownership".into(),
                },
                "",
                "",
                10,
            )
            .unwrap();

        // Assistant response
        service
            .save_message(
                &user_id,
                &conversation_id,
                &ChatMessage {
                    role: ChatRole::Assistant,
                    content: "Ownership is...".into(),
                },
                "gpt-4o",
                "openai",
                25,
            )
            .unwrap();

        // Follow-up user message
        service
            .save_message(
                &user_id,
                &conversation_id,
                &ChatMessage {
                    role: ChatRole::User,
                    content: "What about borrowing?".into(),
                },
                "",
                "",
                12,
            )
            .unwrap();

        let history = service.get_history(&conversation_id).unwrap();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].role, ChatRole::User);
        assert_eq!(history[0].content, "Explain ownership");
        assert_eq!(history[1].role, ChatRole::Assistant);
        assert_eq!(history[1].content, "Ownership is...");
        assert_eq!(history[2].role, ChatRole::User);
        assert_eq!(history[2].content, "What about borrowing?");
    }
}
