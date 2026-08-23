//! macOS application wrapper coordinating event loop, windowing, unified toolbar, and liquid glass rendering.

use newsjournal_core::storage::StorageError;
use serde::{Deserialize, Serialize};

use crate::message::AppMessage;
use crate::navigation::NavTab;
use crate::platform::macos::glass::{MacosContainerClass, MacosGlassStyle, MacosLiquidGlass};
use crate::platform::macos::sidebar::MacosNavBar;
use crate::platform::macos::theme::{MacosAppearanceMode, MacosThemeAdapter};
use crate::platform::macos::toolbar::{MacosToolbar, MacosToolbarAction};
use crate::platform::macos::vibrancy::MacosVibrancyConfig;
use crate::platform::macos::window::MacosWindowConfig;
use crate::runtime::EventLoop;
use crate::state::AppState;
use crate::theme::detector::{SystemThemeDetector, SystemThemeWatcher};
use crate::theme::ResolvedTheme;
use crate::views::{
    build_articles_kanban_view, build_contacts_view, build_modal_view, build_nav_view_models,
    build_settings_view_with_platform, build_tasks_kanban_view, build_toast_view,
    ArticleColumnViewModel, ContactListItemViewModel, ModalViewModel, NavItemViewModel,
    SettingsViewModel, TaskColumnViewModel, ToastContainerViewModel,
};

/// High-level presentation descriptor representing the complete macOS Liquid Glass view tree.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MacosViewTreeDescriptor {
    /// Window title formatted for macOS.
    pub window_title: String,
    /// macOS unified toolbar view model.
    pub toolbar: MacosToolbar,
    /// Toolbar liquid glass styling properties.
    pub toolbar_glass_style: MacosGlassStyle,
    /// Left vertical navigation bar component.
    pub nav_bar: MacosNavBar,
    /// Left vertical navigation tabs view models.
    pub nav_items: Vec<NavItemViewModel>,
    /// Left navigation sidebar liquid glass style.
    pub sidebar_glass_style: MacosGlassStyle,
    /// Active tab.
    pub active_tab: NavTab,
    /// Articles Kanban deck (if active tab is ArticlesKanban).
    pub articles_view: Option<Vec<ArticleColumnViewModel>>,
    /// Tasks Kanban deck (if active tab is TasksKanban).
    pub tasks_view: Option<Vec<TaskColumnViewModel>>,
    /// Contacts view (if active tab is ContactsDirectory).
    pub contacts_view: Option<Vec<ContactListItemViewModel>>,
    /// Settings view (if active tab is Settings).
    pub settings_view: Option<SettingsViewModel>,
    /// In-App Modal Sheet / Slide-over Drawer (if a modal is currently open).
    pub modal_view: Option<ModalViewModel>,
    /// Modal liquid glass style (if open).
    pub modal_glass_style: Option<MacosGlassStyle>,
    /// Floating Toast notification container.
    pub toast_view: ToastContainerViewModel,
    /// Active window vibrancy configuration.
    pub vibrancy_config: MacosVibrancyConfig,
}

/// macOS application configuration bundle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MacosAppConfig {
    /// Window geometry and titlebar configuration.
    pub window: MacosWindowConfig,
    /// macOS appearance and accent color adapter.
    pub theme_adapter: MacosThemeAdapter,
    /// Liquid glass materials generator.
    pub liquid_glass: MacosLiquidGlass,
    /// Native window vibrancy configuration.
    pub vibrancy: MacosVibrancyConfig,
}

/// macOS application wrapper and runtime manager.
#[derive(Debug)]
pub struct MacosApp {
    /// Underlying event loop and state manager.
    pub event_loop: EventLoop,
    /// macOS-specific configuration.
    pub config: MacosAppConfig,
    /// macOS unified toolbar component state.
    pub toolbar: MacosToolbar,
    /// Background system appearance watcher.
    pub theme_watcher: SystemThemeWatcher,
    /// Deadline tick interval in seconds (default 60 seconds).
    pub tick_interval_secs: u64,
}

impl MacosApp {
    /// Creates a new `MacosApp` with default configuration from the given application state.
    #[must_use]
    pub fn new(state: AppState) -> Self {
        Self::with_config(state, MacosAppConfig::default())
    }

    /// Creates a new `MacosApp` with custom configuration.
    #[must_use]
    pub fn with_config(state: AppState, config: MacosAppConfig) -> Self {
        let overdue_count = state.deadline_summary.overdue_count;
        let is_dark = state.theme().resolved == ResolvedTheme::Dark;
        let active_tab = state.active_tab;

        let toolbar = MacosToolbar::new(active_tab, overdue_count, is_dark);
        let event_loop = EventLoop::new(state);
        let theme_watcher = SystemThemeWatcher::new(SystemThemeDetector::new());

        Self {
            event_loop,
            config,
            toolbar,
            theme_watcher,
            tick_interval_secs: 60,
        }
    }

