// @ts-check
// The catalog walked, shared by the page and its worker (catalog-worker.js), and the checks on what names a series in
// it (a name, a path) before it reaches an API path.

import { isWindowKey } from "./words.js";

/** Whether a path passes through a trailing window's step (`…sum._24h`, `…sum._1m.btc`). @param {string} path */
const windowed = (path) => path.split(".").some(isWindowKey);

/**
 * Read catalog children without caching them on their parents (a normal walk would retain tens of megabytes);
 * catalogWalk writes it out (see there).
 */
const aside = (/** @type {any} */ node, /** @type {string} */ key) => Reflect.get(node, key, {});
/** Series names are plain identifiers; anything else never reaches an API path. */
const isSeriesName = (name) => typeof name === "string" && /^[a-z0-9_]{1,200}$/.test(name);
/** A series' path in the catalog: its keys, dotted. */
const isCatalogPath = (path) => typeof path === "string" && /^\w{1,80}(\.\w{1,80}){0,15}$/.test(path);

/**
 * The catalog walked, a node a step (a generator: the page goes on at its own pace): its names to their dotted paths
 * as the client has them (a name in several places, its first, or its place among its windows: see windowed), to the
 * `dated` intervals they're charted at (none: by block only; a name in several places, those of any of them, as its
 * info has them), and those under more than one top folder (a folder's place among them isn't theirs: see command.js's
 * catalogTops). Walked on the page and in catalog-worker.js (see indexCatalog). Its leaves are told as isLeaf
 * (words.js) tells them and read as `aside` reads them, the two written out: a call each per node made the walk 12 to
 * 23% slower with the JIT off (bun, a synthetic catalog of ~66,000 nodes, as a phone's worker without its JIT would
 * run it).
 * @param {any} tree @param {readonly string[]} dated
 * @returns {Generator<void, { index: Map<string, string>, intervals: Map<string, readonly string[]>, sharedTops: Set<string> }>}
 */
function* catalogWalk(tree, dated) {
  const index = new Map(), intervals = new Map(), sharedTops = new Set(), stack = [[tree, ""]];
  // (Each list of indexes the client shares among its series, its dated intervals: found once, shared alike.)
  const datedOf = new Map();
  while (stack.length) {
    const [node, path] = /** @type {[any, string]} */ (stack.pop());
    if (node && typeof node.name === "string" && typeof node.indexes === "function") {
      const all = node.indexes();
      if (!datedOf.has(all)) datedOf.set(all, dated.filter((id) => all.includes(id)));
      const ids = datedOf.get(all), had = intervals.get(node.name);
      if (had) {
        if (had !== ids) intervals.set(node.name, dated.filter((id) => had.includes(id) || ids.includes(id)));
        if (index.get(node.name).split(".")[0] !== path.split(".")[0]) sharedTops.add(node.name);
        // (Its place among its windows, if it has one: its other windows are found beside it there, see app.js's
        // chartWindow. A day's realized profit is also a rarity meter's extreme.)
        if (!windowed(index.get(node.name)) && windowed(path)) index.set(node.name, path);
      } else {
        index.set(node.name, path);
        intervals.set(node.name, ids);
      }
    } else if (node && typeof node === "object") for (const key of Object.keys(node)) stack.push([Reflect.get(node, key, {}), path ? `${path}.${key}` : key]);
    yield;
  }
  return { index, intervals, sharedTops };
}

export { aside, catalogWalk, isCatalogPath, isSeriesName };
