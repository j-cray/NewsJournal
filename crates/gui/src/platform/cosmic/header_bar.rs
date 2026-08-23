//! COSMIC Header Bar component, action buttons, and overdue alert indicators.

use serde::{Deserialize, Serialize};

use crate::navigation::NavTab;
use crate::platform::cosmic::icon::{COSMIC_APP_ICON_NAME, COSMIC_SYMBOLIC_ICON_NAME};

/// Header bar interaction actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CosmicHeaderBarAction {
    /// Open the "Create Article" modal dialog.
    OpenNewArticleModal,
    /// Open the "Create Task" modal dialog.
    OpenNewTaskModal,
    /// Open the "Create Contact" modal dialog.
    OpenNewContactModal,
    /// Trigger primary creation action for active section.
    PrimaryAction,
    /// Toggle global search filter focus.
    ToggleSearch,
    /// Filter view to focus on overdue articles.
    FilterOverdueArticles,
    /// Toggle between Light and Dark theme modes.
    ToggleTheme,
    /// Toggle left sidebar collapse on compact screens.
    ToggleSidebar,
}

/// Header bar presentation model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CosmicHeaderBar {
    /// Primary title string.
    pub app_title: String,
    /// Active section subtitle.
    pub section_title: String,
    /// Application symbolic icon name.
    pub symbolic_icon: String,
    /// Application main icon name.
    pub app_icon: String,
    /// Count of currently overdue articles.
    pub overdue_count: usize,
    /// Whether search bar is currently active / visible in the header.
    pub is_search_active: bool,
    /// Active theme description for tooltip (e.g. "Switch to Light Mode").
    pub theme_toggle_tooltip: String,
}

impl Default for CosmicHeaderBar {
    fn default() -> Self {
        Self {
            app_title: "NewsJournal".to_string(),
            section_title: NavTab::ArticlesKanban.title().to_string(),
            symbolic_icon: COSMIC_SYMBOLIC_ICON_NAME.to_string(),
            app_icon: COSMIC_APP_ICON_NAME.to_string(),
            overdue_count: 0,
            is_search_active: false,
            theme_toggle_tooltip: "Toggle Theme".to_string(),
        }
    }
}

impl CosmicHeaderBar {
    /// Constructs a header bar model from the active navigation tab, overdue count, and theme.
    #[must_use]
    pub fn new(active_tab: NavTab, overdue_count: usize, is_dark_mode: bool) -> Self {
        Self {
            app_title: "NewsJournal".to_string(),
            section_title: active_tab.title().to_string(),
            symbolic_icon: COSMIC_SYMBOLIC_ICON_NAME.to_string(),
            app_icon: COSMIC_APP_ICON_NAME.to_string(),
            overdue_count,
            is_search_active: false,
            theme_toggle_tooltip: if is_dark_mode {
                "Switch to Light Mode".to_string()
            } else {
                "Switch to Dark Mode".to_string()
            },
        }
    }

    /// Returns the combined header title string (e.g. "NewsJournal — Articles").
    #[must_use]
    pub fn display_title(&self) -> String {
        format!("{} — {}", self.app_title, self.section_title)
    }

    /// Whether the urgent overdue badge should be rendered in the header bar.
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
    fn test_cosmic_header_bar_properties() {
        let header = CosmicHeaderBar::new(NavTab::ArticlesKanban, 2, true);
        assert_eq!(header.display_title(), "NewsJournal — Articles");
        assert!(header.has_overdue_alert());
        assert_eq!(
            header.overdue_badge_label(),
            Some("⚠️ 2 Overdue".to_string())
        );
        assert_eq!(header.theme_toggle_tooltip, "Switch to Light Mode");
    }

    #[test]
    fn test_cosmic_header_bar_no_overdue() {
        let header = CosmicHeaderBar::new(NavTab::TasksKanban, 0, false);
        assert_eq!(header.display_title(), "NewsJournal — Tasks");
        assert!(!header.has_overdue_alert());
        assert_eq!(header.overdue_badge_label(), None);
        assert_eq!(header.theme_toggle_tooltip, "Switch to Dark Mode");
    }

    #[test]
    fn test_primary_action_labels() {
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
}
