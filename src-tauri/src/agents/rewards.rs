use std::time::Duration as StdDuration;

use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use super::{AppEvent, Signal, Subscriber};
use crate::domain::modes;
use crate::domain::rewards::{self, Award, RewardOutcome, XpSource};
use crate::overlay::{self, OrbState, OverlayStore};
use itqan_core::db::{settings as settings_repo, Database};
use itqan_core::error::AppError;
use itqan_core::nudges::{AgentKind, Priority};
use itqan_core::ports::{Ports, TodayCounts};
use itqan_core::profile;
use itqan_core::settings;
use itqan_tasks::tasks;

const SOUND: &str = "reward_sound";
const LAST_ALL_DONE: &str = "last_all_done_date";
const HAPPY_FOR: StdDuration = StdDuration::from_secs(4);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Celebration {
    LevelUp,
    AllDone,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, tauri_specta::Event)]
#[serde(rename_all = "camelCase")]
pub struct RewardEarned {
    pub outcome: RewardOutcome,
    pub celebration: Option<Celebration>,
    pub sound: bool,
}

pub struct RewardsAgent;

pub fn sound_enabled(connection: &rusqlite::Connection) -> Result<bool, AppError> {
    Ok(settings_repo::get(connection, SOUND)?.as_deref() != Some("false"))
}

pub fn set_sound(connection: &rusqlite::Connection, enabled: bool) -> Result<(), AppError> {
    settings_repo::set(connection, SOUND, if enabled { "true" } else { "false" })
}

fn award_for(
    connection: &rusqlite::Connection,
    event: &AppEvent,
) -> Result<Option<Award>, AppError> {
    let award = |source, amount, task_id, skill_id, reference_id| Award {
        source,
        amount,
        task_id,
        skill_id,
        reference_id,
    };
    Ok(match *event {
        AppEvent::TaskCompleted { task_id } => {
            let task = tasks::get(connection, task_id)?;
            let amount = rewards::task_xp(task.kind, &profile::get(connection)?);
            Some(award(
                XpSource::Task,
                amount,
                Some(task_id),
                task.skill_id,
                Some(task_id),
            ))
        }
        AppEvent::FocusCompleted {
            session_id,
            minutes,
        } => Some(award(
            XpSource::Focus,
            rewards::focus_xp(minutes),
            None,
            None,
            Some(session_id),
        )),
        AppEvent::MilestoneCompleted { milestone_id } => Some(award(
            XpSource::Milestone,
            rewards::MILESTONE_XP,
            None,
            None,
            Some(milestone_id),
        )),
        AppEvent::GoalCompleted { goal_id } => Some(award(
            XpSource::Goal,
            rewards::GOAL_XP,
            None,
            None,
            Some(goal_id),
        )),
        AppEvent::HabitLogged { .. } => {
            Some(award(XpSource::Habit, rewards::HABIT_XP, None, None, None))
        }
        AppEvent::Tick | AppEvent::ModeChanged { .. } => None,
    })
}

fn celebrate(app: &AppHandle) -> Result<(), AppError> {
    let store = app.state::<OverlayStore>();
    if store.snapshot()?.orb_state == OrbState::Critical {
        return Ok(());
    }
    overlay::publish(app, store.set_orb_state(OrbState::Happy)?)?;
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(HAPPY_FOR).await;
        let store = app.state::<OverlayStore>();
        let still_happy = store
            .snapshot()
            .is_ok_and(|snapshot| snapshot.orb_state == OrbState::Happy);
        let waiting = store
            .has_origin(overlay::BubbleOrigin::FocusDone)
            .unwrap_or(false);
        if still_happy && !waiting {
            if let Ok(snapshot) = store.restore_base() {
                let _ = overlay::publish(&app, snapshot);
            }
        }
    });
    Ok(())
}

impl Subscriber for RewardsAgent {
    fn on_event(
        &self,
        app: &AppHandle,
        event: &AppEvent,
        now: DateTime<Utc>,
    ) -> Result<Vec<Signal>, AppError> {
        let result = app.state::<Database>().with(|connection| {
            let Some(award) = award_for(connection, event)? else {
                return Ok(None);
            };
            let timezone = settings::timezone(connection)?;
            let today = now.with_timezone(&timezone).date_naive();
            let rest_days: Vec<u8> = modes::work_hours(connection)?
                .into_iter()
                .filter(|day| !day.enabled)
                .map(|day| day.weekday)
                .collect();
            let is_rest_day = |date: NaiveDate| {
                rest_days
                    .contains(&u8::try_from(date.weekday().num_days_from_monday()).unwrap_or(0))
            };
            let Some(outcome) = rewards::award(connection, award, now, today, is_rest_day)? else {
                return Ok(None);
            };
            let mut all_done = false;
            if matches!(event, AppEvent::TaskCompleted { .. })
                && settings_repo::get(connection, LAST_ALL_DONE)?.as_deref()
                    != Some(&today.to_string())
            {
                let (start, end) = itqan_core::settings::day_bounds(timezone, today);
                let TodayCounts { done, open } = app
                    .state::<Ports>()
                    .today_task_counts(connection, start, end)?;
                all_done = done > 0 && open == 0;
                if all_done {
                    settings_repo::set(connection, LAST_ALL_DONE, &today.to_string())?;
                }
            }
            Ok(Some((outcome, all_done, sound_enabled(connection)?)))
        })?;
        let Some((outcome, all_done, sound)) = result else {
            return Ok(Vec::new());
        };
        let celebration = if all_done {
            Some(Celebration::AllDone)
        } else if outcome.leveled_up {
            Some(Celebration::LevelUp)
        } else {
            None
        };
        RewardEarned {
            outcome,
            celebration,
            sound,
        }
        .emit(app)?;
        let text = match celebration {
            Some(Celebration::AllDone) => {
                "All done for today. MashaAllah! Rest well, you earned it.".to_owned()
            }
            Some(Celebration::LevelUp) => {
                format!("Level {}. MashaAllah, keep building.", outcome.level.level)
            }
            None => return Ok(Vec::new()),
        };
        celebrate(app)?;
        Ok(vec![Signal {
            agent: AgentKind::Coach,
            kind: "celebration".into(),
            priority: Priority::Rhythm,
            text,
            actions: Vec::new(),
            not_before: None,
            expires_at: now + Duration::hours(2),
        }])
    }
}
