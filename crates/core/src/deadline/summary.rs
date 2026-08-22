//! Batch deadline summary metrics, filtering, and urgency sorting.

use std::cmp::Ordering;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::deadline::config::DeadlineConfig;
use crate::deadline::evaluator::{
    evaluate_article_deadline_with_config, evaluate_task_deadline_with_config,
};
use crate::deadline::status::DeadlineStatus;
use crate::models::{Article, Task};

/// Aggregated metrics across a collection of articles or tasks.
///
/// # Examples
///
/// ```
/// use chrono::{Duration, Utc};
/// use newsjournal_core::deadline::{summarize_articles, DeadlineSummary};
/// use newsjournal_core::{Article, ArticleStage};
///
/// let now = Utc::now();
/// let mut art1 = Article::new("breaking-1", "Headline 1");
/// art1.deadline = Some(now - Duration::hours(2));
///
/// let mut art2 = Article::new("breaking-2", "Headline 2");
/// art2.deadline = Some(now + Duration::hours(5));
///
/// let summary = summarize_articles(&[art1, art2], now);
/// assert_eq!(summary.total_items, 2);
/// assert_eq!(summary.overdue_count, 1);
/// assert_eq!(summary.due_soon_count, 1);
/// assert!(summary.has_overdue());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DeadlineSummary {
    /// Total items evaluated.
    pub total_items: usize,
    /// Number of active items with passed deadlines.
    pub overdue_count: usize,
    /// Number of active items in critical time window.
    pub critical_count: usize,
    /// Number of active items approaching due-soon threshold.
    pub due_soon_count: usize,
    /// Number of active items safely on track in the future.
    pub on_track_count: usize,
    /// Number of published/completed items.
    pub completed_count: usize,
    /// Number of active items without an assigned deadline.
    pub no_deadline_count: usize,
    /// Earliest upcoming active deadline (if any).
    pub earliest_deadline: Option<DateTime<Utc>>,
    /// Largest duration overdue among overdue items.
    pub max_overdue_duration: Option<Duration>,
}

impl DeadlineSummary {
    /// Returns `true` if there are any overdue items.
    #[must_use]
    pub const fn has_overdue(&self) -> bool {
        self.overdue_count > 0
    }

    /// Returns `true` if there are any critical deadline items.
    #[must_use]
    pub const fn has_critical(&self) -> bool {
        self.critical_count > 0
    }

    /// Returns `true` if there are any due-soon or critical deadline items.
    #[must_use]
    pub const fn has_due_soon(&self) -> bool {
        self.due_soon_count > 0 || self.critical_count > 0
    }

    /// Returns the count of items requiring active editorial attention (overdue + critical + due soon).
    #[must_use]
    pub const fn action_required_count(&self) -> usize {
        self.overdue_count + self.critical_count + self.due_soon_count
    }

    /// Returns the total number of active items tracked against a deadline.
    #[must_use]
    pub const fn active_with_deadline_count(&self) -> usize {
        self.overdue_count + self.critical_count + self.due_soon_count + self.on_track_count
    }
}

/// Summarizes deadline metrics for a collection of [`Article`] references using default config.
#[must_use]
pub fn summarize_articles<'a>(
    articles: impl IntoIterator<Item = &'a Article>,
    now: DateTime<Utc>,
) -> DeadlineSummary {
    summarize_articles_with_config(articles, now, &DeadlineConfig::default())
}

/// Summarizes deadline metrics for a collection of [`Article`] references with custom configuration.
#[must_use]
pub fn summarize_articles_with_config<'a>(
    articles: impl IntoIterator<Item = &'a Article>,
    now: DateTime<Utc>,
    config: &DeadlineConfig,
) -> DeadlineSummary {
    let mut summary = DeadlineSummary::default();

    for article in articles {
        summary.total_items += 1;
        let status = evaluate_article_deadline_with_config(article, now, config);

        match status {
            DeadlineStatus::Overdue { duration } => {
                summary.overdue_count += 1;
                match summary.max_overdue_duration {
                    Some(max_dur) if duration > max_dur => {
                        summary.max_overdue_duration = Some(duration);
                    }
                    None => summary.max_overdue_duration = Some(duration),
                    _ => {}
                }
            }
            DeadlineStatus::Critical { .. } => {
                summary.critical_count += 1;
                update_earliest_deadline(&mut summary.earliest_deadline, article.deadline);
            }
            DeadlineStatus::DueSoon { .. } => {
                summary.due_soon_count += 1;
                update_earliest_deadline(&mut summary.earliest_deadline, article.deadline);
            }
            DeadlineStatus::OnTrack { .. } => {
                summary.on_track_count += 1;
                update_earliest_deadline(&mut summary.earliest_deadline, article.deadline);
            }
            DeadlineStatus::Completed => {
                summary.completed_count += 1;
            }
            DeadlineStatus::NoDeadline => {
                summary.no_deadline_count += 1;
            }
        }
    }

    summary
}

