//! In-app toast notification queue and models.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Visual variant for an in-app notification toast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ToastKind {
    /// Informational toast (blue/neutral accent).
    #[default]
    Info,
    /// Success confirmation toast (green accent).
    Success,
    /// Warning alert toast (amber accent).
    Warning,
    /// Error notification toast (red accent).
    Error,
}

/// In-app notification toast record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToastMessage {
    /// Unique identifier for toast dismissal.
    pub id: Uuid,
    /// Variant kind determining color styling.
    pub kind: ToastKind,
    /// Toast title or short headline.
    pub title: String,
    /// Detailed description or body.
    pub body: String,
    /// Timestamp when the toast was created.
    pub created_at: DateTime<Utc>,
    /// Time-to-live in seconds before auto-dismissal. Default is 5 seconds.
    pub ttl_seconds: u32,
}

impl ToastMessage {
    /// Creates a new toast notification with standard 5-second lifetime.
    #[must_use]
    pub fn new(kind: ToastKind, title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            title: title.into(),
            body: body.into(),
            created_at: Utc::now(),
            ttl_seconds: 5,
        }
    }

    /// Convenience constructor for an info toast.
    #[must_use]
    pub fn info(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self::new(ToastKind::Info, title, body)
    }

    /// Convenience constructor for a success toast.
    #[must_use]
    pub fn success(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self::new(ToastKind::Success, title, body)
    }

    /// Convenience constructor for a warning toast.
    #[must_use]
    pub fn warning(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self::new(ToastKind::Warning, title, body)
    }

    /// Convenience constructor for an error toast with longer 8-second lifetime.
    #[must_use]
    pub fn error(title: impl Into<String>, body: impl Into<String>) -> Self {
        let mut toast = Self::new(ToastKind::Error, title, body);
        toast.ttl_seconds = 8;
        toast
    }

    /// Sets a custom TTL in seconds.
    #[must_use]
    pub fn with_ttl(mut self, seconds: u32) -> Self {
        self.ttl_seconds = seconds;
        self
    }

    /// Checks whether this toast has expired relative to the given timestamp.
    #[must_use]
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        if self.ttl_seconds == 0 {
            return false;
        }
        let age = now.signed_duration_since(self.created_at);
        age >= Duration::seconds(i64::from(self.ttl_seconds))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_constructors_and_expiration() {
        let t_info = ToastMessage::info("Notice", "Operation started");
        assert_eq!(t_info.kind, ToastKind::Info);
        assert_eq!(t_info.ttl_seconds, 5);

        let t_err = ToastMessage::error("Failed", "Database lock timeout");
        assert_eq!(t_err.kind, ToastKind::Error);
        assert_eq!(t_err.ttl_seconds, 8);

        let now = t_info.created_at;
        assert!(!t_info.is_expired(now + Duration::seconds(4)));
        assert!(t_info.is_expired(now + Duration::seconds(5)));
        assert!(t_info.is_expired(now + Duration::seconds(10)));
    }
}
