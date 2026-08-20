/**
 * Profitability section builders
 *
 * Capability tiers:
 * - Full (All/STH/LTH): full unrealized with rel series, invested capital, sentiment;
 *   full realized with relToRcap, peakRegret, profitToLossRatio, grossPnl
 * - Mid (Core/AgeRange): unrealized profit/loss/netPnl/nupl (no rel, no invested, no sentiment);
 *   realized with netPnl + delta (no relToRcap, no peakRegret)
 * - Basic (UtxoAmount, Empty, Address): nupl only unrealized;
 *   basic realized profit/loss (no netPnl, no relToRcap)
 */

import { Unit } from "../../utils/units.js";
import {
  ROLLING_WINDOWS,
  line,
  dotted,
  baseline,
  percentRatio,
  percentRatioBaseline,
  sumsTreeBaseline,
  rollingPercentRatioTree,
  mapWindows,
} from "../series.js";
import { colors } from "../../utils/colors.js";
import { priceLine } from "../constants.js";
import {
  mapCohortsWithAll,
  flatMapCohortsWithAll,
  groupedWindowsCumulativeUsd,
} from "../shared.js";

// ============================================================================
// Core Series Builders
// ============================================================================


// ============================================================================
// Unrealized P&L Builders
// ============================================================================

/**
 * Overview chart: net + profit + loss inverted (active), loss raw (hidden)
 * @param {{ usd: AnySeriesPattern }} profit
 * @param {{ usd: AnySeriesPattern, negative: AnySeriesPattern }} loss
 * @param {AnySeriesPattern} netPnlUsd
 * @param {(name: string) => string} title
 * @returns {PartialChartOption}
 */
function unrealizedOverview(profit, loss, netPnlUsd, title) {
  return {
    name: "Overview",
    title: title("Unrealized P&L"),
    bottom: [
      baseline({ series: netPnlUsd, name: "Net", unit: Unit.usd }),
      dotted({
        series: profit.usd,
        name: "Profit",
        color: colors.profit,
        unit: Unit.usd,
      }),
      dotted({
        series: loss.negative,
        name: "Negated Loss",
        color: colors.loss,
        unit: Unit.usd,
      }),
      dotted({
        series: loss.usd,
        name: "Loss",
        color: colors.loss,
        unit: Unit.usd,
        defaultActive: false,
      }),
      priceLine({ unit: Unit.usd }),
    ],
  };
}

/**
 * Relative P&L chart: profit + loss as % of some denominator
 * @param {{ percent: AnySeriesPattern, ratio: AnySeriesPattern }} profit
 * @param {{ percent: AnySeriesPattern, ratio: AnySeriesPattern }} loss
 * @param {string} name
 * @param {(name: string) => string} title
 * @returns {PartialChartOption}
 */
function relPnlChart(profit, loss, name, title) {
  return {
    name,
    title: title(`Unrealized P&L (${name})`),
    bottom: [
      ...percentRatio({
        pattern: profit,
        name: "Profit",
        color: colors.profit,
      }),
      ...percentRatio({ pattern: loss, name: "Loss", color: colors.loss }),
    ],
  };
}

/** @param {{ percent: AnySeriesPattern, ratio: AnySeriesPattern }} net @param {{ percent: AnySeriesPattern, ratio: AnySeriesPattern }} profit @param {{ percent: AnySeriesPattern, ratio: AnySeriesPattern }} loss @param {string} name @param {(name: string) => string} title */
function relPnlChartWithNet(net, profit, loss, name, title) {
  return {
    name,
    title: title(`Unrealized P&L (${name})`),
    bottom: [
      ...percentRatioBaseline({ pattern: net, name: "Net" }),
      ...percentRatio({ pattern: profit, name: "Profit", color: colors.profit }),
      ...percentRatio({ pattern: loss, name: "Loss", color: colors.loss }),
    ],
  };
}

/**
 * Core unrealized items: Overview + Net + NUPL + Profit + Loss
 * @param {{ profit: { usd: AnySeriesPattern }, loss: { usd: AnySeriesPattern, negative: AnySeriesPattern }, netPnl: { usd: AnySeriesPattern }, nupl: NuplPattern }} u
 * @param {(name: string) => string} title
 * @returns {PartialOptionsTree}
 */
