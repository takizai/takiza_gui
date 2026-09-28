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
    pub effort: Option<String>,
    pub mode: Option<String>,
    pub theme: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AppPreferences {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub auto_approve: Option<bool>,
    pub proxy: Option<String>,
    pub last_workspace_dir: Option<PathBuf>,
    pub effort: Option<String>,
    pub mode: Option<String>,
    pub theme: Option<String>,
}

fn load_all_env_sources(workspace: &Option<PathBuf>) {
    // 1. If workspace is provided, try workspace/.env
    if let Some(ws) = workspace {
        let ws_env = ws.join(".env");
        if ws_env.exists() {
            let _ = dotenvy::from_path(&ws_env);
        }
    }

    // 2. Try current working directory .env
    dotenvy::dotenv().ok();

    // 3. Try ~/takiza-harness/.env
    if let Some(home) = dirs_fallback() {
        let harness_env = home.join("takiza-harness").join(".env");
        if harness_env.exists() {
            let _ = dotenvy::from_path(&harness_env);
        }
        let config_env = home.join(".config").join("takiza").join(".env");
        if config_env.exists() {
            let _ = dotenvy::from_path(&config_env);
        }
    }
}

impl Config {
    pub fn config_path() -> Option<PathBuf> {
        if let Ok(curr) = std::env::current_dir() {
            let local = curr.join(".takiza").join("config.json");
            if local.exists() {
                return Some(local);
            }
        }
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
            let mut val: serde_json::Value = if path.exists() {
                fs::read_to_string(&path)
                    .ok()
                    .and_then(|c| serde_json::from_str(&c).ok())
                    .unwrap_or_else(|| serde_json::json!({}))
            } else {
                serde_json::json!({
                    "agreed_to_terms": true,
                    "terms_version": "1.0.0",
                    "theme": "amber"
                })
            };

            // Ensure terms agreement flag is preserved for CLI
            if val.get("agreed_to_terms").is_none() {
                val["agreed_to_terms"] = serde_json::Value::Bool(true);
            }
            if val.get("terms_version").is_none() {
                val["terms_version"] = serde_json::Value::String("1.0.0".to_string());
            }

            if let Some(ref k) = prefs.api_key {
                val["api_key"] = serde_json::Value::String(k.clone());
            }
            if let Some(ref u) = prefs.base_url {
                val["base_url"] = serde_json::Value::String(u.clone());
            }
            if let Some(ref m) = prefs.model {
                val["model"] = serde_json::Value::String(m.clone());
            }
            if let Some(a) = prefs.auto_approve {
                val["auto_approve"] = serde_json::Value::Bool(a);
            }
            if let Some(ref p) = prefs.proxy {
                val["proxy"] = serde_json::Value::String(p.clone());
            }
            if let Some(ref ws) = prefs.last_workspace_dir {
                val["last_workspace_dir"] = serde_json::Value::String(ws.to_string_lossy().to_string());
            }

            if let Some(ref e) = prefs.effort {
                val["effort"] = serde_json::Value::String(e.clone());
            }
            if let Some(ref m) = prefs.mode {
                val["mode"] = serde_json::Value::String(m.clone());
            }
            if let Some(ref t) = prefs.theme {
                val["theme"] = serde_json::Value::String(t.clone());
            }

            let data = serde_json::to_string_pretty(&val)?;
            fs::write(path, data)?;
        }
        Ok(())
    }

    pub fn load(workspace: Option<PathBuf>) -> Self {
        load_all_env_sources(&workspace);

        let prefs = Self::load_preferences();

        // Exact priority order as takiza-harness:
        // 1. prefs.api_key
        // 2. OPENAI_API_KEY
        // 3. GROQ_API_KEY
        // 4. OPENROUTER_API_KEY
        // 5. DEEPSEEK_API_KEY
        let api_key = prefs.api_key
            .filter(|s| !s.trim().is_empty())
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .or_else(|| std::env::var("GROQ_API_KEY").ok())
            .or_else(|| std::env::var("OPENROUTER_API_KEY").ok())
            .or_else(|| std::env::var("DEEPSEEK_API_KEY").ok())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_default();

        let base_url = prefs.base_url
            .filter(|s| !s.trim().is_empty())
            .or_else(|| std::env::var("OPENAI_BASE_URL").ok())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "https://anymodel.org/v1".to_string())
            .trim_end_matches('/')
            .to_string();

        let model = prefs.model
            .filter(|s| !s.trim().is_empty())
            .or_else(|| std::env::var("OPENAI_MODEL").ok())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "llama-3.3-70b-versatile".to_string());

        let proxy = prefs.proxy
            .filter(|s| !s.trim().is_empty())
            .or_else(|| std::env::var("TAKIZA_PROXY").ok())
            .or_else(|| std::env::var("HTTPS_PROXY").ok())
            .or_else(|| std::env::var("HTTP_PROXY").ok())
            .filter(|s| !s.trim().is_empty());

        let auto_approve = prefs.auto_approve
            .or_else(|| std::env::var("TAKIZA_AUTO_APPROVE").ok().map(|v| v == "1" || v.eq_ignore_ascii_case("true")))
            .unwrap_or(false);

        let effort = prefs.effort
            .filter(|s| !s.trim().is_empty())
            .or_else(|| std::env::var("REASONING_EFFORT").ok())
            .filter(|s| !s.trim().is_empty())
            .or_else(|| Some("medium".to_string()));

        let mode = prefs.mode
            .filter(|s| !s.trim().is_empty())
            .or_else(|| std::env::var("TAKIZA_MODE").ok())
            .filter(|s| !s.trim().is_empty())
            .or_else(|| Some("manual".to_string()));

        let theme = prefs.theme
            .filter(|s| !s.trim().is_empty())
            .or_else(|| Some("amber".to_string()));

        let workspace_dir = workspace
            .or_else(|| prefs.last_workspace_dir)
            .or_else(|| {
                dirs_fallback().map(|h| h.join("takiza-harness")).filter(|p| p.exists())
            })
            .unwrap_or_else(|| {
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            });

        Self {
            api_key,
            base_url,
            model,
            workspace_dir,
            auto_approve,
            proxy,
            effort,
            mode,
            theme,
        }
    }
}

pub fn dirs_fallback() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
        .or_else(|| {
            let drive = std::env::var_os("HOMEDRIVE");
            let path = std::env::var_os("HOMEPATH");
            match (drive, path) {
                (Some(d), Some(p)) => {
                    let mut b = PathBuf::from(d);
                    b.push(p);
                    Some(b)
                }
                _ => None,
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_inherits_takiza_harness_env() {
        let config = Config::load(None);
        // Should successfully resolve non-empty API key and base_url
        assert!(!config.api_key.is_empty(), "API key should be resolved");
        assert!(!config.base_url.is_empty(), "Base URL should be resolved");
        assert!(!config.model.is_empty(), "Model should be resolved");
    }
}
