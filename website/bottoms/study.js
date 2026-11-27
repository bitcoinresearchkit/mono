// @ts-check
// The Bottoms study, computed from the server's daily series: every line, held to the ranking's rules (score.js) at
// the cycle bottoms since 2017. No DOM: the page's worker runs it.

import { bottoms, miss, rank } from "./score.js";
import { POWER_LAW } from "/modules/power-law/index.js";
import { refitFloor } from "./refit.js";

const DAY = 864e5, DAY0 = Date.UTC(2009, 0, 1);
/** A date's day, counted as the server's day1 counts. */
export const dayOf = (/** @type {string} */ iso) => Math.round((Date.parse(iso) - DAY0) / DAY);

export const PRICE = "price_close", EPOCH = "halving_epoch";
/** The judged period starts in 2017: only bottoms from then on are ranked on. */
export const FROM = dayOf("2017-01-01");
/** The first day the page's chart shows: a year before the judged period, so the first fall is seen whole. */
export const SHOWN = dayOf("2016-01-01");
/** The server's weeks: a week's value is its last day's, the day `days × week + day` (its Sunday; the running week's, the last day so far). */
export const WEEK = { days: 7, day: 3 };
/** The week a day is in. */
export const weekOf = (/** @type {number} */ day) => Math.ceil((day - WEEK.day) / WEEK.days);
const GENESIS = dayOf("2009-01-03"), RAINBOW = dayOf("2012-01-01");

// ── The lines: each one a price by day, the set read from the server's catalog, so a line it starts serving joins ──

/** @typedef {"line" | "floor" | "rarity"} Kind how a line is made from its base: the base itself, one of Bedrock's floors, or one of the server's rarity levels */
/** @typedef {{ id: string, base: string, kind: Kind, level?: number }} Made */
/** @typedef {Made & { line: Float64Array }} Line */

/** A level as the server's ids write it, `p99_5`, read back: 99.5. */
const levelOf = (/** @type {string} */ tag) => +tag.replace("_", ".");
/** The rarity levels ranked: the cheap tail, where a bottom would sit; from 10% up, a level is a price seen one day in ten. */
const LOW = 5;
/** The days in a window or an age, as the server's ids write it (`200w`, `4y`, `18m`). */
const SPAN = /** @type {Record<string, number>} */ ({ h: 1 / 24, d: 1, w: 7, m: 30, y: 365 });
/** A window or an age in days, from the server's ids' way of writing it (`18m`). */
const daysOf = (/** @type {string} */ n, /** @type {string} */ unit) => +n * SPAN[unit];
/** A published power law fixed when it was fitted, by day: `log10 price = a + b · log10(days since genesis) − below`. */
const law = (/** @type {number} */ a, /** @type {number} */ b, /** @type {number} */ below) => (/** @type {Float64Array} */ close) => {
  const out = new Float64Array(close.length).fill(NaN);
  for (let d = GENESIS + 1; d < out.length; d++) out[d] = 10 ** (a + b * Math.log10(d - GENESIS) - below);
  return out;
};
/**
 * The bottom models worked out here from the date and the closes, by id, each as long as the closes, until the server
 * serves them; each with its author's own parameters, as their own source states them.
 */
