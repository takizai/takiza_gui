// Exercise the real sidebar handlers; only the native dialog and stores are mocked.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const ts = require('typescript');
const vue = require('vue');
const { parse, compileScript } = require('@vue/compiler-sfc');
const pending = [], switches = [];
let refreshed = 0, failSwitch = false;
const agent = vue.reactive({config:{workspace_dir:'/workspace/original'},operationBusy:false,
 async setWorkspaceDir(path) {
  if (failSwitch) throw new Error('Cannot open directory');
  switches.push(path);agent.config.workspace_dir=path;
 }
});
const sessions = {sessions:[],async fetchSessions(){refreshed++;}};
const descriptor = parse(fs.readFileSync('src/components/layout/Sidebar.vue','utf8')).descriptor;
const code = ts.transpileModule(compileScript(descriptor,{id:'workspace-test'}).content,
 {compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2020}}).outputText;
const exportsObject = {};
vm.runInNewContext(code,{exports:exportsObject,require(name){
 if(name==='vue')return vue;
 if(name==='@tauri-apps/plugin-dialog')return {open:options=>new Promise((resolve,reject)=>pending.push({options,resolve,reject}))};
 if(name.includes('/i18n'))return {useI18n:()=>({tr:text=>text})};
 if(name.includes('/agentStore'))return {useAgentStore:()=>agent};
 if(name.includes('/sessionStore'))return {useSessionStore:()=>sessions};
 if(name==='lucide-vue-next'||name.includes('/utils/window'))return {};
 throw new Error(name);
}});
const sidebar=exportsObject.default.setup({isOpen:true},{expose(){},emit(){}});
(async()=>{
 const cancelled=sidebar.switchWorkspace();
 assert.equal(sidebar.switching.value,true);
 assert.equal(pending[0].options.directory,true);
 assert.equal(pending[0].options.multiple,false);
 assert.equal(pending[0].options.defaultPath,'/workspace/original');
 await sidebar.switchWorkspace();assert.equal(pending.length,1,'duplicate native dialogs opened');
 pending[0].resolve(null);await cancelled;
 assert.equal(switches.length,0);assert.equal(refreshed,0);assert.equal(sidebar.switching.value,false);
 const selected=sidebar.switchWorkspace();pending[1].resolve('/workspace/new project');await selected;
 assert.equal(switches[0],'/workspace/new project');assert.equal(refreshed,1);
 const unchanged=sidebar.switchWorkspace();pending[2].resolve('/workspace/new project');await unchanged;
 assert.equal(switches.length,1,'choosing current project reset the workspace');
 const unavailable=sidebar.switchWorkspace();pending[3].reject(new Error('Dialog unavailable'));await unavailable;
 assert.equal(sidebar.switchError.value,'Error: Dialog unavailable');assert.equal(sidebar.switching.value,false);
 failSwitch=true;const failed=sidebar.switchWorkspace();pending[4].resolve('/missing');await failed;
 assert.equal(sidebar.switchError.value,'Error: Cannot open directory');
 assert.equal(agent.config.workspace_dir,'/workspace/new project');assert.equal(refreshed,1);
 agent.operationBusy=true;await sidebar.switchWorkspace();assert.equal(pending.length,5);
 console.log('PASS directory-only picker, current starting path, duplicate guard, cancellation, project refresh, same-directory guard and recoverable errors');
})().catch(error=>{console.error(error);process.exitCode=1;});
