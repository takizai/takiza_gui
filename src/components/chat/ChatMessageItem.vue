<script setup lang="ts">
import { computed } from "vue";
import type { HistoryItem } from "../../types";
import { renderMarkdown } from "../../utils/markdown";
import ThinkingBlock from "./ThinkingBlock.vue";
import ToolCallCard from "./ToolCallCard.vue";
import { User, Sparkles, AlertTriangle } from "lucide-vue-next";

const props = defineProps<{
  item: HistoryItem;
}>();

const renderedContent = computed(() => {
  if (props.item.type === "AssistantMessage") {
    return renderMarkdown(props.item.payload);
  }
  return "";
});
</script>

<template>
  <div class="py-2.5">
    <!-- User Prompt -->
    <div
      v-if="item.type === 'UserPrompt'"
      class="flex gap-3 justify-end items-start"
    >
      <div
        class="max-w-[85%] rounded-2xl rounded-tr-xs bg-blue-600/90 text-white px-4 py-2.5 shadow-sm text-xs leading-relaxed whitespace-pre-wrap select-text"
      >
        {{ item.payload }}
      </div>
      <div
        class="w-7 h-7 rounded-full bg-blue-700/80 flex items-center justify-center shrink-0 border border-blue-500/30"
      >
        <User class="w-3.5 h-3.5 text-white" />
      </div>
    </div>

    <!-- Thinking Block -->
    <div v-else-if="item.type === 'Thought'" class="max-w-[95%]">
      <ThinkingBlock :content="item.payload" />
    </div>

    <!-- Tool Call -->
    <div
      v-else-if="item.type === 'ToolEnd'"
      class="max-w-[95%]"
    >
      <ToolCallCard
        :name="item.payload.name"
        :args="item.payload.args"
        :result="item.payload.result"
        :is-error="item.payload.is_error"
      />
    </div>

    <!-- Assistant Message -->
    <div
      v-else-if="item.type === 'AssistantMessage'"
      class="flex gap-3 items-start"
    >
      <div
        class="w-7 h-7 rounded-lg bg-gradient-to-tr from-cyan-600 to-blue-500 flex items-center justify-center shrink-0 shadow-sm border border-cyan-400/20 mt-1"
      >
        <Sparkles class="w-3.5 h-3.5 text-white" />
      </div>
      <div
        class="flex-1 min-w-0 prose-takiza select-text"
        v-html="renderedContent"
      />
    </div>

    <!-- Error Banner -->
    <div
      v-else-if="item.type === 'Error'"
      class="flex items-center gap-2 p-3 rounded-lg bg-rose-950/40 border border-rose-900/60 text-rose-300 text-xs font-mono select-text"
    >
      <AlertTriangle class="w-4 h-4 text-rose-400 shrink-0" />
      <span>{{ item.payload }}</span>
    </div>
  </div>
</template>
