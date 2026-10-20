//! Generate a conversation name without holding up the next chat turn.
use super::{
    agent::AgentEvent,
    config::Config,
    llm::LlmClient,
    session::{HistoryItem, Session},
};
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
};
use tauri::ipc::Channel;
use tokio::sync::Mutex as AsyncMutex;

static PENDING: OnceLock<Mutex<HashSet<(PathBuf, String)>>> = OnceLock::new();
struct Pending((PathBuf, String));
impl Drop for Pending {
    fn drop(&mut self) {
        PENDING.get().unwrap().lock().unwrap().remove(&self.0);
    }
}

pub async fn schedule(
    config: Config,
    active: Arc<AsyncMutex<Option<Session>>>,
    workspace: PathBuf,
    channel: Channel<AgentEvent>,
) {
    let candidate = {
        let guard = active.lock().await;
        guard
            .as_ref()
            .filter(|session| {
                session
                    .title
                    .as_ref()
                    .is_none_or(|title| title.trim().is_empty())
            })
            .filter(|session| {
                session
                    .history
                    .iter()
                    .any(|item| matches!(item, HistoryItem::AssistantMessage(_)))
            })
            .and_then(|session| {
                session
                    .messages
                    .iter()
                    .find(|message| message.role == "user")
                    .and_then(|message| message.content.as_ref())
                    .map(|prompt| (session.id.clone(), prompt.clone()))
            })
    };
    let Some((id, prompt)) = candidate.filter(|_| !config.model.is_empty()) else {
        return;
    };
    let key = (workspace.clone(), id.clone());
    if !PENDING
        .get_or_init(Default::default)
        .lock()
        .unwrap()
        .insert(key.clone())
    {
        return;
    }
    let pending = Pending(key);
    tokio::spawn(async move {
        let _pending = pending;
        let client = LlmClient::new(config);
        let Ok(Some(title)) = tokio::time::timeout(
            std::time::Duration::from_secs(90),
            client.generate_title(&prompt),
        )
        .await
        else {
            return;
        };
        let mut guard = active.lock().await;
        // Like the CLI, do not apply a delayed name after changing conversations.
        let Some(session) = guard.as_mut().filter(|session| {
            session.id == id
                && session
                    .title
                    .as_ref()
                    .is_none_or(|title| title.trim().is_empty())
        }) else {
            return;
        };
        session.title = Some(title.clone());
        if session.save(&workspace).is_err() {
            session.title = None;
            return;
        }
        let _ = channel.send(AgentEvent::SessionTitle {
            session_id: id,
            title,
        });
    });
}
