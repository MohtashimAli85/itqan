use rusqlite::{params, Connection};

use super::{CoachStyle, Profile};
use crate::error::AppError;

fn coach_style(value: &str) -> CoachStyle {
    match value {
        "manager" => CoachStyle::Manager,
        "trainer" => CoachStyle::Trainer,
        _ => CoachStyle::Mentor,
    }
}

fn coach_style_text(style: CoachStyle) -> &'static str {
    match style {
        CoachStyle::Mentor => "mentor",
        CoachStyle::Manager => "manager",
        CoachStyle::Trainer => "trainer",
    }
}

pub fn get(connection: &Connection) -> Result<Profile, AppError> {
    let (profile, motivators) = connection.query_row(
        "SELECT name, motivators, situation, free_hours_per_week, age, coach_style,
                family_start_minute, family_end_minute, updated_at
         FROM profile WHERE id = 1",
        [],
        |row| {
            let style: String = row.get(5)?;
            let motivators: String = row.get(1)?;
            Ok((
                Profile {
                    name: row.get(0)?,
                    motivators: Vec::new(),
                    situation: row.get(2)?,
                    free_hours_per_week: row.get(3)?,
                    age: row.get(4)?,
                    coach_style: coach_style(&style),
                    family_start_minute: row.get(6)?,
                    family_end_minute: row.get(7)?,
                    updated_at: row.get(8)?,
                },
                motivators,
            ))
        },
    )?;
    Ok(Profile {
        motivators: serde_json::from_str(&motivators)?,
        ..profile
    })
}

pub fn save(connection: &Connection, profile: &Profile) -> Result<(), AppError> {
    connection.execute(
        "UPDATE profile SET name = ?1, motivators = ?2, situation = ?3, free_hours_per_week = ?4,
                age = ?5, coach_style = ?6, family_start_minute = ?7, family_end_minute = ?8,
                updated_at = ?9
         WHERE id = 1",
        params![
            profile.name,
            serde_json::to_string(&profile.motivators)?,
            profile.situation,
            profile.free_hours_per_week,
            profile.age,
            coach_style_text(profile.coach_style),
            profile.family_start_minute,
            profile.family_end_minute,
            profile.updated_at,
        ],
    )?;
    Ok(())
}
