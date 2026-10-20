<script setup lang="ts">
import { useI18n } from "./i18n";
import { ref, onMounted, onUnmounted, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useAgentStore } from "./stores/agentStore";
import { useSessionStore } from "./stores/sessionStore";
import { useZoom } from "./composables/useZoom";
import AppHeader from "./components/layout/AppHeader.vue";
import Sidebar from "./components/layout/Sidebar.vue";
import ChatContainer from "./components/chat/ChatContainer.vue";
import PromptInput from "./components/input/PromptInput.vue";
import PermissionModal from "./components/modals/PermissionModal.vue";
import QuestionPanel from "./components/input/QuestionPanel.vue";
import SkillsModal from "./components/modals/SkillsModal.vue";
import SettingsModal from "./components/modals/SettingsModal.vue";
import UsageModal from "./components/modals/UsageModal.vue";
import RestoreModal from "./components/modals/RestoreModal.vue";

const { tr, locale, setLocale } = useI18n();
setLocale(locale.value);

const agentStore = useAgentStore();
const sessionStore = useSessionStore();
watch([() => agentStore.sessionTitleUpdated, () => agentStore.sessionHistoryVersion], () => { void sessionStore.fetchSessions(); });
const { zoomLevel, showZoomIndicator, initZoom, destroyZoom } = useZoom();

const isWelcome = computed(() => !agentStore.isStreaming && !agentStore.currentSession?.history.some(item =>
  item.type === "UserPrompt" || item.type === "AssistantMessage" || item.type === "Error" || item.type === "ToolEnd" || item.type === "QuestionInteraction"));

watch(() => agentStore.config?.language, language => { if (language) setLocale(language); }, { immediate: true });

const isSettingsOpen = ref(false);
const isUsageOpen = ref(false);
const isSkillsOpen = ref(false);
// Mount each panel on first use, then retain it for closing animations.
// Bundle dialog code with the app so opening never depends on a later chunk fetch.
const settingsLoaded = ref(false);
const skillsLoaded = ref(false);
const usageLoaded = ref(false);
watch(isSettingsOpen, open => { if (open) settingsLoaded.value = true; });
watch(isSkillsOpen, open => { if (open) skillsLoaded.value = true; });
watch(isUsageOpen, open => { if (open) usageLoaded.value = true; });
const isSidebarOpen = ref(false);
const isMaximized = ref(false);
const promptInputRef = ref<InstanceType<typeof PromptInput> | null>(null);

function toggleOutput(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "o") {
    event.preventDefault(); agentStore.outputExpanded = !agentStore.outputExpanded;
  }
}
function refreshSessionHistory() { void sessionStore.fetchSessions(); }

async function checkMaximized() {
  try {
    isMaximized.value = await invoke<boolean>("is_window_maximized");
  } catch (e) {
    // Fallback if not running in Tauri
  }
}

onMounted(async () => {
  initZoom();
  window.addEventListener("keydown", toggleOutput);
  window.addEventListener("focus", refreshSessionHistory);
  checkMaximized();
  window.addEventListener("resize", checkMaximized);

  await agentStore.init();
  await sessionStore.fetchSessions();
});

onUnmounted(() => {
  destroyZoom();
  window.removeEventListener("keydown", toggleOutput);
  window.removeEventListener("focus", refreshSessionHistory);
  window.removeEventListener("resize", checkMaximized);
});
</script>

<template>
  <div
    class="relative h-screen w-screen flex bg-[#07080a] text-zinc-100 font-sans select-none app-canvas"
    :class="
      isMaximized
        ? 'rounded-none border-0'
        : 'rounded-xl border border-white/10 overflow-hidden'
    "
  >
    <!-- Left Sidebar (Full Height Desktop Standard) -->
    <Sidebar
      :is-open="isSidebarOpen"
      @close="isSidebarOpen = false"
    />

    <Transition name="history-scrim">
      <button v-if="isSidebarOpen" class="sidebar-scrim" :aria-label='tr("Закрыть историю")' @click="isSidebarOpen = false" />
    </Transition>

    <!-- Main Content Area -->
    <div class="flex-1 flex flex-col min-w-0 h-full relative overflow-hidden">
      <!-- Integrated Toolbar -->
      <AppHeader
        :sidebar-open="isSidebarOpen"
        @toggle-sidebar="isSidebarOpen = !isSidebarOpen"
        @open-settings="isSettingsOpen = true"
        @open-usage="isUsageOpen = true"
        @open-skills="isSkillsOpen = true"
      />

      <!-- Conversation Canvas -->
      <main class="flex-1 flex flex-col min-w-0 relative overflow-hidden">
        <ChatContainer v-show="!isWelcome" />
        <QuestionPanel />
        <div :class="isWelcome ? 'welcome-composer flex-1 flex flex-col justify-center w-full min-h-0 overflow-y-auto' : 'shrink-0'">
          <div v-if="isWelcome" class="w-full max-w-2xl mx-auto px-6 mb-7">
            <h1 class="text-2xl sm:text-3xl font-medium tracking-tight text-zinc-200 mb-3">{{ tr("С чего начнём?") }}</h1>
            <p class="text-sm text-zinc-500 leading-relaxed">{{ tr("Опишите задачу для вашего проекта.") }}</p>
          </div>
          <PromptInput
            ref="promptInputRef"
            :welcome="isWelcome"
            @open-settings="isSettingsOpen = true"
            @open-skills="isSkillsOpen = true"
            @open-usage="isUsageOpen = true"
          />
        </div>
      </main>
    </div>

    <!-- Floating Zoom Level Indicator Badge -->
    <div
      v-if="showZoomIndicator"
      class="fixed bottom-24 left-1/2 -translate-x-1/2 z-50 px-3.5 py-1.5 rounded-full bg-[#161820]/95 border border-white/20 text-zinc-100 text-xs font-mono shadow-2xl backdrop-blur-xl animate-in fade-in zoom-in-95 duration-150 select-none pointer-events-none"
    >{{ tr("Масштаб:") }} {{ zoomLevel }}%
    </div>

    <!-- Modals -->
    <PermissionModal />
    <RestoreModal />
    <SkillsModal v-if="isSkillsOpen || skillsLoaded"
      :is-open="isSkillsOpen"
      @close="isSkillsOpen = false"
      @insert-skill="(name) => promptInputRef?.insertSkill(name)"
    />
    <SettingsModal v-if="isSettingsOpen || settingsLoaded" :is-open="isSettingsOpen" @close="isSettingsOpen = false" />
    <UsageModal v-if="isUsageOpen || usageLoaded" :is-open="isUsageOpen" @close="isUsageOpen = false" />
  </div>
</template>
