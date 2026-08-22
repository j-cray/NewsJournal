//! Deadline ticker and real-time urgency evaluation tests.

use chrono::{Duration, Utc};
use newsjournal_core::models::{Article, ArticleStage};
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::state::toast::ToastMessage;
use newsjournal_gui::{AppState, EventLoop, UrgencyFilter};

#[test]
fn test_deadline_ticker_and_urgency_transitions() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let now = Utc::now();

    // 1. Article with no deadline
    let a_nodeadline = Article::new("feature-story", "Feature Story on Arts");

    // 2. Article due in 48 hours (OnTrack)
    let mut a_future = Article::new("weekly-review", "Weekly Political Review");
    a_future.deadline = Some(now + Duration::hours(48));

    // 3. Article due in 6 hours (DueSoon)
    let mut a_due_soon = Article::new("breaking-alert", "Breaking Weather Alert");
    a_due_soon.deadline = Some(now + Duration::hours(6));

    // 4. Article due 2 hours ago (Overdue)
    let mut a_overdue = Article::new("late-investigation", "Late Investigation Story");
    a_overdue.deadline = Some(now - Duration::hours(2));

    // 5. Published article with passed deadline (Never Overdue)
    let mut a_published = Article::new("published-story", "Already Published Story");
    a_published.deadline = Some(now - Duration::hours(10));
    a_published.stage = ArticleStage::Published;

    event_loop
        .dispatch(AppMessage::CreateArticle(a_nodeadline))
        .unwrap();
    event_loop
        .dispatch(AppMessage::CreateArticle(a_future))
        .unwrap();
    event_loop
        .dispatch(AppMessage::CreateArticle(a_due_soon))
        .unwrap();
    event_loop
        .dispatch(AppMessage::CreateArticle(a_overdue))
        .unwrap();
    event_loop
        .dispatch(AppMessage::CreateArticle(a_published))
        .unwrap();

    // Initial tick
    event_loop.dispatch(AppMessage::Tick(now)).unwrap();

    assert_eq!(event_loop.state().deadline_summary.total_items, 5);
    assert_eq!(event_loop.state().deadline_summary.overdue_count, 1);
    assert_eq!(event_loop.state().deadline_summary.due_soon_count, 1);
    assert_eq!(event_loop.state().deadline_summary.on_track_count, 1);
    assert_eq!(event_loop.state().deadline_summary.no_deadline_count, 1);
    assert_eq!(event_loop.state().deadline_summary.completed_count, 1);

    // Test Urgency Filter for Overdue Only
    event_loop
        .dispatch(AppMessage::SetUrgencyFilter(UrgencyFilter::OverdueOnly))
        .unwrap();
    assert_eq!(event_loop.state().filtered_articles().len(), 1);
    assert_eq!(
        event_loop.state().filtered_articles()[0].slug,
        "late-investigation"
    );

    // Advance time by 10 hours: a_due_soon (which was due in 6h) should now be OVERDUE
    let later = now + Duration::hours(10);
    event_loop.dispatch(AppMessage::Tick(later)).unwrap();

    assert_eq!(event_loop.state().deadline_summary.overdue_count, 2);
    assert_eq!(event_loop.state().filtered_articles().len(), 2);
}

#[test]
fn test_toast_auto_purge_during_ticks() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let now = Utc::now();
    let toast = ToastMessage::info("Sync", "Syncing database").with_ttl(5);

    event_loop.dispatch(AppMessage::PushToast(toast)).unwrap();
    assert_eq!(event_loop.state().toasts.len(), 1);

    // Tick at 3 seconds: toast remains
    event_loop
        .dispatch(AppMessage::Tick(now + Duration::seconds(3)))
        .unwrap();
    assert_eq!(event_loop.state().toasts.len(), 1);

    // Tick at 6 seconds: toast expired and purged
    event_loop
        .dispatch(AppMessage::Tick(now + Duration::seconds(6)))
        .unwrap();
    assert_eq!(event_loop.state().toasts.len(), 0);
}
