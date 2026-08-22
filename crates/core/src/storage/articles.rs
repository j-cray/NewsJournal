//! SQLite repository operations for Articles.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use uuid::Uuid;

use crate::models::{Article, ArticleStage};
use crate::storage::error::StorageError;

const SELECT_ARTICLE_FIELDS: &str =
    "id, slug, headline, description, stage, deadline, color, created_at, updated_at";

/// Maps a SQLite row to an [`Article`] domain model.
pub fn row_to_article(row: &Row<'_>) -> Result<Article, StorageError> {
    let id_str: String = row.get(0)?;
    let slug: String = row.get(1)?;
    let headline: String = row.get(2)?;
    let description: Option<String> = row.get(3)?;
    let stage_str: String = row.get(4)?;
    let deadline_str: Option<String> = row.get(5)?;
    let color: Option<String> = row.get(6)?;
    let created_at_str: String = row.get(7)?;
    let updated_at_str: String = row.get(8)?;

    let id = Uuid::parse_str(&id_str)
        .map_err(|e| StorageError::InvalidData(format!("invalid article id '{id_str}': {e}")))?;

    let stage: ArticleStage = stage_str.parse().map_err(|e| {
        StorageError::InvalidData(format!("invalid article stage '{stage_str}': {e}"))
    })?;

    let deadline = match deadline_str {
        Some(s) => Some(
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| {
                    StorageError::InvalidData(format!("invalid deadline timestamp '{s}': {e}"))
                })?,
        ),
        None => None,
    };

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

    Ok(Article {
        id,
        slug,
        headline,
        description,
        stage,
        deadline,
        color,
        created_at,
        updated_at,
    })
}

/// Inserts a new article into the database after validating all invariants.
///
/// Returns [`StorageError::Conflict`] if an article with the same slug or ID already exists.
pub fn insert_article(conn: &Connection, article: &Article) -> Result<(), StorageError> {
    article.validate()?;

    let deadline_str = article.deadline.map(|d| d.to_rfc3339());
    let res = conn.execute(
        "INSERT INTO articles (id, slug, headline, description, stage, deadline, color, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            article.id.to_string(),
            article.slug,
            article.headline,
            article.description,
            article.stage.as_str(),
            deadline_str,
            article.color,
            article.created_at.to_rfc3339(),
            article.updated_at.to_rfc3339(),
        ],
    );

    match res {
        Ok(_) => Ok(()),
        Err(err) => {
            let msg = err.to_string();
            if msg.contains("UNIQUE constraint failed: articles.slug")
                || msg.contains("UNIQUE constraint failed: articles.id")
            {
                Err(StorageError::Conflict(format!(
                    "an article with slug '{}' or ID '{}' already exists",
                    article.slug, article.id
                )))
            } else {
                Err(StorageError::Sqlite(err))
            }
        }
    }
}

/// Fetches an article by its unique UUID.
pub fn get_article_by_id(conn: &Connection, id: Uuid) -> Result<Option<Article>, StorageError> {
    let sql = format!("SELECT {SELECT_ARTICLE_FIELDS} FROM articles WHERE id = ?1");
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params![id.to_string()])?;

    if let Some(row) = rows.next()? {
        let article = row_to_article(row)?;
        Ok(Some(article))
    } else {
        Ok(None)
    }
}

/// Fetches an article by its unique slug.
pub fn get_article_by_slug(conn: &Connection, slug: &str) -> Result<Option<Article>, StorageError> {
    let sql = format!("SELECT {SELECT_ARTICLE_FIELDS} FROM articles WHERE slug = ?1");
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params![slug])?;

    if let Some(row) = rows.next()? {
        let article = row_to_article(row)?;
        Ok(Some(article))
    } else {
        Ok(None)
    }
}

/// Updates an existing article in the database.
///
/// Validates invariants before committing. Returns [`StorageError::NotFound`] if the article does not exist,
/// or [`StorageError::Conflict`] if updating the slug collides with another article.
pub fn update_article(conn: &Connection, article: &Article) -> Result<(), StorageError> {
    article.validate()?;

    let deadline_str = article.deadline.map(|d| d.to_rfc3339());
    let res = conn.execute(
        "UPDATE articles
         SET slug = ?1, headline = ?2, description = ?3, stage = ?4, deadline = ?5, color = ?6, updated_at = ?7
         WHERE id = ?8",
        params![
            article.slug,
            article.headline,
            article.description,
            article.stage.as_str(),
            deadline_str,
            article.color,
            article.updated_at.to_rfc3339(),
            article.id.to_string(),
        ],
    );

    match res {
        Ok(0) => Err(StorageError::NotFound(format!(
            "article with id '{}' not found",
            article.id
        ))),
        Ok(_) => Ok(()),
        Err(err) => {
            let msg = err.to_string();
            if msg.contains("UNIQUE constraint failed: articles.slug") {
                Err(StorageError::Conflict(format!(
                    "an article with slug '{}' already exists",
                    article.slug
                )))
            } else {
                Err(StorageError::Sqlite(err))
            }
        }
    }
}

/// Deletes an article by its ID.
///
/// Associated tasks and article_contacts are automatically deleted via SQLite foreign key cascade.
/// Returns `true` if the article was deleted, `false` if it was not found.
pub fn delete_article(conn: &Connection, id: Uuid) -> Result<bool, StorageError> {
    let rows_affected = conn.execute(
        "DELETE FROM articles WHERE id = ?1",
        params![id.to_string()],
    )?;
    Ok(rows_affected > 0)
}

