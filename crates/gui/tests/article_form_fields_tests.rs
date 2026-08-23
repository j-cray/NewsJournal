//! Integration tests for Task 6.2: Article Form Fields.
//!
//! Tests:
//! 1. Required unique Slug input with live collision checks.
//! 2. Headline input and multi-line Description / Notes text area.
//! 3. Stage selector dropdown across all 6 editorial stages.
//! 4. Deadline picker with presets (Date + optional Time selector) and overdue warnings.
//! 5. Curated color picker swatch grid with auto-slug hash defaults and custom overrides.
//! 6. Reducer state transitions and platform view tree integration.

use chrono::{Duration, Utc};
use newsjournal_core::color::assign_color_for_slug;
use newsjournal_core::models::{Article, ArticleStage};
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::platform::cosmic::CosmicApp;
use newsjournal_gui::platform::macos::MacosApp;
use newsjournal_gui::state::modal::{ArticleDraft, ModalState};
use newsjournal_gui::state::AppState;
use newsjournal_gui::views::{
    build_article_form_view, DeadlinePreset, DEFAULT_DEADLINE_HOUR, MAX_HEADLINE_LENGTH,
};
use uuid::Uuid;

#[test]
fn test_slug_field_validation_and_live_collision_checking() {
    let mut state = AppState::in_memory().expect("in-memory state");

    // Pre-populate an existing article
    let existing_article = Article::new("city-hall-audit", "City Hall Audit Underway");
    let existing_id = existing_article.id;
    state.articles.push(existing_article);

    // 1. Create mode: empty slug produces validation error
    let mut draft = ArticleDraft::new();
    let form_view1 = build_article_form_view(&state, &draft);
    assert_eq!(form_view1.slug_field.value, "");
    assert!(!form_view1.slug_field.is_valid);
    assert!(!form_view1.is_submittable);

    // 2. Invalid slug format (spaces and uppercase)
    draft.slug = "Invalid Slug With Spaces".to_string();
    draft.headline = "Valid Story Headline".to_string();
    assert!(!draft.validate());
    let form_view2 = build_article_form_view(&state, &draft);
    assert!(form_view2.slug_field.error.is_some());
    assert!(!form_view2.slug_field.is_valid);

    // 3. Collision with existing article in create mode
    draft.set_slug("city-hall-audit");
    assert!(draft.validate()); // Format is valid
    let existing_slugs: Vec<(Uuid, String)> = state
        .articles
        .iter()
        .map(|a| (a.id, a.slug.clone()))
        .collect();
    assert!(draft.slug_collides_with(&existing_slugs));
    assert!(!draft.validate_with_existing_slugs(&existing_slugs));

    let form_view3 = build_article_form_view(&state, &draft);
    assert!(form_view3.slug_field.is_collision);
    assert!(!form_view3.slug_field.is_valid);
    assert_eq!(
        form_view3.slug_field.error.as_deref(),
        Some("Slug is already in use by another story")
    );
    assert!(!form_view3.is_submittable);

    // 4. Non-colliding valid slug
    draft.set_slug("city-hall-audit-part-2");
    assert!(!draft.slug_collides_with(&existing_slugs));
    assert!(draft.validate_with_existing_slugs(&existing_slugs));

    let form_view4 = build_article_form_view(&state, &draft);
    assert!(!form_view4.slug_field.is_collision);
    assert!(form_view4.slug_field.is_valid);
    assert!(form_view4.slug_field.error.is_none());
    assert!(form_view4.is_submittable);

    // 5. Edit mode: keeps own slug without collision error
    let mut edit_draft = ArticleDraft::from_article(&state.articles[0], vec![]);
    assert_eq!(edit_draft.id, Some(existing_id));
    assert_eq!(edit_draft.slug, "city-hall-audit");
    assert!(!edit_draft.slug_collides_with(&existing_slugs));
    assert!(edit_draft.validate_with_existing_slugs(&existing_slugs));

    let form_view5 = build_article_form_view(&state, &edit_draft);
    assert!(!form_view5.slug_field.is_collision);
    assert!(form_view5.slug_field.is_valid);
    assert!(form_view5.is_submittable);
}

