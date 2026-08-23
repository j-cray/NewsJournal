//! Tasks Kanban board view models, column metadata, and horizontal 3-column deck layout engine.

use newsjournal_core::color::assign_color_for_slug;
use newsjournal_core::models::TaskStatus;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::message::AppMessage;
use crate::state::drag_drop::{DragItem, DropTarget};
use crate::state::filters::UrgencyFilter;
use crate::state::AppState;

/// Default nominal column width in logical pixels for the 3-column Tasks board.
pub const DEFAULT_TASK_COLUMN_WIDTH: f32 = 360.0;

/// Minimum allowed column width in logical pixels.
pub const MIN_TASK_COLUMN_WIDTH: f32 = 280.0;

/// Maximum allowed column width in logical pixels.
pub const MAX_TASK_COLUMN_WIDTH: f32 = 520.0;

/// Default gap between adjacent columns in logical pixels.
pub const DEFAULT_TASK_COLUMN_GAP: f32 = 18.0;

/// Default horizontal padding on left and right deck edges in logical pixels.
pub const DEFAULT_TASK_DECK_PADDING: f32 = 20.0;

/// Total number of task workflow status columns on the Tasks Kanban board.
pub const NUM_TASK_STATUSES: usize = 3;

/// Default left accent strip width for task cards matching parent article color.
pub const TASK_ACCENT_STRIP_WIDTH: f32 = 4.0;

/// Overdue accent border highlight color (Red).
pub const TASK_OVERDUE_RED_HEX: &str = "#EF4444";

/// Due soon warning accent color (Amber).
pub const TASK_DUE_SOON_AMBER_HEX: &str = "#F59E0B";

/// Completed indicator color (Emerald Green).
pub const TASK_COMPLETE_GREEN_HEX: &str = "#10B981";

/// Canonical workflow and visual metadata for a Task status column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TaskStatusMetadata {
    /// Associated `TaskStatus`.
    pub status: TaskStatus,
    /// 0-based workflow sequence index (0..=2).
    pub index: usize,
    /// Human-readable title.
    pub title: &'static str,
    /// Canonical slug representation.
    pub slug: &'static str,
    /// Workflow description or editorial guidance.
    pub description: &'static str,
    /// Fallback emoji representation.
    pub icon_emoji: &'static str,
    /// Standard symbolic icon name.
    pub icon_name: &'static str,
    /// macOS SF Symbol identifier.
    pub sf_symbol: &'static str,
    /// Stage accent hex color for column headers and badges.
    pub accent_hex: &'static str,
    /// Empty state guidance prompt.
    pub empty_state_prompt: &'static str,
}

/// Returns canonical metadata for a given task workflow status.
#[must_use]
pub const fn task_status_metadata(status: TaskStatus) -> TaskStatusMetadata {
    match status {
        TaskStatus::ToDo => TaskStatusMetadata {
            status: TaskStatus::ToDo,
            index: 0,
            title: "To-Do",
            slug: "to_do",
            description: "Queued and unstarted action items",
            icon_emoji: "📋",
            icon_name: "circle",
            sf_symbol: "circle",
            accent_hex: "#F59E0B",
            empty_state_prompt:
                "No to-do tasks queued. Click \"+ Add Task\" or create tasks from articles.",
        },
        TaskStatus::InProgress => TaskStatusMetadata {
            status: TaskStatus::InProgress,
            index: 1,
            title: "In Progress",
            slug: "in_progress",
            description: "Active reporting, interviews & research in flight",
            icon_emoji: "⏳",
            icon_name: "clock",
            sf_symbol: "hourglass",
            accent_hex: "#3B82F6",
            empty_state_prompt:
                "No tasks actively in progress. Drag tasks here when you begin work.",
        },
        TaskStatus::Complete => TaskStatusMetadata {
            status: TaskStatus::Complete,
            index: 2,
            title: "Complete",
            slug: "complete",
            description: "Finished reporting items & verified milestones",
            icon_emoji: "✅",
            icon_name: "check-circle",
            sf_symbol: "checkmark.circle",
            accent_hex: "#10B981",
            empty_state_prompt:
                "No completed tasks yet. Drag finished items here to close them out.",
        },
    }
}

/// Layout geometry, column dimensions, and horizontal scrolling state for the 3-column Tasks Kanban deck.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TasksDeckLayoutConfig {
    /// Nominal column width in logical pixels.
    pub column_width: f32,
    /// Gap between adjacent columns in logical pixels.
    pub column_gap: f32,
    /// Horizontal padding on left and right deck edges in logical pixels.
    pub deck_padding: f32,
    /// Current horizontal scroll offset in logical pixels.
    pub scroll_offset_x: f32,
    /// Current viewport container width in logical pixels.
    pub viewport_width: f32,
    /// Minimum allowed column width.
    pub min_column_width: f32,
    /// Maximum allowed column width.
    pub max_column_width: f32,
}

impl Default for TasksDeckLayoutConfig {
    fn default() -> Self {
        Self {
            column_width: DEFAULT_TASK_COLUMN_WIDTH,
            column_gap: DEFAULT_TASK_COLUMN_GAP,
            deck_padding: DEFAULT_TASK_DECK_PADDING,
            scroll_offset_x: 0.0,
            viewport_width: 1280.0,
            min_column_width: MIN_TASK_COLUMN_WIDTH,
            max_column_width: MAX_TASK_COLUMN_WIDTH,
        }
    }
}

impl TasksDeckLayoutConfig {
    /// Creates a new `TasksDeckLayoutConfig` with the given viewport width.
    #[must_use]
    pub fn new(viewport_width: f32) -> Self {
        Self {
            viewport_width: viewport_width.max(100.0),
            ..Self::default()
        }
    }

    /// Sets the horizontal scroll offset.
    #[must_use]
    pub fn with_scroll(mut self, scroll_offset_x: f32) -> Self {
        self.scroll_offset_x = self.clamped_scroll_offset(scroll_offset_x);
        self
    }

