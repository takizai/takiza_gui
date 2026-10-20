import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Session, SessionMeta } from "../types";
import { useAgentStore } from "./agentStore";

export const useSessionStore = defineStore("sessions", () => {
  const sessions = ref<SessionMeta[]>([]);
  const isLoading = ref(false);
  let generation = 0;

  async function fetchSessions() {
    const request = ++generation;
    isLoading.value = true;
    try {
      const result = await invoke<SessionMeta[]>("get_sessions");
      if (request === generation) sessions.value = result;
    } catch (e) {
      console.error("Failed to load sessions:", e);
    } finally {
      if (request === generation) isLoading.value = false;
    }
  }

  async function selectSession(id: string) {
    const agentStore = useAgentStore();
    await agentStore.loadSession(id);
    await fetchSessions();
  }

  async function newSession() {
    const agentStore = useAgentStore();
    try {
      await agentStore.createNewSession();
    } catch (e) {
      console.error("Failed to reset agent for new session:", e);
    }
    await fetchSessions();
  }

  async function deleteSession(id: string) {
    try {
      const agentStore = useAgentStore();
      const nextSession = await agentStore.withIdleOperation(() => invoke<Session | null>("delete_session", { id }));
      await fetchSessions();
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
