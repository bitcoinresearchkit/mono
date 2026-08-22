/** Shared helpers for options */

import { Unit } from "../utils/units.js";
import {
  ROLLING_WINDOWS,
  line,
  baseline,
  price,
  percentRatio,
  chartsFromPercentCumulativeEntries,
  sumsAndAveragesCumulativeWith,
} from "./series.js";
import { colors } from "../utils/colors.js";

// ============================================================================
// Grouped Cohort Helpers
// ============================================================================

/**
 * Map cohorts to series (without "all" cohort)
 * Use for charts where "all" doesn't have required properties
 * @template T
 * @template R
 * @param {readonly T[]} list
 * @param {(item: T) => R} fn
 * @returns {R[]}
 */
export function mapCohorts(list, fn) {
  return list.map(fn);
}

/**
 * FlatMap cohorts to series (without "all" cohort)
 * Use for charts where "all" doesn't have required properties
 * @template T
 * @template R
 * @param {readonly T[]} list
 * @param {(item: T) => R[]} fn
 * @returns {R[]}
 */
export function flatMapCohorts(list, fn) {
  return list.flatMap(fn);
}

/**
 * Map cohorts to series, with "all" cohort added as defaultActive: false
 * @template T
 * @template A
 * @template R
 * @param {readonly T[]} list
 * @param {A} all
 * @param {(item: T | A) => R} fn
 * @returns {R[]}
 */
export function mapCohortsWithAll(list, all, fn) {
  return [
    ...list.map(fn),
    { ...fn({ ...all, name: "All" }), defaultActive: false },
  ];
}

/**
 * FlatMap cohorts to series, with "all" cohort added as defaultActive: false
 * @template T
 * @template A
 * @template R
 * @param {readonly T[]} list
 * @param {A} all
 * @param {(item: T | A) => R[]} fn
 * @returns {R[]}
 */
export function flatMapCohortsWithAll(list, all, fn) {
  return [
    ...list.flatMap(fn),
    ...fn({ ...all, name: "All" }).map((s) => ({ ...s, defaultActive: false })),
  ];
}

/**
 * Create a title formatter for chart titles
 * @param {string} [cohortTitle]
 * @returns {(name: string) => string}
 */
export const formatCohortTitle = (cohortTitle) => (name) =>
  cohortTitle ? `${name}: ${cohortTitle}` : name;

/**
 * Create line series from an amount pattern (sats stored + lazy btc).
 * @param {Object} args
 * @param {AmountPattern} args.pattern
 * @param {string} args.name
 * @param {Color} [args.color]
 * @param {boolean} [args.defaultActive]
 * @param {number} [args.style]
 * @returns {FetchedLineSeriesBlueprint[]}
 */
export function amount({ pattern, name, color, defaultActive, style }) {
  return [
    line({
      series: pattern.btc,
      name,
      color,
      unit: Unit.btc,
      defaultActive,
      style,
    }),
    line({
      series: pattern.sats,
      name,
      color,
      unit: Unit.sats,
      defaultActive,
      style,
    }),
  ];
}

/**
 * Create baseline series from an amount pattern (sats stored + lazy btc).
 * @param {Object} args
 * @param {AmountPattern} args.pattern
 * @param {string} args.name
 * @param {Color} [args.color]
 * @param {boolean} [args.defaultActive]
 * @returns {FetchedBaselineSeriesBlueprint[]}
 */
export function amountBaseline({ pattern, name, color, defaultActive }) {
  return [
    baseline({
      series: pattern.btc,
      name,
      color,
      unit: Unit.btc,
      defaultActive,
    }),
    baseline({
      series: pattern.sats,
      name,
      color,
      unit: Unit.sats,
      defaultActive,
    }),
  ];
}

/**
 * Create sats/btc/usd line series from a pattern with .sats/.btc/.usd
 * @param {Object} args
 * @param {AnyValuePattern} args.pattern
 * @param {string} args.name
 * @param {Color} [args.color]
 * @param {boolean} [args.defaultActive]
 * @param {number} [args.style]
 * @returns {FetchedLineSeriesBlueprint[]}
 */
