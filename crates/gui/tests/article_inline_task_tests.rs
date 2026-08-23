//! Comprehensive integration tests for Task 6.4: Inline Task Management Sub-Section.
//!
//! Tests:
//! 1. Task checklist item view model formatting, status badges, overdue checks, and notes.
//! 2. Tasks sub-section metrics: completion count, total count, percentage calculation, and progress labels.
//! 3. Quick-add task validation (empty, max length limits, whitespace handling).
//! 4. Quick-add inline task to existing article with immediate persistence, status toggling, and deletion.
//! 5. Staged tasks workflow for new article drafts: staging, toggling, removing, and atomic persistence on SubmitModal.
//! 6. Platform view tree integration across COSMIC and macOS.

use chrono::{Duration, Utc};
use newsjournal_core::models::{Article, ArticleStage, Task, TaskStatus};
use newsjournal_core::validation::MAX_TASK_TITLE_LENGTH;
use newsjournal_gui::commands::CommandExecutor;
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::platform::cosmic::CosmicApp;
use newsjournal_gui::platform::macos::MacosApp;
use newsjournal_gui::state::modal::{ArticleDraft, ModalState};
use newsjournal_gui::state::AppState;
use newsjournal_gui::views::{build_article_form_view, TaskChecklistItemViewModel};
use uuid::Uuid;

#[test]
fn test_task_checklist_item_view_model_and_properties() {
    let now = Utc::now();
    let article_id = Uuid::new_v4();

    // 1. To-Do task with no due date
    let mut t1 = Task::new(article_id, "Request public spending logs");
    t1.notes = Some("Contact city records officer by phone".to_string());
    let item1 = TaskChecklistItemViewModel::new(&t1, now, false);

    assert_eq!(item1.id, t1.id);
    assert_eq!(item1.title, "Request public spending logs");
    assert_eq!(
        item1.notes.as_deref(),
        Some("Contact city records officer by phone")
    );
    assert_eq!(item1.status, TaskStatus::ToDo);
    assert!(!item1.is_complete);
    assert_eq!(item1.status_label, "To-Do");
    assert_eq!(item1.status_badge_color_hex, "#64748B");
    assert!(item1.due_date_display.is_none());
    assert!(!item1.is_overdue);
    assert!(!item1.is_staged);

    // 2. In-Progress task with future due date
    let mut t2 = Task::new(article_id, "Interview whistleblower");
    t2.status = TaskStatus::InProgress;
    let future_date = now + Duration::days(3);
    t2.due_date = Some(future_date);
    let item2 = TaskChecklistItemViewModel::new(&t2, now, false);

    assert_eq!(item2.status, TaskStatus::InProgress);
    assert!(!item2.is_complete);
    assert_eq!(item2.status_label, "In Progress");
    assert_eq!(item2.status_badge_color_hex, "#3B82F6");
    assert!(item2.due_date_display.is_some());
    assert!(!item2.is_overdue);

    // 3. Overdue incomplete task
    let mut t3 = Task::new(article_id, "Fact check timeline");
    t3.status = TaskStatus::ToDo;
    let past_date = now - Duration::days(2);
    t3.due_date = Some(past_date);
    let item3 = TaskChecklistItemViewModel::new(&t3, now, false);

    assert!(!item3.is_complete);
    assert!(item3.is_overdue);

    // 4. Completed task with past due date (overdue should be false because task is complete)
    let mut t4 = Task::new(article_id, "Review audio recording");
    t4.status = TaskStatus::Complete;
    t4.due_date = Some(past_date);
    let item4 = TaskChecklistItemViewModel::new(&t4, now, true);

    assert!(item4.is_complete);
    assert_eq!(item4.status_label, "Complete");
    assert_eq!(item4.status_badge_color_hex, "#10B981");
    assert!(!item4.is_overdue);
    assert!(item4.is_staged);
}

