//! Comprehensive state machine, domain workflow, and model transition tests for `newsjournal-core`.

use std::collections::HashSet;
use std::str::FromStr;

use chrono::{Duration, TimeZone, Utc};
use newsjournal_core::validation::ValidationError;
use newsjournal_core::*;
use uuid::Uuid;

#[test]
fn test_article_stage_state_machine_transitions() {
    let stages = ArticleStage::all();
    assert_eq!(stages.len(), 6);

    // Forward progression verification
    assert_eq!(
        ArticleStage::Pitching.next_stage(),
        Some(ArticleStage::Researching)
    );
    assert_eq!(
        ArticleStage::Researching.next_stage(),
        Some(ArticleStage::Writing)
    );
    assert_eq!(
        ArticleStage::Writing.next_stage(),
        Some(ArticleStage::Editing)
    );
    assert_eq!(
        ArticleStage::Editing.next_stage(),
        Some(ArticleStage::ReadyToPublish)
    );
    assert_eq!(
        ArticleStage::ReadyToPublish.next_stage(),
        Some(ArticleStage::Published)
    );
    assert_eq!(ArticleStage::Published.next_stage(), None);

    // Backward progression verification
    assert_eq!(ArticleStage::Pitching.prev_stage(), None);
    assert_eq!(
        ArticleStage::Researching.prev_stage(),
        Some(ArticleStage::Pitching)
    );
    assert_eq!(
        ArticleStage::Writing.prev_stage(),
        Some(ArticleStage::Researching)
    );
    assert_eq!(
        ArticleStage::Editing.prev_stage(),
        Some(ArticleStage::Writing)
    );
    assert_eq!(
        ArticleStage::ReadyToPublish.prev_stage(),
        Some(ArticleStage::Editing)
    );
    assert_eq!(
        ArticleStage::Published.prev_stage(),
        Some(ArticleStage::ReadyToPublish)
    );

    // Bidirectional consistency
    for &stage in stages {
        if let Some(next) = stage.next_stage() {
            assert_eq!(
                next.prev_stage(),
                Some(stage),
                "broken backward link from {next:?} to {stage:?}"
            );
        }
        if let Some(prev) = stage.prev_stage() {
            assert_eq!(
                prev.next_stage(),
                Some(stage),
                "broken forward link from {prev:?} to {stage:?}"
            );
        }
    }

    // Active vs Published predicates
    for &stage in stages {
        if stage == ArticleStage::Published {
            assert!(stage.is_published());
            assert!(!stage.is_active());
        } else {
            assert!(!stage.is_published());
            assert!(stage.is_active());
        }
    }
}

#[test]
fn test_article_stage_parsing_and_string_formats() {
    // Canonical slugs
    assert_eq!(ArticleStage::Pitching.as_str(), "pitching");
    assert_eq!(ArticleStage::Researching.as_str(), "researching");
    assert_eq!(ArticleStage::Writing.as_str(), "writing");
    assert_eq!(ArticleStage::Editing.as_str(), "editing");
    assert_eq!(ArticleStage::ReadyToPublish.as_str(), "ready_to_publish");
    assert_eq!(ArticleStage::Published.as_str(), "published");

    // Display names
    assert_eq!(ArticleStage::Pitching.display_name(), "Pitching");
    assert_eq!(ArticleStage::Researching.display_name(), "Researching");
    assert_eq!(ArticleStage::Writing.display_name(), "Writing");
    assert_eq!(ArticleStage::Editing.display_name(), "Editing");
    assert_eq!(
        ArticleStage::ReadyToPublish.display_name(),
        "Ready to Publish"
    );
    assert_eq!(ArticleStage::Published.display_name(), "Published");

    // Display trait
    assert_eq!(
        format!("{}", ArticleStage::ReadyToPublish),
        "Ready to Publish"
    );

    // FromStr parsing with aliases and case insensitivity
    let valid_parse_cases = [
        ("pitching", ArticleStage::Pitching),
        ("PITCH", ArticleStage::Pitching),
        ("  researching  ", ArticleStage::Researching),
        ("research", ArticleStage::Researching),
        ("Writing", ArticleStage::Writing),
        ("write", ArticleStage::Writing),
        ("editing", ArticleStage::Editing),
        ("edit", ArticleStage::Editing),
        ("ready_to_publish", ArticleStage::ReadyToPublish),
        ("Ready-To-Publish", ArticleStage::ReadyToPublish),
        ("ready to publish", ArticleStage::ReadyToPublish),
        ("ready", ArticleStage::ReadyToPublish),
        ("published", ArticleStage::Published),
        ("PUBLISH", ArticleStage::Published),
    ];

    for (input, expected) in valid_parse_cases {
        assert_eq!(
            ArticleStage::from_str(input).unwrap(),
            expected,
            "failed to parse '{input}'"
        );
    }

    // Invalid parse cases
    assert!(ArticleStage::from_str("archived").is_err());
    assert!(ArticleStage::from_str("").is_err());
    assert!(ArticleStage::from_str("123").is_err());
}