export const WORKED = /** @type {Record<string, (close: Float64Array) => Float64Array | Promise<Float64Array>>} */ ({
  // Burger's support line (modules/power-law).
  burger_power_law_floor: law(POWER_LAW.intercept, POWER_LAW.slope, POWER_LAW.support),
  // Alexander Mooij's (btcpowerlaw.nl/about, its code): a trend of 10^-16.493 · days^5.688, the floor 0.42 of it.
  mooij_power_law_floor: law(-16.493, 5.688, -Math.log10(0.42)),
  // Sminston With's (sminstonwith.com): the 5% quantile line of every close before the day, from a year of closes on.
  sminston_power_law_floor: (close) => refitFloor(close, GENESIS, 0.05, 365),
  // The Rainbow chart's lowest band (blockchaincenter.net, its page's code): 10^(2.775 · ln(n + 1175) − 19.463), n the
  // days since 2012-01-01, from which it draws.
  rainbow_lowest_band: (close) => {
    const out = new Float64Array(close.length).fill(NaN);
    for (let d = RAINBOW; d < out.length; d++) out[d] = 10 ** (2.775 * Math.log(d - RAINBOW + 1175) - 19.463);
    return out;
  },
});
const alone = (/** @type {string} */ id) => /** @type {Made} */ ({ id, base: id, kind: "line" });
/** A line at one of its base's levels, the level as the server's ids write it. @returns {Made} */
const leveled = (/** @type {string} */ id, /** @type {string} */ base, /** @type {Kind} */ kind, /** @type {string} */ tag) => ({ id, base, kind, level: levelOf(tag) });
/** The cost-basis percentiles ranked: those under the median, where a floor would sit (the medians are ranked as the median realized and capitalized prices). */
const MEDIAN = 50;
/** @typedef {{ bases: Set<string>, bands: Made[], averages: Made[], percentiles: Made[], floors: Made[], rarity: Made[] }} Groups */
/**
 * The rules that make the lines from the names the server serves (its catalog), tried in order on each name; only
 * what they leave out is out. Each rule's `parts` are what every name its pattern takes contains: the catalog is
 * asked for those names alone (`NAMED`), so a rule and the names it reads are written together. `take` files a
 * matched name and says whether it was taken (a name not taken goes on to the next rule).
 * @type {{ parts: string[], pattern: RegExp, take: (m: RegExpExecArray, served: Set<string>, into: Groups) => boolean }[]}
 */
const RULES = [
  // A price the server ranks the price's ratio to (it serves `B_ratio_rarity_…`) is a reference price: ranked as it is
  // and at each low rarity level. Not the short-term holders' (young coins, which follow the price down rather than
  // hold under it), nor the age cohorts' (their realized prices alone are ranked, below).
  { parts: ["rarity"], pattern: /^(\w+?)_rarity_p(\d+(?:_\d+)?)$/, take: ([id, base, tag], served, into) => {
    if (/^(sth|age)_/.test(base) || !served.has(`${base}_ratio_rarity_p${tag}`)) return false;
    into.bases.add(base);
    if (levelOf(tag) <= LOW) into.rarity.push(leveled(id, base, "rarity", tag));
    return true;
  } },
  // The rarity meters, at their low levels.
  { parts: ["rarity"], pattern: /^(\w*rarity_meter)_p(\d+(?:_\d+)?)$/, take: ([id, base, tag], _, into) => levelOf(tag) <= LOW && (into.rarity.push(leveled(id, base, "rarity", tag)), true) },
  // The cost basis's percentiles under the median (`…cost_basis_by_coin_p5`: the price under which 5% of the coins last
  // moved; `by_dollar`: 5% of their dollars), of all holders and the long-term ones, unweighted or awake or mobile.
  { parts: ["cost_basis_by_"], pattern: /^(?:lth_)?(?:awake_|mobile_)?cost_basis_by_(?:coin|dollar)_p(\d+)$/, take: ([id, tag], _, into) => levelOf(tag) < MEDIAN && (into.percentiles.push(alone(id)), true) },
  // The realized prices of coins by age, the bands from a year old (`age_1y_to_18m`, by its youngest).
  { parts: ["_realized_price"], pattern: /^age_(\d+)([a-z])_to_\d+[a-z]_realized_price$/, take: ([id, n, unit], _, into) => daysOf(n, unit) >= SPAN.y && (into.bands.push(alone(id)), true) },
  // Puell's transferred price.
  { parts: ["transferred_price"], pattern: /^transferred_price$/, take: ([id], _, into) => (into.bases.add(id), true) },
  // Every Bedrock floor (`bedrock_W_floor_…`, W its weighting; the cost-basis lines above a floor are not floors).
  { parts: ["_floor_p"], pattern: /^(bedrock_[a-z]+_floor)_p(\d+(?:_\d+)?)$/, take: ([id, base, tag], _, into) => (into.floors.push(leveled(id, base, "floor", tag)), true) },
  // The price's moving averages and trailing lows of a year or more (shorter ones are trading levels, which a fall
  // cuts through).
  { parts: ["price_sma_", "price_ema_", "price_min_"], pattern: /^price_(?:sma|ema|min)_(\d+)([dwmy])$/, take: ([id, n, unit], _, into) => daysOf(n, unit) >= SPAN.y && (into.averages.push(alone(id)), true) },
];
/** The name parts the rules read: the catalog is asked for the names that contain one. */
export const NAMED = [...new Set(RULES.flatMap((rule) => rule.parts))];
/** Every line, from the names the server serves, in groups: the lines themselves, the lines worked out here, the percentiles, Bedrock's floors, then the rarity lines. @returns {Made[]} */
export function linesOf(/** @type {Set<string>} */ served) {
  /** @type {Groups} */ const into = { bases: new Set(), bands: [], averages: [], percentiles: [], floors: [], rarity: [] }, taken = new Set();
  for (const id of served) for (const rule of RULES) {
    const m = rule.pattern.exec(id);
    if (!m) continue;
    // (A name this pattern takes that none of its parts are in: the catalog wouldn't have brought it.)
    if (!rule.parts.some((part) => id.includes(part))) throw new Error(`${id}: a rule takes it, but its name parts (${rule.parts.join(", ")}) don't ask the catalog for it`);
    if (rule.take(m, served, into)) { taken.add(rule); break; }
  }
  // (A rule that took nothing: its parts don't bring its names, or the server stopped serving them. Either way, lines are missing.)
  for (const rule of RULES) if (!taken.has(rule)) throw new Error(`no line taken by the rule for ${rule.pattern}: the catalog's names with ${rule.parts.join(" or ")} have none it takes`);
  return [...[...into.bases].map(alone), ...into.bands, ...into.averages, ...Object.keys(WORKED).map(alone), ...into.percentiles, ...into.floors, ...into.rarity];
}

