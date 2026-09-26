<script setup lang="ts">
import { ref, watch } from "vue";
import { useAgentStore } from "../../stores/agentStore";
import { CURATED_MODELS, THEME_OPTIONS } from "../../utils/models";
import type { AppMode, ThemeName } from "../../types";
import {
  X,
  Save,
  Eye,
  EyeOff,
  Check,
  RotateCcw,
  Sliders,
  Key,
  FolderGit2,
  Zap,
  Palette,
  Sparkles,
  Bot,
  Brain,
  ShieldAlert,
  ShieldCheck,
  CheckCircle2,
} from "lucide-vue-next";

const props = defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const agentStore = useAgentStore();

const activeTab = ref<"models" | "mode" | "appearance" | "general" | "inherited">("models");

const provider = ref("custom");
const baseUrl = ref("");
const model = ref("");
const apiKey = ref("");
const proxy = ref("");
const autoApprove = ref(false);
const effort = ref("medium");
const currentMode = ref<AppMode>("manual");
const currentTheme = ref<ThemeName>("amber");

const showApiKey = ref(false);
const saveSuccess = ref(false);
const isSaving = ref(false);

const presets = [
  {
    id: "anymodel",
    name: "AnyModel (По умолчанию)",
    url: "https://anymodel.org/v1",
    model: "cc/claude-sonnet-5",
  },
  {
    id: "openai",
    name: "OpenAI",
    url: "https://api.openai.com/v1",
    model: "cx/gpt-6-astra",
  },
  {
    id: "anthropic",
    name: "Anthropic",
    url: "https://api.anthropic.com/v1",
    model: "cc/claude-sonnet-5",
  },
  {
    id: "google",
    name: "Google AI",
    url: "https://generativelanguage.googleapis.com/v1beta/openai",
    model: "ag/gemini-3.7-flash-high",
  },
  {
    id: "moonshot",
    name: "Moonshot AI",
    url: "https://api.moonshot.cn/v1",
    model: "kmc/k3",
  },
  {
    id: "groq",
    name: "Groq",
    url: "https://api.groq.com/openai/v1",
    model: "llama-3.3-70b-versatile",
  },
  {
    id: "openrouter",
    name: "OpenRouter",
    url: "https://openrouter.ai/api/v1",
    model: "anthropic/claude-3.5-sonnet",
  },
  {
    id: "ollama",
    name: "Ollama (Local)",
    url: "http://localhost:11434/v1",
    model: "qwen2.5-coder:latest",
  },
];

function selectCuratedModel(curatedId: string) {
  model.value = curatedId;
}

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
      effort.value = agentStore.config.effort || "medium";
      currentMode.value = (agentStore.config.mode as AppMode) || "manual";
      currentTheme.value = (agentStore.config.theme as ThemeName) || "amber";

      const found = presets.find((p) => p.url === agentStore.config?.base_url);
      provider.value = found ? found.id : "custom";
      saveSuccess.value = false;
    }
  }
);

async function selectTheme(themeId: string) {
  currentTheme.value = themeId as ThemeName;
  try {
    await agentStore.setTheme(themeId as ThemeName);
  } catch (e) {
    console.error("Failed to set theme:", e);
  }
}

async function selectMode(modeId: AppMode) {
  currentMode.value = modeId;
  try {
    await agentStore.setMode(modeId);
  } catch (e) {
    console.error("Failed to set mode:", e);
  }
}

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
      effort: effort.value,
      mode: currentMode.value,
      theme: currentTheme.value,
    });
    saveSuccess.value = true;
    setTimeout(() => {
      saveSuccess.value = false;
      emit("close");
    }, 600);
  } catch (e) {
    console.error("Failed to save preferences:", e);
  } finally {
    isSaving.value = false;
  }
}

