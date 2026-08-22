//! Domain models for the NewsJournal application.

pub mod article;
pub mod article_contact;
pub mod contact;
pub mod settings;
pub mod task;

pub use article::{Article, ArticleBuilder, ArticleStage};
pub use article_contact::ArticleContact;
pub use contact::{Contact, ContactBuilder};
pub use settings::{Settings, ThemeMode};
pub use task::{Task, TaskBuilder, TaskStatus};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    #[test]
    fn test_article_stage_workflow_and_ordering() {
        let stages = ArticleStage::all();
        assert_eq!(stages.len(), 6);
        assert_eq!(
            stages,
            &[
                ArticleStage::Pitching,
                ArticleStage::Researching,
                ArticleStage::Writing,
                ArticleStage::Editing,
                ArticleStage::ReadyToPublish,
                ArticleStage::Published,
            ]
        );

        assert_eq!(ArticleStage::default(), ArticleStage::Pitching);
        assert!(ArticleStage::Pitching.is_active());
        assert!(!ArticleStage::Pitching.is_published());
        assert!(ArticleStage::Published.is_published());
        assert!(!ArticleStage::Published.is_active());

        // Forward progression
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

        // Backward progression
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
    }

    #[test]
    fn test_article_stage_display_and_from_str() {
        assert_eq!(ArticleStage::Pitching.display_name(), "Pitching");
        assert_eq!(ArticleStage::Researching.display_name(), "Researching");
        assert_eq!(ArticleStage::Writing.display_name(), "Writing");
        assert_eq!(ArticleStage::Editing.display_name(), "Editing");
        assert_eq!(
            ArticleStage::ReadyToPublish.display_name(),
            "Ready to Publish"
        );
        assert_eq!(ArticleStage::Published.display_name(), "Published");

        assert_eq!(format!("{}", ArticleStage::Pitching), "Pitching");
        assert_eq!(
            format!("{}", ArticleStage::ReadyToPublish),
            "Ready to Publish"
        );

        assert_eq!(
            "pitching".parse::<ArticleStage>().unwrap(),
            ArticleStage::Pitching
        );
        assert_eq!(
            "PITCH".parse::<ArticleStage>().unwrap(),
            ArticleStage::Pitching
        );
        assert_eq!(
            "researching".parse::<ArticleStage>().unwrap(),
            ArticleStage::Researching
        );
        assert_eq!(
            "writing".parse::<ArticleStage>().unwrap(),
            ArticleStage::Writing
        );
        assert_eq!(
            "editing".parse::<ArticleStage>().unwrap(),
            ArticleStage::Editing
        );
        assert_eq!(
            "ready_to_publish".parse::<ArticleStage>().unwrap(),
            ArticleStage::ReadyToPublish
        );
        assert_eq!(
            "ready-to-publish".parse::<ArticleStage>().unwrap(),
            ArticleStage::ReadyToPublish
        );
        assert_eq!(
            "Ready to Publish".parse::<ArticleStage>().unwrap(),
            ArticleStage::ReadyToPublish
        );
        assert_eq!(
            "published".parse::<ArticleStage>().unwrap(),
            ArticleStage::Published
        );

        assert!("invalid_stage".parse::<ArticleStage>().is_err());
    }

    #[test]
    fn test_article_stage_serde_roundtrip() {
        for stage in ArticleStage::all() {
            let json = serde_json::to_string(stage).expect("serialization failed");
            let deserialized: ArticleStage =
                serde_json::from_str(&json).expect("deserialization failed");
            assert_eq!(*stage, deserialized);
        }
    }

    #[test]
    fn test_article_creation_and_builder() {
        let slug = "city-council-investigation";
        let headline = "City Council Approves Transit Expansion";
        let article = Article::new(slug, headline);

        assert_eq!(article.slug, slug);
        assert_eq!(article.headline, headline);
        assert_eq!(article.stage, ArticleStage::Pitching);
        assert!(article.description.is_none());
        assert!(article.deadline.is_none());
        assert!(article.color.is_none());

        let custom_id = Uuid::new_v4();
        let deadline = Utc::now() + Duration::days(5);
        let custom_article = Article::builder("election-preview", "2026 Mayoral Race")
            .id(custom_id)
            .description("In-depth interviews with all candidates.")
            .stage(ArticleStage::Writing)
            .deadline(deadline)
            .color("#2ECC71")
            .build();

        assert_eq!(custom_article.id, custom_id);
        assert_eq!(custom_article.slug, "election-preview");
        assert_eq!(custom_article.headline, "2026 Mayoral Race");
        assert_eq!(
            custom_article.description.as_deref(),
            Some("In-depth interviews with all candidates.")
        );
        assert_eq!(custom_article.stage, ArticleStage::Writing);
        assert_eq!(custom_article.deadline, Some(deadline));
        assert_eq!(custom_article.color.as_deref(), Some("#2ECC71"));
    }

    #[test]
    fn test_article_overdue_evaluation() {
        let now = Utc::now();
        let past = now - Duration::hours(2);
        let future = now + Duration::hours(2);

        let mut article = Article::new("breaking-news", "Breaking News Report");
        assert!(!article.is_overdue(now));

        article = article.with_deadline(future);
        assert!(!article.is_overdue(now));

        article = article.with_deadline(past);
        assert!(article.is_overdue(now));

        // When published, overdue status is false even if deadline is in past
        article.set_stage(ArticleStage::Published);
        assert!(!article.is_overdue(now));
    }

    #[test]
    fn test_article_serde_roundtrip() {
        let article = Article::builder("test-story", "Test Story Headline")
            .description("Notes here")
            .stage(ArticleStage::Editing)
            .deadline(Utc::now() + Duration::days(1))
            .color("#3498DB")
            .build();

        let json = serde_json::to_string(&article).expect("article serialization failed");
        let decoded: Article = serde_json::from_str(&json).expect("article deserialization failed");
        assert_eq!(article.id, decoded.id);
        assert_eq!(article.slug, decoded.slug);
        assert_eq!(article.headline, decoded.headline);
        assert_eq!(article.stage, decoded.stage);
        assert_eq!(article.description, decoded.description);
        assert_eq!(article.color, decoded.color);
    }

    #[test]
    fn test_task_status_transitions_and_parsing() {
        let statuses = TaskStatus::all();
        assert_eq!(statuses.len(), 3);
        assert_eq!(
            statuses,
            &[
                TaskStatus::ToDo,
                TaskStatus::InProgress,
                TaskStatus::Complete
            ]
        );

        assert_eq!(TaskStatus::default(), TaskStatus::ToDo);
        assert_eq!(TaskStatus::ToDo.display_name(), "To-Do");
        assert_eq!(TaskStatus::InProgress.display_name(), "In Progress");
        assert_eq!(TaskStatus::Complete.display_name(), "Complete");

        assert_eq!(TaskStatus::ToDo.next_status(), Some(TaskStatus::InProgress));
        assert_eq!(
            TaskStatus::InProgress.next_status(),
            Some(TaskStatus::Complete)
        );
        assert_eq!(TaskStatus::Complete.next_status(), None);

        assert_eq!(TaskStatus::ToDo.prev_status(), None);
        assert_eq!(TaskStatus::InProgress.prev_status(), Some(TaskStatus::ToDo));
        assert_eq!(
            TaskStatus::Complete.prev_status(),
            Some(TaskStatus::InProgress)
        );

        assert_eq!("todo".parse::<TaskStatus>().unwrap(), TaskStatus::ToDo);
        assert_eq!("to_do".parse::<TaskStatus>().unwrap(), TaskStatus::ToDo);
        assert_eq!(
            "in_progress".parse::<TaskStatus>().unwrap(),
            TaskStatus::InProgress
        );
        assert_eq!(
            "complete".parse::<TaskStatus>().unwrap(),
            TaskStatus::Complete
        );
        assert_eq!("done".parse::<TaskStatus>().unwrap(), TaskStatus::Complete);

        assert!("invalid_task_status".parse::<TaskStatus>().is_err());
    }

    #[test]
    fn test_task_creation_and_overdue() {
        let article_id = Uuid::new_v4();
        let now = Utc::now();
        let past = now - Duration::hours(1);

        let mut task = Task::new(article_id, "Interview Mayor");
        assert_eq!(task.article_id, article_id);
        assert_eq!(task.title, "Interview Mayor");
        assert_eq!(task.status, TaskStatus::ToDo);
        assert!(!task.is_overdue(now));

        task = task.with_due_date(past);
        assert!(task.is_overdue(now));

        task.set_status(TaskStatus::Complete);
        assert!(!task.is_overdue(now));
    }

    #[test]
    fn test_task_builder_and_serde() {
        let article_id = Uuid::new_v4();
        let task = Task::builder(article_id, "Freedom of Information Act Request")
            .notes("Submit FOIA request to transit department")
            .due_date(Utc::now() + Duration::days(3))
            .status(TaskStatus::InProgress)
            .build();

        let json = serde_json::to_string(&task).expect("task serialization failed");
        let decoded: Task = serde_json::from_str(&json).expect("task deserialization failed");
        assert_eq!(task.id, decoded.id);
        assert_eq!(task.article_id, decoded.article_id);
        assert_eq!(task.title, decoded.title);
        assert_eq!(task.status, decoded.status);
    }

    #[test]
    fn test_contact_creation_and_builder() {
        let contact = Contact::new("Jane Doe");
        assert_eq!(contact.name, "Jane Doe");
        assert!(contact.organization.is_none());

        let full_contact = Contact::builder("John Smith")
            .organization("Metropolitan Police")
            .role("Public Information Officer")
            .phone("+1-555-0199")
            .email("john.smith@police.gov")
            .notes("Prefers signal messages for off-the-record background.")
            .build();

        assert_eq!(full_contact.name, "John Smith");
        assert_eq!(
            full_contact.organization.as_deref(),
            Some("Metropolitan Police")
        );
        assert_eq!(
            full_contact.role.as_deref(),
            Some("Public Information Officer")
        );
        assert_eq!(full_contact.phone.as_deref(), Some("+1-555-0199"));
        assert_eq!(full_contact.email.as_deref(), Some("john.smith@police.gov"));

        let json = serde_json::to_string(&full_contact).expect("contact serde failed");
        let decoded: Contact = serde_json::from_str(&json).expect("contact parse failed");
        assert_eq!(full_contact.id, decoded.id);
        assert_eq!(full_contact.name, decoded.name);
    }

    #[test]
    fn test_article_contact_join() {
        let article_id = Uuid::new_v4();
        let contact_id = Uuid::new_v4();
        let link = ArticleContact::new(article_id, contact_id);

        assert_eq!(link.article_id, article_id);
        assert_eq!(link.contact_id, contact_id);

        let json = serde_json::to_string(&link).expect("article_contact serde failed");
        let decoded: ArticleContact =
            serde_json::from_str(&json).expect("article_contact parse failed");
        assert_eq!(link, decoded);
    }

    #[test]
    fn test_theme_mode_and_settings() {
        assert_eq!(ThemeMode::all().len(), 3);
        assert_eq!(ThemeMode::default(), ThemeMode::System);
        assert_eq!(ThemeMode::System.display_name(), "System");
        assert_eq!(ThemeMode::Light.display_name(), "Light");
        assert_eq!(ThemeMode::Dark.display_name(), "Dark");

        assert_eq!("system".parse::<ThemeMode>().unwrap(), ThemeMode::System);
        assert_eq!("auto".parse::<ThemeMode>().unwrap(), ThemeMode::System);
        assert_eq!("os".parse::<ThemeMode>().unwrap(), ThemeMode::System);
        assert_eq!("light".parse::<ThemeMode>().unwrap(), ThemeMode::Light);
        assert_eq!("dark".parse::<ThemeMode>().unwrap(), ThemeMode::Dark);
        assert!("neon".parse::<ThemeMode>().is_err());

        let mut settings = Settings::default();
        assert_eq!(settings.theme_mode, ThemeMode::System);

        settings.set_theme_mode(ThemeMode::Dark);
        assert_eq!(settings.theme_mode, ThemeMode::Dark);

        let json = serde_json::to_string(&settings).expect("settings serde failed");
        let decoded: Settings = serde_json::from_str(&json).expect("settings parse failed");
        assert_eq!(settings, decoded);
    }

    #[test]
    fn test_entity_validation() {
        let valid_article = Article::builder("election-2026", "2026 Mayoral Race")
            .color("#3498DB")
            .build_validated();
        assert!(valid_article.is_ok());

        let invalid_slug_article = Article::builder("Election 2026", "Headline").build_validated();
        assert!(invalid_slug_article.is_err());

        let invalid_color_article = Article::builder("election-2026", "Headline")
            .color("invalid-color")
            .build_validated();
        assert!(invalid_color_article.is_err());

        let valid_task =
            Task::builder(Uuid::new_v4(), "Interview city treasurer").build_validated();
        assert!(valid_task.is_ok());

        let invalid_task = Task::builder(Uuid::new_v4(), "   ").build_validated();
        assert!(invalid_task.is_err());

        let valid_contact = Contact::builder("Jane Doe")
            .email("jane@news.example.org")
            .phone("+1 555-0199")
            .build_validated();
        assert!(valid_contact.is_ok());

        let invalid_contact_email = Contact::builder("Jane Doe")
            .email("not-an-email")
            .build_validated();
        assert!(invalid_contact_email.is_err());

        let invalid_contact_phone = Contact::builder("Jane Doe")
            .phone("abc-123")
            .build_validated();
        assert!(invalid_contact_phone.is_err());

        let invalid_contact_name = Contact::builder("").build_validated();
        assert!(invalid_contact_name.is_err());

        let settings = Settings::default();
        assert!(settings.validate().is_ok());
    }
}
