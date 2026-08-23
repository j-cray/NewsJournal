//! NewsJournal desktop application entry point.

use newsjournal_core::storage::StorageService;
use newsjournal_gui::{AppState, VERSION};

fn main() -> iced::Result {
    println!("Starting NewsJournal v{VERSION}...");

    let storage = match StorageService::open_default() {
        Ok(storage) => {
            println!("Connected to database successfully.");
            storage
        }
        Err(e) => {
            eprintln!("Warning: Failed to open default database ({e}), falling back to in-memory database.");
            StorageService::in_memory().expect("failed to open in-memory storage fallback")
        }
    };

    let mut state = AppState::new(storage);
    if let Err(e) = state.load_all() {
        eprintln!("Warning: Initial data load encountered an error: {e}");
    }

    println!(
        "Loaded {} articles, {} tasks, {} contacts.",
        state.articles.len(),
        state.tasks.len(),
        state.contacts.len()
    );

    #[cfg(target_os = "linux")]
    {
        println!(
            "NewsJournal COSMIC desktop wrapper initialized (App ID: io.github.jcray.newsjournal, Window: 1280x800, Frosted Glass: enabled)."
        );
    }

    #[cfg(target_os = "macos")]
    {
        println!(
            "NewsJournal macOS Liquid Glass wrapper initialized (Window: 1280x800, Vibrancy: enabled)."
        );
    }

    println!("Launching NewsJournal interactive desktop GUI window (1280x800)...");
    newsjournal_gui::run_app(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "linux")]
    use newsjournal_gui::cosmic::CosmicApp;
    #[cfg(target_os = "macos")]
    use newsjournal_gui::macos::MacosApp;
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    use newsjournal_gui::EventLoop;
    use newsjournal_gui::NewsJournalApp;

    #[test]
    fn test_gui_entry_smoketest() {
        let storage = StorageService::in_memory().unwrap();
        let mut state = AppState::new(storage);
        assert!(state.load_all().is_ok());

        let app = NewsJournalApp::new(state.clone());
        assert_eq!(app.state().articles.len(), 0);

        #[cfg(target_os = "linux")]
        {
            let cosmic_app = CosmicApp::new(state);
            assert_eq!(cosmic_app.state().articles.len(), 0);
            assert_eq!(
                cosmic_app.icon_manager.app_id(),
                "io.github.jcray.newsjournal"
            );
        }

        #[cfg(target_os = "macos")]
        {
            let macos_app = MacosApp::new(state);
            assert_eq!(macos_app.state().articles.len(), 0);
            assert_eq!(macos_app.toolbar.app_title, "NewsJournal");
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            let event_loop = EventLoop::new(state);
            assert_eq!(event_loop.state().articles.len(), 0);
        }
    }
}
