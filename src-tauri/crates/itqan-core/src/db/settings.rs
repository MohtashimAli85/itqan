use rusqlite::{params, Connection, OptionalExtension};

use crate::error::AppError;

pub fn get(connection: &Connection, key: &str) -> Result<Option<String>, AppError> {
    Ok(connection
        .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
            row.get(0)
        })
        .optional()?)
}

pub fn set(connection: &Connection, key: &str, value: &str) -> Result<(), AppError> {
    connection.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT (key) DO UPDATE SET value = excluded.value,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
        params![key, value],
    )?;
    Ok(())
}
