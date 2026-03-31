# SPEC 19: AI System Surfaces and UX

**Spec ID:** 19
**Status:** Implementation-grade
**Risk Level:** HIGH-RISK
**Owner Package:** `@cortexos/ai-client`
**Last Updated:** 2026-03-29

---

## 1. Purpose

This specification defines every user-facing AI surface in CortexOS. It establishes how users invoke AI capabilities, how AI responses are presented, how streaming works, how conversation history is managed, and how failures are communicated. The goal is a unified, consistent, and safe AI experience across all contexts in the operating system.

---

## 2. Scope

- Global assistant panel (persistent, context-aware AI chat)
- App assistant hooks (app-defined AI actions exposed through a standard interface)
- Selected text and file AI actions (context-menu-driven AI operations)
- Provider and model disclosure in the UI
- Confirmation dialogs for modifying AI actions
- Failure and degradation behavior when AI providers are unavailable
- AI surface state machine (Loading, Streaming, Complete, Error, RateLimited, QuotaExceeded)
- Conversation history storage, persistence, and search
- Streaming UX with token-by-token rendering and typing indicator
- Multi-turn conversation support in the assistant panel

---

## 3. Out of Scope

- AI model training, fine-tuning, or weight management
- AI provider account creation or billing management
- Direct API key rotation workflows (handled in Settings > AI)
- Voice-based AI interaction
- Image generation AI surfaces
- Autonomous AI agent workflows (AI acting independently without user initiation)
- Third-party AI plugin installation (covered in SPEC 21)

---

## 4. Objectives

1. Provide a single, system-wide AI interaction surface that is accessible from any context.
2. Ensure every AI action clearly communicates which provider and model is being used.
3. Require explicit user confirmation before any AI action that modifies user data.
4. Degrade gracefully when AI providers are unavailable, with actionable error messages.
5. Maintain conversation history per context, persisted across sessions, with full search capability.
6. Stream AI responses token-by-token with visual feedback during generation.
7. Support multi-turn conversations with full context retention within a session.

---

## 5. User-Visible Behavior

### 5.1 Global Assistant Panel

- The global assistant panel is a persistent sidebar panel accessible via:
  - Keyboard shortcut: `Ctrl+Shift+A` (configurable)
  - System tray icon click (AI icon)
  - Panel menu: "AI Assistant"
- The panel slides in from the right side of the screen, taking 380px of width on standard displays.
- The panel contains:
  - A conversation display area (scrollable)
  - A text input field at the bottom (multi-line, grows up to 4 lines then scrolls)
  - A provider/model disclosure badge in the top-right corner
  - A "New Conversation" button
  - A "Search History" button
  - A collapse/close button
- The panel remembers its open/closed state across sessions.
- When opened, the input field receives focus automatically.
- The panel overlays content but does not push or resize application windows.

### 5.2 App Assistant Hooks

- Apps register AI actions through the `@cortexos/ai-client` API.
- Registered actions appear in the app's toolbar or context menu under an "AI" submenu.
- Examples:
  - Text Editor: "Improve Writing", "Summarize Selection", "Translate", "Explain Code"
  - File Manager: "Summarize Files", "Analyze Contents", "Generate README"
  - Notes App: "Summarize Note", "Expand Outline", "Generate Tags"
- Each action has:
  - A display label (e.g., "Improve Writing")
  - An icon (optional, defaults to standard AI icon)
  - A confirmation requirement flag (modifying actions require confirmation)
  - A description shown on hover

### 5.3 Selected Text/File AI Actions

- When the user selects text in any context, a context menu includes an "AI" submenu with actions:
  - **Summarize**: Produces a concise summary of selected text
  - **Translate**: Opens a sub-menu of target languages, then translates
  - **Rewrite**: Rewrites the selected text in a specified tone (formal, casual, concise, expanded)
  - **Explain**: Provides an explanation of the selected text
- When the user selects one or more files in the file manager, the context menu "AI" submenu includes:
  - **Summarize**: Generates a summary of file contents
  - **Analyze**: Provides structural or content analysis
- AI action results appear in a floating overlay card anchored near the selection.
- The card has buttons: "Copy", "Replace" (if modification allowed), "Dismiss".
- "Replace" requires confirmation before modifying the source text.

### 5.4 Provider/Model Disclosure

- The UI must always show the active AI provider and model name.
- Disclosure is controlled by the setting `ai.show_model_disclosure` (default: `true`).
- When enabled, a badge displays in:
  - The top-right corner of the global assistant panel (e.g., "OpenAI / GPT-4o")
  - The header of any AI action result card (e.g., "via Anthropic / Claude")
  - The status bar when an AI operation is in progress
