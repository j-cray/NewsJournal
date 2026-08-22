//! `newsjournal-core`
//!
//! Pure Rust domain logic, data models, validation, and persistence for the NewsJournal application.

pub mod error;
pub mod models;

pub use error::ModelError;
pub use models::{
    Article, ArticleBuilder, ArticleContact, ArticleStage, Contact, ContactBuilder, Settings, Task,
    TaskBuilder, TaskStatus, ThemeMode,
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
