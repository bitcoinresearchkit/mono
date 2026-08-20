import { localhost } from "../utils/env.js";

/**
 * Walk a series tree and collect all chartable series patterns
 * @param {TreeNode | null | undefined} node
 * @param {Map<AnySeriesPattern, string[]>} map
 * @param {string[]} path
 */
function walkSeries(node, map, path) {
  if (node && "by" in node) {
    const seriesNode = /** @type {AnySeriesPattern} */ (node);
    if (!seriesNode.by.day1) return;
    map.set(seriesNode, path);
  } else if (node && typeof node === "object") {
    for (const [key, value] of Object.entries(node)) {
      const kn = key.toLowerCase();
      if (
        key === "sd24h" ||
        key === "emaSlow" ||
        key === "emaFast" ||
        kn === "cents" ||
        kn === "bps" ||
        kn === "ppm" ||
        kn === "constants" ||
        kn === "ohlc" ||
        kn === "split" ||
        kn === "spot" ||
        kn.startsWith("timestamp") ||
        kn.startsWith("coinyears") ||
        kn.endsWith("index") ||
        kn.endsWith("indexes")
      )
        continue;
      const newPath = [...path, key];
      const joined = newPath.join(".");
      if (
        joined.endsWith(".count.total.average") ||
        joined === "cohorts.utxo.all.supply.dominance" ||
        joined === "models.capitalSentiment.phase" ||
        joined === "cohorts.utxo.all.outputs.spentCount.average"
      )
        continue;
      walkSeries(/** @type {TreeNode | null | undefined} */ (value), map, newPath);
    }
  }
}

/**
 * Walk partial options tree and delete referenced series from the map
 * @param {PartialOptionsTree} options
 * @param {Map<AnySeriesPattern, string[]>} map
 */
function walkOptions(options, map) {
  for (const node of options) {
    if ("tree" in node && node.tree) {
      walkOptions(node.tree, map);
    } else if ("top" in node || "bottom" in node) {
      const chartNode = /** @type {PartialChartOption} */ (node);
      markUsedBlueprints(map, chartNode.top);
      markUsedBlueprints(map, chartNode.bottom);
    }
  }
}

/**
 * @param {Map<AnySeriesPattern, string[]>} map
 * @param {(AnyFetchedSeriesBlueprint | FetchedPriceSeriesBlueprint)[]} [arr]
 */
function markUsedBlueprints(map, arr) {
  if (!arr) return;
  for (let i = 0; i < arr.length; i++) {
    const s = arr[i].series;
    if (!s) continue;
    const maybePriceSeries =
      /** @type {{ usd?: AnySeriesPattern, sats?: AnySeriesPattern }} */ (
        /** @type {unknown} */ (s)
      );
    if (maybePriceSeries.usd?.by && maybePriceSeries.sats?.by) {
      map.delete(maybePriceSeries.usd);
      map.delete(maybePriceSeries.sats);
    } else {
      map.delete(/** @type {AnySeriesPattern} */ (s));
    }
  }
}

/**
 * Log unused series to console (localhost only)
 * @param {TreeNode} seriesTree
 * @param {PartialOptionsTree} partialOptions
 */
export function logUnused(seriesTree, partialOptions) {
  if (!localhost) return;

  console.log(extractTreeStructure(partialOptions));

  /** @type {Map<AnySeriesPattern, string[]>} */
  const all = new Map();
  walkSeries(seriesTree, all, []);
  walkOptions(partialOptions, all);

  if (!all.size) return;

  /** @type {Record<string, unknown>} */
  const tree = {};
  for (const path of all.values()) {
    /** @type {Record<string, unknown>} */
    let current = tree;
    for (let i = 0; i < path.length; i++) {
      const part = path[i];
      if (i === path.length - 1) {
        current[part] = null;
      } else {
        current[part] = current[part] || {};
        current = /** @type {Record<string, unknown>} */ (current[part]);
      }
    }
  }

  console.log("Unused series:", { count: all.size, tree });
}

/**
 * Extract tree structure from partial options (names + hierarchy, series grouped by unit)
 * @param {PartialOptionsTree} options
 * @returns {object[]}
 */
export function extractTreeStructure(options) {
  /**
   * Group series by unit
   * @param {(AnyFetchedSeriesBlueprint | FetchedPriceSeriesBlueprint)[]} series
   * @param {boolean} isTop
   * @returns {Record<string, string[]>}
   */
  function groupByUnit(series, isTop) {
    /** @type {Record<string, string[]>} */
    const grouped = {};
    for (const s of series) {
      const pattern = /** @type {AnySeriesPattern | AnyPricePattern} */ (
        s.series
      );
      if (isTop && "usd" in pattern && "sats" in pattern) {
        const title = s.title || s.key || "unnamed";
        (grouped["USD"] ??= []).push(title);
        (grouped["sats"] ??= []).push(title);
      } else {
        const unit = /** @type {AnyFetchedSeriesBlueprint} */ (s).unit;
        const unitName = unit?.name || "unknown";
        const title = s.title || s.key || "unnamed";
        (grouped[unitName] ??= []).push(title);
      }
    }
    return grouped;
  }

  /**
   * @param {AnyPartialOption | PartialOptionsGroup} node
   * @returns {object}
   */
  function processNode(node) {
    if ("tree" in node && node.tree) {
      return {
        name: node.name,
        children: node.tree.map(processNode),
      };
    }
    if ("top" in node || "bottom" in node) {
      const chartNode = /** @type {PartialChartOption} */ (node);
      const top = chartNode.top ? groupByUnit(chartNode.top, true) : undefined;
      const bottom = chartNode.bottom
        ? groupByUnit(chartNode.bottom, false)
        : undefined;
      return {
        name: node.name,
        title: chartNode.title,
        ...(top && Object.keys(top).length > 0 ? { top } : {}),
        ...(bottom && Object.keys(bottom).length > 0 ? { bottom } : {}),
      };
    }
    if ("url" in node) {
      return { name: node.name, url: true };
    }
    return { name: node.name };
  }

  return options.map(processNode);
}
