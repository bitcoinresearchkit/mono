/**
 * Holdings section builders
 *
 * Supply pattern capabilities by cohort type:
 * - DeltaHalfInRelTotalPattern2 (STH/LTH): inProfit + inLoss + dominance + share
 * - SeriesTree_Cohorts_Utxo_All_Supply (All): inProfit + inLoss + share (no dominance)
 * - Core/AgeRange: inProfit + inLoss + dominance (no share)
 * - DeltaHalfInTotalPattern2 (Type.*): inProfit + inLoss (no rel)
 * - DeltaHalfTotalPattern (Empty/UtxoAmount/AddrAmount): total + half only
 */

import { Unit } from "../../utils/units.js";
import {
  ROLLING_WINDOWS,
  line,
  baseline,
  sumsTreeBaseline,
  amountSumsTreeBaseline,
  rollingPercentRatioTree,
  percentRatio,
  percentRatioBaseline,
  chartsFromCount,
} from "../series.js";
import {
  amountBaseline,
  satsBtcUsd,
  flatMapCohorts,
  mapCohortsWithAll,
  flatMapCohortsWithAll,
  groupedWindowsCumulativeWithAll,
} from "../shared.js";
import { colors } from "../../utils/colors.js";
import { priceLine } from "../constants.js";

/**
 * Simple supply series (total + half only, no profit/loss)
 * @param {{ total: AnyValuePattern }} supply
 * @returns {AnyFetchedSeriesBlueprint[]}
 */
function simpleSupplySeries(supply) {
  return satsBtcUsd({
    pattern: supply.total,
    name: "Total",
  });
}


/**
 * @param {readonly (UtxoCohortObject | CohortWithoutRelative)[]} list
 * @param {CohortAll} all
 * @param {(name: string) => string} title
 */
function groupedOutputsFolder(list, all, title) {
  const folder = groupedUnspentOutputsFolder(list, all, title);
  folder.tree.push({
    name: "Spent",
    tree: groupedWindowsCumulativeWithAll({
      list, all, title, metricTitle: "Spent UTXO Count",
      getWindowSeries: (c, key) => c.tree.outputs.spentCount.sum[key],
      getCumulativeSeries: (c) => c.tree.outputs.spentCount.cumulative,
      seriesFn: line, unit: Unit.count,
    }),
  });
  return folder;
}

/**
 * @param {readonly (UtxoCohortObject | AddrCohortObject | CohortWithoutRelative)[]} list
 * @param {CohortAll} all
 * @param {(name: string) => string} title
 */
function groupedUnspentOutputsFolder(list, all, title) {
  return {
    name: "Outputs",
    tree: [
      {
        name: "Unspent",
        tree: [
          {
            name: "Count",
            title: title("UTXO Count"),
            bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
              line({ series: tree.outputs.unspentCount.base, name, color, unit: Unit.count }),
            ),
          },
          ...groupedDeltaItems(list, all, (c) => c.tree.outputs.unspentCount.delta, Unit.count, title, "UTXO Count"),
        ],
      },
    ],
  };
}

/**
 * @param {{ absolute: { _24h: AnySeriesPattern, _1w: AnySeriesPattern, _1m: AnySeriesPattern, _1y: AnySeriesPattern }, rate: { _24h: { percent: AnySeriesPattern, ratio: AnySeriesPattern }, _1w: { percent: AnySeriesPattern, ratio: AnySeriesPattern }, _1m: { percent: AnySeriesPattern, ratio: AnySeriesPattern }, _1y: { percent: AnySeriesPattern, ratio: AnySeriesPattern } } }} delta
 * @param {Unit} unit
 * @param {(name: string) => string} title
 * @param {string} name
 * @returns {PartialOptionsTree}
 */
function singleDeltaItems(delta, unit, title, name) {
  return [
    {
      ...sumsTreeBaseline({
        windows: delta.absolute,
        title,
        metric: `${name} Change`,
        unit,
        legend: "Change",
      }),
      name: "Change",
    },
    {
      ...rollingPercentRatioTree({
        windows: delta.rate,
        title,
        metric: `${name} Growth Rate`,
      }),
      name: "Growth Rate",
    },
  ];
}

