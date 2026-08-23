//! Article Card Component presentation models, color indicator bars, task counters, contact tag pills, and overdue styling.

use chrono::{DateTime, Utc};
use newsjournal_core::color::assign_color_for_slug;
use newsjournal_core::deadline::{
    evaluate_article_deadline, format_compact_duration, format_verbose_duration, DeadlineStatus,
    UrgencyLevel,
};
use newsjournal_core::models::{Article, ArticleStage, Contact, Task, TaskStatus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;
use crate::theme::ResolvedTheme;

/// Default left color accent strip width in logical pixels.
pub const DEFAULT_ACCENT_STRIP_WIDTH: f32 = 4.0;

/// Default corner radius for article cards in logical pixels.
pub const DEFAULT_CARD_CORNER_RADIUS: f32 = 10.0;

/// Default border width for normal article cards in logical pixels.
pub const DEFAULT_CARD_BORDER_WIDTH: f32 = 1.0;

/// Prominent bold border width for overdue article cards in logical pixels.
pub const OVERDUE_CARD_BORDER_WIDTH: f32 = 2.0;

/// Overdue accent red hex color.
pub const OVERDUE_RED_HEX: &str = "#EF4444";

/// Overdue badge background tint hex.
pub const OVERDUE_BG_TINT_HEX: &str = "#7F1D1D";

/// Due soon accent amber hex color.
pub const DUE_SOON_AMBER_HEX: &str = "#F59E0B";

/// Due soon badge background tint hex.
pub const DUE_SOON_BG_TINT_HEX: &str = "#78350F";

/// Success green hex color (e.g. 100% task completion).
pub const SUCCESS_GREEN_HEX: &str = "#10B981";

/// Maximum length of headline preview snippet before truncation.
pub const MAX_HEADLINE_SNIPPET_LEN: usize = 90;

/// Maximum length of description preview snippet before truncation.
pub const MAX_DESCRIPTION_SNIPPET_LEN: usize = 120;

/// Position or display mode of the dynamic color indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndicatorPosition {
    /// Vertical colored strip docked to the left card edge.
    LeftAccentStrip,
    /// Top horizontal accent line above the card body.
    TopAccentLine,
    /// Pill / dot badge placed adjacent to the slug.
    PillBadge,
}

/// Dynamic color-coded indicator bar/badge configuration for an article card.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorIndicatorBarViewModel {
    /// Assigned article hex color (e.g. `"#3B82F6"`).
    pub color_hex: String,
    /// High-contrast foreground color suitable for rendering over this color (`"#FFFFFF"` or `"#0F172A"`).
    pub contrast_fg_hex: String,
    /// Translucent background tint hex color for badge pill surfaces.
    pub translucent_tint_hex: String,
    /// Width / thickness of the accent strip in logical pixels.
    pub width_px: f32,
    /// Indicator presentation position.
    pub position: IndicatorPosition,
}

impl ColorIndicatorBarViewModel {
    /// Creates a new `ColorIndicatorBarViewModel` with default left strip geometry.
    #[must_use]
    pub fn new(color_hex: &str) -> Self {
        let contrast_fg_hex = calculate_contrast_color(color_hex).to_string();
        let translucent_tint_hex = format!("{color_hex}26"); // ~15% alpha
        Self {
            color_hex: color_hex.to_string(),
            contrast_fg_hex,
            translucent_tint_hex,
            width_px: DEFAULT_ACCENT_STRIP_WIDTH,
            position: IndicatorPosition::LeftAccentStrip,
        }
    }
}

/// Formatted slug badge representation for the article card header.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleSlugBadgeViewModel {
    /// Raw slug identifier.
    pub raw_slug: String,
    /// Formatted slug with `#` prefix (e.g. `"#breaking-news"`).
    pub display_text: String,
    /// Truncated display text for compact viewports.
    pub compact_text: String,
}

