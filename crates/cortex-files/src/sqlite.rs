//! SQLite-backed implementation of [`FilesService`].

use std::path::PathBuf;

use cortex_core::{FileId, UserId};
use sha2::{Digest, Sha256};

use crate::error::{FilesError, Result};
use crate::service::FilesService;
use crate::types::{FileContent, FileEntry, FileMetadata, VirtualPath};

// ---------------------------------------------------------------------------
// SqliteFilesService
// ---------------------------------------------------------------------------

/// SQLite-backed virtual filesystem service.
///
/// File metadata lives in the `files` table. Blob content is stored on disk
/// under `data_dir/blobs/<file_id>`.
pub struct SqliteFilesService {
    pool: cortex_db::Pool,
    data_dir: String,
    default_owner: UserId,
}

/// Convert a [`cortex_db::DbError`] into a [`FilesError`].
fn db_to_files(e: cortex_db::DbError) -> FilesError {
    FilesError::IoError(format!("db error: {e}"))
}

/// Helper: map a rusqlite row to [`FileEntry`].
fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<FileEntry> {
    let file_id_str: String = row.get(0)?;
    let parent_id_str: Option<String> = row.get(1)?;
    let name: String = row.get(2)?;
    let is_directory: bool = row.get::<_, i32>(3)? != 0;
    let _content_hash: Option<String> = row.get(4)?;
    let size_bytes: i64 = row.get(5)?;
    let mime_type: String = row.get(6)?;
    let owner_id_str: String = row.get(7)?;
    let created_at: String = row.get(8)?;
    let updated_at: String = row.get(9)?;

    Ok(FileEntry {
        file_id: FileId(uuid::Uuid::parse_str(&file_id_str).unwrap()),
        parent_id: parent_id_str.map(|s| FileId(uuid::Uuid::parse_str(&s).unwrap())),
        name,
        is_directory,
        size_bytes: size_bytes as u64,
        mime_type,
        owner_id: UserId(uuid::Uuid::parse_str(&owner_id_str).unwrap()),
        created_at,
        updated_at,
    })
}

