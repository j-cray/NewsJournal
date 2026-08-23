//! Platform-specific application integration and native glass wrappers.
//!
//! NewsJournal delivers authentic desktop integration on target platforms:
//! - **Linux**: COSMIC desktop application wrapper (`cosmic`) with frosted glass containers,
//!   COSMIC header bar, system theme synchronization, and window management.
//! - **macOS**: `iced` application with `window_vibrancy` liquid glass and native vibrancy effects.

pub mod cosmic;

#[cfg(target_os = "macos")]
pub mod macos;
