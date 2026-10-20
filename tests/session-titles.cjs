const assert=require('node:assert/strict');const fs=require('node:fs');const vm=require('node:vm');const ts=require('typescript');const vue=require('vue');const pinia=require('pinia');
const pending=[];const exportsObject={};
vm.runInNewContext(ts.transpileModule(fs.readFileSync('src/stores/sessionStore.ts','utf8'),{compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2020}}).outputText,{exports:exportsObject,console,require(name){
 if(name==='vue')return vue;if(name==='pinia')return pinia;
 if(name==='@tauri-apps/api/core')return {invoke:()=>new Promise(resolve=>pending.push(resolve))};
 if(name==='./agentStore')return {useAgentStore:()=>({})};throw new Error(name);
}});
(async()=>{pinia.setActivePinia(pinia.createPinia());const store=exportsObject.useSessionStore();const old=store.fetchSessions();const latest=store.fetchSessions();
 pending[1]([{id:'chat',title:'Название от модели'}]);await latest;
 pending[0]([{id:'chat',title:'first message'}]);await old;
 assert.equal(store.sessions[0].title,'Название от модели');assert.equal(store.isLoading,false);
 console.log('PASS late history fetch cannot replace a generated conversation title');
})().catch(error=>{console.error(error);process.exitCode=1;});
