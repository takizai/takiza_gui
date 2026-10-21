use crate::core::agent::{Agent, AgentEvent, PermissionResponse};
use crate::core::config::{AppPreferences, Config};
use crate::core::git::GitInfo;
use crate::core::session::{HistoryItem, Session, SessionMeta};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::core::{checkpoints, attachments};
use tauri::ipc::Channel;
use tauri::State;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GitStatusDto {
    pub branch: Option<String>,
    pub is_dirty: bool,
    pub diff: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UsageStats {
    pub manual_used: u64,
    pub manual_limit: u64,
    pub manual_percentage: u32,
    pub moa_used: u64,
    pub moa_limit: u64,
    pub moa_percentage: u32,
    pub moa_saved: u64,
    pub active_mode: String,
    pub reset_time_utc: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileEntryDto {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
}

struct TurnGuard<'a>(&'a AtomicBool);
impl Drop for TurnGuard<'_> {
    fn drop(&mut self) { self.0.store(false, Ordering::Release); }
}
fn acquire_turn(state: &AppState) -> Result<TurnGuard<'_>, String> {
    state.turn_active.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .map_err(|_| "Another turn or workspace operation is still running".to_string())?;
    Ok(TurnGuard(&state.turn_active))
}
async fn stop_active_turn(state: &AppState) -> Result<(), String> {
    {
        let cancel = state.cancel_token.lock().await;
        if state.turn_active.load(Ordering::Acquire) { cancel.cancel(); }
    }
    tokio::time::timeout(std::time::Duration::from_secs(15), async {
        while state.turn_active.load(Ordering::Acquire) { tokio::time::sleep(std::time::Duration::from_millis(20)).await; }
    }).await.map_err(|_| "The previous operation has not stopped yet".to_string())
}
async fn read_config(state: &AppState) -> Config {
    if let Ok(agent) = state.agent.try_lock() { return agent.config.clone(); }
    Config::load(Some(state.workspace_dir.lock().await.clone()))
}

struct RegenerationRequest { session_id: String, history_index: usize, edited: Option<String> }

#[tauri::command]
pub async fn regenerate_agent_turn(session_id: String, history_index: usize, edited: Option<String>, channel: Channel<AgentEvent>, state: State<'_, AppState>) -> Result<(), String> {
    run_agent_turn(String::new(), channel, state, Some(RegenerationRequest { session_id, history_index, edited })).await
}

#[tauri::command]
pub async fn start_agent_turn(input: String, channel: Channel<AgentEvent>, state: State<'_, AppState>) -> Result<(), String> {
    run_agent_turn(input, channel, state, None).await
}

