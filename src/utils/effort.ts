import rules from './reasoning-models.json';
import type { ApiType, ProviderModel, ReasoningEffort } from '../types';
const valid = ['none', 'minimal', 'low', 'medium', 'high', 'xhigh', 'max'];

/** Explicit provider capability data wins. Thinking support alone is insufficient. */
export function reasoningEfforts(model: string, _api: ApiType = 'chat_completions', metadata?: ProviderModel): ReasoningEffort[] {
  const reported = metadata?.reasoning_effort ?? metadata?.capabilities?.effort;
  if (reported && typeof reported === 'object') {
    if (reported.supported === false) return [];
    if (Array.isArray(reported.levels)) return valid.filter(level => reported.levels!.includes(level)) as ReasoningEffort[];
    const perLevel = valid.filter(level => {
      const support = reported[level];
      return support === true || (typeof support === 'object' && support !== null && !Array.isArray(support) && support.supported === true);
    });
    if (valid.some(level => level in reported)) return perLevel as ReasoningEffort[];
  }
  const capability = metadata?.reasoning_efforts ?? metadata?.capabilities?.reasoning_effort;
  if (Array.isArray(capability)) return valid.filter(level => capability.includes(level)) as ReasoningEffort[];
  if (capability === false) return [];
  if (metadata?.supported_parameters && !metadata.supported_parameters.some(p => ['reasoning_effort', 'reasoning.effort', 'output_config.effort'].includes(p))) return [];
  const explicitlySupported = capability === true || metadata?.supported_parameters?.some(p => ['reasoning_effort', 'reasoning.effort', 'output_config.effort'].includes(p));
  let name = model.toLowerCase().split('/').pop()!;
  if (name.startsWith('claude-')) name = name.replace(/\./g, '-');
  const match = rules.filter(rule => (name === rule.id || /^-\d{4}-?\d{2}-?\d{2}$/.test(name.slice(rule.id.length)) && name.startsWith(rule.id)))
    .sort((a,b) => b.id.length - a.id.length)[0];
  return (match?.levels || (explicitlySupported ? ['low', 'medium', 'high'] : [])) as ReasoningEffort[];
}
export function supportsReasoningEffort(model?: string | null, api: ApiType = 'chat_completions'): boolean {
  return !!model && reasoningEfforts(model, api).length > 1;
}
