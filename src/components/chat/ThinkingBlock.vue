<script setup lang="ts">
import { ref } from "vue";
import { Brain, ChevronDown, ChevronRight } from "lucide-vue-next";

defineProps<{
  content: string;
  isStreaming?: boolean;
}>();

const isExpanded = ref(true);

function toggle() {
  isExpanded.value = !isExpanded.value;
}
</script>

<template>
  <div
    v-if="content"
    class="my-2 rounded-xl border border-white/[0.08] bg-[#101216]/70 overflow-hidden transition-all text-xs"
  >
    <!-- Header -->
    <button
      @click="toggle"
      class="w-full flex items-center justify-between px-3.5 py-2 text-zinc-400 hover:text-zinc-200 hover:bg-white/[0.03] transition-colors cursor-pointer select-none"
    >
      <div class="flex items-center gap-2">
        <Brain class="w-3.5 h-3.5 text-zinc-400" :class="{ 'animate-pulse text-zinc-200': isStreaming }" />
        <span class="font-medium text-zinc-300">
          Цепочка рассуждений
          <span v-if="isStreaming" class="text-[10px] text-zinc-400 font-mono animate-pulse ml-1">
            (размышляет...)
          </span>
        </span>
      </div>
      <component :is="isExpanded ? ChevronDown : ChevronRight" class="w-3.5 h-3.5 text-zinc-500" />
    </button>

    <!-- Body -->
    <div
      v-show="isExpanded"
      class="px-3.5 pb-3 pt-1 text-zinc-400 font-mono text-[11px] leading-relaxed whitespace-pre-wrap border-t border-white/[0.06] bg-black/30"
    >
      {{ content }}
    </div>
  </div>
</template>
