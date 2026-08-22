//! SQLite repository operations for Tasks.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use uuid::Uuid;

use crate::models::{Task, TaskStatus};
use crate::storage::error::StorageError;

const SELECT_TASK_FIELDS: &str =
    "id, article_id, title, notes, due_date, status, created_at, updated_at";

/// Maps a SQLite row to a [`Task`] domain model.
pub fn row_to_task(row: &Row<'_>) -> Result<Task, StorageError> {
    let id_str: String = row.get(0)?;
    let article_id_str: String = row.get(1)?;
    let title: String = row.get(2)?;
    let notes: Option<String> = row.get(3)?;
    let due_date_str: Option<String> = row.get(4)?;
    let status_str: String = row.get(5)?;
    let created_at_str: String = row.get(6)?;
    let updated_at_str: String = row.get(7)?;

    let id = Uuid::parse_str(&id_str)
        .map_err(|e| StorageError::InvalidData(format!("invalid task id '{id_str}': {e}")))?;

    let article_id = Uuid::parse_str(&article_id_str).map_err(|e| {
        StorageError::InvalidData(format!(
            "invalid task parent article id '{article_id_str}': {e}"
        ))
    })?;

    let status: TaskStatus = status_str.parse().map_err(|e| {
        StorageError::InvalidData(format!("invalid task status '{status_str}': {e}"))
    })?;

    let due_date = match due_date_str {
        Some(s) => Some(
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| {
                    StorageError::InvalidData(format!("invalid due_date timestamp '{s}': {e}"))
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

    Ok(Task {
        id,
        article_id,
        title,
        notes,
        due_date,
        status,
        created_at,
        updated_at,
    })
}

/// Inserts a new task associated with an existing article.
///
/// Validates invariants before persisting. Returns [`StorageError::NotFound`] if the parent article does not exist.
pub fn insert_task(conn: &Connection, task: &Task) -> Result<(), StorageError> {
    task.validate()?;

    let due_date_str = task.due_date.map(|d| d.to_rfc3339());
    let res = conn.execute(
        "INSERT INTO tasks (id, article_id, title, notes, due_date, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            task.id.to_string(),
            task.article_id.to_string(),
            task.title,
            task.notes,
            due_date_str,
            task.status.as_str(),
            task.created_at.to_rfc3339(),
            task.updated_at.to_rfc3339(),
        ],
    );

    match res {
        Ok(_) => Ok(()),
        Err(err) => {
            let msg = err.to_string();
            if msg.contains("FOREIGN KEY constraint failed") {
                Err(StorageError::NotFound(format!(
                    "parent article with id '{}' does not exist",
                    task.article_id
                )))
            } else if msg.contains("UNIQUE constraint failed: tasks.id") {
                Err(StorageError::Conflict(format!(
                    "a task with ID '{}' already exists",
                    task.id
                )))
            } else {
                Err(StorageError::Sqlite(err))
            }
        }
    }
}

/// Fetches a task by its unique UUID.
pub fn get_task_by_id(conn: &Connection, id: Uuid) -> Result<Option<Task>, StorageError> {
    let sql = format!("SELECT {SELECT_TASK_FIELDS} FROM tasks WHERE id = ?1");
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params![id.to_string()])?;

    if let Some(row) = rows.next()? {
        let task = row_to_task(row)?;
        Ok(Some(task))
    } else {
        Ok(None)
    }
}

/// Updates an existing task in the database.
///
/// Validates invariants before updating. Returns [`StorageError::NotFound`] if the task or parent article does not exist.
pub fn update_task(conn: &Connection, task: &Task) -> Result<(), StorageError> {
    task.validate()?;

    let due_date_str = task.due_date.map(|d| d.to_rfc3339());
    let res = conn.execute(
        "UPDATE tasks
         SET article_id = ?1, title = ?2, notes = ?3, due_date = ?4, status = ?5, updated_at = ?6
         WHERE id = ?7",
        params![
            task.article_id.to_string(),
            task.title,
            task.notes,
            due_date_str,
            task.status.as_str(),
            task.updated_at.to_rfc3339(),
            task.id.to_string(),
        ],
    );

    match res {
        Ok(0) => Err(StorageError::NotFound(format!(
            "task with id '{}' not found",
            task.id
        ))),
        Ok(_) => Ok(()),
        Err(err) => {
            let msg = err.to_string();
            if msg.contains("FOREIGN KEY constraint failed") {
                Err(StorageError::NotFound(format!(
                    "parent article with id '{}' does not exist",
                    task.article_id
                )))
            } else {
                Err(StorageError::Sqlite(err))
            }
        }
    }
}

/// Deletes a task by its ID. Returns `true` if deleted, `false` if not found.
pub fn delete_task(conn: &Connection, id: Uuid) -> Result<bool, StorageError> {
    let rows_affected = conn.execute("DELETE FROM tasks WHERE id = ?1", params![id.to_string()])?;
    Ok(rows_affected > 0)
}

/// Lists all tasks across all articles, ordered by created_at descending.
pub fn list_tasks(conn: &Connection) -> Result<Vec<Task>, StorageError> {
    let sql = format!("SELECT {SELECT_TASK_FIELDS} FROM tasks ORDER BY created_at DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        row_to_task(row).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })
    })?;

    let mut tasks = Vec::new();
    for row in rows {
        tasks.push(row?);
    }
    Ok(tasks)
}

