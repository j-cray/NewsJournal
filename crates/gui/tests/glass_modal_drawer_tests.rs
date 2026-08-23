//! Integration tests for Task 6.1: Glass Modal / Drawer Container.
//!
//! Tests centered modal vs slide-over drawer layouts, backdrop dimming and blur,
//! header and footer view model generation, keyboard shortcuts, and platform view tree integration.

use newsjournal_core::models::{Article, ArticleStage, ThemeMode};
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::platform::cosmic::CosmicApp;
use newsjournal_gui::platform::macos::MacosApp;
use newsjournal_gui::state::modal::{ArticleDraft, ContactDraft, ModalState, TaskDraft};
use newsjournal_gui::state::AppState;
use newsjournal_gui::views::{
    build_modal_container_view, build_modal_container_view_with_layout, ModalGeometry,
    ModalPlacement,
};
use uuid::Uuid;

#[test]
fn test_modal_placement_defaults_and_explicit_geometry() {
    let none_state = ModalState::None;
    assert_eq!(
        ModalPlacement::default_for(&none_state),
        ModalPlacement::CenteredModal
    );

    let article_draft = ArticleDraft::new();
    let article_state = ModalState::ArticleForm(article_draft);
    assert_eq!(
        ModalPlacement::default_for(&article_state),
        ModalPlacement::SlideOverRight
    );

    let task_state = ModalState::TaskForm(TaskDraft::default());
    assert_eq!(
        ModalPlacement::default_for(&task_state),
        ModalPlacement::SlideOverRight
    );

    let contact_state = ModalState::ContactForm(ContactDraft::new());
    assert_eq!(
        ModalPlacement::default_for(&contact_state),
        ModalPlacement::SlideOverRight
    );

    let delete_article_state = ModalState::ConfirmDeleteArticle {
        id: Uuid::new_v4(),
        slug: "city-hall-audit".to_string(),
        headline: "City Hall Audit Underway".to_string(),
    };
    assert_eq!(
        ModalPlacement::default_for(&delete_article_state),
        ModalPlacement::CenteredModal
    );

    // Verify slide-over drawer geometry
    let drawer_geo = ModalGeometry::resolve(ModalPlacement::SlideOverRight, &article_state);
    assert_eq!(drawer_geo.width, 580.0);
    assert_eq!(drawer_geo.min_width, 420.0);
    assert_eq!(drawer_geo.max_width, 760.0);
    assert!(drawer_geo.height.is_none());
    assert_eq!(drawer_geo.corner_radius, 16.0);
    assert_eq!(drawer_geo.padding, 24.0);
    assert_eq!(drawer_geo.z_index, 100);

    // Verify centered modal geometry
    let centered_geo = ModalGeometry::resolve(ModalPlacement::CenteredModal, &article_state);
    assert_eq!(centered_geo.width, 520.0);
    assert_eq!(centered_geo.min_width, 380.0);
    assert_eq!(centered_geo.max_width, 680.0);
    assert_eq!(centered_geo.max_height, Some(720.0));
    assert_eq!(centered_geo.corner_radius, 16.0);

    // Verify delete confirmation compact geometry
    let delete_geo = ModalGeometry::resolve(ModalPlacement::CenteredModal, &delete_article_state);
    assert_eq!(delete_geo.width, 440.0);
    assert_eq!(delete_geo.min_width, 340.0);
    assert_eq!(delete_geo.max_width, 520.0);
    assert_eq!(delete_geo.max_height, Some(360.0));
    assert_eq!(delete_geo.corner_radius, 14.0);
    assert_eq!(delete_geo.z_index, 110);
}

#[test]
fn test_modal_backdrop_dimming_and_blur() {
    let mut state = AppState::in_memory().expect("in-memory state");

    // Modal closed
    let container_closed = build_modal_container_view(&state);
    assert!(!container_closed.is_open);
    assert!(!container_closed.backdrop.is_visible);
    assert_eq!(container_closed.backdrop.blur_radius, 0.0);

    // Modal open in Dark mode (default)
    state.modal = ModalState::ArticleForm(ArticleDraft::new());
    let container_dark = build_modal_container_view(&state);
    assert!(container_dark.is_open);
    assert!(container_dark.backdrop.is_visible);
    assert_eq!(container_dark.backdrop.color_rgba, (0, 0, 0, 0.65));
    assert_eq!(container_dark.backdrop.blur_radius, 28.0);
    assert!(container_dark.backdrop.dismiss_on_click);

    // Modal open in Light mode
    state.settings.theme_mode = ThemeMode::Light;
    state.theme_engine.set_mode(ThemeMode::Light);
    let container_light = build_modal_container_view(&state);
    assert!(container_light.backdrop.is_visible);
    assert_eq!(container_light.backdrop.color_rgba, (0, 0, 0, 0.40));
    assert_eq!(container_light.backdrop.blur_radius, 28.0);

    // Destructive delete confirmation modal (backdrop click dismiss disabled for safety)
    state.modal = ModalState::ConfirmDeleteTask {
        id: Uuid::new_v4(),
        title: "Check source notes".to_string(),
    };
    let container_delete = build_modal_container_view(&state);
    assert!(container_delete.is_open);
    assert!(container_delete.backdrop.is_visible);
    assert!(!container_delete.backdrop.dismiss_on_click);
    assert!(container_delete.is_destructive_prompt());
}

