//! Deterministic color palette generator, color models, and accessibility engines.
//!
//! Provides deterministic color assignment from a curated, high-contrast, accessible
//! palette (excluding low-contrast/light yellow shades), bidirectional Hex/RGB conversions,
//! and WCAG 2.1 contrast calculations.

pub mod error;
pub mod hash;
pub mod model;
pub mod palette;

pub use error::ColorError;
pub use hash::{fnv1a_hash, fnv1a_hash_str, fnv1a_hash_uuid};
pub use model::Color;
pub use palette::{
    assign_color, assign_color_for_slug, assign_color_for_uuid, NamedColor, Palette,
    CURATED_PALETTE,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_module_reexports() {
        let color = assign_color_for_slug("election-2026");
        assert_eq!(color.a, 255);
        assert!(color.to_hex().starts_with('#'));

        let parsed: Color = "#2563EB".parse().unwrap();
        assert_eq!(parsed.to_rgb(), (37, 99, 235));
    }
}
