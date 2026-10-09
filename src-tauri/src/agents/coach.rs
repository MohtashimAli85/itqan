use std::collections::HashMap;
use std::sync::Mutex;

use chrono::{DateTime, Duration, Timelike, Utc};
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use super::health as health_agent;
use super::Suggestion;
use crate::commands::events::HealthChanged;
use crate::db::{settings as settings_repo, Database};
use crate::domain::health::{self, HabitKind};
use crate::domain::modes::Mode;
use crate::domain::nudges::{self, NewNudge, NudgeId, Outcome, Priority};
use crate::domain::{profile, settings};
use crate::error::AppError;
use crate::overlay::{self, Activity, BubbleOrigin, OverlayStore};
use crate::scheduler::{modes as mode_engine, ModeStatus};
use crate::tray;

pub const OPEN_PANEL: &str = "open-panel";
pub const OPEN_MAIN: &str = "open-main";
pub const LATER: &str = "later";

const IGNORE_AFTER_MINUTES: i64 = 3;
const LATER_MINUTES: i64 = 30;
const DEFAULT_BUDGET: u32 = 3;
const MAX_BUDGET: u32 = 12;
const BUDGET: &str = "nudge_budget_per_hour";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    Deliver,
    Defer,
    Drop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuietReason {
    Prayer,
    Focus,
    Rest,
    Family,
}

pub struct GateInput {
    pub quiet: Option<QuietReason>,
    pub used: u32,
    pub budget: u32,
    pub busy: bool,
}

pub fn gate(suggestion: &Suggestion, now: DateTime<Utc>, input: &GateInput) -> Gate {
    if suggestion.expires_at <= now {
        return Gate::Drop;
    }
    if suggestion.priority == Priority::Critical {
        return Gate::Deliver;
    }
    let waiting = suggestion
        .not_before
        .is_some_and(|not_before| not_before > now);
    if waiting || input.quiet.is_some() || input.busy || input.used >= input.budget {
        return Gate::Defer;
    }
    Gate::Deliver
}

pub fn quiet_reason(
    status: Option<&ModeStatus>,
    family: Option<(u16, u16)>,
    local_minute: u16,
) -> Option<QuietReason> {
    if let Some(status) = status {
        if status.active_prayer.is_some() {
            return Some(QuietReason::Prayer);
        }
        if status.focus.is_some() {
            return Some(QuietReason::Focus);
        }
        if status.mode == Mode::Rest {
            return Some(QuietReason::Rest);
        }
    }
    match family {
        Some((start, end)) if (start..end).contains(&local_minute) => Some(QuietReason::Family),
        _ => None,
    }
}

pub fn budget(connection: &rusqlite::Connection) -> Result<u32, AppError> {
    Ok(settings_repo::get(connection, BUDGET)?
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_BUDGET))
}

pub fn set_budget(connection: &rusqlite::Connection, value: u32) -> Result<u32, AppError> {
    if value > MAX_BUDGET {
        return Err(AppError::InvalidInput(format!(
            "at most {MAX_BUDGET} nudges an hour"
        )));
    }
    settings_repo::set(connection, BUDGET, &value.to_string())?;
    Ok(value)
}

#[derive(Default)]
struct CoachState {
    pending: Vec<Suggestion>,
    shown: HashMap<NudgeId, Suggestion>,
}

#[derive(Default)]
pub struct Coach(Mutex<CoachState>);

impl Coach {
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, CoachState>, AppError> {
        self.0.lock().map_err(|_| AppError::LockPoisoned)
    }
}

pub fn consider(
    app: &AppHandle,
    suggestions: Vec<Suggestion>,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    {
        let mut state = app.state::<Coach>().inner().lock()?;
        for suggestion in suggestions {
            let already_shown = state
                .shown
                .values()
                .any(|shown| shown.kind == suggestion.kind);
            state
                .pending
                .retain(|pending| pending.kind != suggestion.kind);
            if !already_shown {
                state.pending.push(suggestion);
            }
        }
    }
    flush(app, now)
}