impl SqliteFilesService {
    /// Create a new service instance.
    pub fn new(pool: cortex_db::Pool, data_dir: String, default_owner: UserId) -> Self {
        Self {
            pool,
            data_dir,
            default_owner,
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Absolute path to the blob directory.
    fn blobs_dir(&self) -> PathBuf {
        PathBuf::from(&self.data_dir).join("blobs")
    }

    /// Walk path components from the root to resolve a [`VirtualPath`] to a
    /// [`FileId`]. Returns `Ok(None)` when the path does not exist.
    fn resolve_path(&self, path: &VirtualPath) -> Result<Option<FileId>> {
        let raw = path.as_str();
        let components: Vec<&str> = raw.split('/').collect();

        self.pool
            .read(|conn| {
                let mut current_id: Option<FileId> = None;

                for component in &components {
                    let row = match &current_id {
                        Some(pid) => conn.query_row(
                            "SELECT file_id FROM files WHERE parent_id = ?1 AND name = ?2",
                            rusqlite::params![pid.0.to_string(), *component],
                            |row| row.get::<_, String>(0),
                        ),
                        None => conn.query_row(
                            "SELECT file_id FROM files WHERE parent_id IS NULL AND name = ?1",
                            [component],
                            |row| row.get::<_, String>(0),
                        ),
                    };

                    match row {
                        Ok(id_str) => {
                            let uuid = uuid::Uuid::parse_str(&id_str)
                                .map_err(|e| cortex_db::DbError::Query(format!("{e}")))?;
                            current_id = Some(FileId(uuid));
                        }
                        Err(rusqlite::Error::QueryReturnedNoRows) => {
                            return Ok(None);
                        }
                        Err(e) => {
                            return Err(cortex_db::DbError::Query(format!(
                                "resolve_path query failed: {e}"
                            )));
                        }
                    }
                }

                Ok(current_id)
            })
            .map_err(db_to_files)
    }

    /// Resolve the parent directory and the final filename component.
    ///
    /// For `"docs/notes.txt"` the parent is the `FileId` of the `"docs"`
    /// directory and the name is `"notes.txt"`. For a single-component path
    /// like `"readme.txt"` the parent is `None` (root).
    fn resolve_parent(&self, path: &VirtualPath) -> Result<(Option<FileId>, String)> {
        let raw = path.as_str();
        let mut components: Vec<&str> = raw.split('/').collect();

        let file_name = components
            .pop()
            .ok_or_else(|| FilesError::PathViolation("path has no components".into()))?
            .to_string();

        if components.is_empty() {
            return Ok((None, file_name));
        }

        let parent_path = VirtualPath::new_unchecked(components.join("/"));
        let parent_id = self.resolve_path(&parent_path)?;

        match parent_id {
            Some(id) => Ok((Some(id), file_name)),
            None => Err(FilesError::NotFound(format!(
                "parent directory not found for path: {raw}"
            ))),
        }
    }

    /// Detect MIME type from the file extension.
    fn detect_mime(name: &str) -> String {
        let lower = name.to_ascii_lowercase();
        if lower.ends_with(".txt") {
            "text/plain".to_string()
        } else if lower.ends_with(".json") {
            "application/json".to_string()
        } else if lower.ends_with(".md") {
            "text/markdown".to_string()
        } else if lower.ends_with(".html") {
            "text/html".to_string()
        } else if lower.ends_with(".css") {
            "text/css".to_string()
        } else if lower.ends_with(".js") {
            "text/javascript".to_string()
        } else if lower.ends_with(".png") {
            "image/png".to_string()
        } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
            "image/jpeg".to_string()
        } else if lower.ends_with(".pdf") {
            "application/pdf".to_string()
        } else {
            "application/octet-stream".to_string()
        }
    }

    /// Create a directory at the given virtual path. Any missing parent
    /// directories are created automatically (like `mkdir -p`).
    pub fn create_directory(&self, path: &VirtualPath) -> Result<FileEntry> {
        let raw = path.as_str();
        let components: Vec<&str> = raw.split('/').collect();
        let owner_str = self.default_owner.0.to_string();

        self.pool
            .write(|conn| {
                let mut current_parent: Option<FileId> = None;

                for component in &components {
                    let existing = match &current_parent {
                        Some(pid) => conn
                            .query_row(
                                "SELECT file_id FROM files WHERE parent_id = ?1 AND name = ?2",
                                rusqlite::params![pid.0.to_string(), *component],
                                |row| row.get::<_, String>(0),
                            )
                            .ok(),
                        None => conn
                            .query_row(
                                "SELECT file_id FROM files WHERE parent_id IS NULL AND name = ?1",
                                [component],
                                |row| row.get::<_, String>(0),
                            )
                            .ok(),
                    };

                    match existing {
                        Some(id_str) => {
                            current_parent = Some(FileId(uuid::Uuid::parse_str(&id_str).unwrap()));
                        }
                        None => {
                            let new_id = FileId(uuid::Uuid::now_v7());
                            let new_id_str = new_id.0.to_string();

                            conn.execute(
                                "INSERT INTO files \
                                 (file_id, parent_id, name, is_directory, content_hash, \
                                  size_bytes, mime_type, owner_id) \
                                 VALUES (?1, ?2, ?3, 1, NULL, 0, '', ?4)",
                                rusqlite::params![
                                    new_id_str,
                                    current_parent.as_ref().map(|p| p.0.to_string()),
                                    component,
                                    owner_str,
                                ],
                            )
                            .map_err(|e| {
                                cortex_db::DbError::Query(format!("insert directory failed: {e}"))
                            })?;

                            current_parent = Some(new_id);
                        }
                    }
                }

                let dir_id =
                    current_parent.ok_or_else(|| cortex_db::DbError::Query("empty path".into()))?;

                conn.query_row(
                    "SELECT file_id, parent_id, name, is_directory, content_hash, \
                     size_bytes, mime_type, owner_id, created_at, updated_at \
                     FROM files WHERE file_id = ?1",
                    [&dir_id.0.to_string()],
                    row_to_entry,
                )
                .map_err(|e| cortex_db::DbError::Query(format!("fetch directory failed: {e}")))
            })
            .map_err(db_to_files)
    }
}

// ---------------------------------------------------------------------------
// FilesService implementation
// ---------------------------------------------------------------------------

impl FilesService for SqliteFilesService {
    fn list(
        &self,
        path: &VirtualPath,
    ) -> impl std::future::Future<Output = Result<Vec<FileEntry>>> + Send {
        // Resolve synchronously, then move into async block.
        let parent_id = self.resolve_path(path);
        let pool = self.pool.clone();

        async move {
            let parent_id = parent_id?;

            pool.read(|conn| {
                let mut entries = Vec::new();

                let query = match &parent_id {
                    Some(_) => {
                        "SELECT file_id, parent_id, name, is_directory, content_hash, \
                         size_bytes, mime_type, owner_id, created_at, updated_at \
                         FROM files WHERE parent_id = ?1"
                    }
                    None => {
                        "SELECT file_id, parent_id, name, is_directory, content_hash, \
                         size_bytes, mime_type, owner_id, created_at, updated_at \
                         FROM files WHERE parent_id IS NULL"
                    }
                };

                let rows: Vec<FileEntry> = match &parent_id {
                    Some(pid) => {
                        let mut stmt = conn
                            .prepare(query)
                            .map_err(|e| cortex_db::DbError::Query(format!("prepare list: {e}")))?;
                        let mapped = stmt
                            .query_map([&pid.0.to_string()], row_to_entry)
                            .map_err(|e| cortex_db::DbError::Query(format!("list query: {e}")))?;
                        mapped
                            .collect::<std::result::Result<Vec<_>, _>>()
                            .map_err(|e| cortex_db::DbError::Query(format!("row mapping: {e}")))?
                    }
                    None => {
                        let mut stmt = conn
                            .prepare(query)
                            .map_err(|e| cortex_db::DbError::Query(format!("prepare list: {e}")))?;
                        let mapped = stmt
                            .query_map([], row_to_entry)
                            .map_err(|e| cortex_db::DbError::Query(format!("list query: {e}")))?;
                        mapped
                            .collect::<std::result::Result<Vec<_>, _>>()
                            .map_err(|e| cortex_db::DbError::Query(format!("row mapping: {e}")))?
                    }
                };

                entries.extend(rows);
                Ok(entries)
            })
            .map_err(db_to_files)
        }
    }

