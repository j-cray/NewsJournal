//! Tests for Task 8.4: Contact Deletion & Unlink Safety.
//!
//! Verifies:
//! 1. Deleting unlinked contacts displays clean confirmation dialog without active story warnings.
//! 2. Deleting contacts linked only to published stories displays historical unlinking notes.
//! 3. Deleting contacts linked to active stories displays high-priority active story alerts,
//!    slug listings, and tailored destructive action labels.
//! 4. Modal container view models and `ConfirmationDialogViewModel` correctly resolve
//!    all active story metadata and unlinking consequence descriptions.
//! 5. Keyboard shortcuts (`Enter` to confirm, `Esc` to cancel) properly resolve.
//! 6. Reducer state transitions immediately clean up in-memory contacts, `contact_articles`,
//!    and `article_contacts` maps without corrupting articles.
//! 7. SQLite persistence cascade integrity removes `article_contacts` while preserving stories.
//! 8. Cross-platform view trees (COSMIC Linux and macOS iced) seamlessly integrate the confirmation dialog.

use newsjournal_core::models::{Article, ArticleStage, Contact};
use newsjournal_gui::commands::AppCommand;
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::navigation::{resolve_app_shortcut, NavKeyModifiers};
use newsjournal_gui::platform::cosmic::CosmicApp;
use newsjournal_gui::platform::macos::MacosApp;
use newsjournal_gui::state::modal::ModalState;
use newsjournal_gui::state::AppState;
use newsjournal_gui::views::modal::{
    build_modal_container_view, build_modal_container_view_with_layout, DeleteEntityType,
    ModalPlacement,
};

#[test]
fn test_unlinked_contact_deletion_prompt_and_view_model() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let contact = Contact::builder("Anonymous Tipster")
        .organization("Whistleblower Network")
        .role("Source")
        .build();
    let cid = contact.id;
    state.storage.create_contact(contact).expect("save contact");
    state.load_all().expect("reload");

    // Dispatch delete prompt
    state.update(AppMessage::PromptDeleteContact(cid));

    // Verify modal state
    match &state.modal {
        ModalState::ConfirmDeleteContact {
            id,
            name,
            linked_article_count,
            active_article_count,
            active_article_slugs,
        } => {
            assert_eq!(*id, cid);
            assert_eq!(name, "Anonymous Tipster");
            assert_eq!(*linked_article_count, 0);
            assert_eq!(*active_article_count, 0);
            assert!(active_article_slugs.is_empty());
        }
        other => panic!("Expected ConfirmDeleteContact, got {other:?}"),
    }

    assert!(state.modal.is_confirm_delete_contact());
    assert_eq!(state.modal.active_linked_articles_count(), 0);

    // Build container view model
    let container = build_modal_container_view(&state);
    assert!(container.is_open);
    assert!(container.is_destructive_prompt());
    assert_eq!(container.header.title, "Delete Contact?");
    assert_eq!(
        container.header.subtitle,
        "This action will permanently delete the contact record"
    );
    assert!(container.header.badge_text.is_none());

    // Verify ConfirmationDialogViewModel
    let confirmation = container
        .confirmation
        .expect("ConfirmationDialogViewModel present");
    assert_eq!(confirmation.entity_type, DeleteEntityType::Contact);
    assert_eq!(confirmation.entity_id, cid);
    assert_eq!(confirmation.entity_name, "Anonymous Tipster");
    assert!(!confirmation.has_active_links);
    assert!(confirmation.alert_banner.is_none());
    assert_eq!(confirmation.linked_count, 0);
    assert_eq!(confirmation.active_count, 0);
    assert_eq!(confirmation.confirm_button_label, "Delete Permanently");
    assert_eq!(confirmation.cancel_button_label, "Cancel");
    assert_eq!(confirmation.confirm_shortcut, "Enter");
    assert_eq!(confirmation.cancel_shortcut, "Esc");
    assert!(confirmation.warning_message.contains("Anonymous Tipster"));
    assert!(confirmation
        .unlink_consequences
        .iter()
        .any(|c| c.contains("not currently tagged in any stories")));
}

