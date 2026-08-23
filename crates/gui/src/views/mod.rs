//! View models and UI presentation descriptors for NewsJournal.

pub mod articles;
pub mod contacts;
pub mod modal;
pub mod nav;
pub mod settings;
pub mod tasks;
pub mod toast;

pub use articles::{build_articles_kanban_view, ArticleCardViewModel, ArticleColumnViewModel};
pub use contacts::{build_contacts_view, ContactListItemViewModel};
pub use modal::{build_modal_view, ModalViewModel};
pub use nav::{
    build_nav_bar_view, build_nav_bar_view_with_platform, build_nav_item, build_nav_view_models,
    NavBarViewModel, NavItemViewModel,
};
pub use settings::{
    build_settings_view, build_settings_view_with_platform, AppVersionInfo, DatabaseHealthStatus,
    DatabaseStatusViewModel, SettingsViewModel, ThemeOptionViewModel, ThemePreviewColors,
};
pub use tasks::{build_tasks_kanban_view, TaskCardViewModel, TaskColumnViewModel};
pub use toast::{build_toast_view, ToastContainerViewModel};
