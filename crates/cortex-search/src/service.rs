//! Search service trait.

use crate::error::Result;
use crate::types::{SearchQuery, SearchResult};

/// Search service trait.
///
/// Provides full-text search across files, apps, settings, and other
/// indexed content. The `search` method accepts a [`SearchQuery`] and
/// returns ranked [`SearchResult`] values.
pub trait SearchService: Send + Sync {
    /// Execute a search query and return matching results.
    fn search(
        &self,
        query: SearchQuery,
    ) -> impl std::future::Future<Output = Result<Vec<SearchResult>>> + Send;
}
