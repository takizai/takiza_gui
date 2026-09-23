<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useAgentStore } from "../../stores/agentStore";
import { ArrowUp, Square } from "lucide-vue-next";

const agentStore = useAgentStore();
const inputPrompt = ref("");
const textareaRef = ref<HTMLTextAreaElement | null>(null);

function autoResize() {
  if (!textareaRef.value) return;
  textareaRef.value.style.height = "auto";
  const newHeight = Math.min(textareaRef.value.scrollHeight, 180);
  textareaRef.value.style.height = `${newHeight}px`;
}

function handleKeyDown(e: KeyboardEvent) {
  if (e.key === "Enter" && !e.shiftKey) {
    e.preventDefault();
    submit();
  }
}

function submit() {
  const text = inputPrompt.value.trim();
  if (!text || agentStore.isStreaming) return;
  agentStore.sendPrompt(text);
  inputPrompt.value = "";
  if (textareaRef.value) {
    textareaRef.value.style.height = "auto";
  }
}

onMounted(() => {
  textareaRef.value?.focus();
});
</script>

<template>
  <div class="p-4 border-t border-neutral-800/80 bg-neutral-900/60 backdrop-blur-md shrink-0">
    <div class="max-w-4xl mx-auto relative">
      <div
        class="flex items-end gap-2 p-2 rounded-2xl border bg-neutral-900/90 shadow-xl transition-all"
        :class="
          agentStore.isStreaming
            ? 'border-blue-800/50 shadow-blue-500/5'
            : 'border-neutral-700/60 focus-within:border-blue-500/70 focus-within:ring-2 focus-within:ring-blue-500/20'
        "
      >
        <!-- Textarea -->
        <textarea
          ref="textareaRef"
          v-model="inputPrompt"
          @input="autoResize"
          @keydown="handleKeyDown"
          :disabled="agentStore.isStreaming"
          placeholder="Спросите Takiza о кодовой базе или задайте задачу..."
          rows="1"
          class="flex-1 bg-transparent border-0 resize-none text-neutral-100 placeholder-neutral-500 text-xs sm:text-sm px-3 py-1.5 focus:outline-none disabled:opacity-50 leading-relaxed max-h-44"
        />

        <!-- Actions -->
        <div class="flex items-center gap-1.5 shrink-0 pr-1 pb-1">
          <!-- Stop Button -->
          <button
            v-if="agentStore.isStreaming"
            @click="agentStore.stop"
            class="w-8 h-8 rounded-xl bg-rose-600 hover:bg-rose-500 text-white flex items-center justify-center transition-colors shadow-md shadow-rose-500/20 cursor-pointer"
            title="Остановить выполнение (Ctrl+C)"
          >
            <Square class="w-3.5 h-3.5 fill-current" />
          </button>

          <!-- Send Button -->
          <button
            v-else
            @click="submit"
            :disabled="!inputPrompt.trim()"
            class="w-8 h-8 rounded-xl bg-blue-600 hover:bg-blue-500 disabled:opacity-40 disabled:hover:bg-blue-600 text-white flex items-center justify-center transition-colors shadow-md shadow-blue-500/20 cursor-pointer disabled:cursor-not-allowed"
            title="Отправить (Enter)"
          >
            <ArrowUp class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- Hint row -->
      <div class="flex items-center justify-between mt-2 px-1 text-[11px] text-neutral-500 font-mono">
        <span>Enter для отправки • Shift+Enter для новой строки</span>
        <span>Takiza Autonomous Agent</span>
      </div>
    </div>
  </div>
</template>
