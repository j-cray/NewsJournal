use std::path::{Path, PathBuf};
use std::thread;

use chrono::{Duration, Utc};
use newsjournal_core::storage::StorageService;
use newsjournal_core::{Article, ArticleStage, Contact, Settings, Task, TaskStatus, ThemeMode};
use uuid::Uuid;

struct TempDbGuard {
    path: PathBuf,
}

impl TempDbGuard {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("newsjournal_test_{}.db", Uuid::new_v4()));
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDbGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        // Also remove SQLite WAL/SHM files if created
        let _ = std::fs::remove_file(self.path.with_extension("db-wal"));
        let _ = std::fs::remove_file(self.path.with_extension("db-shm"));
    }
}

#[test]
fn test_article_repository_full_lifecycle() {
    let service = StorageService::in_memory().expect("failed to open db");
    let now = Utc::now();
    let deadline = now + Duration::days(5);

    let article = Article::new(
        "state-pension-shortfall",
        "State Pension Fund Faces $2B Shortfall",
    )
    .with_description("Deep dive into unhedged real estate investments")
    .with_stage(ArticleStage::Pitching)
    .with_deadline(deadline)
    .with_color("#E65100");

    // 1. Create
    let created = service
        .create_article(article.clone())
        .expect("failed to create article");
    assert_eq!(created.id, article.id);
    assert_eq!(created.slug, "state-pension-shortfall");
    assert_eq!(created.stage, ArticleStage::Pitching);
    assert_eq!(created.color.as_deref(), Some("#E65100"));
    assert!(created.deadline.is_some());

    // 2. Fetch by ID and Slug
    let fetched_id = service
        .get_article(article.id)
        .unwrap()
        .expect("article not found by id");
    assert_eq!(fetched_id.slug, "state-pension-shortfall");

    let fetched_slug = service
        .get_article_by_slug("state-pension-shortfall")
        .unwrap()
        .expect("article not found by slug");
    assert_eq!(fetched_slug.id, article.id);

    // 3. Update Headline and Stage
    let mut modified = fetched_id;
    modified.headline = "State Pension Fund Shortfall Grows to $3B".to_string();
    modified.stage = ArticleStage::Writing;
    let updated = service
        .update_article(modified)
        .expect("failed to update article");
    assert_eq!(
        updated.headline,
        "State Pension Fund Shortfall Grows to $3B"
    );
    assert_eq!(updated.stage, ArticleStage::Writing);

    // 4. Duplicate Slug Collision
    let duplicate_slug_article = Article::new("state-pension-shortfall", "Another Story Same Slug");
    let err = service
        .create_article(duplicate_slug_article)
        .expect_err("expected duplicate slug error");
    assert!(err.is_conflict());

    // 5. Slug availability checks
    assert!(!service
        .is_slug_available("state-pension-shortfall", None)
        .unwrap());
    assert!(service
        .is_slug_available("state-pension-shortfall", Some(article.id))
        .unwrap());
    assert!(service
        .is_slug_available("completely-new-story-slug", None)
        .unwrap());

    // 6. Stage Transition and Listing by Stage
    service
        .change_article_stage(article.id, ArticleStage::Editing)
        .expect("stage change failed");
    let editing_articles = service
        .list_articles_by_stage(ArticleStage::Editing)
        .unwrap();
    assert_eq!(editing_articles.len(), 1);
    assert_eq!(editing_articles[0].id, article.id);

    let pitching_articles = service
        .list_articles_by_stage(ArticleStage::Pitching)
        .unwrap();
    assert_eq!(pitching_articles.len(), 0);

    // 7. Delete Article
    assert!(service.delete_article(article.id).unwrap());
    assert!(service.get_article(article.id).unwrap().is_none());
    assert!(!service.delete_article(article.id).unwrap());
}

