const assert=require('node:assert/strict');const fs=require('node:fs');const path=require('node:path');const vm=require('node:vm');const ts=require('typescript');const vue=require('vue');
const {parse,compileScript}=require('@vue/compiler-sfc');
const root=path.resolve(__dirname,'..');const catalog=JSON.parse(fs.readFileSync(path.join(root,'src/locales/ui.json'),'utf8'));
const storage=new Map();const html={lang:''};const exportsModule={};
const code=ts.transpileModule(fs.readFileSync(path.join(root,'src/i18n.ts'),'utf8'),{compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2020,esModuleInterop:true}}).outputText;
vm.runInNewContext(code,{exports:exportsModule,document:{documentElement:html},localStorage:{getItem:key=>storage.get(key),setItem:(key,value)=>storage.set(key,value)},require(name){if(name==='vue')return vue;if(name==='./locales/ui.json')return catalog;throw new Error(name)}});
function files(dir){return fs.readdirSync(dir,{withFileTypes:true}).flatMap(item=>item.isDirectory()?files(path.join(dir,item.name)):[path.join(dir,item.name)]);}
const sourceKeys=new Set();
function inspect(source){const ast=ts.createSourceFile('ui.ts',source,ts.ScriptTarget.Latest,true);function visit(node){if(ts.isCallExpression(node)&&((ts.isIdentifier(node.expression)&&node.expression.text==='tr')||(ts.isPropertyAccessExpression(node.expression)&&node.expression.name.text==='tr')||(ts.isCallExpression(node.expression)&&node.expression.arguments.some(arg=>ts.isIdentifier(arg)&&arg.text==='tr')))){const arg=node.arguments[0];if(arg&&ts.isStringLiteral(arg))sourceKeys.add(arg.text);}ts.forEachChild(node,visit)}visit(ast);}
for(const file of files(path.join(root,'src')).filter(file=>file.endsWith('.vue'))){const descriptor=parse(fs.readFileSync(file,'utf8')).descriptor;inspect(compileScript(descriptor,{id:file,inlineTemplate:true}).content);}
inspect(fs.readFileSync(path.join(root,'src/composables/usePromptCommands.ts'),'utf8'));
for(const file of ['src/utils/models.ts','src/utils/promptCommands.ts','src/utils/effort.ts']) {
 const ast=ts.createSourceFile(file,fs.readFileSync(path.join(root,file),'utf8'),ts.ScriptTarget.Latest,true);
 function visit(node){if(ts.isPropertyAssignment(node)&&['name','description','desc'].includes(node.name.getText(ast))&&ts.isStringLiteral(node.initializer)&&/[А-Яа-яЁё]/.test(node.initializer.text))sourceKeys.add(node.initializer.text);ts.forEachChild(node,visit)}visit(ast);
}

for(const key of sourceKeys){assert(catalog[key],`Missing translations: ${key}`);assert.equal(catalog[key].length,2);for(const translated of catalog[key]){assert(translated.trim(),`Empty translation: ${key}`);assert.deepEqual((translated.match(/\{\d+\}/g)||[]).sort(),(key.match(/\{\d+\}/g)||[]).sort(),`Lost interpolation arguments: ${key}`);}}
(async()=>{
 const {tr,setLocale,normalizeLanguage,useI18n}=exportsModule;
 await setLocale('en');assert.equal(tr('Настройки'),'Settings');assert.equal(html.lang,'en');assert.equal(tr('Удалить диалог {0}',{0:'Original title'}),'Delete chat Original title');
 await setLocale('zh-CN');assert.equal(tr('Настройки'),'设置');assert.equal(html.lang,'zh-CN');assert.equal(storage.get('takiza_language'),'zh');
 assert.equal(tr('Original user content 原文'),'Original user content 原文');
 const pending=setLocale('en');await setLocale('ru');await pending;assert.equal(useI18n().locale.value,'ru','late language load replaced a newer selection');
 await setLocale('ru');assert.equal(tr('Настройки'),'Настройки');assert.equal(useI18n().locale.value,'ru');assert.equal(normalizeLanguage('English'),'en');assert.equal(normalizeLanguage('Chinese'),'zh');
 console.log(`PASS ${sourceKeys.size} UI translation keys, EN/ZH interpolation coverage, language persistence, aliases, and unknown text preservation`);
})().catch(error=>{console.error(error);process.exitCode=1});
