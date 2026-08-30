import { BITVIEW_BASE_URL } from "../../../utils/client.js";
import { WorkerClient } from "../worker-client.js";

const WORKER_URL = import.meta.resolve("./worker.js");
const OPENAPI_URL = `${BITVIEW_BASE_URL}/openapi.json`;

/**
 * @typedef {Object} ApiParameter
 * @property {string} name
 * @property {"path" | "query"} in
 * @property {boolean} required
 * @property {string} type
 * @property {string} [valueType]
 * @property {string} [primitive]
 * @property {string} [format]
 * @property {unknown[]} [enum]
 * @property {string} description
 *
 * @typedef {Object} ApiOperation
 * @property {string} key
 * @property {"GET"} method
 * @property {string} path
 * @property {string} label
 * @property {string} summary
 * @property {string} description
 * @property {ApiParameter[]} parameters
 * @property {{ contentType: string, type: string, description: string, fields: { name: string, type: string, required: boolean, description: string, ownDescription: string }[] }} response
 * @property {string} [matchedQuery]
 * @property {number} [matchedTerms]
 * @property {number} [titleMatchedTerms]
 * @property {number} [specificity]
 * @property {number} [score]
 */

const index = new WorkerClient(WORKER_URL, {
  data: { url: OPENAPI_URL },
  failed: "The API index failed",
  stopped: "API search stopped",
});

export function prewarmApiIndex() {
  return index.request("prewarm", {});
}

/** @param {string[]} queries @param {number} [limit] @param {(() => void) | undefined} [onProgress] @returns {Promise<ApiOperation[]>} */
export function searchApi(queries, limit = 8, onProgress) {
  return index.request("search", { queries, limit }, onProgress);
}

/** @param {string} key @returns {Promise<ApiOperation | undefined>} */
export function apiByKey(key) {
  return index.request("byKey", { key });
}

export function terminateApiIndex() {
  index.terminate();
}