/// Summarizes deadline metrics for a collection of [`Task`] references using default config.
#[must_use]
pub fn summarize_tasks<'a>(
    tasks: impl IntoIterator<Item = &'a Task>,
    now: DateTime<Utc>,
) -> DeadlineSummary {
    summarize_tasks_with_config(tasks, now, &DeadlineConfig::default())
}

/// Summarizes deadline metrics for a collection of [`Task`] references with custom configuration.
#[must_use]
pub fn summarize_tasks_with_config<'a>(
    tasks: impl IntoIterator<Item = &'a Task>,
    now: DateTime<Utc>,
    config: &DeadlineConfig,
) -> DeadlineSummary {
    let mut summary = DeadlineSummary::default();

    for task in tasks {
        summary.total_items += 1;
        let status = evaluate_task_deadline_with_config(task, now, config);

        match status {
            DeadlineStatus::Overdue { duration } => {
                summary.overdue_count += 1;
                match summary.max_overdue_duration {
                    Some(max_dur) if duration > max_dur => {
                        summary.max_overdue_duration = Some(duration);
                    }
                    None => summary.max_overdue_duration = Some(duration),
                    _ => {}
                }
            }
            DeadlineStatus::Critical { .. } => {
                summary.critical_count += 1;
                update_earliest_deadline(&mut summary.earliest_deadline, task.due_date);
            }
            DeadlineStatus::DueSoon { .. } => {
                summary.due_soon_count += 1;
                update_earliest_deadline(&mut summary.earliest_deadline, task.due_date);
            }
            DeadlineStatus::OnTrack { .. } => {
                summary.on_track_count += 1;
                update_earliest_deadline(&mut summary.earliest_deadline, task.due_date);
            }
            DeadlineStatus::Completed => {
                summary.completed_count += 1;
            }
            DeadlineStatus::NoDeadline => {
                summary.no_deadline_count += 1;
            }
        }
    }

    summary
}

#[inline]
fn update_earliest_deadline(
    earliest: &mut Option<DateTime<Utc>>,
    candidate: Option<DateTime<Utc>>,
) {
    if let Some(cand) = candidate {
        match earliest {
            Some(curr) if cand < *curr => *earliest = Some(cand),
            None => *earliest = Some(cand),
            _ => {}
        }
    }
}

/// Filters a collection of articles to only those that are currently overdue.
#[must_use]
pub fn filter_overdue_articles<'a>(
    articles: impl IntoIterator<Item = &'a Article>,
    now: DateTime<Utc>,
) -> Vec<&'a Article> {
    articles
        .into_iter()
        .filter(|art| art.is_overdue(now))
        .collect()
}

/// Filters a collection of tasks to only those that are currently overdue.
#[must_use]
pub fn filter_overdue_tasks<'a>(
    tasks: impl IntoIterator<Item = &'a Task>,
    now: DateTime<Utc>,
) -> Vec<&'a Task> {
    tasks
        .into_iter()
        .filter(|tsk| tsk.is_overdue(now))
        .collect()
}

/// Filters a collection of articles to those approaching their deadline within a threshold (default 24h).
#[must_use]
pub fn filter_due_soon_articles<'a>(
    articles: impl IntoIterator<Item = &'a Article>,
    now: DateTime<Utc>,
    threshold: Option<Duration>,
) -> Vec<&'a Article> {
    let config = if let Some(thresh) = threshold {
        DeadlineConfig::default().with_due_soon_threshold(thresh)
    } else {
        DeadlineConfig::default()
    };
    articles
        .into_iter()
        .filter(|art| evaluate_article_deadline_with_config(art, now, &config).is_due_soon())
        .collect()
}

/// Filters a collection of tasks to those approaching their due date within a threshold (default 24h).
#[must_use]
pub fn filter_due_soon_tasks<'a>(
    tasks: impl IntoIterator<Item = &'a Task>,
    now: DateTime<Utc>,
    threshold: Option<Duration>,
) -> Vec<&'a Task> {
    let config = if let Some(thresh) = threshold {
        DeadlineConfig::default().with_due_soon_threshold(thresh)
    } else {
        DeadlineConfig::default()
    };
    tasks
        .into_iter()
        .filter(|tsk| evaluate_task_deadline_with_config(tsk, now, &config).is_due_soon())
        .collect()
}

