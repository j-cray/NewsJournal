//! Task Card Component presentation models, left accent color strips, parent article badges, notes previews, due date alerts, and completion toggles.

use chrono::{DateTime, Utc};
use newsjournal_core::color::assign_color_for_slug;
use newsjournal_core::deadline::{
    format_compact_duration, format_verbose_duration, DeadlineStatus, UrgencyLevel,
};
use newsjournal_core::models::{Task, TaskStatus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::message::AppMessage;
use crate::state::AppState;
use crate::theme::ResolvedTheme;
use crate::views::article_card::calculate_contrast_color;

/// Default left accent strip width for task cards in logical pixels.
pub const DEFAULT_TASK_ACCENT_STRIP_WIDTH: f32 = 4.0;

/// Default corner radius for task cards in logical pixels.
pub const DEFAULT_TASK_CARD_CORNER_RADIUS: f32 = 8.0;

/// Default border width for normal task cards in logical pixels.
pub const DEFAULT_TASK_CARD_BORDER_WIDTH: f32 = 1.0;

/// Prominent bold border width for overdue task cards in logical pixels.
pub const OVERDUE_TASK_CARD_BORDER_WIDTH: f32 = 2.0;

/// Border width for task cards that are due soon in logical pixels.
pub const DUE_SOON_TASK_CARD_BORDER_WIDTH: f32 = 1.5;

/// Maximum length of task title preview snippet before truncation.
pub const MAX_TASK_TITLE_SNIPPET_LEN: usize = 90;

/// Maximum length of task notes preview snippet before truncation.
pub const MAX_TASK_NOTES_SNIPPET_LEN: usize = 140;

/// Overdue accent red hex color.
pub const TASK_OVERDUE_RED_HEX: &str = "#EF4444";

/// Overdue badge background tint hex.
pub const TASK_OVERDUE_BG_TINT_HEX: &str = "#7F1D1D";

/// Due soon accent amber hex color.
pub const TASK_DUE_SOON_AMBER_HEX: &str = "#F59E0B";

/// Due soon badge background tint hex.
pub const TASK_DUE_SOON_BG_TINT_HEX: &str = "#78350F";

/// Completed task green hex color.
pub const TASK_COMPLETE_GREEN_HEX: &str = "#10B981";

/// Completed badge background tint hex.
pub const TASK_COMPLETE_BG_TINT_HEX: &str = "#064E3B";

/// Left accent color strip view model matching the parent article's color.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskAccentStripViewModel {
    /// Assigned article hex color (e.g. `"#3B82F6"`).
    pub color_hex: String,
    /// High-contrast foreground color suitable for rendering over this color (`"#FFFFFF"` or `"#0F172A"`).
    pub contrast_fg_hex: String,
    /// Translucent background tint hex color for badge pill surfaces.
    pub translucent_tint_hex: String,
    /// Width of the accent strip in logical pixels.
    pub width_px: f32,
    /// Whether the task is assigned to a parent article.
    pub is_assigned: bool,
}

impl TaskAccentStripViewModel {
    /// Creates a new `TaskAccentStripViewModel` with default left strip geometry.
    #[must_use]
    pub fn new(color_hex: &str, is_assigned: bool) -> Self {
        let contrast_fg_hex = calculate_contrast_color(color_hex).to_string();
        let translucent_tint_hex = format!("{color_hex}26"); // ~15% alpha
        Self {
            color_hex: color_hex.to_string(),
            contrast_fg_hex,
            translucent_tint_hex,
            width_px: DEFAULT_TASK_ACCENT_STRIP_WIDTH,
            is_assigned,
        }
    }
}

/// Parent article slug badge/pill representation for the task card header.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskArticleSlugBadgeViewModel {
    /// Parent article unique ID.
    pub article_id: Uuid,
    /// Raw parent article slug.
    pub raw_slug: String,
    /// Formatted slug with `#` prefix (e.g. `"#transit-budget"`).
    pub display_text: String,
    /// Truncated display text for compact viewports.
    pub compact_text: String,
    /// Parent article hex color.
    pub color_hex: String,
    /// High-contrast foreground color.
    pub contrast_fg_hex: String,
    /// Translucent background tint hex.
    pub translucent_tint_hex: String,
    /// Whether this task has an assigned parent article.
    pub is_assigned: bool,
    /// Action dispatched when clicking this slug pill to filter or focus on the parent article.
    pub click_filter_action: Option<AppMessage>,
}

