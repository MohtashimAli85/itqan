pub mod coach;
pub mod planner;
pub mod rewards;
pub mod rhythm;

use chrono::{DateTime, Utc};
use tauri::{App, AppHandle, Manager};

pub use itqan_contracts::AppEvent;
pub use itqan_core::bus::{publish, Signal, Subscriber};
use itqan_core::bus::{Bus, SignalSink};
use itqan_core::error::AppError;
use itqan_core::module::Module;

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
    let bus = modules
        .iter()
        .flat_map(|module| {
            module
                .subscribers()
                .into_iter()
                .map(|subscriber| (module.id(), subscriber))
        })
        .fold(
            Bus::new(CoachSink).register("rhythm", rhythm::RhythmAgent),
            |bus, (owner, subscriber)| bus.register(owner, subscriber),
        )
        .register("rewards", rewards::RewardsAgent);
    app.manage(bus);
    app.manage(coach::Coach::default());
}

pub use itqan_core::overlay::action;
