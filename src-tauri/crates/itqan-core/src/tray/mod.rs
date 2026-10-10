mod view;

use std::sync::Mutex;

use chrono::{Duration, NaiveTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::menu::{CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{App, AppHandle, Manager, Wry};
use tauri_specta::Event;

use crate::coach;
use crate::db::Database;
use crate::error::AppError;
use crate::overlay::{self, FollowMode, OverlayStore};
use crate::ports::Ports;
use crate::scheduler;
use crate::settings;
use crate::{focus, modes};
use view::{TrayInputs, TrayView};

#[derive(Debug, Clone, Serialize, Deserialize, Type, tauri_specta::Event)]
pub struct Navigate {
    pub to: String,
}

const TRAY_ID: &str = "itqan";
const QUIT: &str = "quit";
const OPEN: &str = "open";
const SETTINGS: &str = "settings";
const QUICK_ADD: &str = "quick-add";
const FOCUS: &str = "focus";
const PAUSE: &str = "pause";
const REST: &str = "rest";
const ORB_FOLLOW: &str = "orb-follow";
const ORB_CORNER: &str = "orb-corner";
const ORB_HIDDEN: &str = "orb-hidden";
const NEXT_TASK: &str = "next-task";
const PAUSE_MINUTES: i64 = 60;
const MORNING: u32 = 6;

#[derive(Default)]
struct TrayCache(Mutex<Option<TrayView>>);

pub fn setup(app: &App) -> tauri::Result<()> {
    app.manage(TrayCache::default());
    let menu = Menu::with_items(
        app,
        &[&MenuItem::with_id(
            app,
            QUIT,
            "Quit Itqan",
            true,
            None::<&str>,
        )?],
    )?;
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .on_menu_event(|app, event| {
            if let Err(error) = handle(app, event.id.as_ref()) {
                tracing::warn!(%error, "tray action failed");
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone()).icon_as_template(false);
    }
    builder.build(app)?;
    Ok(())
}

pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn load(app: &AppHandle) -> Result<TrayView, AppError> {
    let now = Utc::now();
    let status = scheduler::modes::current(app)?;
    let follow_mode = app.state::<OverlayStore>().snapshot()?.follow_mode;
    app.state::<Database>().with(|connection| {
        let timezone = settings::timezone(connection)?;
        let today = now.with_timezone(&timezone).date_naive();
        let (_, end) = crate::settings::day_bounds(timezone, today);
        let ports = app.state::<Ports>();
        let tasks_left = ports.today_task_counts(connection, now, end)?.open;
        let next_task = ports.next_task_for_today(connection, end)?;
        let tasks = ports.has_task_stats()?;
        Ok(view::view(&TrayInputs {
            tasks,
            focus_minutes_left: status
                .as_ref()
                .and_then(|status| status.focus.as_ref())
                .map(|focus| (focus.ends_at - now).num_minutes() + 1),
            tasks_left,
            next_task,
            paused: coach::paused_until(connection)?.is_some_and(|until| until > now),
            resting: status
                .as_ref()
                .is_some_and(|status| status.rest_until.is_some()),
            follow_mode,
        }))
    })
}

fn menu(app: &AppHandle, view: &TrayView) -> tauri::Result<Menu<Wry>> {
    let next = MenuItem::with_id(
        app,
        NEXT_TASK,
        view.next_task.as_deref().map_or_else(
            || "Nothing due today".to_owned(),
            |task| format!("Next: {task}"),
        ),
        false,
        None::<&str>,
    )?;
    let focus = MenuItem::with_id(
        app,
        FOCUS,
        if view.focus_active {
            "Stop focus"
        } else {
            "Start a 25 minute focus"
        },
        true,
        None::<&str>,
    )?;
    let quick_add = MenuItem::with_id(app, QUICK_ADD, "Quick add…", true, Some("Alt+Space"))?;
    let pause = MenuItem::with_id(
        app,
        PAUSE,
        if view.paused {
            "Resume nudges"
        } else {
            "Pause nudges for an hour"
        },
        true,
        None::<&str>,
    )?;
    let rest = CheckMenuItem::with_id(app, REST, "Rest mode", true, view.resting, None::<&str>)?;
    let orb = Submenu::with_items(
        app,
        "Orb",
        true,
        &[
            &CheckMenuItem::with_id(
                app,
                ORB_FOLLOW,
                "Follow cursor",
                true,
                view.follow_mode == FollowMode::Follow,
                None::<&str>,
            )?,
            &CheckMenuItem::with_id(
                app,
                ORB_CORNER,
                "Sit in corner",
                true,
                view.follow_mode == FollowMode::Corner,
                None::<&str>,
            )?,
            &CheckMenuItem::with_id(
                app,
                ORB_HIDDEN,
                "Hidden",
                true,
                view.follow_mode == FollowMode::Hidden,
                None::<&str>,
            )?,
        ],
    )?;
    let open = MenuItem::with_id(app, OPEN, "Open Itqan", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, SETTINGS, "Settings…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT, "Quit Itqan", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let mut items: Vec<&dyn IsMenuItem<Wry>> = Vec::new();
    if view.tasks {
        items.extend([&next as &dyn IsMenuItem<Wry>, &separator]);
    }
    items.push(&focus);
    if view.tasks {
        items.push(&quick_add);
    }
    let lower = PredefinedMenuItem::separator(app)?;
    let last = PredefinedMenuItem::separator(app)?;
    items.extend([
        &pause as &dyn IsMenuItem<Wry>,
        &rest,
        &orb,
        &lower,
        &open,
        &settings,
        &last,
        &quit,
    ]);
    Menu::with_items(app, &items)
}

pub fn refresh(app: &AppHandle) -> Result<(), AppError> {
    let view = load(app)?;
    let cache = app.state::<TrayCache>();
    let mut cached = cache.0.lock().map_err(|_| AppError::LockPoisoned)?;
    if cached.as_ref() == Some(&view) {
        return Ok(());
    }
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(menu(app, &view)?))?;
        tray.set_title(view.title.as_deref())?;
    }
    *cached = Some(view);
    Ok(())
}

fn handle(app: &AppHandle, id: &str) -> Result<(), AppError> {
    let database = app.state::<Database>();
    let now = Utc::now();
    match id {
        QUIT => app.exit(0),
        OPEN => show_main_window(app),
        SETTINGS => {
            show_main_window(app);
            Navigate {
                to: "/settings".into(),
            }
            .emit(app)?;
        }
        QUICK_ADD => overlay::open_panel(app)?,
        FOCUS => {
            database.with(|connection| match focus::active(connection)? {
                Some(session) => focus::finish(connection, session.id, now, false),
                None => focus::start(connection, focus::DEFAULT_MINUTES, None, now).map(|_| ()),
            })?;
            scheduler::refresh(app)?;
        }
        PAUSE => {
            database.with(|connection| {
                let paused = coach::paused_until(connection)?.is_some_and(|until| until > now);
                coach::set_paused_until(
                    connection,
                    (!paused).then(|| now + Duration::minutes(PAUSE_MINUTES)),
                )
            })?;
        }
        REST => {
            database.with(|connection| {
                let resting = modes::rest_until(connection)?.is_some_and(|until| until > now);
                let until = if resting {
                    None
                } else {
                    let timezone = settings::timezone(connection)?;
                    let tomorrow = now.with_timezone(&timezone).date_naive() + Duration::days(1);
                    timezone
                        .from_local_datetime(
                            &tomorrow.and_time(
                                NaiveTime::from_hms_opt(MORNING, 0, 0).unwrap_or_default(),
                            ),
                        )
                        .earliest()
                        .map(|morning| morning.with_timezone(&Utc))
                };
                modes::set_rest_until(connection, until)
            })?;
            scheduler::refresh(app)?;
        }
        ORB_FOLLOW => overlay::save_follow_mode(app, FollowMode::Follow)?,
        ORB_CORNER => overlay::save_follow_mode(app, FollowMode::Corner)?,
        ORB_HIDDEN => overlay::save_follow_mode(app, FollowMode::Hidden)?,
        _ => {}
    }
    refresh(app)
}