export function satsBtcUsd({ pattern, name, color, defaultActive, style }) {
  return [
    line({
      series: pattern.btc,
      name,
      color,
      unit: Unit.btc,
      defaultActive,
      style,
    }),
    line({
      series: pattern.sats,
      name,
      color,
      unit: Unit.sats,
      defaultActive,
      style,
    }),
    line({
      series: pattern.usd,
      name,
      color,
      unit: Unit.usd,
      defaultActive,
      style,
    }),
  ];
}

/**
 * Create sats/btc/usd baseline series from a value pattern
 * @param {Object} args
 * @param {{ btc: AnySeriesPattern, sats: AnySeriesPattern, usd: AnySeriesPattern }} args.pattern
 * @param {string} args.name
 * @param {Color} [args.color]
 * @param {boolean} [args.defaultActive]
 * @returns {FetchedBaselineSeriesBlueprint[]}
 */
export function satsBtcUsdBaseline({ pattern, name, color, defaultActive }) {
  return [
    baseline({
      series: pattern.btc,
      name,
      color,
      unit: Unit.btc,
      defaultActive,
    }),
    baseline({
      series: pattern.sats,
      name,
      color,
      unit: Unit.sats,
      defaultActive,
    }),
    baseline({
      series: pattern.usd,
      name,
      color,
      unit: Unit.usd,
      defaultActive,
    }),
  ];
}

/**
 * Create sats/btc/usd series from a value pattern's cumulative
 * @param {Object} args
 * @param {{ cumulative: AnyValuePattern }} args.source
 * @param {'cumulative'} args.key
 * @param {string} args.name
 * @param {Color} [args.color]
 * @param {boolean} [args.defaultActive]
 * @returns {FetchedLineSeriesBlueprint[]}
 */
export function satsBtcUsdFrom({ source, key, name, color, defaultActive }) {
  return satsBtcUsd({
    pattern: source[key],
    name,
    color,
    defaultActive,
  });
}

/**
 * Create coinbase/subsidy/fee series from separate sources
 * @param {Object} args
 * @param {{ cumulative: AnyValuePattern }} args.coinbase
 * @param {{ cumulative: AnyValuePattern }} args.subsidy
 * @param {{ cumulative: AnyValuePattern }} args.fee
 * @param {'cumulative'} args.key
 * @returns {FetchedLineSeriesBlueprint[]}
 */
export function revenueBtcSatsUsd({ coinbase, subsidy, fee, key }) {
  return [
    ...satsBtcUsdFrom({
      source: coinbase,
      key,
      name: "Coinbase",
      color: colors.mining.coinbase,
    }),
    ...satsBtcUsdFrom({
      source: subsidy,
      key,
      name: "Subsidy",
      color: colors.mining.subsidy,
    }),
    ...satsBtcUsdFrom({
      source: fee,
      key,
      name: "Fees",
      color: colors.mining.fee,
    }),
  ];
}

/**
 * Create sats/btc/usd series from a rolling window (24h/1w/1m/1y sum)
 * @param {Object} args
 * @param {AnyValuePattern} args.pattern - A BtcSatsUsdPattern (e.g., source.rolling._24h.sum)
 * @param {string} args.name
 * @param {Color} [args.color]
 * @param {boolean} [args.defaultActive]
 * @returns {FetchedLineSeriesBlueprint[]}
 */
export function satsBtcUsdRolling({ pattern, name, color, defaultActive }) {
  return satsBtcUsd({ pattern, name, color, defaultActive });
}

/**
 * Build a full Sum / Rolling / Cumulative tree from a FullValuePattern
 * @param {Object} args
 * @param {FullValuePattern} args.pattern
 * @param {(metric: string) => string} [args.title]
 * @param {string} args.metric
 * @param {Color} [args.color]
 * @returns {PartialOptionsTree}
 */
export function satsBtcUsdFullTree({ pattern, title, metric, color }) {
  return sumsAndAveragesCumulativeWith({
    sum: pattern.sum,
    average: pattern.average,
    cumulative: pattern.cumulative,
    title,
    metric,
    color,
    series: ({ pattern, name, color, defaultActive }) =>
      satsBtcUsd({ pattern, name, color, defaultActive }),
  });
}

/**
 * "Exposed" subtree (quantum-risk / revealed-pubkey addresses).
 * Shape: Compare (funded + total) / Funded / Total / Supply / Share.
 * Shared between Network and Distribution (per-type cohort view).
 * @param {ExposedTree} exposed
 * @param {AddressableType | "all"} key
 * @param {(name: string) => string} title
 * @returns {PartialOptionsGroup}
 */
