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

use newsjournal_core::color::assign_color_for_slug;
use newsjournal_core::validation::slugify;

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
    /// Whether the color is a user-chosen custom override instead of auto-hash assigned.
    pub is_custom_color: bool,
    /// IDs of contacts linked to this article.
    pub tagged_contact_ids: Vec<Uuid>,
    /// Active contact search query for filtering the contact picker.
    pub contact_search_query: String,
    /// In-progress draft for inline contact creation, if open.
    pub inline_contact: Option<Box<ContactDraft>>,
    /// Validation error messages keyed by field name.
    pub validation_errors: HashMap<String, String>,
}

impl ArticleDraft {
    /// Creates a fresh draft for a new article with default color.
    #[must_use]
    pub fn new() -> Self {
        let default_color = assign_color_for_slug("new-article").to_hex();
        Self {
            color_hex: default_color,
            ..Default::default()
        }
    }

    /// Creates a fresh draft initialized with a specific production stage.
    #[must_use]
    pub fn new_with_stage(stage: ArticleStage) -> Self {
        let mut draft = Self::new();
        draft.stage = stage;
        draft
    }

    /// Initializes a draft populated with an existing article's data and tagged contacts.
    #[must_use]
    pub fn from_article(article: &Article, tagged_contact_ids: Vec<Uuid>) -> Self {
        let is_custom_color = article.color.is_some();
        let color_hex = article
            .color
            .clone()
            .unwrap_or_else(|| assign_color_for_slug(&article.slug).to_hex());

        Self {
            id: Some(article.id),
            slug: article.slug.clone(),
            headline: article.headline.clone(),
            description: article.description.clone().unwrap_or_default(),
            stage: article.stage,
            deadline: article.deadline,
            color_hex,
            is_custom_color,
            tagged_contact_ids,
            contact_search_query: String::new(),
            inline_contact: None,
            validation_errors: HashMap::new(),
        }
    }

    /// Updates the slug field and synchronizes the color if not custom-overridden.
    pub fn set_slug(&mut self, slug: &str) {
        self.slug = slug.to_string();
        let clean = self.slug.trim();
        if clean.is_empty() {
            self.validation_errors
                .insert("slug".to_string(), "Slug cannot be empty".to_string());
        } else if !is_valid_slug(clean) {
            self.validation_errors.insert(
                "slug".to_string(),
                "Slug must contain only lowercase letters, digits, and hyphens (no spaces or double hyphens)".to_string(),
            );
        } else {
            self.validation_errors.remove("slug");
        }

        if !self.is_custom_color {
            let key = if clean.is_empty() {
                "new-article"
            } else {
                clean
            };
            self.color_hex = assign_color_for_slug(key).to_hex();
        }
    }

    /// Updates the headline field, optionally auto-generating the slug for new articles.
    pub fn set_headline(&mut self, headline: &str, auto_slug: bool) {
        let old_auto_slug = slugify(&self.headline);
        let should_auto_slug = auto_slug
            && self.id.is_none()
            && (self.slug.is_empty() || self.slug == old_auto_slug || self.slug == "new-article");

        self.headline = headline.to_string();
        if let Err(e) = validate_headline(&self.headline) {
            self.validation_errors
                .insert("headline".to_string(), e.to_string());
        } else {
            self.validation_errors.remove("headline");
        }

        if should_auto_slug {
            self.set_slug(&slugify(headline));
        }
    }

    /// Updates the story description / background notes.
    pub fn set_description(&mut self, description: &str) {
        self.description = description.to_string();
    }

    /// Updates the production stage.
    pub fn set_stage(&mut self, stage: ArticleStage) {
        self.stage = stage;
    }

    /// Updates the deadline timestamp.
    pub fn set_deadline(&mut self, deadline: Option<DateTime<Utc>>) {
        self.deadline = deadline;
    }

    /// Sets an explicit custom color override.
    pub fn set_color(&mut self, color_hex: &str) {
        self.color_hex = color_hex.trim().to_string();
        self.is_custom_color = true;

        let clean = self.color_hex.trim();
        if !clean.is_empty() && !is_valid_hex_color(clean) {
            self.validation_errors.insert(
                "color_hex".to_string(),
                "Color must be a valid #RRGGBB hex code".to_string(),
            );
        } else {
            self.validation_errors.remove("color_hex");
        }
    }

    /// Resets the color to the deterministic hash-based palette color derived from the slug.
    pub fn reset_color_to_hash(&mut self) {
        self.is_custom_color = false;
        self.validation_errors.remove("color_hex");
        let clean = self.slug.trim();
        let key = if clean.is_empty() {
            "new-article"
        } else {
            clean
        };
        self.color_hex = assign_color_for_slug(key).to_hex();
    }

    /// Tags a contact if not already tagged.
    pub fn tag_contact(&mut self, contact_id: Uuid) {
        if !self.tagged_contact_ids.contains(&contact_id) {
            self.tagged_contact_ids.push(contact_id);
        }
    }

    /// Removes a tagged contact by ID.
    pub fn untag_contact(&mut self, contact_id: Uuid) {
        self.tagged_contact_ids.retain(|id| *id != contact_id);
    }

    /// Toggles a contact's tagged state.
    pub fn toggle_contact(&mut self, contact_id: Uuid) {
        if self.tagged_contact_ids.contains(&contact_id) {
            self.untag_contact(contact_id);
        } else {
            self.tag_contact(contact_id);
        }
    }

    /// Returns `true` if the given contact is currently tagged.
    #[must_use]
    pub fn is_contact_tagged(&self, contact_id: Uuid) -> bool {
        self.tagged_contact_ids.contains(&contact_id)
    }

