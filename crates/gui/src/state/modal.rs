//! In-app modal and slide-over drawer state machines with draft forms and live validation.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use newsjournal_core::models::{
    Article, ArticleStage, Contact, Settings, Task, TaskStatus, ThemeMode,
};
use newsjournal_core::validation::{
    is_valid_email, is_valid_hex_color, is_valid_phone, is_valid_slug, normalize_email,
    normalize_hex_color, normalize_phone, validate_headline, validate_name, validate_task_title,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Draft form state for creating or editing an Article.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ArticleDraft {
    /// `None` when creating a new article, `Some(id)` when editing an existing one.
    pub id: Option<Uuid>,
    /// Unique identifier slug.
    pub slug: String,
    /// Article headline.
    pub headline: String,
    /// Detailed description or story notes.
    pub description: String,
    /// Production stage.
    pub stage: ArticleStage,
    /// Optional deadline.
    pub deadline: Option<DateTime<Utc>>,
    /// Custom or assigned hex color.
    pub color_hex: String,
    /// IDs of contacts linked to this article.
    pub tagged_contact_ids: Vec<Uuid>,
    /// Validation error messages keyed by field name.
    pub validation_errors: HashMap<String, String>,
}

impl ArticleDraft {
    /// Creates a fresh draft for a new article.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Initializes a draft populated with an existing article's data and tagged contacts.
    #[must_use]
    pub fn from_article(article: &Article, tagged_contact_ids: Vec<Uuid>) -> Self {
        Self {
            id: Some(article.id),
            slug: article.slug.clone(),
            headline: article.headline.clone(),
            description: article.description.clone().unwrap_or_default(),
            stage: article.stage,
            deadline: article.deadline,
            color_hex: article.color.clone().unwrap_or_default(),
            tagged_contact_ids,
            validation_errors: HashMap::new(),
        }
    }

    /// Validates all draft fields and updates `validation_errors`. Returns `true` if valid.
    pub fn validate(&mut self) -> bool {
        self.validation_errors.clear();

        let clean_slug = self.slug.trim();
        if clean_slug.is_empty() {
            self.validation_errors
                .insert("slug".to_string(), "Slug cannot be empty".to_string());
        } else if !is_valid_slug(clean_slug) {
            self.validation_errors.insert(
                "slug".to_string(),
                "Slug must contain only lowercase letters, digits, and hyphens".to_string(),
            );
        }

        if let Err(e) = validate_headline(&self.headline) {
            self.validation_errors
                .insert("headline".to_string(), e.to_string());
        }

        let clean_color = self.color_hex.trim();
        if !clean_color.is_empty() && !is_valid_hex_color(clean_color) {
            self.validation_errors.insert(
                "color_hex".to_string(),
                "Color must be a valid #RRGGBB hex code".to_string(),
            );
        }

        self.validation_errors.is_empty()
    }

    /// Converts this draft into a domain [`Article`] model if valid.
    pub fn to_article(&mut self) -> Result<Article, String> {
        if !self.validate() {
            return Err("Article draft contains validation errors".to_string());
        }

        let id = self.id.unwrap_or_else(Uuid::new_v4);
        let color = if self.color_hex.trim().is_empty() {
            None
        } else {
            normalize_hex_color(&self.color_hex).ok()
        };

        let description = if self.description.trim().is_empty() {
            None
        } else {
            Some(self.description.trim().to_string())
        };

        let mut article = Article::new(&self.slug, &self.headline);
        article.id = id;
        article.description = description;
        article.stage = self.stage;
        article.deadline = self.deadline;
        article.color = color;

        Ok(article)
    }
}

/// Draft form state for creating or editing a Task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TaskDraft {
    /// `None` when creating a new task, `Some(id)` when editing an existing one.
    pub id: Option<Uuid>,
    /// Parent article ID.
    pub article_id: Option<Uuid>,
    /// Task title.
    pub title: String,
    /// Detailed notes.
    pub notes: String,
    /// Optional due date.
    pub due_date: Option<DateTime<Utc>>,
    /// Status.
    pub status: TaskStatus,
    /// Validation error messages keyed by field name.
    pub validation_errors: HashMap<String, String>,
}

impl TaskDraft {
    /// Creates a draft initialized for a specific parent article.
    #[must_use]
    pub fn new_for_article(article_id: Option<Uuid>) -> Self {
        Self {
            article_id,
            ..Default::default()
        }
    }

    /// Initializes a draft populated with an existing task's data.
    #[must_use]
    pub fn from_task(task: &Task) -> Self {
        Self {
            id: Some(task.id),
            article_id: Some(task.article_id),
            title: task.title.clone(),
            notes: task.notes.clone().unwrap_or_default(),
            due_date: task.due_date,
            status: task.status,
            validation_errors: HashMap::new(),
        }
    }