- When `ai.show_model_disclosure` is `false`, the badge is hidden but the information remains available in the conversation metadata.
- The badge text format is: `"{provider_name} / {model_name}"`
- If the provider/model changes mid-conversation, the badge updates and a system message is inserted: "Model changed to {provider_name} / {model_name}."

### 5.5 Confirmation Expectations

- AI actions are categorized as read-only or modifying:
  - **Read-only**: Summarize, Explain, Analyze, Translate (display-only). No confirmation required. Results appear in a card or inline.
  - **Modifying**: Rewrite, Improve, Replace, Generate-and-insert. Confirmation required.
- Confirmation dialog contents:
  - Title: "Confirm AI Action"
  - Description: "{action_name} will modify {target_description}."
  - Preview: Shows the proposed change in a diff view or before/after comparison
  - Buttons: "Apply" (primary), "Cancel" (secondary)
- For modifying actions on files, the preview must show at minimum a unified diff.
- User can enable "Auto-apply low-risk AI actions" in Settings to skip the extra post-generation Apply step for eligible low-risk inline actions after permission checks have already passed. Default: disabled.

### 5.6 Failure Behavior

When the AI model or provider is unavailable, the system must:
1. Display a clear, human-readable error message in the AI surface.
2. Never crash, hang, or silently fail.
3. Suggest the user check AI settings (Settings > AI > Provider Configuration).

Specific failure messages:
- **Provider unreachable**: "Unable to connect to {provider_name}. Please check your internet connection and provider settings."
- **Authentication failure**: "Authentication with {provider_name} failed. Please verify your API key in Settings > AI."
- **Rate limited**: "You have exceeded the rate limit for {provider_name}. Please wait {retry_after_seconds} seconds and try again."
- **Quota exceeded**: "Your {provider_name} usage quota has been exceeded. Please check your plan or try a different provider."
- **Model unavailable**: "The model '{model_name}' is currently unavailable on {provider_name}. Please try again or select a different model."
- **Response timeout**: "The AI response timed out after {timeout_seconds} seconds. You can increase the timeout in Settings > AI."

---

## 6. System Behavior

### 6.1 AI Surface State Machine

Every AI surface interaction follows a deterministic state machine:

```
Idle -> Loading -> Streaming -> Complete -> Idle
                  |-> Error -> Idle
                  |-> RateLimited -> Idle
                  |-> QuotaExceeded -> Idle
Loading -> Error (if request fails before streaming begins)
```

States:
- **Idle**: No active AI request. Input field is enabled. Surface shows conversation history.
- **Loading**: Request sent, waiting for first token. A typing indicator (three animated dots) is shown in the response area. The input field is disabled. A cancel button appears.
- **Streaming**: Tokens are arriving. Each token is appended to the response. A typing indicator persists at the end of the streaming text. The input field remains disabled. A "Stop generating" button is visible.
- **Complete**: All tokens received. Full response displayed. Input field re-enabled. Copy/Replace/Dismiss actions appear.
- **Error**: An error occurred. Error message displayed in a distinct style (red-tinted card). Input field re-enabled. A "Retry" button is shown.
- **RateLimited**: Rate limit hit. A cooldown timer is displayed. Input field re-enabled after cooldown. The timer counts down in seconds.
- **QuotaExceeded**: Quota exhausted. A message explains the situation with a link to Settings > AI. Input field remains enabled for reading history but new requests are blocked until quota resets or provider changes.

### 6.2 Conversation History

- Each conversation has a unique `conversation_id` (UUID v4).
- Conversations are scoped by context:
  - **Global assistant panel**: Conversations are identified by `conversation_id` only.
  - **App-specific contexts**: Conversations are identified by `(app_id, context_key)` where `context_key` is defined by the app (e.g., a file path for a text editor).
- Conversation history includes:
  - All user messages
  - All AI responses (complete text)
  - System messages (model changes, errors)
  - Metadata: timestamp, provider, model, token_count
- Conversations are persisted in the server-side SQLite store owned by the AI surface layer. The browser client treats conversation history as cached state only.
- The global assistant panel shows a conversation list sidebar (toggleable) showing the last 50 conversations, sorted by last message timestamp.
- Conversations are never auto-deleted. Users can manually delete conversations.
- Search operates over message content across all conversations using full-text search.

### 6.3 Streaming UX

