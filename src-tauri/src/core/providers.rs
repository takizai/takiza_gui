use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Wire identifiers shared with takiza-harness, without built-in model IDs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiType {
    #[default]
    ChatCompletions,
    OpenaiResponses,
    AnthropicMessages,
}
impl ApiType {
    pub fn as_str(self) -> &'static str { match self {
        Self::ChatCompletions => "chat_completions", Self::OpenaiResponses => "openai_responses", Self::AnthropicMessages => "anthropic_messages",
    }}

    pub fn path(self) -> &'static str {
        match self {
            Self::ChatCompletions => "chat/completions",
            Self::OpenaiResponses => "responses",
            Self::AnthropicMessages => "messages",
        }
    }
}

pub fn validate_headers(headers: &BTreeMap<String, String>) -> Result<(), String> {
    let mut seen = std::collections::HashSet::new();
    for (name, value) in headers {
        if !seen.insert(name.to_ascii_lowercase()) {
            return Err("HTTP-заголовки не должны повторяться.".into());
        }
        reqwest::header::HeaderName::from_bytes(name.as_bytes())
            .map_err(|_| "Некорректное имя HTTP-заголовка.")?;
        reqwest::header::HeaderValue::from_str(value)
            .map_err(|_| "Некорректное значение HTTP-заголовка.")?;
    }
    Ok(())
}

pub fn authorize(
    mut request: reqwest::RequestBuilder,
    api: ApiType,
    key: &str,
    extra: &BTreeMap<String, String>,
) -> reqwest::RequestBuilder {
    if api == ApiType::AnthropicMessages {
        request = request.header("anthropic-version", "2023-06-01");
        if !key.is_empty() {
            request = request.header("x-api-key", key);
        }
    } else if !key.is_empty() {
        request = request.bearer_auth(key);
    }
    let mut headers = reqwest::header::HeaderMap::new();
    for (name, value) in extra {
        match (
            reqwest::header::HeaderName::from_bytes(name.as_bytes()),
            reqwest::header::HeaderValue::from_str(value),
        ) {
            (Ok(name), Ok(value)) => {
                headers.insert(name, value);
            }
            _ => return request.header(name, value),
        }
    }
    request.headers(headers)
}
