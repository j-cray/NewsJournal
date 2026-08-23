//! System appearance detection engine and background appearance watcher.

use std::env;
use std::sync::Arc;

use crate::theme::ResolvedTheme;

/// Trait or functional callback for pluggable system theme detection providers.
pub type CustomDetectorFn = Arc<dyn Fn() -> Option<bool> + Send + Sync>;

/// Detection strategy options for querying the operating system dark / light appearance.
#[derive(Clone, Default)]
pub enum DetectionStrategy {
    /// Automatic detection based on operating system signals and desktop environment.
    #[default]
    Auto,
    /// Explicit environment variable inspection (`NEWSJOURNAL_THEME`, `GTK_THEME`, etc.).
    EnvironmentOnly,
    /// Static mock value for deterministic testing.
    Static(bool),
    /// Custom dynamic detector callback.
    Custom(CustomDetectorFn),
}

impl std::fmt::Debug for DetectionStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "Auto"),
            Self::EnvironmentOnly => write!(f, "EnvironmentOnly"),
            Self::Static(val) => write!(f, "Static({val})"),
            Self::Custom(_) => write!(f, "Custom(<callback>)"),
        }
    }
}

impl PartialEq for DetectionStrategy {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Auto, Self::Auto) | (Self::EnvironmentOnly, Self::EnvironmentOnly) => true,
            (Self::Static(a), Self::Static(b)) => a == b,
            (Self::Custom(_), Self::Custom(_)) => false,
            _ => false,
        }
    }
}

/// Cross-platform system theme detector for Linux desktop environments and macOS.
///
/// # Examples
///
/// ```
/// use newsjournal_gui::theme::{SystemThemeDetector, ResolvedTheme};
///
/// let detector = SystemThemeDetector::with_static(true);
/// assert!(detector.is_dark());
/// assert_eq!(detector.detect_theme(), ResolvedTheme::Dark);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SystemThemeDetector {
    /// Active detection strategy.
    pub strategy: DetectionStrategy,
    /// Default fallback when system appearance cannot be queried (default: `true` for Dark).
    pub fallback_is_dark: bool,
}

impl SystemThemeDetector {
    /// Creates a new default `SystemThemeDetector` with `Auto` strategy.
    #[must_use]
    pub fn new() -> Self {
        Self {
            strategy: DetectionStrategy::Auto,
            fallback_is_dark: true,
        }
    }

    /// Creates a static detector returning a fixed boolean for dark mode.
    #[must_use]
    pub const fn with_static(is_dark: bool) -> Self {
        Self {
            strategy: DetectionStrategy::Static(is_dark),
            fallback_is_dark: is_dark,
        }
    }

    /// Creates a detector with a custom dynamic detection callback.
    #[must_use]
    pub fn with_custom<F>(f: F) -> Self
    where
        F: Fn() -> Option<bool> + Send + Sync + 'static,
    {
        Self {
            strategy: DetectionStrategy::Custom(Arc::new(f)),
            fallback_is_dark: true,
        }
    }

    /// Creates an environment-only detector.
    #[must_use]
    pub const fn environment_only() -> Self {
        Self {
            strategy: DetectionStrategy::EnvironmentOnly,
            fallback_is_dark: true,
        }
    }

    /// Queries whether the host operating system currently favors dark mode.
    #[must_use]
    pub fn is_dark(&self) -> bool {
        match &self.strategy {
            DetectionStrategy::Static(is_dark) => *is_dark,
            DetectionStrategy::Custom(cb) => cb().unwrap_or(self.fallback_is_dark),
            DetectionStrategy::EnvironmentOnly => {
                Self::detect_from_env().unwrap_or(self.fallback_is_dark)
            }
            DetectionStrategy::Auto => {
                if let Some(env_dark) = Self::detect_from_env() {
                    return env_dark;
                }
                #[cfg(target_os = "macos")]
                if let Some(macos_dark) = Self::detect_macos_dark() {
                    return macos_dark;
                }
                #[cfg(target_os = "linux")]
                if let Some(linux_dark) = Self::detect_linux_dark() {
                    return linux_dark;
                }
                self.fallback_is_dark
            }
        }
    }

    /// Queries the host operating system and resolves to [`ResolvedTheme`].
    #[must_use]
    pub fn detect_theme(&self) -> ResolvedTheme {
        if self.is_dark() {
            ResolvedTheme::Dark
        } else {
            ResolvedTheme::Light
        }
    }

