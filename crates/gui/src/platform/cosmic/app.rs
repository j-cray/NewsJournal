//! COSMIC application wrapper coordinating event loop, windowing, header bar, and frosted glass rendering.

use newsjournal_core::storage::StorageError;
use serde::{Deserialize, Serialize};

use crate::message::AppMessage;
use crate::navigation::NavTab;
use crate::platform::cosmic::glass::{CosmicContainerClass, CosmicFrostedGlass, CosmicGlassStyle};
use crate::platform::cosmic::header_bar::{CosmicHeaderBar, CosmicHeaderBarAction};
use crate::platform::cosmic::icon::CosmicIconManager;
use crate::platform::cosmic::sidebar::CosmicNavBar;
use crate::platform::cosmic::theme::{CosmicThemeAdapter, CosmicThemeMode};
use crate::platform::cosmic::window::CosmicWindowConfig;
use crate::runtime::EventLoop;
use crate::state::AppState;
use crate::theme::detector::{SystemThemeDetector, SystemThemeWatcher};
use crate::theme::ResolvedTheme;
use crate::views::{
    build_articles_kanban_deck, build_contacts_directory_view, build_contacts_view,
    build_modal_container_view, build_modal_view, build_nav_view_models,
    build_settings_view_with_platform, build_tasks_kanban_deck, build_toast_view,
    ArticleColumnViewModel, ArticlesKanbanDeckViewModel, ContactListItemViewModel,
    ContactsDirectoryViewModel, ModalContainerViewModel, ModalViewModel, NavItemViewModel,
    SettingsViewModel, TaskColumnViewModel, TasksKanbanDeckViewModel, ToastContainerViewModel,
};

/// High-level presentation descriptor representing the complete COSMIC view tree.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CosmicViewTreeDescriptor {
    /// Window title formatted for COSMIC shell.
    pub window_title: String,
    /// Header bar view model.
    pub header_bar: CosmicHeaderBar,
    /// Header bar frosted glass style.
    pub header_glass_style: CosmicGlassStyle,
    /// Left vertical navigation bar component.
    pub nav_bar: CosmicNavBar,
    /// Left vertical navigation tabs view models.
    pub nav_items: Vec<NavItemViewModel>,
    /// Left navigation sidebar frosted glass style.
    pub sidebar_glass_style: CosmicGlassStyle,
    /// Kanban column frosted glass style.
    pub column_glass_style: CosmicGlassStyle,
    /// Active tab.
    pub active_tab: NavTab,
    /// Articles Kanban deck (if active tab is ArticlesKanban).
    pub articles_deck: Option<ArticlesKanbanDeckViewModel>,
    /// Articles Kanban columns list (if active tab is ArticlesKanban).
    pub articles_view: Option<Vec<ArticleColumnViewModel>>,
    /// Tasks Kanban deck (if active tab is TasksKanban).
    pub tasks_deck: Option<TasksKanbanDeckViewModel>,
    /// Tasks Kanban columns list (if active tab is TasksKanban).
    pub tasks_view: Option<Vec<TaskColumnViewModel>>,
    /// Contacts view (if active tab is ContactsDirectory).
    pub contacts_view: Option<Vec<ContactListItemViewModel>>,
    /// Contacts directory complete view model (if active tab is ContactsDirectory).
    pub contacts_directory: Option<ContactsDirectoryViewModel>,
    /// Settings view (if active tab is Settings).
    pub settings_view: Option<SettingsViewModel>,
    /// In-App Modal / Slide-over Drawer (if a modal is currently open).
    pub modal_view: Option<ModalViewModel>,
    /// In-App Modal / Slide-over Drawer Container (if a modal is currently open).
    pub modal_container: Option<ModalContainerViewModel>,
    /// Modal frosted glass style (if open).
    pub modal_glass_style: Option<CosmicGlassStyle>,
    /// Floating Toast notification container.
    pub toast_view: ToastContainerViewModel,
}

