//! Comprehensive unit and integration test suite for Linux COSMIC platform integration.

use chrono::Utc;
use newsjournal_core::models::{Article, Contact, Task};
use newsjournal_core::ArticleStage;
use newsjournal_gui::cosmic::{
    CosmicAccentColor, CosmicApp, CosmicAppConfig, CosmicContainerClass, CosmicFrostedGlass,
    CosmicHeaderBar, CosmicHeaderBarAction, CosmicIconManager, CosmicIconSize,
    CosmicResponsiveBreakpoint, CosmicThemeAdapter, CosmicThemeMode, CosmicWindowConfig,
    COSMIC_APP_ICON_NAME, COSMIC_APP_ID, COSMIC_DEFAULT_WINDOW_HEIGHT, COSMIC_DEFAULT_WINDOW_WIDTH,
    COSMIC_MIN_WINDOW_HEIGHT, COSMIC_MIN_WINDOW_WIDTH, COSMIC_SYMBOLIC_ICON_NAME,
};
use newsjournal_gui::{AppMessage, AppState, NavTab, ResolvedTheme};

#[test]
fn test_cosmic_window_sizing_and_constraints() {
    let mut config = CosmicWindowConfig::new();

    // Verify defaults
    assert_eq!(config.placement.width, COSMIC_DEFAULT_WINDOW_WIDTH);
    assert_eq!(config.placement.height, COSMIC_DEFAULT_WINDOW_HEIGHT);
    assert_eq!(config.min_width, COSMIC_MIN_WINDOW_WIDTH);
    assert_eq!(config.min_height, COSMIC_MIN_WINDOW_HEIGHT);
    assert!(config.transparent);
    assert!(config.resizable);
    assert!(config.decorated);
    assert_eq!(config.breakpoint(), CosmicResponsiveBreakpoint::Standard);

    // Test resizing above minimum
    config.resize(1600.0, 1000.0);
    assert_eq!(config.placement.width, 1600.0);
    assert_eq!(config.placement.height, 1000.0);
    assert_eq!(config.breakpoint(), CosmicResponsiveBreakpoint::Wide);

    // Test clamping below minimum width and height
    config.resize(600.0, 400.0);
    assert_eq!(config.placement.width, COSMIC_MIN_WINDOW_WIDTH);
    assert_eq!(config.placement.height, COSMIC_MIN_WINDOW_HEIGHT);
    assert_eq!(config.breakpoint(), CosmicResponsiveBreakpoint::Compact);

    // Test position update
    config.set_position(100.0, 150.0);
    assert_eq!(config.placement.x, Some(100.0));
    assert_eq!(config.placement.y, Some(150.0));
}

#[test]
fn test_cosmic_window_title_generation() {
    let config = CosmicWindowConfig::new();

    // Standard titles
    assert_eq!(
        config.format_window_title(NavTab::ArticlesKanban, 0),
        "NewsJournal — Articles"
    );
    assert_eq!(
        config.format_window_title(NavTab::TasksKanban, 0),
        "NewsJournal — Tasks"
    );
    assert_eq!(
        config.format_window_title(NavTab::ContactsDirectory, 0),
        "NewsJournal — Contacts"
    );
    assert_eq!(
        config.format_window_title(NavTab::Settings, 0),
        "NewsJournal — Settings"
    );

    // Overdue alert in window title
    assert_eq!(
        config.format_window_title(NavTab::ArticlesKanban, 2),
        "NewsJournal — Articles (⚠️ 2 Overdue)"
    );
}

#[test]
fn test_cosmic_frosted_glass_containers_and_themes() {
    let glass = CosmicFrostedGlass::new();
    let dark_adapter =
        CosmicThemeAdapter::new(CosmicThemeMode::Dark, CosmicAccentColor::CosmicBlue, true);
    let dark_theme = dark_adapter.to_app_theme();

    let light_adapter = CosmicThemeAdapter::new(
        CosmicThemeMode::Light,
        CosmicAccentColor::System76Orange,
        false,
    );
    let light_theme = light_adapter.to_app_theme();

    // Test HeaderBar styling
    let header_dark = glass.style_for(CosmicContainerClass::HeaderBar, &dark_theme);
    let header_light = glass.style_for(CosmicContainerClass::HeaderBar, &light_theme);
    assert_eq!(header_dark.blur_radius, 24.0);
    assert_eq!(header_light.blur_radius, 24.0);
    assert!(header_dark.surface_rgba.3 > 0.80);
    assert!(header_light.surface_rgba.3 > 0.80);

    // Test Sidebar styling
    let sidebar_dark = glass.style_for(CosmicContainerClass::Sidebar, &dark_theme);
    assert_eq!(sidebar_dark.blur_radius, 28.0);
    assert_eq!(sidebar_dark.border_width, 1.0);

    // Test ModalDrawer styling
    let modal_dark = glass.style_for(CosmicContainerClass::ModalDrawer, &dark_theme);
    assert_eq!(modal_dark.blur_radius, 32.0);
    assert_eq!(modal_dark.corner_radius, 16.0);
    assert_eq!(modal_dark.border_width, 1.5);
    assert!(modal_dark.shadow.is_some());

    // Test Toast styling
    let toast_light = glass.style_for(CosmicContainerClass::Toast, &light_theme);
    assert_eq!(toast_light.corner_radius, 12.0);
    assert_eq!(toast_light.border_rgba.0, 248); // System76 Orange accent
    assert_eq!(toast_light.border_rgba.1, 152);
    assert_eq!(toast_light.border_rgba.2, 32);

    // Test Overdue Article Card styling
    let card_dark = glass.style_for(CosmicContainerClass::ArticleCard, &dark_theme);
    let overdue_card = glass.overdue_card_style(&card_dark, &dark_theme.colors);
    assert_eq!(overdue_card.border_width, 2.0);
    assert_eq!(
        overdue_card.border_rgba.0,
        dark_theme.colors.overdue_alert.0
    );
    assert_eq!(
        overdue_card.border_rgba.1,
        dark_theme.colors.overdue_alert.1
    );
    assert_eq!(
        overdue_card.border_rgba.2,
        dark_theme.colors.overdue_alert.2
    );
    assert_eq!(overdue_card.border_rgba.3, 0.85);
}

