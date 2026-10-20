import { tr } from "../i18n";
import { ref } from "vue";
import { useAgentStore } from "../stores/agentStore";
import { useSessionStore } from "../stores/sessionStore";
import { reasoningEfforts } from "../utils/effort";
import type { ThemeName } from "../types";
import { PROMPT_COMMANDS, type CommandName } from "../utils/promptCommands";

export function usePromptCommands(open: (panel: "settings" | "skills" | "usage" | "models") => void) {
  const agent = useAgentStore();
  const sessions = useSessionStore();
  const output = ref("");
  const error = ref("");
  const busy = ref(false);

  async function execute(name: CommandName, args: string): Promise<boolean> {
    if (busy.value) return false;
    output.value = "";
    error.value = "";
    busy.value = true;
    try {
      const argument = args.toLowerCase();
      switch (name) {
        case "stop": await agent.stop(); break;
        case "rewind": case "restore": await agent.openRestore(); break;
        case "help":
          output.value = PROMPT_COMMANDS.map(command => `/${command.name} — ${tr(command.description)}`).join("\n")
            + tr("\n\n $скилл, @скилл или /скилл — вставить скилл в запрос. ↑/↓ — выбор, Tab — дополнение, Esc — закрыть, Shift+Enter — новая строка. Двойной Esc — очистить ввод. Ctrl+O — развернуть вывод. !команда — выполнить в терминале.");
          break;
        case "new": case "reset":
          if (args) throw new Error(tr("/{0} не принимает аргументы.", {"0": name}));
          await agent.createNewSession();
          await sessions.fetchSessions();
          break;
        case "settings": case "provider": open("settings"); break;
        case "skills": await agent.fetchSkills(); open("skills"); break;
        case "usage": open("usage"); break;
        case "model":
          if (!args) { open("models"); break; }
          if (/\s/.test(args)) throw new Error(tr("Укажите один идентификатор модели: /model model-id."));
          await agent.savePreferences({ model: args });
          output.value = tr("Модель: {0}", {"0": agent.config?.model});
          break;
        case "theme":
          if (!args) { output.value = tr("Темы: amber, cyberpunk, emerald, nord, monochrome, dracula, rose, ocean, sakura, solarized"); break; }
          if (!["amber", "cyberpunk", "emerald", "nord", "monochrome", "dracula", "rose", "ocean", "sakura", "solarized"].includes(argument)) throw new Error(tr("Неизвестная тема."));
          await agent.setTheme(argument as ThemeName);
          break;
        case "mode":
          output.value = tr("Режимы Takiza (MoA) в разработке и будут доступны после появления входа в аккаунт. Сейчас используется ручной выбор модели.");
          break;
        case "effort":
          { const cfg = agent.config;
          const metadata = cfg?.providers?.[cfg.active_provider || '']?.models?.find(m => m.id === cfg.model);
          const levels = reasoningEfforts(cfg?.model || '', cfg?.api_type, metadata);
          if (!levels.length) throw new Error(tr('Модель не поддерживает смену effort'));
          if (!args) { output.value = `${tr('Глубина рассуждений')}: ${levels.includes(cfg?.effort as typeof levels[number]) ? cfg?.effort : tr('Автоматически')}. ${levels.join(', ')}`; break; }
          if (!levels.includes(argument as typeof levels[number]) && argument !== 'auto') throw new Error(`${tr('Глубина рассуждений')}: ${levels.join(', ')}, auto`);
          await agent.setEffort(argument === 'auto' ? '' : argument); }
          break;
        case "approval":
          if (!args) { output.value = tr("Подтверждения: {0}. /approval on — спрашивать, /approval off — разрешать автоматически.", {"0": agent.config?.auto_approve ? "off" : "on"}); break; }
          if (!["on", "off"].includes(argument)) throw new Error(tr("Используйте /approval on или /approval off."));
          await agent.savePreferences({ auto_approve: argument === "off" });
          break;
        case "status":
          await agent.refreshGit();
          output.value = tr("Модель: {0}\nРежим: {1}\nПапка: {2}\nВетка: {3}", {"0": agent.config?.model || "—", "1": agent.config?.mode || "manual", "2": agent.config?.workspace_dir || "—", "3": agent.gitStatus?.branch || tr("нет Git")});
          break;
        case "diff":
          await agent.refreshGit();
          output.value = agent.gitStatus?.diff || tr("Нет изменений в отслеживаемых файлах.");
          break;
        case "history":
          await sessions.fetchSessions();
          output.value = sessions.sessions.map(session => `${session.id} — ${session.title}`).join("\n") || tr("Нет сохранённых чатов.");
          break;
        case "resume":
          if (!args) throw new Error(tr("Укажите идентификатор чата: /resume session-id. Список доступен через /history."));
          if (!/^[a-zA-Z0-9_-]+$/.test(args)) throw new Error(tr("Некорректный идентификатор чата. Выберите его из /history."));
          await agent.loadSession(args);
          await sessions.fetchSessions();
          break;
      }
      return true;
    } catch (cause) {
      error.value = String(cause instanceof Error ? cause.message : cause);
      return false;
    } finally {
      busy.value = false;
    }
  }

  return { output, error, busy, execute };
}
