import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke, Channel } from "@tauri-apps/api/core";
import type { AgentEvent, Session, Config, AppPreferences, GitStatus } from "../types";

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

  async function init() {
    try {
      config.value = await invoke<Config>("get_config");
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
  }

  async function sendPrompt(input: string) {
    if (!input.trim() || isStreaming.value) return;

    isStreaming.value = true;
    statusText.value = "Thinking...";
    currentStreamAssistant.value = "";
    currentStreamThought.value = "";
    streamingToolLogs.value = [];
    activeToolCall.value = null;

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
            currentSession.value.history.push({
              type: "AssistantMessage",
              payload: event.payload,
            });
            currentStreamAssistant.value = "";
          }
          break;

        case "PermissionRequest":
          pendingPermission.value = event.payload;
          break;

        case "ToolStart":
          activeToolCall.value = event.payload;
          streamingToolLogs.value = [];
          if (currentSession.value) {
            currentSession.value.history.push({
              type: "ToolStart",
              payload: {
                name: event.payload.name,
                args: event.payload.args,
              },
            });
          }
          break;

        case "ToolLog":
          streamingToolLogs.value.push(event.payload);
          if (currentSession.value) {
            currentSession.value.history.push({
              type: "ToolLog",
              payload: event.payload,
            });
          }
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
    init,
    sendPrompt,
    stop,
    respondPermission,
    refreshGit,
    savePreferences,
    loadSession,
    createNewSession,
  };
});
