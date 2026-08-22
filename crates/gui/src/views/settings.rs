//! Settings view model.

use newsjournal_core::models::ThemeMode;
use serde::{Deserialize, Serialize};

use crate::state::AppState;

/// Formatted view model for application settings page and drawer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingsViewModel {
    /// Active theme mode preference.
    pub theme_mode: ThemeMode,
    /// NewsJournal version string.
    pub version: &'static str,
    /// Total number of stored articles.
    pub total_articles: usize,
    /// Total number of stored tasks.
    pub total_tasks: usize,
    /// Total number of stored contacts.
    pub total_contacts: usize,
}

/// Constructs the settings view model from application state.
#[must_use]
pub fn build_settings_view(state: &AppState) -> SettingsViewModel {
    SettingsViewModel {
        theme_mode: state.settings.theme_mode,
        version: newsjournal_core::VERSION,
        total_articles: state.articles.len(),
        total_tasks: state.tasks.len(),
        total_contacts: state.contacts.len(),
    }
}
