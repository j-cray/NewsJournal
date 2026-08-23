//! Navigation tabs, keyboard shortcuts, and view modes for NewsJournal.

use serde::{Deserialize, Serialize};

/// Navigation tabs displayed in the left vertical toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NavTab {
    /// Articles Kanban board with 6 production stages (Main Deck).
    #[default]
    ArticlesKanban,
    /// Tasks Kanban board with 3 status columns.
    TasksKanban,
    /// Contacts directory list and search.
    ContactsDirectory,
    /// Application settings and theme configuration (docked at bottom).
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

    /// Returns the main navigation tabs positioned at the top of the sidebar.
    #[must_use]
    pub const fn main_tabs() -> &'static [Self] {
        &[
            Self::ArticlesKanban,
            Self::TasksKanban,
            Self::ContactsDirectory,
        ]
    }

    /// Returns the docked navigation tabs positioned at the bottom of the sidebar.
    #[must_use]
    pub const fn docked_tabs() -> &'static [Self] {
        &[Self::Settings]
    }

    /// Returns whether this tab is docked at the bottom of the navigation bar.
    #[must_use]
    pub const fn is_docked(&self) -> bool {
        matches!(self, Self::Settings)
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

    /// Secondary subtitle or description of the section.
    #[must_use]
    pub const fn subtitle(&self) -> &'static str {
        match self {
            Self::ArticlesKanban => "Main Deck",
            Self::TasksKanban => "Task Board",
            Self::ContactsDirectory => "Directory",
            Self::Settings => "Preferences",
        }
    }

    /// Symbolic icon name matching standard desktop icon sets.
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

    /// SF Symbols icon identifier for macOS interface rendering.
    #[must_use]
    pub const fn sf_symbol(&self) -> &'static str {
        match self {
            Self::ArticlesKanban => "doc.richtext",
            Self::TasksKanban => "checklist",
            Self::ContactsDirectory => "person.2",
            Self::Settings => "gearshape",
        }
    }

    /// Keyboard shortcut digit corresponding to this tab (1..4).
    #[must_use]
    pub const fn shortcut_digit(&self) -> char {
        match self {
            Self::ArticlesKanban => '1',
            Self::TasksKanban => '2',
            Self::ContactsDirectory => '3',
            Self::Settings => '4',
        }
    }

    /// Generic keyboard shortcut label (e.g. "Ctrl+1").
    #[must_use]
    pub const fn shortcut_label(&self) -> &'static str {
        match self {
            Self::ArticlesKanban => "Ctrl+1",
            Self::TasksKanban => "Ctrl+2",
            Self::ContactsDirectory => "Ctrl+3",
            Self::Settings => "Ctrl+4",
        }
    }

    /// Platform-aware keyboard shortcut label (macOS "⌘1" vs Linux "Ctrl+1").
    #[must_use]
    pub const fn shortcut_label_for_platform(&self, is_macos: bool) -> &'static str {
        if is_macos {
            match self {
                Self::ArticlesKanban => "⌘1",
                Self::TasksKanban => "⌘2",
                Self::ContactsDirectory => "⌘3",
                Self::Settings => "⌘4",
            }
        } else {
            self.shortcut_label()
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

    /// Constructs a `NavTab` from a numerical digit character ('1'..='4').
    #[must_use]
    pub const fn from_digit(digit: char) -> Option<Self> {
        match digit {
            '1' => Some(Self::ArticlesKanban),
            '2' => Some(Self::TasksKanban),
            '3' => Some(Self::ContactsDirectory),
            '4' => Some(Self::Settings),
            _ => None,
        }
    }

    /// Cycles to the next tab in visual sequence, wrapping back to the beginning.
    #[must_use]
    pub const fn next(&self) -> Self {
        match self {
            Self::ArticlesKanban => Self::TasksKanban,
            Self::TasksKanban => Self::ContactsDirectory,
            Self::ContactsDirectory => Self::Settings,
            Self::Settings => Self::ArticlesKanban,
        }
    }

    /// Cycles to the previous tab in visual sequence, wrapping back to the end.
    #[must_use]
    pub const fn prev(&self) -> Self {
        match self {
            Self::ArticlesKanban => Self::Settings,
            Self::TasksKanban => Self::ArticlesKanban,
            Self::ContactsDirectory => Self::TasksKanban,
            Self::Settings => Self::ContactsDirectory,
        }
    }
}

/// Keyboard modifier keys state for navigation shortcut handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct NavKeyModifiers {
    /// Control key pressed.
    pub ctrl: bool,
    /// Meta/Command (Super) key pressed.
    pub meta: bool,
    /// Alt/Option key pressed.
    pub alt: bool,
    /// Shift key pressed.
    pub shift: bool,
}

impl NavKeyModifiers {
    /// Constructs empty modifiers.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            ctrl: false,
            meta: false,
            alt: false,
            shift: false,
        }
    }

    /// Constructs modifiers with Ctrl/Cmd flag set.
    #[must_use]
    pub const fn ctrl_or_meta(ctrl: bool, meta: bool) -> Self {
        Self {
            ctrl,
            meta,
            alt: false,
            shift: false,
        }
    }

    /// Checks if the primary platform modifier key is pressed.
    /// On macOS, this checks `meta` (Command). On Linux/other, this checks `ctrl`.
    #[must_use]
    pub const fn is_primary_modifier(&self, is_macos: bool) -> bool {
        if is_macos {
            self.meta
        } else {
            self.ctrl
        }
    }
}

