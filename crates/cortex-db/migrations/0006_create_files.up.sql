CREATE TABLE IF NOT EXISTS files (
    file_id TEXT PRIMARY KEY,
    parent_id TEXT REFERENCES files(file_id),
    name TEXT NOT NULL,
    is_directory INTEGER NOT NULL DEFAULT 0,
    content_hash TEXT,
    size_bytes INTEGER NOT NULL DEFAULT 0,
    mime_type TEXT NOT NULL DEFAULT 'application/octet-stream',
    owner_id TEXT NOT NULL REFERENCES users(user_id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(parent_id, name)
);

CREATE INDEX IF NOT EXISTS idx_files_parent ON files(parent_id);
CREATE INDEX IF NOT EXISTS idx_files_owner ON files(owner_id);
