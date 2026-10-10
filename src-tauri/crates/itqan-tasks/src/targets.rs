use std::collections::HashMap;

use chrono::Utc;
use itqan_core::db::Database;
use itqan_core::error::AppError;
use itqan_core::ports::{Ports, ReminderTarget, ReminderTargetKind, TargetInfo};
use rusqlite::{params_from_iter, Connection};
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use crate::tasks::{self, TaskStatus};
use crate::TasksChanged;
use itqan_contracts::AppEvent;
use itqan_core::bus;

pub struct TaskReminders;

impl ReminderTarget for TaskReminders {
    fn describe(
        &self,
        connection: &Connection,
        ids: &[i32],
    ) -> Result<HashMap<i32, TargetInfo>, AppError> {
        let placeholders = vec!["?"; ids.len()].join(", ");
        let mut statement = connection.prepare(&format!(
            "SELECT id, title, status = 'open' FROM tasks WHERE id IN ({placeholders})"
        ))?;
        let rows = statement.query_map(params_from_iter(ids), |row| {
            Ok((
                row.get(0)?,
                TargetInfo {
                    title: Some(row.get(1)?),
                    active: row.get(2)?,
                },
            ))
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    fn complete(&self, app: &AppHandle, id: i32) -> Result<(), AppError> {
        let task = app
            .state::<Database>()
            .with(|connection| tasks::set_status(connection, id, TaskStatus::Done, Utc::now()))?;
        TasksChanged.emit(app)?;
        bus::publish(
            app,
            &AppEvent::TaskCompleted {
                task_id: id,
                kind: task.kind,
                skill_id: task.skill_id,
            },
        )
    }
}

pub fn register(ports: &Ports) -> Result<(), AppError> {
    ports.set_reminder_target(ReminderTargetKind::Task, TaskReminders)
}

#[cfg(test)]
mod tests {
    use crate::test_connection;
    use chrono::TimeZone;
    use chrono_tz::Tz;
    use itqan_core::reminders::{self, ReminderInput};

    use super::*;
    use crate::tasks::TaskInput;

    #[test]
    fn task_reminders_follow_their_task() {
        let connection = test_connection();
        let ports = Ports::default();
        register(&ports).unwrap();
        let now = Utc.with_ymd_and_hms(2026, 10, 10, 8, 0, 0).unwrap();
        let at = now + chrono::Duration::hours(1);
        let task = tasks::create(
            &connection,
            TaskInput {
                title: "Buy dahi".into(),
                ..Default::default()
            },
            now,
        )
        .unwrap();
        let input = |task_id| ReminderInput {
            task_id,
            title: None,
            at,
            rrule: None,
            critical: false,
        };
        reminders::create(&connection, &ports, input(Some(task.id)), Tz::UTC, now).unwrap();

        let due = reminders::due(&connection, &ports, at).unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].title, "Buy dahi");

        tasks::set_status(&connection, task.id, TaskStatus::Done, now).unwrap();
        assert!(reminders::due(&connection, &ports, at).unwrap().is_empty());
        assert!(ports
            .reminder_target(ReminderTargetKind::Task)
            .unwrap()
            .is_some());
    }
}