    /// Sets the column width, clamped to min/max bounds.
    #[must_use]
    pub fn with_column_width(mut self, width: f32) -> Self {
        self.column_width = width.clamp(self.min_column_width, self.max_column_width);
        self
    }

    /// Sets the viewport width and re-clamps scroll offset.
    pub fn set_viewport_width(&mut self, width: f32) {
        self.viewport_width = width.max(100.0);
        self.scroll_offset_x = self.clamped_scroll_offset(self.scroll_offset_x);
    }

    /// Sets the column width and re-clamps.
    pub fn set_column_width(&mut self, width: f32) {
        self.column_width = width.clamp(self.min_column_width, self.max_column_width);
        self.scroll_offset_x = self.clamped_scroll_offset(self.scroll_offset_x);
    }

    /// Computes the total content width required to display all 3 columns, gaps, and deck padding.
    #[must_use]
    pub fn total_content_width(&self) -> f32 {
        let cols_total = (NUM_TASK_STATUSES as f32) * self.column_width;
        let gaps_total = ((NUM_TASK_STATUSES - 1) as f32) * self.column_gap;
        cols_total + gaps_total + (2.0 * self.deck_padding)
    }

    /// Computes the maximum valid horizontal scroll offset.
    #[must_use]
    pub fn max_scroll_offset(&self) -> f32 {
        (self.total_content_width() - self.viewport_width).max(0.0)
    }

    /// Clamps an arbitrary scroll offset into the valid `[0.0, max_scroll_offset]` range.
    #[must_use]
    pub fn clamped_scroll_offset(&self, offset: f32) -> f32 {
        offset.clamp(0.0, self.max_scroll_offset())
    }

    /// Returns `true` if the deck can be scrolled to the left.
    #[must_use]
    pub fn can_scroll_left(&self) -> bool {
        self.scroll_offset_x > 0.5
    }

    /// Returns `true` if the deck can be scrolled to the right.
    #[must_use]
    pub fn can_scroll_right(&self) -> bool {
        self.scroll_offset_x < (self.max_scroll_offset() - 0.5)
    }

    /// Returns the scroll progress ratio from `0.0` (start) to `1.0` (end).
    #[must_use]
    pub fn scroll_progress(&self) -> f32 {
        let max = self.max_scroll_offset();
        if max <= 0.0 {
            0.0
        } else {
            (self.scroll_offset_x / max).clamp(0.0, 1.0)
        }
    }

    /// Computes the absolute X position of a column relative to the deck content root.
    #[must_use]
    pub fn column_x_position(&self, column_index: usize) -> f32 {
        let index = column_index.min(NUM_TASK_STATUSES - 1);
        self.deck_padding + (index as f32) * (self.column_width + self.column_gap)
    }

    /// Computes the screen/viewport X coordinate for a column (taking scroll offset into account).
    #[must_use]
    pub fn column_screen_x(&self, column_index: usize) -> f32 {
        self.column_x_position(column_index) - self.scroll_offset_x
    }

    /// Computes the screen X bounds `(left, right)` for a column.
    #[must_use]
    pub fn column_screen_bounds(&self, column_index: usize) -> (f32, f32) {
        let left = self.column_screen_x(column_index);
        (left, left + self.column_width)
    }

    /// Checks if a column is at least partially visible in the viewport.
    #[must_use]
    pub fn is_column_visible(&self, column_index: usize) -> bool {
        let (left, right) = self.column_screen_bounds(column_index);
        right > 0.0 && left < self.viewport_width
    }

    /// Checks if a column is fully visible without clipping in the viewport.
    #[must_use]
    pub fn is_column_fully_visible(&self, column_index: usize) -> bool {
        let (left, right) = self.column_screen_bounds(column_index);
        left >= 0.0 && right <= self.viewport_width
    }

    /// Returns the indices of all columns currently visible in the viewport.
    #[must_use]
    pub fn visible_column_indices(&self) -> Vec<usize> {
        (0..NUM_TASK_STATUSES)
            .filter(|&i| self.is_column_visible(i))
            .collect()
    }

    /// Calculates the optimal scroll offset to bring the specified column fully into view.
    #[must_use]
    pub fn scroll_offset_for_column(&self, column_index: usize) -> f32 {
        let col_x = self.column_x_position(column_index);
        let col_right = col_x + self.column_width;

        let screen_left = col_x - self.scroll_offset_x;
        let screen_right = col_right - self.scroll_offset_x;

        if screen_left < 0.0 {
            self.clamped_scroll_offset(col_x - self.deck_padding)
        } else if screen_right > self.viewport_width {
            self.clamped_scroll_offset(col_right + self.deck_padding - self.viewport_width)
        } else {
            self.scroll_offset_x
        }
    }

    /// Calculates the optimal scroll offset to bring a specific status column into view.
    #[must_use]
    pub fn scroll_offset_for_status(&self, status: TaskStatus) -> f32 {
        let meta = task_status_metadata(status);
        self.scroll_offset_for_column(meta.index)
    }

    /// Scrolls horizontally by the given delta.
    pub fn scroll_by(&mut self, delta_x: f32) {
        self.scroll_offset_x = self.clamped_scroll_offset(self.scroll_offset_x + delta_x);
    }

    /// Scrolls directly to the specified offset.
    pub fn scroll_to(&mut self, offset: f32) {
        self.scroll_offset_x = self.clamped_scroll_offset(offset);
    }

    /// Scrolls horizontally to bring a status column into view.
    pub fn scroll_to_status(&mut self, status: TaskStatus) {
        self.scroll_offset_x = self.scroll_offset_for_status(status);
    }

    /// Scrolls horizontally to bring a column index into view.
    pub fn scroll_to_column(&mut self, column_index: usize) {
        self.scroll_offset_x = self.scroll_offset_for_column(column_index);
    }

