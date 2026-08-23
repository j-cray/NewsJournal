//! Interactive GUI runner for NewsJournal desktop application using `iced`.

use iced::widget::{
    button, checkbox, column, container, row, scrollable, space, text, text_input, Space,
};
use iced::{event, keyboard, Alignment, Element, Length, Subscription, Task, Theme};
use newsjournal_core::models::{ArticleStage, TaskStatus, ThemeMode};

use crate::message::AppMessage;
use crate::navigation::NavTab;
use crate::runtime::EventLoop;
use crate::state::modal::ModalState;
use crate::state::AppState;
use crate::theme::ResolvedTheme;

/// NewsJournal interactive desktop application wrapper for Iced.
pub struct NewsJournalApp {
    /// Underlying event loop and application state.
    pub event_loop: EventLoop,
}

impl NewsJournalApp {
    /// Create a new application instance with the given initial state.
    #[must_use]
    pub fn new(state: AppState) -> Self {
        Self {
            event_loop: EventLoop::new(state),
        }
    }

    /// Access the underlying state.
    #[must_use]
    pub fn state(&self) -> &AppState {
        self.event_loop.state()
    }

    /// Update application state upon receiving a message.
    pub fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        if let Err(e) = self.event_loop.dispatch(message) {
            eprintln!("Error dispatching message: {e}");
        }
        Task::none()
    }

    /// Resolves the current theme for Iced rendering.
    #[must_use]
    pub fn theme(&self) -> Theme {
        match self.state().resolved_theme() {
            ResolvedTheme::Dark => Theme::Dark,
            ResolvedTheme::Light => Theme::Light,
        }
    }

    /// Subscriptions for periodic deadline ticks and keyboard shortcuts.
    pub fn subscription(&self) -> Subscription<AppMessage> {
        let timer = iced::time::every(std::time::Duration::from_secs(60))
            .map(|_| AppMessage::Tick(chrono::Utc::now()));

        let keyboard_events = event::listen_with(|event, _status, _window| {
            if let event::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) =
                event
            {
                if key == keyboard::Key::Named(keyboard::key::Named::Escape) {
                    return Some(AppMessage::CloseModal);
                }
                if (modifiers.control() || modifiers.command())
                    && key == keyboard::Key::Character("s".into())
                {
                    return Some(AppMessage::SubmitModal);
                }
            }
            None
        });

        Subscription::batch(vec![timer, keyboard_events])
    }

    /// Render the full user interface view tree.
    pub fn view(&self) -> Element<'_, AppMessage> {
        let header = self.view_header();
        let sidebar = self.view_sidebar();
        let main_content = self.view_main_content();

        let body = row![sidebar, main_content].spacing(0);
        let layout = column![header, body].spacing(0);

        if self.state().modal.is_open() {
            let modal_overlay = self.view_modal();
            container(column![layout, modal_overlay]).into()
        } else {
            container(layout).into()
        }
    }

    fn view_header(&self) -> Element<'_, AppMessage> {
        let title = text("📰 NewsJournal").size(20);
        let search = text_input(
            "Search articles, tasks, contacts...",
            &self.state().filters.search_query,
        )
        .on_input(AppMessage::SetSearchQuery)
        .padding(8)
        .width(300);

        let new_article_btn = button("+ New Story")
            .on_press(AppMessage::OpenNewArticleModal)
            .padding([6, 12]);
        let new_task_btn = button("+ New Task")
            .on_press(AppMessage::OpenNewTaskModal(None))
            .padding([6, 12]);
        let new_contact_btn = button("+ New Contact")
            .on_press(AppMessage::OpenNewContactModal)
            .padding([6, 12]);

        let theme_icon = if self.state().resolved_theme() == ResolvedTheme::Dark {
            "☀️ Light"
        } else {
            "🌙 Dark"
        };
        let theme_btn = button(theme_icon)
            .on_press(AppMessage::ToggleTheme)
            .padding([6, 12]);

        let overdue_badge: Element<'_, AppMessage> =
            if self.state().deadline_summary.overdue_count > 0 {
                text(format!(
                    "⚠️ {} Overdue",
                    self.state().deadline_summary.overdue_count
                ))
                .size(14)
                .into()
            } else {
                space::horizontal().into()
            };

        let bar = row![
            title,
            space::horizontal(),
            search,
            overdue_badge,
            new_article_btn,
            new_task_btn,
            new_contact_btn,
            theme_btn,
        ]
        .spacing(12)
        .align_y(Alignment::Center)
        .padding(12);

        container(bar).width(Length::Fill).into()
    }

    fn view_sidebar(&self) -> Element<'_, AppMessage> {
        let active_tab = self.state().active_tab;

        let make_nav_btn = |tab: NavTab, label: &'static str| {
            let is_active = active_tab == tab;
            let mut btn = button(text(label).size(15))
                .on_press(AppMessage::NavigateTo(tab))
                .width(Length::Fill)
                .padding([10, 16]);
            if is_active {
                btn = btn.style(button::primary);
            } else {
                btn = btn.style(button::secondary);
            }
            btn
        };

        let articles_btn = make_nav_btn(NavTab::ArticlesKanban, "📰 Articles Kanban");
        let tasks_btn = make_nav_btn(NavTab::TasksKanban, "✅ Tasks Kanban");
        let contacts_btn = make_nav_btn(NavTab::ContactsDirectory, "👥 Contacts Directory");
        let settings_btn = make_nav_btn(NavTab::Settings, "⚙️ Settings");

        let sidebar_col = column![
            articles_btn,
            tasks_btn,
            contacts_btn,
            space::vertical(),
            settings_btn,
        ]
        .spacing(8)
        .padding(12)
        .width(220)
        .height(Length::Fill);

        container(sidebar_col)
            .width(220)
            .height(Length::Fill)
            .into()
    }

    fn view_main_content(&self) -> Element<'_, AppMessage> {
        match self.state().active_tab {
            NavTab::ArticlesKanban => self.view_articles_deck(),
            NavTab::TasksKanban => self.view_tasks_deck(),
            NavTab::ContactsDirectory => self.view_contacts_directory(),
            NavTab::Settings => self.view_settings(),
        }
    }

    fn view_articles_deck(&self) -> Element<'_, AppMessage> {
        let stages = ArticleStage::all();

        let mut columns_row = row![].spacing(12).padding(12);

        for &stage in stages {
            let articles = self.state().articles_in_stage(stage);
            let stage_title = stage.display_name();

            let header = row![
                text(format!("{stage_title} ({})", articles.len())).size(16),
                space::horizontal(),
                button("+")
                    .on_press(AppMessage::OpenNewArticleInStageModal(stage))
                    .padding([2, 6]),
            ]
            .align_y(Alignment::Center);

            let mut cards_col = column![].spacing(8);

            for article in articles {
                let card = self.view_article_card(article);
                cards_col = cards_col.push(card);
            }

            let col_content = column![header, scrollable(cards_col).height(Length::Fill),]
                .spacing(8)
                .width(240)
                .height(Length::Fill);

            let col_container = container(col_content)
                .padding(8)
                .width(250)
                .height(Length::Fill);

            columns_row = columns_row.push(col_container);
        }

        scrollable(columns_row)
            .direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::default(),
            ))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_article_card(
        &self,
        article: &newsjournal_core::models::Article,
    ) -> Element<'static, AppMessage> {
        let id = article.id;
        let is_overdue = article.is_overdue(chrono::Utc::now());

        let slug = text(format!("[{}]", article.slug)).size(12);
        let headline = text(article.headline.clone()).size(14);

        let tasks_count = self.state().tasks_for_article(id).len();
        let tasks_complete = self
            .state()
            .tasks_for_article(id)
            .iter()
            .filter(|t| t.status == TaskStatus::Complete)
            .count();
        let tasks_info = text(format!("✓ {tasks_complete}/{tasks_count} tasks")).size(11);

        let mut actions = row![
            button("Edit")
                .on_press(AppMessage::OpenEditArticleModal(id))
                .padding([2, 6]),
            button("🗑")
                .on_press(AppMessage::PromptDeleteArticle(id))
                .padding([2, 6]),
        ]
        .spacing(6);

        if let Some(prev_stage) = article.stage.prev_stage() {
            actions = actions.push(
                button("◀")
                    .on_press(AppMessage::MoveArticleStage(id, prev_stage))
                    .padding([2, 4]),
            );
        }

        if let Some(next_stage) = article.stage.next_stage() {
            actions = actions.push(
                button("▶")
                    .on_press(AppMessage::MoveArticleStage(id, next_stage))
                    .padding([2, 4]),
            );
        }

        let mut card_content = column![slug, headline].spacing(4);
        if let Some(desc) = &article.description {
            if !desc.is_empty() {
                card_content = card_content.push(text(desc.clone()).size(12));
            }
        }
        if is_overdue {
            card_content = card_content.push(text("⚠️ OVERDUE").size(11));
        }
        card_content = card_content.push(tasks_info).push(actions);

        container(card_content)
            .padding(10)
            .width(Length::Fill)
            .into()
    }

    fn view_tasks_deck(&self) -> Element<'static, AppMessage> {
        let statuses = TaskStatus::all();
        let mut row_cols = row![].spacing(16).padding(16);

        for &status in statuses {
            let tasks = self.state().tasks_in_status(status);
            let status_title = status.display_name();

            let header = row![
                text(format!("{status_title} ({})", tasks.len())).size(16),
                space::horizontal(),
                button("+")
                    .on_press(AppMessage::OpenNewTaskInStatusModal(status, None))
                    .padding([2, 6]),
            ];

            let mut tasks_col = column![].spacing(8);
            for task in tasks {
                let id = task.id;
                let title = text(task.title.clone()).size(14);

                let parent_slug = self
                    .state()
                    .get_article(task.article_id)
                    .map(|a| a.slug.clone())
                    .unwrap_or_default();
                let parent_text = text(format!("Story: {parent_slug}")).size(11);

                let check = checkbox(task.status == TaskStatus::Complete)
                    .on_toggle(move |_| AppMessage::ToggleTaskStatus(id));

                let actions = row![
                    button("Edit")
                        .on_press(AppMessage::OpenEditTaskModal(id))
                        .padding([2, 6]),
                    button("🗑")
                        .on_press(AppMessage::PromptDeleteTask(id))
                        .padding([2, 6]),
                ]
                .spacing(6);

                let mut task_content = column![row![check, title].spacing(8), parent_text];
                if let Some(notes) = &task.notes {
                    if !notes.is_empty() {
                        task_content = task_content.push(text(notes.clone()).size(12));
                    }
                }
                task_content = task_content.push(actions);

                let task_card = container(task_content.spacing(4))
                    .padding(8)
                    .width(Length::Fill);

                tasks_col = tasks_col.push(task_card);
            }

            let col_container =
                container(column![header, scrollable(tasks_col).height(Length::Fill)].spacing(8))
                    .padding(8)
                    .width(320)
                    .height(Length::Fill);

            row_cols = row_cols.push(col_container);
        }

        row_cols.width(Length::Fill).height(Length::Fill).into()
    }

    fn view_contacts_directory(&self) -> Element<'static, AppMessage> {
        let contacts = self.state().filtered_contacts();

        let header = row![
            text(format!("Contacts Directory ({} contacts)", contacts.len())).size(18),
            space::horizontal(),
            button("+ Add Contact")
                .on_press(AppMessage::OpenNewContactModal)
                .padding([6, 12]),
        ]
        .align_y(Alignment::Center);

        let mut list_col = column![].spacing(8);

        for contact in contacts {
            let id = contact.id;
            let name = text(contact.name.clone()).size(15);
            let org = contact.organization.as_deref().unwrap_or("-");
            let role = contact.role.as_deref().unwrap_or("-");
            let org_role = text(format!("{org} • {role}")).size(13);

            let email = contact.email.as_deref().unwrap_or("No email");
            let phone = contact.phone.as_deref().unwrap_or("No phone");
            let email_phone = text(format!("📧 {email} | 📞 {phone}")).size(12);

            let tagged_articles_count = self.state().articles_for_contact(id).len();
            let tagged_info = text(format!("Tagged in {tagged_articles_count} stories")).size(11);

            let actions = row![
                button("Edit")
                    .on_press(AppMessage::OpenEditContactModal(id))
                    .padding([4, 8]),
                button("🗑 Delete")
                    .on_press(AppMessage::PromptDeleteContact(id))
                    .padding([4, 8]),
            ]
            .spacing(6);

            let contact_card = container(
                row![
                    column![name, org_role, email_phone, tagged_info].spacing(4),
                    space::horizontal(),
                    actions,
                ]
                .align_y(Alignment::Center),
            )
            .padding(12)
            .width(Length::Fill);

            list_col = list_col.push(contact_card);
        }

        let content = column![header, scrollable(list_col).height(Length::Fill)]
            .spacing(12)
            .padding(16);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_settings(&self) -> Element<'_, AppMessage> {
        let title = text("Settings & Appearance").size(20);

        let theme_label = text("Theme Mode:").size(15);
        let theme_system_btn = button("System (Auto)")
            .on_press(AppMessage::SetThemeMode(ThemeMode::System))
            .padding([6, 12]);
        let theme_light_btn = button("Light Mode")
            .on_press(AppMessage::SetThemeMode(ThemeMode::Light))
            .padding([6, 12]);
        let theme_dark_btn = button("Dark Mode")
            .on_press(AppMessage::SetThemeMode(ThemeMode::Dark))
            .padding([6, 12]);

        let theme_row = row![theme_system_btn, theme_light_btn, theme_dark_btn].spacing(10);

        let db_info = text("Database: SQLite (Embedded)").size(14);
        let articles_count =
            text(format!("Total Articles: {}", self.state().articles.len())).size(13);
        let tasks_count = text(format!("Total Tasks: {}", self.state().tasks.len())).size(13);
        let contacts_count =
            text(format!("Total Contacts: {}", self.state().contacts.len())).size(13);

        let content = column![
            title,
            Space::new().height(12.0),
            theme_label,
            theme_row,
            Space::new().height(20.0),
            db_info,
            articles_count,
            tasks_count,
            contacts_count,
        ]
        .spacing(10)
        .padding(24);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_modal(&self) -> Element<'_, AppMessage> {
        match &self.state().modal {
            ModalState::None => space::horizontal().into(),
            ModalState::ArticleForm(draft) => {
                let title = text(if draft.id.is_some() {
                    "Edit Story"
                } else {
                    "Create Story"
                })
                .size(18);

                let slug_input = text_input("Slug (e.g. metro-transit)", &draft.slug)
                    .on_input(AppMessage::UpdateArticleSlug)
                    .padding(8);
                let headline_input = text_input("Headline", &draft.headline)
                    .on_input(AppMessage::UpdateArticleHeadline)
                    .padding(8);
                let desc_input = text_input("Description / Notes", &draft.description)
                    .on_input(AppMessage::UpdateArticleDescription)
                    .padding(8);

                let actions = row![
                    button("Cancel")
                        .on_press(AppMessage::CloseModal)
                        .padding([6, 14]),
                    space::horizontal(),
                    button("Save Story")
                        .on_press(AppMessage::SubmitModal)
                        .padding([6, 14]),
                ];

                let modal_box = container(
                    column![title, slug_input, headline_input, desc_input, actions].spacing(12),
                )
                .padding(20)
                .width(480);

                container(modal_box)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
            }
            ModalState::TaskForm(draft) => {
                let title = text(if draft.id.is_some() {
                    "Edit Task"
                } else {
                    "Create Task"
                })
                .size(18);
                let title_input = text_input("Task Title", &draft.title)
                    .on_input(AppMessage::UpdateTaskDraftTitle)
                    .padding(8);
                let notes_input = text_input("Task Notes", &draft.notes)
                    .on_input(AppMessage::UpdateTaskDraftNotes)
                    .padding(8);

                let actions = row![
                    button("Cancel")
                        .on_press(AppMessage::CloseModal)
                        .padding([6, 14]),
                    space::horizontal(),
                    button("Save Task")
                        .on_press(AppMessage::SubmitModal)
                        .padding([6, 14]),
                ];

                let modal_box =
                    container(column![title, title_input, notes_input, actions].spacing(12))
                        .padding(20)
                        .width(420);

                container(modal_box)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
            }
            ModalState::ContactForm(draft) => {
                let title = text(if draft.id.is_some() {
                    "Edit Contact"
                } else {
                    "Create Contact"
                })
                .size(18);
                let name_input = text_input("Name", &draft.name)
                    .on_input(AppMessage::UpdateContactDraftName)
                    .padding(8);
                let org_input = text_input("Organization", &draft.organization)
                    .on_input(AppMessage::UpdateContactDraftOrg)
                    .padding(8);
                let role_input = text_input("Role", &draft.role)
                    .on_input(AppMessage::UpdateContactDraftRole)
                    .padding(8);
                let email_input = text_input("Email", &draft.email)
                    .on_input(AppMessage::UpdateContactDraftEmail)
                    .padding(8);
                let phone_input = text_input("Phone", &draft.phone)
                    .on_input(AppMessage::UpdateContactDraftPhone)
                    .padding(8);

                let actions = row![
                    button("Cancel")
                        .on_press(AppMessage::CloseModal)
                        .padding([6, 14]),
                    space::horizontal(),
                    button("Save Contact")
                        .on_press(AppMessage::SubmitModal)
                        .padding([6, 14]),
                ];

                let modal_box = container(
                    column![
                        title,
                        name_input,
                        org_input,
                        role_input,
                        email_input,
                        phone_input,
                        actions
                    ]
                    .spacing(12),
                )
                .padding(20)
                .width(460);

                container(modal_box)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
            }
            ModalState::ConfirmDeleteArticle { id, .. } => {
                let id = *id;
                let title = text("Delete Article").size(18);
                let msg =
                    text("Are you sure you want to delete this article and its associated tasks?")
                        .size(14);
                let actions = row![
                    button("Cancel")
                        .on_press(AppMessage::CloseModal)
                        .padding([6, 14]),
                    space::horizontal(),
                    button("Confirm Delete")
                        .on_press(AppMessage::DeleteArticle(id))
                        .padding([6, 14]),
                ];
                let modal_box = container(column![title, msg, actions].spacing(12))
                    .padding(20)
                    .width(400);
                container(modal_box)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
            }
            ModalState::ConfirmDeleteTask { id, .. } => {
                let id = *id;
                let title = text("Delete Task").size(18);
                let msg = text("Are you sure you want to delete this task?").size(14);
                let actions = row![
                    button("Cancel")
                        .on_press(AppMessage::CloseModal)
                        .padding([6, 14]),
                    space::horizontal(),
                    button("Confirm Delete")
                        .on_press(AppMessage::DeleteTask(id))
                        .padding([6, 14]),
                ];
                let modal_box = container(column![title, msg, actions].spacing(12))
                    .padding(20)
                    .width(400);
                container(modal_box)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
            }
            ModalState::ConfirmDeleteContact { id, .. } => {
                let id = *id;
                let title = text("Delete Contact").size(18);
                let msg = text("Are you sure you want to delete this contact?").size(14);
                let actions = row![
                    button("Cancel")
                        .on_press(AppMessage::CloseModal)
                        .padding([6, 14]),
                    space::horizontal(),
                    button("Confirm Delete")
                        .on_press(AppMessage::DeleteContact(id))
                        .padding([6, 14]),
                ];
                let modal_box = container(column![title, msg, actions].spacing(12))
                    .padding(20)
                    .width(400);
                container(modal_box)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
            }
            ModalState::SettingsDrawer(_) => space::horizontal().into(),
        }
    }
}

/// Runs the NewsJournal desktop application window.
pub fn run_app(state: AppState) -> iced::Result {
    let state_holder = std::sync::Arc::new(std::sync::Mutex::new(Some(state)));
    let boot_fn = move || {
        let s = state_holder
            .lock()
            .unwrap()
            .take()
            .unwrap_or_else(|| AppState::in_memory().expect("in-memory state fallback"));
        (NewsJournalApp::new(s), Task::none())
    };

    iced::application(boot_fn, NewsJournalApp::update, NewsJournalApp::view)
        .title("NewsJournal")
        .window(iced::window::Settings {
            size: iced::Size::new(1280.0, 800.0),
            min_size: Some(iced::Size::new(960.0, 600.0)),
            ..Default::default()
        })
        .theme(NewsJournalApp::theme)
        .subscription(NewsJournalApp::subscription)
        .run()
}
