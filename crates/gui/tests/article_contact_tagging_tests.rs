//! Comprehensive integration tests for Task 6.3: Contact Tagging Sub-Section.
//!
//! Tests:
//! 1. Searchable multi-select pill selector for existing contacts (filter by name, org, role, email).
//! 2. Multi-select tagging, untagging, and toggling with live view model updates.
//! 3. Inline "Create Contact" shortcut directly from the article form (form fields, live validation, save, auto-tag).
//! 4. Reducer state transitions with dedicated contact tagging messages.
//! 5. End-to-end article submission with SQLite persistence of tagged contacts.
//! 6. Platform view tree descriptor integration (COSMIC and macOS).

use newsjournal_core::models::Contact;
use newsjournal_core::storage::StorageService;
use newsjournal_gui::commands::{AppCommand, CommandExecutor};
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::platform::cosmic::CosmicApp;
use newsjournal_gui::platform::macos::MacosApp;
use newsjournal_gui::state::modal::{ArticleDraft, ContactDraft, ModalState};
use newsjournal_gui::state::AppState;
use newsjournal_gui::views::{build_article_form_view, ContactPillViewModel};
use uuid::Uuid;

#[test]
fn test_contact_pill_view_model_and_display_labels() {
    // 1. Full details: name, org, and role
    let mut contact1 = Contact::new("Jane Doe");
    contact1.organization = Some("Daily Gazette".to_string());
    contact1.role = Some("Lead Investigative Reporter".to_string());
    contact1.email = Some("jane.doe@dailygazette.com".to_string());
    contact1.phone = Some("5551234567".to_string());

    let pill1 = ContactPillViewModel::new(&contact1, true);
    assert_eq!(pill1.id, contact1.id);
    assert_eq!(pill1.name, "Jane Doe");
    assert_eq!(pill1.initials, "JD");
    assert_eq!(
        pill1.display_label,
        "Jane Doe (Daily Gazette • Lead Investigative Reporter)"
    );
    assert!(pill1.is_tagged);
    assert_eq!(pill1.email.as_deref(), Some("jane.doe@dailygazette.com"));
    assert_eq!(pill1.phone.as_deref(), Some("5551234567"));
    assert!(!pill1.avatar_color_hex.is_empty());
    assert!(pill1.avatar_text_color == "#FFFFFF" || pill1.avatar_text_color == "#0F172A");

    // 2. Org only
    let mut contact2 = Contact::new("Bob Dylan");
    contact2.organization = Some("Rolling Stone".to_string());
    let pill2 = ContactPillViewModel::new(&contact2, false);
    assert_eq!(pill2.display_label, "Bob Dylan (Rolling Stone)");
    assert_eq!(pill2.initials, "BD");
    assert!(!pill2.is_tagged);

    // 3. Role only
    let mut contact3 = Contact::new("Charlie Parker");
    contact3.role = Some("Music Critic".to_string());
    let pill3 = ContactPillViewModel::new(&contact3, false);
    assert_eq!(pill3.display_label, "Charlie Parker (Music Critic)");
    assert_eq!(pill3.initials, "CP");

    // 4. Name only
    let contact4 = Contact::new("Alice");
    let pill4 = ContactPillViewModel::new(&contact4, false);
    assert_eq!(pill4.display_label, "Alice");
    assert_eq!(pill4.initials, "Al");
}

