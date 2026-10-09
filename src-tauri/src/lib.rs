mod agents;
mod ai;
mod commands;
mod db;
mod domain;
mod error;
mod overlay;
mod platform;
mod prayer;
mod scheduler;
mod tray;

use tauri::Manager;
use tauri_specta::{collect_commands, collect_events, Builder};

use crate::db::Database;

const BINDINGS_PATH: &str = "../src/shared/bindings/bindings.ts";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[allow(clippy::expect_used)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::app_info::get_app_info,
            commands::overlay::set_overlay_hit_areas,
            commands::overlay::get_overlay_state,
            commands::overlay::set_orb_state,
            commands::overlay::set_follow_mode,
            commands::overlay::set_orb_progress,
            commands::overlay::show_bubble,
            commands::overlay::resolve_bubble,
        ])
        .events(collect_events![
            overlay::OverlayCursor,
            overlay::OverlayChanged
        ]);

    #[cfg(debug_assertions)]
    builder
        .export(specta_typescript::Typescript::default(), BINDINGS_PATH)
        .expect("failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(platform::plugin())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main_window(app);
        }))
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let database = Database::open(&data_dir.join("itqan.db"))?;
            app.manage(database);
            tracing::info!("database ready");
            tray::setup(app)?;
            overlay::setup(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
