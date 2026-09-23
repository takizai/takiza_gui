<script setup lang="ts">
import { ref, watch } from "vue";
import { useAgentStore } from "../../stores/agentStore";
import { X, Save, Eye, EyeOff, Sparkles, Check } from "lucide-vue-next";

const props = defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const agentStore = useAgentStore();

const provider = ref("custom");
const baseUrl = ref("");
const model = ref("");
const apiKey = ref("");
const proxy = ref("");
const autoApprove = ref(false);

const showApiKey = ref(false);
const saveSuccess = ref(false);
const isSaving = ref(false);

const presets = [
  {
    id: "openrouter",
    name: "OpenRouter",
    url: "https://openrouter.ai/api/v1",
    model: "anthropic/claude-3.5-sonnet",
  },
  {
    id: "openai",
    name: "OpenAI",
    url: "https://api.openai.com/v1",
    model: "gpt-4o",
  },
  {
    id: "deepseek",
    name: "DeepSeek",
    url: "https://api.deepseek.com",
    model: "deepseek-chat",
  },
  {
    id: "groq",
    name: "Groq",
    url: "https://api.groq.com/openai/v1",
    model: "llama-3.3-70b-versatile",
  },
  {
    id: "ollama",
    name: "Ollama (Local)",
    url: "http://localhost:11434/v1",
    model: "qwen2.5-coder:latest",
  },
];

function applyPreset(p: typeof presets[0]) {
  provider.value = p.id;
  baseUrl.value = p.url;
  model.value = p.model;
}

watch(
  () => props.isOpen,
  (open) => {
    if (open && agentStore.config) {
      baseUrl.value = agentStore.config.base_url;
      model.value = agentStore.config.model;
      apiKey.value = agentStore.config.api_key;
      proxy.value = agentStore.config.proxy || "";
      autoApprove.value = agentStore.config.auto_approve;

      const found = presets.find((p) => p.url === agentStore.config?.base_url);
      provider.value = found ? found.id : "custom";
      saveSuccess.value = false;
    }
  }
);

async function handleSave() {
  isSaving.value = true;
  saveSuccess.value = false;
  try {
    await agentStore.savePreferences({
      base_url: baseUrl.value.trim(),
      model: model.value.trim(),
      api_key: apiKey.value.trim(),
      proxy: proxy.value.trim() ? proxy.value.trim() : undefined,
      auto_approve: autoApprove.value,
    });
    saveSuccess.value = true;
    setTimeout(() => {
      saveSuccess.value = false;
      emit("close");
    }, 800);
  } catch (e) {
    console.error("Failed to save preferences:", e);
  } finally {
    isSaving.value = false;
  }
}
</script>