impl TaskArticleSlugBadgeViewModel {
    /// Constructs a new `TaskArticleSlugBadgeViewModel`.
    #[must_use]
    pub fn new(article_id: Uuid, slug: &str, color_hex: &str, is_assigned: bool) -> Self {
        let display_text = if is_assigned {
            format!("#{slug}")
        } else {
            "unassigned".to_string()
        };

        let compact_text = if slug.len() > 18 {
            format!("#{}…", &slug[..16])
        } else {
            display_text.clone()
        };

        let contrast_fg_hex = calculate_contrast_color(color_hex).to_string();
        let translucent_tint_hex = format!("{color_hex}26");

        let click_filter_action = if is_assigned {
            Some(AppMessage::SetArticleFilter(Some(article_id)))
        } else {
            None
        };

        Self {
            article_id,
            raw_slug: slug.to_string(),
            display_text,
            compact_text,
            color_hex: color_hex.to_string(),
            contrast_fg_hex,
            translucent_tint_hex,
            is_assigned,
            click_filter_action,
        }
    }
}

/// Formatted due date badge with relative time, compact countdowns, and urgency visual cues.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskDueDateBadgeViewModel {
    /// Target due date datetime (UTC).
    pub due_date: DateTime<Utc>,
    /// Formatted absolute date string (e.g. `"Aug 24, 2026"` or `"Tomorrow 5:00 PM"`).
    pub formatted_date: String,
    /// Formatted short calendar date (e.g. `"Aug 24"`).
    pub short_date: String,
    /// Human-readable relative time description (e.g. `"Overdue by 2h"`, `"Due in 3d"`, `"Due today"`).
    pub relative_text: String,
    /// Compact relative duration (e.g. `"-2h"`, `"+3d"`, `"0m"`).
    pub compact_duration: String,
    /// Calculated urgency level.
    pub urgency: UrgencyLevel,
    /// High-level badge label text (e.g. `"OVERDUE"`, `"DUE SOON"`, `"Aug 24"`).
    pub badge_label: String,
    /// Primary badge foreground color hex.
    pub badge_fg_hex: String,
    /// Badge background fill color hex.
    pub badge_bg_hex: String,
    /// Symbolic icon name.
    pub icon_name: String,
    /// macOS SF Symbol identifier.
    pub sf_symbol: String,
    /// Overdue flag.
    pub is_overdue: bool,
    /// Due soon flag.
    pub is_due_soon: bool,
    /// Completed flag.
    pub is_completed: bool,
}

/// Notes preview model extracting clean snippets, line counts, and expandability state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskNotesPreviewViewModel {
    /// Full raw notes text.
    pub raw_notes: String,
    /// Formatted single-line/multi-line clean snippet.
    pub snippet: String,
    /// True if the task has non-empty notes.
    pub has_notes: bool,
    /// Total number of lines in the notes.
    pub line_count: usize,
    /// True if the notes exceeded `MAX_TASK_NOTES_SNIPPET_LEN` and were truncated.
    pub is_truncated: bool,
}

impl TaskNotesPreviewViewModel {
    /// Constructs a new `TaskNotesPreviewViewModel`.
    #[must_use]
    pub fn new(notes: Option<&str>) -> Self {
        match notes {
            Some(raw) if !raw.trim().is_empty() => {
                let (snippet, is_truncated, line_count) =
                    format_task_notes_snippet(raw, MAX_TASK_NOTES_SNIPPET_LEN);
                Self {
                    raw_notes: raw.to_string(),
                    snippet,
                    has_notes: true,
                    line_count,
                    is_truncated,
                }
            }
            _ => Self {
                raw_notes: String::new(),
                snippet: String::new(),
                has_notes: false,
                line_count: 0,
                is_truncated: false,
            },
        }
    }
}

/// Interactive completion toggle checkbox presentation model for the task card.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskCheckboxViewModel {
    /// Unique task ID.
    pub task_id: Uuid,
    /// True if the task is completed (`TaskStatus::Complete`).
    pub is_checked: bool,
    /// Associated `TaskStatus`.
    pub status: TaskStatus,
    /// Icon name.
    pub icon_name: String,
    /// macOS SF Symbol identifier.
    pub sf_symbol: String,
    /// Accent color hex for the checkbox indicator.
    pub color_hex: String,
    /// Tooltip text.
    pub tooltip: String,
    /// Action dispatched when toggling the task checkbox.
    pub toggle_action: AppMessage,
}

