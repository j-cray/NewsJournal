//! Comprehensive integration tests for Task 6.5: Save & Cancel Actions.
//!
//! Tests:
//! 1. Keyboard shortcuts (`Escape` to close, `Ctrl+S`/`⌘S` to save, `Enter` to confirm dialogs).
//! 2. Inline sub-form escape cancellation priority.
//! 3. Error validation tooltips across all form fields (Slug, Headline, Color, Inline Contact, Quick Task).
//! 4. Modal footer save/cancel tooltips, shortcuts, and validation summaries.
//! 5. Reducer validation failure handling and warning toast emissions.
//! 6. Platform runtime integration with `CosmicApp` and `MacosApp` key event routing.

use newsjournal_core::models::Article;
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::navigation::{resolve_app_shortcut, resolve_modal_shortcut, NavKeyModifiers};
use newsjournal_gui::platform::cosmic::CosmicApp;
use newsjournal_gui::platform::macos::MacosApp;
use newsjournal_gui::state::modal::{ArticleDraft, ContactDraft, ModalState, TaskDraft};
use newsjournal_gui::state::AppState;
use newsjournal_gui::views::article_form::{build_article_form_view, ValidationSeverity};
use newsjournal_gui::views::ModalFooterViewModel;
use uuid::Uuid;

#[test]
fn test_escape_key_closes_modals_across_all_variants() {
    let linux_none = NavKeyModifiers::none();
    let macos_none = NavKeyModifiers::none();

    // 1. ArticleForm
    let article_modal = ModalState::ArticleForm(ArticleDraft::new());
    assert_eq!(
        resolve_modal_shortcut("Escape", linux_none, false, &article_modal),
        Some(AppMessage::CloseModal)
    );
    assert_eq!(
        resolve_modal_shortcut("Esc", macos_none, true, &article_modal),
        Some(AppMessage::CloseModal)
    );

    // 2. TaskForm
    let task_modal = ModalState::TaskForm(TaskDraft::default());
    assert_eq!(
        resolve_modal_shortcut("Escape", linux_none, false, &task_modal),
        Some(AppMessage::CloseModal)
    );

    // 3. ContactForm
    let contact_modal = ModalState::ContactForm(ContactDraft::new());
    assert_eq!(
        resolve_modal_shortcut("Escape", linux_none, false, &contact_modal),
        Some(AppMessage::CloseModal)
    );

    // 4. SettingsDrawer
    let settings_modal = ModalState::SettingsDrawer(newsjournal_gui::SettingsDraft::from_settings(
        &newsjournal_core::models::Settings::default(),
    ));
    assert_eq!(
        resolve_modal_shortcut("Escape", linux_none, false, &settings_modal),
        Some(AppMessage::CloseModal)
    );

    // 5. Confirm Delete Article
    let delete_modal = ModalState::ConfirmDeleteArticle {
        id: Uuid::new_v4(),
        slug: "city-hall".to_string(),
        headline: "City Hall Probe".to_string(),
    };
    assert_eq!(
        resolve_modal_shortcut("Escape", linux_none, false, &delete_modal),
        Some(AppMessage::CloseModal)
    );

    // 6. Confirm Delete Task
    let delete_task_modal = ModalState::ConfirmDeleteTask {
        id: Uuid::new_v4(),
        title: "Call source".to_string(),
    };
    assert_eq!(
        resolve_modal_shortcut("Escape", linux_none, false, &delete_task_modal),
        Some(AppMessage::CloseModal)
    );

    // 7. Confirm Delete Contact
    let delete_contact_modal =
        ModalState::confirm_delete_contact_simple(Uuid::new_v4(), "Jane Doe", 2);
    assert_eq!(
        resolve_modal_shortcut("Escape", linux_none, false, &delete_contact_modal),
        Some(AppMessage::CloseModal)
    );
}

