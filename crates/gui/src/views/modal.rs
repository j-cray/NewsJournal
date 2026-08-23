//! In-app modal and slide-over drawer view models, container geometry, glass styling, and presentation builders.

use newsjournal_core::models::ArticleStage;
use serde::Serialize;

use crate::state::modal::ModalState;
use crate::state::AppState;
use crate::theme::ResolvedTheme;

/// Layout placement and presentation style for in-app glass overlays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Default)]
pub enum ModalPlacement {
    /// Centered floating glass overlay dialog (optimal for confirmations and compact creation).
    CenteredModal,
    /// Layered slide-over drawer pinned to the right screen edge (optimal for deep editing).
    #[default]
    SlideOverRight,
}

impl ModalPlacement {
    /// Returns the recommended default placement for a given modal state.
    #[must_use]
    pub const fn default_for(state: &ModalState) -> Self {
        match state {
            ModalState::None => Self::CenteredModal,
            ModalState::ConfirmDeleteArticle { .. }
            | ModalState::ConfirmDeleteTask { .. }
            | ModalState::ConfirmDeleteContact { .. } => Self::CenteredModal,
            ModalState::ArticleForm(_)
            | ModalState::TaskForm(_)
            | ModalState::ContactForm(_)
            | ModalState::SettingsDrawer(_) => Self::SlideOverRight,
        }
    }

    /// Returns `true` if this placement is a slide-over drawer.
    #[must_use]
    pub const fn is_drawer(&self) -> bool {
        matches!(self, Self::SlideOverRight)
    }

    /// Returns `true` if this placement is a centered dialog.
    #[must_use]
    pub const fn is_centered(&self) -> bool {
        matches!(self, Self::CenteredModal)
    }
}

/// Sizing and geometry configuration for glass modal / drawer containers.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ModalGeometry {
    /// Logical width in pixels / points.
    pub width: f32,
    /// Minimum clamped width.
    pub min_width: f32,
    /// Maximum clamped width.
    pub max_width: f32,
    /// Logical height in pixels / points (or viewport percentage if flexible).
    pub height: Option<f32>,
    /// Maximum height for centered modals (clamped to screen boundary).
    pub max_height: Option<f32>,
    /// Container corner radius.
    pub corner_radius: f32,
    /// Content inner padding.
    pub padding: f32,
    /// Elevation layer / z-index.
    pub z_index: u32,
}

impl ModalGeometry {
    /// Generates standard geometry for a slide-over right drawer.
    #[must_use]
    pub const fn slide_over_drawer() -> Self {
        Self {
            width: 580.0,
            min_width: 420.0,
            max_width: 760.0,
            height: None, // Full vertical height
            max_height: None,
            corner_radius: 16.0,
            padding: 24.0,
            z_index: 100,
        }
    }

    /// Generates standard geometry for a centered glass modal dialog.
    #[must_use]
    pub const fn centered_modal() -> Self {
        Self {
            width: 520.0,
            min_width: 380.0,
            max_width: 680.0,
            height: None,
            max_height: Some(720.0),
            corner_radius: 16.0,
            padding: 24.0,
            z_index: 100,
        }
    }

    /// Generates compact geometry for delete confirmation dialogs.
    #[must_use]
    pub const fn confirmation_dialog() -> Self {
        Self {
            width: 440.0,
            min_width: 340.0,
            max_width: 520.0,
            height: None,
            max_height: Some(360.0),
            corner_radius: 14.0,
            padding: 20.0,
            z_index: 110,
        }
    }

    /// Resolves geometry based on modal placement and state.
    #[must_use]
    pub const fn resolve(placement: ModalPlacement, state: &ModalState) -> Self {
        match state {
            ModalState::ConfirmDeleteArticle { .. }
            | ModalState::ConfirmDeleteTask { .. }
            | ModalState::ConfirmDeleteContact { .. } => Self::confirmation_dialog(),
            _ => match placement {
                ModalPlacement::CenteredModal => Self::centered_modal(),
                ModalPlacement::SlideOverRight => Self::slide_over_drawer(),
            },
        }
    }
}