#[test]
fn test_contact_linked_only_to_published_stories_deletion_prompt() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let contact = Contact::builder("Archived Source")
        .organization("State Archives")
        .role("Historian")
        .build();
    let cid = contact.id;
    state.storage.create_contact(contact).expect("save contact");

    let mut pub1 = Article::new("historic-flood-1972", "Historic Flood of 1972");
    pub1.stage = ArticleStage::Published;
    let mut pub2 = Article::new("railroad-centennial", "Railroad Centennial");
    pub2.stage = ArticleStage::Published;

    let p1_id = pub1.id;
    let p2_id = pub2.id;
    state.storage.create_article(pub1).expect("save pub1");
    state.storage.create_article(pub2).expect("save pub2");

    state
        .storage
        .link_contact_to_article(p1_id, cid)
        .expect("link1");
    state
        .storage
        .link_contact_to_article(p2_id, cid)
        .expect("link2");
    state.load_all().expect("reload");

    // Dispatch delete prompt
    state.update(AppMessage::PromptDeleteContact(cid));

    match &state.modal {
        ModalState::ConfirmDeleteContact {
            id,
            name,
            linked_article_count,
            active_article_count,
            active_article_slugs,
        } => {
            assert_eq!(*id, cid);
            assert_eq!(name, "Archived Source");
            assert_eq!(*linked_article_count, 2);
            assert_eq!(*active_article_count, 0);
            assert!(active_article_slugs.is_empty());
        }
        other => panic!("Expected ConfirmDeleteContact, got {other:?}"),
    }

    let container = build_modal_container_view(&state);
    assert_eq!(container.header.title, "Delete Contact?");
    assert_eq!(
        container.header.subtitle,
        "Contact is tagged in published stories - unlinking cannot be undone"
    );
    assert_eq!(
        container.header.badge_text,
        Some("2 linked articles".to_string())
    );

    let confirmation = container.confirmation.expect("confirmation view model");
    assert!(!confirmation.has_active_links);
    assert!(confirmation.alert_banner.is_none());
    assert_eq!(confirmation.linked_count, 2);
    assert_eq!(confirmation.active_count, 0);
    assert_eq!(confirmation.confirm_button_label, "Delete & Unlink Contact");
    assert!(confirmation
        .unlink_consequences
        .iter()
        .any(|c| c.contains("Unlinks contact from 2 published stories")));
}

#[test]
fn test_contact_linked_to_active_stories_deletion_safety_warning() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let contact = Contact::builder("Deep Throat")
        .organization("Executive Branch")
        .role("Anonymous Source")
        .build();
    let cid = contact.id;
    state.storage.create_contact(contact).expect("save contact");

    let mut art1 = Article::new("watergate-wiretaps", "Watergate Wiretaps Investigation");
    art1.stage = ArticleStage::Writing;
    let mut art2 = Article::new("white-house-tapes", "White House Secret Tapes");
    art2.stage = ArticleStage::Researching;
    let mut art3 = Article::new("watergate-indictments", "Watergate Indictments Handed Down");
    art3.stage = ArticleStage::Published;

    let a1_id = art1.id;
    let a2_id = art2.id;
    let a3_id = art3.id;

    state.storage.create_article(art1).expect("save art1");
    state.storage.create_article(art2).expect("save art2");
    state.storage.create_article(art3).expect("save art3");

    state
        .storage
        .link_contact_to_article(a1_id, cid)
        .expect("link1");
    state
        .storage
        .link_contact_to_article(a2_id, cid)
        .expect("link2");
    state
        .storage
        .link_contact_to_article(a3_id, cid)
        .expect("link3");
    state.load_all().expect("reload");

    // Dispatch delete prompt
    state.update(AppMessage::PromptDeleteContact(cid));

    match &state.modal {
        ModalState::ConfirmDeleteContact {
            id,
            name,
            linked_article_count,
            active_article_count,
            active_article_slugs,
        } => {
            assert_eq!(*id, cid);
            assert_eq!(name, "Deep Throat");
            assert_eq!(*linked_article_count, 3);
            assert_eq!(*active_article_count, 2);
            assert_eq!(active_article_slugs.len(), 2);
            assert!(active_article_slugs.contains(&"watergate-wiretaps".to_string()));
            assert!(active_article_slugs.contains(&"white-house-tapes".to_string()));
            assert!(!active_article_slugs.contains(&"watergate-indictments".to_string()));
        }
        other => panic!("Expected ConfirmDeleteContact, got {other:?}"),
    }

    assert_eq!(state.modal.active_linked_articles_count(), 2);

    let container = build_modal_container_view(&state);
    assert_eq!(
        container.header.title,
        "Delete Contact & Unlink Active Stories?"
    );
    assert_eq!(
        container.header.subtitle,
        "⚠️ Contact is tagged in active stories - unlinking cannot be undone"
    );
    assert_eq!(
        container.header.badge_text,
        Some("2 active stories".to_string())
    );

    // Verify footer view model
    assert_eq!(container.footer.primary_label, "Delete & Unlink");
    assert!(container.footer.primary_is_destructive);
    assert_eq!(container.footer.primary_shortcut, "Enter");
    assert!(container
        .footer
        .save_tooltip
        .contains("unlink from 2 active stories"));

    // Verify confirmation view model
    let confirmation = container.confirmation.expect("confirmation view model");
    assert!(confirmation.has_active_links);
    assert_eq!(confirmation.linked_count, 3);
    assert_eq!(confirmation.active_count, 2);
    assert_eq!(
        confirmation.confirm_button_label,
        "Delete & Unlink (2 Active Stories)"
    );
    assert!(confirmation
        .alert_banner
        .as_ref()
        .expect("alert banner")
        .contains("Linked to 2 active stories"));
    assert!(confirmation.warning_message.contains("Deep Throat"));
    assert!(confirmation.warning_message.contains("2 active stories"));
    assert!(confirmation.warning_message.contains("3 total stories"));

    // Verify consequences breakdown
    let consequences = &confirmation.unlink_consequences;
    assert!(consequences
        .iter()
        .any(|c| c.contains("Unlinks source from 2 active stories")));
    assert!(consequences
        .iter()
        .any(|c| c.contains("watergate-wiretaps")));
    assert!(consequences.iter().any(|c| c.contains("white-house-tapes")));
    assert!(consequences
        .iter()
        .any(|c| c.contains("Removes contact from 3 total story records")));
}

