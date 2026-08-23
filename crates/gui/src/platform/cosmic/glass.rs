//! COSMIC Frosted Glass styling tokens, materials, and container definitions.

use serde::{Deserialize, Serialize};

use crate::theme::{AppTheme, ColorTokens, ResolvedTheme};

/// Standard COSMIC blur radius in logical pixels.
pub const COSMIC_DEFAULT_BLUR_RADIUS: f32 = 24.0;

/// Deep backdrop blur radius used when modals/drawers are open.
pub const COSMIC_DEEP_BLUR_RADIUS: f32 = 32.0;

/// Subtle blur radius used for cards and embedded widgets.
pub const COSMIC_CARD_BLUR_RADIUS: f32 = 16.0;

/// UI container semantic classification for COSMIC frosted glass rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CosmicContainerClass {
    /// Top window header bar.
    HeaderBar,
    /// Left vertical navigation sidebar.
    Sidebar,
    /// Kanban stage column container.
    KanbanColumn,
    /// Article card item.
    ArticleCard,
    /// Task card item.
    TaskCard,
    /// High-focus in-app glass modal or slide-over drawer.
    ModalDrawer,
    /// Floating toast notification badge.
    Toast,
    /// Filter and search bar container.
    FilterBar,
}

/// Resolved frosted glass material properties for a specific container.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CosmicGlassStyle {
    /// Background blur radius in pixels.
    pub blur_radius: f32,
    /// RGBA surface background color `(r, g, b, a)`.
    pub surface_rgba: (u8, u8, u8, f32),
    /// RGBA highlight border color `(r, g, b, a)`.
    pub border_rgba: (u8, u8, u8, f32),
    /// Border stroke thickness in logical pixels.
    pub border_width: f32,
    /// Corner radius in logical pixels.
    pub corner_radius: f32,
    /// Frost vibrancy multiplier (1.0 = neutral, >1.0 = saturated frost).
    pub vibrancy: f32,
    /// Optional drop-shadow parameters `(offset_x, offset_y, blur_radius, alpha)`.
    pub shadow: Option<(f32, f32, f32, f32)>,
}

/// COSMIC frosted glass materials generator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CosmicFrostedGlass {
    /// Global frost opacity factor.
    pub opacity_factor: f32,
    /// Global blur intensity factor.
    pub blur_factor: f32,
}

impl Default for CosmicFrostedGlass {
    fn default() -> Self {
        Self {
            opacity_factor: 1.0,
            blur_factor: 1.0,
        }
    }
}