#[test]
fn test_searchable_multi_select_contact_filtering() {
    let mut state = AppState::in_memory().expect("in-memory state");

    let mut c1 = Contact::new("Alice Smith");
    c1.organization = Some("City Hall Watch".to_string());
    c1.role = Some("Investigative Journalist".to_string());
    c1.email = Some("alice@cityhall.org".to_string());

    let mut c2 = Contact::new("Bob Jones");
    c2.organization = Some("Metro Daily".to_string());
    c2.role = Some("Senior Editor".to_string());
    c2.email = Some("bob@metrodaily.com".to_string());

    let mut c3 = Contact::new("Charlie Davis");
    c3.organization = Some("Tech Wire".to_string());
    c3.role = Some("Silicon Beat".to_string());
    c3.email = Some("charlie@techwire.io".to_string());

    state.contacts = vec![c1.clone(), c2.clone(), c3.clone()];

    let mut draft = ArticleDraft::new();

    // 1. Initial view: no search query, all 3 contacts available, 0 tagged
    let form1 = build_article_form_view(&state, &draft);
    let section1 = &form1.contacts_section;
    assert_eq!(section1.total_system_contacts, 3);
    assert_eq!(section1.total_tagged_count, 0);
    assert_eq!(section1.tagged_contacts.len(), 0);
    assert_eq!(section1.available_contacts.len(), 3);
    assert_eq!(section1.filtered_contacts.len(), 3);
    assert!(!section1.is_search_active);
    assert!(section1.has_matches);
    assert!(section1.empty_state_message.is_none());

    // 2. Search by contact name (case-insensitive "alice")
    draft.set_contact_search("alice");
    let form2 = build_article_form_view(&state, &draft);
    let section2 = &form2.contacts_section;
    assert!(section2.is_search_active);
    assert_eq!(section2.search_query, "alice");
    assert_eq!(section2.matched_contacts_count, 1);
    assert_eq!(section2.available_contacts.len(), 1);
    assert_eq!(section2.available_contacts[0].name, "Alice Smith");

    // 3. Search by organization name ("Metro")
    draft.set_contact_search("METRO");
    let form3 = build_article_form_view(&state, &draft);
    let section3 = &form3.contacts_section;
    assert_eq!(section3.matched_contacts_count, 1);
    assert_eq!(section3.available_contacts[0].name, "Bob Jones");

    // 4. Search by role ("silicon")
    draft.set_contact_search("silicon");
    let form4 = build_article_form_view(&state, &draft);
    let section4 = &form4.contacts_section;
    assert_eq!(section4.matched_contacts_count, 1);
    assert_eq!(section4.available_contacts[0].name, "Charlie Davis");

    // 5. Search by email domain ("techwire.io")
    draft.set_contact_search("techwire.io");
    let form5 = build_article_form_view(&state, &draft);
    let section5 = &form5.contacts_section;
    assert_eq!(section5.matched_contacts_count, 1);
    assert_eq!(section5.available_contacts[0].name, "Charlie Davis");

    // 6. Search with no matches ("XYZ999")
    draft.set_contact_search("XYZ999");
    let form6 = build_article_form_view(&state, &draft);
    let section6 = &form6.contacts_section;
    assert_eq!(section6.matched_contacts_count, 0);
    assert_eq!(section6.available_contacts.len(), 0);
    assert!(!section6.has_matches);
    assert!(section6.empty_state_message.is_some());
    assert!(section6
        .empty_state_message
        .as_ref()
        .unwrap()
        .contains("No contacts match 'xyz999'"));

    // 7. Clear search query
    draft.clear_contact_search();
    let form7 = build_article_form_view(&state, &draft);
    assert!(!form7.contacts_section.is_search_active);
    assert_eq!(form7.contacts_section.available_contacts.len(), 3);
}

