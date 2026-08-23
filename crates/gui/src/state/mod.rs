//! Central application state for NewsJournal desktop.

pub mod drag_drop;
pub mod filters;
pub mod modal;
pub mod toast;

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use newsjournal_core::deadline::{evaluate_article_deadline, summarize_articles, DeadlineSummary};
use newsjournal_core::models::{Article, ArticleStage, Contact, Settings, Task, TaskStatus};
use newsjournal_core::storage::{StorageError, StorageService};
use uuid::Uuid;

pub use drag_drop::{DragItem, DragState, DropTarget};
pub use filters::{FilterState, UrgencyFilter};
pub use modal::{ArticleDraft, ContactDraft, ModalState, SettingsDraft, TaskDraft};
pub use toast::{ToastKind, ToastMessage};

use crate::navigation::NavTab;
use crate::theme::{AppTheme, ResolvedTheme, ThemeEngine};

/// Unified in-memory state tree powering the NewsJournal user interface.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Thread-safe persistence service.
    pub storage: StorageService,
    /// Currently active navigation tab.
    pub active_tab: NavTab,
    /// Cached list of all articles in the system.
    pub articles: Vec<Article>,
    /// Cached list of all tasks in the system.
    pub tasks: Vec<Task>,
    /// Cached list of all contacts in the directory.
    pub contacts: Vec<Contact>,
    /// Map of article IDs to linked contact IDs.
    pub article_contacts: HashMap<Uuid, Vec<Uuid>>,
    /// Map of contact IDs to linked article IDs.
    pub contact_articles: HashMap<Uuid, Vec<Uuid>>,
    /// User settings (theme mode, etc.).
    pub settings: Settings,
    /// Unified dynamic theme engine.
    pub theme_engine: ThemeEngine,
    /// Active modal / slide-over drawer overlay state.
    pub modal: ModalState,
    /// Active card drag-and-drop session state.
    pub drag: DragState,
    /// Active search queries and filters.
    pub filters: FilterState,
    /// Active in-app toast notification queue.
    pub toasts: Vec<ToastMessage>,
    /// Timestamp of last deadline and state evaluation tick.
    pub last_tick: DateTime<Utc>,
    /// Calculated deadline metrics and urgency summary across all articles.
    pub deadline_summary: DeadlineSummary,
    /// Persistent top-level error banner message, if any.
    pub error_banner: Option<String>,
    /// Global loading indicator flag for async operations.
    pub is_loading: bool,
}

impl AppState {
    /// Creates a new `AppState` instance initialized with the given storage backend.
    #[must_use]
    pub fn new(storage: StorageService) -> Self {
        let now = Utc::now();
        let settings = Settings::default();
        let theme_engine = ThemeEngine::new(settings.theme_mode, true);
        Self {
            storage,
            active_tab: NavTab::ArticlesKanban,
            articles: Vec::new(),
            tasks: Vec::new(),
            contacts: Vec::new(),
            article_contacts: HashMap::new(),
            contact_articles: HashMap::new(),
            settings,
            theme_engine,
            modal: ModalState::None,
            drag: DragState::new(),
            filters: FilterState::new(),
            toasts: Vec::new(),
            last_tick: now,
            deadline_summary: DeadlineSummary::default(),
            error_banner: None,
            is_loading: false,
        }
    }

    /// Initializes a transient in-memory `AppState` suitable for tests and prototyping.
    pub fn in_memory() -> Result<Self, StorageError> {
        let storage = StorageService::in_memory()?;
        let mut state = Self::new(storage);
        state.load_all()?;
        Ok(state)
    }

