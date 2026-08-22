//! Toast notification view models.

use serde::{Deserialize, Serialize};

use crate::state::toast::ToastMessage;
use crate::state::AppState;

/// Formatted view descriptor for rendering active toast alerts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToastContainerViewModel {
    /// Active toasts to render in the bottom-right viewport.
    pub toasts: Vec<ToastMessage>,
    /// Global error banner message, if any.
    pub error_banner: Option<String>,
}

/// Constructs the toast container view descriptor from application state.
#[must_use]
pub fn build_toast_view(state: &AppState) -> ToastContainerViewModel {
    ToastContainerViewModel {
        toasts: state.toasts.clone(),
        error_banner: state.error_banner.clone(),
    }
}
