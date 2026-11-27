// @ts-check
// `prefers-reduced-motion` read once for every page: springs and glides resolve at once under it.

/** @type {MediaQueryList | null} */
const query = typeof matchMedia === "function" ? matchMedia("(prefers-reduced-motion: reduce)") : null;
// From change events, never `.matches` each time: Chromium's read precedes the change event, so it never hears it.
let on = query?.matches ?? false;
/** @type {Set<(on: boolean) => void>} */
const listeners = new Set();
query?.addEventListener("change", (event) => {
  on = event.matches;
  for (const listener of listeners) listener(on);
});

/** Never in a worker (no media queries). */
export const reduced = () => on;

/** @param {(on: boolean) => void} listener */
export function whenReduced(listener) {
  listeners.add(listener);
  return () => void listeners.delete(listener);
}