impl ArticleSlugBadgeViewModel {
    /// Constructs a new `ArticleSlugBadgeViewModel` from raw slug.
    #[must_use]
    pub fn new(slug: &str) -> Self {
        let display_text = format!("#{slug}");
        let compact_text = if slug.len() > 18 {
            format!("#{}…", &slug[..16])
        } else {
            display_text.clone()
        };

        Self {
            raw_slug: slug.to_string(),
            display_text,
            compact_text,
        }
    }
}

/// Formatted deadline badge with countdown, relative time, and urgency visual cues.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleDeadlineBadgeViewModel {
    /// Target deadline datetime (UTC).
    pub deadline: DateTime<Utc>,
    /// Formatted absolute date string (e.g. `"Aug 24, 2026"` or `"Tomorrow 5:00 PM"`).
    pub formatted_date: String,
    /// Formatted short calendar date (e.g. `"Aug 24"`).
    pub short_date: String,
    /// Human-readable relative time description (e.g. `"Overdue by 2h"`, `"Due in 3d"`, `"Due today"`).
    pub relative_text: String,
    /// Compact relative duration (e.g. `"-2h"`, `"+3d"`, `"0m"`).
    pub compact_duration: String,
    /// Calculated urgency level.
    pub urgency: UrgencyLevel,
    /// High-level badge label text (e.g. `"OVERDUE"`, `"DUE SOON"`, `"Aug 24"`).
    pub badge_label: String,
    /// Primary badge foreground color hex.
    pub badge_fg_hex: String,
    /// Badge background fill color hex.
    pub badge_bg_hex: String,
    /// Symbolic icon name.
    pub icon_name: String,
    /// macOS SF Symbol identifier.
    pub sf_symbol: String,
    /// Overdue flag.
    pub is_overdue: bool,
    /// Due soon flag.
    pub is_due_soon: bool,
    /// Critical urgency flag.
    pub is_critical: bool,
}

/// Formatted task completion progress counter for an article card.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArticleTaskCounterViewModel {
    /// Number of completed tasks.
    pub completed: usize,
    /// Total number of tasks.
    pub total: usize,
    /// Formatted counter label (e.g. `"2/5 tasks"`, `"3/3 complete"`, `"No tasks"`).
    pub formatted_label: String,
    /// Compact counter string (e.g. `"2/5"`, `"3/3"`, `"-"`).
    pub compact_label: String,
    /// True if total > 0.
    pub has_tasks: bool,
    /// True if all tasks are complete (and total > 0).
    pub all_completed: bool,
    /// Completion ratio between `0.0` and `1.0`.
    pub progress_ratio: f32,
    /// Completion percentage integer `0..=100`.
    pub progress_percent: u8,
    /// Badge foreground color hex.
    pub badge_fg_hex: String,
    /// Badge background color hex.
    pub badge_bg_hex: String,
    /// Progress bar fill color hex.
    pub bar_fill_hex: String,
    /// Icon name.
    pub icon_name: String,
    /// macOS SF Symbol identifier.
    pub sf_symbol: String,
}

