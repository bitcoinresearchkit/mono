/** Constant helpers for creating price lines and reference lines */

import { colors } from "../utils/colors.js";
import { brk } from "../utils/client.js";
import { line } from "./series.js";

/**
 * Get constant pattern by number dynamically from tree
 * Examples: 0 → _0, 38.2 → _382, -1 → minus1
 * @param {BitviewClient["series"]["constants"]} constants
 * @param {number} num
 * @returns {AnySeriesPattern}
 */
export function getConstant(constants, num) {
  const key =
    num >= 0 ? `_${String(num).replace(".", "")}` : `minus${Math.abs(num)}`;
  const constant = /** @type {AnySeriesPattern | undefined} */ (
    /** @type {Record<string, AnySeriesPattern>} */ (constants)[key]
  );
  if (!constant) throw new Error(`Unknown constant: ${num} (key: ${key})`);
  return constant;
}

/**
 * Create a price line series (horizontal reference line)
 * @param {{ number?: number, name?: string } & Omit<(Parameters<typeof line>)[0], 'name' | 'series'>} args
 */
export function priceLine(args) {
  return line({
    ...args,
    series: getConstant(brk.series.constants, args.number || 0),
    name: args.name || `${args.number ?? 0}`,
    color: args.color ?? colors.gray,
    options: {
      lineStyle: args.style ?? 4,
      lastValueVisible: false,
      crosshairMarkerVisible: false,
      ...args.options,
    },
  });
}

/**
 * @param {{ numbers: number[] } & Omit<(Parameters<typeof priceLine>)[0], 'number'>} args
 */
export function priceLines(args) {
  return args.numbers.map((number) =>
    priceLine({
      ...args,
      number,
    }),
  );
}
