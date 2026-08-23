//! Comprehensive test suite for Contacts Directory list view, sorting, search filtering,
//! toolbar metrics, empty states, avatar initials, and reducer message handlers (Task 8.1).

use chrono::{Duration, Utc};
use newsjournal_core::models::{Article, Contact};
use newsjournal_gui::message::AppMessage;
use newsjournal_gui::state::filters::{ContactSortConfig, ContactSortField, SortDirection};
use newsjournal_gui::views::{
    assign_avatar_color_for_contact, build_contact_column_headers, build_contacts_directory_view,
    build_contacts_view, format_contact_initials, ContactsToolbarViewModel,
};
use newsjournal_gui::{AppState, EventLoop};

/// Helper to set up a test environment with 4 diverse contacts and 3 tagged articles.
fn setup_test_contacts_environment() -> (EventLoop, Vec<Contact>, Vec<Article>) {
    let state = AppState::in_memory().expect("in-memory state");
    let mut event_loop = EventLoop::new(state);

    let base_time = Utc::now();

    // Contact 1: Alice Walker - Mayor spokesperson, 2 articles
    let mut c1 = Contact::builder("Alice Walker")
        .organization("City Hall")
        .role("Chief Spokesperson")
        .email("alice.walker@cityhall.gov")
        .phone("5551112222")
        .notes("Background source on municipal budget")
        .build();
    c1.created_at = base_time - Duration::days(10);

    // Contact 2: Bob Chen - Transit director, 3 articles
    let mut c2 = Contact::builder("Bob Chen")
        .organization("Metro Transit Authority")
        .role("Director of Operations")
        .email("bchen@metro.org")
        .phone("5553334444")
        .notes("On the record interviews for rail projects")
        .build();
    c2.created_at = base_time - Duration::days(5);

    // Contact 3: Clara Oswald - Independent whistleblower, 0 articles
    let mut c3 = Contact::builder("Clara Oswald")
        .email("clara.o@proton.me")
        .phone("5559998888")
        .notes("Confidential whistleblower")
        .build();
    c3.created_at = base_time - Duration::days(1);

    // Contact 4: David Alvarez - Police union rep, 1 article
    let mut c4 = Contact::builder("David Alvarez")
        .organization("Police Union Local 42")
        .role("President")
        .email("dalvarez@local42.org")
        .phone("5554445555")
        .notes("Official statements on public safety contracts")
        .build();
    c4.created_at = base_time - Duration::days(20);

    let contacts = vec![c1.clone(), c2.clone(), c3.clone(), c4.clone()];
    for c in &contacts {
        event_loop
            .dispatch(AppMessage::CreateContact(c.clone()))
            .unwrap();
    }

    // Create 3 articles
    let a1 = Article::builder("subway-crisis", "Subway System Delays").build();
    let a2 = Article::builder("budget-hearing", "City Council Budget Hearing").build();
    let a3 = Article::builder("safety-audit", "Public Safety Audit Released").build();

    let articles = vec![a1.clone(), a2.clone(), a3.clone()];
    for a in &articles {
        event_loop
            .dispatch(AppMessage::CreateArticle(a.clone()))
            .unwrap();
    }

    // Link contacts to articles:
    // c1 (Alice): a1, a2 (2 stories)
    // c2 (Bob): a1, a2, a3 (3 stories)
    // c4 (David): a3 (1 story)
    // c3 (Clara): 0 stories
    event_loop
        .dispatch(AppMessage::LinkContactToArticle {
            article_id: a1.id,
            contact_id: c1.id,
        })
        .unwrap();
    event_loop
        .dispatch(AppMessage::LinkContactToArticle {
            article_id: a2.id,
            contact_id: c1.id,
        })
        .unwrap();

    event_loop
        .dispatch(AppMessage::LinkContactToArticle {
            article_id: a1.id,
            contact_id: c2.id,
        })
        .unwrap();
    event_loop
        .dispatch(AppMessage::LinkContactToArticle {
            article_id: a2.id,
            contact_id: c2.id,
        })
        .unwrap();
    event_loop
        .dispatch(AppMessage::LinkContactToArticle {
            article_id: a3.id,
            contact_id: c2.id,
        })
        .unwrap();

    event_loop
        .dispatch(AppMessage::LinkContactToArticle {
            article_id: a3.id,
            contact_id: c4.id,
        })
        .unwrap();

    (event_loop, contacts, articles)
}

