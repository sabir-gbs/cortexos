# 05 — Settings Service and Settings App

## 1. Purpose

Define the centralized settings service that manages all OS-level, user-level, and AI configuration. Every subsystem reads its configuration through this service. The Settings app provides the user-facing interface for managing all settings.

## 2. Scope

- Settings schema: namespaces, keys, types, defaults, constraints
- Settings resolution algorithm (layered defaults)
- Settings REST API
- AI-specific settings fields (critical product requirement)
- Settings app information architecture and behavior
- Change notification (real-time updates via WebSocket)

## 3. Out of Scope

- Per-app internal state (owned by apps, not settings)
- Filesystem configuration (owned by spec 11)
- AI provider adapter logic (owned by spec 06, reads settings only)

## 4. Objectives

1. Every configurable value in the OS has exactly one canonical setting key.
2. The resolution algorithm is deterministic: given the same inputs, always produces the same effective value.
3. AI settings support the full product requirement: preferred LLM, per-app overrides, per-feature overrides, fallback chain, privacy, budget.
4. Settings changes propagate to all subscribers within 500ms.

## 5. User-Visible Behavior

| Action | Outcome |
|---|---|
| User opens Settings app | Categories displayed in sidebar, selected category content in main area |
| User changes a setting | Change applied immediately, propagated to all affected subsystems |
| User changes AI provider | All AI requests immediately route to new provider |
| User changes theme | Theme switches instantly, no page reload |
| User exports settings | JSON file downloaded with all user settings |
| User imports settings | Settings validated, applied, invalid keys rejected with error messages |

## 6. System Behavior

### 6.1 Settings Resolution Order

Given a setting key `ns.key`, the effective value is determined by:

1. **App-level override**: If an app_id is in context AND the app has an override for this key → use app override
2. **User-level setting**: If the user has set a value for this key → use user value
3. **System-wide setting**: If an admin has set a system-wide value → use system value
4. **Hardcoded default**: The schema-defined default value

Last match wins. Resolution stops at the first layer that has a value.

### 6.2 Change Propagation

