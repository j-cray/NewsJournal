//! Search, filtering, and query state across articles, tasks, and contacts.

use newsjournal_core::models::ArticleStage;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Urgency filter selection for Kanban cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum UrgencyFilter {
    /// Show all cards regardless of deadline status.
    #[default]
    All,
    /// Show only cards that are overdue.
    OverdueOnly,
    /// Show only cards due soon (e.g. within 24h).
    DueSoonOnly,
    /// Show only cards with an active deadline (excluding no deadline).
    HasDeadlineOnly,
}

/// Sortable column/field for contacts directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ContactSortField {
    /// Alphabetical by contact name.
    #[default]
    Name,
    /// Alphabetical by organization affiliation.
    Organization,
    /// Alphabetical by role/beat.
    Role,
    /// Alphabetical by email address.
    Email,
    /// Alphabetical/numerical by phone number.
    Phone,
    /// Numerical by number of linked/tagged stories.
    StoriesCount,
    /// Chronological by creation timestamp.
    Recent,
}

impl ContactSortField {
    /// Returns the human-readable display label for this sort field.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::Organization => "Organization",
            Self::Role => "Role",
            Self::Email => "Email",
            Self::Phone => "Phone",
            Self::StoriesCount => "Tagged Stories",
            Self::Recent => "Recently Added",
        }
    }

    /// Returns the default sort direction for this field when first selected.
    #[must_use]
    pub const fn default_direction(&self) -> SortDirection {
        match self {
            Self::StoriesCount | Self::Recent => SortDirection::Descending,
            Self::Name | Self::Organization | Self::Role | Self::Email | Self::Phone => {
                SortDirection::Ascending
            }
        }
    }
}

/// Generic sort order direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SortDirection {
    /// Ascending order (A-Z, 0-9, oldest first).
    #[default]
    Ascending,
    /// Descending order (Z-A, 9-0, newest first).
    Descending,
}

impl SortDirection {
    /// Inverts the current sort direction.
    #[must_use]
    pub const fn toggle(&self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }

    /// Returns `true` if ascending.
    #[must_use]
    pub const fn is_ascending(&self) -> bool {
        matches!(self, Self::Ascending)
    }

    /// Returns `true` if descending.
    #[must_use]
    pub const fn is_descending(&self) -> bool {
        matches!(self, Self::Descending)
    }

    /// Visual arrow symbol indicator for the direction.
    #[must_use]
    pub const fn symbol(&self) -> &'static str {
        match self {
            Self::Ascending => "▲",
            Self::Descending => "▼",
        }
    }
}

/// Combined sort configuration for contacts directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct ContactSortConfig {
    /// Active field to sort by.
    pub field: ContactSortField,
    /// Active sort direction.
    pub direction: SortDirection,
}

impl ContactSortConfig {
    /// Creates a default `ContactSortConfig` (Name Ascending).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a sort configuration with specified field and direction.
    #[must_use]
    pub const fn with(field: ContactSortField, direction: SortDirection) -> Self {
        Self { field, direction }
    }

    /// Toggles the sort configuration:
    /// - If the selected field matches current field, flips direction.
    /// - If selecting a new field, sets field and uses its default direction.
    pub fn toggle(&mut self, field: ContactSortField) {
        if self.field == field {
            self.direction = self.direction.toggle();
        } else {
            self.field = field;
            self.direction = field.default_direction();
        }
    }

    /// Formats a concise summary label of the sort configuration (e.g. "Name (A–Z)").
    #[must_use]
    pub fn summary_label(&self) -> String {
        let field_label = self.field.label();
        match (self.field, self.direction) {
            (ContactSortField::StoriesCount, SortDirection::Descending) => {
                "Most Stories First".to_string()
            }
            (ContactSortField::StoriesCount, SortDirection::Ascending) => {
                "Fewest Stories First".to_string()
            }
            (ContactSortField::Recent, SortDirection::Descending) => "Newest First".to_string(),
            (ContactSortField::Recent, SortDirection::Ascending) => "Oldest First".to_string(),
            (_, SortDirection::Ascending) => format!("{field_label} (A–Z)"),
            (_, SortDirection::Descending) => format!("{field_label} (Z–A)"),
        }
    }
}

