<script setup lang="ts">
import { useI18n } from "../../i18n";
import { computed, ref, onUnmounted, nextTick } from "vue";
import type { HistoryItem } from "../../types";
import { visiblePrompt, attachedFiles } from "../../utils/attachments";
import { renderMarkdown } from "../../utils/markdown";
import ThinkingBlock from "./ThinkingBlock.vue";
import ToolCallCard from "./ToolCallCard.vue";
import QuestionHistoryCard from "./QuestionHistoryCard.vue";
import { AlertTriangle, Copy, Check, RotateCcw, Pencil, FileText } from "lucide-vue-next";
import { useAgentStore } from "../../stores/agentStore";

const { tr } = useI18n();

const props = defineProps<{
  item: HistoryItem;
  historyIndex: number;
}>();

const agentStore = useAgentStore();
const copied = ref(false);
const errorExpanded = ref(false);
const actionError = ref("");
const isEditing = ref(false);
const editedText = ref("");
const editInput = ref<HTMLTextAreaElement | null>(null);
const actionsDisabled = computed(() => agentStore.isStreaming || agentStore.operationBusy);
const files = computed(() => props.item.type === 'UserPrompt' ? attachedFiles(props.item.payload) : []);
function resizeEdit() {
  if (!editInput.value) return;
  editInput.value.style.height = "auto";
  editInput.value.style.height = `${Math.min(260, Math.max(28, editInput.value.scrollHeight))}px`;
}
async function beginEdit() {
  if (props.item.type !== "UserPrompt") return;
  editedText.value = visiblePrompt(props.item.payload);
  actionError.value = ""; isEditing.value = true;
  await nextTick(); editInput.value?.focus(); resizeEdit();
}
async function regenerate(edited?: string) {
  actionError.value = "";
  try {
    const operation = agentStore.regenerateMessage(props.historyIndex, edited);
    if (agentStore.isStreaming) isEditing.value = false;
    await operation;
  }
  catch (error) { actionError.value = String(error); }
}
let copyTimer: ReturnType<typeof setTimeout> | undefined;
onUnmounted(() => clearTimeout(copyTimer));

const renderedContent = computed(() => {
  if (props.item.type === "AssistantMessage") {
    return renderMarkdown(props.item.payload);
  }
  return "";
});

async function copyText(text: string) {
  try {
    await navigator.clipboard.writeText(text);
    copied.value = true;
    clearTimeout(copyTimer);
    copyTimer = setTimeout(() => {
      copied.value = false;
    }, 1500);
  } catch (err) {
    console.error("Failed to copy:", err);
  }
}

const isVisible = computed(() => {
  if (!props.item || !props.item.type) return false;
  if (props.item.type === "UserPrompt") return !!props.item.payload && props.item.payload.trim().length > 0;
  if (props.item.type === "MoaRouting") return !!props.item.payload && !!props.item.payload.model;
  if (props.item.type === "Thought") return !!props.item.payload && props.item.payload.trim().length > 0;
  if (props.item.type === "QuestionInteraction" || props.item.type === "FileDiff") return true;
  if (props.item.type === "ToolEnd") return !!props.item.payload;
  if (props.item.type === "AssistantMessage") return !!props.item.payload && props.item.payload.trim().length > 0;
  if (props.item.type === "ToolLog") return props.item.payload.includes("Stopped by user");
  if (props.item.type === "Error") return !!props.item.payload;
  return false;
});
</script>

