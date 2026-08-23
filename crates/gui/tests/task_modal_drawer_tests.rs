//! Comprehensive integration test suite for Task Creation & Editing Drawer (Task 7.4).
//!
//! Verifies:
//! 1. "New Task" toolbar button opens drawer with pre-selected parent article.
//! 2. Clicking any task card opens task detail/edit drawer.
//! 3. Parent article dropdown picker correctly lists stories and sets parent.
//! 4. Validation logic for title (required, max length) and parent article (required).
//! 5. Status picker transitions between To-Do, In Progress, and Complete.
//! 6. Quick due date presets and clear action.
//! 7. Immediate synchronization reflecting on Article card task counters.
//! 8. Deleting task via drawer prompts confirmation dialog and updates article task counters.
//! 9. Keyboard shortcuts (Ctrl+S / Cmd+S save, Esc cancel).
//! 10. Platform view tree integration in COSMIC and macOS UI containers.

use chrono::{Duration, Utc};
use newsjournal_core::models::{Article, Task, TaskStatus};
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::navigation::NavKeyModifiers;
use newsjournal_gui::platform::cosmic::CosmicApp;
use newsjournal_gui::platform::macos::MacosApp;
use newsjournal_gui::state::modal::ModalState;
use newsjournal_gui::state::AppState;
use newsjournal_gui::views::article_form::DeadlinePreset;
use newsjournal_gui::views::{
    build_articles_kanban_deck, build_modal_container_view, build_task_form_view,
    build_tasks_kanban_deck,
};

#[test]
fn test_new_task_toolbar_button_opens_task_drawer() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let article = Article::new("city-transit", "City Transit Investigation");
    let aid = article.id;
    state.articles.push(article);

    assert!(!state.modal.is_open());

    // Click "+ New Task" with specific article filter
    state.update(AppMessage::OpenNewTaskModal(Some(aid)));
    assert!(state.modal.is_open());

    match &state.modal {
        ModalState::TaskForm(draft) => {
            assert!(draft.id.is_none());
            assert_eq!(draft.article_id, Some(aid));
            assert_eq!(draft.status, TaskStatus::ToDo);
            assert!(draft.title.is_empty());
        }
        other => panic!("Expected ModalState::TaskForm, got {other:?}"),
    }

    let modal_container = build_modal_container_view(&state);
    assert!(modal_container.is_open);
    assert_eq!(modal_container.header.title, "New Task");
    assert!(modal_container.task_form.is_some());

    let task_form = modal_container.task_form.unwrap();
    assert!(!task_form.is_edit);
    assert_eq!(task_form.save_button_label, "Create Task");
    assert_eq!(task_form.article_picker.selected_article_id, Some(aid));
    assert!(!task_form.can_save); // Title is empty
}

#[test]
fn test_click_task_card_opens_task_edit_drawer() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let article = Article::new("port-security", "Port Security Vulnerabilities");
    let aid = article.id;
    state.articles.push(article);

    let due = Utc::now() + Duration::days(2);
    let task = Task::new(aid, "Review Customs manifest logs")
        .with_status(TaskStatus::InProgress)
        .with_notes("Focus on terminal 4 import records")
        .with_due_date(due);
    let tid = task.id;
    state.tasks.push(task);

    // Click on the task card to open edit drawer
    let deck = build_tasks_kanban_deck(&state);
    let (_, card) = deck.card(tid).expect("task card found in deck");
    assert_eq!(card.click_action, AppMessage::OpenEditTaskModal(tid));

    state.update(card.click_action.clone());
    assert!(state.modal.is_open());

    match &state.modal {
        ModalState::TaskForm(draft) => {
            assert_eq!(draft.id, Some(tid));
            assert_eq!(draft.article_id, Some(aid));
            assert_eq!(draft.title, "Review Customs manifest logs");
            assert_eq!(draft.status, TaskStatus::InProgress);
            assert_eq!(draft.due_date, Some(due));
            assert_eq!(draft.notes, "Focus on terminal 4 import records");
        }
        other => panic!("Expected ModalState::TaskForm, got {other:?}"),
    }

    let container = build_modal_container_view(&state);
    assert!(container.is_open);
    assert_eq!(container.header.title, "Edit Task");
    let form = container.task_form.expect("task_form view model");
    assert!(form.is_edit);
    assert_eq!(form.save_button_label, "Save Changes");
    assert_eq!(form.title_field.value, "Review Customs manifest logs");
    assert_eq!(form.status_picker.current_status, TaskStatus::InProgress);
    assert!(form.can_save);
    assert!(form.header.can_delete);
}