    /// Validates draft fields and updates `validation_errors`. Returns `true` if valid.
    pub fn validate(&mut self) -> bool {
        self.validation_errors.clear();

        if self.article_id.is_none() {
            self.validation_errors.insert(
                "article_id".to_string(),
                "Parent article must be selected".to_string(),
            );
        }

        if let Err(e) = validate_task_title(&self.title) {
            self.validation_errors
                .insert("title".to_string(), e.to_string());
        }

        self.validation_errors.is_empty()
    }

    /// Converts this draft into a domain [`Task`] model if valid.
    pub fn to_task(&mut self) -> Result<Task, String> {
        if !self.validate() {
            return Err("Task draft contains validation errors".to_string());
        }

        let article_id = self
            .article_id
            .ok_or_else(|| "Missing parent article ID".to_string())?;
        let id = self.id.unwrap_or_else(Uuid::new_v4);

        let notes = if self.notes.trim().is_empty() {
            None
        } else {
            Some(self.notes.trim().to_string())
        };

        let mut task = Task::new(article_id, &self.title);
        task.id = id;
        task.notes = notes;
        task.due_date = self.due_date;
        task.status = self.status;

        Ok(task)
    }
}

/// Draft form state for creating or editing a Contact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ContactDraft {
    /// `None` when creating a new contact, `Some(id)` when editing an existing one.
    pub id: Option<Uuid>,
    /// Full contact name (required).
    pub name: String,
    /// Organization / Outlet.
    pub organization: String,
    /// Role or Beat.
    pub role: String,
    /// Phone number.
    pub phone: String,
    /// Email address.
    pub email: String,
    /// Background notes.
    pub notes: String,
    /// Validation error messages keyed by field name.
    pub validation_errors: HashMap<String, String>,
}

impl ContactDraft {
    /// Creates a fresh draft for a new contact.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Initializes a draft populated with an existing contact's data.
    #[must_use]
    pub fn from_contact(contact: &Contact) -> Self {
        Self {
            id: Some(contact.id),
            name: contact.name.clone(),
            organization: contact.organization.clone().unwrap_or_default(),
            role: contact.role.clone().unwrap_or_default(),
            phone: contact.phone.clone().unwrap_or_default(),
            email: contact.email.clone().unwrap_or_default(),
            notes: contact.notes.clone().unwrap_or_default(),
            validation_errors: HashMap::new(),
        }
    }

    /// Validates draft fields and updates `validation_errors`. Returns `true` if valid.
    pub fn validate(&mut self) -> bool {
        self.validation_errors.clear();

        if let Err(e) = validate_name(&self.name) {
            self.validation_errors
                .insert("name".to_string(), e.to_string());
        }

        let clean_email = self.email.trim();
        if !clean_email.is_empty() && !is_valid_email(clean_email) {
            self.validation_errors.insert(
                "email".to_string(),
                "Email address is not in a valid format".to_string(),
            );
        }

        let clean_phone = self.phone.trim();
        if !clean_phone.is_empty() && !is_valid_phone(clean_phone) {
            self.validation_errors.insert(
                "phone".to_string(),
                "Phone number must contain between 7 and 15 digits".to_string(),
            );
        }

        self.validation_errors.is_empty()
    }

    /// Converts this draft into a domain [`Contact`] model if valid.
    pub fn to_contact(&mut self) -> Result<Contact, String> {
        if !self.validate() {
            return Err("Contact draft contains validation errors".to_string());
        }

        let id = self.id.unwrap_or_else(Uuid::new_v4);
        let mut contact = Contact::new(&self.name);
        contact.id = id;
        contact.organization = if self.organization.trim().is_empty() {
            None
        } else {
            Some(self.organization.trim().to_string())
        };
        contact.role = if self.role.trim().is_empty() {
            None
        } else {
            Some(self.role.trim().to_string())
        };
        contact.phone = normalize_phone(&self.phone).ok();
        contact.email = normalize_email(&self.email).ok();
        contact.notes = if self.notes.trim().is_empty() {
            None
        } else {
            Some(self.notes.trim().to_string())
        };

        Ok(contact)
    }
}

/// Draft state for the Settings drawer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingsDraft {
    /// Active theme mode.
    pub theme_mode: ThemeMode,
    /// High contrast accessibility flag.
    pub high_contrast: bool,
    /// Custom brand accent RGB override.
    pub custom_accent: Option<(u8, u8, u8)>,
}

