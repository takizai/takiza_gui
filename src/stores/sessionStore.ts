import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { SessionMeta } from "../types";
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
    await agentStore.createNewSession();
    await fetchSessions();
  }

  async function deleteSession(id: string) {
    try {
      await invoke("delete_session", { id });
      await fetchSessions();
      const agentStore = useAgentStore();
      if (agentStore.currentSession?.id === id) {
        await agentStore.createNewSession();
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