    fn read(
        &self,
        path: &VirtualPath,
    ) -> impl std::future::Future<Output = Result<FileContent>> + Send {
        let file_id = self.resolve_path(path);
        let pool = self.pool.clone();
        let blobs_dir = self.blobs_dir();
        let path_str = path.as_str().to_string();

        async move {
            let file_id = file_id?
                .ok_or_else(|| FilesError::NotFound(format!("file not found: {path_str}")))?;

            let (entry, content_hash) = pool
                .read(|conn| {
                    let entry = conn
                        .query_row(
                            "SELECT file_id, parent_id, name, is_directory, content_hash, \
                             size_bytes, mime_type, owner_id, created_at, updated_at \
                             FROM files WHERE file_id = ?1",
                            [&file_id.0.to_string()],
                            row_to_entry,
                        )
                        .map_err(|e| match e {
                            rusqlite::Error::QueryReturnedNoRows => {
                                cortex_db::DbError::Query(format!("file not found: {}", file_id.0))
                            }
                            other => {
                                cortex_db::DbError::Query(format!("read query failed: {other}"))
                            }
                        })?;

                    let hash: Option<String> = conn
                        .query_row(
                            "SELECT content_hash FROM files WHERE file_id = ?1",
                            [&file_id.0.to_string()],
                            |row| row.get(0),
                        )
                        .map_err(|e| {
                            cortex_db::DbError::Query(format!("hash query failed: {e}"))
                        })?;

                    Ok((entry, hash))
                })
                .map_err(db_to_files)?;

            let data = std::fs::read(blobs_dir.join(file_id.0.to_string()))
                .map_err(|e| FilesError::IoError(format!("cannot read blob: {e}")))?;

            Ok(FileContent {
                metadata: FileMetadata {
                    entry,
                    content_hash,
                },
                data,
            })
        }
    }

