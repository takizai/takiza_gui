<script setup lang="ts">
import { useI18n } from "../../i18n";
import BaseModal from "../common/BaseModal.vue";
import { ref, computed, watch, onUnmounted } from "vue";
import { Search, BookOpen, ArrowLeft, ArrowUpRight, Check, Copy, Loader2, RefreshCw } from "lucide-vue-next";
import { useAgentStore } from "../../stores/agentStore";
import { renderMarkdown } from "../../utils/markdown";
import type { Skill } from "../../types";

const { tr } = useI18n();
const props = defineProps<{ isOpen: boolean }>();
const emit = defineEmits<{ (e: "close"): void; (e: "insert-skill", name: string): void }>();
const agent = useAgentStore();
const search = ref("");
const selected = ref<Skill | null>(null);
const content = ref("");
const loading = ref(false);
const listLoading = ref(false);
const error = ref("");
const listError = ref("");
const detailVisible = ref(false);
const copied = ref(false);
let request = 0;
let listRequest = 0;
let copyTimer: ReturnType<typeof setTimeout> | undefined;
const filtered = computed(() => {
  const query = search.value.trim().toLowerCase();
  return agent.availableSkills.filter(skill => !query || `${skill.name} ${skill.description}`.toLowerCase().includes(query));
});
const rendered = computed(() => renderMarkdown(content.value));
async function selectSkill(skill: Skill, showDetail = true) {
  const revision = ++request;
  selected.value = skill; detailVisible.value = showDetail;
  content.value = ""; error.value = ""; loading.value = true; copied.value = false;
  try { const text = await agent.readSkill(skill.name); if (revision === request) content.value = text; }
  catch { if (revision === request) error.value = tr("Не удалось загрузить содержимое SKILL.md"); }
  finally { if (revision === request) loading.value = false; }
}
async function refreshSkills() {
  const revision = ++listRequest;
  listLoading.value = true; listError.value = "";
  try {
    await agent.fetchSkills();
    if (revision !== listRequest || !props.isOpen) return;
    const next = agent.availableSkills.find(skill => skill.name === selected.value?.name) || agent.availableSkills[0];
    if (next) await selectSkill(next, false);
    else { ++request; selected.value = null; content.value = ""; loading.value = false; }
  } catch { if (revision === listRequest) listError.value = tr("Не удалось загрузить список скиллов"); }
  finally { if (revision === listRequest) listLoading.value = false; }
}
watch(() => props.isOpen, open => {
  if (open) { search.value = ""; detailVisible.value = false; void refreshSkills(); }
  else { ++request; ++listRequest; loading.value = false; listLoading.value = false; }
}, { immediate: true });
onUnmounted(() => { ++request; ++listRequest; clearTimeout(copyTimer); });
async function copyName() {
  if (!selected.value) return;
  try { await navigator.clipboard.writeText(`@${selected.value.name}`); copied.value = true; clearTimeout(copyTimer); copyTimer = setTimeout(() => copied.value = false, 1500); }
  catch { error.value = tr("Не удалось скопировать имя скилла"); }
}
function insert() { if (selected.value) { emit("insert-skill", selected.value.name); emit("close"); } }
</script>

