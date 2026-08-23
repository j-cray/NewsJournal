//! Comprehensive test suite for Settings Page, Theme Selector, Previews, Database Diagnostics, and SQLite Persistence (Task 4.2).

use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use newsjournal_core::models::{Article, Contact, Task, ThemeMode};
use newsjournal_core::storage::StorageService;
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::navigation::NavTab;
use newsjournal_gui::runtime::EventLoop;
use newsjournal_gui::state::modal::{ModalState, SettingsDraft};
use newsjournal_gui::state::AppState;
use newsjournal_gui::theme::ResolvedTheme;
use newsjournal_gui::views::settings::{
    build_settings_view, build_settings_view_with_platform, format_bytes, DatabaseHealthStatus,
    ThemePreviewColors,
};

/// Helper RAII guard to remove temporary test databases on drop.
struct TempDbGuard {
    path: PathBuf,
}

impl TempDbGuard {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("newsjournal_settings_test_{}.db", Uuid::new_v4()));
        Self { path }
    }
}

impl Drop for TempDbGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[test]
fn test_settings_view_model_generation_in_memory() {
    let state = AppState::in_memory().expect("failed to create in-memory state");
    let view = build_settings_view(&state);

    // 1. Theme Configuration
    assert_eq!(view.theme_mode, ThemeMode::System);
    assert!(view.resolved_theme_is_dark);
    assert!(view.system_is_dark);
    assert!(!view.high_contrast);
    assert_eq!(view.custom_accent_hex, None);

    // 2. Theme Options
    assert_eq!(view.theme_options.len(), 3);
    let sys_opt = &view.theme_options[0];
    let light_opt = &view.theme_options[1];
    let dark_opt = &view.theme_options[2];

    assert_eq!(sys_opt.mode, ThemeMode::System);
    assert_eq!(sys_opt.title, "System");
    assert!(sys_opt.is_selected);
    assert!(sys_opt.preview_colors.is_dark);

    assert_eq!(light_opt.mode, ThemeMode::Light);
    assert_eq!(light_opt.title, "Light");
    assert!(!light_opt.is_selected);
    assert!(!light_opt.preview_colors.is_dark);
    assert_eq!(light_opt.preview_colors.background_hex, "#f6f8fa");
    assert_eq!(light_opt.preview_colors.text_hex, "#181c22");

    assert_eq!(dark_opt.mode, ThemeMode::Dark);
    assert_eq!(dark_opt.title, "Dark");
    assert!(!dark_opt.is_selected);
    assert!(dark_opt.preview_colors.is_dark);
    assert_eq!(dark_opt.preview_colors.background_hex, "#121418");
    assert_eq!(dark_opt.preview_colors.text_hex, "#f5f7fa");

    // 3. Database Diagnostics
    assert_eq!(view.database.status, DatabaseHealthStatus::InMemory);
    assert_eq!(view.database.status_label, "In-Memory (Transient)");
    assert_eq!(view.database.status_badge_color, "#1890ff");
    assert_eq!(view.database.path, ":memory:");
    assert!(view.database.is_in_memory);
    assert_eq!(view.database.file_size_bytes, None);
    assert_eq!(view.database.file_size_formatted, "In-Memory Transient");
    assert_eq!(view.database.schema_version, Some(1));
    assert_eq!(view.database.applied_migrations_count, 1);
    assert_eq!(view.database.total_articles, 0);
    assert_eq!(view.database.total_tasks, 0);
    assert_eq!(view.database.total_contacts, 0);

    // 4. Version & Application Info
    assert_eq!(view.app_info.app_name, "NewsJournal");
    assert_eq!(view.app_info.version, newsjournal_core::VERSION);
    assert_eq!(view.app_info.core_version, newsjournal_core::VERSION);
    assert!(!view.app_info.sqlite_version.is_empty());
    assert_eq!(view.version, newsjournal_core::VERSION);
    assert_eq!(view.total_articles, 0);
    assert_eq!(view.total_tasks, 0);
    assert_eq!(view.total_contacts, 0);
}

#[test]
fn test_settings_view_model_generation_file_backed() {
    let guard = TempDbGuard::new();
    let storage = StorageService::open(&guard.path).expect("failed to open file db");
    let mut state = AppState::new(storage);

    // Populate data
    let article = Article::new("city-hall-audit", "City Hall Audit Exposes Waste");
    let article_id = article.id;
    state.storage.create_article(article).unwrap();

    let task1 = Task::new(article_id, "Interview Chief Auditor");
    let task2 = Task::new(article_id, "Analyze Department Ledgers");
    state.storage.create_task(task1).unwrap();
    state.storage.create_task(task2).unwrap();

    let contact = Contact::new("Laura Croft").with_organization("City Controller");
    let contact_id = contact.id;
    state.storage.create_contact(contact).unwrap();
    state
        .storage
        .link_contact_to_article(article_id, contact_id)
        .unwrap();

    state.load_all().unwrap();

    let view = build_settings_view_with_platform(&state, "Linux (COSMIC Frosted Glass)");

    assert_eq!(view.database.status, DatabaseHealthStatus::Connected);
    assert_eq!(view.database.status_label, "Connected (Read/Write)");
    assert_eq!(view.database.status_badge_color, "#52c41a");
    assert!(!view.database.is_in_memory);
    assert_eq!(view.database.path, guard.path.to_string_lossy());
    assert!(view.database.file_size_bytes.is_some());
    assert!(view.database.file_size_bytes.unwrap() > 0);
    assert!(
        view.database.file_size_formatted.contains("KB")
            || view.database.file_size_formatted.contains("B")
    );
    assert_eq!(view.database.schema_version, Some(1));
    assert_eq!(view.database.applied_migrations_count, 1);
    assert_eq!(view.database.total_articles, 1);
    assert_eq!(view.database.total_tasks, 2);
    assert_eq!(view.database.total_contacts, 1);
    assert_eq!(view.database.total_article_contacts, 1);

    assert_eq!(
        view.app_info.platform_target,
        "Linux (COSMIC Frosted Glass)"
    );
}

