//! Comprehensive unit and integration test suite for macOS Liquid Glass platform integration.

use chrono::Utc;
use newsjournal_core::models::{Article, Contact, Task};
use newsjournal_core::ArticleStage;
use newsjournal_gui::macos::{
    MacosAccentColor, MacosApp, MacosAppConfig, MacosAppearanceMode, MacosContainerClass,
    MacosLiquidGlass, MacosResponsiveBreakpoint, MacosThemeAdapter, MacosToolbar,
    MacosToolbarAction, MacosVibrancyBlendingMode, MacosVibrancyConfig, MacosVibrancyMaterial,
    MacosVibrancyState, MacosWindowConfig, MACOS_CARD_BLUR_RADIUS, MACOS_DEFAULT_BLUR_RADIUS,
    MACOS_DEFAULT_WINDOW_HEIGHT, MACOS_DEFAULT_WINDOW_WIDTH, MACOS_MIN_WINDOW_HEIGHT,
    MACOS_MIN_WINDOW_WIDTH, MACOS_POPOVER_BLUR_RADIUS, MACOS_SHEET_BLUR_RADIUS,
    MACOS_TRAFFIC_LIGHT_INSET_X, MACOS_TRAFFIC_LIGHT_INSET_Y,
};
use newsjournal_gui::{AppMessage, AppState, NavTab, ResolvedTheme};

#[test]
fn test_macos_window_sizing_and_constraints() {
    let mut config = MacosWindowConfig::new();

    // Verify defaults
    assert_eq!(config.placement.width, MACOS_DEFAULT_WINDOW_WIDTH);
    assert_eq!(config.placement.height, MACOS_DEFAULT_WINDOW_HEIGHT);
    assert_eq!(config.min_width, MACOS_MIN_WINDOW_WIDTH);
    assert_eq!(config.min_height, MACOS_MIN_WINDOW_HEIGHT);
    assert!(config.titlebar_transparent);
    assert!(config.fullsize_content);
    assert!(config.resizable);
    assert_eq!(config.breakpoint(), MacosResponsiveBreakpoint::Standard);
    assert_eq!(config.traffic_light_inset.0, MACOS_TRAFFIC_LIGHT_INSET_X);
    assert_eq!(config.traffic_light_inset.1, MACOS_TRAFFIC_LIGHT_INSET_Y);

    // Test resizing above minimum
    config.resize(1650.0, 1050.0);
    assert_eq!(config.placement.width, 1650.0);
    assert_eq!(config.placement.height, 1050.0);
    assert_eq!(config.breakpoint(), MacosResponsiveBreakpoint::Wide);

    // Test clamping below minimum width and height
    config.resize(500.0, 350.0);
    assert_eq!(config.placement.width, MACOS_MIN_WINDOW_WIDTH);
    assert_eq!(config.placement.height, MACOS_MIN_WINDOW_HEIGHT);
    assert_eq!(config.breakpoint(), MacosResponsiveBreakpoint::Compact);

    // Test position update
    config.set_position(120.0, 180.0);
    assert_eq!(config.placement.x, Some(120.0));
    assert_eq!(config.placement.y, Some(180.0));
}

#[test]
fn test_macos_window_title_generation() {
    let config = MacosWindowConfig::new();

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
        config.format_window_title(NavTab::ArticlesKanban, 3),
        "NewsJournal — Articles (⚠️ 3 Overdue)"
    );
}

