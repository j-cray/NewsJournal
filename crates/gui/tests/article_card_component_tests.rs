//! Comprehensive integration and verification tests for the Article Card Component (Task 5.2).

use chrono::{Duration, Utc};
use newsjournal_core::deadline::evaluate_article_deadline;
use newsjournal_core::models::{
    ArticleBuilder, ArticleStage, Contact, ContactBuilder, Task, TaskStatus,
};
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::platform::cosmic::app::CosmicApp;
use newsjournal_gui::platform::macos::app::MacosApp;
use newsjournal_gui::runtime::EventLoop;
use newsjournal_gui::state::AppState;
use newsjournal_gui::views::{
    calculate_contrast_color, format_contact_initials, format_deadline_badge,
    ArticleCardContactTagViewModel, ArticleCardOverdueStyleViewModel, ArticleCardViewModel,
    ArticleSlugBadgeViewModel, ArticleTaskCounterViewModel, ColorIndicatorBarViewModel,
    IndicatorPosition, DEFAULT_ACCENT_STRIP_WIDTH, DEFAULT_CARD_BORDER_WIDTH, DUE_SOON_AMBER_HEX,
    OVERDUE_CARD_BORDER_WIDTH, OVERDUE_RED_HEX, SUCCESS_GREEN_HEX,
};

#[test]
fn test_color_indicator_and_contrast_engine() {
    // Dark background colors require white foreground text
    assert_eq!(calculate_contrast_color("#000000"), "#FFFFFF");
    assert_eq!(calculate_contrast_color("#1E3A8A"), "#FFFFFF");
    assert_eq!(calculate_contrast_color("#EF4444"), "#FFFFFF");
    assert_eq!(calculate_contrast_color("#3B82F6"), "#FFFFFF");

    // Bright/light background colors require dark slate text
    assert_eq!(calculate_contrast_color("#FFFFFF"), "#0F172A");
    assert_eq!(calculate_contrast_color("#F59E0B"), "#0F172A");
    assert_eq!(calculate_contrast_color("#FDE047"), "#0F172A");

    let indicator = ColorIndicatorBarViewModel::new("#10B981");
    assert_eq!(indicator.color_hex, "#10B981");
    assert_eq!(indicator.width_px, DEFAULT_ACCENT_STRIP_WIDTH);
    assert_eq!(indicator.position, IndicatorPosition::LeftAccentStrip);
    assert!(indicator.translucent_tint_hex.starts_with("#10B981"));
}

#[test]
fn test_slug_badge_formatting_and_truncation() {
    let standard = ArticleSlugBadgeViewModel::new("transit-budget");
    assert_eq!(standard.raw_slug, "transit-budget");
    assert_eq!(standard.display_text, "#transit-budget");
    assert_eq!(standard.compact_text, "#transit-budget");

    let long_slug = ArticleSlugBadgeViewModel::new("investigative-state-budget-scandal-2026");
    assert_eq!(
        long_slug.raw_slug,
        "investigative-state-budget-scandal-2026"
    );
    assert_eq!(
        long_slug.display_text,
        "#investigative-state-budget-scandal-2026"
    );
    assert_eq!(long_slug.compact_text, "#investigative-st…");
}

#[test]
fn test_task_counter_progress_states() {
    // 1. Zero tasks configured
    let empty_counter = ArticleTaskCounterViewModel::new(0, 0, true);
    assert_eq!(empty_counter.completed, 0);
    assert_eq!(empty_counter.total, 0);
    assert!(!empty_counter.has_tasks);
    assert!(!empty_counter.all_completed);
    assert_eq!(empty_counter.formatted_label, "No tasks");
    assert_eq!(empty_counter.compact_label, "-");
    assert_eq!(empty_counter.progress_ratio, 0.0);
    assert_eq!(empty_counter.progress_percent, 0);

    // 2. Partial completion (e.g. 2 out of 5 tasks completed)
    let partial_counter = ArticleTaskCounterViewModel::new(2, 5, true);
    assert_eq!(partial_counter.completed, 2);
    assert_eq!(partial_counter.total, 5);
    assert!(partial_counter.has_tasks);
    assert!(!partial_counter.all_completed);
    assert_eq!(partial_counter.formatted_label, "2/5 tasks");
    assert_eq!(partial_counter.compact_label, "2/5");
    assert!((partial_counter.progress_ratio - 0.4).abs() < f32::EPSILON);
    assert_eq!(partial_counter.progress_percent, 40);

    // 3. 100% completion (e.g. 4 out of 4 tasks completed)
    let complete_counter = ArticleTaskCounterViewModel::new(4, 4, false);
    assert_eq!(complete_counter.completed, 4);
    assert_eq!(complete_counter.total, 4);
    assert!(complete_counter.has_tasks);
    assert!(complete_counter.all_completed);
    assert_eq!(complete_counter.formatted_label, "4/4 complete");
    assert_eq!(complete_counter.compact_label, "4/4");
    assert_eq!(complete_counter.progress_ratio, 1.0);
    assert_eq!(complete_counter.progress_percent, 100);
    assert_eq!(complete_counter.badge_fg_hex, SUCCESS_GREEN_HEX);
    assert_eq!(complete_counter.icon_name, "check-circle");
    assert_eq!(complete_counter.sf_symbol, "checkmark.circle.fill");
}