/// High-level navigation actions triggered via keyboard shortcuts or user input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NavKeyAction {
    /// Direct selection of a specific navigation tab.
    SelectTab(NavTab),
    /// Advance to the next tab in visual sequence.
    NextTab,
    /// Retreat to the previous tab in visual sequence.
    PrevTab,
    /// Jump directly to the first main tab (Articles).
    FirstTab,
    /// Jump directly to the last/docked tab (Settings).
    LastTab,
}

/// Resolves a keyboard key event and modifiers into a navigation action if matched.
///
/// Supported shortcuts:
/// - `Primary+1..4` (Ctrl+1..4 on Linux, ⌘1..4 on macOS): Direct tab selection.
/// - `Primary+,` (Ctrl+, or ⌘,): Jump to Settings.
/// - `Primary+]` or `PageDown` or `Down`: Next tab.
/// - `Primary+[` or `PageUp` or `Up`: Previous tab.
/// - `Primary+Tab` / `Ctrl+Tab`: Next tab (or PrevTab if Shift is held).
/// - `Home`: First tab (Articles).
/// - `End`: Last tab (Settings).
#[must_use]
pub fn resolve_nav_shortcut(
    key: &str,
    modifiers: NavKeyModifiers,
    is_macos: bool,
) -> Option<NavKeyAction> {
    let has_primary = modifiers.is_primary_modifier(is_macos);
    let key_clean = key.trim();

    // 1. Primary modifier + digit (e.g. Ctrl+1..4 or ⌘1..4) or Alt+1..4
    if has_primary || modifiers.alt {
        if let Some(first_char) = key_clean.chars().next() {
            if ('1'..='4').contains(&first_char) && key_clean.len() == 1 {
                if let Some(tab) = NavTab::from_digit(first_char) {
                    return Some(NavKeyAction::SelectTab(tab));
                }
            }
        }

        // Standard settings shortcut: Cmd+, or Ctrl+,
        if key_clean == "," {
            return Some(NavKeyAction::SelectTab(NavTab::Settings));
        }

        // Tab cycling shortcuts
        if key_clean == "Tab" || key_clean == "tab" {
            return if modifiers.shift {
                Some(NavKeyAction::PrevTab)
            } else {
                Some(NavKeyAction::NextTab)
            };
        }

        if key_clean == "]" || key_clean == "}" {
            return Some(NavKeyAction::NextTab);
        }

        if key_clean == "[" || key_clean == "{" {
            return Some(NavKeyAction::PrevTab);
        }
    }

    // 2. Directional / navigation keys
    match key_clean {
        "ArrowDown" | "Down" | "PageDown" => Some(NavKeyAction::NextTab),
        "ArrowUp" | "Up" | "PageUp" => Some(NavKeyAction::PrevTab),
        "Home" => Some(NavKeyAction::FirstTab),
        "End" => Some(NavKeyAction::LastTab),
        // Vim-style navigation when Alt modifier is active
        "j" | "J" if modifiers.alt => Some(NavKeyAction::NextTab),
        "k" | "K" if modifiers.alt => Some(NavKeyAction::PrevTab),
        _ => None,
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
            assert!(!tab.subtitle().is_empty());
            assert!(!tab.icon_name().is_empty());
            assert!(!tab.icon_emoji().is_empty());
            assert!(!tab.sf_symbol().is_empty());
            assert!(!tab.shortcut_label().is_empty());
            assert!(!tab.shortcut_label_for_platform(true).is_empty());
            assert!(!tab.shortcut_label_for_platform(false).is_empty());
            assert!(('1'..='4').contains(&tab.shortcut_digit()));
        }

        assert_eq!(NavTab::from_index(4), None);
        assert_eq!(NavTab::from_digit('5'), None);
        assert_eq!(NavTab::from_digit('a'), None);
    }

    #[test]
    fn test_main_and_docked_grouping() {
        let main = NavTab::main_tabs();
        assert_eq!(main.len(), 3);
        assert_eq!(main[0], NavTab::ArticlesKanban);
        assert_eq!(main[1], NavTab::TasksKanban);
        assert_eq!(main[2], NavTab::ContactsDirectory);

        let docked = NavTab::docked_tabs();
        assert_eq!(docked.len(), 1);
        assert_eq!(docked[0], NavTab::Settings);

        assert!(!NavTab::ArticlesKanban.is_docked());
        assert!(!NavTab::TasksKanban.is_docked());
        assert!(!NavTab::ContactsDirectory.is_docked());
        assert!(NavTab::Settings.is_docked());
    }

    #[test]
    fn test_platform_shortcut_labels() {
        assert_eq!(
            NavTab::ArticlesKanban.shortcut_label_for_platform(true),
            "⌘1"
        );
        assert_eq!(NavTab::TasksKanban.shortcut_label_for_platform(true), "⌘2");
        assert_eq!(
            NavTab::ContactsDirectory.shortcut_label_for_platform(true),
            "⌘3"
        );
        assert_eq!(NavTab::Settings.shortcut_label_for_platform(true), "⌘4");

        assert_eq!(
            NavTab::ArticlesKanban.shortcut_label_for_platform(false),
            "Ctrl+1"
        );
        assert_eq!(
            NavTab::TasksKanban.shortcut_label_for_platform(false),
            "Ctrl+2"
        );
        assert_eq!(
            NavTab::ContactsDirectory.shortcut_label_for_platform(false),
            "Ctrl+3"
        );
        assert_eq!(
            NavTab::Settings.shortcut_label_for_platform(false),
            "Ctrl+4"
        );
    }

    #[test]
    fn test_cyclic_navigation() {
        assert_eq!(NavTab::ArticlesKanban.next(), NavTab::TasksKanban);
        assert_eq!(NavTab::TasksKanban.next(), NavTab::ContactsDirectory);
        assert_eq!(NavTab::ContactsDirectory.next(), NavTab::Settings);
        assert_eq!(NavTab::Settings.next(), NavTab::ArticlesKanban);

        assert_eq!(NavTab::ArticlesKanban.prev(), NavTab::Settings);
        assert_eq!(NavTab::TasksKanban.prev(), NavTab::ArticlesKanban);
        assert_eq!(NavTab::ContactsDirectory.prev(), NavTab::TasksKanban);
        assert_eq!(NavTab::Settings.prev(), NavTab::ContactsDirectory);
    }

    #[test]
    fn test_shortcut_resolution_linux() {
        let linux_ctrl = NavKeyModifiers {
            ctrl: true,
            meta: false,
            alt: false,
            shift: false,
        };

        assert_eq!(
            resolve_nav_shortcut("1", linux_ctrl, false),
            Some(NavKeyAction::SelectTab(NavTab::ArticlesKanban))
        );
        assert_eq!(
            resolve_nav_shortcut("2", linux_ctrl, false),
            Some(NavKeyAction::SelectTab(NavTab::TasksKanban))
        );
        assert_eq!(
            resolve_nav_shortcut("3", linux_ctrl, false),
            Some(NavKeyAction::SelectTab(NavTab::ContactsDirectory))
        );
        assert_eq!(
            resolve_nav_shortcut("4", linux_ctrl, false),
            Some(NavKeyAction::SelectTab(NavTab::Settings))
        );
        assert_eq!(
            resolve_nav_shortcut(",", linux_ctrl, false),
            Some(NavKeyAction::SelectTab(NavTab::Settings))
        );
        assert_eq!(
            resolve_nav_shortcut("Tab", linux_ctrl, false),
            Some(NavKeyAction::NextTab)
        );

        let linux_ctrl_shift = NavKeyModifiers {
            ctrl: true,
            meta: false,
            alt: false,
            shift: true,
        };
        assert_eq!(
            resolve_nav_shortcut("Tab", linux_ctrl_shift, false),
            Some(NavKeyAction::PrevTab)
        );
    }

    #[test]
    fn test_shortcut_resolution_macos() {
        let macos_cmd = NavKeyModifiers {
            ctrl: false,
            meta: true,
            alt: false,
            shift: false,
        };

        assert_eq!(
            resolve_nav_shortcut("1", macos_cmd, true),
            Some(NavKeyAction::SelectTab(NavTab::ArticlesKanban))
        );
        assert_eq!(
            resolve_nav_shortcut("3", macos_cmd, true),
            Some(NavKeyAction::SelectTab(NavTab::ContactsDirectory))
        );
        assert_eq!(
            resolve_nav_shortcut(",", macos_cmd, true),
            Some(NavKeyAction::SelectTab(NavTab::Settings))
        );
        assert_eq!(
            resolve_nav_shortcut("]", macos_cmd, true),
            Some(NavKeyAction::NextTab)
        );
        assert_eq!(
            resolve_nav_shortcut("[", macos_cmd, true),
            Some(NavKeyAction::PrevTab)
        );
    }

    #[test]
    fn test_shortcut_directional_keys() {
        let none = NavKeyModifiers::none();
        assert_eq!(
            resolve_nav_shortcut("ArrowDown", none, false),
            Some(NavKeyAction::NextTab)
        );
        assert_eq!(
            resolve_nav_shortcut("ArrowUp", none, false),
            Some(NavKeyAction::PrevTab)
        );
        assert_eq!(
            resolve_nav_shortcut("Home", none, false),
            Some(NavKeyAction::FirstTab)
        );
        assert_eq!(
            resolve_nav_shortcut("End", none, false),
            Some(NavKeyAction::LastTab)
        );

        let alt = NavKeyModifiers {
            ctrl: false,
            meta: false,
            alt: true,
            shift: false,
        };
        assert_eq!(
            resolve_nav_shortcut("j", alt, false),
            Some(NavKeyAction::NextTab)
        );
        assert_eq!(
            resolve_nav_shortcut("k", alt, false),
            Some(NavKeyAction::PrevTab)
        );
    }
}
