//! Comprehensive tests for Task 7.1: Tasks Kanban Deck Layout (3-Column Workflow Board).

use chrono::{Duration, Utc};
use newsjournal_core::models::{Article, Task, TaskStatus};
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::navigation::NavTab;
use newsjournal_gui::platform::cosmic::app::CosmicApp;
use newsjournal_gui::platform::macos::app::MacosApp;
use newsjournal_gui::state::drag_drop::{DragItem, DropTarget};
use newsjournal_gui::views::{
    build_tasks_kanban_deck, build_tasks_kanban_deck_with_layout, build_tasks_kanban_view,
    task_status_metadata, TasksDeckLayoutConfig, TasksKanbanDeckViewModel, DEFAULT_TASK_COLUMN_GAP,
    DEFAULT_TASK_COLUMN_WIDTH, DEFAULT_TASK_DECK_PADDING, MAX_TASK_COLUMN_WIDTH,
    MIN_TASK_COLUMN_WIDTH, NUM_TASK_STATUSES,
};
use newsjournal_gui::{AppState, EventLoop};

#[test]
fn test_task_status_metadata_canonical_ordering_and_properties() {
    let statuses = TaskStatus::all();
    assert_eq!(statuses.len(), NUM_TASK_STATUSES);

    let expected = [
        (
            TaskStatus::ToDo,
            0,
            "To-Do",
            "to_do",
            "📋",
            "circle",
            "#F59E0B",
        ),
        (
            TaskStatus::InProgress,
            1,
            "In Progress",
            "in_progress",
            "⏳",
            "clock",
            "#3B82F6",
        ),
        (
            TaskStatus::Complete,
            2,
            "Complete",
            "complete",
            "✅",
            "check-circle",
            "#10B981",
        ),
    ];

    for (status, idx, title, slug, emoji, icon, accent) in expected {
        let meta = task_status_metadata(status);
        assert_eq!(meta.status, status);
        assert_eq!(meta.index, idx);
        assert_eq!(meta.title, title);
        assert_eq!(meta.slug, slug);
        assert_eq!(meta.icon_emoji, emoji);
        assert_eq!(meta.icon_name, icon);
        assert_eq!(meta.accent_hex, accent);
        assert!(!meta.description.is_empty());
        assert!(!meta.sf_symbol.is_empty());
        assert!(!meta.empty_state_prompt.is_empty());
    }

    assert_eq!(
        TasksKanbanDeckViewModel::status_names(),
        ["To-Do", "In Progress", "Complete"]
    );
    assert_eq!(TasksKanbanDeckViewModel::statuses(), TaskStatus::all());
}

#[test]
fn test_empty_tasks_kanban_deck_layout() {
    let state = AppState::in_memory().expect("in-memory state");
    let deck = build_tasks_kanban_deck(&state);

    assert_eq!(deck.columns.len(), 3);
    assert_eq!(deck.total_task_count, 0);
    assert_eq!(deck.todo_count, 0);
    assert_eq!(deck.in_progress_count, 0);
    assert_eq!(deck.completed_count, 0);
    assert_eq!(deck.total_overdue_count, 0);
    assert_eq!(deck.total_due_soon_count, 0);
    assert!(deck.is_empty());
    assert!(!deck.has_overdue());
    assert!(!deck.is_filtered);
    assert_eq!(deck.search_query, "");
    assert_eq!(deck.selected_article_id, None);

    // Toolbar metrics
    assert_eq!(deck.toolbar.title, "Tasks Kanban");
    assert_eq!(deck.toolbar.total_task_count, 0);
    assert_eq!(deck.toolbar.total_count_label, "0 tasks");
    assert_eq!(deck.toolbar.overdue_badge_label, None);
    assert_eq!(deck.toolbar.due_soon_badge_label, None);
    assert_eq!(deck.toolbar.primary_action_label, "+ New Task");

    // Columns structure
    for (i, &status) in TaskStatus::all().iter().enumerate() {
        let col = &deck.columns[i];
        assert_eq!(col.status, status);
        assert_eq!(col.index, i);
        assert_eq!(col.task_count, 0);
        assert!(col.cards.is_empty());
        assert!(col.is_empty);
        assert_eq!(col.header.task_count, 0);
        assert_eq!(col.header.task_count_label, "0 tasks");
        assert_eq!(col.header.count_badge_text, "0");
        assert!(col.empty_state.is_some());
    }

    // Deck-wide welcome empty state
    let empty_state = deck.deck_empty_state.as_ref().unwrap();
    assert_eq!(empty_state.headline, "Welcome to Tasks Kanban");
    assert!(!empty_state.is_filtered);
    assert_eq!(empty_state.action_button_label, "+ Create Your First Task");
}