    /// Access the underlying state.
    #[must_use]
    pub fn state(&self) -> &AppState {
        self.event_loop.state()
    }

    /// Access the underlying mutable state.
    pub fn state_mut(&mut self) -> &mut AppState {
        self.event_loop.state_mut()
    }

    /// Dispatches an `AppMessage` through the event loop and refreshes toolbar state.
    pub fn dispatch(&mut self, message: AppMessage) -> Result<(), StorageError> {
        self.event_loop.dispatch(message)?;
        self.refresh_toolbar();
        Ok(())
    }

    /// Periodic background tick event (re-evaluates deadlines and system appearance changes).
    pub fn tick(&mut self) -> Result<(), StorageError> {
        if let Some(is_dark) = self.theme_watcher.check_for_change() {
            self.dispatch(AppMessage::SystemThemeChanged(is_dark))?;
        }
        self.dispatch(AppMessage::Tick(chrono::Utc::now()))?;
        Ok(())
    }

    /// Resizes the application window.
    pub fn resize_window(&mut self, width: f32, height: f32) {
        self.config.window.resize(width, height);
    }

    /// Sets the macOS appearance mode and synchronizes with the inner `AppState`.
    pub fn set_appearance(&mut self, mode: MacosAppearanceMode) -> Result<(), StorageError> {
        self.config.theme_adapter.appearance = mode;
        let core_theme = self.config.theme_adapter.to_core_theme_mode();
        self.dispatch(AppMessage::SetThemeMode(core_theme))?;
        Ok(())
    }

    /// Handles actions triggered from the macOS unified toolbar.
    pub fn handle_toolbar_action(
        &mut self,
        action: MacosToolbarAction,
    ) -> Result<(), StorageError> {
        match action {
            MacosToolbarAction::OpenNewArticleModal => {
                self.dispatch(AppMessage::OpenNewArticleModal)?;
            }
            MacosToolbarAction::ToggleSearch => {
                self.toolbar.is_search_active = !self.toolbar.is_search_active;
            }
            MacosToolbarAction::FilterOverdueArticles => {
                self.dispatch(AppMessage::NavigateTo(NavTab::ArticlesKanban))?;
                self.dispatch(AppMessage::SetSearchQuery(String::new()))?;
            }
            MacosToolbarAction::ToggleTheme => {
                self.dispatch(AppMessage::ToggleTheme)?;
            }
            MacosToolbarAction::ToggleSidebar => {
                // Handled in UI layout for compact mode
            }
        }
        Ok(())
    }

    /// Synchronizes the toolbar state with current application state and theme.
    pub fn refresh_toolbar(&mut self) {
        let state = self.event_loop.state();
        let active_tab = state.active_tab;
        let overdue_count = state.deadline_summary.overdue_count;
        let app_theme = state.theme();
        let is_dark = app_theme.resolved == ResolvedTheme::Dark;

        self.config.theme_adapter.appearance = match state.settings.theme_mode {
            newsjournal_core::models::ThemeMode::System => MacosAppearanceMode::System,
            newsjournal_core::models::ThemeMode::Light => MacosAppearanceMode::Aqua,
            newsjournal_core::models::ThemeMode::Dark => MacosAppearanceMode::DarkAqua,
        };
        self.config.theme_adapter.system_is_dark = state.theme_engine.system_is_dark;
        self.toolbar = MacosToolbar::new(active_tab, overdue_count, is_dark);
    }

