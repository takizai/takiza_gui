<script setup lang="ts">
import { useI18n } from "../../i18n";
import { ref, watch, nextTick, computed, onUnmounted } from "vue";
import { useAgentStore } from "../../stores/agentStore";
import ChatMessageItem from "./ChatMessageItem.vue";
import ThinkingBlock from "./ThinkingBlock.vue";
import ToolCallCard from "./ToolCallCard.vue";
import { useSmoothStream } from "../../composables/useSmoothStream";
import { renderMarkdown } from "../../utils/markdown";
import {
  ArrowDown,
  Loader2,
} from "lucide-vue-next";

const { tr } = useI18n();

const agentStore = useAgentStore();
const feedRef = ref<HTMLDivElement | null>(null);
const showScrollBottom = ref(false);

const visibleHistoryItems = computed(() => {
  const history = agentStore.currentSession?.history || [];
  return history.map((item, historyIndex) => ({ item, historyIndex })).filter(({ item }) => {
    if (!item || !item.type || item.type === "ResponseStats") return false;
    if (item.type === "ToolLog") return item.payload.includes("Stopped by user");
    if (item.type === "ToolStart") return false;
    if (item.type === "MoaRouting" && (!item.payload || !item.payload.model)) return false;
    if (item.type === "Thought" && (!item.payload || !item.payload.trim())) return false;
    if (item.type === "AssistantMessage" && (!item.payload || !item.payload.trim())) return false;
    if (item.type === "UserPrompt" && (!item.payload || !item.payload.trim())) return false;
    return true;
  });
});

// Coalesce tokens before parsing and highlighting the growing Markdown document.
const displayedAssistant = useSmoothStream(computed(() => agentStore.currentStreamAssistant), computed(() => agentStore.isStreaming));
const streamingAssistantHtml = ref("");
let renderTimer: ReturnType<typeof setTimeout> | undefined;
let scrollFrame = 0;
function renderStream() {
  renderTimer = undefined;
  streamingAssistantHtml.value = renderMarkdown(displayedAssistant.value);
}
watch(displayedAssistant, content => {
  if (!content) {
    clearTimeout(renderTimer);
    renderStream();
  } else if (!renderTimer) renderTimer = setTimeout(renderStream, 32);
}, { immediate: true });
onUnmounted(() => { clearTimeout(renderTimer); cancelAnimationFrame(scrollFrame); });

function handleScroll() {
  if (!feedRef.value) return;
  const { scrollTop, scrollHeight, clientHeight } = feedRef.value;
  showScrollBottom.value = scrollHeight - (scrollTop + clientHeight) > 120;
}

function scrollToBottom(smooth = false) {
  cancelAnimationFrame(scrollFrame);
  nextTick(() => {
    scrollFrame = requestAnimationFrame(() => {
      feedRef.value?.scrollTo({ top: feedRef.value.scrollHeight, behavior: smooth ? "smooth" : "auto" });
    });
  });
}

watch(
  () => [
    agentStore.currentSession?.id,
    visibleHistoryItems.value.length,
    streamingAssistantHtml.value,
    agentStore.currentStreamThought,
    agentStore.streamingToolLogs.length,
    agentStore.activeToolCall?.name,
  ],
  () => {
    if (!showScrollBottom.value) {
      scrollToBottom();
    }
  },
  { flush: "post" }
);

watch(() => agentStore.currentSession?.id, () => { showScrollBottom.value = false; scrollToBottom(); });

</script>

<template>
  <div ref="feedRef" @scroll.passive="handleScroll" class="chat-feed flex-1 min-h-0 overflow-y-auto px-5 sm:px-8 py-6 select-text relative" :aria-label='tr("Диалог")'>
    <div class="max-w-3xl mx-auto pb-6">
      <ChatMessageItem v-for="entry in visibleHistoryItems" :key="`${agentStore.currentSession?.id}:${entry.historyIndex}`" :item="entry.item" :history-index="entry.historyIndex" class="history-message" />
      <ThinkingBlock v-if="agentStore.currentStreamThought" :content="agentStore.currentStreamThought" :is-streaming="agentStore.isStreaming && !agentStore.currentStreamAssistant && !agentStore.activeToolCall" />
      <div v-if="agentStore.currentStreamAssistant" class="chat-assistant">
        <div class="chat-author">Takiza<span class="text-[10px] font-normal text-zinc-500">{{ tr("Пишет ответ") }}</span></div>
        <div class="prose-takiza select-text" v-html="streamingAssistantHtml" />
      </div>
      <ToolCallCard v-if="agentStore.activeToolCall" :name="agentStore.activeToolCall.name" :args="agentStore.activeToolCall.args" :logs="agentStore.streamingToolLogs" :is-running="true" />
      <div v-if="agentStore.isStreaming && !agentStore.currentStreamAssistant && !agentStore.currentStreamThought && !agentStore.activeToolCall" role="status" class="flex items-center gap-2 py-4 text-xs text-zinc-500">
        <Loader2 class="w-3.5 h-3.5 animate-spin" />
        <span>{{ agentStore.pendingPermission ? tr("Ожидаю подтверждение") : agentStore.pendingQuestion ? tr("Нужен ваш ответ") : tr("Ожидаю ответ модели…") }}</span>
      </div>
    </div>
    <div v-if="showScrollBottom" class="sticky bottom-0 flex justify-center pointer-events-none">
      <button @click="scrollToBottom(true)" class="pointer-events-auto flex items-center gap-1.5 rounded-full bg-[#18191c] border border-white/10 px-3 py-1.5 text-[11px] text-zinc-400 hover:text-zinc-200 cursor-pointer" :aria-label='tr("К последнему сообщению")'><ArrowDown class="w-3 h-3" />{{ tr("Вниз") }}</button>
    </div>
  </div>
</template>
