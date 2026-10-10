use std::sync::Mutex;
use std::time::Duration as StdDuration;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use crate::agents::{self, AppEvent};
use crate::domain::focus::{self, FocusStatus, DEFAULT_MINUTES};
use crate::domain::modes::{self, Mode};
use crate::domain::tasks;
use crate::overlay::{self, Activity, BubbleAction, BubbleOrigin, OrbState, OverlayStore};
use itqan_contracts::PrayerWindow;
use itqan_core::db::Database;
use itqan_core::error::AppError;
use itqan_core::ports::Ports;
use itqan_core::settings;

const FOCUS_TICK: StdDuration = StdDuration::from_secs(15);
const IDLE_TICK: StdDuration = StdDuration::from_secs(60);
const OVERTIME_ACTIVITY: i64 = 5;
const OVERTIME_EXTENSION: i64 = 30;
const OVERTIME_UNANSWERED: i64 = 30;
const BREAK_MINUTES: i64 = 5;

pub const SWITCH: &str = "switch";
pub const EXTEND: &str = "extend";
pub const AGAIN: &str = "again";
pub const BREAK: &str = "break";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ModeStatus {
    pub mode: Mode,
    pub scheduled_mode: Mode,
    pub rest_until: Option<DateTime<Utc>>,
    pub focus: Option<FocusStatus>,
    pub next_prayer: Option<PrayerWindow>,
    pub active_prayer: Option<PrayerWindow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, tauri_specta::Event)]
pub struct ModeChanged(pub ModeStatus);

#[derive(Debug, Default)]
struct EngineState {
    last: Option<ModeStatus>,
    extended_until: Option<DateTime<Utc>>,
    overtime_asked_at: Option<DateTime<Utc>>,
    last_progress: Option<Option<f32>>,
}

#[derive(Debug, Default)]
pub struct ModeEngine(Mutex<EngineState>);

fn appearance(mode: Mode) -> OrbState {
    match mode {
        Mode::Work => OrbState::Idle,
        Mode::Evening => OrbState::Evening,
        Mode::Rest => OrbState::Resting,
        Mode::Focus => OrbState::Focus,
    }
}

fn action(id: &str, label: &str) -> BubbleAction {
    BubbleAction {
        id: id.into(),
        label: label.into(),
    }
}

pub fn current(app: &AppHandle) -> Result<Option<ModeStatus>, AppError> {
    let engine = app.state::<ModeEngine>();
    let state = engine.0.lock().map_err(|_| AppError::LockPoisoned)?;
    Ok(state.last.clone())
}

