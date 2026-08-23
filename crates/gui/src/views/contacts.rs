//! Contacts directory view models, table headers, sorting controls, and list presenters.

use newsjournal_core::color::assign_color_for_slug;
use newsjournal_core::models::ArticleStage;
use newsjournal_core::validation::format_phone_display;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::message::AppMessage;
use crate::state::filters::{ContactSortConfig, ContactSortField, SortDirection};
use crate::state::AppState;
use crate::views::article_card::format_contact_initials;
use crate::views::articles::stage_metadata;

/// Default nominal row height for the contacts table in logical pixels.
pub const DEFAULT_CONTACT_ROW_HEIGHT: f32 = 64.0;

/// Maximum character length for contact notes snippet in table row.
pub const MAX_NOTES_SNIPPET_LEN: usize = 120;

/// Maximum character length for organization name snippet.
pub const MAX_ORGANIZATION_SNIPPET_LEN: usize = 40;

/// Maximum character length for role / beat snippet.
pub const MAX_ROLE_SNIPPET_LEN: usize = 35;

/// Diameter of contact avatar badge in logical pixels.
pub const CONTACT_AVATAR_SIZE: f32 = 36.0;

/// Maximum number of tagged article pills displayed inline before "+N more" badge.
pub const MAX_DISPLAYED_TAGGED_ARTICLES: usize = 3;

/// Generates a deterministic high-contrast avatar hex color for a contact.
#[must_use]
pub fn assign_avatar_color_for_contact(id: Uuid, name: &str) -> String {
    let seed = format!("{name}-{id}");
    assign_color_for_slug(&seed).to_hex()
}

/// Truncates a string to `max_len` characters with an ellipsis if it exceeds the limit.
#[must_use]
pub fn truncate_snippet(text: &str, max_len: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= max_len {
        trimmed.to_string()
    } else {
        let snippet: String = trimmed.chars().take(max_len.saturating_sub(1)).collect();
        format!("{}…", snippet.trim_end())
    }
}

/// Presentation model for a linked article pill/badge associated with a contact.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ContactLinkedArticleTagViewModel {
    /// Unique article ID.
    pub id: Uuid,
    /// Article unique slug.
    pub slug: String,
    /// Headline preview snippet.
    pub headline_preview: String,
    /// Current article stage.
    pub stage: ArticleStage,
    /// Stage title (e.g. "Writing", "Researching").
    pub stage_title: &'static str,
    /// Stage accent color hex.
    pub stage_accent_hex: &'static str,
    /// Custom or slug-assigned color hex.
    pub color_hex: String,
    /// Action message to filter the articles deck to this story.
    pub filter_message: AppMessage,
}

/// Detailed indicators showing which fields of a contact matched the active search query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ContactSearchHighlight {
    /// Contact name matched search query.
    pub matches_name: bool,
    /// Organization matched search query.
    pub matches_organization: bool,
    /// Role / beat matched search query.
    pub matches_role: bool,
    /// Email matched search query.
    pub matches_email: bool,
    /// Phone number matched search query.
    pub matches_phone: bool,
    /// Background notes matched search query.
    pub matches_notes: bool,
    /// Linked story slug or headline matched search query.
    pub matches_tagged_story: bool,
}

