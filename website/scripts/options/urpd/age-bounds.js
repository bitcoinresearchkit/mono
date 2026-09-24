import { colors } from "../../utils/colors.js";
import { price } from "../series.js";

/**
 * @param {Bitview.SeriesTree_Cohorts_Urpd_AgeBounds} bounds
 * @returns {PartialOptionsGroup}
 */
export function createAgeBoundsSection(bounds) {
  return {
    name: "Age Bounds",
    tree: /** @type {const} */ ([
      { key: "under4m", name: "<4M" },
      { key: "under5m", name: "<5M" },
      { key: "under6m", name: "<6M" },
    ]).map(({ key, name }) => ({
      name,
      title: `${name} URPD Cost Basis Min/Max`,
      top: [
        price({ series: bounds[key].min, name: "Min", color: colors.stat.min }),
        price({ series: bounds[key].max, name: "Max", color: colors.stat.max }),
      ],
    })),
  };
}
