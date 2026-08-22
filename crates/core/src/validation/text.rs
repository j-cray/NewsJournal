//! Text, headline, name, and hex color validation utilities.

use super::error::ValidationError;

/// Maximum character limit for contact names.
pub const MAX_NAME_LENGTH: usize = 200;

/// Maximum character limit for article headlines.
pub const MAX_HEADLINE_LENGTH: usize = 500;

/// Maximum character limit for task titles.
pub const MAX_TASK_TITLE_LENGTH: usize = 300;

/// Validates that a string field is not empty or composed solely of whitespace.
///
/// # Examples
///
/// ```
/// use newsjournal_core::validation::validate_non_empty;
///
/// assert!(validate_non_empty("headline", "Breaking News").is_ok());
/// assert!(validate_non_empty("headline", "   ").is_err());
/// ```
pub fn validate_non_empty(field: &'static str, value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        Err(ValidationError::EmptyField { field })
    } else {
        Ok(())
    }
}

/// Validates that a string does not exceed a specified maximum length in characters.
///
/// # Examples
///
/// ```
/// use newsjournal_core::validation::validate_max_length;
///
/// assert!(validate_max_length("title", "Short Title", 50).is_ok());
/// assert!(validate_max_length("title", "Way Too Long Title", 5).is_err());
/// ```
pub fn validate_max_length(
    field: &'static str,
    value: &str,
    max: usize,
) -> Result<(), ValidationError> {
    let actual = value.chars().count();
    if actual > max {
        Err(ValidationError::FieldTooLong { field, max, actual })
    } else {
        Ok(())
    }
}

/// Validates a contact or person's name.
///
/// Ensures name is non-empty after trimming and does not exceed [`MAX_NAME_LENGTH`].
///
/// # Examples
///
/// ```
/// use newsjournal_core::validation::validate_name;
///
/// assert!(validate_name("Jane Doe").is_ok());
/// assert!(validate_name("").is_err());
/// ```
pub fn validate_name(name: &str) -> Result<(), ValidationError> {
    validate_non_empty("name", name)?;
    validate_max_length("name", name, MAX_NAME_LENGTH)
}

/// Validates an article headline.
///
/// Ensures headline is non-empty after trimming and does not exceed [`MAX_HEADLINE_LENGTH`].
///
/// # Examples
///
/// ```
/// use newsjournal_core::validation::validate_headline;
///
/// assert!(validate_headline("City Transit Authority Announces Expansion").is_ok());
/// assert!(validate_headline("").is_err());
/// ```
pub fn validate_headline(headline: &str) -> Result<(), ValidationError> {
    validate_non_empty("headline", headline)?;
    validate_max_length("headline", headline, MAX_HEADLINE_LENGTH)
}

/// Validates a task title.
///
/// Ensures title is non-empty after trimming and does not exceed [`MAX_TASK_TITLE_LENGTH`].
///
/// # Examples
///
/// ```
/// use newsjournal_core::validation::validate_task_title;
///
/// assert!(validate_task_title("Fact-check mayor's statement").is_ok());
/// assert!(validate_task_title("  ").is_err());
/// ```
pub fn validate_task_title(title: &str) -> Result<(), ValidationError> {
    validate_non_empty("title", title)?;
    validate_max_length("title", title, MAX_TASK_TITLE_LENGTH)
}

/// Validates a hex color code (e.g. `#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA`).
///
/// # Examples
///
/// ```
/// use newsjournal_core::validation::validate_hex_color;
///
/// assert!(validate_hex_color("#4A90E2").is_ok());
/// assert!(validate_hex_color("#FFF").is_ok());
/// assert!(validate_hex_color("#12345678").is_ok());
///
/// assert!(validate_hex_color("4A90E2").is_err()); // Missing leading '#'
/// assert!(validate_hex_color("#GGGGGG").is_err()); // Non-hex characters
/// assert!(validate_hex_color("#12").is_err()); // Invalid length
/// ```
pub fn validate_hex_color(color: &str) -> Result<(), ValidationError> {
    let trimmed = color.trim();

    if !trimmed.starts_with('#') {
        return Err(ValidationError::InvalidColor {
            color: color.to_string(),
            reason: "hex color code must start with '#'".to_string(),
        });
    }

    let hex_part = &trimmed[1..];
    let len = hex_part.len();
    if len != 3 && len != 4 && len != 6 && len != 8 {
        return Err(ValidationError::InvalidColor {
            color: color.to_string(),
            reason: format!("hex color must have 3, 4, 6, or 8 hexadecimal digits (found {len})"),
        });
    }

    for ch in hex_part.chars() {
        if !ch.is_ascii_hexdigit() {
            return Err(ValidationError::InvalidColor {
                color: color.to_string(),
                reason: format!("hex color contains non-hexadecimal character '{ch}'"),
            });
        }
    }

    Ok(())
}

