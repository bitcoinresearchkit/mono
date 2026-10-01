const SKIP_KEYS = new Set(["cents", "bps", "ppm", "constants", "ohlc", "spot"]);

/**
 * Collect chartable series that no chart references, comparing names so cohort
 * projections and duplicate catalog paths count as the same series.
 * @param {Bitview.SeriesTree} seriesTree
 * @param {ReadonlySet<string>} used
 * @returns {Map<string, string[]>}
 */
export function collectUnusedSeries(seriesTree, used) {
  /** @type {Map<string, string[]>} */
  const unused = new Map();

  /**
   * @param {unknown} node
   * @param {string[]} path
   */
  function walk(node, path) {
    if (!node || typeof node !== "object") return;
    if (
      "name" in node &&
      "indexes" in node &&
      typeof node.indexes === "function"
    ) {
      const series = /** @type {AnySeriesPattern} */ (node);
      if (series.indexes().includes("day1") && !used.has(series.name)) {
        unused.set(series.name, path);
      }
      return;
    }
    for (const [key, child] of Object.entries(node)) {
      const lower = key.toLowerCase();
      if (
        SKIP_KEYS.has(lower) ||
        lower.startsWith("timestamp") ||
        lower.startsWith("coinyears") ||
        lower.endsWith("index") ||
        lower.endsWith("indexes")
      )
        continue;
      walk(child, [...path, key]);
    }
  }

  walk(seriesTree, []);
  return unused;
}

/**
 * @param {Bitview.SeriesTree} seriesTree
 * @param {ReadonlySet<string>} used
 */
export function logUnused(seriesTree, used) {
  const unused = collectUnusedSeries(seriesTree, used);
  /** @typedef {{ [key: string]: UnusedTree | null }} UnusedTree */
  /** @type {UnusedTree} */
  const tree = {};
  for (const path of unused.values()) {
    let current = tree;
    for (const [index, key] of path.entries()) {
      if (index === path.length - 1) current[key] = null;
      else current = current[key] ??= {};
    }
  }
  console.log("Unused series:", { count: unused.size, tree });
}