#[test]
fn test_macos_vibrancy_configurations_and_materials() {
    let default_vibrancy = MacosVibrancyConfig::new();
    assert_eq!(
        default_vibrancy.material,
        MacosVibrancyMaterial::UnderWindowBackground
    );
    assert_eq!(
        default_vibrancy.blending_mode,
        MacosVibrancyBlendingMode::BehindWindow
    );
    assert_eq!(default_vibrancy.state, MacosVibrancyState::FollowsWindow);
    assert_eq!(default_vibrancy.corner_radius, 14.0);
    assert!(default_vibrancy.titlebar_transparent);
    assert!(default_vibrancy.fullsize_content_view);
    assert!(default_vibrancy.is_behind_window());

    // Test material specific presets
    let materials = [
        (
            MacosVibrancyMaterial::Sidebar,
            28.0,
            "NSVisualEffectMaterialSidebar",
        ),
        (
            MacosVibrancyMaterial::HeaderView,
            24.0,
            "NSVisualEffectMaterialHeaderView",
        ),
        (
            MacosVibrancyMaterial::Sheet,
            36.0,
            "NSVisualEffectMaterialSheet",
        ),
        (
            MacosVibrancyMaterial::Popover,
            20.0,
            "NSVisualEffectMaterialPopover",
        ),
        (
            MacosVibrancyMaterial::HudWindow,
            30.0,
            "NSVisualEffectMaterialHUDWindow",
        ),
        (
            MacosVibrancyMaterial::ContentBackground,
            22.0,
            "NSVisualEffectMaterialContentBackground",
        ),
        (
            MacosVibrancyMaterial::Selection,
            16.0,
            "NSVisualEffectMaterialSelection",
        ),
        (
            MacosVibrancyMaterial::UnderWindowBackground,
            32.0,
            "NSVisualEffectMaterialUnderWindowBackground",
        ),
        (
            MacosVibrancyMaterial::Menu,
            18.0,
            "NSVisualEffectMaterialMenu",
        ),
        (
            MacosVibrancyMaterial::FullScreenUI,
            40.0,
            "NSVisualEffectMaterialFullScreenUI",
        ),
    ];

    for (mat, blur, name) in materials {
        assert_eq!(mat.recommended_blur_radius(), blur);
        assert_eq!(mat.appkit_name(), name);
        let cfg = MacosVibrancyConfig::with_material(mat);
        assert_eq!(cfg.material, mat);
    }
}

#[test]
fn test_macos_liquid_glass_containers_and_themes() {
    let glass = MacosLiquidGlass::new();
    let dark_adapter =
        MacosThemeAdapter::new(MacosAppearanceMode::DarkAqua, MacosAccentColor::Blue, true);
    let dark_theme = dark_adapter.to_app_theme();

    let light_adapter =
        MacosThemeAdapter::new(MacosAppearanceMode::Aqua, MacosAccentColor::Purple, false);
    let light_theme = light_adapter.to_app_theme();

    // 1. Toolbar styling
    let toolbar_dark = glass.style_for(MacosContainerClass::Toolbar, &dark_theme);
    let toolbar_light = glass.style_for(MacosContainerClass::Toolbar, &light_theme);
    assert_eq!(toolbar_dark.blur_radius, 24.0);
    assert_eq!(toolbar_light.blur_radius, 24.0);
    assert_eq!(
        toolbar_dark.vibrancy_material,
        MacosVibrancyMaterial::HeaderView
    );
    assert!(toolbar_dark.specular_highlight_rgba.3 > 0.0);
    assert!(toolbar_light.specular_highlight_rgba.3 > 0.0);

    // 2. Sidebar styling
    let sidebar_dark = glass.style_for(MacosContainerClass::Sidebar, &dark_theme);
    assert_eq!(sidebar_dark.blur_radius, MACOS_DEFAULT_BLUR_RADIUS);
    assert_eq!(
        sidebar_dark.vibrancy_material,
        MacosVibrancyMaterial::Sidebar
    );
    assert_eq!(sidebar_dark.border_width, 0.5);

    // 3. Modal Drawer / Sheet styling
    let modal_dark = glass.style_for(MacosContainerClass::ModalDrawer, &dark_theme);
    assert_eq!(modal_dark.blur_radius, MACOS_SHEET_BLUR_RADIUS);
    assert_eq!(modal_dark.corner_radius, 14.0);
    assert_eq!(modal_dark.vibrancy_material, MacosVibrancyMaterial::Sheet);
    assert!(modal_dark.shadow.is_some());

    // 4. Card styling
    let card_dark = glass.style_for(MacosContainerClass::ArticleCard, &dark_theme);
    assert_eq!(card_dark.blur_radius, MACOS_CARD_BLUR_RADIUS);
    assert_eq!(card_dark.corner_radius, 10.0);
    assert_eq!(
        card_dark.vibrancy_material,
        MacosVibrancyMaterial::Selection
    );

    // 5. Toast / Popover styling
    let toast_light = glass.style_for(MacosContainerClass::Toast, &light_theme);
    assert_eq!(toast_light.blur_radius, MACOS_POPOVER_BLUR_RADIUS);
    assert_eq!(toast_light.corner_radius, 12.0);
    assert_eq!(
        toast_light.vibrancy_material,
        MacosVibrancyMaterial::Popover
    );
    assert_eq!(toast_light.border_rgba.0, 175); // Purple accent
    assert_eq!(toast_light.border_rgba.1, 82);
    assert_eq!(toast_light.border_rgba.2, 222);

    // 6. Overdue Article Card styling
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
    assert_eq!(overdue_card.border_rgba.3, 0.90);
    assert_eq!(overdue_card.specular_highlight_rgba, (255, 120, 120, 0.50));
}

