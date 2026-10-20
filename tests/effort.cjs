const assert = require('node:assert/strict');
const {reasoningEfforts:levels} = require('./load-effort.cjs');
const result = (...args) => Array.from(levels(...args));
for(const name of ['foo1','my-reasoner','deepseek-r1','claude-3-7-sonnet','gpt-5-chat-latest','gpt-5-future']) assert.deepEqual(result(name),[],name);
assert.deepEqual(result('openai/o3-mini-2025-01-31'),['low','medium','high']);
assert.deepEqual(result('gpt-5-pro','openai_responses'),['high']);
assert.deepEqual(result('claude-opus-4-6'),['low','medium','high','max']);
assert.deepEqual(result('claude-opus-4-6','anthropic_messages'),['low','medium','high','max']);
assert.deepEqual(result('o3','chat_completions',{capabilities:{reasoning_effort:false}}),[]);
assert.deepEqual(result('o3','chat_completions',{supported_parameters:['temperature']}),[]);
assert.deepEqual(result('custom','chat_completions',{reasoning_efforts:['high','low','bogus']}),['low','high']);
assert.deepEqual(result('custom','openai_responses',{supported_parameters:['reasoning.effort']}),['low','medium','high']);
console.log('PASS model/protocol-specific effort levels, namespace/date aliases, explicit metadata overrides, no thinking-name inference');

for (const protocol of ['chat_completions','openai_responses','anthropic_messages']) {
 assert.deepEqual(result('cc/claude-opus-5-5',protocol),['low','medium','high','xhigh','max']);
 assert.deepEqual(result('anthropic/claude-opus-5.5',protocol),['low','medium','high','xhigh','max']);
 assert.deepEqual(result('o3',protocol),['low','medium','high']);
 assert.deepEqual(result('custom',protocol,{capabilities:{effort:{supported:true,low:{supported:true},high:{supported:false},max:{supported:true}}}}),['low','max']);
 assert.deepEqual(result('cc/claude-opus-5-5',protocol,{reasoning_effort:{supported:false}}),[]);
 assert.deepEqual(result('custom',protocol,{reasoning_effort:{supported:true,levels:['high','max']}}),['high','max']);
 assert.deepEqual(result('cc/claude-opus-5-5',protocol,{supported_parameters:['reasoning_effort']}),['low','medium','high','xhigh','max']);
}
