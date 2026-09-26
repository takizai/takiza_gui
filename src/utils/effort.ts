/**
 * Helper to determine whether an LLM model supports the reasoning effort parameter
 * (e.g. OpenAI o1/o3, Claude 3.7 Sonnet Extended Thinking, DeepSeek R1, Gemini 2.5 thinking, etc.)
 */
export function supportsReasoningEffort(model?: string | null): boolean {
  if (!model) return false;
  const m = model.toLowerCase();
  return (
    m.includes("o1") ||
    m.includes("o3") ||
    m.includes("o4") ||
    m.includes("reasoning") ||
    m.includes("reasoner") ||
    m.includes("deepseek-r1") ||
    m.includes("claude-3-7") ||
    m.includes("claude-3.7") ||
    m.includes("thinking")
  );
}

export const REASONING_EFFORT_OPTIONS = [
  { value: "low", label: "Low", desc: "Быстрый ответ, минимальное время размышления" },
  { value: "medium", label: "Med", desc: "Сбалансированное рассуждение (по умолчанию)" },
  { value: "high", label: "High", desc: "Глубокий анализ и максимальное рассуждение" },
] as const;
