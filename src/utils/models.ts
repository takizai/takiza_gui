import type { CuratedModel } from "../types";

export const CURATED_MODELS: CuratedModel[] = [
  {
    provider: "OpenAI",
    id: "cx/gpt-6-astra",
    name: "GPT-6 Astra",
    description: "OpenAI флагман для комплексной сквозной разработки, анализа и глубокого кодинга",
    is_expensive: true,
  },
  {
    provider: "OpenAI",
    id: "cx/gpt-5.6-sol",
    name: "GPT-5.6 Sol",
    description: "Высокоуровневые рассуждения и сложная STEM-архитектура с длинным контекстом",
    is_expensive: false,
  },
  {
    provider: "Anthropic",
    id: "cc/claude-opus-5",
    name: "Claude Opus 5",
    description: "Флагман Anthropic для продолжительных агентных задач и full-stack инженерии",
    is_expensive: false,
  },
  {
    provider: "Anthropic",
    id: "cc/claude-sonnet-5",
    name: "Claude Sonnet 5",
    description: "Фронтирная производительность для автономной разработки, рефакторинга и верстки",
    is_expensive: false,
  },
  {
    provider: "Google",
    id: "ag/gemini-3.7-flash-high",
    name: "Gemini 3.7 Flash High",
    description: "Высокоскоростная мультимодальная модель Google с глубоким оркестрированием тулов",
    is_expensive: false,
  },
  {
    provider: "Google",
    id: "ag/gemini-3.1-pro-low",
    name: "Gemini 3.1 Pro Low",
    description: "Сбалансированные рассуждения и анализ сложных кодовых баз при экономичном расходе",
    is_expensive: false,
  },
  {
    provider: "Moonshot AI",
    id: "kmc/k3",
    name: "Kimi K3",
    description: "Мультимодальные рассуждения с контекстом 256k и полной поддержкой инструментов",
    is_expensive: false,
  },
  {
    provider: "xAI",
    id: "xai/grok-4.7",
    name: "Grok 4.7",
    description: "Фронтирная reasoning-модель xAI для аудита безопасности, реверс-инжиниринга и Python",
    is_expensive: false,
  },
  {
    provider: "OpenAI",
    id: "cx/gpt-5.6-luna",
    name: "GPT-5.6 Luna",
    description: "Новейшая высокоскоростная reasoning-модель OpenAI для сложного кода и логики",
    is_expensive: false,
  },
  {
    provider: "Moonshot AI",
    id: "kmc/kimi-for-coding",
    name: "Kimi for Coding",
    description: "Специализированный чемпион генерации кода, поиска багов и анализа интерфейсов",
    is_expensive: false,
  },
];

export const THEME_OPTIONS = [
  {
    id: "amber",
    name: "Amber (Takiza Gold)",
    desc: "Фирменный теплый янтарно-золотой акцент Takiza с глубоким контрастом",
    color: "#f59e0b",
    border: "#00c8dc",
  },
  {
    id: "cyberpunk",
    name: "Cyberpunk (Neon Pink & Cyan)",
    desc: "Энергичный неоновый стиль с акцентами мадженты и электрического циана",
    color: "#ff2d95",
    border: "#00f0ff",
  },
  {
    id: "emerald",
    name: "Emerald (Matrix Green)",
    desc: "Классический хакерский терминал с высококонтрастным зеленым фосфором",
    color: "#10b981",
    border: "#059669",
  },
  {
    id: "nord",
    name: "Nord (Frost Arctic Blue)",
    desc: "Спокойная скандинавская палитра с прохладными морозными оттенками циана",
    color: "#38bdf8",
    border: "#60a5fa",
  },
  {
    id: "monochrome",
    name: "Monochrome (Minimalist White)",
    desc: "Чистый минимализм в оттенках серебра и белого без отвлекающих цветов",
    color: "#f4f4f5",
    border: "#71717a",
  },
] as const;
