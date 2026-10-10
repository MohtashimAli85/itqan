#![allow(clippy::needless_pass_by_value)]

use chrono::Utc;
use tauri::{AppHandle, State};

use super::drafts::{self, Answer, DraftBelief};
use crate::db::Database;
use crate::error::CommandError;

#[tauri::command]
#[specta::specta]
pub async fn draft_beliefs(
    app: AppHandle,
    answers: Vec<Answer>,
) -> Result<Vec<DraftBelief>, CommandError> {
    Ok(drafts::draft(&app, &answers).await?)
}

#[tauri::command]
#[specta::specta]
pub fn save_belief_notes(
    database: State<Database>,
    drafts: Vec<DraftBelief>,
) -> Result<(), CommandError> {
    Ok(database.with(|connection| drafts::save_notes(connection, &drafts, Utc::now()))?)
}