#[test]
fn test_task_status_state_machine_transitions() {
    let statuses = TaskStatus::all();
    assert_eq!(statuses.len(), 3);

    // Forward progression
    assert_eq!(TaskStatus::ToDo.next_status(), Some(TaskStatus::InProgress));
    assert_eq!(
        TaskStatus::InProgress.next_status(),
        Some(TaskStatus::Complete)
    );
    assert_eq!(TaskStatus::Complete.next_status(), None);

    // Backward progression
    assert_eq!(TaskStatus::ToDo.prev_status(), None);
    assert_eq!(TaskStatus::InProgress.prev_status(), Some(TaskStatus::ToDo));
    assert_eq!(
        TaskStatus::Complete.prev_status(),
        Some(TaskStatus::InProgress)
    );

    // Bidirectional consistency
    for &status in statuses {
        if let Some(next) = status.next_status() {
            assert_eq!(next.prev_status(), Some(status));
        }
        if let Some(prev) = status.prev_status() {
            assert_eq!(prev.next_status(), Some(status));
        }
    }

    // Complete predicate
    assert!(!TaskStatus::ToDo.is_complete());
    assert!(!TaskStatus::InProgress.is_complete());
    assert!(TaskStatus::Complete.is_complete());
}

#[test]
fn test_task_status_parsing_and_string_formats() {
    assert_eq!(TaskStatus::ToDo.as_str(), "to_do");
    assert_eq!(TaskStatus::InProgress.as_str(), "in_progress");
    assert_eq!(TaskStatus::Complete.as_str(), "complete");

    assert_eq!(TaskStatus::ToDo.display_name(), "To-Do");
    assert_eq!(TaskStatus::InProgress.display_name(), "In Progress");
    assert_eq!(TaskStatus::Complete.display_name(), "Complete");

    let parse_cases = [
        ("to_do", TaskStatus::ToDo),
        ("todo", TaskStatus::ToDo),
        ("ToDo", TaskStatus::ToDo),
        ("in_progress", TaskStatus::InProgress),
        ("inprogress", TaskStatus::InProgress),
        ("doing", TaskStatus::InProgress),
        ("In-Progress", TaskStatus::InProgress),
        ("complete", TaskStatus::Complete),
        ("completed", TaskStatus::Complete),
        ("done", TaskStatus::Complete),
    ];

    for (input, expected) in parse_cases {
        assert_eq!(TaskStatus::from_str(input).unwrap(), expected);
    }

    assert!(TaskStatus::from_str("cancelled").is_err());
    assert!(TaskStatus::from_str("").is_err());
}

#[test]
fn test_theme_mode_models_and_parsing() {
    let modes = ThemeMode::all();
    assert_eq!(modes.len(), 3);
    assert_eq!(ThemeMode::default(), ThemeMode::System);

    assert_eq!(ThemeMode::System.as_str(), "system");
    assert_eq!(ThemeMode::Light.as_str(), "light");
    assert_eq!(ThemeMode::Dark.as_str(), "dark");

    assert_eq!(ThemeMode::System.display_name(), "System");
    assert_eq!(ThemeMode::Light.display_name(), "Light");
    assert_eq!(ThemeMode::Dark.display_name(), "Dark");

    assert_eq!(ThemeMode::from_str("system").unwrap(), ThemeMode::System);
    assert_eq!(ThemeMode::from_str("auto").unwrap(), ThemeMode::System);
    assert_eq!(ThemeMode::from_str("os").unwrap(), ThemeMode::System);
    assert_eq!(ThemeMode::from_str("light").unwrap(), ThemeMode::Light);
    assert_eq!(ThemeMode::from_str("dark").unwrap(), ThemeMode::Dark);
    assert!(ThemeMode::from_str("high-contrast").is_err());

    // Settings struct mutation
    let mut settings = Settings::default();
    assert_eq!(settings.theme_mode, ThemeMode::System);
    let original_updated_at = settings.updated_at;

    settings.set_theme_mode(ThemeMode::Dark);
    assert_eq!(settings.theme_mode, ThemeMode::Dark);
    assert!(settings.updated_at >= original_updated_at);
    assert!(settings.validate().is_ok());

    let custom_settings = Settings::new(ThemeMode::Light);
    assert_eq!(custom_settings.theme_mode, ThemeMode::Light);
}

