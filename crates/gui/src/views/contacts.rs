//! Contacts directory view models.

use newsjournal_core::validation::format_phone_display;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;

/// Formatted view model for a contact in the contacts directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactListItemViewModel {
    /// Unique contact ID.
    pub id: Uuid,
    /// Contact full name.
    pub name: String,
    /// Organization / Newsroom.
    pub organization: String,
    /// Beat or Role.
    pub role: String,
    /// Normalized phone string.
    pub phone: String,
    /// Pretty formatted phone display.
    pub phone_display: String,
    /// Email address.
    pub email: String,
    /// Notes preview.
    pub notes: String,
    /// Number of linked stories.
    pub linked_articles_count: usize,
    /// List of linked article slugs.
    pub linked_article_slugs: Vec<String>,
}

/// Constructs the list of contact view models from application state.
#[must_use]
pub fn build_contacts_view(state: &AppState) -> Vec<ContactListItemViewModel> {
    let filtered_contacts = state.filtered_contacts();

    filtered_contacts
        .into_iter()
        .map(|contact| {
            let linked_articles = state.articles_for_contact(contact.id);
            let linked_article_slugs = linked_articles.iter().map(|a| a.slug.clone()).collect();
            let phone = contact.phone.clone().unwrap_or_default();
            let phone_display = if phone.is_empty() {
                String::new()
            } else {
                format_phone_display(&phone)
            };

            ContactListItemViewModel {
                id: contact.id,
                name: contact.name.clone(),
                organization: contact.organization.clone().unwrap_or_default(),
                role: contact.role.clone().unwrap_or_default(),
                phone,
                phone_display,
                email: contact.email.clone().unwrap_or_default(),
                notes: contact.notes.clone().unwrap_or_default(),
                linked_articles_count: linked_articles.len(),
                linked_article_slugs,
            }
        })
        .collect()
}
