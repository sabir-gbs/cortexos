//! AI runtime error types.
//!
//! Covers every failure mode described in spec 06, section 6.2.

/// Result alias for AI operations.
pub type Result<T> = std::result::Result<T, AiError>;

/// Errors that can occur during AI operations.
///
/// Each variant maps to a specific failure mode from spec 06, section 12.
#[derive(Debug, thiserror::Error)]
pub enum AiError {
    /// No AI provider has been configured at all.
    /// Maps to the "all providers fail" / "no provider configured" case.
    #[error("no AI provider configured")]
    NoProviderConfigured,

    /// The selected provider is unreachable or returned an unexpected error.
    #[error("provider unavailable: {provider}")]
    ProviderUnavailable {
        /// Which provider failed.
        provider: String,
    },

    /// The request timed out waiting for a provider response.
    #[error("request timed out waiting for {provider}")]
    Timeout {
        /// Which provider timed out.
        provider: String,
    },

    /// The configured daily budget has been exceeded.
    /// Maps to QE_002 in spec 06.
    #[error("daily AI budget exceeded")]
    BudgetExceeded,

    /// The provider rate-limited the request.
    #[error("rate limited by provider")]
    RateLimited {
        /// Suggested retry delay in milliseconds, if the provider supplied one.
        retry_after_ms: Option<u64>,
    },

    /// The request was malformed or contained invalid parameters.
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// An unexpected internal error not attributable to any provider.
    #[error("internal AI error")]
    Internal,
}

impl AiError {
    /// Returns `true` if the caller may reasonably retry the same request.
    ///
    /// Matches spec 06 section 6.2: `ProviderUnavailable`, `RateLimited`, and
    /// `Timeout` are retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AiError::ProviderUnavailable { .. }
                | AiError::RateLimited { .. }
                | AiError::Timeout { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_messages() {
        assert_eq!(
            AiError::NoProviderConfigured.to_string(),
            "no AI provider configured"
        );
        assert_eq!(
            AiError::ProviderUnavailable {
                provider: "openai".into()
            }
            .to_string(),
            "provider unavailable: openai"
        );
        assert_eq!(
            AiError::Timeout {
                provider: "anthropic".into()
            }
            .to_string(),
            "request timed out waiting for anthropic"
        );
        assert_eq!(
            AiError::BudgetExceeded.to_string(),
            "daily AI budget exceeded"
        );
        assert_eq!(
            AiError::RateLimited {
                retry_after_ms: Some(5000)
            }
            .to_string(),
            "rate limited by provider"
        );
        assert_eq!(
            AiError::InvalidRequest("bad model".into()).to_string(),
            "invalid request: bad model"
        );
        assert_eq!(AiError::Internal.to_string(), "internal AI error");
    }

    #[test]
    fn retryable_variants() {
        assert!(AiError::ProviderUnavailable {
            provider: "x".into()
        }
        .is_retryable());
        assert!(AiError::Timeout {
            provider: "x".into()
        }
        .is_retryable());
        assert!(AiError::RateLimited {
            retry_after_ms: None
        }
        .is_retryable());

        assert!(!AiError::NoProviderConfigured.is_retryable());
        assert!(!AiError::BudgetExceeded.is_retryable());
        assert!(!AiError::InvalidRequest("x".into()).is_retryable());
        assert!(!AiError::Internal.is_retryable());
    }
}
