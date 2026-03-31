//! CortexOS virtual filesystem and storage abstraction.
//!
//! Manages file metadata, directory trees, content storage, trash, and
//! quotas. All file access from apps is mediated by this service.

pub mod error;
pub mod service;
pub mod sqlite;
pub mod types;

pub use error::{FilesError, Result};
pub use service::FilesService;
pub use sqlite::SqliteFilesService;
pub use types::{FileContent, FileEntry, FileMetadata, VirtualPath};
