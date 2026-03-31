# 06 — AI Runtime: Provider Registry, Preferred LLM, Model Routing

## 1. Purpose

Define the AI runtime that makes AI a first-class system layer in CortexOS. This spec covers the provider abstraction, model registry, deterministic routing, streaming, tool-calling, budget management, and audit logging. The OS must remain fully functional even if no AI provider is configured.

## 2. Scope

- Provider adapter trait and contract
- Provider adapter implementations (OpenAI, Anthropic, Google, Ollama, Zhipu, custom)
- Model capability metadata registry
- Deterministic routing precedence
- Streaming completion support
- Structured output handling
- Tool-calling with permission gates
- Budget policy and cost tracking
- Audit logging for all AI operations
- Context attachment rules

## 3. Out of Scope

- AI surface UX (owned by spec 19)
- AI action permissions (owned by spec 20)
- Settings UI for AI config (owned by spec 05)
- Specific AI provider SDK integration details (each adapter owns its own)

## 4. Objectives

1. No provider lock-in: core runtime never imports provider SDKs directly.
2. Deterministic routing: given the same settings and context, always selects the same provider/model.
3. Graceful degradation: if all providers fail, return a structured error — never crash.
4. Full auditability: every AI request logged with provider, model, tokens, cost, outcome.
5. Privacy controls: PII handling configurable per `ai.privacy_mode` setting.

## 5. User-Visible Behavior

| Action | Outcome |
|---|---|
| User asks AI assistant a question | Tokens stream in real-time, provider/model shown in UI |
| User's preferred provider goes down | Fallback provider used (if enabled), indicator shows "using fallback" |
| All providers fail | "AI unavailable" message with retry button |
| Budget limit reached | "Daily AI budget reached" message, AI disabled until next period |
| User changes provider in Settings | Immediate switch, next request uses new provider |

## 6. System Behavior

### 6.1 Provider Adapter Contract

Every provider implements the `ProviderAdapter` trait:

```rust
#[async_trait]
trait ProviderAdapter: Send + Sync {
    /// Non-streaming chat completion
    async fn chat_completion(&self, req: CompletionRequest) -> Result<CompletionResponse, AIError>;

    /// Streaming chat completion (SSE)
    async fn stream_completion(&self, req: CompletionRequest) -> Result<Box<dyn CompletionStream>, AIError>;

    /// Text embedding
    async fn embed(&self, texts: Vec<String>, model: Option<&str>) -> Result<EmbedResponse, AIError>;

    /// List available models for this provider
    async fn list_models(&self) -> Result<Vec<ModelInfo>, AIError>;

    /// Get metadata for a specific model
    async fn get_model_metadata(&self, model_id: &str) -> Result<ModelInfo, AIError>;

    /// Health check
    async fn health_check(&self) -> Result<HealthStatus, AIError>;

    /// Provider identifier
    fn provider_id(&self) -> AIProviderType;
}

trait CompletionStream: Send + Sync {
    async fn next_token(&mut self) -> Option<Result<StreamToken, AIError>>;
}
```

### 6.2 Error Types

```rust
enum AIError {
    NoProviderConfigured,
    ProviderUnavailable { provider: AIProviderType, source: String },
    RateLimited { provider: AIProviderType, retry_after_ms: Option<u64> },
    AuthFailed { provider: AIProviderType },
    InvalidRequest { detail: String },
    ModelNotFound { model: String, provider: AIProviderType },
    QuotaExceeded { provider: AIProviderType },
    Timeout { provider: AIProviderType, elapsed_ms: u64 },
    ContentFiltered { provider: AIProviderType, reason: String },
}

impl AIError {
    fn is_retryable(&self) -> bool {
        matches!(self,
            AIError::ProviderUnavailable { .. } |
            AIError::RateLimited { .. } |
            AIError::Timeout { .. }
        )
    }
}
```

### 6.3 Routing Precedence (Deterministic)

For each incoming AI request, the runtime resolves the provider/model in this exact order:

1. **Per-feature override**: Check `ai.per_feature_overrides[feature_id]` — if set, use its provider/model
2. **Per-app override**: Check `ai.per_app_overrides[app_id]` — if set, use its provider/model
3. **User preferred**: Check `ai.preferred_provider` and `ai.preferred_model` — if set, use them
4. **No provider configured**: If no override and no global preferred provider exist, fail deterministically with `NoProviderConfigured`
5. **Fallback chain**: If the selected primary provider fails AND `ai.fallback_enabled=true`, try providers in `ai.fallback_chain` order

Resolution stops at the first layer that has a configured value. Empty/null values are treated as "not configured" and skipped.

### 6.4 Timeout Rules

