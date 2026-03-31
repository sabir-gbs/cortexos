//! Files domain types.

use cortex_core::{FileId, Timestamp, UserId};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// VirtualPath
// ---------------------------------------------------------------------------

/// A validated relative path inside the CortexOS virtual filesystem.
///
/// Paths must be non-empty, relative (no leading `/`), and must not contain
/// `..` components or null bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VirtualPath {
    path: String,
}

impl VirtualPath {
    /// Create a new `VirtualPath`, returning an error if validation fails.
    pub fn new(path: impl Into<String>) -> crate::error::Result<Self> {
        let path = path.into();
        Self::validate(&path)?;
        Ok(Self { path })
    }

    /// Create a `VirtualPath` without validation.
    ///
    /// Intended for use in contexts where the path is already known to be
    /// safe (e.g. deserialization, internal construction from trusted data).
    pub fn new_unchecked(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    /// The raw path string.
    pub fn as_str(&self) -> &str {
        &self.path
    }

    /// Validate a path string, returning `Ok(())` if it is acceptable.
    ///
    /// Rejects:
    /// - empty strings
    /// - paths that start with `/`
    /// - paths containing `..`
    /// - paths containing null bytes
    pub fn validate(path: &str) -> crate::error::Result<()> {
        if path.is_empty() {
            return Err(crate::error::FilesError::PathViolation(
                "path must not be empty".into(),
            ));
        }
        if path.starts_with('/') {
            return Err(crate::error::FilesError::PathViolation(
                "path must be relative, not absolute".into(),
            ));
        }
        if path.contains("..") {
            return Err(crate::error::FilesError::PathViolation(
                "path must not contain '..'".into(),
            ));
        }
        if path.contains('\0') {
            return Err(crate::error::FilesError::PathViolation(
                "path must not contain null bytes".into(),
            ));
        }
        Ok(())
    }
}

impl std::fmt::Display for VirtualPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.path)
    }
}

// ---------------------------------------------------------------------------
// FileEntry
// ---------------------------------------------------------------------------

/// A file or directory entry in the virtual filesystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Unique file identifier.
    pub file_id: FileId,
    /// The file or directory name (basename).
    pub name: String,
    /// Parent directory ID (`None` for the root).
    pub parent_id: Option<FileId>,
    /// Whether this entry is a directory.
    pub is_directory: bool,
    /// Size in bytes (0 for directories).
    pub size_bytes: u64,
    /// MIME type (empty string for directories).
    pub mime_type: String,
    /// The user who owns this file.
    pub owner_id: UserId,
    /// When the file was created (ISO 8601).
    pub created_at: Timestamp,
    /// When the file was last modified (ISO 8601).
    pub updated_at: Timestamp,
}

// ---------------------------------------------------------------------------
// FileMetadata
// ---------------------------------------------------------------------------

/// Extended metadata for a file entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// The underlying file entry.
    pub entry: FileEntry,
    /// Optional content hash (e.g. SHA-256 hex digest).
    pub content_hash: Option<String>,
}

// ---------------------------------------------------------------------------
// FileContent
// ---------------------------------------------------------------------------

