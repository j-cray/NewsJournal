//! Comprehensive integration test suite for Contact Creation & Editing Drawer (Task 8.2).
//!
//! Verifies:
//! 1. "New Contact" modal opens drawer with clean fields and empty state for associated articles.
//! 2. Clicking edit on any contact opens contact detail/edit drawer with pre-populated data.
//! 3. Name (required), Organization, Role, Phone, Email, and Notes field presentation.
//! 4. Live validation tooltips and submission gating for name, email format, and phone number digits.
//! 5. Associated articles list displaying all stories where this contact is tagged (stages, slugs, colors, task counts).
//! 6. Click-through actions from associated stories list to open article editor or filter articles deck.
//! 7. Granular field update messages via application reducer.
//! 8. Save & persistence lifecycle updating in-memory cache and storage commands.
//! 9. Delete action trigger from edit drawer transitioning to confirmation dialog.
//! 10. Platform view tree integration in COSMIC and macOS UI containers.

use newsjournal_core::models::{Article, ArticleStage, Contact, Task};
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::platform::cosmic::CosmicApp;
use newsjournal_gui::platform::macos::MacosApp;
use newsjournal_gui::state::modal::{ContactDraft, ModalState};
use newsjournal_gui::state::AppState;
use newsjournal_gui::views::{
    build_contact_form_view, build_contacts_directory_view, build_modal_container_view,
};

#[test]
fn test_new_contact_opens_contact_drawer() {
    let mut state = AppState::in_memory().expect("in-memory state");
    assert!(!state.modal.is_open());

    // Dispatch OpenNewContactModal
    state.update(AppMessage::OpenNewContactModal);
    assert!(state.modal.is_open());

    match &state.modal {
        ModalState::ContactForm(draft) => {
            assert!(draft.id.is_none());
            assert!(draft.name.is_empty());
            assert!(draft.organization.is_empty());
            assert!(draft.role.is_empty());
            assert!(draft.phone.is_empty());
            assert!(draft.email.is_empty());
            assert!(draft.notes.is_empty());
            assert!(!draft.is_valid());
        }
        other => panic!("Expected ModalState::ContactForm, got {other:?}"),
    }

    let modal_container = build_modal_container_view(&state);
    assert!(modal_container.is_open);
    assert_eq!(modal_container.header.title, "New Contact");
    assert!(modal_container.contact_form.is_some());

    let contact_form = modal_container.contact_form.unwrap();
    assert!(!contact_form.is_edit);
    assert_eq!(contact_form.save_button_label, "Create Contact");
    assert!(!contact_form.can_save); // Name is required
    assert!(!contact_form.header.can_delete);
    assert!(contact_form.header.delete_action.is_none());

    // Associated articles section should be empty
    assert!(contact_form.associated_articles.is_empty);
    assert_eq!(contact_form.associated_articles.count, 0);
    assert_eq!(
        contact_form.associated_articles.count_label,
        "0 stories tagged"
    );
    assert!(contact_form
        .associated_articles
        .empty_guidance
        .contains("Newly created contacts have no tagged stories yet"));
}

#[test]
fn test_click_contact_opens_edit_drawer() {
    let mut state = AppState::in_memory().expect("in-memory state");

    let contact = Contact::builder("Diana Prince")
        .organization("Smithsonian Institution")
        .role("Senior Antiquities Specialist")
        .email("diana.prince@smithsonian.example.org")
        .phone("+1 (555) 300-1984")
        .notes("Direct line for verification on ancient artifacts.")
        .build();
    let cid = contact.id;
    state
        .storage
        .create_contact(contact.clone())
        .expect("save contact");
    state.load_all().expect("reload");

    // Open edit contact drawer from directory view item
    let dir_view = build_contacts_directory_view(&state);
    let contact_vm = dir_view
        .contacts
        .iter()
        .find(|c| c.id == cid)
        .expect("contact vm");
    assert_eq!(
        contact_vm.edit_message,
        AppMessage::OpenEditContactModal(cid)
    );

    state.update(contact_vm.edit_message.clone());
    assert!(state.modal.is_open());

    match &state.modal {
        ModalState::ContactForm(draft) => {
            assert_eq!(draft.id, Some(cid));
            assert_eq!(draft.name, "Diana Prince");
            assert_eq!(draft.organization, "Smithsonian Institution");
            assert_eq!(draft.role, "Senior Antiquities Specialist");
            assert_eq!(draft.email, "diana.prince@smithsonian.example.org");
            assert_eq!(draft.phone, "+1 (555) 300-1984");
            assert_eq!(
                draft.notes,
                "Direct line for verification on ancient artifacts."
            );
            assert!(draft.is_valid());
        }
        other => panic!("Expected ModalState::ContactForm, got {other:?}"),
    }

    let modal_container = build_modal_container_view(&state);
    assert!(modal_container.is_open);
    assert_eq!(modal_container.header.title, "Edit Contact");
    assert!(modal_container.contact_form.is_some());

    let contact_form = modal_container.contact_form.unwrap();
    assert!(contact_form.is_edit);
    assert_eq!(contact_form.save_button_label, "Save Contact");
    assert!(contact_form.can_save);
    assert!(contact_form.header.can_delete);
    assert_eq!(
        contact_form.header.delete_action,
        Some(AppMessage::PromptDeleteContact(cid))
    );
    assert_eq!(contact_form.header.avatar_initials, "DP");
    assert_eq!(contact_form.name_field.value, "Diana Prince");
    assert_eq!(
        contact_form.organization_field.value,
        "Smithsonian Institution"
    );
    assert_eq!(
        contact_form.role_field.value,
        "Senior Antiquities Specialist"
    );
    assert_eq!(contact_form.phone_field.value, "+1 (555) 300-1984");
    assert_eq!(
        contact_form.email_field.value,
        "diana.prince@smithsonian.example.org"
    );
}

