<script setup lang="ts">
import { useI18n } from "../../i18n";
import BaseModal from "../common/BaseModal.vue";
import BaseSelect from "../common/BaseSelect.vue";
import ProviderConnectionFields from "../common/ProviderConnectionFields.vue";
import { ref, watch, computed, nextTick } from "vue";
import { reasoningEfforts } from "../../utils/effort";
import { normalizeTheme } from "../../utils/theme";
import { useAgentStore } from "../../stores/agentStore";
import { THEME_OPTIONS } from "../../utils/models";
import { PROVIDERS, providerForUrl } from "../../utils/providers";
import { useProviderModels } from "../../composables/useProviderModels";
import type { ThemeName, Language, ApiType, CredentialSource, HeaderEntry, AppPreferences } from "../../types";
import { Plus, ChevronRight, RefreshCw, Pencil, Check, LoaderCircle } from 'lucide-vue-next';

const { tr, locale, setLocale } = useI18n();

const props = defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const agentStore = useAgentStore();

const activeTab = ref<'connection' | 'appearance' | 'agent'>('connection');
const connectionExpanded = ref(false);
const manualModel = ref(false);
const selectedPreset = ref('');
const initialConnection = ref('');
const initialAgent = ref('');

const provider = ref("custom");
const selectedProfile = ref("");
const addingProvider = ref(false);
const newProviderName = ref("");
const providerNameInput = ref<HTMLInputElement | null>(null);
const apiType = ref<ApiType>('chat_completions');
const credentialSource = ref<CredentialSource>('key');
const keyEnv = ref('');
const headerRows = ref<HeaderEntry[]>([]);
const effectiveEnv = computed(() => credentialSource.value === 'env' ? keyEnv.value.trim() : '');
const effectiveKey = computed(() => credentialSource.value === 'key' ? apiKey.value.trim() : '');
const connectionHeaders = computed(() => Object.fromEntries(headerRows.value.filter(row => row.name.trim()).map(row => [row.name.trim(), row.value])));
function loadTransport(profile: {api_type?: ApiType; api_key_env?: string | null; headers?: Record<string,string>}, key: string) {
  apiType.value = profile.api_type || 'chat_completions';
  keyEnv.value = profile.api_key_env || '';
  credentialSource.value = keyEnv.value ? 'env' : key ? 'key' : 'none';
  headerRows.value = Object.entries(profile.headers || {}).map(([name,value]) => ({name,value}));
}
function resetTransport(type: ApiType = 'chat_completions') {
  apiType.value = type; credentialSource.value = 'key'; keyEnv.value = ''; headerRows.value = [];
}
let previousConnection: { selectedProfile: string; baseUrl: string; model: string; apiKey: string; provider: string; apiType: ApiType; credentialSource: CredentialSource; keyEnv: string; headers: HeaderEntry[] } | undefined;
async function startAddingProvider() {
  if (isSaving.value || addingProvider.value) return;
  previousConnection = { selectedProfile: selectedProfile.value, baseUrl: baseUrl.value, model: model.value, apiKey: apiKey.value, provider: provider.value, apiType: apiType.value, credentialSource: credentialSource.value, keyEnv: keyEnv.value, headers: headerRows.value.map(row => ({...row})) };
  addingProvider.value = true; resetTransport(); connectionExpanded.value = true; manualModel.value = false; selectedPreset.value = '';
  selectedProfile.value = ""; baseUrl.value = ""; model.value = ""; apiKey.value = ""; provider.value = "custom";
  newProviderName.value = ""; saveError.value = "";
  activeTab.value = "connection";
  await nextTick(); providerNameInput.value?.focus();
}
function cancelAddingProvider() {
  if (isSaving.value) return;
  addingProvider.value = false;
  if (previousConnection) {
    selectedProfile.value = previousConnection.selectedProfile; baseUrl.value = previousConnection.baseUrl;
    model.value = previousConnection.model; apiKey.value = previousConnection.apiKey; provider.value = previousConnection.provider;
    apiType.value = previousConnection.apiType; credentialSource.value = previousConnection.credentialSource; keyEnv.value = previousConnection.keyEnv; headerRows.value = previousConnection.headers;
  }
  previousConnection = undefined; newProviderName.value = ""; saveError.value = ""; connectionExpanded.value = false;
}
const savedProfiles = computed(() => agentStore.config?.providers || {});
const profileOptions = computed(() => Object.entries(savedProfiles.value).map(([id, profile]) => ({ value: id, label: profile.name || id, description: profile.base_url })));
const languageOptions = [{ value: 'ru', label: 'Русский' }, { value: 'en', label: 'English' }, { value: 'zh', label: '简体中文' }];
function selectSavedProfile(id: string) {
  if (isSaving.value) return;
  const profile = savedProfiles.value[id];
  if (!profile) return;
  addingProvider.value = false; connectionExpanded.value = false; manualModel.value = false; saveError.value = '';
  selectedProfile.value = id;
  baseUrl.value = profile.base_url;
  model.value = profile.model;
  apiKey.value = profile.api_key_env ? "" : profile.api_key || "";
  loadTransport(profile, apiKey.value);
  provider.value = providerForUrl(profile.base_url)?.id || "custom";
}

