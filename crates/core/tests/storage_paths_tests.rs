//! Integration tests for AppPaths resolution and StorageService path-based initialization.

use std::fs;
use std::sync::Mutex;

use newsjournal_core::models::{Article, Contact, Settings, Task, ThemeMode};
use newsjournal_core::storage::{
    AppPaths, StorageService, DEFAULT_DB_FILENAME, ENV_CACHE_DIR, ENV_CONFIG_DIR, ENV_DATA_DIR,
    ENV_DB_PATH, ENV_STATE_DIR,
};

// Global test mutex to prevent concurrent environment variable modification in tests
static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_app_paths_from_root_initialization_and_creation() {
    let unique_id = uuid::Uuid::new_v4();
    let root = std::env::temp_dir().join(format!("nj_test_root_{unique_id}"));

    let paths = AppPaths::from_root(&root);
    assert_eq!(paths.data_dir(), root.join("data"));
    assert_eq!(paths.config_dir(), root.join("config"));
    assert_eq!(paths.cache_dir(), root.join("cache"));
    assert_eq!(paths.state_dir(), Some(root.join("state").as_path()));
    assert_eq!(
        paths.database_path(),
        root.join("data").join(DEFAULT_DB_FILENAME)
    );

    // Initial state: directories do not exist
    assert!(!paths.data_dir().exists());
    assert!(!paths.config_dir().exists());
    assert!(!paths.cache_dir().exists());

    // Ensure all directories
    paths
        .ensure_all()
        .expect("failed to ensure all directories");
    assert!(paths.data_dir().is_dir());
    assert!(paths.config_dir().is_dir());
    assert!(paths.cache_dir().is_dir());
    assert!(paths.state_dir().unwrap().is_dir());

    // Cleanup
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_storage_service_open_with_paths_lifecycle() {
    let unique_id = uuid::Uuid::new_v4();
    let root = std::env::temp_dir().join(format!("nj_test_lifecycle_{unique_id}"));
    let paths = AppPaths::from_root(&root);

    let article_id;
    let task_id;
    let contact_id;

    // Scope 1: Create entities in file-backed database
    {
        let service =
            StorageService::open_with_paths(&paths).expect("failed to open storage with paths");
        assert!(paths.database_path().exists());

        // Create Article
        let article = Article::new("breaking-investigation", "City Hall Audit Exposed");
        let created_article = service
            .create_article(article)
            .expect("failed to create article");
        article_id = created_article.id;

        // Create Task
        let task = Task::new(article_id, "Review FOIA disclosures");
        let created_task = service.create_task(task).expect("failed to create task");
        task_id = created_task.id;

        // Create Contact
        let contact = Contact::new("Whistleblower Bob");
        let created_contact = service
            .create_contact(contact)
            .expect("failed to create contact");
        contact_id = created_contact.id;

        // Tag contact in article
        service
            .link_contact_to_article(article_id, contact_id)
            .expect("failed to link contact");

        // Update settings
        let settings = Settings::new(ThemeMode::Dark);
        service
            .save_settings(&settings)
            .expect("failed to save settings");
    }

    // Scope 2: Re-open storage and verify persistence
    {
        let service =
            StorageService::open_with_paths(&paths).expect("failed to re-open storage with paths");

        let fetched_article = service
            .get_article(article_id)
            .expect("failed to get article")
            .expect("article not found");
        assert_eq!(fetched_article.slug, "breaking-investigation");
        assert_eq!(fetched_article.headline, "City Hall Audit Exposed");

        let fetched_task = service
            .get_task(task_id)
            .expect("failed to get task")
            .expect("task not found");
        assert_eq!(fetched_task.title, "Review FOIA disclosures");

        let contacts = service
            .list_contacts_for_article(article_id)
            .expect("failed to list contacts for article");
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].name, "Whistleblower Bob");

        let settings = service.get_settings().expect("failed to get settings");
        assert_eq!(settings.theme_mode, ThemeMode::Dark);
    }

    // Cleanup
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_storage_service_with_custom_database_path() {
    let unique_id = uuid::Uuid::new_v4();
    let root = std::env::temp_dir().join(format!("nj_test_custom_db_{unique_id}"));
    let custom_db = root
        .join("nested_folder")
        .join("deep")
        .join("custom_stories.db");

    let paths = AppPaths::from_root(&root).with_database_path(&custom_db);
    assert_eq!(paths.database_path(), custom_db);

    let service =
        StorageService::open_with_paths(&paths).expect("failed to open custom db storage");
    assert!(custom_db.exists());

    let article = Article::new("custom-story", "Custom Nested DB Story");
    let created = service
        .create_article(article)
        .expect("failed to create article in custom db");
    assert_eq!(created.slug, "custom-story");

    // Cleanup
    let _ = fs::remove_dir_all(&root);
}

struct EnvCleanupGuard;

impl Drop for EnvCleanupGuard {
    fn drop(&mut self) {
        std::env::remove_var(ENV_DATA_DIR);
        std::env::remove_var(ENV_CONFIG_DIR);
        std::env::remove_var(ENV_CACHE_DIR);
        std::env::remove_var(ENV_STATE_DIR);
        std::env::remove_var(ENV_DB_PATH);
    }
}

#[test]
fn test_app_paths_environment_variable_overrides() {
    let _guard = ENV_MUTEX.lock().unwrap();
    let _cleanup = EnvCleanupGuard;

    let unique_id = uuid::Uuid::new_v4();
    let base = std::env::temp_dir().join(format!("nj_env_test_{unique_id}"));

    let custom_data = base.join("env_data");
    let custom_config = base.join("env_config");
    let custom_cache = base.join("env_cache");
    let custom_state = base.join("env_state");
    let custom_db = base.join("env_db").join("my_journal.db");

    std::env::set_var(ENV_DATA_DIR, &custom_data);
    std::env::set_var(ENV_CONFIG_DIR, &custom_config);
    std::env::set_var(ENV_CACHE_DIR, &custom_cache);
    std::env::set_var(ENV_STATE_DIR, &custom_state);
    std::env::set_var(ENV_DB_PATH, &custom_db);

    let paths = AppPaths::from_env().expect("failed to resolve from env");
    assert_eq!(paths.data_dir(), custom_data);
    assert_eq!(paths.config_dir(), custom_config);
    assert_eq!(paths.cache_dir(), custom_cache);
    assert_eq!(paths.state_dir(), Some(custom_state.as_path()));
    assert_eq!(paths.database_path(), custom_db);
}

#[test]
fn test_storage_service_default_database_path() {
    let _guard = ENV_MUTEX.lock().unwrap();
    let default_path =
        StorageService::default_database_path().expect("failed to resolve default db path");
    assert!(default_path.ends_with(DEFAULT_DB_FILENAME));
    assert!(default_path.is_absolute());
}