#[test]
fn test_article_tasks_section_view_model_empty_and_populated() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let mut article = Article::new("city-hall-audit", "City Hall Audit Underway");
    let article_id = article.id;
    article.stage = ArticleStage::Writing;
    state.articles.push(article.clone());

    // 1. Empty tasks state
    let draft = ArticleDraft::from_article(&article, Vec::new());
    let form_view1 = build_article_form_view(&state, &draft);

    let sec1 = &form_view1.tasks_section;
    assert_eq!(sec1.total_count, 0);
    assert_eq!(sec1.completed_count, 0);
    assert_eq!(sec1.completion_percentage, 0);
    assert_eq!(sec1.progress_label, "No tasks yet");
    assert!(sec1.empty_state_message.is_some());
    assert_eq!(form_view1.task_stats, (0, 0));

    // 2. Add 4 tasks to state: 2 Complete, 1 InProgress, 1 ToDo
    let mut task1 = Task::new(article_id, "Review budget spreadsheet");
    task1.status = TaskStatus::Complete;

    let mut task2 = Task::new(article_id, "Call procurement officer");
    task2.status = TaskStatus::Complete;

    let mut task3 = Task::new(article_id, "Draft section 1 findings");
    task3.status = TaskStatus::InProgress;

    let task4 = Task::new(article_id, "Verify quotes with editor");

    state.tasks = vec![task1, task2, task3, task4];

    let form_view2 = build_article_form_view(&state, &draft);
    let sec2 = &form_view2.tasks_section;
    assert_eq!(sec2.total_count, 4);
    assert_eq!(sec2.completed_count, 2);
    assert_eq!(sec2.completion_percentage, 50);
    assert_eq!(sec2.progress_label, "2 of 4 completed (50%)");
    assert!(sec2.empty_state_message.is_none());
    assert_eq!(form_view2.task_stats, (2, 4));
}

#[test]
fn test_quick_add_task_validation() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut draft = ArticleDraft::new();

    // 1. Initially empty input
    let form1 = build_article_form_view(&state, &draft);
    assert!(!form1.tasks_section.is_quick_add_valid);
    assert!(form1.tasks_section.quick_task_error.is_none());
    assert!(!draft.is_quick_task_valid());

    // 2. Whitespace only input
    draft.set_quick_task_title("   ");
    let form2 = build_article_form_view(&state, &draft);
    assert!(!form2.tasks_section.is_quick_add_valid);
    assert!(!draft.is_quick_task_valid());

    // 3. Exceeding max length
    let long_title = "A".repeat(MAX_TASK_TITLE_LENGTH + 10);
    draft.set_quick_task_title(&long_title);
    let form3 = build_article_form_view(&state, &draft);
    assert!(!form3.tasks_section.is_quick_add_valid);
    assert!(form3.tasks_section.quick_task_error.is_some());
    assert!(!draft.is_quick_task_valid());

    // 4. Valid title
    draft.set_quick_task_title("Confirm interview time with legal team");
    let form4 = build_article_form_view(&state, &draft);
    assert!(form4.tasks_section.is_quick_add_valid);
    assert!(form4.tasks_section.quick_task_error.is_none());
    assert!(draft.is_quick_task_valid());
}

