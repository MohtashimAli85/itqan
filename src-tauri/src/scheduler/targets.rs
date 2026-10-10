use chrono::Utc;
use itqan_core::db::Database;
use itqan_core::error::AppError;
use itqan_core::ports::{Ports, ReminderTarget, ReminderTargetKind, TargetInfo};
use rusqlite::Connection;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use crate::agents::{self, AppEvent};
use crate::commands::events::TasksChanged;
use crate::db::tasks as task_repo;
use crate::domain::tasks::{self, TaskStatus};

pub struct TaskReminders;

impl ReminderTarget for TaskReminders {
    fn describe(&self, connection: &Connection, id: i32) -> Result<Option<TargetInfo>, AppError> {
        Ok(task_repo::find(connection, id)?.map(|task| TargetInfo {
            title: Some(task.title),
            active: task.status == TaskStatus::Open,
        }))
    }

    fn complete(&self, app: &AppHandle, id: i32) -> Result<(), AppError> {
        app.state::<Database>()
            .with(|connection| tasks::set_status(connection, id, TaskStatus::Done, Utc::now()))?;
        TasksChanged.emit(app)?;
        agents::publish(app, AppEvent::TaskCompleted { task_id: id })
    }
}

pub struct HabitReminders;

impl ReminderTarget for HabitReminders {
    fn describe(&self, _: &Connection, _: i32) -> Result<Option<TargetInfo>, AppError> {
        Ok(Some(TargetInfo {
            title: None,
            active: true,
        }))
    }

    fn complete(&self, _: &AppHandle, _: i32) -> Result<(), AppError> {
        Ok(())
    }
}

pub fn register(ports: &Ports) -> Result<(), AppError> {
    ports.set_reminder_target(ReminderTargetKind::Task, TaskReminders)?;
    ports.set_reminder_target(ReminderTargetKind::Habit, HabitReminders)
}
