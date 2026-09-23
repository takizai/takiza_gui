<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useAgentStore } from "./stores/agentStore";
import { useSessionStore } from "./stores/sessionStore";
import AppHeader from "./components/layout/AppHeader.vue";
import Sidebar from "./components/layout/Sidebar.vue";
import ChatContainer from "./components/chat/ChatContainer.vue";
import PromptInput from "./components/input/PromptInput.vue";
import PermissionModal from "./components/modals/PermissionModal.vue";
import SettingsModal from "./components/modals/SettingsModal.vue";

const agentStore = useAgentStore();
const sessionStore = useSessionStore();

const isSettingsOpen = ref(false);

onMounted(async () => {
  await agentStore.init();
  await sessionStore.fetchSessions();
});
</script>

<template>
  <div class="h-screen w-screen flex flex-col bg-[#0c0d0e] text-neutral-100 overflow-hidden font-sans">
    <!-- Header -->
    <AppHeader @open-settings="isSettingsOpen = true" />

    <!-- Main Workspace Area -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Left Sidebar (Sessions) -->
      <Sidebar />

      <!-- Center Chat Area -->
      <main class="flex-1 flex flex-col min-w-0 bg-[#0c0d0e]/60 relative">
        <ChatContainer />
        <PromptInput />
      </main>
    </div>

    <!-- Modals -->
    <PermissionModal />
    <SettingsModal :is-open="isSettingsOpen" @close="isSettingsOpen = false" />
  </div>
</template>