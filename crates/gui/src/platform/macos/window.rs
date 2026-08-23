//! macOS window configuration, titlebar styling, traffic light insets, and layout breakpoints.

use serde::{Deserialize, Serialize};

use crate::navigation::NavTab;

/// Minimum allowable window width in logical points.
pub const MACOS_MIN_WINDOW_WIDTH: f32 = 960.0;

/// Minimum allowable window height in logical points.
pub const MACOS_MIN_WINDOW_HEIGHT: f32 = 600.0;

/// Default launch window width in logical points.
pub const MACOS_DEFAULT_WINDOW_WIDTH: f32 = 1280.0;

/// Default launch window height in logical points.
pub const MACOS_DEFAULT_WINDOW_HEIGHT: f32 = 800.0;

/// Traffic light buttons horizontal margin inset in logical points.
pub const MACOS_TRAFFIC_LIGHT_INSET_X: f32 = 16.0;

/// Traffic light buttons vertical margin inset in logical points.
pub const MACOS_TRAFFIC_LIGHT_INSET_Y: f32 = 16.0;

/// Screen layout responsive category based on macOS window width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacosResponsiveBreakpoint {
    /// Compact layout (< 1100px width), collapsible nav sidebar.
    Compact,
    /// Standard desktop layout (1100px - 1500px width).
    Standard,
    /// Expanded multi-column layout (> 1500px width).
    Wide,
}

/// macOS native titlebar appearance style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MacosTitlebarStyle {
    /// Unified compact titlebar blending with toolbar (`NSWindowStyleMaskUnifiedTitleAndToolbar`).
    #[default]
    UnifiedCompact,
    /// Unified tall titlebar with additional vertical padding.
    UnifiedTall,
    /// Transparent titlebar with hidden title and overlay traffic lights.
    TransparentHidden,
}

/// Window placement and dimension coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MacosWindowPlacement {
    /// Window width in logical points.
    pub width: f32,
    /// Window height in logical points.
    pub height: f32,
    /// Window horizontal position X (optional).
    pub x: Option<f32>,
    /// Window vertical position Y (optional).
    pub y: Option<f32>,
}

impl Default for MacosWindowPlacement {
    fn default() -> Self {
        Self {
            width: MACOS_DEFAULT_WINDOW_WIDTH,
            height: MACOS_DEFAULT_WINDOW_HEIGHT,
            x: None,
            y: None,
        }
    }
}

/// macOS window configuration manager.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacosWindowConfig {
    /// Current window placement and geometry.
    pub placement: MacosWindowPlacement,
    /// Minimum window width limit.
    pub min_width: f32,
    /// Minimum window height limit.
    pub min_height: f32,
    /// Whether the window supports resizing by the user.
    pub resizable: bool,
    /// Whether the titlebar is transparent and blends into content.
    pub titlebar_transparent: bool,
    /// Whether window content extends full-size beneath the titlebar.
    pub fullsize_content: bool,
    /// Titlebar appearance style.
    pub titlebar_style: MacosTitlebarStyle,
    /// Traffic light buttons position insets `(inset_x, inset_y)`.
    pub traffic_light_inset: (f32, f32),
    /// Whether the window is currently in macOS native fullscreen.
    pub is_fullscreen: bool,
    /// Whether the window is currently active and focused.
    pub is_active: bool,
}

impl Default for MacosWindowConfig {
    fn default() -> Self {
        Self {
            placement: MacosWindowPlacement::default(),
            min_width: MACOS_MIN_WINDOW_WIDTH,
            min_height: MACOS_MIN_WINDOW_HEIGHT,
            resizable: true,
            titlebar_transparent: true,
            fullsize_content: true,
            titlebar_style: MacosTitlebarStyle::UnifiedCompact,
            traffic_light_inset: (MACOS_TRAFFIC_LIGHT_INSET_X, MACOS_TRAFFIC_LIGHT_INSET_Y),
            is_fullscreen: false,
            is_active: true,
        }
    }
}

impl MacosWindowConfig {
    /// Creates a new `MacosWindowConfig` with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates the current responsive breakpoint based on window width.
    #[must_use]
    pub fn breakpoint(&self) -> MacosResponsiveBreakpoint {
        let width = self.placement.width;
        if width < 1100.0 {
            MacosResponsiveBreakpoint::Compact
        } else if width <= 1500.0 {
            MacosResponsiveBreakpoint::Standard
        } else {
            MacosResponsiveBreakpoint::Wide
        }
    }

    /// Clamps requested dimensions against min window width and height constraints.
    pub fn resize(&mut self, width: f32, height: f32) {
        self.placement.width = width.max(self.min_width);
        self.placement.height = height.max(self.min_height);
    }

    /// Updates window position coordinates.
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.placement.x = Some(x);
        self.placement.y = Some(y);
    }

    /// Generates a standardized macOS window title based on the active tab and overdue alerts.
    #[must_use]
    pub fn format_window_title(&self, active_tab: NavTab, overdue_count: usize) -> String {
        if overdue_count > 0 {
            format!(
                "NewsJournal — {} (⚠️ {} Overdue)",
                active_tab.title(),
                overdue_count
            )
        } else {
            format!("NewsJournal — {}", active_tab.title())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_window_defaults_and_clamping() {
        let mut config = MacosWindowConfig::new();
        assert_eq!(config.placement.width, 1280.0);
        assert_eq!(config.placement.height, 800.0);
        assert!(config.titlebar_transparent);
        assert!(config.fullsize_content);
        assert_eq!(config.breakpoint(), MacosResponsiveBreakpoint::Standard);

        // Resize below minimum constraints
        config.resize(600.0, 400.0);
        assert_eq!(config.placement.width, MACOS_MIN_WINDOW_WIDTH);
        assert_eq!(config.placement.height, MACOS_MIN_WINDOW_HEIGHT);
        assert_eq!(config.breakpoint(), MacosResponsiveBreakpoint::Compact);

        // Resize above standard breakpoint
        config.resize(1700.0, 1100.0);
        assert_eq!(config.breakpoint(), MacosResponsiveBreakpoint::Wide);
    }

    #[test]
    fn test_macos_window_title_formatting() {
        let config = MacosWindowConfig::new();
        assert_eq!(
            config.format_window_title(NavTab::ArticlesKanban, 0),
            "NewsJournal — Articles"
        );
        assert_eq!(
            config.format_window_title(NavTab::ArticlesKanban, 4),
            "NewsJournal — Articles (⚠️ 4 Overdue)"
        );
    }
}