/// Checks whether a color string is a valid hex color code according to [`validate_hex_color`].
///
/// # Examples
///
/// ```
/// use newsjournal_core::validation::is_valid_hex_color;
///
/// assert!(is_valid_hex_color("#2ECC71"));
/// assert!(!is_valid_hex_color("not-a-color"));
/// ```
#[must_use]
pub fn is_valid_hex_color(color: &str) -> bool {
    validate_hex_color(color).is_ok()
}

/// Normalizes a hex color string into standard uppercase format.
///
/// Expands 3-digit `#RGB` hex codes to 6-digit `#RRGGBB` format.
///
/// # Examples
///
/// ```
/// use newsjournal_core::validation::normalize_hex_color;
///
/// assert_eq!(normalize_hex_color("#abc").unwrap(), "#AABBCC");
/// assert_eq!(normalize_hex_color("#4a90e2").unwrap(), "#4A90E2");
/// ```
pub fn normalize_hex_color(color: &str) -> Result<String, ValidationError> {
    validate_hex_color(color)?;
    let trimmed = color.trim();
    let hex_part = &trimmed[1..];

    if hex_part.len() == 3 {
        let mut chars = hex_part.chars();
        let r = chars.next().unwrap();
        let g = chars.next().unwrap();
        let b = chars.next().unwrap();
        Ok(format!("#{r}{r}{g}{g}{b}{b}").to_uppercase())
    } else {
        Ok(trimmed.to_uppercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_non_empty() {
        assert!(validate_non_empty("test", "hello").is_ok());
        assert!(validate_non_empty("test", " a ").is_ok());

        assert!(matches!(
            validate_non_empty("headline", ""),
            Err(ValidationError::EmptyField { field: "headline" })
        ));
        assert!(matches!(
            validate_non_empty("name", "    "),
            Err(ValidationError::EmptyField { field: "name" })
        ));
    }

    #[test]
    fn test_validate_max_length() {
        assert!(validate_max_length("slug", "abc", 5).is_ok());
        assert!(validate_max_length("slug", "abcde", 5).is_ok());
        assert!(matches!(
            validate_max_length("slug", "abcdef", 5),
            Err(ValidationError::FieldTooLong {
                field: "slug",
                max: 5,
                actual: 6
            })
        ));
    }

    #[test]
    fn test_validate_headline_name_task_title() {
        assert!(validate_name("Alice Journalist").is_ok());
        assert!(validate_name("").is_err());
        assert!(validate_name(&"x".repeat(MAX_NAME_LENGTH + 1)).is_err());

        assert!(validate_headline("Mayoral Candidate Announces Platform").is_ok());
        assert!(validate_headline("").is_err());
        assert!(validate_headline(&"h".repeat(MAX_HEADLINE_LENGTH + 1)).is_err());

        assert!(validate_task_title("Contact city spokesperson").is_ok());
        assert!(validate_task_title("").is_err());
        assert!(validate_task_title(&"t".repeat(MAX_TASK_TITLE_LENGTH + 1)).is_err());
    }

    #[test]
    fn test_hex_colors() {
        let valid = ["#FFF", "#fff", "#123", "#4A90E2", "#2ecc71", "#12345678"];
        for color in valid {
            assert!(
                validate_hex_color(color).is_ok(),
                "color '{color}' should be valid"
            );
            assert!(is_valid_hex_color(color));
        }

        let invalid = ["", "FFF", "#12", "#12345", "#GGGGGG", "# 123", "blue"];
        for color in invalid {
            assert!(
                validate_hex_color(color).is_err(),
                "color '{color}' should be invalid"
            );
            assert!(!is_valid_hex_color(color));
        }
    }

    #[test]
    fn test_normalize_hex_color() {
        assert_eq!(normalize_hex_color("#f0a").unwrap(), "#FF00AA");
        assert_eq!(normalize_hex_color("#4a90e2").unwrap(), "#4A90E2");
        assert!(normalize_hex_color("invalid").is_err());
    }
}