#[test]
fn test_contact_validation_rules_and_tooltips() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut draft = ContactDraft::new();

    // 1. Initially empty name
    let form = build_contact_form_view(&state, &draft);
    assert!(!form.can_save);
    assert!(!form.name_field.is_valid);

    // 2. Set whitespace-only name
    draft.set_name("   ");
    let form = build_contact_form_view(&state, &draft);
    assert!(!form.can_save);
    assert!(!form.name_field.is_valid);
    assert!(form.name_field.error.is_some());
    assert_eq!(form.validation_tooltips.len(), 1);
    assert_eq!(form.validation_tooltips[0].field_id, "name");

    // 3. Set valid name
    draft.set_name("Dana Scully");
    let form = build_contact_form_view(&state, &draft);
    assert!(form.can_save);
    assert!(form.name_field.is_valid);
    assert!(form.name_field.error.is_none());
    assert_eq!(form.validation_tooltips.len(), 0);

    // 4. Invalid email format
    draft.set_email("dana.scully@");
    let form = build_contact_form_view(&state, &draft);
    assert!(!form.can_save);
    assert!(!form.email_field.is_valid);
    assert!(form.email_field.error.is_some());
    assert_eq!(form.validation_tooltips.len(), 1);
    assert_eq!(form.validation_tooltips[0].field_id, "email");

    // 5. Invalid phone number (too short)
    draft.set_phone("123");
    let form = build_contact_form_view(&state, &draft);
    assert!(!form.can_save);
    assert!(!form.phone_field.is_valid);
    assert!(form.phone_field.error.is_some());
    assert_eq!(form.validation_tooltips.len(), 2);

    // 6. Fix email and phone
    draft.set_email("dana.scully@fbi.example.gov");
    draft.set_phone("202-555-0199");
    let form = build_contact_form_view(&state, &draft);
    assert!(form.can_save);
    assert!(form.email_field.is_valid);
    assert!(form.phone_field.is_valid);
    assert_eq!(form.validation_tooltips.len(), 0);
}

