//! Deadline status, urgency classifications, and badge formatting.

use std::fmt;

use chrono::Duration;
use serde::{Deserialize, Serialize};

/// High-level urgency rating for ordering, badge styling, and visual alerting.
///
/// # Examples
///
/// ```
/// use newsjournal_core::deadline::UrgencyLevel;
///
/// assert!(UrgencyLevel::Overdue > UrgencyLevel::High);
/// assert!(UrgencyLevel::High > UrgencyLevel::Medium);
/// assert!(UrgencyLevel::Medium > UrgencyLevel::Low);
/// assert!(UrgencyLevel::Low > UrgencyLevel::None);
/// ```
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum UrgencyLevel {
    /// No active deadline tracking required (no deadline or already published/completed).
    #[default]
    None,
    /// Deadline is well in the future (> 24 hours away by default).
    Low,
    /// Deadline is approaching soon (within 24 hours by default).
    Medium,
    /// Deadline is critically imminent (within 1 hour by default).
    High,
    /// Deadline has already expired.
    Overdue,
}

impl UrgencyLevel {
    /// Returns the display string for the urgency level.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Overdue => "overdue",
        }
    }

    /// Returns `true` if this urgency level requires user action/attention (Medium, High, or Overdue).
    #[must_use]
    pub const fn requires_attention(&self) -> bool {
        matches!(self, Self::Medium | Self::High | Self::Overdue)
    }

    /// Returns `true` if the item is in an expired / overdue state.
    #[must_use]
    pub const fn is_overdue(&self) -> bool {
        matches!(self, Self::Overdue)
    }
}

impl fmt::Display for UrgencyLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Evaluated deadline status describing the temporal state of an article or task.
///
/// # Examples
///
/// ```
/// use chrono::Duration;
/// use newsjournal_core::deadline::DeadlineStatus;
///
/// let overdue = DeadlineStatus::Overdue {
///     duration: Duration::hours(2),
/// };
/// assert!(overdue.is_overdue());
/// assert_eq!(overdue.badge_text(), "Overdue (2h)");
/// assert_eq!(overdue.badge_color_hint(), "red");
///
/// let due_soon = DeadlineStatus::DueSoon {
///     remaining: Duration::hours(5),
/// };
/// assert!(due_soon.is_due_soon());
/// assert_eq!(due_soon.badge_text(), "Due in 5h");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum DeadlineStatus {
    /// Entity does not have a deadline configured.
    NoDeadline,
    /// Entity has been completed or published and is no longer subject to deadline alerts.
    Completed,
    /// Deadline has passed by the specified elapsed duration.
    Overdue {
        /// Elapsed time since the deadline passed.
        duration: Duration,
    },
    /// Deadline is imminent within the critical window (e.g. <= 1 hour).
    Critical {
        /// Time remaining until the deadline.
        remaining: Duration,
    },
    /// Deadline is approaching within the due-soon window (e.g. <= 24 hours).
    DueSoon {
        /// Time remaining until the deadline.
        remaining: Duration,
    },
    /// Deadline is safely on track in the future (> 24 hours away).
    OnTrack {
        /// Time remaining until the deadline.
        remaining: Duration,
    },
}

impl DeadlineStatus {
    /// Returns `true` if the deadline has expired and the item is overdue.
    #[must_use]
    pub const fn is_overdue(&self) -> bool {
        matches!(self, Self::Overdue { .. })
    }

    /// Returns `true` if the deadline is approaching soon (either `DueSoon` or `Critical`).
    #[must_use]
    pub const fn is_due_soon(&self) -> bool {
        matches!(self, Self::DueSoon { .. } | Self::Critical { .. })
    }

    /// Returns `true` if the deadline is in a critical window.
    #[must_use]
    pub const fn is_critical(&self) -> bool {
        matches!(self, Self::Critical { .. })
    }

    /// Returns `true` if the deadline is on track and far enough in the future.
    #[must_use]
    pub const fn is_on_track(&self) -> bool {
        matches!(self, Self::OnTrack { .. })
    }

    /// Returns `true` if the item is finished/published.
    #[must_use]
    pub const fn is_completed(&self) -> bool {
        matches!(self, Self::Completed)
    }

    /// Returns `true` if a deadline was set on the entity (even if expired or completed).
    #[must_use]
    pub const fn has_deadline(&self) -> bool {
        !matches!(self, Self::NoDeadline)
    }

