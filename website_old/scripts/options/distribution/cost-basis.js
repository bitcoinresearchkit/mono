/**
 * Cost Basis section builders
 *
 * Structure:
 * - Per Coin: sats-weighted (profitability + distribution)
 * - Per Dollar: value-weighted (profitability + distribution)
 * - Profitability: cross-cutting (per coin + per dollar on same chart)
 * - Supply Density: cost basis supply density percentage
 *
 * Only for cohorts WITH costBasis (All, STH, LTH)
 */

import { colors } from "../../utils/colors.js";
import { entries } from "../../utils/array.js";
import { price, percentRatio } from "../series.js";
import { mapCohortsWithAll, flatMapCohortsWithAll } from "../shared.js";

const ACTIVE_PCTS = new Set(["pct75", "pct50", "pct25"]);

/**
 * @param {PercentilesPattern} p
 * @param {(name: string) => string} [n]
 * @returns {FetchedPriceSeriesBlueprint[]}
 */
function percentileSeries(p, n = (x) => x) {
  return entries(p)
    .reverse()
    .map(([key, s], i, arr) =>
      price({
        series: s,
        name: n(key.replace("pct", "P")),
        color: colors.at(i, arr.length),
        ...(ACTIVE_PCTS.has(key) ? {} : { defaultActive: false }),
      }),
    );
}

// ============================================================================
// Single cohort helpers
// ============================================================================

/**
 * Per Coin or Per Dollar folder for a single cohort
 * @param {Object} args
 * @param {AnyPricePattern} args.avgPrice - realized price (per coin) or capitalized price (per dollar)
 * @param {string} args.avgName
 * @param {AnyPricePattern} args.inProfit
 * @param {AnyPricePattern} args.inLoss
 * @param {PercentilesPattern} args.percentiles
 * @param {Color} args.color
 * @param {string} args.weightLabel
 * @param {(name: string) => string} args.title
 * @param {AnyPricePattern} [args.min]
 * @param {AnyPricePattern} [args.max]
 * @returns {PartialOptionsTree}
 */
function singleWeightFolder({ avgPrice, avgName, inProfit, inLoss, percentiles, color, weightLabel, title, min, max }) {
  return [
    {
      name: "Average",
      title: title(`Cost Basis Average (${weightLabel})`),
      top: [
        price({ series: inProfit, name: "In Profit", color: colors.profit }),
        price({ series: avgPrice, name: avgName, color }),
        price({ series: inLoss, name: "In Loss", color: colors.loss }),
      ],
    },
    {
      name: "Distribution",
      title: title(`Cost Basis Distribution (${weightLabel})`),
      top: [
        price({ series: avgPrice, name: avgName, color }),
        ...(max ? [price({ series: max, name: "P100", color: colors.stat.max, defaultActive: false })] : []),
        ...percentileSeries(percentiles),
        ...(min ? [price({ series: min, name: "P0", color: colors.stat.min, defaultActive: false })] : []),
      ],
    },
  ];
}

