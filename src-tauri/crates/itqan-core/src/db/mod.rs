pub mod enums;
pub mod migrations;
pub mod settings;

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::error::AppError;
use crate::module::Module;

pub struct Database(Mutex<Connection>);

impl Database {
    pub fn open(path: &Path, modules: &[&dyn Module]) -> Result<Self, AppError> {
        let mut connection = Connection::open(path)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        migrations::run(&mut connection)?;
        migrations::run_modules(&mut connection, modules)?;
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

#[cfg(any(test, feature = "test-support"))]
pub fn test_connection() -> Connection {
    test_connection_with(&[])
}

#[cfg(any(test, feature = "test-support"))]
#[allow(clippy::unwrap_used)]
pub fn test_connection_with(modules: &[&dyn Module]) -> Connection {
    let mut connection = Connection::open_in_memory().unwrap();
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .unwrap();
    migrations::run(&mut connection).unwrap();
    migrations::run_modules(&mut connection, modules).unwrap();
    connection
}