#[test]
fn test_parent_article_dropdown_selector_switching() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let a1 = Article::new("election-finance", "Campaign Finance Audit");
    let a2 = Article::new("housing-crisis", "Affordable Housing Report");
    let a1_id = a1.id;
    let a2_id = a2.id;
    state.articles.push(a1);
    state.articles.push(a2);

    state.update(AppMessage::OpenNewTaskModal(Some(a1_id)));

    if let ModalState::TaskForm(ref draft) = state.modal {
        let form_view = build_task_form_view(&state, draft);
        assert_eq!(form_view.article_picker.options.len(), 2);
        assert_eq!(form_view.article_picker.selected_article_id, Some(a1_id));
        let selected = form_view.article_picker.selected_option.unwrap();
        assert_eq!(selected.slug, "election-finance");
        assert_eq!(selected.display_slug, "#election-finance");
    } else {
        panic!("Expected TaskForm");
    }

    // Switch parent article to Housing Crisis
    state.update(AppMessage::UpdateTaskDraftArticle(Some(a2_id)));

    if let ModalState::TaskForm(ref draft) = state.modal {
        assert_eq!(draft.article_id, Some(a2_id));
        let form_view = build_task_form_view(&state, draft);
        assert_eq!(form_view.article_picker.selected_article_id, Some(a2_id));
        let selected = form_view.article_picker.selected_option.unwrap();
        assert_eq!(selected.slug, "housing-crisis");
        assert_eq!(selected.display_slug, "#housing-crisis");
    } else {
        panic!("Expected TaskForm");
    }
}

#[test]
fn test_task_validation_errors_and_prevention() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let article = Article::new("water-safety", "Drinking Water Safety Investigation");
    let aid = article.id;
    state.articles.push(article);

    state.update(AppMessage::OpenNewTaskModal(None));
    // Explicitly clear article ID to test missing parent validation
    state.update(AppMessage::UpdateTaskDraftArticle(None));

    // 1. Title empty & no article
    if let ModalState::TaskForm(ref draft) = state.modal {
        let form_view = build_task_form_view(&state, draft);
        assert!(!form_view.can_save);
        assert!(!form_view.article_picker.is_valid);
    }

    // Attempt to submit invalid form -> warning toast, modal stays open
    let commands = state.update(AppMessage::SubmitModal);
    assert!(state.modal.is_open());
    assert!(!commands.is_empty());

    // 2. Set valid title, article still None
    state.update(AppMessage::UpdateTaskDraftTitle(
        "Collect water samples at reservoir".to_string(),
    ));
    if let ModalState::TaskForm(ref draft) = state.modal {
        let form_view = build_task_form_view(&state, draft);
        assert!(!form_view.can_save);
        assert!(!form_view.article_picker.is_valid);
    }

    // 3. Set valid parent article -> form now valid
    state.update(AppMessage::UpdateTaskDraftArticle(Some(aid)));
    if let ModalState::TaskForm(ref draft) = state.modal {
        let form_view = build_task_form_view(&state, draft);
        assert!(form_view.can_save);
        assert!(form_view.title_field.is_valid);
        assert!(form_view.article_picker.is_valid);
    }
}

#[test]
fn test_status_picker_and_due_date_presets() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let article = Article::new("hospital-audit", "Hospital Emergency Room Wait Times");
    let aid = article.id;
    state.articles.push(article);

    state.update(AppMessage::OpenNewTaskModal(Some(aid)));
    state.update(AppMessage::UpdateTaskDraftTitle(
        "Interview ER chief of medicine".to_string(),
    ));

    // Change status to In Progress
    state.update(AppMessage::SetTaskDraftStatus(TaskStatus::InProgress));
    if let ModalState::TaskForm(ref draft) = state.modal {
        assert_eq!(draft.status, TaskStatus::InProgress);
    }

    // Apply Tomorrow 5 PM due date preset
    state.update(AppMessage::SetTaskDraftDueDatePreset(
        DeadlinePreset::Tomorrow5PM,
    ));
    if let ModalState::TaskForm(ref draft) = state.modal {
        assert!(draft.due_date.is_some());
        let form_view = build_task_form_view(&state, draft);
        assert!(form_view.due_date_field.has_due_date);
        assert!(!form_view.due_date_field.formatted_date.is_empty());
    }

    // Clear due date
    state.update(AppMessage::ClearTaskDraftDueDate);
    if let ModalState::TaskForm(ref draft) = state.modal {
        assert!(draft.due_date.is_none());
        let form_view = build_task_form_view(&state, draft);
        assert!(!form_view.due_date_field.has_due_date);
        assert_eq!(form_view.due_date_field.badge_label, "NO DUE DATE");
    }
}

