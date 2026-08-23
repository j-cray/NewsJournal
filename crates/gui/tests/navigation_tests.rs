//! Comprehensive integration tests for the Left Navigation Bar component,
//! keyboard shortcut resolution engine, active state highlighting, and platform sidebars.

use chrono::{Duration, Utc};
use newsjournal_core::models::{Article, Contact, Task, TaskStatus};
use newsjournal_core::ArticleStage;
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::navigation::{resolve_nav_shortcut, NavKeyAction, NavKeyModifiers, NavTab};
use newsjournal_gui::platform::cosmic::glass::CosmicFrostedGlass;
use newsjournal_gui::platform::cosmic::sidebar::{
    CosmicNavBar, COSMIC_NAV_ITEM_HEIGHT, COSMIC_SIDEBAR_COMPACT_WIDTH,
    COSMIC_SIDEBAR_STANDARD_WIDTH,
};
use newsjournal_gui::platform::cosmic::CosmicApp;
use newsjournal_gui::platform::macos::glass::MacosLiquidGlass;
use newsjournal_gui::platform::macos::sidebar::{
    MacosNavBar, MACOS_NAV_ITEM_HEIGHT, MACOS_SIDEBAR_COMPACT_WIDTH, MACOS_SIDEBAR_STANDARD_WIDTH,
};
use newsjournal_gui::platform::macos::vibrancy::MacosVibrancyConfig;
use newsjournal_gui::platform::macos::MacosApp;
use newsjournal_gui::views::{build_nav_bar_view, build_nav_bar_view_with_platform};
use newsjournal_gui::{AppState, EventLoop};

#[test]
fn test_nav_tab_full_properties_and_cycles() {
    let all = NavTab::all();
    assert_eq!(all.len(), 4);

    let main = NavTab::main_tabs();
    assert_eq!(main.len(), 3);
    assert_eq!(main[0], NavTab::ArticlesKanban);
    assert_eq!(main[1], NavTab::TasksKanban);
    assert_eq!(main[2], NavTab::ContactsDirectory);

    let docked = NavTab::docked_tabs();
    assert_eq!(docked.len(), 1);
    assert_eq!(docked[0], NavTab::Settings);

    assert_eq!(NavTab::ArticlesKanban.title(), "Articles");
    assert_eq!(NavTab::ArticlesKanban.subtitle(), "Main Deck");
    assert_eq!(NavTab::ArticlesKanban.icon_name(), "newspaper");
    assert_eq!(NavTab::ArticlesKanban.icon_emoji(), "📰");
    assert_eq!(NavTab::ArticlesKanban.sf_symbol(), "doc.richtext");
    assert_eq!(NavTab::ArticlesKanban.shortcut_digit(), '1');
    assert_eq!(NavTab::ArticlesKanban.shortcut_label(), "Ctrl+1");
    assert_eq!(
        NavTab::ArticlesKanban.shortcut_label_for_platform(true),
        "⌘1"
    );
    assert_eq!(
        NavTab::ArticlesKanban.shortcut_label_for_platform(false),
        "Ctrl+1"
    );

    assert_eq!(NavTab::Settings.title(), "Settings");
    assert_eq!(NavTab::Settings.subtitle(), "Preferences");
    assert_eq!(NavTab::Settings.shortcut_digit(), '4');
    assert!(NavTab::Settings.is_docked());

    // Cycle next
    assert_eq!(NavTab::ArticlesKanban.next(), NavTab::TasksKanban);
    assert_eq!(NavTab::TasksKanban.next(), NavTab::ContactsDirectory);
    assert_eq!(NavTab::ContactsDirectory.next(), NavTab::Settings);
    assert_eq!(NavTab::Settings.next(), NavTab::ArticlesKanban);

    // Cycle prev
    assert_eq!(NavTab::ArticlesKanban.prev(), NavTab::Settings);
    assert_eq!(NavTab::TasksKanban.prev(), NavTab::ArticlesKanban);
    assert_eq!(NavTab::ContactsDirectory.prev(), NavTab::TasksKanban);
    assert_eq!(NavTab::Settings.prev(), NavTab::ContactsDirectory);
}