    fn write(
        &self,
        path: &VirtualPath,
        data: &[u8],
    ) -> impl std::future::Future<Output = Result<FileEntry>> + Send {
        let parent_result = self.resolve_parent(path);
        let pool = self.pool.clone();
        let data_dir = self.data_dir.clone();
        let default_owner = self.default_owner.clone();
        let data_owned = data.to_vec();

        async move {
            let (parent_id, file_name) = parent_result?;

            let hash = {
                let mut hasher = Sha256::new();
                hasher.update(&data_owned);
                format!("{:x}", hasher.finalize())
            };
            let size = data_owned.len() as i64;
            let mime = Self::detect_mime(&file_name);
            let owner_str = default_owner.0.to_string();
            let parent_str = parent_id.as_ref().map(|p| p.0.to_string());

            let entry = pool
                .write(|conn| {
                    // Check if a file with this parent+name already exists.
                    let existing: Option<String> = match &parent_id {
                        Some(pid) => conn
                            .query_row(
                                "SELECT file_id FROM files WHERE parent_id = ?1 AND name = ?2",
                                rusqlite::params![pid.0.to_string(), file_name],
                                |row| row.get(0),
                            )
                            .ok(),
                        None => conn
                            .query_row(
                                "SELECT file_id FROM files WHERE parent_id IS NULL AND name = ?1",
                                [&file_name],
                                |row| row.get(0),
                            )
                            .ok(),
                    };

                    if let Some(existing_id_str) = existing {
                        // UPDATE existing file.
                        conn.execute(
                            "UPDATE files SET content_hash = ?1, size_bytes = ?2, \
                             mime_type = ?3, updated_at = datetime('now') \
                             WHERE file_id = ?4",
                            rusqlite::params![&hash, size, &mime, &existing_id_str],
                        )
                        .map_err(|e| {
                            cortex_db::DbError::Query(format!("update file failed: {e}"))
                        })?;

                        conn.query_row(
                            "SELECT file_id, parent_id, name, is_directory, content_hash, \
                             size_bytes, mime_type, owner_id, created_at, updated_at \
                             FROM files WHERE file_id = ?1",
                            [&existing_id_str],
                            row_to_entry,
                        )
                        .map_err(|e| {
                            cortex_db::DbError::Query(format!("fetch after update failed: {e}"))
                        })
                    } else {
                        // INSERT new file.
                        let new_id = FileId(uuid::Uuid::now_v7());
                        let new_id_str = new_id.0.to_string();

                        conn.execute(
                            "INSERT INTO files \
                             (file_id, parent_id, name, is_directory, content_hash, \
                              size_bytes, mime_type, owner_id) \
                             VALUES (?1, ?2, ?3, 0, ?4, ?5, ?6, ?7)",
                            rusqlite::params![
                                new_id_str, parent_str, file_name, hash, size, mime, owner_str,
                            ],
                        )
                        .map_err(|e| {
                            cortex_db::DbError::Query(format!("insert file failed: {e}"))
                        })?;

                        conn.query_row(
                            "SELECT file_id, parent_id, name, is_directory, content_hash, \
                             size_bytes, mime_type, owner_id, created_at, updated_at \
                             FROM files WHERE file_id = ?1",
                            [&new_id_str],
                            row_to_entry,
                        )
                        .map_err(|e| {
                            cortex_db::DbError::Query(format!("fetch after insert failed: {e}"))
                        })
                    }
                })
                .map_err(db_to_files)?;

            // Write blob to disk after the transaction commits.
            let blobs_dir = PathBuf::from(&data_dir).join("blobs");
            std::fs::create_dir_all(&blobs_dir)
                .map_err(|e| FilesError::IoError(format!("cannot create blobs dir: {e}")))?;
            std::fs::write(blobs_dir.join(entry.file_id.0.to_string()), &data_owned)
                .map_err(|e| FilesError::IoError(format!("cannot write blob: {e}")))?;

            Ok(entry)
        }
    }

