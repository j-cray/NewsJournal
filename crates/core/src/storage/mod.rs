//! SQLite persistence engine, embedded migrations, and storage services.
//!
//! Provides relational persistence with foreign key integrity, schema versioning,
//! and migration runners.

pub mod connection;
pub mod error;
pub mod migration;

pub use connection::{configure_connection, open_in_memory, open_in_memory_unmigrated};
pub use error::StorageError;
pub use migration::{
    run_migrations, AppliedMigration, Migration, MigrationReport, MigrationRunner, MIGRATIONS,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory_has_schema() {
        let conn = open_in_memory().expect("failed to open in-memory db");
        let runner = MigrationRunner::new();
        let version = runner
            .current_version(&conn)
            .expect("failed to get version");
        assert_eq!(version, Some(1));
    }
}
