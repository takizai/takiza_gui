import type { ProviderModel } from '../types';

// Absence of metadata is not evidence of vision support, including gateway aliases.
export function supportsImageInput(model?: ProviderModel): boolean {
  if (!model) return false;
  const inputs = model.architecture?.input_modalities ?? model.input_modalities ?? model.modalities?.input;
  if (Array.isArray(inputs)) return inputs.includes('image');
  return model.capabilities?.vision === true || model.capabilities?.image_input === true;
}

export const TEXT_FILE_ACCEPT = 'text/*,.txt,.md,.mdx,.csv,.tsv,.json,.jsonl,.yaml,.yml,.toml,.xml,.html,.css,.scss,.js,.jsx,.ts,.tsx,.vue,.svelte,.rs,.py,.go,.java,.c,.h,.cpp,.hpp,.cs,.rb,.php,.sh,.sql,.log,.ini,.conf,.svg';
export const IMAGE_FILE_ACCEPT = 'image/png,image/jpeg,image/gif,image/webp';
export const MAX_TEXT_BYTES = 1024 * 1024;
export const MAX_IMAGE_BYTES = 10 * 1024 * 1024;
