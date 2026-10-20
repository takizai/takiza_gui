<script setup lang="ts">
import { useI18n } from "../../i18n";
import { computed } from "vue";
import type { QuestionAnswer, QuestionItem } from "../../types";
import { HelpCircle, Check, MessageSquareQuote, SkipForward, Edit3 } from "lucide-vue-next";

const { tr } = useI18n();

const props = defineProps<{
  questions?: QuestionItem[];
  answers: QuestionAnswer[];
  skipped: boolean;
  editable?: boolean;
}>();

const emit = defineEmits<{ edit: [index: number] }>();

const hasAnswers = computed(() => props.answers && props.answers.length > 0);
</script>

<template>
  <div class="rounded-xl border border-white/[0.06] bg-white/[0.02] p-4 space-y-3">
    <!-- Header -->
    <div class="flex items-center justify-between pb-2 border-b border-white/[0.06]">
      <div class="flex items-center gap-2">
        <div class="w-6 h-6 rounded-lg bg-amber-500/10 border border-amber-500/20 flex items-center justify-center shrink-0">
          <HelpCircle class="w-3.5 h-3.5 text-amber-400" />
        </div>
        <div>
          <span class="text-xs font-semibold text-zinc-200">{{ tr("Уточнения") }}</span>
        </div>
      </div>

      <div v-if="skipped" class="flex items-center gap-1 px-2 py-0.5 rounded-md bg-zinc-800/80 border border-white/[0.06] text-[10px] text-zinc-400 font-mono">
        <SkipForward class="w-3 h-3 text-zinc-400" />
        <span>{{ tr("Пропущено") }}</span>
      </div>
      <div v-else-if="!editable" class="flex items-center gap-1 px-2 py-0.5 rounded-md bg-emerald-500/10 border border-emerald-500/20 text-[10px] text-emerald-300 font-mono">
        <Check class="w-3 h-3 text-emerald-400" />
        <span>{{ tr("Ответ дан") }}</span>
      </div>
    </div>

    <!-- Skipped State -->
    <div v-if="skipped" class="text-xs text-zinc-400 italic py-1">{{ tr("Опрос завершён досрочно. Уже данные ответы сохранены.") }}</div>

    <!-- Answered List -->
    <div v-if="hasAnswers" class="space-y-3 pt-1">
      <div
        v-for="ans in answers"
        :key="ans.question_index"
        class="rounded-xl p-3 bg-white/[0.02] border border-white/[0.05] space-y-2 text-xs"
      >
        <!-- Question prompt -->
        <div class="font-medium text-zinc-200 flex items-start gap-2">
          <span class="text-amber-400 font-mono text-[11px] shrink-0 mt-0.5">Q{{ ans.question_index + 1 }}:</span>
          <span class="leading-relaxed flex-1 min-w-0 break-words">{{ ans.question }}</span>
          <button v-if="editable" type="button" @click="emit('edit', ans.question_index)"
            :aria-label="tr('Изменить ответ на вопрос {0}', {'0': ans.question_index + 1})"
            class="flex shrink-0 items-center gap-1 rounded-md px-2 py-1 text-[11px] text-amber-300 hover:bg-amber-500/10 focus-visible:outline-2 focus-visible:outline-amber-400">
            <Edit3 class="h-3 w-3" />{{ tr("Изменить") }}</button>
        </div>

        <!-- Selected Options -->
        <div v-if="ans.selected_options && ans.selected_options.length > 0" class="flex flex-wrap gap-1.5 pt-1 pl-5">
          <span
            v-for="(opt, oIdx) in ans.selected_options"
            :key="oIdx"
            class="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg bg-amber-500/15 border border-amber-500/30 text-amber-200 text-[11px] font-medium shadow-xs"
          >
            <Check class="w-3 h-3 text-amber-400 stroke-[2.5]" />
            <span>{{ opt }}</span>
          </span>
        </div>

        <p v-if="!ans.selected_options?.length && !ans.custom_text?.trim()" class="pl-5 text-[11px] text-zinc-500">{{ tr("Нет ответа") }}</p>

        <!-- Custom User Text / Notes -->
        <div
          v-if="ans.custom_text && ans.custom_text.trim()"
          class="flex items-start gap-2 pl-5 pt-1 text-[11px] text-zinc-300 font-mono bg-black/30 rounded-lg p-2 border border-white/[0.04]"
        >
          <MessageSquareQuote class="w-3.5 h-3.5 text-zinc-500 shrink-0 mt-0.5" />
          <span class="whitespace-pre-wrap min-w-0 break-words">{{ ans.custom_text.trim() }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