/// Lists all tasks belonging to a specific parent article, ordered by creation date.
pub fn list_tasks_for_article(
    conn: &Connection,
    article_id: Uuid,
) -> Result<Vec<Task>, StorageError> {
    let sql = format!(
        "SELECT {SELECT_TASK_FIELDS} FROM tasks WHERE article_id = ?1 ORDER BY created_at ASC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![article_id.to_string()], |row| {
        row_to_task(row).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })
    })?;

    let mut tasks = Vec::new();
    for row in rows {
        tasks.push(row?);
    }
    Ok(tasks)
}

/// Lists all tasks filtered by workflow status (`to_do`, `in_progress`, `complete`).
pub fn list_tasks_by_status(
    conn: &Connection,
    status: TaskStatus,
) -> Result<Vec<Task>, StorageError> {
    let sql = format!(
        "SELECT {SELECT_TASK_FIELDS} FROM tasks WHERE status = ?1 ORDER BY created_at DESC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![status.as_str()], |row| {
        row_to_task(row).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })
    })?;

    let mut tasks = Vec::new();
    for row in rows {
        tasks.push(row?);
    }
    Ok(tasks)
}

/// Updates the status of a task and updates `updated_at`.
pub fn change_task_status(
    conn: &Connection,
    id: Uuid,
    new_status: TaskStatus,
) -> Result<Task, StorageError> {
    let now = Utc::now().to_rfc3339();
    let rows = conn.execute(
        "UPDATE tasks SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![new_status.as_str(), now, id.to_string()],
    )?;

    if rows == 0 {
        return Err(StorageError::NotFound(format!(
            "task with id '{id}' not found"
        )));
    }

    get_task_by_id(conn, id)?
        .ok_or_else(|| StorageError::NotFound(format!("task with id '{id}' not found")))
}

/// Computes task count summary for an article as `(total_count, completed_count)`.
pub fn count_tasks_for_article(
    conn: &Connection,
    article_id: Uuid,
) -> Result<(usize, usize), StorageError> {
    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM tasks WHERE article_id = ?1",
        params![article_id.to_string()],
        |row| row.get(0),
    )?;

    let completed: i64 = conn.query_row(
        "SELECT COUNT(*) FROM tasks WHERE article_id = ?1 AND status = 'complete'",
        params![article_id.to_string()],
        |row| row.get(0),
    )?;

    Ok((total.max(0) as usize, completed.max(0) as usize))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Article;
    use crate::storage::articles::insert_article;
    use crate::storage::connection::open_in_memory;

    #[test]
    fn test_task_crud_flow() {
        let conn = open_in_memory().expect("failed to open db");
        let article = Article::new("budget-investigation", "City Budget Investigation");
        insert_article(&conn, &article).expect("insert article failed");

        let task = Task::new(article.id, "Interview City Treasurer")
            .with_notes("Request 2025 revenue audits")
            .with_status(TaskStatus::ToDo);

        // Insert
        insert_task(&conn, &task).expect("insert task failed");

        // Get by ID
        let fetched = get_task_by_id(&conn, task.id)
            .expect("get by id failed")
            .expect("task not found");
        assert_eq!(fetched.title, "Interview City Treasurer");
        assert_eq!(
            fetched.notes.as_deref(),
            Some("Request 2025 revenue audits")
        );
        assert_eq!(fetched.status, TaskStatus::ToDo);

        // Update
        let mut updated = fetched;
        updated.title = "Interview City Treasurer and Deputy".to_string();
        updated.status = TaskStatus::InProgress;
        update_task(&conn, &updated).expect("update task failed");

        let after_update = get_task_by_id(&conn, task.id).unwrap().unwrap();
        assert_eq!(after_update.title, "Interview City Treasurer and Deputy");
        assert_eq!(after_update.status, TaskStatus::InProgress);

        // List
        let all_tasks = list_tasks(&conn).expect("list tasks failed");
        assert_eq!(all_tasks.len(), 1);

        // List for article
        let article_tasks =
            list_tasks_for_article(&conn, article.id).expect("list for article failed");
        assert_eq!(article_tasks.len(), 1);

        // Status change
        let completed_task =
            change_task_status(&conn, task.id, TaskStatus::Complete).expect("status change failed");
        assert_eq!(completed_task.status, TaskStatus::Complete);

        // Counts
        let (total, completed) = count_tasks_for_article(&conn, article.id).unwrap();
        assert_eq!(total, 1);
        assert_eq!(completed, 1);

        // List by status
        let complete_list = list_tasks_by_status(&conn, TaskStatus::Complete).unwrap();
        assert_eq!(complete_list.len(), 1);
        let todo_list = list_tasks_by_status(&conn, TaskStatus::ToDo).unwrap();
        assert_eq!(todo_list.len(), 0);

        // Delete
        let deleted = delete_task(&conn, task.id).expect("delete task failed");
        assert!(deleted);
        assert!(get_task_by_id(&conn, task.id).unwrap().is_none());
    }

    #[test]
    fn test_task_orphaned_foreign_key_fails() {
        let conn = open_in_memory().expect("failed to open db");
        let non_existent_article_id = Uuid::new_v4();
        let task = Task::new(non_existent_article_id, "Orphaned Task");

        let err = insert_task(&conn, &task).expect_err("expected foreign key failure");
        assert!(err.is_not_found());
    }
}
