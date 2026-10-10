pub mod categories;
pub mod commands;
pub mod quick_add;
mod repo;
pub mod targets;
pub mod tasks;

use chrono::{DateTime, Utc};
use itqan_contracts::TaskKind;
use itqan_core::error::AppError;
use itqan_core::module::{Migration, Module};
use itqan_core::ports::{Ports, TaskStats, TodayCounts};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize, Type, tauri_specta::Event)]
pub struct TasksChanged;

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "tasks_categories",
    sql: include_str!("../migrations/0001_tasks_categories.sql"),
}];

pub struct TasksModule;

impl Module for TasksModule {
    fn id(&self) -> &'static str {
        "tasks"
    }

    fn migrations(&self) -> &'static [Migration] {
        MIGRATIONS
    }

    fn setup(&self, app: &AppHandle) -> Result<(), AppError> {
        register(&app.state::<Ports>())
    }

    fn teardown(&self, app: &AppHandle) -> Result<(), AppError> {
        unregister(&app.state::<Ports>())
    }
}

fn register(ports: &Ports) -> Result<(), AppError> {
    targets::register(ports)?;
    ports.set_task_stats(Stats)
}

fn unregister(ports: &Ports) -> Result<(), AppError> {
    ports.clear_reminder_target(itqan_core::ports::ReminderTargetKind::Task)?;
    ports.clear_task_stats()
}

struct Stats;

impl TaskStats for Stats {
    fn today_counts(
        &self,
        connection: &Connection,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<TodayCounts, AppError> {
        let (done, open) = tasks::today_counts(connection, start, end)?;
        Ok(TodayCounts { done, open })
    }

    fn next_for_today(
        &self,
        connection: &Connection,
        end: DateTime<Utc>,
    ) -> Result<Option<String>, AppError> {
        Ok(tasks::next_for_today(connection, end)?.map(|task| task.title))
    }

    fn completed_between(
        &self,
        connection: &Connection,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<u32, AppError> {
        tasks::completed_between(connection, from, to)
    }

    fn done_count(&self, connection: &Connection, kind: TaskKind) -> Result<u32, AppError> {
        tasks::done_count(connection, kind)
    }
}

#[cfg(test)]
fn test_connection() -> Connection {
    itqan_core::db::test_connection_with(&[&TasksModule])
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use itqan_core::db::migrations;
    use itqan_core::ports::ReminderTargetKind;

    use super::*;

    #[test]
    fn existing_categories_and_task_links_survive_the_rename() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .unwrap();
        migrations::run(&mut connection).unwrap();
        let at = Utc.with_ymd_and_hms(2026, 10, 10, 9, 0, 0).unwrap();
        connection
            .execute(
                "INSERT INTO categories (name, colour, icon, sort_order) VALUES ('Itqan', 'orange', 'code', 9)",
                [],
            )
            .unwrap();
        let category = connection.last_insert_rowid();
        connection
            .execute(
                "INSERT INTO tasks (title, category_id, created_at, updated_at) VALUES ('Ship 2.0.6', ?1, ?2, ?2)",
                rusqlite::params![category, at],
            )
            .unwrap();
        let task = connection.last_insert_rowid();

        let snapshot = |connection: &Connection, table: &str| -> Vec<(String, bool, i64)> {
            let mut statement = connection
                .prepare(&format!(
                    "SELECT name, builtin, sort_order FROM {table} ORDER BY id"
                ))
                .unwrap();
            statement
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap()
        };
        let before = snapshot(&connection, "categories");

        migrations::run_modules(&mut connection, &[&TasksModule]).unwrap();

        assert_eq!(snapshot(&connection, "tasks_categories"), before);
        assert_eq!(before.iter().filter(|(_, builtin, _)| *builtin).count(), 5);

        let names: Vec<String> = categories::list(&connection)
            .unwrap()
            .into_iter()
            .map(|category| category.name)
            .collect();
        assert!(names.contains(&"Itqan".to_owned()));
        assert_eq!(
            tasks::get(&connection, task as i32).unwrap().category_id,
            Some(category as i32)
        );
        connection
            .execute("DELETE FROM tasks_categories WHERE id = ?1", [category])
            .unwrap();
        assert_eq!(
            tasks::get(&connection, task as i32).unwrap().category_id,
            None
        );
        let old: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE name = 'categories'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(old, 0);
    }

    #[test]
    fn done_count_counts_done_tasks_of_one_kind() {
        let connection = test_connection();
        let at = Utc.with_ymd_and_hms(2026, 10, 10, 9, 0, 0).unwrap();
        for (title, kind, done) in [
            ("Ship it", TaskKind::Output, true),
            ("Ship more", TaskKind::Output, true),
            ("Draft", TaskKind::Output, false),
            ("Read", TaskKind::Learning, true),
        ] {
            let task = tasks::create(
                &connection,
                tasks::TaskInput {
                    title: title.into(),
                    kind,
                    ..tasks::TaskInput::default()
                },
                at,
            )
            .unwrap();
            if done {
                tasks::set_status(&connection, task.id, tasks::TaskStatus::Done, at).unwrap();
            }
        }

        assert_eq!(Stats.done_count(&connection, TaskKind::Output).unwrap(), 2);
        assert_eq!(
            Stats.done_count(&connection, TaskKind::Learning).unwrap(),
            1
        );
        assert_eq!(Stats.done_count(&connection, TaskKind::Habit).unwrap(), 0);
    }

    #[test]
    fn setup_registers_the_task_target_and_stats() {
        let ports = Ports::default();

        register(&ports).unwrap();

        assert!(ports
            .reminder_target(ReminderTargetKind::Task)
            .unwrap()
            .is_some());
        assert!(ports.set_task_stats(Stats).is_err());

        unregister(&ports).unwrap();
        assert!(ports
            .reminder_target(ReminderTargetKind::Task)
            .unwrap()
            .is_none());
        register(&ports).unwrap();
    }
}