    /// Scrolls one viewport page to the left.
    pub fn scroll_page_left(&mut self) {
        let step = (self.viewport_width - self.column_width).max(self.column_width);
        self.scroll_by(-step);
    }

    /// Scrolls one viewport page to the right.
    pub fn scroll_page_right(&mut self) {
        let step = (self.viewport_width - self.column_width).max(self.column_width);
        self.scroll_by(step);
    }

    /// Computes which column index (0..2) is located at the given screen X coordinate.
    #[must_use]
    pub fn column_index_at_screen_x(&self, screen_x: f32) -> Option<usize> {
        for i in 0..NUM_TASK_STATUSES {
            let (left, right) = self.column_screen_bounds(i);
            if screen_x >= left && screen_x <= right {
                return Some(i);
            }
        }
        None
    }

    /// Computes which `TaskStatus` is located at the given screen X coordinate.
    #[must_use]
    pub fn status_at_screen_x(&self, screen_x: f32) -> Option<TaskStatus> {
        let idx = self.column_index_at_screen_x(screen_x)?;
        TaskStatus::all().get(idx).copied()
    }
}

pub use crate::views::task_card::TaskCardViewModel;

/// Floating visual drag ghost following cursor during a task drag-and-drop session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskDragGhostViewModel {
    /// Unique task ID.
    pub id: Uuid,
    /// Task title.
    pub title: String,
    /// Parent article slug.
    pub parent_article_slug: String,
    /// Parent article color hex.
    pub parent_article_color: String,
    /// Initial origin status.
    pub origin_status: TaskStatus,
    /// Current pointer coordinates `(x, y)`.
    pub pointer_pos: (f32, f32),
    /// Screen X position for rendering.
    pub render_x: f32,
    /// Screen Y position for rendering.
    pub render_y: f32,
    /// Ghost width in logical pixels.
    pub width_px: f32,
    /// Ghost height in logical pixels.
    pub height_px: f32,
    /// Visual rotation tilt in degrees.
    pub tilt_degrees: f32,
    /// Visual scale factor.
    pub scale: f32,
    /// Surface opacity.
    pub opacity: f32,
    /// Shadow blur radius in pixels.
    pub shadow_blur_px: f32,
    /// Shadow opacity alpha.
    pub shadow_alpha: f32,
    /// Border highlight color hex.
    pub border_color_hex: String,
    /// Border width in logical pixels.
    pub border_width_px: f32,
    /// Dynamic status badge text.
    pub badge_label: String,
    /// Whether the current hover target is a valid drop destination.
    pub is_valid_target: bool,
    /// The status column currently hovered over, if any.
    pub hover_status: Option<TaskStatus>,
}

/// Presentation view model for the visual drop indicator placeholder in target Task Kanban columns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskDropPlaceholderViewModel {
    /// Target task status of the column.
    pub target_status: TaskStatus,
    /// Target insertion index in the column's card stack.
    pub insert_index: usize,
    /// Width of the placeholder container in logical pixels.
    pub width_px: f32,
    /// Height of the placeholder container in logical pixels.
    pub height_px: f32,
    /// Border color hex matching target status or dragged card accent.
    pub border_color_hex: String,
    /// Border width in logical pixels (default 2.0).
    pub border_width_px: f32,
    /// Border stroke style (`"dashed"`).
    pub border_style: &'static str,
    /// Translucent background tint hex.
    pub bg_tint_hex: String,
    /// Descriptive prompt text (e.g. `"+ Move task to In Progress"`).
    pub prompt_text: String,
    /// Whether this placeholder represents a valid drop target.
    pub is_valid: bool,
    /// Whether the pulsing animation indicator is active.
    pub is_pulse_active: bool,
}

impl TaskDropPlaceholderViewModel {
    /// Constructs a `TaskDropPlaceholderViewModel` for a target status column.
    #[must_use]
    pub fn new(
        target_status: TaskStatus,
        insert_index: usize,
        dragged_title: &str,
        dragged_color_hex: &str,
        column_width: f32,
        is_valid: bool,
    ) -> Self {
        let width_px = (column_width - 8.0).max(240.0);
        let height_px = 88.0;
        let prompt_text = if is_valid {
            format!(
                "+ Move \"{dragged_title}\" to {}",
                target_status.display_name()
            )
        } else {
            "Cannot drop in current column".to_string()
        };

        let bg_tint_hex = if is_valid {
            format!("{dragged_color_hex}1A")
        } else {
            "#EF44441A".to_string()
        };

        let border_color_hex = if is_valid {
            dragged_color_hex.to_string()
        } else {
            "#EF4444".to_string()
        };

        Self {
            target_status,
            insert_index,
            width_px,
            height_px,
            border_color_hex,
            border_width_px: 2.0,
            border_style: "dashed",
            bg_tint_hex,
            prompt_text,
            is_valid,
            is_pulse_active: is_valid,
        }
    }
}

/// Header view model for an individual task status Kanban column.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskColumnHeaderViewModel {
    /// Workflow status.
    pub status: TaskStatus,
    /// 0-based column index across the 3-column deck (0..=2).
    pub index: usize,
    /// Column title.
    pub title: &'static str,
    /// Editorial description for this column.
    pub description: &'static str,
    /// Emoji icon.
    pub icon_emoji: &'static str,
    /// Standard symbolic icon name.
    pub icon_name: &'static str,
    /// macOS SF Symbol identifier.
    pub sf_symbol: &'static str,
    /// Status accent hex color.
    pub accent_hex: &'static str,
    /// Total task count in this column.
    pub task_count: usize,
    /// Formatted card count string (e.g. "0 tasks", "1 task", "4 tasks").
    pub task_count_label: String,
    /// Formatted compact badge label (e.g. "0", "1", "4").
    pub count_badge_text: String,
    /// Number of overdue tasks in this column.
    pub overdue_count: usize,
    /// Overdue alert badge label if any tasks are overdue (e.g. Some("⚠️ 1 Overdue")).
    pub overdue_badge_label: Option<String>,
    /// Number of due-soon tasks in this column.
    pub due_soon_count: usize,
    /// Due soon warning badge label if any tasks are due soon.
    pub due_soon_badge_label: Option<String>,
    /// Quick action button label (e.g. "+ Add Task", "+ Start Task").
    pub quick_add_label: &'static str,
    /// Tooltip text for the quick action button.
    pub quick_add_tooltip: String,
    /// Message dispatched when clicking the column header quick add button.
    pub quick_add_action: AppMessage,
}

