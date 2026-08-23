//! Comprehensive integration and verification tests for the Task Card Component (Task 7.2).

use chrono::{Duration, Utc};
use newsjournal_core::deadline::UrgencyLevel;
use newsjournal_core::models::{Article, Task, TaskStatus};
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::navigation::NavTab;
use newsjournal_gui::platform::cosmic::app::CosmicApp;
use newsjournal_gui::platform::macos::app::MacosApp;
use newsjournal_gui::state::AppState;
use newsjournal_gui::views::{
    format_task_due_date_badge, format_task_notes_snippet, format_task_title_snippet,
    TaskAccentStripViewModel, TaskArticleSlugBadgeViewModel, TaskCardOverdueStyleViewModel,
    TaskCardViewModel, TaskCheckboxViewModel, TaskDueDateBadgeViewModel, TaskNotesPreviewViewModel,
    DEFAULT_TASK_ACCENT_STRIP_WIDTH, DEFAULT_TASK_CARD_BORDER_WIDTH,
    DEFAULT_TASK_CARD_CORNER_RADIUS, DUE_SOON_TASK_CARD_BORDER_WIDTH, MAX_TASK_NOTES_SNIPPET_LEN,
    MAX_TASK_TITLE_SNIPPET_LEN, OVERDUE_TASK_CARD_BORDER_WIDTH, TASK_COMPLETE_BG_TINT_HEX,
    TASK_COMPLETE_GREEN_HEX, TASK_DUE_SOON_AMBER_HEX, TASK_DUE_SOON_BG_TINT_HEX,
    TASK_OVERDUE_BG_TINT_HEX, TASK_OVERDUE_RED_HEX,
};
use uuid::Uuid;

#[test]
fn test_task_accent_strip_properties() {
    let blue_strip = TaskAccentStripViewModel::new("#3B82F6", true);
    assert_eq!(blue_strip.color_hex, "#3B82F6");
    assert_eq!(blue_strip.width_px, DEFAULT_TASK_ACCENT_STRIP_WIDTH);
    assert_eq!(blue_strip.contrast_fg_hex, "#FFFFFF");
    assert!(blue_strip.translucent_tint_hex.starts_with("#3B82F6"));
    assert!(blue_strip.is_assigned);

    let yellow_strip = TaskAccentStripViewModel::new("#FDE047", false);
    assert_eq!(yellow_strip.color_hex, "#FDE047");
    assert_eq!(yellow_strip.contrast_fg_hex, "#0F172A");
    assert!(!yellow_strip.is_assigned);
}

#[test]
fn test_task_slug_badge_formatting_and_action() {
    let article_id = Uuid::new_v4();

    // Standard short slug
    let standard = TaskArticleSlugBadgeViewModel::new(article_id, "city-hall", "#10B981", true);
    assert_eq!(standard.article_id, article_id);
    assert_eq!(standard.raw_slug, "city-hall");
    assert_eq!(standard.display_text, "#city-hall");
    assert_eq!(standard.compact_text, "#city-hall");
    assert_eq!(standard.color_hex, "#10B981");
    assert!(standard.is_assigned);
    assert_eq!(
        standard.click_filter_action,
        Some(AppMessage::SetArticleFilter(Some(article_id)))
    );

    // Long slug truncation
    let long_slug = TaskArticleSlugBadgeViewModel::new(
        article_id,
        "investigative-state-budget-scandal-2026",
        "#EF4444",
        true,
    );
    assert_eq!(
        long_slug.raw_slug,
        "investigative-state-budget-scandal-2026"
    );
    assert_eq!(
        long_slug.display_text,
        "#investigative-state-budget-scandal-2026"
    );
    assert_eq!(long_slug.compact_text, "#investigative-st…");

    // Unassigned task badge
    let unassigned =
        TaskArticleSlugBadgeViewModel::new(Uuid::nil(), "unassigned", "#64748B", false);
    assert_eq!(unassigned.display_text, "unassigned");
    assert!(!unassigned.is_assigned);
    assert_eq!(unassigned.click_filter_action, None);
}

