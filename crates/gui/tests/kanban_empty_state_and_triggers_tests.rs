//! Comprehensive test suite for Task 5.4: Empty State & Creation Triggers.
//!
//! Tests:
//! 1. Column headers with card counts, count badges, overdue alerts, due-soon badges, and quick-add buttons.
//! 2. Stage-tailored column empty states with icons, prompts, action buttons, and drop guidance hints.
//! 3. Distinction between natural column empty states and filtered column empty states.
//! 4. Deck-wide first-time user empty state with welcome prompt and "+ Create Your First Article" trigger.
//! 5. Deck-wide filtered empty state with "No Matching Articles" and "Clear Filters" action.
//! 6. Kanban deck top toolbar view model with metrics, overdue alert pills, and "+ New Article" trigger.
//! 7. Stage-specific creation trigger dispatch (`AppMessage::OpenNewArticleInStageModal`) pre-populating stage.
//! 8. COSMIC and macOS view tree assembly with column headers, empty states, and toolbar models.

use chrono::{Duration, Utc};
use newsjournal_core::models::{Article, ArticleStage};
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::navigation::NavTab;
use newsjournal_gui::platform::cosmic::app::CosmicApp;
use newsjournal_gui::platform::cosmic::header_bar::CosmicHeaderBar;
use newsjournal_gui::platform::macos::app::MacosApp;
use newsjournal_gui::platform::macos::toolbar::MacosToolbar;
use newsjournal_gui::state::modal::ModalState;
use newsjournal_gui::views::{
    build_articles_kanban_deck, stage_metadata, ArticleColumnHeaderViewModel,
    ColumnEmptyStateViewModel, KanbanToolbarViewModel, NUM_ARTICLE_STAGES,
};
use newsjournal_gui::{AppState, EventLoop};

#[test]
fn test_column_headers_card_counts_and_overdue_badges() {
    let stages = ArticleStage::all();
    assert_eq!(stages.len(), NUM_ARTICLE_STAGES);

    // Test header formatting for 0 cards
    for &stage in stages {
        let meta = stage_metadata(stage);
        let header = ArticleColumnHeaderViewModel::build(stage, 0, 0, 0);

        assert_eq!(header.stage, stage);
        assert_eq!(header.index, meta.index);
        assert_eq!(header.title, meta.title);
        assert_eq!(header.description, meta.description);
        assert_eq!(header.icon_emoji, meta.icon_emoji);
        assert_eq!(header.icon_name, meta.icon_name);
        assert_eq!(header.sf_symbol, meta.sf_symbol);
        assert_eq!(header.accent_hex, meta.accent_hex);

        assert_eq!(header.card_count, 0);
        assert_eq!(header.card_count_label, "0 stories");
        assert_eq!(header.count_badge_text, "0");
        assert_eq!(header.overdue_count, 0);
        assert_eq!(header.overdue_badge_label, None);
        assert_eq!(header.due_soon_count, 0);
        assert_eq!(header.due_soon_badge_label, None);
        assert!(!header.has_overdue());
        assert!(!header.has_due_soon());

        assert_eq!(
            header.quick_add_action,
            AppMessage::OpenNewArticleInStageModal(stage)
        );
        assert!(header.quick_add_tooltip.contains(meta.title));
    }

    // Test quick add button labels per stage
    assert_eq!(
        ArticleColumnHeaderViewModel::quick_add_label_for_stage(ArticleStage::Pitching),
        "+ Add Pitch"
    );
    assert_eq!(
        ArticleColumnHeaderViewModel::quick_add_label_for_stage(ArticleStage::Researching),
        "+ Add Research"
    );
    assert_eq!(
        ArticleColumnHeaderViewModel::quick_add_label_for_stage(ArticleStage::Writing),
        "+ Add Draft"
    );
    assert_eq!(
        ArticleColumnHeaderViewModel::quick_add_label_for_stage(ArticleStage::Editing),
        "+ Add Review"
    );
    assert_eq!(
        ArticleColumnHeaderViewModel::quick_add_label_for_stage(ArticleStage::ReadyToPublish),
        "+ Add Story"
    );
    assert_eq!(
        ArticleColumnHeaderViewModel::quick_add_label_for_stage(ArticleStage::Published),
        "+ Archive"
    );

    // Test header formatting for 1 card (singular)
    let header_1 = ArticleColumnHeaderViewModel::build(ArticleStage::Writing, 1, 0, 1);
    assert_eq!(header_1.card_count_label, "1 story");
    assert_eq!(header_1.count_badge_text, "1");
    assert!(!header_1.has_overdue());
    assert!(header_1.has_due_soon());
    assert_eq!(
        header_1.due_soon_badge_label,
        Some("🕒 1 Due Soon".to_string())
    );

    // Test header formatting for 5 cards with 2 overdue
    let header_5 = ArticleColumnHeaderViewModel::build(ArticleStage::Editing, 5, 2, 0);
    assert_eq!(header_5.card_count_label, "5 stories");
    assert_eq!(header_5.count_badge_text, "5");
    assert!(header_5.has_overdue());
    assert!(!header_5.has_due_soon());
    assert_eq!(
        header_5.overdue_badge_label,
        Some("⚠️ 2 Overdue".to_string())
    );
}

