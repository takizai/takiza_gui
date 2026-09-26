<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useAgentStore } from "../../stores/agentStore";
import { CURATED_MODELS } from "../../utils/models";
import {
  Check,
  Search,
  Zap,
  Sliders,
  Sparkles,
  Edit3,
  X,
} from "lucide-vue-next";

const props = withDefaults(
  defineProps<{
    isOpen: boolean;
    align?: "left" | "right";
    direction?: "up" | "down";
  }>(),
  {
    align: "left",
    direction: "up",
  }
);

const emit = defineEmits<{
  (e: "close"): void;
  (e: "open-settings"): void;
}>();

const agentStore = useAgentStore();
const searchQuery = ref("");
const isCustomInputOpen = ref(false);
const customModelInput = ref("");
const containerRef = ref<HTMLDivElement | null>(null);

const currentModel = computed(() => agentStore.config?.model || "");

// Extended models list including popular open-source/groq options
const allModels = computed(() => {
  const models = [...CURATED_MODELS];
  // Add common Groq & DeepSeek models if not present
  if (!models.some((m) => m.id === "llama-3.3-70b-versatile")) {
    models.push({
      provider: "Groq",
      id: "llama-3.3-70b-versatile",
      name: "Llama 3.3 70B",
      description: "Быстрая открытая модель с ультранизкой задержкой на Groq",
      is_expensive: false,
    });
  }
  if (!models.some((m) => m.id === "deepseek-chat")) {
    models.push({
      provider: "DeepSeek",
      id: "deepseek-chat",
      name: "DeepSeek V3",
      description: "Высокоинтеллектуальная модель общего назначения от DeepSeek",
      is_expensive: false,
    });
  }
  return models;
});

const filteredModels = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  if (!q) return allModels.value;
  return allModels.value.filter(
    (m) =>
      m.name.toLowerCase().includes(q) ||
      m.id.toLowerCase().includes(q) ||
      m.provider.toLowerCase().includes(q) ||
      m.description.toLowerCase().includes(q)
  );
});

// Group filtered models by provider
const groupedModels = computed(() => {
  const groups: Record<string, typeof allModels.value> = {};
  for (const m of filteredModels.value) {
    if (!groups[m.provider]) {
      groups[m.provider] = [];
    }
    groups[m.provider].push(m);
  }
  return groups;
});

async function selectModel(modelId: string) {
  try {
    await agentStore.savePreferences({ model: modelId });
    emit("close");
  } catch (err) {
    console.error("Failed to set model:", err);
  }
}

async function enableMoaMode() {
  try {
    await agentStore.setMode("moa");
    emit("close");
  } catch (err) {
    console.error("Failed to enable MoA:", err);
  }
}

async function submitCustomModel() {
  const val = customModelInput.value.trim();
  if (!val) return;
  await selectModel(val);
  customModelInput.value = "";
  isCustomInputOpen.value = false;
}

function handleClickOutside(event: MouseEvent) {
  if (
    containerRef.value &&
    !containerRef.value.contains(event.target as Node)
  ) {
    emit("close");
  }
}

function handleKeyDown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    emit("close");
  }
}

onMounted(() => {
  document.addEventListener("mousedown", handleClickOutside);
  document.addEventListener("keydown", handleKeyDown);
});

onUnmounted(() => {
  document.removeEventListener("mousedown", handleClickOutside);
  document.removeEventListener("keydown", handleKeyDown);
});
</script>

