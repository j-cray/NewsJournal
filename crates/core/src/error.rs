//! Error types for `newsjournal-core`.

use thiserror::Error;

use crate::color::ColorError;
use crate::validation::ValidationError;

/// Errors that can occur during domain model parsing and operations.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum ModelError {
    /// Invalid article stage string representation.
    #[error("unknown article stage: {0}")]
    InvalidArticleStage(String),

    /// Invalid task status string representation.
    #[error("unknown task status: {0}")]
    InvalidTaskStatus(String),

    /// Invalid theme mode string representation.
    #[error("unknown theme mode: {0}")]
    InvalidThemeMode(String),

    /// Validation failure on entity fields or format constraints.
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),

    /// Color parsing, conversion, or palette error.
    #[error("color error: {0}")]
    Color(#[from] ColorError),
}