#[test]
fn test_contacts_directory_initial_state() {
    let (event_loop, _, _) = setup_test_contacts_environment();
    let state = event_loop.state();

    let directory = build_contacts_directory_view(state);
    assert_eq!(directory.total_count, 4);
    assert_eq!(directory.filtered_count, 4);
    assert_eq!(directory.total_citations_count, 6); // 2 + 3 + 0 + 1 = 6
    assert!(!directory.is_empty);
    assert!(!directory.is_filtered_empty);
    assert!(directory.empty_state.is_none());

    // Default sort is Name Ascending
    let names: Vec<&str> = directory.contacts.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["Alice Walker", "Bob Chen", "Clara Oswald", "David Alvarez"]
    );

    // Verify row view model fields for Alice Walker
    let alice = &directory.contacts[0];
    assert_eq!(alice.name, "Alice Walker");
    assert_eq!(alice.initials, "AW");
    assert_eq!(alice.organization, "City Hall");
    assert!(alice.has_organization);
    assert_eq!(alice.role, "Chief Spokesperson");
    assert!(alice.has_role);
    assert_eq!(alice.email, "alice.walker@cityhall.gov");
    assert!(alice.has_email);
    assert_eq!(alice.phone, "5551112222");
    assert_eq!(alice.phone_display, "(555) 111-2222");
    assert!(alice.has_phone);
    assert_eq!(alice.linked_articles_count, 2);
    assert_eq!(alice.linked_articles_count_label, "2 stories");
    assert!(alice
        .linked_article_slugs
        .contains(&"subway-crisis".to_string()));
    assert!(alice
        .linked_article_slugs
        .contains(&"budget-hearing".to_string()));
    assert_eq!(alice.linked_articles.len(), 2);
}

