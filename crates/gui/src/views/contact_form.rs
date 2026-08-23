//! Contact creation and editing form field presentation models, validation state, and associated articles view builders.

use chrono::Utc;
use newsjournal_core::color::assign_color_for_slug;
use newsjournal_core::models::{Article, ArticleStage};
use newsjournal_core::validation::{
    format_phone_display, is_valid_email, is_valid_phone, validate_name,
};
use serde::Serialize;
use uuid::Uuid;

use crate::message::AppMessage;
use crate::state::modal::ContactDraft;
use crate::state::AppState;
use crate::views::article_card::{calculate_contrast_color, format_contact_initials};
use crate::views::article_form::ValidationTooltipViewModel;
use crate::views::articles::stage_metadata;
use crate::views::contacts::{assign_avatar_color_for_contact, truncate_snippet};

/// Maximum allowed length for contact name.
pub const MAX_CONTACT_NAME_LENGTH: usize = 100;

/// Maximum allowed length for organization.
pub const MAX_CONTACT_ORG_LENGTH: usize = 100;

/// Maximum allowed length for role.
pub const MAX_CONTACT_ROLE_LENGTH: usize = 100;

/// Sizing and presentation view model for the Contact Name input field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContactNameFieldViewModel {
    /// Field section label.
    pub label: &'static str,
    /// Current name text value.
    pub value: String,
    /// Placeholder guidance hint.
    pub placeholder: &'static str,
    /// Maximum character limit.
    pub max_length: usize,
    /// Current character count.
    pub char_count: usize,
    /// Active validation error message (if any).
    pub error: Option<String>,
    /// Contextual validation error tooltip (if any).
    pub tooltip: Option<ValidationTooltipViewModel>,
    /// Whether the name is non-empty and passes validation.
    pub is_valid: bool,
}

impl ContactNameFieldViewModel {
    /// Builds the name field view model from the contact draft.
    #[must_use]
    pub fn build(draft: &ContactDraft) -> Self {
        let value = draft.name.clone();
        let char_count = value.chars().count();
        let error = draft.validation_errors.get("name").cloned();
        let tooltip = error
            .as_ref()
            .map(|err| ValidationTooltipViewModel::error("name", err.clone()));
        let is_valid = error.is_none() && validate_name(&value).is_ok();

        Self {
            label: "Full Name *",
            value,
            placeholder: "Full name (e.g. Jane Doe, Dr. Alex Rivera)",
            max_length: MAX_CONTACT_NAME_LENGTH,
            char_count,
            error,
            tooltip,
            is_valid,
        }
    }
}

/// Sizing and presentation view model for the Contact Organization input field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContactOrganizationFieldViewModel {
    /// Field section label.
    pub label: &'static str,
    /// Current organization text value.
    pub value: String,
    /// Placeholder guidance hint.
    pub placeholder: &'static str,
    /// Maximum character limit.
    pub max_length: usize,
    /// Current character count.
    pub char_count: usize,
    /// Whether the field currently contains a non-empty string.
    pub has_value: bool,
}

impl ContactOrganizationFieldViewModel {
    /// Builds the organization field view model from the contact draft.
    #[must_use]
    pub fn build(draft: &ContactDraft) -> Self {
        let value = draft.organization.clone();
        let char_count = value.chars().count();
        let has_value = !value.trim().is_empty();

        Self {
            label: "Organization / Outlet",
            value,
            placeholder: "Agency, newsroom, university, company, or affiliation",
            max_length: MAX_CONTACT_ORG_LENGTH,
            char_count,
            has_value,
        }
    }
}

/// Sizing and presentation view model for the Contact Role / Beat input field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContactRoleFieldViewModel {
    /// Field section label.
    pub label: &'static str,
    /// Current role text value.
    pub value: String,
    /// Placeholder guidance hint.
    pub placeholder: &'static str,
    /// Maximum character limit.
    pub max_length: usize,
    /// Current character count.
    pub char_count: usize,
    /// Whether the field currently contains a non-empty string.
    pub has_value: bool,
}

impl ContactRoleFieldViewModel {
    /// Builds the role field view model from the contact draft.
    #[must_use]
    pub fn build(draft: &ContactDraft) -> Self {
        let value = draft.role.clone();
        let char_count = value.chars().count();
        let has_value = !value.trim().is_empty();

        Self {
            label: "Role / Beat / Expertise",
            value,
            placeholder: "Job title, reporting beat, subject area, or relationship",
            max_length: MAX_CONTACT_ROLE_LENGTH,
            char_count,
            has_value,
        }
    }
}

