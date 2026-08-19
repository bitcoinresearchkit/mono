/**
 * Comprehensive test that fetches all endpoints in the tree.
 */

import { BitviewClient } from "../index.js";

/**
 * @typedef {import('../index.js').AnySeriesPattern} AnyMetricPattern
 */

/**
 * Check if an object is a metric pattern (has indexes() method and by object).
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

    // Check if this is a metric pattern using the indexes() method
    if (isMetricPattern(attr)) {
      metrics.push({ path: currentPath, metric: attr });
      continue;
    }

    // Recurse into nested tree nodes
    if (typeof attr === "object" && !Array.isArray(attr)) {
      metrics.push(...getAllMetrics(attr, currentPath));
    }
  }

  return metrics;
}

async function testAllEndpoints() {
  const client = new BitviewClient({
    baseUrl: "http://localhost:3110",
    timeout: 15000,
  });

  const metrics = getAllMetrics(client.series);
  console.log(`\nFound ${metrics.length} metrics`);

  let success = 0;

  for (const { path, metric } of metrics) {
    // Use the indexes() method to get all available indexes
    const indexes = metric.indexes();

    for (const idxName of indexes) {
      const fullPath = `${path}.by.${idxName}`;

      try {
        // Verify both access methods work: .by[index] and .get(index)
        const endpointByProperty = metric.by[idxName];
        const endpointByGet = metric.get(idxName);

        if (!endpointByProperty) {
          throw new Error(`metric.by.${idxName} is undefined`);
        }
        if (!endpointByGet) {
          throw new Error(`metric.get('${idxName}') returned undefined`);
        }

        await endpointByProperty.last(0);
        success++;
        console.log(`OK: ${fullPath}`);
      } catch (e) {
        console.log(
          `FAIL: ${fullPath} -> ${e instanceof Error ? e.message : e}`,
        );
        throw e;
      }
    }
  }

  console.log(`\n=== Results ===`);
  console.log(`Success: ${success}`);
}

testAllEndpoints();