#[test]
fn test_theme_selection_immediate_preview_and_persistence() {
    let guard = TempDbGuard::new();
    let storage = StorageService::open(&guard.path).expect("failed to open file db");
    let state = AppState::new(storage);
    let mut event_loop = EventLoop::new(state);

    // Initial state: System mode
    assert_eq!(event_loop.state().settings.theme_mode, ThemeMode::System);
    assert_eq!(event_loop.state().resolved_theme(), ResolvedTheme::Dark);

    // 1. Select Light theme mode
    event_loop
        .dispatch(AppMessage::SetThemeMode(ThemeMode::Light))
        .unwrap();

    // Verify immediate in-memory state and preview update
    assert_eq!(event_loop.state().settings.theme_mode, ThemeMode::Light);
    assert_eq!(event_loop.state().resolved_theme(), ResolvedTheme::Light);

    let view_light = build_settings_view(event_loop.state());
    assert_eq!(view_light.theme_mode, ThemeMode::Light);
    assert!(!view_light.resolved_theme_is_dark);
    assert!(view_light.theme_options[1].is_selected);
    assert!(!view_light.theme_options[0].is_selected);
    assert!(!view_light.theme_options[2].is_selected);

    // Verify immediate persistence in SQLite database
    let persisted_settings = event_loop.state().storage.get_settings().unwrap();
    assert_eq!(persisted_settings.theme_mode, ThemeMode::Light);

    // 2. Select Dark theme mode
    event_loop
        .dispatch(AppMessage::SetThemeMode(ThemeMode::Dark))
        .unwrap();

    assert_eq!(event_loop.state().settings.theme_mode, ThemeMode::Dark);
    assert_eq!(event_loop.state().resolved_theme(), ResolvedTheme::Dark);

    let view_dark = build_settings_view(event_loop.state());
    assert_eq!(view_dark.theme_mode, ThemeMode::Dark);
    assert!(view_dark.resolved_theme_is_dark);
    assert!(view_dark.theme_options[2].is_selected);
    assert!(!view_dark.theme_options[0].is_selected);
    assert!(!view_dark.theme_options[1].is_selected);

    let persisted_dark = event_loop.state().storage.get_settings().unwrap();
    assert_eq!(persisted_dark.theme_mode, ThemeMode::Dark);

    // 3. Toggle theme mode
    event_loop.dispatch(AppMessage::ToggleTheme).unwrap();
    assert_eq!(event_loop.state().settings.theme_mode, ThemeMode::Light);
    assert_eq!(event_loop.state().resolved_theme(), ResolvedTheme::Light);

    let persisted_toggled = event_loop.state().storage.get_settings().unwrap();
    assert_eq!(persisted_toggled.theme_mode, ThemeMode::Light);
}

#[test]
fn test_settings_persistence_across_app_reloads() {
    let guard = TempDbGuard::new();

    // Session 1: configure settings and mutate
    {
        let storage = StorageService::open(&guard.path).expect("open session 1");
        let state = AppState::new(storage);
        let mut event_loop = EventLoop::new(state);

        event_loop
            .dispatch(AppMessage::SetThemeMode(ThemeMode::Dark))
            .unwrap();
        assert_eq!(event_loop.state().settings.theme_mode, ThemeMode::Dark);
    }

    // Session 2: re-open fresh AppState from same storage
    {
        let storage2 = StorageService::open(&guard.path).expect("open session 2");
        let mut state2 = AppState::new(storage2);
        state2.load_all().unwrap();

        assert_eq!(state2.settings.theme_mode, ThemeMode::Dark);
        assert_eq!(state2.resolved_theme(), ResolvedTheme::Dark);

        let view = build_settings_view(&state2);
        assert_eq!(view.theme_mode, ThemeMode::Dark);
        assert!(view.theme_options[2].is_selected);
    }
}