/// Sizing and presentation view model for the Contact Phone input field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContactPhoneFieldViewModel {
    /// Field section label.
    pub label: &'static str,
    /// Current raw phone text value.
    pub value: String,
    /// Pretty formatted phone display.
    pub formatted_display: String,
    /// Placeholder guidance hint.
    pub placeholder: &'static str,
    /// Active validation error message (if any).
    pub error: Option<String>,
    /// Contextual validation error tooltip (if any).
    pub tooltip: Option<ValidationTooltipViewModel>,
    /// Whether the phone number format is valid.
    pub is_valid: bool,
    /// Whether the field currently contains a non-empty string.
    pub has_value: bool,
}

impl ContactPhoneFieldViewModel {
    /// Builds the phone field view model from the contact draft.
    #[must_use]
    pub fn build(draft: &ContactDraft) -> Self {
        let value = draft.phone.clone();
        let clean = value.trim();
        let has_value = !clean.is_empty();
        let formatted_display = if has_value {
            format_phone_display(clean)
        } else {
            String::new()
        };

        let error = draft.validation_errors.get("phone").cloned();
        let tooltip = error
            .as_ref()
            .map(|err| ValidationTooltipViewModel::error("phone", err.clone()));
        let is_valid = error.is_none() && (clean.is_empty() || is_valid_phone(clean));

        Self {
            label: "Phone Number",
            value,
            formatted_display,
            placeholder: "Direct phone number (e.g. +1 555-123-4567)",
            error,
            tooltip,
            is_valid,
            has_value,
        }
    }
}

/// Sizing and presentation view model for the Contact Email input field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContactEmailFieldViewModel {
    /// Field section label.
    pub label: &'static str,
    /// Current email text value.
    pub value: String,
    /// Placeholder guidance hint.
    pub placeholder: &'static str,
    /// Active validation error message (if any).
    pub error: Option<String>,
    /// Contextual validation error tooltip (if any).
    pub tooltip: Option<ValidationTooltipViewModel>,
    /// Whether the email address format is valid.
    pub is_valid: bool,
    /// Whether the field currently contains a non-empty string.
    pub has_value: bool,
}

impl ContactEmailFieldViewModel {
    /// Builds the email field view model from the contact draft.
    #[must_use]
    pub fn build(draft: &ContactDraft) -> Self {
        let value = draft.email.clone();
        let clean = value.trim();
        let has_value = !clean.is_empty();

        let error = draft.validation_errors.get("email").cloned();
        let tooltip = error
            .as_ref()
            .map(|err| ValidationTooltipViewModel::error("email", err.clone()));
        let is_valid = error.is_none() && (clean.is_empty() || is_valid_email(clean));

        Self {
            label: "Email Address",
            value,
            placeholder: "Direct email address (e.g. source@example.org)",
            error,
            tooltip,
            is_valid,
            has_value,
        }
    }
}

/// Sizing and presentation view model for the multi-line Contact Notes text area.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContactNotesFieldViewModel {
    /// Field section label.
    pub label: &'static str,
    /// Current notes text content.
    pub value: String,
    /// Placeholder guidance text.
    pub placeholder: &'static str,
    /// Total character count.
    pub char_count: usize,
    /// Total line count.
    pub line_count: usize,
    /// Whether the field currently contains a non-empty string.
    pub has_value: bool,
}

impl ContactNotesFieldViewModel {
    /// Builds the notes field view model from the contact draft.
    #[must_use]
    pub fn build(draft: &ContactDraft) -> Self {
        let value = draft.notes.clone();
        let char_count = value.chars().count();
        let line_count = value.lines().count().max(1);
        let has_value = !value.trim().is_empty();

        Self {
            label: "Background & Confidential Notes",
            value,
            placeholder: "Key source details, Signal / PGP handles, preferred interview hours, vetting history, and topic expertise…",
            char_count,
            line_count,
            has_value,
        }
    }
}

