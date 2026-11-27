// @ts-check
// The rest (rest.js, rest.css): what a first chart doesn't run, loaded after it draws, so it isn't in the way.
import { typing } from "/studio/dom.js";
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
const restIn = /** @type {PromiseWithResolvers<void>} */ (Promise.withResolvers());
/** @type {Promise<void> | undefined} */
let loading;
/** @param {() => void} run */
export const withRest = (run) => (command ? run() : void restIn.promise.then(run));
// Failed (the server lost): asked again by the next press or the live check.
export const loadRest = () => (loading ??= Promise.all([import("/studio/rest.js"), restSheet()]).then(([rest]) => {
  ({ command, pen, grid, heatmaps, inspector, menus, presetsEdit, gestures, xkcd } = rest);
  restIn.resolve();
}, (error) => {
  loading = undefined;
  // A press is of its moment: let go of, not done minutes on as the rest comes. A step reads where it is now: kept.
  for (const type of PRESSES) kept.get(type)?.clear();
  throw error;
}));
/** @type {Promise<unknown> | undefined} */
let sheet;
// In before anything it styles shows.
const restSheet = () => (sheet ??= new Promise((resolve, reject) => {
  const link = Object.assign(document.createElement("link"), { rel: "stylesheet", href: import.meta.resolve("/studio/rest.css") });
  link.onload = resolve;
  link.onerror = () => (link.remove(), (sheet = undefined), reject(new Error("/studio/rest.css did not load")));
  document.head.append(link);
}));
export const askRest = () => void loadRest().catch(() => {});
// Replayed in order once the rest is in, the last of each kind only (three taps would open, close, open).
// Not held: a press on the chart, a key's default but the page's chords (⌘K would open the browser's search), a start over.
const PRESSES = ["click", "dblclick", "contextmenu", "keydown"];
/** @type {Map<string, Map<unknown, Event>>} */
const kept = new Map([...PRESSES, "popstate", "hashchange", "storage"].map((type) => [type, new Map()]));
/** @param {Event} event */
const kindOf = (event) => (event instanceof MouseEvent ? (event.target instanceof Element && event.target.closest("button, a, [role=button], [data-action]")) || event.target
  : event instanceof KeyboardEvent ? [event.metaKey, event.ctrlKey, event.altKey, event.shiftKey, event.key].join() : "key" in event ? event.key : "");
/**
 * In a field, the keys it edits with (undo, redo, select all, the clipboard) stay the field's own.
 * @param {Event} event
 */
const pageChord = (event) => event instanceof KeyboardEvent && (event.metaKey || event.ctrlKey) && event.key.length === 1
  && !(typing(event.target) && "acvxyz".includes(event.key.toLowerCase()))
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
