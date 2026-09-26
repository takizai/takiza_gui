<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useSessionStore } from "../../stores/sessionStore";
import { useAgentStore } from "../../stores/agentStore";
import {
  Plus,
  MessageSquare,
  Trash2,
  FolderCode,
  Clock,
  ArrowRight,
  FolderSync,
  X,
  Settings,
  Search,
  GitBranch,
  Gauge,
  Zap,
} from "lucide-vue-next";
import { handleWindowDrag, handleWindowDblClick } from "../../utils/window";

const props = defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits<{
  (e: "open-settings"): void;
  (e: "open-usage"): void;
}>();

const sessionStore = useSessionStore();
const agentStore = useAgentStore();

const searchQuery = ref("");
const isChangingWorkspace = ref(false);
const customWorkspacePath = ref("");
const switchError = ref<string | null>(null);

onMounted(() => {
  sessionStore.fetchSessions();
});

const isDirty = computed(() => agentStore.gitStatus?.is_dirty);
const branchName = computed(() => agentStore.gitStatus?.branch || "no-git");

const filteredSessions = computed(() => {
  const query = searchQuery.value.trim().toLowerCase();
  if (!query) return sessionStore.sessions;
  return sessionStore.sessions.filter(
    (s) =>
      (s.title && s.title.toLowerCase().includes(query)) ||
      s.id.toLowerCase().includes(query)
  );
});

// Group sessions by date
const groupedSessions = computed(() => {
  const today: typeof sessionStore.sessions = [];
  const yesterday: typeof sessionStore.sessions = [];
  const earlier: typeof sessionStore.sessions = [];

  const now = new Date();
  const todayStr = now.toISOString().split("T")[0];
  const yesterdayDate = new Date(now);
  yesterdayDate.setDate(yesterdayDate.getDate() - 1);
  const yesterdayStr = yesterdayDate.toISOString().split("T")[0];

  for (const s of filteredSessions.value) {
    const sessionDate = s.created_at.split(" ")[0];
    if (sessionDate === todayStr) {
      today.push(s);
    } else if (sessionDate === yesterdayStr) {
      yesterday.push(s);
    } else {
      earlier.push(s);
    }
  }

  return { today, yesterday, earlier };
});

async function switchWorkspace(path: string) {
  switchError.value = null;
  try {
    await agentStore.setWorkspaceDir(path);
    await sessionStore.fetchSessions();
    isChangingWorkspace.value = false;
    customWorkspacePath.value = "";
  } catch (e: any) {
    switchError.value = String(e);
  }
}
</script>

