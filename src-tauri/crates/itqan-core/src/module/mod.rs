pub mod commands;

use std::collections::HashMap;
use std::sync::RwLock;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};

use crate::bus::Subscriber;
use crate::db::Database;
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

    fn teardown(&self, _app: &AppHandle) -> Result<(), AppError> {
        Ok(())
    }

    fn subscribers(&self) -> Vec<Box<dyn Subscriber>> {
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ModuleState {
    pub id: String,
    pub enabled: bool,
}

pub struct Modules {
    all: &'static [&'static dyn Module],
    enabled: RwLock<HashMap<&'static str, bool>>,
}

impl Modules {
    pub fn load(
        connection: &Connection,
        all: &'static [&'static dyn Module],
    ) -> Result<Self, AppError> {
        let mut statement = connection.prepare("SELECT id, enabled FROM modules")?;
        let stored: HashMap<String, bool> = statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;
        let enabled = all
            .iter()
            .map(|module| {
                (
                    module.id(),
                    stored.get(module.id()).copied().unwrap_or(true),
                )
            })
            .collect();
        Ok(Self {
            all,
            enabled: RwLock::new(enabled),
        })
    }

    pub fn is_enabled(&self, id: &str) -> Result<bool, AppError> {
        Ok(self
            .enabled
            .read()
            .map_err(|_| AppError::LockPoisoned)?
            .get(id)
            .copied()
            .unwrap_or(true))
    }

    pub fn states(&self) -> Result<Vec<ModuleState>, AppError> {
        let enabled = self.enabled.read().map_err(|_| AppError::LockPoisoned)?;
        Ok(self
            .all
            .iter()
            .map(|module| ModuleState {
                id: module.id().to_owned(),
                enabled: enabled.get(module.id()).copied().unwrap_or(true),
            })
            .collect())
    }

    fn find(&self, id: &str) -> Result<&'static dyn Module, AppError> {
        self.all
            .iter()
            .copied()
            .find(|module| module.id() == id)
            .ok_or_else(|| AppError::NotFound(format!("module {id}")))
    }

    pub fn set(
        &self,
        connection: &Connection,
        id: &str,
        enabled: bool,
        now: DateTime<Utc>,
    ) -> Result<Option<&'static dyn Module>, AppError> {
        let module = self.find(id)?;
        if self.is_enabled(id)? == enabled {
            return Ok(None);
        }
        connection.execute(
            "INSERT INTO modules (id, enabled, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT (id) DO UPDATE SET enabled = excluded.enabled, updated_at = excluded.updated_at",
            params![module.id(), enabled, now],
        )?;
        self.enabled
            .write()
            .map_err(|_| AppError::LockPoisoned)?
            .insert(module.id(), enabled);
        Ok(Some(module))
    }
}

pub fn start(app: &AppHandle, all: &'static [&'static dyn Module]) -> Result<(), AppError> {
    let modules = app
        .state::<Database>()
        .with(|connection| Modules::load(connection, all))?;
    for module in all {
        if modules.is_enabled(module.id())? {
            module.setup(app)?;
        }
    }
    app.manage(modules);
    Ok(())
}

pub fn set_enabled(app: &AppHandle, id: &str, enabled: bool) -> Result<(), AppError> {
    let modules = app.state::<Modules>();
    let changed = app
        .state::<Database>()
        .with(|connection| modules.set(connection, id, enabled, Utc::now()))?;
    let Some(module) = changed else {
        return Ok(());
    };
    if enabled {
        module.setup(app)
    } else {
        module.teardown(app)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_connection;

    struct Named(&'static str);

    impl Module for Named {
        fn id(&self) -> &'static str {
            self.0
        }
    }

    static ALL: &[&dyn Module] = &[&Named("tasks"), &Named("salah")];

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-10-10T09:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn modules_start_enabled_and_core_owners_are_always_on() {
        let connection = test_connection();
        let modules = Modules::load(&connection, ALL).unwrap();

        assert_eq!(
            modules.states().unwrap(),
            vec![
                ModuleState {
                    id: "tasks".into(),
                    enabled: true
                },
                ModuleState {
                    id: "salah".into(),
                    enabled: true
                },
            ]
        );
        assert!(modules.is_enabled("rhythm").unwrap());
    }

    #[test]
    fn disabling_is_stored_and_survives_a_restart() {
        let connection = test_connection();
        let modules = Modules::load(&connection, ALL).unwrap();

        let changed = modules.set(&connection, "salah", false, now()).unwrap();
        assert_eq!(changed.map(Module::id), Some("salah"));
        assert!(modules
            .set(&connection, "salah", false, now())
            .unwrap()
            .is_none());
        assert!(!modules.is_enabled("salah").unwrap());

        let reloaded = Modules::load(&connection, ALL).unwrap();
        assert!(!reloaded.is_enabled("salah").unwrap());
        assert!(reloaded.is_enabled("tasks").unwrap());

        reloaded.set(&connection, "salah", true, now()).unwrap();
        assert!(Modules::load(&connection, ALL)
            .unwrap()
            .is_enabled("salah")
            .unwrap());
    }

    #[test]
    fn unknown_modules_are_rejected() {
        let connection = test_connection();
        let modules = Modules::load(&connection, ALL).unwrap();

        assert!(matches!(
            modules.set(&connection, "memory", false, now()),
            Err(AppError::NotFound(_))
        ));
    }
}