#[test]
fn test_contact_deletion_submission_and_in_memory_state_cleanup() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let contact = Contact::builder("Jane Whistleblower")
        .organization("Metro Transit Agency")
        .role("Safety Inspector")
        .build();
    let cid = contact.id;
    state.storage.create_contact(contact).expect("save contact");

    let mut active_art = Article::new("transit-derailment", "Transit Safety Investigation");
    active_art.stage = ArticleStage::Editing;
    let mut pub_art = Article::new("transit-fare-hike", "Transit Fare Hike Review");
    pub_art.stage = ArticleStage::Published;

    let a1_id = active_art.id;
    let a2_id = pub_art.id;

    state.storage.create_article(active_art).expect("save art1");
    state.storage.create_article(pub_art).expect("save art2");

    state
        .storage
        .link_contact_to_article(a1_id, cid)
        .expect("link1");
    state
        .storage
        .link_contact_to_article(a2_id, cid)
        .expect("link2");
    state.load_all().expect("reload");

    assert_eq!(state.contacts.len(), 1);
    assert_eq!(state.contact_articles.get(&cid).map(Vec::len), Some(2));
    assert_eq!(state.article_contacts.get(&a1_id).cloned(), Some(vec![cid]));
    assert_eq!(state.article_contacts.get(&a2_id).cloned(), Some(vec![cid]));

    // Step 1: Prompt deletion
    state.update(AppMessage::PromptDeleteContact(cid));
    assert!(state.modal.is_open());

    // Step 2: Confirm deletion via SubmitModal
    let commands = state.update(AppMessage::SubmitModal);

    // Verify side-effect commands
    assert!(commands
        .iter()
        .any(|cmd| matches!(cmd, AppCommand::DeleteContact(id) if *id == cid)));
    assert!(commands.iter().any(|cmd| matches!(
        cmd,
        AppCommand::EmitToast(toast)
            if toast.title == "Contact Deleted"
                && toast.body.contains("deleted and unlinked from 2 stories (1 active)")
    )));

    // Verify in-memory state is synchronously cleaned up
    assert!(state.contacts.is_empty());
    assert!(!state.contact_articles.contains_key(&cid));
    assert_eq!(state.article_contacts.get(&a1_id), Some(&Vec::new()));
    assert_eq!(state.article_contacts.get(&a2_id), Some(&Vec::new()));

    // Verify articles are not deleted or corrupted
    assert_eq!(state.articles.len(), 2);
    assert!(state.articles.iter().any(|a| a.id == a1_id));
    assert!(state.articles.iter().any(|a| a.id == a2_id));
}

#[test]
fn test_contact_deletion_keyboard_shortcuts() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let contact = Contact::builder("Key Shortcut Source")
        .organization("Newsroom")
        .build();
    let cid = contact.id;
    state.storage.create_contact(contact).expect("save contact");
    state.load_all().expect("reload");

    // Open confirmation dialog
    state.update(AppMessage::PromptDeleteContact(cid));
    assert!(state.modal.is_open());

    let none_mods = NavKeyModifiers::none();

    // 1. Esc key closes modal (cancellation)
    let esc_msg = resolve_app_shortcut("Escape", none_mods, false, &state);
    assert_eq!(esc_msg, Some(AppMessage::CloseModal));

    // 2. Enter key submits delete modal (confirmation)
    let enter_msg = resolve_app_shortcut("Enter", none_mods, false, &state);
    assert_eq!(enter_msg, Some(AppMessage::SubmitModal));
}

