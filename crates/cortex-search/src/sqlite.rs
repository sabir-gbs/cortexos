//! SQLite-backed implementation of [`SearchService`].
//!
//! Uses an FTS5 virtual table (`search_index`) for full-text search with
//! BM25 relevance ranking. The schema is created by migration 0011.

use cortex_db::DbError;
use cortex_db::Pool;

use crate::error::{Result, SearchError};
use crate::service::SearchService;
use crate::types::{SearchQuery, SearchResult};

/// Convert a [`DbError`] into a [`SearchError`].
///
/// Used to translate the pool's error type into the search crate's error type.
fn db_to_search(e: DbError) -> SearchError {
    SearchError::IndexError(e.to_string())
}

/// SQLite-backed search service using FTS5.
pub struct SqliteSearchService {
    pool: Pool,
}

impl SqliteSearchService {
    /// Create a new `SqliteSearchService` backed by the given database pool.
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Index a document into the FTS5 search index.
    ///
    /// Inserts (or replaces) the document identified by `source_id` so that it
    /// can be found by subsequent [`SearchService::search`] calls.
    pub fn index_document(&self, source_id: &str, content_type: &str, content: &str) -> Result<()> {
        self.pool
            .write(|conn| {
                conn.execute(
                    "INSERT OR REPLACE INTO search_index (source_id, content_type, content) \
                     VALUES (?1, ?2, ?3)",
                    rusqlite::params![source_id, content_type, content],
                )
                .map_err(|e| DbError::Query(e.to_string()))?;
                Ok(())
            })
            .map_err(db_to_search)
    }
}

impl SearchService for SqliteSearchService {
    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>> {
        // Validate query length.
        if query.query.len() < 2 {
            return Err(SearchError::QueryTooShort);
        }

        let limit = query.limit.unwrap_or(20);
        let has_content_type = query.content_type.is_some();

        // Build the SQL query.
        //
        // FTS5 column order in the migration is: content(0), content_type(1), source_id(2).
        // snippet() column index 0 targets the `content` column for highlighting.
        //
        // bm25() returns negative values for matches (more negative = better match).
        // We select -bm25() so the value is positive and higher means more relevant.
        // Relevance is normalized to [0.0, 1.0) using 1.0 / (1.0 + score).
        let sql = if has_content_type {
            "\
             SELECT source_id, \
                    content_type, \
                    snippet(search_index, 0, '<<', '>>', '...', 32), \
                    -bm25(search_index) \
             FROM search_index \
             WHERE search_index MATCH ?1 \
               AND content_type = ?2 \
             ORDER BY bm25(search_index) \
             LIMIT ?3"
        } else {
            "\
             SELECT source_id, \
                    content_type, \
                    snippet(search_index, 0, '<<', '>>', '...', 32), \
                    -bm25(search_index) \
             FROM search_index \
             WHERE search_index MATCH ?1 \
             ORDER BY bm25(search_index) \
             LIMIT ?2"
        };

        let query_text = query.query.clone();
        let content_type_filter = query.content_type.clone();

        self.pool
            .read(|conn| {
                let mut stmt = conn
                    .prepare(sql)
                    .map_err(|e| DbError::Query(e.to_string()))?;

                let mut rows = if has_content_type {
                    stmt.query(rusqlite::params![query_text, content_type_filter, limit])
                        .map_err(|e| DbError::Query(e.to_string()))?
                } else {
                    stmt.query(rusqlite::params![query_text, limit])
                        .map_err(|e| DbError::Query(e.to_string()))?
                };

                let mut results = Vec::new();

                while let Some(row) = rows.next().map_err(|e| DbError::Query(e.to_string()))? {
                    let source_id: String =
                        row.get(0).map_err(|e| DbError::Query(e.to_string()))?;
                    let content_type: String =
                        row.get(1).map_err(|e| DbError::Query(e.to_string()))?;
                    let snippet: String = row.get(2).map_err(|e| DbError::Query(e.to_string()))?;
                    let neg_bm25: f64 = row.get(3).map_err(|e| DbError::Query(e.to_string()))?;

                    // neg_bm25 is positive for matches. Normalize to (0.0, 1.0].
                    let relevance = 1.0 / (1.0 + neg_bm25);

                    results.push(SearchResult {
                        source_id,
                        content_type,
                        snippet,
                        relevance,
                    });
                }

                Ok(results)
            })
            .map_err(db_to_search)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_db::run_migrations;

    /// Helper: create a pool with all migrations applied.
    fn setup_pool() -> Pool {
        let pool = Pool::open_in_memory().unwrap();
        run_migrations(&pool).unwrap();
        pool
    }

    #[tokio::test]
    async fn index_and_search_returns_results() {
        let pool = setup_pool();
        let svc = SqliteSearchService::new(pool);

        svc.index_document(
            "doc-1",
            "file",
            "The quick brown fox jumps over the lazy dog",
        )
        .unwrap();
        svc.index_document(
            "doc-2",
            "file",
            "A completely different document about cats",
        )
        .unwrap();

        let results = svc
            .search(SearchQuery {
                query: "brown fox".to_string(),
                limit: None,
                content_type: None,
            })
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source_id, "doc-1");
        assert_eq!(results[0].content_type, "file");
        assert!(results[0].snippet.contains("<<"));
        assert!((0.0..=1.0).contains(&results[0].relevance));
    }

    #[tokio::test]
    async fn search_no_matches_returns_empty() {
        let pool = setup_pool();
        let svc = SqliteSearchService::new(pool);

        svc.index_document("doc-1", "file", "Hello world").unwrap();

        let results = svc
            .search(SearchQuery {
                query: "xyzzy nonexistent".to_string(),
                limit: None,
                content_type: None,
            })
            .await
            .unwrap();

        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn search_with_content_type_filter() {
        let pool = setup_pool();
        let svc = SqliteSearchService::new(pool);

        svc.index_document("doc-1", "file", "CortexOS configuration file")
            .unwrap();
        svc.index_document("doc-2", "app", "CortexOS launcher app")
            .unwrap();
        svc.index_document("doc-3", "setting", "CortexOS display setting")
            .unwrap();

        let results = svc
            .search(SearchQuery {
                query: "CortexOS".to_string(),
                limit: None,
                content_type: Some("app".to_string()),
            })
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source_id, "doc-2");
        assert_eq!(results[0].content_type, "app");
    }

    #[tokio::test]
    async fn search_with_custom_limit() {
        let pool = setup_pool();
        let svc = SqliteSearchService::new(pool);

        for i in 0..10 {
            svc.index_document(
                &format!("doc-{i}"),
                "file",
                &format!("document number {i} about testing"),
            )
            .unwrap();
        }

        let results = svc
            .search(SearchQuery {
                query: "testing".to_string(),
                limit: Some(3),
                content_type: None,
            })
            .await
            .unwrap();

        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn search_query_too_short() {
        let pool = setup_pool();
        let svc = SqliteSearchService::new(pool);

        let err = svc
            .search(SearchQuery {
                query: "x".to_string(),
                limit: None,
                content_type: None,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, SearchError::QueryTooShort));
    }
}
