//! Embedded SQLite migration engine and schema runner.
//!
//! Provides transactional, versioned database schema migrations with automatic tracking,
//! integrity checks, and embedded SQL scripts.
//!
//! # Examples
//!
//! ```
//! use newsjournal_core::storage::{MigrationRunner, open_in_memory};
//!
//! let mut conn = open_in_memory().expect("failed to open database");
//! let runner = MigrationRunner::default();
//! let current_version = runner.current_version(&conn).expect("failed to get version");
//! assert_eq!(current_version, Some(1));
//! ```

use chrono::Utc;
use rusqlite::{params, Connection};

use crate::storage::error::StorageError;

/// A versioned SQL schema migration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Migration {
    /// Monotonically increasing migration version number.
    pub version: i64,
    /// Human-readable descriptive name of the migration.
    pub name: &'static str,
    /// Embedded SQL statements to execute for this migration.
    pub sql: &'static str,
}

/// Record of a migration that has been successfully applied to the database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedMigration {
    /// Migration version number.
    pub version: i64,
    /// Migration descriptive name.
    pub name: String,
    /// UTC timestamp when the migration was applied.
    pub applied_at: String,
}

/// Execution summary returned after running migrations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    /// Number of new migrations applied in this run.
    pub applied_count: usize,
    /// List of migration versions applied in this run.
    pub versions_applied: Vec<i64>,
    /// Schema version prior to running migrations.
    pub initial_version: Option<i64>,
    /// Final schema version after running migrations.
    pub final_version: Option<i64>,
}

/// Standard embedded schema migrations for NewsJournal.
pub const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "0001_initial_schema",
    sql: include_str!("migrations/0001_initial_schema.sql"),
}];

/// Migration runner responsible for verifying and applying schema migrations.
#[derive(Debug, Clone)]
pub struct MigrationRunner {
    migrations: Vec<Migration>,
}

impl Default for MigrationRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl MigrationRunner {
    /// Creates a new `MigrationRunner` loaded with the default embedded migrations.
    #[must_use]
    pub fn new() -> Self {
        Self {
            migrations: MIGRATIONS.to_vec(),
        }
    }

    /// Creates a `MigrationRunner` with a custom list of migrations (useful for testing).
    #[must_use]
    pub fn with_migrations(migrations: Vec<Migration>) -> Self {
        let mut sorted = migrations;
        sorted.sort_by_key(|m| m.version);
        Self { migrations: sorted }
    }

    /// Returns a slice of all migrations registered in this runner.
    #[must_use]
    pub fn migrations(&self) -> &[Migration] {
        &self.migrations
    }

    /// Ensures that the `schema_migrations` metadata table exists in the database.
    pub fn ensure_migration_table(conn: &Connection) -> Result<(), StorageError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                applied_at TEXT NOT NULL
            );",
        )?;
        Ok(())
    }

    /// Retrieves all applied migrations from the database ordered by version ascending.
    pub fn applied_migrations(
        &self,
        conn: &Connection,
    ) -> Result<Vec<AppliedMigration>, StorageError> {
        Self::ensure_migration_table(conn)?;

        let mut stmt = conn.prepare(
            "SELECT version, name, applied_at FROM schema_migrations ORDER BY version ASC;",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(AppliedMigration {
                version: row.get(0)?,
                name: row.get(1)?,
                applied_at: row.get(2)?,
            })
        })?;

        let mut applied = Vec::new();
        for row in rows {
            applied.push(row?);
        }
        Ok(applied)
    }

    /// Returns the current latest applied schema version, or `None` if no migrations have been applied.
    pub fn current_version(&self, conn: &Connection) -> Result<Option<i64>, StorageError> {
        Self::ensure_migration_table(conn)?;

        let mut stmt = conn.prepare("SELECT MAX(version) FROM schema_migrations;")?;
        let version: Option<i64> = stmt.query_row([], |row| row.get(0))?;
        Ok(version)
    }

    /// Returns a list of migrations that have not yet been applied to the database.
    pub fn pending_migrations<'a>(
        &'a self,
        conn: &Connection,
    ) -> Result<Vec<&'a Migration>, StorageError> {
        let applied = self.applied_migrations(conn)?;
        let applied_versions: std::collections::HashSet<i64> =
            applied.iter().map(|m| m.version).collect();

        // Verify history integrity
        for app in &applied {
            if let Some(known) = self.migrations.iter().find(|m| m.version == app.version) {
                if known.name != app.name {
                    return Err(StorageError::MigrationMismatch {
                        version: app.version,
                        expected: known.name.to_string(),
                        found: app.name.clone(),
                    });
                }
            } else {
                return Err(StorageError::CorruptedMigrationHistory { found: app.version });
            }
        }

        let pending = self
            .migrations
            .iter()
            .filter(|m| !applied_versions.contains(&m.version))
            .collect();

        Ok(pending)
    }

    /// Applies all pending migrations sequentially inside isolated database transactions.
    pub fn run(&self, conn: &mut Connection) -> Result<MigrationReport, StorageError> {
        // Enforce foreign key constraints
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        Self::ensure_migration_table(conn)?;

        let initial_version = self.current_version(conn)?;
        let pending = self.pending_migrations(conn)?;

        let mut applied_count = 0;
        let mut versions_applied = Vec::new();

        for migration in pending {
            let tx = conn.transaction()?;

            // Execute migration SQL inside transaction
            if let Err(err) = tx.execute_batch(migration.sql) {
                return Err(StorageError::MigrationFailed {
                    version: migration.version,
                    name: migration.name.to_string(),
                    reason: err.to_string(),
                });
            }

            // Record migration in schema_migrations table
            let now_iso = Utc::now().to_rfc3339();
            tx.execute(
                "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3);",
                params![migration.version, migration.name, now_iso],
            )?;

            tx.commit()?;
            applied_count += 1;
            versions_applied.push(migration.version);
        }

        let final_version = self.current_version(conn)?;

        Ok(MigrationReport {
            applied_count,
            versions_applied,
            initial_version,
            final_version,
        })
    }
}

/// Convenience function to execute all default schema migrations on a connection.
pub fn run_migrations(conn: &mut Connection) -> Result<MigrationReport, StorageError> {
    MigrationRunner::new().run(conn)
}
