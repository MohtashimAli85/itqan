use chrono::{DateTime, Duration, NaiveTime, TimeZone, Utc};
use tauri::{AppHandle, Manager};

use itqan_core::actions::{ActionHandler, LATER};
use tauri_specta::Event;

use crate::habits::{self as health, AgeBand, HabitKind};
use crate::HealthChanged;
use itqan_contracts::AppEvent;
use itqan_core::activity::Activity;
use itqan_core::bus::{self, Signal, Subscriber};
use itqan_core::db::Database;
use itqan_core::error::AppError;
use itqan_core::nudges::{self, AgentKind, Priority};
use itqan_core::overlay::action;
use itqan_core::profile;
use itqan_core::settings;

const WATER_EVERY: Duration = Duration::hours(2);
const EXPIRES_AFTER: Duration = Duration::minutes(15);

pub const HABIT_ACTIONS: &str = "habit";
pub const STRETCH: &str = "stretch";
pub const EYE_REST: &str = "eye-rest";
pub const WATER: &str = "water";

pub struct HealthAgent;

pub fn habit_action(kind: HabitKind) -> String {
    let name = match kind {
        HabitKind::Stretch => "stretch",
        HabitKind::EyeRest => "eyeRest",
        HabitKind::Water => "water",
        HabitKind::Medicine => "medicine",
    };
    format!("{HABIT_ACTIONS}:{name}")
}

pub fn parse_habit_action(action: &str) -> Option<HabitKind> {
    match action.strip_prefix(HABIT_ACTIONS)?.strip_prefix(':')? {
        "stretch" => Some(HabitKind::Stretch),
        "eyeRest" => Some(HabitKind::EyeRest),
        "water" => Some(HabitKind::Water),
        _ => None,
    }
}

pub struct HabitActions;

impl ActionHandler for HabitActions {
    fn handle(&self, app: &AppHandle, action: &str) -> Result<(), AppError> {
        let Some(kind) = parse_habit_action(action) else {
            return Ok(());
        };
        app.state::<Database>()
            .with(|connection| health::log(connection, kind, Utc::now()))?;
        if kind == HabitKind::Stretch {
            app.state::<Activity>().reset_streak();
        }
        HealthChanged.emit(app)?;
        bus::publish(app, AppEvent::HabitLogged { kind })
    }
}

fn due(last: Option<DateTime<Utc>>, now: DateTime<Utc>, every: Duration) -> bool {
    last.is_none_or(|last| now - last >= every)
}

fn suggestion(
    kind: &str,
    text: String,
    actions: Vec<itqan_core::overlay::BubbleAction>,
    now: DateTime<Utc>,
) -> Signal {
    Signal {
        agent: AgentKind::Health,
        kind: kind.into(),
        priority: Priority::Health,
        text,
        actions,
        not_before: None,
        expires_at: now + EXPIRES_AFTER,
    }
}

impl Subscriber for HealthAgent {
    fn on_event(
        &self,
        app: &AppHandle,
        event: &AppEvent,
        now: DateTime<Utc>,
    ) -> Result<Vec<Signal>, AppError> {
        if *event != AppEvent::Tick {
            return Ok(Vec::new());
        }
        let sitting = app.state::<Activity>().sitting_for(now);
        app.state::<Database>().with(|connection| {
            if !health::is_enabled(connection)? {
                return Ok(Vec::new());
            }
            let band = AgeBand::from_age(profile::get(connection)?.age);
            let mut found = Vec::new();

            if sitting >= band.stretch_after()
                && due(
                    nudges::last_fired(connection, STRETCH)?,
                    now,
                    band.stretch_after(),
                )
            {
                found.push(suggestion(
                    STRETCH,
                    band.stretch_text(sitting),
                    vec![
                        action(&habit_action(HabitKind::Stretch), "Done"),
                        action(LATER, "Later"),
                    ],
                    now,
                ));
            } else if sitting >= band.eye_rest_after()
                && due(
                    nudges::last_fired(connection, EYE_REST)?,
                    now,
                    band.eye_rest_after(),
                )
            {
                found.push(suggestion(
                    EYE_REST,
                    health::eye_rest_text(),
                    vec![action(&habit_action(HabitKind::EyeRest), "Done")],
                    now,
                ));
            }

            let timezone = settings::timezone(connection)?;
            let local = now.with_timezone(&timezone);
            let start_of_day = timezone
                .from_local_datetime(&local.date_naive().and_time(NaiveTime::MIN))
                .earliest()
                .map_or(now, |start| start.with_timezone(&Utc));
            let water = health::habit(connection, HabitKind::Water)?;
            let target = water.target.unwrap_or(8);
            let today = health::logged_since(connection, HabitKind::Water, start_of_day)?;
            let behind = today + 1 < health::water_expected(target, local.time());
            if water.enabled
                && behind
                && sitting > Duration::zero()
                && due(nudges::last_fired(connection, WATER)?, now, WATER_EVERY)
            {
                found.push(suggestion(
                    WATER,
                    health::water_text(today, target),
                    vec![
                        action(&habit_action(HabitKind::Water), "Drank a glass"),
                        action(LATER, "Later"),
                    ],
                    now,
                ));
            }
            Ok(found)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn habit_actions_round_trip() {
        for kind in [HabitKind::Stretch, HabitKind::EyeRest, HabitKind::Water] {
            assert_eq!(parse_habit_action(&habit_action(kind)), Some(kind));
        }
        assert_eq!(parse_habit_action("open-panel"), None);
        assert_eq!(parse_habit_action("habit:medicine"), None);
    }

    #[test]
    fn nudges_wait_for_their_interval() {
        let now = Utc::now();
        assert!(due(None, now, Duration::minutes(50)));
        assert!(!due(
            Some(now - Duration::minutes(20)),
            now,
            Duration::minutes(50)
        ));
        assert!(due(
            Some(now - Duration::minutes(50)),
            now,
            Duration::minutes(50)
        ));
    }
}
