//! Native Responses and Messages transports. Kept behind the common agent contract.
use super::*;
use crate::core::providers::ApiType;
use std::{collections::BTreeMap, time::Duration};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderData {
    pub api_type: ApiType,
    pub endpoint: String,
    pub model: String,
    #[serde(default)]
    pub credential_scope: String,
    pub items: Vec<Value>,
}
impl LlmClient {
    fn credential_scope(&self) -> String {
        use sha2::{Digest, Sha256};
        let scope =
            serde_json::to_vec(&(&self.config.api_key, &self.config.provider_headers)).unwrap();
        format!("{:x}", Sha256::digest(scope))
    }
    pub fn take_provider_data(&self) -> Option<ProviderData> {
        self.provider_data.lock().unwrap().take()
    }
    pub(super) fn authorize(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        crate::core::providers::authorize(
            request,
            self.config.api_type,
            &self.config.api_key,
            &self.config.provider_headers,
        )
    }
    fn replay<'a>(&self, message: &'a ChatMessage) -> Option<&'a [Value]> {
        message
            .provider_data
            .as_ref()
            .filter(|data| {
                data.api_type == self.config.api_type
                    && data.endpoint == self.config.base_url
                    && data.model == self.config.model
                    && data.credential_scope == self.credential_scope()
            })
            .map(|data| data.items.as_slice())
    }
    fn native_body(
        &self,
        messages: &[ChatMessage],
        max_tokens: u64,
        tools: bool,
        stream: bool,
    ) -> Result<Value> {
        let mut input = Vec::new();
        let mut system = Vec::new();
        let responses = self.config.api_type == ApiType::OpenaiResponses;
        for message in messages {
            if let Some(items) = self.replay(message) {
                if responses {
                    input.extend_from_slice(items);
                } else {
                    input.push(json!({"role":"assistant", "content":items}));
                }
                continue;
            }
            if message.role == "system" && !responses {
                if let Some(content) = &message.content {
                    system.push(content.clone());
                }
                continue;
            }
            if message.role == "tool" {
                let id = message
                    .tool_call_id
                    .as_deref()
                    .context("Tool result has no call ID")?;
                if responses {
                    input.push(json!({"type":"function_call_output", "call_id":id,"output":message.content.as_deref().unwrap_or("")}));
                } else {
                    input.push(json!({"role":"user","content":[{"type":"tool_result","tool_use_id":id,"content":message.content.as_deref().unwrap_or("")}]}));
                }
                continue;
            }
            let mut content = Vec::new();
            if let Some(text) = message.content.as_deref().filter(|text| !text.is_empty()) {
                content.push(if responses { json!({"type":if message.role=="assistant" {"output_text"} else {"input_text"},"text":text}) }
                    else { json!({"type":"text","text":text}) });
            }
            for image in &message.image_urls {
                anyhow::ensure!(
                    message.role == "user",
                    "Images must belong to a user message"
                );
                if responses {
                    content.push(json!({"type":"input_image","image_url":image}));
                } else if let Some(data) = image.strip_prefix("data:") {
                    let (media_type, data) = data
                        .split_once(";base64,")
                        .context("Invalid image data URL")?;
                    content.push(json!({"type":"image","source":{"type":"base64","media_type":media_type,"data":data}}));
                } else {
                    content.push(json!({"type":"image","source":{"type":"url","url":image}}));
                }
            }
            if !responses {
                for call in message.tool_calls.iter().flatten() {
                    content.push(
                        json!({"type":"tool_use","id":call.id,"name":call.function.name,
                        "input":serde_json::from_str::<Value>(&call.function.arguments).context("Invalid historical tool arguments")?}),
                    );
                }
            }
            if !content.is_empty() {
                // Synthetic assistant history uses the easy-message string
                // form; output_text blocks belong to full response items.
                if responses && message.role == "assistant" {
                    input.push(json!({"role":"assistant","content":message.content.as_deref().unwrap_or("")}));
                } else {
                    input.push(json!({"role":message.role,"content":content}));
                }
            }
            if responses {
                for call in message.tool_calls.iter().flatten() {
                    let arguments = if call.id.starts_with("rejected_") {
                        serde_json::from_str::<Value>(&call.function.arguments)
                            .context("Invalid historical tool arguments")?
                            .to_string()
                    } else {
                        call.function.arguments.clone()
                    };
                    input.push(json!({"type":"function_call","call_id":call.id,"name":call.function.name,"arguments":arguments}));
                }
            }
        }
        // Messages API expects consecutive tool results in one user turn.
        if !responses {
            let mut merged: Vec<Value> = Vec::new();
            for mut message in input {
                if let Some(previous) = merged.last_mut().filter(|p| p["role"] == message["role"]) {
                    previous["content"]
                        .as_array_mut()
                        .unwrap()
                        .append(message["content"].as_array_mut().unwrap());
                } else {
                    merged.push(message);
                }
            }
            input = merged;
        }
        let mut body = if responses {
            json!({"model":self.config.model,"input":input,"max_output_tokens":max_tokens,
            "stream":stream,"store":false,"include":["reasoning.encrypted_content"]})
        } else {
            json!({"model":self.config.model,"messages":input,"system":system.join("\n\n"),"max_tokens":max_tokens,"stream":stream})
        };
        crate::core::reasoning::apply(&mut body, self.config.api_type.as_str(), &self.config.reasoning_levels(), self.config.effort.as_deref());
        if tools {
            body["tools"] = Value::Array(tools_for_api(&get_tool_definitions()).as_array().context("Invalid tools")?.iter().map(|tool| {
                let function = &tool["function"];
                if responses { json!({"type":"function","name":function["name"],"description":function["description"],"parameters":function["parameters"],"strict":false}) }
                else { json!({"name":function["name"],"description":function["description"],"input_schema":function["parameters"]}) }
            }).collect());
        }
        Ok(body)
    }
    pub(super) async fn native_title(&self, messages: &[ChatMessage]) -> Option<String> {
        self.native_request(
            messages,
            4096,
            false,
            false,
            None,
            &CancellationToken::new(),
            Duration::from_secs(60),
        )
        .await
        .ok()
        .map(|output| output.text)
    }
    pub(super) async fn native_chat(
        &self,
        messages: &[ChatMessage],
        stream: bool,
        tx: Option<&mpsc::Sender<AgentEvent>>,
        cancel: &CancellationToken,
    ) -> Result<LlmResponse> {
        *self.provider_data.lock().unwrap() = None;
        let output = self
            .native_request(
                messages,
                8192,
                true,
                stream,
                tx,
                cancel,
                Duration::from_secs(120),
            )
            .await?;
        anyhow::ensure!(output.finish.as_deref() != Some("length"), "Provider truncated the response. No tools were executed; increase the output allowance or select another model.");
        let calls = output
            .calls
            .into_iter()
            .map(|call| checked_tool_call(call.id, call.name, call.arguments))
            .collect::<Result<Vec<_>>>()?;
        *self.provider_data.lock().unwrap() = Some(ProviderData {
            api_type: self.config.api_type,
            endpoint: self.config.base_url.clone(),
            model: self.config.model.clone(),
            credential_scope: self.credential_scope(),
            items: output.items,
        });
        if !calls.is_empty() {
            Ok(LlmResponse::ToolCalls {
                tools: calls,
                thought: (!output.thought.is_empty()).then_some(output.thought),
                content: (!output.text.is_empty()).then_some(output.text),
            })
        } else {
            Ok(if output.text.trim().is_empty() {
                LlmResponse::Empty
            } else {
                LlmResponse::Message(output.text)
            })
        }
    }
    #[allow(clippy::too_many_arguments)]
    async fn native_request(
        &self,
        messages: &[ChatMessage],
        max_tokens: u64,
        tools: bool,
        stream: bool,
        tx: Option<&mpsc::Sender<AgentEvent>>,
        cancel: &CancellationToken,
        timeout: Duration,
    ) -> Result<NativeOutput> {
        let body = self.native_body(messages, max_tokens, tools, stream)?;
        let url = format!("{}/{}", self.config.base_url, self.config.api_type.path());
        let response = {
            let mut attempt = 0;
            loop {
                attempt += 1;
                // Streaming uses the client's per-read timeout, so active long answers
                // are not cut off by a total request deadline.
                let request = self.client.post(&url);
                let request = if stream { request } else { request.timeout(timeout) };
                let request = self.authorize(request).json(&body);
                let response = tokio::select! {
                    _ = cancel.cancelled() => anyhow::bail!("Request cancelled"),
                    result = request.send() => result.map_err(|_| anyhow::anyhow!("Provider request failed"))?,
                };
                if response.status().as_u16() == 429 && attempt < 4 {
                    tokio::select! { _ = cancel.cancelled() => anyhow::bail!("Request cancelled"),
                    _ = tokio::time::sleep(Duration::from_millis(500 * attempt)) => {} }
                    continue;
                }
                if !response.status().is_success() {
                    let status = response.status();
                    let text = tokio::select! { _ = cancel.cancelled() => anyhow::bail!("Request cancelled"), result = response.text() => result? };
                    anyhow::bail!("Provider API error (HTTP {status}): {}", self.redact(&text));
                }
                break response;
            }
        };
        let output = if stream {
            self.read_native_stream(
                response,
                tx.context("Streaming requires event sink")?,
                cancel,
            )
            .await
            .map_err(|error| anyhow::anyhow!("{}", self.redact(&error.to_string())))?
        } else {
            let value: Value = tokio::select! { _ = cancel.cancelled() => anyhow::bail!("Request cancelled"), result = response.json() => result? };
            NativeOutput::parse(self.config.api_type, value)
                .map_err(|error| anyhow::anyhow!("{}", self.redact(&error.to_string())))?
        };
        Ok(output)
    }
    async fn read_native_stream(
        &self,
        response: reqwest::Response,
        tx: &mpsc::Sender<AgentEvent>,
        cancel: &CancellationToken,
    ) -> Result<NativeOutput> {
        let mut bytes = response.bytes_stream();
        let mut buffer = Vec::new();
        let mut data = String::new();
        let mut state = NativeStream::default();
        loop {
            let chunk = tokio::select! { _ = cancel.cancelled() => anyhow::bail!("Request cancelled"), result = bytes.next() => result };
            let chunk = match chunk {
                Some(Ok(chunk)) => chunk,
                Some(Err(error)) => {
                    return Err(anyhow::anyhow!(
                        "Native stream interrupted: {}",
                        error.without_url()
                    ))
                }
                None => return Err(anyhow::anyhow!("Native stream closed before completion")),
            };
            buffer.extend_from_slice(&chunk);
            anyhow::ensure!(
                buffer.len() + data.len() <= 8 * 1024 * 1024,
                "Provider stream event is too large"
            );
            while let Some(end) = buffer.iter().position(|byte| *byte == b'\n') {
                let line = String::from_utf8(buffer.drain(..=end).collect())?;
                let line = line.trim_end_matches(['\r', '\n']);
                if let Some(fragment) = line.strip_prefix("data:") {
                    if !data.is_empty() {
                        data.push('\n');
                    }
                    data.push_str(fragment.trim_start());
                } else if line.is_empty() && !data.is_empty() {
                    let value: Value = serde_json::from_str(&std::mem::take(&mut data))?;
                    if let Some(output) = state.event(self.config.api_type, value, tx).await? {
                        return Ok(output);
                    }
                }
            }
        }
    }
}
#[derive(Default)]
struct NativeOutput {
    text: String,
    thought: String,
    calls: Vec<ToolCall>,
    items: Vec<Value>,
    finish: Option<String>,
}
impl NativeOutput {
    fn parse(api: ApiType, value: Value) -> Result<Self> {
        let responses = api == ApiType::OpenaiResponses;
        anyhow::ensure!(
            value.get("error").is_none_or(Value::is_null) && value["status"] != "failed",
            "Provider failed: {}",
            value["error"]
        );
        let items = value[if responses { "output" } else { "content" }]
            .as_array()
            .context("Provider omitted response content")?
            .clone();
        let mut output = Self {
            items: items.clone(),
            ..Default::default()
        };
        output.finish = Some(
            if value["status"] == "incomplete"
                || matches!(
                    value["stop_reason"].as_str(),
                    Some("max_tokens" | "model_context_window_exceeded")
                )
            {
                "length"
            } else {
                "stop"
            }
            .into(),
        );
        for item in items {
            match item["type"].as_str().unwrap_or("") {
                "message" => {
                    for part in item["content"].as_array().into_iter().flatten() {
                        if part["type"] == "refusal" {
                            output.text.push_str(part["refusal"].as_str().unwrap_or(""));
                        } else if part["type"] == "output_text" {
                            output.text.push_str(part["text"].as_str().unwrap_or(""));
                        }
                    }
                }
                "thinking" => output
                    .thought
                    .push_str(item["thinking"].as_str().unwrap_or("")),
                "reasoning" => {
                    for summary in item["summary"].as_array().into_iter().flatten() {
                        output
                            .thought
                            .push_str(summary["text"].as_str().unwrap_or(""));
                    }
                }
                "text" => output.text.push_str(item["text"].as_str().unwrap_or("")),
                "function_call" | "tool_use" => {
                    let id = item[if responses { "call_id" } else { "id" }]
                        .as_str()
                        .context("Provider omitted tool call ID")?;
                    output.calls.push(ToolCall {
                        id: id.into(),
                        name: item["name"]
                            .as_str()
                            .context("Provider omitted tool name")?
                            .into(),
                        arguments: if responses {
                            item["arguments"]
                                .as_str()
                                .context("Invalid function arguments")?
                                .into()
                        } else {
                            item["input"].to_string()
                        },
                    });
                }
                _ => {}
            }
        }
        anyhow::ensure!(
            output.calls.len() <= 128,
            "Provider returned too many tool calls"
        );
        Ok(output)
    }
}
#[derive(Default)]
struct NativeStream {
    blocks: BTreeMap<u64, Value>,
    arguments: BTreeMap<u64, String>,
    usage: Value,
    stop: Value,
}
impl NativeStream {
    async fn event(
        &mut self,
        api: ApiType,
        event: Value,
        tx: &mpsc::Sender<AgentEvent>,
    ) -> Result<Option<NativeOutput>> {
        let kind = event["type"].as_str().unwrap_or("");
        let mut text = None;
        let mut thought = None;
        if api == ApiType::OpenaiResponses {
            match kind {
                "response.output_text.delta" | "response.refusal.delta" => {
                    text = event["delta"].as_str()
                }
                "response.reasoning_summary_text.delta" => thought = event["delta"].as_str(),
                "response.completed" | "response.incomplete" => {
                    return NativeOutput::parse(api, event["response"].clone()).map(Some)
                }
                "response.failed" | "error" => anyhow::bail!(
                    "Provider stream failed: {}",
                    event.get("error").unwrap_or(&event["response"]["error"])
                ),
                _ => {}
            }
        } else {
            let index = event["index"].as_u64().unwrap_or(0);
            anyhow::ensure!(index < 128, "Invalid content block index");
            match kind {
                "message_start" => self.usage = event["message"]["usage"].clone(),
                "content_block_start" => {
                    self.blocks.insert(index, event["content_block"].clone());
                    // Some gateways send an entire text block at its start.
                    text = event["content_block"]["text"].as_str();
                }
                "content_block_delta" => {
                    let block = self
                        .blocks
                        .get_mut(&index)
                        .context("Delta without content block")?;
                    let delta = &event["delta"];
                    let (field, fragment) = match delta["type"].as_str().unwrap_or("") {
                        "text_delta" => {
                            text = delta["text"].as_str();
                            ("text", text)
                        }
                        "thinking_delta" => {
                            thought = delta["thinking"].as_str();
                            ("thinking", thought)
                        }
                        "signature_delta" => ("signature", delta["signature"].as_str()),
                        "input_json_delta" => {
                            self.arguments.entry(index).or_default().push_str(
                                delta["partial_json"]
                                    .as_str()
                                    .context("Invalid tool delta")?,
                            );
                            ("", None)
                        }
                        _ => ("", None),
                    };
                    if let Some(fragment) = fragment {
                        let mut value = block[field].as_str().unwrap_or("").to_string();
                        value.push_str(fragment);
                        block[field] = json!(value);
                    }
                }
                "content_block_stop" => {
                    if let Some(arguments) = self.arguments.remove(&index) {
                        self.blocks.get_mut(&index).context("Unknown tool block")?["input"] =
                            serde_json::from_str(&arguments)
                                .map_err(|_| anyhow::anyhow!("Incomplete native tool arguments"))?;
                    }
                }
                "message_delta" => {
                    self.stop = event["delta"]["stop_reason"].clone();
                    if let Some(usage) = event["usage"].as_object() {
                        for (key, value) in usage {
                            self.usage[key] = value.clone();
                        }
                    }
                }
                "message_stop" => {
                    anyhow::ensure!(self.arguments.is_empty(), "Unfinished tool blocks");
                    return NativeOutput::parse(api, json!({"content":self.blocks.values().collect::<Vec<_>>(),"usage":self.usage,"stop_reason":self.stop})).map(Some);
                }
                "error" => anyhow::bail!("Provider stream failed: {}", event["error"]),
                _ => {}
            }
        }
        if let Some(text) = text.filter(|text| !text.is_empty()) {
            let _ = tx.send(AgentEvent::AssistantToken(text.into())).await;
        }
        if let Some(thought) = thought.filter(|text| !text.is_empty()) {
            let _ = tx.send(AgentEvent::ThoughtToken(thought.into())).await;
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    fn config() -> Config {
        serde_json::from_value(json!({"api_key":"legacy-key","base_url":"http://localhost/v1","model":"fixture","workspace_dir":std::env::temp_dir(),"auto_approve":false})).unwrap()
    }
    #[test]
    fn custom_headers_override_defaults_without_duplicate_credentials() {
        for api in [
            ApiType::ChatCompletions,
            ApiType::OpenaiResponses,
            ApiType::AnthropicMessages,
        ] {
            let mut config = config();
            config.api_type = api;
            let name = if api == ApiType::AnthropicMessages {
                "x-api-key"
            } else {
                "authorization"
            };
            config
                .provider_headers
                .insert(name.into(), "custom-value".into());
            let client = LlmClient::new(config);
            let request = client
                .authorize(client.client.get("http://localhost/v1/models"))
                .build()
                .unwrap();
            assert_eq!(request.headers().get_all(name).iter().count(), 1);
            assert_eq!(request.headers()[name], "custom-value");
        }
    }
    fn message(role: &str, text: &str) -> ChatMessage {
        ChatMessage {
            provider_data: None,
            role: role.into(),
            content: Some(text.into()),
            image_urls: vec![],
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }
    fn event(value: Value) -> String {
        format!("data: {value}\r\n\r\n")
    }
    async fn server(
        api: ApiType,
        responses: Vec<(String, bool)>,
    ) -> (LlmClient, tokio::task::JoinHandle<Vec<Value>>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            let mut requests = vec![];
            for (body, stream) in responses {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut bytes = vec![];
                let mut chunk = [0; 4096];
                let header_end = loop {
                    let n = socket.read(&mut chunk).await.unwrap();
                    assert!(n > 0);
                    bytes.extend_from_slice(&chunk[..n]);
                    if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                        break end + 4;
                    }
                };
                let headers = String::from_utf8_lossy(&bytes[..header_end]).to_lowercase();
                assert!(
                    headers.starts_with(&format!("post /v1/{} ", api.path())),
                    "{headers}"
                );
                assert!(headers.contains("x-route: tenant-a\r\n"));
                if api == ApiType::AnthropicMessages {
                    assert!(headers.contains("x-api-key: profile-key\r\n"));
                    assert!(headers.contains("anthropic-version: 2023-06-01\r\n"));
                    assert!(!headers.contains("authorization:"));
                } else {
                    assert!(headers.contains("authorization: bearer profile-key\r\n"));
                }
                let len = headers
                    .lines()
                    .find_map(|l| l.strip_prefix("content-length:"))
                    .unwrap()
                    .trim()
                    .parse::<usize>()
                    .unwrap();
                while bytes.len() < header_end + len {
                    let n = socket.read(&mut chunk).await.unwrap();
                    assert!(n > 0);
                    bytes.extend_from_slice(&chunk[..n]);
                }
                requests
                    .push(serde_json::from_slice(&bytes[header_end..header_end + len]).unwrap());
                let header=format!("HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",if stream {"text/event-stream"} else {"application/json"},body.len());
                socket.write_all(header.as_bytes()).await.unwrap();
                for part in body.as_bytes().chunks(7) {
                    socket.write_all(part).await.unwrap();
                }
            }
            requests
        });
        let mut config = config();
        config.api_type = api;
        config.api_key = "profile-key".into();
        config.base_url = format!("http://{address}/v1");
        config.model = "deployment".into();
        config
            .provider_headers
            .insert("x-route".into(), "tenant-a".into());
        (LlmClient::new(config), task)
    }
    fn tool_output(api: ApiType) -> Value {
        if api == ApiType::OpenaiResponses {
            json!({"status":"completed","usage":{"input_tokens":10,"output_tokens":5},"output":[
                {"type":"reasoning","id":"reasoning-1","summary":[],"encrypted_content":"encrypted"},
                {"type":"function_call","id":"fc-1","call_id":"call-1","name":"read_file","arguments":"{\"path\":\"x\"}"}
            ]})
        } else {
            json!({"stop_reason":"tool_use","usage":{"input_tokens":10,"output_tokens":5},"content":[
                {"type":"thinking","thinking":"checking","signature":"signed"},
                {"type":"tool_use","id":"call-1","name":"read_file","input":{"path":"x"}}
            ]})
        }
    }
    fn final_output(api: ApiType) -> Value {
        if api == ApiType::OpenaiResponses {
            json!({"status":"completed","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":"Готово"}]}]})
        } else {
            json!({"stop_reason":"end_turn","content":[{"type":"text","text":"Готово"}]})
        }
    }
    #[tokio::test]
    async fn native_stream_tool_roundtrip_preserves_signed_items_and_session_serialization() {
        for api in [ApiType::OpenaiResponses, ApiType::AnthropicMessages] {
            let body = if api == ApiType::OpenaiResponses {
                event(json!({"type":"response.output_text.delta","delta":"Проверю"}))
                    + &event(json!({"type":"response.completed","response":tool_output(api)}))
            } else {
                [json!({"type":"message_start","message":{"usage":{"input_tokens":10,"output_tokens":0}}}),
                 json!({"type":"content_block_start","index":0,"content_block":{"type":"thinking","thinking":"","signature":""}}),
                 json!({"type":"content_block_delta","index":0,"delta":{"type":"thinking_delta","thinking":"checking"}}),
                 json!({"type":"content_block_delta","index":0,"delta":{"type":"signature_delta","signature":"signed"}}),
                 json!({"type":"content_block_stop","index":0}),
                 json!({"type":"content_block_start","index":1,"content_block":{"type":"tool_use","id":"call-1","name":"read_file","input":{}}}),
                 json!({"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":"{\"path\":"}}),
                 json!({"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":"\"x\"}"}}),
                 json!({"type":"content_block_stop","index":1}),
                 json!({"type":"message_delta","delta":{"stop_reason":"tool_use"},"usage":{"output_tokens":5}}),
                 json!({"type":"message_stop"})].into_iter().map(event).collect::<String>()
            };
            let (client, server) = server(
                api,
                vec![(body, true), (final_output(api).to_string(), false)],
            )
            .await;
            let (tx, mut rx) = mpsc::channel(64);
            let mut messages = vec![
                message("system", "system instruction"),
                message("user", "inspect"),
            ];
            messages[1]
                .image_urls
                .push("data:image/png;base64,aGVsbG8=".into());
            let result = client
                .chat_step_stream(&messages, &tx, &CancellationToken::new())
                .await
                .unwrap();
            let LlmResponse::ToolCalls { tools: calls, .. } = result else {
                panic!("Expected tools")
            };
            assert_eq!(calls[0].id, "call-1");
            assert_eq!(calls[0].arguments, "{\"path\":\"x\"}");
            let mut assistant = message("assistant", "");
            assistant.content = None;
            assistant.provider_data = client.take_provider_data();
            assistant.tool_calls = Some(vec![LlmToolCall {
                id: calls[0].id.clone(),
                call_type: "function".into(),
                function: LlmFunctionCall {
                    name: calls[0].name.clone(),
                    arguments: calls[0].arguments.clone(),
                },
            }]);
            let mut other_config = client.config.clone();
            other_config.api_key = "other-key".into();
            let other = LlmClient::new(other_config);
            assert!(
                other.replay(&assistant).is_none(),
                "Opaque continuation data crossed credential scopes"
            );
            messages.push(assistant);
            let mut tool = message("tool", "file text");
            tool.tool_call_id = Some("call-1".into());
            messages.push(tool);
            let messages: Vec<ChatMessage> =
                serde_json::from_str(&serde_json::to_string(&messages).unwrap()).unwrap();
            let result = client.chat_step(&messages).await.unwrap();
            assert!(matches!(result,LlmResponse::Message(ref text) if text=="Готово"));
            let requests = server.await.unwrap();
            if api == ApiType::OpenaiResponses {
                assert_eq!(requests[0]["tools"][0]["strict"], false);
                assert_eq!(requests[0]["store"], false);
                assert_eq!(requests[1]["input"][2]["encrypted_content"], "encrypted");
                assert_eq!(requests[1]["input"][4]["call_id"], "call-1");
                assert_eq!(requests[0]["input"][1]["content"][1]["type"], "input_image");
            } else {
                assert_eq!(requests[0]["system"], "system instruction");
                assert!(requests[0]["tools"][0]["input_schema"].is_object());
                assert_eq!(
                    requests[1]["messages"][1]["content"][0]["signature"],
                    "signed"
                );
                assert_eq!(
                    requests[1]["messages"][2]["content"][0]["tool_use_id"],
                    "call-1"
                );
                assert_eq!(
                    requests[0]["messages"][0]["content"][1]["source"]["media_type"],
                    "image/png"
                );
            }
            while let Ok(event) = rx.try_recv() {
                if let AgentEvent::ThoughtToken(text) = event {
                    assert!(!text.contains("signed"));
                }
            }
        }
    }
    #[tokio::test]
    async fn native_eof_never_executes_partial_calls() {
        for api in [ApiType::OpenaiResponses, ApiType::AnthropicMessages] {
            let partial = if api == ApiType::OpenaiResponses {
                event(
                    json!({"type":"response.output_item.added","item":{"type":"function_call","call_id":"bad","name":"read_file"}}),
                )
            } else {
                event(
                    json!({"type":"content_block_start","index":0,"content_block":{"type":"tool_use","id":"bad","name":"read_file","input":{}}}),
                )
            };
            let (client, server) = server(api, vec![(partial.clone(), true)]).await;
            let (tx, _rx) = mpsc::channel(64);
            assert!(client
                .chat_step_stream(
                    &[message("user", "inspect")],
                    &tx,
                    &CancellationToken::new()
                )
                .await
                .is_err());
            assert!(client.take_provider_data().is_none());
            assert_eq!(server.await.unwrap().len(), 1);
        }
    }

    #[tokio::test]
    async fn cancellation_interrupts_native_headers_and_body_without_retrying() {
        for api in [ApiType::OpenaiResponses, ApiType::AnthropicMessages] {
            for send_headers in [false, true] {
                let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
                let address = listener.local_addr().unwrap();
                let cancel = CancellationToken::new();
                let signal = cancel.clone();
                let server = tokio::spawn(async move {
                    let (mut socket, _) = listener.accept().await.unwrap();
                    let mut request = [0; 4096];
                    assert!(socket.read(&mut request).await.unwrap() > 0);
                    if send_headers {
                        socket
                            .write_all(
                                b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n\r\n",
                            )
                            .await
                            .unwrap();
                    }
                    signal.cancel();
                    tokio::time::sleep(Duration::from_millis(200)).await;
                });
                let mut config = config();
                config.api_type = api;
                config.base_url = format!("http://{address}/v1");
                let client = LlmClient::new(config);
                let (tx, _rx) = mpsc::channel(16);
                let result = tokio::time::timeout(
                    Duration::from_secs(1),
                    client.chat_step_stream(&[message("user", "hello")], &tx, &cancel),
                )
                .await
                .unwrap();
                assert!(result.unwrap_err().to_string().contains("cancelled"));
                assert!(client.take_provider_data().is_none());
                server.await.unwrap();
            }
        }
    }
    #[tokio::test]
    async fn native_titles_and_truncated_calls_use_the_selected_protocol() {
        for api in [ApiType::OpenaiResponses, ApiType::AnthropicMessages] {
            let mut truncated = tool_output(api);
            if api == ApiType::OpenaiResponses {
                truncated["status"] = json!("incomplete");
            } else {
                truncated["stop_reason"] = json!("max_tokens");
            }
            let (client, server) = server(
                api,
                vec![
                    (final_output(api).to_string(), false),
                    (truncated.to_string(), false),
                ],
            )
            .await;
            assert_eq!(client.generate_title("prompt").await.unwrap(), "Готово");
            assert!(client
                .chat_step(&[message("user", "inspect")])
                .await
                .is_err());
            assert!(client.take_provider_data().is_none());
            assert!(server.await.unwrap()[0].get("tools").is_none());
        }
    }
}
