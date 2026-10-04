/**
 * Activity section builders
 *
 * Capabilities by cohort type:
 * - All/STH: activity (full), SOPR (rolling + adjusted), sell side risk, value (flows + breakdown), coins
 * - LTH: activity (full), SOPR (rolling), sell side risk, value (flows + breakdown), coins
 * - Core/AgeRange: activity, SOPR (24h only), value, coins
 * - Others (UtxoAmount, Empty, Address): no activity, value only
 */

import { Unit } from "../../utils/units.js";
import {
  line,
  baseline,
  percentRatio,
  chartsFromCount,
  averagesArray,
  mapWindows,
  ROLLING_WINDOWS,
} from "../series.js";
import {
  satsBtcUsd,
  satsBtcUsdFullTree,
  mapCohortsWithAll,
  groupedWindowsCumulativeWithAll,
  groupedWindowsCumulativeSatsBtcUsd,
} from "../shared.js";
import { colors } from "../../utils/colors.js";
import { lazyGroup } from "../lazy.js";

// ============================================================================
// Shared Volume Helpers
// ============================================================================

/**
 * @param {FullActivityPattern | AgeRangePattern["activity"] | BasicUtxoPattern["activity"]} activity
 * @param {Color} color
 * @param {(name: string) => string} title
 * @returns {PartialOptionsTree}
 */
function volumeTree(activity, color, title) {
  const tv = activity.transferVolume;
  const inProfit = "transferVolumeInProfit" in activity ? activity.transferVolumeInProfit : activity.transferVolume.inProfit;
  const inLoss = "transferVolumeInLoss" in activity ? activity.transferVolumeInLoss : activity.transferVolume.inLoss;
  return [
    ...satsBtcUsdFullTree({
      pattern: tv,
      title,
      metric: "Transfer Volume",
      color,
    }),
    {
      name: "Profitability",
        tree: [
          ...ROLLING_WINDOWS.map((w) => ({
            name: w.name,
            title: title(`${w.title} Transfer Volume Profitability`),
            bottom: [
              ...satsBtcUsd({
                pattern: inProfit.sum[w.key],
                name: "In Profit",
                color: colors.profit,
              }),
              ...satsBtcUsd({
                pattern: inLoss.sum[w.key],
                name: "In Loss",
                color: colors.loss,
              }),
            ],
          })),
          {
            name: "Cumulative",
            title: title("Cumulative Transfer Volume Profitability"),
            bottom: [
              ...satsBtcUsd({
                pattern: inProfit.cumulative,
                name: "In Profit",
                color: colors.profit,
              }),
              ...satsBtcUsd({
                pattern: inLoss.cumulative,
                name: "In Loss",
                color: colors.loss,
              }),
            ],
          },
          {
            name: "In Profit",
            tree: satsBtcUsdFullTree({
              pattern: inProfit,
              title,
              metric: "Transfer Volume In Profit",
              color: colors.profit,
            }),
          },
          {
            name: "In Loss",
            tree: satsBtcUsdFullTree({
              pattern: inLoss,
              title,
              metric: "Transfer Volume In Loss",
              color: colors.loss,
            }),
          },
        ],
      },
  ];
}

/**
 * @param {FullActivityPattern | AgeRangePattern["activity"] | BasicUtxoPattern["activity"]} activity
 * @param {Color} color
 * @param {(name: string) => string} title
 * @returns {PartialOptionsGroup}
 */
function volumeFolder(activity, color, title) {
  return { name: "Volume", tree: volumeTree(activity, color, title) };
}

/**
 * @param {FullActivityPattern | AgeRangePattern["activity"] | BasicUtxoPattern["activity"]} activity
 * @param {AllUtxoPattern["ratios"]["adjustedSopr"]["transferVolume"]} adjustedTransferVolume
 * @param {Color} color
 * @param {(name: string) => string} title
 * @returns {PartialOptionsGroup}
 */
function volumeFolderWithAdjusted(activity, adjustedTransferVolume, color, title) {
  return {
    name: "Volume",
    tree: [
      ...volumeTree(activity, color, title),
      { name: "Adjusted", tree: fiatCountTree(adjustedTransferVolume, title, "Adjusted Transfer Volume") },
    ],
  };
}

// ============================================================================
// Shared SOPR Helpers
// ============================================================================

/**
 * @param {PatternAll["ratios"] | PatternFull["ratios"] | LongTermPattern["ratios"]} ratios
 */
function soprWindows(ratios) {
  return {
    _24h: ratios.sopr,
    _1w: ratios.soprRatioExtended._1w,
    _1m: ratios.soprRatioExtended._1m,
    _1y: ratios.soprRatioExtended._1y,
  };
}

