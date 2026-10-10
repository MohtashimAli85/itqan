pub mod agent;
pub mod commands;
mod repo;
pub mod rewards;

use itqan_core::bus::Subscriber;
use itqan_core::module::{Migration, Module};

pub use agent::RewardEarned;

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "progress_tables",
    sql: include_str!("../migrations/0001_progress_tables.sql"),
}];

pub struct ProgressModule;

impl Module for ProgressModule {
    fn id(&self) -> &'static str {
        "progress"
    }

    fn migrations(&self) -> &'static [Migration] {
        MIGRATIONS
    }

    fn subscribers(&self) -> Vec<Box<dyn Subscriber>> {
        vec![Box::new(agent::RewardsAgent)]
    }
}

#[cfg(test)]
fn test_connection() -> rusqlite::Connection {
    itqan_core::db::test_connection_with(&[&ProgressModule])
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use itqan_core::db::migrations;
    use rusqlite::Connection;

    use super::*;

    #[test]
    fn xp_events_and_the_streak_survive_the_rename() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .unwrap();
        migrations::run(&mut connection).unwrap();
        let at = Utc.with_ymd_and_hms(2026, 10, 10, 9, 0, 0).unwrap();
        connection
            .execute(
                "INSERT INTO tasks (title, created_at, updated_at) VALUES ('Ship 2.0.7', ?1, ?1)",
                [at],
            )
            .unwrap();
        let task = connection.last_insert_rowid();
        connection
            .execute(
                "INSERT INTO xp_events (source, amount, task_id, reference_id, at) VALUES ('task', 30, ?1, ?1, ?2)",
                rusqlite::params![task, at],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO xp_events (source, amount, at) VALUES ('habit', 3, ?1)",
                [at],
            )
            .unwrap();
        connection
            .execute(
                "UPDATE streaks SET current = 4, best = 9, freezes = 2, last_active_day = '2026-10-09' WHERE id = 1",
                [],
            )
            .unwrap();

        migrations::run_modules(&mut connection, &[&ProgressModule]).unwrap();

        assert_eq!(repo::total_xp(&connection).unwrap(), 33);
        let streak = repo::streak(&connection).unwrap();
        assert_eq!((streak.current, streak.best, streak.freezes), (4, 9, 2));
        assert_eq!(
            streak.last_active_day.unwrap().to_string(),
            "2026-10-09".to_owned()
        );
        let duplicate = connection.execute(
            "INSERT OR IGNORE INTO progress_xp_events (source, amount, reference_id, at) VALUES ('task', 30, ?1, ?2)",
            rusqlite::params![task, at],
        );
        assert_eq!(duplicate.unwrap(), 0);
        connection
            .execute("DELETE FROM tasks WHERE id = ?1", [task])
            .unwrap();
        let orphaned: Option<i64> = connection
            .query_row(
                "SELECT task_id FROM progress_xp_events WHERE source = 'task'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(orphaned, None);
        let old: i64 = connection
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE name IN ('xp_events', 'streaks', 'xp_events_at', 'xp_events_once')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(old, 0);
    }
}