#[test]
fn test_tasks_deck_layout_geometry_and_scrolling() {
    let mut layout = TasksDeckLayoutConfig::new(1000.0).with_column_width(300.0);

    assert_eq!(layout.column_width, 300.0);
    assert_eq!(layout.column_gap, DEFAULT_TASK_COLUMN_GAP);
    assert_eq!(layout.deck_padding, DEFAULT_TASK_DECK_PADDING);

    // Total content width: 3 * 300 + 2 * 18 + 2 * 20 = 900 + 36 + 40 = 976.0
    let expected_width =
        (3.0 * 300.0) + (2.0 * DEFAULT_TASK_COLUMN_GAP) + (2.0 * DEFAULT_TASK_DECK_PADDING);
    assert_eq!(layout.total_content_width(), expected_width);

    // Viewport is 1000.0 >= 976.0, so max_scroll is 0.0
    assert_eq!(layout.max_scroll_offset(), 0.0);
    assert!(!layout.can_scroll_left());
    assert!(!layout.can_scroll_right());

    // Narrow viewport to trigger horizontal scrolling
    layout.set_viewport_width(600.0);
    let max_scroll = layout.max_scroll_offset();
    assert_eq!(max_scroll, 976.0 - 600.0); // 376.0
    assert!(!layout.can_scroll_left());
    assert!(layout.can_scroll_right());

    // Column positions
    let x0 = layout.column_x_position(0);
    let x1 = layout.column_x_position(1);
    let x2 = layout.column_x_position(2);

    assert_eq!(x0, DEFAULT_TASK_DECK_PADDING);
    assert_eq!(
        x1,
        DEFAULT_TASK_DECK_PADDING + 300.0 + DEFAULT_TASK_COLUMN_GAP
    );
    assert_eq!(
        x2,
        DEFAULT_TASK_DECK_PADDING + 2.0 * (300.0 + DEFAULT_TASK_COLUMN_GAP)
    );

    // Visibility before scrolling
    assert!(layout.is_column_fully_visible(0));
    assert!(layout.is_column_visible(1));
    assert!(!layout.is_column_visible(2));
    assert_eq!(layout.visible_column_indices(), vec![0, 1]);

    // Scroll to column 2 (Complete)
    layout.scroll_to_status(TaskStatus::Complete);
    assert!(layout.can_scroll_left());
    assert!(layout.is_column_visible(2));

    // Screen coordinate mapping
    let mapped_col = layout.column_index_at_screen_x(layout.column_screen_x(1) + 10.0);
    assert_eq!(mapped_col, Some(1));
    let mapped_status = layout.status_at_screen_x(layout.column_screen_x(2) + 10.0);
    assert_eq!(mapped_status, Some(TaskStatus::Complete));

    // Clamping min/max column widths
    let mut config = TasksDeckLayoutConfig::default();
    config.set_column_width(50.0);
    assert_eq!(config.column_width, MIN_TASK_COLUMN_WIDTH);
    config.set_column_width(1000.0);
    assert_eq!(config.column_width, MAX_TASK_COLUMN_WIDTH);
}

