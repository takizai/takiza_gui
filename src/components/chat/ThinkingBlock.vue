<script setup lang="ts">
import { useI18n } from "../../i18n";
import { computed, ref, useId } from "vue";
import { Brain, ChevronRight, Loader2 } from "lucide-vue-next";

const { tr } = useI18n();
const props = defineProps<{ content: string; isStreaming?: boolean }>();
const isExpanded = ref(!!props.isStreaming);
const contentId = useId();
const preview = computed(() => props.content.trim().replace(/\s+/g, " ").slice(0, 160));
</script>

<template>
  <section v-if="content" class="chat-activity" :aria-label='tr("Рассуждения модели")'>
    <button @click="isExpanded = !isExpanded" :aria-expanded="isExpanded" :aria-controls="contentId" class="activity-row">
      <Loader2 v-if="isStreaming" class="activity-icon animate-spin" />
      <Brain v-else class="activity-icon" />
      <span class="shrink-0" :class="isStreaming ? 'text-zinc-300' : 'text-zinc-400'">{{ isStreaming ? tr("Размышляет") : tr("Рассуждения") }}</span>
      <span v-if="!isExpanded" class="min-w-0 flex-1 truncate text-zinc-500">{{ preview }}</span>
      <span v-else class="flex-1" />
      <ChevronRight class="w-3 h-3 shrink-0 text-zinc-500 transition-transform" :class="{ 'rotate-90': isExpanded }" />
    </button>
    <Transition name="activity">
      <div v-if="isExpanded" :id="contentId" class="activity-detail">
        <div class="reasoning-content max-h-52 overflow-y-auto overscroll-contain whitespace-pre-wrap break-words text-xs text-zinc-400 leading-6 select-text">{{ content }}</div>
      </div>
    </Transition>
  </section>
</template>