impl TaskCheckboxViewModel {
    /// Constructs a `TaskCheckboxViewModel` for a task.
    #[must_use]
    pub fn new(task_id: Uuid, status: TaskStatus) -> Self {
        let is_checked = status.is_complete();
        let (icon_name, sf_symbol, color_hex, tooltip) = match status {
            TaskStatus::Complete => (
                "check-circle".to_string(),
                "checkmark.circle.fill".to_string(),
                TASK_COMPLETE_GREEN_HEX.to_string(),
                "Mark task as To-Do".to_string(),
            ),
            TaskStatus::InProgress => (
                "clock".to_string(),
                "hourglass".to_string(),
                "#3B82F6".to_string(),
                "Mark task as Complete".to_string(),
            ),
            TaskStatus::ToDo => (
                "circle".to_string(),
                "circle".to_string(),
                "#94A3B8".to_string(),
                "Mark task as Complete".to_string(),
            ),
        };

        Self {
            task_id,
            is_checked,
            status,
            icon_name,
            sf_symbol,
            color_hex,
            tooltip,
            toggle_action: AppMessage::ToggleTaskStatus(task_id),
        }
    }
}

/// Visual overdue and urgent styling properties for a task card.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskCardOverdueStyleViewModel {
    /// True if the task is currently overdue.
    pub is_overdue: bool,
    /// True if the task is due soon (within 24 hours).
    pub is_due_soon: bool,
    /// True if the task is complete.
    pub is_completed: bool,
    /// Card border highlight hex color.
    pub border_color_hex: String,
    /// Card border stroke width in logical pixels.
    pub border_width_px: f32,
    /// Card background tint hex (if highlighted).
    pub background_tint_hex: Option<String>,
    /// Alert badge text label (e.g. `"⚠️ Overdue"`).
    pub badge_text: Option<String>,
    /// Alert badge color hex.
    pub badge_color_hex: Option<String>,
    /// Whether an accent glow should be applied.
    pub accent_glow: bool,
}

impl TaskCardOverdueStyleViewModel {
    /// Computes the overdue visual style configuration.
    #[must_use]
    pub fn compute(
        is_overdue: bool,
        is_due_soon: bool,
        status: TaskStatus,
        parent_color: &str,
        is_dark: bool,
    ) -> Self {
        let is_completed = status.is_complete();

        if is_completed {
            Self {
                is_overdue: false,
                is_due_soon: false,
                is_completed: true,
                border_color_hex: if is_dark {
                    "#334155".to_string()
                } else {
                    "#E2E8F0".to_string()
                },
                border_width_px: DEFAULT_TASK_CARD_BORDER_WIDTH,
                background_tint_hex: None,
                badge_text: None,
                badge_color_hex: None,
                accent_glow: false,
            }
        } else if is_overdue {
            Self {
                is_overdue: true,
                is_due_soon: false,
                is_completed: false,
                border_color_hex: TASK_OVERDUE_RED_HEX.to_string(),
                border_width_px: OVERDUE_TASK_CARD_BORDER_WIDTH,
                background_tint_hex: Some(if is_dark {
                    TASK_OVERDUE_BG_TINT_HEX.to_string()
                } else {
                    "#FEE2E2".to_string()
                }),
                badge_text: Some("⚠️ Overdue".to_string()),
                badge_color_hex: Some(TASK_OVERDUE_RED_HEX.to_string()),
                accent_glow: true,
            }
        } else if is_due_soon {
            Self {
                is_overdue: false,
                is_due_soon: true,
                is_completed: false,
                border_color_hex: TASK_DUE_SOON_AMBER_HEX.to_string(),
                border_width_px: DUE_SOON_TASK_CARD_BORDER_WIDTH,
                background_tint_hex: Some(if is_dark {
                    TASK_DUE_SOON_BG_TINT_HEX.to_string()
                } else {
                    "#FEF3C7".to_string()
                }),
                badge_text: Some("🕒 Due Soon".to_string()),
                badge_color_hex: Some(TASK_DUE_SOON_AMBER_HEX.to_string()),
                accent_glow: false,
            }
        } else {
            Self {
                is_overdue: false,
                is_due_soon: false,
                is_completed: false,
                border_color_hex: if is_dark {
                    "#334155".to_string()
                } else {
                    "#E2E8F0".to_string()
                },
                border_width_px: DEFAULT_TASK_CARD_BORDER_WIDTH,
                background_tint_hex: Some(format!("{parent_color}08")), // subtle 3% tint
                badge_text: None,
                badge_color_hex: None,
                accent_glow: false,
            }
        }
    }
}

