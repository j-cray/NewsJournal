//! Comprehensive tests for Task 5.1: Articles Kanban Deck Layout and Horizontal Scrolling.

use chrono::{Duration, Utc};
use newsjournal_core::models::{Article, ArticleStage, Contact, Task, TaskStatus};
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::navigation::NavTab;
use newsjournal_gui::platform::cosmic::app::CosmicApp;
use newsjournal_gui::platform::macos::app::MacosApp;
use newsjournal_gui::state::drag_drop::{DragItem, DropTarget};
use newsjournal_gui::views::{
    build_articles_kanban_deck, build_articles_kanban_view, stage_metadata,
    ArticlesKanbanDeckViewModel, DeckLayoutConfig, DEFAULT_COLUMN_GAP, DEFAULT_COLUMN_WIDTH,
    DEFAULT_DECK_PADDING, MAX_COLUMN_WIDTH, MIN_COLUMN_WIDTH, NUM_ARTICLE_STAGES,
};
use newsjournal_gui::{AppState, EventLoop};
use uuid::Uuid;

#[test]
fn test_stage_metadata_canonical_ordering_and_properties() {
    let stages = ArticleStage::all();
    assert_eq!(stages.len(), NUM_ARTICLE_STAGES);

    let expected = [
        (
            ArticleStage::Pitching,
            0,
            "Pitching",
            "pitching",
            "💡",
            "lightbulb",
            "#F59E0B",
        ),
        (
            ArticleStage::Researching,
            1,
            "Researching",
            "researching",
            "🔍",
            "search",
            "#06B6D4",
        ),
        (
            ArticleStage::Writing,
            2,
            "Writing",
            "writing",
            "✍️",
            "edit-3",
            "#3B82F6",
        ),
        (
            ArticleStage::Editing,
            3,
            "Editing",
            "editing",
            "📝",
            "check-square",
            "#8B5CF6",
        ),
        (
            ArticleStage::ReadyToPublish,
            4,
            "Ready to Publish",
            "ready_to_publish",
            "🚀",
            "send",
            "#10B981",
        ),
        (
            ArticleStage::Published,
            5,
            "Published",
            "published",
            "📰",
            "archive",
            "#64748B",
        ),
    ];

    for (stage, idx, title, slug, emoji, icon, accent) in expected {
        let meta = stage_metadata(stage);
        assert_eq!(meta.stage, stage);
        assert_eq!(meta.index, idx);
        assert_eq!(meta.title, title);
        assert_eq!(meta.slug, slug);
        assert_eq!(meta.icon_emoji, emoji);
        assert_eq!(meta.icon_name, icon);
        assert_eq!(meta.accent_hex, accent);
        assert!(!meta.description.is_empty());
        assert!(!meta.sf_symbol.is_empty());
        assert!(!meta.empty_state_prompt.is_empty());
    }

    assert_eq!(
        ArticlesKanbanDeckViewModel::stage_names(),
        [
            "Pitching",
            "Researching",
            "Writing",
            "Editing",
            "Ready to Publish",
            "Published"
        ]
    );
    assert_eq!(ArticlesKanbanDeckViewModel::stages(), ArticleStage::all());
}

#[test]
fn test_empty_kanban_deck_layout() {
    let state = AppState::in_memory().expect("in-memory state");
    let deck = build_articles_kanban_deck(&state);

    assert_eq!(deck.columns.len(), 6);
    assert_eq!(deck.total_article_count, 0);
    assert_eq!(deck.total_overdue_count, 0);
    assert_eq!(deck.total_due_soon_count, 0);
    assert!(deck.is_empty());
    assert!(!deck.has_overdue());
    assert!(!deck.is_filtered);
    assert_eq!(deck.search_query, "");
    assert_eq!(deck.selected_stage_filter, None);

    for (i, col) in deck.columns.iter().enumerate() {
        assert_eq!(col.index, i);
        assert_eq!(col.card_count, 0);
        assert_eq!(col.overdue_count, 0);
        assert_eq!(col.due_soon_count, 0);
        assert!(col.cards.is_empty());
        assert!(col.is_empty);
        assert!(!col.is_hovered);
        assert!(!col.empty_state_prompt.is_empty());
    }
}