#[test]
fn test_article_mutations_and_builder_workflow() {
    let mut article = Article::new("city-council-budget", "Budget Hearing Scheduled");
    assert_eq!(article.stage, ArticleStage::Pitching);
    assert!(article.color.is_none());
    assert!(article.description.is_none());
    assert!(article.deadline.is_none());

    let created_at = article.created_at;
    let initial_updated_at = article.updated_at;

    // Transition stage
    article.set_stage(ArticleStage::Researching);
    assert_eq!(article.stage, ArticleStage::Researching);
    assert_eq!(article.created_at, created_at);
    assert!(article.updated_at >= initial_updated_at);

    // Fluent builder
    let custom_id = Uuid::new_v4();
    let fixed_time = Utc.with_ymd_and_hms(2026, 8, 22, 18, 0, 0).unwrap();

    let full_article = Article::builder("election-2026", "General Election Coverage")
        .id(custom_id)
        .description("Comprehensive profiles of all 12 mayoral candidates.")
        .stage(ArticleStage::Writing)
        .deadline(fixed_time)
        .color("#2563EB")
        .created_at(fixed_time - Duration::days(5))
        .updated_at(fixed_time)
        .build_validated()
        .expect("article should be valid");

    assert_eq!(full_article.id, custom_id);
    assert_eq!(full_article.slug, "election-2026");
    assert_eq!(full_article.headline, "General Election Coverage");
    assert_eq!(
        full_article.description.as_deref(),
        Some("Comprehensive profiles of all 12 mayoral candidates.")
    );
    assert_eq!(full_article.stage, ArticleStage::Writing);
    assert_eq!(full_article.deadline, Some(fixed_time));
    assert_eq!(full_article.color.as_deref(), Some("#2563EB"));
    assert_eq!(full_article.created_at, fixed_time - Duration::days(5));
    assert_eq!(full_article.updated_at, fixed_time);

    // Negative validation tests
    let invalid_slug_art =
        Article::builder("INVALID SLUG WITH SPACES", "Headline").build_validated();
    assert!(matches!(
        invalid_slug_art,
        Err(ValidationError::InvalidSlug { .. })
    ));

    let empty_headline_art = Article::builder("valid-slug", "").build_validated();
    assert!(matches!(
        empty_headline_art,
        Err(ValidationError::EmptyField { field: "headline" })
    ));

    let invalid_color_art = Article::builder("valid-slug", "Headline")
        .color("not-a-hex-code")
        .build_validated();
    assert!(matches!(
        invalid_color_art,
        Err(ValidationError::InvalidColor { .. })
    ));
}

#[test]
fn test_task_mutations_and_builder_workflow() {
    let article_id = Uuid::new_v4();
    let mut task = Task::new(article_id, "Request FOIA Documents");
    assert_eq!(task.article_id, article_id);
    assert_eq!(task.status, TaskStatus::ToDo);

    let initial_updated_at = task.updated_at;
    task.set_status(TaskStatus::InProgress);
    assert_eq!(task.status, TaskStatus::InProgress);
    assert!(task.updated_at >= initial_updated_at);

    let custom_id = Uuid::new_v4();
    let due_date = Utc.with_ymd_and_hms(2026, 8, 25, 12, 0, 0).unwrap();

    let full_task = Task::builder(article_id, "Interview Whistleblower")
        .id(custom_id)
        .notes("Secure Signal communication only")
        .status(TaskStatus::ToDo)
        .due_date(due_date)
        .created_at(due_date - Duration::days(2))
        .updated_at(due_date - Duration::days(1))
        .build_validated()
        .expect("task should be valid");

    assert_eq!(full_task.id, custom_id);
    assert_eq!(full_task.article_id, article_id);
    assert_eq!(full_task.title, "Interview Whistleblower");
    assert_eq!(
        full_task.notes.as_deref(),
        Some("Secure Signal communication only")
    );
    assert_eq!(full_task.due_date, Some(due_date));
    assert_eq!(full_task.status, TaskStatus::ToDo);

    let invalid_task = Task::builder(article_id, "   ").build_validated();
    assert!(matches!(
        invalid_task,
        Err(ValidationError::EmptyField { field: "title" })
    ));
}

