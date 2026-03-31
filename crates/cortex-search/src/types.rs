//! Search domain types.

use serde::{Deserialize, Serialize};

/// A search query submitted by the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// The search text.
    pub query: String,
    /// Maximum number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Optional content type filter (e.g. "file", "app", "setting").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

/// A single search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Identifier of the source document or entity.
    pub source_id: String,
    /// The type of content this result represents.
    pub content_type: String,
    /// A text snippet from the matching content.
    pub snippet: String,
    /// Relevance score (higher is better, range 0.0 - 1.0).
    pub relevance: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_query_construction_minimal() {
        let query = SearchQuery {
            query: "hello".to_string(),
            limit: None,
            content_type: None,
        };
        assert_eq!(query.query, "hello");
        assert!(query.limit.is_none());
        assert!(query.content_type.is_none());
    }

    #[test]
    fn search_query_construction_full() {
        let query = SearchQuery {
            query: "config".to_string(),
            limit: Some(20),
            content_type: Some("file".to_string()),
        };
        assert_eq!(query.limit, Some(20));
        assert_eq!(query.content_type.as_deref(), Some("file"));
    }

    #[test]
    fn search_query_serde_roundtrip() {
        let query = SearchQuery {
            query: "test query".to_string(),
            limit: Some(10),
            content_type: Some("app".to_string()),
        };
        let json = serde_json::to_string(&query).unwrap();
        let parsed: SearchQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(query.query, parsed.query);
        assert_eq!(query.limit, parsed.limit);
        assert_eq!(query.content_type, parsed.content_type);
    }

    #[test]
    fn search_query_skips_none_fields() {
        let query = SearchQuery {
            query: "minimal".to_string(),
            limit: None,
            content_type: None,
        };
        let json = serde_json::to_string(&query).unwrap();
        assert!(!json.contains("limit"));
        assert!(!json.contains("content_type"));
    }

    #[test]
    fn search_result_construction() {
        let result = SearchResult {
            source_id: "doc-123".to_string(),
            content_type: "file".to_string(),
            snippet: "Hello world...".to_string(),
            relevance: 0.95,
        };
        assert_eq!(result.source_id, "doc-123");
        assert!((result.relevance - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn search_result_serde_roundtrip() {
        let result = SearchResult {
            source_id: "app-456".to_string(),
            content_type: "app".to_string(),
            snippet: "Calculator...".to_string(),
            relevance: 0.87,
        };
        let json = serde_json::to_string(&result).unwrap();
        let parsed: SearchResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.source_id, parsed.source_id);
        assert_eq!(result.content_type, parsed.content_type);
        assert!((result.relevance - parsed.relevance).abs() < f64::EPSILON);
    }
}
