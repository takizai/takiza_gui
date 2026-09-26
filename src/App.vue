<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useAgentStore } from "./stores/agentStore";
import { useSessionStore } from "./stores/sessionStore";
import { useZoom } from "./composables/useZoom";
import AppHeader from "./components/layout/AppHeader.vue";
import Sidebar from "./components/layout/Sidebar.vue";
import ChatContainer from "./components/chat/ChatContainer.vue";
import PromptInput from "./components/input/PromptInput.vue";
import PermissionModal from "./components/modals/PermissionModal.vue";
import SettingsModal from "./components/modals/SettingsModal.vue";
import UsageModal from "./components/modals/UsageModal.vue";

const agentStore = useAgentStore();
const sessionStore = useSessionStore();
const { zoomLevel, showZoomIndicator, initZoom, destroyZoom } = useZoom();

const isSettingsOpen = ref(false);
const isUsageOpen = ref(false);
const isSidebarOpen = ref(true);
const isMaximized = ref(false);

async function checkMaximized() {
  try {
    isMaximized.value = await invoke<boolean>("is_window_maximized");
  } catch (e) {
    // Fallback if not running in Tauri
  }
}

onMounted(async () => {
  initZoom();
  checkMaximized();
  window.addEventListener("resize", checkMaximized);

  await agentStore.init();
  await sessionStore.fetchSessions();
});

onUnmounted(() => {
  destroyZoom();
  window.removeEventListener("resize", checkMaximized);
});
</script>

<template>
  <div
    :data-theme="agentStore.config?.theme || 'amber'"
    class="h-screen w-screen flex bg-[#07080a] text-zinc-100 font-sans select-none app-canvas transition-all duration-150"
    :class="
      isMaximized
        ? 'rounded-none border-0'
        : 'rounded-2xl border border-white/10 shadow-[0_0_50px_rgba(0,0,0,0.8)] overflow-hidden'
    "
  >
    <!-- Left Sidebar (Full Height Desktop Standard) -->
    <Sidebar
      :is-open="isSidebarOpen"
      @open-settings="isSettingsOpen = true"
      @open-usage="isUsageOpen = true"
    />

    <!-- Main Content Area -->
    <div class="flex-1 flex flex-col min-w-0 h-full relative overflow-hidden">
      <!-- Integrated Toolbar -->
      <AppHeader
        :sidebar-open="isSidebarOpen"
        @toggle-sidebar="isSidebarOpen = !isSidebarOpen"
        @open-settings="isSettingsOpen = true"
        @open-usage="isUsageOpen = true"
      />

      <!-- Conversation Canvas -->
      <main class="flex-1 flex flex-col min-w-0 relative overflow-hidden">
        <ChatContainer />
        <PromptInput @open-settings="isSettingsOpen = true" />
      </main>
    </div>

    <!-- Floating Zoom Level Indicator Badge -->
    <div
      v-if="showZoomIndicator"
      class="fixed bottom-24 left-1/2 -translate-x-1/2 z-50 px-3.5 py-1.5 rounded-full bg-[#161820]/95 border border-white/20 text-zinc-100 text-xs font-mono shadow-2xl backdrop-blur-xl animate-in fade-in zoom-in-95 duration-150 select-none pointer-events-none"
    >
      Масштаб: {{ zoomLevel }}%
    </div>

    <!-- Modals -->
    <PermissionModal />
    <SettingsModal :is-open="isSettingsOpen" @close="isSettingsOpen = false" />
    <UsageModal :is-open="isUsageOpen" @close="isUsageOpen = false" />
  </div>
</template>