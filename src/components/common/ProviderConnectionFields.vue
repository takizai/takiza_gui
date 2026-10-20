<script setup lang="ts">
import { ref, computed } from 'vue';
import { Eye, EyeOff, Plus, X } from 'lucide-vue-next';
import { useI18n } from '../../i18n';
import type { ApiType, CredentialSource, HeaderEntry } from '../../types';
import BaseSelect from './BaseSelect.vue';

const props = withDefaults(defineProps<{
  baseUrl: string; apiKey: string; apiType: ApiType; credentialSource: CredentialSource;
  keyEnv: string; headers: HeaderEntry[]; disabled?: boolean;
}>(), { disabled: false });
const emit = defineEmits<{
  (e: 'update:baseUrl', value: string): void;
  (e: 'update:apiKey', value: string): void;
  (e: 'update:apiType', value: ApiType): void;
  (e: 'update:credentialSource', value: CredentialSource): void;
  (e: 'update:keyEnv', value: string): void;
  (e: 'update:headers', value: HeaderEntry[]): void;
}>();
const { tr } = useI18n();
const reveal = ref(false);
const showHeaders = ref(false);
const protocols = [
  { value: 'chat_completions', label: 'OpenAI Chat Completions', description: '/chat/completions' },
  { value: 'openai_responses', label: 'OpenAI Responses', description: '/responses' },
  { value: 'anthropic_messages', label: 'Anthropic Messages', description: '/messages' },
];
const sources = computed(() => [
  { value: 'key', label: tr('Ключ или токен') }, { value: 'env', label: tr('Переменная окружения') }, { value: 'none', label: tr('Без API-ключа') },
]);
const hasHeaders = computed(() => showHeaders.value);
function updateHeader(index: number, key: keyof HeaderEntry, value: string) {
  emit('update:headers', props.headers.map((header, position) => position === index ? { ...header, [key]: value } : header));
}
function addHeader() {
  showHeaders.value = true;
  emit('update:headers', [...props.headers, { name: '', value: '' }]);
}
</script>