#[test]
fn test_contact_associated_articles_listing_and_interactions() {
    let mut state = AppState::in_memory().expect("in-memory state");

    // Create a contact
    let contact = Contact::builder("Fox Mulder")
        .organization("Federal Bureau of Investigation")
        .role("Special Agent")
        .email("fox.mulder@fbi.example.gov")
        .phone("2025550143")
        .build();
    let cid = contact.id;
    state
        .storage
        .create_contact(contact.clone())
        .expect("save contact");

    // Create 3 stories
    let mut art1 = Article::new("ufo-sightings-probe", "UFO Sightings Probe Expanded");
    art1.stage = ArticleStage::Writing;
    let art1_id = art1.id;
    state.storage.create_article(art1).expect("art1");

    let mut art2 = Article::new(
        "black-budget-audit",
        "Black Budget Audit Unveils Hidden R&D",
    );
    art2.stage = ArticleStage::Researching;
    let art2_id = art2.id;
    state.storage.create_article(art2).expect("art2");

    let mut art3 = Article::new("arctic-anomaly", "Arctic Station Anomaly Report");
    art3.stage = ArticleStage::Published;
    let art3_id = art3.id;
    state.storage.create_article(art3).expect("art3");

    // Add tasks to art1
    let t1 = Task::new(art1_id, "Analyze radar telemetry");
    let t2 = Task::new(art1_id, "Interview air traffic controllers");
    state.storage.create_task(t1).expect("t1");
    state.storage.create_task(t2).expect("t2");

    // Link contact to art1, art2, art3
    state
        .storage
        .link_contact_to_article(art1_id, cid)
        .expect("tag art1");
    state
        .storage
        .link_contact_to_article(art2_id, cid)
        .expect("tag art2");
    state
        .storage
        .link_contact_to_article(art3_id, cid)
        .expect("tag art3");

    state.load_all().expect("reload");

    let draft = ContactDraft::from_contact(&contact);
    let form = build_contact_form_view(&state, &draft);

    assert!(!form.associated_articles.is_empty);
    assert_eq!(form.associated_articles.count, 3);
    assert_eq!(form.associated_articles.count_label, "3 stories tagged");
    assert_eq!(form.associated_articles.items.len(), 3);

    let art1_item = form
        .associated_articles
        .items
        .iter()
        .find(|i| i.article_id == art1_id)
        .expect("art1 item");
    assert_eq!(art1_item.slug, "ufo-sightings-probe");
    assert_eq!(art1_item.display_slug, "#ufo-sightings-probe");
    assert_eq!(art1_item.stage, ArticleStage::Writing);
    assert_eq!(art1_item.stage_title, "Writing");
    assert_eq!(art1_item.task_summary_label, "0/2 tasks");
    assert_eq!(
        art1_item.open_article_message,
        AppMessage::OpenEditArticleModal(art1_id)
    );
    assert_eq!(
        art1_item.filter_message,
        AppMessage::SetContactFilter(Some(cid))
    );

    let art3_item = form
        .associated_articles
        .items
        .iter()
        .find(|i| i.article_id == art3_id)
        .expect("art3 item");
    assert_eq!(art3_item.stage, ArticleStage::Published);
    assert_eq!(art3_item.stage_title, "Published");
}

#[test]
fn test_contact_draft_granular_field_updates_via_reducer() {
    let mut state = AppState::in_memory().expect("in-memory state");
    state.update(AppMessage::OpenNewContactModal);

    // Update fields granularly
    state.update(AppMessage::UpdateContactDraftName(
        "Clarice Starling".to_string(),
    ));
    state.update(AppMessage::UpdateContactDraftOrg(
        "FBI Behavioral Science Unit".to_string(),
    ));
    state.update(AppMessage::UpdateContactDraftRole(
        "Special Agent Trainee".to_string(),
    ));
    state.update(AppMessage::UpdateContactDraftPhone(
        "202-555-0188".to_string(),
    ));
    state.update(AppMessage::UpdateContactDraftEmail(
        "clarice.starling@fbi.example.gov".to_string(),
    ));
    state.update(AppMessage::UpdateContactDraftNotes(
        "Top of class at Quantico.".to_string(),
    ));

    match &state.modal {
        ModalState::ContactForm(draft) => {
            assert_eq!(draft.name, "Clarice Starling");
            assert_eq!(draft.organization, "FBI Behavioral Science Unit");
            assert_eq!(draft.role, "Special Agent Trainee");
            assert_eq!(draft.phone, "202-555-0188");
            assert_eq!(draft.email, "clarice.starling@fbi.example.gov");
            assert_eq!(draft.notes, "Top of class at Quantico.");
            assert!(draft.is_valid());
        }
        other => panic!("Expected ModalState::ContactForm, got {other:?}"),
    }

    let form = build_contact_form_view(
        &state,
        match &state.modal {
            ModalState::ContactForm(d) => d,
            _ => unreachable!(),
        },
    );
    assert!(form.can_save);
    assert_eq!(form.name_field.value, "Clarice Starling");
    assert_eq!(form.phone_field.formatted_display, "(202) 555-0188");
}