async fn run_agent_turn(mut input: String, channel: Channel<AgentEvent>, state: State<'_, AppState>, regeneration: Option<RegenerationRequest>) -> Result<(), String> {
    let cancel_token = CancellationToken::new();
    let _turn_guard = {
        let mut ct_lock = state.cancel_token.lock().await;
        let guard = acquire_turn(&state)?;
        *ct_lock = cancel_token.clone();
        guard
    };

    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(100);

    let permissions = state.permissions.clone();
    let questions = state.questions.clone();

    // Spawn agent turn in Tokio task
    let mut agent = state.agent.lock().await;

    // Account-backed modes are unavailable until Takiza sign-in ships.
    // Ignore legacy MoA preferences and keep the user's selected provider/model.
    agent.config.mode = Some("manual".to_string());


    // Ensure active session exists and is aligned with agent
    {
        let mut sess_lock = state.current_session.lock().await;
        if sess_lock.is_none() {
            let model = agent.config.model.clone();
            *sess_lock = Some(Session::new(model));
            let cfg = agent.config.clone();
            agent.reset(cfg);
        } else if let Some(ref mut sess) = *sess_lock {
            if sess.messages.is_empty() && agent.message_count() > 0 {
                let cfg = agent.config.clone();
                agent.reset(cfg);
            }
        }
    }

    let ws = state.workspace_dir.lock().await.clone();
    let checkpoint_session = state.current_session.lock().await.clone();
    let regenerated_session = if let Some(request) = regeneration {
        let session = checkpoint_session.as_ref().ok_or("Сессия не найдена")?;
        if session.id != request.session_id { return Err("Активная сессия изменилась".into()); }
        let revised = session.for_regeneration(request.history_index, request.edited.as_deref())?;
        input = match revised.history.last() { Some(HistoryItem::UserPrompt(text)) => text.clone(), _ => unreachable!() };
        Some(revised)
    } else { None };

    let direct_command = input.strip_prefix('!').map(|command| command.trim().to_string());
    let agent_fut = async {
        if let Some(command) = direct_command {
            agent.handle_direct_command(input.clone(), command, event_tx, cancel_token).await;
        } else {
            if let Some(mut session) = checkpoint_session {
                session.messages = agent.get_messages().to_vec();
                let workspace = ws.clone();
                let label = input.clone();
                let cancel = cancel_token.clone();
                let _ = event_tx.send(AgentEvent::StatusUpdate("Saving checkpoint...".into())).await;
                let capture = tokio::task::spawn_blocking(move || checkpoints::capture_session_cancellable(&workspace, &session, &label, cancel)).await;
                if cancel_token.is_cancelled() {
                    let _ = event_tx.send(AgentEvent::Interrupted).await;
                    let _ = event_tx.send(AgentEvent::Finished).await;
                    return;
                }
                let error = match capture { Ok(Ok(_)) => None, Ok(Err(error)) => Some(format!("{error:#}")), Err(error) => Some(error.to_string()) };
                if let Some(error) = error { let _ = event_tx.send(AgentEvent::ToolLog(format!("Checkpoint unavailable: {error}"))).await; }
            }
            if let Some(session) = regenerated_session {
                // The checkpoint above retains the original response and subsequent chat.
                agent.set_messages(session.messages.clone());
                *state.current_session.lock().await = Some(session);
                agent.regenerate_response(input.clone(), event_tx, cancel_token, permissions, questions).await;
            } else {
                agent.handle_user_input(input.clone(), event_tx, cancel_token, permissions, questions).await;
            }
        }
    };

    let channel_clone = channel.clone();
    let mut collected_history: Vec<HistoryItem> = Vec::new();

    // Event forwarder loop
    let forwarder = tokio::spawn(async move {
        let mut partial_text = String::new();
        let mut partial_thought = String::new();
        let mut asked_questions = Vec::new();
        while let Some(event) = event_rx.recv().await {
            // Reasoning belongs before the answer, including responses without tools.
            if matches!(&event, AgentEvent::AssistantMessage(_) | AgentEvent::ToolStart { .. } | AgentEvent::Interrupted | AgentEvent::Finished | AgentEvent::Error(_)) {
                if !partial_thought.trim().is_empty() { collected_history.push(HistoryItem::Thought(std::mem::take(&mut partial_thought))); }
            }
            if matches!(&event, AgentEvent::ToolStart { .. } | AgentEvent::Interrupted | AgentEvent::Finished | AgentEvent::Error(_)) {
                if !partial_text.trim().is_empty() { collected_history.push(HistoryItem::AssistantMessage(std::mem::take(&mut partial_text))); }
            }
            match &event {
                AgentEvent::AssistantToken(text) => partial_text.push_str(text),
                AgentEvent::ThoughtToken(text) => partial_thought.push_str(text),
                AgentEvent::UserMessage(u) => collected_history.push(HistoryItem::UserPrompt(u.clone())),
                AgentEvent::MoaRouting { model, category, complexity, source } => collected_history.push(HistoryItem::MoaRouting {
                    model: model.clone(),
                    category: category.clone(),
                    complexity: complexity.clone(),
                    source: source.clone(),
                }),
                AgentEvent::AssistantThought(t) => { partial_thought.clear(); collected_history.push(HistoryItem::Thought(t.clone())); },
                AgentEvent::QuestionRequest { questions, .. } => asked_questions = questions.clone(),
                AgentEvent::QuestionAnswered { answers, skipped, .. } => collected_history.push(HistoryItem::QuestionInteraction {
                    questions: std::mem::take(&mut asked_questions),
                    answers: answers.clone(),
                    skipped: *skipped,
                }),
                AgentEvent::ToolEnd { name, args, result, is_error, .. } => collected_history.push(HistoryItem::ToolEnd {
                    name: name.clone(),
                    args: args.clone(),
                    result: result.clone(),
                    is_error: *is_error,
                }),
                AgentEvent::AssistantMessage(m) => { partial_text.clear(); collected_history.push(HistoryItem::AssistantMessage(m.clone())); },
                AgentEvent::ToolLog(text) => collected_history.push(HistoryItem::ToolLog(text.clone())),
                AgentEvent::Interrupted => collected_history.push(HistoryItem::ToolLog("⏸ Stopped by user".into())),
                AgentEvent::Error(e) => collected_history.push(HistoryItem::Error(e.clone())),
                _ => {}
            }

            let _ = channel_clone.send(event);
        }
        collected_history
    });

    agent_fut.await;
    let new_history = forwarder.await.unwrap_or_default();

    // Sync agent messages back into active session and save
    {
        let mut sess_lock = state.current_session.lock().await;
        if let Some(ref mut sess) = *sess_lock {
            sess.messages = agent.get_messages().to_vec();
            sess.history.extend(new_history);
            sess.save(&ws).map_err(|error| format!("Cannot save session: {error}"))?;
        }
    }

    if !input.starts_with('!') && !state.cancel_token.lock().await.is_cancelled() {
        crate::core::titles::schedule(agent.config.clone(), state.current_session.clone(), ws, channel).await;
    }

    Ok(())
}

