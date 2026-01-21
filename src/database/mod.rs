/*!
 * Database module for persistent storage of translations and sessions.
 *
 * This module provides SQLite-based persistence for:
 * - Translation sessions with resume capability
 * - Translation cache for cross-session deduplication
 * - Quality validation results
 *
 * # Architecture
 *
 * - `schema`: Database schema definitions and migrations
 * - `connection`: Connection pool management
 * - `repository`: Data access layer for all database operations
 * - `models`: Database entity models and DTOs
 */

pub mod schema;
pub mod connection;
pub mod repository;
pub mod models;

// Re-export main types (public API, may not be used internally)
#[allow(unused_imports)]
pub use connection::DatabaseConnection;
#[allow(unused_imports)]
pub use repository::Repository;
