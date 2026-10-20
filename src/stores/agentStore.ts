import { defineStore } from "pinia";
import { ref, computed, watch } from "vue";
import { normalizeTheme, THEME_ACCENTS } from "../utils/theme";
import { randomThinkingMessage } from "../utils/activity";
import { hydrateSessionImages, attachmentMarkers } from "../utils/attachments";
import { invoke, Channel } from "@tauri-apps/api/core";
import type {
  AgentEvent,
  Session,
  Config,
  AppPreferences,
  GitStatus,
  UsageStats,
  AppMode,
  ThemeName,
  Language,
  Skill,
  QuestionRequestPayload,
  QuestionResponse,
} from "../types";

export const useAgentStore = defineStore("agent", () => {
  const currentSession = ref<Session | null>(null);
  const sessionHistoryVersion = ref(0);
  const sessionTitleUpdated = ref<{ session_id: string; title: string } | null>(null);
  const pendingPrompts = ref<{ id: number; input: string; imageUrls: string[] }[]>([]);
  const queuePaused = ref(false);
  const operationBusy = ref(false);
  const restoreOpen = ref(false);
  const outputExpanded = ref(false);
  let nextPromptId = 1;
  let running: Promise<void> | null = null;

  function removeQueuedPrompt(id: number) {
    pendingPrompts.value = pendingPrompts.value.filter(prompt => prompt.id !== id);
  }
  type Regeneration = { sessionId: string; historyIndex: number; edited?: string };
  function startTurn(input: string, images: string[], regeneration?: Regeneration): Promise<void> {
    const turn = runPrompt(input, images, regeneration);
    running = turn;
    void turn.finally(() => {
      if (running === turn) running = null;
      drainQueue();
    });
    return turn;
  }
  function drainQueue() {
    if (isStreaming.value || operationBusy.value || queuePaused.value) return;
    const next = pendingPrompts.value.shift();
    if (next) void startTurn(next.input, next.imageUrls);
  }
  function resumeQueue() { queuePaused.value = false; drainQueue(); }
  async function withIdleOperation<T>(operation: () => Promise<T>): Promise<T> {
    if (operationBusy.value) throw new Error("Другая операция ещё выполняется.");
    operationBusy.value = true;
    pendingPrompts.value = [];
    queuePaused.value = false;
    try {
      if (running || isStreaming.value) { await invoke("cancel_agent"); if (running) await running; }
      queuePaused.value = false;
      return await operation();
    } finally { operationBusy.value = false; }
  }
  async function openRestore() {
    await withIdleOperation(async () => {});
    restoreOpen.value = true;
  }
  async function restoreCheckpoint(id: string) {
    await withIdleOperation(async () => {
      currentSession.value = hydrateSessionImages(await invoke<Session>("restore_checkpoint", { id }));
      currentStreamAssistant.value = "";
      currentStreamThought.value = "";
      restoreOpen.value = false;
      await refreshGit();
    });
  }
  async function sendPrompt(input: string, imageUrls: string[] = []) {
    if (!input.trim()) return;
    if (operationBusy.value) throw new Error("Дождись завершения текущей операции.");
    if (!isStreaming.value && !pendingPrompts.value.length) queuePaused.value = false;
    if (isStreaming.value || queuePaused.value || pendingPrompts.value.length) {
      pendingPrompts.value.push({ id: nextPromptId++, input, imageUrls });
      drainQueue();
      return;
    }
    await startTurn(input, imageUrls);
  }
  async function regenerateMessage(historyIndex: number, edited?: string) {
    if (isStreaming.value || operationBusy.value) throw new Error("Дождитесь завершения текущего ответа.");
    const session = currentSession.value;
    if (!session || !session.history[historyIndex]) throw new Error("Сообщение не найдено.");
    let userIndex = historyIndex;
    while (userIndex >= 0 && session.history[userIndex]?.type !== "UserPrompt") userIndex--;
    const user = session.history[userIndex];
    if (!user || user.type !== "UserPrompt" || user.payload.startsWith("!")) throw new Error("Можно перегенерировать только ответ на запрос к модели.");
    let input = user.payload;
    if (edited !== undefined) {
      if (!edited.trim()) throw new Error("Введите текст сообщения.");
      const markers = attachmentMarkers(user.payload);
      input = [edited.trim(), ...markers].join("\n");
    }
    if (input.startsWith("!")) throw new Error("Перегенерация доступна только для запросов к модели.");
    pendingPrompts.value = [];
    queuePaused.value = false;
    const history = session.history.slice(0, userIndex + 1);
    history[userIndex] = { ...user, payload: input };
    currentSession.value = { ...session, history };
    await startTurn(input, user.imageUrls || [], { sessionId: session.id, historyIndex: userIndex, ...(edited !== undefined ? { edited: input } : {}) });
  }
  const isStreaming = ref(false);
  const statusText = ref("Ready");
  const currentStreamAssistant = ref("");
  const currentStreamThought = ref("");
  const streamingToolLogs = ref<string[]>([]);
  const activeToolCall = ref<{ id: string; name: string; args: string } | null>(null);
  const pendingPermission = ref<{ id: string; name: string; command: string } | null>(null);
  const pendingQuestion = ref<QuestionRequestPayload | null>(null);
  const availableSkills = ref<Skill[]>([]);
  const gitStatus = ref<GitStatus | null>(null);
  const config = ref<Config | null>(null);
  const usageStats = ref<UsageStats | null>(null);
  const currentTurnMoaModel = ref<string | null>(null);

  function applyTheme(themeName?: string | null) {
    const th = normalizeTheme(themeName);
    document.documentElement.setAttribute("data-theme", th);
  }

  const themeAccent = computed(() => THEME_ACCENTS[normalizeTheme(config.value?.theme)]);
  watch(() => config.value?.theme, applyTheme, { immediate: true, flush: "sync" });

  async function init() {
    try {
      config.value = await invoke<Config>("get_config");
    } catch (e) {
      console.error("Failed to load config:", e);
    }

    try {
      const session = await invoke<Session | null>("get_current_session");
      currentSession.value = session ? hydrateSessionImages(session) : null;
    } catch (e) {
      console.error("Failed to load current session:", e);
    }

    try {
      gitStatus.value = await invoke<GitStatus>("get_git_info");
    } catch (e) {
      console.error("Failed to load git status:", e);
    }


    try {
      await fetchSkills();
    } catch (e) {
      // Non-critical on init
    }
  }

  async function runPrompt(input: string, imageUrls: string[], regeneration?: Regeneration) {
    let failed = false;
    const thinking = randomThinkingMessage();

    isStreaming.value = true;
    statusText.value = input.startsWith("!") ? "Executing command..." : thinking;
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
    if (!regeneration) currentSession.value.history.push({
      type: "UserPrompt",
      payload: input,
      imageUrls,
    });

    const channel = new Channel<AgentEvent>();

    function commitThought() {
      if (currentStreamThought.value.trim() && currentSession.value) {
        currentSession.value.history.push({ type: "Thought", payload: currentStreamThought.value });
      }
      currentStreamThought.value = "";
    }

    channel.onmessage = (event: AgentEvent) => {
      switch (event.type) {
        case "SessionTitle":
          if (currentSession.value?.id === event.payload.session_id) currentSession.value.title = event.payload.title;
          sessionTitleUpdated.value = event.payload;
          break;
        case "StatusUpdate":
          statusText.value = event.payload === "Thinking..." ? thinking : event.payload;
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
          commitThought();
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

        case "QuestionRequest":
          pendingQuestion.value = event.payload;
          break;

        case "QuestionAnswered":
          pendingQuestion.value = null;
          if (currentSession.value) {
            currentSession.value.history.push({
              type: "QuestionInteraction",
              payload: {
                answers: event.payload.answers,
                skipped: event.payload.skipped,
              },
            });
          }
          break;

        case "ToolStart":
          commitThought();
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
          // Refresh git status after mutating tool modifications
          const mutatingTools = ["write_file", "edit_file", "edit_file_lines", "run_command", "execute_command"];
          if (mutatingTools.includes(event.payload.name)) {
            refreshGit();
          }
          break;

        case "Interrupted":
          commitThought();
          queuePaused.value = true;
          statusText.value = "Stopped";
          if (currentSession.value) currentSession.value.history.push({ type: "ToolLog", payload: "⏸ Stopped by user" });
          break;
        case "Error":
          commitThought();
          failed = true;
          queuePaused.value = true;
          if (currentSession.value) {
            currentSession.value.history.push({
              type: "Error",
              payload: event.payload,
            });
          }
          statusText.value = "Error";
          break;

        case "Finished":
          commitThought();
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
          // The backend still persists this turn after Finished. Keep the next
          // queued turn waiting until the invoke promise actually resolves.
          statusText.value = failed ? "Error" : "Finishing...";
          currentStreamAssistant.value = "";
          currentStreamThought.value = "";
          activeToolCall.value = null;
          pendingPermission.value = null;
          pendingQuestion.value = null;
          refreshGit();
          break;
      }
    };

    try {
      if (regeneration) {
        await invoke("regenerate_agent_turn", { sessionId: regeneration.sessionId, historyIndex: regeneration.historyIndex, edited: regeneration.edited ?? null, channel });
      } else await invoke("start_agent_turn", { input, channel });
      const saved = await invoke<Session | null>("get_current_session");
      if (saved) currentSession.value = hydrateSessionImages(saved);
      sessionHistoryVersion.value++;
    } catch (err: any) {
      failed = true;
      queuePaused.value = true;
      console.error("Turn invocation error:", err);
      if (regeneration && currentSession.value?.id === regeneration.sessionId) {
        try {
          const saved = await invoke<Session | null>("get_current_session");
          if (saved && saved.id === regeneration.sessionId) currentSession.value = hydrateSessionImages(saved);
        } catch { /* Keep the local response and show the original error below. */ }
      }
      if (currentSession.value) {
        currentSession.value.history.push({
          type: "Error",
          payload: String(err),
        });
      }
    } finally {
      isStreaming.value = false;
      statusText.value = failed ? "Error" : "Ready";
      pendingPermission.value = null;
      pendingQuestion.value = null;
      activeToolCall.value = null;
    }
  }

  async function stop() {
    pendingPrompts.value = [];
    queuePaused.value = true;
    pendingQuestion.value = null;
    pendingPermission.value = null;
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

  async function respondQuestion(response: QuestionResponse) {
    if (!pendingQuestion.value) return;
    const id = pendingQuestion.value.id;
    pendingQuestion.value = null;
    try {
      await invoke("respond_question", { id, response });
    } catch (e) {
      console.error("Failed to respond to question:", e);
    }
  }

  async function fetchSkills() {
    try {
      availableSkills.value = await invoke<Skill[]>("get_available_skills");
    } catch (e) {
      console.error("Failed to load available skills:", e);
      throw e;
    }
  }

  async function readSkill(name: string): Promise<string> {
    try {
      return await invoke<string>("read_skill_content", { name });
    } catch (e) {
      console.error("Failed to read skill content:", e);
      throw e;
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
    return withIdleOperation(async () => {
    try {
      config.value = await invoke<Config>("save_preferences", { prefs });
    } catch (e) {
      console.error("Failed to save preferences:", e);
      throw e;
    }
      });
  }

  async function loadSession(id: string) {
    return withIdleOperation(async () => {
    try {
      currentSession.value = hydrateSessionImages(await invoke<Session>("load_session", { id }));
    } catch (e) {
      console.error("Failed to load session:", e); throw e;
    }
      });
  }

  async function createNewSession() {
    return withIdleOperation(async () => {
    try {
      currentSession.value = hydrateSessionImages(await invoke<Session>("new_session"));
    } catch (e) {
      console.error("Failed to create new session:", e); throw e;
    }
      });
  }

  async function setWorkspaceDir(path: string) {
    return withIdleOperation(async () => {
    try {
      config.value = await invoke<Config>("set_workspace_dir", { path });
      const session = await invoke<Session | null>("get_current_session");
      currentSession.value = session ? hydrateSessionImages(session) : null;
      await refreshGit();
      await fetchSkills();
    } catch (e) {
      console.error("Failed to switch workspace dir:", e);
      throw e;
    }
      });
  }

  async function resetToHarnessDefaults() {
    return withIdleOperation(async () => {
    try {
      config.value = await invoke<Config>("reset_to_harness_defaults");
      await refreshGit();
      await fetchSkills();
    } catch (e) {
      console.error("Failed to reset to harness defaults:", e);
      throw e;
    }
      });
  }

  async function setEffort(effort: string) {
    return withIdleOperation(async () => {
    try {
      config.value = await invoke<Config>("set_effort", { effort });
    } catch (e) {
      console.error("Failed to set effort:", e);
      throw e;
    }
      });
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
    return withIdleOperation(async () => {
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
      });
  }

  async function setLanguage(language: Language) {
    config.value = await invoke<Config>("set_language", { language });
  }

  async function setTheme(theme: ThemeName) {
    return withIdleOperation(async () => {
    try {
      config.value = await invoke<Config>("set_theme", { theme });
      return config.value;
    } catch (e) {
      console.error("Failed to switch theme:", e);
      throw e;
    }
      });
  }

  return {
    currentSession, sessionTitleUpdated, sessionHistoryVersion,
    pendingPrompts, queuePaused, operationBusy, restoreOpen, outputExpanded,
    removeQueuedPrompt, resumeQueue, withIdleOperation, openRestore, restoreCheckpoint,
    isStreaming,
    statusText,
    currentStreamAssistant,
    currentStreamThought,
    streamingToolLogs,
    activeToolCall,
    pendingPermission,
    pendingQuestion,
    availableSkills,
    gitStatus,
    config,
    usageStats,
    currentTurnMoaModel,
    themeAccent,
    init,
    sendPrompt, regenerateMessage,
    stop,
    respondPermission,
    respondQuestion,
    fetchSkills,
    readSkill,
    refreshGit,
    savePreferences,
    loadSession,
    createNewSession,
    setWorkspaceDir,
    resetToHarnessDefaults,
    setEffort,
    loadUsage,
    setMode,
    setTheme, setLanguage,
  };
});