#[test]
fn test_macos_theme_adapter_modes_and_accents() {
    let accents = [
        (MacosAccentColor::Blue, (0, 122, 255), "#007aff"),
        (MacosAccentColor::Purple, (175, 82, 222), "#af52de"),
        (MacosAccentColor::Pink, (255, 45, 85), "#ff2d55"),
        (MacosAccentColor::Red, (255, 59, 48), "#ff3b30"),
        (MacosAccentColor::Orange, (255, 149, 0), "#ff9500"),
        (MacosAccentColor::Yellow, (255, 204, 0), "#ffcc00"),
        (MacosAccentColor::Green, (52, 199, 89), "#34c759"),
        (MacosAccentColor::Graphite, (142, 142, 147), "#8e8e93"),
        (MacosAccentColor::Multicolor, (0, 122, 255), "#007aff"),
    ];

    for (accent, rgb, hex) in accents {
        assert_eq!(accent.rgb(), rgb);
        assert_eq!(accent.hex(), hex);

        let adapter = MacosThemeAdapter::new(MacosAppearanceMode::DarkAqua, accent, true);
        let theme = adapter.to_app_theme();
        assert_eq!(theme.colors.accent, rgb);
    }

    // High Contrast Modes
    let hc_dark = MacosThemeAdapter::new(
        MacosAppearanceMode::AccessibilityHighContrastDarkAqua,
        MacosAccentColor::Blue,
        true,
    );
    let hc_dark_theme = hc_dark.to_app_theme();
    assert_eq!(hc_dark_theme.colors.text_primary, (255, 255, 255));
    assert_eq!(hc_dark_theme.colors.border.3, 0.30);

    let hc_light = MacosThemeAdapter::new(
        MacosAppearanceMode::AccessibilityHighContrastAqua,
        MacosAccentColor::Blue,
        false,
    );
    let hc_light_theme = hc_light.to_app_theme();
    assert_eq!(hc_light_theme.colors.text_primary, (0, 0, 0));
    assert_eq!(hc_light_theme.colors.border.3, 0.25);

    // WCAG Contrast ratio tests
    let white = (255, 255, 255);
    let black = (0, 0, 0);
    assert!(MacosThemeAdapter::contrast_ratio(white, black) > 20.0);
}

#[test]
fn test_macos_toolbar_component() {
    let toolbar = MacosToolbar::new(NavTab::ArticlesKanban, 2, true);
    assert_eq!(toolbar.app_title, "NewsJournal");
    assert_eq!(toolbar.section_title, "Articles");
    assert_eq!(toolbar.display_title(), "NewsJournal — Articles");
    assert!(toolbar.has_overdue_alert());
    assert_eq!(
        toolbar.overdue_badge_label(),
        Some("⚠️ 2 Overdue".to_string())
    );
    assert_eq!(toolbar.theme_toggle_tooltip, "Switch to Light Mode");
    assert_eq!(toolbar.traffic_light_clearance, 76);

    assert_eq!(toolbar.segmented_tabs.len(), 4);
    assert!(toolbar.segmented_tabs[0].is_selected);
    assert_eq!(toolbar.segmented_tabs[0].tab, NavTab::ArticlesKanban);
    assert_eq!(toolbar.segmented_tabs[0].sf_symbol, "doc.richtext");

    assert_eq!(
        MacosToolbar::primary_action_label(NavTab::ArticlesKanban),
        "+ New Article"
    );
    assert_eq!(
        MacosToolbar::primary_action_label(NavTab::TasksKanban),
        "+ New Task"
    );
    assert_eq!(
        MacosToolbar::primary_action_label(NavTab::ContactsDirectory),
        "+ New Contact"
    );
    assert_eq!(
        MacosToolbar::primary_action_label(NavTab::Settings),
        "Save Settings"
    );
}