pub fn tick(app: &AppHandle, now: DateTime<Utc>) -> Result<(), AppError> {
    let database = app.state::<Database>();
    let stale = database.with(|connection| {
        nudges::unanswered_before(connection, now - Duration::minutes(IGNORE_AFTER_MINUTES))
    })?;
    for id in stale {
        let shown = app.state::<Coach>().inner().lock()?.shown.remove(&id);
        if let Some(suggestion) = shown {
            let origin = BubbleOrigin::Nudge {
                id,
                critical: suggestion.priority == Priority::Critical,
            };
            if let Some(snapshot) = app.state::<OverlayStore>().dismiss_origin(origin)? {
                overlay::publish(app, snapshot)?;
            }
        }
        database.with(|connection| nudges::resolve(connection, id, Outcome::Ignored, None, now))?;
    }
    flush(app, now)
}

fn flush(app: &AppHandle, now: DateTime<Utc>) -> Result<(), AppError> {
    let database = app.state::<Database>();
    let status = mode_engine::current(app)?;
    let (budget, used, profile, local_minute) = database.with(|connection| {
        let timezone = settings::timezone(connection)?;
        let local = now.with_timezone(&timezone);
        Ok((
            budget(connection)?,
            nudges::budgeted_since(connection, nudges::last_hour(now))?,
            profile::get(connection)?,
            u16::try_from(local.hour() * 60 + local.minute()).unwrap_or(0),
        ))
    })?;
    let family = profile.family_start_minute.zip(profile.family_end_minute);
    let quiet = quiet_reason(status.as_ref(), family, local_minute);
    let style = serde_json::to_value(profile.coach_style)?
        .as_str()
        .map(str::to_owned);
    let mode = status
        .as_ref()
        .and_then(|status| serde_json::to_value(status.mode).ok())
        .and_then(|value| value.as_str().map(str::to_owned));

    let coach = app.state::<Coach>();
    let mut state = coach.inner().lock()?;
    let mut pending = std::mem::take(&mut state.pending);
    pending.sort_by_key(|suggestion| std::cmp::Reverse(suggestion.priority));
    let mut input = GateInput {
        quiet,
        used,
        budget,
        busy: state
            .shown
            .values()
            .any(|shown| shown.priority != Priority::Critical),
    };
    for suggestion in pending {
        match gate(&suggestion, now, &input) {
            Gate::Drop => {}
            Gate::Defer => state.pending.push(suggestion),
            Gate::Deliver => {
                let id = database.with(|connection| {
                    nudges::record(
                        connection,
                        &NewNudge {
                            agent: suggestion.agent,
                            kind: &suggestion.kind,
                            priority: suggestion.priority,
                            text: &suggestion.text,
                            style: style.as_deref(),
                            mode: mode.as_deref(),
                            fired_at: now,
                        },
                    )
                })?;
                let critical = suggestion.priority == Priority::Critical;
                let store = app.state::<OverlayStore>();
                let (_, snapshot) = store.show_bubble(
                    suggestion.text.clone(),
                    suggestion.actions.clone(),
                    Some(BubbleOrigin::Nudge { id, critical }),
                )?;
                overlay::publish(app, snapshot)?;
                tracing::info!(nudge = id, kind = %suggestion.kind, "nudge delivered");
                if !critical {
                    input.used += 1;
                    input.busy = true;
                }
                state.shown.insert(id, suggestion);
            }
        }
    }
    Ok(())
}

