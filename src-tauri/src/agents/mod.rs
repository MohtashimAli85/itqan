pub mod coach;
pub mod health;
pub mod planner;
pub mod rewards;
pub mod rhythm;

use chrono::{DateTime, Utc};
use tauri::{App, AppHandle, Manager};

use crate::domain::focus::FocusSessionId;
use crate::domain::goals::{GoalId, MilestoneId};
use crate::domain::health::HabitKind;
use crate::domain::modes::Mode;
use crate::domain::nudges::{AgentKind, Priority};
use crate::domain::tasks::TaskId;
use crate::error::AppError;
use crate::overlay::BubbleAction;

#[derive(Debug, Clone, PartialEq)]
pub enum AppEvent {
    Tick,
    TaskCompleted {
        task_id: TaskId,
    },
    FocusCompleted {
        session_id: FocusSessionId,
        minutes: u16,
    },
    MilestoneCompleted {
        milestone_id: MilestoneId,
    },
    GoalCompleted {
        goal_id: GoalId,
    },
    HabitLogged {
        kind: HabitKind,
    },
    ModeChanged {
        from: Option<Mode>,
        to: Mode,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Suggestion {
    pub agent: AgentKind,
    pub kind: String,
    pub priority: Priority,
    pub text: String,
    pub actions: Vec<BubbleAction>,
    pub not_before: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
}

pub trait Agent: Send + Sync {
    fn on_event(
        &self,
        app: &AppHandle,
        event: &AppEvent,
        now: DateTime<Utc>,
    ) -> Result<Vec<Suggestion>, AppError>;
}

pub struct Agents(Vec<Box<dyn Agent>>);

pub fn setup(app: &App) {
    app.manage(Agents(vec![
        Box::new(rhythm::RhythmAgent),
        Box::new(health::HealthAgent),
        Box::new(rewards::RewardsAgent),
    ]));
    app.manage(coach::Coach::default());
}

pub fn publish(app: &AppHandle, event: AppEvent) -> Result<(), AppError> {
    let now = Utc::now();
    let mut suggestions = Vec::new();
    for agent in &app.state::<Agents>().0 {
        match agent.on_event(app, &event, now) {
            Ok(mut found) => suggestions.append(&mut found),
            Err(error) => tracing::warn!(%error, "agent failed"),
        }
    }
    coach::consider(app, suggestions, now)
}

pub fn action(id: &str, label: &str) -> BubbleAction {
    BubbleAction {
        id: id.into(),
        label: label.into(),
    }
}
