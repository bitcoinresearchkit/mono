/**
 * Capitalization section builders
 */

import { Unit } from "../../utils/units.js";
import { colors } from "../../utils/colors.js";
import { ROLLING_WINDOWS, line, baseline, mapWindows, sumsTreeBaseline, rollingPercentRatioTree, percentRatio, percentRatioBaseline } from "../series.js";
import { ratioBottomSeries, mapCohortsWithAll, flatMapCohortsWithAll } from "../shared.js";
import { priceLine } from "../constants.js";

// ============================================================================
// Shared building blocks
// ============================================================================

/**
 * Single cohort: Change + Growth Rate items (flat)
 * @param {CohortWithRealizedCap["tree"]} tree
 * @param {(name: string) => string} title
 * @returns {PartialOptionsTree}
 */
function singleDeltaItems(tree, title) {
  return [
    { ...sumsTreeBaseline({ windows: mapWindows(tree.realized.cap.delta.absolute, (c) => c.usd), title, metric: "Realized Cap Change", unit: Unit.usd, legend: "Change" }), name: "Change" },
    { ...rollingPercentRatioTree({ windows: tree.realized.cap.delta.rate, title, metric: "Realized Cap Growth Rate" }), name: "Growth Rate" },
  ];
}

/**
 * Grouped: Change + Growth Rate items (flat)
 * @param {readonly CohortWithRealizedCap[]} list
 * @param {CohortAll} all
 * @param {(name: string) => string} title
 * @returns {PartialOptionsTree}
 */
function groupedDeltaItems(list, all, title) {
  return [
    {
      name: "Change",
      tree: ROLLING_WINDOWS.map((w) => ({
        name: w.name,
        title: title(`${w.title} Realized Cap Change`),
        bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
          baseline({ series: tree.realized.cap.delta.absolute[w.key].usd, name, color, unit: Unit.usd }),
        ),
      })),
    },
    {
      name: "Growth Rate",
      tree: ROLLING_WINDOWS.map((w) => ({
        name: w.name,
        title: title(`${w.title} Realized Cap Growth Rate`),
        bottom: flatMapCohortsWithAll(list, all, ({ name, color, tree }) =>
          percentRatioBaseline({ pattern: tree.realized.cap.delta.rate[w.key], name, color }),
        ),
      })),
    },
  ];
}

/**
 * Grouped: MVRV + Change + Growth Rate items (flat)
 * @param {readonly (UtxoCohortObject | CohortWithoutRelative)[]} list
 * @param {CohortAll} all
 * @param {(name: string) => string} title
 * @returns {PartialOptionsTree}
 */
function groupedDeltaAndMvrv(list, all, title) {
  return [
    {
      name: "MVRV",
      title: title("MVRV"),
      bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
        baseline({ series: tree.realized.mvrv, name, color, unit: Unit.ratio, base: 1 }),
      ),
    },
    ...groupedDeltaItems(list, all, title),
  ];
}

/**
 * @param {readonly CohortWithRealizedCap[]} list
 * @param {CohortAll} all
 * @param {(name: string) => string} title
 * @returns {PartialChartOption}
 */
function groupedRealizedCapTotal(list, all, title) {
  return {
    name: "Total",
    title: title("Realized Cap"),
    bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
      line({ series: tree.realized.cap.usd, name, color, unit: Unit.usd }),
    ),
  };
}

// ============================================================================
// Single Cohort Sections
// ============================================================================

