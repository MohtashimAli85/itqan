use std::sync::atomic::{AtomicI64, Ordering};

use chrono::{DateTime, Duration, Utc};

pub const BREAK_RESETS_AFTER: Duration = Duration::minutes(5);

#[derive(Debug, Default)]
pub struct Activity {
    last: AtomicI64,
    streak_start: AtomicI64,
}

impl Activity {
    pub fn touch(&self) {
        self.touch_at(Utc::now());
    }

    fn touch_at(&self, now: DateTime<Utc>) {
        let now = now.timestamp();
        let last = self.last.swap(now, Ordering::Relaxed);
        if last == 0 || now - last > BREAK_RESETS_AFTER.num_seconds() {
            self.streak_start.store(now, Ordering::Relaxed);
        }
    }

    pub fn active_within(&self, now: DateTime<Utc>, window: Duration) -> bool {
        let last = self.last.load(Ordering::Relaxed);
        last > 0 && now.timestamp() - last <= window.num_seconds()
    }

    pub fn sitting_for(&self, now: DateTime<Utc>) -> Duration {
        if !self.active_within(now, BREAK_RESETS_AFTER) {
            return Duration::zero();
        }
        let start = self.streak_start.load(Ordering::Relaxed);
        Duration::seconds((now.timestamp() - start).max(0))
    }

    pub fn reset_streak(&self) {
        self.streak_start
            .store(Utc::now().timestamp(), Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn at(minute: i64) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 10, 9, 0, 0).unwrap() + Duration::minutes(minute)
    }

    #[test]
    fn sitting_time_grows_with_activity_and_resets_after_a_break() {
        let activity = Activity::default();
        assert_eq!(activity.sitting_for(at(0)), Duration::zero());

        for minute in 0..=50 {
            activity.touch_at(at(minute));
        }
        assert_eq!(activity.sitting_for(at(50)), Duration::minutes(50));

        activity.touch_at(at(57));
        assert_eq!(activity.sitting_for(at(57)), Duration::zero());
        activity.touch_at(at(60));
        assert_eq!(activity.sitting_for(at(60)), Duration::minutes(3));
    }

    #[test]
    fn idle_people_are_not_sitting() {
        let activity = Activity::default();
        activity.touch_at(at(0));
        assert_eq!(activity.sitting_for(at(10)), Duration::zero());
    }
}
