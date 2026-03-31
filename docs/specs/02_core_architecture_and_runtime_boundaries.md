# 02 — Core Architecture and Runtime Boundaries

## 1. Purpose

Define the fundamental architectural boundaries of CortexOS: the trust model, communication patterns, error taxonomy, and the contract between trusted (server) and untrusted (client) layers. Every other subsystem spec inherits the rules established here.

## 2. Scope

- Trusted vs. untrusted layer definitions and responsibilities
- Source-of-truth mapping for every data domain
- Client-server communication patterns and protocols
- Degraded-mode and restore behavior
- Canonical error taxonomy, error codes, and error response format
- Boundary rules governing what may cross the client-server boundary

## 3. Out of Scope

- Individual subsystem internals (owned by child specs 03–22)
- Frontend rendering architecture (owned by spec 07 Desktop Shell)
- AI-specific provider logic (owned by spec 06)
- App sandboxing implementation (owned by spec 09)

## 4. Objectives

1. Every developer (human or coding agent) can determine which side owns a piece of logic by reading this spec alone.
2. No ambiguity about where state lives, where validation happens, or where authorization is enforced.
3. Error responses are uniform across every API endpoint so clients handle them consistently.
4. The system degrades gracefully: losing the AI provider never crashes the OS; losing the backend leaves the user informed.

## 5. User-Visible Behavior

| Scenario | User-Visible Outcome |
|---|---|
| Backend unreachable | Offline banner in shell, queued mutations shown with pending indicator |
| AI provider down | AI surfaces show "AI unavailable" with link to Settings; OS otherwise fully functional |
| Session expires | Redirect to login, unsaved work preserved in local draft storage |
| Permission denied | Operation blocked with toast explaining which permission is needed |
| Rate limited | "Slow down" toast, auto-retry indicator for transient operations |

## 6. System Behavior

### 6.1 Trust Model

- **Server (Rust)**: Trusted. Source of truth for all state. Enforces all authorization, validation, and business rules.
- **Client (TypeScript / browser)**: Untrusted. Renders UI, captures user input, forwards actions to server. Never makes authorization decisions. Never stores secrets. Never trusts its own state as canonical.

**Invariant INV-02-1**: Every mutation of server-owned state MUST be validated server-side, regardless of client-side validation.

**Invariant INV-02-2**: The client MUST treat all locally cached state as potentially stale and reconcile on reconnection.

### 6.2 Source-of-Truth Mapping

| Data Domain | Source of Truth | Owning Crate | Storage |
|---|---|---|---|
| User identity & sessions | Server | cortex-auth | SQLite |
| Settings & preferences | Server | cortex-settings | SQLite |
| Virtual filesystem metadata | Server | cortex-files | SQLite |
| File content blobs | Server | cortex-files | Content-addressable store |
| Permissions & grants | Server | cortex-policy | SQLite |
| AI configuration & routing | Server | cortex-ai | SQLite + in-memory cache |
| App lifecycle state | Server | cortex-runtime | SQLite + in-memory |
| Window positions (session) | Server | cortex-api | SQLite (per session) |
| Notification queue | Server | cortex-notify | SQLite |
| Search index | Server | cortex-search | SQLite FTS5 |
| Logs & metrics | Server | cortex-observability | SQLite + stdout |
| UI render state | Client only | — | Browser memory |

### 6.3 Startup Sequence

1. Backend starts: load config → init DB → start HTTP/WS server → register services
2. Client loads: fetch `/api/v1/health` → authenticate → open WebSocket → request initial state snapshot
3. Desktop shell renders: taskbar, desktop, default app set
4. Previously running apps restored (if session recovery enabled)

### 6.4 Communication Protocols

| Protocol | Purpose | Pattern |
|---|---|---|
| REST (`/api/v1/...`) | CRUD operations | Request → Response (JSON) |
| WebSocket (`/ws`) | Real-time events | Bidirectional, server-push |
| SSE (internal) | AI streaming | Server → Client, token-by-token |

### 6.5 Degraded Mode Rules

**Backend unreachable (INV-02-3)**:
- Client shows persistent offline banner
- User-initiated mutations queued in IndexedDB with pending status
- On reconnection: replay queued mutations, reconcile state
- Read-only operations show last-known cached state with "offline" badge

**AI provider unavailable (INV-02-4)**:
- AI surfaces render with "AI currently unavailable" message
- Non-AI OS features fully operational
- Retry logic per spec 06 (fallback chain)
- No crash, no hang, no infinite spinner