#[test]
fn test_cosmic_theme_adapter_modes_and_accents() {
    // Test all accent colors
    let accents = [
        (CosmicAccentColor::CosmicBlue, (58, 123, 213), "#3a7bd5"),
        (CosmicAccentColor::System76Orange, (248, 152, 32), "#f89820"),
        (CosmicAccentColor::Teal, (54, 207, 201), "#36cfc9"),
        (CosmicAccentColor::Emerald, (82, 196, 26), "#52c41a"),
        (CosmicAccentColor::Violet, (146, 84, 222), "#9254de"),
    ];

    for (accent, rgb, hex) in accents {
        assert_eq!(accent.rgb(), rgb);
        assert_eq!(accent.hex(), hex);

        let adapter = CosmicThemeAdapter::new(CosmicThemeMode::Dark, accent, true);
        let theme = adapter.to_app_theme();
        assert_eq!(theme.colors.accent, rgb);
    }

    // High Contrast Modes
    let hc_dark = CosmicThemeAdapter::new(
        CosmicThemeMode::HighContrastDark,
        CosmicAccentColor::CosmicBlue,
        true,
    );
    let hc_dark_theme = hc_dark.to_app_theme();
    assert_eq!(hc_dark_theme.colors.text_primary, (255, 255, 255));
    assert_eq!(hc_dark_theme.colors.border.3, 0.28);

    let hc_light = CosmicThemeAdapter::new(
        CosmicThemeMode::HighContrastLight,
        CosmicAccentColor::CosmicBlue,
        false,
    );
    let hc_light_theme = hc_light.to_app_theme();
    assert_eq!(hc_light_theme.colors.text_primary, (0, 0, 0));
    assert_eq!(hc_light_theme.colors.border.3, 0.22);
}

#[test]
fn test_cosmic_icon_manager_and_assets() {
    let icon_mgr = CosmicIconManager::new();
    assert_eq!(icon_mgr.app_id(), COSMIC_APP_ID);
    assert_eq!(icon_mgr.icon_name(), COSMIC_APP_ICON_NAME);
    assert_eq!(icon_mgr.symbolic_icon_name(), COSMIC_SYMBOLIC_ICON_NAME);
    assert!(icon_mgr.validate_assets());

    // Check SVG content
    let app_svg = icon_mgr.app_icon_svg_bytes();
    let sym_svg = icon_mgr.symbolic_icon_svg_bytes();
    assert!(!app_svg.is_empty());
    assert!(!sym_svg.is_empty());

    let app_svg_str = std::str::from_utf8(app_svg).unwrap();
    assert!(app_svg_str.contains("<svg"));
    assert!(app_svg_str.contains("cosmic-glass-grad"));

    // Check icon sizes
    assert_eq!(CosmicIconSize::Symbolic16.pixels(), 16);
    assert_eq!(CosmicIconSize::Small24.pixels(), 24);
    assert_eq!(CosmicIconSize::Medium32.pixels(), 32);
    assert_eq!(CosmicIconSize::Large48.pixels(), 48);
    assert_eq!(CosmicIconSize::ExtraLarge64.pixels(), 64);
    assert_eq!(CosmicIconSize::Tile128.pixels(), 128);
    assert_eq!(CosmicIconSize::Ultra256.pixels(), 256);
}

#[test]
fn test_cosmic_header_bar_component() {
    let header = CosmicHeaderBar::new(NavTab::ArticlesKanban, 3, true);
    assert_eq!(header.app_title, "NewsJournal");
    assert_eq!(header.section_title, "Articles");
    assert_eq!(header.display_title(), "NewsJournal — Articles");
    assert!(header.has_overdue_alert());
    assert_eq!(
        header.overdue_badge_label(),
        Some("⚠️ 3 Overdue".to_string())
    );
    assert_eq!(header.theme_toggle_tooltip, "Switch to Light Mode");

    assert_eq!(
        CosmicHeaderBar::primary_action_label(NavTab::ArticlesKanban),
        "+ New Article"
    );
    assert_eq!(
        CosmicHeaderBar::primary_action_label(NavTab::TasksKanban),
        "+ New Task"
    );
    assert_eq!(
        CosmicHeaderBar::primary_action_label(NavTab::ContactsDirectory),
        "+ New Contact"
    );
    assert_eq!(
        CosmicHeaderBar::primary_action_label(NavTab::Settings),
        "Save Settings"
    );
}

