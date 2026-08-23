//! Unified thread-safe persistence and domain repository service.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{
    Article, ArticleContact, ArticleStage, Contact, Settings, Task, TaskStatus, ThemeMode,
};
use crate::storage::article_contacts;
use crate::storage::articles;
use crate::storage::connection::{open_file, open_in_memory};
use crate::storage::contacts;
use crate::storage::error::StorageError;
use crate::storage::paths::AppPaths;
use crate::storage::settings;
use crate::storage::tasks;

/// Summary metrics of entities stored in the SQLite database.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DatabaseEntityCounts {
    /// Total number of stored articles.
    pub total_articles: usize,
    /// Total number of stored tasks.
    pub total_tasks: usize,
    /// Total number of stored contacts.
    pub total_contacts: usize,
    /// Total number of article-contact links.
    pub total_article_contacts: usize,
}

/// Thread-safe storage service providing unified CRUD and relational repository operations.
///
/// Wraps an underlying SQLite database connection protected by a mutex, allowing safe
/// concurrent cloning and usage across UI threads and background evaluators.
///
/// # Examples
///
/// ```
/// use newsjournal_core::storage::StorageService;
/// use newsjournal_core::{Article, Task, Contact, ArticleStage};
///
/// let service = StorageService::in_memory().expect("failed to init db");
///
/// let article = Article::new("breaking-news", "Major City Story")
///     .with_stage(ArticleStage::Researching);
/// let created = service.create_article(article).expect("failed to create article");
///
/// assert_eq!(created.slug, "breaking-news");
/// ```
#[derive(Debug, Clone)]
pub struct StorageService {
    conn: Arc<Mutex<Connection>>,
    db_path: Option<PathBuf>,
}

