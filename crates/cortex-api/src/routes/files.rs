//! Files API routes.
//!
//! Framework-agnostic handler functions for the virtual filesystem.

use crate::app_state::AppState;
use crate::error::{ApiError, Result};
use cortex_core::SuccessResponse;
use cortex_files::types::VirtualPath;
use cortex_files::FilesService;
use serde::{Deserialize, Serialize};

/// Response body for file metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntryResponse {
    pub file_id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub is_directory: bool,
    pub size_bytes: u64,
    pub mime_type: String,
    pub owner_id: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Response body for file content (metadata + base64 data).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContentResponse {
    pub metadata: FileEntryResponse,
    pub content_hash: Option<String>,
    pub data: String, // base64-encoded
}

/// Request body for writing a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteFileRequest {
    pub path: String,
    pub content_base64: String,
}

fn entry_to_response(e: &cortex_files::types::FileEntry) -> FileEntryResponse {
    FileEntryResponse {
        file_id: e.file_id.0.to_string(),
        parent_id: e.parent_id.as_ref().map(|p| p.0.to_string()),
        name: e.name.clone(),
        is_directory: e.is_directory,
        size_bytes: e.size_bytes,
        mime_type: e.mime_type.clone(),
        owner_id: e.owner_id.0.to_string(),
        created_at: e.created_at.clone(),
        updated_at: e.updated_at.clone(),
    }
}

/// List directory contents.
pub async fn list(state: &AppState, path: &str) -> Result<SuccessResponse<Vec<FileEntryResponse>>> {
    let vpath =
        VirtualPath::new(path).map_err(|e| ApiError::BadRequest(format!("invalid path: {e}")))?;

    let entries = state
        .files
        .list(&vpath)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let items = entries.iter().map(entry_to_response).collect();

    Ok(SuccessResponse {
        data: items,
        meta: None,
    })
}

/// Read file content.
pub async fn read(state: &AppState, path: &str) -> Result<SuccessResponse<FileContentResponse>> {
    let vpath =
        VirtualPath::new(path).map_err(|e| ApiError::BadRequest(format!("invalid path: {e}")))?;

    let content = state.files.read(&vpath).await.map_err(|e| match e {
        cortex_files::FilesError::NotFound(msg) => ApiError::NotFound(msg),
        cortex_files::FilesError::PathViolation(msg) => ApiError::BadRequest(msg),
        _ => ApiError::Internal(e.to_string()),
    })?;

    Ok(SuccessResponse {
        data: FileContentResponse {
            metadata: entry_to_response(&content.metadata.entry),
            content_hash: content.metadata.content_hash,
            data: base64_encode(&content.data),
        },
        meta: None,
    })
}

/// Write (create or update) a file.
pub async fn write(
    state: &AppState,
    req: WriteFileRequest,
) -> Result<SuccessResponse<FileEntryResponse>> {
    let vpath = VirtualPath::new(&req.path)
        .map_err(|e| ApiError::BadRequest(format!("invalid path: {e}")))?;

    let data = base64_decode(&req.content_base64)
        .map_err(|e| ApiError::BadRequest(format!("invalid base64: {e}")))?;

    let entry = state
        .files
        .write(&vpath, &data)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(SuccessResponse {
        data: entry_to_response(&entry),
        meta: None,
    })
}

/// Delete a file or directory.
pub async fn delete(state: &AppState, path: &str) -> Result<SuccessResponse<()>> {
    let vpath =
        VirtualPath::new(path).map_err(|e| ApiError::BadRequest(format!("invalid path: {e}")))?;

    state.files.delete(&vpath).await.map_err(|e| match e {
        cortex_files::FilesError::NotFound(msg) => ApiError::NotFound(msg),
        _ => ApiError::Internal(e.to_string()),
    })?;

    Ok(SuccessResponse {
        data: (),
        meta: None,
    })
}

/// Move/rename a file.
pub async fn move_file(
    state: &AppState,
    from: &str,
    to: &str,
) -> Result<SuccessResponse<FileEntryResponse>> {
    let from_path = VirtualPath::new(from)
        .map_err(|e| ApiError::BadRequest(format!("invalid source path: {e}")))?;
    let to_path = VirtualPath::new(to)
        .map_err(|e| ApiError::BadRequest(format!("invalid destination path: {e}")))?;

    let entry = state
        .files
        .move_file(&from_path, &to_path)
        .await
        .map_err(|e| match e {
            cortex_files::FilesError::NotFound(msg) => ApiError::NotFound(msg),
            _ => ApiError::Internal(e.to_string()),
        })?;

    Ok(SuccessResponse {
        data: entry_to_response(&entry),
        meta: None,
    })
}

// -- helpers --

fn base64_encode(data: &[u8]) -> String {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
    BASE64.encode(data)
}

fn base64_decode(s: &str) -> std::result::Result<Vec<u8>, String> {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
    BASE64.decode(s).map_err(|e| format!("{e}"))
}
