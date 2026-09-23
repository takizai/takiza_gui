use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub workspace_dir: PathBuf,
    pub auto_approve: bool,
    pub proxy: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AppPreferences {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub auto_approve: Option<bool>,
    pub proxy: Option<String>,
}

impl Config {
    pub fn config_path() -> Option<PathBuf> {
        dirs_fallback().map(|p| p.join(".config").join("takiza").join("config.json"))
    }

    pub fn load_preferences() -> AppPreferences {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(prefs) = serde_json::from_str::<AppPreferences>(&content) {
                        return prefs;
                    }
                }
            }
        }
        AppPreferences::default()
    }

    pub fn save_preferences(prefs: &AppPreferences) -> std::io::Result<()> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let data = serde_json::to_string_pretty(prefs)?;
            fs::write(path, data)?;
        }
        Ok(())
    }

    pub fn load(workspace: Option<PathBuf>) -> Self {
        dotenvy::dotenv().ok();

        let prefs = Self::load_preferences();

        let api_key = prefs.api_key
            .or_else(|| std::env::var("OPENROUTER_API_KEY").ok())
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .or_else(|| std::env::var("DEEPSEEK_API_KEY").ok())
            .or_else(|| std::env::var("GROQ_API_KEY").ok())
            .unwrap_or_default();

        let base_url = prefs.base_url
            .or_else(|| std::env::var("OPENAI_BASE_URL").ok())
            .unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string())
            .trim_end_matches('/')
            .to_string();

        let model = prefs.model
            .or_else(|| std::env::var("OPENAI_MODEL").ok())
            .unwrap_or_else(|| "anthropic/claude-3.5-sonnet".to_string());

        let proxy = prefs.proxy
            .or_else(|| std::env::var("TAKIZA_PROXY").ok())
            .or_else(|| std::env::var("HTTPS_PROXY").ok())
            .or_else(|| std::env::var("HTTP_PROXY").ok())
            .filter(|s| !s.trim().is_empty());

        let auto_approve = prefs.auto_approve
            .or_else(|| std::env::var("TAKIZA_AUTO_APPROVE").ok().map(|v| v == "1" || v.eq_ignore_ascii_case("true")))
            .unwrap_or(false);

        let workspace_dir = workspace.unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        });

        Self {
            api_key,
            base_url,
            model,
            workspace_dir,
            auto_approve,
            proxy,
        }
    }
}

fn dirs_fallback() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}