/// Presentation model for a single associated reporting story linked to the contact.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ContactAssociatedArticleItemViewModel {
    /// Unique article ID.
    pub article_id: Uuid,
    /// Raw slug string.
    pub slug: String,
    /// Formatted display slug (e.g. `"#transit-probe"`).
    pub display_slug: String,
    /// Full headline text.
    pub headline: String,
    /// Truncated preview headline.
    pub headline_preview: String,
    /// Editorial workflow stage.
    pub stage: ArticleStage,
    /// Stage title label (e.g. "Writing", "Published").
    pub stage_title: &'static str,
    /// Stage accent color hex.
    pub stage_accent_hex: &'static str,
    /// Assigned or custom article hex color.
    pub color_hex: String,
    /// High-contrast text color against the article color.
    pub contrast_fg_hex: String,
    /// Translucent background tint hex color.
    pub translucent_tint_hex: String,
    /// Deadline date display string (if set).
    pub deadline_display: Option<String>,
    /// True if the story deadline is overdue.
    pub is_overdue: bool,
    /// Formatted task completion counter label (e.g. "2/5 tasks").
    pub task_summary_label: String,
    /// Action message to open the article editor drawer.
    pub open_article_message: AppMessage,
    /// Action message to filter the articles board to this story.
    pub filter_message: AppMessage,
}

impl ContactAssociatedArticleItemViewModel {
    /// Constructs an associated article item view model from an Article and app state.
    #[must_use]
    pub fn build(article: &Article, contact_id: Uuid, state: &AppState) -> Self {
        let meta = stage_metadata(article.stage);
        let slug = article.slug.clone();
        let display_slug = format!("#{slug}");
        let headline = article.headline.clone();
        let headline_preview = truncate_snippet(&headline, 55);

        let color_hex = article
            .color
            .clone()
            .unwrap_or_else(|| assign_color_for_slug(&slug).to_hex());
        let contrast_fg_hex = calculate_contrast_color(&color_hex).to_string();
        let translucent_tint_hex = format!("{color_hex}26");

        let is_overdue = article.is_overdue(Utc::now());
        let deadline_display = article
            .deadline
            .map(|dt| dt.format("%b %d, %Y").to_string());

        let (completed, total) = state.task_completion_stats(article.id);
        let task_summary_label = if total > 0 {
            format!("{completed}/{total} tasks")
        } else {
            "0 tasks".to_string()
        };

        Self {
            article_id: article.id,
            slug,
            display_slug,
            headline,
            headline_preview,
            stage: article.stage,
            stage_title: meta.title,
            stage_accent_hex: meta.accent_hex,
            color_hex,
            contrast_fg_hex,
            translucent_tint_hex,
            deadline_display,
            is_overdue,
            task_summary_label,
            open_article_message: AppMessage::OpenEditArticleModal(article.id),
            filter_message: AppMessage::SetContactFilter(Some(contact_id)),
        }
    }
}

/// Sizing and presentation view model for the Associated Articles sub-section.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ContactAssociatedArticlesSectionViewModel {
    /// Sub-section title.
    pub title: &'static str,
    /// Contextual guidance subtitle.
    pub subtitle: &'static str,
    /// Total count of associated stories.
    pub count: usize,
    /// Formatted count badge label (e.g. "3 stories", "1 story", "0 stories").
    pub count_label: String,
    /// List of rich associated article items.
    pub items: Vec<ContactAssociatedArticleItemViewModel>,
    /// Whether there are currently no associated stories.
    pub is_empty: bool,
    /// Explanatory empty state text.
    pub empty_guidance: &'static str,
}

impl ContactAssociatedArticlesSectionViewModel {
    /// Builds the associated articles sub-section from the draft and application state.
    #[must_use]
    pub fn build(draft: &ContactDraft, state: &AppState) -> Self {
        let (items, count) = if let Some(contact_id) = draft.id {
            let articles = state.articles_for_contact(contact_id);
            let total = articles.len();
            let vms: Vec<ContactAssociatedArticleItemViewModel> = articles
                .into_iter()
                .map(|art| ContactAssociatedArticleItemViewModel::build(art, contact_id, state))
                .collect();
            (vms, total)
        } else {
            (Vec::new(), 0)
        };

        let is_empty = count == 0;
        let count_label = match count {
            0 => "0 stories tagged".to_string(),
            1 => "1 story tagged".to_string(),
            n => format!("{n} stories tagged"),
        };

        let empty_guidance = if draft.id.is_some() {
            "This contact is not currently tagged in any reporting stories. You can tag them from any Article drawer."
        } else {
            "Newly created contacts have no tagged stories yet. Once saved, tag this source across relevant reporting stories."
        };

        Self {
            title: "Tagged Reporting Stories",
            subtitle: "Stories and investigative articles citing or linked to this source",
            count,
            count_label,
            items,
            is_empty,
            empty_guidance,
        }
    }
}

