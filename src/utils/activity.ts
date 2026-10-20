export const THINKING_MESSAGES = [
  "Thinking...", "Feeding the brain hamster...", "Connecting the dots...",
  "Warming up the neurons...", "Consulting the rubber duck...", "Untangling the spaghetti...",
  "Brewing a fresh idea...", "Herding the thought cats...",
] as const;
export function randomThinkingMessage(): string {
  return THINKING_MESSAGES[Math.floor(Math.random() * THINKING_MESSAGES.length)]!;
}
