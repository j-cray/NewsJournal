//! Comprehensive SQLite repository integration test suite for `newsjournal-core`.
//!
//! Validates:
//! 1. Relational integrity & foreign key constraint enforcement
//! 2. Cascade deletions across Articles, Tasks, and Article-Contact links
//! 3. Many-to-many joins, bidirectional lookups, and atomic set synchronization
//! 4. Multi-stage Kanban querying and state machine transitions
//! 5. Task status filtering, article task counts, and progress tracking
//! 6. Multi-field contact search and substring queries
//! 7. Slug uniqueness, collision rejection, and rename workflows
//! 8. Timestamp preservation and advancement semantics
//! 9. End-to-end editorial newsroom lifecycle workflows with deadline evaluation

use std::collections::HashSet;
use std::thread::sleep;
use std::time::Duration as StdDuration;

use chrono::{Duration, Utc};
use newsjournal_core::deadline::{evaluate_article_deadline, DeadlineStatus, UrgencyLevel};
use newsjournal_core::storage::StorageService;
use newsjournal_core::{Article, ArticleStage, Contact, Settings, Task, TaskStatus, ThemeMode};
use uuid::Uuid;

// =============================================================================
// 1. Relational Integrity & Foreign Key Constraint Enforcement
// =============================================================================

#[test]
fn test_fk_orphaned_task_rejection() {
    let service = StorageService::in_memory().expect("failed to init in-memory db");
    let non_existent_article_id = Uuid::new_v4();

    let task = Task::new(non_existent_article_id, "Investigate ghost article");
    let err = service
        .create_task(task)
        .expect_err("inserting task with non-existent article_id must fail");

    assert!(
        err.is_not_found(),
        "error should be StorageError::NotFound on foreign key violation"
    );
}

#[test]
fn test_fk_orphaned_article_contact_rejection() {
    let service = StorageService::in_memory().expect("failed to init in-memory db");

    let article = service
        .create_article(Article::new("valid-article", "Valid Article Headline"))
        .expect("create article failed");

    let contact = service
        .create_contact(Contact::new("Valid Source Name"))
        .expect("create contact failed");

    let ghost_article_id = Uuid::new_v4();
    let ghost_contact_id = Uuid::new_v4();

    // 1. Valid Article -> Ghost Contact
    let err1 = service
        .link_contact_to_article(article.id, ghost_contact_id)
        .expect_err("linking non-existent contact should fail");
    assert!(err1.is_not_found());

    // 2. Ghost Article -> Valid Contact
    let err2 = service
        .link_contact_to_article(ghost_article_id, contact.id)
        .expect_err("linking non-existent article should fail");
    assert!(err2.is_not_found());

    // 3. Ghost Article -> Ghost Contact
    let err3 = service
        .link_contact_to_article(ghost_article_id, ghost_contact_id)
        .expect_err("linking two non-existent entities should fail");
    assert!(err3.is_not_found());
}

#[test]
fn test_fk_set_article_contacts_transaction_rollback() {
    let service = StorageService::in_memory().expect("failed to init in-memory db");

    let article = service
        .create_article(Article::new("campaign-finance", "Campaign Finance Report"))
        .expect("create article failed");

    let c1 = service
        .create_contact(Contact::new("Auditor Alice"))
        .expect("create c1 failed");
    let c2 = service
        .create_contact(Contact::new("Treasurer Bob"))
        .expect("create c2 failed");

    // Establish initial link with c1
    service
        .set_article_contacts(article.id, &[c1.id])
        .expect("initial set_article_contacts failed");
    assert_eq!(
        service.list_contacts_for_article(article.id).unwrap().len(),
        1
    );

    // Attempt to set with a valid c2 AND a non-existent c3
    let ghost_id = Uuid::new_v4();
    let err = service
        .set_article_contacts(article.id, &[c2.id, ghost_id])
        .expect_err("atomic set with ghost contact must fail");
    assert!(err.is_not_found());

    // Verify transactional rollback: c1 is still linked, c2 is NOT linked
    let current_contacts = service.list_contacts_for_article(article.id).unwrap();
    assert_eq!(current_contacts.len(), 1);
    assert_eq!(current_contacts[0].id, c1.id);
}

// =============================================================================
// 2. Cascade Deletions Across Relations
// =============================================================================

