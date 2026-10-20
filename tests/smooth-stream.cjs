const assert=require('node:assert/strict');
const fs=require('node:fs');
const vm=require('node:vm');
const ts=require('typescript');
const vue=require('vue');
const callbacks=new Map();let next=0;let time=0;
const media={matches:false,addEventListener(){},removeEventListener(){}};
const exportsModule={};
vm.runInNewContext(ts.transpileModule(fs.readFileSync('src/composables/useSmoothStream.ts','utf8'),{compilerOptions:{module:ts.ModuleKind.CommonJS}}).outputText,{
 exports:exportsModule,window:{matchMedia:()=>media},requestAnimationFrame:fn=>{callbacks.set(++next,fn);return next},cancelAnimationFrame:id=>callbacks.delete(id),require:name=>{if(name==='vue')return vue;throw new Error(name)}
});
const source=vue.ref('');const active=vue.ref(true);const scope=vue.effectScope();let displayed;
scope.run(()=>displayed=exportsModule.useSmoothStream(source,active));
function frame(){time+=16;const queued=[...callbacks.values()];callbacks.clear();queued.forEach(fn=>fn(time));}
source.value='😀Плавный потоковый ответ без потерь';assert.equal(displayed.value,'');frame();assert(displayed.value.length>0&&displayed.value.length<source.value.length);
for(let i=0;i<40;i++){frame();assert(!/[\uD800-\uDBFF]$/.test(displayed.value),'partial emoji rendered');}
assert.equal(displayed.value,source.value);
source.value+=' и продолжение';frame();assert(source.value.startsWith(displayed.value));
active.value=false;assert.equal(displayed.value,source.value);assert.equal(callbacks.size,0);
source.value='';assert.equal(displayed.value,'');active.value=true;media.matches=true;source.value='Reduced motion';assert.equal(displayed.value,source.value);
media.matches=false;source.value+=' pending';assert(callbacks.size>0);scope.stop();assert.equal(callbacks.size,0);
console.log('PASS progressive stream, exact completion, Unicode, reduced motion, reset, and cleanup');