/// Sorts articles by urgency in place:
/// 1. Overdue articles (longest overdue first)
/// 2. Active articles with upcoming deadlines (soonest deadline first)
/// 3. Active articles without deadlines
/// 4. Published articles (most recently updated first)
pub fn sort_articles_by_urgency(articles: &mut [Article], now: DateTime<Utc>) {
    let config = DeadlineConfig::default();
    articles.sort_by(|a, b| compare_article_urgency(a, b, now, &config));
}

/// Sorts tasks by urgency in place:
/// 1. Overdue tasks (longest overdue first)
/// 2. Active tasks with upcoming due dates (soonest due date first)
/// 3. Active tasks without due dates
/// 4. Completed tasks
pub fn sort_tasks_by_urgency(tasks: &mut [Task], now: DateTime<Utc>) {
    let config = DeadlineConfig::default();
    tasks.sort_by(|a, b| compare_task_urgency(a, b, now, &config));
}

fn compare_article_urgency(
    a: &Article,
    b: &Article,
    now: DateTime<Utc>,
    config: &DeadlineConfig,
) -> Ordering {
    let status_a = evaluate_article_deadline_with_config(a, now, config);
    let status_b = evaluate_article_deadline_with_config(b, now, config);

    compare_deadline_statuses(
        status_a,
        a.deadline,
        a.updated_at,
        status_b,
        b.deadline,
        b.updated_at,
    )
}

fn compare_task_urgency(
    a: &Task,
    b: &Task,
    now: DateTime<Utc>,
    config: &DeadlineConfig,
) -> Ordering {
    let status_a = evaluate_task_deadline_with_config(a, now, config);
    let status_b = evaluate_task_deadline_with_config(b, now, config);

    compare_deadline_statuses(
        status_a,
        a.due_date,
        a.updated_at,
        status_b,
        b.due_date,
        b.updated_at,
    )
}

