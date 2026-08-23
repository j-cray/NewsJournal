//! Task creation and editing form field presentation models, validation state, parent article picker, and view builders.

use chrono::{DateTime, Duration, Timelike, Utc};
use newsjournal_core::color::assign_color_for_slug;
use newsjournal_core::deadline::{format_compact_duration, format_verbose_duration, UrgencyLevel};
use newsjournal_core::models::{Article, TaskStatus};
use serde::Serialize;
use uuid::Uuid;

use crate::message::AppMessage;
use crate::state::modal::TaskDraft;
use crate::state::AppState;
use crate::theme::ResolvedTheme;
use crate::views::article_card::calculate_contrast_color;
use crate::views::article_form::{
    DeadlinePreset, DeadlinePresetViewModel, ValidationTooltipViewModel, DEFAULT_DEADLINE_HOUR,
};
use crate::views::task_card::format_task_title_snippet;
use crate::views::tasks::task_status_metadata;

/// Maximum length for a task title.
pub const MAX_TASK_TITLE_LENGTH: usize = 250;

/// Presentation model for a single selectable parent article option in the story picker dropdown.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskParentArticleOptionViewModel {
    /// Article unique ID.
    pub article_id: Uuid,
    /// Raw slug.
    pub slug: String,
    /// Formatted display slug (e.g. `"#transit-probe"`).
    pub display_slug: String,
    /// Full article headline.
    pub headline: String,
    /// Truncated preview headline.
    pub headline_snippet: String,
    /// Assigned accent hex color.
    pub color_hex: String,
    /// High contrast foreground text color.
    pub contrast_fg_hex: String,
    /// Translucent background tint hex color.
    pub translucent_tint_hex: String,
    /// Whether this option is currently selected.
    pub is_selected: bool,
    /// Total tasks currently linked to this article.
    pub total_tasks: usize,
    /// Completed tasks currently linked to this article.
    pub completed_tasks: usize,
}

impl TaskParentArticleOptionViewModel {
    /// Constructs a `TaskParentArticleOptionViewModel` from an Article and app state.
    #[must_use]
    pub fn build(article: &Article, is_selected: bool, state: &AppState) -> Self {
        let slug = article.slug.clone();
        let display_slug = format!("#{slug}");
        let headline = article.headline.clone();
        let headline_snippet = format_task_title_snippet(&headline, 50);

        let color_hex = article
            .color
            .clone()
            .unwrap_or_else(|| assign_color_for_slug(&slug).to_hex());
        let contrast_fg_hex = calculate_contrast_color(&color_hex).to_string();
        let translucent_tint_hex = format!("{color_hex}26");

        let (completed_tasks, total_tasks) = state.task_completion_stats(article.id);

        Self {
            article_id: article.id,
            slug,
            display_slug,
            headline,
            headline_snippet,
            color_hex,
            contrast_fg_hex,
            translucent_tint_hex,
            is_selected,
            total_tasks,
            completed_tasks,
        }
    }
}

/// Sizing and presentation view model for the Parent Article dropdown picker.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskParentArticlePickerViewModel {
    /// Currently selected article ID, if any.
    pub selected_article_id: Option<Uuid>,
    /// Currently selected article option view model (if resolved).
    pub selected_option: Option<TaskParentArticleOptionViewModel>,
    /// List of all available articles to select from.
    pub options: Vec<TaskParentArticleOptionViewModel>,
    /// Total count of available stories.
    pub total_stories_count: usize,
    /// Active validation error message (if any).
    pub error: Option<String>,
    /// Contextual validation tooltip (if any).
    pub tooltip: Option<ValidationTooltipViewModel>,
    /// Whether a parent article is currently selected.
    pub is_valid: bool,
    /// Placeholder guidance text.
    pub placeholder: &'static str,
}

/// Sizing and presentation view model for the Task Title input field.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskTitleFieldViewModel {
    /// Current title text.
    pub value: String,
    /// Placeholder hint.
    pub placeholder: &'static str,
    /// Maximum character limit (250).
    pub max_length: usize,
    /// Current character count.
    pub char_count: usize,
    /// Active validation error message (if any).
    pub error: Option<String>,
    /// Contextual validation error tooltip (if any).
    pub tooltip: Option<ValidationTooltipViewModel>,
    /// Whether the title is non-empty and passes validation.
    pub is_valid: bool,
}