When a setting changes:
1. Persist to SQLite
2. Emit `settings.changed` event on WebSocket channel `settings`
3. All subscribers (including the same client's other windows) receive the event
4. Subscribers re-read the effective value and update

### 6.3 Validation

Every setting mutation passes schema validation before persistence. Invalid values are rejected with `VAL_003` error including the constraint that failed.

## 7. Architecture

```
┌──────────────────────┐
│   Settings App (TS)  │
│   (reads/writes API) │
└──────────┬───────────┘
           │ REST /api/v1/settings/*
┌──────────┴───────────┐
│   cortex-settings    │
│  ┌─────────────────┐ │
│  │ Schema Registry │ │
│  │ (type, default, │ │
│  │  constraints)   │ │
│  └────────┬────────┘ │
│  ┌────────┴────────┐ │
│  │ Resolution      │ │
│  │ Engine          │ │
│  └────────┬────────┘ │
│  ┌────────┴────────┐ │
│  │ Store (SQLite)  │ │
│  └─────────────────┘ │
└──────────────────────┘
```

## 8. Data Model

### 8.1 Setting Schema Entry

```rust
struct SettingSchema {
    key: String,              // e.g. "ai.preferred_provider"
    namespace: String,        // e.g. "ai"
    value_type: SettingType,
    default_value: serde_json::Value,
    constraints: Vec<Constraint>,
    description: String,
    read_only: bool,          // System settings may be read-only
    requires_restart: bool,   // Some changes need restart
}

enum SettingType {
    String,
    Integer,
    Float,
    Boolean,
    Enum(Vec<String>),
    Array(Box<SettingType>),
    Map(String, Box<SettingType>),  // key → value type
    Object(Vec<(String, SettingType)>),
}

enum Constraint {
    Min(i64),
    Max(i64),
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),          // Regex
    OneOf(Vec<serde_json::Value>),
    Custom(String),           // Named validator function
}
```

### 8.2 Stored Setting

```rust
struct StoredSetting {
    key: String,
    scope: SettingScope,
    scope_id: Option<String>,  // user_id or app_id
    value: serde_json::Value,
    updated_at: chrono::DateTime<chrono::Utc>,
    updated_by: String,        // user_id or "system"
}

enum SettingScope {
    System,    // System-wide default override
    User,      // Per-user setting
    App,       // Per-app override (scope_id = app_id)
}
```

### 8.3 AI Settings Fields (CRITICAL — Complete List)

| Key | Type | Default | Valid Range | Description |
|---|---|---|---|---|
| `ai.preferred_provider` | Enum | `null` | `OpenAI`, `Anthropic`, `Google`, `Ollama`, `Zhipu`, `Custom`, `null` | OS-level preferred AI provider |
| `ai.preferred_model` | String | `""` | Any non-empty string or `""` | OS-level preferred model ID (empty = use provider default) |
| `ai.fallback_enabled` | Boolean | `false` | `true`, `false` | Enable fallback to alternative providers on failure |
| `ai.fallback_chain` | Array(Enum) | `[]` | Ordered list of provider enums | Provider fallback order; first = primary, last = last resort |
| `ai.per_app_overrides` | Map(String, Object) | `{}` | Key=app_id, Value={provider,model} | Per-app provider/model overrides |
| `ai.per_feature_overrides` | Map(String, Object) | `{}` | Key=feature_name, Value={provider,model} | Per-feature provider/model overrides (e.g., "summarize", "translate") |
| `ai.privacy_mode` | Enum | `"none"` | `none`, `redact_pii`, `hash_pii` | How PII is handled in AI requests/logs |
| `ai.allow_file_access` | Boolean | `false` | `true`, `false` | Whether AI can read file contents (subject to per-file permissions) |
| `ai.allow_clipboard_access` | Boolean | `false` | `true`, `false` | Whether AI can read clipboard contents |
| `ai.budget_policy` | Object | `{"enabled":false}` | `{enabled:bool, daily_limit_cents:u64, warn_threshold_pct:u8}` | AI spending budget (cents in USD) |
| `ai.show_model_disclosure` | Boolean | `true` | `true`, `false` | Show which provider/model is being used in AI surfaces |
| `ai.timeout_connect_ms` | Integer | `10000` | `1000`-`60000` | Max time to establish provider connection |
| `ai.timeout_first_token_ms` | Integer | `30000` | `1000`-`120000` | Max wait for first streaming token |
| `ai.timeout_total_ms` | Integer | `120000` | `1000`-`600000` | Max total runtime for one AI request |
| `ai.timeout_stream_idle_ms` | Integer | `60000` | `1000`-`300000` | Max idle gap between streaming tokens |
| `ai.audit_retention_days` | Integer | `90` | `1`-`3650` | Retention period for AI audit records |
| `ai.auto_apply_low_risk_actions` | Boolean | `false` | `true`, `false` | Skip confirmation for explicitly low-risk AI actions |

### 8.4 Complete Settings Namespace Registry

| Namespace | Description | Example Keys |
|---|---|---|
| `system` | OS-level configuration | `system.version`, `system.build_number` |
| `user` | User profile preferences | `user.display_name`, `user.avatar`, `user.language` |
| `ai` | AI runtime configuration | (see 8.3 above) |
| `display` | Visual settings | `display.theme`, `display.font_scale`, `display.animation_enabled` |
| `accessibility` | Accessibility settings | `accessibility.high_contrast`, `accessibility.reduce_motion`, `accessibility.keyboard_nav` |
| `privacy` | Privacy controls | `privacy.telemetry_enabled`, `privacy.crash_reports` |
| `apps` | App-specific settings | `apps.default_text_editor`, `apps.file_associations` |
| `network` | Network settings | `network.proxy_enabled`, `network.proxy_url` |

## 9. Public Interfaces

### 9.1 REST API

```
GET    /api/v1/settings/{namespace}/{key}         → Get effective value
GET    /api/v1/settings/{namespace}                → List all in namespace
PUT    /api/v1/settings/{namespace}/{key}          → Set value (user scope)
PUT    /api/v1/settings/batch                       → Set multiple values
DELETE /api/v1/settings/{namespace}/{key}          → Reset to default
GET    /api/v1/settings/schema/{namespace}/{key}   → Get schema info
GET    /api/v1/settings/export                      → Export all user settings
POST   /api/v1/settings/import                      → Import settings (validated)
```

### 9.2 WebSocket Events

```
settings.changed  → { key: string, scope: SettingScope, new_value: value, old_value: value }
```

### 9.3 Internal Rust API

```rust
trait SettingsService: Send + Sync {
    fn get_effective(&self, key: &str, user_id: &UserId, app_id: Option<&str>) -> Result<serde_json::Value>;
    fn set(&self, key: &str, scope: SettingScope, scope_id: Option<&str>, value: serde_json::Value) -> Result<()>;
    fn delete(&self, key: &str, scope: SettingScope, scope_id: Option<&str>) -> Result<()>;
    fn subscribe(&self, key: &str, callback: Box<dyn Fn(SettingsChangeEvent) + Send + Sync>) -> Result<()>;
    fn get_schema(&self, key: &str) -> Result<&SettingSchema>;
    fn export_all(&self, user_id: &UserId) -> Result<HashMap<String, serde_json::Value>>;
    fn import_all(&self, user_id: &UserId, settings: HashMap<String, serde_json::Value>) -> Result<Vec<ImportError>>;
}
```

## 10. Internal Interfaces

- `cortex-settings` reads from and writes to `cortex-db` (SQLite table `settings`)
- `cortex-settings` emits events via the command bus (spec 10)
- `cortex-ai` reads AI settings through the `SettingsService` trait
- `cortex-policy` reads permission-related settings
- All settings reads are cached in-memory with invalidation on change events

## 11. State Management

- Settings stored in SQLite table: `(key TEXT, scope TEXT, scope_id TEXT, value JSON, updated_at TEXT, updated_by TEXT)`
- Unique constraint: `(key, scope, scope_id)`
- In-memory LRU cache for hot settings (max 1000 entries, TTL 60s)
- Cache invalidated on write or on `settings.changed` event

## 12. Failure Modes and Error Handling

| Failure | Handling |
|---|---|
| Invalid setting value | Reject with VAL_003, include constraint that failed |
| Unknown setting key | Reject with NF_001 |
| Read-only setting write attempt | Reject with POL_001 |
| Schema validation failure | Reject with VAL_003, include expected type |
| DB write failure | Return TRANS_001, auto-retry 3x |
| Import with invalid keys | Accept valid keys, return list of rejected keys with reasons |

## 13. Security and Permissions

- System-scope settings require `system.settings.write` permission
- User-scope settings require only authenticated session
- App-scope overrides require `system.settings.app_override` permission
- Settings API key values (e.g., AI API keys) are stored encrypted at rest, never returned in export
- `GET /api/v1/settings/export` excludes keys matching `*.secret`, `*.api_key`, `*.token` patterns

## 14. Performance Requirements

| Metric | Target |
|---|---|
| Single setting read (cached) | < 1ms |
| Single setting read (DB) | < 10ms |
| Setting write (including event emit) | < 50ms |
| Full namespace list | < 100ms |
| Settings export | < 500ms |
| Change propagation to subscriber | < 500ms |

## 15. Accessibility Requirements

- Settings app fully keyboard-navigable (Tab through categories, Arrow keys within lists)
- All setting labels have associated form controls
- Error messages appear adjacent to the relevant setting
- Color is never the sole indicator of setting state (always accompanied by text/icon)

## 16. Observability and Logging

- Every setting write logged at INFO: `{key, scope, scope_id, old_value_hash, new_value_hash}` (hash, not raw value, for sensitive settings)
- Schema validation failures logged at WARN
- Import operations logged with count of accepted/rejected keys
- Cache hit/miss ratios exposed as metric: `settings_cache_hit_ratio`

## 17. Testing Requirements

- Unit: resolution algorithm with all 4 layers
- Unit: schema validation for each constraint type
- Unit: every AI setting key validated against its schema
- Integration: REST API CRUD for all scopes
- Integration: change event propagation via WebSocket
- E2E: Settings app flow — change AI provider, verify routing updates

## 18. Acceptance Criteria

- [ ] All 11 AI settings fields have complete schema entries
- [ ] Resolution algorithm produces correct values for all scope combinations
- [ ] Settings changes propagate via WebSocket within 500ms
- [ ] Settings app displays all categories with correct controls for each type
- [ ] Invalid values rejected with descriptive error messages
- [ ] Export/import round-trips without data loss (excluding secret fields)
- [ ] Cache invalidation works on change events
- [ ] No setting value logged in plaintext for sensitive keys

## 19. Build Order and Dependencies

**Layer 4**. Depends on:
- 01 — Repository conventions
- 02 — Core architecture (error taxonomy)
- 03 — Identity/auth (user context)
- 04 — Permissions (setting write permissions)

Blocks:
- 06 — AI runtime (reads AI settings)
- 07 — Desktop shell (reads display settings)
- 15 — Accessibility (reads accessibility.* settings)
- 16 — Theme (reads display.theme)

## 20. Non-Goals and Anti-Patterns

**Non-Goals**:
- Multi-user settings negotiation (single-user v1)
- Settings versioning/history (v1 — only current value stored)
- Settings sync across devices

**Anti-Patterns**:
- NEVER hardcode setting values in code — always read from settings service
- NEVER store secrets in plaintext in the settings table
- NEVER allow apps to read settings from other apps' namespaces without permission
- NEVER skip schema validation on writes

## 21. Implementation Instructions for Claude Code / Codex

1. Define `SettingSchema` enum and all 11 AI settings schemas as a static registry in `cortex-settings`.
2. Implement the SQLite store with the `settings` table and unique constraint.
3. Implement the resolution engine: `get_effective(key, user_id, app_id)` checking app→user→system→default.
4. Implement the REST API endpoints with schema validation middleware.
5. Implement change event emission on every write.
6. Build the Settings app UI: sidebar categories, main area with typed form controls per setting type.
7. Write tests: resolution algorithm, schema validation, API CRUD, change propagation.