#[test]
fn test_keyboard_shortcut_resolution_exhaustive() {
    // Linux Primary (Ctrl)
    let linux_ctrl = NavKeyModifiers {
        ctrl: true,
        meta: false,
        alt: false,
        shift: false,
    };
    assert_eq!(
        resolve_nav_shortcut("1", linux_ctrl, false),
        Some(NavKeyAction::SelectTab(NavTab::ArticlesKanban))
    );
    assert_eq!(
        resolve_nav_shortcut("2", linux_ctrl, false),
        Some(NavKeyAction::SelectTab(NavTab::TasksKanban))
    );
    assert_eq!(
        resolve_nav_shortcut("3", linux_ctrl, false),
        Some(NavKeyAction::SelectTab(NavTab::ContactsDirectory))
    );
    assert_eq!(
        resolve_nav_shortcut("4", linux_ctrl, false),
        Some(NavKeyAction::SelectTab(NavTab::Settings))
    );
    assert_eq!(
        resolve_nav_shortcut(",", linux_ctrl, false),
        Some(NavKeyAction::SelectTab(NavTab::Settings))
    );
    assert_eq!(
        resolve_nav_shortcut("Tab", linux_ctrl, false),
        Some(NavKeyAction::NextTab)
    );
    assert_eq!(
        resolve_nav_shortcut("]", linux_ctrl, false),
        Some(NavKeyAction::NextTab)
    );
    assert_eq!(
        resolve_nav_shortcut("[", linux_ctrl, false),
        Some(NavKeyAction::PrevTab)
    );

    // macOS Primary (Meta / Cmd)
    let macos_meta = NavKeyModifiers {
        ctrl: false,
        meta: true,
        alt: false,
        shift: false,
    };
    assert_eq!(
        resolve_nav_shortcut("1", macos_meta, true),
        Some(NavKeyAction::SelectTab(NavTab::ArticlesKanban))
    );
    assert_eq!(
        resolve_nav_shortcut("4", macos_meta, true),
        Some(NavKeyAction::SelectTab(NavTab::Settings))
    );
    assert_eq!(
        resolve_nav_shortcut(",", macos_meta, true),
        Some(NavKeyAction::SelectTab(NavTab::Settings))
    );

    // Directional keys without modifiers
    let none = NavKeyModifiers::none();
    assert_eq!(
        resolve_nav_shortcut("ArrowDown", none, false),
        Some(NavKeyAction::NextTab)
    );
    assert_eq!(
        resolve_nav_shortcut("ArrowUp", none, false),
        Some(NavKeyAction::PrevTab)
    );
    assert_eq!(
        resolve_nav_shortcut("Home", none, false),
        Some(NavKeyAction::FirstTab)
    );
    assert_eq!(
        resolve_nav_shortcut("End", none, false),
        Some(NavKeyAction::LastTab)
    );
}

#[test]
fn test_nav_bar_view_model_and_active_highlighting() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let now = Utc::now();

    // 1. Add 2 articles (1 overdue, 1 on-time)
    let mut a1 = Article::new("headline-scoop", "City Budget Revealed");
    a1.stage = ArticleStage::Writing;
    a1.deadline = Some(now - Duration::hours(3)); // Overdue
    let a1_id = a1.id;
    event_loop.dispatch(AppMessage::CreateArticle(a1)).unwrap();

    let mut a2 = Article::new("profile-piece", "Interview with Fire Chief");
    a2.stage = ArticleStage::Researching;
    a2.deadline = Some(now + Duration::days(2)); // Not overdue
    event_loop.dispatch(AppMessage::CreateArticle(a2)).unwrap();

    // 2. Add tasks
    event_loop
        .dispatch(AppMessage::CreateTask(Task::new(
            a1_id,
            "Analyze balance sheet",
        )))
        .unwrap();
    let mut t2 = Task::new(a1_id, "Call budget director");
    t2.status = TaskStatus::Complete;
    event_loop.dispatch(AppMessage::CreateTask(t2)).unwrap();

    // 3. Add contacts
    event_loop
        .dispatch(AppMessage::CreateContact(Contact::new("Finance Lead")))
        .unwrap();
    event_loop
        .dispatch(AppMessage::CreateContact(Contact::new("Press Officer")))
        .unwrap();

    let state = event_loop.state();

    // Verify initial active tab = Articles
    let nav_bar = build_nav_bar_view(state);
    assert_eq!(nav_bar.active_tab, NavTab::ArticlesKanban);
    assert_eq!(nav_bar.main_items.len(), 3);
    assert_eq!(nav_bar.docked_items.len(), 1);
    assert_eq!(nav_bar.all_items.len(), 4);

    let nav_bar_mac = build_nav_bar_view_with_platform(state, true);
    assert_eq!(
        nav_bar_mac
            .item_for_tab(NavTab::ArticlesKanban)
            .unwrap()
            .shortcut_label,
        "⌘1"
    );
    let nav_bar_linux = build_nav_bar_view_with_platform(state, false);
    assert_eq!(
        nav_bar_linux
            .item_for_tab(NavTab::ArticlesKanban)
            .unwrap()
            .shortcut_label,
        "Ctrl+1"
    );

    let art_item = nav_bar.item_for_tab(NavTab::ArticlesKanban).unwrap();
    assert!(art_item.is_active);
    assert_eq!(art_item.badge_count, Some(2));
    assert!(art_item.has_overdue_alert);
    assert_eq!(art_item.overdue_count, 1);
    assert_eq!(art_item.badge_label, Some("2 (⚠️ 1)".to_string()));

    let task_item = nav_bar.item_for_tab(NavTab::TasksKanban).unwrap();
    assert!(!task_item.is_active);
    assert_eq!(task_item.badge_count, Some(2));
    assert_eq!(task_item.badge_label, Some("2".to_string()));

    let contact_item = nav_bar.item_for_tab(NavTab::ContactsDirectory).unwrap();
    assert!(!contact_item.is_active);
    assert_eq!(contact_item.badge_count, Some(2));

    let settings_item = nav_bar.item_for_tab(NavTab::Settings).unwrap();
    assert!(!settings_item.is_active);
    assert!(settings_item.is_docked);
    assert_eq!(settings_item.badge_count, None);

    // Switch to Tasks
    event_loop
        .dispatch(AppMessage::NavigateTo(NavTab::TasksKanban))
        .unwrap();
    let state = event_loop.state();
    let nav_bar = build_nav_bar_view(state);
    assert_eq!(nav_bar.active_tab, NavTab::TasksKanban);
    assert!(
        !nav_bar
            .item_for_tab(NavTab::ArticlesKanban)
            .unwrap()
            .is_active
    );
    assert!(nav_bar.item_for_tab(NavTab::TasksKanban).unwrap().is_active);

    // Cycle Next (Tasks -> Contacts)
    event_loop.dispatch(AppMessage::NavigateNextTab).unwrap();
    assert_eq!(event_loop.state().active_tab, NavTab::ContactsDirectory);

    // Cycle Next (Contacts -> Settings)
    event_loop.dispatch(AppMessage::NavigateNextTab).unwrap();
    assert_eq!(event_loop.state().active_tab, NavTab::Settings);

    // Cycle Prev (Settings -> Contacts)
    event_loop.dispatch(AppMessage::NavigatePrevTab).unwrap();
    assert_eq!(event_loop.state().active_tab, NavTab::ContactsDirectory);

    // Handle NavKeyAction::FirstTab (Articles)
    event_loop
        .dispatch(AppMessage::HandleNavKeyAction(NavKeyAction::FirstTab))
        .unwrap();
    assert_eq!(event_loop.state().active_tab, NavTab::ArticlesKanban);
}

