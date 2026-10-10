pub mod rewards;

use chrono::{DateTime, Utc};
use tauri::{App, AppHandle, Manager};

pub use itqan_contracts::AppEvent;
use itqan_core::bus::{Bus, SignalSink};
pub use itqan_core::bus::{Signal, Subscriber};
use itqan_core::coach;
use itqan_core::error::AppError;
use itqan_core::module::Module;
use itqan_core::rhythm;

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

pub fn setup(app: &App, modules: &[&dyn Module]) {
    let mut bus = Bus::new(CoachSink).register("rhythm", rhythm::RhythmAgent);
    for module in modules {
        for subscriber in module.subscribers() {
            bus = bus.register(module.id(), subscriber);
        }
    }
    let bus = bus.register("rewards", rewards::RewardsAgent);
    app.manage(bus);
    app.manage(coach::Coach::default());
}
