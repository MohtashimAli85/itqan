use chrono::Utc;
use tauri::State;

use crate::domain::reminders::{self, Reminder, ReminderId, ReminderInput};
use crate::domain::tasks::TaskId;
use crate::scheduler::Scheduler;
use itqan_core::db::Database;
use itqan_core::error::CommandError;
use itqan_core::settings;

#[tauri::command]
#[specta::specta]
pub fn create_reminder(
    database: State<Database>,
    scheduler: State<Scheduler>,
    input: ReminderInput,
) -> Result<Reminder, CommandError> {
    let reminder = database.with(|connection| {
        let timezone = settings::timezone(connection)?;
        reminders::create(connection, input, timezone, Utc::now())
    })?;
    scheduler.wake();
    Ok(reminder)
}

#[tauri::command]
#[specta::specta]
pub fn list_task_reminders(
    database: State<Database>,
    task_id: TaskId,
) -> Result<Vec<Reminder>, CommandError> {
    Ok(database.with(|connection| reminders::list_for_task(connection, task_id))?)
}

#[tauri::command]
#[specta::specta]
pub fn delete_reminder(
    database: State<Database>,
    scheduler: State<Scheduler>,
    id: ReminderId,
) -> Result<(), CommandError> {
    database.with(|connection| reminders::delete(connection, id))?;
    scheduler.wake();
    Ok(())
}