/**
 * @template {{ name: string, color: Color }} T
 * @template {{ name: string, color: Color }} A
 * @param {readonly T[]} list
 * @param {A} all
 * @param {(c: T | A) => DeltaPattern} getDelta
 * @param {Unit} unit
 * @param {(name: string) => string} title
 * @param {string} name
 * @returns {PartialOptionsTree}
 */
function groupedDeltaItems(list, all, getDelta, unit, title, name) {
  return [
      {
        name: "Change",
        tree: ROLLING_WINDOWS.map((w) => ({
          name: w.name,
          title: title(`${w.title} ${name} Change`),
          bottom: mapCohortsWithAll(list, all, (c) =>
            baseline({
              series: getDelta(c).absolute[w.key],
              name: c.name,
              color: c.color,
              unit,
            }),
          ),
        })),
      },
      {
        name: "Growth Rate",
        tree: ROLLING_WINDOWS.map((w) => ({
          name: w.name,
          title: title(`${w.title} ${name} Growth Rate`),
          bottom: flatMapCohortsWithAll(list, all, (c) =>
            percentRatioBaseline({
              pattern: getDelta(c).rate[w.key],
              name: c.name,
              color: c.color,
            }),
          ),
        })),
      },
  ];
}

/**
 * Amount-valued single-cohort delta: Change exposes sats + lazy btc per window.
 * @param {AmountDeltaPattern} delta
 * @param {(name: string) => string} title
 * @param {string} name
 * @returns {PartialOptionsTree}
 */
function singleAmountDeltaItems(delta, title, name) {
  return [
    {
      ...amountSumsTreeBaseline({
        windows: delta.absolute,
        title,
        metric: `${name} Change`,
        legend: "Change",
      }),
      name: "Change",
    },
    {
      ...rollingPercentRatioTree({
        windows: delta.rate,
        title,
        metric: `${name} Growth Rate`,
      }),
      name: "Growth Rate",
    },
  ];
}

/**
 * Amount-valued grouped-cohort delta: Change exposes sats + lazy btc per window.
 * @template {{ name: string, color: Color }} T
 * @template {{ name: string, color: Color }} A
 * @param {readonly T[]} list
 * @param {A} all
 * @param {(c: T | A) => AmountDeltaPattern} getDelta
 * @param {(name: string) => string} title
 * @param {string} name
 * @returns {PartialOptionsTree}
 */
function groupedAmountDeltaItems(list, all, getDelta, title, name) {
  return [
    {
      name: "Change",
      tree: ROLLING_WINDOWS.map((w) => ({
        name: w.name,
        title: title(`${w.title} ${name} Change`),
        bottom: flatMapCohortsWithAll(list, all, (c) =>
          amountBaseline({
            pattern: getDelta(c).absolute[w.key],
            name: c.name,
            color: c.color,
          }),
        ),
      })),
    },
    {
      name: "Growth Rate",
      tree: ROLLING_WINDOWS.map((w) => ({
        name: w.name,
        title: title(`${w.title} ${name} Growth Rate`),
        bottom: flatMapCohortsWithAll(list, all, (c) =>
          percentRatioBaseline({
            pattern: getDelta(c).rate[w.key],
            name: c.name,
            color: c.color,
          }),
        ),
      })),
    },
  ];
}

// ============================================================================
// Single Cohort Composable Builders
// ============================================================================

/**
 * Amount chart: total + halved + in profit + in loss in sats/btc/usd.
 * @param {{ total: AnyValuePattern, half: AnyValuePattern, inProfit: AnyValuePattern, inLoss: AnyValuePattern }} supply
 * @param {(name: string) => string} title
 * @returns {PartialChartOption}
 */
function profitabilityAmountChart(supply, title) {
  return {
    name: "Amount",
    title: title("Supply Profitability"),
    bottom: [
      ...satsBtcUsd({ pattern: supply.total, name: "Total", color: colors.default }),
      ...satsBtcUsd({ pattern: supply.inProfit, name: "In Profit", color: colors.profit }),
      ...satsBtcUsd({ pattern: supply.inLoss, name: "In Loss", color: colors.loss }),
      ...satsBtcUsd({ pattern: supply.half, name: "Halved", color: colors.gray, style: 4 }),
    ],
  };
}

