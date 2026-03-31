# Appendix A: Required AI Settings Fields

## Purpose

This appendix provides a complete, authoritative reference for every AI-related settings field in CortexOS. Each field is documented with its full type signature, default value, valid range, consuming component, effective timing, and validation rules. No other document may define contradictory defaults or behavior for these fields.

## Field Index

| # | Field Name | Type | Section |
|---|-----------|------|---------|
| 1 | ai.preferred_provider | enum | A.1 |
| 2 | ai.preferred_model | string | A.2 |
| 3 | ai.fallback_enabled | boolean | A.3 |
| 4 | ai.fallback_chain | array of enum | A.4 |
| 5 | ai.per_app_overrides | map | A.5 |
| 6 | ai.per_feature_overrides | map | A.6 |
| 7 | ai.privacy_mode | enum | A.7 |
| 8 | ai.allow_file_access | boolean | A.8 |
| 9 | ai.allow_clipboard_access | boolean | A.9 |
| 10 | ai.budget_policy | object | A.10 |
| 11 | ai.show_model_disclosure | boolean | A.11 |
| 12 | ai.timeout_connect_ms | integer | A.12 |
| 13 | ai.timeout_first_token_ms | integer | A.13 |
| 14 | ai.timeout_total_ms | integer | A.14 |
| 15 | ai.timeout_stream_idle_ms | integer | A.15 |
| 16 | ai.audit_retention_days | integer | A.16 |
| 17 | ai.auto_apply_low_risk_actions | boolean | A.17 |

---

## A.1. ai.preferred_provider

| Property | Value |
|----------|-------|
| **Field Name** | `ai.preferred_provider` |
| **Type** | `enum` |
| **Enum Variants** | `"OpenAI"`, `"Anthropic"`, `"Google"`, `"Ollama"`, `"Zhipu"`, `"Custom"` |
| **Default Value** | `null` (no provider selected; AI features disabled until configured) |
| **Nullable** | Yes (`null` is the default and is valid) |
| **Read By** | `cortex-ai` (provider routing), `cortex-settings` (settings UI), `apps/settings-app` (settings form) |
| **Takes Effect** | Immediately upon save. The AI runtime reads this field on every request dispatch. No restart required. |
| **Storage Location** | Settings store, namespace `ai` |
| **Settings Level** | User-level (per-user setting, not system-wide) |

### Description

The primary AI provider for all AI requests in CortexOS. When set to a non-null value, all AI requests route to this provider unless overridden by `ai.per_app_overrides` or `ai.per_feature_overrides`. When set to `null`, AI features are disabled and the AI assistant panel displays a configuration prompt.

### Validation Rules

1. The value must be one of the six enum variants or `null`.
2. Setting a provider does not validate that the provider is reachable. Provider reachability is checked at request time, not at save time.
3. Setting the value to a provider that has no configured credentials will result in an authentication error at request time. This is acceptable; the setting save must not be blocked.
4. The value `"Ollama"` implies a local provider. The AI runtime must attempt to connect to the default Ollama endpoint (`http://localhost:11434`) unless a custom endpoint is configured via the provider registry.
5. The value `"Zhipu"` selects the first-class Zhipu / Z.ai provider adapter and uses the configured provider registry entry for endpoint and credentials.
6. The value `"Custom"` requires a corresponding entry in the provider registry (spec 06) with a user-defined endpoint URL. If no custom provider is registered, requests will fail at dispatch time.

### UI Behavior

- Rendered as a dropdown/radio group in the Settings app under the AI section.
- Options shown: "None" (null), "OpenAI", "Anthropic", "Google", "Ollama", "Zhipu", "Custom".
- When the user selects a provider that requires an API key, the settings form immediately shows an API key input field.
- When the user selects "Ollama", the settings form shows an optional endpoint URL field (defaulting to `http://localhost:11434`).
- When the user selects "Zhipu", the settings form shows the configured endpoint/profile label and an API key field if required by the adapter.
- When the user selects "Custom", the settings form shows a required endpoint URL field and an optional API key field.

---

## A.2. ai.preferred_model

