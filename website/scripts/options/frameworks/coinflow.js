import { createWeightedUrpdSection } from "../urpd/index.js";
import { bitview } from "../../utils/client.js";
import { colors } from "../../utils/colors.js";
import { Unit } from "../../utils/units.js";
import { AGE_CUTOFFS } from "../age-cutoffs.js";
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
  const { coinflow } = bitview.series;
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

  const priceCohorts = [
    ...frameworkCohorts.map(({ name, color, tree }) => ({
      name,
      color,
      price: tree.price,
      capitalizedPrice: tree.capitalizedPrice,
    })),
    ...AGE_CUTOFFS.map(({ key, name }, index, all) => ({
      name,
      color: colors.at(index, all.length),
      price: coinflow[`${key}Price`],
      capitalizedPrice: coinflow[`${key}CapitalizedPrice`],
    })),
  ];

  return {
    name: "Coinflow",
    tree: [
      {
        name: "Price",
        tree: [
          {
            name: "Compare",
            title: "Coinflow Price by Holder Age",
            top: priceCohorts.map(({ name, color, price: cohortPrice }) =>
              price({
                series: cohortPrice,
                name,
                color,
              }),
            ),
            bottom: priceCohorts.map(({ name, color, price: cohortPrice }) =>
              line({
                series: cohortPrice.ratio,
                name: `Spot / ${name}`,
                color,
                unit: Unit.ratio,
              }),
            ),
          },
          ...priceCohorts.map(({ name, color, price: cohortPrice }) => {
            const title =
              name === "All" ? "Coinflow Price" : `${name} Coinflow Price`;
            const [chart] = simplePriceRatioTree({
              pattern: cohortPrice,
              title,
              legend: name,
              color,
            });
            return { ...chart, name };
          }),
        ],
      },
      {
        name: "Capitalized Price",
        tree: [
          {
            name: "Compare",
            title: "Coinflow Capitalized Price by Holder Age",
            top: priceCohorts.map(({ name, color, capitalizedPrice }) =>
              price({ series: capitalizedPrice, name, color }),
            ),
          },
          ...priceCohorts.map(({ name, color, capitalizedPrice }) => {
            const title =
              name === "All"
                ? "Coinflow Capitalized Price"
                : `${name} Coinflow Capitalized Price`;
            const [chart] = simplePriceRatioTree({
              pattern: capitalizedPrice,
              title,
              legend: name,
              color,
            });
            return { ...chart, name };
          }),
        ],
      },

      createWeightedUrpdSection("Coinflow", coinflow.urpd),
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
              ageRangeRatioChart(ranges, "spendingRate", "Spending Rate"),
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
