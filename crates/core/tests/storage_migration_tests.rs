//! Comprehensive integration tests for SQLite schema migrations, foreign keys, and indexes.

use chrono::Utc;
use newsjournal_core::storage::{
    configure_connection, open_in_memory, open_in_memory_unmigrated, run_migrations, Migration,
    MigrationRunner, StorageError,
};
use rusqlite::{params, Connection};
use uuid::Uuid;

#[test]
fn test_migration_runner_fresh_database() {
    let mut conn = open_in_memory_unmigrated().expect("failed to open unmigrated db");
    let runner = MigrationRunner::new();

    assert_eq!(runner.current_version(&conn).unwrap(), None);
    assert_eq!(runner.applied_migrations(&conn).unwrap().len(), 0);
    assert_eq!(runner.pending_migrations(&conn).unwrap().len(), 1);

    let report = runner.run(&mut conn).expect("migration failed");
    assert_eq!(report.applied_count, 1);
    assert_eq!(report.versions_applied, vec![1]);
    assert_eq!(report.initial_version, None);
    assert_eq!(report.final_version, Some(1));

    assert_eq!(runner.current_version(&conn).unwrap(), Some(1));
    let applied = runner.applied_migrations(&conn).unwrap();
    assert_eq!(applied.len(), 1);
    assert_eq!(applied[0].version, 1);
    assert_eq!(applied[0].name, "0001_initial_schema");
    assert!(!applied[0].applied_at.is_empty());
}

#[test]
fn test_migration_runner_idempotency() {
    let mut conn = open_in_memory().expect("failed to open migrated db");
    let runner = MigrationRunner::new();

    // Re-running on already migrated db
    let report = runner.run(&mut conn).expect("re-running migration failed");
    assert_eq!(report.applied_count, 0);
    assert!(report.versions_applied.is_empty());
    assert_eq!(report.initial_version, Some(1));
    assert_eq!(report.final_version, Some(1));

    assert_eq!(runner.pending_migrations(&conn).unwrap().len(), 0);
}

#[test]
fn test_run_migrations_convenience_function() {
    let mut conn = open_in_memory_unmigrated().expect("failed to open db");
    let report = run_migrations(&mut conn).expect("run_migrations failed");
    assert_eq!(report.applied_count, 1);
    assert_eq!(report.final_version, Some(1));
}

#[test]
fn test_tables_and_indexes_exist() {
    let conn = open_in_memory().expect("failed to open db");

    let mut stmt = conn
        .prepare("SELECT name, type FROM sqlite_master WHERE type IN ('table', 'index');")
        .unwrap();
    let objects: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    let table_names: Vec<&str> = objects
        .iter()
        .filter(|(_, t)| t == "table")
        .map(|(n, _)| n.as_str())
        .collect();

    assert!(table_names.contains(&"articles"));
    assert!(table_names.contains(&"tasks"));
    assert!(table_names.contains(&"contacts"));
    assert!(table_names.contains(&"article_contacts"));
    assert!(table_names.contains(&"settings"));
    assert!(table_names.contains(&"schema_migrations"));

    let index_names: Vec<&str> = objects
        .iter()
        .filter(|(_, t)| t == "index")
        .map(|(n, _)| n.as_str())
        .collect();

    // Article indexes
    assert!(index_names.contains(&"idx_articles_slug"));
    assert!(index_names.contains(&"idx_articles_stage"));
    assert!(index_names.contains(&"idx_articles_deadline"));
    assert!(index_names.contains(&"idx_articles_created_at"));
    assert!(index_names.contains(&"idx_articles_updated_at"));

    // Task indexes
    assert!(index_names.contains(&"idx_tasks_article_id"));
    assert!(index_names.contains(&"idx_tasks_status"));
    assert!(index_names.contains(&"idx_tasks_due_date"));
    assert!(index_names.contains(&"idx_tasks_created_at"));
    assert!(index_names.contains(&"idx_tasks_updated_at"));

    // Contact indexes
    assert!(index_names.contains(&"idx_contacts_name"));
    assert!(index_names.contains(&"idx_contacts_organization"));
    assert!(index_names.contains(&"idx_contacts_email"));
    assert!(index_names.contains(&"idx_contacts_created_at"));
    assert!(index_names.contains(&"idx_contacts_updated_at"));

    // ArticleContact indexes
    assert!(index_names.contains(&"idx_article_contacts_article_id"));
    assert!(index_names.contains(&"idx_article_contacts_contact_id"));
}

