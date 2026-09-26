use crate::core::llm::ChatMessage;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum HistoryItem {
    UserPrompt(String),
    MoaRouting {
        model: String,
        category: String,
        complexity: String,
        #[serde(default)]
        source: Option<String>,
    },
    Thought(String),
    ToolStart {
        name: String,
        args: String,
    },
    ToolLog(String),
    ToolEnd {
        name: String,
        #[serde(default)]
        args: String,
        result: String,
        is_error: bool,
    },
    AssistantMessage(String),
    Error(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Session {
    pub id: String,
    pub created_at: String,
    pub model: String,
    #[serde(default)]
    pub title: Option<String>,
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub history: Vec<HistoryItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SessionMeta {
    pub id: String,
    pub created_at: String,
    pub model: String,
    pub title: String,
    pub message_count: usize,
}

impl Session {
    pub fn new(model: String) -> Self {
        let now = Local::now();
        let id = now.format("%Y%m%d_%H%M%S").to_string();
        let created_at = now.format("%Y-%m-%d %H:%M:%S").to_string();

        Self {
            id,
            created_at,
            model,
            title: None,
            messages: Vec::new(),
            history: Vec::new(),
        }
    }

    pub fn title(&self) -> String {
        if let Some(ref t) = self.title {
            let trimmed = t.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        for msg in &self.messages {
            if msg.role == "user" {
                if let Some(content) = &msg.content {
                    let first_line = content.lines().next().unwrap_or("").trim();
                    if !first_line.is_empty() {
                        let mut t = first_line.chars().take(40).collect::<String>();
                        if first_line.chars().count() > 40 {
                            t.push_str("...");
                        }
                        return t;
                    }
                }
            }
        }
        "Новый диалог".to_string()
    }

    pub fn message_count(&self) -> usize {
        self.messages.iter().filter(|m| m.role != "system").count()
    }

    pub fn sessions_dir(workspace: &Path) -> PathBuf {
        workspace.join(".takiza").join("sessions")
    }

    pub fn save(&self, workspace: &Path) -> std::io::Result<()> {
        let dir = Self::sessions_dir(workspace);
        fs::create_dir_all(&dir)?;
        let file_path = dir.join(format!("{}.json", self.id));
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::other(e))?;
        fs::write(&file_path, json)?;

        let md_path = dir.join(format!("{}.md", self.id));
        let md_content = self.generate_markdown(workspace);
        let _ = fs::write(md_path, md_content);

        if !self.messages.is_empty() {
            let takiza_dir = workspace.join(".takiza");
            let _ = fs::create_dir_all(&takiza_dir);
            let latest_file = takiza_dir.join("latest_session");
            let _ = fs::write(latest_file, &self.id);
        }

        Ok(())
    }

    pub fn latest(workspace: &Path) -> Option<Self> {
        for id in Self::list_by_activity(workspace) {
            if let Some(session) = Self::load(workspace, &id) {
                if !session.messages.is_empty() {
                    return Some(session);
                }
            }
        }

        let latest_file = workspace.join(".takiza").join("latest_session");
        if let Ok(id) = fs::read_to_string(latest_file) {
            let id = id.trim();
            if !id.is_empty() {
                if let Some(session) = Self::load(workspace, id) {
                    if !session.messages.is_empty() {
                        return Some(session);
                    }
                }
            }
        }

        None
    }

    pub fn list_by_activity(workspace: &Path) -> Vec<String> {
        let dir = Self::sessions_dir(workspace);
        let mut entries_with_mtime = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".json") {
                    let mtime = entry
                        .metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    let id = name.trim_end_matches(".json").to_string();
                    entries_with_mtime.push((mtime, id));
                }
            }
        }
        entries_with_mtime.sort_by(|a, b| b.0.cmp(&a.0));
        entries_with_mtime.into_iter().map(|(_, id)| id).collect()
    }

    pub fn list_meta(workspace: &Path) -> Vec<SessionMeta> {
        let mut metas = Vec::new();
        for id in Self::list_by_activity(workspace) {
            if let Some(s) = Self::load(workspace, &id) {
                if s.messages.is_empty() && s.history.is_empty() {
                    continue;
                }
                metas.push(SessionMeta {
                    id: s.id.clone(),
                    created_at: s.created_at.clone(),
                    model: s.model.clone(),
                    title: s.title(),
                    message_count: s.message_count(),
                });
            }
        }
        metas
    }

    pub fn load(workspace: &Path, id: &str) -> Option<Self> {
        let dir = Self::sessions_dir(workspace);
        let file_path = dir.join(format!("{}.json", id));
        let content = fs::read_to_string(file_path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn delete(workspace: &Path, id: &str) -> std::io::Result<()> {
        let dir = Self::sessions_dir(workspace);
        let file_path = dir.join(format!("{}.json", id));
        let md_path = dir.join(format!("{}.md", id));
        let _ = fs::remove_file(md_path);

        let latest_file = workspace.join(".takiza").join("latest_session");
        if let Ok(cur_id) = fs::read_to_string(&latest_file) {
            if cur_id.trim() == id {
                let _ = fs::remove_file(&latest_file);
            }
        }

        fs::remove_file(file_path)
    }

    pub fn generate_markdown(&self, workspace: &Path) -> String {
        let mut md = String::new();
        let display_title = self.title.as_deref().unwrap_or(&self.id);
        md.push_str(&format!("# Takiza Code Session: {}\n\n", display_title));
        md.push_str(&format!("- **Session ID**: `{}`\n", self.id));
        md.push_str(&format!("- **Created**: {}\n", self.created_at));
        md.push_str(&format!("- **Model**: `{}`\n", self.model));
        md.push_str(&format!("- **Workspace**: `{}`\n\n", workspace.display()));
        md.push_str("---\n\n");

        for item in &self.history {
            match item {
                HistoryItem::UserPrompt(p) => {
                    md.push_str(&format!("### 👤 User\n\n{}\n\n", p));
                }
                HistoryItem::MoaRouting { model, category, complexity, source } => {
                    let src_str = source.as_deref().map(|s| format!(" [{}]", s)).unwrap_or_default();
                    md.push_str(&format!("> ⚡ **Takiza MoA{}**: `{}` ({} • {})\n\n", src_str, model, category, complexity));
                }
                HistoryItem::Thought(t) => {
                    md.push_str(&format!("> 💭 **Thinking**:\n> {}\n\n", t.replace('\n', "\n> ")));
                }
                HistoryItem::ToolStart { name, args } => {
                    md.push_str(&format!("⚙️ **Tool Call**: `{}`\n```json\n{}\n```\n\n", name, args));
                }
                HistoryItem::ToolLog(l) => {
                    md.push_str(&format!("- `{}`\n", l));
                }
                HistoryItem::ToolEnd { name, result, is_error, .. } => {
                    let status = if *is_error { "❌ Error" } else { "✅ Output" };
                    md.push_str(&format!("**{} ({})**:\n```\n{}\n```\n\n", status, name, result));
                }
                HistoryItem::AssistantMessage(m) => {
                    md.push_str(&format!("### 🤖 Takiza\n\n{}\n\n", m));
                }
                HistoryItem::Error(e) => {
                    md.push_str(&format!("> ⚠️ **Error**: {}\n\n", e));
                }
            }
        }

        md
    }
}
