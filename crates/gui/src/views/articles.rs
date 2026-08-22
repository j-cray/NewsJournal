//! Articles Kanban board view models.

use chrono::{DateTime, Utc};
use newsjournal_core::color::assign_color_for_slug;
use newsjournal_core::deadline::{evaluate_article_deadline, DeadlineStatus};
use newsjournal_core::models::ArticleStage;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::drag_drop::{DragItem, DropTarget};
use crate::state::AppState;

/// Formatted view model for a single Article card in the Kanban deck.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleCardViewModel {
    /// Unique article ID.
    pub id: Uuid,
    /// Slug identifier.
    pub slug: String,
    /// Article headline.
    pub headline: String,
    /// Active stage.
    pub stage: ArticleStage,
    /// Assigned accent hex color.
    pub color_hex: String,
    /// Optional target deadline.
    pub deadline: Option<DateTime<Utc>>,
    /// Calculated deadline status.
    pub deadline_status: DeadlineStatus,
    /// Overdue flag for prominent red alert styling.
    pub is_overdue: bool,
    /// Due soon flag for amber alert styling.
    pub is_due_soon: bool,
    /// Number of completed tasks.
    pub task_completed: usize,
    /// Total number of tasks.
    pub task_total: usize,
    /// Number of contacts tagged on this story.
    pub tagged_contact_count: usize,
    /// True if this card is currently being dragged.
    pub is_dragging: bool,
}

/// Formatted view model for one of the 6 production stage Kanban columns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleColumnViewModel {
    /// Production stage.
    pub stage: ArticleStage,
    /// Column title.
    pub title: &'static str,
    /// List of cards in this column.
    pub cards: Vec<ArticleCardViewModel>,
    /// Whether this column is actively hovered during drag-and-drop.
    pub is_hovered: bool,
}

/// Constructs the 6-column Articles Kanban view model from application state.
#[must_use]
pub fn build_articles_kanban_view(state: &AppState) -> Vec<ArticleColumnViewModel> {
    let stages = [
        ArticleStage::Pitching,
        ArticleStage::Researching,
        ArticleStage::Writing,
        ArticleStage::Editing,
        ArticleStage::ReadyToPublish,
        ArticleStage::Published,
    ];

    let active_drag_id = match state.drag.active_item {
        Some(DragItem::ArticleCard { id, .. }) => Some(id),
        _ => None,
    };

    let hover_stage = match state.drag.hover_target {
        Some(DropTarget::ArticleColumn(stage)) => Some(stage),
        _ => None,
    };

    let filtered_articles = state.filtered_articles();

    stages
        .iter()
        .map(|&stage| {
            let cards: Vec<ArticleCardViewModel> = filtered_articles
                .iter()
                .filter(|a| a.stage == stage)
                .map(|article| {
                    let deadline_status = evaluate_article_deadline(article, state.last_tick);
                    let (task_completed, task_total) = state.task_completion_stats(article.id);
                    let tagged_contact_count = state
                        .article_contacts
                        .get(&article.id)
                        .map(Vec::len)
                        .unwrap_or(0);
                    let color_hex = article
                        .color
                        .clone()
                        .unwrap_or_else(|| assign_color_for_slug(&article.slug).to_hex());

                    ArticleCardViewModel {
                        id: article.id,
                        slug: article.slug.clone(),
                        headline: article.headline.clone(),
                        stage: article.stage,
                        color_hex,
                        deadline: article.deadline,
                        is_overdue: deadline_status.is_overdue(),
                        is_due_soon: deadline_status.is_due_soon(),
                        deadline_status,
                        task_completed,
                        task_total,
                        tagged_contact_count,
                        is_dragging: active_drag_id == Some(article.id),
                    }
                })
                .collect();

            let title = match stage {
                ArticleStage::Pitching => "Pitching",
                ArticleStage::Researching => "Researching",
                ArticleStage::Writing => "Writing",
                ArticleStage::Editing => "Editing",
                ArticleStage::ReadyToPublish => "Ready to Publish",
                ArticleStage::Published => "Published",
            };

            ArticleColumnViewModel {
                stage,
                title,
                cards,
                is_hovered: hover_stage == Some(stage),
            }
        })
        .collect()
}
