//! macOS Unified Toolbar component, segmented tab items, and action handlers.

use serde::{Deserialize, Serialize};

use crate::navigation::NavTab;

/// User interaction actions dispatched from the macOS toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MacosToolbarAction {
    /// Open the "Create Article" modal sheet.
    OpenNewArticleModal,
    /// Open the "Create Task" modal sheet.
    OpenNewTaskModal,
    /// Open the "Create Contact" modal sheet.
    OpenNewContactModal,
    /// Trigger primary creation action for active section.
    PrimaryAction,
    /// Toggle global search filter visibility / focus.
    ToggleSearch,
    /// Filter view to focus on overdue articles.
    FilterOverdueArticles,
    /// Toggle between Light and Dark theme modes.
    ToggleTheme,
    /// Toggle left sidebar collapse on compact screens.
    ToggleSidebar,
}

/// Segmented tab button item in the unified toolbar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MacosToolbarTabItem {
    /// Associated navigation tab.
    pub tab: NavTab,
    /// Display label.
    pub label: String,
    /// SF Symbols-compatible icon identifier.
    pub sf_symbol: String,
    /// Whether this tab is currently selected.
    pub is_selected: bool,
}

/// macOS Unified Toolbar presentation model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MacosToolbar {
    /// Primary application title.
    pub app_title: String,
    /// Active section subtitle.
    pub section_title: String,
    /// Left margin clearance for macOS window traffic lights in logical points.
    pub traffic_light_clearance: u32,
    /// Segmented tab items for quick navigation.
    pub segmented_tabs: Vec<MacosToolbarTabItem>,
    /// Count of currently overdue articles.
    pub overdue_count: usize,
    /// Whether search bar is currently visible in the toolbar.
    pub is_search_active: bool,
    /// Theme toggle tooltip string.
    pub theme_toggle_tooltip: String,
}

impl Default for MacosToolbar {
    fn default() -> Self {
        Self::new(NavTab::ArticlesKanban, 0, true)
    }
}

impl MacosToolbar {
    /// Constructs a macOS toolbar model from current active tab, overdue count, and theme.
    #[must_use]
    pub fn new(active_tab: NavTab, overdue_count: usize, is_dark_mode: bool) -> Self {
        let segmented_tabs = vec![
            MacosToolbarTabItem {
                tab: NavTab::ArticlesKanban,
                label: "Articles".to_string(),
                sf_symbol: "doc.richtext".to_string(),
                is_selected: active_tab == NavTab::ArticlesKanban,
            },
            MacosToolbarTabItem {
                tab: NavTab::TasksKanban,
                label: "Tasks".to_string(),
                sf_symbol: "checklist".to_string(),
                is_selected: active_tab == NavTab::TasksKanban,
            },
            MacosToolbarTabItem {
                tab: NavTab::ContactsDirectory,
                label: "Contacts".to_string(),
                sf_symbol: "person.2".to_string(),
                is_selected: active_tab == NavTab::ContactsDirectory,
            },
            MacosToolbarTabItem {
                tab: NavTab::Settings,
                label: "Settings".to_string(),
                sf_symbol: "gearshape".to_string(),
                is_selected: active_tab == NavTab::Settings,
            },
        ];

        Self {
            app_title: "NewsJournal".to_string(),
            section_title: active_tab.title().to_string(),
            traffic_light_clearance: 76,
            segmented_tabs,
            overdue_count,
            is_search_active: false,
            theme_toggle_tooltip: if is_dark_mode {
                "Switch to Light Mode".to_string()
            } else {
                "Switch to Dark Mode".to_string()
            },
        }
    }

    /// Returns the combined window title string.
    #[must_use]
    pub fn display_title(&self) -> String {
        format!("{} — {}", self.app_title, self.section_title)
    }

    /// Whether the overdue alert badge should be displayed.
    #[must_use]
    pub const fn has_overdue_alert(&self) -> bool {
        self.overdue_count > 0
    }

    /// Label text for the overdue alert badge.
    #[must_use]
    pub fn overdue_badge_label(&self) -> Option<String> {
        if self.overdue_count > 0 {
            Some(format!("⚠️ {} Overdue", self.overdue_count))
        } else {
            None
        }
    }

    /// Label text for the primary action button based on the active section.
    #[must_use]
    pub fn primary_action_label(active_tab: NavTab) -> &'static str {
        match active_tab {
            NavTab::ArticlesKanban => "+ New Article",
            NavTab::TasksKanban => "+ New Task",
            NavTab::ContactsDirectory => "+ New Contact",
            NavTab::Settings => "Save Settings",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_toolbar_properties() {
        let toolbar = MacosToolbar::new(NavTab::ArticlesKanban, 3, true);
        assert_eq!(toolbar.display_title(), "NewsJournal — Articles");
        assert!(toolbar.has_overdue_alert());
        assert_eq!(
            toolbar.overdue_badge_label(),
            Some("⚠️ 3 Overdue".to_string())
        );
        assert_eq!(toolbar.traffic_light_clearance, 76);
        assert_eq!(toolbar.segmented_tabs.len(), 4);
        assert!(toolbar.segmented_tabs[0].is_selected);
        assert!(!toolbar.segmented_tabs[1].is_selected);
    }

    #[test]
    fn test_macos_toolbar_action_labels() {
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
}