/// The full content of a file, including metadata and raw bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    /// Metadata for the file.
    pub metadata: FileMetadata,
    /// The raw file bytes.
    pub data: Vec<u8>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_core::{FileId, UserId};

    fn fake_file_id() -> FileId {
        FileId(uuid::Uuid::now_v7())
    }

    fn fake_user_id() -> UserId {
        UserId(uuid::Uuid::now_v7())
    }

    fn now_timestamp() -> Timestamp {
        "2026-03-30T12:00:00Z".to_string()
    }

    // ---- VirtualPath validation: rejections ----

    #[test]
    fn virtual_path_rejects_empty() {
        let err = VirtualPath::new("").unwrap_err();
        match err {
            crate::error::FilesError::PathViolation(msg) => {
                assert!(msg.contains("empty"), "unexpected message: {msg}");
            }
            other => panic!("expected PathViolation, got {other:?}"),
        }
    }

    #[test]
    fn virtual_path_rejects_absolute() {
        let err = VirtualPath::new("/etc/passwd").unwrap_err();
        match err {
            crate::error::FilesError::PathViolation(msg) => {
                assert!(msg.contains("relative"), "unexpected message: {msg}");
            }
            other => panic!("expected PathViolation, got {other:?}"),
        }
    }

    #[test]
    fn virtual_path_rejects_parent_traversal() {
        let err = VirtualPath::new("../secret").unwrap_err();
        match err {
            crate::error::FilesError::PathViolation(msg) => {
                assert!(msg.contains("'..'"), "unexpected message: {msg}");
            }
            other => panic!("expected PathViolation, got {other:?}"),
        }
    }

    #[test]
    fn virtual_path_rejects_parent_traversal_mid() {
        let err = VirtualPath::new("foo/../bar").unwrap_err();
        match err {
            crate::error::FilesError::PathViolation(msg) => {
                assert!(msg.contains("'..'"), "unexpected message: {msg}");
            }
            other => panic!("expected PathViolation, got {other:?}"),
        }
    }

    #[test]
    fn virtual_path_rejects_null_bytes() {
        let err = VirtualPath::new("foo\0bar").unwrap_err();
        match err {
            crate::error::FilesError::PathViolation(msg) => {
                assert!(msg.contains("null"), "unexpected message: {msg}");
            }
            other => panic!("expected PathViolation, got {other:?}"),
        }
    }

    // ---- VirtualPath validation: acceptances ----

    #[test]
    fn virtual_path_accepts_simple_name() {
        let vp = VirtualPath::new("readme.txt").unwrap();
        assert_eq!(vp.as_str(), "readme.txt");
    }

    #[test]
    fn virtual_path_accepts_nested_relative() {
        let vp = VirtualPath::new("docs/notes/session.md").unwrap();
        assert_eq!(vp.as_str(), "docs/notes/session.md");
    }

    #[test]
    fn virtual_path_accepts_single_component() {
        let vp = VirtualPath::new("file").unwrap();
        assert_eq!(vp.as_str(), "file");
    }

    // ---- VirtualPath display ----

    #[test]
    fn virtual_path_display() {
        let vp = VirtualPath::new_unchecked("hello/world");
        assert_eq!(format!("{vp}"), "hello/world");
    }

    // ---- FileEntry construction ----

    #[test]
    fn file_entry_construction() {
        let file_id = fake_file_id();
        let parent_id = fake_file_id();
        let owner = fake_user_id();
        let ts = now_timestamp();

        let entry = FileEntry {
            file_id: file_id.clone(),
            name: "document.pdf".to_string(),
            parent_id: Some(parent_id.clone()),
            is_directory: false,
            size_bytes: 2048,
            mime_type: "application/pdf".to_string(),
            owner_id: owner.clone(),
            created_at: ts.clone(),
            updated_at: ts,
        };

        assert_eq!(entry.name, "document.pdf");
        assert!(!entry.is_directory);
        assert_eq!(entry.size_bytes, 2048);
        assert_eq!(entry.mime_type, "application/pdf");
        assert_eq!(entry.file_id, file_id);
        assert_eq!(entry.parent_id, Some(parent_id));
        assert_eq!(entry.owner_id, owner);
    }

    #[test]
    fn file_entry_directory() {
        let entry = FileEntry {
            file_id: fake_file_id(),
            name: "documents".to_string(),
            parent_id: None,
            is_directory: true,
            size_bytes: 0,
            mime_type: String::new(),
            owner_id: fake_user_id(),
            created_at: now_timestamp(),
            updated_at: now_timestamp(),
        };
        assert!(entry.is_directory);
        assert_eq!(entry.size_bytes, 0);
        assert!(entry.mime_type.is_empty());
        assert!(entry.parent_id.is_none());
    }

    // ---- Serialization roundtrips ----

    #[test]
    fn virtual_path_serde_roundtrip() {
        let vp = VirtualPath::new("photos/vacation/beach.jpg").unwrap();
        let json = serde_json::to_string(&vp).unwrap();
        let vp2: VirtualPath = serde_json::from_str(&json).unwrap();
        assert_eq!(vp, vp2);
    }

    #[test]
    fn file_entry_serde_roundtrip() {
        let entry = FileEntry {
            file_id: fake_file_id(),
            name: "notes.txt".to_string(),
            parent_id: Some(fake_file_id()),
            is_directory: false,
            size_bytes: 128,
            mime_type: "text/plain".to_string(),
            owner_id: fake_user_id(),
            created_at: now_timestamp(),
            updated_at: now_timestamp(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let entry2: FileEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.file_id, entry2.file_id);
        assert_eq!(entry.name, entry2.name);
        assert_eq!(entry.parent_id, entry2.parent_id);
        assert_eq!(entry.is_directory, entry2.is_directory);
        assert_eq!(entry.size_bytes, entry2.size_bytes);
        assert_eq!(entry.mime_type, entry2.mime_type);
        assert_eq!(entry.owner_id, entry2.owner_id);
        assert_eq!(entry.created_at, entry2.created_at);
        assert_eq!(entry.updated_at, entry2.updated_at);
    }

    #[test]
    fn file_metadata_serde_roundtrip() {
        let meta = FileMetadata {
            entry: FileEntry {
                file_id: fake_file_id(),
                name: "image.png".to_string(),
                parent_id: None,
                is_directory: false,
                size_bytes: 4096,
                mime_type: "image/png".to_string(),
                owner_id: fake_user_id(),
                created_at: now_timestamp(),
                updated_at: now_timestamp(),
            },
            content_hash: Some(
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
            ),
        };
        let json = serde_json::to_string(&meta).unwrap();
        let meta2: FileMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(meta.entry.file_id, meta2.entry.file_id);
        assert_eq!(meta.content_hash, meta2.content_hash);
    }

    #[test]
    fn file_content_serde_roundtrip() {
        let content = FileContent {
            metadata: FileMetadata {
                entry: FileEntry {
                    file_id: fake_file_id(),
                    name: "data.bin".to_string(),
                    parent_id: None,
                    is_directory: false,
                    size_bytes: 5,
                    mime_type: "application/octet-stream".to_string(),
                    owner_id: fake_user_id(),
                    created_at: now_timestamp(),
                    updated_at: now_timestamp(),
                },
                content_hash: None,
            },
            data: vec![1, 2, 3, 4, 5],
        };
        let json = serde_json::to_string(&content).unwrap();
        let content2: FileContent = serde_json::from_str(&json).unwrap();

        assert_eq!(content.data, content2.data);
        assert_eq!(content.metadata.entry.name, content2.metadata.entry.name);
    }
}
