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

    // Ensure session messages are aligned with agent
    {
        let mut sess_lock = state.current_session.lock().await;
        if let Some(ref mut sess) = *sess_lock {
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
                AgentEvent::AssistantThought(t) => collected_history.push(HistoryItem::Thought(t.clone())),
                AgentEvent::ToolStart { name, args, .. } => collected_history.push(HistoryItem::ToolStart {
                    name: name.clone(),
                    args: args.clone(),
                }),
                AgentEvent::ToolLog(l) => collected_history.push(HistoryItem::ToolLog(l.clone())),
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
    let ws = state.workspace_dir.lock().await.clone();
    let model = {
        let agent = state.agent.lock().await;
        agent.config.model.clone()
    };

    let session = Session::new(model);
    let _ = session.save(&ws);

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
pub async fn delete_session(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let ws = state.workspace_dir.lock().await.clone();
    Session::delete(&ws, &id).map_err(|e| e.to_string())?;

    let mut cur = state.current_session.lock().await;
    if let Some(ref s) = *cur {
        if s.id == id {
            *cur = None;
        }
    }
    Ok(())
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

    Ok(new_cfg)
}

#[tauri::command]
pub async fn get_git_info(state: State<'_, AppState>) -> Result<GitStatusDto, String> {
    let ws = state.workspace_dir.lock().await.clone();
    let info = GitInfo::get(&ws);
    let diff = GitInfo::diff(&ws);
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