impl TaskColumnHeaderViewModel {
    /// Formats task count into a human-readable label.
    #[must_use]
    pub fn format_task_count_label(count: usize) -> String {
        match count {
            1 => "1 task".to_string(),
            n => format!("{n} tasks"),
        }
    }

    /// Returns the quick-add button label for a given task status.
    #[must_use]
    pub const fn quick_add_label_for_status(status: TaskStatus) -> &'static str {
        match status {
            TaskStatus::ToDo => "+ Add Task",
            TaskStatus::InProgress => "+ Start Task",
            TaskStatus::Complete => "+ Complete Task",
        }
    }

    /// Constructs the column header view model.
    #[must_use]
    pub fn build(
        status: TaskStatus,
        task_count: usize,
        overdue_count: usize,
        due_soon_count: usize,
        selected_article_id: Option<Uuid>,
    ) -> Self {
        let meta = task_status_metadata(status);
        let task_count_label = Self::format_task_count_label(task_count);
        let count_badge_text = task_count.to_string();
        let overdue_badge_label = if overdue_count > 0 {
            Some(format!("⚠️ {overdue_count} Overdue"))
        } else {
            None
        };
        let due_soon_badge_label = if due_soon_count > 0 {
            Some(format!("🕒 {due_soon_count} Due Soon"))
        } else {
            None
        };
        let quick_add_label = Self::quick_add_label_for_status(status);
        let quick_add_tooltip = format!("Create new task in {} column", meta.title);
        let quick_add_action = AppMessage::OpenNewTaskInStatusModal(status, selected_article_id);

        Self {
            status,
            index: meta.index,
            title: meta.title,
            description: meta.description,
            icon_emoji: meta.icon_emoji,
            icon_name: meta.icon_name,
            sf_symbol: meta.sf_symbol,
            accent_hex: meta.accent_hex,
            task_count,
            task_count_label,
            count_badge_text,
            overdue_count,
            overdue_badge_label,
            due_soon_count,
            due_soon_badge_label,
            quick_add_label,
            quick_add_tooltip,
            quick_add_action,
        }
    }

    /// Returns `true` if this column has any overdue tasks.
    #[must_use]
    pub const fn has_overdue(&self) -> bool {
        self.overdue_count > 0
    }

    /// Returns `true` if this column has any due-soon tasks.
    #[must_use]
    pub const fn has_due_soon(&self) -> bool {
        self.due_soon_count > 0
    }
}

/// Status-tailored empty state presentation model when a task column has no cards.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskColumnEmptyStateViewModel {
    /// Status.
    pub status: TaskStatus,
    /// Status emoji icon.
    pub icon_emoji: &'static str,
    /// Standard icon name.
    pub icon_name: &'static str,
    /// macOS SF Symbol identifier.
    pub sf_symbol: &'static str,
    /// Status accent hex color.
    pub accent_hex: &'static str,
    /// Empty state headline.
    pub title: &'static str,
    /// Empty state guidance prompt.
    pub prompt: &'static str,
    /// Action button label for quick creation.
    pub action_button_label: &'static str,
    /// Action message to dispatch upon clicking the empty state action button.
    pub action_message: Option<AppMessage>,
    /// Whether this column accepts dropped cards from other statuses.
    pub is_drop_target_hint: bool,
    /// Drop guidance hint text.
    pub drop_hint_text: &'static str,
    /// Whether this empty state is caused by active filtering/search rather than naturally having 0 tasks.
    pub is_filtered_empty: bool,
    /// Contextual explanation if empty due to active filter.
    pub filtered_explanation: Option<String>,
}

impl TaskColumnEmptyStateViewModel {
    /// Returns the headline for a status's empty state.
    #[must_use]
    pub const fn title_for_status(status: TaskStatus) -> &'static str {
        match status {
            TaskStatus::ToDo => "No To-Do Tasks",
            TaskStatus::InProgress => "No Tasks in Progress",
            TaskStatus::Complete => "No Completed Tasks",
        }
    }

    /// Returns the action button label for a status's empty state.
    #[must_use]
    pub const fn action_label_for_status(status: TaskStatus) -> &'static str {
        match status {
            TaskStatus::ToDo => "+ Add Task",
            TaskStatus::InProgress => "+ Start Task",
            TaskStatus::Complete => "+ Add Completed",
        }
    }

    /// Returns the drop guidance hint text for a status's empty state.
    #[must_use]
    pub const fn drop_hint_for_status(status: TaskStatus) -> &'static str {
        match status {
            TaskStatus::ToDo => "Tasks can be dragged here from other columns",
            TaskStatus::InProgress => "Drag tasks here when you start reporting",
            TaskStatus::Complete => "Drag finished tasks here to mark complete",
        }
    }

    /// Constructs the status-tailored empty state view model.
    #[must_use]
    pub fn build(
        status: TaskStatus,
        is_filtered: bool,
        search_query: &str,
        selected_article_id: Option<Uuid>,
    ) -> Self {
        let meta = task_status_metadata(status);
        let title = Self::title_for_status(status);
        let prompt = meta.empty_state_prompt;
        let action_button_label = Self::action_label_for_status(status);
        let action_message = Some(AppMessage::OpenNewTaskInStatusModal(
            status,
            selected_article_id,
        ));
        let is_drop_target_hint = true;
        let drop_hint_text = Self::drop_hint_for_status(status);
        let filtered_explanation = if is_filtered {
            if !search_query.trim().is_empty() {
                Some(format!(
                    "No tasks in {} match \"{}\"",
                    meta.title,
                    search_query.trim()
                ))
            } else {
                Some(format!("No tasks in {} match active filters", meta.title))
            }
        } else {
            None
        };

        Self {
            status,
            icon_emoji: meta.icon_emoji,
            icon_name: meta.icon_name,
            sf_symbol: meta.sf_symbol,
            accent_hex: meta.accent_hex,
            title,
            prompt,
            action_button_label,
            action_message,
            is_drop_target_hint,
            drop_hint_text,
            is_filtered_empty: is_filtered,
            filtered_explanation,
        }
    }
}

