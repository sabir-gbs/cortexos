# 10 — System Command Bus and Event Model

## 1. Purpose
Define the typed, asynchronous communication layer that all services and apps use to interact. Every inter-service and inter-app message flows through this bus with typed contracts.

## 2. Scope
- Synchronous commands (request/response with timeout)
- Asynchronous events (publish/subscribe)
- Streaming events
- Typed command and event contracts
- Event categories and delivery guarantees
- Rate limiting and dead letter handling

## 3. Out of Scope
- Specific command/event payloads (owned by emitting subsystem specs)
- Transport implementation (WebSocket details owned by spec 02)
- Permission checks on commands (owned by spec 04, enforced before dispatch)

## 4. Objectives
1. Zero untyped payloads — every command and event has a typed schema.
2. Apps communicate ONLY through the bus — no direct IPC.
3. Delivery guarantees are explicit: at-least-once for events, effectively exactly-once for commands via durable idempotency records.
4. Rate limiting prevents any single app from overwhelming the bus.

## 5. User-Visible Behavior
| Scenario | Outcome |
|---|---|
| App sends command | Response received within timeout, or timeout error |
| Service emits event | All subscribers notified |
| App exceeds rate limit | Commands rejected with RL_001 |
| Subscriber crashes | Events queued in dead letter, retried on reconnect |

## 6. System Behavior

### 6.1 Command Types
- **Synchronous Command**: Request → Response (with timeout). Caller blocks until response or timeout.
- **Event**: Fire-and-forget notification to all subscribers. Publisher doesn't wait.
- **Streaming Event**: Continuous stream of events on a channel (e.g., AI tokens).

### 6.2 Event Categories
```
SystemEvent    → startup, shutdown, config_changed
AppEvent       → app.started, app.stopped, app.crashed, app.focused
UIEvent        → theme.changed, layout.changed, locale.changed
FileEvent      → file.created, file.modified, file.deleted, file.moved
AIEvent        → ai.request_started, ai.request_completed, ai.provider_changed
PermissionEvent → permission.granted, permission.revoked, permission.denied
SettingEvent   → settings.changed
WmEvent        → window.created, window.updated, window.closed, window.focused, wm.workspace.changed
NotifyEvent    → notification.created, notification.dismissed, notification.read, notification.all_read, notification.expired
```

### 6.3 Delivery Guarantees
- **Commands**: Effectively exactly-once via durable idempotency keys. Every command has a unique `command_id`. Duplicate IDs return the previously persisted response while the idempotency record remains within retention.
- **Events**: At-least-once. Subscribers must be idempotent. Events include sequence numbers for ordering.

### 6.4 Rate Limiting
- Per-app command rate: 100 commands/minute default, configurable
- Per-service event rate: no hard limit (trusted services)
- Burst: allows 2x rate for 10 seconds, then enforces

### 6.5 Dead Letter Queue
- Failed deliveries (subscriber disconnected or errored) go to dead letter queue
- Dead letter entries have: event_type, payload, target_subscriber, failure_reason, timestamp
- Retried on subscriber reconnect (up to 3 times)
- After 3 failures: permanently dead, logged at WARN

## 7. Architecture
```
┌─────────────────────────────────────────┐
│           Command Bus (Rust)            │
│                                         │
│  ┌──────────────────────────────────┐   │
│  │     Command Router               │   │
│  │  (dispatch to registered         │   │
│  │   handlers by command type)      │   │
│  └──────────────────────────────────┘   │
│                                         │
│  ┌──────────────────────────────────┐   │
│  │     Event Dispatcher             │   │
│  │  (fan-out to subscribers)        │   │
│  └──────────────────────────────────┘   │
│                                         │
│  ┌──────────────────────────────────┐   │
│  │     Rate Limiter                 │   │
│  │  (token bucket per app)          │   │
│  └──────────────────────────────────┘   │
│                                         │
│  ┌──────────────────────────────────┐   │
│  │     Dead Letter Queue            │   │
│  │  (failed deliveries)             │   │
│  └──────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

## 8. Data Model

```rust
struct Command<T: Serialize, R: Serialize> {
    command_type: String,          // e.g. "file.read"
    command_id: String,            // UUID (idempotency key)
    app_id: Option<String>,        // Calling app (None for services)
    timestamp: chrono::DateTime<chrono::Utc>,
    payload: T,
    timeout_ms: u64,               // Default: 5000
    _phantom: std::marker::PhantomData<R>,
}

struct Event<T: Serialize> {
    event_type: String,            // e.g. "file.created"
    event_id: String,              // UUID
    source: String,                // Service or app that emitted
    timestamp: chrono::DateTime<chrono::Utc>,
    sequence: u64,                 // Monotonic per event_type
    payload: T,
}

struct Subscription {
    subscriber_id: String,
    event_pattern: String,         // Glob: "file.*" or exact: "app.started"
    created_at: chrono::DateTime<chrono::Utc>,
}

struct DeadLetterEntry {
    id: String,
    event_type: String,
    schema_version: u32,
    payload_json: String,         // Serialized form of a previously validated typed event
    target_subscriber: String,
    failure_reason: String,
    retry_count: u8,
    created_at: chrono::DateTime<chrono::Utc>,
}
```

## 9. Public Interfaces

### Rust Trait API
```rust
trait BusCommand: Serialize + for<'de> Deserialize<'de> + Send + Sync {
    const COMMAND_TYPE: &'static str;
    type Response: Serialize + for<'de> Deserialize<'de> + Send + Sync;
}