#[test]
fn test_kanban_deck_populated_columns_and_metrics() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);
    let now = Utc::now();

    // 1. Pitching article (healthy)
    let mut a1 = Article::new("city-budget-leak", "City Hall Budget Leak");
    a1.stage = ArticleStage::Pitching;
    a1.deadline = Some(now + Duration::days(5));
    let a1_id = a1.id;
    event_loop.dispatch(AppMessage::CreateArticle(a1)).unwrap();

    // 2. Writing article (overdue)
    let mut a2 = Article::new("bridge-inspection", "Bridge Inspection Delayed");
    a2.stage = ArticleStage::Writing;
    a2.deadline = Some(now - Duration::hours(3));
    let a2_id = a2.id;
    event_loop.dispatch(AppMessage::CreateArticle(a2)).unwrap();

    // 3. Writing article (due soon, within 6 hours)
    let mut a3 = Article::new("school-board-vote", "School Board Election Results");
    a3.stage = ArticleStage::Writing;
    a3.deadline = Some(now + Duration::hours(6));
    let a3_id = a3.id;
    event_loop.dispatch(AppMessage::CreateArticle(a3)).unwrap();

    // 4. Published article (past deadline, but published so not overdue)
    let mut a4 = Article::new("marathon-recap", "Annual Marathon Sets Record");
    a4.stage = ArticleStage::Published;
    a4.deadline = Some(now - Duration::days(1));
    let a4_id = a4.id;
    event_loop.dispatch(AppMessage::CreateArticle(a4)).unwrap();

    // Add tasks for a2
    let mut t1 = Task::new(a2_id, "Interview lead structural engineer");
    t1.status = TaskStatus::Complete;
    let t2 = Task::new(a2_id, "Review FOIA documents");
    event_loop.dispatch(AppMessage::CreateTask(t1)).unwrap();
    event_loop.dispatch(AppMessage::CreateTask(t2)).unwrap();

    // Add contact for a1
    let contact = Contact::new("Whistleblower X").with_organization("City Hall");
    let contact_id = contact.id;
    event_loop
        .dispatch(AppMessage::CreateContact(contact))
        .unwrap();
    event_loop
        .dispatch(AppMessage::LinkContactToArticle {
            article_id: a1_id,
            contact_id,
        })
        .unwrap();

    let state = event_loop.state();
    let deck = build_articles_kanban_deck(state);

    assert_eq!(deck.total_article_count, 4);
    assert_eq!(deck.total_overdue_count, 1);
    assert_eq!(deck.total_due_soon_count, 1);
    assert!(!deck.is_empty());
    assert!(deck.has_overdue());

    // Check Pitching column
    let pitching = deck.column(ArticleStage::Pitching).unwrap();
    assert_eq!(pitching.card_count, 1);
    assert_eq!(pitching.overdue_count, 0);
    assert!(!pitching.is_empty);
    assert_eq!(pitching.cards[0].id, a1_id);
    assert_eq!(pitching.cards[0].tagged_contact_count, 1);

    // Check Writing column
    let writing = deck.column(ArticleStage::Writing).unwrap();
    assert_eq!(writing.card_count, 2);
    assert_eq!(writing.overdue_count, 1);
    assert_eq!(writing.due_soon_count, 1);
    let card_a2 = writing.cards.iter().find(|c| c.id == a2_id).unwrap();
    assert!(card_a2.is_overdue);
    assert_eq!(card_a2.task_completed, 1);
    assert_eq!(card_a2.task_total, 2);
    let card_a3 = writing.cards.iter().find(|c| c.id == a3_id).unwrap();
    assert!(card_a3.is_due_soon);

    // Check Published column
    let published = deck.column(ArticleStage::Published).unwrap();
    assert_eq!(published.card_count, 1);
    assert_eq!(published.overdue_count, 0); // Published is never overdue
    assert_eq!(published.cards[0].id, a4_id);

    // Check Researching and Editing (empty)
    assert_eq!(deck.count_for_stage(ArticleStage::Researching), 0);
    assert_eq!(deck.count_for_stage(ArticleStage::Editing), 0);
    assert_eq!(deck.count_for_stage(ArticleStage::ReadyToPublish), 0);

    // Lookups
    assert_eq!(
        deck.column_by_index(0).unwrap().stage,
        ArticleStage::Pitching
    );
    assert_eq!(
        deck.column_by_index(2).unwrap().stage,
        ArticleStage::Writing
    );
    assert_eq!(
        deck.column_by_index(5).unwrap().stage,
        ArticleStage::Published
    );
    assert!(deck.column_by_index(6).is_none());

    let (col, card) = deck.card(a2_id).unwrap();
    assert_eq!(col.stage, ArticleStage::Writing);
    assert_eq!(card.slug, "bridge-inspection");
    assert!(deck.card(Uuid::new_v4()).is_none());

    // Backwards compatibility
    let view_cols = build_articles_kanban_view(state);
    assert_eq!(view_cols.len(), 6);
    assert_eq!(view_cols, deck.columns);
}

