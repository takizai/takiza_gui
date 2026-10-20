use crate::core::agent::{Agent, PermissionRegistry, QuestionRegistry};
use crate::core::config::Config;
use crate::core::session::Session;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, atomic::AtomicBool};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub struct AppState {
    pub turn_active: AtomicBool,
    pub agent: Mutex<Agent>,
    pub current_session: Arc<Mutex<Option<Session>>>,
    pub cancel_token: Mutex<CancellationToken>,
    pub permissions: PermissionRegistry,
    pub questions: QuestionRegistry,
    pub workspace_dir: Mutex<PathBuf>,
}

impl AppState {
    pub fn new(workspace_dir: Option<PathBuf>) -> Self {
        Self::from_config(Config::load(workspace_dir))
    }

    pub(crate) fn from_config(config: Config) -> Self {
        let ws = config.workspace_dir.clone();
        let agent = Agent::new(config.clone());
        let permissions = Arc::new(Mutex::new(HashMap::new()));
        let questions = Arc::new(Mutex::new(HashMap::new()));
        let cancel_token = Mutex::new(CancellationToken::new());

        // A normal launch starts fresh. Opening saved history is an explicit action.
        let current_session = Arc::new(Mutex::new(Some(Session::new(config.model.clone()))));

        Self {
            turn_active: AtomicBool::new(false),
            agent: Mutex::new(agent),
            current_session,
            cancel_token,
            permissions,
            questions,
            workspace_dir: Mutex::new(ws),
        }
    }
}
