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
  dotsBaseline,
  percentRatio,
  chartsFromCount,
  averagesArray,
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

// ============================================================================
// Shared Volume Helpers
// ============================================================================

/**
 * @param {TransferVolumePattern} tv
 * @param {Color} color
 * @param {(name: string) => string} title
 * @returns {PartialOptionsTree}
 */
function volumeTree(tv, color, title) {
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
                pattern: tv.inProfit.sum[w.key],
                name: "In Profit",
                color: colors.profit,
              }),
              ...satsBtcUsd({
                pattern: tv.inLoss.sum[w.key],
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
                pattern: tv.inProfit.cumulative,
                name: "In Profit",
                color: colors.profit,
              }),
              ...satsBtcUsd({
                pattern: tv.inLoss.cumulative,
                name: "In Loss",
                color: colors.loss,
              }),
            ],
          },
          {
            name: "In Profit",
            tree: satsBtcUsdFullTree({
              pattern: tv.inProfit,
              title,
              metric: "Transfer Volume In Profit",
              color: colors.profit,
            }),
          },
          {
            name: "In Loss",
            tree: satsBtcUsdFullTree({
              pattern: tv.inLoss,
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
 * @param {{ transferVolume: TransferVolumePattern }} activity
 * @param {Color} color
 * @param {(name: string) => string} title
 * @returns {PartialOptionsGroup}
 */
function volumeFolder(activity, color, title) {
  return { name: "Volume", tree: volumeTree(activity.transferVolume, color, title) };
}

/**
 * @param {{ transferVolume: TransferVolumePattern }} activity
 * @param {CountPattern<number>} adjustedTransferVolume
 * @param {Color} color
 * @param {(name: string) => string} title
 * @returns {PartialOptionsGroup}
 */
function volumeFolderWithAdjusted(activity, adjustedTransferVolume, color, title) {
  return {
    name: "Volume",
    tree: [
      ...volumeTree(activity.transferVolume, color, title),
      { name: "Adjusted", tree: chartsFromCount({ pattern: adjustedTransferVolume, title, metric: "Adjusted Transfer Volume", unit: Unit.usd }) },
    ],
  };
}

// ============================================================================
// Shared SOPR Helpers
// ============================================================================

/**
 * @param {RollingWindowPattern<number>} ratio
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
 * @param {CountPattern<number>} valueDestroyed
 * @param {(name: string) => string} title
 * @returns {PartialOptionsTree}
 */
function valueDestroyedTree(valueDestroyed, title) {
  return chartsFromCount({ pattern: valueDestroyed, title, metric: "Value Destroyed", unit: Unit.usd });
}

/**
 * @param {CountPattern<number>} valueDestroyed
 * @param {(name: string) => string} title
 * @returns {PartialOptionsGroup}
 */
function valueDestroyedFolder(valueDestroyed, title) {
  return { name: "Value Destroyed", tree: valueDestroyedTree(valueDestroyed, title) };
}

/**
 * @param {CountPattern<number>} valueDestroyed
 * @param {CountPattern<number>} adjusted
 * @param {(name: string) => string} title
 * @returns {PartialOptionsGroup}
 */
function valueDestroyedFolderWithAdjusted(valueDestroyed, adjusted, title) {
  return {
    name: "Value Destroyed",
    tree: [
      ...valueDestroyedTree(valueDestroyed, title),
      { name: "Adjusted", tree: chartsFromCount({ pattern: adjusted, title, metric: "Adjusted Value Destroyed", unit: Unit.usd }) },
    ],
  };
}

// ============================================================================
// Shared Sell Side Risk Helpers
// ============================================================================

/**
 * @param {SellSideRiskPattern} sellSideRisk
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
 * @param {PartialOptionsGroup} volumeItem
 * @param {PartialOptionsGroup} soprFolder
 * @param {PartialOptionsGroup} valueDestroyedItem
 * @returns {PartialOptionsTree}
 */
function singleFullActivityTree(cohort, title, volumeItem, soprFolder, valueDestroyedItem) {
  const { tree, color } = cohort;
  return [
    volumeItem,
    soprFolder,
    valueDestroyedItem,
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
    {
      name: "Dormancy",
      tree: averagesArray({
        windows: tree.activity.dormancy,
        title,
        metric: "Dormancy",
        unit: Unit.days,
      }),
    },
    {
      name: "Sell Side Risk",
      tree: singleSellSideRiskTree(tree.realized.sellSideRiskRatio, title),
    },
  ];
}

/** @param {{ cohort: CohortAll | CohortFull, title: (name: string) => string }} args */
export function createActivitySectionWithAdjusted({ cohort, title }) {
  const { tree, color } = cohort;
  const sopr = tree.realized.sopr;
  return {
    name: "Activity",
    tree: singleFullActivityTree(cohort, title,
      volumeFolderWithAdjusted(tree.activity, sopr.adjusted.transferVolume, color, title),
      {
        name: "SOPR",
        tree: [
          ...singleRollingSoprTree(sopr.ratio, title),
          { name: "Adjusted", tree: singleRollingSoprTree(sopr.adjusted.ratio, title, "Adjusted ") },
        ],
      },
      valueDestroyedFolderWithAdjusted(sopr.valueDestroyed, sopr.adjusted.valueDestroyed, title),
    ),
  };
}

/** @param {{ cohort: CohortFull | CohortLongTerm, title: (name: string) => string }} args */
export function createActivitySection({ cohort, title }) {
  const { tree, color } = cohort;
  return {
    name: "Activity",
    tree: singleFullActivityTree(cohort, title,
      volumeFolder(tree.activity, color, title),
      { name: "SOPR", tree: singleRollingSoprTree(tree.realized.sopr.ratio, title) },
      valueDestroyedFolder(tree.realized.sopr.valueDestroyed, title),
    ),
  };
}

/**
 * Activity section for Core/AgeRange cohorts (24h SOPR only)
 * @param {{ cohort: CohortAgeRange | CohortCore, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createActivitySectionWithActivity({ cohort, title }) {
  const { tree, color } = cohort;
  const sopr = tree.realized.sopr;

  return {
    name: "Activity",
    tree: [
      volumeFolder(tree.activity, color, title),
      {
        name: "SOPR",
        title: title("SOPR (24h)"),
        bottom: [
          dotsBaseline({
            series: sopr.ratio._24h,
            name: "SOPR",
            unit: Unit.ratio,
            base: 1,
          }),
        ],
      },
      valueDestroyedFolder(sopr.valueDestroyed, title),
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

/**
 * @template {{ name: string, color: Color }} T
 * @template {{ name: string, color: Color }} A
 * @param {readonly T[]} list
 * @param {A} all
 * @param {(name: string) => string} title
 * @param {(c: T | A) => { sum: Record<string, AnyValuePattern>, cumulative: AnyValuePattern, inProfit: { sum: Record<string, AnyValuePattern>, cumulative: AnyValuePattern }, inLoss: { sum: Record<string, AnyValuePattern>, cumulative: AnyValuePattern } }} getTransferVolume
 * @param {(c: T | A) => CountPattern<number>} getAdjustedTransferVolume
 * @returns {PartialOptionsGroup}
 */
function groupedVolumeFolderWithAdjusted(list, all, title, getTransferVolume, getAdjustedTransferVolume) {
  return {
    name: "Volume",
    tree: [
      ...groupedVolumeTree(list, all, title, getTransferVolume),
      {
        name: "Adjusted",
        tree: groupedWindowsCumulativeWithAll({
          list, all, title, metricTitle: "Adjusted Transfer Volume",
          getWindowSeries: (c, key) => getAdjustedTransferVolume(c).sum[key],
          getCumulativeSeries: (c) => getAdjustedTransferVolume(c).cumulative,
          seriesFn: line, unit: Unit.usd,
        }),
      },
    ],
  };
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
 * @param {readonly T[]} list
 * @param {A} all
 * @param {(name: string) => string} title
 * @param {(c: T | A) => CountPattern<number>} getValueDestroyed
 * @returns {PartialOptionsTree}
 */
function groupedValueDestroyedTree(list, all, title, getValueDestroyed) {
  return groupedWindowsCumulativeWithAll({
    list, all, title, metricTitle: "Value Destroyed",
    getWindowSeries: (c, key) => getValueDestroyed(c).sum[key],
    getCumulativeSeries: (c) => getValueDestroyed(c).cumulative,
    seriesFn: line, unit: Unit.usd,
  });
}

/**
 * @template {{ name: string, color: Color }} T
 * @template {{ name: string, color: Color }} A
 * @param {readonly T[]} list
 * @param {A} all
 * @param {(name: string) => string} title
 * @param {(c: T | A) => CountPattern<number>} getValueDestroyed
 * @returns {PartialOptionsGroup}
 */
function groupedValueDestroyedFolder(list, all, title, getValueDestroyed) {
  return { name: "Value Destroyed", tree: groupedValueDestroyedTree(list, all, title, getValueDestroyed) };
}

/**
 * @template {{ name: string, color: Color }} T
 * @template {{ name: string, color: Color }} A
 * @param {readonly T[]} list
 * @param {A} all
 * @param {(name: string) => string} title
 * @param {(c: T | A) => CountPattern<number>} getValueDestroyed
 * @param {(c: T | A) => CountPattern<number>} getAdjustedValueDestroyed
 * @returns {PartialOptionsGroup}
 */
function groupedValueDestroyedFolderWithAdjusted(list, all, title, getValueDestroyed, getAdjustedValueDestroyed) {
  return {
    name: "Value Destroyed",
    tree: [
      ...groupedValueDestroyedTree(list, all, title, getValueDestroyed),
      { name: "Adjusted", tree: groupedValueDestroyedTree(list, all, title, getAdjustedValueDestroyed) },
    ],
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

/** @param {{ list: readonly CohortFull[], all: CohortAll, title: (name: string) => string }} args */
export function createGroupedActivitySectionWithAdjusted({ list, all, title }) {
  return {
    name: "Activity",
    tree: groupedFullActivityTree(list, all, title,
      groupedVolumeFolderWithAdjusted(list, all, title, (c) => c.tree.activity.transferVolume, (c) => c.tree.realized.sopr.adjusted.transferVolume),
      {
        name: "SOPR",
        tree: [
          ...groupedSoprCharts(list, all, (c) => c.tree.realized.sopr.ratio, title),
          { name: "Adjusted", tree: groupedSoprCharts(list, all, (c) => c.tree.realized.sopr.adjusted.ratio, title, "Adjusted ") },
        ],
      },
      groupedValueDestroyedFolderWithAdjusted(list, all, title, (c) => c.tree.realized.sopr.valueDestroyed, (c) => c.tree.realized.sopr.adjusted.valueDestroyed),
    ),
  };
}

/** @param {{ list: readonly (CohortFull | CohortLongTerm)[], all: CohortAll, title: (name: string) => string }} args */
export function createGroupedActivitySection({ list, all, title }) {
  return {
    name: "Activity",
    tree: groupedFullActivityTree(list, all, title,
      groupedVolumeFolder(list, all, title, (c) => c.tree.activity.transferVolume),
      { name: "SOPR", tree: groupedSoprCharts(list, all, (c) => c.tree.realized.sopr.ratio, title) },
      groupedValueDestroyedFolder(list, all, title, (c) => c.tree.realized.sopr.valueDestroyed),
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
            series: tree.activity.dormancy[w.key],
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
            series: tree.realized.sellSideRiskRatio[w.key].ratio,
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
      groupedVolumeFolder(list, all, title, (c) => c.tree.activity.transferVolume),
      {
        name: "SOPR",
        title: title("SOPR (24h)"),
        bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
          baseline({
            series: tree.realized.sopr.ratio._24h,
            name,
            color,
            unit: Unit.ratio,
            base: 1,
          }),
        ),
      },
      groupedValueDestroyedFolder(list, all, title, (c) => c.tree.realized.sopr.valueDestroyed),
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
