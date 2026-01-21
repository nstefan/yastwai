/*!
 * Repository layer for database operations.
 *
 * This module provides a high-level API for all database operations,
 * abstracting away the SQL details and providing type-safe access.
 */

use anyhow::Result;
use log::debug;
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};

use super::connection::DatabaseConnection;
use super::models::{
    CacheRecord, SessionRecord, SessionStatus, SourceEntryRecord, TranslatedEntryRecord,
    TranslationStatus, ValidationResultRecord, ValidationType,
};

/// Repository for database operations
#[derive(Clone)]
pub struct Repository {
    /// Database connection
    db: DatabaseConnection,
}

impl Repository {
    /// Create a new repository with the given database connection
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Create a repository with the default database location
    pub fn new_default() -> Result<Self> {
        let db = DatabaseConnection::new_default()?;
        Ok(Self::new(db))
    }

    /// Create a repository with an in-memory database (for testing)
    pub fn new_in_memory() -> Result<Self> {
        let db = DatabaseConnection::new_in_memory()?;
        Ok(Self::new(db))
    }

    // =========================================================================
    // Session Operations
    // =========================================================================

    /// Create a new translation session
    pub async fn create_session(&self, session: &SessionRecord) -> Result<()> {
        let session = session.clone();

        self.db
            .execute_async(move |conn| {
                conn.execute(
                    r#"
                    INSERT INTO sessions (
                        id, source_file_path, source_file_hash, source_language, target_language,
                        provider, model, total_entries, completed_entries, status,
                        created_at, updated_at, completed_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                    "#,
                    params![
                        session.id,
                        session.source_file_path,
                        session.source_file_hash,
                        session.source_language,
                        session.target_language,
                        session.provider,
                        session.model,
                        session.total_entries,
                        session.completed_entries,
                        session.status.to_string(),
                        session.created_at,
                        session.updated_at,
                        session.completed_at,
                    ],
                )?;
                Ok(())
            })
            .await
    }

    /// Get a session by ID
    pub async fn get_session(&self, session_id: &str) -> Result<Option<SessionRecord>> {
        let session_id = session_id.to_string();

        self.db
            .execute_async(move |conn| {
                Self::get_session_sync(conn, &session_id)
            })
            .await
    }

    /// Get a session by ID (synchronous version for use within transactions)
    fn get_session_sync(conn: &Connection, session_id: &str) -> Result<Option<SessionRecord>> {
        let result = conn
            .query_row(
                r#"
                SELECT id, source_file_path, source_file_hash, source_language, target_language,
                       provider, model, total_entries, completed_entries, status,
                       created_at, updated_at, completed_at
                FROM sessions WHERE id = ?1
                "#,
                [session_id],
                |row| {
                    Ok(SessionRecord {
                        id: row.get(0)?,
                        source_file_path: row.get(1)?,
                        source_file_hash: row.get(2)?,
                        source_language: row.get(3)?,
                        target_language: row.get(4)?,
                        provider: row.get(5)?,
                        model: row.get(6)?,
                        total_entries: row.get(7)?,
                        completed_entries: row.get(8)?,
                        status: row
                            .get::<_, String>(9)?
                            .parse()
                            .unwrap_or(SessionStatus::InProgress),
                        created_at: row.get(10)?,
                        updated_at: row.get(11)?,
                        completed_at: row.get(12)?,
                    })
                },
            )
            .optional()?;

        Ok(result)
    }

    /// Find a resumable session for the given file and language pair
    pub async fn find_resumable_session(
        &self,
        source_file_hash: &str,
        source_language: &str,
        target_language: &str,
        provider: &str,
        model: &str,
    ) -> Result<Option<SessionRecord>> {
        let source_file_hash = source_file_hash.to_string();
        let source_language = source_language.to_string();
        let target_language = target_language.to_string();
        let provider = provider.to_string();
        let model = model.to_string();

        self.db
            .execute_async(move |conn| {
                let result = conn
                    .query_row(
                        r#"
                        SELECT id, source_file_path, source_file_hash, source_language, target_language,
                               provider, model, total_entries, completed_entries, status,
                               created_at, updated_at, completed_at
                        FROM sessions
                        WHERE source_file_hash = ?1
                          AND source_language = ?2
                          AND target_language = ?3
                          AND provider = ?4
                          AND model = ?5
                          AND status IN ('in_progress', 'paused')
                        ORDER BY updated_at DESC
                        LIMIT 1
                        "#,
                        params![source_file_hash, source_language, target_language, provider, model],
                        |row| {
                            Ok(SessionRecord {
                                id: row.get(0)?,
                                source_file_path: row.get(1)?,
                                source_file_hash: row.get(2)?,
                                source_language: row.get(3)?,
                                target_language: row.get(4)?,
                                provider: row.get(5)?,
                                model: row.get(6)?,
                                total_entries: row.get(7)?,
                                completed_entries: row.get(8)?,
                                status: row
                                    .get::<_, String>(9)?
                                    .parse()
                                    .unwrap_or(SessionStatus::InProgress),
                                created_at: row.get(10)?,
                                updated_at: row.get(11)?,
                                completed_at: row.get(12)?,
                            })
                        },
                    )
                    .optional()?;

                Ok(result)
            })
            .await
    }

    /// Update session status
    pub async fn update_session_status(
        &self,
        session_id: &str,
        status: SessionStatus,
    ) -> Result<()> {
        let session_id = session_id.to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.db
            .execute_async(move |conn| {
                let completed_at = if status == SessionStatus::Completed {
                    Some(now.clone())
                } else {
                    None
                };

                conn.execute(
                    r#"
                    UPDATE sessions
                    SET status = ?1, updated_at = ?2, completed_at = COALESCE(?3, completed_at)
                    WHERE id = ?4
                    "#,
                    params![status.to_string(), now, completed_at, session_id],
                )?;
                Ok(())
            })
            .await
    }

    /// Update session progress
    pub async fn update_session_progress(
        &self,
        session_id: &str,
        completed_entries: i64,
    ) -> Result<()> {
        let session_id = session_id.to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.db
            .execute_async(move |conn| {
                conn.execute(
                    "UPDATE sessions SET completed_entries = ?1, updated_at = ?2 WHERE id = ?3",
                    params![completed_entries, now, session_id],
                )?;
                Ok(())
            })
            .await
    }

    /// List all sessions with optional status filter
    pub async fn list_sessions(
        &self,
        status_filter: Option<SessionStatus>,
    ) -> Result<Vec<SessionRecord>> {
        self.db
            .execute_async(move |conn| {
                // Helper function to parse a session row
                fn parse_session_row(row: &rusqlite::Row) -> rusqlite::Result<SessionRecord> {
                    Ok(SessionRecord {
                        id: row.get(0)?,
                        source_file_path: row.get(1)?,
                        source_file_hash: row.get(2)?,
                        source_language: row.get(3)?,
                        target_language: row.get(4)?,
                        provider: row.get(5)?,
                        model: row.get(6)?,
                        total_entries: row.get(7)?,
                        completed_entries: row.get(8)?,
                        status: row
                            .get::<_, String>(9)?
                            .parse()
                            .unwrap_or(SessionStatus::InProgress),
                        created_at: row.get(10)?,
                        updated_at: row.get(11)?,
                        completed_at: row.get(12)?,
                    })
                }

                let sessions: Vec<SessionRecord> = if let Some(status) = status_filter {
                    let mut stmt = conn.prepare(
                        r#"
                        SELECT id, source_file_path, source_file_hash, source_language, target_language,
                               provider, model, total_entries, completed_entries, status,
                               created_at, updated_at, completed_at
                        FROM sessions
                        WHERE status = ?1
                        ORDER BY updated_at DESC
                        "#,
                    )?;
                    stmt.query_map([status.to_string()], parse_session_row)?
                        .filter_map(|r| r.ok())
                        .collect()
                } else {
                    let mut stmt = conn.prepare(
                        r#"
                        SELECT id, source_file_path, source_file_hash, source_language, target_language,
                               provider, model, total_entries, completed_entries, status,
                               created_at, updated_at, completed_at
                        FROM sessions
                        ORDER BY updated_at DESC
                        "#,
                    )?;
                    stmt.query_map([], parse_session_row)?
                        .filter_map(|r| r.ok())
                        .collect()
                };

                Ok(sessions)
            })
            .await
    }

    /// Delete a session and all related data
    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        let session_id = session_id.to_string();

        self.db
            .execute_async(move |conn| {
                // Due to CASCADE, deleting the session will delete related entries
                conn.execute("DELETE FROM sessions WHERE id = ?1", [&session_id])?;
                Ok(())
            })
            .await
    }

