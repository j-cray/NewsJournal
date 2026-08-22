//! Property-based test suite for `newsjournal-core` validation engine.

use newsjournal_core::color::Color;
use newsjournal_core::validation::{
    format_phone_display, is_valid_email, is_valid_hex_color, is_valid_phone, is_valid_slug,
    normalize_email, normalize_hex_color, normalize_phone, slugify, validate_email,
    validate_headline, validate_hex_color, validate_max_length, validate_name, validate_non_empty,
    validate_phone, validate_slug, validate_task_title, ValidationError, MAX_EMAIL_LENGTH,
    MAX_HEADLINE_LENGTH, MAX_NAME_LENGTH, MAX_PHONE_DIGITS, MAX_SLUG_LENGTH, MAX_TASK_TITLE_LENGTH,
    MIN_PHONE_DIGITS,
};
use proptest::prelude::*;

proptest! {
    // -------------------------------------------------------------------------
    // 1. Slug Properties
    // -------------------------------------------------------------------------

    #[test]
    fn test_prop_slugify_is_always_valid_and_bounded(input in ".*") {
        let slug = slugify(&input);

        // Invariant 1: Slug is always valid according to validate_slug
        prop_assert!(validate_slug(&slug).is_ok());
        prop_assert!(is_valid_slug(&slug));

        // Invariant 2: Length is bounded by [1, MAX_SLUG_LENGTH]
        prop_assert!(!slug.is_empty());
        prop_assert!(slug.len() <= MAX_SLUG_LENGTH);

        // Invariant 3: Does not start or end with a hyphen or underscore
        prop_assert!(!slug.starts_with('-') && !slug.starts_with('_'));
        prop_assert!(!slug.ends_with('-') && !slug.ends_with('_'));

        // Invariant 4: Contains only valid characters
        let only_valid_chars = slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');
        prop_assert!(only_valid_chars);

        // Invariant 5: Idempotency on the generated slug
        let reslugified = slugify(&slug);
        prop_assert_eq!(&reslugified, &slug);
    }

    #[test]
    fn test_prop_valid_slug_generation(
        first in "[a-z0-9]",
        middle in "[a-z0-9_-]{0,50}",
        last in "[a-z0-9]"
    ) {
        let mut candidate = format!("{first}{middle}{last}");
        // Eliminate consecutive delimiters if any were randomly generated
        while candidate.contains("--") || candidate.contains("__") || candidate.contains("-_") || candidate.contains("_-") {
            candidate = candidate.replace("--", "-").replace("__", "_").replace("-_", "-").replace("_-", "_");
        }

        if candidate.len() <= MAX_SLUG_LENGTH && !candidate.starts_with('-') && !candidate.starts_with('_') && !candidate.ends_with('-') && !candidate.ends_with('_') {
            prop_assert!(validate_slug(&candidate).is_ok());
            prop_assert!(is_valid_slug(&candidate));
        }
    }

    #[test]
    fn test_prop_slug_uppercase_rejection(
        prefix in "[a-z0-9]{1,10}",
        upper in "[A-Z]{1,5}",
        suffix in "[a-z0-9]{1,10}"
    ) {
        let candidate = format!("{prefix}{upper}{suffix}");
        prop_assert!(validate_slug(&candidate).is_err());
        prop_assert!(!is_valid_slug(&candidate));
    }

    #[test]
    fn test_prop_slug_consecutive_delimiters_rejection(
        prefix in "[a-z0-9]{1,10}",
        delimiter in prop::sample::select(vec!["--", "__", "-_", "_-"]),
        suffix in "[a-z0-9]{1,10}"
    ) {
        let candidate = format!("{prefix}{delimiter}{suffix}");
        prop_assert!(validate_slug(&candidate).is_err());
    }

    #[test]
    fn test_prop_slug_length_rejection(
        len in (MAX_SLUG_LENGTH + 1)..=300usize
    ) {
        let candidate = "a".repeat(len);
        let result = validate_slug(&candidate);
        let is_expected_err = matches!(
            result,
            Err(ValidationError::FieldTooLong {
                field: "slug",
                max: MAX_SLUG_LENGTH,
                actual
            }) if actual == len
        );
        prop_assert!(is_expected_err);
    }

    // -------------------------------------------------------------------------
    // 2. Email Properties
    // -------------------------------------------------------------------------

    #[test]
    fn test_prop_valid_email_roundtrip(
        local in "[a-z0-9_+-]{1,20}",
        domain in "[a-z0-9]{2,15}",
        tld in "[a-z]{2,6}"
    ) {
        let email = format!("{local}@{domain}.{tld}");
        if email.len() <= MAX_EMAIL_LENGTH && !local.starts_with('.') && !local.ends_with('.') && !local.contains("..") {
            prop_assert!(validate_email(&email).is_ok());
            prop_assert!(is_valid_email(&email));

            let normalized = normalize_email(&email).expect("normalization should succeed");
            prop_assert_eq!(&normalized, &email.to_lowercase());

            // Idempotency: normalizing again returns identical string
            let renorm = normalize_email(&normalized).expect("renormalization should succeed");
            prop_assert_eq!(normalized, renorm);
        }
    }

    #[test]
    fn test_prop_email_whitespace_padding(
        local in "[a-z0-9]{2,10}",
        domain in "[a-z0-9]{2,10}",
        tld in "[a-z]{2,4}",
        spaces_before in "[ ]{1,5}",
        spaces_after in "[ ]{1,5}"
    ) {
        let clean = format!("{local}@{domain}.{tld}");
        let padded = format!("{spaces_before}{clean}{spaces_after}");

        let normalized = normalize_email(&padded).expect("padded email normalization should succeed");
        prop_assert_eq!(normalized, clean);
    }

    #[test]
    fn test_prop_email_missing_at_symbol(s in "[a-zA-Z0-9._-]{1,50}") {
        if !s.contains('@') {
            prop_assert!(validate_email(&s).is_err());
            prop_assert!(!is_valid_email(&s));
        }
    }

    #[test]
    fn test_prop_email_multiple_at_symbols(
        p1 in "[a-z0-9]{1,10}",
        p2 in "[a-z0-9]{1,10}",
        p3 in "[a-z0-9]{1,10}\\.[a-z]{2,4}"
    ) {
        let candidate = format!("{p1}@{p2}@{p3}");
        prop_assert!(validate_email(&candidate).is_err());
    }

    // -------------------------------------------------------------------------
    // 3. Phone Properties
    // -------------------------------------------------------------------------

    #[test]
    fn test_prop_phone_normalization_preserves_digits(
        has_plus in any::<bool>(),
        d1 in "[0-9]{3}",
        d2 in "[0-9]{3,4}",
        d3 in "[0-9]{4}"
    ) {
        let prefix = if has_plus { "+" } else { "" };
        let formatted = format!("{prefix}({d1}) {d2}-{d3}");

        prop_assert!(validate_phone(&formatted).is_ok());
        prop_assert!(is_valid_phone(&formatted));

        let normalized = normalize_phone(&formatted).expect("normalization should succeed");

        // Normalization retains exactly the digits and leading '+'
        let expected_digits = format!("{d1}{d2}{d3}");
        let expected_norm = if has_plus {
            format!("+{expected_digits}")
        } else {
            expected_digits
        };
        prop_assert_eq!(&normalized, &expected_norm);

        // Idempotency: normalizing normalized string produces the same result
        let renorm = normalize_phone(&normalized).expect("renormalization should succeed");
        prop_assert_eq!(normalized, renorm);
    }

    #[test]
    fn test_prop_phone_10_digit_display_formatting(
        d1 in "[0-9]{3}",
        d2 in "[0-9]{3}",
        d3 in "[0-9]{4}"
    ) {
        let raw = format!("{d1}{d2}{d3}");
        let display = format_phone_display(&raw);
        prop_assert_eq!(display, format!("({d1}) {d2}-{d3}"));

        let with_plus1 = format!("+1{raw}");
        let display_plus1 = format_phone_display(&with_plus1);
        prop_assert_eq!(display_plus1, format!("+1 ({d1}) {d2}-{d3}"));
    }

    #[test]
    fn test_prop_phone_digit_count_boundaries(
        too_few in 0..(MIN_PHONE_DIGITS),
        too_many in (MAX_PHONE_DIGITS + 1)..=30usize
    ) {
        let short_phone = "5".repeat(too_few);
        prop_assert!(validate_phone(&short_phone).is_err());

        let long_phone = "5".repeat(too_many);
        prop_assert!(validate_phone(&long_phone).is_err());
    }

    // -------------------------------------------------------------------------
    // 4. Hex Color Properties
    // -------------------------------------------------------------------------

    #[test]
    fn test_prop_rgb_to_hex_roundtrip(r in 0u8..=255, g in 0u8..=255, b in 0u8..=255) {
        let color = Color::rgb(r, g, b);
        let hex = color.to_hex();

        prop_assert!(validate_hex_color(&hex).is_ok());
        prop_assert!(is_valid_hex_color(&hex));

        let parsed = Color::from_hex(&hex).expect("parsing canonical hex must succeed");
        prop_assert_eq!(color, parsed);

        let normalized = normalize_hex_color(&hex).expect("normalizing hex must succeed");
        prop_assert_eq!(&hex, &normalized);
    }

    #[test]
    fn test_prop_rgba_to_hex_roundtrip(r in 0u8..=255, g in 0u8..=255, b in 0u8..=255, a in 0u8..=255) {
        let color = Color::rgba(r, g, b, a);
        let hex = color.to_hex_rgba();

        prop_assert!(validate_hex_color(&hex).is_ok());
        prop_assert!(is_valid_hex_color(&hex));

        let parsed = Color::from_hex(&hex).expect("parsing canonical rgba hex must succeed");
        prop_assert_eq!(color, parsed);

        let normalized = normalize_hex_color(&hex).expect("normalizing rgba hex must succeed");
        prop_assert_eq!(&hex, &normalized);
    }

    #[test]
    fn test_prop_invalid_hex_characters(
        prefix in prop::sample::select(vec!["#", ""]),
        invalid_char in "[g-zG-Z!@$%^&*]",
        suffix in "[0-9a-fA-F]{2,5}"
    ) {
        let hex = format!("{prefix}{invalid_char}{suffix}");
        prop_assert!(validate_hex_color(&hex).is_err());
        prop_assert!(!is_valid_hex_color(&hex));
    }

    // -------------------------------------------------------------------------
    // 5. Text & Length Bounds
    // -------------------------------------------------------------------------

    #[test]
    fn test_prop_whitespace_only_text_fails(spaces in "[ \t\n\r]{1,100}") {
        prop_assert!(validate_non_empty("headline", &spaces).is_err());
        prop_assert!(validate_name(&spaces).is_err());
        prop_assert!(validate_headline(&spaces).is_err());
        prop_assert!(validate_task_title(&spaces).is_err());
    }

    #[test]
    fn test_prop_text_max_length_enforcement(
        field in prop::sample::select(vec!["name", "headline", "title"]),
        limit in 10usize..=200usize,
        excess in 1usize..=50usize
    ) {
        let valid_str = "x".repeat(limit);
        prop_assert!(validate_max_length(field, &valid_str, limit).is_ok());

        let invalid_str = "x".repeat(limit + excess);
        let result = validate_max_length(field, &invalid_str, limit);
        let is_expected = matches!(
            result,
            Err(ValidationError::FieldTooLong { max, actual, .. }) if max == limit && actual == limit + excess
        );
        prop_assert!(is_expected);
    }

    #[test]
    fn test_prop_entity_text_limits(
        name_len in (MAX_NAME_LENGTH + 1)..=(MAX_NAME_LENGTH + 50),
        headline_len in (MAX_HEADLINE_LENGTH + 1)..=(MAX_HEADLINE_LENGTH + 50),
        task_len in (MAX_TASK_TITLE_LENGTH + 1)..=(MAX_TASK_TITLE_LENGTH + 50)
    ) {
        let long_name = "n".repeat(name_len);
        prop_assert!(validate_name(&long_name).is_err());

        let long_headline = "h".repeat(headline_len);
        prop_assert!(validate_headline(&long_headline).is_err());

        let long_task = "t".repeat(task_len);
        prop_assert!(validate_task_title(&long_task).is_err());
    }
}
