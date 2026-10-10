use chrono::{DateTime, Duration, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::repo;

use itqan_core::db::settings as settings_repo;
use itqan_core::error::AppError;
use itqan_core::reminders::{self, Reminder, ReminderInput};

pub use itqan_contracts::{HabitId, HabitKind};

const ENABLED: &str = "health_enabled";
const MAX_WATER_TARGET: u16 = 20;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Habit {
    pub id: HabitId,
    pub kind: HabitKind,
    pub name: String,
    pub target: Option<u16>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Medicine {
    pub habit: Habit,
    pub reminders: Vec<Reminder>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HealthOverview {
    pub enabled: bool,
    pub water_today: u16,
    pub water_target: u16,
    pub stretches_today: u16,
    pub eye_rests_today: u16,
    pub medicines: Vec<Medicine>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgeBand {
    Unknown,
    Twenties,
    Thirties,
    Later,
}

impl AgeBand {
    pub fn from_age(age: Option<u8>) -> Self {
        match age {
            None => Self::Unknown,
            Some(age) if age < 30 => Self::Twenties,
            Some(age) if age < 45 => Self::Thirties,
            Some(_) => Self::Later,
        }
    }

    pub fn stretch_after(self) -> Duration {
        Duration::minutes(match self {
            Self::Twenties => 60,
            Self::Unknown | Self::Thirties => 50,
            Self::Later => 45,
        })
    }

    pub fn eye_rest_after(self) -> Duration {
        Duration::minutes(match self {
            Self::Twenties => 40,
            Self::Unknown | Self::Thirties => 45,
            Self::Later => 60,
        })
    }

    pub fn stretch_text(self, sitting: Duration) -> String {
        let minutes = sitting.num_minutes();
        match self {
            Self::Twenties => format!(
                "You've been at it for {minutes} minutes. Stand up, roll your shoulders and reset your posture."
            ),
            Self::Unknown | Self::Thirties => format!(
                "{minutes} minutes in the chair. A two minute walk keeps your back happy."
            ),
            Self::Later => format!(
                "{minutes} minutes sitting. Stand, stretch, and do a few slow squats if they feel good."
            ),
        }
    }
}

pub fn eye_rest_text() -> String {
    "Eye break: look at something far away for 20 seconds.".into()
}

pub fn water_text(today: u16, target: u16) -> String {
    format!("Water check. You've had {today} of {target} glasses today.")
}

pub fn water_expected(target: u16, local_time: NaiveTime) -> u16 {
    let start = NaiveTime::from_hms_opt(9, 0, 0).unwrap_or_default();
    let elapsed = (local_time - start).num_minutes().clamp(0, 12 * 60);
    u16::try_from(i64::from(target) * elapsed / (12 * 60)).unwrap_or(target)
}

pub fn is_enabled(connection: &Connection) -> Result<bool, AppError> {
    Ok(settings_repo::get(connection, ENABLED)?.as_deref() != Some("false"))
}

pub fn set_enabled(connection: &Connection, enabled: bool) -> Result<(), AppError> {
    settings_repo::set(connection, ENABLED, if enabled { "true" } else { "false" })
}

pub fn habit(connection: &Connection, kind: HabitKind) -> Result<Habit, AppError> {
    repo::first_of_kind(connection, kind)?
        .ok_or_else(|| AppError::NotFound(format!("{kind:?} habit")))
}

pub fn log(connection: &Connection, kind: HabitKind, now: DateTime<Utc>) -> Result<(), AppError> {
    let habit = habit(connection, kind)?;
    repo::insert_log(connection, habit.id, 1, now)
}

pub fn logged_since(
    connection: &Connection,
    kind: HabitKind,
    since: DateTime<Utc>,
) -> Result<u16, AppError> {
    let habit = habit(connection, kind)?;
    repo::sum_since(connection, habit.id, since)
}

pub fn set_water_target(connection: &Connection, target: u16) -> Result<(), AppError> {
    if !(1..=MAX_WATER_TARGET).contains(&target) {
        return Err(AppError::InvalidInput(format!(
            "aim for 1 to {MAX_WATER_TARGET} glasses"
        )));
    }
    let water = habit(connection, HabitKind::Water)?;
    repo::set_target(connection, water.id, target)
}

pub fn overview(
    connection: &Connection,
    start_of_day: DateTime<Utc>,
) -> Result<HealthOverview, AppError> {
    let water = habit(connection, HabitKind::Water)?;
    let medicines = repo::list_of_kind(connection, HabitKind::Medicine)?
        .into_iter()
        .map(|habit| {
            Ok(Medicine {
                reminders: reminders::list_for_habit(connection, habit.id)?,
                habit,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;
    Ok(HealthOverview {
        enabled: is_enabled(connection)?,
        water_today: repo::sum_since(connection, water.id, start_of_day)?,
        water_target: water.target.unwrap_or(8),
        stretches_today: logged_since(connection, HabitKind::Stretch, start_of_day)?,
        eye_rests_today: logged_since(connection, HabitKind::EyeRest, start_of_day)?,
        medicines,
    })
}

pub fn add_medicine(
    connection: &Connection,
    name: &str,
    times: &[NaiveTime],
    timezone: Tz,
    now: DateTime<Utc>,
) -> Result<Medicine, AppError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 80 {
        return Err(AppError::InvalidInput(
            "a medicine name is 1 to 80 characters".into(),
        ));
    }
    if times.is_empty() || times.len() > 6 {
        return Err(AppError::InvalidInput("pick 1 to 6 times a day".into()));
    }
    let transaction = connection.unchecked_transaction()?;
    let habit_id = repo::insert_habit(&transaction, HabitKind::Medicine, name, now)?;
    let today = now.with_timezone(&timezone).date_naive();
    for time in times {
        let at = timezone
            .from_local_datetime(&today.and_time(*time))
            .earliest()
            .ok_or_else(|| AppError::InvalidInput("that time does not exist today".into()))?
            .with_timezone(&Utc);
        let reminder = reminders::insert(
            &transaction,
            ReminderInput {
                task_id: None,
                title: Some(name.to_owned()),
                at,
                rrule: Some("FREQ=DAILY".into()),
                critical: true,
            },
            timezone,
            now,
        )?;
        reminders::attach_habit(&transaction, reminder, habit_id)?;
    }
    transaction.commit()?;
    let habit = repo::find(connection, habit_id)?
        .ok_or_else(|| AppError::NotFound(format!("habit {habit_id}")))?;
    Ok(Medicine {
        reminders: reminders::list_for_habit(connection, habit_id)?,
        habit,
    })
}

pub fn delete_medicine(connection: &Connection, id: HabitId) -> Result<(), AppError> {
    let habit =
        repo::find(connection, id)?.ok_or_else(|| AppError::NotFound(format!("habit {id}")))?;
    if habit.kind != HabitKind::Medicine {
        return Err(AppError::InvalidInput(
            "only medicines can be removed".into(),
        ));
    }
    repo::delete(connection, id)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use rusqlite::Connection;

    fn test_connection() -> Connection {
        itqan_core::db::test_connection_with(&[&crate::HealthModule])
    }

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 10, 6, 0, 0).unwrap()
    }

    #[test]
    fn age_bands_shift_the_emphasis() {
        assert_eq!(AgeBand::from_age(Some(26)), AgeBand::Twenties);
        assert_eq!(AgeBand::from_age(Some(38)), AgeBand::Thirties);
        assert_eq!(AgeBand::from_age(Some(52)), AgeBand::Later);
        assert!(AgeBand::Twenties
            .stretch_text(Duration::minutes(60))
            .contains("posture"));
        assert!(AgeBand::Thirties
            .stretch_text(Duration::minutes(50))
            .contains("walk"));
        assert!(AgeBand::Later.stretch_after() < AgeBand::Twenties.stretch_after());
    }

    #[test]
    fn water_pace_spreads_the_target_over_the_day() {
        let at = |hour| NaiveTime::from_hms_opt(hour, 0, 0).unwrap();
        assert_eq!(water_expected(8, at(8)), 0);
        assert_eq!(water_expected(8, at(15)), 4);
        assert_eq!(water_expected(8, at(23)), 8);
    }

    #[test]
    fn logs_count_toward_today_and_health_can_be_turned_off() {
        let connection = test_connection();
        log(&connection, HabitKind::Water, now()).unwrap();
        log(&connection, HabitKind::Water, now()).unwrap();
        log(&connection, HabitKind::Stretch, now()).unwrap();

        let today = overview(&connection, now() - Duration::hours(1)).unwrap();
        assert_eq!(today.water_today, 2);
        assert_eq!(today.water_target, 8);
        assert_eq!(today.stretches_today, 1);
        assert!(today.enabled);

        set_enabled(&connection, false).unwrap();
        assert!(!is_enabled(&connection).unwrap());
    }

    #[test]
    fn medicines_create_critical_daily_reminders() {
        let connection = test_connection();
        let times = [
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(21, 0, 0).unwrap(),
        ];
        let medicine =
            add_medicine(&connection, "Vitamin D", &times, Tz::Asia__Karachi, now()).unwrap();
        assert_eq!(medicine.reminders.len(), 2);
        assert!(medicine
            .reminders
            .iter()
            .all(|r| r.critical && r.rrule.as_deref() == Some("FREQ=DAILY")));

        delete_medicine(&connection, medicine.habit.id).unwrap();
        assert!(overview(&connection, now()).unwrap().medicines.is_empty());
        let water = habit(&connection, HabitKind::Water).unwrap();
        assert!(delete_medicine(&connection, water.id).is_err());
    }
}
