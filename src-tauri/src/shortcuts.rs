use tauri::plugin::TauriPlugin;
use tauri::{App, AppHandle, Manager, Wry};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::error::AppError;
use crate::overlay::{self, OverlayStore};

fn panel_shortcut() -> Shortcut {
    Shortcut::new(Some(Modifiers::ALT), Code::Space)
}

fn toggle_panel(app: &AppHandle) -> Result<(), AppError> {
    let store = app.state::<OverlayStore>();
    if store.snapshot()?.panel_open {
        overlay::publish(app, store.set_panel_open(false)?)?;
        overlay::set_keyboard_focus(app, false)
    } else {
        overlay::open_panel(app)
    }
}

pub fn plugin() -> TauriPlugin<Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, shortcut, event| {
            if event.state() == ShortcutState::Pressed && *shortcut == panel_shortcut() {
                if let Err(error) = toggle_panel(app) {
                    tracing::warn!(%error, "panel shortcut failed");
                }
            }
        })
        .build()
}

pub fn register(app: &App) {
    if let Err(error) = app.global_shortcut().register(panel_shortcut()) {
        tracing::warn!(%error, "could not register the panel shortcut");
    }
}
