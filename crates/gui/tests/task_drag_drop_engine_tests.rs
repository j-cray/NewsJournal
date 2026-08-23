//! Comprehensive integration test suite for Task 7.3: Tasks Kanban Drag-and-Drop Movement Engine.

use newsjournal_core::models::{Article, Task, TaskStatus};
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::navigation::NavTab;
use newsjournal_gui::platform::cosmic::app::CosmicApp;
use newsjournal_gui::platform::macos::app::MacosApp;
use newsjournal_gui::state::drag_drop::{DragItem, DropTarget};
use newsjournal_gui::views::{build_tasks_kanban_deck_with_layout, TasksDeckLayoutConfig};
use newsjournal_gui::{AppState, EventLoop};

#[test]
fn test_task_pointer_click_and_drag_mechanics_and_threshold() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let article = Article::new("city-transit", "City Light Rail Expansion");
    let article_id = article.id;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    let task = Task::new(article_id, "Interview Transit Director");
    let task_id = task.id;
    event_loop.dispatch(AppMessage::CreateTask(task)).unwrap();

    // 1. Initial idle state
    assert!(!event_loop.state().drag.is_dragging());
    assert_eq!(event_loop.state().drag.pointer_pos, None);

    // 2. Start drag with initial pointer coordinates (x: 140.0, y: 250.0)
    event_loop
        .dispatch(AppMessage::DragStartWithPos {
            item: DragItem::TaskCard {
                id: task_id,
                article_id,
                origin_status: TaskStatus::ToDo,
            },
            pos: (140.0, 250.0),
        })
        .unwrap();

    assert!(event_loop.state().drag.is_dragging());
    assert_eq!(event_loop.state().drag.drag_start_pos, Some((140.0, 250.0)));
    assert_eq!(event_loop.state().drag.pointer_pos, Some((140.0, 250.0)));
    assert_eq!(event_loop.state().drag.drag_delta(), (0.0, 0.0));
    assert_eq!(event_loop.state().drag.drag_distance(), 0.0);
    assert!(!event_loop.state().drag.has_exceeded_drag_threshold(6.0));

    // 3. Move pointer slightly (below threshold)
    event_loop
        .dispatch(AppMessage::DragMove {
            pointer_pos: (143.0, 254.0),
        })
        .unwrap();
    assert_eq!(event_loop.state().drag.drag_delta(), (3.0, 4.0));
    assert_eq!(event_loop.state().drag.drag_distance(), 5.0);
    assert!(!event_loop.state().drag.has_exceeded_drag_threshold(6.0));
    assert!(event_loop.state().drag.has_exceeded_drag_threshold(4.0));

    // 4. Move pointer across columns (large movement)
    event_loop
        .dispatch(AppMessage::DragMove {
            pointer_pos: (460.0, 280.0),
        })
        .unwrap();
    assert!(event_loop.state().drag.has_exceeded_drag_threshold(10.0));
    assert_eq!(event_loop.state().drag.drag_delta(), (320.0, 30.0));
}

#[test]
fn test_task_screen_hit_testing_and_scroll_offset_geometry() {
    let mut layout = TasksDeckLayoutConfig::new(1280.0)
        .with_column_width(360.0)
        .with_scroll(0.0);

    // Padding = 20.0, Column width = 360.0, Gap = 18.0
    // Col 0 (To-Do): [20.0, 380.0]
    // Gap: [380.0, 398.0]
    // Col 1 (In Progress): [398.0, 758.0]
    // Gap: [758.0, 776.0]
    // Col 2 (Complete): [776.0, 1136.0]

    assert_eq!(layout.column_index_at_screen_x(100.0), Some(0));
    assert_eq!(layout.status_at_screen_x(100.0), Some(TaskStatus::ToDo));

    assert_eq!(layout.column_index_at_screen_x(500.0), Some(1));
    assert_eq!(
        layout.status_at_screen_x(500.0),
        Some(TaskStatus::InProgress)
    );

    assert_eq!(layout.column_index_at_screen_x(900.0), Some(2));
    assert_eq!(layout.status_at_screen_x(900.0), Some(TaskStatus::Complete));

    // Gap between col 0 and 1 -> returns None
    assert_eq!(layout.column_index_at_screen_x(388.0), None);
    assert_eq!(layout.status_at_screen_x(388.0), None);

    // Test with narrower viewport and horizontal scroll offset = 378.0 (scrolled 1 column right)
    layout.set_viewport_width(600.0);
    let scrolled_layout = layout.with_scroll(378.0);
    // Col 1 (In Progress) screen bounds: [398.0 - 378.0, 758.0 - 378.0] = [20.0, 380.0]
    assert_eq!(scrolled_layout.column_index_at_screen_x(100.0), Some(1));
    assert_eq!(
        scrolled_layout.status_at_screen_x(100.0),
        Some(TaskStatus::InProgress)
    );
}

