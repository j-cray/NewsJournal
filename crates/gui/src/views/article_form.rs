//! Article creation and editing form field presentation models, validation state, and view builders.

use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc, Weekday};
use newsjournal_core::color::{assign_color_for_slug, Color, CURATED_PALETTE};
use newsjournal_core::models::{ArticleStage, Contact, Task, TaskStatus};
use newsjournal_core::validation::{slugify, validate_task_title, MAX_SLUG_LENGTH};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::modal::ArticleDraft;
use crate::state::AppState;
use crate::views::article_card::{calculate_contrast_color, format_contact_initials};

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

/// Presentation model for a single contact pill / chip in the Article Form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContactPillViewModel {
    /// Unique contact ID.
    pub id: Uuid,
    /// Full contact name (e.g. "Jane Doe").
    pub name: String,
    /// Organization or publication (e.g. "Daily News").
    pub organization: Option<String>,
    /// Professional role or beat (e.g. "City Hall Reporter").
    pub role: Option<String>,
    /// Formatted single-line label combining name and org/role (e.g. "Jane Doe (Daily News)").
    pub display_label: String,
    /// Short 1-2 letter uppercase initials for avatar badge (e.g. "JD").
    pub initials: String,
    /// Deterministic hex color for avatar badge background.
    pub avatar_color_hex: String,
    /// High-contrast text color on avatar badge (`#FFFFFF` or `#0F172A`).
    pub avatar_text_color: &'static str,
    /// Whether this contact is currently tagged in the article draft.
    pub is_tagged: bool,
    /// Email address if present.
    pub email: Option<String>,
    /// Phone number if present.
    pub phone: Option<String>,
}

impl ContactPillViewModel {
    /// Constructs a contact pill view model from a [`Contact`] and tagged status.
    #[must_use]
    pub fn new(contact: &Contact, is_tagged: bool) -> Self {
        let initials = format_contact_initials(&contact.name);
        let slug_key = slugify(&contact.name);
        let avatar_color = assign_color_for_slug(if slug_key.is_empty() {
            "contact"
        } else {
            &slug_key
        });
        let avatar_color_hex = avatar_color.to_hex();
        let avatar_text_color = calculate_contrast_color(&avatar_color_hex);

        let display_label = match (&contact.organization, &contact.role) {
            (Some(org), Some(role)) if !org.is_empty() && !role.is_empty() => {
                format!("{} ({} • {})", contact.name, org, role)
            }
            (Some(org), _) if !org.is_empty() => format!("{} ({})", contact.name, org),
            (_, Some(role)) if !role.is_empty() => format!("{} ({})", contact.name, role),
            _ => contact.name.clone(),
        };

        Self {
            id: contact.id,
            name: contact.name.clone(),
            organization: contact.organization.clone(),
            role: contact.role.clone(),
            display_label,
            initials,
            avatar_color_hex,
            avatar_text_color,
            is_tagged,
            email: contact.email.clone(),
            phone: contact.phone.clone(),
        }
    }
}

/// Presentation view model for the inline "Create Contact" sub-form within the article form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InlineContactFormViewModel {
    /// Contact full name input value.
    pub name_value: String,
    /// Contact organization / outlet input value.
    pub organization_value: String,
    /// Contact role / beat input value.
    pub role_value: String,
    /// Contact phone number input value.
    pub phone_value: String,
    /// Contact email address input value.
    pub email_value: String,
    /// Story / contact notes input value.
    pub notes_value: String,
    /// Name field validation error (if any).
    pub name_error: Option<String>,
    /// Email field validation error (if any).
    pub email_error: Option<String>,
    /// Phone field validation error (if any).
    pub phone_error: Option<String>,
    /// Whether the inline form is currently valid and ready to create.
    pub is_valid: bool,
}

/// Presentation view model for the Contact Tagging sub-section in the Article Form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContactTaggingSectionViewModel {
    /// Current search filter query entered by user.
    pub search_query: String,
    /// Whether a non-empty search query is active.
    pub is_search_active: bool,
    /// All contacts currently tagged in this article draft.
    pub tagged_contacts: Vec<ContactPillViewModel>,
    /// Contacts matching search query that are available to be tagged (not already tagged).
    pub available_contacts: Vec<ContactPillViewModel>,
    /// All contacts matching search query (with `is_tagged` set accordingly).
    pub filtered_contacts: Vec<ContactPillViewModel>,
    /// Total count of tagged contacts in this draft.
    pub total_tagged_count: usize,
    /// Total count of all contacts available in the system.
    pub total_system_contacts: usize,
    /// Count of contacts matching active search filter.
    pub matched_contacts_count: usize,
    /// Whether any contacts match the current search query.
    pub has_matches: bool,
    /// Whether the inline contact creation sub-form is currently expanded / open.
    pub is_inline_contact_open: bool,
    /// Inline contact creation sub-form model (if open).
    pub inline_form: Option<InlineContactFormViewModel>,
    /// Search input placeholder hint text.
    pub search_placeholder: &'static str,
    /// Empty state message when no contacts exist or match.
    pub empty_state_message: Option<String>,
}

