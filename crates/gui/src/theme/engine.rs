//! Unified theme engine managing user preferences, system appearance signals, and live styling resolution.

use newsjournal_core::models::ThemeMode;
use serde::{Deserialize, Serialize};

use crate::theme::detector::SystemThemeDetector;
use crate::theme::{AppTheme, ColorTokens, GlassMaterial, ResolvedTheme};

/// Central theme engine managing dynamic theme resolution, accessibility, and platform synchronization.
///
/// # Examples
///
/// ```
/// use newsjournal_gui::theme::{ThemeEngine, ResolvedTheme};
/// use newsjournal_core::models::ThemeMode;
///
/// let mut engine = ThemeEngine::new(ThemeMode::System, true);
/// assert_eq!(engine.resolved(), ResolvedTheme::Dark);
///
/// engine.set_mode(ThemeMode::Light);
/// assert_eq!(engine.resolved(), ResolvedTheme::Light);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeEngine {
    /// Active user preference mode (`System`, `Light`, `Dark`).
    pub mode: ThemeMode,
    /// Whether the underlying operating system reports dark mode.
    pub system_is_dark: bool,
    /// Fully resolved application theme.
    pub theme: AppTheme,
    /// Optional custom brand/accent color RGB override.
    pub custom_accent: Option<(u8, u8, u8)>,
    /// High-contrast accessibility sharpening toggle.
    pub high_contrast: bool,
}

impl Default for ThemeEngine {
    fn default() -> Self {
        Self::new(ThemeMode::System, true)
    }
}

impl ThemeEngine {
    /// Creates a new `ThemeEngine` with the given mode preference and system dark mode flag.
    #[must_use]
    pub fn new(mode: ThemeMode, system_is_dark: bool) -> Self {
        let mut engine = Self {
            mode,
            system_is_dark,
            theme: AppTheme::new(mode, system_is_dark),
            custom_accent: None,
            high_contrast: false,
        };
        engine.recompute();
        engine
    }

    /// Creates a `ThemeEngine` automatically querying the provided [`SystemThemeDetector`].
    #[must_use]
    pub fn with_detector(mode: ThemeMode, detector: &SystemThemeDetector) -> Self {
        Self::new(mode, detector.is_dark())
    }

    /// Sets the user theme mode preference and recomputes the active theme.
    ///
    /// Returns `true` if the resolved visual appearance (`Light` vs `Dark`) changed.
    pub fn set_mode(&mut self, mode: ThemeMode) -> bool {
        let prev_resolved = self.resolved();
        self.mode = mode;
        self.recompute();
        self.resolved() != prev_resolved
    }

    /// Updates the detected system dark appearance status and recomputes if in `System` mode.
    ///
    /// Returns `true` if the resolved visual appearance changed as a result.
    pub fn set_system_is_dark(&mut self, is_dark: bool) -> bool {
        let prev_resolved = self.resolved();
        self.system_is_dark = is_dark;
        self.recompute();
        self.resolved() != prev_resolved
    }

    /// Synchronizes system appearance with the given detector.
    ///
    /// Returns `true` if a change occurred.
    pub fn sync_with_detector(&mut self, detector: &SystemThemeDetector) -> bool {
        let is_dark = detector.is_dark();
        self.set_system_is_dark(is_dark)
    }

    /// Sets an optional custom brand accent color RGB override.
    pub fn set_custom_accent(&mut self, accent: Option<(u8, u8, u8)>) {
        self.custom_accent = accent;
        self.recompute();
    }

    /// Enables or disables high-contrast accessibility mode.
    pub fn set_high_contrast(&mut self, high_contrast: bool) {
        self.high_contrast = high_contrast;
        self.recompute();
    }

    /// Toggles the active theme mode between Light and Dark.
    ///
    /// If currently in `System` mode, switches to the explicit opposite of the resolved appearance.
    pub fn toggle_mode(&mut self) -> ThemeMode {
        let next_mode = match self.mode {
            ThemeMode::System => {
                if self.is_dark() {
                    ThemeMode::Light
                } else {
                    ThemeMode::Dark
                }
            }
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        };
        self.set_mode(next_mode);
        next_mode
    }

    /// Returns the active resolved theme variant (`Light` or `Dark`).
    #[must_use]
    pub const fn resolved(&self) -> ResolvedTheme {
        self.theme.resolved
    }

    /// Returns whether the resolved theme is dark.
    #[must_use]
    pub fn is_dark(&self) -> bool {
        self.resolved() == ResolvedTheme::Dark
    }

    /// Returns a reference to the active resolved [`AppTheme`].
    #[must_use]
    pub const fn theme(&self) -> &AppTheme {
        &self.theme
    }

    /// Returns a reference to the active [`ColorTokens`].
    #[must_use]
    pub const fn colors(&self) -> &ColorTokens {
        &self.theme.colors
    }

    /// Returns a reference to the active [`GlassMaterial`].
    #[must_use]
    pub const fn glass(&self) -> &GlassMaterial {
        &self.theme.glass
    }

