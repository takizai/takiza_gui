<script setup lang="ts">
import { useI18n } from "../../i18n";
import { nextTick, onBeforeUnmount, ref, watch } from "vue";
import { X } from "lucide-vue-next";
import { modalStack } from "../../utils/modalStack";

const { tr } = useI18n();

const props = withDefaults(defineProps<{
  isOpen: boolean;
  title: string;
  size?: "sm" | "md" | "lg" | "xl";
  dismissible?: boolean;
  panelClass?: string;
  showHeader?: boolean;
}>(), { size: "md", dismissible: true, panelClass: "", showHeader: true });
const emit = defineEmits<{ (e: "close"): void }>();
const panel = ref<HTMLElement | null>(null);
const token = Symbol("modal");
let previousFocus: HTMLElement | null = null;
let registered = false;
const isTop = () => modalStack[modalStack.length - 1] === token;
function focusable() {
  return Array.from(panel.value?.querySelectorAll<HTMLElement>(
    'button:not([disabled]), a[href], input:not([disabled]), textarea:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])'
  ) || []).filter(element => element.getClientRects().length && !element.closest('[inert]'));
}
function requestClose() { if (isTop() && props.dismissible) emit("close"); }
function ownedDropdown(target: EventTarget | null) {
  const owner = target instanceof HTMLElement ? target.closest('[data-dropdown-owner]')?.getAttribute('data-dropdown-owner') : null;
  return !!owner && !!panel.value?.contains(document.getElementById(owner));
}
function onKeydown(event: KeyboardEvent) {
  if (!props.isOpen || !isTop()) return;
  if (event.key === "Escape") {
    if ((event.target as HTMLElement)?.closest('[role="combobox"][aria-expanded="true"]')) return;
    event.preventDefault(); event.stopPropagation(); requestClose();
  } else if (event.key === "Tab") {
    if (ownedDropdown(event.target)) return;
    const elements = focusable();
    const first = elements[0]; const last = elements[elements.length - 1];
    if (!first) { event.preventDefault(); panel.value?.focus(); return; }
    if (!panel.value?.contains(document.activeElement) || (event.shiftKey && (document.activeElement === first || document.activeElement === panel.value))) {
      event.preventDefault(); (event.shiftKey ? last : first)?.focus();
    } else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
  }
}
function containFocus(event: FocusEvent) {
  if (props.isOpen && isTop() && !panel.value?.contains(event.target as Node) && !ownedDropdown(event.target)) (focusable()[0] || panel.value)?.focus();
}
function release() {
  if (!registered) return;
  const wasTop = isTop();
  const index = modalStack.indexOf(token);
  if (index !== -1) modalStack.splice(index, 1);
  registered = false;
  document.removeEventListener("keydown", onKeydown, true);
  document.removeEventListener("focusin", containFocus);
  if (wasTop && previousFocus?.isConnected) previousFocus.focus({ preventScroll: true });
}
watch(() => props.isOpen, async open => {
  if (!open) return;
  if (!registered) {
    previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    modalStack.push(token); registered = true;
    document.addEventListener("keydown", onKeydown, true);
    document.addEventListener("focusin", containFocus);
  }
  await nextTick();
  if (props.isOpen && isTop()) (focusable()[0] || panel.value)?.focus({ preventScroll: true });
}, { immediate: true });
onBeforeUnmount(release);
</script>

<template>
  <Teleport to="body">
    <Transition name="modal" appear @after-leave="release">
      <div v-if="isOpen" class="modal-backdrop" @pointerdown.self="requestClose">
        <section ref="panel" role="dialog" aria-modal="true" :aria-label="title" tabindex="-1"
          class="modal-panel" :class="[`modal-size-${size}`, panelClass]">
          <header v-if="showHeader" class="modal-header">
            <h2 class="text-sm font-medium text-zinc-200 truncate">{{ title }}</h2>
            <button v-if="dismissible" class="quiet-icon" :aria-label='tr("Закрыть окно")' @click="requestClose"><X class="w-4 h-4" /></button>
          </header>
          <slot />
        </section>
      </div>
    </Transition>
  </Teleport>
</template>
