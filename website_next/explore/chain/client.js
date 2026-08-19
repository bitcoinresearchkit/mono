import { fetchBrkJson } from "../../utils/api.js";

/**
 * @typedef {import("../../modules/bitview-client/index.js").BlockInfoV1} Block
 * @typedef {import("../../modules/bitview-client/index.js").MempoolBlock} MempoolBlock
 * @typedef {Block & { tx_count?: number }} BlockResponse
 */

/** @param {BlockResponse} block */
function normalizeBlock(block) {
  block.txCount ??= /** @type {number} */ (block.tx_count);

  return block;
}

/** @param {string} path @param {AbortSignal} signal */
async function getBlocks(path, signal) {
  const blocks = await fetchBrkJson(path, signal);

  return /** @type {BlockResponse[]} */ (blocks).map(normalizeBlock);
}

export const chainClient = {
  /** @param {string} hash @param {AbortSignal} signal */
  async getBlock(hash, signal) {
    const block = await fetchBrkJson(
      `/api/v1/block/${encodeURIComponent(hash)}`,
      signal,
    );

    return normalizeBlock(/** @type {BlockResponse} */ (block));
  },

  /** @param {AbortSignal} signal */
  getBlocks(signal) {
    return getBlocks("/api/v1/blocks", signal);
  },

  /** @param {number} height @param {AbortSignal} signal */
  getBlocksFromHeight(height, signal) {
    return getBlocks(`/api/v1/blocks/${height}`, signal);
  },

  /** @param {AbortSignal} signal */
  getMempoolBlocks(signal) {
    return fetchBrkJson(
      "/api/v1/fees/mempool-blocks",
      signal,
    );
  },
};