const baseUrl = ref("");
const model = ref("");
const apiKey = ref("");
const proxy = ref("");
const autoApprove = ref(false);
const effort = ref("");
const maxSteps = ref(100);
const currentTheme = ref<ThemeName>("amber");

const isSaving = ref(false);
const saveError = ref("");

const draftProfile = computed(() => {
  const profile = savedProfiles.value[selectedProfile.value];
  if (!profile || profile.base_url.replace(/\/+$/, '') !== baseUrl.value.replace(/\/+$/, '')
    || (profile.api_type || 'chat_completions') !== apiType.value || (profile.api_key_env || '') !== effectiveEnv.value
    || (!effectiveEnv.value && (profile.api_key || '') !== effectiveKey.value)
    || JSON.stringify(Object.entries(profile.headers || {}).sort()) !== JSON.stringify(Object.entries(connectionHeaders.value).sort())) return undefined;
  return profile;
});
const connection = computed(() => ({ baseUrl: baseUrl.value, apiKey: effectiveKey.value, apiType: apiType.value, apiKeyEnv: effectiveEnv.value, headers: connectionHeaders.value, proxy: proxy.value, models: draftProfile.value?.models }));
const { models: providerModels, loading: modelsLoading, error: modelsError, refresh: refreshModels } = useProviderModels(connection, computed(() => props.isOpen && activeTab.value === "connection"), false);

function applyPreset(p: (typeof PROVIDERS)[number]) {
  if (isSaving.value) return;
  if (addingProvider.value) {
    if (baseUrl.value.replace(/\/+$/, "") !== p.url) { apiKey.value = ""; model.value = ""; }
    resetTransport(p.id === 'anthropic' ? 'anthropic_messages' : 'chat_completions');
    provider.value = p.id; baseUrl.value = p.url;
    if (!newProviderName.value.trim()) newProviderName.value = p.name;
    return;
  }
  const saved = Object.entries(savedProfiles.value).find(([, profile]) => profile.base_url.replace(/\/+$/, "") === p.url);
  if (saved) { selectSavedProfile(saved[0]); return; }
  if (baseUrl.value.replace(/\/+$/, "") === p.url) return;
  selectedProfile.value = "";
  apiKey.value = "";
  provider.value = p.id;
  baseUrl.value = p.url;
  resetTransport(p.id === 'anthropic' ? 'anthropic_messages' : 'chat_completions');
  model.value = "";
}

const connectionSnapshot = computed(() => JSON.stringify([selectedProfile.value, baseUrl.value.trim(), model.value.trim(), effectiveKey.value,
  effectiveEnv.value, apiType.value, Object.entries(connectionHeaders.value).sort()]));
