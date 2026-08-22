//! Comprehensive integration tests for the deadline and overdue engine.

use chrono::{DateTime, Duration, TimeZone, Utc};
use newsjournal_core::deadline::*;
use newsjournal_core::*;
use uuid::Uuid;

fn fixed_now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 22, 12, 0, 0).unwrap()
}

#[test]
fn test_article_published_stage_invariants() {
    let now = fixed_now();

    // 1. Published article with past deadline
    let mut past_art = Article::new("historic-story", "Old Historic Investigation");
    past_art.deadline = Some(now - Duration::days(365));
    past_art.stage = ArticleStage::Published;

    assert!(!past_art.is_overdue(now));
    assert!(!is_article_overdue(&past_art, now));
    assert_eq!(past_art.deadline_status(now), DeadlineStatus::Completed);
    assert_eq!(
        past_art.deadline_status(now).urgency_level(),
        UrgencyLevel::None
    );
    assert_eq!(past_art.deadline_status(now).badge_color_hint(), "slate");

    // 2. Published article with future deadline
    let mut future_art = Article::new("embargoed-story", "Scheduled Feature");
    future_art.deadline = Some(now + Duration::days(7));
    future_art.stage = ArticleStage::Published;

    assert!(!future_art.is_overdue(now));
    assert_eq!(future_art.deadline_status(now), DeadlineStatus::Completed);

    // 3. Published article with no deadline
    let mut no_dl_art = Article::new("breaking-tweet", "Quick Update");
    no_dl_art.stage = ArticleStage::Published;
    assert_eq!(no_dl_art.deadline_status(now), DeadlineStatus::Completed);
}

#[test]
fn test_active_article_stages_overdue_behavior() {
    let now = fixed_now();
    let past_deadline = now - Duration::hours(4);

    let active_stages = [
        ArticleStage::Pitching,
        ArticleStage::Researching,
        ArticleStage::Writing,
        ArticleStage::Editing,
        ArticleStage::ReadyToPublish,
    ];

    for stage in active_stages {
        let mut article = Article::new(
            format!("story-in-{}", stage.as_str()),
            format!("Story in {}", stage.display_name()),
        );
        article.deadline = Some(past_deadline);
        article.stage = stage;

        assert!(
            article.is_overdue(now),
            "Article in stage {stage:?} should be overdue"
        );
        assert!(is_article_overdue(&article, now));

        let status = article.deadline_status(now);
        assert!(status.is_overdue());
        assert_eq!(status.urgency_level(), UrgencyLevel::Overdue);
        assert_eq!(status.overdue_duration(), Some(Duration::hours(4)));
        assert_eq!(status.badge_text(), "Overdue (4h)");
        assert_eq!(status.badge_color_hint(), "red");
    }
}

#[test]
fn test_task_complete_status_invariants() {
    let now = fixed_now();
    let article_id = Uuid::new_v4();

    // 1. Complete task with past due date
    let mut past_task = Task::new(article_id, "Archived Interview");
    past_task.due_date = Some(now - Duration::days(30));
    past_task.status = TaskStatus::Complete;

    assert!(!past_task.is_overdue(now));
    assert!(!is_task_overdue(&past_task, now));
    assert_eq!(past_task.deadline_status(now), DeadlineStatus::Completed);
    assert_eq!(
        past_task.deadline_status(now).urgency_level(),
        UrgencyLevel::None
    );

    // 2. Active tasks with past due date
    let mut todo_task = Task::new(article_id, "Pending FOIA Request");
    todo_task.due_date = Some(now - Duration::hours(12));
    todo_task.status = TaskStatus::ToDo;
    assert!(todo_task.is_overdue(now));
    assert_eq!(
        todo_task.deadline_status(now),
        DeadlineStatus::Overdue {
            duration: Duration::hours(12)
        }
    );

    let mut in_prog_task = Task::new(article_id, "Review Document Dump");
    in_prog_task.due_date = Some(now - Duration::minutes(45));
    in_prog_task.status = TaskStatus::InProgress;
    assert!(in_prog_task.is_overdue(now));
    assert_eq!(
        in_prog_task.deadline_status(now),
        DeadlineStatus::Overdue {
            duration: Duration::minutes(45)
        }
    );
}

