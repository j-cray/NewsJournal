//! Article creation and editing form field presentation models, validation state, and view builders.

use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc, Weekday};
use newsjournal_core::color::{assign_color_for_slug, Color, CURATED_PALETTE};
use newsjournal_core::models::ArticleStage;
use newsjournal_core::validation::MAX_SLUG_LENGTH;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::modal::ArticleDraft;
use crate::state::AppState;
use crate::views::article_card::calculate_contrast_color;

/// Maximum length for an article headline.
pub const MAX_HEADLINE_LENGTH: usize = 250;

/// Default target time of day for deadline presets (17:00 / 5:00 PM).
pub const DEFAULT_DEADLINE_HOUR: u32 = 17;

/// Sizing and visual presentation view model for the unique Slug field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArticleSlugFieldViewModel {
    /// Current slug text value.
    pub value: String,
    /// Placeholder hint text.
    pub placeholder: &'static str,
    /// Maximum character limit (128).
    pub max_length: usize,
    /// Current character count.
    pub char_count: usize,
    /// Active validation error message (if any).
    pub error: Option<String>,
    /// Whether the slug format is currently valid and non-empty.
    pub is_valid: bool,
    /// Whether the slug collides with an existing story in the database.
    pub is_collision: bool,
    /// Explanatory guidance text.
    pub help_text: &'static str,
}

/// Sizing and presentation view model for the Story Headline field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArticleHeadlineFieldViewModel {
    /// Current headline text value.
    pub value: String,
    /// Placeholder hint text.
    pub placeholder: &'static str,
    /// Maximum character limit (250).
    pub max_length: usize,
    /// Current character count.
    pub char_count: usize,
    /// Active validation error message (if any).
    pub error: Option<String>,
    /// Whether the headline is non-empty and within limits.
    pub is_valid: bool,
}

/// Presentation view model for the multi-line Story Description / Notes text area.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArticleDescriptionFieldViewModel {
    /// Current text content.
    pub value: String,
    /// Placeholder guidance.
    pub placeholder: &'static str,
    /// Total character count.
    pub char_count: usize,
    /// Line count for sizing the text area.
    pub line_count: usize,
}

/// Presentation model for a single stage option in the Stage selector dropdown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StageOptionViewModel {
    /// The article stage enum variant.
    pub stage: ArticleStage,
    /// User-visible title (e.g. "Ready to Publish").
    pub label: &'static str,
    /// Short label or code (e.g. "Ready").
    pub short_label: &'static str,
    /// Visual stage icon emoji.
    pub icon_emoji: &'static str,
    /// Stage badge color hex code.
    pub badge_color_hex: &'static str,
    /// Editorial workflow description for this stage.
    pub description: &'static str,
    /// Whether this option is currently selected.
    pub is_selected: bool,
    /// Workflow sequence step number (1 to 6).
    pub step_number: usize,
}

/// Presentation view model for the Stage dropdown selector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArticleStageFieldViewModel {
    /// Currently selected stage.
    pub selected_stage: ArticleStage,
    /// User-visible label of selected stage.
    pub selected_label: &'static str,
    /// Icon emoji of selected stage.
    pub selected_icon: &'static str,
    /// Badge color hex of selected stage.
    pub selected_badge_color_hex: &'static str,
    /// All 6 available editorial stage options.
    pub options: Vec<StageOptionViewModel>,
}

/// Quick preset intervals for selecting article deadlines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeadlinePreset {
    /// Today at 17:00 UTC.
    Today5PM,
    /// Tomorrow at 17:00 UTC.
    Tomorrow5PM,
    /// End of current week (Friday at 17:00 UTC).
    EndOfWeek,
    /// Next Monday at 09:00 UTC.
    NextWeek,
    /// Exactly two weeks from now at 17:00 UTC.
    InTwoWeeks,
    /// Clear / Remove deadline.
    Clear,
}