/**
 * @param {RollingWindowPattern} ratio
 * @param {(name: string) => string} title
 * @param {string} [prefix]
 * @returns {PartialOptionsTree}
 */
function singleRollingSoprTree(ratio, title, prefix = "") {
  return [
    {
      name: "Compare",
      title: title(`${prefix}SOPR`),
      bottom: ROLLING_WINDOWS.map((w) =>
        baseline({
          series: ratio[w.key],
          name: w.name,
          color: w.color,
          unit: Unit.ratio,
          base: 1,
        }),
      ),
    },
    ...ROLLING_WINDOWS.map((w) => ({
      name: w.name,
      title: title(`${w.title} ${prefix}SOPR`.trim()),
      bottom: [
        baseline({
          series: ratio[w.key],
          name: "SOPR",
          unit: Unit.ratio,
          base: 1,
        }),
      ],
    })),
  ];
}

/**
 * @param {{ valueDestroyed: AllUtxoPattern["realized"]["valueDestroyed"], title: (name: string) => string }} args
 * @returns {PartialOptionsTree}
 */
function valueDestroyedTree({ valueDestroyed, title }) {
  return fiatCountTree(valueDestroyed, title, "Value Destroyed");
}

/**
 * @param {AllUtxoPattern["realized"]["valueDestroyed"]} pattern
 * @param {(name: string) => string} title
 * @param {string} metric
 * @returns {PartialOptionsTree}
 */
function fiatCountTree(pattern, title, metric) {
  return chartsFromCount({
    pattern: {
      sum: mapWindows(pattern.sum, (value) => value.usd),
      average: mapWindows(pattern.average, (value) => value.usd),
      cumulative: pattern.cumulative.usd,
    },
    title,
    metric,
    unit: Unit.usd,
  });
}

/**
 * @param {{ valueDestroyed: AllUtxoPattern["realized"]["valueDestroyed"], title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
function valueDestroyedFolder({ valueDestroyed, title }) {
  return {
    name: "Value Destroyed",
    tree: valueDestroyedTree({ valueDestroyed, title }),
  };
}

/**
 * @param {{ valueDestroyed: AllUtxoPattern["realized"]["valueDestroyed"], adjusted: AllUtxoPattern["ratios"]["adjustedSopr"]["valueDestroyed"], title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
function valueDestroyedFolderWithAdjusted({ valueDestroyed, adjusted, title }) {
  return {
    name: "Value Destroyed",
    tree: [
      ...valueDestroyedTree({ valueDestroyed, title }),
      { name: "Adjusted", tree: fiatCountTree(adjusted, title, "Adjusted Value Destroyed") },
    ],
  };
}

// ============================================================================
// Shared Sell Side Risk Helpers
// ============================================================================

/**
 * @param {PatternAll["ratios"]["sellSideRiskRatio"] | PatternFull["ratios"]["sellSideRiskRatio"] | LongTermPattern["ratios"]["sellSideRiskRatio"]} sellSideRisk
 * @param {(name: string) => string} title
 * @returns {PartialOptionsTree}
 */
function singleSellSideRiskTree(sellSideRisk, title) {
  return [
    {
      name: "Compare",
      title: title("Sell Side Risk"),
      bottom: ROLLING_WINDOWS.flatMap((w) =>
        percentRatio({
          pattern: sellSideRisk[w.key],
          name: w.name,
          color: w.color,
        }),
      ),
    },
    ...ROLLING_WINDOWS.map((w) => ({
      name: w.name,
      title: title(`${w.title} Sell Side Risk`),
      bottom: percentRatio({
        pattern: sellSideRisk[w.key],
        name: "Sell Side Risk",
        color: w.color,
      }),
    })),
  ];
}

// ============================================================================
// Single Cohort Activity Sections
// ============================================================================

/**
 * Single activity tree items shared by adjusted and unadjusted full cohorts
 * @param {CohortAll | CohortFull | CohortLongTerm} cohort
 * @param {(name: string) => string} title
 * @param {{ volume: () => PartialOptionsGroup, sopr: () => PartialOptionsGroup, valueDestroyed: () => PartialOptionsGroup }} create
 * @returns {PartialOptionsTree}
 */
