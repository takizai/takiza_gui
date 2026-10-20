<script setup lang="ts">
import { useI18n } from "../../i18n";
import { computed, ref } from "vue";
import { useAgentStore } from "../../stores/agentStore";
import WindowControls from "./WindowControls.vue";
import { handleWindowDrag, handleWindowDblClick } from "../../utils/window";
import { PanelLeft, MoreHorizontal, X } from "lucide-vue-next";

const { tr } = useI18n();

defineProps<{ sidebarOpen: boolean }>();
const emit = defineEmits<{
  (e: "open-settings"): void;
  (e: "open-usage"): void;
  (e: "open-skills"): void;
  (e: "toggle-sidebar"): void;
}>();
const agentStore = useAgentStore();
const menuOpen = ref(false);
const menuButton = ref<HTMLButtonElement | null>(null);
const menuError = ref("");
const saving = ref(false);
const autoApprove = computed(() => agentStore.config?.auto_approve ?? false);
async function changePreference() {
  if (saving.value) return;
  saving.value = true;
  menuError.value = "";
  try {
    await agentStore.savePreferences({ auto_approve: !autoApprove.value });
  } catch (error) { menuError.value = String(error); }
  finally { saving.value = false; }
}
</script>

<template>
  <header class="h-12 px-3 flex items-center gap-3 shrink-0 select-none border-b border-white/[0.06]"
    data-tauri-drag-region @mousedown="handleWindowDrag" @dblclick="handleWindowDblClick">
    <button @click="emit('toggle-sidebar')" class="quiet-icon" :aria-label='tr("История диалогов")' :aria-expanded="sidebarOpen">
      <PanelLeft class="w-4 h-4" />
    </button>
    <span class="text-xs text-zinc-400 truncate" data-tauri-drag-region>{{ agentStore.currentSession?.title || "Takiza" }}</span>
    <div class="flex-1 h-full" data-tauri-drag-region />
    <span v-if="agentStore.isStreaming" role="status" class="hidden sm:block text-xs text-zinc-400 truncate max-w-48">{{ agentStore.statusText }}</span>
    <span v-if="autoApprove" class="text-[11px] text-amber-400" :title='tr("Действия выполняются без подтверждения")'>Auto</span>
    <div class="relative" @keydown.esc="menuOpen = false">
      <button ref="menuButton" @click="menuOpen = !menuOpen" class="quiet-icon" :aria-label='tr("Действия и настройки")' :aria-expanded="menuOpen" aria-controls="app-actions">
        <MoreHorizontal class="w-4 h-4" />
      </button>
      <template v-if="menuOpen">
        <button class="fixed inset-0 z-30 cursor-default" :aria-label='tr("Закрыть меню")' @click="menuOpen = false" />
        <div id="app-actions" class="floating-menu absolute right-0 top-full mt-2 w-64 p-1.5 z-40">
          <div class="flex items-center justify-between px-2 py-1 text-xs text-zinc-500">{{ tr("Действия") }}<button class="quiet-icon" :aria-label='tr("Закрыть меню")' @click="menuOpen = false"><X class="w-3 h-3" /></button></div>
          <button class="action-row" disabled :title='tr("Будет доступно после появления входа в аккаунт")'>{{ tr("Режимы · MoA") }}<span>{{ tr("В разработке") }}</span></button>
          <button class="action-row" :disabled="saving || !agentStore.config" @click="changePreference()">{{ tr("Подтверждения") }}<span :class="autoApprove ? 'text-amber-400' : ''">{{ autoApprove ? tr("Выключены") : tr("Включены") }}</span></button>
          <div class="my-1 border-t border-white/[0.06]" />
          <button class="action-row" @click="menuOpen = false; menuButton?.focus(); emit('open-skills')">{{ tr("Скиллы") }}</button>
          <button class="action-row" @click="menuOpen = false; menuButton?.focus(); emit('open-usage')">Usage <span>{{ tr("В разработке") }}</span></button>
          <button class="action-row" @click="menuOpen = false; menuButton?.focus(); emit('open-settings')">{{ tr("Настройки") }}</button>
          <p v-if="menuError" role="alert" class="px-2 py-1 text-xs text-rose-300 break-words">{{ tr(menuError) }}</p>
        </div>
      </template>
    </div>
    <WindowControls />
  </header>
</template>
