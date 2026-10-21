use crate::{commands, core::{agent::{Agent, AgentEvent}, config::Config, llm::{ChatMessage, LlmClient, LlmResponse}, session::{HistoryItem, Session}}, ipc::Channel, state::AppState, State};
use std::{collections::HashMap, path::PathBuf, sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}}};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, sync::{Mutex, mpsc}};
use tokio_util::sync::CancellationToken;
struct Workspace(PathBuf);
impl Workspace { fn new() -> Self { let p = std::env::temp_dir().join(format!("takiza-gui-bridge-{}-{}", std::process::id(), chrono::Utc::now().timestamp_nanos_opt().unwrap())); std::fs::create_dir_all(&p).unwrap(); Self(p) } }
impl Drop for Workspace { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }
fn config(workspace: PathBuf, base_url: String) -> Config {
    Config { api_key_env: None, api_type: Default::default(), provider_headers: Default::default(), active_provider: None, providers: Default::default(), api_key: "fixture-key".into(), base_url, model: "fixture".into(), workspace_dir: workspace,
        auto_approve: true, proxy: None, effort: None, mode: Some("manual".into()), theme: None, language: None, max_steps: 100 }
}
fn state(workspace: &Workspace) -> AppState {
    let cfg = config(workspace.0.clone(), "http://127.0.0.1:1/v1".into());
    AppState { turn_active: AtomicBool::new(false), agent: Mutex::new(Agent::new(cfg)),
        current_session: Arc::new(Mutex::new(Some(Session::new("fixture".into())))), cancel_token: Mutex::new(CancellationToken::new()),
        permissions: Arc::new(Mutex::new(HashMap::new())), questions: Arc::new(Mutex::new(HashMap::new())), workspace_dir: Mutex::new(workspace.0.clone()) }
}
async fn server(body: Vec<u8>, requests: usize, split: bool) -> (String, Arc<AtomicUsize>, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let count = Arc::new(AtomicUsize::new(0)); let observed = count.clone();
    let task = tokio::spawn(async move {
        for _ in 0..requests {
            let (mut stream, _) = listener.accept().await.unwrap(); stream.set_nodelay(true).unwrap();
            let mut input = Vec::new();
            loop {
                let mut chunk = [0u8; 4096]; let n = stream.read(&mut chunk).await.unwrap(); if n == 0 { break; } input.extend_from_slice(&chunk[..n]);
                if let Some(end) = input.windows(4).position(|window| window == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&input[..end]).to_lowercase();
                    let length = headers.lines().find_map(|line| line.strip_prefix("content-length:").and_then(|v| v.trim().parse::<usize>().ok())).unwrap_or(0);
                    if input.len() >= end + 4 + length { break; }
                }
            }
            observed.fetch_add(1, Ordering::SeqCst);
            stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).as_bytes()).await.unwrap();
            if split { for byte in &body { stream.write_all(&[*byte]).await.unwrap(); tokio::time::sleep(std::time::Duration::from_millis(1)).await; } }
            else { stream.write_all(&body).await.unwrap(); }
        }
    });
    (format!("http://{address}/v1"), count, task)
}
#[tokio::test]
async fn direct_command_never_calls_model_and_preserves_history() {
    let ws = Workspace::new(); let state = state(&ws);
    let events = Arc::new(std::sync::Mutex::new(Vec::new())); let captured = events.clone();
    let channel = Channel(Arc::new(move |event| captured.lock().unwrap().push(event)));
    commands::start_agent_turn("!echo direct-fixture".into(), channel, State(&state)).await.unwrap();
    let session = commands::get_current_session(State(&state)).await.unwrap().unwrap();
    assert!(session.history.iter().any(|item| matches!(item, HistoryItem::ToolEnd { result, is_error: false, .. } if result.contains("direct-fixture"))));
    assert!(events.lock().unwrap().iter().any(|event| matches!(event, AgentEvent::Finished)));
    assert!(!state.turn_active.load(Ordering::SeqCst));
}
#[tokio::test]
async fn restore_replaces_files_chat_and_creates_undo_checkpoint() {
    let ws = Workspace::new(); let state = state(&ws);
    std::fs::write(ws.0.join("file.txt"), "before").unwrap();
    let mut before = Session::new("fixture".into()); before.history.push(HistoryItem::AssistantMessage("old answer".into()));
    let point = crate::core::checkpoints::capture_session(&ws.0, &before, "return here").unwrap();
    std::fs::write(ws.0.join("file.txt"), "after").unwrap();
    let mut after = before.clone(); after.history.push(HistoryItem::AssistantMessage("remove me".into()));
    *state.current_session.lock().await = Some(after);
    let restored = commands::restore_checkpoint(point.id, State(&state)).await.unwrap();
    assert_eq!(std::fs::read_to_string(ws.0.join("file.txt")).unwrap(), "before");
    assert_eq!(restored.history.len(), 1);
    assert_eq!(commands::get_current_session(State(&state)).await.unwrap().unwrap().history.len(), 1);
    assert_eq!(crate::core::checkpoints::list(&ws.0, &before.id).unwrap().len(), 2);
}
#[tokio::test]
async fn attachments_keep_documents_as_paths_and_images_in_multimodal_messages() {
    let ws = Workspace::new(); let state = state(&ws);
    let image = ws.0.join("some image.png"); std::fs::write(&image, b"\x89PNG\r\n\x1a\nfixture").unwrap();
    let doc = ws.0.join("notes.txt"); std::fs::write(&doc, "not injected").unwrap();
    let imported = commands::import_paths(vec![image.to_string_lossy().into(), doc.to_string_lossy().into()], State(&state)).await.unwrap();
    let json = serde_json::to_value(imported).unwrap();
    assert_eq!(json["images"].as_array().unwrap().len(), 1);
    assert!(json["images"][0]["url"].as_str().unwrap().starts_with("data:image/png;base64,"));
    assert!(json["text"].as_str().unwrap().contains("notes.txt"));
    assert!(!json["text"].as_str().unwrap().contains("not injected"));
}
#[tokio::test]
async fn sse_split_utf8_and_unterminated_last_event_are_preserved() {
    let ws = Workspace::new();
    let body = "data: {\"choices\":[{\"delta\":{\"content\":\"Привет 🐹\"},\"finish_reason\":\"stop\"}]}".as_bytes().to_vec();
    let (url, _, task) = server(body, 1, true).await;
    let llm = LlmClient::new(config(ws.0.clone(), url)); let (tx, mut rx) = mpsc::channel(100);
    let messages: Vec<ChatMessage> = serde_json::from_value(serde_json::json!([{"role":"user","content":"hi"}])).unwrap();
    let response = llm.chat_step_stream(&messages, &tx, &CancellationToken::new()).await.unwrap();
    assert!(matches!(response, LlmResponse::Message(ref text) if text == "Привет 🐹"));
    let mut streamed = String::new(); while let Ok(event) = rx.try_recv() { if let AgentEvent::AssistantToken(text) = event { streamed.push_str(&text); } }
    assert_eq!(streamed, "Привет 🐹"); task.await.unwrap();
}
#[tokio::test]
async fn empty_responses_retry_three_times_then_fail() {
    let ws = Workspace::new(); let (url, count, task) = server(b"data: [DONE]\n\n".to_vec(), 3, false).await;
    let mut agent = Agent::new(config(ws.0.clone(), url)); let (tx, mut rx) = mpsc::channel(100);
    agent.handle_user_input("test".into(), tx, CancellationToken::new(), Arc::new(Mutex::new(HashMap::new())), Arc::new(Mutex::new(HashMap::new()))).await;
    assert_eq!(count.load(Ordering::SeqCst), 3);
    let mut failed = false; while let Ok(event) = rx.try_recv() { if matches!(event, AgentEvent::Error(_)) { failed = true; } }
    assert!(failed); task.await.unwrap();
}
async fn cancellation_at_prompt(tool: &str, arguments: serde_json::Value, permission: bool) {
    let ws = Workspace::new();
    let body = format!("data: {}\n\ndata: [DONE]\n\n", serde_json::json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"fixture-call","function":{"name":tool,"arguments":arguments.to_string()}}]}}]}));
    let (url, _, server) = server(body.into_bytes(), 1, false).await;
    let mut cfg = config(ws.0.clone(), url); cfg.auto_approve = !permission;
    let mut agent = Agent::new(cfg); let (tx, mut rx) = mpsc::channel(100);
    let cancel = CancellationToken::new(); let token = cancel.clone();
    let permissions = Arc::new(Mutex::new(HashMap::new())); let questions = Arc::new(Mutex::new(HashMap::new()));
    let p = permissions.clone(); let q = questions.clone();
    let task = tokio::spawn(async move { agent.handle_user_input("test".into(), tx, token, p, q).await; });
    let mut reached = false;
    while let Some(event) = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv()).await.unwrap() {
        if matches!(event, AgentEvent::PermissionRequest { .. } | AgentEvent::QuestionRequest { .. }) { reached = true; cancel.cancel(); break; }
    }
    assert!(reached);
    tokio::time::timeout(std::time::Duration::from_secs(1), task).await.unwrap().unwrap();
    assert!(permissions.lock().await.is_empty()); assert!(questions.lock().await.is_empty()); server.await.unwrap();
}
#[tokio::test]
async fn cancelling_question_releases_agent_and_pending_response() {
    cancellation_at_prompt("ask_question", serde_json::json!({"questions":[{"question":"Choose","options":["one","two"]}]}), false).await;
}
#[tokio::test]
async fn cancelling_permission_releases_agent_and_pending_response() {
    cancellation_at_prompt("run_command", serde_json::json!({"command":"echo permission-fixture"}), true).await;
}
#[cfg(unix)]
#[tokio::test]
async fn cancelling_direct_command_stops_descendants_and_releases_turn() {
    let ws = Workspace::new(); let state = state(&ws);
    let (started, wait_started) = tokio::sync::oneshot::channel();
    let started = std::sync::Mutex::new(Some(started));
    let channel = Channel(Arc::new(move |event| {
        if matches!(event, AgentEvent::ToolLog(ref text) if text == "started") {
            if let Some(sender) = started.lock().unwrap().take() { let _ = sender.send(()); }
        }
    }));
    let turn = commands::start_agent_turn("!echo started; sleep 30 & wait".into(), channel, State(&state));
    let stop = async { wait_started.await.unwrap(); commands::cancel_agent(State(&state)).await.unwrap(); };
    let (result, _) = tokio::time::timeout(std::time::Duration::from_secs(2), async { tokio::join!(turn, stop) }).await.unwrap();
    result.unwrap(); assert!(!state.turn_active.load(Ordering::SeqCst));
}