#[test]
fn test_cosmic_sidebar_and_view_tree_integration() {
    let state = AppState::in_memory().expect("in-memory state");
    let glass = CosmicFrostedGlass::default();

    let sidebar = CosmicNavBar::new(&state, &glass, false);
    assert_eq!(sidebar.width, COSMIC_SIDEBAR_STANDARD_WIDTH);
    assert_eq!(sidebar.item_height, COSMIC_NAV_ITEM_HEIGHT);
    assert!(!sidebar.is_compact);
    assert_eq!(sidebar.active_tab(), NavTab::ArticlesKanban);

    let compact = CosmicNavBar::new(&state, &glass, true);
    assert_eq!(compact.width, COSMIC_SIDEBAR_COMPACT_WIDTH);
    assert!(compact.is_compact);

    let mut cosmic_app = CosmicApp::new(state);
    let view_tree = cosmic_app.build_view_tree();
    assert_eq!(view_tree.nav_bar.active_tab(), NavTab::ArticlesKanban);
    assert_eq!(view_tree.nav_bar.main_items().len(), 3);
    assert_eq!(view_tree.nav_bar.docked_items().len(), 1);

    // Navigate via action message
    cosmic_app
        .dispatch(AppMessage::NavigateTo(NavTab::Settings))
        .unwrap();
    let updated_tree = cosmic_app.build_view_tree();
    assert_eq!(updated_tree.nav_bar.active_tab(), NavTab::Settings);
    assert!(updated_tree.settings_view.is_some());
}

#[test]
fn test_macos_sidebar_and_view_tree_integration() {
    let state = AppState::in_memory().expect("in-memory state");
    let glass = MacosLiquidGlass::default();
    let vibrancy = MacosVibrancyConfig::default();

    let sidebar = MacosNavBar::new(&state, &glass, &vibrancy, false);
    assert_eq!(sidebar.width, MACOS_SIDEBAR_STANDARD_WIDTH);
    assert_eq!(sidebar.item_height, MACOS_NAV_ITEM_HEIGHT);
    assert_eq!(sidebar.traffic_light_clearance, 76);
    assert!(!sidebar.is_compact);
    assert_eq!(sidebar.active_tab(), NavTab::ArticlesKanban);

    let compact = MacosNavBar::new(&state, &glass, &vibrancy, true);
    assert_eq!(compact.width, MACOS_SIDEBAR_COMPACT_WIDTH);
    assert!(compact.is_compact);

    let mut macos_app = MacosApp::new(state);
    let view_tree = macos_app.build_view_tree();
    assert_eq!(view_tree.nav_bar.active_tab(), NavTab::ArticlesKanban);
    assert_eq!(view_tree.nav_bar.main_items().len(), 3);
    assert_eq!(view_tree.nav_bar.docked_items().len(), 1);

    // Verify macOS shortcut labels
    let articles_item = view_tree
        .nav_bar
        .item_for_tab(NavTab::ArticlesKanban)
        .unwrap();
    assert_eq!(articles_item.shortcut_label, "⌘1");
    let settings_item = view_tree.nav_bar.item_for_tab(NavTab::Settings).unwrap();
    assert_eq!(settings_item.shortcut_label, "⌘4");

    // Navigate via action message
    macos_app
        .dispatch(AppMessage::NavigateTo(NavTab::ContactsDirectory))
        .unwrap();
    let updated_tree = macos_app.build_view_tree();
    assert_eq!(updated_tree.nav_bar.active_tab(), NavTab::ContactsDirectory);
    assert!(updated_tree.contacts_view.is_some());
}