impl SettingsDraft {
    /// Creates a settings draft from active settings.
    #[must_use]
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            theme_mode: settings.theme_mode,
            high_contrast: false,
            custom_accent: None,
        }
    }

    /// Creates a settings draft with explicit theme options.
    #[must_use]
    pub fn with_options(
        settings: &Settings,
        high_contrast: bool,
        custom_accent: Option<(u8, u8, u8)>,
    ) -> Self {
        Self {
            theme_mode: settings.theme_mode,
            high_contrast,
            custom_accent,
        }
    }

    /// Converts the draft to a domain `Settings` model.
    #[must_use]
    pub fn to_settings(&self) -> Settings {
        Settings::new(self.theme_mode)
    }
}

/// Active modal or slide-over drawer overlay state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ModalState {
    /// No modal is open.
    #[default]
    None,
    /// Article creation or editing drawer.
    ArticleForm(ArticleDraft),
    /// Task creation or editing drawer.
    TaskForm(TaskDraft),
    /// Contact creation or editing drawer.
    ContactForm(ContactDraft),
    /// Settings slide-over drawer.
    SettingsDrawer(SettingsDraft),
    /// Confirmation dialog for deleting an article.
    ConfirmDeleteArticle {
        /// Article ID to delete.
        id: Uuid,
        /// Article slug.
        slug: String,
        /// Article headline.
        headline: String,
    },
    /// Confirmation dialog for deleting a task.
    ConfirmDeleteTask {
        /// Task ID to delete.
        id: Uuid,
        /// Task title.
        title: String,
    },
    /// Confirmation dialog for deleting a contact.
    ConfirmDeleteContact {
        /// Contact ID to delete.
        id: Uuid,
        /// Contact name.
        name: String,
        /// Number of articles linked to this contact.
        linked_article_count: usize,
    },
}

impl ModalState {
    /// Returns `true` if any modal or drawer is currently displayed.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Closes the modal.
    pub fn close(&mut self) {
        *self = Self::None;
    }

    /// Returns a user-visible header title for the open modal.
    #[must_use]
    pub fn title(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::ArticleForm(draft) => {
                if draft.id.is_some() {
                    "Edit Article"
                } else {
                    "New Article"
                }
            }
            Self::TaskForm(draft) => {
                if draft.id.is_some() {
                    "Edit Task"
                } else {
                    "New Task"
                }
            }
            Self::ContactForm(draft) => {
                if draft.id.is_some() {
                    "Edit Contact"
                } else {
                    "New Contact"
                }
            }
            Self::SettingsDrawer(_) => "Application Settings",
            Self::ConfirmDeleteArticle { .. } => "Delete Article?",
            Self::ConfirmDeleteTask { .. } => "Delete Task?",
            Self::ConfirmDeleteContact { .. } => "Delete Contact?",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_article_draft_validation_and_conversion() {
        let mut draft = ArticleDraft::new();
        assert!(!draft.validate());
        assert!(draft.validation_errors.contains_key("slug"));
        assert!(draft.validation_errors.contains_key("headline"));

        draft.slug = "city-hall-probe".to_string();
        draft.headline = "City Hall Probe Launched".to_string();
        draft.color_hex = "#1A85FF".to_string();
        assert!(draft.validate());

        let article = draft.to_article().expect("valid article");
        assert_eq!(article.slug, "city-hall-probe");
        assert_eq!(article.headline, "City Hall Probe Launched");
        assert_eq!(article.color, Some("#1A85FF".to_string()));
    }

    #[test]
    fn test_task_draft_validation_and_conversion() {
        let article_id = Uuid::new_v4();
        let mut draft = TaskDraft::new_for_article(Some(article_id));
        assert!(!draft.validate());

        draft.title = "Call whistleblower".to_string();
        assert!(draft.validate());

        let task = draft.to_task().expect("valid task");
        assert_eq!(task.article_id, article_id);
        assert_eq!(task.title, "Call whistleblower");
    }

    #[test]
    fn test_contact_draft_validation_and_conversion() {
        let mut draft = ContactDraft::new();
        assert!(!draft.validate());

        draft.name = "Jane Doe".to_string();
        draft.email = "jane@news.org".to_string();
        draft.phone = "+1 (555) 234-5678".to_string();
        assert!(draft.validate());

        let contact = draft.to_contact().expect("valid contact");
        assert_eq!(contact.name, "Jane Doe");
        assert_eq!(contact.email, Some("jane@news.org".to_string()));
        assert_eq!(contact.phone, Some("+15552345678".to_string()));
    }

    #[test]
    fn test_modal_state_lifecycle() {
        let mut modal = ModalState::None;
        assert!(!modal.is_open());
        assert_eq!(modal.title(), "");

        modal = ModalState::ArticleForm(ArticleDraft::new());
        assert!(modal.is_open());
        assert_eq!(modal.title(), "New Article");

        modal.close();
        assert!(!modal.is_open());
    }
}