/// Presentation model for a single workflow status option in the Task Status selector.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskStatusOptionViewModel {
    /// Associated `TaskStatus`.
    pub status: TaskStatus,
    /// Status title label (e.g. "To-Do", "In Progress", "Complete").
    pub label: &'static str,
    /// Canonical slug representation.
    pub slug: &'static str,
    /// Emoji icon.
    pub icon_emoji: &'static str,
    /// Standard symbolic icon name.
    pub icon_name: &'static str,
    /// macOS SF Symbol identifier.
    pub sf_symbol: &'static str,
    /// Accent hex color.
    pub accent_hex: &'static str,
    /// Short editorial description.
    pub description: &'static str,
    /// Whether this status is currently selected.
    pub is_selected: bool,
    /// Message dispatched when clicking this status option.
    pub select_action: AppMessage,
}

/// Presentation view model for the Task Status segmented switcher / selector.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskStatusPickerViewModel {
    /// Currently selected status.
    pub current_status: TaskStatus,
    /// Available status options in workflow sequence.
    pub options: Vec<TaskStatusOptionViewModel>,
}

/// Presentation view model for the Task Due Date field and quick presets.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskDueDateFieldViewModel {
    /// Target due date timestamp (UTC).
    pub due_date: Option<DateTime<Utc>>,
    /// True if a due date is specified.
    pub has_due_date: bool,
    /// Formatted absolute date string (e.g. "Aug 24, 2026 5:00 PM").
    pub formatted_date: String,
    /// Short calendar date string.
    pub short_date: String,
    /// Relative time description (e.g. "Overdue by 2h", "Due in 3d").
    pub relative_text: String,
    /// Compact relative duration (e.g. "-2h", "+3d").
    pub compact_duration: String,
    /// Calculated urgency level.
    pub urgency: UrgencyLevel,
    /// Overdue flag.
    pub is_overdue: bool,
    /// Due soon flag (within 24h).
    pub is_due_soon: bool,
    /// Completed flag.
    pub is_completed: bool,
    /// High-level badge label text (e.g. "OVERDUE", "DUE SOON", "No Due Date").
    pub badge_label: String,
    /// Badge foreground color hex.
    pub badge_fg_hex: String,
    /// Badge background fill color hex.
    pub badge_bg_hex: String,
    /// Quick deadline presets.
    pub presets: Vec<DeadlinePresetViewModel>,
    /// Action dispatched to clear the due date.
    pub clear_action: AppMessage,
}

/// Presentation view model for the multi-line Task Notes / References text area.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TaskNotesFieldViewModel {
    /// Current text content.
    pub value: String,
    /// Placeholder guidance text.
    pub placeholder: &'static str,
    /// Total character count.
    pub char_count: usize,
    /// Line count.
    pub line_count: usize,
}

/// Header presentation model for the Task drawer.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskFormHeaderViewModel {
    /// True if editing an existing task, false if creating a new task.
    pub is_edit: bool,
    /// Drawer headline title ("New Task" / "Edit Task").
    pub title: &'static str,
    /// Explanatory contextual subtitle.
    pub subtitle: &'static str,
    /// Header emoji icon.
    pub icon_emoji: &'static str,
    /// Standard symbolic icon name.
    pub icon_name: &'static str,
    /// macOS SF Symbol identifier.
    pub sf_symbol: &'static str,
    /// Current workflow status title.
    pub status_label: &'static str,
    /// Current workflow status accent color.
    pub status_color_hex: &'static str,
    /// Parent article slug if selected.
    pub parent_slug: Option<String>,
    /// Parent article color hex if selected.
    pub parent_color_hex: Option<String>,
    /// True if the delete action is available (editing existing task).
    pub can_delete: bool,
    /// Action dispatched when clicking the delete task button.
    pub delete_action: Option<AppMessage>,
}

