//! Cross-platform application directory resolution and path management.
//!
//! Provides standard OS-specific data, configuration, cache, and state directory
//! paths using the `directories` crate (`~/.local/share/newsjournal` on Linux,
//! `~/Library/Application Support/newsjournal` on macOS), along with environment variable
//! overrides and filesystem initialization helpers.

use std::fmt;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use crate::storage::error::StorageError;

/// Standard project qualifier (empty for standard open-source desktop convention).
pub const QUALIFIER: &str = "";

/// Standard organization name.
pub const ORGANIZATION: &str = "";

/// Standard application name.
pub const APPLICATION: &str = "newsjournal";

/// Default SQLite database filename.
pub const DEFAULT_DB_FILENAME: &str = "newsjournal.db";

/// Environment variable to override the data directory.
pub const ENV_DATA_DIR: &str = "NEWSJOURNAL_DATA_DIR";

/// Environment variable to override the config directory.
pub const ENV_CONFIG_DIR: &str = "NEWSJOURNAL_CONFIG_DIR";

/// Environment variable to override the cache directory.
pub const ENV_CACHE_DIR: &str = "NEWSJOURNAL_CACHE_DIR";

/// Environment variable to override the state directory.
pub const ENV_STATE_DIR: &str = "NEWSJOURNAL_STATE_DIR";

/// Environment variable to directly override the SQLite database file path.
pub const ENV_DB_PATH: &str = "NEWSJOURNAL_DB_PATH";

/// Cross-platform application path locator.
///
/// Encapsulates resolved paths for application data (such as the SQLite database),
/// user configurations, caches, and runtime state.
///
/// # Platform Defaults
/// - **Linux**:
///   - Data: `$XDG_DATA_HOME/newsjournal` (defaults to `~/.local/share/newsjournal`)
///   - Config: `$XDG_CONFIG_HOME/newsjournal` (defaults to `~/.config/newsjournal`)
///   - Cache: `$XDG_CACHE_HOME/newsjournal` (defaults to `~/.cache/newsjournal`)
///   - State: `$XDG_STATE_HOME/newsjournal` (defaults to `~/.local/state/newsjournal`)
/// - **macOS**:
///   - Data: `~/Library/Application Support/newsjournal`
///   - Config: `~/Library/Application Support/newsjournal`
///   - Cache: `~/Library/Caches/newsjournal`
/// - **Windows**:
///   - Data: `%APPDATA%\newsjournal\data`
///   - Config: `%APPDATA%\newsjournal\config`
///   - Cache: `%LOCALAPPDATA%\newsjournal\cache`
///
/// # Examples
///
/// ```
/// use newsjournal_core::storage::AppPaths;
///
/// let paths = AppPaths::resolve().expect("failed to resolve app paths");
/// let db_path = paths.database_path();
/// assert!(db_path.ends_with("newsjournal.db"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    data_dir: PathBuf,
    config_dir: PathBuf,
    cache_dir: PathBuf,
    state_dir: Option<PathBuf>,
    custom_db_path: Option<PathBuf>,
}

