//! Curated accessible color palette and deterministic color assignment engine.

use uuid::Uuid;

use super::error::ColorError;
use super::hash::{fnv1a_hash, fnv1a_hash_str, fnv1a_hash_uuid};
use super::model::Color;

/// A named color entry in the curated palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NamedColor {
    /// Human-readable label for the color (e.g. "Cobalt Blue").
    pub name: &'static str,
    /// The RGB color value.
    pub color: Color,
}

impl NamedColor {
    /// Creates a new named color entry.
    #[must_use]
    pub const fn new(name: &'static str, color: Color) -> Self {
        Self { name, color }
    }
}

/// Standard curated palette containing 16 vibrant, distinct, high-contrast colors.
///
/// Strictly excludes low-contrast and unreadable light yellow shades to ensure
/// readability across light and dark theme backgrounds.
pub const CURATED_PALETTE: &[NamedColor] = &[
    NamedColor::new("Crimson", Color::rgb(229, 57, 53)),
    NamedColor::new("Coral", Color::rgb(244, 81, 30)),
    NamedColor::new("Amber Rust", Color::rgb(217, 119, 6)),
    NamedColor::new("Emerald", Color::rgb(5, 150, 105)),
    NamedColor::new("Teal", Color::rgb(13, 148, 136)),
    NamedColor::new("Cyan Ocean", Color::rgb(8, 145, 178)),
    NamedColor::new("Cobalt Blue", Color::rgb(37, 99, 235)),
    NamedColor::new("Indigo", Color::rgb(79, 70, 229)),
    NamedColor::new("Violet", Color::rgb(124, 58, 237)),
    NamedColor::new("Purple Plum", Color::rgb(147, 51, 234)),
    NamedColor::new("Fuchsia", Color::rgb(192, 38, 211)),
    NamedColor::new("Rose", Color::rgb(225, 29, 72)),
    NamedColor::new("Slate Blue", Color::rgb(71, 85, 105)),
    NamedColor::new("Burnt Orange", Color::rgb(234, 88, 12)),
    NamedColor::new("Pine Green", Color::rgb(4, 120, 87)),
    NamedColor::new("Cerulean", Color::rgb(2, 132, 199)),
];

/// A collection of colors for deterministic assignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    colors: Vec<Color>,
}

impl Default for Palette {
    /// Returns the standard curated 16-color palette.
    fn default() -> Self {
        Self::curated()
    }
}

impl Palette {
    /// Creates a palette from the curated set of accessible colors.
    #[must_use]
    pub fn curated() -> Self {
        let colors = CURATED_PALETTE.iter().map(|nc| nc.color).collect();
        Self { colors }
    }

    /// Creates a new palette from a custom list of colors.
    ///
    /// # Errors
    ///
    /// Returns [`ColorError::EmptyPalette`] if the provided slice is empty.
    pub fn new(colors: Vec<Color>) -> Result<Self, ColorError> {
        if colors.is_empty() {
            Err(ColorError::EmptyPalette)
        } else {
            Ok(Self { colors })
        }
    }

    /// Deterministically assigns a color from this palette for a given string key.
    ///
    /// The same string key will always produce the exact same color on every run.
    ///
    /// # Examples
    ///
    /// ```
    /// use newsjournal_core::color::Palette;
    ///
    /// let palette = Palette::curated();
    /// let c1 = palette.assign("story-slug-1");
    /// let c2 = palette.assign("story-slug-1");
    /// assert_eq!(c1, c2);
    /// ```
    #[must_use]
    pub fn assign(&self, key: &str) -> Color {
        let hash = fnv1a_hash_str(key);
        let index = (hash % (self.colors.len() as u64)) as usize;
        self.colors[index]
    }

    /// Deterministically assigns a color from this palette for raw bytes.
    #[must_use]
    pub fn assign_bytes(&self, bytes: &[u8]) -> Color {
        let hash = fnv1a_hash(bytes);
        let index = (hash % (self.colors.len() as u64)) as usize;
        self.colors[index]
    }

    /// Deterministically assigns a color from this palette for a UUID.
    ///
    /// # Examples
    ///
    /// ```
    /// use newsjournal_core::color::Palette;
    /// use uuid::Uuid;
    ///
    /// let palette = Palette::curated();
    /// let id = Uuid::new_v4();
    /// let c1 = palette.assign_uuid(&id);
    /// let c2 = palette.assign_uuid(&id);
    /// assert_eq!(c1, c2);
    /// ```
    #[must_use]
    pub fn assign_uuid(&self, uuid: &Uuid) -> Color {
        let hash = fnv1a_hash_uuid(uuid);
        let index = (hash % (self.colors.len() as u64)) as usize;
        self.colors[index]
    }