export function exposedSubtree(exposed, key, title) {
  return {
    name: "Exposed",
    tree: [
      {
        name: "Compare",
        title: title("Exposed Address Count"),
        bottom: [
          line({ series: exposed.count.funded[key], name: "Funded", unit: Unit.count }),
          line({
            series: exposed.count.total[key],
            name: "Total",
            color: colors.gray,
            unit: Unit.count,
          }),
        ],
      },
      {
        name: "Funded",
        title: title("Funded Exposed Address Count"),
        bottom: [
          line({ series: exposed.count.funded[key], name: "Funded Exposed", unit: Unit.count }),
        ],
      },
      {
        name: "Total",
        title: title("Total Exposed Address Count"),
        bottom: [
          line({
            series: exposed.count.total[key],
            name: "Total Exposed",
            color: colors.gray,
            unit: Unit.count,
          }),
        ],
      },
      {
        name: "Supply",
        title: title("Supply in Exposed Addresses"),
        bottom: satsBtcUsd({ pattern: exposed.supply[key], name: "Supply" }),
      },
      {
        name: "Share",
        title: title("Share of Supply in Exposed Addresses"),
        bottom: percentRatio({ pattern: exposed.supply.share[key], name: "Supply" }),
      },
    ],
  };
}

/**
 * Windowed reused/respent counts with sums and optional averages.
 * @param {CountPattern<number>} reused
 * @param {CountPattern<number>} respent
 * @param {(name: string) => string} title
 * @param {string} metric
 * @returns {PartialOptionsTree}
 */
export function reusedCountTree(reused, respent, title, metric) {
  return [
    ...ROLLING_WINDOWS.map((window) => ({
      name: window.name,
      title: title(`${window.title} ${metric}`),
      bottom: [
        line({
          series: reused.sum[window.key],
          name: "2+ Funded",
          unit: Unit.count,
        }),
        line({
          series: respent.sum[window.key],
          name: "2+ Spent",
          color: colors.gray,
          unit: Unit.count,
        }),
        line({
          series: reused.average[window.key],
          name: "2+ Funded Avg",
          unit: Unit.count,
          defaultActive: false,
          style: 1,
        }),
        line({
          series: respent.average[window.key],
          name: "2+ Spent Avg",
          color: colors.gray,
          unit: Unit.count,
          defaultActive: false,
          style: 1,
        }),
      ],
    })),
    {
      name: "Cumulative",
      title: title(`Cumulative ${metric}`),
      bottom: [
        line({
          series: reused.cumulative,
          name: "2+ Funded",
          unit: Unit.count,
        }),
        line({
          series: respent.cumulative,
          name: "2+ Spent",
          color: colors.gray,
          unit: Unit.count,
        }),
      ],
    },
  ];
}

/**
 * "Reused" subtree (per-type / per-cohort, no "Active" window since that
 * data is only tracked globally). Respent (addresses whose outputs have
 * been spent more than once) is a subset of reused, so each chart layers
 * both series in two colors: reused in the primary color, respent in
 * gray. Shape: Funded / Total / Outputs / Inputs / Supply / Share.
 * @param {ReusedTree} reused
 * @param {RespentTree} respent
 * @param {AddressableType | "all"} key
 * @param {(name: string) => string} title
 * @returns {PartialOptionsGroup}
 */
