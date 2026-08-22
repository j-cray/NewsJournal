//! SQLite connection creation and configuration helpers.

use rusqlite::Connection;

use crate::storage::error::StorageError;
use crate::storage::migration::run_migrations;

/// Configures standard PRAGMA settings on a SQLite connection.
///
/// Sets:
/// - `foreign_keys = ON` (enforces relational cascades and checks)
/// - `busy_timeout = 5000` (5-second timeout for busy locks)
/// - `journal_mode = WAL` (Write-Ahead Logging for improved concurrency where supported)
pub fn configure_connection(conn: &Connection) -> Result<(), StorageError> {
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA busy_timeout = 5000;",
    )?;
    Ok(())
}

/// Opens an in-memory SQLite database configured with pragmas and all migrations applied.
///
/// Ideal for testing, ephemeral state, and headless verification.
pub fn open_in_memory() -> Result<Connection, StorageError> {
    let mut conn = Connection::open_in_memory()?;
    configure_connection(&conn)?;
    run_migrations(&mut conn)?;
    Ok(conn)
}

/// Opens an in-memory SQLite database configured with pragmas but without running migrations.
pub fn open_in_memory_unmigrated() -> Result<Connection, StorageError> {
    let conn = Connection::open_in_memory()?;
    configure_connection(&conn)?;
    Ok(conn)
}