/// Presentation view model for a single task item within the Article Form checklist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TaskChecklistItemViewModel {
    /// Unique task ID.
    pub id: Uuid,
    /// Task title.
    pub title: String,
    /// Detailed notes (if any).
    pub notes: Option<String>,
    /// Task workflow status.
    pub status: TaskStatus,
    /// Whether the task is marked as Complete.
    pub is_complete: bool,
    /// User-visible status label (e.g. "To-Do", "In Progress", "Complete").
    pub status_label: &'static str,
    /// Status badge color hex code.
    pub status_badge_color_hex: &'static str,
    /// Formatted due date string (if set).
    pub due_date_display: Option<String>,
    /// Whether the due date is in the past and task is incomplete.
    pub is_overdue: bool,
    /// Whether this task is currently staged in draft (not yet persisted in DB).
    pub is_staged: bool,
}

impl TaskChecklistItemViewModel {
    /// Constructs a task checklist item view model from a [`Task`], reference time, and staged flag.
    #[must_use]
    pub fn new(task: &Task, now: DateTime<Utc>, is_staged: bool) -> Self {
        let is_complete = task.status == TaskStatus::Complete;
        let is_overdue = !is_complete && task.due_date.map(|due| now > due).unwrap_or(false);

        let due_date_display = task.due_date.map(|due| due.format("%b %d, %Y").to_string());

        let (status_label, status_badge_color_hex) = match task.status {
            TaskStatus::ToDo => ("To-Do", "#64748B"),
            TaskStatus::InProgress => ("In Progress", "#3B82F6"),
            TaskStatus::Complete => ("Complete", "#10B981"),
        };

        Self {
            id: task.id,
            title: task.title.clone(),
            notes: task.notes.clone(),
            status: task.status,
            is_complete,
            status_label,
            status_badge_color_hex,
            due_date_display,
            is_overdue,
            is_staged,
        }
    }
}

