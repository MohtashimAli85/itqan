use rusqlite::Connection;

use crate::error::AppError;

const MIGRATIONS: &[&str] = &[include_str!("../../migrations/0001_settings.sql")];

pub fn schema_version(connection: &Connection) -> Result<u32, AppError> {
    Ok(connection.query_row("PRAGMA user_version", [], |row| row.get(0))?)
}

pub fn run(connection: &mut Connection) -> Result<(), AppError> {
    let applied = schema_version(connection)? as usize;
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(applied) {
        let transaction = connection.transaction()?;
        transaction.execute_batch(sql)?;
        transaction.pragma_update(None, "user_version", index as i64 + 1)?;
        transaction.commit()?;
    }
    Ok(())
}