/**
 * Composition chart: in profit / in loss as % of own supply.
 * @param {{ inProfit: { share: { percent: AnySeriesPattern, ratio: AnySeriesPattern } }, inLoss: { share: { percent: AnySeriesPattern, ratio: AnySeriesPattern } } }} supply
 * @param {(name: string) => string} title
 * @returns {PartialChartOption}
 */
function profitabilityCompositionChart(supply, title) {
  return {
    name: "Composition",
    title: title("Supply Profitability Composition"),
    bottom: [
      ...percentRatio({ pattern: supply.inProfit.share, name: "In Profit", color: colors.profit }),
      ...percentRatio({ pattern: supply.inLoss.share, name: "In Loss", color: colors.loss }),
      priceLine({ number: 100, color: colors.default, style: 0, unit: Unit.percentage }),
      priceLine({ number: 50, unit: Unit.percentage }),
    ],
  };
}


/**
 * @param {{ dominance: PercentRatioPattern }} supply
 * @param {Color} color
 * @param {(name: string) => string} title
 * @returns {PartialChartOption}
 */
function dominanceChart(supply, color, title) {
  return {
    name: "Dominance",
    title: title("Supply Dominance"),
    bottom: percentRatio({ pattern: supply.dominance, name: "Dominance", color }),
  };
}

/**
 * @param {OutputsPattern} outputs
 * @param {Color} color
 * @param {(name: string) => string} title
 * @returns {PartialOptionsGroup}
 */
function outputsFolder(outputs, color, title) {
  const folder = unspentOutputsFolder(outputs, color, title);
  folder.tree.push({
    name: "Spent",
    tree: chartsFromCount({ pattern: outputs.spentCount, title, metric: "Spent UTXO Count", unit: Unit.count, color }),
  });
  return folder;
}

/**
 * @param {{ unspentCount: { base: AnySeriesPattern, delta: DeltaPattern } }} outputs
 * @param {Color} color
 * @param {(name: string) => string} title
 * @returns {PartialOptionsGroup}
 */
function unspentOutputsFolder(outputs, color, title) {
  return {
    name: "Outputs",
    tree: [
      countFolder(outputs.unspentCount, "Unspent", "UTXO Count", color, title),
    ],
  };
}

/**
 * @param {{ base: AnySeriesPattern, delta: DeltaPattern }} pattern
 * @param {string} name
 * @param {string} chartTitle
 * @param {Color} color
 * @param {(name: string) => string} title
 * @returns {PartialOptionsGroup}
 */
function countFolder(pattern, name, chartTitle, color, title) {
  return {
    name,
    tree: [
      {
        name: "Count",
        title: title(chartTitle),
        bottom: [
          line({
            series: pattern.base,
            name: "Count",
            color,
            unit: Unit.count,
          }),
        ],
      },
      ...singleDeltaItems(pattern.delta, Unit.count, title, chartTitle),
    ],
  };
}

// ============================================================================
// Single Cohort Holdings Sections
// ============================================================================

/**
 * @param {{ cohort: UtxoCohortObject | CohortWithoutRelative, title: (name: string) => string }} args
 * @returns {PartialOptionsTree}
 */
export function createHoldingsSection({ cohort, title }) {
  const { supply } = cohort.tree;
  return [
    {
      name: "Supply",
      tree: [
        {
          name: "Total",
          title: title("Supply"),
          bottom: simpleSupplySeries(supply),
        },
        dominanceChart(supply, cohort.color, title),
        ...singleAmountDeltaItems(supply.delta, title, "Supply"),
      ],
    },
    outputsFolder(cohort.tree.outputs, cohort.color, title),
  ];
}

/**
 * @param {{ cohort: CohortAll, title: (name: string) => string }} args
 * @returns {PartialOptionsTree}
 */