#[tauri::command]
pub async fn cancel_agent(state: State<'_, AppState>) -> Result<(), String> {
    let ct = state.cancel_token.lock().await;
    ct.cancel();
    Ok(())
}

#[tauri::command]
pub async fn respond_permission(
    id: String,
    decision: PermissionResponse,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut lock = state.permissions.lock().await;
    if let Some(sender) = lock.remove(&id) {
        let _ = sender.send(decision);
        Ok(())
    } else {
        Err(format!("No pending permission request found for id: {}", id))
    }
}

#[tauri::command]
pub async fn respond_question(
    id: String,
    response: crate::core::agent::QuestionResponse,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut lock = state.questions.lock().await;
    if let Some(sender) = lock.remove(&id) {
        let _ = sender.send(response);
        Ok(())
    } else {
        Err(format!("No pending question request found for id: {}", id))
    }
}

#[tauri::command]
pub async fn get_available_skills(
    state: State<'_, AppState>,
) -> Result<Vec<crate::core::skills::Skill>, String> {
    let ws = state.workspace_dir.lock().await.clone();
    Ok(crate::core::skills::discover_skills(&ws))
}

#[tauri::command]
pub async fn read_skill_content(
    name: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let ws = state.workspace_dir.lock().await.clone();
    crate::core::skills::get_skill_content(&ws, &name)
        .ok_or_else(|| format!("Skill '{}' not found in workspace or system skills directories", name))
}

#[tauri::command]
pub async fn get_current_session(state: State<'_, AppState>) -> Result<Option<Session>, String> {
    let lock = state.current_session.lock().await;
    Ok(lock.clone())
}

#[tauri::command]
pub async fn get_sessions(state: State<'_, AppState>) -> Result<Vec<SessionMeta>, String> {
    let ws = state.workspace_dir.lock().await.clone();
    Ok(Session::list_meta(&ws))
}

#[tauri::command]
pub async fn load_session(id: String, state: State<'_, AppState>) -> Result<Session, String> {
    stop_active_turn(&state).await?;
    let _operation = acquire_turn(&state)?;

    let ws = state.workspace_dir.lock().await.clone();
    let session = Session::load(&ws, &id)
        .ok_or_else(|| format!("Session not found: {}", id))?;

    {
        let mut agent = state.agent.lock().await;
        let cfg = agent.config.clone();
        agent.reset(cfg);
        if !session.messages.is_empty() {
            agent.set_messages(session.messages.clone());
        }
    }

    {
        let mut cur = state.current_session.lock().await;
        *cur = Some(session.clone());
    }

    Ok(session)
}