    /// Returns `true` if the item is actively tracked against a deadline (not completed and has a deadline).
    #[must_use]
    pub const fn is_active_deadline(&self) -> bool {
        matches!(
            self,
            Self::Overdue { .. }
                | Self::Critical { .. }
                | Self::DueSoon { .. }
                | Self::OnTrack { .. }
        )
    }

    /// Returns the remaining duration until the deadline, or `None` if not an active future deadline.
    #[must_use]
    pub const fn remaining_time(&self) -> Option<Duration> {
        match self {
            Self::Critical { remaining }
            | Self::DueSoon { remaining }
            | Self::OnTrack { remaining } => Some(*remaining),
            Self::NoDeadline | Self::Completed | Self::Overdue { .. } => None,
        }
    }

    /// Returns the duration by which the item is overdue, or `None` if not overdue.
    #[must_use]
    pub const fn overdue_duration(&self) -> Option<Duration> {
        match self {
            Self::Overdue { duration } => Some(*duration),
            Self::NoDeadline
            | Self::Completed
            | Self::Critical { .. }
            | Self::DueSoon { .. }
            | Self::OnTrack { .. } => None,
        }
    }

    /// Maps this status into its corresponding [`UrgencyLevel`].
    #[must_use]
    pub const fn urgency_level(&self) -> UrgencyLevel {
        match self {
            Self::NoDeadline | Self::Completed => UrgencyLevel::None,
            Self::OnTrack { .. } => UrgencyLevel::Low,
            Self::DueSoon { .. } => UrgencyLevel::Medium,
            Self::Critical { .. } => UrgencyLevel::High,
            Self::Overdue { .. } => UrgencyLevel::Overdue,
        }
    }

    /// Returns a recommended color key for UI badge rendering (`"red"`, `"orange"`, `"amber"`, `"emerald"`, `"gray"`).
    #[must_use]
    pub const fn badge_color_hint(&self) -> &'static str {
        match self {
            Self::Overdue { .. } => "red",
            Self::Critical { .. } => "orange",
            Self::DueSoon { .. } => "amber",
            Self::OnTrack { .. } => "emerald",
            Self::Completed => "slate",
            Self::NoDeadline => "gray",
        }
    }

    /// Produces a compact, user-friendly badge label suitable for Kanban cards and tables.
    ///
    /// # Examples
    ///
    /// - `Overdue { duration: 2h }` -> `"Overdue (2h)"`
    /// - `Critical { remaining: 35m }` -> `"Due in 35m"`
    /// - `DueSoon { remaining: 18h }` -> `"Due in 18h"`
    /// - `OnTrack { remaining: 3d }` -> `"Due in 3d"`
    /// - `Completed` -> `"Published"` / `"Completed"`
    /// - `NoDeadline` -> `"No Deadline"`
    #[must_use]
    pub fn badge_text(&self) -> String {
        match self {
            Self::NoDeadline => "No Deadline".to_string(),
            Self::Completed => "Completed".to_string(),
            Self::Overdue { duration } => {
                format!("Overdue ({})", format_compact_duration(*duration))
            }
            Self::Critical { remaining } | Self::DueSoon { remaining } => {
                format!("Due in {}", format_compact_duration(*remaining))
            }
            Self::OnTrack { remaining } => {
                format!("Due in {}", format_compact_duration(*remaining))
            }
        }
    }

    /// Produces an extended human-readable description of the deadline status.
    #[must_use]
    pub fn human_relative(&self) -> String {
        match self {
            Self::NoDeadline => "No deadline configured".to_string(),
            Self::Completed => "Completed and archived".to_string(),
            Self::Overdue { duration } => {
                format!("Overdue by {}", format_verbose_duration(*duration))
            }
            Self::Critical { remaining } => {
                format!("Critical: due in {}", format_verbose_duration(*remaining))
            }
            Self::DueSoon { remaining } => {
                format!("Due soon: in {}", format_verbose_duration(*remaining))
            }
            Self::OnTrack { remaining } => {
                format!("On track: due in {}", format_verbose_duration(*remaining))
            }
        }
    }
}

impl fmt::Display for DeadlineStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.badge_text())
    }
}