/// Dimming overlay and background blur configuration for modal backdrops.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ModalBackdropViewModel {
    /// Whether the backdrop overlay is visible and active.
    pub is_visible: bool,
    /// Background dimming color RGBA `(r, g, b, a)`.
    pub color_rgba: (u8, u8, u8, f32),
    /// Background blur radius in logical points.
    pub blur_radius: f32,
    /// Whether clicking on the backdrop closes the active modal.
    pub dismiss_on_click: bool,
}

impl ModalBackdropViewModel {
    /// Builds backdrop view model from application theme and modal state.
    #[must_use]
    pub fn new(is_dark: bool, is_open: bool, is_destructive_prompt: bool) -> Self {
        let color_rgba = if is_dark {
            (0, 0, 0, 0.65)
        } else {
            (0, 0, 0, 0.40)
        };

        let blur_radius = if is_open { 28.0 } else { 0.0 };

        Self {
            is_visible: is_open,
            color_rgba,
            blur_radius,
            dismiss_on_click: !is_destructive_prompt,
        }
    }
}

/// Header bar presentation model for modal containers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModalHeaderViewModel {
    /// Main header title.
    pub title: &'static str,
    /// Contextual subtitle or entity hint.
    pub subtitle: &'static str,
    /// Category or action icon emoji (e.g. "📰", "✅", "👥", "⚙️", "🗑️").
    pub icon_emoji: &'static str,
    /// Desktop symbolic icon name (e.g. "document-edit", "user", "trash").
    pub icon_name: &'static str,
    /// Optional status / stage badge text (e.g. "Writing", "Overdue").
    pub badge_text: Option<String>,
    /// Optional status badge hex color.
    pub badge_color_hex: Option<String>,
    /// Whether the close button is visible.
    pub show_close_button: bool,
    /// Close button action tooltip (e.g. "Close (Esc)").
    pub close_tooltip: &'static str,
    /// Keyboard shortcut to close (e.g. "Esc").
    pub close_shortcut: &'static str,
}