/// Presentation view model for the Inline Task Management sub-section in the Article Form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArticleTasksSectionViewModel {
    /// List of all tasks for this article in display order.
    pub tasks: Vec<TaskChecklistItemViewModel>,
    /// Total count of tasks.
    pub total_count: usize,
    /// Count of completed tasks.
    pub completed_count: usize,
    /// Completion percentage (0 to 100).
    pub completion_percentage: u8,
    /// Human-friendly progress summary (e.g. "3 of 5 completed (60%)").
    pub progress_label: String,
    /// Current text in the quick-add task input field.
    pub quick_task_input: String,
    /// Placeholder hint text for quick-add input.
    pub quick_task_placeholder: &'static str,
    /// Validation error for the quick-task input field (if any).
    pub quick_task_error: Option<String>,
    /// Whether the quick-task input is non-empty and valid.
    pub is_quick_add_valid: bool,
    /// Empty state guidance message if no tasks exist.
    pub empty_state_message: Option<String>,
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
    /// Contact tagging sub-section presentation model.
    pub contacts_section: ContactTaggingSectionViewModel,
    /// Inline task management sub-section presentation model.
    pub tasks_section: ArticleTasksSectionViewModel,
    /// Number of contacts tagged in this draft.
    pub tagged_contacts_count: usize,
    /// Task completion counts `(completed_count, total_count)`.
    pub task_stats: (usize, usize),
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

    // 7. Build Contact Tagging sub-section view model
    let clean_query = draft.contact_search_query.trim().to_lowercase();
    let is_search_active = !clean_query.is_empty();

    let mut tagged_contacts = Vec::new();
    let mut available_contacts = Vec::new();
    let mut filtered_contacts = Vec::new();

    for contact in &state.contacts {
        let is_tagged = draft.is_contact_tagged(contact.id);
        let pill = ContactPillViewModel::new(contact, is_tagged);

        if is_tagged {
            tagged_contacts.push(pill.clone());
        }

        let matches_search = if is_search_active {
            contact.name.to_lowercase().contains(&clean_query)
                || contact
                    .organization
                    .as_ref()
                    .map(|s| s.to_lowercase().contains(&clean_query))
                    .unwrap_or(false)
                || contact
                    .role
                    .as_ref()
                    .map(|s| s.to_lowercase().contains(&clean_query))
                    .unwrap_or(false)
                || contact
                    .email
                    .as_ref()
                    .map(|s| s.to_lowercase().contains(&clean_query))
                    .unwrap_or(false)
        } else {
            true
        };

        if matches_search {
            filtered_contacts.push(pill.clone());
            if !is_tagged {
                available_contacts.push(pill);
            }
        }
    }

    let inline_form = draft.inline_contact.as_ref().map(|ic| {
        let name_error = ic.validation_errors.get("name").cloned();
        let email_error = ic.validation_errors.get("email").cloned();
        let phone_error = ic.validation_errors.get("phone").cloned();
        let is_valid = !ic.name.trim().is_empty()
            && name_error.is_none()
            && email_error.is_none()
            && phone_error.is_none();

        InlineContactFormViewModel {
            name_value: ic.name.clone(),
            organization_value: ic.organization.clone(),
            role_value: ic.role.clone(),
            phone_value: ic.phone.clone(),
            email_value: ic.email.clone(),
            notes_value: ic.notes.clone(),
            name_error,
            email_error,
            phone_error,
            is_valid,
        }
    });

    let total_system_contacts = state.contacts.len();
    let total_tagged_count = draft.tagged_contact_ids.len();
    let matched_contacts_count = filtered_contacts.len();
    let has_matches = matched_contacts_count > 0;

    let empty_state_message = if total_system_contacts == 0 {
        Some(
            "No contacts in directory yet. Use '+ Create Contact' to add sources for this story."
                .to_string(),
        )
    } else if is_search_active && !has_matches {
        Some(format!(
            "No contacts match '{clean_query}'. Click '+ Create Contact' to add them."
        ))
    } else {
        None
    };

    let contacts_section = ContactTaggingSectionViewModel {
        search_query: draft.contact_search_query.clone(),
        is_search_active,
        tagged_contacts,
        available_contacts,
        filtered_contacts,
        total_tagged_count,
        total_system_contacts,
        matched_contacts_count,
        has_matches,
        is_inline_contact_open: draft.is_inline_contact_open(),
        inline_form,
        search_placeholder: "Search contacts by name, outlet, or role...",
        empty_state_message,
    };

    // 8. Build Inline Task Management sub-section view model
    let mut all_tasks = Vec::new();

    // If editing an existing article, fetch tasks from state
    if let Some(article_id) = draft.id {
        for task in state.tasks_for_article(article_id) {
            all_tasks.push(TaskChecklistItemViewModel::new(task, now, false));
        }
    }

    // Add any staged tasks from the draft
    for staged_task in &draft.staged_tasks {
        all_tasks.push(TaskChecklistItemViewModel::new(staged_task, now, true));
    }

    let total_tasks_count = all_tasks.len();
    let completed_tasks_count = all_tasks.iter().filter(|t| t.is_complete).count();
    let completion_percentage = if total_tasks_count > 0 {
        ((completed_tasks_count as f32 / total_tasks_count as f32) * 100.0).round() as u8
    } else {
        0
    };

    let progress_label = if total_tasks_count == 0 {
        "No tasks yet".to_string()
    } else {
        format!(
            "{completed_tasks_count} of {total_tasks_count} completed ({completion_percentage}%)"
        )
    };

    let quick_input_clean = draft.quick_task_title.trim();
    let quick_task_error = if quick_input_clean.is_empty() {
        None
    } else {
        validate_task_title(quick_input_clean)
            .err()
            .map(|e| e.to_string())
    };
    let is_quick_add_valid = !quick_input_clean.is_empty() && quick_task_error.is_none();

    let empty_tasks_message = if total_tasks_count == 0 {
        Some("No tasks for this story yet. Type a task above to quick-add.".to_string())
    } else {
        None
    };

    let tasks_section = ArticleTasksSectionViewModel {
        tasks: all_tasks,
        total_count: total_tasks_count,
        completed_count: completed_tasks_count,
        completion_percentage,
        progress_label,
        quick_task_input: draft.quick_task_title.clone(),
        quick_task_placeholder:
            "Add a task for this story (e.g. Call city auditor, verify records)...",
        quick_task_error,
        is_quick_add_valid,
        empty_state_message: empty_tasks_message,
    };

    // 9. Overall error calculation and submission status
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
        contacts_section,
        tasks_section,
        tagged_contacts_count: draft.tagged_contact_ids.len(),
        task_stats: (completed_tasks_count, total_tasks_count),
        is_submittable,
        total_error_count: total_errors,
        error_summary,
    }
}