/**
 * Full capitalization (has invested capital, own market cap ratio, full MVRV)
 * @param {{ cohort: CohortAll | CohortFull | CohortLongTerm, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createValuationSectionFull({ cohort, title }) {
  const { tree, color } = cohort;
  return {
    name: "Capitalization",
    tree: [
      { name: "Total", title: title("Realized Cap"), bottom: [line({ series: tree.realized.cap.usd, name: "Realized Cap", color, unit: Unit.usd })] },
      {
        name: "Profitability",
        tree: [
          {
            name: "Amount",
            title: title("Invested Capital"),
            bottom: [
              line({ series: tree.realized.cap.usd, name: "Total", color: colors.default, unit: Unit.usd }),
              line({ series: tree.unrealized.investedCapitalInProfit.usd, name: "In Profit", color: colors.profit, unit: Unit.usd }),
              line({ series: tree.unrealized.investedCapitalInLoss.usd, name: "In Loss", color: colors.loss, unit: Unit.usd }),
            ],
          },
          {
            name: "Composition",
            title: title("Invested Capital Composition"),
            bottom: [
              ...percentRatio({ pattern: tree.relative.investedCapital.inProfit.share, name: "In Profit", color: colors.profit }),
              ...percentRatio({ pattern: tree.relative.investedCapital.inLoss.share, name: "In Loss", color: colors.loss }),
              priceLine({ number: 100, color: colors.default, style: 0, unit: Unit.percentage }),
              priceLine({ number: 50, unit: Unit.percentage }),
            ],
          },
        ],
      },
      { name: "MVRV", title: title("MVRV"), bottom: ratioBottomSeries(tree.realized.price) },
      ...singleDeltaItems(tree, title),
    ],
  };
}

/**
 * Capitalization without MVRV.
 * @param {{ cohort: CohortWithRealizedCap, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createValuationSectionBase({ cohort, title }) {
  const { tree } = cohort;
  return {
    name: "Capitalization",
    tree: [
      { name: "Total", title: title("Realized Cap"), bottom: [line({ series: tree.realized.cap.usd, name: "Realized Cap", color: cohort.color, unit: Unit.usd })] },
      ...singleDeltaItems(tree, title),
    ],
  };
}

/**
 * Basic capitalization (no invested capital, simple MVRV)
 * @param {{ cohort: CohortCore | CohortBasic | CohortAddr | CohortWithoutRelative, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createValuationSection({ cohort, title }) {
  const base = createValuationSectionBase({ cohort, title });
  return {
    ...base,
    tree: [
      ...base.tree,
      { name: "MVRV", title: title("MVRV"), bottom: [baseline({ series: cohort.tree.realized.mvrv, name: "MVRV", unit: Unit.ratio, base: 1 })] },
    ],
  };
}

// ============================================================================
// Grouped Cohort Sections
// ============================================================================

/**
 * @param {{ list: readonly CohortWithRealizedCap[], all: CohortAll, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createGroupedValuationSectionBase({ list, all, title }) {
  return {
    name: "Capitalization",
    tree: [
      groupedRealizedCapTotal(list, all, title),
      ...groupedDeltaItems(list, all, title),
    ],
  };
}

/**
 * @param {{ list: readonly (UtxoCohortObject | CohortWithoutRelative)[], all: CohortAll, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createGroupedValuationSection({ list, all, title }) {
  return {
    name: "Capitalization",
    tree: [
      groupedRealizedCapTotal(list, all, title),
      ...groupedDeltaAndMvrv(list, all, title),
    ],
  };
}

/**
 * @param {{ list: readonly (CohortAll | CohortFull | CohortLongTerm)[], all: CohortAll, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createGroupedValuationSectionWithOwnMarketCap({ list, all, title }) {
  return {
    name: "Capitalization",
    tree: [
      {
        name: "Total",
        title: title("Realized Cap"),
        bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
          line({ series: tree.realized.cap.usd, name, color, unit: Unit.usd }),
        ),
      },
      {
        name: "In Profit",
        title: title("Invested Capital In Profit"),
        bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
          line({ series: tree.unrealized.investedCapitalInProfit.usd, name, color, unit: Unit.usd }),
        ),
      },
      {
        name: "In Loss",
        title: title("Invested Capital In Loss"),
        bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
          line({ series: tree.unrealized.investedCapitalInLoss.usd, name, color, unit: Unit.usd }),
        ),
      },
      ...groupedDeltaAndMvrv(list, all, title),
    ],
  };
}
