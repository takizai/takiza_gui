<script setup lang="ts">
import { useI18n } from "../../i18n";
import BaseModal from "../common/BaseModal.vue";
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useAgentStore } from "../../stores/agentStore";
import type { CheckpointSummary } from "../../types";

const { tr } = useI18n();
const agent = useAgentStore();
const points = ref<CheckpointSummary[]>([]);
const selected = ref("");
const changes = ref<string[]>([]);
const error = ref("");
const busy = ref(false);
const previewed = ref(false);
let revision = 0;
watch(() => agent.restoreOpen, async open => {
  revision++; selected.value = ""; previewed.value = false; changes.value = []; error.value = "";
  if (!open) return;
  const current = revision;
  busy.value = true;
  try { const result = await invoke<CheckpointSummary[]>("list_checkpoints"); if (current === revision) points.value = result; }
  catch (cause) { if (current === revision) error.value = String(cause); }
  finally { if (current === revision) busy.value = false; }
});
async function preview(id: string) {
  const current = ++revision;
  selected.value = id; previewed.value = false; error.value = ""; busy.value = true;
  try { const result = await invoke<string[]>("preview_restore", { id }); if (current === revision) { changes.value = result; previewed.value = true; } }
  catch (cause) { if (current === revision) error.value = String(cause); }
  finally { if (current === revision) busy.value = false; }
}
async function restore() {
  busy.value = true; error.value = "";
  try { await agent.restoreCheckpoint(selected.value); }
  catch (cause) { error.value = String(cause); }
  finally { busy.value = false; }
}
</script>
<template>
  <BaseModal :is-open="agent.restoreOpen" :title='tr("Вернуться к точке")' size="lg" :dismissible="!busy" @close="agent.restoreOpen = false">
    <div class="modal-scroll p-5">

      <p class="text-xs text-zinc-400 mb-4">{{ tr("Восстановятся файлы и диалог. Последующие ответы исчезнут из чата. Текущее состояние сохранится в отдельной точке для отмены отката.") }}</p>
      <div class="space-y-2">
        <button v-for="point in points" :key="point.id" @click="preview(point.id)" :disabled="busy" class="w-full text-left rounded-xl border p-3 text-xs" :class="selected === point.id ? 'border-amber-500/50 bg-amber-500/10' : 'border-white/10 hover:bg-white/5'">
          <div class="text-zinc-500 mb-1">{{ point.created_at }}</div><div class="whitespace-pre-wrap break-words">{{ point.label }}</div>
        </button>
      </div>
      <p v-if="!busy && !points.length" class="text-sm text-zinc-500">{{ tr("В этом чате пока нет точек возврата.") }}</p>
      <pre v-if="previewed" class="my-4 max-h-44 overflow-auto whitespace-pre-wrap break-words text-xs text-zinc-400">{{ changes.join('\n') || tr("Файлы не изменились. Диалог будет восстановлен.") }}</pre>
      <p v-if="error" role="alert" class="my-3 text-xs text-rose-300 whitespace-pre-wrap">{{ error }}</p>
      <div class="flex justify-end gap-3 mt-4"><button :disabled="busy" @click="agent.restoreOpen = false" class="text-xs text-zinc-400">{{ tr("Отмена") }}</button><button @click="restore" :disabled="busy || !previewed" class="rounded-lg bg-amber-500 px-4 py-2 text-xs text-black disabled:opacity-40">{{ busy ? tr("Подождите…") : tr("Восстановить") }}</button></div>
    </div>
  </BaseModal>
</template>
