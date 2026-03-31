-- Idempotency tracking for the command bus.
-- Ensures every command is processed exactly once, even across restarts.
CREATE TABLE IF NOT EXISTS command_bus_idempotency (
    command_id TEXT PRIMARY KEY,
    processed_at TEXT NOT NULL DEFAULT (datetime('now')),
    event_name TEXT NOT NULL,
    channel TEXT NOT NULL
);

-- Full event log for audit / replay / dead-letter recovery.
CREATE TABLE IF NOT EXISTS command_bus_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    command_id TEXT NOT NULL,
    event_name TEXT NOT NULL,
    channel TEXT NOT NULL,
    payload TEXT NOT NULL DEFAULT '{}',
    correlation_id TEXT,
    timestamp TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'published'
        CHECK (status IN ('published', 'failed', 'dead_letter')),
    error_message TEXT,
    processed_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (command_id) REFERENCES command_bus_idempotency(command_id)
);

CREATE INDEX IF NOT EXISTS idx_bus_events_event_name
    ON command_bus_events(event_name);

CREATE INDEX IF NOT EXISTS idx_bus_events_status
    ON command_bus_events(status);

CREATE INDEX IF NOT EXISTS idx_bus_events_timestamp
    ON command_bus_events(timestamp);