#[test]
fn test_headline_input_auto_slug_generation_and_description_metrics() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut draft = ArticleDraft::new();

    // 1. Headline validation
    assert!(!draft.validate());
    draft.headline = "".to_string();
    let form1 = build_article_form_view(&state, &draft);
    assert!(!form1.headline_field.is_valid);
    assert_eq!(form1.headline_field.max_length, MAX_HEADLINE_LENGTH);

    // 2. Auto-slug generation from headline for new article
    draft.set_headline(
        "Breaking News: Mayor Announces $50M Green Transit Expansion!",
        true,
    );
    assert_eq!(
        draft.slug,
        "breaking-news-mayor-announces-50m-green-transit-expansion"
    );
    assert_eq!(
        draft.headline,
        "Breaking News: Mayor Announces $50M Green Transit Expansion!"
    );

    let form2 = build_article_form_view(&state, &draft);
    assert!(form2.headline_field.is_valid);
    assert_eq!(form2.headline_field.char_count, 60);
    assert!(form2.slug_field.is_valid);
    assert_eq!(
        form2.slug_field.value,
        "breaking-news-mayor-announces-50m-green-transit-expansion"
    );

    // 3. Multi-line description metrics
    draft.set_description(
        "Story Angle: Federal vs City funding breakdown.\nKey Contacts: Transit CFO & Union Rep.\nDeadline: Before Friday vote.",
    );
    let form3 = build_article_form_view(&state, &draft);
    assert_eq!(form3.description_field.line_count, 3);
    assert!(form3.description_field.char_count > 50);
}

#[test]
fn test_stage_selector_options_and_sequential_metadata() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut draft = ArticleDraft::new();

    // Default stage is Pitching
    assert_eq!(draft.stage, ArticleStage::Pitching);
    let form_pitch = build_article_form_view(&state, &draft);
    assert_eq!(
        form_pitch.stage_field.selected_stage,
        ArticleStage::Pitching
    );
    assert_eq!(form_pitch.stage_field.selected_label, "Pitching");
    assert_eq!(form_pitch.stage_field.selected_icon, "💡");
    assert_eq!(form_pitch.stage_field.options.len(), 6);

    // Verify all 6 stages are sequentially present
    let expected_stages = [
        (ArticleStage::Pitching, "Pitching", 1, "💡"),
        (ArticleStage::Researching, "Researching", 2, "🔍"),
        (ArticleStage::Writing, "Writing", 3, "✍️"),
        (ArticleStage::Editing, "Editing", 4, "✂️"),
        (ArticleStage::ReadyToPublish, "Ready to Publish", 5, "🚀"),
        (ArticleStage::Published, "Published", 6, "📰"),
    ];

    for (stage, label, step, icon) in expected_stages {
        draft.set_stage(stage);
        let form = build_article_form_view(&state, &draft);
        assert_eq!(form.stage_field.selected_stage, stage);
        assert_eq!(form.stage_field.selected_label, label);

        let opt = form
            .stage_field
            .options
            .iter()
            .find(|o| o.stage == stage)
            .expect("stage option found");
        assert_eq!(opt.label, label);
        assert_eq!(opt.step_number, step);
        assert_eq!(opt.icon_emoji, icon);
        assert!(opt.is_selected);
        assert!(!opt.badge_color_hex.is_empty());
        assert!(!opt.description.is_empty());
    }
}