/// Formatted view model for a single Task card in the Tasks Kanban board.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskCardViewModel {
    /// Unique task ID.
    pub id: Uuid,
    /// Parent article ID.
    pub article_id: Uuid,
    /// Parent article slug.
    pub parent_article_slug: String,
    /// Parent article headline (if available).
    pub parent_article_headline: Option<String>,
    /// Parent article color hex.
    pub parent_article_color: String,
    /// Task title.
    pub title: String,
    /// Clean title snippet.
    pub title_snippet: String,
    /// Full raw task notes.
    pub notes: String,
    /// Notes preview presentation model.
    pub notes_preview: TaskNotesPreviewViewModel,
    /// Left accent color strip matching the parent article's color.
    pub accent_strip: TaskAccentStripViewModel,
    /// Parent article slug badge/pill.
    pub slug_badge: TaskArticleSlugBadgeViewModel,
    /// Optional due date.
    pub due_date: Option<DateTime<Utc>>,
    /// Formatted due date label (e.g. "Due in 3h", "Overdue (2h)").
    pub due_date_label: Option<String>,
    /// Rich due date badge presentation model.
    pub due_date_badge: Option<TaskDueDateBadgeViewModel>,
    /// Workflow status.
    pub status: TaskStatus,
    /// Checkbox toggle presentation model.
    pub checkbox: TaskCheckboxViewModel,
    /// Visual overdue and card styling properties.
    pub style: TaskCardOverdueStyleViewModel,
    /// True if this card is currently being dragged.
    pub is_dragging: bool,
    /// True if this task is overdue relative to the current timestamp.
    pub is_overdue: bool,
    /// True if this task is due soon.
    pub is_due_soon: bool,
    /// True if this task is completed.
    pub is_completed: bool,
    /// Overdue / deadline alert label.
    pub overdue_label: Option<String>,
    /// Action dispatched when clicking to open edit drawer.
    pub click_action: AppMessage,
    /// Action dispatched when toggling task completion checkbox.
    pub toggle_action: AppMessage,
    /// Action dispatched when clicking the article slug badge.
    pub filter_by_article_action: AppMessage,
}

impl TaskCardViewModel {
    /// Constructs a `TaskCardViewModel` from a domain task and app state.
    #[must_use]
    pub fn build(task: &Task, state: &AppState, is_dragging: bool) -> Self {
        let parent = state.get_article(task.article_id);
        let has_assigned_parent = parent.is_some();
        let slug = parent
            .map(|a| a.slug.clone())
            .unwrap_or_else(|| "unassigned".to_string());
        let headline = parent.map(|a| a.headline.clone());
        let color = parent
            .and_then(|a| a.color.clone())
            .unwrap_or_else(|| assign_color_for_slug(&slug).to_hex());

        let now = state.last_tick;
        let is_overdue = task.is_overdue(now);
        let is_due_soon = task.is_due_soon(now);
        let is_completed = task.status.is_complete();
        let is_dark = state.resolved_theme() == ResolvedTheme::Dark;

        let deadline_status = task.deadline_status(now);
        let due_date_label = match deadline_status {
            DeadlineStatus::NoDeadline => None,
            DeadlineStatus::Completed => None,
            other => Some(other.badge_text()),
        };

        let overdue_label = if is_overdue {
            Some("⚠️ Overdue".to_string())
        } else if is_due_soon {
            Some("🕒 Due Soon".to_string())
        } else {
            None
        };

        let title_snippet = format_task_title_snippet(&task.title, MAX_TASK_TITLE_SNIPPET_LEN);
        let notes_preview = TaskNotesPreviewViewModel::new(task.notes.as_deref());
        let accent_strip = TaskAccentStripViewModel::new(&color, has_assigned_parent);
        let slug_badge =
            TaskArticleSlugBadgeViewModel::new(task.article_id, &slug, &color, has_assigned_parent);
        let due_date_badge = format_task_due_date_badge(task.due_date, task.status, now, is_dark);
        let checkbox = TaskCheckboxViewModel::new(task.id, task.status);
        let style = TaskCardOverdueStyleViewModel::compute(
            is_overdue,
            is_due_soon,
            task.status,
            &color,
            is_dark,
        );

        Self {
            id: task.id,
            article_id: task.article_id,
            parent_article_slug: slug,
            parent_article_headline: headline,
            parent_article_color: color,
            title: task.title.clone(),
            title_snippet,
            notes: task.notes.clone().unwrap_or_default(),
            notes_preview,
            accent_strip,
            slug_badge,
            due_date: task.due_date,
            due_date_label,
            due_date_badge,
            status: task.status,
            checkbox,
            style,
            is_dragging,
            is_overdue,
            is_due_soon,
            is_completed,
            overdue_label,
            click_action: AppMessage::OpenEditTaskModal(task.id),
            toggle_action: AppMessage::ToggleTaskStatus(task.id),
            filter_by_article_action: AppMessage::SetArticleFilter(Some(task.article_id)),
        }
    }
}