#[test]
fn test_task_creation_and_article_card_counter_synchronization() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let article = Article::new("transit-strike", "Transit Workers Union Contract Talks");
    let aid = article.id;
    state.articles.push(article);

    // Verify initial Article card task counter is "No tasks" (0/0)
    let articles_deck_initial = build_articles_kanban_deck(&state);
    let (_, card_initial) = articles_deck_initial.card(aid).expect("article card found");
    assert_eq!(card_initial.task_counter.formatted_label, "No tasks");
    assert_eq!(card_initial.task_counter.total, 0);
    assert_eq!(card_initial.task_counter.completed, 0);

    // 1. Create First Task (To-Do)
    state.update(AppMessage::OpenNewTaskModal(Some(aid)));
    state.update(AppMessage::UpdateTaskDraftTitle(
        "Request union contract draft".to_string(),
    ));
    state.update(AppMessage::SetTaskDraftStatus(TaskStatus::ToDo));
    let save_cmds = state.update(AppMessage::SubmitModal);
    assert!(!state.modal.is_open());
    assert!(!save_cmds.is_empty());
    assert_eq!(state.tasks.len(), 1);

    // Verify Article card task counter immediately shows "0/1 tasks"
    let articles_deck_after_1 = build_articles_kanban_deck(&state);
    let (_, card_1) = articles_deck_after_1.card(aid).expect("article card");
    assert_eq!(card_1.task_counter.formatted_label, "0/1 tasks");
    assert_eq!(card_1.task_counter.total, 1);
    assert_eq!(card_1.task_counter.completed, 0);
    assert!(!card_1.task_counter.all_completed);

    // 2. Create Second Task (Complete)
    state.update(AppMessage::OpenNewTaskInStatusModal(
        TaskStatus::Complete,
        Some(aid),
    ));
    state.update(AppMessage::UpdateTaskDraftTitle(
        "Review background history of negotiations".to_string(),
    ));
    state.update(AppMessage::SubmitModal);
    assert_eq!(state.tasks.len(), 2);

    // Verify Article card task counter immediately shows "1/2 tasks" (50% progress)
    let articles_deck_after_2 = build_articles_kanban_deck(&state);
    let (_, card_2) = articles_deck_after_2.card(aid).expect("article card");
    assert_eq!(card_2.task_counter.formatted_label, "1/2 tasks");
    assert_eq!(card_2.task_counter.total, 2);
    assert_eq!(card_2.task_counter.completed, 1);
    assert_eq!(card_2.task_counter.progress_percent, 50);

    // 3. Edit First Task -> Mark Complete via Drawer
    let first_task_id = state.tasks[0].id;
    state.update(AppMessage::OpenEditTaskModal(first_task_id));
    state.update(AppMessage::SetTaskDraftStatus(TaskStatus::Complete));
    state.update(AppMessage::SubmitModal);

    // Verify Article card task counter now shows "2/2 complete" (100% progress)
    let articles_deck_after_complete = build_articles_kanban_deck(&state);
    let (_, card_complete) = articles_deck_after_complete
        .card(aid)
        .expect("article card");
    assert_eq!(card_complete.task_counter.formatted_label, "2/2 complete");
    assert!(card_complete.task_counter.all_completed);
    assert_eq!(card_complete.task_counter.progress_percent, 100);
}