#[test]
fn test_task_title_snippet_formatting_and_truncation() {
    let short_title = "Interview the mayor";
    assert_eq!(
        format_task_title_snippet(short_title, MAX_TASK_TITLE_SNIPPET_LEN),
        "Interview the mayor"
    );

    let multiline_title = "Verify city council vote records\n\tfrom yesterday's hearing";
    assert_eq!(
        format_task_title_snippet(multiline_title, MAX_TASK_TITLE_SNIPPET_LEN),
        "Verify city council vote records  from yesterday's hearing"
    );

    let very_long_title = "A".repeat(120);
    let snippet = format_task_title_snippet(&very_long_title, MAX_TASK_TITLE_SNIPPET_LEN);
    assert!(snippet.ends_with('…'));
    assert!(snippet.chars().count() <= MAX_TASK_TITLE_SNIPPET_LEN);
}

#[test]
fn test_task_notes_preview_extraction_and_truncation() {
    // 1. Empty or None notes
    let empty_notes = TaskNotesPreviewViewModel::new(None);
    assert!(!empty_notes.has_notes);
    assert_eq!(empty_notes.line_count, 0);
    assert_eq!(empty_notes.snippet, "");
    assert!(!empty_notes.is_truncated);

    let whitespace_notes = TaskNotesPreviewViewModel::new(Some("   \n\t  "));
    assert!(!whitespace_notes.has_notes);
    assert_eq!(whitespace_notes.snippet, "");

    // 2. Single-line notes
    let single_line = TaskNotesPreviewViewModel::new(Some("Reach out to spokesperson at 2 PM."));
    assert!(single_line.has_notes);
    assert_eq!(single_line.line_count, 1);
    assert_eq!(single_line.snippet, "Reach out to spokesperson at 2 PM.");
    assert!(!single_line.is_truncated);

    // 3. Multi-line notes joined with bullets
    let multiline = TaskNotesPreviewViewModel::new(Some(
        "1. Request FOIA documents\n2. Call police department\n3. Check court docket",
    ));
    assert!(multiline.has_notes);
    assert_eq!(multiline.line_count, 3);
    assert_eq!(
        multiline.snippet,
        "1. Request FOIA documents • 2. Call police department • 3. Check court docket"
    );
    assert!(!multiline.is_truncated);

    // 4. Long notes truncated cleanly
    let long_text = "Detailed background context: ".to_string() + &"important fact ".repeat(15);
    let truncated_notes = TaskNotesPreviewViewModel::new(Some(&long_text));
    assert!(truncated_notes.has_notes);
    assert!(truncated_notes.is_truncated);
    assert!(truncated_notes.snippet.ends_with('…'));
    assert!(truncated_notes.snippet.chars().count() <= MAX_TASK_NOTES_SNIPPET_LEN);

    // Direct helper check
    let (direct_snippet, is_trunc, lines) =
        format_task_notes_snippet("Line 1\nLine 2", MAX_TASK_NOTES_SNIPPET_LEN);
    assert_eq!(direct_snippet, "Line 1 • Line 2");
    assert!(!is_trunc);
    assert_eq!(lines, 2);
}