/// Unified presentation view model for the complete Task Creation & Editing Drawer.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskFormViewModel {
    /// Task unique ID if editing, or `None` if creating.
    pub id: Option<Uuid>,
    /// True if editing an existing task.
    pub is_edit: bool,
    /// Header presentation view model.
    pub header: TaskFormHeaderViewModel,
    /// Parent article selector view model.
    pub article_picker: TaskParentArticlePickerViewModel,
    /// Task title field view model.
    pub title_field: TaskTitleFieldViewModel,
    /// Status picker segmented switcher view model.
    pub status_picker: TaskStatusPickerViewModel,
    /// Due date picker and preset shortcuts view model.
    pub due_date_field: TaskDueDateFieldViewModel,
    /// Notes / reference details text area view model.
    pub notes_field: TaskNotesFieldViewModel,
    /// All active field validation error tooltips.
    pub validation_tooltips: Vec<ValidationTooltipViewModel>,
    /// Whether the form is currently valid and ready for submission.
    pub can_save: bool,
    /// Primary submit button label ("Create Task" or "Save Changes").
    pub save_button_label: &'static str,
    /// Keyboard shortcut string for saving ("⌘S" or "Ctrl+S").
    pub save_shortcut: &'static str,
    /// Keyboard shortcut string for canceling ("Esc").
    pub cancel_shortcut: &'static str,
}

/// Constructs the complete `TaskFormViewModel` from application state and active `TaskDraft`.
#[must_use]
pub fn build_task_form_view(state: &AppState, draft: &TaskDraft) -> TaskFormViewModel {
    build_task_form_view_with_layout(state, draft, cfg!(target_os = "macos"))
}

