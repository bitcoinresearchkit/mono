import { bitview } from "../../utils/client.js";
import { colors } from "../../utils/colors.js";
import { Unit } from "../../utils/units.js";
import { line, price } from "../series.js";

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
 *   supplyInLossThreshold: Record<string, AnySeriesPattern>,
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
          series: mode.tree.supplyInLossThreshold[percentile.key],
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
 * Create Bedrock model section.
 * @returns {PartialOptionsGroup}
 */
export function createBedrockSection() {
  const { market, cohorts, cointime, coinflow, bedrock } = bitview.series;
  const modes = /** @type {readonly BedrockMode[]} */ ([
    {
      name: "Raw",
      tree: bedrock.raw,
      inLoss: cohorts.relative.supply.inLoss.share.all.ratio,
    },
    {
      name: "Cointime",
      tree: bedrock.cointime,
      inLoss: cointime.supply.active.inLoss.share.ratio,
    },
    {
      name: "Coinflow",
      tree: bedrock.coinflow,
      inLoss: coinflow.supply.mobile.inLoss.share,
    },
  ]);

  return {
    name: "Bedrock",
    tree: modes.map((mode) => modeChart(mode, market.ath.high)),
  };
}