#[test]
fn test_task_checkbox_states_and_actions() {
    let task_id = Uuid::new_v4();

    // To-Do
    let todo_cb = TaskCheckboxViewModel::new(task_id, TaskStatus::ToDo);
    assert!(!todo_cb.is_checked);
    assert_eq!(todo_cb.status, TaskStatus::ToDo);
    assert_eq!(todo_cb.icon_name, "circle");
    assert_eq!(todo_cb.sf_symbol, "circle");
    assert_eq!(todo_cb.toggle_action, AppMessage::ToggleTaskStatus(task_id));

    // In Progress
    let prog_cb = TaskCheckboxViewModel::new(task_id, TaskStatus::InProgress);
    assert!(!prog_cb.is_checked);
    assert_eq!(prog_cb.status, TaskStatus::InProgress);
    assert_eq!(prog_cb.icon_name, "clock");
    assert_eq!(prog_cb.sf_symbol, "hourglass");
    assert_eq!(prog_cb.color_hex, "#3B82F6");

    // Complete
    let comp_cb = TaskCheckboxViewModel::new(task_id, TaskStatus::Complete);
    assert!(comp_cb.is_checked);
    assert_eq!(comp_cb.status, TaskStatus::Complete);
    assert_eq!(comp_cb.icon_name, "check-circle");
    assert_eq!(comp_cb.sf_symbol, "checkmark.circle.fill");
    assert_eq!(comp_cb.color_hex, TASK_COMPLETE_GREEN_HEX);
    assert_eq!(comp_cb.tooltip, "Mark task as To-Do");
}

#[test]
fn test_task_due_date_badge_formatting() {
    let now = Utc::now();

    // 1. None due date -> returns None
    assert_eq!(
        format_task_due_date_badge(None, TaskStatus::ToDo, now, false),
        None
    );

    // 2. Overdue task
    let past_deadline = now - Duration::hours(3);
    let overdue_badge: TaskDueDateBadgeViewModel =
        format_task_due_date_badge(Some(past_deadline), TaskStatus::ToDo, now, false)
            .expect("overdue badge");
    assert!(overdue_badge.is_overdue);
    assert!(!overdue_badge.is_due_soon);
    assert!(!overdue_badge.is_completed);
    assert_eq!(overdue_badge.badge_label, "OVERDUE");
    assert_eq!(overdue_badge.badge_fg_hex, TASK_OVERDUE_RED_HEX);
    assert_eq!(overdue_badge.urgency, UrgencyLevel::Overdue);
    assert_eq!(overdue_badge.icon_name, "alert-triangle");

    // 3. Due soon task (e.g. in 4 hours)
    let soon_deadline = now + Duration::hours(4);
    let due_soon_badge =
        format_task_due_date_badge(Some(soon_deadline), TaskStatus::InProgress, now, false)
            .expect("due soon badge");
    assert!(!due_soon_badge.is_overdue);
    assert!(due_soon_badge.is_due_soon);
    assert_eq!(due_soon_badge.badge_label, "DUE SOON");
    assert_eq!(due_soon_badge.badge_fg_hex, TASK_DUE_SOON_AMBER_HEX);
    assert_eq!(due_soon_badge.urgency, UrgencyLevel::High);

    // 4. Future normal task (e.g. in 5 days)
    let future_deadline = now + Duration::days(5);
    let future_badge =
        format_task_due_date_badge(Some(future_deadline), TaskStatus::ToDo, now, false)
            .expect("future badge");
    assert!(!future_badge.is_overdue);
    assert!(!future_badge.is_due_soon);
    assert_eq!(future_badge.urgency, UrgencyLevel::Low);

    // 5. Completed task with past deadline -> overdue suppressed
    let completed_badge =
        format_task_due_date_badge(Some(past_deadline), TaskStatus::Complete, now, false)
            .expect("completed badge");
    assert!(!completed_badge.is_overdue);
    assert!(!completed_badge.is_due_soon);
    assert!(completed_badge.is_completed);
    assert_eq!(completed_badge.badge_label, "Completed");
    assert_eq!(completed_badge.badge_fg_hex, TASK_COMPLETE_GREEN_HEX);
    assert_eq!(completed_badge.urgency, UrgencyLevel::None);
}

