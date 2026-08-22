//! SQLite repository operations for Article-Contact many-to-many join relationships.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use uuid::Uuid;

use crate::models::{Article, ArticleContact, Contact};
use crate::storage::articles::row_to_article;
use crate::storage::contacts::row_to_contact;
use crate::storage::error::StorageError;

/// Maps a SQLite row to an [`ArticleContact`] join domain model.
pub fn row_to_article_contact(row: &Row<'_>) -> Result<ArticleContact, StorageError> {
    let article_id_str: String = row.get(0)?;
    let contact_id_str: String = row.get(1)?;
    let created_at_str: String = row.get(2)?;

    let article_id = Uuid::parse_str(&article_id_str).map_err(|e| {
        StorageError::InvalidData(format!("invalid article_id '{article_id_str}': {e}"))
    })?;

    let contact_id = Uuid::parse_str(&contact_id_str).map_err(|e| {
        StorageError::InvalidData(format!("invalid contact_id '{contact_id_str}': {e}"))
    })?;

    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| {
            StorageError::InvalidData(format!(
                "invalid created_at timestamp '{created_at_str}': {e}"
            ))
        })?;

    Ok(ArticleContact {
        article_id,
        contact_id,
        created_at,
    })
}

/// Links a contact to an article.
///
/// If the link already exists, this operation is idempotent and returns the existing link.
/// If either the article or contact does not exist, returns [`StorageError::NotFound`].
pub fn link_contact_to_article(
    conn: &Connection,
    article_id: Uuid,
    contact_id: Uuid,
) -> Result<ArticleContact, StorageError> {
    let now = Utc::now();
    let now_str = now.to_rfc3339();

    let res = conn.execute(
        "INSERT OR IGNORE INTO article_contacts (article_id, contact_id, created_at)
         VALUES (?1, ?2, ?3)",
        params![article_id.to_string(), contact_id.to_string(), now_str],
    );

    match res {
        Ok(_) => {
            // Verify that the link actually exists (or if foreign key was ignored/failed)
            let existing: Option<String> = conn
                .query_row(
                    "SELECT created_at FROM article_contacts WHERE article_id = ?1 AND contact_id = ?2",
                    params![article_id.to_string(), contact_id.to_string()],
                    |r| r.get(0),
                )
                .map(Some)
                .or_else(|err| {
                    if matches!(err, rusqlite::Error::QueryReturnedNoRows) {
                        Ok(None)
                    } else {
                        Err(err)
                    }
                })?;

            if let Some(ts_str) = existing {
                let ts = DateTime::parse_from_rfc3339(&ts_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| StorageError::InvalidData(e.to_string()))?;
                Ok(ArticleContact {
                    article_id,
                    contact_id,
                    created_at: ts,
                })
            } else {
                Err(StorageError::NotFound(format!(
                    "failed to link article '{article_id}' and contact '{contact_id}': article or contact not found"
                )))
            }
        }
        Err(err) => {
            let msg = err.to_string();
            if msg.contains("FOREIGN KEY constraint failed") {
                Err(StorageError::NotFound(format!(
                    "article '{article_id}' or contact '{contact_id}' does not exist"
                )))
            } else {
                Err(StorageError::Sqlite(err))
            }
        }
    }
}

/// Unlinks a contact from an article.
///
/// Returns `true` if the link was present and removed, `false` if no link existed.
pub fn unlink_contact_from_article(
    conn: &Connection,
    article_id: Uuid,
    contact_id: Uuid,
) -> Result<bool, StorageError> {
    let rows_affected = conn.execute(
        "DELETE FROM article_contacts WHERE article_id = ?1 AND contact_id = ?2",
        params![article_id.to_string(), contact_id.to_string()],
    )?;
    Ok(rows_affected > 0)
}

/// Checks whether a contact is linked to a specific article.
pub fn is_contact_linked_to_article(
    conn: &Connection,
    article_id: Uuid,
    contact_id: Uuid,
) -> Result<bool, StorageError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM article_contacts WHERE article_id = ?1 AND contact_id = ?2",
        params![article_id.to_string(), contact_id.to_string()],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

/// Lists all contacts tagged to a specific article, ordered alphabetically.
pub fn list_contacts_for_article(
    conn: &Connection,
    article_id: Uuid,
) -> Result<Vec<Contact>, StorageError> {
    let sql = "SELECT c.id, c.name, c.organization, c.role, c.phone, c.email, c.notes, c.created_at, c.updated_at
               FROM contacts c
               INNER JOIN article_contacts ac ON c.id = ac.contact_id
               WHERE ac.article_id = ?1
               ORDER BY c.name COLLATE NOCASE ASC";
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![article_id.to_string()], |row| {
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

/// Lists all articles that tag a specific contact, ordered by creation date descending.
pub fn list_articles_for_contact(
    conn: &Connection,
    contact_id: Uuid,
) -> Result<Vec<Article>, StorageError> {
    let sql = "SELECT a.id, a.slug, a.headline, a.description, a.stage, a.deadline, a.color, a.created_at, a.updated_at
               FROM articles a
               INNER JOIN article_contacts ac ON a.id = ac.article_id
               WHERE ac.contact_id = ?1
               ORDER BY a.created_at DESC";
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![contact_id.to_string()], |row| {
        row_to_article(row).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })
    })?;

    let mut articles = Vec::new();
    for row in rows {
        articles.push(row?);
    }
    Ok(articles)
}

/// Lists all article-contact join records across the system.
pub fn list_article_contacts(conn: &Connection) -> Result<Vec<ArticleContact>, StorageError> {
    let sql =
        "SELECT article_id, contact_id, created_at FROM article_contacts ORDER BY created_at DESC";
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| {
        row_to_article_contact(row).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })
    })?;

    let mut links = Vec::new();
    for row in rows {
        links.push(row?);
    }
    Ok(links)
}

