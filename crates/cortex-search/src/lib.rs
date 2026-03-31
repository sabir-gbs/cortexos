//! CortexOS search indexing and global command palette.
//!
//! Provides full-text search across files and app content, plus the
//! global command palette surface for quick navigation.

pub mod error;
pub mod service;
pub mod sqlite;
pub mod types;

pub use error::{Result, SearchError};
pub use service::SearchService;
pub use sqlite::SqliteSearchService;
pub use types::{SearchQuery, SearchResult};
