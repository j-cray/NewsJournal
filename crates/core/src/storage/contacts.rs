//! SQLite repository operations for Contacts and sources.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use uuid::Uuid;

use crate::models::Contact;
use crate::storage::error::StorageError;

const SELECT_CONTACT_FIELDS: &str =
    "id, name, organization, role, phone, email, notes, created_at, updated_at";

/// Maps a SQLite row to a [`Contact`] domain model.
pub fn row_to_contact(row: &Row<'_>) -> Result<Contact, StorageError> {
    let id_str: String = row.get(0)?;
    let name: String = row.get(1)?;
    let organization: Option<String> = row.get(2)?;
    let role: Option<String> = row.get(3)?;
    let phone: Option<String> = row.get(4)?;
    let email: Option<String> = row.get(5)?;
    let notes: Option<String> = row.get(6)?;
    let created_at_str: String = row.get(7)?;
    let updated_at_str: String = row.get(8)?;

    let id = Uuid::parse_str(&id_str)
        .map_err(|e| StorageError::InvalidData(format!("invalid contact id '{id_str}': {e}")))?;

    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| {
            StorageError::InvalidData(format!(
                "invalid created_at timestamp '{created_at_str}': {e}"
            ))
        })?;

    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| {
            StorageError::InvalidData(format!(
                "invalid updated_at timestamp '{updated_at_str}': {e}"
            ))
        })?;

    Ok(Contact {
        id,
        name,
        organization,
        role,
        phone,
        email,
        notes,
        created_at,
        updated_at,
    })
}

/// Inserts a new contact into the database after validating all invariants.
pub fn insert_contact(conn: &Connection, contact: &Contact) -> Result<(), StorageError> {
    contact.validate()?;

    let res = conn.execute(
        "INSERT INTO contacts (id, name, organization, role, phone, email, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            contact.id.to_string(),
            contact.name,
            contact.organization,
            contact.role,
            contact.phone,
            contact.email,
            contact.notes,
            contact.created_at.to_rfc3339(),
            contact.updated_at.to_rfc3339(),
        ],
    );

    match res {
        Ok(_) => Ok(()),
        Err(err) => {
            let msg = err.to_string();
            if msg.contains("UNIQUE constraint failed: contacts.id") {
                Err(StorageError::Conflict(format!(
                    "a contact with ID '{}' already exists",
                    contact.id
                )))
            } else {
                Err(StorageError::Sqlite(err))
            }
        }
    }
}

/// Fetches a contact by its unique UUID.
pub fn get_contact_by_id(conn: &Connection, id: Uuid) -> Result<Option<Contact>, StorageError> {
    let sql = format!("SELECT {SELECT_CONTACT_FIELDS} FROM contacts WHERE id = ?1");
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params![id.to_string()])?;

    if let Some(row) = rows.next()? {
        let contact = row_to_contact(row)?;
        Ok(Some(contact))
    } else {
        Ok(None)
    }
}

/// Updates an existing contact in the database.
///
/// Validates invariants before committing. Returns [`StorageError::NotFound`] if the contact does not exist.
pub fn update_contact(conn: &Connection, contact: &Contact) -> Result<(), StorageError> {
    contact.validate()?;

    let res = conn.execute(
        "UPDATE contacts
         SET name = ?1, organization = ?2, role = ?3, phone = ?4, email = ?5, notes = ?6, updated_at = ?7
         WHERE id = ?8",
        params![
            contact.name,
            contact.organization,
            contact.role,
            contact.phone,
            contact.email,
            contact.notes,
            contact.updated_at.to_rfc3339(),
            contact.id.to_string(),
        ],
    );

    match res {
        Ok(0) => Err(StorageError::NotFound(format!(
            "contact with id '{}' not found",
            contact.id
        ))),
        Ok(_) => Ok(()),
        Err(err) => Err(StorageError::Sqlite(err)),
    }
}