    /// Constructs the full macOS view tree presentation model.
    #[must_use]
    pub fn build_view_tree(&self) -> MacosViewTreeDescriptor {
        let state = self.event_loop.state();
        let app_theme = state.theme();
        let glass = &self.config.liquid_glass;

        let window_title = self
            .config
            .window
            .format_window_title(state.active_tab, state.deadline_summary.overdue_count);

        let toolbar_glass_style = glass.style_for(MacosContainerClass::Toolbar, app_theme);
        let sidebar_glass_style = glass.style_for(MacosContainerClass::Sidebar, app_theme);

        let nav_items = build_nav_view_models(state);

        let articles_view = if state.active_tab == NavTab::ArticlesKanban {
            Some(build_articles_kanban_view(state))
        } else {
            None
        };

        let tasks_view = if state.active_tab == NavTab::TasksKanban {
            Some(build_tasks_kanban_view(state))
        } else {
            None
        };

        let contacts_view = if state.active_tab == NavTab::ContactsDirectory {
            Some(build_contacts_view(state))
        } else {
            None
        };

        let settings_view = if state.active_tab == NavTab::Settings {
            Some(build_settings_view_with_platform(
                state,
                "macOS (Liquid Glass)",
            ))
        } else {
            None
        };

        let (modal_view, modal_glass_style) = if state.modal.is_open() {
            (
                Some(build_modal_view(state)),
                Some(glass.style_for(MacosContainerClass::ModalDrawer, app_theme)),
            )
        } else {
            (None, None)
        };

        let toast_view = build_toast_view(state);
        let nav_bar = MacosNavBar::new(state, glass, &self.config.vibrancy, false);

        MacosViewTreeDescriptor {
            window_title,
            toolbar: self.toolbar.clone(),
            toolbar_glass_style,
            nav_bar,
            nav_items,
            sidebar_glass_style,
            active_tab: state.active_tab,
            articles_view,
            tasks_view,
            contacts_view,
            settings_view,
            modal_view,
            modal_glass_style,
            toast_view,
            vibrancy_config: self.config.vibrancy.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use newsjournal_core::models::Article;

    #[test]
    fn test_macos_app_initialization_and_view_tree() {
        let state = AppState::in_memory().expect("in-memory state");
        let mut app = MacosApp::new(state);

        assert_eq!(app.state().active_tab, NavTab::ArticlesKanban);
        assert_eq!(app.toolbar.app_title, "NewsJournal");

        let view_tree = app.build_view_tree();
        assert_eq!(view_tree.active_tab, NavTab::ArticlesKanban);
        assert!(view_tree.articles_view.is_some());
        assert!(view_tree.tasks_view.is_none());
        assert!(view_tree.modal_view.is_none());
        assert_eq!(view_tree.nav_items.len(), 4);

        // Test dispatching a message
        let article = Article::new("state-budget", "State Budget Investigative Report");
        app.dispatch(AppMessage::CreateArticle(article))
            .expect("create article");

        assert_eq!(app.state().articles.len(), 1);
        let updated_tree = app.build_view_tree();
        let articles_cols = updated_tree.articles_view.unwrap();
        assert_eq!(articles_cols[0].cards.len(), 1);
    }

    #[test]
    fn test_macos_app_toolbar_action_modal_opening() {
        let state = AppState::in_memory().expect("in-memory state");
        let mut app = MacosApp::new(state);

        app.handle_toolbar_action(MacosToolbarAction::OpenNewArticleModal)
            .expect("open modal");

        assert!(app.state().modal.is_open());
        let tree = app.build_view_tree();
        assert!(tree.modal_view.is_some());
        assert!(tree.modal_glass_style.is_some());
    }

    #[test]
    fn test_macos_app_theme_toggle() {
        let state = AppState::in_memory().expect("in-memory state");
        let mut app = MacosApp::new(state);
        assert_eq!(app.state().resolved_theme(), ResolvedTheme::Dark);

        app.handle_toolbar_action(MacosToolbarAction::ToggleTheme)
            .expect("toggle theme");

        assert_eq!(app.state().resolved_theme(), ResolvedTheme::Light);
        assert_eq!(
            app.state().settings.theme_mode,
            newsjournal_core::models::ThemeMode::Light
        );

        let tree = app.build_view_tree();
        assert_eq!(tree.toolbar.theme_toggle_tooltip, "Switch to Dark Mode");
    }

    #[test]
    fn test_macos_app_dynamic_system_theme_tick() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let flag = Arc::new(AtomicBool::new(true));
        let flag_clone = Arc::clone(&flag);

        let detector =
            SystemThemeDetector::with_custom(move || Some(flag_clone.load(Ordering::SeqCst)));
        let state = AppState::in_memory().expect("in-memory state");
        let mut app = MacosApp::new(state);
        app.theme_watcher = SystemThemeWatcher::new(detector);

        assert_eq!(app.state().resolved_theme(), ResolvedTheme::Dark);

        // System shifts to Light appearance
        flag.store(false, Ordering::SeqCst);
        app.tick().expect("tick successful");

        assert_eq!(app.state().resolved_theme(), ResolvedTheme::Light);
    }

    #[test]
    fn test_macos_app_window_resize() {
        let state = AppState::in_memory().expect("in-memory state");
        let mut app = MacosApp::new(state);

        app.resize_window(1440.0, 900.0);
        assert_eq!(app.config.window.placement.width, 1440.0);
        assert_eq!(app.config.window.placement.height, 900.0);
    }
}
