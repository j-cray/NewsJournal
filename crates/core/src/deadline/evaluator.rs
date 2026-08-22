//! Core evaluation functions for article and task deadline tracking.

use chrono::{DateTime, Duration, Utc};

use crate::deadline::config::DeadlineConfig;
use crate::deadline::status::DeadlineStatus;
use crate::models::{Article, Task};

/// Evaluates raw deadline parameters against a reference timestamp.
///
/// If `is_completed` is true, returns [`DeadlineStatus::Completed`].
/// If `deadline` is `None`, returns [`DeadlineStatus::NoDeadline`].
/// If `now > deadline`, returns [`DeadlineStatus::Overdue`].
/// Otherwise, evaluates remaining duration against critical and due-soon thresholds.
///
/// # Examples
///
/// ```
/// use chrono::{Duration, Utc};
/// use newsjournal_core::deadline::{evaluate_raw_deadline, DeadlineConfig, DeadlineStatus};
///
/// let now = Utc::now();
/// let config = DeadlineConfig::default();
///
/// // Published/completed item is never overdue
/// let status = evaluate_raw_deadline(Some(now - Duration::hours(5)), true, now, &config);
/// assert_eq!(status, DeadlineStatus::Completed);
///
/// // Past deadline on active item is overdue
/// let status = evaluate_raw_deadline(Some(now - Duration::hours(5)), false, now, &config);
/// assert!(status.is_overdue());
/// ```
#[must_use]
pub fn evaluate_raw_deadline(
    deadline: Option<DateTime<Utc>>,
    is_completed: bool,
    now: DateTime<Utc>,
    config: &DeadlineConfig,
) -> DeadlineStatus {
    if is_completed {
        return DeadlineStatus::Completed;
    }

    let Some(deadline) = deadline else {
        return DeadlineStatus::NoDeadline;
    };

    if now > deadline {
        let duration = now.signed_duration_since(deadline);
        DeadlineStatus::Overdue { duration }
    } else {
        let remaining = deadline.signed_duration_since(now);
        if remaining <= config.critical_threshold {
            DeadlineStatus::Critical { remaining }
        } else if remaining <= config.due_soon_threshold {
            DeadlineStatus::DueSoon { remaining }
        } else {
            DeadlineStatus::OnTrack { remaining }
        }
    }
}

/// Evaluates an article's deadline against the reference timestamp using default thresholds.
///
/// Articles in the [`ArticleStage::Published`](crate::models::ArticleStage::Published)
/// stage are automatically excluded from overdue alerts and return [`DeadlineStatus::Completed`].
#[must_use]
pub fn evaluate_article_deadline(article: &Article, now: DateTime<Utc>) -> DeadlineStatus {
    evaluate_article_deadline_with_config(article, now, &DeadlineConfig::default())
}

/// Evaluates an article's deadline against the reference timestamp with custom configuration.
#[must_use]
pub fn evaluate_article_deadline_with_config(
    article: &Article,
    now: DateTime<Utc>,
    config: &DeadlineConfig,
) -> DeadlineStatus {
    evaluate_raw_deadline(article.deadline, article.stage.is_published(), now, config)
}

/// Evaluates a task's deadline against the reference timestamp using default thresholds.
///
/// Tasks in the [`TaskStatus::Complete`](crate::models::TaskStatus::Complete)
/// status are automatically excluded from overdue alerts and return [`DeadlineStatus::Completed`].
#[must_use]
pub fn evaluate_task_deadline(task: &Task, now: DateTime<Utc>) -> DeadlineStatus {
    evaluate_task_deadline_with_config(task, now, &DeadlineConfig::default())
}

/// Evaluates a task's deadline against the reference timestamp with custom configuration.
#[must_use]
pub fn evaluate_task_deadline_with_config(
    task: &Task,
    now: DateTime<Utc>,
    config: &DeadlineConfig,
) -> DeadlineStatus {
    evaluate_raw_deadline(task.due_date, task.status.is_complete(), now, config)
}

/// Returns `true` if an article is active and its deadline has passed.
/// Published articles are never considered overdue.
#[must_use]
pub fn is_article_overdue(article: &Article, now: DateTime<Utc>) -> bool {
    evaluate_article_deadline(article, now).is_overdue()
}

/// Returns `true` if a task is active and its due date has passed.
/// Completed tasks are never considered overdue.
#[must_use]
pub fn is_task_overdue(task: &Task, now: DateTime<Utc>) -> bool {
    evaluate_task_deadline(task, now).is_overdue()
}

/// Returns `true` if an article has an active deadline within the due-soon window (default 24h).
#[must_use]
pub fn is_article_due_soon(
    article: &Article,
    now: DateTime<Utc>,
    threshold: Option<Duration>,
) -> bool {
    let config = if let Some(thresh) = threshold {
        DeadlineConfig::default().with_due_soon_threshold(thresh)
    } else {
        DeadlineConfig::default()
    };
    evaluate_article_deadline_with_config(article, now, &config).is_due_soon()
}