### 6.6 Restore Behavior

On WebSocket reconnect:
1. Client sends `reconnect` message with last-known `session_id`
2. Server validates session, sends state delta since last event
3. Client reconciles: apply delta, replay pending mutations
4. Resume suspended apps per spec 09

On OS restart (server crash recovery):
1. Server reads last-known session from DB
2. Restores settings, filesystem state, permissions
3. Attempts to resume previously running apps (per spec 09 crash recovery)

## 7. Architecture

```
┌─────────────────────────────────────────┐
│           Browser (Untrusted)           │
│  ┌───────────┐  ┌───────────┐          │
│  │Desktop    │  │ Apps      │          │
│  │Shell      │  │ (sandboxed)│          │
│  └─────┬─────┘  └─────┬─────┘          │
│        └───────┬───────┘               │
│            API Client                   │
│        (typed fetch + WS)              │
└────────────────┬────────────────────────┘
                 │ HTTPS + WSS
┌────────────────┴────────────────────────┐
│           Rust Server (Trusted)         │
│  ┌──────────┐                           │
│  │cortex-api│ (HTTP/WS gateway)         │
│  └──┬───────┘                           │
│     │  ┌──────────┐ ┌────────┐ ┌──────┐│
│     └──│cortex-   │ │cortex- │ │cortex││
│        │auth      │ │policy  │ │config││
│        └──────────┘ └────────┘ └──────┘│
│     ┌──────────┐ ┌────────┐ ┌────────┐ │
│     │cortex-   │ │cortex- │ │cortex- │ │
│     │settings  │ │files   │ │ai      │ │
│     └──────────┘ └────────┘ └────────┘ │
│     ┌──────────┐ ┌────────┐ ┌────────┐ │
│     │cortex-   │ │cortex- │ │cortex- │ │
│     │runtime   │ │notify  │ │search  │ │
│     └──────────┘ └────────┘ └────────┘ │
│              ┌──────────┐               │
│              │cortex-db │ (SQLite)      │
│              └──────────┘               │
└─────────────────────────────────────────┘
```

## 8. Data Model

### 8.1 Error Response Format (Universal)

All API errors return this structure:

```typescript
interface CortexError {
  error: {
    code: string;        // e.g. "PERMISSION_DENIED"
    category: ErrorCategory;
    message: string;     // Human-readable, safe to display
    details: Record<string, unknown>;  // Machine-readable context
    retryable: boolean;
    retry_after_ms?: number;  // Present for RateLimit
  };
}

type ErrorCategory =
  | "Transient"
  | "Permanent"
  | "Auth"
  | "Policy"
  | "NotFound"
  | "Conflict"
  | "RateLimit"
  | "QuotaExceeded"
  | "ProviderUnavailable"
  | "Timeout"
  | "Validation";
```

### 8.2 Error Taxonomy

| Code | Category | HTTP Status | Retryable | Description |
|---|---|---|---|---|
| TRANS_001 | Transient | 503 | Yes | Temporary backend unavailability |
| PERM_001 | Permanent | 500 | No | Unexpected server error |
| AUTH_001 | Auth | 401 | No | Not authenticated |
| AUTH_002 | Auth | 403 | No | Session expired or invalid |
| POL_001 | Policy | 403 | No | Permission denied |
| POL_002 | Policy | 403 | No | Policy violation |
| NF_001 | NotFound | 404 | No | Resource not found |
| NF_002 | NotFound | 404 | No | App not installed |
| CONF_001 | Conflict | 409 | No | Resource version conflict |
| CONF_002 | Conflict | 409 | No | Duplicate resource |
| RL_001 | RateLimit | 429 | Yes | Rate limit exceeded |
| QE_001 | QuotaExceeded | 429 | No | Storage quota exceeded |
| QE_002 | QuotaExceeded | 429 | No | AI budget exceeded |
| PU_001 | ProviderUnavailable | 502 | Yes | AI provider unreachable |
| PU_002 | ProviderUnavailable | 502 | Yes | All AI providers failed |
| TM_001 | Timeout | 504 | Yes | Request timeout |
| TM_002 | Timeout | 504 | Yes | AI response timeout |
| VAL_001 | Validation | 400 | No | Invalid request body |
| VAL_002 | Validation | 400 | No | Invalid parameter |
| VAL_003 | Validation | 422 | No | Schema validation failed |

### 8.3 API Request Envelope

