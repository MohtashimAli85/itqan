use rusqlite::{params, Connection};

use super::Profile;
use crate::error::AppError;

pub fn get(connection: &Connection) -> Result<Profile, AppError> {
    Ok(connection.query_row(
        "SELECT name, situation, free_hours_per_week, age, family_start_minute,
                family_end_minute, updated_at
         FROM profile WHERE id = 1",
        [],
        |row| {
            Ok(Profile {
                name: row.get(0)?,
                situation: row.get(1)?,
                free_hours_per_week: row.get(2)?,
                age: row.get(3)?,
                family_start_minute: row.get(4)?,
                family_end_minute: row.get(5)?,
                updated_at: row.get(6)?,
                ..Profile::default()
            })
        },
    )?)
}

pub fn save(connection: &Connection, profile: &Profile) -> Result<(), AppError> {
    connection.execute(
        "UPDATE profile SET name = ?1, situation = ?2, free_hours_per_week = ?3, age = ?4,
                family_start_minute = ?5, family_end_minute = ?6, updated_at = ?7
         WHERE id = 1",
        params![
            profile.name,
            profile.situation,
            profile.free_hours_per_week,
            profile.age,
            profile.family_start_minute,
            profile.family_end_minute,
            profile.updated_at,
        ],
    )?;
    Ok(())
}