export function reusedSubtree(reused, respent, key, title) {
  return {
    name: "Reused",
    tree: [
      {
        name: "Funded",
        title: title("Funded Reused Addresses"),
        bottom: [
          line({ series: reused.count.funded[key], name: "2+ Funded", unit: Unit.count }),
          line({
            series: respent.count.funded[key],
            name: "2+ Spent",
            color: colors.gray,
            unit: Unit.count,
          }),
        ],
      },
      {
        name: "Total",
        title: title("Total Reused Addresses"),
        bottom: [
          line({ series: reused.count.total[key], name: "2+ Funded", unit: Unit.count }),
          line({
            series: respent.count.total[key],
            name: "2+ Spent",
            color: colors.gray,
            unit: Unit.count,
          }),
        ],
      },
      {
        name: "Outputs",
        tree: [
          {
            name: "Count",
            tree: reusedCountTree(
              reused.events.outputToReusedAddrCount[key],
              respent.events.outputToReusedAddrCount[key],
              title,
              "Transaction Outputs to Reused Addresses",
            ),
          },
          {
            name: "Share",
            tree: chartsFromPercentCumulativeEntries({
              entries: [
                {
                  name: "2+ Funded",
                  pattern: reused.events.outputToReusedAddrShare[key],
                },
                {
                  name: "2+ Spent",
                  pattern: respent.events.outputToReusedAddrShare[key],
                  color: colors.gray,
                },
              ],
              title,
              metric: "Share of Transaction Outputs to Reused Addresses",
            }),
          },
        ],
      },
      {
        name: "Inputs",
        tree: [
          {
            name: "Count",
            tree: reusedCountTree(
              reused.events.inputFromReusedAddrCount[key],
              respent.events.inputFromReusedAddrCount[key],
              title,
              "Transaction Inputs from Reused Addresses",
            ),
          },
          {
            name: "Share",
            tree: chartsFromPercentCumulativeEntries({
              entries: [
                {
                  name: "2+ Funded",
                  pattern: reused.events.inputFromReusedAddrShare[key],
                },
                {
                  name: "2+ Spent",
                  pattern: respent.events.inputFromReusedAddrShare[key],
                  color: colors.gray,
                },
              ],
              title,
              metric: "Share of Transaction Inputs from Reused Addresses",
            }),
          },
        ],
      },
      {
        name: "Supply",
        title: title("Supply in Reused Addresses"),
        bottom: [
          ...satsBtcUsd({ pattern: reused.supply[key], name: "2+ Funded" }),
          ...satsBtcUsd({
            pattern: respent.supply[key],
            name: "2+ Spent",
            color: colors.gray,
          }),
        ],
      },
      {
        name: "Share",
        title: title("Share of Supply in Reused Addresses"),
        bottom: [
          ...percentRatio({ pattern: reused.supply.share[key], name: "2+ Funded" }),
          ...percentRatio({
            pattern: respent.supply.share[key],
            name: "2+ Spent",
            color: colors.gray,
          }),
        ],
      },
    ],
  };
}

/**
 * "Average Holdings" subtree: Compare (both) + Per UTXO + Per Funded Address.
 * Shared between Network and Distribution.
 * @param {AvgAmountPattern} pattern
 * @param {(name: string) => string} title
 * @returns {PartialOptionsGroup}
 */
export function avgHoldingsSubtree(pattern, title) {
  return {
    name: "Average Holdings",
    tree: [
      {
        name: "Compare",
        title: title("Average Holdings"),
        bottom: [
          ...satsBtcUsd({ pattern: pattern.utxo, name: "Per UTXO" }),
          ...satsBtcUsd({
            pattern: pattern.addr,
            name: "Per Funded Address",
            color: colors.gray,
          }),
        ],
      },
      {
        name: "Per UTXO",
        title: title("Average Holdings per UTXO"),
        bottom: satsBtcUsd({ pattern: pattern.utxo, name: "Per UTXO" }),
      },
      {
        name: "Per Address",
        title: title("Average Holdings per Funded Address"),
        bottom: satsBtcUsd({
          pattern: pattern.addr,
          name: "Per Funded Address",
        }),
      },
    ],
  };
}

/**
 * Create Price + Ratio charts from a simple price pattern (BpsCentsRatioSatsUsdPattern)
 * @param {Object} args
 * @param {AnyPricePattern & { ratio: AnySeriesPattern }} args.pattern
 * @param {string} args.title
 * @param {string} args.legend
 * @param {Color} [args.color]
 * @returns {PartialOptionsTree}
 */
export function simplePriceRatioTree({ pattern, title, legend, color }) {
  return [
    {
      name: title,
      title,
      top: [price({ series: pattern, name: legend, color })],
      bottom: [
        baseline({
          series: pattern.ratio,
          name: "Ratio",
          unit: Unit.ratio,
          base: 1,
        }),
      ],
    },
  ];
}

/**
 * @param {{ pct01: AnyPricePattern, pct05: AnyPricePattern, pct1: AnyPricePattern, pct2: AnyPricePattern, pct5: AnyPricePattern, pct10: AnyPricePattern, pct20: AnyPricePattern, pct30: AnyPricePattern, pct40: AnyPricePattern, pct50: AnyPricePattern, pct60: AnyPricePattern, pct70: AnyPricePattern, pct80: AnyPricePattern, pct90: AnyPricePattern, pct95: AnyPricePattern, pct98: AnyPricePattern, pct99: AnyPricePattern, pct995: AnyPricePattern, pct999: AnyPricePattern }} p
 */
