//! macOS desktop integration for Liquid Glass and native vibrancy (`iced` + `window_vibrancy`).
//!
//! Provides the macOS application wrapper, liquid glass materials, native `NSVisualEffectView`
//! vibrancy configurations, unified toolbar, system appearance adapter, and window management.

pub mod app;
pub mod glass;
pub mod theme;
pub mod toolbar;
pub mod vibrancy;
pub mod window;

pub use app::{MacosApp, MacosAppConfig, MacosViewTreeDescriptor};
pub use glass::{
    MacosContainerClass, MacosGlassStyle, MacosLiquidGlass, MACOS_CARD_BLUR_RADIUS,
    MACOS_DEFAULT_BLUR_RADIUS, MACOS_POPOVER_BLUR_RADIUS, MACOS_SHEET_BLUR_RADIUS,
};
pub use theme::{MacosAccentColor, MacosAppearanceMode, MacosThemeAdapter};
pub use toolbar::{MacosToolbar, MacosToolbarAction, MacosToolbarTabItem};
pub use vibrancy::{
    MacosVibrancyBlendingMode, MacosVibrancyConfig, MacosVibrancyMaterial, MacosVibrancyState,
};
pub use window::{
    MacosResponsiveBreakpoint, MacosTitlebarStyle, MacosWindowConfig, MacosWindowPlacement,
    MACOS_DEFAULT_WINDOW_HEIGHT, MACOS_DEFAULT_WINDOW_WIDTH, MACOS_MIN_WINDOW_HEIGHT,
    MACOS_MIN_WINDOW_WIDTH, MACOS_TRAFFIC_LIGHT_INSET_X, MACOS_TRAFFIC_LIGHT_INSET_Y,
};