#[test]
fn test_boundary_conditions_and_urgency_tiers() {
    let now = fixed_now();
    let config = DeadlineConfig::default();

    // Exactly at deadline (now == deadline) -> remaining is 0 -> Critical (<= 1h)
    let status_exact = evaluate_raw_deadline(Some(now), false, now, &config);
    assert!(status_exact.is_critical());
    assert_eq!(
        status_exact,
        DeadlineStatus::Critical {
            remaining: Duration::zero()
        }
    );

    // 1 second past deadline -> Overdue
    let status_over_1s =
        evaluate_raw_deadline(Some(now - Duration::seconds(1)), false, now, &config);
    assert!(status_over_1s.is_overdue());
    assert_eq!(
        status_over_1s,
        DeadlineStatus::Overdue {
            duration: Duration::seconds(1)
        }
    );

    // 59 minutes remaining -> Critical
    let status_59m = evaluate_raw_deadline(Some(now + Duration::minutes(59)), false, now, &config);
    assert!(status_59m.is_critical());
    assert!(status_59m.is_due_soon());
    assert_eq!(status_59m.urgency_level(), UrgencyLevel::High);

    // Exactly 1 hour remaining -> Critical (<= 1h)
    let status_1h = evaluate_raw_deadline(Some(now + Duration::hours(1)), false, now, &config);
    assert!(status_1h.is_critical());

    // 1 hour and 1 minute remaining -> DueSoon
    let status_61m = evaluate_raw_deadline(Some(now + Duration::minutes(61)), false, now, &config);
    assert!(!status_61m.is_critical());
    assert!(status_61m.is_due_soon());
    assert_eq!(status_61m.urgency_level(), UrgencyLevel::Medium);

    // 24 hours remaining -> DueSoon (<= 24h)
    let status_24h = evaluate_raw_deadline(Some(now + Duration::hours(24)), false, now, &config);
    assert!(status_24h.is_due_soon());
    assert_eq!(status_24h.urgency_level(), UrgencyLevel::Medium);

    // 24 hours and 1 minute remaining -> OnTrack
    let status_24h1m =
        evaluate_raw_deadline(Some(now + Duration::minutes(1441)), false, now, &config);
    assert!(status_24h1m.is_on_track());
    assert!(!status_24h1m.is_due_soon());
    assert_eq!(status_24h1m.urgency_level(), UrgencyLevel::Low);
}

#[test]
fn test_custom_threshold_configurations() {
    let now = fixed_now();
    let strict_config = DeadlineConfig::builder()
        .critical_threshold(Duration::minutes(15))
        .due_soon_threshold(Duration::hours(4))
        .build();

    let mut article = Article::new("budget-investigation", "Annual City Budget Analysis");
    article.deadline = Some(now + Duration::hours(6));

    // Under default config (24h), 6 hours is DueSoon
    let default_status = evaluate_article_deadline(&article, now);
    assert!(default_status.is_due_soon());

    // Under strict config (4h), 6 hours is OnTrack
    let custom_status = evaluate_article_deadline_with_config(&article, now, &strict_config);
    assert!(custom_status.is_on_track());
}

#[test]
fn test_batch_summaries_and_statistics() {
    let now = fixed_now();

    let mut a_overdue_1 = Article::new("a1", "Overdue 6h");
    a_overdue_1.deadline = Some(now - Duration::hours(6));

    let mut a_overdue_2 = Article::new("a2", "Overdue 2h");
    a_overdue_2.deadline = Some(now - Duration::hours(2));

    let mut a_crit = Article::new("a3", "Critical 30m");
    a_crit.deadline = Some(now + Duration::minutes(30));

    let mut a_soon = Article::new("a4", "Due Soon 10h");
    a_soon.deadline = Some(now + Duration::hours(10));

    let mut a_track = Article::new("a5", "On Track 3d");
    a_track.deadline = Some(now + Duration::days(3));

    let mut a_pub = Article::new("a6", "Published");
    a_pub.deadline = Some(now - Duration::days(5));
    a_pub.stage = ArticleStage::Published;

    let a_none = Article::new("a7", "No Deadline");

    let articles = vec![
        a_overdue_1,
        a_overdue_2,
        a_crit,
        a_soon,
        a_track,
        a_pub,
        a_none,
    ];

    let summary = summarize_articles(&articles, now);

    assert_eq!(summary.total_items, 7);
    assert_eq!(summary.overdue_count, 2);
    assert_eq!(summary.critical_count, 1);
    assert_eq!(summary.due_soon_count, 1);
    assert_eq!(summary.on_track_count, 1);
    assert_eq!(summary.completed_count, 1);
    assert_eq!(summary.no_deadline_count, 1);

    assert_eq!(summary.max_overdue_duration, Some(Duration::hours(6)));
    assert_eq!(summary.earliest_deadline, Some(now + Duration::minutes(30)));
    assert_eq!(summary.action_required_count(), 4); // 2 overdue + 1 crit + 1 soon
    assert_eq!(summary.active_with_deadline_count(), 5); // 2 + 1 + 1 + 1
    assert!(summary.has_overdue());
    assert!(summary.has_critical());
    assert!(summary.has_due_soon());
}