<template>
  <aside
    class="border-r border-white/[0.07] bg-[#090a0d] flex flex-col shrink-0 overflow-hidden transition-all duration-200 select-none z-20"
    :class="isOpen ? 'w-64' : 'w-0 border-r-0'"
  >
    <!-- Brand Lockup -->
    <div
      class="h-14 px-4 flex items-center justify-between shrink-0 border-b border-white/[0.06] cursor-default"
      data-tauri-drag-region
      @mousedown="handleWindowDrag"
      @dblclick="handleWindowDblClick"
    >
      <div class="flex items-center gap-2.5">
        <div class="relative">
          <img
            src="../../assets/logo.png"
            alt="Takiza"
            class="w-7 h-7 object-contain filter drop-shadow-[0_0_10px_rgba(245,158,11,0.25)]"
          />
        </div>
        <div>
          <div class="flex items-center gap-1.5">
            <span class="font-bold text-sm tracking-wider text-zinc-100 font-mono">TAKIZA</span>
            <span class="text-[9px] font-mono px-1 py-0.2 rounded bg-amber-500/10 text-amber-400 border border-amber-500/20">
              v0.1
            </span>
          </div>
          <div class="text-[10px] text-zinc-500 font-mono">Autonomous Rust Agent</div>
        </div>
      </div>
    </div>

    <!-- Quick Action / New Session -->
    <div class="p-3 space-y-2 border-b border-white/[0.06]">
      <button
        @click="sessionStore.newSession"
        :disabled="agentStore.isStreaming"
        class="w-full flex items-center justify-between px-3 py-2 rounded-xl bg-white/[0.05] hover:bg-white/[0.09] active:scale-[0.98] border border-white/[0.09] text-zinc-100 font-medium text-xs shadow-sm transition-all cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed group"
      >
        <div class="flex items-center gap-2">
          <Plus class="w-3.5 h-3.5 text-amber-400 group-hover:rotate-90 transition-transform duration-200" />
          <span>Новая сессия</span>
        </div>
        <span class="text-[10px] font-mono text-zinc-500 bg-white/[0.04] px-1.5 py-0.5 rounded border border-white/[0.06]">
          ⌘N
        </span>
      </button>

      <!-- Search Input -->
      <div class="relative">
        <Search class="w-3.5 h-3.5 text-zinc-500 absolute left-2.5 top-2.5" />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Поиск диалогов..."
          class="w-full pl-8 pr-3 py-1.5 rounded-lg bg-white/[0.02] border border-white/[0.06] text-[11px] text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-white/20 transition-colors"
        />
        <button
          v-if="searchQuery"
          @click="searchQuery = ''"
          class="absolute right-2 top-2 text-zinc-500 hover:text-zinc-200"
        >
          <X class="w-3 h-3" />
        </button>
      </div>
    </div>

    <!-- Sessions List (Grouped by Recency) -->
    <div class="flex-1 overflow-y-auto p-2 space-y-4">
      <div v-if="sessionStore.isLoading && sessionStore.sessions.length === 0" class="p-6 text-center text-xs text-zinc-500">
        Загрузка истории...
      </div>

      <div v-else-if="filteredSessions.length === 0" class="p-6 text-center text-xs text-zinc-500">
        {{ searchQuery ? "Ничего не найдено" : "Нет сохраненных сессий" }}
      </div>

      <div v-else class="space-y-3">
        <!-- Today Group -->
        <div v-if="groupedSessions.today.length > 0">
          <div class="px-2 pb-1 text-[10px] font-semibold text-zinc-500 uppercase tracking-wider font-mono">
            Сегодня
          </div>
          <div class="space-y-1">
            <div
              v-for="s in groupedSessions.today"
              :key="s.id"
              @click="sessionStore.selectSession(s.id)"
              class="group relative flex items-start gap-2.5 p-2 rounded-xl text-xs cursor-pointer transition-all border"
              :class="
                agentStore.currentSession?.id === s.id
                  ? 'bg-white/[0.08] border-white/[0.12] text-zinc-100 font-medium shadow-sm'
                  : 'border-transparent text-zinc-400 hover:bg-white/[0.03] hover:text-zinc-200'
              "
            >
              <div
                v-if="agentStore.currentSession?.id === s.id"
                class="absolute -left-2 top-2 bottom-2 w-1 rounded-r bg-amber-400"
              />
              <MessageSquare class="w-3.5 h-3.5 mt-0.5 shrink-0 text-zinc-400 group-hover:text-zinc-200 transition-colors" />
              <div class="flex-1 min-w-0 pr-5">
                <div class="truncate text-zinc-200 leading-tight">
                  {{ s.title || s.id }}
                </div>
                <div class="flex items-center gap-1.5 mt-1 text-[10px] text-zinc-500 font-mono">
                  <span>{{ s.message_count }} сообщ.</span>
                </div>
              </div>
              <button
                @click.stop="sessionStore.deleteSession(s.id)"
                class="absolute right-2 top-2 p-1 rounded-lg opacity-0 group-hover:opacity-100 hover:bg-rose-500/20 hover:text-rose-300 text-zinc-500 transition-all cursor-pointer"
                title="Удалить"
              >
                <Trash2 class="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        </div>

        <!-- Yesterday Group -->
        <div v-if="groupedSessions.yesterday.length > 0">
          <div class="px-2 pb-1 text-[10px] font-semibold text-zinc-500 uppercase tracking-wider font-mono">
            Вчера
          </div>
          <div class="space-y-1">
            <div
              v-for="s in groupedSessions.yesterday"
              :key="s.id"
              @click="sessionStore.selectSession(s.id)"
              class="group relative flex items-start gap-2.5 p-2 rounded-xl text-xs cursor-pointer transition-all border"
              :class="
                agentStore.currentSession?.id === s.id
                  ? 'bg-white/[0.08] border-white/[0.12] text-zinc-100 font-medium shadow-sm'
                  : 'border-transparent text-zinc-400 hover:bg-white/[0.03] hover:text-zinc-200'
              "
            >
              <div
                v-if="agentStore.currentSession?.id === s.id"
                class="absolute -left-2 top-2 bottom-2 w-1 rounded-r bg-amber-400"
              />
              <MessageSquare class="w-3.5 h-3.5 mt-0.5 shrink-0 text-zinc-400 group-hover:text-zinc-200 transition-colors" />
              <div class="flex-1 min-w-0 pr-5">
                <div class="truncate text-zinc-200 leading-tight">
                  {{ s.title || s.id }}
                </div>
                <div class="flex items-center gap-1.5 mt-1 text-[10px] text-zinc-500 font-mono">
                  <span>{{ s.message_count }} сообщ.</span>
                </div>
              </div>
              <button
                @click.stop="sessionStore.deleteSession(s.id)"
                class="absolute right-2 top-2 p-1 rounded-lg opacity-0 group-hover:opacity-100 hover:bg-rose-500/20 hover:text-rose-300 text-zinc-500 transition-all cursor-pointer"
                title="Удалить"
              >
                <Trash2 class="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        </div>

        <!-- Earlier Group -->
        <div v-if="groupedSessions.earlier.length > 0">
          <div class="px-2 pb-1 text-[10px] font-semibold text-zinc-500 uppercase tracking-wider font-mono">
            Ранее
          </div>
          <div class="space-y-1">
            <div
              v-for="s in groupedSessions.earlier"
              :key="s.id"
              @click="sessionStore.selectSession(s.id)"
              class="group relative flex items-start gap-2.5 p-2 rounded-xl text-xs cursor-pointer transition-all border"
              :class="
                agentStore.currentSession?.id === s.id
                  ? 'bg-white/[0.08] border-white/[0.12] text-zinc-100 font-medium shadow-sm'
                  : 'border-transparent text-zinc-400 hover:bg-white/[0.03] hover:text-zinc-200'
              "
            >
              <div
                v-if="agentStore.currentSession?.id === s.id"
                class="absolute -left-2 top-2 bottom-2 w-1 rounded-r bg-amber-400"
              />
              <MessageSquare class="w-3.5 h-3.5 mt-0.5 shrink-0 text-zinc-400 group-hover:text-zinc-200 transition-colors" />
              <div class="flex-1 min-w-0 pr-5">
                <div class="truncate text-zinc-200 leading-tight">
                  {{ s.title || s.id }}
                </div>
                <div class="flex items-center gap-1.5 mt-1 text-[10px] text-zinc-500 font-mono">
                  <Clock class="w-2.5 h-2.5 text-zinc-600" />
                  <span>{{ s.created_at.split(' ')[0] }}</span>
                  <span>•</span>
                  <span>{{ s.message_count }} сообщ.</span>
                </div>
              </div>
              <button
                @click.stop="sessionStore.deleteSession(s.id)"
                class="absolute right-2 top-2 p-1 rounded-lg opacity-0 group-hover:opacity-100 hover:bg-rose-500/20 hover:text-rose-300 text-zinc-500 transition-all cursor-pointer"
                title="Удалить"
              >
                <Trash2 class="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Workspace & Project Footer Card -->
    <div class="border-t border-white/[0.07] bg-[#07080a] p-3 space-y-2">
      <!-- Switcher Popover -->
      <div v-if="isChangingWorkspace" class="p-3 rounded-xl border border-white/[0.08] space-y-2 bg-[#121418] mb-2">
        <div class="flex items-center justify-between text-xs font-semibold text-zinc-200">
          <span>Сменить проект</span>
          <button @click="isChangingWorkspace = false" class="text-zinc-400 hover:text-white cursor-pointer">
            <X class="w-3.5 h-3.5" />
          </button>
        </div>

        <button
          @click="switchWorkspace('~/takiza-harness')"
          class="w-full text-left p-1.5 rounded-lg bg-white/[0.04] hover:bg-white/[0.08] border border-white/[0.08] text-[11px] font-mono text-zinc-300 flex items-center justify-between cursor-pointer"
        >
          <span>~/takiza-harness</span>
          <FolderSync class="w-3 h-3 text-zinc-400" />
        </button>

        <div class="flex items-center gap-1.5">
          <input
            v-model="customWorkspacePath"
            @keydown.enter="customWorkspacePath.trim() && switchWorkspace(customWorkspacePath.trim())"
            placeholder="/путь/к/папке"
            class="flex-1 px-2 py-1.5 bg-[#090a0c] border border-white/[0.1] rounded-lg text-[11px] font-mono text-zinc-200 focus:outline-none focus:border-white/30"
          />
          <button
            @click="customWorkspacePath.trim() && switchWorkspace(customWorkspacePath.trim())"
            class="p-1.5 rounded-lg bg-white/[0.08] hover:bg-white/[0.15] text-zinc-200 border border-white/[0.1] cursor-pointer"
          >
            <ArrowRight class="w-3 h-3" />
          </button>
        </div>

        <div v-if="switchError" class="text-[10px] text-rose-400 leading-tight">
          {{ switchError }}
        </div>
      </div>

      <!-- Mode & Usage Quota Summary Pill -->
      <div
        @click="emit('open-usage')"
        class="mb-2 p-2 rounded-xl border border-white/[0.06] bg-white/[0.02] hover:bg-white/[0.05] transition-all cursor-pointer flex items-center justify-between gap-2 select-none group"
        title="Нажмите для просмотра квот и расхода токенов"
      >
        <div class="flex items-center gap-2 min-w-0">
          <div
            class="w-6 h-6 rounded-lg flex items-center justify-center shrink-0 border"
            :class="
              agentStore.config?.mode === 'moa'
                ? 'bg-emerald-500/15 border-emerald-500/30 text-emerald-400'
                : 'bg-amber-500/15 border-amber-500/30 text-amber-400'
            "
          >
            <Zap v-if="agentStore.config?.mode === 'moa'" class="w-3 h-3 fill-current" />
            <Gauge v-else class="w-3 h-3" />
          </div>
          <div class="min-w-0">
            <div class="text-[11px] font-semibold text-zinc-200 truncate flex items-center gap-1.5">
              <span>{{ agentStore.config?.mode === 'moa' ? 'Takiza MoA' : 'Takiza Manual' }}</span>
              <span
                v-if="agentStore.config?.mode === 'moa'"
                class="text-[9px] font-mono px-1 py-0.1 rounded bg-emerald-500/20 text-emerald-300"
              >
                ⚡ ~45%
              </span>
            </div>
            <div class="text-[10px] font-mono text-zinc-400">
              {{ agentStore.config?.mode === 'moa' ? '210k / 1.0M (21%)' : '520k / 1.0M (52%)' }}
            </div>
          </div>
        </div>

        <Gauge class="w-3.5 h-3.5 text-zinc-500 group-hover:text-zinc-300 transition-colors shrink-0" />
      </div>

      <!-- Project Status Card -->
      <div
        class="p-2.5 rounded-xl border border-white/[0.06] bg-white/[0.02] hover:bg-white/[0.04] transition-all cursor-pointer flex items-center justify-between gap-2"
        @click="isChangingWorkspace = !isChangingWorkspace"
        title="Нажмите для смены директории проекта"
      >
        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-1.5 text-zinc-200 text-xs font-medium truncate">
            <FolderCode class="w-3.5 h-3.5 text-amber-400 shrink-0" />
            <span class="truncate font-mono">{{ agentStore.config?.workspace_dir?.split('/').pop() || "Workspace" }}</span>
          </div>
          <div class="flex items-center gap-1.5 mt-1 text-[10px] text-zinc-500 font-mono">
            <GitBranch class="w-3 h-3 text-zinc-500" />
            <span class="truncate">{{ branchName }}</span>
            <span v-if="isDirty" class="w-1.5 h-1.5 rounded-full bg-amber-400 animate-pulse" />
          </div>
        </div>

        <button
          @click.stop="emit('open-settings')"
          class="p-1.5 rounded-lg hover:bg-white/[0.08] text-zinc-400 hover:text-zinc-100 transition-colors"
          title="Настройки"
        >
          <Settings class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  </aside>
</template>
