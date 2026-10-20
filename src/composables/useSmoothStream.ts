import { onScopeDispose, ref, watch, type Ref } from "vue";

// A short display buffer absorbs uneven network chunks without delaying completion.
export function useSmoothStream(source: Readonly<Ref<string>>, active: Readonly<Ref<boolean>>) {
  const displayed = ref("");
  const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
  let frame = 0;
  let previousTime = 0;
  function stop() { cancelAnimationFrame(frame); frame = 0; previousTime = 0; }
  function advance(time: number) {
    frame = 0;
    const target = source.value;
    const elapsed = previousTime ? Math.min(40, time - previousTime) : 16;
    previousTime = time;
    if (!target.startsWith(displayed.value)) displayed.value = "";
    const backlog = target.length - displayed.value.length;
    let end = Math.min(target.length, displayed.value.length + Math.max(1, Math.ceil(backlog * elapsed / 45), backlog - 128));
    // Never render half of an emoji / surrogate pair.
    if (end < target.length && /[\uD800-\uDBFF]/.test(target[end - 1] || "")) end++;
    displayed.value = target.slice(0, end);
    if (end < target.length) frame = requestAnimationFrame(advance);
    else previousTime = 0;
  }
  function update() {
    if (!active.value || reducedMotion.matches || !source.value) { stop(); displayed.value = source.value; }
    else if (!frame) frame = requestAnimationFrame(advance);
  }
  watch([source, active], update, { immediate: true, flush: "sync" });
  reducedMotion.addEventListener("change", update);
  onScopeDispose(() => { stop(); reducedMotion.removeEventListener("change", update); });
  return displayed;
}
