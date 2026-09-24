import { colors } from "../../utils/colors.js";
import { price, pricePercentileSeries } from "../series.js";

/**
 * @param {string} label
 * @param {Bitview.CapitalizedCostSupplyPattern} urpd
 * @returns {PartialOptionsGroup}
 */
export function createCostBasisSection(label, urpd) {
  return {
    name: "Cost Basis",
    tree: /** @type {const} */ ([
      { key: "all", name: "All" },
      { key: "sth", name: "STH" },
      { key: "lth", name: "LTH" },
    ]).map(({ key: cohort, name }) => ({
      name,
      tree: /** @type {const} */ ([
        { key: "perCoin", name: "Per Coin" },
        { key: "perDollar", name: "Per Dollar" },
      ]).map(({ key, name: weight }) => ({
        name: weight,
        title: `Bitcoin ${name} ${label}-Weighted Cost Basis Distribution (${weight})`,
        top: [
          ...(key === "perDollar"
            ? [
                price({
                  series: urpd.capitalizedPrice[cohort],
                  name: "Capitalized Price",
                  color: colors.capitalized,
                }),
              ]
            : []),
          ...pricePercentileSeries(urpd.costBasis[cohort][key]),
        ],
      })),
    })),
  };
}
