// Exercise the real Vue setup and command handlers with a mocked Tauri bridge.
// Run: node tests/prompt-input.cjs
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const ts = require('typescript');
const { parse, compileScript } = require('@vue/compiler-sfc');
const vue = require('vue');
const root = path.resolve(__dirname, '..');
const calls = [];
const prompts = [];
const imageModel = vue.ref(true);
const panels = [];
let failInvoke = false;
const skills = [{ name: 'design', description: 'Design rules' }, { name: 'help', description: 'Help skill' }];
const agent = vue.reactive({
  availableSkills: skills, isStreaming: false, currentSession: null, operationBusy: false, pendingPrompts: [],
  createNewSession: async () => { agent.currentSession = await invoke('new_session'); },
  loadSession: async id => { agent.currentSession = await invoke('load_session', { id }); },
  openRestore: async () => { calls.push(['restore']); },
  stop: async () => { calls.push(['stop']); },
  config: { model: 'test-model', theme: 'amber', mode: 'manual', auto_approve: false, workspace_dir: '/workspace' },
  gitStatus: { branch: 'main', diff: 'diff fixture' },
  sendPrompt: async text => { prompts.push(text); },
  fetchSkills: async () => {}, refreshGit: async () => {},
  savePreferences: async prefs => { calls.push(['preferences', prefs]); Object.assign(agent.config, prefs); },
  setTheme: async theme => { agent.config.theme = theme; },
  setMode: async mode => { agent.config.mode = mode; },
  setEffort: async effort => { agent.config.effort = effort; },
});
const sessions = { sessions: [{ id: 'saved-chat', title: 'Saved chat' }], fetchSessions: async () => {} };
async function invoke(name, args) {
  calls.push([name, args]);
  if (failInvoke) throw new Error('Tauri fixture failure');
  if (name === 'import_image') return { text: '', images: [{ marker: '[Image: \"/cached/image.png\"]', url: 'data:image/png;base64,fixture', name: 'image.png' }] };
  if (name === 'import_text_file') return { marker: `[File: ${JSON.stringify({name:args.name,path:'/cached/'+args.name})}]`, name: args.name, size: args.bytes.length };
  if (name === 'paste_attachments') return { text: args.text || '', images: [] };
  return { id: args?.id || 'new-chat', messages: [], history: [] };
}
let utils, commandModule;
function evaluate(source, filename) {
  const compiled = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020, esModuleInterop: true } }).outputText;
  const exports = {};
  vm.runInNewContext(compiled, { exports, document: { getElementById: () => null }, require(name) {
    if (name.endsWith('/i18n')) return { tr: (text, args = {}) => text.replace(/\{(\w+)\}/g, (match, key) => args[key] ?? match), useI18n: () => ({ tr: (text, args = {}) => text.replace(/\{(\w+)\}/g, (match, key) => args[key] ?? match) }) };
    if (name === 'vue') return { ...vue, onMounted: () => {}, onUnmounted: () => {} };
    if (name.includes('stores/agentStore')) return { useAgentStore: () => agent };
    if (name.includes('stores/sessionStore')) return { useSessionStore: () => sessions };
    if (name === '@tauri-apps/api/core') return { invoke };
    if (name === '@tauri-apps/api/webviewWindow') return {};
    if (name.includes('utils/attachments')) return evaluate(fs.readFileSync(path.join(root, 'src/utils/attachments.ts'), 'utf8'), 'attachments.ts');
    if (name.includes('utils/promptCommands')) return utils;
    if (name.includes('utils/modelAttachments')) return evaluate(fs.readFileSync(path.join(root, 'src/utils/modelAttachments.ts'), 'utf8'), 'modelAttachments.ts');
    if (name.includes('composables/useProviderModels')) return { useProviderModels: () => ({ models: vue.computed(() => [{id:agent.config.model, input_modalities: imageModel.value ? ['text','image'] : ['text']}]), refresh: async () => {} }) };
    if (name.includes('composables/usePromptCommands')) return commandModule;
    if (name.includes('utils/effort')) return require('./load-effort.cjs');
    if (name === 'lucide-vue-next' || name.endsWith('.vue')) return {};
    throw new Error(`Unexpected dependency ${name}`);
  } }, { filename });
  return exports;
}
utils = evaluate(fs.readFileSync(path.join(root, 'src/utils/promptCommands.ts'), 'utf8'), 'promptCommands.ts');
commandModule = evaluate(fs.readFileSync(path.join(root, 'src/composables/usePromptCommands.ts'), 'utf8'), 'usePromptCommands.ts');
const descriptor = parse(fs.readFileSync(path.join(root, 'src/components/input/PromptInput.vue'), 'utf8')).descriptor;
const component = evaluate(compileScript(descriptor, { id: 'prompt-test' }).content, 'PromptInput.vue').default;
const input = component.setup({}, { expose() {}, emit(name) { panels.push(name); } });
input.textareaRef.value = { selectionStart: 0, scrollHeight: 30, style: {}, focus() {}, setSelectionRange(start) { this.selectionStart = start; } };
const commands = commandModule.usePromptCommands(panel => panels.push(panel));
async function flush() { for (let i = 0; i < 8; i++) await vue.nextTick(); }
async function draft(text, cursor = text.length) { await input.setDraft(text, cursor); await flush(); }
function key(name, extra = {}) {
  const event = { key: name, shiftKey: false, isComposing: false, keyCode: 0, prevented: false, preventDefault() { this.prevented = true; }, ...extra };
  input.handleKeyDown(event);
  return event;
}

