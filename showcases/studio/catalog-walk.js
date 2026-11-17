// @ts-check
// The catalog walked, shared by the page and its worker (catalog-worker.js).

/** A path through a trailing window's step (`…sum._24h`, `…sum._1m.btc`), as the client keys windows. */
export const WINDOWED = /\._\d+[hdwmy](?:\.|$)/;

/**
 * The catalog walked, a node a step (a generator: the page goes on at its own pace): its names to their dotted paths
 * as the client has them (a name in several places, its first, or its place among its windows: see WINDOWED), those by block only (by date in none of them), and
 * those under more than one top folder (a folder's place among them isn't theirs: see app.js's catalogTops). Walked
 * on the page and in catalog-worker.js (see indexCatalog), so its leaves are told as isLeaf tells them, read as
 * `aside` reads them.
 * @param {any} tree @param {readonly string[]} dated
 * @returns {Generator<void, { index: Map<string, string>, byBlock: Set<string>, sharedTops: Set<string> }>}
 */
export function* catalogWalk(tree, dated) {
  const index = new Map(), byBlock = new Set(), sharedTops = new Set(), stack = [[tree, ""]];
  while (stack.length) {
    const [node, path] = /** @type {[any, string]} */ (stack.pop());
    if (node && typeof node.name === "string" && typeof node.indexes === "function") {
      // (A name in several places, each with its own intervals (an interval's first height, under each): by date if
      // any of them is.)
      const byDate = dated.some((id) => node.indexes().includes(id));
      if (index.has(node.name)) {
        if (byDate) byBlock.delete(node.name);
        if (index.get(node.name).split(".")[0] !== path.split(".")[0]) sharedTops.add(node.name);
        // (Its place among its windows, if it has one: its other windows are found beside it there, see app.js's
        // chartWindow. A day's realized profit is also a rarity meter's extreme.)
        if (!WINDOWED.test(index.get(node.name)) && WINDOWED.test(path)) index.set(node.name, path);
      } else {
        index.set(node.name, path);
        if (!byDate) byBlock.add(node.name);
      }
    } else if (node && typeof node === "object") for (const key of Object.keys(node)) stack.push([Reflect.get(node, key, {}), path ? `${path}.${key}` : key]);
    yield;
  }
  return { index, byBlock, sharedTops };
}