- Tokens are rendered as they arrive from the provider.
- Token rendering debouncing: tokens are batched for rendering every 50ms to avoid excessive DOM/layout updates.
- A typing indicator (animated ellipsis) appears at the end of the streaming text.
- When streaming completes, the typing indicator is removed and the full response is finalized.
- The user can click "Stop generating" during streaming, which cancels the request and preserves the partial response with a "[Stopped]" marker.
- Partial responses are saved to conversation history marked as `complete: false`.
- If the user sends a new message while a response is streaming, the current stream is cancelled first (with confirmation: "Cancel current response?").

### 6.4 Multi-Turn Conversation

- The global assistant panel supports multi-turn conversations.
- Full conversation context (all previous messages) is sent with each new request, up to the model's context window limit.
- If the context exceeds the window limit, the oldest messages are summarized (or truncated) with a system message: "Earlier conversation context was truncated to fit the model's context window."
- Context window management strategy (configurable, default: `truncate_oldest`):
  - `truncate_oldest`: Remove oldest messages until context fits.
  - `summarize_older`: Generate a summary of older messages and prepend as a system message.
- Conversation branching is not supported in v1 (all messages are linear).

---

## 7. Architecture

```
+-----------------------------------------------+
|                 @cortexos/ai-client            |
|                                                |
|  +------------------+  +--------------------+  |
|  | Global Assistant |  | App AI Hooks       |  |
|  | Panel            |  | Registry           |  |
|  +--------+---------+  +---------+----------+  |
|           |                      |             |
|  +--------v----------------------v----------+  |
|  |          AI Surface Controller           |  |
|  +------------------+----------------------+  |
|                     |                          |
|  +------------------v----------------------+  |
|  |          Streaming Renderer            |  |
|  +------------------+----------------------+  |
|                     |                          |
|  +------------------v----------------------+  |
|  |     Conversation Store (Persistence)    |  |
|  +------------------+----------------------+  |
|                     |                          |
+---------------------|--------------------------+
                      |
          +-----------v-----------+
          |     cortex-ai         |
          |  (AI runtime layer)   |
          +-----------------------+
```

### Component Responsibilities

- **Global Assistant Panel**: UI component for the persistent sidebar. Manages panel open/close state, conversation list, and input.
- **App AI Hooks Registry**: In-memory registry of AI actions registered by apps. Each entry maps `(app_id, action_id)` to an `AiActionDefinition`.
- **AI Surface Controller**: Central orchestrator. Receives user requests, manages state machine transitions, routes to the correct provider, and handles confirmations.
- **Streaming Renderer**: Manages token-by-token rendering, debouncing, typing indicator, and stop/cancel logic.
- **Conversation Store**: Handles persistence, retrieval, and search of conversation history in the server-side SQLite store.
- **cortex-ai**: The runtime/provider layer that abstracts provider-specific APIs into a uniform interface.

---

## 8. Data Model

### 8.1 Conversation

```rust
struct Conversation {
    conversation_id: Uuid,
    context: ConversationContext,
    messages: Vec<Message>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    metadata: ConversationMetadata,
}

enum ConversationContext {
    Global,
    App {
        app_id: String,
        context_key: String,
    },
}

struct ConversationMetadata {
    total_tokens_used: u64,
    primary_provider: String,
    primary_model: String,
    message_count: u32,
}
```

### 8.2 Message

```rust
struct Message {
    message_id: Uuid,
    role: MessageRole,
    content: String,
    timestamp: DateTime<Utc>,
    token_count: u32,
    model_info: Option<ModelInfo>,
    complete: bool,
    metadata: HashMap<String, String>,
}

enum MessageRole {
    User,
    Assistant,
    System,
}

struct ModelInfo {
    provider: String,
    model: String,
}
```

### 8.3 AiActionDefinition

```rust
struct AiActionDefinition {
    action_id: String,
    app_id: String,
    label: String,
    description: String,
    icon: Option<Icon>,
    category: AiActionCategory,
    requires_confirmation: bool,
    handler: AiActionHandler,
}

enum AiActionCategory {
    Text { operation: TextOperation },
    File { operation: FileOperation },
    Custom { operation_name: String },
}

enum TextOperation {
    Summarize,
    Translate { target_language: Option<String> },
    Rewrite { tone: Option<Tone> },
    Explain,
    Improve,
    Custom(String),
}

enum FileOperation {
    Summarize,
    Analyze,
    Generate(String),
}

enum Tone {
    Formal,
    Casual,
    Concise,
    Expanded,
    Custom(String),
}
```

### 8.4 AiSurfaceState