    fn delete(&self, path: &VirtualPath) -> impl std::future::Future<Output = Result<()>> + Send {
        let file_id = self.resolve_path(path);
        let pool = self.pool.clone();
        let data_dir = self.data_dir.clone();
        let path_str = path.as_str().to_string();

        async move {
            let file_id = file_id?
                .ok_or_else(|| FilesError::NotFound(format!("file not found: {path_str}")))?;

            // Collect all file IDs to delete (directory children included).
            // Children must be deleted before their parent due to FK
            // constraints, so we collect in reverse order (deepest first).
            let ids_to_delete: Vec<FileId> = pool
                .read(|conn| {
                    let is_dir: bool = conn
                        .query_row(
                            "SELECT is_directory FROM files WHERE file_id = ?1",
                            [&file_id.0.to_string()],
                            |row| row.get::<_, i32>(0),
                        )
                        .map(|v| v != 0)
                        .unwrap_or(false);

                    if !is_dir {
                        return Ok(vec![file_id.clone()]);
                    }

                    // BFS to collect all descendants, then reverse so that
                    // children come before parents (deepest nesting first).
                    let mut all_ids = Vec::new();
                    let mut stack = vec![file_id.clone()];

                    while let Some(parent) = stack.pop() {
                        let children: Vec<String> = {
                            let mut stmt = conn
                                .prepare("SELECT file_id FROM files WHERE parent_id = ?1")
                                .map_err(|e| {
                                    cortex_db::DbError::Query(format!("prepare children: {e}"))
                                })?;
                            let mapped = stmt
                                .query_map([&parent.0.to_string()], |row| row.get::<_, String>(0))
                                .map_err(|e| {
                                    cortex_db::DbError::Query(format!("query children: {e}"))
                                })?;
                            mapped
                                .collect::<std::result::Result<Vec<_>, _>>()
                                .map_err(|e| {
                                    cortex_db::DbError::Query(format!("collect children: {e}"))
                                })?
                        };

                        for child_str in &children {
                            let child_id = FileId(uuid::Uuid::parse_str(child_str).unwrap());
                            stack.push(child_id.clone());
                            all_ids.push(child_id);
                        }
                    }

                    // Children first, then the directory itself.
                    all_ids.push(file_id.clone());

                    Ok(all_ids)
                })
                .map_err(db_to_files)?;

            // Delete database rows inside a transaction.
            pool.write(|conn| {
                for id in &ids_to_delete {
                    conn.execute("DELETE FROM files WHERE file_id = ?1", [&id.0.to_string()])
                        .map_err(|e| {
                            cortex_db::DbError::Query(format!("delete row failed: {e}"))
                        })?;
                }
                Ok(())
            })
            .map_err(db_to_files)?;

            // Delete blob files from disk.
            let blobs_dir = PathBuf::from(&data_dir).join("blobs");
            for id in &ids_to_delete {
                let blob_path = blobs_dir.join(id.0.to_string());
                if blob_path.exists() {
                    std::fs::remove_file(&blob_path)
                        .map_err(|e| FilesError::IoError(format!("cannot delete blob: {e}")))?;
                }
            }

            Ok(())
        }
    }

    fn move_file(
        &self,
        from: &VirtualPath,
        to: &VirtualPath,
    ) -> impl std::future::Future<Output = Result<FileEntry>> + Send {
        let source_id = self.resolve_path(from);
        let to_resolve = self.resolve_parent(to);
        let from_str = from.as_str().to_string();
        let pool = self.pool.clone();

        async move {
            let source_id = source_id?
                .ok_or_else(|| FilesError::NotFound(format!("source not found: {from_str}")))?;
            let (new_parent_id, new_name) = to_resolve?;

            pool.write(|conn| {
                conn.execute(
                    "UPDATE files SET name = ?1, parent_id = ?2, \
                     updated_at = datetime('now') WHERE file_id = ?3",
                    rusqlite::params![
                        new_name,
                        new_parent_id.as_ref().map(|p| p.0.to_string()),
                        source_id.0.to_string(),
                    ],
                )
                .map_err(|e| cortex_db::DbError::Query(format!("move update failed: {e}")))?;

                conn.query_row(
                    "SELECT file_id, parent_id, name, is_directory, content_hash, \
                     size_bytes, mime_type, owner_id, created_at, updated_at \
                     FROM files WHERE file_id = ?1",
                    [&source_id.0.to_string()],
                    row_to_entry,
                )
                .map_err(|e| cortex_db::DbError::Query(format!("fetch after move failed: {e}")))
            })
            .map_err(db_to_files)
        }
    }

