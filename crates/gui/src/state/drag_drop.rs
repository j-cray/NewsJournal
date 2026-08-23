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

/// Active drag-and-drop session state tracking pointer coordinates and target insertion points.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DragState {
    /// The card currently being dragged, if any.
    pub active_item: Option<DragItem>,
    /// The column target currently hovered over.
    pub hover_target: Option<DropTarget>,
    /// Pointer coordinates `(x, y)` where the drag gesture was initiated.
    pub drag_start_pos: Option<(f32, f32)>,
    /// Current mouse cursor coordinates `(x, y)` in logical pixels.
    pub pointer_pos: Option<(f32, f32)>,
    /// Target card insertion index in the hovered column.
    pub drop_insert_index: Option<usize>,
}

impl DragState {
    /// Creates a new inactive drag state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            active_item: None,
            hover_target: None,
            drag_start_pos: None,
            pointer_pos: None,
            drop_insert_index: None,
        }
    }

    /// Starts dragging an item without explicit initial pointer coordinates.
    pub fn start_drag(&mut self, item: DragItem) {
        self.active_item = Some(item);
        self.hover_target = None;
        self.drag_start_pos = None;
        self.pointer_pos = None;
        self.drop_insert_index = None;
    }

    /// Starts dragging an item with initial pointer coordinates.
    pub fn start_drag_with_position(&mut self, item: DragItem, start_pos: (f32, f32)) {
        self.active_item = Some(item);
        self.hover_target = None;
        self.drag_start_pos = Some(start_pos);
        self.pointer_pos = Some(start_pos);
        self.drop_insert_index = None;
    }

    /// Updates the current pointer coordinates during drag motion.
    pub fn update_pointer_position(&mut self, pos: (f32, f32)) {
        self.pointer_pos = Some(pos);
    }

    /// Updates the currently hovered drop target.
    pub fn update_hover(&mut self, target: Option<DropTarget>) {
        self.hover_target = target;
        if target.is_none() {
            self.drop_insert_index = None;
        }
    }

    /// Updates the currently hovered drop target and target insertion index.
    pub fn update_hover_with_index(
        &mut self,
        target: Option<DropTarget>,
        insert_index: Option<usize>,
    ) {
        self.hover_target = target;
        self.drop_insert_index = insert_index;
    }

    /// Checks if a drag operation is currently in progress.
    #[must_use]
    pub const fn is_dragging(&self) -> bool {
        self.active_item.is_some()
    }

    /// Computes the pointer displacement vector `(dx, dy)` from drag start to current position.
    #[must_use]
    pub fn drag_delta(&self) -> (f32, f32) {
        match (self.drag_start_pos, self.pointer_pos) {
            (Some(start), Some(curr)) => (curr.0 - start.0, curr.1 - start.1),
            _ => (0.0, 0.0),
        }
    }

    /// Computes the Euclidean distance in logical pixels traveled since drag start.
    #[must_use]
    pub fn drag_distance(&self) -> f32 {
        let (dx, dy) = self.drag_delta();
        (dx * dx + dy * dy).sqrt()
    }

    /// Returns `true` if the drag motion has exceeded the given movement threshold in pixels.
    #[must_use]
    pub fn has_exceeded_drag_threshold(&self, threshold_px: f32) -> bool {
        self.drag_distance() >= threshold_px
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
            self.drag_start_pos = None;
            self.pointer_pos = None;
            self.drop_insert_index = None;
            match (item, target) {
                (Some(i), Some(t)) => Some((i, t)),
                _ => None,
            }
        } else {
            self.cancel();
            None
        }
    }

    /// Cancels and resets all active drag session state.
    pub fn cancel(&mut self) {
        self.active_item = None;
        self.hover_target = None;
        self.drag_start_pos = None;
        self.pointer_pos = None;
        self.drop_insert_index = None;
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

    #[test]
    fn test_drag_state_pointer_tracking_and_metrics() {
        let mut state = DragState::new();
        let article_id = Uuid::new_v4();

        // 1. Start drag with initial coordinates
        state.start_drag_with_position(
            DragItem::ArticleCard {
                id: article_id,
                origin_stage: ArticleStage::Pitching,
            },
            (100.0, 200.0),
        );
        assert!(state.is_dragging());
        assert_eq!(state.drag_start_pos, Some((100.0, 200.0)));
        assert_eq!(state.pointer_pos, Some((100.0, 200.0)));
        assert_eq!(state.drag_delta(), (0.0, 0.0));
        assert_eq!(state.drag_distance(), 0.0);
        assert!(!state.has_exceeded_drag_threshold(5.0));

        // 2. Move pointer slightly (below threshold)
        state.update_pointer_position((102.0, 201.0));
        assert_eq!(state.drag_delta(), (2.0, 1.0));
        let dist = state.drag_distance();
        assert!((dist - 5.0_f32.sqrt()).abs() < 0.001);
        assert!(!state.has_exceeded_drag_threshold(5.0));

        // 3. Move pointer significantly (exceed threshold)
        state.update_pointer_position((130.0, 240.0));
        assert_eq!(state.drag_delta(), (30.0, 40.0));
        assert_eq!(state.drag_distance(), 50.0);
        assert!(state.has_exceeded_drag_threshold(5.0));
        assert!(state.has_exceeded_drag_threshold(49.0));
        assert!(!state.has_exceeded_drag_threshold(51.0));

        // 4. Update hover target with insertion index
        state.update_hover_with_index(
            Some(DropTarget::ArticleColumn(ArticleStage::Researching)),
            Some(2),
        );
        assert!(state.is_valid_drop());
        assert_eq!(state.drop_insert_index, Some(2));

        // 5. Complete drop clears pointer and insertion states
        let res = state.complete_drop();
        assert!(res.is_some());
        assert!(!state.is_dragging());
        assert_eq!(state.drag_start_pos, None);
        assert_eq!(state.pointer_pos, None);
        assert_eq!(state.drop_insert_index, None);
    }
}
