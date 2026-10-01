import { colors } from "../../utils/colors.js";
import { price } from "../series.js";
import { URPD_COHORTS } from "./cohorts.js";

/**
 * @param {Bitview.SeriesTree_Cohorts_Urpd_AgeBounds} bounds
 * @returns {PartialOptionsGroup}
 */
export function createAgeBoundsSection(bounds) {
  return {
    name: "Age Bounds",
    tree: URPD_COHORTS.map(({ key, name: cohortName }) => {
      const name = key === "sth" ? "<5M" : cohortName;
      return {
        name,
        title: `${name} URPD Cost Basis Min/Max`,
        top: [
          price({
            series: bounds[key].min,
            name: "Min",
            color: colors.stat.min,
          }),
          price({
            series: bounds[key].max,
            name: "Max",
            color: colors.stat.max,
          }),
        ],
      };
    }),
  };
}
