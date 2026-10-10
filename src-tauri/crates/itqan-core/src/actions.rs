use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use tauri::AppHandle;

use crate::error::AppError;

pub const LATER: &str = "later";

pub trait ActionHandler: Send + Sync {
    fn handle(&self, app: &AppHandle, action: &str) -> Result<(), AppError>;
}

#[derive(Default)]
pub struct ActionRouter {
    handlers: RwLock<HashMap<&'static str, Arc<dyn ActionHandler>>>,
}

impl ActionRouter {
    pub fn register(
        &self,
        namespace: &'static str,
        handler: impl ActionHandler + 'static,
    ) -> Result<(), AppError> {
        let mut handlers = self.handlers.write().map_err(|_| AppError::LockPoisoned)?;
        if handlers.contains_key(namespace) {
            return Err(AppError::InvalidInput(format!(
                "actions under {namespace}: already have a handler"
            )));
        }
        handlers.insert(namespace, Arc::new(handler));
        Ok(())
    }

    pub fn unregister(&self, namespace: &str) -> Result<(), AppError> {
        self.handlers
            .write()
            .map_err(|_| AppError::LockPoisoned)?
            .remove(namespace);
        Ok(())
    }

    pub fn route(&self, app: &AppHandle, action: &str) -> Result<(), AppError> {
        match self.handler_for(action)? {
            Some(handler) => handler.handle(app, action),
            None => Ok(()),
        }
    }

    fn handler_for(&self, action: &str) -> Result<Option<Arc<dyn ActionHandler>>, AppError> {
        let Some((namespace, _)) = action.split_once(':') else {
            return Ok(None);
        };
        Ok(self
            .handlers
            .read()
            .map_err(|_| AppError::LockPoisoned)?
            .get(namespace)
            .cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Noop;

    impl ActionHandler for Noop {
        fn handle(&self, _: &AppHandle, _: &str) -> Result<(), AppError> {
            Ok(())
        }
    }

    #[test]
    fn actions_route_by_namespace_only() {
        let router = ActionRouter::default();
        router.register("habit", Noop).unwrap();

        assert!(router.handler_for("habit:water").unwrap().is_some());
        assert!(router.handler_for("habit").unwrap().is_none());
        assert!(router.handler_for("done").unwrap().is_none());
        assert!(router.handler_for("focus:start").unwrap().is_none());
        assert!(router.register("habit", Noop).is_err());

        router.unregister("habit").unwrap();
        assert!(router.handler_for("habit:water").unwrap().is_none());
    }
}