```rust
enum AiSurfaceState {
    Idle,
    Loading {
        request_id: Uuid,
        started_at: DateTime<Utc>,
    },
    Streaming {
        request_id: Uuid,
        started_at: DateTime<Utc>,
        tokens_received: u32,
    },
    Complete {
        request_id: Uuid,
        total_tokens: u32,
        duration: Duration,
    },
    Error {
        request_id: Option<Uuid>,
        error: AiSurfaceError,
    },
    RateLimited {
        retry_after: Duration,
    },
    QuotaExceeded {
        provider: String,
    },
}

enum AiSurfaceError {
    ProviderUnreachable { provider: String },
    AuthenticationFailed { provider: String },
    ModelUnavailable { provider: String, model: String },
    Timeout { timeout_seconds: u32 },
    InternalError { message: String },
}
```

### 8.5 Context Menu AI Action

```rust
struct ContextMenuAiAction {
    action_id: String,
    label: String,
    icon: Option<Icon>,
    requires_confirmation: bool,
    applicable_to: ActionTarget,
}

enum ActionTarget {
    SelectedText { content: String },
    SelectedFiles { paths: Vec<PathBuf> },
    Both { text: Option<String>, files: Vec<PathBuf> },
}
```

---

## 9. Public Interfaces

### 9.1 AI Surface API (for UI components)

```rust
trait AiSurfaceApi {
    /// Open the global assistant panel
    fn open_assistant_panel(&self) -> Result<()>;

    /// Close the global assistant panel
    fn close_assistant_panel(&self) -> Result<()>;

    /// Send a message in the current conversation
    fn send_message(
        &self,
        conversation_id: Uuid,
        message: String,
    ) -> Result<()>;

    /// Cancel an in-progress streaming response
    fn cancel_streaming(&self, request_id: Uuid) -> Result<()>;

    /// Get the current state of an AI surface
    fn get_surface_state(&self, surface_id: Uuid) -> AiSurfaceState;

    /// Get conversation history
    fn get_conversation(&self, conversation_id: Uuid) -> Result<Conversation>;

    /// Search conversations
    fn search_conversations(&self, query: String, limit: u32) -> Result<Vec<ConversationSearchResult>>;

    /// Delete a conversation
    fn delete_conversation(&self, conversation_id: Uuid) -> Result<()>;
}

struct ConversationSearchResult {
    conversation_id: Uuid,
    matched_message_id: Uuid,
    matched_text: String,
    relevance_score: f32,
    timestamp: DateTime<Utc>,
}
```

### 9.2 App AI Hooks Registration API

```rust
trait AiHooksRegistry {
    /// Register an AI action for an app
    fn register_action(&self, app_id: &str, action: AiActionDefinition) -> Result<()>;

    /// Unregister all actions for an app
    fn unregister_app_actions(&self, app_id: &str) -> Result<()>;

    /// Get all registered actions for an app
    fn get_app_actions(&self, app_id: &str) -> Vec<AiActionDefinition>;

    /// Execute a registered action
    fn execute_action(
        &self,
        action_id: &str,
        target: ActionTarget,
        skip_confirmation: bool,
    ) -> Result<ActionResult>;
}

struct ActionResult {
    output: String,
    modified_resources: Vec<PathBuf>,
    tokens_used: u32,
}
```

### 9.3 Settings API (relevant keys)

```rust
struct AiSurfaceSettings {
    /// Whether to show provider/model disclosure badges
    show_model_disclosure: bool, // default: true

    /// Keyboard shortcut to open assistant panel
    assistant_panel_shortcut: String, // default: "Ctrl+Shift+A"

    /// Whether to auto-apply low-risk AI actions without confirmation
    auto_apply_low_risk_actions: bool, // default: false

    /// Request timeouts are read from ai.timeout_* settings in Appendix A
    request_timeout_ms: u32, // derived from ai.timeout_total_ms

    /// Context window management strategy
    context_strategy: ContextStrategy, // default: TruncateOldest

    /// Maximum number of conversations to show in history sidebar
    max_history_display: u32, // default: 50
}

enum ContextStrategy {
    TruncateOldest,
    SummarizeOlder,
}
```

---

## 10. Internal Interfaces

### 10.1 Streaming Token Interface

```rust
trait StreamingProvider {
    /// Start a streaming request. Returns a receiver for tokens.
    fn stream_request(
        &self,
        messages: Vec<ProviderMessage>,
        model: &str,
    ) -> Result<mpsc::Receiver<StreamEvent>>;
}

enum StreamEvent {
    Token { text: String },
    Done { total_tokens: u32 },
    Error { error: ProviderError },
}
```

### 10.2 Conversation Persistence Interface

