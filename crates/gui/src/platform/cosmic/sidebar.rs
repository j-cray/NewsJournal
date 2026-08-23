//! COSMIC Left Navigation Bar sidebar component, glass styling, and interactions.

use serde::{Deserialize, Serialize};

use crate::navigation::NavTab;
use crate::platform::cosmic::glass::{CosmicContainerClass, CosmicFrostedGlass, CosmicGlassStyle};
use crate::state::AppState;
use crate::views::{build_nav_bar_view_with_platform, NavBarViewModel, NavItemViewModel};

/// Default width of the COSMIC sidebar in standard mode (pixels/points).
pub const COSMIC_SIDEBAR_STANDARD_WIDTH: f32 = 220.0;

/// Compact width of the COSMIC sidebar in icon-only collapsed mode.
pub const COSMIC_SIDEBAR_COMPACT_WIDTH: f32 = 64.0;

/// Default height of an individual navigation item row.
pub const COSMIC_NAV_ITEM_HEIGHT: f32 = 44.0;

/// User interaction actions originating from the COSMIC navigation bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CosmicNavAction {
    /// User selected a specific navigation tab.
    SelectTab(NavTab),
    /// Advance to the next tab in visual sequence.
    NextTab,
    /// Retreat to the previous tab in visual sequence.
    PrevTab,
}

/// Visual style tokens for an individual navigation item button in the COSMIC sidebar.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CosmicNavItemStyle {
    /// Background opacity/fill.
    pub background_opacity: f32,
    /// Border radius in pixels.
    pub corner_radius: f32,
    /// Whether to render the prominent left accent bar indicator.
    pub show_accent_indicator: bool,
    /// Whether high-contrast active text color should be used.
    pub is_active_highlight: bool,
    /// Overdue alert badge background color in hex.
    pub alert_badge_color: &'static str,
}

impl CosmicNavItemStyle {
    /// Computes the visual button style given its active and hover states.
    #[must_use]
    pub fn for_state(is_active: bool, is_hovered: bool, is_dark: bool) -> Self {
        let (background_opacity, show_accent_indicator, is_active_highlight) =
            match (is_active, is_hovered) {
                (true, _) => (if is_dark { 0.22 } else { 0.18 }, true, true),
                (false, true) => (if is_dark { 0.10 } else { 0.08 }, false, false),
                (false, false) => (0.0, false, false),
            };

        Self {
            background_opacity,
            corner_radius: 8.0,
            show_accent_indicator,
            is_active_highlight,
            alert_badge_color: "#E53E3E",
        }
    }
}

/// COSMIC Left Vertical Navigation Bar presentation component.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CosmicNavBar {
    /// Presentation data model containing navigation items and badge metrics.
    pub model: NavBarViewModel,
    /// Frosted glass background style applied to the sidebar panel.
    pub glass_style: CosmicGlassStyle,
    /// Current rendered width in logical points.
    pub width: f32,
    /// Whether the sidebar is collapsed to compact icon-only mode.
    pub is_compact: bool,
    /// Row height per navigation item.
    pub item_height: f32,
}

impl CosmicNavBar {
    /// Constructs a new COSMIC sidebar model from application state and glass material settings.
    #[must_use]
    pub fn new(state: &AppState, glass: &CosmicFrostedGlass, is_compact: bool) -> Self {
        let app_theme = state.theme();
        let glass_style = glass.style_for(CosmicContainerClass::Sidebar, app_theme);
        let model = build_nav_bar_view_with_platform(state, false);
        let width = if is_compact {
            COSMIC_SIDEBAR_COMPACT_WIDTH
        } else {
            COSMIC_SIDEBAR_STANDARD_WIDTH
        };

        Self {
            model,
            glass_style,
            width,
            is_compact,
            item_height: COSMIC_NAV_ITEM_HEIGHT,
        }
    }

    /// Returns the currently active navigation tab.
    #[must_use]
    pub fn active_tab(&self) -> NavTab {
        self.model.active_tab
    }

    /// Returns the main navigation items positioned at the top of the sidebar.
    #[must_use]
    pub fn main_items(&self) -> &[NavItemViewModel] {
        &self.model.main_items
    }

    /// Returns the docked navigation items positioned at the bottom of the sidebar.
    #[must_use]
    pub fn docked_items(&self) -> &[NavItemViewModel] {
        &self.model.docked_items
    }

    /// Returns whether any section has an active overdue deadline alert.
    #[must_use]
    pub const fn has_overdue_alert(&self) -> bool {
        self.model.has_overdue_alert()
    }

    /// Returns the view model for a specific navigation tab.
    #[must_use]
    pub fn item_for_tab(&self, tab: NavTab) -> Option<&NavItemViewModel> {
        self.model.item_for_tab(tab)
    }

    /// Returns button style configuration for a specific tab item.
    #[must_use]
    pub fn style_for_item(
        &self,
        item: &NavItemViewModel,
        is_hovered: bool,
        is_dark: bool,
    ) -> CosmicNavItemStyle {
        CosmicNavItemStyle::for_state(item.is_active, is_hovered, is_dark)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::cosmic::glass::CosmicFrostedGlass;

    #[test]
    fn test_cosmic_nav_bar_construction() {
        let state = AppState::in_memory().expect("in-memory state");
        let glass = CosmicFrostedGlass::default();

        let nav_bar = CosmicNavBar::new(&state, &glass, false);
        assert_eq!(nav_bar.width, COSMIC_SIDEBAR_STANDARD_WIDTH);
        assert!(!nav_bar.is_compact);
        assert_eq!(nav_bar.active_tab(), NavTab::ArticlesKanban);
        assert_eq!(nav_bar.main_items().len(), 3);
        assert_eq!(nav_bar.docked_items().len(), 1);
        assert_eq!(nav_bar.docked_items()[0].tab, NavTab::Settings);

        let compact_bar = CosmicNavBar::new(&state, &glass, true);
        assert_eq!(compact_bar.width, COSMIC_SIDEBAR_COMPACT_WIDTH);
        assert!(compact_bar.is_compact);
    }

    #[test]
    fn test_cosmic_nav_item_styles() {
        let active_style = CosmicNavItemStyle::for_state(true, false, true);
        assert!(active_style.show_accent_indicator);
        assert!(active_style.is_active_highlight);
        assert!(active_style.background_opacity > 0.15);

        let inactive_hover_style = CosmicNavItemStyle::for_state(false, true, true);
        assert!(!inactive_hover_style.show_accent_indicator);
        assert!(!inactive_hover_style.is_active_highlight);
        assert!(inactive_hover_style.background_opacity > 0.0);

        let idle_style = CosmicNavItemStyle::for_state(false, false, true);
        assert!(!idle_style.show_accent_indicator);
        assert_eq!(idle_style.background_opacity, 0.0);
    }
}