#[test]
fn test_multi_select_tagging_and_untagging() {
    let mut state = AppState::in_memory().expect("in-memory state");

    let c1 = Contact::new("Alice Smith");
    let c2 = Contact::new("Bob Jones");
    let c3 = Contact::new("Charlie Davis");
    state.contacts = vec![c1.clone(), c2.clone(), c3.clone()];

    let mut draft = ArticleDraft::new();

    // 1. Tag c1
    draft.tag_contact(c1.id);
    assert!(draft.is_contact_tagged(c1.id));
    assert!(!draft.is_contact_tagged(c2.id));
    assert_eq!(draft.tagged_contact_ids, vec![c1.id]);

    let form1 = build_article_form_view(&state, &draft);
    assert_eq!(form1.contacts_section.total_tagged_count, 1);
    assert_eq!(form1.contacts_section.tagged_contacts.len(), 1);
    assert_eq!(form1.contacts_section.tagged_contacts[0].id, c1.id);
    assert!(form1.contacts_section.tagged_contacts[0].is_tagged);
    assert_eq!(form1.contacts_section.available_contacts.len(), 2);
    assert_eq!(form1.tagged_contacts_count, 1);

    // 2. Tag c2 (multi-select)
    draft.tag_contact(c2.id);
    assert_eq!(draft.tagged_contact_ids.len(), 2);

    let form2 = build_article_form_view(&state, &draft);
    assert_eq!(form2.contacts_section.total_tagged_count, 2);
    assert_eq!(form2.contacts_section.tagged_contacts.len(), 2);
    assert_eq!(form2.contacts_section.available_contacts.len(), 1);
    assert_eq!(form2.contacts_section.available_contacts[0].id, c3.id);

    // 3. Toggle c2 (un-tags c2)
    draft.toggle_contact(c2.id);
    assert!(!draft.is_contact_tagged(c2.id));
    assert_eq!(draft.tagged_contact_ids, vec![c1.id]);

    let form3 = build_article_form_view(&state, &draft);
    assert_eq!(form3.contacts_section.total_tagged_count, 1);
    assert_eq!(form3.contacts_section.available_contacts.len(), 2);

    // 4. Toggle c3 (tags c3)
    draft.toggle_contact(c3.id);
    assert!(draft.is_contact_tagged(c3.id));
    assert_eq!(draft.tagged_contact_ids, vec![c1.id, c3.id]);

    // 5. Untag c1
    draft.untag_contact(c1.id);
    assert!(!draft.is_contact_tagged(c1.id));
    assert_eq!(draft.tagged_contact_ids, vec![c3.id]);

    // 6. Tagging an already tagged contact is idempotent
    draft.tag_contact(c3.id);
    assert_eq!(draft.tagged_contact_ids, vec![c3.id]);
}

#[test]
fn test_empty_contacts_directory_state() {
    let state = AppState::in_memory().expect("in-memory state");
    let draft = ArticleDraft::new();

    let form = build_article_form_view(&state, &draft);
    let section = &form.contacts_section;
    assert_eq!(section.total_system_contacts, 0);
    assert_eq!(section.total_tagged_count, 0);
    assert_eq!(section.available_contacts.len(), 0);
    assert_eq!(section.tagged_contacts.len(), 0);
    assert!(section.empty_state_message.is_some());
    assert!(section
        .empty_state_message
        .as_ref()
        .unwrap()
        .contains("No contacts in directory yet"));
}

