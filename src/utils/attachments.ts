import type { Session } from "../types";
const FILE_MARKER = /\[File: (\{(?:"(?:\\.|[^"\\])*"|[^"{}])*\})\]/g;

export function attachmentMarkers(text: string): string[] {
  return [...(text.match(/\[Image: ("(?:\\.|[^"\\])*")\]/g) || []), ...(text.match(FILE_MARKER) || [])];
}

export function attachedFiles(text: string): { name: string; path: string }[] {
  const files: { name: string; path: string }[] = [];
  for (const match of text.matchAll(FILE_MARKER)) {
    try {
      const file = JSON.parse(match[1]!);
      if (typeof file.name === 'string' && typeof file.path === 'string') files.push(file);
    } catch { /* Ordinary text resembling a marker stays harmless. */ }
  }
  return files;
}

export function visiblePrompt(text: string): string {
  return text.replace(/\[Image: ("(?:\\.|[^"\\])*")\]\s*/g, "").replace(FILE_MARKER, "").trim();
}
export function hydrateSessionImages(session: Session): Session {
  const users = session.messages.filter(message => message.role === "user");
  let index = 0;
  let routedModel: string | undefined;
  let previousType: string | undefined;
  for (const item of session.history) {
    const previous = previousType;
    previousType = item.type;
    if (item.type === "UserPrompt") {
      if (previous !== "MoaRouting") routedModel = undefined;
      if (item.payload.startsWith("!")) continue;
      // The agent may insert continuation messages without visible user prompts.
      const match = users.findIndex((message, position) => position >= index && message.content === item.payload);
      if (match >= 0) { item.imageUrls = users[match].image_urls || []; index = match + 1; }
    } else if (item.type === "MoaRouting") routedModel = item.payload.model;
    else if (item.type === "AssistantMessage") item.moaModel = routedModel;
  }
  return session;
}