    /// Recomputes `self.theme` based on `mode`, `system_is_dark`, `custom_accent`, and `high_contrast`.
    pub fn recompute(&mut self) {
        let resolved = match self.mode {
            ThemeMode::System => {
                if self.system_is_dark {
                    ResolvedTheme::Dark
                } else {
                    ResolvedTheme::Light
                }
            }
            ThemeMode::Light => ResolvedTheme::Light,
            ThemeMode::Dark => ResolvedTheme::Dark,
        };

        let mut colors = match resolved {
            ResolvedTheme::Dark => ColorTokens::dark(),
            ResolvedTheme::Light => ColorTokens::light(),
        };

        if let Some(custom) = self.custom_accent {
            colors.accent = custom;
        }

        if self.high_contrast {
            match resolved {
                ResolvedTheme::Dark => {
                    colors.text_primary = (255, 255, 255);
                    colors.border = (255, 255, 255, 0.32);
                }
                ResolvedTheme::Light => {
                    colors.text_primary = (0, 0, 0);
                    colors.border = (0, 0, 0, 0.25);
                }
            }
        }

        self.theme = AppTheme {
            mode: self.mode,
            resolved,
            glass: GlassMaterial::default(),
            colors,
        };
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

    /// Calculates the WCAG contrast ratio between two colors (1.0 to 21.0).
    #[must_use]
    pub fn contrast_ratio(c1: (u8, u8, u8), c2: (u8, u8, u8)) -> f32 {
        let l1 = Self::relative_luminance(c1);
        let l2 = Self::relative_luminance(c2);
        let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (lighter + 0.05) / (darker + 0.05)
    }

    /// Evaluates text contrast against background for the active theme.
    #[must_use]
    pub fn text_contrast_ratio(&self) -> f32 {
        Self::contrast_ratio(self.colors().background, self.colors().text_primary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_engine_mode_transitions() {
        let mut engine = ThemeEngine::new(ThemeMode::System, true);
        assert_eq!(engine.resolved(), ResolvedTheme::Dark);
        assert!(engine.is_dark());

        // System changes to light
        let changed = engine.set_system_is_dark(false);
        assert!(changed);
        assert_eq!(engine.resolved(), ResolvedTheme::Light);
        assert!(!engine.is_dark());

        // Force Dark mode
        let changed = engine.set_mode(ThemeMode::Dark);
        assert!(changed);
        assert_eq!(engine.resolved(), ResolvedTheme::Dark);

        // System changes to light, but mode is forced Dark
        let changed = engine.set_system_is_dark(false);
        assert!(!changed);
        assert_eq!(engine.resolved(), ResolvedTheme::Dark);

        // Force Light mode
        let changed = engine.set_mode(ThemeMode::Light);
        assert!(changed);
        assert_eq!(engine.resolved(), ResolvedTheme::Light);
    }

    #[test]
    fn test_theme_engine_toggle() {
        let mut engine = ThemeEngine::new(ThemeMode::System, true);
        assert_eq!(engine.resolved(), ResolvedTheme::Dark);

        let next = engine.toggle_mode();
        assert_eq!(next, ThemeMode::Light);
        assert_eq!(engine.resolved(), ResolvedTheme::Light);

        let next = engine.toggle_mode();
        assert_eq!(next, ThemeMode::Dark);
        assert_eq!(engine.resolved(), ResolvedTheme::Dark);
    }

    #[test]
    fn test_theme_engine_custom_accent() {
        let mut engine = ThemeEngine::new(ThemeMode::Dark, true);
        let custom_orange = (248, 152, 32);
        engine.set_custom_accent(Some(custom_orange));
        assert_eq!(engine.colors().accent, custom_orange);

        engine.set_custom_accent(None);
        assert_ne!(engine.colors().accent, custom_orange);
    }

    #[test]
    fn test_theme_engine_high_contrast() {
        let mut engine = ThemeEngine::new(ThemeMode::Dark, true);
        engine.set_high_contrast(true);
        assert_eq!(engine.colors().text_primary, (255, 255, 255));
        assert_eq!(engine.colors().border.3, 0.32);

        let mut light_engine = ThemeEngine::new(ThemeMode::Light, true);
        light_engine.set_high_contrast(true);
        assert_eq!(light_engine.colors().text_primary, (0, 0, 0));
        assert_eq!(light_engine.colors().border.3, 0.25);
    }

    #[test]
    fn test_theme_engine_wcag_contrast() {
        let dark_engine = ThemeEngine::new(ThemeMode::Dark, true);
        let ratio_dark = dark_engine.text_contrast_ratio();
        assert!(ratio_dark >= 7.0, "Dark theme must meet WCAG AAA");

        let light_engine = ThemeEngine::new(ThemeMode::Light, false);
        let ratio_light = light_engine.text_contrast_ratio();
        assert!(ratio_light >= 7.0, "Light theme must meet WCAG AAA");
    }
}