#[test]
fn test_inline_contact_creation_lifecycle_and_validation() {
    let mut draft = ArticleDraft::new();
    assert!(!draft.is_inline_contact_open());
    assert!(draft.inline_contact.is_none());

    // 1. Open inline contact form
    draft.open_inline_contact();
    assert!(draft.is_inline_contact_open());
    assert!(draft.inline_contact.is_some());

    // 2. Validate empty inline contact (name is required)
    let state = AppState::in_memory().expect("in-memory state");
    let form1 = build_article_form_view(&state, &draft);
    assert!(form1.contacts_section.is_inline_contact_open);
    let inline1 = form1.contacts_section.inline_form.as_ref().unwrap();
    assert_eq!(inline1.name_value, "");
    assert!(!inline1.is_valid);

    // 3. Set name
    draft.set_inline_contact_name("Dr. Eleanor Vance");
    let form2 = build_article_form_view(&state, &draft);
    let inline2 = form2.contacts_section.inline_form.as_ref().unwrap();
    assert_eq!(inline2.name_value, "Dr. Eleanor Vance");
    assert!(inline2.is_valid);
    assert!(inline2.name_error.is_none());

    // 4. Set invalid email
    draft.set_inline_contact_email("not-an-email");
    let form3 = build_article_form_view(&state, &draft);
    let inline3 = form3.contacts_section.inline_form.as_ref().unwrap();
    assert!(!inline3.is_valid);
    assert!(inline3.email_error.is_some());

    // 5. Correct email
    draft.set_inline_contact_email("eleanor@vance-lab.org");
    let form4 = build_article_form_view(&state, &draft);
    let inline4 = form4.contacts_section.inline_form.as_ref().unwrap();
    assert!(inline4.is_valid);
    assert!(inline4.email_error.is_none());

    // 6. Set invalid phone
    draft.set_inline_contact_phone("123");
    let form5 = build_article_form_view(&state, &draft);
    let inline5 = form5.contacts_section.inline_form.as_ref().unwrap();
    assert!(!inline5.is_valid);
    assert!(inline5.phone_error.is_some());

    // 7. Correct phone and set org, role, notes
    draft.set_inline_contact_phone("555-987-6543");
    draft.set_inline_contact_org("Vance Research Labs");
    draft.set_inline_contact_role("Director of Biotechnology");
    draft.set_inline_contact_notes("Primary whistleblower on water contamination.");

    let form6 = build_article_form_view(&state, &draft);
    let inline6 = form6.contacts_section.inline_form.as_ref().unwrap();
    assert!(inline6.is_valid);
    assert_eq!(inline6.organization_value, "Vance Research Labs");
    assert_eq!(inline6.role_value, "Director of Biotechnology");
    assert_eq!(
        inline6.notes_value,
        "Primary whistleblower on water contamination."
    );

    // 8. Close inline contact form
    draft.close_inline_contact();
    assert!(!draft.is_inline_contact_open());
    assert!(draft.inline_contact.is_none());

    // 9. Toggle inline contact form
    draft.toggle_inline_contact();
    assert!(draft.is_inline_contact_open());
    draft.toggle_inline_contact();
    assert!(!draft.is_inline_contact_open());
}