#[test]
fn test_storage_cascade_and_persistence_integrity() {
    let state = AppState::in_memory().expect("in-memory state");
    let contact = Contact::builder("City Council Member")
        .organization("City Hall")
        .role("Elected Official")
        .build();
    let cid = contact.id;
    state.storage.create_contact(contact).expect("save contact");

    let art1 = Article::new("council-rezoning", "City Council Rezoning Vote");
    let art2 = Article::new("budget-amendment", "Budget Amendment Debate");
    let a1_id = art1.id;
    let a2_id = art2.id;

    state.storage.create_article(art1).expect("save art1");
    state.storage.create_article(art2).expect("save art2");

    state
        .storage
        .link_contact_to_article(a1_id, cid)
        .expect("link1");
    state
        .storage
        .link_contact_to_article(a2_id, cid)
        .expect("link2");

    // Verify storage has links
    let links1 = state
        .storage
        .list_contacts_for_article(a1_id)
        .expect("links1");
    assert_eq!(links1.len(), 1);
    assert_eq!(links1[0].id, cid);

    // Execute DeleteContact command
    let deleted = state.storage.delete_contact(cid).expect("delete contact");
    assert!(deleted);

    // Verify SQLite cascade removed article_contacts links
    let post_links1 = state
        .storage
        .list_contacts_for_article(a1_id)
        .expect("links1");
    let post_links2 = state
        .storage
        .list_contacts_for_article(a2_id)
        .expect("links2");
    assert!(post_links1.is_empty());
    assert!(post_links2.is_empty());

    // Verify articles still exist in storage
    assert!(state.storage.get_article(a1_id).expect("get1").is_some());
    assert!(state.storage.get_article(a2_id).expect("get2").is_some());

    // Verify contact is gone
    assert!(state.storage.get_contact(cid).expect("get c").is_none());
}

#[test]
fn test_cosmic_and_macos_app_view_tree_confirmation_modal_integration() {
    let mut state = AppState::in_memory().expect("in-memory state");
    let contact = Contact::builder("Cross-Platform Contact")
        .organization("Platform Engineering")
        .build();
    let cid = contact.id;
    state.storage.create_contact(contact).expect("save contact");

    let mut art = Article::new("cross-platform-ui", "Cross Platform UI Engine");
    art.stage = ArticleStage::Writing;
    let aid = art.id;
    state.storage.create_article(art).expect("save art");
    state
        .storage
        .link_contact_to_article(aid, cid)
        .expect("link");
    state.load_all().expect("reload");

    // Trigger delete prompt
    state.update(AppMessage::PromptDeleteContact(cid));

    // 1. COSMIC Linux view tree
    let cosmic_app = CosmicApp::new(state.clone());
    let cosmic_tree = cosmic_app.build_view_tree();
    assert!(cosmic_tree.modal_view.is_some());
    assert!(cosmic_tree.modal_container.is_some());
    let cosmic_container = cosmic_tree.modal_container.unwrap();
    assert_eq!(
        cosmic_container.header.title,
        "Delete Contact & Unlink Active Stories?"
    );
    assert_eq!(
        cosmic_container.header.badge_text,
        Some("1 active stories".to_string())
    );
    assert!(cosmic_container.confirmation.is_some());
    assert_eq!(
        cosmic_container
            .confirmation
            .as_ref()
            .unwrap()
            .confirm_button_label,
        "Delete & Unlink (1 Active Stories)"
    );

    // 2. macOS iced view tree
    let macos_app = MacosApp::new(state.clone());
    let macos_tree = macos_app.build_view_tree();
    assert!(macos_tree.modal_view.is_some());
    assert!(macos_tree.modal_container.is_some());
    let macos_container = macos_tree.modal_container.unwrap();
    assert_eq!(
        macos_container.header.title,
        "Delete Contact & Unlink Active Stories?"
    );
    assert_eq!(
        macos_container.header.badge_text,
        Some("1 active stories".to_string())
    );
    assert!(macos_container.confirmation.is_some());
    assert_eq!(
        macos_container
            .confirmation
            .as_ref()
            .unwrap()
            .confirm_button_label,
        "Delete & Unlink (1 Active Stories)"
    );

    // 3. Layout builder explicit placement
    let explicit_container =
        build_modal_container_view_with_layout(&state, ModalPlacement::CenteredModal, true);
    assert_eq!(explicit_container.placement, ModalPlacement::CenteredModal);
    assert!(explicit_container.confirmation.is_some());
}
