// @ts-check
import { placeWas } from "./places-were.js";
/** @typedef {{ presets: string[], sections: Map<string, string>, at: Map<string, { path: string[] } | { preset: number, window?: string }>, clashes: string[] }} AddressBook */

/**
 * Two under one place still alike are each tagged with a short hash of their name, so an address is the same whatever
 * else exists.
 * A preset once at an address keeps it for its window; a clash goes in `clashes` (seo.mjs stops the build). One that
 * moved keeps its old addresses too (places-were.js), never over a place there now.
 * @param {{ p: string[], w?: Record<string, string> }[]} list @returns {Generator<void, AddressBook>}
 */
export function* addressSteps(list) {
  const STEP = 200;
  let done = 0;
  const words = new Map();
  const word = (/** @type {string} */ text) => words.get(text) ?? words.set(text, slug(text)).get(text);
  const slug = (/** @type {string} */ text) => text.replace(/\+/g, " plus ").replace(/(^|\s)-(?=\d)|[−–]/g, "$1 minus ").replace(/≥/g, " at least ").replace(/≤/g, " at most ").replace(/>/g, " over ").replace(/</g, " under ").replace(/%/g, " percent ").replace(/&/g, " and ")
    .normalize("NFKD").replace(/[\u0300-\u036f]/g, "").toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "") || "x";
  const tag = (/** @type {string} */ text) => {
    let h = 2166136261;
    for (const c of text) h = Math.imul(h ^ /** @type {number} */ (c.codePointAt(0)), 16777619);
    return (h >>> 0).toString(36).slice(0, 4);
  };
  const sectionPaths = new Map();
  for (const preset of list) {
    for (let depth = 1; depth < preset.p.length; depth++) sectionPaths.set(JSON.stringify(preset.p.slice(0, depth)), preset.p.slice(0, depth));
    if (++done % STEP === 0) yield;
  }
  const alike = new Map(), count = (/** @type {string} */ key) => alike.set(key, (alike.get(key) ?? 0) + 1);
  const keyOf = (/** @type {string[]} */ path) => `${JSON.stringify(path.slice(0, -1))}|${word(/** @type {string} */ (path.at(-1)))}`;
  const keys = new Map(), keyed = (/** @type {string[]} */ path) => ((at) => keys.get(at) ?? keys.set(at, keyOf(path)).get(at))(JSON.stringify(path));
  for (const path of sectionPaths.values()) count(keyed(path));
  for (const preset of list) {
    count(keyed(preset.p));
    if (++done % STEP === 0) yield;
  }
  const named = (/** @type {string[]} */ path) => ((name) => (alike.get(keyed(path)) > 1 ? `${word(name)}-${tag(name)}` : word(name)))(/** @type {string} */ (path.at(-1)));
  const sections = new Map([["[]", "/presets/"]]), at = /** @type {AddressBook["at"]} */ (new Map([["/presets/", { path: [] }]]));
  const sectionAt = (/** @type {string[]} */ path) => {
    const key = JSON.stringify(path), known = sections.get(key);
    if (known) return known;
    const address = `${sectionAt(path.slice(0, -1))}${named(path)}/`;
    sections.set(key, address);
    at.set(address, { path });
    return address;
  };
  const presetsAt = [];
  for (const [i, preset] of list.entries()) {
    const address = `${sectionAt(preset.p.slice(0, -1))}${named(preset.p)}`;
    at.set(address, { preset: i });
    presetsAt.push(address);
    if (++done % STEP === 0) yield;
  }
  const clashes = [];
  for (const [i, preset] of list.entries()) for (const [name, window] of Object.entries(preset.w ?? {})) {
    const address = `${sectionAt(preset.p.slice(0, -1))}${word(name)}`;
    if (!at.has(address)) at.set(address, { preset: i, window });
    else clashes.push(`${address}, once ${preset.p.join(" › ")} at ${window}`);
    if (++done % STEP === 0) yield;
  }
  const addressOf = (/** @type {string[]} */ path) => `/presets/${path.map(word).join("/")}`;
  const alias = (/** @type {string} */ address, /** @type {{ path: string[] } | { preset: number, window?: string }} */ to) => at.has(address) || at.set(address, to);
  // (A section gone opens the deepest section that holds all its charts now.)
  const goneSections = new Map();
  for (const [i, preset] of list.entries()) for (const { path: old, window: folded } of placeWas(preset.p)) {
    alias(addressOf(old), folded ? { preset: i, window: folded } : { preset: i });
    // (A window's name as it was beside the chart, or as it is now, the window's place moved as the chart's was; a chart
    // folded into this one had none of this one's.)
    if (!folded) for (const [name, window] of Object.entries(preset.w ?? {})) for (const place of [[...old.slice(0, -1), name], ...placeWas([...preset.p.slice(0, -1), name]).map((was) => was.path)]) alias(addressOf(place), { preset: i, window });
    for (let depth = 1; depth < old.length; depth++) {
      const key = JSON.stringify(old.slice(0, depth));
      if (sections.has(key)) continue;
      const now = goneSections.get(key), parent = preset.p.slice(0, -1);
      goneSections.set(key, { old: old.slice(0, depth), now: now ? now.now.slice(0, now.now.findIndex((step, d) => step !== parent[d]) >>> 0) : parent });
    }
    if (++done % STEP === 0) yield;
  }
  for (const { old, now } of goneSections.values()) alias(`${addressOf(old)}/`, { path: now });
  return { presets: presetsAt, sections, at, clashes };
}
/** @param {{ p: string[], w?: Record<string, string> }[]} list */
export function addressesOf(list) {
  const steps = addressSteps(list);
  let step = steps.next();
  while (!step.done) step = steps.next();
  return step.value;
}
