pub mod agent;
pub mod commands;
pub mod habits;
mod repo;

use std::collections::HashMap;

use itqan_core::actions::ActionRouter;
use itqan_core::bus::Subscriber;
use itqan_core::error::AppError;
use itqan_core::module::{Migration, Module};
use itqan_core::ports::{Ports, ReminderTarget, ReminderTargetKind, TargetInfo};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize, Type, tauri_specta::Event)]
pub struct HealthChanged;

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "health_tables",
    sql: include_str!("../migrations/0001_health_tables.sql"),
}];

pub struct HealthModule;

impl Module for HealthModule {
    fn id(&self) -> &'static str {
        "health"
    }

    fn migrations(&self) -> &'static [Migration] {
        MIGRATIONS
    }

    fn setup(&self, app: &AppHandle) -> Result<(), AppError> {
        app.state::<Ports>()
            .set_reminder_target(ReminderTargetKind::Habit, HabitReminders)?;
        app.state::<ActionRouter>()
            .register(agent::HABIT_ACTIONS, agent::HabitActions)
    }

    fn subscribers(&self) -> Vec<Box<dyn Subscriber>> {
        vec![Box::new(agent::HealthAgent)]
    }
}

struct HabitReminders;

impl ReminderTarget for HabitReminders {
    fn describe(&self, _: &Connection, ids: &[i32]) -> Result<HashMap<i32, TargetInfo>, AppError> {
        Ok(ids
            .iter()
            .map(|id| {
                (
                    *id,
                    TargetInfo {
                        title: None,
                        active: true,
                    },
                )
            })
            .collect())
    }

    fn complete(&self, _: &AppHandle, _: i32) -> Result<(), AppError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use itqan_contracts::HabitKind;
    use itqan_core::db::migrations;

    use super::*;

    #[test]
    fn existing_habits_and_logs_survive_the_rename() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .unwrap();
        migrations::run(&mut connection).unwrap();
        let at = Utc.with_ymd_and_hms(2026, 10, 10, 9, 0, 0).unwrap();
        connection
            .execute(
                "INSERT INTO habits (kind, name, created_at) VALUES ('medicine', 'Vitamin D', ?1)",
                [at],
            )
            .unwrap();
        let medicine = connection.last_insert_rowid();
        connection
            .execute(
                "INSERT INTO habit_logs (habit_id, amount, logged_at) VALUES (?1, 1, ?2)",
                rusqlite::params![medicine, at],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO reminders (title, anchor_at, timezone, created_at, habit_id)
                 VALUES ('Vitamin D', ?1, 'UTC', ?1, ?2)",
                rusqlite::params![at, medicine],
            )
            .unwrap();

        migrations::run_modules(&mut connection, &[&HealthModule]).unwrap();

        let water = habits::habit(&connection, HabitKind::Water).unwrap();
        assert_eq!(water.target, Some(8));
        assert_eq!(
            habits::logged_since(&connection, HabitKind::Medicine, at).unwrap(),
            1
        );
        let reminders =
            itqan_core::reminders::list_for_habit(&connection, medicine as i32).unwrap();
        assert_eq!(reminders.len(), 1);
        connection
            .execute("DELETE FROM health_habits WHERE id = ?1", [medicine])
            .unwrap();
        let left: i64 = connection
            .query_row("SELECT COUNT(*) FROM reminders", [], |row| row.get(0))
            .unwrap();
        assert_eq!(left, 0);
    }
}