impl ArticleTaskCounterViewModel {
    /// Constructs a new `ArticleTaskCounterViewModel`.
    #[must_use]
    pub fn new(completed: usize, total: usize, is_dark: bool) -> Self {
        let has_tasks = total > 0;
        let all_completed = has_tasks && completed >= total;
        let progress_ratio = if total == 0 {
            0.0
        } else {
            (completed as f32 / total as f32).clamp(0.0, 1.0)
        };
        let progress_percent = (progress_ratio * 100.0).round() as u8;

        let (formatted_label, compact_label) = if total == 0 {
            ("No tasks".to_string(), "-".to_string())
        } else if all_completed {
            (
                format!("{completed}/{total} complete"),
                format!("{completed}/{total}"),
            )
        } else {
            (
                format!("{completed}/{total} tasks"),
                format!("{completed}/{total}"),
            )
        };

        let (badge_fg_hex, badge_bg_hex, bar_fill_hex, icon_name, sf_symbol) = if total == 0 {
            if is_dark {
                (
                    "#94A3B8".to_string(),
                    "#1E293B".to_string(),
                    "#475569".to_string(),
                    "check-square".to_string(),
                    "checklist".to_string(),
                )
            } else {
                (
                    "#64748B".to_string(),
                    "#F1F5F9".to_string(),
                    "#CBD5E1".to_string(),
                    "check-square".to_string(),
                    "checklist".to_string(),
                )
            }
        } else if all_completed {
            (
                "#10B981".to_string(),
                if is_dark {
                    "#064E3B".to_string()
                } else {
                    "#D1FAE5".to_string()
                },
                "#10B981".to_string(),
                "check-circle".to_string(),
                "checkmark.circle.fill".to_string(),
            )
        } else if is_dark {
            (
                "#60A5FA".to_string(),
                "#1E3A8A".to_string(),
                "#3B82F6".to_string(),
                "check-square".to_string(),
                "checklist".to_string(),
            )
        } else {
            (
                "#2563EB".to_string(),
                "#DBEAFE".to_string(),
                "#3B82F6".to_string(),
                "check-square".to_string(),
                "checklist".to_string(),
            )
        };

        Self {
            completed,
            total,
            formatted_label,
            compact_label,
            has_tasks,
            all_completed,
            progress_ratio,
            progress_percent,
            badge_fg_hex,
            badge_bg_hex,
            bar_fill_hex,
            icon_name,
            sf_symbol,
        }
    }
}

/// Tagged contact pill / avatar badge displayed on an article card.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleCardContactTagViewModel {
    /// Unique contact ID.
    pub id: Uuid,
    /// Full contact name.
    pub name: String,
    /// 1 or 2 letter initials (e.g. `"AS"` for Alice Smith).
    pub initials: String,
    /// Contact role if specified (e.g. `"Investigative Reporter"`).
    pub role: Option<String>,
    /// Contact organization if specified (e.g. `"The Daily Star"`).
    pub organization: Option<String>,
    /// Formatted badge label (name or compact snippet).
    pub badge_label: String,
    /// Deterministic avatar hex color generated from the contact's name/slug.
    pub avatar_color_hex: String,
    /// High-contrast text color for avatar initials.
    pub avatar_contrast_hex: String,
}

impl ArticleCardContactTagViewModel {
    /// Constructs a contact tag badge view model from a `Contact`.
    #[must_use]
    pub fn from_contact(contact: &Contact) -> Self {
        let initials = format_contact_initials(&contact.name);
        let avatar_color =
            assign_color_for_slug(&newsjournal_core::validation::slugify(&contact.name));
        let avatar_color_hex = avatar_color.to_hex();
        let avatar_contrast_hex = calculate_contrast_color(&avatar_color_hex).to_string();

        let badge_label = if contact.name.len() > 16 {
            format!("{}…", &contact.name[..14])
        } else {
            contact.name.clone()
        };

        Self {
            id: contact.id,
            name: contact.name.clone(),
            initials,
            role: contact.role.clone(),
            organization: contact.organization.clone(),
            badge_label,
            avatar_color_hex,
            avatar_contrast_hex,
        }
    }
}

/// Overdue visual alert styling and state descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArticleCardOverdueStyleViewModel {
    /// True if the card is overdue.
    pub is_overdue: bool,
    /// Border color hex (`#EF4444` when overdue).
    pub border_color_hex: String,
    /// Border stroke width in logical pixels (`2.0` when overdue, `1.0` otherwise).
    pub border_width_px: f32,
    /// Prominent alert badge text (e.g. `Some("OVERDUE".to_string())` or `None`).
    pub badge_text: Option<String>,
    /// Alert badge background hex.
    pub badge_bg_hex: String,
    /// Alert badge text foreground hex.
    pub badge_fg_hex: String,
    /// Shadow / glow highlight color hex when overdue.
    pub glow_color_hex: Option<String>,
}

