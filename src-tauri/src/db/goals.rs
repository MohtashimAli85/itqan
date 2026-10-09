use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};

use super::enums::{column, optional_column, to_text};
use crate::domain::goals::{Goal, GoalId, GoalStatus, Milestone, MilestoneId, MilestoneStatus};
use crate::domain::profile::Motivator;
use crate::error::AppError;

const GOAL_COLUMNS: &str = "id, title, motivator, target_date, status, created_at, completed_at";
const MILESTONE_COLUMNS: &str = "id, goal_id, title, week_start, status, completed_at";

fn goal(row: &Row) -> rusqlite::Result<Goal> {
    Ok(Goal {
        id: row.get(0)?,
        title: row.get(1)?,
        motivator: optional_column(row, 2)?,
        target_date: row.get(3)?,
        status: column(row, 4)?,
        created_at: row.get(5)?,
        completed_at: row.get(6)?,
    })
}

fn milestone(row: &Row) -> rusqlite::Result<Milestone> {
    Ok(Milestone {
        id: row.get(0)?,
        goal_id: row.get(1)?,
        title: row.get(2)?,
        week_start: row.get(3)?,
        status: column(row, 4)?,
        completed_at: row.get(5)?,
    })
}

fn id(connection: &Connection) -> Result<i32, AppError> {
    i32::try_from(connection.last_insert_rowid())
        .map_err(|_| AppError::InvalidInput("id out of range".into()))
}

fn motivator_text(motivator: Option<Motivator>) -> Result<Option<String>, AppError> {
    motivator.as_ref().map(to_text).transpose()
}

pub fn list(connection: &Connection) -> Result<Vec<Goal>, AppError> {
    let sql = format!(
        "SELECT {GOAL_COLUMNS} FROM goals
         ORDER BY status = 'archived', status = 'done', target_date IS NULL, target_date, id"
    );
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map([], goal)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn find(connection: &Connection, id: GoalId) -> Result<Option<Goal>, AppError> {
    let sql = format!("SELECT {GOAL_COLUMNS} FROM goals WHERE id = ?1");
    Ok(connection.query_row(&sql, [id], goal).optional()?)
}

pub fn insert(
    connection: &Connection,
    title: &str,
    motivator: Option<Motivator>,
    target_date: Option<NaiveDate>,
    now: DateTime<Utc>,
) -> Result<GoalId, AppError> {
    connection.execute(
        "INSERT INTO goals (title, motivator, target_date, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![title, motivator_text(motivator)?, target_date, now],
    )?;
    id(connection)
}

pub fn update(
    connection: &Connection,
    id: GoalId,
    title: &str,
    motivator: Option<Motivator>,
    target_date: Option<NaiveDate>,
) -> Result<(), AppError> {
    connection.execute(
        "UPDATE goals SET title = ?2, motivator = ?3, target_date = ?4 WHERE id = ?1",
        params![id, title, motivator_text(motivator)?, target_date],
    )?;
    Ok(())
}

pub fn set_status(
    connection: &Connection,
    id: GoalId,
    status: GoalStatus,
    completed_at: Option<DateTime<Utc>>,
) -> Result<(), AppError> {
    connection.execute(
        "UPDATE goals SET status = ?2, completed_at = ?3 WHERE id = ?1",
        params![id, to_text(&status)?, completed_at],
    )?;
    Ok(())
}

pub fn delete(connection: &Connection, id: GoalId) -> Result<(), AppError> {
    connection.execute("DELETE FROM goals WHERE id = ?1", [id])?;
    Ok(())
}

pub fn milestones(connection: &Connection, goal_id: GoalId) -> Result<Vec<Milestone>, AppError> {
    let sql = format!(
        "SELECT {MILESTONE_COLUMNS} FROM milestones WHERE goal_id = ?1 ORDER BY sort_order, id"
    );
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map([goal_id], milestone)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn find_milestone(
    connection: &Connection,
    id: MilestoneId,
) -> Result<Option<Milestone>, AppError> {
    let sql = format!("SELECT {MILESTONE_COLUMNS} FROM milestones WHERE id = ?1");
    Ok(connection.query_row(&sql, [id], milestone).optional()?)
}

pub fn next_sort_order(connection: &Connection, goal_id: GoalId) -> Result<i32, AppError> {
    Ok(connection.query_row(
        "SELECT coalesce(max(sort_order), 0) + 1 FROM milestones WHERE goal_id = ?1",
        [goal_id],
        |row| row.get(0),
    )?)
}

pub fn insert_milestone(
    connection: &Connection,
    goal_id: GoalId,
    title: &str,
    week_start: NaiveDate,
    sort_order: i32,
) -> Result<(), AppError> {
    connection.execute(
        "INSERT INTO milestones (goal_id, title, week_start, sort_order) VALUES (?1, ?2, ?3, ?4)",
        params![goal_id, title, week_start, sort_order],
    )?;
    Ok(())
}

pub fn set_milestone_status(
    connection: &Connection,
    id: MilestoneId,
    status: MilestoneStatus,
    completed_at: Option<DateTime<Utc>>,
) -> Result<(), AppError> {
    connection.execute(
        "UPDATE milestones SET status = ?2, completed_at = ?3 WHERE id = ?1",
        params![id, to_text(&status)?, completed_at],
    )?;
    Ok(())
}

pub fn delete_milestone(connection: &Connection, id: MilestoneId) -> Result<(), AppError> {
    connection.execute("DELETE FROM milestones WHERE id = ?1", [id])?;
    Ok(())
}