async function handleResetToHarness() {
  isSaving.value = true;
  try {
    await agentStore.resetToHarnessDefaults();
    if (agentStore.config) {
      baseUrl.value = agentStore.config.base_url;
      model.value = agentStore.config.model;
      apiKey.value = agentStore.config.api_key;
      proxy.value = agentStore.config.proxy || "";
      autoApprove.value = agentStore.config.auto_approve;
      currentMode.value = (agentStore.config.mode as AppMode) || "manual";
      currentTheme.value = (agentStore.config.theme as ThemeName) || "amber";
      const found = presets.find((p) => p.url === agentStore.config?.base_url);
      provider.value = found ? found.id : "custom";
    }
    saveSuccess.value = true;
    setTimeout(() => {
      saveSuccess.value = false;
    }, 1200);
  } catch (e) {
    console.error("Failed to reset to harness defaults:", e);
  } finally {
    isSaving.value = false;
  }
}
</script>

<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-md animate-in fade-in duration-200"
    @click.self="emit('close')"
  >
    <div
      class="w-full max-w-2xl bg-[#0e1015] border border-white/10 rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh] animate-in zoom-in-95 duration-200 select-none"
    >
      <!-- Modal Header -->
      <div class="px-5 py-4 border-b border-white/[0.08] flex items-center justify-between bg-white/[0.02]">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-xl bg-amber-500/10 border border-amber-500/25 flex items-center justify-center text-amber-400">
            <Sliders class="w-4 h-4" />
          </div>
          <div>
            <h2 class="text-sm font-semibold text-zinc-100 flex items-center gap-2">
              <span>Настройки Takiza</span>
              <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-white/[0.06] text-zinc-400 border border-white/[0.06]">
                GUI & TUI Synced
              </span>
            </h2>
            <p class="text-[11px] text-zinc-400">Конфигурация синхронизирована с ~/.config/takiza/config.json</p>
          </div>
        </div>

        <button
          @click="emit('close')"
          class="p-1.5 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-white/[0.06] transition-colors cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Navigation Tabs -->
      <div class="flex items-center gap-1 px-5 border-b border-white/[0.06] bg-white/[0.01]">
        <button
          @click="activeTab = 'models'"
          class="px-3 py-2.5 text-xs font-medium border-b-2 transition-all cursor-pointer flex items-center gap-1.5"
          :class="
            activeTab === 'models'
              ? 'border-amber-400 text-amber-300'
              : 'border-transparent text-zinc-400 hover:text-zinc-200'
          "
        >
          <Bot class="w-3.5 h-3.5" />
          <span>Модели</span>
        </button>

        <button
          @click="activeTab = 'mode'"
          class="px-3 py-2.5 text-xs font-medium border-b-2 transition-all cursor-pointer flex items-center gap-1.5"
          :class="
            activeTab === 'mode'
              ? 'border-amber-400 text-amber-300'
              : 'border-transparent text-zinc-400 hover:text-zinc-200'
          "
        >
          <Zap class="w-3.5 h-3.5" />
          <span>Режим (MoA / Manual)</span>
        </button>

        <button
          @click="activeTab = 'appearance'"
          class="px-3 py-2.5 text-xs font-medium border-b-2 transition-all cursor-pointer flex items-center gap-1.5"
          :class="
            activeTab === 'appearance'
              ? 'border-amber-400 text-amber-300'
              : 'border-transparent text-zinc-400 hover:text-zinc-200'
          "
        >
          <Palette class="w-3.5 h-3.5" />
          <span>Темы</span>
        </button>

        <button
          @click="activeTab = 'general'"
          class="px-3 py-2.5 text-xs font-medium border-b-2 transition-all cursor-pointer flex items-center gap-1.5"
          :class="
            activeTab === 'general'
              ? 'border-amber-400 text-amber-300'
              : 'border-transparent text-zinc-400 hover:text-zinc-200'
          "
        >
          <Key class="w-3.5 h-3.5" />
          <span>API & Безопасность</span>
        </button>

        <button
          @click="activeTab = 'inherited'"
          class="px-3 py-2.5 text-xs font-medium border-b-2 transition-all cursor-pointer flex items-center gap-1.5"
          :class="
            activeTab === 'inherited'
              ? 'border-amber-400 text-amber-300'
              : 'border-transparent text-zinc-400 hover:text-zinc-200'
          "
        >
          <FolderGit2 class="w-3.5 h-3.5" />
          <span>Harness Env</span>
        </button>
      </div>

      <!-- Tab Content Area -->
      <div class="p-5 overflow-y-auto space-y-4 max-h-[62vh]">
        <!-- TAB 1: CURATED & CUSTOM MODELS -->
        <div v-if="activeTab === 'models'" class="space-y-4">
          <div>
            <div class="flex items-center justify-between mb-1.5">
              <label class="text-xs font-semibold text-zinc-200 flex items-center gap-1.5">
                <Sparkles class="w-3.5 h-3.5 text-amber-400" />
                <span>Каталог фронтирных моделей (Curated Frontier Catalog)</span>
              </label>
              <span class="text-[10px] text-zinc-500 font-mono">Из TUI</span>
            </div>
            <p class="text-[11px] text-zinc-400 mb-2.5">
              Выберите передовую модель для решения инженерных задач:
            </p>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
              <div
                v-for="m in CURATED_MODELS"
                :key="m.id"
                @click="selectCuratedModel(m.id)"
                class="p-2.5 rounded-xl border text-left transition-all cursor-pointer flex flex-col justify-between gap-1 group relative"
                :class="
                  model === m.id
                    ? 'bg-amber-500/10 border-amber-500/40 shadow-[0_0_12px_rgba(245,158,11,0.08)]'
                    : 'bg-white/[0.02] border-white/[0.06] hover:bg-white/[0.05] hover:border-white/[0.12]'
                "
              >
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-1.5">
                    <span class="text-xs font-semibold text-zinc-100 group-hover:text-amber-300 transition-colors">
                      {{ m.name }}
                    </span>
                    <span
                      v-if="m.is_expensive"
                      class="text-[9px] font-mono px-1 py-0.2 rounded bg-rose-500/20 text-rose-300 border border-rose-500/30"
                    >
                      Expensive
                    </span>
                  </div>
                  <span class="text-[10px] font-mono text-zinc-400">{{ m.provider }}</span>
                </div>
                <p class="text-[10px] text-zinc-400 line-clamp-2 leading-relaxed">
                  {{ m.description }}
                </p>
                <div class="flex items-center justify-between pt-1 border-t border-white/[0.04]">
                  <span class="text-[10px] font-mono text-zinc-400">{{ m.id }}</span>
                  <Check v-if="model === m.id" class="w-3.5 h-3.5 text-amber-400" />
                </div>
              </div>
            </div>
          </div>

          <!-- Active Model Input -->
          <div class="pt-2 border-t border-white/[0.06]">
            <label class="block text-xs font-medium text-zinc-300 mb-1.5">
              Идентификатор активной модели (Model ID):
            </label>
            <input
              v-model="model"
              placeholder="e.g. cx/gpt-6-astra или llama-3.3-70b-versatile"
              class="w-full px-3 py-2 bg-[#090a0c] border border-white/[0.1] rounded-xl text-xs font-mono text-zinc-200 placeholder-zinc-600 focus:outline-none focus:border-amber-400/50"
            />
          </div>

          <!-- Quick Presets -->
          <div>
            <label class="block text-xs font-medium text-zinc-400 mb-1.5">
              Быстрые пресеты провайдеров:
            </label>
            <div class="flex flex-wrap gap-1.5">
              <button
                v-for="p in presets"
                :key="p.id"
                @click="applyPreset(p)"
                class="px-2.5 py-1 rounded-lg text-xs font-mono border transition-all cursor-pointer"
                :class="
                  provider === p.id
                    ? 'bg-amber-500/15 border-amber-500/30 text-amber-300'
                    : 'bg-white/[0.02] border-white/[0.06] text-zinc-400 hover:text-zinc-200 hover:bg-white/[0.05]'
                "
              >
                {{ p.name }}
              </button>
            </div>
          </div>
        </div>

        <!-- TAB 2: MODE (MANUAL VS MOA) -->
        <div v-if="activeTab === 'mode'" class="space-y-4">
          <div class="text-xs text-zinc-300 leading-relaxed">
            Takiza поддерживает два режима выполнения задач, аналогично команде <code class="px-1.5 py-0.5 rounded bg-white/[0.06] text-amber-300 font-mono">/mode</code> в TUI:
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <!-- Manual Mode Card -->
            <div
              @click="selectMode('manual')"
              class="p-4 rounded-xl border text-left transition-all cursor-pointer flex flex-col justify-between gap-3 relative"
              :class="
                currentMode === 'manual'
                  ? 'bg-amber-500/10 border-amber-500/40 shadow-[0_0_15px_rgba(245,158,11,0.08)]'
                  : 'bg-white/[0.02] border-white/[0.06] hover:bg-white/[0.05] hover:border-white/[0.12]'
              "
            >
              <div>
                <div class="flex items-center justify-between mb-1.5">
                  <div class="flex items-center gap-2">
                    <ShieldCheck class="w-4 h-4 text-amber-400" />
                    <span class="text-xs font-bold text-zinc-100">Takiza Manual</span>
                  </div>
                  <CheckCircle2 v-if="currentMode === 'manual'" class="w-4 h-4 text-amber-400" />
                </div>
                <p class="text-[11px] text-zinc-400 leading-relaxed">
                  Ручной выбор конкретной модели из каталога frontier LLM. Вы всегда точно контролируете, какая модель обрабатывает каждый запрос.
                </p>
              </div>

              <div class="text-[10px] font-mono text-zinc-500 pt-2 border-t border-white/[0.06]">
                Базовый расход квоты • Прямой контроль
              </div>
            </div>

            <!-- MoA Mode Card -->
            <div
              @click="selectMode('moa')"
              class="p-4 rounded-xl border text-left transition-all cursor-pointer flex flex-col justify-between gap-3 relative"
              :class="
                currentMode === 'moa'
                  ? 'bg-emerald-500/10 border-emerald-500/40 shadow-[0_0_15px_rgba(16,185,129,0.08)]'
                  : 'bg-white/[0.02] border-white/[0.06] hover:bg-white/[0.05] hover:border-white/[0.12]'
              "
            >
              <div>
                <div class="flex items-center justify-between mb-1.5">
                  <div class="flex items-center gap-2">
                    <Zap class="w-4 h-4 text-emerald-400 fill-current" />
                    <span class="text-xs font-bold text-zinc-100">Takiza MoA</span>
                    <span class="text-[9px] font-mono px-1.5 py-0.2 rounded bg-emerald-500/20 text-emerald-300 border border-emerald-500/30">
                      ⚡ ~45% Cheaper
                    </span>
                  </div>
                  <CheckCircle2 v-if="currentMode === 'moa'" class="w-4 h-4 text-emerald-400" />
                </div>
                <p class="text-[11px] text-zinc-400 leading-relaxed">
                  Смесь экспертов (Mixture of Agents). Запрос пользователя классифицируется LoRA-маршрутизатором и направляется в оптимальную модель.
                </p>
              </div>

              <div class="text-[10px] font-mono text-emerald-400/90 pt-2 border-t border-white/[0.06]">
                Авто-роутинг • До ~45% экономии токенов
              </div>
            </div>
          </div>

          <!-- Deep MoA Explainer -->
          <div class="p-3.5 rounded-xl bg-white/[0.02] border border-white/[0.06] space-y-2 text-[11px] text-zinc-400 leading-relaxed">
            <div class="font-medium text-zinc-200 flex items-center gap-1.5">
              <Bot class="w-3.5 h-3.5 text-emerald-400" />
              <span>Таблица маршрутизации MoA:</span>
            </div>
            <ul class="space-y-1 list-disc list-inside text-zinc-400">
              <li><strong class="text-zinc-200">System Architecture & SIMD:</strong> GPT-6 Astra</li>
              <li><strong class="text-zinc-200">Concurrency & Async FFI:</strong> Claude Opus 5</li>
              <li><strong class="text-zinc-200">Dependency & Multimodal UI:</strong> Claude Sonnet 5</li>
              <li><strong class="text-zinc-200">Routine Backend & Tests:</strong> Gemini 3.7 Flash High</li>
            </ul>
          </div>
        </div>

        <!-- TAB 3: THEMES -->
        <div v-if="activeTab === 'appearance'" class="space-y-3">
          <div class="text-xs text-zinc-300 mb-1">
            Выберите визуальную тему оформления (синхронизируется с TUI командой <code class="px-1.5 py-0.5 rounded bg-white/[0.06] text-amber-300 font-mono">/theme</code>):
          </div>

          <div class="space-y-2">
            <div
              v-for="t in THEME_OPTIONS"
              :key="t.id"
              @click="selectTheme(t.id)"
              class="p-3 rounded-xl border flex items-center justify-between gap-3 transition-all cursor-pointer group"
              :class="
                currentTheme === t.id
                  ? 'bg-white/[0.06] border-white/30 shadow-lg'
                  : 'bg-white/[0.02] border-white/[0.06] hover:bg-white/[0.04]'
              "
            >
              <div class="flex items-center gap-3">
                <!-- Color Swatch Circle -->
                <div
                  class="w-8 h-8 rounded-xl flex items-center justify-center border shadow-xs"
                  :style="{ backgroundColor: `${t.color}22`, borderColor: `${t.color}66` }"
                >
                  <div class="w-3.5 h-3.5 rounded-full" :style="{ backgroundColor: t.color }" />
                </div>
                <div>
                  <div class="text-xs font-semibold text-zinc-100 flex items-center gap-2">
                    <span>{{ t.name }}</span>
                    <span
                      v-if="currentTheme === t.id"
                      class="text-[9px] font-mono px-1.5 py-0.2 rounded bg-white/[0.1] text-zinc-200"
                    >
                      Активна
                    </span>
                  </div>
                  <p class="text-[11px] text-zinc-400 mt-0.5">{{ t.desc }}</p>
                </div>
              </div>

              <Check v-if="currentTheme === t.id" class="w-4 h-4 text-zinc-200" />
            </div>
          </div>
        </div>

        <!-- TAB 4: API & GENERAL -->
        <div v-if="activeTab === 'general'" class="space-y-3.5">
          <div>
            <label class="block text-xs font-medium text-zinc-300 mb-1">
              API Endpoint URL:
            </label>
            <input
              v-model="baseUrl"
              placeholder="https://anymodel.org/v1"
              class="w-full px-3 py-2 bg-[#090a0c] border border-white/[0.1] rounded-xl text-xs font-mono text-zinc-200 placeholder-zinc-600 focus:outline-none focus:border-amber-400/50"
            />
          </div>

          <div>
            <div class="flex items-center justify-between mb-1">
              <label class="text-xs font-medium text-zinc-300">
                API Ключ:
              </label>
              <button
                @click="showApiKey = !showApiKey"
                class="text-[11px] text-zinc-400 hover:text-zinc-200 flex items-center gap-1 cursor-pointer"
              >
                <EyeOff v-if="showApiKey" class="w-3 h-3" />
                <Eye v-else class="w-3 h-3" />
                <span>{{ showApiKey ? "Скрыть" : "Показать" }}</span>
              </button>
            </div>
            <input
              v-model="apiKey"
              :type="showApiKey ? 'text' : 'password'"
              placeholder="gsk_... или sk-..."
              class="w-full px-3 py-2 bg-[#090a0c] border border-white/[0.1] rounded-xl text-xs font-mono text-zinc-200 placeholder-zinc-600 focus:outline-none focus:border-amber-400/50"
            />
          </div>

          <div>
            <label class="block text-xs font-medium text-zinc-300 mb-1">
              Прокси (HTTP/HTTPS/SOCKS5):
            </label>
            <input
              v-model="proxy"
              placeholder="socks5://127.0.0.1:1080 или http://..."
              class="w-full px-3 py-2 bg-[#090a0c] border border-white/[0.1] rounded-xl text-xs font-mono text-zinc-200 placeholder-zinc-600 focus:outline-none focus:border-amber-400/50"
            />
          </div>

          <!-- Reasoning Effort -->
          <div>
            <label class="block text-xs font-medium text-zinc-300 mb-1 flex items-center gap-1.5">
              <Brain class="w-3.5 h-3.5 text-amber-400" />
              <span>Reasoning Effort (Глубина рассуждений для reasoning моделей):</span>
            </label>
            <div class="flex items-center gap-2">
              <button
                v-for="e in ['low', 'medium', 'high']"
                :key="e"
                @click="effort = e"
                class="flex-1 py-1.5 rounded-lg border text-xs font-mono capitalize transition-all cursor-pointer"
                :class="
                  effort === e
                    ? 'bg-amber-500/20 border-amber-500/40 text-amber-300 font-semibold'
                    : 'bg-white/[0.02] border-white/[0.06] text-zinc-400 hover:text-zinc-200'
                "
              >
                {{ e }}
              </button>
            </div>
          </div>

          <!-- Auto Approve Toggle -->
          <div
            class="p-3 rounded-xl border border-white/[0.08] bg-white/[0.02] flex items-center justify-between cursor-pointer"
            @click="autoApprove = !autoApprove"
          >
            <div>
              <div class="text-xs font-medium text-zinc-200 flex items-center gap-1.5">
                <ShieldAlert v-if="autoApprove" class="w-3.5 h-3.5 text-amber-400" />
                <ShieldCheck v-else class="w-3.5 h-3.5 text-zinc-400" />
                <span>Режим Auto-Approve (без подтверждений команд)</span>
              </div>
              <p class="text-[11px] text-zinc-500 mt-0.5">
                {{ autoApprove ? "Команды выполняются автономно" : "Каждая bash команда запрашивает подтверждение" }}
              </p>
            </div>

            <div
              class="w-9 h-5 rounded-full transition-colors relative flex items-center p-0.5"
              :class="autoApprove ? 'bg-amber-500' : 'bg-white/10'"
            >
              <div
                class="w-4 h-4 rounded-full bg-white transition-transform"
                :class="autoApprove ? 'translate-x-4' : 'translate-x-0'"
              />
            </div>
          </div>
        </div>

        <!-- TAB 5: INHERITED HARNESS CONFIG -->
        <div v-if="activeTab === 'inherited'" class="space-y-3 font-mono text-[11px]">
          <div class="p-3 rounded-xl bg-white/[0.02] border border-white/[0.06] space-y-1.5 text-zinc-400">
            <div class="text-zinc-200 font-sans font-semibold text-xs mb-1">
              Синхронизированные пути конфигурации:
            </div>
            <div>• Файл настроек: ~/.config/takiza/config.json</div>
            <div>• Рабочий каталог: {{ agentStore.config?.workspace_dir }}</div>
            <div>• Модель по умолчанию: llama-3.3-70b-versatile</div>
            <div>• Эндпоинт по умолчанию: https://anymodel.org/v1</div>
            <div>• Активный режим: {{ agentStore.config?.mode || 'manual' }}</div>
            <div>• Активная тема: {{ agentStore.config?.theme || 'amber' }}</div>
          </div>

          <div class="pt-2">
            <button
              @click="handleResetToHarness"
              :disabled="isSaving"
              class="w-full py-2 rounded-xl bg-rose-500/10 hover:bg-rose-500/20 border border-rose-500/25 text-rose-300 text-xs font-sans font-medium flex items-center justify-center gap-1.5 transition-colors cursor-pointer disabled:opacity-50"
            >
              <RotateCcw class="w-3.5 h-3.5" />
              <span>Сбросить переопределения к настройкам Takiza Harness</span>
            </button>
          </div>
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="px-5 py-3 border-t border-white/[0.08] bg-white/[0.02] flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span v-if="saveSuccess" class="text-xs text-emerald-400 flex items-center gap-1">
            <Check class="w-3.5 h-3.5" />
            <span>Настройки успешно сохранены!</span>
          </span>
        </div>

        <div class="flex items-center gap-2">
          <button
            @click="emit('close')"
            class="px-3.5 py-1.5 rounded-xl text-xs font-medium text-zinc-400 hover:text-zinc-200 hover:bg-white/[0.05] transition-colors cursor-pointer"
          >
            Отмена
          </button>
          <button
            @click="handleSave"
            :disabled="isSaving"
            class="px-4 py-1.5 rounded-xl text-xs font-medium bg-amber-500 hover:bg-amber-400 text-zinc-950 flex items-center gap-1.5 transition-colors cursor-pointer disabled:opacity-50 shadow-md shadow-amber-950/30"
          >
            <Save class="w-3.5 h-3.5" />
            <span>{{ isSaving ? "Сохранение..." : "Сохранить" }}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