impl ContactSearchHighlight {
    /// Computes which fields match the given search query string.
    #[must_use]
    pub fn evaluate(
        query: &str,
        contact: &newsjournal_core::models::Contact,
        tagged_slugs: &[String],
    ) -> Option<Self> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return None;
        }

        let matches_name = contact.name.to_lowercase().contains(&q);
        let matches_organization = contact
            .organization
            .as_deref()
            .map(|o| o.to_lowercase().contains(&q))
            .unwrap_or(false);
        let matches_role = contact
            .role
            .as_deref()
            .map(|r| r.to_lowercase().contains(&q))
            .unwrap_or(false);
        let matches_email = contact
            .email
            .as_deref()
            .map(|e| e.to_lowercase().contains(&q))
            .unwrap_or(false);
        let matches_phone = contact
            .phone
            .as_deref()
            .map(|p| p.contains(&q))
            .unwrap_or(false);
        let matches_notes = contact
            .notes
            .as_deref()
            .map(|n| n.to_lowercase().contains(&q))
            .unwrap_or(false);
        let matches_tagged_story = tagged_slugs.iter().any(|s| s.to_lowercase().contains(&q));

        if matches_name
            || matches_organization
            || matches_role
            || matches_email
            || matches_phone
            || matches_notes
            || matches_tagged_story
        {
            Some(Self {
                matches_name,
                matches_organization,
                matches_role,
                matches_email,
                matches_phone,
                matches_notes,
                matches_tagged_story,
            })
        } else {
            None
        }
    }
}

/// Formatted view model for a single contact row / item in the contacts directory.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ContactListItemViewModel {
    /// Unique contact ID.
    pub id: Uuid,
    /// Contact full name.
    pub name: String,
    /// Uppercase 1-2 letter initials monogram.
    pub initials: String,
    /// Deterministic avatar hex color.
    pub avatar_color_hex: String,
    /// Organization / Newsroom name.
    pub organization: String,
    /// Whether organization is non-empty.
    pub has_organization: bool,
    /// Beat or Role title.
    pub role: String,
    /// Whether role is non-empty.
    pub has_role: bool,
    /// Normalized phone string.
    pub phone: String,
    /// Pretty formatted phone display (e.g. "(555) 123-4567").
    pub phone_display: String,
    /// Whether phone is non-empty.
    pub has_phone: bool,
    /// Email address.
    pub email: String,
    /// Whether email is non-empty.
    pub has_email: bool,
    /// Full notes text.
    pub notes: String,
    /// Truncated notes preview snippet.
    pub notes_preview: String,
    /// Whether notes are non-empty.
    pub has_notes: bool,
    /// Number of linked stories.
    pub linked_articles_count: usize,
    /// Formatted story count label (e.g. "3 stories", "1 story", "0 stories").
    pub linked_articles_count_label: String,
    /// List of linked article slugs.
    pub linked_article_slugs: Vec<String>,
    /// Rich linked article pill view models (up to `MAX_DISPLAYED_TAGGED_ARTICLES`).
    pub linked_articles: Vec<ContactLinkedArticleTagViewModel>,
    /// Number of additional linked stories beyond the displayed limit.
    pub overflow_linked_articles_count: usize,
    /// Whether this item matches the active search query.
    pub is_search_match: bool,
    /// Detailed search highlight breakdown.
    pub search_highlight: Option<ContactSearchHighlight>,
    /// Action message to open the edit contact drawer.
    pub edit_message: AppMessage,
    /// Action message to prompt deletion of this contact.
    pub delete_message: AppMessage,
    /// Action message to filter the articles board by this contact.
    pub filter_by_contact_message: AppMessage,
}

impl ContactListItemViewModel {
    /// Formats linked articles count into a user-friendly badge label.
    #[must_use]
    pub fn format_story_count_label(count: usize) -> String {
        match count {
            0 => "0 stories".to_string(),
            1 => "1 story".to_string(),
            n => format!("{n} stories"),
        }
    }
}

/// View model for an interactive sortable column header in the contacts table.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ContactColumnHeaderViewModel {
    /// Field associated with this column.
    pub field: ContactSortField,
    /// User-visible column label.
    pub label: &'static str,
    /// Explanatory tooltip for sorting.
    pub tooltip: &'static str,
    /// Whether the table is currently sorted by this column.
    pub is_sorted: bool,
    /// Active sort direction if sorted by this column.
    pub sort_direction: Option<SortDirection>,
    /// Visual sort indicator icon / symbol ("▲", "▼", or "↕").
    pub sort_indicator: &'static str,
    /// Action message dispatched when clicking this column header.
    pub toggle_message: AppMessage,
}

