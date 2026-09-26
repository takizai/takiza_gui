import { invoke } from "@tauri-apps/api/core";

export function handleWindowDrag(e: MouseEvent) {
  if (e.button !== 0) return;
  const target = e.target as HTMLElement | null;
  if (target && target.closest("button, input, textarea, a, select, [data-no-drag]")) {
    return;
  }
  invoke("start_dragging_window").catch(() => {});
}

export function handleWindowDblClick(e: MouseEvent) {
  const target = e.target as HTMLElement | null;
  if (target && target.closest("button, input, textarea, a, select, [data-no-drag]")) {
    return;
  }
  invoke("toggle_maximize_window").catch(() => {});
}