const agentSnapshot = computed(() => JSON.stringify([proxy.value.trim(), autoApprove.value, effort.value, maxSteps.value]));
const connectionDirty = computed(() => addingProvider.value || connectionSnapshot.value !== initialConnection.value);
const agentDirty = computed(() => agentSnapshot.value !== initialAgent.value);
const hasPendingChanges = computed(() => connectionDirty.value || agentDirty.value);
const presetOptions = computed(() => PROVIDERS.map(p => ({ value: p.id, label: p.name })));
const modelOptions = computed(() => {
  const options = providerModels.value.map(item => ({value:item.id,label:item.name || item.id,description:item.name && item.name !== item.id ? item.id : undefined}));
  if (model.value && !options.some(option => option.value === model.value)) options.unshift({value:model.value,label:model.value,description:undefined});
  return options;
});
const availableEfforts = computed(() => reasoningEfforts(model.value, apiType.value, providerModels.value.find(item => item.id === model.value)));
const effortOptions = computed(() => [{value:'',label:tr('Автоматически')}, ...availableEfforts.value.map(value => ({value,label:value}))]);
const selectedEffort = computed({get: () => availableEfforts.value.includes(effort.value as import('../../types').ReasoningEffort) ? effort.value : '',set: (value: string) => {effort.value = value;}});
const protocolLabel = computed(() => apiType.value === 'anthropic_messages' ? 'Anthropic Messages' : apiType.value === 'openai_responses' ? 'OpenAI Responses' : 'Chat Completions');
function choosePreset(id: string) {
  const preset = PROVIDERS.find(item => item.id === id);
  if (preset) { selectedPreset.value = id; applyPreset(preset); }
}
function loadDraft() {
  const config = agentStore.config;
  if (!config) return;
  addingProvider.value = false; newProviderName.value = ''; previousConnection = undefined; saveError.value = '';
  selectedProfile.value = config.active_provider || '';
  baseUrl.value = config.base_url; model.value = config.model; apiKey.value = config.api_key_env ? '' : config.api_key;
  loadTransport({api_type:config.api_type,api_key_env:config.api_key_env,headers:config.provider_headers},apiKey.value);
  proxy.value = config.proxy || ''; autoApprove.value = config.auto_approve; effort.value = config.effort || '';
  maxSteps.value = config.max_steps ?? 100; currentTheme.value = normalizeTheme(config.theme);
  provider.value = providerForUrl(config.base_url)?.id || 'custom';
  connectionExpanded.value = !config.base_url; manualModel.value = false; selectedPreset.value = '';
  initialConnection.value = connectionSnapshot.value; initialAgent.value = agentSnapshot.value;
}

watch(() => props.isOpen, open => { if (open) loadDraft(); }, { immediate: true });

const languageSaving = ref(false);
const themeSaving = ref(false);
const uiBusy = computed(() => isSaving.value || languageSaving.value || themeSaving.value);
const appearanceStatus = ref('');
async function selectLanguage(value: string) {
  if (uiBusy.value) return;
  languageSaving.value = true; saveError.value = ''; appearanceStatus.value = '';
  try {
    const language = value as Language;
    await agentStore.setLanguage(language);
    await setLocale(language); appearanceStatus.value = tr('Сохранено');
  }
  catch { saveError.value = tr("Не удалось сохранить язык"); }
  finally { languageSaving.value = false; }
}

async function selectTheme(themeId: string) {
  if (uiBusy.value || themeId === currentTheme.value) return;
  const previousTheme = currentTheme.value;
  currentTheme.value = themeId as ThemeName; themeSaving.value = true; saveError.value = ''; appearanceStatus.value = '';
  try { await agentStore.setTheme(themeId as ThemeName); appearanceStatus.value = tr('Сохранено'); }
  catch { currentTheme.value = previousTheme; saveError.value = tr('Не удалось сохранить тему'); }
  finally { themeSaving.value = false; }
}

