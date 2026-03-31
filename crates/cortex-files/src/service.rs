//! Files service trait.

use crate::error::Result;
use crate::types::{FileContent, FileEntry, FileMetadata, VirtualPath};

/// Async service interface for the CortexOS virtual filesystem.
///
/// All file operations from apps go through this trait. Direct host
/// filesystem access from app code is not permitted.
pub trait FilesService: Send + Sync {
    /// List files and directories contained at the given path.
    fn list(
        &self,
        path: &VirtualPath,
    ) -> impl std::future::Future<Output = Result<Vec<FileEntry>>> + Send;

    /// Read a file's full content (metadata + bytes).
    fn read(
        &self,
        path: &VirtualPath,
    ) -> impl std::future::Future<Output = Result<FileContent>> + Send;

    /// Write bytes to a file at the given path, creating or replacing it.
    fn write(
        &self,
        path: &VirtualPath,
        data: &[u8],
    ) -> impl std::future::Future<Output = Result<FileEntry>> + Send;

    /// Delete the file or directory at the given path.
    fn delete(&self, path: &VirtualPath) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Move a file or directory from one path to another.
    fn move_file(
        &self,
        from: &VirtualPath,
        to: &VirtualPath,
    ) -> impl std::future::Future<Output = Result<FileEntry>> + Send;

    /// Retrieve extended metadata for the file at the given path.
    fn get_metadata(
        &self,
        path: &VirtualPath,
    ) -> impl std::future::Future<Output = Result<FileMetadata>> + Send;
}