    /// Delete sessions older than the specified number of days
    pub async fn delete_old_sessions(&self, days: i64) -> Result<i64> {
        self.db
            .execute_async(move |conn| {
                let deleted = conn.execute(
                    r#"
                    DELETE FROM sessions
                    WHERE created_at < datetime('now', '-' || ?1 || ' days')
                    "#,
                    [days],
                )?;
                Ok(deleted as i64)
            })
            .await
    }

    // =========================================================================
    // Source Entry Operations
    // =========================================================================

    /// Insert source entries for a session (batch insert)
    pub async fn insert_source_entries(&self, entries: Vec<SourceEntryRecord>) -> Result<()> {
        self.db
            .transaction_async(move |tx| {
                for entry in entries {
                    tx.execute(
                        r#"
                        INSERT INTO source_entries (session_id, seq_num, start_time_ms, end_time_ms, source_text)
                        VALUES (?1, ?2, ?3, ?4, ?5)
                        "#,
                        params![
                            entry.session_id,
                            entry.seq_num,
                            entry.start_time_ms,
                            entry.end_time_ms,
                            entry.source_text,
                        ],
                    )?;
                }
                Ok(())
            })
            .await
    }

    /// Get all source entries for a session
    pub async fn get_source_entries(&self, session_id: &str) -> Result<Vec<SourceEntryRecord>> {
        let session_id = session_id.to_string();

        self.db
            .execute_async(move |conn| {
                let mut stmt = conn.prepare(
                    r#"
                    SELECT id, session_id, seq_num, start_time_ms, end_time_ms, source_text
                    FROM source_entries
                    WHERE session_id = ?1
                    ORDER BY seq_num
                    "#,
                )?;

                let rows = stmt.query_map([&session_id], |row| {
                    Ok(SourceEntryRecord {
                        id: row.get(0)?,
                        session_id: row.get(1)?,
                        seq_num: row.get(2)?,
                        start_time_ms: row.get(3)?,
                        end_time_ms: row.get(4)?,
                        source_text: row.get(5)?,
                    })
                })?;

                let entries: Vec<SourceEntryRecord> = rows.filter_map(|r| r.ok()).collect();
                Ok(entries)
            })
            .await
    }