<template>
  <BaseModal :is-open="isOpen" :title='tr("Библиотека скиллов")' size="xl" panel-class="skills-modal" @close="emit('close')">
    <div class="skills-layout" :class="{ 'skills-show-detail': detailVisible }">
      <aside class="skills-browser" :aria-label='tr("Список скиллов")'>
        <div class="flex items-center gap-2 p-3 border-b border-white/[0.06]">
          <div class="relative min-w-0 flex-1">
            <Search class="absolute left-2.5 top-2.5 w-3.5 h-3.5 text-zinc-500" />
            <input v-model="search" :aria-label='tr("Поиск скиллов")' :placeholder='tr("Поиск скиллов…")' class="w-full rounded-lg bg-white/[0.035] pl-8 pr-2 py-2 text-xs text-zinc-300" />
          </div>
          <button @click="refreshSkills" :disabled="listLoading" class="quiet-icon" :aria-label='tr("Обновить список скиллов")'><RefreshCw class="w-3.5 h-3.5" :class="{ 'animate-spin': listLoading }" /></button>
        </div>
        <p class="px-4 pt-3 pb-1 text-[10px] text-zinc-500">{{ tr("Скиллы ·") }} {{ filtered.length }}</p>
        <nav class="min-h-0 flex-1 overflow-y-auto overscroll-contain p-2" :aria-label='tr("Доступные скиллы")'>
          <p v-if="listLoading && !agent.availableSkills.length" class="p-3 text-xs text-zinc-500">{{ tr("Загрузка…") }}</p>
          <p v-else-if="listError" role="alert" class="p-3 text-xs text-rose-300">{{ listError }}</p>
          <p v-else-if="!filtered.length" class="p-3 text-xs text-zinc-500">{{ search ? tr("Ничего не найдено") : tr("Скиллы пока не установлены") }}</p>
          <button v-for="skill in filtered" :key="skill.name" @click="selectSkill(skill)" :aria-pressed="selected?.name === skill.name"
            class="skill-list-item" :class="{ 'skill-selected': selected?.name === skill.name }">
            <span class="block truncate text-xs text-zinc-200">{{ skill.name }}</span>
            <span class="block mt-1 text-[11px] leading-relaxed text-zinc-500 line-clamp-2">{{ skill.description }}</span>
            <span class="block mt-2 text-[10px] text-zinc-600">{{ skill.is_workspace ? tr("Проект") : tr("Глобальный") }}</span>
          </button>
        </nav>
      </aside>
      <section class="skills-detail" :aria-label='tr("Просмотр скилла")'>
        <template v-if="selected">
          <div class="flex items-start gap-3 p-4 border-b border-white/[0.06] shrink-0">
            <button @click="detailVisible = false" class="quiet-icon skills-back" :aria-label='tr("К списку скиллов")'><ArrowLeft class="w-4 h-4" /></button>
            <div class="min-w-0 flex-1">
              <h3 class="text-sm font-medium text-zinc-200 break-words">{{ selected.name }}</h3>
              <p class="mt-1 text-xs leading-relaxed text-zinc-500 line-clamp-2">{{ selected.description }}</p>
            </div>
          </div>
          <div class="flex-1 min-h-0 overflow-y-auto overscroll-contain p-4 sm:p-5 select-text">
            <p v-if="loading" role="status" class="flex items-center gap-2 text-xs text-zinc-500"><Loader2 class="w-3.5 h-3.5 animate-spin" />{{ tr("Загрузка содержимого…") }}</p>
            <div v-else-if="error" role="alert" class="text-xs text-zinc-400"><p>{{ error }}</p><button @click="selectSkill(selected)" class="mt-2 underline cursor-pointer">{{ tr("Повторить") }}</button></div>
            <div v-else-if="content" class="prose-takiza skills-markdown" v-html="rendered" />
            <p v-else class="text-xs text-zinc-500">{{ tr("Скилл не содержит документации.") }}</p>
          </div>
          <footer class="flex flex-wrap items-center justify-between gap-2 px-4 py-3 border-t border-white/[0.06] shrink-0">
            <button @click="copyName" class="message-action-button" :aria-label='tr("Скопировать имя скилла")'><Check v-if="copied" class="w-3 h-3" /><Copy v-else class="w-3 h-3" />{{ copied ? tr("Скопировано") : tr("Копировать имя") }}</button>
            <button @click="insert" class="editor-save"><span>{{ tr("Использовать в чате") }}</span><ArrowUpRight class="w-3 h-3" /></button>
          </footer>
        </template>
        <div v-else class="flex flex-1 flex-col items-center justify-center gap-3 p-6 text-center text-xs text-zinc-500"><BookOpen class="w-6 h-6 text-zinc-600" /><p>{{ tr("Выберите скилл, чтобы прочитать инструкции.") }}</p></div>
      </section>
    </div>
  </BaseModal>
</template>
