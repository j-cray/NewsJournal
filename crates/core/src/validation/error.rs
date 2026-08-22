//! Validation error types for NewsJournal.

use thiserror::Error;

/// Errors that can occur during field, slug, email, phone, or entity validation.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum ValidationError {
    /// A required field was empty or contained only whitespace.
    #[error("field '{field}' cannot be empty")]
    EmptyField {
        /// Name of the empty field.
        field: &'static str,
    },

    /// A field exceeded its permitted character limit.
    #[error("field '{field}' exceeds maximum length of {max} characters (got {actual})")]
    FieldTooLong {
        /// Name of the field.
        field: &'static str,
        /// Maximum allowed length.
        max: usize,
        /// Actual character count.
        actual: usize,
    },

    /// An article or resource slug format was invalid.
    #[error("invalid slug '{slug}': {reason}")]
    InvalidSlug {
        /// The invalid slug string.
        slug: String,
        /// Description of why the slug is invalid.
        reason: String,
    },

    /// An email address format was invalid.
    #[error("invalid email address '{email}': {reason}")]
    InvalidEmail {
        /// The invalid email string.
        email: String,
        /// Description of why the email is invalid.
        reason: String,
    },

    /// A phone number format was invalid.
    #[error("invalid phone number '{phone}': {reason}")]
    InvalidPhone {
        /// The invalid phone string.
        phone: String,
        /// Description of why the phone number is invalid.
        reason: String,
    },

    /// A hex color code format was invalid.
    #[error("invalid hex color '{color}': {reason}")]
    InvalidColor {
        /// The invalid hex color string.
        color: String,
        /// Description of why the color is invalid.
        reason: String,
    },
}
