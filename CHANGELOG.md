# Changelog

## Unreleased

## 0.2.1

- Added a reasoning effort selector beside the prompt, opposite the model picker. Choices follow model capabilities, and unsupported models display “Not supported”.
- Fixed long-running LLM streams being cut off by the total request timeout. Completed Chat Completions responses no longer fail when a proxy closes the HTTP body without its final chunk; incomplete responses and truncated tool calls are rejected.
- Added a confirmation dialog before opening a project folder without a `.takiza` directory, explaining where conversations, attachments and checkpoints will be stored.
- Unified the main interface, dialogs and dropdowns on the system font, including Segoe UI on Windows.
- Pinned compatible Tauri JavaScript and Rust versions and added version checks before desktop packaging.

## 0.2.0

- Redesigned the chat, settings, skills and model picker with a minimal interface, smooth streaming, modal and history transitions, and Russian, English and Chinese localization. Added the public TUI themes to the GUI.
- Added provider profiles with independent credentials, model catalogs and native OpenAI Chat Completions, OpenAI Responses and Anthropic Messages connections. Reasoning effort choices follow model capabilities across protocols; models are never selected from a hardcoded default.
- Added model-aware file attachments, a native project directory picker, visible tool activity and reasoning, message editing and response regeneration without duplicate prompts.
- Added the built-in `grill_me` questionnaire tool and `/grill-me` command, with recommended answers, free text and cancellation.
- Shared project histories with the TUI, including legacy transcripts and preserved client metadata. Empty sessions are hidden, generated titles refresh immediately, and startup opens a fresh chat.
- Added explicit Windows installers, Linux packages and AppImage, and macOS bundles. Release publication waits for all platform builds to finish.

### Installation

- Windows x64: use `-setup.exe` for installation in your user account, or `.msi` for managed deployment. WebView2 is required; the installer can download it when absent.
- Linux x64: install `.deb` or `.rpm`, or make the `.AppImage` executable and run it without installation.
- macOS: open the DMG matching Apple Silicon or Intel, then drag Takiza to Applications.

These builds are not code signed or notarized; Windows and macOS may display security warnings. AppImage does not isolate configuration: GUI and TUI continue to share Takiza settings and project history.