<template>
  <div class="space-y-3">
    <div>
      <label for="provider-api-url" class="block text-xs text-zinc-400 mb-1.5">{{ tr('Адрес API') }}</label>
      <input id="provider-api-url" :value="baseUrl" @input="emit('update:baseUrl', ($event.target as HTMLInputElement).value)"
        :disabled="disabled" type="url" spellcheck="false" placeholder="https://api.example.com/v1"
        class="w-full rounded-lg border border-white/[0.08] bg-black/20 px-3 py-2 text-xs font-mono text-zinc-200" />
    </div>
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div><label for="provider-api-type" class="block text-xs text-zinc-400 mb-1.5">{{ tr('Протокол API') }}</label>
        <BaseSelect id="provider-api-type" :model-value="apiType" :options="protocols" :disabled="disabled" class="w-full"
          @update:model-value="emit('update:apiType', $event as ApiType)" /></div>
      <div><label for="provider-auth-source" class="block text-xs text-zinc-400 mb-1.5">{{ tr('Авторизация') }}</label>
        <BaseSelect id="provider-auth-source" :model-value="credentialSource" :options="sources" :disabled="disabled" class="w-full"
          @update:model-value="emit('update:credentialSource', $event as CredentialSource)" /></div>
    </div>
    <div v-if="credentialSource === 'key'">
      <div class="flex items-center justify-between mb-1.5">
        <label for="provider-api-key" class="text-xs text-zinc-400">{{ tr('API-ключ или токен') }}</label>
        <button type="button" @click="reveal = !reveal" :disabled="disabled" :aria-label="reveal ? tr('Скрыть ключ') : tr('Показать ключ')" class="text-zinc-500 hover:text-zinc-200 cursor-pointer"><EyeOff v-if="reveal" class="w-3.5 h-3.5" /><Eye v-else class="w-3.5 h-3.5" /></button>
      </div>
      <input id="provider-api-key" :value="apiKey" @input="emit('update:apiKey', ($event.target as HTMLInputElement).value)" :disabled="disabled"
        :type="reveal ? 'text' : 'password'" autocomplete="off" spellcheck="false" :placeholder="tr('Необязательно для локального API')"
        class="w-full rounded-lg border border-white/[0.08] bg-black/20 px-3 py-2 text-xs font-mono text-zinc-200" />
      <p class="text-[10px] text-zinc-500 mt-1.5">{{ apiType === 'anthropic_messages' ? tr('Ключ передаётся в x-api-key.') : tr('Токен передаётся как Authorization: Bearer.') }}</p>
    </div>
    <div v-else-if="credentialSource === 'env'">
      <label for="provider-key-env" class="block text-xs text-zinc-400 mb-1.5">{{ tr('Имя переменной окружения') }}</label>
      <input id="provider-key-env" :value="keyEnv" @input="emit('update:keyEnv', ($event.target as HTMLInputElement).value)" :disabled="disabled"
        autocomplete="off" spellcheck="false" placeholder="MY_PROVIDER_API_KEY" class="w-full rounded-lg border border-white/[0.08] bg-black/20 px-3 py-2 text-xs font-mono text-zinc-200" />
      <p class="text-[10px] text-zinc-500 mt-1.5">{{ tr('Значение берётся из окружения процесса GUI и не записывается в конфиг.') }}</p>
    </div>
    <div class="pt-1">
      <button type="button" :disabled="disabled" @click="hasHeaders ? showHeaders = false : headers.length ? showHeaders = true : addHeader()"
        class="text-[11px] text-zinc-400 hover:text-zinc-200 cursor-pointer">{{ tr('HTTP-заголовки') }}<span v-if="headers.length" class="ml-1.5 text-zinc-600">{{ headers.length }}</span></button>
      <div v-if="hasHeaders" class="mt-2 space-y-2">
        <p class="text-[10px] text-zinc-500 leading-relaxed">{{ tr('Произвольная авторизация и параметры подключения. Эти заголовки заменяют стандартные.') }}</p>
        <div v-for="(header, index) in headers" :key="index" class="flex gap-2 items-center">
          <input :value="header.name" @input="updateHeader(index, 'name', ($event.target as HTMLInputElement).value)" :disabled="disabled"
            :aria-label="tr('Имя заголовка {0}', {'0': index + 1})" placeholder="Authorization" autocomplete="off" spellcheck="false"
            class="w-[40%] min-w-0 rounded-lg border border-white/[0.08] bg-black/20 px-2 py-2 text-[11px] font-mono text-zinc-300" />
          <input :value="header.value" @input="updateHeader(index, 'value', ($event.target as HTMLInputElement).value)" :disabled="disabled" :type="reveal ? 'text' : 'password'"
            :aria-label="tr('Значение заголовка {0}', {'0': index + 1})" :placeholder="tr('Значение')" autocomplete="off" spellcheck="false"
            class="flex-1 w-0 min-w-0 rounded-lg border border-white/[0.08] bg-black/20 px-2 py-2 text-[11px] font-mono text-zinc-300" />
          <button type="button" @click="emit('update:headers', headers.filter((_, position) => position !== index))" :disabled="disabled" :aria-label="tr('Удалить заголовок {0}', {'0': index + 1})" class="quiet-icon shrink-0"><X class="w-3 h-3" /></button>
        </div>
        <div class="flex items-center justify-between">
          <button type="button" @click="addHeader" :disabled="disabled" class="inline-flex items-center gap-1 text-[11px] text-zinc-400 hover:text-zinc-200 cursor-pointer"><Plus class="w-3 h-3" />{{ tr('Добавить заголовок') }}</button>
          <button type="button" @click="reveal = !reveal" :disabled="disabled" class="text-[10px] text-zinc-500 hover:text-zinc-200 cursor-pointer">{{ reveal ? tr('Скрыть значения') : tr('Показать значения') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>
