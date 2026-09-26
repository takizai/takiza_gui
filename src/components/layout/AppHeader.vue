<script setup lang="ts">
import { ref, computed } from "vue";
import { useAgentStore } from "../../stores/agentStore";
import WindowControls from "./WindowControls.vue";
import { handleWindowDrag, handleWindowDblClick } from "../../utils/window";
import ModelContextMenu from "../common/ModelContextMenu.vue";
import {
  Settings,
  ShieldCheck,
  ShieldAlert,
  PanelLeftClose,
  PanelLeftOpen,
  ChevronDown,
  Gauge,
  Zap,
} from "lucide-vue-next";

const props = defineProps<{
  sidebarOpen: boolean;
}>();

const emit = defineEmits<{
  (e: "open-settings"): void;
  (e: "open-usage"): void;
  (e: "toggle-sidebar"): void;
}>();

const agentStore = useAgentStore();
const isModelMenuOpen = ref(false);

const isMoa = computed(() => agentStore.config?.mode === "moa");

const currentModelShort = computed(() => {
  const m = agentStore.config?.model || "Takiza AI";
  return m.split("/").pop() || m;
});

const autoApprove = computed(() => agentStore.config?.auto_approve ?? false);

async function toggleMode() {
  const next = isMoa.value ? "manual" : "moa";
  try {
    await agentStore.setMode(next);
  } catch (e) {
    console.error(e);
  }
}

async function toggleAutoApprove() {
  if (!agentStore.config) return;
  try {
    await agentStore.savePreferences({
      auto_approve: !autoApprove.value,
    });
  } catch (e) {
    console.error(e);
  }
}
</script>