/// Header presentation model for the Contact Creation & Edit Drawer.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ContactFormHeaderViewModel {
    /// True if editing an existing contact, false if creating a new contact.
    pub is_edit: bool,
    /// Drawer headline title ("New Contact" / "Edit Contact").
    pub title: &'static str,
    /// Contextual explanatory subtitle.
    pub subtitle: &'static str,
    /// Avatar monogram initials.
    pub avatar_initials: String,
    /// Deterministic avatar hex color.
    pub avatar_color_hex: String,
    /// High-contrast foreground text color for avatar monogram.
    pub contrast_fg_hex: String,
    /// True if the delete action is available (editing an existing contact).
    pub can_delete: bool,
    /// Action dispatched when clicking the delete contact button.
    pub delete_action: Option<AppMessage>,
    /// Fallback Unicode emoji icon.
    pub icon_emoji: &'static str,
    /// Standard symbolic icon name.
    pub icon_name: &'static str,
    /// macOS SF Symbol identifier.
    pub sf_symbol: &'static str,
    /// Total count of tagged articles.
    pub linked_articles_count: usize,
    /// Formatted linked articles count label.
    pub linked_articles_count_label: String,
}

impl ContactFormHeaderViewModel {
    /// Builds the contact drawer header presentation model.
    #[must_use]
    pub fn build(draft: &ContactDraft, linked_count: usize) -> Self {
        let is_edit = draft.id.is_some();
        let title = if is_edit {
            "Edit Contact"
        } else {
            "New Contact"
        };
        let subtitle = if is_edit {
            "Update source information, beats, and linked reporting"
        } else {
            "Add a source, interviewee, subject matter expert, or collaborator"
        };

        let dummy_id = draft.id.unwrap_or(Uuid::nil());
        let avatar_color_hex = assign_avatar_color_for_contact(dummy_id, &draft.name);
        let avatar_initials = format_contact_initials(&draft.name);
        let contrast_fg_hex = calculate_contrast_color(&avatar_color_hex).to_string();

        let can_delete = is_edit;
        let delete_action = draft.id.map(AppMessage::PromptDeleteContact);

        let linked_articles_count_label = match linked_count {
            0 => "0 linked stories".to_string(),
            1 => "1 linked story".to_string(),
            n => format!("{n} linked stories"),
        };

        Self {
            is_edit,
            title,
            subtitle,
            avatar_initials,
            avatar_color_hex,
            contrast_fg_hex,
            can_delete,
            delete_action,
            icon_emoji: "👥",
            icon_name: "user",
            sf_symbol: "person.fill",
            linked_articles_count: linked_count,
            linked_articles_count_label,
        }
    }
}

/// Unified presentation view model for the complete Contact Creation & Editing Drawer.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ContactFormViewModel {
    /// Contact unique ID if editing, or `None` if creating.
    pub id: Option<Uuid>,
    /// True if editing an existing contact.
    pub is_edit: bool,
    /// Header presentation view model.
    pub header: ContactFormHeaderViewModel,
    /// Full Name input field view model.
    pub name_field: ContactNameFieldViewModel,
    /// Organization input field view model.
    pub organization_field: ContactOrganizationFieldViewModel,
    /// Role / Beat input field view model.
    pub role_field: ContactRoleFieldViewModel,
    /// Phone number input field view model.
    pub phone_field: ContactPhoneFieldViewModel,
    /// Email address input field view model.
    pub email_field: ContactEmailFieldViewModel,
    /// Background notes input field view model.
    pub notes_field: ContactNotesFieldViewModel,
    /// Associated stories sub-section view model.
    pub associated_articles: ContactAssociatedArticlesSectionViewModel,
    /// All active field validation error tooltips.
    pub validation_tooltips: Vec<ValidationTooltipViewModel>,
    /// Whether the form is currently valid and ready for submission.
    pub can_save: bool,
    /// Primary submit button label ("Create Contact" or "Save Contact").
    pub save_button_label: &'static str,
    /// Explanatory tooltip for the save button.
    pub save_tooltip: String,
    /// Save button keyboard shortcut display ("⌘S" or "Ctrl+S").
    pub save_shortcut: &'static str,
    /// Secondary dismiss button label ("Cancel").
    pub cancel_button_label: &'static str,
    /// Explanatory tooltip for the cancel button.
    pub cancel_tooltip: String,
    /// Cancel button keyboard shortcut display ("Esc").
    pub cancel_shortcut: &'static str,
    /// Submit action message.
    pub submit_action: AppMessage,
    /// Cancel action message.
    pub cancel_action: AppMessage,
}