function singleFullActivityTree(cohort, title, create) {
  const { tree, color } = cohort;
  return [
    lazyGroup("Volume", create.volume),
    lazyGroup("SOPR", create.sopr),
    lazyGroup("Value Destroyed", create.valueDestroyed),
    lazyGroup("Coindays Destroyed", () => ({
      name: "Coindays Destroyed",
      tree: chartsFromCount({
        pattern: tree.activity.coindaysDestroyed,
        title,
        metric: "Coindays Destroyed",
        unit: Unit.coindays,
        color,
      }),
    })),
    lazyGroup("Dormancy", () => ({
      name: "Dormancy",
      tree: averagesArray({
        windows: tree.ratios.dormancy,
        title,
        metric: "Dormancy",
        unit: Unit.days,
      }),
    })),
    lazyGroup("Sell Side Risk", () => ({
      name: "Sell Side Risk",
      tree: singleSellSideRiskTree(tree.ratios.sellSideRiskRatio, title),
    })),
  ];
}

/** @param {{ cohort: CohortAll | CohortFull, title: (name: string) => string }} args */
export function createActivitySectionWithAdjusted({ cohort, title }) {
  const { tree, color } = cohort;
  return {
    name: "Activity",
    tree: singleFullActivityTree(cohort, title, {
      volume: () =>
        volumeFolderWithAdjusted(
          tree.activity,
          tree.ratios.adjustedSopr.transferVolume,
          color,
          title,
        ),
      sopr: () => ({
        name: "SOPR",
        tree: [
          ...singleRollingSoprTree(soprWindows(tree.ratios), title),
          {
            name: "Adjusted",
            tree: singleRollingSoprTree(
              tree.ratios.adjustedSopr.ratio,
              title,
              "Adjusted ",
            ),
          },
        ],
      }),
      valueDestroyed: () =>
        valueDestroyedFolderWithAdjusted({
          valueDestroyed: tree.realized.valueDestroyed,
          adjusted: tree.ratios.adjustedSopr.valueDestroyed,
          title,
        }),
    }),
  };
}

/** @param {{ cohort: CohortFull | CohortLongTerm, title: (name: string) => string }} args */
export function createActivitySection({ cohort, title }) {
  const { tree, color } = cohort;
  return {
    name: "Activity",
    tree: singleFullActivityTree(cohort, title, {
      volume: () => volumeFolder(tree.activity, color, title),
      sopr: () => ({
        name: "SOPR",
        tree: singleRollingSoprTree(soprWindows(tree.ratios), title),
      }),
      valueDestroyed: () =>
        valueDestroyedFolder({
          valueDestroyed: tree.realized.valueDestroyed,
          title,
        }),
    }),
  };
}

/**
 * Activity section for Core/AgeRange cohorts (24h SOPR only)
 * @param {{ cohort: CohortAgeRange | CohortCore, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createActivitySectionWithActivity({ cohort, title }) {
  const { tree, color } = cohort;
  return {
    name: "Activity",
    tree: [
      volumeFolder(tree.activity, color, title),
      valueDestroyedFolder({ valueDestroyed: tree.realized.valueDestroyed, title }),
      {
        name: "Coindays Destroyed",
        tree: chartsFromCount({
          pattern: tree.activity.coindaysDestroyed,
          title,
        metric: "Coindays Destroyed",
          unit: Unit.coindays,
          color,
        }),
      },
    ],
  };
}

/**
 * Minimal activity section: volume only
 * @param {{ cohort: CohortBasicWithMarketCap | CohortBasicWithoutMarketCap | CohortWithoutRelative | CohortAddr | AddrCohortObject, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createActivitySectionMinimal({ cohort, title }) {
  return {
    name: "Activity",
    tree: satsBtcUsdFullTree({
      pattern: cohort.tree.activity.transferVolume,
      title,
      metric: "Transfer Volume",
    }),
  };
}

/**
 * Grouped minimal activity: volume
 * @param {{ list: readonly (UtxoCohortObject | CohortWithoutRelative | CohortAddr | AddrCohortObject)[], all: CohortAll, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createGroupedActivitySectionMinimal({ list, all, title }) {
  return {
    name: "Activity",
    tree: groupedWindowsCumulativeSatsBtcUsd({
      list, all, title, metricTitle: "Transfer Volume",
      getMetric: (c) => c.tree.activity.transferVolume,
    }),
  };
}

/**
 * Grouped profitability folder (compare + in profit + in loss)
 * @template {{ name: string, color: Color }} T
 * @template {{ name: string, color: Color }} A
 * @param {readonly T[]} list
 * @param {A} all
 * @param {(name: string) => string} title
 * @param {(c: T | A) => { sum: Record<string, AnyValuePattern>, cumulative: AnyValuePattern }} getInProfit
 * @param {(c: T | A) => { sum: Record<string, AnyValuePattern>, cumulative: AnyValuePattern }} getInLoss
 * @returns {PartialOptionsTree}
 */
