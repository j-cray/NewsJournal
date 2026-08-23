//! macOS Liquid Glass styling tokens, materials, and container definitions.
//!
//! Liquid Glass on macOS delivers deep translucency, subtle specular top-edge highlights,
//! delicate borders, and authentic Apple-like vibrancy effects.

use serde::{Deserialize, Serialize};

use crate::platform::macos::vibrancy::MacosVibrancyMaterial;
use crate::theme::{AppTheme, ColorTokens, ResolvedTheme};

/// Standard macOS liquid glass blur radius in logical points.
pub const MACOS_DEFAULT_BLUR_RADIUS: f32 = 28.0;

/// Deep backdrop blur radius used for modal sheets and slide-over drawers.
pub const MACOS_SHEET_BLUR_RADIUS: f32 = 36.0;

/// Subtle blur radius used for floating cards and kanban decks.
pub const MACOS_CARD_BLUR_RADIUS: f32 = 18.0;

/// Floating popover and toast blur radius.
pub const MACOS_POPOVER_BLUR_RADIUS: f32 = 20.0;

/// UI container semantic classification for macOS liquid glass rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MacosContainerClass {
    /// Unified top window toolbar / titlebar.
    Toolbar,
    /// Left vertical navigation sidebar.
    Sidebar,
    /// Kanban stage column translucent container.
    KanbanColumn,
    /// Article card item with specular light reflection.
    ArticleCard,
    /// Task card item with parent article accent.
    TaskCard,
    /// High-focus in-app glass modal sheet or slide-over drawer.
    ModalDrawer,
    /// Floating toast notification badge / popover.
    Toast,
    /// Filter and search bar container.
    FilterBar,
}

/// Resolved liquid glass material properties for a specific macOS container.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MacosGlassStyle {
    /// Background blur radius in logical points.
    pub blur_radius: f32,
    /// RGBA surface background color `(r, g, b, a)`.
    pub surface_rgba: (u8, u8, u8, f32),
    /// RGBA highlight border color `(r, g, b, a)`.
    pub border_rgba: (u8, u8, u8, f32),
    /// RGBA specular highlight reflection (top-edge light refraction) `(r, g, b, a)`.
    pub specular_highlight_rgba: (u8, u8, u8, f32),
    /// Border stroke thickness in logical pixels.
    pub border_width: f32,
    /// Corner radius in logical points.
    pub corner_radius: f32,
    /// Vibrancy multiplier factor.
    pub vibrancy: f32,
    /// Optional drop-shadow parameters `(offset_x, offset_y, blur_radius, alpha)`.
    pub shadow: Option<(f32, f32, f32, f32)>,
    /// Associated AppKit vibrancy material.
    pub vibrancy_material: MacosVibrancyMaterial,
}

/// macOS Liquid Glass materials generator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MacosLiquidGlass {
    /// Global liquid opacity factor (1.0 = standard).
    pub opacity_factor: f32,
    /// Global blur intensity factor (1.0 = standard).
    pub blur_factor: f32,
    /// Specular reflection intensity factor (1.0 = standard).
    pub specular_intensity: f32,
}

impl Default for MacosLiquidGlass {
    fn default() -> Self {
        Self {
            opacity_factor: 1.0,
            blur_factor: 1.0,
            specular_intensity: 1.0,
        }
    }
}