| Timeout Type | Default | Configurable Via | Applies To |
|---|---|---|---|
| Connection | 10s | `ai.timeout_connect_ms` | Establishing HTTP connection |
| First token | 30s | `ai.timeout_first_token_ms` | Time to receive first streaming token |
| Total | 120s | `ai.timeout_total_ms` | Total request duration |
| Streaming idle | 60s | `ai.timeout_stream_idle_ms` | Max gap between tokens during streaming |

### 6.5 Budget Policy

```rust
struct BudgetPolicy {
    enabled: bool,
    daily_limit_cents: u64,       // USD cents
    warn_threshold_pct: u8,       // 0-100, warn when usage reaches this %
}

// Enforcement:
// - Before each request: check BudgetState for current period
// - Soft limit (warn_threshold_pct): log warning, allow request, show UI warning
// - Hard limit (daily_limit_cents): reject with QE_002, show "budget reached" in UI
// - Budget resets at midnight UTC
```

### 6.6 Context Attachment Rules

- AI sees only: user messages, conversation history, and explicitly permitted context
- File content: only if `ai.allow_file_access=true` AND user has file permission
- Clipboard: only if `ai.allow_clipboard_access=true`
- Context window management: if total context exceeds model's `max_context_tokens`, truncate oldest messages first
- Never attach context the user hasn't explicitly permitted

## 7. Architecture

```
┌─────────────────────────────────────────┐
│            cortex-ai                    │
│                                         │
│  ┌──────────────────────────────────┐   │
│  │        Routing Engine            │   │
│  │  1. per-feature → 2. per-app →   │   │
│  │  3. user preferred → 4. default  │   │
│  │  → 5. fallback chain             │   │
│  └──────────┬───────────────────────┘   │
│             │                           │
│  ┌──────────┴───────────────────────┐   │
│  │       Provider Registry          │   │
│  │  ┌─────┐ ┌──────┐ ┌──────────┐  │   │
│  │  │OpenAI│ │Anthr.│ │Google   │  │   │
│  │  └─────┘ └──────┘ └──────────┘  │   │
│  │  ┌──────┐ ┌──────┐ ┌────────┐   │   │
│  │  │Ollama│ │Zhipu │ │Custom  │   │   │
│  │  └──────┘ └──────┘ └────────┘   │   │
│  └──────────────────────────────────┘   │
│                                         │
│  ┌──────────┐  ┌───────────┐           │
│  │Budget    │  │Audit      │           │
│  │Tracker   │  │Logger     │           │
│  └──────────┘  └───────────┘           │
└─────────────────────────────────────────┘
```

## 8. Data Model

```rust
/// Provider configuration (stored in settings, read at startup)
struct ProviderConfig {
    id: String,                    // Unique identifier
    name: String,                  // Display name
    adapter_type: AIProviderType,  // Which adapter to use
    api_endpoint: String,          // Base URL
    api_key_ref: Option<String>,   // Reference to encrypted key in settings
    enabled: bool,
    priority: u32,                 // Lower = higher priority for fallback
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum AIProviderType {
    OpenAI,
    Anthropic,
    Google,
    Ollama,
    Zhipu,
    Custom,
}

/// Model capability metadata
struct ModelInfo {
    id: String,                        // e.g. "gpt-4o", "claude-sonnet-4-6"
    provider_id: AIProviderType,
    display_name: String,              // e.g. "GPT-4o", "Claude Sonnet 4.6"
    capabilities: ModelCapabilities,
    pricing: ModelPricing,
}

struct ModelCapabilities {
    supports_streaming: bool,
    supports_tools: bool,
    supports_vision: bool,
    supports_structured_output: bool,
    max_context_tokens: u32,
    max_output_tokens: u32,
    latency_tier: LatencyTier,
}

#[derive(Debug, Clone)]
enum LatencyTier { Fast, Medium, Slow }

struct ModelPricing {
    input_price_per_million_tokens: u64,   // USD cents
    output_price_per_million_tokens: u64,  // USD cents
}

/// Task type for routing
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum AITaskType {
    Chat,
    Summarize,
    Translate,
    Code,
    Analyze,
    Extract,
    Rewrite,
    Classify,
    Embed,
    Custom(String),
}

/// Routing request
struct RoutingRequest {
    task_type: AITaskType,
    app_id: Option<String>,
    feature_id: Option<String>,
    context: Vec<ChatMessage>,
    preferred_model_override: Option<String>,
}

/// Routing decision (logged)
struct RoutingDecision {
    provider_id: AIProviderType,
    model_id: String,
    fallback_chain_used: bool,
    fallback_position: Option<u32>,
    reason: RoutingReason,
}

enum RoutingReason {
    PerFeatureOverride,
    PerAppOverride,
    UserPreference,
    SystemDefault,
    Fallback,
}

/// Completion request (provider-agnostic)
struct CompletionRequest {
    messages: Vec<ChatMessage>,
    model: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    tools: Vec<ToolDefinition>,
    structured_output_schema: Option<serde_json::Value>,
}

struct ChatMessage {
    role: MessageRole,
    content: String,
    tool_calls: Option<Vec<ToolCall>>,
    tool_call_id: Option<String>,
}

enum MessageRole { System, User, Assistant, Tool }

struct ToolDefinition {
    name: String,
    description: String,
    parameters: serde_json::Value,  // JSON Schema
}

struct ToolCall {
    id: String,
    name: String,
    arguments: serde_json::Value,
}

/// Completion response
struct CompletionResponse {
    content: String,
    model_used: String,
    provider_used: AIProviderType,
    token_usage: TokenUsage,
    latency_ms: u64,
    tool_calls: Vec<ToolCall>,
    finish_reason: FinishReason,
}

struct TokenUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

enum FinishReason { Stop, ToolCall, MaxTokens, ContentFilter }

/// Budget state
struct BudgetState {
    user_id: String,
    period_start: chrono::DateTime<chrono::Utc>,
    tokens_used: u64,
    estimated_cost_cents: u64,
    limit_cents: u64,
    warned: bool,
}

/// Audit entry (append-only)
struct AuditEntry {
    timestamp: chrono::DateTime<chrono::Utc>,
    user_id: String,
    app_id: Option<String>,
    provider_id: AIProviderType,
    model_id: String,
    task_type: AITaskType,
    prompt_tokens: u32,
    completion_tokens: u32,
    latency_ms: u64,
    status: AIRequestStatus,
    privacy_mode: PrivacyMode,
    estimated_cost_cents: u64,
}

enum AIRequestStatus { Success, Error(AIError), Cancelled }
enum PrivacyMode { None, RedactPII, HashPII }
```

