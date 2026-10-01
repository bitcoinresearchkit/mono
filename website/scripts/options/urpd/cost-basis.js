import { colors } from "../../utils/colors.js";
import { Unit } from "../../utils/units.js";
import { line, price, pricePercentileSeries } from "../series.js";
import { URPD_COHORTS } from "./cohorts.js";

/**
 * @param {string} label
 * @param {Bitview.SeriesTree_Cointime_Urpd | Bitview.SeriesTree_Coinflow_Urpd} urpd
 * @returns {PartialOptionsGroup}
 */
export function createCostBasisSection(label, urpd) {
  return {
    name: "Cost Basis",
    tree: URPD_COHORTS.map(({ key: cohort, name }) => ({
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
                  series: urpd[cohort].capitalizedPrice,
                  name: "Capitalized Price",
                  color: colors.capitalized,
                }),
              ]
            : []),
          ...pricePercentileSeries(urpd[cohort].costBasis[key]),
        ],
        bottom:
          key === "perDollar"
            ? [
                line({
                  series: urpd[cohort].capitalizedPrice.ratio,
                  name: "Spot / Capitalized Price",
                  color: colors.capitalized,
                  unit: Unit.ratio,
                }),
              ]
            : [],
      })),
    })),
  };
}
