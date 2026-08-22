//! Slug validation and slugification utilities.

use super::error::ValidationError;

/// Maximum allowed length for an article slug.
pub const MAX_SLUG_LENGTH: usize = 128;

/// Validates that a string conforms to strict slug formatting requirements.
///
/// A valid slug:
/// - Is non-empty and does not exceed [`MAX_SLUG_LENGTH`] characters (128).
/// - Contains only ASCII lowercase letters (`a-z`), digits (`0-9`), hyphens (`-`), and underscores (`_`).
/// - Does not start or end with a hyphen or underscore.
/// - Does not contain consecutive hyphens or underscores (`--`, `__`, `-_`, `_-`).
///
/// # Examples
///
/// ```
/// use newsjournal_core::validation::validate_slug;
///
/// assert!(validate_slug("city-council-investigation-2026").is_ok());
/// assert!(validate_slug("breaking_news_update").is_ok());
/// assert!(validate_slug("story123").is_ok());
///
/// assert!(validate_slug("City-Council").is_err()); // Contains uppercase
/// assert!(validate_slug("-leading-dash").is_err()); // Leading dash
/// assert!(validate_slug("double--dash").is_err()); // Consecutive dashes
/// assert!(validate_slug("").is_err()); // Empty
/// ```
pub fn validate_slug(slug: &str) -> Result<(), ValidationError> {
    if slug.is_empty() {
        return Err(ValidationError::InvalidSlug {
            slug: slug.to_string(),
            reason: "slug cannot be empty".to_string(),
        });
    }

    if slug.len() > MAX_SLUG_LENGTH {
        return Err(ValidationError::FieldTooLong {
            field: "slug",
            max: MAX_SLUG_LENGTH,
            actual: slug.len(),
        });
    }

    let first_char = slug.chars().next().unwrap();
    if first_char == '-' || first_char == '_' {
        return Err(ValidationError::InvalidSlug {
            slug: slug.to_string(),
            reason: "slug cannot start with a hyphen or underscore".to_string(),
        });
    }

    let last_char = slug.chars().last().unwrap();
    if last_char == '-' || last_char == '_' {
        return Err(ValidationError::InvalidSlug {
            slug: slug.to_string(),
            reason: "slug cannot end with a hyphen or underscore".to_string(),
        });
    }

    let mut prev_is_separator = false;
    for ch in slug.chars() {
        if ch.is_ascii_uppercase() {
            return Err(ValidationError::InvalidSlug {
                slug: slug.to_string(),
                reason: "slug must be lowercase (contains uppercase characters)".to_string(),
            });
        }

        let is_separator = ch == '-' || ch == '_';
        if is_separator && prev_is_separator {
            return Err(ValidationError::InvalidSlug {
                slug: slug.to_string(),
                reason: "slug cannot contain consecutive hyphens or underscores".to_string(),
            });
        }
        prev_is_separator = is_separator;

        if !ch.is_ascii_lowercase() && !ch.is_ascii_digit() && !is_separator {
            return Err(ValidationError::InvalidSlug {
                slug: slug.to_string(),
                reason: format!(
                    "slug contains invalid character '{ch}' (only lowercase alphanumeric, hyphens, and underscores allowed)"
                ),
            });
        }
    }

    Ok(())
}

/// Checks whether a string is a valid slug according to [`validate_slug`].
///
/// # Examples
///
/// ```
/// use newsjournal_core::validation::is_valid_slug;
///
/// assert!(is_valid_slug("election-2026-analysis"));
/// assert!(!is_valid_slug("election 2026 analysis"));
/// ```
#[must_use]
pub fn is_valid_slug(slug: &str) -> bool {
    validate_slug(slug).is_ok()
}