/// Constructs the list of table column headers based on active sort configuration.
#[must_use]
pub fn build_contact_column_headers(
    sort_config: &ContactSortConfig,
) -> Vec<ContactColumnHeaderViewModel> {
    let columns = [
        (
            ContactSortField::Name,
            "Name",
            "Sort contacts alphabetically by name",
        ),
        (
            ContactSortField::Organization,
            "Organization",
            "Sort contacts by organization affiliation",
        ),
        (
            ContactSortField::Role,
            "Role / Beat",
            "Sort contacts by beat or job title",
        ),
        (
            ContactSortField::Email,
            "Email",
            "Sort contacts by email address",
        ),
        (
            ContactSortField::Phone,
            "Phone",
            "Sort contacts by phone number",
        ),
        (
            ContactSortField::StoriesCount,
            "Tagged Stories",
            "Sort contacts by number of linked reporting stories",
        ),
    ];

    columns
        .into_iter()
        .map(|(field, label, tooltip)| {
            let is_sorted = sort_config.field == field;
            let sort_direction = if is_sorted {
                Some(sort_config.direction)
            } else {
                None
            };
            let sort_indicator = match sort_direction {
                Some(SortDirection::Ascending) => "▲",
                Some(SortDirection::Descending) => "▼",
                None => "↕",
            };

            ContactColumnHeaderViewModel {
                field,
                label,
                tooltip,
                is_sorted,
                sort_direction,
                sort_indicator,
                toggle_message: AppMessage::ToggleContactSort(field),
            }
        })
        .collect()
}

/// Top toolbar view model for the Contacts Directory page.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ContactsToolbarViewModel {
    /// Main page section title.
    pub title: &'static str,
    /// Secondary section subtitle.
    pub subtitle: &'static str,
    /// Total number of contacts in directory.
    pub total_contacts_count: usize,
    /// Formatted total contacts label (e.g. "12 contacts", "1 contact").
    pub total_count_label: String,
    /// Number of contacts matching current search filters.
    pub filtered_contacts_count: usize,
    /// Filtered count badge label if actively filtered (e.g. "3 of 12 matching").
    pub filtered_count_label: Option<String>,
    /// Total story citation links across all contacts.
    pub total_citations_count: usize,
    /// Formatted citations count label (e.g. "24 story citations").
    pub total_citations_label: String,
    /// Current search query string.
    pub search_query: String,
    /// Whether search is actively filtering contacts.
    pub is_search_active: bool,
    /// Active sort configuration.
    pub sort_config: ContactSortConfig,
    /// Human-readable summary of the active sort (e.g. "Name (A–Z)").
    pub sort_summary_label: String,
    /// Primary quick action button label ("+ Add Contact").
    pub primary_action_label: &'static str,
    /// Primary action tooltip.
    pub primary_action_tooltip: &'static str,
    /// Primary action shortcut display (e.g. "⌘⇧C" or "Ctrl+Shift+C").
    pub primary_action_shortcut: &'static str,
    /// Primary action symbolic icon name.
    pub primary_action_icon_name: &'static str,
    /// Primary action macOS SF Symbol identifier.
    pub primary_action_sf_symbol: &'static str,
    /// Primary action fallback emoji icon.
    pub primary_action_icon_emoji: &'static str,
    /// Message dispatched when clicking the primary action button.
    pub primary_action_message: AppMessage,
    /// Message dispatched to clear search query.
    pub clear_search_message: AppMessage,
    /// Message dispatched to clear all filters.
    pub clear_all_filters_message: AppMessage,
    /// Whether any filter or search query is currently active.
    pub is_filtered: bool,
}

impl ContactsToolbarViewModel {
    /// Formats total contacts count into a user-friendly label.
    #[must_use]
    pub fn format_total_count_label(count: usize) -> String {
        match count {
            0 => "0 contacts".to_string(),
            1 => "1 contact".to_string(),
            n => format!("{n} contacts"),
        }
    }