#[test]
fn test_populated_tasks_kanban_deck() {
    let mut state = AppState::in_memory().expect("in-memory state");

    // Create 2 articles
    let a1 = Article::new("city-council", "City Council Approves Budget");
    let a1_id = a1.id;
    let a2 = Article::new("transit-strike", "Transit Workers Issue Strike Notice");
    let a2_id = a2.id;

    state.storage.create_article(a1).unwrap();
    state.storage.create_article(a2).unwrap();

    // Create tasks in different statuses
    let t1 = Task::new(a1_id, "Interview Mayor")
        .with_notes("Key questions on property tax hike")
        .with_status(TaskStatus::ToDo);
    let t2 = Task::new(a1_id, "Review budget spreadsheet").with_status(TaskStatus::InProgress);
    let t3 = Task::new(a2_id, "Photograph picket line").with_status(TaskStatus::InProgress);
    let t4 = Task::new(a2_id, "Fact check union statement").with_status(TaskStatus::Complete);

    let t1_id = t1.id;
    let t4_id = t4.id;

    state.storage.create_task(t1).unwrap();
    state.storage.create_task(t2).unwrap();
    state.storage.create_task(t3).unwrap();
    state.storage.create_task(t4).unwrap();

    state.load_all().unwrap();

    let custom_layout =
        TasksDeckLayoutConfig::new(1400.0).with_column_width(DEFAULT_TASK_COLUMN_WIDTH);
    let deck = build_tasks_kanban_deck_with_layout(&state, custom_layout);

    assert_eq!(deck.total_task_count, 4);
    assert_eq!(deck.todo_count, 1);
    assert_eq!(deck.in_progress_count, 2);
    assert_eq!(deck.completed_count, 1);
    assert!(!deck.is_empty());
    assert_eq!(deck.deck_empty_state, None);

    // Check To-Do column
    let todo_col = deck.column(TaskStatus::ToDo).unwrap();
    assert_eq!(todo_col.task_count, 1);
    assert_eq!(todo_col.header.task_count_label, "1 task");
    assert_eq!(todo_col.cards[0].id, t1_id);
    assert_eq!(todo_col.cards[0].title, "Interview Mayor");
    assert_eq!(todo_col.cards[0].parent_article_slug, "city-council");
    assert_eq!(todo_col.empty_state, None);

    // Check In-Progress column
    let in_prog_col = deck.column(TaskStatus::InProgress).unwrap();
    assert_eq!(in_prog_col.task_count, 2);
    assert_eq!(in_prog_col.header.task_count_label, "2 tasks");

    // Check Complete column
    let complete_col = deck.column(TaskStatus::Complete).unwrap();
    assert_eq!(complete_col.task_count, 1);
    assert_eq!(complete_col.cards[0].id, t4_id);
    assert_eq!(complete_col.cards[0].parent_article_slug, "transit-strike");

    // Test card lookup
    let found = deck.card(t1_id);
    assert!(found.is_some());
    let (col, card) = found.unwrap();
    assert_eq!(col.status, TaskStatus::ToDo);
    assert_eq!(card.title, "Interview Mayor");

    // Test count helpers
    assert_eq!(deck.count_for_status(TaskStatus::ToDo), 1);
    assert_eq!(deck.count_for_status(TaskStatus::InProgress), 2);
    assert_eq!(deck.count_for_status(TaskStatus::Complete), 1);
}

#[test]
fn test_tasks_deck_overdue_and_due_soon_metrics() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let now = Utc::now();

    let article = Article::new("breaking-news", "Breaking News Pipeline");
    let article_id = article.id;
    state.storage.create_article(article).unwrap();

    // 1. Overdue task (past due in To-Do)
    let overdue_task = Task::new(article_id, "Get quote from chief")
        .with_due_date(now - Duration::hours(5))
        .with_status(TaskStatus::ToDo);

    // 2. Due soon task (due in 2 hours in InProgress)
    let due_soon_task = Task::new(article_id, "Draft section 1")
        .with_due_date(now + Duration::hours(2))
        .with_status(TaskStatus::InProgress);

    // 3. Completed task with past due date (NOT overdue because Complete)
    let complete_past_task = Task::new(article_id, "Confirm sources")
        .with_due_date(now - Duration::hours(10))
        .with_status(TaskStatus::Complete);

    state.storage.create_task(overdue_task).unwrap();
    state.storage.create_task(due_soon_task).unwrap();
    state.storage.create_task(complete_past_task).unwrap();

    state.load_all().unwrap();
    state.last_tick = now;

    let deck = build_tasks_kanban_deck(&state);

    assert_eq!(deck.total_overdue_count, 1);
    assert_eq!(deck.total_due_soon_count, 1);
    assert!(deck.has_overdue());

    // Toolbar badges
    assert_eq!(
        deck.toolbar.overdue_badge_label,
        Some("⚠️ 1 Overdue".to_string())
    );
    assert_eq!(
        deck.toolbar.due_soon_badge_label,
        Some("🕒 1 Due Soon".to_string())
    );

    // To-Do column overdue count
    let todo_col = deck.column(TaskStatus::ToDo).unwrap();
    assert_eq!(todo_col.overdue_count, 1);
    assert_eq!(
        todo_col.header.overdue_badge_label,
        Some("⚠️ 1 Overdue".to_string())
    );
    assert!(todo_col.cards[0].is_overdue);

    // In Progress column due soon count
    let in_prog_col = deck.column(TaskStatus::InProgress).unwrap();
    assert_eq!(in_prog_col.due_soon_count, 1);
    assert_eq!(
        in_prog_col.header.due_soon_badge_label,
        Some("🕒 1 Due Soon".to_string())
    );
    assert!(in_prog_col.cards[0].is_due_soon);

    // Complete column has 0 overdue
    let complete_col = deck.column(TaskStatus::Complete).unwrap();
    assert_eq!(complete_col.overdue_count, 0);
    assert!(!complete_col.cards[0].is_overdue);
}

