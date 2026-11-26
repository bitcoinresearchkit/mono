// @ts-check
// Where presets were before they moved: each row a place then and the place it is now, a chart's or a section's (and
// so of everything under it). A "*" in a step stands for any words there, the same on both sides, in order. Old
// addresses keep opening them (addresses.js), and charts saved from them find them (adoptPresets); a row is kept for
// good.

/** @type {[string, string][]} */
const PLACES_WERE = [
  ["Price › All-time high › Drawdown", "Price › All-time high, drawdown"],
  ["Price › All-time high › Days since", "Price › All-time high, days since"],
  ["Price › Moving averages › SMA", "Price › Moving averages, SMA"],
  ["Price › Moving averages › EMA", "Price › Moving averages, EMA"],
  ["Price › Range › High and low", "Price › Range"],
  ["Valuation › On-chain prices", "Valuation › On-chain prices › Compare"],
  ["Valuation › Cost basis clouds", "Valuation › On-chain prices › Cost basis clouds"],
  ["Valuation › Compass", "Valuation › On-chain prices › Compass"],
  ["Valuation › Cointime caps", "Valuation › On-chain prices › Cointime caps"],
  ["Valuation › Cointime prices", "Valuation › On-chain prices › Cointime prices"],
  ["Valuation › Rarity meter › * › Bands", "Valuation › Rarity meter › *, bands"],
  ["Valuation › Rarity meter › * › Level and score", "Valuation › Rarity meter › *, level and score"],
  ["Valuation › Rarity meter › Components › * › * › Bands", "Valuation › Rarity meter › Components › * › *, bands"],
  ["Valuation › Rarity meter › Components › * › * › Ratios", "Valuation › Rarity meter › Components › * › *, ratios"],
  ["Valuation › Rarity meter › Components › * › * › * › Bands", "Valuation › Rarity meter › Components › * › * › *, bands"],
  ["Valuation › Rarity meter › Components › * › * › * › Ratios", "Valuation › Rarity meter › Components › * › * › *, ratios"],
  ["Bedrock", "Valuation › Bedrock"],
  ["Valuation › Cumulative destruction", "Valuation › On-chain prices › Cumulative destruction"],
  ["Valuation › Delta price", "Valuation › On-chain prices › Delta price"],
  // The whole market at the top of its subjects; its totals and NUPL merged into the charts that were already there.
  ["Holders › All coins › Supply › Total", "Supply › Circulating"],
  ["Holders › All coins › Supply › In profit and loss", "Supply › In profit and loss"],
  ["Holders › All coins › Supply › Share in profit", "Supply › Share in profit"],
  ["Holders › All coins › Supply › Change", "Supply › Change"],
  ["Holders › All coins › Capital › Total", "Valuation › Market and realized cap"],
  ["Holders › All coins › Capital", "Valuation › Realized cap"],
  ["Holders › All coins › Cost basis", "Valuation › Cost basis"],
  ["Holders › All coins › Unrealized › NUPL", "Valuation › NUPL"],
  ["Holders › All coins › Realized › Profit and loss", "Profit and loss › Realized profit and loss"],
  ["Holders › All coins › Realized › *", "Profit and loss › *"],
  ["Holders › All coins › Unrealized › Profit and loss", "Profit and loss › Unrealized profit and loss"],
  ["Holders › All coins › Unrealized › Relative to market cap", "Profit and loss › Relative to market cap"],
  ["Holders › All coins › UTXOs › Change", "Activity › UTXO change"],
  ["Holders › All coins › UTXOs › Spent", "Activity › UTXOs spent"],
  ["Holders › All coins › Activity › *", "Activity › *"],
  ["Supply › Velocity", "Activity › Velocity"],
  ["Coin age › Last moved", "Supply › Last moved"],
  ["Cointime › Supply", "Supply › Vaulted and active"],
  ["Cointime › Activity", "Activity › Cointime"],
  ["Cointime › Value › Value *", "Activity › Cointime › Cointime value *"],
];

/** @typedef {{ steps: string[], wild: number }} Side */
/** @param {string} place @returns {Side} */
const side = (place) => ((steps) => ({ steps, wild: steps.filter((step) => step.includes("*")).length }))(place.split(" › "));
/**
 * Rows by the top section they start from (never a wildcard), the most specific first: the longest, then the fewest
 * wildcards.
 * @param {[Side, Side][]} pairs
 */
const byTop = (pairs) => {
  const at = /** @type {Map<string, [Side, Side][]>} */ (new Map());
  for (const pair of pairs.sort(([a], [b]) => b.steps.length - a.steps.length || a.wild - b.wild)) {
    if (pair[0].steps[0].includes("*")) throw new Error(`places-were.js: a row starts with a wildcard, ${pair[0].steps.join(" › ")}`);
    at.set(pair[0].steps[0], [...(at.get(pair[0].steps[0]) ?? []), pair]);
  }
  return at;
};
const sides = PLACES_WERE.map(([was, now]) => [side(was), side(now)]);
const toWas = byTop(sides.map(([was, now]) => [now, was])), toNow = byTop(sides.map(([was, now]) => [was, now]));
/** The words each "*" of `pattern` stands for at the start of `path`, or null. @param {string[]} pattern @param {string[]} path */
const match = (pattern, path) => {
  if (pattern.length > path.length) return null;
  const words = [];
  for (const [i, step] of pattern.entries()) {
    const star = step.indexOf("*");
    if (star < 0 ? step !== path[i] : !(path[i].length >= step.length - 1 && path[i].startsWith(step.slice(0, star)) && path[i].endsWith(step.slice(star + 1)))) return null;
    if (star >= 0) words.push(path[i].slice(star, path[i].length - (step.length - star - 1)));
  }
  return words;
};
/** Each row's other side for `path`, the most specific first. @param {Map<string, [Side, Side][]>} table @param {string[]} path */
const across = (table, path) => (table.get(path[0]) ?? []).flatMap(([from, to]) => ((words) => (words ? [[...to.steps.map((step) => (step.includes("*") ? step.replace("*", /** @type {string} */ (words.shift())) : step)), ...path.slice(from.steps.length)]] : []))(match(from.steps, path)));
/** The places a place was, a merged one's too, the most specific first (none if it never moved). @param {string[]} path */
export const placeWas = (path) => across(toWas, path);
/** A place moved since, where it is now; null if it never moved. @param {string[]} path */
export const placeNow = (path) => across(toNow, path)[0] ?? null;