#[tauri::command]
pub async fn new_session(state: State<'_, AppState>) -> Result<Session, String> {
    stop_active_turn(&state).await?;
    let _operation = acquire_turn(&state)?;

    let model = {
        let agent = state.agent.lock().await;
        agent.config.model.clone()
    };

    let session = Session::new(model);
    // Don't save empty session to disk yet! Only save once messages exist.

    {
        let mut agent = state.agent.lock().await;
        let cfg = agent.config.clone();
        agent.reset(cfg);
    }

    {
        let mut cur = state.current_session.lock().await;
        *cur = Some(session.clone());
    }

    Ok(session)
}

#[tauri::command]
pub async fn delete_session(id: String, state: State<'_, AppState>) -> Result<Option<Session>, String> {
    stop_active_turn(&state).await?;
    let _operation = acquire_turn(&state)?;

    let ws = state.workspace_dir.lock().await.clone();
    Session::delete(&ws, &id).map_err(|e| e.to_string())?;

    let mut cur = state.current_session.lock().await;
    let mut next_session = None;
    let mut was_current = false;

    if let Some(ref s) = *cur {
        if s.id == id {
            was_current = true;
        }
    }

    if was_current {
        // Select next available session if one exists
        let metas = Session::list_meta(&ws);
        if let Some(first_meta) = metas.first() {
            if let Some(loaded) = Session::load(&ws, &first_meta.id) {
                *cur = Some(loaded.clone());
                next_session = Some(loaded);
            } else {
                *cur = None;
            }
        } else {
            *cur = None;
        }

        let mut agent = state.agent.lock().await;
        let cfg = agent.config.clone();
        agent.reset(cfg);
        if let Some(ref sess) = next_session {
            if !sess.messages.is_empty() {
                agent.set_messages(sess.messages.clone());
            }
        }
    }

    Ok(next_session)
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<Config, String> {
    let mut config = read_config(&state).await;
    config.mode = Some("manual".to_string());
    Ok(config)
}

/// Query the selected connection instead of offering another provider's model IDs.
#[tauri::command]
pub async fn list_provider_models(
    base_url: String,
    api_key: String,
    proxy: Option<String>,
    api_type: Option<crate::core::providers::ApiType>,
    headers: Option<std::collections::BTreeMap<String, String>>,
    api_key_env: Option<String>,
) -> Result<Vec<serde_json::Value>, String> {
    use crate::core::providers::{ApiType, authorize, validate_headers};
    use std::collections::BTreeMap;
    let api_type = api_type.unwrap_or_default();
    let headers = headers.unwrap_or_default();
    validate_headers(&headers)?;
    let api_key = match api_key_env.filter(|name| !name.trim().is_empty()) {
        Some(name) => std::env::var(name).map_err(|_| "Переменная окружения с ключом не задана.")?,
        None => api_key,
    };
    let endpoint = format!("{}/models", crate::core::config::normalize_base_url(&base_url));
    let base = reqwest::Url::parse(&endpoint).map_err(|_| "Некорректный URL провайдера".to_string())?;
    if !matches!(base.scheme(), "http" | "https") { return Err("Нужен HTTP или HTTPS URL".into()); }
    let mut builder = reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).timeout(std::time::Duration::from_secs(15));
    if let Some(value) = proxy.filter(|value| !value.trim().is_empty()) {
        builder = builder.proxy(reqwest::Proxy::all(value).map_err(|_| "Некорректный прокси".to_string())?);
    }
    let client = builder.build().map_err(|_| "Не удалось создать подключение".to_string())?;
    let mut catalog = BTreeMap::new();
    let mut after: Option<String> = None;
    for _ in 0..100 {
        let mut url = base.clone();
        if api_type == ApiType::AnthropicMessages {
            url.query_pairs_mut().append_pair("limit", "1000");
            if let Some(cursor) = &after { url.query_pairs_mut().append_pair("after_id", cursor); }
        }
        let response = authorize(client.get(url), api_type, &api_key, &headers).send().await
            .map_err(|_| "Не удалось получить модели. Проверьте подключение и прокси.".to_string())?;
        if !response.status().is_success() { return Err(format!("Провайдер вернул HTTP {}. Проверьте авторизацию и URL.", response.status().as_u16())); }
        let body: serde_json::Value = response.json().await.map_err(|_| "Провайдер вернул некорректный список моделей".to_string())?;
        let data = body.get("data").or_else(|| body.get("models")).and_then(|data| data.as_array())
            .ok_or_else(|| "Провайдер не поддерживает список моделей. Укажите ID вручную.".to_string())?;
        for item in data {
            if let Some(id) = item.get("id").and_then(|id| id.as_str()).filter(|id| !id.is_empty()) {
                let mut model = item.clone();
                if let Some(name) = item.get("display_name").and_then(serde_json::Value::as_str) { model["name"] = serde_json::json!(name); }
                catalog.insert(id.to_string(), model);
            }
        }
        if body["has_more"] != true { return Ok(catalog.into_values().collect()); }
        let cursor = body["last_id"].as_str().ok_or("Провайдер не вернул курсор списка моделей.")?.to_string();
        if after.as_ref() == Some(&cursor) { return Err("Провайдер повторил курсор списка моделей.".into()); }
        after = Some(cursor);
    }
    Err("Список моделей превысил 100 страниц.".into())
}

