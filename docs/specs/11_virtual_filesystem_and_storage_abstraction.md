# 11 — Virtual Filesystem and Storage Abstraction

## 1. Purpose
Define the virtual filesystem that provides file storage for all apps in CortexOS. Files have stable identities independent of paths, content is stored separately from metadata, and all access is permission-gated.

## 2. Scope
- File/directory model with UUID-based identity
- Content-addressable storage (SHA-256)
- Metadata management (separate from content)
- Save/open semantics with permission checks
- Trash/restore, move/rename, conflict resolution
- Storage quotas
- File associations (mime type → default app)
- Advisory file locking

## 3. Out of Scope
- File manager UI (owned by spec 17d)
- Search indexing of file content (owned by spec 12)
- Permission model details (owned by spec 04, this spec checks only)

## 4. Objectives
1. Files have stable identity: moving/renaming never changes the file's UUID.
2. Content is deduplicated: identical content stored once.
3. Concurrent edits are detected and surfaced to users.
4. Apps never access files directly — all access through the file service.

## 5. User-Visible Behavior

| Action | Outcome |
|---|---|
| User saves file in app | File stored, path visible in file manager |
| User moves file | File appears at new location, same identity |
| User deletes file | Moved to trash, restorable within 30 days |
| User empties trash | Files permanently deleted, storage freed |
| Storage quota reached | Write rejected with "Storage full" message |
| Concurrent edit conflict | User prompted to choose: keep mine / keep theirs |

## 6. System Behavior

### 6.1 File Identity vs Path
- Every file/directory has a unique `id` (UUID v4)
- A file has exactly one canonical `path` (alias)
- Moving a file changes its path but preserves its `id`
- Path uniqueness enforced: no two files share the same path

### 6.2 Content Store
- Content stored by SHA-256 hash (content-addressable)
- Multiple files with identical content share storage (dedup)
- Content streamed: never fully loaded into memory
- Content blobs stored on filesystem under `/data/content/{hash[:2]}/{hash}`

### 6.3 Trash Behavior
- Delete = set `is_deleted=true`, move path to `/trash/{id}/`
- Trash entries have `deleted_at` timestamp
- Retention: 30 days default, configurable
- Cron job purges expired trash entries (runs daily)
- Restore: recreate original path, set `is_deleted=false`
- Permanent delete: remove metadata entry + content blob (if no other references)

### 6.4 Conflict Resolution
- Every file has a monotonically increasing `version` counter
- Write includes expected version: `write(file_id, content, expected_version)`
- If `expected_version != current_version` → return `ConflictDetected`
- User chooses: keep-mine (force overwrite), keep-theirs (discard), cancel
- No silent overwrites ever (INV-11-1)