impl ArticleCardOverdueStyleViewModel {
    /// Constructs the overdue styling descriptor.
    #[must_use]
    pub fn new(is_overdue: bool, is_due_soon: bool, is_dark: bool) -> Self {
        if is_overdue {
            Self {
                is_overdue: true,
                border_color_hex: OVERDUE_RED_HEX.to_string(),
                border_width_px: OVERDUE_CARD_BORDER_WIDTH,
                badge_text: Some("OVERDUE".to_string()),
                badge_bg_hex: if is_dark {
                    OVERDUE_BG_TINT_HEX.to_string()
                } else {
                    "#FEE2E2".to_string()
                },
                badge_fg_hex: OVERDUE_RED_HEX.to_string(),
                glow_color_hex: Some(if is_dark {
                    "#EF444466".to_string()
                } else {
                    "#EF444433".to_string()
                }),
            }
        } else if is_due_soon {
            Self {
                is_overdue: false,
                border_color_hex: DUE_SOON_AMBER_HEX.to_string(),
                border_width_px: DEFAULT_CARD_BORDER_WIDTH + 0.5,
                badge_text: Some("DUE SOON".to_string()),
                badge_bg_hex: if is_dark {
                    DUE_SOON_BG_TINT_HEX.to_string()
                } else {
                    "#FEF3C7".to_string()
                },
                badge_fg_hex: DUE_SOON_AMBER_HEX.to_string(),
                glow_color_hex: None,
            }
        } else {
            Self {
                is_overdue: false,
                border_color_hex: if is_dark {
                    "#334155".to_string()
                } else {
                    "#E2E8F0".to_string()
                },
                border_width_px: DEFAULT_CARD_BORDER_WIDTH,
                badge_text: None,
                badge_bg_hex: String::new(),
                badge_fg_hex: String::new(),
                glow_color_hex: None,
            }
        }
    }
}

/// Comprehensive, rich view model for an Article card in the Kanban deck.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArticleCardViewModel {
    /// Unique article ID.
    pub id: Uuid,
    /// Slug identifier.
    pub slug: String,
    /// Formatted slug badge representation.
    pub slug_badge: ArticleSlugBadgeViewModel,
    /// Full article headline.
    pub headline: String,
    /// Shortened headline for compact displays.
    pub headline_snippet: String,
    /// Short preview snippet of article description/notes if present.
    pub description_snippet: Option<String>,
    /// Active workflow stage.
    pub stage: ArticleStage,
    /// Assigned accent hex color.
    pub color_hex: String,
    /// Dynamic color-coded indicator bar/badge.
    pub color_indicator: ColorIndicatorBarViewModel,
    /// Optional target deadline datetime (UTC).
    pub deadline: Option<DateTime<Utc>>,
    /// Calculated deadline status enum.
    pub deadline_status: DeadlineStatus,
    /// Formatted deadline badge with relative and absolute countdown cues.
    pub deadline_badge: Option<ArticleDeadlineBadgeViewModel>,
    /// Overdue flag for prominent red alert styling.
    pub is_overdue: bool,
    /// Due soon flag for amber warning styling.
    pub is_due_soon: bool,
    /// Overdue visual alert styling and state descriptor.
    pub overdue_style: ArticleCardOverdueStyleViewModel,
    /// Task completion counter view model.
    pub task_counter: ArticleTaskCounterViewModel,
    /// Number of completed tasks.
    pub task_completed: usize,
    /// Total number of tasks.
    pub task_total: usize,
    /// Tagged contact pills for this article.
    pub tagged_contacts: Vec<ArticleCardContactTagViewModel>,
    /// Total number of contacts tagged on this story.
    pub tagged_contact_count: usize,
    /// True if this card is currently being dragged.
    pub is_dragging: bool,
    /// True if this card is actively hovered.
    pub is_hovered: bool,
}

