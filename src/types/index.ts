export interface MoaRoutingPayload {
  model: string;
  category: string;
  complexity: string;
  source?: string;
}

export type AgentEvent =
  | { type: "StatusUpdate"; payload: string }
  | { type: "MoaRouting"; payload: MoaRoutingPayload }
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
  | { type: "MoaRouting"; payload: MoaRoutingPayload }
  | { type: "Thought"; payload: string }
  | { type: "ToolStart"; payload: { name: string; args: string } }
  | { type: "ToolLog"; payload: string }
  | { type: "ToolEnd"; payload: { name: string; args: string; result: string; is_error: boolean } }
  | { type: "AssistantMessage"; payload: string; moaModel?: string }
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

export type ReasoningEffort = "low" | "medium" | "high";
export type AppMode = "manual" | "moa";
export type ThemeName = "amber" | "cyberpunk" | "emerald" | "nord" | "monochrome";

export interface CuratedModel {
  id: string;
  provider: string;
  name: string;
  description: string;
  is_expensive: boolean;
}

export interface UsageStats {
  manual_used: number;
  manual_limit: number;
  manual_percentage: number;
  moa_used: number;
  moa_limit: number;
  moa_percentage: number;
  moa_saved: number;
  active_mode: string;
  reset_time_utc: string;
}

export interface Config {
  api_key: string;
  base_url: string;
  model: string;
  workspace_dir: string;
  auto_approve: boolean;
  proxy: string | null;
  effort?: string | null;
  mode?: string | null;
  theme?: string | null;
}

export interface AppPreferences {
  api_key?: string;
  base_url?: string;
  model?: string;
  auto_approve?: boolean;
  proxy?: string;
  effort?: string;
  mode?: string;
  theme?: string;
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
