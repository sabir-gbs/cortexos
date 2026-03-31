//! AI API routes.
//!
//! Framework-agnostic handler functions for AI chat operations.

use crate::app_state::AppState;
use crate::error::{ApiError, Result};
use cortex_ai::{AiService, ChatMessage, ChatRequest, ChatRole};
use cortex_core::SuccessResponse;
use serde::{Deserialize, Serialize};

/// Serialize a serde value that is known to produce a quoted string,
/// then strip the surrounding quotes.  Returns an internal error
/// instead of silently defaulting to an empty string.
fn serde_enum_to_str<T: Serialize>(val: &T) -> std::result::Result<String, serde_json::Error> {
    let quoted = serde_json::to_string(val)?;
    Ok(quoted.trim_matches('"').to_string())
}

/// Chat request body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequestBody {
    pub messages: Vec<ChatMessageBody>,
    pub model: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
}

/// Chat message body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageBody {
    pub role: String,
    pub content: String,
}

/// Chat response body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponseBody {
    pub message: ChatMessageBody,
    pub usage: TokenUsageBody,
    pub model: String,
    pub provider: String,
}

/// Token usage body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageBody {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Provider info body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfoBody {
    pub provider_type: String,
    pub display_name: String,
    pub is_enabled: bool,
    pub priority: i32,
}

/// Model info body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfoBody {
    pub id: String,
    pub display_name: String,
    pub provider: String,
}

/// Send a chat request.
pub async fn chat(
    state: &AppState,
    req: ChatRequestBody,
) -> Result<SuccessResponse<ChatResponseBody>> {
    let messages: Vec<ChatMessage> = req
        .messages
        .into_iter()
        .map(|m| {
            let role = match m.role.as_str() {
                "system" => ChatRole::System,
                "assistant" => ChatRole::Assistant,
                _ => ChatRole::User,
            };
            ChatMessage {
                role,
                content: m.content,
            }
        })
        .collect();

    let chat_req = ChatRequest {
        messages,
        model: req.model,
        temperature: req.temperature,
        max_tokens: req.max_tokens,
    };

    let ai = state.ai.lock().await;
    let response = ai.chat(chat_req).await.map_err(|e| match e {
        cortex_ai::AiError::NoProviderConfigured => {
            ApiError::BadRequest("no AI provider configured".to_string())
        }
        cortex_ai::AiError::ProviderUnavailable { provider } => {
            ApiError::Internal(format!("provider unavailable: {provider}"))
        }
        cortex_ai::AiError::BudgetExceeded => {
            ApiError::BadRequest("daily AI budget exceeded".to_string())
        }
        cortex_ai::AiError::RateLimited { .. } => {
            ApiError::Internal("rate limited by provider".to_string())
        }
        cortex_ai::AiError::InvalidRequest(msg) => ApiError::BadRequest(msg),
        cortex_ai::AiError::Timeout { provider } => {
            ApiError::Internal(format!("timeout waiting for {provider}"))
        }
        cortex_ai::AiError::Internal => ApiError::Internal("AI error".to_string()),
    })?;

    let role_str = serde_enum_to_str(&response.message.role)
        .map_err(|e| ApiError::Internal(format!("failed to serialize chat role: {e}")))?;
    let provider_str = serde_enum_to_str(&response.provider)
        .map_err(|e| ApiError::Internal(format!("failed to serialize provider: {e}")))?;

    Ok(SuccessResponse {
        data: ChatResponseBody {
            message: ChatMessageBody {
                role: role_str,
                content: response.message.content,
            },
            usage: TokenUsageBody {
                prompt_tokens: response.usage.prompt_tokens,
                completion_tokens: response.usage.completion_tokens,
                total_tokens: response.usage.total_tokens,
            },
            model: response.model,
            provider: provider_str,
        },
        meta: None,
    })
}

/// List configured AI providers.
pub async fn list_providers(state: &AppState) -> Result<SuccessResponse<Vec<ProviderInfoBody>>> {
    let ai = state.ai.lock().await;
    let providers = ai
        .list_providers()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let items = providers
        .into_iter()
        .map(|p| {
            let provider_type = serde_enum_to_str(&p.provider_type).map_err(|e| {
                ApiError::Internal(format!("failed to serialize provider type: {e}"))
            })?;
            Ok(ProviderInfoBody {
                provider_type,
                display_name: p.display_name,
                is_enabled: p.is_enabled,
                priority: p.priority,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(SuccessResponse {
        data: items,
        meta: None,
    })
}

/// List available AI models.
pub async fn list_models(
    state: &AppState,
    provider: Option<&str>,
) -> Result<SuccessResponse<Vec<ModelInfoBody>>> {
    let ai = state.ai.lock().await;
    let models = ai
        .list_models(provider)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let items = models
        .into_iter()
        .map(|m| {
            let provider = serde_enum_to_str(&m.provider).map_err(|e| {
                ApiError::Internal(format!("failed to serialize model provider: {e}"))
            })?;
            Ok(ModelInfoBody {
                id: m.id,
                display_name: m.display_name,
                provider,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(SuccessResponse {
        data: items,
        meta: None,
    })
}