    /// Loads all articles, tasks, contacts, tags, and settings from storage into memory cache.
    pub fn load_all(&mut self) -> Result<(), StorageError> {
        self.is_loading = true;
        let articles = self.storage.list_articles()?;
        let tasks = self.storage.list_tasks()?;
        let contacts = self.storage.list_contacts()?;
        let settings = self.storage.get_settings()?;

        let mut article_contacts = HashMap::new();
        let mut contact_articles = HashMap::new();

        for article in &articles {
            let tagged = self.storage.list_contacts_for_article(article.id)?;
            let contact_ids: Vec<Uuid> = tagged.into_iter().map(|c| c.id).collect();
            for &cid in &contact_ids {
                contact_articles
                    .entry(cid)
                    .or_insert_with(Vec::new)
                    .push(article.id);
            }
            article_contacts.insert(article.id, contact_ids);
        }

        self.theme_engine.set_mode(settings.theme_mode);
        self.articles = articles;
        self.tasks = tasks;
        self.contacts = contacts;
        self.settings = settings;
        self.article_contacts = article_contacts;
        self.contact_articles = contact_articles;
        self.is_loading = false;

        self.recalculate_deadlines(Utc::now());
        Ok(())
    }

    /// Returns a reference to the active resolved theme.
    #[must_use]
    pub fn theme(&self) -> &AppTheme {
        self.theme_engine.theme()
    }

    /// Returns the active resolved theme variant (`Light` or `Dark`).
    #[must_use]
    pub fn resolved_theme(&self) -> ResolvedTheme {
        self.theme_engine.resolved()
    }

    /// Returns whether the resolved theme is dark.
    #[must_use]
    pub fn is_dark(&self) -> bool {
        self.theme_engine.is_dark()
    }

    /// Recalculates deadline urgency summaries for all articles.
    pub fn recalculate_deadlines(&mut self, now: DateTime<Utc>) {
        self.last_tick = now;
        self.deadline_summary = summarize_articles(&self.articles, now);
    }

    // ==========================================
    // Query & Selector Helpers
    // ==========================================

    /// Returns articles assigned to a specific production stage.
    #[must_use]
    pub fn articles_in_stage(&self, stage: ArticleStage) -> Vec<&Article> {
        self.articles.iter().filter(|a| a.stage == stage).collect()
    }

    /// Returns tasks assigned to a specific status column.
    #[must_use]
    pub fn tasks_in_status(&self, status: TaskStatus) -> Vec<&Task> {
        self.tasks.iter().filter(|t| t.status == status).collect()
    }