    /// Formats total citation count into a user-friendly label.
    #[must_use]
    pub fn format_total_citations_label(count: usize) -> String {
        match count {
            0 => "0 story citations".to_string(),
            1 => "1 story citation".to_string(),
            n => format!("{n} story citations"),
        }
    }

    /// Returns the platform-appropriate keyboard shortcut display string for the Add Contact action.
    #[must_use]
    pub const fn primary_action_shortcut_for_platform(is_macos: bool) -> &'static str {
        if is_macos {
            "⌘⇧C"
        } else {
            "Ctrl+Shift+C"
        }
    }

    /// Builds the toolbar view model from application state and metrics.
    #[must_use]
    pub fn build(
        state: &AppState,
        total_contacts_count: usize,
        filtered_contacts_count: usize,
        total_citations_count: usize,
    ) -> Self {
        let total_count_label = Self::format_total_count_label(total_contacts_count);
        let total_citations_label = Self::format_total_citations_label(total_citations_count);
        let search_query = state.filters.search_query.clone();
        let is_search_active = !search_query.trim().is_empty();
        let sort_config = state.filters.contact_sort;
        let sort_summary_label = sort_config.summary_label();

        let filtered_count_label = if is_search_active && total_contacts_count > 0 {
            Some(format!(
                "{filtered_contacts_count} of {total_contacts_count} matching"
            ))
        } else {
            None
        };

        let is_filtered = state.filters.is_active();

        Self {
            title: "Contacts Directory",
            subtitle: "Sources, interviewees, subject matter experts & collaborators",
            total_contacts_count,
            total_count_label,
            filtered_contacts_count,
            filtered_count_label,
            total_citations_count,
            total_citations_label,
            search_query,
            is_search_active,
            sort_config,
            sort_summary_label,
            primary_action_label: "+ Add Contact",
            primary_action_tooltip: "Create a new contact record (⌘⇧C / Ctrl+Shift+C)",
            primary_action_shortcut: "⌘⇧C",
            primary_action_icon_name: "user-plus",
            primary_action_sf_symbol: "person.badge.plus",
            primary_action_icon_emoji: "➕",
            primary_action_message: AppMessage::OpenNewContactModal,
            clear_search_message: AppMessage::SetSearchQuery(String::new()),
            clear_all_filters_message: AppMessage::ClearFilters,
            is_filtered,
        }
    }
}

/// Directory-wide empty state presentation model when no contacts exist or search yields no matches.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ContactsEmptyStateViewModel {
    /// Large headline text.
    pub headline: String,
    /// Explanatory guidance text.
    pub subtext: String,
    /// Primary call to action button label.
    pub action_button_label: String,
    /// Primary action message.
    pub action_message: AppMessage,
    /// Secondary action button label (if any).
    pub secondary_action_label: Option<String>,
    /// Secondary action message (if any).
    pub secondary_action_message: Option<AppMessage>,
    /// Whether this empty state is caused by active filtering/search.
    pub is_filtered: bool,
    /// Standard symbolic icon name.
    pub icon_name: &'static str,
    /// macOS SF Symbol identifier.
    pub sf_symbol: &'static str,
    /// Fallback Unicode emoji icon.
    pub icon_emoji: &'static str,
}

impl ContactsEmptyStateViewModel {
    /// Constructs an empty state view model for an empty database or zero search matches.
    #[must_use]
    pub fn build(is_filtered: bool, query: &str) -> Self {
        if is_filtered {
            let q = query.trim();
            let subtext = if q.is_empty() {
                "No contacts match the active filter criteria. Try clearing filters to see all contacts."
                    .to_string()
            } else {
                format!(
                    "No sources or contacts match \"{q}\". Try searching for a different name, organization, beat, email, or phone number."
                )
            };

            Self {
                headline: "No Contacts Found".to_string(),
                subtext,
                action_button_label: "Clear Search".to_string(),
                action_message: AppMessage::SetSearchQuery(String::new()),
                secondary_action_label: Some("+ Add Contact".to_string()),
                secondary_action_message: Some(AppMessage::OpenNewContactModal),
                is_filtered: true,
                icon_name: "search",
                sf_symbol: "magnifyingglass",
                icon_emoji: "🔍",
            }
        } else {
            Self {
                headline: "No Contacts in Directory".to_string(),
                subtext: "Build your source database. Add sources, interviewees, spokespersons, and subject matter experts to tag them across your reporting stories.".to_string(),
                action_button_label: "+ Add First Contact".to_string(),
                action_message: AppMessage::OpenNewContactModal,
                secondary_action_label: None,
                secondary_action_message: None,
                is_filtered: false,
                icon_name: "users",
                sf_symbol: "person.2",
                icon_emoji: "👥",
            }
        }
    }
}

