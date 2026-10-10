use std::collections::HashSet;

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
    include_str!("../../migrations/0011_modules.sql"),
    include_str!("../../migrations/0012_beliefs.sql"),
];

pub fn schema_version(connection: &Connection) -> Result<u32, AppError> {
    Ok(connection.query_row("PRAGMA user_version", [], |row| row.get(0))?)
}

pub fn run(connection: &mut Connection) -> Result<(), AppError> {
    run_until(connection, MIGRATIONS.len())
}

pub fn run_until(connection: &mut Connection, target: usize) -> Result<(), AppError> {
    let applied = schema_version(connection)? as usize;
    if applied > MIGRATIONS.len() {
        return Err(AppError::NewerSchema {
            found: applied,
            known: MIGRATIONS.len(),
        });
    }
    for (index, sql) in MIGRATIONS.iter().enumerate().take(target).skip(applied) {
        let transaction = connection.transaction()?;
        transaction.execute_batch(sql)?;
        #[allow(clippy::cast_possible_wrap)]
        transaction.pragma_update(None, "user_version", index as i64 + 1)?;
        transaction.commit()?;
    }
    Ok(())
}

pub fn run_modules(connection: &mut Connection, modules: &[&dyn Module]) -> Result<(), AppError> {
    validate(modules)?;
    for module in modules {
        for migration in module.migrations() {
            let applied: bool = connection.query_row(
                "SELECT EXISTS (SELECT 1 FROM schema_migrations WHERE module = ?1 AND version = ?2)",
                params![module.id(), migration.version],
                |row| row.get(0),
            )?;
            if applied {
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

fn validate(modules: &[&dyn Module]) -> Result<(), AppError> {
    let mut ids = HashSet::new();
    for module in modules {
        if !ids.insert(module.id()) {
            return Err(AppError::InvalidInput(format!(
                "module {} is registered twice",
                module.id()
            )));
        }
        let increasing = module
            .migrations()
            .iter()
            .try_fold(0, |previous, migration| {
                (migration.version > previous).then_some(migration.version)
            })
            .is_some();
        if !increasing {
            return Err(AppError::InvalidInput(format!(
                "{} migrations must be numbered 1, 2, 3, … in increasing order",
                module.id()
            )));
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

    #[test]
    fn a_database_from_a_newer_build_is_refused() {
        let mut connection = Connection::open_in_memory().unwrap();
        run(&mut connection).unwrap();
        connection
            .pragma_update(
                None,
                "user_version",
                i64::try_from(MIGRATIONS.len()).unwrap() + 1,
            )
            .unwrap();

        assert!(matches!(
            run(&mut connection),
            Err(AppError::NewerSchema { .. })
        ));
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

    fn tables(connection: &Connection) -> i64 {
        connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE name = 'notes_items'",
                [],
                |row| row.get(0),
            )
            .unwrap()
    }

    #[test]
    fn badly_numbered_migrations_are_rejected_before_anything_runs() {
        const OUT_OF_ORDER: &[Migration] = &[NOTES[0], NOTES[1], NOTES[0]];
        let mut connection = Connection::open_in_memory().unwrap();
        run(&mut connection).unwrap();

        let error = run_modules(&mut connection, &[&Notes(OUT_OF_ORDER)]).unwrap_err();

        assert!(error.to_string().contains("increasing order"));
        assert_eq!(tables(&connection), 0);
        assert!(recorded(&connection).is_empty());
    }

    #[test]
    fn duplicate_module_ids_are_rejected() {
        let mut connection = Connection::open_in_memory().unwrap();
        run(&mut connection).unwrap();

        let error = run_modules(&mut connection, &[&Notes(NOTES), &Notes(NOTES)]).unwrap_err();

        assert!(error.to_string().contains("registered twice"));
        assert_eq!(tables(&connection), 0);
    }

    #[test]
    fn a_missing_earlier_version_still_runs() {
        const LATER_FIRST: &[Migration] = &[Migration {
            version: 2,
            name: "notes",
            sql: "CREATE TABLE notes_items (id INTEGER PRIMARY KEY) STRICT;",
        }];
        const EARLIER: &[Migration] = &[
            Migration {
                version: 1,
                name: "notes_tags",
                sql: "CREATE TABLE notes_tags (id INTEGER PRIMARY KEY) STRICT;",
            },
            LATER_FIRST[0],
        ];
        let mut connection = Connection::open_in_memory().unwrap();
        run(&mut connection).unwrap();

        run_modules(&mut connection, &[&Notes(LATER_FIRST)]).unwrap();
        run_modules(&mut connection, &[&Notes(EARLIER)]).unwrap();

        assert_eq!(recorded(&connection).len(), 2);
    }
}
