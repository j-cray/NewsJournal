//! Article entity and stage models.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::color::{assign_color_for_slug, Color};
use crate::error::ModelError;
use crate::validation::{validate_headline, validate_hex_color, validate_slug, ValidationError};

/// The editorial stages an article progresses through on the Kanban board.
///
/// # Examples
///
/// ```
/// use newsjournal_core::ArticleStage;
///
/// let stage = ArticleStage::Pitching;
/// assert_eq!(stage.display_name(), "Pitching");
/// assert_eq!(stage.next_stage(), Some(ArticleStage::Researching));
/// assert!(stage.is_active());
/// ```
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum ArticleStage {
    /// Initial idea or proposed story pitch.
    #[default]
    Pitching,
    /// Background reporting, interviews, and document gathering.
    Researching,
    /// Drafting the article text.
    Writing,
    /// Copy-editing, fact-checking, and structural review.
    Editing,
    /// Final sign-off received, ready for publication layout.
    ReadyToPublish,
    /// Published live in print, web, or broadcast.
    Published,
}

impl ArticleStage {
    /// Returns all available article stages in standard workflow order.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Pitching,
            Self::Researching,
            Self::Writing,
            Self::Editing,
            Self::ReadyToPublish,
            Self::Published,
        ]
    }

    /// Returns the canonical machine-readable slug/identifier for the stage.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pitching => "pitching",
            Self::Researching => "researching",
            Self::Writing => "writing",
            Self::Editing => "editing",
            Self::ReadyToPublish => "ready_to_publish",
            Self::Published => "published",
        }
    }

    /// Returns the human-readable display title for the stage.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Pitching => "Pitching",
            Self::Researching => "Researching",
            Self::Writing => "Writing",
            Self::Editing => "Editing",
            Self::ReadyToPublish => "Ready to Publish",
            Self::Published => "Published",
        }
    }

    /// Returns `true` if this stage represents a completed/published article.
    #[must_use]
    pub const fn is_published(&self) -> bool {
        matches!(self, Self::Published)
    }

    /// Returns `true` if the article is still in active editorial production.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        !self.is_published()
    }

    /// Returns the subsequent workflow stage, or `None` if already at `Published`.
    #[must_use]
    pub const fn next_stage(&self) -> Option<Self> {
        match self {
            Self::Pitching => Some(Self::Researching),
            Self::Researching => Some(Self::Writing),
            Self::Writing => Some(Self::Editing),
            Self::Editing => Some(Self::ReadyToPublish),
            Self::ReadyToPublish => Some(Self::Published),
            Self::Published => None,
        }
    }

    /// Returns the preceding workflow stage, or `None` if already at `Pitching`.
    #[must_use]
    pub const fn prev_stage(&self) -> Option<Self> {
        match self {
            Self::Pitching => None,
            Self::Researching => Some(Self::Pitching),
            Self::Writing => Some(Self::Researching),
            Self::Editing => Some(Self::Writing),
            Self::ReadyToPublish => Some(Self::Editing),
            Self::Published => Some(Self::ReadyToPublish),
        }
    }
}

impl fmt::Display for ArticleStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl FromStr for ArticleStage {
    type Err = ModelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_lowercase().replace(['-', ' '], "_");
        match normalized.as_str() {
            "pitching" | "pitch" => Ok(Self::Pitching),
            "researching" | "research" => Ok(Self::Researching),
            "writing" | "write" => Ok(Self::Writing),
            "editing" | "edit" => Ok(Self::Editing),
            "ready_to_publish" | "readytopublish" | "ready" => Ok(Self::ReadyToPublish),
            "published" | "publish" => Ok(Self::Published),
            _ => Err(ModelError::InvalidArticleStage(s.to_string())),
        }
    }
}

/// Core domain entity representing an article or journalistic story.
///
/// # Examples
///
/// ```
/// use newsjournal_core::{Article, ArticleStage};
///
/// let article = Article::new("city-council-investigation", "City Council Approves Transit Expansion")
///     .with_stage(ArticleStage::Writing)
///     .with_color("#4A90E2");
///
/// assert_eq!(article.slug, "city-council-investigation");
/// assert_eq!(article.stage, ArticleStage::Writing);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Article {
    /// Unique identifier for the article.
    pub id: Uuid,
    /// URL and filesystem safe unique slug (e.g. `2026-city-budget-investigation`).
    pub slug: String,
    /// Headline or working title of the article.
    pub headline: String,
    /// Detailed description, angle, notes, or story summary.
    pub description: Option<String>,
    /// Current workflow stage on the Kanban board.
    pub stage: ArticleStage,
    /// Publication or submission deadline.
    pub deadline: Option<DateTime<Utc>>,
    /// Visual color accent code (e.g. Hex `#4A90E2`).
    pub color: Option<String>,
    /// Timestamp when the record was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the record was last modified.
    pub updated_at: DateTime<Utc>,
}

