//! NewsJournal desktop application entry point.

use newsjournal_core::storage::StorageService;
use newsjournal_gui::{AppState, EventLoop, VERSION};

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

    let event_loop = EventLoop::new(state);
    println!(
        "NewsJournal initialized with {} articles, {} tasks, {} contacts.",
        event_loop.state().articles.len(),
        event_loop.state().tasks.len(),
        event_loop.state().contacts.len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gui_entry_smoketest() {
        let storage = StorageService::in_memory().unwrap();
        let mut state = AppState::new(storage);
        assert!(state.load_all().is_ok());
        let event_loop = EventLoop::new(state);
        assert_eq!(event_loop.state().articles.len(), 0);
    }
}