impl ModalHeaderViewModel {
    /// Constructs header view model from the active modal state.
    #[must_use]
    pub fn from_modal_state(state: &ModalState) -> Self {
        match state {
            ModalState::None => Self {
                title: "",
                subtitle: "",
                icon_emoji: "",
                icon_name: "",
                badge_text: None,
                badge_color_hex: None,
                show_close_button: false,
                close_tooltip: "Close",
                close_shortcut: "Esc",
            },

            ModalState::ArticleForm(draft) => {
                let is_edit = draft.id.is_some();
                let title = if is_edit {
                    "Edit Article"
                } else {
                    "New Article"
                };
                let subtitle = if is_edit {
                    "Update story details, deadlines, and contacts"
                } else {
                    "Create a new reporting pitch or story"
                };

                let badge_text = Some(match draft.stage {
                    ArticleStage::Pitching => "Pitching".to_string(),
                    ArticleStage::Researching => "Researching".to_string(),
                    ArticleStage::Writing => "Writing".to_string(),
                    ArticleStage::Editing => "Editing".to_string(),
                    ArticleStage::ReadyToPublish => "Ready to Publish".to_string(),
                    ArticleStage::Published => "Published".to_string(),
                });

                let badge_color_hex = if draft.color_hex.is_empty() {
                    None
                } else {
                    Some(draft.color_hex.clone())
                };

                Self {
                    title,
                    subtitle,
                    icon_emoji: "📰",
                    icon_name: "document-edit",
                    badge_text,
                    badge_color_hex,
                    show_close_button: true,
                    close_tooltip: "Close (Esc)",
                    close_shortcut: "Esc",
                }
            }

            ModalState::TaskForm(draft) => {
                let is_edit = draft.id.is_some();
                Self {
                    title: if is_edit { "Edit Task" } else { "New Task" },
                    subtitle: "Task checklist item linked to reporting",
                    icon_emoji: "✅",
                    icon_name: "checkbox",
                    badge_text: None,
                    badge_color_hex: None,
                    show_close_button: true,
                    close_tooltip: "Close (Esc)",
                    close_shortcut: "Esc",
                }
            }

            ModalState::ContactForm(draft) => {
                let is_edit = draft.id.is_some();
                Self {
                    title: if is_edit {
                        "Edit Contact"
                    } else {
                        "New Contact"
                    },
                    subtitle: "Source, reporter, or editorial contact details",
                    icon_emoji: "👥",
                    icon_name: "user",
                    badge_text: None,
                    badge_color_hex: None,
                    show_close_button: true,
                    close_tooltip: "Close (Esc)",
                    close_shortcut: "Esc",
                }
            }

            ModalState::SettingsDrawer(_) => Self {
                title: "Application Settings",
                subtitle: "Appearance, theme, database status and storage paths",
                icon_emoji: "⚙️",
                icon_name: "settings",
                badge_text: None,
                badge_color_hex: None,
                show_close_button: true,
                close_tooltip: "Close (Esc)",
                close_shortcut: "Esc",
            },

            ModalState::ConfirmDeleteArticle { slug, .. } => Self {
                title: "Delete Article?",
                subtitle: "This action will permanently delete the article and unlink tasks",
                icon_emoji: "⚠️",
                icon_name: "alert-triangle",
                badge_text: Some(format!("slug: {slug}")),
                badge_color_hex: Some("#FF4D4F".to_string()),
                show_close_button: true,
                close_tooltip: "Cancel (Esc)",
                close_shortcut: "Esc",
            },

            ModalState::ConfirmDeleteTask { .. } => Self {
                title: "Delete Task?",
                subtitle: "This action will permanently delete the task from the story",
                icon_emoji: "⚠️",
                icon_name: "alert-triangle",
                badge_text: None,
                badge_color_hex: Some("#FF4D4F".to_string()),
                show_close_button: true,
                close_tooltip: "Cancel (Esc)",
                close_shortcut: "Esc",
            },

            ModalState::ConfirmDeleteContact {
                linked_article_count,
                ..
            } => Self {
                title: "Delete Contact?",
                subtitle: "This action will permanently delete the contact record",
                icon_emoji: "⚠️",
                icon_name: "alert-triangle",
                badge_text: if *linked_article_count > 0 {
                    Some(format!("{linked_article_count} linked articles"))
                } else {
                    None
                },
                badge_color_hex: Some("#FF4D4F".to_string()),
                show_close_button: true,
                close_tooltip: "Cancel (Esc)",
                close_shortcut: "Esc",
            },
        }
    }
}

/// Action footer presentation model for modal containers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModalFooterViewModel {
    /// Primary confirmation/submission button label.
    pub primary_label: &'static str,
    /// Primary button shortcut text (e.g. "Ctrl+S" or "⌘S" or "Enter").
    pub primary_shortcut: String,
    /// Whether the primary button is a destructive action (e.g. red Delete button).
    pub primary_is_destructive: bool,
    /// Whether the primary button is disabled (e.g. due to invalid fields).
    pub primary_is_disabled: bool,
    /// Secondary dismiss button label (e.g. "Cancel").
    pub secondary_label: &'static str,
    /// Secondary button shortcut text (e.g. "Esc").
    pub secondary_shortcut: &'static str,
    /// Number of active validation errors in the current draft.
    pub validation_error_count: usize,
    /// Optional summary validation hint.
    pub validation_hint: Option<String>,
}

