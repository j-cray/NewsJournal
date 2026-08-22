//! Command executor unit and integration tests.

use newsjournal_core::models::{Article, Contact, Settings, Task, TaskStatus, ThemeMode};
use newsjournal_core::storage::StorageService;
use newsjournal_core::ArticleStage;
use newsjournal_gui::commands::{AppCommand, CommandExecutor};
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::state::toast::ToastMessage;

#[test]
fn test_command_executor_all_variants() {
    let storage = StorageService::in_memory().unwrap();
    let executor = CommandExecutor::new();

    // 1. None
    let res = executor.execute(AppCommand::None, &storage).unwrap();
    assert_eq!(res, None);

    // 2. Save Article
    let article = Article::new("court-ruling", "Federal Court Issues Injunction");
    let article_id = article.id;
    let res = executor
        .execute(AppCommand::SaveArticle(article), &storage)
        .unwrap();
    assert_eq!(res, Some(AppMessage::Refresh));
    assert!(storage.get_article(article_id).unwrap().is_some());

    // 3. Move Article Stage
    let res = executor
        .execute(
            AppCommand::MoveArticleStage {
                id: article_id,
                stage: ArticleStage::ReadyToPublish,
            },
            &storage,
        )
        .unwrap();
    assert_eq!(res, Some(AppMessage::Refresh));
    assert_eq!(
        storage.get_article(article_id).unwrap().unwrap().stage,
        ArticleStage::ReadyToPublish
    );

    // 4. Save Task
    let task = Task::new(article_id, "Obtain copy of legal brief");
    let task_id = task.id;
    let res = executor
        .execute(AppCommand::SaveTask(task), &storage)
        .unwrap();
    assert_eq!(res, Some(AppMessage::Refresh));
    assert!(storage.get_task(task_id).unwrap().is_some());

    // 5. Move Task Status
    let res = executor
        .execute(
            AppCommand::MoveTaskStatus {
                id: task_id,
                status: TaskStatus::InProgress,
            },
            &storage,
        )
        .unwrap();
    assert_eq!(res, Some(AppMessage::Refresh));
    assert_eq!(
        storage.get_task(task_id).unwrap().unwrap().status,
        TaskStatus::InProgress
    );

    // 6. Save Contact
    let contact = Contact::new("Judge Advocate");
    let contact_id = contact.id;
    let res = executor
        .execute(AppCommand::SaveContact(contact), &storage)
        .unwrap();
    assert_eq!(res, Some(AppMessage::Refresh));
    assert!(storage.get_contact(contact_id).unwrap().is_some());

    // 7. Set Article Contacts
    let res = executor
        .execute(
            AppCommand::SetArticleContacts {
                article_id,
                contact_ids: vec![contact_id],
            },
            &storage,
        )
        .unwrap();
    assert_eq!(res, Some(AppMessage::Refresh));
    assert_eq!(
        storage.list_contacts_for_article(article_id).unwrap().len(),
        1
    );

    // 8. Save Settings
    let settings = Settings::new(ThemeMode::Dark);
    let res = executor
        .execute(AppCommand::SaveSettings(settings), &storage)
        .unwrap();
    assert_eq!(res, Some(AppMessage::Refresh));
    assert_eq!(storage.get_settings().unwrap().theme_mode, ThemeMode::Dark);

    // 9. Toast / Error / Reload
    let toast = ToastMessage::info("T", "B");
    let res = executor
        .execute(AppCommand::EmitToast(toast.clone()), &storage)
        .unwrap();
    assert_eq!(res, Some(AppMessage::PushToast(toast)));

    let res = executor
        .execute(AppCommand::EmitError("DB Err".into()), &storage)
        .unwrap();
    assert_eq!(res, Some(AppMessage::SetError(Some("DB Err".into()))));

    let res = executor
        .execute(AppCommand::ReloadAllData, &storage)
        .unwrap();
    assert_eq!(res, Some(AppMessage::Refresh));

    // 10. Delete Task, Contact, Article
    executor
        .execute(AppCommand::DeleteTask(task_id), &storage)
        .unwrap();
    assert!(storage.get_task(task_id).unwrap().is_none());

    executor
        .execute(AppCommand::DeleteContact(contact_id), &storage)
        .unwrap();
    assert!(storage.get_contact(contact_id).unwrap().is_none());

    executor
        .execute(AppCommand::DeleteArticle(article_id), &storage)
        .unwrap();
    assert!(storage.get_article(article_id).unwrap().is_none());
}
