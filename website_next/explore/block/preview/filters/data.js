import { bitview } from "../../../../utils/client.js";
import { FILTERS } from "./model.js";
import {
  createOpReturnFilterData,
  OP_RETURN_COUNT_SOURCES,
} from "./op-return/data.js";

const tx = bitview.series.transactions;
const txIndexes = bitview.series.indexes.txIndex;

/** @param {unknown} expected */
const equals = (expected) => (/** @type {unknown} */ value) => value === expected;

/** @param {unknown} expected */
const differs = (expected) => (/** @type {unknown} */ value) => value !== expected;

/** @param {unknown} value */
const isOtherVersion = (value) => value !== 1 && value !== 2 && value !== 3;

const COUNT_SOURCES = [
  { key: "version:1", series: tx.features.count.v1 },
  { key: "version:2", series: tx.features.count.v2 },
  { key: "version:3", series: tx.features.count.v3 },
  { key: "version:other", series: tx.features.count.otherVersion },
  { key: "rbf:yes", series: tx.features.count.explicitlyRbf },
  { key: "input:one", series: tx.features.count.oneInput },
  { key: "output:one", series: tx.features.count.oneOutput },
  { key: "type:p2pk", series: tx.features.count.p2pk },
  { key: "type:p2pkh", series: tx.features.count.p2pkh },
  { key: "type:p2sh", series: tx.features.count.p2sh },
  { key: "type:p2wpkh", series: tx.features.count.p2wpkh },
  { key: "type:p2wsh", series: tx.features.count.p2wsh },
  { key: "type:taproot", series: tx.features.count.p2tr },
  { key: "type:p2a", series: tx.features.count.p2a },
  { key: "type:multisig", series: tx.features.count.p2ms },
  { key: "type:op_return", series: tx.features.count.opReturn },
  { key: "type:empty", series: tx.features.count.empty },
  { key: "type:unknown", series: tx.features.count.unknown },
  { key: "behavior:cpfp_parent", series: tx.fees.count.cpfpParent },
  { key: "behavior:cpfp_child", series: tx.fees.count.cpfpChild },
  { key: "behavior:coinjoin", series: tx.patterns.count.coinjoin },
  { key: "behavior:consolidation", series: tx.patterns.count.consolidation },
  { key: "behavior:batch", series: tx.patterns.count.batchPayout },
  { key: "data:fake_pubkey", series: tx.features.count.fakePubkey },
  { key: "data:fake_scripthash", series: tx.features.count.fakeScripthash },
  { key: "data:inscription", series: tx.features.count.inscription },
  { key: "data:annex", series: tx.features.count.annex },
  { key: "data:dust", series: tx.features.count.dustOutput },
  { key: "sighash:all", series: tx.features.count.sighashAll },
  { key: "sighash:none", series: tx.features.count.sighashNone },
  { key: "sighash:single", series: tx.features.count.sighashSingle },
  { key: "sighash:default", series: tx.features.count.sighashDefault },
  {
    key: "sighash:anyone_can_pay",
    series: tx.features.count.sighashAnyoneCanPay,
  },
  { key: "policy:nonstandard", series: tx.policy.count },
  ...OP_RETURN_COUNT_SOURCES,
];