#[test]
fn test_contact_initials_and_tag_pills() {
    assert_eq!(format_contact_initials("Bob"), "Bo");
    assert_eq!(format_contact_initials("Bob Woodward"), "BW");
    assert_eq!(format_contact_initials("Carl Bernstein"), "CB");
    assert_eq!(format_contact_initials("Hunter S. Thompson"), "HT");
    assert_eq!(format_contact_initials("Cher"), "Ch");
    assert_eq!(format_contact_initials(""), "??");

    let contact = ContactBuilder::new("Sarah Koenig")
        .role("Host / Executive Producer")
        .organization("Serial Productions")
        .build();

    let pill = ArticleCardContactTagViewModel::from_contact(&contact);
    assert_eq!(pill.id, contact.id);
    assert_eq!(pill.name, "Sarah Koenig");
    assert_eq!(pill.initials, "SK");
    assert_eq!(pill.role, Some("Host / Executive Producer".to_string()));
    assert_eq!(pill.organization, Some("Serial Productions".to_string()));
    assert_eq!(pill.badge_label, "Sarah Koenig");
    assert!(!pill.avatar_color_hex.is_empty());
    assert!(!pill.avatar_contrast_hex.is_empty());
}

#[test]
fn test_overdue_visual_alert_state_and_border() {
    // Overdue state
    let overdue_style = ArticleCardOverdueStyleViewModel::new(true, false, true);
    assert!(overdue_style.is_overdue);
    assert_eq!(overdue_style.border_color_hex, OVERDUE_RED_HEX);
    assert_eq!(overdue_style.border_width_px, OVERDUE_CARD_BORDER_WIDTH);
    assert_eq!(overdue_style.badge_text, Some("OVERDUE".to_string()));
    assert!(overdue_style.glow_color_hex.is_some());

    // Due soon state
    let due_soon_style = ArticleCardOverdueStyleViewModel::new(false, true, true);
    assert!(!due_soon_style.is_overdue);
    assert_eq!(due_soon_style.border_color_hex, DUE_SOON_AMBER_HEX);
    assert_eq!(
        due_soon_style.border_width_px,
        DEFAULT_CARD_BORDER_WIDTH + 0.5
    );
    assert_eq!(due_soon_style.badge_text, Some("DUE SOON".to_string()));

    // Normal state
    let normal_style = ArticleCardOverdueStyleViewModel::new(false, false, true);
    assert!(!normal_style.is_overdue);
    assert_eq!(normal_style.border_width_px, DEFAULT_CARD_BORDER_WIDTH);
    assert_eq!(normal_style.badge_text, None);
    assert!(normal_style.glow_color_hex.is_none());
}