```typescript
// Standard request headers
interface RequestHeaders {
  "Authorization": `Bearer ${string}`;  // Session token
  "X-Request-ID": string;               // UUID for tracing
  "X-Correlation-ID"?: string;          // Distributed tracing
  "Content-Type": "application/json";
}

// Standard success response
interface SuccessResponse<T> {
  data: T;
  meta?: {
    request_id: string;
    timestamp: string;  // ISO 8601
  };
}
```

### 8.4 WebSocket Message Format

```typescript
// Client → Server
interface WSClientMessage {
  type: "command" | "subscribe" | "unsubscribe" | "ping";
  id: string;           // Client-generated message ID
  channel?: string;     // For subscribe/unsubscribe
  payload?: unknown;    // Typed per command
}

// Server → Client
interface WSServerMessage {
  type: "event" | "command_response" | "error" | "pong";
  id: string;           // Echoes client message ID, or server-generated for events
  channel?: string;     // Event channel
  payload?: unknown;    // Typed per event
  error?: CortexError;  // Present for error type
}
```

## 9. Public Interfaces

### 9.1 REST API Conventions

- Base path: `/api/v1/`
- All endpoints return `CortexError` on failure
- Pagination: `?limit=N&cursor=base64cursor`
- Filtering: `?filter[field]=value`
- Sorting: `?sort=field&order=asc|desc`

### 9.2 WebSocket Channels

| Channel | Direction | Purpose |
|---|---|---|
| `system` | Server→Client | System-wide events (startup, shutdown, config changes) |
| `apps` | Server→Client | App lifecycle events |
| `files` | Server→Client | Filesystem change events |
| `ai` | Server→Client | AI request lifecycle events |
| `notifications` | Server→Client | Notification delivery |
| `settings` | Server→Client | Setting change notifications |

## 10. Internal Interfaces

### 10.1 Inter-Crate Communication Rules

- Crates communicate via Rust trait interfaces (dependency injection)
- No crate may directly access another crate's database tables
- All cross-crate calls go through typed trait methods
- `cortex-core` defines shared types; all crates depend on it
- `cortex-api` depends on all service crates; service crates never depend on `cortex-api`

### 10.2 Dependency Direction (acyclic)

```
cortex-core ← (all crates depend on this)
cortex-config ← cortex-db
cortex-db ← cortex-auth, cortex-policy, cortex-settings, cortex-files, cortex-ai
cortex-auth ← cortex-policy
cortex-policy ← cortex-settings (reads only)
cortex-settings ← cortex-db
cortex-files ← cortex-policy, cortex-db
cortex-ai ← cortex-policy, cortex-settings, cortex-db
cortex-runtime ← cortex-policy, cortex-files
cortex-notify ← cortex-runtime
cortex-search ← cortex-files, cortex-db
cortex-observability ← cortex-core (only)
cortex-api ← all service crates
```

## 11. State Management

### 11.1 Server State (Authoritative)

- All domain state stored in SQLite via cortex-db
- Migrations are numbered and forward-only
- State changes emit events through the command bus
- Optimistic concurrency via version counters on mutable entities

### 11.2 Client State (Cache)

- Client caches server state locally (IndexedDB for persistence, memory for active)
- Cache is always treated as stale; validated against server on reconnect
- No client-side state is ever treated as source of truth for security decisions

### 11.3 Session State

- Sessions identified by cryptographically random token
- Session stored server-side in SQLite
- Token transmitted via `Authorization: Bearer <token>` header
- Session expiry: 24 hours default, configurable
- Refresh: sliding window on activity

## 12. Failure Modes and Error Handling

### 12.1 Failure Categories

| Failure Type | Example | Recovery Strategy |
|---|---|---|
| Transient backend | DB locked, temp network | Auto-retry with backoff (3 attempts, exponential) |
| Auth failure | Expired token | Redirect to login, preserve draft state |
| Policy violation | Missing permission | Show error with permission name, offer to request |
| Conflict | Version mismatch | Show conflict, offer user resolution |
| Rate limit | Too many requests | Back off, show countdown, auto-retry after `retry_after_ms` |
| Provider unavailable | AI provider down | Graceful degradation, fallback chain (spec 06) |
| Timeout | Slow operation | Show timeout message, offer retry |
| Validation | Bad input | Show field-level errors, no retry |
| Quota exceeded | Storage full | Show quota message, offer cleanup options |

### 12.2 Circuit Breaker Rules

- If a service returns 5 errors in 30 seconds, circuit opens for 60 seconds
- During open circuit: return cached/stale data where possible, otherwise TRANS_001
- Half-open: allow 1 request; if it succeeds, close circuit