<template>
  <div
    v-if="isOpen"
    ref="containerRef"
    class="absolute z-50 w-80 bg-[#101218] border border-white/10 rounded-2xl shadow-2xl backdrop-blur-2xl flex flex-col overflow-hidden animate-in fade-in zoom-in-95 duration-150 select-none text-zinc-100"
    :class="[
      direction === 'up' ? 'bottom-full mb-2' : 'top-full mt-2',
      align === 'right' ? 'right-0' : 'left-0',
    ]"
    @click.stop
  >
    <!-- Header with Search -->
    <div class="p-2.5 border-b border-white/[0.08] bg-white/[0.02]">
      <div class="flex items-center justify-between mb-2 px-1">
        <span class="text-xs font-semibold text-zinc-200 flex items-center gap-1.5">
          <Sparkles class="w-3.5 h-3.5 text-amber-400" />
          <span>Выбор модели</span>
        </span>
        <button
          @click="emit('close')"
          class="p-1 rounded-md text-zinc-500 hover:text-zinc-200 hover:bg-white/[0.06] transition-colors cursor-pointer"
        >
          <X class="w-3 h-3" />
        </button>
      </div>

      <!-- Quick Search Bar -->
      <div class="relative flex items-center">
        <Search class="w-3.5 h-3.5 absolute left-2.5 text-zinc-500 pointer-events-none" />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Поиск по названию или ID..."
          class="w-full pl-8 pr-3 py-1.5 bg-[#090a0d] border border-white/[0.08] rounded-xl text-xs font-mono text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-amber-500/40"
          autofocus
        />
      </div>
    </div>

    <!-- Switch to MoA Mode Quick Banner -->
    <div class="p-2 border-b border-white/[0.06] bg-emerald-500/[0.04]">
      <button
        @click="enableMoaMode"
        class="w-full p-2 rounded-xl border border-emerald-500/25 bg-emerald-500/10 hover:bg-emerald-500/20 text-left transition-all cursor-pointer flex items-center justify-between gap-2 group"
      >
        <div class="flex items-center gap-2">
          <div class="w-6 h-6 rounded-lg bg-emerald-500/20 flex items-center justify-center text-emerald-400">
            <Zap class="w-3.5 h-3.5 fill-current animate-pulse" />
          </div>
          <div>
            <div class="text-xs font-semibold text-emerald-300 flex items-center gap-1.5">
              <span>Включить Takiza MoA</span>
              <span class="text-[9px] font-mono px-1 py-0.2 rounded bg-emerald-500/20 text-emerald-200">
                ⚡ ~45%
              </span>
            </div>
            <p class="text-[10px] text-zinc-400">Авто-роутинг моделей под сложность задачи</p>
          </div>
        </div>
      </button>
    </div>

    <!-- Models Scrollable List -->
    <div class="p-2 space-y-3 overflow-y-auto max-h-72">
      <div v-for="(models, provider) in groupedModels" :key="provider" class="space-y-1">
        <!-- Provider Category Title -->
        <div class="px-2 text-[10px] font-mono uppercase tracking-wider text-zinc-500 font-semibold">
          {{ provider }}
        </div>

        <!-- Model Items -->
        <div class="space-y-0.5">
          <button
            v-for="m in models"
            :key="m.id"
            @click="selectModel(m.id)"
            class="w-full text-left px-2.5 py-1.5 rounded-xl border transition-all cursor-pointer flex items-center justify-between gap-2 group"
            :class="
              currentModel === m.id
                ? 'bg-amber-500/15 border-amber-500/30 text-amber-200 shadow-xs'
                : 'border-transparent hover:bg-white/[0.05] text-zinc-300 hover:text-white'
            "
          >
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-1.5">
                <span class="text-xs font-medium truncate">{{ m.name }}</span>
                <span
                  v-if="m.is_expensive"
                  class="text-[9px] font-mono px-1 py-0.1 rounded bg-rose-500/20 text-rose-300 border border-rose-500/30"
                >
                  Exp
                </span>
              </div>
              <div class="text-[10px] font-mono text-zinc-500 truncate">
                {{ m.id }}
              </div>
            </div>

            <!-- Active Indicator -->
            <Check v-if="currentModel === m.id" class="w-3.5 h-3.5 text-amber-400 shrink-0" />
          </button>
        </div>
      </div>

      <div v-if="filteredModels.length === 0" class="text-center py-4 text-xs text-zinc-500 font-mono">
        Модели не найдены
      </div>
    </div>

    <!-- Custom Model Input Section -->
    <div v-if="isCustomInputOpen" class="p-2 border-t border-white/[0.08] bg-[#0c0e12] space-y-2">
      <div class="text-[11px] text-zinc-400">Введите Model ID:</div>
      <div class="flex items-center gap-1.5">
        <input
          v-model="customModelInput"
          @keydown.enter="submitCustomModel"
          placeholder="например: anthropic/claude-3-7-sonnet"
          class="flex-1 px-2 py-1.5 bg-[#090a0d] border border-white/[0.1] rounded-lg text-xs font-mono text-zinc-200 focus:outline-none focus:border-amber-400/50"
        />
        <button
          @click="submitCustomModel"
          class="px-2.5 py-1.5 rounded-lg bg-amber-500 text-zinc-950 font-medium text-xs hover:bg-amber-400 transition-colors cursor-pointer"
        >
          ОК
        </button>
      </div>
    </div>

    <!-- Context Menu Footer -->
    <div class="p-2 border-t border-white/[0.08] bg-white/[0.01] flex items-center justify-between text-xs text-zinc-400">
      <button
        @click="isCustomInputOpen = !isCustomInputOpen"
        class="flex items-center gap-1 px-2 py-1 rounded-lg hover:text-zinc-200 hover:bg-white/[0.06] transition-colors cursor-pointer text-[11px]"
      >
        <Edit3 class="w-3 h-3" />
        <span>{{ isCustomInputOpen ? "Скрыть ввод" : "Ввести ID..." }}</span>
      </button>

      <button
        @click="emit('open-settings'); emit('close');"
        class="flex items-center gap-1 px-2 py-1 rounded-lg hover:text-zinc-200 hover:bg-white/[0.06] transition-colors cursor-pointer text-[11px]"
      >
        <Sliders class="w-3 h-3" />
        <span>Все настройки...</span>
      </button>
    </div>
  </div>
</template>
