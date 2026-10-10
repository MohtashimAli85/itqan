#![allow(clippy::needless_pass_by_value)]

use chrono::Utc;
use itqan_contracts::TaskId;
use tauri::State;

use super::{Reminder, ReminderId, ReminderInput};
use crate::db::Database;
use crate::error::CommandError;
use crate::ports::Ports;
use crate::scheduler::Scheduler;
use crate::settings;

#[tauri::command]
#[specta::specta]
pub fn create_reminder(
    database: State<Database>,
    ports: State<Ports>,
    scheduler: State<Scheduler>,
    input: ReminderInput,
) -> Result<Reminder, CommandError> {
    let reminder = database.with(|connection| {
        let timezone = settings::timezone(connection)?;
        super::create(connection, &ports, input, timezone, Utc::now())
    })?;
    scheduler.wake();
    Ok(reminder)
}

#[tauri::command]
#[specta::specta]
pub fn list_task_reminders(
    database: State<Database>,
    ports: State<Ports>,
    task_id: TaskId,
) -> Result<Vec<Reminder>, CommandError> {
    Ok(database.with(|connection| super::list_for_task(connection, &ports, task_id))?)
}

#[tauri::command]
#[specta::specta]
pub fn delete_reminder(
    database: State<Database>,
    scheduler: State<Scheduler>,
    id: ReminderId,
) -> Result<(), CommandError> {
    database.with(|connection| super::delete(connection, id))?;
    scheduler.wake();
    Ok(())
}