## 13. Security and Permissions

**SEC-02-1**: Client-side JavaScript MUST NEVER contain API keys, passwords, or session secrets beyond the current session token.

**SEC-02-2**: All API endpoints MUST validate authentication before processing, except `/api/v1/health` and `/api/v1/auth/login`.

**SEC-02-3**: WebSocket connections MUST authenticate on connect; unauthenticated connections are rejected.

**SEC-02-4**: All server-to-server communication within the Rust process is trusted; the trust boundary is the HTTP/WS interface.

**SEC-02-5**: Input validation occurs at the API boundary; internal crate calls assume validated inputs.

**SEC-02-6**: No user-supplied string is ever interpolated into SQL queries (parameterized queries only).

## 14. Performance Requirements

| Metric | Target | Measurement |
|---|---|---|
| API response time (p50) | < 50ms | Server-side histogram |
| API response time (p99) | < 500ms | Server-side histogram |
| WebSocket message delivery | < 100ms | Client-side measurement |
| Initial page load (TTI) | < 2s | Lighthouse |
| Reconnect + rehydrate | < 1s | Client-side measurement |
| Error response overhead | < 5ms added latency | Server-side |

## 15. Accessibility Requirements

- All error messages MUST be human-readable and screen-reader accessible
- Error toasts MUST be announced via ARIA live regions
- Offline/online status changes MUST be announced to assistive technology
- Loading states MUST use `aria-busy="true"` and announce completion

## 16. Observability and Logging

- Every API request logged with: method, path, status, latency_ms, request_id, user_id (if authed)
- Every WebSocket connection logged: connect, disconnect, subscribe, unsubscribe
- Error responses logged at WARN level with full CortexError structure
- Circuit breaker state changes logged at INFO level
- All logs include `trace_id` for distributed tracing correlation

## 17. Testing Requirements

| Test Type | Coverage Target | Key Tests |
|---|---|---|
| Unit | Every error code path | Error construction, categorization, retryability |
| Integration | Every API endpoint | Happy path + every error code per endpoint |
| E2E | Degraded mode | Backend disconnect, AI provider down, session expiry |
| Contract | Error format | Every error response matches CortexError schema |
| Chaos | Network failures | Random disconnects, latency injection, partial responses |

## 18. Acceptance Criteria

- [ ] Every API endpoint returns errors in the exact CortexError format
- [ ] Client correctly handles all 20 error codes with appropriate UX
- [ ] Backend disconnect shows offline banner within 2 seconds
- [ ] Reconnection restores state without data loss
- [ ] AI provider failure shows graceful message, OS otherwise functional
- [ ] No client-side code contains authorization logic
- [ ] No circular crate dependencies exist
- [ ] All cross-crate interfaces are typed Rust traits
- [ ] Circuit breaker activates after 5 failures in 30 seconds
- [ ] All logs include trace_id

## 19. Build Order and Dependencies

**Layer 1** (this spec). Depends on:
- 01 — Repository, Toolchain, Engineering Conventions (for crate structure)

Blocks:
- All specs 03–23 depend on the error taxonomy, trust model, and boundary rules defined here

## 20. Non-Goals and Anti-Patterns

**Non-Goals**:
- Multi-process deployment (v1 is single-process)
- Horizontal scaling (single-server, single-user)
- Real-time collaboration between multiple users
- Offline-first with conflict-free replicated data types

**Anti-Patterns**:
- NEVER trust client-side state as authoritative for security decisions
- NEVER bypass the error taxonomy with raw HTTP status codes or untyped error messages
- NEVER create circular crate dependencies
- NEVER access another crate's database tables directly
- NEVER log session tokens, passwords, or API keys
- NEVER use string interpolation for SQL queries

## 21. Implementation Instructions for Claude Code / Codex

1. Implement `cortex-core` first: define `CortexError`, `ErrorCategory`, all error codes as an enum, and the `SuccessResponse<T>` / `CortexError` response types.
2. Implement `cortex-db` next: SQLite connection pool, migration runner, transaction helpers.
3. Implement `cortex-api` error middleware: converts domain errors into `CortexError` responses automatically.
4. Write integration tests: for each error code, verify the HTTP response matches the schema.
5. Write the WebSocket message types as shared TypeScript interfaces (auto-generated from Rust types via `ts-rs` or equivalent).
6. Implement the circuit breaker as a reusable middleware in `cortex-api`.
7. Verify no circular dependencies: `cargo check` must succeed with the full workspace.
