//! Command side-effects and execution abstraction for NewsJournal.

use newsjournal_core::models::{Article, ArticleStage, Contact, Settings, Task, TaskStatus};
use newsjournal_core::storage::{StorageError, StorageService};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::message::AppMessage;
use crate::state::toast::ToastMessage;

/// Asynchronous or synchronous side-effect commands returned by the state reducer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppCommand {
    /// No operation.
    None,
    /// Persist or update an article in storage.
    SaveArticle(Article),
    /// Delete an article from storage.
    DeleteArticle(Uuid),
    /// Update an article's stage in storage.
    MoveArticleStage {
        /// Target article ID.
        id: Uuid,
        /// New production stage.
        stage: ArticleStage,
    },
    /// Persist or update a task in storage.
    SaveTask(Task),
    /// Delete a task from storage.
    DeleteTask(Uuid),
    /// Update a task's status in storage.
    MoveTaskStatus {
        /// Target task ID.
        id: Uuid,
        /// New task status.
        status: TaskStatus,
    },
    /// Persist or update a contact in storage.
    SaveContact(Contact),
    /// Delete a contact from storage.
    DeleteContact(Uuid),
    /// Atomically sync tagged contact IDs for an article.
    SetArticleContacts {
        /// Target article ID.
        article_id: Uuid,
        /// Linked contact IDs.
        contact_ids: Vec<Uuid>,
    },
    /// Persist settings in storage.
    SaveSettings(Settings),
    /// Reload all entities from storage.
    ReloadAllData,
    /// Emit a notification toast.
    EmitToast(ToastMessage),
    /// Emit a top-level error banner.
    EmitError(String),
}

/// Executes commands against the persistence storage layer.
#[derive(Debug, Clone, Default)]
pub struct CommandExecutor;

impl CommandExecutor {
    /// Creates a new command executor.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Executes an [`AppCommand`] against the provided storage service, optionally returning
    /// follow-up messages (e.g. refresh, toast notification).
    pub fn execute(
        &self,
        command: AppCommand,
        storage: &StorageService,
    ) -> Result<Option<AppMessage>, StorageError> {
        match command {
            AppCommand::None => Ok(None),
            AppCommand::SaveArticle(article) => {
                let existing = storage.get_article(article.id)?;
                if existing.is_some() {
                    storage.update_article(article)?;
                } else {
                    storage.create_article(article)?;
                }
                Ok(Some(AppMessage::Refresh))
            }
            AppCommand::DeleteArticle(id) => {
                storage.delete_article(id)?;
                Ok(Some(AppMessage::Refresh))
            }
            AppCommand::MoveArticleStage { id, stage } => {
                if let Some(mut article) = storage.get_article(id)? {
                    article.stage = stage;
                    storage.update_article(article)?;
                }
                Ok(Some(AppMessage::Refresh))
            }
            AppCommand::SaveTask(task) => {
                let existing = storage.get_task(task.id)?;
                if existing.is_some() {
                    storage.update_task(task)?;
                } else {
                    storage.create_task(task)?;
                }
                Ok(Some(AppMessage::Refresh))
            }
            AppCommand::DeleteTask(id) => {
                storage.delete_task(id)?;
                Ok(Some(AppMessage::Refresh))
            }
            AppCommand::MoveTaskStatus { id, status } => {
                if let Some(mut task) = storage.get_task(id)? {
                    task.status = status;
                    storage.update_task(task)?;
                }
                Ok(Some(AppMessage::Refresh))
            }
            AppCommand::SaveContact(contact) => {
                let existing = storage.get_contact(contact.id)?;
                if existing.is_some() {
                    storage.update_contact(contact)?;
                } else {
                    storage.create_contact(contact)?;
                }
                Ok(Some(AppMessage::Refresh))
            }
            AppCommand::DeleteContact(id) => {
                storage.delete_contact(id)?;
                Ok(Some(AppMessage::Refresh))
            }
            AppCommand::SetArticleContacts {
                article_id,
                contact_ids,
            } => {
                storage.set_article_contacts(article_id, &contact_ids)?;
                Ok(Some(AppMessage::Refresh))
            }
            AppCommand::SaveSettings(settings) => {
                storage.save_settings(&settings)?;
                Ok(Some(AppMessage::Refresh))
            }
            AppCommand::ReloadAllData => Ok(Some(AppMessage::Refresh)),
            AppCommand::EmitToast(toast) => Ok(Some(AppMessage::PushToast(toast))),
            AppCommand::EmitError(msg) => Ok(Some(AppMessage::SetError(Some(msg)))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_execution_article_crud() {
        let storage = StorageService::in_memory().unwrap();
        let executor = CommandExecutor::new();

        let article = Article::new("breaking-news", "Breaking News Headline");
        let article_id = article.id;

        // Save
        let msg = executor
            .execute(AppCommand::SaveArticle(article), &storage)
            .unwrap();
        assert_eq!(msg, Some(AppMessage::Refresh));
        assert!(storage.get_article(article_id).unwrap().is_some());

        // Move stage
        let msg = executor
            .execute(
                AppCommand::MoveArticleStage {
                    id: article_id,
                    stage: ArticleStage::Writing,
                },
                &storage,
            )
            .unwrap();
        assert_eq!(msg, Some(AppMessage::Refresh));
        assert_eq!(
            storage.get_article(article_id).unwrap().unwrap().stage,
            ArticleStage::Writing
        );

        // Delete
        let msg = executor
            .execute(AppCommand::DeleteArticle(article_id), &storage)
            .unwrap();
        assert_eq!(msg, Some(AppMessage::Refresh));
        assert!(storage.get_article(article_id).unwrap().is_none());
    }
}
