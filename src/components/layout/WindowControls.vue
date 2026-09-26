<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Minus, Square, Copy, X } from "lucide-vue-next";

const isMaximized = ref(false);

async function checkMaximized() {
  try {
    isMaximized.value = await invoke<boolean>("is_window_maximized");
  } catch (e) {
    // In browser preview fallback
  }
}

async function minimize() {
  try {
    await invoke("minimize_window");
  } catch (e) {
    console.error("Failed to minimize:", e);
  }
}

async function toggleMaximize() {
  try {
    isMaximized.value = await invoke<boolean>("toggle_maximize_window");
  } catch (e) {
    console.error("Failed to toggle maximize:", e);
  }
}

async function closeWindow() {
  try {
    await invoke("close_window");
  } catch (e) {
    console.error("Failed to close:", e);
  }
}

onMounted(() => {
  checkMaximized();
  window.addEventListener("resize", checkMaximized);
});
</script>

<template>
  <div class="flex items-center select-none" data-tauri-drag-region="false">
    <!-- Minimize -->
    <button
      @click="minimize"
      class="h-7 w-8 flex items-center justify-center text-zinc-400 hover:text-zinc-100 hover:bg-white/[0.08] transition-colors cursor-pointer rounded-md"
      title="Свернуть"
    >
      <Minus class="w-3.5 h-3.5" />
    </button>

    <!-- Maximize / Restore -->
    <button
      @click="toggleMaximize"
      class="h-7 w-8 flex items-center justify-center text-zinc-400 hover:text-zinc-100 hover:bg-white/[0.08] transition-colors cursor-pointer rounded-md"
      :title="isMaximized ? 'Восстановить' : 'Развернуть'"
    >
      <Copy v-if="isMaximized" class="w-3 h-3 rotate-180" />
      <Square v-else class="w-3 h-3" />
    </button>

    <!-- Close -->
    <button
      @click="closeWindow"
      class="h-7 w-8 flex items-center justify-center text-zinc-400 hover:text-white hover:bg-rose-600 transition-colors cursor-pointer rounded-md"
      title="Закрыть"
    >
      <X class="w-3.5 h-3.5" />
    </button>
  </div>
</template>