<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 bg-black/70 backdrop-blur-xs flex items-center justify-center p-4 z-50 animate-in fade-in duration-200"
  >
    <div
      class="w-full max-w-xl rounded-2xl bg-neutral-900 border border-neutral-700/80 shadow-2xl p-6 space-y-5"
    >
      <!-- Header -->
      <div class="flex items-center justify-between pb-3 border-b border-neutral-800">
        <div class="flex items-center gap-2.5">
          <div
            class="w-8 h-8 rounded-lg bg-blue-600/20 border border-blue-500/30 flex items-center justify-center text-blue-400"
          >
            <Sparkles class="w-4 h-4" />
          </div>
          <h3 class="text-sm font-semibold text-neutral-100">Настройки провайдера и моделей</h3>
        </div>
        <button
          @click="emit('close')"
          class="p-1.5 rounded-lg hover:bg-neutral-800 text-neutral-400 hover:text-white transition-colors cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Presets buttons -->
      <div>
        <label class="block text-xs font-medium text-neutral-400 mb-2">Быстрые пресеты провайдеров:</label>
        <div class="grid grid-cols-2 sm:grid-cols-3 gap-2">
          <button
            v-for="p in presets"
            :key="p.id"
            @click="applyPreset(p)"
            class="px-2.5 py-1.5 rounded-lg text-xs font-medium border text-left transition-colors cursor-pointer"
            :class="
              provider === p.id
                ? 'bg-blue-600/20 border-blue-500 text-blue-300'
                : 'bg-neutral-800/40 border-neutral-700/60 text-neutral-300 hover:bg-neutral-800/80'
            "
          >
            <div class="font-semibold">{{ p.name }}</div>
            <div class="text-[10px] text-neutral-400 truncate">{{ p.model.split('/')[1] || p.model }}</div>
          </button>
        </div>
      </div>

      <!-- Form Inputs -->
      <div class="space-y-3 text-xs">
        <div>
          <label class="block font-medium text-neutral-300 mb-1">Base URL (API Endpoint)</label>
          <input
            v-model="baseUrl"
            type="text"
            placeholder="https://openrouter.ai/api/v1"
            class="w-full px-3 py-2 rounded-lg bg-neutral-950 border border-neutral-700/80 text-neutral-100 font-mono focus:outline-none focus:border-blue-500"
          />
        </div>

        <div>
          <label class="block font-medium text-neutral-300 mb-1">Модель (Model ID)</label>
          <input
            v-model="model"
            type="text"
            placeholder="anthropic/claude-3.5-sonnet"
            class="w-full px-3 py-2 rounded-lg bg-neutral-950 border border-neutral-700/80 text-neutral-100 font-mono focus:outline-none focus:border-blue-500"
          />
        </div>

        <div>
          <label class="block font-medium text-neutral-300 mb-1">API Key</label>
          <div class="relative">
            <input
              v-model="apiKey"
              :type="showApiKey ? 'text' : 'password'"
              placeholder="sk-..."
              class="w-full pl-3 pr-10 py-2 rounded-lg bg-neutral-950 border border-neutral-700/80 text-neutral-100 font-mono focus:outline-none focus:border-blue-500"
            />
            <button
              type="button"
              @click="showApiKey = !showApiKey"
              class="absolute right-2 top-2 p-1 text-neutral-400 hover:text-neutral-200 cursor-pointer"
            >
              <EyeOff v-if="showApiKey" class="w-3.5 h-3.5" />
              <Eye v-else class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>

        <div>
          <label class="block font-medium text-neutral-300 mb-1">Proxy (опционально)</label>
          <input
            v-model="proxy"
            type="text"
            placeholder="http://127.0.0.1:7890"
            class="w-full px-3 py-2 rounded-lg bg-neutral-950 border border-neutral-700/80 text-neutral-100 font-mono focus:outline-none focus:border-blue-500"
          />
        </div>

        <div class="pt-2 flex items-center justify-between">
          <div>
            <div class="font-medium text-neutral-200">Авто-подтверждение команд (Auto-Approve)</div>
            <div class="text-[11px] text-neutral-400">
              Выполнять bash-команды без модального запроса подтверждения
            </div>
          </div>
          <input
            v-model="autoApprove"
            type="checkbox"
            class="w-4 h-4 rounded text-blue-600 focus:ring-blue-500 focus:ring-offset-neutral-900 cursor-pointer"
          />
        </div>
      </div>

      <!-- Footer Buttons -->
      <div class="flex items-center justify-end gap-2 pt-3 border-t border-neutral-800">
        <button
          @click="emit('close')"
          class="px-3.5 py-1.5 rounded-lg border border-neutral-700 hover:bg-neutral-800 text-neutral-300 text-xs font-medium transition-colors cursor-pointer"
        >
          Отмена
        </button>

        <button
          @click="handleSave"
          :disabled="isSaving"
          class="px-4 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white text-xs font-medium flex items-center gap-1.5 transition-colors shadow-sm cursor-pointer disabled:opacity-50"
        >
          <Check v-if="saveSuccess" class="w-3.5 h-3.5 text-emerald-300" />
          <Save v-else class="w-3.5 h-3.5" />
          <span>{{ saveSuccess ? "Сохранено!" : "Сохранить" }}</span>
        </button>
      </div>
    </div>
  </div>
</template>