impl ArticleCardViewModel {
    /// Constructs a complete `ArticleCardViewModel` from an `Article` and the full `AppState`.
    #[must_use]
    pub fn build(article: &Article, state: &AppState) -> Self {
        let tasks = state.tasks_for_article(article.id);
        let contacts = state.contacts_for_article(article.id);
        let now = state.last_tick;
        let is_dark = state.resolved_theme() == ResolvedTheme::Dark;

        let active_drag_id = match state.drag.active_item {
            Some(crate::state::drag_drop::DragItem::ArticleCard { id, .. }) => Some(id),
            _ => None,
        };

        let is_dragging = active_drag_id == Some(article.id);
        let is_hovered = false;

        Self::build_with_context(
            article,
            &tasks,
            &contacts,
            now,
            is_dark,
            is_dragging,
            is_hovered,
        )
    }

    /// Constructs an `ArticleCardViewModel` with explicit context inputs for isolated testing and custom pipelines.
    #[must_use]
    pub fn build_with_context(
        article: &Article,
        tasks: &[&Task],
        contacts: &[&Contact],
        now: DateTime<Utc>,
        is_dark: bool,
        is_dragging: bool,
        is_hovered: bool,
    ) -> Self {
        let slug_badge = ArticleSlugBadgeViewModel::new(&article.slug);

        let color_hex = article
            .color
            .clone()
            .unwrap_or_else(|| assign_color_for_slug(&article.slug).to_hex());

        let color_indicator = ColorIndicatorBarViewModel::new(&color_hex);

        let headline_snippet = if article.headline.len() > MAX_HEADLINE_SNIPPET_LEN {
            format!("{}…", &article.headline[..MAX_HEADLINE_SNIPPET_LEN - 3])
        } else {
            article.headline.clone()
        };

        let description_snippet = article.description.as_ref().and_then(|desc| {
            let trimmed = desc.trim();
            if trimmed.is_empty() {
                None
            } else if trimmed.len() > MAX_DESCRIPTION_SNIPPET_LEN {
                Some(format!("{}…", &trimmed[..MAX_DESCRIPTION_SNIPPET_LEN - 3]))
            } else {
                Some(trimmed.to_string())
            }
        });

        let deadline_status = evaluate_article_deadline(article, now);
        let is_overdue = deadline_status.is_overdue();
        let is_due_soon = deadline_status.is_due_soon();

        let deadline_badge = format_deadline_badge(
            article.deadline,
            article.stage,
            deadline_status,
            now,
            is_dark,
        );

        let overdue_style = ArticleCardOverdueStyleViewModel::new(is_overdue, is_due_soon, is_dark);

        let task_total = tasks.len();
        let task_completed = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Complete)
            .count();
        let task_counter = ArticleTaskCounterViewModel::new(task_completed, task_total, is_dark);

        let tagged_contacts: Vec<ArticleCardContactTagViewModel> = contacts
            .iter()
            .map(|&contact| ArticleCardContactTagViewModel::from_contact(contact))
            .collect();
        let tagged_contact_count = tagged_contacts.len();

        Self {
            id: article.id,
            slug: article.slug.clone(),
            slug_badge,
            headline: article.headline.clone(),
            headline_snippet,
            description_snippet,
            stage: article.stage,
            color_hex,
            color_indicator,
            deadline: article.deadline,
            deadline_status,
            deadline_badge,
            is_overdue,
            is_due_soon,
            overdue_style,
            task_counter,
            task_completed,
            task_total,
            tagged_contacts,
            tagged_contact_count,
            is_dragging,
            is_hovered,
        }
    }
}

