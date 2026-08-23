//! COSMIC desktop integration for Linux (`libcosmic`).
//!
//! Provides the COSMIC application wrapper, frosted glass theme materials,
//! application icon management, header bar, left navigation bar, and window sizing configurations.

pub mod app;
pub mod glass;
pub mod header_bar;
pub mod icon;
pub mod sidebar;
pub mod theme;
pub mod window;

pub use app::{CosmicApp, CosmicAppConfig, CosmicViewTreeDescriptor};
pub use glass::{
    CosmicContainerClass, CosmicFrostedGlass, CosmicGlassStyle, COSMIC_CARD_BLUR_RADIUS,
    COSMIC_DEEP_BLUR_RADIUS, COSMIC_DEFAULT_BLUR_RADIUS,
};
pub use header_bar::{CosmicHeaderBar, CosmicHeaderBarAction};
pub use icon::{
    CosmicIconManager, CosmicIconSize, COSMIC_APP_ICON_NAME, COSMIC_APP_ID,
    COSMIC_SYMBOLIC_ICON_NAME, NEWSJOURNAL_ICON_SVG, NEWSJOURNAL_SYMBOLIC_ICON_SVG,
};
pub use sidebar::{
    CosmicNavAction, CosmicNavBar, CosmicNavItemStyle, COSMIC_NAV_ITEM_HEIGHT,
    COSMIC_SIDEBAR_COMPACT_WIDTH, COSMIC_SIDEBAR_STANDARD_WIDTH,
};
pub use theme::{CosmicAccentColor, CosmicThemeAdapter, CosmicThemeMode};
pub use window::{
    CosmicResponsiveBreakpoint, CosmicWindowConfig, CosmicWindowPlacement,
    COSMIC_DEFAULT_WINDOW_HEIGHT, COSMIC_DEFAULT_WINDOW_WIDTH, COSMIC_MIN_WINDOW_HEIGHT,
    COSMIC_MIN_WINDOW_WIDTH,
};
