# UI Design Specification: Takiza GUI (Rebuild)

## 1. Overview & Brand Identity
Takiza GUI is a modern, high-performance desktop interface for the Takiza autonomous Rust agent. It embodies the aesthetics of **Linear, Vercel, and Claude**:
- **Palette**: Deep Obsidian Canvas (`#090a0c`), Charcoal Surface (`#121316`), and crisp Silver-White typography and borders (`rgba(255, 255, 255, 0.08)` to `0.15`).
- **Signature Emblem**: Fluid ribbon geometry featuring pure white, platinum-silver, and warm amber loops.
- **Tone**: Editorial, focused, distraction-free, engineered for developers and technical creators.

---

## 2. Spatial Composition & Layout Architecture

### A. App Shell
- **Container**: Full-height application window (`h-screen overflow-hidden flex bg-[#090a0c] text-zinc-200 antialiased`).
- **Collapsible Sidebar (Left)**:
  - Width: `w-64` (expanded) or `w-0` / `hidden` (collapsed).
  - Header: Takiza brand lockup with 28px logo, app version, and collapse toggle button.
  - Action button: "+ New Session" with subtle silver border (`hover:border-white/20 active:scale-[0.98]`).
  - Session list: Scrollable history grouped by recency with delete/select interactions.
  - Footer: Model switcher indicator, connection status pill, and Settings gear icon.
- **Top Header Bar**:
  - Height: `h-12` (`border-b border-white/[0.06] bg-[#0d0e11]/80 backdrop-blur-md`).
  - Breadcrumb: Session title / active project path.
  - Right controls: Active provider/model badge, git branch indicator, sidebar toggle button.
- **Central Canvas (Main)**:
  - Max reading width: `max-w-4xl mx-auto w-full px-4 sm:px-6`.
  - Independent scroll container with custom slim scrollbar.
- **Floating Input Dock (Bottom)**:
  - Centered floating dock (`max-w-4xl mx-auto pb-5 px-4`).
  - Auto-resizing textarea with clean border (`rounded-2xl bg-[#14161b]/90 border border-white/10 shadow-2xl backdrop-blur-xl focus-within:border-white/25`).
  - Bottom action bar: Model badge, hotkey hint `Ctrl+Enter`, and Send / Stop button.

---

## 3. Brand & Logo Touchpoints
1. **Sidebar Brand Lockup**:
   - `src/assets/logo.png` rendered at `h-7 w-7 object-contain` alongside stylized "TAKIZA" typography.
2. **Hero Empty State (Welcome Canvas)**:
   - Centered 96x96px emblem with soft ambient glow (`shadow-[0_0_50px_rgba(255,255,255,0.06)]`).
   - Title: "What do you want to build today?"
   - Quick action prompt cards (e.g., "Analyze current repository", "Run tests and inspect failures", "Refactor module").
3. **Live Activity Indicator**:
   - When the agent is thinking or streaming tokens, a subtle breathing glow / micro-pulse effect appears next to the logo.

---

## 4. Message Flow & Cards

### User Message
- Compact, aligned to the right or full-width pill with distinct subtle surface (`bg-[#181a20] border border-white/10 text-zinc-100 rounded-2xl px-5 py-3.5`).

### Assistant Message
- Full-width, unboxed, airy typography (`prose-takiza text-zinc-200 leading-relaxed`).
- Header with miniature Takiza emblem and timestamp.
- Actions: Copy message markdown, rerun prompt.

### Thinking Accordion (`ThinkingBlock.vue`)
- Collapsible container with muted silver icon and elapsed timer (`0.8s`).
- Delicate border (`border-white/[0.08] bg-[#111215]/60 rounded-xl`).
- Expanding body showing chain-of-thought in dimmed text (`text-zinc-400 font-mono text-xs leading-relaxed`).

### Tool Execution Card (`ToolCallCard.vue`)
- Terminal-inspired card with status pill:
  - Running: Yellow/Amber pulse badge.
  - Completed: Emerald badge with exit code.
  - Failed: Rose badge.
- Header: Tool name (e.g., `run_command`, `write_to_file`) with parameters preview.
- Body: Monospace terminal block with stdout/stderr syntax highlighting and one-click copy button.

---

## 5. Modals & Settings Dialog (`SettingsModal.vue`)
- Backdrop: `bg-black/70 backdrop-blur-sm`.
- Container: `rounded-2xl border border-white/10 bg-[#121418] shadow-2xl max-w-2xl w-full`.
- Tabs:
  1. **General**: Working directory, theme, auto-scroll preferences.
  2. **Model & API Keys**: Provider selector (OpenAI / Anthropic / Gemini / DeepSeek / Ollama), API key inputs with reveal toggles, model selection.
  3. **Inherited Settings**: Visual confirmation of settings sourced from `~/.config/takiza/config.json` and `.env`.
- Clean footer with Cancel and Save buttons.

---

## 6. Design Tokens & Classes

| Token | Class / Value | Description |
| :--- | :--- | :--- |
| **Canvas** | `bg-[#090a0c]` | Primary deep black background |
| **Surface** | `bg-[#121316]` | Card and sidebar background |
| **Elevated** | `bg-[#181a20]` | Popovers, inputs, active items |
| **Border Subtly** | `border-white/[0.07]` | Hairline panel dividers |
| **Border Active** | `border-white/20` | Focus, hover states |
| **Text Primary** | `text-zinc-100` | Headings, primary content |
| **Text Muted** | `text-zinc-400` | Secondary descriptions, timestamps |
| **Text Dim** | `text-zinc-500` | Labels, hotkeys, status text |
| **Accent Glow** | `shadow-[0_0_24px_rgba(255,255,255,0.06)]` | Ambient focus lighting |