/// Top toolbar presentation model for the Tasks Kanban deck view.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TasksToolbarViewModel {
    /// Section title.
    pub title: &'static str,
    /// Total task count across all columns.
    pub total_task_count: usize,
    /// Formatted total tasks label (e.g. "8 total tasks", "0 tasks").
    pub total_count_label: String,
    /// To-Do task count.
    pub todo_count: usize,
    /// In Progress task count.
    pub in_progress_count: usize,
    /// Completed task count.
    pub completed_count: usize,
    /// Total overdue tasks count.
    pub total_overdue_count: usize,
    /// Overdue alert badge label if any tasks are overdue (e.g. Some("⚠️ 2 Overdue")).
    pub overdue_badge_label: Option<String>,
    /// Total due-soon tasks count.
    pub total_due_soon_count: usize,
    /// Due soon warning badge label if any tasks are due soon.
    pub due_soon_badge_label: Option<String>,
    /// Primary quick action button label (e.g. "+ New Task").
    pub primary_action_label: &'static str,
    /// Primary quick action button tooltip.
    pub primary_action_tooltip: &'static str,
    /// Primary action keyboard shortcut display string (e.g. "⌘T" or "Ctrl+T").
    pub primary_action_shortcut: &'static str,
    /// Message dispatched when clicking the primary action button.
    pub primary_action_message: AppMessage,
    /// Current search query string.
    pub search_query: String,
    /// Whether search is actively filtering.
    pub is_search_active: bool,
    /// Active parent article filter (if filtering by article).
    pub selected_article_id: Option<Uuid>,
    /// Active parent article slug display (if filtering by article).
    pub selected_article_slug: Option<String>,
    /// Active parent article color hex (if filtering by article).
    pub selected_article_color: Option<String>,
    /// Active urgency filter.
    pub urgency_filter: UrgencyFilter,
    /// Whether any filter is currently applied.
    pub is_filtered: bool,
}

impl TasksToolbarViewModel {
    /// Formats total task count into a header label.
    #[must_use]
    pub fn format_total_count_label(count: usize) -> String {
        match count {
            0 => "0 tasks".to_string(),
            1 => "1 task".to_string(),
            n => format!("{n} total tasks"),
        }
    }

    /// Constructs the toolbar presentation model from state and deck counts.
    #[must_use]
    pub fn build(
        state: &AppState,
        total_task_count: usize,
        todo_count: usize,
        in_progress_count: usize,
        completed_count: usize,
        total_overdue_count: usize,
        total_due_soon_count: usize,
    ) -> Self {
        let total_count_label = Self::format_total_count_label(total_task_count);
        let overdue_badge_label = if total_overdue_count > 0 {
            Some(format!("⚠️ {total_overdue_count} Overdue"))
        } else {
            None
        };
        let due_soon_badge_label = if total_due_soon_count > 0 {
            Some(format!("🕒 {total_due_soon_count} Due Soon"))
        } else {
            None
        };
        let search_query = state.filters.search_query.clone();
        let is_search_active = !search_query.trim().is_empty();
        let urgency_filter = state.filters.urgency_filter;
        let selected_article_id = state.filters.selected_article_id;

        let (selected_article_slug, selected_article_color) = if let Some(aid) = selected_article_id
        {
            let parent = state.get_article(aid);
            let slug = parent.map(|a| a.slug.clone());
            let color = parent
                .and_then(|a| a.color.clone())
                .or_else(|| slug.as_ref().map(|s| assign_color_for_slug(s).to_hex()));
            (slug, color)
        } else {
            (None, None)
        };

        let is_filtered = state.filters.is_active();

        Self {
            title: "Tasks Kanban",
            total_task_count,
            total_count_label,
            todo_count,
            in_progress_count,
            completed_count,
            total_overdue_count,
            overdue_badge_label,
            total_due_soon_count,
            due_soon_badge_label,
            primary_action_label: "+ New Task",
            primary_action_tooltip: "Create a new reporting task (⌘T / Ctrl+T)",
            primary_action_shortcut: "⌘T",
            primary_action_message: AppMessage::OpenNewTaskModal(selected_article_id),
            search_query,
            is_search_active,
            selected_article_id,
            selected_article_slug,
            selected_article_color,
            urgency_filter,
            is_filtered,
        }
    }

    /// Returns `true` if any tasks in the deck are overdue.
    #[must_use]
    pub const fn has_overdue(&self) -> bool {
        self.total_overdue_count > 0
    }

    /// Returns `true` if any tasks in the deck are due soon.
    #[must_use]
    pub const fn has_due_soon(&self) -> bool {
        self.total_due_soon_count > 0
    }
}

/// Deck-wide empty state presentation model when no tasks exist in the deck.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TasksDeckEmptyStateViewModel {
    /// Large headline text.
    pub headline: String,
    /// Explanatory guidance text.
    pub subtext: String,
    /// Primary call to action button label.
    pub action_button_label: String,
    /// Primary action message.
    pub action_message: AppMessage,
    /// Secondary action button label (if any, e.g. "Clear Filters").
    pub secondary_action_label: Option<String>,
    /// Secondary action message (if any).
    pub secondary_action_message: Option<AppMessage>,
    /// Whether this empty state is caused by active filtering.
    pub is_filtered: bool,
    /// Standard symbolic icon name.
    pub icon_name: &'static str,
    /// macOS SF Symbol identifier.
    pub sf_symbol: &'static str,
    /// Emoji representation.
    pub icon_emoji: &'static str,
}

