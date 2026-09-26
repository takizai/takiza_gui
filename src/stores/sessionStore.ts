import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Session, SessionMeta } from "../types";
import { useAgentStore } from "./agentStore";

export const useSessionStore = defineStore("sessions", () => {
  const sessions = ref<SessionMeta[]>([]);
  const isLoading = ref(false);

  async function fetchSessions() {
    isLoading.value = true;
    try {
      sessions.value = await invoke<SessionMeta[]>("get_sessions");
    } catch (e) {
      console.error("Failed to load sessions:", e);
    } finally {
      isLoading.value = false;
    }
  }

  async function selectSession(id: string) {
    const agentStore = useAgentStore();
    await agentStore.loadSession(id);
    await fetchSessions();
  }

  async function newSession() {
    const agentStore = useAgentStore();
    agentStore.currentSession = null;
    try {
      await invoke("new_session");
    } catch (e) {
      console.error("Failed to reset agent for new session:", e);
    }
    await fetchSessions();
  }

  async function deleteSession(id: string) {
    try {
      const nextSession = await invoke<Session | null>("delete_session", { id });
      await fetchSessions();
      const agentStore = useAgentStore();
      if (agentStore.currentSession?.id === id) {
        agentStore.currentSession = nextSession;
      }
    } catch (e) {
      console.error("Failed to delete session:", e);
    }
  }

  return {
    sessions,
    isLoading,
    fetchSessions,
    selectSession,
    newSession,
    deleteSession,
  };
});
