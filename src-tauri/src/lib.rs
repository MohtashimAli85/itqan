mod agents;
mod commands;
mod db;
mod domain;
mod overlay;
mod platform;
mod prayer;
mod scheduler;
mod shortcuts;
mod tray;

use tauri::Manager;
use tauri_specta::{collect_commands, collect_events, Builder};

use crate::db::Database;

#[cfg(any(debug_assertions, test))]
const BINDINGS_PATH: &str = "../src/shared/bindings/bindings.ts";

fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::app_info::get_app_info,
            commands::overlay::set_overlay_hit_areas,
            commands::overlay::get_overlay_state,
            commands::overlay::set_orb_state,
            commands::overlay::set_follow_mode,
            commands::overlay::set_orb_progress,
            commands::overlay::show_bubble,
            commands::overlay::resolve_bubble,
            commands::overlay::set_panel_open,
            commands::tasks::list_tasks,
            commands::tasks::create_task,
            commands::tasks::update_task,
            commands::tasks::set_task_status,
            commands::tasks::set_task_top_three,
            commands::tasks::delete_task,
            commands::tasks::list_categories,
            commands::tasks::create_category,
            commands::tasks::quick_add_task,
            commands::reminders::create_reminder,
            commands::reminders::list_task_reminders,
            commands::reminders::delete_reminder,
            commands::settings::get_timezone,
            commands::settings::set_timezone,
            commands::modes::get_mode_status,
            commands::modes::set_rest,
            commands::modes::start_focus,
            commands::modes::stop_focus,
            commands::modes::get_work_hours,
            commands::modes::set_work_hours,
            commands::modes::get_prayer_settings,
            commands::modes::set_prayer_settings,
            commands::modes::get_prayer_day,
            commands::profile::get_profile,
            commands::profile::save_profile,
            commands::profile::is_onboarded,
            commands::profile::complete_onboarding,
            commands::goals::list_goals,
            commands::goals::create_goal,
            commands::goals::update_goal,
            commands::goals::set_goal_status,
            commands::goals::delete_goal,
            commands::goals::list_milestones,
            commands::goals::add_milestones,
            commands::goals::set_milestone_status,
            commands::goals::delete_milestone,
            commands::goals::propose_plan,
            commands::goals::list_skills,
            commands::goals::create_skill,
            commands::goals::delete_skill,
            commands::coach::get_nudge_budget,
            commands::coach::set_nudge_budget,
            commands::coach::list_recent_nudges,
            commands::health::get_health_overview,
            commands::health::set_health_enabled,
            commands::health::log_habit,
            commands::health::set_water_target,
            commands::health::add_medicine,
            commands::health::delete_medicine,
            commands::rewards::get_progress,
            commands::rewards::get_reward_sound,
            commands::rewards::set_reward_sound,
            commands::ai::get_ai_status,
            commands::ai::get_ai_preset_defaults,
            commands::ai::save_ai_settings,
            commands::ai::set_ai_key,
            commands::ai::delete_ai_key,
            commands::ai::test_ai_connection,
            commands::ai::ask_itqan,
            commands::system::get_autostart,
            commands::system::set_autostart,
            commands::system::get_coach_settings,
            commands::system::set_nudges_paused,
        ])
        .events(collect_events![
            overlay::OverlayCursor,
            overlay::OverlayChanged,
            commands::events::TasksChanged,
            scheduler::ModeChanged,
            commands::goals::GoalsChanged,
            commands::events::HealthChanged,
            agents::rewards::RewardEarned,
            commands::events::Navigate,
        ])
}

#[cfg(any(debug_assertions, test))]
fn export_bindings(builder: &Builder<tauri::Wry>) -> Result<(), String> {
    builder
        .export(specta_typescript::Typescript::default(), BINDINGS_PATH)
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[allow(clippy::expect_used)]
pub fn run() {
    let builder = specta_builder();

    #[cfg(debug_assertions)]
    export_bindings(&builder).expect("failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(platform::plugin())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(shortcuts::plugin())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main_window(app);
        }))
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
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
            overlay::restore_follow_mode(app.handle())?;
            agents::setup(app);
            scheduler::start(app.handle());
            shortcuts::register(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    #[test]
    fn typescript_bindings_export() {
        super::export_bindings(&super::specta_builder()).unwrap();
    }
}
