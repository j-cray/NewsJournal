//! Integration test suite for `newsjournal-core` color palette and accessibility engine.

use std::collections::HashSet;

use newsjournal_core::color::{
    assign_color, assign_color_for_slug, assign_color_for_uuid, fnv1a_hash, fnv1a_hash_str,
    fnv1a_hash_uuid, Color, ColorError, NamedColor, Palette, CURATED_PALETTE,
};
use newsjournal_core::{Article, ModelError};
use uuid::Uuid;

#[test]
fn test_color_hex_parsing_and_formatting() {
    // 3-digit shorthand
    let white3 = Color::from_hex("#FFF").expect("failed to parse #FFF");
    assert_eq!(white3, Color::rgb(255, 255, 255));
    assert_eq!(white3.to_hex(), "#FFFFFF");

    let black3 = Color::from_hex("#000").expect("failed to parse #000");
    assert_eq!(black3, Color::rgb(0, 0, 0));
    assert_eq!(black3.to_hex(), "#000000");

    let mixed3 = Color::from_hex("#1a3").expect("failed to parse #1a3");
    assert_eq!(mixed3, Color::rgb(0x11, 0xAA, 0x33));

    // 4-digit shorthand with alpha
    let semi_white = Color::from_hex("#FFF8").expect("failed to parse #FFF8");
    assert_eq!(semi_white, Color::rgba(255, 255, 255, 0x88));
    assert_eq!(semi_white.to_hex_rgba(), "#FFFFFF88");

    // 6-digit standard
    let cobalt = Color::from_hex("#2563EB").expect("failed to parse #2563EB");
    assert_eq!(cobalt, Color::rgb(37, 99, 235));
    assert_eq!(cobalt.to_hex(), "#2563EB");
    assert_eq!(cobalt.to_string(), "#2563EB");

    // Lowercase hex input
    let emerald = Color::from_hex("#059669").expect("failed to parse lowercase");
    assert_eq!(emerald, Color::rgb(5, 150, 105));
    assert_eq!(emerald.to_hex(), "#059669");

    // 8-digit standard with alpha
    let translucent = Color::from_hex("#E11D4880").expect("failed to parse #E11D4880");
    assert_eq!(translucent, Color::rgba(225, 29, 72, 128));
    assert_eq!(translucent.to_hex_rgba(), "#E11D4880");
    assert_eq!(translucent.to_string(), "#E11D4880");

    // Roundtrip verification
    for nc in CURATED_PALETTE {
        let hex = nc.color.to_hex();
        let parsed = Color::from_hex(&hex).expect("roundtrip parse failed");
        assert_eq!(nc.color, parsed);
    }
}

#[test]
fn test_color_numeric_and_float_conversions() {
    let color = Color::rgba(128, 64, 32, 255);
    assert_eq!(color.to_rgb(), (128, 64, 32));
    assert_eq!(color.to_rgba(), (128, 64, 32, 255));

    let (r, g, b) = color.to_f32_rgb();
    assert!((r - (128.0 / 255.0)).abs() < 1e-6);
    assert!((g - (64.0 / 255.0)).abs() < 1e-6);
    assert!((b - (32.0 / 255.0)).abs() < 1e-6);

    let (fr, fg, fb, fa) = color.to_f32_rgba();
    assert!((fr - (128.0 / 255.0)).abs() < 1e-6);
    assert!((fg - (64.0 / 255.0)).abs() < 1e-6);
    assert!((fb - (32.0 / 255.0)).abs() < 1e-6);
    assert!((fa - 1.0).abs() < 1e-6);

    let from_u32 = Color::from_rgb_u32(0x804020);
    assert_eq!(from_u32, Color::rgb(128, 64, 32));

    let from_u32_rgba = Color::from_rgba_u32(0x80402080);
    assert_eq!(from_u32_rgba, Color::rgba(128, 64, 32, 128));
}