    fn get_metadata(
        &self,
        path: &VirtualPath,
    ) -> impl std::future::Future<Output = Result<FileMetadata>> + Send {
        let file_id = self.resolve_path(path);
        let pool = self.pool.clone();
        let path_str = path.as_str().to_string();

        async move {
            let file_id = file_id?
                .ok_or_else(|| FilesError::NotFound(format!("file not found: {path_str}")))?;

            pool.read(|conn| {
                let (entry, content_hash): (FileEntry, Option<String>) = conn
                    .query_row(
                        "SELECT file_id, parent_id, name, is_directory, content_hash, \
                         size_bytes, mime_type, owner_id, created_at, updated_at \
                         FROM files WHERE file_id = ?1",
                        [&file_id.0.to_string()],
                        |row| {
                            let entry = row_to_entry(row)?;
                            let hash: Option<String> = row.get(4)?;
                            Ok((entry, hash))
                        },
                    )
                    .map_err(|e| match e {
                        rusqlite::Error::QueryReturnedNoRows => {
                            cortex_db::DbError::Query(format!("file not found: {}", file_id.0))
                        }
                        other => {
                            cortex_db::DbError::Query(format!("metadata query failed: {other}"))
                        }
                    })?;

                Ok(FileMetadata {
                    entry,
                    content_hash,
                })
            })
            .map_err(db_to_files)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_db::run_migrations;

    /// Helper: create a fresh in-memory database with migrations applied and a
    /// test user inserted.
    fn setup() -> SqliteFilesService {
        let pool = cortex_db::Pool::open_in_memory().unwrap();
        run_migrations(&pool).unwrap();

        let owner = UserId(uuid::Uuid::now_v7());
        // Insert a test user so FK constraints are satisfied.
        pool.write(|conn| {
            conn.execute(
                "INSERT INTO users (user_id, username, password_hash) VALUES (?1, ?2, ?3)",
                rusqlite::params![owner.0.to_string(), "testuser", "hash"],
            )
            .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
            Ok(())
        })
        .unwrap();

        let tmp_dir = tempfile::tempdir().unwrap();
        let data_dir = tmp_dir.path().to_string_lossy().to_string();
        // Leak the TempDir so it stays alive for the full test. Acceptable in
        // test code.
        std::mem::forget(tmp_dir);

        SqliteFilesService::new(pool, data_dir, owner)
    }

    #[tokio::test]
    async fn write_and_read_file() {
        let svc = setup();

        let path = VirtualPath::new("hello.txt").unwrap();
        let content = b"Hello, CortexOS!";

        let entry = svc.write(&path, content).await.unwrap();
        assert_eq!(entry.name, "hello.txt");
        assert_eq!(entry.size_bytes, content.len() as u64);
        assert_eq!(entry.mime_type, "text/plain");
        assert!(!entry.is_directory);

        let file = svc.read(&path).await.unwrap();
        assert_eq!(file.data, content);
        assert_eq!(file.metadata.entry.file_id, entry.file_id);
    }

    #[tokio::test]
    async fn write_updates_existing_file() {
        let svc = setup();

        let path = VirtualPath::new("doc.txt").unwrap();
        let _ = svc.write(&path, b"version 1").await.unwrap();
        let entry = svc.write(&path, b"version 2").await.unwrap();

        assert_eq!(entry.size_bytes, 9);
        let file = svc.read(&path).await.unwrap();
        assert_eq!(file.data, b"version 2");
    }

    #[tokio::test]
    async fn write_file_in_nested_directory() {
        let svc = setup();

        // Create parent directory first.
        let dir_path = VirtualPath::new("docs").unwrap();
        let dir_entry = svc.create_directory(&dir_path).unwrap();
        assert!(dir_entry.is_directory);
        assert_eq!(dir_entry.name, "docs");

        let file_path = VirtualPath::new("docs/notes.txt").unwrap();
        let entry = svc.write(&file_path, b"my notes").await.unwrap();
        assert_eq!(entry.name, "notes.txt");
        assert_eq!(entry.parent_id, Some(dir_entry.file_id));
    }

    #[tokio::test]
    async fn list_directory_contents() {
        let svc = setup();

        svc.create_directory(&VirtualPath::new("docs").unwrap())
            .unwrap();
        svc.write(&VirtualPath::new("docs/a.txt").unwrap(), b"a")
            .await
            .unwrap();
        svc.write(&VirtualPath::new("docs/b.txt").unwrap(), b"b")
            .await
            .unwrap();

        let entries = svc.list(&VirtualPath::new("docs").unwrap()).await.unwrap();
        assert_eq!(entries.len(), 2);
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"a.txt"));
        assert!(names.contains(&"b.txt"));
    }

