use chrono::{DateTime, Duration, Utc};
use chrono_tz::Tz;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::db::reminders as repo;
use crate::domain::recurrence;
use crate::domain::tasks::TaskId;
use crate::error::AppError;

pub type ReminderId = i32;

pub const SNOOZE_MINUTES: i64 = 10;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Reminder {
    pub id: ReminderId,
    pub task_id: Option<TaskId>,
    pub title: String,
    pub anchor_at: DateTime<Utc>,
    pub next_at: Option<DateTime<Utc>>,
    pub rrule: Option<String>,
    pub timezone: String,
    pub critical: bool,
    pub snoozed_until: Option<DateTime<Utc>>,
    pub last_fired_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ReminderInput {
    pub task_id: Option<TaskId>,
    pub title: Option<String>,
    pub at: DateTime<Utc>,
    pub rrule: Option<String>,
    pub critical: bool,
}

impl Reminder {
    fn timezone(&self) -> Tz {
        self.timezone.parse().unwrap_or(Tz::UTC)
    }
}

pub fn create(
    connection: &Connection,
    input: ReminderInput,
    timezone: Tz,
    now: DateTime<Utc>,
) -> Result<Reminder, AppError> {
    let title = input
        .title
        .map(|title| title.trim().to_owned())
        .filter(|title| !title.is_empty());
    if input.task_id.is_none() && title.is_none() {
        return Err(AppError::InvalidInput(
            "a reminder needs a task or a title".into(),
        ));
    }
    let rrule = input
        .rrule
        .map(|rule| rule.trim().to_owned())
        .filter(|rule| !rule.is_empty());
    let next_at = match &rrule {
        Some(rule) => {
            recurrence::validate(rule, input.at, timezone)?;
            if input.at >= now {
                Some(input.at)
            } else {
                recurrence::next_after(rule, input.at, timezone, now)?
            }
        }
        None => Some(input.at),
    };
    let id = repo::insert(
        connection,
        &repo::NewReminder {
            task_id: input.task_id,
            title: title.as_deref(),
            anchor_at: input.at,
            next_at,
            rrule: rrule.as_deref(),
            timezone: timezone.name(),
            critical: input.critical,
            created_at: now,
        },
    )?;
    get(connection, id)
}

pub fn get(connection: &Connection, id: ReminderId) -> Result<Reminder, AppError> {
    repo::find(connection, id)?.ok_or_else(|| AppError::NotFound(format!("reminder {id}")))
}

pub fn list_for_task(connection: &Connection, task_id: TaskId) -> Result<Vec<Reminder>, AppError> {
    repo::list_for_task(connection, task_id)
}

pub fn due(connection: &Connection, now: DateTime<Utc>) -> Result<Vec<Reminder>, AppError> {
    repo::due(connection, now)
}

pub fn next_due_at(connection: &Connection) -> Result<Option<DateTime<Utc>>, AppError> {
    repo::next_due_at(connection)
}

pub fn mark_fired(
    connection: &Connection,
    reminder: &Reminder,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    let snoozed_until = reminder.snoozed_until.filter(|snoozed| *snoozed > now);
    let next_at = match reminder.next_at {
        Some(next) if next <= now => match &reminder.rrule {
            Some(rule) => {
                recurrence::next_after(rule, reminder.anchor_at, reminder.timezone(), now)?
            }
            None => None,
        },
        other => other,
    };
    repo::update_schedule(connection, reminder.id, next_at, snoozed_until, Some(now))
}

pub fn snooze(
    connection: &Connection,
    id: ReminderId,
    now: DateTime<Utc>,
) -> Result<Reminder, AppError> {
    let reminder = get(connection, id)?;
    let until = now + Duration::minutes(SNOOZE_MINUTES);
    repo::update_schedule(
        connection,
        id,
        reminder.next_at,
        Some(until),
        reminder.last_fired_at,
    )?;
    get(connection, id)
}

pub fn delete(connection: &Connection, id: ReminderId) -> Result<(), AppError> {
    get(connection, id)?;
    repo::delete(connection, id)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::db::test_connection;

    fn at(hour: u32, minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 10, hour, minute, 0).unwrap()
    }

    fn standalone(time: DateTime<Utc>, rrule: Option<&str>) -> ReminderInput {
        ReminderInput {
            task_id: None,
            title: Some("Take medicine".into()),
            at: time,
            rrule: rrule.map(str::to_owned),
            critical: true,
        }
    }

    #[test]
    fn a_reminder_needs_a_task_or_a_title() {
        let connection = test_connection();
        let input = ReminderInput {
            title: Some("  ".into()),
            ..standalone(at(9, 0), None)
        };
        assert!(create(&connection, input, Tz::UTC, at(8, 0)).is_err());
    }

    #[test]
    fn task_reminders_take_the_task_title() {
        let connection = test_connection();
        let task = crate::domain::tasks::create(
            &connection,
            crate::domain::tasks::TaskInput {
                title: "Buy dahi".into(),
                ..Default::default()
            },
            at(8, 0),
        )
        .unwrap();
        let reminder = create(
            &connection,
            ReminderInput {
                task_id: Some(task.id),
                title: None,
                at: at(14, 0),
                rrule: None,
                critical: false,
            },
            Tz::UTC,
            at(8, 0),
        )
        .unwrap();
        assert_eq!(reminder.title, "Buy dahi");
    }

    #[test]
    fn one_off_reminders_fire_once() {
        let connection = test_connection();
        let reminder = create(&connection, standalone(at(9, 0), None), Tz::UTC, at(8, 0)).unwrap();

        assert!(due(&connection, at(8, 59)).unwrap().is_empty());
        let fired = due(&connection, at(9, 0)).unwrap();
        assert_eq!(fired.len(), 1);

        mark_fired(&connection, &fired[0], at(9, 0)).unwrap();
        assert!(due(&connection, at(23, 0)).unwrap().is_empty());
        assert_eq!(get(&connection, reminder.id).unwrap().next_at, None);
    }

    #[test]
    fn recurring_reminders_advance_and_skip_missed_occurrences() {
        let connection = test_connection();
        let anchor = Utc.with_ymd_and_hms(2026, 10, 1, 9, 0, 0).unwrap();
        let reminder = create(
            &connection,
            standalone(anchor, Some("FREQ=DAILY")),
            Tz::UTC,
            at(8, 0),
        )
        .unwrap();
        assert_eq!(reminder.next_at, Some(at(9, 0)));

        let fired = due(&connection, at(9, 30)).unwrap();
        mark_fired(&connection, &fired[0], at(9, 30)).unwrap();

        let next = get(&connection, reminder.id).unwrap().next_at.unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 10, 11, 9, 0, 0).unwrap());
    }

    #[test]
    fn snoozing_brings_a_fired_reminder_back() {
        let connection = test_connection();
        let reminder = create(&connection, standalone(at(9, 0), None), Tz::UTC, at(8, 0)).unwrap();
        let fired = due(&connection, at(9, 0)).unwrap();
        mark_fired(&connection, &fired[0], at(9, 0)).unwrap();

        snooze(&connection, reminder.id, at(9, 1)).unwrap();

        assert!(due(&connection, at(9, 10)).unwrap().is_empty());
        assert_eq!(due(&connection, at(9, 11)).unwrap().len(), 1);
        assert_eq!(next_due_at(&connection).unwrap(), Some(at(9, 11)));
    }
}
