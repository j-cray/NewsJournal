//! macOS native `NSVisualEffectView` window vibrancy and materials configuration.
//!
//! Provides type-safe abstractions for configuring native macOS visual effect views,
//! blending modes (`BehindWindow`, `WithinWindow`), and titlebar vibrancy materials.

use serde::{Deserialize, Serialize};

/// Standard macOS vibrancy materials mapped to AppKit `NSVisualEffectMaterial`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MacosVibrancyMaterial {
    /// Translucent material for navigation sidebars (`NSVisualEffectMaterialSidebar`).
    #[default]
    Sidebar,
    /// Translucent material for window header bars and toolbars (`NSVisualEffectMaterialHeaderView`).
    HeaderView,
    /// High-focus material for modal sheets and slide-over drawers (`NSVisualEffectMaterialSheet`).
    Sheet,
    /// Floating material for toasts and popover bubbles (`NSVisualEffectMaterialPopover`).
    Popover,
    /// Opaque heads-up display panel material (`NSVisualEffectMaterialHUDWindow`).
    HudWindow,
    /// Main content background translucent material (`NSVisualEffectMaterialContentBackground`).
    ContentBackground,
    /// Highlighted list/table selection vibrancy (`NSVisualEffectMaterialSelection`).
    Selection,
    /// Translucent material underlying the entire window (`NSVisualEffectMaterialUnderWindowBackground`).
    UnderWindowBackground,
    /// Context menu vibrancy material (`NSVisualEffectMaterialMenu`).
    Menu,
    /// Ultra-immersive full screen backdrop UI material (`NSVisualEffectMaterialFullScreenUI`).
    FullScreenUI,
}

impl MacosVibrancyMaterial {
    /// Returns the recommended backdrop blur radius in logical points for this material.
    #[must_use]
    pub const fn recommended_blur_radius(&self) -> f32 {
        match self {
            Self::Sidebar => 28.0,
            Self::HeaderView => 24.0,
            Self::Sheet => 36.0,
            Self::Popover => 20.0,
            Self::HudWindow => 30.0,
            Self::ContentBackground => 22.0,
            Self::Selection => 16.0,
            Self::UnderWindowBackground => 32.0,
            Self::Menu => 18.0,
            Self::FullScreenUI => 40.0,
        }
    }

    /// AppKit constant identifier string.
    #[must_use]
    pub const fn appkit_name(&self) -> &'static str {
        match self {
            Self::Sidebar => "NSVisualEffectMaterialSidebar",
            Self::HeaderView => "NSVisualEffectMaterialHeaderView",
            Self::Sheet => "NSVisualEffectMaterialSheet",
            Self::Popover => "NSVisualEffectMaterialPopover",
            Self::HudWindow => "NSVisualEffectMaterialHUDWindow",
            Self::ContentBackground => "NSVisualEffectMaterialContentBackground",
            Self::Selection => "NSVisualEffectMaterialSelection",
            Self::UnderWindowBackground => "NSVisualEffectMaterialUnderWindowBackground",
            Self::Menu => "NSVisualEffectMaterialMenu",
            Self::FullScreenUI => "NSVisualEffectMaterialFullScreenUI",
        }
    }
}

/// Blending mode for visual effect views (`NSVisualEffectBlendingMode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MacosVibrancyBlendingMode {
    /// Blends with content behind the application window (`NSVisualEffectBlendingModeBehindWindow`).
    #[default]
    BehindWindow,
    /// Blends with content within the application window (`NSVisualEffectBlendingModeWithinWindow`).
    WithinWindow,
}

/// Active state of the vibrancy effect (`NSVisualEffectState`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MacosVibrancyState {
    /// Automatically active when window is key, inactive when background (`NSVisualEffectStateFollowsWindow`).
    #[default]
    FollowsWindow,
    /// Always actively rendering backdrop blur (`NSVisualEffectStateActive`).
    Active,
    /// Inactive / disabled rendering (`NSVisualEffectStateInactive`).
    Inactive,
}

/// Complete configuration for macOS native window vibrancy and visual effects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacosVibrancyConfig {
    /// Primary vibrancy material for window backdrop.
    pub material: MacosVibrancyMaterial,
    /// Blending mode (behind vs within window).
    pub blending_mode: MacosVibrancyBlendingMode,
    /// Vibrancy activation state.
    pub state: MacosVibrancyState,
    /// Window corner radius in logical points.
    pub corner_radius: f32,
    /// Bitmask specifying which corners are rounded (15 = all 4 corners).
    pub corner_mask: u32,
    /// Whether the titlebar is transparent and blends into content.
    pub titlebar_transparent: bool,
    /// Whether window content extends full-size beneath the titlebar.
    pub fullsize_content_view: bool,
    /// Whether native macOS window shadow is enabled.
    pub window_shadow: bool,
}

impl Default for MacosVibrancyConfig {
    fn default() -> Self {
        Self {
            material: MacosVibrancyMaterial::UnderWindowBackground,
            blending_mode: MacosVibrancyBlendingMode::BehindWindow,
            state: MacosVibrancyState::FollowsWindow,
            corner_radius: 14.0,
            corner_mask: 15,
            titlebar_transparent: true,
            fullsize_content_view: true,
            window_shadow: true,
        }
    }
}

impl MacosVibrancyConfig {
    /// Creates a new `MacosVibrancyConfig` with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a vibrancy config configured for a specific material.
    #[must_use]
    pub fn with_material(material: MacosVibrancyMaterial) -> Self {
        Self {
            material,
            ..Default::default()
        }
    }

    /// Checks if the configuration uses behind-window blending.
    #[must_use]
    pub const fn is_behind_window(&self) -> bool {
        matches!(self.blending_mode, MacosVibrancyBlendingMode::BehindWindow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vibrancy_materials_and_defaults() {
        let config = MacosVibrancyConfig::new();
        assert_eq!(
            config.material,
            MacosVibrancyMaterial::UnderWindowBackground
        );
        assert_eq!(
            config.blending_mode,
            MacosVibrancyBlendingMode::BehindWindow
        );
        assert_eq!(config.state, MacosVibrancyState::FollowsWindow);
        assert_eq!(config.corner_radius, 14.0);
        assert!(config.titlebar_transparent);
        assert!(config.fullsize_content_view);
        assert!(config.window_shadow);
        assert!(config.is_behind_window());

        assert_eq!(
            MacosVibrancyMaterial::Sidebar.appkit_name(),
            "NSVisualEffectMaterialSidebar"
        );
        assert_eq!(MacosVibrancyMaterial::Sheet.recommended_blur_radius(), 36.0);
    }
}