#[tokio::test]
async fn object_arguments_and_parallel_calls_without_indices_are_not_discarded() {
    let ws = Workspace::new();
    let body = format!("data: {}\n\ndata: [DONE]\n\n", serde_json::json!({"choices":[{"delta":{"tool_calls":[
        {"id":"first","function":{"name":"read_file","arguments":{"path":"file.txt"}}},
        {"id":"second","function":{"name":"list_dir","arguments":{"path":"."}}}
    ]}}]}));
    let (url, _, task) = server(body.into_bytes(), 1, false).await;
    let llm = LlmClient::new(config(ws.0.clone(), url)); let (tx, _rx) = mpsc::channel(100);
    let response = llm.chat_step_stream(&[], &tx, &CancellationToken::new()).await.unwrap();
    match response {
        LlmResponse::ToolCalls { tools, .. } => {
            assert_eq!(tools.len(), 2); assert_eq!(tools[0].id, "first"); assert_eq!(tools[1].id, "second");
            assert_eq!(serde_json::from_str::<serde_json::Value>(&tools[0].arguments).unwrap()["path"], "file.txt");
        }, _ => panic!("Expected tool calls"),
    }
    task.await.unwrap();
}
#[tokio::test]
async fn incomplete_calls_stop_once_before_execution_and_date_is_current() {
    let ws = Workspace::new();
    let body = format!("data: {}\n\ndata: [DONE]\n\n", serde_json::json!({"choices":[{"delta":{"tool_calls":[
        {"index":0,"id":"bad","function":{"name":"web_search"}}
    ]}}]}));
    let (url, count, task) = server(body.into_bytes(), 1, false).await;
    let mut agent = Agent::new(config(ws.0.clone(), url));
    assert!(agent.get_messages()[0].content.as_ref().unwrap().contains(&chrono::Local::now().format("%Y-%m-%d").to_string()));
    let (tx, mut rx) = mpsc::channel(100);
    agent.handle_user_input("search news".into(), tx, CancellationToken::new(), Arc::new(Mutex::new(HashMap::new())), Arc::new(Mutex::new(HashMap::new()))).await;
    let mut errors = 0;
    while let Ok(event) = rx.try_recv() {
        assert!(!matches!(event, AgentEvent::ToolStart { .. }));
        if let AgentEvent::Error(error) = event { assert!(error.contains("missing JSON arguments")); errors += 1; }
    }
    assert_eq!(errors, 1); assert_eq!(count.load(Ordering::SeqCst), 1); task.await.unwrap();
}