#[test]
fn test_escape_key_closes_inline_subform_first_when_open() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let mut draft = ArticleDraft::new();
    draft.open_inline_contact();
    assert!(draft.is_inline_contact_open());
    state.modal = ModalState::ArticleForm(draft);

    let modifiers = NavKeyModifiers::none();

    // First Escape closes the inline sub-form
    let msg1 = resolve_app_shortcut("Escape", modifiers, false, &state);
    assert_eq!(msg1, Some(AppMessage::CloseArticleDraftInlineContact));

    let commands1 = state.update(msg1.unwrap());
    assert!(commands1.is_empty());
    assert!(state.modal.is_open());
    if let ModalState::ArticleForm(ref d) = state.modal {
        assert!(!d.is_inline_contact_open());
    } else {
        panic!("Expected ArticleForm modal");
    }

    // Second Escape closes the modal entirely
    let msg2 = resolve_app_shortcut("Escape", modifiers, false, &state);
    assert_eq!(msg2, Some(AppMessage::CloseModal));

    let commands2 = state.update(msg2.unwrap());
    assert!(commands2.is_empty());
    assert!(!state.modal.is_open());
}

#[test]
fn test_save_shortcuts_on_linux_and_macos() {
    let linux_ctrl = NavKeyModifiers {
        ctrl: true,
        meta: false,
        alt: false,
        shift: false,
    };
    let macos_cmd = NavKeyModifiers {
        ctrl: false,
        meta: true,
        alt: false,
        shift: false,
    };
    let none = NavKeyModifiers::none();

    let modal = ModalState::ArticleForm(ArticleDraft::new());

    // Linux Ctrl+S and Ctrl+Enter
    assert_eq!(
        resolve_modal_shortcut("s", linux_ctrl, false, &modal),
        Some(AppMessage::SubmitModal)
    );
    assert_eq!(
        resolve_modal_shortcut("S", linux_ctrl, false, &modal),
        Some(AppMessage::SubmitModal)
    );
    assert_eq!(
        resolve_modal_shortcut("Enter", linux_ctrl, false, &modal),
        Some(AppMessage::SubmitModal)
    );
    assert_eq!(
        resolve_modal_shortcut("Return", linux_ctrl, false, &modal),
        Some(AppMessage::SubmitModal)
    );

    // macOS ⌘S and ⌘Enter
    assert_eq!(
        resolve_modal_shortcut("s", macos_cmd, true, &modal),
        Some(AppMessage::SubmitModal)
    );
    assert_eq!(
        resolve_modal_shortcut("S", macos_cmd, true, &modal),
        Some(AppMessage::SubmitModal)
    );
    assert_eq!(
        resolve_modal_shortcut("Enter", macos_cmd, true, &modal),
        Some(AppMessage::SubmitModal)
    );
    assert_eq!(
        resolve_modal_shortcut("Return", macos_cmd, true, &modal),
        Some(AppMessage::SubmitModal)
    );

    // Modifier mismatch: Ctrl on macOS or ⌘ on Linux should not trigger primary
    assert_eq!(resolve_modal_shortcut("s", linux_ctrl, true, &modal), None);
    assert_eq!(resolve_modal_shortcut("s", macos_cmd, false, &modal), None);

    // Plain 's' or plain 'Enter' in ArticleForm should not trigger submit
    assert_eq!(resolve_modal_shortcut("s", none, false, &modal), None);
    assert_eq!(resolve_modal_shortcut("Enter", none, false, &modal), None);
}

#[test]
fn test_confirm_dialog_enter_shortcut() {
    let none = NavKeyModifiers::none();
    let delete_modal = ModalState::ConfirmDeleteArticle {
        id: Uuid::new_v4(),
        slug: "city-hall".to_string(),
        headline: "City Hall".to_string(),
    };

    // Plain Enter in confirmation dialog submits modal
    assert_eq!(
        resolve_modal_shortcut("Enter", none, false, &delete_modal),
        Some(AppMessage::SubmitModal)
    );
    assert_eq!(
        resolve_modal_shortcut("Return", none, false, &delete_modal),
        Some(AppMessage::SubmitModal)
    );
}

