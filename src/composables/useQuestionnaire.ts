import { computed, ref, watch, type Ref } from "vue";
import type { QuestionAnswer, QuestionRequestPayload, QuestionResponse } from "../types";

export function useQuestionnaire(
  payload: Ref<QuestionRequestPayload | null>,
  respond: (response: QuestionResponse) => void | Promise<void>,
) {
  const currentIndex = ref(0);
  const selectedOptionsMap = ref<Record<number, string[]>>({});
  const customTextMap = ref<Record<number, string>>({});
  const customActiveMap = ref<Record<number, boolean>>({});
  const isReviewing = ref(false);
  const editingFromReview = ref(false);
  const submitting = ref(false);
  const questions = computed(() => payload.value?.questions || []);
  const totalQuestions = computed(() => questions.value.length);
  const currentQuestion = computed(() => questions.value[currentIndex.value] || null);
  const isMultiSelect = computed(() => !!currentQuestion.value?.is_multi_select);
  const hasOptions = computed(() => (currentQuestion.value?.options?.length || 0) > 0);
  const answers = computed<QuestionAnswer[]>(() => questions.value.map((question, index) => ({
    question_index: index,
    question: question.question,
    selected_options: [...(selectedOptionsMap.value[index] || [])],
    custom_text: customTextMap.value[index]?.trim() || undefined,
  })));

  watch(() => payload.value?.id, () => {
    currentIndex.value = 0;
    selectedOptionsMap.value = {};
    customTextMap.value = {};
    customActiveMap.value = {};
    isReviewing.value = false;
    editingFromReview.value = false;
    submitting.value = false;
  }, { immediate: true });

  function toggleOption(option: string) {
    const index = currentIndex.value;
    const selected = [...(selectedOptionsMap.value[index] || [])];
    if (isMultiSelect.value) {
      const existing = selected.indexOf(option);
      if (existing >= 0) selected.splice(existing, 1);
      else selected.push(option);
      selectedOptionsMap.value[index] = selected;
    } else {
      selectedOptionsMap.value[index] = [option];
    }
  }

  function isOptionSelected(option: string) {
    return (selectedOptionsMap.value[currentIndex.value] || []).includes(option);
  }

  function showReview() {
    isReviewing.value = true;
  }

  function editQuestion(index: number) {
    if (index < 0 || index >= totalQuestions.value) return;
    currentIndex.value = index;
    editingFromReview.value = true;
    isReviewing.value = false;
  }

  function nextQuestion() {
    if (editingFromReview.value || currentIndex.value >= totalQuestions.value - 1) {
      showReview();
    } else {
      currentIndex.value++;
    }
  }

  function prevQuestion() {
    currentIndex.value = Math.max(0, currentIndex.value - 1);
  }

  async function submitAnswers() {
    // A shortcut from a question opens review; sending requires another action there.
    if (!isReviewing.value) {
      showReview();
      return;
    }
    if (submitting.value || !payload.value) return;
    submitting.value = true;
    try {
      await respond({ answers: answers.value, skipped: false });
    } finally {
      submitting.value = false;
    }
  }

  async function skip() {
    if (submitting.value || !payload.value) return;
    submitting.value = true;
    try { await respond({ answers: answers.value, skipped: true }); }
    finally { submitting.value = false; }
  }

  return {
    currentIndex, selectedOptionsMap, customTextMap, customActiveMap,
    questions, totalQuestions, currentQuestion, isMultiSelect, hasOptions,
    answers, isReviewing, editingFromReview, submitting,
    toggleOption, isOptionSelected, showReview, editQuestion,
    nextQuestion, prevQuestion, submitAnswers, skip,
  };
}