function unrealizedCore(u, title) {
  return [
    unrealizedOverview(u.profit, u.loss, u.netPnl.usd, title),
    {
      name: "Net",
      title: title("Net Unrealized P&L"),
      bottom: [baseline({ series: u.netPnl.usd, name: "Net", unit: Unit.usd })],
    },
    { name: "NUPL", title: title("NUPL"), bottom: nuplSeries(u.nupl) },
    {
      name: "Profit",
      title: title("Unrealized Profit"),
      bottom: [
        line({
          series: u.profit.usd,
          name: "Profit",
          color: colors.profit,
          unit: Unit.usd,
        }),
      ],
    },
    {
      name: "Loss",
      title: title("Unrealized Loss"),
      bottom: [
        line({
          series: u.loss.usd,
          name: "Loss",
          color: colors.loss,
          unit: Unit.usd,
        }),
      ],
    },
  ];
}

/**
 * Core unrealized items + Gross
 * @param {{ profit: { usd: AnySeriesPattern }, loss: { usd: AnySeriesPattern, negative: AnySeriesPattern }, netPnl: { usd: AnySeriesPattern }, grossPnl: { usd: AnySeriesPattern }, nupl: NuplPattern }} u
 * @param {(name: string) => string} title
 * @returns {PartialOptionsTree}
 */
function unrealizedCoreWithGross(u, title) {
  return [
    ...unrealizedCore(u, title),
    {
      name: "Gross",
      title: title("Gross Unrealized P&L"),
      bottom: [
        line({
          series: u.grossPnl.usd,
          name: "Gross",
          color: colors.gross,
          unit: Unit.usd,
        }),
      ],
    },
  ];
}

/**
 * % of Own P&L chart
 * @param {AllRelativePattern | FullRelativePattern} u
 * @param {(name: string) => string} title
 * @returns {PartialChartOption}
 */
function ownPnlChart(u, title) {
  return {
    name: "% of Own P&L",
    title: title("Unrealized P&L (% of Own P&L)"),
    bottom: [
      ...percentRatioBaseline({ pattern: u.netPnl.toOwnGrossPnl, name: "Net" }),
      ...percentRatio({
        pattern: u.profit.toOwnGrossPnl,
        name: "Profit",
        color: colors.profit,
        defaultActive: false,
      }),
      ...percentRatio({
        pattern: u.loss.toOwnGrossPnl,
        name: "Loss",
        color: colors.loss,
        defaultActive: false,
      }),
    ],
  };
}

/** @param {AllRelativePattern} u @param {(name: string) => string} title */
function unrealizedTreeAll(u, title) {
  return [
    ...unrealizedCoreWithGross(u, title),
    ownPnlChart(u, title),
    relPnlChart(u.profit.toMcap, u.loss.toMcap, "% of Market Cap", title),
  ];
}

/** @param {FullRelativePattern} u @param {(name: string) => string} title */
function unrealizedTreeFull(u, title) {
  return [
    ...unrealizedCoreWithGross(u, title),
    ownPnlChart(u, title),
    relPnlChart(u.profit.toMcap, u.loss.toMcap, "% of Market Cap", title),
    relPnlChartWithNet(u.netPnl.toOwnMcap, u.profit.toOwnMcap, u.loss.toOwnMcap, "% of Own Market Cap", title),
  ];
}

/** @param {FullRelativePattern} u @param {(name: string) => string} title */
function unrealizedTreeLongTerm(u, title) {
  return [
    ...unrealizedCoreWithGross(u, title),
    ownPnlChart(u, title),
    {
      name: "% of Market Cap",
      title: title("Unrealized P&L (% of Market Cap)"),
      bottom: [
        ...percentRatio({
          pattern: u.profit.toMcap,
          name: "Profit",
          color: colors.profit,
        }),
        ...percentRatio({
          pattern: u.loss.toMcap,
          name: "Loss",
          color: colors.loss,
        }),
      ],
    },
    relPnlChartWithNet(u.netPnl.toOwnMcap, u.profit.toOwnMcap, u.loss.toOwnMcap, "% of Own Market Cap", title),
  ];
}

// ============================================================================
// Invested Capital, Sentiment, NUPL
// ============================================================================

/**
 * Sentiment (Full unrealized only)
 * @param {FullRelativePattern | AllRelativePattern} u
 * @returns {AnyFetchedSeriesBlueprint[]}
 */
function sentimentSeries(u) {
  return [
    baseline({
      series: u.sentiment.net.usd,
      name: "Net Sentiment",
      unit: Unit.usd,
    }),
    line({
      series: u.sentiment.greedIndex.usd,
      name: "Greed Index",
      color: colors.profit,
      unit: Unit.usd,
      defaultActive: false,
    }),
    line({
      series: u.sentiment.painIndex.usd,
      name: "Pain Index",
      color: colors.loss,
      unit: Unit.usd,
      defaultActive: false,
    }),
  ];
}