#[test]
fn test_task_visual_drag_ghost_and_source_card_dimming() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let article = Article::new("clean-energy", "Wind Power Grid Integration")
        .with_description("In-depth investigative report on wind farm grid integration.");
    let article_id = article.id;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    let task = Task::new(article_id, "Interview grid engineers")
        .with_notes("Key contact: Dr. Elizabeth Thorne");
    let task_id = task.id;
    event_loop.dispatch(AppMessage::CreateTask(task)).unwrap();

    // 1. Build idle deck view model
    let idle_deck =
        build_tasks_kanban_deck_with_layout(event_loop.state(), TasksDeckLayoutConfig::default());
    assert!(!idle_deck.is_dragging);
    assert!(idle_deck.drag_ghost.is_none());

    let idle_card = &idle_deck.columns[0].cards[0];
    assert!(!idle_card.is_dragging);
    assert_eq!(idle_card.drag_opacity, 1.0);

    // 2. Start drag with pointer coordinates (x: 180.0, y: 220.0)
    event_loop
        .dispatch(AppMessage::DragStartWithPos {
            item: DragItem::TaskCard {
                id: task_id,
                article_id,
                origin_status: TaskStatus::ToDo,
            },
            pos: (180.0, 220.0),
        })
        .unwrap();

    let dragging_deck =
        build_tasks_kanban_deck_with_layout(event_loop.state(), TasksDeckLayoutConfig::default());
    assert!(dragging_deck.is_dragging);
    assert!(dragging_deck.drag_ghost.is_some());

    // Verify source card in To-Do column is dimmed
    let source_card = &dragging_deck.columns[0].cards[0];
    assert!(source_card.is_dragging);
    assert_eq!(source_card.drag_opacity, 0.35);

    // Verify Drag Ghost view model properties
    let ghost = dragging_deck.drag_ghost.as_ref().unwrap();
    assert_eq!(ghost.id, task_id);
    assert_eq!(ghost.title, "Interview grid engineers");
    assert_eq!(ghost.parent_article_slug, "clean-energy");
    assert_eq!(ghost.origin_status, TaskStatus::ToDo);
    assert_eq!(ghost.pointer_pos, (180.0, 220.0));
    assert_eq!(ghost.tilt_degrees, 2.5);
    assert_eq!(ghost.scale, 1.03);
    assert_eq!(ghost.opacity, 0.92);
    assert_eq!(ghost.shadow_blur_px, 24.0);
    assert_eq!(ghost.badge_label, "DRAGGING TASK");
    assert!(!ghost.is_valid_target);

    // 3. Hover over valid target column (In Progress)
    event_loop
        .dispatch(AppMessage::DragHover(Some(DropTarget::TaskColumn(
            TaskStatus::InProgress,
        ))))
        .unwrap();

    let hover_deck =
        build_tasks_kanban_deck_with_layout(event_loop.state(), TasksDeckLayoutConfig::default());
    let updated_ghost = hover_deck.drag_ghost.unwrap();
    assert!(updated_ghost.is_valid_target);
    assert_eq!(updated_ghost.badge_label, "MOVE TO IN PROGRESS");
    assert_eq!(updated_ghost.border_color_hex, "#10B981"); // Green valid drop border

    // 4. Hover over valid target column (Complete)
    event_loop
        .dispatch(AppMessage::DragHover(Some(DropTarget::TaskColumn(
            TaskStatus::Complete,
        ))))
        .unwrap();

    let hover_deck_complete =
        build_tasks_kanban_deck_with_layout(event_loop.state(), TasksDeckLayoutConfig::default());
    let ghost_complete = hover_deck_complete.drag_ghost.unwrap();
    assert!(ghost_complete.is_valid_target);
    assert_eq!(ghost_complete.badge_label, "MOVE TO COMPLETE");
    assert_eq!(ghost_complete.border_color_hex, "#10B981");

    // 5. Hover over same origin column (To-Do)
    event_loop
        .dispatch(AppMessage::DragHover(Some(DropTarget::TaskColumn(
            TaskStatus::ToDo,
        ))))
        .unwrap();

    let invalid_hover_deck =
        build_tasks_kanban_deck_with_layout(event_loop.state(), TasksDeckLayoutConfig::default());
    let invalid_ghost = invalid_hover_deck.drag_ghost.unwrap();
    assert!(!invalid_ghost.is_valid_target);
    assert_eq!(invalid_ghost.badge_label, "SAME STATUS");
}