const MEMBERSHIP_SOURCES = /** @type {Map<string, BlockPreviewFilterSource>} */ (new Map([
  ["version:1", { matches: equals(1), series: tx.raw.txVersion }],
  ["version:2", { matches: equals(2), series: tx.raw.txVersion }],
  ["version:3", { matches: equals(3), series: tx.raw.txVersion }],
  ["version:other", { matches: isOtherVersion, series: tx.raw.txVersion }],
  ["rbf:yes", { matches: Boolean, series: tx.raw.isExplicitlyRbf }],
  ["rbf:no", { matches: equals(false), series: tx.raw.isExplicitlyRbf }],
  ["input:one", { matches: equals(1), series: txIndexes.inputCount }],
  ["input:multi", { matches: differs(1), series: txIndexes.inputCount }],
  ["output:one", { matches: equals(1), series: txIndexes.outputCount }],
  ["output:multi", { matches: differs(1), series: txIndexes.outputCount }],
  ["type:p2pk", { matches: Boolean, series: tx.features.hasP2pk }],
  ["type:p2pkh", { matches: Boolean, series: tx.features.hasP2pkh }],
  ["type:p2sh", { matches: Boolean, series: tx.features.hasP2sh }],
  ["type:p2wpkh", { matches: Boolean, series: tx.features.hasP2wpkh }],
  ["type:p2wsh", { matches: Boolean, series: tx.features.hasP2wsh }],
  ["type:taproot", { matches: Boolean, series: tx.features.hasP2tr }],
  ["type:p2a", { matches: Boolean, series: tx.features.hasP2a }],
  ["type:multisig", { matches: Boolean, series: tx.features.hasP2ms }],
  ["type:op_return", { matches: Boolean, series: tx.features.hasOpReturn }],
  ["type:empty", { matches: Boolean, series: tx.features.hasEmpty }],
  ["type:unknown", { matches: Boolean, series: tx.features.hasUnknown }],
  [
    "behavior:cpfp_parent",
    { matches: Boolean, series: tx.fees.isCpfpParent },
  ],
  [
    "behavior:cpfp_child",
    { matches: Boolean, series: tx.fees.isCpfpChild },
  ],
  ["behavior:coinjoin", { matches: Boolean, series: tx.patterns.isCoinjoin }],
  [
    "behavior:consolidation",
    { matches: Boolean, series: tx.patterns.isConsolidation },
  ],
  ["behavior:batch", { matches: Boolean, series: tx.patterns.isBatchPayout }],
  ["data:fake_pubkey", { matches: Boolean, series: tx.features.hasFakePubkey }],
  [
    "data:fake_scripthash",
    { matches: Boolean, series: tx.features.hasFakeScripthash },
  ],
  ["data:inscription", { matches: Boolean, series: tx.features.hasInscription }],
  ["data:annex", { matches: Boolean, series: tx.features.hasAnnex }],
  ["data:dust", { matches: Boolean, series: tx.features.hasDustOutput }],
  ["sighash:all", { matches: Boolean, series: tx.features.hasSighashAll }],
  ["sighash:none", { matches: Boolean, series: tx.features.hasSighashNone }],
  ["sighash:single", { matches: Boolean, series: tx.features.hasSighashSingle }],
  ["sighash:default", { matches: Boolean, series: tx.features.hasSighashDefault }],
  [
    "sighash:anyone_can_pay",
    { matches: Boolean, series: tx.features.hasSighashAnyoneCanPay },
  ],
  ["policy:nonstandard", { matches: Boolean, series: tx.policy.isNonstandard }],
]));

const FILTERS_BY_KEY = /** @type {Map<string, BlockPreviewFilter>} */ (
  new Map(FILTERS.map((filter) => [filter.key, filter]))
);
const INSPECT_SERIES = [...new Map(
  [...MEMBERSHIP_SOURCES.values()].map(({ series }) => [series.name, series]),
).values()];

/** @param {string} key */
function filterIndex(key) {
  return /** @type {BlockPreviewFilter} */ (FILTERS_BY_KEY.get(key)).index;
}

/**
 * @param {number} height
 * @param {BlockPreviewRange} range
 * @param {AbortSignal} signal
 */