#[tokio::test]
async fn provider_models_are_loaded_from_the_selected_endpoint() {
    let (url, count, task) = server(br#"{"data":[{"id":"provider/b"},{"id":"provider/a","architecture":{"input_modalities":["text","image"]}},{"id":"provider/b"}]}"#.to_vec(), 1, false).await;
    let models = commands::list_provider_models(url, "fixture-key".into(), None, None, None, None).await.unwrap();
    assert_eq!(models.iter().map(|model| model["id"].as_str().unwrap()).collect::<Vec<_>>(), vec!["provider/a", "provider/b"]);
    assert_eq!(models[0]["architecture"]["input_modalities"][1], "image");
    assert_eq!(count.load(Ordering::SeqCst), 1);
    task.await.unwrap();
}

#[tokio::test]
async fn unavailable_provider_catalog_does_not_offer_another_providers_models() {
    let (url, _, task) = server(br#"{"unsupported":[]}"#.to_vec(), 1, false).await;
    let error = commands::list_provider_models(url, String::new(), None, None, None, None).await.unwrap_err();
    assert!(error.contains("вручную"));
    task.await.unwrap();
    assert!(commands::list_provider_models("file:///tmp/models".into(), String::new(), None, None, None, None).await.is_err());
}

#[tokio::test]
async fn account_features_are_unavailable_without_login() {
    let ws = Workspace::new(); let state = state(&ws);
    assert!(commands::set_mode("moa".into(), State(&state)).await.unwrap_err().contains("в разработке"));
    assert!(commands::get_usage().await.unwrap_err().contains("провайдера"));
}

#[tokio::test]
async fn streamed_reasoning_is_saved_before_the_final_answer() {
    let ws = Workspace::new(); let state = state(&ws);
    let body = concat!(
        "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"Inspect the project first.\"}}]}\n\n",
        "data: {\"choices\":[{\"delta\":{\"content\":\"The project is ready.\"}}]}\n\n",
        "data: [DONE]\n\n"
    ).as_bytes().to_vec();
    let (url, _, task) = server(body, 1, false).await;
    *state.agent.lock().await = Agent::new(config(ws.0.clone(), url));
    let channel = Channel(Arc::new(|_| {}));
    commands::start_agent_turn("Review this project".into(), channel, State(&state)).await.unwrap();
    let session = commands::get_current_session(State(&state)).await.unwrap().unwrap();
    let thought = session.history.iter().position(|item| matches!(item, HistoryItem::Thought(text) if text == "Inspect the project first.")).unwrap();
    let answer = session.history.iter().position(|item| matches!(item, HistoryItem::AssistantMessage(text) if text == "The project is ready.")).unwrap();
    assert!(thought < answer, "Reasoning was persisted after the answer");
    task.await.unwrap();
}

#[test]
fn regeneration_targets_the_selected_occurrence_and_keeps_images() {
    let mut session = Session::new("fixture".into());
    session.history = vec![HistoryItem::UserPrompt("same".into()), HistoryItem::AssistantMessage("first".into()), HistoryItem::UserPrompt("same".into()), HistoryItem::AssistantMessage("old second".into())];
    session.messages = serde_json::from_value(serde_json::json!([
        {"role":"system","content":"system"}, {"role":"user","content":"same"}, {"role":"assistant","content":"first"},
        {"role":"user","content":"Continue."}, {"role":"assistant","content":"internal continuation"},
        {"role":"user","content":"same","image_urls":["data:image/png;base64,fixture"]}, {"role":"assistant","content":"old second"}
    ])).unwrap();
    let revised = session.for_regeneration(2, Some("edited")).unwrap();
    assert_eq!(revised.id, session.id);
    assert_eq!(revised.history.len(), 3);
    assert_eq!(revised.messages.len(), 6);
    assert_eq!(revised.messages[5].content.as_deref(), Some("edited"));
    assert_eq!(revised.messages[5].image_urls, vec!["data:image/png;base64,fixture"]);
    assert!(matches!(&revised.history[0], HistoryItem::UserPrompt(text) if text == "same"));
    assert_eq!(session.history.len(), 4, "Preparing a retry mutated the original session");
    assert!(session.for_regeneration(1, None).is_err());
    assert!(session.for_regeneration(2, Some("   ")).is_err());
}

#[tokio::test]
async fn regeneration_replaces_the_response_without_a_new_user_message() {
    let ws = Workspace::new(); let state = state(&ws);
    let (url, count, task) = server(b"data: {\"choices\":[{\"delta\":{\"content\":\"Replacement answer.\"}}]}\n\ndata: [DONE]\n\n".to_vec(), 1, false).await;
    let mut session = Session::new("fixture".into());
    session.history = vec![HistoryItem::UserPrompt("original prompt".into()), HistoryItem::AssistantMessage("old answer".into())];
    session.messages = serde_json::from_value(serde_json::json!([{"role":"user","content":"original prompt"},{"role":"assistant","content":"old answer"}])).unwrap();
    let id = session.id.clone();
    *state.current_session.lock().await = Some(session.clone());
    let mut agent = Agent::new(config(ws.0.clone(), url)); agent.set_messages(session.messages);
    *state.agent.lock().await = agent;
    let events = Arc::new(std::sync::Mutex::new(Vec::new())); let captured = events.clone();
    let channel = Channel(Arc::new(move |event| captured.lock().unwrap().push(event)));
    commands::regenerate_agent_turn(id.clone(), 0, None, channel, State(&state)).await.unwrap();
    let revised = commands::get_current_session(State(&state)).await.unwrap().unwrap();
    assert_eq!(revised.id, id);
    assert_eq!(revised.history.iter().filter(|item| matches!(item, HistoryItem::UserPrompt(_))).count(), 1);
    assert_eq!(revised.messages.iter().filter(|message| message.role == "user").count(), 1);
    assert!(!events.lock().unwrap().iter().any(|event| matches!(event, AgentEvent::UserMessage(_))));
    assert!(revised.history.iter().any(|item| matches!(item, HistoryItem::AssistantMessage(text) if text == "Replacement answer.")));
    assert!(!revised.history.iter().any(|item| matches!(item, HistoryItem::AssistantMessage(text) if text == "old answer")));
    assert_eq!(Session::load(&ws.0, &id).unwrap().history.len(), revised.history.len());
    assert!(!crate::core::checkpoints::list(&ws.0, &id).unwrap().is_empty());
    assert_eq!(count.load(Ordering::SeqCst), 1); task.await.unwrap();
}

#[tokio::test]
async fn text_attachments_are_cached_inside_workspace_and_validated() {
    let ws = Workspace::new(); let state = state(&ws);
    let imported = commands::import_text_file("../../test.rs".into(), b"fn main() {}".to_vec(), State(&state)).await.unwrap();
    let value = serde_json::to_value(imported).unwrap();
    let marker = value["marker"].as_str().unwrap();
    let file: serde_json::Value = serde_json::from_str(marker.strip_prefix("[File: ").unwrap().strip_suffix(']').unwrap()).unwrap();
    let path = PathBuf::from(file["path"].as_str().unwrap());
    assert_eq!(path.parent().unwrap(), ws.0.join(".takiza/attachments"));
    assert_eq!(std::fs::read_to_string(path).unwrap(), "fn main() {}");
    assert_eq!(value["size"], 12);
    assert!(commands::import_text_file("binary.txt".into(), vec![0, 1, 2], State(&state)).await.is_err());
    assert!(commands::import_text_file("too-large.txt".into(), vec![b'x'; 1024 * 1024 + 1], State(&state)).await.is_err());
}

#[tokio::test]
async fn provider_catalog_uses_protocol_auth_env_headers_and_anthropic_pagination() {
    use crate::core::providers::ApiType;
    use std::collections::BTreeMap;
    let variable = format!("TAKIZA_GUI_TEST_KEY_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap());
    std::env::set_var(&variable, "env-fixture-token");
    for api in [ApiType::ChatCompletions, ApiType::OpenaiResponses, ApiType::AnthropicMessages] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            for page in 0..if api == ApiType::AnthropicMessages { 2 } else { 1 } {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut bytes = Vec::new(); let mut chunk = [0;4096];
                while !bytes.windows(4).any(|window| window == b"\r\n\r\n") { let n = socket.read(&mut chunk).await.unwrap(); assert!(n>0); bytes.extend_from_slice(&chunk[..n]); }
                let request = String::from_utf8_lossy(&bytes).to_lowercase();
                assert!(request.starts_with("get /v1/models"));
                assert!(request.contains("x-tenant: fixture\r\n"));
                if api == ApiType::AnthropicMessages {
                    assert_eq!(request.matches("x-api-key:").count(),1);
                    assert!(request.contains("x-api-key: override-key\r\n"));
                    assert!(request.contains("anthropic-version: custom-version\r\n"));
                    if page == 1 { assert!(request.contains("after_id=first")); }
                } else if api == ApiType::OpenaiResponses {
                    assert!(request.contains("authorization: bearer env-fixture-token\r\n"));
                    assert!(!request.contains("unused-key"));
                } else {
                    assert_eq!(request.matches("authorization:").count(),1);
                    assert!(request.contains("authorization: basic dXNlcjpwYXNz\r\n".to_lowercase().as_str()));
                    assert!(!request.contains("unused-key"));
                }
                let body = if api == ApiType::AnthropicMessages && page == 0 { serde_json::json!({"data":[{"id":"first","display_name":"First"}],"has_more":true,"last_id":"first"}) }
                    else { serde_json::json!({"data":[{"id":"second","display_name":"Second","input_modalities":["text","image"]}],"has_more":false}) }.to_string();
                socket.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();
            }
        });
        let mut headers = BTreeMap::from([("x-tenant".into(),"fixture".into())]);
        if api == ApiType::AnthropicMessages { headers.insert("X-API-Key".into(),"override-key".into());headers.insert("Anthropic-Version".into(),"custom-version".into()); }
        if api == ApiType::ChatCompletions { headers.insert("Authorization".into(),"Basic dXNlcjpwYXNz".into()); }
        let result = commands::list_provider_models(format!("http://{address}/v1"),"unused-key".into(),None,Some(api),Some(headers),
            (api == ApiType::OpenaiResponses).then(|| variable.clone())).await.unwrap();
        assert_eq!(result.len(),if api == ApiType::AnthropicMessages {2}else{1});
        assert_eq!(result.last().unwrap()["name"],"Second");
        assert_eq!(result.last().unwrap()["input_modalities"][1],"image");
        server.await.unwrap();
    }
    std::env::remove_var(&variable);
}

