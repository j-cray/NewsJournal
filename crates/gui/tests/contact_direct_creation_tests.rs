//! Comprehensive integration tests for Task 8.3: Direct Creation Button ("Add Contact").
//!
//! Tests:
//! 1. Top action bar "Add Contact" button properties, tooltip, platform shortcut display, icons, and messages.
//! 2. Direct creation action opening clean contact modal drawer with required name gating.
//! 3. Full direct creation and persistence lifecycle from top action bar button to SQLite storage.
//! 4. Cross-platform keyboard shortcut triggers (`⌘⇧C` on macOS and `Ctrl+Shift+C` on Linux).
//! 5. COSMIC header bar action routing (`OpenNewContactModal` and `PrimaryAction` on `ContactsDirectory`).
//! 6. macOS unified toolbar action routing (`OpenNewContactModal` and `PrimaryAction` on `ContactsDirectory`).
//! 7. Direct creation triggers on natural and filtered empty states.

use newsjournal_gui::message::AppMessage;
use newsjournal_gui::navigation::{resolve_app_shortcut, NavKeyModifiers, NavTab};
use newsjournal_gui::platform::cosmic::app::CosmicApp;
use newsjournal_gui::platform::cosmic::header_bar::{CosmicHeaderBar, CosmicHeaderBarAction};
use newsjournal_gui::platform::macos::app::MacosApp;
use newsjournal_gui::platform::macos::toolbar::{MacosToolbar, MacosToolbarAction};
use newsjournal_gui::state::modal::ModalState;
use newsjournal_gui::state::AppState;
use newsjournal_gui::views::{
    build_contacts_directory_view, build_modal_container_view, ContactsEmptyStateViewModel,
    ContactsToolbarViewModel,
};
use newsjournal_gui::EventLoop;

#[test]
fn test_contacts_toolbar_primary_action_properties() {
    let state = AppState::in_memory().expect("in-memory state");
    let directory = build_contacts_directory_view(&state);
    let toolbar = directory.toolbar;

    // Action button label & tooltip
    assert_eq!(toolbar.primary_action_label, "+ Add Contact");
    assert_eq!(
        toolbar.primary_action_tooltip,
        "Create a new contact record (⌘⇧C / Ctrl+Shift+C)"
    );

    // Default shortcut & platform-aware shortcut helper
    assert_eq!(toolbar.primary_action_shortcut, "⌘⇧C");
    assert_eq!(
        ContactsToolbarViewModel::primary_action_shortcut_for_platform(true),
        "⌘⇧C"
    );
    assert_eq!(
        ContactsToolbarViewModel::primary_action_shortcut_for_platform(false),
        "Ctrl+Shift+C"
    );

    // Icon representations
    assert_eq!(toolbar.primary_action_icon_name, "user-plus");
    assert_eq!(toolbar.primary_action_sf_symbol, "person.badge.plus");
    assert_eq!(toolbar.primary_action_icon_emoji, "➕");

    // Action message
    assert_eq!(
        toolbar.primary_action_message,
        AppMessage::OpenNewContactModal
    );
}

#[test]
fn test_direct_creation_opens_clean_contact_modal() {
    let mut state = AppState::in_memory().expect("in-memory state");
    assert!(!state.modal.is_open());

    // Dispatch the primary action message from the top toolbar
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
    assert!(!contact_form.can_save); // Save is disabled until required name is supplied
    assert!(!contact_form.header.can_delete);
    assert_eq!(contact_form.header.delete_action, None);

    // Verify empty state for associated articles in direct creation
    assert!(contact_form.associated_articles.is_empty);
    assert_eq!(contact_form.associated_articles.count, 0);
}