impl TasksDeckEmptyStateViewModel {
    /// Constructs a deck empty state view model if total task count is 0.
    #[must_use]
    pub fn build(
        total_task_count: usize,
        is_filtered: bool,
        search_query: &str,
        selected_article_id: Option<Uuid>,
    ) -> Option<Self> {
        if total_task_count > 0 {
            return None;
        }

        if is_filtered {
            let subtext = if !search_query.trim().is_empty() {
                format!(
                    "No tasks found matching \"{}\". Try adjusting your search query or clear all filters.",
                    search_query.trim()
                )
            } else if selected_article_id.is_some() {
                "No tasks found for the selected article. Click \"+ New Task\" to add action items."
                    .to_string()
            } else {
                "No tasks found matching the active filters.".to_string()
            };

            Some(Self {
                headline: "No Matching Tasks".to_string(),
                subtext,
                action_button_label: "Clear Filters".to_string(),
                action_message: AppMessage::ClearFilters,
                secondary_action_label: Some("+ New Task".to_string()),
                secondary_action_message: Some(AppMessage::OpenNewTaskModal(selected_article_id)),
                is_filtered: true,
                icon_name: "search",
                sf_symbol: "magnifyingglass",
                icon_emoji: "🔍",
            })
        } else {
            Some(Self {
                headline: "Welcome to Tasks Kanban".to_string(),
                subtext: "Organize reporting tasks, interview prep, and fact-checking checklists across To-Do, In Progress, and Complete columns. Get started by creating your first task.".to_string(),
                action_button_label: "+ Create Your First Task".to_string(),
                action_message: AppMessage::OpenNewTaskModal(selected_article_id),
                secondary_action_label: None,
                secondary_action_message: None,
                is_filtered: false,
                icon_name: "checklist",
                sf_symbol: "checklist",
                icon_emoji: "✅",
            })
        }
    }
}

/// Formatted view model for one of the 3 Task status Kanban columns.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskColumnViewModel {
    /// Task status.
    pub status: TaskStatus,
    /// 0-based column index (0..=2).
    pub index: usize,
    /// Column title.
    pub title: &'static str,
    /// Editorial description for this status.
    pub description: &'static str,
    /// Emoji icon.
    pub icon_emoji: &'static str,
    /// Standard icon name.
    pub icon_name: &'static str,
    /// macOS SF Symbol identifier.
    pub sf_symbol: &'static str,
    /// Status accent hex color.
    pub accent_hex: &'static str,
    /// Column header presentation model with counts and quick action button.
    pub header: TaskColumnHeaderViewModel,
    /// List of task cards in this column.
    pub cards: Vec<TaskCardViewModel>,
    /// Total card count in this column.
    pub task_count: usize,
    /// Number of overdue cards in this column.
    pub overdue_count: usize,
    /// Number of cards due soon in this column.
    pub due_soon_count: usize,
    /// Whether this column is hovered during drag-and-drop.
    pub is_hovered: bool,
    /// Whether this column is a valid drop destination for the currently dragged card.
    pub is_valid_drop_target: bool,
    /// Whether this column is the active valid drop target.
    pub is_active_drop_target: bool,
    /// Visual drop indicator placeholder if this column is the active drop target.
    pub drop_placeholder: Option<TaskDropPlaceholderViewModel>,
    /// Column border highlight hex during drag hover.
    pub drop_highlight_border_hex: Option<String>,
    /// Column background tint hex during drag hover.
    pub drop_highlight_bg_tint_hex: Option<String>,
    /// Whether this column has zero cards.
    pub is_empty: bool,
    /// Status-specific empty state guidance message.
    pub empty_state_prompt: &'static str,
    /// Status-specific empty state model when task_count == 0.
    pub empty_state: Option<TaskColumnEmptyStateViewModel>,
}

/// Formatted view model for the entire Tasks Kanban Deck (3 columns, horizontal scrolling, drag ghost).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TasksKanbanDeckViewModel {
    /// Top toolbar presentation model with metrics and '+ New Task' trigger.
    pub toolbar: TasksToolbarViewModel,
    /// The 3 workflow status columns in sequential order (To-Do, In Progress, Complete).
    pub columns: Vec<TaskColumnViewModel>,
    /// Total number of tasks across all 3 columns.
    pub total_task_count: usize,
    /// To-Do task count.
    pub todo_count: usize,
    /// In Progress task count.
    pub in_progress_count: usize,
    /// Completed task count.
    pub completed_count: usize,
    /// Total number of overdue tasks across the entire deck.
    pub total_overdue_count: usize,
    /// Total number of tasks due soon across the entire deck.
    pub total_due_soon_count: usize,
    /// Filtered parent article ID if active.
    pub selected_article_id: Option<Uuid>,
    /// Filtered parent article slug if active.
    pub selected_article_slug: Option<String>,
    /// Filtered parent article color if active.
    pub selected_article_color: Option<String>,
    /// Whether any search query or article filter is actively applied.
    pub is_filtered: bool,
    /// Active search filter query string.
    pub search_query: String,
    /// Deck-wide empty state if total_task_count == 0.
    pub deck_empty_state: Option<TasksDeckEmptyStateViewModel>,
    /// Active card currently being dragged, if any.
    pub active_drag_item: Option<DragItem>,
    /// Whether a drag-and-drop session is currently active.
    pub is_dragging: bool,
    /// Current hovered drop target.
    pub hover_target: Option<DropTarget>,
    /// Whether the current drag hover target is valid for dropping.
    pub is_valid_drop: bool,
    /// Floating visual drag ghost following the cursor, if drag is active.
    pub drag_ghost: Option<TaskDragGhostViewModel>,
    /// Horizontal scrolling and geometry configuration for the deck.
    pub layout: TasksDeckLayoutConfig,
}

