//! Comprehensive state machine integration tests for navigation, CRUD, and filters.

use newsjournal_core::models::{Article, Contact, Settings, Task, TaskStatus, ThemeMode};
use newsjournal_core::ArticleStage;
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::{AppState, EventLoop, NavTab};

#[test]
fn test_navigation_transitions() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    assert_eq!(event_loop.state().active_tab, NavTab::ArticlesKanban);

    event_loop
        .dispatch(AppMessage::NavigateTo(NavTab::TasksKanban))
        .unwrap();
    assert_eq!(event_loop.state().active_tab, NavTab::TasksKanban);

    event_loop
        .dispatch(AppMessage::NavigateTo(NavTab::ContactsDirectory))
        .unwrap();
    assert_eq!(event_loop.state().active_tab, NavTab::ContactsDirectory);

    event_loop
        .dispatch(AppMessage::NavigateTo(NavTab::Settings))
        .unwrap();
    assert_eq!(event_loop.state().active_tab, NavTab::Settings);

    event_loop
        .dispatch(AppMessage::NavigateTo(NavTab::ArticlesKanban))
        .unwrap();
    assert_eq!(event_loop.state().active_tab, NavTab::ArticlesKanban);
}

#[test]
fn test_article_crud_flow() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let article = Article::new("port-scandal", "Port Authority Audit Exposes Misconduct");
    let article_id = article.id;

    // Create
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();
    assert_eq!(event_loop.state().articles.len(), 1);
    assert_eq!(event_loop.state().articles[0].slug, "port-scandal");

    // Move Stage
    event_loop
        .dispatch(AppMessage::MoveArticleStage(
            article_id,
            ArticleStage::Researching,
        ))
        .unwrap();
    assert_eq!(
        event_loop.state().articles[0].stage,
        ArticleStage::Researching
    );
    assert_eq!(
        event_loop
            .state()
            .articles_in_stage(ArticleStage::Researching)
            .len(),
        1
    );

    // Update
    let mut updated = event_loop.state().articles[0].clone();
    updated.headline = "Updated: Port Authority Audit Exposes Misconduct".to_string();
    event_loop
        .dispatch(AppMessage::UpdateArticle(updated))
        .unwrap();
    assert_eq!(
        event_loop.state().articles[0].headline,
        "Updated: Port Authority Audit Exposes Misconduct"
    );

    // Delete
    event_loop
        .dispatch(AppMessage::DeleteArticle(article_id))
        .unwrap();
    assert_eq!(event_loop.state().articles.len(), 0);
}

#[test]
fn test_task_crud_and_status_toggling() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let article = Article::new("wildfire-season", "Early Wildfire Season Threatens County");
    let article_id = article.id;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    let task = Task::new(article_id, "Interview Fire Chief");
    let task_id = task.id;

    // Create Task
    event_loop.dispatch(AppMessage::CreateTask(task)).unwrap();
    assert_eq!(event_loop.state().tasks.len(), 1);
    assert_eq!(event_loop.state().tasks[0].status, TaskStatus::ToDo);

    // Toggle Status: ToDo -> InProgress
    event_loop
        .dispatch(AppMessage::ToggleTaskStatus(task_id))
        .unwrap();
    assert_eq!(event_loop.state().tasks[0].status, TaskStatus::InProgress);

    // Toggle Status: InProgress -> Complete
    event_loop
        .dispatch(AppMessage::ToggleTaskStatus(task_id))
        .unwrap();
    assert_eq!(event_loop.state().tasks[0].status, TaskStatus::Complete);

    // Toggle Status: Complete -> ToDo
    event_loop
        .dispatch(AppMessage::ToggleTaskStatus(task_id))
        .unwrap();
    assert_eq!(event_loop.state().tasks[0].status, TaskStatus::ToDo);

    // Explicit Move
    event_loop
        .dispatch(AppMessage::MoveTaskStatus(task_id, TaskStatus::Complete))
        .unwrap();
    assert_eq!(event_loop.state().tasks[0].status, TaskStatus::Complete);

    // Check completion stats on article
    let (completed, total) = event_loop.state().task_completion_stats(article_id);
    assert_eq!((completed, total), (1, 1));

    // Delete Task
    event_loop
        .dispatch(AppMessage::DeleteTask(task_id))
        .unwrap();
    assert_eq!(event_loop.state().tasks.len(), 0);
}

