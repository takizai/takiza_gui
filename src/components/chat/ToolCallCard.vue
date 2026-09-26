<script setup lang="ts">
import { ref, computed, watch } from "vue";
import {
  FileText,
  FileEdit,
  Terminal,
  Search,
  FolderTree,
  CheckCircle2,
  XCircle,
  Loader2,
  ChevronDown,
  ChevronRight,
  Copy,
  Check,
} from "lucide-vue-next";
import DiffViewer from "./DiffViewer.vue";

const props = defineProps<{
  name: string;
  args: string;
  result?: string;
  logs?: string[];
  isError?: boolean;
  isRunning?: boolean;
}>();

// Default expanded while running or if failed with error. Finished successful tools stay collapsed to keep chat clean.
const isExpanded = ref(props.isRunning || props.isError);

watch(
  () => props.isRunning,
  (running) => {
    if (running) {
      isExpanded.value = true;
    } else if (!props.isError) {
      // Auto-collapse on clean completion so assistant text is not buried
      isExpanded.value = false;
    }
  }
);

watch(
  () => props.isError,
  (err) => {
    if (err) {
      isExpanded.value = true;
    }
  }
);

const copied = ref(false);

const parsedArgs = computed(() => {
  try {
    return JSON.parse(props.args);
  } catch {
    return null;
  }
});

const displaySummary = computed(() => {
  if (parsedArgs.value) {
    if (parsedArgs.value.path) return parsedArgs.value.path;
    if (parsedArgs.value.command) return parsedArgs.value.command;
    if (parsedArgs.value.query) return `"${parsedArgs.value.query}"`;
    if (parsedArgs.value.pattern) return parsedArgs.value.pattern;
  }
  return props.args ? props.args.slice(0, 60) : "";
});

const isDiff = computed(() => {
  return (
    props.name === "edit_file" ||
    (props.result && (props.result.includes("@@") || props.result.startsWith("diff --git")))
  );
});

const toolIcon = computed(() => {
  switch (props.name) {
    case "run_command":
      return Terminal;
    case "read_file":
      return FileText;
    case "write_file":
    case "edit_file":
      return FileEdit;
    case "grep_search":
    case "find_files":
      return Search;
    case "list_dir":
      return FolderTree;
    default:
      return Terminal;
  }
});

async function copyOutput() {
  const content = props.result || props.logs?.join("\n") || "";
  if (!content) return;
  try {
    await navigator.clipboard.writeText(content);
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 1500);
  } catch (e) {
    console.error(e);
  }
}
</script>

<template>
  <div
    class="my-2.5 rounded-xl border bg-[#0d0e12] overflow-hidden text-xs transition-colors"
    :class="
      isError
        ? 'border-rose-500/30'
        : isRunning
        ? 'border-white/20 shadow-[0_0_15px_rgba(255,255,255,0.03)]'
        : 'border-white/[0.08]'
    "
  >
    <!-- Header -->
    <div
      @click="isExpanded = !isExpanded"
      class="flex items-center justify-between px-3.5 py-2 bg-white/[0.02] hover:bg-white/[0.05] transition-colors cursor-pointer select-none"
    >
      <div class="flex items-center gap-2.5 min-w-0 pr-3">
        <component
          :is="toolIcon"
          class="w-4 h-4 shrink-0"
          :class="isError ? 'text-rose-400' : isRunning ? 'text-zinc-200' : 'text-zinc-400'"
        />
        <span class="font-mono font-medium text-zinc-200 shrink-0">
          {{ name }}
        </span>
        <span class="text-zinc-500 font-mono truncate text-[11px]">
          {{ displaySummary }}
        </span>
      </div>

      <div class="flex items-center gap-2 shrink-0">
        <!-- Status indicator -->
        <span
          v-if="isRunning"
          class="flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-amber-500/10 border border-amber-500/20 text-amber-300 text-[10px] font-mono"
        >
          <Loader2 class="w-3 h-3 animate-spin" />
          <span>running</span>
        </span>
        <span
          v-else-if="isError"
          class="flex items-center gap-1 px-2 py-0.5 rounded-full bg-rose-500/10 border border-rose-500/20 text-rose-300 text-[10px] font-mono"
        >
          <XCircle class="w-3 h-3" />
          <span>failed</span>
        </span>
        <span
          v-else
          class="flex items-center gap-1 px-2 py-0.5 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-[10px] font-mono"
        >
          <CheckCircle2 class="w-3 h-3" />
          <span>success</span>
        </span>

        <component
          :is="isExpanded ? ChevronDown : ChevronRight"
          class="w-3.5 h-3.5 text-zinc-500 ml-1"
        />
      </div>
    </div>

    <!-- Body / Logs / Diff -->
    <div v-show="isExpanded" class="p-3 border-t border-white/[0.06] bg-black/40 relative">
      <!-- Copy button if output present -->
      <button
        v-if="result || (logs && logs.length > 0)"
        @click.stop="copyOutput"
        class="absolute right-3 top-3 p-1 rounded-md bg-white/[0.04] hover:bg-white/[0.1] text-zinc-400 hover:text-zinc-200 transition-colors z-10 cursor-pointer"
        title="Копировать вывод"
      >
        <Check v-if="copied" class="w-3.5 h-3.5 text-emerald-400" />
        <Copy v-else class="w-3.5 h-3.5" />
      </button>

      <!-- Live streamed logs -->
      <div
        v-if="logs && logs.length > 0"
        class="mb-2 p-2.5 rounded-lg bg-black/60 font-mono text-[11px] text-zinc-300 max-h-48 overflow-y-auto border border-white/[0.06]"
      >
        <div v-for="(log, lIdx) in logs" :key="lIdx" class="whitespace-pre-wrap">
          {{ log }}
        </div>
      </div>

      <!-- Diff result if applicable -->
      <DiffViewer v-if="isDiff && result" :diffText="result" />

      <!-- Regular result output -->
      <div
        v-else-if="result"
        class="font-mono text-[11px] text-zinc-300 whitespace-pre-wrap max-h-60 overflow-y-auto p-2.5 rounded-lg bg-black/60 border border-white/[0.06]"
      >
        {{ result }}
      </div>
    </div>
  </div>
</template>
