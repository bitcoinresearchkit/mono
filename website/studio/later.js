// @ts-check
// What a first chart doesn't run (⌘K, the grid, the inspector, the menus, editing, drawing, heatmaps, gestures), loaded
// once the chart has drawn, or at a first press or key, whichever comes first; at once where it's what shows (the home is
// the grid; a heatmap draws with heatmaps): see rest.js. Each module is here once it's in. What needs one before then
// waits for it (withRest); what shows beside the chart is shown as it comes.
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
/**
 * The rest, loaded once: its modules here, then what waited for them, then what was pressed or typed meanwhile. Not
 * loading (the server lost): what was pressed let go of, asked again at the next press or key; the page it's needed to
 * show (Home) says so, see start.
 */
export const loadRest = () => (loading ??= import("/studio/rest.js").then((rest) => {
  ({ command, pen, grid, heatmaps, inspector, menus, presetsEdit, gestures, xkcd } = rest);
  restIn.resolve();
}, (error) => {
  loading = undefined;
  throw error;
}));
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