trait BusEvent: Serialize + for<'de> Deserialize<'de> + Send + Sync {
    const EVENT_TYPE: &'static str;
}

trait CommandBus: Send + Sync {
    fn register_handler<C>(&mut self, handler: Box<dyn Fn(Command<C, C::Response>) -> Result<C::Response, CortexError> + Send + Sync>) -> Result<()>
    where
        C: BusCommand + 'static;

    fn send_command<C>(&self, command: Command<C, C::Response>) -> Result<C::Response, CortexError>
    where
        C: BusCommand + 'static;

    fn subscribe<E>(&mut self, pattern: &str, subscriber_id: &str, handler: Box<dyn Fn(E) + Send + Sync>) -> Result<()>
    where
        E: BusEvent + 'static;

    fn unsubscribe(&mut self, pattern: &str, subscriber_id: &str) -> Result<()>;

    fn publish<E>(&self, event: Event<E>) -> Result<()>
    where
        E: BusEvent + 'static;
}
```

### TypeScript Client API
```typescript
interface CommandBusClient {
  sendCommand<T, R>(type: string, payload: T, timeoutMs?: number): Promise<R>;
  subscribe<E>(pattern: string, handler: (event: Event<E>) => void): () => void;  // returns unsubscribe fn
}
```

## 10. Internal Interfaces
- cortex-api: bridges WebSocket messages to/from command bus
- All service crates: register command handlers at startup
- cortex-runtime: subscribes to app events
- cortex-policy: intercepts commands for permission checks before dispatch

## 11. State Management
- Command handlers registered in memory (HashMap<command_type, handler>)
- Subscriptions in memory (HashMap<pattern, Vec<subscriber>>)
- Dead letter entries in SQLite (persistent across restarts)
- Idempotency records in SQLite (persistent across restarts, minimum retention 24 hours)
- Optional in-memory LRU mirror for hot responses (max 10000 entries, TTL 5 minutes) backed by the durable idempotency store

## 12. Failure Modes and Error Handling

| Failure | Handling |
|---|---|
| No handler registered | Return NF_001 "No handler for command type" |
| Handler timeout | Return TM_001, log at WARN |
| Handler error | Return error to caller, log at ERROR |
| Subscriber disconnected | Queue in dead letter, retry on reconnect |
| Rate limit exceeded | Return RL_001 with retry_after_ms |
| Bus overloaded | Apply backpressure, log at WARN |

## 13. Security and Permissions
- Apps can only send commands they have permission for (checked before dispatch)
- Events are published by trusted services only; apps can only subscribe
- Command payloads are validated against registered schema
- No command can bypass cortex-policy (INV-10-1)

## 14. Performance Requirements

| Metric | Target |
|---|---|
| Command dispatch latency (in-process) | < 1ms |
| Event fan-out latency | < 5ms |
| Dead letter write | < 10ms |
| Max concurrent commands | 1000/second |
| Max concurrent events | 10000/second |

## 15. Accessibility Requirements
- Bus errors surfaced to users must be human-readable
- No direct accessibility requirements (infrastructure layer)

## 16. Observability and Logging
- Every command logged at DEBUG: type, app_id, duration_ms, success
- Every event publish logged at TRACE: type, subscriber_count
- Rate limit hits logged at WARN
- Dead letter entries logged at WARN
- Metrics: commands_per_second, events_per_second, command_latency_ms, dead_letter_count

## 17. Testing Requirements
- Unit: command dispatch with typed request/response
- Unit: event fan-out to multiple subscribers
- Unit: rate limiting token bucket
- Unit: idempotency (duplicate command returns cached response)
- Integration: full round-trip command through API
- Integration: dead letter creation and retry

## 18. Acceptance Criteria
- [ ] All commands and events have typed schemas (verified at compile time in Rust)
- [ ] Command idempotency works (same ID returns cached response)
- [ ] Events delivered to all matching subscribers
- [ ] Rate limiting enforced per-app
- [ ] Dead letter queue captures failed deliveries
- [ ] No untyped payloads in any command or event path
- [ ] Permission check on every command before dispatch

## 19. Build Order and Dependencies
**Layer 6**. Depends on: 01, 02 (error taxonomy), 04 (permissions)

## 20. Non-Goals and Anti-Patterns
- No message persistence (commands are in-memory, events fire-and-forget)
- No message ordering guarantees across different event types
- No distributed bus (single process)
- NEVER emit untyped event payloads
- NEVER allow apps to bypass the bus for direct communication
- NEVER skip rate limiting for any app

## 21. Implementation Instructions for Claude Code / Codex
1. Define Command<T,R>, Event<T>, Subscription, DeadLetterEntry types.
2. Implement CommandBus trait with handler registration and dispatch.
3. Implement event fan-out with glob pattern matching.
4. Implement token-bucket rate limiter per app_id.
5. Implement dead letter queue with SQLite persistence.
6. Implement idempotency cache (LRU + TTL).
7. Write tests: typed dispatch, fan-out, rate limiting, idempotency, dead letter.
