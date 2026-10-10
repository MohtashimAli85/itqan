use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::domain::health::{Habit, HabitId, HabitKind};
use itqan_core::db::enums::{column, to_text};
use itqan_core::error::AppError;

const COLUMNS: &str = "id, kind, name, target, enabled";

fn habit(row: &Row) -> rusqlite::Result<Habit> {
    Ok(Habit {
        id: row.get(0)?,
        kind: column(row, 1)?,
        name: row.get(2)?,
        target: row.get(3)?,
        enabled: row.get(4)?,
    })
}

pub fn find(connection: &Connection, id: HabitId) -> Result<Option<Habit>, AppError> {
    let sql = format!("SELECT {COLUMNS} FROM habits WHERE id = ?1");
    Ok(connection.query_row(&sql, [id], habit).optional()?)
}

pub fn first_of_kind(connection: &Connection, kind: HabitKind) -> Result<Option<Habit>, AppError> {
    let sql = format!("SELECT {COLUMNS} FROM habits WHERE kind = ?1 ORDER BY id LIMIT 1");
    Ok(connection
        .query_row(&sql, [to_text(&kind)?], habit)
        .optional()?)
}

pub fn list_of_kind(connection: &Connection, kind: HabitKind) -> Result<Vec<Habit>, AppError> {
    let sql = format!("SELECT {COLUMNS} FROM habits WHERE kind = ?1 ORDER BY id");
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map([to_text(&kind)?], habit)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn insert_habit(
    connection: &Connection,
    kind: HabitKind,
    name: &str,
    now: DateTime<Utc>,
) -> Result<HabitId, AppError> {
    connection.execute(
        "INSERT INTO habits (kind, name, created_at) VALUES (?1, ?2, ?3)",
        params![to_text(&kind)?, name, now],
    )?;
    HabitId::try_from(connection.last_insert_rowid())
        .map_err(|_| AppError::InvalidInput("habit id out of range".into()))
}

pub fn set_target(connection: &Connection, id: HabitId, target: u16) -> Result<(), AppError> {
    connection.execute(
        "UPDATE habits SET target = ?2 WHERE id = ?1",
        params![id, target],
    )?;
    Ok(())
}

pub fn delete(connection: &Connection, id: HabitId) -> Result<(), AppError> {
    connection.execute("DELETE FROM habits WHERE id = ?1", [id])?;
    Ok(())
}

pub fn insert_log(
    connection: &Connection,
    habit_id: HabitId,
    amount: u16,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    connection.execute(
        "INSERT INTO habit_logs (habit_id, amount, logged_at) VALUES (?1, ?2, ?3)",
        params![habit_id, amount, now],
    )?;
    Ok(())
}

pub fn sum_since(
    connection: &Connection,
    habit_id: HabitId,
    since: DateTime<Utc>,
) -> Result<u16, AppError> {
    Ok(connection.query_row(
        "SELECT coalesce(sum(amount), 0) FROM habit_logs WHERE habit_id = ?1 AND logged_at >= ?2",
        params![habit_id, since],
        |row| row.get(0),
    )?)
}