    /// Sets the search filter for the contact picker.
    pub fn set_contact_search(&mut self, query: &str) {
        self.contact_search_query = query.to_string();
    }

    /// Clears the contact picker search filter.
    pub fn clear_contact_search(&mut self) {
        self.contact_search_query.clear();
    }

    /// Opens the inline contact creation sub-form with a fresh blank draft.
    pub fn open_inline_contact(&mut self) {
        self.inline_contact = Some(Box::new(ContactDraft::new()));
    }

    /// Closes and cancels the inline contact creation sub-form.
    pub fn close_inline_contact(&mut self) {
        self.inline_contact = None;
    }

    /// Toggles the inline contact creation sub-form open/closed state.
    pub fn toggle_inline_contact(&mut self) {
        if self.inline_contact.is_some() {
            self.inline_contact = None;
        } else {
            self.inline_contact = Some(Box::new(ContactDraft::new()));
        }
    }

    /// Returns `true` if the inline contact creation sub-form is open.
    #[must_use]
    pub const fn is_inline_contact_open(&self) -> bool {
        self.inline_contact.is_some()
    }

    /// Updates the name field of the inline contact sub-form.
    pub fn set_inline_contact_name(&mut self, name: &str) {
        if let Some(ref mut contact_draft) = self.inline_contact {
            contact_draft.name = name.to_string();
            contact_draft.validate();
        }
    }

    /// Updates the organization field of the inline contact sub-form.
    pub fn set_inline_contact_org(&mut self, org: &str) {
        if let Some(ref mut contact_draft) = self.inline_contact {
            contact_draft.organization = org.to_string();
        }
    }

    /// Updates the role field of the inline contact sub-form.
    pub fn set_inline_contact_role(&mut self, role: &str) {
        if let Some(ref mut contact_draft) = self.inline_contact {
            contact_draft.role = role.to_string();
        }
    }

    /// Updates the email field of the inline contact sub-form.
    pub fn set_inline_contact_email(&mut self, email: &str) {
        if let Some(ref mut contact_draft) = self.inline_contact {
            contact_draft.email = email.to_string();
            contact_draft.validate();
        }
    }

    /// Updates the phone field of the inline contact sub-form.
    pub fn set_inline_contact_phone(&mut self, phone: &str) {
        if let Some(ref mut contact_draft) = self.inline_contact {
            contact_draft.phone = phone.to_string();
            contact_draft.validate();
        }
    }

    /// Updates the notes field of the inline contact sub-form.
    pub fn set_inline_contact_notes(&mut self, notes: &str) {
        if let Some(ref mut contact_draft) = self.inline_contact {
            contact_draft.notes = notes.to_string();
        }
    }

    /// Checks whether the current slug collides with any existing articles (excluding this article's ID).
    #[must_use]
    pub fn slug_collides_with(&self, existing_articles: &[(Uuid, String)]) -> bool {
        let clean_slug = self.slug.trim();
        if clean_slug.is_empty() {
            return false;
        }
        existing_articles
            .iter()
            .any(|(id, slug)| slug.trim().eq_ignore_ascii_case(clean_slug) && Some(*id) != self.id)
    }

    /// Returns `true` if there are currently no validation errors.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.validation_errors.is_empty()
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
                "Slug must contain only lowercase letters, digits, and hyphens (no spaces or double hyphens)".to_string(),
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

    /// Validates draft fields AND performs a live uniqueness check against existing articles.
    pub fn validate_with_existing_slugs(&mut self, existing_articles: &[(Uuid, String)]) -> bool {
        let format_valid = self.validate();

        if self.slug_collides_with(existing_articles) {
            self.validation_errors.insert(
                "slug".to_string(),
                "Slug is already in use by another story".to_string(),
            );
            return false;
        }

        format_valid && !self.validation_errors.contains_key("slug")
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

    #[test]
    fn test_article_draft_setters_and_collision_checks() {
        let mut draft = ArticleDraft::new();
        assert!(!draft.is_custom_color);

        // Auto-slug from headline
        draft.set_headline(
            "Investigation: Port Security Vulnerabilities Exposed!",
            true,
        );
        assert_eq!(
            draft.slug,
            "investigation-port-security-vulnerabilities-exposed"
        );
        assert!(!draft.color_hex.is_empty());
        assert!(!draft.is_custom_color);

        // Custom color override
        let initial_color = draft.color_hex.clone();
        assert!(!initial_color.is_empty());
        draft.set_color("#E53935");
        assert!(draft.is_custom_color);
        assert_eq!(draft.color_hex, "#E53935");

        // Slug change does not overwrite custom color
        draft.set_slug("port-security-audit");
        assert_eq!(draft.color_hex, "#E53935");

        // Reset to slug hash
        draft.reset_color_to_hash();
        assert!(!draft.is_custom_color);
        assert_ne!(draft.color_hex, "#E53935");

        // Collision checking
        let existing = vec![
            (Uuid::new_v4(), "transit-strike".to_string()),
            (Uuid::new_v4(), "port-security-audit".to_string()),
        ];
        assert!(draft.slug_collides_with(&existing));

        let valid = draft.validate_with_existing_slugs(&existing);
        assert!(!valid);
        assert_eq!(
            draft.validation_errors.get("slug").map(String::as_str),
            Some("Slug is already in use by another story")
        );

        // Non-colliding slug
        draft.set_slug("port-security-follow-up");
        assert!(!draft.slug_collides_with(&existing));
        assert!(draft.validate_with_existing_slugs(&existing));
    }
}