#[test]
fn test_global_creation_shortcuts_when_modal_closed() {
    let state = AppState::in_memory().expect("in-memory state");
    assert!(!state.modal.is_open());

    let linux_ctrl = NavKeyModifiers {
        ctrl: true,
        meta: false,
        alt: false,
        shift: false,
    };
    let linux_ctrl_shift = NavKeyModifiers {
        ctrl: true,
        meta: false,
        alt: false,
        shift: true,
    };

    // Ctrl+N: New Article
    assert_eq!(
        resolve_app_shortcut("n", linux_ctrl, false, &state),
        Some(AppMessage::OpenNewArticleModal)
    );

    // Ctrl+T: New Task
    assert_eq!(
        resolve_app_shortcut("t", linux_ctrl, false, &state),
        Some(AppMessage::OpenNewTaskModal(None))
    );

    // Ctrl+Shift+N: New Task
    assert_eq!(
        resolve_app_shortcut("n", linux_ctrl_shift, false, &state),
        Some(AppMessage::OpenNewTaskModal(None))
    );

    // Ctrl+Shift+C: New Contact
    assert_eq!(
        resolve_app_shortcut("c", linux_ctrl_shift, false, &state),
        Some(AppMessage::OpenNewContactModal)
    );

    // Ctrl+R: Refresh
    assert_eq!(
        resolve_app_shortcut("r", linux_ctrl, false, &state),
        Some(AppMessage::Refresh)
    );

    // F5: Refresh
    assert_eq!(
        resolve_app_shortcut("F5", NavKeyModifiers::none(), false, &state),
        Some(AppMessage::Refresh)
    );
}

#[test]
fn test_reducer_validation_failure_emits_warning_toast() {
    let mut state = AppState::in_memory().expect("in-memory state");
    state.modal = ModalState::ArticleForm(ArticleDraft::new());

    // Submit invalid draft
    let commands = state.update(AppMessage::SubmitModal);
    assert!(state.modal.is_open());
    assert_eq!(commands.len(), 1);

    match &commands[0] {
        newsjournal_gui::AppCommand::EmitToast(toast) => {
            assert_eq!(toast.title, "Validation Error");
            assert_eq!(toast.kind, newsjournal_gui::ToastKind::Warning);
        }
        other => panic!("Expected EmitToast warning, got: {other:?}"),
    }

    // Submit valid draft
    let mut valid_draft = ArticleDraft::new();
    valid_draft.slug = "transit-reform".to_string();
    valid_draft.headline = "Transit Reform Package Approved".to_string();
    valid_draft.validate();
    state.modal = ModalState::ArticleForm(valid_draft);

    let commands_valid = state.update(AppMessage::SubmitModal);
    assert!(!state.modal.is_open());
    assert!(commands_valid
        .iter()
        .any(|c| matches!(c, newsjournal_gui::AppCommand::SaveArticle(_))));
    assert!(commands_valid.iter().any(|c| matches!(
        c,
        newsjournal_gui::AppCommand::EmitToast(t) if t.kind == newsjournal_gui::ToastKind::Success
    )));
}

