import { Unit } from "../../../utils/units.js";
import { line, ROLLING_WINDOWS } from "../../series.js";
import { satsBtcUsd } from "../../shared.js";

/**
 * @typedef {{
 *   average: Record<"_24h" | "_1w" | "_1m" | "_1y", AnySeriesPattern>,
 *   sum: Record<"_24h" | "_1w" | "_1m" | "_1y", AnySeriesPattern>,
 *   cumulative: AnySeriesPattern,
 * }} CointimeAgeRangeCoindays
 *
 * @typedef {Object} CointimeAgeRange
 * @property {string} name
 * @property {Color} color
 * @property {{
 *   coindaysCreated: CointimeAgeRangeCoindays,
 *   coindaysConsumed: CointimeAgeRangeCoindays,
 *   coindaysStored: CointimeAgeRangeCoindays,
 *   wakefulness: AnySeriesPattern,
 *   dormancy: AnySeriesPattern,
 *   wakefulnessToDormancy: AnySeriesPattern,
 *   supply: { awake: AnyValuePattern, dormant: AnyValuePattern },
 * }} tree
 */

/**
 * @param {readonly CointimeAgeRange[]} ranges
 * @param {"wakefulness" | "dormancy" | "wakefulnessToDormancy"} key
 * @param {string} name
 * @param {string} legend
 * @returns {PartialChartOption}
 */
function activityChart(ranges, key, name, legend) {
  return {
    name,
    title: `${legend} by UTXO Age`,
    bottom: ranges.map((range) =>
      line({
        series: range.tree[key],
        name: range.name,
        color: range.color,
        unit: Unit.ratio,
      }),
    ),
  };
}

/**
 * @param {readonly CointimeAgeRange[]} ranges
 * @param {"awake" | "dormant"} key
 * @param {string} name
 * @returns {PartialChartOption}
 */
function supplyChart(ranges, key, name) {
  return {
    name,
    title: `${name} Supply by UTXO Age`,
    bottom: ranges.flatMap((range) =>
      satsBtcUsd({
        pattern: range.tree.supply[key],
        name: range.name,
        color: range.color,
      }),
    ),
  };
}

/**
 * @param {readonly CointimeAgeRange[]} ranges
 * @param {"coindaysCreated" | "coindaysConsumed" | "coindaysStored"} key
 * @param {string} name
 * @returns {PartialOptionsGroup}
 */
function coindaysTree(ranges, key, name) {
  return {
    name,
    tree: [
      ...ROLLING_WINDOWS.map((window) => ({
        name: window.name,
        title: `${window.title} ${name} by UTXO Age`,
        bottom: ranges.flatMap((range) => [
          line({
            series: range.tree[key].sum[window.key],
            name: range.name,
            color: range.color,
            unit: Unit.coindays,
          }),
          line({
            series: range.tree[key].average[window.key],
            name: `${range.name} Avg`,
            color: range.color,
            unit: Unit.coindays,
            defaultActive: false,
            style: 1,
          }),
        ]),
      })),
      {
        name: "Cumulative",
        title: `Cumulative ${name} by UTXO Age`,
        bottom: ranges.map((range) =>
          line({
            series: range.tree[key].cumulative,
            name: range.name,
            color: range.color,
            unit: Unit.coindays,
          }),
        ),
      },
    ],
  };
}

/**
 * @param {readonly CointimeAgeRange[]} ranges
 * @returns {PartialOptionsGroup}
 */
export function createCointimeAgeRangeSupplySection(ranges) {
  return {
    name: "By UTXO Age",
    tree: [
      supplyChart(ranges, "awake", "Awake"),
      supplyChart(ranges, "dormant", "Dormant"),
    ],
  };
}

/**
 * @param {readonly CointimeAgeRange[]} ranges
 * @returns {PartialOptionsGroup}
 */
export function createCointimeAgeRangeActivitySection(ranges) {
  return {
    name: "By UTXO Age",
    tree: [
      activityChart(
        ranges,
        "wakefulness",
        "Wakefulness",
        "Wakefulness",
      ),
      activityChart(ranges, "dormancy", "Dormancy", "Dormancy"),
      activityChart(
        ranges,
        "wakefulnessToDormancy",
        "Activity Ratio",
        "Wakefulness / Dormancy",
      ),
      {
        name: "Coindays",
        tree: [
          coindaysTree(ranges, "coindaysCreated", "Coindays Created"),
          coindaysTree(ranges, "coindaysConsumed", "Coindays Consumed"),
          coindaysTree(ranges, "coindaysStored", "Coindays Stored"),
        ],
      },
    ],
  };
}
