use std::sync::Arc;

use tauri::{AppHandle, Manager};
use tokio::sync::Notify;

use crate::error::AppError;

pub struct Scheduler {
    wake: Arc<Notify>,
}

impl Scheduler {
    pub fn new(wake: Arc<Notify>) -> Self {
        Self { wake }
    }

    pub fn wake(&self) {
        self.wake.notify_one();
    }
}

type RefreshFn = dyn Fn(&AppHandle) -> Result<(), AppError> + Send + Sync;

pub struct Refresher(Box<RefreshFn>);

impl Refresher {
    pub fn new(
        refresh: impl Fn(&AppHandle) -> Result<(), AppError> + Send + Sync + 'static,
    ) -> Self {
        Self(Box::new(refresh))
    }
}

pub fn refresh(app: &AppHandle) -> Result<(), AppError> {
    let refresher = app
        .try_state::<Refresher>()
        .ok_or_else(|| AppError::NotFound("scheduler refresher".into()))?;
    (refresher.0)(app)
}
