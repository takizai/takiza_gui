<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useAgentStore } from "../../stores/agentStore";
import { ArrowUp, Square, ShieldAlert, Brain, Zap, ChevronDown } from "lucide-vue-next";
import { supportsReasoningEffort, REASONING_EFFORT_OPTIONS } from "../../utils/effort";
import ModelContextMenu from "../common/ModelContextMenu.vue";

const emit = defineEmits<{
  (e: "open-settings"): void;
}>();

const agentStore = useAgentStore();
const inputPrompt = ref("");
const textareaRef = ref<HTMLTextAreaElement | null>(null);
const isModelMenuOpen = ref(false);

const isMoa = computed(() => agentStore.config?.mode === "moa");

const currentModelShort = computed(() => {
  const m = agentStore.config?.model || "Takiza AI";
  return m.split("/").pop() || m;
});

const supportsEffort = computed(() => {
  return supportsReasoningEffort(agentStore.config?.model);
});

const currentEffort = computed(() => {
  return agentStore.config?.effort?.toLowerCase() || "medium";
});

async function onSelectEffort(effort: "low" | "medium" | "high") {
  try {
    await agentStore.setEffort(effort);
  } catch (err) {
    console.error("Failed to set effort:", err);
  }
}

function autoResize() {
  if (!textareaRef.value) return;
  textareaRef.value.style.height = "auto";
  const newHeight = Math.min(textareaRef.value.scrollHeight, 220);
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
  <div class="px-4 pb-5 pt-1 shrink-0 z-10">
    <div class="max-w-3xl mx-auto">
      <div
        class="rounded-2xl border bg-[#111318]/95 backdrop-blur-2xl shadow-2xl transition-all p-3"
        :class="
          agentStore.isStreaming
            ? 'border-amber-500/30 shadow-[0_0_30px_rgba(245,158,11,0.06)]'
            : 'border-white/10 focus-within:border-white/25 focus-within:shadow-[0_0_35px_rgba(0,0,0,0.8)]'
        "
      >
        <!-- Textarea Input -->
        <textarea
          ref="textareaRef"
          v-model="inputPrompt"
          @input="autoResize"
          @keydown="handleKeyDown"
          :disabled="agentStore.isStreaming"
          placeholder="Спросите Takiza или поставьте задачу для автономного решения..."
          rows="1"
          class="w-full bg-transparent border-0 resize-none text-zinc-100 placeholder-zinc-500 text-xs sm:text-sm px-2.5 py-1.5 focus:outline-none disabled:opacity-50 leading-relaxed max-h-52 font-normal"
        />

        <!-- Bottom Controls Bar -->
        <div class="flex items-center justify-between pt-2 border-t border-white/[0.06] mt-1.5 px-1 select-none">
          <!-- Left metadata chips -->
          <div class="flex items-center gap-2">
            <!-- In MoA Mode: Do NOT show model name, only show Takiza MoA indicator -->
            <div
              v-if="isMoa"
              @click="agentStore.setMode('manual')"
              class="flex items-center gap-1.5 px-2.5 py-0.5 rounded-lg bg-emerald-500/10 hover:bg-emerald-500/20 border border-emerald-500/25 text-[10px] text-emerald-300 font-mono select-none cursor-pointer transition-all"
              title="Takiza MoA активен: модель подбирается автоматически (~45% экономии токенов). Нажмите для перехода в Manual режим."
            >
              <Zap class="w-3 h-3 fill-current text-emerald-400 animate-pulse" />
              <span>Takiza MoA (Auto)</span>
            </div>

            <!-- In Manual Mode: Model selector with Context Menu Popover -->
            <div v-else class="relative">
              <button
                @click="isModelMenuOpen = !isModelMenuOpen"
                @contextmenu.prevent="isModelMenuOpen = true"
                class="flex items-center gap-1.5 px-2 py-0.5 rounded-lg bg-white/[0.04] hover:bg-white/[0.08] border border-white/[0.07] text-[11px] font-mono text-zinc-300 hover:text-white transition-colors cursor-pointer"
                :title="`Текущая модель: ${agentStore.config?.model}. Нажмите для выбора из меню`"
              >
                <span class="w-1.5 h-1.5 rounded-full bg-amber-400"></span>
                <span class="max-w-[130px] sm:max-w-[190px] truncate">{{ currentModelShort }}</span>
                <ChevronDown class="w-3 h-3 text-zinc-500" />
              </button>

              <!-- Context Menu Popover -->
              <ModelContextMenu
                :is-open="isModelMenuOpen"
                direction="up"
                align="left"
                @close="isModelMenuOpen = false"
                @open-settings="emit('open-settings')"
              />
            </div>

            <!-- Reasoning Effort Selector Pill (active when model supports effort) -->
            <div
              v-if="supportsEffort"
              class="flex items-center rounded-lg bg-white/[0.03] border border-amber-500/25 p-0.5 text-[10px] font-mono shadow-xs transition-all animate-in fade-in zoom-in-95 duration-200"
              title="Reasoning Effort: глубина рассуждений модели (low / med / high)"
            >
              <div class="px-1.5 text-amber-400/90 flex items-center gap-1 select-none">
                <Brain class="w-3 h-3 text-amber-400" />
                <span class="hidden md:inline font-medium">Effort:</span>
              </div>
              <button
                v-for="opt in REASONING_EFFORT_OPTIONS"
                :key="opt.value"
                @click="onSelectEffort(opt.value)"
                class="px-1.5 py-0.5 rounded transition-all cursor-pointer select-none"
                :class="
                  currentEffort === opt.value
                    ? 'bg-amber-500/25 text-amber-300 font-semibold shadow-xs'
                    : 'text-zinc-400 hover:text-zinc-200 hover:bg-white/[0.05]'
                "
                :title="opt.desc"
              >
                {{ opt.label }}
              </button>
            </div>

            <!-- Auto-Approve Warning Pill -->
            <div
              v-if="agentStore.config?.auto_approve"
              class="hidden sm:flex items-center gap-1 px-2 py-0.5 rounded-lg bg-amber-500/10 border border-amber-500/20 text-[10px] text-amber-300 font-mono"
            >
              <ShieldAlert class="w-3 h-3 text-amber-400" />
              <span>Auto-Approve</span>
            </div>
          </div>

          <!-- Right Action button & Hotkey -->
          <div class="flex items-center gap-3">
            <span class="hidden sm:inline text-[11px] text-zinc-500 font-mono">
              Enter ↵
            </span>

            <!-- Stop Button -->
            <button
              v-if="agentStore.isStreaming"
              @click="agentStore.stop"
              class="px-3 py-1.5 rounded-xl bg-rose-600/90 hover:bg-rose-500 text-white flex items-center gap-1.5 text-xs font-medium transition-all shadow-lg shadow-rose-950/50 cursor-pointer active:scale-95"
              title="Остановить генерацию"
            >
              <Square class="w-3 h-3 fill-current" />
              <span>Стоп</span>
            </button>

            <!-- Send Button -->
            <button
              v-else
              @click="submit"
              :disabled="!inputPrompt.trim()"
              class="w-8 h-8 rounded-xl bg-zinc-100 hover:bg-white disabled:opacity-30 disabled:hover:bg-zinc-100 text-zinc-950 flex items-center justify-center transition-all shadow-md cursor-pointer disabled:cursor-not-allowed active:scale-95"
              title="Отправить (Enter)"
            >
              <ArrowUp class="w-4 h-4 stroke-[2.5]" />
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
