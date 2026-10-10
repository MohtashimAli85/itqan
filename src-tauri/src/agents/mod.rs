pub mod coach;
pub mod health;
pub mod planner;
pub mod rewards;
pub mod rhythm;

use chrono::{DateTime, Utc};
use tauri::{App, AppHandle, Manager};

pub use itqan_contracts::AppEvent;
pub use itqan_core::bus::{publish, Signal, Subscriber};
use itqan_core::bus::{Bus, SignalSink};
use itqan_core::error::AppError;

struct CoachSink;

impl SignalSink for CoachSink {
    fn consider(
        &self,
        app: &AppHandle,
        signals: Vec<Signal>,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        coach::consider(app, signals, now)
    }
}

pub fn setup(app: &App) {
    app.manage(
        Bus::new(CoachSink)
            .register("rhythm", rhythm::RhythmAgent)
            .register("health", health::HealthAgent)
            .register("rewards", rewards::RewardsAgent),
    );
    app.manage(coach::Coach::default());
}

pub use itqan_core::overlay::action;
