//! Action and event messages dispatched through the NewsJournal GUI event loop.

use chrono::{DateTime, Utc};
use newsjournal_core::models::{
    Article, ArticleStage, Contact, Settings, Task, TaskStatus, ThemeMode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::navigation::{NavKeyAction, NavTab};
use crate::state::drag_drop::{DragItem, DropTarget};
use crate::state::filters::UrgencyFilter;
use crate::state::modal::{ArticleDraft, ContactDraft, SettingsDraft, TaskDraft};
use crate::state::toast::ToastMessage;

/// Exhaustive event and action messages processed by the application reducer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AppMessage {
    // ==========================================
    // Navigation
    // ==========================================
    /// Switch active navigation view to specified tab.
    NavigateTo(NavTab),
    /// Cycle to next navigation tab in visual order.
    NavigateNextTab,
    /// Cycle to previous navigation tab in visual order.
    NavigatePrevTab,
    /// Process a resolved keyboard navigation action.
    HandleNavKeyAction(NavKeyAction),

    // ==========================================
    // Modal & Drawer Lifecycle
    // ==========================================
    /// Opens the modal to create a brand new article.
    OpenNewArticleModal,
    /// Opens the drawer to edit an existing article by ID.
    OpenEditArticleModal(Uuid),
    /// Opens the drawer to create a new task, optionally associated with an article.
    OpenNewTaskModal(Option<Uuid>),
    /// Opens the drawer to edit an existing task by ID.
    OpenEditTaskModal(Uuid),
    /// Opens the modal to create a new contact.
    OpenNewContactModal,
    /// Opens the drawer to edit an existing contact by ID.
    OpenEditContactModal(Uuid),
    /// Opens the application settings slide-over drawer.
    OpenSettings,
    /// Opens the confirmation dialog to delete an article.
    PromptDeleteArticle(Uuid),
    /// Opens the confirmation dialog to delete a task.
    PromptDeleteTask(Uuid),
    /// Opens the confirmation dialog to delete a contact.
    PromptDeleteContact(Uuid),
    /// Closes any active modal or drawer.
    CloseModal,
    /// Updates the in-progress draft for an article form.
    UpdateArticleDraft(ArticleDraft),
    /// Updates the in-progress draft for a task form.
    UpdateTaskDraft(TaskDraft),
    /// Updates the in-progress draft for a contact form.
    UpdateContactDraft(ContactDraft),
    /// Updates the in-progress draft for settings.
    UpdateSettingsDraft(SettingsDraft),
    /// Submits the active modal draft for validation and storage persistence.
    SubmitModal,

    // ==========================================
    // Direct Entity CRUD Actions
    // ==========================================
    /// Saves a newly created article.
    CreateArticle(Article),
    /// Updates an existing article.
    UpdateArticle(Article),
    /// Deletes an article by ID.
    DeleteArticle(Uuid),
    /// Moves an article to a different Kanban stage.
    MoveArticleStage(Uuid, ArticleStage),

    /// Saves a newly created task.
    CreateTask(Task),
    /// Updates an existing task.
    UpdateTask(Task),
    /// Deletes a task by ID.
    DeleteTask(Uuid),
    /// Moves a task to a different Kanban status column.
    MoveTaskStatus(Uuid, TaskStatus),
    /// Toggles a task status (ToDo -> InProgress -> Complete -> ToDo).
    ToggleTaskStatus(Uuid),

    /// Saves a newly created contact.
    CreateContact(Contact),
    /// Updates an existing contact.
    UpdateContact(Contact),
    /// Deletes a contact by ID.
    DeleteContact(Uuid),
    /// Links a contact to an article.
    LinkContactToArticle {
        /// Target article ID.
        article_id: Uuid,
        /// Target contact ID.
        contact_id: Uuid,
    },
    /// Unlinks a contact from an article.
    UnlinkContactFromArticle {
        /// Target article ID.
        article_id: Uuid,
        /// Target contact ID.
        contact_id: Uuid,
    },
    /// Atomically updates all contacts linked to an article.
    SetArticleContacts {
        /// Target article ID.
        article_id: Uuid,
        /// List of contact IDs.
        contact_ids: Vec<Uuid>,
    },

    /// Saves updated application settings.
    SaveSettings(Settings),
    /// Updates the theme mode.
    SetThemeMode(ThemeMode),
    /// Toggles active theme mode between light and dark.
    ToggleTheme,
    /// System appearance change notification (is_dark).
    SystemThemeChanged(bool),
    /// Updates high contrast accessibility preference.
    SetHighContrast(bool),
    /// Updates custom brand accent color override (RGB).
    SetCustomAccent(Option<(u8, u8, u8)>),
    /// Resets all application settings to defaults and persists.
    ResetSettingsToDefaults,

    // ==========================================
    // Drag & Drop Interactions
    // ==========================================
    /// Initiates mouse click-and-drag for a card.
    DragStart(DragItem),
    /// Initiates mouse click-and-drag for a card with initial pointer coordinates (x, y).
    DragStartWithPos {
        /// Drag item to start dragging.
        item: DragItem,
        /// Initial pointer coordinates `(x, y)`.
        pos: (f32, f32),
    },
    /// Updates mouse pointer coordinates during active card drag motion.
    DragMove {
        /// Current pointer coordinates `(x, y)`.
        pointer_pos: (f32, f32),
    },
    /// Updates the currently hovered drop column target.
    DragHover(Option<DropTarget>),
    /// Updates the currently hovered drop column target with target insertion index.
    DragHoverWithIndex {
        /// Target column.
        target: Option<DropTarget>,
        /// Optional target insertion index.
        insert_index: Option<usize>,
    },
    /// Drops the currently dragged card into the hovered target column.
    DragDrop,
    /// Cancels active drag session.
    DragCancel,

    // ==========================================
    // Search & Filtering
    // ==========================================
    /// Sets text search query.
    SetSearchQuery(String),
    /// Sets urgency deadline filter.
    SetUrgencyFilter(UrgencyFilter),
    /// Sets stage filter.
    SetStageFilter(Option<ArticleStage>),
    /// Sets contact filter.
    SetContactFilter(Option<Uuid>),
    /// Clears all active filters.
    ClearFilters,

    // ==========================================
    // System, Ticker & Notifications
    // ==========================================
    /// Periodic tick event for deadline evaluation and toast cleanup.
    Tick(DateTime<Utc>),
    /// Reloads all data from storage.
    Refresh,
    /// Dispatches a notification toast.
    PushToast(ToastMessage),
    /// Dismisses a notification toast by ID.
    DismissToast(Uuid),
    /// Sets or clears the top-level error banner.
    SetError(Option<String>),
    /// Window resized event.
    WindowResized {
        /// New width in logical points/pixels.
        width: u32,
        /// New height in logical points/pixels.
        height: u32,
    },
}