#[tauri::command]
pub async fn save_preferences(
    prefs: AppPreferences,
    state: State<'_, AppState>,
) -> Result<Config, String> {
    if prefs.mode.as_deref() == Some("moa") {
        return Err("MoA в разработке и будет доступен после появления входа в аккаунт Takiza".into());
    }
    stop_active_turn(&state).await?;
    let _operation = acquire_turn(&state)?;

    Config::save_preferences(&prefs).map_err(|e| e.to_string())?;

    let ws = state.workspace_dir.lock().await.clone();
    let new_cfg = Config::load(Some(ws));

    {
        let mut agent = state.agent.lock().await;
        let msgs = agent.get_messages().to_vec();
        *agent = Agent::new(new_cfg.clone());
        agent.set_messages(msgs);
    }

    {
        let mut cur_sess = state.current_session.lock().await;
        if let Some(ref mut sess) = *cur_sess {
            sess.model = new_cfg.model.clone();
        }
    }

    Ok(new_cfg)
}

#[tauri::command]
pub async fn set_effort(effort: String, state: State<'_, AppState>) -> Result<Config, String> {
    let cfg = read_config(&state).await;
    if !effort.is_empty() && !cfg.reasoning_levels().contains(&effort) {
        return Err(format!("Reasoning effort '{}' is unavailable for {}", effort, cfg.model));
    }
    stop_active_turn(&state).await?;
    let _operation = acquire_turn(&state)?;

    let prefs = AppPreferences { effort: Some(effort), ..Default::default() };
    Config::save_preferences(&prefs).map_err(|e| e.to_string())?;

    let ws = state.workspace_dir.lock().await.clone();
    let new_cfg = Config::load(Some(ws));

    {
        let mut agent = state.agent.lock().await;
        let msgs = agent.get_messages().to_vec();
        *agent = Agent::new(new_cfg.clone());
        agent.set_messages(msgs);
    }

    Ok(new_cfg)
}

#[tauri::command]
pub async fn set_mode(mode: String, state: State<'_, AppState>) -> Result<Config, String> {
    if mode.trim().eq_ignore_ascii_case("moa") {
        return Err("MoA в разработке и будет доступен после появления входа в аккаунт Takiza".into());
    }
    stop_active_turn(&state).await?;
    let _operation = acquire_turn(&state)?;

    let clean = mode.trim().to_lowercase();
    let valid_mode = match clean.as_str() {
        "manual" | "moa" => clean,
        _ => return Err(format!("Invalid mode '{}'. Must be 'manual' or 'moa'", mode)),
    };

    let prefs = AppPreferences { mode: Some(valid_mode), ..Default::default() };
    Config::save_preferences(&prefs).map_err(|e| e.to_string())?;

    let ws = state.workspace_dir.lock().await.clone();
    let new_cfg = Config::load(Some(ws));

    {
        let mut agent = state.agent.lock().await;
        let msgs = agent.get_messages().to_vec();
        *agent = Agent::new(new_cfg.clone());
        agent.set_messages(msgs);
    }

    Ok(new_cfg)
}

