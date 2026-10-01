import { colors } from "../../utils/colors.js";
import { percentRatio } from "../series.js";
import { URPD_COHORTS } from "./cohorts.js";

/**
 * @param {string} label
 * @param {Bitview.SeriesTree_Cointime_Urpd | Bitview.SeriesTree_Coinflow_Urpd} urpd
 * @returns {PartialOptionsGroup}
 */
export function createSupplyDensitySection(label, urpd) {
  return {
    name: "Supply Density",
    tree: URPD_COHORTS.map(({ key, name: cohortName }) => {
      const name = key === "sth" ? "STH (<5M)" : cohortName;
      return {
        name,
        title: `Bitcoin ${name} ${label}-Weighted Supply Density`,
        bottom: [
          ...percentRatio({
            pattern: urpd[key].supplyDensity.total,
            name: "Total",
            color: colors.bitcoin,
          }),
          ...percentRatio({
            pattern: urpd[key].supplyDensity.inProfit,
            name: "In Profit",
            color: colors.profit,
          }),
          ...percentRatio({
            pattern: urpd[key].supplyDensity.inLoss,
            name: "In Loss",
            color: colors.loss,
          }),
        ],
      };
    }),
  };
}
