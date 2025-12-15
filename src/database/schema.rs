/*!
 * Database schema definitions and migrations.
 *
 * This module contains the SQL schema for all database tables
 * and handles schema migrations for version upgrades.
 */

use anyhow::{Context, Result};
use rusqlite::Connection;
use log::{debug, info};

/// Current schema version
pub const SCHEMA_VERSION: i32 = 1;

/// Initialize the database schema
pub fn initialize_schema(conn: &Connection) -> Result<()> {
    // Check current schema version
    let current_version = get_schema_version(conn)?;

    if current_version == 0 {
        // Fresh database - create all tables
        info!("Initializing database schema v{}", SCHEMA_VERSION);
        create_all_tables(conn)?;
        set_schema_version(conn, SCHEMA_VERSION)?;
    } else if current_version < SCHEMA_VERSION {
        // Need to migrate
        info!(
            "Migrating database schema from v{} to v{}",
            current_version, SCHEMA_VERSION
        );
        migrate_schema(conn, current_version)?;
    } else {
        debug!("Database schema is up to date (v{})", current_version);
    }

    Ok(())
}

/// Get the current schema version from the database
fn get_schema_version(conn: &Connection) -> Result<i32> {
    // Check if the schema_version table exists
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'",
            [],
            |row| row.get(0),
        )
        .context("Failed to check schema_version table existence")?;

    if !table_exists {
        return Ok(0);
    }

    let version: i32 = conn
        .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);

    Ok(version)
}

/// Set the schema version in the database
fn set_schema_version(conn: &Connection, version: i32) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO schema_version (id, version, updated_at) VALUES (1, ?1, datetime('now'))",
        [version],
    )?;
    Ok(())
}

/// Create all database tables
fn create_all_tables(conn: &Connection) -> Result<()> {
    // Enable WAL mode for better concurrency and crash recovery
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;

    // Enable foreign keys
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    // Create schema version table
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS schema_version (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            version INTEGER NOT NULL,
            updated_at TEXT NOT NULL
        );
        "#,
    )?;

    // Create sessions table
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            source_file_path TEXT NOT NULL,
            source_file_hash TEXT NOT NULL,
            source_language TEXT NOT NULL,
            target_language TEXT NOT NULL,
            provider TEXT NOT NULL,
            model TEXT NOT NULL,
            total_entries INTEGER NOT NULL,
            completed_entries INTEGER DEFAULT 0,
            status TEXT DEFAULT 'in_progress',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            completed_at TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
        CREATE INDEX IF NOT EXISTS idx_sessions_source_hash ON sessions(source_file_hash);
        CREATE INDEX IF NOT EXISTS idx_sessions_languages ON sessions(source_language, target_language);
        "#,
    )?;

    // Create source_entries table
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS source_entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
            seq_num INTEGER NOT NULL,
            start_time_ms INTEGER NOT NULL,
            end_time_ms INTEGER NOT NULL,
            source_text TEXT NOT NULL,
            UNIQUE(session_id, seq_num)
        );

        CREATE INDEX IF NOT EXISTS idx_source_entries_session ON source_entries(session_id);
        "#,
    )?;

    // Create translated_entries table
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS translated_entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_entry_id INTEGER NOT NULL REFERENCES source_entries(id) ON DELETE CASCADE,
            translated_text TEXT NOT NULL,
            translation_status TEXT DEFAULT 'pending',
            quality_score REAL,
            validation_errors TEXT,
            attempt_count INTEGER DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(source_entry_id)
        );

        CREATE INDEX IF NOT EXISTS idx_translated_status ON translated_entries(translation_status);
        CREATE INDEX IF NOT EXISTS idx_translated_source ON translated_entries(source_entry_id);
        "#,
    )?;

    // Create translation_cache table
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS translation_cache (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_text_hash TEXT NOT NULL,
            source_text TEXT NOT NULL,
            source_language TEXT NOT NULL,
            target_language TEXT NOT NULL,
            translated_text TEXT NOT NULL,
            provider TEXT NOT NULL,
            model TEXT NOT NULL,
            created_at TEXT NOT NULL,
            hit_count INTEGER DEFAULT 1,
            UNIQUE(source_text_hash, source_language, target_language, provider, model)
        );

        CREATE INDEX IF NOT EXISTS idx_cache_lookup ON translation_cache(source_text_hash, source_language, target_language);
        CREATE INDEX IF NOT EXISTS idx_cache_provider ON translation_cache(provider, model);
        "#,
    )?;

    // Create validation_results table
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS validation_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            translated_entry_id INTEGER NOT NULL REFERENCES translated_entries(id) ON DELETE CASCADE,
            validation_type TEXT NOT NULL,
            passed INTEGER NOT NULL,
            severity TEXT,
            message TEXT,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_validation_entry ON validation_results(translated_entry_id);
        CREATE INDEX IF NOT EXISTS idx_validation_type ON validation_results(validation_type);
        "#,
    )?;

    info!("Database schema created successfully");
    Ok(())
}