/**
 * @param {{ cohort: CohortAll | CohortFull | CohortLongTerm, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createCostBasisSectionWithPercentiles({ cohort, title }) {
  const { tree, color } = cohort;
  const cb = tree.costBasis;
  return {
    name: "Cost Basis",
    tree: [
      {
        name: "Per Coin",
        tree: singleWeightFolder({
          avgPrice: tree.realized.price, avgName: "All",
          inProfit: cb.inProfit.perCoin, inLoss: cb.inLoss.perCoin,
          percentiles: cb.perCoin, color, weightLabel: "BTC-weighted", title,
          min: cb.min, max: cb.max,
        }),
      },
      {
        name: "Per Dollar",
        tree: singleWeightFolder({
          avgPrice: tree.realized.capitalized.price, avgName: "All",
          inProfit: cb.inProfit.perDollar, inLoss: cb.inLoss.perDollar,
          percentiles: cb.perDollar, color, weightLabel: "USD-weighted", title,
        }),
      },
      {
        name: "Supply Density",
        title: title("Cost Basis Supply Density"),
        bottom: percentRatio({ pattern: cb.supplyDensity, name: "Supply Density", color: colors.bitcoin }),
      },
    ],
  };
}

// ============================================================================
// Grouped cohort helpers
// ============================================================================

/**
 * Per Coin or Per Dollar folder for grouped cohorts
 * @param {Object} args
 * @param {readonly (CohortAll | CohortFull | CohortLongTerm)[]} args.list
 * @param {CohortAll} args.all
 * @param {(c: CohortAll | CohortFull | CohortLongTerm) => AnyPricePattern} args.getAvgPrice
 * @param {(c: CohortAll | CohortFull | CohortLongTerm) => AnyPricePattern} args.getInProfit
 * @param {(c: CohortAll | CohortFull | CohortLongTerm) => AnyPricePattern} args.getInLoss
 * @param {(c: CohortAll | CohortFull | CohortLongTerm) => PercentilesPattern} args.getPercentiles
 * @param {string} args.avgTitle
 * @param {string} args.weightLabel
 * @param {(name: string) => string} args.title
 * @returns {PartialOptionsTree}
 */
function groupedWeightFolder({ list, all, getAvgPrice, getInProfit, getInLoss, getPercentiles, avgTitle, weightLabel, title }) {
  return [
    {
      name: "Average",
      title: title(`Cost Basis ${avgTitle} (${weightLabel})`),
      top: mapCohortsWithAll(list, all, (c) =>
        price({ series: getAvgPrice(c), name: c.name, color: c.color }),
      ),
    },
    {
      name: "In Profit",
      title: title(`Cost Basis In Profit (${weightLabel})`),
      top: mapCohortsWithAll(list, all, (c) =>
        price({ series: getInProfit(c), name: c.name, color: c.color }),
      ),
    },
    {
      name: "In Loss",
      title: title(`Cost Basis In Loss (${weightLabel})`),
      top: mapCohortsWithAll(list, all, (c) =>
        price({ series: getInLoss(c), name: c.name, color: c.color }),
      ),
    },
    ...(/** @type {const} */ ([
      ["pct50", "Median"],
      ["pct75", "Q3"],
      ["pct25", "Q1"],
    ])).map(([pct, label]) => ({
      name: label,
      title: title(`Cost Basis ${label} (${weightLabel})`),
      top: mapCohortsWithAll(list, all, (c) =>
        price({ series: getPercentiles(c)[pct], name: c.name, color: c.color }),
      ),
    })),
  ];
}

/**
 * @param {{ list: readonly (CohortAll | CohortFull | CohortLongTerm)[], all: CohortAll, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createGroupedCostBasisSectionWithPercentiles({ list, all, title }) {
  return {
    name: "Cost Basis",
    tree: [
      {
        name: "Per Coin",
        tree: groupedWeightFolder({
          list, all, title,
          getAvgPrice: (c) => c.tree.realized.price,
          getInProfit: (c) => c.tree.costBasis.inProfit.perCoin,
          getInLoss: (c) => c.tree.costBasis.inLoss.perCoin,
          getPercentiles: (c) => c.tree.costBasis.perCoin,
          avgTitle: "Average", weightLabel: "BTC-weighted",
        }),
      },
      {
        name: "Per Dollar",
        tree: groupedWeightFolder({
          list, all, title,
          getAvgPrice: (c) => c.tree.realized.capitalized.price,
          getInProfit: (c) => c.tree.costBasis.inProfit.perDollar,
          getInLoss: (c) => c.tree.costBasis.inLoss.perDollar,
          getPercentiles: (c) => c.tree.costBasis.perDollar,
          avgTitle: "Average", weightLabel: "USD-weighted",
        }),
      },
      {
        name: "Supply Density",
        title: title("Cost Basis Supply Density"),
        bottom: flatMapCohortsWithAll(list, all, ({ name, color, tree }) =>
          percentRatio({ pattern: tree.costBasis.supplyDensity, name, color }),
        ),
      },
    ],
  };
}
