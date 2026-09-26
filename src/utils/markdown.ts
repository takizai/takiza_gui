import MarkdownIt from "markdown-it";
import hljs from "highlight.js";

// Global helper for code block copy buttons inside rendered markdown
if (typeof window !== "undefined") {
  (window as any).__copyCode = (btn: HTMLElement) => {
    const code = btn.getAttribute("data-code");
    if (code) {
      navigator.clipboard.writeText(decodeURIComponent(code));
      const span = btn.querySelector("span") || btn;
      const oldText = span.textContent;
      span.textContent = "Copied!";
      setTimeout(() => {
        span.textContent = oldText;
      }, 1500);
    }
  };
}

function renderCodeCard(code: string, lang: string): string {
  // Strip excess leading/trailing empty lines
  const cleanCode = code.replace(/^\n+/, "").replace(/\n+$/, "");

  let highlighted = "";
  if (lang && hljs.getLanguage(lang)) {
    try {
      highlighted = hljs.highlight(cleanCode, { language: lang, ignoreIllegals: true }).value;
    } catch {
      highlighted = md.utils.escapeHtml(cleanCode);
    }
  } else {
    highlighted = md.utils.escapeHtml(cleanCode);
  }

  const displayLang = (lang || "code").toLowerCase();
  const encodedCode = encodeURIComponent(cleanCode);

  return `<div class="code-card-wrapper">
  <div class="code-card-header">
    <div class="flex items-center gap-2">
      <span class="w-1.5 h-1.5 rounded-full bg-amber-400/80"></span>
      <span class="font-mono text-[10px] uppercase tracking-wider text-zinc-400 font-semibold">${displayLang}</span>
    </div>
    <button class="copy-code-trigger flex items-center gap-1 px-2 py-0.5 rounded text-[10px] text-zinc-400 hover:text-zinc-100 hover:bg-white/[0.08] transition-colors cursor-pointer" data-code="${encodedCode}" onclick="window.__copyCode && window.__copyCode(this)">
      <span>Copy</span>
    </button>
  </div>
  <pre class="hljs"><code class="language-${displayLang}">${highlighted}</code></pre>
</div>`;
}

const md: InstanceType<typeof MarkdownIt> = new MarkdownIt({
  html: false,
  linkify: true,
  breaks: true,
});

// Override fence renderer to avoid markdown-it double wrapping in <pre><code>
md.renderer.rules.fence = (tokens, idx) => {
  const token = tokens[idx];
  const lang = token.info ? token.info.trim() : "";
  return renderCodeCard(token.content, lang);
};

// Override standard indented code blocks
md.renderer.rules.code_block = (tokens, idx) => {
  const token = tokens[idx];
  return renderCodeCard(token.content, "");
};

export function renderMarkdown(content: string): string {
  if (!content) return "";
  return md.render(content);
}