export function createBlockPreviewFilterData(height, range, signal) {
  const opReturn = createOpReturnFilterData(height, range, signal);
  const sourcePromises = new Map();
  const membershipPromises = new Map();
  let countsPromise = /** @type {Promise<Uint32Array> | null} */ (null);

  async function fetchCounts() {
    const response = await Promise.all(COUNT_SOURCES.map(({ series }) => {
      return series.by.height.get(height).fetch({ signal });
    }));
    const counts = new Uint32Array(FILTERS.length);

    for (let index = 0; index < COUNT_SOURCES.length; index += 1) {
      counts[filterIndex(COUNT_SOURCES[index].key)] = Number(response[index].data[0]);
    }

    const txCount = range.end - range.start;
    counts[filterIndex("rbf:no")] = txCount - counts[filterIndex("rbf:yes")];
    counts[filterIndex("input:multi")] =
      txCount - counts[filterIndex("input:one")];
    counts[filterIndex("output:multi")] =
      txCount - counts[filterIndex("output:one")];

    signal.throwIfAborted();

    return counts;
  }

  function loadCounts() {
    countsPromise ??= fetchCounts();

    return countsPromise;
  }

  /** @param {BlockPreviewFilterSource["series"]} series */
  function loadSource(series) {
    let promise = sourcePromises.get(series.name);

    if (promise === undefined) {
      promise = series.by.tx_index
        .skip(range.start)
        .take(range.end - range.start)
        .fetch({ signal, memCache: false })
        .then(({ data }) => data);
      sourcePromises.set(series.name, promise);
    }

    return /** @type {Promise<unknown[]>} */ (promise);
  }

  /** @param {BlockPreviewFilter} filter */
  function loadMembership(filter) {
    let promise = membershipPromises.get(filter.key);

    if (promise === undefined) {
      if (filter.group === "op_return") {
        promise = opReturn.loadMembership(filter.key);
      } else {
        const source = /** @type {BlockPreviewFilterSource} */ (
          MEMBERSHIP_SOURCES.get(filter.key)
        );
        promise = loadSource(source.series).then((values) => {
          const membership = new Uint8Array(values.length);

          for (let index = 0; index < values.length; index += 1) {
            membership[index] = Number(source.matches(values[index]));
          }

          return membership;
        });
      }

      membershipPromises.set(filter.key, promise);
    }

    return /** @type {Promise<Uint8Array>} */ (promise);
  }

  /**
   * @param {number} txIndex
   * @param {AbortSignal} requestSignal
   */
  async function loadTransactionFilters(txIndex, requestSignal) {
    const response = await Promise.all(INSPECT_SERIES.map((series) => {
      return series.by.tx_index.get(txIndex).fetch({ signal: requestSignal });
    }));
    const values = new Map(INSPECT_SERIES.map((series, index) => {
      return [series.name, response[index].data[0]];
    }));
    const keys = /** @type {string[]} */ ([]);

    for (const [key, source] of MEMBERSHIP_SOURCES) {
      if (source.matches(values.get(source.series.name))) keys.push(key);
    }

    if (keys.includes("type:op_return")) {
      keys.push(...await opReturn.loadTransactionKeys(txIndex));
    }

    requestSignal.throwIfAborted();

    return keys;
  }

  return /** @type {const} */ ({
    loadCounts,
    loadMembership,
    loadTransactionFilters,
  });
}

/** @typedef {import("../data.js").BlockPreviewRange} BlockPreviewRange */
/** @typedef {import("./model.js").BlockPreviewFilter} BlockPreviewFilter */
/** @typedef {ReturnType<typeof createBlockPreviewFilterData>} BlockPreviewFilterData */

/**
 * @typedef {Object} BlockPreviewFilterSource
 * @property {(value: any) => boolean} matches
 * @property {{
 *   name: string,
 *   by: {
 *     tx_index: {
 *       get: (index: number) => {
 *         fetch: (options: { signal: AbortSignal }) => Promise<{ data: unknown[] }>,
 *       },
 *       skip: (start: number) => {
 *         take: (count: number) => {
 *           fetch: (options: { signal: AbortSignal, memCache: false }) => Promise<{ data: unknown[] }>,
 *         },
 *       },
 *     },
 *   },
 * }} series
 */
