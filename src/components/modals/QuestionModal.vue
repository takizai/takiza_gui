<script setup lang="ts">
import { useI18n } from "../../i18n";
import BaseModal from "../common/BaseModal.vue";
import { computed, onMounted, onUnmounted } from "vue";
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

function toggleCustomActive() {
  const qIdx = currentIndex.value;
  customActiveMap.value[qIdx] = !customActiveMap.value[qIdx];
}

function handleKeyDown(e: KeyboardEvent) {
  if (!agentStore.pendingQuestion || e.repeat) return;

  if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
    e.preventDefault();
    submitAnswers();
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
  <BaseModal :show-header="false" :is-open="!!agentStore.pendingQuestion" :title='tr("Уточнение деталей реализации")' size="lg" :dismissible="!submitting" @close="skip">
    <QuestionnaireReview v-if="isReviewing" :answers="answers" :submitting="submitting" @edit="editQuestion" @submit="submitAnswers" @cancel="skip" />
    <div v-else-if="currentQuestion" class="modal-scroll flex flex-col min-h-0">
      <!-- Header -->
      <div class="px-5 py-4 border-b border-white/[0.08] flex items-center justify-between bg-white/[0.02]">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl bg-amber-500/15 border border-amber-500/30 flex items-center justify-center shrink-0 shadow-inner">
            <Sparkles class="w-4 h-4 text-amber-400" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h3 class="text-sm font-semibold text-zinc-100">{{ tr("Уточнение деталей реализации") }}</h3>
              <span
                v-if="currentQuestion.header"
                class="text-[10px] font-mono px-2 py-0.5 rounded-full bg-amber-500/10 text-amber-300 border border-amber-500/25"
              >
                {{ currentQuestion.header }}
              </span>
            </div>
            <p class="text-xs text-zinc-400 mt-0.5">{{ tr("Агент запрашивает ваши предпочтения перед выполнением шагов") }}</p>
          </div>
        </div>

        <!-- Progress Counter -->
        <div v-if="totalQuestions > 1" class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-white/[0.04] border border-white/[0.08] text-xs font-mono text-zinc-300">
          <span class="text-amber-400 font-semibold">{{ currentIndex + 1 }}</span>
          <span class="text-zinc-500">/</span>
          <span>{{ totalQuestions }}</span>
        </div>
      </div>

      <!-- Question Body (Scrollable) -->
      <div class="flex-1 overflow-y-auto p-5 space-y-4">
        <!-- Question Title -->
        <div class="space-y-1">
          <div class="flex items-center justify-between">
            <span class="text-[11px] font-mono text-zinc-400">
              {{ isMultiSelect ? tr("Можно выбрать несколько вариантов:") : hasOptions ? tr("Выберите один вариант:") : tr("Напишите ваш ответ:") }}
            </span>
            <span v-if="isMultiSelect" class="text-[10px] font-mono text-amber-400/90">{{ tr("[Множественный выбор]") }}</span>
          </div>

          <h2 class="text-sm sm:text-base font-medium text-zinc-100 leading-snug">
            {{ currentQuestion.question }}
          </h2>
        </div>

        <!-- Predefined Options Cards -->
        <div v-if="hasOptions" class="space-y-2 pt-1">
          <button
            v-for="(opt, optIdx) in currentQuestion.options"
            :key="optIdx"
            type="button"
            @click="toggleOption(opt)"
            class="w-full text-left p-3 rounded-xl border transition-all cursor-pointer flex items-center justify-between gap-3 group active:scale-[0.99]"
            :class="
              isOptionSelected(opt)
                ? 'bg-amber-500/15 border-amber-500/50 shadow-[0_0_15px_rgba(245,158,11,0.12)] text-zinc-100'
                : 'bg-white/[0.03] border-white/[0.07] hover:bg-white/[0.06] hover:border-white/15 text-zinc-300'
            "
          >
            <div class="flex items-center gap-3 min-w-0">
              <!-- Checkbox / Radio Icon Indicator -->
              <div
                class="w-5 h-5 rounded-md flex items-center justify-center shrink-0 transition-colors border"
                :class="
                  isOptionSelected(opt)
                    ? 'bg-amber-400 border-amber-400 text-black font-bold'
                    : 'border-white/20 bg-black/30 text-transparent group-hover:border-white/40'
                "
              >
                <Check v-if="isOptionSelected(opt)" class="w-3.5 h-3.5 stroke-[3]" />
              </div>

              <span class="text-xs sm:text-sm font-normal leading-relaxed">
                {{ opt }}
              </span>
            </div>

            <span
              v-if="isOptionSelected(opt)"
              class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-amber-400/20 text-amber-300 shrink-0"
            >{{ tr("Выбрано") }}</span>
          </button>
        </div>

        <!-- Custom Answer / Write-in Section -->
        <div
          v-if="currentQuestion.allow_custom !== false"
          class="rounded-xl border border-white/[0.06] bg-black/40 p-3.5 space-y-2.5 transition-all"
        >
          <div
            @click="toggleCustomActive"
            class="flex items-center justify-between text-xs cursor-pointer select-none text-zinc-400 hover:text-zinc-200"
          >
            <div class="flex items-center gap-2">
              <Edit3 class="w-3.5 h-3.5 text-amber-400" />
              <span class="font-medium">
                {{ hasOptions ? tr("Свой вариант ответа или дополнение") : tr("Ваш ответ на вопрос") }}
              </span>
            </div>
            <span class="text-[10px] font-mono text-zinc-500">
              {{ (customTextMap[currentIndex] || customActiveMap[currentIndex] || !hasOptions) ? tr("Скрыть") : tr("Написать свой ответ") }}
            </span>
          </div>

          <!-- Text input or textarea -->
          <div
            v-if="customActiveMap[currentIndex] || !hasOptions || customTextMap[currentIndex]"
            class="pt-1 animate-in fade-in duration-150"
          >
            <textarea
              v-model="customTextMap[currentIndex]"
              :placeholder="currentQuestion.placeholder || (hasOptions ? tr('Напишите свой вариант или уточнение...') : tr('Введите ваш ответ...'))"
              rows="3"
              class="w-full bg-[#0a0b0e] border border-white/[0.08] focus:border-amber-500/40 rounded-xl p-3 text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none transition-colors resize-none leading-relaxed"
            />
          </div>
        </div>

        <!-- Tips hint -->
        <div class="text-[11px] text-zinc-500 flex items-center justify-between px-1">
          <span>{{ tr("Нажмите") }}<kbd class="px-1.5 py-0.5 rounded bg-white/[0.06] text-zinc-300 font-mono text-[10px]">Ctrl+Enter</kbd>{{ tr("для проверки ответов") }}</span>
          <span><kbd class="px-1.5 py-0.5 rounded bg-white/[0.06] text-zinc-300 font-mono text-[10px]">Esc</kbd>{{ tr("чтобы пропустить") }}</span>
        </div>
      </div>

      <!-- Footer Buttons -->
      <div class="px-5 py-3.5 border-t border-white/[0.08] bg-white/[0.02] flex items-center justify-between gap-3">
        <!-- Skip button -->
        <button
          @click="skip"
          class="px-3.5 py-2 rounded-xl border border-white/10 hover:bg-white/[0.06] text-zinc-400 hover:text-zinc-200 text-xs font-medium transition-colors cursor-pointer flex items-center gap-1.5"
          :title='tr("Пропустить опросник и использовать стандартные настройки агента")'
        >
          <X class="w-3.5 h-3.5 text-zinc-400" />
          <span>{{ tr("Пропустить") }}</span>
        </button>

        <!-- Navigation & Submit -->
        <div class="flex items-center gap-2">
          <!-- Back button (if multi-question) -->
          <button
            v-if="currentIndex > 0"
            @click="prevQuestion"
            class="px-3 py-2 rounded-xl bg-white/[0.05] hover:bg-white/[0.1] border border-white/10 text-zinc-300 text-xs font-medium transition-colors cursor-pointer flex items-center gap-1"
          >
            <ArrowLeft class="w-3.5 h-3.5" />
            <span>{{ tr("Назад") }}</span>
          </button>

          <!-- Next question button -->
          <button
            v-if="!editingFromReview && currentIndex < totalQuestions - 1"
            @click="nextQuestion"
            class="px-4 py-2 rounded-xl bg-zinc-200 hover:bg-white text-zinc-950 text-xs font-medium transition-all shadow-md cursor-pointer flex items-center gap-1.5 active:scale-95"
          >
            <span>{{ tr("Далее") }}</span>
            <ArrowRight class="w-3.5 h-3.5" />
          </button>

          <!-- Final Submit button -->
          <button
            v-else
            @click="nextQuestion"
            class="px-4 py-2 rounded-xl bg-amber-400 hover:bg-amber-300 text-zinc-950 text-xs font-semibold transition-all shadow-[0_0_20px_rgba(245,158,11,0.3)] cursor-pointer flex items-center gap-1.5 active:scale-95"
          >
            <ArrowRight class="w-3.5 h-3.5" />
            <span>{{ editingFromReview ? tr("К списку ответов") : tr("Проверить ответы") }}</span>
          </button>
        </div>
      </div>
    </div>
  </BaseModal>
</template>
