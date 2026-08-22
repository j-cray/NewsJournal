//! User settings and application theme configuration models.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::ModelError;

/// Theme mode preference for the desktop user interface.
///
/// # Examples
///
/// ```
/// use newsjournal_core::ThemeMode;
///
/// let mode = ThemeMode::System;
/// assert_eq!(mode.display_name(), "System");
/// assert_eq!("dark".parse::<ThemeMode>().unwrap(), ThemeMode::Dark);
/// ```
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    /// Automatically match the operating system dark / light appearance.
    #[default]
    System,
    /// Force light theme mode.
    Light,
    /// Force dark theme mode.
    Dark,
}

impl ThemeMode {
    /// Returns all supported theme modes.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::System, Self::Light, Self::Dark]
    }

    /// Returns the canonical machine-readable slug for the theme mode.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    /// Returns the human-readable display title for the theme mode.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }
}

impl fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl FromStr for ThemeMode {
    type Err = ModelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_lowercase();
        match normalized.as_str() {
            "system" | "auto" | "os" => Ok(Self::System),
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            _ => Err(ModelError::InvalidThemeMode(s.to_string())),
        }
    }
}

/// Persistent user settings and preferences.
///
/// # Examples
///
/// ```
/// use newsjournal_core::{Settings, ThemeMode};
///
/// let mut settings = Settings::default();
/// assert_eq!(settings.theme_mode, ThemeMode::System);
///
/// settings.set_theme_mode(ThemeMode::Dark);
/// assert_eq!(settings.theme_mode, ThemeMode::Dark);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// Selected UI theme appearance.
    pub theme_mode: ThemeMode,
    /// Timestamp when settings were last updated.
    pub updated_at: DateTime<Utc>,
}

impl Settings {
    /// Creates a new `Settings` instance with the specified theme mode and current timestamp.
    #[must_use]
    pub fn new(theme_mode: ThemeMode) -> Self {
        Self {
            theme_mode,
            updated_at: Utc::now(),
        }
    }

    /// Sets the theme mode and updates the `updated_at` timestamp.
    pub fn set_theme_mode(&mut self, theme_mode: ThemeMode) {
        self.theme_mode = theme_mode;
        self.touch();
    }

    /// Updates the `updated_at` timestamp to the current UTC time.
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Validates all field invariants of settings.
    pub fn validate(&self) -> Result<(), crate::validation::ValidationError> {
        Ok(())
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::default(),
            updated_at: Utc::now(),
        }
    }
}
