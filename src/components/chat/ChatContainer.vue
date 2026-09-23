<script setup lang="ts">
import { ref, watch, nextTick, computed } from "vue";
import { useAgentStore } from "../../stores/agentStore";
import ChatMessageItem from "./ChatMessageItem.vue";
import ThinkingBlock from "./ThinkingBlock.vue";
import ToolCallCard from "./ToolCallCard.vue";
import { renderMarkdown } from "../../utils/markdown";
import { Sparkles, Terminal, FileCode, CheckCircle } from "lucide-vue-next";

const agentStore = useAgentStore();
const feedRef = ref<HTMLDivElement | null>(null);

const historyItems = computed(() => {
  return agentStore.currentSession?.history || [];
});

const streamingAssistantHtml = computed(() => {
  return renderMarkdown(agentStore.currentStreamAssistant);
});

function scrollToBottom() {
  nextTick(() => {
    if (feedRef.value) {
      feedRef.value.scrollTop = feedRef.value.scrollHeight;
    }
  });
}

// Watch for changes to auto-scroll
watch(
  () => [
    historyItems.value.length,
    agentStore.currentStreamAssistant,
    agentStore.currentStreamThought,
    agentStore.streamingToolLogs.length,
    agentStore.activeToolCall,
  ],
  () => {
    scrollToBottom();
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
    class="flex-1 overflow-y-auto px-4 md:px-8 py-4 space-y-2 select-text"
  >
    <!-- Empty State -->
    <div
      v-if="historyItems.length === 0 && !agentStore.isStreaming"
      class="h-full flex flex-col items-center justify-center text-center p-8 max-w-lg mx-auto"
    >
      <div
        class="w-14 h-14 rounded-2xl bg-gradient-to-tr from-cyan-600 to-blue-500 flex items-center justify-center shadow-xl shadow-cyan-500/20 mb-4"
      >
        <Sparkles class="w-7 h-7 text-white" />
      </div>
      <h2 class="text-lg font-bold text-neutral-100 mb-1">
        Takiza Code Assistant
      </h2>
      <p class="text-xs text-neutral-400 mb-6 leading-relaxed">
        Автономный AI-агент для разработки: читает и редактирует код, выполняет команды в терминале и анализирует проект.
      </p>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5 w-full text-left">
        <button
          @click="sendQuickPrompt('Проанализируй структуру проекта и расскажи, как он устроен')"
          class="p-3 rounded-xl border border-neutral-800 bg-neutral-900/60 hover:bg-neutral-800/80 hover:border-neutral-700 transition-colors text-xs text-neutral-300 flex items-start gap-2.5 cursor-pointer"
        >
          <FileCode class="w-4 h-4 text-cyan-400 shrink-0 mt-0.5" />
          <span>Обзор проекта</span>
        </button>

        <button
          @click="sendQuickPrompt('Покажи текущие изменения в git (git status и git diff)')"
          class="p-3 rounded-xl border border-neutral-800 bg-neutral-900/60 hover:bg-neutral-800/80 hover:border-neutral-700 transition-colors text-xs text-neutral-300 flex items-start gap-2.5 cursor-pointer"
        >
          <Terminal class="w-4 h-4 text-blue-400 shrink-0 mt-0.5" />
          <span>Проверить Git статус</span>
        </button>

        <button
          @click="sendQuickPrompt('Проверь проект командой cargo check / npm test и найди ошибки')"
          class="p-3 rounded-xl border border-neutral-800 bg-neutral-900/60 hover:bg-neutral-800/80 hover:border-neutral-700 transition-colors text-xs text-neutral-300 flex items-start gap-2.5 cursor-pointer"
        >
          <CheckCircle class="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
          <span>Запустить проверку кода</span>
        </button>

        <button
          @click="sendQuickPrompt('Какие файлы изменены в последних коммитах?')"
          class="p-3 rounded-xl border border-neutral-800 bg-neutral-900/60 hover:bg-neutral-800/80 hover:border-neutral-700 transition-colors text-xs text-neutral-300 flex items-start gap-2.5 cursor-pointer"
        >
          <Sparkles class="w-4 h-4 text-purple-400 shrink-0 mt-0.5" />
          <span>История изменений</span>
        </button>
      </div>
    </div>

    <!-- Message History Feed -->
    <div v-else class="max-w-4xl mx-auto space-y-1">
      <ChatMessageItem
        v-for="(item, idx) in historyItems"
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

      <!-- Streaming Active Tool Call -->
      <div v-if="agentStore.activeToolCall" class="max-w-[95%]">
        <ToolCallCard
          :name="agentStore.activeToolCall.name"
          :args="agentStore.activeToolCall.args"
          :logs="agentStore.streamingToolLogs"
          :is-running="true"
        />
      </div>

      <!-- Streaming Assistant Token Message -->
      <div
        v-if="agentStore.currentStreamAssistant"
        class="flex gap-3 items-start py-2.5"
      >
        <div
          class="w-7 h-7 rounded-lg bg-gradient-to-tr from-cyan-600 to-blue-500 flex items-center justify-center shrink-0 shadow-sm border border-cyan-400/20 mt-1"
        >
          <Sparkles class="w-3.5 h-3.5 text-white" />
        </div>
        <div
          class="flex-1 min-w-0 prose-takiza select-text"
          v-html="streamingAssistantHtml"
        />
      </div>
    </div>
  </div>
</template>