#[test]
fn test_error_validation_tooltips_generation() {
    let mut state = AppState::in_memory().expect("in-memory state");
    state
        .articles
        .push(Article::new("existing-probe", "Existing Story Probe"));

    // 1. Empty draft -> tooltips for slug and headline
    let mut draft = ArticleDraft::new();
    draft.validate();

    let form_view = build_article_form_view(&state, &draft);
    assert!(!form_view.is_submittable);
    assert!(form_view.slug_field.tooltip.is_some());
    assert!(form_view.headline_field.tooltip.is_some());

    let slug_tt = form_view.slug_field.tooltip.as_ref().unwrap();
    assert_eq!(slug_tt.field_id, "slug");
    assert_eq!(slug_tt.severity, ValidationSeverity::Error);
    assert_eq!(slug_tt.icon_emoji, "⚠️");
    assert!(slug_tt.is_visible);

    // 2. Slug collision tooltip
    draft.slug = "existing-probe".to_string();
    draft.headline = "Valid Story Headline".to_string();
    let existing_slugs = vec![(state.articles[0].id, "existing-probe".to_string())];
    draft.validate_with_existing_slugs(&existing_slugs);

    let form_view_collision = build_article_form_view(&state, &draft);
    assert!(form_view_collision.slug_field.is_collision);
    let collision_tt = form_view_collision.slug_field.tooltip.unwrap();
    assert_eq!(collision_tt.field_id, "slug");
    assert_eq!(
        collision_tt.message,
        "Slug is already in use by another story"
    );

    // 3. Color hex error tooltip
    draft.set_slug("new-unique-probe");
    draft.color_hex = "invalid-hex".to_string();
    draft.validate();

    let form_view_color = build_article_form_view(&state, &draft);
    assert!(form_view_color.color_picker.tooltip.is_some());
    let color_tt = form_view_color.color_picker.tooltip.unwrap();
    assert_eq!(color_tt.field_id, "color_hex");
    assert_eq!(color_tt.message, "Color must be a valid #RRGGBB hex code");

    // 4. Inline contact tooltips
    draft.reset_color_to_hash();
    draft.open_inline_contact();
    draft.set_inline_contact_name("");
    draft.set_inline_contact_email("invalid-email-format");
    draft.set_inline_contact_phone("123"); // Too short

    let form_view_inline = build_article_form_view(&state, &draft);
    let inline = form_view_inline.contacts_section.inline_form.unwrap();
    assert!(inline.name_tooltip.is_some());
    assert!(inline.email_tooltip.is_some());
    assert!(inline.phone_tooltip.is_some());

    // 5. Quick task tooltip
    draft.close_inline_contact();
    draft.set_quick_task_title("   ");
    let form_view_task = build_article_form_view(&state, &draft);
    assert!(form_view_task.tasks_section.quick_task_tooltip.is_none()); // empty input produces no error until typed

    draft.set_quick_task_title(&"a".repeat(350)); // exceeds 300 chars
    let form_view_task2 = build_article_form_view(&state, &draft);
    assert!(form_view_task2.tasks_section.quick_task_tooltip.is_some());
    let task_tt = form_view_task2
        .tasks_section
        .quick_task_tooltip
        .as_ref()
        .unwrap();
    assert_eq!(task_tt.field_id, "quick_task");

    // 6. all_validation_tooltips helper
    let all_tooltips = form_view_task2.all_validation_tooltips();
    assert!(all_tooltips.iter().any(|t| t.field_id == "quick_task"));
}