/// Constructs the `TaskFormViewModel` with explicit macOS platform flag.
#[must_use]
pub fn build_task_form_view_with_layout(
    state: &AppState,
    draft: &TaskDraft,
    is_macos: bool,
) -> TaskFormViewModel {
    let is_edit = draft.id.is_some();
    let is_dark = state.resolved_theme() == ResolvedTheme::Dark;
    let now = state.last_tick;

    // 1. Build Parent Article Picker
    let mut options = Vec::new();
    let mut selected_option = None;

    for article in &state.articles {
        let is_sel = draft.article_id == Some(article.id);
        let opt = TaskParentArticleOptionViewModel::build(article, is_sel, state);
        if is_sel {
            selected_option = Some(opt.clone());
        }
        options.push(opt);
    }

    let article_err = draft.validation_errors.get("article_id").cloned();
    let article_tooltip = article_err
        .as_ref()
        .map(|msg| ValidationTooltipViewModel::error("article_id", msg.clone()));

    let article_picker = TaskParentArticlePickerViewModel {
        selected_article_id: draft.article_id,
        selected_option,
        total_stories_count: state.articles.len(),
        error: article_err,
        tooltip: article_tooltip,
        is_valid: draft.article_id.is_some(),
        placeholder: "Select parent article...",
        options,
    };

    // 2. Build Title Field
    let title_clean = draft.title.trim();
    let title_err = draft.validation_errors.get("title").cloned();
    let title_tooltip = title_err
        .as_ref()
        .map(|msg| ValidationTooltipViewModel::error("title", msg.clone()));
    let title_is_valid = !title_clean.is_empty() && title_err.is_none();

    let title_field = TaskTitleFieldViewModel {
        value: draft.title.clone(),
        placeholder: "e.g. Call whistleblower to confirm documents…",
        max_length: MAX_TASK_TITLE_LENGTH,
        char_count: draft.title.chars().count(),
        error: title_err,
        tooltip: title_tooltip,
        is_valid: title_is_valid,
    };

    // 3. Build Status Picker
    let status_options = TaskStatus::all()
        .iter()
        .map(|&st| {
            let meta = task_status_metadata(st);
            TaskStatusOptionViewModel {
                status: st,
                label: meta.title,
                slug: meta.slug,
                icon_emoji: meta.icon_emoji,
                icon_name: meta.icon_name,
                sf_symbol: meta.sf_symbol,
                accent_hex: meta.accent_hex,
                description: meta.description,
                is_selected: draft.status == st,
                select_action: AppMessage::SetTaskDraftStatus(st),
            }
        })
        .collect();

    let status_picker = TaskStatusPickerViewModel {
        current_status: draft.status,
        options: status_options,
    };

    // 4. Build Due Date Field & Presets
    let is_completed = draft.status.is_complete();
    let presets = vec![
        DeadlinePresetViewModel {
            preset: DeadlinePreset::Today5PM,
            label: DeadlinePreset::Today5PM.label(),
            is_active: draft
                .due_date
                .map(|dl| dl.date_naive() == now.date_naive() && dl.hour() == DEFAULT_DEADLINE_HOUR)
                .unwrap_or(false),
        },
        DeadlinePresetViewModel {
            preset: DeadlinePreset::Tomorrow5PM,
            label: DeadlinePreset::Tomorrow5PM.label(),
            is_active: draft
                .due_date
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
            is_active: draft.due_date.is_none(),
        },
    ];

    let due_date_field = match draft.due_date {
        Some(due) => {
            let formatted_date = due.format("%b %-d, %Y %-I:%M %p").to_string();
            let short_date = due.format("%b %-d").to_string();
            let diff = due.signed_duration_since(now);
            let total_secs = diff.num_seconds();
            let is_overdue = total_secs < 0 && !is_completed;
            let is_due_soon = (0..=86400).contains(&total_secs) && !is_completed;

            let relative_text = if is_completed {
                "Completed".to_string()
            } else {
                format_verbose_duration(diff)
            };

            let compact_duration = if is_completed {
                "Done".to_string()
            } else {
                format_compact_duration(diff)
            };

            let (badge_label, badge_fg_hex, badge_bg_hex, urgency) = if is_completed {
                (
                    "COMPLETED".to_string(),
                    "#10B981".to_string(),
                    if is_dark { "#064E3B" } else { "#D1FAE5" }.to_string(),
                    UrgencyLevel::None,
                )
            } else if is_overdue {
                (
                    "OVERDUE".to_string(),
                    "#EF4444".to_string(),
                    if is_dark { "#7F1D1D" } else { "#FEE2E2" }.to_string(),
                    UrgencyLevel::Overdue,
                )
            } else if is_due_soon {
                (
                    "DUE SOON".to_string(),
                    "#F59E0B".to_string(),
                    if is_dark { "#78350F" } else { "#FEF3C7" }.to_string(),
                    UrgencyLevel::High,
                )
            } else {
                (
                    short_date.clone(),
                    if is_dark { "#94A3B8" } else { "#64748B" }.to_string(),
                    if is_dark { "#1E293B" } else { "#F1F5F9" }.to_string(),
                    UrgencyLevel::Low,
                )
            };

            TaskDueDateFieldViewModel {
                due_date: Some(due),
                has_due_date: true,
                formatted_date,
                short_date,
                relative_text,
                compact_duration,
                urgency,
                is_overdue,
                is_due_soon,
                is_completed,
                badge_label,
                badge_fg_hex,
                badge_bg_hex,
                presets,
                clear_action: AppMessage::ClearTaskDraftDueDate,
            }
        }
        None => TaskDueDateFieldViewModel {
            due_date: None,
            has_due_date: false,
            formatted_date: "No due date set".to_string(),
            short_date: "-".to_string(),
            relative_text: "No deadline".to_string(),
            compact_duration: "-".to_string(),
            urgency: UrgencyLevel::None,
            is_overdue: false,
            is_due_soon: false,
            is_completed: false,
            badge_label: "NO DUE DATE".to_string(),
            badge_fg_hex: if is_dark { "#94A3B8" } else { "#64748B" }.to_string(),
            badge_bg_hex: if is_dark { "#1E293B" } else { "#F1F5F9" }.to_string(),
            presets,
            clear_action: AppMessage::ClearTaskDraftDueDate,
        },
    };

    // 5. Build Notes Field
    let notes_line_count = draft
        .notes
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count()
        .max(1);

    let notes_field = TaskNotesFieldViewModel {
        value: draft.notes.clone(),
        placeholder:
            "Add background notes, interview questions, sources to call, or fact-checking links…",
        char_count: draft.notes.chars().count(),
        line_count: notes_line_count,
    };

    // 6. Build Header
    let status_meta = task_status_metadata(draft.status);
    let parent_article = draft.article_id.and_then(|aid| state.get_article(aid));
    let parent_slug = parent_article.map(|a| a.slug.clone());
    let parent_color_hex = parent_article.map(|a| {
        a.color
            .clone()
            .unwrap_or_else(|| assign_color_for_slug(&a.slug).to_hex())
    });

    let delete_action = draft.id.map(AppMessage::PromptDeleteTask);

    let header = TaskFormHeaderViewModel {
        is_edit,
        title: if is_edit { "Edit Task" } else { "New Task" },
        subtitle: if is_edit {
            "Update reporting task details, status, and deadlines"
        } else {
            "Create a new reporting task linked to a story"
        },
        icon_emoji: "✅",
        icon_name: "checkbox",
        sf_symbol: "checkmark.circle",
        status_label: status_meta.title,
        status_color_hex: status_meta.accent_hex,
        parent_slug,
        parent_color_hex,
        can_delete: is_edit,
        delete_action,
    };

    // 7. Validation Tooltips and Can Save
    let mut validation_tooltips = Vec::new();
    for (field, msg) in &draft.validation_errors {
        let field_static: &'static str = match field.as_str() {
            "title" => "title",
            "article_id" => "article_id",
            _ => "general",
        };
        validation_tooltips.push(ValidationTooltipViewModel::error(field_static, msg.clone()));
    }

    let is_empty_required = draft.title.trim().is_empty() || draft.article_id.is_none();
    let can_save = draft.validation_errors.is_empty() && !is_empty_required;

    let save_button_label = if is_edit {
        "Save Changes"
    } else {
        "Create Task"
    };

    let save_shortcut = if is_macos { "⌘S" } else { "Ctrl+S" };
    let cancel_shortcut = "Esc";

    TaskFormViewModel {
        id: draft.id,
        is_edit,
        header,
        article_picker,
        title_field,
        status_picker,
        due_date_field,
        notes_field,
        validation_tooltips,
        can_save,
        save_button_label,
        save_shortcut,
        cancel_shortcut,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::modal::TaskDraft;
    use newsjournal_core::models::{Article, Task, TaskStatus};

    #[test]
    fn test_task_form_view_model_new_and_edit() {
        let mut state = AppState::in_memory().expect("in-memory state");
        let article = Article::new("city-transit", "City Transit Investigation");
        let aid = article.id;
        state.articles.push(article);

        // New Task Draft
        let mut draft = TaskDraft::new_for_article(Some(aid));
        let view_new = build_task_form_view(&state, &draft);
        assert!(!view_new.is_edit);
        assert_eq!(view_new.header.title, "New Task");
        assert_eq!(view_new.save_button_label, "Create Task");
        assert_eq!(view_new.article_picker.options.len(), 1);
        assert_eq!(view_new.article_picker.selected_article_id, Some(aid));
        assert!(!view_new.can_save); // empty title

        // Fill Title
        draft.set_title("Interview union rep");
        let view_valid = build_task_form_view(&state, &draft);
        assert!(view_valid.can_save);
        assert!(view_valid.title_field.is_valid);
        assert_eq!(view_valid.title_field.value, "Interview union rep");

        // Existing Task Draft (Edit)
        let task = Task::new(aid, "Verify maintenance logs")
            .with_status(TaskStatus::InProgress)
            .with_notes("Check 2025 inspection records");
        let edit_draft = TaskDraft::from_task(&task);
        let view_edit = build_task_form_view(&state, &edit_draft);
        assert!(view_edit.is_edit);
        assert_eq!(view_edit.header.title, "Edit Task");
        assert_eq!(view_edit.save_button_label, "Save Changes");
        assert_eq!(
            view_edit.status_picker.current_status,
            TaskStatus::InProgress
        );
        assert_eq!(view_edit.notes_field.value, "Check 2025 inspection records");
        assert!(view_edit.header.can_delete);
        assert!(view_edit.header.delete_action.is_some());
    }

    #[test]
    fn test_task_parent_article_picker_resolution() {
        let mut state = AppState::in_memory().expect("in-memory state");
        let a1 = Article::new("story-one", "Story One Headline");
        let a2 = Article::new("story-two", "Story Two Headline");
        let a2_id = a2.id;
        state.articles.push(a1);
        state.articles.push(a2);

        let draft = TaskDraft::new_for_article(Some(a2_id));
        let view = build_task_form_view(&state, &draft);

        assert_eq!(view.article_picker.options.len(), 2);
        assert!(view.article_picker.is_valid);
        let selected = view.article_picker.selected_option.expect("selected opt");
        assert_eq!(selected.article_id, a2_id);
        assert_eq!(selected.slug, "story-two");
        assert_eq!(selected.display_slug, "#story-two");
    }

    #[test]
    fn test_task_status_picker_options() {
        let state = AppState::in_memory().expect("in-memory state");
        let draft = TaskDraft::new_for_status(TaskStatus::Complete, None);
        let view = build_task_form_view(&state, &draft);

        assert_eq!(view.status_picker.options.len(), 3);
        let complete_opt = view
            .status_picker
            .options
            .iter()
            .find(|o| o.status == TaskStatus::Complete)
            .expect("complete option");
        assert!(complete_opt.is_selected);
        assert_eq!(complete_opt.label, "Complete");
        assert_eq!(complete_opt.icon_emoji, "✅");
    }
}