impl ModalFooterViewModel {
    /// Constructs footer view model from the active modal state and platform context.
    #[must_use]
    pub fn from_modal_state(state: &ModalState, is_macos: bool) -> Self {
        let save_shortcut = if is_macos { "⌘S" } else { "Ctrl+S" };

        match state {
            ModalState::None => Self {
                primary_label: "",
                primary_shortcut: String::new(),
                primary_is_destructive: false,
                primary_is_disabled: true,
                secondary_label: "",
                secondary_shortcut: "Esc",
                validation_error_count: 0,
                validation_hint: None,
            },

            ModalState::ArticleForm(draft) => {
                let err_count = draft.validation_errors.len();
                let is_edit = draft.id.is_some();
                let hint = if err_count > 0 {
                    Some(format!("{err_count} required field(s) need attention"))
                } else {
                    None
                };

                Self {
                    primary_label: if is_edit {
                        "Save Changes"
                    } else {
                        "Create Article"
                    },
                    primary_shortcut: save_shortcut.to_string(),
                    primary_is_destructive: false,
                    primary_is_disabled: err_count > 0,
                    secondary_label: "Cancel",
                    secondary_shortcut: "Esc",
                    validation_error_count: err_count,
                    validation_hint: hint,
                }
            }

            ModalState::TaskForm(draft) => {
                let err_count = draft.validation_errors.len();
                let is_edit = draft.id.is_some();
                let hint = if err_count > 0 {
                    Some(format!("{err_count} field(s) need attention"))
                } else {
                    None
                };

                Self {
                    primary_label: if is_edit { "Save Task" } else { "Create Task" },
                    primary_shortcut: save_shortcut.to_string(),
                    primary_is_destructive: false,
                    primary_is_disabled: err_count > 0,
                    secondary_label: "Cancel",
                    secondary_shortcut: "Esc",
                    validation_error_count: err_count,
                    validation_hint: hint,
                }
            }

            ModalState::ContactForm(draft) => {
                let err_count = draft.validation_errors.len();
                let is_edit = draft.id.is_some();
                let hint = if err_count > 0 {
                    Some(format!("{err_count} field(s) need attention"))
                } else {
                    None
                };

                Self {
                    primary_label: if is_edit {
                        "Save Contact"
                    } else {
                        "Create Contact"
                    },
                    primary_shortcut: save_shortcut.to_string(),
                    primary_is_destructive: false,
                    primary_is_disabled: err_count > 0,
                    secondary_label: "Cancel",
                    secondary_shortcut: "Esc",
                    validation_error_count: err_count,
                    validation_hint: hint,
                }
            }

            ModalState::SettingsDrawer(_) => Self {
                primary_label: "Done",
                primary_shortcut: "Esc".to_string(),
                primary_is_destructive: false,
                primary_is_disabled: false,
                secondary_label: "Close",
                secondary_shortcut: "Esc",
                validation_error_count: 0,
                validation_hint: None,
            },

            ModalState::ConfirmDeleteArticle { .. }
            | ModalState::ConfirmDeleteTask { .. }
            | ModalState::ConfirmDeleteContact { .. } => Self {
                primary_label: "Delete Permanently",
                primary_shortcut: "Enter".to_string(),
                primary_is_destructive: true,
                primary_is_disabled: false,
                secondary_label: "Cancel",
                secondary_shortcut: "Esc",
                validation_error_count: 0,
                validation_hint: None,
            },
        }
    }
}

/// Resolved glass material styling for the modal overlay container.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ModalGlassMaterial {
    /// Background blur radius in pixels/points.
    pub blur_radius: f32,
    /// RGBA surface background color.
    pub surface_rgba: (u8, u8, u8, f32),
    /// RGBA border stroke highlight.
    pub border_rgba: (u8, u8, u8, f32),
    /// RGBA specular highlight reflection (for macOS).
    pub specular_highlight_rgba: (u8, u8, u8, f32),
    /// Border stroke width.
    pub border_width: f32,
    /// Corner radius.
    pub corner_radius: f32,
    /// Drop shadow `(offset_x, offset_y, blur_radius, alpha)`.
    pub shadow: Option<(f32, f32, f32, f32)>,
}

