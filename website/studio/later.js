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
/** The rest, once. Failed (the server lost): asked again by the next press, or the live check (see checkLive). */
export const loadRest = () => (loading ??= Promise.all([import("/studio/rest.js"), restSheet()]).then(([rest]) => {
  ({ command, pen, grid, heatmaps, inspector, menus, presetsEdit, gestures, xkcd } = rest);
  restIn.resolve();
}, (error) => {
  loading = undefined;
  // (A press is of its moment: let go of, not done minutes on as the rest comes. A step reads where it is now: kept.)
  for (const type of PRESSES) kept.get(type)?.clear();
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
// Held till the rest is in, then replayed in order, of each kind the last only: a control's presses (three taps would
// open, close, open), a key's, a step's (it reads where it is now). A press on the chart isn't held, nor a key's
// default but for the page's own chords (⌘K would open the browser's search), nor a start over (Try again, Reload).
const PRESSES = ["click", "dblclick", "contextmenu", "keydown"];
/** The last held, by type, then kind. @type {Map<string, Map<unknown, Event>>} */
const kept = new Map([...PRESSES, "popstate", "hashchange", "storage"].map((type) => [type, new Map()]));
/** A press's kind is its control; a key's, its key; a stored change's, its key; a step's, a step. @param {Event} event */
const kindOf = (event) => (event instanceof MouseEvent ? (event.target instanceof Element && event.target.closest("button, a, [role=button], [data-action]")) || event.target : "key" in event ? event.key : "");
/** The page's chords are what its controls say as their keys (data-kbd: "⌘ K", "⇧ ⌘ Z"). @param {Event} event */
const pageChord = (event) => event instanceof KeyboardEvent && (event.metaKey || event.ctrlKey) && event.key.length === 1
  && Boolean(document.querySelector(`[data-kbd="${event.shiftKey ? "⇧ " : ""}⌘ ${CSS.escape(event.key.toUpperCase())}"]`));
/** @param {Event} event */
function hold(event) {
  if (command) return;
  askRest();
  if (!event.isTrusted || (event.target instanceof Element && event.target.closest("#offline, .booting"))) return;
  event.stopImmediatePropagation();
  if (event.type === "click" || event.type === "contextmenu" || pageChord(event)) event.preventDefault();
  const kinds = /** @type {Map<unknown, Event>} */ (kept.get(event.type)), kind = kindOf(event);
  kinds.set(kind, event);
  withRest(() => kinds.get(kind) === event && event.target?.dispatchEvent(new /** @type {any} */ (event.constructor)(event.type, event)));
}
for (const type of kept.keys()) addEventListener(type, hold, true);
addEventListener("pointerdown", askRest, true);