#[test]
fn test_deadline_badges_across_lifecycle() {
    let now = Utc::now();

    // 1. Overdue deadline
    let past = now - Duration::hours(5);
    let overdue_article = ArticleBuilder::new("overdue-story", "Overdue Story")
        .stage(ArticleStage::Writing)
        .deadline(past)
        .build();
    let overdue_status = evaluate_article_deadline(&overdue_article, now);
    let overdue_badge =
        format_deadline_badge(Some(past), ArticleStage::Writing, overdue_status, now, true)
            .expect("overdue badge exists");

    assert!(overdue_badge.is_overdue);
    assert_eq!(overdue_badge.badge_label, "OVERDUE");
    assert_eq!(overdue_badge.badge_fg_hex, OVERDUE_RED_HEX);
    assert_eq!(overdue_badge.icon_name, "alert-circle");
    assert_eq!(overdue_badge.sf_symbol, "exclamationmark.triangle.fill");

    // 2. Due soon deadline (18 hours away)
    let soon = now + Duration::hours(18);
    let soon_article = ArticleBuilder::new("soon-story", "Soon Story")
        .stage(ArticleStage::Editing)
        .deadline(soon)
        .build();
    let soon_status = evaluate_article_deadline(&soon_article, now);
    let soon_badge =
        format_deadline_badge(Some(soon), ArticleStage::Editing, soon_status, now, true)
            .expect("soon badge exists");

    assert!(!soon_badge.is_overdue);
    assert!(soon_badge.is_due_soon);
    assert_eq!(soon_badge.badge_label, "DUE SOON");
    assert_eq!(soon_badge.badge_fg_hex, DUE_SOON_AMBER_HEX);
    assert_eq!(soon_badge.icon_name, "clock");

    // 3. Normal on-track deadline (4 days away)
    let future = now + Duration::days(4);
    let future_article = ArticleBuilder::new("future-story", "Future Story")
        .stage(ArticleStage::Researching)
        .deadline(future)
        .build();
    let future_status = evaluate_article_deadline(&future_article, now);
    let future_badge = format_deadline_badge(
        Some(future),
        ArticleStage::Researching,
        future_status,
        now,
        true,
    )
    .expect("future badge exists");

    assert!(!future_badge.is_overdue);
    assert!(!future_badge.is_due_soon);
    assert_eq!(future_badge.icon_name, "calendar");

    // 4. Published article auto-clearing
    let pub_article = ArticleBuilder::new("pub-story", "Published Story")
        .stage(ArticleStage::Published)
        .deadline(past)
        .build();
    let pub_status = evaluate_article_deadline(&pub_article, now);
    let pub_badge =
        format_deadline_badge(Some(past), ArticleStage::Published, pub_status, now, true)
            .expect("published badge exists");

    assert!(!pub_badge.is_overdue);
    assert_eq!(pub_badge.badge_label, "Published");
    assert_eq!(pub_badge.relative_text, "Published");
}