#[test]
fn test_settings_drawer_workflow_and_reducer() {
    let guard = TempDbGuard::new();
    let storage = StorageService::open(&guard.path).expect("open db");
    let state = AppState::new(storage);
    let mut event_loop = EventLoop::new(state);

    // 1. Open Settings drawer
    event_loop.dispatch(AppMessage::OpenSettings).unwrap();
    match &event_loop.state().modal {
        ModalState::SettingsDrawer(draft) => {
            assert_eq!(draft.theme_mode, ThemeMode::System);
            assert!(!draft.high_contrast);
        }
        other => panic!("Expected ModalState::SettingsDrawer, got {other:?}"),
    }

    // 2. Update Draft
    let updated_draft = SettingsDraft {
        theme_mode: ThemeMode::Light,
        high_contrast: true,
        custom_accent: Some((30, 160, 240)),
    };
    event_loop
        .dispatch(AppMessage::UpdateSettingsDraft(updated_draft))
        .unwrap();

    // 3. Submit Modal
    event_loop.dispatch(AppMessage::SubmitModal).unwrap();

    // Verify settings applied
    assert_eq!(event_loop.state().settings.theme_mode, ThemeMode::Light);
    assert_eq!(event_loop.state().resolved_theme(), ResolvedTheme::Light);
    assert!(event_loop.state().theme_engine.high_contrast);
    assert_eq!(
        event_loop.state().theme_engine.custom_accent,
        Some((30, 160, 240))
    );

    // Verify persisted in SQLite
    let persisted = event_loop.state().storage.get_settings().unwrap();
    assert_eq!(persisted.theme_mode, ThemeMode::Light);

    // Verify notification toast emitted
    assert!(!event_loop.state().toasts.is_empty());
    assert_eq!(
        event_loop.state().toasts.last().unwrap().title,
        "Settings Updated"
    );
}

#[test]
fn test_reset_settings_to_defaults() {
    let guard = TempDbGuard::new();
    let storage = StorageService::open(&guard.path).expect("open db");
    let state = AppState::new(storage);
    let mut event_loop = EventLoop::new(state);

    // Customize
    event_loop
        .dispatch(AppMessage::SetThemeMode(ThemeMode::Dark))
        .unwrap();
    event_loop
        .dispatch(AppMessage::SetHighContrast(true))
        .unwrap();
    event_loop
        .dispatch(AppMessage::SetCustomAccent(Some((200, 50, 50))))
        .unwrap();

    assert_eq!(event_loop.state().settings.theme_mode, ThemeMode::Dark);
    assert!(event_loop.state().theme_engine.high_contrast);
    assert_eq!(
        event_loop.state().theme_engine.custom_accent,
        Some((200, 50, 50))
    );

    // Reset
    event_loop
        .dispatch(AppMessage::ResetSettingsToDefaults)
        .unwrap();

    assert_eq!(event_loop.state().settings.theme_mode, ThemeMode::System);
    assert!(!event_loop.state().theme_engine.high_contrast);
    assert_eq!(event_loop.state().theme_engine.custom_accent, None);

    let persisted = event_loop.state().storage.get_settings().unwrap();
    assert_eq!(persisted.theme_mode, ThemeMode::System);

    assert_eq!(
        event_loop.state().toasts.last().unwrap().title,
        "Settings Reset"
    );
}

#[test]
fn test_format_bytes_utility() {
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(512), "512 B");
    assert_eq!(format_bytes(1024), "1.0 KB");
    assert_eq!(format_bytes(1536), "1.5 KB");
    assert_eq!(format_bytes(1048576), "1.0 MB");
    assert_eq!(format_bytes(2621440), "2.5 MB");
    assert_eq!(format_bytes(1073741824), "1.00 GB");
}

#[test]
fn test_theme_options_preview_colors_consistency() {
    // Light mode preview
    let light_preview = ThemePreviewColors::for_mode(ThemeMode::Light, true, None);
    assert!(!light_preview.is_dark);
    assert_eq!(light_preview.background_hex, "#f6f8fa");
    assert_eq!(light_preview.text_hex, "#181c22");
    assert_eq!(light_preview.accent_hex, "#1890ff");

    // Dark mode preview
    let dark_preview = ThemePreviewColors::for_mode(ThemeMode::Dark, false, None);
    assert!(dark_preview.is_dark);
    assert_eq!(dark_preview.background_hex, "#121418");
    assert_eq!(dark_preview.text_hex, "#f5f7fa");
    assert_eq!(dark_preview.accent_hex, "#409eff");

    // Custom accent preview
    let custom_accent_preview =
        ThemePreviewColors::for_mode(ThemeMode::Dark, true, Some((255, 0, 128)));
    assert_eq!(custom_accent_preview.accent_hex, "#ff0080");

    // System mode preview mirroring system appearance
    let sys_dark = ThemePreviewColors::for_mode(ThemeMode::System, true, None);
    assert!(sys_dark.is_dark);
    let sys_light = ThemePreviewColors::for_mode(ThemeMode::System, false, None);
    assert!(!sys_light.is_dark);
}

#[test]
fn test_navigation_to_settings_tab_view_tree() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    event_loop
        .dispatch(AppMessage::NavigateTo(NavTab::Settings))
        .unwrap();
    assert_eq!(event_loop.state().active_tab, NavTab::Settings);

    let view = build_settings_view(event_loop.state());
    assert_eq!(view.theme_mode, ThemeMode::System);
}