#[test]
fn test_article_crud_and_slug_uniqueness() {
    let conn = open_in_memory().expect("failed to open db");
    let id1 = Uuid::new_v4().to_string();
    let id2 = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    // Insert article 1
    conn.execute(
        "INSERT INTO articles (id, slug, headline, description, stage, deadline, color, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);",
        params![
            id1,
            "budget-investigation",
            "City Budget Investigation",
            "Deep dive into city financials",
            "researching",
            Option::<String>::None,
            "#4A90E2",
            now,
            now
        ],
    ).unwrap();

    // Duplicate slug insert must fail with UNIQUE constraint violation
    let result = conn.execute(
        "INSERT INTO articles (id, slug, headline, description, stage, deadline, color, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);",
        params![
            id2,
            "budget-investigation", // duplicate
            "Different Headline",
            None::<String>,
            "pitching",
            None::<String>,
            None::<String>,
            now,
            now
        ],
    );
    assert!(result.is_err());
}

#[test]
fn test_cascade_delete_article_removes_tasks() {
    let conn = open_in_memory().expect("failed to open db");
    let article_id = Uuid::new_v4().to_string();
    let task1_id = Uuid::new_v4().to_string();
    let task2_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    // Insert article
    conn.execute(
        "INSERT INTO articles (id, slug, headline, description, stage, deadline, color, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);",
        params![
            article_id,
            "transit-story",
            "Transit Story",
            None::<String>,
            "writing",
            None::<String>,
            None::<String>,
            now,
            now
        ],
    ).unwrap();

    // Insert tasks for the article
    conn.execute(
        "INSERT INTO tasks (id, article_id, title, notes, due_date, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
        params![
            task1_id,
            article_id,
            "Interview transit chief",
            None::<String>,
            None::<String>,
            "to_do",
            now,
            now
        ],
    )
    .unwrap();

    conn.execute(
        "INSERT INTO tasks (id, article_id, title, notes, due_date, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
        params![
            task2_id,
            article_id,
            "Analyze route metrics",
            None::<String>,
            None::<String>,
            "in_progress",
            now,
            now
        ],
    )
    .unwrap();

    // Verify tasks exist
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tasks WHERE article_id = ?1;",
            params![article_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 2);

    // Delete the parent article
    conn.execute("DELETE FROM articles WHERE id = ?1;", params![article_id])
        .unwrap();

    // Verify tasks were cascade deleted
    let remaining_tasks: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tasks WHERE article_id = ?1;",
            params![article_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(remaining_tasks, 0);
}

#[test]
fn test_cascade_delete_article_removes_article_contacts() {
    let conn = open_in_memory().expect("failed to open db");
    let article_id = Uuid::new_v4().to_string();
    let contact_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    // Insert article
    conn.execute(
        "INSERT INTO articles (id, slug, headline, description, stage, deadline, color, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);",
        params![
            article_id,
            "housing-investigation",
            "Housing Crisis",
            None::<String>,
            "pitching",
            None::<String>,
            None::<String>,
            now,
            now
        ],
    ).unwrap();

    // Insert contact
    conn.execute(
        "INSERT INTO contacts (id, name, organization, role, phone, email, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);",
        params![
            contact_id,
            "Mayor Office",
            "City Hall",
            "Spokesperson",
            None::<String>,
            "spokesperson@cityhall.gov",
            None::<String>,
            now,
            now
        ],
    ).unwrap();

    // Link article and contact
    conn.execute(
        "INSERT INTO article_contacts (article_id, contact_id, created_at)
         VALUES (?1, ?2, ?3);",
        params![article_id, contact_id, now],
    )
    .unwrap();

    // Delete article
    conn.execute("DELETE FROM articles WHERE id = ?1;", params![article_id])
        .unwrap();

    // Verify article_contacts link is removed
    let link_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM article_contacts WHERE article_id = ?1;",
            params![article_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(link_count, 0);

    // Verify contact itself is NOT deleted
    let contact_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM contacts WHERE id = ?1;",
            params![contact_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(contact_count, 1);
}

#[test]
fn test_cascade_delete_contact_removes_article_contacts() {
    let conn = open_in_memory().expect("failed to open db");
    let article_id = Uuid::new_v4().to_string();
    let contact_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    // Insert article
    conn.execute(
        "INSERT INTO articles (id, slug, headline, description, stage, deadline, color, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);",
        params![
            article_id,
            "education-reform",
            "Education Reform",
            None::<String>,
            "pitching",
            None::<String>,
            None::<String>,
            now,
            now
        ],
    ).unwrap();

    // Insert contact
    conn.execute(
        "INSERT INTO contacts (id, name, organization, role, phone, email, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);",
        params![
            contact_id,
            "School Board President",
            "Unified School District",
            "President",
            None::<String>,
            None::<String>,
            None::<String>,
            now,
            now
        ],
    ).unwrap();

    // Link
    conn.execute(
        "INSERT INTO article_contacts (article_id, contact_id, created_at)
         VALUES (?1, ?2, ?3);",
        params![article_id, contact_id, now],
    )
    .unwrap();

    // Delete contact
    conn.execute("DELETE FROM contacts WHERE id = ?1;", params![contact_id])
        .unwrap();

    // Verify article_contacts link removed
    let link_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM article_contacts WHERE contact_id = ?1;",
            params![contact_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(link_count, 0);

    // Verify article itself is NOT deleted
    let article_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM articles WHERE id = ?1;",
            params![article_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(article_count, 1);
}

#[test]
fn test_foreign_key_violation_on_orphaned_task_insert() {
    let conn = open_in_memory().expect("failed to open db");
    let non_existent_article_id = Uuid::new_v4().to_string();
    let task_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let result = conn.execute(
        "INSERT INTO tasks (id, article_id, title, notes, due_date, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
        params![
            task_id,
            non_existent_article_id,
            "Orphaned task",
            None::<String>,
            None::<String>,
            "to_do",
            now,
            now
        ],
    );

    assert!(result.is_err());
}

#[test]
fn test_foreign_key_violation_on_orphaned_article_contact_insert() {
    let conn = open_in_memory().expect("failed to open db");
    let non_existent_article_id = Uuid::new_v4().to_string();
    let non_existent_contact_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let result = conn.execute(
        "INSERT INTO article_contacts (article_id, contact_id, created_at)
         VALUES (?1, ?2, ?3);",
        params![non_existent_article_id, non_existent_contact_id, now],
    );

    assert!(result.is_err());
}

#[test]
fn test_settings_key_value_upsert() {
    let conn = open_in_memory().expect("failed to open db");
    let now = Utc::now().to_rfc3339();

    // Insert setting
    conn.execute(
        "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3);",
        params!["theme_mode", "dark", now],
    )
    .unwrap();

    let val: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1;",
            params!["theme_mode"],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(val, "dark");

    // Upsert setting
    let updated_now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at;",
        params!["theme_mode", "light", updated_now],
    )
    .unwrap();

    let updated_val: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1;",
            params!["theme_mode"],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(updated_val, "light");
}

#[test]
fn test_migration_failure_rolls_back_transaction() {
    let mut conn = Connection::open_in_memory().unwrap();
    configure_connection(&conn).unwrap();

    let migrations = vec![
        Migration {
            version: 1,
            name: "0001_good",
            sql: "CREATE TABLE table_one (id INTEGER PRIMARY KEY);",
        },
        Migration {
            version: 2,
            name: "0002_bad",
            sql: "INVALID SQL SYNTAX HERE;",
        },
    ];

    let runner = MigrationRunner::with_migrations(migrations);
    let result = runner.run(&mut conn);

    assert!(matches!(
        result,
        Err(StorageError::MigrationFailed { version: 2, .. })
    ));

    // Migration 1 should be committed because it was in its own transaction
    assert_eq!(runner.current_version(&conn).unwrap(), Some(1));

    // Table from migration 2 was not created
    let table_one_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'table_one';",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(table_one_exists, 1);
}

#[test]
fn test_mismatched_migration_name_error() {
    let mut conn = open_in_memory().expect("failed to open db");

    // Try running with a runner having a different name for version 1
    let mismatched = vec![Migration {
        version: 1,
        name: "0001_different_name",
        sql: "SELECT 1;",
    }];

    let runner = MigrationRunner::with_migrations(mismatched);
    let result = runner.run(&mut conn);

    assert!(matches!(
        result,
        Err(StorageError::MigrationMismatch {
            version: 1,
            ref expected,
            ref found,
        }) if expected == "0001_different_name" && found == "0001_initial_schema"
    ));
}

#[test]
fn test_corrupted_migration_history_error() {
    let mut conn = open_in_memory_unmigrated().expect("failed to open db");
    MigrationRunner::ensure_migration_table(&conn).unwrap();

    // Manually insert an unknown high version
    conn.execute(
        "INSERT INTO schema_migrations (version, name, applied_at) VALUES (999, 'unknown', '2026-01-01');",
        [],
    ).unwrap();

    let runner = MigrationRunner::new();
    let result = runner.run(&mut conn);

    assert!(matches!(
        result,
        Err(StorageError::CorruptedMigrationHistory { found: 999 })
    ));
}
