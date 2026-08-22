//! Error types for color parsing and palette operations.

use thiserror::Error;

/// Errors that can occur during color parsing, conversion, and palette operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ColorError {
    /// The hex string was empty, lacked a '#' prefix, or had an invalid structure.
    #[error("invalid hex color string '{color}': {reason}")]
    InvalidHexFormat {
        /// The invalid hex color input.
        color: String,
        /// Description of why the format is invalid.
        reason: String,
    },

    /// The hex string contained a non-hexadecimal character.
    #[error("hex color '{color}' contains invalid character '{character}'")]
    InvalidHexDigit {
        /// The invalid hex color input.
        color: String,
        /// The offending non-hex character.
        character: char,
    },

    /// The hex string had an invalid digit count.
    #[error("hex color '{color}' has invalid length {length}; expected 3, 4, 6, or 8 hex digits")]
    InvalidLength {
        /// The invalid hex color input.
        color: String,
        /// The number of hex digits encountered.
        length: usize,
    },

    /// An operation was attempted on an empty color palette.
    #[error("color palette cannot be empty")]
    EmptyPalette,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_error_display() {
        let err = ColorError::InvalidHexFormat {
            color: "abc".to_string(),
            reason: "missing '#' prefix".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "invalid hex color string 'abc': missing '#' prefix"
        );

        let err_digit = ColorError::InvalidHexDigit {
            color: "#GGGGGG".to_string(),
            character: 'G',
        };
        assert_eq!(
            err_digit.to_string(),
            "hex color '#GGGGGG' contains invalid character 'G'"
        );

        let err_len = ColorError::InvalidLength {
            color: "#12".to_string(),
            length: 2,
        };
        assert_eq!(
            err_len.to_string(),
            "hex color '#12' has invalid length 2; expected 3, 4, 6, or 8 hex digits"
        );

        let err_empty = ColorError::EmptyPalette;
        assert_eq!(err_empty.to_string(), "color palette cannot be empty");
    }
}