    /// Returns all tasks belonging to a specific parent article.
    #[must_use]
    pub fn tasks_for_article(&self, article_id: Uuid) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| t.article_id == article_id)
            .collect()
    }

    /// Returns `(completed_count, total_count)` of tasks for a parent article.
    #[must_use]
    pub fn task_completion_stats(&self, article_id: Uuid) -> (usize, usize) {
        let tasks = self.tasks_for_article(article_id);
        let total = tasks.len();
        let completed = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Complete)
            .count();
        (completed, total)
    }

    /// Returns all contacts tagged on a specific article.
    #[must_use]
    pub fn contacts_for_article(&self, article_id: Uuid) -> Vec<&Contact> {
        if let Some(contact_ids) = self.article_contacts.get(&article_id) {
            contact_ids
                .iter()
                .filter_map(|&id| self.contacts.iter().find(|c| c.id == id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Returns all articles tagged with a specific contact.
    #[must_use]
    pub fn articles_for_contact(&self, contact_id: Uuid) -> Vec<&Article> {
        if let Some(article_ids) = self.contact_articles.get(&contact_id) {
            article_ids
                .iter()
                .filter_map(|&id| self.articles.iter().find(|a| a.id == id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Finds an article by unique ID.
    #[must_use]
    pub fn get_article(&self, id: Uuid) -> Option<&Article> {
        self.articles.iter().find(|a| a.id == id)
    }

    /// Finds a task by unique ID.
    #[must_use]
    pub fn get_task(&self, id: Uuid) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }

    /// Finds a contact by unique ID.
    #[must_use]
    pub fn get_contact(&self, id: Uuid) -> Option<&Contact> {
        self.contacts.iter().find(|c| c.id == id)
    }

    /// Returns contacts filtered by the active search query.
    #[must_use]
    pub fn filtered_contacts(&self) -> Vec<&Contact> {
        let query = self.filters.search_query.trim().to_lowercase();
        if query.is_empty() {
            self.contacts.iter().collect()
        } else {
            self.contacts
                .iter()
                .filter(|c| {
                    c.name.to_lowercase().contains(&query)
                        || c.organization
                            .as_deref()
                            .map(|o| o.to_lowercase().contains(&query))
                            .unwrap_or(false)
                        || c.role
                            .as_deref()
                            .map(|r| r.to_lowercase().contains(&query))
                            .unwrap_or(false)
                        || c.email
                            .as_deref()
                            .map(|e| e.to_lowercase().contains(&query))
                            .unwrap_or(false)
                        || c.phone
                            .as_deref()
                            .map(|p| p.contains(&query))
                            .unwrap_or(false)
                })
                .collect()
        }
    }

    /// Returns articles filtered by search query, stage, and urgency filters.
    #[must_use]
    pub fn filtered_articles(&self) -> Vec<&Article> {
        let query = self.filters.search_query.trim().to_lowercase();
        let now = self.last_tick;

        self.articles
            .iter()
            .filter(|a| {
                if let Some(stage) = self.filters.selected_stage {
                    if a.stage != stage {
                        return false;
                    }
                }

                if let Some(cid) = self.filters.selected_contact_id {
                    let has_contact = self
                        .article_contacts
                        .get(&a.id)
                        .map(|cids| cids.contains(&cid))
                        .unwrap_or(false);
                    if !has_contact {
                        return false;
                    }
                }

                if !query.is_empty() {
                    let match_text = a.headline.to_lowercase().contains(&query)
                        || a.slug.to_lowercase().contains(&query)
                        || a.description
                            .as_deref()
                            .map(|d| d.to_lowercase().contains(&query))
                            .unwrap_or(false);
                    if !match_text {
                        return false;
                    }
                }

                match self.filters.urgency_filter {
                    UrgencyFilter::All => true,
                    UrgencyFilter::OverdueOnly => evaluate_article_deadline(a, now).is_overdue(),
                    UrgencyFilter::DueSoonOnly => evaluate_article_deadline(a, now).is_due_soon(),
                    UrgencyFilter::HasDeadlineOnly => a.deadline.is_some(),
                }
            })
            .collect()
    }

    /// Returns tasks filtered by search query, parent article, and urgency filters.
    #[must_use]
    pub fn filtered_tasks(&self) -> Vec<&Task> {
        let query = self.filters.search_query.trim().to_lowercase();
        let now = self.last_tick;

        self.tasks
            .iter()
            .filter(|t| {
                if let Some(target_aid) = self.filters.selected_article_id {
                    if t.article_id != target_aid {
                        return false;
                    }
                }

                if !query.is_empty() {
                    let parent = self.get_article(t.article_id);
                    let title_match = t.title.to_lowercase().contains(&query);
                    let notes_match = t
                        .notes
                        .as_deref()
                        .map(|n| n.to_lowercase().contains(&query))
                        .unwrap_or(false);
                    let parent_match = parent
                        .map(|a| {
                            a.slug.to_lowercase().contains(&query)
                                || a.headline.to_lowercase().contains(&query)
                        })
                        .unwrap_or(false);

                    if !title_match && !notes_match && !parent_match {
                        return false;
                    }
                }

                match self.filters.urgency_filter {
                    UrgencyFilter::All => true,
                    UrgencyFilter::OverdueOnly => t.is_overdue(now),
                    UrgencyFilter::DueSoonOnly => t.is_due_soon(now),
                    UrgencyFilter::HasDeadlineOnly => t.due_date.is_some(),
                }
            })
            .collect()
    }

    // ==========================================
    // Notification & Alert Management
    // ==========================================

    /// Appends a new toast notification to the queue.
    pub fn push_toast(&mut self, toast: ToastMessage) {
        self.toasts.push(toast);
    }

    /// Removes a toast notification by its unique ID.
    pub fn dismiss_toast(&mut self, id: Uuid) {
        self.toasts.retain(|t| t.id != id);
    }

    /// Purges all expired toast notifications from the queue.
    pub fn purge_expired_toasts(&mut self, now: DateTime<Utc>) {
        self.toasts.retain(|t| !t.is_expired(now));
    }

    /// Sets or clears the top-level error banner.
    pub fn set_error(&mut self, error: Option<String>) {
        self.error_banner = error;
    }

    // ==========================================
    // Navigation State Transitions
    // ==========================================

    /// Switches the active navigation view to the specified tab.
    pub fn navigate_to(&mut self, tab: NavTab) {
        self.active_tab = tab;
    }

    /// Cycles the active navigation view to the next tab in visual order.
    pub fn navigate_next(&mut self) {
        self.active_tab = self.active_tab.next();
    }

    /// Cycles the active navigation view to the previous tab in visual order.
    pub fn navigate_prev(&mut self) {
        self.active_tab = self.active_tab.prev();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use newsjournal_core::models::{ArticleBuilder, ContactBuilder, TaskBuilder};

    #[test]
    fn test_app_state_initialization_and_queries() {
        let mut state = AppState::in_memory().expect("in-memory state");
        assert_eq!(state.active_tab, NavTab::ArticlesKanban);
        assert!(state.articles.is_empty());
        assert!(state.tasks.is_empty());
        assert!(state.contacts.is_empty());

        // Create sample domain entities
        let article = ArticleBuilder::new("flood-coverage", "Spring Flood Coverage")
            .stage(ArticleStage::Writing)
            .build();
        let saved_article = state.storage.create_article(article).unwrap();

        let task1 = TaskBuilder::new(saved_article.id, "Interview Mayor")
            .status(TaskStatus::Complete)
            .build();
        let task2 = TaskBuilder::new(saved_article.id, "Map evacuation zones")
            .status(TaskStatus::InProgress)
            .build();
        state.storage.create_task(task1).unwrap();
        state.storage.create_task(task2).unwrap();

        let contact = ContactBuilder::new("Mayor Smith")
            .organization("City Hall")
            .build();
        let saved_contact = state.storage.create_contact(contact).unwrap();
        state
            .storage
            .link_contact_to_article(saved_article.id, saved_contact.id)
            .unwrap();

        // Reload state cache
        state.load_all().unwrap();

        assert_eq!(state.articles.len(), 1);
        assert_eq!(state.tasks.len(), 2);
        assert_eq!(state.contacts.len(), 1);

        assert_eq!(state.articles_in_stage(ArticleStage::Writing).len(), 1);
        assert_eq!(state.articles_in_stage(ArticleStage::Pitching).len(), 0);

        let (completed, total) = state.task_completion_stats(saved_article.id);
        assert_eq!((completed, total), (1, 2));

        let linked_contacts = state.contacts_for_article(saved_article.id);
        assert_eq!(linked_contacts.len(), 1);
        assert_eq!(linked_contacts[0].name, "Mayor Smith");

        let linked_articles = state.articles_for_contact(saved_contact.id);
        assert_eq!(linked_articles.len(), 1);
        assert_eq!(linked_articles[0].slug, "flood-coverage");
    }

    #[test]
    fn test_toast_queue_lifecycle() {
        let mut state = AppState::in_memory().unwrap();
        let toast1 = ToastMessage::info("Notice 1", "Body 1");
        let id1 = toast1.id;
        let toast2 = ToastMessage::error("Error 2", "Body 2");

        state.push_toast(toast1);
        state.push_toast(toast2);
        assert_eq!(state.toasts.len(), 2);

        state.dismiss_toast(id1);
        assert_eq!(state.toasts.len(), 1);
        assert_eq!(state.toasts[0].title, "Error 2");

        let future = Utc::now() + Duration::seconds(10);
        state.purge_expired_toasts(future);
        assert_eq!(state.toasts.len(), 0);
    }
}