    #[tokio::test]
    async fn delete_file() {
        let svc = setup();

        let path = VirtualPath::new("temp.txt").unwrap();
        svc.write(&path, b"temporary").await.unwrap();
        svc.delete(&path).await.unwrap();

        let result = svc.read(&path).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            FilesError::NotFound(_) => {}
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn delete_directory_recursive() {
        let svc = setup();

        svc.create_directory(&VirtualPath::new("dir").unwrap())
            .unwrap();
        svc.write(&VirtualPath::new("dir/child.txt").unwrap(), b"child")
            .await
            .unwrap();

        svc.delete(&VirtualPath::new("dir").unwrap()).await.unwrap();

        // Child should be gone too.
        let result = svc.read(&VirtualPath::new("dir/child.txt").unwrap()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn move_file() {
        let svc = setup();

        svc.write(&VirtualPath::new("original.txt").unwrap(), b"data")
            .await
            .unwrap();
        svc.create_directory(&VirtualPath::new("dest").unwrap())
            .unwrap();

        let moved = svc
            .move_file(
                &VirtualPath::new("original.txt").unwrap(),
                &VirtualPath::new("dest/renamed.txt").unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(moved.name, "renamed.txt");

        // Old path should be gone.
        let old = svc.read(&VirtualPath::new("original.txt").unwrap()).await;
        assert!(old.is_err());

        // New path should be readable with same content.
        let file = svc
            .read(&VirtualPath::new("dest/renamed.txt").unwrap())
            .await
            .unwrap();
        assert_eq!(file.data, b"data");
    }

    #[tokio::test]
    async fn get_metadata_returns_content_hash() {
        let svc = setup();

        let path = VirtualPath::new("hashed.txt").unwrap();
        let data = b"hash me";
        svc.write(&path, data).await.unwrap();

        let meta = svc.get_metadata(&path).await.unwrap();
        assert!(meta.content_hash.is_some());
        assert_eq!(meta.entry.name, "hashed.txt");

        // Verify the hash matches manual SHA-256 computation.
        let expected = {
            let mut hasher = Sha256::new();
            hasher.update(data);
            format!("{:x}", hasher.finalize())
        };
        assert_eq!(meta.content_hash.unwrap(), expected);
    }

    #[tokio::test]
    async fn read_nonexistent_file_returns_not_found() {
        let svc = setup();

        let result = svc.read(&VirtualPath::new("nope.txt").unwrap()).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            FilesError::NotFound(msg) => {
                assert!(msg.contains("nope.txt"), "unexpected message: {msg}");
            }
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn create_nested_directory() {
        let svc = setup();

        let dir = svc
            .create_directory(&VirtualPath::new("a/b/c").unwrap())
            .unwrap();
        assert!(dir.is_directory);
        assert_eq!(dir.name, "c");

        // Verify intermediate directories were created.
        let meta_a = svc
            .get_metadata(&VirtualPath::new("a").unwrap())
            .await
            .unwrap();
        assert!(meta_a.entry.is_directory);

        let meta_b = svc
            .get_metadata(&VirtualPath::new("a/b").unwrap())
            .await
            .unwrap();
        assert!(meta_b.entry.is_directory);
    }

    #[tokio::test]
    async fn mime_type_detection() {
        let svc = setup();

        let cases = vec![
            ("file.json", "application/json"),
            ("file.md", "text/markdown"),
            ("file.html", "text/html"),
            ("file.css", "text/css"),
            ("file.js", "text/javascript"),
            ("file.png", "image/png"),
            ("file.jpg", "image/jpeg"),
            ("file.jpeg", "image/jpeg"),
            ("file.pdf", "application/pdf"),
            ("file.txt", "text/plain"),
            ("file.bin", "application/octet-stream"),
        ];

        for (name, expected_mime) in cases {
            let path = VirtualPath::new(name).unwrap();
            let entry = svc.write(&path, b"x").await.unwrap();
            assert_eq!(entry.mime_type, expected_mime, "wrong MIME type for {name}");
        }
    }

    #[tokio::test]
    async fn list_root_directory() {
        let svc = setup();

        // Create some root-level entries
        svc.create_directory(&VirtualPath::new("Documents").unwrap())
            .unwrap();
        svc.write(&VirtualPath::new("readme.txt").unwrap(), b"hello")
            .await
            .unwrap();

        // Use "." as the canonical root sentinel — resolve_path won't find
        // any file named "." so it returns None, and list() queries
        // parent_id IS NULL (root-level entries).
        let root = VirtualPath::new(".").unwrap();
        let entries = svc.list(&root).await.unwrap();

        assert_eq!(entries.len(), 2);
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"Documents"));
        assert!(names.contains(&"readme.txt"));
    }

    #[tokio::test]
    async fn list_root_empty() {
        let svc = setup();

        // Root with no entries should return empty list
        let root = VirtualPath::new(".").unwrap();
        let entries = svc.list(&root).await.unwrap();
        assert!(entries.is_empty());
    }
}