/// Helper function to compute high-contrast foreground color (`#FFFFFF` or `#0F172A`) for any hex background.
#[must_use]
pub fn calculate_contrast_color(hex: &str) -> &'static str {
    let clean_hex = hex.trim_start_matches('#');
    if clean_hex.len() >= 6 {
        let r = u8::from_str_radix(&clean_hex[0..2], 16).unwrap_or(128) as f32;
        let g = u8::from_str_radix(&clean_hex[2..4], 16).unwrap_or(128) as f32;
        let b = u8::from_str_radix(&clean_hex[4..6], 16).unwrap_or(128) as f32;

        // Standard sRGB luminance approximation
        let luminance = (0.299 * r) + (0.587 * g) + (0.114 * b);
        if luminance > 155.0 {
            "#0F172A" // Dark slate foreground for bright backgrounds
        } else {
            "#FFFFFF" // White foreground for darker backgrounds
        }
    } else {
        "#FFFFFF"
    }
}

/// Helper function to format 1 or 2 letter initials for a contact name.
#[must_use]
pub fn format_contact_initials(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    match words.len() {
        0 => "??".to_string(),
        1 => {
            let chars: Vec<char> = words[0].chars().collect();
            if chars.len() >= 2 {
                format!("{}{}", chars[0].to_uppercase(), chars[1].to_lowercase())
            } else if !chars.is_empty() {
                chars[0].to_uppercase().to_string()
            } else {
                "?".to_string()
            }
        }
        _ => {
            let first = words[0].chars().next().unwrap_or('?').to_uppercase();
            let last = words[words.len() - 1]
                .chars()
                .next()
                .unwrap_or('?')
                .to_uppercase();
            format!("{first}{last}")
        }
    }
}

