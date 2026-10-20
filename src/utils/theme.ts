import type { ThemeName } from "../types";

export const THEME_ACCENTS: Record<ThemeName, string> = {
  "dracula": "#bd93f9", "rose": "#ff8091", "ocean": "#50b4ff", "sakura": "#f5aad7", "solarized": "#dcb94b",
  amber: "#fbbf24", cyberpunk: "#ff2d95", emerald: "#34d399", nord: "#88c0d0", monochrome: "#e4e4e7",
};

export function normalizeTheme(value?: string | null): ThemeName {
  const name = value?.trim().toLowerCase() || "amber";
  return Object.prototype.hasOwnProperty.call(THEME_ACCENTS, name) ? name as ThemeName : "amber";
}

export const THEME_IDS = Object.keys(THEME_ACCENTS) as ThemeName[];
