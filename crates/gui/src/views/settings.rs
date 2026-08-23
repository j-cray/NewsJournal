//! Settings view model and presentation descriptors for NewsJournal.

use newsjournal_core::models::ThemeMode;
use newsjournal_core::storage::StorageService;
use serde::{Deserialize, Serialize};

use crate::state::AppState;
use crate::theme::{ColorTokens, ResolvedTheme};

/// Color preview swatch values for UI theme cards in the settings view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemePreviewColors {
    /// Background hex color code (e.g. `"#121418"`).
    pub background_hex: String,
    /// Card surface hex color code (e.g. `"#1e222a"`).
    pub surface_hex: String,
    /// Primary text hex color code (e.g. `"#f5f7fa"`).
    pub text_hex: String,
    /// Accent hex color code (e.g. `"#409eff"`).
    pub accent_hex: String,
    /// Border hex color code (e.g. `"#ffffff"`).
    pub border_hex: String,
    /// Whether this preview resolves to a dark appearance.
    pub is_dark: bool,
}

impl ThemePreviewColors {
    /// Constructs color previews for a specific theme mode.
    #[must_use]
    pub fn for_mode(
        mode: ThemeMode,
        system_is_dark: bool,
        custom_accent: Option<(u8, u8, u8)>,
    ) -> Self {
        let is_dark = match mode {
            ThemeMode::System => system_is_dark,
            ThemeMode::Light => false,
            ThemeMode::Dark => true,
        };

        let tokens = if is_dark {
            ColorTokens::dark()
        } else {
            ColorTokens::light()
        };

        let accent_rgb = custom_accent.unwrap_or(tokens.accent);

        Self {
            background_hex: format!(
                "#{:02x}{:02x}{:02x}",
                tokens.background.0, tokens.background.1, tokens.background.2
            ),
            surface_hex: format!(
                "#{:02x}{:02x}{:02x}",
                tokens.surface.0, tokens.surface.1, tokens.surface.2
            ),
            text_hex: format!(
                "#{:02x}{:02x}{:02x}",
                tokens.text_primary.0, tokens.text_primary.1, tokens.text_primary.2
            ),
            accent_hex: format!(
                "#{:02x}{:02x}{:02x}",
                accent_rgb.0, accent_rgb.1, accent_rgb.2
            ),
            border_hex: format!(
                "#{:02x}{:02x}{:02x}",
                tokens.border.0, tokens.border.1, tokens.border.2
            ),
            is_dark,
        }
    }
}

/// Formatted view descriptor for an individual selectable theme mode option.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeOptionViewModel {
    /// Associated theme mode enum value.
    pub mode: ThemeMode,
    /// Human-readable title (e.g. "System", "Light", "Dark").
    pub title: &'static str,
    /// Explanatory description of the theme mode behavior.
    pub description: &'static str,
    /// Icon identifier for GUI renderers.
    pub icon_name: &'static str,
    /// Whether this option is currently selected.
    pub is_selected: bool,
    /// Preview color swatches for card rendering.
    pub preview_colors: ThemePreviewColors,
}

/// Operational connection health status for the SQLite database.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatabaseHealthStatus {
    /// Connected and read/write verified.
    Connected,
    /// Transient in-memory database.
    InMemory,
    /// Error accessing or reading storage.
    Error,
}

impl DatabaseHealthStatus {
    /// Human-readable status label.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Connected => "Connected (Read/Write)",
            Self::InMemory => "In-Memory (Transient)",
            Self::Error => "Storage Error",
        }
    }

    /// Color hex code for UI badge indicator.
    #[must_use]
    pub const fn badge_color_hex(&self) -> &'static str {
        match self {
            Self::Connected => "#52c41a", // Green
            Self::InMemory => "#1890ff",  // Blue
            Self::Error => "#ff4d4f",     // Red
        }
    }
}

