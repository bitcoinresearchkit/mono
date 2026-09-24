import { createCostBasisSection } from "./cost-basis.js";
import { createSupplyDensitySection } from "./supply-density.js";

/**
 * @param {string} label
 * @param {Bitview.CapitalizedCostSupplyPattern} urpd
 * @returns {PartialOptionsGroup}
 */
export function createWeightedUrpdSection(label, urpd) {
  return {
    name: "URPD",
    tree: [
      createCostBasisSection(label, urpd),
      createSupplyDensitySection(label, urpd.supplyDensity),
    ],
  };
}
