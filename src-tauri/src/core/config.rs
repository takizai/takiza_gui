use serde::{Deserialize, Serialize};
use std::fs;
use std::collections::BTreeMap;
use std::path::PathBuf;
use super::providers::ApiType;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProviderProfile {
    #[serde(default)] pub name: String,
    #[serde(default)] pub base_url: String,
    #[serde(default)] pub model: String,
    #[serde(default)] pub api_type: ApiType,
    #[serde(default)] pub headers: BTreeMap<String, String>,
    pub api_key: Option<String>,
    pub api_key_env: Option<String>,
    #[serde(default)] pub models: Vec<serde_json::Value>,
    #[serde(flatten)] pub extra: BTreeMap<String, serde_json::Value>,
}

fn nonempty(value: Option<String>) -> Option<String> {
    value.map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
}

fn profile_key(profile: &ProviderProfile) -> String {
    match profile.api_key_env.as_deref().filter(|name| !name.trim().is_empty()) {
        Some(name) => std::env::var(name).unwrap_or_default(),
        None => profile.api_key.clone().unwrap_or_default(),
    }
}

// An active profile owns the whole connection. Never mix its endpoint with
// a global OPENAI_API_KEY or model belonging to another provider.
fn resolve_connection(prefs: &AppPreferences, env: impl Fn(&str) -> Option<String>) -> (String, String, String) {
    if let Some(profile) = prefs.active_provider.as_ref().and_then(|id| prefs.providers.get(id)) {
        let key = match profile.api_key_env.as_deref().filter(|name| !name.trim().is_empty()) { Some(name) => env(name).unwrap_or_default(), None => profile.api_key.clone().unwrap_or_default() };
        return (key, normalize_base_url(&profile.base_url), profile.model.trim().to_string());
    }
    let key = match prefs.api_key_env.as_deref().filter(|name| !name.trim().is_empty()) {
        Some(name) => env(name).unwrap_or_default(),
        None if prefs.api_key.is_some() => prefs.api_key.clone().unwrap_or_default(),
        None => nonempty(prefs.api_key.clone()).or_else(|| ["OPENAI_API_KEY", "GROQ_API_KEY", "OPENROUTER_API_KEY", "DEEPSEEK_API_KEY"].iter().find_map(|name| nonempty(env(name)))).unwrap_or_default(),
    };
    let url = nonempty(prefs.base_url.clone()).or_else(|| nonempty(env("OPENAI_BASE_URL"))).unwrap_or_else(|| "https://anymodel.org/v1".into());
    let model = nonempty(prefs.model.clone()).or_else(|| nonempty(env("OPENAI_MODEL"))).unwrap_or_default();
    (key, normalize_base_url(&url), model)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)] pub api_key_env: Option<String>,
    #[serde(default)] pub api_type: ApiType,
    #[serde(default)] pub provider_headers: BTreeMap<String, String>,
    #[serde(default)] pub active_provider: Option<String>,
    #[serde(default)] pub providers: BTreeMap<String, ProviderProfile>,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub workspace_dir: PathBuf,
    pub auto_approve: bool,
    pub proxy: Option<String>,
    pub effort: Option<String>,
    pub mode: Option<String>,
    pub theme: Option<String>,
    pub language: Option<String>,
    #[serde(default = "default_max_steps")]
    pub max_steps: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AppPreferences {
    pub model_catalog: Option<Vec<serde_json::Value>>,
    pub api_type: Option<ApiType>,
    pub headers: Option<BTreeMap<String, String>>,
    pub api_key_env: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_provider_name: Option<String>,
    pub active_provider: Option<String>,
    #[serde(default)] pub providers: BTreeMap<String, ProviderProfile>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub auto_approve: Option<bool>,
    pub proxy: Option<String>,
    pub last_workspace_dir: Option<PathBuf>,
    pub effort: Option<String>,
    pub mode: Option<String>,
    pub theme: Option<String>,
    pub language: Option<String>,
    pub max_steps: Option<usize>,
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
    pub fn reasoning_levels(&self) -> Vec<String> {
        let metadata = self.active_provider.as_ref().and_then(|id| self.providers.get(id))
            .filter(|p| normalize_base_url(&p.base_url) == normalize_base_url(&self.base_url) && p.api_type == self.api_type)
            .and_then(|p| p.models.iter().find(|m| m["id"].as_str() == Some(self.model.as_str())));
        super::reasoning::levels(&self.model, self.api_type.as_str(), metadata)
    }

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
        let path = Self::config_path().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Cannot locate preferences file"))?;
        save_preferences_at(&path, prefs)
    }

    pub fn load(workspace: Option<PathBuf>) -> Self {
        load_all_env_sources(&workspace);

        let prefs = Self::load_preferences();

        let (api_key, base_url, model) = resolve_connection(&prefs, |name| std::env::var(name).ok());
        let active_provider = prefs.active_provider.clone();
        let active_profile = active_provider.as_ref().and_then(|id| prefs.providers.get(id));
        let api_type = active_profile.map(|profile| profile.api_type).or(prefs.api_type).unwrap_or_default();
        let provider_headers = active_profile.map(|profile| profile.headers.clone()).or(prefs.headers.clone()).unwrap_or_default();
        let api_key_env = active_profile.and_then(|profile| profile.api_key_env.clone()).or_else(|| if active_profile.is_none() { prefs.api_key_env.clone() } else { None });
        let providers = prefs.providers.iter().map(|(id, profile)| {
            let mut resolved = profile.clone();
            resolved.api_key = Some(profile_key(profile));
            (id.clone(), resolved)
        }).collect();

        let proxy = prefs.proxy
            .filter(|s| !s.trim().is_empty())
            .or_else(|| std::env::var("TAKIZA_PROXY").ok())
            .or_else(|| std::env::var("HTTPS_PROXY").ok())
            .or_else(|| std::env::var("HTTP_PROXY").ok())
            .filter(|s| !s.trim().is_empty());

        let auto_approve = prefs.auto_approve
            .or_else(|| std::env::var("TAKIZA_AUTO_APPROVE").ok().map(|v| v == "1" || v.eq_ignore_ascii_case("true")))
            .unwrap_or(false);

        let effort = super::reasoning::preference(prefs.effort, std::env::var("REASONING_EFFORT").ok());

        let mode = prefs.mode
            .filter(|s| !s.trim().is_empty())
            .or_else(|| std::env::var("TAKIZA_MODE").ok())
            .filter(|s| !s.trim().is_empty())
            .or_else(|| Some("manual".to_string()));

        let language = Some(normalize_language(prefs.language.as_deref()).to_string());
        let theme = Some(prefs.theme.as_deref().map(normalize_theme)
            .filter(|theme| matches!(theme.as_str(), "amber" | "cyberpunk" | "emerald" | "nord" | "monochrome" | "dracula" | "rose" | "ocean" | "sakura" | "solarized"))
            .unwrap_or_else(|| "amber".to_string()));

        let workspace_dir = workspace
            .or_else(|| prefs.last_workspace_dir)
            .or_else(|| {
                dirs_fallback().map(|h| h.join("takiza-harness")).filter(|p| p.exists())
            })
            .unwrap_or_else(|| {
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            });

        Self {
            api_key_env,
            api_type,
            provider_headers,
            active_provider,
            providers,
            api_key,
            base_url,
            model,
            workspace_dir,
            auto_approve,
            proxy,
            effort,
            mode,
            theme,
            language,
            max_steps: std::env::var("TAKIZA_MAX_STEPS").ok().and_then(|value| value.trim().parse().ok()).or(prefs.max_steps).unwrap_or_else(default_max_steps),
        }
    }
}

