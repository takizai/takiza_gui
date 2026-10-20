const fs = require('node:fs');
const vm = require('node:vm');
const ts = require('typescript');
const exportsObject = {};
const code = ts.transpileModule(fs.readFileSync('src/utils/effort.ts', 'utf8'), {compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2020,esModuleInterop:true}}).outputText;
vm.runInNewContext(code, {exports:exportsObject,URL,require(name) {
 if (name.endsWith('reasoning-models.json')) return JSON.parse(fs.readFileSync('src/utils/reasoning-models.json','utf8'));
 throw new Error(name);
}});
module.exports = exportsObject;