impl MacosLiquidGlass {
    /// Creates a new `MacosLiquidGlass` instance with default settings.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            opacity_factor: 1.0,
            blur_factor: 1.0,
            specular_intensity: 1.0,
        }
    }

    /// Resolves concrete liquid glass styling for a given container class and active theme.
    #[must_use]
    pub fn style_for(&self, class: MacosContainerClass, theme: &AppTheme) -> MacosGlassStyle {
        let is_dark = theme.resolved == ResolvedTheme::Dark;
        let colors = &theme.colors;

        match class {
            MacosContainerClass::Toolbar => MacosGlassStyle {
                blur_radius: 24.0 * self.blur_factor,
                surface_rgba: if is_dark {
                    (28, 32, 40, 0.72 * self.opacity_factor)
                } else {
                    (245, 247, 252, 0.78 * self.opacity_factor)
                },
                border_rgba: if is_dark {
                    (255, 255, 255, 0.10)
                } else {
                    (0, 0, 0, 0.08)
                },
                specular_highlight_rgba: (
                    255,
                    255,
                    255,
                    if is_dark {
                        0.15 * self.specular_intensity
                    } else {
                        0.40 * self.specular_intensity
                    },
                ),
                border_width: 0.5,
                corner_radius: 0.0,
                vibrancy: 1.25,
                shadow: Some((0.0, 1.0, 4.0, if is_dark { 0.25 } else { 0.05 })),
                vibrancy_material: MacosVibrancyMaterial::HeaderView,
            },

            MacosContainerClass::Sidebar => MacosGlassStyle {
                blur_radius: MACOS_DEFAULT_BLUR_RADIUS * self.blur_factor,
                surface_rgba: if is_dark {
                    (20, 24, 30, 0.80 * self.opacity_factor)
                } else {
                    (240, 243, 248, 0.85 * self.opacity_factor)
                },
                border_rgba: if is_dark {
                    (255, 255, 255, 0.08)
                } else {
                    (0, 0, 0, 0.06)
                },
                specular_highlight_rgba: (
                    255,
                    255,
                    255,
                    if is_dark {
                        0.10 * self.specular_intensity
                    } else {
                        0.30 * self.specular_intensity
                    },
                ),
                border_width: 0.5,
                corner_radius: 0.0,
                vibrancy: 1.30,
                shadow: Some((1.0, 0.0, 8.0, if is_dark { 0.20 } else { 0.04 })),
                vibrancy_material: MacosVibrancyMaterial::Sidebar,
            },

            MacosContainerClass::KanbanColumn => MacosGlassStyle {
                blur_radius: 20.0 * self.blur_factor,
                surface_rgba: if is_dark {
                    (30, 35, 45, 0.60 * self.opacity_factor)
                } else {
                    (238, 242, 248, 0.65 * self.opacity_factor)
                },
                border_rgba: if is_dark {
                    (255, 255, 255, 0.08)
                } else {
                    (0, 0, 0, 0.05)
                },
                specular_highlight_rgba: (
                    255,
                    255,
                    255,
                    if is_dark {
                        0.12 * self.specular_intensity
                    } else {
                        0.25 * self.specular_intensity
                    },
                ),
                border_width: 1.0,
                corner_radius: 12.0,
                vibrancy: 1.15,
                shadow: None,
                vibrancy_material: MacosVibrancyMaterial::ContentBackground,
            },

            MacosContainerClass::ArticleCard | MacosContainerClass::TaskCard => MacosGlassStyle {
                blur_radius: MACOS_CARD_BLUR_RADIUS * self.blur_factor,
                surface_rgba: if is_dark {
                    (38, 44, 56, 0.82 * self.opacity_factor)
                } else {
                    (255, 255, 255, 0.88 * self.opacity_factor)
                },
                border_rgba: if is_dark {
                    (255, 255, 255, 0.12)
                } else {
                    (0, 0, 0, 0.08)
                },
                specular_highlight_rgba: (
                    255,
                    255,
                    255,
                    if is_dark {
                        0.20 * self.specular_intensity
                    } else {
                        0.50 * self.specular_intensity
                    },
                ),
                border_width: 1.0,
                corner_radius: 10.0,
                vibrancy: 1.20,
                shadow: Some((0.0, 3.0, 10.0, if is_dark { 0.30 } else { 0.08 })),
                vibrancy_material: MacosVibrancyMaterial::Selection,
            },

            MacosContainerClass::ModalDrawer => MacosGlassStyle {
                blur_radius: MACOS_SHEET_BLUR_RADIUS * self.blur_factor,
                surface_rgba: if is_dark {
                    (26, 30, 40, 0.92 * self.opacity_factor)
                } else {
                    (255, 255, 255, 0.94 * self.opacity_factor)
                },
                border_rgba: if is_dark {
                    (255, 255, 255, 0.18)
                } else {
                    (0, 0, 0, 0.10)
                },
                specular_highlight_rgba: (
                    255,
                    255,
                    255,
                    if is_dark {
                        0.25 * self.specular_intensity
                    } else {
                        0.60 * self.specular_intensity
                    },
                ),
                border_width: 1.0,
                corner_radius: 14.0,
                vibrancy: 1.40,
                shadow: Some((0.0, 14.0, 36.0, if is_dark { 0.60 } else { 0.22 })),
                vibrancy_material: MacosVibrancyMaterial::Sheet,
            },

            MacosContainerClass::Toast => MacosGlassStyle {
                blur_radius: MACOS_POPOVER_BLUR_RADIUS * self.blur_factor,
                surface_rgba: if is_dark {
                    (32, 38, 50, 0.90 * self.opacity_factor)
                } else {
                    (255, 255, 255, 0.92 * self.opacity_factor)
                },
                border_rgba: (colors.accent.0, colors.accent.1, colors.accent.2, 0.45),
                specular_highlight_rgba: (
                    255,
                    255,
                    255,
                    if is_dark {
                        0.20 * self.specular_intensity
                    } else {
                        0.40 * self.specular_intensity
                    },
                ),
                border_width: 1.0,
                corner_radius: 12.0,
                vibrancy: 1.30,
                shadow: Some((0.0, 6.0, 18.0, if is_dark { 0.40 } else { 0.15 })),
                vibrancy_material: MacosVibrancyMaterial::Popover,
            },

            MacosContainerClass::FilterBar => MacosGlassStyle {
                blur_radius: 22.0 * self.blur_factor,
                surface_rgba: if is_dark {
                    (30, 35, 45, 0.75 * self.opacity_factor)
                } else {
                    (246, 248, 252, 0.80 * self.opacity_factor)
                },
                border_rgba: if is_dark {
                    (255, 255, 255, 0.09)
                } else {
                    (0, 0, 0, 0.06)
                },
                specular_highlight_rgba: (
                    255,
                    255,
                    255,
                    if is_dark {
                        0.14 * self.specular_intensity
                    } else {
                        0.30 * self.specular_intensity
                    },
                ),
                border_width: 1.0,
                corner_radius: 8.0,
                vibrancy: 1.15,
                shadow: None,
                vibrancy_material: MacosVibrancyMaterial::HudWindow,
            },
        }
    }

    /// Generates special high-visibility overdue card style with specular red glass border highlight.
    #[must_use]
    pub fn overdue_card_style(
        &self,
        base_style: &MacosGlassStyle,
        colors: &ColorTokens,
    ) -> MacosGlassStyle {
        let mut style = *base_style;
        style.border_rgba = (
            colors.overdue_alert.0,
            colors.overdue_alert.1,
            colors.overdue_alert.2,
            0.90,
        );
        style.specular_highlight_rgba = (255, 120, 120, 0.50);
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
    fn test_macos_liquid_glass_styles_dark_and_light() {
        let glass = MacosLiquidGlass::new();
        let dark_theme = AppTheme::new(ThemeMode::Dark, true);
        let light_theme = AppTheme::new(ThemeMode::Light, false);

        let dark_toolbar = glass.style_for(MacosContainerClass::Toolbar, &dark_theme);
        assert_eq!(dark_toolbar.blur_radius, 24.0);
        assert_eq!(
            dark_toolbar.vibrancy_material,
            MacosVibrancyMaterial::HeaderView
        );
        assert!(dark_toolbar.specular_highlight_rgba.3 > 0.0);

        let light_modal = glass.style_for(MacosContainerClass::ModalDrawer, &light_theme);
        assert_eq!(light_modal.blur_radius, 36.0);
        assert_eq!(light_modal.corner_radius, 14.0);
        assert_eq!(light_modal.vibrancy_material, MacosVibrancyMaterial::Sheet);
    }

    #[test]
    fn test_macos_overdue_card_style() {
        let glass = MacosLiquidGlass::new();
        let theme = AppTheme::new(ThemeMode::Dark, true);
        let card = glass.style_for(MacosContainerClass::ArticleCard, &theme);
        let overdue = glass.overdue_card_style(&card, &theme.colors);

        assert_eq!(overdue.border_width, 2.0);
        assert_eq!(overdue.border_rgba.0, theme.colors.overdue_alert.0);
        assert_eq!(overdue.border_rgba.1, theme.colors.overdue_alert.1);
        assert_eq!(overdue.border_rgba.2, theme.colors.overdue_alert.2);
        assert_eq!(overdue.border_rgba.3, 0.90);
        assert!(overdue.shadow.is_some());
    }
}