export function createHoldingsSectionAll({ cohort, title }) {
  const { supply } = cohort.tree;
  return [
    {
      name: "Supply",
      tree: [
        {
          name: "Total",
          title: title("Supply"),
          bottom: simpleSupplySeries(supply),
        },
        {
          name: "Profitability",
          tree: [
            profitabilityAmountChart(supply, title),
            profitabilityCompositionChart(supply, title),
          ],
        },
        ...singleAmountDeltaItems(supply.delta, title, "Supply"),
      ],
    },
    unspentOutputsFolder(cohort.tree.outputs, cohort.color, title),
    countFolder(cohort.addressCount, "Addresses", "Address Count", cohort.color, title),
  ];
}

/**
 * @param {{ cohort: CohortFull | CohortLongTerm, title: (name: string) => string }} args
 * @returns {PartialOptionsTree}
 */
export function createHoldingsSectionWithRelative({ cohort, title }) {
  const { supply } = cohort.tree;
  return [
    {
      name: "Supply",
      tree: [
        {
          name: "Total",
          title: title("Supply"),
          bottom: simpleSupplySeries(supply),
        },
        dominanceChart(supply, cohort.color, title),
        {
          name: "Profitability",
          tree: [
            profitabilityAmountChart(supply, title),
            profitabilityCompositionChart(supply, title),
          ],
        },
        ...singleAmountDeltaItems(supply.delta, title, "Supply"),
      ],
    },
    outputsFolder(cohort.tree.outputs, cohort.color, title),
  ];
}

/**
 * @param {{ cohort: CohortCore | CohortAgeRange, title: (name: string) => string }} args
 * @returns {PartialOptionsTree}
 */
export function createHoldingsSectionWithOwnSupply({ cohort, title }) {
  const { supply } = cohort.tree;
  return [
    {
      name: "Supply",
      tree: [
        {
          name: "Total",
          title: title("Supply"),
          bottom: simpleSupplySeries(supply),
        },
        dominanceChart(supply, cohort.color, title),
        {
          name: "Profitability",
          tree: [profitabilityAmountChart(supply, title)],
        },
        ...singleAmountDeltaItems(supply.delta, title, "Supply"),
      ],
    },
    outputsFolder(cohort.tree.outputs, cohort.color, title),
  ];
}

/**
 * @param {{ cohort: CohortWithoutRelative, title: (name: string) => string }} args
 * @returns {PartialOptionsTree}
 */
export function createHoldingsSectionWithProfitLoss({ cohort, title }) {
  const { supply } = cohort.tree;
  return [
    {
      name: "Supply",
      tree: [
        {
          name: "Total",
          title: title("Supply"),
          bottom: simpleSupplySeries(supply),
        },
        dominanceChart(supply, cohort.color, title),
        {
          name: "Profitability",
          tree: [profitabilityAmountChart(supply, title)],
        },
        ...singleAmountDeltaItems(supply.delta, title, "Supply"),
      ],
    },
    outputsFolder(cohort.tree.outputs, cohort.color, title),
  ];
}

/**
 * @param {{ cohort: CohortAddr, title: (name: string) => string }} args
 * @returns {PartialOptionsTree}
 */
export function createHoldingsSectionAddress({ cohort, title }) {
  const { supply } = cohort.tree;
  return [
    {
      name: "Supply",
      tree: [
        {
          name: "Total",
          title: title("Supply"),
          bottom: simpleSupplySeries(supply),
        },
        dominanceChart(supply, cohort.color, title),
        {
          name: "Profitability",
          tree: [profitabilityAmountChart(supply, title)],
        },
        ...singleAmountDeltaItems(supply.delta, title, "Supply"),
      ],
    },
    outputsFolder(cohort.tree.outputs, cohort.color, title),
    countFolder(cohort.addressCount, "Addresses", "Address Count", cohort.color, title),
  ];
}

/**
 * @param {{ cohort: AddrCohortObject, title: (name: string) => string }} args
 * @returns {PartialOptionsTree}
 */
