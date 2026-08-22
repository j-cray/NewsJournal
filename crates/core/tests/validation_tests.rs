//! Comprehensive integration test suite for `newsjournal-core` validation engine.

use newsjournal_core::validation::{
    format_phone_display, is_valid_email, is_valid_hex_color, is_valid_phone, is_valid_slug,
    normalize_email, normalize_hex_color, normalize_phone, slugify, validate_email,
    validate_headline, validate_hex_color, validate_name, validate_non_empty, validate_phone,
    validate_slug, validate_task_title, ValidationError, MAX_EMAIL_LENGTH, MAX_HEADLINE_LENGTH,
    MAX_NAME_LENGTH, MAX_PHONE_DIGITS, MAX_SLUG_LENGTH, MAX_TASK_TITLE_LENGTH, MIN_PHONE_DIGITS,
};
use newsjournal_core::{Article, Contact, ModelError, Task};
use uuid::Uuid;

#[test]
fn test_slug_validation_comprehensive() {
    // Valid slugs
    let valid_cases = [
        "breaking-news",
        "story_2026_08_22",
        "city-hall-budget-investigation",
        "article123",
        "a",
        "1",
        "x-y-z",
        "x_y_z",
        "2026-08-22-special-report",
    ];
    for slug in valid_cases {
        assert!(validate_slug(slug).is_ok(), "failed for valid slug: {slug}");
        assert!(is_valid_slug(slug));
    }

    // Invalid slugs
    let invalid_cases = [
        ("", "empty"),
        ("   ", "whitespace"),
        ("-leading-dash", "leading dash"),
        ("_leading_underscore", "leading underscore"),
        ("trailing-dash-", "trailing dash"),
        ("trailing-underscore_", "trailing underscore"),
        ("double--dash", "consecutive hyphens"),
        ("double__underscore", "consecutive underscores"),
        ("mixed-_hyphen_underscore", "consecutive mixed"),
        ("uppercase-Slug", "uppercase characters"),
        ("slug with spaces", "spaces"),
        ("slug/with/slashes", "slashes"),
        ("slug.with.dots", "dots"),
        ("slug@email", "at symbol"),
        ("slug#tag", "hash symbol"),
        ("slug!", "exclamation"),
    ];
    for (slug, reason) in invalid_cases {
        assert!(
            validate_slug(slug).is_err(),
            "expected error for {slug} ({reason})"
        );
        assert!(!is_valid_slug(slug));
    }

    // Boundary check
    let exact_max = "a".repeat(MAX_SLUG_LENGTH);
    assert!(validate_slug(&exact_max).is_ok());

    let over_max = "a".repeat(MAX_SLUG_LENGTH + 1);
    assert!(matches!(
        validate_slug(&over_max),
        Err(ValidationError::FieldTooLong {
            field: "slug",
            max: 128,
            actual: 129
        })
    ));
}

#[test]
fn test_slugify_various_inputs() {
    assert_eq!(
        slugify("Mayor's Office Budget: $4.2 Billion Transit Expansion (2026)"),
        "mayor-s-office-budget-4-2-billion-transit-expansion-2026"
    );
    assert_eq!(
        slugify("  *** EXCLUSIVE: Confidential Whistleblower Interview ***  "),
        "exclusive-confidential-whistleblower-interview"
    );
    assert_eq!(slugify("100% Verified Facts!"), "100-verified-facts");
    assert_eq!(slugify("---___---"), "story");
    assert_eq!(slugify(""), "story");
}

#[test]
fn test_email_validation_comprehensive() {
    let valid_emails = [
        "reporter@newspaper.com",
        "first.middle.last@domain.co.uk",
        "tips+anonymous@leak.org",
        "investigations@agency.gov",
        "user_name.123%promo-test@mail.sub.example.net",
    ];
    for email in valid_emails {
        assert!(
            validate_email(email).is_ok(),
            "failed for valid email: {email}"
        );
        assert!(is_valid_email(email));
    }

    let invalid_emails = [
        "",
        "   ",
        "no-at-sign.com",
        "two@@signs.com",
        "@missing-user.com",
        "user@missing-domain",
        "user@domain..com",
        ".user@domain.com",
        "user.@domain.com",
        "us..er@domain.com",
        "user@-domain.com",
        "user@domain-.com",
        "user@.com",
        "user@domain.c",
        "user@domain.123",
        "user with space@domain.com",
    ];
    for email in invalid_emails {
        assert!(
            validate_email(email).is_err(),
            "expected error for email: {email}"
        );
        assert!(!is_valid_email(email));
    }

    // Normalization
    assert_eq!(
        normalize_email("  EDITOR@DailyGazette.COM  ").unwrap(),
        "editor@dailygazette.com"
    );

    // Test email length boundary
    let too_long_email = format!("user@{}.com", "a".repeat(MAX_EMAIL_LENGTH));
    assert!(validate_email(&too_long_email).is_err());
}

