//! Tasks Kanban board view models.

use chrono::{DateTime, Utc};
use newsjournal_core::color::assign_color_for_slug;
use newsjournal_core::models::TaskStatus;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::drag_drop::{DragItem, DropTarget};
use crate::state::AppState;

/// Formatted view model for a single Task card in the Tasks board.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskCardViewModel {
    /// Unique task ID.
    pub id: Uuid,
    /// Parent article ID.
    pub article_id: Uuid,
    /// Parent article slug.
    pub parent_article_slug: String,
    /// Parent article color hex.
    pub parent_article_color: String,
    /// Task title.
    pub title: String,
    /// Task notes.
    pub notes: String,
    /// Optional due date.
    pub due_date: Option<DateTime<Utc>>,
    /// Status.
    pub status: TaskStatus,
    /// True if this card is currently being dragged.
    pub is_dragging: bool,
}

/// Formatted view model for one of the 3 Task status columns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskColumnViewModel {
    /// Task status.
    pub status: TaskStatus,
    /// Column title.
    pub title: &'static str,
    /// List of task cards in this column.
    pub cards: Vec<TaskCardViewModel>,
    /// Whether this column is hovered during drag-and-drop.
    pub is_hovered: bool,
}

/// Constructs the 3-column Tasks Kanban view model from application state.
#[must_use]
pub fn build_tasks_kanban_view(state: &AppState) -> Vec<TaskColumnViewModel> {
    let statuses = [
        TaskStatus::ToDo,
        TaskStatus::InProgress,
        TaskStatus::Complete,
    ];

    let active_drag_id = match state.drag.active_item {
        Some(DragItem::TaskCard { id, .. }) => Some(id),
        _ => None,
    };

    let hover_status = match state.drag.hover_target {
        Some(DropTarget::TaskColumn(status)) => Some(status),
        _ => None,
    };

    statuses
        .iter()
        .map(|&status| {
            let cards: Vec<TaskCardViewModel> = state
                .tasks
                .iter()
                .filter(|t| {
                    if let Some(target_aid) = state.filters.selected_article_id {
                        if t.article_id != target_aid {
                            return false;
                        }
                    }
                    t.status == status
                })
                .map(|task| {
                    let parent = state.get_article(task.article_id);
                    let slug = parent
                        .map(|a| a.slug.clone())
                        .unwrap_or_else(|| "unassigned".to_string());
                    let color = parent
                        .and_then(|a| a.color.clone())
                        .unwrap_or_else(|| assign_color_for_slug(&slug).to_hex());

                    TaskCardViewModel {
                        id: task.id,
                        article_id: task.article_id,
                        parent_article_slug: slug,
                        parent_article_color: color,
                        title: task.title.clone(),
                        notes: task.notes.clone().unwrap_or_default(),
                        due_date: task.due_date,
                        status: task.status,
                        is_dragging: active_drag_id == Some(task.id),
                    }
                })
                .collect();

            let title = match status {
                TaskStatus::ToDo => "To-Do",
                TaskStatus::InProgress => "In Progress",
                TaskStatus::Complete => "Complete",
            };

            TaskColumnViewModel {
                status,
                title,
                cards,
                is_hovered: hover_status == Some(status),
            }
        })
        .collect()
}
