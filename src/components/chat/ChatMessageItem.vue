<script setup lang="ts">
import { computed, ref } from "vue";
import type { HistoryItem } from "../../types";
import { renderMarkdown } from "../../utils/markdown";
import ThinkingBlock from "./ThinkingBlock.vue";
import ToolCallCard from "./ToolCallCard.vue";
import { User, AlertTriangle, Copy, Check, RotateCcw, Zap } from "lucide-vue-next";
import { useAgentStore } from "../../stores/agentStore";

const props = defineProps<{
  item: HistoryItem;
}>();

const agentStore = useAgentStore();
const copied = ref(false);

const renderedContent = computed(() => {
  if (props.item.type === "AssistantMessage") {
    return renderMarkdown(props.item.payload);
  }
  return "";
});

async function copyText(text: string) {
  try {
    await navigator.clipboard.writeText(text);
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 1500);
  } catch (err) {
    console.error("Failed to copy:", err);
  }
}

function rerunPrompt(promptText: string) {
  agentStore.sendPrompt(promptText);
}

const isVisible = computed(() => {
  if (!props.item || !props.item.type) return false;
  if (props.item.type === "UserPrompt") return !!props.item.payload && props.item.payload.trim().length > 0;
  if (props.item.type === "MoaRouting") return !!props.item.payload && !!props.item.payload.model;
  if (props.item.type === "Thought") return !!props.item.payload && props.item.payload.trim().length > 0;
  if (props.item.type === "ToolEnd") return !!props.item.payload;
  if (props.item.type === "AssistantMessage") return !!props.item.payload && props.item.payload.trim().length > 0;
  if (props.item.type === "Error") return !!props.item.payload;
  return false;
});
</script>

<template>
  <div v-if="isVisible" class="py-3 group">
    <!-- User Prompt -->
    <div
      v-if="item.type === 'UserPrompt'"
      class="flex gap-3 justify-end items-start"
    >
      <!-- Quick rerun button on hover -->
      <button
        @click="rerunPrompt(item.payload)"
        class="mt-2 p-1.5 rounded-lg opacity-0 group-hover:opacity-100 hover:bg-white/[0.08] text-zinc-500 hover:text-zinc-200 transition-all cursor-pointer"
        title="Повторить этот запрос"
      >
        <RotateCcw class="w-3.5 h-3.5" />
      </button>

      <div
        class="max-w-[85%] rounded-2xl rounded-tr-xs bg-[#14161d] border border-white/10 text-zinc-100 px-5 py-3 shadow-md text-xs sm:text-sm leading-relaxed whitespace-pre-wrap select-text"
      >
        {{ item.payload }}
      </div>

      <div
        class="w-7 h-7 rounded-full bg-white/[0.06] border border-white/10 flex items-center justify-center shrink-0 mt-0.5"
      >
        <User class="w-3.5 h-3.5 text-zinc-300" />
      </div>
    </div>

    <!-- MoA Routing Banner -->
    <div
      v-else-if="item.type === 'MoaRouting'"
      class="max-w-[95%] py-1 animate-in fade-in slide-in-from-top-1 duration-200"
    >
      <div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-xl bg-emerald-500/10 border border-emerald-500/25 text-xs text-emerald-300 shadow-sm backdrop-blur-sm select-none">
        <Zap class="w-3.5 h-3.5 text-emerald-400 fill-current" />
        <span class="font-medium text-zinc-300">Takiza MoA:</span>
        <span class="font-mono text-zinc-100 font-semibold">{{ item.payload.model }}</span>
        <span v-if="item.payload.source" class="text-[10px] font-mono px-1.5 py-0.2 rounded bg-emerald-500/20 text-emerald-200 border border-emerald-500/30">
          {{ item.payload.source }}
        </span>
        <span class="text-zinc-500">•</span>
        <span class="text-[11px] text-zinc-400">{{ item.payload.category }} ({{ item.payload.complexity }})</span>
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
      class="flex gap-3.5 items-start relative"
    >
      <div class="shrink-0 mt-1 select-none">
        <img
          src="../../assets/logo.png"
          alt="Takiza"
          class="w-7 h-7 object-contain filter drop-shadow-[0_0_10px_rgba(245,158,11,0.2)]"
        />
      </div>

      <div class="flex-1 min-w-0 pr-6">
        <!-- MoA routed model badge (displayed only in MoA mode / when routed) -->
        <div
          v-if="item.moaModel"
          class="inline-flex items-center gap-1.5 px-2.5 py-0.5 mb-2.5 rounded-lg bg-emerald-500/10 border border-emerald-500/25 text-[11px] font-mono text-emerald-300 select-none shadow-xs"
        >
          <Zap class="w-3 h-3 text-emerald-400 fill-current" />
          <span>Выбрана модель: <strong class="text-zinc-100 font-semibold">{{ item.moaModel }}</strong></span>
        </div>

        <div
          class="prose-takiza select-text"
          v-html="renderedContent"
        />

        <!-- Action bar -->
        <div class="flex items-center gap-2 mt-2 opacity-0 group-hover:opacity-100 transition-opacity">
          <button
            @click="copyText(item.payload)"
            class="flex items-center gap-1 px-2.5 py-1 rounded-md text-[11px] text-zinc-500 hover:text-zinc-200 hover:bg-white/[0.05] transition-colors cursor-pointer"
            title="Скопировать ответ"
          >
            <Check v-if="copied" class="w-3 h-3 text-emerald-400" />
            <Copy v-else class="w-3 h-3" />
            <span>{{ copied ? "Скопировано" : "Копировать ответ" }}</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Error Banner -->
    <div
      v-else-if="item.type === 'Error'"
      class="flex items-center gap-2 p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-300 text-xs font-mono select-text"
    >
      <AlertTriangle class="w-4 h-4 text-rose-400 shrink-0" />
      <span>{{ item.payload }}</span>
    </div>
  </div>
</template>
