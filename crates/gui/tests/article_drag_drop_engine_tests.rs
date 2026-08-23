//! Comprehensive integration test suite for Task 5.3: Articles Kanban Drag-and-Drop Movement Engine.

use chrono::{Duration, Utc};
use newsjournal_core::models::{Article, Contact, Task};
use newsjournal_core::ArticleStage;
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::platform::cosmic::CosmicApp;
use newsjournal_gui::platform::macos::MacosApp;
use newsjournal_gui::state::drag_drop::{DragItem, DropTarget};
use newsjournal_gui::views::{build_articles_kanban_deck_with_layout, DeckLayoutConfig};
use newsjournal_gui::{AppState, EventLoop};

#[test]
fn test_pointer_click_and_drag_mechanics_and_threshold() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let article = Article::new("transit-expansion", "City Expands Light Rail Corridor");
    let article_id = article.id;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    // 1. Initial idle state
    assert!(!event_loop.state().drag.is_dragging());
    assert_eq!(event_loop.state().drag.pointer_pos, None);

    // 2. Start drag with pointer coordinates (x: 120.0, y: 300.0)
    event_loop
        .dispatch(AppMessage::DragStartWithPos {
            item: DragItem::ArticleCard {
                id: article_id,
                origin_stage: ArticleStage::Pitching,
            },
            pos: (120.0, 300.0),
        })
        .unwrap();

    assert!(event_loop.state().drag.is_dragging());
    assert_eq!(event_loop.state().drag.drag_start_pos, Some((120.0, 300.0)));
    assert_eq!(event_loop.state().drag.pointer_pos, Some((120.0, 300.0)));
    assert_eq!(event_loop.state().drag.drag_delta(), (0.0, 0.0));
    assert_eq!(event_loop.state().drag.drag_distance(), 0.0);
    assert!(!event_loop.state().drag.has_exceeded_drag_threshold(6.0));

    // 3. Move pointer slightly (below threshold)
    event_loop
        .dispatch(AppMessage::DragMove {
            pointer_pos: (123.0, 304.0),
        })
        .unwrap();
    assert_eq!(event_loop.state().drag.drag_delta(), (3.0, 4.0));
    assert_eq!(event_loop.state().drag.drag_distance(), 5.0);
    assert!(!event_loop.state().drag.has_exceeded_drag_threshold(6.0));
    assert!(event_loop.state().drag.has_exceeded_drag_threshold(4.0));

    // 4. Move pointer across columns (large movement)
    event_loop
        .dispatch(AppMessage::DragMove {
            pointer_pos: (450.0, 320.0),
        })
        .unwrap();
    assert!(event_loop.state().drag.has_exceeded_drag_threshold(10.0));
    assert_eq!(event_loop.state().drag.drag_delta(), (330.0, 20.0));
}

#[test]
fn test_screen_hit_testing_and_scroll_offset_geometry() {
    let layout = DeckLayoutConfig::default()
        .with_column_width(300.0)
        .with_scroll(0.0);

    // Padding = 20.0, Column width = 300.0, Gap = 16.0
    // Col 0 (Pitching): [20.0, 320.0]
    // Gap: [320.0, 336.0]
    // Col 1 (Researching): [336.0, 636.0]
    // Col 2 (Writing): [652.0, 952.0]
    // Col 3 (Editing): [968.0, 1268.0]
    // Col 4 (ReadyToPublish): [1284.0, 1584.0]
    // Col 5 (Published): [1600.0, 1900.0]

    assert_eq!(layout.column_index_at_screen_x(100.0), Some(0));
    assert_eq!(
        layout.stage_at_screen_x(100.0),
        Some(ArticleStage::Pitching)
    );

    assert_eq!(layout.column_index_at_screen_x(400.0), Some(1));
    assert_eq!(
        layout.stage_at_screen_x(400.0),
        Some(ArticleStage::Researching)
    );

    assert_eq!(layout.column_index_at_screen_x(700.0), Some(2));
    assert_eq!(layout.stage_at_screen_x(700.0), Some(ArticleStage::Writing));

    // Gap between col 0 and 1 -> returns None
    assert_eq!(layout.column_index_at_screen_x(328.0), None);

    // Test with horizontal scroll offset = 336.0 (scrolled 1 column right)
    let scrolled_layout = layout.with_scroll(336.0);
    // Col 1 (Researching) now starts at screen X: 0.0 -> [0.0, 300.0]
    assert_eq!(scrolled_layout.column_index_at_screen_x(100.0), Some(1));
    assert_eq!(
        scrolled_layout.stage_at_screen_x(100.0),
        Some(ArticleStage::Researching)
    );
}

