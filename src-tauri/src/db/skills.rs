use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::domain::skills::{with_level, Skill, SkillId};
use crate::error::AppError;

fn skill(row: &Row) -> rusqlite::Result<Skill> {
    Ok(with_level(row.get(0)?, row.get(1)?, row.get(2)?))
}

pub fn list(connection: &Connection) -> Result<Vec<Skill>, AppError> {
    let mut statement =
        connection.prepare("SELECT id, name, xp FROM skills ORDER BY xp DESC, name")?;
    let rows = statement.query_map([], skill)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn find(connection: &Connection, id: SkillId) -> Result<Option<Skill>, AppError> {
    Ok(connection
        .query_row("SELECT id, name, xp FROM skills WHERE id = ?1", [id], skill)
        .optional()?)
}

pub fn find_by_name(connection: &Connection, name: &str) -> Result<Option<Skill>, AppError> {
    Ok(connection
        .query_row(
            "SELECT id, name, xp FROM skills WHERE name = ?1",
            [name],
            skill,
        )
        .optional()?)
}

pub fn insert(
    connection: &Connection,
    name: &str,
    now: DateTime<Utc>,
) -> Result<SkillId, AppError> {
    connection.execute(
        "INSERT INTO skills (name, created_at) VALUES (?1, ?2)",
        params![name, now],
    )?;
    SkillId::try_from(connection.last_insert_rowid())
        .map_err(|_| AppError::InvalidInput("skill id out of range".into()))
}

pub fn delete(connection: &Connection, id: SkillId) -> Result<(), AppError> {
    connection.execute("DELETE FROM skills WHERE id = ?1", [id])?;
    Ok(())
}
