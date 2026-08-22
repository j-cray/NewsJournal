//! Email address validation and normalization utilities.

use super::error::ValidationError;

/// Maximum total length for an email address according to RFC 5321.
pub const MAX_EMAIL_LENGTH: usize = 254;

/// Maximum length for the local part of an email address.
pub const MAX_LOCAL_PART_LENGTH: usize = 64;

/// Validates that an email address matches standard internet format.
///
/// Rules applied:
/// - Total length must not exceed 254 characters and must not be empty.
/// - Must contain exactly one `@` symbol dividing the local part and domain.
/// - Local part: 1 to 64 chars, allowed chars are alphanumeric and `.` `_` `%` `+` `-`.
///   Cannot start or end with a dot, and cannot contain consecutive dots (`..`).
/// - Domain: 1 to 253 chars, contains at least one dot separating subdomains/TLD.
///   Each domain label must be 1 to 63 chars, alphanumeric or hyphen (not starting/ending with hyphen).
///   The top-level domain (TLD) must be at least 2 alphabetic characters.
///
/// # Examples
///
/// ```
/// use newsjournal_core::validation::validate_email;
///
/// assert!(validate_email("reporter@newsroom.example.com").is_ok());
/// assert!(validate_email("jane.doe+tips@investigative.org").is_ok());
///
/// assert!(validate_email("plainaddress").is_err());
/// assert!(validate_email("@missing-user.com").is_err());
/// assert!(validate_email("user@.com").is_err());
/// ```
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    let trimmed = email.trim();

    if trimmed.is_empty() {
        return Err(ValidationError::InvalidEmail {
            email: email.to_string(),
            reason: "email address cannot be empty".to_string(),
        });
    }

    if trimmed.len() > MAX_EMAIL_LENGTH {
        return Err(ValidationError::FieldTooLong {
            field: "email",
            max: MAX_EMAIL_LENGTH,
            actual: trimmed.len(),
        });
    }

    let parts: Vec<&str> = trimmed.split('@').collect();
    if parts.len() != 2 {
        return Err(ValidationError::InvalidEmail {
            email: email.to_string(),
            reason: "email address must contain exactly one '@' symbol".to_string(),
        });
    }

    let (local, domain) = (parts[0], parts[1]);

    // Validate local part
    if local.is_empty() {
        return Err(ValidationError::InvalidEmail {
            email: email.to_string(),
            reason: "local part (before '@') cannot be empty".to_string(),
        });
    }

    if local.len() > MAX_LOCAL_PART_LENGTH {
        return Err(ValidationError::InvalidEmail {
            email: email.to_string(),
            reason: format!(
                "local part exceeds maximum length of {MAX_LOCAL_PART_LENGTH} characters"
            ),
        });
    }

    if local.starts_with('.') || local.ends_with('.') {
        return Err(ValidationError::InvalidEmail {
            email: email.to_string(),
            reason: "local part cannot start or end with a dot '.'".to_string(),
        });
    }

    if local.contains("..") {
        return Err(ValidationError::InvalidEmail {
            email: email.to_string(),
            reason: "local part cannot contain consecutive dots '..'".to_string(),
        });
    }

    for ch in local.chars() {
        let is_valid_local_char =
            ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '%' | '+' | '-');
        if !is_valid_local_char {
            return Err(ValidationError::InvalidEmail {
                email: email.to_string(),
                reason: format!("local part contains invalid character '{ch}'"),
            });
        }
    }

    // Validate domain part
    if domain.is_empty() {
        return Err(ValidationError::InvalidEmail {
            email: email.to_string(),
            reason: "domain part (after '@') cannot be empty".to_string(),
        });
    }

    if domain.starts_with('.') || domain.ends_with('.') {
        return Err(ValidationError::InvalidEmail {
            email: email.to_string(),
            reason: "domain cannot start or end with a dot '.'".to_string(),
        });
    }

    if domain.contains("..") {
        return Err(ValidationError::InvalidEmail {
            email: email.to_string(),
            reason: "domain cannot contain consecutive dots '..'".to_string(),
        });
    }

    let labels: Vec<&str> = domain.split('.').collect();
    if labels.len() < 2 {
        return Err(ValidationError::InvalidEmail {
            email: email.to_string(),
            reason: "domain must contain at least one dot separating domain and TLD".to_string(),
        });
    }

    for (idx, label) in labels.iter().enumerate() {
        if label.is_empty() {
            return Err(ValidationError::InvalidEmail {
                email: email.to_string(),
                reason: "domain contains an empty label".to_string(),
            });
        }

        if label.len() > 63 {
            return Err(ValidationError::InvalidEmail {
                email: email.to_string(),
                reason: format!("domain label '{label}' exceeds maximum length of 63 characters"),
            });
        }

        if label.starts_with('-') || label.ends_with('-') {
            return Err(ValidationError::InvalidEmail {
                email: email.to_string(),
                reason: format!("domain label '{label}' cannot start or end with a hyphen '-'"),
            });
        }

        for ch in label.chars() {
            if !ch.is_ascii_alphanumeric() && ch != '-' {
                return Err(ValidationError::InvalidEmail {
                    email: email.to_string(),
                    reason: format!("domain label '{label}' contains invalid character '{ch}'"),
                });
            }
        }

        // TLD validation (last label)
        if idx == labels.len() - 1 {
            if label.len() < 2 {
                return Err(ValidationError::InvalidEmail {
                    email: email.to_string(),
                    reason: "top-level domain (TLD) must be at least 2 characters".to_string(),
                });
            }

            if !label.chars().all(|c| c.is_ascii_alphabetic()) {
                return Err(ValidationError::InvalidEmail {
                    email: email.to_string(),
                    reason: "top-level domain (TLD) must contain only alphabetic characters"
                        .to_string(),
                });
            }
        }
    }

    Ok(())
}