#[test]
fn test_visual_drag_ghost_and_source_card_dimming() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let now = Utc::now();
    let mut article = Article::new("clean-energy", "Wind Power Grid Integration")
        .with_description("In-depth investigative report on wind farm grid integration.")
        .with_deadline(now + Duration::hours(12));
    article.color = Some("#3B82F6".to_string());
    let article_id = article.id;

    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    let task = Task::new(article_id, "Interview grid engineers");
    event_loop.dispatch(AppMessage::CreateTask(task)).unwrap();

    let contact = Contact::new("Aris Thorne")
        .with_organization("State Energy Commission")
        .with_role("Chief Engineer");
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

    // 1. Build idle deck view model
    let idle_deck =
        build_articles_kanban_deck_with_layout(event_loop.state(), DeckLayoutConfig::default());
    assert!(!idle_deck.is_dragging);
    assert!(idle_deck.drag_ghost.is_none());

    let idle_card = &idle_deck.columns[0].cards[0];
    assert!(!idle_card.is_dragging);
    assert_eq!(idle_card.drag_opacity, 1.0);

    // 2. Start drag with pointer coordinates (x: 180.0, y: 220.0)
    event_loop
        .dispatch(AppMessage::DragStartWithPos {
            item: DragItem::ArticleCard {
                id: article_id,
                origin_stage: ArticleStage::Pitching,
            },
            pos: (180.0, 220.0),
        })
        .unwrap();

    let dragging_deck =
        build_articles_kanban_deck_with_layout(event_loop.state(), DeckLayoutConfig::default());
    assert!(dragging_deck.is_dragging);
    assert!(dragging_deck.drag_ghost.is_some());

    // Verify source card in Pitching column is dimmed
    let source_card = &dragging_deck.columns[0].cards[0];
    assert!(source_card.is_dragging);
    assert_eq!(source_card.drag_opacity, 0.35);

    // Verify Drag Ghost view model properties
    let ghost = dragging_deck.drag_ghost.as_ref().unwrap();
    assert_eq!(ghost.id, article_id);
    assert_eq!(ghost.slug, "clean-energy");
    assert_eq!(ghost.display_slug, "#clean-energy");
    assert_eq!(ghost.headline, "Wind Power Grid Integration");
    assert_eq!(ghost.origin_stage, ArticleStage::Pitching);
    assert_eq!(ghost.color_hex, "#3B82F6");
    assert_eq!(ghost.pointer_pos, (180.0, 220.0));
    assert_eq!(ghost.tilt_degrees, 2.5);
    assert_eq!(ghost.scale, 1.03);
    assert_eq!(ghost.opacity, 0.92);
    assert_eq!(ghost.shadow_blur_px, 24.0);
    assert_eq!(ghost.badge_label, "DRAGGING STORY");
    assert!(ghost.deadline_badge.is_some());
    assert_eq!(ghost.task_counter_label, Some("0/1 tasks".to_string()));
    assert_eq!(ghost.tagged_contact_count, 1);
    assert_eq!(ghost.tagged_contact_initials, vec!["AT".to_string()]);

    // 3. Hover over valid target column (Writing)
    event_loop
        .dispatch(AppMessage::DragHover(Some(DropTarget::ArticleColumn(
            ArticleStage::Writing,
        ))))
        .unwrap();

    let hover_deck =
        build_articles_kanban_deck_with_layout(event_loop.state(), DeckLayoutConfig::default());
    let updated_ghost = hover_deck.drag_ghost.unwrap();
    assert!(updated_ghost.is_valid_target);
    assert_eq!(updated_ghost.badge_label, "MOVE TO WRITING");
    assert_eq!(updated_ghost.border_color_hex, "#10B981"); // Green valid drop border
}