```rust
trait ConversationStore {
    fn save_conversation(&self, conversation: &Conversation) -> Result<()>;
    fn load_conversation(&self, conversation_id: Uuid) -> Result<Conversation>;
    fn list_conversations(&self, context: Option<ConversationContext>, limit: u32) -> Result<Vec<ConversationSummary>>;
    fn delete_conversation(&self, conversation_id: Uuid) -> Result<()>;
    fn search(&self, query: &str, limit: u32) -> Result<Vec<ConversationSearchResult>>;
    fn append_message(&self, conversation_id: Uuid, message: &Message) -> Result<()>;
    fn update_message(&self, conversation_id: Uuid, message_id: Uuid, content: &str, complete: bool) -> Result<()>;
}

struct ConversationSummary {
    conversation_id: Uuid,
    context: ConversationContext,
    preview: String, // First 100 chars of last user message
    message_count: u32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

### 10.3 Confirmation Dialog Interface

```rust
trait ConfirmationDialog {
    fn show_confirmation(
        &self,
        title: &str,
        description: &str,
        preview: ConfirmationPreview,
    ) -> ConfirmationResult;
}

enum ConfirmationPreview {
    TextDiff { before: String, after: String },
    FileDiff { diff: String },
    Description { text: String },
}

enum ConfirmationResult {
    Confirmed,
    Cancelled,
}
```

---

## 11. State Management

### 11.1 In-Memory State

The following state is held in memory by the AI Surface Controller:

| State Key | Type | Persistence |
|---|---|---|
| `active_surface_states` | `HashMap<Uuid, AiSurfaceState>` | Not persisted (reset on restart) |
| `open_conversations` | `HashMap<Uuid, Conversation>` | Persisted on every message append |
| `registered_actions` | `HashMap<(String, String), AiActionDefinition>` | Not persisted (apps re-register on launch) |
| `assistant_panel_open` | `bool` | Persisted in user preferences |
| `active_conversation_id` | `Option<Uuid>` | Persisted in user preferences |

### 11.2 Persistence Invariants

- Every message is persisted to the server-side conversation store before the surface state transitions to `Complete`, `Error`, `RateLimited`, or `QuotaExceeded`.
- Partial streaming responses are persisted every 2 seconds during streaming to prevent data loss on crash.
- Conversation metadata (`updated_at`, `total_tokens_used`, `message_count`) is updated atomically with the last message.
- Conversation writes are transactional at the database level. A response is not marked complete until the final persisted state commits successfully.

### 11.3 State Transition Rules

1. Only `Idle` can transition to `Loading`.
2. `Loading` can transition to `Streaming`, `Error`, `RateLimited`, or `QuotaExceeded`.
3. `Streaming` can transition to `Complete`, `Error`, `RateLimited`, `QuotaExceeded`, or `Idle` (on cancel).
4. `Complete`, `Error`, `RateLimited`, and `QuotaExceeded` must transition to `Idle` before a new request.
5. On cancel during streaming, the state transitions to `Idle` and the partial response is preserved.

---

## 12. Failure Modes and Error Handling

### 12.1 Provider Unreachable

- **Trigger**: Network failure, DNS resolution failure, or provider server down.
- **Detection**: Connection timeout (5 seconds) or TCP connection refused.
- **Response**: Transition to `Error` state with `ProviderUnreachable`. Show error message with provider name. Log error with request details.
- **Recovery**: User retries manually. No automatic retry to avoid compounding failures.

### 12.2 Authentication Failure

- **Trigger**: HTTP 401 or 403 from provider API.
- **Detection**: Response status code check.
- **Response**: Transition to `Error` state with `AuthenticationFailed`. Show message directing to Settings > AI.
- **Recovery**: User must update API key in settings.

### 12.3 Rate Limiting

- **Trigger**: HTTP 429 from provider API.
- **Detection**: Response status code and `Retry-After` header.
- **Response**: Transition to `RateLimited` state. Display countdown timer.
- **Recovery**: Input re-enables after cooldown. If `Retry-After` header is absent, default cooldown is 60 seconds.

### 12.4 Quota Exceeded

- **Trigger**: Provider-specific quota error (e.g., OpenAI "insufficient_quota").
- **Detection**: Error code in provider response.
- **Response**: Transition to `QuotaExceeded` state. Block new requests until provider is changed or quota resets.
- **Recovery**: User changes provider or waits for quota reset (monthly/daily depending on provider).

### 12.5 Response Timeout

- **Trigger**: No token received within `request_timeout_seconds` (default: 60).
- **Detection**: Timer in the streaming layer.
- **Response**: Cancel the request. Transition to `Error` with `Timeout`. Show timeout message with suggestion to increase timeout.
- **Recovery**: User can retry or adjust timeout in settings.

### 12.6 Streaming Interruption

- **Trigger**: Connection drops during streaming.
- **Detection**: Read error on stream or no token for 30 seconds.
- **Response**: Preserve partial response. Transition to `Error` with a message: "Connection lost during response. Partial response preserved."
- **Recovery**: User can retry. Partial response is visible and copyable.

### 12.7 Conversation Store Failure

- **Trigger**: Disk write failure for conversation persistence.
- **Detection**: Filesystem error on write.
- **Response**: Log error at `error` level. Show a non-blocking notification: "Failed to save conversation. Your messages may not persist."
- **Recovery**: Retries on next message. If 3 consecutive failures occur, disable AI surfaces and show a persistent warning.

---

## 13. Security and Permissions

- The AI surface does not send any user data to providers unless the user explicitly triggers an action.
- Selected text sent to AI is transmitted over HTTPS to the configured provider.
- Conversation history stored locally is not encrypted at rest in v1 (user is responsible for OS-level disk encryption).
- The AI surface cannot access files directly; it requests content through `cortex-files` with appropriate permissions.
- Conversation history search operates locally only; no data is sent to providers for search.
- Provider API keys are stored in the system keyring via `cortex-auth`, never in plaintext configuration files.
- The `ai.show_model_disclosure` setting ensures users can always identify which provider is processing their data.

---

## 14. Performance Requirements

| Metric | Requirement |
|---|---|
| Assistant panel open/close latency | Less than 100ms |
| First token display latency | Less than 500ms after provider begins streaming |
| Token rendering throughput | At least 100 tokens per second rendering rate |
| Conversation history load time | Less than 200ms for a conversation with up to 1000 messages |
| Search query latency | Less than 500ms across 100 conversations |
| Memory usage of assistant panel | Less than 50MB when idle |
| Partial response auto-save interval | Every 2 seconds during streaming |
| Context menu AI submenu display latency | Less than 50ms |

---

## 15. Accessibility Requirements

- The assistant panel must be fully navigable by keyboard (Tab, Enter, Escape).
- Streaming text must be announced via screen reader using ARIA live regions (`aria-live="polite"`).
- The typing indicator must have an accessible label: "AI is generating a response".
- Error messages must use `role="alert"` for screen reader announcement.
- Confirmation dialogs must trap focus within the dialog while open.
- The model disclosure badge must have an accessible description: "Using {provider} model {model}".
- High contrast mode must be supported: error states use distinct shapes/icons in addition to color.
- Minimum contrast ratio for all text in AI surfaces: 4.5:1 (WCAG AA).

---

## 16. Observability and Logging

### 16.1 Log Events

| Event | Level | Fields |
|---|---|---|
| `ai_surface_message_sent` | info | conversation_id, app_id, message_length, provider, model |
| `ai_surface_response_complete` | info | conversation_id, request_id, token_count, duration_ms, provider, model |
| `ai_surface_response_error` | error | conversation_id, request_id, error_type, error_message, provider, model |
| `ai_surface_rate_limited` | warn | conversation_id, provider, retry_after_seconds |
| `ai_surface_quota_exceeded` | warn | conversation_id, provider |
| `ai_surface_action_executed` | info | action_id, app_id, target_type, tokens_used, duration_ms |
| `ai_surface_confirmation_shown` | info | action_id, app_id, risk_level |
| `ai_surface_confirmation_result` | info | action_id, app_id, result (confirmed/cancelled) |
| `ai_surface_streaming_cancelled` | info | conversation_id, request_id, tokens_received |
| `ai_surface_conversation_created` | info | conversation_id, context |
| `ai_surface_conversation_deleted` | info | conversation_id |

### 16.2 Metrics

- `ai_surface_requests_total` (counter, labels: provider, model, outcome)
- `ai_surface_request_duration_seconds` (histogram, labels: provider, model)
- `ai_surface_tokens_received_total` (counter, labels: provider, model)
- `ai_surface_active_conversations` (gauge)
- `ai_surface_errors_total` (counter, labels: error_type, provider)

---

## 17. Testing Requirements

### 17.1 Unit Tests

- State machine: every valid transition must be tested. Every invalid transition must be tested (must panic or return error).
- Conversation persistence: write, read, search, delete operations.
- Streaming renderer: token batching, typing indicator lifecycle, cancel behavior.
- Confirmation dialog: correct preview generation for text diff, file diff, description.
- Context menu actions: correct action list for text selection, file selection, combined selection.
- Provider/model disclosure badge: shown when setting is true, hidden when false, updates on provider change.

### 17.2 Integration Tests

- End-to-end message flow: user sends message -> provider receives request -> tokens stream back -> message saved to conversation.
- Failure scenarios: provider unreachable, auth failure, rate limit, quota exceeded, timeout.
- Multi-turn conversation: context window management, truncation, summarization.
- App AI hooks: register action, appear in context menu, execute, confirm, result display.

### 17.3 UI Tests

- Assistant panel: open/close, keyboard navigation, conversation list, search.
- Context menu: AI submenu appears, actions execute, results display in card.
- Confirmation dialog: focus trapping, cancel, confirm.
- Streaming: token-by-token rendering, stop button, partial response preservation.

### 17.4 Performance Tests

- Send 100 messages in rapid succession; verify no memory leak and all responses render correctly.
- Load a conversation with 1000 messages; verify scroll performance and rendering within 200ms.
- Run search across 100 conversations; verify results within 500ms.

---

## 18. Acceptance Criteria

- [ ] Global assistant panel opens via `Ctrl+Shift+A` and system tray icon.
- [ ] Assistant panel shows conversation history, supports new conversation, and search.
- [ ] Multi-turn conversations maintain context across turns.
- [ ] App-registered AI actions appear in context menus and app toolbars.
- [ ] Selected text context menu includes AI submenu with Summarize, Translate, Rewrite, Explain.
- [ ] Selected files context menu includes AI submenu with Summarize, Analyze.
- [ ] Provider/model disclosure badge is visible in assistant panel, result cards, and status bar.
- [ ] Setting `ai.show_model_disclosure=false` hides the badge.
- [ ] Modifying AI actions require confirmation with preview before executing.
- [ ] Read-only AI actions do not require confirmation.
- [ ] Provider unreachable shows clear error with provider name and suggestion to check settings.
- [ ] Authentication failure shows error with link to Settings > AI.
- [ ] Rate limit shows countdown timer and re-enables input after cooldown.
- [ ] Quota exceeded blocks new requests and shows explanation.
- [ ] Streaming renders tokens as they arrive with typing indicator.
- [ ] "Stop generating" cancels stream and preserves partial response.
- [ ] Conversation history persists across OS restarts.
- [ ] Conversation search returns relevant results across all conversations.
- [ ] All error states are non-crashing and show actionable messages.
- [ ] State machine transitions are correct for all defined paths.
- [ ] Keyboard navigation works for all AI surfaces.
- [ ] Screen reader announces streaming text and error messages.
- [ ] All surfaces support high contrast mode.

---

## 19. Build Order and Dependencies
**Layer 13**. Depends on: 06 (AI runtime), 07 (desktop shell), 16 (theme tokens), 20 (AI safety)

### 19.1 Package / Service Dependencies

```
@cortexos/ai-client depends on:
  - cortex-ai (provider adapter, routing, streaming interface)
  - cortex-files (file access for file-based AI actions)
  - cortex-settings (user preferences for AI surface settings)
  - cortex-policy (permission and confirmation checks for AI actions)
  - @cortexos/ui-components (panels, dialogs, menus)
  - cortex-search (full-text search for conversation history)

