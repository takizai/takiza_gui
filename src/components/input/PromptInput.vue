<script setup lang="ts">
import { useI18n } from "../../i18n";
import { ref, onMounted, onUnmounted, computed, watch, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { AttachmentImport, ImageAttachment, FileAttachment } from "../../types";
import { visiblePrompt } from "../../utils/attachments";
import { supportsImageInput, TEXT_FILE_ACCEPT, IMAGE_FILE_ACCEPT, MAX_TEXT_BYTES, MAX_IMAGE_BYTES } from "../../utils/modelAttachments";
import { reasoningEfforts } from "../../utils/effort";
import { useProviderModels } from "../../composables/useProviderModels";
import { useAgentStore } from "../../stores/agentStore";
import { ArrowUp, Square, ChevronDown, Paperclip, FileText, X, LoaderCircle } from "lucide-vue-next";
import ModelContextMenu from "../common/ModelContextMenu.vue";
import { getPromptCompletions, parsePromptInput, type Completion } from "../../utils/promptCommands";
import { usePromptCommands } from "../../composables/usePromptCommands";

const { tr } = useI18n();

withDefaults(defineProps<{ welcome?: boolean }>(), { welcome: false });

const emit = defineEmits<{
  (e: "open-settings"): void;
  (e: "open-skills"): void;
  (e: "open-usage"): void;
}>();

const agentStore = useAgentStore();
const inputPrompt = ref("");
const pendingImages = ref<ImageAttachment[]>([]);
const pendingFiles = ref<FileAttachment[]>([]);
const fileInputRef = ref<HTMLInputElement | null>(null);
const importing = ref(false);
const connection = computed(() => ({
  baseUrl: agentStore.config?.base_url || '', apiKey: agentStore.config?.api_key || '',
  proxy: agentStore.config?.proxy,
  apiType: agentStore.config?.api_type, apiKeyEnv: agentStore.config?.api_key_env || '', headers: agentStore.config?.provider_headers,
  models: agentStore.config?.providers?.[agentStore.config?.active_provider || '']?.models,
}));
const { models: providerModels, refresh: refreshModels } = useProviderModels(connection, computed(() => !!connection.value.baseUrl), false);
const acceptsImages = computed(() => supportsImageInput(providerModels.value.find(model => model.id === agentStore.config?.model)
  ?? connection.value.models?.find(model => model.id === agentStore.config?.model)));
const attachmentHint = computed(() => !agentStore.config?.model ? tr('Сначала выберите модель')
  : acceptsImages.value ? tr('Прикрепить файлы · текст, код, изображения') : tr('Прикрепить файлы · текст и код. Поддержка изображений не подтверждена.'));
const acceptedFiles = computed(() => [TEXT_FILE_ACCEPT, ...(acceptsImages.value ? [IMAGE_FILE_ACCEPT] : [])].join(','));
// Refresh capability metadata once per active connection, never for draft providers.
watch([() => connection.value.baseUrl, () => connection.value.apiKey, () => connection.value.proxy, () => connection.value.apiType, () => connection.value.apiKeyEnv, () => JSON.stringify(connection.value.headers || {})], () => { void refreshModels(); });
let lastEscape = 0;
let unlistenDrop: (() => void) | undefined;
let disposed = false;
const textareaRef = ref<HTMLTextAreaElement | null>(null);
const isModelMenuOpen = ref(false);
const { output, error, busy: commandBusy, execute } = usePromptCommands(panel => {
  if (panel === "models") isModelMenuOpen.value = true;
  else if (panel === "settings") emit("open-settings");
  else if (panel === "skills") emit("open-skills");
  else emit("open-usage");
});
const caret = ref(0);
const selectedCompletion = ref(0);
const dismissedCompletion = ref("");
const draftSignature = computed(() => `${caret.value}:${inputPrompt.value}`);
const completions = computed(() => getPromptCompletions(inputPrompt.value, caret.value, agentStore.availableSkills, reasoningEfforts(agentStore.config?.model || '', agentStore.config?.api_type, providerModels.value.find(m => m.id === agentStore.config?.model))));
const showCompletions = computed(() => !commandBusy.value
  && completions.value.length > 0 && dismissedCompletion.value !== draftSignature.value);
const activeCompletionId = computed(() => showCompletions.value ? `prompt-option-${selectedCompletion.value}` : undefined);

watch(draftSignature, () => { selectedCompletion.value = 0; });
watch(completions, items => { selectedCompletion.value = Math.min(selectedCompletion.value, Math.max(0, items.length - 1)); });
watch(selectedCompletion, () => nextTick(() => {
  document.getElementById(activeCompletionId.value || "")?.scrollIntoView({ block: "nearest" });
}));

function updateCaret() {
  caret.value = textareaRef.value?.selectionStart ?? inputPrompt.value.length;
}

function onInput() {
  updateCaret();
  error.value = "";
  autoResize();
}

async function setDraft(text: string, cursor = text.length) {
  inputPrompt.value = text;
  caret.value = cursor;
  dismissedCompletion.value = "";
  await nextTick();
  textareaRef.value?.focus();
  textareaRef.value?.setSelectionRange(cursor, cursor);
  autoResize();
}

function applyCompletion(completion: Completion) {
  const replacement = completion.replacement;
  let suffix = inputPrompt.value.slice(completion.end);
  if (replacement.endsWith(" ") && suffix.startsWith(" ")) suffix = suffix.slice(1);
  void setDraft(inputPrompt.value.slice(0, completion.start) + replacement + suffix,
    completion.start + replacement.length);
}

function insertSkill(name: string) {
  const mention = `@${name}`;
  if (!inputPrompt.value.split(/\s+/).includes(mention)) {
    void setDraft(`${mention} ${inputPrompt.value}`);
  } else textareaRef.value?.focus();
}

defineExpose({
  insertSkill,
});

const currentModelShort = computed(() => {
  const config = agentStore.config;
  const m = config?.model || tr("Выбрать модель");
  const profile = config?.providers?.[config.active_provider || ""];
  let provider = profile?.name || "";
  if (!provider && config?.base_url) {
    try { provider = new URL(config.base_url).hostname; } catch { /* Not configured yet. */ }
  }
  return provider ? `${provider} · ${m}` : m;
});

function autoResize() {
  if (!textareaRef.value) return;
  textareaRef.value.style.height = "auto";
  const newHeight = Math.min(textareaRef.value.scrollHeight, 220);
  textareaRef.value.style.height = `${newHeight}px`;
}

function handleKeyDown(e: KeyboardEvent) {
  if (e.isComposing || e.keyCode === 229) return;
  if (e.key === "Escape") {
    const now = Date.now();
    if (now - lastEscape < 500) { e.preventDefault(); pendingImages.value = []; pendingFiles.value = []; void setDraft(""); }
    lastEscape = now;
  } else lastEscape = 0;
  if (showCompletions.value) {
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      const count = completions.value.length;
      selectedCompletion.value = (selectedCompletion.value + (e.key === "ArrowDown" ? 1 : -1) + count) % count;
      return;
    }
    if (e.key === "Escape") {
      e.preventDefault();
      dismissedCompletion.value = draftSignature.value;
      return;
    }
    if (e.key === "Tab" || (e.key === "Enter" && !e.shiftKey)) {
      const completion = completions.value[selectedCompletion.value];
      if (completion) {
        e.preventDefault();
        const exactCommand = completion.kind === "command" && inputPrompt.value.trim().toLowerCase() === completion.label;
        const exactArgument = completion.kind === "argument" && inputPrompt.value.trim().toLowerCase() === completion.description;
        if (e.key === "Enter" && (exactCommand || exactArgument)) void submit();
        else applyCompletion(completion);
        return;
      }
    }
  }
  if (e.key === "Enter" && !e.shiftKey) {
    e.preventDefault();
    void submit();
  }
}

