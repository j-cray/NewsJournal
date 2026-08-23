//! Articles Kanban board view models and horizontal deck layout engine.

use newsjournal_core::models::ArticleStage;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::drag_drop::DropTarget;
use crate::state::AppState;

/// Default nominal column width in logical pixels.
pub const DEFAULT_COLUMN_WIDTH: f32 = 320.0;

/// Minimum allowed column width in logical pixels.
pub const MIN_COLUMN_WIDTH: f32 = 260.0;

/// Maximum allowed column width in logical pixels.
pub const MAX_COLUMN_WIDTH: f32 = 440.0;

/// Default gap between columns in logical pixels.
pub const DEFAULT_COLUMN_GAP: f32 = 16.0;

/// Default horizontal padding around the deck in logical pixels.
pub const DEFAULT_DECK_PADDING: f32 = 20.0;

/// Total number of production stage columns on the Articles Kanban board.
pub const NUM_ARTICLE_STAGES: usize = 6;

/// Canonical editorial and visual metadata for an article production stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StageMetadata {
    /// Associated `ArticleStage`.
    pub stage: ArticleStage,
    /// 0-based workflow sequence index (0..=5).
    pub index: usize,
    /// Human-readable title.
    pub title: &'static str,
    /// Canonical slug representation.
    pub slug: &'static str,
    /// Editorial description / guidance.
    pub description: &'static str,
    /// Emoji representation.
    pub icon_emoji: &'static str,
    /// Standard icon name.
    pub icon_name: &'static str,
    /// macOS SF Symbol identifier.
    pub sf_symbol: &'static str,
    /// Stage accent hex color.
    pub accent_hex: &'static str,
    /// Empty state guidance prompt.
    pub empty_state_prompt: &'static str,
}

/// Returns canonical metadata for a given article production stage.
#[must_use]
pub const fn stage_metadata(stage: ArticleStage) -> StageMetadata {
    match stage {
        ArticleStage::Pitching => StageMetadata {
            stage: ArticleStage::Pitching,
            index: 0,
            title: "Pitching",
            slug: "pitching",
            description: "Story ideas & proposed pitches",
            icon_emoji: "💡",
            icon_name: "lightbulb",
            sf_symbol: "lightbulb",
            accent_hex: "#F59E0B",
            empty_state_prompt:
                "No story pitches yet. Click \"+ New Article\" or drag here to begin.",
        },
        ArticleStage::Researching => StageMetadata {
            stage: ArticleStage::Researching,
            index: 1,
            title: "Researching",
            slug: "researching",
            description: "Background reporting & source gathering",
            icon_emoji: "🔍",
            icon_name: "search",
            sf_symbol: "magnifyingglass",
            accent_hex: "#06B6D4",
            empty_state_prompt:
                "No articles in research. Move pitches here to start investigations.",
        },
        ArticleStage::Writing => StageMetadata {
            stage: ArticleStage::Writing,
            index: 2,
            title: "Writing",
            slug: "writing",
            description: "Active drafting & storytelling",
            icon_emoji: "✍️",
            icon_name: "edit-3",
            sf_symbol: "pencil.and.outline",
            accent_hex: "#3B82F6",
            empty_state_prompt: "No drafts in progress. Drag researched stories here to draft.",
        },
        ArticleStage::Editing => StageMetadata {
            stage: ArticleStage::Editing,
            index: 3,
            title: "Editing",
            slug: "editing",
            description: "Copy-editing, fact-checking & review",
            icon_emoji: "📝",
            icon_name: "check-square",
            sf_symbol: "doc.text.magnifyingglass",
            accent_hex: "#8B5CF6",
            empty_state_prompt: "No articles awaiting edit. Drag completed drafts here for review.",
        },
        ArticleStage::ReadyToPublish => StageMetadata {
            stage: ArticleStage::ReadyToPublish,
            index: 4,
            title: "Ready to Publish",
            slug: "ready_to_publish",
            description: "Editorial sign-off & layout preparation",
            icon_emoji: "🚀",
            icon_name: "send",
            sf_symbol: "checkmark.seal",
            accent_hex: "#10B981",
            empty_state_prompt: "No stories queued for publication. Cleared edits appear here.",
        },
        ArticleStage::Published => StageMetadata {
            stage: ArticleStage::Published,
            index: 5,
            title: "Published",
            slug: "published",
            description: "Archived & live published stories",
            icon_emoji: "📰",
            icon_name: "archive",
            sf_symbol: "newspaper",
            accent_hex: "#64748B",
            empty_state_prompt: "No published stories yet. Published work is archived here.",
        },
    }
}

