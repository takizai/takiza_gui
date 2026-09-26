import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const zoomLevel = ref<number>(
  parseInt(localStorage.getItem("takiza_ui_zoom") || "100", 10)
);
const showZoomIndicator = ref(false);
let indicatorTimer: ReturnType<typeof setTimeout> | null = null;

export function useZoom() {
  async function applyZoom(val: number) {
    const clamped = Math.min(Math.max(val, 70), 160);
    zoomLevel.value = clamped;
    localStorage.setItem("takiza_ui_zoom", clamped.toString());

    // Reset any legacy CSS zoom that broke the viewport
    document.documentElement.style.zoom = "";
    document.body.style.zoom = "";

    // Call native Tauri webview zoom
    const scaleFactor = clamped / 100.0;
    try {
      await invoke("set_webview_zoom", { scaleFactor });
    } catch {
      // In browser fallback
    }

    // Trigger visual indicator
    showZoomIndicator.value = true;
    if (indicatorTimer) clearTimeout(indicatorTimer);
    indicatorTimer = setTimeout(() => {
      showZoomIndicator.value = false;
    }, 1200);
  }

  function zoomIn() {
    applyZoom(zoomLevel.value + 10);
  }

  function zoomOut() {
    applyZoom(zoomLevel.value - 10);
  }

  function resetZoom() {
    applyZoom(100);
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.ctrlKey || e.metaKey) {
      if (e.key === "=" || e.key === "+" || e.code === "NumpadAdd") {
        e.preventDefault();
        zoomIn();
      } else if (e.key === "-" || e.key === "_" || e.code === "NumpadSubtract") {
        e.preventDefault();
        zoomOut();
      } else if (e.key === "0" || e.code === "Numpad0") {
        e.preventDefault();
        resetZoom();
      }
    }
  }

  function handleWheel(e: WheelEvent) {
    if (e.ctrlKey || e.metaKey) {
      e.preventDefault();
      if (e.deltaY < 0) {
        zoomIn();
      } else if (e.deltaY > 0) {
        zoomOut();
      }
    }
  }

  function initZoom() {
    applyZoom(zoomLevel.value);
    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("wheel", handleWheel, { passive: false });
  }

  function destroyZoom() {
    window.removeEventListener("keydown", handleKeyDown);
    window.removeEventListener("wheel", handleWheel);
  }

  return {
    zoomLevel,
    showZoomIndicator,
    zoomIn,
    zoomOut,
    resetZoom,
    initZoom,
    destroyZoom,
  };
}