#[test]
fn test_stage_specific_column_empty_states() {
    let expected_empty_state_data = [
        (
            ArticleStage::Pitching,
            "No Story Pitches",
            "+ New Pitch",
            "Pitches can also be dragged from other stages",
        ),
        (
            ArticleStage::Researching,
            "No Articles in Research",
            "+ Add Research",
            "Drag approved pitches here to begin research",
        ),
        (
            ArticleStage::Writing,
            "No Drafts in Progress",
            "+ Start Draft",
            "Drag researched stories here to start drafting",
        ),
        (
            ArticleStage::Editing,
            "No Articles in Review",
            "+ Add to Review",
            "Drag completed drafts here for review & fact-checking",
        ),
        (
            ArticleStage::ReadyToPublish,
            "No Stories Queued",
            "+ Queue Story",
            "Drag approved stories here for publication sign-off",
        ),
        (
            ArticleStage::Published,
            "No Published Stories",
            "+ Archive Story",
            "Drag live stories here to archive them",
        ),
    ];

    for (stage, title, action_label, drop_hint) in expected_empty_state_data {
        let meta = stage_metadata(stage);
        let empty_state = ColumnEmptyStateViewModel::build(stage, false, "");

        assert_eq!(empty_state.stage, stage);
        assert_eq!(empty_state.title, title);
        assert_eq!(empty_state.prompt, meta.empty_state_prompt);
        assert_eq!(empty_state.action_button_label, action_label);
        assert_eq!(
            empty_state.action_message,
            Some(AppMessage::OpenNewArticleInStageModal(stage))
        );
        assert!(empty_state.is_drop_target_hint);
        assert_eq!(empty_state.drop_hint_text, drop_hint);
        assert!(!empty_state.is_filtered_empty);
        assert_eq!(empty_state.filtered_explanation, None);
        assert_eq!(empty_state.icon_emoji, meta.icon_emoji);
        assert_eq!(empty_state.icon_name, meta.icon_name);
        assert_eq!(empty_state.sf_symbol, meta.sf_symbol);
        assert_eq!(empty_state.accent_hex, meta.accent_hex);
    }
}

#[test]
fn test_filtered_empty_state_vs_natural_empty_state() {
    // 1. Natural empty state (no filter)
    let natural = ColumnEmptyStateViewModel::build(ArticleStage::Writing, false, "");
    assert!(!natural.is_filtered_empty);
    assert_eq!(natural.filtered_explanation, None);

    // 2. Filtered empty state with search query
    let filtered_search = ColumnEmptyStateViewModel::build(ArticleStage::Writing, true, "budget");
    assert!(filtered_search.is_filtered_empty);
    assert_eq!(
        filtered_search.filtered_explanation,
        Some("No stories in Writing match \"budget\"".to_string())
    );

    // 3. Filtered empty state without text query (e.g. urgency filter only)
    let filtered_no_text = ColumnEmptyStateViewModel::build(ArticleStage::Editing, true, "   ");
    assert!(filtered_no_text.is_filtered_empty);
    assert_eq!(
        filtered_no_text.filtered_explanation,
        Some("No stories in Editing match active filters".to_string())
    );
}