#[test]
fn test_modal_header_presentation_and_badges() {
    let mut state = AppState::in_memory().expect("in-memory state");

    // Article creation header
    let mut article_draft = ArticleDraft::new();
    article_draft.stage = ArticleStage::Researching;
    state.modal = ModalState::ArticleForm(article_draft);
    let container_article_new = build_modal_container_view(&state);
    assert_eq!(container_article_new.header.title, "New Article");
    assert_eq!(
        container_article_new.header.subtitle,
        "Create a new reporting pitch or story"
    );
    assert_eq!(container_article_new.header.icon_emoji, "📰");
    assert_eq!(container_article_new.header.icon_name, "document-edit");
    assert_eq!(
        container_article_new.header.badge_text,
        Some("Researching".to_string())
    );
    assert!(container_article_new.header.show_close_button);
    assert_eq!(container_article_new.header.close_shortcut, "Esc");

    // Article edit header
    let mut edit_draft = ArticleDraft::new();
    edit_draft.id = Some(Uuid::new_v4());
    edit_draft.stage = ArticleStage::Editing;
    edit_draft.color_hex = "#1A85FF".to_string();
    state.modal = ModalState::ArticleForm(edit_draft);
    let container_article_edit = build_modal_container_view(&state);
    assert_eq!(container_article_edit.header.title, "Edit Article");
    assert_eq!(
        container_article_edit.header.subtitle,
        "Update story details, deadlines, and contacts"
    );
    assert_eq!(
        container_article_edit.header.badge_text,
        Some("Editing".to_string())
    );
    assert_eq!(
        container_article_edit.header.badge_color_hex,
        Some("#1A85FF".to_string())
    );

    // Task edit header
    let task_draft = TaskDraft {
        id: Some(Uuid::new_v4()),
        ..Default::default()
    };
    state.modal = ModalState::TaskForm(task_draft);
    let container_task = build_modal_container_view(&state);
    assert_eq!(container_task.header.title, "Edit Task");
    assert_eq!(container_task.header.icon_emoji, "✅");

    // Contact creation header
    state.modal = ModalState::ContactForm(ContactDraft::new());
    let container_contact = build_modal_container_view(&state);
    assert_eq!(container_contact.header.title, "New Contact");
    assert_eq!(container_contact.header.icon_emoji, "👥");

    // Delete contact confirmation header
    state.modal = ModalState::confirm_delete_contact_simple(Uuid::new_v4(), "Deep Throat", 3);
    let container_del_contact = build_modal_container_view(&state);
    assert_eq!(container_del_contact.header.title, "Delete Contact?");
    assert_eq!(container_del_contact.header.icon_emoji, "⚠️");
    assert_eq!(
        container_del_contact.header.badge_text,
        Some("3 linked articles".to_string())
    );
    assert_eq!(
        container_del_contact.header.badge_color_hex,
        Some("#FF4D4F".to_string())
    );
}

#[test]
fn test_modal_footer_actions_shortcuts_and_validation_hints() {
    let mut state = AppState::in_memory().expect("in-memory state");

    // Invalid article draft on Linux
    let mut invalid_draft = ArticleDraft::new();
    invalid_draft.validate();
    state.modal = ModalState::ArticleForm(invalid_draft);
    let footer_linux = build_modal_container_view_with_layout(
        &state,
        ModalPlacement::SlideOverRight,
        false, // Linux
    )
    .footer;
    assert_eq!(footer_linux.primary_label, "Create Article");
    assert_eq!(footer_linux.primary_shortcut, "Ctrl+S");
    assert!(footer_linux.primary_is_disabled);
    assert_eq!(footer_linux.secondary_label, "Cancel");
    assert_eq!(footer_linux.secondary_shortcut, "Esc");
    assert!(footer_linux.validation_error_count > 0);
    assert!(footer_linux.validation_hint.is_some());

    // Valid article draft on macOS
    let mut valid_draft = ArticleDraft::new();
    valid_draft.id = Some(Uuid::new_v4());
    valid_draft.slug = "transit-strike".to_string();
    valid_draft.headline = "Transit Union Calls 48-Hour Strike".to_string();
    valid_draft.validate();
    state.modal = ModalState::ArticleForm(valid_draft);
    let footer_macos = build_modal_container_view_with_layout(
        &state,
        ModalPlacement::SlideOverRight,
        true, // macOS
    )
    .footer;
    assert_eq!(footer_macos.primary_label, "Save Changes");
    assert_eq!(footer_macos.primary_shortcut, "⌘S");
    assert!(!footer_macos.primary_is_disabled);
    assert_eq!(footer_macos.validation_error_count, 0);
    assert!(footer_macos.validation_hint.is_none());

    // Destructive delete confirmation
    state.modal = ModalState::ConfirmDeleteArticle {
        id: Uuid::new_v4(),
        slug: "old-story".to_string(),
        headline: "Old Story".to_string(),
    };
    let footer_delete = build_modal_container_view(&state).footer;
    assert_eq!(footer_delete.primary_label, "Delete Permanently");
    assert_eq!(footer_delete.primary_shortcut, "Enter");
    assert!(footer_delete.primary_is_destructive);
    assert!(!footer_delete.primary_is_disabled);
    assert_eq!(footer_delete.secondary_label, "Cancel");
}

