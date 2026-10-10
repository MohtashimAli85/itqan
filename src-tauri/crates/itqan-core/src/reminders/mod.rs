pub mod commands;
mod repo;

use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use chrono_tz::Tz;
use itqan_contracts::{HabitId, TaskId};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::error::AppError;
use crate::ports::{Ports, ReminderTargetKind, TargetInfo};
use crate::recurrence;
use repo::ReminderRow;

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

fn target(reminder: &ReminderRow) -> Option<(ReminderTargetKind, i32)> {
    match (reminder.task_id, reminder.habit_id) {
        (Some(id), _) => Some((ReminderTargetKind::Task, id)),
        (None, Some(id)) => Some((ReminderTargetKind::Habit, id)),
        (None, None) => None,
    }
}

fn resolve_all(
    connection: &Connection,
    ports: &Ports,
    rows: Vec<ReminderRow>,
) -> Result<Vec<(Reminder, bool)>, AppError> {
    let mut wanted: HashMap<ReminderTargetKind, Vec<i32>> = HashMap::new();
    for (kind, id) in rows.iter().filter_map(target) {
        wanted.entry(kind).or_default().push(id);
    }
    let mut known = HashMap::new();
    for (kind, mut ids) in wanted {
        ids.sort_unstable();
        ids.dedup();
        let infos = match ports.reminder_target(kind)? {
            Some(owner) => Some(owner.describe(connection, &ids)?),
            None => {
                tracing::debug!(?kind, "reminders held: no target registered");
                None
            }
        };
        known.insert(kind, infos);
    }
    Ok(rows
        .into_iter()
        .map(|row| {
            let info = match target(&row) {
                None => Some(TargetInfo {
                    title: None,
                    active: true,
                }),
                Some((kind, id)) => known
                    .get(&kind)
                    .and_then(Option::as_ref)
                    .and_then(|infos: &HashMap<i32, TargetInfo>| infos.get(&id))
                    .cloned(),
            };
            let active = info.as_ref().is_some_and(|info| info.active);
            (own(row, info.and_then(|info| info.title)), active)
        })
        .collect())
}

fn reminders_only(resolved: Vec<(Reminder, bool)>) -> Vec<Reminder> {
    resolved.into_iter().map(|(reminder, _)| reminder).collect()
}

fn own(row: ReminderRow, target_title: Option<String>) -> Reminder {
    Reminder {
        id: row.id,
        task_id: row.task_id,
        title: target_title.or(row.title).unwrap_or_default(),
        anchor_at: row.anchor_at,
        next_at: row.next_at,
        rrule: row.rrule,
        timezone: row.timezone,
        critical: row.critical,
        snoozed_until: row.snoozed_until,
        last_fired_at: row.last_fired_at,
    }
}

fn find_row(connection: &Connection, id: ReminderId) -> Result<ReminderRow, AppError> {
    repo::find(connection, id)?.ok_or_else(|| AppError::NotFound(format!("reminder {id}")))
}

pub fn create(
    connection: &Connection,
    ports: &Ports,
    input: ReminderInput,
    timezone: Tz,
    now: DateTime<Utc>,
) -> Result<Reminder, AppError> {
    let id = insert(connection, input, timezone, now)?;
    get(connection, ports, id)
}

pub fn insert(
    connection: &Connection,
    input: ReminderInput,
    timezone: Tz,
    now: DateTime<Utc>,
) -> Result<ReminderId, AppError> {
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
    repo::insert(
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
    )
}

pub fn get(connection: &Connection, ports: &Ports, id: ReminderId) -> Result<Reminder, AppError> {
    reminders_only(resolve_all(
        connection,
        ports,
        vec![find_row(connection, id)?],
    )?)
    .pop()
    .ok_or_else(|| AppError::NotFound(format!("reminder {id}")))
}

pub fn target_of(
    connection: &Connection,
    id: ReminderId,
) -> Result<Option<(ReminderTargetKind, i32)>, AppError> {
    Ok(target(&find_row(connection, id)?))
}

pub fn list_for_task(
    connection: &Connection,
    ports: &Ports,
    task_id: TaskId,
) -> Result<Vec<Reminder>, AppError> {
    Ok(reminders_only(resolve_all(
        connection,
        ports,
        repo::list_for_task(connection, task_id)?,
    )?))
}

pub fn list_for_habit(
    connection: &Connection,
    habit_id: HabitId,
) -> Result<Vec<Reminder>, AppError> {
    Ok(repo::list_for_habit(connection, habit_id)?
        .into_iter()
        .map(|row| own(row, None))
        .collect())
}

