//! Navigation bar view models, badge computations, and presentation builders.

use serde::Serialize;

use crate::navigation::NavTab;
use crate::state::AppState;

/// Visual button descriptor for rendering an item in the left vertical navigation bar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NavItemViewModel {
    /// Associated navigation tab.
    pub tab: NavTab,
    /// Button display title (e.g. "Articles").
    pub title: &'static str,
    /// Subtitle / section description (e.g. "Main Deck").
    pub subtitle: &'static str,
    /// Desktop symbolic icon name (e.g. "newspaper").
    pub icon_name: &'static str,
    /// Fallback Unicode emoji icon (e.g. "📰").
    pub icon_emoji: &'static str,
    /// SF Symbols identifier for macOS (e.g. "doc.richtext").
    pub sf_symbol: &'static str,
    /// Formatted keyboard shortcut label (e.g. "Ctrl+1" or "⌘1").
    pub shortcut_label: String,
    /// Whether this tab is currently the active view.
    pub is_active: bool,
    /// Whether this item is docked at the bottom of the navigation bar (e.g. Settings).
    pub is_docked: bool,
    /// Numerical badge count (e.g. total articles, total tasks, or total contacts).
    pub badge_count: Option<usize>,
    /// Optional formatted badge string for rendering in the UI.
    pub badge_label: Option<String>,
    /// Whether this item has an active overdue deadline alert.
    pub has_overdue_alert: bool,
    /// Count of overdue articles relevant to this tab.
    pub overdue_count: usize,
}

impl NavItemViewModel {
    /// Returns the primary text label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        self.title
    }

    /// Returns whether this button should display an alert badge indicator.
    #[must_use]
    pub const fn show_alert_badge(&self) -> bool {
        self.has_overdue_alert && self.overdue_count > 0
    }
}

/// Complete presentation model representing the left vertical navigation bar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NavBarViewModel {
    /// Main navigation items placed at the top (Articles, Tasks, Contacts).
    pub main_items: Vec<NavItemViewModel>,
    /// Docked navigation items placed at the bottom (Settings).
    pub docked_items: Vec<NavItemViewModel>,
    /// Flattened list of all navigation items.
    pub all_items: Vec<NavItemViewModel>,
    /// Currently active navigation tab.
    pub active_tab: NavTab,
    /// Total count of overdue articles across the workspace.
    pub overdue_alert_count: usize,
    /// Application brand title.
    pub brand_title: &'static str,
    /// Application brand subtitle.
    pub brand_subtitle: &'static str,
    /// Brand logo icon emoji.
    pub brand_icon_emoji: &'static str,
    /// Current NewsJournal application version.
    pub app_version: &'static str,
}

impl NavBarViewModel {
    /// Retrieves the view model for a specific navigation tab.
    #[must_use]
    pub fn item_for_tab(&self, tab: NavTab) -> Option<&NavItemViewModel> {
        self.all_items.iter().find(|item| item.tab == tab)
    }

    /// Retrieves the active navigation item view model.
    #[must_use]
    pub fn active_item(&self) -> &NavItemViewModel {
        self.item_for_tab(self.active_tab)
            .unwrap_or(&self.main_items[0])
    }

    /// Returns whether any section has an active overdue alert.
    #[must_use]
    pub const fn has_overdue_alert(&self) -> bool {
        self.overdue_alert_count > 0
    }

    /// Returns the total number of articles stored in the workspace.
    #[must_use]
    pub fn total_article_count(&self) -> usize {
        self.item_for_tab(NavTab::ArticlesKanban)
            .and_then(|item| item.badge_count)
            .unwrap_or(0)
    }

    /// Returns the total number of tasks stored in the workspace.
    #[must_use]
    pub fn total_task_count(&self) -> usize {
        self.item_for_tab(NavTab::TasksKanban)
            .and_then(|item| item.badge_count)
            .unwrap_or(0)
    }

    /// Returns the total number of contacts stored in the workspace.
    #[must_use]
    pub fn total_contact_count(&self) -> usize {
        self.item_for_tab(NavTab::ContactsDirectory)
            .and_then(|item| item.badge_count)
            .unwrap_or(0)
    }
}

/// Builds an individual navigation item view model from application state and platform context.
#[must_use]
pub fn build_nav_item(tab: NavTab, state: &AppState, is_macos: bool) -> NavItemViewModel {
    let overdue_count = state.deadline_summary.overdue_count;

    let (badge_count, badge_label, has_overdue_alert, tab_overdue_count) = match tab {
        NavTab::ArticlesKanban => {
            let count = state.articles.len();
            let label = if overdue_count > 0 {
                Some(format!("{count} (⚠️ {overdue_count})"))
            } else if count > 0 {
                Some(count.to_string())
            } else {
                None
            };
            (Some(count), label, overdue_count > 0, overdue_count)
        }
        NavTab::TasksKanban => {
            let count = state.tasks.len();
            let label = if count > 0 {
                Some(count.to_string())
            } else {
                None
            };
            (Some(count), label, false, 0)
        }
        NavTab::ContactsDirectory => {
            let count = state.contacts.len();
            let label = if count > 0 {
                Some(count.to_string())
            } else {
                None
            };
            (Some(count), label, false, 0)
        }
        NavTab::Settings => (None, None, false, 0),
    };

    NavItemViewModel {
        tab,
        title: tab.title(),
        subtitle: tab.subtitle(),
        icon_name: tab.icon_name(),
        icon_emoji: tab.icon_emoji(),
        sf_symbol: tab.sf_symbol(),
        shortcut_label: tab.shortcut_label_for_platform(is_macos).to_string(),
        is_active: state.active_tab == tab,
        is_docked: tab.is_docked(),
        badge_count,
        badge_label,
        has_overdue_alert,
        overdue_count: tab_overdue_count,
    }
}