#[test]
fn test_cosmic_and_macos_app_modal_container_view_tree_integration() {
    // 1. Test COSMIC application
    let state_cosmic = AppState::in_memory().expect("in-memory state");
    let mut cosmic_app = CosmicApp::new(state_cosmic);

    assert!(cosmic_app.build_view_tree().modal_container.is_none());

    cosmic_app
        .dispatch(AppMessage::OpenNewArticleModal)
        .expect("open article modal");
    assert!(cosmic_app.state().modal.is_open());

    let tree_cosmic = cosmic_app.build_view_tree();
    assert!(tree_cosmic.modal_container.is_some());
    let modal_cosmic = tree_cosmic.modal_container.unwrap();
    assert!(modal_cosmic.is_open);
    assert_eq!(modal_cosmic.header.title, "New Article");
    assert!(modal_cosmic.backdrop.is_visible);
    assert_eq!(modal_cosmic.glass.blur_radius, 32.0); // COSMIC deep blur
    assert_eq!(modal_cosmic.glass.corner_radius, 16.0);
    assert!(modal_cosmic.glass.shadow.is_some());

    cosmic_app
        .dispatch(AppMessage::CloseModal)
        .expect("close modal");
    assert!(cosmic_app.build_view_tree().modal_container.is_none());

    // 2. Test macOS application
    let state_macos = AppState::in_memory().expect("in-memory state");
    let mut macos_app = MacosApp::new(state_macos);

    assert!(macos_app.build_view_tree().modal_container.is_none());

    macos_app
        .dispatch(AppMessage::OpenNewContactModal)
        .expect("open contact modal");
    assert!(macos_app.state().modal.is_open());

    let tree_macos = macos_app.build_view_tree();
    assert!(tree_macos.modal_container.is_some());
    let modal_macos = tree_macos.modal_container.unwrap();
    assert!(modal_macos.is_open);
    assert_eq!(modal_macos.header.title, "New Contact");
    assert!(modal_macos.backdrop.is_visible);
    assert_eq!(modal_macos.glass.blur_radius, 36.0); // macOS sheet blur
    assert_eq!(modal_macos.glass.corner_radius, 14.0);
    assert!(modal_macos.glass.specular_highlight_rgba.3 > 0.0); // Specular highlight on macOS

    macos_app
        .dispatch(AppMessage::CloseModal)
        .expect("close modal");
    assert!(macos_app.build_view_tree().modal_container.is_none());
}

#[test]
fn test_modal_lifecycle_and_transition_between_forms() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut app = CosmicApp::new(state);

    // 1. Create Article & save
    let article = Article::new("harbor-clean-up", "Harbor Clean-Up Commences");
    let article_id = article.id;
    app.dispatch(AppMessage::CreateArticle(article.clone()))
        .expect("create article");

    // 2. Open edit article modal
    app.dispatch(AppMessage::OpenEditArticleModal(article_id))
        .expect("open edit modal");
    let tree1 = app.build_view_tree();
    let container1 = tree1.modal_container.expect("modal open");
    assert_eq!(container1.header.title, "Edit Article");
    assert_eq!(container1.placement, ModalPlacement::SlideOverRight);

    // 3. Open prompt delete article dialog
    app.dispatch(AppMessage::PromptDeleteArticle(article_id))
        .expect("prompt delete");
    let tree2 = app.build_view_tree();
    let container2 = tree2.modal_container.expect("delete dialog open");
    assert_eq!(container2.header.title, "Delete Article?");
    assert_eq!(container2.placement, ModalPlacement::CenteredModal);
    assert!(container2.is_destructive_prompt());
    assert!(!container2.backdrop.dismiss_on_click);

    // 4. Close modal
    app.dispatch(AppMessage::CloseModal).expect("close modal");
    let tree3 = app.build_view_tree();
    assert!(tree3.modal_container.is_none());
}