/// Layout constants, geometry, and horizontal scrolling state for the 6-column Kanban deck.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DeckLayoutConfig {
    /// Nominal column width in logical pixels.
    pub column_width: f32,
    /// Gap between adjacent columns in logical pixels.
    pub column_gap: f32,
    /// Horizontal padding on left and right deck edges in logical pixels.
    pub deck_padding: f32,
    /// Current horizontal scroll offset in logical pixels.
    pub scroll_offset_x: f32,
    /// Current viewport container width in logical pixels.
    pub viewport_width: f32,
    /// Minimum allowed column width.
    pub min_column_width: f32,
    /// Maximum allowed column width.
    pub max_column_width: f32,
}

impl Default for DeckLayoutConfig {
    fn default() -> Self {
        Self {
            column_width: DEFAULT_COLUMN_WIDTH,
            column_gap: DEFAULT_COLUMN_GAP,
            deck_padding: DEFAULT_DECK_PADDING,
            scroll_offset_x: 0.0,
            viewport_width: 1280.0,
            min_column_width: MIN_COLUMN_WIDTH,
            max_column_width: MAX_COLUMN_WIDTH,
        }
    }
}

impl DeckLayoutConfig {
    /// Creates a new `DeckLayoutConfig` with the given viewport width.
    #[must_use]
    pub fn new(viewport_width: f32) -> Self {
        Self {
            viewport_width: viewport_width.max(100.0),
            ..Self::default()
        }
    }

    /// Sets the horizontal scroll offset.
    #[must_use]
    pub fn with_scroll(mut self, scroll_offset_x: f32) -> Self {
        self.scroll_offset_x = self.clamped_scroll_offset(scroll_offset_x);
        self
    }

    /// Sets the column width, clamped to min/max bounds.
    #[must_use]
    pub fn with_column_width(mut self, width: f32) -> Self {
        self.column_width = width.clamp(self.min_column_width, self.max_column_width);
        self
    }

    /// Sets the viewport width and re-clamps scroll offset.
    pub fn set_viewport_width(&mut self, width: f32) {
        self.viewport_width = width.max(100.0);
        self.scroll_offset_x = self.clamped_scroll_offset(self.scroll_offset_x);
    }

    /// Sets the column width and re-clamps.
    pub fn set_column_width(&mut self, width: f32) {
        self.column_width = width.clamp(self.min_column_width, self.max_column_width);
        self.scroll_offset_x = self.clamped_scroll_offset(self.scroll_offset_x);
    }

    /// Computes the total content width required to display all 6 columns, gaps, and deck padding.
    #[must_use]
    pub fn total_content_width(&self) -> f32 {
        let cols_total = (NUM_ARTICLE_STAGES as f32) * self.column_width;
        let gaps_total = ((NUM_ARTICLE_STAGES - 1) as f32) * self.column_gap;
        cols_total + gaps_total + (2.0 * self.deck_padding)
    }

    /// Computes the maximum valid horizontal scroll offset.
    #[must_use]
    pub fn max_scroll_offset(&self) -> f32 {
        (self.total_content_width() - self.viewport_width).max(0.0)
    }

    /// Clamps an arbitrary scroll offset into the valid `[0.0, max_scroll_offset]` range.
    #[must_use]
    pub fn clamped_scroll_offset(&self, offset: f32) -> f32 {
        offset.clamp(0.0, self.max_scroll_offset())
    }

    /// Returns `true` if the deck can be scrolled to the left.
    #[must_use]
    pub fn can_scroll_left(&self) -> bool {
        self.scroll_offset_x > 0.5
    }

    /// Returns `true` if the deck can be scrolled to the right.
    #[must_use]
    pub fn can_scroll_right(&self) -> bool {
        self.scroll_offset_x < (self.max_scroll_offset() - 0.5)
    }

    /// Returns the scroll progress ratio from `0.0` (start) to `1.0` (end).
    #[must_use]
    pub fn scroll_progress(&self) -> f32 {
        let max = self.max_scroll_offset();
        if max <= 0.0 {
            0.0
        } else {
            (self.scroll_offset_x / max).clamp(0.0, 1.0)
        }
    }