pub fn resolve(app: &AppHandle, id: NudgeId, action: Option<&str>) -> Result<(), AppError> {
    let now = Utc::now();
    let outcome = match action {
        None => Outcome::Dismissed,
        Some(LATER) => Outcome::Snoozed,
        Some(_) => Outcome::Accepted,
    };
    app.state::<Database>()
        .with(|connection| nudges::resolve(connection, id, outcome, action, now))?;
    let shown = app.state::<Coach>().inner().lock()?.shown.remove(&id);
    if let (Outcome::Snoozed, Some(mut suggestion)) = (outcome, shown) {
        suggestion.not_before = Some(now + Duration::minutes(LATER_MINUTES));
        suggestion.expires_at = suggestion
            .expires_at
            .max(now + Duration::minutes(LATER_MINUTES * 2));
        app.state::<Coach>()
            .inner()
            .lock()?
            .pending
            .push(suggestion);
    }
    match action {
        Some(OPEN_PANEL) => overlay::open_panel(app)?,
        Some(OPEN_MAIN) => tray::show_main_window(app),
        Some(other) => {
            if let Some(kind) = health_agent::parse_habit_action(other) {
                app.state::<Database>()
                    .with(|connection| health::log(connection, kind, now))?;
                if kind == HabitKind::Stretch {
                    app.state::<Activity>().reset_streak();
                }
                HealthChanged.emit(app)?;
            }
        }
        None => {}
    }
    flush(app, now)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::domain::nudges::AgentKind;

    fn at(minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 10, 9, minute, 0).unwrap()
    }

    fn suggestion(priority: Priority) -> Suggestion {
        Suggestion {
            agent: AgentKind::Coach,
            kind: "standup".into(),
            priority,
            text: "What are you shipping today?".into(),
            actions: Vec::new(),
            not_before: None,
            expires_at: at(30),
        }
    }

    fn open() -> GateInput {
        GateInput {
            quiet: None,
            used: 0,
            budget: 3,
            busy: false,
        }
    }

    #[test]
    fn expired_suggestions_are_dropped() {
        assert_eq!(
            gate(&suggestion(Priority::Rhythm), at(30), &open()),
            Gate::Drop
        );
    }

    #[test]
    fn quiet_time_busy_bubbles_and_budget_defer_normal_nudges() {
        let rhythm = suggestion(Priority::Rhythm);
        assert_eq!(gate(&rhythm, at(0), &open()), Gate::Deliver);

        let quiet = GateInput {
            quiet: Some(QuietReason::Prayer),
            ..open()
        };
        assert_eq!(gate(&rhythm, at(0), &quiet), Gate::Defer);

        let busy = GateInput {
            busy: true,
            ..open()
        };
        assert_eq!(gate(&rhythm, at(0), &busy), Gate::Defer);

        let spent = GateInput { used: 3, ..open() };
        assert_eq!(gate(&rhythm, at(0), &spent), Gate::Defer);
    }

    #[test]
    fn critical_nudges_break_through() {
        let critical = suggestion(Priority::Critical);
        let everything = GateInput {
            quiet: Some(QuietReason::Focus),
            used: 10,
            budget: 3,
            busy: true,
        };
        assert_eq!(gate(&critical, at(0), &everything), Gate::Deliver);
    }

    #[test]
    fn snoozed_suggestions_wait() {
        let later = Suggestion {
            not_before: Some(at(10)),
            ..suggestion(Priority::Rhythm)
        };
        assert_eq!(gate(&later, at(5), &open()), Gate::Defer);
        assert_eq!(gate(&later, at(10), &open()), Gate::Deliver);
    }

    #[test]
    fn family_time_is_quiet() {
        assert_eq!(
            quiet_reason(None, Some((1140, 1260)), 1200),
            Some(QuietReason::Family)
        );
        assert_eq!(quiet_reason(None, Some((1140, 1260)), 1260), None);
        assert_eq!(quiet_reason(None, None, 1200), None);
    }

    #[test]
    fn budget_setting_is_capped() {
        let connection = crate::db::test_connection();
        assert_eq!(budget(&connection).unwrap(), 3);
        assert_eq!(set_budget(&connection, 5).unwrap(), 5);
        assert_eq!(budget(&connection).unwrap(), 5);
        assert!(set_budget(&connection, 50).is_err());
    }
}