/// Diagnostic metadata and statistics for SQLite persistence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatabaseStatusViewModel {
    /// Operational status of the database connection.
    pub status: DatabaseHealthStatus,
    /// Human-readable status label.
    pub status_label: &'static str,
    /// Badge accent hex color.
    pub status_badge_color: &'static str,
    /// Absolute filesystem path or `":memory:"`.
    pub path: String,
    /// Whether database is running in-memory.
    pub is_in_memory: bool,
    /// Database file size in bytes if file-backed.
    pub file_size_bytes: Option<u64>,
    /// Formatted human-readable file size (e.g. `"124.5 KB"` or `"In-Memory Transient"`).
    pub file_size_formatted: String,
    /// Active schema version number.
    pub schema_version: Option<i64>,
    /// Formatted schema version label.
    pub schema_version_formatted: String,
    /// Total count of applied schema migrations.
    pub applied_migrations_count: usize,
    /// Total number of stored articles in database.
    pub total_articles: usize,
    /// Total number of stored tasks in database.
    pub total_tasks: usize,
    /// Total number of stored contacts in database.
    pub total_contacts: usize,
    /// Total number of article-contact links.
    pub total_article_contacts: usize,
}

/// Version and runtime target information descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppVersionInfo {
    /// Application product name.
    pub app_name: &'static str,
    /// Current application release version string.
    pub version: &'static str,
    /// NewsJournal core crate version.
    pub core_version: &'static str,
    /// Running operating system and glass rendering architecture.
    pub platform_target: &'static str,
    /// Build profile ("debug" or "release").
    pub build_profile: &'static str,
    /// SQLite engine version string.
    pub sqlite_version: &'static str,
}

/// Formatted view model for application settings page and drawer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingsViewModel {
    /// Active theme mode preference (`System`, `Light`, `Dark`).
    pub theme_mode: ThemeMode,
    /// Active resolved theme appearance (`true` if Dark, `false` if Light).
    pub resolved_theme_is_dark: bool,
    /// System OS dark appearance status.
    pub system_is_dark: bool,
    /// Whether high-contrast accessibility mode is active.
    pub high_contrast: bool,
    /// Custom accent hex color override, if set.
    pub custom_accent_hex: Option<String>,
    /// List of available theme mode options with preview colors.
    pub theme_options: Vec<ThemeOptionViewModel>,
    /// SQLite database health and diagnostics.
    pub database: DatabaseStatusViewModel,
    /// Application and engine version info.
    pub app_info: AppVersionInfo,
    /// NewsJournal version string.
    pub version: &'static str,
    /// Total number of stored articles.
    pub total_articles: usize,
    /// Total number of stored tasks.
    pub total_tasks: usize,
    /// Total number of stored contacts.
    pub total_contacts: usize,
}

/// Formats a byte size into a human-readable string (B, KB, MB, GB).
#[must_use]
pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;
    if b < KB {
        format!("{bytes} B")
    } else if b < MB {
        format!("{:.1} KB", b / KB)
    } else if b < GB {
        format!("{:.1} MB", b / MB)
    } else {
        format!("{:.2} GB", b / GB)
    }
}

/// Returns the default platform description based on compilation target.
#[must_use]
pub const fn default_platform_target() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        "Linux (COSMIC Frosted Glass)"
    }
    #[cfg(target_os = "macos")]
    {
        "macOS (Liquid Glass)"
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        "Cross-Platform Desktop"
    }
}

/// Constructs the settings view model from application state using default platform target.
#[must_use]
pub fn build_settings_view(state: &AppState) -> SettingsViewModel {
    build_settings_view_with_platform(state, default_platform_target())
}