impl DeadlinePreset {
    /// Returns user-facing label for this preset.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Today5PM => "Today 5 PM",
            Self::Tomorrow5PM => "Tomorrow 5 PM",
            Self::EndOfWeek => "End of Week (Fri)",
            Self::NextWeek => "Next Week (Mon)",
            Self::InTwoWeeks => "In 2 Weeks",
            Self::Clear => "No Deadline",
        }
    }

    /// Computes the target UTC datetime for this preset given a reference time.
    #[must_use]
    pub fn calculate_target_datetime(&self, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match self {
            Self::Today5PM => Utc
                .with_ymd_and_hms(
                    now.year(),
                    now.month(),
                    now.day(),
                    DEFAULT_DEADLINE_HOUR,
                    0,
                    0,
                )
                .single(),
            Self::Tomorrow5PM => {
                let tomorrow = now + Duration::days(1);
                Utc.with_ymd_and_hms(
                    tomorrow.year(),
                    tomorrow.month(),
                    tomorrow.day(),
                    DEFAULT_DEADLINE_HOUR,
                    0,
                    0,
                )
                .single()
            }
            Self::EndOfWeek => {
                // Find next Friday (or today if Friday)
                let days_until_friday = match now.weekday() {
                    Weekday::Mon => 4,
                    Weekday::Tue => 3,
                    Weekday::Wed => 2,
                    Weekday::Thu => 1,
                    Weekday::Fri => 0,
                    Weekday::Sat => 6,
                    Weekday::Sun => 5,
                };
                let friday = now + Duration::days(days_until_friday);
                Utc.with_ymd_and_hms(
                    friday.year(),
                    friday.month(),
                    friday.day(),
                    DEFAULT_DEADLINE_HOUR,
                    0,
                    0,
                )
                .single()
            }
            Self::NextWeek => {
                // Find next Monday
                let days_until_monday = match now.weekday() {
                    Weekday::Mon => 7,
                    Weekday::Tue => 6,
                    Weekday::Wed => 5,
                    Weekday::Thu => 4,
                    Weekday::Fri => 3,
                    Weekday::Sat => 2,
                    Weekday::Sun => 1,
                };
                let next_monday = now + Duration::days(days_until_monday);
                Utc.with_ymd_and_hms(
                    next_monday.year(),
                    next_monday.month(),
                    next_monday.day(),
                    9,
                    0,
                    0,
                )
                .single()
            }
            Self::InTwoWeeks => {
                let future = now + Duration::days(14);
                Utc.with_ymd_and_hms(
                    future.year(),
                    future.month(),
                    future.day(),
                    DEFAULT_DEADLINE_HOUR,
                    0,
                    0,
                )
                .single()
            }
            Self::Clear => None,
        }
    }
}

/// Presentation model for a quick deadline preset button.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeadlinePresetViewModel {
    /// Preset variant.
    pub preset: DeadlinePreset,
    /// User-visible button label.
    pub label: &'static str,
    /// Whether this preset currently matches the selected deadline.
    pub is_active: bool,
}

/// Presentation view model for the Deadline Picker field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArticleDeadlineFieldViewModel {
    /// Current UTC deadline timestamp (if set).
    pub deadline: Option<DateTime<Utc>>,
    /// Whether a deadline is configured.
    pub has_deadline: bool,
    /// ISO formatted date string (e.g. "2026-08-25") for date input controls.
    pub date_iso: Option<String>,
    /// 24-hour formatted time string (e.g. "17:00") for time input controls.
    pub time_hhmm: Option<String>,
    /// Human-friendly formatted date/time (e.g. "Aug 25, 2026 at 5:00 PM").
    pub formatted_display: String,
    /// Human-readable relative urgency hint (e.g. "Due in 3 days", "⚠️ Overdue by 2 hours").
    pub relative_hint: String,
    /// Whether the current deadline timestamp is in the past and article is not Published.
    pub is_overdue: bool,
    /// Quick preset buttons.
    pub presets: Vec<DeadlinePresetViewModel>,
}

/// Presentation model for an accessible curated color swatch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ColorSwatchViewModel {
    /// Human-readable color name (e.g. "Cobalt Blue").
    pub name: &'static str,
    /// Hex color code (e.g. "#2563EB").
    pub hex: String,
    /// 8-bit RGB components.
    pub rgb: (u8, u8, u8),
    /// High-contrast foreground text color for rendering on this background (`#000000` or `#FFFFFF`).
    pub contrast_text_color: &'static str,
    /// Whether this swatch matches the currently selected color.
    pub is_selected: bool,
}

