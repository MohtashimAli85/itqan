use tauri::Manager;
use tauri_specta::{collect_commands, collect_events, Builder};

use itqan_core::actions::ActionRouter;
use itqan_core::db::Database;
use itqan_core::module::Module;
use itqan_core::ports::Ports;
use itqan_core::{overlay, platform, scheduler, shortcuts, tray};

static MODULES: &[&dyn Module] = &[
    &itqan_salah::SalahModule,
    &itqan_health::HealthModule,
    &itqan_tasks::TasksModule,
    &itqan_progress::ProgressModule,
];

#[cfg(any(debug_assertions, test))]
const BINDINGS_PATH: &str = "../src/shared/bindings/bindings.ts";

fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            itqan_core::app_info::commands::get_app_info,
            itqan_core::overlay::commands::set_overlay_hit_areas,
            itqan_core::overlay::commands::get_overlay_state,
            itqan_core::overlay::commands::set_orb_state,
            itqan_core::overlay::commands::set_follow_mode,
            itqan_core::overlay::commands::set_orb_progress,
            itqan_core::overlay::commands::show_bubble,
            itqan_core::overlay::commands::resolve_bubble,
            itqan_core::overlay::commands::set_panel_open,
            itqan_tasks::commands::list_tasks,
            itqan_tasks::commands::create_task,
            itqan_tasks::commands::update_task,
            itqan_tasks::commands::set_task_status,
            itqan_tasks::commands::set_task_top_three,
            itqan_tasks::commands::delete_task,
            itqan_tasks::commands::list_categories,
            itqan_tasks::commands::create_category,
            itqan_tasks::commands::quick_add_task,
            itqan_core::reminders::commands::create_reminder,
            itqan_core::reminders::commands::list_task_reminders,
            itqan_core::reminders::commands::delete_reminder,
            itqan_core::settings::commands::get_timezone,
            itqan_core::settings::commands::set_timezone,
            itqan_core::modes::commands::get_mode_status,
            itqan_core::modes::commands::set_rest,
            itqan_core::modes::commands::start_focus,
            itqan_core::modes::commands::stop_focus,
            itqan_core::modes::commands::get_work_hours,
            itqan_core::modes::commands::set_work_hours,
            itqan_salah::commands::get_prayer_settings,
            itqan_salah::commands::set_prayer_settings,
            itqan_salah::commands::get_prayer_day,
            itqan_core::profile::commands::get_profile,
            itqan_core::profile::commands::save_profile,
            itqan_core::profile::commands::is_onboarded,
            itqan_core::profile::commands::complete_onboarding,
            itqan_core::goals::commands::list_goals,
            itqan_core::goals::commands::create_goal,
            itqan_core::goals::commands::update_goal,
            itqan_core::goals::commands::set_goal_status,
            itqan_core::goals::commands::delete_goal,
            itqan_core::goals::commands::list_milestones,
            itqan_core::goals::commands::add_milestones,
            itqan_core::goals::commands::set_milestone_status,
            itqan_core::goals::commands::delete_milestone,
            itqan_core::goals::commands::propose_plan,
            itqan_core::goals::commands::list_skills,
            itqan_core::goals::commands::create_skill,
            itqan_core::goals::commands::delete_skill,
            itqan_core::coach::commands::get_nudge_budget,
            itqan_core::coach::commands::set_nudge_budget,
            itqan_core::coach::commands::list_recent_nudges,
            itqan_health::commands::get_health_overview,
            itqan_health::commands::set_health_enabled,
            itqan_health::commands::log_habit,
            itqan_health::commands::set_water_target,
            itqan_health::commands::add_medicine,
            itqan_health::commands::delete_medicine,
            itqan_progress::commands::get_progress,
            itqan_progress::commands::get_reward_sound,
            itqan_progress::commands::set_reward_sound,
            itqan_core::ai::commands::get_ai_status,
            itqan_core::ai::commands::get_ai_preset_defaults,
            itqan_core::ai::commands::save_ai_settings,
            itqan_core::ai::commands::set_ai_key,
            itqan_core::ai::commands::delete_ai_key,
            itqan_core::ai::commands::test_ai_connection,
            itqan_core::ai::commands::ask_itqan,
            itqan_core::system::get_autostart,
            itqan_core::system::set_autostart,
            itqan_core::system::get_coach_settings,
            itqan_core::system::set_nudges_paused,
            itqan_core::module::commands::list_modules,
            itqan_core::module::commands::set_module_enabled,
            itqan_core::beliefs::commands::draft_beliefs,
            itqan_core::beliefs::commands::save_belief_notes,
            itqan_core::proposals::commands::list_proposals,
            itqan_core::proposals::commands::accept_proposal,
            itqan_core::proposals::commands::reject_proposal,
            itqan_core::system::open_main_window,
        ])
        .events(collect_events![
            overlay::OverlayCursor,
            overlay::OverlayChanged,
            itqan_tasks::TasksChanged,
            scheduler::ModeChanged,
            itqan_core::goals::commands::GoalsChanged,
            itqan_health::HealthChanged,
            itqan_progress::RewardEarned,
            tray::Navigate,
            itqan_core::module::commands::ModulesChanged,
            itqan_core::proposals::commands::ProposalsChanged,
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
            let database = Database::open(&data_dir.join("itqan.db"), MODULES)?;
            app.manage(database);
            tracing::info!("database ready");
            app.manage(Ports::default());
            app.manage(ActionRouter::default());
            app.manage(itqan_core::proposals::ProposalRegistry::default());
            itqan_core::module::start(app.handle(), MODULES)?;
            tray::setup(app)?;
            overlay::setup(app)?;
            overlay::restore_follow_mode(app.handle())?;
            itqan_core::coach::setup(app, MODULES);
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
