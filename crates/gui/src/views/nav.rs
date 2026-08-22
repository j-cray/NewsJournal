//! Navigation bar view models.

use crate::navigation::NavTab;
use crate::state::AppState;
use serde::{Deserialize, Serialize};

/// Visual button descriptor for rendering the left navigation bar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavItemViewModel {
    /// Associated navigation tab.
    pub tab: NavTab,
    /// Button display title.
    pub title: &'static str,
    /// Icon name.
    pub icon_name: &'static str,
    /// Fallback icon emoji.
    pub icon_emoji: &'static str,
    /// Keyboard shortcut string.
    pub shortcut_label: &'static str,
    /// Whether this tab is currently selected.
    pub is_active: bool,
}

/// Computes navigation bar button states from application state.
#[must_use]
pub fn build_nav_view_models(state: &AppState) -> Vec<NavItemViewModel> {
    NavTab::all()
        .iter()
        .map(|&tab| NavItemViewModel {
            tab,
            title: tab.title(),
            icon_name: tab.icon_name(),
            icon_emoji: tab.icon_emoji(),
            shortcut_label: tab.shortcut_label(),
            is_active: state.active_tab == tab,
        })
        .collect()
}
