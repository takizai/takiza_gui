<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  diffText: string;
}>();

interface DiffLine {
  type: "add" | "del" | "ctx" | "meta";
  content: string;
  oldNum?: number;
  newNum?: number;
}

const parsedLines = computed<DiffLine[]>(() => {
  if (!props.diffText) return [];
  const lines = props.diffText.split("\n");
  const result: DiffLine[] = [];
  let oldLine = 1;
  let newLine = 1;

  for (const line of lines) {
    if (line.startsWith("@@")) {
      result.push({ type: "meta", content: line });
    } else if (line.startsWith("+") && !line.startsWith("+++")) {
      result.push({ type: "add", content: line.slice(1), newNum: newLine++ });
    } else if (line.startsWith("-") && !line.startsWith("---")) {
      result.push({ type: "del", content: line.slice(1), oldNum: oldLine++ });
    } else {
      result.push({
        type: "ctx",
        content: line.startsWith(" ") ? line.slice(1) : line,
        oldNum: oldLine++,
        newNum: newLine++,
      });
    }
  }
  return result;
});
</script>

<template>
  <div class="rounded-xl border border-white/[0.08] bg-black/60 font-mono text-[11px] overflow-x-auto my-2">
    <div
      v-for="(line, idx) in parsedLines"
      :key="idx"
      class="flex leading-5 px-2.5 select-text"
      :class="{
        'bg-emerald-500/10 text-emerald-300': line.type === 'add',
        'bg-rose-500/10 text-rose-300': line.type === 'del',
        'text-zinc-400 bg-white/[0.02] italic py-0.5 border-y border-white/[0.04]': line.type === 'meta',
        'text-zinc-400 hover:bg-white/[0.02]': line.type === 'ctx',
      }"
    >
      <!-- Line indicators -->
      <span
        v-if="line.type !== 'meta'"
        class="w-8 shrink-0 text-right pr-2 text-zinc-600 select-none text-[10px]"
      >
        {{ line.oldNum ?? "" }}
      </span>
      <span
        v-if="line.type !== 'meta'"
        class="w-8 shrink-0 text-right pr-2 text-zinc-600 select-none text-[10px]"
      >
        {{ line.newNum ?? "" }}
      </span>
      <span class="w-4 shrink-0 select-none text-center font-bold" :class="line.type === 'add' ? 'text-emerald-400' : line.type === 'del' ? 'text-rose-400' : 'text-zinc-600'">
        {{ line.type === 'add' ? '+' : line.type === 'del' ? '-' : ' ' }}
      </span>
      <span class="whitespace-pre flex-1">{{ line.content }}</span>
    </div>
  </div>
</template>