impl ModalGlassMaterial {
    /// Constructs default COSMIC frosted glass material for modal overlay.
    #[must_use]
    pub fn cosmic(is_dark: bool) -> Self {
        Self {
            blur_radius: 32.0,
            surface_rgba: if is_dark {
                (24, 28, 36, 0.94)
            } else {
                (255, 255, 255, 0.96)
            },
            border_rgba: if is_dark {
                (255, 255, 255, 0.20)
            } else {
                (0, 0, 0, 0.12)
            },
            specular_highlight_rgba: (255, 255, 255, 0.0),
            border_width: 1.5,
            corner_radius: 16.0,
            shadow: Some((0.0, 12.0, 32.0, if is_dark { 0.55 } else { 0.20 })),
        }
    }

    /// Constructs default macOS liquid glass material for modal sheet / drawer.
    #[must_use]
    pub fn macos(is_dark: bool) -> Self {
        Self {
            blur_radius: 36.0,
            surface_rgba: if is_dark {
                (26, 30, 40, 0.92)
            } else {
                (255, 255, 255, 0.94)
            },
            border_rgba: if is_dark {
                (255, 255, 255, 0.18)
            } else {
                (0, 0, 0, 0.10)
            },
            specular_highlight_rgba: (255, 255, 255, if is_dark { 0.25 } else { 0.60 }),
            border_width: 1.0,
            corner_radius: 14.0,
            shadow: Some((0.0, 14.0, 36.0, if is_dark { 0.60 } else { 0.22 })),
        }
    }
}

/// Unified presentation descriptor for an active in-app glass modal or slide-over drawer.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModalContainerViewModel {
    /// Whether any modal overlay is active.
    pub is_open: bool,
    /// Placement layout (centered vs slide-over right drawer).
    pub placement: ModalPlacement,
    /// Container geometry parameters.
    pub geometry: ModalGeometry,
    /// Backdrop dimming and blur configuration.
    pub backdrop: ModalBackdropViewModel,
    /// Header presentation view model.
    pub header: ModalHeaderViewModel,
    /// Footer action presentation view model.
    pub footer: ModalFooterViewModel,
    /// Glass material styling properties.
    pub glass: ModalGlassMaterial,
    /// Active modal state snapshot.
    pub state: ModalState,
}

impl ModalContainerViewModel {
    /// Returns `true` if the container is currently open.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.is_open
    }

    /// Returns the main header title.
    #[must_use]
    pub const fn title(&self) -> &'static str {
        self.header.title
    }

    /// Returns `true` if the modal represents a destructive confirmation.
    #[must_use]
    pub const fn is_destructive_prompt(&self) -> bool {
        self.footer.primary_is_destructive
    }
}

/// Formatted legacy view descriptor for active in-app modal (kept for backward compatibility).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModalViewModel {
    /// Whether any modal overlay is active.
    pub is_open: bool,
    /// Modal header title.
    pub title: &'static str,
    /// Active modal state snapshot.
    pub state: ModalState,
}

/// Constructs the active legacy modal view descriptor from application state.
#[must_use]
pub fn build_modal_view(state: &AppState) -> ModalViewModel {
    ModalViewModel {
        is_open: state.modal.is_open(),
        title: state.modal.title(),
        state: state.modal.clone(),
    }
}

/// Constructs the complete `ModalContainerViewModel` using default platform detection and placement.
#[must_use]
pub fn build_modal_container_view(state: &AppState) -> ModalContainerViewModel {
    let is_macos = cfg!(target_os = "macos");
    let placement = ModalPlacement::default_for(&state.modal);
    build_modal_container_view_with_layout(state, placement, is_macos)
}

