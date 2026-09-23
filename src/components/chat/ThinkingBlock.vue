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
    class="my-2 rounded-lg border border-purple-900/30 bg-purple-950/15 overflow-hidden transition-all text-xs"
  >
    <!-- Header -->
    <button
      @click="toggle"
      class="w-full flex items-center justify-between px-3 py-2 text-purple-300 hover:text-purple-200 hover:bg-purple-900/20 transition-colors cursor-pointer select-none"
    >
      <div class="flex items-center gap-2">
        <Brain class="w-3.5 h-3.5 text-purple-400" :class="{ 'animate-pulse': isStreaming }" />
        <span class="font-medium">
          Рассуждения
          <span v-if="isStreaming" class="text-[10px] text-purple-400 font-mono animate-pulse">
            (думает...)
          </span>
        </span>
      </div>
      <component :is="isExpanded ? ChevronDown : ChevronRight" class="w-3.5 h-3.5 text-purple-400" />
    </button>

    <!-- Body -->
    <div
      v-show="isExpanded"
      class="px-3 pb-2.5 pt-1 text-neutral-300 font-mono text-[11px] leading-relaxed whitespace-pre-wrap border-t border-purple-900/20 bg-black/20"
    >
      {{ content }}
    </div>
  </div>
</template>