#[test]
fn test_reducer_contact_tagging_messages_and_inline_save() {
    let mut state = AppState::in_memory().expect("in-memory state");

    let c1 = Contact::new("Alice Smith");
    let c1_id = c1.id;
    state.contacts = vec![c1.clone()];

    // Open article form
    state.update(AppMessage::OpenNewArticleModal);
    assert!(matches!(state.modal, ModalState::ArticleForm(_)));

    // 1. Toggle contact via message
    state.update(AppMessage::ToggleArticleDraftContact(c1_id));
    if let ModalState::ArticleForm(ref d) = state.modal {
        assert!(d.is_contact_tagged(c1_id));
        assert_eq!(d.tagged_contact_ids, vec![c1_id]);
    } else {
        panic!("expected ArticleForm modal");
    }

    // 2. Remove contact via message
    state.update(AppMessage::RemoveArticleDraftContact(c1_id));
    if let ModalState::ArticleForm(ref d) = state.modal {
        assert!(!d.is_contact_tagged(c1_id));
        assert!(d.tagged_contact_ids.is_empty());
    } else {
        panic!("expected ArticleForm modal");
    }

    // 3. Add contact via message
    state.update(AppMessage::AddArticleDraftContact(c1_id));
    if let ModalState::ArticleForm(ref d) = state.modal {
        assert!(d.is_contact_tagged(c1_id));
    } else {
        panic!("expected ArticleForm modal");
    }

    // 4. Set search filter via message
    state.update(AppMessage::SetArticleDraftContactSearch(
        "smith".to_string(),
    ));
    if let ModalState::ArticleForm(ref d) = state.modal {
        assert_eq!(d.contact_search_query, "smith");
    } else {
        panic!("expected ArticleForm modal");
    }

    // 5. Open inline contact form via message
    state.update(AppMessage::OpenArticleDraftInlineContact);
    if let ModalState::ArticleForm(ref d) = state.modal {
        assert!(d.is_inline_contact_open());
    } else {
        panic!("expected ArticleForm modal");
    }

    // 6. Update inline contact draft via message
    let mut new_contact_draft = ContactDraft::new();
    new_contact_draft.name = "Mayor John Doe".to_string();
    new_contact_draft.organization = "City of Springfield".to_string();
    new_contact_draft.role = "Mayor".to_string();
    new_contact_draft.email = "mayor@springfield.gov".to_string();
    new_contact_draft.validate();

    state.update(AppMessage::UpdateArticleDraftInlineContact(
        new_contact_draft,
    ));

    // 7. Save inline contact via message
    let commands = state.update(AppMessage::SaveArticleDraftInlineContact);
    assert_eq!(commands.len(), 2);

    let saved_contact_id = match &commands[0] {
        AppCommand::SaveContact(contact) => {
            assert_eq!(contact.name, "Mayor John Doe");
            assert_eq!(contact.organization.as_deref(), Some("City of Springfield"));
            assert_eq!(contact.role.as_deref(), Some("Mayor"));
            assert_eq!(contact.email.as_deref(), Some("mayor@springfield.gov"));
            contact.id
        }
        other => panic!("expected SaveContact command, got {other:?}"),
    };

    // Verify toast emitted
    assert!(matches!(&commands[1], AppCommand::EmitToast(_)));

    // Verify state:
    // - newly created contact is added to state.contacts
    assert!(state.contacts.iter().any(|c| c.id == saved_contact_id));

    // - article draft has both c1_id and saved_contact_id tagged!
    if let ModalState::ArticleForm(ref d) = state.modal {
        assert!(!d.is_inline_contact_open());
        assert_eq!(d.contact_search_query, "");
        assert!(d.is_contact_tagged(c1_id));
        assert!(d.is_contact_tagged(saved_contact_id));
        assert_eq!(d.tagged_contact_ids.len(), 2);
    } else {
        panic!("expected ArticleForm modal");
    }
}

#[test]
fn test_end_to_end_article_submission_with_tagged_contacts_persistence() {
    let storage = StorageService::in_memory().expect("in-memory storage");
    let executor = CommandExecutor::new();

    // Create 2 contacts in storage
    let mut c1 = Contact::new("Dana Scully");
    c1.organization = Some("FBI".to_string());
    storage.create_contact(c1.clone()).expect("create c1");

    let mut c2 = Contact::new("Fox Mulder");
    c2.organization = Some("FBI".to_string());
    storage.create_contact(c2.clone()).expect("create c2");

    let mut state = AppState::new(storage.clone());
    state.load_all().expect("load all data");
    assert_eq!(state.contacts.len(), 2);

    // Open article form
    state.update(AppMessage::OpenNewArticleModal);
    state.update(AppMessage::UpdateArticleHeadline(
        "Unexplained Aerial Phenomena Investigation".to_string(),
    ));

    // Tag both contacts
    state.update(AppMessage::AddArticleDraftContact(c1.id));
    state.update(AppMessage::AddArticleDraftContact(c2.id));

    // Submit modal
    let commands = state.update(AppMessage::SubmitModal);
    assert_eq!(commands.len(), 3);

    let mut article_id = Uuid::nil();
    for cmd in commands {
        if let AppCommand::SaveArticle(ref art) = cmd {
            article_id = art.id;
        }
        let follow_up = executor.execute(cmd, &storage).expect("command execution");
        if let Some(msg) = follow_up {
            state.update(msg);
        }
    }

    assert_ne!(article_id, Uuid::nil());

    // Verify in storage: article exists
    let saved_article = storage
        .get_article(article_id)
        .expect("get article")
        .expect("article found");
    assert_eq!(
        saved_article.headline,
        "Unexplained Aerial Phenomena Investigation"
    );

    // Verify in storage: contacts are tagged
    let tagged_contacts = storage
        .list_contacts_for_article(article_id)
        .expect("list contacts for article");
    assert_eq!(tagged_contacts.len(), 2);
    let tagged_ids: Vec<Uuid> = tagged_contacts.into_iter().map(|c| c.id).collect();
    assert!(tagged_ids.contains(&c1.id));
    assert!(tagged_ids.contains(&c2.id));

    // Reload state and verify state caches associations
    state.load_all().expect("reload state");
    let cached_tags = state.article_contacts.get(&article_id).unwrap();
    assert_eq!(cached_tags.len(), 2);
    assert!(cached_tags.contains(&c1.id));
    assert!(cached_tags.contains(&c2.id));

    // Edit article and remove one contact
    state.update(AppMessage::OpenEditArticleModal(article_id));
    state.update(AppMessage::RemoveArticleDraftContact(c1.id));

    let edit_commands = state.update(AppMessage::SubmitModal);
    for cmd in edit_commands {
        let follow_up = executor.execute(cmd, &storage).expect("command execution");
        if let Some(msg) = follow_up {
            state.update(msg);
        }
    }

    // Verify storage now only has c2 tagged
    let updated_tags = storage
        .list_contacts_for_article(article_id)
        .expect("list contacts");
    assert_eq!(updated_tags.len(), 1);
    assert_eq!(updated_tags[0].id, c2.id);
}

