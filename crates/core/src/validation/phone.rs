//! Phone number validation, normalization, and formatting utilities.

use super::error::ValidationError;

/// Maximum raw character length for a phone number input string.
pub const MAX_PHONE_RAW_LENGTH: usize = 32;

/// Minimum number of digits required in a valid phone number.
pub const MIN_PHONE_DIGITS: usize = 7;

/// Maximum number of digits permitted under the ITU-T E.164 standard.
pub const MAX_PHONE_DIGITS: usize = 15;

/// Validates that a string represents a valid phone number.
///
/// Rules:
/// - Must not be empty after trimming and must not exceed [`MAX_PHONE_RAW_LENGTH`] (32 chars).
/// - Allowed characters: digits `0-9`, an optional leading `+`, and formatting characters (space, `-`, `.`, `(`, `)`).
/// - Total digit count must be between [`MIN_PHONE_DIGITS`] (7) and [`MAX_PHONE_DIGITS`] (15).
/// - Formatting parentheses must be balanced and properly ordered.
///
/// # Examples
///
/// ```
/// use newsjournal_core::validation::validate_phone;
///
/// assert!(validate_phone("+1 (555) 123-4567").is_ok());
/// assert!(validate_phone("555-0199").is_ok());
/// assert!(validate_phone("+44 20 7946 0958").is_ok());
///
/// assert!(validate_phone("123").is_err()); // Too few digits
/// assert!(validate_phone("call-me-now").is_err()); // Alphabetic characters
/// assert!(validate_phone("+1+555-1234").is_err()); // Multiple plus signs
/// ```
pub fn validate_phone(phone: &str) -> Result<(), ValidationError> {
    let trimmed = phone.trim();

    if trimmed.is_empty() {
        return Err(ValidationError::InvalidPhone {
            phone: phone.to_string(),
            reason: "phone number cannot be empty".to_string(),
        });
    }

    if trimmed.len() > MAX_PHONE_RAW_LENGTH {
        return Err(ValidationError::FieldTooLong {
            field: "phone",
            max: MAX_PHONE_RAW_LENGTH,
            actual: trimmed.len(),
        });
    }

    let mut digit_count = 0;
    let mut open_paren = false;
    let mut paren_closed = false;

    for (idx, ch) in trimmed.chars().enumerate() {
        if ch == '+' {
            if idx != 0 {
                return Err(ValidationError::InvalidPhone {
                    phone: phone.to_string(),
                    reason: "'+' prefix is only allowed at the beginning of the phone number"
                        .to_string(),
                });
            }
            continue;
        }

        if ch.is_ascii_digit() {
            digit_count += 1;
            continue;
        }

        match ch {
            '(' => {
                if open_paren || paren_closed {
                    return Err(ValidationError::InvalidPhone {
                        phone: phone.to_string(),
                        reason: "unexpected or duplicate '(' character in phone number".to_string(),
                    });
                }
                open_paren = true;
            }
            ')' => {
                if !open_paren || paren_closed {
                    return Err(ValidationError::InvalidPhone {
                        phone: phone.to_string(),
                        reason: "unmatched ')' character in phone number".to_string(),
                    });
                }
                open_paren = false;
                paren_closed = true;
            }
            ' ' | '-' | '.' => {
                // Allowed separator characters
            }
            _ => {
                return Err(ValidationError::InvalidPhone {
                    phone: phone.to_string(),
                    reason: format!(
                        "phone number contains invalid character '{ch}' (only digits, '+', spaces, '-', '.', '(', and ')' are allowed)"
                    ),
                });
            }
        }
    }

    if open_paren {
        return Err(ValidationError::InvalidPhone {
            phone: phone.to_string(),
            reason: "unclosed '(' in phone number".to_string(),
        });
    }

    if digit_count < MIN_PHONE_DIGITS {
        return Err(ValidationError::InvalidPhone {
            phone: phone.to_string(),
            reason: format!(
                "phone number must contain at least {MIN_PHONE_DIGITS} digits (found {digit_count})"
            ),
        });
    }

    if digit_count > MAX_PHONE_DIGITS {
        return Err(ValidationError::InvalidPhone {
            phone: phone.to_string(),
            reason: format!(
                "phone number exceeds maximum of {MAX_PHONE_DIGITS} digits under E.164 (found {digit_count})"
            ),
        });
    }

    Ok(())
}

