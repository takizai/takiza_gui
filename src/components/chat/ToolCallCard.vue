<script setup lang="ts">
import { ref, computed } from "vue";
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

const isExpanded = ref(true);

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
</script>

<template>
  <div
    class="my-2.5 rounded-lg border bg-neutral-900/60 overflow-hidden text-xs transition-colors"
    :class="
      isError
        ? 'border-rose-900/50 bg-rose-950/10'
        : isRunning
        ? 'border-blue-900/60 bg-blue-950/10 shadow-sm'
        : 'border-neutral-800'
    "
  >
    <!-- Header -->
    <div
      @click="isExpanded = !isExpanded"
      class="flex items-center justify-between px-3 py-2 bg-neutral-800/40 hover:bg-neutral-800/70 transition-colors cursor-pointer select-none"
    >
      <div class="flex items-center gap-2 min-w-0 pr-3">
        <component
          :is="toolIcon"
          class="w-4 h-4 shrink-0"
          :class="isError ? 'text-rose-400' : isRunning ? 'text-blue-400' : 'text-neutral-400'"
        />
        <span class="font-mono font-medium text-neutral-200 shrink-0">
          {{ name }}
        </span>
        <span class="text-neutral-400 font-mono truncate text-[11px]">
          {{ displaySummary }}
        </span>
      </div>

      <div class="flex items-center gap-2 shrink-0">
        <!-- Status indicator -->
        <span
          v-if="isRunning"
          class="flex items-center gap-1.5 text-blue-400 text-[11px] font-medium"
        >
          <Loader2 class="w-3.5 h-3.5 animate-spin" />
          <span>Выполняется...</span>
        </span>
        <span
          v-else-if="isError"
          class="flex items-center gap-1 text-rose-400 text-[11px] font-medium"
        >
          <XCircle class="w-3.5 h-3.5" />
          <span>Ошибка</span>
        </span>
        <span
          v-else
          class="flex items-center gap-1 text-emerald-400 text-[11px] font-medium"
        >
          <CheckCircle2 class="w-3.5 h-3.5" />
          <span>Готово</span>
        </span>

        <component
          :is="isExpanded ? ChevronDown : ChevronRight"
          class="w-3.5 h-3.5 text-neutral-400 ml-1"
        />
      </div>
    </div>

    <!-- Body / Logs / Diff -->
    <div v-show="isExpanded" class="p-3 border-t border-neutral-800/80 bg-black/40">
      <!-- Live streamed logs -->
      <div
        v-if="logs && logs.length > 0"
        class="mb-2 p-2 rounded bg-neutral-950 font-mono text-[11px] text-neutral-300 max-h-48 overflow-y-auto border border-neutral-800/60"
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
        class="font-mono text-[11px] text-neutral-300 whitespace-pre-wrap max-h-60 overflow-y-auto p-2 rounded bg-neutral-950/70 border border-neutral-800/60"
      >
        {{ result }}
      </div>
    </div>
  </div>
</template>