impl StorageService {
    /// Creates a new `StorageService` taking ownership of an already opened and migrated connection.
    #[must_use]
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: None,
        }
    }

    /// Opens an in-memory SQLite database, configures pragmas, applies all migrations, and returns the service.
    pub fn in_memory() -> Result<Self, StorageError> {
        let conn = open_in_memory()?;
        Ok(Self::new(conn))
    }

    /// Opens a file-backed SQLite database at the specified path, applies migrations, and returns the service.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path_buf = path.as_ref().to_path_buf();
        let conn = open_file(&path_buf)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: Some(path_buf),
        })
    }

    /// Opens the default application SQLite database based on standard OS directory conventions
    /// and active environment overrides (`NEWSJOURNAL_*`).
    ///
    /// Automatically ensures parent directories exist and runs all pending schema migrations.
    pub fn open_default() -> Result<Self, StorageError> {
        let paths = AppPaths::resolve()?;
        Self::open_with_paths(&paths)
    }

    /// Opens the SQLite database configured by the specified `AppPaths`.
    ///
    /// Automatically ensures parent directories exist and runs all pending schema migrations.
    pub fn open_with_paths(paths: &AppPaths) -> Result<Self, StorageError> {
        paths.ensure_data_dir()?;
        let db_path = paths.database_path();
        Self::open(db_path)
    }

    /// Resolves the default SQLite database path according to operating system conventions
    /// and active environment variables.
    pub fn default_database_path() -> Result<PathBuf, StorageError> {
        let paths = AppPaths::resolve()?;
        Ok(paths.database_path())
    }

    /// Creates a `StorageService` from an existing shared `Arc<Mutex<Connection>>`.
    #[must_use]
    pub fn with_shared(conn: Arc<Mutex<Connection>>) -> Self {
        Self {
            conn,
            db_path: None,
        }
    }

    /// Creates a `StorageService` from an existing shared `Arc<Mutex<Connection>>` with an explicit database path.
    #[must_use]
    pub fn with_shared_and_path(conn: Arc<Mutex<Connection>>, db_path: Option<PathBuf>) -> Self {
        Self { conn, db_path }
    }

    /// Returns the path to the database file if file-backed, or `None` if in-memory.
    #[must_use]
    pub fn database_path(&self) -> Option<&Path> {
        self.db_path.as_deref()
    }

    /// Returns whether this storage service is backed by an in-memory database.
    #[must_use]
    pub fn is_in_memory(&self) -> bool {
        self.db_path.is_none()
    }

    /// Returns the size of the database file on disk in bytes, or `None` if in-memory or file does not exist.
    pub fn database_file_size(&self) -> Result<Option<u64>, StorageError> {
        if let Some(path) = &self.db_path {
            match std::fs::metadata(path) {
                Ok(metadata) => Ok(Some(metadata.len())),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(StorageError::Io(e)),
            }
        } else {
            Ok(None)
        }
    }

    /// Returns the current schema migration version applied to the database.
    pub fn current_schema_version(&self) -> Result<Option<i64>, StorageError> {
        let conn = self.lock()?;
        let runner = crate::storage::migration::MigrationRunner::new();
        runner.current_version(&conn)
    }

    /// Returns the total count of applied schema migrations.
    pub fn applied_migrations_count(&self) -> Result<usize, StorageError> {
        let conn = self.lock()?;
        let runner = crate::storage::migration::MigrationRunner::new();
        let applied = runner.applied_migrations(&conn)?;
        Ok(applied.len())
    }

    /// Retrieves entity count metrics across all tables in the database.
    pub fn entity_counts(&self) -> Result<DatabaseEntityCounts, StorageError> {
        let conn = self.lock()?;
        let total_articles: usize =
            conn.query_row("SELECT count(*) FROM articles", [], |r| r.get(0))?;
        let total_tasks: usize = conn.query_row("SELECT count(*) FROM tasks", [], |r| r.get(0))?;
        let total_contacts: usize =
            conn.query_row("SELECT count(*) FROM contacts", [], |r| r.get(0))?;
        let total_article_contacts: usize =
            conn.query_row("SELECT count(*) FROM article_contacts", [], |r| r.get(0))?;
        Ok(DatabaseEntityCounts {
            total_articles,
            total_tasks,
            total_contacts,
            total_article_contacts,
        })
    }

    /// Returns the underlying SQLite library version.
    #[must_use]
    pub fn sqlite_version() -> &'static str {
        rusqlite::version()
    }

    /// Returns a cloned `Arc<Mutex<Connection>>` for shared low-level access.
    #[must_use]
    pub fn shared_connection(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }

    /// Acquires the lock on the internal SQLite connection.
    fn lock(&self) -> Result<MutexGuard<'_, Connection>, StorageError> {
        self.conn
            .lock()
            .map_err(|e| StorageError::Lock(format!("database mutex poisoned: {e}")))
    }

    // ==========================================
    // Article Operations
    // ==========================================

    /// Persists a new article record into the database.
    ///
    /// Validates all field constraints. Returns [`StorageError::Conflict`] if the slug or ID already exists.
    pub fn create_article(&self, article: Article) -> Result<Article, StorageError> {
        let conn = self.lock()?;
        articles::insert_article(&conn, &article)?;
        Ok(article)
    }

    /// Retrieves an article by its unique UUID.
    pub fn get_article(&self, id: Uuid) -> Result<Option<Article>, StorageError> {
        let conn = self.lock()?;
        articles::get_article_by_id(&conn, id)
    }

    /// Retrieves an article by its unique slug.
    pub fn get_article_by_slug(&self, slug: &str) -> Result<Option<Article>, StorageError> {
        let conn = self.lock()?;
        articles::get_article_by_slug(&conn, slug)
    }

    /// Updates an existing article and validates fields.
    pub fn update_article(&self, mut article: Article) -> Result<Article, StorageError> {
        article.touch();
        let conn = self.lock()?;
        articles::update_article(&conn, &article)?;
        Ok(article)
    }

    /// Deletes an article by ID. Associated tasks and contact links are cascade-deleted.
    pub fn delete_article(&self, id: Uuid) -> Result<bool, StorageError> {
        let conn = self.lock()?;
        articles::delete_article(&conn, id)
    }

    /// Lists all articles ordered by created_at descending.
    pub fn list_articles(&self) -> Result<Vec<Article>, StorageError> {
        let conn = self.lock()?;
        articles::list_articles(&conn)
    }

    /// Lists all articles belonging to a specific workflow stage.
    pub fn list_articles_by_stage(
        &self,
        stage: ArticleStage,
    ) -> Result<Vec<Article>, StorageError> {
        let conn = self.lock()?;
        articles::list_articles_by_stage(&conn, stage)
    }

    /// Updates the editorial stage of an article.
    pub fn change_article_stage(
        &self,
        id: Uuid,
        stage: ArticleStage,
    ) -> Result<Article, StorageError> {
        let conn = self.lock()?;
        articles::change_article_stage(&conn, id, stage)
    }

    /// Checks if a slug is available for a new or existing article.
    pub fn is_slug_available(
        &self,
        slug: &str,
        exclude_id: Option<Uuid>,
    ) -> Result<bool, StorageError> {
        let conn = self.lock()?;
        articles::is_slug_available(&conn, slug, exclude_id)
    }

    // ==========================================
    // Task Operations
    // ==========================================

    /// Persists a new task associated with an existing parent article.
    pub fn create_task(&self, task: Task) -> Result<Task, StorageError> {
        let conn = self.lock()?;
        tasks::insert_task(&conn, &task)?;
        Ok(task)
    }

    /// Retrieves a task by its unique UUID.
    pub fn get_task(&self, id: Uuid) -> Result<Option<Task>, StorageError> {
        let conn = self.lock()?;
        tasks::get_task_by_id(&conn, id)
    }

    /// Updates an existing task and validates fields.
    pub fn update_task(&self, mut task: Task) -> Result<Task, StorageError> {
        task.touch();
        let conn = self.lock()?;
        tasks::update_task(&conn, &task)?;
        Ok(task)
    }

    /// Deletes a task by ID.
    pub fn delete_task(&self, id: Uuid) -> Result<bool, StorageError> {
        let conn = self.lock()?;
        tasks::delete_task(&conn, id)
    }

    /// Lists all tasks across all articles.
    pub fn list_tasks(&self) -> Result<Vec<Task>, StorageError> {
        let conn = self.lock()?;
        tasks::list_tasks(&conn)
    }

    /// Lists all tasks belonging to a specific parent article.
    pub fn list_tasks_for_article(&self, article_id: Uuid) -> Result<Vec<Task>, StorageError> {
        let conn = self.lock()?;
        tasks::list_tasks_for_article(&conn, article_id)
    }

    /// Lists all tasks filtered by status.
    pub fn list_tasks_by_status(&self, status: TaskStatus) -> Result<Vec<Task>, StorageError> {
        let conn = self.lock()?;
        tasks::list_tasks_by_status(&conn, status)
    }

    /// Updates the workflow status of a task.
    pub fn change_task_status(&self, id: Uuid, status: TaskStatus) -> Result<Task, StorageError> {
        let conn = self.lock()?;
        tasks::change_task_status(&conn, id, status)
    }

    /// Computes task count summary for an article as `(total_count, completed_count)`.
    pub fn article_task_counts(&self, article_id: Uuid) -> Result<(usize, usize), StorageError> {
        let conn = self.lock()?;
        tasks::count_tasks_for_article(&conn, article_id)
    }

    // ==========================================
    // Contact Operations
    // ==========================================

    /// Persists a new contact record into the database.
    pub fn create_contact(&self, contact: Contact) -> Result<Contact, StorageError> {
        let conn = self.lock()?;
        contacts::insert_contact(&conn, &contact)?;
        Ok(contact)
    }

    /// Retrieves a contact by its unique UUID.
    pub fn get_contact(&self, id: Uuid) -> Result<Option<Contact>, StorageError> {
        let conn = self.lock()?;
        contacts::get_contact_by_id(&conn, id)
    }

    /// Updates an existing contact record and validates fields.
    pub fn update_contact(&self, mut contact: Contact) -> Result<Contact, StorageError> {
        contact.touch();
        let conn = self.lock()?;
        contacts::update_contact(&conn, &contact)?;
        Ok(contact)
    }

    /// Deletes a contact by ID. Associated article tagging links are cascade-deleted.
    pub fn delete_contact(&self, id: Uuid) -> Result<bool, StorageError> {
        let conn = self.lock()?;
        contacts::delete_contact(&conn, id)
    }

    /// Lists all contacts alphabetically by name.
    pub fn list_contacts(&self) -> Result<Vec<Contact>, StorageError> {
        let conn = self.lock()?;
        contacts::list_contacts(&conn)
    }

    /// Searches contacts matching a query string across name, organization, role, email, phone, and notes.
    pub fn search_contacts(&self, query: &str) -> Result<Vec<Contact>, StorageError> {
        let conn = self.lock()?;
        contacts::search_contacts(&conn, query)
    }

    // ==========================================
    // Article-Contact Tagging Operations
    // ==========================================

    /// Links a contact to an article.
    pub fn link_contact_to_article(
        &self,
        article_id: Uuid,
        contact_id: Uuid,
    ) -> Result<ArticleContact, StorageError> {
        let conn = self.lock()?;
        article_contacts::link_contact_to_article(&conn, article_id, contact_id)
    }

    /// Unlinks a contact from an article.
    pub fn unlink_contact_from_article(
        &self,
        article_id: Uuid,
        contact_id: Uuid,
    ) -> Result<bool, StorageError> {
        let conn = self.lock()?;
        article_contacts::unlink_contact_from_article(&conn, article_id, contact_id)
    }

    /// Checks if a contact is linked to an article.
    pub fn is_contact_linked_to_article(
        &self,
        article_id: Uuid,
        contact_id: Uuid,
    ) -> Result<bool, StorageError> {
        let conn = self.lock()?;
        article_contacts::is_contact_linked_to_article(&conn, article_id, contact_id)
    }

    /// Lists all contacts tagged to an article.
    pub fn list_contacts_for_article(
        &self,
        article_id: Uuid,
    ) -> Result<Vec<Contact>, StorageError> {
        let conn = self.lock()?;
        article_contacts::list_contacts_for_article(&conn, article_id)
    }

    /// Lists all articles that tag a specific contact.
    pub fn list_articles_for_contact(
        &self,
        contact_id: Uuid,
    ) -> Result<Vec<Article>, StorageError> {
        let conn = self.lock()?;
        article_contacts::list_articles_for_contact(&conn, contact_id)
    }

    /// Lists all article-contact join records.
    pub fn list_article_contacts(&self) -> Result<Vec<ArticleContact>, StorageError> {
        let conn = self.lock()?;
        article_contacts::list_article_contacts(&conn)
    }

    /// Synchronizes the complete set of contacts tagged to an article.
    pub fn set_article_contacts(
        &self,
        article_id: Uuid,
        contact_ids: &[Uuid],
    ) -> Result<(), StorageError> {
        let mut conn = self.lock()?;
        article_contacts::set_article_contacts(&mut conn, article_id, contact_ids)
    }

    /// Counts how many contacts are linked to an article.
    pub fn article_contact_count(&self, article_id: Uuid) -> Result<usize, StorageError> {
        let conn = self.lock()?;
        article_contacts::count_contacts_for_article(&conn, article_id)
    }

    /// Counts how many articles a contact is tagged in.
    pub fn contact_article_count(&self, contact_id: Uuid) -> Result<usize, StorageError> {
        let conn = self.lock()?;
        article_contacts::count_articles_for_contact(&conn, contact_id)
    }

    // ==========================================
    // Settings Operations
    // ==========================================

    /// Retrieves user settings, initializing defaults in the database if not present.
    pub fn get_settings(&self) -> Result<Settings, StorageError> {
        let conn = self.lock()?;
        settings::get_settings(&conn)
    }

    /// Saves user settings.
    pub fn save_settings(&self, settings: &Settings) -> Result<Settings, StorageError> {
        let conn = self.lock()?;
        settings::save_settings(&conn, settings)?;
        Ok(settings.clone())
    }

    /// Gets a generic configuration value by key.
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, StorageError> {
        let conn = self.lock()?;
        settings::get_setting(&conn, key)
    }

    /// Sets a generic configuration value by key.
    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), StorageError> {
        let conn = self.lock()?;
        settings::set_setting(&conn, key, value)
    }

    /// Retrieves the current UI theme mode preference.
    pub fn get_theme_mode(&self) -> Result<ThemeMode, StorageError> {
        let conn = self.lock()?;
        settings::get_theme_mode(&conn)
    }

    /// Sets and persists the UI theme mode preference.
    pub fn set_theme_mode(&self, mode: ThemeMode) -> Result<(), StorageError> {
        let conn = self.lock()?;
        settings::set_theme_mode(&conn, mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_service_comprehensive_workflow() {
        let service = StorageService::in_memory().expect("failed to init db");

        // 1. Create Article
        let article = Article::new(
            "harbor-clean-up-delays",
            "Harbor Clean-Up Delayed by 6 Months",
        )
        .with_description("Investigating EPA water quality testing results")
        .with_stage(ArticleStage::Writing)
        .with_auto_color();

        let created_article = service
            .create_article(article)
            .expect("create article failed");
        assert_eq!(created_article.slug, "harbor-clean-up-delays");
        assert_eq!(created_article.stage, ArticleStage::Writing);

        // 2. Create Tasks
        let task1 = Task::new(created_article.id, "Interview EPA Regional Director")
            .with_notes("Ask about contaminant levels in Zone B")
            .with_status(TaskStatus::InProgress);
        let task2 = Task::new(created_article.id, "Review water testing lab logs")
            .with_status(TaskStatus::Complete);

        service.create_task(task1).expect("create task1 failed");
        service.create_task(task2).expect("create task2 failed");

        let (total_tasks, completed_tasks) = service
            .article_task_counts(created_article.id)
            .expect("task counts failed");
        assert_eq!(total_tasks, 2);
        assert_eq!(completed_tasks, 1);

        // 3. Create Contacts and Link
        let contact1 = Contact::new("Elena Vance")
            .with_organization("Harbor Environmental Action")
            .with_role("Lead Whistleblower")
            .with_email("elena@action.org");
        let contact2 = Contact::new("Capt. Marcus Thorne")
            .with_organization("Port Authority")
            .with_role("Operations Director");

        let c1 = service.create_contact(contact1).expect("create c1 failed");
        let c2 = service.create_contact(contact2).expect("create c2 failed");

        service
            .link_contact_to_article(created_article.id, c1.id)
            .expect("link c1 failed");
        service
            .link_contact_to_article(created_article.id, c2.id)
            .expect("link c2 failed");

        assert_eq!(
            service.article_contact_count(created_article.id).unwrap(),
            2
        );
        assert_eq!(service.contact_article_count(c1.id).unwrap(), 1);

        let tagged_contacts = service
            .list_contacts_for_article(created_article.id)
            .expect("list contacts for article failed");
        assert_eq!(tagged_contacts.len(), 2);

        // 4. Update Article Stage
        let transitioned = service
            .change_article_stage(created_article.id, ArticleStage::ReadyToPublish)
            .expect("stage change failed");
        assert_eq!(transitioned.stage, ArticleStage::ReadyToPublish);

        // 5. Settings
        service.set_theme_mode(ThemeMode::Dark).unwrap();
        assert_eq!(service.get_theme_mode().unwrap(), ThemeMode::Dark);

        // 6. Delete Article cascades tasks and article_contacts
        assert!(service.delete_article(created_article.id).unwrap());
        assert_eq!(
            service
                .list_tasks_for_article(created_article.id)
                .unwrap()
                .len(),
            0
        );
        assert_eq!(
            service.article_contact_count(created_article.id).unwrap(),
            0
        );

        // Contacts still exist
        assert_eq!(service.list_contacts().unwrap().len(), 2);
    }

    #[test]
    fn test_storage_service_diagnostics_and_counts() {
        let service = StorageService::in_memory().expect("failed to init db");
        assert!(service.is_in_memory());
        assert_eq!(service.database_path(), None);
        assert_eq!(service.database_file_size().unwrap(), None);
        assert_eq!(service.current_schema_version().unwrap(), Some(1));
        assert_eq!(service.applied_migrations_count().unwrap(), 1);
        assert!(!StorageService::sqlite_version().is_empty());

        let counts_empty = service.entity_counts().unwrap();
        assert_eq!(counts_empty.total_articles, 0);
        assert_eq!(counts_empty.total_tasks, 0);
        assert_eq!(counts_empty.total_contacts, 0);
        assert_eq!(counts_empty.total_article_contacts, 0);

        let article = Article::new("test-article", "Test Article Headline");
        let a = service.create_article(article).unwrap();
        let task = Task::new(a.id, "Test task");
        service.create_task(task).unwrap();
        let contact = Contact::new("Test Contact");
        let c = service.create_contact(contact).unwrap();
        service.link_contact_to_article(a.id, c.id).unwrap();

        let counts_populated = service.entity_counts().unwrap();
        assert_eq!(counts_populated.total_articles, 1);
        assert_eq!(counts_populated.total_tasks, 1);
        assert_eq!(counts_populated.total_contacts, 1);
        assert_eq!(counts_populated.total_article_contacts, 1);
    }
}