/// Constructs the settings view model from application state for a specified platform target.
#[must_use]
pub fn build_settings_view_with_platform(
    state: &AppState,
    platform_target: &'static str,
) -> SettingsViewModel {
    let active_mode = state.settings.theme_mode;
    let system_is_dark = state.theme_engine.system_is_dark;
    let custom_accent = state.theme_engine.custom_accent;
    let resolved_is_dark = state.theme_engine.resolved() == ResolvedTheme::Dark;
    let high_contrast = state.theme_engine.high_contrast;

    let custom_accent_hex =
        custom_accent.map(|rgb| format!("#{:02x}{:02x}{:02x}", rgb.0, rgb.1, rgb.2));

    // Build Theme Option Cards
    let theme_options = vec![
        ThemeOptionViewModel {
            mode: ThemeMode::System,
            title: "System",
            description: "Automatically matches your operating system appearance.",
            icon_name: "preferences-system",
            is_selected: active_mode == ThemeMode::System,
            preview_colors: ThemePreviewColors::for_mode(
                ThemeMode::System,
                system_is_dark,
                custom_accent,
            ),
        },
        ThemeOptionViewModel {
            mode: ThemeMode::Light,
            title: "Light",
            description: "Crisp, bright canvas with high-contrast text and clean glass surfaces.",
            icon_name: "weather-clear",
            is_selected: active_mode == ThemeMode::Light,
            preview_colors: ThemePreviewColors::for_mode(
                ThemeMode::Light,
                system_is_dark,
                custom_accent,
            ),
        },
        ThemeOptionViewModel {
            mode: ThemeMode::Dark,
            title: "Dark",
            description: "Vibrant frosted glass with deep charcoal backgrounds and neon accents.",
            icon_name: "weather-clear-night",
            is_selected: active_mode == ThemeMode::Dark,
            preview_colors: ThemePreviewColors::for_mode(
                ThemeMode::Dark,
                system_is_dark,
                custom_accent,
            ),
        },
    ];

    // Build Database Status
    let db_path = state.storage.database_path();
    let is_in_memory = state.storage.is_in_memory();
    let (path_str, status, file_size_bytes, file_size_formatted) = if is_in_memory {
        (
            ":memory:".to_string(),
            DatabaseHealthStatus::InMemory,
            None,
            "In-Memory Transient".to_string(),
        )
    } else if let Some(path) = db_path {
        let path_string = path.to_string_lossy().to_string();
        let size_bytes = state.storage.database_file_size().ok().flatten();
        let formatted = size_bytes
            .map(format_bytes)
            .unwrap_or_else(|| "0 B".to_string());
        (
            path_string,
            DatabaseHealthStatus::Connected,
            size_bytes,
            formatted,
        )
    } else {
        (
            ":memory:".to_string(),
            DatabaseHealthStatus::InMemory,
            None,
            "In-Memory Transient".to_string(),
        )
    };

    let schema_version = state.storage.current_schema_version().ok().flatten();
    let schema_version_formatted = schema_version
        .map(|v| format!("v{v} (000{v}_initial_schema)"))
        .unwrap_or_else(|| "Unversioned".to_string());

    let applied_migrations_count = state.storage.applied_migrations_count().unwrap_or(0);
    let counts = state.storage.entity_counts().unwrap_or_default();

    let database = DatabaseStatusViewModel {
        status,
        status_label: status.display_name(),
        status_badge_color: status.badge_color_hex(),
        path: path_str,
        is_in_memory,
        file_size_bytes,
        file_size_formatted,
        schema_version,
        schema_version_formatted,
        applied_migrations_count,
        total_articles: counts.total_articles,
        total_tasks: counts.total_tasks,
        total_contacts: counts.total_contacts,
        total_article_contacts: counts.total_article_contacts,
    };

    let build_profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };

    let app_info = AppVersionInfo {
        app_name: "NewsJournal",
        version: newsjournal_core::VERSION,
        core_version: newsjournal_core::VERSION,
        platform_target,
        build_profile,
        sqlite_version: StorageService::sqlite_version(),
    };

    SettingsViewModel {
        theme_mode: active_mode,
        resolved_theme_is_dark: resolved_is_dark,
        system_is_dark,
        high_contrast,
        custom_accent_hex,
        theme_options,
        database,
        app_info,
        version: newsjournal_core::VERSION,
        total_articles: state.articles.len(),
        total_tasks: state.tasks.len(),
        total_contacts: state.contacts.len(),
    }
}
