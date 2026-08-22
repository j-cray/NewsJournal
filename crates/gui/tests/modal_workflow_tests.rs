//! Modal and drawer workflow integration tests.

use newsjournal_core::models::{Article, Contact, TaskStatus};
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::state::modal::{ArticleDraft, ContactDraft, ModalState, TaskDraft};
use newsjournal_gui::{AppState, EventLoop};

#[test]
fn test_article_modal_create_and_validation_workflow() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    // Open New Article Modal
    event_loop
        .dispatch(AppMessage::OpenNewArticleModal)
        .unwrap();
    assert!(event_loop.state().modal.is_open());
    assert_eq!(event_loop.state().modal.title(), "New Article");

    // Submit without filling fields (Validation Failure)
    event_loop.dispatch(AppMessage::SubmitModal).unwrap();
    assert!(event_loop.state().modal.is_open());
    assert_eq!(event_loop.state().articles.len(), 0);

    // Populate invalid slug
    let mut draft = ArticleDraft::new();
    draft.slug = "INVALID SLUG WITH SPACES".to_string();
    draft.headline = "Valid Headline".to_string();
    event_loop
        .dispatch(AppMessage::UpdateArticleDraft(draft))
        .unwrap();
    event_loop.dispatch(AppMessage::SubmitModal).unwrap();
    assert!(event_loop.state().modal.is_open());
    assert_eq!(event_loop.state().articles.len(), 0);

    // Populate valid fields
    let mut valid_draft = ArticleDraft::new();
    valid_draft.slug = "transit-strike".to_string();
    valid_draft.headline = "City Transit Strike Averted at Final Hour".to_string();
    valid_draft.color_hex = "#1A85FF".to_string();
    event_loop
        .dispatch(AppMessage::UpdateArticleDraft(valid_draft))
        .unwrap();

    // Submit valid
    event_loop.dispatch(AppMessage::SubmitModal).unwrap();
    assert!(!event_loop.state().modal.is_open());
    assert_eq!(event_loop.state().articles.len(), 1);
    assert_eq!(event_loop.state().articles[0].slug, "transit-strike");
}

#[test]
fn test_article_modal_edit_and_tagging_workflow() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let article = Article::new("hospital-audit", "Hospital Network Audit");
    let article_id = article.id;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    let contact = Contact::new("Chief Medical Officer");
    let contact_id = contact.id;
    event_loop
        .dispatch(AppMessage::CreateContact(contact))
        .unwrap();

    // Open Edit Article Modal
    event_loop
        .dispatch(AppMessage::OpenEditArticleModal(article_id))
        .unwrap();
    assert!(event_loop.state().modal.is_open());
    assert_eq!(event_loop.state().modal.title(), "Edit Article");

    // Modify headline and tag contact
    if let ModalState::ArticleForm(mut draft) = event_loop.state().modal.clone() {
        draft.headline = "Hospital Network Financial Audit Complete".to_string();
        draft.tagged_contact_ids.push(contact_id);
        event_loop
            .dispatch(AppMessage::UpdateArticleDraft(draft))
            .unwrap();
    }

    // Submit
    event_loop.dispatch(AppMessage::SubmitModal).unwrap();
    assert!(!event_loop.state().modal.is_open());
    assert_eq!(
        event_loop.state().articles[0].headline,
        "Hospital Network Financial Audit Complete"
    );
    assert_eq!(event_loop.state().contacts_for_article(article_id).len(), 1);
}

