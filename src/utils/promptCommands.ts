import type { Skill } from "../types";

export const PROMPT_COMMANDS = [
  { name: "grill-me", description: "Разобрать план через интервью", argument: "тема" },
  { name: "stop", description: "Остановить ответ и очистить очередь" },
  { name: "rewind", description: "Вернуть файлы и чат к точке возврата" },
  { name: "restore", description: "Выбрать точку восстановления" },
  { name: "help", description: "Показать команды и горячие клавиши" },
  { name: "new", description: "Начать новый чат" },
  { name: "reset", description: "Начать чат с чистым контекстом" },
  { name: "settings", description: "Открыть настройки" },
  { name: "skills", description: "Открыть библиотеку скиллов" },
  { name: "model", description: "Выбрать модель", argument: "model-id" },
  { name: "theme", description: "Сменить тему", argument: "amber | cyberpunk | emerald | nord | monochrome | dracula | rose | ocean | sakura | solarized" },
  { name: "mode", description: "Режимы · в разработке", argument: "manual | moa" },
  { name: "effort", description: "Глубина рассуждений", argument: "model-dependent" },
  { name: "approval", description: "Настроить подтверждения действий", argument: "on | off" },
  { name: "provider", description: "Настроить провайдера и подключение" },
  { name: "usage", description: "Информация о провайдере" },
  { name: "status", description: "Показать модель, режим и рабочую папку" },
  { name: "diff", description: "Показать изменения файлов в Git" },
  { name: "history", description: "Показать сохранённые чаты" },
  { name: "resume", description: "Открыть сохранённый чат", argument: "session-id" },
] as const;

export type CommandName = typeof PROMPT_COMMANDS[number]["name"];
export interface Completion {
  id: string;
  label: string;
  description: string;
  kind: "command" | "skill" | "argument";
  replacement: string;
  start: number;
  end: number;
}

const argumentsByCommand: Record<string, string[]> = {
  theme: ["amber", "cyberpunk", "emerald", "nord", "monochrome", "dracula", "rose", "ocean", "sakura", "solarized"],
  mode: ["manual", "moa"], approval: ["on", "off"],
};

export function getPromptCompletions(text: string, cursor: number, skills: Skill[], efforts: readonly string[] = []): Completion[] {
  const before = text.slice(0, cursor);
  const argument = before.match(/^\s*\/(\w+)\s+([^\s]*)$/);
  const options = argument?.[1] === 'effort' ? (efforts.length ? [...efforts, 'auto'] : []) : argumentsByCommand[argument?.[1] || ''];
  if (argument && options) {
    return options.filter(value => value.startsWith(argument[2].toLowerCase())).map(value => ({
      id: `${argument[1]}:${value}`, label: value, description: `/${argument[1]} ${value}`,
      kind: "argument", replacement: value + " ", start: cursor - argument[2].length, end: cursor + (text.slice(cursor).match(/^[^\s]*/)?.[0].length || 0),
    }));
  }
  const token = before.match(/(?:^|\s)([@$/][^\s@$/]*)$/)?.[1];
  if (!token) return [];
  // Complete a token under the cursor, preserving the remainder of the draft.
  const start = cursor - token.length;
  const end = cursor + (text.slice(cursor).match(/^[^\s@$/]*/)?.[0].length || 0);
  const query = token.slice(1).toLowerCase();
  const atStart = !text.slice(0, start).trim();
  const commands: Completion[] = token[0] === "/" && atStart ? PROMPT_COMMANDS
    .filter(command => command.name.startsWith(query)).map(command => ({
      id: `command:${command.name}`, label: `/${command.name}`, description: command.description,
      kind: "command", replacement: `/${command.name} `, start, end,
    })) : [];
  const matches: Completion[] = skills.filter(skill => skill.name.toLowerCase().includes(query)
    && !(token[0] === "/" && atStart && PROMPT_COMMANDS.some(command => command.name === skill.name.toLowerCase())))
    .map(skill => ({
      id: `skill:${skill.name}`, label: `${token[0]}${skill.name}`, description: skill.description,
      kind: "skill", replacement: `@${skill.name} `, start, end,
    }));
  return [...commands, ...matches];
}

export type PromptInput = { kind: "prompt"; text: string } | { kind: "command"; name: CommandName; args: string }
  | { kind: "skill"; text: string } | { kind: "unknown"; name: string };

export function parsePromptInput(text: string, skills: Skill[]): PromptInput {
  const trimmed = text.trim();
  const match = trimmed.match(/^\/(\S+)(?:\s+([\s\S]*))?$/);
  if (!match) return { kind: "prompt", text: trimmed };
  const name = match[1].toLowerCase();
  if (name === "grill-me") return {kind:"prompt",text:`/grill-me${match[2]?.trim() ? ` ${match[2].trim()}` : ""}`};
  if (PROMPT_COMMANDS.some(command => command.name === name)) {
    return { kind: "command", name: name as CommandName, args: match[2]?.trim() || "" };
  }
  const skill = skills.find(skill => skill.name.toLowerCase() === name);
  if (skill) return { kind: "skill", text: `@${skill.name}${match[2] ? ` ${match[2]}` : " "}` };
  return { kind: "unknown", name: match[1] };
}
