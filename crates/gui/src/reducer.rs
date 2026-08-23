//! Pure state transition reducer for the NewsJournal GUI.

use chrono::Utc;
use newsjournal_core::color::assign_color_for_slug;
use newsjournal_core::models::TaskStatus;

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
                let mut draft = ArticleDraft::new();
                let default_color = assign_color_for_slug("new-article");
                draft.color_hex = default_color.to_hex();
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
            AppMessage::UpdateArticleDraft(draft) => {
                self.modal = ModalState::ArticleForm(draft);
                Vec::new()
            }
            AppMessage::UpdateTaskDraft(draft) => {
                self.modal = ModalState::TaskForm(draft);
                Vec::new()
            }
            AppMessage::UpdateContactDraft(draft) => {
                self.modal = ModalState::ContactForm(draft);
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
                        if draft.validate() {
                            match draft.to_article() {
                                Ok(article) => {
                                    let tagged = draft.tagged_contact_ids.clone();
                                    let aid = article.id;
                                    let slug = article.slug.clone();
                                    vec![
                                        AppCommand::SaveArticle(article),
                                        AppCommand::SetArticleContacts {
                                            article_id: aid,
                                            contact_ids: tagged,
                                        },
                                        AppCommand::EmitToast(ToastMessage::success(
                                            "Article Saved",
                                            format!("Story '{slug}' saved successfully"),
                                        )),
                                    ]
                                }
                                Err(err) => {
                                    self.modal = ModalState::ArticleForm(draft);
                                    vec![AppCommand::EmitError(err)]
                                }
                            }
                        } else {
                            self.modal = ModalState::ArticleForm(draft);
                            Vec::new()
                        }
                    }
                    ModalState::TaskForm(mut draft) => {
                        if draft.validate() {
                            match draft.to_task() {
                                Ok(task) => {
                                    let title = task.title.clone();
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
                            self.modal = ModalState::TaskForm(draft);
                            Vec::new()
                        }
                    }
                    ModalState::ContactForm(mut draft) => {
                        if draft.validate() {
                            match draft.to_contact() {
                                Ok(contact) => {
                                    let name = contact.name.clone();
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
                            self.modal = ModalState::ContactForm(draft);
                            Vec::new()
                        }
                    }
                    ModalState::SettingsDrawer(draft) => {
                        let settings = draft.to_settings();
                        self.settings = settings.clone();
                        self.theme_engine.set_mode(settings.theme_mode);
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
                vec![AppCommand::SaveTask(task)]
            }
            AppMessage::UpdateTask(task) => {
                vec![AppCommand::SaveTask(task)]
            }
            AppMessage::DeleteTask(id) => {
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

            // ==========================================
            // Drag & Drop Interactions
            // ==========================================
            AppMessage::DragStart(item) => {
                self.drag.start_drag(item);
                Vec::new()
            }
            AppMessage::DragHover(target) => {
                self.drag.update_hover(target);
                Vec::new()
            }
            AppMessage::DragDrop => {
                if let Some((item, target)) = self.drag.complete_drop() {
                    match (item, target) {
                        (
                            DragItem::ArticleCard { id, .. },
                            DropTarget::ArticleColumn(new_stage),
                        ) => {
                            if let Some(a) = self.articles.iter_mut().find(|a| a.id == id) {
                                a.stage = new_stage;
                            }
                            self.recalculate_deadlines(Utc::now());
                            vec![AppCommand::MoveArticleStage {
                                id,
                                stage: new_stage,
                            }]
                        }
                        (DragItem::TaskCard { id, .. }, DropTarget::TaskColumn(new_status)) => {
                            if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id) {
                                t.status = new_status;
                            }
                            vec![AppCommand::MoveTaskStatus {
                                id,
                                status: new_status,
                            }]
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
