// Check provider isolation and stale replies in the real model-loading composable.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const path = require('node:path');
const ts = require('typescript');
const vue = require('vue');
const root = path.resolve(__dirname, '..');
const pending = [];
function load(file) {
  const exports = {};
  const code = ts.transpileModule(fs.readFileSync(path.join(root, file), 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020 },
  }).outputText;
  vm.runInNewContext(code, { exports, setTimeout, clearTimeout, URL, require(name) {
    if (name === 'vue') return vue;
    if (name === '@tauri-apps/api/core') return { invoke(command, args) {
      assert.equal(command, 'list_provider_models');
      return new Promise((resolve, reject) => pending.push({ args, resolve, reject }));
    } };
    if (name === '../utils/providers') return load('src/utils/providers.ts');
    throw new Error(`Unexpected dependency ${name}`);
  } });
  return exports;
}
(async () => {
  const scope = vue.effectScope();
  const connection = vue.ref({ baseUrl: 'https://anymodel.org/v1', apiKey: 'gateway-fixture' });
  const enabled = vue.ref(true);
  let picker;
  scope.run(() => { picker = load('src/composables/useProviderModels.ts').useProviderModels(connection, enabled, false); });
  await vue.nextTick();
  assert.equal(pending.length, 0, 'draft settings must not send credentials automatically');
  const first = picker.refresh();
  connection.value = { baseUrl: 'https://api.openai.com/v1', apiKey: 'native-fixture' };
  await vue.nextTick();
  const second = picker.refresh();
  pending[1].resolve([{ id: 'native-model', architecture: { input_modalities: ['text', 'image'] } }]);
  await second;
  assert.equal(picker.models.value[0].id, 'native-model');
  assert.equal(picker.provider.value.id, 'openai');
  assert.equal(picker.models.value[0].architecture.input_modalities[1], 'image', 'provider capability metadata was lost');
  connection.value = { ...connection.value, models: [{ id: 'native-model' }] };
  await vue.nextTick();
  assert.equal(picker.models.value[0].architecture.input_modalities[1], 'image', 'config replacement lost loaded capabilities');
  pending[0].resolve([{ id: 'gateway/model' }]);
  await first;
  assert.equal(picker.models.value[0].id, 'native-model', 'late reply from old provider replaced current catalog');
  assert.equal(pending[1].args.apiKey, 'native-fixture');
  const failed = picker.refresh();
  pending[2].reject('Provider unavailable');
  await failed;
  assert.equal(picker.models.value.length, 0, 'failed catalog must not retain unrelated models');
  assert.equal(picker.error.value, 'Provider unavailable');
  assert.equal(picker.loading.value, false);
  connection.value = { ...connection.value, apiType: 'anthropic_messages', apiKeyEnv: 'PROVIDER_KEY', headers: {Authorization:'Custom fixture'}, models: [] };
  await vue.nextTick();
  const native = picker.refresh();
  assert.equal(pending[3].args.apiType,'anthropic_messages');assert.equal(pending[3].args.apiKeyEnv,'PROVIDER_KEY');assert.equal(pending[3].args.headers.Authorization,'Custom fixture');
  pending[3].resolve([{id:'native-model',reasoning_efforts:['low','max']}]);await native;
  enabled.value = false;
  await vue.nextTick();
  assert.deepEqual(Array.from(picker.models.value[0].reasoning_efforts),['low','max'],'closing picker lost capability metadata');
  enabled.value=true;await vue.nextTick();
  assert.deepEqual(Array.from(picker.models.value[0].reasoning_efforts),['low','max']);
  enabled.value=false;await vue.nextTick();await picker.refresh();
  assert.equal(pending.length, 4, 'closed picker must not query the provider');
  scope.stop();
  console.log('PASS provider isolation, explicit draft loading, stale response protection, errors, and closed picker');
})().catch(error => { console.error(error); process.exitCode = 1; });