impl AppPaths {
    /// Constructs a new `AppPaths` instance with explicit directory paths.
    #[must_use]
    pub fn new(
        data_dir: impl Into<PathBuf>,
        config_dir: impl Into<PathBuf>,
        cache_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            data_dir: data_dir.into(),
            config_dir: config_dir.into(),
            cache_dir: cache_dir.into(),
            state_dir: None,
            custom_db_path: None,
        }
    }

    /// Resolves standard application paths from the operating system defaults,
    /// applying any active environment variable overrides (`NEWSJOURNAL_*`).
    ///
    /// This is the primary entry point for normal application startup.
    pub fn resolve() -> Result<Self, StorageError> {
        Self::from_env()
    }

    /// Resolves standard application paths using operating system conventions
    /// without reading environment variable overrides.
    pub fn default_paths() -> Result<Self, StorageError> {
        let (data_dir, config_dir, cache_dir, state_dir) =
            match ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
                Some(dirs) => (
                    dirs.data_dir().to_path_buf(),
                    dirs.config_dir().to_path_buf(),
                    dirs.cache_dir().to_path_buf(),
                    dirs.state_dir().map(Path::to_path_buf),
                ),
                None => {
                    let home = std::env::var_os("HOME")
                        .map(PathBuf::from)
                        .unwrap_or_else(std::env::temp_dir);
                    let base = home.join(format!(".{APPLICATION}"));
                    (
                        base.join("data"),
                        base.join("config"),
                        base.join("cache"),
                        Some(base.join("state")),
                    )
                }
            };

        Ok(Self {
            data_dir,
            config_dir,
            cache_dir,
            state_dir,
            custom_db_path: None,
        })
    }

    /// Resolves application paths inspecting environment variable overrides first:
    /// - `NEWSJOURNAL_DATA_DIR`
    /// - `NEWSJOURNAL_CONFIG_DIR`
    /// - `NEWSJOURNAL_CACHE_DIR`
    /// - `NEWSJOURNAL_STATE_DIR`
    /// - `NEWSJOURNAL_DB_PATH`
    pub fn from_env() -> Result<Self, StorageError> {
        let mut paths = Self::default_paths()?;

        if let Some(data) = std::env::var_os(ENV_DATA_DIR).filter(|v| !v.is_empty()) {
            paths.data_dir = PathBuf::from(data);
        }

        if let Some(config) = std::env::var_os(ENV_CONFIG_DIR).filter(|v| !v.is_empty()) {
            paths.config_dir = PathBuf::from(config);
        }

        if let Some(cache) = std::env::var_os(ENV_CACHE_DIR).filter(|v| !v.is_empty()) {
            paths.cache_dir = PathBuf::from(cache);
        }

        if let Some(state) = std::env::var_os(ENV_STATE_DIR).filter(|v| !v.is_empty()) {
            paths.state_dir = Some(PathBuf::from(state));
        }

        if let Some(db_path) = std::env::var_os(ENV_DB_PATH).filter(|v| !v.is_empty()) {
            paths.custom_db_path = Some(PathBuf::from(db_path));
        }

        Ok(paths)
    }

    /// Creates an isolated `AppPaths` instance rooted under a single directory.
    ///
    /// The directories will be mapped to:
    /// - Data: `{root}/data`
    /// - Config: `{root}/config`
    /// - Cache: `{root}/cache`
    /// - State: `{root}/state`
    ///
    /// Ideal for integration testing, sandboxed execution, and portable mode.
    pub fn from_root(root: impl AsRef<Path>) -> Self {
        let root_ref = root.as_ref();
        Self {
            data_dir: root_ref.join("data"),
            config_dir: root_ref.join("config"),
            cache_dir: root_ref.join("cache"),
            state_dir: Some(root_ref.join("state")),
            custom_db_path: None,
        }
    }

    /// Returns a reference to the application data directory.
    #[must_use]
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Returns a reference to the application configuration directory.
    #[must_use]
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Returns a reference to the application cache directory.
    #[must_use]
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Returns a reference to the optional application state directory.
    #[must_use]
    pub fn state_dir(&self) -> Option<&Path> {
        self.state_dir.as_deref()
    }

    /// Returns the optional custom SQLite database file path if one was explicitly configured.
    #[must_use]
    pub fn custom_database_path(&self) -> Option<&Path> {
        self.custom_db_path.as_deref()
    }

    /// Returns the target SQLite database path.
    ///
    /// If a custom database path was specified, returns that path.
    /// Otherwise, returns `{data_dir}/newsjournal.db`.
    #[must_use]
    pub fn database_path(&self) -> PathBuf {
        if let Some(custom) = &self.custom_db_path {
            custom.clone()
        } else {
            self.data_dir.join(DEFAULT_DB_FILENAME)
        }
    }

    /// Sets a custom database file path.
    #[must_use]
    pub fn with_database_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.custom_db_path = Some(path.into());
        self
    }

    /// Overrides the data directory.
    #[must_use]
    pub fn with_data_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.data_dir = path.into();
        self
    }

    /// Overrides the configuration directory.
    #[must_use]
    pub fn with_config_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.config_dir = path.into();
        self
    }

    /// Overrides the cache directory.
    #[must_use]
    pub fn with_cache_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.cache_dir = path.into();
        self
    }

    /// Overrides the state directory.
    #[must_use]
    pub fn with_state_dir(mut self, path: Option<PathBuf>) -> Self {
        self.state_dir = path;
        self
    }

    /// Ensures that the data directory (and any parent directory for a custom database path) exists.
    ///
    /// Creates the directory hierarchy if it does not already exist.
    pub fn ensure_data_dir(&self) -> Result<&Path, StorageError> {
        std::fs::create_dir_all(&self.data_dir)?;
        if let Some(custom) = &self.custom_db_path {
            if let Some(parent) = custom.parent() {
                std::fs::create_dir_all(parent)?;
            }
        }
        Ok(&self.data_dir)
    }

    /// Ensures that all application directories (data, config, cache, state) exist.
    pub fn ensure_all(&self) -> Result<(), StorageError> {
        self.ensure_data_dir()?;
        std::fs::create_dir_all(&self.config_dir)?;
        std::fs::create_dir_all(&self.cache_dir)?;
        if let Some(state) = &self.state_dir {
            std::fs::create_dir_all(state)?;
        }
        Ok(())
    }
}

