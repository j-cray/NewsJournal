//! COSMIC theme adapter, system mode detector, and accent color bridge.

use serde::{Deserialize, Serialize};

use newsjournal_core::models::ThemeMode;

use crate::theme::{AppTheme, ColorTokens, ResolvedTheme};

/// COSMIC desktop theme variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CosmicThemeMode {
    /// Follows the active COSMIC desktop setting.
    #[default]
    System,
    /// Explicit dark mode.
    Dark,
    /// Explicit light mode.
    Light,
    /// High-contrast dark accessibility mode.
    HighContrastDark,
    /// High-contrast light accessibility mode.
    HighContrastLight,
}

/// Curated COSMIC desktop accent color palettes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CosmicAccentColor {
    /// Classic COSMIC Blue `(58, 123, 213)`.
    CosmicBlue,
    /// System76 Signature Orange `(248, 152, 32)`.
    System76Orange,
    /// Teal / Cyan `(54, 207, 201)`.
    Teal,
    /// Emerald Green `(82, 196, 26)`.
    Emerald,
    /// Purple / Violet `(146, 84, 222)`.
    Violet,
}

impl CosmicAccentColor {
    /// Returns the RGB tuple for the accent color.
    #[must_use]
    pub const fn rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::CosmicBlue => (58, 123, 213),
            Self::System76Orange => (248, 152, 32),
            Self::Teal => (54, 207, 201),
            Self::Emerald => (82, 196, 26),
            Self::Violet => (146, 84, 222),
        }
    }

    /// Returns the hex code string.
    #[must_use]
    pub const fn hex(&self) -> &'static str {
        match self {
            Self::CosmicBlue => "#3a7bd5",
            Self::System76Orange => "#f89820",
            Self::Teal => "#36cfc9",
            Self::Emerald => "#52c41a",
            Self::Violet => "#9254de",
        }
    }
}

/// Adapter bridging COSMIC desktop appearance with NewsJournal's theme engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CosmicThemeAdapter {
    /// Active COSMIC theme mode.
    pub mode: CosmicThemeMode,
    /// Preferred accent color.
    pub accent: CosmicAccentColor,
    /// Whether the underlying desktop reports dark preference.
    pub system_is_dark: bool,
}

impl Default for CosmicThemeAdapter {
    fn default() -> Self {
        Self {
            mode: CosmicThemeMode::System,
            accent: CosmicAccentColor::CosmicBlue,
            system_is_dark: true,
        }
    }
}

impl CosmicThemeAdapter {
    /// Creates a new COSMIC theme adapter.
    #[must_use]
    pub fn new(mode: CosmicThemeMode, accent: CosmicAccentColor, system_is_dark: bool) -> Self {
        Self {
            mode,
            accent,
            system_is_dark,
        }
    }

    /// Converts COSMIC theme mode into domain `ThemeMode`.
    #[must_use]
    pub fn to_core_theme_mode(&self) -> ThemeMode {
        match self.mode {
            CosmicThemeMode::System => ThemeMode::System,
            CosmicThemeMode::Dark | CosmicThemeMode::HighContrastDark => ThemeMode::Dark,
            CosmicThemeMode::Light | CosmicThemeMode::HighContrastLight => ThemeMode::Light,
        }
    }

    /// Translates COSMIC theme settings into a fully resolved `AppTheme`.
    #[must_use]
    pub fn to_app_theme(&self) -> AppTheme {
        let is_dark = match self.mode {
            CosmicThemeMode::System => self.system_is_dark,
            CosmicThemeMode::Dark | CosmicThemeMode::HighContrastDark => true,
            CosmicThemeMode::Light | CosmicThemeMode::HighContrastLight => false,
        };

        let mut colors = if is_dark {
            ColorTokens::dark()
        } else {
            ColorTokens::light()
        };

        // Override brand accent with COSMIC accent selection
        colors.accent = self.accent.rgb();

        // If high contrast mode is selected, sharpen text and borders
        if self.mode == CosmicThemeMode::HighContrastDark {
            colors.text_primary = (255, 255, 255);
            colors.border = (255, 255, 255, 0.28);
        } else if self.mode == CosmicThemeMode::HighContrastLight {
            colors.text_primary = (0, 0, 0);
            colors.border = (0, 0, 0, 0.22);
        }

        AppTheme {
            mode: self.to_core_theme_mode(),
            resolved: if is_dark {
                ResolvedTheme::Dark
            } else {
                ResolvedTheme::Light
            },
            glass: crate::theme::GlassMaterial::default(),
            colors,
        }
    }

    /// Calculates relative luminance for WCAG contrast evaluation.
    #[must_use]
    pub fn relative_luminance(rgb: (u8, u8, u8)) -> f32 {
        let (r, g, b) = (
            f32::from(rgb.0) / 255.0,
            f32::from(rgb.1) / 255.0,
            f32::from(rgb.2) / 255.0,
        );

        let transform = |c: f32| {
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };

        0.2126 * transform(r) + 0.7152 * transform(g) + 0.0722 * transform(b)
    }

    /// Calculates the WCAG contrast ratio between two colors (e.g. text on background).
    #[must_use]
    pub fn contrast_ratio(c1: (u8, u8, u8), c2: (u8, u8, u8)) -> f32 {
        let l1 = Self::relative_luminance(c1);
        let l2 = Self::relative_luminance(c2);
        let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (lighter + 0.05) / (darker + 0.05)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosmic_theme_adapter_resolution() {
        let adapter = CosmicThemeAdapter::new(
            CosmicThemeMode::System,
            CosmicAccentColor::System76Orange,
            true,
        );
        let theme = adapter.to_app_theme();

        assert_eq!(theme.resolved, ResolvedTheme::Dark);
        assert_eq!(theme.colors.accent, (248, 152, 32));

        let light_adapter = CosmicThemeAdapter::new(
            CosmicThemeMode::Light,
            CosmicAccentColor::CosmicBlue,
            true, // system is dark, but mode is Light
        );
        let light_theme = light_adapter.to_app_theme();
        assert_eq!(light_theme.resolved, ResolvedTheme::Light);
        assert_eq!(light_theme.colors.accent, (58, 123, 213));
    }

    #[test]
    fn test_high_contrast_sharpening() {
        let hc_dark = CosmicThemeAdapter::new(
            CosmicThemeMode::HighContrastDark,
            CosmicAccentColor::CosmicBlue,
            true,
        );
        let theme = hc_dark.to_app_theme();
        assert_eq!(theme.colors.text_primary, (255, 255, 255));
        assert_eq!(theme.colors.border.3, 0.28);
    }

    #[test]
    fn test_wcag_contrast_ratio_evaluation() {
        let white = (255, 255, 255);
        let black = (0, 0, 0);
        let ratio = CosmicThemeAdapter::contrast_ratio(white, black);
        assert!(ratio > 20.0); // Maximum contrast ~21:1

        let bg_dark = (18, 20, 24);
        let text_light = (245, 247, 250);
        let dark_ratio = CosmicThemeAdapter::contrast_ratio(bg_dark, text_light);
        assert!(dark_ratio >= 7.0); // Meets WCAG AAA standard
    }
}