    /// Computes the absolute X position of a column relative to the deck content root.
    #[must_use]
    pub fn column_x_position(&self, column_index: usize) -> f32 {
        let index = column_index.min(NUM_ARTICLE_STAGES - 1);
        self.deck_padding + (index as f32) * (self.column_width + self.column_gap)
    }

    /// Computes the screen/viewport X coordinate for a column (taking scroll offset into account).
    #[must_use]
    pub fn column_screen_x(&self, column_index: usize) -> f32 {
        self.column_x_position(column_index) - self.scroll_offset_x
    }

    /// Computes the screen X bounds `(left, right)` for a column.
    #[must_use]
    pub fn column_screen_bounds(&self, column_index: usize) -> (f32, f32) {
        let left = self.column_screen_x(column_index);
        (left, left + self.column_width)
    }

    /// Checks if a column is at least partially visible in the viewport.
    #[must_use]
    pub fn is_column_visible(&self, column_index: usize) -> bool {
        let (left, right) = self.column_screen_bounds(column_index);
        right > 0.0 && left < self.viewport_width
    }

    /// Checks if a column is fully visible without clipping in the viewport.
    #[must_use]
    pub fn is_column_fully_visible(&self, column_index: usize) -> bool {
        let (left, right) = self.column_screen_bounds(column_index);
        left >= 0.0 && right <= self.viewport_width
    }

    /// Returns the indices of all columns currently visible in the viewport.
    #[must_use]
    pub fn visible_column_indices(&self) -> Vec<usize> {
        (0..NUM_ARTICLE_STAGES)
            .filter(|&i| self.is_column_visible(i))
            .collect()
    }

    /// Calculates the optimal scroll offset to bring the specified column fully into view.
    #[must_use]
    pub fn scroll_offset_for_column(&self, column_index: usize) -> f32 {
        let col_x = self.column_x_position(column_index);
        let col_right = col_x + self.column_width;

        let screen_left = col_x - self.scroll_offset_x;
        let screen_right = col_right - self.scroll_offset_x;

        if screen_left < 0.0 {
            // Need to scroll left to reveal column
            self.clamped_scroll_offset(col_x - self.deck_padding)
        } else if screen_right > self.viewport_width {
            // Need to scroll right to reveal column
            self.clamped_scroll_offset(col_right + self.deck_padding - self.viewport_width)
        } else {
            // Already visible
            self.scroll_offset_x
        }
    }

    /// Calculates the optimal scroll offset to bring a specific stage column into view.
    #[must_use]
    pub fn scroll_offset_for_stage(&self, stage: ArticleStage) -> f32 {
        let meta = stage_metadata(stage);
        self.scroll_offset_for_column(meta.index)
    }

    /// Scrolls horizontally by the given delta (positive moves right, negative moves left).
    pub fn scroll_by(&mut self, delta_x: f32) {
        self.scroll_offset_x = self.clamped_scroll_offset(self.scroll_offset_x + delta_x);
    }

    /// Scrolls directly to the specified offset.
    pub fn scroll_to(&mut self, offset: f32) {
        self.scroll_offset_x = self.clamped_scroll_offset(offset);
    }

    /// Scrolls horizontally to bring a stage column into view.
    pub fn scroll_to_stage(&mut self, stage: ArticleStage) {
        self.scroll_offset_x = self.scroll_offset_for_stage(stage);
    }

    /// Scrolls horizontally to bring a column index into view.
    pub fn scroll_to_column(&mut self, column_index: usize) {
        self.scroll_offset_x = self.scroll_offset_for_column(column_index);
    }

    /// Scrolls one viewport page to the left.
    pub fn scroll_page_left(&mut self) {
        let step = (self.viewport_width - self.column_width).max(self.column_width);
        self.scroll_by(-step);
    }

    /// Scrolls one viewport page to the right.
    pub fn scroll_page_right(&mut self) {
        let step = (self.viewport_width - self.column_width).max(self.column_width);
        self.scroll_by(step);
    }
}

