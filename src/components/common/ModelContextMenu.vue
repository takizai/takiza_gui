<script setup lang="ts">
import { useI18n } from "../../i18n";
import { ref, computed, onMounted, onUnmounted, watch, nextTick, type CSSProperties } from "vue";
import { useAgentStore } from "../../stores/agentStore";
import { useProviderModels } from "../../composables/useProviderModels";
import {
  Check,
  Search,
  Sliders,
  Edit3,
  X,
  RefreshCw,
} from "lucide-vue-next";

const { tr } = useI18n();

const props = withDefaults(
  defineProps<{
    isOpen: boolean;
    align?: "left" | "right";
    direction?: "up" | "down";
  }>(),
  {
    align: "left",
    direction: "up",
  }
);

const emit = defineEmits<{
  (e: "close"): void;
  (e: "open-settings"): void;
}>();

const agentStore = useAgentStore();
const searchQuery = ref("");
const isCustomInputOpen = ref(false);
const customModelInput = ref("");
const containerRef = ref<HTMLDivElement | null>(null);
const anchorRef = ref<HTMLElement | null>(null);
const listRef = ref<HTMLElement | null>(null);
const searchRef = ref<HTMLInputElement | null>(null);
const placement = ref<CSSProperties>({ visibility: "hidden" });
let repositionFrame = 0;
let observer: ResizeObserver | undefined;
let trigger: HTMLElement | null = null;
let opensUp = true;
function positionPanel(resetSide = false) {
  const anchor = anchorRef.value?.parentElement;
  if (!props.isOpen || !anchor) return;
  const rect = anchor.getBoundingClientRect();
  const gap = 6; const edge = 12;
  const width = Math.min(280, window.innerWidth - edge * 2);
  const below = window.innerHeight - rect.bottom - gap - edge;
  const above = rect.top - gap - edge;
  if (resetSide) opensUp = props.direction === "up" ? above >= 280 || above > below : below < 280 && above > below;
  const up = opensUp;
  const left = Math.max(edge, Math.min(props.align === "right" ? rect.right - width : rect.left, window.innerWidth - width - edge));
  placement.value = { position: "fixed", width: `${width}px`, left: `${left}px`, height: `${Math.max(0, Math.min(300, up ? above : below))}px`,
    ...(up ? { bottom: `${Math.max(edge, window.innerHeight - rect.top + gap)}px` } : { top: `${Math.max(edge, rect.bottom + gap)}px` }) };
}
function schedulePosition(event?: Event) {
  if (!props.isOpen || (event?.target instanceof Node && containerRef.value?.contains(event.target))) return;
  cancelAnimationFrame(repositionFrame);
  repositionFrame = requestAnimationFrame(() => positionPanel());
}
function restoreFocus() {
  if (document.activeElement === document.body || containerRef.value?.contains(document.activeElement)) trigger?.focus({ preventScroll: true });
}
watch(() => props.isOpen, async open => {
  if (!open) return;
  trigger = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  positionPanel(true);
  await nextTick();
  if (props.isOpen) { positionPanel(); searchRef.value?.focus({ preventScroll: true }); }
});

const currentModel = computed(() => agentStore.config?.model || "");