#[test]
fn test_visual_drop_placeholders_and_column_highlights() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let article = Article::new("water-rights", "Western River Water Rights Dispute");
    let article_id = article.id;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    // Start drag from Pitching
    event_loop
        .dispatch(AppMessage::DragStartWithPos {
            item: DragItem::ArticleCard {
                id: article_id,
                origin_stage: ArticleStage::Pitching,
            },
            pos: (200.0, 200.0),
        })
        .unwrap();

    // Hover over Editing column with insertion index 0
    event_loop
        .dispatch(AppMessage::DragHoverWithIndex {
            target: Some(DropTarget::ArticleColumn(ArticleStage::Editing)),
            insert_index: Some(0),
        })
        .unwrap();

    let deck =
        build_articles_kanban_deck_with_layout(event_loop.state(), DeckLayoutConfig::default());

    // Col 0 (Pitching - origin): not valid target
    let col_pitching = &deck.columns[0];
    assert!(!col_pitching.is_valid_drop_target);
    assert!(!col_pitching.is_active_drop_target);
    assert!(col_pitching.drop_placeholder.is_none());

    // Col 3 (Editing - hovered target): active drop target
    let col_editing = &deck.columns[3];
    assert!(col_editing.is_hovered);
    assert!(col_editing.is_valid_drop_target);
    assert!(col_editing.is_active_drop_target);
    assert_eq!(
        col_editing.drop_highlight_border_hex,
        Some("#10B981".to_string())
    );
    assert!(col_editing.drop_placeholder.is_some());

    let placeholder = col_editing.drop_placeholder.as_ref().unwrap();
    assert_eq!(placeholder.target_stage, ArticleStage::Editing);
    assert_eq!(placeholder.insert_index, 0);
    assert_eq!(placeholder.border_style, "dashed");
    assert!(placeholder.is_valid);
    assert!(placeholder.is_pulse_active);
    assert_eq!(placeholder.prompt_text, "+ Move '#water-rights' to Editing");

    // Hover over same column (Pitching - invalid target)
    event_loop
        .dispatch(AppMessage::DragHover(Some(DropTarget::ArticleColumn(
            ArticleStage::Pitching,
        ))))
        .unwrap();

    let invalid_hover_deck =
        build_articles_kanban_deck_with_layout(event_loop.state(), DeckLayoutConfig::default());
    let col_pitching_hover = &invalid_hover_deck.columns[0];
    assert!(col_pitching_hover.is_hovered);
    assert!(!col_pitching_hover.is_valid_drop_target);
    assert!(!col_pitching_hover.is_active_drop_target);
    assert_eq!(
        col_pitching_hover.drop_highlight_border_hex,
        Some("#EF4444".to_string())
    ); // Red invalid target border

    let invalid_placeholder = col_pitching_hover.drop_placeholder.as_ref().unwrap();
    assert!(!invalid_placeholder.is_valid);
    assert_eq!(
        invalid_placeholder.prompt_text,
        "Cannot drop in current stage"
    );
}

#[test]
fn test_stage_change_dispatch_and_sqlite_persistence() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let article = Article::new("space-telescope", "New Telescope Unveils Early Galaxies");
    let article_id = article.id;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();
    assert_eq!(event_loop.state().articles[0].stage, ArticleStage::Pitching);

    let workflow_stages = [
        ArticleStage::Researching,
        ArticleStage::Writing,
        ArticleStage::Editing,
        ArticleStage::ReadyToPublish,
        ArticleStage::Published,
    ];

    let mut current_stage = ArticleStage::Pitching;

    for &next_stage in &workflow_stages {
        // 1. Start drag from current stage
        event_loop
            .dispatch(AppMessage::DragStartWithPos {
                item: DragItem::ArticleCard {
                    id: article_id,
                    origin_stage: current_stage,
                },
                pos: (150.0, 200.0),
            })
            .unwrap();
        assert!(event_loop.state().drag.is_dragging());

        // 2. Hover next stage
        event_loop
            .dispatch(AppMessage::DragHover(Some(DropTarget::ArticleColumn(
                next_stage,
            ))))
            .unwrap();
        assert!(event_loop.state().drag.is_valid_drop());

        // 3. Drop
        event_loop.dispatch(AppMessage::DragDrop).unwrap();
        assert!(!event_loop.state().drag.is_dragging());
        assert_eq!(event_loop.state().articles[0].stage, next_stage);

        // 4. Verify SQLite storage persistence
        let from_storage = event_loop
            .state()
            .storage
            .get_article(article_id)
            .unwrap()
            .expect("article exists in SQLite");
        assert_eq!(from_storage.stage, next_stage);

        current_stage = next_stage;
    }

    // Verify toast notification was queued for last transition
    assert!(!event_loop.state().toasts.is_empty());
    let latest_toast = event_loop.state().toasts.last().unwrap();
    assert_eq!(latest_toast.title, "Article Moved");
    assert!(latest_toast.body.contains("Published"));
}

