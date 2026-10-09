use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::domain::categories::{Category, CategoryId};
use crate::error::AppError;

const COLUMNS: &str = "id, name, colour, icon, builtin";

fn from_row(row: &Row) -> rusqlite::Result<Category> {
    Ok(Category {
        id: row.get(0)?,
        name: row.get(1)?,
        colour: row.get(2)?,
        icon: row.get(3)?,
        builtin: row.get(4)?,
    })
}

pub fn list(connection: &Connection) -> Result<Vec<Category>, AppError> {
    let sql = format!("SELECT {COLUMNS} FROM categories ORDER BY sort_order, id");
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map([], from_row)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn find(connection: &Connection, id: CategoryId) -> Result<Option<Category>, AppError> {
    let sql = format!("SELECT {COLUMNS} FROM categories WHERE id = ?1");
    Ok(connection.query_row(&sql, [id], from_row).optional()?)
}

pub fn exists_by_name(connection: &Connection, name: &str) -> Result<bool, AppError> {
    Ok(connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM categories WHERE lower(name) = lower(?1))",
        [name],
        |row| row.get(0),
    )?)
}

pub fn insert(
    connection: &Connection,
    name: &str,
    colour: &str,
    icon: &str,
) -> Result<CategoryId, AppError> {
    connection.execute(
        "INSERT INTO categories (name, colour, icon, sort_order)
         VALUES (?1, ?2, ?3, (SELECT coalesce(max(sort_order), 0) + 1 FROM categories))",
        params![name, colour, icon],
    )?;
    CategoryId::try_from(connection.last_insert_rowid())
        .map_err(|_| AppError::InvalidInput("category id out of range".into()))
}