#[tauri::command]
pub async fn set_language(language: String, state: State<'_, AppState>) -> Result<Config, String> {
    if !matches!(language.as_str(), "ru" | "en" | "zh") { return Err("Unsupported interface language".into()); }
    Config::save_preferences(&AppPreferences { language: Some(language.clone()), ..Default::default() }).map_err(|error| error.to_string())?;
    if let Ok(mut agent) = state.agent.try_lock() { agent.config.language = Some(language); return Ok(agent.config.clone()); }
    Ok(Config::load(Some(state.workspace_dir.lock().await.clone())))
}

#[tauri::command]
pub async fn set_theme(theme: String, state: State<'_, AppState>) -> Result<Config, String> {
    stop_active_turn(&state).await?;
    let _operation = acquire_turn(&state)?;

    let clean = crate::core::config::normalize_theme(&theme);
    let valid_theme = match clean.as_str() {
        "amber" | "cyberpunk" | "emerald" | "nord" | "monochrome" | "dracula" | "rose" | "ocean" | "sakura" | "solarized" => clean,
        _ => return Err(format!("Invalid theme '{}'. Unknown theme", theme)),
    };

    Config::save_preferences(&AppPreferences { theme: Some(valid_theme.clone()), ..Default::default() })
        .map_err(|e| e.to_string())?;
    // Appearance does not require rebuilding the LLM client or its settings.
    let mut agent = state.agent.lock().await;
    agent.config.theme = Some(valid_theme);
    Ok(agent.config.clone())
}

#[tauri::command]
pub async fn get_usage() -> Result<UsageStats, String> {
    Err("Usage Takiza в разработке. Расход и лимиты доступны в кабинете вашего провайдера.".into())
}

#[tauri::command]
pub async fn get_git_info(state: State<'_, AppState>) -> Result<GitStatusDto, String> {
    let ws = state.workspace_dir.lock().await.clone();
    let info = GitInfo::get(&ws);
    let diff = if info.is_dirty {
        GitInfo::diff(&ws)
    } else {
        None
    };
    Ok(GitStatusDto {
        branch: info.branch,
        is_dirty: info.is_dirty,
        diff,
    })
}

#[tauri::command]
pub async fn list_files(
    subpath: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<FileEntryDto>, String> {
    let ws = state.workspace_dir.lock().await.clone();
    let target = match subpath {
        Some(p) => ws.join(p),
        None => ws.clone(),
    };

    if !target.exists() || !target.is_dir() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    if let Ok(dir_entries) = fs::read_dir(&target) {
        for entry in dir_entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }
            let path = entry.path();
            let is_dir = path.is_dir();
            let size = if is_dir { None } else { entry.metadata().ok().map(|m| m.len()) };

            let rel_path = path.strip_prefix(&ws)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            entries.push(FileEntryDto {
                name,
                path: rel_path,
                is_dir,
                size,
            });
        }
    }

    // Sort dirs first, then alphabetically
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(entries)
}