#[test]
fn test_overdue_recalculation_upon_drop_into_published() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let now = Utc::now();
    // Article deadline 3 hours in the past -> overdue when in Editing
    let article = Article::new("overdue-investigation", "Mayoral Election Audit")
        .with_stage(ArticleStage::Editing)
        .with_deadline(now - Duration::hours(3));
    let article_id = article.id;

    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    let pre_drop_deck =
        build_articles_kanban_deck_with_layout(event_loop.state(), DeckLayoutConfig::default());
    assert_eq!(pre_drop_deck.total_overdue_count, 1);
    assert!(pre_drop_deck.columns[3].cards[0].is_overdue);

    // Drag to Published
    event_loop
        .dispatch(AppMessage::DragStartWithPos {
            item: DragItem::ArticleCard {
                id: article_id,
                origin_stage: ArticleStage::Editing,
            },
            pos: (800.0, 250.0),
        })
        .unwrap();
    event_loop
        .dispatch(AppMessage::DragHover(Some(DropTarget::ArticleColumn(
            ArticleStage::Published,
        ))))
        .unwrap();
    event_loop.dispatch(AppMessage::DragDrop).unwrap();

    // After dropping into Published, overdue status is cleared per domain rules
    let post_drop_deck =
        build_articles_kanban_deck_with_layout(event_loop.state(), DeckLayoutConfig::default());
    assert_eq!(post_drop_deck.total_overdue_count, 0);
    assert_eq!(post_drop_deck.columns[5].card_count, 1);
    assert!(!post_drop_deck.columns[5].cards[0].is_overdue);
}

#[test]
fn test_drag_cancellation_and_clean_state_reset() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let article = Article::new("crypto-regulations", "New Crypto Asset Reporting Rules");
    let article_id = article.id;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    // Start drag
    event_loop
        .dispatch(AppMessage::DragStartWithPos {
            item: DragItem::ArticleCard {
                id: article_id,
                origin_stage: ArticleStage::Pitching,
            },
            pos: (150.0, 200.0),
        })
        .unwrap();
    event_loop
        .dispatch(AppMessage::DragMove {
            pointer_pos: (250.0, 350.0),
        })
        .unwrap();
    event_loop
        .dispatch(AppMessage::DragHover(Some(DropTarget::ArticleColumn(
            ArticleStage::Writing,
        ))))
        .unwrap();

    let active_deck =
        build_articles_kanban_deck_with_layout(event_loop.state(), DeckLayoutConfig::default());
    assert!(active_deck.is_dragging);
    assert!(active_deck.drag_ghost.is_some());
    assert_eq!(active_deck.columns[0].cards[0].drag_opacity, 0.35);

    // Cancel drag explicitly
    event_loop.dispatch(AppMessage::DragCancel).unwrap();

    let cancelled_deck =
        build_articles_kanban_deck_with_layout(event_loop.state(), DeckLayoutConfig::default());
    assert!(!cancelled_deck.is_dragging);
    assert!(cancelled_deck.drag_ghost.is_none());
    assert_eq!(cancelled_deck.columns[0].cards[0].drag_opacity, 1.0);
    assert_eq!(
        cancelled_deck.columns[0].cards[0].stage,
        ArticleStage::Pitching
    );
    assert_eq!(event_loop.state().drag.drag_start_pos, None);
    assert_eq!(event_loop.state().drag.pointer_pos, None);
    assert_eq!(event_loop.state().drag.hover_target, None);
}

