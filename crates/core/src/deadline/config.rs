//! Deadline evaluation configuration and urgency thresholds.

use chrono::Duration;
use serde::{Deserialize, Serialize};

/// Default threshold for warning that a deadline is approaching (24 hours).
pub const DEFAULT_DUE_SOON_HOURS: i64 = 24;

/// Default threshold for critical deadline alert (1 hour).
pub const DEFAULT_CRITICAL_HOURS: i64 = 1;

/// Configuration defining thresholds for deadline evaluation and urgency classification.
///
/// # Examples
///
/// ```
/// use chrono::Duration;
/// use newsjournal_core::deadline::DeadlineConfig;
///
/// let config = DeadlineConfig::default();
/// assert_eq!(config.critical_threshold, Duration::hours(1));
/// assert_eq!(config.due_soon_threshold, Duration::hours(24));
///
/// let custom = DeadlineConfig::builder()
///     .critical_threshold(Duration::minutes(30))
///     .due_soon_threshold(Duration::hours(48))
///     .build();
///
/// assert_eq!(custom.critical_threshold, Duration::minutes(30));
/// assert_eq!(custom.due_soon_threshold, Duration::hours(48));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeadlineConfig {
    /// Time remaining before deadline below which status becomes `Critical`.
    pub critical_threshold: Duration,
    /// Time remaining before deadline below which status becomes `DueSoon`.
    pub due_soon_threshold: Duration,
}

impl Default for DeadlineConfig {
    fn default() -> Self {
        Self {
            critical_threshold: Duration::hours(DEFAULT_CRITICAL_HOURS),
            due_soon_threshold: Duration::hours(DEFAULT_DUE_SOON_HOURS),
        }
    }
}

impl DeadlineConfig {
    /// Creates a new `DeadlineConfig` with explicit thresholds.
    #[must_use]
    pub const fn new(critical_threshold: Duration, due_soon_threshold: Duration) -> Self {
        Self {
            critical_threshold,
            due_soon_threshold,
        }
    }

    /// Initializes a builder for constructing a custom `DeadlineConfig`.
    #[must_use]
    pub fn builder() -> DeadlineConfigBuilder {
        DeadlineConfigBuilder::default()
    }

    /// Sets the critical threshold.
    #[must_use]
    pub const fn with_critical_threshold(mut self, threshold: Duration) -> Self {
        self.critical_threshold = threshold;
        self
    }

    /// Sets the due soon threshold.
    #[must_use]
    pub const fn with_due_soon_threshold(mut self, threshold: Duration) -> Self {
        self.due_soon_threshold = threshold;
        self
    }

    /// Convenience helper to create a config with specified hours for critical and due soon.
    #[must_use]
    pub fn from_hours(critical_hours: i64, due_soon_hours: i64) -> Self {
        Self {
            critical_threshold: Duration::hours(critical_hours),
            due_soon_threshold: Duration::hours(due_soon_hours),
        }
    }
}

/// Builder for [`DeadlineConfig`].
#[derive(Debug, Clone, Copy, Default)]
pub struct DeadlineConfigBuilder {
    critical_threshold: Option<Duration>,
    due_soon_threshold: Option<Duration>,
}

impl DeadlineConfigBuilder {
    /// Sets the critical urgency duration threshold.
    #[must_use]
    pub const fn critical_threshold(mut self, threshold: Duration) -> Self {
        self.critical_threshold = Some(threshold);
        self
    }

    /// Sets the due soon urgency duration threshold.
    #[must_use]
    pub const fn due_soon_threshold(mut self, threshold: Duration) -> Self {
        self.due_soon_threshold = Some(threshold);
        self
    }

    /// Builds the [`DeadlineConfig`], falling back to default values for unset fields.
    #[must_use]
    pub fn build(self) -> DeadlineConfig {
        let default = DeadlineConfig::default();
        DeadlineConfig {
            critical_threshold: self
                .critical_threshold
                .unwrap_or(default.critical_threshold),
            due_soon_threshold: self
                .due_soon_threshold
                .unwrap_or(default.due_soon_threshold),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DeadlineConfig::default();
        assert_eq!(config.critical_threshold, Duration::hours(1));
        assert_eq!(config.due_soon_threshold, Duration::hours(24));
    }

    #[test]
    fn test_custom_config_constructor() {
        let config = DeadlineConfig::new(Duration::minutes(15), Duration::hours(12));
        assert_eq!(config.critical_threshold, Duration::minutes(15));
        assert_eq!(config.due_soon_threshold, Duration::hours(12));
    }

    #[test]
    fn test_builder_pattern() {
        let config = DeadlineConfig::builder()
            .critical_threshold(Duration::hours(2))
            .due_soon_threshold(Duration::hours(36))
            .build();

        assert_eq!(config.critical_threshold, Duration::hours(2));
        assert_eq!(config.due_soon_threshold, Duration::hours(36));
    }

    #[test]
    fn test_builder_defaults() {
        let partial = DeadlineConfig::builder()
            .critical_threshold(Duration::minutes(45))
            .build();

        assert_eq!(partial.critical_threshold, Duration::minutes(45));
        assert_eq!(partial.due_soon_threshold, Duration::hours(24));
    }

    #[test]
    fn test_with_helpers() {
        let config = DeadlineConfig::default()
            .with_critical_threshold(Duration::minutes(20))
            .with_due_soon_threshold(Duration::hours(8));

        assert_eq!(config.critical_threshold, Duration::minutes(20));
        assert_eq!(config.due_soon_threshold, Duration::hours(8));
    }

    #[test]
    fn test_from_hours() {
        let config = DeadlineConfig::from_hours(3, 72);
        assert_eq!(config.critical_threshold, Duration::hours(3));
        assert_eq!(config.due_soon_threshold, Duration::hours(72));
    }
}