cortex-ai must be built before @cortexos/ai-client.
cortex-files, cortex-settings, cortex-policy, and @cortexos/ui-components must be available before @cortexos/ai-client is integrated.
```

### 19.2 Build Order

1. `cortex-core` (shared types)
2. `cortex-policy` (permission framework)
3. `cortex-files` (filesystem abstraction)
4. `cortex-settings` (settings infrastructure)
5. `cortex-search` (search primitives)
6. `@cortexos/ui-components` (UI component library)
7. `cortex-ai` (runtime/provider layer)
8. `@cortexos/ai-client` (this package)

---

## 20. Non-Goals and Anti-Patterns

### 20.1 Non-Goals

- Image or multimodal AI input/output in v1.
- Voice-based AI interaction.
- Autonomous AI agent behaviors (AI must always be user-initiated).
- AI model training or customization within the OS.
- Cross-device conversation syncing.
- Real-time collaborative AI conversations.
- Plugin system for adding new AI providers at runtime (providers are configured, not installed).

### 20.2 Anti-Patterns

- **Silent failures**: AI errors must never be silently swallowed. Every error must be surfaced to the user.
- **Blocking UI**: AI requests must never block the UI thread. All provider communication is async.
- **Stale state**: The UI must never show a loading indicator for a completed or failed request.
- **Hardcoded providers**: Provider names, endpoints, and models must be configurable, not hardcoded.
- **Guessing user intent**: The AI surface must not send requests without explicit user action. No proactive AI suggestions in v1.
- **Unconfirmed modifications**: No AI action that modifies user data may execute without explicit confirmation, regardless of how "safe" it seems.
- **Storing API keys in plain text**: API keys must always be stored in the system keyring.
- **Ignoring model boundaries**: Conversation context must respect the model's maximum context window; it must not silently truncate without informing the user.
- **Persistent loading states**: If a request fails, the loading state must be cleared within 1 second.
- **Modal blocking for AI**: The assistant panel must not block interaction with the underlying application.

---

## 21. Implementation Instructions for Claude Code / Codex

### 21.1 File Structure

```text
packages/ai-client/
  src/
    controller.ts          # AiSurfaceController: central orchestrator
    stateMachine.ts        # AiSurfaceState transitions and validation
    streaming.ts           # StreamingRenderer: token batching and rendering
    conversation/
      store.ts             # ConversationStore client over server-side persistence
      search.ts            # Full-text conversation search client
      types.ts             # Conversation, Message, ConversationContext types
    hooks/
      registry.ts          # AiHooksRegistry implementation
      action.ts            # AiActionDefinition, action execution logic
    ui/
      AssistantPanel.tsx
      ContextMenu.tsx
      ResultCard.tsx
      ConfirmationDialog.tsx
      DisclosureBadge.tsx
    settings.ts            # AiSurfaceSettings and setting key definitions
    errors.ts              # AiSurfaceError types