async function handleSave() {
  if (uiBusy.value) return;
  if (!hasPendingChanges.value) { emit('close'); return; }
  isSaving.value = true;

  saveError.value = "";
  try {
    const prefs: AppPreferences = {};
    if (connectionDirty.value) {
    if (addingProvider.value && !newProviderName.value.trim()) throw new Error(tr("Укажите название провайдера."));
    if (!baseUrl.value.trim() || !model.value.trim()) throw new Error(tr("Укажите URL провайдера и ID модели."));
    {
      let endpoint: URL;
      try { endpoint = new URL(baseUrl.value.trim()); } catch { throw new Error(tr("Укажите корректный HTTP или HTTPS адрес API.")); }
      if (!['http:', 'https:'].includes(endpoint.protocol) || !endpoint.hostname) throw new Error(tr("Укажите корректный HTTP или HTTPS адрес API."));
    }
    if (credentialSource.value === 'env' && !/^[A-Za-z_][A-Za-z0-9_]*$/.test(effectiveEnv.value)) throw new Error(tr('Некорректное имя переменной окружения.'));
    const headerNames = new Set<string>();
    for (const row of headerRows.value) {
      if (!row.name.trim() && !row.value) continue;
      if (!/^[!#$%&'*+.^_`|~0-9A-Za-z-]+$/.test(row.name.trim())) throw new Error(tr('Некорректное имя HTTP-заголовка.'));
      if (/[^\t\x20-\x7e\x80-\xff]/.test(row.value)) throw new Error(tr('Некорректное значение HTTP-заголовка.'));
      const normalized = row.name.trim().toLowerCase();
      if (headerNames.has(normalized)) throw new Error(tr('HTTP-заголовки не должны повторяться.'));
      headerNames.add(normalized);
    }
    Object.assign(prefs, {
      ...(addingProvider.value ? { new_provider_name: newProviderName.value.trim() } : selectedProfile.value ? { active_provider: selectedProfile.value } : {}),
      base_url: baseUrl.value.trim(), model: model.value.trim(), api_key: effectiveKey.value,
      api_key_env: effectiveEnv.value, api_type: apiType.value, headers: connectionHeaders.value,
    });
    }
    if (agentDirty.value) {
      if (!Number.isInteger(maxSteps.value) || maxSteps.value < 0) throw new Error(tr('Лимит шагов должен быть целым числом от 0.'));
      Object.assign(prefs,{max_steps:maxSteps.value,proxy:proxy.value.trim(),auto_approve:autoApprove.value,...(availableEfforts.value.length ? {effort:selectedEffort.value} : {})});
    }
    if (providerModels.value.length && (connectionDirty.value || agentDirty.value)) prefs.model_catalog = providerModels.value;
    await agentStore.savePreferences(prefs);
    if (addingProvider.value) {
      addingProvider.value = false; previousConnection = undefined; newProviderName.value = "";
      selectedProfile.value = agentStore.config?.active_provider || "";
    }
    initialConnection.value = connectionSnapshot.value; initialAgent.value = agentSnapshot.value;
    emit('close');
  } catch (e) {
    saveError.value = e instanceof Error ? e.message : tr(String(e));
  } finally {
    isSaving.value = false;
  }
}

</script>

<template>
  <BaseModal :is-open="isOpen" :title="tr('Настройки')" size="lg" panel-class="settings-modal" :dismissible="!uiBusy" @close="emit('close')">
    <nav :inert="uiBusy" class="settings-tabs" :aria-label="tr('Разделы настроек')">
      <button v-for="tab in [{id:'connection',label:tr('Подключение')},{id:'appearance',label:tr('Оформление')},{id:'agent',label:tr('Агент')}]" :key="tab.id"
        type="button" :aria-current="activeTab === tab.id ? 'page' : undefined" :class="{'settings-tab-active':activeTab === tab.id}"
        @click="activeTab = tab.id as typeof activeTab">{{ tab.label }}</button>
    </nav>
    <div class="modal-scroll settings-content" :inert="uiBusy" :aria-busy="uiBusy">
      <div v-if="activeTab === 'connection'" class="space-y-6">
        <div>
          <div class="settings-label-row">
            <label :for="addingProvider ? 'new-provider-name' : 'saved-provider'">{{ addingProvider ? tr('Новый провайдер') : tr('Провайдер') }}</label>
            <button v-if="!addingProvider" type="button" @click="startAddingProvider" class="settings-text-action" :aria-label="tr('Добавить провайдера')"><Plus class="w-3.5 h-3.5" />{{ tr('Добавить') }}</button>
            <button v-else type="button" @click="cancelAddingProvider" class="settings-text-action">{{ tr('Отменить добавление') }}</button>
          </div>
          <input v-if="addingProvider" id="new-provider-name" ref="providerNameInput" v-model="newProviderName" :placeholder="tr('Название провайдера')" autocomplete="off" class="settings-input" />
          <BaseSelect v-else-if="profileOptions.length" id="saved-provider" :model-value="selectedProfile" :options="profileOptions" searchable :placeholder="tr('Выберите профиль')" class="w-full" @update:model-value="selectSavedProfile" />
          <p v-else class="text-xs text-zinc-500">{{ tr('Текущее подключение') }}</p>
        </div>

        <div v-if="addingProvider" class="space-y-4">
          <details class="settings-disclosure"><summary><ChevronRight class="w-3 h-3" />{{ tr('Заполнить из пресета') }}</summary>
            <BaseSelect id="provider-preset" :model-value="selectedPreset" :options="presetOptions" :placeholder="tr('Выбрать пресет')" class="w-full mt-3" @update:model-value="choosePreset" />
          </details>
          <ProviderConnectionFields v-model:base-url="baseUrl" v-model:api-key="apiKey" v-model:api-type="apiType" v-model:credential-source="credentialSource" v-model:key-env="keyEnv" v-model:headers="headerRows" />
        </div>
        <div v-else>
          <button type="button" @click="connectionExpanded = !connectionExpanded" :aria-expanded="connectionExpanded" class="settings-connection-toggle">
            <span><span class="block text-xs text-zinc-300">{{ tr('Параметры подключения') }}</span><span class="block text-[10px] text-zinc-500 mt-1">{{ protocolLabel }}</span></span>
            <ChevronRight class="w-3.5 h-3.5 text-zinc-500 transition-transform" :class="{'rotate-90':connectionExpanded}" />
          </button>
          <ProviderConnectionFields v-if="connectionExpanded" class="mt-4" v-model:base-url="baseUrl" v-model:api-key="apiKey" v-model:api-type="apiType" v-model:credential-source="credentialSource" v-model:key-env="keyEnv" v-model:headers="headerRows" />
        </div>

        <div class="settings-model-field">
          <div class="settings-label-row">
            <label :for="manualModel ? 'settings-model-id' : 'settings-model'">{{ tr('Модель') }}</label>
            <div class="flex items-center gap-3">
              <button type="button" @click="manualModel = !manualModel" class="settings-text-action" :aria-pressed="manualModel"><Pencil class="w-3 h-3" />{{ manualModel ? tr('Выбрать из списка') : tr('Ввести ID') }}</button>
              <button type="button" @click="refreshModels" :disabled="modelsLoading || !baseUrl.trim()" class="settings-text-action" :aria-label="tr('Загрузить модели провайдера')" :title="tr('Обновить модели')"><RefreshCw class="w-3 h-3" :class="{'animate-spin motion-reduce:animate-none':modelsLoading}" />{{ modelsLoading ? tr('Загрузка…') : tr('Обновить') }}</button>
            </div>
          </div>
          <input v-if="manualModel" id="settings-model-id" v-model="model" :placeholder="tr('ID модели у выбранного провайдера')" class="settings-input font-mono" spellcheck="false" />
          <BaseSelect v-else id="settings-model" v-model="model" :options="modelOptions" :disabled="modelsLoading || !modelOptions.length" searchable
            :placeholder="modelsLoading ? tr('Загрузка…') : tr('Загрузите список или введите ID')" class="w-full" />
          <p v-if="modelsError" role="status" class="mt-2 text-[11px] text-rose-300">{{ tr(modelsError) }}</p>
          <p v-else class="mt-2 text-[10px] text-zinc-600">{{ tr('Модели текущего провайдера. Можно указать ID вручную.') }}</p>
        </div>
      </div>

      <div v-else-if="activeTab === 'appearance'" class="space-y-7">
        <div class="settings-row"><label for="interface-language">{{ tr('Язык интерфейса') }}</label>
          <BaseSelect id="interface-language" :model-value="locale" :options="languageOptions" class="w-36" @update:model-value="selectLanguage" /></div>
        <div>
          <p class="text-xs text-zinc-400 mb-3">{{ tr('Тема') }}</p>
          <div class="settings-theme-grid">
            <button v-for="theme in THEME_OPTIONS" :key="theme.id" type="button" @click="selectTheme(theme.id)" :aria-pressed="currentTheme === theme.id"
              :title="tr(theme.desc)" class="settings-theme-option" :class="{'settings-theme-selected':currentTheme === theme.id}">
              <span class="settings-theme-swatch" :style="{backgroundColor:theme.color}"><Check v-if="currentTheme === theme.id" class="w-3 h-3 text-black/70" /></span>
              <span>{{ theme.id.charAt(0).toUpperCase() + theme.id.slice(1) }}</span>
            </button>
          </div>
        </div>
        <p class="text-[11px] text-zinc-600">{{ tr('Изменения применяются сразу.') }}</p>
      </div>

      <div v-else class="space-y-5">
        <div class="settings-row"><div><label for="command-approval">{{ tr('Подтверждения команд') }}</label><p class="settings-hint">{{ autoApprove ? tr('Команды выполняются автономно') : tr('Перед запуском команд') }}</p></div>
          <button id="command-approval" type="button" role="switch" :aria-checked="!autoApprove" :aria-label="tr('Подтверждения команд')" @click="autoApprove = !autoApprove" class="settings-switch" :class="{'settings-switch-on':!autoApprove}"><span /></button></div>
        <div class="settings-row"><label for="reasoning-effort">{{ tr('Глубина рассуждений') }}</label><BaseSelect v-if="availableEfforts.length" id="reasoning-effort" v-model="selectedEffort" :options="effortOptions" class="w-36" /><span v-else class="text-xs text-zinc-500">{{ tr('Модель не поддерживает смену effort') }}</span></div>
        <div class="settings-row"><div><label for="max-steps">{{ tr('Лимит шагов агента') }}</label><p class="settings-hint">{{ tr('0 — без ограничения') }}</p></div><input id="max-steps" v-model.number="maxSteps" type="number" min="0" step="1" class="settings-input !w-28 text-right" /></div>
        <details class="settings-disclosure pt-3 border-t border-white/[0.06]"><summary><ChevronRight class="w-3 h-3" />{{ tr('Дополнительно') }}</summary>
          <div class="pt-4 space-y-4">
            <div><label for="settings-proxy" class="block text-xs text-zinc-400 mb-2">{{ tr('Прокси') }}</label><input id="settings-proxy" v-model="proxy" class="settings-input font-mono" placeholder="http://… / socks5://…" /></div>
            <div><p class="text-[11px] text-zinc-500 mb-1">{{ tr('Проект') }}</p><p class="text-[11px] text-zinc-400 break-all select-text">{{ agentStore.config?.workspace_dir }}</p></div>
            <p class="text-[10px] text-zinc-600">{{ tr('Профили подключений общие с CLI.') }}</p>
          </div>
        </details>
      </div>
    </div>
    <footer class="settings-footer">
      <div class="min-w-0 flex-1 mr-4">
        <p v-if="saveError" role="alert" class="text-[11px] text-rose-300 break-words">{{ saveError }}</p>
        <p v-else-if="uiBusy" role="status" class="flex gap-2 items-center text-[11px] text-zinc-500"><LoaderCircle class="w-3 h-3 animate-spin motion-reduce:animate-none" />{{ tr('Сохранение...') }}</p>
        <button v-else-if="hasPendingChanges" type="button" @click="loadDraft" class="settings-text-action">{{ tr('Сбросить изменения') }}</button>
        <p v-else-if="appearanceStatus" role="status" class="text-[11px] text-zinc-500">{{ appearanceStatus }}</p>
      </div>
      <button v-if="hasPendingChanges" type="button" @click="emit('close')" :disabled="uiBusy" class="settings-cancel">{{ tr('Отмена') }}</button>
      <button type="button" @click="handleSave" :disabled="uiBusy" class="settings-save">{{ addingProvider ? tr('Добавить и выбрать') : hasPendingChanges ? tr('Сохранить') : tr('Готово') }}</button>
    </footer>
  </BaseModal>
</template>