#[test]
fn test_invalid_hex_error_handling() {
    // Missing '#' prefix
    let err_prefix = Color::from_hex("2563EB");
    assert!(matches!(
        err_prefix,
        Err(ColorError::InvalidHexFormat { .. })
    ));

    // Invalid non-hex characters
    let err_char = Color::from_hex("#GGGGGG");
    assert!(matches!(
        err_char,
        Err(ColorError::InvalidHexDigit { character: 'G', .. })
    ));

    // Invalid lengths
    let invalid_lengths = ["#1", "#12", "#12345", "#1234567", "#123456789"];
    for invalid in invalid_lengths {
        let err_len = Color::from_hex(invalid);
        assert!(
            matches!(err_len, Err(ColorError::InvalidLength { .. })),
            "expected InvalidLength for '{invalid}'"
        );
    }
}

#[test]
fn test_wcag_luminance_and_contrast_spec() {
    let black = Color::rgb(0, 0, 0);
    let white = Color::rgb(255, 255, 255);

    // WCAG luminance extremes
    assert_eq!(black.relative_luminance(), 0.0);
    assert!((white.relative_luminance() - 1.0).abs() < 1e-5);

    // Maximum contrast ratio is 21.0
    let bw_contrast = black.contrast_ratio(&white);
    assert!((bw_contrast - 21.0).abs() < 0.01);

    // Symmetry
    assert_eq!(black.contrast_ratio(&white), white.contrast_ratio(&black));

    // Minimum contrast ratio with itself is 1.0
    assert!((cobalt_color().contrast_ratio(&cobalt_color()) - 1.0).abs() < 1e-5);

    // Text color optimization
    let dark_navy = Color::rgb(15, 23, 42);
    assert_eq!(dark_navy.best_text_color_for_background(), white);

    let light_gray = Color::rgb(243, 244, 246);
    assert_eq!(
        light_gray.best_text_color_for_background(),
        Color::rgb(30, 41, 59)
    );
}

fn cobalt_color() -> Color {
    Color::rgb(37, 99, 235)
}

#[test]
fn test_curated_palette_accessibility_invariants() {
    assert!(
        CURATED_PALETTE.len() >= 16,
        "palette should contain at least 16 colors"
    );

    let mut hexes = HashSet::new();
    let mut names = HashSet::new();

    for nc in CURATED_PALETTE {
        // Distinctness
        assert!(
            hexes.insert(nc.color.to_hex()),
            "duplicate hex: {}",
            nc.color.to_hex()
        );
        assert!(names.insert(nc.name), "duplicate name: {}", nc.name);

        let (r, g, b) = nc.color.to_rgb();

        // Check for absence of unreadable light yellow (high R, high G, low B)
        let is_light_yellow = r >= 220 && g >= 200 && b <= 120;
        assert!(
            !is_light_yellow,
            "color '{}' ({}) resembles unreadable light yellow",
            nc.name,
            nc.color.to_hex()
        );

        // Check that luminance is safely within readable thresholds
        let lum = nc.color.relative_luminance();
        assert!(
            lum <= 0.70,
            "color '{}' ({}) luminance {:.3} is too washed-out",
            nc.name,
            nc.color.to_hex(),
            lum
        );
        assert!(
            lum >= 0.02,
            "color '{}' ({}) luminance {:.3} is too dark",
            nc.name,
            nc.color.to_hex(),
            lum
        );

        // Optimal text color against this palette entry achieves >= 3.0 contrast
        let best_text = nc.color.best_text_color_for_background();
        let text_contrast = nc.color.contrast_ratio(&best_text);
        assert!(
            text_contrast >= 3.0,
            "color '{}' ({}) best text contrast {:.2} is below 3.0",
            nc.name,
            nc.color.to_hex(),
            text_contrast
        );
    }
}

#[test]
fn test_deterministic_hashing_and_palette_assignment() {
    let slugs = [
        "breaking-news-city-hall",
        "investigation-police-misconduct",
        "transit-authority-budget-2026",
        "climate-change-coastal-erosion",
        "school-board-curriculum-debate",
        "hospital-staffing-shortages",
        "sports-championship-preview",
        "arts-festival-downtown",
    ];

    for slug in slugs {
        let c1 = assign_color_for_slug(slug);
        let c2 = assign_color_for_slug(slug);
        let c3 = assign_color_for_slug(&format!("  {slug}  ")); // Trim check
        assert_eq!(c1, c2, "assignment must be deterministic for '{slug}'");
        assert_eq!(
            c1, c3,
            "whitespace trimming should not change color for '{slug}'"
        );
    }

    // UUID deterministic assignment
    let u1 = Uuid::new_v4();
    let cu1 = assign_color_for_uuid(&u1);
    let cu2 = assign_color_for_uuid(&u1);
    assert_eq!(cu1, cu2, "UUID color assignment must be deterministic");

    // Palette distribution check over 100 sample slugs
    let mut chosen_colors = HashSet::new();
    for i in 0..100 {
        let slug = format!("news-story-slug-sample-{i}");
        let color = assign_color_for_slug(&slug);
        chosen_colors.insert(color);
    }
    // With 16 colors and 100 distinct slugs, we should hit at least half of the palette
    assert!(
        chosen_colors.len() >= 8,
        "Color distribution across palette is too narrow (only {} colors used)",
        chosen_colors.len()
    );
}

