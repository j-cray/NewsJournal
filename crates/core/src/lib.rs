//! `newsjournal-core`
//!
//! Pure Rust domain logic, data models, validation, and persistence for the NewsJournal application.

/// NewsJournal core crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_version_present() {
        assert!(!VERSION.is_empty());
    }
}
