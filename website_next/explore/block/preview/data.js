import { fetchBrkSeries } from "../../../utils/api.js";

/**
 * @template T
 * @param {string} name
 * @param {string} index
 * @param {number} start
 * @param {number} end
 * @param {AbortSignal} signal
 * @returns {Promise<{ data: T[] }>}
 */
function fetchSeriesSlice(name, index, start, end, signal) {
  return fetchBrkSeries(name, index, start, end, signal);
}

/**
 * @param {import("../../../modules/bitview-client/index.js").BlockInfoV1} block
 * @param {AbortSignal} signal
 */
async function loadBlockPreviewRange(block, signal) {
  const firstTxIndex = (
    await fetchSeriesSlice(
      "first_tx_index",
      "height",
      block.height,
      block.height + 1,
      signal,
    )
  ).data[0];

  signal.throwIfAborted();

  const start = /** @type {number} */ (firstTxIndex);
  const end = start + block.txCount;

  return { start, end };
}

/**
 * @param {import("../../../modules/bitview-client/index.js").BlockInfoV1} block
 * @param {AbortSignal} signal
 */
export async function loadBlockPreview(block, signal) {
  const { start, end } = await loadBlockPreviewRange(block, signal);
  const [weights, feeRates] = await Promise.all([
    fetchSeriesSlice("tx_weight", "tx_index", start, end, signal),
    fetchSeriesSlice("fee_rate", "tx_index", start, end, signal),
  ]);

  signal.throwIfAborted();

  return {
    blockWeight: block.weight,
    range: { start, end },
    feeRates: /** @type {number[]} */ (feeRates.data),
    weights: /** @type {number[]} */ (weights.data),
  };
}

/**
 * @param {number} txIndex
 * @param {AbortSignal} signal
 */
export async function loadBlockPreviewTxid(txIndex, signal) {
  const txid = (
    await fetchSeriesSlice("txid", "tx_index", txIndex, txIndex + 1, signal)
  ).data[0];

  signal.throwIfAborted();

  return /** @type {string} */ (txid);
}

/**
 * @typedef {Object} BlockPreviewRange
 * @property {number} start
 * @property {number} end
 */

/**
 * @typedef {Object} BlockPreviewData
 * @property {number} blockWeight
 * @property {BlockPreviewRange} range
 * @property {number[]} weights
 * @property {number[]} feeRates
 */

/**
 * @typedef {Object} BlockPreviewTransaction
 * @property {number} txIndex
 * @property {number} weight
 * @property {number} feeRate
 */
