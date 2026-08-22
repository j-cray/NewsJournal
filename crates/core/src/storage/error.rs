//! Storage and database error types.

use thiserror::Error;

/// Errors that can occur during database operations and migration execution.
#[derive(Debug, Error)]
pub enum StorageError {
    /// Low-level SQLite database error.
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// A migration with the same version has already been applied with a different name or content.
    #[error(
        "migration version mismatch for version {version}: expected '{expected}', found '{found}'"
    )]
    MigrationMismatch {
        /// Version number of the conflicted migration.
        version: i64,
        /// Expected migration name.
        expected: String,
        /// Found migration name.
        found: String,
    },

    /// Migration execution failure.
    #[error("failed to apply migration {version} ({name}): {reason}")]
    MigrationFailed {
        /// Migration version that failed.
        version: i64,
        /// Migration name that failed.
        name: String,
        /// Detailed error reason.
        reason: String,
    },

    /// An applied migration version was found that is higher than any known migration or out of order.
    #[error("corrupted migration history: version {found} is invalid or out of order")]
    CorruptedMigrationHistory {
        /// The unexpected migration version found.
        found: i64,
    },

    /// Invalid data encountered during serialization or deserialization.
    #[error("storage data error: {0}")]
    InvalidData(String),

    /// Validation failure on entity fields or format constraints.
    #[error("validation error: {0}")]
    Validation(#[from] crate::validation::ValidationError),

    /// Domain model error.
    #[error("model error: {0}")]
    Model(#[from] crate::error::ModelError),

    /// Mutex lock acquisition failure.
    #[error("database lock error: {0}")]
    Lock(String),

    /// Entity not found in the database.
    #[error("entity not found: {0}")]
    NotFound(String),

    /// Conflict on unique constraint (e.g. duplicate slug).
    #[error("conflict error: {0}")]
    Conflict(String),
}

impl StorageError {
    /// Returns `true` if this error represents an entity not found condition.
    #[must_use]
    pub const fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }

    /// Returns `true` if this error represents a unique constraint conflict.
    #[must_use]
    pub const fn is_conflict(&self) -> bool {
        matches!(self, Self::Conflict(_))
    }
}