pub use crate::views::article_card::{
    calculate_contrast_color, format_contact_initials, format_deadline_badge,
    ArticleCardContactTagViewModel, ArticleCardOverdueStyleViewModel, ArticleCardViewModel,
    ArticleDeadlineBadgeViewModel, ArticleSlugBadgeViewModel, ArticleTaskCounterViewModel,
    ColorIndicatorBarViewModel, IndicatorPosition, DEFAULT_ACCENT_STRIP_WIDTH,
    DEFAULT_CARD_BORDER_WIDTH, DEFAULT_CARD_CORNER_RADIUS, DUE_SOON_AMBER_HEX,
    DUE_SOON_BG_TINT_HEX, MAX_DESCRIPTION_SNIPPET_LEN, MAX_HEADLINE_SNIPPET_LEN,
    OVERDUE_BG_TINT_HEX, OVERDUE_CARD_BORDER_WIDTH, OVERDUE_RED_HEX, SUCCESS_GREEN_HEX,
};

/// Formatted view model for one of the 6 production stage Kanban columns.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArticleColumnViewModel {
    /// Production stage.
    pub stage: ArticleStage,
    /// 0-based column index across the 6-stage deck (0..=5).
    pub index: usize,
    /// Column title.
    pub title: &'static str,
    /// Editorial description or guidance for this stage.
    pub description: &'static str,
    /// Emoji icon for the stage.
    pub icon_emoji: &'static str,
    /// Standard symbolic icon name.
    pub icon_name: &'static str,
    /// macOS SF Symbol identifier.
    pub sf_symbol: &'static str,
    /// Stage accent hex color for headers and accents.
    pub accent_hex: &'static str,
    /// List of cards in this column.
    pub cards: Vec<ArticleCardViewModel>,
    /// Total card count in this column.
    pub card_count: usize,
    /// Number of overdue cards in this column.
    pub overdue_count: usize,
    /// Number of cards due soon in this column.
    pub due_soon_count: usize,
    /// Whether this column is actively hovered during drag-and-drop.
    pub is_hovered: bool,
    /// Whether this column has zero cards.
    pub is_empty: bool,
    /// Stage-specific empty state guidance message.
    pub empty_state_prompt: &'static str,
}

/// Formatted view model for the entire Articles Kanban Deck (6 columns, horizontal scrolling).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArticlesKanbanDeckViewModel {
    /// The 6 production stage columns in sequential workflow order.
    pub columns: Vec<ArticleColumnViewModel>,
    /// Total number of articles across all 6 columns.
    pub total_article_count: usize,
    /// Total number of overdue articles across the entire deck.
    pub total_overdue_count: usize,
    /// Total number of articles due soon across the entire deck.
    pub total_due_soon_count: usize,
    /// Whether any search query or stage filter is actively applied.
    pub is_filtered: bool,
    /// Active search filter query string.
    pub search_query: String,
    /// Active stage filter if filtering to a single stage.
    pub selected_stage_filter: Option<ArticleStage>,
    /// Horizontal scrolling and geometry configuration for the deck.
    pub layout: DeckLayoutConfig,
}

impl ArticlesKanbanDeckViewModel {
    /// Looks up a column by its production stage.
    #[must_use]
    pub fn column(&self, stage: ArticleStage) -> Option<&ArticleColumnViewModel> {
        self.columns.iter().find(|col| col.stage == stage)
    }

    /// Looks up a mutable column by its production stage.
    pub fn column_mut(&mut self, stage: ArticleStage) -> Option<&mut ArticleColumnViewModel> {
        self.columns.iter_mut().find(|col| col.stage == stage)
    }

    /// Looks up a column by its 0-based index.
    #[must_use]
    pub fn column_by_index(&self, index: usize) -> Option<&ArticleColumnViewModel> {
        self.columns.get(index)
    }

    /// Looks up a mutable column by its 0-based index.
    pub fn column_by_index_mut(&mut self, index: usize) -> Option<&mut ArticleColumnViewModel> {
        self.columns.get_mut(index)
    }

    /// Finds a card by its unique UUID across all columns in the deck.
    #[must_use]
    pub fn card(&self, id: Uuid) -> Option<(&ArticleColumnViewModel, &ArticleCardViewModel)> {
        for col in &self.columns {
            if let Some(card) = col.cards.iter().find(|c| c.id == id) {
                return Some((col, card));
            }
        }
        None
    }