#[test]
fn test_tasks_deck_filtering_by_article_and_search() {
    let mut state = AppState::in_memory().expect("in-memory state");

    let a1 = Article::new("investigation-one", "City Finance Audit");
    let a1_id = a1.id;
    let a2 = Article::new("sports-report", "High School Championship");
    let a2_id = a2.id;

    state.storage.create_article(a1).unwrap();
    state.storage.create_article(a2).unwrap();

    let t1 = Task::new(a1_id, "Review FOIA documents").with_status(TaskStatus::ToDo);
    let t2 = Task::new(a1_id, "Interview auditor").with_status(TaskStatus::InProgress);
    let t3 = Task::new(a2_id, "Interview coach").with_status(TaskStatus::ToDo);

    state.storage.create_task(t1).unwrap();
    state.storage.create_task(t2).unwrap();
    state.storage.create_task(t3).unwrap();
    state.load_all().unwrap();

    // 1. Filter by article a1
    state.filters.selected_article_id = Some(a1_id);
    let deck1 = build_tasks_kanban_deck(&state);

    assert_eq!(deck1.total_task_count, 2);
    assert_eq!(deck1.selected_article_id, Some(a1_id));
    assert_eq!(
        deck1.selected_article_slug,
        Some("investigation-one".to_string())
    );
    assert!(deck1.is_filtered);
    assert_eq!(deck1.count_for_status(TaskStatus::ToDo), 1);
    assert_eq!(deck1.count_for_status(TaskStatus::InProgress), 1);
    assert_eq!(deck1.count_for_status(TaskStatus::Complete), 0);

    // 2. Search filter
    state.filters.selected_article_id = None;
    state.filters.search_query = "coach".to_string();
    let deck2 = build_tasks_kanban_deck(&state);

    assert_eq!(deck2.total_task_count, 1);
    assert_eq!(deck2.columns[0].cards[0].title, "Interview coach");

    // 3. Filter resulting in 0 results
    state.filters.search_query = "nonexistent-query-xyz".to_string();
    let deck3 = build_tasks_kanban_deck(&state);

    assert_eq!(deck3.total_task_count, 0);
    assert!(deck3.deck_empty_state.is_some());
    let empty = deck3.deck_empty_state.unwrap();
    assert_eq!(empty.headline, "No Matching Tasks");
    assert!(empty.is_filtered);
    assert_eq!(empty.action_button_label, "Clear Filters");
}

