<script setup lang="ts">
import { useAgentStore } from "../../stores/agentStore";
import { ShieldAlert, Terminal, Check, CheckCheck, X } from "lucide-vue-next";

const agentStore = useAgentStore();
</script>

<template>
  <div
    v-if="agentStore.pendingPermission"
    class="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4 z-50 animate-in fade-in duration-150 select-none"
  >
    <div
      class="w-full max-w-lg rounded-2xl bg-[#111317] border border-white/10 shadow-2xl p-6 space-y-4"
    >
      <!-- Title -->
      <div class="flex items-center gap-3">
        <div
          class="w-10 h-10 rounded-xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center shrink-0"
        >
          <ShieldAlert class="w-5 h-5 text-amber-400" />
        </div>
        <div>
          <h3 class="text-sm font-semibold text-zinc-100">
            Запрос на выполнение команды
          </h3>
          <p class="text-xs text-zinc-400">
            Агент запрашивает запуск системной команды в терминале:
          </p>
        </div>
      </div>

      <!-- Command Block -->
      <div
        class="rounded-xl border border-white/[0.08] bg-black/60 p-3.5 font-mono text-xs text-zinc-200 flex items-start gap-2.5 overflow-x-auto shadow-inner"
      >
        <Terminal class="w-4 h-4 text-zinc-500 shrink-0 mt-0.5" />
        <span class="whitespace-pre-wrap break-all">{{ agentStore.pendingPermission.command }}</span>
      </div>

      <div class="text-[11px] text-zinc-500 leading-relaxed">
        Команда будет запущена в рабочей директории проекта с правами текущего пользователя.
      </div>

      <!-- Buttons -->
      <div class="flex items-center justify-end gap-2.5 pt-3 border-t border-white/[0.08]">
        <button
          @click="agentStore.respondPermission('Deny')"
          class="px-3.5 py-2 rounded-xl border border-white/10 hover:bg-white/[0.06] text-zinc-300 text-xs font-medium flex items-center gap-1.5 transition-colors cursor-pointer"
        >
          <X class="w-3.5 h-3.5 text-rose-400" />
          <span>Отклонить</span>
        </button>

        <button
          @click="agentStore.respondPermission('AllowOnce')"
          class="px-3.5 py-2 rounded-xl bg-white/[0.08] hover:bg-white/[0.15] border border-white/15 text-zinc-100 text-xs font-medium flex items-center gap-1.5 transition-all shadow-sm cursor-pointer active:scale-95"
        >
          <Check class="w-3.5 h-3.5" />
          <span>Разрешить разово</span>
        </button>

        <button
          @click="agentStore.respondPermission('AllowAlways')"
          class="px-3.5 py-2 rounded-xl bg-zinc-100 hover:bg-white text-zinc-950 text-xs font-medium flex items-center gap-1.5 transition-all shadow-md cursor-pointer active:scale-95"
        >
          <CheckCheck class="w-3.5 h-3.5 stroke-[2.5]" />
          <span>Разрешать всегда</span>
        </button>
      </div>
    </div>
  </div>
</template>