/// Comprehensive presentation model for the entire Contacts Directory view.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ContactsDirectoryViewModel {
    /// Directory top toolbar.
    pub toolbar: ContactsToolbarViewModel,
    /// Interactive sortable column headers.
    pub column_headers: Vec<ContactColumnHeaderViewModel>,
    /// Sorted and filtered contact row items.
    pub contacts: Vec<ContactListItemViewModel>,
    /// Empty state presentation model if no contacts to show.
    pub empty_state: Option<ContactsEmptyStateViewModel>,
    /// Whether the directory has 0 total contacts in storage.
    pub is_empty: bool,
    /// Whether the directory has contacts but active search filtered all of them out.
    pub is_filtered_empty: bool,
    /// Total contacts count in storage.
    pub total_count: usize,
    /// Number of contacts matching filters.
    pub filtered_count: usize,
    /// Total story citations count.
    pub total_citations_count: usize,
    /// Selected contact ID for detail inspection or highlight.
    pub selected_contact_id: Option<Uuid>,
}

/// Constructs the full `ContactsDirectoryViewModel` hierarchy from application state.
#[must_use]
pub fn build_contacts_directory_view(state: &AppState) -> ContactsDirectoryViewModel {
    let total_count = state.contacts.len();
    let total_citations_count = state.total_contact_citations_count();
    let contacts = build_contacts_view(state);
    let filtered_count = contacts.len();
    let is_search_active = !state.filters.search_query.trim().is_empty();
    let is_empty = total_count == 0;
    let is_filtered_empty = !is_empty && filtered_count == 0;

    let toolbar =
        ContactsToolbarViewModel::build(state, total_count, filtered_count, total_citations_count);
    let column_headers = build_contact_column_headers(&state.filters.contact_sort);

    let empty_state = if filtered_count == 0 {
        Some(ContactsEmptyStateViewModel::build(
            is_search_active || state.filters.is_active(),
            &state.filters.search_query,
        ))
    } else {
        None
    };

    ContactsDirectoryViewModel {
        toolbar,
        column_headers,
        contacts,
        empty_state,
        is_empty,
        is_filtered_empty,
        total_count,
        filtered_count,
        total_citations_count,
        selected_contact_id: state.filters.selected_contact_id,
    }
}