pub fn attach_habit(
    connection: &Connection,
    id: ReminderId,
    habit_id: HabitId,
) -> Result<(), AppError> {
    repo::set_habit(connection, id, habit_id)
}

pub fn due(
    connection: &Connection,
    ports: &Ports,
    now: DateTime<Utc>,
) -> Result<Vec<Reminder>, AppError> {
    Ok(resolve_all(connection, ports, repo::due(connection, now)?)?
        .into_iter()
        .filter_map(|(reminder, active)| active.then_some(reminder))
        .collect())
}

pub fn next_due_at(
    connection: &Connection,
    ports: &Ports,
) -> Result<Option<DateTime<Utc>>, AppError> {
    let (rows, due_times): (Vec<_>, Vec<_>) = repo::pending(connection)?.into_iter().unzip();
    Ok(resolve_all(connection, ports, rows)?
        .into_iter()
        .zip(due_times)
        .find_map(|((_, active), due_at)| active.then_some(due_at)))
}

pub fn mark_fired(
    connection: &Connection,
    reminder: &Reminder,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    let snoozed_until = reminder.snoozed_until.filter(|snoozed| *snoozed > now);
    let next_at = advance(
        reminder.next_at,
        reminder.rrule.as_deref(),
        reminder.anchor_at,
        &reminder.timezone,
        now,
    )?;
    repo::update_schedule(connection, reminder.id, next_at, snoozed_until, Some(now))
}

fn advance(
    next_at: Option<DateTime<Utc>>,
    rrule: Option<&str>,
    anchor_at: DateTime<Utc>,
    timezone: &str,
    now: DateTime<Utc>,
) -> Result<Option<DateTime<Utc>>, AppError> {
    match next_at {
        Some(next) if next <= now => match rrule {
            Some(rule) => {
                recurrence::next_after(rule, anchor_at, timezone.parse().unwrap_or(Tz::UTC), now)
            }
            None => Ok(None),
        },
        other => Ok(other),
    }
}

pub fn snooze(
    connection: &Connection,
    ports: &Ports,
    id: ReminderId,
    now: DateTime<Utc>,
) -> Result<Reminder, AppError> {
    let reminder = find_row(connection, id)?;
    let until = now + Duration::minutes(SNOOZE_MINUTES);
    repo::update_schedule(
        connection,
        id,
        reminder.next_at,
        Some(until),
        reminder.last_fired_at,
    )?;
    get(connection, ports, id)
}