#[test]
fn test_article_color_integration_and_helpers() {
    // 1. Article with explicit custom color
    let article_with_color =
        Article::new("custom-colored-story", "Custom Headline").with_color("#E53935");
    assert_eq!(article_with_color.color.as_deref(), Some("#E53935"));
    assert_eq!(
        article_with_color.color_or_default(),
        Color::from_hex("#E53935").unwrap()
    );

    // 2. Article without color gets deterministic fallback
    let article_no_color = Article::new("fallback-story", "Fallback Story");
    assert!(article_no_color.color.is_none());
    assert_eq!(
        article_no_color.color_or_default(),
        assign_color_for_slug("fallback-story")
    );

    // 3. Article with_auto_color persists deterministic color
    let article_auto = Article::new("auto-assigned-story", "Auto Headline").with_auto_color();
    assert_eq!(
        article_auto.color.as_deref(),
        Some(
            assign_color_for_slug("auto-assigned-story")
                .to_hex()
                .as_str()
        )
    );

    // 4. Article assign_default_color mutation
    let mut mutating_article = Article::new("mutating-story", "Mutating Headline");
    let original_updated_at = mutating_article.updated_at;
    mutating_article.assign_default_color();
    assert_eq!(
        mutating_article.color.as_deref(),
        Some(assign_color_for_slug("mutating-story").to_hex().as_str())
    );
    assert!(mutating_article.updated_at >= original_updated_at);

    // 5. Article builder auto_color
    let built_article = Article::builder("builder-story", "Builder Headline")
        .auto_color()
        .build();
    assert_eq!(
        built_article.color.as_deref(),
        Some(assign_color_for_slug("builder-story").to_hex().as_str())
    );
}

#[test]
fn test_error_conversion_into_model_error() {
    let color_err = ColorError::InvalidLength {
        color: "#123".to_string(),
        length: 3,
    };
    let model_err: ModelError = color_err.clone().into();
    assert_eq!(model_err, ModelError::Color(color_err));
}

#[test]
fn test_custom_palette_construction() {
    let colors = vec![
        Color::rgb(255, 0, 0),
        Color::rgb(0, 255, 0),
        Color::rgb(0, 0, 255),
    ];
    let palette = Palette::new(colors.clone()).expect("valid palette");
    assert_eq!(palette.len(), 3);
    assert!(!palette.is_empty());
    assert_eq!(palette.colors(), &colors[..]);

    let assigned = palette.assign("some-article-slug");
    assert!(colors.contains(&assigned));

    let assigned_uuid = palette.assign_uuid(&Uuid::nil());
    assert!(colors.contains(&assigned_uuid));

    let empty_err = Palette::new(vec![]);
    assert!(matches!(empty_err, Err(ColorError::EmptyPalette)));
}

#[test]
fn test_hash_functions_and_named_color() {
    let raw_hash = fnv1a_hash(b"test-bytes");
    let str_hash = fnv1a_hash_str("test-bytes");
    assert_eq!(raw_hash, str_hash);

    let uuid = Uuid::nil();
    let uuid_hash = fnv1a_hash_uuid(&uuid);
    assert_eq!(uuid_hash, fnv1a_hash(uuid.as_bytes()));

    let general_color = assign_color("general-topic");
    assert_eq!(general_color, assign_color_for_slug("general-topic"));

    let nc = NamedColor::new("Test Red", Color::rgb(255, 0, 0));
    assert_eq!(nc.name, "Test Red");
    assert_eq!(nc.color, Color::rgb(255, 0, 0));
}
