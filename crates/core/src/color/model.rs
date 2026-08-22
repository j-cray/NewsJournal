//! Core Color data model, color space conversions, and WCAG contrast calculations.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::error::ColorError;

/// Represents an sRGB color with 8-bit channels and an alpha channel.
///
/// Provides conversions to and from hexadecimal strings, normalized float tuples,
/// and WCAG 2.1 contrast and luminance calculations.
///
/// # Examples
///
/// ```
/// use newsjournal_core::color::Color;
///
/// let cobalt = Color::from_hex("#2563EB").unwrap();
/// assert_eq!(cobalt.to_rgb(), (37, 99, 235));
/// assert_eq!(cobalt.to_hex(), "#2563EB");
///
/// let white = Color::rgb(255, 255, 255);
/// assert!(cobalt.contrast_ratio(&white) > 4.5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    /// Red color channel (0-255).
    pub r: u8,
    /// Green color channel (0-255).
    pub g: u8,
    /// Blue color channel (0-255).
    pub b: u8,
    /// Alpha transparency channel (0-255, where 255 is fully opaque).
    pub a: u8,
}

impl Default for Color {
    /// Returns default brand cobalt blue (`#2563EB`).
    fn default() -> Self {
        Self::rgb(37, 99, 235)
    }
}

impl Color {
    /// Creates a new fully opaque color from 8-bit red, green, and blue components.
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Creates a new color with an explicit 8-bit alpha component.
    #[must_use]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Creates an opaque color from a 24-bit integer literal (e.g. `0x2563EB`).
    #[must_use]
    pub const fn from_rgb_u32(val: u32) -> Self {
        let r = ((val >> 16) & 0xFF) as u8;
        let g = ((val >> 8) & 0xFF) as u8;
        let b = (val & 0xFF) as u8;
        Self::rgb(r, g, b)
    }

    /// Creates a color from a 32-bit RGBA integer literal (e.g. `0x2563EBFF`).
    #[must_use]
    pub const fn from_rgba_u32(val: u32) -> Self {
        let r = ((val >> 24) & 0xFF) as u8;
        let g = ((val >> 16) & 0xFF) as u8;
        let b = ((val >> 8) & 0xFF) as u8;
        let a = (val & 0xFF) as u8;
        Self::rgba(r, g, b, a)
    }

    /// Parses a color from a hexadecimal string.
    ///
    /// Supports `#RGB`, `#RGBA`, `#RRGGBB`, and `#RRGGBBAA` formats.
    ///
    /// # Errors
    ///
    /// Returns [`ColorError`] if the string does not start with `#`, has invalid length,
    /// or contains non-hexadecimal characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use newsjournal_core::color::Color;
    ///
    /// assert_eq!(Color::from_hex("#FFF").unwrap(), Color::rgb(255, 255, 255));
    /// assert_eq!(Color::from_hex("#059669").unwrap(), Color::rgb(5, 150, 105));
    /// ```
    pub fn from_hex(hex: &str) -> Result<Self, ColorError> {
        let trimmed = hex.trim();

        if !trimmed.starts_with('#') {
            return Err(ColorError::InvalidHexFormat {
                color: hex.to_string(),
                reason: "hex color code must start with '#'".to_string(),
            });
        }

        let hex_body = &trimmed[1..];
        let len = hex_body.len();

        for ch in hex_body.chars() {
            if !ch.is_ascii_hexdigit() {
                return Err(ColorError::InvalidHexDigit {
                    color: hex.to_string(),
                    character: ch,
                });
            }
        }

        match len {
            3 => {
                let r = u8::from_str_radix(&hex_body[0..1], 16).unwrap();
                let g = u8::from_str_radix(&hex_body[1..2], 16).unwrap();
                let b = u8::from_str_radix(&hex_body[2..3], 16).unwrap();
                Ok(Self::rgb(r * 17, g * 17, b * 17))
            }
            4 => {
                let r = u8::from_str_radix(&hex_body[0..1], 16).unwrap();
                let g = u8::from_str_radix(&hex_body[1..2], 16).unwrap();
                let b = u8::from_str_radix(&hex_body[2..3], 16).unwrap();
                let a = u8::from_str_radix(&hex_body[3..4], 16).unwrap();
                Ok(Self::rgba(r * 17, g * 17, b * 17, a * 17))
            }
            6 => {
                let r = u8::from_str_radix(&hex_body[0..2], 16).unwrap();
                let g = u8::from_str_radix(&hex_body[2..4], 16).unwrap();
                let b = u8::from_str_radix(&hex_body[4..6], 16).unwrap();
                Ok(Self::rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex_body[0..2], 16).unwrap();
                let g = u8::from_str_radix(&hex_body[2..4], 16).unwrap();
                let b = u8::from_str_radix(&hex_body[4..6], 16).unwrap();
                let a = u8::from_str_radix(&hex_body[6..8], 16).unwrap();
                Ok(Self::rgba(r, g, b, a))
            }
            _ => Err(ColorError::InvalidLength {
                color: hex.to_string(),
                length: len,
            }),
        }
    }

