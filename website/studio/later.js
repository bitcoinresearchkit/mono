// @ts-check
// The rest (rest.js, rest.css): what a first chart doesn't run, loaded after it draws, so it isn't in the way.
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
/** What waits for the rest: one queue, so what was asked and what was pressed run in the order they came. */
const restIn = /** @type {PromiseWithResolvers<void>} */ (Promise.withResolvers());
/** @type {Promise<void> | undefined} */
let loading;
/** `run` once the rest is in. @param {() => void} run */
export const withRest = (run) => (command ? run() : void restIn.promise.then(run));
/** The rest, once. Failed (the server lost): the page says it's offline, and the next press asks again. */
export const loadRest = () => (loading ??= Promise.all([import("/studio/rest.js"), restSheet()]).then(([rest]) => {
  ({ command, pen, grid, heatmaps, inspector, menus, presetsEdit, gestures, xkcd } = rest);
  document.body.removeAttribute("data-offline");
  restIn.resolve();
}, (error) => {
  loading = undefined;
  document.body.toggleAttribute("data-offline", true);
  throw error;
}));
/** @type {Promise<unknown> | undefined} */
let sheet;
/** rest.css, in before anything it styles shows; once, however often rest.js is asked again. */
const restSheet = () => (sheet ??= new Promise((resolve, reject) => {
  const link = Object.assign(document.createElement("link"), { rel: "stylesheet", href: import.meta.resolve("/studio/rest.css") });
  link.onload = resolve;
  link.onerror = () => (link.remove(), (sheet = undefined), reject(new Error("/studio/rest.css did not load")));
  document.head.append(link);
}));
/** For a first chart drawn, or a first press. */
export const askRest = () => void loadRest().catch(() => {});
// Held till the rest is in, then replayed in order (of clicks the last only: three taps would open, close, open). A
// press on the chart isn't held, nor a key's default but for the page's own chords (⌘K would open the browser's search).
const HELD = ["click", "dblclick", "contextmenu", "keydown", "popstate", "hashchange", "storage"];
/** The page's chords are what its controls say as their keys (data-kbd: "⌘ K", "⇧ ⌘ Z"). @param {Event} event */
const pageChord = (event) => event instanceof KeyboardEvent && (event.metaKey || event.ctrlKey) && event.key.length === 1
  && Boolean(document.querySelector(`[data-kbd="${event.shiftKey ? "⇧ " : ""}⌘ ${CSS.escape(event.key.toUpperCase())}"]`));
/** @type {Event | undefined} */
let lastClick;
/** @param {Event} event */
function hold(event) {
  if (command) return;
  askRest();
  if (!event.isTrusted) return;
  event.stopImmediatePropagation();
  if (event.type === "click" || event.type === "contextmenu" || pageChord(event)) event.preventDefault();
  if (event.type === "click") lastClick = event;
  withRest(() => (event.type !== "click" || event === lastClick) && event.target?.dispatchEvent(new /** @type {any} */ (event.constructor)(event.type, event)));
}
for (const type of HELD) addEventListener(type, hold, true);
addEventListener("pointerdown", askRest, true);
