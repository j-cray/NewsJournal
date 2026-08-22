//! Drag and drop Kanban interaction tests.

use newsjournal_core::models::{Article, Task, TaskStatus};
use newsjournal_core::ArticleStage;
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::state::drag_drop::{DragItem, DropTarget};
use newsjournal_gui::{AppState, EventLoop};

#[test]
fn test_article_drag_and_drop_across_kanban_columns() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let article = Article::new("housing-crisis", "Affordable Housing Crunch Deepens");
    let article_id = article.id;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();
    assert_eq!(event_loop.state().articles[0].stage, ArticleStage::Pitching);

    // 1. Start drag
    event_loop
        .dispatch(AppMessage::DragStart(DragItem::ArticleCard {
            id: article_id,
            origin_stage: ArticleStage::Pitching,
        }))
        .unwrap();
    assert!(event_loop.state().drag.is_dragging());

    // 2. Hover over Writing column
    event_loop
        .dispatch(AppMessage::DragHover(Some(DropTarget::ArticleColumn(
            ArticleStage::Writing,
        ))))
        .unwrap();
    assert!(event_loop.state().drag.is_valid_drop());

    // 3. Drop
    event_loop.dispatch(AppMessage::DragDrop).unwrap();
    assert!(!event_loop.state().drag.is_dragging());
    assert_eq!(event_loop.state().articles[0].stage, ArticleStage::Writing);

    // Verify storage persistence
    let from_storage = event_loop
        .state()
        .storage
        .get_article(article_id)
        .unwrap()
        .unwrap();
    assert_eq!(from_storage.stage, ArticleStage::Writing);
}

#[test]
fn test_article_drag_invalid_target_and_cancellation() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let article = Article::new("transit-budget", "Transit Budget Review");
    let article_id = article.id;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    // Start drag
    event_loop
        .dispatch(AppMessage::DragStart(DragItem::ArticleCard {
            id: article_id,
            origin_stage: ArticleStage::Pitching,
        }))
        .unwrap();

    // Hover over invalid target type (Task column)
    event_loop
        .dispatch(AppMessage::DragHover(Some(DropTarget::TaskColumn(
            TaskStatus::InProgress,
        ))))
        .unwrap();
    assert!(!event_loop.state().drag.is_valid_drop());

    // Drop on invalid target should cancel cleanly
    event_loop.dispatch(AppMessage::DragDrop).unwrap();
    assert!(!event_loop.state().drag.is_dragging());
    assert_eq!(event_loop.state().articles[0].stage, ArticleStage::Pitching);

    // Explicit drag cancel
    event_loop
        .dispatch(AppMessage::DragStart(DragItem::ArticleCard {
            id: article_id,
            origin_stage: ArticleStage::Pitching,
        }))
        .unwrap();
    assert!(event_loop.state().drag.is_dragging());
    event_loop.dispatch(AppMessage::DragCancel).unwrap();
    assert!(!event_loop.state().drag.is_dragging());
}

#[test]
fn test_task_drag_and_drop_across_columns() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let article = Article::new("harbor-clean", "Harbor Cleanup Operation");
    let article_id = article.id;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    let task = Task::new(article_id, "Sample water quality");
    let task_id = task.id;
    event_loop.dispatch(AppMessage::CreateTask(task)).unwrap();
    assert_eq!(event_loop.state().tasks[0].status, TaskStatus::ToDo);

    // Start drag
    event_loop
        .dispatch(AppMessage::DragStart(DragItem::TaskCard {
            id: task_id,
            article_id,
            origin_status: TaskStatus::ToDo,
        }))
        .unwrap();

    // Hover Complete
    event_loop
        .dispatch(AppMessage::DragHover(Some(DropTarget::TaskColumn(
            TaskStatus::Complete,
        ))))
        .unwrap();
    assert!(event_loop.state().drag.is_valid_drop());

    // Drop
    event_loop.dispatch(AppMessage::DragDrop).unwrap();
    assert!(!event_loop.state().drag.is_dragging());
    assert_eq!(event_loop.state().tasks[0].status, TaskStatus::Complete);

    let from_storage = event_loop
        .state()
        .storage
        .get_task(task_id)
        .unwrap()
        .unwrap();
    assert_eq!(from_storage.status, TaskStatus::Complete);
}