#[test]
fn test_task_modal_create_and_edit_workflow() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let article = Article::new("energy-grid", "State Energy Grid Reliability");
    let article_id = article.id;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    // Open New Task Modal
    event_loop
        .dispatch(AppMessage::OpenNewTaskModal(Some(article_id)))
        .unwrap();
    assert!(event_loop.state().modal.is_open());
    assert_eq!(event_loop.state().modal.title(), "New Task");

    // Fill valid task draft
    let mut draft = TaskDraft::new_for_article(Some(article_id));
    draft.title = "Review Grid Reliability Report".to_string();
    draft.notes = "Look at peak summer demand surge numbers".to_string();
    event_loop
        .dispatch(AppMessage::UpdateTaskDraft(draft))
        .unwrap();

    // Submit
    event_loop.dispatch(AppMessage::SubmitModal).unwrap();
    assert!(!event_loop.state().modal.is_open());
    assert_eq!(event_loop.state().tasks.len(), 1);
    assert_eq!(
        event_loop.state().tasks[0].title,
        "Review Grid Reliability Report"
    );

    // Open Edit Task Modal
    let task_id = event_loop.state().tasks[0].id;
    event_loop
        .dispatch(AppMessage::OpenEditTaskModal(task_id))
        .unwrap();
    assert_eq!(event_loop.state().modal.title(), "Edit Task");

    if let ModalState::TaskForm(mut edit_draft) = event_loop.state().modal.clone() {
        edit_draft.status = TaskStatus::Complete;
        event_loop
            .dispatch(AppMessage::UpdateTaskDraft(edit_draft))
            .unwrap();
    }

    event_loop.dispatch(AppMessage::SubmitModal).unwrap();
    assert!(!event_loop.state().modal.is_open());
    assert_eq!(event_loop.state().tasks[0].status, TaskStatus::Complete);
}

#[test]
fn test_contact_modal_workflow() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    // Open New Contact
    event_loop
        .dispatch(AppMessage::OpenNewContactModal)
        .unwrap();
    assert_eq!(event_loop.state().modal.title(), "New Contact");

    let mut draft = ContactDraft::new();
    draft.name = "Sarah Connor".to_string();
    draft.organization = "Tech Defense".to_string();
    draft.email = "sarah@techdefense.org".to_string();
    draft.phone = "+1 555 987 6543".to_string();
    event_loop
        .dispatch(AppMessage::UpdateContactDraft(draft))
        .unwrap();

    event_loop.dispatch(AppMessage::SubmitModal).unwrap();
    assert!(!event_loop.state().modal.is_open());
    assert_eq!(event_loop.state().contacts.len(), 1);
    assert_eq!(event_loop.state().contacts[0].name, "Sarah Connor");

    // Edit Contact
    let contact_id = event_loop.state().contacts[0].id;
    event_loop
        .dispatch(AppMessage::OpenEditContactModal(contact_id))
        .unwrap();
    assert_eq!(event_loop.state().modal.title(), "Edit Contact");

    if let ModalState::ContactForm(mut edit_draft) = event_loop.state().modal.clone() {
        edit_draft.role = "Lead Security Analyst".to_string();
        event_loop
            .dispatch(AppMessage::UpdateContactDraft(edit_draft))
            .unwrap();
    }

    event_loop.dispatch(AppMessage::SubmitModal).unwrap();
    assert!(!event_loop.state().modal.is_open());
    assert_eq!(
        event_loop.state().contacts[0].role.as_deref(),
        Some("Lead Security Analyst")
    );
}

#[test]
fn test_confirm_delete_dialogs_and_cancellation() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let article = Article::new("obsolete-story", "Obsolete Draft Story");
    let article_id = article.id;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();
    assert_eq!(event_loop.state().articles.len(), 1);

    // Prompt Delete
    event_loop
        .dispatch(AppMessage::PromptDeleteArticle(article_id))
        .unwrap();
    assert!(event_loop.state().modal.is_open());
    assert_eq!(event_loop.state().modal.title(), "Delete Article?");

    // Cancel modal
    event_loop.dispatch(AppMessage::CloseModal).unwrap();
    assert!(!event_loop.state().modal.is_open());
    assert_eq!(event_loop.state().articles.len(), 1);

    // Prompt Delete and Confirm
    event_loop
        .dispatch(AppMessage::PromptDeleteArticle(article_id))
        .unwrap();
    event_loop.dispatch(AppMessage::SubmitModal).unwrap();
    assert!(!event_loop.state().modal.is_open());
    assert_eq!(event_loop.state().articles.len(), 0);
}
