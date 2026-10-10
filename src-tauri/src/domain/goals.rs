use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::db::goals as repo;
use itqan_core::error::AppError;
use itqan_core::profile::Motivator;

pub use itqan_contracts::{GoalId, MilestoneId};

const MAX_TITLE_LENGTH: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum GoalStatus {
    Active,
    Done,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum MilestoneStatus {
    Open,
    Done,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Goal {
    pub id: GoalId,
    pub title: String,
    pub motivator: Option<Motivator>,
    pub target_date: Option<NaiveDate>,
    pub status: GoalStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GoalInput {
    pub title: String,
    pub motivator: Option<Motivator>,
    pub target_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Milestone {
    pub id: MilestoneId,
    pub goal_id: GoalId,
    pub title: String,
    pub week_start: NaiveDate,
    pub status: MilestoneStatus,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneInput {
    pub title: String,
    pub week_start: NaiveDate,
}

fn clean_title(title: &str) -> Result<String, AppError> {
    let title = title.trim();
    if title.is_empty() || title.chars().count() > MAX_TITLE_LENGTH {
        return Err(AppError::InvalidInput(format!(
            "a title is 1 to {MAX_TITLE_LENGTH} characters"
        )));
    }
    Ok(title.to_owned())
}

pub fn list(connection: &Connection) -> Result<Vec<Goal>, AppError> {
    repo::list(connection)
}

pub fn get(connection: &Connection, id: GoalId) -> Result<Goal, AppError> {
    repo::find(connection, id)?.ok_or_else(|| AppError::NotFound(format!("goal {id}")))
}

pub fn create(
    connection: &Connection,
    input: GoalInput,
    now: DateTime<Utc>,
) -> Result<Goal, AppError> {
    let title = clean_title(&input.title)?;
    let id = repo::insert(connection, &title, input.motivator, input.target_date, now)?;
    get(connection, id)
}

pub fn update(connection: &Connection, id: GoalId, input: GoalInput) -> Result<Goal, AppError> {
    get(connection, id)?;
    let title = clean_title(&input.title)?;
    repo::update(connection, id, &title, input.motivator, input.target_date)?;
    get(connection, id)
}

pub fn set_status(
    connection: &Connection,
    id: GoalId,
    status: GoalStatus,
    now: DateTime<Utc>,
) -> Result<Goal, AppError> {
    get(connection, id)?;
    repo::set_status(
        connection,
        id,
        status,
        (status == GoalStatus::Done).then_some(now),
    )?;
    get(connection, id)
}

pub fn delete(connection: &Connection, id: GoalId) -> Result<(), AppError> {
    get(connection, id)?;
    repo::delete(connection, id)
}

pub fn milestones(connection: &Connection, goal_id: GoalId) -> Result<Vec<Milestone>, AppError> {
    repo::milestones(connection, goal_id)
}

pub fn add_milestones(
    connection: &Connection,
    goal_id: GoalId,
    inputs: Vec<MilestoneInput>,
) -> Result<Vec<Milestone>, AppError> {
    get(connection, goal_id)?;
    let titles = inputs
        .iter()
        .map(|input| clean_title(&input.title))
        .collect::<Result<Vec<_>, _>>()?;
    let transaction = connection.unchecked_transaction()?;
    let start = repo::next_sort_order(&transaction, goal_id)?;
    for (offset, (title, input)) in titles.iter().zip(&inputs).enumerate() {
        let order = start + i32::try_from(offset).unwrap_or(i32::MAX);
        repo::insert_milestone(&transaction, goal_id, title, input.week_start, order)?;
    }
    transaction.commit()?;
    milestones(connection, goal_id)
}

pub fn set_milestone_status(
    connection: &Connection,
    id: MilestoneId,
    status: MilestoneStatus,
    now: DateTime<Utc>,
) -> Result<Milestone, AppError> {
    repo::set_milestone_status(
        connection,
        id,
        status,
        (status == MilestoneStatus::Done).then_some(now),
    )?;
    repo::find_milestone(connection, id)?
        .ok_or_else(|| AppError::NotFound(format!("milestone {id}")))
}

pub fn delete_milestone(connection: &Connection, id: MilestoneId) -> Result<(), AppError> {
    repo::delete_milestone(connection, id)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use itqan_core::db::test_connection;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 10, 9, 0, 0).unwrap()
    }

    fn date(month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, month, day).unwrap()
    }

    #[test]
    fn goals_round_trip_and_complete() {
        let connection = test_connection();
        let goal = create(
            &connection,
            GoalInput {
                title: " Ship Itqan v1 ".into(),
                motivator: Some(Motivator::Building),
                target_date: Some(date(12, 31)),
            },
            now(),
        )
        .unwrap();
        assert_eq!(goal.title, "Ship Itqan v1");
        assert_eq!(goal.motivator, Some(Motivator::Building));

        let done = set_status(&connection, goal.id, GoalStatus::Done, now()).unwrap();
        assert_eq!(done.completed_at, Some(now()));
        assert!(create(
            &connection,
            GoalInput {
                title: "  ".into(),
                motivator: None,
                target_date: None
            },
            now()
        )
        .is_err());
    }

    #[test]
    fn milestones_append_in_order_and_go_with_their_goal() {
        let connection = test_connection();
        let goal = create(
            &connection,
            GoalInput {
                title: "Learn Rust".into(),
                motivator: None,
                target_date: None,
            },
            now(),
        )
        .unwrap();
        add_milestones(
            &connection,
            goal.id,
            vec![
                MilestoneInput {
                    title: "Basics".into(),
                    week_start: date(10, 12),
                },
                MilestoneInput {
                    title: "CLI".into(),
                    week_start: date(10, 19),
                },
            ],
        )
        .unwrap();
        let all = add_milestones(
            &connection,
            goal.id,
            vec![MilestoneInput {
                title: "Ship".into(),
                week_start: date(10, 26),
            }],
        )
        .unwrap();
        assert_eq!(
            all.iter().map(|m| m.title.as_str()).collect::<Vec<_>>(),
            ["Basics", "CLI", "Ship"]
        );

        let done =
            set_milestone_status(&connection, all[0].id, MilestoneStatus::Done, now()).unwrap();
        assert_eq!(done.status, MilestoneStatus::Done);

        delete(&connection, goal.id).unwrap();
        assert!(milestones(&connection, goal.id).unwrap().is_empty());
    }
}
