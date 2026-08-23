//! View models and UI presentation descriptors for NewsJournal.

pub mod article_card;
pub mod articles;
pub mod contacts;
pub mod modal;
pub mod nav;
pub mod settings;
pub mod tasks;
pub mod toast;

pub use article_card::{
    calculate_contrast_color, format_contact_initials, format_deadline_badge,
    ArticleCardContactTagViewModel, ArticleCardOverdueStyleViewModel, ArticleCardViewModel,
    ArticleDeadlineBadgeViewModel, ArticleSlugBadgeViewModel, ArticleTaskCounterViewModel,
    ColorIndicatorBarViewModel, DragGhostViewModel, DropPlaceholderViewModel, IndicatorPosition,
    DEFAULT_ACCENT_STRIP_WIDTH, DEFAULT_CARD_BORDER_WIDTH, DEFAULT_CARD_CORNER_RADIUS,
    DUE_SOON_AMBER_HEX, DUE_SOON_BG_TINT_HEX, MAX_DESCRIPTION_SNIPPET_LEN,
    MAX_HEADLINE_SNIPPET_LEN, OVERDUE_BG_TINT_HEX, OVERDUE_CARD_BORDER_WIDTH, OVERDUE_RED_HEX,
    SUCCESS_GREEN_HEX,
};
pub use articles::{
    build_articles_kanban_deck, build_articles_kanban_deck_with_layout, build_articles_kanban_view,
    stage_metadata, ArticleColumnViewModel, ArticlesKanbanDeckViewModel, DeckLayoutConfig,
    StageMetadata, DEFAULT_COLUMN_GAP, DEFAULT_COLUMN_WIDTH, DEFAULT_DECK_PADDING,
    MAX_COLUMN_WIDTH, MIN_COLUMN_WIDTH, NUM_ARTICLE_STAGES,
};
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
