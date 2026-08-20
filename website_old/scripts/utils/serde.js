import { entries, fromEntries } from "./array.js";

export const serdeBool = {
  /**
   * @param {boolean} v
   */
  serialize(v) {
    return String(v);
  },
  /**
   * @param {string} v
   */
  deserialize(v) {
    if (v === "true" || v === "1") {
      return true;
    } else {
      return false;
    }
  },
};

export const INDEX_LABEL = /** @type {const} */ ({
  height: "blk",
  minute10: "10mn",
  minute30: "30mn",
  hour1: "1h",
  hour4: "4h",
  hour12: "12h",
  day1: "1d",
  day3: "3d",
  week1: "1w",
  month1: "1m",
  month3: "3m",
  month6: "6m",
  year1: "1y",
  year10: "10y",
  halving: "halv",
  epoch: "epch",
});

/** @typedef {typeof INDEX_LABEL} IndexLabelMap */
/** @typedef {keyof IndexLabelMap} ChartableIndex */
/** @typedef {IndexLabelMap[ChartableIndex]} IndexLabel */

export const INDEX_FROM_LABEL = fromEntries(
  entries(INDEX_LABEL).map(([k, v]) => [v, k]),
);
