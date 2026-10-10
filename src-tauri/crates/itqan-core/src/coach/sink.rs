use chrono::{DateTime, Utc};
use tauri::{App, AppHandle, Manager};

use super::Coach;
use crate::bus::{Bus, Signal, SignalSink};
use crate::error::AppError;
use crate::module::Module;
use crate::rhythm;

struct CoachSink;

impl SignalSink for CoachSink {
    fn consider(
        &self,
        app: &AppHandle,
        signals: Vec<Signal>,
        now: DateTime<Utc>,
    ) -> Result<(), AppError> {
        super::consider(app, signals, now)
    }
}

pub fn setup(app: &App, modules: &[&dyn Module]) {
    let mut bus = Bus::new(CoachSink).register("rhythm", rhythm::RhythmAgent);
    for module in modules {
        for subscriber in module.subscribers() {
            bus = bus.register(module.id(), subscriber);
        }
    }
    app.manage(bus);
    app.manage(Coach::default());
}