#[test]
fn test_deadline_picker_presets_formatting_and_overdue_alerts() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut draft = ArticleDraft::new();
    let now = Utc::now();

    // 1. No deadline
    assert!(draft.deadline.is_none());
    let form_none = build_article_form_view(&state, &draft);
    assert!(!form_none.deadline_field.has_deadline);
    assert_eq!(
        form_none.deadline_field.formatted_display,
        "No deadline set"
    );
    assert!(!form_none.deadline_field.is_overdue);

    // 2. Apply Tomorrow 5 PM preset
    let tomorrow_dt = DeadlinePreset::Tomorrow5PM
        .calculate_target_datetime(now)
        .expect("tomorrow 5pm calculated");
    draft.set_deadline(Some(tomorrow_dt));
    let form_tomorrow = build_article_form_view(&state, &draft);
    assert!(form_tomorrow.deadline_field.has_deadline);
    assert!(form_tomorrow.deadline_field.date_iso.is_some());
    assert_eq!(
        form_tomorrow.deadline_field.time_hhmm.as_deref(),
        Some(format!("{DEFAULT_DEADLINE_HOUR:02}:00").as_str())
    );
    assert!(!form_tomorrow.deadline_field.is_overdue);
    assert!(form_tomorrow.deadline_field.relative_hint.contains("Due"));

    // 3. Overdue deadline in the past
    let past_dt = now - Duration::hours(5);
    draft.set_deadline(Some(past_dt));
    draft.set_stage(ArticleStage::Writing);
    let form_overdue = build_article_form_view(&state, &draft);
    assert!(form_overdue.deadline_field.is_overdue);
    assert!(form_overdue
        .deadline_field
        .relative_hint
        .contains("⚠️ Overdue by 5 hour(s)"));

    // 4. Overdue suppressed when stage is Published
    draft.set_stage(ArticleStage::Published);
    let form_published = build_article_form_view(&state, &draft);
    assert!(!form_published.deadline_field.is_overdue);
    assert_eq!(
        form_published.deadline_field.relative_hint,
        "Story published"
    );

    // 5. Presets list verification
    assert_eq!(form_published.deadline_field.presets.len(), 6);
    let preset_labels: Vec<&str> = form_published
        .deadline_field
        .presets
        .iter()
        .map(|p| p.label)
        .collect();
    assert_eq!(
        preset_labels,
        vec![
            "Today 5 PM",
            "Tomorrow 5 PM",
            "End of Week (Fri)",
            "Next Week (Mon)",
            "In 2 Weeks",
            "No Deadline"
        ]
    );
}

#[test]
fn test_color_picker_swatch_grid_and_slug_hash_custom_overrides() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut draft = ArticleDraft::new();

    // 1. Initial color derived from default slug
    let initial_auto_hex = assign_color_for_slug("new-article").to_hex();
    assert_eq!(draft.color_hex, initial_auto_hex);
    assert!(!draft.is_custom_color);

    // 2. Slug change automatically recomputes color hash
    draft.set_slug("maritime-safety-inquiry");
    let expected_hash_hex = assign_color_for_slug("maritime-safety-inquiry").to_hex();
    assert_eq!(draft.color_hex, expected_hash_hex);
    assert!(!draft.is_custom_color);

    let form_auto = build_article_form_view(&state, &draft);
    assert_eq!(form_auto.color_picker.selected_hex, expected_hash_hex);
    assert!(!form_auto.color_picker.is_custom);
    assert_eq!(form_auto.color_picker.auto_hash_hex, expected_hash_hex);
    assert_eq!(form_auto.color_picker.swatches.len(), 16);

    // 3. User selects a curated swatch / custom override
    let custom_hex = "#E53935"; // Crimson
    draft.set_color(custom_hex);
    assert!(draft.is_custom_color);
    assert_eq!(draft.color_hex, custom_hex);

    // 4. Changing slug does NOT overwrite custom color
    draft.set_slug("harbor-inspection-widens");
    assert_eq!(draft.color_hex, custom_hex);
    assert!(draft.is_custom_color);

    let form_custom = build_article_form_view(&state, &draft);
    assert_eq!(form_custom.color_picker.selected_hex, custom_hex);
    assert!(form_custom.color_picker.is_custom);

    let selected_swatch = form_custom
        .color_picker
        .swatches
        .iter()
        .find(|s| s.is_selected)
        .expect("swatch selected");
    assert_eq!(selected_swatch.name, "Crimson");
    assert_eq!(selected_swatch.hex, "#E53935");
    assert!(!selected_swatch.contrast_text_color.is_empty());

    // 5. Reset to slug hash
    draft.reset_color_to_hash();
    assert!(!draft.is_custom_color);
    let reset_expected_hex = assign_color_for_slug("harbor-inspection-widens").to_hex();
    assert_eq!(draft.color_hex, reset_expected_hex);

    let form_reset = build_article_form_view(&state, &draft);
    assert!(!form_reset.color_picker.is_custom);
    assert_eq!(form_reset.color_picker.selected_hex, reset_expected_hex);
}