#[test]
fn test_contacts_sorting_by_all_columns() {
    let (mut event_loop, _, _) = setup_test_contacts_environment();

    // 1. Sort by Name DESC
    event_loop
        .dispatch(AppMessage::SetContactSort(ContactSortConfig::with(
            ContactSortField::Name,
            SortDirection::Descending,
        )))
        .unwrap();
    let view = build_contacts_view(event_loop.state());
    let names: Vec<&str> = view.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["David Alvarez", "Clara Oswald", "Bob Chen", "Alice Walker"]
    );

    // 2. Sort by Organization ASC (Clara has None, should be placed at the end)
    event_loop
        .dispatch(AppMessage::SetContactSort(ContactSortConfig::with(
            ContactSortField::Organization,
            SortDirection::Ascending,
        )))
        .unwrap();
    let view = build_contacts_view(event_loop.state());
    let names: Vec<&str> = view.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "Alice Walker",  // City Hall
            "Bob Chen",      // Metro Transit Authority
            "David Alvarez", // Police Union Local 42
            "Clara Oswald"   // None (placed last)
        ]
    );

    // 3. Sort by Organization DESC
    event_loop
        .dispatch(AppMessage::SetContactSort(ContactSortConfig::with(
            ContactSortField::Organization,
            SortDirection::Descending,
        )))
        .unwrap();
    let view = build_contacts_view(event_loop.state());
    let names: Vec<&str> = view.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "Clara Oswald",  // None reversed to first in DESC
            "David Alvarez", // Police Union Local 42
            "Bob Chen",      // Metro Transit Authority
            "Alice Walker"   // City Hall
        ]
    );

    // 4. Sort by Role ASC (Clara has None)
    event_loop
        .dispatch(AppMessage::SetContactSort(ContactSortConfig::with(
            ContactSortField::Role,
            SortDirection::Ascending,
        )))
        .unwrap();
    let view = build_contacts_view(event_loop.state());
    let names: Vec<&str> = view.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "Alice Walker",  // Chief Spokesperson
            "Bob Chen",      // Director of Operations
            "David Alvarez", // President
            "Clara Oswald"   // None
        ]
    );

    // 5. Sort by Email ASC
    event_loop
        .dispatch(AppMessage::SetContactSort(ContactSortConfig::with(
            ContactSortField::Email,
            SortDirection::Ascending,
        )))
        .unwrap();
    let view = build_contacts_view(event_loop.state());
    let names: Vec<&str> = view.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "Alice Walker",  // alice.walker@cityhall.gov
            "Bob Chen",      // bchen@metro.org
            "Clara Oswald",  // clara.o@proton.me
            "David Alvarez"  // dalvarez@local42.org
        ]
    );

    // 6. Sort by Tagged Stories Count DESC (Most stories first)
    event_loop
        .dispatch(AppMessage::SetContactSort(ContactSortConfig::with(
            ContactSortField::StoriesCount,
            SortDirection::Descending,
        )))
        .unwrap();
    let view = build_contacts_view(event_loop.state());
    let names: Vec<&str> = view.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "Bob Chen",      // 3 stories
            "Alice Walker",  // 2 stories
            "David Alvarez", // 1 story
            "Clara Oswald"   // 0 stories
        ]
    );

    // 7. Sort by Tagged Stories Count ASC (Fewest stories first)
    event_loop
        .dispatch(AppMessage::SetContactSort(ContactSortConfig::with(
            ContactSortField::StoriesCount,
            SortDirection::Ascending,
        )))
        .unwrap();
    let view = build_contacts_view(event_loop.state());
    let names: Vec<&str> = view.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "Clara Oswald",  // 0 stories
            "David Alvarez", // 1 story
            "Alice Walker",  // 2 stories
            "Bob Chen"       // 3 stories
        ]
    );

    // 8. Sort by Recent (Creation Date DESC: Newest first)
    // Created dates: Clara (1d ago), Bob (5d ago), Alice (10d ago), David (20d ago)
    event_loop
        .dispatch(AppMessage::SetContactSort(ContactSortConfig::with(
            ContactSortField::Recent,
            SortDirection::Descending,
        )))
        .unwrap();
    let view = build_contacts_view(event_loop.state());
    let names: Vec<&str> = view.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["Clara Oswald", "Bob Chen", "Alice Walker", "David Alvarez"]
    );
}

#[test]
fn test_toggle_column_sort_messages() {
    let (mut event_loop, _, _) = setup_test_contacts_environment();

    // Initial is Name ASC
    assert_eq!(
        event_loop.state().filters.contact_sort.field,
        ContactSortField::Name
    );
    assert_eq!(
        event_loop.state().filters.contact_sort.direction,
        SortDirection::Ascending
    );

    // Toggle Name -> flips to DESC
    event_loop
        .dispatch(AppMessage::ToggleContactSort(ContactSortField::Name))
        .unwrap();
    assert_eq!(
        event_loop.state().filters.contact_sort.direction,
        SortDirection::Descending
    );

    // Toggle Organization -> switches to Organization ASC
    event_loop
        .dispatch(AppMessage::ToggleContactSort(
            ContactSortField::Organization,
        ))
        .unwrap();
    assert_eq!(
        event_loop.state().filters.contact_sort.field,
        ContactSortField::Organization
    );
    assert_eq!(
        event_loop.state().filters.contact_sort.direction,
        SortDirection::Ascending
    );

    // Toggle StoriesCount -> switches to StoriesCount DESC (default direction for count)
    event_loop
        .dispatch(AppMessage::ToggleContactSort(
            ContactSortField::StoriesCount,
        ))
        .unwrap();
    assert_eq!(
        event_loop.state().filters.contact_sort.field,
        ContactSortField::StoriesCount
    );
    assert_eq!(
        event_loop.state().filters.contact_sort.direction,
        SortDirection::Descending
    );

    // Reset sort -> restores default Name ASC
    event_loop.dispatch(AppMessage::ResetContactSort).unwrap();
    assert_eq!(
        event_loop.state().filters.contact_sort.field,
        ContactSortField::Name
    );
    assert_eq!(
        event_loop.state().filters.contact_sort.direction,
        SortDirection::Ascending
    );
}

