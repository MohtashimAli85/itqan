use tauri::plugin::TauriPlugin;
use tauri::{Manager, WebviewWindow, Wry};
use tauri_nspanel::{
    tauri_panel, CollectionBehavior, ManagerExt, PanelLevel, StyleMask, WebviewWindowExt,
};

use std::time::Duration;

use crate::error::AppError;

tauri_panel! {
    panel!(OverlayPanel {
        config: {
            can_become_key_window: true,
            can_become_main_window: false,
            is_floating_panel: true
        }
    })
}

pub fn plugin() -> TauriPlugin<Wry> {
    tauri_nspanel::init()
}

pub fn configure_overlay(window: &WebviewWindow) -> Result<(), AppError> {
    let panel = window.to_panel::<OverlayPanel>()?;
    panel
        .add_style_mask(StyleMask::empty().nonactivating_panel().value())
        .map_err(|error| AppError::Platform(error.to_string()))?;
    panel.set_level(PanelLevel::Status.value());
    panel.set_collection_behavior(
        CollectionBehavior::new()
            .can_join_all_spaces()
            .full_screen_auxiliary()
            .stationary()
            .ignores_cycle()
            .value(),
    );
    panel.set_has_shadow(false);
    panel.set_hides_on_deactivate(false);
    Ok(())
}

pub fn focus_overlay(window: &WebviewWindow, focused: bool) -> Result<(), AppError> {
    let panel = window
        .app_handle()
        .get_webview_panel(window.label())
        .map_err(|_| AppError::Platform("overlay panel is missing".into()))?;
    if focused {
        panel.show_and_make_key();
    } else if panel.as_panel().isKeyWindow() {
        panel.hide();
        panel.order_front_regardless();
    }
    Ok(())
}

const HID_SYSTEM_STATE: i32 = 1;
const KEY_DOWN: u32 = 10;
const ANY_INPUT: u32 = u32::MAX;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventSourceSecondsSinceLastEventType(source_state: i32, event_type: u32) -> f64;
}

fn seconds_since(event_type: u32) -> Option<Duration> {
    // SAFETY: a pure query of the window server's input clock; it takes plain integers,
    // touches no memory we own, and needs no permission.
    let seconds = unsafe { CGEventSourceSecondsSinceLastEventType(HID_SYSTEM_STATE, event_type) };
    (seconds.is_finite() && seconds >= 0.0).then(|| Duration::from_secs_f64(seconds))
}

pub fn since_last_input() -> Option<Duration> {
    seconds_since(ANY_INPUT)
}

pub fn since_last_key() -> Option<Duration> {
    seconds_since(KEY_DOWN)
}
