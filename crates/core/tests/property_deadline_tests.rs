//! Property-based test suite for `newsjournal-core` deadline and overdue engine.

use chrono::{DateTime, Duration, TimeZone, Utc};
use newsjournal_core::deadline::*;
use newsjournal_core::*;
use proptest::prelude::*;
use uuid::Uuid;

fn arbitrary_datetime() -> impl Strategy<Value = DateTime<Utc>> {
    // Generate timestamps between 2020-01-01 and 2030-01-01 (in seconds)
    (1577836800i64..=1893456000i64).prop_map(|secs| Utc.timestamp_opt(secs, 0).unwrap())
}

fn arbitrary_article_stage() -> impl Strategy<Value = ArticleStage> {
    prop::sample::select(vec![
        ArticleStage::Pitching,
        ArticleStage::Researching,
        ArticleStage::Writing,
        ArticleStage::Editing,
        ArticleStage::ReadyToPublish,
        ArticleStage::Published,
    ])
}

fn arbitrary_task_status() -> impl Strategy<Value = TaskStatus> {
    prop::sample::select(vec![
        TaskStatus::ToDo,
        TaskStatus::InProgress,
        TaskStatus::Complete,
    ])
}

proptest! {
    // -------------------------------------------------------------------------
    // 1. Published Article Invariant
    // -------------------------------------------------------------------------

    #[test]
    fn test_prop_published_article_is_never_overdue(
        now in arbitrary_datetime(),
        deadline in prop::option::of(arbitrary_datetime()),
        slug in "[a-z0-9-]{1,20}"
    ) {
        let mut article = Article::new(&slug, "Published Story");
        article.stage = ArticleStage::Published;
        article.deadline = deadline;

        // An article in Published stage is NEVER overdue regardless of deadline or evaluation time
        prop_assert!(!article.is_overdue(now));
        prop_assert!(!is_article_overdue(&article, now));
        prop_assert_eq!(article.deadline_status(now), DeadlineStatus::Completed);
        prop_assert_eq!(article.deadline_status(now).urgency_level(), UrgencyLevel::None);
        prop_assert_eq!(article.deadline_status(now).badge_color_hint(), "slate");
    }

    // -------------------------------------------------------------------------
    // 2. Complete Task Invariant
    // -------------------------------------------------------------------------

    #[test]
    fn test_prop_complete_task_is_never_overdue(
        now in arbitrary_datetime(),
        due_date in prop::option::of(arbitrary_datetime()),
        title in "[a-zA-Z0-9 ]{1,30}"
    ) {
        let mut task = Task::new(Uuid::new_v4(), &title);
        task.status = TaskStatus::Complete;
        task.due_date = due_date;

        prop_assert!(!task.is_overdue(now));
        prop_assert!(!is_task_overdue(&task, now));
        prop_assert_eq!(task.deadline_status(now), DeadlineStatus::Completed);
        prop_assert_eq!(task.deadline_status(now).urgency_level(), UrgencyLevel::None);
    }

    // -------------------------------------------------------------------------
    // 3. Active Stage Overdue Evaluation
    // -------------------------------------------------------------------------

    #[test]
    fn test_prop_active_article_overdue_evaluation(
        now in arbitrary_datetime(),
        offset_secs in -864000i64..=864000i64,
        stage in prop::sample::select(vec![
            ArticleStage::Pitching,
            ArticleStage::Researching,
            ArticleStage::Writing,
            ArticleStage::Editing,
            ArticleStage::ReadyToPublish,
        ])
    ) {
        let deadline = now + Duration::seconds(offset_secs);
        let mut article = Article::new("active-story", "Active Headline");
        article.stage = stage;
        article.deadline = Some(deadline);

        if now > deadline {
            // Must be overdue
            prop_assert!(article.is_overdue(now));
            prop_assert!(is_article_overdue(&article, now));

            let status = article.deadline_status(now);
            prop_assert!(status.is_overdue());
            prop_assert_eq!(status.urgency_level(), UrgencyLevel::Overdue);
            prop_assert_eq!(status.overdue_duration(), Some(now - deadline));
            prop_assert_eq!(status.badge_color_hint(), "red");
        } else {
            // Must NOT be overdue
            prop_assert!(!article.is_overdue(now));
            prop_assert!(!is_article_overdue(&article, now));

            let remaining = deadline - now;
            let status = article.deadline_status(now);
            prop_assert!(!status.is_overdue());

            if remaining <= Duration::hours(1) {
                prop_assert!(status.is_critical());
                prop_assert_eq!(status.urgency_level(), UrgencyLevel::High);
                prop_assert_eq!(status.badge_color_hint(), "orange");
            } else if remaining <= Duration::hours(24) {
                prop_assert!(status.is_due_soon());
                prop_assert_eq!(status.urgency_level(), UrgencyLevel::Medium);
                prop_assert_eq!(status.badge_color_hint(), "amber");
            } else {
                prop_assert!(status.is_on_track());
                prop_assert_eq!(status.urgency_level(), UrgencyLevel::Low);
                prop_assert_eq!(status.badge_color_hint(), "emerald");
            }
        }
    }

    // -------------------------------------------------------------------------
    // 4. Batch Summary Conservation Law
    // -------------------------------------------------------------------------

    #[test]
    fn test_prop_article_summary_conservation_laws(
        now in arbitrary_datetime(),
        articles_data in prop::collection::vec(
            (
                "[a-z0-9-]{1,15}",
                arbitrary_article_stage(),
                prop::option::of(arbitrary_datetime())
            ),
            0..=40
        )
    ) {
        let articles: Vec<Article> = articles_data
            .into_iter()
            .enumerate()
            .map(|(i, (slug_prefix, stage, deadline))| {
                let mut a = Article::new(format!("{slug_prefix}-{i}"), format!("Headline {i}"));
                a.stage = stage;
                a.deadline = deadline;
                a
            })
            .collect();

        let summary = summarize_articles(&articles, now);

        // Conservation of total items
        prop_assert_eq!(summary.total_items, articles.len());

        // Conservation of partition breakdown
        let breakdown_sum = summary.overdue_count
            + summary.critical_count
            + summary.due_soon_count
            + summary.on_track_count
            + summary.completed_count
            + summary.no_deadline_count;
        prop_assert_eq!(summary.total_items, breakdown_sum);

        // Derived summary calculations
        prop_assert_eq!(
            summary.action_required_count(),
            summary.overdue_count + summary.critical_count + summary.due_soon_count
        );
        prop_assert_eq!(
            summary.active_with_deadline_count(),
            summary.overdue_count + summary.critical_count + summary.due_soon_count + summary.on_track_count
        );

        prop_assert_eq!(summary.has_overdue(), summary.overdue_count > 0);
        prop_assert_eq!(summary.has_critical(), summary.critical_count > 0);
        prop_assert_eq!(summary.has_due_soon(), summary.due_soon_count > 0);
    }

    #[test]
    fn test_prop_task_summary_conservation_laws(
        now in arbitrary_datetime(),
        tasks_data in prop::collection::vec(
            (
                arbitrary_task_status(),
                prop::option::of(arbitrary_datetime())
            ),
            0..=40
        )
    ) {
        let parent_id = Uuid::new_v4();
        let tasks: Vec<Task> = tasks_data
            .into_iter()
            .enumerate()
            .map(|(i, (status, due_date))| {
                let mut t = Task::new(parent_id, format!("Task {i}"));
                t.status = status;
                t.due_date = due_date;
                t
            })
            .collect();

        let summary = summarize_tasks(&tasks, now);

        prop_assert_eq!(summary.total_items, tasks.len());
        let breakdown_sum = summary.overdue_count
            + summary.critical_count
            + summary.due_soon_count
            + summary.on_track_count
            + summary.completed_count
            + summary.no_deadline_count;
        prop_assert_eq!(summary.total_items, breakdown_sum);
    }

    // -------------------------------------------------------------------------
    // 5. Sorting by Urgency Invariants
    // -------------------------------------------------------------------------

    #[test]
    fn test_prop_sort_articles_by_urgency(
        now in arbitrary_datetime(),
        articles_data in prop::collection::vec(
            (
                "[a-z0-9-]{1,15}",
                arbitrary_article_stage(),
                prop::option::of(arbitrary_datetime())
            ),
            1..=30
        )
    ) {
        let mut articles: Vec<Article> = articles_data
            .into_iter()
            .enumerate()
            .map(|(i, (slug_prefix, stage, deadline))| {
                let mut a = Article::new(format!("{slug_prefix}-{i}"), format!("Headline {i}"));
                a.stage = stage;
                a.deadline = deadline;
                a
            })
            .collect();

        sort_articles_by_urgency(&mut articles, now);

        // Verify strictly non-increasing order of urgency priority
        for window in articles.windows(2) {
            let a = &window[0];
            let b = &window[1];

            let status_a = a.deadline_status(now);
            let status_b = b.deadline_status(now);

            let urg_a = status_a.urgency_level();
            let urg_b = status_b.urgency_level();

            // Primary order: Urgency level descending
            prop_assert!(urg_a >= urg_b);

            // Secondary order within same urgency tier
            if urg_a == urg_b {
                match (status_a, status_b) {
                    (DeadlineStatus::Overdue { duration: d_a }, DeadlineStatus::Overdue { duration: d_b }) => {
                        // Longer overdue first
                        prop_assert!(d_a >= d_b);
                    }
                    (
                        DeadlineStatus::Critical { remaining: r_a },
                        DeadlineStatus::Critical { remaining: r_b }
                    ) | (
                        DeadlineStatus::DueSoon { remaining: r_a },
                        DeadlineStatus::DueSoon { remaining: r_b }
                    ) | (
                        DeadlineStatus::OnTrack { remaining: r_a },
                        DeadlineStatus::OnTrack { remaining: r_b }
                    ) => {
                        // Sooner deadline first
                        prop_assert!(r_a <= r_b);
                    }
                    _ => {}
                }
            }
        }
    }
}