/// Lists all articles ordered by created_at descending.
pub fn list_articles(conn: &Connection) -> Result<Vec<Article>, StorageError> {
    let sql = format!("SELECT {SELECT_ARTICLE_FIELDS} FROM articles ORDER BY created_at DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
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

/// Lists all articles belonging to a specific editorial workflow stage.
pub fn list_articles_by_stage(
    conn: &Connection,
    stage: ArticleStage,
) -> Result<Vec<Article>, StorageError> {
    let sql = format!(
        "SELECT {SELECT_ARTICLE_FIELDS} FROM articles WHERE stage = ?1 ORDER BY created_at DESC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![stage.as_str()], |row| {
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

/// Checks whether a slug is available for a new or existing article.
///
/// If `exclude_id` is provided, that specific article's current slug is ignored (useful when editing).
pub fn is_slug_available(
    conn: &Connection,
    slug: &str,
    exclude_id: Option<Uuid>,
) -> Result<bool, StorageError> {
    let count: i64 = match exclude_id {
        Some(id) => conn.query_row(
            "SELECT COUNT(*) FROM articles WHERE slug = ?1 AND id != ?2",
            params![slug, id.to_string()],
            |row| row.get(0),
        )?,
        None => conn.query_row(
            "SELECT COUNT(*) FROM articles WHERE slug = ?1",
            params![slug],
            |row| row.get(0),
        )?,
    };

    Ok(count == 0)
}

/// Updates the editorial stage of an article and updates `updated_at`.
pub fn change_article_stage(
    conn: &Connection,
    id: Uuid,
    new_stage: ArticleStage,
) -> Result<Article, StorageError> {
    let now = Utc::now().to_rfc3339();
    let rows = conn.execute(
        "UPDATE articles SET stage = ?1, updated_at = ?2 WHERE id = ?3",
        params![new_stage.as_str(), now, id.to_string()],
    )?;

    if rows == 0 {
        return Err(StorageError::NotFound(format!(
            "article with id '{id}' not found"
        )));
    }

    get_article_by_id(conn, id)?
        .ok_or_else(|| StorageError::NotFound(format!("article with id '{id}' not found")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::connection::open_in_memory;

    #[test]
    fn test_article_crud_flow() {
        let conn = open_in_memory().expect("failed to open db");
        let article = Article::new("city-transit-audit", "City Transit Audit Underway")
            .with_description("Investigating procurement delays")
            .with_stage(ArticleStage::Researching)
            .with_color("#2E7D32");

        // Insert
        insert_article(&conn, &article).expect("insert failed");

        // Get by ID
        let fetched = get_article_by_id(&conn, article.id)
            .expect("get by id failed")
            .expect("article not found");
        assert_eq!(fetched.slug, "city-transit-audit");
        assert_eq!(fetched.headline, "City Transit Audit Underway");
        assert_eq!(fetched.stage, ArticleStage::Researching);
        assert_eq!(fetched.color.as_deref(), Some("#2E7D32"));

        // Get by Slug
        let fetched_slug = get_article_by_slug(&conn, "city-transit-audit")
            .expect("get by slug failed")
            .expect("article not found");
        assert_eq!(fetched_slug.id, article.id);

        // Update
        let mut updated = fetched;
        updated.headline = "City Transit Audit Expands to Metro Line".to_string();
        updated.stage = ArticleStage::Writing;
        update_article(&conn, &updated).expect("update failed");

        let after_update = get_article_by_id(&conn, article.id).unwrap().unwrap();
        assert_eq!(
            after_update.headline,
            "City Transit Audit Expands to Metro Line"
        );
        assert_eq!(after_update.stage, ArticleStage::Writing);

        // List
        let list = list_articles(&conn).expect("list failed");
        assert_eq!(list.len(), 1);

        // Stage change
        let changed = change_article_stage(&conn, article.id, ArticleStage::Editing)
            .expect("stage change failed");
        assert_eq!(changed.stage, ArticleStage::Editing);

        // List by stage
        let editing_list =
            list_articles_by_stage(&conn, ArticleStage::Editing).expect("list by stage failed");
        assert_eq!(editing_list.len(), 1);
        let writing_list =
            list_articles_by_stage(&conn, ArticleStage::Writing).expect("list by stage failed");
        assert_eq!(writing_list.len(), 0);

        // Slug availability
        assert!(!is_slug_available(&conn, "city-transit-audit", None).unwrap());
        assert!(is_slug_available(&conn, "city-transit-audit", Some(article.id)).unwrap());
        assert!(is_slug_available(&conn, "new-unique-slug", None).unwrap());

        // Delete
        let deleted = delete_article(&conn, article.id).expect("delete failed");
        assert!(deleted);
        assert!(get_article_by_id(&conn, article.id).unwrap().is_none());
        assert!(!delete_article(&conn, article.id).unwrap());
    }

    #[test]
    fn test_duplicate_slug_conflict() {
        let conn = open_in_memory().expect("failed to open db");
        let a1 = Article::new("exclusive-scoop", "First Scoop");
        let a2 = Article::new("exclusive-scoop", "Second Scoop");

        insert_article(&conn, &a1).expect("insert a1 failed");
        let err = insert_article(&conn, &a2).expect_err("expected duplicate slug error");
        assert!(err.is_conflict());
    }

    #[test]
    fn test_update_nonexistent_article_returns_not_found() {
        let conn = open_in_memory().expect("failed to open db");
        let a = Article::new("nonexistent", "Headline");
        let err = update_article(&conn, &a).expect_err("expected not found error");
        assert!(err.is_not_found());
    }
}