/// Formats a duration into a compact token like `"45s"`, `"12m"`, `"5h"`, `"3d"`, `"2w"`.
#[must_use]
pub fn format_compact_duration(duration: Duration) -> String {
    let total_secs = duration.num_seconds().max(0);
    if total_secs < 60 {
        format!("{total_secs}s")
    } else {
        let total_mins = duration.num_minutes();
        if total_mins < 60 {
            format!("{total_mins}m")
        } else {
            let total_hours = duration.num_hours();
            if total_hours < 24 {
                format!("{total_hours}h")
            } else {
                let total_days = duration.num_days();
                if total_days < 7 {
                    format!("{total_days}d")
                } else {
                    let total_weeks = total_days / 7;
                    format!("{total_weeks}w")
                }
            }
        }
    }
}

/// Formats a duration into a verbose readable string like `"45 seconds"`, `"12 minutes"`, `"5 hours"`, `"3 days"`.
#[must_use]
pub fn format_verbose_duration(duration: Duration) -> String {
    let total_secs = duration.num_seconds().max(0);
    if total_secs < 60 {
        if total_secs == 1 {
            "1 second".to_string()
        } else {
            format!("{total_secs} seconds")
        }
    } else {
        let total_mins = duration.num_minutes();
        if total_mins < 60 {
            if total_mins == 1 {
                "1 minute".to_string()
            } else {
                format!("{total_mins} minutes")
            }
        } else {
            let total_hours = duration.num_hours();
            if total_hours < 24 {
                if total_hours == 1 {
                    "1 hour".to_string()
                } else {
                    format!("{total_hours} hours")
                }
            } else {
                let total_days = duration.num_days();
                if total_days < 7 {
                    if total_days == 1 {
                        "1 day".to_string()
                    } else {
                        format!("{total_days} days")
                    }
                } else {
                    let total_weeks = total_days / 7;
                    if total_weeks == 1 {
                        "1 week".to_string()
                    } else {
                        format!("{total_weeks} weeks")
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urgency_ordering() {
        assert!(UrgencyLevel::Overdue > UrgencyLevel::High);
        assert!(UrgencyLevel::High > UrgencyLevel::Medium);
        assert!(UrgencyLevel::Medium > UrgencyLevel::Low);
        assert!(UrgencyLevel::Low > UrgencyLevel::None);

        assert!(UrgencyLevel::Overdue.is_overdue());
        assert!(!UrgencyLevel::High.is_overdue());

        assert!(UrgencyLevel::Overdue.requires_attention());
        assert!(UrgencyLevel::High.requires_attention());
        assert!(UrgencyLevel::Medium.requires_attention());
        assert!(!UrgencyLevel::Low.requires_attention());
        assert!(!UrgencyLevel::None.requires_attention());
    }

    #[test]
    fn test_urgency_level_as_str_and_display() {
        assert_eq!(UrgencyLevel::None.as_str(), "none");
        assert_eq!(UrgencyLevel::Low.as_str(), "low");
        assert_eq!(UrgencyLevel::Medium.as_str(), "medium");
        assert_eq!(UrgencyLevel::High.as_str(), "high");
        assert_eq!(UrgencyLevel::Overdue.as_str(), "overdue");

        assert_eq!(format!("{}", UrgencyLevel::High), "high");
    }

    #[test]
    fn test_status_predicates() {
        let no_dl = DeadlineStatus::NoDeadline;
        assert!(!no_dl.is_overdue());
        assert!(!no_dl.is_due_soon());
        assert!(!no_dl.is_critical());
        assert!(!no_dl.is_on_track());
        assert!(!no_dl.is_completed());
        assert!(!no_dl.has_deadline());
        assert!(!no_dl.is_active_deadline());
        assert_eq!(no_dl.urgency_level(), UrgencyLevel::None);
        assert_eq!(no_dl.badge_color_hint(), "gray");
        assert_eq!(no_dl.badge_text(), "No Deadline");
        assert_eq!(no_dl.remaining_time(), None);
        assert_eq!(no_dl.overdue_duration(), None);

        let completed = DeadlineStatus::Completed;
        assert!(completed.is_completed());
        assert!(!completed.is_overdue());
        assert!(completed.has_deadline());
        assert!(!completed.is_active_deadline());
        assert_eq!(completed.urgency_level(), UrgencyLevel::None);
        assert_eq!(completed.badge_color_hint(), "slate");
        assert_eq!(completed.badge_text(), "Completed");

        let overdue = DeadlineStatus::Overdue {
            duration: Duration::hours(3),
        };
        assert!(overdue.is_overdue());
        assert!(!overdue.is_due_soon());
        assert!(overdue.has_deadline());
        assert!(overdue.is_active_deadline());
        assert_eq!(overdue.urgency_level(), UrgencyLevel::Overdue);
        assert_eq!(overdue.overdue_duration(), Some(Duration::hours(3)));
        assert_eq!(overdue.remaining_time(), None);
        assert_eq!(overdue.badge_color_hint(), "red");
        assert_eq!(overdue.badge_text(), "Overdue (3h)");

        let critical = DeadlineStatus::Critical {
            remaining: Duration::minutes(25),
        };
        assert!(!critical.is_overdue());
        assert!(critical.is_due_soon());
        assert!(critical.is_critical());
        assert!(critical.is_active_deadline());
        assert_eq!(critical.urgency_level(), UrgencyLevel::High);
        assert_eq!(critical.remaining_time(), Some(Duration::minutes(25)));
        assert_eq!(critical.overdue_duration(), None);
        assert_eq!(critical.badge_color_hint(), "orange");
        assert_eq!(critical.badge_text(), "Due in 25m");

        let due_soon = DeadlineStatus::DueSoon {
            remaining: Duration::hours(14),
        };
        assert!(!due_soon.is_overdue());
        assert!(due_soon.is_due_soon());
        assert!(!due_soon.is_critical());
        assert_eq!(due_soon.urgency_level(), UrgencyLevel::Medium);
        assert_eq!(due_soon.badge_color_hint(), "amber");
        assert_eq!(due_soon.badge_text(), "Due in 14h");

        let on_track = DeadlineStatus::OnTrack {
            remaining: Duration::days(5),
        };
        assert!(on_track.is_on_track());
        assert!(!on_track.is_due_soon());
        assert_eq!(on_track.urgency_level(), UrgencyLevel::Low);
        assert_eq!(on_track.badge_color_hint(), "emerald");
        assert_eq!(on_track.badge_text(), "Due in 5d");
    }

    #[test]
    fn test_format_compact_and_verbose_duration() {
        assert_eq!(format_compact_duration(Duration::seconds(30)), "30s");
        assert_eq!(format_compact_duration(Duration::minutes(15)), "15m");
        assert_eq!(format_compact_duration(Duration::hours(6)), "6h");
        assert_eq!(format_compact_duration(Duration::days(4)), "4d");
        assert_eq!(format_compact_duration(Duration::days(21)), "3w");

        assert_eq!(format_verbose_duration(Duration::seconds(1)), "1 second");
        assert_eq!(format_verbose_duration(Duration::seconds(45)), "45 seconds");
        assert_eq!(format_verbose_duration(Duration::minutes(1)), "1 minute");
        assert_eq!(format_verbose_duration(Duration::minutes(30)), "30 minutes");
        assert_eq!(format_verbose_duration(Duration::hours(1)), "1 hour");
        assert_eq!(format_verbose_duration(Duration::hours(8)), "8 hours");
        assert_eq!(format_verbose_duration(Duration::days(1)), "1 day");
        assert_eq!(format_verbose_duration(Duration::days(5)), "5 days");
        assert_eq!(format_verbose_duration(Duration::days(7)), "1 week");
        assert_eq!(format_verbose_duration(Duration::days(28)), "4 weeks");
    }

    #[test]
    fn test_human_relative_formatting() {
        let no_dl = DeadlineStatus::NoDeadline;
        assert_eq!(no_dl.human_relative(), "No deadline configured");

        let completed = DeadlineStatus::Completed;
        assert_eq!(completed.human_relative(), "Completed and archived");

        let overdue = DeadlineStatus::Overdue {
            duration: Duration::hours(2),
        };
        assert_eq!(overdue.human_relative(), "Overdue by 2 hours");

        let critical = DeadlineStatus::Critical {
            remaining: Duration::minutes(30),
        };
        assert_eq!(critical.human_relative(), "Critical: due in 30 minutes");

        let due_soon = DeadlineStatus::DueSoon {
            remaining: Duration::hours(5),
        };
        assert_eq!(due_soon.human_relative(), "Due soon: in 5 hours");

        let on_track = DeadlineStatus::OnTrack {
            remaining: Duration::days(2),
        };
        assert_eq!(on_track.human_relative(), "On track: due in 2 days");
    }

    #[test]
    fn test_serde_roundtrip() {
        let status = DeadlineStatus::Overdue {
            duration: Duration::hours(4),
        };
        let serialized = serde_json::to_string(&status).expect("serialization failed");
        let deserialized: DeadlineStatus =
            serde_json::from_str(&serialized).expect("deserialization failed");
        assert_eq!(status, deserialized);
    }
}
