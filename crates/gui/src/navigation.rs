//! Navigation tabs and view modes for NewsJournal.

use serde::{Deserialize, Serialize};

/// Navigation tabs displayed in the left vertical toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NavTab {
    /// Articles Kanban board with 6 production stages.
    #[default]
    ArticlesKanban,
    /// Tasks Kanban board with 3 status columns.
    TasksKanban,
    /// Contacts directory list and search.
    ContactsDirectory,
    /// Application settings and theme configuration.
    Settings,
}

impl NavTab {
    /// Returns all navigation tabs in visual order.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::ArticlesKanban,
            Self::TasksKanban,
            Self::ContactsDirectory,
            Self::Settings,
        ]
    }

    /// User-visible title of the navigation tab.
    #[must_use]
    pub const fn title(&self) -> &'static str {
        match self {
            Self::ArticlesKanban => "Articles",
            Self::TasksKanban => "Tasks",
            Self::ContactsDirectory => "Contacts",
            Self::Settings => "Settings",
        }
    }

    /// Symbolic icon name matching standard icon sets.
    #[must_use]
    pub const fn icon_name(&self) -> &'static str {
        match self {
            Self::ArticlesKanban => "newspaper",
            Self::TasksKanban => "check-square",
            Self::ContactsDirectory => "users",
            Self::Settings => "settings",
        }
    }

    /// Fallback Unicode emoji icon.
    #[must_use]
    pub const fn icon_emoji(&self) -> &'static str {
        match self {
            Self::ArticlesKanban => "📰",
            Self::TasksKanban => "✅",
            Self::ContactsDirectory => "👥",
            Self::Settings => "⚙️",
        }
    }

    /// Keyboard shortcut label (e.g. "Ctrl+1" or "⌘1").
    #[must_use]
    pub const fn shortcut_label(&self) -> &'static str {
        match self {
            Self::ArticlesKanban => "Ctrl+1",
            Self::TasksKanban => "Ctrl+2",
            Self::ContactsDirectory => "Ctrl+3",
            Self::Settings => "Ctrl+4",
        }
    }

    /// Numerical 0-based index of the tab.
    #[must_use]
    pub const fn index(&self) -> usize {
        match self {
            Self::ArticlesKanban => 0,
            Self::TasksKanban => 1,
            Self::ContactsDirectory => 2,
            Self::Settings => 3,
        }
    }

    /// Constructs a `NavTab` from a 0-based index if valid.
    #[must_use]
    pub const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::ArticlesKanban),
            1 => Some(Self::TasksKanban),
            2 => Some(Self::ContactsDirectory),
            3 => Some(Self::Settings),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nav_tab_properties() {
        assert_eq!(NavTab::default(), NavTab::ArticlesKanban);
        let all = NavTab::all();
        assert_eq!(all.len(), 4);

        for (i, tab) in all.iter().enumerate() {
            assert_eq!(tab.index(), i);
            assert_eq!(NavTab::from_index(i), Some(*tab));
            assert!(!tab.title().is_empty());
            assert!(!tab.icon_name().is_empty());
            assert!(!tab.icon_emoji().is_empty());
            assert!(!tab.shortcut_label().is_empty());
        }

        assert_eq!(NavTab::from_index(4), None);
    }
}
