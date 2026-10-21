<script setup lang="ts">
import { useI18n } from "../../i18n";
import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import BaseModal from "../common/BaseModal.vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useSessionStore } from "../../stores/sessionStore";
import { useAgentStore } from "../../stores/agentStore";
import { Plus, Trash2, FolderCode, X, Search, LoaderCircle } from "lucide-vue-next";
import { handleWindowDrag, handleWindowDblClick } from "../../utils/window";

const { tr } = useI18n();

const props = defineProps<{ isOpen: boolean }>();
const emit = defineEmits<{ (e: "close"): void }>();
const sessionStore = useSessionStore();
watch(() => props.isOpen, open => { if (open) void sessionStore.fetchSessions(); });
const agentStore = useAgentStore();
const searchQuery = ref("");
const switchError = ref("");
const switching = ref(false);
const pendingWorkspace = ref('');
async function confirmWorkspace() {
  if (!pendingWorkspace.value || switching.value || agentStore.operationBusy) return;
  const path = pendingWorkspace.value;
  switching.value = true;
  switchError.value = '';
  try {
    await agentStore.setWorkspaceDir(path);
    pendingWorkspace.value = '';
    await sessionStore.fetchSessions();
  } catch (error) { switchError.value = String(error); }
  finally { switching.value = false; }
}
const filteredSessions = computed(() => {
  const query = searchQuery.value.trim().toLowerCase();
  return query ? sessionStore.sessions.filter(s => (s.title || s.id).toLowerCase().includes(query)) : sessionStore.sessions;
});
async function switchWorkspace() {
  if (switching.value || pendingWorkspace.value || agentStore.operationBusy) return;
  switching.value = true;
  switchError.value = "";
  try {
    const path = await open({
      directory: true,
      multiple: false,
      title: tr("Выбрать папку проекта"),
      defaultPath: agentStore.config?.workspace_dir || undefined,
    });
    if (!path || path === agentStore.config?.workspace_dir) return;
    const initialized = await invoke<boolean>('workspace_has_takiza', { path });
    if (!initialized) { pendingWorkspace.value = path; return; }
    await agentStore.setWorkspaceDir(path);
    await sessionStore.fetchSessions();
  } catch (error) { switchError.value = String(error); }
  finally { switching.value = false; }
}
</script>

<template>
  <Transition name="history">
  <aside v-if="isOpen" :inert="!isOpen" class="history-sidebar w-60 h-full border-r border-white/[0.07] bg-[#101114] flex flex-col shrink-0 z-20" :aria-label='tr("История диалогов")'>
    <div class="h-12 px-4 flex items-center justify-between shrink-0" data-tauri-drag-region @mousedown="handleWindowDrag" @dblclick="handleWindowDblClick">
      <span class="text-sm font-medium text-zinc-300">{{ tr("Диалоги") }}</span>
      <button class="quiet-icon" @click="emit('close')" :aria-label='tr("Закрыть историю")'><X class="w-4 h-4" /></button>
    </div>
    <div class="px-3 pb-3 space-y-3">
      <button @click="sessionStore.newSession" :disabled="agentStore.isStreaming" class="flex items-center gap-2 w-full px-3 py-2 rounded-lg bg-white/[0.05] hover:bg-white/[0.08] text-xs text-zinc-200 disabled:opacity-40 cursor-pointer">
        <Plus class="w-4 h-4" />{{ tr("Новый диалог") }}</button>
      <div class="relative">
        <Search class="w-3.5 h-3.5 absolute left-2 top-2 text-zinc-500" />
        <input v-model="searchQuery" :aria-label='tr("Поиск диалогов")' :placeholder='tr("Поиск")' class="w-full rounded-md pl-7 pr-2 py-1.5 text-xs bg-transparent text-zinc-300 placeholder-zinc-500" />
      </div>
    </div>
    <nav class="flex-1 overflow-y-auto min-h-0 px-2 pb-3" :aria-label='tr("Сохранённые диалоги")'>
      <p v-if="sessionStore.isLoading && !sessionStore.sessions.length" class="px-3 py-4 text-xs text-zinc-500">{{ tr("Загрузка…") }}</p>
      <p v-else-if="!filteredSessions.length" class="px-3 py-4 text-xs text-zinc-500">{{ searchQuery ? tr("Ничего не найдено") : tr("Здесь появятся ваши диалоги") }}</p>
      <div v-for="session in filteredSessions" :key="session.id" class="session-row group flex items-center rounded-lg mb-0.5"
        :class="agentStore.currentSession?.id === session.id ? 'bg-white/[0.07] text-zinc-100' : 'text-zinc-400 hover:bg-white/[0.03]'">
        <button @click="sessionStore.selectSession(session.id)" :aria-current="agentStore.currentSession?.id === session.id ? 'page' : undefined"
          class="flex-1 min-w-0 text-left truncate px-3 py-2 text-xs cursor-pointer" :title="session.title || session.id">{{ session.title || session.id }}</button>
        <button @click="sessionStore.deleteSession(session.id)" class="quiet-icon session-delete mr-1" :aria-label="tr('Удалить диалог {0}', {'0': session.title || session.id})"><Trash2 class="w-3.5 h-3.5" /></button>
      </div>
    </nav>
    <div class="border-t border-white/[0.07] p-3">
      <button class="flex items-center gap-2 w-full text-xs text-zinc-400 hover:text-zinc-200 py-1 cursor-pointer disabled:opacity-40 disabled:cursor-default transition-colors" :title="agentStore.config?.workspace_dir || tr('Выбрать папку проекта')"
        :aria-label="tr('Выбрать папку проекта')" :aria-busy="switching" :disabled="switching || agentStore.operationBusy" @click="switchWorkspace">
        <LoaderCircle v-if="switching" class="w-4 h-4 shrink-0 animate-spin motion-reduce:animate-none" /><FolderCode v-else class="w-4 h-4 shrink-0" />
        <span class="truncate flex-1 text-left">{{ switching ? tr("Открытие…") : agentStore.config?.workspace_dir?.split(/[\\/]/).filter(Boolean).pop() || tr("Выбрать проект") }}</span>
      </button>
      <p v-if="switchError" role="alert" class="mt-2 text-xs text-rose-300 break-words">{{ tr(switchError) }}</p>
    </div>
  </aside>
  </Transition>
  <BaseModal :is-open="!!pendingWorkspace" :title="tr('Новая папка проекта')" size="sm" :dismissible="!switching" @close="pendingWorkspace = ''; switchError = ''">
    <div class="px-5 py-4 space-y-3 text-sm text-zinc-400">
      <p>{{ tr('В этой папке ещё не использовалась Takiza. При работе здесь будут сохраняться диалоги, вложения и контрольные точки в папке .takiza.') }}</p>
      <p class="break-all text-xs text-zinc-500">{{ pendingWorkspace }}</p>
      <p v-if="switchError" role="alert" class="text-xs text-rose-300">{{ tr(switchError) }}</p>
    </div>
    <div class="flex justify-end gap-2 px-5 pb-5">
      <button type="button" class="px-3 py-2 rounded-lg text-xs text-zinc-400 hover:bg-white/5 disabled:opacity-40" :disabled="switching" @click="pendingWorkspace = ''; switchError = ''">{{ tr('Отмена') }}</button>
      <button type="button" class="px-3 py-2 rounded-lg text-xs text-zinc-200 bg-white/10 hover:bg-white/15 disabled:opacity-40" :disabled="switching || agentStore.operationBusy" @click="confirmWorkspace">{{ switching ? tr('Открытие…') : tr('Открыть папку') }}</button>
    </div>
  </BaseModal>
</template>