#[test]
fn test_deck_layout_horizontal_geometry_and_scrolling() {
    let mut layout = DeckLayoutConfig::new(1280.0);

    assert_eq!(layout.column_width, DEFAULT_COLUMN_WIDTH); // 320.0
    assert_eq!(layout.column_gap, DEFAULT_COLUMN_GAP); // 16.0
    assert_eq!(layout.deck_padding, DEFAULT_DECK_PADDING); // 20.0
    assert_eq!(layout.viewport_width, 1280.0);
    assert_eq!(layout.scroll_offset_x, 0.0);

    // Total content width = 6 * 320 + 5 * 16 + 2 * 20 = 1920 + 80 + 40 = 2040.0
    let total_width = layout.total_content_width();
    assert_eq!(total_width, 2040.0);

    // Max scroll offset = 2040 - 1280 = 760.0
    let max_scroll = layout.max_scroll_offset();
    assert_eq!(max_scroll, 760.0);

    // Initial scroll state at offset 0.0
    assert!(!layout.can_scroll_left());
    assert!(layout.can_scroll_right());
    assert_eq!(layout.scroll_progress(), 0.0);

    // Column positions:
    // col 0: 20 + 0 * 336 = 20.0 (bounds 20..340)
    // col 1: 20 + 1 * 336 = 356.0 (bounds 356..676)
    // col 2: 20 + 2 * 336 = 692.0 (bounds 692..1012)
    // col 3: 20 + 3 * 336 = 1028.0 (bounds 1028..1348) -> partially visible in 1280px viewport
    // col 4: 20 + 4 * 336 = 1364.0 (bounds 1364..1684) -> hidden
    // col 5: 20 + 5 * 336 = 1700.0 (bounds 1700..2020) -> hidden
    assert_eq!(layout.column_x_position(0), 20.0);
    assert_eq!(layout.column_x_position(1), 356.0);
    assert_eq!(layout.column_x_position(2), 692.0);
    assert_eq!(layout.column_x_position(3), 1028.0);
    assert_eq!(layout.column_x_position(4), 1364.0);
    assert_eq!(layout.column_x_position(5), 1700.0);

    assert!(layout.is_column_fully_visible(0));
    assert!(layout.is_column_fully_visible(1));
    assert!(layout.is_column_fully_visible(2));
    assert!(layout.is_column_visible(3));
    assert!(!layout.is_column_fully_visible(3)); // 1348 > 1280
    assert!(!layout.is_column_visible(4));
    assert!(!layout.is_column_visible(5));

    assert_eq!(layout.visible_column_indices(), vec![0, 1, 2, 3]);

    // Scroll to reveal col 4 (Ready to Publish)
    let offset_col4 = layout.scroll_offset_for_column(4);
    layout.scroll_to(offset_col4);
    assert!(layout.is_column_visible(4));
    assert!(layout.can_scroll_left());

    // Scroll to last column (Published)
    layout.scroll_to_stage(ArticleStage::Published);
    assert_eq!(layout.scroll_offset_x, max_scroll);
    assert!(layout.can_scroll_left());
    assert!(!layout.can_scroll_right());
    assert_eq!(layout.scroll_progress(), 1.0);
    assert!(layout.is_column_fully_visible(5));

    // Page left
    layout.scroll_page_left();
    assert!(layout.scroll_offset_x < max_scroll);

    // Page right
    layout.scroll_page_right();
    assert_eq!(layout.scroll_offset_x, max_scroll);

    // Scroll by delta with clamping
    layout.scroll_by(-200.0);
    assert_eq!(layout.scroll_offset_x, max_scroll - 200.0);
    layout.scroll_by(-10000.0);
    assert_eq!(layout.scroll_offset_x, 0.0); // Clamped at 0.0
    layout.scroll_by(50000.0);
    assert_eq!(layout.scroll_offset_x, max_scroll); // Clamped at max

    // Wide screen viewport (e.g. 2560px ultra-wide)
    layout.set_viewport_width(2560.0);
    assert_eq!(layout.max_scroll_offset(), 0.0);
    assert_eq!(layout.scroll_offset_x, 0.0);
    assert!(!layout.can_scroll_left());
    assert!(!layout.can_scroll_right());
    assert_eq!(layout.visible_column_indices(), vec![0, 1, 2, 3, 4, 5]);
    for i in 0..6 {
        assert!(layout.is_column_fully_visible(i));
    }
}

#[test]
fn test_deck_layout_column_resizing_and_clamping() {
    let mut layout = DeckLayoutConfig::new(1024.0);

    // Test column width clamping
    layout.set_column_width(500.0);
    assert_eq!(layout.column_width, MAX_COLUMN_WIDTH); // 440.0

    layout.set_column_width(100.0);
    assert_eq!(layout.column_width, MIN_COLUMN_WIDTH); // 260.0

    layout = layout.with_column_width(300.0);
    assert_eq!(layout.column_width, 300.0);

    // Test builder with scroll
    let scrolled = DeckLayoutConfig::new(1000.0)
        .with_column_width(350.0)
        .with_scroll(400.0);
    assert_eq!(scrolled.column_width, 350.0);
    assert_eq!(scrolled.scroll_offset_x, 400.0);
}

