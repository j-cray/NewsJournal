//! Real-time deadline evaluation, overdue tracking, and urgency engine.
//!
//! This module provides domain logic for computing deadline expiration,
//! categorizing urgency tiers (critical, due soon, on track, overdue), formatting
//! user-facing alert badges, and calculating batch summary statistics for articles and tasks.
//!
//! # Core Invariants
//! - Articles in [`ArticleStage::Published`](crate::models::ArticleStage::Published) are **never** overdue.
//! - Tasks with [`TaskStatus::Complete`](crate::models::TaskStatus::Complete) are **never** overdue.
//! - When `now > deadline`, status transitions to [`DeadlineStatus::Overdue`].
//! - Deterministic urgency ordering enables high-priority triage on Kanban decks and directories.
//!
//! # Examples
//!
//! ```
//! use chrono::{Duration, Utc};
//! use newsjournal_core::deadline::{
//!     evaluate_article_deadline, DeadlineConfig, DeadlineStatus, UrgencyLevel,
//! };
//! use newsjournal_core::{Article, ArticleStage};
//!
//! let now = Utc::now();
//! let mut article = Article::new("front-page-investigation", "Mayoral Race Polling");
//! article.deadline = Some(now - Duration::hours(2));
//!
//! // Active stage with past deadline -> Overdue
//! let status = evaluate_article_deadline(&article, now);
//! assert!(status.is_overdue());
//! assert_eq!(status.urgency_level(), UrgencyLevel::Overdue);
//! assert_eq!(status.badge_text(), "Overdue (2h)");
//!
//! // Transitioning to Published immediately clears overdue status
//! article.stage = ArticleStage::Published;
//! let status = evaluate_article_deadline(&article, now);
//! assert_eq!(status, DeadlineStatus::Completed);
//! assert!(!status.is_overdue());
//! ```

pub mod config;
pub mod evaluator;
pub mod status;
pub mod summary;

pub use config::{
    DeadlineConfig, DeadlineConfigBuilder, DEFAULT_CRITICAL_HOURS, DEFAULT_DUE_SOON_HOURS,
};
pub use evaluator::{
    evaluate_article_deadline, evaluate_article_deadline_with_config, evaluate_raw_deadline,
    evaluate_task_deadline, evaluate_task_deadline_with_config, is_article_due_soon,
    is_article_overdue, is_task_due_soon, is_task_overdue,
};
pub use status::{format_compact_duration, format_verbose_duration, DeadlineStatus, UrgencyLevel};
pub use summary::{
    filter_due_soon_articles, filter_due_soon_tasks, filter_overdue_articles, filter_overdue_tasks,
    sort_articles_by_urgency, sort_tasks_by_urgency, summarize_articles,
    summarize_articles_with_config, summarize_tasks, summarize_tasks_with_config, DeadlineSummary,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Article, ArticleStage};
    use chrono::{Duration, Utc};

    #[test]
    fn test_deadline_module_reexports_and_integration() {
        let now = Utc::now();
        let mut article = Article::new("reexport-test", "Testing Top-Level Reexports");
        article.deadline = Some(now + Duration::hours(2));
        article.stage = ArticleStage::Writing;

        let status = evaluate_article_deadline(&article, now);
        assert!(status.is_due_soon());
        assert_eq!(status.urgency_level(), UrgencyLevel::Medium);
    }
}
