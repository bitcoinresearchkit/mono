import { colors } from "../../utils/colors.js";
import { bitview } from "../../utils/client.js";
import { Unit } from "../../utils/units.js";
import {
  ROLLING_WINDOWS,
  chartsFromCount,
  chartsFromPercentCumulative,
  line,
  percentRatio,
} from "../series.js";
import { groupedWindowsCumulative } from "../shared.js";

const TYPE_DEFINITIONS = /** @type {const} */ ([
  { key: "runes", name: "Runes", defaultActive: true },
  { key: "veriBlock", name: "VeriBlock", defaultActive: true },
  { key: "omni", name: "Omni", defaultActive: true },
  { key: "stacks", name: "Stacks", defaultActive: true },
  { key: "blockstack", name: "Blockstack", defaultActive: false },
  { key: "colu", name: "Colu", defaultActive: false },
  { key: "openAssets", name: "Open Assets", defaultActive: false },
  { key: "komodo", name: "Komodo", defaultActive: false },
  { key: "coinSpark", name: "CoinSpark", defaultActive: false },
  { key: "poet", name: "Po.et", defaultActive: false },
  { key: "docproof", name: "Docproof", defaultActive: false },
  { key: "openTimestamps", name: "OpenTimestamps", defaultActive: true },
  { key: "factom", name: "Factom", defaultActive: false },
  { key: "eternityWall", name: "Eternity Wall", defaultActive: false },
  { key: "memo", name: "Memo", defaultActive: false },
  { key: "bitproof", name: "Bitproof", defaultActive: false },
  { key: "ascribe", name: "Ascribe", defaultActive: false },
  { key: "stampery", name: "Stampery", defaultActive: false },
  { key: "epobc", name: "EPOBC", defaultActive: false },
  { key: "bareHash", name: "Bare Hash", defaultActive: false },
  { key: "text", name: "Text", defaultActive: true },
  { key: "empty", name: "Empty", defaultActive: false },
  { key: "unknown", name: "Unknown", defaultActive: true },
]);

const METRICS = /** @type {const} */ ([
  {
    key: "txCount",
    name: "Transactions",
    title: "Transaction Count",
    unit: Unit.count,
  },
  {
    key: "outputCount",
    name: "Outputs",
    title: "Output Count",
    unit: Unit.count,
  },
  {
    key: "dataBytes",
    name: "Data Bytes",
    title: "Data Bytes",
    unit: Unit.bytes,
  },
  {
    key: "txVsize",
    name: "Transaction vSize",
    title: "Transaction vSize",
    unit: Unit.vb,
  },
  {
    key: "fees",
    name: "Fees",
    title: "Transaction Fees",
    unit: Unit.sats,
  },
]);

/**
 * @typedef {Object} OpReturnMetrics
 * @property {typeof bitview.series.opReturn.byKind.outputCount.runes} outputCount
 * @property {typeof bitview.series.opReturn.byKind.dataBytes.runes} dataBytes
 * @property {typeof bitview.series.opReturn.byKind.txCount.runes} txCount
 * @property {typeof bitview.series.opReturn.byKind.txVsize.runes} txVsize
 * @property {typeof bitview.series.opReturn.byKind.fees.runes} fees
 * @property {typeof bitview.series.opReturn.byKind.dataBytes.runes.dataShare} dataShare
 * @property {typeof bitview.series.opReturn.byKind.dataBytes.runes.chainShare} chainShare
 * @property {typeof bitview.series.opReturn.byKind.fees.runes.feeShare} feeShare
 */

/**
 * @typedef {Object} OpReturnEntry
 * @property {string} name
 * @property {Color} color
 * @property {boolean} [defaultActive]
 * @property {OpReturnMetrics} pattern
 */

/**
 * @param {Object} args
 * @param {readonly OpReturnEntry[]} args.list
 * @param {string} args.category
 * @returns {PartialOptionsTree}
 */
function createComparisons({ list, category }) {
  return METRICS.map((metric) => ({
    name: metric.name,
    tree: groupedWindowsCumulative({
      list,
      title: (title) => `OP_RETURN ${title}`,
      metricTitle: `${metric.title} by ${category}`,
      getWindowSeries: (entry, window) =>
        entry.pattern[metric.key].sum[window],
      getCumulativeSeries: (entry) =>
        entry.pattern[metric.key].cumulative,
      seriesFn: line,
      unit: metric.unit,
    }),
  }));
}

/**
 * @param {Object} args
 * @param {OpReturnMetrics} args.pattern
 * @param {string} args.name
 * @param {Color} args.color
 * @returns {PartialOptionsTree}
 */