impl TasksKanbanDeckViewModel {
    /// Looks up a column by its workflow status.
    #[must_use]
    pub fn column(&self, status: TaskStatus) -> Option<&TaskColumnViewModel> {
        self.columns.iter().find(|col| col.status == status)
    }

    /// Looks up a mutable column by its workflow status.
    pub fn column_mut(&mut self, status: TaskStatus) -> Option<&mut TaskColumnViewModel> {
        self.columns.iter_mut().find(|col| col.status == status)
    }

    /// Looks up a column by its 0-based index.
    #[must_use]
    pub fn column_by_index(&self, index: usize) -> Option<&TaskColumnViewModel> {
        self.columns.get(index)
    }

    /// Looks up a mutable column by its 0-based index.
    pub fn column_by_index_mut(&mut self, index: usize) -> Option<&mut TaskColumnViewModel> {
        self.columns.get_mut(index)
    }

    /// Finds a task card by its unique UUID across all columns in the deck.
    #[must_use]
    pub fn card(&self, id: Uuid) -> Option<(&TaskColumnViewModel, &TaskCardViewModel)> {
        for col in &self.columns {
            if let Some(card) = col.cards.iter().find(|c| c.id == id) {
                return Some((col, card));
            }
        }
        None
    }

    /// Returns the task status located at the given screen X coordinate.
    #[must_use]
    pub fn status_at_point(&self, screen_x: f32, _screen_y: f32) -> Option<TaskStatus> {
        self.layout.status_at_screen_x(screen_x)
    }

    /// Returns the column index located at the given screen X coordinate.
    #[must_use]
    pub fn column_index_at_point(&self, screen_x: f32, _screen_y: f32) -> Option<usize> {
        self.layout.column_index_at_screen_x(screen_x)
    }

    /// Returns the drop target located at the given screen X coordinate.
    #[must_use]
    pub fn drop_target_at_point(&self, screen_x: f32, _screen_y: f32) -> Option<DropTarget> {
        self.layout
            .status_at_screen_x(screen_x)
            .map(DropTarget::TaskColumn)
    }

    /// Returns `true` if all columns in the deck have zero tasks.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.total_task_count == 0
    }

    /// Returns `true` if any task in the deck is overdue.
    #[must_use]
    pub fn has_overdue(&self) -> bool {
        self.total_overdue_count > 0
    }

    /// Returns the number of tasks in the specified status column.
    #[must_use]
    pub fn count_for_status(&self, status: TaskStatus) -> usize {
        self.column(status).map_or(0, |col| col.task_count)
    }

    /// Returns the number of overdue tasks in the specified status column.
    #[must_use]
    pub fn overdue_count_for_status(&self, status: TaskStatus) -> usize {
        self.column(status).map_or(0, |col| col.overdue_count)
    }

    /// Returns the number of due-soon tasks in the specified status column.
    #[must_use]
    pub fn due_soon_count_for_status(&self, status: TaskStatus) -> usize {
        self.column(status).map_or(0, |col| col.due_soon_count)
    }

    /// Returns the titles of all 3 statuses in sequential order.
    #[must_use]
    pub const fn status_names() -> [&'static str; NUM_TASK_STATUSES] {
        ["To-Do", "In Progress", "Complete"]
    }

    /// Returns all 3 statuses in sequential workflow order.
    #[must_use]
    pub const fn statuses() -> &'static [TaskStatus] {
        TaskStatus::all()
    }
}

/// Constructs the 3-column Tasks Kanban view model list from application state.
#[must_use]
pub fn build_tasks_kanban_view(state: &AppState) -> Vec<TaskColumnViewModel> {
    let deck = build_tasks_kanban_deck(state);
    deck.columns
}

/// Constructs the complete 3-column Tasks Kanban Deck view model from application state with default layout.
#[must_use]
pub fn build_tasks_kanban_deck(state: &AppState) -> TasksKanbanDeckViewModel {
    build_tasks_kanban_deck_with_layout(state, TasksDeckLayoutConfig::default())
}