export function percentileBands(p) {
  return percentileBandsWith(p, (e) => e);
}

/**
 * @template E
 * @template T
 * @param {{ pct01: E, pct05: E, pct1: E, pct2: E, pct5: E, pct10: E, pct20: E, pct30: E, pct40: E, pct50: E, pct60: E, pct70: E, pct80: E, pct90: E, pct95: E, pct98: E, pct99: E, pct995: E, pct999: E }} p
 * @param {(entry: E) => T} extract
 */
export function percentileBandsWith(p, extract) {
  return [
    {
      name: "P95",
      prop: extract(p.pct95),
      color: colors.ratioPct._95,
      defaultActive: true,
      lineStyle: 0,
    },
    {
      name: "P98",
      prop: extract(p.pct98),
      color: colors.ratioPct._98,
      defaultActive: true,
      lineStyle: 0,
    },
    {
      name: "P99",
      prop: extract(p.pct99),
      color: colors.ratioPct._99,
      defaultActive: true,
      lineStyle: 0,
    },
    {
      name: "P99.5",
      prop: extract(p.pct995),
      color: colors.ratioPct._99_5,
      defaultActive: true,
      lineStyle: 0,
    },
    {
      name: "P99.9",
      prop: extract(p.pct999),
      color: colors.ratioPct._99_9,
      defaultActive: true,
      lineStyle: 0,
    },
    {
      name: "P5",
      prop: extract(p.pct5),
      color: colors.ratioPct._5,
      defaultActive: true,
      lineStyle: 0,
    },
    {
      name: "P2",
      prop: extract(p.pct2),
      color: colors.ratioPct._2,
      defaultActive: true,
      lineStyle: 0,
    },
    {
      name: "P1",
      prop: extract(p.pct1),
      color: colors.ratioPct._1,
      defaultActive: true,
      lineStyle: 0,
    },
    {
      name: "P0.5",
      prop: extract(p.pct05),
      color: colors.ratioPct._0_5,
      defaultActive: true,
      lineStyle: 0,
    },
    {
      name: "P0.1",
      prop: extract(p.pct01),
      color: colors.ratioPct._0_1,
      defaultActive: true,
      lineStyle: 0,
    },
    {
      name: "P50",
      prop: extract(p.pct50),
      color: colors.ratioPct._50,
      defaultActive: true,
      lineStyle: 0,
    },
    { name: "P10", prop: extract(p.pct10), color: colors.ratioPct._10 },
    { name: "P20", prop: extract(p.pct20), color: colors.ratioPct._20 },
    { name: "P30", prop: extract(p.pct30), color: colors.ratioPct._30 },
    { name: "P40", prop: extract(p.pct40), color: colors.ratioPct._40 },
    { name: "P60", prop: extract(p.pct60), color: colors.ratioPct._60 },
    { name: "P70", prop: extract(p.pct70), color: colors.ratioPct._70 },
    { name: "P80", prop: extract(p.pct80), color: colors.ratioPct._80 },
    { name: "P90", prop: extract(p.pct90), color: colors.ratioPct._90 },
  ];
}

/**
 * @param {{ name: string, prop: AnyPricePattern, color: Color, defaultActive?: boolean, lineStyle?: number }[]} bands
 */
export function priceBands(bands) {
  return bands.map(({ name, prop, color, defaultActive, lineStyle }) =>
    price({
      series: prop,
      name,
      color,
      defaultActive: defaultActive ?? false,
      options: { lineStyle: lineStyle ?? 1 },
    }),
  );
}

/**
 * @param {{ name: string, prop: AnySeriesPattern, color: Color, defaultActive?: boolean, lineStyle?: number }[]} bands
 */
function ratioBands(bands) {
  return bands.map(({ name, prop, color, defaultActive, lineStyle }) =>
    line({
      series: prop,
      name,
      color,
      defaultActive: defaultActive ?? false,
      unit: Unit.ratio,
      options: { lineStyle: lineStyle ?? 1 },
    }),
  );
}

