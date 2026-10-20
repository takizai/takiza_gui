const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const ts = require('typescript');
const vue = require('vue');
const pinia = require('pinia');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const calls = [];
const turns = [];
let saved = { id: 'chat', messages: [{role:'user',content:'Original [Image: "photo.png"]',image_urls:['fixture-image']},{role:'assistant',content:'Old answer'}], history: [{type:'UserPrompt',payload:'Original [Image: "photo.png"]',imageUrls:['fixture-image']},{type:'Thought',payload:'Old reasoning'},{type:'AssistantMessage',payload:'Old answer'}] };
const clone = value => JSON.parse(JSON.stringify(value));
class Channel { onmessage() {} }
function invoke(command, args) {
  calls.push([command,args]);
  if (command === 'regenerate_agent_turn') return new Promise((resolve,reject) => turns.push({...args,resolve,reject}));
  if (command === 'get_current_session') return Promise.resolve(clone(saved));
  return Promise.resolve(null);
}
const modules = {};
function load(file) {
 if(modules[file]) return modules[file];
 const exports={};
 const code=ts.transpileModule(fs.readFileSync(path.join(root,file),'utf8'),{compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2020}}).outputText;
 vm.runInNewContext(code,{exports,console:{...console,error(){}},document:{documentElement:{setAttribute(){}}},require(name){
  if(name==='vue')return vue;if(name==='pinia')return pinia;if(name==='@tauri-apps/api/core')return {invoke,Channel};
  if(name.startsWith('../utils/'))return load(`src/utils/${name.split('/').pop()}.ts`);
  throw new Error(name);
 }});
 return modules[file]=exports;
}
function finish(agent,turn,text){
 saved={...clone(agent.currentSession),messages:[{role:'user',content:agent.currentSession.history[0].payload,image_urls:['fixture-image']},{role:'assistant',content:text}]};
 saved.history.push({type:'AssistantMessage',payload:text});
 turn.channel.onmessage({type:'AssistantMessage',payload:text});turn.channel.onmessage({type:'Finished'});turn.resolve();
}
(async()=>{
 pinia.setActivePinia(pinia.createPinia());const agent=load('src/stores/agentStore.ts').useAgentStore();agent.currentSession=clone(saved);
 const retry=agent.regenerateMessage(2);
 assert.equal(turns.length,1);assert.equal(turns[0].historyIndex,0);assert.equal(turns[0].edited,null);
 assert.equal(agent.currentSession.history.length,1);assert.equal(agent.currentSession.history[0].payload,'Original [Image: "photo.png"]');
 assert.equal(agent.isStreaming,true);
 await assert.rejects(agent.regenerateMessage(0),/завершения/);
 finish(agent,turns[0],'New answer');await retry;
 assert.equal(agent.currentSession.history.filter(item=>item.type==='UserPrompt').length,1);
 assert.equal(agent.currentSession.history[1].payload,'New answer');
 const edit=agent.regenerateMessage(0,'Edited question');assert.equal(turns[1].edited,'Edited question\n[Image: "photo.png"]');
 assert.equal(agent.currentSession.history[0].imageUrls[0],'fixture-image');finish(agent,turns[1],'Edited answer');await edit;
 saved.history[0].payload += '\n[File: {"name":"code.ts","path":"/cached/code.ts"}]';
 agent.currentSession=clone(saved);
 const fileEdit=agent.regenerateMessage(0,'Keep file');
 assert(turns[2].edited.includes('[File: {"name":"code.ts","path":"/cached/code.ts"}]'));
 finish(agent,turns[2],'Edited answer');await fileEdit;
 const before=clone(saved);const failed=agent.regenerateMessage(1);turns[3].reject(new Error('Fixture failure'));await failed;
 assert.equal(agent.currentSession.history[0].payload,before.history[0].payload);assert.equal(agent.currentSession.history[1].payload,'Edited answer');
 assert.equal(agent.currentSession.history.at(-1).type,'Error');assert.equal(agent.isStreaming,false);
 assert.equal(calls.filter(([name])=>name==='start_agent_turn').length,0,'regeneration submitted a new prompt');
 console.log('PASS regeneration without duplicate prompts, original history indices, edit with attachments, busy guard, and failure recovery');
})().catch(error=>{console.error(error);process.exitCode=1});
