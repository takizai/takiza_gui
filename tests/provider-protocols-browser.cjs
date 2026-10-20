// Optional production UI regression. Set PLAYWRIGHT_MODULE to a Playwright install.
const {chromium}=require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const fs=require('node:fs');const path=require('node:path');const assert=require('node:assert/strict');
(async()=>{
 const browser=await chromium.launch({headless:true});const page=await browser.newPage({viewport:{width:1100,height:820}});
 const errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.route('http://takiza.test/**',route=>{const url=new URL(route.request().url());const file=path.join(process.cwd(),'dist',url.pathname==='/'?'index.html':url.pathname);return route.fulfill({contentType:file.endsWith('.js')?'application/javascript':file.endsWith('.css')?'text/css':'text/html',body:fs.readFileSync(file)});});
 await page.addInitScript(()=>{
  const profile={name:'CLI profile',base_url:'https://old.invalid/v1',model:'old-model',api_key:'fixture',api_type:'openai_responses',headers:{'x-tenant':'original'},models:[...Array.from({length:350},(_,i)=>({id:`fixture-${i}`,name:`Fixture ${i}`})),{id:'old-model'}]};
  window.fixtureConfig={...profile,provider_headers:profile.headers,api_key_env:null,active_provider:'original',providers:{original:profile},theme:'monochrome',workspace_dir:'/demo',language:'ru',auto_approve:false};window.fixtureCalls=[];
  window.__TAURI_INTERNALS__={transformCallback:()=>1,metadata:{currentWindow:{label:'main'},currentWebview:{label:'main'}},invoke:async(cmd,args)=>{
   window.fixtureCalls.push([cmd,args]);if(cmd==='get_config')return window.fixtureConfig;
   if(cmd==='get_sessions'||cmd==='get_available_skills')return [];
   if(cmd==='get_git_info')return {branch:'main',is_dirty:false};
   if(cmd==='list_provider_models')return args.apiType==='anthropic_messages'?[{id:'native-model',name:'Native model'}]:[{id:'old-model'}];
   if(cmd==='save_preferences'){
    const prefs=args.prefs;const id=prefs.new_provider_name?'new-account':window.fixtureConfig.active_provider;
    const saved={...window.fixtureConfig.providers[id],name:prefs.new_provider_name||window.fixtureConfig.providers[id].name,base_url:prefs.base_url,model:prefs.model,api_key:prefs.api_key,api_key_env:prefs.api_key_env||null,api_type:prefs.api_type,headers:prefs.headers,models:[]};
    window.fixtureConfig.providers={...window.fixtureConfig.providers,[id]:saved};window.fixtureConfig={...window.fixtureConfig,...saved,active_provider:id,provider_headers:saved.headers};
    if(saved.api_key_env)window.fixtureConfig.api_key='resolved-fixture-env-secret';
    return window.fixtureConfig;
   }
   return null;
  }};
 });
 await page.goto('http://takiza.test/');await page.getByRole('heading',{name:'С чего начнём?'}).waitFor();
 const settings=page.getByRole('dialog',{name:'Настройки'});
 const openSettings=async()=>{await page.getByRole('button',{name:'Действия и настройки'}).click();await page.getByRole('button',{name:'Настройки',exact:true}).click();await settings.waitFor();await settings.evaluate(async el=>{await Promise.all(el.getAnimations({subtree:true}).map(animation=>animation.finished));});};
 await openSettings();
 assert((await settings.evaluate(el=>getComputedStyle(el).fontFamily)).includes('Noto Sans'));
 await page.screenshot({path:'/tmp/takiza-settings-minimal.png'});
 const content=page.locator('.settings-content');const originalScroll=await content.evaluate(el=>el.scrollTop);
 const trigger=page.locator('#settings-model');const originalTrigger=await trigger.boundingBox();
 await trigger.click();await page.getByRole('combobox',{name:'Поиск в списке',exact:true}).waitFor();
 await page.waitForTimeout(180);
 const menu=page.locator('[data-dropdown-owner="settings-model"]');const initialMenu=await menu.boundingBox();
 assert.equal(await content.evaluate(el=>el.scrollTop),originalScroll,'opening selected last item scrolled modal');
 assert.deepEqual(await trigger.boundingBox(),originalTrigger);
 const search=page.getByRole('combobox',{name:'Поиск в списке',exact:true});
 for(const query of ['Fixture 34','does-not-exist','']) {
   await search.fill(query);await page.waitForTimeout(30);
   const bounds=await menu.boundingBox();for(const key of ['x','y','width','height'])assert(Math.abs(bounds[key]-initialMenu[key])<1,`popup jumped on ${query}: ${key}`);
   assert.equal(await content.evaluate(el=>el.scrollTop),originalScroll);
 }
 await search.press('ArrowUp');assert.equal(await content.evaluate(el=>el.scrollTop),originalScroll);
 await search.press('Escape');await menu.waitFor({state:'hidden'});await settings.waitFor();
 await page.getByRole('button',{name:'Ввести ID',exact:true}).click();await page.locator('#settings-model-id').fill('cc/claude-opus-5-5');
 await page.getByRole('button',{name:/Параметры подключения/}).click();
 for(const protocol of [/Chat Completions/,/OpenAI Responses/,/Anthropic Messages/]) {
   await page.locator('#provider-api-type').click();await page.getByRole('option',{name:protocol}).click();
   await page.getByRole('button',{name:'Агент',exact:true}).click();
   await page.locator('#reasoning-effort').click();
   const effort=page.locator('[data-dropdown-owner="reasoning-effort"]');
   assert.deepEqual(await effort.getByRole('option').allTextContents(),['Автоматически','low','medium','high','xhigh','max']);
   await effort.getByRole('option',{name:'max',exact:true}).click();
   await page.getByRole('button',{name:'Подключение',exact:true}).click();
 }
 await page.getByRole('button',{name:'Сбросить изменения',exact:true}).click();
 await page.getByRole('button',{name:/Параметры подключения/}).click();
 assert((await page.locator('#provider-api-type').innerText()).includes('OpenAI Responses'));
 await page.getByRole('button',{name:/HTTP-заголовки/}).click();assert.equal(await page.getByRole('textbox',{name:'Имя заголовка 1',exact:true}).inputValue(),'x-tenant');
 await page.getByRole('button',{name:'Добавить провайдера',exact:true}).click();
 await page.locator('#new-provider-name').fill('Native account');await page.locator('#provider-api-url').fill('https://native.invalid/v1');
 await page.locator('#provider-api-type').click();await page.getByRole('option',{name:/Anthropic Messages/}).click();
 await page.locator('#provider-auth-source').click();await page.getByRole('option',{name:'Переменная окружения',exact:true}).click();await page.locator('#provider-key-env').fill('NATIVE_TEST_KEY');
 assert.equal(await page.locator('#provider-api-key').count(),0);
 await page.getByRole('button',{name:'HTTP-заголовки',exact:true}).click();
 await page.getByRole('textbox',{name:'Имя заголовка 1',exact:true}).fill('Authorization');await page.locator('input[aria-label="Значение заголовка 1"]').fill('Custom fixture-token');
 assert.equal(await page.locator('input[aria-label="Значение заголовка 1"]').getAttribute('type'),'password');
 await page.getByRole('button',{name:'Загрузить модели провайдера',exact:true}).click();await page.locator('#settings-model').click();await page.getByRole('combobox',{name:'Поиск в списке',exact:true}).fill('Native');await page.getByRole('combobox',{name:'Поиск в списке',exact:true}).press('Enter');
 assert(await page.evaluate(()=>window.fixtureCalls.some(([cmd,args])=>cmd==='list_provider_models'&&args.apiType==='anthropic_messages'&&args.apiKey===''&&args.apiKeyEnv==='NATIVE_TEST_KEY'&&args.headers.Authorization==='Custom fixture-token')));
 await page.getByRole('button',{name:'Добавить и выбрать',exact:true}).click();await settings.waitFor({state:'hidden'});
 assert(await page.evaluate(()=>{const p=window.fixtureCalls.filter(([cmd])=>cmd==='save_preferences').at(-1)[1].prefs;return p.api_type==='anthropic_messages'&&p.api_key_env==='NATIVE_TEST_KEY'&&p.api_key==='';}));
 await openSettings();await page.getByRole('button',{name:/Параметры подключения/}).click();
 assert((await page.locator('#provider-api-type').innerText()).includes('Anthropic Messages'));assert.equal(await page.locator('#provider-key-env').inputValue(),'NATIVE_TEST_KEY');assert.equal(await page.locator('#provider-api-key').count(),0);
 await page.getByRole('button',{name:/HTTP-заголовки/}).click();assert.equal(await page.locator('input[aria-label="Значение заголовка 1"]').inputValue(),'Custom fixture-token');
 await page.locator('#provider-auth-source').click();await page.getByRole('option',{name:'Без API-ключа',exact:true}).click();
 await page.getByRole('button',{name:'Сохранить',exact:true}).click();await settings.waitFor({state:'hidden'});
 assert(await page.evaluate(()=>{const p=window.fixtureCalls.filter(([cmd])=>cmd==='save_preferences').at(-1)[1].prefs;return p.api_key===''&&p.api_key_env===''&&p.headers.Authorization==='Custom fixture-token';}));
 await openSettings();await page.getByRole('button',{name:/Параметры подключения/}).click();await page.setViewportSize({width:390,height:720});assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
 assert.deepEqual(errors,[]);await browser.close();console.log('PASS stable searchable dropdown with 351 models, Claude effort on all three protocols, production protocol selection, CLI inheritance, env auth, custom headers, catalog credentials, save/reopen, no-key mode and mobile layout');
})().catch(error=>{console.error(error);process.exit(1)});
