use crate::core::agent::{QuestionAnswer, QuestionItem};
use crate::core::llm::ChatMessage;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum HistoryItem {
    FileDiff(String),
    ResponseStats { elapsed_ms: u64, tokens: Option<u64>, usage_complete: bool },
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
    QuestionInteraction {
        questions: Vec<QuestionItem>,
        answers: Vec<QuestionAnswer>,
        skipped: bool,
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
    #[serde(default, deserialize_with = "super::session_format::deserialize_history")]
    pub history: Vec<HistoryItem>,
    #[serde(default, flatten)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
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
        let id = now.format("%Y%m%d_%H%M%S_%9f").to_string();
        let created_at = now.format("%Y-%m-%d %H:%M:%S").to_string();

        Self {
            id,
            created_at,
            model,
            title: None,
            messages: Vec::new(),
            history: Vec::new(),
            extra: Default::default(),
        }
    }

    /// Retain the existing user message and rebuild the response from that turn.
    pub fn for_regeneration(&self, history_index: usize, edited: Option<&str>) -> Result<Self, String> {
        let prompt = match self.history.get(history_index) {
            Some(HistoryItem::UserPrompt(prompt)) if !prompt.starts_with('!') => prompt,
            _ => return Err("Выберите сообщение пользователя, отправленное модели".into()),
        };
        let mut cursor = 0;
        let mut target = None;
        for (index, item) in self.history.iter().enumerate().take(history_index + 1) {
            if let HistoryItem::UserPrompt(text) = item {
                if text.starts_with('!') { continue; }
                let found = self.messages.iter().enumerate().skip(cursor).find(|(_, message)| message.role == "user" && message.content.as_deref() == Some(text.as_str()))
                    .map(|(position, _)| position).ok_or("Не удалось сопоставить сообщение с контекстом сессии")?;
                cursor = found + 1;
                if index == history_index { target = Some(found); }
            }
        }
        let target = target.ok_or("Сообщение отсутствует в контексте сессии")?;
        let input = edited.unwrap_or(prompt).trim();
        if input.is_empty() || input.starts_with('!') { return Err("Введите непустой запрос к модели".into()); }
        let mut session = self.clone();
        session.history.truncate(history_index + 1);
        session.history[history_index] = HistoryItem::UserPrompt(input.into());
        session.messages.truncate(target + 1);
        session.messages[target].content = Some(input.into());
        Ok(session)
    }

    pub fn checkpoint_history(&self) -> Vec<HistoryItem> {
        if !self.history.is_empty() { return self.history.clone(); }
        self.messages.iter().filter_map(|message| match message.role.as_str() {
            "user" => message.content.clone().map(HistoryItem::UserPrompt),
            "assistant" => message.content.clone().filter(|text| !text.is_empty()).map(HistoryItem::AssistantMessage),
            _ => None,
        }).collect()
    }

    pub fn title(&self) -> String {
        if let Some(ref t) = self.title {
            let trimmed = t.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        for prompt in self.messages.iter().filter(|message| message.role == "user").filter_map(|message| message.content.as_deref())
            .chain(self.history.iter().filter_map(|item| if let HistoryItem::UserPrompt(prompt) = item { Some(prompt.as_str()) } else { None })) {
            let title = Self::title_from_prompt(prompt);
            if !title.is_empty() { return title; }
        }
        "Empty conversation".into()
    }

    pub fn title_from_prompt(prompt: &str) -> String {
        let line = prompt.lines().find(|line| !line.trim().is_empty()).unwrap_or("").trim();
        let mut title = line.chars().take(40).collect::<String>();
        if line.chars().count() > 40 { title.push_str("..."); }
        title
    }

    pub fn has_conversation(&self) -> bool {
        self.messages.iter().any(|message| message.role != "system" &&
            (message.content.as_ref().is_some_and(|text| !text.trim().is_empty()) || !message.image_urls.is_empty()
                || message.tool_calls.as_ref().is_some_and(|calls| !calls.is_empty())))
            || self.history.iter().any(|item| match item {
                HistoryItem::UserPrompt(text) | HistoryItem::AssistantMessage(text) => !text.trim().is_empty(),
                HistoryItem::QuestionInteraction { questions, answers, .. } => !questions.is_empty() || !answers.is_empty(),
                _ => false,
            })
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
        let temporary = file_path.with_extension(format!("json.{}.{}.tmp", std::process::id(), Local::now().timestamp_nanos_opt().unwrap()));
        fs::write(&temporary, json)?;
        fs::rename(&temporary, &file_path)?;

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
                if session.has_conversation() {
                    return Some(session);
                }
            }
        }

        let latest_file = workspace.join(".takiza").join("latest_session");
        if let Ok(id) = fs::read_to_string(latest_file) {
            let id = id.trim();
            if !id.is_empty() {
                if let Some(session) = Self::load(workspace, id) {
                    if session.has_conversation() {
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
        entries_with_mtime.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
        entries_with_mtime.into_iter().map(|(_, id)| id).collect()
    }

    pub fn list(workspace: &Path) -> Vec<String> {
        Self::list_by_activity(workspace).into_iter().filter(|id| Self::load(workspace,id).is_some_and(|session| session.has_conversation())).collect()
    }

    pub fn list_meta(workspace: &Path) -> Vec<SessionMeta> {
        let mut metas = Vec::new();
        for id in Self::list_by_activity(workspace) {
            if let Some(s) = Self::load(workspace, &id) {
                if !s.has_conversation() {
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
        let mut session: Self = serde_json::from_str(&content).ok()?;
        if session.history.is_empty() { session.history = session.checkpoint_history(); }
        Some(session)
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
                HistoryItem::FileDiff(diff) => md.push_str(&format!("```diff\n{diff}\n```\n\n")),
                HistoryItem::ResponseStats { elapsed_ms, tokens, .. } => md.push_str(&format!("> {} ms · {:?} tokens\n\n", elapsed_ms, tokens)),
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
                HistoryItem::QuestionInteraction { answers, skipped, .. } => {
                    md.push_str("### ❓ Questionnaire\n\n");
                    if *skipped {
                        md.push_str("> *Skipped by user*\n\n");
                    } else {
                        for ans in answers {
                            md.push_str(&format!("- **Вопрос**: {}\n", ans.question));
                            if !ans.selected_options.is_empty() {
                                md.push_str(&format!("  - **Выбрано**: {}\n", ans.selected_options.join(", ")));
                            }
                            if let Some(ref c) = ans.custom_text {
                                if !c.trim().is_empty() {
                                    md.push_str(&format!("  - **Свой ответ**: {}\n", c.trim()));
                                }
                            }
                        }
                        md.push('\n');
                    }
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

#[cfg(test)]
#[path = "../../../tests/common/session_compat.rs"]
mod shared_format_tests;