#[test]
fn test_modal_footer_view_model_tooltips_and_status() {
    // 1. Invalid draft
    let mut invalid_draft = ArticleDraft::new();
    invalid_draft.validate();
    let invalid_modal = ModalState::ArticleForm(invalid_draft);

    let footer_linux = ModalFooterViewModel::from_modal_state(&invalid_modal, false);
    assert!(!footer_linux.can_save);
    assert!(footer_linux.primary_is_disabled);
    assert_eq!(footer_linux.primary_shortcut, "Ctrl+S");
    assert_eq!(footer_linux.secondary_shortcut, "Esc");
    assert!(footer_linux.save_tooltip.contains("Cannot save"));
    assert!(footer_linux.save_tooltip.contains("Ctrl+S"));
    assert_eq!(
        footer_linux.cancel_tooltip,
        "Cancel and discard changes (Esc)"
    );
    assert_eq!(footer_linux.validation_error_count, 2);

    let footer_macos = ModalFooterViewModel::from_modal_state(&invalid_modal, true);
    assert_eq!(footer_macos.primary_shortcut, "⌘S");
    assert!(footer_macos.save_tooltip.contains("⌘S"));

    // 2. Valid draft
    let mut valid_draft = ArticleDraft::new();
    valid_draft.slug = "city-probe".to_string();
    valid_draft.headline = "City Probe Begins".to_string();
    valid_draft.validate();
    let valid_modal = ModalState::ArticleForm(valid_draft);

    let footer_valid = ModalFooterViewModel::from_modal_state(&valid_modal, false);
    assert!(footer_valid.can_save);
    assert!(!footer_valid.primary_is_disabled);
    assert_eq!(footer_valid.save_tooltip, "Create Article (Ctrl+S)");

    // 3. Confirm Delete footer
    let delete_modal = ModalState::ConfirmDeleteArticle {
        id: Uuid::new_v4(),
        slug: "city-probe".to_string(),
        headline: "City Probe".to_string(),
    };
    let footer_delete = ModalFooterViewModel::from_modal_state(&delete_modal, false);
    assert!(footer_delete.can_save);
    assert!(footer_delete.primary_is_destructive);
    assert_eq!(footer_delete.primary_shortcut, "Enter");
    assert_eq!(
        footer_delete.save_tooltip,
        "Permanently delete item (Enter)"
    );
    assert_eq!(footer_delete.cancel_tooltip, "Cancel and keep item (Esc)");
}

#[test]
fn test_cosmic_and_macos_app_handle_key_event_integration() {
    // 1. COSMIC app test
    let state = AppState::in_memory().expect("in-memory state");
    let mut cosmic_app = CosmicApp::new(state);

    let linux_ctrl = NavKeyModifiers {
        ctrl: true,
        meta: false,
        alt: false,
        shift: false,
    };
    let none = NavKeyModifiers::none();

    // Ctrl+N opens new article modal
    assert!(!cosmic_app.state().modal.is_open());
    let handled = cosmic_app.handle_key_event("n", linux_ctrl).unwrap();
    assert!(handled);
    assert!(cosmic_app.state().modal.is_open());

    // Escape closes modal
    let handled_esc = cosmic_app.handle_key_event("Escape", none).unwrap();
    assert!(handled_esc);
    assert!(!cosmic_app.state().modal.is_open());

    // 2. macOS app test
    let state_macos = AppState::in_memory().expect("in-memory state");
    let mut macos_app = MacosApp::new(state_macos);

    let macos_cmd = NavKeyModifiers {
        ctrl: false,
        meta: true,
        alt: false,
        shift: false,
    };

    // ⌘N opens new article modal
    assert!(!macos_app.state().modal.is_open());
    let handled_macos = macos_app.handle_key_event("n", macos_cmd).unwrap();
    assert!(handled_macos);
    assert!(macos_app.state().modal.is_open());

    // ⌘S with invalid form triggers validation warning toast and keeps modal open
    let handled_save_fail = macos_app.handle_key_event("s", macos_cmd).unwrap();
    assert!(handled_save_fail);
    assert!(macos_app.state().modal.is_open());
    assert!(macos_app
        .state()
        .toasts
        .iter()
        .any(|t| t.title == "Validation Error"));

    // Populate valid data and save via ⌘S
    macos_app
        .dispatch(AppMessage::UpdateArticleSlug(
            "harbor-deepening".to_string(),
        ))
        .unwrap();
    macos_app
        .dispatch(AppMessage::UpdateArticleHeadline(
            "Harbor Deepening Project".to_string(),
        ))
        .unwrap();
    let handled_save_success = macos_app.handle_key_event("s", macos_cmd).unwrap();
    assert!(handled_save_success);
    assert!(!macos_app.state().modal.is_open());
    assert_eq!(macos_app.state().articles.len(), 1);
    assert_eq!(macos_app.state().articles[0].slug, "harbor-deepening");
}