#[test]
fn test_task_visual_drop_placeholders_and_column_highlights() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let article = Article::new("water-rights", "Western River Water Rights Dispute");
    let article_id = article.id;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    let task = Task::new(article_id, "Request Basin Authority Records");
    let task_id = task.id;
    event_loop.dispatch(AppMessage::CreateTask(task)).unwrap();

    // Start drag from To-Do
    event_loop
        .dispatch(AppMessage::DragStartWithPos {
            item: DragItem::TaskCard {
                id: task_id,
                article_id,
                origin_status: TaskStatus::ToDo,
            },
            pos: (200.0, 200.0),
        })
        .unwrap();

    // Hover over In Progress column with insertion index 0
    event_loop
        .dispatch(AppMessage::DragHoverWithIndex {
            target: Some(DropTarget::TaskColumn(TaskStatus::InProgress)),
            insert_index: Some(0),
        })
        .unwrap();

    let deck =
        build_tasks_kanban_deck_with_layout(event_loop.state(), TasksDeckLayoutConfig::default());

    // Col 0 (To-Do - origin): not valid target
    let col_todo = &deck.columns[0];
    assert!(!col_todo.is_valid_drop_target);
    assert!(!col_todo.is_active_drop_target);
    assert!(col_todo.drop_placeholder.is_none());

    // Col 1 (In Progress - hovered target): active drop target
    let col_prog = &deck.columns[1];
    assert!(col_prog.is_hovered);
    assert!(col_prog.is_valid_drop_target);
    assert!(col_prog.is_active_drop_target);
    assert_eq!(
        col_prog.drop_highlight_border_hex,
        Some("#3B82F6".to_string())
    );
    assert!(col_prog.drop_placeholder.is_some());

    let placeholder = col_prog.drop_placeholder.as_ref().unwrap();
    assert_eq!(placeholder.target_status, TaskStatus::InProgress);
    assert_eq!(placeholder.insert_index, 0);
    assert_eq!(placeholder.border_style, "dashed");
    assert!(placeholder.is_valid);
    assert!(placeholder.is_pulse_active);
    assert_eq!(
        placeholder.prompt_text,
        "+ Move \"Request Basin Authority Records\" to In Progress"
    );

    // Hover over same column (To-Do - invalid target)
    event_loop
        .dispatch(AppMessage::DragHover(Some(DropTarget::TaskColumn(
            TaskStatus::ToDo,
        ))))
        .unwrap();

    let invalid_hover_deck =
        build_tasks_kanban_deck_with_layout(event_loop.state(), TasksDeckLayoutConfig::default());
    let col_todo_hover = &invalid_hover_deck.columns[0];
    assert!(col_todo_hover.is_hovered);
    assert!(!col_todo_hover.is_valid_drop_target);
    assert!(!col_todo_hover.is_active_drop_target);
    assert_eq!(
        col_todo_hover.drop_highlight_border_hex,
        Some("#EF4444".to_string())
    );

    let invalid_placeholder = col_todo_hover.drop_placeholder.as_ref().unwrap();
    assert!(!invalid_placeholder.is_valid);
    assert_eq!(
        invalid_placeholder.prompt_text,
        "Cannot drop in current column"
    );
}

