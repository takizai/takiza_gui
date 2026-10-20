<script setup lang="ts">
import { useI18n } from "../../i18n";
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useAgentStore } from "../../stores/agentStore";
import { useQuestionnaire } from "../../composables/useQuestionnaire";
import QuestionnaireReview from "../input/QuestionnaireReview.vue";
import {
  Check,
  ArrowRight,
  ArrowLeft,
  X,
  Sparkles,
  Edit3,
} from "lucide-vue-next";

const { tr } = useI18n();

const agentStore = useAgentStore();

const payload = computed(() => agentStore.pendingQuestion);
const {
  currentIndex, customTextMap, customActiveMap,
  totalQuestions, currentQuestion, isMultiSelect, hasOptions,
  answers, isReviewing, editingFromReview, submitting,
  toggleOption, isOptionSelected, editQuestion,
  nextQuestion, prevQuestion, submitAnswers, skip,
} = useQuestionnaire(payload, (response) => agentStore.respondQuestion(response));
const customInputRef = ref<HTMLInputElement | null>(null);

function toggleCustomActive() {
  const qIdx = currentIndex.value;
  customActiveMap.value[qIdx] = !customActiveMap.value[qIdx];
  if (customActiveMap.value[qIdx]) {
    setTimeout(() => {
      customInputRef.value?.focus();
    }, 50);
  }
}

function handleKeyDown(e: KeyboardEvent) {
  if (!agentStore.pendingQuestion || e.repeat) return;

  const target = e.target as HTMLElement;
  const isInputFocused = target?.tagName === "INPUT" || target?.tagName === "TEXTAREA";

  if (e.key === "Escape") {
    skip();
    return;
  }

  if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
    e.preventDefault();
    submitAnswers();
    return;
  }

  if (isReviewing.value) return;

  // Quick single-digit selection (1..9) when not typing in text input
  if (!isInputFocused && hasOptions.value && /^[1-9]$/.test(e.key)) {
    const optIdx = parseInt(e.key, 10) - 1;
    const opts = currentQuestion.value?.options || [];
    if (optIdx < opts.length) {
      e.preventDefault();
      toggleOption(opts[optIdx]);
    }
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeyDown);
});
</script>