impl fmt::Display for AppPaths {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AppPaths {{ db: '{}', data: '{}', config: '{}', cache: '{}' }}",
            self.database_path().display(),
            self.data_dir().display(),
            self.config_dir().display(),
            self.cache_dir().display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_paths_resolution() {
        let paths = AppPaths::default_paths().expect("failed to resolve default paths");
        assert!(!paths.data_dir().as_os_str().is_empty());
        assert!(!paths.config_dir().as_os_str().is_empty());
        assert!(!paths.cache_dir().as_os_str().is_empty());

        let db_path = paths.database_path();
        assert!(db_path.ends_with(DEFAULT_DB_FILENAME));
        assert_eq!(db_path, paths.data_dir().join(DEFAULT_DB_FILENAME));
    }

    #[test]
    fn test_custom_database_path_override() {
        let paths = AppPaths::default_paths()
            .expect("failed to resolve default paths")
            .with_database_path("/custom/path/to/my_news.db");

        assert_eq!(
            paths.database_path(),
            PathBuf::from("/custom/path/to/my_news.db")
        );
        assert_eq!(
            paths.custom_database_path(),
            Some(Path::new("/custom/path/to/my_news.db"))
        );
    }

    #[test]
    fn test_from_root() {
        let root = Path::new("/tmp/test_newsjournal_root");
        let paths = AppPaths::from_root(root);

        assert_eq!(paths.data_dir(), root.join("data"));
        assert_eq!(paths.config_dir(), root.join("config"));
        assert_eq!(paths.cache_dir(), root.join("cache"));
        assert_eq!(paths.state_dir(), Some(root.join("state").as_path()));
        assert_eq!(
            paths.database_path(),
            root.join("data").join(DEFAULT_DB_FILENAME)
        );
    }

    #[test]
    fn test_ensure_data_dir_and_ensure_all() {
        let unique_id = uuid::Uuid::new_v4();
        let temp_dir = std::env::temp_dir().join(format!("newsjournal_test_{unique_id}"));

        let paths = AppPaths::from_root(&temp_dir);
        assert!(!paths.data_dir().exists());
        assert!(!paths.config_dir().exists());
        assert!(!paths.cache_dir().exists());

        paths.ensure_data_dir().expect("failed to ensure data dir");
        assert!(paths.data_dir().exists());
        assert!(!paths.config_dir().exists());

        paths.ensure_all().expect("failed to ensure all dirs");
        assert!(paths.data_dir().exists());
        assert!(paths.config_dir().exists());
        assert!(paths.cache_dir().exists());
        assert!(paths.state_dir().unwrap().exists());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_display_formatting() {
        let paths = AppPaths::new("/tmp/data", "/tmp/config", "/tmp/cache");
        let formatted = format!("{paths}");
        assert!(formatted.contains("/tmp/data/newsjournal.db"));
        assert!(formatted.contains("/tmp/config"));
        assert!(formatted.contains("/tmp/cache"));
    }

    #[test]
    fn test_builder_overrides() {
        let base = AppPaths::new("/data1", "/config1", "/cache1");
        let modified = base
            .with_data_dir("/data2")
            .with_config_dir("/config2")
            .with_cache_dir("/cache2")
            .with_state_dir(Some(PathBuf::from("/state2")));

        assert_eq!(modified.data_dir(), Path::new("/data2"));
        assert_eq!(modified.config_dir(), Path::new("/config2"));
        assert_eq!(modified.cache_dir(), Path::new("/cache2"));
        assert_eq!(modified.state_dir(), Some(Path::new("/state2")));
    }
}