impl CosmicFrostedGlass {
    /// Creates a new `CosmicFrostedGlass` instance.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            opacity_factor: 1.0,
            blur_factor: 1.0,
        }
    }

    /// Resolves the concrete frosted glass styling for a given container class and active theme.
    #[must_use]
    pub fn style_for(&self, class: CosmicContainerClass, theme: &AppTheme) -> CosmicGlassStyle {
        let is_dark = theme.resolved == ResolvedTheme::Dark;
        let colors = &theme.colors;

        match class {
            CosmicContainerClass::HeaderBar => CosmicGlassStyle {
                blur_radius: COSMIC_DEFAULT_BLUR_RADIUS * self.blur_factor,
                surface_rgba: if is_dark {
                    (22, 26, 34, 0.82 * self.opacity_factor)
                } else {
                    (250, 252, 255, 0.85 * self.opacity_factor)
                },
                border_rgba: if is_dark {
                    (255, 255, 255, 0.12)
                } else {
                    (0, 0, 0, 0.08)
                },
                border_width: 1.0,
                corner_radius: 0.0,
                vibrancy: 1.15,
                shadow: Some((0.0, 2.0, 8.0, if is_dark { 0.35 } else { 0.08 })),
            },

            CosmicContainerClass::Sidebar => CosmicGlassStyle {
                blur_radius: 28.0 * self.blur_factor,
                surface_rgba: if is_dark {
                    (18, 20, 26, 0.88 * self.opacity_factor)
                } else {
                    (245, 247, 250, 0.90 * self.opacity_factor)
                },
                border_rgba: if is_dark {
                    (255, 255, 255, 0.10)
                } else {
                    (0, 0, 0, 0.06)
                },
                border_width: 1.0,
                corner_radius: 0.0,
                vibrancy: 1.20,
                shadow: Some((2.0, 0.0, 10.0, if is_dark { 0.30 } else { 0.06 })),
            },

            CosmicContainerClass::KanbanColumn => CosmicGlassStyle {
                blur_radius: 18.0 * self.blur_factor,
                surface_rgba: if is_dark {
                    (26, 30, 38, 0.70 * self.opacity_factor)
                } else {
                    (240, 243, 248, 0.75 * self.opacity_factor)
                },
                border_rgba: if is_dark {
                    (255, 255, 255, 0.08)
                } else {
                    (0, 0, 0, 0.06)
                },
                border_width: 1.0,
                corner_radius: 12.0,
                vibrancy: 1.10,
                shadow: None,
            },

            CosmicContainerClass::ArticleCard | CosmicContainerClass::TaskCard => {
                CosmicGlassStyle {
                    blur_radius: COSMIC_CARD_BLUR_RADIUS * self.blur_factor,
                    surface_rgba: if is_dark {
                        (34, 38, 48, 0.85 * self.opacity_factor)
                    } else {
                        (255, 255, 255, 0.90 * self.opacity_factor)
                    },
                    border_rgba: if is_dark {
                        (255, 255, 255, 0.14)
                    } else {
                        (0, 0, 0, 0.08)
                    },
                    border_width: 1.0,
                    corner_radius: 10.0,
                    vibrancy: 1.12,
                    shadow: Some((0.0, 4.0, 12.0, if is_dark { 0.35 } else { 0.10 })),
                }
            }

            CosmicContainerClass::ModalDrawer => CosmicGlassStyle {
                blur_radius: COSMIC_DEEP_BLUR_RADIUS * self.blur_factor,
                surface_rgba: if is_dark {
                    (24, 28, 36, 0.94 * self.opacity_factor)
                } else {
                    (255, 255, 255, 0.96 * self.opacity_factor)
                },
                border_rgba: if is_dark {
                    (255, 255, 255, 0.20)
                } else {
                    (0, 0, 0, 0.12)
                },
                border_width: 1.5,
                corner_radius: 16.0,
                vibrancy: 1.30,
                shadow: Some((0.0, 12.0, 32.0, if is_dark { 0.55 } else { 0.20 })),
            },

            CosmicContainerClass::Toast => CosmicGlassStyle {
                blur_radius: 20.0 * self.blur_factor,
                surface_rgba: if is_dark {
                    (30, 34, 44, 0.92 * self.opacity_factor)
                } else {
                    (255, 255, 255, 0.95 * self.opacity_factor)
                },
                border_rgba: (colors.accent.0, colors.accent.1, colors.accent.2, 0.40),
                border_width: 1.0,
                corner_radius: 12.0,
                vibrancy: 1.25,
                shadow: Some((0.0, 6.0, 16.0, if is_dark { 0.40 } else { 0.15 })),
            },

            CosmicContainerClass::FilterBar => CosmicGlassStyle {
                blur_radius: 20.0 * self.blur_factor,
                surface_rgba: if is_dark {
                    (28, 32, 40, 0.80 * self.opacity_factor)
                } else {
                    (248, 250, 253, 0.85 * self.opacity_factor)
                },
                border_rgba: if is_dark {
                    (255, 255, 255, 0.10)
                } else {
                    (0, 0, 0, 0.07)
                },
                border_width: 1.0,
                corner_radius: 8.0,
                vibrancy: 1.10,
                shadow: None,
            },
        }
    }

    /// Generates special high-visibility overdue card style with a prominent red glass border highlight.
    #[must_use]
    pub fn overdue_card_style(
        &self,
        base_style: &CosmicGlassStyle,
        colors: &ColorTokens,
    ) -> CosmicGlassStyle {
        let mut style = *base_style;
        style.border_rgba = (
            colors.overdue_alert.0,
            colors.overdue_alert.1,
            colors.overdue_alert.2,
            0.85,
        );
        style.border_width = 2.0;
        style.shadow = Some((
            0.0, 4.0, 16.0, 0.45, // Enhanced red shadow glow
        ));
        style
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use newsjournal_core::models::ThemeMode;

    #[test]
    fn test_cosmic_glass_styles_dark_and_light() {
        let glass = CosmicFrostedGlass::new();
        let dark_theme = AppTheme::new(ThemeMode::Dark, true);
        let light_theme = AppTheme::new(ThemeMode::Light, false);

        let dark_header = glass.style_for(CosmicContainerClass::HeaderBar, &dark_theme);
        assert_eq!(dark_header.blur_radius, 24.0);
        assert!(dark_header.surface_rgba.3 > 0.8);
        assert_eq!(dark_header.border_width, 1.0);

        let light_modal = glass.style_for(CosmicContainerClass::ModalDrawer, &light_theme);
        assert_eq!(light_modal.blur_radius, 32.0);
        assert_eq!(light_modal.corner_radius, 16.0);
        assert_eq!(light_modal.border_width, 1.5);
    }

    #[test]
    fn test_overdue_card_style_generation() {
        let glass = CosmicFrostedGlass::new();
        let theme = AppTheme::new(ThemeMode::Dark, true);
        let card_style = glass.style_for(CosmicContainerClass::ArticleCard, &theme);

        let overdue_style = glass.overdue_card_style(&card_style, &theme.colors);
        assert_eq!(overdue_style.border_width, 2.0);
        assert_eq!(overdue_style.border_rgba.0, theme.colors.overdue_alert.0);
        assert_eq!(overdue_style.border_rgba.1, theme.colors.overdue_alert.1);
        assert_eq!(overdue_style.border_rgba.2, theme.colors.overdue_alert.2);
        assert_eq!(overdue_style.border_rgba.3, 0.85);
    }
}