#[test]
fn test_task_repository_full_lifecycle_and_cascading() {
    let service = StorageService::in_memory().expect("failed to open db");

    let article = Article::new(
        "hospital-er-overcrowding",
        "ER Wait Times Spike Across County",
    );
    service
        .create_article(article.clone())
        .expect("create article failed");

    // 1. Create Tasks
    let task1 = Task::new(article.id, "FOIA request for ambulance diversion logs")
        .with_notes("Send to County Emergency Services")
        .with_due_date(Utc::now() + Duration::days(2))
        .with_status(TaskStatus::ToDo);

    let task2 =
        Task::new(article.id, "Interview ER Nurses Union Rep").with_status(TaskStatus::InProgress);

    let t1 = service.create_task(task1).expect("create task1 failed");
    let t2 = service.create_task(task2).expect("create task2 failed");

    // 2. Fetch Tasks
    assert_eq!(
        service.get_task(t1.id).unwrap().unwrap().title,
        "FOIA request for ambulance diversion logs"
    );
    let article_tasks = service.list_tasks_for_article(article.id).unwrap();
    assert_eq!(article_tasks.len(), 2);

    // 3. Task Counts
    let (total, completed) = service.article_task_counts(article.id).unwrap();
    assert_eq!(total, 2);
    assert_eq!(completed, 0);

    // 4. Update Status
    service
        .change_task_status(t1.id, TaskStatus::Complete)
        .unwrap();
    let (total_after, completed_after) = service.article_task_counts(article.id).unwrap();
    assert_eq!(total_after, 2);
    assert_eq!(completed_after, 1);

    // 5. Orphaned Task foreign key enforcement
    let orphaned_task = Task::new(Uuid::new_v4(), "Orphaned Task");
    let err = service
        .create_task(orphaned_task)
        .expect_err("expected foreign key failure");
    assert!(err.is_not_found());

    // 6. Delete Parent Article -> Tasks should be cascade deleted
    assert!(service.delete_article(article.id).unwrap());
    assert!(service.get_task(t1.id).unwrap().is_none());
    assert!(service.get_task(t2.id).unwrap().is_none());
    assert_eq!(service.list_tasks().unwrap().len(), 0);
}

#[test]
fn test_contact_repository_search_and_tagging_relationships() {
    let service = StorageService::in_memory().expect("failed to open db");

    // Create 2 articles
    let a1 = service
        .create_article(Article::new("airport-noise-study", "Airport Noise Study"))
        .unwrap();
    let a2 = service
        .create_article(Article::new("flight-path-rezoning", "Flight Path Rezoning"))
        .unwrap();

    // Create 3 contacts
    let c1 = service
        .create_contact(
            Contact::new("Dr. Naomi Chen")
                .with_organization("Aviation Acoustics Institute")
                .with_role("Lead Acoustical Engineer")
                .with_email("nchen@acoustics.org")
                .with_phone("+1 555-432-1098")
                .with_notes("Primary technical expert on decibel thresholds"),
        )
        .unwrap();

    let c2 = service
        .create_contact(
            Contact::new("Captain David Ross")
                .with_organization("Regional Pilots Association")
                .with_role("Safety Committee Chair")
                .with_email("dross@pilots.example.com"),
        )
        .unwrap();

    let c3 = service
        .create_contact(
            Contact::new("Councilwoman Sarah Jenkins")
                .with_organization("City Council")
                .with_role("District 4 Representative"),
        )
        .unwrap();

    // 1. Search Contacts
    let search_org = service.search_contacts("Acoustics").unwrap();
    assert_eq!(search_org.len(), 1);
    assert_eq!(search_org[0].id, c1.id);

    let search_notes = service.search_contacts("decibel").unwrap();
    assert_eq!(search_notes.len(), 1);
    assert_eq!(search_notes[0].id, c1.id);

    let search_all = service.search_contacts("").unwrap();
    assert_eq!(search_all.len(), 3);

    // 2. Link Contacts to Articles
    service.link_contact_to_article(a1.id, c1.id).unwrap();
    service.link_contact_to_article(a1.id, c2.id).unwrap();
    service.link_contact_to_article(a2.id, c1.id).unwrap();
    service.link_contact_to_article(a2.id, c3.id).unwrap();

    // Verify Many-to-Many
    let a1_contacts = service.list_contacts_for_article(a1.id).unwrap();
    assert_eq!(a1_contacts.len(), 2);

    let c1_articles = service.list_articles_for_contact(c1.id).unwrap();
    assert_eq!(c1_articles.len(), 2);

    assert_eq!(service.article_contact_count(a1.id).unwrap(), 2);
    assert_eq!(service.contact_article_count(c1.id).unwrap(), 2);
    assert_eq!(service.contact_article_count(c2.id).unwrap(), 1);

    // 3. Bulk Sync Contacts for Article
    service.set_article_contacts(a1.id, &[c3.id]).unwrap();
    let a1_contacts_synced = service.list_contacts_for_article(a1.id).unwrap();
    assert_eq!(a1_contacts_synced.len(), 1);
    assert_eq!(a1_contacts_synced[0].id, c3.id);

    // 4. Delete Contact -> Unlinks without deleting articles
    assert!(service.delete_contact(c3.id).unwrap());
    assert_eq!(service.list_contacts_for_article(a1.id).unwrap().len(), 0);
    assert!(service.get_article(a1.id).unwrap().is_some());
    assert!(service.get_article(a2.id).unwrap().is_some());
}