    /// Get pending source entries (not yet translated or marked for retry)
    pub async fn get_pending_entries(&self, session_id: &str) -> Result<Vec<SourceEntryRecord>> {
        let session_id = session_id.to_string();

        self.db
            .execute_async(move |conn| {
                let mut stmt = conn.prepare(
                    r#"
                    SELECT se.id, se.session_id, se.seq_num, se.start_time_ms, se.end_time_ms, se.source_text
                    FROM source_entries se
                    LEFT JOIN translated_entries te ON se.id = te.source_entry_id
                    WHERE se.session_id = ?1
                      AND (te.id IS NULL OR te.translation_status IN ('pending', 'retry', 'failed'))
                    ORDER BY se.seq_num
                    "#,
                )?;

                let rows = stmt.query_map([&session_id], |row| {
                    Ok(SourceEntryRecord {
                        id: row.get(0)?,
                        session_id: row.get(1)?,
                        seq_num: row.get(2)?,
                        start_time_ms: row.get(3)?,
                        end_time_ms: row.get(4)?,
                        source_text: row.get(5)?,
                    })
                })?;

                let entries: Vec<SourceEntryRecord> = rows.filter_map(|r| r.ok()).collect();
                Ok(entries)
            })
            .await
    }

    // =========================================================================
    // Translated Entry Operations
    // =========================================================================

