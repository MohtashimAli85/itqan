use tauri::{AppHandle, Manager};

use crate::error::AppError;

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
    app.try_state::<Refresher>()
        .map_or(Ok(()), |refresher| (refresher.0)(app))
}
