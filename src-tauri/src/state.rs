use crate::core::agent::{Agent, PermissionRegistry};
use crate::core::config::Config;
use crate::core::session::Session;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub struct AppState {
    pub agent: Mutex<Agent>,
    pub current_session: Mutex<Option<Session>>,
    pub cancel_token: Mutex<CancellationToken>,
    pub permissions: PermissionRegistry,
    pub workspace_dir: Mutex<PathBuf>,
}

impl AppState {
    pub fn new(workspace_dir: Option<PathBuf>) -> Self {
        let config = Config::load(workspace_dir.clone());
        let ws = config.workspace_dir.clone();
        let agent = Agent::new(config.clone());
        let permissions = Arc::new(Mutex::new(HashMap::new()));
        let cancel_token = Mutex::new(CancellationToken::new());

        // Try load latest session or create new
        let current_session = Mutex::new(Session::latest(&ws).or_else(|| {
            Some(Session::new(config.model.clone()))
        }));

        Self {
            agent: Mutex::new(agent),
            current_session,
            cancel_token,
            permissions,
            workspace_dir: Mutex::new(ws),
        }
    }
}
