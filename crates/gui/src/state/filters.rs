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
}