/// Constructs the full `ContactFormViewModel` hierarchy with default platform detection.
#[must_use]
pub fn build_contact_form_view(state: &AppState, draft: &ContactDraft) -> ContactFormViewModel {
    let is_macos = cfg!(target_os = "macos");
    build_contact_form_view_with_layout(state, draft, is_macos)
}

/// Constructs the full `ContactFormViewModel` hierarchy with explicit platform layout context.
#[must_use]
pub fn build_contact_form_view_with_layout(
    state: &AppState,
    draft: &ContactDraft,
    is_macos: bool,
) -> ContactFormViewModel {
    let is_edit = draft.id.is_some();
    let name_field = ContactNameFieldViewModel::build(draft);
    let organization_field = ContactOrganizationFieldViewModel::build(draft);
    let role_field = ContactRoleFieldViewModel::build(draft);
    let phone_field = ContactPhoneFieldViewModel::build(draft);
    let email_field = ContactEmailFieldViewModel::build(draft);
    let notes_field = ContactNotesFieldViewModel::build(draft);
    let associated_articles = ContactAssociatedArticlesSectionViewModel::build(draft, state);

    let header = ContactFormHeaderViewModel::build(draft, associated_articles.count);

    let mut validation_tooltips = Vec::new();
    if let Some(ref tt) = name_field.tooltip {
        validation_tooltips.push(tt.clone());
    }
    if let Some(ref tt) = phone_field.tooltip {
        validation_tooltips.push(tt.clone());
    }
    if let Some(ref tt) = email_field.tooltip {
        validation_tooltips.push(tt.clone());
    }

    let is_name_empty = draft.name.trim().is_empty();
    let can_save = draft.validation_errors.is_empty() && !is_name_empty;

    let save_shortcut = if is_macos { "⌘S" } else { "Ctrl+S" };
    let cancel_shortcut = "Esc";

    let save_button_label = if is_edit {
        "Save Contact"
    } else {
        "Create Contact"
    };

    let save_tooltip = if can_save {
        format!("{save_button_label} ({save_shortcut})")
    } else if !draft.validation_errors.is_empty() {
        let count = draft.validation_errors.len();
        format!("Cannot save: {count} field(s) need attention ({save_shortcut})")
    } else {
        format!("Cannot save: Contact name is required ({save_shortcut})")
    };

    let cancel_button_label = "Cancel";
    let cancel_tooltip = format!("Cancel and discard changes ({cancel_shortcut})");

    ContactFormViewModel {
        id: draft.id,
        is_edit,
        header,
        name_field,
        organization_field,
        role_field,
        phone_field,
        email_field,
        notes_field,
        associated_articles,
        validation_tooltips,
        can_save,
        save_button_label,
        save_tooltip,
        save_shortcut,
        cancel_button_label,
        cancel_tooltip,
        cancel_shortcut,
        submit_action: AppMessage::SubmitModal,
        cancel_action: AppMessage::CloseModal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use newsjournal_core::models::{Article, Contact};

    #[test]
    fn test_contact_form_view_model_creation_mode() {
        let state = AppState::in_memory().expect("in-memory state");
        let mut draft = ContactDraft::new();

        // 1. Fresh empty draft
        let vm = build_contact_form_view(&state, &draft);
        assert!(!vm.is_edit);
        assert_eq!(vm.header.title, "New Contact");
        assert_eq!(vm.save_button_label, "Create Contact");
        assert!(!vm.can_save);
        assert!(!vm.header.can_delete);
        assert!(vm.header.delete_action.is_none());
        assert!(vm.associated_articles.is_empty);
        assert_eq!(vm.associated_articles.count, 0);

        // 2. Set valid name
        draft.set_name("Sarah Jenkins");
        let vm = build_contact_form_view(&state, &draft);
        assert!(vm.can_save);
        assert_eq!(vm.name_field.value, "Sarah Jenkins");
        assert!(vm.name_field.is_valid);
        assert_eq!(vm.header.avatar_initials, "SJ");
        assert!(vm.header.avatar_color_hex.starts_with('#'));

        // 3. Set organization and role
        draft.set_organization("ProPublica");
        draft.set_role("Investigative Fellow");
        let vm = build_contact_form_view(&state, &draft);
        assert_eq!(vm.organization_field.value, "ProPublica");
        assert!(vm.organization_field.has_value);
        assert_eq!(vm.role_field.value, "Investigative Fellow");
        assert!(vm.role_field.has_value);

        // 4. Set invalid email
        draft.set_email("invalid-email");
        let vm = build_contact_form_view(&state, &draft);
        assert!(!vm.can_save);
        assert!(!vm.email_field.is_valid);
        assert!(vm.email_field.error.is_some());
        assert_eq!(vm.validation_tooltips.len(), 1);

        // 5. Fix email and add phone
        draft.set_email("sarah@propublica.org");
        draft.set_phone("+1 555-987-6543");
        let vm = build_contact_form_view(&state, &draft);
        assert!(vm.can_save);
        assert!(vm.email_field.is_valid);
        assert!(vm.phone_field.is_valid);
        assert_eq!(vm.phone_field.formatted_display, "+1 (555) 987-6543");
        assert_eq!(vm.validation_tooltips.len(), 0);
    }

    #[test]
    fn test_contact_form_view_model_edit_mode_and_associated_articles() {
        let mut state = AppState::in_memory().expect("in-memory state");

        // 1. Create a contact and two articles linked to this contact
        let contact = Contact::builder("Marcus Brody")
            .organization("Museum of Antiquities")
            .role("Curator")
            .email("marcus@museum.example.edu")
            .phone("5551234567")
            .notes("Expert on artifact provenance and smuggling routes.")
            .build();
        let contact_id = contact.id;
        state
            .storage
            .create_contact(contact.clone())
            .expect("save contact");

        let art1 = Article::builder("antiquities-heist", "Antiquities Heist Investigation")
            .stage(ArticleStage::Writing)
            .build();
        let art1_id = art1.id;
        state.storage.create_article(art1).expect("save art1");

        let art2 = Article::builder("smuggling-network", "Smuggling Network Exposed")
            .stage(ArticleStage::Researching)
            .build();
        let art2_id = art2.id;
        state.storage.create_article(art2).expect("save art2");

        state
            .storage
            .link_contact_to_article(art1_id, contact_id)
            .expect("link art1");
        state
            .storage
            .link_contact_to_article(art2_id, contact_id)
            .expect("link art2");

        state.load_all().expect("reload state");

        // 2. Open edit drawer for this contact
        let draft = ContactDraft::from_contact(&contact);
        let vm = build_contact_form_view(&state, &draft);

        assert!(vm.is_edit);
        assert_eq!(vm.header.title, "Edit Contact");
        assert_eq!(vm.save_button_label, "Save Contact");
        assert!(vm.can_save);
        assert!(vm.header.can_delete);
        assert_eq!(
            vm.header.delete_action,
            Some(AppMessage::PromptDeleteContact(contact_id))
        );
        assert_eq!(vm.header.avatar_initials, "MB");

        // 3. Verify associated articles sub-section
        assert!(!vm.associated_articles.is_empty);
        assert_eq!(vm.associated_articles.count, 2);
        assert_eq!(vm.associated_articles.count_label, "2 stories tagged");
        assert_eq!(vm.associated_articles.items.len(), 2);

        let slugs: Vec<&str> = vm
            .associated_articles
            .items
            .iter()
            .map(|i| i.slug.as_str())
            .collect();
        assert!(slugs.contains(&"antiquities-heist"));
        assert!(slugs.contains(&"smuggling-network"));

        let heist_item = vm
            .associated_articles
            .items
            .iter()
            .find(|i| i.slug == "antiquities-heist")
            .unwrap();
        assert_eq!(heist_item.stage, ArticleStage::Writing);
        assert_eq!(heist_item.stage_title, "Writing");
        assert_eq!(heist_item.display_slug, "#antiquities-heist");
        assert_eq!(
            heist_item.open_article_message,
            AppMessage::OpenEditArticleModal(art1_id)
        );
        assert_eq!(
            heist_item.filter_message,
            AppMessage::SetContactFilter(Some(contact_id))
        );
    }
}
