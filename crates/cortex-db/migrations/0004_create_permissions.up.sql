CREATE TABLE IF NOT EXISTS permission_grants (
    grant_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(user_id),
    app_id TEXT NOT NULL,
    permission TEXT NOT NULL,
    granted_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id, app_id, permission)
);

CREATE INDEX IF NOT EXISTS idx_permissions_user_app ON permission_grants(user_id, app_id);