/// Constructs the list of sorted and filtered contact view models from application state.
#[must_use]
pub fn build_contacts_view(state: &AppState) -> Vec<ContactListItemViewModel> {
    let sorted_contacts = state.sorted_and_filtered_contacts();
    let search_query = &state.filters.search_query;

    sorted_contacts
        .into_iter()
        .map(|contact| {
            let linked_articles = state.articles_for_contact(contact.id);
            let linked_articles_count = linked_articles.len();
            let linked_articles_count_label =
                ContactListItemViewModel::format_story_count_label(linked_articles_count);

            let linked_article_slugs: Vec<String> =
                linked_articles.iter().map(|a| a.slug.clone()).collect();

            let rich_linked_articles: Vec<ContactLinkedArticleTagViewModel> = linked_articles
                .iter()
                .take(MAX_DISPLAYED_TAGGED_ARTICLES)
                .map(|article| {
                    let meta = stage_metadata(article.stage);
                    let color_hex = article
                        .color
                        .clone()
                        .unwrap_or_else(|| assign_color_for_slug(&article.slug).to_hex());

                    ContactLinkedArticleTagViewModel {
                        id: article.id,
                        slug: article.slug.clone(),
                        headline_preview: truncate_snippet(&article.headline, 30),
                        stage: article.stage,
                        stage_title: meta.title,
                        stage_accent_hex: meta.accent_hex,
                        color_hex,
                        filter_message: AppMessage::SetContactFilter(Some(contact.id)),
                    }
                })
                .collect();

            let overflow_linked_articles_count =
                linked_articles_count.saturating_sub(MAX_DISPLAYED_TAGGED_ARTICLES);

            let organization = contact.organization.clone().unwrap_or_default();
            let has_organization = !organization.trim().is_empty();

            let role = contact.role.clone().unwrap_or_default();
            let has_role = !role.trim().is_empty();

            let phone = contact.phone.clone().unwrap_or_default();
            let has_phone = !phone.trim().is_empty();
            let phone_display = if has_phone {
                format_phone_display(&phone)
            } else {
                String::new()
            };

            let email = contact.email.clone().unwrap_or_default();
            let has_email = !email.trim().is_empty();

            let notes = contact.notes.clone().unwrap_or_default();
            let has_notes = !notes.trim().is_empty();
            let notes_preview = truncate_snippet(&notes, MAX_NOTES_SNIPPET_LEN);

            let initials = format_contact_initials(&contact.name);
            let avatar_color_hex = assign_avatar_color_for_contact(contact.id, &contact.name);

            let search_highlight =
                ContactSearchHighlight::evaluate(search_query, contact, &linked_article_slugs);

            let is_search_match = search_highlight.is_some();

            ContactListItemViewModel {
                id: contact.id,
                name: contact.name.clone(),
                initials,
                avatar_color_hex,
                organization,
                has_organization,
                role,
                has_role,
                phone,
                phone_display,
                has_phone,
                email,
                has_email,
                notes,
                notes_preview,
                has_notes,
                linked_articles_count,
                linked_articles_count_label,
                linked_article_slugs,
                linked_articles: rich_linked_articles,
                overflow_linked_articles_count,
                is_search_match,
                search_highlight,
                edit_message: AppMessage::OpenEditContactModal(contact.id),
                delete_message: AppMessage::PromptDeleteContact(contact.id),
                filter_by_contact_message: AppMessage::SetContactFilter(Some(contact.id)),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use newsjournal_core::models::{Article, Contact};

    #[test]
    fn test_format_contact_initials() {
        assert_eq!(format_contact_initials("Jane Doe"), "JD");
        assert_eq!(format_contact_initials("Alice"), "Al");
        assert_eq!(format_contact_initials("Alice Smith"), "AS");
        assert_eq!(format_contact_initials("John Quincy Adams"), "JA");
        assert_eq!(format_contact_initials(""), "??");
    }

    #[test]
    fn test_truncate_snippet() {
        assert_eq!(truncate_snippet("Short snippet", 20), "Short snippet");
        assert_eq!(
            truncate_snippet("A very long snippet that exceeds the limit", 10),
            "A very lo…"
        );
    }

    #[test]
    fn test_assign_avatar_color_for_contact() {
        let id = Uuid::new_v4();
        let color1 = assign_avatar_color_for_contact(id, "Jane Doe");
        let color2 = assign_avatar_color_for_contact(id, "Jane Doe");
        assert_eq!(color1, color2);
        assert!(color1.starts_with('#'));
        assert_eq!(color1.len(), 7);
    }

    #[test]
    fn test_build_contact_column_headers() {
        let sort = ContactSortConfig::with(ContactSortField::Name, SortDirection::Ascending);
        let headers = build_contact_column_headers(&sort);
        assert_eq!(headers.len(), 6);

        let name_header = &headers[0];
        assert_eq!(name_header.field, ContactSortField::Name);
        assert!(name_header.is_sorted);
        assert_eq!(name_header.sort_direction, Some(SortDirection::Ascending));
        assert_eq!(name_header.sort_indicator, "▲");

        let org_header = &headers[1];
        assert_eq!(org_header.field, ContactSortField::Organization);
        assert!(!org_header.is_sorted);
        assert_eq!(org_header.sort_direction, None);
        assert_eq!(org_header.sort_indicator, "↕");
    }

    #[test]
    fn test_contacts_directory_view_lifecycle() {
        let mut state = AppState::in_memory().expect("in-memory state");

        // 1. Initial state with 0 contacts
        let dir_view = build_contacts_directory_view(&state);
        assert!(dir_view.is_empty);
        assert_eq!(dir_view.total_count, 0);
        assert_eq!(dir_view.filtered_count, 0);
        assert!(dir_view.empty_state.is_some());
        let empty = dir_view.empty_state.unwrap();
        assert_eq!(empty.headline, "No Contacts in Directory");
        assert!(!empty.is_filtered);

        // 2. Add contacts
        let c1 = Contact::builder("Alice Smith")
            .organization("Daily Tribune")
            .role("Senior Reporter")
            .email("alice@tribune.example.com")
            .phone("5551112222")
            .notes("Covers city hall")
            .build();

        let c2 = Contact::builder("Bob Jones")
            .organization("City Gazette")
            .role("Editor-in-Chief")
            .email("bob@gazette.example.com")
            .phone("5553334444")
            .build();

        let c1_id = c1.id;
        state.storage.create_contact(c1).expect("create c1");
        state.storage.create_contact(c2).expect("create c2");

        let art = Article::builder("subway-investigation", "Subway Investigation").build();
        let art_id = art.id;
        state.storage.create_article(art).expect("create art");
        state
            .storage
            .link_contact_to_article(art_id, c1_id)
            .expect("tag c1");

        state.load_all().expect("reload");

        let dir_view = build_contacts_directory_view(&state);
        assert!(!dir_view.is_empty);
        assert_eq!(dir_view.total_count, 2);
        assert_eq!(dir_view.filtered_count, 2);
        assert_eq!(dir_view.total_citations_count, 1);
        assert!(dir_view.empty_state.is_none());

        // Check first contact (Alice Smith sorted alphabetically)
        let alice_vm = &dir_view.contacts[0];
        assert_eq!(alice_vm.name, "Alice Smith");
        assert_eq!(alice_vm.initials, "AS");
        assert_eq!(alice_vm.organization, "Daily Tribune");
        assert_eq!(alice_vm.role, "Senior Reporter");
        assert_eq!(alice_vm.phone_display, "(555) 111-2222");
        assert_eq!(alice_vm.linked_articles_count, 1);
        assert_eq!(alice_vm.linked_articles[0].slug, "subway-investigation");

        // 3. Search filtering
        state.filters.set_search("gazette");
        let dir_view = build_contacts_directory_view(&state);
        assert_eq!(dir_view.filtered_count, 1);
        assert_eq!(dir_view.contacts[0].name, "Bob Jones");
        assert!(dir_view.contacts[0].is_search_match);
        let highlight = dir_view.contacts[0].search_highlight.as_ref().unwrap();
        assert!(highlight.matches_organization);
        assert!(!highlight.matches_name);

        // 4. Search matching tagged story
        state.filters.set_search("subway");
        let dir_view = build_contacts_directory_view(&state);
        assert_eq!(dir_view.filtered_count, 1);
        assert_eq!(dir_view.contacts[0].name, "Alice Smith");

        // 5. Search with no matches
        state.filters.set_search("nonexistent");
        let dir_view = build_contacts_directory_view(&state);
        assert_eq!(dir_view.filtered_count, 0);
        assert!(dir_view.is_filtered_empty);
        assert!(dir_view.empty_state.is_some());
        let empty = dir_view.empty_state.unwrap();
        assert_eq!(empty.headline, "No Contacts Found");
        assert!(empty.is_filtered);
    }
}
