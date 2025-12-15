/*!
 * Database connection management.
 *
 * This module handles SQLite database connection creation, initialization,
 * and provides async-safe access patterns using tokio's spawn_blocking.
 */

use anyhow::{Context, Result};
use log::{debug, info};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::schema;

/// Default database filename
const DEFAULT_DB_FILENAME: &str = "yastwai.db";

/// Default database directory name under user's data directory
const DEFAULT_DB_DIRNAME: &str = "yastwai";

/// Database connection wrapper with thread-safe access
#[derive(Clone)]
pub struct DatabaseConnection {
    /// Path to the database file
    db_path: PathBuf,
    /// Thread-safe connection wrapped in Arc<Mutex>
    connection: Arc<Mutex<Connection>>,
}

impl DatabaseConnection {
    /// Create a new database connection at the default location
    pub fn new_default() -> Result<Self> {
        let db_path = Self::default_database_path()?;
        Self::new(&db_path)
    }

    /// Create a new database connection at the specified path
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create database directory: {:?}", parent))?;
        }

        info!("Opening database at: {:?}", db_path);

        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open database: {:?}", db_path))?;

        // Initialize schema
        schema::initialize_schema(&conn)?;

        Ok(Self {
            db_path,
            connection: Arc::new(Mutex::new(conn)),
        })
    }

    /// Create an in-memory database (for testing)
    pub fn new_in_memory() -> Result<Self> {
        debug!("Creating in-memory database");

        let conn =
            Connection::open_in_memory().context("Failed to create in-memory database")?;

        // Initialize schema
        schema::initialize_schema(&conn)?;

        Ok(Self {
            db_path: PathBuf::from(":memory:"),
            connection: Arc::new(Mutex::new(conn)),
        })
    }

    /// Get the default database path
    pub fn default_database_path() -> Result<PathBuf> {
        // Try to use the system data directory
        let base_dir = dirs::data_local_dir()
            .or_else(dirs::data_dir)
            .or_else(|| dirs::home_dir().map(|h| h.join(".local").join("share")))
            .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?;

        let db_dir = base_dir.join(DEFAULT_DB_DIRNAME);
        let db_path = db_dir.join(DEFAULT_DB_FILENAME);

        Ok(db_path)
    }

    /// Get the database file path
    pub fn path(&self) -> &Path {
        &self.db_path
    }

    /// Execute a database operation with the connection
    ///
    /// This method acquires the mutex lock and executes the provided closure
    /// with access to the connection. For async contexts, use `execute_async`.
    pub fn execute<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let conn = self
            .connection
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire database lock: {}", e))?;

        f(&conn)
    }

    /// Execute a mutable database operation with the connection
    pub fn execute_mut<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Connection) -> Result<T>,
    {
        let mut conn = self
            .connection
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire database lock: {}", e))?;

        f(&mut conn)
    }

    /// Execute a database operation asynchronously using spawn_blocking
    ///
    /// This is the preferred method for async contexts as it prevents
    /// blocking the async runtime.
    pub async fn execute_async<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let conn = self.connection.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| anyhow::anyhow!("Failed to acquire database lock: {}", e))?;

            f(&conn)
        })
        .await
        .context("Database task panicked")?
    }

    /// Execute a mutable database operation asynchronously
    pub async fn execute_mut_async<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Connection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let conn = self.connection.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = conn
                .lock()
                .map_err(|e| anyhow::anyhow!("Failed to acquire database lock: {}", e))?;

            f(&mut conn)
        })
        .await
        .context("Database task panicked")?
    }

    /// Begin a transaction and execute operations within it
    pub fn transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&rusqlite::Transaction) -> Result<T>,
    {
        let mut conn = self
            .connection
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire database lock: {}", e))?;

        let tx = conn.transaction()?;
        let result = f(&tx)?;
        tx.commit()?;

        Ok(result)
    }

    /// Begin an async transaction and execute operations within it
    pub async fn transaction_async<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&rusqlite::Transaction) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let conn = self.connection.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = conn
                .lock()
                .map_err(|e| anyhow::anyhow!("Failed to acquire database lock: {}", e))?;

            let tx = conn.transaction()?;
            let result = f(&tx)?;
            tx.commit()?;

            Ok(result)
        })
        .await
        .context("Database transaction task panicked")?
    }

    /// Vacuum the database to reclaim space
    pub fn vacuum(&self) -> Result<()> {
        self.execute(|conn| {
            conn.execute("VACUUM", [])?;
            Ok(())
        })
    }

    /// Get database statistics
    pub fn stats(&self) -> Result<DatabaseStats> {
        self.execute(|conn| {
            let session_count: i64 = conn
                .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
                .unwrap_or(0);

            let cache_count: i64 = conn
                .query_row("SELECT COUNT(*) FROM translation_cache", [], |row| row.get(0))
                .unwrap_or(0);

            let total_entries: i64 = conn
                .query_row("SELECT COUNT(*) FROM source_entries", [], |row| row.get(0))
                .unwrap_or(0);

            let translated_entries: i64 = conn
                .query_row("SELECT COUNT(*) FROM translated_entries", [], |row| row.get(0))
                .unwrap_or(0);

            // Get file size if not in-memory
            let file_size = if self.db_path.to_string_lossy() != ":memory:" {
                std::fs::metadata(&self.db_path)
                    .map(|m| m.len())
                    .unwrap_or(0)
            } else {
                0
            };

            Ok(DatabaseStats {
                session_count,
                cache_count,
                total_entries,
                translated_entries,
                file_size_bytes: file_size,
            })
        })
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    /// Number of translation sessions
    pub session_count: i64,
    /// Number of cached translations
    pub cache_count: i64,
    /// Total number of source entries across all sessions
    pub total_entries: i64,
    /// Number of translated entries
    pub translated_entries: i64,
    /// Database file size in bytes
    pub file_size_bytes: u64,
}