function createMetrics({ pattern, name, color }) {
  return METRICS.map((metric) => ({
    name: metric.name,
    tree: chartsFromCount({
      pattern: pattern[metric.key],
      metric: `OP_RETURN ${name} ${metric.title}`,
      unit: metric.unit,
      color,
    }),
  }));
}

/**
 * @param {Object} args
 * @param {readonly OpReturnEntry[]} args.list
 * @param {string} args.category
 * @param {string} [args.subject]
 * @returns {PartialOptionsTree}
 */
function createShares({ list, category, subject }) {
  const shares = /** @type {const} */ ([
    {
      key: "dataShare",
      name: "OP_RETURN Share",
      title: subject
        ? `Cumulative ${subject} Share of OP_RETURN Data`
        : `Cumulative OP_RETURN Data Share by ${category}`,
    },
    {
      key: "chainShare",
      name: "Blockchain Share",
      title: subject
        ? `Cumulative ${subject} OP_RETURN Data Share of Blockchain`
        : `Cumulative Blockchain Share by OP_RETURN ${category}`,
    },
  ]);

  return shares.map(({ key, name: shareName, title }) => ({
    name: shareName,
    title,
    bottom: list.flatMap(({ name, color, defaultActive, pattern }) =>
      percentRatio({
        pattern: pattern[key],
        name,
        color,
        defaultActive,
      }),
    ),
  }));
}

/**
 * @param {Object} args
 * @param {readonly OpReturnEntry[]} args.list
 * @param {string} args.category
 * @returns {PartialOptionsGroup}
 */
function createFeeShareComparison({ list, category }) {
  const metric = `OP_RETURN Share of Transaction Fees by ${category}`;
  return {
    name: "Fee Share",
    tree: [
      ...ROLLING_WINDOWS.map((window) => ({
        name: window.name,
        title: `${window.title} ${metric}`,
        bottom: list.flatMap(
          ({ name, color, defaultActive, pattern }) =>
            percentRatio({
              pattern: pattern.feeShare[window.key],
              name,
              color,
              defaultActive,
            }),
        ),
      })),
      {
        name: "Cumulative",
        title: `Cumulative ${metric}`,
        bottom: list.flatMap(
          ({ name, color, defaultActive, pattern }) =>
            percentRatio({
              pattern: pattern.feeShare,
              name,
              color,
              defaultActive,
            }),
        ),
      },
    ],
  };
}

/**
 * @param {Object} args
 * @param {OpReturnMetrics} args.pattern
 * @param {string} args.name
 * @param {Color} args.color
 * @returns {PartialOptionsGroup}
 */
function createFeeShare({ pattern, name, color }) {
  return {
    name: "Fee Share",
    tree: chartsFromPercentCumulative({
      pattern: pattern.feeShare,
      metric: `OP_RETURN ${name} Share of Transaction Fees`,
      color,
    }),
  };
}

/**
 * @param {Object} args
 * @param {readonly OpReturnEntry[]} args.list
 * @param {string} args.name
 * @param {string} args.category
 * @returns {PartialOptionsGroup}
 */
function createBreakdown({ name, list, category }) {
  return {
    name,
    tree: [
      {
        name: "Compare",
        tree: [
          ...createComparisons({ list, category }),
          createFeeShareComparison({ list, category }),
          ...createShares({ list, category }),
        ],
      },
      ...list.map(({ name, color, pattern }) => ({
        name,
        tree: [
          ...createMetrics({ pattern, name, color }),
          createFeeShare({ pattern, name, color }),
          ...createShares({
            list: [{ name, color, pattern }],
            category,
            subject: name,
          }),
        ],
      })),
    ],
  };
}