#[test]
fn test_article_card_view_model_full_assembly_and_state_sync() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);
    let now = Utc::now();
    event_loop.state_mut().last_tick = now;

    // Create an article with custom color, past deadline (overdue), headline, and description
    let past_deadline = now - Duration::hours(4);
    let article = ArticleBuilder::new("harbor-spill", "Harbor Oil Spill Environmental Impact")
        .description("Investigation into maritime safety violations and environmental cleanup.")
        .stage(ArticleStage::Editing)
        .color("#8B5CF6")
        .deadline(past_deadline)
        .build();

    let article_id = article.id;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .expect("create article");

    // Create contacts and link to article
    let contact1 = Contact::new("Dr. Alan Grant");
    let contact2 = Contact::new("Ellie Sattler");
    let c1_id = contact1.id;
    let c2_id = contact2.id;

    event_loop
        .dispatch(AppMessage::CreateContact(contact1))
        .expect("create c1");
    event_loop
        .dispatch(AppMessage::CreateContact(contact2))
        .expect("create c2");
    event_loop
        .dispatch(AppMessage::SetArticleContacts {
            article_id,
            contact_ids: vec![c1_id, c2_id],
        })
        .expect("tag contacts");

    // Create 3 tasks: 2 complete, 1 in progress
    let mut t1 = Task::new(article_id, "Analyze water toxicity reports");
    t1.status = TaskStatus::Complete;
    let mut t2 = Task::new(article_id, "Interview coast guard spokesperson");
    t2.status = TaskStatus::Complete;
    let t3 = Task::new(article_id, "Review port authority surveillance footage");

    event_loop
        .dispatch(AppMessage::CreateTask(t1))
        .expect("create t1");
    event_loop
        .dispatch(AppMessage::CreateTask(t2))
        .expect("create t2");
    event_loop
        .dispatch(AppMessage::CreateTask(t3))
        .expect("create t3");

    // Build the ArticleCardViewModel
    let current_article = event_loop
        .state()
        .get_article(article_id)
        .expect("article exists");
    let card = ArticleCardViewModel::build(current_article, event_loop.state());

    // Verify all properties of the card view model
    assert_eq!(card.id, article_id);
    assert_eq!(card.slug, "harbor-spill");
    assert_eq!(card.slug_badge.display_text, "#harbor-spill");
    assert_eq!(card.headline, "Harbor Oil Spill Environmental Impact");
    assert!(card.description_snippet.is_some());
    assert_eq!(card.stage, ArticleStage::Editing);
    assert_eq!(card.color_hex, "#8B5CF6");
    assert_eq!(
        card.color_indicator.position,
        IndicatorPosition::LeftAccentStrip
    );

    // Verify task completion stats
    assert_eq!(card.task_completed, 2);
    assert_eq!(card.task_total, 3);
    assert_eq!(card.task_counter.formatted_label, "2/3 tasks");
    assert_eq!(card.task_counter.compact_label, "2/3");
    assert_eq!(card.task_counter.progress_percent, 67);
    assert!(!card.task_counter.all_completed);

    // Verify tagged contacts
    assert_eq!(card.tagged_contact_count, 2);
    assert_eq!(card.tagged_contacts.len(), 2);
    assert_eq!(card.tagged_contacts[0].name, "Dr. Alan Grant");
    assert_eq!(card.tagged_contacts[0].initials, "DG");
    assert_eq!(card.tagged_contacts[1].name, "Ellie Sattler");
    assert_eq!(card.tagged_contacts[1].initials, "ES");

    // Verify overdue status & bold visual styling
    assert!(card.is_overdue);
    assert_eq!(card.overdue_style.border_color_hex, OVERDUE_RED_HEX);
    assert_eq!(
        card.overdue_style.border_width_px,
        OVERDUE_CARD_BORDER_WIDTH
    );
    assert_eq!(card.overdue_style.badge_text, Some("OVERDUE".to_string()));
    assert!(card.deadline_badge.is_some());
    let badge = card.deadline_badge.as_ref().unwrap();
    assert!(badge.is_overdue);
    assert_eq!(badge.badge_label, "OVERDUE");
}

#[test]
fn test_cosmic_and_macos_app_view_tree_card_integration() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);
    let now = Utc::now();
    event_loop.state_mut().last_tick = now;

    let overdue_article = ArticleBuilder::new("city-election", "City Council Special Election")
        .stage(ArticleStage::Writing)
        .deadline(now - Duration::minutes(30))
        .build();

    let on_track_article = ArticleBuilder::new("tech-summit", "Annual Tech Summit Keynote")
        .stage(ArticleStage::Pitching)
        .deadline(now + Duration::days(7))
        .build();

    event_loop
        .dispatch(AppMessage::CreateArticle(overdue_article))
        .expect("create article 1");
    event_loop
        .dispatch(AppMessage::CreateArticle(on_track_article))
        .expect("create article 2");

    let final_state = event_loop.state().clone();

    // Test Linux COSMIC App integration
    let cosmic_app = CosmicApp::new(final_state.clone());
    let cosmic_tree = cosmic_app.build_view_tree();
    assert!(cosmic_tree.articles_deck.is_some());
    let cosmic_deck = cosmic_tree.articles_deck.unwrap();

    let writing_col = cosmic_deck
        .column(ArticleStage::Writing)
        .expect("writing col");
    assert_eq!(writing_col.card_count, 1);
    let writing_card = &writing_col.cards[0];
    assert!(writing_card.is_overdue);
    assert_eq!(writing_card.overdue_style.border_color_hex, OVERDUE_RED_HEX);

    let pitching_col = cosmic_deck
        .column(ArticleStage::Pitching)
        .expect("pitching col");
    assert_eq!(pitching_col.card_count, 1);
    let pitching_card = &pitching_col.cards[0];
    assert!(!pitching_card.is_overdue);

    // Test macOS App integration
    let macos_app = MacosApp::new(final_state);
    let macos_tree = macos_app.build_view_tree();
    assert!(macos_tree.articles_deck.is_some());
    let macos_deck = macos_tree.articles_deck.unwrap();

    let macos_writing_col = macos_deck
        .column(ArticleStage::Writing)
        .expect("writing col");
    assert_eq!(macos_writing_col.card_count, 1);
    assert!(macos_writing_col.cards[0].is_overdue);
}