/**
 * NUPL series
 * @param {NuplPattern} nupl
 * @returns {AnyFetchedSeriesBlueprint[]}
 */
function nuplSeries(nupl) {
  return [baseline({ series: nupl.ratio, name: "NUPL", unit: Unit.ratio })];
}

// ============================================================================
// Realized P&L Helpers
// ============================================================================

/**
 * Flat metric folder: Compare + windows + Cumulative + optional % of Realized Cap
 * @param {Object} args
 * @param {{ sum: Record<string, { usd: AnySeriesPattern }>, cumulative: { usd: AnySeriesPattern } }} args.pattern
 * @param {string} args.metricTitle
 * @param {Color} args.color
 * @param {(name: string) => string} args.title
 * @returns {PartialOptionsTree}
 */
function realizedMetricFolder({ pattern, metricTitle, color, title }) {
  return [
    {
      name: "Compare",
      title: title(`Realized ${metricTitle}`),
      bottom: ROLLING_WINDOWS.map((w) =>
        line({
          series: pattern.sum[w.key].usd,
          name: w.name,
          color: w.color,
          unit: Unit.usd,
        }),
      ),
    },
    ...ROLLING_WINDOWS.map((w) => ({
      name: w.name,
      title: title(`${w.title} Realized ${metricTitle}`),
      bottom: [
        line({
          series: pattern.sum[w.key].usd,
          name: metricTitle,
          color,
          unit: Unit.usd,
        }),
      ],
    })),
    {
      name: "Cumulative",
      title: title(`Cumulative Realized ${metricTitle}`),
      bottom: [
        line({
          series: pattern.cumulative.usd,
          name: metricTitle,
          color,
          unit: Unit.usd,
        }),
      ],
    },
  ];
}

/**
 * Net P&L folder: Compare + windows + Cumulative + optional % of Rcap + Change/
 * @param {Object} args
 * @param {NetPnlFullPattern | NetPnlBasicPattern} args.netPnl
 * @param {(name: string) => string} args.title
 * @param {PartialOptionsTree} [args.extraChange] - Additional change items (% of Mcap, % of Rcap)
 * @returns {PartialOptionsGroup}
 */
function realizedNetFolder({ netPnl, title, extraChange = [] }) {
  return {
    name: "Net",
    tree: [
      {
        name: "Compare",
        title: title("Net Realized P&L"),
        bottom: ROLLING_WINDOWS.map((w) =>
          baseline({
            series: netPnl.sum[w.key].usd,
            name: w.name,
            color: w.color,
            unit: Unit.usd,
          }),
        ),
      },
      ...ROLLING_WINDOWS.map((w) => ({
        name: w.name,
        title: title(`${w.title} Net Realized P&L`),
        bottom: [
          baseline({
            series: netPnl.sum[w.key].usd,
            name: "Net",
            unit: Unit.usd,
          }),
        ],
      })),
      {
        name: "Cumulative",
        title: title("Cumulative Net Realized P&L"),
        bottom: [
          baseline({
            series: netPnl.cumulative.usd,
            name: "Net",
            unit: Unit.usd,
          }),
        ],
      },
      {
        ...sumsTreeBaseline({
          windows: mapWindows(netPnl.delta.absolute, (c) => c.usd),
          title,
          metric: "Net Realized P&L Change",
          unit: Unit.usd,
          legend: "Change",
        }),
        name: "Change",
      },
      {
        name: "Growth Rate",
        tree: [
          ...rollingPercentRatioTree({
            windows: netPnl.delta.rate,
            title,
            metric: "Net Realized P&L Growth Rate",
          }).tree,
          ...extraChange,
        ],
      },
    ],
  };
}

/**
 * Realized overview folder: one chart per window showing net + profit (dotted) + neg. loss (dotted) + loss (hidden) + gross (hidden)
 * @param {Object} args
 * @param {{ sum: Record<string, { usd: AnySeriesPattern }> }} args.profit
 * @param {{ sum: Record<string, { usd: AnySeriesPattern }>, negative: { sum: Record<string, AnySeriesPattern> } }} args.loss
 * @param {{ sum: Record<string, { usd: AnySeriesPattern }> }} args.netPnl
 * @param {{ sum: Record<string, { usd: AnySeriesPattern }> }} [args.grossPnl]
 * @param {{ sum: Record<string, { usd: AnySeriesPattern }> }} [args.peakRegret]
 * @param {(name: string) => string} args.title
 * @returns {PartialOptionsGroup}
 */
