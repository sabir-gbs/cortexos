CREATE VIRTUAL TABLE IF NOT EXISTS search_index USING fts5(
    content,
    content_type,
    source_id,
    tokenize='unicode61'
);

CREATE TABLE IF NOT EXISTS search_index_meta (
    source_id TEXT PRIMARY KEY,
    content_type TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