/// Constructs the complete 3-column Tasks Kanban Deck view model from application state with custom layout geometry.
#[must_use]
pub fn build_tasks_kanban_deck_with_layout(
    state: &AppState,
    layout: TasksDeckLayoutConfig,
) -> TasksKanbanDeckViewModel {
    let statuses = TaskStatus::all();

    let (active_drag_item, dragged_task_card, drag_ghost) = match state.drag.active_item {
        Some(DragItem::TaskCard {
            id,
            article_id,
            origin_status,
        }) => {
            let card_opt = state
                .get_task(id)
                .map(|task| TaskCardViewModel::build(task, state, true));

            let ghost_opt = card_opt.as_ref().map(|card| {
                let pointer_pos = state.drag.pointer_pos.unwrap_or_else(|| {
                    let orig_idx = task_status_metadata(origin_status).index;
                    let (orig_left, _) = layout.column_screen_bounds(orig_idx);
                    (orig_left + layout.column_width / 2.0, 160.0)
                });
                let width_px = (layout.column_width - 8.0).max(240.0);
                let height_px = 88.0;
                let render_x = pointer_pos.0 - (width_px / 2.0);
                let render_y = pointer_pos.1 - 24.0;

                let hover_status = match state.drag.hover_target {
                    Some(DropTarget::TaskColumn(s)) => Some(s),
                    _ => None,
                };
                let is_valid_target = match hover_status {
                    Some(s) => s != origin_status,
                    None => false,
                };
                let badge_label = match hover_status {
                    Some(s) if s != origin_status => {
                        format!("MOVE TO {}", s.display_name().to_uppercase())
                    }
                    Some(_) => "SAME STATUS".to_string(),
                    None => "DRAGGING TASK".to_string(),
                };

                TaskDragGhostViewModel {
                    id: card.id,
                    title: card.title.clone(),
                    parent_article_slug: card.parent_article_slug.clone(),
                    parent_article_color: card.parent_article_color.clone(),
                    origin_status,
                    pointer_pos,
                    render_x,
                    render_y,
                    width_px,
                    height_px,
                    tilt_degrees: 2.5,
                    scale: 1.03,
                    opacity: 0.92,
                    shadow_blur_px: 24.0,
                    shadow_alpha: 0.38,
                    border_color_hex: if is_valid_target {
                        "#10B981".to_string()
                    } else {
                        card.parent_article_color.clone()
                    },
                    border_width_px: 2.0,
                    badge_label,
                    is_valid_target,
                    hover_status,
                }
            });

            (
                Some(DragItem::TaskCard {
                    id,
                    article_id,
                    origin_status,
                }),
                card_opt,
                ghost_opt,
            )
        }
        Some(article_item) => (Some(article_item), None, None),
        None => (None, None, None),
    };

    let hover_status = match state.drag.hover_target {
        Some(DropTarget::TaskColumn(status)) => Some(status),
        _ => None,
    };

    let is_dragging = state.drag.is_dragging();
    let is_valid_drop = state.drag.is_valid_drop();

    let filtered_tasks = state.filtered_tasks();
    let is_filtered = state.filters.is_active();
    let search_query = state.filters.search_query.clone();
    let selected_article_id = state.filters.selected_article_id;

    let columns: Vec<TaskColumnViewModel> = statuses
        .iter()
        .map(|&status| {
            let meta = task_status_metadata(status);
            let is_this_column_hovered = hover_status == Some(status);
            let is_valid_target = matches!(active_drag_item, Some(DragItem::TaskCard { .. }));
            let is_active_target = is_this_column_hovered && is_valid_target;

            let cards: Vec<TaskCardViewModel> = filtered_tasks
                .iter()
                .filter(|t| t.status == status)
                .map(|&task| {
                    let is_this_card_dragging = active_drag_item
                        .map(|item| item.id() == task.id)
                        .unwrap_or(false);
                    TaskCardViewModel::build(task, state, is_this_card_dragging)
                })
                .collect();

            let task_count = cards.len();
            let overdue_count = cards.iter().filter(|c| c.is_overdue).count();
            let due_soon_count = cards.iter().filter(|c| c.is_due_soon).count();

            let header = TaskColumnHeaderViewModel::build(
                status,
                task_count,
                overdue_count,
                due_soon_count,
                selected_article_id,
            );

            let empty_state = if task_count == 0 {
                Some(TaskColumnEmptyStateViewModel::build(
                    status,
                    is_filtered,
                    &search_query,
                    selected_article_id,
                ))
            } else {
                None
            };

            let drop_placeholder = if is_active_target {
                dragged_task_card.as_ref().map(|dragged| {
                    TaskDropPlaceholderViewModel::new(
                        status,
                        state.drag.drop_insert_index.unwrap_or(task_count),
                        &dragged.title,
                        &dragged.parent_article_color,
                        layout.column_width,
                        is_valid_drop,
                    )
                })
            } else {
                None
            };

            let (drop_highlight_border_hex, drop_highlight_bg_tint_hex) = if is_active_target {
                (
                    Some(meta.accent_hex.to_string()),
                    Some(format!("{}1A", meta.accent_hex)),
                )
            } else {
                (None, None)
            };

            TaskColumnViewModel {
                status,
                index: meta.index,
                title: meta.title,
                description: meta.description,
                icon_emoji: meta.icon_emoji,
                icon_name: meta.icon_name,
                sf_symbol: meta.sf_symbol,
                accent_hex: meta.accent_hex,
                header,
                cards,
                task_count,
                overdue_count,
                due_soon_count,
                is_hovered: is_this_column_hovered,
                is_valid_drop_target: is_valid_target,
                is_active_drop_target: is_active_target,
                drop_placeholder,
                drop_highlight_border_hex,
                drop_highlight_bg_tint_hex,
                is_empty: task_count == 0,
                empty_state_prompt: meta.empty_state_prompt,
                empty_state,
            }
        })
        .collect();

    let total_task_count = columns.iter().map(|c| c.task_count).sum();
    let todo_count = columns
        .iter()
        .find(|c| c.status == TaskStatus::ToDo)
        .map_or(0, |c| c.task_count);
    let in_progress_count = columns
        .iter()
        .find(|c| c.status == TaskStatus::InProgress)
        .map_or(0, |c| c.task_count);
    let completed_count = columns
        .iter()
        .find(|c| c.status == TaskStatus::Complete)
        .map_or(0, |c| c.task_count);
    let total_overdue_count = columns.iter().map(|c| c.overdue_count).sum();
    let total_due_soon_count = columns.iter().map(|c| c.due_soon_count).sum();

    let (selected_article_slug, selected_article_color) = if let Some(aid) = selected_article_id {
        let parent = state.get_article(aid);
        let slug = parent.map(|a| a.slug.clone());
        let color = parent
            .and_then(|a| a.color.clone())
            .or_else(|| slug.as_ref().map(|s| assign_color_for_slug(s).to_hex()));
        (slug, color)
    } else {
        (None, None)
    };

    let toolbar = TasksToolbarViewModel::build(
        state,
        total_task_count,
        todo_count,
        in_progress_count,
        completed_count,
        total_overdue_count,
        total_due_soon_count,
    );

    let deck_empty_state = TasksDeckEmptyStateViewModel::build(
        total_task_count,
        is_filtered,
        &search_query,
        selected_article_id,
    );

    TasksKanbanDeckViewModel {
        toolbar,
        columns,
        total_task_count,
        todo_count,
        in_progress_count,
        completed_count,
        total_overdue_count,
        total_due_soon_count,
        selected_article_id,
        selected_article_slug,
        selected_article_color,
        is_filtered,
        search_query,
        deck_empty_state,
        active_drag_item,
        is_dragging,
        hover_target: state.drag.hover_target,
        is_valid_drop,
        drag_ghost,
        layout,
    }
}