#[test]
fn test_deck_filtering_and_drag_drop_integration() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let mut a1 = Article::new("climate-report", "State Climate Policy Update");
    a1.stage = ArticleStage::Pitching;
    let a1_id = a1.id;
    event_loop.dispatch(AppMessage::CreateArticle(a1)).unwrap();

    let mut a2 = Article::new("sports-stadium", "Stadium Financing Plan");
    a2.stage = ArticleStage::Writing;
    let _a2_id = a2.id;
    event_loop.dispatch(AppMessage::CreateArticle(a2)).unwrap();

    // 1. Search filtering
    event_loop
        .dispatch(AppMessage::SetSearchQuery("climate".to_string()))
        .unwrap();
    let deck = build_articles_kanban_deck(event_loop.state());
    assert!(deck.is_filtered);
    assert_eq!(deck.search_query, "climate");
    assert_eq!(deck.total_article_count, 1);
    assert_eq!(deck.column(ArticleStage::Pitching).unwrap().card_count, 1);
    assert_eq!(deck.column(ArticleStage::Writing).unwrap().card_count, 0);

    // Clear search
    event_loop
        .dispatch(AppMessage::SetSearchQuery(String::new()))
        .unwrap();

    // 2. Drag and drop hover target integration
    event_loop
        .dispatch(AppMessage::DragStart(DragItem::ArticleCard {
            id: a1_id,
            origin_stage: ArticleStage::Pitching,
        }))
        .unwrap();
    event_loop
        .dispatch(AppMessage::DragHover(Some(DropTarget::ArticleColumn(
            ArticleStage::Writing,
        ))))
        .unwrap();

    let deck = build_articles_kanban_deck(event_loop.state());
    let pitching = deck.column(ArticleStage::Pitching).unwrap();
    assert!(pitching.cards[0].is_dragging);
    assert!(!pitching.is_hovered);

    let writing = deck.column(ArticleStage::Writing).unwrap();
    assert!(writing.is_hovered);

    // Drop into Writing column
    event_loop.dispatch(AppMessage::DragDrop).unwrap();
    let deck = build_articles_kanban_deck(event_loop.state());
    assert_eq!(deck.column(ArticleStage::Pitching).unwrap().card_count, 0);
    assert_eq!(deck.column(ArticleStage::Writing).unwrap().card_count, 2);
    let writing = deck.column(ArticleStage::Writing).unwrap();
    assert!(!writing.is_hovered);
    assert!(!writing.cards[0].is_dragging);
    assert!(!writing.cards[1].is_dragging);
}

#[test]
fn test_cosmic_and_macos_app_view_tree_integration() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut cosmic_app = CosmicApp::new(state.clone());
    let mut macos_app = MacosApp::new(state);

    // 1. Cosmic view tree on ArticlesKanban tab
    let cosmic_tree = cosmic_app.build_view_tree();
    assert_eq!(cosmic_tree.active_tab, NavTab::ArticlesKanban);
    assert!(cosmic_tree.articles_deck.is_some());
    assert!(cosmic_tree.articles_view.is_some());
    let deck = cosmic_tree.articles_deck.unwrap();
    assert_eq!(deck.columns.len(), 6);
    assert_eq!(cosmic_tree.column_glass_style.corner_radius, 12.0);

    // 2. macOS view tree on ArticlesKanban tab
    let macos_tree = macos_app.build_view_tree();
    assert_eq!(macos_tree.active_tab, NavTab::ArticlesKanban);
    assert!(macos_tree.articles_deck.is_some());
    assert!(macos_tree.articles_view.is_some());
    let deck = macos_tree.articles_deck.unwrap();
    assert_eq!(deck.columns.len(), 6);
    assert_eq!(macos_tree.column_glass_style.corner_radius, 12.0);

    // 3. Switching to TasksKanban tab
    cosmic_app
        .dispatch(AppMessage::NavigateTo(NavTab::TasksKanban))
        .unwrap();
    macos_app
        .dispatch(AppMessage::NavigateTo(NavTab::TasksKanban))
        .unwrap();

    let cosmic_tree_tasks = cosmic_app.build_view_tree();
    assert_eq!(cosmic_tree_tasks.active_tab, NavTab::TasksKanban);
    assert!(cosmic_tree_tasks.articles_deck.is_none());
    assert!(cosmic_tree_tasks.articles_view.is_none());
    assert!(cosmic_tree_tasks.tasks_view.is_some());

    let macos_tree_tasks = macos_app.build_view_tree();
    assert_eq!(macos_tree_tasks.active_tab, NavTab::TasksKanban);
    assert!(macos_tree_tasks.articles_deck.is_none());
    assert!(macos_tree_tasks.articles_view.is_none());
    assert!(macos_tree_tasks.tasks_view.is_some());
}
