use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, State};
use tauri_specta::Event;

use crate::ai;
use crate::bus;
use crate::db::Database;
use crate::error::{AppError, CommandError};
use crate::goals::{
    self, Goal, GoalId, GoalInput, GoalStatus, Milestone, MilestoneId, MilestoneInput,
    MilestoneStatus,
};
use crate::planner::{self, PlanProposal, PlanRequest};
use crate::profile;
use crate::settings;
use crate::skills::{self, Skill, SkillId};
use itqan_contracts::AppEvent;

#[derive(Debug, Clone, Serialize, Deserialize, Type, tauri_specta::Event)]
pub struct GoalsChanged;

fn changed<T>(app: &AppHandle, value: T) -> Result<T, CommandError> {
    GoalsChanged.emit(app).map_err(AppError::from)?;
    Ok(value)
}

#[tauri::command]
#[specta::specta]
pub fn list_goals(database: State<Database>) -> Result<Vec<Goal>, CommandError> {
    Ok(database.with(goals::list)?)
}

#[tauri::command]
#[specta::specta]
pub fn create_goal(
    app: AppHandle,
    database: State<Database>,
    input: GoalInput,
) -> Result<Goal, CommandError> {
    let goal = database.with(|connection| goals::create(connection, input, Utc::now()))?;
    changed(&app, goal)
}

#[tauri::command]
#[specta::specta]
pub fn update_goal(
    app: AppHandle,
    database: State<Database>,
    id: GoalId,
    input: GoalInput,
) -> Result<Goal, CommandError> {
    let goal = database.with(|connection| goals::update(connection, id, input))?;
    changed(&app, goal)
}

#[tauri::command]
#[specta::specta]
pub fn set_goal_status(
    app: AppHandle,
    database: State<Database>,
    id: GoalId,
    status: GoalStatus,
) -> Result<Goal, CommandError> {
    let goal = database.with(|connection| goals::set_status(connection, id, status, Utc::now()))?;
    if status == GoalStatus::Done {
        bus::publish(&app, AppEvent::GoalCompleted { goal_id: id })?;
    }
    changed(&app, goal)
}

#[tauri::command]
#[specta::specta]
pub fn delete_goal(
    app: AppHandle,
    database: State<Database>,
    id: GoalId,
) -> Result<(), CommandError> {
    database.with(|connection| goals::delete(connection, id))?;
    changed(&app, ())
}

#[tauri::command]
#[specta::specta]
pub fn list_milestones(
    database: State<Database>,
    goal_id: GoalId,
) -> Result<Vec<Milestone>, CommandError> {
    Ok(database.with(|connection| goals::milestones(connection, goal_id))?)
}

#[tauri::command]
#[specta::specta]
pub fn add_milestones(
    app: AppHandle,
    database: State<Database>,
    goal_id: GoalId,
    milestones: Vec<MilestoneInput>,
) -> Result<Vec<Milestone>, CommandError> {
    let saved =
        database.with(|connection| goals::add_milestones(connection, goal_id, milestones))?;
    changed(&app, saved)
}

#[tauri::command]
#[specta::specta]
pub fn set_milestone_status(
    app: AppHandle,
    database: State<Database>,
    id: MilestoneId,
    status: MilestoneStatus,
) -> Result<Milestone, CommandError> {
    let milestone = database
        .with(|connection| goals::set_milestone_status(connection, id, status, Utc::now()))?;
    if status == MilestoneStatus::Done {
        bus::publish(&app, AppEvent::MilestoneCompleted { milestone_id: id })?;
    }
    changed(&app, milestone)
}

#[tauri::command]
#[specta::specta]
pub fn delete_milestone(
    app: AppHandle,
    database: State<Database>,
    id: MilestoneId,
) -> Result<(), CommandError> {
    database.with(|connection| goals::delete_milestone(connection, id))?;
    changed(&app, ())
}

#[tauri::command]
#[specta::specta]
pub async fn propose_plan(app: AppHandle, goal_id: GoalId) -> Result<PlanProposal, CommandError> {
    let request = app.state::<Database>().with(|connection| {
        let goal = goals::get(connection, goal_id)?;
        let timezone = settings::timezone(connection)?;
        Ok(PlanRequest {
            goal_title: goal.title,
            target_date: goal.target_date,
            today: Utc::now().with_timezone(&timezone).date_naive(),
            free_hours_per_week: profile::get(connection)?.free_hours_per_week,
        })
    })?;
    let rules = planner::rule_plan(&request);
    if !ai::settings(&app)?.enabled {
        return Ok(rules);
    }
    Ok(match planner::ai_plan(&app, &request, &rules).await {
        Ok(plan) => plan,
        Err(error) => PlanProposal {
            notice: Some(format!("{error}. Here is a simple plan instead.")),
            ..rules
        },
    })
}

#[tauri::command]
#[specta::specta]
pub fn list_skills(database: State<Database>) -> Result<Vec<Skill>, CommandError> {
    Ok(database.with(skills::list)?)
}

#[tauri::command]
#[specta::specta]
pub fn create_skill(
    app: AppHandle,
    database: State<Database>,
    name: String,
) -> Result<Skill, CommandError> {
    let skill = database.with(|connection| skills::create(connection, &name, Utc::now()))?;
    changed(&app, skill)
}

#[tauri::command]
#[specta::specta]
pub fn delete_skill(
    app: AppHandle,
    database: State<Database>,
    id: SkillId,
) -> Result<(), CommandError> {
    database.with(|connection| skills::delete(connection, id))?;
    changed(&app, ())
}
