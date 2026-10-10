use chrono::{Duration, Utc};
use tauri::{AppHandle, State};

use crate::domain::focus;
use crate::domain::modes::{self, WorkDay};
use crate::domain::tasks::TaskId;
use crate::scheduler::{self, ModeStatus};
use itqan_core::db::Database;
use itqan_core::error::CommandError;

#[tauri::command]
#[specta::specta]
pub fn get_mode_status(app: AppHandle) -> Result<ModeStatus, CommandError> {
    Ok(scheduler::refresh(&app)?)
}

#[tauri::command]
#[specta::specta]
pub fn set_rest(
    app: AppHandle,
    database: State<Database>,
    minutes: Option<u16>,
) -> Result<ModeStatus, CommandError> {
    let until = minutes.map(|minutes| Utc::now() + Duration::minutes(i64::from(minutes)));
    database.with(|connection| modes::set_rest_until(connection, until))?;
    Ok(scheduler::refresh(&app)?)
}

#[tauri::command]
#[specta::specta]
pub fn start_focus(
    app: AppHandle,
    database: State<Database>,
    minutes: u16,
    task_id: Option<TaskId>,
) -> Result<ModeStatus, CommandError> {
    database.with(|connection| {
        modes::set_rest_until(connection, None)?;
        focus::start(connection, minutes, task_id, Utc::now())
    })?;
    Ok(scheduler::refresh(&app)?)
}

#[tauri::command]
#[specta::specta]
pub fn stop_focus(app: AppHandle, database: State<Database>) -> Result<ModeStatus, CommandError> {
    database.with(|connection| match focus::active(connection)? {
        Some(session) => focus::finish(connection, session.id, Utc::now(), false),
        None => Ok(()),
    })?;
    Ok(scheduler::refresh(&app)?)
}

#[tauri::command]
#[specta::specta]
pub fn get_work_hours(database: State<Database>) -> Result<Vec<WorkDay>, CommandError> {
    Ok(database.with(modes::work_hours)?)
}

#[tauri::command]
#[specta::specta]
pub fn set_work_hours(
    app: AppHandle,
    database: State<Database>,
    days: Vec<WorkDay>,
) -> Result<Vec<WorkDay>, CommandError> {
    let days = database.with(|connection| modes::set_work_hours(connection, days))?;
    scheduler::refresh(&app)?;
    Ok(days)
}
