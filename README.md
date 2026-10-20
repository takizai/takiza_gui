# Takiza GUI — AI Software Engineering Agent Desktop

Современное настольное приложение (Desktop GUI) для автономного AI-агента разработки **Takiza Code**, построенное на стеке **Tauri 2 + Vue 3 (Composition API) + Tailwind CSS** с высокопроизводительным ядром на **Rust**.

Вдохновлено **Cursor**, **Windsurf** и **Claude Desktop**.

---

## Возможности

- **Потоковый диалог и рассуждения (Thinking / Reasoning)**:
  - Мгновенный посимвольный стриминг ответа модели.
  - Сворачиваемый блок цепочки рассуждений модели (Reasoning tokens) с плавной анимацией.
- **Интерактивные карточки инструментов (Tool Calling)**:
  - `read_file`, `find_files`, `grep_search`, `list_dir` с превью аргументов и результатов.
  - `edit_file` и `write_file` с автоматическим визуальным **Diff-просмотрщиком** (строки с подсветкой вставок и удалений).
  - `run_command` со стримингом вывода терминала (stdout / stderr) в реальном времени.
- **Безопасность и подтверждение прав (Permission Requests)**:
  - В безопасном режиме агент перед выполнением потенциально деструктивных bash-команд запрашивает разрешение.
  - Выбор: «Разрешить разово», «Всегда разрешать» или «Отклонить».
  - Переключатель Safe Mode / Auto-Approve прямо в шапке приложения.
- **Управление сессиями и историей**:
  - Автосохранение истории сессий в `.takiza/sessions/`.
  - Сайдбар с быстрым переключением диалогов и созданием новых сессий.
- **Поддержка AI-провайдеров**:
  - Быстрые пресеты: **OpenRouter**, **OpenAI**, **DeepSeek**, **Groq**, **Ollama**.
  - Удобное окно настроек с управлением API-ключами, моделями, Base URL и Proxy.
- **Интеграция с Git**:
  - Отображение текущей ветки и статуса изменений (dirty/modified) в реальном времени.

---

## Архитектура

```
takiza-gui/
├── src/                          # Vue 3 Frontend (Vite + Tailwind CSS + Pinia)
│   ├── components/
│   │   ├── chat/                 # ChatContainer, ChatMessageItem, ToolCallCard, DiffViewer, ThinkingBlock
│   │   ├── layout/               # AppHeader, Sidebar
│   │   ├── input/                # PromptInput
│   │   └── modals/               # PermissionModal, SettingsModal
│   ├── stores/                   # agentStore (Tauri Channel стриминг), sessionStore
│   └── utils/                    # Markdown рендерер с подсветкой кода highlight.js
└── src-tauri/                    # Rust Core бэкенд (Tauri 2)
    ├── src/
    │   ├── commands.rs           # Tauri IPC команды (start_agent_turn, cancel, sessions, git)
    │   ├── state.rs              # Глобальное состояние AppState (Mutex<Agent>, CancellationToken)
    │   └── core/                 # Движок агента, портированный из takiza-harness:
    │       ├── agent.rs          # Цикл агента (Function Calling loop, CancellationToken)
    │       ├── tools.rs          # Выполнение инструментов (tokio async bash, filesystem)
    │       ├── llm.rs            # Стриминговый SSE клиент LLM
    │       ├── session.rs        # Сессии и экспорт в Markdown/JSON
    │       ├── git.rs            # Git branch, dirty status, diff
    │       └── config.rs         # Конфигурация и сохранение настроек
```

---

## Разработка и Запуск

### Запуск в режиме разработки:
```bash
npm run tauri dev
```

### Установка готового приложения