#[test]
fn test_deck_level_global_empty_state_and_first_time_action() {
    let state = AppState::in_memory().expect("in-memory state");
    let deck = build_articles_kanban_deck(&state);

    assert_eq!(deck.total_article_count, 0);
    assert!(deck.deck_empty_state.is_some());

    let empty = deck.deck_empty_state.as_ref().unwrap();
    assert_eq!(empty.headline, "Welcome to Articles Kanban");
    assert!(empty.subtext.contains("newsroom reporting pipeline"));
    assert_eq!(empty.action_button_label, "+ Create Your First Article");
    assert_eq!(empty.action_message, AppMessage::OpenNewArticleModal);
    assert_eq!(empty.secondary_action_label, None);
    assert_eq!(empty.secondary_action_message, None);
    assert!(!empty.is_filtered);
    assert_eq!(empty.icon_emoji, "📰");
    assert_eq!(empty.icon_name, "newspaper");
    assert_eq!(empty.sf_symbol, "doc.richtext");

    // All 6 columns should also have their column-level empty states populated
    for col in &deck.columns {
        assert!(col.is_empty);
        assert!(col.empty_state.is_some());
        assert_eq!(col.header.card_count, 0);
    }
}

#[test]
fn test_deck_level_filtered_empty_state_and_clear_filters() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    // Add 1 article
    let mut article = Article::new("city-election", "City Council Election Preview");
    article.stage = ArticleStage::Writing;
    event_loop
        .dispatch(AppMessage::CreateArticle(article))
        .unwrap();

    // Verify deck is NOT empty initially
    let deck_populated = build_articles_kanban_deck(event_loop.state());
    assert_eq!(deck_populated.total_article_count, 1);
    assert!(deck_populated.deck_empty_state.is_none());

    // Writing column is not empty; other 5 columns are empty
    let writing_col = deck_populated.column(ArticleStage::Writing).unwrap();
    assert!(!writing_col.is_empty);
    assert!(writing_col.empty_state.is_none());
    assert_eq!(writing_col.header.card_count, 1);

    let pitching_col = deck_populated.column(ArticleStage::Pitching).unwrap();
    assert!(pitching_col.is_empty);
    assert!(pitching_col.empty_state.is_some());

    // Apply search filter that yields 0 matches
    event_loop
        .dispatch(AppMessage::SetSearchQuery("nonexistent-topic".to_string()))
        .unwrap();

    let deck_filtered = build_articles_kanban_deck(event_loop.state());
    assert_eq!(deck_filtered.total_article_count, 0);
    assert!(deck_filtered.is_filtered);
    assert!(deck_filtered.deck_empty_state.is_some());

    let filter_empty = deck_filtered.deck_empty_state.as_ref().unwrap();
    assert_eq!(filter_empty.headline, "No Matching Articles");
    assert!(filter_empty.subtext.contains("nonexistent-topic"));
    assert_eq!(filter_empty.action_button_label, "Clear Filters");
    assert_eq!(filter_empty.action_message, AppMessage::ClearFilters);
    assert_eq!(
        filter_empty.secondary_action_label,
        Some("+ New Article".to_string())
    );
    assert_eq!(
        filter_empty.secondary_action_message,
        Some(AppMessage::OpenNewArticleModal)
    );
    assert!(filter_empty.is_filtered);
    assert_eq!(filter_empty.icon_emoji, "🔍");

    // Clear filters and verify deck recovers
    event_loop.dispatch(AppMessage::ClearFilters).unwrap();
    let deck_recovered = build_articles_kanban_deck(event_loop.state());
    assert_eq!(deck_recovered.total_article_count, 1);
    assert!(deck_recovered.deck_empty_state.is_none());
}