#[test]
fn test_contact_save_and_persistence_lifecycle() {
    let mut state = AppState::in_memory().expect("in-memory state");

    // 1. Create a new contact through the drawer
    state.update(AppMessage::OpenNewContactModal);
    state.update(AppMessage::UpdateContactDraftName(
        "Walter Skinner".to_string(),
    ));
    state.update(AppMessage::UpdateContactDraftOrg(
        "Federal Bureau of Investigation".to_string(),
    ));
    state.update(AppMessage::UpdateContactDraftRole(
        "Assistant Director".to_string(),
    ));
    state.update(AppMessage::UpdateContactDraftEmail(
        "walter.skinner@fbi.example.gov".to_string(),
    ));

    assert!(state.modal.is_open());
    state.update(AppMessage::SubmitModal);

    // Modal should now be closed
    assert!(!state.modal.is_open());

    // Contact should exist in in-memory state
    assert_eq!(state.contacts.len(), 1);
    let saved = &state.contacts[0];
    assert_eq!(saved.name, "Walter Skinner");
    assert_eq!(
        saved.organization.as_deref(),
        Some("Federal Bureau of Investigation")
    );
    assert_eq!(saved.role.as_deref(), Some("Assistant Director"));
    let saved_id = saved.id;

    // 2. Re-open edit drawer and modify role
    state.update(AppMessage::OpenEditContactModal(saved_id));
    assert!(state.modal.is_open());
    state.update(AppMessage::UpdateContactDraftRole(
        "Deputy Director".to_string(),
    ));
    state.update(AppMessage::SubmitModal);

    assert!(!state.modal.is_open());
    let updated = state
        .contacts
        .iter()
        .find(|c| c.id == saved_id)
        .expect("updated contact");
    assert_eq!(updated.role.as_deref(), Some("Deputy Director"));
}

#[test]
fn test_contact_delete_action_from_edit_drawer() {
    let mut state = AppState::in_memory().expect("in-memory state");

    let contact = Contact::builder("Deep Throat")
        .organization("Executive Branch")
        .role("Anonymous Source")
        .build();
    let cid = contact.id;
    state
        .storage
        .create_contact(contact.clone())
        .expect("save contact");

    let art = Article::new("watergate-followup", "Watergate Follow-up Investigation");
    let aid = art.id;
    state.storage.create_article(art).expect("save art");
    state
        .storage
        .link_contact_to_article(aid, cid)
        .expect("link contact");
    state.load_all().expect("reload");

    // Open edit drawer
    state.update(AppMessage::OpenEditContactModal(cid));

    let form = match &state.modal {
        ModalState::ContactForm(d) => build_contact_form_view(&state, d),
        _ => panic!("Expected ContactForm"),
    };

    assert!(form.header.can_delete);
    assert_eq!(
        form.header.delete_action,
        Some(AppMessage::PromptDeleteContact(cid))
    );

    // Trigger delete prompt
    state.update(form.header.delete_action.unwrap());
    match &state.modal {
        ModalState::ConfirmDeleteContact {
            id,
            name,
            linked_article_count,
        } => {
            assert_eq!(*id, cid);
            assert_eq!(name, "Deep Throat");
            assert_eq!(*linked_article_count, 1);
        }
        other => panic!("Expected ConfirmDeleteContact, got {other:?}"),
    }
}

#[test]
fn test_cosmic_and_macos_app_contact_drawer_view_tree_integration() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let contact = Contact::builder("Gillian Anderson")
        .organization("BBC / Bad Wolf")
        .role("Lead Actor / Narrator")
        .email("gillian@badwolf.example.co.uk")
        .build();
    state.storage.create_contact(contact.clone()).expect("save");
    state.load_all().expect("reload");

    state.update(AppMessage::OpenEditContactModal(contact.id));

    // 1. COSMIC view tree
    let cosmic_app = CosmicApp::new(state.clone());
    let cosmic_tree = cosmic_app.build_view_tree();
    assert!(cosmic_tree.modal_container.is_some());
    let cosmic_modal = cosmic_tree.modal_container.unwrap();
    assert!(cosmic_modal.is_open);
    assert_eq!(cosmic_modal.header.title, "Edit Contact");
    assert!(cosmic_modal.contact_form.is_some());
    let cf_cosmic = cosmic_modal.contact_form.unwrap();
    assert_eq!(cf_cosmic.name_field.value, "Gillian Anderson");
    assert_eq!(cf_cosmic.save_shortcut, "Ctrl+S");

    // 2. macOS view tree
    let macos_app = MacosApp::new(state);
    let macos_tree = macos_app.build_view_tree();
    assert!(macos_tree.modal_container.is_some());
    let macos_modal = macos_tree.modal_container.unwrap();
    assert!(macos_modal.is_open);
    assert_eq!(macos_modal.header.title, "Edit Contact");
    assert!(macos_modal.contact_form.is_some());
    let cf_macos = macos_modal.contact_form.unwrap();
    assert_eq!(cf_macos.name_field.value, "Gillian Anderson");
    assert_eq!(cf_macos.save_shortcut, "⌘S");
}
