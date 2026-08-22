//! Validation engine for the NewsJournal application.
//!
//! Provides strict slug verification, email formatting, phone number verification and normalization,
//! non-empty text validation, and hex color validation.

pub mod email;
pub mod error;
pub mod phone;
pub mod slug;
pub mod text;

pub use email::{
    is_valid_email, normalize_email, validate_email, MAX_EMAIL_LENGTH, MAX_LOCAL_PART_LENGTH,
};
pub use error::ValidationError;
pub use phone::{
    format_phone_display, is_valid_phone, normalize_phone, validate_phone, MAX_PHONE_DIGITS,
    MAX_PHONE_RAW_LENGTH, MIN_PHONE_DIGITS,
};
pub use slug::{is_valid_slug, slugify, validate_slug, MAX_SLUG_LENGTH};
pub use text::{
    is_valid_hex_color, normalize_hex_color, validate_headline, validate_hex_color,
    validate_max_length, validate_name, validate_non_empty, validate_task_title,
    MAX_HEADLINE_LENGTH, MAX_NAME_LENGTH, MAX_TASK_TITLE_LENGTH,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_module_reexports() {
        assert!(validate_slug("breaking-news-story").is_ok());
        assert!(validate_email("reporter@news.org").is_ok());
        assert!(validate_phone("+1 (555) 019-2834").is_ok());
        assert!(validate_name("Alice Johnson").is_ok());
        assert!(validate_headline("Council Passes Landmark Climate Bill").is_ok());
        assert!(validate_task_title("Conduct interview").is_ok());
        assert!(validate_hex_color("#3498DB").is_ok());
    }
}