impl Article {
    /// Creates a new `Article` with default timestamps and stage `Pitching`.
    pub fn new(slug: impl Into<String>, headline: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            slug: slug.into(),
            headline: headline.into(),
            description: None,
            stage: ArticleStage::Pitching,
            deadline: None,
            color: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Initializes a fluent builder for constructing an `Article`.
    pub fn builder(slug: impl Into<String>, headline: impl Into<String>) -> ArticleBuilder {
        ArticleBuilder::new(slug, headline)
    }

    /// Sets the article description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the article workflow stage.
    pub fn with_stage(mut self, stage: ArticleStage) -> Self {
        self.stage = stage;
        self
    }

    /// Sets the publication deadline.
    pub fn with_deadline(mut self, deadline: DateTime<Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Sets the color accent hex string.
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Automatically sets the color accent based on the deterministic palette assignment for this article's slug.
    pub fn with_auto_color(mut self) -> Self {
        self.color = Some(assign_color_for_slug(&self.slug).to_hex());
        self
    }

    /// Deterministically resolves the article's color.
    ///
    /// If an explicit valid hex color is stored on this record, it is parsed and returned.
    /// Otherwise, a fallback deterministic color is generated from the article's slug.
    #[must_use]
    pub fn color_or_default(&self) -> Color {
        self.color
            .as_deref()
            .and_then(|c| Color::from_hex(c).ok())
            .unwrap_or_else(|| assign_color_for_slug(&self.slug))
    }

    /// Updates the color to the deterministic default for the slug and touches `updated_at`.
    pub fn assign_default_color(&mut self) {
        self.color = Some(assign_color_for_slug(&self.slug).to_hex());
        self.touch();
    }

    /// Updates the `updated_at` timestamp to the current UTC time.
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Updates the stage and touches the `updated_at` timestamp.
    pub fn set_stage(&mut self, stage: ArticleStage) {
        self.stage = stage;
        self.touch();
    }

    /// Validates all field invariants of the article.
    ///
    /// Checks:
    /// - Slug is non-empty, safe alphanumeric/dashes/underscores, and within length limits.
    /// - Headline is non-empty and within length limits.
    /// - Color code (if present) is a valid hex color.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_slug(&self.slug)?;
        validate_headline(&self.headline)?;
        if let Some(ref color) = self.color {
            validate_hex_color(color)?;
        }
        Ok(())
    }

    /// Evaluates if the article is currently overdue given a reference timestamp.
    /// Articles in the `Published` stage are never considered overdue.
    #[must_use]
    pub fn is_overdue(&self, current_time: DateTime<Utc>) -> bool {
        if self.stage.is_published() {
            return false;
        }
        match self.deadline {
            Some(deadline) => current_time > deadline,
            None => false,
        }
    }

    /// Returns the comprehensive evaluated deadline status for this article.
    #[must_use]
    pub fn deadline_status(&self, current_time: DateTime<Utc>) -> crate::deadline::DeadlineStatus {
        crate::deadline::evaluate_article_deadline(self, current_time)
    }

    /// Returns `true` if the article has an upcoming deadline within the warning window.
    #[must_use]
    pub fn is_due_soon(&self, current_time: DateTime<Utc>) -> bool {
        crate::deadline::is_article_due_soon(self, current_time, None)
    }
}

/// Fluent builder for creating an `Article`.
#[derive(Debug, Clone)]
pub struct ArticleBuilder {
    id: Option<Uuid>,
    slug: String,
    headline: String,
    description: Option<String>,
    stage: ArticleStage,
    deadline: Option<DateTime<Utc>>,
    color: Option<String>,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
}

impl ArticleBuilder {
    /// Creates a new builder with required fields.
    pub fn new(slug: impl Into<String>, headline: impl Into<String>) -> Self {
        Self {
            id: None,
            slug: slug.into(),
            headline: headline.into(),
            description: None,
            stage: ArticleStage::Pitching,
            deadline: None,
            color: None,
            created_at: None,
            updated_at: None,
        }
    }

    /// Overrides the generated UUID.
    pub fn id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the article stage.
    pub fn stage(mut self, stage: ArticleStage) -> Self {
        self.stage = stage;
        self
    }

    /// Sets the deadline.
    pub fn deadline(mut self, deadline: DateTime<Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Sets the color accent.
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Automatically sets the color accent using the deterministic palette generator.
    pub fn auto_color(mut self) -> Self {
        self.color = Some(assign_color_for_slug(&self.slug).to_hex());
        self
    }

    /// Overrides the created_at timestamp.
    pub fn created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = Some(created_at);
        self
    }

    /// Overrides the updated_at timestamp.
    pub fn updated_at(mut self, updated_at: DateTime<Utc>) -> Self {
        self.updated_at = Some(updated_at);
        self
    }

    /// Builds the `Article` entity without validating invariants.
    #[must_use]
    pub fn build(self) -> Article {
        let now = Utc::now();
        Article {
            id: self.id.unwrap_or_else(Uuid::new_v4),
            slug: self.slug,
            headline: self.headline,
            description: self.description,
            stage: self.stage,
            deadline: self.deadline,
            color: self.color,
            created_at: self.created_at.unwrap_or(now),
            updated_at: self.updated_at.unwrap_or(now),
        }
    }

    /// Builds and validates the `Article` entity.
    pub fn build_validated(self) -> Result<Article, ValidationError> {
        let article = self.build();
        article.validate()?;
        Ok(article)
    }
}
