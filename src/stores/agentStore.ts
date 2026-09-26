import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke, Channel } from "@tauri-apps/api/core";
import type { AgentEvent, Session, Config, AppPreferences, GitStatus, UsageStats, AppMode, ThemeName } from "../types";

export const useAgentStore = defineStore("agent", () => {
  const currentSession = ref<Session | null>(null);
  const isStreaming = ref(false);
  const statusText = ref("Ready");
  const currentStreamAssistant = ref("");
  const currentStreamThought = ref("");
  const streamingToolLogs = ref<string[]>([]);
  const activeToolCall = ref<{ id: string; name: string; args: string } | null>(null);
  const pendingPermission = ref<{ id: string; name: string; command: string } | null>(null);
  const gitStatus = ref<GitStatus | null>(null);
  const config = ref<Config | null>(null);
  const usageStats = ref<UsageStats | null>(null);
  const currentTurnMoaModel = ref<string | null>(null);

  function applyTheme(themeName?: string | null) {
    const th = themeName || "amber";
    document.documentElement.setAttribute("data-theme", th);
  }

  async function init() {
    try {
      config.value = await invoke<Config>("get_config");
      applyTheme(config.value?.theme);
    } catch (e) {
      console.error("Failed to load config:", e);
    }

    try {
      currentSession.value = await invoke<Session | null>("get_current_session");
    } catch (e) {
      console.error("Failed to load current session:", e);
    }

    try {
      gitStatus.value = await invoke<GitStatus>("get_git_info");
    } catch (e) {
      console.error("Failed to load git status:", e);
    }

    try {
      await loadUsage();
    } catch (e) {
      // Non-critical on init
    }
  }

  async function sendPrompt(input: string) {
    if (!input.trim() || isStreaming.value) return;

    isStreaming.value = true;
    statusText.value = "Thinking...";
    currentStreamAssistant.value = "";
    currentStreamThought.value = "";
    streamingToolLogs.value = [];
    activeToolCall.value = null;
    currentTurnMoaModel.value = null;

    if (!currentSession.value) {
      currentSession.value = {
        id: new Date().toISOString(),
        created_at: new Date().toLocaleString(),
        model: config.value?.model || "AI Model",
        messages: [],
        history: [],
      };
    }

    // Add user prompt to visible history immediately
    currentSession.value.history.push({
      type: "UserPrompt",
      payload: input,
    });

    const channel = new Channel<AgentEvent>();

    channel.onmessage = (event: AgentEvent) => {
      switch (event.type) {
        case "StatusUpdate":
          statusText.value = event.payload;
          if (event.payload.includes("⚡ Выбрана модель:")) {
            const m = event.payload.match(/⚡ Выбрана модель:\s*([^\s(]+)/);
            if (m && m[1]) {
              currentTurnMoaModel.value = m[1];
            }
          }
          break;

        case "MoaRouting":
          currentTurnMoaModel.value = event.payload.model;
          if (currentSession.value) {
            const history = currentSession.value.history;
            const last = history[history.length - 1];
            if (
              !last ||
              last.type !== "MoaRouting" ||
              last.payload.model !== event.payload.model
            ) {
              history.push({
                type: "MoaRouting",
                payload: {
                  model: event.payload.model,
                  category: event.payload.category,
                  complexity: event.payload.complexity,
                  source: event.payload.source,
                },
              });
            }
          }
          break;

        case "AssistantToken":
          currentStreamAssistant.value += event.payload;
          break;

        case "ThoughtToken":
          currentStreamThought.value += event.payload;
          break;

        case "AssistantThought":
          if (currentSession.value && event.payload) {
            currentSession.value.history.push({
              type: "Thought",
              payload: event.payload,
            });
            currentStreamThought.value = "";
          }
          break;

        case "AssistantMessage":
          if (currentSession.value) {
            const history = currentSession.value.history;
            const last = history[history.length - 1];
            if (
              !last ||
              last.type !== "AssistantMessage" ||
              last.payload.trim() !== event.payload.trim()
            ) {
              history.push({
                type: "AssistantMessage",
                payload: event.payload,
                moaModel: currentTurnMoaModel.value || undefined,
              });
            }
            currentStreamAssistant.value = "";
          }
          break;

        case "PermissionRequest":
          pendingPermission.value = event.payload;
          break;

        case "ToolStart":
          // Flush any pending streamed assistant text before the tool card
          if (currentStreamAssistant.value.trim() && currentSession.value) {
            currentSession.value.history.push({
              type: "AssistantMessage",
              payload: currentStreamAssistant.value.trim(),
              moaModel: currentTurnMoaModel.value || undefined,
            });
            currentStreamAssistant.value = "";
          }
          activeToolCall.value = event.payload;
          streamingToolLogs.value = [];
          break;

        case "ToolLog":
          streamingToolLogs.value.push(event.payload);
          break;

        case "ToolEnd":
          activeToolCall.value = null;
          if (currentSession.value) {
            currentSession.value.history.push({
              type: "ToolEnd",
              payload: {
                name: event.payload.name,
                args: event.payload.args,
                result: event.payload.result,
                is_error: event.payload.is_error,
              },
            });
          }
          // Refresh git status after tool modifications
          refreshGit();
          break;

        case "Error":
          if (currentSession.value) {
            currentSession.value.history.push({
              type: "Error",
              payload: event.payload,
            });
          }
          statusText.value = "Error";
          break;

        case "Finished":
          // Commit any trailing streamed assistant tokens
          if (currentStreamAssistant.value.trim() && currentSession.value) {
            const history = currentSession.value.history;
            const last = history[history.length - 1];
            if (
              !last ||
              last.type !== "AssistantMessage" ||
              last.payload.trim() !== currentStreamAssistant.value.trim()
            ) {
              history.push({
                type: "AssistantMessage",
                payload: currentStreamAssistant.value.trim(),
                moaModel: currentTurnMoaModel.value || undefined,
              });
            }
          }
          isStreaming.value = false;
          statusText.value = "Ready";
          currentStreamAssistant.value = "";
          currentStreamThought.value = "";
          activeToolCall.value = null;
          pendingPermission.value = null;
          refreshGit();
          break;
      }
    };

    try {
      await invoke("start_agent_turn", { input, channel });
    } catch (err: any) {
      console.error("Turn invocation error:", err);
      if (currentSession.value) {
        currentSession.value.history.push({
          type: "Error",
          payload: String(err),
        });
      }
    } finally {
      isStreaming.value = false;
      statusText.value = "Ready";
      pendingPermission.value = null;
      activeToolCall.value = null;
    }
  }

  async function stop() {
    try {
      await invoke("cancel_agent");
    } catch (e) {
      console.error("Failed to cancel agent:", e);
    }
  }

  async function respondPermission(decision: "AllowOnce" | "AllowAlways" | "Deny") {
    if (!pendingPermission.value) return;
    const id = pendingPermission.value.id;
    pendingPermission.value = null;
    try {
      await invoke("respond_permission", { id, decision });
    } catch (e) {
      console.error("Failed to respond to permission:", e);
    }
  }

  async function refreshGit() {
    try {
      gitStatus.value = await invoke<GitStatus>("get_git_info");
    } catch (e) {
      console.error("Failed to refresh git status:", e);
    }
  }

  async function savePreferences(prefs: AppPreferences) {
    try {
      config.value = await invoke<Config>("save_preferences", { prefs });
    } catch (e) {
      console.error("Failed to save preferences:", e);
      throw e;
    }
  }

  async function loadSession(id: string) {
    try {
      currentSession.value = await invoke<Session>("load_session", { id });
    } catch (e) {
      console.error("Failed to load session:", e);
    }
  }

  async function createNewSession() {
    try {
      currentSession.value = await invoke<Session>("new_session");
    } catch (e) {
      console.error("Failed to create new session:", e);
    }
  }

  async function setWorkspaceDir(path: string) {
    try {
      config.value = await invoke<Config>("set_workspace_dir", { path });
      currentSession.value = await invoke<Session | null>("get_current_session");
      await refreshGit();
    } catch (e) {
      console.error("Failed to switch workspace dir:", e);
      throw e;
    }
  }

  async function resetToHarnessDefaults() {
    try {
      config.value = await invoke<Config>("reset_to_harness_defaults");
      await refreshGit();
    } catch (e) {
      console.error("Failed to reset to harness defaults:", e);
      throw e;
    }
  }

  async function setEffort(effort: "low" | "medium" | "high") {
    try {
      config.value = await invoke<Config>("set_effort", { effort });
    } catch (e) {
      console.error("Failed to set effort:", e);
      throw e;
    }
  }

  async function loadUsage() {
    try {
      const stats = await invoke<UsageStats>("get_usage");
      usageStats.value = stats;
      return stats;
    } catch (e) {
      console.error("Failed to load usage stats:", e);
      throw e;
    }
  }

  async function setMode(mode: AppMode) {
    try {
      config.value = await invoke<Config>("set_mode", { mode });
      if (usageStats.value) {
        usageStats.value.active_mode = mode;
      }
      return config.value;
    } catch (e) {
      console.error("Failed to switch mode:", e);
      throw e;
    }
  }

  async function setTheme(theme: ThemeName) {
    try {
      config.value = await invoke<Config>("set_theme", { theme });
      applyTheme(theme);
      return config.value;
    } catch (e) {
      console.error("Failed to switch theme:", e);
      throw e;
    }
  }

  return {
    currentSession,
    isStreaming,
    statusText,
    currentStreamAssistant,
    currentStreamThought,
    streamingToolLogs,
    activeToolCall,
    pendingPermission,
    gitStatus,
    config,
    usageStats,
    currentTurnMoaModel,
    init,
    sendPrompt,
    stop,
    respondPermission,
    refreshGit,
    savePreferences,
    loadSession,
    createNewSession,
    setWorkspaceDir,
    resetToHarnessDefaults,
    setEffort,
    loadUsage,
    setMode,
    setTheme,
  };
});