#[test]
fn test_cosmic_app_full_lifecycle_and_view_tree() {
    let state = AppState::in_memory().expect("in-memory state");
    let config = CosmicAppConfig {
        theme_adapter: CosmicThemeAdapter::new(
            CosmicThemeMode::Dark,
            CosmicAccentColor::Teal,
            true,
        ),
        ..Default::default()
    };

    let mut app = CosmicApp::with_config(state, config);

    // Initial state
    assert_eq!(app.state().articles.len(), 0);
    assert_eq!(app.state().tasks.len(), 0);
    assert_eq!(app.state().contacts.len(), 0);
    assert_eq!(app.state().active_tab, NavTab::ArticlesKanban);
    assert_eq!(app.header_bar.display_title(), "NewsJournal — Articles");

    // Build initial view tree
    let view_tree = app.build_view_tree();
    assert_eq!(view_tree.window_title, "NewsJournal — Articles");
    assert_eq!(view_tree.active_tab, NavTab::ArticlesKanban);
    assert!(view_tree.articles_view.is_some());
    assert!(view_tree.tasks_view.is_none());
    assert!(view_tree.modal_view.is_none());

    // 1. Create an Article
    let article = Article::new("transit-investigation", "Transit Overhaul Deep Dive");
    let article_id = article.id;
    app.dispatch(AppMessage::CreateArticle(article))
        .expect("create article");

    assert_eq!(app.state().articles.len(), 1);

    // 2. Create a Task linked to the article
    let task = Task::new(article_id, "FOIA Request Submission");
    app.dispatch(AppMessage::CreateTask(task))
        .expect("create task");

    assert_eq!(app.state().tasks.len(), 1);

    // 3. Create a Contact and link to article
    let contact = Contact::new("Jane Reporter");
    let contact_id = contact.id;
    app.dispatch(AppMessage::CreateContact(contact))
        .expect("create contact");
    app.dispatch(AppMessage::LinkContactToArticle {
        article_id,
        contact_id,
    })
    .expect("link contact");

    assert_eq!(app.state().contacts.len(), 1);

    // 4. Test Navigation tab switching
    app.dispatch(AppMessage::NavigateTo(NavTab::TasksKanban))
        .expect("switch to tasks");
    assert_eq!(app.state().active_tab, NavTab::TasksKanban);
    assert_eq!(app.header_bar.display_title(), "NewsJournal — Tasks");

    let tasks_tree = app.build_view_tree();
    assert!(tasks_tree.tasks_view.is_some());
    assert!(tasks_tree.articles_view.is_none());

    // 5. Test Header Action: Open New Article Modal
    app.handle_header_action(CosmicHeaderBarAction::OpenNewArticleModal)
        .expect("open modal");
    assert!(app.state().modal.is_open());

    let modal_tree = app.build_view_tree();
    assert!(modal_tree.modal_view.is_some());
    assert!(modal_tree.modal_glass_style.is_some());

    // Close Modal
    app.dispatch(AppMessage::CloseModal).expect("close modal");
    assert!(!app.state().modal.is_open());

    // 6. Test Header Action: Toggle Theme
    app.handle_header_action(CosmicHeaderBarAction::ToggleTheme)
        .expect("toggle theme");
    assert_eq!(app.config.theme_adapter.mode, CosmicThemeMode::Light);
    assert_eq!(
        app.config.theme_adapter.to_app_theme().resolved,
        ResolvedTheme::Light
    );

    // 7. Test Window Resizing
    app.resize_window(1550.0, 950.0);
    assert_eq!(app.config.window.placement.width, 1550.0);
    assert_eq!(app.config.window.placement.height, 950.0);
    assert_eq!(
        app.config.window.breakpoint(),
        CosmicResponsiveBreakpoint::Wide
    );

    // 8. Test Overdue Deadline evaluation and background tick
    // Update article deadline to the past
    let past_deadline = Utc::now() - chrono::Duration::hours(2);
    let mut overdue_article = app
        .state()
        .storage
        .get_article(article_id)
        .unwrap()
        .unwrap();
    overdue_article.deadline = Some(past_deadline);
    overdue_article.stage = ArticleStage::Writing;
    app.dispatch(AppMessage::UpdateArticle(overdue_article))
        .expect("update deadline");

    // Execute tick
    app.tick().expect("tick deadline monitor");

    assert_eq!(app.state().deadline_summary.overdue_count, 1);
    assert!(app.header_bar.has_overdue_alert());
    assert_eq!(
        app.header_bar.overdue_badge_label(),
        Some("⚠️ 1 Overdue".to_string())
    );

    let overdue_tree = app.build_view_tree();
    assert!(overdue_tree.window_title.contains("(⚠️ 1 Overdue)"));
}