/// Presentation view model for the Color Picker & Swatch grid.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArticleColorPickerViewModel {
    /// Currently selected hex color code.
    pub selected_hex: String,
    /// 8-bit RGB components of the selected color.
    pub selected_rgb: (u8, u8, u8),
    /// High contrast text color for badges using this color.
    pub contrast_text_color: &'static str,
    /// Deterministic hash color derived from the slug.
    pub auto_hash_hex: String,
    /// Whether the current color is a custom user override.
    pub is_custom: bool,
    /// 16 curated accessible color swatches.
    pub swatches: Vec<ColorSwatchViewModel>,
    /// Custom hex input validation error (if any).
    pub custom_hex_error: Option<String>,
}

/// Comprehensive presentation model for all Article Form fields in the modal/drawer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArticleFormViewModel {
    /// Whether this form is editing an existing article (`true`) or creating a new one (`false`).
    pub is_edit: bool,
    /// Target article ID (if editing).
    pub article_id: Option<Uuid>,
    /// Unique Slug field presentation model.
    pub slug_field: ArticleSlugFieldViewModel,
    /// Headline field presentation model.
    pub headline_field: ArticleHeadlineFieldViewModel,
    /// Description / story notes field presentation model.
    pub description_field: ArticleDescriptionFieldViewModel,
    /// Stage selector dropdown presentation model.
    pub stage_field: ArticleStageFieldViewModel,
    /// Deadline picker presentation model.
    pub deadline_field: ArticleDeadlineFieldViewModel,
    /// Color picker swatch grid presentation model.
    pub color_picker: ArticleColorPickerViewModel,
    /// Number of contacts tagged in this draft.
    pub tagged_contacts_count: usize,
    /// Whether the form is currently submittable (valid format, unique slug, non-empty required fields).
    pub is_submittable: bool,
    /// Total count of validation errors.
    pub total_error_count: usize,
    /// Formatted error summary string (if errors exist).
    pub error_summary: Option<String>,
}

