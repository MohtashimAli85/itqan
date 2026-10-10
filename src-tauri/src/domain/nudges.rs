use chrono::{DateTime, Duration, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::db::nudges as repo;
use itqan_core::error::AppError;

pub use itqan_core::bus::{AgentKind, Priority};

pub type NudgeId = i32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Outcome {
    Accepted,
    Snoozed,
    Dismissed,
    Ignored,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Nudge {
    pub id: NudgeId,
    pub agent: AgentKind,
    pub kind: String,
    pub priority: Priority,
    pub text: String,
    pub style: Option<String>,
    pub mode: Option<String>,
    pub fired_at: DateTime<Utc>,
    pub outcome: Option<Outcome>,
    pub action: Option<String>,
    pub outcome_at: Option<DateTime<Utc>>,
}

pub struct NewNudge<'a> {
    pub agent: AgentKind,
    pub kind: &'a str,
    pub priority: Priority,
    pub text: &'a str,
    pub style: Option<&'a str>,
    pub mode: Option<&'a str>,
    pub fired_at: DateTime<Utc>,
}

pub fn record(connection: &Connection, nudge: &NewNudge) -> Result<NudgeId, AppError> {
    repo::insert(connection, nudge)
}

pub fn resolve(
    connection: &Connection,
    id: NudgeId,
    outcome: Outcome,
    action: Option<&str>,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    repo::set_outcome(connection, id, outcome, action, now)
}

pub fn budgeted_since(connection: &Connection, since: DateTime<Utc>) -> Result<u32, AppError> {
    repo::count_non_critical_since(connection, since)
}

pub fn unanswered_before(
    connection: &Connection,
    before: DateTime<Utc>,
) -> Result<Vec<NudgeId>, AppError> {
    repo::unanswered_before(connection, before)
}

pub fn last_fired(connection: &Connection, kind: &str) -> Result<Option<DateTime<Utc>>, AppError> {
    repo::last_fired(connection, kind)
}

pub fn recent(connection: &Connection, limit: u32) -> Result<Vec<Nudge>, AppError> {
    repo::recent(connection, limit)
}

pub fn last_hour(now: DateTime<Utc>) -> DateTime<Utc> {
    now - Duration::hours(1)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::db::test_connection;

    fn at(minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 10, 9, minute, 0).unwrap()
    }

    fn new(priority: Priority, fired_at: DateTime<Utc>) -> NewNudge<'static> {
        NewNudge {
            agent: AgentKind::Coach,
            kind: "standup",
            priority,
            text: "What are you shipping today?",
            style: Some("mentor"),
            mode: Some("work"),
            fired_at,
        }
    }

    #[test]
    fn budget_counts_only_recent_non_critical_nudges() {
        let connection = test_connection();
        record(&connection, &new(Priority::Rhythm, at(0))).unwrap();
        record(&connection, &new(Priority::Health, at(30))).unwrap();
        record(&connection, &new(Priority::Critical, at(40))).unwrap();

        assert_eq!(budgeted_since(&connection, at(10)).unwrap(), 1);
        assert_eq!(budgeted_since(&connection, at(0)).unwrap(), 2);
    }

    #[test]
    fn outcomes_are_logged_and_unanswered_nudges_found() {
        let connection = test_connection();
        let answered = record(&connection, &new(Priority::Rhythm, at(0))).unwrap();
        let waiting = record(&connection, &new(Priority::Rhythm, at(1))).unwrap();
        record(&connection, &new(Priority::Critical, at(1))).unwrap();
        resolve(
            &connection,
            answered,
            Outcome::Accepted,
            Some("open-panel"),
            at(2),
        )
        .unwrap();

        assert_eq!(
            unanswered_before(&connection, at(5)).unwrap(),
            vec![waiting]
        );
        let log = recent(&connection, 10).unwrap();
        let first = log.iter().find(|nudge| nudge.id == answered).unwrap();
        assert_eq!(first.outcome, Some(Outcome::Accepted));
        assert_eq!(first.action.as_deref(), Some("open-panel"));
    }
}
