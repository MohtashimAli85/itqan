use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};

use super::{Belief, BeliefId, BeliefStatus, NewBelief};
use crate::db::enums::{column, to_text};
use crate::error::AppError;

const COLUMNS: &str = "id, statement, kind, subject, value, strength, confidence, source,
                       evidence, status, created_at, confirmed_at, updated_at";

fn json_column<T: serde::de::DeserializeOwned>(row: &Row, index: usize) -> rusqlite::Result<T> {
    let text: String = row.get(index)?;
    serde_json::from_str(&text).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(index, rusqlite::types::Type::Text, error.into())
    })
}

fn from_row(row: &Row) -> rusqlite::Result<Belief> {
    let value: Option<String> = row.get(4)?;
    Ok(Belief {
        id: row.get(0)?,
        statement: row.get(1)?,
        kind: column(row, 2)?,
        subject: row.get(3)?,
        value: value.and_then(|text| serde_json::from_str(&text).ok()),
        strength: column(row, 5)?,
        confidence: row.get(6)?,
        source: column(row, 7)?,
        evidence: json_column(row, 8)?,
        status: column(row, 9)?,
        created_at: row.get(10)?,
        confirmed_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

pub fn with_status(connection: &Connection, status: BeliefStatus) -> Result<Vec<Belief>, AppError> {
    let sql = format!("SELECT {COLUMNS} FROM beliefs WHERE status = ?1 ORDER BY id");
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map([to_text(&status)?], from_row)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn active_for(connection: &Connection, subject: &str) -> Result<Option<Belief>, AppError> {
    let sql = format!("SELECT {COLUMNS} FROM beliefs WHERE status = 'active' AND subject = ?1");
    Ok(connection.query_row(&sql, [subject], from_row).optional()?)
}

pub fn insert(
    connection: &Connection,
    belief: &NewBelief,
    now: DateTime<Utc>,
) -> Result<BeliefId, AppError> {
    connection.execute(
        "INSERT INTO beliefs (statement, kind, subject, value, strength, confidence, source,
                              status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
        params![
            belief.statement,
            to_text(&belief.kind)?,
            belief.subject,
            belief
                .value
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?,
            to_text(&belief.strength)?,
            belief.confidence,
            to_text(&belief.source)?,
            to_text(&belief.status)?,
            now,
        ],
    )?;
    BeliefId::try_from(connection.last_insert_rowid())
        .map_err(|_| AppError::InvalidInput("belief id out of range".into()))
}

pub fn set_status(
    connection: &Connection,
    id: BeliefId,
    status: BeliefStatus,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    connection.execute(
        "UPDATE beliefs SET status = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, to_text(&status)?, now],
    )?;
    Ok(())
}
