import { BITVIEW_BASE_URL, bitview } from "../../../utils/client.js";
import { WorkerClient } from "../worker-client.js";

const WORKER_URL = import.meta.resolve("./worker.js");
const SERIES_URL = `${BITVIEW_BASE_URL}/api/series`;
const FORBIDDEN_KEYS = new Set(["__proto__", "constructor", "prototype"]);

/** @typedef {{ path: string, name: string, endpoint: string, indexes: string[], type: string, suggestedUnit?: string, matchedQuery?: string, matchedTerms?: number, specificity?: number, relevance?: number, score?: number }} CatalogMetric */

/** @param {unknown} value */
function isMetric(value) {
  if (!value || typeof value !== "object") return false;
  const by = /** @type {{ by?: Record<string, unknown> }} */ (value).by;
  return Boolean(
    by &&
      Object.values(by).some(
        (endpoint) =>
          endpoint &&
          typeof endpoint === "object" &&
          typeof /** @type {{ fetch?: unknown }} */ (endpoint).fetch === "function",
      ),
  );
}

/** @param {string} path */
function normalizePath(path) {
  const normalized = path
    .trim()
    .replace(/^bitview\.series\./, "")
    .replace(/^series\./, "");
  const keys = normalized.split(".").filter(Boolean);

  if (!keys.length || keys.some((key) => FORBIDDEN_KEYS.has(key))) {
    throw new Error(`Invalid metric path: ${path}`);
  }
  return keys;
}

/** @param {typeof bitview} client @param {string} path */
function resolveMetric(client, path) {
  const keys = normalizePath(path);
  /** @type {unknown} */
  let value = client.series;

  for (const key of keys) {
    if (!value || typeof value !== "object" || !Object.hasOwn(value, key)) {
      throw new Error(`Metric not found: ${path}`);
    }
    value = /** @type {Record<string, unknown>} */ (value)[key];
  }

  if (!isMetric(value)) throw new Error(`Metric not found: ${path}`);
  return /** @type {TimeframeMetric} */ (value);
}

/** @param {string} path */
export function createMetric(path) {
  resolveMetric(bitview, path);
  return (/** @type {typeof bitview} */ client) => resolveMetric(client, path);
}

const index = new WorkerClient(WORKER_URL, {
  data: { url: SERIES_URL },
  failed: "The metric index failed",
  stopped: "Metric search stopped",
});

export function prewarmMetricIndex() {
  return index.request("prewarm", {});
}

/** @param {string[]} queries @param {number} [limit] @param {string[]} [prefixes] @param {(() => void) | undefined} [onProgress] @returns {Promise<CatalogMetric[]>} */
export function searchMetrics(queries, limit = 16, prefixes = [], onProgress) {
  return index.request(
    "search",
    { queries, limit, prefixes },
    onProgress,
  );
}

/** @param {string} query @param {(() => void) | undefined} [onProgress] @returns {Promise<string[]>} */
export function mentionedMetricNames(query, onProgress) {
  return index.request("mentions", { query }, onProgress);
}

/** @param {string} name @returns {Promise<CatalogMetric | undefined>} */
export function metricByName(name) {
  return index.request("byName", { name });
}

/** @param {string[]} paths @param {(() => void) | undefined} [onProgress] @returns {Promise<CatalogMetric[]>} */
export function metricsByPaths(paths, onProgress) {
  return index.request("byPaths", { paths }, onProgress);
}

/** @param {{ name: string }} metric @param {string} query @returns {Promise<{ totalSeries: number, groups: { family: string, examples: string[] }[], series: (CatalogMetric & { selector: string, matchedTerms: number, specificity: number })[] } | undefined>} */
export function metricVariants(metric, query = "") {
  return index.request("variants", {
    name: metric.name,
    path: /** @type {{ path?: string }} */ (metric).path,
    query,
  });
}

export function terminateMetricIndex() {
  index.terminate();
}
