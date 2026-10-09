use rusqlite::Connection;

use crate::error::AppError;

const MIGRATIONS: &[&str] = &[
    include_str!("../../migrations/0001_settings.sql"),
    include_str!("../../migrations/0002_tasks.sql"),
];

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_applies_every_migration() {
        let mut connection = Connection::open_in_memory().unwrap();

        run(&mut connection).unwrap();

        assert_eq!(
            schema_version(&connection).unwrap() as usize,
            MIGRATIONS.len()
        );
        connection
            .execute(
                "INSERT INTO settings (key, value) VALUES ('theme', 'dark')",
                [],
            )
            .unwrap();
    }

    #[test]
    fn run_is_idempotent() {
        let mut connection = Connection::open_in_memory().unwrap();

        run(&mut connection).unwrap();
        run(&mut connection).unwrap();

        assert_eq!(
            schema_version(&connection).unwrap() as usize,
            MIGRATIONS.len()
        );
    }
}
