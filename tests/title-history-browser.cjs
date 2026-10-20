const {chromium}=require(process.env.PLAYWRIGHT_MODULE||'playwright');const fs=require('node:fs');const path=require('node:path');const assert=require('node:assert/strict');
(async()=>{const browser=await chromium.launch({headless:true});try {
 const page=await browser.newPage({viewport:{width:1100,height:820}});const errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.route('http://takiza.test/**',route=>{const url=new URL(route.request().url());const file=path.join(process.cwd(),'dist',url.pathname==='/'?'index.html':url.pathname);return route.fulfill({contentType:file.endsWith('.js')?'application/javascript':file.endsWith('.css')?'text/css':'text/html',body:fs.readFileSync(file)});});
 await page.addInitScript(()=>{
  window.fixtureSession={id:'fresh-chat',created_at:'',model:'fixture',messages:[],history:[]};window.fixtureMetas=[{id:'previous',title:'Предыдущий диалог',message_count:2}];window.historyRequests=0;
  window.__TAURI_INTERNALS__={transformCallback:()=>1,metadata:{currentWindow:{label:'main'},currentWebview:{label:'main'}},invoke:async(cmd,args)=>{
   if(cmd==='get_config')return {model:'fixture',base_url:'https://fixture.invalid/v1',api_key:'fixture-key',providers:{},workspace_dir:'/project',theme:'nord',language:'ru',auto_approve:false};
   if(cmd==='get_current_session')return structuredClone(window.fixtureSession);
   if(cmd==='get_sessions'){window.historyRequests++;return structuredClone(window.fixtureMetas);}
   if(cmd==='get_available_skills'||cmd==='list_provider_models')return [];
   if(cmd==='get_git_info')return {branch:'main',is_dirty:false};
   if(cmd==='start_agent_turn'){
    window.fixtureChannel=args.channel;const history=[{type:'UserPrompt',payload:args.input},{type:'AssistantMessage',payload:'Ответ готов'}];
    window.fixtureSession={...window.fixtureSession,messages:[{role:'user',content:args.input},{role:'assistant',content:'Ответ готов'}],history};
    window.fixtureMetas.unshift({id:'fresh-chat',title:args.input,message_count:2});
    args.channel.onmessage({type:'AssistantToken',payload:'Ответ готов'});args.channel.onmessage({type:'Finished'});return;
   }
   return null;
  }};
 });
 await page.goto('http://takiza.test/');await page.getByRole('heading',{name:'С чего начнём?'}).waitFor();
 await page.getByRole('button',{name:'История диалогов',exact:true}).click();const history=page.getByRole('navigation',{name:'Сохранённые диалоги'});
 await history.getByRole('button',{name:'Предыдущий диалог',exact:true}).waitFor();
 const prompt=page.getByPlaceholder('Что нужно сделать?');await prompt.fill('Разработать редактор');await prompt.press('Enter');
 await history.getByRole('button',{name:'Разработать редактор',exact:true}).waitFor();
 const requests=await page.evaluate(()=>window.historyRequests);
 await page.evaluate(()=>{
  window.fixtureSession.title='Архитектура текстового редактора';window.fixtureMetas[0].title='Архитектура текстового редактора';
  window.fixtureChannel.onmessage({type:'SessionTitle',payload:{session_id:'fresh-chat',title:'Архитектура текстового редактора'}});
 });
 await history.getByRole('button',{name:'Архитектура текстового редактора',exact:true}).waitFor();assert.equal(await history.getByRole('button',{name:'Разработать редактор',exact:true}).count(),0);
 assert((await page.evaluate(()=>window.historyRequests))>requests);assert.deepEqual(errors,[]);
 console.log('PASS visible history refreshes after saved response and background title without closing the sidebar');
} finally {await browser.close();}})().catch(error=>{console.error(error);process.exitCode=1;});