#[tauri::command]
pub async fn get_file_content(path: String, state: State<'_, AppState>) -> Result<String, String> {
    let ws = state.workspace_dir.lock().await.clone();
    let full_path = ws.join(path);
    fs::read_to_string(full_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn workspace_has_takiza(path: String) -> Result<bool, String> {
    let directory = std::path::PathBuf::from(&path);
    if !directory.is_dir() { return Err(format!("Указанный путь не является директорией: {}", path)); }
    match fs::metadata(directory.join(".takiza")) {
        Ok(metadata) => Ok(metadata.is_dir()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.to_string()),
    }
}

#[tauri::command]
pub async fn set_workspace_dir(
    path: String,
    state: State<'_, AppState>,
) -> Result<Config, String> {
    stop_active_turn(&state).await?;
    let _operation = acquire_turn(&state)?;

    let raw_path = if path.starts_with("~/") || path.starts_with("~\\") {
        if let Some(home) = crate::core::config::dirs_fallback() {
            home.join(&path[2..])
        } else {
            std::path::PathBuf::from(&path)
        }
    } else {
        std::path::PathBuf::from(&path)
    };

    if !raw_path.exists() {
        return Err(format!("Каталог не найден: {}", path));
    }
    if !raw_path.is_dir() {
        return Err(format!("Указанный путь не является директорией: {}", path));
    }

    let abs_ws = raw_path.canonicalize().unwrap_or(raw_path);

    // Save as last workspace dir
    let prefs = AppPreferences { last_workspace_dir: Some(abs_ws.clone()), ..Default::default() };
    let _ = Config::save_preferences(&prefs);

    let new_cfg = Config::load(Some(abs_ws.clone()));

    {
        let mut ws_lock = state.workspace_dir.lock().await;
        *ws_lock = abs_ws.clone();
    }

    let session = Session::latest(&abs_ws).unwrap_or_else(|| Session::new(new_cfg.model.clone()));
    {
        let mut agent = state.agent.lock().await;
        *agent = Agent::new(new_cfg.clone());
        if !session.messages.is_empty() { agent.set_messages(session.messages.clone()); }
    }
    *state.current_session.lock().await = Some(session);

    Ok(new_cfg)
}

#[tauri::command]
pub async fn reset_to_harness_defaults(
    state: State<'_, AppState>,
) -> Result<Config, String> {
    stop_active_turn(&state).await?;
    let _operation = acquire_turn(&state)?;

    let ws = state.workspace_dir.lock().await.clone();

    // Reset override fields in config.json
    let mut prefs = Config::load_preferences();
    prefs.api_key = None;
    prefs.base_url = None;
    prefs.model = None;
    prefs.proxy = None;
    let _ = Config::save_preferences(&prefs);

    let new_cfg = Config::load(Some(ws));

    {
        let mut agent = state.agent.lock().await;
        let msgs = agent.get_messages().to_vec();
        *agent = Agent::new(new_cfg.clone());
        agent.set_messages(msgs);
    }

    Ok(new_cfg)
}

#[tauri::command]
pub async fn minimize_window(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_maximize_window(window: tauri::Window) -> Result<bool, String> {
    if window.is_maximized().unwrap_or(false) {
        window.unmaximize().map_err(|e| e.to_string())?;
        Ok(false)
    } else {
        window.maximize().map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
pub async fn close_window(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn is_window_maximized(window: tauri::Window) -> Result<bool, String> {
    Ok(window.is_maximized().unwrap_or(false))
}

#[tauri::command]
pub fn start_dragging_window(window: tauri::Window) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_webview_zoom(scale_factor: f64, window: tauri::WebviewWindow) -> Result<(), String> {
    window.set_zoom(scale_factor).map_err(|e| e.to_string())
}


#[derive(Serialize)]
pub struct CheckpointSummary { id: String, created_at: String, label: String }

#[tauri::command]
pub async fn list_checkpoints(state: State<'_, AppState>) -> Result<Vec<CheckpointSummary>, String> {
    let workspace = state.workspace_dir.lock().await.clone();
    let session = state.current_session.lock().await.clone().ok_or("No active chat")?;
    tokio::task::spawn_blocking(move || checkpoints::list(&workspace, &session.id)
        .map(|points| points.into_iter().map(|point| CheckpointSummary {
            id: point.id, created_at: point.created_at, label: point.label }).collect::<Vec<_>>())
        .map_err(|error| error.to_string())).await.map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn preview_restore(id: String, state: State<'_, AppState>) -> Result<Vec<String>, String> {
    stop_active_turn(&state).await?;
    let workspace = state.workspace_dir.lock().await.clone();
    let session = state.current_session.lock().await.clone().ok_or("No active chat")?;
    tokio::task::spawn_blocking(move || {
        let point = checkpoints::list(&workspace, &session.id).map_err(|error| error.to_string())?
            .into_iter().find(|point| point.id == id).ok_or("Checkpoint not found in this chat")?;
        checkpoints::preview(&workspace, &point).map_err(|error| error.to_string())
    }).await.map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn restore_checkpoint(id: String, state: State<'_, AppState>) -> Result<Session, String> {
    stop_active_turn(&state).await?;
    let _operation = acquire_turn(&state)?;
    let mut agent = state.agent.lock().await;
    let workspace = state.workspace_dir.lock().await.clone();
    let mut session = state.current_session.lock().await.clone().ok_or("No active chat")?;
    session.messages = agent.get_messages().to_vec();
    let restored = tokio::task::spawn_blocking(move || {
        let point = checkpoints::list(&workspace, &session.id).map_err(|error| error.to_string())?
            .into_iter().find(|point| point.id == id).ok_or("Checkpoint not found in this chat")?;
        let mut restored = checkpoints::conversation_at(&point, &session).map_err(|error| error.to_string())?;
        checkpoints::restore_session(&workspace, &point, &session).map_err(|error| error.to_string())?;
        restored.history = restored.checkpoint_history();
        restored.save(&workspace).map_err(|error| error.to_string())?;
        Ok::<_, String>(restored)
    }).await.map_err(|error| error.to_string())??;
    let config = agent.config.clone();
    agent.reset(config);
    agent.set_messages(restored.messages.clone());
    *state.current_session.lock().await = Some(restored.clone());
    Ok(restored)
}

#[derive(Serialize)]
pub struct ImageAttachment { marker: String, url: String, name: String }
#[derive(Serialize)]
pub struct AttachmentImport { text: String, images: Vec<ImageAttachment> }

fn attachment_import(text: String) -> Result<AttachmentImport, String> {
    use base64::Engine;
    let mut images = Vec::new();
    let mut remaining = text.as_str();
    let mut plain = String::new();
    while let Some(start) = remaining.find("[Image: ") {
        plain.push_str(&remaining[..start]);
        let after = &remaining[start + "[Image: ".len()..];
        let mut stream = serde_json::Deserializer::from_str(after).into_iter::<String>();
        let path = stream.next().ok_or("Missing image path")?.map_err(|error| error.to_string())?;
        let offset = stream.byte_offset();
        if !after[offset..].starts_with(']') { return Err("Invalid image attachment".into()); }
        let bytes = fs::read(&path).map_err(|error| error.to_string())?;
        let (mime, _) = attachments::image_type(&bytes).ok_or("Unsupported image attachment")?;
        let name = std::path::Path::new(&path).file_name().unwrap_or_default().to_string_lossy().to_string();
        images.push(ImageAttachment { marker: format!("[Image: {}]", serde_json::to_string(&path).unwrap()),
            url: format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)), name });
        remaining = &after[offset + 1..];
    }
    plain.push_str(remaining);
    Ok(AttachmentImport { text: plain, images })
}

#[tauri::command]
pub async fn import_image(bytes: Vec<u8>, state: State<'_, AppState>) -> Result<AttachmentImport, String> {
    if bytes.len() > 10 * 1024 * 1024 { return Err("Image attachment exceeds 10 MiB.".into()); }
    let workspace = state.workspace_dir.lock().await.clone();
    tokio::task::spawn_blocking(move || attachment_import(attachments::cache_image(&bytes, &workspace)?))
        .await.map_err(|error| error.to_string())?
}

#[derive(Serialize)]
pub struct FileAttachment { marker: String, name: String, size: usize }

#[tauri::command]
pub async fn import_text_file(name: String, bytes: Vec<u8>, state: State<'_, AppState>) -> Result<FileAttachment, String> {
    let workspace = state.workspace_dir.lock().await.clone();
    tokio::task::spawn_blocking(move || {
        let marker = attachments::cache_text_file(&name, &bytes, &workspace)?;
        Ok(FileAttachment { marker, name, size: bytes.len() })
    }).await.map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn import_paths(paths: Vec<String>, state: State<'_, AppState>) -> Result<AttachmentImport, String> {
    let workspace = state.workspace_dir.lock().await.clone();
    tokio::task::spawn_blocking(move || attachment_import(attachments::file_list(&paths.join("\n"), &workspace)?))
        .await.map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn paste_attachments(text: Option<String>, state: State<'_, AppState>) -> Result<AttachmentImport, String> {
    let workspace = state.workspace_dir.lock().await.clone();
    tokio::task::spawn_blocking(move || {
        let pasted = if let Some(text) = text { attachments::pasted_text(&text, &workspace)? }
            else { crate::core::clipboard::paste(&workspace)? };
        attachment_import(pasted)
    }).await.map_err(|error| error.to_string())?
}
