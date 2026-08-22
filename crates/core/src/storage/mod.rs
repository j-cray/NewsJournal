//! SQLite persistence engine, embedded migrations, and storage services.
//!
//! Provides relational persistence with foreign key integrity, schema versioning,
//! and migration runners.

pub mod article_contacts;
pub mod articles;
pub mod connection;
pub mod contacts;
pub mod error;
pub mod migration;
pub mod paths;
pub mod service;
pub mod settings;
pub mod tasks;

pub use connection::{configure_connection, open_file, open_in_memory, open_in_memory_unmigrated};
pub use error::StorageError;
pub use migration::{
    run_migrations, AppliedMigration, Migration, MigrationReport, MigrationRunner, MIGRATIONS,
};
pub use paths::{
    AppPaths, APPLICATION, DEFAULT_DB_FILENAME, ENV_CACHE_DIR, ENV_CONFIG_DIR, ENV_DATA_DIR,
    ENV_DB_PATH, ENV_STATE_DIR, ORGANIZATION, QUALIFIER,
};
pub use service::StorageService;

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

    #[test]
    fn test_storage_service_in_memory_initialization() {
        let service = StorageService::in_memory().expect("failed to initialize storage service");
        let settings = service
            .get_settings()
            .expect("failed to get default settings");
        assert_eq!(settings.theme_mode, crate::models::ThemeMode::System);
    }
}
