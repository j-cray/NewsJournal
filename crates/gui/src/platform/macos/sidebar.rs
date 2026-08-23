//! macOS Left Navigation Bar sidebar component, liquid glass styling, and native vibrancy.

use serde::{Deserialize, Serialize};

use crate::navigation::NavTab;
use crate::platform::macos::glass::{MacosContainerClass, MacosGlassStyle, MacosLiquidGlass};
use crate::platform::macos::vibrancy::MacosVibrancyConfig;
use crate::state::AppState;
use crate::views::{build_nav_bar_view_with_platform, NavBarViewModel, NavItemViewModel};

/// Standard width of the macOS liquid glass sidebar (logical points).
pub const MACOS_SIDEBAR_STANDARD_WIDTH: f32 = 210.0;

/// Compact width of the macOS sidebar in icon-only mode.
pub const MACOS_SIDEBAR_COMPACT_WIDTH: f32 = 60.0;

/// Standard row height per navigation item on macOS.
pub const MACOS_NAV_ITEM_HEIGHT: f32 = 40.0;

/// User interaction actions originating from the macOS navigation bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MacosNavAction {
    /// User selected a specific navigation tab.
    SelectTab(NavTab),
    /// Advance to the next tab in visual sequence.
    NextTab,
    /// Retreat to the previous tab in visual sequence.
    PrevTab,
}

/// Visual style tokens for an individual navigation item button in the macOS liquid glass sidebar.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MacosNavItemStyle {
    /// Background opacity/fill for selection.
    pub background_opacity: f32,
    /// Border radius in points matching Apple Human Interface Guidelines (typically 6.0pt).
    pub corner_radius: f32,
    /// SF Symbols identifier.
    pub sf_symbol: &'static str,
    /// Whether high-contrast active text color should be used.
    pub is_active_highlight: bool,
    /// Overdue alert badge background color in hex.
    pub alert_badge_color: &'static str,
}

impl MacosNavItemStyle {
    /// Computes the visual button style given its active and hover states.
    #[must_use]
    pub fn for_state(tab: NavTab, is_active: bool, is_hovered: bool, is_dark: bool) -> Self {
        let (background_opacity, is_active_highlight) = match (is_active, is_hovered) {
            (true, _) => (if is_dark { 0.25 } else { 0.20 }, true),
            (false, true) => (if is_dark { 0.12 } else { 0.09 }, false),
            (false, false) => (0.0, false),
        };

        Self {
            background_opacity,
            corner_radius: 6.0,
            sf_symbol: tab.sf_symbol(),
            is_active_highlight,
            alert_badge_color: "#FF3B30",
        }
    }
}

/// macOS Left Vertical Navigation Bar presentation component.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MacosNavBar {
    /// Presentation data model containing navigation items and badge metrics.
    pub model: NavBarViewModel,
    /// Liquid glass background style applied to the sidebar panel.
    pub glass_style: MacosGlassStyle,
    /// Native vibrancy configuration for NSVisualEffectView sidebar material.
    pub vibrancy: MacosVibrancyConfig,
    /// Current rendered width in logical points.
    pub width: f32,
    /// Whether the sidebar is collapsed to compact icon-only mode.
    pub is_compact: bool,
    /// Row height per navigation item.
    pub item_height: f32,
    /// Traffic light buttons clearance height in points.
    pub traffic_light_clearance: u32,
}

impl MacosNavBar {
    /// Constructs a new macOS sidebar model from application state and liquid glass material settings.
    #[must_use]
    pub fn new(
        state: &AppState,
        glass: &MacosLiquidGlass,
        vibrancy: &MacosVibrancyConfig,
        is_compact: bool,
    ) -> Self {
        let app_theme = state.theme();
        let glass_style = glass.style_for(MacosContainerClass::Sidebar, app_theme);
        let model = build_nav_bar_view_with_platform(state, true);
        let width = if is_compact {
            MACOS_SIDEBAR_COMPACT_WIDTH
        } else {
            MACOS_SIDEBAR_STANDARD_WIDTH
        };

        Self {
            model,
            glass_style,
            vibrancy: vibrancy.clone(),
            width,
            is_compact,
            item_height: MACOS_NAV_ITEM_HEIGHT,
            traffic_light_clearance: 76,
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
    ) -> MacosNavItemStyle {
        MacosNavItemStyle::for_state(item.tab, item.is_active, is_hovered, is_dark)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::macos::glass::MacosLiquidGlass;

    #[test]
    fn test_macos_nav_bar_construction() {
        let state = AppState::in_memory().expect("in-memory state");
        let glass = MacosLiquidGlass::default();
        let vibrancy = MacosVibrancyConfig::default();

        let nav_bar = MacosNavBar::new(&state, &glass, &vibrancy, false);
        assert_eq!(nav_bar.width, MACOS_SIDEBAR_STANDARD_WIDTH);
        assert!(!nav_bar.is_compact);
        assert_eq!(nav_bar.active_tab(), NavTab::ArticlesKanban);
        assert_eq!(nav_bar.main_items().len(), 3);
        assert_eq!(nav_bar.docked_items().len(), 1);
        assert_eq!(nav_bar.docked_items()[0].tab, NavTab::Settings);
        assert_eq!(nav_bar.traffic_light_clearance, 76);

        // macOS shortcuts check
        let art_item = nav_bar.item_for_tab(NavTab::ArticlesKanban).unwrap();
        assert_eq!(art_item.shortcut_label, "⌘1");
        assert_eq!(art_item.sf_symbol, "doc.richtext");

        let compact_bar = MacosNavBar::new(&state, &glass, &vibrancy, true);
        assert_eq!(compact_bar.width, MACOS_SIDEBAR_COMPACT_WIDTH);
        assert!(compact_bar.is_compact);
    }

    #[test]
    fn test_macos_nav_item_styles() {
        let active_style = MacosNavItemStyle::for_state(NavTab::ArticlesKanban, true, false, true);
        assert!(active_style.is_active_highlight);
        assert_eq!(active_style.sf_symbol, "doc.richtext");
        assert!(active_style.background_opacity > 0.15);

        let idle_style = MacosNavItemStyle::for_state(NavTab::TasksKanban, false, false, false);
        assert!(!idle_style.is_active_highlight);
        assert_eq!(idle_style.sf_symbol, "checklist");
        assert_eq!(idle_style.background_opacity, 0.0);
    }
}
