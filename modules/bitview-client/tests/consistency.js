/**
 * Consistency test: verifies that all series sharing the same index have the same length.
 * Useful for catching stale/inconsistent state after a reorg rollback.
 */

import { BitviewClient } from "../index.js";

/**
 * @typedef {import('../index.js').AnySeriesPattern} AnyMetricPattern
 */

/**
 * @param {any} obj
 * @returns {obj is AnyMetricPattern}
 */
function isMetricPattern(obj) {
  return (
    obj &&
    typeof obj === "object" &&
    typeof obj.indexes === "function" &&
    obj.by &&
    typeof obj.by === "object"
  );
}

/**
 * Recursively collect all metric patterns from the tree.
 * @param {Record<string, any>} obj
 * @param {string} path
 * @returns {Array<{path: string, metric: AnyMetricPattern}>}
 */
function getAllMetrics(obj, path = "") {
  /** @type {Array<{path: string, metric: AnyMetricPattern}>} */
  const metrics = [];

  for (const key of Object.keys(obj)) {
    const attr = obj[key];
    if (!attr || typeof attr !== "object") continue;

    const currentPath = path ? `${path}.${key}` : key;

    if (isMetricPattern(attr)) {
      metrics.push({ path: currentPath, metric: attr });
    }

    if (typeof attr === "object" && !Array.isArray(attr)) {
      metrics.push(...getAllMetrics(attr, currentPath));
    }
  }

  return metrics;
}

async function testConsistency() {
  const client = new BitviewClient({
    baseUrl: "http://localhost:3110",
    timeout: 15000,
  });

  const metrics = getAllMetrics(client.series);
  console.log(`\nFound ${metrics.length} metrics`);

  /** @type {Map<string, Array<{path: string, total: number}>>} */
  const byIndex = new Map();

  for (const { path, metric } of metrics) {
    const indexes = metric.indexes();

    for (const idxName of indexes) {
      const fullPath = `${path}.by.${idxName}`;
      const endpoint = metric.by[idxName];

      if (!endpoint) {
        console.log(`SKIP: ${fullPath} (undefined endpoint)`);
        continue;
      }

      try {
        const result = await endpoint.last(0);
        const total = result.end;

        if (!byIndex.has(idxName)) {
          byIndex.set(idxName, []);
        }
        /** @type {Array<{path: string, total: number}>} */ (byIndex.get(idxName)).push({ path: fullPath, total });
      } catch (e) {
        console.log(
          `FAIL: ${fullPath} -> ${e instanceof Error ? e.message : e}`,
        );
        return;
      }
    }
  }

  let failed = false;

  for (const [index, entries] of byIndex) {
    const totals = new Set(entries.map((e) => e.total));

    if (totals.size === 1) {
      const [total] = totals;
      console.log(`OK: ${index} — ${entries.length} series, all length ${total}`);
      continue;
    }

    failed = true;
    console.log(`\nMISMATCH: ${index} — ${entries.length} series with ${totals.size} different lengths:`);

    /** @type {Map<number, string[]>} */
    const grouped = new Map();
    for (const { path, total } of entries) {
      if (!grouped.has(total)) grouped.set(total, []);
      /** @type {string[]} */ (grouped.get(total)).push(path);
    }

    for (const [total, paths] of [...grouped].sort((a, b) => b[0] - a[0])) {
      console.log(`  length ${total}: (${paths.length} series)`);
      for (const p of paths) {
        console.log(`    ${p}`);
      }
    }
  }

  if (failed) {
    console.log("\nFAILED: length mismatches detected");
    throw new Error("length mismatches detected");
  } else {
    console.log("\nPASSED: all indexes consistent");
  }
}

testConsistency();