#[test]
fn test_cosmic_and_macos_app_view_tree_drag_and_drop_integration() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut cosmic_app = CosmicApp::new(state.clone());
    let mut macos_app = MacosApp::new(state);

    let article = Article::new("ai-breakthrough", "Neural Network Synthesis Breakthrough");
    let article_id = article.id;

    cosmic_app
        .dispatch(AppMessage::CreateArticle(article.clone()))
        .unwrap();
    macos_app
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    // 1. Verify idle view tree
    let cosmic_tree = cosmic_app.build_view_tree();
    let cosmic_deck = cosmic_tree.articles_deck.expect("articles deck exists");
    assert!(!cosmic_deck.is_dragging);
    assert!(cosmic_deck.drag_ghost.is_none());

    let macos_tree = macos_app.build_view_tree();
    let macos_deck = macos_tree.articles_deck.expect("articles deck exists");
    assert!(!macos_deck.is_dragging);
    assert!(macos_deck.drag_ghost.is_none());

    // 2. Start drag on COSMIC app
    cosmic_app
        .dispatch(AppMessage::DragStartWithPos {
            item: DragItem::ArticleCard {
                id: article_id,
                origin_stage: ArticleStage::Pitching,
            },
            pos: (200.0, 300.0),
        })
        .unwrap();
    cosmic_app
        .dispatch(AppMessage::DragHover(Some(DropTarget::ArticleColumn(
            ArticleStage::Researching,
        ))))
        .unwrap();

    let cosmic_active_tree = cosmic_app.build_view_tree();
    let cosmic_active_deck = cosmic_active_tree.articles_deck.unwrap();
    assert!(cosmic_active_deck.is_dragging);
    assert!(cosmic_active_deck.drag_ghost.is_some());
    assert_eq!(cosmic_active_deck.columns[0].cards[0].drag_opacity, 0.35);
    assert!(cosmic_active_deck.columns[1].is_active_drop_target);

    // 3. Drop on COSMIC app
    cosmic_app.dispatch(AppMessage::DragDrop).unwrap();
    let cosmic_dropped_tree = cosmic_app.build_view_tree();
    let cosmic_dropped_deck = cosmic_dropped_tree.articles_deck.unwrap();
    assert!(!cosmic_dropped_deck.is_dragging);
    assert_eq!(cosmic_dropped_deck.columns[1].card_count, 1);
    assert_eq!(
        cosmic_dropped_deck.columns[1].cards[0].stage,
        ArticleStage::Researching
    );

    // 4. Start drag on macOS app
    macos_app
        .dispatch(AppMessage::DragStartWithPos {
            item: DragItem::ArticleCard {
                id: article_id,
                origin_stage: ArticleStage::Pitching,
            },
            pos: (350.0, 400.0),
        })
        .unwrap();
    macos_app
        .dispatch(AppMessage::DragHover(Some(DropTarget::ArticleColumn(
            ArticleStage::ReadyToPublish,
        ))))
        .unwrap();

    let macos_active_tree = macos_app.build_view_tree();
    let macos_active_deck = macos_active_tree.articles_deck.unwrap();
    assert!(macos_active_deck.is_dragging);
    assert!(macos_active_deck.drag_ghost.is_some());
    assert!(macos_active_deck.columns[4].is_active_drop_target);

    // 5. Drop on macOS app
    macos_app.dispatch(AppMessage::DragDrop).unwrap();
    let macos_dropped_tree = macos_app.build_view_tree();
    let macos_dropped_deck = macos_dropped_tree.articles_deck.unwrap();
    assert!(!macos_dropped_deck.is_dragging);
    assert_eq!(macos_dropped_deck.columns[4].card_count, 1);
    assert_eq!(
        macos_dropped_deck.columns[4].cards[0].stage,
        ArticleStage::ReadyToPublish
    );
}
