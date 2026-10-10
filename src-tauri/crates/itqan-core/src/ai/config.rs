use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::db::settings as settings_repo;
use crate::error::AppError;

const SETTINGS: &str = "ai_settings";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Preset {
    Groq,
    Ollama,
    LlamaCpp,
    OpenAi,
    OpenRouter,
    Custom,
}

pub struct PresetDefaults {
    pub base_url: &'static str,
    pub fast_model: &'static str,
    pub smart_model: &'static str,
    pub needs_key: bool,
    pub reasoning_effort: bool,
    pub local: bool,
}

impl Preset {
    pub fn defaults(self) -> PresetDefaults {
        let (base_url, fast_model, smart_model, needs_key, reasoning_effort, local) = match self {
            Self::Groq => (
                "https://api.groq.com/openai/v1",
                "openai/gpt-oss-20b",
                "openai/gpt-oss-120b",
                true,
                true,
                false,
            ),
            Self::Ollama => (
                "http://localhost:11434/v1",
                "gpt-oss:20b",
                "gpt-oss:20b",
                false,
                false,
                true,
            ),
            Self::LlamaCpp => (
                "http://localhost:8080/v1",
                "gpt-oss-20b",
                "gpt-oss-20b",
                false,
                false,
                true,
            ),
            Self::OpenAi => (
                "https://api.openai.com/v1",
                "gpt-4.1-mini",
                "gpt-4.1",
                true,
                false,
                false,
            ),
            Self::OpenRouter => (
                "https://openrouter.ai/api/v1",
                "openai/gpt-oss-20b",
                "openai/gpt-oss-120b",
                true,
                false,
                false,
            ),
            Self::Custom => ("", "", "", true, false, false),
        };
        PresetDefaults {
            base_url,
            fast_model,
            smart_model,
            needs_key,
            reasoning_effort,
            local,
        }
    }

    pub fn key_account(self) -> &'static str {
        match self {
            Self::Groq => "ai-groq",
            Self::Ollama => "ai-ollama",
            Self::LlamaCpp => "ai-llamacpp",
            Self::OpenAi => "ai-openai",
            Self::OpenRouter => "ai-openrouter",
            Self::Custom => "ai-custom",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AiSettings {
    pub enabled: bool,
    pub preset: Preset,
    pub base_url: String,
    pub fast_model: String,
    pub smart_model: String,
    pub local_fallback: bool,
}

impl Default for AiSettings {
    fn default() -> Self {
        let defaults = Preset::Groq.defaults();
        Self {
            enabled: false,
            preset: Preset::Groq,
            base_url: defaults.base_url.into(),
            fast_model: defaults.fast_model.into(),
            smart_model: defaults.smart_model.into(),
            local_fallback: false,
        }
    }
}

pub fn is_local_url(url: &str) -> bool {
    let rest = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .unwrap_or("");
    let host = rest.split(['/', ':']).next().unwrap_or("");
    matches!(host, "localhost" | "127.0.0.1" | "[::1]")
}

pub fn validate(settings: &AiSettings) -> Result<(), AppError> {
    let url = settings.base_url.trim();
    let secure = url.starts_with("https://");
    let local_plain = url.starts_with("http://") && is_local_url(url);
    if !secure && !local_plain {
        return Err(AppError::InvalidInput(
            "use an https address, or http only for localhost".into(),
        ));
    }
    if settings.fast_model.trim().is_empty() || settings.smart_model.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "choose a fast and a smart model".into(),
        ));
    }
    Ok(())
}

pub fn load(connection: &Connection) -> Result<AiSettings, AppError> {
    Ok(match settings_repo::get(connection, SETTINGS)? {
        Some(json) => serde_json::from_str(&json)?,
        None => AiSettings::default(),
    })
}

pub fn save(connection: &Connection, settings: &AiSettings) -> Result<(), AppError> {
    validate(settings)?;
    let cleaned = AiSettings {
        base_url: settings.base_url.trim().trim_end_matches('/').to_owned(),
        fast_model: settings.fast_model.trim().to_owned(),
        smart_model: settings.smart_model.trim().to_owned(),
        ..settings.clone()
    };
    settings_repo::set(connection, SETTINGS, &serde_json::to_string(&cleaned)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_connection;

    #[test]
    fn plain_http_is_only_allowed_for_local_servers() {
        let local = AiSettings {
            base_url: "http://localhost:11434/v1".into(),
            ..AiSettings::default()
        };
        assert!(validate(&local).is_ok());

        let remote = AiSettings {
            base_url: "http://api.example.com/v1".into(),
            ..AiSettings::default()
        };
        assert!(validate(&remote).is_err());
        assert!(!is_local_url("http://localhost.evil.com/v1"));
    }

    #[test]
    fn settings_round_trip_without_trailing_slashes() {
        let connection = test_connection();
        assert!(!load(&connection).unwrap().enabled);

        let settings = AiSettings {
            enabled: true,
            base_url: "https://api.groq.com/openai/v1/ ".into(),
            ..AiSettings::default()
        };
        save(&connection, &settings).unwrap();
        let loaded = load(&connection).unwrap();
        assert!(loaded.enabled);
        assert_eq!(loaded.base_url, "https://api.groq.com/openai/v1");
    }
}