function groupedProfitabilityArray(list, all, title, getInProfit, getInLoss) {
  return [
    {
      name: "In Profit",
      tree: groupedWindowsCumulativeSatsBtcUsd({
        list,
        all,
        title,
        metricTitle: "Transfer Volume In Profit",
        getMetric: (c) => getInProfit(c),
      }),
    },
    {
      name: "In Loss",
      tree: groupedWindowsCumulativeSatsBtcUsd({
        list,
        all,
        title,
        metricTitle: "Transfer Volume In Loss",
        getMetric: (c) => getInLoss(c),
      }),
    },
  ];
}

/**
 * @template {{ name: string, color: Color }} T
 * @template {{ name: string, color: Color }} A
 * @param {readonly T[]} list
 * @param {A} all
 * @param {(name: string) => string} title
 * @param {(c: T | A) => { sum: Record<string, AnyValuePattern>, cumulative: AnyValuePattern, inProfit: { sum: Record<string, AnyValuePattern>, cumulative: AnyValuePattern }, inLoss: { sum: Record<string, AnyValuePattern>, cumulative: AnyValuePattern } }} getTransferVolume
 * @returns {PartialOptionsTree}
 */
function groupedVolumeTree(list, all, title, getTransferVolume) {
  return [
    ...groupedWindowsCumulativeSatsBtcUsd({
      list,
      all,
      title,
      metricTitle: "Transfer Volume",
      getMetric: (c) => getTransferVolume(c),
    }),
    ...groupedProfitabilityArray(
      list,
      all,
      title,
      (c) => getTransferVolume(c).inProfit,
      (c) => getTransferVolume(c).inLoss,
    ),
  ];
}

/**
 * @template {{ name: string, color: Color }} T
 * @template {{ name: string, color: Color }} A
 * @param {readonly T[]} list
 * @param {A} all
 * @param {(name: string) => string} title
 * @param {(c: T | A) => { sum: Record<string, AnyValuePattern>, cumulative: AnyValuePattern, inProfit: { sum: Record<string, AnyValuePattern>, cumulative: AnyValuePattern }, inLoss: { sum: Record<string, AnyValuePattern>, cumulative: AnyValuePattern } }} getTransferVolume
 * @returns {PartialOptionsGroup}
 */
function groupedVolumeFolder(list, all, title, getTransferVolume) {
  return { name: "Volume", tree: groupedVolumeTree(list, all, title, getTransferVolume) };
}

// ============================================================================
// Grouped SOPR Helpers
// ============================================================================

/**
 * @template {{ color: Color, name: string }} T
 * @template {{ color: Color, name: string }} A
 * @param {readonly T[]} list
 * @param {A} all
 * @param {(item: T | A) => { _24h: AnySeriesPattern, _1w: AnySeriesPattern, _1m: AnySeriesPattern, _1y: AnySeriesPattern }} getRatio
 * @param {(name: string) => string} title
 * @param {string} [prefix]
 * @returns {PartialOptionsTree}
 */
function groupedSoprCharts(list, all, getRatio, title, prefix = "") {
  return ROLLING_WINDOWS.map((w) => ({
    name: w.name,
    title: title(`${w.title} ${prefix}SOPR`.trim()),
    bottom: mapCohortsWithAll(list, all, (c) =>
      baseline({
        series: getRatio(c)[w.key],
        name: c.name,
        color: c.color,
        unit: Unit.ratio,
        base: 1,
      }),
    ),
  }));
}

/**
 * @template {{ name: string, color: Color }} T
 * @template {{ name: string, color: Color }} A
 * @param {{ list: readonly T[], all: A, title: (name: string) => string, getValueDestroyed: (c: T | A) => AllUtxoPattern["realized"]["valueDestroyed"] }} args
 * @returns {PartialOptionsTree}
 */
function groupedValueDestroyedTree({ list, all, title, getValueDestroyed }) {
  return groupedWindowsCumulativeWithAll({
    list, all, title, metricTitle: "Value Destroyed",
    getWindowSeries: (c, key) => getValueDestroyed(c).sum[key].usd,
    getCumulativeSeries: (c) => getValueDestroyed(c).cumulative.usd,
    seriesFn: line, unit: Unit.usd,
  });
}

/**
 * @template {{ name: string, color: Color }} T
 * @template {{ name: string, color: Color }} A
 * @param {{ list: readonly T[], all: A, title: (name: string) => string, getValueDestroyed: (c: T | A) => AllUtxoPattern["realized"]["valueDestroyed"] }} args
 * @returns {PartialOptionsGroup}
 */
function groupedValueDestroyedFolder({ list, all, title, getValueDestroyed }) {
  return {
    name: "Value Destroyed",
    tree: groupedValueDestroyedTree({ list, all, title, getValueDestroyed }),
  };
}