async function submit() {
  const text = inputPrompt.value.trim();
  if ((!text && !pendingImages.value.length && !pendingFiles.value.length) || agentStore.operationBusy || commandBusy.value || importing.value) return;
  if (pendingImages.value.length && !acceptsImages.value) {
    error.value = tr('Эта модель не подтвердила поддержку изображений. Выберите другую модель или уберите изображения.');
    return;
  }
  const input = parsePromptInput(text, agentStore.availableSkills);
  if (input.kind === "unknown") {
    error.value = tr("Неизвестная команда /{0}. Доступные команды: /help.", {"0": input.name});
    return;
  }
  if (input.kind === "skill") {
    await setDraft(input.text);
    return;
  }
  if (input.kind === "command") {
    if (await execute(input.name, input.args)) await setDraft("");
    return;
  }
  output.value = "";
  error.value = "";
  const prompt = [input.text, ...pendingFiles.value.map(file => file.marker), ...pendingImages.value.map(image => image.marker)].filter(Boolean).join("\n");
  void agentStore.sendPrompt(prompt, pendingImages.value.map(image => image.url)).catch(cause => { error.value = String(cause); });
  pendingImages.value = [];
  pendingFiles.value = [];
  await setDraft("");
}

async function importAttachments(operation: () => Promise<AttachmentImport>) {
  if (importing.value || agentStore.operationBusy) return;
  importing.value = true;
  const start = textareaRef.value?.selectionStart ?? inputPrompt.value.length;
  const end = textareaRef.value?.selectionEnd ?? start;
  try {
    const imported = await operation();
    if (imported.images.length && !acceptsImages.value) throw new Error(tr('Эта модель не подтвердила поддержку изображений. Выберите другую модель или уберите изображения.'));
    pendingImages.value.push(...imported.images);
    if (imported.text) await setDraft(inputPrompt.value.slice(0, start) + imported.text + inputPrompt.value.slice(end), start + imported.text.length);
  } catch (cause) { error.value = String(cause); }
  finally { importing.value = false; }
}

