//! Glass styling tokens, color definitions, and theme resolution for NewsJournal.

pub mod detector;
pub mod engine;

pub use detector::{DetectionStrategy, SystemThemeDetector, SystemThemeWatcher};
pub use engine::ThemeEngine;

use newsjournal_core::models::ThemeMode;
use serde::{Deserialize, Serialize};

/// Concrete theme variant resolved from user preference and system mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ResolvedTheme {
    /// High-contrast bright styling.
    Light,
    /// High-contrast dark styling with vibrant translucent overlays.
    #[default]
    Dark,
}

/// Glass styling tokens for COSMIC frosted glass and macOS liquid glass.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GlassMaterial {
    /// Background blur radius in logical points.
    pub blur_radius: f32,
    /// Surface background opacity (0.0 to 1.0).
    pub surface_opacity: f32,
    /// Border highlight opacity (0.0 to 1.0).
    pub border_opacity: f32,
    /// Border stroke width in logical pixels.
    pub border_width: f32,
    /// Standard container corner radius.
    pub corner_radius: f32,
}

impl Default for GlassMaterial {
    fn default() -> Self {
        Self {
            blur_radius: 24.0,
            surface_opacity: 0.78,
            border_opacity: 0.22,
            border_width: 1.0,
            corner_radius: 12.0,
        }
    }
}

/// High-contrast, accessible UI color tokens.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColorTokens {
    /// Main window background RGB.
    pub background: (u8, u8, u8),
    /// Surface card background RGBA (with alpha).
    pub surface: (u8, u8, u8, f32),
    /// Subtle card border RGBA.
    pub border: (u8, u8, u8, f32),
    /// Primary high-contrast text RGB.
    pub text_primary: (u8, u8, u8),
    /// Secondary supporting text RGB.
    pub text_secondary: (u8, u8, u8),
    /// Muted / disabled text RGB.
    pub text_muted: (u8, u8, u8),
    /// Primary brand accent RGB.
    pub accent: (u8, u8, u8),
    /// Urgent / Overdue alert RGB.
    pub overdue_alert: (u8, u8, u8),
    /// Warning / Due soon RGB.
    pub due_soon_warning: (u8, u8, u8),
    /// Success / Complete indicator RGB.
    pub success: (u8, u8, u8),
    /// Layered modal backdrop overlay RGBA.
    pub modal_backdrop: (u8, u8, u8, f32),
}

impl ColorTokens {
    /// Dark mode color tokens.
    #[must_use]
    pub const fn dark() -> Self {
        Self {
            background: (18, 20, 24),
            surface: (30, 34, 42, 0.75),
            border: (255, 255, 255, 0.12),
            text_primary: (245, 247, 250),
            text_secondary: (175, 182, 194),
            text_muted: (112, 120, 134),
            accent: (64, 158, 255),
            overdue_alert: (255, 77, 79),
            due_soon_warning: (250, 173, 20),
            success: (82, 196, 26),
            modal_backdrop: (0, 0, 0, 0.65),
        }
    }

    /// Light mode color tokens.
    #[must_use]
    pub const fn light() -> Self {
        Self {
            background: (246, 248, 250),
            surface: (255, 255, 255, 0.85),
            border: (0, 0, 0, 0.08),
            text_primary: (24, 28, 34),
            text_secondary: (78, 86, 98),
            text_muted: (138, 145, 156),
            accent: (24, 144, 255),
            overdue_alert: (207, 19, 34),
            due_soon_warning: (212, 107, 8),
            success: (56, 158, 13),
            modal_backdrop: (0, 0, 0, 0.40),
        }
    }
}

/// Unified application theme configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppTheme {
    /// Selected user preference mode.
    pub mode: ThemeMode,
    /// Resolved active variant.
    pub resolved: ResolvedTheme,
    /// Glass styling configuration.
    pub glass: GlassMaterial,
    /// Resolved color palette tokens.
    pub colors: ColorTokens,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self::new(ThemeMode::System, false)
    }
}

impl AppTheme {
    /// Creates and resolves a theme based on user mode preference and OS environment.
    #[must_use]
    pub fn new(mode: ThemeMode, system_is_dark: bool) -> Self {
        let resolved = match mode {
            ThemeMode::System => {
                if system_is_dark {
                    ResolvedTheme::Dark
                } else {
                    ResolvedTheme::Light
                }
            }
            ThemeMode::Light => ResolvedTheme::Light,
            ThemeMode::Dark => ResolvedTheme::Dark,
        };

        let colors = match resolved {
            ResolvedTheme::Dark => ColorTokens::dark(),
            ResolvedTheme::Light => ColorTokens::light(),
        };

        Self {
            mode,
            resolved,
            glass: GlassMaterial::default(),
            colors,
        }
    }

    /// Updates theme resolution when system mode changes.
    pub fn update_system_preference(&mut self, system_is_dark: bool) {
        *self = Self::new(self.mode, system_is_dark);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_resolution() {
        let theme_sys_dark = AppTheme::new(ThemeMode::System, true);
        assert_eq!(theme_sys_dark.resolved, ResolvedTheme::Dark);

        let theme_sys_light = AppTheme::new(ThemeMode::System, false);
        assert_eq!(theme_sys_light.resolved, ResolvedTheme::Light);

        let theme_force_dark = AppTheme::new(ThemeMode::Dark, false);
        assert_eq!(theme_force_dark.resolved, ResolvedTheme::Dark);

        let theme_force_light = AppTheme::new(ThemeMode::Light, true);
        assert_eq!(theme_force_light.resolved, ResolvedTheme::Light);
    }
}