    /// Insert or update a translated entry
    pub async fn upsert_translated_entry(&self, entry: &TranslatedEntryRecord) -> Result<i64> {
        let entry = entry.clone();

        self.db
            .execute_async(move |conn| {
                conn.execute(
                    r#"
                    INSERT INTO translated_entries (
                        source_entry_id, translated_text, translation_status,
                        quality_score, validation_errors, attempt_count, created_at, updated_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                    ON CONFLICT(source_entry_id) DO UPDATE SET
                        translated_text = excluded.translated_text,
                        translation_status = excluded.translation_status,
                        quality_score = excluded.quality_score,
                        validation_errors = excluded.validation_errors,
                        attempt_count = translated_entries.attempt_count + 1,
                        updated_at = excluded.updated_at
                    "#,
                    params![
                        entry.source_entry_id,
                        entry.translated_text,
                        entry.translation_status.to_string(),
                        entry.quality_score,
                        entry.validation_errors,
                        entry.attempt_count,
                        entry.created_at,
                        entry.updated_at,
                    ],
                )?;

                Ok(conn.last_insert_rowid())
            })
            .await
    }

    /// Batch insert translated entries
    pub async fn insert_translated_entries(&self, entries: Vec<TranslatedEntryRecord>) -> Result<()> {
        self.db
            .transaction_async(move |tx| {
                for entry in entries {
                    tx.execute(
                        r#"
                        INSERT INTO translated_entries (
                            source_entry_id, translated_text, translation_status,
                            quality_score, validation_errors, attempt_count, created_at, updated_at
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                        ON CONFLICT(source_entry_id) DO UPDATE SET
                            translated_text = excluded.translated_text,
                            translation_status = excluded.translation_status,
                            quality_score = excluded.quality_score,
                            validation_errors = excluded.validation_errors,
                            attempt_count = translated_entries.attempt_count + 1,
                            updated_at = excluded.updated_at
                        "#,
                        params![
                            entry.source_entry_id,
                            entry.translated_text,
                            entry.translation_status.to_string(),
                            entry.quality_score,
                            entry.validation_errors,
                            entry.attempt_count,
                            entry.created_at,
                            entry.updated_at,
                        ],
                    )?;
                }
                Ok(())
            })
            .await
    }

    /// Get all translated entries for a session
    pub async fn get_translated_entries(
        &self,
        session_id: &str,
    ) -> Result<Vec<(SourceEntryRecord, TranslatedEntryRecord)>> {
        let session_id = session_id.to_string();

        self.db
            .execute_async(move |conn| {
                let mut stmt = conn.prepare(
                    r#"
                    SELECT 
                        se.id, se.session_id, se.seq_num, se.start_time_ms, se.end_time_ms, se.source_text,
                        te.id, te.source_entry_id, te.translated_text, te.translation_status,
                        te.quality_score, te.validation_errors, te.attempt_count, te.created_at, te.updated_at
                    FROM source_entries se
                    INNER JOIN translated_entries te ON se.id = te.source_entry_id
                    WHERE se.session_id = ?1
                    ORDER BY se.seq_num
                    "#,
                )?;

                let rows = stmt.query_map([&session_id], |row| {
                    Ok((
                        SourceEntryRecord {
                            id: row.get(0)?,
                            session_id: row.get(1)?,
                            seq_num: row.get(2)?,
                            start_time_ms: row.get(3)?,
                            end_time_ms: row.get(4)?,
                            source_text: row.get(5)?,
                        },
                        TranslatedEntryRecord {
                            id: row.get(6)?,
                            source_entry_id: row.get(7)?,
                            translated_text: row.get(8)?,
                            translation_status: row
                                .get::<_, String>(9)?
                                .parse()
                                .unwrap_or(TranslationStatus::Pending),
                            quality_score: row.get(10)?,
                            validation_errors: row.get(11)?,
                            attempt_count: row.get(12)?,
                            created_at: row.get(13)?,
                            updated_at: row.get(14)?,
                        },
                    ))
                })?;

                let entries: Vec<_> = rows.filter_map(|r| r.ok()).collect();
                Ok(entries)
            })
            .await
    }

    /// Update translation status for an entry
    pub async fn update_translation_status(
        &self,
        translated_entry_id: i64,
        status: TranslationStatus,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        self.db
            .execute_async(move |conn| {
                conn.execute(
                    "UPDATE translated_entries SET translation_status = ?1, updated_at = ?2 WHERE id = ?3",
                    params![status.to_string(), now, translated_entry_id],
                )?;
                Ok(())
            })
            .await
    }

    // =========================================================================
    // Cache Operations
    // =========================================================================

    /// Compute SHA256 hash of text
    pub fn hash_text(text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Get a cached translation
    pub async fn get_cached_translation(
        &self,
        source_text: &str,
        source_language: &str,
        target_language: &str,
        provider: &str,
        model: &str,
    ) -> Result<Option<String>> {
        let source_text_hash = Self::hash_text(source_text);
        let source_language = source_language.to_string();
        let target_language = target_language.to_string();
        let provider = provider.to_string();
        let model = model.to_string();

        self.db
            .execute_async(move |conn| {
                let result: Option<(i64, String)> = conn
                    .query_row(
                        r#"
                        SELECT id, translated_text
                        FROM translation_cache
                        WHERE source_text_hash = ?1
                          AND source_language = ?2
                          AND target_language = ?3
                          AND provider = ?4
                          AND model = ?5
                        "#,
                        params![source_text_hash, source_language, target_language, provider, model],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .optional()?;

                if let Some((id, translated_text)) = result {
                    // Increment hit count
                    conn.execute(
                        "UPDATE translation_cache SET hit_count = hit_count + 1 WHERE id = ?1",
                        [id],
                    )?;
                    debug!("Cache hit for translation");
                    Ok(Some(translated_text))
                } else {
                    Ok(None)
                }
            })
            .await
    }

    /// Store a translation in the cache
    pub async fn cache_translation(&self, record: &CacheRecord) -> Result<()> {
        let record = record.clone();

        self.db
            .execute_async(move |conn| {
                conn.execute(
                    r#"
                    INSERT INTO translation_cache (
                        source_text_hash, source_text, source_language, target_language,
                        translated_text, provider, model, created_at, hit_count
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                    ON CONFLICT(source_text_hash, source_language, target_language, provider, model)
                    DO UPDATE SET hit_count = translation_cache.hit_count + 1
                    "#,
                    params![
                        record.source_text_hash,
                        record.source_text,
                        record.source_language,
                        record.target_language,
                        record.translated_text,
                        record.provider,
                        record.model,
                        record.created_at,
                        record.hit_count,
                    ],
                )?;
                Ok(())
            })
            .await
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> Result<CacheStats> {
        self.db
            .execute_async(|conn| {
                let total_entries: i64 = conn
                    .query_row("SELECT COUNT(*) FROM translation_cache", [], |row| row.get(0))
                    .unwrap_or(0);

                let total_hits: i64 = conn
                    .query_row(
                        "SELECT COALESCE(SUM(hit_count), 0) FROM translation_cache",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);

                Ok(CacheStats {
                    total_entries,
                    total_hits,
                })
            })
            .await
    }

    /// Clear the translation cache
    pub async fn clear_cache(&self) -> Result<i64> {
        self.db
            .execute_async(|conn| {
                let deleted = conn.execute("DELETE FROM translation_cache", [])?;
                Ok(deleted as i64)
            })
            .await
    }

    /// Get recent cache entries for a language pair (for cache warming)
    ///
    /// Returns the most frequently used entries for the given language pair,
    /// limited to the specified count.
    pub async fn get_recent_cache_entries(
        &self,
        source_language: &str,
        target_language: &str,
        provider: &str,
        model: &str,
        limit: usize,
    ) -> Result<Vec<CacheRecord>> {
        let source_language = source_language.to_string();
        let target_language = target_language.to_string();
        let provider = provider.to_string();
        let model = model.to_string();

        self.db
            .execute_async(move |conn| {
                let mut stmt = conn.prepare(
                    r#"
                    SELECT id, source_text_hash, source_text, source_language, target_language,
                           translated_text, provider, model, created_at, hit_count
                    FROM translation_cache
                    WHERE source_language = ?1
                      AND target_language = ?2
                      AND provider = ?3
                      AND model = ?4
                    ORDER BY hit_count DESC, created_at DESC
                    LIMIT ?5
                    "#,
                )?;

                let records = stmt
                    .query_map(
                        params![source_language, target_language, provider, model, limit as i64],
                        |row| {
                            Ok(CacheRecord {
                                id: row.get(0)?,
                                source_text_hash: row.get(1)?,
                                source_text: row.get(2)?,
                                source_language: row.get(3)?,
                                target_language: row.get(4)?,
                                translated_text: row.get(5)?,
                                provider: row.get(6)?,
                                model: row.get(7)?,
                                created_at: row.get(8)?,
                                hit_count: row.get(9)?,
                            })
                        },
                    )?
                    .filter_map(|r| r.ok())
                    .collect();

                Ok(records)
            })
            .await
    }

    // =========================================================================
    // Validation Result Operations
    // =========================================================================

    /// Insert validation results for a translated entry
    pub async fn insert_validation_results(&self, results: Vec<ValidationResultRecord>) -> Result<()> {
        self.db
            .transaction_async(move |tx| {
                for result in results {
                    tx.execute(
                        r#"
                        INSERT INTO validation_results (
                            translated_entry_id, validation_type, passed, severity, message, created_at
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                        "#,
                        params![
                            result.translated_entry_id,
                            result.validation_type.to_string(),
                            result.passed as i32,
                            result.severity.map(|s| s.to_string()),
                            result.message,
                            result.created_at,
                        ],
                    )?;
                }
                Ok(())
            })
            .await
    }

    /// Get validation results for a translated entry
    pub async fn get_validation_results(
        &self,
        translated_entry_id: i64,
    ) -> Result<Vec<ValidationResultRecord>> {
        self.db
            .execute_async(move |conn| {
                let mut stmt = conn.prepare(
                    r#"
                    SELECT id, translated_entry_id, validation_type, passed, severity, message, created_at
                    FROM validation_results
                    WHERE translated_entry_id = ?1
                    "#,
                )?;

                let rows = stmt.query_map([translated_entry_id], |row| {
                    Ok(ValidationResultRecord {
                        id: row.get(0)?,
                        translated_entry_id: row.get(1)?,
                        validation_type: row
                            .get::<_, String>(2)?
                            .parse()
                            .unwrap_or(ValidationType::MarkerCheck),
                        passed: row.get::<_, i32>(3)? != 0,
                        severity: row
                            .get::<_, Option<String>>(4)?
                            .and_then(|s| s.parse().ok()),
                        message: row.get(5)?,
                        created_at: row.get(6)?,
                    })
                })?;

                let results: Vec<ValidationResultRecord> = rows.filter_map(|r| r.ok()).collect();
                Ok(results)
            })
            .await
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of cache entries
    pub total_entries: i64,
    /// Total number of cache hits
    pub total_hits: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_repo() -> Repository {
        Repository::new_in_memory().expect("Failed to create test repository")
    }

    #[tokio::test]
    async fn test_createSession_shouldInsertSession() {
        let repo = create_test_repo().await;

        let session = SessionRecord::new(
            "test-session-1".to_string(),
            "/path/to/video.mkv".to_string(),
            "abc123hash".to_string(),
            "en".to_string(),
            "fr".to_string(),
            "ollama".to_string(),
            "llama2".to_string(),
            100,
        );

        repo.create_session(&session).await.expect("Failed to create session");

        let retrieved = repo
            .get_session("test-session-1")
            .await
            .expect("Failed to get session");

        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, "test-session-1");
        assert_eq!(retrieved.source_language, "en");
        assert_eq!(retrieved.target_language, "fr");
        assert_eq!(retrieved.total_entries, 100);
    }

    #[tokio::test]
    async fn test_findResumableSession_shouldFindMatchingSession() {
        let repo = create_test_repo().await;

        let session = SessionRecord::new(
            "resumable-session".to_string(),
            "/path/to/video.mkv".to_string(),
            "file-hash-123".to_string(),
            "en".to_string(),
            "fr".to_string(),
            "ollama".to_string(),
            "llama2".to_string(),
            50,
        );

        repo.create_session(&session).await.expect("Failed to create session");

        let found = repo
            .find_resumable_session("file-hash-123", "en", "fr", "ollama", "llama2")
            .await
            .expect("Failed to find session");

        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "resumable-session");
    }

    #[tokio::test]
    async fn test_insertSourceEntries_shouldInsertAll() {
        let repo = create_test_repo().await;

        // Create session first
        let session = SessionRecord::new(
            "entry-test-session".to_string(),
            "/path/to/video.mkv".to_string(),
            "hash".to_string(),
            "en".to_string(),
            "fr".to_string(),
            "ollama".to_string(),
            "llama2".to_string(),
            3,
        );
        repo.create_session(&session).await.unwrap();

        // Insert entries
        let entries = vec![
            SourceEntryRecord::new("entry-test-session".to_string(), 1, 0, 1000, "Hello".to_string()),
            SourceEntryRecord::new("entry-test-session".to_string(), 2, 1000, 2000, "World".to_string()),
            SourceEntryRecord::new("entry-test-session".to_string(), 3, 2000, 3000, "Test".to_string()),
        ];

        repo.insert_source_entries(entries).await.expect("Failed to insert entries");

        let retrieved = repo
            .get_source_entries("entry-test-session")
            .await
            .expect("Failed to get entries");

        assert_eq!(retrieved.len(), 3);
        assert_eq!(retrieved[0].source_text, "Hello");
        assert_eq!(retrieved[1].source_text, "World");
        assert_eq!(retrieved[2].source_text, "Test");
    }

    #[tokio::test]
    async fn test_cacheTranslation_shouldStoreAndRetrieve() {
        let repo = create_test_repo().await;

        let cache_record = CacheRecord::new(
            Repository::hash_text("Hello"),
            "Hello".to_string(),
            "en".to_string(),
            "fr".to_string(),
            "Bonjour".to_string(),
            "ollama".to_string(),
            "llama2".to_string(),
        );

        repo.cache_translation(&cache_record).await.expect("Failed to cache");

        let cached = repo
            .get_cached_translation("Hello", "en", "fr", "ollama", "llama2")
            .await
            .expect("Failed to get cached");

        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), "Bonjour");
    }

    #[tokio::test]
    async fn test_getCachedTranslation_shouldIncrementHitCount() {
        let repo = create_test_repo().await;

        let cache_record = CacheRecord::new(
            Repository::hash_text("Test"),
            "Test".to_string(),
            "en".to_string(),
            "fr".to_string(),
            "Essai".to_string(),
            "ollama".to_string(),
            "llama2".to_string(),
        );

        repo.cache_translation(&cache_record).await.unwrap();

        // Hit the cache multiple times
        repo.get_cached_translation("Test", "en", "fr", "ollama", "llama2").await.unwrap();
        repo.get_cached_translation("Test", "en", "fr", "ollama", "llama2").await.unwrap();
        repo.get_cached_translation("Test", "en", "fr", "ollama", "llama2").await.unwrap();

        let stats = repo.get_cache_stats().await.unwrap();
        assert_eq!(stats.total_entries, 1);
        assert!(stats.total_hits >= 3); // Initial + 3 reads
    }

    #[test]
    fn test_hashText_shouldProduceConsistentHash() {
        let hash1 = Repository::hash_text("Hello, World!");
        let hash2 = Repository::hash_text("Hello, World!");
        let hash3 = Repository::hash_text("Different text");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64); // SHA256 produces 64 hex chars
    }

    #[tokio::test]
    async fn test_insertTranslatedEntries_shouldStoreTranslations() {
        let repo = create_test_repo().await;

        // Create session and source entries first
        let session = SessionRecord::new(
            "translated-test".to_string(),
            "/path/to/video.mkv".to_string(),
            "hash".to_string(),
            "en".to_string(),
            "fr".to_string(),
            "ollama".to_string(),
            "llama2".to_string(),
            2,
        );
        repo.create_session(&session).await.unwrap();

        let entries = vec![
            SourceEntryRecord::new("translated-test".to_string(), 1, 0, 1000, "Hello".to_string()),
            SourceEntryRecord::new("translated-test".to_string(), 2, 1000, 2000, "World".to_string()),
        ];
        repo.insert_source_entries(entries).await.unwrap();

        // Get source entries to get their IDs
        let source_entries = repo.get_source_entries("translated-test").await.unwrap();
        assert_eq!(source_entries.len(), 2);

        // Insert translated entries
        let translated_entries = vec![
            TranslatedEntryRecord::new(source_entries[0].id, "Bonjour".to_string()),
            TranslatedEntryRecord::new(source_entries[1].id, "Monde".to_string()),
        ];
        repo.insert_translated_entries(translated_entries).await.unwrap();

        // Retrieve and verify
        let retrieved = repo.get_translated_entries("translated-test").await.unwrap();
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].1.translated_text, "Bonjour");
        assert_eq!(retrieved[1].1.translated_text, "Monde");
    }

    #[tokio::test]
    async fn test_getPendingEntries_shouldReturnUntranslated() {
        let repo = create_test_repo().await;

        // Create session and source entries
        let session = SessionRecord::new(
            "pending-test".to_string(),
            "/path/to/video.mkv".to_string(),
            "hash".to_string(),
            "en".to_string(),
            "fr".to_string(),
            "ollama".to_string(),
            "llama2".to_string(),
            3,
        );
        repo.create_session(&session).await.unwrap();

        let entries = vec![
            SourceEntryRecord::new("pending-test".to_string(), 1, 0, 1000, "One".to_string()),
            SourceEntryRecord::new("pending-test".to_string(), 2, 1000, 2000, "Two".to_string()),
            SourceEntryRecord::new("pending-test".to_string(), 3, 2000, 3000, "Three".to_string()),
        ];
        repo.insert_source_entries(entries).await.unwrap();

        // Initially all should be pending
        let pending = repo.get_pending_entries("pending-test").await.unwrap();
        assert_eq!(pending.len(), 3);

        // Translate one entry
        let source_entries = repo.get_source_entries("pending-test").await.unwrap();
        let translated = vec![TranslatedEntryRecord::new(
            source_entries[0].id,
            "Un".to_string(),
        )];
        repo.insert_translated_entries(translated).await.unwrap();

        // Now only 2 should be pending
        let pending = repo.get_pending_entries("pending-test").await.unwrap();
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].source_text, "Two");
        assert_eq!(pending[1].source_text, "Three");
    }

    #[tokio::test]
    async fn test_updateSessionStatus_shouldChangeStatus() {
        let repo = create_test_repo().await;

        let session = SessionRecord::new(
            "status-test".to_string(),
            "/path/to/video.mkv".to_string(),
            "hash".to_string(),
            "en".to_string(),
            "fr".to_string(),
            "ollama".to_string(),
            "llama2".to_string(),
            10,
        );
        repo.create_session(&session).await.unwrap();

        // Update to completed
        repo.update_session_status("status-test", SessionStatus::Completed)
            .await
            .unwrap();

        let updated = repo.get_session("status-test").await.unwrap().unwrap();
        assert_eq!(updated.status, SessionStatus::Completed);
        assert!(updated.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_deleteSession_shouldRemoveSessionAndEntries() {
        let repo = create_test_repo().await;

        let session = SessionRecord::new(
            "delete-test".to_string(),
            "/path/to/video.mkv".to_string(),
            "hash".to_string(),
            "en".to_string(),
            "fr".to_string(),
            "ollama".to_string(),
            "llama2".to_string(),
            2,
        );
        repo.create_session(&session).await.unwrap();

        let entries = vec![
            SourceEntryRecord::new("delete-test".to_string(), 1, 0, 1000, "Hello".to_string()),
        ];
        repo.insert_source_entries(entries).await.unwrap();

        // Delete session
        repo.delete_session("delete-test").await.unwrap();

        // Verify session is gone
        let result = repo.get_session("delete-test").await.unwrap();
        assert!(result.is_none());

        // Source entries should also be gone (cascade delete)
        let entries = repo.get_source_entries("delete-test").await.unwrap();
        assert!(entries.is_empty());
    }
}
