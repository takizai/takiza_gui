<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, useId, watch, type CSSProperties } from 'vue';
import { Check, ChevronDown, Search } from 'lucide-vue-next';
import { useI18n } from '../../i18n';
defineOptions({ inheritAttrs: false });
const props = withDefaults(defineProps<{
  id?: string; modelValue: string; options: { value: string; label: string; description?: string }[];
  placeholder?: string; disabled?: boolean; searchable?: boolean;
}>(), { placeholder: '', disabled: false, searchable: false });
const emit = defineEmits<{ (e: 'update:modelValue', value: string): void }>();
const { tr } = useI18n();
const uid = useId();
const listId = `select-${uid}`;
const triggerId = computed(() => props.id || `select-trigger-${uid}`);
const trigger = ref<HTMLButtonElement | null>(null);
const panel = ref<HTMLElement | null>(null);
const list = ref<HTMLElement | null>(null);
const searchInput = ref<HTMLInputElement | null>(null);
const query = ref('');
const opened = ref(false);
const active = ref(0);
const placement = ref<CSSProperties>({ visibility: 'hidden' });
const selected = computed(() => props.options.find(option => option.value === props.modelValue));
const visibleOptions = computed(() => {
  const text = query.value.trim().toLowerCase();
  return text ? props.options.filter(option => `${option.label} ${option.value} ${option.description || ''}`.toLowerCase().includes(text)) : props.options;
});
let observer: ResizeObserver | undefined;
let frame = 0;
let opensUp = false;
let typed = '';
let typedAt = 0;
function position(resetSide = false) {
  if (!opened.value || !trigger.value) return;
  const rect = trigger.value.getBoundingClientRect();
  const edge = 12, gap = 6;
  const below = innerHeight - rect.bottom - edge - gap;
  const above = rect.top - edge - gap;
  if (resetSide) opensUp = below < 220 && above > below;
  const up = opensUp;
  const width = Math.min(Math.max(rect.width, 220), innerWidth - edge * 2);
  placement.value = { position: 'fixed', width: `${width}px`,
    left: `${Math.max(edge, Math.min(rect.left, innerWidth - width - edge))}px`,
    maxHeight: `${Math.max(0, Math.min(300, up ? above : below))}px`,
    ...(props.searchable ? { height: `${Math.max(0, Math.min(300, up ? above : below))}px` } : {}),
    ...(up ? { bottom: `${innerHeight - rect.top + gap}px` } : { top: `${rect.bottom + gap}px` }),
  };
}
function close(restore = false) {
  opened.value = false;
  if (restore) trigger.value?.focus({ preventScroll: true });
}
async function show() {
  if (props.disabled || !props.options.length) return;
  query.value = '';
  active.value = Math.max(0, props.options.findIndex(option => option.value === props.modelValue));
  opened.value = true; position(true);
  await nextTick();
  if (props.searchable) searchInput.value?.focus({ preventScroll: true });
  scrollActive();
}
function choose(index: number) {
  const option = visibleOptions.value[index];
  if (!option || props.disabled) return;
  emit('update:modelValue', option.value); close(true);
}
function scrollActive() {
  const container = list.value;
  const option = container?.querySelector<HTMLElement>(`#${CSS.escape(`${listId}-${active.value}`)}`);
  if (!container || !option) return;
  const bounds = container.getBoundingClientRect(), row = option.getBoundingClientRect();
  if (row.top < bounds.top) container.scrollTop -= bounds.top - row.top;
  else if (row.bottom > bounds.bottom) container.scrollTop += row.bottom - bounds.bottom;
}
function onKeydown(event: KeyboardEvent) {
  if (props.disabled) return;
  if (event.key === 'Tab') { if (opened.value) close(true); return; }
  if (event.key === 'Escape' && opened.value) {
    event.preventDefault(); event.stopPropagation(); close(true); return;
  }
  if (props.searchable && event.target === searchInput.value && !['ArrowDown', 'ArrowUp', 'Enter'].includes(event.key)) return;
  if (['ArrowDown', 'ArrowUp', 'Home', 'End', 'Enter', ' '].includes(event.key)) {
    event.preventDefault();
    if (!opened.value) { void show(); return; }
    if (event.key === 'Enter' || event.key === ' ') { choose(active.value); return; }
    const count = visibleOptions.value.length;
    if (!count) return;
    if (event.key === 'Home') active.value = 0;
    else if (event.key === 'End') active.value = count - 1;
    else active.value = (active.value + (event.key === 'ArrowDown' ? 1 : -1) + count) % count;
    void nextTick(scrollActive);
  } else if (!props.searchable && event.key.length === 1 && !event.ctrlKey && !event.metaKey && !event.altKey) {
    typed = Date.now() - typedAt > 600 ? event.key : typed + event.key; typedAt = Date.now();
    if (!opened.value) void show();
    const match = props.options.findIndex(option => option.label.toLowerCase().startsWith(typed.toLowerCase()));
    if (match >= 0) { active.value = match; void nextTick(scrollActive); }
  }
}
function onBlur(event: FocusEvent) {
  if (panel.value?.contains(event.relatedTarget as Node) || trigger.value?.contains(event.relatedTarget as Node)) return;
  close();
}
function keepFocus(event: MouseEvent) {
  if (!(event.target as HTMLElement).closest('input')) event.preventDefault();
}
function outside(event: PointerEvent) {
  if (!trigger.value?.contains(event.target as Node) && !panel.value?.contains(event.target as Node)) close();
}
function schedulePosition() {
  if (!opened.value || frame) return;
  frame = requestAnimationFrame(() => { frame = 0; position(); });
}
function onResize() { if (opened.value) { position(true); } }
function onScroll(event: Event) { if (!panel.value?.contains(event.target as Node)) schedulePosition(); }
watch(query, () => { active.value = 0; if (list.value) list.value.scrollTop = 0; });
watch(() => props.disabled, disabled => { if (disabled) close(); });
watch(() => props.options, () => {
  if (!props.options.length) close();
  active.value = Math.min(active.value, Math.max(0, visibleOptions.value.length - 1)); void nextTick(schedulePosition);
});
onMounted(() => {
  document.addEventListener('pointerdown', outside); window.addEventListener('resize', onResize); window.addEventListener('scroll', onScroll, true);
  observer = new ResizeObserver(schedulePosition); if (trigger.value) observer.observe(trigger.value);
});
onUnmounted(() => {
  document.removeEventListener('pointerdown', outside); window.removeEventListener('resize', onResize); window.removeEventListener('scroll', onScroll, true); observer?.disconnect(); cancelAnimationFrame(frame);
});
</script>

