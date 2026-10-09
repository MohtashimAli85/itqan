use std::sync::atomic::{AtomicI64, Ordering};

use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Default)]
pub struct Activity(AtomicI64);

impl Activity {
    pub fn touch(&self) {
        self.0.store(Utc::now().timestamp(), Ordering::Relaxed);
    }

    pub fn active_within(&self, now: DateTime<Utc>, window: Duration) -> bool {
        let last = self.0.load(Ordering::Relaxed);
        last > 0 && now.timestamp() - last <= window.num_seconds()
    }
}