<template>
  <div v-if="agentStore.pendingQuestion && isReviewing" class="px-4 pb-2 shrink-0 z-20">
    <div class="max-w-3xl mx-auto w-full">
      <QuestionnaireReview :answers="answers" :submitting="submitting" @edit="editQuestion" @submit="submitAnswers" @cancel="skip" />
    </div>
  </div>
  <div
    v-else-if="agentStore.pendingQuestion && currentQuestion"
    class="px-4 pb-2 shrink-0 z-20 animate-in fade-in slide-in-from-bottom-2 duration-200 select-none"
  >
    <div class="max-w-3xl mx-auto w-full">
      <div
        class="rounded-2xl border border-amber-500/30 bg-[#111318]/95 backdrop-blur-2xl shadow-[0_10px_35px_rgba(0,0,0,0.6)] p-3.5 flex flex-col gap-3 transition-all"
      >
        <!-- Top Metadata & Navigation Bar -->
        <div class="flex items-center justify-between gap-2 border-b border-white/[0.06] pb-2">
          <div class="flex items-center gap-2 min-w-0">
            <!-- Icon Badge -->
            <div class="w-6 h-6 rounded-lg bg-amber-500/15 border border-amber-500/30 flex items-center justify-center shrink-0">
              <Sparkles class="w-3.5 h-3.5 text-amber-400" />
            </div>

            <!-- Title & Category -->
            <div class="flex items-center gap-2 min-w-0">
              <span class="text-xs font-semibold text-zinc-100 truncate">{{ tr("Вопрос от агента") }}</span>
              <span
                v-if="currentQuestion.header"
                class="text-[10px] font-mono px-2 py-0.5 rounded-md bg-amber-500/10 text-amber-300 border border-amber-500/25 truncate"
              >
                {{ currentQuestion.header }}
              </span>
            </div>

            <!-- Selection Mode Tag -->
            <span
              class="hidden sm:inline-flex text-[10px] font-mono px-1.5 py-0.5 rounded text-zinc-400 bg-white/[0.04] border border-white/[0.06]"
            >
              {{ isMultiSelect ? tr("Множественный выбор") : hasOptions ? tr("Один вариант") : tr("Свободный ответ") }}
            </span>
          </div>

          <!-- Stepper & Close Button -->
          <div class="flex items-center gap-2 shrink-0">
            <div
              v-if="totalQuestions > 1"
              class="flex items-center gap-1 px-2 py-0.5 rounded-lg bg-white/[0.05] border border-white/[0.08] text-[11px] font-mono text-zinc-300"
            >
              <span class="text-amber-400 font-semibold">{{ currentIndex + 1 }}</span>
              <span class="text-zinc-500">/</span>
              <span>{{ totalQuestions }}</span>
            </div>

            <button
              @click="skip"
              class="p-1 rounded-lg hover:bg-white/[0.08] text-zinc-400 hover:text-zinc-200 transition-colors cursor-pointer"
              :title='tr("Пропустить опросник (Esc)")'
            >
              <X class="w-4 h-4" />
            </button>
          </div>
        </div>

        <!-- Question Prompt Text -->
        <div class="px-0.5">
          <h3 class="text-xs sm:text-sm font-medium text-zinc-100 leading-snug">
            {{ currentQuestion.question }}
          </h3>
        </div>

        <!-- Options Container (Scrollable if many options) -->
        <div v-if="hasOptions" class="max-h-44 overflow-y-auto space-y-1.5 pr-1 custom-scrollbar">
          <button
            v-for="(opt, optIdx) in currentQuestion.options"
            :key="optIdx"
            type="button"
            @click="toggleOption(opt)"
            class="w-full text-left py-2 px-3 rounded-xl border transition-all cursor-pointer flex items-center justify-between gap-2.5 group active:scale-[0.99]"
            :class="
              isOptionSelected(opt)
                ? 'bg-amber-500/15 border-amber-500/40 text-amber-200 shadow-[0_0_12px_rgba(245,158,11,0.08)]'
                : 'bg-white/[0.03] border-white/[0.07] hover:bg-white/[0.06] hover:border-white/15 text-zinc-300'
            "
          >
            <div class="flex items-center gap-2.5 min-w-0">
              <!-- Checkbox / Radio Circle -->
              <div
                class="w-4 h-4 rounded-md flex items-center justify-center shrink-0 transition-colors border text-[10px]"
                :class="
                  isOptionSelected(opt)
                    ? 'bg-amber-500 border-amber-500 text-zinc-950 font-bold'
                    : 'border-zinc-600 bg-black/20 group-hover:border-zinc-500'
                "
              >
                <Check v-if="isOptionSelected(opt)" class="w-3 h-3 stroke-[3]" />
              </div>

              <!-- Option Number & Text -->
              <span class="text-[11px] font-mono text-zinc-500 group-hover:text-zinc-400 shrink-0">
                [{{ (optIdx + 1) % 10 }}]
              </span>
              <span class="text-xs truncate sm:text-wrap font-normal leading-tight">
                {{ opt }}
              </span>
            </div>
          </button>

          <!-- Custom Write-in Row (if allowed) -->
          <div v-if="currentQuestion.allow_custom !== false" class="pt-0.5">
            <button
              v-if="!customActiveMap[currentIndex]"
              type="button"
              @click="toggleCustomActive"
              class="w-full text-left py-1.5 px-3 rounded-xl border border-dashed border-white/10 hover:border-amber-500/30 hover:bg-amber-500/5 text-zinc-400 hover:text-amber-300 text-xs flex items-center gap-2 transition-all cursor-pointer"
            >
              <Edit3 class="w-3.5 h-3.5 text-zinc-500 group-hover:text-amber-400" />
              <span>
                {{ customTextMap[currentIndex]?.trim() ? tr("Свой ответ: {0}", {"0": customTextMap[currentIndex]}) : tr("Написать свой вариант ответа...") }}
              </span>
            </button>

            <!-- Inline Custom Input -->
            <div v-else class="flex items-center gap-2">
              <input
                ref="customInputRef"
                type="text"
                v-model="customTextMap[currentIndex]"
                :placeholder="currentQuestion.placeholder || tr('Напишите ваш ответ...')"
                class="flex-1 bg-white/[0.04] border border-amber-500/40 rounded-xl px-3 py-1.5 text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none focus:ring-1 focus:ring-amber-500/50"
                @keydown.enter.exact.prevent="nextQuestion"
              />
              <button
                type="button"
                @click="toggleCustomActive"
                class="px-2.5 py-1.5 text-xs rounded-xl bg-white/[0.05] hover:bg-white/[0.1] text-zinc-300 hover:text-white border border-white/10 transition-colors cursor-pointer"
              >{{ tr("Готово") }}</button>
            </div>
          </div>
        </div>

        <!-- Open-ended Answer Input (if no options predefined) -->
        <div v-else class="space-y-2">
          <input
            ref="customInputRef"
            type="text"
            v-model="customTextMap[currentIndex]"
            :placeholder="currentQuestion.placeholder || tr('Введите ваш ответ агенту...')"
            class="w-full bg-white/[0.04] border border-white/10 focus:border-amber-500/40 rounded-xl px-3 py-2 text-xs sm:text-sm text-zinc-100 placeholder-zinc-500 focus:outline-none focus:ring-1 focus:ring-amber-500/50"
            @keydown.enter.exact.prevent="nextQuestion"
          />
        </div>

        <!-- Bottom Actions Bar -->
        <div class="flex items-center justify-between pt-1 border-t border-white/[0.06] text-xs">
          <!-- Back button / shortcuts -->
          <div class="flex items-center gap-2 text-zinc-400">
            <button
              v-if="currentIndex > 0"
              @click="prevQuestion"
              class="flex items-center gap-1 px-2.5 py-1 rounded-lg bg-white/[0.04] hover:bg-white/[0.08] text-zinc-300 hover:text-white transition-colors cursor-pointer"
            >
              <ArrowLeft class="w-3.5 h-3.5" />
              <span>{{ tr("Назад") }}</span>
            </button>
            <span class="hidden sm:inline-flex text-[10px] text-zinc-500 font-mono">{{ tr("Ctrl+Enter ↵ проверить ответы") }}</span>
          </div>

          <!-- Next / Finish buttons -->
          <div class="flex items-center gap-2">
            <button
              @click="skip"
              class="px-2.5 py-1 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-white/[0.05] transition-colors cursor-pointer"
            >{{ tr("Пропустить") }}</button>

            <button
              @click="nextQuestion"
              class="flex items-center gap-1.5 px-3 py-1 rounded-lg bg-amber-500 hover:bg-amber-400 text-zinc-950 font-medium transition-all shadow-[0_0_15px_rgba(245,158,11,0.2)] active:scale-95 cursor-pointer"
            >
              <span>{{ editingFromReview ? tr("К списку ответов") : currentIndex < totalQuestions - 1 ? tr("Далее") : tr("Проверить ответы") }}</span>
              <ArrowRight class="w-3.5 h-3.5 stroke-[2.5]" />
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.02);
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 9999px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.25);
}
</style>
