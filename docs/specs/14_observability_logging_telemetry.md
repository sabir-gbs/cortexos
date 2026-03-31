# 14 — Observability, Logging, and Telemetry

## 1. Purpose
Define the structured logging, metrics, and telemetry system that provides visibility into CortexOS runtime behavior for debugging, performance monitoring, and auditing.

## 2. Scope
- Structured JSON logging
- Log levels and filtering
- Distributed tracing (correlation IDs)
- Metrics collection and exposure
- Health check endpoints
- Log retention and rotation
- Privacy controls

## 3. Out of Scope
- External log aggregation services (v1 — local only)
- Custom dashboard UI (admin diagnostics owned by spec 22)
- APM (application performance monitoring) integration
- Log-based alerting

## 4. Objectives
1. Every log entry is structured JSON with consistent fields.
2. Every request can be traced end-to-end via correlation IDs.
3. No user content (file content, AI prompts) appears in logs unless explicitly configured.
4. Log retention is bounded and configurable.

## 5. User-Visible Behavior

| Scenario | Outcome |
|---|---|
| System running normally | No visible impact, logs written to stdout |
| Error occurs | User sees error toast, log captures full context |
| User checks system health | Health endpoint returns status |
| User disables telemetry | Aggregated stats no longer collected |

## 6. System Behavior

### 6.1 Log Format
Every log entry is JSON:
```json
{
  "timestamp": "2026-03-30T12:00:00.000Z",
  "level": "INFO",
  "service": "cortex-files",
  "trace_id": "abc123",
  "span_id": "def456",
  "message": "File created",
  "fields": {
    "file_id": "...",
    "path": "/home/user/test.txt",
    "user_id": "..."
  }
}
```

### 6.2 Log Levels
| Level | Usage |
|---|---|
| TRACE | Detailed function entry/exit, variable values |
| DEBUG | Request/response details, query plans |
| INFO | Business events: file created, app launched, setting changed |
| WARN | Recoverable issues: conflict detected, rate limit hit, fallback used |
| ERROR | Unrecoverable issues: handler panic, DB write failure |
| (No FATAL) | Process-level crashes handled by OS |

### 6.3 Distributed Tracing
- Every incoming HTTP/WS request generates a `trace_id` (UUID)
- Each handler generates a `span_id` (UUID)
- `trace_id` propagated to all downstream calls
- Logs within a request share the same `trace_id`

### 6.4 Metrics
```rust
struct Metrics {
    // Counters
    requests_total: Counter,          // By method, path, status
    errors_total: Counter,            // By error_code
    ai_requests_total: Counter,       // By provider, model, status
    file_operations_total: Counter,   // By operation type

    // Histograms
    request_duration_ms: Histogram,   // By method, path
    ai_request_latency_ms: Histogram, // By provider, model

    // Gauges
    active_sessions: Gauge,
    apps_running: Gauge,
    storage_bytes_used: Gauge,
    search_index_size_bytes: Gauge,
}
```

### 6.5 Health Check
```
GET /api/v1/health → {
  "status": "healthy" | "degraded" | "unhealthy",
  "uptime_seconds": 3600,
  "version": "0.1.0",
  "checks": {
    "database": { "status": "healthy", "latency_ms": 2 },
    "ai_provider": { "status": "degraded", "provider": "OpenAI", "message": "Rate limited" },
    "filesystem": { "status": "healthy" },
    "search_index": { "status": "healthy", "document_count": 1234 }
  }
}
```
- `healthy`: all checks pass
- `degraded`: non-critical check fails (AI provider down)
- `unhealthy`: critical check fails (database down)

### 6.6 Privacy Controls
- `privacy.telemetry_enabled` setting: if false, no aggregated stats collected
- `ai.privacy_mode` applied to AI-related logs
- File content NEVER logged
- User passwords/tokens NEVER logged
- Settings values logged as hashes for sensitive keys
- API keys NEVER logged

## 7. Architecture
```
┌──────────────────────────────────────────┐
│        cortex-observability              │
│                                          │
│  ┌────────────────────────────────────┐  │
│  │  Log Layer                         │  │
│  │  (tracing-subscriber, JSON fmt,   │  │
│  │   stdout output, searchable sink) │  │
│  └────────────────────────────────────┘  │
│                                          │
│  ┌────────────────────────────────────┐  │
│  │  Metrics Layer                     │  │
│  │  (in-memory counters/gauges,       │  │
│  │   Prometheus-compatible expose)    │  │
│  └────────────────────────────────────┘  │
│                                          │
│  ┌────────────────────────────────────┐  │
│  │  Trace Layer                       │  │
│  │  (correlation ID propagation,      │  │
│  │   span tracking)                   │  │
│  └────────────────────────────────────┘  │
│                                          │
│  ┌────────────────────────────────────┐  │
│  │  Health Check Registry             │  │
│  │  (registered checks per service)   │  │
│  └────────────────────────────────────┘  │
└──────────────────────────────────────────┘
```