function realizedOverviewFolder({
  profit,
  loss,
  netPnl,
  grossPnl,
  peakRegret,
  title,
}) {
  return {
    name: "Overview",
    tree: ROLLING_WINDOWS.map((w) => ({
      name: w.name,
      title: title(`${w.title} Realized P&L`),
      bottom: [
        baseline({
          series: netPnl.sum[w.key].usd,
          name: "Net",
          unit: Unit.usd,
        }),
        dotted({
          series: profit.sum[w.key].usd,
          name: "Profit",
          color: colors.profit,
          unit: Unit.usd,
        }),
        dotted({
          series: loss.negative.sum[w.key],
          name: "Negated Loss",
          color: colors.loss,
          unit: Unit.usd,
        }),
        dotted({
          series: loss.sum[w.key].usd,
          name: "Loss",
          color: colors.loss,
          unit: Unit.usd,
          defaultActive: false,
        }),
        ...(grossPnl
          ? [
              dotted({
                series: grossPnl.sum[w.key].usd,
                name: "Gross",
                color: colors.gross,
                unit: Unit.usd,
                defaultActive: false,
              }),
            ]
          : []),
        ...(peakRegret
          ? [
              dotted({
                series: peakRegret.sum[w.key].usd,
                name: "Peak Regret",
                color: colors.regret,
                unit: Unit.usd,
                defaultActive: false,
              }),
            ]
          : []),
        priceLine({ unit: Unit.usd }),
      ],
    })),
  };
}

// ============================================================================
// Realized Subfolder Builders
// ============================================================================

/**
 * Full realized subfolder (All/STH/LTH)
 * @param {FullRealizedProfitabilityPattern} r
 * @param {(name: string) => string} title
 * @returns {PartialOptionsGroup}
 */
function realizedSubfolderFull(r, title) {
  return {
    name: "Realized",
    tree: [
      realizedOverviewFolder({
        profit: r.profit,
        loss: r.loss,
        netPnl: r.netPnl,
        grossPnl: r.grossPnl,
        peakRegret: r.peakRegret,
        title,
      }),
      realizedNetFolder({
        netPnl: r.netPnl,
        title,
        extraChange: [
          {
            name: "% of Market Cap",
            title: title("Net Realized P&L Change (% of Market Cap)"),
            bottom: percentRatioBaseline({
              pattern: r.netPnl.change1m.toMcap,
              name: "1m Change",
            }),
          },
          {
            name: "% of Realized Cap",
            title: title("Net Realized P&L Change (% of Realized Cap)"),
            bottom: percentRatioBaseline({
              pattern: r.netPnl.change1m.toRcap,
              name: "1m Change",
            }),
          },
        ],
      }),
      {
        name: "Profit",
        tree: realizedMetricFolder({
          pattern: r.profit,
          metricTitle: "Profit",
          color: colors.profit,
          title,
        }),
      },
      {
        name: "Loss",
        tree: realizedMetricFolder({
          pattern: r.loss,
          metricTitle: "Loss",
          color: colors.loss,
          title,
        }),
      },
      {
        name: "Gross",
        tree: realizedMetricFolder({
          pattern: r.grossPnl,
          metricTitle: "Gross P&L",
          color: colors.gross,
          title,
        }),
      },
      {
        name: "P/L Ratio",
        tree: [
          {
            name: "Compare",
            title: title("Realized P/L Ratio"),
            bottom: ROLLING_WINDOWS.map((w) =>
              baseline({
                series: r.profitToLossRatio[w.key],
                name: w.name,
                color: w.color,
                unit: Unit.ratio,
                base: 1,
              }),
            ),
          },
          ...ROLLING_WINDOWS.map((w) => ({
            name: w.name,
            title: title(`${w.title} Realized P/L Ratio`),
            bottom: [
              baseline({
                series: r.profitToLossRatio[w.key],
                name: "P/L Ratio",
                unit: Unit.ratio,
                base: 1,
              }),
            ],
          })),
        ],
      },
      {
        name: "Peak Regret",
        tree: [
          {
            name: "Compare",
            title: title("Peak Regret"),
            bottom: ROLLING_WINDOWS.map((w) =>
              line({
                series: r.peakRegret.sum[w.key].usd,
                name: w.name,
                color: w.color,
                unit: Unit.usd,
              }),
            ),
          },
          ...ROLLING_WINDOWS.map((w) => ({
            name: w.name,
            title: title(`${w.title} Peak Regret`),
            bottom: [
              line({
                series: r.peakRegret.sum[w.key].usd,
                name: "Peak Regret",
                unit: Unit.usd,
              }),
            ],
          })),
          {
            name: "Cumulative",
            title: title("Cumulative Peak Regret"),
            bottom: [
              line({
                series: r.peakRegret.cumulative.usd,
                name: "Peak Regret",
                unit: Unit.usd,
              }),
            ],
          },
        ],
      },
    ],
  };
}

