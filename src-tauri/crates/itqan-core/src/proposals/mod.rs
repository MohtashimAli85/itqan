pub mod commands;
mod repo;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Duration, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;

use crate::beliefs::{self, BeliefId, BeliefStatus};
use crate::error::AppError;

pub type ProposalId = i32;

pub const BELIEF: &str = "core:belief";
const PENDING_FOR: i64 = 14;
const QUIET_AFTER_REJECT: i64 = 30;
const QUIET_AFTER_EXPIRY: i64 = 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ProposalStatus {
    Pending,
    Accepted,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    pub id: ProposalId,
    pub kind: String,
    pub title: String,
    pub reason: String,
    pub effect: String,
    pub status: ProposalStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewProposal {
    pub kind: String,
    pub key: String,
    pub title: String,
    pub reason: String,
    pub effect: String,
    pub payload: serde_json::Value,
    pub belief_id: Option<BeliefId>,
}

pub trait ProposalHandler: Send + Sync {
    fn apply(&self, connection: &Connection, payload: &serde_json::Value) -> Result<(), AppError>;

    fn after_apply(&self, _app: &AppHandle) -> Result<(), AppError> {
        Ok(())
    }
}

struct ConfirmBelief;

impl ProposalHandler for ConfirmBelief {
    fn apply(
        &self,
        _connection: &Connection,
        _payload: &serde_json::Value,
    ) -> Result<(), AppError> {
        Ok(())
    }
}

pub struct ProposalRegistry {
    handlers: RwLock<HashMap<&'static str, Arc<dyn ProposalHandler>>>,
}

impl Default for ProposalRegistry {
    fn default() -> Self {
        let mut handlers: HashMap<&'static str, Arc<dyn ProposalHandler>> = HashMap::new();
        handlers.insert(BELIEF, Arc::new(ConfirmBelief));
        Self {
            handlers: RwLock::new(handlers),
        }
    }
}

impl ProposalRegistry {
    pub fn register(
        &self,
        kind: &'static str,
        handler: impl ProposalHandler + 'static,
    ) -> Result<(), AppError> {
        let mut handlers = self.handlers.write().map_err(|_| AppError::LockPoisoned)?;
        if handlers.contains_key(kind) {
            return Err(AppError::InvalidInput(format!(
                "proposals of kind {kind} already have a handler"
            )));
        }
        handlers.insert(kind, Arc::new(handler));
        Ok(())
    }

    pub fn unregister(&self, kind: &str) -> Result<(), AppError> {
        self.handlers
            .write()
            .map_err(|_| AppError::LockPoisoned)?
            .remove(kind);
        Ok(())
    }

    pub fn kinds(&self) -> Result<Vec<String>, AppError> {
        Ok(self
            .handlers
            .read()
            .map_err(|_| AppError::LockPoisoned)?
            .keys()
            .map(|kind| (*kind).to_owned())
            .collect())
    }

    fn handler(&self, kind: &str) -> Result<Option<Arc<dyn ProposalHandler>>, AppError> {
        Ok(self
            .handlers
            .read()
            .map_err(|_| AppError::LockPoisoned)?
            .get(kind)
            .cloned())
    }
}

fn expire(connection: &Connection, due: &repo::Due) -> Result<(), AppError> {
    repo::decide(
        connection,
        due.id,
        ProposalStatus::Expired,
        due.expires_at,
        Some(due.expires_at + Duration::days(QUIET_AFTER_EXPIRY)),
    )
}

pub fn sweep(
    connection: &Connection,
    registry: &ProposalRegistry,
    now: DateTime<Utc>,
) -> Result<bool, AppError> {
    let kinds = registry.kinds()?;
    let mut expired = false;
    for due in repo::due(connection, now)? {
        if kinds.contains(&due.kind) {
            expire(connection, &due)?;
            expired = true;
        }
    }
    Ok(expired)
}

pub fn open(
    connection: &Connection,
    proposal: &NewProposal,
    now: DateTime<Utc>,
) -> Result<Option<ProposalId>, AppError> {
    for due in repo::due(connection, now)? {
        if due.key == proposal.key {
            expire(connection, &due)?;
        }
    }
    if repo::is_blocked(connection, &proposal.key, now)? {
        return Ok(None);
    }
    repo::insert(connection, proposal, now, now + Duration::days(PENDING_FOR)).map(Some)
}

pub fn pending(
    connection: &Connection,
    registry: &ProposalRegistry,
    now: DateTime<Utc>,
) -> Result<Vec<Proposal>, AppError> {
    repo::pending(connection, &registry.kinds()?, now)
}

pub fn accept(
    connection: &Connection,
    registry: &ProposalRegistry,
    id: ProposalId,
    now: DateTime<Utc>,
) -> Result<Option<Arc<dyn ProposalHandler>>, AppError> {
    let transaction = connection.unchecked_transaction()?;
    let Some(stored) = repo::find_pending(&transaction, id, now)? else {
        return Ok(None);
    };
    if let Some(belief) = stored.belief_id {
        let status = beliefs::find(&transaction, belief)?.status;
        if !matches!(status, BeliefStatus::Proposed | BeliefStatus::Active) {
            repo::decide(
                &transaction,
                id,
                ProposalStatus::Expired,
                now,
                Some(now + Duration::days(QUIET_AFTER_EXPIRY)),
            )?;
            transaction.commit()?;
            return Ok(None);
        }
    }
    let handler = registry
        .handler(&stored.proposal.kind)?
        .ok_or_else(|| AppError::NotFound(format!("a handler for {}", stored.proposal.kind)))?;
    handler.apply(&transaction, &stored.payload)?;
    if let Some(belief) = stored.belief_id {
        beliefs::confirm(&transaction, belief, now)?;
    }
    repo::decide(&transaction, id, ProposalStatus::Accepted, now, None)?;
    transaction.commit()?;
    Ok(Some(handler))
}

pub fn reject(connection: &Connection, id: ProposalId, now: DateTime<Utc>) -> Result<(), AppError> {
    let transaction = connection.unchecked_transaction()?;
    let Some(stored) = repo::find_pending(&transaction, id, now)? else {
        return Ok(());
    };
    if let Some(belief) = stored.belief_id {
        if beliefs::find(&transaction, belief)?.status == BeliefStatus::Proposed {
            beliefs::reject(&transaction, belief, now)?;
        }
    }
    repo::decide(
        &transaction,
        id,
        ProposalStatus::Rejected,
        now,
        Some(now + Duration::days(QUIET_AFTER_REJECT)),
    )?;
    transaction.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::beliefs::{BeliefKind, BeliefSource, NewBelief, Strength};
    use crate::db::settings;
    use crate::db::test_connection;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 10, 9, 0, 0).unwrap()
    }

    struct SetFocus;

    impl ProposalHandler for SetFocus {
        fn apply(
            &self,
            connection: &Connection,
            payload: &serde_json::Value,
        ) -> Result<(), AppError> {
            let minutes = payload["minutes"]
                .as_u64()
                .ok_or_else(|| AppError::InvalidInput("minutes".into()))?;
            settings::set(connection, "focus_minutes", &minutes.to_string())
        }
    }

    fn registry() -> ProposalRegistry {
        let registry = ProposalRegistry::default();
        registry.register("core:focus_minutes", SetFocus).unwrap();
        registry
    }

    fn focus(minutes: u64, belief_id: Option<BeliefId>) -> NewProposal {
        NewProposal {
            kind: "core:focus_minutes".into(),
            key: "core:focus_minutes".into(),
            title: format!("Make {minutes} minutes the default focus length?"),
            reason: "You finish 30 minute sessions".into(),
            effect: "New focus sessions last this long".into(),
            payload: serde_json::json!({ "minutes": minutes }),
            belief_id,
        }
    }

    fn observed(connection: &Connection, minutes: u64) -> BeliefId {
        beliefs::insert(
            connection,
            &NewBelief {
                statement: format!("{minutes} minute focus works for me"),
                kind: BeliefKind::Pattern,
                subject: "focus.minutes".into(),
                value: Some(serde_json::json!({ "minutes": minutes })),
                strength: Strength::Medium,
                confidence: 0.65,
                source: BeliefSource::Observed,
                status: BeliefStatus::Proposed,
            },
            now(),
        )
        .unwrap()
    }

    #[test]
    fn accepting_applies_the_change_and_confirms_the_belief_once() {
        let connection = test_connection();
        let registry = registry();
        let belief = observed(&connection, 30);
        let id = open(&connection, &focus(30, Some(belief)), now())
            .unwrap()
            .unwrap();

        assert!(accept(&connection, &registry, id, now()).unwrap().is_some());
        assert!(accept(&connection, &registry, id, now()).unwrap().is_none());

        assert_eq!(
            settings::get(&connection, "focus_minutes")
                .unwrap()
                .as_deref(),
            Some("30")
        );
        let confirmed = beliefs::find(&connection, belief).unwrap();
        assert_eq!(confirmed.status, BeliefStatus::Active);
        assert_eq!(confirmed.source, BeliefSource::Confirmed);
        assert!(confirmed.confidence >= 0.9);
        assert_eq!(confirmed.confirmed_at, Some(now()));
        assert!(pending(&connection, &registry, now()).unwrap().is_empty());
    }

    #[test]
    fn a_failing_handler_changes_nothing() {
        let connection = test_connection();
        let registry = registry();
        let mut broken = focus(30, None);
        broken.payload = serde_json::json!({});
        let id = open(&connection, &broken, now()).unwrap().unwrap();

        assert!(accept(&connection, &registry, id, now()).is_err());
        assert_eq!(pending(&connection, &registry, now()).unwrap().len(), 1);
    }

    #[test]
    fn confirming_replaces_the_active_belief_for_the_subject() {
        let connection = test_connection();
        let registry = registry();
        let old = observed(&connection, 50);
        beliefs::confirm(&connection, old, now()).unwrap();
        let new = observed(&connection, 30);
        let id = open(&connection, &focus(30, Some(new)), now())
            .unwrap()
            .unwrap();

        accept(&connection, &registry, id, now()).unwrap();

        assert_eq!(
            beliefs::find(&connection, old).unwrap().status,
            BeliefStatus::Archived
        );
        assert_eq!(
            beliefs::find(&connection, new).unwrap().status,
            BeliefStatus::Active
        );
    }

    #[test]
    fn rejecting_quiets_the_setting_for_thirty_days_whatever_the_value() {
        let connection = test_connection();
        let registry = registry();
        let belief = observed(&connection, 30);
        let id = open(&connection, &focus(30, Some(belief)), now())
            .unwrap()
            .unwrap();

        reject(&connection, id, now()).unwrap();

        assert_eq!(
            beliefs::find(&connection, belief).unwrap().status,
            BeliefStatus::Rejected
        );
        assert!(
            open(&connection, &focus(25, None), now() + Duration::days(29))
                .unwrap()
                .is_none()
        );
        assert!(
            open(&connection, &focus(25, None), now() + Duration::days(31))
                .unwrap()
                .is_some()
        );
        assert!(pending(&connection, &registry, now()).unwrap().len() <= 1);
    }

    #[test]
    fn an_expired_proposal_cannot_be_accepted_and_does_not_block_a_new_one() {
        let connection = test_connection();
        let registry = registry();
        let id = open(&connection, &focus(30, None), now()).unwrap().unwrap();
        let late = now() + Duration::days(15);

        assert!(accept(&connection, &registry, id, late).unwrap().is_none());
        assert!(settings::get(&connection, "focus_minutes")
            .unwrap()
            .is_none());
        assert!(
            open(&connection, &focus(30, None), now() + Duration::days(29))
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn rejecting_never_demotes_an_active_belief() {
        let connection = test_connection();
        let belief = observed(&connection, 30);
        beliefs::confirm(&connection, belief, now()).unwrap();
        let id = open(&connection, &focus(30, Some(belief)), now())
            .unwrap()
            .unwrap();

        reject(&connection, id, now()).unwrap();

        assert_eq!(
            beliefs::find(&connection, belief).unwrap().status,
            BeliefStatus::Active
        );
    }

    #[test]
    fn a_stale_belief_turns_the_proposal_away() {
        let connection = test_connection();
        let registry = registry();
        let belief = observed(&connection, 30);
        let id = open(&connection, &focus(30, Some(belief)), now())
            .unwrap()
            .unwrap();
        beliefs::reject(&connection, belief, now()).unwrap();

        assert!(accept(&connection, &registry, id, now()).unwrap().is_none());
        assert!(settings::get(&connection, "focus_minutes")
            .unwrap()
            .is_none());
        assert!(pending(&connection, &registry, now()).unwrap().is_empty());
    }

    #[test]
    fn one_pending_proposal_per_key() {
        let connection = test_connection();
        assert!(open(&connection, &focus(30, None), now())
            .unwrap()
            .is_some());
        assert!(open(&connection, &focus(25, None), now())
            .unwrap()
            .is_none());
    }

    #[test]
    fn ignored_proposals_expire_and_stay_quiet_for_two_weeks() {
        let connection = test_connection();
        let registry = registry();
        open(&connection, &focus(30, None), now()).unwrap().unwrap();

        let later = now() + Duration::days(15);
        assert!(pending(&connection, &registry, later).unwrap().is_empty());
        assert!(open(&connection, &focus(30, None), later)
            .unwrap()
            .is_none());
        assert!(
            open(&connection, &focus(30, None), later + Duration::days(15))
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn proposals_without_a_handler_are_hidden_and_do_not_expire() {
        let connection = test_connection();
        let full = registry();
        open(&connection, &focus(30, None), now()).unwrap().unwrap();
        let without = ProposalRegistry::default();

        let later = now() + Duration::days(20);
        assert!(pending(&connection, &without, now()).unwrap().is_empty());
        assert!(!sweep(&connection, &without, later).unwrap());
        assert!(accept(&connection, &without, 1, now()).is_err());

        full.unregister("core:focus_minutes").unwrap();
        full.register("core:focus_minutes", SetFocus).unwrap();
        let back = pending(&connection, &full, now()).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].status, ProposalStatus::Pending);
    }
}
