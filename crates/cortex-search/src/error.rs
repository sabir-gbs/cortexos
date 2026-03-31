//! Search error types.

/// Result alias for search operations.
pub type Result<T> = std::result::Result<T, SearchError>;

/// Errors that can occur during search operations.
#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    /// The search query was too short.
    #[error("query too short")]
    QueryTooShort,
    /// An indexing error occurred.
    #[error("index error: {0}")]
    IndexError(String),
    /// An unexpected internal error.
    #[error("internal error")]
    Internal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_too_short_display() {
        let err = SearchError::QueryTooShort;
        assert_eq!(format!("{err}"), "query too short");
    }

    #[test]
    fn index_error_display() {
        let err = SearchError::IndexError("corrupted segment".to_string());
        assert_eq!(format!("{err}"), "index error: corrupted segment");
    }

    #[test]
    fn internal_display() {
        let err = SearchError::Internal;
        assert_eq!(format!("{err}"), "internal error");
    }
}