/// Helper function to clean and format a task title preview snippet with truncation.
#[must_use]
pub fn format_task_title_snippet(title: &str, max_len: usize) -> String {
    let clean = title.trim().replace(['\n', '\r', '\t'], " ");
    if clean.chars().count() > max_len {
        let truncated: String = clean.chars().take(max_len.saturating_sub(1)).collect();
        format!("{}…", truncated.trim_end())
    } else {
        clean
    }
}

/// Helper function to clean and format task notes snippet with line counting and truncation.
#[must_use]
pub fn format_task_notes_snippet(notes: &str, max_len: usize) -> (String, bool, usize) {
    let lines: Vec<&str> = notes
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let line_count = lines.len();

    if lines.is_empty() {
        return (String::new(), false, 0);
    }

    let joined = lines.join(" • ");
    if joined.chars().count() > max_len {
        let truncated: String = joined.chars().take(max_len.saturating_sub(1)).collect();
        (format!("{}…", truncated.trim_end()), true, line_count)
    } else {
        (joined, false, line_count)
    }
}

/// Formats the due date badge view model based on status, timestamps, and theme.
#[must_use]
pub fn format_task_due_date_badge(
    due_date: Option<DateTime<Utc>>,
    status: TaskStatus,
    now: DateTime<Utc>,
    is_dark: bool,
) -> Option<TaskDueDateBadgeViewModel> {
    let dt = due_date?;

    if status.is_complete() {
        let formatted_date = dt.format("%b %-d").to_string();
        let short_date = dt.format("%b %-d").to_string();
        return Some(TaskDueDateBadgeViewModel {
            due_date: dt,
            formatted_date,
            short_date,
            relative_text: "Completed".to_string(),
            compact_duration: "done".to_string(),
            urgency: UrgencyLevel::None,
            badge_label: "Completed".to_string(),
            badge_fg_hex: TASK_COMPLETE_GREEN_HEX.to_string(),
            badge_bg_hex: if is_dark {
                TASK_COMPLETE_BG_TINT_HEX.to_string()
            } else {
                "#D1FAE5".to_string()
            },
            icon_name: "check-circle".to_string(),
            sf_symbol: "checkmark.circle.fill".to_string(),
            is_overdue: false,
            is_due_soon: false,
            is_completed: true,
        });
    }

    let formatted_date = dt.format("%b %-d, %Y %-I:%M %p").to_string();
    let short_date = dt.format("%b %-d").to_string();

    let diff = dt.signed_duration_since(now);
    let is_overdue = diff.num_seconds() < 0;
    let is_due_soon = !is_overdue && diff.num_hours() < 24;

    let relative_text = format_verbose_duration(diff);
    let compact_duration = format_compact_duration(diff);

    let (urgency, badge_label, badge_fg_hex, badge_bg_hex, icon_name, sf_symbol) = if is_overdue {
        (
            UrgencyLevel::Overdue,
            "OVERDUE".to_string(),
            TASK_OVERDUE_RED_HEX.to_string(),
            if is_dark {
                TASK_OVERDUE_BG_TINT_HEX.to_string()
            } else {
                "#FEE2E2".to_string()
            },
            "alert-triangle".to_string(),
            "exclamationmark.triangle.fill".to_string(),
        )
    } else if is_due_soon {
        (
            UrgencyLevel::High,
            "DUE SOON".to_string(),
            TASK_DUE_SOON_AMBER_HEX.to_string(),
            if is_dark {
                TASK_DUE_SOON_BG_TINT_HEX.to_string()
            } else {
                "#FEF3C7".to_string()
            },
            "clock".to_string(),
            "hourglass".to_string(),
        )
    } else {
        (
            UrgencyLevel::Low,
            short_date.clone(),
            if is_dark {
                "#94A3B8".to_string()
            } else {
                "#64748B".to_string()
            },
            if is_dark {
                "#1E293B".to_string()
            } else {
                "#F1F5F9".to_string()
            },
            "calendar".to_string(),
            "calendar".to_string(),
        )
    };

    Some(TaskDueDateBadgeViewModel {
        due_date: dt,
        formatted_date,
        short_date,
        relative_text,
        compact_duration,
        urgency,
        badge_label,
        badge_fg_hex,
        badge_bg_hex,
        icon_name,
        sf_symbol,
        is_overdue,
        is_due_soon,
        is_completed: false,
    })
}