#[test]
fn test_contact_mutations_and_builder_workflow() {
    let custom_id = Uuid::new_v4();
    let fixed_time = Utc.with_ymd_and_hms(2026, 8, 20, 10, 0, 0).unwrap();

    let contact = Contact::builder("Deep Throat")
        .id(custom_id)
        .organization("Federal Agency")
        .role("Anonymous Source")
        .email("informant@whistleblower.org")
        .phone("+1 (555) 019-2834")
        .notes("Meets in underground parking garage")
        .created_at(fixed_time)
        .updated_at(fixed_time)
        .build_validated()
        .expect("contact should be valid");

    assert_eq!(contact.id, custom_id);
    assert_eq!(contact.name, "Deep Throat");
    assert_eq!(contact.organization.as_deref(), Some("Federal Agency"));
    assert_eq!(contact.role.as_deref(), Some("Anonymous Source"));
    assert_eq!(
        contact.email.as_deref(),
        Some("informant@whistleblower.org")
    );
    assert_eq!(contact.phone.as_deref(), Some("+1 (555) 019-2834"));
    assert_eq!(
        contact.notes.as_deref(),
        Some("Meets in underground parking garage")
    );

    let empty_name_contact = Contact::builder("").build_validated();
    assert!(matches!(
        empty_name_contact,
        Err(ValidationError::EmptyField { field: "name" })
    ));

    let invalid_email_contact = Contact::builder("Valid Name")
        .email("not-an-email")
        .build_validated();
    assert!(matches!(
        invalid_email_contact,
        Err(ValidationError::InvalidEmail { .. })
    ));

    let invalid_phone_contact = Contact::builder("Valid Name")
        .phone("123")
        .build_validated();
    assert!(matches!(
        invalid_phone_contact,
        Err(ValidationError::InvalidPhone { .. })
    ));
}

#[test]
fn test_article_contact_join_model_properties() {
    let article_id = Uuid::new_v4();
    let contact_id = Uuid::new_v4();
    let fixed_time = Utc.with_ymd_and_hms(2026, 8, 22, 14, 30, 0).unwrap();

    let link1 = ArticleContact::with_timestamp(article_id, contact_id, fixed_time);
    assert_eq!(link1.article_id, article_id);
    assert_eq!(link1.contact_id, contact_id);
    assert_eq!(link1.created_at, fixed_time);

    let link2 = ArticleContact::new(article_id, contact_id);
    assert_eq!(link2.article_id, article_id);
    assert_eq!(link2.contact_id, contact_id);

    // Hash and set deduplication
    let mut set = HashSet::new();
    set.insert(link1.clone());
    assert!(set.contains(&link1));
}

#[test]
fn test_comprehensive_json_serde_roundtrips() {
    // 1. Article Serde
    let article = Article::builder("deep-dive-story", "Deep Dive Headline")
        .description("Detailed description")
        .stage(ArticleStage::Editing)
        .deadline(Utc::now() + Duration::days(2))
        .color("#E11D48")
        .build();
    let art_json = serde_json::to_string(&article).expect("article serialization failed");
    let art_deserialized: Article =
        serde_json::from_str(&art_json).expect("article deserialization failed");
    assert_eq!(article, art_deserialized);

    // 2. Task Serde
    let task = Task::builder(article.id, "Follow up with sources")
        .notes("Important notes")
        .status(TaskStatus::InProgress)
        .due_date(Utc::now() + Duration::hours(12))
        .build();
    let task_json = serde_json::to_string(&task).expect("task serialization failed");
    let task_deserialized: Task =
        serde_json::from_str(&task_json).expect("task deserialization failed");
    assert_eq!(task, task_deserialized);

    // 3. Contact Serde
    let contact = Contact::builder("Bob Woodward")
        .organization("The Washington Post")
        .role("Investigative Journalist")
        .email("bob@washpost.com")
        .phone("+1 (555) 234-5678")
        .notes("Legendary reporter")
        .build();
    let contact_json = serde_json::to_string(&contact).expect("contact serialization failed");
    let contact_deserialized: Contact =
        serde_json::from_str(&contact_json).expect("contact deserialization failed");
    assert_eq!(contact, contact_deserialized);

    // 4. ArticleContact Serde
    let join = ArticleContact::new(article.id, contact.id);
    let join_json = serde_json::to_string(&join).expect("join serialization failed");
    let join_deserialized: ArticleContact =
        serde_json::from_str(&join_json).expect("join deserialization failed");
    assert_eq!(join, join_deserialized);

    // 5. Settings Serde
    let settings = Settings::new(ThemeMode::Dark);
    let settings_json = serde_json::to_string(&settings).expect("settings serialization failed");
    let settings_deserialized: Settings =
        serde_json::from_str(&settings_json).expect("settings deserialization failed");
    assert_eq!(settings, settings_deserialized);
}
