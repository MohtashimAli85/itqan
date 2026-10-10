use rusqlite::{params, Connection};

use crate::domain::modes::WorkDay;
use itqan_core::error::AppError;

pub fn list(connection: &Connection) -> Result<Vec<WorkDay>, AppError> {
    let mut statement = connection.prepare(
        "SELECT weekday, enabled, start_minute, end_minute FROM work_hours ORDER BY weekday",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(WorkDay {
            weekday: row.get(0)?,
            enabled: row.get(1)?,
            start_minute: row.get(2)?,
            end_minute: row.get(3)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn save(connection: &Connection, days: &[WorkDay]) -> Result<(), AppError> {
    let transaction = connection.unchecked_transaction()?;
    for day in days {
        transaction.execute(
            "UPDATE work_hours SET enabled = ?2, start_minute = ?3, end_minute = ?4
             WHERE weekday = ?1",
            params![day.weekday, day.enabled, day.start_minute, day.end_minute],
        )?;
    }
    transaction.commit()?;
    Ok(())
}
