// @ts-check
/**
 * The presets' addresses: each preset's and section's, and each address back to its place.
 * @typedef {{ presets: string[], sections: Map<string, string>, at: Map<string, { path: string[] } | { preset: number }> }} AddressBook
 */

/**
 * A preset's place as shown: each step by its label where it has one (`l`, worded as its titles are), else its place
 * (`p`, kept as it was: its address, key and links). @param {{ p: string[], l?: string[] }} preset
 */
export const placeShown = (preset) => preset.l ?? preset.p;

/**
 * Each place of the presets' an address of its own, for search engines and links: their page /presets/, a section
 * /presets/valuation/, a preset /presets/valuation/mvrv (a section's ending in a slash, as a folder's does: the page
 * knows which before they're read). Names as an address writes them: lowercase words joined by dashes, signs as words
 * (>4M RP: over-4m-rp; +90%: plus-90-percent). Two under one place still alike, each tagged by its own name (a short
 * hash of it): an address is the same whatever else there is, and in whatever order. Made a few hundred places a
 * step (a generator: the page makes them while it's idle, a slice at a time); addressesOf, all at once.
 * @param {{ p: string[] }[]} list
 * @returns {Generator<void, AddressBook>}
 */
export function* addressSteps(list) {
  const STEP = 200;
  let done = 0;
  // (A name's word made once: names come again and again, a section's under each of its presets.)
  const words = new Map();
  const word = (/** @type {string} */ text) => words.get(text) ?? words.set(text, slug(text)).get(text);
  const slug = (/** @type {string} */ text) => text.replace(/\+/g, " plus ").replace(/(^|\s)-(?=\d)|[−–]/g, "$1 minus ").replace(/≥/g, " at least ").replace(/≤/g, " at most ").replace(/>/g, " over ").replace(/</g, " under ").replace(/%/g, " percent ").replace(/&/g, " and ")
    .normalize("NFKD").replace(/[\u0300-\u036f]/g, "").toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "") || "x";
  const tag = (/** @type {string} */ text) => {
    let h = 2166136261;
    for (const c of text) h = Math.imul(h ^ /** @type {number} */ (c.codePointAt(0)), 16777619);
    return (h >>> 0).toString(36).slice(0, 4);
  };
  // (How many under one place would be written alike, sections and presets together: those, tagged.)
  const sectionPaths = new Map();
  for (const preset of list) {
    for (let depth = 1; depth < preset.p.length; depth++) sectionPaths.set(JSON.stringify(preset.p.slice(0, depth)), preset.p.slice(0, depth));
    if (++done % STEP === 0) yield;
  }
  const alike = new Map(), count = (/** @type {string} */ key) => alike.set(key, (alike.get(key) ?? 0) + 1);
  const keyOf = (/** @type {string[]} */ path) => `${JSON.stringify(path.slice(0, -1))}|${word(/** @type {string} */ (path.at(-1)))}`;
  // (Each path's key made once, for its count and its name alike.)
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
  return { presets: presetsAt, sections, at };
}
/** The presets' addresses, all at once (see addressSteps). @param {{ p: string[] }[]} list */
export function addressesOf(list) {
  const steps = addressSteps(list);
  let step = steps.next();
  while (!step.done) step = steps.next();
  return step.value;
}
