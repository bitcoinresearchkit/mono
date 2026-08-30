import { entries } from "../utils/array.js";
import { bitview } from "../utils/client.js";
import { colors } from "../utils/colors.js";

/**
 * Shared display metadata for every UTXO age range.
 *
 * Consumers attach their own tree because Distribution, Cointime, and
 * Coinflow expose different metrics for the same ranges.
 */
export const ageRanges = entries(bitview.AGE_RANGE_NAMES).map(
  ([key, names], index, all) => ({
    key,
    name: names.short,
    title: `UTXOs ${names.long}`,
    color: colors.at(index, all.length),
  }),
);