```

### 21.2 Implementation Order

1. **Phase 1 - Types and State Machine** (`state_machine.rs`, `error.rs`, `conversation/types.rs`):
   - Define all types from Section 8.
   - Implement `AiSurfaceState` with validated transitions.
   - Write exhaustive unit tests for all transitions (valid and invalid).

2. **Phase 2 - Conversation Store** (`conversation/store.ts`, `conversation/search.ts`):
   - Implement `ConversationStore` client against the authoritative server-side SQLite-backed conversation store.
   - Implement full-text search using `cortex-search`.
   - Test persistence, retrieval, search, and deletion.

3. **Phase 3 - Streaming Renderer** (`streaming.rs`):
   - Implement `StreamingRenderer` with 50ms token batching.
   - Typing indicator lifecycle management.
   - Cancel/stop behavior.
   - Test with simulated token streams.

4. **Phase 4 - Controller** (`controller.rs`):
   - Implement `AiSurfaceController` connecting all components.
   - State machine management on top of store and streaming.
   - Integration with `cortex-ai` runtime.
   - Test full request lifecycle including failures.

5. **Phase 5 - Hooks Registry** (`hooks/registry.rs`, `hooks/action.rs`):
   - Implement `AiHooksRegistry`.
   - Action registration, unregistration, listing.
   - Action execution with confirmation flow.
   - Test with mock app actions.

6. **Phase 6 - UI Components** (`ui/`):
   - Assistant panel (open/close, conversation list, input, streaming display).
   - Context menu AI submenu.
   - Result card with Copy/Replace/Dismiss.
   - Confirmation dialog with preview.
   - Disclosure badge.
   - Test all components with keyboard navigation and screen reader compatibility.

7. **Phase 7 - Settings Integration** (`settings.rs`):
   - Load `ai.show_model_disclosure`, shortcuts, timeout, context strategy.
   - React to setting changes at runtime.

### 21.3 Key Implementation Notes

- The UI layer must use the project-standard async primitives for browser code and must never block rendering while waiting on AI responses.
- Conversation persistence is server-authoritative. Client code must never treat local cache as the source of truth.
- The streaming renderer must never hold a lock on the UI thread.
- Use UUID v4 identifiers for conversations and messages.
- Use UTC timestamps everywhere.
- Error messages shown to users must come from a dedicated message catalog, not from error enum string conversion. This ensures consistent, human-readable messages.
- The `AiSurfaceController` must be a singleton per user session.
- All UI components must use `@cortexos/ui-components` accessibility patterns.
- Register a graceful shutdown handler that flushes any in-flight client state and waits for final persistence acknowledgement from the server when possible.

### 21.4 Configuration Defaults

All defaults must be defined as constants in `settings.rs`:

```rust
const DEFAULT_SHOW_MODEL_DISCLOSURE: bool = true;
const DEFAULT_ASSISTANT_SHORTCUT: &str = "Ctrl+Shift+A";
const DEFAULT_AUTO_APPLY_LOW_RISK: bool = false;
const DEFAULT_REQUEST_TIMEOUT_MS: u32 = 120_000;
const DEFAULT_CONTEXT_STRATEGY: ContextStrategy = ContextStrategy::TruncateOldest;
const DEFAULT_MAX_HISTORY_DISPLAY: u32 = 50;
const STREAMING_DEBOUNCE_MS: u64 = 50;
const PARTIAL_SAVE_INTERVAL_SECS: u64 = 2;
```