// ============================================================================
// Grouped Activity Sections
// ============================================================================

/**
 * Grouped activity tree items shared by adjusted and unadjusted full cohorts
 * @param {readonly (CohortFull | CohortLongTerm)[]} list
 * @param {CohortAll} all
 * @param {(name: string) => string} title
 * @param {PartialOptionsGroup} volumeItem
 * @param {PartialOptionsGroup} soprFolder
 * @param {PartialOptionsGroup} valueDestroyedItem
 * @returns {PartialOptionsTree}
 */
function groupedFullActivityTree(list, all, title, volumeItem, soprFolder, valueDestroyedItem) {
  return [
    volumeItem,
    soprFolder,
    valueDestroyedItem,
    ...groupedActivitySharedItems(list, all, title),
  ];
}

/** @param {{ list: readonly (CohortFull | CohortLongTerm)[], all: CohortAll, title: (name: string) => string }} args */
export function createGroupedActivitySection({ list, all, title }) {
  return {
    name: "Activity",
    tree: groupedFullActivityTree(list, all, title,
      groupedVolumeFolder(list, all, title, (c) => ({
        ...c.tree.activity.transferVolume,
        inProfit: c.tree.activity.transferVolumeInProfit,
        inLoss: c.tree.activity.transferVolumeInLoss,
      })),
      { name: "SOPR", tree: groupedSoprCharts(list, all, (c) => soprWindows(c.tree.ratios), title) },
      groupedValueDestroyedFolder({
        list,
        all,
        title,
        getValueDestroyed: (cohort) =>
          cohort.tree.realized.valueDestroyed,
      }),
    ),
  };
}

/**
 * Shared grouped activity items: coindays, dormancy, sell side risk
 * @param {readonly (CohortFull | CohortLongTerm)[]} list
 * @param {CohortAll} all
 * @param {(name: string) => string} title
 * @returns {PartialOptionsTree}
 */
function groupedActivitySharedItems(list, all, title) {
  return [
    {
      name: "Coindays Destroyed",
      tree: groupedWindowsCumulativeWithAll({
        list,
        all,
        title,
        metricTitle: "Coindays Destroyed",
        getWindowSeries: (c, key) => c.tree.activity.coindaysDestroyed.sum[key],
        getCumulativeSeries: (c) =>
          c.tree.activity.coindaysDestroyed.cumulative,
        seriesFn: line,
        unit: Unit.coindays,
      }),
    },
    {
      name: "Dormancy",
      tree: ROLLING_WINDOWS.map((w) => ({
        name: w.name,
        title: title(`${w.title} Dormancy`),
        bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
          line({
            series: tree.ratios.dormancy[w.key],
            name,
            color,
            unit: Unit.days,
          }),
        ),
      })),
    },
    {
      name: "Sell Side Risk",
      tree: ROLLING_WINDOWS.map((w) => ({
        name: w.name,
        title: title(`${w.title} Sell Side Risk`),
        bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
          line({
            series: tree.ratios.sellSideRiskRatio[w.key].ratio,
            name,
            color,
            unit: Unit.ratio,
          }),
        ),
      })),
    },
  ];
}

/**
 * Grouped activity for Core/AgeRange cohorts
 * @param {{ list: readonly (CohortAgeRange | CohortCore)[], all: CohortAll, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createGroupedActivitySectionWithActivity({ list, all, title }) {
  return {
    name: "Activity",
    tree: [
      groupedVolumeFolder(list, all, title, (c) => {
        const a = c.tree.activity;
        return "transferVolumeInProfit" in a ? {
          ...a.transferVolume,
          inProfit: a.transferVolumeInProfit,
          inLoss: a.transferVolumeInLoss,
        } : a.transferVolume;
      }),
      groupedValueDestroyedFolder({
        list,
        all,
        title,
        getValueDestroyed: (cohort) =>
          cohort.tree.realized.valueDestroyed,
      }),
      {
        name: "Coindays Destroyed",
        tree: [
          ...ROLLING_WINDOWS.map((w) => ({
            name: w.name,
            title: title(`${w.title} Coindays Destroyed`),
            bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
              line({
                series: tree.activity.coindaysDestroyed.sum[w.key],
                name,
                color,
                unit: Unit.coindays,
              }),
            ),
          })),
          {
            name: "Cumulative",
            title: title("Cumulative Coindays Destroyed"),
            bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
              line({
                series: tree.activity.coindaysDestroyed.cumulative,
                name,
                color,
                unit: Unit.coindays,
              }),
            ),
          },
        ],
      },
    ],
  };
}
