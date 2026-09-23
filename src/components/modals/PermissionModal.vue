<script setup lang="ts">
import { useAgentStore } from "../../stores/agentStore";
import { ShieldAlert, Terminal, Check, CheckCheck, X } from "lucide-vue-next";

const agentStore = useAgentStore();
</script>

<template>
  <div
    v-if="agentStore.pendingPermission"
    class="fixed inset-0 bg-black/70 backdrop-blur-xs flex items-center justify-center p-4 z-50 animate-in fade-in duration-200"
  >
    <div
      class="w-full max-w-lg rounded-2xl bg-neutral-900 border border-neutral-700/80 shadow-2xl p-5 space-y-4"
    >
      <!-- Title -->
      <div class="flex items-center gap-3">
        <div
          class="w-10 h-10 rounded-xl bg-amber-950/60 border border-amber-800/60 flex items-center justify-center shrink-0"
        >
          <ShieldAlert class="w-5 h-5 text-amber-400" />
        </div>
        <div>
          <h3 class="text-sm font-semibold text-neutral-100">
            Запрос на выполнение команды
          </h3>
          <p class="text-xs text-neutral-400">
            Агент запрашивает запуск системной команды в терминале:
          </p>
        </div>
      </div>

      <!-- Command Block -->
      <div
        class="rounded-xl border border-neutral-800 bg-neutral-950 p-3 font-mono text-xs text-cyan-300 flex items-start gap-2.5 overflow-x-auto shadow-inner"
      >
        <Terminal class="w-4 h-4 text-neutral-500 shrink-0 mt-0.5" />
        <span class="whitespace-pre-wrap break-all">{{ agentStore.pendingPermission.command }}</span>
      </div>

      <div class="text-[11px] text-neutral-400 leading-relaxed">
        Команда будет выполнена в рабочей директории с вашими текущими правами доступа.
      </div>

      <!-- Buttons -->
      <div class="flex items-center justify-end gap-2 pt-2 border-t border-neutral-800">
        <button
          @click="agentStore.respondPermission('Deny')"
          class="px-3 py-1.5 rounded-lg border border-neutral-700 hover:bg-neutral-800 text-neutral-300 text-xs font-medium flex items-center gap-1.5 transition-colors cursor-pointer"
        >
          <X class="w-3.5 h-3.5 text-rose-400" />
          <span>Отклонить</span>
        </button>

        <button
          @click="agentStore.respondPermission('AllowOnce')"
          class="px-3.5 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white text-xs font-medium flex items-center gap-1.5 transition-colors shadow-sm cursor-pointer"
        >
          <Check class="w-3.5 h-3.5" />
          <span>Разрешить разово</span>
        </button>

        <button
          @click="agentStore.respondPermission('AllowAlways')"
          class="px-3.5 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-medium flex items-center gap-1.5 transition-colors shadow-sm cursor-pointer"
        >
          <CheckCheck class="w-3.5 h-3.5" />
          <span>Разрешать всегда</span>
        </button>
      </div>
    </div>
  </div>
</template>
