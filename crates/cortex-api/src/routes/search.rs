//! Search API routes.
//!
//! Framework-agnostic handler functions for full-text search.

use crate::app_state::AppState;
use crate::error::{ApiError, Result};
use cortex_core::SuccessResponse;
use cortex_search::SearchQuery;
use cortex_search::SearchService;
use serde::{Deserialize, Serialize};

/// Response body for search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub source_id: String,
    pub content_type: String,
    pub snippet: String,
    pub relevance: f64,
}

/// Search request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<u32>,
}

/// Execute a full-text search.
pub async fn search(
    state: &AppState,
    req: SearchRequest,
) -> Result<SuccessResponse<Vec<SearchResult>>> {
    let query = SearchQuery {
        query: req.query,
        limit: req.limit,
        content_type: None,
    };
    let results = state.search.search(query).await.map_err(|e| match e {
        cortex_search::SearchError::QueryTooShort => {
            ApiError::BadRequest("query too short".to_string())
        }
        _ => ApiError::Internal(e.to_string()),
    })?;

    let items = results
        .into_iter()
        .map(|r| SearchResult {
            source_id: r.source_id,
            content_type: r.content_type,
            snippet: r.snippet,
            relevance: r.relevance,
        })
        .collect();

    Ok(SuccessResponse {
        data: items,
        meta: None,
    })
}