#[test]
fn test_quick_add_inline_task_to_existing_article_workflow() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let article = Article::new("transit-investigation", "Transit Investigation");
    let article_id = article.id;
    state
        .storage
        .create_article(article.clone())
        .expect("create article");
    state.articles.push(article.clone());

    // Open edit modal for the article
    state.update(AppMessage::OpenEditArticleModal(article_id));

    // Type quick task title
    state.update(AppMessage::SetArticleDraftQuickTask(
        "Request GPS transit logs".to_string(),
    ));

    // Submit quick task
    let commands = state.update(AppMessage::AddArticleDraftQuickTask);
    assert_eq!(commands.len(), 2);
    let executor = CommandExecutor::new();
    for cmd in commands {
        executor
            .execute(cmd, &state.storage)
            .expect("execute save command");
    }

    // Verify task is now in in-memory state and persisted
    assert_eq!(state.tasks.len(), 1);
    let task_id = state.tasks[0].id;
    assert_eq!(state.tasks[0].article_id, article_id);
    assert_eq!(state.tasks[0].title, "Request GPS transit logs");
    assert_eq!(state.tasks[0].status, TaskStatus::ToDo);

    // Verify draft input was cleared
    if let ModalState::ArticleForm(ref draft) = state.modal {
        assert_eq!(draft.quick_task_title, "");
    } else {
        panic!("Expected ArticleForm modal");
    }

    // Verify view model reflects the new task
    if let ModalState::ArticleForm(ref draft) = state.modal {
        let form_view = build_article_form_view(&state, draft);
        assert_eq!(form_view.tasks_section.total_count, 1);
        assert_eq!(form_view.tasks_section.completed_count, 0);
        assert_eq!(
            form_view.tasks_section.tasks[0].title,
            "Request GPS transit logs"
        );
    }

    // Toggle task status inline (ToDo -> Complete)
    let toggle_cmds = state.update(AppMessage::ToggleArticleDraftTask(task_id));
    assert_eq!(toggle_cmds.len(), 1);
    for cmd in toggle_cmds {
        executor
            .execute(cmd, &state.storage)
            .expect("execute toggle command");
    }
    assert_eq!(state.tasks[0].status, TaskStatus::Complete);

    if let ModalState::ArticleForm(ref draft) = state.modal {
        let form_view = build_article_form_view(&state, draft);
        assert_eq!(form_view.tasks_section.completed_count, 1);
        assert_eq!(form_view.tasks_section.completion_percentage, 100);
        assert_eq!(
            form_view.tasks_section.progress_label,
            "1 of 1 completed (100%)"
        );
    }

    // Toggle task status inline again (Complete -> ToDo)
    let toggle_cmds2 = state.update(AppMessage::ToggleArticleDraftTask(task_id));
    for cmd in toggle_cmds2 {
        executor
            .execute(cmd, &state.storage)
            .expect("execute toggle command");
    }
    assert_eq!(state.tasks[0].status, TaskStatus::ToDo);

    // Delete task inline
    let delete_cmds = state.update(AppMessage::DeleteArticleDraftTask(task_id));
    assert_eq!(delete_cmds.len(), 2); // DeleteTask + Toast
    for cmd in delete_cmds {
        executor
            .execute(cmd, &state.storage)
            .expect("execute delete command");
    }
    assert_eq!(state.tasks.len(), 0);

    if let ModalState::ArticleForm(ref draft) = state.modal {
        let form_view = build_article_form_view(&state, draft);
        assert_eq!(form_view.tasks_section.total_count, 0);
    }
}