<template>
  <header
    class="h-11 border-b border-white/[0.06] bg-[#07080a]/95 backdrop-blur-xl px-3 flex items-center justify-between z-10 shrink-0 select-none cursor-default"
    data-tauri-drag-region
    @mousedown="handleWindowDrag"
    @dblclick="handleWindowDblClick"
  >
    <!-- Left: Sidebar toggle & Session title -->
    <div class="flex items-center gap-2.5 min-w-0" data-tauri-drag-region>
      <button
        @click="emit('toggle-sidebar')"
        class="p-1.5 rounded-lg text-zinc-400 hover:text-zinc-100 hover:bg-white/[0.06] transition-colors cursor-pointer shrink-0"
        :title="sidebarOpen ? 'Скрыть панель сессий' : 'Показать панель сессий'"
      >
        <PanelLeftClose v-if="sidebarOpen" class="w-4 h-4" />
        <PanelLeftOpen v-else class="w-4 h-4" />
      </button>

      <div class="h-3.5 w-px bg-white/[0.08]" />

      <!-- Active Session Title (Draggable) -->
      <div class="flex items-center gap-2 min-w-0" data-tauri-drag-region>
        <span class="text-xs font-medium text-zinc-300 truncate max-w-[200px] sm:max-w-[320px]">
          {{ agentStore.currentSession?.title || "Новый диалог" }}
        </span>
      </div>
    </div>

    <!-- Center Drag Region Spacer -->
    <div class="flex-1 h-full min-w-4" data-tauri-drag-region />

    <!-- Right Controls: Model Pill, Safe Mode, Status, Settings, Custom Window Controls -->
    <div class="flex items-center gap-2 shrink-0">
      <!-- Status Indicator -->
      <div
        class="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[11px] font-medium border transition-colors"
        :class="
          agentStore.isStreaming
            ? 'bg-amber-500/10 text-amber-300 border-amber-500/30'
            : 'bg-white/[0.03] text-zinc-400 border-white/[0.06]'
        "
      >
        <l-tailspin v-if="agentStore.isStreaming" size="12" stroke="2" speed="0.9" color="#f59e0b"></l-tailspin>
        <span
          v-else
          class="w-1.5 h-1.5 rounded-full bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.6)]"
        />
        <span class="hidden sm:inline font-mono">{{ agentStore.statusText }}</span>
      </div>

      <!-- Model Picker with Context Menu Popover (hidden in MoA mode) -->
      <div v-if="!isMoa" class="relative">
        <button
          @click="isModelMenuOpen = !isModelMenuOpen"
          @contextmenu.prevent="isModelMenuOpen = true"
          class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs font-mono bg-white/[0.03] hover:bg-white/[0.07] border border-white/[0.08] text-zinc-300 hover:text-white transition-all cursor-pointer group"
          :title="`Текущая модель: ${agentStore.config?.model}. Нажмите для выбора из меню`"
        >
          <span class="w-1.5 h-1.5 rounded-full bg-amber-400/90 group-hover:scale-125 transition-transform" />
          <span class="max-w-[110px] sm:max-w-[150px] truncate text-[11px]">{{ currentModelShort }}</span>
          <ChevronDown class="w-3 h-3 text-zinc-500 group-hover:text-zinc-300 transition-colors" />
        </button>

        <ModelContextMenu
          :is-open="isModelMenuOpen"
          direction="down"
          align="right"
          @close="isModelMenuOpen = false"
          @open-settings="emit('open-settings')"
        />
      </div>

      <!-- Auto Approve Toggle -->
      <button
        @click="toggleAutoApprove"
        class="flex items-center gap-1.5 px-2 py-1 rounded-lg text-xs font-medium border transition-all cursor-pointer"
        :class="
          autoApprove
            ? 'bg-amber-500/10 border-amber-500/30 text-amber-300 hover:bg-amber-500/20'
            : 'bg-white/[0.02] border-white/[0.06] text-zinc-400 hover:text-zinc-200 hover:bg-white/[0.05]'
        "
        :title="autoApprove ? 'Auto-Approve активен' : 'Safe Mode'"
      >
        <ShieldAlert v-if="autoApprove" class="w-3.5 h-3.5 text-amber-400" />
        <ShieldCheck v-else class="w-3.5 h-3.5 text-zinc-400" />
        <span class="hidden md:inline text-[11px]">{{ autoApprove ? "Auto" : "Safe" }}</span>
      </button>

      <!-- Takiza Mode Switcher Button (Manual / MoA) -->
      <button
        @click="toggleMode"
        class="flex items-center gap-1.5 px-2 py-1 rounded-lg text-xs font-medium border transition-all cursor-pointer"
        :class="
          isMoa
            ? 'bg-emerald-500/15 border-emerald-500/35 text-emerald-300 hover:bg-emerald-500/25'
            : 'bg-white/[0.02] border-white/[0.06] text-zinc-400 hover:text-zinc-200 hover:bg-white/[0.05]'
        "
        :title="isMoa ? 'Takiza MoA: авто-роутинг (~45% экономии токенов). Нажмите для переключения на Manual' : 'Takiza Manual: ручной выбор модели. Нажмите для переключения на MoA'"
      >
        <Zap v-if="isMoa" class="w-3.5 h-3.5 text-emerald-400 fill-current" />
        <span class="text-[11px] font-mono">{{ isMoa ? "MoA" : "Manual" }}</span>
      </button>

      <!-- Usage Quota Dashboard Button -->
      <button
        @click="emit('open-usage')"
        class="p-1.5 rounded-lg text-zinc-400 hover:text-zinc-100 hover:bg-white/[0.06] transition-colors border border-transparent hover:border-white/[0.06] cursor-pointer"
        title="Лимиты и расход токенов (Usage Quotas)"
      >
        <Gauge class="w-4 h-4" />
      </button>

      <!-- Settings Icon -->
      <button
        @click="emit('open-settings')"
        class="p-1.5 rounded-lg text-zinc-400 hover:text-zinc-100 hover:bg-white/[0.06] transition-colors border border-transparent hover:border-white/[0.06] cursor-pointer"
        title="Настройки"
      >
        <Settings class="w-4 h-4" />
      </button>

      <!-- Divider -->
      <div class="h-4 w-px bg-white/[0.08] mx-0.5" />

      <!-- Custom Window Controls (_ □ ✕) -->
      <WindowControls />
    </div>
  </header>
</template>
