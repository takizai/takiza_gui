<script setup lang="ts">
import { useI18n } from "../../i18n";
import { Send, X } from "lucide-vue-next";
import type { QuestionAnswer } from "../../types";
import QuestionHistoryCard from "../chat/QuestionHistoryCard.vue";

const { tr } = useI18n();

defineProps<{ answers: QuestionAnswer[]; submitting: boolean }>();
defineEmits<{ edit: [index: number]; submit: []; cancel: [] }>();
</script>

<template>
  <section class="w-full rounded-2xl border border-amber-500/30 bg-[#111318] p-4 shadow-xl" :aria-label='tr("Проверка ответов")'>
    <h2 class="text-sm font-semibold text-zinc-100">{{ tr("Проверьте ответы") }}</h2>
    <p class="mt-1 text-xs text-zinc-400">{{ tr("Можно изменить любой ответ. Агент получит ответы после подтверждения.") }}</p>
    <div class="mt-3 max-h-[50vh] overflow-y-auto overscroll-contain">
      <QuestionHistoryCard :answers="answers" :skipped="false" editable @edit="$emit('edit', $event)" />
    </div>
    <div class="mt-3 flex items-center justify-between gap-3">
      <button type="button" :disabled="submitting" @click="$emit('cancel')" class="flex items-center gap-1.5 rounded-lg px-3 py-2 text-xs text-zinc-400 hover:bg-white/5 focus-visible:outline-2 focus-visible:outline-amber-400 disabled:opacity-50">
        <X class="h-3.5 w-3.5" />{{ tr("Отменить") }}</button>
      <button type="button" :disabled="submitting" @click="$emit('submit')" class="flex items-center gap-1.5 rounded-lg bg-amber-500 px-3 py-2 text-xs font-semibold text-zinc-950 hover:bg-amber-400 focus-visible:outline-2 focus-visible:outline-white disabled:opacity-50">
        <Send class="h-3.5 w-3.5" /> {{ submitting ? tr("Отправка…") : tr("Подтвердить и отправить") }}
      </button>
    </div>
    <p class="mt-2 text-right text-[10px] text-zinc-500">{{ tr("Ctrl+Enter — отправить") }}</p>
  </section>
</template>
