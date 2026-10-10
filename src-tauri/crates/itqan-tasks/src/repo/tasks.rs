use chrono::{DateTime, Utc};
use rusqlite::types::Type as SqlType;
use rusqlite::{named_params, params, Connection, OptionalExtension, Row};

use crate::tasks::{Task, TaskFilter, TaskId, TaskInput, TaskKind, TaskStatus};
use itqan_core::error::AppError;

const COLUMNS: &str = "id, title, notes, category_id, kind, priority, due_at, is_top_three, \
                       status, completed_at, parent_id, created_at, updated_at, goal_id, skill_id";

fn invalid(column: usize, value: &str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        column,
        SqlType::Text,
        format!("unexpected value {value}").into(),
    )
}

fn from_row(row: &Row) -> rusqlite::Result<Task> {
    let kind: String = row.get(4)?;
    let status: String = row.get(8)?;
    Ok(Task {
        id: row.get(0)?,
        title: row.get(1)?,
        notes: row.get(2)?,
        category_id: row.get(3)?,
        kind: TaskKind::parse(&kind).ok_or_else(|| invalid(4, &kind))?,
        priority: row.get(5)?,
        due_at: row.get(6)?,
        is_top_three: row.get(7)?,
        status: TaskStatus::parse(&status).ok_or_else(|| invalid(8, &status))?,
        completed_at: row.get(9)?,
        parent_id: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
        goal_id: row.get(13)?,
        skill_id: row.get(14)?,
    })
}

pub fn list(connection: &Connection, filter: &TaskFilter) -> Result<Vec<Task>, AppError> {
    let sql = format!(
        "SELECT {COLUMNS} FROM tasks
         WHERE (:status IS NULL OR status = :status)
           AND (:category_id IS NULL OR category_id = :category_id)
           AND (:due_before IS NULL OR (due_at IS NOT NULL AND due_at < :due_before))
         ORDER BY status = 'done', is_top_three DESC, due_at IS NULL, due_at,
                  priority DESC, id"
    );
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map(
        named_params! {
            ":status": filter.status.map(TaskStatus::as_str),
            ":category_id": filter.category_id,
            ":due_before": filter.due_before,
        },
        from_row,
    )?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn find(connection: &Connection, id: TaskId) -> Result<Option<Task>, AppError> {
    let sql = format!("SELECT {COLUMNS} FROM tasks WHERE id = ?1");
    Ok(connection.query_row(&sql, [id], from_row).optional()?)
}

pub fn insert(
    connection: &Connection,
    input: &TaskInput,
    now: DateTime<Utc>,
) -> Result<TaskId, AppError> {
    connection.execute(
        "INSERT INTO tasks (title, notes, category_id, kind, priority, due_at, parent_id,
                            goal_id, skill_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
        params![
            input.title,
            input.notes,
            input.category_id,
            input.kind.as_str(),
            input.priority,
            input.due_at,
            input.parent_id,
            input.goal_id,
            input.skill_id,
            now,
        ],
    )?;
    TaskId::try_from(connection.last_insert_rowid())
        .map_err(|_| AppError::InvalidInput("task id out of range".into()))
}

pub fn update(
    connection: &Connection,
    id: TaskId,
    input: &TaskInput,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    connection.execute(
        "UPDATE tasks SET title = ?2, notes = ?3, category_id = ?4, kind = ?5, priority = ?6,
                          due_at = ?7, parent_id = ?8, goal_id = ?9, skill_id = ?10,
                          updated_at = ?11
         WHERE id = ?1",
        params![
            id,
            input.title,
            input.notes,
            input.category_id,
            input.kind.as_str(),
            input.priority,
            input.due_at,
            input.parent_id,
            input.goal_id,
            input.skill_id,
            now,
        ],
    )?;
    Ok(())
}

pub fn set_status(
    connection: &Connection,
    id: TaskId,
    status: TaskStatus,
    completed_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    connection.execute(
        "UPDATE tasks SET status = ?2, completed_at = ?3, updated_at = ?4 WHERE id = ?1",
        params![id, status.as_str(), completed_at, now],
    )?;
    Ok(())
}

pub fn set_top_three(
    connection: &Connection,
    id: TaskId,
    on: bool,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    connection.execute(
        "UPDATE tasks SET is_top_three = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, on, now],
    )?;
    Ok(())
}

pub fn count_open_top_three(connection: &Connection) -> Result<u32, AppError> {
    Ok(connection.query_row(
        "SELECT count(*) FROM tasks WHERE is_top_three = 1 AND status = 'open'",
        [],
        |row| row.get(0),
    )?)
}

pub fn count_completed_between(
    connection: &Connection,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<u32, AppError> {
    Ok(connection.query_row(
        "SELECT count(*) FROM tasks
         WHERE status = 'done' AND completed_at >= ?1 AND completed_at < ?2",
        params![from, to],
        |row| row.get(0),
    )?)
}

pub fn next_for_today(
    connection: &Connection,
    end: DateTime<Utc>,
) -> Result<Option<Task>, AppError> {
    let sql = format!(
        "SELECT {COLUMNS} FROM tasks
         WHERE status = 'open' AND parent_id IS NULL
           AND (is_top_three = 1 OR (due_at IS NOT NULL AND due_at < ?1))
         ORDER BY is_top_three DESC, due_at IS NULL, due_at, priority DESC, id
         LIMIT 1"
    );
    Ok(connection.query_row(&sql, [end], from_row).optional()?)
}

pub fn count_open_for_today(connection: &Connection, end: DateTime<Utc>) -> Result<u32, AppError> {
    Ok(connection.query_row(
        "SELECT count(*) FROM tasks
         WHERE status = 'open' AND parent_id IS NULL
           AND (is_top_three = 1 OR (due_at IS NOT NULL AND due_at < ?1))",
        [end],
        |row| row.get(0),
    )?)
}

pub fn has_subtasks(connection: &Connection, id: TaskId) -> Result<bool, AppError> {
    Ok(connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM tasks WHERE parent_id = ?1)",
        [id],
        |row| row.get(0),
    )?)
}

pub fn delete(connection: &Connection, id: TaskId) -> Result<(), AppError> {
    connection.execute("DELETE FROM tasks WHERE id = ?1", [id])?;
    Ok(())
}

pub fn count_done(connection: &Connection, kind: TaskKind) -> Result<u32, AppError> {
    Ok(connection.query_row(
        "SELECT count(*) FROM tasks WHERE status = 'done' AND kind = ?1",
        [kind.as_str()],
        |row| row.get(0),
    )?)
}