/// Returns `true` if a task has an active due date within the due-soon window (default 24h).
#[must_use]
pub fn is_task_due_soon(task: &Task, now: DateTime<Utc>, threshold: Option<Duration>) -> bool {
    let config = if let Some(thresh) = threshold {
        DeadlineConfig::default().with_due_soon_threshold(thresh)
    } else {
        DeadlineConfig::default()
    };
    evaluate_task_deadline_with_config(task, now, &config).is_due_soon()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ArticleStage, TaskStatus};

    #[test]
    fn test_article_published_is_never_overdue() {
        let now = Utc::now();
        let past_deadline = now - Duration::days(5);

        let mut article = Article::new("breaking-investigation", "City Hall Audit");
        article.deadline = Some(past_deadline);
        article.stage = ArticleStage::Published;

        assert!(!is_article_overdue(&article, now));
        assert_eq!(
            evaluate_article_deadline(&article, now),
            DeadlineStatus::Completed
        );
    }

    #[test]
    fn test_article_active_overdue() {
        let now = Utc::now();
        let past_deadline = now - Duration::hours(3);

        for stage in [
            ArticleStage::Pitching,
            ArticleStage::Researching,
            ArticleStage::Writing,
            ArticleStage::Editing,
            ArticleStage::ReadyToPublish,
        ] {
            let mut article = Article::new("breaking-news", "Breaking Headline");
            article.deadline = Some(past_deadline);
            article.stage = stage;

            assert!(
                is_article_overdue(&article, now),
                "Stage {stage:?} should be overdue"
            );
            let status = evaluate_article_deadline(&article, now);
            assert!(status.is_overdue());
            assert_eq!(status.overdue_duration(), Some(Duration::hours(3)));
        }
    }

    #[test]
    fn test_article_critical_due_soon_on_track() {
        let now = Utc::now();

        let mut critical_art = Article::new("slug-1", "Headline 1");
        critical_art.deadline = Some(now + Duration::minutes(30));
        let status = evaluate_article_deadline(&critical_art, now);
        assert!(status.is_critical());
        assert!(status.is_due_soon());
        assert!(!status.is_overdue());

        let mut due_soon_art = Article::new("slug-2", "Headline 2");
        due_soon_art.deadline = Some(now + Duration::hours(12));
        let status = evaluate_article_deadline(&due_soon_art, now);
        assert!(status.is_due_soon());
        assert!(!status.is_critical());
        assert!(!status.is_overdue());

        let mut on_track_art = Article::new("slug-3", "Headline 3");
        on_track_art.deadline = Some(now + Duration::days(3));
        let status = evaluate_article_deadline(&on_track_art, now);
        assert!(status.is_on_track());
        assert!(!status.is_due_soon());

        let no_deadline_art = Article::new("slug-4", "Headline 4");
        let status = evaluate_article_deadline(&no_deadline_art, now);
        assert_eq!(status, DeadlineStatus::NoDeadline);
    }

    #[test]
    fn test_task_complete_is_never_overdue() {
        let now = Utc::now();
        let past_due = now - Duration::hours(10);
        let article_id = uuid::Uuid::new_v4();

        let mut task = Task::new(article_id, "Interview Source");
        task.due_date = Some(past_due);
        task.status = TaskStatus::Complete;

        assert!(!is_task_overdue(&task, now));
        assert_eq!(
            evaluate_task_deadline(&task, now),
            DeadlineStatus::Completed
        );
    }

    #[test]
    fn test_task_active_overdue_and_due_soon() {
        let now = Utc::now();
        let article_id = uuid::Uuid::new_v4();

        let mut task_todo = Task::new(article_id, "Interview Mayor");
        task_todo.due_date = Some(now - Duration::hours(1));
        task_todo.status = TaskStatus::ToDo;
        assert!(is_task_overdue(&task_todo, now));

        let mut task_prog = Task::new(article_id, "Analyze Spreadsheet");
        task_prog.due_date = Some(now + Duration::hours(5));
        task_prog.status = TaskStatus::InProgress;
        assert!(!is_task_overdue(&task_prog, now));
        assert!(is_task_due_soon(&task_prog, now, None));
    }

    #[test]
    fn test_custom_threshold_evaluation() {
        let now = Utc::now();
        let config = DeadlineConfig::builder()
            .critical_threshold(Duration::minutes(15))
            .due_soon_threshold(Duration::hours(6))
            .build();

        let mut article = Article::new("investigative-leak", "Whistleblower Documents");
        article.deadline = Some(now + Duration::hours(12));

        // Under default (24h), this is DueSoon
        assert!(evaluate_article_deadline(&article, now).is_due_soon());

        // Under custom (6h), 12h is OnTrack
        let status = evaluate_article_deadline_with_config(&article, now, &config);
        assert!(status.is_on_track());
    }
}