/** @returns {PartialOptionsGroup} */
export function createOpReturnSection() {
  const opReturn = bitview.series.opReturn;
  const outputCount = bitview.series.outputs.byType.outputCount.opReturn;
  const types = TYPE_DEFINITIONS.map((type, index) => ({
    ...type,
    color: colors.at(index, TYPE_DEFINITIONS.length),
    pattern: {
      outputCount: opReturn.byKind.outputCount[type.key],
      dataBytes: opReturn.byKind.dataBytes[type.key],
      txCount: opReturn.byKind.txCount[type.key],
      txVsize: opReturn.byKind.txVsize[type.key],
      fees: opReturn.byKind.fees[type.key],
      dataShare: opReturn.byKind.dataBytes[type.key].dataShare,
      chainShare: opReturn.byKind.dataBytes[type.key].chainShare,
      feeShare: opReturn.byKind.fees[type.key].feeShare,
    },
  }));
  const policyClassifications = [
    {
      name: "Standard",
      color: colors.profit,
      defaultActive: true,
      pattern: {
        outputCount: opReturn.policy.outputCount.preV30Standard,
        dataBytes: opReturn.policy.dataBytes.preV30Standard,
        txCount: opReturn.policy.txCount.preV30Standard,
        txVsize: opReturn.policy.txVsize.preV30Standard,
        fees: opReturn.policy.fees.preV30Standard,
        dataShare: opReturn.policy.dataBytes.preV30Standard.dataShare,
        chainShare: opReturn.policy.dataBytes.preV30Standard.chainShare,
        feeShare: opReturn.policy.fees.preV30Standard.feeShare,
      },
    },
    {
      name: "Nonstandard",
      color: colors.loss,
      defaultActive: true,
      pattern: {
        outputCount: opReturn.policy.outputCount.preV30Nonstandard,
        dataBytes: opReturn.policy.dataBytes.preV30Nonstandard,
        txCount: opReturn.policy.txCount.preV30Nonstandard,
        txVsize: opReturn.policy.txVsize.preV30Nonstandard,
        fees: opReturn.policy.fees.preV30Nonstandard,
        dataShare: opReturn.policy.dataBytes.preV30Nonstandard.dataShare,
        chainShare: opReturn.policy.dataBytes.preV30Nonstandard.chainShare,
        feeShare: opReturn.policy.fees.preV30Nonstandard.feeShare,
      },
    },
  ];
  const nonstandardReasons = [
    {
      name: "Over 82 Bytes",
      color: colors.loss,
      defaultActive: true,
      pattern: {
        outputCount: opReturn.policy.outputCount.oversized,
        dataBytes: opReturn.policy.dataBytes.oversized,
        txCount: opReturn.policy.txCount.oversized,
        txVsize: opReturn.policy.txVsize.oversized,
        fees: opReturn.policy.fees.oversized,
        dataShare: opReturn.policy.dataBytes.oversized.dataShare,
        chainShare: opReturn.policy.dataBytes.oversized.chainShare,
        feeShare: opReturn.policy.fees.oversized.feeShare,
      },
    },
    {
      name: "Multiple Outputs",
      color: colors.bitcoin,
      defaultActive: true,
      pattern: {
        outputCount: opReturn.policy.outputCount.multiple,
        dataBytes: opReturn.policy.dataBytes.multiple,
        txCount: opReturn.policy.txCount.multiple,
        txVsize: opReturn.policy.txVsize.multiple,
        fees: opReturn.policy.fees.multiple,
        dataShare: opReturn.policy.dataBytes.multiple.dataShare,
        chainShare: opReturn.policy.dataBytes.multiple.chainShare,
        feeShare: opReturn.policy.fees.multiple.feeShare,
      },
    },
  ];

  return {
    name: "OP_RETURN",
    tree: [
      {
        name: "Transactions",
        tree: chartsFromCount({
          pattern: opReturn.total.txCount,
          metric: "OP_RETURN Transaction Count",
          unit: Unit.count,
          color: colors.scriptType.opReturn,
        }),
      },
      {
        name: "Outputs",
        tree: chartsFromCount({
          pattern: outputCount,
          metric: "OP_RETURN Output Count",
          unit: Unit.count,
          color: colors.scriptType.opReturn,
        }),
      },
      {
        name: "Data Bytes",
        tree: chartsFromCount({
          pattern: opReturn.total.dataBytes,
          metric: "OP_RETURN Data Bytes",
          unit: Unit.bytes,
          color: colors.scriptType.opReturn,
        }),
      },
      {
        name: "Transaction vSize",
        tree: chartsFromCount({
          pattern: opReturn.total.txVsize,
          metric: "OP_RETURN Transaction vSize",
          unit: Unit.vb,
          color: colors.scriptType.opReturn,
        }),
      },
      {
        name: "Fees",
        tree: chartsFromCount({
          pattern: opReturn.total.fees,
          metric: "OP_RETURN Transaction Fees",
          unit: Unit.sats,
          color: colors.scriptType.opReturn,
        }),
      },
      {
        name: "Fee Share",
        tree: chartsFromPercentCumulative({
          pattern: opReturn.total.feeShare,
          metric: "OP_RETURN Share of Transaction Fees",
          color: colors.scriptType.opReturn,
        }),
      },
      {
        name: "Blockchain Share",
        title: "Cumulative OP_RETURN Data Share of Blockchain Size",
        bottom: percentRatio({
          pattern: opReturn.total.chainShare,
          name: "OP_RETURN Data",
          color: colors.scriptType.opReturn,
        }),
      },
      createBreakdown({ name: "Types", list: types, category: "Type" }),
      {
        name: "Pre-v30 Policy",
        tree: [
          createBreakdown({
            name: "Classification",
            list: policyClassifications,
            category: "Pre-v30 Policy",
          }),
          createBreakdown({
            name: "Nonstandard Reasons",
            list: nonstandardReasons,
            category: "Pre-v30 Nonstandard Reason",
          }),
        ],
      },
    ],
  };
}