/// Converts a headline or free-form title into a URL and filesystem safe slug.
///
/// This helper:
/// 1. Converts input to lowercase ASCII equivalents where possible.
/// 2. Replaces non-alphanumeric characters with hyphens.
/// 3. Collapses consecutive hyphens into a single hyphen.
/// 4. Trims leading and trailing hyphens.
/// 5. Truncates to [`MAX_SLUG_LENGTH`] characters.
/// 6. Falls back to `"story"` if the resulting slug would be empty.
///
/// # Examples
///
/// ```
/// use newsjournal_core::validation::slugify;
///
/// assert_eq!(
///     slugify("Breaking: City Council Approves 2026 Transit Expansion!"),
///     "breaking-city-council-approves-2026-transit-expansion"
/// );
/// assert_eq!(slugify("   --- Hello, World! ---   "), "hello-world");
/// assert_eq!(slugify("!!!"), "story");
/// ```
#[must_use]
pub fn slugify(input: &str) -> String {
    let mut slug = String::with_capacity(input.len());
    let mut prev_hyphen = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            prev_hyphen = false;
        } else if !prev_hyphen && !slug.is_empty() {
            slug.push('-');
            prev_hyphen = true;
        }
    }

    // Strip trailing hyphen
    while slug.ends_with('-') || slug.ends_with('_') {
        slug.pop();
    }

    // Truncate to maximum length
    if slug.len() > MAX_SLUG_LENGTH {
        slug.truncate(MAX_SLUG_LENGTH);
        while slug.ends_with('-') || slug.ends_with('_') {
            slug.pop();
        }
    }

    if slug.is_empty() {
        "story".to_string()
    } else {
        slug
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_slugs() {
        let valid = [
            "article",
            "city-hall-investigation",
            "breaking_news",
            "budget_2026-report",
            "123-numbers-456",
            "a-b-c-d",
            "a_b_c_d",
            "x",
        ];

        for s in valid {
            assert!(validate_slug(s).is_ok(), "expected '{s}' to be valid");
            assert!(is_valid_slug(s));
        }
    }

    #[test]
    fn test_invalid_slugs() {
        let invalid = [
            ("", "empty"),
            ("   ", "whitespace"),
            ("-leading-dash", "leading dash"),
            ("_leading_underscore", "leading underscore"),
            ("trailing-dash-", "trailing dash"),
            ("trailing-underscore_", "trailing underscore"),
            ("Double--Dash", "uppercase and double dash"),
            ("double--dash", "consecutive dashes"),
            ("double__underscore", "consecutive underscores"),
            ("mixed-_dash_underscore", "consecutive mixed"),
            ("has spaces in slug", "spaces"),
            ("has/slash", "slash"),
            ("has.dot", "dot"),
            ("emoji-📰", "emoji"),
            ("uppercase-SLUG", "uppercase"),
        ];

        for (s, reason) in invalid {
            assert!(
                validate_slug(s).is_err(),
                "expected '{s}' ({reason}) to be invalid"
            );
            assert!(!is_valid_slug(s));
        }
    }

    #[test]
    fn test_slug_max_length() {
        let max_str = "a".repeat(MAX_SLUG_LENGTH);
        assert!(validate_slug(&max_str).is_ok());

        let too_long = "a".repeat(MAX_SLUG_LENGTH + 1);
        let err = validate_slug(&too_long).unwrap_err();
        assert!(matches!(
            err,
            ValidationError::FieldTooLong {
                field: "slug",
                max: 128,
                actual: 129
            }
        ));
    }

    #[test]
    fn test_slugify_conversions() {
        assert_eq!(
            slugify("City Council Votes 5-2 on Budget"),
            "city-council-votes-5-2-on-budget"
        );
        assert_eq!(
            slugify("  Investigative Report: Police Misconduct (Part 1)  "),
            "investigative-report-police-misconduct-part-1"
        );
        assert_eq!(
            slugify("Special @#$$% Characters & More!"),
            "special-characters-more"
        );
        assert_eq!(
            slugify("---leading-and-trailing---"),
            "leading-and-trailing"
        );
        assert_eq!(slugify(""), "story");
        assert_eq!(slugify("   ???   "), "story");

        let long_input = "a ".repeat(200);
        let slugified = slugify(&long_input);
        assert!(slugified.len() <= MAX_SLUG_LENGTH);
        assert!(validate_slug(&slugified).is_ok());
    }
}
