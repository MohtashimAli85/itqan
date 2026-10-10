use tauri::AppHandle;

use crate::bus::Subscriber;
use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Migration {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
}

pub trait Module: Send + Sync + 'static {
    fn id(&self) -> &'static str;

    fn migrations(&self) -> &'static [Migration] {
        &[]
    }

    fn setup(&self, _app: &AppHandle) -> Result<(), AppError> {
        Ok(())
    }

    fn subscribers(&self) -> Vec<Box<dyn Subscriber>> {
        Vec::new()
    }
}
