//! View models and UI presentation descriptors for NewsJournal.

pub mod article_card;
pub mod article_form;
pub mod articles;
pub mod contact_form;
pub mod contacts;
pub mod modal;
pub mod nav;
pub mod settings;
pub mod task_card;
pub mod task_form;
pub mod tasks;
pub mod toast;

pub use contact_form::{
    build_contact_form_view, build_contact_form_view_with_layout,
    ContactAssociatedArticleItemViewModel, ContactAssociatedArticlesSectionViewModel,
    ContactEmailFieldViewModel, ContactFormHeaderViewModel, ContactFormViewModel,
    ContactNameFieldViewModel, ContactNotesFieldViewModel, ContactOrganizationFieldViewModel,
    ContactPhoneFieldViewModel, ContactRoleFieldViewModel, MAX_CONTACT_NAME_LENGTH,
    MAX_CONTACT_ORG_LENGTH, MAX_CONTACT_ROLE_LENGTH,
};

pub use task_form::{
    build_task_form_view, TaskDueDateFieldViewModel, TaskFormHeaderViewModel, TaskFormViewModel,
    TaskNotesFieldViewModel, TaskParentArticleOptionViewModel, TaskParentArticlePickerViewModel,
    TaskStatusOptionViewModel, TaskStatusPickerViewModel, TaskTitleFieldViewModel,
    MAX_TASK_TITLE_LENGTH,
};

pub use article_form::{
    build_article_form_view, ArticleColorPickerViewModel, ArticleDeadlineFieldViewModel,
    ArticleDescriptionFieldViewModel, ArticleFormViewModel, ArticleHeadlineFieldViewModel,
    ArticleSlugFieldViewModel, ArticleStageFieldViewModel, ArticleTasksSectionViewModel,
    ColorSwatchViewModel, ContactPillViewModel, ContactTaggingSectionViewModel, DeadlinePreset,
    DeadlinePresetViewModel, InlineContactFormViewModel, StageOptionViewModel,
    TaskChecklistItemViewModel, DEFAULT_DEADLINE_HOUR, MAX_HEADLINE_LENGTH,
};

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
    stage_metadata, ArticleColumnHeaderViewModel, ArticleColumnViewModel,
    ArticlesKanbanDeckViewModel, ColumnEmptyStateViewModel, DeckEmptyStateViewModel,
    DeckLayoutConfig, KanbanToolbarViewModel, StageMetadata, DEFAULT_COLUMN_GAP,
    DEFAULT_COLUMN_WIDTH, DEFAULT_DECK_PADDING, MAX_COLUMN_WIDTH, MIN_COLUMN_WIDTH,
    NUM_ARTICLE_STAGES,
};
pub use contacts::{
    assign_avatar_color_for_contact, build_contact_column_headers, build_contacts_directory_view,
    build_contacts_view, truncate_snippet, ContactColumnHeaderViewModel,
    ContactLinkedArticleTagViewModel, ContactListItemViewModel, ContactSearchHighlight,
    ContactsDirectoryViewModel, ContactsEmptyStateViewModel, ContactsToolbarViewModel,
    CONTACT_AVATAR_SIZE, DEFAULT_CONTACT_ROW_HEIGHT, MAX_DISPLAYED_TAGGED_ARTICLES,
    MAX_NOTES_SNIPPET_LEN, MAX_ORGANIZATION_SNIPPET_LEN, MAX_ROLE_SNIPPET_LEN,
};
pub use modal::{
    build_modal_container_view, build_modal_container_view_with_layout, build_modal_view,
    ModalBackdropViewModel, ModalContainerViewModel, ModalFooterViewModel, ModalGeometry,
    ModalGlassMaterial, ModalHeaderViewModel, ModalPlacement, ModalViewModel,
};
pub use nav::{
    build_nav_bar_view, build_nav_bar_view_with_platform, build_nav_item, build_nav_view_models,
    NavBarViewModel, NavItemViewModel,
};
pub use settings::{
    build_settings_view, build_settings_view_with_platform, AppVersionInfo, DatabaseHealthStatus,
    DatabaseStatusViewModel, SettingsViewModel, ThemeOptionViewModel, ThemePreviewColors,
};
pub use task_card::{
    format_task_due_date_badge, format_task_notes_snippet, format_task_title_snippet,
    TaskAccentStripViewModel, TaskArticleSlugBadgeViewModel, TaskCardOverdueStyleViewModel,
    TaskCardViewModel, TaskCheckboxViewModel, TaskDueDateBadgeViewModel, TaskNotesPreviewViewModel,
    DEFAULT_TASK_ACCENT_STRIP_WIDTH, DEFAULT_TASK_CARD_BORDER_WIDTH,
    DEFAULT_TASK_CARD_CORNER_RADIUS, DUE_SOON_TASK_CARD_BORDER_WIDTH, MAX_TASK_NOTES_SNIPPET_LEN,
    MAX_TASK_TITLE_SNIPPET_LEN, OVERDUE_TASK_CARD_BORDER_WIDTH, TASK_COMPLETE_BG_TINT_HEX,
    TASK_COMPLETE_GREEN_HEX, TASK_DUE_SOON_AMBER_HEX, TASK_DUE_SOON_BG_TINT_HEX,
    TASK_OVERDUE_BG_TINT_HEX, TASK_OVERDUE_RED_HEX,
};
pub use tasks::{
    build_tasks_kanban_deck, build_tasks_kanban_deck_with_layout, build_tasks_kanban_view,
    task_status_metadata, TaskColumnEmptyStateViewModel, TaskColumnHeaderViewModel,
    TaskColumnViewModel, TaskDragGhostViewModel, TaskDropPlaceholderViewModel, TaskStatusMetadata,
    TasksDeckEmptyStateViewModel, TasksDeckLayoutConfig, TasksKanbanDeckViewModel,
    TasksToolbarViewModel, DEFAULT_TASK_COLUMN_GAP, DEFAULT_TASK_COLUMN_WIDTH,
    DEFAULT_TASK_DECK_PADDING, MAX_TASK_COLUMN_WIDTH, MIN_TASK_COLUMN_WIDTH, NUM_TASK_STATUSES,
    TASK_ACCENT_STRIP_WIDTH,
};
pub use toast::{build_toast_view, ToastContainerViewModel};
