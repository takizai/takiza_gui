<script setup lang="ts">
import { useI18n } from "../../i18n";
import BaseModal from "../common/BaseModal.vue";
import { computed } from "vue";
import { useAgentStore } from "../../stores/agentStore";
import { providerForUrl } from "../../utils/providers";

const { tr } = useI18n();
defineProps<{ isOpen: boolean }>();
const emit = defineEmits<{ (e: "close"): void }>();
const agent = useAgentStore();
const providerName = computed(() => {
  const config = agent.config;
  return config?.providers?.[config.active_provider || ""]?.name
    || providerForUrl(config?.base_url)?.name || tr("Пользовательский провайдер");
});
</script>

<template>
  <BaseModal :is-open="isOpen" :title='tr("Информация о провайдере")' size="sm" @close="emit('close')">
    <div class="modal-scroll p-5">
      <p class="text-sm leading-relaxed text-zinc-400">{{ tr("Статистика использования в GUI пока недоступна.") }}</p>
      <dl v-if="agent.config?.base_url" class="mt-5 space-y-4 text-xs">
        <div>
          <dt class="text-zinc-500 mb-1">{{ tr("Провайдер") }}</dt>
          <dd class="text-zinc-200 break-words select-text">{{ providerName }}</dd>
        </div>
        <div>
          <dt class="text-zinc-500 mb-1">{{ tr("Адрес API") }}</dt>
          <dd class="text-zinc-300 font-mono break-all select-text">{{ agent.config.base_url }}</dd>
        </div>
        <div v-if="agent.config.model">
          <dt class="text-zinc-500 mb-1">{{ tr("Модель") }}</dt>
          <dd class="text-zinc-300 font-mono break-all select-text">{{ agent.config.model }}</dd>
        </div>
      </dl>
      <p v-else class="mt-4 text-xs text-zinc-500">{{ tr("Сначала настройте подключение к провайдеру.") }}</p>
    </div>
  </BaseModal>
</template>