| Property | Value |
|----------|-------|
| **Field Name** | `ai.preferred_model` |
| **Type** | `string` |
| **Default Value** | `""` (empty string; uses the provider's default model) |
| **Nullable** | No (empty string is used instead of null) |
| **Max Length** | 128 characters |
| **Read By** | `cortex-ai` (model routing), `cortex-settings` (settings UI), `apps/settings-app` (settings form) |
| **Takes Effect** | Immediately upon save. The AI runtime reads this field on every request dispatch. No restart required. |
| **Storage Location** | Settings store, namespace `ai` |
| **Settings Level** | User-level |

### Description

The preferred model identifier for AI requests. When non-empty, the AI runtime passes this model identifier to the selected provider. When empty, the AI runtime uses the provider's default model (defined in the provider registry, spec 06).

### Validation Rules

1. The value must be a string of 0 to 128 characters.
2. The value is not validated against a known model list. Model identifiers are provider-specific and may change outside CortexOS control.
3. If the specified model is not available on the selected provider, the provider returns an error at request time. This error is surfaced to the user. The setting save is not blocked.
4. The value must be trimmed (no leading or trailing whitespace). The settings service strips whitespace on save.
5. The value must not contain control characters (ASCII 0-31). The settings service rejects values containing control characters.

### UI Behavior

- Rendered as a text input in the Settings app under the AI section.
- Label: "Preferred Model (optional)"
- Help text: "Leave empty to use the provider's default model. Enter a specific model identifier (e.g., gpt-4, claude-3-opus, gemini-pro)."
- The field is visible only when `ai.preferred_provider` is set to a non-null value.
- The field may optionally show a dropdown of known models for the selected provider (populated from the provider registry), but must also allow free-text entry.

---

## A.3. ai.fallback_enabled

| Property | Value |
|----------|-------|
| **Field Name** | `ai.fallback_enabled` |
| **Type** | `boolean` |
| **Default Value** | `false` |
| **Valid Values** | `true`, `false` |
| **Read By** | `cortex-ai` (fallback routing), `cortex-settings` (settings UI), `apps/settings-app` (settings form) |
| **Takes Effect** | Immediately upon save. Affects the next AI request dispatched. No restart required. |
| **Storage Location** | Settings store, namespace `ai` |
| **Settings Level** | User-level |

### Description

Controls whether the AI runtime will attempt fallback providers when the primary provider fails. When `true`, the AI runtime consults `ai.fallback_chain` on provider failure and attempts the next provider in the chain. When `false`, provider failures are immediately surfaced to the user with no retry.

### Validation Rules

1. Must be a boolean value (`true` or `false`).
2. Setting this to `true` when `ai.fallback_chain` is empty is valid but has no effect (no fallback providers configured).
3. Setting this to `true` when `ai.preferred_provider` is `null` is valid but has no effect (no primary provider configured).

### UI Behavior

- Rendered as a toggle switch in the Settings app under the AI section.
- Label: "Enable Provider Fallback"
- Help text: "When enabled, if your preferred AI provider is unavailable, CortexOS will try the next provider in your fallback chain."
- The toggle is visible only when `ai.preferred_provider` is set to a non-null value.
- When toggled ON, the fallback chain configuration UI becomes visible.

---

## A.4. ai.fallback_chain

| Property | Value |
|----------|-------|
| **Field Name** | `ai.fallback_chain` |
| **Type** | `array of enum` |
| **Element Type** | Same enum as `ai.preferred_provider`: `"OpenAI"`, `"Anthropic"`, `"Google"`, `"Ollama"`, `"Zhipu"`, `"Custom"` |
| **Default Value** | `[]` (empty array) |
| **Max Length** | 5 elements |
| **Read By** | `cortex-ai` (fallback routing), `cortex-settings` (settings UI), `apps/settings-app` (settings form) |
| **Takes Effect** | Immediately upon save. Affects the next fallback attempt. No restart required. |
| **Storage Location** | Settings store, namespace `ai` |
| **Settings Level** | User-level |

### Description

An ordered list of providers to attempt when the primary provider fails and `ai.fallback_enabled` is `true`. The AI runtime attempts each provider in order until one succeeds or all have been exhausted. The chain is traversed from index 0 to the last element.

### Fallback Behavior

1. Primary provider (`ai.preferred_provider`) is always tried first, regardless of this chain.
2. If the primary provider fails (network error, timeout, authentication error, rate limit, or model not found), the AI runtime checks `ai.fallback_enabled`.
3. If `ai.fallback_enabled` is `true` and `ai.fallback_chain` is non-empty, the runtime attempts provider at index 0.
4. If that provider also fails, the runtime attempts index 1, and so on.
5. If all providers in the chain fail, the error from the last attempted provider is surfaced to the user.
6. Duplicate providers in the chain are valid but result in repeated attempts to the same provider. The settings UI should warn about duplicates but must not prevent saving.
7. The `ai.preferred_provider` appearing in the fallback chain is valid but results in a redundant attempt. The settings UI should warn about this but must not prevent saving.

### Validation Rules

1. Each element must be one of the six provider enum variants.
2. Array length must be 0 to 5 inclusive.
3. `null` elements are not allowed within the array.
4. The array may contain duplicates (valid but warned).
5. The array may contain the same value as `ai.preferred_provider` (valid but warned).

### UI Behavior

- Rendered as a sortable list of providers in the Settings app.
- Visible only when `ai.fallback_enabled` is `true`.
- User can add providers from a dropdown and reorder them via drag-and-drop or up/down buttons.
- User can remove providers from the chain.
- Visual warning (yellow) shown for duplicate entries.
- Visual warning (yellow) shown if the chain includes the primary provider.

---

## A.5. ai.per_app_overrides

| Property | Value |
|----------|-------|
| **Field Name** | `ai.per_app_overrides` |
| **Type** | `map of string -> object` |
| **Key Type** | App ID (string matching `[a-z][a-z0-9-]*`, max 64 chars) |
| **Value Type** | `{ provider: enum, model: string }` |
| **Default Value** | `{}` (empty map) |
| **Max Entries** | 50 |
| **Read By** | `cortex-ai` (per-app routing), `cortex-settings` (settings UI), `apps/settings-app` (settings form) |
| **Takes Effect** | Immediately upon save. Affects AI requests from the specified app. No restart required. |
| **Storage Location** | Settings store, namespace `ai` |
| **Settings Level** | User-level |

### Description

Per-app overrides for AI provider and model selection. When an app dispatches an AI request, the AI runtime checks this map for the app's ID. If a match is found, the specified provider and model are used instead of the global `ai.preferred_provider` and `ai.preferred_model`.

### Value Object Structure

```
{
  provider: "OpenAI" | "Anthropic" | "Google" | "Ollama" | "Zhipu" | "Custom",
  model: string  // empty string means provider default
}
```

### Routing Precedence (from highest to lowest)

1. `ai.per_feature_overrides` (feature-specific)
2. `ai.per_app_overrides` (app-specific)
3. `ai.preferred_provider` / `ai.preferred_model` (global)

### Validation Rules

1. Keys must be valid app IDs: lowercase alphanumeric plus hyphens, starting with a letter, max 64 characters.
2. The `provider` field in the value object must be one of the six provider enum variants. It must not be null.
3. The `model` field must be a string of 0 to 128 characters. It may be empty (meaning use the provider default).
4. The map must not contain more than 50 entries.
5. App IDs that do not correspond to installed apps are allowed (they simply have no effect until the app is installed).
6. The `provider` field in the override does not require credentials to be configured at save time. Errors surface at request time.

### UI Behavior

- Rendered as a list of app-specific overrides in the Settings app.
- Each override shows: app name (or app ID if name not available), provider dropdown, model text input.
- User can add a new override by selecting an installed app from a dropdown.
- User can remove an override.
- Help text: "Override the AI provider and model for specific apps. These take precedence over your global preferred provider."

---

## A.6. ai.per_feature_overrides

| Property | Value |
|----------|-------|
| **Field Name** | `ai.per_feature_overrides` |
| **Type** | `map of string -> object` |
| **Key Type** | Feature name (string matching `[a-z][a-z0-9_-]*`, max 64 chars) |
| **Value Type** | `{ provider: enum, model: string }` |
| **Default Value** | `{}` (empty map) |
| **Max Entries** | 50 |
| **Read By** | `cortex-ai` (per-feature routing), `cortex-settings` (settings UI), `apps/settings-app` (settings form) |
| **Takes Effect** | Immediately upon save. Affects AI requests tagged with the specified feature name. No restart required. |
| **Storage Location** | Settings store, namespace `ai` |
| **Settings Level** | User-level |

### Description

Per-feature overrides for AI provider and model selection. When an AI request is dispatched with a feature tag, the AI runtime checks this map for the feature name. If a match is found, the specified provider and model are used. This is the highest-priority routing override.

### Defined Feature Names (v1)

| Feature Name | Description |
|-------------|-------------|
| `global_assistant` | The global AI assistant panel |
| `text_completion` | Text completion in editors |
| `text_summarization` | Summarization of selected text |
| `code_assistance` | Code-related AI features |
| `translation` | Translation features |
| `smart_search` | AI-enhanced search |
| `image_description` | Image description/analysis |

Additional feature names may be added by first-party apps and third-party apps via the SDK (spec 21).

### Value Object Structure

```
{
  provider: "OpenAI" | "Anthropic" | "Google" | "Ollama" | "Zhipu" | "Custom",
  model: string  // empty string means provider default
}
```

### Validation Rules

1. Keys must be valid feature names: lowercase alphanumeric plus hyphens and underscores, starting with a letter, max 64 characters.
2. The `provider` field in the value object must be one of the six provider enum variants. It must not be null.
3. The `model` field must be a string of 0 to 128 characters. It may be empty.
4. The map must not contain more than 50 entries.
5. Feature names that are not in the known set are allowed (they may be used by future apps).

### UI Behavior

- Rendered as a list of feature-specific overrides in the Settings app.
- Each override shows: feature name (with human-readable label), provider dropdown, model text input.
- User can add a new override by selecting a feature from a predefined list or entering a custom feature name.
- User can remove an override.
- Help text: "Override the AI provider and model for specific features. These take priority over app-specific and global settings."

---

## A.7. ai.privacy_mode

| Property | Value |
|----------|-------|
| **Field Name** | `ai.privacy_mode` |
| **Type** | `enum` |
| **Enum Variants** | `"none"`, `"redact_pii"`, `"hash_pii"` |
| **Default Value** | `"none"` |
| **Nullable** | No |
| **Read By** | `cortex-ai` (request preprocessing), `cortex-settings` (settings UI), `apps/settings-app` (settings form) |
| **Takes Effect** | Immediately upon save. Affects preprocessing of the next AI request. No restart required. |
| **Storage Location** | Settings store, namespace `ai` |
| **Settings Level** | User-level |

### Description

Controls how the AI runtime preprocesses user data before sending it to an external AI provider. This setting has no effect when using a local provider (`"Ollama"` or `"Custom"` pointing to localhost), as data does not leave the machine.

### Mode Definitions

| Mode | Behavior |
|------|----------|
| `"none"` | No preprocessing. User data is sent to the provider as-is. |
| `"redact_pii"` | The AI runtime attempts to detect and redact personally identifiable information before sending. Detected PII is replaced with placeholder tokens (e.g., `[EMAIL]`, `[PHONE]`, `[NAME]`, `[SSN]`, `[CREDIT_CARD]`). Placeholders are restored in the response if present. |
| `"hash_pii"` | The AI runtime detects PII and replaces it with deterministic SHA-256 hashes (truncated to 16 hex characters). Hashes are stored locally for the duration of the session to enable restoration in responses. Hashes are not persisted across sessions. |

### PII Detection Categories

The following categories are detected when `privacy_mode` is not `"none"`:

- Email addresses
- Phone numbers (US and international formats)
- Social security numbers
- Credit card numbers
- Dates of birth
- Street addresses (best-effort, US format)
- IP addresses

### Validation Rules

1. Must be one of the three enum variants.
2. PII detection is best-effort, not guaranteed. The settings UI must display a disclaimer: "PII detection is best-effort and may not catch all personally identifiable information."
3. When set to `"redact_pii"` or `"hash_pii"`, the AI runtime logs (at debug level) how many PII instances were detected and redacted/hashed per request. It does not log the original PII values.

### UI Behavior

- Rendered as a radio group or segmented control in the Settings app.
- Label: "Privacy Mode"
- Options: "Off", "Redact PII", "Hash PII"
- Disclaimer text visible when "Redact PII" or "Hash PII" is selected.
- Help text: "Controls how your data is preprocessed before being sent to external AI providers. Has no effect when using local providers."

---

## A.8. ai.allow_file_access

| Property | Value |
|----------|-------|
| **Field Name** | `ai.allow_file_access` |
| **Type** | `boolean` |
| **Default Value** | `false` |
| **Valid Values** | `true`, `false` |
| **Read By** | `cortex-ai` (context attachment), `cortex-policy` (permission enforcement), `cortex-settings` (settings UI), `apps/settings-app` (settings form) |
| **Takes Effect** | Immediately upon save. Affects whether the AI runtime can attach file contents to AI requests. No restart required. |
| **Storage Location** | Settings store, namespace `ai` |
| **Settings Level** | User-level |

### Description

Controls whether the AI runtime is permitted to read and attach file contents to AI requests as context. When `false`, the AI runtime must not read any file contents, even if an AI action or feature requests file context. When `true`, the AI runtime may read file contents subject to additional per-file permission checks enforced by `cortex-policy`.

### Validation Rules

1. Must be a boolean value.
2. This is a global toggle. Even when `true`, individual file access still requires the user to grant permission via the policy system (spec 04). This setting is the first gate; the policy system is the second gate.
3. Setting this to `true` does not grant blanket file access. It enables the possibility of file access subject to policy.

### Interaction with Policy System

```
File access decision:
  1. Check ai.allow_file_access -> if false, deny
  2. Check cortex-policy for file access grant -> if no grant, deny
  3. Check cortex-policy for scope (which files) -> allow only within scope
  4. Allow access
```

### UI Behavior

- Rendered as a toggle switch in the Settings app.
- Label: "Allow AI to Access Files"
- Warning icon and text: "Enabling this allows AI features to read file contents as context. Individual file access still requires your permission."
- When toggled from `true` to `false`, all existing AI file access grants remain in the policy store but are effectively disabled. They become active again if the toggle is set back to `true`.

---

## A.9. ai.allow_clipboard_access

| Property | Value |
|----------|-------|
| **Field Name** | `ai.allow_clipboard_access` |
| **Type** | `boolean` |
| **Default Value** | `false` |
| **Valid Values** | `true`, `false` |
| **Read By** | `cortex-ai` (context attachment), `cortex-policy` (permission enforcement), `cortex-settings` (settings UI), `apps/settings-app` (settings form) |
| **Takes Effect** | Immediately upon save. Affects whether the AI runtime can read clipboard contents. No restart required. |
| **Storage Location** | Settings store, namespace `ai` |
| **Settings Level** | User-level |

### Description

Controls whether the AI runtime is permitted to read the system clipboard contents as context for AI requests. When `false`, the AI runtime must not access the clipboard under any circumstances. When `true`, the AI runtime may read clipboard contents when explicitly requested by an AI action, subject to the policy system.

### Validation Rules

1. Must be a boolean value.
2. This is a global toggle. Even when `true`, clipboard access may be further restricted by the policy system.
3. Clipboard access is always one-time: the AI runtime reads the clipboard once for a specific request and does not cache the contents beyond the duration of that request.

### UI Behavior

- Rendered as a toggle switch in the Settings app.
- Label: "Allow AI to Access Clipboard"
- Warning icon and text: "Enabling this allows AI features to read your clipboard contents when explicitly requested."
- When toggled from `true` to `false`, any pending clipboard read requests are cancelled.

---

## A.10. ai.budget_policy

| Property | Value |
|----------|-------|
| **Field Name** | `ai.budget_policy` |
| **Type** | `object` |
| **Default Value** | `{ "enabled": false, "daily_limit_cents": 0, "warn_threshold_pct": 80 }` |
| **Read By** | `cortex-ai` (budget enforcement), `cortex-settings` (settings UI), `apps/settings-app` (settings form), `cortex-observability` (budget metrics) |
| **Takes Effect** | Immediately upon save. Budget tracking starts or stops immediately. No restart required. |
| **Storage Location** | Settings store, namespace `ai` |
| **Settings Level** | User-level |

### Description

Controls spending limits for AI API usage. When enabled, the AI runtime tracks cumulative spending per day and enforces limits. Budget tracking applies only to external providers (OpenAI, Anthropic, Google). Local providers (Ollama, Custom pointing to localhost) are excluded from budget tracking.

### Object Structure

```
ai.budget_policy: {
  enabled:              boolean,   // whether budget tracking is active
  daily_limit_cents:    integer,   // maximum daily spend in US cents
  warn_threshold_pct:   integer    // percentage (0-100) at which to warn
}
```

### Field Details

| Sub-Field | Type | Default | Valid Range | Description |
|-----------|------|---------|-------------|-------------|
| `enabled` | boolean | `false` | `true`, `false` | Master switch for budget tracking |
| `daily_limit_cents` | integer | `0` | `0` to `100000` (0 to $1,000.00 USD) | Maximum daily spend in US cents. `0` means unlimited when `enabled` is `true`. |
| `warn_threshold_pct` | integer | `80` | `0` to `100` | Percentage of daily limit at which a warning notification is shown. `0` disables the warning. |

### Budget Tracking Behavior

1. Spending is tracked per calendar day (UTC midnight to UTC midnight).
2. Spending is estimated based on token counts and per-model pricing from the provider registry (spec 06).
3. Estimated spending is recorded in the observability system.
4. The spending counter resets at UTC midnight.
5. On first request of a new day, the counter starts at zero.

### Budget Enforcement Behavior

1. Before each AI request, `cortex-ai` checks current daily spending against `daily_limit_cents`.
2. If `enabled` is `false`, no budget check is performed.
3. If `enabled` is `true` and `daily_limit_cents` is `0`, no limit is enforced but spending is still tracked.
4. If current spending >= `daily_limit_cents`, the request is rejected with a `BudgetExceeded` error. The user is shown a notification explaining the limit has been reached.
5. If current spending >= `warn_threshold_pct` of `daily_limit_cents`, a warning notification is shown. The request is still processed.
6. Budget errors are not eligible for fallback chain retry.

### Validation Rules

1. `enabled` must be a boolean.
2. `daily_limit_cents` must be a non-negative integer between 0 and 100000 inclusive.
3. `warn_threshold_pct` must be an integer between 0 and 100 inclusive.
4. If `enabled` is `true` and `daily_limit_cents` is greater than `0`, then `warn_threshold_pct` should be greater than `0`. The settings UI shows a hint if this is not the case, but does not block saving.
5. All three sub-fields must be present in the object. Partial objects are rejected by the settings validation layer.

### UI Behavior

- Rendered as a section in the Settings app with three controls:
  - Toggle: "Enable Budget Tracking" (maps to `enabled`)
  - Number input: "Daily Spending Limit (USD)" (maps to `daily_limit_cents`, displayed as dollars with two decimal places, stored as cents)
  - Slider: "Warning Threshold" (maps to `warn_threshold_pct`, displayed as percentage)
- When budget tracking is enabled, the Settings app shows current daily spending.
- The settings app displays spending as estimated; actual charges depend on the provider.

---

## A.11. ai.show_model_disclosure

| Property | Value |
|----------|-------|
| **Field Name** | `ai.show_model_disclosure` |
| **Type** | `boolean` |
| **Default Value** | `true` |
| **Valid Values** | `true`, `false` |
| **Read By** | `cortex-ai` (response metadata), `apps/desktop-shell` (assistant panel rendering), `cortex-settings` (settings UI), `apps/settings-app` (settings form) |
| **Takes Effect** | Immediately upon save. Affects whether model disclosure is shown on the next AI response. No restart required. |
| **Storage Location** | Settings store, namespace `ai` |
| **Settings Level** | User-level |

### Description

Controls whether the AI assistant panel displays the provider and model name alongside AI responses. When `true`, each AI response includes a disclosure line showing the provider name and model name (e.g., "Response by OpenAI / gpt-4"). When `false`, no provider or model information is shown in the UI.

### Validation Rules

1. Must be a boolean value.
2. This setting controls only the user-visible disclosure. The provider and model are always included in the internal audit trail (spec 20) regardless of this setting.
3. This setting does not affect logging or observability data. Provider and model information is always logged.

### UI Behavior

- Rendered as a toggle switch in the Settings app.
- Label: "Show AI Model Disclosure"
- Help text: "When enabled, AI responses will show which provider and model generated them."
- Default is ON (`true`) to ensure transparency by default.

---

## A.12. ai.timeout_connect_ms

| Property | Value |
|----------|-------|
| **Field Name** | `ai.timeout_connect_ms` |
| **Type** | `integer` |
| **Default Value** | `10000` |
| **Valid Range** | `1000` to `60000` milliseconds |
| **Read By** | `cortex-ai` |
| **Takes Effect** | Immediately on the next request |
| **Storage Location** | Settings store, namespace `ai` |
| **Settings Level** | User-level |

### Description

Maximum time allowed to establish a TCP/TLS/HTTP connection to an AI provider before the request fails with a timeout.

### Validation Rules

1. Must be an integer in the inclusive range `1000..60000`.
2. Values lower than 1000ms are rejected to avoid accidental unusable configuration.
3. Values higher than 60000ms are rejected to prevent hung outbound connections.

---

## A.13. ai.timeout_first_token_ms

| Property | Value |
|----------|-------|
| **Field Name** | `ai.timeout_first_token_ms` |
| **Type** | `integer` |
| **Default Value** | `30000` |
| **Valid Range** | `1000` to `120000` milliseconds |
| **Read By** | `cortex-ai`, `@cortexos/ai-client` |
| **Takes Effect** | Immediately on the next streaming request |
| **Storage Location** | Settings store, namespace `ai` |
| **Settings Level** | User-level |

### Description

Maximum time to wait between dispatching a streaming request and receiving the first token or structured chunk from the provider.

### Validation Rules

1. Must be an integer in the inclusive range `1000..120000`.
2. Applies only to streaming responses. Non-streaming requests use `ai.timeout_total_ms`.

---

## A.14. ai.timeout_total_ms

| Property | Value |
|----------|-------|
| **Field Name** | `ai.timeout_total_ms` |
| **Type** | `integer` |
| **Default Value** | `120000` |
| **Valid Range** | `1000` to `600000` milliseconds |
| **Read By** | `cortex-ai`, `@cortexos/ai-client` |
| **Takes Effect** | Immediately on the next request |
| **Storage Location** | Settings store, namespace `ai` |
| **Settings Level** | User-level |

### Description

Absolute wall-clock timeout for a single AI request, including retries within the same provider attempt.

### Validation Rules

1. Must be an integer in the inclusive range `1000..600000`.
2. The settings service must reject values lower than `ai.timeout_connect_ms`.

---

## A.15. ai.timeout_stream_idle_ms

| Property | Value |
|----------|-------|
| **Field Name** | `ai.timeout_stream_idle_ms` |
| **Type** | `integer` |
| **Default Value** | `60000` |
| **Valid Range** | `1000` to `300000` milliseconds |
| **Read By** | `cortex-ai`, `@cortexos/ai-client` |
| **Takes Effect** | Immediately on the next streaming request |
| **Storage Location** | Settings store, namespace `ai` |
| **Settings Level** | User-level |

### Description

Maximum idle gap allowed between streamed tokens/chunks before a live response is treated as stalled and cancelled.

### Validation Rules

1. Must be an integer in the inclusive range `1000..300000`.
2. The settings service must reject values greater than `ai.timeout_total_ms`.

---

## A.16. ai.audit_retention_days

| Property | Value |
|----------|-------|
| **Field Name** | `ai.audit_retention_days` |
| **Type** | `integer` |
| **Default Value** | `90` |
| **Valid Range** | `1` to `3650` days |
| **Read By** | `cortex-observability`, `cortex-admin`, `cortex-policy` (AI action audit retention enforcement) |
| **Takes Effect** | On the next retention sweep |
| **Storage Location** | Settings store, namespace `ai` |
| **Settings Level** | User-level |

### Description

Retention period for AI audit records. After this window, records may be deleted by a background retention job unless a legal hold or explicit export retention policy overrides deletion.

### Validation Rules

1. Must be an integer in the inclusive range `1..3650`.
2. Lower values are allowed but the settings UI must show a destructive-data-retention warning when set below 30 days.

---

## A.17. ai.auto_apply_low_risk_actions

| Property | Value |
|----------|-------|
| **Field Name** | `ai.auto_apply_low_risk_actions` |
| **Type** | `boolean` |
| **Default Value** | `false` |
| **Valid Values** | `true`, `false` |
| **Read By** | `@cortexos/ai-client`, `cortex-policy` (AI action confirmation layer), `apps/settings-app` |
| **Takes Effect** | Immediately on the next eligible AI action |
| **Storage Location** | Settings store, namespace `ai` |
| **Settings Level** | User-level |

### Description

Controls whether CortexOS may skip a secondary confirmation dialog for AI actions already classified as Low risk. This setting never bypasses permission prompts and never applies to Medium or High risk AI actions.

### Validation Rules

1. Must be a boolean value.
2. When enabled, only actions classified as `Low` by spec 20 may be auto-applied.
3. This setting must not suppress audit logging, provider/model disclosure metadata, or permission grant checks.

---

## Cross-Field Validation Rules

The following rules apply across multiple fields:

### R.1. Fallback Chain Consistency

When `ai.fallback_enabled` is `true` and `ai.fallback_chain` is non-empty, each provider in the fallback chain should ideally have credentials configured. The settings service does not enforce this at save time but the AI runtime logs a warning at request time if a provider in the chain has no credentials.

### R.2. Override Provider Consistency

When `ai.per_app_overrides` or `ai.per_feature_overrides` references a provider, that provider should ideally have credentials configured. Not enforced at save time.

### R.3. Privacy Mode and Local Providers

When `ai.preferred_provider` is `"Ollama"` or `"Custom"` pointing to `localhost`/`127.0.0.1`, the `ai.privacy_mode` setting has no effect (data does not leave the machine). The settings UI may show an informational note about this but does not disable the privacy_mode control.

### R.4. Budget Policy and Local Providers

When `ai.preferred_provider` is `"Ollama"` or `"Custom"` pointing to `localhost`/`127.0.0.1`, the `ai.budget_policy` tracking has no effect (no cost incurred). Budget tracking applies only to external providers.

### R.5. Null Preferred Provider

When `ai.preferred_provider` is `null`:
- `ai.preferred_model` has no effect.
- `ai.fallback_enabled` and `ai.fallback_chain` have no effect.
- `ai.per_app_overrides` entries still take effect for their respective apps.
- `ai.per_feature_overrides` entries still take effect for their respective features.
- The global AI assistant is disabled and shows a configuration prompt.

### R.6. Timeout Ordering

The following ordering must always hold:

- `ai.timeout_connect_ms <= ai.timeout_first_token_ms`
- `ai.timeout_connect_ms <= ai.timeout_total_ms`
- `ai.timeout_stream_idle_ms <= ai.timeout_total_ms`

If a settings mutation would violate this ordering, the settings service rejects it with a validation error.

## Settings Resolution Algorithm

The AI runtime resolves the effective provider and model for each request using this algorithm:

```
function resolveProviderModel(app_id, feature_name):
  // Step 1: Check per-feature override
  if feature_name is not null and ai.per_feature_overrides[feature_name] exists:
    override = ai.per_feature_overrides[feature_name]
    return (override.provider, override.model or "")

  // Step 2: Check per-app override
  if app_id is not null and ai.per_app_overrides[app_id] exists:
    override = ai.per_app_overrides[app_id]
    return (override.provider, override.model or "")

  // Step 3: Use global settings
  if ai.preferred_provider is not null:
    return (ai.preferred_provider, ai.preferred_model or "")

  // Step 4: No provider configured
  return (null, null)  // AI request will fail with NoProviderConfigured
```

## Settings Serialization Format

All AI settings are serialized as JSON in the settings store:

```json
{
  "ai": {
    "preferred_provider": null,
    "preferred_model": "",
    "fallback_enabled": false,
    "fallback_chain": [],
    "per_app_overrides": {},
    "per_feature_overrides": {},
    "privacy_mode": "none",
    "allow_file_access": false,
    "allow_clipboard_access": false,
    "budget_policy": {
      "enabled": false,
      "daily_limit_cents": 0,
      "warn_threshold_pct": 80
    },
    "show_model_disclosure": true,
    "timeout_connect_ms": 10000,
    "timeout_first_token_ms": 30000,
    "timeout_total_ms": 120000,
    "timeout_stream_idle_ms": 60000,
    "audit_retention_days": 90,
    "auto_apply_low_risk_actions": false
  }
}
```

This is the canonical serialization format. All consumers must handle this exact structure. Fields must not be renamed or restructured without a corresponding settings migration (spec 05).
