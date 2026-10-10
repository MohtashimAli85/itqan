use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Timelike, Utc};
use tauri::{AppHandle, Manager};

use crate::bus::{Signal, Subscriber};
use crate::coach::{LATER, OPEN_MAIN, OPEN_PANEL};
use crate::db::{settings as settings_repo, Database};
use crate::error::AppError;
use crate::modes;
use crate::nudges::{AgentKind, Priority};
use crate::overlay::action;
use crate::ports::Ports;
use crate::profile;
use crate::profile::{CoachStyle, Motivator, Profile};
use crate::settings;
use itqan_contracts::AppEvent;

const STANDUP_WINDOW_MINUTES: i64 = 180;
const CHECKIN_WINDOW_MINUTES: i64 = 240;
const LAST_STANDUP: &str = "last_standup_date";
const LAST_CHECKIN: &str = "last_checkin_date";
const DONE: &str = "done";

pub struct RhythmAgent;

fn greeting_name(profile: &Profile) -> String {
    profile
        .name
        .as_deref()
        .map(|name| format!(", {name}"))
        .unwrap_or_default()
}

pub fn standup_text(profile: &Profile) -> String {
    let name = greeting_name(profile);
    match profile.coach_style {
        CoachStyle::Mentor => {
            format!("Good morning{name}. What would make today a good day? Pick your top 3.")
        }
        CoachStyle::Manager => {
            format!("Morning{name}. What are you shipping today? Set your top 3.")
        }
        CoachStyle::Trainer => format!("Top 3 for today{name}. Pick them now, then start."),
    }
}

fn top_motivator(profile: &Profile) -> Option<Motivator> {
    profile
        .motivators
        .iter()
        .max_by_key(|weight| weight.weight)
        .map(|weight| weight.motivator)
}

pub fn checkin_text(profile: &Profile, completed: u32) -> String {
    if completed == 0 {
        return match profile.coach_style {
            CoachStyle::Mentor => {
                "Work hours are done. Nothing got ticked off today, and that's okay. What got in the way?".into()
            }
            CoachStyle::Manager => {
                "Work hours are done and nothing got ticked off. What blocked you?".into()
            }
            CoachStyle::Trainer => {
                "Nothing ticked off today. Reset: what's the first thing you'll finish tomorrow?".into()
            }
        };
    }
    let plural = if completed == 1 { "" } else { "s" };
    let question = match top_motivator(profile) {
        Some(Motivator::Building) => "What did you build?",
        Some(Motivator::Learning) => "What did you learn today?",
        Some(Motivator::Health) => "Did you move enough today?",
        _ => "What went well today?",
    };
    format!("You finished {completed} task{plural} today. MashaAllah. {question}")
}

fn read_date(connection: &rusqlite::Connection, key: &str) -> Result<Option<NaiveDate>, AppError> {
    Ok(settings_repo::get(connection, key)?.and_then(|value| value.parse().ok()))
}

impl Subscriber for RhythmAgent {
    fn on_event(
        &self,
        app: &AppHandle,
        event: &AppEvent,
        now: DateTime<Utc>,
    ) -> Result<Vec<Signal>, AppError> {
        if !matches!(event, AppEvent::Tick | AppEvent::ModeChanged { .. }) {
            return Ok(Vec::new());
        }
        app.state::<Database>().with(|connection| {
            let timezone = settings::timezone(connection)?;
            let local = now.with_timezone(&timezone);
            let today = local.date_naive();
            let weekday = u8::try_from(local.weekday().num_days_from_monday()).unwrap_or(0);
            let Some(day) = modes::work_hours(connection)?
                .into_iter()
                .find(|day| day.weekday == weekday && day.enabled)
            else {
                return Ok(Vec::new());
            };
            let minute = i64::from(local.hour() * 60 + local.minute());
            let start = i64::from(day.start_minute);
            let end = i64::from(day.end_minute);
            let local_midnight = timezone
                .from_local_datetime(&today.and_hms_opt(0, 0, 0).unwrap_or_default())
                .earliest()
                .map(|midnight| midnight.with_timezone(&Utc))
                .unwrap_or(now);
            let profile = profile::get(connection)?;
            let mut suggestions = Vec::new();

            if (start..start + STANDUP_WINDOW_MINUTES).contains(&minute)
                && read_date(connection, LAST_STANDUP)? != Some(today)
            {
                settings_repo::set(connection, LAST_STANDUP, &today.to_string())?;
                suggestions.push(Signal {
                    agent: AgentKind::Coach,
                    kind: "standup".into(),
                    priority: Priority::Rhythm,
                    text: standup_text(&profile),
                    actions: vec![action(OPEN_PANEL, "Plan my day"), action(LATER, "Later")],
                    not_before: None,
                    expires_at: local_midnight + Duration::minutes(start + STANDUP_WINDOW_MINUTES),
                });
            }
            if (end..end + CHECKIN_WINDOW_MINUTES).contains(&minute)
                && read_date(connection, LAST_CHECKIN)? != Some(today)
            {
                settings_repo::set(connection, LAST_CHECKIN, &today.to_string())?;
                let completed = app.state::<Ports>().tasks_completed_between(
                    connection,
                    local_midnight,
                    now,
                )?;
                suggestions.push(Signal {
                    agent: AgentKind::Coach,
                    kind: "checkin".into(),
                    priority: Priority::Rhythm,
                    text: checkin_text(&profile, completed),
                    actions: vec![
                        action(OPEN_MAIN, "See today"),
                        action(DONE, "Done for today"),
                    ],
                    not_before: None,
                    expires_at: local_midnight + Duration::minutes(end + CHECKIN_WINDOW_MINUTES),
                });
            }
            Ok(suggestions)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::MotivatorWeight;

    fn profile(style: CoachStyle, top: Motivator) -> Profile {
        Profile {
            name: Some("Mohtashim".into()),
            coach_style: style,
            motivators: vec![
                MotivatorWeight {
                    motivator: top,
                    weight: 60,
                },
                MotivatorWeight {
                    motivator: Motivator::Money,
                    weight: 40,
                },
            ],
            ..Profile::default()
        }
    }

    #[test]
    fn standup_follows_the_coach_style() {
        assert!(
            standup_text(&profile(CoachStyle::Manager, Motivator::Building))
                .starts_with("Morning, Mohtashim. What are you shipping today?")
        );
        assert!(
            standup_text(&profile(CoachStyle::Mentor, Motivator::Building)).contains("good day")
        );
    }

    #[test]
    fn checkin_asks_about_the_top_motivator() {
        let builder = checkin_text(&profile(CoachStyle::Mentor, Motivator::Building), 3);
        assert_eq!(
            builder,
            "You finished 3 tasks today. MashaAllah. What did you build?"
        );
        let learner = checkin_text(&profile(CoachStyle::Mentor, Motivator::Learning), 1);
        assert!(learner.starts_with("You finished 1 task today."));
        assert!(learner.ends_with("What did you learn today?"));
    }

    #[test]
    fn empty_days_never_shame() {
        for style in [CoachStyle::Mentor, CoachStyle::Manager, CoachStyle::Trainer] {
            let text = checkin_text(&profile(style, Motivator::Building), 0);
            assert!(!text.to_lowercase().contains("lazy"));
            assert!(!text.to_lowercase().contains("fail"));
            assert!(text.ends_with('?'));
        }
    }
}