/// Migrate the schema from one version to another
fn migrate_schema(conn: &Connection, from_version: i32) -> Result<()> {
    let mut current = from_version;

    while current < SCHEMA_VERSION {
        match current {
            // Add migration steps here as schema evolves
            // Example:
            // 1 => {
            //     migrate_v1_to_v2(conn)?;
            //     current = 2;
            // }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unknown schema version: {}. Cannot migrate.",
                    current
                ));
            }
        }
    }

    set_schema_version(conn, SCHEMA_VERSION)?;
    info!("Schema migration completed to v{}", SCHEMA_VERSION);
    Ok(())
}

/// Drop all tables (for testing purposes only)
#[cfg(test)]
pub fn drop_all_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        DROP TABLE IF EXISTS validation_results;
        DROP TABLE IF EXISTS translated_entries;
        DROP TABLE IF EXISTS source_entries;
        DROP TABLE IF EXISTS sessions;
        DROP TABLE IF EXISTS translation_cache;
        DROP TABLE IF EXISTS schema_version;
        "#,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn create_test_connection() -> Connection {
        Connection::open_in_memory().expect("Failed to create in-memory database")
    }

    #[test]
    fn test_initializeSchema_withFreshDatabase_shouldCreateAllTables() {
        let conn = create_test_connection();

        initialize_schema(&conn).expect("Failed to initialize schema");

        // Verify tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"sessions".to_string()));
        assert!(tables.contains(&"source_entries".to_string()));
        assert!(tables.contains(&"translated_entries".to_string()));
        assert!(tables.contains(&"translation_cache".to_string()));
        assert!(tables.contains(&"validation_results".to_string()));
        assert!(tables.contains(&"schema_version".to_string()));
    }

    #[test]
    fn test_initializeSchema_calledTwice_shouldBeIdempotent() {
        let conn = create_test_connection();

        initialize_schema(&conn).expect("First initialization failed");
        initialize_schema(&conn).expect("Second initialization failed");

        let version = get_schema_version(&conn).expect("Failed to get version");
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn test_getSchemaVersion_withFreshDatabase_shouldReturnZero() {
        let conn = create_test_connection();
        let version = get_schema_version(&conn).expect("Failed to get version");
        assert_eq!(version, 0);
    }

    #[test]
    fn test_setSchemaVersion_shouldPersistVersion() {
        let conn = create_test_connection();

        // Create the schema_version table first
        conn.execute_batch(
            r#"
            CREATE TABLE schema_version (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                version INTEGER NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )
        .unwrap();

        set_schema_version(&conn, 5).expect("Failed to set version");
        let version = get_schema_version(&conn).expect("Failed to get version");
        assert_eq!(version, 5);
    }

    #[test]
    fn test_foreignKeys_shouldBeEnabled() {
        let conn = create_test_connection();
        initialize_schema(&conn).expect("Failed to initialize schema");

        // Insert a session
        conn.execute(
            "INSERT INTO sessions (id, source_file_path, source_file_hash, source_language, target_language, provider, model, total_entries, created_at, updated_at)
             VALUES ('test-session', '/path/to/file', 'hash123', 'en', 'fr', 'ollama', 'llama2', 10, datetime('now'), datetime('now'))",
            [],
        ).expect("Failed to insert session");

        // Try to insert a source entry with invalid session_id (should fail due to foreign key)
        let result = conn.execute(
            "INSERT INTO source_entries (session_id, seq_num, start_time_ms, end_time_ms, source_text)
             VALUES ('nonexistent-session', 1, 0, 1000, 'Hello')",
            [],
        );

        assert!(result.is_err(), "Foreign key constraint should prevent insert");
    }
}