/// Checks whether a phone number is valid according to [`validate_phone`].
///
/// # Examples
///
/// ```
/// use newsjournal_core::validation::is_valid_phone;
///
/// assert!(is_valid_phone("+1 555-0199"));
/// assert!(!is_valid_phone("invalid"));
/// ```
#[must_use]
pub fn is_valid_phone(phone: &str) -> bool {
    validate_phone(phone).is_ok()
}

/// Normalizes a phone number by stripping formatting characters into an E.164-compatible canonical digit string.
///
/// # Examples
///
/// ```
/// use newsjournal_core::validation::normalize_phone;
///
/// assert_eq!(
///     normalize_phone("+1 (555) 123-4567").unwrap(),
///     "+15551234567"
/// );
/// assert_eq!(
///     normalize_phone("555.123.4567").unwrap(),
///     "5551234567"
/// );
/// ```
pub fn normalize_phone(phone: &str) -> Result<String, ValidationError> {
    validate_phone(phone)?;
    let trimmed = phone.trim();
    let has_plus = trimmed.starts_with('+');

    let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();

    if has_plus {
        Ok(format!("+{digits}"))
    } else {
        Ok(digits)
    }
}

/// Formats a phone number for standardized user-facing display when possible.
///
/// If the number has 10 digits, formats as `(XXX) XXX-XXXX`.
/// If the number has 11 digits starting with 1, formats as `+1 (XXX) XXX-XXXX`.
/// Otherwise returns the cleaned or original string.
///
/// # Examples
///
/// ```
/// use newsjournal_core::validation::format_phone_display;
///
/// assert_eq!(format_phone_display("5551234567"), "(555) 123-4567");
/// assert_eq!(format_phone_display("+15551234567"), "+1 (555) 123-4567");
/// ```
#[must_use]
pub fn format_phone_display(phone: &str) -> String {
    let trimmed = phone.trim();
    let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() == 10 {
        format!("({}) {}-{}", &digits[0..3], &digits[3..6], &digits[6..10])
    } else if digits.len() == 11 && digits.starts_with('1') {
        format!(
            "+1 ({}) {}-{}",
            &digits[1..4],
            &digits[4..7],
            &digits[7..11]
        )
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_phone_numbers() {
        let valid = [
            "+1 (555) 123-4567",
            "555-123-4567",
            "+15551234567",
            "+44 20 7946 0958",
            "555.123.4567",
            "(555) 0199000",
            "+33 1 42 68 55 00",
            "1234567",
            "123456789012345",
        ];

        for phone in valid {
            assert!(
                validate_phone(phone).is_ok(),
                "expected '{phone}' to be valid"
            );
            assert!(is_valid_phone(phone));
        }
    }

    #[test]
    fn test_invalid_phone_numbers() {
        let invalid = [
            ("", "empty"),
            ("   ", "whitespace only"),
            ("123456", "too few digits (6)"),
            ("1234567890123456", "too many digits (16)"),
            ("+1 (555) 123-456a", "contains letters"),
            ("555-1234-@", "special character @"),
            ("1+555-123-4567", "plus in middle"),
            ("(555-123-4567", "unclosed paren"),
            ("555)-123-4567", "unmatched closing paren"),
            ("((555)) 123-4567", "duplicate open paren"),
        ];

        for (phone, reason) in invalid {
            assert!(
                validate_phone(phone).is_err(),
                "expected '{phone}' ({reason}) to be invalid"
            );
            assert!(!is_valid_phone(phone));
        }
    }

    #[test]
    fn test_normalize_phone() {
        assert_eq!(
            normalize_phone("+1 (555) 123-4567").unwrap(),
            "+15551234567"
        );
        assert_eq!(normalize_phone("  555.019.9999  ").unwrap(), "5550199999");
        assert_eq!(
            normalize_phone("+44 20 7946 0958").unwrap(),
            "+442079460958"
        );
        assert!(normalize_phone("short").is_err());
    }

    #[test]
    fn test_format_phone_display() {
        assert_eq!(format_phone_display("5551234567"), "(555) 123-4567");
        assert_eq!(format_phone_display("15551234567"), "+1 (555) 123-4567");
        assert_eq!(format_phone_display("+15551234567"), "+1 (555) 123-4567");
        assert_eq!(format_phone_display("+44 20 7946 0958"), "+44 20 7946 0958");
    }
}
