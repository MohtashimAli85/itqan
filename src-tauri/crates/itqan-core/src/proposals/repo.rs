use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};

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
             WHERE key = ?1
               AND ((status = 'pending' AND expires_at > ?2) OR suppressed_until > ?2)
         )",
        params![key, now],
        |row| row.get(0),
    )?)
}

pub struct Due {
    pub id: ProposalId,
    pub kind: String,
    pub key: String,
    pub expires_at: DateTime<Utc>,
}

pub fn due(connection: &Connection, now: DateTime<Utc>) -> Result<Vec<Due>, AppError> {
    let mut statement = connection.prepare(
        "SELECT id, kind, key, expires_at FROM proposals
         WHERE status = 'pending' AND expires_at <= ?1",
    )?;
    let rows = statement.query_map([now], |row| {
        Ok(Due {
            id: row.get(0)?,
            kind: row.get(1)?,
            key: row.get(2)?,
            expires_at: row.get(3)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
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

pub fn pending(
    connection: &Connection,
    kinds: &[String],
    now: DateTime<Utc>,
) -> Result<Vec<Proposal>, AppError> {
    if kinds.is_empty() {
        return Ok(Vec::new());
    }
    let sql = format!(
        "SELECT {COLUMNS} FROM proposals
         WHERE status = 'pending' AND expires_at > ?1 AND kind IN ({})
         ORDER BY created_at, id",
        placeholders(kinds.len(), 2)
    );
    let mut values: Vec<&dyn rusqlite::ToSql> = vec![&now];
    values.extend(kinds.iter().map(|kind| kind as &dyn rusqlite::ToSql));
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map(values.as_slice(), from_row)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn find_pending(
    connection: &Connection,
    id: ProposalId,
    now: DateTime<Utc>,
) -> Result<Option<Stored>, AppError> {
    let sql = format!(
        "SELECT {COLUMNS}, payload, belief_id FROM proposals
         WHERE id = ?1 AND status = 'pending' AND expires_at > ?2"
    );
    connection
        .query_row(&sql, params![id, now], |row| {
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