/**
 * Mid realized subfolder (Core/AgeRange — has netPnl + delta, no relToRcap/peakRegret)
 * @param {MidRealizedPattern} r
 * @param {(name: string) => string} title
 * @returns {PartialOptionsGroup}
 */
function realizedSubfolderMid(r, title) {
  return {
    name: "Realized",
    tree: [
      realizedOverviewFolder({
        profit: r.profit,
        loss: r.loss,
        netPnl: r.netPnl,
        title,
      }),
      realizedNetFolder({ netPnl: r.netPnl, title }),
      {
        name: "Profit",
        tree: realizedMetricFolder({
          pattern: r.profit,
          metricTitle: "Profit",
          color: colors.profit,
          title,
        }),
      },
      {
        name: "Loss",
        tree: realizedMetricFolder({
          pattern: r.loss,
          metricTitle: "Loss",
          color: colors.loss,
          title,
        }),
      },
    ],
  };
}

/**
 * Basic realized subfolder (no netPnl, no relToRcap)
 * @param {BasicRealizedProfitabilityPattern} r
 * @param {(name: string) => string} title
 * @returns {PartialOptionsGroup}
 */
function realizedSubfolderBasic(r, title) {
  return {
    name: "Realized",
    tree: [
      {
        name: "Profit",
        tree: realizedMetricFolder({
          pattern: r.profit,
          metricTitle: "Profit",
          color: colors.profit,
          title,
        }),
      },
      {
        name: "Loss",
        tree: realizedMetricFolder({
          pattern: r.loss,
          metricTitle: "Loss",
          color: colors.loss,
          title,
        }),
      },
    ],
  };
}

// ============================================================================
// Single Cohort Section Builders
// ============================================================================

/**
 * Basic profitability section (NUPL only unrealized, basic realized)
 * @param {{ cohort: UtxoCohortObject, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createProfitabilitySection({ cohort, title }) {
  return {
    name: "Profitability",
    tree: [
      { name: "Unrealized", tree: [{ name: "NUPL", title: title("NUPL"), bottom: nuplSeries(cohort.tree.unrealized.nupl) }] },
      realizedSubfolderBasic(cohort.tree.realized, title),
    ],
  };
}

/**
 * Realized profit and loss without unrealized metrics.
 * @param {{ cohort: AddrCohortObject, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createProfitabilitySectionRealized({ cohort, title }) {
  return {
    name: "Profitability",
    tree: [realizedSubfolderBasic(cohort.tree.realized, title)],
  };
}

/**
 * Profitability section with unrealized P&L + NUPL (no netPnl, no rel)
 * For: CohortWithoutRelative (p2ms, unknown, empty)
 * @param {{ cohort: CohortWithoutRelative, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createProfitabilitySectionWithProfitLoss({ cohort, title }) {
  const u = cohort.tree.unrealized;
  return {
    name: "Profitability",
    tree: [
      {
        name: "Unrealized",
        tree: [
          {
            name: "Overview",
            title: title("Unrealized P&L"),
            bottom: [
              line({ series: u.profit.usd, name: "Profit", color: colors.profit, unit: Unit.usd }),
              line({ series: u.loss.negative, name: "Negated Loss", color: colors.loss, unit: Unit.usd }),
              line({ series: u.loss.usd, name: "Loss", color: colors.loss, unit: Unit.usd, defaultActive: false }),
              priceLine({ unit: Unit.usd }),
            ],
          },
          { name: "NUPL", title: title("NUPL"), bottom: nuplSeries(u.nupl) },
          {
            name: "Profit",
            title: title("Unrealized Profit"),
            bottom: [
              line({
                series: u.profit.usd,
                name: "Profit",
                color: colors.profit,
                unit: Unit.usd,
              }),
            ],
          },
          {
            name: "Loss",
            title: title("Unrealized Loss"),
            bottom: [
              line({
                series: u.loss.usd,
                name: "Loss",
                color: colors.loss,
                unit: Unit.usd,
              }),
            ],
          },
        ],
      },
      realizedSubfolderBasic(cohort.tree.realized, title),
    ],
  };
}

/**
 * Section for All cohort
 * @param {{ cohort: CohortAll, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createProfitabilitySectionAll({ cohort, title }) {
  const u = cohort.tree.unrealized;
  const r = cohort.tree.realized;
  return {
    name: "Profitability",
    tree: [
      { name: "Unrealized", tree: unrealizedTreeAll(u, title) },
      realizedSubfolderFull(r, title),
      {
        name: "Sentiment",
        title: title("Market Sentiment"),
        bottom: sentimentSeries(u),
      },
    ],
  };
}

/**
 * Section for cohorts with full realized and unrealized profitability data.
 * This section does not use SOPR, whose shape differs between STH and LTH.
 * @param {{ cohort: { tree: { unrealized: FullRelativePattern, realized: FullRealizedProfitabilityPattern } }, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createProfitabilitySectionFull({ cohort, title }) {
  const u = cohort.tree.unrealized;
  const r = cohort.tree.realized;
  return {
    name: "Profitability",
    tree: [
      { name: "Unrealized", tree: unrealizedTreeFull(u, title) },
      realizedSubfolderFull(r, title),
      {
        name: "Sentiment",
        title: title("Market Sentiment"),
        bottom: sentimentSeries(u),
      },
    ],
  };
}


/**
 * Section for LongTerm cohort
 * @param {{ cohort: CohortLongTerm, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createProfitabilitySectionLongTerm({ cohort, title }) {
  const u = cohort.tree.unrealized;
  const r = cohort.tree.realized;
  return {
    name: "Profitability",
    tree: [
      { name: "Unrealized", tree: unrealizedTreeLongTerm(u, title) },
      realizedSubfolderFull(r, title),
      {
        name: "Sentiment",
        title: title("Market Sentiment"),
        bottom: sentimentSeries(u),
      },
    ],
  };
}

/**
 * Section for AgeRange cohorts (mid-tier: has unrealized profit/loss/netPnl, mid realized)
 * @param {{ cohort: CohortAgeRange, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createProfitabilitySectionWithInvestedCapitalPct({
  cohort,
  title,
}) {
  const u = cohort.tree.unrealized;
  const r = cohort.tree.realized;
  return {
    name: "Profitability",
    tree: [
      { name: "Unrealized", tree: unrealizedCore(u, title) },
      realizedSubfolderMid(r, title),
    ],
  };
}



// ============================================================================
// Grouped Cohort Helpers
// ============================================================================

/**
 * Grouped realized profit + loss items
 * @param {readonly CohortWithRealizedProfitLoss[]} list
 * @param {CohortAll} all
 * @param {(name: string) => string} title
 * @returns {PartialOptionsTree}
 */
