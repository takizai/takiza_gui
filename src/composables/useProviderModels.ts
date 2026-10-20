import { computed, onScopeDispose, ref, watch, type Ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { providerForUrl } from '../utils/providers';
import type { ApiType, ProviderModel } from '../types';

export function useProviderModels(connection: Readonly<Ref<{ baseUrl: string; apiKey: string; proxy?: string | null; apiType?: ApiType; apiKeyEnv?: string; headers?: Record<string,string>; models?: ProviderModel[] }>>, enabled: Readonly<Ref<boolean>>, autoLoad = true) {
  const catalog = ref<ProviderModel[]>([]);
  const loading = ref(false);
  const error = ref('');
  let generation = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const provider = computed(() => providerForUrl(connection.value.baseUrl));
  const models = computed(() => catalog.value.map(model => {
    const { id } = model;
    return { ...model, id, name: model.name || id, provider: provider.value?.name || '', description: model.description || '' };
  }));
  async function refresh() {
    const request = ++generation;
    if (!enabled.value || !connection.value.baseUrl) { loading.value = false; return; }
    catalog.value = [];
    error.value = '';
    loading.value = true;
    try {
      const result = await invoke<ProviderModel[]>('list_provider_models', {
        baseUrl: connection.value.baseUrl, apiKey: connection.value.apiKey, proxy: connection.value.proxy || null,
        apiType: connection.value.apiType || 'chat_completions', apiKeyEnv: connection.value.apiKeyEnv || null, headers: connection.value.headers || {},
      });
      if (request === generation) catalog.value = result;
    } catch (cause) {
      if (request === generation) error.value = String(cause);
    } finally { if (request === generation) loading.value = false; }
  }
  watch([() => connection.value.baseUrl, () => connection.value.apiKey, () => connection.value.proxy, () => connection.value.apiType, () => connection.value.apiKeyEnv, () => JSON.stringify(connection.value.headers || {})], () => {
    ++generation;
    clearTimeout(timer);
    catalog.value = connection.value.models || [];
    error.value = '';
    loading.value = !catalog.value.length && autoLoad && enabled.value && !!connection.value.baseUrl;
    if (loading.value) timer = setTimeout(refresh, 350);
  }, { immediate: true });
  // Hiding the picker must retain its model metadata for the agent settings tab.
  watch(enabled, active => {
    ++generation; clearTimeout(timer); loading.value = false;
    if (active && autoLoad && !catalog.value.length && connection.value.baseUrl) {
      loading.value = true; timer = setTimeout(refresh, 350);
    }
  });
  onScopeDispose(() => { ++generation; clearTimeout(timer); });
  return { models, loading, error, provider, refresh };
}
