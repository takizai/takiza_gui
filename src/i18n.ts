import { ref } from "vue";
import type { Language } from "./types";
let messages: Record<string, readonly string[]> = {};
let catalogLoading: Promise<void> | undefined;
let localeRequest = 0;
export function normalizeLanguage(value?: string | null): Language {
  const name = value?.trim().toLowerCase();
  if (name === "en" || name === "english") return "en";
  if (name === "zh" || name === "zh-cn" || name === "chinese") return "zh";
  return "ru";
}
function storedLanguage(): Language {
  try { return normalizeLanguage(localStorage.getItem("takiza_language")); } catch { return "ru"; }
}
const locale = ref<Language>(storedLanguage());
export async function setLocale(value: string) {
  const language = normalizeLanguage(value);
  const request = ++localeRequest;
  if (language !== "ru") {
    catalogLoading ||= import("./locales/ui.json").then(module => { messages = module.default; }).catch(error => { catalogLoading = undefined; throw error; });
    await catalogLoading;
  }
  if (request !== localeRequest) return;
  locale.value = language;
  document.documentElement.lang = locale.value === "zh" ? "zh-CN" : locale.value;
  try { localStorage.setItem("takiza_language", locale.value); } catch { /* Storage can be unavailable. */ }
}
export function tr(source: string, params: Record<string, unknown> = {}): string {
  const text = locale.value === "ru" ? source : messages[source]?.[locale.value === "en" ? 0 : 1] ?? source;
  return text.replace(/\{(\w+)\}/g, (match, key: string) => params[key] === undefined ? match : String(params[key]));
}
export function useI18n() { return { tr, locale, setLocale }; }