#[test]
fn test_quick_add_staged_tasks_for_new_article_workflow() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let executor = CommandExecutor::new();

    // Open modal to create a new article
    state.update(AppMessage::OpenNewArticleModal);

    // Set article headline and slug
    state.update(AppMessage::UpdateArticleHeadline(
        "Water Quality Report 2026".to_string(),
    ));

    // Stage Task 1
    state.update(AppMessage::SetArticleDraftQuickTask(
        "Collect sample readings".to_string(),
    ));
    state.update(AppMessage::AddArticleDraftQuickTask);

    // Stage Task 2
    state.update(AppMessage::SetArticleDraftQuickTask(
        "Interview lab technician".to_string(),
    ));
    state.update(AppMessage::AddArticleDraftQuickTask);

    // Stage Task 3 (will delete)
    state.update(AppMessage::SetArticleDraftQuickTask(
        "Temporary scratch note".to_string(),
    ));
    state.update(AppMessage::AddArticleDraftQuickTask);

    // Verify 3 tasks staged in draft
    if let ModalState::ArticleForm(ref draft) = state.modal {
        assert_eq!(draft.staged_tasks.len(), 3);
        let form_view = build_article_form_view(&state, draft);
        assert_eq!(form_view.tasks_section.total_count, 3);
        assert_eq!(form_view.tasks_section.completed_count, 0);
    } else {
        panic!("Expected ArticleForm modal");
    }

    // Toggle Task 1 in staged draft
    let staged_task1_id = if let ModalState::ArticleForm(ref draft) = state.modal {
        draft.staged_tasks[0].id
    } else {
        panic!("Expected ArticleForm modal");
    };
    state.update(AppMessage::ToggleArticleDraftTask(staged_task1_id));

    // Delete Task 3 from staged draft
    let staged_task3_id = if let ModalState::ArticleForm(ref draft) = state.modal {
        draft.staged_tasks[2].id
    } else {
        panic!("Expected ArticleForm modal");
    };
    state.update(AppMessage::DeleteArticleDraftTask(staged_task3_id));

    if let ModalState::ArticleForm(ref draft) = state.modal {
        assert_eq!(draft.staged_tasks.len(), 2);
        assert_eq!(draft.staged_tasks[0].status, TaskStatus::Complete);
        assert_eq!(draft.staged_tasks[1].status, TaskStatus::ToDo);
    }

    // Submit the Article form
    let submit_cmds = state.update(AppMessage::SubmitModal);
    assert!(!submit_cmds.is_empty());
    for cmd in submit_cmds {
        executor
            .execute(cmd, &state.storage)
            .expect("execute submit command");
    }

    // Reload state from storage
    state.load_all().expect("reload state");

    // Modal should now be closed
    assert!(!state.modal.is_open());

    // Verify article was saved
    assert_eq!(state.articles.len(), 1);
    let created_article_id = state.articles[0].id;
    assert_eq!(state.articles[0].slug, "water-quality-report-2026");

    // Verify staged tasks were saved with the assigned article_id
    assert_eq!(state.tasks.len(), 2);
    let t1 = state
        .tasks
        .iter()
        .find(|t| t.title == "Collect sample readings")
        .expect("task 1 found");
    assert_eq!(t1.article_id, created_article_id);
    assert_eq!(t1.status, TaskStatus::Complete);

    let t2 = state
        .tasks
        .iter()
        .find(|t| t.title == "Interview lab technician")
        .expect("task 2 found");
    assert_eq!(t2.article_id, created_article_id);
    assert_eq!(t2.status, TaskStatus::ToDo);
}

#[test]
fn test_cosmic_and_macos_app_view_tree_inline_tasks_integration() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let article = Article::new("port-security", "Port Security Audit");
    let article_id = article.id;
    state.articles.push(article.clone());

    let mut t1 = Task::new(article_id, "Check harbor camera feeds");
    t1.status = TaskStatus::Complete;
    let t2 = Task::new(article_id, "Interview coast guard commander");
    state.tasks = vec![t1, t2];

    // Open article form
    state.update(AppMessage::OpenEditArticleModal(article_id));

    // Test COSMIC App view tree
    let cosmic_app = CosmicApp::new(state.clone());
    let cosmic_tree = cosmic_app.build_view_tree();
    assert!(cosmic_tree.modal_container.is_some());
    let modal_container = cosmic_tree.modal_container.unwrap();
    assert!(modal_container.article_form.is_some());
    let form_view = modal_container.article_form.unwrap();
    assert_eq!(form_view.tasks_section.total_count, 2);
    assert_eq!(form_view.tasks_section.completed_count, 1);
    assert_eq!(form_view.tasks_section.completion_percentage, 50);
    assert_eq!(form_view.task_stats, (1, 2));

    // Test macOS App view tree
    let macos_app = MacosApp::new(state.clone());
    let macos_tree = macos_app.build_view_tree();
    assert!(macos_tree.modal_container.is_some());
    let macos_modal_container = macos_tree.modal_container.unwrap();
    assert!(macos_modal_container.article_form.is_some());
    let macos_form_view = macos_modal_container.article_form.unwrap();
    assert_eq!(macos_form_view.tasks_section.total_count, 2);
    assert_eq!(macos_form_view.tasks_section.completed_count, 1);
    assert_eq!(macos_form_view.task_stats, (1, 2));
}
