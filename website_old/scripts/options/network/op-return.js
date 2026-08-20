import { colors } from "../../utils/colors.js";
import { brk } from "../../utils/client.js";
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

/** @typedef {(typeof brk.series.opReturn.byKind)[keyof typeof brk.series.opReturn.byKind]} OpReturnMetrics */

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
  const opReturn = brk.series.opReturn;
  const outputCount = brk.series.outputs.byType.outputCount.opReturn;
  const types = TYPE_DEFINITIONS.map((type, index) => ({
    ...type,
    color: colors.at(index, TYPE_DEFINITIONS.length),
    pattern: opReturn.byKind[type.key],
  }));
  const policyClassifications = [
    {
      name: "Standard",
      color: colors.profit,
      defaultActive: true,
      pattern: opReturn.policy.preV30Standard,
    },
    {
      name: "Nonstandard",
      color: colors.loss,
      defaultActive: true,
      pattern: opReturn.policy.preV30Nonstandard,
    },
  ];
  const nonstandardReasons = [
    {
      name: "Over 82 Bytes",
      color: colors.loss,
      defaultActive: true,
      pattern: opReturn.policy.oversized,
    },
    {
      name: "Multiple Outputs",
      color: colors.bitcoin,
      defaultActive: true,
      pattern: opReturn.policy.multiple,
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