#[tokio::test]
async fn ordinary_startup_is_empty_and_saved_history_requires_explicit_selection() {
    let workspace = Workspace::new();
    let cfg = config(workspace.0.clone(), "http://unused.invalid/v1".into());
    let mut saved = Session::new("previous-model".into());
    saved.history.push(HistoryItem::UserPrompt("previous conversation".into()));
    saved.messages.push(ChatMessage { role:"user".into(), content:Some("previous conversation".into()), image_urls:vec![], tool_calls:None,tool_call_id:None,name:None,provider_data:None });
    saved.save(&workspace.0).unwrap();
    let original = std::fs::read(Session::sessions_dir(&workspace.0).join(format!("{}.json", saved.id))).unwrap();
    let state = AppState::from_config(cfg.clone());
    let fresh = commands::get_current_session(State(&state)).await.unwrap().unwrap();
    assert_ne!(fresh.id,saved.id);
    assert!(fresh.history.is_empty() && fresh.messages.is_empty());
    assert_eq!(state.agent.lock().await.message_count(),0);
    assert_eq!(Session::list_meta(&workspace.0).len(),1,"startup must not persist empty chats");
    let loaded = commands::load_session(saved.id.clone(),State(&state)).await.unwrap();
    assert_eq!(loaded.id,saved.id);
    assert_eq!(state.agent.lock().await.message_count(),1);
    let restarted = AppState::from_config(cfg);
    assert!(restarted.current_session.lock().await.as_ref().unwrap().history.is_empty());
    assert_eq!(original,std::fs::read(Session::sessions_dir(&workspace.0).join(format!("{}.json",saved.id))).unwrap());
}

