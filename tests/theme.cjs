// Test the real store against a mocked Tauri bridge, including reloads and errors.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const ts = require('typescript');
const vue = require('vue');
const pinia = require('pinia');
const root = path.resolve(__dirname, '..');
let selectedTheme;
let themeModule;
let failSave = false;
const initial = { theme: 'Nord', model: 'saved-model', mode: 'moa', api_key: 'test-only', base_url: 'http://localhost/v1' };
let backend = { ...initial };
async function invoke(name, args) {
  if (name === 'get_config') return { ...backend };
  if (name === 'set_theme') {
    if (failSave) throw new Error('fixture save failure');
    backend = { ...backend, theme: args.theme };
    return { ...backend };
  }
  if (name === 'save_preferences') { backend = { ...backend, ...args.prefs }; return { ...backend }; }
  if (name === 'reset_to_harness_defaults') { backend = { ...initial, theme: 'Emerald' }; return { ...backend }; }
  if (name === 'get_available_skills') return [];
  return null;
}
function load(file) {
  const source = ts.transpileModule(fs.readFileSync(path.join(root, file), 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020 },
  }).outputText;
  const exports = {};
  vm.runInNewContext(source, { exports, console: { ...console, error() {} }, document: {
    documentElement: { setAttribute(name, value) { assert.equal(name, 'data-theme'); selectedTheme = value; } },
  }, require(name) {
    if (name === 'vue') return vue;
    if (name === 'pinia') return pinia;
    if (name === '@tauri-apps/api/core') return { invoke };
    if (name === '../utils/theme') return themeModule;
    if (name === '../utils/activity') return load('src/utils/activity.ts');
    if (name === '../utils/attachments') return load('src/utils/attachments.ts');
    throw new Error(`Unexpected import ${name}`);
  } }, { filename: file });
  return exports;
}
themeModule = load('src/utils/theme.ts');
const { useAgentStore } = load('src/stores/agentStore.ts');
pinia.setActivePinia(pinia.createPinia());

(async () => {
  assert.equal(Object.keys(themeModule.THEME_ACCENTS).length, 10);
  const store = useAgentStore();
  await store.init();
  assert.equal(selectedTheme, 'nord', 'legacy TUI spelling was not applied to the DOM');
  assert.equal(store.themeAccent, '#88c0d0');
  for (const theme of Object.keys(themeModule.THEME_ACCENTS)) {
    await store.setTheme(theme);
    assert.equal(selectedTheme, theme);
    assert.equal(store.themeAccent, themeModule.THEME_ACCENTS[theme]);
    assert.equal(store.config.model, initial.model);
    assert.equal(store.config.api_key, initial.api_key);
    assert.equal(store.config.mode, initial.mode);
  }
  await store.savePreferences({ theme: 'Cyberpunk' });
  assert.equal(selectedTheme, 'cyberpunk', 'saving general preferences did not apply the theme');
  await store.resetToHarnessDefaults();
  assert.equal(selectedTheme, 'emerald', 'reset did not update the visible theme');
  failSave = true;
  await assert.rejects(store.setTheme('nord'));
  assert.equal(selectedTheme, 'emerald', 'failed save changed the visible theme');
  assert.equal(store.config.theme, 'Emerald');
  const css = fs.readFileSync(path.join(root, 'src/assets/main.css'), 'utf8');
  for (const [theme, color] of Object.entries(themeModule.THEME_ACCENTS)) {
    const palette = css.match(new RegExp(`:root\\[data-theme="${theme}"\\] \\{([^}]+)`))?.[1];
    assert(palette?.includes(`--accent-400: ${color}`), `missing active palette for ${theme}`);
  }
  assert(css.includes('--color-amber-400: var(--accent-400)'));
  console.log('PASS all theme palettes, legacy names, immediate store updates, reload/reset, failed saves, and preserved model settings');
})().catch(error => { console.error(error); process.exitCode = 1; });