#[test]
fn test_task_card_overdue_and_urgent_styling() {
    let parent_color = "#3B82F6";

    // 1. Normal active task
    let normal_style =
        TaskCardOverdueStyleViewModel::compute(false, false, TaskStatus::ToDo, parent_color, false);
    assert!(!normal_style.is_overdue);
    assert!(!normal_style.is_due_soon);
    assert!(!normal_style.is_completed);
    assert_eq!(normal_style.border_width_px, DEFAULT_TASK_CARD_BORDER_WIDTH);
    assert_eq!(normal_style.border_color_hex, "#E2E8F0");
    assert!(!normal_style.accent_glow);

    // 2. Overdue task
    let overdue_style =
        TaskCardOverdueStyleViewModel::compute(true, false, TaskStatus::ToDo, parent_color, false);
    assert!(overdue_style.is_overdue);
    assert_eq!(
        overdue_style.border_width_px,
        OVERDUE_TASK_CARD_BORDER_WIDTH
    );
    assert_eq!(overdue_style.border_color_hex, TASK_OVERDUE_RED_HEX);
    assert_eq!(overdue_style.badge_text, Some("⚠️ Overdue".to_string()));
    assert!(overdue_style.accent_glow);

    // 3. Due soon task
    let due_soon_style = TaskCardOverdueStyleViewModel::compute(
        false,
        true,
        TaskStatus::InProgress,
        parent_color,
        false,
    );
    assert!(due_soon_style.is_due_soon);
    assert_eq!(
        due_soon_style.border_width_px,
        DUE_SOON_TASK_CARD_BORDER_WIDTH
    );
    assert_eq!(due_soon_style.border_color_hex, TASK_DUE_SOON_AMBER_HEX);
    assert_eq!(due_soon_style.badge_text, Some("🕒 Due Soon".to_string()));
    assert!(!due_soon_style.accent_glow);

    // 4. Completed task with overdue flag -> suppressed
    let complete_style = TaskCardOverdueStyleViewModel::compute(
        true,
        false,
        TaskStatus::Complete,
        parent_color,
        false,
    );
    assert!(!complete_style.is_overdue);
    assert!(complete_style.is_completed);
    assert_eq!(
        complete_style.border_width_px,
        DEFAULT_TASK_CARD_BORDER_WIDTH
    );
    assert_eq!(complete_style.badge_text, None);

    // 5. Dark mode tint tests
    let dark_overdue =
        TaskCardOverdueStyleViewModel::compute(true, false, TaskStatus::ToDo, parent_color, true);
    assert_eq!(
        dark_overdue.background_tint_hex,
        Some(TASK_OVERDUE_BG_TINT_HEX.to_string())
    );

    let dark_due_soon = TaskCardOverdueStyleViewModel::compute(
        false,
        true,
        TaskStatus::InProgress,
        parent_color,
        true,
    );
    assert_eq!(
        dark_due_soon.background_tint_hex,
        Some(TASK_DUE_SOON_BG_TINT_HEX.to_string())
    );

    let dark_completed_badge = format_task_due_date_badge(
        Some(Utc::now() - Duration::hours(1)),
        TaskStatus::Complete,
        Utc::now(),
        true,
    )
    .expect("dark completed badge");
    assert_eq!(dark_completed_badge.badge_bg_hex, TASK_COMPLETE_BG_TINT_HEX);

    assert_eq!(DEFAULT_TASK_CARD_CORNER_RADIUS, 8.0);
}

