//! macOS theme adapter, appearance mode detector, and system accent color bridge.

use serde::{Deserialize, Serialize};

use newsjournal_core::models::ThemeMode;

use crate::theme::{AppTheme, ColorTokens, ResolvedTheme};

/// macOS desktop appearance modes (`NSAppearanceName`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MacosAppearanceMode {
    /// Follows the active macOS system appearance.
    #[default]
    System,
    /// Standard macOS Aqua appearance (Light).
    Aqua,
    /// macOS Dark Aqua appearance (Dark).
    DarkAqua,
    /// macOS Vibrant Light appearance for translucent glass overlays.
    VibrantLight,
    /// macOS Vibrant Dark appearance for dark translucent glass overlays.
    VibrantDark,
    /// Accessibility High Contrast Aqua (Light).
    AccessibilityHighContrastAqua,
    /// Accessibility High Contrast Dark Aqua (Dark).
    AccessibilityHighContrastDarkAqua,
}

/// Curated macOS system accent color choices (`NSColor.controlAccentColor`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MacosAccentColor {
    /// Classic Apple Blue `(0, 122, 255)`.
    #[default]
    Blue,
    /// Apple Purple `(175, 82, 222)`.
    Purple,
    /// Apple Pink `(255, 45, 85)`.
    Pink,
    /// Apple Red `(255, 59, 48)`.
    Red,
    /// Apple Orange `(255, 149, 0)`.
    Orange,
    /// Apple Yellow `(255, 204, 0)`.
    Yellow,
    /// Apple Green `(52, 199, 89)`.
    Green,
    /// Apple Graphite / Neutral `(142, 142, 147)`.
    Graphite,
    /// Multicolor accent (AppKit standard).
    Multicolor,
}

impl MacosAccentColor {
    /// Returns the RGB tuple for the accent color.
    #[must_use]
    pub const fn rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::Blue | Self::Multicolor => (0, 122, 255),
            Self::Purple => (175, 82, 222),
            Self::Pink => (255, 45, 85),
            Self::Red => (255, 59, 48),
            Self::Orange => (255, 149, 0),
            Self::Yellow => (255, 204, 0),
            Self::Green => (52, 199, 89),
            Self::Graphite => (142, 142, 147),
        }
    }

    /// Returns the hex code string.
    #[must_use]
    pub const fn hex(&self) -> &'static str {
        match self {
            Self::Blue | Self::Multicolor => "#007aff",
            Self::Purple => "#af52de",
            Self::Pink => "#ff2d55",
            Self::Red => "#ff3b30",
            Self::Orange => "#ff9500",
            Self::Yellow => "#ffcc00",
            Self::Green => "#34c759",
            Self::Graphite => "#8e8e93",
        }
    }
}

/// Adapter bridging macOS system appearance with NewsJournal's unified theme engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacosThemeAdapter {
    /// Active macOS appearance mode.
    pub appearance: MacosAppearanceMode,
    /// Preferred accent color.
    pub accent: MacosAccentColor,
    /// Whether the underlying macOS environment reports dark mode.
    pub system_is_dark: bool,
}

impl Default for MacosThemeAdapter {
    fn default() -> Self {
        Self {
            appearance: MacosAppearanceMode::System,
            accent: MacosAccentColor::Blue,
            system_is_dark: true,
        }
    }
}

impl MacosThemeAdapter {
    /// Creates a new macOS theme adapter.
    #[must_use]
    pub fn new(
        appearance: MacosAppearanceMode,
        accent: MacosAccentColor,
        system_is_dark: bool,
    ) -> Self {
        Self {
            appearance,
            accent,
            system_is_dark,
        }
    }

    /// Converts macOS appearance mode into domain `ThemeMode`.
    #[must_use]
    pub fn to_core_theme_mode(&self) -> ThemeMode {
        match self.appearance {
            MacosAppearanceMode::System => ThemeMode::System,
            MacosAppearanceMode::DarkAqua
            | MacosAppearanceMode::VibrantDark
            | MacosAppearanceMode::AccessibilityHighContrastDarkAqua => ThemeMode::Dark,
            MacosAppearanceMode::Aqua
            | MacosAppearanceMode::VibrantLight
            | MacosAppearanceMode::AccessibilityHighContrastAqua => ThemeMode::Light,
        }
    }

    /// Translates macOS theme settings into a fully resolved `AppTheme`.
    #[must_use]
    pub fn to_app_theme(&self) -> AppTheme {
        let is_dark = match self.appearance {
            MacosAppearanceMode::System => self.system_is_dark,
            MacosAppearanceMode::DarkAqua
            | MacosAppearanceMode::VibrantDark
            | MacosAppearanceMode::AccessibilityHighContrastDarkAqua => true,
            MacosAppearanceMode::Aqua
            | MacosAppearanceMode::VibrantLight
            | MacosAppearanceMode::AccessibilityHighContrastAqua => false,
        };

        let mut colors = if is_dark {
            ColorTokens::dark()
        } else {
            ColorTokens::light()
        };

        // Override brand accent with macOS accent selection
        colors.accent = self.accent.rgb();

        // High contrast accessibility modes
        if self.appearance == MacosAppearanceMode::AccessibilityHighContrastDarkAqua {
            colors.text_primary = (255, 255, 255);
            colors.border = (255, 255, 255, 0.30);
        } else if self.appearance == MacosAppearanceMode::AccessibilityHighContrastAqua {
            colors.text_primary = (0, 0, 0);
            colors.border = (0, 0, 0, 0.25);
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
    fn test_macos_theme_adapter_resolution() {
        let adapter =
            MacosThemeAdapter::new(MacosAppearanceMode::System, MacosAccentColor::Purple, true);
        let theme = adapter.to_app_theme();

        assert_eq!(theme.resolved, ResolvedTheme::Dark);
        assert_eq!(theme.colors.accent, (175, 82, 222));

        let light_adapter =
            MacosThemeAdapter::new(MacosAppearanceMode::Aqua, MacosAccentColor::Green, true);
        let light_theme = light_adapter.to_app_theme();
        assert_eq!(light_theme.resolved, ResolvedTheme::Light);
        assert_eq!(light_theme.colors.accent, (52, 199, 89));
    }

    #[test]
    fn test_macos_accents_and_hex() {
        let accents = [
            (MacosAccentColor::Blue, (0, 122, 255), "#007aff"),
            (MacosAccentColor::Purple, (175, 82, 222), "#af52de"),
            (MacosAccentColor::Pink, (255, 45, 85), "#ff2d55"),
            (MacosAccentColor::Red, (255, 59, 48), "#ff3b30"),
            (MacosAccentColor::Orange, (255, 149, 0), "#ff9500"),
            (MacosAccentColor::Yellow, (255, 204, 0), "#ffcc00"),
            (MacosAccentColor::Green, (52, 199, 89), "#34c759"),
            (MacosAccentColor::Graphite, (142, 142, 147), "#8e8e93"),
            (MacosAccentColor::Multicolor, (0, 122, 255), "#007aff"),
        ];

        for (accent, rgb, hex) in accents {
            assert_eq!(accent.rgb(), rgb);
            assert_eq!(accent.hex(), hex);
        }
    }

    #[test]
    fn test_macos_high_contrast_mode() {
        let hc_dark = MacosThemeAdapter::new(
            MacosAppearanceMode::AccessibilityHighContrastDarkAqua,
            MacosAccentColor::Blue,
            true,
        );
        let theme = hc_dark.to_app_theme();
        assert_eq!(theme.colors.text_primary, (255, 255, 255));
        assert_eq!(theme.colors.border.3, 0.30);
    }
}