pub fn evaluate(app: &AppHandle) -> Result<(ModeStatus, StdDuration), AppError> {
    let now = Utc::now();
    let database = app.state::<Database>();
    let ports = app.state::<Ports>();
    let (days, evening_end, rest_until, prayer, timezone, active_focus, today_progress) = database
        .with(|connection| {
            let timezone = settings::timezone(connection)?;
            let today = now.with_timezone(&timezone).date_naive();
            let (start, end) = itqan_core::settings::day_bounds(timezone, today);
            let (done, open) = tasks::today_counts(connection, start, end)?;
            let today_progress = (done + open > 0).then(|| done as f32 / (done + open) as f32);
            Ok((
                modes::work_hours(connection)?,
                modes::evening_end_minute(connection)?,
                modes::rest_until(connection)?.filter(|until| *until > now),
                ports.prayer(connection, now, timezone)?,
                timezone,
                focus::active(connection)?,
                today_progress,
            ))
        })?;
    let mut focus_status = active_focus.map(|session| focus::status(session, now, &prayer.windows));
    if let Some(status) = focus_status.as_ref().filter(|status| status.is_finished()) {
        let id = status.session.id;
        let minutes = status.session.planned_minutes;
        database.with(|connection| focus::finish(connection, id, now, true))?;
        focus_status = None;
        celebrate_focus(app)?;
        agents::publish(
            app,
            AppEvent::FocusCompleted {
                session_id: id,
                minutes,
            },
        )?;
    }
    let scheduled_mode = modes::scheduled_mode(
        &days,
        evening_end,
        now.with_timezone(&timezone).naive_local(),
    );

    let engine = app.state::<ModeEngine>();
    let mut state = engine.0.lock().map_err(|_| AppError::LockPoisoned)?;
    let was_working = state.last.as_ref().map(|status| status.mode) == Some(Mode::Work);
    let extended = state.extended_until.is_some_and(|until| until > now);
    if was_working
        && scheduled_mode != Mode::Work
        && focus_status.is_none()
        && rest_until.is_none()
        && !extended
        && state.overtime_asked_at.is_none()
        && app
            .state::<Activity>()
            .active_within(now, Duration::minutes(OVERTIME_ACTIVITY))
    {
        state.overtime_asked_at = Some(now);
        ask_overtime(app)?;
    }
    if state
        .overtime_asked_at
        .is_some_and(|asked| now - asked > Duration::minutes(OVERTIME_UNANSWERED))
    {
        state.overtime_asked_at = None;
        if let Some(snapshot) = app
            .state::<OverlayStore>()
            .dismiss_origin(BubbleOrigin::Overtime)?
        {
            overlay::publish(app, snapshot)?;
        }
    }
    if scheduled_mode == Mode::Work {
        state.overtime_asked_at = None;
    }

    let mode = if focus_status.is_some() {
        Mode::Focus
    } else if rest_until.is_some() {
        Mode::Rest
    } else if extended || state.overtime_asked_at.is_some() {
        Mode::Work
    } else {
        scheduled_mode
    };
    let status = ModeStatus {
        mode,
        scheduled_mode,
        rest_until,
        focus: focus_status,
        next_prayer: prayer.next,
        active_prayer: prayer.active,
    };

    let previous = state.last.replace(status.clone());
    if previous.as_ref().map(|status| status.mode) != Some(mode) {
        let store = app.state::<OverlayStore>();
        overlay::publish(
            app,
            store.set_appearance(appearance(mode), mode == Mode::Focus)?,
        )?;
    }
    let progress = status
        .focus
        .as_ref()
        .map(|focus| focus.progress)
        .or(today_progress)
        .map(|value| (value * 100.0).round() / 100.0);
    if state.last_progress != Some(progress) {
        state.last_progress = Some(progress);
        let store = app.state::<OverlayStore>();
        overlay::publish(app, store.set_progress(progress)?)?;
    }
    let mode_changed = previous.as_ref().map(|status| status.mode) != Some(mode);
    let from = previous.as_ref().map(|status| status.mode);
    if previous.as_ref() != Some(&status) {
        ModeChanged(status.clone()).emit(app)?;
    }
    drop(state);
    if mode_changed {
        agents::publish(app, AppEvent::ModeChanged { from, to: mode })?;
    }
    let pause = if status.focus.is_some() {
        FOCUS_TICK
    } else {
        IDLE_TICK
    };
    Ok((status, pause))
}

fn ask_overtime(app: &AppHandle) -> Result<(), AppError> {
    let store = app.state::<OverlayStore>();
    let (_, snapshot) = store.show_bubble(
        "Work hours are over. Wrap up and switch to evening?".into(),
        vec![action(SWITCH, "Switch now"), action(EXTEND, "30 more min")],
        Some(BubbleOrigin::Overtime),
    )?;
    overlay::publish(app, snapshot)
}

fn celebrate_focus(app: &AppHandle) -> Result<(), AppError> {
    let store = app.state::<OverlayStore>();
    store.show_bubble(
        "Focus done, MashaAllah. Stretch for five minutes?".into(),
        vec![
            action(BREAK, "Take a break"),
            action(AGAIN, "Another 25 min"),
        ],
        Some(BubbleOrigin::FocusDone),
    )?;
    overlay::publish(app, store.set_orb_state(OrbState::Happy)?)
}

pub fn resolve_overtime(app: &AppHandle, action: Option<&str>) -> Result<(), AppError> {
    let engine = app.state::<ModeEngine>();
    let mut state = engine.0.lock().map_err(|_| AppError::LockPoisoned)?;
    state.overtime_asked_at = None;
    state.extended_until =
        (action == Some(EXTEND)).then(|| Utc::now() + Duration::minutes(OVERTIME_EXTENSION));
    Ok(())
}

pub fn resolve_focus_done(app: &AppHandle, action: Option<&str>) -> Result<(), AppError> {
    let database = app.state::<Database>();
    let now = Utc::now();
    match action {
        Some(AGAIN) => {
            database.with(|connection| focus::start(connection, DEFAULT_MINUTES, None, now))?;
        }
        Some(BREAK) => {
            database.with(|connection| {
                modes::set_rest_until(connection, Some(now + Duration::minutes(BREAK_MINUTES)))
            })?;
        }
        _ => {}
    }
    Ok(())
}