pub fn normalize_language(value: Option<&str>) -> &'static str {
    match value.unwrap_or("ru").trim().to_lowercase().as_str() {
        "en" | "english" => "en", "zh" | "zh-cn" | "chinese" => "zh", _ => "ru",
    }
}
pub fn normalize_theme(value: &str) -> String {
    value.trim().to_lowercase()
}

pub fn normalize_base_url(value: &str) -> String {
    let value = value.trim().trim_end_matches('/');
    if let Ok(mut url) = reqwest::Url::parse(value) {
        // AnyModel redirects HTTP to HTTPS. Avoid that redirect so POST and
        // Authorization reach the API together. Custom/local HTTP stays valid.
        if url.scheme() == "http" && url.host_str() == Some("anymodel.org") {
            url.set_scheme("https").expect("HTTP URL supports HTTPS");
            return url.to_string().trim_end_matches('/').to_string();
        }
    }
    value.to_string()
}

fn save_preferences_at(path: &std::path::Path, prefs: &AppPreferences) -> std::io::Result<()> {
    let mut value: serde_json::Value = if path.exists() {
        serde_json::from_slice(&fs::read(path)?).map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?
    } else {
        serde_json::json!({ "agreed_to_terms": true, "terms_version": "1.0.0", "theme": "Amber" })
    };
    let target = value.as_object_mut().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Preferences must be a JSON object"))?;
    let mut patch = serde_json::to_value(prefs)?;
    if let Some(headers) = &prefs.headers {
        super::providers::validate_headers(headers).map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    }
    if let Some(env) = &prefs.api_key_env {
        let env = env.trim();
        if env.is_empty() { patch["api_key_env"] = serde_json::Value::Null; }
        else {
            if !env.chars().enumerate().all(|(i, c)| c == '_' || c.is_ascii_alphabetic() || (i > 0 && c.is_ascii_digit())) {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Некорректное имя переменной окружения."));
            }
            patch["api_key"] = serde_json::json!("");
            patch["api_key_env"] = serde_json::json!(env);
        }
    }
    // Update the selected provider in place, preserving its cached models,
    // headers, API type and the other profiles used by the TUI.
    let mut selected = prefs.active_provider.clone().or_else(|| target.get("active_provider").and_then(|value| value.as_str()).map(str::to_owned));
    // Legacy flat connections gain a real profile when protocol/auth is edited,
    // since the CLI resolves these settings only from a named provider.
    let profile_name = prefs.new_provider_name.clone().or_else(|| {
        (selected.is_none() && (prefs.api_type.is_some() || prefs.headers.is_some() || prefs.api_key_env.is_some()))
            .then(|| target.get("provider").and_then(|value| value.as_str()).unwrap_or("Default").to_string())
    });
    if let Some(name) = &profile_name {
        let invalid = |message: &str| std::io::Error::new(std::io::ErrorKind::InvalidInput, message);
        if name.trim().is_empty() { return Err(invalid("Укажите название провайдера.")); }
        let endpoint = normalize_base_url(prefs.base_url.as_deref().unwrap_or_default());
        let url = reqwest::Url::parse(&endpoint).map_err(|_| invalid("Укажите корректный HTTP или HTTPS адрес API."))?;
        if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
            return Err(invalid("Укажите корректный HTTP или HTTPS адрес API."));
        }
        let model = prefs.model.as_deref().unwrap_or_default().trim();
        if model.is_empty() { return Err(invalid("Укажите URL провайдера и ID модели.")); }
        let profiles = target.entry("providers").or_insert_with(|| serde_json::json!({})).as_object_mut()
            .ok_or_else(|| invalid("Provider profiles must be a JSON object"))?;
        let id = (1..).map(|number| format!("gui-provider-{number}")).find(|id| !profiles.contains_key(id)).unwrap();
        profiles.insert(id.clone(), serde_json::json!({
            "name": name.trim(), "base_url": endpoint, "model": model,
            "api_key": prefs.api_key.as_deref().unwrap_or_default().trim(), "models": [],
        }));
        patch["base_url"] = serde_json::json!(endpoint);
        patch["model"] = serde_json::json!(model);
        patch["api_key"] = serde_json::json!(prefs.api_key.as_deref().unwrap_or_default().trim());
        selected = Some(id);
    }
    if prefs.api_key_env.as_deref().is_some_and(|name| !name.trim().is_empty()) { patch["api_key"] = serde_json::json!(""); }
    if let Some(url) = &prefs.base_url {
        let current_matches = selected.as_ref().and_then(|id| target.get("providers")?.get(id)?.get("base_url")?.as_str())
            .is_some_and(|current| normalize_base_url(current) == normalize_base_url(url));
        if !current_matches && target.get("providers").and_then(|value| value.as_object()).is_some() {
            selected = target.get("providers").and_then(|value| value.as_object()).and_then(|profiles| profiles.iter().find(|(_, profile)| profile.get("base_url").and_then(|value| value.as_str()).is_some_and(|current| normalize_base_url(current) == normalize_base_url(url))).map(|(id, _)| id.clone()));
            if selected.is_none() {
                let profiles = target.get("providers").and_then(|value| value.as_object()).unwrap();
                let id = (1..).map(|number| format!("gui-provider-{number}")).find(|id| !profiles.contains_key(id)).unwrap();
                target.get_mut("providers").unwrap().as_object_mut().unwrap().insert(id.clone(), serde_json::json!({ "name": "Custom", "base_url": url, "model": "", "api_key": "" }));
                selected = Some(id);
            }
        }
    }
    if let Some(id) = &selected {
        let profile = target.get_mut("providers").and_then(|value| value.as_object_mut()).and_then(|profiles| profiles.get_mut(id)).and_then(|profile| profile.as_object_mut())
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Unknown provider profile"))?;
        let connection_signature = |profile: &serde_json::Map<String, serde_json::Value>| serde_json::json!([
            normalize_base_url(profile.get("base_url").and_then(|v| v.as_str()).unwrap_or_default()),
            profile.get("api_type").and_then(|v| v.as_str()).unwrap_or("chat_completions"),
            profile.get("api_key").and_then(|v| v.as_str()).unwrap_or_default(),
            profile.get("api_key_env").and_then(|v| v.as_str()).unwrap_or_default(),
            profile.get("headers").cloned().unwrap_or_else(|| serde_json::json!({})),
        ]);
        let before = connection_signature(profile);
        for key in ["api_key", "base_url", "model", "api_type", "headers", "api_key_env"] {
            if let Some(value) = patch.get(key).filter(|value| !value.is_null() || (key == "api_key_env" && prefs.api_key_env.is_some())) { profile.insert(key.into(), value.clone()); }
            // Profile connections are not duplicated into legacy global fields.
            patch.as_object_mut().unwrap().remove(key);
        }
        if before != connection_signature(profile) { profile.insert("models".into(), serde_json::json!([])); }
        if let Some(catalog) = &prefs.model_catalog { profile.insert("models".into(), serde_json::json!(catalog)); }
        target.insert("active_provider".into(), serde_json::json!(id));
    }
    for (key, value) in patch.as_object().unwrap() {
        if key == "providers" || key == "active_provider" || key == "new_provider_name" || key == "model_catalog" { continue; }
        if !value.is_null() || (key == "api_key_env" && prefs.api_key_env.is_some()) { target.insert(key.clone(), value.clone()); }
    }
    if let Some(theme) = &prefs.theme {
        // Keep the format understood by existing TUI releases. The frontend
        // normalizes this value when loading it.
        let canonical = match normalize_theme(theme).as_str() {
            "amber" => "Amber", "cyberpunk" => "Cyberpunk", "emerald" => "Emerald",
            "nord" => "Nord", "monochrome" => "Monochrome",
            "dracula" => "Dracula", "rose" => "Rose", "ocean" => "Ocean", "sakura" => "Sakura", "solarized" => "Solarized",
            _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Unknown theme")),
        };
        target.insert("theme".into(), serde_json::json!(canonical));
    }
    if let Some(language) = &prefs.language { target.insert("language".into(), serde_json::json!(normalize_language(Some(language)))); }
    target.entry("agreed_to_terms").or_insert(serde_json::json!(true));
    target.entry("terms_version").or_insert(serde_json::json!("1.0.0"));
    if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
    fs::write(path, serde_json::to_vec_pretty(&value)?)
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
    fn adding_named_providers_preserves_profiles_and_allows_separate_accounts() {
        let path = std::env::temp_dir().join(format!("takiza-add-provider-{}.json", chrono::Local::now().timestamp_nanos_opt().unwrap()));
        let original = serde_json::json!({
            "active_provider": "gui-provider-1", "model": "legacy-model", "api_key": "legacy-key", "cli_metadata": { "keep": true },
            "providers": { "gui-provider-1": { "name": "Original", "base_url": "https://same.test/v1", "api_key": "original-key", "model": "original-model", "models": [{"id":"cached-model"}], "headers": {"keep":"yes"} } }
        });
        for initial in [original.clone(), serde_json::json!({"model":"legacy-model","api_key":"legacy-key"})] {
            fs::write(&path, serde_json::to_vec(&initial).unwrap()).unwrap();
            let request = AppPreferences { new_provider_name: Some("  My provider  ".into()), base_url: Some(" https://same.test/v1/ ".into()),
                api_key: Some("  new-account-key  ".into()), model: Some("  selected-model  ".into()), ..Default::default() };
            save_preferences_at(&path, &request).unwrap();
            let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            let id = saved["active_provider"].as_str().unwrap();
            let added = &saved["providers"][id];
            assert_eq!(added["name"], "My provider");
            assert_eq!(added["model"], "selected-model");
            assert_eq!(added["base_url"], "https://same.test/v1");
            assert_eq!(added["api_key"], "new-account-key");
            assert!(saved.get("new_provider_name").is_none());
            if initial.get("providers").is_some() {
                assert_eq!(id, "gui-provider-2");
                assert_eq!(saved["providers"]["gui-provider-1"], original["providers"]["gui-provider-1"]);
                assert_eq!(saved["cli_metadata"], original["cli_metadata"]);
            }
            assert_eq!(saved["api_key"], initial["api_key"]);
            let reloaded: AppPreferences = serde_json::from_value(saved).unwrap();
            assert_eq!(resolve_connection(&reloaded, |_| Some("wrong-key".into())), ("new-account-key".into(), "https://same.test/v1".into(), "selected-model".into()));
        }
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn invalid_new_provider_does_not_change_config() {
        let path = std::env::temp_dir().join(format!("takiza-invalid-provider-{}.json", chrono::Local::now().timestamp_nanos_opt().unwrap()));
        let original = r#"{"active_provider":"keep","providers":{"keep":{"name":"Keep","base_url":"http://localhost/v1","model":"keep-model"}}}"#;
        fs::write(&path, original).unwrap();
        for (name, url, model) in [(" ", "https://new.test/v1", "model"), ("New", "ftp://new.test", "model"), ("New", "invalid", "model"), ("New", "http://localhost/v1", "")] {
            let request = AppPreferences { new_provider_name: Some(name.into()), base_url: Some(url.into()), model: Some(model.into()), ..Default::default() };
            assert!(save_preferences_at(&path, &request).is_err());
            assert_eq!(fs::read_to_string(&path).unwrap(), original);
        }
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn protocol_env_and_headers_roundtrip_without_materializing_secrets() {
        let path = std::env::temp_dir().join(format!("takiza-protocol-{}.json", chrono::Local::now().timestamp_nanos_opt().unwrap()));
        let original = serde_json::json!({"active_provider":"selected","providers":{"selected":{"name":"Existing","base_url":"https://provider.test/v1","model":"chosen","api_key":"stale-direct-key","models":[{"id":"cached"}],"context_window":1234},"other":{"name":"Other","base_url":"https://other.test/v1","model":"keep","api_key":"other-key"}}});
        fs::write(&path, serde_json::to_vec(&original).unwrap()).unwrap();
        let prefs = AppPreferences { api_type: Some(ApiType::AnthropicMessages), api_key_env: Some("PROVIDER_TEST_KEY".into()),
            api_key: Some("must-not-be-written".into()), headers: Some(BTreeMap::from([("Authorization".into(), "Custom fixture".into())])), ..Default::default() };
        save_preferences_at(&path, &prefs).unwrap();
        let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(saved["providers"]["selected"]["api_type"], "anthropic_messages");
        assert_eq!(saved["providers"]["selected"]["api_key"], "");
        assert_eq!(saved["providers"]["selected"]["api_key_env"], "PROVIDER_TEST_KEY");
        assert_eq!(saved["providers"]["selected"]["models"], serde_json::json!([]));
        assert_eq!(saved["providers"]["other"], original["providers"]["other"]);
        assert_eq!(saved["providers"]["selected"]["context_window"], 1234);
        let loaded: AppPreferences = serde_json::from_value(saved).unwrap();
        assert_eq!(resolve_connection(&loaded, |name| (name == "PROVIDER_TEST_KEY").then(|| "live-env-key".into())).0, "live-env-key");
        assert_eq!(resolve_connection(&loaded, |_| None).0, "");
        save_preferences_at(&path, &AppPreferences { api_key_env: Some("".into()), api_key: Some("".into()), headers: Some(Default::default()), ..Default::default() }).unwrap();
        let loaded: AppPreferences = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(resolve_connection(&loaded, |_| Some("unrelated".into())).0, "");
        assert!(loaded.providers["selected"].api_key_env.is_none());
        assert!(loaded.providers["selected"].headers.is_empty());
        let before = fs::read(&path).unwrap();
        for headers in [BTreeMap::from([("bad name".into(), "value".into())]), BTreeMap::from([("x-test".into(), "bad\r\nvalue".into())]), BTreeMap::from([("Authorization".into(), "one".into()), ("authorization".into(), "two".into())])] {
            assert!(save_preferences_at(&path, &AppPreferences { headers: Some(headers), ..Default::default() }).is_err());
            assert_eq!(fs::read(&path).unwrap(), before);
        }
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn active_provider_owns_key_endpoint_and_model() {
        let prefs: AppPreferences = serde_json::from_value(serde_json::json!({
            "active_provider": "anymodel", "api_key": "", "base_url": "", "model": "",
            "providers": { "anymodel": { "name": "AnyModel", "api_key": "profile-key", "base_url": "https://anymodel.org/v1", "model": "ds/test-model" } }
        })).unwrap();
        let result = resolve_connection(&prefs, |_| Some("unrelated-legacy-value".into()));
        assert_eq!(result, ("profile-key".into(), "https://anymodel.org/v1".into(), "ds/test-model".into()));
    }

    #[test]
    fn named_profile_env_key_never_falls_back_to_another_provider() {
        let mut prefs: AppPreferences = serde_json::from_value(serde_json::json!({
            "active_provider": "selected", "providers": { "selected": { "base_url": "https://example.test/v1", "model": "native-model", "api_key_env": "SELECTED_PROVIDER_KEY" } }
        })).unwrap();
        assert_eq!(resolve_connection(&prefs, |name| if name == "SELECTED_PROVIDER_KEY" { Some("selected-key".into()) } else { Some("wrong-key".into()) }).0, "selected-key");
        prefs.providers.get_mut("selected").unwrap().api_key_env = None;
        assert_eq!(resolve_connection(&prefs, |_| Some("wrong-key".into())).0, "");
    }

    #[test]
    fn profile_model_patch_and_switch_preserve_other_profiles_and_metadata() {
        let path = std::env::temp_dir().join(format!("takiza-profiles-{}.json", chrono::Local::now().timestamp_nanos_opt().unwrap()));
        let original = serde_json::json!({
            "active_provider": "one", "model": "", "api_key": "", "base_url": "",
            "providers": {
                "one": { "name": "One", "api_key": "key-one", "base_url": "https://one.test/v1", "model": "old-one", "models": [{"id":"new-one"}], "headers": {"x-test":"keep"}, "api_type":"chat_completions" },
                "two": { "name": "Two", "api_key": "key-two", "base_url": "https://two.test/v1", "model": "old-two" }
            }
        });
        fs::write(&path, serde_json::to_vec(&original).unwrap()).unwrap();
        save_preferences_at(&path, &AppPreferences { model: Some("new-one".into()), ..Default::default() }).unwrap();
        let updated: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(updated["providers"]["one"]["model"], "new-one");
        for key in ["api_key", "models", "headers", "api_type"] { assert_eq!(updated["providers"]["one"][key], original["providers"]["one"][key]); }
        assert_eq!(updated["providers"]["two"], original["providers"]["two"]);
        save_preferences_at(&path, &AppPreferences { active_provider: Some("two".into()), ..Default::default() }).unwrap();
        let switched: AppPreferences = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(resolve_connection(&switched, |_| None), ("key-two".into(), "https://two.test/v1".into(), "old-two".into()));
        save_preferences_at(&path, &AppPreferences { effort: Some("high".into()), ..Default::default() }).unwrap();
        let appearance: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(appearance["providers"], updated["providers"]);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn tui_language_and_theme_aliases_are_compatible() {
        for value in ["zh", "zh-CN", "Chinese"] { assert_eq!(normalize_language(Some(value)), "zh"); }
        assert_eq!(normalize_language(Some("English")), "en");
        assert_eq!(normalize_language(Some("Russian")), "ru");
        let path = std::env::temp_dir().join(format!("takiza-language-{}.json", chrono::Local::now().timestamp_nanos_opt().unwrap()));
        let original = serde_json::json!({"active_provider":"selected", "providers":{"selected":{"model":"keep-model","api_key":"test-only","base_url":"https://provider.test/v1"}}});
        fs::write(&path, serde_json::to_vec(&original).unwrap()).unwrap();
        for language in ["en", "zh", "ru"] {
            save_preferences_at(&path, &AppPreferences { language: Some(language.into()), ..Default::default() }).unwrap();
            let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            assert_eq!(saved["language"], language);
            assert_eq!(saved["providers"], original["providers"]);
            assert_eq!(saved["active_provider"], original["active_provider"]);
        }
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn anymodel_http_uses_https_without_changing_custom_endpoints() {
        assert_eq!(normalize_base_url(" http://anymodel.org/v1/ "), "https://anymodel.org/v1");
        assert_eq!(normalize_base_url("https://anymodel.org/v1/"), "https://anymodel.org/v1");
        assert_eq!(normalize_base_url("http://localhost:11434/v1/"), "http://localhost:11434/v1");
        assert_eq!(normalize_base_url("http://anymodel.org.example/v1"), "http://anymodel.org.example/v1");
    }

    #[test]
    fn theme_patch_preserves_other_preferences_and_uses_tui_spelling() {
        let path = std::env::temp_dir().join(format!("takiza-gui-theme-{}-{}.json", std::process::id(), chrono::Local::now().timestamp_nanos_opt().unwrap()));
        let original = serde_json::json!({
            "agreed_to_terms": true, "terms_version": "1.0.0", "accepted_at": "2026-10-02",
            "theme": "Amber", "provider": "custom", "model": "saved-model", "api_key": "test-only",
            "base_url": "http://localhost/v1", "mode": "moa", "effort": "high", "auto_approve": false,
            "proxy": "http://localhost:8080", "last_workspace_dir": "/workspace", "extra_setting": 42
        });
        fs::write(&path, serde_json::to_vec(&original).unwrap()).unwrap();
        for (theme, canonical) in [("amber", "Amber"), ("cyberpunk", "Cyberpunk"), ("emerald", "Emerald"),
            ("nord", "Nord"), ("monochrome", "Monochrome"), ("dracula", "Dracula"), ("rose", "Rose"), ("ocean", "Ocean"), ("sakura", "Sakura"), ("solarized", "Solarized")] {
            save_preferences_at(&path, &AppPreferences { theme: Some(theme.into()), ..Default::default() }).unwrap();
            let mut saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            assert_eq!(saved["theme"], canonical);
            saved["theme"] = original["theme"].clone();
            assert_eq!(saved, original);
        }
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn theme_patch_does_not_overwrite_invalid_preferences() {
        let path = std::env::temp_dir().join(format!("takiza-gui-theme-invalid-{}-{}.json", std::process::id(), chrono::Local::now().timestamp_nanos_opt().unwrap()));
        fs::write(&path, "invalid json").unwrap();
        let prefs = AppPreferences { theme: Some("nord".into()), ..Default::default() };
        assert!(save_preferences_at(&path, &prefs).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "invalid json");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_load_inherits_takiza_harness_env() {
        // Parse a synthetic harness .env without changing process-wide variables
        // or requiring the developer's credentials on clean CI machines.
        let source = b"HARNESS_KEY=test-only\nOPENAI_API_KEY=unrelated\nOPENAI_MODEL=unrelated\n";
        let variables: BTreeMap<String, String> = dotenvy::from_read_iter(&source[..])
            .collect::<Result<_, _>>().unwrap();
        let prefs: AppPreferences = serde_json::from_value(serde_json::json!({
            "active_provider": "harness",
            "providers": {"harness": {
                "api_key_env": "HARNESS_KEY", "base_url": "https://provider.test/v1/",
                "model": "chosen-model", "api_type": "anthropic_messages"
            }}
        })).unwrap();
        assert_eq!(resolve_connection(&prefs, |name| variables.get(name).cloned()),
            ("test-only".into(), "https://provider.test/v1".into(), "chosen-model".into()));
        assert_eq!(resolve_connection(&prefs, |_| None),
            (String::new(), "https://provider.test/v1".into(), "chosen-model".into()));
    }
}

fn default_max_steps() -> usize { 100 }