/// Unified query and search filter state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FilterState {
    /// Text query for searching headlines, names, slugs, or titles.
    pub search_query: String,
    /// Urgency deadline filter.
    pub urgency_filter: UrgencyFilter,
    /// Optional production stage filter.
    pub selected_stage: Option<ArticleStage>,
    /// Optional contact filter to show only stories linked to this contact.
    pub selected_contact_id: Option<Uuid>,
    /// Optional parent article filter for task listings.
    pub selected_article_id: Option<Uuid>,
    /// Active sort configuration for the contacts directory.
    pub contact_sort: ContactSortConfig,
}

impl FilterState {
    /// Creates a fresh, empty filter state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets all filters to default.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Sets the text search query.
    pub fn set_search(&mut self, query: impl Into<String>) {
        self.search_query = query.into();
    }

    /// Sets the urgency filter.
    pub fn set_urgency(&mut self, filter: UrgencyFilter) {
        self.urgency_filter = filter;
    }

    /// Sets the stage filter.
    pub fn set_stage(&mut self, stage: Option<ArticleStage>) {
        self.selected_stage = stage;
    }

    /// Sets the contacts directory sort configuration.
    pub fn set_contact_sort(&mut self, sort: ContactSortConfig) {
        self.contact_sort = sort;
    }

    /// Toggles or updates the contacts directory sort field.
    pub fn toggle_contact_sort(&mut self, field: ContactSortField) {
        self.contact_sort.toggle(field);
    }

    /// Resets the contacts directory sort to default (Name ASC).
    pub fn reset_contact_sort(&mut self) {
        self.contact_sort = ContactSortConfig::default();
    }

    /// Checks if any search or filter criteria are actively applied.
    #[must_use]
    pub fn is_active(&self) -> bool {
        !self.search_query.trim().is_empty()
            || self.urgency_filter != UrgencyFilter::All
            || self.selected_stage.is_some()
            || self.selected_contact_id.is_some()
            || self.selected_article_id.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_state_lifecycle() {
        let mut filter = FilterState::new();
        assert!(!filter.is_active());

        filter.set_search("investigation");
        assert!(filter.is_active());

        filter.clear();
        assert!(!filter.is_active());

        filter.set_urgency(UrgencyFilter::OverdueOnly);
        assert!(filter.is_active());

        filter.set_stage(Some(ArticleStage::Writing));
        assert!(filter.is_active());
    }

    #[test]
    fn test_contact_sort_config_toggle() {
        let mut sort = ContactSortConfig::new();
        assert_eq!(sort.field, ContactSortField::Name);
        assert_eq!(sort.direction, SortDirection::Ascending);
        assert_eq!(sort.summary_label(), "Name (A–Z)");

        // Toggle same field -> Descending
        sort.toggle(ContactSortField::Name);
        assert_eq!(sort.direction, SortDirection::Descending);
        assert_eq!(sort.summary_label(), "Name (Z–A)");

        // Switch to StoriesCount -> defaults to Descending
        sort.toggle(ContactSortField::StoriesCount);
        assert_eq!(sort.field, ContactSortField::StoriesCount);
        assert_eq!(sort.direction, SortDirection::Descending);
        assert_eq!(sort.summary_label(), "Most Stories First");

        // Toggle StoriesCount -> Ascending
        sort.toggle(ContactSortField::StoriesCount);
        assert_eq!(sort.direction, SortDirection::Ascending);
        assert_eq!(sort.summary_label(), "Fewest Stories First");

        // Switch to Organization -> defaults to Ascending
        sort.toggle(ContactSortField::Organization);
        assert_eq!(sort.field, ContactSortField::Organization);
        assert_eq!(sort.direction, SortDirection::Ascending);
    }
}