#[test]
fn test_task_deletion_from_drawer_updates_article_counter() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let article = Article::new("bridge-inspection", "Bridge Inspection Safety Review");
    let aid = article.id;
    state.articles.push(article);

    let t1 = Task::new(aid, "Collect sensor telemetry data");
    let t2 = Task::new(aid, "Interview chief civil engineer").with_status(TaskStatus::Complete);
    let t1_id = t1.id;
    state.tasks.push(t1);
    state.tasks.push(t2);

    // Initially 1/2 complete
    let (completed, total) = state.task_completion_stats(aid);
    assert_eq!((completed, total), (1, 2));

    // Open Edit Drawer for Task 1 and click Delete
    state.update(AppMessage::OpenEditTaskModal(t1_id));
    let form = if let ModalState::TaskForm(ref draft) = state.modal {
        build_task_form_view(&state, draft)
    } else {
        panic!("Expected TaskForm");
    };

    assert!(form.header.can_delete);
    let delete_action = form.header.delete_action.expect("delete action");
    assert_eq!(delete_action, AppMessage::PromptDeleteTask(t1_id));

    // Dispatch delete prompt -> confirmation dialog
    state.update(delete_action);
    assert_eq!(
        state.modal,
        ModalState::ConfirmDeleteTask {
            id: t1_id,
            title: "Collect sensor telemetry data".to_string(),
        }
    );

    // Confirm deletion
    state.update(AppMessage::SubmitModal);
    assert!(!state.modal.is_open());
    assert_eq!(state.tasks.len(), 1);

    // Now 1/1 complete (100%)
    let (completed_after, total_after) = state.task_completion_stats(aid);
    assert_eq!((completed_after, total_after), (1, 1));

    let deck = build_articles_kanban_deck(&state);
    let (_, card) = deck.card(aid).expect("article card");
    assert_eq!(card.task_counter.formatted_label, "1/1 complete");
    assert!(card.task_counter.all_completed);
}

#[test]
fn test_cosmic_and_macos_app_task_drawer_view_tree_integration() {
    // 1. Linux COSMIC App
    let state_linux = AppState::in_memory().expect("in-memory state");
    let mut cosmic_app = CosmicApp::new(state_linux);

    let article = Article::new("port-ops", "Port Operations Audit");
    let aid = article.id;
    cosmic_app
        .dispatch(AppMessage::CreateArticle(article))
        .expect("create article");

    // Open Task Modal
    cosmic_app
        .dispatch(AppMessage::OpenNewTaskModal(Some(aid)))
        .expect("open task modal");

    let tree_linux = cosmic_app.build_view_tree();
    assert!(tree_linux.modal_container.is_some());
    let container_linux = tree_linux.modal_container.unwrap();
    assert!(container_linux.is_open);
    assert_eq!(container_linux.header.title, "New Task");
    assert!(container_linux.task_form.is_some());
    let form_linux = container_linux.task_form.unwrap();
    assert_eq!(form_linux.save_shortcut, "Ctrl+S");

    // Keyboard shortcut to close (Esc)
    let handled_esc = cosmic_app
        .handle_key_event("Escape", NavKeyModifiers::none())
        .expect("handle escape");
    assert!(handled_esc);
    assert!(!cosmic_app.state().modal.is_open());

    // 2. macOS App
    let state_mac = AppState::in_memory().expect("in-memory state");
    let mut mac_app = MacosApp::new(state_mac);

    let art_mac = Article::new("wildfire-prep", "Statewide Wildfire Preparedness");
    let mac_aid = art_mac.id;
    mac_app
        .dispatch(AppMessage::CreateArticle(art_mac))
        .expect("create article");

    mac_app
        .dispatch(AppMessage::OpenNewTaskModal(Some(mac_aid)))
        .expect("open task modal");

    let tree_mac = mac_app.build_view_tree();
    assert!(tree_mac.modal_container.is_some());
    let container_mac = tree_mac.modal_container.unwrap();
    assert!(container_mac.is_open);
    assert!(container_mac.task_form.is_some());
    let form_mac = container_mac.task_form.unwrap();
    assert_eq!(form_mac.save_shortcut, "⌘S");

    // Save shortcut on macOS (Cmd+S)
    mac_app
        .dispatch(AppMessage::UpdateTaskDraftTitle(
            "Review CAL FIRE equipment dispatch records".to_string(),
        ))
        .expect("update title");

    let handled_save = mac_app
        .handle_key_event(
            "s",
            NavKeyModifiers {
                meta: true,
                ..Default::default()
            },
        )
        .expect("handle cmd+s");
    assert!(handled_save);
    assert!(!mac_app.state().modal.is_open());
    assert_eq!(mac_app.state().tasks.len(), 1);
    assert_eq!(
        mac_app.state().tasks[0].title,
        "Review CAL FIRE equipment dispatch records"
    );
}