/**
 * @typedef {{ price: AnyPricePattern, ratio: AnySeriesPattern }} PriceRatioBand
 * @typedef {Record<"pct01" | "pct05" | "pct1" | "pct2" | "pct5" | "pct10" | "pct20" | "pct30" | "pct40" | "pct50" | "pct60" | "pct70" | "pct80" | "pct90" | "pct95" | "pct98" | "pct99" | "pct995" | "pct999", PriceRatioBand>} PriceRatioPercentiles
 */

/**
 * Price + Ratio charts with percentile bands
 * @param {Object} args
 * @param {AnyPricePattern & { ratio: AnySeriesPattern }} args.pattern
 * @param {PriceRatioPercentiles} args.percentiles
 * @param {string} args.title
 * @param {string} args.legend
 * @param {Color} [args.color]
 * @param {FetchedPriceSeriesBlueprint[]} [args.priceReferences]
 * @returns {PartialOptionsTree}
 */
export function priceRatioPercentilesTree({
  pattern,
  percentiles,
  title,
  legend,
  color,
  priceReferences,
}) {
  const pctUsd = percentileBandsWith(percentiles, (e) => e.price);
  const pctRatio = percentileBandsWith(percentiles, (e) => e.ratio);
  return [
    {
      name: title,
      title,
      top: [
        price({ series: pattern, name: legend, color }),
        ...(priceReferences ?? []),
        ...priceBands(pctUsd),
      ],
      bottom: [
        baseline({
          series: pattern.ratio,
          name: "Ratio",
          unit: Unit.ratio,
          base: 1,
        }),
        ...ratioBands(pctRatio),
      ],
    },
  ];
}

/**
 * Create coinbase/subsidy/fee rolling sum series from separate sources
 * @param {Object} args
 * @param {AnyValuePattern} args.coinbase - Rolling sum pattern (e.g., mining.rewards.coinbase.rolling._24h.sum)
 * @param {AnyValuePattern} args.subsidy
 * @param {AnyValuePattern} args.fee
 * @returns {FetchedLineSeriesBlueprint[]}
 */
export function revenueRollingBtcSatsUsd({ coinbase, subsidy, fee }) {
  return [
    ...satsBtcUsd({
      pattern: coinbase,
      name: "Coinbase",
      color: colors.mining.coinbase,
    }),
    ...satsBtcUsd({
      pattern: subsidy,
      name: "Subsidy",
      color: colors.mining.subsidy,
    }),
    ...satsBtcUsd({
      pattern: fee,
      name: "Fees",
      color: colors.mining.fee,
    }),
  ];
}

/**
 * Ratio bottom series
 * @param {AnyRatioPattern} ratio
 * @returns {AnyFetchedSeriesBlueprint[]}
 */
export function ratioBottomSeries(ratio) {
  return [
    baseline({
      series: ratio.ratio,
      name: "Ratio",
      unit: Unit.ratio,
      base: 1,
    }),
  ];
}

// ============================================================================
// Grouped Rolling Windows + Cumulative
// ============================================================================

/**
 * List-only primitive: rolling window charts + cumulative for a flat
 * cohort list, no "All" aggregate. Each cohort's `defaultActive` (if
 * present) is forwarded to `seriesFn`.
 * @template {{ name: string, color: Color, defaultActive?: boolean }} T
 * @param {Object} args
 * @param {readonly T[]} args.list
 * @param {(name: string) => string} args.title
 * @param {string} args.metricTitle
 * @param {(c: T, windowKey: "_24h" | "_1w" | "_1m" | "_1y") => AnySeriesPattern} args.getWindowSeries
 * @param {(c: T) => AnySeriesPattern} args.getCumulativeSeries
 * @param {(args: { series: AnySeriesPattern, name: string, color: Color, unit: Unit, defaultActive?: boolean }) => AnyFetchedSeriesBlueprint} args.seriesFn
 * @param {Unit} args.unit
 * @returns {PartialOptionsTree}
 */
export function groupedWindowsCumulative({
  list,
  title,
  metricTitle,
  getWindowSeries,
  getCumulativeSeries,
  seriesFn,
  unit,
}) {
  return [
    ...ROLLING_WINDOWS.map((w) => ({
      name: w.name,
      title: title(`${w.title} ${metricTitle}`),
      bottom: list.map((c) =>
        seriesFn({
          series: getWindowSeries(c, w.key),
          name: c.name,
          color: c.color,
          unit,
          defaultActive: c.defaultActive,
        }),
      ),
    })),
    {
      name: "Cumulative",
      title: title(`Cumulative ${metricTitle}`),
      bottom: list.map((c) =>
        seriesFn({
          series: getCumulativeSeries(c),
          name: c.name,
          color: c.color,
          unit,
          defaultActive: c.defaultActive,
        }),
      ),
    },
  ];
}

