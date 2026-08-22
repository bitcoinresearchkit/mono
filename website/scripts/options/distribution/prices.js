/**
 * Prices section builders
 *
 * Structure (single cohort):
 * - Compare: Both prices on one chart
 * - Realized: Price + Ratio (MVRV)
 * - Capitalized: Price + Ratio
 *
 * Structure (grouped cohorts):
 * - Realized: Price + Ratio comparison across cohorts
 * - Capitalized: Price + Ratio comparison across cohorts
 *
 * Cohorts without percentile patterns use basic Price/Ratio charts.
 */

import { colors } from "../../utils/colors.js";
import { mapCohortsWithAll } from "../shared.js";
import { baseline, price } from "../series.js";
import { Unit } from "../../utils/units.js";

/**
 * Create prices section for cohorts with full ratio patterns
 * (CohortAll, CohortFull, CohortLongTerm)
 * @param {{ cohort: CohortAll | CohortFull | CohortLongTerm, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createPricesSectionFull({ cohort, title }) {
  const { tree, color } = cohort;
  return {
    name: "Prices",
    tree: [
      {
        name: "Compare",
        title: title("Realized Prices"),
        top: [
          price({ series: tree.realized.price, name: "Realized", color: colors.realized }),
          price({ series: tree.realized.capitalizedPrice, name: "Capitalized", color: colors.capitalized }),
        ],
      },
      {
        name: "Realized",
        title: title("Realized Price"),
        top: [
          price({
            series: tree.realized.price,
            name: "Realized",
            color,
          }),
        ],
        bottom: [
          baseline({
            series: tree.realized.price.ratio,
            name: "Ratio",
            unit: Unit.ratio,
            base: 1,
          }),
        ],
      },
      {
        name: "Capitalized",
        title: title("Capitalized Price"),
        top: [
          price({
            series: tree.realized.capitalizedPrice,
            name: "Capitalized",
            color,
          }),
        ],
        bottom: [
          baseline({
            series: tree.realized.capitalizedPrice.ratio,
            name: "Ratio",
            unit: Unit.ratio,
            base: 1,
          }),
        ],
      },
    ],
  };
}

/**
 * Create prices section for cohorts with basic ratio patterns only
 * (CohortCore, CohortBasic, CohortAddr, CohortWithoutRelative)
 * @param {{ cohort: CohortCore | CohortBasic | CohortAddr | CohortWithoutRelative | CohortAgeRange, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
export function createPricesSectionBasic({ cohort, title }) {
  const { tree, color } = cohort;
  return {
    name: "Prices",
    tree: [
      {
        name: "Realized",
        title: title("Realized Price"),
        top: [
          price({
            series: tree.realized.price,
            name: "Realized",
            color,
          }),
        ],
        bottom: [
          baseline({
            series: tree.realized.price.ratio,
            name: "Ratio",
            unit: Unit.ratio,
            base: 1,
          }),
        ],
      },
    ],
  };
}

/**
 * Create prices section for grouped cohorts
 * @param {{ list: readonly CohortObject[], all: CohortAll, title: (name: string) => string }} args
 * @returns {PartialOptionsGroup}
 */
/**
 * @param {readonly CohortWithRealizedPrice[]} list
 * @param {CohortAll} all
 * @param {(name: string) => string} title
 * @returns {PartialOptionsTree}
 */
function groupedRealizedPriceItems(list, all, title) {
  return [
    {
      name: "Realized",
      tree: [
        {
          name: "Price",
          title: title("Realized Price"),
          top: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
            price({ series: tree.realized.price, name, color }),
          ),
        },
        {
          name: "Ratio",
          title: title("Realized Price Ratio"),
          bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
            baseline({ series: tree.realized.mvrv, name, color, unit: Unit.ratio, base: 1 }),
          ),
        },
      ],
    },
  ];
}

/** @param {{ list: readonly CohortWithRealizedPrice[], all: CohortAll, title: (name: string) => string }} args */
export function createGroupedPricesSection({ list, all, title }) {
  return {
    name: "Prices",
    tree: groupedRealizedPriceItems(list, all, title),
  };
}

/** @param {{ list: readonly (CohortAll | CohortFull | CohortLongTerm)[], all: CohortAll, title: (name: string) => string }} args */
export function createGroupedPricesSectionFull({ list, all, title }) {
  return {
    name: "Prices",
    tree: [
      ...groupedRealizedPriceItems(list, all, title),
      {
        name: "Capitalized",
        tree: [
          {
            name: "Price",
            title: title("Capitalized Price"),
            top: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
              price({ series: tree.realized.capitalizedPrice, name, color }),
            ),
          },
          {
            name: "Ratio",
            title: title("Capitalized Price Ratio"),
            bottom: mapCohortsWithAll(list, all, ({ name, color, tree }) =>
              baseline({ series: tree.realized.capitalizedPrice.ratio, name, color, unit: Unit.ratio, base: 1 }),
            ),
          },
        ],
      },
    ],
  };
}