/// Constructs the complete `ModalContainerViewModel` with explicit placement and platform context.
#[must_use]
pub fn build_modal_container_view_with_layout(
    state: &AppState,
    placement: ModalPlacement,
    is_macos: bool,
) -> ModalContainerViewModel {
    let is_dark = state.theme().resolved == ResolvedTheme::Dark;
    let is_open = state.modal.is_open();

    let is_destructive = matches!(
        state.modal,
        ModalState::ConfirmDeleteArticle { .. }
            | ModalState::ConfirmDeleteTask { .. }
            | ModalState::ConfirmDeleteContact { .. }
    );

    let geometry = ModalGeometry::resolve(placement, &state.modal);
    let backdrop = ModalBackdropViewModel::new(is_dark, is_open, is_destructive);
    let header = ModalHeaderViewModel::from_modal_state(&state.modal);
    let footer = ModalFooterViewModel::from_modal_state(&state.modal, is_macos);

    let glass = if is_macos {
        ModalGlassMaterial::macos(is_dark)
    } else {
        ModalGlassMaterial::cosmic(is_dark)
    };

    ModalContainerViewModel {
        is_open,
        placement,
        geometry,
        backdrop,
        header,
        footer,
        glass,
        state: state.modal.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::modal::{ArticleDraft, ContactDraft, TaskDraft};
    use uuid::Uuid;

    #[test]
    fn test_modal_placement_defaults() {
        assert_eq!(
            ModalPlacement::default_for(&ModalState::None),
            ModalPlacement::CenteredModal
        );
        assert_eq!(
            ModalPlacement::default_for(&ModalState::ArticleForm(ArticleDraft::new())),
            ModalPlacement::SlideOverRight
        );
        assert_eq!(
            ModalPlacement::default_for(&ModalState::TaskForm(TaskDraft::default())),
            ModalPlacement::SlideOverRight
        );
        assert_eq!(
            ModalPlacement::default_for(&ModalState::ContactForm(ContactDraft::new())),
            ModalPlacement::SlideOverRight
        );
        assert_eq!(
            ModalPlacement::default_for(&ModalState::ConfirmDeleteArticle {
                id: Uuid::new_v4(),
                slug: "test-slug".to_string(),
                headline: "Test".to_string(),
            }),
            ModalPlacement::CenteredModal
        );

        assert!(ModalPlacement::SlideOverRight.is_drawer());
        assert!(!ModalPlacement::SlideOverRight.is_centered());
        assert!(ModalPlacement::CenteredModal.is_centered());
        assert!(!ModalPlacement::CenteredModal.is_drawer());
    }

    #[test]
    fn test_modal_geometry_resolution() {
        let drawer_geo = ModalGeometry::resolve(ModalPlacement::SlideOverRight, &ModalState::None);
        assert_eq!(drawer_geo.width, 580.0);
        assert_eq!(drawer_geo.min_width, 420.0);
        assert_eq!(drawer_geo.max_width, 760.0);
        assert!(drawer_geo.height.is_none());

        let centered_geo = ModalGeometry::resolve(ModalPlacement::CenteredModal, &ModalState::None);
        assert_eq!(centered_geo.width, 520.0);
        assert_eq!(centered_geo.max_height, Some(720.0));

        let delete_geo = ModalGeometry::resolve(
            ModalPlacement::SlideOverRight,
            &ModalState::ConfirmDeleteArticle {
                id: Uuid::new_v4(),
                slug: "test".to_string(),
                headline: "Test".to_string(),
            },
        );
        assert_eq!(delete_geo.width, 440.0);
        assert_eq!(delete_geo.max_height, Some(360.0));
    }

    #[test]
    fn test_modal_backdrop_view_model() {
        let dark_open = ModalBackdropViewModel::new(true, true, false);
        assert!(dark_open.is_visible);
        assert_eq!(dark_open.color_rgba, (0, 0, 0, 0.65));
        assert_eq!(dark_open.blur_radius, 28.0);
        assert!(dark_open.dismiss_on_click);

        let dark_destructive = ModalBackdropViewModel::new(true, true, true);
        assert!(!dark_destructive.dismiss_on_click);

        let light_open = ModalBackdropViewModel::new(false, true, false);
        assert_eq!(light_open.color_rgba, (0, 0, 0, 0.40));

        let closed = ModalBackdropViewModel::new(true, false, false);
        assert!(!closed.is_visible);
        assert_eq!(closed.blur_radius, 0.0);
    }

    #[test]
    fn test_modal_header_view_model_states() {
        // Article new
        let mut article_draft = ArticleDraft::new();
        article_draft.stage = ArticleStage::Writing;
        article_draft.color_hex = "#1A85FF".to_string();
        let header_article_new =
            ModalHeaderViewModel::from_modal_state(&ModalState::ArticleForm(article_draft));
        assert_eq!(header_article_new.title, "New Article");
        assert_eq!(header_article_new.icon_emoji, "📰");
        assert_eq!(header_article_new.badge_text, Some("Writing".to_string()));
        assert_eq!(
            header_article_new.badge_color_hex,
            Some("#1A85FF".to_string())
        );
        assert!(header_article_new.show_close_button);

        // Delete article
        let header_del =
            ModalHeaderViewModel::from_modal_state(&ModalState::ConfirmDeleteArticle {
                id: Uuid::new_v4(),
                slug: "city-hall".to_string(),
                headline: "City Hall Report".to_string(),
            });
        assert_eq!(header_del.title, "Delete Article?");
        assert_eq!(header_del.icon_emoji, "⚠️");
        assert_eq!(header_del.badge_text, Some("slug: city-hall".to_string()));
    }

    #[test]
    fn test_modal_footer_view_model_validation_and_shortcuts() {
        // Invalid article draft (empty)
        let mut invalid_draft = ArticleDraft::new();
        invalid_draft.validate();
        let footer_invalid_linux =
            ModalFooterViewModel::from_modal_state(&ModalState::ArticleForm(invalid_draft), false);
        assert_eq!(footer_invalid_linux.primary_label, "Create Article");
        assert_eq!(footer_invalid_linux.primary_shortcut, "Ctrl+S");
        assert!(footer_invalid_linux.primary_is_disabled);
        assert!(footer_invalid_linux.validation_error_count > 0);
        assert!(footer_invalid_linux.validation_hint.is_some());

        // Valid article draft on macOS
        let mut valid_draft = ArticleDraft::new();
        valid_draft.slug = "probe-widens".to_string();
        valid_draft.headline = "Probe Widens Into Port Operations".to_string();
        valid_draft.validate();
        let footer_valid_mac =
            ModalFooterViewModel::from_modal_state(&ModalState::ArticleForm(valid_draft), true);
        assert_eq!(footer_valid_mac.primary_shortcut, "⌘S");
        assert!(!footer_valid_mac.primary_is_disabled);
        assert_eq!(footer_valid_mac.validation_error_count, 0);

        // Destructive delete confirmation
        let footer_del = ModalFooterViewModel::from_modal_state(
            &ModalState::ConfirmDeleteTask {
                id: Uuid::new_v4(),
                title: "Call witness".to_string(),
            },
            false,
        );
        assert_eq!(footer_del.primary_label, "Delete Permanently");
        assert!(footer_del.primary_is_destructive);
        assert!(!footer_del.primary_is_disabled);
    }

    #[test]
    fn test_build_modal_container_view_lifecycle() {
        let mut state = AppState::in_memory().expect("in-memory state");
        assert!(!state.modal.is_open());

        let closed_container = build_modal_container_view(&state);
        assert!(!closed_container.is_open());
        assert!(!closed_container.backdrop.is_visible);

        // Open article modal
        let draft = ArticleDraft::new();
        state.modal = ModalState::ArticleForm(draft);
        let open_container = build_modal_container_view(&state);
        assert!(open_container.is_open());
        assert!(open_container.backdrop.is_visible);
        assert_eq!(open_container.placement, ModalPlacement::SlideOverRight);
        assert_eq!(open_container.header.title, "New Article");
        assert_eq!(open_container.geometry.width, 580.0);
    }
}