async function selectFiles(event: Event) {
  const element = event.target as HTMLInputElement;
  const files = Array.from(element.files || []);
  element.value = ''; // Picking the same file again after removal must work.
  if (!files.length || importing.value || agentStore.operationBusy || !agentStore.config?.model) return;
  if (files.length + pendingFiles.value.length + pendingImages.value.length > 10) {
    error.value = tr('Можно прикрепить не больше 10 файлов.'); return;
  }
  importing.value = true;
  error.value = '';
  try {
    for (const file of files) {
      const image = /\.(png|jpe?g|gif|webp)$/i.test(file.name) || IMAGE_FILE_ACCEPT.split(',').includes(file.type);
      if (image && !acceptsImages.value) throw new Error(tr('Эта модель не подтвердила поддержку изображений. Выберите другую модель или уберите изображения.'));
      const textFile = file.type.startsWith('text/') || TEXT_FILE_ACCEPT.split(',').some(extension => extension.startsWith('.') && file.name.toLowerCase().endsWith(extension));
      if (!image && !textFile) throw new Error(tr('Формат файла «{0}» не поддерживается. Выберите текстовый файл или исходный код.', { '0': file.name }));
      if (file.size > (image ? MAX_IMAGE_BYTES : MAX_TEXT_BYTES)) throw new Error(tr('Файл «{0}» слишком большой. Лимит: {1} МБ.', { '0': file.name, '1': image ? 10 : 1 }));
      const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
      if (disposed) return;
      if (image) {
        const result = await invoke<AttachmentImport>('import_image', { bytes });
        pendingImages.value.push(...result.images.map(item => ({ ...item, name: file.name })));
      } else {
        const result = await invoke<FileAttachment>('import_text_file', { name: file.name, bytes });
        pendingFiles.value.push(result);
      }
    }
  } catch (cause) { error.value = String(cause); }
  finally { importing.value = false; textareaRef.value?.focus(); }
}

async function onPaste(event: ClipboardEvent) {
  event.preventDefault();
  const images = Array.from(event.clipboardData?.files || []).filter(file => file.type.startsWith("image/"));
  const text = event.clipboardData?.getData("text/uri-list") || event.clipboardData?.getData("text/plain");
  if (images.length && !acceptsImages.value) {
    error.value = tr('Эта модель не подтвердила поддержку изображений. Выберите другую модель или уберите изображения.'); return;
  }
  await importAttachments(async () => {
    if (text?.trim().startsWith("file:")) return invoke<AttachmentImport>("paste_attachments", { text });
    if (images.length) {
      const imported: AttachmentImport = { text: "", images: [] };
      for (const image of images) {
        const result = await invoke<AttachmentImport>("import_image", { bytes: Array.from(new Uint8Array(await image.arrayBuffer())) });
        imported.images.push(...result.images);
      }
      return imported;
    }
    return invoke<AttachmentImport>("paste_attachments", { text: text || null });
  });
}
onMounted(async () => {
  void refreshModels();
  textareaRef.value?.focus();
  autoResize();
  try {
    const unlisten = await getCurrentWebviewWindow().onDragDropEvent(event => {
      if (event.payload.type === "drop") void importAttachments(() => invoke<AttachmentImport>("import_paths", { paths: event.payload.type === "drop" ? event.payload.paths : [] }));
    });
    if (disposed) unlisten(); else unlistenDrop = unlisten;
  } catch { /* Browser preview has no native drop bridge. */ }
});
onUnmounted(() => { disposed = true; unlistenDrop?.(); });
</script>