#[test]
fn test_direct_creation_full_lifecycle_and_persistence() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    // 1. Initial empty directory state
    assert_eq!(event_loop.state().contacts.len(), 0);
    let initial_dir = build_contacts_directory_view(event_loop.state());
    assert_eq!(initial_dir.total_count, 0);

    // 2. Click "+ Add Contact" in top action bar
    event_loop
        .dispatch(AppMessage::OpenNewContactModal)
        .expect("open contact modal");
    assert!(event_loop.state().modal.is_open());

    // 3. Fill in fields via granular update messages
    event_loop
        .dispatch(AppMessage::UpdateContactDraftName(
            "Eleanor Vance".to_string(),
        ))
        .unwrap();
    event_loop
        .dispatch(AppMessage::UpdateContactDraftOrg(
            "Hill House Historical Society".to_string(),
        ))
        .unwrap();
    event_loop
        .dispatch(AppMessage::UpdateContactDraftRole(
            "Archivist & Researcher".to_string(),
        ))
        .unwrap();
    event_loop
        .dispatch(AppMessage::UpdateContactDraftEmail(
            "eleanor@hillhouse.org".to_string(),
        ))
        .unwrap();
    event_loop
        .dispatch(AppMessage::UpdateContactDraftPhone(
            "(555) 432-8765".to_string(),
        ))
        .unwrap();
    event_loop
        .dispatch(AppMessage::UpdateContactDraftNotes(
            "Key primary source on 19th-century regional estates.".to_string(),
        ))
        .unwrap();

    // Verify form validation passes
    match &event_loop.state().modal {
        ModalState::ContactForm(draft) => {
            assert!(draft.is_valid());
            assert_eq!(draft.name, "Eleanor Vance");
            assert_eq!(draft.phone, "(555) 432-8765");
        }
        other => panic!("Expected ContactForm, got {other:?}"),
    }

    // 4. Save contact
    event_loop
        .dispatch(AppMessage::SubmitModal)
        .expect("save contact form");

    // Modal is closed
    assert!(!event_loop.state().modal.is_open());
    assert_eq!(event_loop.state().contacts.len(), 1);

    // 5. Verify updated Contacts Directory view
    let updated_dir = build_contacts_directory_view(event_loop.state());
    assert_eq!(updated_dir.total_count, 1);
    assert_eq!(updated_dir.filtered_count, 1);
    assert_eq!(updated_dir.toolbar.total_count_label, "1 contact");

    let saved = &updated_dir.contacts[0];
    assert_eq!(saved.name, "Eleanor Vance");
    assert_eq!(saved.initials, "EV");
    assert_eq!(saved.organization, "Hill House Historical Society");
    assert_eq!(saved.role, "Archivist & Researcher");
    assert_eq!(saved.email, "eleanor@hillhouse.org");
    assert_eq!(saved.phone, "5554328765");
    assert_eq!(saved.phone_display, "(555) 432-8765");
    assert_eq!(
        saved.notes,
        "Key primary source on 19th-century regional estates."
    );
}

#[test]
fn test_keyboard_shortcut_direct_creation_triggers() {
    let state = AppState::in_memory().expect("in-memory state");

    // 1. macOS shortcut: ⌘⇧C (Cmd + Shift + C)
    let macos_modifiers = NavKeyModifiers {
        ctrl: false,
        alt: false,
        shift: true,
        meta: true, // Command on macOS
    };
    let macos_msg = resolve_app_shortcut("c", macos_modifiers, true, &state);
    assert_eq!(macos_msg, Some(AppMessage::OpenNewContactModal));

    // Also uppercase "C"
    let macos_msg_upper = resolve_app_shortcut("C", macos_modifiers, true, &state);
    assert_eq!(macos_msg_upper, Some(AppMessage::OpenNewContactModal));

    // 2. Linux shortcut: Ctrl+Shift+C (Ctrl + Shift + C)
    let linux_modifiers = NavKeyModifiers {
        ctrl: true,
        alt: false,
        shift: true,
        meta: false,
    };
    let linux_msg = resolve_app_shortcut("c", linux_modifiers, false, &state);
    assert_eq!(linux_msg, Some(AppMessage::OpenNewContactModal));

    let linux_msg_upper = resolve_app_shortcut("C", linux_modifiers, false, &state);
    assert_eq!(linux_msg_upper, Some(AppMessage::OpenNewContactModal));

    // 3. Dispatch through CosmicApp key event handler
    let mut cosmic_app = CosmicApp::new(state.clone());
    let handled_cosmic = cosmic_app
        .handle_key_event("C", linux_modifiers)
        .expect("handle key event");
    assert!(handled_cosmic);
    assert!(cosmic_app.state().modal.is_open());

    // 4. Dispatch through MacosApp key event handler
    let mut macos_app = MacosApp::new(state);
    let handled_macos = macos_app
        .handle_key_event("C", macos_modifiers)
        .expect("handle key event");
    assert!(handled_macos);
    assert!(macos_app.state().modal.is_open());
}

