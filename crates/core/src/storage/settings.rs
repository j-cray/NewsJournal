//! SQLite repository operations for user Settings and configuration key-value pairs.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use crate::models::{Settings, ThemeMode};
use crate::storage::error::StorageError;

const SETTING_THEME_MODE_KEY: &str = "theme_mode";

/// Fetches a raw configuration string value by its key.
pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, StorageError> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let val = stmt.query_row(params![key], |row| row.get(0)).optional()?;
    Ok(val)
}

/// Sets or updates a configuration string value by its key with current UTC timestamp.
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), StorageError> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO settings (key, value, updated_at)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![key, value, now],
    )?;
    Ok(())
}

/// Fetches the current application settings, initializing default values in the database if not present.
pub fn get_settings(conn: &Connection) -> Result<Settings, StorageError> {
    let row: Option<(String, String)> = conn
        .query_row(
            "SELECT value, updated_at FROM settings WHERE key = ?1",
            params![SETTING_THEME_MODE_KEY],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;

    match row {
        Some((val, updated_at_str)) => {
            let theme_mode: ThemeMode = val.parse().unwrap_or_default();
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(Settings {
                theme_mode,
                updated_at,
            })
        }
        None => {
            let default_settings = Settings::default();
            save_settings(conn, &default_settings)?;
            Ok(default_settings)
        }
    }
}

/// Persists the complete `Settings` entity to the database.
pub fn save_settings(conn: &Connection, settings: &Settings) -> Result<(), StorageError> {
    settings.validate()?;
    let now_str = settings.updated_at.to_rfc3339();
    conn.execute(
        "INSERT INTO settings (key, value, updated_at)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![
            SETTING_THEME_MODE_KEY,
            settings.theme_mode.as_str(),
            now_str
        ],
    )?;
    Ok(())
}

/// Reads the current configured theme mode.
pub fn get_theme_mode(conn: &Connection) -> Result<ThemeMode, StorageError> {
    let settings = get_settings(conn)?;
    Ok(settings.theme_mode)
}

/// Sets and persists the user interface theme mode.
pub fn set_theme_mode(conn: &Connection, mode: ThemeMode) -> Result<(), StorageError> {
    set_setting(conn, SETTING_THEME_MODE_KEY, mode.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::connection::open_in_memory;

    #[test]
    fn test_settings_default_and_updates() {
        let conn = open_in_memory().expect("failed to open db");

        // Initial settings should default to System
        let settings = get_settings(&conn).expect("failed to get default settings");
        assert_eq!(settings.theme_mode, ThemeMode::System);

        // Update to Dark
        set_theme_mode(&conn, ThemeMode::Dark).expect("failed to set dark theme");
        let updated_mode = get_theme_mode(&conn).expect("failed to get theme mode");
        assert_eq!(updated_mode, ThemeMode::Dark);

        // Update via save_settings
        let new_settings = Settings::new(ThemeMode::Light);
        save_settings(&conn, &new_settings).expect("failed to save settings");
        let after_save = get_settings(&conn).expect("failed to get settings");
        assert_eq!(after_save.theme_mode, ThemeMode::Light);

        // Generic key-value settings
        assert_eq!(get_setting(&conn, "custom_key").unwrap(), None);
        set_setting(&conn, "custom_key", "custom_val").unwrap();
        assert_eq!(
            get_setting(&conn, "custom_key").unwrap(),
            Some("custom_val".to_string())
        );
        set_setting(&conn, "custom_key", "updated_val").unwrap();
        assert_eq!(
            get_setting(&conn, "custom_key").unwrap(),
            Some("updated_val".to_string())
        );
    }
}
