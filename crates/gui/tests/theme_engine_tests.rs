use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use newsjournal_core::models::ThemeMode;
use newsjournal_gui::cosmic::CosmicApp;
use newsjournal_gui::macos::MacosApp;
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::theme::detector::{SystemThemeDetector, SystemThemeWatcher};
use newsjournal_gui::theme::{ResolvedTheme, ThemeEngine};
use newsjournal_gui::{AppState, EventLoop};
use proptest::prelude::*;

#[test]
fn test_system_theme_detector_mock_and_env() {
    // Static detector
    let dark_detector = SystemThemeDetector::with_static(true);
    assert!(dark_detector.is_dark());
    assert_eq!(dark_detector.detect_theme(), ResolvedTheme::Dark);

    let light_detector = SystemThemeDetector::with_static(false);
    assert!(!light_detector.is_dark());
    assert_eq!(light_detector.detect_theme(), ResolvedTheme::Light);

    // Dynamic custom detector
    let is_dark_flag = Arc::new(AtomicBool::new(true));
    let flag_clone = Arc::clone(&is_dark_flag);
    let dynamic_detector =
        SystemThemeDetector::with_custom(move || Some(flag_clone.load(Ordering::SeqCst)));

    assert!(dynamic_detector.is_dark());
    is_dark_flag.store(false, Ordering::SeqCst);
    assert!(!dynamic_detector.is_dark());
}

#[test]
fn test_system_theme_watcher_lifecycle() {
    let is_dark_flag = Arc::new(AtomicBool::new(true));
    let flag_clone = Arc::clone(&is_dark_flag);
    let detector =
        SystemThemeDetector::with_custom(move || Some(flag_clone.load(Ordering::SeqCst)));

    let mut watcher = SystemThemeWatcher::new(detector);
    assert!(watcher.last_is_dark);

    // Initial check: no change
    assert_eq!(watcher.check_for_change(), None);

    // Change to light
    is_dark_flag.store(false, Ordering::SeqCst);
    assert_eq!(watcher.check_for_change(), Some(false));
    assert_eq!(watcher.check_for_change(), None);

    // Change back to dark
    is_dark_flag.store(true, Ordering::SeqCst);
    assert_eq!(watcher.check_for_change(), Some(true));
    assert_eq!(watcher.check_for_change(), None);
}

#[test]
fn test_theme_engine_mode_resolution_and_overrides() {
    // Default system mode with system reporting dark
    let mut engine = ThemeEngine::new(ThemeMode::System, true);
    assert_eq!(engine.resolved(), ResolvedTheme::Dark);
    assert!(engine.is_dark());

    // System mode with system reporting light
    let changed = engine.set_system_is_dark(false);
    assert!(changed);
    assert_eq!(engine.resolved(), ResolvedTheme::Light);
    assert!(!engine.is_dark());

    // Override with explicit Dark mode
    let changed = engine.set_mode(ThemeMode::Dark);
    assert!(changed);
    assert_eq!(engine.resolved(), ResolvedTheme::Dark);

    // System changes to light, but mode is forced Dark => no change to resolved theme
    let changed = engine.set_system_is_dark(false);
    assert!(!changed);
    assert_eq!(engine.resolved(), ResolvedTheme::Dark);

    // Override with explicit Light mode
    let changed = engine.set_mode(ThemeMode::Light);
    assert!(changed);
    assert_eq!(engine.resolved(), ResolvedTheme::Light);

    // Switch back to System mode while OS is light => resolved remains Light
    let changed = engine.set_mode(ThemeMode::System);
    assert!(!changed);
    assert_eq!(engine.resolved(), ResolvedTheme::Light);

    // System changes to dark while in System mode => resolved becomes Dark
    let changed = engine.set_system_is_dark(true);
    assert!(changed);
    assert_eq!(engine.resolved(), ResolvedTheme::Dark);
}