// ── The bottoms ──

/** @typedef {import("./score.js").Bottom} Bottom */
/** The bottoms the lines are ranked on: those of the judged period. @returns {Bottom[]} */
export const lowsOf = (/** @type {Float64Array} */ close, /** @type {Float64Array} */ epoch) => bottoms(close, epoch).filter((l) => l.low >= FROM);
/** @typedef {{ day: number, lows: Bottom[], ranked: Ranked[], ceilings: Made[] }} Board a ranking as the page shows it: the last day it was made from, its bottoms, its lines in order, and the lines left out as ceilings */
/**
 * The days each line is read on for the ranking, a span per bottom: its fall from its high through the evening before
 * its low, all `miss` reads (the open fall's days since its low so far don't move its miss). Where an earlier
 * ranking (`known`, the stored one) says the price met the line on a day, only through the evening before that day:
 * all `miss` needs to find the same day again. If it no longer finds it there, the line comes back unscored at that
 * bottom (`stale`) and the rest of its fall is read.
 * @returns {{ id: string, from: number, to: number }[]}
 */
export function spans(/** @type {Bottom[]} */ lows, /** @type {string[]} */ ids, /** @type {Board | null} */ known) {
  const at = new Map(known?.ranked.map((r) => [r.row.id, r.row.at]));
  return lows.flatMap(({ peak, low }) => {
    const k = known?.lows.findIndex((l) => l.peak === peak) ?? -1;
    return ids.map((id) => { const met = k >= 0 ? at.get(id)?.[k]?.met ?? -1 : -1; return { id, from: peak, to: (met > peak && met <= low ? met : low) - 1 }; });
  });
}
/** The spans read short of their bottom's eve (`spans`) whose line `ranked` left unscored there: the rest of the fall, to read. */
export function stale(/** @type {Bottom[]} */ lows, /** @type {ReturnType<typeof spans>} */ read, /** @type {Ranked[]} */ ranked) {
  const rows = new Map(ranked.map((r) => [r.row.id, r.row]));
  return read.flatMap(({ id, from, to }) => {
    const k = lows.findIndex((l) => l.peak === from), row = rows.get(id);
    return row && to < lows[k].low - 1 && !row.at[k] ? [{ id, from: to + 1, to: lows[k].low - 1 }] : [];
  });
}

// ── The ranking ──

/** @typedef {Made & { at: (import("./score.js").At | null)[] }} Row (`at`: one per bottom) */
/** @typedef {{ row: Row, rank: number, worst: number }} Ranked (`rank`: shared by lines equal at every bottom; `worst`: its largest miss, Infinity with no value at a bottom) */
/**
 * Every line held to each bottom, in the ranking's order: those with a value at each fall's high (a cohort born since,
 * as coins over 15 years old, can't be held to a bottom before it). @returns {Ranked[]}
 */