#[test]
fn test_task_status_change_dispatch_and_sqlite_persistence() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let article = Article::new("election-audit", "County Ballot Recount Verification");
    let article_id = article.id;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    let task = Task::new(article_id, "Interview Election Commissioners");
    let task_id = task.id;
    event_loop.dispatch(AppMessage::CreateTask(task)).unwrap();

    assert_eq!(event_loop.state().tasks[0].status, TaskStatus::ToDo);

    let workflow_transitions = [
        (TaskStatus::ToDo, TaskStatus::InProgress),
        (TaskStatus::InProgress, TaskStatus::Complete),
        (TaskStatus::Complete, TaskStatus::ToDo),
    ];

    for (from_status, to_status) in workflow_transitions {
        // 1. Start drag from from_status
        event_loop
            .dispatch(AppMessage::DragStartWithPos {
                item: DragItem::TaskCard {
                    id: task_id,
                    article_id,
                    origin_status: from_status,
                },
                pos: (150.0, 200.0),
            })
            .unwrap();
        assert!(event_loop.state().drag.is_dragging());

        // 2. Hover destination status
        event_loop
            .dispatch(AppMessage::DragHover(Some(DropTarget::TaskColumn(
                to_status,
            ))))
            .unwrap();
        assert!(event_loop.state().drag.is_valid_drop());

        // 3. Drop
        event_loop.dispatch(AppMessage::DragDrop).unwrap();
        assert!(!event_loop.state().drag.is_dragging());
        assert_eq!(event_loop.state().tasks[0].status, to_status);

        // 4. Verify SQLite storage persistence
        let from_storage = event_loop
            .state()
            .storage
            .get_task(task_id)
            .unwrap()
            .expect("task exists in SQLite");
        assert_eq!(from_storage.status, to_status);

        // Verify toast notification was queued
        let latest_toast = event_loop.state().toasts.last().unwrap();
        assert_eq!(latest_toast.title, "Task Moved");
        assert!(latest_toast.body.contains(to_status.display_name()));
    }
}

#[test]
fn test_task_drag_cancellation_and_clean_state_reset() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let article = Article::new("housing-crisis", "Downtown Rent Stabilization Hearing");
    let article_id = article.id;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    let task = Task::new(article_id, "Collect tenant statements");
    let task_id = task.id;
    event_loop.dispatch(AppMessage::CreateTask(task)).unwrap();

    // Start drag
    event_loop
        .dispatch(AppMessage::DragStartWithPos {
            item: DragItem::TaskCard {
                id: task_id,
                article_id,
                origin_status: TaskStatus::ToDo,
            },
            pos: (150.0, 200.0),
        })
        .unwrap();
    event_loop
        .dispatch(AppMessage::DragMove {
            pointer_pos: (250.0, 350.0),
        })
        .unwrap();
    event_loop
        .dispatch(AppMessage::DragHover(Some(DropTarget::TaskColumn(
            TaskStatus::InProgress,
        ))))
        .unwrap();

    let active_deck =
        build_tasks_kanban_deck_with_layout(event_loop.state(), TasksDeckLayoutConfig::default());
    assert!(active_deck.is_dragging);
    assert!(active_deck.drag_ghost.is_some());
    assert_eq!(active_deck.columns[0].cards[0].drag_opacity, 0.35);

    // Cancel drag explicitly
    event_loop.dispatch(AppMessage::DragCancel).unwrap();

    let cancelled_deck =
        build_tasks_kanban_deck_with_layout(event_loop.state(), TasksDeckLayoutConfig::default());
    assert!(!cancelled_deck.is_dragging);
    assert!(cancelled_deck.drag_ghost.is_none());
    assert_eq!(cancelled_deck.columns[0].cards[0].drag_opacity, 1.0);
    assert_eq!(cancelled_deck.columns[0].cards[0].status, TaskStatus::ToDo);
    assert_eq!(event_loop.state().drag.drag_start_pos, None);
    assert_eq!(event_loop.state().drag.pointer_pos, None);
    assert_eq!(event_loop.state().drag.hover_target, None);
}

#[test]
fn test_task_invalid_drop_target_rejection() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let article = Article::new("court-ruling", "Federal Appeals Court Ruling");
    let article_id = article.id;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    let task = Task::new(article_id, "Review 80-page decision");
    let task_id = task.id;
    event_loop.dispatch(AppMessage::CreateTask(task)).unwrap();

    // Start drag
    event_loop
        .dispatch(AppMessage::DragStart(DragItem::TaskCard {
            id: task_id,
            article_id,
            origin_status: TaskStatus::ToDo,
        }))
        .unwrap();

    // Hover over an Article Column (invalid target for task card)
    event_loop
        .dispatch(AppMessage::DragHover(Some(DropTarget::ArticleColumn(
            newsjournal_core::ArticleStage::Editing,
        ))))
        .unwrap();

    assert!(!event_loop.state().drag.is_valid_drop());

    // Dropping on an invalid target must cancel drag and leave task status unchanged
    event_loop.dispatch(AppMessage::DragDrop).unwrap();
    assert!(!event_loop.state().drag.is_dragging());
    assert_eq!(event_loop.state().tasks[0].status, TaskStatus::ToDo);
}