function groupedRealizedProfitLossItems(list, all, title) {
  return [
    { name: "Profit", tree: groupedWindowsCumulativeUsd({ list, all, title, metricTitle: "Realized Profit", getMetric: (c) => c.tree.realized.profit }) },
    { name: "Loss", tree: groupedWindowsCumulativeUsd({ list, all, title, metricTitle: "Realized Loss", getMetric: (c) => c.tree.realized.loss }) },
  ];
}

/**
 * Grouped realized net item
 * @param {readonly (CohortAgeRange | CohortCore | CohortAll | CohortFull | CohortLongTerm)[]} list
 * @param {CohortAll} all
 * @param {(name: string) => string} title
 * @returns {PartialOptionsGroup}
 */
function groupedRealizedNetItem(list, all, title) {
  return { name: "Net", tree: groupedWindowsCumulativeUsd({ list, all, title, metricTitle: "Net Realized P&L", getMetric: (c) => c.tree.realized.netPnl, seriesFn: baseline }) };
}

/**
 * @param {readonly CohortWithRealizedProfitLoss[]} list
 * @param {CohortAll} all
 * @param {(name: string) => string} title
 * @returns {PartialOptionsGroup}
 */
function groupedRealizedSubfolder(list, all, title) {
  return { name: "Realized", tree: groupedRealizedProfitLossItems(list, all, title) };
}

/**
 * @param {readonly (CohortAgeRange | CohortCore)[]} list
 * @param {CohortAll} all
 * @param {(name: string) => string} title
 * @returns {PartialOptionsGroup}
 */
function groupedRealizedSubfolderMid(list, all, title) {
  return { name: "Realized", tree: [groupedRealizedNetItem(list, all, title), ...groupedRealizedProfitLossItems(list, all, title)] };
}

/**
 * Grouped net realized P&L delta (Absolute + Rate with all rolling windows)
 * @param {readonly (CohortAll | CohortFull | CohortLongTerm)[]} list
 * @param {CohortAll} all
 * @param {(name: string) => string} title
 * @returns {PartialOptionsTree}
 */