export function ranking(/** @type {Float64Array} */ close, /** @type {Bottom[]} */ lows, /** @type {Line[]} */ all) {
  return rank(all.filter(({ line }) => lows.every(({ peak }) => line[peak] === line[peak])).map(({ line, ...made }) => /** @type {Row} */ ({ ...made, at: lows.map((l) => miss(close, line, l)) })), lows.map((_, k) => k));
}
/**
 * A line the price closed under in two weeks of three since the judged period began is a ceiling, not a floor: it isn't
 * ranked. (Not half: lines cluster at 49–52%, which would move in and out from release to release; none lie between 53
 * and 84%.)
 */
const CEILING = 2 / 3;
/**
 * Whether a line is a ceiling, from its value each week from week `from` (the server's weeks, a week's value its last
 * day's) against the price's close on those days.
 */
export function ceiling(/** @type {Float64Array} */ close, /** @type {ArrayLike<number | null>} */ weekly, /** @type {number} */ from) {
  const last = close.length - 1;
  let weeks = 0, under = 0;
  for (let w = Math.max(0, weekOf(FROM) - from); w < weekly.length; w++) {
    const v = weekly[w], c = close[Math.min(last, (from + w) * WEEK.days + WEEK.day)];
    if (v != null && v === v && c > 0) weeks++, (under += +(c < v));
  }
  return under >= weeks * CEILING;
}

// ── The lines' names: every one from its definition above, so a name says exactly which line it is ──

const NAMES = /** @type {Record<string, string>} */ ({
  cvdd: "CVDD", balanced_price: "Balanced price", delta_price: "Delta price", burger_power_law_floor: "Burger's power-law floor",
  mooij_power_law_floor: "Mooij's power-law floor", sminston_power_law_floor: "Sminston With's power-law floor", rainbow_lowest_band: "Rainbow chart's lowest band",
  transferred_price: "Transferred price", active_price: "Active price", vaulted_price: "Vaulted price", cointime_price: "Cointime price", true_market_mean: "True market mean",
  rarity_meter: "Rarity meter", local_rarity_meter: "Local rarity meter", cycle_rarity_meter: "Cycle rarity meter",
});
const UNIT = /** @type {Record<string, string>} */ ({ h: "hour", d: "day", w: "week", m: "month", y: "year" });
/** A base's name, from its id's parts; a Bedrock floor's with its level, as Bedrock names its floors. */
function baseName(/** @type {string} */ id, /** @type {number | undefined} */ level) {
  if (NAMES[id]) return NAMES[id];
  let m = /^price_(sma|ema|min)_(\d+)([a-z])$/.exec(id);
  if (m) return `${m[2]}-${UNIT[m[3]]} ${{ sma: "average", ema: "exponential average", min: "low" }[m[1]]}`;
  if ((m = /^age_(\d+)([a-z])_to_(\d+)([a-z])_realized_price$/.exec(id))) return `${m[1]}-${UNIT[m[2]]} to ${m[3]}-${UNIT[m[4]]} coins' realized price`;
  if ((m = /^bedrock_(\w+)_floor$/.exec(id))) return `Bedrock ${m[1]} ${level}% floor`;
  // (A cost basis by coin is a realized price, by dollar a capitalized one: the median by coin is the median realized price.)
  if ((m = /^(lth_)?(awake_|mobile_)?(?:(median_)?(realized|capitalized)_price|cost_basis_by_(coin|dollar)_p(\d+))$/.exec(id))) {
    const said = `${m[1] ? "long-term holders' " : ""}${m[2]?.replace("_", " ") ?? ""}${m[3] ? "median " : m[6] ? `${m[6]}th percentile ` : ""}${m[4] ?? (m[5] === "coin" ? "realized" : "capitalized")} price`;
    return said[0].toUpperCase() + said.slice(1);
  }
  throw new Error(`a line with no name: ${id}`);
}
/** A line's name, from its definition: its base's, then for a rarity line its level, as the share of days it is rarer than (the meters are rarity lines by their own name). */
export function fullName(/** @type {Made} */ m) {
  const base = baseName(m.base, m.level);
  return m.kind === "rarity" ? `${base}, ${+(100 - /** @type {number} */ (m.level)).toFixed(1)}% ${m.base.endsWith("rarity_meter") ? "" : "rarity "}line` : base;
}