#[test]
fn test_cosmic_and_macos_app_task_drag_and_drop_view_tree_integration() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut cosmic_app = CosmicApp::new(state.clone());
    let mut macos_app = MacosApp::new(state);

    let article = Article::new("solar-incentives", "State Expands Solar Tax Incentives");
    let article_id = article.id;

    cosmic_app
        .dispatch(AppMessage::CreateArticle(article.clone()))
        .unwrap();
    macos_app
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    let task = Task::new(article_id, "Compile solar installer survey data");
    let task_id = task.id;

    cosmic_app
        .dispatch(AppMessage::CreateTask(task.clone()))
        .unwrap();
    macos_app.dispatch(AppMessage::CreateTask(task)).unwrap();

    // Switch to Tasks tab
    cosmic_app
        .dispatch(AppMessage::NavigateTo(NavTab::TasksKanban))
        .unwrap();
    macos_app
        .dispatch(AppMessage::NavigateTo(NavTab::TasksKanban))
        .unwrap();

    // 1. Verify idle view tree
    let cosmic_tree = cosmic_app.build_view_tree();
    let cosmic_deck = cosmic_tree.tasks_deck.expect("tasks deck exists");
    assert!(!cosmic_deck.is_dragging);
    assert!(cosmic_deck.drag_ghost.is_none());
    assert_eq!(cosmic_deck.columns[0].task_count, 1);

    let macos_tree = macos_app.build_view_tree();
    let macos_deck = macos_tree.tasks_deck.expect("tasks deck exists");
    assert!(!macos_deck.is_dragging);
    assert!(macos_deck.drag_ghost.is_none());
    assert_eq!(macos_deck.columns[0].task_count, 1);

    // 2. Start drag on COSMIC app
    cosmic_app
        .dispatch(AppMessage::DragStartWithPos {
            item: DragItem::TaskCard {
                id: task_id,
                article_id,
                origin_status: TaskStatus::ToDo,
            },
            pos: (200.0, 300.0),
        })
        .unwrap();
    cosmic_app
        .dispatch(AppMessage::DragHover(Some(DropTarget::TaskColumn(
            TaskStatus::InProgress,
        ))))
        .unwrap();

    let cosmic_active_tree = cosmic_app.build_view_tree();
    let cosmic_active_deck = cosmic_active_tree.tasks_deck.unwrap();
    assert!(cosmic_active_deck.is_dragging);
    assert!(cosmic_active_deck.drag_ghost.is_some());
    assert_eq!(cosmic_active_deck.columns[0].cards[0].drag_opacity, 0.35);
    assert!(cosmic_active_deck.columns[1].is_active_drop_target);

    // 3. Drop on COSMIC app
    cosmic_app.dispatch(AppMessage::DragDrop).unwrap();
    let cosmic_dropped_tree = cosmic_app.build_view_tree();
    let cosmic_dropped_deck = cosmic_dropped_tree.tasks_deck.unwrap();
    assert!(!cosmic_dropped_deck.is_dragging);
    assert_eq!(cosmic_dropped_deck.columns[0].task_count, 0);
    assert_eq!(cosmic_dropped_deck.columns[1].task_count, 1);
    assert_eq!(
        cosmic_dropped_deck.columns[1].cards[0].status,
        TaskStatus::InProgress
    );

    // 4. Start drag on macOS app (move from InProgress to Complete)
    macos_app
        .dispatch(AppMessage::DragStartWithPos {
            item: DragItem::TaskCard {
                id: task_id,
                article_id,
                origin_status: TaskStatus::InProgress,
            },
            pos: (350.0, 400.0),
        })
        .unwrap();
    macos_app
        .dispatch(AppMessage::DragHover(Some(DropTarget::TaskColumn(
            TaskStatus::Complete,
        ))))
        .unwrap();

    let macos_active_tree = macos_app.build_view_tree();
    let macos_active_deck = macos_active_tree.tasks_deck.unwrap();
    assert!(macos_active_deck.is_dragging);
    assert!(macos_active_deck.drag_ghost.is_some());
    assert!(macos_active_deck.columns[2].is_active_drop_target);

    // 5. Drop on macOS app
    macos_app.dispatch(AppMessage::DragDrop).unwrap();
    let macos_dropped_tree = macos_app.build_view_tree();
    let macos_dropped_deck = macos_dropped_tree.tasks_deck.unwrap();
    assert!(!macos_dropped_deck.is_dragging);
    assert_eq!(macos_dropped_deck.columns[1].task_count, 0);
    assert_eq!(macos_dropped_deck.columns[2].task_count, 1);
    assert_eq!(
        macos_dropped_deck.columns[2].cards[0].status,
        TaskStatus::Complete
    );
}