function groupedRealizedNetPnlDeltaItems(list, all, title) {
  return [
    {
      name: "Change",
      tree: ROLLING_WINDOWS.map((w) => ({
        name: w.name,
        title: title(`${w.title} Net Realized P&L Change`),
        bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
          baseline({
            series: tree.realized.netPnl.delta.absolute[w.key].usd,
            name,
            color,
            unit: Unit.usd,
          }),
        ),
      })),
    },
    {
      name: "Growth Rate",
      tree: ROLLING_WINDOWS.map((w) => ({
        name: w.name,
        title: title(`${w.title} Net Realized P&L Growth Rate`),
        bottom: flatMapCohortsWithAll(list, all, ({ name, color, tree }) =>
          percentRatioBaseline({
            pattern: tree.realized.netPnl.delta.rate[w.key],
            name,
            color,
          }),
        ),
      })),
    },
  ];
}

/**
 * Grouped realized subfolder for full cohorts
 * @param {readonly (CohortAll | CohortFull | CohortLongTerm)[]} list
 * @param {CohortAll} all
 * @param {(name: string) => string} title
 * @returns {PartialOptionsGroup}
 */
function groupedRealizedSubfolderFull(list, all, title) {
  return {
    name: "Realized",
    tree: [
      {
        name: "Net",
        tree: [
          ...groupedWindowsCumulativeUsd({
            list,
            all,
            title,
            metricTitle: "Net Realized P&L",
            getMetric: (c) => c.tree.realized.netPnl,
            seriesFn: baseline,
          }),
          ...groupedRealizedNetPnlDeltaItems(list, all, title),
        ],
      },
      ...groupedRealizedProfitLossItems(list, all, title),
      {
        name: "Gross",
        tree: groupedWindowsCumulativeUsd({
          list,
          all,
          title,
          metricTitle: "Realized Gross P&L",
          getMetric: (c) => c.tree.realized.grossPnl,
        }),
      },
      {
        name: "P/L Ratio",
        tree: ROLLING_WINDOWS.map((w) => ({
          name: w.name,
          title: title(`${w.title} Realized P/L Ratio`),
          bottom: mapCohortsWithAll(list, all, (c) =>
            baseline({
              series: c.tree.realized.profitToLossRatio[w.key],
              name: c.name,
              color: c.color,
              unit: Unit.ratio,
              base: 1,
            }),
          ),
        })),
      },
      {
        name: "Peak Regret",
        tree: groupedWindowsCumulativeUsd({
          list,
          all,
          title,
          metricTitle: "Peak Regret",
          getMetric: (c) => c.tree.realized.peakRegret,
        }),
      },
    ],
  };
}

/**
 * Grouped NUPL chart
 * @template {{ name: string, color: Color, tree: { unrealized: { nupl: NuplPattern } } }} T
 * @template {{ name: string, color: Color, tree: { unrealized: { nupl: NuplPattern } } }} A
 * @param {readonly T[]} list
 * @param {A} all
 * @param {(name: string) => string} title
 * @returns {PartialOptionsTree}
 */
function groupedNuplCharts(list, all, title) {
  return [
    {
      name: "NUPL",
      title: title("NUPL"),
      bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
        baseline({
          series: tree.unrealized.nupl.ratio,
          name,
          color,
          unit: Unit.ratio,
        }),
      ),
    },
  ];
}

/**
 * Grouped unrealized: Net → NUPL → Profit → Loss (no relative)
 * @param {readonly (CohortAgeRange | CohortCore)[]} list
 * @param {CohortAll} all
 * @param {(name: string) => string} title
 * @returns {PartialOptionsTree}
 */
function groupedUnrealizedMid(list, all, title) {
  return [
    {
      name: "Net",
      title: title("Net Unrealized P&L"),
      bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
        baseline({
          series: tree.unrealized.netPnl.usd,
          name,
          color,
          unit: Unit.usd,
        }),
      ),
    },
    ...groupedNuplCharts(list, all, title),
    {
      name: "Profit",
      title: title("Unrealized Profit"),
      bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
        line({
          series: tree.unrealized.profit.usd,
          name,
          color,
          unit: Unit.usd,
        }),
      ),
    },
    {
      name: "Loss",
      title: title("Unrealized Loss"),
      bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
        line({ series: tree.unrealized.loss.usd, name, color, unit: Unit.usd }),
      ),
    },
  ];
}

/**
 * Grouped unrealized: Net → NUPL → Profit → Loss → Relative(Market Cap)
 * @param {readonly (CohortFull | CohortLongTerm)[]} list
 * @param {CohortAll} all
 * @param {(name: string) => string} title
 * @returns {PartialOptionsTree}
 */