/// Builds the complete `ArticleFormViewModel` from current application state and active draft.
#[must_use]
pub fn build_article_form_view(state: &AppState, draft: &ArticleDraft) -> ArticleFormViewModel {
    let now = Utc::now();
    let is_edit = draft.id.is_some();
    let existing_slugs: Vec<(Uuid, String)> = state
        .articles
        .iter()
        .map(|a| (a.id, a.slug.clone()))
        .collect();

    // 1. Build Slug field view model
    let clean_slug = draft.slug.trim();
    let is_collision = draft.slug_collides_with(&existing_slugs);
    let slug_error = draft.validation_errors.get("slug").cloned().or_else(|| {
        if is_collision {
            Some("Slug is already in use by another story".to_string())
        } else {
            None
        }
    });

    let slug_field = ArticleSlugFieldViewModel {
        value: draft.slug.clone(),
        placeholder: "e.g. city-hall-transit-probe",
        max_length: MAX_SLUG_LENGTH,
        char_count: draft.slug.chars().count(),
        is_valid: slug_error.is_none() && !clean_slug.is_empty(),
        is_collision,
        error: slug_error,
        help_text: "Unique story identifier used for filesystem references and tags. Lowercase alphanumeric and hyphens.",
    };

    // 2. Build Headline field view model
    let headline_error = draft.validation_errors.get("headline").cloned();
    let headline_char_count = draft.headline.chars().count();
    let headline_field = ArticleHeadlineFieldViewModel {
        value: draft.headline.clone(),
        placeholder: "Enter story headline...",
        max_length: MAX_HEADLINE_LENGTH,
        char_count: headline_char_count,
        is_valid: headline_error.is_none() && !draft.headline.trim().is_empty(),
        error: headline_error,
    };

    // 3. Build Description field view model
    let desc_lines = if draft.description.is_empty() {
        1
    } else {
        draft.description.lines().count().max(1)
    };
    let description_field = ArticleDescriptionFieldViewModel {
        value: draft.description.clone(),
        placeholder:
            "Enter story background, key angles, interview questions, or reporting notes...",
        char_count: draft.description.chars().count(),
        line_count: desc_lines,
    };

    // 4. Build Stage field view model
    let stage_options = vec![
        StageOptionViewModel {
            stage: ArticleStage::Pitching,
            label: "Pitching",
            short_label: "Pitch",
            icon_emoji: "💡",
            badge_color_hex: "#8B5CF6",
            description: "Initial story idea, pitch approval, and preliminary angle assessment.",
            is_selected: draft.stage == ArticleStage::Pitching,
            step_number: 1,
        },
        StageOptionViewModel {
            stage: ArticleStage::Researching,
            label: "Researching",
            short_label: "Research",
            icon_emoji: "🔍",
            badge_color_hex: "#3B82F6",
            description: "Source outreach, document requests, interviews, and fact gathering.",
            is_selected: draft.stage == ArticleStage::Researching,
            step_number: 2,
        },
        StageOptionViewModel {
            stage: ArticleStage::Writing,
            label: "Writing",
            short_label: "Drafting",
            icon_emoji: "✍️",
            badge_color_hex: "#EAB308",
            description: "Drafting story copy, structure, and supporting sidebars.",
            is_selected: draft.stage == ArticleStage::Writing,
            step_number: 3,
        },
        StageOptionViewModel {
            stage: ArticleStage::Editing,
            label: "Editing",
            short_label: "Editing",
            icon_emoji: "✂️",
            badge_color_hex: "#F97316",
            description: "Sub-editing, copy checking, legal verification, and headline polish.",
            is_selected: draft.stage == ArticleStage::Editing,
            step_number: 4,
        },
        StageOptionViewModel {
            stage: ArticleStage::ReadyToPublish,
            label: "Ready to Publish",
            short_label: "Ready",
            icon_emoji: "🚀",
            badge_color_hex: "#10B981",
            description: "Final proofs approved, layout positioned, staged for publication.",
            is_selected: draft.stage == ArticleStage::ReadyToPublish,
            step_number: 5,
        },
        StageOptionViewModel {
            stage: ArticleStage::Published,
            label: "Published",
            short_label: "Live",
            icon_emoji: "📰",
            badge_color_hex: "#6B7280",
            description: "Story live and in circulation. Overdue tracking automatically disabled.",
            is_selected: draft.stage == ArticleStage::Published,
            step_number: 6,
        },
    ];

    let selected_stage_opt = stage_options
        .iter()
        .find(|opt| opt.is_selected)
        .unwrap_or(&stage_options[0]);

    let stage_field = ArticleStageFieldViewModel {
        selected_stage: draft.stage,
        selected_label: selected_stage_opt.label,
        selected_icon: selected_stage_opt.icon_emoji,
        selected_badge_color_hex: selected_stage_opt.badge_color_hex,
        options: stage_options,
    };

    // 5. Build Deadline field view model
    let has_deadline = draft.deadline.is_some();
    let is_overdue = draft
        .deadline
        .map(|dl| now > dl && draft.stage != ArticleStage::Published)
        .unwrap_or(false);

    let (date_iso, time_hhmm, formatted_display, relative_hint) = if let Some(dl) = draft.deadline {
        let d_iso = dl.format("%Y-%m-%d").to_string();
        let t_hhmm = dl.format("%H:%M").to_string();
        let disp = dl.format("%b %d, %Y at %I:%M %p").to_string();

        let hint = if is_overdue {
            let diff = now.signed_duration_since(dl);
            let hours = diff.num_hours();
            if hours < 1 {
                "⚠️ Overdue just now".to_string()
            } else if hours < 24 {
                format!("⚠️ Overdue by {hours} hour(s)")
            } else {
                let days = diff.num_days();
                format!("⚠️ Overdue by {days} day(s)")
            }
        } else if draft.stage == ArticleStage::Published {
            "Story published".to_string()
        } else {
            let diff = dl.signed_duration_since(now);
            let hours = diff.num_hours();
            if hours < 24 && dl.date_naive() == now.date_naive() {
                format!("Due today at {}", dl.format("%I:%M %p"))
            } else if hours < 48 && dl.date_naive() == (now + Duration::days(1)).date_naive() {
                format!("Due tomorrow at {}", dl.format("%I:%M %p"))
            } else {
                let days = diff.num_days();
                format!("Due in {days} day(s)")
            }
        };

        (Some(d_iso), Some(t_hhmm), disp, hint)
    } else {
        (
            None,
            None,
            "No deadline set".to_string(),
            "No deadline scheduled for this story".to_string(),
        )
    };

    let presets = vec![
        DeadlinePresetViewModel {
            preset: DeadlinePreset::Today5PM,
            label: DeadlinePreset::Today5PM.label(),
            is_active: draft
                .deadline
                .map(|dl| dl.date_naive() == now.date_naive() && dl.hour() == DEFAULT_DEADLINE_HOUR)
                .unwrap_or(false),
        },
        DeadlinePresetViewModel {
            preset: DeadlinePreset::Tomorrow5PM,
            label: DeadlinePreset::Tomorrow5PM.label(),
            is_active: draft
                .deadline
                .map(|dl| {
                    dl.date_naive() == (now + Duration::days(1)).date_naive()
                        && dl.hour() == DEFAULT_DEADLINE_HOUR
                })
                .unwrap_or(false),
        },
        DeadlinePresetViewModel {
            preset: DeadlinePreset::EndOfWeek,
            label: DeadlinePreset::EndOfWeek.label(),
            is_active: false,
        },
        DeadlinePresetViewModel {
            preset: DeadlinePreset::NextWeek,
            label: DeadlinePreset::NextWeek.label(),
            is_active: false,
        },
        DeadlinePresetViewModel {
            preset: DeadlinePreset::InTwoWeeks,
            label: DeadlinePreset::InTwoWeeks.label(),
            is_active: false,
        },
        DeadlinePresetViewModel {
            preset: DeadlinePreset::Clear,
            label: DeadlinePreset::Clear.label(),
            is_active: draft.deadline.is_none(),
        },
    ];

    let deadline_field = ArticleDeadlineFieldViewModel {
        deadline: draft.deadline,
        has_deadline,
        date_iso,
        time_hhmm,
        formatted_display,
        relative_hint,
        is_overdue,
        presets,
    };

    // 6. Build Color Picker view model
    let auto_hash_hex = assign_color_for_slug(if clean_slug.is_empty() {
        "new-article"
    } else {
        clean_slug
    })
    .to_hex();

    let selected_hex = if draft.color_hex.trim().is_empty() {
        auto_hash_hex.clone()
    } else {
        draft.color_hex.clone()
    };

    let selected_color = Color::from_hex(&selected_hex).unwrap_or_default();
    let contrast_text_color = calculate_contrast_color(&selected_hex);

    let swatches: Vec<ColorSwatchViewModel> = CURATED_PALETTE
        .iter()
        .map(|nc| {
            let hex = nc.color.to_hex();
            let contrast = calculate_contrast_color(&hex);
            ColorSwatchViewModel {
                name: nc.name,
                hex: hex.clone(),
                rgb: nc.color.to_rgb(),
                contrast_text_color: contrast,
                is_selected: hex.eq_ignore_ascii_case(&selected_hex),
            }
        })
        .collect();

    let custom_hex_error = draft.validation_errors.get("color_hex").cloned();

    let color_picker = ArticleColorPickerViewModel {
        selected_hex,
        selected_rgb: selected_color.to_rgb(),
        contrast_text_color,
        auto_hash_hex,
        is_custom: draft.is_custom_color,
        swatches,
        custom_hex_error,
    };

    // 7. Overall error calculation and submission status
    let mut total_errors = draft.validation_errors.len();
    if is_collision && !draft.validation_errors.contains_key("slug") {
        total_errors += 1;
    }

    let is_submittable = total_errors == 0
        && !clean_slug.is_empty()
        && !draft.headline.trim().is_empty()
        && !is_collision;

    let error_summary = if total_errors > 0 {
        Some(format!("{total_errors} field(s) require your attention"))
    } else {
        None
    };

    ArticleFormViewModel {
        is_edit,
        article_id: draft.id,
        slug_field,
        headline_field,
        description_field,
        stage_field,
        deadline_field,
        color_picker,
        tagged_contacts_count: draft.tagged_contact_ids.len(),
        is_submittable,
        total_error_count: total_errors,
        error_summary,
    }
}
