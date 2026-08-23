//! COSMIC application icon assets, identifiers, and desktop integration.

use serde::{Deserialize, Serialize};

/// Canonical COSMIC Application ID for NewsJournal.
pub const COSMIC_APP_ID: &str = "io.github.jcray.newsjournal";

/// Standard icon name for system desktop entries and launcher icon themes.
pub const COSMIC_APP_ICON_NAME: &str = "newsjournal";

/// Symbolic icon name used in header bars and status indicators.
pub const COSMIC_SYMBOLIC_ICON_NAME: &str = "newsjournal-symbolic";

/// Raw embedded SVG content for the main application icon.
pub const NEWSJOURNAL_ICON_SVG: &str = include_str!("../../../assets/icons/newsjournal.svg");

/// Raw embedded SVG content for the symbolic header icon.
pub const NEWSJOURNAL_SYMBOLIC_ICON_SVG: &str =
    include_str!("../../../assets/icons/newsjournal-symbolic.svg");

/// Icon resolution and format specifications for COSMIC desktop integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CosmicIconSize {
    /// 16x16 pixels (symbolic toolbar size).
    Symbolic16,
    /// 24x24 pixels (standard header button size).
    Small24,
    /// 32x32 pixels (compact menu size).
    Medium32,
    /// 48x48 pixels (launcher size).
    Large48,
    /// 64x64 pixels (high-DPI launcher size).
    ExtraLarge64,
    /// 128x128 pixels (full fidelity app tile).
    Tile128,
    /// 256x256 pixels (ultra-HD icon).
    Ultra256,
}

impl CosmicIconSize {
    /// Returns the pixel dimension as an integer.
    #[must_use]
    pub const fn pixels(&self) -> u32 {
        match self {
            Self::Symbolic16 => 16,
            Self::Small24 => 24,
            Self::Medium32 => 32,
            Self::Large48 => 48,
            Self::ExtraLarge64 => 64,
            Self::Tile128 => 128,
            Self::Ultra256 => 256,
        }
    }
}

/// Icon management coordinator for COSMIC desktop integration.
#[derive(Debug, Clone, Default)]
pub struct CosmicIconManager;

impl CosmicIconManager {
    /// Creates a new icon manager instance.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Returns the primary application ID.
    #[must_use]
    pub const fn app_id(&self) -> &'static str {
        COSMIC_APP_ID
    }

    /// Returns the standard icon name for desktop entry registration.
    #[must_use]
    pub const fn icon_name(&self) -> &'static str {
        COSMIC_APP_ICON_NAME
    }

    /// Returns the symbolic icon name for header bar usage.
    #[must_use]
    pub const fn symbolic_icon_name(&self) -> &'static str {
        COSMIC_SYMBOLIC_ICON_NAME
    }

    /// Returns the raw SVG bytes of the full application icon.
    #[must_use]
    pub fn app_icon_svg_bytes(&self) -> &'static [u8] {
        NEWSJOURNAL_ICON_SVG.as_bytes()
    }

    /// Returns the raw SVG bytes of the symbolic application icon.
    #[must_use]
    pub fn symbolic_icon_svg_bytes(&self) -> &'static [u8] {
        NEWSJOURNAL_SYMBOLIC_ICON_SVG.as_bytes()
    }

    /// Validates that embedded icon assets are non-empty and well-formed XML/SVG.
    #[must_use]
    pub fn validate_assets(&self) -> bool {
        let full_svg = NEWSJOURNAL_ICON_SVG.trim();
        let sym_svg = NEWSJOURNAL_SYMBOLIC_ICON_SVG.trim();

        full_svg.starts_with("<svg")
            && full_svg.ends_with("</svg>")
            && sym_svg.starts_with("<svg")
            && sym_svg.ends_with("</svg>")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosmic_icon_metadata_and_constants() {
        let manager = CosmicIconManager::new();
        assert_eq!(manager.app_id(), "io.github.jcray.newsjournal");
        assert_eq!(manager.icon_name(), "newsjournal");
        assert_eq!(manager.symbolic_icon_name(), "newsjournal-symbolic");
        assert!(manager.validate_assets());
    }

    #[test]
    fn test_cosmic_icon_sizes() {
        assert_eq!(CosmicIconSize::Symbolic16.pixels(), 16);
        assert_eq!(CosmicIconSize::Small24.pixels(), 24);
        assert_eq!(CosmicIconSize::Medium32.pixels(), 32);
        assert_eq!(CosmicIconSize::Large48.pixels(), 48);
        assert_eq!(CosmicIconSize::ExtraLarge64.pixels(), 64);
        assert_eq!(CosmicIconSize::Tile128.pixels(), 128);
        assert_eq!(CosmicIconSize::Ultra256.pixels(), 256);
    }

    #[test]
    fn test_icon_svg_content_validity() {
        let manager = CosmicIconManager::new();
        let full = manager.app_icon_svg_bytes();
        let sym = manager.symbolic_icon_svg_bytes();

        assert!(!full.is_empty());
        assert!(!sym.is_empty());

        let full_str = std::str::from_utf8(full).expect("valid utf8");
        let sym_str = std::str::from_utf8(sym).expect("valid utf8");

        assert!(full_str.contains("cosmic-glass-grad"));
        assert!(sym_str.contains("<path"));
    }
}