/**
 * "With all" variant: same as {@link groupedWindowsCumulative} plus an
 * "All" aggregate appended to each chart with `defaultActive: false`.
 * Composes on top of the primitive.
 * @template {{ name: string, color: Color }} T
 * @template {{ name: string, color: Color }} A
 * @param {Object} args
 * @param {readonly T[]} args.list
 * @param {A} args.all
 * @param {(name: string) => string} args.title
 * @param {string} args.metricTitle
 * @param {(c: T | A, windowKey: "_24h" | "_1w" | "_1m" | "_1y") => AnySeriesPattern} args.getWindowSeries
 * @param {(c: T | A) => AnySeriesPattern} args.getCumulativeSeries
 * @param {(args: { series: AnySeriesPattern, name: string, color: Color, unit: Unit, defaultActive?: boolean }) => AnyFetchedSeriesBlueprint} args.seriesFn
 * @param {Unit} args.unit
 * @returns {PartialOptionsTree}
 */
export function groupedWindowsCumulativeWithAll({
  list,
  all,
  title,
  metricTitle,
  getWindowSeries,
  getCumulativeSeries,
  seriesFn,
  unit,
}) {
  return groupedWindowsCumulative({
    list: [...list, { ...all, name: "All", defaultActive: false }],
    title,
    metricTitle,
    getWindowSeries,
    getCumulativeSeries,
    seriesFn,
    unit,
  });
}

/**
 * USD variant: windows access .sum[key].usd, cumulative accesses .cumulative.usd
 * @template {{ name: string, color: Color }} T
 * @template {{ name: string, color: Color }} A
 * @param {Object} args
 * @param {readonly T[]} args.list
 * @param {A} args.all
 * @param {(name: string) => string} args.title
 * @param {string} args.metricTitle
 * @param {(c: T | A) => { sum: Record<string, { usd: AnySeriesPattern }>, cumulative: { usd: AnySeriesPattern } }} args.getMetric
 * @param {(args: { series: AnySeriesPattern, name: string, color: Color, unit: Unit }) => AnyFetchedSeriesBlueprint} [args.seriesFn]
 * @returns {PartialOptionsTree}
 */
export function groupedWindowsCumulativeUsd({
  list,
  all,
  title,
  metricTitle,
  getMetric,
  seriesFn = line,
}) {
  return groupedWindowsCumulativeWithAll({
    list,
    all,
    title,
    metricTitle,
    seriesFn,
    unit: Unit.usd,
    getWindowSeries: (c, key) => getMetric(c).sum[key].usd,
    getCumulativeSeries: (c) => getMetric(c).cumulative.usd,
  });
}

/**
 * Multi-unit variant: windows access .sum[key] as satsBtcUsd pattern, cumulative same
 * @template {{ name: string, color: Color }} T
 * @template {{ name: string, color: Color }} A
 * @param {Object} args
 * @param {readonly T[]} args.list
 * @param {A} args.all
 * @param {(name: string) => string} args.title
 * @param {string} args.metricTitle
 * @param {(c: T | A) => { sum: Record<string, AnyValuePattern>, cumulative: AnyValuePattern }} args.getMetric
 * @returns {PartialOptionsTree}
 */
export function groupedWindowsCumulativeSatsBtcUsd({
  list,
  all,
  title,
  metricTitle,
  getMetric,
}) {
  return [
    ...ROLLING_WINDOWS.map((w) => ({
      name: w.name,
      title: title(`${w.title} ${metricTitle}`),
      bottom: flatMapCohortsWithAll(list, all, (c) =>
        satsBtcUsd({
          pattern: getMetric(c).sum[w.key],
          name: c.name,
          color: c.color,
        }),
      ),
    })),
    {
      name: "Cumulative",
      title: title(`Cumulative ${metricTitle}`),
      bottom: flatMapCohortsWithAll(list, all, (c) =>
        satsBtcUsd({
          pattern: getMetric(c).cumulative,
          name: c.name,
          color: c.color,
        }),
      ),
    },
  ];
}