const activeProfile = computed(() => agentStore.config?.providers?.[agentStore.config?.active_provider || ""]);
const connection = computed(() => ({ apiType: agentStore.config?.api_type, apiKeyEnv: agentStore.config?.api_key_env || '', headers: agentStore.config?.provider_headers, models: activeProfile.value?.models, baseUrl: agentStore.config?.base_url || "", apiKey: agentStore.config?.api_key || "", proxy: agentStore.config?.proxy }));
const { models: allModels, loading, error: modelError, provider, refresh } = useProviderModels(connection, computed(() => props.isOpen));
const saveError = ref("");
const saving = ref(false);
const activeModelIndex = ref(0);
watch(searchQuery, () => { activeModelIndex.value = 0; if (listRef.value) listRef.value.scrollTop = 0; });
function handleModelKeys(event: KeyboardEvent) {
  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault();
    const count = filteredModels.value.length;
    if (!count) return;
    activeModelIndex.value = (activeModelIndex.value + (event.key === 'ArrowDown' ? 1 : -1) + count) % count;
    void nextTick(() => {
      const list = listRef.value, row = list?.querySelector<HTMLElement>(`[data-model-index="${activeModelIndex.value}"]`);
      if (!list || !row) return;
      const bounds = list.getBoundingClientRect(), rect = row.getBoundingClientRect();
      if (rect.top < bounds.top) list.scrollTop -= bounds.top - rect.top;
      else if (rect.bottom > bounds.bottom) list.scrollTop += rect.bottom - bounds.bottom;
    });
  } else if (event.key === 'Enter') {
    event.preventDefault();
    const model = filteredModels.value[activeModelIndex.value];
    if (model) void selectModel(model.id);
  }
}

const filteredModels = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  if (!q) return allModels.value;
  return allModels.value.filter(
    (m) =>
      m.name.toLowerCase().includes(q) ||
      m.id.toLowerCase().includes(q) ||
      m.provider.toLowerCase().includes(q) ||
      m.description.toLowerCase().includes(q)
  );
});

function openSettings() {
  trigger?.focus({ preventScroll: true });
  emit("open-settings"); emit("close");
}

async function selectModel(modelId: string) {
  if (saving.value) return;
  saving.value = true;
  saveError.value = "";
  try {
    await agentStore.savePreferences({ model: modelId, mode: "manual", ...(allModels.value.length ? {model_catalog:allModels.value} : {}) });
    emit("close");
  } catch (err) {
    saveError.value = String(err);
  } finally { saving.value = false; }
}

async function submitCustomModel() {
  const val = customModelInput.value.trim();
  if (!val) return;
  await selectModel(val);
  customModelInput.value = "";
  isCustomInputOpen.value = false;
}

function handleClickOutside(event: MouseEvent) {
  if (
    containerRef.value &&
    !containerRef.value.contains(event.target as Node)
    && !anchorRef.value?.parentElement?.contains(event.target as Node)
  ) {
    emit("close");
  }
}

function handleKeyDown(event: KeyboardEvent) {
  if (props.isOpen && event.key === "Escape") {
    event.preventDefault();
    event.stopPropagation();
    emit("close");
  }
}

onMounted(() => {
  observer = new ResizeObserver(() => schedulePosition());
  if (anchorRef.value?.parentElement) observer.observe(anchorRef.value.parentElement);
  window.addEventListener("resize", schedulePosition);
  window.addEventListener("scroll", schedulePosition, true);
  document.addEventListener("mousedown", handleClickOutside);
  document.addEventListener("keydown", handleKeyDown);
});

onUnmounted(() => {
  observer?.disconnect();
  cancelAnimationFrame(repositionFrame);
  window.removeEventListener("resize", schedulePosition);
  window.removeEventListener("scroll", schedulePosition, true);
  document.removeEventListener("mousedown", handleClickOutside);
  document.removeEventListener("keydown", handleKeyDown);
});
</script>

