use std::time::Duration;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AiError {
    #[error("the AI provider rejected the key")]
    Unauthorized,
    #[error("the AI provider is rate limiting requests")]
    RateLimited,
    #[error("the AI provider returned an error ({0})")]
    Server(u16),
    #[error("could not reach the AI provider: {0}")]
    Network(String),
    #[error("unexpected reply from the AI provider: {0}")]
    BadResponse(String),
    #[error("AI is not set up")]
    NotConfigured,
}

impl AiError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimited | Self::Server(_) | Self::Network(_))
    }
}

pub struct Client {
    http: reqwest::Client,
    base_url: String,
    key: Option<String>,
}

pub struct ChatRequest<'a> {
    pub model: &'a str,
    pub system: &'a str,
    pub user: &'a str,
    pub json: bool,
    pub reasoning_effort: Option<&'a str>,
    pub max_tokens: u32,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    kind: &'static str,
}

#[derive(Serialize)]
struct ChatBody<'a> {
    model: &'a str,
    messages: [Message<'a>; 2],
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<&'a str>,
}

#[derive(Deserialize)]
struct ChatReply {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ReplyMessage,
}

#[derive(Deserialize)]
struct ReplyMessage {
    content: Option<String>,
}

fn status_error(status: StatusCode) -> AiError {
    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => AiError::Unauthorized,
        StatusCode::TOO_MANY_REQUESTS => AiError::RateLimited,
        other => AiError::Server(other.as_u16()),
    }
}

fn network(error: &reqwest::Error) -> AiError {
    if error.is_timeout() {
        AiError::Network("timed out".into())
    } else {
        AiError::Network("connection failed".into())
    }
}

impl Client {
    pub fn new(base_url: &str, key: Option<String>, timeout: Duration) -> Result<Self, AiError> {
        let http = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|error| AiError::Network(error.to_string()))?;
        Ok(Self {
            http,
            base_url: base_url.trim_end_matches('/').to_owned(),
            key,
        })
    }

    fn request(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.key {
            Some(key) => builder.bearer_auth(key),
            None => builder,
        }
    }

    pub async fn chat(&self, request: ChatRequest<'_>) -> Result<String, AiError> {
        let body = ChatBody {
            model: request.model,
            messages: [
                Message {
                    role: "system",
                    content: request.system,
                },
                Message {
                    role: "user",
                    content: request.user,
                },
            ],
            max_tokens: request.max_tokens,
            response_format: request.json.then_some(ResponseFormat {
                kind: "json_object",
            }),
            reasoning_effort: request.reasoning_effort,
        };
        let response = self
            .request(
                self.http
                    .post(format!("{}/chat/completions", self.base_url)),
            )
            .json(&body)
            .send()
            .await
            .map_err(|error| network(&error))?;
        if !response.status().is_success() {
            return Err(status_error(response.status()));
        }
        let reply: ChatReply = response
            .json()
            .await
            .map_err(|_| AiError::BadResponse("not a chat completion".into()))?;
        reply
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .filter(|content| !content.trim().is_empty())
            .ok_or_else(|| AiError::BadResponse("empty reply".into()))
    }
}