Скачайте сборку своей платформы из [GitHub Releases](https://github.com/takizai/takiza_gui/releases).

| Платформа | Основной вариант | Дополнительно |
| --- | --- | --- |
| Windows x64 | `-setup.exe`, установка для текущего пользователя | `.msi` для управляемой установки |
| Linux x64 | `.deb` или `.rpm` | `.AppImage` без установки |
| macOS Apple Silicon / Intel | `.dmg`, перенести Takiza в Applications | архив `.app` |

Windows требует WebView2; установщик загружает его при отсутствии. Linux требует
WebKitGTK 4.1; пакетный менеджер устанавливает зависимости deb/rpm.
AppImage требует совместимой системы и не является изолированным профилем:
настройки Takiza и история выбранного проекта остаются общими с TUI.
Сборки пока без подписи кода и notarization; Windows и macOS могут показывать предупреждения.

### Сборка установщиков

```bash
npm ci
npm run tauri -- build
```

Установщики находятся в `src-tauri/target/release/bundle/`.
Для сборки только исполняемого файла: `npm run tauri -- build --no-bundle`.
Сборка требует системных зависимостей Tauri для выбранной платформы.

## Версия сборки

Источник версии — `package.json`. Команды `npm run dev`, `npm run build` и
`npm run tauri -- build` синхронизируют оба lock-файла, Cargo-манифест и Tauri.
При прямом вызове Cargo сначала выполните `npm run version:sync`.

Release workflow собирает выбранный Git-тег и проверяет его совпадение с версией
в манифесте через `RELEASE_TAG`. Старые теги не понижают версию рабочей копии.
Релиз создаётся как черновик и публикуется только после тестов и всех сборок.

## Команды и скиллы в поле ввода

Введите `/`, чтобы выбрать команду, или `@`, чтобы выбрать скилл. Доступные
скиллы также появляются в меню `/`; `/имя-скилла` вставляет `@имя-скилла`
в запрос. После выбора допишите задачу и отправьте её. Стрелки ↑/↓ выбирают
подсказку, Tab дополняет ввод, Enter выбирает подсказку или выполняет полностью
введённую команду, Esc закрывает меню, Shift+Enter добавляет новую строку.

Команды выполняются локально, без запроса модели: `/help`, `/new`, `/reset`,
`/settings`, `/skills`, `/model [model-id]`, `/theme [название]`, `/mode [manual|moa]`,
`/effort [уровень|auto]`, `/approval [on|off]`, `/provider`, `/usage`, `/status`,
`/diff`, `/history`, `/resume session-id`. `/provider` открывает настройки подключения;
`/approval on` включает запрос подтверждений, `off` — автоматическое разрешение.
`/new` и `/reset` начинают новый чат, сохраняя предыдущую сессию. Результаты команд
и ошибки отображаются над полем ввода. Неизвестные команды не отправляются модели.

GUI использует `active_provider` и `providers` из конфига Takiza: адрес,
API-ключ (или `api_key_env`), модель и сохранённый каталог берутся из выбранного
профиля. Смена модели обновляет этот профиль; другие подключения сохраняются.
В `/provider` можно выбрать существующий профиль или нажать «Добавить провайдера»:
указать название, адрес API, ключ и модель, затем «Добавить и выбрать».
Модель загружается из API выбранного подключения или вводится вручную.
Новый профиль сохраняется в общий конфиг CLI/GUI; другие профили остаются на месте.
Старый конфиг с полями
`api_key`, `base_url`, `model` верхнего уровня также поддерживается.
Подключение поддерживает протоколы из Takiza CLI: OpenAI Chat Completions,
OpenAI Responses и Anthropic Messages. Протокол выбирается в блоке подключения
как при создании, так и при редактировании профиля. Стриминг, инструменты,
изображения, reasoning и заголовки чатов используют выбранный протокол.
Авторизация: ключ/токен, `api_key_env` (окружение процесса GUI) или без API-ключа;
произвольные `headers` заменяют стандартные без дублирования. В частности,
через `Authorization` можно передать Bearer, Basic или готовый access token.
Для полностью собственной авторизации выберите «Без API-ключа» и задайте заголовки.
При выборе env в конфиг записывается только имя переменной, а не её значение.
Это авторизация LLM-провайдера; браузерный OAuth для MCP из CLI сюда не относится.
MoA пока в разработке; `/usage` показывает сведения о текущем провайдере, адресе API и модели.

Проверка команд и клавиатурных подсказок: `npm run test:prompt`.


### Composer and agent controls

- Keep typing while the agent works: Enter adds the next prompt to a FIFO queue.
  Stop clears the queue; a failed turn pauses remaining prompts until Resume.
- `/rewind` and `/restore` show checkpoints with a file preview. Restoring replaces
  both files and chat, including reasoning and tool responses. A safety checkpoint
  lets you undo the restore. Chats and attachments remain in `.takiza`.
- Paste or drop PNG, JPEG, GIF and WebP images to attach them with previews. Other
  files are inserted as paths. Image attachments survive session reloads.
- `!command` runs a shell command in the workspace without calling the model.
  `/stop`, `/new`, `/resume` and restore commands also work during a turn.
- Double Escape clears the draft and its attachments. Ctrl+O (Cmd+O on macOS)
  expands tool output and errors; long output is compact by default.
- Select skills with `$name`, the existing `@name` alias, or a Markdown link to
  SKILL.md. Nested local and global `.agents`, `.codex` and `.claude` roots and
  `CODEX_HOME` are supported.
- The agent defaults to 100 steps. Set the limit in Settings or use
  `TAKIZA_MAX_STEPS`; `0` removes the limit. Empty answers are retried up to three
  attempts. The English thinking phrase stays fixed for each turn.

Validation: `npm test`, `npm run build`, and `npm run test:core`. The last command
checks real Rust core modules and command handlers with a small State/Channel IPC
mock, so it works without Linux WebKit development packages. Its HTTP fixtures
listen only on localhost and use no API credentials. A full desktop build still
requires the normal Tauri system dependencies.


### Web search

The agent has a `web_search` tool in its normal tool loop. It searches when asked
explicitly or when information is unfamiliar, uncertain or likely outdated.
Arguments: `query` (required) and `num_results` (optional, 1–10, default 5).
Results contain titles, source URLs and short snippets, not full page contents.
The model is instructed to cite source URLs and report failed verification honestly.

Search appears as ordinary command/tool execution, with compact output and the
usual expansion control. Stop cancels in-flight requests. Requests have a 20-second
outer deadline, a 15-second HTTP timeout and a 1 MiB response limit. Search uses
the configured proxy and never sends the model API key to a search provider.

Set `TAVILY_API_KEY` in the environment or loaded `.env` to use Tavily Search:

```dotenv
TAVILY_API_KEY=tvly-your-key
```

Create a key at https://app.tavily.com/ and restart Takiza after configuring it.
Tavily uses `search_depth=basic` and disables automatic parameter selection,
so each search consumes one API credit. Results include titles, URLs and snippets;
Takiza does not request generated answers, images or raw page contents.

Provider priority is Tavily, then `BRAVE_SEARCH_API_KEY` (Brave Search API), then
Bing RSS when neither key is configured. Failed authenticated searches are reported
without silently switching providers. Provider availability, limits and applicable usage terms
still apply; provider errors appear as ordinary tool failures.

### Язык интерфейса и темы

В настройках → «Темы» доступны русский, English и 简体中文. Выбор применяется
сразу и сохраняется в поле `language` общего конфига Takiza. Содержимое чата,
reasoning, вывод инструментов и документация скиллов остаются на исходном языке.
Поддерживаются десять тем: Amber, Cyberpunk, Emerald, Nord, Monochrome,
Dracula, Rose, Ocean, Sakura и Solarized.

Библиотека скиллов поддерживает поиск, просмотр документации и вставку в чат.
На узком экране список и документ открываются по отдельности; кнопка назад
возвращает к результатам поиска. Проверки локализаций: `npm run test:i18n`.

Глубина рассуждений зависит от выбранной модели, независимо от протокола подключения.
Каталог `GET {base_url}/models` может сообщить точные уровни через `reasoning_efforts`,
`reasoning_effort.levels` или `capabilities.effort`; явный запрет провайдера имеет приоритет.
Если таких данных нет, используется таблица документированных возможностей моделей
в `src/utils/reasoning-models.json`. Неизвестные модели не получают предположений по
словам thinking/reasoner. Claude Opus 5.5, включая `cc/claude-opus-5-5` у AnyModel,
доступны low/medium/high/xhigh/max. `auto` оставляет выбор провайдеру.
Chat Completions отправляет `reasoning_effort`, Responses — `reasoning.effort`,
Anthropic Messages — `output_config.effort`. Неподдерживаемое или устаревшее значение
при смене модели не попадает в запрос.
Источники: [AnyModel](https://anymodel.org/ru/blog/reasoning-effort-settings-when-high-vs-low-pays-off),
[Anthropic](https://platform.claude.com/docs/en/build-with-claude/effort),
[OpenAI](https://developers.openai.com/api/docs/guides/reasoning).

Обычный запуск GUI открывает новый пустой чат. Сохранённые сессии открываются явно через историю; пустые чаты при запуске не записываются на диск.

`/grill-me [тема]` запускает встроенное интервью для проверки плана или архитектуры.
Без темы агент уточнит её или использует обсуждаемый план. Доступны рекомендуемые
ответы, альтернативы, свой ввод и отмена с сохранением уже данных ответов.
Установка скилла не требуется.

После ответа GUI генерирует короткое название диалога выбранной моделью через
активного провайдера. Генерация идёт в фоне; готовое название сохраняется в сессии
и обновляет историю без перезапуска. Существующие названия сохраняются.

GUI и CLI читают общую `.takiza/sessions` выбранного проекта: поддерживаются старый
формат истории CLI и общий формат `{type,payload}`. Пустые записи и повреждённые
файлы исключаются из списков без удаления. Метаданные другого клиента сохраняются.
История GUI обновляется после сохранения ответа, получения названия, открытия
панели и возвращения фокуса в приложение.