export function createHoldingsSectionAddressAmount({ cohort, title }) {
  const { supply } = cohort.tree;
  return [
    {
      name: "Supply",
      tree: [
        {
          name: "Total",
          title: title("Supply"),
          bottom: simpleSupplySeries(supply),
        },
        dominanceChart(supply, cohort.color, title),
        ...singleAmountDeltaItems(supply.delta, title, "Supply"),
      ],
    },
    unspentOutputsFolder(cohort.tree.outputs, cohort.color, title),
    countFolder(cohort.addressCount, "Addresses", "Address Count", cohort.color, title),
  ];
}

// ============================================================================
// Grouped Cohort Supply Helpers
// ============================================================================

/**
 * @template {{ name: string, color: Color, tree: { supply: { total: AnyValuePattern } } }} T
 * @param {readonly T[]} list
 * @param {CohortAll} all
 * @param {(name: string) => string} title
 * @returns {PartialChartOption}
 */
function groupedSupplyTotal(list, all, title) {
  return { name: "Total", title: title("Supply"), bottom: flatMapCohortsWithAll(list, all, ({ name, color, tree }) => satsBtcUsd({ pattern: tree.supply.total, name, color })) };
}

/**
 * @template {{ name: string, color: Color, tree: { supply: { inProfit: AnyValuePattern, inLoss: AnyValuePattern } } }} T
 * @param {readonly T[]} list
 * @param {CohortAll} all
 * @param {(name: string) => string} title
 * @returns {PartialOptionsTree}
 */
function groupedSupplyProfitLoss(list, all, title) {
  return [
    { name: "In Profit", title: title("Supply In Profit"), bottom: flatMapCohortsWithAll(list, all, ({ name, color, tree }) => satsBtcUsd({ pattern: tree.supply.inProfit, name, color })) },
    { name: "In Loss", title: title("Supply In Loss"), bottom: flatMapCohortsWithAll(list, all, ({ name, color, tree }) => satsBtcUsd({ pattern: tree.supply.inLoss, name, color })) },
  ];
}

/**
 * @template {{ name: string, color: Color, tree: { supply: { dominance: PercentRatioPattern } } }} T
 * @param {readonly T[]} list
 * @param {(name: string) => string} title
 * @returns {PartialChartOption}
 */
function groupedDominanceChart(list, title) {
  return {
    name: "Dominance",
    title: title("Supply Dominance"),
    bottom: flatMapCohorts(list, ({ name, color, tree }) =>
      percentRatio({ pattern: tree.supply.dominance, name, color }),
    ),
  };
}

// ============================================================================
// Grouped Cohort Holdings Sections
// ============================================================================

/**
 * @param {{ list: readonly CohortAddr[], all: CohortAll, title: (name: string) => string }} args
 * @returns {PartialOptionsTree}
 */
export function createGroupedHoldingsSectionAddress({ list, all, title }) {
  return [
    {
      name: "Supply",
      tree: [
        groupedSupplyTotal(list, all, title),
        groupedDominanceChart(list, title),
        {
          name: "Profitability",
          tree: groupedSupplyProfitLoss(list, all, title),
        },
        ...groupedAmountDeltaItems(list, all, (c) => c.tree.supply.delta, title, "Supply"),
      ],
    },
    groupedOutputsFolder(list, all, title),
    {
      name: "Addresses",
      tree: [
        {
          name: "Count",
          title: title("Address Count"),
          bottom: mapCohortsWithAll(list, all, ({ name, color, addressCount }) =>
            line({ series: addressCount.base, name, color, unit: Unit.count }),
          ),
        },
        ...groupedDeltaItems(list, all, (c) => c.addressCount.delta, Unit.count, title, "Address Count"),
      ],
    },
    {
      name: "Average Holdings",
      tree: [
        {
          name: "Per UTXO",
          title: title("Average Holdings per UTXO"),
          bottom: flatMapCohortsWithAll(list, all, ({ name, color, avgAmount }) =>
            satsBtcUsd({ pattern: avgAmount.utxo, name, color }),
          ),
        },
        {
          name: "Per Address",
          title: title("Average Holdings per Funded Address"),
          bottom: flatMapCohortsWithAll(list, all, ({ name, color, avgAmount }) =>
            satsBtcUsd({ pattern: avgAmount.addr, name, color }),
          ),
        },
      ],
    },
  ];
}

