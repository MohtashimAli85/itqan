use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::error::AppError;
use crate::focus::{FocusSession, FocusSessionId};
use itqan_contracts::TaskId;

const COLUMNS: &str = "id, task_id, started_at, planned_minutes, ended_at, completed";

fn from_row(row: &Row) -> rusqlite::Result<FocusSession> {
    Ok(FocusSession {
        id: row.get(0)?,
        task_id: row.get(1)?,
        started_at: row.get(2)?,
        planned_minutes: row.get(3)?,
        ended_at: row.get(4)?,
        completed: row.get(5)?,
    })
}

pub fn active(connection: &Connection) -> Result<Option<FocusSession>, AppError> {
    let sql = format!("SELECT {COLUMNS} FROM focus_sessions WHERE ended_at IS NULL LIMIT 1");
    Ok(connection.query_row(&sql, [], from_row).optional()?)
}

pub fn find(connection: &Connection, id: FocusSessionId) -> Result<Option<FocusSession>, AppError> {
    let sql = format!("SELECT {COLUMNS} FROM focus_sessions WHERE id = ?1");
    Ok(connection.query_row(&sql, [id], from_row).optional()?)
}

pub fn insert(
    connection: &Connection,
    task_id: Option<TaskId>,
    planned_minutes: u16,
    started_at: DateTime<Utc>,
) -> Result<FocusSessionId, AppError> {
    connection.execute(
        "INSERT INTO focus_sessions (task_id, planned_minutes, started_at) VALUES (?1, ?2, ?3)",
        params![task_id, planned_minutes, started_at],
    )?;
    FocusSessionId::try_from(connection.last_insert_rowid())
        .map_err(|_| AppError::InvalidInput("focus session id out of range".into()))
}

pub fn finish(
    connection: &Connection,
    id: FocusSessionId,
    ended_at: DateTime<Utc>,
    completed: bool,
) -> Result<(), AppError> {
    connection.execute(
        "UPDATE focus_sessions SET ended_at = ?2, completed = ?3 WHERE id = ?1",
        params![id, ended_at, completed],
    )?;
    Ok(())
}

pub fn completed_since(
    connection: &Connection,
    since: DateTime<Utc>,
) -> Result<Vec<(DateTime<Utc>, u16)>, AppError> {
    let mut statement = connection.prepare(
        "SELECT started_at, planned_minutes FROM focus_sessions
         WHERE completed = 1 AND started_at >= ?1",
    )?;
    let rows = statement.query_map([since], |row| Ok((row.get(0)?, row.get(1)?)))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn completed_count(connection: &Connection) -> Result<u32, AppError> {
    Ok(connection.query_row(
        "SELECT count(*) FROM focus_sessions WHERE completed = 1",
        [],
        |row| row.get(0),
    )?)
}
