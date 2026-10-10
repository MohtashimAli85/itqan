#![allow(clippy::needless_pass_by_value)]

use chrono::Utc;
use tauri::{AppHandle, State};

use super::drafts::{self, Answer, DraftBelief};
use crate::ai::{self, Job};
use crate::db::Database;
use crate::error::CommandError;

#[tauri::command]
#[specta::specta]
pub async fn draft_beliefs(
    app: AppHandle,
    answers: Vec<Answer>,
) -> Result<Vec<DraftBelief>, CommandError> {
    let user = drafts::prompt(&answers);
    if user.is_empty() {
        return Ok(Vec::new());
    }
    let reply = ai::complete(&app, Job::Profiling, drafts::SYSTEM, &user).await?;
    Ok(drafts::parse(&reply))
}

#[tauri::command]
#[specta::specta]
pub fn save_belief_notes(
    database: State<Database>,
    drafts: Vec<DraftBelief>,
) -> Result<(), CommandError> {
    Ok(database.with(|connection| drafts::save_notes(connection, &drafts, Utc::now()))?)
}
