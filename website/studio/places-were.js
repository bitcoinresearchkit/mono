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
/** @param {Map<string, [Side, Side][]>} table @param {string[]} path @returns {string[] | null} */
const across = (table, path) => {
  for (const row of table.get(path[0]) ?? []) {
    const words = match(row[0].steps, path);
    if (words) return [...row[1].steps.map((step) => (step.includes("*") ? step.replace("*", /** @type {string} */ (words.shift())) : step)), ...path.slice(row[0].steps.length)];
  }
  return null;
};
/** A place as it was, by the most specific row that names it now; null if it never moved. @param {string[]} path */
export const placeWas = (path) => across(toWas, path);
/** A place moved since, where it is now; null if it never moved. @param {string[]} path */
export const placeNow = (path) => across(toNow, path);
