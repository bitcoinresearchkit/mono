import { colors } from "../../utils/colors.js";
import { percentRatio } from "../series.js";

/**
 * @param {string} label
 * @param {Bitview.CapitalizedCostSupplyPattern["supplyDensity"]} densities
 * @returns {PartialOptionsGroup}
 */
export function createSupplyDensitySection(label, densities) {
  return {
    name: "Supply Density",
    tree: /** @type {const} */ ([
      { key: "all", name: "All" },
      { key: "under4m", name: "<4M" },
      { key: "under5m", name: "STH (<5M)" },
      { key: "under6m", name: "<6M" },
    ]).map(({ key, name }) => ({
      name,
      title: `Bitcoin ${name} ${label}-Weighted Supply Density`,
      bottom: [
        ...percentRatio({
          pattern: densities[key].total,
          name: "Total",
          color: colors.bitcoin,
        }),
        ...percentRatio({
          pattern: densities[key].inProfit,
          name: "In Profit",
          color: colors.profit,
        }),
        ...percentRatio({
          pattern: densities[key].inLoss,
          name: "In Loss",
          color: colors.loss,
        }),
      ],
    })),
  };
}
