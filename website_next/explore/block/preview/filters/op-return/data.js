import { brk } from "../../../../../utils/client.js";
import {
  OP_RETURN_KIND_FILTERS,
  OP_RETURN_POLICY_FILTERS,
} from "./model.js";

const MAX_STANDARD_BYTES = 82;
const byKind = brk.series.opReturn.byKind;
const policy = brk.series.opReturn.policy;

export const OP_RETURN_COUNT_SOURCES = [
  ...OP_RETURN_KIND_FILTERS.map(([, kind, clientKey]) => {
    return {
      key: `op_return:${kind}`,
      series: byKind.txCount[clientKey].block,
    };
  }),
  ...OP_RETURN_POLICY_FILTERS.map(([, filter, clientKey]) => {
    return {
      key: `op_return_policy:${filter}`,
      series: policy.txCount[clientKey].block,
    };
  }),
];

/**
 * @param {string} policy
 * @param {number} count
 * @param {boolean} oversized
 */
function matchesPolicy(policy, count, oversized) {
  if (policy === "standard") return count === 1 && !oversized;
  if (policy === "oversized") return oversized;
  if (policy === "multiple") return count > 1;

  return oversized || count > 1;
}

/**
 * @param {number} height
 * @param {BlockPreviewRange} range
 * @param {AbortSignal} signal
 */
export function createOpReturnFilterData(height, range, signal) {
  const raw = brk.series.opReturn.raw;
  let rawPromise = /** @type {Promise<OpReturnRows> | null} */ (null);

  async function fetchRows() {
    const indexes = await raw.firstIndex.by.height
      .skip(height)
      .take(2)
      .fetch({ signal });
    const start = /** @type {number} */ (indexes.data[0]);
    const end = indexes.data.length === 2
      ? /** @type {number} */ (indexes.data[1])
      : await raw.toTxIndex.by.op_return_index.len();

    signal.throwIfAborted();
    if (start === end) return { bytes: [], kinds: [], txIndexes: [] };

    const count = end - start;
    const [txIndexes, kinds, bytes] = await Promise.all([
      raw.toTxIndex.by.op_return_index
        .skip(start)
        .take(count)
        .fetch({ signal, memCache: false }),
      raw.kind.by.op_return_index
        .skip(start)
        .take(count)
        .fetch({ signal, memCache: false }),
      raw.postOpReturnBytes.by.op_return_index
        .skip(start)
        .take(count)
        .fetch({ signal, memCache: false }),
    ]);

    signal.throwIfAborted();

    return {
      bytes: bytes.data,
      kinds: kinds.data,
      txIndexes: txIndexes.data,
    };
  }

  function loadRows() {
    rawPromise ??= fetchRows();

    return rawPromise;
  }

  /** @param {string} key */
  async function loadMembership(key) {
    const rows = await loadRows();
    const membership = new Uint8Array(range.end - range.start);
    const separator = key.indexOf(":");
    const group = key.slice(0, separator);
    const value = key.slice(separator + 1);

    if (group === "op_return") {
      for (let index = 0; index < rows.txIndexes.length; index += 1) {
        if (rows.kinds[index] === value) {
          membership[rows.txIndexes[index] - range.start] = 1;
        }
      }

      return membership;
    }

    for (let index = 0; index < rows.txIndexes.length;) {
      const txIndex = rows.txIndexes[index];
      let count = 0;
      let oversized = false;

      while (index < rows.txIndexes.length && rows.txIndexes[index] === txIndex) {
        oversized ||= rows.bytes[index] > MAX_STANDARD_BYTES;
        count += 1;
        index += 1;
      }

      if (matchesPolicy(value, count, oversized)) {
        membership[txIndex - range.start] = 1;
      }
    }

    return membership;
  }

  /** @param {number} txIndex */
  async function loadTransactionKeys(txIndex) {
    const rows = await loadRows();
    const keys = /** @type {string[]} */ ([]);
    const kinds = new Set();
    let count = 0;
    let oversized = false;

    for (let index = 0; index < rows.txIndexes.length; index += 1) {
      const rowTxIndex = rows.txIndexes[index];

      if (rowTxIndex < txIndex) continue;
      if (rowTxIndex > txIndex) break;

      kinds.add(rows.kinds[index]);
      oversized ||= rows.bytes[index] > MAX_STANDARD_BYTES;
      count += 1;
    }

    for (const kind of kinds) keys.push(`op_return:${kind}`);
    for (const [, filter] of OP_RETURN_POLICY_FILTERS) {
      if (matchesPolicy(filter, count, oversized)) {
        keys.push(`op_return_policy:${filter}`);
      }
    }

    return keys;
  }

  return /** @type {const} */ ({ loadMembership, loadTransactionKeys });
}

/**
 * @typedef {Object} OpReturnRows
 * @property {number[]} bytes
 * @property {string[]} kinds
 * @property {number[]} txIndexes
 */

/** @typedef {import("../../data.js").BlockPreviewRange} BlockPreviewRange */
