use crate::core::agent::{Agent, AgentEvent, PermissionResponse};
use crate::core::config::{AppPreferences, Config};
use crate::core::git::GitInfo;
use crate::core::session::{HistoryItem, Session, SessionMeta};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::fs;
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

#[tauri::command]
pub async fn start_agent_turn(
    input: String,
    channel: Channel<AgentEvent>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let cancel_token = CancellationToken::new();
    {
        let mut ct_lock = state.cancel_token.lock().await;
        *ct_lock = cancel_token.clone();
    }

    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(100);

    let permissions = state.permissions.clone();

    // Spawn agent turn in Tokio task
    let mut agent = state.agent.lock().await;

    // Check MoA mode routing
    if agent.config.mode.as_deref() == Some("moa") {
        let route = crate::core::moa_router::resolve_moa_route(&input, None).await;
        agent.set_model(route.selected_model.clone());

        let moa_ev = AgentEvent::MoaRouting {
            model: route.selected_model.clone(),
            category: route.category.clone(),
            complexity: route.complexity.clone(),
            source: route.source.clone(),
        };
        let _ = event_tx.send(moa_ev).await;

        let status_msg = format!(
            "⚡ Выбрана модель: {} ({} • {})",
            route.selected_model, route.category, route.complexity
        );
        let _ = event_tx.send(AgentEvent::StatusUpdate(status_msg)).await;
    }

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

    let agent_fut = agent.handle_user_input(input.clone(), event_tx, cancel_token, permissions);

    let channel_clone = channel.clone();
    let mut collected_history: Vec<HistoryItem> = Vec::new();

    // Event forwarder loop
    let forwarder = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match &event {
                AgentEvent::UserMessage(u) => collected_history.push(HistoryItem::UserPrompt(u.clone())),
                AgentEvent::MoaRouting { model, category, complexity, source } => collected_history.push(HistoryItem::MoaRouting {
                    model: model.clone(),
                    category: category.clone(),
                    complexity: complexity.clone(),
                    source: source.clone(),
                }),
                AgentEvent::AssistantThought(t) => collected_history.push(HistoryItem::Thought(t.clone())),
                AgentEvent::ToolEnd { name, args, result, is_error, .. } => collected_history.push(HistoryItem::ToolEnd {
                    name: name.clone(),
                    args: args.clone(),
                    result: result.clone(),
                    is_error: *is_error,
                }),
                AgentEvent::AssistantMessage(m) => collected_history.push(HistoryItem::AssistantMessage(m.clone())),
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
    let ws = { state.workspace_dir.lock().await.clone() };
    {
        let mut sess_lock = state.current_session.lock().await;
        if let Some(ref mut sess) = *sess_lock {
            sess.messages = agent.get_messages().to_vec();
            sess.history.extend(new_history);
            let _ = sess.save(&ws);
        }
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
    let agent = state.agent.lock().await;
    Ok(agent.config.clone())
}

#[tauri::command]
pub async fn save_preferences(
    prefs: AppPreferences,
    state: State<'_, AppState>,
) -> Result<Config, String> {
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
    let mut prefs = Config::load_preferences();
    prefs.effort = Some(effort);
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
    let clean = mode.trim().to_lowercase();
    let valid_mode = match clean.as_str() {
        "manual" | "moa" => clean,
        _ => return Err(format!("Invalid mode '{}'. Must be 'manual' or 'moa'", mode)),
    };

    let mut prefs = Config::load_preferences();
    prefs.mode = Some(valid_mode);
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
pub async fn set_theme(theme: String, state: State<'_, AppState>) -> Result<Config, String> {
    let clean = theme.trim().to_lowercase();
    let valid_theme = match clean.as_str() {
        "amber" | "cyberpunk" | "emerald" | "nord" | "monochrome" => clean,
        _ => return Err(format!("Invalid theme '{}'. Must be amber, cyberpunk, emerald, nord, or monochrome", theme)),
    };

    let mut prefs = Config::load_preferences();
    prefs.theme = Some(valid_theme);
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
pub async fn get_usage(state: State<'_, AppState>) -> Result<UsageStats, String> {
    let mode = {
        let agent = state.agent.lock().await;
        agent.config.mode.clone().unwrap_or_else(|| "manual".to_string())
    };

    let manual_limit = 1_000_000u64;
    let manual_used = 520_000u64;
    let manual_percentage = ((manual_used as f64 / manual_limit as f64) * 100.0) as u32;

    let moa_limit = 1_000_000u64;
    let moa_used = 210_000u64;
    let moa_percentage = ((moa_used as f64 / moa_limit as f64) * 100.0) as u32;
    let moa_saved = 172_000u64;

    Ok(UsageStats {
        manual_used,
        manual_limit,
        manual_percentage,
        moa_used,
        moa_limit,
        moa_percentage,
        moa_saved,
        active_mode: mode,
        reset_time_utc: "00:00 UTC".to_string(),
    })
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
pub async fn set_workspace_dir(
    path: String,
    state: State<'_, AppState>,
) -> Result<Config, String> {
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
    let mut prefs = Config::load_preferences();
    prefs.last_workspace_dir = Some(abs_ws.clone());
    let _ = Config::save_preferences(&prefs);

    let new_cfg = Config::load(Some(abs_ws.clone()));

    {
        let mut ws_lock = state.workspace_dir.lock().await;
        *ws_lock = abs_ws.clone();
    }

    {
        let mut agent = state.agent.lock().await;
        let msgs = agent.get_messages().to_vec();
        *agent = Agent::new(new_cfg.clone());
        agent.set_messages(msgs);
    }

    {
        let mut cur_sess = state.current_session.lock().await;
        *cur_sess = Session::latest(&abs_ws).or_else(|| {
            Some(Session::new(new_cfg.model.clone()))
        });
    }

    Ok(new_cfg)
}

#[tauri::command]
pub async fn reset_to_harness_defaults(
    state: State<'_, AppState>,
) -> Result<Config, String> {
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