#[test]
fn test_batch_filtering_and_sorting() {
    let now = fixed_now();

    let mut a_overdue_long = Article::new("a-overdue-long", "Overdue Long");
    a_overdue_long.deadline = Some(now - Duration::days(3));

    let mut a_overdue_short = Article::new("a-overdue-short", "Overdue Short");
    a_overdue_short.deadline = Some(now - Duration::hours(1));

    let mut a_crit = Article::new("a-crit", "Critical");
    a_crit.deadline = Some(now + Duration::minutes(20));

    let mut a_soon = Article::new("a-soon", "Due Soon");
    a_soon.deadline = Some(now + Duration::hours(5));

    let mut a_far = Article::new("a-far", "Far Future");
    a_far.deadline = Some(now + Duration::days(10));

    let a_nodelay = Article::new("a-nodelay", "No Deadline");

    let mut a_done = Article::new("a-done", "Published Story");
    a_done.stage = ArticleStage::Published;

    let article_list = vec![
        a_done.clone(),
        a_soon.clone(),
        a_far.clone(),
        a_nodelay.clone(),
        a_overdue_short.clone(),
        a_crit.clone(),
        a_overdue_long.clone(),
    ];

    // Test filter overdue
    let overdue_filtered = filter_overdue_articles(&article_list, now);
    assert_eq!(overdue_filtered.len(), 2);
    let overdue_slugs: Vec<&str> = overdue_filtered.iter().map(|a| a.slug.as_str()).collect();
    assert!(overdue_slugs.contains(&"a-overdue-long"));
    assert!(overdue_slugs.contains(&"a-overdue-short"));

    // Test filter due soon
    let due_soon_filtered = filter_due_soon_articles(&article_list, now, None);
    assert_eq!(due_soon_filtered.len(), 2); // a_crit and a_soon
    let due_soon_slugs: Vec<&str> = due_soon_filtered.iter().map(|a| a.slug.as_str()).collect();
    assert!(due_soon_slugs.contains(&"a-crit"));
    assert!(due_soon_slugs.contains(&"a-soon"));

    // Test sort by urgency
    let mut sortable = article_list.clone();
    sort_articles_by_urgency(&mut sortable, now);

    let sorted_slugs: Vec<&str> = sortable.iter().map(|a| a.slug.as_str()).collect();
    assert_eq!(
        sorted_slugs,
        vec![
            "a-overdue-long",
            "a-overdue-short",
            "a-crit",
            "a-soon",
            "a-far",
            "a-nodelay",
            "a-done",
        ]
    );
}

#[test]
fn test_task_filtering_and_sorting() {
    let now = fixed_now();
    let art_id = Uuid::new_v4();

    let mut t_overdue = Task::new(art_id, "T Overdue");
    t_overdue.due_date = Some(now - Duration::hours(5));

    let mut t_soon = Task::new(art_id, "T Soon");
    t_soon.due_date = Some(now + Duration::hours(3));

    let mut t_complete = Task::new(art_id, "T Complete");
    t_complete.due_date = Some(now - Duration::days(2));
    t_complete.status = TaskStatus::Complete;

    let tasks = vec![t_complete.clone(), t_soon.clone(), t_overdue.clone()];

    let overdue_tasks = filter_overdue_tasks(&tasks, now);
    assert_eq!(overdue_tasks.len(), 1);
    assert_eq!(overdue_tasks[0].title, "T Overdue");

    let due_soon_tasks = filter_due_soon_tasks(&tasks, now, None);
    assert_eq!(due_soon_tasks.len(), 1);
    assert_eq!(due_soon_tasks[0].title, "T Soon");

    let mut sortable_tasks = tasks.clone();
    sort_tasks_by_urgency(&mut sortable_tasks, now);
    let sorted_titles: Vec<&str> = sortable_tasks.iter().map(|t| t.title.as_str()).collect();
    assert_eq!(sorted_titles, vec!["T Overdue", "T Soon", "T Complete"]);
}

#[test]
fn test_json_serde_for_all_deadline_structures() {
    let config = DeadlineConfig::from_hours(2, 48);
    let json_config = serde_json::to_string(&config).unwrap();
    let restored_config: DeadlineConfig = serde_json::from_str(&json_config).unwrap();
    assert_eq!(config, restored_config);

    let status = DeadlineStatus::Overdue {
        duration: Duration::hours(7),
    };
    let json_status = serde_json::to_string(&status).unwrap();
    let restored_status: DeadlineStatus = serde_json::from_str(&json_status).unwrap();
    assert_eq!(status, restored_status);

    let summary = DeadlineSummary {
        total_items: 10,
        overdue_count: 2,
        critical_count: 1,
        due_soon_count: 3,
        on_track_count: 2,
        completed_count: 1,
        no_deadline_count: 1,
        earliest_deadline: Some(fixed_now()),
        max_overdue_duration: Some(Duration::hours(12)),
    };
    let json_summary = serde_json::to_string(&summary).unwrap();
    let restored_summary: DeadlineSummary = serde_json::from_str(&json_summary).unwrap();
    assert_eq!(summary, restored_summary);
}
