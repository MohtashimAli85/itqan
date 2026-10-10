use tauri::plugin::{Builder, TauriPlugin};
use tauri::{WebviewWindow, Wry};

use std::time::Duration;

use crate::error::AppError;

pub fn plugin() -> TauriPlugin<Wry> {
    Builder::new("platform").build()
}

pub fn configure_overlay(_window: &WebviewWindow) -> Result<(), AppError> {
    Ok(())
}

pub fn focus_overlay(window: &WebviewWindow, focused: bool) -> Result<(), AppError> {
    if focused {
        window.set_focus()?;
    }
    Ok(())
}

pub fn since_last_input() -> Option<Duration> {
    None
}

pub fn since_last_key() -> Option<Duration> {
    None
}
