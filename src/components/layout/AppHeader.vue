<script setup lang="ts">
import { computed } from "vue";
import { useAgentStore } from "../../stores/agentStore";
import {
  Sparkles,
  GitBranch,
  Settings,
  ShieldCheck,
  ShieldAlert,
  Loader2,
  Terminal,
} from "lucide-vue-next";

const emit = defineEmits<{
  (e: "open-settings"): void;
}>();

const agentStore = useAgentStore();

const isDirty = computed(() => agentStore.gitStatus?.is_dirty);
const branchName = computed(() => agentStore.gitStatus?.branch || "no-git");
const currentModel = computed(() => agentStore.config?.model || "Takiza AI");
const autoApprove = computed(() => agentStore.config?.auto_approve ?? false);

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
    class="h-14 border-b border-neutral-800 bg-neutral-900/60 backdrop-blur-md px-4 flex items-center justify-between z-20 shrink-0"
  >
    <!-- Brand / Title -->
    <div class="flex items-center gap-3">
      <div
        class="w-8 h-8 rounded-lg bg-gradient-to-tr from-cyan-600 to-blue-500 flex items-center justify-center shadow-lg shadow-cyan-500/20"
      >
        <Sparkles class="w-4 h-4 text-white" />
      </div>
      <div>
        <div class="flex items-center gap-2">
          <span class="font-bold text-sm tracking-wide text-neutral-100">Takiza</span>
          <span class="text-xs px-1.5 py-0.5 rounded bg-cyan-950 text-cyan-400 border border-cyan-800/60 font-mono">
            GUI
          </span>
        </div>
        <p class="text-[10px] text-neutral-400 font-mono truncate max-w-[200px]">
          {{ agentStore.config?.workspace_dir || "Workspace" }}
        </p>
      </div>
    </div>

    <!-- Center status -->
    <div class="flex items-center gap-3">
      <!-- Git Branch -->
      <div
        class="hidden sm:flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-mono bg-neutral-800/80 border border-neutral-700/60 text-neutral-300"
      >
        <GitBranch class="w-3.5 h-3.5 text-neutral-400" />
        <span>{{ branchName }}</span>
        <span
          v-if="isDirty"
          class="w-1.5 h-1.5 rounded-full bg-amber-400 animate-pulse"
          title="Modified files"
        />
      </div>

      <!-- Agent Status Indicator -->
      <div
        class="flex items-center gap-2 px-3 py-1 rounded-full text-xs font-medium border"
        :class="
          agentStore.isStreaming
            ? 'bg-blue-950/60 text-blue-300 border-blue-800/80'
            : 'bg-neutral-800/50 text-neutral-400 border-neutral-700/40'
        "
      >
        <Loader2 v-if="agentStore.isStreaming" class="w-3.5 h-3.5 animate-spin text-blue-400" />
        <span
          v-else
          class="w-2 h-2 rounded-full bg-emerald-500"
        />
        <span>{{ agentStore.statusText }}</span>
      </div>
    </div>

    <!-- Right Controls -->
    <div class="flex items-center gap-2.5">
      <!-- Model Badge -->
      <div
        class="hidden md:flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs font-mono bg-neutral-800/60 border border-neutral-700/60 text-neutral-300 max-w-[220px] truncate"
        :title="currentModel"
      >
        <Terminal class="w-3.5 h-3.5 text-cyan-400 shrink-0" />
        <span class="truncate">{{ currentModel }}</span>
      </div>

      <!-- Auto Approve Toggle -->
      <button
        @click="toggleAutoApprove"
        class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs font-medium border transition-colors cursor-pointer"
        :class="
          autoApprove
            ? 'bg-amber-950/40 border-amber-800/60 text-amber-300 hover:bg-amber-900/50'
            : 'bg-neutral-800/40 border-neutral-700/50 text-neutral-400 hover:bg-neutral-800/80'
        "
        :title="autoApprove ? 'Auto-approve commands enabled (Danger)' : 'Ask permission before running bash commands'"
      >
        <ShieldAlert v-if="autoApprove" class="w-3.5 h-3.5 text-amber-400" />
        <ShieldCheck v-else class="w-3.5 h-3.5 text-neutral-400" />
        <span class="hidden sm:inline">{{ autoApprove ? "Auto-Approve" : "Safe Mode" }}</span>
      </button>

      <!-- Settings Button -->
      <button
        @click="emit('open-settings')"
        class="p-2 rounded-lg bg-neutral-800/50 hover:bg-neutral-700/60 text-neutral-300 hover:text-white transition-colors border border-neutral-700/40 cursor-pointer"
        title="Settings"
      >
        <Settings class="w-4 h-4" />
      </button>
    </div>
  </header>
</template>