    /// Returns the color at the given 0-indexed position, or `None` if out of bounds.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<Color> {
        self.colors.get(index).copied()
    }

    /// Returns a slice of all colors in the palette.
    #[must_use]
    pub fn colors(&self) -> &[Color] {
        &self.colors
    }

    /// Returns the number of colors in the palette.
    #[must_use]
    pub fn len(&self) -> usize {
        self.colors.len()
    }

    /// Returns `true` if the palette contains no colors.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }
}

/// Deterministically assigns a curated color for a string key.
///
/// # Examples
///
/// ```
/// use newsjournal_core::color::assign_color;
///
/// let color = assign_color("breaking-news-investigation");
/// assert_eq!(color, assign_color("breaking-news-investigation"));
/// ```
#[must_use]
pub fn assign_color(key: &str) -> Color {
    Palette::curated().assign(key)
}

/// Deterministically assigns a curated color for an article slug.
///
/// # Examples
///
/// ```
/// use newsjournal_core::color::assign_color_for_slug;
///
/// let color = assign_color_for_slug("city-council-budget-2026");
/// assert_eq!(color, assign_color_for_slug("city-council-budget-2026"));
/// ```
#[must_use]
pub fn assign_color_for_slug(slug: &str) -> Color {
    assign_color(slug)
}

/// Deterministically assigns a curated color for a entity UUID.
///
/// # Examples
///
/// ```
/// use newsjournal_core::color::assign_color_for_uuid;
/// use uuid::Uuid;
///
/// let id = Uuid::new_v4();
/// let color = assign_color_for_uuid(&id);
/// assert_eq!(color, assign_color_for_uuid(&id));
/// ```
#[must_use]
pub fn assign_color_for_uuid(uuid: &Uuid) -> Color {
    Palette::curated().assign_uuid(uuid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curated_palette_length_and_uniqueness() {
        assert_eq!(CURATED_PALETTE.len(), 16);
        let mut hexes = std::collections::HashSet::new();
        let mut names = std::collections::HashSet::new();

        for entry in CURATED_PALETTE {
            assert!(
                hexes.insert(entry.color.to_hex()),
                "duplicate color hex: {}",
                entry.color.to_hex()
            );
            assert!(
                names.insert(entry.name),
                "duplicate color name: {}",
                entry.name
            );
        }
    }

    #[test]
    fn test_curated_palette_excludes_light_yellow_and_low_contrast() {
        for entry in CURATED_PALETTE {
            let (r, g, b) = entry.color.to_rgb();

            // Light yellow typically has high red (>= 220), high green (>= 220), and low blue (<= 120)
            let is_light_yellow = r >= 220 && g >= 200 && b <= 120;
            assert!(
                !is_light_yellow,
                "Palette color '{}' ({}) resembles unreadable light yellow",
                entry.name,
                entry.color.to_hex()
            );

            // Verify relative luminance is in a balanced, visible range (not near-white > 0.85, not near-black < 0.01)
            let lum = entry.color.relative_luminance();
            assert!(
                lum <= 0.70,
                "Palette color '{}' ({}) luminance {:.3} is too high (washed out)",
                entry.name,
                entry.color.to_hex(),
                lum
            );
            assert!(
                lum >= 0.02,
                "Palette color '{}' ({}) luminance {:.3} is too dark",
                entry.name,
                entry.color.to_hex(),
                lum
            );
        }
    }

    #[test]
    fn test_deterministic_assignment() {
        let palette = Palette::curated();
        let keys = [
            "article-slug-1",
            "article-slug-2",
            "council-election-2026",
            "police-reform-deep-dive",
            "housing-crisis-part-1",
        ];

        for key in keys {
            let c1 = palette.assign(key);
            let c2 = palette.assign(key);
            let c3 = assign_color_for_slug(key);
            assert_eq!(c1, c2);
            assert_eq!(c1, c3);
        }
    }

    #[test]
    fn test_custom_palette() {
        let custom_colors = vec![Color::rgb(255, 0, 0), Color::rgb(0, 255, 0)];
        let palette = Palette::new(custom_colors.clone()).unwrap();
        assert_eq!(palette.len(), 2);
        assert!(!palette.is_empty());
        assert_eq!(palette.get(0), Some(Color::rgb(255, 0, 0)));
        assert_eq!(palette.get(1), Some(Color::rgb(0, 255, 0)));
        assert_eq!(palette.get(2), None);

        let empty = Palette::new(vec![]);
        assert!(matches!(empty, Err(ColorError::EmptyPalette)));
    }
}