## 8. Data Model
```rust
struct LogEntry {
    timestamp: chrono::DateTime<chrono::Utc>,
    level: LogLevel,
    service: String,
    trace_id: Option<String>,
    span_id: Option<String>,
    parent_span_id: Option<String>,
    message: String,
    fields: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum LogLevel { Trace, Debug, Info, Warn, Error }

struct HealthCheck {
    name: String,
    check_fn: Box<dyn Fn() -> HealthStatus + Send + Sync>,
}

struct HealthStatus {
    status: ServiceHealth,
    latency_ms: Option<u64>,
    message: Option<String>,
}

enum ServiceHealth { Healthy, Degraded, Unhealthy }
```

## 9. Public Interfaces
```
GET  /api/v1/health          → Health check (no auth required)
GET  /api/v1/metrics         → Prometheus-compatible metrics (admin only)
```

## 10. Internal Interfaces
- All crates use `tracing` crate macros: `tracing::info!`, `tracing::warn!`, etc.
- `cortex-observability` provides the subscriber initialization
- Each service registers health checks at startup
- Metrics are global statics accessed via convenience macros

## 11. State Management
- Logs: written to stdout (JSON) and mirrored into a bounded searchable store for admin tooling
- Metrics: in-memory, reset on restart
- Health checks: stateless, evaluated on each request
- Retention: stdout handling remains external; the searchable mirror has CortexOS-managed retention

## 12. Failure Modes and Error Handling
| Failure | Handling |
|---|---|
| Log write fails (stdout blocked) | Attempt searchable-store write, increment dropped_log_count metric, emit fallback diagnostic on next available channel |
| Health check times out | Return "unhealthy" for that check |
| Metrics overflow (counter) | Wrap around (u64 sufficient for all practical cases) |

## 13. Security and Permissions
- Metrics endpoint requires admin permission
- Health endpoint is unauthenticated (for load balancers)
- Logs never contain secrets, tokens, or passwords
- Log files readable by OS user only (644)

## 14. Performance Requirements
| Metric | Target |
|---|---|
| Log write overhead | < 10μs per entry |
| Metrics increment | < 1μs |
| Health check evaluation | < 500ms total |
| Metrics scrape | < 100ms |

## 15. Accessibility Requirements
- No direct user-facing UI (infrastructure layer)

## 16. Observability and Logging
- Self-referential: the logging system logs its own initialization
- Dropped log entries tracked as metric
- Log level changes via settings take effect immediately (no restart)

## 17. Testing Requirements
- Unit: log format validation (every entry has required fields)
- Unit: metric counter/gauge accuracy
- Unit: health check aggregation logic
- Integration: trace_id propagation across service calls
- Integration: metrics endpoint returns correct values

## 18. Acceptance Criteria
- [ ] All log entries are valid JSON with required fields
- [ ] Log levels filterable (only WARN+ in production)
- [ ] Trace IDs propagated across all service calls
- [ ] Health endpoint returns accurate status
- [ ] No secrets/tokens in any log entry
- [ ] Metrics endpoint returns all defined metrics
- [ ] Log write overhead < 10μs

## 19. Build Order and Dependencies
**Layer 9**. Depends on: 01, 02 (error taxonomy), cortex-core only

## 20. Non-Goals and Anti-Patterns
- No log aggregation to external services (v1)
- No custom log UI (v1)
- No alerting system (v1)
- NEVER log user content (file text, AI prompts) without privacy_mode applied
- NEVER log API keys, passwords, or tokens
- NEVER use println! or dbg! for production logging

## 21. Implementation Instructions for Claude Code / Codex
1. Add `cortex-observability` crate with `tracing-subscriber` JSON formatter.
2. Implement log level filtering from settings.
3. Implement trace_id middleware for HTTP requests.
4. Implement metrics registry with Counter, Gauge, Histogram types.
5. Implement health check registry and aggregation.
6. Implement `/api/v1/health` and `/api/v1/metrics` endpoints.
7. Write tests: log format, trace propagation, health aggregation.