#[test]
fn test_theme_engine_toggle_mode() {
    let mut engine = ThemeEngine::new(ThemeMode::System, true);
    assert_eq!(engine.resolved(), ResolvedTheme::Dark);

    // Toggling from System (Dark) gives explicit Light
    let next = engine.toggle_mode();
    assert_eq!(next, ThemeMode::Light);
    assert_eq!(engine.resolved(), ResolvedTheme::Light);

    // Toggling from Light gives Dark
    let next = engine.toggle_mode();
    assert_eq!(next, ThemeMode::Dark);
    assert_eq!(engine.resolved(), ResolvedTheme::Dark);

    // Toggling from Dark gives Light
    let next = engine.toggle_mode();
    assert_eq!(next, ThemeMode::Light);
    assert_eq!(engine.resolved(), ResolvedTheme::Light);
}

#[test]
fn test_theme_engine_custom_accent_and_high_contrast() {
    let mut engine = ThemeEngine::new(ThemeMode::Dark, true);
    let default_accent = engine.colors().accent;

    let cyan = (54, 207, 201);
    engine.set_custom_accent(Some(cyan));
    assert_eq!(engine.colors().accent, cyan);

    engine.set_high_contrast(true);
    assert_eq!(engine.colors().text_primary, (255, 255, 255));
    assert_eq!(engine.colors().accent, cyan); // Custom accent preserved

    engine.set_custom_accent(None);
    assert_eq!(engine.colors().accent, default_accent);
}

#[test]
fn test_theme_engine_wcag_contrast_ratios() {
    let dark_engine = ThemeEngine::new(ThemeMode::Dark, true);
    let light_engine = ThemeEngine::new(ThemeMode::Light, false);

    let dark_ratio = dark_engine.text_contrast_ratio();
    let light_ratio = light_engine.text_contrast_ratio();

    // WCAG AAA requires >= 7.0:1 for standard text
    assert!(
        dark_ratio >= 7.0,
        "Dark theme text contrast {dark_ratio:.2} must meet WCAG AAA (>= 7.0)"
    );
    assert!(
        light_ratio >= 7.0,
        "Light theme text contrast {light_ratio:.2} must meet WCAG AAA (>= 7.0)"
    );
}

#[test]
fn test_event_loop_theme_dispatch_and_sqlite_persistence() {
    let state = AppState::in_memory().expect("in-memory state");
    let storage = state.storage.clone();
    let mut event_loop = EventLoop::new(state);

    assert_eq!(event_loop.state().settings.theme_mode, ThemeMode::System);
    assert_eq!(event_loop.state().resolved_theme(), ResolvedTheme::Dark);

    // 1. Dispatch SetThemeMode(Light)
    event_loop
        .dispatch(AppMessage::SetThemeMode(ThemeMode::Light))
        .unwrap();

    assert_eq!(event_loop.state().settings.theme_mode, ThemeMode::Light);
    assert_eq!(event_loop.state().resolved_theme(), ResolvedTheme::Light);

    // Verify persisted in SQLite
    let persisted = storage.get_settings().unwrap();
    assert_eq!(persisted.theme_mode, ThemeMode::Light);

    // 2. Reload state from storage
    let mut new_state = AppState::new(storage.clone());
    new_state.load_all().unwrap();
    assert_eq!(new_state.settings.theme_mode, ThemeMode::Light);
    assert_eq!(new_state.resolved_theme(), ResolvedTheme::Light);

    // 3. Dispatch ToggleTheme
    let mut new_event_loop = EventLoop::new(new_state);
    new_event_loop.dispatch(AppMessage::ToggleTheme).unwrap();

    assert_eq!(new_event_loop.state().settings.theme_mode, ThemeMode::Dark);
    assert_eq!(new_event_loop.state().resolved_theme(), ResolvedTheme::Dark);

    let persisted = storage.get_settings().unwrap();
    assert_eq!(persisted.theme_mode, ThemeMode::Dark);

    // 4. Dispatch SystemThemeChanged when in System mode
    new_event_loop
        .dispatch(AppMessage::SetThemeMode(ThemeMode::System))
        .unwrap();
    assert_eq!(new_event_loop.state().resolved_theme(), ResolvedTheme::Dark);

    new_event_loop
        .dispatch(AppMessage::SystemThemeChanged(false))
        .unwrap();
    assert_eq!(
        new_event_loop.state().resolved_theme(),
        ResolvedTheme::Light
    );
    // User preference in settings remains System
    assert_eq!(
        new_event_loop.state().settings.theme_mode,
        ThemeMode::System
    );
}

