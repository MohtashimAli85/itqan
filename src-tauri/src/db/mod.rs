pub mod categories;
pub mod focus;
pub mod migrations;
pub mod prayer;
pub mod reminders;
pub mod settings;
pub mod tasks;
pub mod work_hours;

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::error::AppError;

pub struct Database(Mutex<Connection>);

impl Database {
    pub fn open(path: &Path) -> Result<Self, AppError> {
        let mut connection = Connection::open(path)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        migrations::run(&mut connection)?;
        Ok(Self(Mutex::new(connection)))
    }

    pub fn with<T>(
        &self,
        f: impl FnOnce(&Connection) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let connection = self.0.lock().map_err(|_| AppError::LockPoisoned)?;
        f(&connection)
    }
}

#[cfg(test)]
pub fn test_connection() -> Connection {
    let mut connection = Connection::open_in_memory().unwrap();
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .unwrap();
    migrations::run(&mut connection).unwrap();
    connection
}
