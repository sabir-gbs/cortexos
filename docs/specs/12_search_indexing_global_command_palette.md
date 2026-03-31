# 12 — Search Indexing and Global Command Palette

## 1. Purpose
Define the search and command palette system that provides global search across files, apps, settings, and commands, with full-text indexing of file content.

## 2. Scope
- Global search across files, apps, settings, commands
- Full-text indexing of file content (incremental updates)
- Command palette (Ctrl+Space)
- Search API
- Index management (build, update, query)
- Ranking and relevance scoring

## 3. Out of Scope
- File content reading (owned by spec 11, search only indexes)
- Specific app search features (apps own their internal search)
- AI-powered search (v1 — keyword only)

## 4. Objectives
1. Search results returned in < 100ms for typical queries.
2. Index updates are incremental — no full rebuild on single file change.
3. Command palette provides instant access to all OS commands.

## 5. User-Visible Behavior

| Action | Outcome |
|---|---|
| User presses Ctrl+Space | Command palette overlay appears |
| User types in command palette | Results filter in real-time |
| User types in search bar | Files, apps, settings, commands shown ranked |
| File created/modified/deleted | Search index updated within 5 seconds |

## 6. System Behavior

### 6.1 Search Result Types
```typescript
type SearchResultType = "app" | "command" | "file" | "setting";

interface SearchResult {
  type: SearchResultType;
  id: string;                    // Resource ID
  title: string;                 // Primary display text
  subtitle?: string;             // Secondary text (path, description)
  icon?: string;
  score: number;                 // Relevance score 0-1
  action: SearchAction;          // What happens on select
}

type SearchAction =
  | { type: "open_app"; app_id: string }
  | { type: "open_file"; file_id: string; app_id: string }
  | { type: "open_setting"; key: string }
  | { type: "execute_command"; command_id: string };
```

### 6.2 Ranking Algorithm
- Base score from term frequency (TF-IDF-lite): matches in title > matches in content
- Type priority weight: apps (1.5) > commands (1.3) > files (1.0) > settings (0.8)
- Recency boost: recently opened items get +0.2
- Exact prefix match: +0.3 bonus

### 6.3 Index Structure
- Metadata index: file records, app manifests, setting schemas, command registry
- Full-text index: SQLite FTS5 on file content
- Index updated incrementally on file events (not full rebuild)

### 6.4 Command Palette
- Triggered by Ctrl+Space
- Shows: all registered commands + recent items at top
- Commands registered by apps and services via command bus
- Keyboard-navigable: Arrow keys to select, Enter to execute, Escape to dismiss

## 7. Architecture
```
┌─────────────────────────────────┐
│         cortex-search           │
│  ┌────────────────────────────┐ │
│  │  Index Manager             │ │
│  │  (FTS5, metadata index,   │ │
│  │   incremental updates)     │ │
│  └────────────┬───────────────┘ │
│  ┌────────────┴───────────────┐ │
│  │  Query Engine              │ │
│  │  (ranking, filtering,      │ │
│  │   result merging)          │ │
│  └────────────┬───────────────┘ │
│  ┌────────────┴───────────────┐ │
│  │  Event Subscriber          │ │
│  │  (listens to FileEvent,    │ │
│  │   AppEvent, SettingEvent)  │ │
│  └────────────────────────────┘ │
└─────────────────────────────────┘
```

## 8. Data Model
```rust
struct SearchIndex {
    fts_table: String,             // SQLite FTS5 virtual table
    metadata_table: String,        // Key-value metadata index
}

struct IndexedItem {
    id: String,
    item_type: SearchResultType,
    title: String,
    content: Option<String>,       // For full-text (file content, descriptions)
    metadata: serde_json::Value,
    indexed_at: chrono::DateTime<chrono::Utc>,
}

struct SearchQuery {
    query: String,
    filters: Option<Vec<SearchFilter>>,
    limit: Option<u32>,            // Default: 20
    offset: Option<u32>,
}

struct SearchFilter {
    field: String,                 // "type", "mime_type", "category"
    operator: String,              // "eq", "contains"
    value: serde_json::Value,
}
```

## 9. Public Interfaces
```
GET /api/v1/search?q={query}&type={filter}&limit={n}    → Search results
GET /api/v1/search/suggestions?q={prefix}               → Auto-complete suggestions
POST /api/v1/search/reindex                              → Force full reindex (admin only)
```

## 10. Internal Interfaces
- Subscribes to `file.created`, `file.modified`, `file.deleted` events
- Subscribes to `app.launched`, `app.stopped` events
- Subscribes to `settings.changed` events
- Reads file content from cortex-files for indexing

## 11. State Management
- FTS5 index in SQLite (persistent)
- Metadata index in SQLite
- Full reindex on first startup; incremental thereafter
- Index stored in the `cortex-db`-managed SQLite store (or a dedicated SQLite database owned by `cortex-search` and managed under the same backup/retention policy)

## 12. Failure Modes and Error Handling
| Failure | Handling |
|---|---|
| Index corrupted | Rebuild from scratch, log at WARN |
| File content unreadable | Skip file, log at WARN |
| Search query timeout | Return partial results + timeout indicator |
| Index write failure | Queue for retry, log at ERROR |

## 13. Security and Permissions
- Search results filtered by user permissions (files user can't read don't appear)
- Command palette only shows commands user can execute
- No file content returned in search results (only metadata)

## 14. Performance Requirements
| Metric | Target |
|---|---|
| Search query | < 100ms |
| Auto-complete | < 50ms |
| Index update (single file) | < 500ms |
| Full reindex (1000 files) | < 30s |

## 15. Accessibility Requirements
- Command palette fully keyboard navigable
- Search results announced to screen readers
- Loading states indicated with aria-busy

## 16. Observability and Logging
- Search queries logged at DEBUG (query, result_count, latency_ms)
- Index updates logged at TRACE
- Full reindex logged at INFO
- Metrics: search_queries_total, search_latency_ms, index_size_bytes

## 17. Testing Requirements
- Unit: ranking algorithm with known inputs
- Unit: prefix matching for auto-complete
- Integration: index update on file event → search returns new file
- Integration: permission filtering in results
- E2E: create file → search → find file in results

## 18. Acceptance Criteria
- [ ] Search returns files, apps, settings, commands
- [ ] Ranking follows specified algorithm
- [ ] Index updates within 5 seconds of file change
- [ ] Command palette triggered by Ctrl+Space
- [ ] Results filtered by user permissions
- [ ] Auto-complete suggestions work for prefixes

## 19. Build Order and Dependencies
**Layer 9**. Depends on: 01, 02, 10 (events), 11 (file content for indexing)

## 20. Non-Goals and Anti-Patterns
- No AI-powered semantic search (v1)
- No search across file content for binary files
- No search history persistence (v1)
- NEVER index file content the user doesn't have permission to read
- NEVER return full file content in search results

## 21. Implementation Instructions for Claude Code / Codex
1. Set up SQLite FTS5 virtual table for full-text index.
2. Implement Index Manager: subscribe to events, update index incrementally.
3. Implement Query Engine: search across index with ranking.
4. Implement auto-complete: prefix matching on titles.
5. Implement command palette: command registry + search UI.
6. Write tests: ranking, incremental updates, permission filtering.