<template>
  <button v-bind="$attrs" :id="triggerId" ref="trigger" type="button" role="combobox" aria-haspopup="listbox"
    :aria-expanded="opened" :aria-controls="opened ? listId : undefined" :aria-activedescendant="opened && !searchable ? `${listId}-${active}` : undefined"
    :disabled="disabled" class="select-trigger" @click="opened ? close() : show()" @keydown="onKeydown" @blur="onBlur">
    <span class="min-w-0 flex-1 text-left"><span class="block truncate" :class="!selected ? 'text-zinc-500' : ''">{{ selected?.label || placeholder }}</span>
      <span v-if="selected?.description" class="block truncate mt-0.5 text-[10px] text-zinc-500 font-normal">{{ selected.description }}</span></span>
    <ChevronDown class="w-3.5 h-3.5 shrink-0 text-zinc-500 transition-transform duration-150 motion-reduce:transition-none" :class="{ 'rotate-180': opened }" />
  </button>
  <Teleport to="body"><Transition name="popover">
    <div v-if="opened" ref="panel" :style="placement" :data-dropdown-owner="triggerId" class="select-menu select-menu-layout" @mousedown="keepFocus">
      <div v-if="searchable" class="relative shrink-0 p-1 pb-2 border-b border-white/[0.06] mb-1">
        <Search class="absolute left-3 top-3 w-3.5 h-3.5 text-zinc-500 pointer-events-none" />
        <input ref="searchInput" v-model="query" role="combobox" aria-expanded="true" aria-autocomplete="list" :aria-controls="listId"
          :aria-activedescendant="visibleOptions[active] ? `${listId}-${active}` : undefined" :aria-label="tr('Поиск в списке')" :placeholder="tr('Поиск')"
          class="w-full pl-8 pr-2 py-2 rounded-lg bg-black/20 text-xs text-zinc-200 placeholder-zinc-600" @keydown="onKeydown" @blur="onBlur" />
      </div>
      <div :id="listId" ref="list" role="listbox" :aria-labelledby="triggerId" class="min-h-0 overflow-y-auto overscroll-contain">
        <button v-for="(option, index) in visibleOptions" :id="`${listId}-${index}`" :key="option.value" type="button" role="option"
          :aria-selected="modelValue === option.value" tabindex="-1" class="select-option" :class="{ 'select-option-active': active === index }"
          @pointermove="active = index" @click="choose(index)">
          <span class="min-w-0 flex-1"><span class="block truncate">{{ option.label }}</span><span v-if="option.description" class="block truncate mt-1 text-[10px] text-zinc-500">{{ option.description }}</span></span>
          <Check v-if="modelValue === option.value" class="w-3.5 h-3.5 shrink-0 text-amber-300" />
        </button>
        <p v-if="!visibleOptions.length" role="status" class="px-3 py-4 text-xs text-zinc-500">{{ tr('Ничего не найдено') }}</p>
      </div>
    </div>
  </Transition></Teleport>
</template>
