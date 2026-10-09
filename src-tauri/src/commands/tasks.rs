use chrono::{Local, Utc};
use tauri::{AppHandle, State};
use tauri_specta::Event;

use super::events::TasksChanged;
use crate::db::Database;
use crate::domain::categories::{self, Category, CategoryInput};
use crate::domain::quick_add::{self, QuickAddOutcome};
use crate::domain::tasks::{self, Task, TaskFilter, TaskId, TaskInput, TaskStatus};
use crate::error::{AppError, CommandError};

fn changed(app: &AppHandle, task: Task) -> Result<Task, CommandError> {
    TasksChanged.emit(app).map_err(AppError::from)?;
    Ok(task)
}

#[tauri::command]
#[specta::specta]
pub fn list_tasks(
    database: State<Database>,
    filter: TaskFilter,
) -> Result<Vec<Task>, CommandError> {
    Ok(database.with(|connection| tasks::list(connection, &filter))?)
}

#[tauri::command]
#[specta::specta]
pub fn create_task(
    app: AppHandle,
    database: State<Database>,
    input: TaskInput,
) -> Result<Task, CommandError> {
    let task = database.with(|connection| tasks::create(connection, input, Utc::now()))?;
    changed(&app, task)
}

#[tauri::command]
#[specta::specta]
pub fn update_task(
    app: AppHandle,
    database: State<Database>,
    id: TaskId,
    input: TaskInput,
) -> Result<Task, CommandError> {
    let task = database.with(|connection| tasks::update(connection, id, input, Utc::now()))?;
    changed(&app, task)
}

#[tauri::command]
#[specta::specta]
pub fn set_task_status(
    app: AppHandle,
    database: State<Database>,
    id: TaskId,
    status: TaskStatus,
) -> Result<Task, CommandError> {
    let task = database.with(|connection| tasks::set_status(connection, id, status, Utc::now()))?;
    changed(&app, task)
}

#[tauri::command]
#[specta::specta]
pub fn set_task_top_three(
    app: AppHandle,
    database: State<Database>,
    id: TaskId,
    on: bool,
) -> Result<Task, CommandError> {
    let task = database.with(|connection| tasks::set_top_three(connection, id, on, Utc::now()))?;
    changed(&app, task)
}

#[tauri::command]
#[specta::specta]
pub fn delete_task(
    app: AppHandle,
    database: State<Database>,
    id: TaskId,
) -> Result<(), CommandError> {
    database.with(|connection| tasks::delete(connection, id))?;
    TasksChanged.emit(&app).map_err(AppError::from)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn list_categories(database: State<Database>) -> Result<Vec<Category>, CommandError> {
    Ok(database.with(categories::list)?)
}

#[tauri::command]
#[specta::specta]
pub fn create_category(
    database: State<Database>,
    input: CategoryInput,
) -> Result<Category, CommandError> {
    Ok(database.with(|connection| categories::create(connection, input))?)
}

#[tauri::command]
#[specta::specta]
pub fn quick_add_task(
    app: AppHandle,
    database: State<Database>,
    text: String,
) -> Result<QuickAddOutcome, CommandError> {
    let outcome = database.with(|connection| quick_add::create(connection, &text, Local::now()))?;
    TasksChanged.emit(&app).map_err(AppError::from)?;
    Ok(outcome)
}