/// Computes navigation bar button states from application state (defaulting to non-macOS shortcut labels).
#[must_use]
pub fn build_nav_view_models(state: &AppState) -> Vec<NavItemViewModel> {
    NavTab::all()
        .iter()
        .map(|&tab| build_nav_item(tab, state, false))
        .collect()
}

/// Constructs the complete `NavBarViewModel` from application state using default platform detection.
#[must_use]
pub fn build_nav_bar_view(state: &AppState) -> NavBarViewModel {
    let is_macos = cfg!(target_os = "macos");
    build_nav_bar_view_with_platform(state, is_macos)
}

/// Constructs the complete `NavBarViewModel` from application state with explicit platform flag.
#[must_use]
pub fn build_nav_bar_view_with_platform(state: &AppState, is_macos: bool) -> NavBarViewModel {
    let main_items: Vec<NavItemViewModel> = NavTab::main_tabs()
        .iter()
        .map(|&tab| build_nav_item(tab, state, is_macos))
        .collect();

    let docked_items: Vec<NavItemViewModel> = NavTab::docked_tabs()
        .iter()
        .map(|&tab| build_nav_item(tab, state, is_macos))
        .collect();

    let mut all_items = main_items.clone();
    all_items.extend(docked_items.clone());

    NavBarViewModel {
        main_items,
        docked_items,
        all_items,
        active_tab: state.active_tab,
        overdue_alert_count: state.deadline_summary.overdue_count,
        brand_title: "NewsJournal",
        brand_subtitle: "Reporting Organizer",
        brand_icon_emoji: "📰",
        app_version: newsjournal_core::VERSION,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::AppMessage;
    use crate::EventLoop;
    use chrono::{Duration, Utc};
    use newsjournal_core::models::{Article, Contact, Task};
    use newsjournal_core::ArticleStage;

    #[test]
    fn test_nav_bar_view_construction_and_badges() {
        let state = AppState::in_memory().expect("in-memory state");
        let mut event_loop = EventLoop::new(state);

        let now = Utc::now();

        // Create overdue article
        let mut article = Article::new("breaking-story", "City Council Votes on Transit");
        article.stage = ArticleStage::Writing;
        article.deadline = Some(now - Duration::hours(2));
        let article_id = article.id;
        event_loop
            .dispatch(AppMessage::CreateArticle(article))
            .unwrap();

        // Create 2 tasks
        event_loop
            .dispatch(AppMessage::CreateTask(Task::new(
                article_id,
                "Get quote from Mayor",
            )))
            .unwrap();
        event_loop
            .dispatch(AppMessage::CreateTask(Task::new(
                article_id,
                "Check municipal budget",
            )))
            .unwrap();

        // Create 1 contact
        event_loop
            .dispatch(AppMessage::CreateContact(Contact::new("Mayor Adams")))
            .unwrap();

        let state = event_loop.state();

        let nav_bar_linux = build_nav_bar_view_with_platform(state, false);
        assert_eq!(nav_bar_linux.main_items.len(), 3);
        assert_eq!(nav_bar_linux.docked_items.len(), 1);
        assert_eq!(nav_bar_linux.all_items.len(), 4);
        assert_eq!(nav_bar_linux.active_tab, NavTab::ArticlesKanban);
        assert_eq!(nav_bar_linux.overdue_alert_count, 1);
        assert!(nav_bar_linux.has_overdue_alert());
        assert_eq!(nav_bar_linux.total_article_count(), 1);
        assert_eq!(nav_bar_linux.total_task_count(), 2);
        assert_eq!(nav_bar_linux.total_contact_count(), 1);

        let articles_item = nav_bar_linux.item_for_tab(NavTab::ArticlesKanban).unwrap();
        assert!(articles_item.is_active);
        assert!(!articles_item.is_docked);
        assert_eq!(articles_item.title, "Articles");
        assert_eq!(articles_item.subtitle, "Main Deck");
        assert_eq!(articles_item.shortcut_label, "Ctrl+1");
        assert!(articles_item.has_overdue_alert);
        assert_eq!(articles_item.overdue_count, 1);
        assert_eq!(articles_item.badge_count, Some(1));
        assert_eq!(articles_item.badge_label, Some("1 (⚠️ 1)".to_string()));

        let tasks_item = nav_bar_linux.item_for_tab(NavTab::TasksKanban).unwrap();
        assert!(!tasks_item.is_active);
        assert_eq!(tasks_item.badge_count, Some(2));
        assert_eq!(tasks_item.badge_label, Some("2".to_string()));
        assert_eq!(tasks_item.shortcut_label, "Ctrl+2");

        let settings_item = nav_bar_linux.item_for_tab(NavTab::Settings).unwrap();
        assert!(settings_item.is_docked);
        assert_eq!(settings_item.title, "Settings");
        assert_eq!(settings_item.subtitle, "Preferences");
        assert_eq!(settings_item.shortcut_label, "Ctrl+4");
        assert_eq!(settings_item.badge_count, None);

        // Check macOS shortcuts
        let nav_bar_macos = build_nav_bar_view_with_platform(state, true);
        let mac_art = nav_bar_macos.item_for_tab(NavTab::ArticlesKanban).unwrap();
        assert_eq!(mac_art.shortcut_label, "⌘1");
        let mac_set = nav_bar_macos.item_for_tab(NavTab::Settings).unwrap();
        assert_eq!(mac_set.shortcut_label, "⌘4");
    }
}
