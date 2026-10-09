use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::db::categories as repo;
use crate::error::AppError;

pub type CategoryId = i32;

pub const COLOURS: &[&str] = &["work", "personal", "health", "amber", "brand", "critical"];
const MAX_NAME_LENGTH: usize = 40;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: CategoryId,
    pub name: String,
    pub colour: String,
    pub icon: String,
    pub builtin: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CategoryInput {
    pub name: String,
    pub colour: String,
    pub icon: String,
}

pub fn list(connection: &Connection) -> Result<Vec<Category>, AppError> {
    repo::list(connection)
}

pub fn create(connection: &Connection, input: CategoryInput) -> Result<Category, AppError> {
    let name = input.name.trim().to_owned();
    if name.is_empty() || name.chars().count() > MAX_NAME_LENGTH {
        return Err(AppError::InvalidInput(format!(
            "a category name is 1 to {MAX_NAME_LENGTH} characters"
        )));
    }
    if !COLOURS.contains(&input.colour.as_str()) {
        return Err(AppError::InvalidInput("unknown colour".into()));
    }
    if repo::exists_by_name(connection, &name)? {
        return Err(AppError::InvalidInput(format!("{name} already exists")));
    }
    let id = repo::insert(connection, &name, &input.colour, input.icon.trim())?;
    repo::find(connection, id)?.ok_or_else(|| AppError::NotFound(format!("category {id}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_connection;

    fn input(name: &str, colour: &str) -> CategoryInput {
        CategoryInput {
            name: name.into(),
            colour: colour.into(),
            icon: "tag".into(),
        }
    }

    #[test]
    fn builtin_categories_are_seeded_in_order() {
        let connection = test_connection();
        let names: Vec<String> = list(&connection)
            .unwrap()
            .into_iter()
            .map(|c| c.name)
            .collect();

        assert_eq!(
            names,
            ["Work", "Personal", "Health", "Learning", "Building"]
        );
    }

    #[test]
    fn create_validates_name_colour_and_duplicates() {
        let connection = test_connection();

        assert!(create(&connection, input("Family", "personal")).is_ok());
        assert!(create(&connection, input("family", "personal")).is_err());
        assert!(create(&connection, input("Deen", "#ff0000")).is_err());
        assert!(create(&connection, input("  ", "work")).is_err());
    }
}
