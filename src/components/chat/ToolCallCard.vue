<script setup lang="ts">
import { useI18n } from "../../i18n";
import { ref, computed, useId, onUnmounted } from "vue";
import { FileText, FileEdit, Terminal, Search, FolderTree, Check, CircleAlert, Loader2, ChevronRight, Copy, Wrench, BookOpen, MessageCircle } from "lucide-vue-next";
import { useAgentStore } from "../../stores/agentStore";
import DiffViewer from "./DiffViewer.vue";

const { tr } = useI18n();
const props = defineProps<{ name: string; args: string; result?: string; logs?: string[]; isError?: boolean; isRunning?: boolean }>();
const agent = useAgentStore();
const contentId = useId();
const locallyExpanded = ref(false);
const isExpanded = computed(() => locallyExpanded.value || agent.outputExpanded);
function toggleOutput() { const next = !isExpanded.value; agent.outputExpanded = false; locallyExpanded.value = next; }
const output = computed(() => props.result ?? props.logs?.join("\n") ?? "");
const errorPreview = computed(() => output.value.split("\n").slice(0, 2).join("\n").slice(0, 360));
const livePreview = computed(() => {
  const logs = props.logs || [];
  for (let index = logs.length - 1; index >= 0; index--) {
    if (logs[index].trim()) return logs[index].slice(0, 180);
  }
  return "";
});
const parsedArgs = computed<Record<string, unknown> | null>(() => {
  try { const value: unknown = JSON.parse(props.args); return value && typeof value === "object" && !Array.isArray(value) ? value as Record<string, unknown> : null; }
  catch { return null; }
});
const formattedArgs = computed(() => parsedArgs.value ? JSON.stringify(parsedArgs.value, null, 2) : props.args);
const displaySummary = computed(() => {
  const args = parsedArgs.value;
  if (args) {
    for (const key of ["topic", "path", "command", "query", "pattern", "name"]) if (typeof args[key] === "string") return args[key] as string;
  }
  return props.args.slice(0, 120);
});
const tools = computed<Record<string, { label: string; icon: typeof Terminal }>>(() => ({
  run_command: { label: tr("Команда"), icon: Terminal }, execute_command: { label: tr("Команда"), icon: Terminal },
  read_file: { label: tr("Чтение файла"), icon: FileText }, write_file: { label: tr("Запись файла"), icon: FileEdit },
  edit_file: { label: tr("Изменение файла"), icon: FileEdit }, edit_file_lines: { label: tr("Изменение файла"), icon: FileEdit },
  web_search: { label: tr("Поиск в интернете"), icon: Search }, grep_search: { label: tr("Поиск в коде"), icon: Search },
  find_files: { label: tr("Поиск файлов"), icon: Search }, list_dir: { label: tr("Содержимое папки"), icon: FolderTree },
  "grill-me": { label: tr("Разбор плана"), icon: MessageCircle },
  read_skill: { label: tr("Чтение скилла"), icon: BookOpen }, ask_question: { label: tr("Уточнение"), icon: MessageCircle },
}));
const tool = computed(() => tools.value[props.name] || { label: props.name, icon: Wrench });
const isDiff = computed(() => !!props.result && /(^diff --git |^@@\s+-\d)/m.test(props.result));
const status = computed(() => props.isRunning ? tr("В работе") : props.isError ? tr("Ошибка") : tr("Готово"));
const copied = ref(false);
let copyTimer: ReturnType<typeof setTimeout> | undefined;
onUnmounted(() => clearTimeout(copyTimer));
async function copyOutput() {
  if (!output.value) return;
  try { await navigator.clipboard.writeText(output.value); copied.value = true; clearTimeout(copyTimer); copyTimer = setTimeout(() => copied.value = false, 1500); }
  catch (error) { console.error("Failed to copy output:", error); }
}
</script>

<template>
  <section class="chat-activity" :class="{ 'activity-error': isError }" :aria-label="tr('Инструмент: {0}', {'0': name})">
    <button @click="toggleOutput" :aria-expanded="isExpanded" :aria-controls="contentId" class="activity-row" :title="`${name} · ${displaySummary}`">
      <component :is="tool.icon" class="activity-icon" />
      <span class="shrink-0 max-w-[45%] truncate text-zinc-400">{{ tool.label }}</span>
      <span class="min-w-0 flex-1 truncate font-mono text-[11px] text-zinc-500">{{ displaySummary }}</span>
      <span class="flex items-center gap-1.5 shrink-0 text-[10px]" :class="isError ? 'text-rose-400' : 'text-zinc-500'">
        <Loader2 v-if="isRunning" class="w-3 h-3 animate-spin" /><CircleAlert v-else-if="isError" class="w-3 h-3" /><Check v-else class="w-3 h-3" />
        <span class="hidden sm:inline">{{ status }}</span><span class="sr-only sm:hidden">{{ status }}</span>
      </span>
      <ChevronRight class="w-3 h-3 shrink-0 text-zinc-500 transition-transform" :class="{ 'rotate-90': isExpanded }" />
    </button>
    <p v-if="!isExpanded && (isError || (isRunning && livePreview))" class="ml-7 mr-3 mb-2 text-[11px] font-mono whitespace-pre-wrap break-words" :class="isError ? 'text-rose-300' : 'text-zinc-500'">{{ isError ? errorPreview : livePreview }}</p>
    <Transition name="activity">
      <div v-if="isExpanded" :id="contentId" class="activity-detail">
        <div class="flex items-center justify-between gap-3 mb-2 text-[10px] text-zinc-500">
          <span class="font-mono">{{ name }}</span>
          <button v-if="output" @click="copyOutput" class="flex items-center gap-1 rounded p-1 hover:text-zinc-200 cursor-pointer" :aria-label="copied ? tr('Вывод скопирован') : tr('Копировать вывод')">
            <Check v-if="copied" class="w-3 h-3" /><Copy v-else class="w-3 h-3" />{{ copied ? tr("Скопировано") : tr("Копировать") }}
          </button>
        </div>
        <div class="max-h-80 overflow-auto overscroll-contain select-text">
          <div v-if="args" class="mb-3">
            <p class="activity-caption">{{ tr("Параметры") }}</p><pre class="activity-output">{{ formattedArgs }}</pre>
          </div>
          <div v-if="output">
            <p class="activity-caption">{{ isRunning ? tr("Вывод · в работе") : tr("Результат") }}</p>
            <DiffViewer v-if="isDiff" :diff-text="result || ''" /><pre v-else class="activity-output" :class="{ 'text-rose-300': isError }">{{ output }}</pre>
          </div>
          <p v-else class="text-xs text-zinc-500">{{ isRunning ? tr("Ожидаю вывод…") : tr("Выполнено без вывода.") }}</p>
        </div>
      </div>
    </Transition>
  </section>
</template>
