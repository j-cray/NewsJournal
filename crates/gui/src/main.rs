//! NewsJournal desktop application entry point.

use newsjournal_core::storage::StorageService;
#[cfg(target_os = "linux")]
use newsjournal_gui::cosmic::{CosmicApp, CosmicAppConfig};
#[cfg(not(target_os = "linux"))]
use newsjournal_gui::EventLoop;
use newsjournal_gui::{AppState, VERSION};

fn main() {
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

    #[cfg(target_os = "linux")]
    {
        let cosmic_app = CosmicApp::with_config(state, CosmicAppConfig::default());
        println!(
            "NewsJournal COSMIC desktop wrapper initialized (App ID: {}, Window: {}x{}, Frosted Glass: enabled).",
            cosmic_app.icon_manager.app_id(),
            cosmic_app.config.window.placement.width,
            cosmic_app.config.window.placement.height
        );
        println!(
            "Loaded {} articles, {} tasks, {} contacts.",
            cosmic_app.state().articles.len(),
            cosmic_app.state().tasks.len(),
            cosmic_app.state().contacts.len()
        );
    }

    #[cfg(not(target_os = "linux"))]
    {
        let event_loop = EventLoop::new(state);
        println!(
            "NewsJournal initialized with {} articles, {} tasks, {} contacts.",
            event_loop.state().articles.len(),
            event_loop.state().tasks.len(),
            event_loop.state().contacts.len()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gui_entry_smoketest() {
        let storage = StorageService::in_memory().unwrap();
        let mut state = AppState::new(storage);
        assert!(state.load_all().is_ok());

        #[cfg(target_os = "linux")]
        {
            let cosmic_app = CosmicApp::new(state);
            assert_eq!(cosmic_app.state().articles.len(), 0);
            assert_eq!(
                cosmic_app.icon_manager.app_id(),
                "io.github.jcray.newsjournal"
            );
        }

        #[cfg(not(target_os = "linux"))]
        {
            let event_loop = EventLoop::new(state);
            assert_eq!(event_loop.state().articles.len(), 0);
        }
    }
}
