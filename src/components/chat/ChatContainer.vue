<script setup lang="ts">
import { ref, watch, nextTick, computed } from "vue";
import { useAgentStore } from "../../stores/agentStore";
import ChatMessageItem from "./ChatMessageItem.vue";
import ThinkingBlock from "./ThinkingBlock.vue";
import ToolCallCard from "./ToolCallCard.vue";
import { renderMarkdown } from "../../utils/markdown";
import {
  Terminal,
  FileCode,
  CheckCircle,
  GitPullRequest,
  ArrowDown,
  Zap,
} from "lucide-vue-next";

const agentStore = useAgentStore();
const feedRef = ref<HTMLDivElement | null>(null);
const showScrollBottom = ref(false);

const visibleHistoryItems = computed(() => {
  const history = agentStore.currentSession?.history || [];
  return history.filter((item) => {
    if (!item || !item.type) return false;
    if (item.type === ("ToolLog" as string) || item.type === ("ToolStart" as string)) return false;
    if (item.type === "MoaRouting" && (!item.payload || !item.payload.model)) return false;
    if (item.type === "Thought" && (!item.payload || !item.payload.trim())) return false;
    if (item.type === "AssistantMessage" && (!item.payload || !item.payload.trim())) return false;
    if (item.type === "UserPrompt" && (!item.payload || !item.payload.trim())) return false;
    return true;
  });
});

const streamingAssistantHtml = computed(() => {
  return renderMarkdown(agentStore.currentStreamAssistant);
});

function handleScroll() {
  if (!feedRef.value) return;
  const { scrollTop, scrollHeight, clientHeight } = feedRef.value;
  showScrollBottom.value = scrollHeight - (scrollTop + clientHeight) > 120;
}

function scrollToBottom(smooth = false) {
  nextTick(() => {
    if (feedRef.value) {
      feedRef.value.scrollTo({
        top: feedRef.value.scrollHeight,
        behavior: smooth ? "smooth" : "auto",
      });
    }
  });
}

watch(
  () => [
    visibleHistoryItems.value.length,
    agentStore.currentStreamAssistant,
    agentStore.currentStreamThought,
    agentStore.streamingToolLogs.length,
    agentStore.activeToolCall,
  ],
  () => {
    if (!showScrollBottom.value) {
      scrollToBottom();
    }
  },
  { deep: true }
);

function sendQuickPrompt(text: string) {
  agentStore.sendPrompt(text);
}
</script>

