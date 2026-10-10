use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::db::skills as repo;
use crate::domain::levels::{self, LevelProgress};
use itqan_core::error::AppError;

pub type SkillId = i32;

const MAX_NAME_LENGTH: usize = 60;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub id: SkillId,
    pub name: String,
    pub xp: u32,
    pub level: LevelProgress,
}

pub fn list(connection: &Connection) -> Result<Vec<Skill>, AppError> {
    repo::list(connection)
}

pub fn create(connection: &Connection, name: &str, now: DateTime<Utc>) -> Result<Skill, AppError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME_LENGTH {
        return Err(AppError::InvalidInput(format!(
            "a skill name is 1 to {MAX_NAME_LENGTH} characters"
        )));
    }
    if repo::find_by_name(connection, name)?.is_some() {
        return Err(AppError::InvalidInput(format!("{name} is already a skill")));
    }
    let id = repo::insert(connection, name, now)?;
    repo::find(connection, id)?.ok_or_else(|| AppError::NotFound(format!("skill {id}")))
}

pub fn delete(connection: &Connection, id: SkillId) -> Result<(), AppError> {
    repo::delete(connection, id)
}

pub fn with_level(id: SkillId, name: String, xp: u32) -> Skill {
    Skill {
        id,
        name,
        xp,
        level: levels::progress(xp),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itqan_core::db::test_connection;

    #[test]
    fn skills_are_unique_regardless_of_case() {
        let connection = test_connection();
        let rust = create(&connection, "Rust", Utc::now()).unwrap();
        assert_eq!(rust.level.level, 1);
        assert!(create(&connection, "rust", Utc::now()).is_err());

        delete(&connection, rust.id).unwrap();
        assert!(list(&connection).unwrap().is_empty());
    }
}