async fn gui_grill_roundtrip(skipped: bool) {
    use crate::core::agent::{QuestionAnswer,QuestionResponse};
    let ws=Workspace::new();
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address=listener.local_addr().unwrap();
    let args=serde_json::json!({"topic":"Editor architecture","questions":[{"question":"Where should drafts live?","recommendation":"Local files","alternatives":["Cloud"]},{"question":"First audience?","recommendation":"Developers"}]});
    let first=format!("data: {}\n\ndata: [DONE]\n\n",serde_json::json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"grill-round","function":{"name":"grill-me","arguments":args.to_string()}}]}}]}));
    let last="data: {\"choices\":[{\"delta\":{\"content\":\"Interview summary.\"}}]}\n\ndata: [DONE]\n\n".to_string();
    let server=tokio::spawn(async move {
        let mut requests=Vec::new();
        for body in [first,last] {
            let (mut socket,_)=listener.accept().await.unwrap();let mut bytes=Vec::new();
            loop {
                let mut chunk=[0;4096];let n=socket.read(&mut chunk).await.unwrap();assert!(n>0);bytes.extend_from_slice(&chunk[..n]);
                if let Some(end)=bytes.windows(4).position(|w|w==b"\r\n\r\n") {
                    let headers=String::from_utf8_lossy(&bytes[..end]).to_lowercase();
                    let length=headers.lines().find_map(|line|line.strip_prefix("content-length:").and_then(|v|v.trim().parse::<usize>().ok())).unwrap();
                    if bytes.len()>=end+4+length {requests.push(serde_json::from_slice::<serde_json::Value>(&bytes[end+4..end+4+length]).unwrap());break;}
                }
            }
            socket.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();
        }
        requests
    });
    let state=Arc::new(state(&ws));let mut cfg=config(ws.0.clone(),format!("http://{address}/v1"));cfg.language=Some("ru".into());
    *state.agent.lock().await=Agent::new(cfg);
    let (tx,mut rx)=mpsc::unbounded_channel();let channel=Channel(Arc::new(move |event|{tx.send(event).unwrap();}));
    let active=state.clone();let task=tokio::spawn(async move {commands::start_agent_turn("/grill-me Editor architecture".into(),channel,State(&active)).await.unwrap();});
    let mut received=false;let mut started=false;
    while let Some(event)=tokio::time::timeout(std::time::Duration::from_secs(3),rx.recv()).await.unwrap() {
        match event {
            AgentEvent::ToolStart {name,..} if name=="grill-me" => started=true,
            AgentEvent::QuestionRequest {id,questions} => {
                assert_eq!(questions.len(),2);assert!(questions[0].question.contains("Рекомендуемый ответ:"));
                assert_eq!(questions[0].options,["Local files","Cloud"]);assert!(questions[0].allow_custom);
                let response=QuestionResponse {answers:vec![QuestionAnswer{question_index:0,question:questions[0].question.clone(),selected_options:vec!["Local files".into()],custom_text:Some("Encrypted at rest".into())}],skipped};
                commands::respond_question(id,response,State(&state)).await.unwrap();received=true;
            },
            AgentEvent::Finished => break,
            _=>{},
        }
    }
    task.await.unwrap();assert!(received && started);assert!(state.questions.lock().await.is_empty());
    let session=commands::get_current_session(State(&state)).await.unwrap().unwrap();
    assert!(session.history.iter().any(|item|matches!(item,HistoryItem::QuestionInteraction{answers,skipped:cancelled,..} if *cancelled==skipped && answers[0].custom_text.as_deref()==Some("Encrypted at rest"))));
    let requests=server.await.unwrap();
    assert!(requests[0]["tools"].as_array().unwrap().iter().any(|tool|tool["function"]["name"]=="grill-me"));
    let result=requests[1]["messages"].as_array().unwrap().iter().find(|message|message["role"]=="tool" && message["tool_call_id"]=="grill-round").unwrap();
    let result:serde_json::Value=serde_json::from_str(result["content"].as_str().unwrap()).unwrap();
    assert_eq!(result["status"],if skipped {"cancelled"} else {"completed"});
    assert_eq!(result["answers"][0]["custom_text"],"Encrypted at rest");
    if skipped {assert!(result["next_step"].as_str().unwrap().starts_with("Stop"));}
    assert_eq!(session.messages.iter().filter(|message|message.role=="user").count(),1);
}
#[tokio::test] async fn gui_grill_me_round_returns_answers_and_persists_question_history() {gui_grill_roundtrip(false).await;}
#[tokio::test] async fn gui_grill_me_cancel_preserves_partial_answers_without_default_implementation() {gui_grill_roundtrip(true).await;}
#[tokio::test] async fn gui_grill_me_stop_releases_pending_question() {
    cancellation_at_prompt("grill-me",serde_json::json!({"topic":"Architecture","questions":[{"question":"Storage?","recommendation":"Local"}]}),false).await;
}

