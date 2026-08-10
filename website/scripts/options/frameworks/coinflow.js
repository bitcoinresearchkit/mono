import { brk } from "../../utils/client.js";
import { colors } from "../../utils/colors.js";
import { Unit } from "../../utils/units.js";
import { ageRanges } from "../age-ranges.js";
import { line, price } from "../series.js";
import { satsBtcUsd, simplePriceRatioTree } from "../shared.js";

/**
 * @typedef {Object} CoinflowAgeRange
 * @property {string} name
 * @property {Color} color
 * @property {{
 *   mobility: AnySeriesPattern,
 *   spendingRate: AnySeriesPattern,
 *   spendingExposure: AnySeriesPattern,
 *   supply: { mobile: AnyValuePattern, immobile: AnyValuePattern },
 * }} tree
 */

/**
 * @param {readonly CoinflowAgeRange[]} ranges
 * @param {"mobility" | "spendingRate" | "spendingExposure"} key
 * @param {string} name
 * @returns {PartialChartOption}
 */
function ageRangeRatioChart(ranges, key, name) {
  return {
    name,
    title: `${name} by UTXO Age`,
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
 * @param {readonly CoinflowAgeRange[]} ranges
 * @param {"mobile" | "immobile"} key
 * @param {string} name
 * @returns {PartialChartOption}
 */
function ageRangeSupplyChart(ranges, key, name) {
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
 * Create Coinflow section.
 * @returns {PartialOptionsGroup}
 */
export function createCoinflowSection() {
  const { coinflow } = brk.series.frameworks;
  const ranges = ageRanges.map(({ key, ...range }) => ({
    ...range,
    tree: {
      spendingRate: coinflow.ageRange.spendingRate[key],
      spendingExposure: coinflow.ageRange.spendingExposure[key],
      mobility: coinflow.ageRange.spendingExposure.mobility[key],
      supply: {
        mobile: coinflow.ageRange.supply.mobile[key],
        immobile: coinflow.ageRange.supply.immobile[key],
      },
    },
  }));
  const terms = [
    { name: "STH", color: colors.term.short, tree: coinflow.sth },
    { name: "LTH", color: colors.term.long, tree: coinflow.lth },
  ];
  const cohorts = [
    { name: "All", color: colors.loss, tree: coinflow },
    ...terms,
  ];
  const frameworkCohorts = [
    { name: "All", color: colors.coinflow, tree: coinflow },
    ...terms,
  ];

  const horizons = /** @type {const} */ ([
    { key: "_8y", name: "8Y" },
    { key: "_4y", name: "4Y" },
    { key: "_2y", name: "2Y" },
    { key: "_1y", name: "1Y" },
    { key: "_6m", name: "6M" },
    { key: "_3m", name: "3M" },
    { key: "_1m", name: "1M" },
  ]).map((horizon, index, all) => ({
    ...horizon,
    color: colors.at(index, all.length),
  }));

  return {
    name: "Coinflow",
    tree: [
      {
        name: "Price",
        tree: [
          {
            name: "Compare",
            title: "Coinflow Price by Holder Term",
            top: frameworkCohorts.map(({ name, color, tree }) =>
              price({
                series: tree.price,
                name,
                color,
              }),
            ),
            bottom: frameworkCohorts.map(({ name, color, tree }) =>
              line({
                series: tree.price.ratio,
                name: `Spot / ${name}`,
                color,
                unit: Unit.ratio,
              }),
            ),
          },
          ...frameworkCohorts.map(({ name, color, tree }) => {
            const title =
              name === "All" ? "Coinflow Price" : `${name} Coinflow Price`;
            const [chart] = simplePriceRatioTree({
              pattern: tree.price,
              title,
              legend: name,
              color,
            });
            return { ...chart, name };
          }),
        ],
      },
      {
        name: "Capitalization",
        tree: [
          {
            name: "Compare",
            title: "Coinflow Cap by Holder Term",
            bottom: frameworkCohorts.map(({ name, color, tree }) =>
              line({
                series: tree.cap.usd,
                name,
                color,
                unit: Unit.usd,
              }),
            ),
          },
          ...frameworkCohorts.map(({ name, color, tree }) => ({
            name,
            title: name === "All" ? "Coinflow Cap" : `${name} Coinflow Cap`,
            bottom: [
              line({
                series: tree.cap.usd,
                name,
                color,
                unit: Unit.usd,
              }),
            ],
          })),
        ],
      },
      {
        name: "Supply",
        tree: [
          {
            name: "Overview",
            title: "Mobile vs Immobile Supply",
            bottom: [
              ...satsBtcUsd({
                pattern: coinflow.supply.mobile,
                name: "Mobile",
                color: colors.mobile,
              }),
              ...satsBtcUsd({
                pattern: coinflow.supply.immobile,
                name: "Immobile",
                color: colors.immobile,
              }),
            ],
          },
          {
            name: "By Holder Term",
            tree: [
              {
                name: "Mobile",
                title: "Mobile Supply by Holder Term",
                bottom: terms.flatMap(({ name, color, tree }) =>
                  satsBtcUsd({
                    pattern: tree.supply.mobile,
                    name,
                    color,
                  }),
                ),
              },
              {
                name: "Immobile",
                title: "Immobile Supply by Holder Term",
                bottom: terms.flatMap(({ name, color, tree }) =>
                  satsBtcUsd({
                    pattern: tree.supply.immobile,
                    name,
                    color,
                  }),
                ),
              },
            ],
          },
          {
            name: "By UTXO Age",
            tree: [
              ageRangeSupplyChart(ranges, "mobile", "Mobile"),
              ageRangeSupplyChart(ranges, "immobile", "Immobile"),
            ],
          },
          {
            name: "In Loss",
            tree: [
              {
                name: "By Holder Term",
                title: "Mobile Supply in Loss by Holder Term",
                bottom: cohorts.map(({ name, color, tree }) =>
                  line({
                    series: tree.supply.mobile.inLoss.share,
                    name,
                    color,
                    unit: Unit.ratio,
                  }),
                ),
              },
              {
                name: "By Horizon",
                tree: cohorts.map(({ name, tree }) => ({
                  name,
                  title:
                    name === "All"
                      ? "Mobile Supply in Loss by Horizon"
                      : `${name} Mobile Supply in Loss by Horizon`,
                  bottom: horizons.map((horizon) =>
                    line({
                      series:
                        tree.horizon[horizon.key].supply.inLoss.share,
                      name: horizon.name,
                      color: horizon.color,
                      unit: Unit.ratio,
                    }),
                  ),
                })),
              },
            ],
          },
        ],
      },
      {
        name: "Activity",
        tree: [
          {
            name: "By UTXO Age",
            tree: [
              ageRangeRatioChart(ranges, "mobility", "Mobility"),
              ageRangeRatioChart(
                ranges,
                "spendingRate",
                "Spending Rate",
              ),
              ageRangeRatioChart(
                ranges,
                "spendingExposure",
                "Spending Exposure",
              ),
            ],
          },
        ],
      },
    ],
  };
}