<template>
  <span ref="anchorRef" class="hidden" aria-hidden="true" />
  <Teleport to="body">
    <Transition name="popover" @after-leave="restoreFocus">
      <div v-if="isOpen" ref="containerRef" :style="placement" role="dialog" :aria-label='tr("Выбор модели")'
        class="model-popover floating-menu z-[110] flex flex-col overflow-hidden text-zinc-200" @click.stop>
        <div class="px-3 pt-3 pb-2.5 shrink-0 border-b border-white/[0.06]">
          <div class="flex items-center justify-between gap-2 mb-1.5">
            <span class="text-[10px] font-medium tracking-wide text-zinc-500 truncate">{{ activeProfile?.name || provider?.name || tr("Модели") }}</span>
            <button @click="refresh" :disabled="loading" :aria-label="tr('Обновить модели')" :title="tr('Обновить модели')" class="ml-auto p-1 text-zinc-500 hover:text-zinc-200 cursor-pointer disabled:opacity-40"><RefreshCw class="w-3 h-3" :class="{ 'animate-spin motion-reduce:animate-none': loading }" /></button>
            <button @click="emit('close')" :aria-label='tr("Закрыть выбор модели")' class="p-1 rounded text-zinc-500 hover:text-zinc-200 cursor-pointer"><X class="w-3 h-3" /></button>
          </div>
          <div class="relative flex items-center">
            <Search class="w-3 h-3 absolute left-2 text-zinc-500 pointer-events-none" />
            <input ref="searchRef" v-model="searchQuery" :aria-label='tr("Поиск модели")' :placeholder='tr("Поиск модели…")'
              @keydown="handleModelKeys" role="combobox" aria-autocomplete="list" aria-controls="provider-model-options" aria-expanded="true"
              :aria-activedescendant="filteredModels[activeModelIndex] ? `provider-model-option-${activeModelIndex}` : undefined"
              class="w-full pl-7 pr-2 py-2 bg-black/20 border border-white/[0.04] rounded-lg text-xs text-zinc-200 placeholder-zinc-600" />
          </div>
        </div>
        <div ref="listRef" class="min-h-0 flex-1 overflow-y-auto overscroll-contain p-1.5" role="listbox" id="provider-model-options" :aria-label="tr('Модели')">
          <p v-if="loading" role="status" class="p-2 text-xs text-zinc-500">{{ tr("Загрузка…") }}</p>
          <p v-if="modelError" role="status" class="p-2 text-xs text-zinc-400 break-words">{{ tr(modelError) }} <button @click="refresh" class="underline cursor-pointer">{{ tr("Повторить") }}</button></p>
          <p v-if="saveError" role="alert" class="p-2 text-xs text-rose-300 break-words">{{ tr(saveError) }}</p>
          <button v-for="(model, index) in filteredModels" :key="model.id" :disabled="saving" @mousedown.prevent @click="selectModel(model.id)"
            :id="`provider-model-option-${index}`" :data-model-index="index" role="option" tabindex="-1"
            :title="model.id" :aria-selected="currentModel === model.id" @pointermove="activeModelIndex = index"
            class="select-option" :class="{ 'select-option-active': activeModelIndex === index }">
            <span class="min-w-0 flex-1">
              <span class="block truncate">{{ model.name }}</span>
              <span v-if="model.name !== model.id" class="block truncate text-[10px] text-zinc-500">{{ model.id }}</span>
            </span>
            <Check v-if="currentModel === model.id" class="w-3.5 h-3.5 shrink-0 text-amber-300" />
          </button>
          <p v-if="!loading && !modelError && !filteredModels.length" class="p-3 text-xs text-zinc-500">{{ tr("Модели не найдены") }}</p>
        </div>
        <form v-if="isCustomInputOpen" @submit.prevent="submitCustomModel" class="flex items-center gap-2 px-2 py-2 border-t border-white/[0.06] shrink-0">
          <input v-model="customModelInput" :aria-label='tr("ID модели")' :placeholder='tr("ID модели")' class="min-w-0 flex-1 bg-white/[0.03] rounded-md p-1.5 text-xs" />
          <button :disabled="saving || !customModelInput.trim()" class="text-xs text-zinc-300 cursor-pointer disabled:opacity-40">{{ tr("ОК") }}</button>
        </form>
        <div class="flex items-center justify-between px-2 py-1.5 border-t border-white/[0.06] shrink-0">
          <button @click="isCustomInputOpen = !isCustomInputOpen; nextTick(positionPanel)" class="flex items-center gap-1 p-1 text-[10px] text-zinc-500 hover:text-zinc-200 cursor-pointer"><Edit3 class="w-3 h-3" />{{ tr("Ввести ID") }}</button>
          <button @click="openSettings" class="flex items-center gap-1 p-1 text-[10px] text-zinc-500 hover:text-zinc-200 cursor-pointer"><Sliders class="w-3 h-3" />{{ tr("Настройки") }}</button>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