#[test]
fn test_task_card_view_model_build_full_lifecycle() {
    let mut state = AppState::in_memory().expect("in-memory state");

    // Create a parent article
    let article = Article::new(
        "subway-expansion",
        "City Approves $2.5B Subway Expansion Project",
    )
    .with_color("#8B5CF6");
    let a_id = article.id;
    state.storage.create_article(article).expect("inserted");

    // Create tasks for this article
    let task_normal = Task::new(a_id, "Interview transit director")
        .with_notes("Prepare questions regarding timeline and budget allocation")
        .with_due_date(Utc::now() + Duration::days(2))
        .with_status(TaskStatus::InProgress);
    let t_id = task_normal.id;
    state
        .storage
        .create_task(task_normal.clone())
        .expect("task inserted");
    state.load_all().expect("state loaded");

    let card = TaskCardViewModel::build(&task_normal, &state, false);

    assert_eq!(card.id, t_id);
    assert_eq!(card.article_id, a_id);
    assert_eq!(card.parent_article_slug, "subway-expansion");
    assert_eq!(
        card.parent_article_headline,
        Some("City Approves $2.5B Subway Expansion Project".to_string())
    );
    assert_eq!(card.parent_article_color, "#8B5CF6");
    assert_eq!(card.title, "Interview transit director");
    assert_eq!(card.title_snippet, "Interview transit director");
    assert!(card.notes_preview.has_notes);
    assert_eq!(card.accent_strip.color_hex, "#8B5CF6");
    assert_eq!(card.slug_badge.display_text, "#subway-expansion");
    assert_eq!(
        card.slug_badge.click_filter_action,
        Some(AppMessage::SetArticleFilter(Some(a_id)))
    );
    assert_eq!(card.status, TaskStatus::InProgress);
    assert!(!card.checkbox.is_checked);
    assert_eq!(card.click_action, AppMessage::OpenEditTaskModal(t_id));
    assert_eq!(card.toggle_action, AppMessage::ToggleTaskStatus(t_id));
    assert!(!card.is_overdue);
    assert!(!card.is_dragging);
}

#[test]
fn test_platform_view_tree_task_cards_integration() {
    let mut state = AppState::in_memory().expect("in-memory state");
    state.active_tab = NavTab::TasksKanban;

    let article = Article::new(
        "harbor-cleanup",
        "Annual Harbor Environmental Cleanup Begins",
    )
    .with_color("#0EA5E9");
    let a_id = article.id;
    state.storage.create_article(article).expect("inserted");

    let task1 = Task::new(a_id, "Photograph cleanup crews at pier 4").with_status(TaskStatus::ToDo);
    let task2 =
        Task::new(a_id, "Review water quality sample data").with_status(TaskStatus::InProgress);
    let task3 = Task::new(a_id, "Confirm EPA permit status").with_status(TaskStatus::Complete);

    let t1_id = task1.id;
    let t2_id = task2.id;

    state.storage.create_task(task1).expect("inserted task 1");
    state.storage.create_task(task2).expect("inserted task 2");
    state.storage.create_task(task3).expect("inserted task 3");
    state.load_all().expect("state loaded");

    // 1. Cosmic Platform
    let cosmic_app = CosmicApp::new(state.clone());
    let cosmic_tree = cosmic_app.build_view_tree();
    let tasks_deck = cosmic_tree.tasks_deck.expect("cosmic tasks deck");
    assert_eq!(tasks_deck.total_task_count, 3);
    assert_eq!(tasks_deck.todo_count, 1);
    assert_eq!(tasks_deck.in_progress_count, 1);
    assert_eq!(tasks_deck.completed_count, 1);

    let todo_card = &tasks_deck.columns[0].cards[0];
    assert_eq!(todo_card.id, t1_id);
    assert_eq!(todo_card.parent_article_slug, "harbor-cleanup");
    assert_eq!(todo_card.accent_strip.color_hex, "#0EA5E9");
    assert_eq!(todo_card.status, TaskStatus::ToDo);

    // 2. macOS Platform
    let macos_app = MacosApp::new(state);
    let macos_tree = macos_app.build_view_tree();
    let macos_deck = macos_tree.tasks_deck.expect("macos tasks deck");
    assert_eq!(macos_deck.total_task_count, 3);
    assert_eq!(macos_deck.todo_count, 1);
    assert_eq!(macos_deck.in_progress_count, 1);
    assert_eq!(macos_deck.completed_count, 1);

    let in_prog_card = &macos_deck.columns[1].cards[0];
    assert_eq!(in_prog_card.id, t2_id);
    assert_eq!(in_prog_card.parent_article_slug, "harbor-cleanup");
    assert_eq!(in_prog_card.accent_strip.color_hex, "#0EA5E9");
    assert_eq!(in_prog_card.status, TaskStatus::InProgress);
}