<template>
  <article v-if="isVisible" class="chat-entry group" :data-kind="item.type">
    <div v-if="item.type === 'UserPrompt'" class="chat-user-row">
      <form v-if="isEditing" class="message-editor" @submit.prevent="regenerate(editedText)">
        <div class="flex items-center gap-1.5 mb-3 text-[10px] text-zinc-500 select-none"><Pencil class="w-3 h-3" />{{ tr("Редактирование") }}</div>
        <textarea ref="editInput" v-model="editedText" @input="resizeEdit" @keydown.esc.prevent="isEditing = false" @keydown.ctrl.enter.prevent="regenerate(editedText)" @keydown.meta.enter.prevent="regenerate(editedText)"
          :disabled="actionsDisabled" :aria-label='tr("Редактировать сообщение")' rows="1" class="message-editor-input" />
        <div v-if="files.length" class="flex flex-wrap gap-2 mt-3 text-xs text-zinc-400">
          <span v-for="file in files" :key="file.path" class="inline-flex items-center gap-1.5"><FileText class="w-3 h-3" />{{ file.name }}</span>
        </div>
        <div v-if="item.imageUrls?.length" class="flex flex-wrap gap-2 mt-3">
          <img v-for="(url, index) in item.imageUrls" :key="index" :src="url" :alt='tr("Прикреплённое изображение")' class="max-h-32 max-w-full rounded-lg" />
        </div>
        <div class="flex items-center justify-end gap-2 mt-3 pt-2 border-t border-white/[0.05]">
          <span class="hidden sm:block mr-auto text-[10px] text-zinc-600 select-none">{{ tr("Esc — отмена · Ctrl+Enter — сохранить") }}</span>
          <button type="button" @click="isEditing = false" :disabled="actionsDisabled" class="editor-cancel">{{ tr("Отмена") }}</button>
          <button type="submit" :disabled="actionsDisabled || !editedText.trim()" class="editor-save" :aria-label='tr("Сохранить и перегенерировать")' :title='tr("Заменить ответ и последующую историю. Точка возврата сохранится.")'><Check class="w-3 h-3" />{{ tr("Сохранить") }}</button>
        </div>
      </form>
      <template v-else>
      <div class="chat-user-message">
        <p class="whitespace-pre-wrap break-words">{{ visiblePrompt(item.payload) }}</p>
        <div v-if="files.length" class="flex flex-wrap gap-2 mt-2 text-xs text-zinc-400">
          <span v-for="file in files" :key="file.path" class="inline-flex items-center gap-1.5 rounded-lg border border-white/[0.06] px-2 py-1 max-w-full"><FileText class="w-3 h-3 shrink-0" /><span class="truncate">{{ file.name }}</span></span>
        </div>
        <div v-if="item.imageUrls?.length" class="flex flex-wrap gap-2 mt-3">
          <img v-for="(url, index) in item.imageUrls" :key="index" :src="url" :alt='tr("Прикреплённое изображение")' loading="lazy" class="max-h-52 max-w-full rounded-lg" />
        </div>
      </div>
      <div class="message-actions justify-end">
        <button @click="beginEdit" :disabled="actionsDisabled" class="message-action-button" :aria-label='tr("Переделать сообщение")'><Pencil class="w-3 h-3" />{{ tr("Переделать") }}</button>
        <button @click="regenerate()" :disabled="actionsDisabled" class="message-action-button" :aria-label='tr("Перегенерировать ответ на сообщение")' :title='tr("Перегенерировать ответ с этого места")'><RotateCcw class="w-3 h-3" />{{ tr("Перегенерировать") }}</button>
      </div>
      </template>
    </div>
    <p v-else-if="item.type === 'MoaRouting'" class="text-[11px] text-zinc-500 py-1">{{ tr("Модель ·") }} {{ item.payload.model }}</p>
    <ThinkingBlock v-else-if="item.type === 'Thought'" :content="item.payload" />
    <QuestionHistoryCard v-else-if="item.type === 'QuestionInteraction'" :questions="item.payload.questions" :answers="item.payload.answers" :skipped="item.payload.skipped" />
    <ToolCallCard v-else-if="item.type === 'FileDiff'" name="edit_file" args="{}" :result="item.payload" />
    <ToolCallCard v-else-if="item.type === 'ToolEnd'" :name="item.payload.name" :args="item.payload.args" :result="item.payload.result" :is-error="item.payload.is_error" />
    <div v-else-if="item.type === 'AssistantMessage'" class="chat-assistant">
      <div class="chat-author">Takiza<span v-if="item.moaModel" class="font-normal text-zinc-500">{{ item.moaModel }}</span></div>
      <div class="prose-takiza select-text" v-html="renderedContent" />
      <div class="message-actions mt-2">
        <button @click="copyText(item.payload)" class="message-action-button" :aria-label='tr("Скопировать ответ")'>
          <Check v-if="copied" class="w-3 h-3" /><Copy v-else class="w-3 h-3" />{{ copied ? tr("Скопировано") : tr("Копировать") }}
        </button>
        <button @click="regenerate()" :disabled="actionsDisabled" class="message-action-button" :aria-label='tr("Перегенерировать ответ")' :title='tr("Перегенерировать ответ с этого места")'><RotateCcw class="w-3 h-3" />{{ tr("Перегенерировать") }}</button>
      </div>
    </div>
    <p v-else-if="item.type === 'ToolLog'" class="text-xs text-zinc-500 py-2">{{ item.payload }}</p>
    <div v-else-if="item.type === 'Error'" class="chat-error" role="alert">
      <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" />
      <div class="min-w-0">
        <p class="text-xs font-medium mb-1">{{ tr("Не удалось завершить действие") }}</p>
        <pre class="whitespace-pre-wrap break-words font-mono text-[11px]">{{ errorExpanded || agentStore.outputExpanded ? item.payload : item.payload.split('\n').slice(0, 3).join('\n') }}</pre>
        <button v-if="item.payload.split('\n').length > 3" @click="errorExpanded = !errorExpanded" class="mt-2 text-[11px] text-zinc-400 cursor-pointer">{{ errorExpanded ? tr("Свернуть") : tr("Подробнее · Ctrl+O") }}</button>
      </div>
    </div>
    <p v-if="actionError" role="alert" class="text-xs text-rose-300">{{ tr(actionError) }}</p>
  </article>
</template>