/// Checks whether an email address is valid according to [`validate_email`].
///
/// # Examples
///
/// ```
/// use newsjournal_core::validation::is_valid_email;
///
/// assert!(is_valid_email("source@example.com"));
/// assert!(!is_valid_email("invalid-email"));
/// ```
#[must_use]
pub fn is_valid_email(email: &str) -> bool {
    validate_email(email).is_ok()
}

/// Validates and normalizes an email address by trimming whitespace and converting to lowercase.
///
/// # Examples
///
/// ```
/// use newsjournal_core::validation::normalize_email;
///
/// assert_eq!(
///     normalize_email("  Reporter@NewsPaper.COM  ").unwrap(),
///     "reporter@newspaper.com"
/// );
/// ```
pub fn normalize_email(email: &str) -> Result<String, ValidationError> {
    let trimmed = email.trim();
    validate_email(trimmed)?;
    Ok(trimmed.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_emails() {
        let valid = [
            "reporter@example.com",
            "first.last@news.org",
            "jane_doe123@sub.domain.co.uk",
            "user+tag%promo-beat@mail.service.io",
            "editor@times.agency",
            "source1@gov.state.us",
        ];

        for email in valid {
            assert!(
                validate_email(email).is_ok(),
                "expected '{email}' to be valid"
            );
            assert!(is_valid_email(email));
        }
    }

    #[test]
    fn test_invalid_emails() {
        let invalid = [
            ("", "empty"),
            ("   ", "whitespace only"),
            ("missing-at.com", "missing @"),
            ("user@missing-tld", "missing tld"),
            ("@domain.com", "missing local"),
            ("user@", "missing domain"),
            ("user@@domain.com", "double @"),
            (".user@domain.com", "leading dot local"),
            ("user.@domain.com", "trailing dot local"),
            ("us..er@domain.com", "double dot local"),
            ("user@domain..com", "double dot domain"),
            ("user@-domain.com", "leading hyphen label"),
            ("user@domain-.com", "trailing hyphen label"),
            ("user@domain.c", "tld too short"),
            ("user@domain.123", "numeric tld"),
            ("user name@domain.com", "space in local"),
            ("user@dom ain.com", "space in domain"),
        ];

        for (email, reason) in invalid {
            assert!(
                validate_email(email).is_err(),
                "expected '{email}' ({reason}) to be invalid"
            );
            assert!(!is_valid_email(email));
        }
    }

    #[test]
    fn test_email_length_boundaries() {
        let long_local = format!("{}@example.com", "a".repeat(65));
        assert!(validate_email(&long_local).is_err());

        let max_local = format!("{}@example.com", "a".repeat(64));
        assert!(validate_email(&max_local).is_ok());

        let huge_email = format!("user@{}.com", "a".repeat(260));
        assert!(validate_email(&huge_email).is_err());
    }

    #[test]
    fn test_normalize_email() {
        assert_eq!(
            normalize_email("  ALICE@JOURNAL.ORG  ").unwrap(),
            "alice@journal.org"
        );
        assert_eq!(
            normalize_email("News.Tips+2026@Agency.Net").unwrap(),
            "news.tips+2026@agency.net"
        );
        assert!(normalize_email("invalid").is_err());
    }
}