#[test]
fn test_reducer_article_form_field_messages_and_submission_workflow() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut app = CosmicApp::new(state);

    // 1. Open new article modal
    app.dispatch(AppMessage::OpenNewArticleModal)
        .expect("open modal");
    assert!(app.state().modal.is_open());

    // 2. Dispatch UpdateArticleHeadline -> auto-updates slug
    app.dispatch(AppMessage::UpdateArticleHeadline(
        "Investigation: Police Drone Surveillance Contracts".to_string(),
    ))
    .expect("update headline");

    if let ModalState::ArticleForm(ref draft) = app.state().modal {
        assert_eq!(
            draft.slug,
            "investigation-police-drone-surveillance-contracts"
        );
        assert_eq!(
            draft.headline,
            "Investigation: Police Drone Surveillance Contracts"
        );
    } else {
        panic!("expected ArticleForm modal");
    }

    // 3. Dispatch UpdateArticleDescription
    app.dispatch(AppMessage::UpdateArticleDescription(
        "Freedom of Information Act request pending on vendor pricing.".to_string(),
    ))
    .expect("update description");

    // 4. Dispatch SetArticleDraftStage
    app.dispatch(AppMessage::SetArticleDraftStage(ArticleStage::Researching))
        .expect("set stage");

    // 5. Dispatch SetArticleDraftDeadlinePreset
    app.dispatch(AppMessage::SetArticleDraftDeadlinePreset(
        DeadlinePreset::Tomorrow5PM,
    ))
    .expect("set preset");

    // 6. Dispatch SetArticleDraftColor
    app.dispatch(AppMessage::SetArticleDraftColor("#059669".to_string())) // Emerald
        .expect("set color");

    // 7. Verify view tree has ArticleFormViewModel populated
    let tree = app.build_view_tree();
    let container = tree.modal_container.expect("modal container visible");
    let form = container.article_form.expect("article form populated");

    assert_eq!(
        form.headline_field.value,
        "Investigation: Police Drone Surveillance Contracts"
    );
    assert_eq!(
        form.slug_field.value,
        "investigation-police-drone-surveillance-contracts"
    );
    assert_eq!(form.stage_field.selected_stage, ArticleStage::Researching);
    assert!(form.deadline_field.has_deadline);
    assert_eq!(form.color_picker.selected_hex, "#059669");
    assert!(form.color_picker.is_custom);
    assert!(form.is_submittable);
    assert_eq!(form.total_error_count, 0);

    // 8. Submit modal -> saves article to database and closes modal
    app.dispatch(AppMessage::SubmitModal).expect("submit modal");
    assert!(!app.state().modal.is_open());
    assert_eq!(app.state().articles.len(), 1);

    let saved = &app.state().articles[0];
    assert_eq!(
        saved.slug,
        "investigation-police-drone-surveillance-contracts"
    );
    assert_eq!(
        saved.headline,
        "Investigation: Police Drone Surveillance Contracts"
    );
    assert_eq!(saved.stage, ArticleStage::Researching);
    assert!(saved.deadline.is_some());
    assert_eq!(saved.color.as_deref(), Some("#059669"));

    // 9. Rejection on duplicate slug submission
    app.dispatch(AppMessage::OpenNewArticleModal)
        .expect("open second modal");
    app.dispatch(AppMessage::UpdateArticleSlug(
        "investigation-police-drone-surveillance-contracts".to_string(),
    ))
    .expect("set colliding slug");
    app.dispatch(AppMessage::UpdateArticleHeadline(
        "Another Story with Colliding Slug".to_string(),
    ))
    .expect("set headline");

    // Try to submit duplicate -> must fail and keep modal open with collision error
    app.dispatch(AppMessage::SubmitModal)
        .expect("submit duplicate modal");
    assert!(app.state().modal.is_open());
    assert_eq!(app.state().articles.len(), 1); // Not saved

    if let ModalState::ArticleForm(ref duplicate_draft) = app.state().modal {
        assert!(duplicate_draft.validation_errors.contains_key("slug"));
    } else {
        panic!("expected ArticleForm modal to remain open on collision");
    }
}

#[test]
fn test_macos_view_tree_modal_container_article_form_integration() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut macos_app = MacosApp::new(state);

    macos_app
        .dispatch(AppMessage::OpenNewArticleInStageModal(
            ArticleStage::Writing,
        ))
        .expect("open new article in writing stage");

    let tree = macos_app.build_view_tree();
    assert!(tree.modal_container.is_some());

    let container = tree.modal_container.unwrap();
    assert!(container.is_open);
    assert_eq!(container.header.title, "New Article");
    assert_eq!(container.header.badge_text.as_deref(), Some("Writing"));

    let form = container.article_form.expect("article form populated");
    assert_eq!(form.stage_field.selected_stage, ArticleStage::Writing);
    assert_eq!(form.stage_field.selected_label, "Writing");
}