#[test]
fn test_kanban_toolbar_metrics_and_quick_action_trigger() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);
    let now = Utc::now();

    // Toolbar on empty state
    let deck_empty = build_articles_kanban_deck(event_loop.state());
    let toolbar = &deck_empty.toolbar;
    assert_eq!(toolbar.title, "Articles Kanban");
    assert_eq!(toolbar.total_article_count, 0);
    assert_eq!(toolbar.total_count_label, "0 stories");
    assert_eq!(toolbar.total_overdue_count, 0);
    assert_eq!(toolbar.overdue_badge_label, None);
    assert_eq!(toolbar.total_due_soon_count, 0);
    assert_eq!(toolbar.due_soon_badge_label, None);
    assert_eq!(toolbar.primary_action_label, "+ New Article");
    assert_eq!(toolbar.primary_action_shortcut, "⌘N");
    assert_eq!(
        toolbar.primary_action_message,
        AppMessage::OpenNewArticleModal
    );
    assert!(!toolbar.has_overdue());
    assert!(!toolbar.has_due_soon());

    // Add 1 healthy article and 1 overdue article
    let mut a1 = Article::new("investigation-one", "Investigation One");
    a1.stage = ArticleStage::Pitching;
    a1.deadline = Some(now + Duration::days(10));
    event_loop.dispatch(AppMessage::CreateArticle(a1)).unwrap();

    let mut a2 = Article::new("investigation-two", "Investigation Two");
    a2.stage = ArticleStage::Writing;
    a2.deadline = Some(now - Duration::hours(5)); // Overdue
    event_loop.dispatch(AppMessage::CreateArticle(a2)).unwrap();

    let deck_populated = build_articles_kanban_deck(event_loop.state());
    let tb = &deck_populated.toolbar;
    assert_eq!(tb.total_article_count, 2);
    assert_eq!(tb.total_count_label, "2 total stories");
    assert_eq!(tb.total_overdue_count, 1);
    assert_eq!(tb.overdue_badge_label, Some("⚠️ 1 Overdue".to_string()));
    assert!(tb.has_overdue());

    // Test label formatting helper
    assert_eq!(
        KanbanToolbarViewModel::format_total_count_label(0),
        "0 stories"
    );
    assert_eq!(
        KanbanToolbarViewModel::format_total_count_label(1),
        "1 story"
    );
    assert_eq!(
        KanbanToolbarViewModel::format_total_count_label(8),
        "8 total stories"
    );
}

#[test]
fn test_stage_specific_creation_triggers_open_draft_with_stage() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    for &stage in ArticleStage::all() {
        // Dispatch stage-specific quick add
        event_loop
            .dispatch(AppMessage::OpenNewArticleInStageModal(stage))
            .unwrap();

        match event_loop.state().modal {
            ModalState::ArticleForm(ref draft) => {
                assert_eq!(draft.id, None);
                assert_eq!(draft.stage, stage);
                assert!(!draft.color_hex.is_empty());
                assert!(draft.slug.is_empty());
                assert!(draft.headline.is_empty());
            }
            _ => panic!("Expected ModalState::ArticleForm for stage {:?}", stage),
        }

        // Close modal
        event_loop.dispatch(AppMessage::CloseModal).unwrap();
        assert_eq!(event_loop.state().modal, ModalState::None);
    }
}

#[test]
fn test_cosmic_and_macos_view_tree_empty_states_and_toolbar_integration() {
    let state = AppState::in_memory().expect("in-memory state");

    // 1. COSMIC View Tree
    let cosmic_app = CosmicApp::new(state.clone());
    let cosmic_view_tree = cosmic_app.build_view_tree();

    assert_eq!(cosmic_view_tree.active_tab, NavTab::ArticlesKanban);
    assert_eq!(
        CosmicHeaderBar::primary_action_label(NavTab::ArticlesKanban),
        "+ New Article"
    );

    let cosmic_deck = cosmic_view_tree.articles_deck.as_ref().unwrap();
    assert_eq!(cosmic_deck.toolbar.primary_action_label, "+ New Article");
    assert_eq!(cosmic_deck.columns.len(), 6);
    assert!(cosmic_deck.deck_empty_state.is_some());

    for col in &cosmic_deck.columns {
        assert!(col.is_empty);
        assert!(col.empty_state.is_some());
        assert_eq!(col.header.card_count, 0);
        assert_eq!(col.header.card_count_label, "0 stories");
        assert!(
            col.header.quick_add_label.starts_with("+ Add")
                || col.header.quick_add_label.starts_with("+ Archive")
        );
    }

    // 2. macOS View Tree
    let macos_app = MacosApp::new(state);
    let macos_view_tree = macos_app.build_view_tree();

    assert_eq!(macos_view_tree.active_tab, NavTab::ArticlesKanban);
    assert_eq!(
        MacosToolbar::primary_action_label(NavTab::ArticlesKanban),
        "+ New Article"
    );

    let macos_deck = macos_view_tree.articles_deck.as_ref().unwrap();
    assert_eq!(macos_deck.toolbar.primary_action_label, "+ New Article");
    assert_eq!(macos_deck.columns.len(), 6);
    assert!(macos_deck.deck_empty_state.is_some());
}
