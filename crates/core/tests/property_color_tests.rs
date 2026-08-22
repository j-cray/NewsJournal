//! Property-based test suite for `newsjournal-core` color palette and accessibility engine.

use newsjournal_core::color::{
    assign_color, assign_color_for_slug, assign_color_for_uuid, fnv1a_hash, fnv1a_hash_str,
    fnv1a_hash_uuid, Color, Palette, CURATED_PALETTE,
};
use proptest::prelude::*;
use uuid::Uuid;

proptest! {
    // -------------------------------------------------------------------------
    // 1. Hashing Determinism & Invariants
    // -------------------------------------------------------------------------

    #[test]
    fn test_prop_fnv1a_determinism_and_equivalence(bytes in prop::collection::vec(any::<u8>(), 0..=200)) {
        let h1 = fnv1a_hash(&bytes);
        let h2 = fnv1a_hash(&bytes);
        prop_assert_eq!(h1, h2);

        if let Ok(s) = std::str::from_utf8(&bytes) {
            let str_hash = fnv1a_hash_str(s);
            prop_assert_eq!(fnv1a_hash(s.trim().as_bytes()), str_hash);
        }
    }

    #[test]
    fn test_prop_fnv1a_uuid_determinism(b in prop::array::uniform16(any::<u8>())) {
        let u = Uuid::from_bytes(b);
        let h1 = fnv1a_hash_uuid(&u);
        let h2 = fnv1a_hash_uuid(&u);
        prop_assert_eq!(h1, h2);
        prop_assert_eq!(h1, fnv1a_hash(u.as_bytes()));
    }

    // -------------------------------------------------------------------------
    // 2. WCAG Relative Luminance Properties
    // -------------------------------------------------------------------------

    #[test]
    fn test_prop_relative_luminance_bounds(r in 0u8..=255, g in 0u8..=255, b in 0u8..=255) {
        let color = Color::rgb(r, g, b);
        let lum = color.relative_luminance();
        prop_assert!((0.0..=1.0001).contains(&lum));
    }

    #[test]
    fn test_prop_relative_luminance_monotonicity(
        r1 in 0u8..=255, g1 in 0u8..=255, b1 in 0u8..=255,
        dr in 0u8..=50, dg in 0u8..=50, db in 0u8..=50
    ) {
        let r2 = r1.saturating_add(dr);
        let g2 = g1.saturating_add(dg);
        let b2 = b1.saturating_add(db);

        let c1 = Color::rgb(r1, g1, b1);
        let c2 = Color::rgb(r2, g2, b2);

        let lum1 = c1.relative_luminance();
        let lum2 = c2.relative_luminance();

        prop_assert!(lum1 <= lum2 + 1e-6);
    }

    // -------------------------------------------------------------------------
    // 3. WCAG Contrast Ratio Properties
    // -------------------------------------------------------------------------

    #[test]
    fn test_prop_contrast_ratio_symmetry_and_bounds(
        r1 in 0u8..=255, g1 in 0u8..=255, b1 in 0u8..=255,
        r2 in 0u8..=255, g2 in 0u8..=255, b2 in 0u8..=255
    ) {
        let c1 = Color::rgb(r1, g1, b1);
        let c2 = Color::rgb(r2, g2, b2);

        let cr1 = c1.contrast_ratio(&c2);
        let cr2 = c2.contrast_ratio(&c1);

        // Symmetry
        prop_assert!((cr1 - cr2).abs() < 1e-5);

        // Bounds: 1.0 <= contrast ratio <= 21.0
        prop_assert!(cr1 >= 1.0 - 1e-5);
        prop_assert!(cr1 <= 21.0 + 1e-5);

        // Self contrast is exactly 1.0
        prop_assert!((c1.contrast_ratio(&c1) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_prop_best_text_color_maximizes_contrast(r in 0u8..=255, g in 0u8..=255, b in 0u8..=255) {
        let background = Color::rgb(r, g, b);
        let best_text = background.best_text_color_for_background();

        let white = Color::rgb(255, 255, 255);
        let dark_slate = Color::rgb(30, 41, 59);

        // Best text color must be either white or dark slate
        prop_assert!(best_text == white || best_text == dark_slate);

        let best_contrast = background.contrast_ratio(&best_text);
        let white_contrast = background.contrast_ratio(&white);
        let dark_contrast = background.contrast_ratio(&dark_slate);

        // The chosen text color must have contrast >= the alternative text color (within float margin)
        let max_possible = white_contrast.max(dark_contrast);
        prop_assert!(best_contrast >= max_possible - 1e-5);
    }

    // -------------------------------------------------------------------------
    // 4. Palette Assignment Invariants
    // -------------------------------------------------------------------------

    #[test]
    fn test_prop_slug_palette_assignment_invariants(slug in ".*") {
        let assigned = assign_color_for_slug(&slug);

        // Assigned color is always an active member of CURATED_PALETTE
        let is_in_palette = CURATED_PALETTE.iter().any(|nc| nc.color == assigned);
        prop_assert!(is_in_palette);

        // Whitespace padding does not alter assignment
        let padded = format!("   {slug}   ");
        let assigned_padded = assign_color_for_slug(&padded);
        prop_assert_eq!(assigned, assigned_padded);

        // assign_color top-level alias matches
        prop_assert_eq!(assigned, assign_color(&slug));
    }

    #[test]
    fn test_prop_uuid_palette_assignment_invariants(b in prop::array::uniform16(any::<u8>())) {
        let u = Uuid::from_bytes(b);
        let assigned = assign_color_for_uuid(&u);

        let is_in_palette = CURATED_PALETTE.iter().any(|nc| nc.color == assigned);
        prop_assert!(is_in_palette);
    }

    #[test]
    fn test_prop_custom_palette_assignment(
        palette_colors in prop::collection::vec(
            (0u8..=255, 0u8..=255, 0u8..=255).prop_map(|(r, g, b)| Color::rgb(r, g, b)),
            1..=30
        ),
        slug in ".*"
    ) {
        let palette = Palette::new(palette_colors.clone()).expect("palette is non-empty");
        let assigned = palette.assign(&slug);

        prop_assert!(palette_colors.contains(&assigned));
    }
}
