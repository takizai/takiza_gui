<script setup lang="ts">
import { computed } from "vue";
import { useAgentStore } from "../../stores/agentStore";
import {
  X,
  Zap,
  Gauge,
  Clock,
  Sparkles,
  TrendingDown,
  ShieldCheck,
  RefreshCw,
} from "lucide-vue-next";

const props = defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const agentStore = useAgentStore();

const isMoaActive = computed(() => {
  return (agentStore.config?.mode || agentStore.usageStats?.active_mode) === "moa";
});

const manualPercent = computed(() => agentStore.usageStats?.manual_percentage ?? 52);
const moaPercent = computed(() => agentStore.usageStats?.moa_percentage ?? 21);

async function toggleMode() {
  const newMode = isMoaActive.value ? "manual" : "moa";
  try {
    await agentStore.setMode(newMode);
  } catch (e) {
    console.error("Failed to toggle mode:", e);
  }
}

async function refreshStats() {
  try {
    await agentStore.loadUsage();
  } catch (e) {
    console.error("Failed to refresh usage:", e);
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
      class="w-full max-w-xl bg-[#0e1015] border border-white/10 rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh] animate-in zoom-in-95 duration-200 select-none"
    >
      <!-- Modal Header -->
      <div class="px-5 py-4 border-b border-white/[0.08] flex items-center justify-between bg-white/[0.02]">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-xl bg-amber-500/10 border border-amber-500/25 flex items-center justify-center text-amber-400">
            <Gauge class="w-4 h-4" />
          </div>
          <div>
            <h2 class="text-sm font-semibold text-zinc-100 flex items-center gap-2">
              <span>Лимиты и расход токенов</span>
              <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-white/[0.06] text-zinc-400 border border-white/[0.06]">
                Usage Quotas
              </span>
            </h2>
            <p class="text-[11px] text-zinc-400">Суточный расход токенов и статистика режимов работы</p>
          </div>
        </div>

        <div class="flex items-center gap-1.5">
          <button
            @click="refreshStats"
            class="p-1.5 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-white/[0.06] transition-colors cursor-pointer"
            title="Обновить данные"
          >
            <RefreshCw class="w-3.5 h-3.5" />
          </button>
          <button
            @click="emit('close')"
            class="p-1.5 rounded-lg text-zinc-400 hover:text-zinc-200 hover:bg-white/[0.06] transition-colors cursor-pointer"
          >
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- Modal Body -->
      <div class="p-5 space-y-4 overflow-y-auto">
        <!-- Mode Switcher Card -->
        <div
          class="p-3.5 rounded-xl border flex items-center justify-between gap-4 transition-all"
          :class="
            isMoaActive
              ? 'bg-emerald-500/5 border-emerald-500/30'
              : 'bg-amber-500/5 border-amber-500/30'
          "
        >
          <div class="flex items-center gap-3">
            <div
              class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0 border"
              :class="
                isMoaActive
                  ? 'bg-emerald-500/15 border-emerald-500/30 text-emerald-400'
                  : 'bg-amber-500/15 border-amber-500/30 text-amber-400'
              "
            >
              <Zap v-if="isMoaActive" class="w-4 h-4 fill-current" />
              <ShieldCheck v-else class="w-4 h-4" />
            </div>
            <div>
              <div class="flex items-center gap-2">
                <span class="text-xs font-semibold text-zinc-100">
                  {{ isMoaActive ? "Режим Takiza MoA" : "Режим Takiza Manual" }}
                </span>
                <span
                  class="text-[10px] font-mono px-1.5 py-0.2 rounded-full uppercase"
                  :class="
                    isMoaActive
                      ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/30'
                      : 'bg-amber-500/20 text-amber-300 border border-amber-500/30'
                  "
                >
                  Активен
                </span>
              </div>
              <p class="text-[11px] text-zinc-400 mt-0.5">
                {{
                  isMoaActive
                    ? "Умная маршрутизация через смесь экспертов (~45% экономии)"
                    : "Ручной выбор модели из каталога передовых frontier LLM"
                }}
              </p>
            </div>
          </div>

          <button
            @click="toggleMode"
            class="px-3 py-1.5 rounded-lg text-xs font-medium border transition-all shrink-0 cursor-pointer"
            :class="
              isMoaActive
                ? 'bg-white/[0.05] hover:bg-white/[0.1] border-white/10 text-zinc-200'
                : 'bg-emerald-500/15 hover:bg-emerald-500/25 border-emerald-500/30 text-emerald-300'
            "
          >
            {{ isMoaActive ? "Переключить на Manual" : "⚡ Включить MoA" }}
          </button>
        </div>

        <!-- Quota 1: Takiza Manual -->
        <div
          class="p-4 rounded-xl border transition-all"
          :class="
            !isMoaActive
              ? 'bg-[#12141a] border-amber-500/30 shadow-[0_0_15px_rgba(245,158,11,0.05)]'
              : 'bg-[#12141a]/60 border-white/[0.06]'
          "
        >
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-2">
              <span class="text-xs font-semibold text-zinc-200">Takiza Manual Quota</span>
              <span
                v-if="!isMoaActive"
                class="text-[10px] font-mono px-1.5 py-0.2 rounded bg-amber-500/15 text-amber-300 border border-amber-500/30"
              >
                [active]
              </span>
            </div>
            <div class="text-xs font-mono text-zinc-300">
              <span class="font-bold text-amber-400">520,000</span>
              <span class="text-zinc-500"> / 1,000,000 токенов</span>
            </div>
          </div>

          <!-- Progress Bar -->
          <div class="w-full h-2 rounded-full bg-white/[0.06] overflow-hidden p-0.5">
            <div
              class="h-full rounded-full bg-linear-to-r from-amber-500 to-amber-400 transition-all duration-500"
              :style="{ width: `${manualPercent}%` }"
            />
          </div>

          <div class="flex items-center justify-between text-[11px] text-zinc-400 mt-2 font-mono">
            <span>Лимит: 1.0M/день</span>
            <span>Остаток: 480k (48%)</span>
            <span class="text-amber-400/90 font-medium">52% использовано</span>
          </div>
        </div>

        <!-- Quota 2: Takiza MoA -->
        <div
          class="p-4 rounded-xl border transition-all"
          :class="
            isMoaActive
              ? 'bg-[#12141a] border-emerald-500/30 shadow-[0_0_15px_rgba(16,185,129,0.05)]'
              : 'bg-[#12141a]/60 border-white/[0.06]'
          "
        >
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-2">
              <span class="text-xs font-semibold text-zinc-200">Takiza MoA Quota</span>
              <span class="text-[10px] font-mono px-1.5 py-0.2 rounded bg-emerald-500/15 text-emerald-300 border border-emerald-500/30">
                ⚡ ~45% Экономия
              </span>
              <span
                v-if="isMoaActive"
                class="text-[10px] font-mono px-1.5 py-0.2 rounded bg-emerald-500/20 text-emerald-300 border border-emerald-500/30"
              >
                [active]
              </span>
            </div>
            <div class="text-xs font-mono text-zinc-300">
              <span class="font-bold text-emerald-400">210,000</span>
              <span class="text-zinc-500"> / 1,000,000 токенов</span>
            </div>
          </div>

          <!-- Progress Bar -->
          <div class="w-full h-2 rounded-full bg-white/[0.06] overflow-hidden p-0.5">
            <div
              class="h-full rounded-full bg-linear-to-r from-emerald-500 to-emerald-400 transition-all duration-500"
              :style="{ width: `${moaPercent}%` }"
            />
          </div>

          <div class="flex items-center justify-between text-[11px] text-zinc-400 mt-2 font-mono">
            <span>Лимит: 1.0M/день</span>
            <span>Остаток: 790k (79%)</span>
            <span class="text-emerald-400/90 font-medium">21% использовано</span>
          </div>

          <!-- Savings Badge -->
          <div class="mt-3 pt-2.5 border-t border-white/[0.06] flex items-center justify-between text-xs">
            <div class="flex items-center gap-1.5 text-emerald-400 font-medium">
              <TrendingDown class="w-3.5 h-3.5" />
              <span>Сэкономлено токенов через MoA:</span>
            </div>
            <span class="font-mono font-bold text-emerald-300 bg-emerald-500/10 px-2 py-0.5 rounded border border-emerald-500/20">
              ~172,000 токенов
            </span>
          </div>
        </div>

        <!-- Info Card -->
        <div class="p-3.5 rounded-xl bg-white/[0.02] border border-white/[0.06] text-[11px] text-zinc-400 space-y-1.5 leading-relaxed">
          <div class="flex items-center gap-1.5 text-zinc-200 font-medium">
            <Sparkles class="w-3.5 h-3.5 text-amber-400" />
            <span>Как работает Mixture of Agents (MoA):</span>
          </div>
          <p>
            MoA анализирует сложность, контекст и категорию задачи перед запросом к LLM. Рутинные задачи направляются в высокоэффективные модели, а критически сложная архитектура и отладка параллелизма — в глубокие reasoning флагманы.
          </p>
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="px-5 py-3 border-t border-white/[0.08] bg-white/[0.02] flex items-center justify-between text-xs text-zinc-400">
        <div class="flex items-center gap-1.5 font-mono text-[11px]">
          <Clock class="w-3.5 h-3.5 text-zinc-500" />
          <span>Квоты сбрасываются ежедневно в 00:00 UTC</span>
        </div>

        <button
          @click="emit('close')"
          class="px-4 py-1.5 rounded-xl bg-white/[0.08] hover:bg-white/[0.12] text-zinc-200 font-medium transition-colors cursor-pointer"
        >
          Закрыть
        </button>
      </div>
    </div>
  </div>
</template>
