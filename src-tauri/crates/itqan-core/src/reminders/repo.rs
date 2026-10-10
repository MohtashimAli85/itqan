use chrono::{DateTime, Utc};
use itqan_contracts::{HabitId, TaskId};
use rusqlite::{params, Connection, OptionalExtension, Row};

use super::ReminderId;
use crate::error::AppError;

const SELECT: &str = "SELECT r.id, r.task_id, r.habit_id, r.title, r.anchor_at, r.next_at,
                             r.rrule, r.timezone, r.critical, r.snoozed_until, r.last_fired_at
                      FROM reminders r";

const DUE_AT: &str =
    "min(coalesce(r.snoozed_until, r.next_at), coalesce(r.next_at, r.snoozed_until))";

#[derive(Debug, Clone, PartialEq)]
pub struct ReminderRow {
    pub id: ReminderId,
    pub task_id: Option<TaskId>,
    pub habit_id: Option<HabitId>,
    pub title: Option<String>,
    pub anchor_at: DateTime<Utc>,
    pub next_at: Option<DateTime<Utc>>,
    pub rrule: Option<String>,
    pub timezone: String,
    pub critical: bool,
    pub snoozed_until: Option<DateTime<Utc>>,
    pub last_fired_at: Option<DateTime<Utc>>,
}

pub struct NewReminder<'a> {
    pub task_id: Option<TaskId>,
    pub title: Option<&'a str>,
    pub anchor_at: DateTime<Utc>,
    pub next_at: Option<DateTime<Utc>>,
    pub rrule: Option<&'a str>,
    pub timezone: &'a str,
    pub critical: bool,
    pub created_at: DateTime<Utc>,
}

fn from_row(row: &Row) -> rusqlite::Result<ReminderRow> {
    Ok(ReminderRow {
        id: row.get(0)?,
        task_id: row.get(1)?,
        habit_id: row.get(2)?,
        title: row.get(3)?,
        anchor_at: row.get(4)?,
        next_at: row.get(5)?,
        rrule: row.get(6)?,
        timezone: row.get(7)?,
        critical: row.get(8)?,
        snoozed_until: row.get(9)?,
        last_fired_at: row.get(10)?,
    })
}

fn rows(
    connection: &Connection,
    sql: &str,
    params: impl rusqlite::Params,
) -> Result<Vec<ReminderRow>, AppError> {
    let mut statement = connection.prepare(sql)?;
    let rows = statement.query_map(params, from_row)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn insert(connection: &Connection, reminder: &NewReminder) -> Result<ReminderId, AppError> {
    connection.execute(
        "INSERT INTO reminders (task_id, title, anchor_at, next_at, rrule, timezone, critical,
                                created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            reminder.task_id,
            reminder.title,
            reminder.anchor_at,
            reminder.next_at,
            reminder.rrule,
            reminder.timezone,
            reminder.critical,
            reminder.created_at,
        ],
    )?;
    ReminderId::try_from(connection.last_insert_rowid())
        .map_err(|_| AppError::InvalidInput("reminder id out of range".into()))
}

pub fn find(connection: &Connection, id: ReminderId) -> Result<Option<ReminderRow>, AppError> {
    let sql = format!("{SELECT} WHERE r.id = ?1");
    Ok(connection.query_row(&sql, [id], from_row).optional()?)
}

pub fn list_for_task(
    connection: &Connection,
    task_id: TaskId,
) -> Result<Vec<ReminderRow>, AppError> {
    rows(
        connection,
        &format!("{SELECT} WHERE r.task_id = ?1 ORDER BY r.next_at"),
        [task_id],
    )
}

pub fn list_for_habit(
    connection: &Connection,
    habit_id: HabitId,
) -> Result<Vec<ReminderRow>, AppError> {
    rows(
        connection,
        &format!("{SELECT} WHERE r.habit_id = ?1 ORDER BY r.anchor_at"),
        [habit_id],
    )
}

pub fn set_habit(
    connection: &Connection,
    id: ReminderId,
    habit_id: HabitId,
) -> Result<(), AppError> {
    connection.execute(
        "UPDATE reminders SET habit_id = ?2 WHERE id = ?1",
        params![id, habit_id],
    )?;
    Ok(())
}

pub fn due(connection: &Connection, now: DateTime<Utc>) -> Result<Vec<ReminderRow>, AppError> {
    rows(
        connection,
        &format!(
            "{SELECT} WHERE {DUE_AT} IS NOT NULL AND {DUE_AT} <= ?1
             ORDER BY r.critical DESC, {DUE_AT}"
        ),
        [now],
    )
}

pub fn pending(connection: &Connection) -> Result<Vec<(ReminderRow, DateTime<Utc>)>, AppError> {
    let sql = format!(
        "SELECT r.id, r.task_id, r.habit_id, r.title, r.anchor_at, r.next_at, r.rrule,
                r.timezone, r.critical, r.snoozed_until, r.last_fired_at, {DUE_AT}
         FROM reminders r WHERE {DUE_AT} IS NOT NULL ORDER BY {DUE_AT}"
    );
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map([], |row| Ok((from_row(row)?, row.get(11)?)))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn update_schedule(
    connection: &Connection,
    id: ReminderId,
    next_at: Option<DateTime<Utc>>,
    snoozed_until: Option<DateTime<Utc>>,
    last_fired_at: Option<DateTime<Utc>>,
) -> Result<(), AppError> {
    connection.execute(
        "UPDATE reminders SET next_at = ?2, snoozed_until = ?3, last_fired_at = ?4 WHERE id = ?1",
        params![id, next_at, snoozed_until, last_fired_at],
    )?;
    Ok(())
}

pub fn delete(connection: &Connection, id: ReminderId) -> Result<(), AppError> {
    connection.execute("DELETE FROM reminders WHERE id = ?1", [id])?;
    Ok(())
}