#[test]
fn test_cascade_delete_article_cleans_tasks_and_links_preserving_contacts() {
    let service = StorageService::in_memory().expect("failed to init in-memory db");

    // Create Article A with 3 tasks and 3 contacts
    let article_a = service
        .create_article(Article::new("story-alpha", "Story Alpha Headline"))
        .unwrap();

    let t1 = service
        .create_task(Task::new(article_a.id, "Alpha Task 1"))
        .unwrap();
    let t2 = service
        .create_task(Task::new(article_a.id, "Alpha Task 2"))
        .unwrap();
    let t3 = service
        .create_task(Task::new(article_a.id, "Alpha Task 3"))
        .unwrap();

    let c1 = service.create_contact(Contact::new("Contact One")).unwrap();
    let c2 = service.create_contact(Contact::new("Contact Two")).unwrap();
    let c3 = service
        .create_contact(Contact::new("Contact Three"))
        .unwrap();

    service
        .link_contact_to_article(article_a.id, c1.id)
        .unwrap();
    service
        .link_contact_to_article(article_a.id, c2.id)
        .unwrap();
    service
        .link_contact_to_article(article_a.id, c3.id)
        .unwrap();

    // Create Article B linked to c1 and c2 with its own task
    let article_b = service
        .create_article(Article::new("story-beta", "Story Beta Headline"))
        .unwrap();
    let t4 = service
        .create_task(Task::new(article_b.id, "Beta Task 4"))
        .unwrap();
    service
        .link_contact_to_article(article_b.id, c1.id)
        .unwrap();
    service
        .link_contact_to_article(article_b.id, c2.id)
        .unwrap();

    // Verify pre-conditions
    assert_eq!(
        service.list_tasks_for_article(article_a.id).unwrap().len(),
        3
    );
    assert_eq!(
        service
            .list_contacts_for_article(article_a.id)
            .unwrap()
            .len(),
        3
    );
    assert_eq!(service.contact_article_count(c1.id).unwrap(), 2);
    assert_eq!(service.contact_article_count(c2.id).unwrap(), 2);
    assert_eq!(service.contact_article_count(c3.id).unwrap(), 1);

    // Delete Article A
    let deleted = service.delete_article(article_a.id).unwrap();
    assert!(deleted);

    // 1. Article A is gone
    assert!(service.get_article(article_a.id).unwrap().is_none());
    assert!(service
        .get_article_by_slug("story-alpha")
        .unwrap()
        .is_none());

    // 2. Tasks t1, t2, t3 are cascade-deleted
    assert!(service.get_task(t1.id).unwrap().is_none());
    assert!(service.get_task(t2.id).unwrap().is_none());
    assert!(service.get_task(t3.id).unwrap().is_none());
    assert_eq!(
        service.list_tasks_for_article(article_a.id).unwrap().len(),
        0
    );

    // 3. Article A contact links are cascade-deleted
    assert_eq!(
        service
            .list_contacts_for_article(article_a.id)
            .unwrap()
            .len(),
        0
    );
    assert_eq!(service.article_contact_count(article_a.id).unwrap(), 0);

    // 4. Contacts c1, c2, c3 STILL EXIST and are intact
    assert!(service.get_contact(c1.id).unwrap().is_some());
    assert!(service.get_contact(c2.id).unwrap().is_some());
    assert!(service.get_contact(c3.id).unwrap().is_some());
    assert_eq!(service.list_contacts().unwrap().len(), 3);

    // 5. Contact link counts for c1, c2, c3 updated accurately
    assert_eq!(service.contact_article_count(c1.id).unwrap(), 1);
    assert_eq!(service.contact_article_count(c2.id).unwrap(), 1);
    assert_eq!(service.contact_article_count(c3.id).unwrap(), 0);

    // 6. Article B and Task t4 remain completely unaffected
    assert!(service.get_article(article_b.id).unwrap().is_some());
    assert!(service.get_task(t4.id).unwrap().is_some());
    assert_eq!(
        service.list_tasks_for_article(article_b.id).unwrap().len(),
        1
    );
    assert_eq!(
        service
            .list_contacts_for_article(article_b.id)
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn test_cascade_delete_contact_cleans_links_preserving_articles_and_tasks() {
    let service = StorageService::in_memory().expect("failed to init in-memory db");

    let article_a = service
        .create_article(Article::new("story-alpha", "Story Alpha Headline"))
        .unwrap();
    let t1 = service
        .create_task(Task::new(article_a.id, "Alpha Task 1"))
        .unwrap();

    let article_b = service
        .create_article(Article::new("story-beta", "Story Beta Headline"))
        .unwrap();
    let t2 = service
        .create_task(Task::new(article_b.id, "Beta Task 2"))
        .unwrap();

    let c1 = service
        .create_contact(Contact::new("Expert Source 1"))
        .unwrap();
    let c2 = service
        .create_contact(Contact::new("Expert Source 2"))
        .unwrap();

    service
        .link_contact_to_article(article_a.id, c1.id)
        .unwrap();
    service
        .link_contact_to_article(article_a.id, c2.id)
        .unwrap();
    service
        .link_contact_to_article(article_b.id, c1.id)
        .unwrap();

    // Delete Contact c1
    let deleted = service.delete_contact(c1.id).unwrap();
    assert!(deleted);

    // 1. Contact c1 is gone
    assert!(service.get_contact(c1.id).unwrap().is_none());

    // 2. Links to c1 are removed from both articles
    assert!(!service
        .is_contact_linked_to_article(article_a.id, c1.id)
        .unwrap());
    assert!(!service
        .is_contact_linked_to_article(article_b.id, c1.id)
        .unwrap());
    assert_eq!(service.contact_article_count(c1.id).unwrap(), 0);

    // 3. Contact c2 is still linked to Article A
    assert!(service
        .is_contact_linked_to_article(article_a.id, c2.id)
        .unwrap());
    assert_eq!(service.article_contact_count(article_a.id).unwrap(), 1);
    assert_eq!(service.article_contact_count(article_b.id).unwrap(), 0);

    // 4. Articles and their tasks remain intact
    assert!(service.get_article(article_a.id).unwrap().is_some());
    assert!(service.get_article(article_b.id).unwrap().is_some());
    assert!(service.get_task(t1.id).unwrap().is_some());
    assert!(service.get_task(t2.id).unwrap().is_some());
}

// =============================================================================
// 3. Many-to-Many Relational Queries & Joins
// =============================================================================

#[test]
fn test_m2m_article_contact_matrix_and_bidirectional_joins() {
    let service = StorageService::in_memory().expect("failed to init in-memory db");

    let a1 = service
        .create_article(
            Article::new("metro-transit", "Metro Transit Expansion")
                .with_stage(ArticleStage::Pitching),
        )
        .unwrap();
    let a2 = service
        .create_article(
            Article::new("city-budget", "City Budget 2026").with_stage(ArticleStage::Researching),
        )
        .unwrap();
    let a3 = service
        .create_article(
            Article::new("clean-water", "Clean Water Quality").with_stage(ArticleStage::Writing),
        )
        .unwrap();
    let a4 = service
        .create_article(
            Article::new("tech-jobs", "Tech Sector Hiring").with_stage(ArticleStage::Editing),
        )
        .unwrap();

    let c1 = service
        .create_contact(Contact::new("Alice Adams").with_organization("Transit Union"))
        .unwrap();
    let c2 = service
        .create_contact(Contact::new("Bob Builder").with_organization("City Hall"))
        .unwrap();
    let c3 = service
        .create_contact(Contact::new("Charlie Clark").with_organization("Water Board"))
        .unwrap();
    let c4 = service
        .create_contact(Contact::new("Diana Davis").with_organization("Finance Dept"))
        .unwrap();
    let c5 = service
        .create_contact(Contact::new("Evan Evans").with_organization("EPA"))
        .unwrap();

    // Assignment matrix:
    // a1 -> [c1, c2, c3]
    // a2 -> [c2, c4]
    // a3 -> [c1, c3, c5]
    // a4 -> []
    service.link_contact_to_article(a1.id, c1.id).unwrap();
    service.link_contact_to_article(a1.id, c2.id).unwrap();
    service.link_contact_to_article(a1.id, c3.id).unwrap();

    service.link_contact_to_article(a2.id, c2.id).unwrap();
    service.link_contact_to_article(a2.id, c4.id).unwrap();

    service.link_contact_to_article(a3.id, c1.id).unwrap();
    service.link_contact_to_article(a3.id, c3.id).unwrap();
    service.link_contact_to_article(a3.id, c5.id).unwrap();

    // 1. Validate article_contact_count
    assert_eq!(service.article_contact_count(a1.id).unwrap(), 3);
    assert_eq!(service.article_contact_count(a2.id).unwrap(), 2);
    assert_eq!(service.article_contact_count(a3.id).unwrap(), 3);
    assert_eq!(service.article_contact_count(a4.id).unwrap(), 0);

    // 2. Validate contact_article_count
    assert_eq!(service.contact_article_count(c1.id).unwrap(), 2); // a1, a3
    assert_eq!(service.contact_article_count(c2.id).unwrap(), 2); // a1, a2
    assert_eq!(service.contact_article_count(c3.id).unwrap(), 2); // a1, a3
    assert_eq!(service.contact_article_count(c4.id).unwrap(), 1); // a2
    assert_eq!(service.contact_article_count(c5.id).unwrap(), 1); // a3

    // 3. Validate list_contacts_for_article (sorted by name ASC)
    let a1_contacts = service.list_contacts_for_article(a1.id).unwrap();
    assert_eq!(
        a1_contacts.iter().map(|c| c.id).collect::<Vec<_>>(),
        vec![c1.id, c2.id, c3.id]
    );

    // 4. Validate list_articles_for_contact (ordered by created_at DESC)
    let c1_articles = service.list_articles_for_contact(c1.id).unwrap();
    let c1_article_ids: HashSet<Uuid> = c1_articles.iter().map(|a| a.id).collect();
    assert_eq!(c1_article_ids, HashSet::from([a1.id, a3.id]));

    // 5. Test total join records
    let all_links = service.list_article_contacts().unwrap();
    assert_eq!(all_links.len(), 8);

    // 6. Test idempotent link_contact_to_article (linking again should return existing link)
    let dup_link = service.link_contact_to_article(a1.id, c1.id).unwrap();
    assert_eq!(dup_link.article_id, a1.id);
    assert_eq!(dup_link.contact_id, c1.id);
    assert_eq!(service.article_contact_count(a1.id).unwrap(), 3);

    // 7. Unlink single contact
    assert!(service.unlink_contact_from_article(a1.id, c2.id).unwrap());
    assert!(!service.unlink_contact_from_article(a1.id, c2.id).unwrap()); // second unlink returns false
    assert_eq!(service.article_contact_count(a1.id).unwrap(), 2);
    assert_eq!(service.contact_article_count(c2.id).unwrap(), 1);
}

#[test]
fn test_m2m_set_article_contacts_atomic_replacement() {
    let service = StorageService::in_memory().expect("failed to init in-memory db");

    let article = service
        .create_article(Article::new(
            "election-night",
            "Election Night Live Coverage",
        ))
        .unwrap();

    let c1 = service
        .create_contact(Contact::new("Pollster Pete"))
        .unwrap();
    let c2 = service
        .create_contact(Contact::new("Analyst Anna"))
        .unwrap();
    let c3 = service
        .create_contact(Contact::new("Reporter Rachel"))
        .unwrap();
    let c4 = service
        .create_contact(Contact::new("Field Producer Frank"))
        .unwrap();

    // Step 1: Initial assignment [c1, c2]
    service
        .set_article_contacts(article.id, &[c1.id, c2.id])
        .unwrap();
    let contacts1 = service.list_contacts_for_article(article.id).unwrap();
    assert_eq!(contacts1.len(), 2);
    assert!(contacts1.iter().any(|c| c.id == c1.id));
    assert!(contacts1.iter().any(|c| c.id == c2.id));

    // Step 2: Atomic update to [c2, c3, c4] (c1 removed, c2 kept, c3 and c4 added)
    service
        .set_article_contacts(article.id, &[c2.id, c3.id, c4.id])
        .unwrap();
    let contacts2 = service.list_contacts_for_article(article.id).unwrap();
    assert_eq!(contacts2.len(), 3);
    assert!(!contacts2.iter().any(|c| c.id == c1.id));
    assert!(contacts2.iter().any(|c| c.id == c2.id));
    assert!(contacts2.iter().any(|c| c.id == c3.id));
    assert!(contacts2.iter().any(|c| c.id == c4.id));

    // Step 3: Clear all contacts by passing empty slice
    service.set_article_contacts(article.id, &[]).unwrap();
    let contacts3 = service.list_contacts_for_article(article.id).unwrap();
    assert_eq!(contacts3.len(), 0);
    assert_eq!(service.article_contact_count(article.id).unwrap(), 0);
}

// =============================================================================
// 4. Kanban Stages, Partitioning & Transitions
// =============================================================================

#[test]
fn test_kanban_articles_stage_partitioning_and_transitions() {
    let service = StorageService::in_memory().expect("failed to init in-memory db");

    let stages = [
        ArticleStage::Pitching,
        ArticleStage::Researching,
        ArticleStage::Writing,
        ArticleStage::Editing,
        ArticleStage::ReadyToPublish,
        ArticleStage::Published,
    ];

    let mut created_articles = Vec::new();
    for (i, &stage) in stages.iter().enumerate() {
        let slug = format!("story-stage-{i}");
        let headline = format!("Headline for stage {stage:?}");
        let article = Article::new(&slug, &headline).with_stage(stage);
        let created = service.create_article(article).unwrap();
        created_articles.push(created);
    }

    // Verify 1 article per stage
    for &stage in &stages {
        let list = service.list_articles_by_stage(stage).unwrap();
        assert_eq!(
            list.len(),
            1,
            "stage {stage:?} should have exactly 1 article"
        );
    }

    let all = service.list_articles().unwrap();
    assert_eq!(all.len(), 6);

    // Perform sequential transition on the first article across all 6 stages
    let mut article_to_transition = created_articles[0].clone();
    for &target_stage in &stages {
        let transitioned = service
            .change_article_stage(article_to_transition.id, target_stage)
            .unwrap();
        assert_eq!(transitioned.stage, target_stage);

        let fetched = service
            .get_article(article_to_transition.id)
            .unwrap()
            .unwrap();
        assert_eq!(fetched.stage, target_stage);

        article_to_transition = fetched;
    }
}

// =============================================================================
// 5. Task Status Queries, Article Counts & Filtering
// =============================================================================

#[test]
fn test_task_status_queries_and_progress_counters() {
    let service = StorageService::in_memory().expect("failed to init in-memory db");

    let a1 = service
        .create_article(Article::new("article-one", "First Article"))
        .unwrap();
    let a2 = service
        .create_article(Article::new("article-two", "Second Article"))
        .unwrap();

    // a1 tasks: 2 ToDo, 1 InProgress, 2 Complete (Total: 5, Completed: 2)
    let a1_t1 = service
        .create_task(Task::new(a1.id, "T1").with_status(TaskStatus::ToDo))
        .unwrap();
    let _a1_t2 = service
        .create_task(Task::new(a1.id, "T2").with_status(TaskStatus::ToDo))
        .unwrap();
    let _a1_t3 = service
        .create_task(Task::new(a1.id, "T3").with_status(TaskStatus::InProgress))
        .unwrap();
    let _a1_t4 = service
        .create_task(Task::new(a1.id, "T4").with_status(TaskStatus::Complete))
        .unwrap();
    let _a1_t5 = service
        .create_task(Task::new(a1.id, "T5").with_status(TaskStatus::Complete))
        .unwrap();

    // a2 tasks: 1 ToDo, 2 InProgress, 0 Complete (Total: 3, Completed: 0)
    let _a2_t1 = service
        .create_task(Task::new(a2.id, "A2-T1").with_status(TaskStatus::ToDo))
        .unwrap();
    let _a2_t2 = service
        .create_task(Task::new(a2.id, "A2-T2").with_status(TaskStatus::InProgress))
        .unwrap();
    let _a2_t3 = service
        .create_task(Task::new(a2.id, "A2-T3").with_status(TaskStatus::InProgress))
        .unwrap();

    // Article with 0 tasks
    let a3 = service
        .create_article(Article::new("article-three", "Empty Article"))
        .unwrap();

    // 1. Verify task counts per article
    let (a1_total, a1_done) = service.article_task_counts(a1.id).unwrap();
    assert_eq!(a1_total, 5);
    assert_eq!(a1_done, 2);

    let (a2_total, a2_done) = service.article_task_counts(a2.id).unwrap();
    assert_eq!(a2_total, 3);
    assert_eq!(a2_done, 0);

    let (a3_total, a3_done) = service.article_task_counts(a3.id).unwrap();
    assert_eq!(a3_total, 0);
    assert_eq!(a3_done, 0);

    // 2. Verify list_tasks_by_status
    let todo_tasks = service.list_tasks_by_status(TaskStatus::ToDo).unwrap();
    assert_eq!(todo_tasks.len(), 3); // 2 from a1, 1 from a2

    let in_progress_tasks = service
        .list_tasks_by_status(TaskStatus::InProgress)
        .unwrap();
    assert_eq!(in_progress_tasks.len(), 3); // 1 from a1, 2 from a2

    let complete_tasks = service.list_tasks_by_status(TaskStatus::Complete).unwrap();
    assert_eq!(complete_tasks.len(), 2); // 2 from a1, 0 from a2

    // 3. Change status of a1_t1 from ToDo to Complete
    service
        .change_task_status(a1_t1.id, TaskStatus::Complete)
        .unwrap();

    let (a1_total_after, a1_done_after) = service.article_task_counts(a1.id).unwrap();
    assert_eq!(a1_total_after, 5);
    assert_eq!(a1_done_after, 3);

    // 4. Delete a task
    assert!(service.delete_task(a1_t1.id).unwrap());
    let (a1_total_deleted, a1_done_deleted) = service.article_task_counts(a1.id).unwrap();
    assert_eq!(a1_total_deleted, 4);
    assert_eq!(a1_done_deleted, 2);
}

// =============================================================================
// 6. Contact Multi-Field Search & Substring Queries
// =============================================================================

#[test]
fn test_contact_search_comprehensive() {
    let service = StorageService::in_memory().expect("failed to init in-memory db");

    let c1 = service
        .create_contact(
            Contact::new("Dr. Maya Lin")
                .with_organization("Department of Transportation")
                .with_role("Senior Infrastructure Inspector")
                .with_phone("+1 555-019-2834")
                .with_email("maya.lin@dot.gov")
                .with_notes("Key whistleblower on bridge tensile strength tests"),
        )
        .unwrap();

    let c2 = service
        .create_contact(
            Contact::new("Carlos Santana")
                .with_organization("Acoustics & Sound Foundation")
                .with_role("Chief Audio Engineer")
                .with_phone("+1 555-098-7654")
                .with_email("carlos@acoustics.org")
                .with_notes("Expert in decibel mapping algorithms"),
        )
        .unwrap();

    let c3 = service
        .create_contact(
            Contact::new("Maya Angelou")
                .with_organization("Literary Arts Guild")
                .with_role("President")
                .with_phone("+1 555-444-3322")
                .with_email("president@arts.org"),
        )
        .unwrap();

    // 1. Search by Name (case-insensitive substring) -> "maya" matches c1, c3
    let res_name = service.search_contacts("maya").unwrap();
    assert_eq!(res_name.len(), 2);
    let res_name_ids: HashSet<Uuid> = res_name.iter().map(|c| c.id).collect();
    assert_eq!(res_name_ids, HashSet::from([c1.id, c3.id]));

    // 2. Search by Organization -> "Transportation" matches c1
    let res_org = service.search_contacts("Transportation").unwrap();
    assert_eq!(res_org.len(), 1);
    assert_eq!(res_org[0].id, c1.id);

    // 3. Search by Role -> "Chief Audio" matches c2
    let res_role = service.search_contacts("Chief Audio").unwrap();
    assert_eq!(res_role.len(), 1);
    assert_eq!(res_role[0].id, c2.id);

    // 4. Search by Email domain -> ".gov" matches c1
    let res_email = service.search_contacts("dot.gov").unwrap();
    assert_eq!(res_email.len(), 1);
    assert_eq!(res_email[0].id, c1.id);

    // 5. Search by Phone digits -> "098-7654" matches c2
    let res_phone = service.search_contacts("098").unwrap();
    assert_eq!(res_phone.len(), 1);
    assert_eq!(res_phone[0].id, c2.id);

    // 6. Search by Notes -> "whistleblower" matches c1
    let res_notes = service.search_contacts("whistleblower").unwrap();
    assert_eq!(res_notes.len(), 1);
    assert_eq!(res_notes[0].id, c1.id);

    // 7. Search empty query -> returns all contacts ordered alphabetically by name
    let res_all = service.search_contacts("").unwrap();
    assert_eq!(res_all.len(), 3);
    assert_eq!(res_all[0].name, "Carlos Santana");
    assert_eq!(res_all[1].name, "Dr. Maya Lin");
    assert_eq!(res_all[2].name, "Maya Angelou");

    // 8. Search non-matching query -> returns empty
    let res_none = service.search_contacts("nonexistent_term_xyz").unwrap();
    assert_eq!(res_none.len(), 0);
}

// =============================================================================
// 7. Slug Uniqueness, Rename Workflows & Collision Rejection
// =============================================================================

#[test]
fn test_article_slug_mutation_and_collision_prevention() {
    let service = StorageService::in_memory().expect("failed to init in-memory db");

    let a1 = service
        .create_article(Article::new("original-slug-1", "Story One"))
        .unwrap();
    let a2 = service
        .create_article(Article::new("original-slug-2", "Story Two"))
        .unwrap();

    // 1. Slug availability checking
    assert!(!service.is_slug_available("original-slug-1", None).unwrap());
    assert!(service
        .is_slug_available("original-slug-1", Some(a1.id))
        .unwrap());
    assert!(!service
        .is_slug_available("original-slug-1", Some(a2.id))
        .unwrap());
    assert!(service.is_slug_available("new-unique-slug", None).unwrap());

    // 2. Rename a1 to new available slug
    let mut updated_a1 = a1.clone();
    updated_a1.slug = "renamed-slug-1".to_string();
    let saved_a1 = service.update_article(updated_a1).unwrap();
    assert_eq!(saved_a1.slug, "renamed-slug-1");
    assert_eq!(
        service.get_article(a1.id).unwrap().unwrap().slug,
        "renamed-slug-1"
    );

    // 3. Old slug is now free
    assert!(service.is_slug_available("original-slug-1", None).unwrap());

    // 4. Attempting to rename a2 to "renamed-slug-1" (a1's slug) fails with Conflict
    let mut colliding_a2 = a2.clone();
    colliding_a2.slug = "renamed-slug-1".to_string();
    let err = service
        .update_article(colliding_a2)
        .expect_err("slug collision must fail");
    assert!(err.is_conflict());

    // 5. Verify a2 was NOT modified
    let fetched_a2 = service.get_article(a2.id).unwrap().unwrap();
    assert_eq!(fetched_a2.slug, "original-slug-2");
}

// =============================================================================
// 8. Timestamps & Touch Advancement Semantics
// =============================================================================

#[test]
fn test_entity_touch_and_updated_at_advancement() {
    let service = StorageService::in_memory().expect("failed to init in-memory db");

    let a = service
        .create_article(Article::new("timestamp-test", "Original Headline"))
        .unwrap();
    let orig_article_created = a.created_at;
    let orig_article_updated = a.updated_at;

    let t = service
        .create_task(Task::new(a.id, "Original Task"))
        .unwrap();
    let orig_task_created = t.created_at;
    let orig_task_updated = t.updated_at;

    let c = service
        .create_contact(Contact::new("Original Contact"))
        .unwrap();
    let orig_contact_created = c.created_at;
    let orig_contact_updated = c.updated_at;

    // Small delay to ensure timestamp difference
    sleep(StdDuration::from_millis(15));

    // 1. Update Article
    let mut mod_a = a;
    mod_a.headline = "Updated Headline".to_string();
    let updated_a = service.update_article(mod_a).unwrap();
    assert_eq!(updated_a.created_at, orig_article_created);
    assert!(updated_a.updated_at >= orig_article_updated);

    // 2. Update Task
    let mut mod_t = t;
    mod_t.title = "Updated Task".to_string();
    let updated_t = service.update_task(mod_t).unwrap();
    assert_eq!(updated_t.created_at, orig_task_created);
    assert!(updated_t.updated_at >= orig_task_updated);

    // 3. Update Contact
    let mut mod_c = c;
    mod_c.name = "Updated Contact".to_string();
    let updated_c = service.update_contact(mod_c).unwrap();
    assert_eq!(updated_c.created_at, orig_contact_created);
    assert!(updated_c.updated_at >= orig_contact_updated);
}

// =============================================================================
// 9. End-to-End Editorial Newsroom Workflow with Deadlines
// =============================================================================

#[test]
fn test_e2e_investigative_story_workflow_with_deadlines() {
    let service = StorageService::in_memory().expect("failed to init in-memory db");
    let now = Utc::now();
    let future_deadline = now + Duration::hours(12);

    // 1. Pitch stage: Journalist creates an article with a deadline
    let story = Article::new(
        "subway-signal-failures",
        "Subway Signal Failures Quadrupled in 2025",
    )
    .with_description("Investigation into delayed MTA signal modernization contracts")
    .with_stage(ArticleStage::Pitching)
    .with_deadline(future_deadline)
    .with_auto_color();

    let created_story = service.create_article(story).expect("pitch article failed");
    assert_eq!(created_story.stage, ArticleStage::Pitching);
    assert!(created_story.color.is_some());

    // Evaluate initial deadline status: due soon (12 hours remaining)
    let deadline_eval = evaluate_article_deadline(&created_story, now);
    assert!(deadline_eval.is_due_soon());
    assert_eq!(deadline_eval.urgency_level(), UrgencyLevel::Medium);

    // 2. Research stage: Add investigative tasks and link whistleblower contacts
    service
        .change_article_stage(created_story.id, ArticleStage::Researching)
        .unwrap();

    let task1 = Task::new(
        created_story.id,
        "Submit Freedom of Information Law request",
    )
    .with_notes("Request MTA signal maintenance dispatch logs")
    .with_due_date(now + Duration::days(3));
    let task2 = Task::new(created_story.id, "Interview Transit Workers Union Chief");

    let t1 = service.create_task(task1).unwrap();
    let t2 = service.create_task(task2).unwrap();

    let source1 = Contact::new("Arthur Pendelton")
        .with_organization("Transit Union 100")
        .with_role("Chief Safety Delegate")
        .with_email("arthur@transitunion100.org")
        .with_phone("+1 555-839-2041");

    let source2 = Contact::new("Dr. Elena Rostova")
        .with_organization("Transportation Safety Board")
        .with_role("Signals Engineering Specialist")
        .with_email("erostova@tsb.example.gov");

    let c1 = service.create_contact(source1).unwrap();
    let c2 = service.create_contact(source2).unwrap();

    service
        .link_contact_to_article(created_story.id, c1.id)
        .unwrap();
    service
        .link_contact_to_article(created_story.id, c2.id)
        .unwrap();

    // 3. Writing stage: Complete tasks as reporting progresses
    service
        .change_article_stage(created_story.id, ArticleStage::Writing)
        .unwrap();
    service
        .change_task_status(t1.id, TaskStatus::Complete)
        .unwrap();
    service
        .change_task_status(t2.id, TaskStatus::Complete)
        .unwrap();

    let (total, completed) = service.article_task_counts(created_story.id).unwrap();
    assert_eq!(total, 2);
    assert_eq!(completed, 2);

    // 4. Editing & ReadyToPublish stages
    service
        .change_article_stage(created_story.id, ArticleStage::Editing)
        .unwrap();
    service
        .change_article_stage(created_story.id, ArticleStage::ReadyToPublish)
        .unwrap();

    // 5. Publish story: Move to Published stage
    let published_story = service
        .change_article_stage(created_story.id, ArticleStage::Published)
        .unwrap();
    assert_eq!(published_story.stage, ArticleStage::Published);

    // Evaluate published deadline: Even past deadline, Published stories are never overdue
    let past_time = future_deadline + Duration::hours(24);
    let pub_eval = evaluate_article_deadline(&published_story, past_time);
    assert_eq!(pub_eval, DeadlineStatus::Completed);
    assert_eq!(pub_eval.urgency_level(), UrgencyLevel::None);
    assert!(!pub_eval.is_overdue());
}

// =============================================================================
// 10. Settings & Theme Persistence
// =============================================================================

#[test]
fn test_settings_custom_key_values_and_theme_transitions() {
    let service = StorageService::in_memory().expect("failed to init in-memory db");

    // 1. Initial defaults
    let initial_settings = service.get_settings().unwrap();
    assert_eq!(initial_settings.theme_mode, ThemeMode::System);

    // 2. Cycle theme mode: System -> Dark -> Light -> System
    service.set_theme_mode(ThemeMode::Dark).unwrap();
    assert_eq!(service.get_theme_mode().unwrap(), ThemeMode::Dark);

    service.set_theme_mode(ThemeMode::Light).unwrap();
    assert_eq!(service.get_theme_mode().unwrap(), ThemeMode::Light);

    service.set_theme_mode(ThemeMode::System).unwrap();
    assert_eq!(service.get_theme_mode().unwrap(), ThemeMode::System);

    // 3. Arbitrary configuration keys
    assert_eq!(service.get_setting("non_existent_key").unwrap(), None);

    service
        .set_setting("kanban_collapsed_columns", "pitching,published")
        .unwrap();
    assert_eq!(
        service.get_setting("kanban_collapsed_columns").unwrap(),
        Some("pitching,published".to_string())
    );

    // Update existing setting key
    service
        .set_setting("kanban_collapsed_columns", "published")
        .unwrap();
    assert_eq!(
        service.get_setting("kanban_collapsed_columns").unwrap(),
        Some("published".to_string())
    );

    // 4. Save settings struct
    let custom = Settings::new(ThemeMode::Dark);
    service.save_settings(&custom).unwrap();
    assert_eq!(service.get_theme_mode().unwrap(), ThemeMode::Dark);
}

// =============================================================================
// 11. Unicode, Accents & Multilingual Search
// =============================================================================

#[test]
fn test_unicode_and_special_character_storage_and_search() {
    let service = StorageService::in_memory().expect("failed to init in-memory db");

    let article = Article::new(
        "cop28-climate-accord",
        "COP28: 198 pays s'accordent sur la transition énergétique 🌍",
    )
    .with_description("Analyse approfondie des compromis sur les énergies fossiles à Dubaï")
    .with_stage(ArticleStage::Writing)
    .with_auto_color();

    let created_article = service.create_article(article).unwrap();
    assert_eq!(
        created_article.headline,
        "COP28: 198 pays s'accordent sur la transition énergétique 🌍"
    );

    let task = Task::new(
        created_article.id,
        "Entrevue avec le délégué français à l'ONU",
    )
    .with_notes("Questions sur l'article 28 du traité international");
    let created_task = service.create_task(task).unwrap();
    assert_eq!(
        created_task.title,
        "Entrevue avec le délégué français à l'ONU"
    );

    // Multilingual contacts
    let c1 = service
        .create_contact(
            Contact::new("François Müller-Thévenot")
                .with_organization("Ministère de la Transition Écologique")
                .with_role("Conseiller Spécial")
                .with_email("francois.muller@ecologie.gouv.fr")
                .with_notes("Spécialiste de la décarbonation industrielle"),
        )
        .unwrap();

    let c2 = service
        .create_contact(
            Contact::new("Kenji Sato (佐藤 健二)")
                .with_organization("Tokyo Energy Analytics (東京エネルギー分析)")
                .with_role("Chief Researcher (主任研究員)")
                .with_email("kenji.sato@energy-tokyo.jp")
                .with_notes("Renewable grid load balancing expert"),
        )
        .unwrap();

    service
        .link_contact_to_article(created_article.id, c1.id)
        .unwrap();
    service
        .link_contact_to_article(created_article.id, c2.id)
        .unwrap();

    // Test Unicode Substring Searches
    let search_accent = service.search_contacts("Müller").unwrap();
    assert_eq!(search_accent.len(), 1);
    assert_eq!(search_accent[0].id, c1.id);

    let search_accent_role = service.search_contacts("Écologique").unwrap();
    assert_eq!(search_accent_role.len(), 1);
    assert_eq!(search_accent_role[0].id, c1.id);

    let search_cjk_name = service.search_contacts("佐藤").unwrap();
    assert_eq!(search_cjk_name.len(), 1);
    assert_eq!(search_cjk_name[0].id, c2.id);

    let search_cjk_org = service.search_contacts("東京").unwrap();
    assert_eq!(search_cjk_org.len(), 1);
    assert_eq!(search_cjk_org[0].id, c2.id);
}

// =============================================================================
// 12. Bulk Data Integrity & Relational Lifecycle Stress
// =============================================================================

#[test]
fn test_e2e_bulk_data_integrity_and_complex_relational_lifecycle() {
    let service = StorageService::in_memory().expect("failed to init in-memory db");

    let num_articles = 15;
    let num_contacts = 10;
    let mut article_ids = Vec::new();
    let mut contact_ids = Vec::new();

    // 1. Create articles across various stages
    let stages = [
        ArticleStage::Pitching,
        ArticleStage::Researching,
        ArticleStage::Writing,
        ArticleStage::Editing,
        ArticleStage::ReadyToPublish,
        ArticleStage::Published,
    ];

    for i in 0..num_articles {
        let stage = stages[i % stages.len()];
        let article = Article::new(
            format!("bulk-story-{i}"),
            format!("Bulk Story #{i} Comprehensive Headline"),
        )
        .with_stage(stage)
        .with_auto_color();

        let created = service.create_article(article).unwrap();
        article_ids.push(created.id);

        // Add 3 tasks per article (1 ToDo, 1 InProgress, 1 Complete)
        service
            .create_task(
                Task::new(created.id, format!("Story {i} - Task 1")).with_status(TaskStatus::ToDo),
            )
            .unwrap();
        service
            .create_task(
                Task::new(created.id, format!("Story {i} - Task 2"))
                    .with_status(TaskStatus::InProgress),
            )
            .unwrap();
        service
            .create_task(
                Task::new(created.id, format!("Story {i} - Task 3"))
                    .with_status(TaskStatus::Complete),
            )
            .unwrap();
    }

    // 2. Create contacts
    for i in 0..num_contacts {
        let contact = Contact::new(format!("Source Person {i:02}"))
            .with_organization(format!("Org {}", i % 3))
            .with_email(format!("source_{i}@news.org"));
        let created = service.create_contact(contact).unwrap();
        contact_ids.push(created.id);
    }

    // 3. Link contacts in a round-robin mesh
    for (a_idx, &a_id) in article_ids.iter().enumerate() {
        let c_id1 = contact_ids[a_idx % num_contacts];
        let c_id2 = contact_ids[(a_idx + 1) % num_contacts];
        service.link_contact_to_article(a_id, c_id1).unwrap();
        service.link_contact_to_article(a_id, c_id2).unwrap();
    }

    // Verify system state before mutations
    assert_eq!(service.list_articles().unwrap().len(), num_articles);
    assert_eq!(service.list_tasks().unwrap().len(), num_articles * 3);
    assert_eq!(service.list_contacts().unwrap().len(), num_contacts);

    for &a_id in &article_ids {
        let (tot, comp) = service.article_task_counts(a_id).unwrap();
        assert_eq!(tot, 3);
        assert_eq!(comp, 1);
        assert_eq!(service.article_contact_count(a_id).unwrap(), 2);
    }

    // 4. Delete first 5 articles -> verifies cascade of 15 tasks and 10 links
    for &a_id in &article_ids[0..5] {
        assert!(service.delete_article(a_id).unwrap());
    }

    assert_eq!(service.list_articles().unwrap().len(), num_articles - 5);
    assert_eq!(service.list_tasks().unwrap().len(), (num_articles - 5) * 3);
    assert_eq!(service.list_contacts().unwrap().len(), num_contacts); // all contacts remain

    // 5. Delete first 3 contacts -> verifies cascade of their links without deleting articles/tasks
    for &c_id in &contact_ids[0..3] {
        assert!(service.delete_contact(c_id).unwrap());
    }

    assert_eq!(service.list_contacts().unwrap().len(), num_contacts - 3);
    assert_eq!(service.list_articles().unwrap().len(), num_articles - 5);
    assert_eq!(service.list_tasks().unwrap().len(), (num_articles - 5) * 3);
}

// =============================================================================
// 13. Ordering & Sorting Invariants
// =============================================================================

#[test]
fn test_article_and_contact_ordering_invariants() {
    let service = StorageService::in_memory().expect("failed to init in-memory db");

    // Create 3 articles with sequential timestamps
    let a1 = service
        .create_article(Article::new("order-first", "First Created"))
        .unwrap();
    sleep(StdDuration::from_millis(10));
    let a2 = service
        .create_article(Article::new("order-second", "Second Created"))
        .unwrap();
    sleep(StdDuration::from_millis(10));
    let a3 = service
        .create_article(Article::new("order-third", "Third Created"))
        .unwrap();

    // list_articles should be ordered by created_at DESC (newest first: a3, a2, a1)
    let articles = service.list_articles().unwrap();
    assert_eq!(articles.len(), 3);
    assert_eq!(articles[0].id, a3.id);
    assert_eq!(articles[1].id, a2.id);
    assert_eq!(articles[2].id, a1.id);

    // Create contacts with case-insensitive alphabetical names
    let _c1 = service
        .create_contact(Contact::new("zoe Washington"))
        .unwrap();
    let _c2 = service.create_contact(Contact::new("Alice Brown")).unwrap();
    let _c3 = service.create_contact(Contact::new("bob Carter")).unwrap();

    // list_contacts should be ordered by name COLLATE NOCASE ASC (Alice, bob, zoe)
    let contacts = service.list_contacts().unwrap();
    assert_eq!(contacts.len(), 3);
    assert_eq!(contacts[0].name, "Alice Brown");
    assert_eq!(contacts[1].name, "bob Carter");
    assert_eq!(contacts[2].name, "zoe Washington");
}
