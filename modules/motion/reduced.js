// @ts-check
// Reduced motion, once for every page: the reader's `prefers-reduced-motion`, read live. Springs and glides made under
// it resolve at once (physics.js: a spring at its end, a flick no glide), so a page's own motion asks only here.
//
//   import { reduced, whenReduced } from "/modules/motion/reduced.js";
//   if (reduced()) …;                         // now, as the reader has it
//   whenReduced((on) => …);                   // told as it changes (returns its stop)

/** @type {MediaQueryList | null} */
const query = typeof matchMedia === "function" ? matchMedia("(prefers-reduced-motion: reduce)") : null;
// (Kept from its change events, never read off the list each time: Chromium hands a listened list's change event to a
// `.matches` read made before it's dispatched, so a page reading it each frame never hears the change.)
let on = query?.matches ?? false;
/** @type {Set<(on: boolean) => void>} */
const listeners = new Set();
query?.addEventListener("change", (event) => {
  on = event.matches;
  for (const listener of listeners) listener(on);
});

/** Whether motion is reduced now (a worker, with no media queries: never). */
export const reduced = () => on;

/** Told whether motion is reduced each time that changes; returns its stop. @param {(on: boolean) => void} listener */
export function whenReduced(listener) {
  listeners.add(listener);
  return () => void listeners.delete(listener);
}