/**
 * Grouped holdings for address amount cohorts (no inProfit/inLoss, has address count)
 * @param {{ list: readonly AddrCohortObject[], all: CohortAll, title: (name: string) => string }} args
 * @returns {PartialOptionsTree}
 */
export function createGroupedHoldingsSectionAddressAmount({ list, all, title }) {
  return [
    {
      name: "Supply",
      tree: [
        groupedSupplyTotal(list, all, title),
        groupedDominanceChart(list, title),
        ...groupedAmountDeltaItems(list, all, (c) => c.tree.supply.delta, title, "Supply"),
      ],
    },
    groupedUnspentOutputsFolder(list, all, title),
    {
      name: "Addresses",
      tree: [
        {
          name: "Count",
          title: title("Address Count"),
          bottom: mapCohortsWithAll(list, all, ({ name, color, addressCount }) =>
            line({ series: addressCount.base, name, color, unit: Unit.count }),
          ),
        },
        ...groupedDeltaItems(list, all, (c) => c.addressCount.delta, Unit.count, title, "Address Count"),
      ],
    },
  ];
}

/** @param {{ list: readonly (UtxoCohortObject | CohortWithoutRelative)[], all: CohortAll, title: (name: string) => string }} args */
export function createGroupedHoldingsSection({ list, all, title }) {
  return [
    {
      name: "Supply",
      tree: [
        groupedSupplyTotal(list, all, title),
        groupedDominanceChart(list, title),
        ...groupedAmountDeltaItems(list, all, (c) => c.tree.supply.delta, title, "Supply"),
      ],
    },
    groupedOutputsFolder(list, all, title),
  ];
}

/** @param {{ list: readonly CohortWithoutRelative[], all: CohortAll, title: (name: string) => string }} args */
export function createGroupedHoldingsSectionWithProfitLoss({ list, all, title }) {
  return [
    {
      name: "Supply",
      tree: [
        groupedSupplyTotal(list, all, title),
        groupedDominanceChart(list, title),
        {
          name: "Profitability",
          tree: groupedSupplyProfitLoss(list, all, title),
        },
        ...groupedAmountDeltaItems(list, all, (c) => c.tree.supply.delta, title, "Supply"),
      ],
    },
    groupedOutputsFolder(list, all, title),
  ];
}

/** @param {{ list: readonly (CohortCore | CohortAgeRange)[], all: CohortAll, title: (name: string) => string }} args */
export function createGroupedHoldingsSectionWithOwnSupply({ list, all, title }) {
  return [
    {
      name: "Supply",
      tree: [
        groupedSupplyTotal(list, all, title),
        groupedDominanceChart(list, title),
        {
          name: "Profitability",
          tree: groupedSupplyProfitLoss(list, all, title),
        },
        ...groupedAmountDeltaItems(list, all, (c) => c.tree.supply.delta, title, "Supply"),
      ],
    },
    groupedOutputsFolder(list, all, title),
  ];
}

/**
 * Grouped holdings with full relative series (dominance + share)
 * For: CohortFull, CohortLongTerm
 * @param {{ list: readonly (CohortFull | CohortLongTerm)[], all: CohortAll, title: (name: string) => string }} args
 * @returns {PartialOptionsTree}
 */
export function createGroupedHoldingsSectionWithRelative({ list, all, title }) {
  return [
    {
      name: "Supply",
      tree: [
        groupedSupplyTotal(list, all, title),
        groupedDominanceChart(list, title),
        {
          name: "Profitability",
          tree: [
            ...groupedSupplyProfitLoss(list, all, title),
            {
              name: "Composition",
              tree: [
                { name: "In Profit", title: title("Supply In Profit Composition"), bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) => line({ series: tree.supply.inProfit.share.percent, name, color, unit: Unit.percentage })) },
                { name: "In Loss", title: title("Supply In Loss Composition"), bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) => line({ series: tree.supply.inLoss.share.percent, name, color, unit: Unit.percentage })) },
              ],
            },
          ],
        },
        ...groupedAmountDeltaItems(list, all, (c) => c.tree.supply.delta, title, "Supply"),
      ],
    },
    groupedOutputsFolder(list, all, title),
  ];
}
