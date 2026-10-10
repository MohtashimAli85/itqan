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

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "health_tables",
        sql: include_str!("../migrations/0001_health_tables.sql"),
    },
    Migration {
        version: 2,
        name: "health_index_names",
        sql: include_str!("../migrations/0002_health_index_names.sql"),
    },
];

pub struct HealthModule;

impl Module for HealthModule {
    fn id(&self) -> &'static str {
        "health"
    }

    fn migrations(&self) -> &'static [Migration] {
        MIGRATIONS
    }

    fn setup(&self, app: &AppHandle) -> Result<(), AppError> {
        register(&app.state::<Ports>(), &app.state::<ActionRouter>())
    }

    fn teardown(&self, app: &AppHandle) -> Result<(), AppError> {
        unregister(&app.state::<Ports>(), &app.state::<ActionRouter>())
    }

    fn subscribers(&self) -> Vec<Box<dyn Subscriber>> {
        vec![Box::new(agent::HealthAgent)]
    }
}

fn register(ports: &Ports, actions: &ActionRouter) -> Result<(), AppError> {
    ports.set_reminder_target(ReminderTargetKind::Habit, HabitReminders)?;
    actions.register(agent::HABIT_ACTIONS, agent::HabitActions)
}

fn unregister(ports: &Ports, actions: &ActionRouter) -> Result<(), AppError> {
    ports.clear_reminder_target(ReminderTargetKind::Habit)?;
    actions.unregister(agent::HABIT_ACTIONS)
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
        let count = |sql: &str| -> i64 { connection.query_row(sql, [], |row| row.get(0)).unwrap() };
        assert_eq!(
            count("SELECT COUNT(*) FROM sqlite_master WHERE name IN ('habits', 'habit_logs', 'habit_logs_habit_time')"),
            0
        );
        assert_eq!(
            count("SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'health_habit_logs_habit_time'"),
            1
        );
        connection
            .execute("DELETE FROM health_habits WHERE id = ?1", [medicine])
            .unwrap();
        assert_eq!(count("SELECT COUNT(*) FROM reminders"), 0);
        assert_eq!(count("SELECT COUNT(*) FROM health_habit_logs"), 0);
    }

    #[test]
    fn setup_registers_the_habit_target_and_actions() {
        let ports = Ports::default();
        let actions = ActionRouter::default();

        register(&ports, &actions).unwrap();

        assert!(ports
            .reminder_target(ReminderTargetKind::Habit)
            .unwrap()
            .is_some());
        assert!(actions.handles(agent::HABIT_ACTIONS).unwrap());

        unregister(&ports, &actions).unwrap();
        assert!(ports
            .reminder_target(ReminderTargetKind::Habit)
            .unwrap()
            .is_none());
        assert!(!actions.handles(agent::HABIT_ACTIONS).unwrap());
        register(&ports, &actions).unwrap();
    }
}