#[test]
fn test_cosmic_header_bar_direct_contact_creation_action() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut cosmic_app = CosmicApp::new(state);

    // Label check
    assert_eq!(
        CosmicHeaderBar::primary_action_label(NavTab::ContactsDirectory),
        "+ New Contact"
    );

    // 1. Explicit OpenNewContactModal header action
    cosmic_app
        .handle_header_action(CosmicHeaderBarAction::OpenNewContactModal)
        .expect("handle open new contact action");
    assert!(cosmic_app.state().modal.is_open());

    // Close modal
    cosmic_app
        .dispatch(AppMessage::CloseModal)
        .expect("close modal");
    assert!(!cosmic_app.state().modal.is_open());

    // 2. Generic PrimaryAction when navigated to ContactsDirectory tab
    cosmic_app
        .dispatch(AppMessage::NavigateTo(NavTab::ContactsDirectory))
        .expect("navigate");
    assert_eq!(cosmic_app.state().active_tab, NavTab::ContactsDirectory);

    cosmic_app
        .handle_header_action(CosmicHeaderBarAction::PrimaryAction)
        .expect("handle primary action on contacts tab");
    assert!(cosmic_app.state().modal.is_open());
    match &cosmic_app.state().modal {
        ModalState::ContactForm(draft) => assert!(draft.id.is_none()),
        other => panic!("Expected ContactForm, got {other:?}"),
    }
}

#[test]
fn test_macos_toolbar_direct_contact_creation_action() {
    let state = AppState::in_memory().expect("in-memory state");
    let mut macos_app = MacosApp::new(state);

    // Label check
    assert_eq!(
        MacosToolbar::primary_action_label(NavTab::ContactsDirectory),
        "+ New Contact"
    );

    // 1. Explicit OpenNewContactModal toolbar action
    macos_app
        .handle_toolbar_action(MacosToolbarAction::OpenNewContactModal)
        .expect("handle open new contact action");
    assert!(macos_app.state().modal.is_open());

    // Close modal
    macos_app
        .dispatch(AppMessage::CloseModal)
        .expect("close modal");
    assert!(!macos_app.state().modal.is_open());

    // 2. Generic PrimaryAction when navigated to ContactsDirectory tab
    macos_app
        .dispatch(AppMessage::NavigateTo(NavTab::ContactsDirectory))
        .expect("navigate");
    assert_eq!(macos_app.state().active_tab, NavTab::ContactsDirectory);

    macos_app
        .handle_toolbar_action(MacosToolbarAction::PrimaryAction)
        .expect("handle primary action on contacts tab");
    assert!(macos_app.state().modal.is_open());
    match &macos_app.state().modal {
        ModalState::ContactForm(draft) => assert!(draft.id.is_none()),
        other => panic!("Expected ContactForm, got {other:?}"),
    }
}

#[test]
fn test_empty_state_direct_creation_triggers() {
    let state = AppState::in_memory().expect("in-memory state");

    // 1. Natural empty directory (0 contacts stored)
    let empty_vm = ContactsEmptyStateViewModel::build(false, "");
    assert!(!empty_vm.is_filtered);
    assert_eq!(empty_vm.headline, "No Contacts in Directory");
    assert_eq!(empty_vm.action_button_label, "+ Add First Contact");
    assert_eq!(empty_vm.action_message, AppMessage::OpenNewContactModal);
    assert_eq!(empty_vm.secondary_action_label, None);

    // 2. Filtered empty directory (contacts exist, but query matches 0)
    let filtered_empty_vm = ContactsEmptyStateViewModel::build(true, "Nonexistent Source");
    assert!(filtered_empty_vm.is_filtered);
    assert_eq!(filtered_empty_vm.headline, "No Contacts Found");
    assert_eq!(filtered_empty_vm.action_button_label, "Clear Search");
    assert_eq!(
        filtered_empty_vm.secondary_action_label,
        Some("+ Add Contact".to_string())
    );
    assert_eq!(
        filtered_empty_vm.secondary_action_message,
        Some(AppMessage::OpenNewContactModal)
    );

    // 3. Dispatching empty state primary action opens creation drawer
    let mut event_loop = EventLoop::new(state);
    event_loop.dispatch(empty_vm.action_message).unwrap();
    assert!(event_loop.state().modal.is_open());

    // Close and dispatch filtered secondary action
    event_loop.dispatch(AppMessage::CloseModal).unwrap();
    assert!(!event_loop.state().modal.is_open());

    event_loop
        .dispatch(filtered_empty_vm.secondary_action_message.unwrap())
        .unwrap();
    assert!(event_loop.state().modal.is_open());
}
