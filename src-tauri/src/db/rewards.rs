use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

use crate::domain::rewards::{Award, Streak, XpSource};
use crate::domain::skills::SkillId;
use itqan_core::db::enums::{column, to_text};
use itqan_core::error::AppError;
use itqan_tasks::tasks::TaskKind;

pub fn insert_event(
    connection: &Connection,
    award: &Award,
    now: DateTime<Utc>,
) -> Result<bool, AppError> {
    let inserted = connection.execute(
        "INSERT OR IGNORE INTO xp_events (source, amount, task_id, skill_id, reference_id, at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            to_text(&award.source)?,
            award.amount,
            award.task_id,
            award.skill_id,
            award.reference_id,
            now,
        ],
    )?;
    Ok(inserted == 1)
}

pub fn total_xp(connection: &Connection) -> Result<u32, AppError> {
    Ok(connection.query_row(
        "SELECT coalesce(sum(amount), 0) FROM xp_events",
        [],
        |row| row.get(0),
    )?)
}

pub fn add_skill_xp(
    connection: &Connection,
    skill_id: SkillId,
    amount: u32,
) -> Result<(), AppError> {
    connection.execute(
        "UPDATE skills SET xp = xp + ?2 WHERE id = ?1",
        params![skill_id, amount],
    )?;
    Ok(())
}

pub fn streak(connection: &Connection) -> Result<Streak, AppError> {
    Ok(connection.query_row(
        "SELECT current, best, freezes, last_active_day FROM streaks WHERE id = 1",
        [],
        |row| {
            Ok(Streak {
                current: row.get(0)?,
                best: row.get(1)?,
                freezes: row.get(2)?,
                last_active_day: row.get(3)?,
            })
        },
    )?)
}

pub fn save_streak(connection: &Connection, streak: &Streak) -> Result<(), AppError> {
    connection.execute(
        "UPDATE streaks SET current = ?1, best = ?2, freezes = ?3, last_active_day = ?4 WHERE id = 1",
        params![streak.current, streak.best, streak.freezes, streak.last_active_day],
    )?;
    Ok(())
}

pub fn events_since(
    connection: &Connection,
    since: DateTime<Utc>,
) -> Result<Vec<(DateTime<Utc>, u32, XpSource)>, AppError> {
    let mut statement =
        connection.prepare("SELECT at, amount, source FROM xp_events WHERE at >= ?1")?;
    let rows = statement.query_map([since], |row| {
        Ok((row.get(0)?, row.get(1)?, column(row, 2)?))
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn focus_since(
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

pub fn count_done_tasks(connection: &Connection, kind: TaskKind) -> Result<u32, AppError> {
    Ok(connection.query_row(
        "SELECT count(*) FROM tasks WHERE status = 'done' AND kind = ?1",
        [kind.as_str()],
        |row| row.get(0),
    )?)
}

pub fn count_completed_focus(connection: &Connection) -> Result<u32, AppError> {
    Ok(connection.query_row(
        "SELECT count(*) FROM focus_sessions WHERE completed = 1",
        [],
        |row| row.get(0),
    )?)
}