/// Formats the deadline badge view model based on status and time delta.
#[must_use]
pub fn format_deadline_badge(
    deadline: Option<DateTime<Utc>>,
    stage: ArticleStage,
    status: DeadlineStatus,
    now: DateTime<Utc>,
    is_dark: bool,
) -> Option<ArticleDeadlineBadgeViewModel> {
    let dl = deadline?;

    if stage == ArticleStage::Published {
        // Published stories do not show urgent countdown badges
        return Some(ArticleDeadlineBadgeViewModel {
            deadline: dl,
            formatted_date: dl.format("%b %-d, %Y").to_string(),
            short_date: dl.format("%b %-d").to_string(),
            relative_text: "Published".to_string(),
            compact_duration: "done".to_string(),
            urgency: UrgencyLevel::None,
            badge_label: "Published".to_string(),
            badge_fg_hex: if is_dark {
                "#94A3B8".to_string()
            } else {
                "#64748B".to_string()
            },
            badge_bg_hex: if is_dark {
                "#1E293B".to_string()
            } else {
                "#F1F5F9".to_string()
            },
            icon_name: "archive".to_string(),
            sf_symbol: "newspaper".to_string(),
            is_overdue: false,
            is_due_soon: false,
            is_critical: false,
        });
    }

    let is_overdue = status.is_overdue();
    let is_due_soon = status.is_due_soon();
    let is_critical = status.is_critical();
    let urgency = status.urgency_level();

    let duration_delta = dl.signed_duration_since(now);
    let relative_text = format_verbose_duration(duration_delta);
    let compact_duration = format_compact_duration(duration_delta);

    let formatted_date = dl.format("%b %-d, %Y %H:%M UTC").to_string();
    let short_date = dl.format("%b %-d").to_string();

    let (badge_label, badge_fg_hex, badge_bg_hex, icon_name, sf_symbol) = match urgency {
        UrgencyLevel::Overdue => (
            "OVERDUE".to_string(),
            OVERDUE_RED_HEX.to_string(),
            if is_dark {
                OVERDUE_BG_TINT_HEX.to_string()
            } else {
                "#FEE2E2".to_string()
            },
            "alert-circle".to_string(),
            "exclamationmark.triangle.fill".to_string(),
        ),
        UrgencyLevel::High => (
            "CRITICAL".to_string(),
            "#F97316".to_string(), // orange-500
            if is_dark {
                "#7C2D12".to_string()
            } else {
                "#FFEDD5".to_string()
            },
            "alert-triangle".to_string(),
            "exclamationmark.circle.fill".to_string(),
        ),
        UrgencyLevel::Medium => (
            "DUE SOON".to_string(),
            DUE_SOON_AMBER_HEX.to_string(),
            if is_dark {
                DUE_SOON_BG_TINT_HEX.to_string()
            } else {
                "#FEF3C7".to_string()
            },
            "clock".to_string(),
            "clock.badge.exclamationmark".to_string(),
        ),
        UrgencyLevel::Low => (
            short_date.clone(),
            if is_dark {
                "#93C5FD".to_string()
            } else {
                "#2563EB".to_string()
            },
            if is_dark {
                "#1E3A8A".to_string()
            } else {
                "#DBEAFE".to_string()
            },
            "calendar".to_string(),
            "calendar".to_string(),
        ),
        UrgencyLevel::None => (
            short_date.clone(),
            if is_dark {
                "#94A3B8".to_string()
            } else {
                "#64748B".to_string()
            },
            if is_dark {
                "#1E293B".to_string()
            } else {
                "#F1F5F9".to_string()
            },
            "calendar".to_string(),
            "calendar".to_string(),
        ),
    };

    Some(ArticleDeadlineBadgeViewModel {
        deadline: dl,
        formatted_date,
        short_date,
        relative_text,
        compact_duration,
        urgency,
        badge_label,
        badge_fg_hex,
        badge_bg_hex,
        icon_name,
        sf_symbol,
        is_overdue,
        is_due_soon,
        is_critical,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use newsjournal_core::models::ArticleBuilder;

    #[test]
    fn test_calculate_contrast_color() {
        assert_eq!(calculate_contrast_color("#FFFFFF"), "#0F172A");
        assert_eq!(calculate_contrast_color("#F59E0B"), "#0F172A");
        assert_eq!(calculate_contrast_color("#000000"), "#FFFFFF");
        assert_eq!(calculate_contrast_color("#1E3A8A"), "#FFFFFF");
        assert_eq!(calculate_contrast_color("#EF4444"), "#FFFFFF");
    }

    #[test]
    fn test_format_contact_initials() {
        assert_eq!(format_contact_initials("Alice"), "Al");
        assert_eq!(format_contact_initials("Alice Smith"), "AS");
        assert_eq!(format_contact_initials("John Quincy Adams"), "JA");
        assert_eq!(format_contact_initials(""), "??");
        assert_eq!(format_contact_initials("A"), "A");
    }

    #[test]
    fn test_color_indicator_bar_view_model() {
        let indicator = ColorIndicatorBarViewModel::new("#3B82F6");
        assert_eq!(indicator.color_hex, "#3B82F6");
        assert_eq!(indicator.contrast_fg_hex, "#FFFFFF");
        assert_eq!(indicator.width_px, DEFAULT_ACCENT_STRIP_WIDTH);
        assert_eq!(indicator.position, IndicatorPosition::LeftAccentStrip);
    }

    #[test]
    fn test_article_slug_badge_view_model() {
        let badge = ArticleSlugBadgeViewModel::new("energy-crisis");
        assert_eq!(badge.raw_slug, "energy-crisis");
        assert_eq!(badge.display_text, "#energy-crisis");
        assert_eq!(badge.compact_text, "#energy-crisis");

        let long_badge = ArticleSlugBadgeViewModel::new("super-long-investigative-slug-name");
        assert_eq!(long_badge.compact_text, "#super-long-inves…");
    }

    #[test]
    fn test_task_counter_view_model() {
        let no_tasks = ArticleTaskCounterViewModel::new(0, 0, true);
        assert!(!no_tasks.has_tasks);
        assert_eq!(no_tasks.formatted_label, "No tasks");
        assert_eq!(no_tasks.progress_ratio, 0.0);

        let partial = ArticleTaskCounterViewModel::new(2, 5, true);
        assert!(partial.has_tasks);
        assert!(!partial.all_completed);
        assert_eq!(partial.formatted_label, "2/5 tasks");
        assert_eq!(partial.progress_percent, 40);

        let completed = ArticleTaskCounterViewModel::new(3, 3, false);
        assert!(completed.all_completed);
        assert_eq!(completed.formatted_label, "3/3 complete");
        assert_eq!(completed.progress_percent, 100);
        assert_eq!(completed.badge_fg_hex, "#10B981");
    }

    #[test]
    fn test_overdue_and_due_soon_styles() {
        let overdue = ArticleCardOverdueStyleViewModel::new(true, false, true);
        assert!(overdue.is_overdue);
        assert_eq!(overdue.border_width_px, OVERDUE_CARD_BORDER_WIDTH);
        assert_eq!(overdue.border_color_hex, OVERDUE_RED_HEX);
        assert_eq!(overdue.badge_text, Some("OVERDUE".to_string()));

        let due_soon = ArticleCardOverdueStyleViewModel::new(false, true, true);
        assert!(!due_soon.is_overdue);
        assert_eq!(due_soon.border_color_hex, DUE_SOON_AMBER_HEX);
        assert_eq!(due_soon.badge_text, Some("DUE SOON".to_string()));

        let normal = ArticleCardOverdueStyleViewModel::new(false, false, true);
        assert!(!normal.is_overdue);
        assert_eq!(normal.border_width_px, DEFAULT_CARD_BORDER_WIDTH);
        assert_eq!(normal.badge_text, None);
    }

    #[test]
    fn test_deadline_badge_formatting() {
        let now = Utc::now();

        // Overdue article
        let past = now - Duration::hours(3);
        let status = DeadlineStatus::Overdue {
            duration: Duration::hours(3),
        };
        let badge = format_deadline_badge(Some(past), ArticleStage::Writing, status, now, true)
            .expect("badge exists");
        assert!(badge.is_overdue);
        assert_eq!(badge.urgency, UrgencyLevel::Overdue);
        assert_eq!(badge.badge_label, "OVERDUE");

        // Published article
        let pub_badge =
            format_deadline_badge(Some(past), ArticleStage::Published, status, now, true)
                .expect("badge exists");
        assert!(!pub_badge.is_overdue);
        assert_eq!(pub_badge.badge_label, "Published");
    }

    #[test]
    fn test_full_article_card_view_model_builder() {
        let now = Utc::now();
        let article = ArticleBuilder::new("clean-energy", "Clean Energy Revolution")
            .description("Deep dive into solar and wind grid integration")
            .stage(ArticleStage::Writing)
            .color("#06B6D4")
            .deadline(now + Duration::hours(12))
            .build();

        let task1 = Task::new(article.id, "Interview grid operators");
        let mut task2 = Task::new(article.id, "Fact check stats");
        task2.status = TaskStatus::Complete;
        let tasks = vec![&task1, &task2];

        let contact = Contact::new("Dr. Elena Vance");
        let contacts = vec![&contact];

        let card = ArticleCardViewModel::build_with_context(
            &article, &tasks, &contacts, now, true, false, false,
        );

        assert_eq!(card.slug, "clean-energy");
        assert_eq!(card.slug_badge.display_text, "#clean-energy");
        assert_eq!(card.headline, "Clean Energy Revolution");
        assert_eq!(card.color_hex, "#06B6D4");
        assert_eq!(card.task_completed, 1);
        assert_eq!(card.task_total, 2);
        assert_eq!(card.task_counter.formatted_label, "1/2 tasks");
        assert_eq!(card.tagged_contacts.len(), 1);
        assert_eq!(card.tagged_contacts[0].name, "Dr. Elena Vance");
        assert_eq!(card.tagged_contacts[0].initials, "DV");
        assert!(card.is_due_soon);
        assert!(!card.is_overdue);
        assert!(card.deadline_badge.is_some());
    }
}
