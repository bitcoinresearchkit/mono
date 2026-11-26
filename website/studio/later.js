// @ts-check
// What a first chart doesn't run (rest.js, rest.css): loaded once it has drawn, at a first press or key, or at once where
// it's what shows (the home, a heatmap). Each module here once it's in; what needs one before then asks withRest.
/** @typedef {typeof import("/studio/rest.js")} Rest */
/** @type {Rest["command"]} */
export let command;
/** @type {Rest["pen"]} */
export let pen;
/** @type {Rest["grid"]} */
export let grid;
/** @type {Rest["heatmaps"]} */
export let heatmaps;
/** @type {Rest["inspector"]} */
export let inspector;
/** @type {Rest["menus"]} */
export let menus;
/** @type {Rest["presetsEdit"]} */
export let presetsEdit;
/** @type {Rest["gestures"]} */
export let gestures;
/** @type {Rest["xkcd"]} */
export let xkcd;
/** In once the rest is; what waits on it runs in the order it asked. @type {PromiseWithResolvers<void>} */
const restIn = Promise.withResolvers();
/** @type {Promise<void> | undefined} */
let loading;
/** `run` once the rest is in: now, if it is. @param {() => void} run */
export const withRest = (run) => (command ? run() : void restIn.promise.then(run));
/** The rest, once. Failed (the server lost), what was held is let go of, asked again at the next press; Home says so. */
export const loadRest = () => (loading ??= Promise.all([import("/studio/rest.js"), restSheet()]).then(([rest]) => {
  ({ command, pen, grid, heatmaps, inspector, menus, presetsEdit, gestures, xkcd } = rest);
  restIn.resolve();
}, (error) => {
  loading = undefined;
  throw error;
}));
/** rest.css, in before anything it styles shows. @returns {Promise<unknown>} */
const restSheet = () => new Promise((resolve, reject) => {
  const link = Object.assign(document.createElement("link"), { rel: "stylesheet", href: import.meta.resolve("/studio/rest.css") });
  link.onload = resolve;
  link.onerror = () => (link.remove(), reject(new Error("/studio/rest.css did not load")));
  document.head.append(link);
});
/** The rest asked for, not waited on (a first chart drawn, a first press). */
export const askRest = () => void loadRest().catch(() => {});
// Till the rest is in, what it answers waits for it and is replayed in order: a click, a key, Back and Forward, a link
// in the address, another tab's save. A press on the chart (a pan, a pinch) and a key's own (a character, Tab) aren't held.
const HELD = ["click", "dblclick", "contextmenu", "keydown", "popstate", "hashchange", "storage"];
/** @param {Event} event */
function hold(event) {
  if (command) return;
  askRest();
  if (!event.isTrusted) return;
  event.stopImmediatePropagation();
  if (event.type === "click" || event.type === "contextmenu") event.preventDefault();
  loadRest().then(() => event.target?.dispatchEvent(new /** @type {any} */ (event.constructor)(event.type, event)), () => {});
}
for (const type of HELD) addEventListener(type, hold, true);
addEventListener("pointerdown", askRest, true);