#[tokio::test]
async fn generated_titles_update_history_in_the_background_and_preserve_new_messages() {
    let ws=Workspace::new();let state=state(&ws);
    let (url,count,task)=server(r#"{"choices":[{"message":{"content":"\"Название от модели\""}}]}"#.as_bytes().to_vec(),1,false).await;
    let cfg=config(ws.0.clone(),url);
    {let mut guard=state.current_session.lock().await;let session=guard.as_mut().unwrap();
        session.messages=serde_json::from_value(serde_json::json!([{"role":"user","content":"first message text"},{"role":"assistant","content":"answer"}])).unwrap();
        session.history=vec![HistoryItem::UserPrompt("first message text".into()),HistoryItem::AssistantMessage("answer".into())];session.save(&ws.0).unwrap();}
    let (tx,mut rx)=mpsc::unbounded_channel();let channel=Channel(Arc::new(move |event|{let _=tx.send(event);}));
    crate::core::titles::schedule(cfg.clone(),state.current_session.clone(),ws.0.clone(),channel.clone()).await;
    crate::core::titles::schedule(cfg,state.current_session.clone(),ws.0.clone(),channel).await;
    {let mut guard=state.current_session.lock().await;guard.as_mut().unwrap().history.push(HistoryItem::UserPrompt("new message while naming".into()));}
    let event=tokio::time::timeout(std::time::Duration::from_secs(2),rx.recv()).await.unwrap().unwrap();
    let id=match event {AgentEvent::SessionTitle{session_id,title}=>{assert_eq!(title,"Название от модели");session_id},_=>panic!("expected title update")};
    let saved=Session::load(&ws.0,&id).unwrap();assert_eq!(saved.title.as_deref(),Some("Название от модели"));assert_eq!(saved.history.len(),3);
    assert_eq!(commands::get_sessions(State(&state)).await.unwrap()[0].title,"Название от модели");
    assert_eq!(count.load(Ordering::SeqCst),1);task.await.unwrap();
}

#[tokio::test]
async fn late_title_never_renames_a_different_chat() {
    let ws=Workspace::new();let state=state(&ws);
    let (url,_,task)=server(br#"{"choices":[{"message":{"content":"Old conversation title"}}]}"#.to_vec(),1,true).await;
    {let mut guard=state.current_session.lock().await;let session=guard.as_mut().unwrap();
        session.messages=serde_json::from_value(serde_json::json!([{"role":"user","content":"Old prompt"}])).unwrap();session.history.push(HistoryItem::AssistantMessage("Old answer".into()));}
    let cfg=config(ws.0.clone(),url);let (tx,mut rx)=mpsc::unbounded_channel();let channel=Channel(Arc::new(move |event|{let _=tx.send(event);}));
    crate::core::titles::schedule(cfg,state.current_session.clone(),ws.0.clone(),channel).await;
    *state.current_session.lock().await=Some(Session::new("fixture".into()));
    task.await.unwrap();assert!(rx.recv().await.is_none());
    assert!(state.current_session.lock().await.as_ref().unwrap().title.is_none());
}

#[tokio::test]
async fn workspace_detection_is_read_only_and_requires_a_takiza_directory() {
    let ws = Workspace::new();
    let path = ws.0.to_string_lossy().to_string();
    assert!(!commands::workspace_has_takiza(path.clone()).await.unwrap());
    assert!(!ws.0.join(".takiza").exists());
    std::fs::write(ws.0.join(".takiza"), "not a directory").unwrap();
    assert!(!commands::workspace_has_takiza(path.clone()).await.unwrap());
    std::fs::remove_file(ws.0.join(".takiza")).unwrap();
    std::fs::create_dir(ws.0.join(".takiza")).unwrap();
    assert!(commands::workspace_has_takiza(path).await.unwrap());
    assert!(commands::workspace_has_takiza(ws.0.join("missing").to_string_lossy().into()).await.is_err());
}

#[tokio::test]
async fn stream_requires_completion_and_rejects_truncated_tool_calls() {
    let ws = Workspace::new();
    for (body, success) in [
        ("data: {\"choices\":[{\"delta\":{\"content\":\"ok\"},\"finish_reason\":\"stop\"}]}\n\n", true),
        ("data: {\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n\ndata: {\"choices\":[{\"finish_reason\":\"stop\"}]}\n\n", true),
        ("data: {\"choices\":[{\"delta\":{\"content\":\"partial\"}}]}\n\n", false),
        ("data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"id\":\"a\",\"function\":{\"name\":\"list_dir\",\"arguments\":\"{}\"}}]},\"finish_reason\":\"length\"}]}\n\n", false),
        ("data: {\"error\":{\"message\":\"provider failed\"}}\n\ndata: [DONE]\n\n", false),
    ] {
        let (url, _, task) = server(body.as_bytes().to_vec(), 1, false).await;
        let llm = LlmClient::new(config(ws.0.clone(), url));
        let (tx, _rx) = mpsc::channel(100);
        let result = llm.chat_step_stream(&[], &tx, &CancellationToken::new()).await;
        assert_eq!(result.is_ok(), success, "{body}: {result:?}");
        task.await.unwrap();
    }
}

#[tokio::test]
async fn completed_stream_does_not_wait_for_a_broken_http_body() {
    let ws = Workspace::new();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = [0; 8192];
        socket.read(&mut request).await.unwrap();
        // A proxy closes the HTTP body without its terminating chunk after the
        // provider has already sent a complete response.
        let body = "data: {\"choices\":[{\"delta\":{\"content\":\"done\"},\"finish_reason\":\"stop\"}]}\n\n";
        socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\n\r\n").await.unwrap();
        socket.write_all(format!("{:x}\r\n{}\r\n", body.len(), body).as_bytes()).await.unwrap();
    });
    let llm = LlmClient::new(config(ws.0.clone(), format!("http://{address}/v1")));
    let (tx, _rx) = mpsc::channel(100);
    let response = llm.chat_step_stream(&[], &tx, &CancellationToken::new()).await.unwrap();
    assert!(matches!(response, LlmResponse::Message(ref text) if text == "done"));
    task.await.unwrap();
}