<template>
  <div
    ref="feedRef"
    @scroll="handleScroll"
    class="flex-1 overflow-y-auto px-4 sm:px-6 py-6 select-text relative"
  >
    <!-- Empty State / Welcome Canvas -->
    <div
      v-if="visibleHistoryItems.length === 0 && !agentStore.isStreaming"
      class="min-h-[580px] flex flex-col items-center justify-center text-center p-4 max-w-2xl mx-auto"
    >
      <!-- Hero Logo Emblem with Atmospheric Glow -->
      <div class="relative mb-6 select-none group">
        <div class="absolute -inset-6 bg-amber-500/10 rounded-full blur-3xl opacity-70 group-hover:opacity-100 transition-opacity" />
        <div class="absolute -inset-2 bg-white/5 rounded-full blur-xl" />
        <img
          src="../../assets/logo.png"
          alt="Takiza"
          class="relative w-24 h-24 object-contain filter drop-shadow-[0_0_30px_rgba(245,158,11,0.25)] transition-transform duration-500 group-hover:scale-105"
        />
      </div>

      <div class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-white/[0.04] border border-white/[0.08] text-[11px] font-mono text-zinc-400 mb-3">
        <span class="w-1.5 h-1.5 rounded-full bg-amber-400"></span>
        <span>Takiza Autonomous Agent</span>
      </div>

      <h2 class="text-2xl font-semibold text-zinc-100 tracking-tight mb-2">
        What do you want to build today?
      </h2>
      <p class="text-xs sm:text-sm text-zinc-400 mb-8 max-w-md leading-relaxed">
        Сформулируйте задачу, и автономный агент спланирует шаги, выполнит команды в терминале и внесет изменения в проект.
      </p>

      <!-- Quick Action Cards Grid -->
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 w-full text-left">
        <button
          @click="sendQuickPrompt('Проанализируй структуру и стек технологий текущего проекта')"
          class="p-4 rounded-2xl border border-white/[0.08] bg-[#0c0d11]/80 hover:bg-[#12141a] hover:border-amber-500/40 transition-all text-xs text-zinc-300 flex items-start gap-3.5 cursor-pointer group active:scale-[0.99] shadow-lg hover:shadow-amber-500/5"
        >
          <div class="p-2.5 rounded-xl bg-white/[0.04] border border-white/[0.06] group-hover:border-amber-500/30 group-hover:bg-amber-500/10 transition-colors">
            <FileCode class="w-4 h-4 text-zinc-300 group-hover:text-amber-400 transition-colors" />
          </div>
          <div>
            <div class="flex items-center gap-1.5">
              <span class="font-medium text-zinc-100 text-xs">Архитектура</span>
              <span class="text-[9px] font-mono px-1 py-0.2 rounded bg-white/[0.06] text-zinc-400 uppercase">Code</span>
            </div>
            <div class="text-[11px] text-zinc-500 mt-1 leading-relaxed">Обзор структуры файлов, зависимостей и модулей</div>
          </div>
        </button>

        <button
          @click="sendQuickPrompt('Покажи текущий git status и последние изменения в кодовой базе (git diff)')"
          class="p-4 rounded-2xl border border-white/[0.08] bg-[#0c0d11]/80 hover:bg-[#12141a] hover:border-amber-500/40 transition-all text-xs text-zinc-300 flex items-start gap-3.5 cursor-pointer group active:scale-[0.99] shadow-lg hover:shadow-amber-500/5"
        >
          <div class="p-2.5 rounded-xl bg-white/[0.04] border border-white/[0.06] group-hover:border-amber-500/30 group-hover:bg-amber-500/10 transition-colors">
            <GitPullRequest class="w-4 h-4 text-zinc-300 group-hover:text-amber-400 transition-colors" />
          </div>
          <div>
            <div class="flex items-center gap-1.5">
              <span class="font-medium text-zinc-100 text-xs">Git статус и diff</span>
              <span class="text-[9px] font-mono px-1 py-0.2 rounded bg-white/[0.06] text-zinc-400 uppercase">VCS</span>
            </div>
            <div class="text-[11px] text-zinc-500 mt-1 leading-relaxed">Проверить модифицированные файлы и состояние ветки</div>
          </div>
        </button>

        <button
          @click="sendQuickPrompt('Запусти проверку сборки и тестов проекта и проанализируй ошибки')"
          class="p-4 rounded-2xl border border-white/[0.08] bg-[#0c0d11]/80 hover:bg-[#12141a] hover:border-amber-500/40 transition-all text-xs text-zinc-300 flex items-start gap-3.5 cursor-pointer group active:scale-[0.99] shadow-lg hover:shadow-amber-500/5"
        >
          <div class="p-2.5 rounded-xl bg-white/[0.04] border border-white/[0.06] group-hover:border-amber-500/30 group-hover:bg-amber-500/10 transition-colors">
            <CheckCircle class="w-4 h-4 text-zinc-300 group-hover:text-amber-400 transition-colors" />
          </div>
          <div>
            <div class="flex items-center gap-1.5">
              <span class="font-medium text-zinc-100 text-xs">Тестирование</span>
              <span class="text-[9px] font-mono px-1 py-0.2 rounded bg-white/[0.06] text-zinc-400 uppercase">Check</span>
            </div>
            <div class="text-[11px] text-zinc-500 mt-1 leading-relaxed">Выполнить тесты (cargo / npm) и исправить ошибки</div>
          </div>
        </button>

        <button
          @click="sendQuickPrompt('Помоги составить пошаговый план реализации новой задачи')"
          class="p-4 rounded-2xl border border-white/[0.08] bg-[#0c0d11]/80 hover:bg-[#12141a] hover:border-amber-500/40 transition-all text-xs text-zinc-300 flex items-start gap-3.5 cursor-pointer group active:scale-[0.99] shadow-lg hover:shadow-amber-500/5"
        >
          <div class="p-2.5 rounded-xl bg-white/[0.04] border border-white/[0.06] group-hover:border-amber-500/30 group-hover:bg-amber-500/10 transition-colors">
            <Terminal class="w-4 h-4 text-zinc-300 group-hover:text-amber-400 transition-colors" />
          </div>
          <div>
            <div class="flex items-center gap-1.5">
              <span class="font-medium text-zinc-100 text-xs">План реализации</span>
              <span class="text-[9px] font-mono px-1 py-0.2 rounded bg-white/[0.06] text-zinc-400 uppercase">Plan</span>
            </div>
            <div class="text-[11px] text-zinc-500 mt-1 leading-relaxed">Сформировать шаги решения с разбором зависимостей</div>
          </div>
        </button>
      </div>
    </div>

    <!-- Active Conversation Feed -->
    <div v-else class="max-w-3xl mx-auto space-y-4 pb-8">
      <ChatMessageItem
        v-for="(item, idx) in visibleHistoryItems"
        :key="idx"
        :item="item"
      />

      <!-- Streaming Active Thought -->
      <div v-if="agentStore.currentStreamThought" class="max-w-[95%]">
        <ThinkingBlock
          :content="agentStore.currentStreamThought"
          :is-streaming="true"
        />
      </div>

      <!-- Streaming Assistant Token Message with ldrs dot-stream -->
      <div
        v-if="agentStore.currentStreamAssistant"
        class="flex gap-3.5 items-start py-3"
      >
        <div class="shrink-0 mt-1 select-none">
          <img
            src="../../assets/logo.png"
            alt="Takiza"
            class="w-6.5 h-6.5 object-contain filter drop-shadow-[0_0_10px_rgba(245,158,11,0.3)] animate-pulse"
          />
        </div>
        <div class="flex-1 min-w-0 prose-takiza select-text">
          <div
            v-if="agentStore.currentTurnMoaModel && visibleHistoryItems.every(i => i.type !== 'MoaRouting')"
            class="inline-flex items-center gap-1.5 px-2.5 py-0.5 mb-2.5 rounded-lg bg-emerald-500/10 border border-emerald-500/25 text-[11px] font-mono text-emerald-300 select-none shadow-xs"
          >
            <Zap class="w-3 h-3 text-emerald-400 fill-current animate-pulse" />
            <span>Выбрана модель: <strong class="text-zinc-100 font-semibold">{{ agentStore.currentTurnMoaModel }}</strong></span>
          </div>

          <div v-html="streamingAssistantHtml" />
          <div v-if="agentStore.isStreaming && !agentStore.activeToolCall" class="inline-flex items-center gap-1.5 mt-2 py-0.5 px-2 rounded-md bg-white/[0.03] border border-white/[0.06]">
            <l-dot-stream size="20" speed="1.2" color="#f59e0b"></l-dot-stream>
            <span class="text-[10px] font-mono text-zinc-500">Генерация ответа...</span>
          </div>
        </div>
      </div>

      <!-- Streaming Active Tool Call -->
      <div v-if="agentStore.activeToolCall" class="max-w-[95%]">
        <ToolCallCard
          :name="agentStore.activeToolCall.name"
          :args="agentStore.activeToolCall.args"
          :logs="agentStore.streamingToolLogs"
          :is-running="true"
        />
      </div>

      <!-- Initial Model Loading / Thinking Indicator (ldrs quantum) -->
      <div
        v-if="agentStore.isStreaming && !agentStore.currentStreamAssistant && !agentStore.currentStreamThought && !agentStore.activeToolCall"
        class="flex items-center gap-3 py-4 px-2 animate-in fade-in duration-200"
      >
        <div class="shrink-0">
          <img
            src="../../assets/logo.png"
            alt="Takiza"
            class="w-6.5 h-6.5 object-contain filter drop-shadow-[0_0_8px_rgba(245,158,11,0.25)] animate-pulse"
          />
        </div>
        <div class="flex items-center gap-2.5 px-3.5 py-2 rounded-xl bg-white/[0.03] border border-white/[0.08] shadow-sm">
          <l-quantum size="18" speed="1.6" color="#f59e0b"></l-quantum>
          <span class="text-xs font-mono text-zinc-300">
            {{ agentStore.statusText || "Ожидание ответа модели..." }}
          </span>
        </div>
      </div>
    </div>

    <!-- Floating Scroll to Bottom Button -->
    <div
      v-if="showScrollBottom"
      class="fixed bottom-28 right-8 z-30 animate-in fade-in zoom-in-95 duration-150"
    >
      <button
        @click="scrollToBottom(true)"
        class="p-2 rounded-full bg-[#12141a] border border-white/20 text-zinc-300 hover:text-white shadow-2xl hover:bg-[#1a1d26] transition-all cursor-pointer flex items-center gap-1.5 text-xs"
      >
        <ArrowDown class="w-4 h-4" />
        <span class="pr-1 text-[11px] font-medium">Вниз</span>
      </button>
    </div>
  </div>
</template>