#[test]
fn test_contact_crud_and_tagging_relationships() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let article = Article::new("budget-investigation", "City Budget Shortfall Analysis");
    let article_id = article.id;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    let contact = Contact::new("Auditor General")
        .with_email("auditor@city.gov")
        .with_phone("+1 555 123 4567");
    let contact_id = contact.id;

    // Create Contact
    event_loop
        .dispatch(AppMessage::CreateContact(contact))
        .unwrap();
    assert_eq!(event_loop.state().contacts.len(), 1);

    // Link Contact to Article
    event_loop
        .dispatch(AppMessage::LinkContactToArticle {
            article_id,
            contact_id,
        })
        .unwrap();
    assert_eq!(event_loop.state().contacts_for_article(article_id).len(), 1);
    assert_eq!(event_loop.state().articles_for_contact(contact_id).len(), 1);

    // Unlink Contact
    event_loop
        .dispatch(AppMessage::UnlinkContactFromArticle {
            article_id,
            contact_id,
        })
        .unwrap();
    assert_eq!(event_loop.state().contacts_for_article(article_id).len(), 0);
    assert_eq!(event_loop.state().articles_for_contact(contact_id).len(), 0);

    // Delete Contact
    event_loop
        .dispatch(AppMessage::DeleteContact(contact_id))
        .unwrap();
    assert_eq!(event_loop.state().contacts.len(), 0);
}

#[test]
fn test_settings_and_theme_updates() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    assert_eq!(event_loop.state().settings.theme_mode, ThemeMode::System);

    event_loop
        .dispatch(AppMessage::SetThemeMode(ThemeMode::Dark))
        .unwrap();
    assert_eq!(event_loop.state().settings.theme_mode, ThemeMode::Dark);

    let custom_settings = Settings::new(ThemeMode::Light);
    event_loop
        .dispatch(AppMessage::SaveSettings(custom_settings))
        .unwrap();
    assert_eq!(event_loop.state().settings.theme_mode, ThemeMode::Light);
}

#[test]
fn test_search_and_filter_state_machine() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let a1 = Article::new("water-crisis", "Lead Detected in Downtown Drinking Water");
    let a2 = Article::new("tech-jobs", "Tech Sector Hiring Rebounds in Quarter 3")
        .with_stage(ArticleStage::Published);

    event_loop.dispatch(AppMessage::CreateArticle(a1)).unwrap();
    event_loop.dispatch(AppMessage::CreateArticle(a2)).unwrap();

    let c1 = Contact::new("Dr. Alice Waters").with_organization("Water Dept");
    let c2 = Contact::new("Bob Builder").with_organization("Construction Corp");
    event_loop.dispatch(AppMessage::CreateContact(c1)).unwrap();
    event_loop.dispatch(AppMessage::CreateContact(c2)).unwrap();

    // Search contacts
    event_loop
        .dispatch(AppMessage::SetSearchQuery("alice".to_string()))
        .unwrap();
    assert_eq!(event_loop.state().filtered_contacts().len(), 1);
    assert_eq!(
        event_loop.state().filtered_contacts()[0].name,
        "Dr. Alice Waters"
    );

    // Search articles
    event_loop
        .dispatch(AppMessage::SetSearchQuery("drinking".to_string()))
        .unwrap();
    assert_eq!(event_loop.state().filtered_articles().len(), 1);
    assert_eq!(
        event_loop.state().filtered_articles()[0].slug,
        "water-crisis"
    );

    // Filter stage
    event_loop.dispatch(AppMessage::ClearFilters).unwrap();
    event_loop
        .dispatch(AppMessage::SetStageFilter(Some(ArticleStage::Published)))
        .unwrap();
    assert_eq!(event_loop.state().filtered_articles().len(), 1);
    assert_eq!(event_loop.state().filtered_articles()[0].slug, "tech-jobs");

    // Clear filters
    event_loop.dispatch(AppMessage::ClearFilters).unwrap();
    assert_eq!(event_loop.state().filtered_articles().len(), 2);
    assert_eq!(event_loop.state().filtered_contacts().len(), 2);
}
