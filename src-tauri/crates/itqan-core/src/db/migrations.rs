use rusqlite::{params, Connection};

use crate::error::AppError;
use crate::module::Module;

const MIGRATIONS: &[&str] = &[
    include_str!("../../migrations/0001_settings.sql"),
    include_str!("../../migrations/0002_tasks.sql"),
    include_str!("../../migrations/0003_reminders.sql"),
    include_str!("../../migrations/0004_modes.sql"),
    include_str!("../../migrations/0005_profile.sql"),
    include_str!("../../migrations/0006_goals.sql"),
    include_str!("../../migrations/0007_nudges.sql"),
    include_str!("../../migrations/0008_health.sql"),
    include_str!("../../migrations/0009_rewards.sql"),
    include_str!("../../migrations/0010_schema_migrations.sql"),
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

pub fn run_modules(connection: &mut Connection, modules: &[&dyn Module]) -> Result<(), AppError> {
    for module in modules {
        let applied: u32 = connection.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations WHERE module = ?1",
            [module.id()],
            |row| row.get(0),
        )?;
        let mut previous = 0;
        for migration in module.migrations() {
            if migration.version <= previous {
                return Err(AppError::InvalidInput(format!(
                    "{} migrations must be numbered in increasing order",
                    module.id()
                )));
            }
            previous = migration.version;
            if migration.version <= applied {
                continue;
            }
            let transaction = connection.transaction()?;
            transaction.execute_batch(migration.sql)?;
            transaction.execute(
                "INSERT INTO schema_migrations (module, version, name) VALUES (?1, ?2, ?3)",
                params![module.id(), migration.version, migration.name],
            )?;
            transaction.commit()?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::Migration;

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

    struct Notes(&'static [Migration]);

    impl Module for Notes {
        fn id(&self) -> &'static str {
            "notes"
        }

        fn migrations(&self) -> &'static [Migration] {
            self.0
        }
    }

    const NOTES: &[Migration] = &[
        Migration {
            version: 1,
            name: "notes",
            sql: "CREATE TABLE notes_items (id INTEGER PRIMARY KEY, body TEXT NOT NULL) STRICT;",
        },
        Migration {
            version: 2,
            name: "notes_pinned",
            sql: "ALTER TABLE notes_items ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0;",
        },
    ];

    fn recorded(connection: &Connection) -> Vec<(String, u32, String)> {
        let mut statement = connection
            .prepare("SELECT module, version, name FROM schema_migrations ORDER BY version")
            .unwrap();
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    }

    #[test]
    fn module_migrations_run_once_in_order_and_are_recorded() {
        let mut connection = Connection::open_in_memory().unwrap();
        run(&mut connection).unwrap();

        run_modules(&mut connection, &[&Notes(&NOTES[..1])]).unwrap();
        connection
            .execute("INSERT INTO notes_items (body) VALUES ('keep me')", [])
            .unwrap();
        run_modules(&mut connection, &[&Notes(NOTES)]).unwrap();
        run_modules(&mut connection, &[&Notes(NOTES)]).unwrap();

        let (body, pinned): (String, i64) = connection
            .query_row("SELECT body, pinned FROM notes_items", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .unwrap();
        assert_eq!((body.as_str(), pinned), ("keep me", 0));
        assert_eq!(
            recorded(&connection),
            vec![
                ("notes".into(), 1, "notes".into()),
                ("notes".into(), 2, "notes_pinned".into())
            ]
        );
    }

    #[test]
    fn module_migrations_must_increase() {
        const OUT_OF_ORDER: &[Migration] = &[NOTES[1], NOTES[0]];
        let mut connection = Connection::open_in_memory().unwrap();
        run(&mut connection).unwrap();

        assert!(run_modules(&mut connection, &[&Notes(OUT_OF_ORDER)]).is_err());
    }
}