#[test]
fn test_platform_view_tree_descriptors_with_contact_tagging() {
    let mut state = AppState::in_memory().expect("in-memory state");

    let contact = Contact::new("Sarah Koenig");
    state.contacts = vec![contact.clone()];

    // 1. Open article modal in CosmicApp
    let mut cosmic_app = CosmicApp::new(state.clone());
    cosmic_app
        .dispatch(AppMessage::OpenNewArticleModal)
        .expect("dispatch");
    cosmic_app
        .dispatch(AppMessage::AddArticleDraftContact(contact.id))
        .expect("dispatch");

    let cosmic_view = cosmic_app.build_view_tree();
    assert!(cosmic_view.modal_container.is_some());
    let modal_container = cosmic_view.modal_container.as_ref().unwrap();
    assert!(modal_container.article_form.is_some());
    let article_form = modal_container.article_form.as_ref().unwrap();
    assert_eq!(article_form.contacts_section.total_tagged_count, 1);
    assert_eq!(article_form.contacts_section.tagged_contacts.len(), 1);
    assert_eq!(
        article_form.contacts_section.tagged_contacts[0].name,
        "Sarah Koenig"
    );

    // Verify JSON serialization includes contacts_section
    let cosmic_json = serde_json::to_string(&cosmic_view).expect("serialize cosmic view");
    assert!(cosmic_json.contains("contacts_section"));
    assert!(cosmic_json.contains("Sarah Koenig"));

    // 2. Open article modal in MacosApp
    let mut macos_app = MacosApp::new(state);
    macos_app
        .dispatch(AppMessage::OpenNewArticleModal)
        .expect("dispatch");
    macos_app
        .dispatch(AppMessage::AddArticleDraftContact(contact.id))
        .expect("dispatch");

    let macos_view = macos_app.build_view_tree();
    assert!(macos_view.modal_container.is_some());
    let macos_container = macos_view.modal_container.as_ref().unwrap();
    assert!(macos_container.article_form.is_some());
    let macos_form = macos_container.article_form.as_ref().unwrap();
    assert_eq!(macos_form.contacts_section.total_tagged_count, 1);

    let macos_json = serde_json::to_string(&macos_view).expect("serialize macos view");
    assert!(macos_json.contains("contacts_section"));
    assert!(macos_json.contains("Sarah Koenig"));
}
