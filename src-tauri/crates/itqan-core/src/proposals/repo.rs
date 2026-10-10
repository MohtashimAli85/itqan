use chrono::{DateTime, Utc};
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Row};

use super::{NewProposal, Proposal, ProposalId, ProposalStatus};
use crate::beliefs::BeliefId;
use crate::db::enums::{column, to_text};
use crate::error::AppError;

const COLUMNS: &str = "id, kind, title, reason, effect, status, created_at, expires_at";

fn from_row(row: &Row) -> rusqlite::Result<Proposal> {
    Ok(Proposal {
        id: row.get(0)?,
        kind: row.get(1)?,
        title: row.get(2)?,
        reason: row.get(3)?,
        effect: row.get(4)?,
        status: column(row, 5)?,
        created_at: row.get(6)?,
        expires_at: row.get(7)?,
    })
}

pub struct Stored {
    pub proposal: Proposal,
    pub payload: serde_json::Value,
    pub belief_id: Option<BeliefId>,
}

pub fn is_blocked(
    connection: &Connection,
    key: &str,
    now: DateTime<Utc>,
) -> Result<bool, AppError> {
    Ok(connection.query_row(
        "SELECT EXISTS (
             SELECT 1 FROM proposals
             WHERE key = ?1 AND (status = 'pending' OR suppressed_until > ?2)
         )",
        params![key, now],
        |row| row.get(0),
    )?)
}

pub fn insert(
    connection: &Connection,
    proposal: &NewProposal,
    now: DateTime<Utc>,
    expires_at: DateTime<Utc>,
) -> Result<ProposalId, AppError> {
    connection.execute(
        "INSERT INTO proposals (kind, key, title, reason, effect, payload, belief_id, status,
                                created_at, expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending', ?8, ?9)",
        params![
            proposal.kind,
            proposal.key,
            proposal.title,
            proposal.reason,
            proposal.effect,
            serde_json::to_string(&proposal.payload)?,
            proposal.belief_id,
            now,
            expires_at,
        ],
    )?;
    ProposalId::try_from(connection.last_insert_rowid())
        .map_err(|_| AppError::InvalidInput("proposal id out of range".into()))
}

fn placeholders(count: usize, offset: usize) -> String {
    (0..count)
        .map(|index| format!("?{}", index + offset))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn pending(connection: &Connection, kinds: &[String]) -> Result<Vec<Proposal>, AppError> {
    if kinds.is_empty() {
        return Ok(Vec::new());
    }
    let sql = format!(
        "SELECT {COLUMNS} FROM proposals WHERE status = 'pending' AND kind IN ({})
         ORDER BY created_at, id",
        placeholders(kinds.len(), 1)
    );
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map(params_from_iter(kinds), from_row)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn expire(
    connection: &Connection,
    kinds: &[String],
    now: DateTime<Utc>,
    quiet_until: DateTime<Utc>,
) -> Result<(), AppError> {
    if kinds.is_empty() {
        return Ok(());
    }
    let sql = format!(
        "UPDATE proposals SET status = 'expired', decided_at = ?1, suppressed_until = ?2
         WHERE status = 'pending' AND expires_at <= ?1 AND kind IN ({})",
        placeholders(kinds.len(), 3)
    );
    let mut values: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now), Box::new(quiet_until)];
    values.extend(
        kinds
            .iter()
            .map(|kind| Box::new(kind.clone()) as Box<dyn rusqlite::ToSql>),
    );
    connection.execute(&sql, params_from_iter(values.iter().map(AsRef::as_ref)))?;
    Ok(())
}

pub fn find_pending(connection: &Connection, id: ProposalId) -> Result<Option<Stored>, AppError> {
    let sql = format!(
        "SELECT {COLUMNS}, payload, belief_id FROM proposals WHERE id = ?1 AND status = 'pending'"
    );
    connection
        .query_row(&sql, [id], |row| {
            let payload: String = row.get(8)?;
            Ok((from_row(row)?, payload, row.get(9)?))
        })
        .optional()?
        .map(|(proposal, payload, belief_id)| {
            Ok(Stored {
                proposal,
                payload: serde_json::from_str(&payload)?,
                belief_id,
            })
        })
        .transpose()
}

pub fn decide(
    connection: &Connection,
    id: ProposalId,
    status: ProposalStatus,
    now: DateTime<Utc>,
    quiet_until: Option<DateTime<Utc>>,
) -> Result<(), AppError> {
    connection.execute(
        "UPDATE proposals SET status = ?2, decided_at = ?3, suppressed_until = ?4 WHERE id = ?1",
        params![id, to_text(&status)?, now, quiet_until],
    )?;
    Ok(())
}
