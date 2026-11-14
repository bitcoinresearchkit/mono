// @ts-check
// The band lookup and cutoff expansion, shared by formulas and their published data sources.
/** @type {WeakMap<object, Map<string, { bands: any[], at: number, path: string[] }>>} */
const indexes = new WeakMap();
/** @param {any} tree @param {string} name */
export function inBands(tree, name) {
  let found = indexes.get(tree);
  if (!found) {
    const index = new Map();
    indexes.set(tree, index);
    found = index;
    for (const bands of [...[tree.age.ranges, tree.utxos.amounts, tree.addresses.balances].map(Object.values), ...Object.values(tree.profitability).map((/** @type {any} */ family) => Object.values(family).reverse())]) {
      const walk = (/** @type {any} */ node, /** @type {string[]} */ path, /** @type {number} */ at) => {
        if (typeof node?.name === "string" && typeof node.indexes === "function") index.set(node.name, { bands, at, path });
        else if (node && typeof node === "object") for (const [key, value] of Object.entries(node)) walk(value, [...path, key], at);
      };
      bands.forEach((band, at) => walk(band, [], at));
    }
  }
  return found.get(name);
}
/** @param {NonNullable<ReturnType<typeof inBands>>} found @param {string} mode */
export function bandNames(found, mode) {
  return (mode === "over" ? found.bands.slice(found.at).reverse() : found.bands.slice(0, found.at)).flatMap((other) => {
    const leaf = found.path.reduce((leaf, key) => leaf?.[key], other);
    return typeof leaf?.name === "string" && typeof leaf.indexes === "function" ? [leaf.name] : [];
  });
}