    /// Inspects standard environment variables for theme hints.
    #[must_use]
    pub fn detect_from_env() -> Option<bool> {
        // High priority explicit NewsJournal env vars
        if let Ok(val) = env::var("NEWSJOURNAL_THEME") {
            let normalized = val.trim().to_lowercase();
            if normalized == "dark" || normalized == "1" || normalized == "true" {
                return Some(true);
            }
            if normalized == "light" || normalized == "0" || normalized == "false" {
                return Some(false);
            }
        }

        if let Ok(val) = env::var("NEWSJOURNAL_COLOR_SCHEME") {
            let normalized = val.trim().to_lowercase();
            if normalized.contains("dark") {
                return Some(true);
            }
            if normalized.contains("light") {
                return Some(false);
            }
        }

        // Standard Linux / GTK / terminal hints
        if let Ok(val) = env::var("DARK_MODE") {
            let normalized = val.trim().to_lowercase();
            if normalized == "1" || normalized == "true" {
                return Some(true);
            }
            if normalized == "0" || normalized == "false" {
                return Some(false);
            }
        }

        if let Ok(val) = env::var("GTK_THEME") {
            let normalized = val.to_lowercase();
            if normalized.contains(":dark") || normalized.ends_with("-dark") {
                return Some(true);
            }
            if normalized.contains(":light") || normalized.ends_with("-light") {
                return Some(false);
            }
        }

        None
    }

    /// Queries macOS system preferences for Dark Aqua appearance.
    #[cfg(target_os = "macos")]
    fn detect_macos_dark() -> Option<bool> {
        let output = std::process::Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
            .ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Some(stdout.trim().eq_ignore_ascii_case("Dark"))
        } else {
            // If AppleInterfaceStyle is not set, macOS is in Light Aqua mode
            Some(false)
        }
    }

    /// Queries Linux desktop portals or COSMIC configuration.
    #[cfg(target_os = "linux")]
    fn detect_linux_dark() -> Option<bool> {
        // 1. Check COSMIC desktop configuration file if present
        if let Ok(home) = env::var("HOME") {
            let cosmic_mode_path =
                std::path::Path::new(&home).join(".config/cosmic/com.system76.CosmicTheme.Mode/v1");
            if cosmic_mode_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&cosmic_mode_path) {
                    let trimmed = content.trim().to_lowercase();
                    if trimmed.contains("dark") {
                        return Some(true);
                    }
                    if trimmed.contains("light") {
                        return Some(false);
                    }
                }
            }
        }

        // 2. Query FreeDesktop / GNOME color-scheme via gsettings if available
        if let Ok(output) = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "color-scheme"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("prefer-dark") {
                    return Some(true);
                }
                if stdout.contains("prefer-light") || stdout.contains("default") {
                    return Some(false);
                }
            }
        }

        None
    }
}

/// Periodic appearance watcher that tracks OS theme transitions.
#[derive(Debug, Clone, PartialEq)]
pub struct SystemThemeWatcher {
    /// Detector instance.
    pub detector: SystemThemeDetector,
    /// Last detected dark mode status.
    pub last_is_dark: bool,
}

impl SystemThemeWatcher {
    /// Creates a new `SystemThemeWatcher` initialized with the detector's current state.
    #[must_use]
    pub fn new(detector: SystemThemeDetector) -> Self {
        let initial_dark = detector.is_dark();
        Self {
            detector,
            last_is_dark: initial_dark,
        }
    }

    /// Checks if the operating system appearance has changed since the last check.
    ///
    /// Returns `Some(new_is_dark)` if a change was detected, or `None` if unchanged.
    pub fn check_for_change(&mut self) -> Option<bool> {
        let current_dark = self.detector.is_dark();
        if current_dark != self.last_is_dark {
            self.last_is_dark = current_dark;
            Some(current_dark)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_theme_detector_static() {
        let dark_detector = SystemThemeDetector::with_static(true);
        assert!(dark_detector.is_dark());
        assert_eq!(dark_detector.detect_theme(), ResolvedTheme::Dark);

        let light_detector = SystemThemeDetector::with_static(false);
        assert!(!light_detector.is_dark());
        assert_eq!(light_detector.detect_theme(), ResolvedTheme::Light);
    }

    #[test]
    fn test_system_theme_detector_custom_callback() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let flag = Arc::new(AtomicBool::new(true));
        let flag_clone = Arc::clone(&flag);

        let detector =
            SystemThemeDetector::with_custom(move || Some(flag_clone.load(Ordering::SeqCst)));
        assert!(detector.is_dark());

        flag.store(false, Ordering::SeqCst);
        assert!(!detector.is_dark());
    }

    #[test]
    fn test_system_theme_watcher_change_detection() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let flag = Arc::new(AtomicBool::new(true));
        let flag_clone = Arc::clone(&flag);

        let detector =
            SystemThemeDetector::with_custom(move || Some(flag_clone.load(Ordering::SeqCst)));
        let mut watcher = SystemThemeWatcher::new(detector);

        // No change yet
        assert_eq!(watcher.check_for_change(), None);

        // Change to light
        flag.store(false, Ordering::SeqCst);
        assert_eq!(watcher.check_for_change(), Some(false));
        // Subsequence check reports no change
        assert_eq!(watcher.check_for_change(), None);

        // Change back to dark
        flag.store(true, Ordering::SeqCst);
        assert_eq!(watcher.check_for_change(), Some(true));
        assert_eq!(watcher.check_for_change(), None);
    }
}
