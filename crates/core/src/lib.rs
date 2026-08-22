//! `newsjournal-core`
//!
//! Pure Rust domain logic, data models, validation, and persistence for the NewsJournal application.

pub mod color;
pub mod deadline;
pub mod error;
pub mod models;
pub mod validation;

pub use color::{
    assign_color, assign_color_for_slug, assign_color_for_uuid, fnv1a_hash, fnv1a_hash_str,
    fnv1a_hash_uuid, Color, ColorError, NamedColor, Palette, CURATED_PALETTE,
};
pub use deadline::{
    evaluate_article_deadline, evaluate_article_deadline_with_config, evaluate_raw_deadline,
    evaluate_task_deadline, evaluate_task_deadline_with_config, filter_due_soon_articles,
    filter_due_soon_tasks, filter_overdue_articles, filter_overdue_tasks, format_compact_duration,
    format_verbose_duration, is_article_due_soon, is_article_overdue, is_task_due_soon,
    is_task_overdue, sort_articles_by_urgency, sort_tasks_by_urgency, summarize_articles,
    summarize_articles_with_config, summarize_tasks, summarize_tasks_with_config, DeadlineConfig,
    DeadlineConfigBuilder, DeadlineStatus, DeadlineSummary, UrgencyLevel, DEFAULT_CRITICAL_HOURS,
    DEFAULT_DUE_SOON_HOURS,
};
pub use error::ModelError;
pub use models::{
    Article, ArticleBuilder, ArticleContact, ArticleStage, Contact, ContactBuilder, Settings, Task,
    TaskBuilder, TaskStatus, ThemeMode,
};
pub use validation::{
    format_phone_display, is_valid_email, is_valid_hex_color, is_valid_phone, is_valid_slug,
    normalize_email, normalize_hex_color, normalize_phone, slugify, validate_email,
    validate_headline, validate_hex_color, validate_max_length, validate_name, validate_non_empty,
    validate_phone, validate_slug, validate_task_title, ValidationError, MAX_EMAIL_LENGTH,
    MAX_HEADLINE_LENGTH, MAX_LOCAL_PART_LENGTH, MAX_NAME_LENGTH, MAX_PHONE_DIGITS,
    MAX_PHONE_RAW_LENGTH, MAX_SLUG_LENGTH, MAX_TASK_TITLE_LENGTH, MIN_PHONE_DIGITS,
};

/// NewsJournal core crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_version_present() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_top_level_reexports() {
        let article = Article::new("breaking-news", "Breaking News Headline");
        assert_eq!(article.stage, ArticleStage::Pitching);

        let task = Task::new(article.id, "Fact check sources");
        assert_eq!(task.status, TaskStatus::ToDo);

        let contact = Contact::new("Reporter Alice");
        assert_eq!(contact.name, "Reporter Alice");

        let link = ArticleContact::new(article.id, contact.id);
        assert_eq!(link.article_id, article.id);

        let settings = Settings::default();
        assert_eq!(settings.theme_mode, ThemeMode::System);

        let now = chrono::Utc::now();
        let status = evaluate_article_deadline(&article, now);
        assert_eq!(status, DeadlineStatus::NoDeadline);
        assert_eq!(status.urgency_level(), UrgencyLevel::None);

        let config = DeadlineConfig::default();
        assert_eq!(
            config.critical_threshold,
            chrono::Duration::hours(DEFAULT_CRITICAL_HOURS)
        );
        assert_eq!(
            config.due_soon_threshold,
            chrono::Duration::hours(DEFAULT_DUE_SOON_HOURS)
        );
    }
}
