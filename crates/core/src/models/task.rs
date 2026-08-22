//! Task entity and status models.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ModelError;
use crate::validation::{validate_task_title, ValidationError};

/// Workflow status of an article-linked task.
///
/// # Examples
///
/// ```
/// use newsjournal_core::TaskStatus;
///
/// let status = TaskStatus::ToDo;
/// assert_eq!(status.display_name(), "To-Do");
/// assert_eq!(status.next_status(), Some(TaskStatus::InProgress));
/// assert!(!status.is_complete());
/// ```
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Task is queued and pending action.
    #[default]
    ToDo,
    /// Task is currently actively being worked on.
    InProgress,
    /// Task has been completed.
    Complete,
}

impl TaskStatus {
    /// Returns all task statuses in workflow order.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::ToDo, Self::InProgress, Self::Complete]
    }

    /// Returns the canonical machine-readable slug for the status.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ToDo => "to_do",
            Self::InProgress => "in_progress",
            Self::Complete => "complete",
        }
    }

    /// Returns the human-readable display title for the status.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::ToDo => "To-Do",
            Self::InProgress => "In Progress",
            Self::Complete => "Complete",
        }
    }

    /// Returns `true` if the task is marked complete.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        matches!(self, Self::Complete)
    }

    /// Returns the subsequent workflow status, or `None` if already `Complete`.
    #[must_use]
    pub const fn next_status(&self) -> Option<Self> {
        match self {
            Self::ToDo => Some(Self::InProgress),
            Self::InProgress => Some(Self::Complete),
            Self::Complete => None,
        }
    }

    /// Returns the preceding workflow status, or `None` if already `ToDo`.
    #[must_use]
    pub const fn prev_status(&self) -> Option<Self> {
        match self {
            Self::ToDo => None,
            Self::InProgress => Some(Self::ToDo),
            Self::Complete => Some(Self::InProgress),
        }
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl FromStr for TaskStatus {
    type Err = ModelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_lowercase().replace(['-', ' '], "_");
        match normalized.as_str() {
            "todo" | "to_do" => Ok(Self::ToDo),
            "in_progress" | "inprogress" | "doing" => Ok(Self::InProgress),
            "complete" | "completed" | "done" => Ok(Self::Complete),
            _ => Err(ModelError::InvalidTaskStatus(s.to_string())),
        }
    }
}

/// Core domain entity representing a reporting task linked to a parent article.
///
/// # Examples
///
/// ```
/// use uuid::Uuid;
/// use newsjournal_core::{Task, TaskStatus};
///
/// let article_id = Uuid::new_v4();
/// let task = Task::new(article_id, "Interview City Comptroller")
///     .with_notes("Ask about revenue projections")
///     .with_status(TaskStatus::InProgress);
///
/// assert_eq!(task.article_id, article_id);
/// assert_eq!(task.title, "Interview City Comptroller");
/// assert_eq!(task.status, TaskStatus::InProgress);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier for the task.
    pub id: Uuid,
    /// ID of the parent article this task belongs to.
    pub article_id: Uuid,
    /// Task title or action item description.
    pub title: String,
    /// Detailed notes, interview questions, or reference links.
    pub notes: Option<String>,
    /// Optional due date for task completion.
    pub due_date: Option<DateTime<Utc>>,
    /// Current completion status.
    pub status: TaskStatus,
    /// Timestamp when the task was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the task was last modified.
    pub updated_at: DateTime<Utc>,
}

impl Task {
    /// Creates a new `Task` associated with a parent article with default `ToDo` status.
    pub fn new(article_id: Uuid, title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            article_id,
            title: title.into(),
            notes: None,
            due_date: None,
            status: TaskStatus::ToDo,
            created_at: now,
            updated_at: now,
        }
    }

    /// Initializes a fluent builder for constructing a `Task`.
    pub fn builder(article_id: Uuid, title: impl Into<String>) -> TaskBuilder {
        TaskBuilder::new(article_id, title)
    }

    /// Sets the task notes.
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Sets the task due date.
    pub fn with_due_date(mut self, due_date: DateTime<Utc>) -> Self {
        self.due_date = Some(due_date);
        self
    }

    /// Sets the task status.
    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.status = status;
        self
    }

    /// Updates the `updated_at` timestamp to the current UTC time.
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Sets the status and touches `updated_at`.
    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.touch();
    }

    /// Validates all field invariants of the task.
    ///
    /// Checks:
    /// - Title is non-empty and does not exceed maximum length.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_task_title(&self.title)?;
        Ok(())
    }

    /// Evaluates if the task is currently overdue relative to a reference timestamp.
    /// Completed tasks are never considered overdue.
    #[must_use]
    pub fn is_overdue(&self, current_time: DateTime<Utc>) -> bool {
        if self.status.is_complete() {
            return false;
        }
        match self.due_date {
            Some(due_date) => current_time > due_date,
            None => false,
        }
    }

    /// Returns the comprehensive evaluated deadline status for this task.
    #[must_use]
    pub fn deadline_status(&self, current_time: DateTime<Utc>) -> crate::deadline::DeadlineStatus {
        crate::deadline::evaluate_task_deadline(self, current_time)
    }

    /// Returns `true` if the task has an upcoming due date within the warning window.
    #[must_use]
    pub fn is_due_soon(&self, current_time: DateTime<Utc>) -> bool {
        crate::deadline::is_task_due_soon(self, current_time, None)
    }
}

/// Fluent builder for creating a `Task`.
#[derive(Debug, Clone)]
pub struct TaskBuilder {
    id: Option<Uuid>,
    article_id: Uuid,
    title: String,
    notes: Option<String>,
    due_date: Option<DateTime<Utc>>,
    status: TaskStatus,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
}

impl TaskBuilder {
    /// Creates a new builder with required parent `article_id` and `title`.
    pub fn new(article_id: Uuid, title: impl Into<String>) -> Self {
        Self {
            id: None,
            article_id,
            title: title.into(),
            notes: None,
            due_date: None,
            status: TaskStatus::ToDo,
            created_at: None,
            updated_at: None,
        }
    }

    /// Overrides the generated UUID.
    pub fn id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets task notes.
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Sets task due date.
    pub fn due_date(mut self, due_date: DateTime<Utc>) -> Self {
        self.due_date = Some(due_date);
        self
    }

    /// Sets task status.
    pub fn status(mut self, status: TaskStatus) -> Self {
        self.status = status;
        self
    }

    /// Overrides created_at timestamp.
    pub fn created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = Some(created_at);
        self
    }

    /// Overrides updated_at timestamp.
    pub fn updated_at(mut self, updated_at: DateTime<Utc>) -> Self {
        self.updated_at = Some(updated_at);
        self
    }

    /// Builds the `Task` entity without validating invariants.
    #[must_use]
    pub fn build(self) -> Task {
        let now = Utc::now();
        Task {
            id: self.id.unwrap_or_else(Uuid::new_v4),
            article_id: self.article_id,
            title: self.title,
            notes: self.notes,
            due_date: self.due_date,
            status: self.status,
            created_at: self.created_at.unwrap_or(now),
            updated_at: self.updated_at.unwrap_or(now),
        }
    }

    /// Builds and validates the `Task` entity.
    pub fn build_validated(self) -> Result<Task, ValidationError> {
        let task = self.build();
        task.validate()?;
        Ok(task)
    }
}