#[test]
fn test_settings_persistence_flow() {
    let service = StorageService::in_memory().expect("failed to open db");

    // Default settings
    let initial = service.get_settings().expect("failed to load settings");
    assert_eq!(initial.theme_mode, ThemeMode::System);

    // Update theme mode
    service
        .set_theme_mode(ThemeMode::Dark)
        .expect("failed to set dark");
    assert_eq!(service.get_theme_mode().unwrap(), ThemeMode::Dark);

    let saved = service.get_settings().unwrap();
    assert_eq!(saved.theme_mode, ThemeMode::Dark);

    // Save full settings struct
    let custom = Settings::new(ThemeMode::Light);
    service.save_settings(&custom).unwrap();
    assert_eq!(service.get_theme_mode().unwrap(), ThemeMode::Light);

    // Generic config keys
    service
        .set_setting("editor_font_size", "14")
        .expect("set config failed");
    assert_eq!(
        service.get_setting("editor_font_size").unwrap(),
        Some("14".to_string())
    );
}

#[test]
fn test_concurrent_multithreaded_storage_service() {
    let service = StorageService::in_memory().expect("failed to open db");
    let mut handles = Vec::new();

    // Spawn 8 worker threads doing concurrent creates, updates, and queries
    for thread_idx in 0..8 {
        let s = service.clone();
        let handle = thread::spawn(move || {
            for i in 0..10 {
                let slug = format!("thread-{thread_idx}-story-{i}");
                let headline = format!("Headline from thread {thread_idx} item {i}");
                let article = Article::new(&slug, &headline)
                    .with_stage(ArticleStage::Researching)
                    .with_auto_color();

                let created = s.create_article(article).expect("concurrent create failed");

                let task =
                    Task::new(created.id, format!("Task {i}")).with_status(TaskStatus::InProgress);
                s.create_task(task).expect("concurrent task create failed");

                let contact = Contact::new(format!("Source {thread_idx}-{i}"))
                    .with_email(format!("source_{thread_idx}_{i}@test.com"));
                let created_contact = s
                    .create_contact(contact)
                    .expect("concurrent contact create failed");

                s.link_contact_to_article(created.id, created_contact.id)
                    .expect("concurrent link failed");

                let fetched = s.get_article_by_slug(&slug).unwrap().unwrap();
                assert_eq!(fetched.id, created.id);
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().expect("thread panicked");
    }

    // Total articles = 8 threads * 10 articles = 80
    let all_articles = service.list_articles().expect("list all failed");
    assert_eq!(all_articles.len(), 80);

    let all_tasks = service.list_tasks().expect("list all tasks failed");
    assert_eq!(all_tasks.len(), 80);

    let all_contacts = service.list_contacts().expect("list all contacts failed");
    assert_eq!(all_contacts.len(), 80);
}

#[test]
fn test_file_backed_database_creation_and_reload() {
    let guard = TempDbGuard::new();
    let db_path = guard.path();

    // Phase 1: Open file DB and write data
    let article_id = {
        let service = StorageService::open(db_path).expect("failed to open file db");
        let article = Article::new("file-db-test", "File Database Persistence Verification")
            .with_stage(ArticleStage::ReadyToPublish)
            .with_color("#1565C0");

        let created = service.create_article(article).expect("create failed");
        let task = Task::new(created.id, "Verify file reload integrity");
        service.create_task(task).expect("create task failed");

        let contact = Contact::new("Database Inspector").with_email("inspector@db.local");
        let created_contact = service
            .create_contact(contact)
            .expect("create contact failed");
        service
            .link_contact_to_article(created.id, created_contact.id)
            .expect("link failed");

        service.set_theme_mode(ThemeMode::Dark).unwrap();
        created.id
    };

    // Phase 2: Re-open the database from the same path with a fresh StorageService
    {
        let service2 = StorageService::open(db_path).expect("failed to reopen file db");

        let reloaded_article = service2
            .get_article(article_id)
            .unwrap()
            .expect("article not found after reload");
        assert_eq!(reloaded_article.slug, "file-db-test");
        assert_eq!(reloaded_article.stage, ArticleStage::ReadyToPublish);
        assert_eq!(reloaded_article.color.as_deref(), Some("#1565C0"));

        let reloaded_tasks = service2.list_tasks_for_article(article_id).unwrap();
        assert_eq!(reloaded_tasks.len(), 1);
        assert_eq!(reloaded_tasks[0].title, "Verify file reload integrity");

        let reloaded_contacts = service2.list_contacts_for_article(article_id).unwrap();
        assert_eq!(reloaded_contacts.len(), 1);
        assert_eq!(reloaded_contacts[0].name, "Database Inspector");

        assert_eq!(service2.get_theme_mode().unwrap(), ThemeMode::Dark);
    }
}