## 9. Public Interfaces

### 9.1 Internal Rust API

```rust
trait AIRuntimeService: Send + Sync {
    /// Complete a request with automatic routing
    async fn complete(&self, req: RoutingRequest) -> Result<CompletionResponse, AIError>;

    /// Stream a request with automatic routing
    async fn stream(&self, req: RoutingRequest) -> Result<Box<dyn CompletionStream>, AIError>;

    /// Get current routing decision (without executing)
    fn resolve_routing(&self, req: &RoutingRequest) -> RoutingDecision;

    /// Get available models for a provider
    async fn list_models(&self, provider: AIProviderType) -> Result<Vec<ModelInfo>, AIError>;

    /// Get budget status
    fn get_budget_status(&self, user_id: &str) -> BudgetState;

    /// Get audit log (paginated)
    fn get_audit_log(&self, user_id: &str, limit: u32, cursor: Option<&str>) -> Vec<AuditEntry>;
}
```

### 9.2 REST API

```
POST   /api/v1/ai/complete          → Non-streaming completion
POST   /api/v1/ai/stream            → Streaming completion (SSE)
GET    /api/v1/ai/models             → List available models
GET    /api/v1/ai/budget             → Get budget status
GET    /api/v1/ai/audit              → Get audit log
GET    /api/v1/ai/status             → Provider health status
```

## 10. Internal Interfaces

- Reads settings from `cortex-settings` (provider config, routing rules, budget policy)
- Checks permissions via `cortex-policy` (tool-call execution, file access)
- Logs to `cortex-observability` (structured logs, metrics, audit entries)
- Emits events via command bus: `ai.request_started`, `ai.request_completed`, `ai.provider_changed`

## 11. State Management

- Provider configs: stored in settings (encrypted API keys), cached in memory
- Model registry: loaded at startup from provider APIs, cached with 1-hour TTL refresh
- Budget state: stored in SQLite, updated atomically per request
- Audit log: append-only SQLite table, no updates or deletes
- Conversation history: stored per-context in SQLite, not in settings

## 12. Failure Modes and Error Handling

| Failure | Handling |
|---|---|
| Primary provider down | Try fallback chain (if enabled), else return PU_002 |
| Rate limited by provider | Auto-retry after `retry_after_ms` (max 3x), else return RL_001 |
| Auth failed | Return PU_001 with "check API key" message, do NOT retry |
| Timeout | Return TM_002, suggest user try again or use faster model |
| Content filtered | Return AIError::ContentFiltered with provider reason |
| Budget exceeded | Return QE_002, disable AI until budget resets |
| All providers fail | Return PU_002 with "all providers unavailable", never crash |
| Model not found | Return AIError::ModelNotFound, suggest checking settings |

## 13. Security and Permissions

