//! In-app modal and drawer view models.

use serde::{Deserialize, Serialize};

use crate::state::modal::ModalState;
use crate::state::AppState;

/// Formatted view descriptor for active in-app modal or slide-over drawer overlay.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModalViewModel {
    /// Whether any modal overlay is active.
    pub is_open: bool,
    /// Modal header title.
    pub title: &'static str,
    /// Active modal state snapshot.
    pub state: ModalState,
}

/// Constructs the active modal view descriptor from application state.
#[must_use]
pub fn build_modal_view(state: &AppState) -> ModalViewModel {
    ModalViewModel {
        is_open: state.modal.is_open(),
        title: state.modal.title(),
        state: state.modal.clone(),
    }
}
