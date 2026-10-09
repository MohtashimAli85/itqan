use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};

use super::enums::{column, optional_column, to_text};
use crate::domain::nudges::{NewNudge, Nudge, NudgeId, Outcome, Priority};
use crate::error::AppError;

fn nudge(row: &Row) -> rusqlite::Result<Nudge> {
    Ok(Nudge {
        id: row.get(0)?,
        agent: column(row, 1)?,
        kind: row.get(2)?,
        priority: column(row, 3)?,
        text: row.get(4)?,
        style: row.get(5)?,
        mode: row.get(6)?,
        fired_at: row.get(7)?,
        outcome: optional_column(row, 8)?,
        action: row.get(9)?,
        outcome_at: row.get(10)?,
    })
}

pub fn insert(connection: &Connection, nudge: &NewNudge) -> Result<NudgeId, AppError> {
    connection.execute(
        "INSERT INTO nudges (agent, kind, priority, text, style, mode, fired_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            to_text(&nudge.agent)?,
            nudge.kind,
            to_text(&nudge.priority)?,
            nudge.text,
            nudge.style,
            nudge.mode,
            nudge.fired_at,
        ],
    )?;
    NudgeId::try_from(connection.last_insert_rowid())
        .map_err(|_| AppError::InvalidInput("nudge id out of range".into()))
}

pub fn set_outcome(
    connection: &Connection,
    id: NudgeId,
    outcome: Outcome,
    action: Option<&str>,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    connection.execute(
        "UPDATE nudges SET outcome = ?2, action = ?3, outcome_at = ?4
         WHERE id = ?1 AND outcome IS NULL",
        params![id, to_text(&outcome)?, action, now],
    )?;
    Ok(())
}

pub fn count_non_critical_since(
    connection: &Connection,
    since: DateTime<Utc>,
) -> Result<u32, AppError> {
    Ok(connection.query_row(
        "SELECT count(*) FROM nudges WHERE fired_at >= ?1 AND priority != ?2",
        params![since, to_text(&Priority::Critical)?],
        |row| row.get(0),
    )?)
}

pub fn unanswered_before(
    connection: &Connection,
    before: DateTime<Utc>,
) -> Result<Vec<NudgeId>, AppError> {
    let mut statement = connection.prepare(
        "SELECT id FROM nudges WHERE outcome IS NULL AND fired_at < ?1 AND priority != ?2",
    )?;
    let rows = statement.query_map(params![before, to_text(&Priority::Critical)?], |row| {
        row.get(0)
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn last_fired(connection: &Connection, kind: &str) -> Result<Option<DateTime<Utc>>, AppError> {
    Ok(connection.query_row(
        "SELECT max(fired_at) FROM nudges WHERE kind = ?1",
        [kind],
        |row| row.get(0),
    )?)
}

pub fn recent(connection: &Connection, limit: u32) -> Result<Vec<Nudge>, AppError> {
    let mut statement = connection.prepare(
        "SELECT id, agent, kind, priority, text, style, mode, fired_at, outcome, action, outcome_at
         FROM nudges ORDER BY fired_at DESC, id DESC LIMIT ?1",
    )?;
    let rows = statement.query_map([limit], nudge)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}
