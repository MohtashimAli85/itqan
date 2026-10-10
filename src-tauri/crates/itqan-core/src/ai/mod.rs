pub mod ask;
pub mod client;
pub mod commands;
pub mod config;
pub mod keys;
pub mod redact;

use std::time::Duration;

use tauri::{AppHandle, Manager};

use crate::db::Database;
use crate::error::AppError;
use client::{AiError, ChatRequest, Client};
use config::{AiSettings, Preset};

const BACKOFF: [Duration; 3] = [
    Duration::from_millis(500),
    Duration::from_millis(1500),
    Duration::from_millis(4000),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Job {
    ConnectionTest,
    Planning,
    Question,
    Profiling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Fast,
    Smart,
}

pub fn role(job: Job) -> Role {
    match job {
        Job::ConnectionTest => Role::Fast,
        Job::Planning | Job::Question | Job::Profiling => Role::Smart,
    }
}

fn effort(role: Role) -> &'static str {
    match role {
        Role::Fast => "low",
        Role::Smart => "medium",
    }
}

fn timeout(role: Role) -> Duration {
    match role {
        Role::Fast => Duration::from_secs(20),
        Role::Smart => Duration::from_secs(90),
    }
}

struct Target {
    preset: Preset,
    base_url: String,
    model: String,
    key: Option<String>,
}

fn targets(settings: &AiSettings, role: Role, fallback: bool) -> Result<Vec<Target>, AppError> {
    if !settings.enabled {
        return Err(AppError::Ai(AiError::NotConfigured));
    }
    let model = match role {
        Role::Fast => settings.fast_model.clone(),
        Role::Smart => settings.smart_model.clone(),
    };
    let key = keys::get(settings.preset)?;
    if settings.preset.defaults().needs_key && key.is_none() {
        return Err(AppError::Ai(AiError::NotConfigured));
    }
    let mut found = vec![Target {
        preset: settings.preset,
        base_url: settings.base_url.clone(),
        model,
        key,
    }];
    if fallback && settings.local_fallback && !settings.preset.defaults().local {
        let local = Preset::Ollama.defaults();
        found.push(Target {
            preset: Preset::Ollama,
            base_url: local.base_url.into(),
            model: local.fast_model.into(),
            key: None,
        });
    }
    Ok(found)
}

pub fn settings(app: &AppHandle) -> Result<AiSettings, AppError> {
    app.state::<Database>().with(config::load)
}

pub async fn complete(
    app: &AppHandle,
    job: Job,
    system: &str,
    user: &str,
) -> Result<String, AppError> {
    let role = role(job);
    let targets = targets(&settings(app)?, role, job != Job::ConnectionTest)?;
    let mut last = AiError::NotConfigured;
    for target in targets {
        let client =
            Client::new(&target.base_url, target.key, timeout(role)).map_err(AppError::Ai)?;
        let reasoning = target
            .preset
            .defaults()
            .reasoning_effort
            .then(|| effort(role));
        for (attempt, pause) in std::iter::once(None)
            .chain(BACKOFF.iter().map(Some))
            .enumerate()
        {
            if let Some(pause) = pause {
                tokio::time::sleep(*pause).await;
            }
            let result = client
                .chat(ChatRequest {
                    model: &target.model,
                    system,
                    user,
                    json: true,
                    reasoning_effort: reasoning,
                    max_tokens: 2048,
                })
                .await;
            match result {
                Ok(text) => return Ok(text),
                Err(error) => {
                    tracing::warn!(attempt, %error, "ai request failed");
                    let retry = error.is_retryable();
                    last = error;
                    if !retry {
                        break;
                    }
                }
            }
        }
    }
    Err(AppError::Ai(last))
}

pub async fn test_connection(app: &AppHandle) -> Result<(), AppError> {
    complete(
        app,
        Job::ConnectionTest,
        "Reply with the JSON object {\"ok\":true} and nothing else.",
        "Connection test",
    )
    .await
    .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jobs_route_to_the_right_model() {
        assert_eq!(role(Job::ConnectionTest), Role::Fast);
        assert_eq!(role(Job::Planning), Role::Smart);
        assert_eq!(role(Job::Question), Role::Smart);
        assert_eq!(effort(Role::Fast), "low");
    }

    #[test]
    fn disabled_ai_has_no_targets() {
        assert!(targets(&AiSettings::default(), Role::Fast, true).is_err());
    }

    #[test]
    fn retries_only_transient_failures() {
        assert!(AiError::RateLimited.is_retryable());
        assert!(AiError::Server(503).is_retryable());
        assert!(!AiError::Unauthorized.is_retryable());
        assert!(!AiError::BadResponse("x".into()).is_retryable());
    }
}
