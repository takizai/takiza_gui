export const PROVIDERS = [
  { id: 'anymodel', name: 'AnyModel', url: 'https://anymodel.org/v1', model: '' },
  { id: 'openai', name: 'OpenAI', url: 'https://api.openai.com/v1', model: '' },
  { id: 'anthropic', name: 'Anthropic', url: 'https://api.anthropic.com/v1', model: '' },
  { id: 'google', name: 'Google AI', url: 'https://generativelanguage.googleapis.com/v1beta/openai', model: '' },
  { id: 'moonshot', name: 'Moonshot AI', url: 'https://api.moonshot.cn/v1', model: '' },
  { id: 'groq', name: 'Groq', url: 'https://api.groq.com/openai/v1', model: '' },
  { id: 'deepseek', name: 'DeepSeek', url: 'https://api.deepseek.com/v1', model: '' },
  { id: 'openrouter', name: 'OpenRouter', url: 'https://openrouter.ai/api/v1', model: '' },
  { id: 'ollama', name: 'Ollama', url: 'http://localhost:11434/v1', model: '' },
] as const;

export function providerForUrl(value = '') {
  const normalized = value.trim().replace(/\/+$/, '');
  return PROVIDERS.find(provider => provider.url === normalized);
}