#[test]
fn test_phone_validation_and_normalization() {
    let valid_phones = [
        "+1 (555) 123-4567",
        "555-123-4567",
        "+44 20 7946 0958",
        "+33 1 42 68 55 00",
        "555.019.2834",
        "(555) 0192834",
        "+15551234567",
        "1234567",
    ];
    for phone in valid_phones {
        assert!(
            validate_phone(phone).is_ok(),
            "failed for valid phone: {phone}"
        );
        assert!(is_valid_phone(phone));
    }

    let invalid_phones = [
        ("", "empty"),
        ("   ", "whitespace"),
        ("123456", "too few digits (6)"),
        ("1234567890123456", "too many digits (16)"),
        ("phone-number", "letters"),
        ("+1 +555-1234", "multiple plus signs"),
        ("1+555-1234", "plus in middle"),
        ("(555-1234", "unclosed paren"),
        ("555)-1234", "unopened paren"),
        ("555-1234-@", "invalid symbol"),
    ];
    for (phone, reason) in invalid_phones {
        assert!(
            validate_phone(phone).is_err(),
            "expected error for phone: {phone} ({reason})"
        );
        assert!(!is_valid_phone(phone));
    }

    assert_eq!(
        normalize_phone("+1 (555) 123-4567").unwrap(),
        "+15551234567"
    );
    assert_eq!(normalize_phone("555.123.4567").unwrap(), "5551234567");

    assert_eq!(format_phone_display("5551234567"), "(555) 123-4567");
    assert_eq!(format_phone_display("+15551234567"), "+1 (555) 123-4567");

    // Digit limits
    const { assert!(MIN_PHONE_DIGITS <= MAX_PHONE_DIGITS) };
    let min_digits = "1".repeat(MIN_PHONE_DIGITS);
    assert!(validate_phone(&min_digits).is_ok());
    let max_digits = "1".repeat(MAX_PHONE_DIGITS);
    assert!(validate_phone(&max_digits).is_ok());
}

#[test]
fn test_text_and_color_validation() {
    assert!(validate_non_empty("name", "Alice").is_ok());
    assert!(validate_non_empty("name", "   ").is_err());

    assert!(validate_name("Bob Woodward").is_ok());
    assert!(validate_name("").is_err());
    assert!(validate_name(&"x".repeat(MAX_NAME_LENGTH + 1)).is_err());

    assert!(validate_headline("Breaking: Historic Treaty Signed").is_ok());
    assert!(validate_headline("").is_err());
    assert!(validate_headline(&"h".repeat(MAX_HEADLINE_LENGTH + 1)).is_err());

    assert!(validate_task_title("Review draft").is_ok());
    assert!(validate_task_title("").is_err());
    assert!(validate_task_title(&"t".repeat(MAX_TASK_TITLE_LENGTH + 1)).is_err());

    assert!(validate_hex_color("#3498DB").is_ok());
    assert!(is_valid_hex_color("#3498DB"));
    assert!(validate_hex_color("#fff").is_ok());
    assert!(validate_hex_color("#12345678").is_ok());
    assert!(validate_hex_color("3498DB").is_err());
    assert!(!is_valid_hex_color("3498DB"));
    assert!(validate_hex_color("#GGGGGG").is_err());
    assert!(validate_hex_color("#12").is_err());

    assert_eq!(normalize_hex_color("#abc").unwrap(), "#AABBCC");
    assert_eq!(normalize_hex_color("#3498db").unwrap(), "#3498DB");
}

#[test]
fn test_model_error_conversion() {
    let val_err = ValidationError::EmptyField { field: "headline" };
    let model_err: ModelError = val_err.clone().into();
    assert_eq!(model_err, ModelError::Validation(val_err));
    assert_eq!(
        format!("{model_err}"),
        "validation error: field 'headline' cannot be empty"
    );
}

#[test]
fn test_article_validation_flow() {
    let article = Article::builder("clean-energy-transition", "Clean Energy Bill Approved")
        .color("#27AE60")
        .build_validated()
        .expect("should be valid");

    assert_eq!(article.slug, "clean-energy-transition");

    let invalid_article =
        Article::builder("Clean Energy", "Clean Energy Bill Approved").build_validated();
    assert!(matches!(
        invalid_article,
        Err(ValidationError::InvalidSlug { .. })
    ));
}

#[test]
fn test_task_validation_flow() {
    let task = Task::builder(Uuid::new_v4(), "Verify source claims")
        .build_validated()
        .expect("should be valid");
    assert_eq!(task.title, "Verify source claims");

    let invalid_task = Task::builder(Uuid::new_v4(), "").build_validated();
    assert!(matches!(
        invalid_task,
        Err(ValidationError::EmptyField { field: "title" })
    ));
}

#[test]
fn test_contact_validation_flow() {
    let contact = Contact::builder("Deep Throat")
        .email("informant@securemail.org")
        .phone("+1-555-0199")
        .build_validated()
        .expect("should be valid");
    assert_eq!(contact.name, "Deep Throat");

    let invalid_contact = Contact::builder("Deep Throat")
        .email("invalid-email-address")
        .build_validated();
    assert!(matches!(
        invalid_contact,
        Err(ValidationError::InvalidEmail { .. })
    ));
}