/// COSMIC application configuration bundle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CosmicAppConfig {
    /// Window geometry and state configuration.
    pub window: CosmicWindowConfig,
    /// COSMIC theme adapter.
    pub theme_adapter: CosmicThemeAdapter,
    /// Frosted glass materials settings.
    pub glass: CosmicFrostedGlass,
}

/// COSMIC application wrapper and runtime manager.
#[derive(Debug)]
pub struct CosmicApp {
    /// Underlying event loop and state manager.
    pub event_loop: EventLoop,
    /// COSMIC-specific configuration.
    pub config: CosmicAppConfig,
    /// COSMIC header bar component state.
    pub header_bar: CosmicHeaderBar,
    /// COSMIC icon manager.
    pub icon_manager: CosmicIconManager,
    /// Background system appearance watcher.
    pub theme_watcher: SystemThemeWatcher,
    /// Deadline tick interval in seconds (default 60 seconds).
    pub tick_interval_secs: u64,
}

impl CosmicApp {
    /// Creates a new `CosmicApp` with default configuration from the given application state.
    #[must_use]
    pub fn new(state: AppState) -> Self {
        Self::with_config(state, CosmicAppConfig::default())
    }

    /// Creates a new `CosmicApp` with custom configuration.
    #[must_use]
    pub fn with_config(state: AppState, config: CosmicAppConfig) -> Self {
        let overdue_count = state.deadline_summary.overdue_count;
        let is_dark = state.theme().resolved == ResolvedTheme::Dark;
        let active_tab = state.active_tab;

        let header_bar = CosmicHeaderBar::new(active_tab, overdue_count, is_dark);
        let event_loop = EventLoop::new(state);
        let icon_manager = CosmicIconManager::new();
        let theme_watcher = SystemThemeWatcher::new(SystemThemeDetector::new());

        Self {
            event_loop,
            config,
            header_bar,
            icon_manager,
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

    /// Dispatches an `AppMessage` through the event loop and refreshes header bar state.
    pub fn dispatch(&mut self, message: AppMessage) -> Result<(), StorageError> {
        self.event_loop.dispatch(message)?;
        self.refresh_header_bar();
        Ok(())
    }

    /// Dispatches a keyboard shortcut key event through the application shortcut resolver.
    /// Returns `Ok(true)` if a shortcut was resolved and dispatched, or `Ok(false)` otherwise.
    pub fn handle_key_event(
        &mut self,
        key: &str,
        modifiers: crate::navigation::NavKeyModifiers,
    ) -> Result<bool, StorageError> {
        let is_macos = false;
        if let Some(msg) =
            crate::navigation::resolve_app_shortcut(key, modifiers, is_macos, self.state())
        {
            self.dispatch(msg)?;
            Ok(true)
        } else {
            Ok(false)
        }
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

    /// Sets the COSMIC theme mode and synchronizes with the inner `AppState`.
    pub fn set_theme_mode(&mut self, mode: CosmicThemeMode) -> Result<(), StorageError> {
        self.config.theme_adapter.mode = mode;
        let core_theme = self.config.theme_adapter.to_core_theme_mode();
        self.dispatch(AppMessage::SetThemeMode(core_theme))?;
        Ok(())
    }

    /// Handles actions triggered from the COSMIC header bar.
    pub fn handle_header_action(
        &mut self,
        action: CosmicHeaderBarAction,
    ) -> Result<(), StorageError> {
        match action {
            CosmicHeaderBarAction::OpenNewArticleModal => {
                self.dispatch(AppMessage::OpenNewArticleModal)?;
            }
            CosmicHeaderBarAction::OpenNewTaskModal => {
                self.dispatch(AppMessage::OpenNewTaskModal(None))?;
            }
            CosmicHeaderBarAction::OpenNewContactModal => {
                self.dispatch(AppMessage::OpenNewContactModal)?;
            }
            CosmicHeaderBarAction::PrimaryAction => match self.state().active_tab {
                NavTab::ArticlesKanban => {
                    self.dispatch(AppMessage::OpenNewArticleModal)?;
                }
                NavTab::TasksKanban => {
                    self.dispatch(AppMessage::OpenNewTaskModal(None))?;
                }
                NavTab::ContactsDirectory => {
                    self.dispatch(AppMessage::OpenNewContactModal)?;
                }
                NavTab::Settings => {
                    self.dispatch(AppMessage::SaveSettings(self.state().settings.clone()))?;
                }
            },
            CosmicHeaderBarAction::ToggleSearch => {
                self.header_bar.is_search_active = !self.header_bar.is_search_active;
            }
            CosmicHeaderBarAction::FilterOverdueArticles => {
                // Focus on overdue articles
                self.dispatch(AppMessage::NavigateTo(NavTab::ArticlesKanban))?;
                self.dispatch(AppMessage::SetSearchQuery(String::new()))?;
            }
            CosmicHeaderBarAction::ToggleTheme => {
                self.dispatch(AppMessage::ToggleTheme)?;
            }
            CosmicHeaderBarAction::ToggleSidebar => {
                // Handled in UI layout for compact mode
            }
        }
        Ok(())
    }

    /// Synchronizes the header bar state with current application state and theme.
    pub fn refresh_header_bar(&mut self) {
        let state = self.event_loop.state();
        let active_tab = state.active_tab;
        let overdue_count = state.deadline_summary.overdue_count;
        let app_theme = state.theme();
        let is_dark = app_theme.resolved == ResolvedTheme::Dark;

        self.config.theme_adapter.mode = match state.settings.theme_mode {
            newsjournal_core::models::ThemeMode::System => CosmicThemeMode::System,
            newsjournal_core::models::ThemeMode::Light => CosmicThemeMode::Light,
            newsjournal_core::models::ThemeMode::Dark => CosmicThemeMode::Dark,
        };
        self.config.theme_adapter.system_is_dark = state.theme_engine.system_is_dark;
        self.header_bar = CosmicHeaderBar::new(active_tab, overdue_count, is_dark);
    }

    /// Constructs the full COSMIC view tree presentation model.
    #[must_use]
    pub fn build_view_tree(&self) -> CosmicViewTreeDescriptor {
        let state = self.event_loop.state();
        let app_theme = state.theme();
        let glass = &self.config.glass;

        let window_title = self
            .config
            .window
            .format_window_title(state.active_tab, state.deadline_summary.overdue_count);

        let header_glass_style = glass.style_for(CosmicContainerClass::HeaderBar, app_theme);
        let sidebar_glass_style = glass.style_for(CosmicContainerClass::Sidebar, app_theme);
        let column_glass_style = glass.style_for(CosmicContainerClass::KanbanColumn, app_theme);

        let nav_items = build_nav_view_models(state);

        let (articles_view, articles_deck) = if state.active_tab == NavTab::ArticlesKanban {
            let deck = build_articles_kanban_deck(state);
            (Some(deck.columns.clone()), Some(deck))
        } else {
            (None, None)
        };

        let (tasks_view, tasks_deck) = if state.active_tab == NavTab::TasksKanban {
            let deck = build_tasks_kanban_deck(state);
            (Some(deck.columns.clone()), Some(deck))
        } else {
            (None, None)
        };

        let (contacts_view, contacts_directory) = if state.active_tab == NavTab::ContactsDirectory {
            (
                Some(build_contacts_view(state)),
                Some(build_contacts_directory_view(state)),
            )
        } else {
            (None, None)
        };

        let settings_view = if state.active_tab == NavTab::Settings {
            Some(build_settings_view_with_platform(
                state,
                "Linux (COSMIC Frosted Glass)",
            ))
        } else {
            None
        };

        let (modal_view, modal_container, modal_glass_style) = if state.modal.is_open() {
            (
                Some(build_modal_view(state)),
                Some(build_modal_container_view(state)),
                Some(glass.style_for(CosmicContainerClass::ModalDrawer, app_theme)),
            )
        } else {
            (None, None, None)
        };

        let toast_view = build_toast_view(state);
        let nav_bar = CosmicNavBar::new(state, glass, false);

        CosmicViewTreeDescriptor {
            window_title,
            header_bar: self.header_bar.clone(),
            header_glass_style,
            nav_bar,
            nav_items,
            sidebar_glass_style,
            column_glass_style,
            active_tab: state.active_tab,
            articles_deck,
            articles_view,
            tasks_deck,
            tasks_view,
            contacts_view,
            contacts_directory,
            settings_view,
            modal_view,
            modal_container,
            modal_glass_style,
            toast_view,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use newsjournal_core::models::Article;

    #[test]
    fn test_cosmic_app_initialization_and_view_tree() {
        let state = AppState::in_memory().expect("in-memory state");
        let mut app = CosmicApp::new(state);

        assert_eq!(app.state().active_tab, NavTab::ArticlesKanban);
        assert_eq!(app.header_bar.app_title, "NewsJournal");

        let view_tree = app.build_view_tree();
        assert_eq!(view_tree.active_tab, NavTab::ArticlesKanban);
        assert!(view_tree.articles_view.is_some());
        assert!(view_tree.tasks_view.is_none());
        assert!(view_tree.modal_view.is_none());
        assert_eq!(view_tree.nav_items.len(), 4);

        // Test dispatching a message
        let article = Article::new("green-energy", "Green Energy Transition");
        app.dispatch(AppMessage::CreateArticle(article))
            .expect("create article");

        assert_eq!(app.state().articles.len(), 1);
        let updated_tree = app.build_view_tree();
        let articles_cols = updated_tree.articles_view.unwrap();
        // Pitching column has 1 article card
        assert_eq!(articles_cols[0].cards.len(), 1);
    }

    #[test]
    fn test_cosmic_app_header_action_modal_opening() {
        let state = AppState::in_memory().expect("in-memory state");
        let mut app = CosmicApp::new(state);

        app.handle_header_action(CosmicHeaderBarAction::OpenNewArticleModal)
            .expect("open modal");

        assert!(app.state().modal.is_open());
        let tree = app.build_view_tree();
        assert!(tree.modal_view.is_some());
        assert!(tree.modal_glass_style.is_some());
    }

    #[test]
    fn test_cosmic_app_theme_toggle() {
        let state = AppState::in_memory().expect("in-memory state");
        let mut app = CosmicApp::new(state);
        assert_eq!(app.state().resolved_theme(), ResolvedTheme::Dark);

        app.handle_header_action(CosmicHeaderBarAction::ToggleTheme)
            .expect("toggle theme");

        assert_eq!(app.state().resolved_theme(), ResolvedTheme::Light);
        assert_eq!(
            app.state().settings.theme_mode,
            newsjournal_core::models::ThemeMode::Light
        );

        let tree = app.build_view_tree();
        assert_eq!(tree.header_bar.theme_toggle_tooltip, "Switch to Dark Mode");
    }

    #[test]
    fn test_cosmic_app_dynamic_system_theme_tick() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let flag = Arc::new(AtomicBool::new(true));
        let flag_clone = Arc::clone(&flag);

        let detector =
            SystemThemeDetector::with_custom(move || Some(flag_clone.load(Ordering::SeqCst)));
        let state = AppState::in_memory().expect("in-memory state");
        let mut app = CosmicApp::new(state);
        app.theme_watcher = SystemThemeWatcher::new(detector);

        assert_eq!(app.state().resolved_theme(), ResolvedTheme::Dark);

        // System shifts to Light appearance
        flag.store(false, Ordering::SeqCst);
        app.tick().expect("tick successful");

        assert_eq!(app.state().resolved_theme(), ResolvedTheme::Light);
    }

    #[test]
    fn test_cosmic_app_window_resize() {
        let state = AppState::in_memory().expect("in-memory state");
        let mut app = CosmicApp::new(state);

        app.resize_window(1400.0, 900.0);
        assert_eq!(app.config.window.placement.width, 1400.0);
        assert_eq!(app.config.window.placement.height, 900.0);
    }
}
