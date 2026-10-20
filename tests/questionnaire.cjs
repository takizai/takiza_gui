const assert=require('node:assert/strict');const fs=require('node:fs');const vm=require('node:vm');const ts=require('typescript');const vue=require('vue');
const exportsObject={};vm.runInNewContext(ts.transpileModule(fs.readFileSync('src/composables/useQuestionnaire.ts','utf8'),{compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2020}}).outputText,{exports:exportsObject,require:name=>{assert.equal(name,'vue');return vue;}});
(async()=>{
 const payload=vue.ref({id:'grill',questions:[{question:'Storage?',options:['Local','Cloud']},{question:'Audience?',options:['Developers']}]});
 const responses=[];let finish;const scope=vue.effectScope();let panel;
 scope.run(()=>panel=exportsObject.useQuestionnaire(payload,response=>{responses.push(response);return new Promise(resolve=>finish=resolve);}));
 panel.toggleOption('Local');panel.customTextMap.value[0]='Encrypted';panel.nextQuestion();
 const pending=panel.skip();await panel.skip();assert.equal(responses.length,1,'duplicate cancellation submitted');assert.equal(responses[0].skipped,true);
 assert.equal(responses[0].answers[0].selected_options[0],'Local');assert.equal(responses[0].answers[0].custom_text,'Encrypted');assert.equal(responses[0].answers[1].selected_options.length,0);
 finish();await pending;assert.equal(panel.submitting.value,false);
 payload.value={id:'next-round',questions:[{question:'Scope?',options:['Small']}]};await vue.nextTick();assert.equal(panel.currentIndex.value,0);assert.equal(panel.answers.value[0].selected_options.length,0);
 scope.stop();console.log('PASS questionnaire cancellation preserves partial answers, guards duplicates and resets the next round');
})().catch(error=>{console.error(error);process.exitCode=1;});