#[test]
fn test_tasks_deck_drag_drop_session() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let article = Article::new("court-trial", "High Court Trial Coverage");
    let a_id = article.id;
    state.storage.create_article(article).unwrap();

    let task = Task::new(a_id, "Record jury verdict").with_status(TaskStatus::ToDo);
    let task_id = task.id;
    state.storage.create_task(task).unwrap();
    state.load_all().unwrap();

    // Start dragging task
    state.drag.start_drag(DragItem::TaskCard {
        id: task_id,
        article_id: a_id,
        origin_status: TaskStatus::ToDo,
    });
    state
        .drag
        .update_hover(Some(DropTarget::TaskColumn(TaskStatus::InProgress)));

    let deck = build_tasks_kanban_deck(&state);
    assert!(deck.is_dragging);
    assert_eq!(
        deck.hover_target,
        Some(DropTarget::TaskColumn(TaskStatus::InProgress))
    );
    assert!(deck.drag_ghost.is_some());

    let ghost = deck.drag_ghost.as_ref().unwrap();
    assert_eq!(ghost.id, task_id);
    assert_eq!(ghost.title, "Record jury verdict");
    assert!(ghost.is_valid_target);
    assert_eq!(ghost.badge_label, "MOVE TO IN PROGRESS");

    // In Progress column has active drop target placeholder
    let in_prog = deck.column(TaskStatus::InProgress).unwrap();
    assert!(in_prog.is_active_drop_target);
    assert!(in_prog.drop_placeholder.is_some());
}

#[test]
fn test_tasks_deck_event_loop_messages() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let article = Article::new("mayor-race", "Mayoral Election Debates");
    let a_id = article.id;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    // 1. Dispatch SetArticleFilter
    event_loop
        .dispatch(AppMessage::SetArticleFilter(Some(a_id)))
        .unwrap();
    assert_eq!(event_loop.state().filters.selected_article_id, Some(a_id));

    // 2. Dispatch OpenNewTaskInStatusModal
    event_loop
        .dispatch(AppMessage::OpenNewTaskInStatusModal(
            TaskStatus::InProgress,
            Some(a_id),
        ))
        .unwrap();
    assert!(event_loop.state().modal.is_open());
    if let newsjournal_gui::state::modal::ModalState::TaskForm(draft) = &event_loop.state().modal {
        assert_eq!(draft.status, TaskStatus::InProgress);
        assert_eq!(draft.article_id, Some(a_id));
    } else {
        panic!("Expected ModalState::TaskForm");
    }
}

#[test]
fn test_tasks_deck_in_cosmic_and_macos_view_trees() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let article = Article::new("election-2026", "Local Election Coverage");
    let a_id = article.id;
    state.storage.create_article(article).unwrap();

    let task = Task::new(a_id, "Candidate profiles").with_status(TaskStatus::ToDo);
    state.storage.create_task(task).unwrap();
    state.load_all().unwrap();

    // Switch to TasksKanban tab
    state.active_tab = NavTab::TasksKanban;

    // COSMIC App View Tree
    let cosmic_app = CosmicApp::new(state.clone());
    let cosmic_tree = cosmic_app.build_view_tree();

    assert_eq!(cosmic_tree.active_tab, NavTab::TasksKanban);
    assert!(cosmic_tree.tasks_deck.is_some());
    assert!(cosmic_tree.tasks_view.is_some());
    let cosmic_deck = cosmic_tree.tasks_deck.unwrap();
    assert_eq!(cosmic_deck.total_task_count, 1);
    assert_eq!(cosmic_deck.columns[0].cards[0].title, "Candidate profiles");

    // macOS App View Tree
    let macos_app = MacosApp::new(state);
    let macos_tree = macos_app.build_view_tree();

    assert_eq!(macos_tree.active_tab, NavTab::TasksKanban);
    assert!(macos_tree.tasks_deck.is_some());
    assert!(macos_tree.tasks_view.is_some());
    let macos_deck = macos_tree.tasks_deck.unwrap();
    assert_eq!(macos_deck.total_task_count, 1);
    assert_eq!(macos_deck.columns[0].cards[0].title, "Candidate profiles");
}

#[test]
fn test_build_tasks_kanban_view_backward_compatibility() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let a = Article::new("headline-slug", "Sample Headline");
    let a_id = a.id;
    state.storage.create_article(a).unwrap();

    let t = Task::new(a_id, "Sample Task").with_status(TaskStatus::InProgress);
    state.storage.create_task(t).unwrap();
    state.load_all().unwrap();

    let columns = build_tasks_kanban_view(&state);
    assert_eq!(columns.len(), 3);
    assert_eq!(columns[0].status, TaskStatus::ToDo);
    assert_eq!(columns[1].status, TaskStatus::InProgress);
    assert_eq!(columns[2].status, TaskStatus::Complete);
    assert_eq!(columns[1].task_count, 1);
    assert_eq!(columns[1].cards[0].title, "Sample Task");
}