pub fn hold_until(
    connection: &Connection,
    id: ReminderId,
    until: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<(), AppError> {
    let reminder = find_row(connection, id)?;
    let next_at = advance(
        reminder.next_at,
        reminder.rrule.as_deref(),
        reminder.anchor_at,
        &reminder.timezone,
        now,
    )?;
    repo::update_schedule(connection, id, next_at, Some(until), reminder.last_fired_at)
}

pub fn delete(connection: &Connection, id: ReminderId) -> Result<(), AppError> {
    find_row(connection, id)?;
    repo::delete(connection, id)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use tauri::AppHandle;

    use super::*;
    use crate::db::test_connection;
    use crate::ports::ReminderTarget;

    struct SqlTasks;

    impl ReminderTarget for SqlTasks {
        fn describe(
            &self,
            connection: &Connection,
            ids: &[i32],
        ) -> Result<HashMap<i32, TargetInfo>, AppError> {
            let mut infos = HashMap::new();
            for id in ids {
                if let Ok(info) = connection.query_row(
                    "SELECT title, status = 'open' FROM tasks WHERE id = ?1",
                    [id],
                    |row| {
                        Ok(TargetInfo {
                            title: Some(row.get(0)?),
                            active: row.get(1)?,
                        })
                    },
                ) {
                    infos.insert(*id, info);
                }
            }
            Ok(infos)
        }

        fn complete(&self, _: &AppHandle, _: i32) -> Result<(), AppError> {
            Ok(())
        }
    }

    fn ports() -> Ports {
        let ports = Ports::default();
        ports
            .set_reminder_target(ReminderTargetKind::Task, SqlTasks)
            .unwrap();
        ports
    }

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

    fn task(connection: &Connection, title: &str, status: &str) -> TaskId {
        connection
            .execute(
                "INSERT INTO tasks (title, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
                rusqlite::params![title, status, at(8, 0)],
            )
            .unwrap();
        TaskId::try_from(connection.last_insert_rowid()).unwrap()
    }

    fn task_reminder(task_id: TaskId, time: DateTime<Utc>) -> ReminderInput {
        ReminderInput {
            task_id: Some(task_id),
            title: None,
            at: time,
            rrule: None,
            critical: false,
        }
    }

    #[test]
    fn a_reminder_needs_a_task_or_a_title() {
        let connection = test_connection();
        let input = ReminderInput {
            title: Some("  ".into()),
            ..standalone(at(9, 0), None)
        };
        assert!(create(&connection, &ports(), input, Tz::UTC, at(8, 0)).is_err());
    }

    #[test]
    fn task_reminders_take_the_task_title() {
        let connection = test_connection();
        let task_id = task(&connection, "Buy dahi", "open");
        let reminder = create(
            &connection,
            &ports(),
            task_reminder(task_id, at(14, 0)),
            Tz::UTC,
            at(8, 0),
        )
        .unwrap();
        assert_eq!(reminder.title, "Buy dahi");
    }

    #[test]
    fn reminders_for_finished_or_unowned_targets_do_not_fire() {
        let connection = test_connection();
        let ports = ports();
        let done = task(&connection, "Done already", "done");
        let open = task(&connection, "Still open", "open");
        create(
            &connection,
            &ports,
            task_reminder(done, at(9, 0)),
            Tz::UTC,
            at(8, 0),
        )
        .unwrap();
        create(
            &connection,
            &ports,
            task_reminder(open, at(9, 30)),
            Tz::UTC,
            at(8, 0),
        )
        .unwrap();

        let fired = due(&connection, &ports, at(10, 0)).unwrap();
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].title, "Still open");
        assert_eq!(next_due_at(&connection, &ports).unwrap(), Some(at(9, 30)));

        let unowned = Ports::default();
        assert!(due(&connection, &unowned, at(10, 0)).unwrap().is_empty());
        assert_eq!(next_due_at(&connection, &unowned).unwrap(), None);
    }

    #[test]
    fn one_off_reminders_fire_once() {
        let connection = test_connection();
        let ports = ports();
        let reminder = create(
            &connection,
            &ports,
            standalone(at(9, 0), None),
            Tz::UTC,
            at(8, 0),
        )
        .unwrap();

        assert!(due(&connection, &ports, at(8, 59)).unwrap().is_empty());
        let fired = due(&connection, &ports, at(9, 0)).unwrap();
        assert_eq!(fired.len(), 1);

        mark_fired(&connection, &fired[0], at(9, 0)).unwrap();
        assert!(due(&connection, &ports, at(23, 0)).unwrap().is_empty());
        assert_eq!(get(&connection, &ports, reminder.id).unwrap().next_at, None);
    }

    #[test]
    fn recurring_reminders_advance_and_skip_missed_occurrences() {
        let connection = test_connection();
        let ports = ports();
        let anchor = Utc.with_ymd_and_hms(2026, 10, 1, 9, 0, 0).unwrap();
        let reminder = create(
            &connection,
            &ports,
            standalone(anchor, Some("FREQ=DAILY")),
            Tz::UTC,
            at(8, 0),
        )
        .unwrap();
        assert_eq!(reminder.next_at, Some(at(9, 0)));

        let fired = due(&connection, &ports, at(9, 30)).unwrap();
        mark_fired(&connection, &fired[0], at(9, 30)).unwrap();

        let next = get(&connection, &ports, reminder.id)
            .unwrap()
            .next_at
            .unwrap();
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 10, 11, 9, 0, 0).unwrap());
    }

    #[test]
    fn holding_a_reminder_moves_it_past_quiet_time() {
        let connection = test_connection();
        let ports = ports();
        let reminder = create(
            &connection,
            &ports,
            standalone(at(9, 0), None),
            Tz::UTC,
            at(8, 0),
        )
        .unwrap();

        hold_until(&connection, reminder.id, at(9, 25), at(9, 5)).unwrap();

        assert!(due(&connection, &ports, at(9, 10)).unwrap().is_empty());
        assert_eq!(due(&connection, &ports, at(9, 25)).unwrap().len(), 1);
    }

    #[test]
    fn snoozing_brings_a_fired_reminder_back() {
        let connection = test_connection();
        let ports = ports();
        let reminder = create(
            &connection,
            &ports,
            standalone(at(9, 0), None),
            Tz::UTC,
            at(8, 0),
        )
        .unwrap();
        let fired = due(&connection, &ports, at(9, 0)).unwrap();
        mark_fired(&connection, &fired[0], at(9, 0)).unwrap();

        snooze(&connection, &ports, reminder.id, at(9, 1)).unwrap();

        assert!(due(&connection, &ports, at(9, 10)).unwrap().is_empty());
        assert_eq!(due(&connection, &ports, at(9, 11)).unwrap().len(), 1);
        assert_eq!(next_due_at(&connection, &ports).unwrap(), Some(at(9, 11)));
    }
}