- API keys stored encrypted at rest in settings (never in env vars or config files)
- API keys never logged, never returned in API responses
- Tool-calling requires permission check via cortex-policy for each tool
- AI cannot bypass permission system (INV-06-1)
- AI cannot execute arbitrary code (INV-06-2)
- AI cannot access files without going through cortex-files (INV-06-3)
- AI actions never silent — always logged in audit trail (INV-06-4)

## 14. Performance Requirements

| Metric | Target |
|---|---|
| Routing resolution (in-memory) | < 1ms |
| First streaming token (p50) | < 2s |
| First streaming token (p99) | < 10s |
| Audit log write | < 5ms |
| Budget check | < 1ms |
| Model registry refresh | < 30s (background) |

## 15. Accessibility Requirements

- AI status messages ("AI unavailable", "budget reached") must be ARIA live regions
- Streaming text updates must be announced incrementally to screen readers
- Provider/model disclosure must be accessible text, not icon-only

## 16. Observability and Logging

Every AI request generates:
- Structured log: `{timestamp, user_id, app_id, provider, model, task_type, tokens_in, tokens_out, latency_ms, status, cost_cents}`
- Metric: `ai_request_duration_ms` histogram, `ai_tokens_used` counter, `ai_cost_cents` counter, `ai_error_count` by type
- Audit entry: persisted in append-only `ai_audit` table

Privacy mode handling:
- `none`: log as-is
- `redact_pii`: redact names, emails, phone numbers from logged content
- `hash_pii`: SHA-256 hash PII fields in logs

## 17. Testing Requirements

- Unit: routing resolution for all 5 precedence levels
- Unit: fallback chain behavior (primary fail → fallback → success)
- Unit: budget enforcement (soft warn, hard block)
- Unit: timeout handling (connection, first-token, total, idle)
- Integration: end-to-end completion with mock provider adapter
- Integration: tool-call permission gating
- Integration: audit log completeness
- E2E: provider switch in Settings → next request uses new provider

## 18. Acceptance Criteria

- [ ] ProviderAdapter trait implemented with all methods
- [ ] At least 1 adapter implemented (OpenAI or Anthropic)
- [ ] Routing resolution matches the 5-level precedence exactly
- [ ] Fallback chain works: primary fail → next in chain → success
- [ ] All providers fail: returns structured error, never crashes
- [ ] Streaming tokens delivered via SSE
- [ ] Budget enforcement: soft warn at threshold, hard block at limit
- [ ] Every request logged in audit trail
- [ ] Privacy mode applied to logged content
- [ ] Tool calls permission-checked via cortex-policy
- [ ] No provider SDK imported in core runtime code
- [ ] OS functional with zero providers configured

## 19. Build Order and Dependencies

**Layer 5**. Depends on:
- 01 — Repository conventions
- 02 — Core architecture (error taxonomy, trust model)
- 04 — Permissions (tool-call permission checks)
- 05 — Settings (AI settings fields, provider config)

Blocks:
- 19 — AI system surfaces (needs AI runtime to be available)
- 20 — AI action permissions (needs tool-call infrastructure)

## 20. Non-Goals and Anti-Patterns

**Non-Goals**:
- AI model fine-tuning or training
- Custom model hosting
- Multi-turn conversation management (conversation history managed by surfaces, not runtime)
- Agent/autonomous AI loops (v1 only reactive)

**Anti-Patterns**:
- NEVER import a provider SDK in the core AI runtime crate — all provider logic behind adapters
- NEVER hardcode a provider name or model ID in routing logic
- NEVER skip the audit log for any AI request
- NEVER allow AI to bypass cortex-policy for permission checks
- NEVER store API keys in plaintext
- NEVER silently switch providers without logging the switch

## 21. Implementation Instructions for Claude Code / Codex

1. Define `ProviderAdapter` trait, `AIError`, `CompletionRequest`, `CompletionResponse` in `cortex-ai`.
2. Define all data model types (ProviderConfig, ModelInfo, RoutingRequest, RoutingDecision, BudgetState, AuditEntry).
3. Implement the `RoutingEngine`: reads settings, resolves provider/model per 5-level precedence.
4. Implement `ProviderRegistry`: HashMap of AIProviderType → ProviderAdapter, loaded from settings at startup.
5. Implement budget tracker: SQLite table, atomic check-and-decrement per request.
6. Implement audit logger: append-only SQLite insert per request.
7. Implement one adapter (OpenAI recommended as first): HTTP client, SSE parsing, error mapping.
8. Write tests: routing resolution, fallback chain, budget enforcement, timeout handling, audit completeness.
9. Verify: no `openai`, `anthropic`, or provider SDK crate in `cortex-ai/Cargo.toml` — adapter crates depend on SDKs, not core.
