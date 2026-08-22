//! `newsjournal-core`
//!
//! Pure Rust domain logic, data models, validation, and persistence for the NewsJournal application.

pub mod error;
pub mod models;
pub mod validation;

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
    }
}