<template>
  <div class="px-6 pb-5 pt-1 shrink-0 z-10">
    <div class="mx-auto relative" :class="welcome ? 'max-w-2xl' : 'max-w-3xl'">
      <div v-if="agentStore.pendingPrompts.length" class="mb-2 rounded-xl border border-white/10 bg-[#111318] p-3 text-xs">
        <div class="flex justify-between text-zinc-400 mb-2">{{ tr("Очередь ·") }} {{ agentStore.pendingPrompts.length }}
          <button v-if="agentStore.queuePaused" @click="agentStore.resumeQueue" class="text-amber-300">{{ tr("Продолжить") }}</button>
        </div>
        <div v-for="prompt in agentStore.pendingPrompts" :key="prompt.id" class="flex gap-2 py-1 text-zinc-300">
          <span class="truncate flex-1">{{ visiblePrompt(prompt.input) || tr("Вложение") }}</span>
          <button @click="agentStore.removeQueuedPrompt(prompt.id)" :aria-label='tr("Удалить запрос из очереди")'>✕</button>
        </div>
      </div>
      <div v-if="pendingFiles.length" class="flex flex-wrap gap-2 mb-2">
        <div v-for="(file, index) in pendingFiles" :key="file.marker" class="flex items-center gap-2 max-w-full rounded-xl border border-white/[0.07] bg-white/[0.025] py-2 pl-3 pr-1.5 text-xs text-zinc-300">
          <FileText class="w-4 h-4 shrink-0 text-zinc-500" />
          <span class="truncate max-w-48" :title="file.name">{{ file.name }}</span>
          <span class="text-[10px] text-zinc-500 shrink-0">{{ Math.max(1, Math.ceil(file.size / 1024)) }} KB</span>
          <button type="button" @click="pendingFiles.splice(index, 1)" :disabled="importing" class="quiet-icon !w-6 !h-6" :aria-label="tr('Убрать файл {0}', {'0': file.name})"><X class="w-3 h-3" /></button>
        </div>
      </div>
      <div v-if="pendingImages.length" class="flex gap-2 mb-2 overflow-x-auto">
        <div v-for="(image, index) in pendingImages" :key="image.marker" class="relative shrink-0">
          <img :src="image.url" :alt="image.name" class="h-20 w-20 object-cover rounded-lg" />
          <button @click="pendingImages.splice(index, 1)" class="absolute top-0 right-0 bg-black/80 px-1 rounded" :aria-label='tr("Убрать изображение")'>✕</button>
        </div>
      </div>
      <div v-if="output || error" role="status" aria-live="polite"
        class="mb-2 rounded-xl border border-white/10 bg-[#111318] px-4 py-3 text-xs text-zinc-300">
        <button type="button" @click="output = ''; error = ''" :aria-label='tr("Закрыть результат команды")'
          class="float-right ml-3 text-zinc-500 hover:text-white cursor-pointer">✕</button>
        <pre class="max-h-56 overflow-auto whitespace-pre-wrap break-words select-text font-mono" :class="{ 'text-rose-300': error }">{{ error || output }}</pre>
      </div>
      <div v-if="showCompletions" class="absolute bottom-full left-0 right-0 mb-2 overflow-hidden rounded-xl border border-white/10 bg-[#111318] shadow-2xl z-30">
        <div class="px-3 py-2 text-[10px] text-zinc-500 border-b border-white/[0.06]">{{ tr("Команды и скиллы · ↑↓ выбрать · Tab дополнить · Esc закрыть") }}</div>
        <div id="prompt-completions" role="listbox" :aria-label='tr("Команды и скиллы")' class="max-h-64 overflow-y-auto p-1">
          <button v-for="(completion, index) in completions" :id="`prompt-option-${index}`" :key="completion.id"
            type="button" role="option" :aria-selected="index === selectedCompletion" tabindex="-1"
            @mousedown.prevent @click="applyCompletion(completion)"
            class="w-full flex items-center gap-3 rounded-lg px-3 py-2 text-left cursor-pointer"
            :class="index === selectedCompletion ? 'bg-amber-500/15 text-amber-200' : 'text-zinc-300 hover:bg-white/[0.04]'">
            <span class="font-mono text-xs shrink-0">{{ completion.label }}</span>
            <span class="text-[11px] text-zinc-500 truncate flex-1">{{ completion.kind === 'command' ? tr(completion.description) : completion.description }}</span>
            <span v-if="completion.kind === 'skill'" class="text-[9px] text-amber-400 shrink-0">{{ tr("скилл") }}</span>
          </button>
        </div>
      </div>
      <div class="prompt-surface flex items-end gap-2 rounded-2xl border border-white/[0.09] bg-white/[0.035] px-4 py-3 transition-colors focus-within:border-white/20 focus-within:bg-white/[0.05]">
        <input ref="fileInputRef" type="file" multiple :accept="acceptedFiles" class="hidden" tabindex="-1" @change="selectFiles" />
        <button type="button" @click="fileInputRef?.click()" :disabled="importing || commandBusy || agentStore.operationBusy || !agentStore.config?.model"
          class="quiet-icon mb-0.5 text-zinc-500 hover:text-zinc-200 disabled:opacity-30 disabled:cursor-default shrink-0" :aria-label="tr('Прикрепить файлы')" :title="attachmentHint" :aria-busy="importing">
          <LoaderCircle v-if="importing" class="w-4 h-4 animate-spin motion-reduce:animate-none" /><Paperclip v-else class="w-4 h-4" />
        </button>
        <textarea
          ref="textareaRef" v-model="inputPrompt" @input="onInput" @paste="onPaste"
          @click="updateCaret" @keyup="updateCaret" @select="updateCaret" @keydown="handleKeyDown"
          :disabled="agentStore.operationBusy || commandBusy || importing"
          role="combobox" aria-autocomplete="list" :aria-label='tr("Сообщение, команда или скилл")'
          :aria-expanded="showCompletions" :aria-controls="showCompletions ? 'prompt-completions' : undefined"
          :aria-activedescendant="activeCompletionId"
          :placeholder='tr("Что нужно сделать?")' rows="1" wrap="soft"
          class="flex-1 w-0 min-w-0 overflow-x-hidden bg-transparent border-0 resize-none text-zinc-100 placeholder-zinc-500 text-sm py-1.5 focus:outline-none disabled:opacity-50 leading-relaxed max-h-52"
        />
        <button type="button" @click="agentStore.isStreaming ? agentStore.stop() : submit()"
          :disabled="!agentStore.isStreaming && ((!inputPrompt.trim() && !pendingImages.length && !pendingFiles.length) || commandBusy || importing || agentStore.operationBusy)"
          class="send-prompt w-8 h-8 mb-0.5 rounded-full bg-zinc-200/90 hover:bg-white disabled:bg-white/[0.06] disabled:text-zinc-600 text-zinc-950 flex items-center justify-center shrink-0 transition-colors cursor-pointer disabled:cursor-default"
          :aria-label='agentStore.isStreaming ? tr("Остановить генерацию") : tr("Отправить сообщение")' :title='agentStore.isStreaming ? tr("Остановить") : tr("Отправить (Enter)")'>
          <Square v-if="agentStore.isStreaming" class="w-3 h-3 fill-current" /><ArrowUp v-else class="w-4 h-4" />
        </button>
      </div>
      <div class="relative mt-2 px-1 w-fit">
        <button @click="isModelMenuOpen = !isModelMenuOpen" @contextmenu.prevent="isModelMenuOpen = true"
          class="flex items-center gap-1.5 py-1 px-1 text-[11px] text-zinc-500 hover:text-zinc-300 cursor-pointer rounded-md"
          :title="tr('Модель: {0}', {'0': agentStore.config?.model || tr('не выбрана')})" :aria-label='tr("Выбор модели")' :aria-expanded="isModelMenuOpen">
          <span class="max-w-60 truncate">{{ currentModelShort }}</span><ChevronDown class="w-3 h-3" />
        </button>
        <ModelContextMenu :is-open="isModelMenuOpen" :direction="welcome ? 'down' : 'up'" align="left"
          @close="isModelMenuOpen = false" @open-settings="emit('open-settings')" />
      </div>
    </div>
  </div>
</template>
