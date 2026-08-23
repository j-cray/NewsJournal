//! Comprehensive tests for the NewsJournal interactive GUI runner and iced integration.

use newsjournal_core::models::{Article, Contact, Task, ThemeMode};
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::navigation::NavTab;
use newsjournal_gui::platform::runner::NewsJournalApp;
use newsjournal_gui::state::modal::ModalState;
use newsjournal_gui::state::AppState;
use newsjournal_gui::theme::ResolvedTheme;

fn send(app: &mut NewsJournalApp, msg: AppMessage) {
    let _ = app.update(msg);
}

#[test]
fn test_runner_initialization_and_theme() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut app = NewsJournalApp::new(state);

    assert_eq!(app.state().articles.len(), 0);
    assert_eq!(app.state().tasks.len(), 0);
    assert_eq!(app.state().contacts.len(), 0);
    assert_eq!(app.state().active_tab, NavTab::ArticlesKanban);

    // Initial theme should match resolved dark/light
    let _theme = app.theme();

    // Toggle theme
    send(&mut app, AppMessage::SetThemeMode(ThemeMode::Light));
    assert_eq!(app.state().resolved_theme(), ResolvedTheme::Light);

    send(&mut app, AppMessage::SetThemeMode(ThemeMode::Dark));
    assert_eq!(app.state().resolved_theme(), ResolvedTheme::Dark);
}

#[test]
fn test_runner_views_across_all_navigation_tabs() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut app = NewsJournalApp::new(state);

    // Populate state with sample data
    let article = Article::new("city-transit", "City Council Transit Plan");
    let article_id = article.id;
    send(&mut app, AppMessage::CreateArticle(article));

    let task = Task::new(article_id, "Interview transit director");
    send(&mut app, AppMessage::CreateTask(task));

    let contact = Contact::new("Jane Doe").with_organization("Transit Dept");
    send(&mut app, AppMessage::CreateContact(contact));

    // 1. Articles Kanban Tab View
    send(&mut app, AppMessage::NavigateTo(NavTab::ArticlesKanban));
    assert_eq!(app.state().active_tab, NavTab::ArticlesKanban);
    {
        let _view = app.view();
    }

    // 2. Tasks Kanban Tab View
    send(&mut app, AppMessage::NavigateTo(NavTab::TasksKanban));
    assert_eq!(app.state().active_tab, NavTab::TasksKanban);
    {
        let _view = app.view();
    }

    // 3. Contacts Directory Tab View
    send(&mut app, AppMessage::NavigateTo(NavTab::ContactsDirectory));
    assert_eq!(app.state().active_tab, NavTab::ContactsDirectory);
    {
        let _view = app.view();
    }

    // 4. Settings Tab View
    send(&mut app, AppMessage::NavigateTo(NavTab::Settings));
    assert_eq!(app.state().active_tab, NavTab::Settings);
    {
        let _view = app.view();
    }
}

#[test]
fn test_runner_modal_views() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut app = NewsJournalApp::new(state);

    // 1. Article Draft Modal
    send(&mut app, AppMessage::OpenNewArticleModal);
    assert!(app.state().modal.is_open());
    {
        let _view = app.view();
    }

    send(
        &mut app,
        AppMessage::UpdateArticleSlug("investigative-report".into()),
    );
    send(
        &mut app,
        AppMessage::UpdateArticleHeadline("Investigative Story".into()),
    );
    send(&mut app, AppMessage::SubmitModal);
    assert_eq!(app.state().articles.len(), 1);
    assert!(!app.state().modal.is_open());

    // 2. Task Draft Modal
    let article_id = app.state().articles[0].id;
    send(&mut app, AppMessage::OpenNewTaskModal(Some(article_id)));
    assert!(app.state().modal.is_open());
    {
        let _view = app.view();
    }

    send(
        &mut app,
        AppMessage::UpdateTaskDraftTitle("Gather public documents".into()),
    );
    send(&mut app, AppMessage::SubmitModal);
    assert_eq!(app.state().tasks.len(), 1);
    assert!(!app.state().modal.is_open());

    // 3. Contact Draft Modal
    send(&mut app, AppMessage::OpenNewContactModal);
    assert!(app.state().modal.is_open());
    {
        let _view = app.view();
    }

    send(
        &mut app,
        AppMessage::UpdateContactDraftName("John Public".into()),
    );
    send(
        &mut app,
        AppMessage::UpdateContactDraftOrg("Public Records Office".into()),
    );
    send(&mut app, AppMessage::SubmitModal);
    assert_eq!(app.state().contacts.len(), 1);
    assert!(!app.state().modal.is_open());

    // 4. Confirm Delete Modals
    send(&mut app, AppMessage::PromptDeleteArticle(article_id));
    assert!(matches!(
        app.state().modal,
        ModalState::ConfirmDeleteArticle { .. }
    ));
    {
        let _view = app.view();
    }

    send(&mut app, AppMessage::CloseModal);
    assert!(!app.state().modal.is_open());
}

#[test]
fn test_runner_subscription_and_ticks() {
    let state = AppState::in_memory().expect("in-memory state");
    let app = NewsJournalApp::new(state);

    let _sub = app.subscription();
}
