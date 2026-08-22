//! Event loop runner and dispatcher coordinating messages, state transitions, and persistence commands.

use std::collections::VecDeque;

use newsjournal_core::storage::StorageError;

use crate::commands::CommandExecutor;
use crate::message::AppMessage;
use crate::state::AppState;

/// Event loop coordinator managing state updates and side-effect command execution.
#[derive(Debug)]
pub struct EventLoop {
    /// Unified application state.
    pub state: AppState,
    /// Storage side-effect command executor.
    pub executor: CommandExecutor,
    /// Pending message queue.
    queue: VecDeque<AppMessage>,
}

impl EventLoop {
    /// Creates a new `EventLoop` wrapping the given application state.
    #[must_use]
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            executor: CommandExecutor::new(),
            queue: VecDeque::new(),
        }
    }

    /// Returns a reference to the underlying application state.
    #[must_use]
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Returns a mutable reference to the underlying application state.
    pub fn state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }

    /// Dispatches a single message through the state reducer, executes resulting side-effect commands,
    /// and recursively processes any emitted follow-up messages until the queue is exhausted.
    pub fn dispatch(&mut self, message: AppMessage) -> Result<(), StorageError> {
        self.queue.push_back(message);

        while let Some(msg) = self.queue.pop_front() {
            let commands = self.state.update(msg);
            for cmd in commands {
                if let Some(follow_up) = self.executor.execute(cmd, &self.state.storage)? {
                    self.queue.push_back(follow_up);
                }
            }
        }

        Ok(())
    }

    /// Dispatches an iterator of messages sequentially.
    pub fn dispatch_batch(
        &mut self,
        messages: impl IntoIterator<Item = AppMessage>,
    ) -> Result<(), StorageError> {
        for msg in messages {
            self.dispatch(msg)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use newsjournal_core::models::Article;
    use newsjournal_core::ArticleStage;

    #[test]
    fn test_event_loop_dispatch_and_cascading_refresh() {
        let state = AppState::in_memory().expect("in-memory state");
        let mut event_loop = EventLoop::new(state);

        let article = Article::new("city-transit", "City Transit Overhaul");
        let article_id = article.id;

        // Dispatch CreateArticle message
        event_loop
            .dispatch(AppMessage::CreateArticle(article))
            .expect("dispatch successful");

        // The CreateArticle command executes SaveArticle, which emits Refresh, which updates the state cache
        assert_eq!(event_loop.state().articles.len(), 1);
        assert_eq!(event_loop.state().articles[0].slug, "city-transit");

        // Dispatch MoveArticleStage
        event_loop
            .dispatch(AppMessage::MoveArticleStage(
                article_id,
                ArticleStage::Editing,
            ))
            .expect("dispatch successful");

        assert_eq!(event_loop.state().articles[0].stage, ArticleStage::Editing);

        // Verify storage was updated
        let from_storage = event_loop
            .state()
            .storage
            .get_article(article_id)
            .unwrap()
            .unwrap();
        assert_eq!(from_storage.stage, ArticleStage::Editing);
    }
}