/// Replaces the set of contacts tagged to an article in a transaction.
pub fn set_article_contacts(
    conn: &mut Connection,
    article_id: Uuid,
    contact_ids: &[Uuid],
) -> Result<(), StorageError> {
    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM article_contacts WHERE article_id = ?1",
        params![article_id.to_string()],
    )?;

    let now_str = Utc::now().to_rfc3339();
    {
        let mut insert_stmt = tx.prepare(
            "INSERT INTO article_contacts (article_id, contact_id, created_at) VALUES (?1, ?2, ?3)",
        )?;

        for contact_id in contact_ids {
            let res = insert_stmt.execute(params![
                article_id.to_string(),
                contact_id.to_string(),
                now_str
            ]);
            match res {
                Ok(_) => {}
                Err(err) => {
                    let msg = err.to_string();
                    if msg.contains("FOREIGN KEY constraint failed") {
                        return Err(StorageError::NotFound(format!(
                            "article '{article_id}' or contact '{contact_id}' does not exist"
                        )));
                    }
                    return Err(StorageError::Sqlite(err));
                }
            }
        }
    }

    tx.commit()?;
    Ok(())
}

/// Counts how many contacts are linked to an article.
pub fn count_contacts_for_article(
    conn: &Connection,
    article_id: Uuid,
) -> Result<usize, StorageError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM article_contacts WHERE article_id = ?1",
        params![article_id.to_string()],
        |row| row.get(0),
    )?;
    Ok(count.max(0) as usize)
}

/// Counts how many articles a contact is tagged in.
pub fn count_articles_for_contact(
    conn: &Connection,
    contact_id: Uuid,
) -> Result<usize, StorageError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM article_contacts WHERE contact_id = ?1",
        params![contact_id.to_string()],
        |row| row.get(0),
    )?;
    Ok(count.max(0) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Article, Contact};
    use crate::storage::articles::insert_article;
    use crate::storage::connection::open_in_memory;
    use crate::storage::contacts::insert_contact;

    #[test]
    fn test_article_contact_linking_flow() {
        let mut conn = open_in_memory().expect("failed to open db");
        let article1 = Article::new("subway-contract-expose", "Subway Contract Investigation");
        let article2 = Article::new("city-council-election", "City Council Election Preview");
        insert_article(&conn, &article1).expect("insert article1 failed");
        insert_article(&conn, &article2).expect("insert article2 failed");

        let contact1 = Contact::new("Council President Rivera");
        let contact2 = Contact::new("Auditor General Vance");
        insert_contact(&conn, &contact1).expect("insert contact1 failed");
        insert_contact(&conn, &contact2).expect("insert contact2 failed");

        // Link contact1 to article1
        let link1 = link_contact_to_article(&conn, article1.id, contact1.id)
            .expect("link contact1 to article1 failed");
        assert_eq!(link1.article_id, article1.id);
        assert_eq!(link1.contact_id, contact1.id);

        // Idempotent second link
        let link1_dup = link_contact_to_article(&conn, article1.id, contact1.id)
            .expect("idempotent link failed");
        assert_eq!(link1_dup.article_id, article1.id);

        // Link contact2 to article1 as well
        link_contact_to_article(&conn, article1.id, contact2.id)
            .expect("link contact2 to article1 failed");

        // Link contact1 to article2 as well
        link_contact_to_article(&conn, article2.id, contact1.id)
            .expect("link contact1 to article2 failed");

        // Query links
        assert!(is_contact_linked_to_article(&conn, article1.id, contact1.id).unwrap());
        assert!(is_contact_linked_to_article(&conn, article1.id, contact2.id).unwrap());
        assert!(is_contact_linked_to_article(&conn, article2.id, contact1.id).unwrap());
        assert!(!is_contact_linked_to_article(&conn, article2.id, contact2.id).unwrap());

        // Contacts for article1
        let a1_contacts = list_contacts_for_article(&conn, article1.id).unwrap();
        assert_eq!(a1_contacts.len(), 2);

        // Articles for contact1
        let c1_articles = list_articles_for_contact(&conn, contact1.id).unwrap();
        assert_eq!(c1_articles.len(), 2);

        // Counts
        assert_eq!(count_contacts_for_article(&conn, article1.id).unwrap(), 2);
        assert_eq!(count_contacts_for_article(&conn, article2.id).unwrap(), 1);
        assert_eq!(count_articles_for_contact(&conn, contact1.id).unwrap(), 2);
        assert_eq!(count_articles_for_contact(&conn, contact2.id).unwrap(), 1);

        // All links
        let all_links = list_article_contacts(&conn).unwrap();
        assert_eq!(all_links.len(), 3);

        // Bulk set contacts for article1 (replace with only contact2)
        set_article_contacts(&mut conn, article1.id, &[contact2.id]).unwrap();
        let a1_contacts_after = list_contacts_for_article(&conn, article1.id).unwrap();
        assert_eq!(a1_contacts_after.len(), 1);
        assert_eq!(a1_contacts_after[0].id, contact2.id);

        // Unlink contact2 from article1
        let unlinked =
            unlink_contact_from_article(&conn, article1.id, contact2.id).expect("unlink failed");
        assert!(unlinked);
        assert!(!is_contact_linked_to_article(&conn, article1.id, contact2.id).unwrap());
    }

    #[test]
    fn test_link_nonexistent_fails() {
        let conn = open_in_memory().expect("failed to open db");
        let a_id = Uuid::new_v4();
        let c_id = Uuid::new_v4();

        let err = link_contact_to_article(&conn, a_id, c_id).expect_err("expected not found");
        assert!(err.is_not_found());
    }
}