#[test]
fn test_macos_app_full_lifecycle_and_view_tree() {
    let state = AppState::in_memory().expect("in-memory state");
    let config = MacosAppConfig {
        theme_adapter: MacosThemeAdapter::new(
            MacosAppearanceMode::DarkAqua,
            MacosAccentColor::Purple,
            true,
        ),
        ..Default::default()
    };

    let mut app = MacosApp::with_config(state, config);

    // Initial state
    assert_eq!(app.state().articles.len(), 0);
    assert_eq!(app.state().tasks.len(), 0);
    assert_eq!(app.state().contacts.len(), 0);
    assert_eq!(app.state().active_tab, NavTab::ArticlesKanban);
    assert_eq!(app.toolbar.display_title(), "NewsJournal — Articles");

    // Build initial view tree
    let view_tree = app.build_view_tree();
    assert_eq!(view_tree.window_title, "NewsJournal — Articles");
    assert_eq!(view_tree.active_tab, NavTab::ArticlesKanban);
    assert!(view_tree.articles_view.is_some());
    assert!(view_tree.tasks_view.is_none());
    assert!(view_tree.modal_view.is_none());
    assert_eq!(
        view_tree.vibrancy_config.material,
        MacosVibrancyMaterial::UnderWindowBackground
    );

    // 1. Create an Article
    let article = Article::new("city-council-investigation", "City Council Zoning Report");
    let article_id = article.id;
    app.dispatch(AppMessage::CreateArticle(article))
        .expect("create article");

    assert_eq!(app.state().articles.len(), 1);

    // 2. Create a Task linked to the article
    let task = Task::new(article_id, "Analyze Public Zoning Filings");
    app.dispatch(AppMessage::CreateTask(task))
        .expect("create task");

    assert_eq!(app.state().tasks.len(), 1);

    // 3. Create a Contact and link to article
    let contact = Contact::new("Council Member Smith");
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
    assert_eq!(app.toolbar.display_title(), "NewsJournal — Tasks");

    let tasks_tree = app.build_view_tree();
    assert!(tasks_tree.tasks_view.is_some());
    assert!(tasks_tree.articles_view.is_none());

    // 5. Test Toolbar Action: Open New Article Modal
    app.handle_toolbar_action(MacosToolbarAction::OpenNewArticleModal)
        .expect("open modal");
    assert!(app.state().modal.is_open());

    let modal_tree = app.build_view_tree();
    assert!(modal_tree.modal_view.is_some());
    assert!(modal_tree.modal_glass_style.is_some());
    assert_eq!(
        modal_tree
            .modal_glass_style
            .as_ref()
            .unwrap()
            .vibrancy_material,
        MacosVibrancyMaterial::Sheet
    );

    // Close Modal
    app.dispatch(AppMessage::CloseModal).expect("close modal");
    assert!(!app.state().modal.is_open());

    // 6. Test Toolbar Action: Toggle Theme
    app.handle_toolbar_action(MacosToolbarAction::ToggleTheme)
        .expect("toggle theme");
    assert_eq!(
        app.config.theme_adapter.appearance,
        MacosAppearanceMode::Aqua
    );
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
        MacosResponsiveBreakpoint::Wide
    );

    // 8. Test Overdue Deadline evaluation and background tick
    let past_deadline = Utc::now() - chrono::Duration::hours(3);
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

    // Execute background tick
    app.tick().expect("tick deadline monitor");

    assert_eq!(app.state().deadline_summary.overdue_count, 1);
    assert!(app.toolbar.has_overdue_alert());
    assert_eq!(
        app.toolbar.overdue_badge_label(),
        Some("⚠️ 1 Overdue".to_string())
    );

    let overdue_tree = app.build_view_tree();
    assert!(overdue_tree.window_title.contains("(⚠️ 1 Overdue)"));
}
