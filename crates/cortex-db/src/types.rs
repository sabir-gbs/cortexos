//! Database types and connection pool.

use std::path::Path;
use std::sync::Arc;

use parking_lot::Mutex;
use rusqlite::Connection;

use crate::error::{DbError, Result};

/// Thread-safe SQLite connection wrapper.
///
/// In v1, CortexOS uses a single-process model with an in-memory SQLite pool
/// backed by a WAL-mode file. Access is serialized through a Mutex since
/// SQLite supports concurrent reads but serialized writes.
#[derive(Clone)]
pub struct Pool {
    inner: Arc<Mutex<Connection>>,
}

impl Pool {
    /// Open a new SQLite database at the given path.
    ///
    /// Creates the file if it doesn't exist. Enables WAL mode for
    /// concurrent read performance.
    pub fn open(database_url: &str) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = Path::new(database_url).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                DbError::Connection(format!("cannot create database directory: {e}"))
            })?;
        }

        let conn = Connection::open(database_url)
            .map_err(|e| DbError::Connection(format!("cannot open database: {e}")))?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| DbError::Connection(format!("pragma setup failed: {e}")))?;

        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    /// Open an in-memory SQLite database (for testing).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| DbError::Connection(format!("cannot open in-memory database: {e}")))?;

        conn.execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(|e| DbError::Connection(format!("pragma setup failed: {e}")))?;

        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    /// Acquire a lock on the connection and execute a closure.
    ///
    /// This serializes all database access. For read-heavy workloads,
    /// consider using `read` which documents the read intent.
    pub fn with_conn<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let conn = self.inner.lock();
        f(&conn)
    }

    /// Execute a write operation inside a transaction.
    pub fn write<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let mut conn = self.inner.lock();
        let tx = conn
            .transaction()
            .map_err(|e| DbError::Query(format!("cannot begin transaction: {e}")))?;
        let result = f(&tx)?;
        tx.commit()
            .map_err(|e| DbError::Query(format!("commit failed: {e}")))?;
        Ok(result)
    }

    /// Read-only access (currently same as with_conn but documents intent).
    pub fn read<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        self.with_conn(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_in_memory_works() {
        let pool = Pool::open_in_memory().unwrap();
        pool.with_conn(|conn| {
            let val: i64 = conn
                .query_row("SELECT 1", [], |row| row.get(0))
                .map_err(|e| DbError::Query(e.to_string()))?;
            assert_eq!(val, 1);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn write_transaction_commits() {
        let pool = Pool::open_in_memory().unwrap();
        pool.write(|conn| {
            conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, val TEXT)", [])
                .map_err(|e| DbError::Query(e.to_string()))?;
            Ok(())
        })
        .unwrap();

        pool.read(|conn| {
            let mut stmt = conn
                .prepare("SELECT COUNT(*) FROM test")
                .map_err(|e| DbError::Query(e.to_string()))?;
            let count: i64 = stmt
                .query_row([], |row| row.get(0))
                .map_err(|e| DbError::Query(e.to_string()))?;
            assert_eq!(count, 0);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn write_transaction_rollback_on_error() {
        let pool = Pool::open_in_memory().unwrap();
        // Create table
        pool.write(|conn| {
            conn.execute(
                "CREATE TABLE test (id INTEGER PRIMARY KEY, val TEXT NOT NULL)",
                [],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;
            Ok(())
        })
        .unwrap();

        // Try an insert that will fail
        let result: Result<()> = pool.write(|conn| {
            conn.execute("INSERT INTO test (id, val) VALUES (1, NULL)", [])
                .map_err(|e| DbError::Query(e.to_string()))?;
            Ok(())
        });
        assert!(result.is_err());

        // Verify no row was committed
        pool.read(|conn| {
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM test", [], |row| row.get(0))
                .map_err(|e| DbError::Query(e.to_string()))?;
            assert_eq!(count, 0);
            Ok(())
        })
        .unwrap();
    }
}
