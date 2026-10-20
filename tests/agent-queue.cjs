const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const ts = require('typescript');
const vue = require('vue');
const pinia = require('pinia');
const root = path.resolve(__dirname, '..');
const turns = [];
const cancellations = [];
let saved = { id: 'chat', created_at: '', model: 'test', messages: [], history: [] };
const clone = value => JSON.parse(JSON.stringify(value));
class Channel { onmessage() {} }
function invoke(name, args) {
  if (name === 'start_agent_turn') return new Promise(resolve => turns.push({ ...args, resolve }));
  if (name === 'get_current_session') return Promise.resolve(clone(saved));
  if (name === 'cancel_agent') { cancellations.push(name); return Promise.resolve(); }
  if (name === 'restore_checkpoint') { saved = { ...saved, history: [{ type: 'AssistantMessage', payload: 'before checkpoint' }] }; return Promise.resolve(clone(saved)); }
  if (name === 'new_session') { saved = { ...saved, id: 'new', history: [] }; return Promise.resolve(clone(saved)); }
  return Promise.resolve(null);
}
const modules = {};
function load(file) {
  if (modules[file]) return modules[file];
  const exports = {};
  const code = ts.transpileModule(fs.readFileSync(path.join(root, file), 'utf8'), { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020 } }).outputText;
  vm.runInNewContext(code, { exports, console, document: { documentElement: { setAttribute() {} } }, require(name) {
    if (name === 'vue') return vue;
    if (name === 'pinia') return pinia;
    if (name === '@tauri-apps/api/core') return { invoke, Channel };
    if (name.startsWith('../utils/')) return load(`src/utils/${name.split('/').pop()}.ts`);
    throw new Error(name);
  } }, { filename: file });
  return modules[file] = exports;
}
const tick = async () => { for (let i = 0; i < 12; i++) await Promise.resolve(); };
function finish(turn, text) {
  saved.history.push({ type: 'UserPrompt', payload: turn.input }, { type: 'AssistantMessage', payload: text });
  turn.channel.onmessage({ type: 'AssistantMessage', payload: text });
  turn.channel.onmessage({ type: 'Finished' });
}
pinia.setActivePinia(pinia.createPinia());
(async () => {
  const { hydrateSessionImages } = load('src/utils/attachments.ts');
  const historyFixture = hydrateSessionImages({ messages: [
    { role: 'user', content: 'first' }, { role: 'user', content: 'Continue.' },
    { role: 'user', content: 'with image', image_urls: ['image-fixture'] },
  ], history: [
    { type: 'MoaRouting', payload: { model: 'routed' } }, { type: 'UserPrompt', payload: 'first' },
    { type: 'AssistantMessage', payload: 'routed answer' },
    { type: 'UserPrompt', payload: '!echo direct' }, { type: 'UserPrompt', payload: 'with image' },
    { type: 'AssistantMessage', payload: 'manual answer' },
  ] });
  assert.equal(historyFixture.history[2].moaModel, 'routed');
  assert.equal(historyFixture.history[4].imageUrls[0], 'image-fixture');
  assert.equal(historyFixture.history[5].moaModel, undefined);
  const agent = load('src/stores/agentStore.ts').useAgentStore();
  agent.currentSession = clone(saved);
  const first = agent.sendPrompt('one');
  const stableThinking = agent.statusText;
  turns[0].channel.onmessage({ type: 'StatusUpdate', payload: 'Thinking...' });
  assert.equal(agent.statusText, stableThinking);
  await agent.sendPrompt('two', ['data:image/png;base64,fixture']);
  await agent.sendPrompt('three');
  assert.equal(turns.length, 1);
  assert.equal(agent.pendingPrompts.length, 2);
  turns[0].channel.onmessage({ type: 'ThoughtToken', payload: 'Check the source first.' });
  assert.equal(agent.currentStreamThought, 'Check the source first.');
  turns[0].channel.onmessage({ type: 'ToolStart', payload: { id: 'read', name: 'read_file', args: '{"path":"src/App.vue"}' } });
  assert.equal(agent.currentStreamThought, '');
  assert.equal(agent.currentSession.history[1].type, 'Thought');
  assert.equal(agent.activeToolCall.name, 'read_file');
  turns[0].channel.onmessage({ type: 'ToolLog', payload: 'Read 10 lines' });
  assert.equal(agent.streamingToolLogs[0], 'Read 10 lines');
  turns[0].channel.onmessage({ type: 'ToolEnd', payload: { id: 'read', name: 'read_file', args: '{"path":"src/App.vue"}', result: 'source', is_error: false } });
  assert.equal(agent.activeToolCall, null);
  assert.equal(agent.currentSession.history[2].type, 'ToolEnd');
  turns[0].channel.onmessage({ type: 'ThoughtToken', payload: 'Now prepare the answer.' });
  finish(turns[0], 'answer one');
  const visibleTypes = agent.currentSession.history.map(item => item.type);
  assert.equal(visibleTypes.join(','), 'UserPrompt,Thought,ToolEnd,Thought,AssistantMessage');
  assert.equal(agent.currentStreamThought, '', 'reasoning must commit before the final answer');
  await tick();
  assert(agent.isStreaming, 'Finished must wait for backend persistence');
  assert.equal(turns.length, 1);
  turns[0].resolve(); await first; await tick();
  assert.equal(turns.length, 2);
  assert.equal(turns[1].input, 'two');
  finish(turns[1], 'answer two'); turns[1].resolve(); await tick();
  assert.equal(turns[2].input, 'three');
  turns[2].channel.onmessage({ type: 'Error', payload: 'fixture failure' });
  await agent.sendPrompt('paused');
  turns[2].channel.onmessage({ type: 'Finished' }); turns[2].resolve(); await tick();
  assert(agent.queuePaused); assert.equal(turns.length, 3);
  agent.resumeQueue(); await tick(); assert.equal(turns[3].input, 'paused');
  await agent.sendPrompt('discard on stop');
  await agent.stop(); assert.equal(agent.pendingPrompts.length, 0);
  turns[3].channel.onmessage({ type: 'Interrupted' }); turns[3].channel.onmessage({ type: 'Finished' }); turns[3].resolve(); await tick();
  const fresh = agent.sendPrompt('fresh'); assert.equal(turns[4].input, 'fresh');
  const restore = agent.restoreCheckpoint('point'); await tick();
  assert(agent.operationBusy); assert(cancellations.length >= 2);
  turns[4].channel.onmessage({ type: 'AssistantMessage', payload: 'obsolete' });
  turns[4].channel.onmessage({ type: 'Interrupted' }); turns[4].resolve(); await fresh; await restore; await tick();
  assert.equal(agent.currentSession.history.length, 1);
  assert.equal(agent.currentSession.history[0].payload, 'before checkpoint');
  assert(!agent.queuePaused); assert(!agent.operationBusy);
  assert(agent.sessionHistoryVersion>0,'saved turns must notify the history list');
  const sessionId=agent.currentSession.id;
  turns[0].channel.onmessage({type:'SessionTitle',payload:{session_id:'old-session',title:'Old title'}});
  assert.notEqual(agent.currentSession.title,'Old title');
  turns[0].channel.onmessage({type:'SessionTitle',payload:{session_id:sessionId,title:'Название от модели'}});
  assert.equal(agent.currentSession.title,'Название от модели');
  assert.equal(agent.sessionTitleUpdated.title,'Название от модели');
  console.log('PASS FIFO, persistence ordering, stable thinking, pause/resume, cancellation, fresh prompt after stop, and restore without obsolete responses');
})().catch(error => { console.error(error); process.exitCode = 1; });
