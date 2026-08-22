//! View models presentation tests.

use chrono::{Duration, Utc};
use newsjournal_core::models::{Article, Contact, Task, TaskStatus};
use newsjournal_core::ArticleStage;
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::state::toast::ToastMessage;
use newsjournal_gui::views::{
    build_articles_kanban_view, build_contacts_view, build_modal_view, build_nav_view_models,
    build_settings_view, build_tasks_kanban_view, build_toast_view,
};
use newsjournal_gui::{AppState, EventLoop, NavTab};

#[test]
fn test_view_models_generation() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let now = Utc::now();

    // Create article with deadline and color
    let mut article = Article::new("subway-expansion", "Subway Line 4 Extension Approved");
    article.stage = ArticleStage::Writing;
    article.deadline = Some(now - Duration::hours(1)); // Overdue
    article.color = Some("#FF5733".to_string());
    let article_id = article.id;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    // Create 2 tasks for this article
    let t1 = Task::new(article_id, "Interview transit director");
    let mut t2 = Task::new(article_id, "Draw subway route map");
    t2.status = TaskStatus::Complete;
    event_loop.dispatch(AppMessage::CreateTask(t1)).unwrap();
    event_loop.dispatch(AppMessage::CreateTask(t2)).unwrap();

    // Create contact and tag
    let contact = Contact::new("Director Martinez")
        .with_phone("5551234567")
        .with_organization("MTA");
    let contact_id = contact.id;
    event_loop
        .dispatch(AppMessage::CreateContact(contact))
        .unwrap();
    event_loop
        .dispatch(AppMessage::LinkContactToArticle {
            article_id,
            contact_id,
        })
        .unwrap();

    // Add toast
    event_loop
        .dispatch(AppMessage::PushToast(ToastMessage::info("T", "B")))
        .unwrap();

    let state = event_loop.state();

    // 1. Nav View Model
    let nav_items = build_nav_view_models(state);
    assert_eq!(nav_items.len(), 4);
    assert!(nav_items[0].is_active);
    assert_eq!(nav_items[0].tab, NavTab::ArticlesKanban);

    // 2. Articles Kanban View Model
    let article_columns = build_articles_kanban_view(state);
    assert_eq!(article_columns.len(), 6);
    let writing_col = article_columns
        .iter()
        .find(|c| c.stage == ArticleStage::Writing)
        .unwrap();
    assert_eq!(writing_col.cards.len(), 1);
    let card = &writing_col.cards[0];
    assert_eq!(card.slug, "subway-expansion");
    assert_eq!(card.color_hex, "#FF5733");
    assert!(card.is_overdue);
    assert_eq!(card.task_completed, 1);
    assert_eq!(card.task_total, 2);
    assert_eq!(card.tagged_contact_count, 1);

    // 3. Tasks Kanban View Model
    let task_columns = build_tasks_kanban_view(state);
    assert_eq!(task_columns.len(), 3);
    let todo_col = task_columns
        .iter()
        .find(|c| c.status == TaskStatus::ToDo)
        .unwrap();
    assert_eq!(todo_col.cards.len(), 1);
    assert_eq!(todo_col.cards[0].parent_article_slug, "subway-expansion");
    assert_eq!(todo_col.cards[0].parent_article_color, "#FF5733");

    let complete_col = task_columns
        .iter()
        .find(|c| c.status == TaskStatus::Complete)
        .unwrap();
    assert_eq!(complete_col.cards.len(), 1);

    // 4. Contacts View Model
    let contacts_view = build_contacts_view(state);
    assert_eq!(contacts_view.len(), 1);
    assert_eq!(contacts_view[0].name, "Director Martinez");
    assert_eq!(contacts_view[0].phone_display, "(555) 123-4567");
    assert_eq!(contacts_view[0].linked_articles_count, 1);
    assert_eq!(
        contacts_view[0].linked_article_slugs,
        vec!["subway-expansion"]
    );

    // 5. Settings View Model
    let settings_view = build_settings_view(state);
    assert_eq!(settings_view.total_articles, 1);
    assert_eq!(settings_view.total_tasks, 2);
    assert_eq!(settings_view.total_contacts, 1);

    // 6. Modal View Model
    let modal_view = build_modal_view(state);
    assert!(!modal_view.is_open);

    // 7. Toast View Model
    let toast_view = build_toast_view(state);
    assert_eq!(toast_view.toasts.len(), 2);
    assert_eq!(toast_view.toasts[0].title, "Article Created");
    assert_eq!(toast_view.toasts[1].title, "T");
}
