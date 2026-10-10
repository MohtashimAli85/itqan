use chrono::{DateTime, Utc};
use itqan_contracts::AppEvent;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};

use crate::error::AppError;
use crate::module::Modules;
use crate::overlay::BubbleAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Priority {
    LearningTip,
    Rhythm,
    Health,
    Drift,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum AgentKind {
    Coach,
    Planner,
    Health,
    FocusGuardian,
    Learning,
    Reviewer,
    Memory,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Signal {
    pub agent: AgentKind,
    pub kind: String,
    pub priority: Priority,
    pub text: String,
    pub actions: Vec<BubbleAction>,
    pub not_before: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
}

pub trait Subscriber: Send + Sync {
    fn on_event(
        &self,
        app: &AppHandle,
        event: &AppEvent,
        now: DateTime<Utc>,
    ) -> Result<Vec<Signal>, AppError>;
}

impl<T: Subscriber + ?Sized> Subscriber for Box<T> {
    fn on_event(
        &self,
        app: &AppHandle,
        event: &AppEvent,
        now: DateTime<Utc>,
    ) -> Result<Vec<Signal>, AppError> {
        (**self).on_event(app, event, now)
    }
}

pub trait SignalSink: Send + Sync {
    fn consider(
        &self,
        app: &AppHandle,
        signals: Vec<Signal>,
        now: DateTime<Utc>,
    ) -> Result<(), AppError>;
}

struct Registered {
    owner: &'static str,
    subscriber: Box<dyn Subscriber>,
}

pub struct Bus {
    sink: Box<dyn SignalSink>,
    subscribers: Vec<Registered>,
}

impl Bus {
    pub fn new(sink: impl SignalSink + 'static) -> Self {
        Self {
            sink: Box::new(sink),
            subscribers: Vec::new(),
        }
    }

    #[must_use]
    pub fn register(mut self, owner: &'static str, subscriber: impl Subscriber + 'static) -> Self {
        self.subscribers.push(Registered {
            owner,
            subscriber: Box::new(subscriber),
        });
        self
    }
}

pub fn publish(app: &AppHandle, event: &AppEvent) -> Result<(), AppError> {
    let now = Utc::now();
    let bus = app.state::<Bus>();
    let modules = app.try_state::<Modules>();
    let mut signals = Vec::new();
    for entry in &bus.subscribers {
        if let Some(modules) = &modules {
            if !modules.is_enabled(entry.owner)? {
                continue;
            }
        }
        match entry.subscriber.on_event(app, event, now) {
            Ok(mut found) => signals.append(&mut found),
            Err(error) => tracing::warn!(%error, owner = entry.owner, "agent failed"),
        }
    }
    bus.sink.consider(app, signals, now)
}
