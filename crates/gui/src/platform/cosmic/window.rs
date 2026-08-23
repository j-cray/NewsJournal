//! COSMIC window configuration, layout breakpoints, and sizing constraints.

use serde::{Deserialize, Serialize};

use crate::navigation::NavTab;

/// Minimum allowable window width in logical points.
pub const COSMIC_MIN_WINDOW_WIDTH: f32 = 960.0;

/// Minimum allowable window height in logical points.
pub const COSMIC_MIN_WINDOW_HEIGHT: f32 = 600.0;

/// Default launch window width in logical points.
pub const COSMIC_DEFAULT_WINDOW_WIDTH: f32 = 1280.0;

/// Default launch window height in logical points.
pub const COSMIC_DEFAULT_WINDOW_HEIGHT: f32 = 800.0;

/// Screen layout responsive category based on window width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CosmicResponsiveBreakpoint {
    /// Compact screen layout (< 1100px width), collapsible nav sidebar.
    Compact,
    /// Standard desktop screen layout (1100px - 1500px width).
    Standard,
    /// Ultra-wide / expanded multi-column layout (> 1500px width).
    Wide,
}

/// Window placement and dimension coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CosmicWindowPlacement {
    /// Window width in logical points.
    pub width: f32,
    /// Window height in logical points.
    pub height: f32,
    /// Window horizontal position X (optional).
    pub x: Option<f32>,
    /// Window vertical position Y (optional).
    pub y: Option<f32>,
}

impl Default for CosmicWindowPlacement {
    fn default() -> Self {
        Self {
            width: COSMIC_DEFAULT_WINDOW_WIDTH,
            height: COSMIC_DEFAULT_WINDOW_HEIGHT,
            x: None,
            y: None,
        }
    }
}

/// COSMIC window configuration manager.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CosmicWindowConfig {
    /// Current window placement and geometry.
    pub placement: CosmicWindowPlacement,
    /// Minimum window width limit.
    pub min_width: f32,
    /// Minimum window height limit.
    pub min_height: f32,
    /// Whether the window supports resizing by the user.
    pub resizable: bool,
    /// Whether the window requests transparent compositor surface for frosted glass blur.
    pub transparent: bool,
    /// Whether native window decorations are enabled.
    pub decorated: bool,
    /// Whether the window is currently maximized.
    pub is_maximized: bool,
    /// Whether the window is currently focused.
    pub is_focused: bool,
}

impl Default for CosmicWindowConfig {
    fn default() -> Self {
        Self {
            placement: CosmicWindowPlacement::default(),
            min_width: COSMIC_MIN_WINDOW_WIDTH,
            min_height: COSMIC_MIN_WINDOW_HEIGHT,
            resizable: true,
            transparent: true,
            decorated: true,
            is_maximized: false,
            is_focused: true,
        }
    }
}

impl CosmicWindowConfig {
    /// Creates a new `CosmicWindowConfig` with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates the current responsive breakpoint based on window width.
    #[must_use]
    pub fn breakpoint(&self) -> CosmicResponsiveBreakpoint {
        let width = self.placement.width;
        if width < 1100.0 {
            CosmicResponsiveBreakpoint::Compact
        } else if width <= 1500.0 {
            CosmicResponsiveBreakpoint::Standard
        } else {
            CosmicResponsiveBreakpoint::Wide
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

    /// Generates a standardized COSMIC window title based on the active tab and overdue alerts.
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
    fn test_cosmic_window_defaults_and_clamping() {
        let mut config = CosmicWindowConfig::new();
        assert_eq!(config.placement.width, 1280.0);
        assert_eq!(config.placement.height, 800.0);
        assert!(config.transparent);
        assert!(config.resizable);
        assert_eq!(config.breakpoint(), CosmicResponsiveBreakpoint::Standard);

        // Resize below minimum width/height
        config.resize(500.0, 400.0);
        assert_eq!(config.placement.width, COSMIC_MIN_WINDOW_WIDTH);
        assert_eq!(config.placement.height, COSMIC_MIN_WINDOW_HEIGHT);

        // Resize to wide
        config.resize(1600.0, 900.0);
        assert_eq!(config.breakpoint(), CosmicResponsiveBreakpoint::Wide);

        // Resize to compact
        config.resize(1000.0, 700.0);
        assert_eq!(config.breakpoint(), CosmicResponsiveBreakpoint::Compact);
    }

    #[test]
    fn test_cosmic_window_title_formatting() {
        let config = CosmicWindowConfig::new();
        assert_eq!(
            config.format_window_title(NavTab::ArticlesKanban, 0),
            "NewsJournal — Articles"
        );
        assert_eq!(
            config.format_window_title(NavTab::TasksKanban, 3),
            "NewsJournal — Tasks (⚠️ 3 Overdue)"
        );
    }
}
