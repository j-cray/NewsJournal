//! Drag-and-drop state machine for Articles and Tasks Kanban boards.

use newsjournal_core::{ArticleStage, TaskStatus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Entity card currently being dragged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DragItem {
    /// An Article card dragged from an origin Kanban column.
    ArticleCard {
        /// Unique article ID.
        id: Uuid,
        /// Initial stage column before drag began.
        origin_stage: ArticleStage,
    },
    /// A Task card dragged from an origin Kanban column.
    TaskCard {
        /// Unique task ID.
        id: Uuid,
        /// Parent article ID.
        article_id: Uuid,
        /// Initial task status before drag began.
        origin_status: TaskStatus,
    },
}

impl DragItem {
    /// Returns the unique ID of the dragged item.
    #[must_use]
    pub const fn id(&self) -> Uuid {
        match *self {
            Self::ArticleCard { id, .. } | Self::TaskCard { id, .. } => id,
        }
    }
}

/// Target column where a dragged card can be dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DropTarget {
    /// Target article stage column.
    ArticleColumn(ArticleStage),
    /// Target task status column.
    TaskColumn(TaskStatus),
}

/// Active drag-and-drop session state.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DragState {
    /// The card currently being dragged, if any.
    pub active_item: Option<DragItem>,
    /// The column target currently hovered over.
    pub hover_target: Option<DropTarget>,
}

impl DragState {
    /// Creates a new inactive drag state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            active_item: None,
            hover_target: None,
        }
    }

    /// Starts dragging an item.
    pub fn start_drag(&mut self, item: DragItem) {
        self.active_item = Some(item);
        self.hover_target = None;
    }

    /// Updates the currently hovered drop target.
    pub fn update_hover(&mut self, target: Option<DropTarget>) {
        self.hover_target = target;
    }

    /// Checks if a drag operation is currently in progress.
    #[must_use]
    pub const fn is_dragging(&self) -> bool {
        self.active_item.is_some()
    }

    /// Determines if the current hover target is a valid drop location for the active item.
    #[must_use]
    pub fn is_valid_drop(&self) -> bool {
        match (self.active_item, self.hover_target) {
            (
                Some(DragItem::ArticleCard { origin_stage, .. }),
                Some(DropTarget::ArticleColumn(target_stage)),
            ) => origin_stage != target_stage,
            (
                Some(DragItem::TaskCard { origin_status, .. }),
                Some(DropTarget::TaskColumn(target_status)),
            ) => origin_status != target_status,
            _ => false,
        }
    }

    /// Completes the drag operation if valid, returning the dragged item and destination target.
    pub fn complete_drop(&mut self) -> Option<(DragItem, DropTarget)> {
        if self.is_valid_drop() {
            let item = self.active_item.take();
            let target = self.hover_target.take();
            match (item, target) {
                (Some(i), Some(t)) => Some((i, t)),
                _ => None,
            }
        } else {
            self.cancel();
            None
        }
    }

    /// Cancels and resets the drag operation.
    pub fn cancel(&mut self) {
        self.active_item = None;
        self.hover_target = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_article_drag_and_drop_lifecycle() {
        let mut state = DragState::new();
        assert!(!state.is_dragging());

        let article_id = Uuid::new_v4();
        state.start_drag(DragItem::ArticleCard {
            id: article_id,
            origin_stage: ArticleStage::Pitching,
        });
        assert!(state.is_dragging());
        assert!(!state.is_valid_drop());

        // Hover over same column (not a transition)
        state.update_hover(Some(DropTarget::ArticleColumn(ArticleStage::Pitching)));
        assert!(!state.is_valid_drop());

        // Hover over different column (valid transition)
        state.update_hover(Some(DropTarget::ArticleColumn(ArticleStage::Researching)));
        assert!(state.is_valid_drop());

        // Hover over invalid target type (task column)
        state.update_hover(Some(DropTarget::TaskColumn(TaskStatus::InProgress)));
        assert!(!state.is_valid_drop());

        // Hover back to valid and complete
        state.update_hover(Some(DropTarget::ArticleColumn(ArticleStage::Writing)));
        assert!(state.is_valid_drop());

        let drop = state.complete_drop();
        assert_eq!(
            drop,
            Some((
                DragItem::ArticleCard {
                    id: article_id,
                    origin_stage: ArticleStage::Pitching
                },
                DropTarget::ArticleColumn(ArticleStage::Writing)
            ))
        );
        assert!(!state.is_dragging());
    }

    #[test]
    fn test_task_drag_and_drop_lifecycle() {
        let mut state = DragState::new();
        let task_id = Uuid::new_v4();
        let article_id = Uuid::new_v4();

        state.start_drag(DragItem::TaskCard {
            id: task_id,
            article_id,
            origin_status: TaskStatus::ToDo,
        });
        assert!(state.is_dragging());

        state.update_hover(Some(DropTarget::TaskColumn(TaskStatus::Complete)));
        assert!(state.is_valid_drop());

        state.cancel();
        assert!(!state.is_dragging());
        assert_eq!(state.hover_target, None);
    }
}