impl std::fmt::Display for DatabaseStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Sessions: {}, Cache entries: {}, Source entries: {}, Translated: {}, Size: {} KB",
            self.session_count,
            self.cache_count,
            self.total_entries,
            self.translated_entries,
            self.file_size_bytes / 1024
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newInMemory_shouldCreateValidConnection() {
        let db = DatabaseConnection::new_in_memory().expect("Failed to create in-memory DB");
        assert_eq!(db.path().to_string_lossy(), ":memory:");
    }

    #[test]
    fn test_execute_shouldRunOperation() {
        let db = DatabaseConnection::new_in_memory().expect("Failed to create DB");

        let result = db.execute(|conn| {
            let count: i64 = conn.query_row("SELECT 1 + 1", [], |row| row.get(0))?;
            Ok(count)
        });

        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_transaction_shouldCommitOnSuccess() {
        let db = DatabaseConnection::new_in_memory().expect("Failed to create DB");

        db.transaction(|tx| {
            tx.execute(
                "INSERT INTO sessions (id, source_file_path, source_file_hash, source_language, target_language, provider, model, total_entries, created_at, updated_at)
                 VALUES ('tx-test', '/path', 'hash', 'en', 'fr', 'ollama', 'llama2', 10, datetime('now'), datetime('now'))",
                [],
            )?;
            Ok(())
        }).expect("Transaction failed");

        // Verify the insert was committed
        let count: i64 = db
            .execute(|conn| {
                Ok(conn.query_row(
                    "SELECT COUNT(*) FROM sessions WHERE id = 'tx-test'",
                    [],
                    |row| row.get(0),
                )?)
            })
            .unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn test_stats_shouldReturnValidStats() {
        let db = DatabaseConnection::new_in_memory().expect("Failed to create DB");

        let stats = db.stats().expect("Failed to get stats");

        assert_eq!(stats.session_count, 0);
        assert_eq!(stats.cache_count, 0);
        assert_eq!(stats.total_entries, 0);
    }

    #[tokio::test]
    async fn test_executeAsync_shouldRunInBlockingContext() {
        let db = DatabaseConnection::new_in_memory().expect("Failed to create DB");

        let result = db
            .execute_async(|conn| {
                let count: i64 = conn.query_row("SELECT 42", [], |row| row.get(0))?;
                Ok(count)
            })
            .await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_transactionAsync_shouldWork() {
        let db = DatabaseConnection::new_in_memory().expect("Failed to create DB");

        db.transaction_async(|tx| {
            tx.execute(
                "INSERT INTO sessions (id, source_file_path, source_file_hash, source_language, target_language, provider, model, total_entries, created_at, updated_at)
                 VALUES ('async-tx-test', '/path', 'hash', 'en', 'fr', 'ollama', 'llama2', 10, datetime('now'), datetime('now'))",
                [],
            )?;
            Ok(())
        }).await.expect("Async transaction failed");

        // Verify
        let count: i64 = db
            .execute_async(|conn| {
                Ok(conn.query_row(
                    "SELECT COUNT(*) FROM sessions WHERE id = 'async-tx-test'",
                    [],
                    |row| row.get(0),
                )?)
            })
            .await
            .unwrap();

        assert_eq!(count, 1);
    }
}