### 6.5 File Associations
- Mapping: mime_type → default_app_id
- Default associations for built-in types (text/plain → text-editor, image/* → media-viewer)
- User can override in Settings under `apps.file_associations`
- Double-click in file manager opens with default app

## 7. Architecture

```
┌──────────────────────────────────────────┐
│            cortex-files                  │
│                                          │
│  ┌────────────────────────────────────┐  │
│  │      File Service (API)            │  │
│  │  (CRUD, permissions check,         │  │
│  │   conflict detection)              │  │
│  └──────────────┬─────────────────────┘  │
│                 │                        │
│  ┌──────────────┴─────────────────────┐  │
│  │      Metadata Store (SQLite)       │  │
│  │  (file records, directories,       │  │
│  │   permissions, tags, versions)     │  │
│  └──────────────┬─────────────────────┘  │
│                 │                        │
│  ┌──────────────┴─────────────────────┐  │
│  │      Content Store (Filesystem)    │  │
│  │  (SHA-256 addressed blobs,         │  │
│  │   streaming read/write, dedup)     │  │
│  └────────────────────────────────────┘  │
└──────────────────────────────────────────┘
```

## 8. Data Model

```rust
struct FileRecord {
    id: String,                      // UUID v4
    name: String,                    // Display name: "report.txt"
    parent_id: Option<String>,       // None = root
    path: String,                    // Canonical: "/home/user/Documents/report.txt"
    content_hash: Option<String>,    // SHA-256, None for directories
    size: u64,                       // Bytes, 0 for directories
    mime_type: Option<String>,       // Detected or declared
    owner_id: String,                // User ID
    permissions: FilePermissions,
    tags: Vec<String>,
    version: u64,                    // Monotonically increasing
    is_deleted: bool,
    is_directory: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    modified_at: chrono::DateTime<chrono::Utc>,
}

struct FilePermissions {
    owner_read: bool,
    owner_write: bool,
    owner_delete: bool,
    // No group/other in single-user v1
}

struct FileLock {
    file_id: String,
    lock_type: LockType,
    holder_id: String,              // app_id or user_id
    acquired_at: chrono::DateTime<chrono::Utc>,
}

enum LockType { Shared, Exclusive }

struct QuotaState {
    user_id: String,
    bytes_used: u64,
    bytes_limit: u64,               // Default: 1GB = 1073741824
}

struct FileAssociation {
    mime_type: String,
    app_id: String,
    is_default: bool,
}
```

### Directory Structure
```
/home/{user}/
  Documents/
  Downloads/
  Pictures/
  Music/
  Desktop/
/system/             (read-only)
  assets/
  config/
/apps/{app_id}/      (app sandbox, per-app isolated)
/tmp/                (ephemeral, cleared on restart)
/trash/{file_id}/    (deleted files, 30-day retention)
```

## 9. Public Interfaces

### REST API
```
GET    /api/v1/files/{id}                     → Get file metadata
GET    /api/v1/files/{id}/content             → Stream file content
PUT    /api/v1/files/{id}/content             → Write file content (streaming)
POST   /api/v1/files                          → Create file or directory
PUT    /api/v1/files/{id}/move                → Move to new parent/path
PUT    /api/v1/files/{id}/rename              → Rename
DELETE /api/v1/files/{id}                     → Soft delete (trash)
DELETE /api/v1/files/{id}/permanent           → Permanent delete
POST   /api/v1/files/{id}/restore             → Restore from trash
GET    /api/v1/files/{id}/lock                → Get lock status
PUT    /api/v1/files/{id}/lock                → Acquire lock
DELETE /api/v1/files/{id}/lock                → Release lock
GET    /api/v1/files/search?path=...          → List directory contents
GET    /api/v1/quota                           → Get quota status
```

### WebSocket Events
```
file.created   → { id, path, is_directory }
file.modified  → { id, path, version, size }
file.deleted   → { id, path }
file.moved     → { id, old_path, new_path }
file.locked    → { id, lock_type, holder_id }
file.unlocked  → { id }
```

### Internal Rust API
```rust
trait FileService: Send + Sync {
    fn create(&self, parent_id: &str, name: &str, is_directory: bool, owner: &str) -> Result<FileRecord>;
    fn get(&self, id: &str) -> Result<FileRecord>;
    fn get_by_path(&self, path: &str) -> Result<FileRecord>;
    fn list_directory(&self, parent_id: &str) -> Result<Vec<FileRecord>>;
    fn read_content(&self, id: &str) -> Result<Box<dyn std::io::Read>>;
    fn write_content(&self, id: &str, content: Box<dyn std::io::Read>, expected_version: u64) -> Result<FileRecord>;
    fn move_file(&self, id: &str, new_parent_id: &str) -> Result<FileRecord>;
    fn rename(&self, id: &str, new_name: &str) -> Result<FileRecord>;
    fn delete(&self, id: &str) -> Result<()>;                    // Soft delete
    fn delete_permanent(&self, id: &str) -> Result<()>>;
    fn restore(&self, id: &str) -> Result<FileRecord>;
    fn acquire_lock(&self, id: &str, lock_type: LockType, holder: &str) -> Result<()>;
    fn release_lock(&self, id: &str, holder: &str) -> Result<()>;
    fn get_quota(&self, user_id: &str) -> Result<QuotaState>;
}
```

## 10. Internal Interfaces
- Permission checks via `cortex-policy` before every read/write/delete
- Events emitted via command bus (spec 10)
- Metadata stored via `cortex-db` (SQLite)
- Content blobs stored on local filesystem
- Search indexing notified via events (spec 12 subscribes to file events)

## 11. State Management
- File metadata in SQLite `files` table with indexes on `path`, `parent_id`, `owner_id`, `is_deleted`
- Content blobs on filesystem, referenced by hash
- Quota tracked: `SUM(size) WHERE owner_id = ? AND is_deleted = false`
- File locks in memory (cleared on restart — advisory only)
- Home directory structure created on user first login

## 12. Failure Modes and Error Handling

| Failure | Handling |
|---|---|
| Path already exists | Return CONF_001 |
| File not found | Return NF_001 |
| Version mismatch (conflict) | Return CONF_001 with both versions |
| Quota exceeded | Return QE_001 |
| Permission denied | Return POL_001 |
| Content hash mismatch | Retry write, if persists return PERM_001 |
| Parent directory doesn't exist | Return NF_001 |
| Name invalid (/, \\, .., null) | Return VAL_001 |
| Lock held by another | Return CONF_001 with lock holder info |

## 13. Security and Permissions
- Every file operation checks permissions via cortex-policy
- Apps can only access files within their declared permission paths
- `/system/` is read-only for all apps
- `/apps/{app_id}/` only accessible by that app
- File content never logged
- Content hashes not reversible to content
- API keys, secrets in files follow same permission model

## 14. Performance Requirements

| Metric | Target |
|---|---|
| File metadata read | < 5ms |
| Small file read (<1MB) | < 50ms |
| Small file write (<1MB) | < 100ms |
| Large file stream (100MB) | Sustained 50MB/s |
| Directory list (1000 files) | < 200ms |
| Quota check | < 5ms |
| Trash purge (100 files) | < 1s |

## 15. Accessibility Requirements
- File names support Unicode
- MIME type detection handles edge cases gracefully
- Error messages describe the problem clearly

## 16. Observability and Logging
- File operations logged at INFO: {operation, file_id, path, user_id, app_id}
- Conflict detections logged at WARN
- Quota warnings at 80% logged at WARN
- Content hashes never logged with file names
- Metrics: file_operations_total, storage_bytes_used, conflict_count

## 17. Testing Requirements
- Unit: file creation, move (path changes, id preserved), rename
- Unit: content deduplication (same content → same hash, one blob)
- Unit: version conflict detection
- Unit: quota enforcement
- Integration: full save/open cycle through API with permission check
- Integration: trash/restore cycle
- E2E: create file in text editor → verify in file manager → delete → restore

## 18. Acceptance Criteria
- [ ] File identity (UUID) preserved across move/rename
- [ ] Content deduplication verified (same content, one blob)
- [ ] Version conflict detected and reported (no silent overwrite)
- [ ] Trash: delete → restore → file at original path
- [ ] Quota: writes rejected when quota exceeded
- [ ] All operations permission-checked via cortex-policy
- [ ] Home directory structure created on first login
- [ ] /system/ is read-only
- [ ] File associations resolve to correct default apps
- [ ] Advisory locks work and auto-release on disconnect

## 19. Build Order and Dependencies
**Layer 4**. Depends on: 01, 02, 04 (permissions), cortex-db

## 20. Non-Goals and Anti-Patterns
- No symlinks/hardlinks (v1 — paths are single canonical)
- No file versioning history (v1 — only current version)
- No encryption at rest (v1 — relies on OS-level encryption)
- No network filesystem support
- NEVER silently overwrite on conflict
- NEVER allow apps to bypass file service for direct content access
- NEVER store file content in SQLite (blobs go to filesystem)
- NEVER log file content or full paths of sensitive directories

## 21. Implementation Instructions for Claude Code / Codex
1. Define FileRecord, FilePermissions, FileLock, QuotaState, FileAssociation structs.
2. Implement SQLite schema: `files` table with indexes.
3. Implement content store: SHA-256 hash → filesystem blob, streaming read/write.
4. Implement FileService trait: CRUD operations with permission checks.
5. Implement conflict detection: version check on every write.
6. Implement trash: soft delete with `is_deleted` flag, restore recreates path.
7. Implement quota: aggregate size on write, reject if over limit.
8. Implement advisory locks: in-memory HashMap, auto-release.
9. Implement file associations: mime → app mapping with defaults.
10. Write tests: identity preservation, dedup, conflict detection, quota, trash cycle.
