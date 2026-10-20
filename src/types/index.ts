export interface MoaRoutingPayload {
  model: string;
  category: string;
  complexity: string;
  source?: string;
}

export interface Skill {
  name: string;
  description: string;
  path: string;
  directory: string;
  is_workspace: boolean;
}

export interface QuestionItem {
  question: string;
  header?: string;
  options?: string[];
  is_multi_select?: boolean;
  allow_custom?: boolean;
  placeholder?: string;
}

export interface QuestionAnswer {
  question_index: number;
  question: string;
  selected_options: string[];
  custom_text?: string;
}

export interface QuestionResponse {
  answers: QuestionAnswer[];
  skipped: boolean;
}

export interface QuestionRequestPayload {
  id: string;
  questions: QuestionItem[];
}

export type AgentEvent =
  | { type: "SessionTitle"; payload: { session_id: string; title: string } }
  | { type: "StatusUpdate"; payload: string }
  | { type: "MoaRouting"; payload: MoaRoutingPayload }
  | { type: "UserMessage"; payload: string }
  | { type: "AssistantMessage"; payload: string }
  | { type: "AssistantThought"; payload: string }
  | { type: "AssistantToken"; payload: string }
  | { type: "ThoughtToken"; payload: string }
  | { type: "PermissionRequest"; payload: { id: string; name: string; command: string } }
  | { type: "QuestionRequest"; payload: QuestionRequestPayload }
  | { type: "QuestionAnswered"; payload: { id: string; answers: QuestionAnswer[]; skipped: boolean } }
  | { type: "ToolStart"; payload: { id: string; name: string; args: string } }
  | { type: "ToolLog"; payload: string }
  | { type: "ToolEnd"; payload: { id: string; name: string; args: string; result: string; is_error: boolean } }
  | { type: "Error"; payload: string }
  | { type: "Interrupted"; payload?: null }
  | { type: "Finished"; payload?: null };

export type HistoryItem =
  | { type: "FileDiff"; payload: string }
  | { type: "ResponseStats"; payload: { elapsed_ms: number; tokens?: number | null; usage_complete: boolean } }
  | { type: "UserPrompt"; payload: string; imageUrls?: string[] }
  | { type: "MoaRouting"; payload: MoaRoutingPayload }
  | { type: "Thought"; payload: string }
  | { type: "ToolStart"; payload: { name: string; args: string } }
  | { type: "ToolLog"; payload: string }
  | { type: "ToolEnd"; payload: { name: string; args: string; result: string; is_error: boolean } }
  | {
      type: "QuestionInteraction";
      payload: {
        questions?: QuestionItem[];
        answers: QuestionAnswer[];
        skipped: boolean;
      };
    }
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

export type ReasoningEffort = "none" | "minimal" | "low" | "medium" | "high" | "xhigh" | "max";
export type AppMode = "manual" | "moa";
export type ThemeName = "amber" | "cyberpunk" | "emerald" | "nord" | "monochrome" | "dracula" | "rose" | "ocean" | "sakura" | "solarized";

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

export interface EffortCapability {
  supported?: boolean;
  levels?: string[];
  [level: string]: boolean | string[] | {supported?: boolean} | undefined;
}

export interface ProviderModel {
  id: string;
  description?: string;
  name?: string;
  architecture?: { input_modalities?: string[] };
  input_modalities?: string[];
  modalities?: { input?: string[] };
  capabilities?: { vision?: boolean; image_input?: boolean; effort?: EffortCapability; reasoning_effort?: boolean | string[] };
  reasoning_efforts?: string[];
  reasoning_effort?: EffortCapability;
  supported_parameters?: string[];
}

export type ApiType = 'chat_completions' | 'openai_responses' | 'anthropic_messages';
export type CredentialSource = 'key' | 'env' | 'none';
export interface HeaderEntry { name: string; value: string }

export interface ProviderProfile {
  name: string;
  base_url: string;
  model: string;
  api_key?: string | null;
  api_key_env?: string | null;
  api_type?: ApiType;
  headers?: Record<string, string>;
  models?: ProviderModel[];
}

export type Language = "ru" | "en" | "zh";

export interface Config {
  api_type?: ApiType;
  api_key_env?: string | null;
  provider_headers?: Record<string, string>;
  language?: string | null;
  active_provider?: string | null;
  providers?: Record<string, ProviderProfile>;
  api_key: string;
  base_url: string;
  model: string;
  workspace_dir: string;
  auto_approve: boolean;
  proxy: string | null;
  effort?: string | null;
  mode?: string | null;
  theme?: string | null;
  max_steps?: number;
}

export interface AppPreferences {
  model_catalog?: ProviderModel[];
  api_type?: ApiType;
  api_key_env?: string;
  headers?: Record<string, string>;
  new_provider_name?: string;
  language?: Language;
  active_provider?: string;
  max_steps?: number;
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

export interface ImageAttachment { marker: string; url: string; name: string }
export interface FileAttachment { marker: string; name: string; size: number }
export interface AttachmentImport { text: string; images: ImageAttachment[] }
export interface CheckpointSummary { id: string; created_at: string; label: string }