#[test]
fn test_cosmic_app_theme_engine_synchronization() {
    let is_dark_flag = Arc::new(AtomicBool::new(true));
    let flag_clone = Arc::clone(&is_dark_flag);
    let detector =
        SystemThemeDetector::with_custom(move || Some(flag_clone.load(Ordering::SeqCst)));

    let state = AppState::in_memory().expect("in-memory state");
    let mut app = CosmicApp::new(state);
    app.theme_watcher = SystemThemeWatcher::new(detector);

    assert_eq!(app.state().resolved_theme(), ResolvedTheme::Dark);
    assert_eq!(
        app.build_view_tree().header_bar.theme_toggle_tooltip,
        "Switch to Light Mode"
    );

    // OS appearance shifts to Light
    is_dark_flag.store(false, Ordering::SeqCst);
    app.tick().unwrap();

    assert_eq!(app.state().resolved_theme(), ResolvedTheme::Light);
    assert_eq!(
        app.build_view_tree().header_bar.theme_toggle_tooltip,
        "Switch to Dark Mode"
    );

    // Explicitly set theme mode via CosmicApp method
    app.set_theme_mode(newsjournal_gui::cosmic::CosmicThemeMode::Dark)
        .unwrap();
    assert_eq!(app.state().resolved_theme(), ResolvedTheme::Dark);
    assert_eq!(
        app.build_view_tree().header_bar.theme_toggle_tooltip,
        "Switch to Light Mode"
    );
}

#[test]
fn test_macos_app_theme_engine_synchronization() {
    let is_dark_flag = Arc::new(AtomicBool::new(true));
    let flag_clone = Arc::clone(&is_dark_flag);
    let detector =
        SystemThemeDetector::with_custom(move || Some(flag_clone.load(Ordering::SeqCst)));

    let state = AppState::in_memory().expect("in-memory state");
    let mut app = MacosApp::new(state);
    app.theme_watcher = SystemThemeWatcher::new(detector);

    assert_eq!(app.state().resolved_theme(), ResolvedTheme::Dark);
    assert_eq!(
        app.build_view_tree().toolbar.theme_toggle_tooltip,
        "Switch to Light Mode"
    );

    // OS appearance shifts to Light
    is_dark_flag.store(false, Ordering::SeqCst);
    app.tick().unwrap();

    assert_eq!(app.state().resolved_theme(), ResolvedTheme::Light);
    assert_eq!(
        app.build_view_tree().toolbar.theme_toggle_tooltip,
        "Switch to Dark Mode"
    );

    // Explicitly set appearance via MacosApp method
    app.set_appearance(newsjournal_gui::macos::MacosAppearanceMode::DarkAqua)
        .unwrap();
    assert_eq!(app.state().resolved_theme(), ResolvedTheme::Dark);
    assert_eq!(
        app.build_view_tree().toolbar.theme_toggle_tooltip,
        "Switch to Light Mode"
    );
}

// Property-based testing
proptest! {
    #[test]
    fn prop_theme_engine_resolution_consistency(
        is_system_dark in any::<bool>(),
        mode_idx in 0..3usize,
        high_contrast in any::<bool>(),
    ) {
        let mode = match mode_idx {
            0 => ThemeMode::System,
            1 => ThemeMode::Light,
            _ => ThemeMode::Dark,
        };

        let mut engine = ThemeEngine::new(mode, is_system_dark);
        engine.set_high_contrast(high_contrast);

        let expected_resolved = match mode {
            ThemeMode::System => {
                if is_system_dark {
                    ResolvedTheme::Dark
                } else {
                    ResolvedTheme::Light
                }
            }
            ThemeMode::Light => ResolvedTheme::Light,
            ThemeMode::Dark => ResolvedTheme::Dark,
        };

        prop_assert_eq!(engine.resolved(), expected_resolved);
        prop_assert_eq!(engine.is_dark(), expected_resolved == ResolvedTheme::Dark);

        // Contrast ratio must always be accessible (>= 4.5 minimum for AA, >= 7.0 for AAA)
        let contrast = engine.text_contrast_ratio();
        prop_assert!(contrast >= 7.0);
    }
}
