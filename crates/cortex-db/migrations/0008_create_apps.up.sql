CREATE TABLE IF NOT EXISTS app_instances (
    instance_id TEXT PRIMARY KEY,
    app_id TEXT NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(user_id),
    state TEXT NOT NULL DEFAULT 'stopped',
    window_id TEXT,
    launched_at TEXT,
    stopped_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_app_instances_user ON app_instances(user_id);
CREATE INDEX IF NOT EXISTS idx_app_instances_app ON app_instances(app_id);
