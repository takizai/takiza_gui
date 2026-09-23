export type AgentEvent =
  | { type: "StatusUpdate"; payload: string }
  | { type: "UserMessage"; payload: string }
  | { type: "AssistantMessage"; payload: string }
  | { type: "AssistantThought"; payload: string }
  | { type: "AssistantToken"; payload: string }
  | { type: "ThoughtToken"; payload: string }
  | { type: "PermissionRequest"; payload: { id: string; name: string; command: string } }
  | { type: "ToolStart"; payload: { id: string; name: string; args: string } }
  | { type: "ToolLog"; payload: string }
  | { type: "ToolEnd"; payload: { id: string; name: string; args: string; result: string; is_error: boolean } }
  | { type: "Error"; payload: string }
  | { type: "Finished"; payload?: null };

export type HistoryItem =
  | { type: "UserPrompt"; payload: string }
  | { type: "Thought"; payload: string }
  | { type: "ToolStart"; payload: { name: string; args: string } }
  | { type: "ToolLog"; payload: string }
  | { type: "ToolEnd"; payload: { name: string; args: string; result: string; is_error: boolean } }
  | { type: "AssistantMessage"; payload: string }
  | { type: "Error"; payload: string };

export interface Session {
  id: string;
  created_at: string;
  model: string;
  title?: string;
  messages: any[];
  history: HistoryItem[];
}

export interface SessionMeta {
  id: string;
  created_at: string;
  model: string;
  title: string;
  message_count: number;
}

export interface Config {
  api_key: string;
  base_url: string;
  model: string;
  workspace_dir: string;
  auto_approve: boolean;
  proxy: string | null;
}

export interface AppPreferences {
  api_key?: string;
  base_url?: string;
  model?: string;
  auto_approve?: boolean;
  proxy?: string;
}

export interface GitStatus {
  branch: string | null;
  is_dirty: boolean;
  diff: string | null;
}

export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number | null;
}
