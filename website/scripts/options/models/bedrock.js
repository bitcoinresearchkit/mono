import { brk } from "../../utils/client.js";
import { colors } from "../../utils/colors.js";
import { Unit } from "../../utils/units.js";
import { line, price, pricePercentileSeries } from "../series.js";

const FLOOR_PERCENTILES = /** @type {const} */ ([
  { key: "pct95", name: "P95" },
  { key: "pct98", name: "P98" },
  { key: "pct99", name: "P99" },
  { key: "pct995", name: "P99.5" },
  { key: "pct999", name: "P99.9" },
]);

const LEVEL_PERCENTILES = /** @type {const} */ ([
  { key: "pct10", name: "P10" },
  { key: "pct20", name: "P20" },
  { key: "pct30", name: "P30" },
  { key: "pct40", name: "P40" },
  { key: "pct50", name: "P50" },
  { key: "pct60", name: "P60" },
  { key: "pct70", name: "P70" },
  { key: "pct80", name: "P80" },
  { key: "pct90", name: "P90" },
]);

/**
 * @typedef {Object} BedrockMode
 * @property {string} name
 * @property {AnySeriesPattern} inLoss
 * @property {{
 *   floor: Record<string, AnyPricePattern>,
 *   level: Record<string, AnyPricePattern>,
 *   lossThreshold: Record<string, AnySeriesPattern>,
 * }} tree
 */

/**
 * @param {BedrockMode} mode
 * @param {AnyPricePattern} ath
 * @returns {PartialChartOption}
 */
function modeChart(mode, ath) {
  return {
    name: mode.name,
    title: `Bitcoin Bedrock Model: ${mode.name}`,
    top: [
      ...FLOOR_PERCENTILES.map((percentile, index) =>
        price({
          series: mode.tree.floor[percentile.key],
          name: percentile.name,
          color: colors.bedrock.percentiles[index],
        }),
      ),
      ...LEVEL_PERCENTILES.map((percentile, index) =>
        price({
          series: mode.tree.level[percentile.key],
          name: `L${percentile.name.slice(1)}`,
          color: colors.bedrock.levels[index],
          style: 1,
        }),
      ),
      price({
        series: ath,
        name: "L100",
        color: colors.bedrock.levels[9],
        style: 1,
      }),
    ],
    bottom: [
      line({
        series: mode.inLoss,
        name: "Loss",
        color: colors.default,
        defaultActive: false,
        unit: Unit.ratio,
      }),
      ...FLOOR_PERCENTILES.map((percentile, index) =>
        line({
          series: mode.tree.lossThreshold[percentile.key],
          name: percentile.name,
          color: colors.bedrock.percentiles[index],
          defaultActive: false,
          unit: Unit.ratio,
        }),
      ),
    ],
  };
}

/**
 * @param {string} name
 * @param {PercentilesPattern} percentiles
 * @param {AnyPricePattern} p100
 * @returns {PartialChartOption}
 */
function costBasisChart(name, percentiles, p100) {
  return {
    name,
    title: `Bitcoin ${name}-Weighted Cost Basis Distribution (BTC-weighted)`,
    top: [
      price({
        series: p100,
        name: "P100",
        color: colors.stat.max,
        defaultActive: false,
      }),
      ...pricePercentileSeries(percentiles),
    ],
  };
}

/**
 * Create Bedrock model section.
 * @returns {PartialOptionsGroup}
 */
export function createBedrockSection() {
  const { market, cohorts, cointime, coinflow, bedrock } = brk.series;
  const horizonModes = /** @type {const} */ ([
    { key: "coinflow8y", horizon: "_8y", name: "Coinflow 8Y" },
    { key: "coinflow4y", horizon: "_4y", name: "Coinflow 4Y" },
    { key: "coinflow2y", horizon: "_2y", name: "Coinflow 2Y" },
    { key: "coinflow1y", horizon: "_1y", name: "Coinflow 1Y" },
    { key: "coinflow6m", horizon: "_6m", name: "Coinflow 6M" },
    { key: "coinflow3m", horizon: "_3m", name: "Coinflow 3M" },
    { key: "coinflow1m", horizon: "_1m", name: "Coinflow 1M" },
  ]).map((mode) => ({
    name: mode.name,
    tree: bedrock[mode.key],
    inLoss: coinflow.horizon[mode.horizon].supply.inLoss.share,
  }));

  const modes = /** @type {readonly BedrockMode[]} */ ([
    {
      name: "Raw",
      tree: bedrock.raw,
      inLoss: cohorts.relative.supply.inLoss.share.all.ratio,
    },
    {
      name: "Cointime",
      tree: bedrock.cointime,
      inLoss: cointime.supply.active.inLoss.share,
    },
    {
      name: "Coinflow",
      tree: bedrock.coinflow,
      inLoss: coinflow.supply.mobile.inLoss.share,
    },
    ...horizonModes,
  ]);

  return {
    name: "Bedrock",
    tree: [
      ...modes.map((mode) => modeChart(mode, market.ath.high)),
      {
        name: "Cost Basis",
        tree: [
          costBasisChart("Cointime", bedrock.costBasis.cointime, cohorts.costBasis.all.max),
          costBasisChart("Coinflow", bedrock.costBasis.coinflow, cohorts.costBasis.all.max),
        ],
      },
    ],
  };
}