#[test]
fn test_search_filtering_across_fields() {
    let (mut event_loop, _, _) = setup_test_contacts_environment();

    // 1. Search by Name
    event_loop
        .dispatch(AppMessage::SetSearchQuery("david".to_string()))
        .unwrap();
    let dir = build_contacts_directory_view(event_loop.state());
    assert_eq!(dir.filtered_count, 1);
    assert_eq!(dir.contacts[0].name, "David Alvarez");
    let hl = dir.contacts[0].search_highlight.as_ref().unwrap();
    assert!(hl.matches_name);
    assert!(!hl.matches_organization);

    // 2. Search by Organization
    event_loop
        .dispatch(AppMessage::SetSearchQuery("metro transit".to_string()))
        .unwrap();
    let dir = build_contacts_directory_view(event_loop.state());
    assert_eq!(dir.filtered_count, 1);
    assert_eq!(dir.contacts[0].name, "Bob Chen");
    let hl = dir.contacts[0].search_highlight.as_ref().unwrap();
    assert!(hl.matches_organization);

    // 3. Search by Role
    event_loop
        .dispatch(AppMessage::SetSearchQuery("spokesperson".to_string()))
        .unwrap();
    let dir = build_contacts_directory_view(event_loop.state());
    assert_eq!(dir.filtered_count, 1);
    assert_eq!(dir.contacts[0].name, "Alice Walker");
    let hl = dir.contacts[0].search_highlight.as_ref().unwrap();
    assert!(hl.matches_role);

    // 4. Search by Email domain
    event_loop
        .dispatch(AppMessage::SetSearchQuery("proton.me".to_string()))
        .unwrap();
    let dir = build_contacts_directory_view(event_loop.state());
    assert_eq!(dir.filtered_count, 1);
    assert_eq!(dir.contacts[0].name, "Clara Oswald");
    let hl = dir.contacts[0].search_highlight.as_ref().unwrap();
    assert!(hl.matches_email);

    // 5. Search by Phone number digits
    event_loop
        .dispatch(AppMessage::SetSearchQuery("9998888".to_string()))
        .unwrap();
    let dir = build_contacts_directory_view(event_loop.state());
    assert_eq!(dir.filtered_count, 1);
    assert_eq!(dir.contacts[0].name, "Clara Oswald");
    let hl = dir.contacts[0].search_highlight.as_ref().unwrap();
    assert!(hl.matches_phone);

    // 6. Search by Notes snippet
    event_loop
        .dispatch(AppMessage::SetSearchQuery("whistleblower".to_string()))
        .unwrap();
    let dir = build_contacts_directory_view(event_loop.state());
    assert_eq!(dir.filtered_count, 1);
    assert_eq!(dir.contacts[0].name, "Clara Oswald");
    let hl = dir.contacts[0].search_highlight.as_ref().unwrap();
    assert!(hl.matches_notes);

    // 7. Search by linked Story Slug
    event_loop
        .dispatch(AppMessage::SetSearchQuery("safety-audit".to_string()))
        .unwrap();
    let dir = build_contacts_directory_view(event_loop.state());
    // Bob and David are tagged in safety-audit
    assert_eq!(dir.filtered_count, 2);
    let matched_names: Vec<&str> = dir.contacts.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(matched_names, vec!["Bob Chen", "David Alvarez"]);

    // 8. Clear search
    event_loop
        .dispatch(AppMessage::SetSearchQuery(String::new()))
        .unwrap();
    let dir = build_contacts_directory_view(event_loop.state());
    assert_eq!(dir.filtered_count, 4);
    assert!(!dir.toolbar.is_search_active);
}

