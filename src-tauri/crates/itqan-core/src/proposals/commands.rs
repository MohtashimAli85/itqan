#![allow(clippy::needless_pass_by_value)]

use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;

use super::{Proposal, ProposalId, ProposalRegistry};
use crate::db::Database;
use crate::error::{AppError, CommandError};

#[derive(Debug, Clone, Serialize, Deserialize, Type, tauri_specta::Event)]
pub struct ProposalsChanged;

pub fn sweep(app: &AppHandle) -> Result<(), AppError> {
    let expired = app.state::<Database>().with(|connection| {
        super::sweep(connection, &app.state::<ProposalRegistry>(), Utc::now())
    })?;
    if expired {
        ProposalsChanged.emit(app)?;
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn list_proposals(
    database: State<Database>,
    registry: State<ProposalRegistry>,
) -> Result<Vec<Proposal>, CommandError> {
    Ok(database.with(|connection| super::pending(connection, &registry, Utc::now()))?)
}

fn after(app: &AppHandle) -> Result<Vec<Proposal>, AppError> {
    ProposalsChanged.emit(app)?;
    app.state::<Database>()
        .with(|connection| super::pending(connection, &app.state::<ProposalRegistry>(), Utc::now()))
}

#[tauri::command]
#[specta::specta]
pub fn accept_proposal(app: AppHandle, id: ProposalId) -> Result<Vec<Proposal>, CommandError> {
    let registry = app.state::<ProposalRegistry>();
    let handler = app
        .state::<Database>()
        .with(|connection| super::accept(connection, &registry, id, Utc::now()))?;
    if let Some(handler) = handler {
        if let Err(error) = handler.after_apply(&app) {
            tracing::warn!(%error, "after accepting a proposal");
        }
    }
    Ok(after(&app)?)
}

#[tauri::command]
#[specta::specta]
pub fn reject_proposal(app: AppHandle, id: ProposalId) -> Result<Vec<Proposal>, CommandError> {
    app.state::<Database>()
        .with(|connection| super::reject(connection, id, Utc::now()))?;
    Ok(after(&app)?)
}
