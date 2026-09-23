<script setup lang="ts">
import { onMounted } from "vue";
import { useSessionStore } from "../../stores/sessionStore";
import { useAgentStore } from "../../stores/agentStore";
import {
  Plus,
  MessageSquare,
  Trash2,
  FolderCode,
  Clock,
} from "lucide-vue-next";

const sessionStore = useSessionStore();
const agentStore = useAgentStore();

onMounted(() => {
  sessionStore.fetchSessions();
});
</script>

<template>
  <aside
    class="w-64 border-r border-neutral-800 bg-neutral-900/40 flex flex-col shrink-0 overflow-hidden"
  >
    <!-- New Chat Button -->
    <div class="p-3 border-b border-neutral-800/80">
      <button
        @click="sessionStore.newSession"
        :disabled="agentStore.isStreaming"
        class="w-full flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-blue-600 hover:bg-blue-500 disabled:opacity-50 disabled:cursor-not-allowed text-white font-medium text-xs shadow-md shadow-blue-500/10 transition-colors cursor-pointer"
      >
        <Plus class="w-4 h-4" />
        <span>Новый диалог</span>
      </button>
    </div>

    <!-- Sessions List -->
    <div class="flex-1 overflow-y-auto p-2 space-y-1">
      <div class="px-2 py-1 text-[11px] font-semibold text-neutral-400 uppercase tracking-wider">
        История сессий
      </div>

      <div
        v-if="sessionStore.isLoading && sessionStore.sessions.length === 0"
        class="p-4 text-center text-xs text-neutral-400"
      >
        Загрузка...
      </div>

      <div
        v-else-if="sessionStore.sessions.length === 0"
        class="p-4 text-center text-xs text-neutral-400"
      >
        Нет сохраненных сессий
      </div>

      <div
        v-for="s in sessionStore.sessions"
        :key="s.id"
        @click="sessionStore.selectSession(s.id)"
        class="group relative flex items-start gap-2.5 p-2 rounded-lg text-xs cursor-pointer transition-colors"
        :class="
          agentStore.currentSession?.id === s.id
            ? 'bg-neutral-800/90 text-white font-medium shadow-sm'
            : 'text-neutral-400 hover:bg-neutral-800/40 hover:text-neutral-200'
        "
      >
        <MessageSquare class="w-3.5 h-3.5 mt-0.5 shrink-0 text-neutral-400 group-hover:text-blue-400 transition-colors" />

        <div class="flex-1 min-w-0 pr-6">
          <div class="truncate text-neutral-200 leading-tight">
            {{ s.title || s.id }}
          </div>
          <div class="flex items-center gap-1.5 mt-1 text-[10px] text-neutral-400 font-mono">
            <Clock class="w-2.5 h-2.5" />
            <span>{{ s.created_at.split(' ')[0] }}</span>
            <span>•</span>
            <span>{{ s.message_count }} сообщ.</span>
          </div>
        </div>

        <!-- Delete button -->
        <button
          @click.stop="sessionStore.deleteSession(s.id)"
          class="absolute right-2 top-2 p-1 rounded opacity-0 group-hover:opacity-100 hover:bg-rose-950/60 hover:text-rose-400 text-neutral-400 transition-all cursor-pointer"
          title="Удалить сессию"
        >
          <Trash2 class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <!-- Workspace footer -->
    <div class="p-3 border-t border-neutral-800/80 bg-neutral-950/50 flex items-center gap-2 text-xs text-neutral-400">
      <FolderCode class="w-4 h-4 text-neutral-400 shrink-0" />
      <span class="truncate font-mono text-[11px]" :title="agentStore.config?.workspace_dir">
        {{ agentStore.config?.workspace_dir || "Workspace" }}
      </span>
    </div>
  </aside>
</template>