/// Deletes a contact by its ID.
///
/// Associated rows in `article_contacts` are automatically deleted via SQLite foreign key cascade.
/// Returns `true` if deleted, `false` if not found.
pub fn delete_contact(conn: &Connection, id: Uuid) -> Result<bool, StorageError> {
    let rows_affected = conn.execute(
        "DELETE FROM contacts WHERE id = ?1",
        params![id.to_string()],
    )?;
    Ok(rows_affected > 0)
}

/// Lists all contacts alphabetically by name.
pub fn list_contacts(conn: &Connection) -> Result<Vec<Contact>, StorageError> {
    let sql =
        format!("SELECT {SELECT_CONTACT_FIELDS} FROM contacts ORDER BY name COLLATE NOCASE ASC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        row_to_contact(row).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })
    })?;

    let mut contacts = Vec::new();
    for row in rows {
        contacts.push(row?);
    }
    Ok(contacts)
}

/// Searches contacts matching a query string across name, organization, role, email, and notes.
pub fn search_contacts(conn: &Connection, query: &str) -> Result<Vec<Contact>, StorageError> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return list_contacts(conn);
    }

    let pattern = format!("%{trimmed}%");
    let sql = format!(
        "SELECT {SELECT_CONTACT_FIELDS} FROM contacts
         WHERE name LIKE ?1
            OR organization LIKE ?1
            OR role LIKE ?1
            OR email LIKE ?1
            OR phone LIKE ?1
            OR notes LIKE ?1
         ORDER BY name COLLATE NOCASE ASC"
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![pattern], |row| {
        row_to_contact(row).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })
    })?;

    let mut contacts = Vec::new();
    for row in rows {
        contacts.push(row?);
    }
    Ok(contacts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::connection::open_in_memory;

    #[test]
    fn test_contact_crud_flow() {
        let conn = open_in_memory().expect("failed to open db");
        let contact = Contact::new("Dr. Aris Thorne")
            .with_organization("Department of Public Health")
            .with_role("Chief Medical Epidemiologist")
            .with_email("thorne@health.example.gov")
            .with_phone("+1 555-019-2834")
            .with_notes("On-the-record expert on water safety");

        // Insert
        insert_contact(&conn, &contact).expect("insert contact failed");

        // Get by ID
        let fetched = get_contact_by_id(&conn, contact.id)
            .expect("get contact failed")
            .expect("contact not found");
        assert_eq!(fetched.name, "Dr. Aris Thorne");
        assert_eq!(
            fetched.organization.as_deref(),
            Some("Department of Public Health")
        );
        assert_eq!(
            fetched.role.as_deref(),
            Some("Chief Medical Epidemiologist")
        );
        assert_eq!(fetched.email.as_deref(), Some("thorne@health.example.gov"));
        assert_eq!(fetched.phone.as_deref(), Some("+1 555-019-2834"));

        // Update
        let mut updated = fetched;
        updated.role = Some("Senior Health Director".to_string());
        update_contact(&conn, &updated).expect("update contact failed");

        let after_update = get_contact_by_id(&conn, contact.id).unwrap().unwrap();
        assert_eq!(after_update.role.as_deref(), Some("Senior Health Director"));

        // List
        let list = list_contacts(&conn).expect("list contacts failed");
        assert_eq!(list.len(), 1);

        // Search
        let results = search_contacts(&conn, "epidemiologist").unwrap();
        assert_eq!(results.len(), 0); // Changed to Senior Health Director

        let results2 = search_contacts(&conn, "Health").unwrap();
        assert_eq!(results2.len(), 1);

        let results3 = search_contacts(&conn, "water safety").unwrap();
        assert_eq!(results3.len(), 1);

        // Delete
        let deleted = delete_contact(&conn, contact.id).expect("delete contact failed");
        assert!(deleted);
        assert!(get_contact_by_id(&conn, contact.id).unwrap().is_none());
    }

    #[test]
    fn test_contact_validation_failure_on_insert() {
        let conn = open_in_memory().expect("failed to open db");
        let contact = Contact::new("").with_email("invalid-email");
        let err = insert_contact(&conn, &contact).expect_err("expected validation failure");
        assert!(matches!(err, StorageError::Validation(_)));
    }
}