fn compare_deadline_statuses(
    status_a: DeadlineStatus,
    deadline_a: Option<DateTime<Utc>>,
    updated_a: DateTime<Utc>,
    status_b: DeadlineStatus,
    deadline_b: Option<DateTime<Utc>>,
    updated_b: DateTime<Utc>,
) -> Ordering {
    match (status_a, status_b) {
        // Both overdue: most overdue first (earliest deadline first)
        (DeadlineStatus::Overdue { .. }, DeadlineStatus::Overdue { .. }) => {
            deadline_a.cmp(&deadline_b)
        }
        (DeadlineStatus::Overdue { .. }, _) => Ordering::Less,
        (_, DeadlineStatus::Overdue { .. }) => Ordering::Greater,

        // Both upcoming active deadlines: soonest first
        (
            DeadlineStatus::Critical { .. }
            | DeadlineStatus::DueSoon { .. }
            | DeadlineStatus::OnTrack { .. },
            DeadlineStatus::Critical { .. }
            | DeadlineStatus::DueSoon { .. }
            | DeadlineStatus::OnTrack { .. },
        ) => deadline_a.cmp(&deadline_b),

        (
            DeadlineStatus::Critical { .. }
            | DeadlineStatus::DueSoon { .. }
            | DeadlineStatus::OnTrack { .. },
            _,
        ) => Ordering::Less,
        (
            _,
            DeadlineStatus::Critical { .. }
            | DeadlineStatus::DueSoon { .. }
            | DeadlineStatus::OnTrack { .. },
        ) => Ordering::Greater,

        // Both no deadline: most recently updated first
        (DeadlineStatus::NoDeadline, DeadlineStatus::NoDeadline) => updated_b.cmp(&updated_a),
        (DeadlineStatus::NoDeadline, DeadlineStatus::Completed) => Ordering::Less,
        (DeadlineStatus::Completed, DeadlineStatus::NoDeadline) => Ordering::Greater,

        // Both completed: most recently updated first
        (DeadlineStatus::Completed, DeadlineStatus::Completed) => updated_b.cmp(&updated_a),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ArticleStage, TaskStatus};
    use uuid::Uuid;

    #[test]
    fn test_summarize_articles_mixed_deck() {
        let now = Utc::now();

        let mut a1 = Article::new("art-1", "Overdue Article");
        a1.deadline = Some(now - Duration::hours(5));
        a1.stage = ArticleStage::Writing;

        let mut a2 = Article::new("art-2", "Critical Article");
        a2.deadline = Some(now + Duration::minutes(30));
        a2.stage = ArticleStage::Editing;

        let mut a3 = Article::new("art-3", "Due Soon Article");
        a3.deadline = Some(now + Duration::hours(10));
        a3.stage = ArticleStage::Researching;

        let mut a4 = Article::new("art-4", "On Track Article");
        a4.deadline = Some(now + Duration::days(4));
        a4.stage = ArticleStage::Pitching;

        let mut a5 = Article::new("art-5", "Published Article");
        a5.deadline = Some(now - Duration::days(1));
        a5.stage = ArticleStage::Published;

        let a6 = Article::new("art-6", "No Deadline Article");

        let articles = vec![a1, a2, a3, a4, a5, a6];
        let summary = summarize_articles(&articles, now);

        assert_eq!(summary.total_items, 6);
        assert_eq!(summary.overdue_count, 1);
        assert_eq!(summary.critical_count, 1);
        assert_eq!(summary.due_soon_count, 1);
        assert_eq!(summary.on_track_count, 1);
        assert_eq!(summary.completed_count, 1);
        assert_eq!(summary.no_deadline_count, 1);
        assert_eq!(summary.max_overdue_duration, Some(Duration::hours(5)));
        assert_eq!(summary.action_required_count(), 3);
        assert_eq!(summary.active_with_deadline_count(), 4);
        assert!(summary.has_overdue());
        assert!(summary.has_critical());
        assert!(summary.has_due_soon());
    }

    #[test]
    fn test_summarize_tasks() {
        let now = Utc::now();
        let art_id = Uuid::new_v4();

        let mut t1 = Task::new(art_id, "Overdue Task");
        t1.due_date = Some(now - Duration::hours(2));

        let mut t2 = Task::new(art_id, "Completed Task");
        t2.due_date = Some(now - Duration::hours(5));
        t2.status = TaskStatus::Complete;

        let tasks = vec![t1, t2];
        let summary = summarize_tasks(&tasks, now);

        assert_eq!(summary.total_items, 2);
        assert_eq!(summary.overdue_count, 1);
        assert_eq!(summary.completed_count, 1);
    }

    #[test]
    fn test_filter_articles_and_tasks() {
        let now = Utc::now();
        let art_id = Uuid::new_v4();

        let mut a1 = Article::new("a1", "A1");
        a1.deadline = Some(now - Duration::hours(1));

        let mut a2 = Article::new("a2", "A2");
        a2.deadline = Some(now + Duration::hours(5));

        let articles = vec![a1, a2];
        let overdue = filter_overdue_articles(&articles, now);
        assert_eq!(overdue.len(), 1);
        assert_eq!(overdue[0].slug, "a1");

        let due_soon = filter_due_soon_articles(&articles, now, None);
        assert_eq!(due_soon.len(), 1);
        assert_eq!(due_soon[0].slug, "a2");

        let mut t1 = Task::new(art_id, "T1");
        t1.due_date = Some(now - Duration::minutes(30));

        let mut t2 = Task::new(art_id, "T2");
        t2.due_date = Some(now + Duration::hours(2));

        let tasks = vec![t1, t2];
        let overdue_t = filter_overdue_tasks(&tasks, now);
        assert_eq!(overdue_t.len(), 1);
        assert_eq!(overdue_t[0].title, "T1");
    }

    #[test]
    fn test_sort_articles_by_urgency() {
        let now = Utc::now();

        let mut a_overdue_old = Article::new("overdue-old", "Old Overdue");
        a_overdue_old.deadline = Some(now - Duration::hours(10));

        let mut a_overdue_recent = Article::new("overdue-recent", "Recent Overdue");
        a_overdue_recent.deadline = Some(now - Duration::hours(1));

        let mut a_soon = Article::new("due-soon", "Due Soon");
        a_soon.deadline = Some(now + Duration::hours(2));

        let mut a_far = Article::new("due-far", "Due Far");
        a_far.deadline = Some(now + Duration::days(5));

        let a_no_dl = Article::new("no-dl", "No Deadline");

        let mut a_pub = Article::new("published", "Published");
        a_pub.stage = ArticleStage::Published;
        a_pub.deadline = Some(now - Duration::days(3));

        let mut articles = vec![
            a_pub.clone(),
            a_far.clone(),
            a_no_dl.clone(),
            a_overdue_recent.clone(),
            a_soon.clone(),
            a_overdue_old.clone(),
        ];

        sort_articles_by_urgency(&mut articles, now);

        let slugs: Vec<&str> = articles.iter().map(|a| a.slug.as_str()).collect();
        assert_eq!(
            slugs,
            vec![
                "overdue-old",
                "overdue-recent",
                "due-soon",
                "due-far",
                "no-dl",
                "published"
            ]
        );
    }
}
