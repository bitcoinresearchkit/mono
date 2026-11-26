// @ts-check
// The catalog walked, shared by the page and its worker (catalog-worker.js), and the checks on what names a series in
// it (a name, a path) before it reaches an API path.

import { isWindowKey } from "./words.js";

/** @param {string} path */
const windowed = (path) => path.split(".").some(isWindowKey);

// Read without caching on the parent: a normal walk would retain the whole catalog.
const aside = (/** @type {any} */ node, /** @type {string} */ key) => Reflect.get(node, key, {});
const isSeriesName = (name) => typeof name === "string" && /^[a-z0-9_]{1,200}$/.test(name);
const isCatalogPath = (path) => typeof path === "string" && /^\w{1,80}(\.\w{1,80}){0,15}$/.test(path);

/**
 * Hot: isLeaf and aside are written out, a call each per node was slower with the JIT off.
 * @param {any} tree
 * @param {readonly string[]} dated
 * @returns {Generator<void, { index: Map<string, string>, intervals: Map<string, readonly string[]>, sharedTops: Set<string> }>}
 */
function* catalogWalk(tree, dated) {
  const index = new Map(), intervals = new Map(), sharedTops = new Set(), stack = [[tree, ""]];
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