function groupedUnrealizedWithMarketCap(list, all, title) {
  return [
    ...groupedUnrealizedMid(list, all, title),
    {
      name: "% of Market Cap",
      tree: [
        {
          name: "Profit",
          title: title("Unrealized Profit (% of Market Cap)"),
          bottom: flatMapCohortsWithAll(list, all, ({ name, color, tree }) =>
            percentRatio({
              pattern: tree.unrealized.profit.toMcap,
              name,
              color,
            }),
          ),
        },
        {
          name: "Loss",
          title: title("Unrealized Loss (% of Market Cap)"),
          bottom: flatMapCohortsWithAll(list, all, ({ name, color, tree }) =>
            percentRatio({ pattern: tree.unrealized.loss.toMcap, name, color }),
          ),
        },
      ],
    },
  ];
}

/**
 * Grouped sentiment (full unrealized only)
 * @param {readonly (CohortAll | CohortFull | CohortLongTerm)[]} list
 * @param {CohortAll} all
 * @param {(name: string) => string} title
 * @returns {PartialOptionsGroup}
 */
function groupedSentiment(list, all, title) {
  return {
    name: "Sentiment",
    tree: [
      {
        name: "Net",
        title: title("Net Sentiment"),
        bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
          baseline({
            series: tree.unrealized.sentiment.net.usd,
            name,
            color,
            unit: Unit.usd,
          }),
        ),
      },
      {
        name: "Greed",
        title: title("Greed Index"),
        bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
          line({
            series: tree.unrealized.sentiment.greedIndex.usd,
            name,
            color,
            unit: Unit.usd,
          }),
        ),
      },
      {
        name: "Pain",
        title: title("Pain Index"),
        bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
          line({
            series: tree.unrealized.sentiment.painIndex.usd,
            name,
            color,
            unit: Unit.usd,
          }),
        ),
      },
    ],
  };
}

// ============================================================================
// Grouped Section Builders
// ============================================================================

/**
 * Grouped profitability section (basic — NUPL only)
 * @param {{ list: readonly (UtxoCohortObject | CohortWithoutRelative)[], all: CohortAll, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createGroupedProfitabilitySection({ list, all, title }) {
  return {
    name: "Profitability",
    tree: [
      { name: "Unrealized", tree: groupedNuplCharts(list, all, title) },
      groupedRealizedSubfolder(list, all, title),
    ],
  };
}

/**
 * Grouped realized profit and loss without unrealized metrics.
 * @param {{ list: readonly AddrCohortObject[], all: CohortAll, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createGroupedProfitabilitySectionRealized({
  list,
  all,
  title,
}) {
  return {
    name: "Profitability",
    tree: [groupedRealizedSubfolder(list, all, title)],
  };
}

/**
 * Grouped profitability with unrealized profit/loss + NUPL
 * For: CohortWithoutRelative (p2ms, unknown, empty)
 * @param {{ list: readonly CohortWithoutRelative[], all: CohortAll, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createGroupedProfitabilitySectionWithProfitLoss({
  list,
  all,
  title,
}) {
  return {
    name: "Profitability",
    tree: [
      {
        name: "Unrealized",
        tree: [
          ...groupedNuplCharts(list, all, title),
          {
            name: "Profit",
            title: title("Unrealized Profit"),
            bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
              line({
                series: tree.unrealized.profit.usd,
                name,
                color,
                unit: Unit.usd,
              }),
            ),
          },
          {
            name: "Loss",
            title: title("Unrealized Loss"),
            bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
              line({
                series: tree.unrealized.loss.usd,
                name,
                color,
                unit: Unit.usd,
              }),
            ),
          },
        ],
      },
      groupedRealizedSubfolder(list, all, title),
    ],
  };
}


/**
 * Grouped section for Core/AgeRange cohorts
 * @param {{ list: readonly (CohortAgeRange | CohortCore)[], all: CohortAll, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createGroupedProfitabilitySectionWithInvestedCapitalPct({
  list,
  all,
  title,
}) {
  return {
    name: "Profitability",
    tree: [
      { name: "Unrealized", tree: groupedUnrealizedMid(list, all, title) },
      groupedRealizedSubfolderMid(list, all, title),
    ],
  };
}

/**
 * Grouped section with NUPL + relToMcap
 * @param {{ list: readonly (CohortFull | CohortLongTerm)[], all: CohortAll, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createGroupedProfitabilitySectionWithNupl({
  list,
  all,
  title,
}) {
  return {
    name: "Profitability",
    tree: [
      {
        name: "Unrealized",
        tree: groupedUnrealizedWithMarketCap(list, all, title),
      },
      groupedRealizedSubfolderFull(list, all, title),
      groupedSentiment(list, all, title),
    ],
  };
}
