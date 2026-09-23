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

### Сборка релизного приложения:
```bash
npm run tauri build -- --no-bundle
```
Бинарный исполняемый файл будет доступен в: `src-tauri/target/release/takiza-gui`.