    /// Formats the color as an uppercase 6-character hex string (e.g. `#2563EB`).
    #[must_use]
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Formats the color as an uppercase 8-character hex string including alpha (e.g. `#2563EBFF`).
    #[must_use]
    pub fn to_hex_rgba(&self) -> String {
        format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
    }

    /// Returns the red, green, and blue components as an 8-bit tuple.
    #[must_use]
    pub const fn to_rgb(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    /// Returns the red, green, blue, and alpha components as an 8-bit tuple.
    #[must_use]
    pub const fn to_rgba(&self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }

    /// Returns the RGB components normalized to the floating-point range `0.0..=1.0`.
    #[must_use]
    pub fn to_f32_rgb(&self) -> (f32, f32, f32) {
        (
            f32::from(self.r) / 255.0,
            f32::from(self.g) / 255.0,
            f32::from(self.b) / 255.0,
        )
    }

    /// Returns the RGBA components normalized to the floating-point range `0.0..=1.0`.
    #[must_use]
    pub fn to_f32_rgba(&self) -> (f32, f32, f32, f32) {
        (
            f32::from(self.r) / 255.0,
            f32::from(self.g) / 255.0,
            f32::from(self.b) / 255.0,
            f32::from(self.a) / 255.0,
        )
    }

    /// Calculates the WCAG 2.1 relative luminance `L` of the color in the range `0.0..=1.0`.
    ///
    /// Formula: `L = 0.2126 * R + 0.7152 * G + 0.0722 * B` (with gamma correction).
    #[must_use]
    pub fn relative_luminance(&self) -> f32 {
        let (r, g, b) = self.to_f32_rgb();

        let r_lin = if r <= 0.04045 {
            r / 12.92
        } else {
            ((r + 0.055) / 1.055).powf(2.4)
        };

        let g_lin = if g <= 0.04045 {
            g / 12.92
        } else {
            ((g + 0.055) / 1.055).powf(2.4)
        };

        let b_lin = if b <= 0.04045 {
            b / 12.92
        } else {
            ((b + 0.055) / 1.055).powf(2.4)
        };

        0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin
    }

    /// Calculates the WCAG 2.1 contrast ratio between this color and another color.
    ///
    /// Returns a value between `1.0` (identical luminance) and `21.0` (black vs white).
    ///
    /// # Examples
    ///
    /// ```
    /// use newsjournal_core::color::Color;
    ///
    /// let black = Color::rgb(0, 0, 0);
    /// let white = Color::rgb(255, 255, 255);
    /// assert!((black.contrast_ratio(&white) - 21.0).abs() < 0.1);
    /// ```
    #[must_use]
    pub fn contrast_ratio(&self, other: &Self) -> f32 {
        let l1 = self.relative_luminance();
        let l2 = other.relative_luminance();

        let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (lighter + 0.05) / (darker + 0.05)
    }

    /// Determines whether the contrast ratio against a background meets a minimum threshold.
    ///
    /// Common WCAG thresholds:
    /// - Normal text AA: `4.5`
    /// - Large text / UI components AA: `3.0`
    /// - Normal text AAA: `7.0`
    #[must_use]
    pub fn is_accessible_against(&self, background: &Self, min_ratio: f32) -> bool {
        self.contrast_ratio(background) >= min_ratio
    }

    /// Chooses the optimal text color (pure white `#FFFFFF` or dark charcoal `#1E293B`)
    /// to maximize readability and contrast against this color as a background.
    #[must_use]
    pub fn best_text_color_for_background(&self) -> Self {
        let white = Self::rgb(255, 255, 255);
        let dark = Self::rgb(30, 41, 59);

        let contrast_white = self.contrast_ratio(&white);
        let contrast_dark = self.contrast_ratio(&dark);

        if contrast_white >= contrast_dark {
            white
        } else {
            dark
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.a == 255 {
            write!(f, "{}", self.to_hex())
        } else {
            write!(f, "{}", self.to_hex_rgba())
        }
    }
}

impl FromStr for Color {
    type Err = ColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_constructors_and_tuples() {
        let color = Color::rgb(10, 20, 30);
        assert_eq!(color.r, 10);
        assert_eq!(color.g, 20);
        assert_eq!(color.b, 30);
        assert_eq!(color.a, 255);
        assert_eq!(color.to_rgb(), (10, 20, 30));
        assert_eq!(color.to_rgba(), (10, 20, 30, 255));

        let color_rgba = Color::rgba(10, 20, 30, 128);
        assert_eq!(color_rgba.to_rgba(), (10, 20, 30, 128));
    }

    #[test]
    fn test_u32_constructors() {
        let c1 = Color::from_rgb_u32(0x2563EB);
        assert_eq!(c1, Color::rgb(37, 99, 235));

        let c2 = Color::from_rgba_u32(0x2563EB80);
        assert_eq!(c2, Color::rgba(37, 99, 235, 128));
    }

    #[test]
    fn test_from_hex_formats() {
        assert_eq!(Color::from_hex("#FFF").unwrap(), Color::rgb(255, 255, 255));
        assert_eq!(
            Color::from_hex("#F008").unwrap(),
            Color::rgba(255, 0, 0, 136)
        );
        assert_eq!(
            Color::from_hex("#4A90E2").unwrap(),
            Color::rgb(74, 144, 226)
        );
        assert_eq!(
            Color::from_hex("#4A90E2FF").unwrap(),
            Color::rgba(74, 144, 226, 255)
        );
    }

    #[test]
    fn test_from_hex_invalid() {
        assert!(matches!(
            Color::from_hex("4A90E2"),
            Err(ColorError::InvalidHexFormat { .. })
        ));
        assert!(matches!(
            Color::from_hex("#GGGGGG"),
            Err(ColorError::InvalidHexDigit { character: 'G', .. })
        ));
        assert!(matches!(
            Color::from_hex("#12345"),
            Err(ColorError::InvalidLength { length: 5, .. })
        ));
    }

    #[test]
    fn test_to_hex_and_display() {
        let c = Color::rgb(74, 144, 226);
        assert_eq!(c.to_hex(), "#4A90E2");
        assert_eq!(c.to_string(), "#4A90E2");

        let c_alpha = Color::rgba(74, 144, 226, 128);
        assert_eq!(c_alpha.to_hex_rgba(), "#4A90E280");
        assert_eq!(c_alpha.to_string(), "#4A90E280");
    }

    #[test]
    fn test_from_str() {
        let parsed: Color = "#2563EB".parse().unwrap();
        assert_eq!(parsed, Color::rgb(37, 99, 235));
    }

    #[test]
    fn test_f32_conversions() {
        let c = Color::rgba(255, 0, 128, 255);
        let (r, g, b) = c.to_f32_rgb();
        assert!((r - 1.0).abs() < 1e-5);
        assert!((g - 0.0).abs() < 1e-5);
        assert!((b - 0.50196).abs() < 1e-3);
    }

    #[test]
    fn test_wcag_luminance_and_contrast() {
        let white = Color::rgb(255, 255, 255);
        let black = Color::rgb(0, 0, 0);

        assert!((white.relative_luminance() - 1.0).abs() < 1e-5);
        assert!((black.relative_luminance() - 0.0).abs() < 1e-5);

        let contrast = black.contrast_ratio(&white);
        assert!((contrast - 21.0).abs() < 0.1);

        assert!(white.is_accessible_against(&black, 4.5));
    }

    #[test]
    fn test_best_text_color_for_background() {
        let dark_blue = Color::rgb(15, 23, 42);
        assert_eq!(
            dark_blue.best_text_color_for_background(),
            Color::rgb(255, 255, 255)
        );

        let light_bg = Color::rgb(240, 240, 240);
        assert_eq!(
            light_bg.best_text_color_for_background(),
            Color::rgb(30, 41, 59)
        );
    }

    #[test]
    fn test_serde_roundtrip() {
        let color = Color::rgb(37, 99, 235);
        let json = serde_json::to_string(&color).unwrap();
        assert_eq!(json, "\"#2563EB\"");

        let deserialized: Color = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, color);
    }
}