#[test]
fn test_contacts_empty_state_and_toolbar_badges() {
    let (mut event_loop, _, _) = setup_test_contacts_environment();

    // 1. Search with no matches -> filtered empty state
    event_loop
        .dispatch(AppMessage::SetSearchQuery("quantum physics".to_string()))
        .unwrap();
    let dir = build_contacts_directory_view(event_loop.state());
    assert_eq!(dir.filtered_count, 0);
    assert_eq!(dir.total_count, 4);
    assert!(dir.is_filtered_empty);
    assert!(!dir.is_empty);
    assert!(dir.empty_state.is_some());
    let empty = dir.empty_state.unwrap();
    assert_eq!(empty.headline, "No Contacts Found");
    assert!(empty.is_filtered);
    assert_eq!(empty.action_button_label, "Clear Search");

    // 2. Toolbar formatting
    let toolbar = ContactsToolbarViewModel::build(event_loop.state(), 4, 0, 6);
    assert_eq!(toolbar.total_count_label, "4 contacts");
    assert_eq!(toolbar.total_citations_label, "6 story citations");
    assert_eq!(
        toolbar.filtered_count_label,
        Some("0 of 4 matching".to_string())
    );

    // 3. Complete database empty state
    let empty_state = AppState::in_memory().expect("in-memory");
    let empty_dir = build_contacts_directory_view(&empty_state);
    assert!(empty_dir.is_empty);
    assert!(!empty_dir.is_filtered_empty);
    assert_eq!(empty_dir.total_count, 0);
    assert_eq!(empty_dir.filtered_count, 0);
    assert_eq!(empty_dir.total_citations_count, 0);
    let empty_vm = empty_dir.empty_state.unwrap();
    assert_eq!(empty_vm.headline, "No Contacts in Directory");
    assert!(!empty_vm.is_filtered);
    assert_eq!(empty_vm.action_button_label, "+ Add First Contact");
}

#[test]
fn test_initials_and_avatar_colors() {
    assert_eq!(format_contact_initials("Jane Doe"), "JD");
    assert_eq!(format_contact_initials("Alice"), "Al");
    assert_eq!(format_contact_initials("Alice Smith"), "AS");
    assert_eq!(format_contact_initials("John Quincy Adams"), "JA");
    assert_eq!(format_contact_initials(""), "??");

    let id = uuid::Uuid::new_v4();
    let col = assign_avatar_color_for_contact(id, "Jane Doe");
    assert!(col.starts_with('#'));
    assert_eq!(col.len(), 7);
}

#[test]
fn test_column_headers_generation() {
    let sort = ContactSortConfig::with(ContactSortField::StoriesCount, SortDirection::Descending);
    let headers = build_contact_column_headers(&sort);
    assert_eq!(headers.len(), 6);

    let stories_header = headers
        .iter()
        .find(|h| h.field == ContactSortField::StoriesCount)
        .unwrap();
    assert!(stories_header.is_sorted);
    assert_eq!(
        stories_header.sort_direction,
        Some(SortDirection::Descending)
    );
    assert_eq!(stories_header.sort_indicator, "▼");

    let name_header = headers
        .iter()
        .find(|h| h.field == ContactSortField::Name)
        .unwrap();
    assert!(!name_header.is_sorted);
    assert_eq!(name_header.sort_direction, None);
    assert_eq!(name_header.sort_indicator, "↕");
}