(async () => {
  const capabilities = evaluate(fs.readFileSync(path.join(root, 'src/utils/modelAttachments.ts'), 'utf8'), 'modelAttachments.ts');
  assert.equal(capabilities.supportsImageInput({id:'claude-any-alias'}),false,'model names must not imply image support');
  assert.equal(capabilities.supportsImageInput({id:'test',input_modalities:['text']}),false);
  assert.equal(capabilities.supportsImageInput({id:'test',architecture:{input_modalities:['text','image']}}),true);
  assert.equal(capabilities.supportsImageInput({id:'test',capabilities:{vision:true}}),true);
  assert(utils.getPromptCompletions('/', 1, skills).some(item => item.label === '/new'));
  assert(utils.getPromptCompletions('@des', 4, skills).every(item => item.kind === 'skill'));
  assert.equal(utils.getPromptCompletions('/help', 5, skills).filter(item => item.label === '/help').length, 1);
  assert.equal(utils.parsePromptInput('/help', skills).kind, 'command');
  assert.equal(utils.parsePromptInput('/design build a page', skills).text, '@design build a page');
  assert.equal(utils.parsePromptInput('/bogus', skills).kind, 'unknown');
  assert.equal(utils.parsePromptInput('Explain /model please', skills).kind, 'prompt');
  assert.equal(utils.getPromptCompletions('/mode m', 7, skills).length, 2);
  await commands.execute('mode', 'moa');
  assert.equal(agent.config.mode, 'manual', 'unfinished account mode must stay unavailable');
  assert(commands.output.value.includes('в разработке'));
  await commands.execute('model', 'vendor/new-model');
  assert.equal(agent.config.model, 'vendor/new-model');
  const effortBefore = agent.config.effort;
  assert.equal(await commands.execute('effort','high'),false);
  assert.equal(agent.config.effort,effortBefore);
  agent.config.model='o3-mini';assert.equal(await commands.execute('effort','high'),true);assert.equal(agent.config.effort,'high');
  assert.equal(await commands.execute('effort','xhigh'),false);assert.equal(agent.config.effort,'high');
  agent.config.model='gpt-5-pro';assert.equal(await commands.execute('effort','low'),false);
  assert.equal(await commands.execute('effort','auto'),true);assert.equal(agent.config.effort,'');
  await commands.execute('theme', 'nord');
  assert.equal(agent.config.theme, 'nord');
  assert.equal(await commands.execute('theme', 'wrong'), false);
  assert.equal(agent.config.theme, 'nord');
  await commands.execute('approval', 'off');
  assert.equal(agent.config.auto_approve, true);
  await commands.execute('approval', 'on');
  assert.equal(agent.config.auto_approve, false);
  await commands.execute('resume', 'saved-chat');
  assert.equal(agent.currentSession.id, 'saved-chat');
  failInvoke = true;
  assert.equal(await commands.execute('resume', 'missing'), false);
  assert.match(commands.error.value, /Tauri fixture failure/);
  failInvoke = false;
  await commands.execute('diff', '');
  assert.equal(commands.output.value, 'diff fixture');
  await commands.execute('skills', '');
  assert(panels.includes('skills'));

  await draft('/ne');
  assert(input.showCompletions.value);
  assert(key('Tab').prevented);
  await flush();
  assert.equal(input.inputPrompt.value, '/new ');
  key('Enter'); await flush();
  assert.equal(agent.currentSession.id, 'new-chat');
  assert.equal(prompts.length, 0, 'local command reached the model');
  await draft('/bogus'); await input.submit();
  assert.match(input.error.value, /Неизвестная команда/);
  assert.equal(input.inputPrompt.value, '/bogus');
  assert.equal(prompts.length, 0, 'unknown command reached the model');
  await draft('/');
  key('ArrowDown'); await flush();
  assert.equal(input.selectedCompletion.value, 1);
  key('Escape'); await flush();
  assert.equal(input.showCompletions.value, false);
  await draft('/d');
  assert(input.showCompletions.value, 'menu did not reopen after editing');
  assert.equal(key('Enter', { shiftKey: true }).prevented, false);
  assert.equal(key('Enter', { isComposing: true }).prevented, false);
  await draft('Make @des trailing text', 9);
  key('Tab'); await flush();
  assert.equal(input.inputPrompt.value, 'Make @design trailing text', 'completion lost the draft suffix');
  await draft('/design');
  key('Enter'); await flush();
  assert.equal(input.inputPrompt.value, '@design ');
  assert.equal(prompts.length, 0, 'selecting a skill submitted an unfinished task');
  assert(utils.getPromptCompletions('/gri',4,skills).some(item=>item.label==='/grill-me'));
  assert.equal(utils.parsePromptInput('/grill-me',skills).text,'/grill-me');
  assert.equal(utils.parsePromptInput('/grill-me architecture',skills).kind,'prompt');
  assert.equal(utils.parsePromptInput('/grill-meet',skills).kind,'unknown');
  await draft('/grill-me Архитектура редактора');await input.submit();
  assert.equal(prompts.at(-1),'/grill-me Архитектура редактора');assert.equal(input.inputPrompt.value,'');
  await draft('@design build a page'); await input.submit();
  assert.equal(prompts.at(-1), '@design build a page');
  await draft('plain question'); await input.submit();
  assert.equal(prompts.at(-1), 'plain question');
  await draft('/model'); key('Enter'); await flush();
  assert.equal(input.isModelMenuOpen.value, true);
  await draft('/usage'); key('Enter'); await flush();
  assert(panels.includes('open-usage'));
  await draft('/theme nope'); await input.submit();
  assert.equal(input.inputPrompt.value, '/theme nope', 'failed command lost its draft');
  await draft('clear this'); key('Escape'); key('Escape'); await flush();
  assert.equal(input.inputPrompt.value, '', 'double Esc did not clear the draft');
  agent.isStreaming = true;
  await draft('queued question'); await input.submit();
  assert.equal(prompts.at(-1), 'queued question', 'busy composer refused another prompt');
  await commands.execute('stop', ''); await commands.execute('rewind', '');
  assert(calls.some(([name]) => name === 'stop'));
  assert(calls.some(([name]) => name === 'restore'));
  assert(utils.getPromptCompletions('$des', 4, skills).some(item => item.kind === 'skill'));
  agent.isStreaming = false;
  await draft('paste here', 6);
  input.textareaRef.value.selectionEnd = 6;
  await input.onPaste({ preventDefault() {}, clipboardData: { files: [], getData: type => type === 'text/plain' ? 'safe text ' : '' } });
  assert.equal(input.inputPrompt.value, 'paste safe text here');
  await draft('describe');
  await input.onPaste({ preventDefault() {}, clipboardData: { files: [{ type: 'image/png', arrayBuffer: async () => new Uint8Array([137, 80, 78, 71]).buffer }], getData: () => '' } });
  assert.equal(input.pendingImages.value.length, 1);
  await input.submit();
  assert(prompts.at(-1).includes('[Image: "/cached/image.png"]'));
  assert.equal(input.pendingImages.value.length, 0);
  imageModel.value = false;
  await draft('text with file');
  const pick = files => input.selectFiles({target:{files,value:'selected'}});
  const source = {name:'hello.ts',type:'text/plain',size:5,arrayBuffer:async()=>new Uint8Array([104,101,108,108,111]).buffer};
  await pick([source]);
  assert.equal(input.pendingFiles.value.length, 1);
  await input.submit();
  assert(prompts.at(-1).includes('[File: '));
  assert.equal(input.pendingFiles.value.length, 0);
  const before = calls.length;
  await pick([{...source,name:'photo.png',type:'image/png'}]);
  assert.equal(calls.length,before,'text-only model imported an image');
  assert(input.error.value.includes('поддержку изображений'));
  await pick([{...source,name:'document.pdf',type:'application/pdf'}]);
  assert.equal(calls.length,before,'unsupported PDF reached the import bridge');
  await pick([{...source,size:1024*1024+1}]);
  assert.equal(calls.length,before,'oversized text reached the import bridge');
  imageModel.value = true;
  await pick([{...source,name:'photo.png',type:'image/png'}]);
  assert.equal(input.pendingImages.value.length,1);
  imageModel.value = false;
  const submitted = prompts.length;
  await input.submit();
  assert.equal(prompts.length,submitted,'model switch submitted unsupported images');
  assert.equal(input.pendingImages.value.length,1,'model switch lost attachments');
  key('Escape'); key('Escape'); await flush();
  await pick([source]);
  key('Escape'); key('Escape'); await flush();
  assert.equal(input.pendingFiles.value.length,0);
  console.log('PASS file picker, capabilities, model switch, format and size validation, attachment-only send, command dispatch, no model calls for commands, argument validation, skills, keyboard selection, IME, draft preservation, and errors');
})().catch(error => { console.error(error); process.exitCode = 1; });