    /// Returns `true` if all columns in the deck have zero articles.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.total_article_count == 0
    }

    /// Returns `true` if any article in the deck is overdue.
    #[must_use]
    pub fn has_overdue(&self) -> bool {
        self.total_overdue_count > 0
    }

    /// Returns the number of articles in the specified stage column.
    #[must_use]
    pub fn count_for_stage(&self, stage: ArticleStage) -> usize {
        self.column(stage).map_or(0, |col| col.card_count)
    }

    /// Returns the number of overdue articles in the specified stage column.
    #[must_use]
    pub fn overdue_count_for_stage(&self, stage: ArticleStage) -> usize {
        self.column(stage).map_or(0, |col| col.overdue_count)
    }

    /// Returns the number of due-soon articles in the specified stage column.
    #[must_use]
    pub fn due_soon_count_for_stage(&self, stage: ArticleStage) -> usize {
        self.column(stage).map_or(0, |col| col.due_soon_count)
    }

    /// Returns the titles of all 6 stages in sequential order.
    #[must_use]
    pub const fn stage_names() -> [&'static str; NUM_ARTICLE_STAGES] {
        [
            "Pitching",
            "Researching",
            "Writing",
            "Editing",
            "Ready to Publish",
            "Published",
        ]
    }

    /// Returns all 6 stages in sequential workflow order.
    #[must_use]
    pub const fn stages() -> &'static [ArticleStage] {
        ArticleStage::all()
    }
}

/// Constructs the 6-column Articles Kanban view model list from application state.
#[must_use]
pub fn build_articles_kanban_view(state: &AppState) -> Vec<ArticleColumnViewModel> {
    let deck = build_articles_kanban_deck(state);
    deck.columns
}

/// Constructs the complete 6-column Articles Kanban Deck view model from application state with default layout.
#[must_use]
pub fn build_articles_kanban_deck(state: &AppState) -> ArticlesKanbanDeckViewModel {
    build_articles_kanban_deck_with_layout(state, DeckLayoutConfig::default())
}

/// Constructs the complete 6-column Articles Kanban Deck view model from application state with custom layout geometry.
#[must_use]
pub fn build_articles_kanban_deck_with_layout(
    state: &AppState,
    layout: DeckLayoutConfig,
) -> ArticlesKanbanDeckViewModel {
    let stages = ArticleStage::all();

    let hover_stage = match state.drag.hover_target {
        Some(DropTarget::ArticleColumn(stage)) => Some(stage),
        _ => None,
    };

    let filtered_articles = state.filtered_articles();

    let mut total_article_count = 0;
    let mut total_overdue_count = 0;
    let mut total_due_soon_count = 0;

    let columns: Vec<ArticleColumnViewModel> = stages
        .iter()
        .map(|&stage| {
            let meta = stage_metadata(stage);

            let cards: Vec<ArticleCardViewModel> = filtered_articles
                .iter()
                .filter(|a| a.stage == stage)
                .map(|article| {
                    let card = ArticleCardViewModel::build(article, state);

                    if card.is_overdue {
                        total_overdue_count += 1;
                    }
                    if card.is_due_soon {
                        total_due_soon_count += 1;
                    }

                    card
                })
                .collect();

            let card_count = cards.len();
            let overdue_count = cards.iter().filter(|c| c.is_overdue).count();
            let due_soon_count = cards.iter().filter(|c| c.is_due_soon).count();
            total_article_count += card_count;

            ArticleColumnViewModel {
                stage,
                index: meta.index,
                title: meta.title,
                description: meta.description,
                icon_emoji: meta.icon_emoji,
                icon_name: meta.icon_name,
                sf_symbol: meta.sf_symbol,
                accent_hex: meta.accent_hex,
                card_count,
                overdue_count,
                due_soon_count,
                cards,
                is_hovered: hover_stage == Some(stage),
                is_empty: card_count == 0,
                empty_state_prompt: meta.empty_state_prompt,
            }
        })
        .collect();

    let is_filtered = state.filters.is_active();
    let search_query = state.filters.search_query.clone();
    let selected_stage_filter = state.filters.selected_stage;

    ArticlesKanbanDeckViewModel {
        columns,
        total_article_count,
        total_overdue_count,
        total_due_soon_count,
        is_filtered,
        search_query,
        selected_stage_filter,
        layout,
    }
}
