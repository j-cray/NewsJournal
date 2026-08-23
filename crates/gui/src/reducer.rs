//! Pure state transition reducer for the NewsJournal GUI.

use chrono::Utc;
use newsjournal_core::models::{Task, TaskStatus};
use newsjournal_core::validation::validate_task_title;
use uuid::Uuid;

use crate::commands::AppCommand;
use crate::message::AppMessage;
use crate::navigation::{NavKeyAction, NavTab};
use crate::state::drag_drop::{DragItem, DropTarget};
use crate::state::modal::{ArticleDraft, ContactDraft, ModalState, SettingsDraft, TaskDraft};
use crate::state::toast::ToastMessage;
use crate::state::AppState;

impl AppState {
    /// Pure state update reducer that applies incoming [`AppMessage`] events and returns
    /// any required [`AppCommand`] side-effects for execution.
    pub fn update(&mut self, message: AppMessage) -> Vec<AppCommand> {
        match message {
            // ==========================================
            // Navigation
            // ==========================================
            AppMessage::NavigateTo(tab) => {
                self.navigate_to(tab);
                Vec::new()
            }
            AppMessage::NavigateNextTab => {
                self.navigate_next();
                Vec::new()
            }
            AppMessage::NavigatePrevTab => {
                self.navigate_prev();
                Vec::new()
            }
            AppMessage::HandleNavKeyAction(action) => {
                match action {
                    NavKeyAction::SelectTab(tab) => self.navigate_to(tab),
                    NavKeyAction::NextTab => self.navigate_next(),
                    NavKeyAction::PrevTab => self.navigate_prev(),
                    NavKeyAction::FirstTab => self.navigate_to(NavTab::ArticlesKanban),
                    NavKeyAction::LastTab => self.navigate_to(NavTab::Settings),
                }
                Vec::new()
            }

            // ==========================================
            // Modal & Drawer Lifecycle
            // ==========================================
            AppMessage::OpenNewArticleModal => {
                let draft = ArticleDraft::new();
                self.modal = ModalState::ArticleForm(draft);
                Vec::new()
            }
            AppMessage::OpenNewArticleInStageModal(stage) => {
                let draft = ArticleDraft::new_with_stage(stage);
                self.modal = ModalState::ArticleForm(draft);
                Vec::new()
            }
            AppMessage::OpenEditArticleModal(id) => {
                if let Some(article) = self.get_article(id) {
                    let tagged_ids = self.article_contacts.get(&id).cloned().unwrap_or_default();
                    let draft = ArticleDraft::from_article(article, tagged_ids);
                    self.modal = ModalState::ArticleForm(draft);
                }
                Vec::new()
            }
            AppMessage::OpenNewTaskModal(article_id) => {
                let default_article_id = article_id.or_else(|| self.articles.first().map(|a| a.id));
                self.modal = ModalState::TaskForm(TaskDraft::new_for_article(default_article_id));
                Vec::new()
            }
            AppMessage::OpenNewTaskInStatusModal(status, article_id) => {
                let default_article_id = article_id.or_else(|| self.articles.first().map(|a| a.id));
                self.modal =
                    ModalState::TaskForm(TaskDraft::new_for_status(status, default_article_id));
                Vec::new()
            }
            AppMessage::OpenEditTaskModal(id) => {
                if let Some(task) = self.get_task(id) {
                    self.modal = ModalState::TaskForm(TaskDraft::from_task(task));
                }
                Vec::new()
            }
            AppMessage::OpenNewContactModal => {
                self.modal = ModalState::ContactForm(ContactDraft::new());
                Vec::new()
            }
            AppMessage::OpenEditContactModal(id) => {
                if let Some(contact) = self.get_contact(id) {
                    self.modal = ModalState::ContactForm(ContactDraft::from_contact(contact));
                }
                Vec::new()
            }
            AppMessage::OpenSettings => {
                self.modal =
                    ModalState::SettingsDrawer(SettingsDraft::from_settings(&self.settings));
                Vec::new()
            }
            AppMessage::PromptDeleteArticle(id) => {
                if let Some(article) = self.get_article(id) {
                    self.modal = ModalState::ConfirmDeleteArticle {
                        id,
                        slug: article.slug.clone(),
                        headline: article.headline.clone(),
                    };
                }
                Vec::new()
            }
            AppMessage::PromptDeleteTask(id) => {
                if let Some(task) = self.get_task(id) {
                    self.modal = ModalState::ConfirmDeleteTask {
                        id,
                        title: task.title.clone(),
                    };
                }
                Vec::new()
            }
            AppMessage::PromptDeleteContact(id) => {
                if let Some(contact) = self.get_contact(id) {
                    let linked_count = self.contact_articles.get(&id).map(Vec::len).unwrap_or(0);
                    self.modal = ModalState::ConfirmDeleteContact {
                        id,
                        name: contact.name.clone(),
                        linked_article_count: linked_count,
                    };
                }
                Vec::new()
            }
            AppMessage::CloseModal => {
                self.modal.close();
                Vec::new()
            }
            AppMessage::UpdateArticleDraft(mut draft) => {
                let existing_slugs: Vec<(Uuid, String)> = self
                    .articles
                    .iter()
                    .map(|a| (a.id, a.slug.clone()))
                    .collect();
                draft.validate_with_existing_slugs(&existing_slugs);
                self.modal = ModalState::ArticleForm(draft);
                Vec::new()
            }
            AppMessage::UpdateArticleSlug(slug) => {
                if let ModalState::ArticleForm(mut draft) = self.modal.clone() {
                    draft.set_slug(&slug);
                    let existing_slugs: Vec<(Uuid, String)> = self
                        .articles
                        .iter()
                        .map(|a| (a.id, a.slug.clone()))
                        .collect();
                    draft.validate_with_existing_slugs(&existing_slugs);
                    self.modal = ModalState::ArticleForm(draft);
                }
                Vec::new()
            }
            AppMessage::UpdateArticleHeadline(headline) => {
                if let ModalState::ArticleForm(mut draft) = self.modal.clone() {
                    draft.set_headline(&headline, true);
                    let existing_slugs: Vec<(Uuid, String)> = self
                        .articles
                        .iter()
                        .map(|a| (a.id, a.slug.clone()))
                        .collect();
                    draft.validate_with_existing_slugs(&existing_slugs);
                    self.modal = ModalState::ArticleForm(draft);
                }
                Vec::new()
            }
            AppMessage::UpdateArticleDescription(description) => {
                if let ModalState::ArticleForm(mut draft) = self.modal.clone() {
                    draft.set_description(&description);
                    self.modal = ModalState::ArticleForm(draft);
                }
                Vec::new()
            }
            AppMessage::SetArticleDraftStage(stage) => {
                if let ModalState::ArticleForm(mut draft) = self.modal.clone() {
                    draft.set_stage(stage);
                    self.modal = ModalState::ArticleForm(draft);
                }
                Vec::new()
            }
            AppMessage::SetArticleDraftDeadline(deadline) => {
                if let ModalState::ArticleForm(mut draft) = self.modal.clone() {
                    draft.set_deadline(deadline);
                    self.modal = ModalState::ArticleForm(draft);
                }
                Vec::new()
            }
            AppMessage::SetArticleDraftDeadlinePreset(preset) => {
                if let ModalState::ArticleForm(mut draft) = self.modal.clone() {
                    let now = Utc::now();
                    draft.set_deadline(preset.calculate_target_datetime(now));
                    self.modal = ModalState::ArticleForm(draft);
                }
                Vec::new()
            }
            AppMessage::SetArticleDraftColor(color_hex) => {
                if let ModalState::ArticleForm(mut draft) = self.modal.clone() {
                    draft.set_color(&color_hex);
                    draft.validate();
                    self.modal = ModalState::ArticleForm(draft);
                }
                Vec::new()
            }
            AppMessage::ResetArticleDraftColorToSlug => {
                if let ModalState::ArticleForm(mut draft) = self.modal.clone() {
                    draft.reset_color_to_hash();
                    self.modal = ModalState::ArticleForm(draft);
                }
                Vec::new()
            }
            AppMessage::ToggleArticleDraftContact(contact_id) => {
                if let ModalState::ArticleForm(mut draft) = self.modal.clone() {
                    draft.toggle_contact(contact_id);
                    self.modal = ModalState::ArticleForm(draft);
                }
                Vec::new()
            }
            AppMessage::AddArticleDraftContact(contact_id) => {
                if let ModalState::ArticleForm(mut draft) = self.modal.clone() {
                    draft.tag_contact(contact_id);
                    self.modal = ModalState::ArticleForm(draft);
                }
                Vec::new()
            }
            AppMessage::RemoveArticleDraftContact(contact_id) => {
                if let ModalState::ArticleForm(mut draft) = self.modal.clone() {
                    draft.untag_contact(contact_id);
                    self.modal = ModalState::ArticleForm(draft);
                }
                Vec::new()
            }
            AppMessage::SetArticleDraftContactSearch(query) => {
                if let ModalState::ArticleForm(mut draft) = self.modal.clone() {
                    draft.set_contact_search(&query);
                    self.modal = ModalState::ArticleForm(draft);
                }
                Vec::new()
            }
            AppMessage::OpenArticleDraftInlineContact => {
                if let ModalState::ArticleForm(mut draft) = self.modal.clone() {
                    draft.open_inline_contact();
                    self.modal = ModalState::ArticleForm(draft);
                }
                Vec::new()
            }
            AppMessage::CloseArticleDraftInlineContact => {
                if let ModalState::ArticleForm(mut draft) = self.modal.clone() {
                    draft.close_inline_contact();
                    self.modal = ModalState::ArticleForm(draft);
                }
                Vec::new()
            }
            AppMessage::UpdateArticleDraftInlineContact(contact_draft) => {
                if let ModalState::ArticleForm(mut draft) = self.modal.clone() {
                    draft.inline_contact = Some(Box::new(contact_draft));
                    self.modal = ModalState::ArticleForm(draft);
                }
                Vec::new()
            }
            AppMessage::SaveArticleDraftInlineContact => {
                if let ModalState::ArticleForm(mut draft) = self.modal.clone() {
                    if let Some(mut contact_draft) = draft.inline_contact.as_deref().cloned() {
                        if contact_draft.validate() {
                            match contact_draft.to_contact() {
                                Ok(contact) => {
                                    let contact_id = contact.id;
                                    let contact_name = contact.name.clone();
                                    draft.tag_contact(contact_id);
                                    draft.close_inline_contact();
                                    draft.clear_contact_search();
                                    self.modal = ModalState::ArticleForm(draft);

                                    // Add to in-memory state so it's immediately available to views
                                    if !self.contacts.iter().any(|c| c.id == contact_id) {
                                        self.contacts.push(contact.clone());
                                    }

                                    vec![
                                        AppCommand::SaveContact(contact),
                                        AppCommand::EmitToast(ToastMessage::success(
                                            "Contact Created",
                                            format!("Added and tagged '{contact_name}'"),
                                        )),
                                    ]
                                }
                                Err(err) => {
                                    draft.inline_contact = Some(Box::new(contact_draft));
                                    self.modal = ModalState::ArticleForm(draft);
                                    vec![AppCommand::EmitError(err)]
                                }
                            }
                        } else {
                            draft.inline_contact = Some(Box::new(contact_draft));
                            self.modal = ModalState::ArticleForm(draft);
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
            AppMessage::SetArticleDraftQuickTask(title) => {
                if let ModalState::ArticleForm(mut draft) = self.modal.clone() {
                    draft.set_quick_task_title(&title);
                    self.modal = ModalState::ArticleForm(draft);
                }
                Vec::new()
            }
            AppMessage::AddArticleDraftQuickTask => {
                if let ModalState::ArticleForm(mut draft) = self.modal.clone() {
                    let clean_title = draft.quick_task_title.trim().to_string();
                    if let Err(e) = validate_task_title(&clean_title) {
                        return vec![AppCommand::EmitToast(ToastMessage::error(
                            "Invalid Task Title",
                            e.to_string(),
                        ))];
                    }

                    if let Some(article_id) = draft.id {
                        let task = Task::new(article_id, &clean_title);
                        draft.clear_quick_task_title();
                        self.modal = ModalState::ArticleForm(draft);

                        if !self.tasks.iter().any(|t| t.id == task.id) {
                            self.tasks.push(task.clone());
                        }

                        vec![
                            AppCommand::SaveTask(task),
                            AppCommand::EmitToast(ToastMessage::success(
                                "Task Added",
                                format!("Added '{clean_title}' to story"),
                            )),
                        ]
                    } else {
                        match draft.add_staged_task(&clean_title) {
                            Ok(task) => {
                                let title = task.title.clone();
                                self.modal = ModalState::ArticleForm(draft);
                                vec![AppCommand::EmitToast(ToastMessage::info(
                                    "Task Staged",
                                    format!("Added '{title}' (will be saved with story)"),
                                ))]
                            }
                            Err(e) => {
                                self.modal = ModalState::ArticleForm(draft);
                                vec![AppCommand::EmitToast(ToastMessage::error(
                                    "Invalid Task Title",
                                    e,
                                ))]
                            }
                        }
                    }
                } else {
                    Vec::new()
                }
            }
            AppMessage::ToggleArticleDraftTask(task_id) => {
                let mut commands = Vec::new();
                if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                    task.status = if task.status == TaskStatus::Complete {
                        TaskStatus::ToDo
                    } else {
                        TaskStatus::Complete
                    };
                    task.updated_at = Utc::now();
                    commands.push(AppCommand::SaveTask(task.clone()));
                } else if let ModalState::ArticleForm(mut draft) = self.modal.clone() {
                    draft.toggle_staged_task(task_id);
                    self.modal = ModalState::ArticleForm(draft);
                }
                commands
            }
            AppMessage::DeleteArticleDraftTask(task_id) => {
                let mut commands = Vec::new();
                if let Some(pos) = self.tasks.iter().position(|t| t.id == task_id) {
                    let task = self.tasks.remove(pos);
                    commands.push(AppCommand::DeleteTask(task_id));
                    commands.push(AppCommand::EmitToast(ToastMessage::info(
                        "Task Removed",
                        format!("Deleted task '{}'", task.title),
                    )));
                } else if let ModalState::ArticleForm(mut draft) = self.modal.clone() {
                    draft.remove_staged_task(task_id);
                    self.modal = ModalState::ArticleForm(draft);
                    commands.push(AppCommand::EmitToast(ToastMessage::info(
                        "Task Removed",
                        "Removed staged task from draft",
                    )));
                }
                commands
            }
            AppMessage::UpdateTaskDraft(draft) => {
                self.modal = ModalState::TaskForm(draft);
                Vec::new()
            }
            AppMessage::UpdateTaskDraftTitle(title) => {
                if let ModalState::TaskForm(mut draft) = self.modal.clone() {
                    draft.set_title(&title);
                    self.modal = ModalState::TaskForm(draft);
                }
                Vec::new()
            }
            AppMessage::UpdateTaskDraftArticle(article_id) => {
                if let ModalState::TaskForm(mut draft) = self.modal.clone() {
                    draft.set_article_id(article_id);
                    self.modal = ModalState::TaskForm(draft);
                }
                Vec::new()
            }
            AppMessage::UpdateTaskDraftNotes(notes) => {
                if let ModalState::TaskForm(mut draft) = self.modal.clone() {
                    draft.set_notes(&notes);
                    self.modal = ModalState::TaskForm(draft);
                }
                Vec::new()
            }
            AppMessage::SetTaskDraftStatus(status) => {
                if let ModalState::TaskForm(mut draft) = self.modal.clone() {
                    draft.set_status(status);
                    self.modal = ModalState::TaskForm(draft);
                }
                Vec::new()
            }
            AppMessage::SetTaskDraftDueDate(due_date) => {
                if let ModalState::TaskForm(mut draft) = self.modal.clone() {
                    draft.set_due_date(due_date);
                    self.modal = ModalState::TaskForm(draft);
                }
                Vec::new()
            }
            AppMessage::SetTaskDraftDueDatePreset(preset) => {
                if let ModalState::TaskForm(mut draft) = self.modal.clone() {
                    let now = Utc::now();
                    draft.set_due_date(preset.calculate_target_datetime(now));
                    self.modal = ModalState::TaskForm(draft);
                }
                Vec::new()
            }
            AppMessage::ClearTaskDraftDueDate => {
                if let ModalState::TaskForm(mut draft) = self.modal.clone() {
                    draft.clear_due_date();
                    self.modal = ModalState::TaskForm(draft);
                }
                Vec::new()
            }
            AppMessage::UpdateContactDraft(draft) => {
                self.modal = ModalState::ContactForm(draft);
                Vec::new()
            }
            AppMessage::UpdateContactDraftName(name) => {
                if let ModalState::ContactForm(ref mut draft) = self.modal {
                    draft.set_name(&name);
                }
                Vec::new()
            }
            AppMessage::UpdateContactDraftOrg(org) => {
                if let ModalState::ContactForm(ref mut draft) = self.modal {
                    draft.set_organization(&org);
                }
                Vec::new()
            }
            AppMessage::UpdateContactDraftRole(role) => {
                if let ModalState::ContactForm(ref mut draft) = self.modal {
                    draft.set_role(&role);
                }
                Vec::new()
            }
            AppMessage::UpdateContactDraftPhone(phone) => {
                if let ModalState::ContactForm(ref mut draft) = self.modal {
                    draft.set_phone(&phone);
                }
                Vec::new()
            }
            AppMessage::UpdateContactDraftEmail(email) => {
                if let ModalState::ContactForm(ref mut draft) = self.modal {
                    draft.set_email(&email);
                }
                Vec::new()
            }
            AppMessage::UpdateContactDraftNotes(notes) => {
                if let ModalState::ContactForm(ref mut draft) = self.modal {
                    draft.set_notes(&notes);
                }
                Vec::new()
            }
            AppMessage::UpdateSettingsDraft(draft) => {
                self.modal = ModalState::SettingsDrawer(draft);
                Vec::new()
            }
            AppMessage::SubmitModal => {
                let current_modal = std::mem::take(&mut self.modal);
                match current_modal {
                    ModalState::ArticleForm(mut draft) => {
                        let existing_slugs: Vec<(Uuid, String)> = self
                            .articles
                            .iter()
                            .map(|a| (a.id, a.slug.clone()))
                            .collect();
                        if draft.validate_with_existing_slugs(&existing_slugs) {
                            match draft.to_article() {
                                Ok(article) => {
                                    let tagged = draft.tagged_contact_ids.clone();
                                    let aid = article.id;
                                    let slug = article.slug.clone();
                                    let mut commands = vec![
                                        AppCommand::SaveArticle(article),
                                        AppCommand::SetArticleContacts {
                                            article_id: aid,
                                            contact_ids: tagged,
                                        },
                                    ];

                                    // Save any staged tasks
                                    for mut staged_task in draft.staged_tasks {
                                        staged_task.article_id = aid;
                                        if !self.tasks.iter().any(|t| t.id == staged_task.id) {
                                            self.tasks.push(staged_task.clone());
                                        }
                                        commands.push(AppCommand::SaveTask(staged_task));
                                    }

                                    commands.push(AppCommand::EmitToast(ToastMessage::success(
                                        "Article Saved",
                                        format!("Story '{slug}' saved successfully"),
                                    )));
                                    commands
                                }
                                Err(err) => {
                                    self.modal = ModalState::ArticleForm(draft);
                                    vec![AppCommand::EmitError(err)]
                                }
                            }
                        } else {
                            let err_count = draft.validation_errors.len();
                            self.modal = ModalState::ArticleForm(draft);
                            vec![AppCommand::EmitToast(ToastMessage::warning(
                                "Validation Error",
                                format!(
                                    "{err_count} required field(s) need attention before saving"
                                ),
                            ))]
                        }
                    }
                    ModalState::TaskForm(mut draft) => {
                        if draft.validate() {
                            match draft.to_task() {
                                Ok(task) => {
                                    let title = task.title.clone();
                                    let task_id = task.id;
                                    if let Some(pos) =
                                        self.tasks.iter().position(|t| t.id == task_id)
                                    {
                                        self.tasks[pos] = task.clone();
                                    } else {
                                        self.tasks.push(task.clone());
                                    }

                                    vec![
                                        AppCommand::SaveTask(task),
                                        AppCommand::EmitToast(ToastMessage::success(
                                            "Task Saved",
                                            format!("Task '{title}' saved successfully"),
                                        )),
                                    ]
                                }
                                Err(err) => {
                                    self.modal = ModalState::TaskForm(draft);
                                    vec![AppCommand::EmitError(err)]
                                }
                            }
                        } else {
                            let err_count = draft.validation_errors.len();
                            self.modal = ModalState::TaskForm(draft);
                            vec![AppCommand::EmitToast(ToastMessage::warning(
                                "Validation Error",
                                format!("{err_count} field(s) need attention before saving"),
                            ))]
                        }
                    }
                    ModalState::ContactForm(mut draft) => {
                        if draft.validate() {
                            match draft.to_contact() {
                                Ok(contact) => {
                                    let name = contact.name.clone();
                                    let cid = contact.id;
                                    if let Some(pos) =
                                        self.contacts.iter().position(|c| c.id == cid)
                                    {
                                        self.contacts[pos] = contact.clone();
                                    } else {
                                        self.contacts.push(contact.clone());
                                    }

                                    vec![
                                        AppCommand::SaveContact(contact),
                                        AppCommand::EmitToast(ToastMessage::success(
                                            "Contact Saved",
                                            format!("Contact '{name}' saved successfully"),
                                        )),
                                    ]
                                }
                                Err(err) => {
                                    self.modal = ModalState::ContactForm(draft);
                                    vec![AppCommand::EmitError(err)]
                                }
                            }
                        } else {
                            let err_count = draft.validation_errors.len();
                            self.modal = ModalState::ContactForm(draft);
                            vec![AppCommand::EmitToast(ToastMessage::warning(
                                "Validation Error",
                                format!("{err_count} field(s) need attention before saving"),
                            ))]
                        }
                    }
                    ModalState::SettingsDrawer(draft) => {
                        let settings = draft.to_settings();
                        self.settings = settings.clone();
                        self.theme_engine.set_mode(settings.theme_mode);
                        self.theme_engine.set_high_contrast(draft.high_contrast);
                        self.theme_engine.set_custom_accent(draft.custom_accent);
                        vec![
                            AppCommand::SaveSettings(settings),
                            AppCommand::EmitToast(ToastMessage::success(
                                "Settings Updated",
                                "Preferences saved",
                            )),
                        ]
                    }
                    ModalState::ConfirmDeleteArticle { id, slug, .. } => {
                        vec![
                            AppCommand::DeleteArticle(id),
                            AppCommand::EmitToast(ToastMessage::info(
                                "Article Deleted",
                                format!("Article '{slug}' deleted"),
                            )),
                        ]
                    }
                    ModalState::ConfirmDeleteTask { id, title } => {
                        if let Some(pos) = self.tasks.iter().position(|t| t.id == id) {
                            self.tasks.remove(pos);
                        }
                        vec![
                            AppCommand::DeleteTask(id),
                            AppCommand::EmitToast(ToastMessage::info(
                                "Task Deleted",
                                format!("Task '{title}' deleted"),
                            )),
                        ]
                    }
                    ModalState::ConfirmDeleteContact { id, name, .. } => {
                        vec![
                            AppCommand::DeleteContact(id),
                            AppCommand::EmitToast(ToastMessage::info(
                                "Contact Deleted",
                                format!("Contact '{name}' deleted"),
                            )),
                        ]
                    }
                    ModalState::None => Vec::new(),
                }
            }

            // ==========================================
            // Direct Entity CRUD Actions
            // ==========================================
            AppMessage::CreateArticle(article) => {
                let slug = article.slug.clone();
                vec![
                    AppCommand::SaveArticle(article),
                    AppCommand::EmitToast(ToastMessage::success(
                        "Article Created",
                        format!("Article '{slug}' created"),
                    )),
                ]
            }
            AppMessage::UpdateArticle(article) => {
                vec![AppCommand::SaveArticle(article)]
            }
            AppMessage::DeleteArticle(id) => {
                vec![AppCommand::DeleteArticle(id)]
            }
            AppMessage::MoveArticleStage(id, stage) => {
                if let Some(a) = self.articles.iter_mut().find(|a| a.id == id) {
                    a.stage = stage;
                }
                self.recalculate_deadlines(Utc::now());
                vec![AppCommand::MoveArticleStage { id, stage }]
            }
            AppMessage::CreateTask(task) => {
                if !self.tasks.iter().any(|t| t.id == task.id) {
                    self.tasks.push(task.clone());
                }
                vec![AppCommand::SaveTask(task)]
            }
            AppMessage::UpdateTask(task) => {
                if let Some(pos) = self.tasks.iter().position(|t| t.id == task.id) {
                    self.tasks[pos] = task.clone();
                } else {
                    self.tasks.push(task.clone());
                }
                vec![AppCommand::SaveTask(task)]
            }
            AppMessage::DeleteTask(id) => {
                if let Some(pos) = self.tasks.iter().position(|t| t.id == id) {
                    self.tasks.remove(pos);
                }
                vec![AppCommand::DeleteTask(id)]
            }
            AppMessage::MoveTaskStatus(id, status) => {
                if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id) {
                    t.status = status;
                }
                vec![AppCommand::MoveTaskStatus { id, status }]
            }
            AppMessage::ToggleTaskStatus(id) => {
                if let Some(task) = self.get_task(id) {
                    let next = match task.status {
                        TaskStatus::ToDo => TaskStatus::InProgress,
                        TaskStatus::InProgress => TaskStatus::Complete,
                        TaskStatus::Complete => TaskStatus::ToDo,
                    };
                    if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id) {
                        t.status = next;
                    }
                    vec![AppCommand::MoveTaskStatus { id, status: next }]
                } else {
                    Vec::new()
                }
            }
            AppMessage::CreateContact(contact) => {
                vec![AppCommand::SaveContact(contact)]
            }
            AppMessage::UpdateContact(contact) => {
                vec![AppCommand::SaveContact(contact)]
            }
            AppMessage::DeleteContact(id) => {
                vec![AppCommand::DeleteContact(id)]
            }
            AppMessage::LinkContactToArticle {
                article_id,
                contact_id,
            } => {
                let mut current = self
                    .article_contacts
                    .get(&article_id)
                    .cloned()
                    .unwrap_or_default();
                if !current.contains(&contact_id) {
                    current.push(contact_id);
                    vec![AppCommand::SetArticleContacts {
                        article_id,
                        contact_ids: current,
                    }]
                } else {
                    Vec::new()
                }
            }
            AppMessage::UnlinkContactFromArticle {
                article_id,
                contact_id,
            } => {
                let mut current = self
                    .article_contacts
                    .get(&article_id)
                    .cloned()
                    .unwrap_or_default();
                if current.contains(&contact_id) {
                    current.retain(|&id| id != contact_id);
                    vec![AppCommand::SetArticleContacts {
                        article_id,
                        contact_ids: current,
                    }]
                } else {
                    Vec::new()
                }
            }
            AppMessage::SetArticleContacts {
                article_id,
                contact_ids,
            } => {
                vec![AppCommand::SetArticleContacts {
                    article_id,
                    contact_ids,
                }]
            }
            AppMessage::SaveSettings(settings) => {
                self.settings = settings.clone();
                self.theme_engine.set_mode(settings.theme_mode);
                vec![AppCommand::SaveSettings(settings)]
            }
            AppMessage::SetThemeMode(theme_mode) => {
                self.settings.theme_mode = theme_mode;
                self.theme_engine.set_mode(theme_mode);
                vec![AppCommand::SaveSettings(self.settings.clone())]
            }
            AppMessage::ToggleTheme => {
                let next_mode = self.theme_engine.toggle_mode();
                self.settings.theme_mode = next_mode;
                vec![AppCommand::SaveSettings(self.settings.clone())]
            }
            AppMessage::SystemThemeChanged(is_dark) => {
                self.theme_engine.set_system_is_dark(is_dark);
                Vec::new()
            }
            AppMessage::SetHighContrast(high_contrast) => {
                self.theme_engine.set_high_contrast(high_contrast);
                Vec::new()
            }
            AppMessage::SetCustomAccent(accent) => {
                self.theme_engine.set_custom_accent(accent);
                Vec::new()
            }
            AppMessage::ResetSettingsToDefaults => {
                let default_settings = newsjournal_core::models::Settings::default();
                self.settings = default_settings.clone();
                self.theme_engine.set_mode(default_settings.theme_mode);
                self.theme_engine.set_high_contrast(false);
                self.theme_engine.set_custom_accent(None);
                vec![
                    AppCommand::SaveSettings(default_settings),
                    AppCommand::EmitToast(ToastMessage::info(
                        "Settings Reset",
                        "Preferences restored to default values",
                    )),
                ]
            }

            // ==========================================
            // Drag & Drop Interactions
            // ==========================================
            AppMessage::DragStart(item) => {
                self.drag.start_drag(item);
                Vec::new()
            }
            AppMessage::DragStartWithPos { item, pos } => {
                self.drag.start_drag_with_position(item, pos);
                Vec::new()
            }
            AppMessage::DragMove { pointer_pos } => {
                self.drag.update_pointer_position(pointer_pos);
                Vec::new()
            }
            AppMessage::DragHover(target) => {
                self.drag.update_hover(target);
                Vec::new()
            }
            AppMessage::DragHoverWithIndex {
                target,
                insert_index,
            } => {
                self.drag.update_hover_with_index(target, insert_index);
                Vec::new()
            }
            AppMessage::DragDrop => {
                if let Some((item, target)) = self.drag.complete_drop() {
                    match (item, target) {
                        (
                            DragItem::ArticleCard { id, .. },
                            DropTarget::ArticleColumn(new_stage),
                        ) => {
                            let mut slug_name = "story".to_string();
                            if let Some(a) = self.articles.iter_mut().find(|a| a.id == id) {
                                a.stage = new_stage;
                                slug_name = a.slug.clone();
                            }
                            self.recalculate_deadlines(Utc::now());
                            vec![
                                AppCommand::MoveArticleStage {
                                    id,
                                    stage: new_stage,
                                },
                                AppCommand::EmitToast(ToastMessage::info(
                                    "Article Moved",
                                    format!(
                                        "Story '#{slug_name}' moved to {}",
                                        new_stage.display_name()
                                    ),
                                )),
                            ]
                        }
                        (DragItem::TaskCard { id, .. }, DropTarget::TaskColumn(new_status)) => {
                            let mut task_title = "task".to_string();
                            if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id) {
                                t.status = new_status;
                                task_title = t.title.clone();
                            }
                            vec![
                                AppCommand::MoveTaskStatus {
                                    id,
                                    status: new_status,
                                },
                                AppCommand::EmitToast(ToastMessage::info(
                                    "Task Moved",
                                    format!(
                                        "Task '{task_title}' moved to {}",
                                        new_status.display_name()
                                    ),
                                )),
                            ]
                        }
                        _ => Vec::new(),
                    }
                } else {
                    self.drag.cancel();
                    Vec::new()
                }
            }
            AppMessage::DragCancel => {
                self.drag.cancel();
                Vec::new()
            }

            // ==========================================
            // Search & Filtering
            // ==========================================
            AppMessage::SetSearchQuery(q) => {
                self.filters.set_search(q);
                Vec::new()
            }
            AppMessage::SetUrgencyFilter(u) => {
                self.filters.set_urgency(u);
                Vec::new()
            }
            AppMessage::SetStageFilter(s) => {
                self.filters.set_stage(s);
                Vec::new()
            }
            AppMessage::SetContactFilter(c) => {
                self.filters.selected_contact_id = c;
                Vec::new()
            }
            AppMessage::SetArticleFilter(a) => {
                self.filters.selected_article_id = a;
                Vec::new()
            }
            AppMessage::SetContactSort(sort) => {
                self.filters.set_contact_sort(sort);
                Vec::new()
            }
            AppMessage::SetContactSortField(field) => {
                self.filters.contact_sort.field = field;
                Vec::new()
            }
            AppMessage::SetContactSortDirection(direction) => {
                self.filters.contact_sort.direction = direction;
                Vec::new()
            }
            AppMessage::ToggleContactSort(field) => {
                self.filters.toggle_contact_sort(field);
                Vec::new()
            }
            AppMessage::ResetContactSort => {
                self.filters.reset_contact_sort();
                Vec::new()
            }
            AppMessage::ClearFilters => {
                self.filters.clear();
                Vec::new()
            }

            // ==========================================
            // System, Ticker & Notifications
            // ==========================================
            AppMessage::Tick(now) => {
                self.recalculate_deadlines(now);
                self.purge_expired_toasts(now);
                Vec::new()
            }
            AppMessage::Refresh => {
                if let Err(e) = self.load_all() {
                    self.set_error(Some(format!("Failed to refresh data: {e}")));
                }
                Vec::new()
            }
            AppMessage::PushToast(toast) => {
                self.push_toast(toast);
                Vec::new()
            }
            AppMessage::DismissToast(id) => {
                self.dismiss_toast(id);
                Vec::new()
            }
            AppMessage::SetError(err) => {
                self.set_error(err);
                Vec::new()
            }
            AppMessage::WindowResized { .. } => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ResolvedTheme;
    use newsjournal_core::models::ThemeMode;

    #[test]
    fn test_theme_mode_reducer_actions() {
        let mut state = AppState::in_memory().expect("in-memory state");
        assert_eq!(state.settings.theme_mode, ThemeMode::System);
        assert_eq!(state.resolved_theme(), ResolvedTheme::Dark);

        // Set explicit Light mode
        let commands = state.update(AppMessage::SetThemeMode(ThemeMode::Light));
        assert_eq!(commands.len(), 1);
        assert_eq!(state.settings.theme_mode, ThemeMode::Light);
        assert_eq!(state.resolved_theme(), ResolvedTheme::Light);

        // Toggle theme (Light -> Dark)
        let commands = state.update(AppMessage::ToggleTheme);
        assert_eq!(commands.len(), 1);
        assert_eq!(state.settings.theme_mode, ThemeMode::Dark);
        assert_eq!(state.resolved_theme(), ResolvedTheme::Dark);

        // Set to System mode
        state.update(AppMessage::SetThemeMode(ThemeMode::System));
        assert_eq!(state.settings.theme_mode, ThemeMode::System);

        // OS changes to Light appearance
        let commands = state.update(AppMessage::SystemThemeChanged(false));
        assert!(commands.is_empty());
        assert_eq!(state.settings.theme_mode, ThemeMode::System);
        assert_eq!(state.resolved_theme(), ResolvedTheme::Light);

        // High contrast toggle
        state.update(AppMessage::SetHighContrast(true));
        assert!(state.theme_engine.high_contrast);

        // Custom accent
        let orange = (248, 152, 32);
        state.update(AppMessage::SetCustomAccent(Some(orange)));
        assert_eq!(state.theme_engine.colors().accent, orange);
    }
}
