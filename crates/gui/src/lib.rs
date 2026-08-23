//! `newsjournal-gui`
//!
//! Cross-platform desktop interface and application runtime for NewsJournal.

pub mod commands;
pub mod message;
pub mod navigation;
pub mod platform;
pub mod reducer;
pub mod runtime;
pub mod state;
pub mod theme;
pub mod views;

pub use commands::{AppCommand, CommandExecutor};
pub use message::AppMessage;
pub use navigation::NavTab;
pub use platform::cosmic;
pub use runtime::EventLoop;
pub use state::AppState;
pub use state::{
    ArticleDraft, ContactDraft, DragItem, DragState, DropTarget, FilterState, ModalState,
    SettingsDraft, TaskDraft, ToastKind, ToastMessage, UrgencyFilter,
};
pub use theme::{AppTheme, ColorTokens, GlassMaterial, ResolvedTheme};
pub use views::*;

/// NewsJournal GUI crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
