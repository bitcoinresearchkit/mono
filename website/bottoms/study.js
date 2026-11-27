// @ts-check
// The Bottoms study, computed from the server's daily series: every line, held to the ranking's rules (score.js) at
// the cycle bottoms since 2017. No DOM: the page's worker runs it.

import { bottoms, miss, rank } from "./score.js";
import { POWER_LAW } from "/modules/power-law/index.js";

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
const GENESIS = dayOf("2009-01-03");

// ── The lines: each one a price by day, the set read from the server's catalog, so a line it starts serving joins ──

/** @typedef {"line" | "floor" | "rarity"} Kind how a line is made from its base: the base itself, one of Bedrock's floors, or one of the server's rarity levels */
/** @typedef {{ id: string, base: string, kind: Kind, level?: number }} Made */
/** @typedef {Made & { line: Float64Array }} Line */

/** A level as the server's ids write it, `p99_5`, read back: 99.5. */
const levelOf = (/** @type {string} */ tag) => +tag.replace("_", ".");
/** The rarity levels ranked: the cheap tail, where a bottom would sit; from 10% up, a level is a price seen one day in ten. */
const LOW = 5;
/** The days in a moving average's window, as the server's ids write it (`200w`, `4y`). */
const SPAN = /** @type {Record<string, number>} */ ({ d: 1, w: 7, m: 30, y: 365 });
/**
 * The lines worked out here from the date alone, by id, each over `n` days: Burger's support line, until the server
 * serves the power laws.
 */
export const WORKED = /** @type {Record<string, (n: number) => Float64Array>} */ ({
  power_law_floor: (n) => {
    const { intercept, slope, support } = POWER_LAW, floor = new Float64Array(n).fill(NaN);
    for (let d = GENESIS + 1; d < n; d++) floor[d] = 10 ** (intercept - support + slope * Math.log10(d - GENESIS));
    return floor;
  },
});
const alone = (/** @type {string} */ id) => /** @type {Made} */ ({ id, base: id, kind: "line" });
/** A line at one of its base's levels, the level as the server's ids write it. @returns {Made} */
const leveled = (/** @type {string} */ id, /** @type {string} */ base, /** @type {Kind} */ kind, /** @type {string} */ tag) => ({ id, base, kind, level: levelOf(tag) });
/** The cost-basis percentiles ranked: those under the median, where a floor would sit (the medians are ranked as the median realized and capitalized prices). */
const MEDIAN = 50;
/** @typedef {{ bases: Set<string>, averages: Made[], percentiles: Made[], floors: Made[], rarity: Made[] }} Groups */
/**
 * The rules that make the lines from the names the server serves (its catalog), tried in order on each name; only
 * what they leave out is out. Each rule's `parts` are what every name its pattern takes contains: the catalog is
 * asked for those names alone (`NAMED`), so a rule and the names it reads are written together. `take` files a
 * matched name and says whether it was taken (a name not taken goes on to the next rule).
 * @type {{ parts: string[], pattern: RegExp, take: (m: RegExpExecArray, served: Set<string>, into: Groups) => boolean }[]}
 */
const RULES = [
  // A price the server ranks the price's ratio to (it serves `B_ratio_rarity_…`) is a reference price: ranked as it is
  // and at each low rarity level. Its age slices (`age_…`) are cuts of the holders, not models; the short- and
  // long-term holders are the published split.
  { parts: ["rarity"], pattern: /^(\w+?)_rarity_p(\d+(?:_\d+)?)$/, take: ([id, base, tag], served, into) => {
    if (base.startsWith("age_") || !served.has(`${base}_ratio_rarity_p${tag}`)) return false;
    into.bases.add(base);
    if (levelOf(tag) <= LOW) into.rarity.push(leveled(id, base, "rarity", tag));
    return true;
  } },
  // The rarity meters, at their low levels.
  { parts: ["rarity"], pattern: /^(\w*rarity_meter)_p(\d+(?:_\d+)?)$/, take: ([id, base, tag], _, into) => levelOf(tag) <= LOW && (into.rarity.push(leveled(id, base, "rarity", tag)), true) },
  // The cost basis's percentiles under the median (`…cost_basis_by_coin_p5`: the price under which 5% of the coins last
  // moved; `by_dollar`: 5% of their dollars), of all holders and the published split, unweighted or awake or mobile.
  { parts: ["cost_basis_by_"], pattern: /^(?:sth_|lth_)?(?:awake_|mobile_)?cost_basis_by_(?:coin|dollar)_p(\d+)$/, take: ([id, tag], _, into) => levelOf(tag) < MEDIAN && (into.percentiles.push(alone(id)), true) },
  // Every Bedrock floor (`bedrock_W_floor_…`, W its weighting; the cost-basis lines above a floor are not floors).
  { parts: ["_floor_p"], pattern: /^(bedrock_[a-z]+_floor)_p(\d+(?:_\d+)?)$/, take: ([id, base, tag], _, into) => (into.floors.push(leveled(id, base, "floor", tag)), true) },
  // The price's moving averages of a year or more (shorter ones are trading averages, which a fall cuts through).
  { parts: ["price_sma_", "price_ema_"], pattern: /^price_(?:sma|ema)_(\d+)([dwmy])$/, take: ([id, n, unit], _, into) => +n * SPAN[unit] >= SPAN.y && (into.averages.push(alone(id)), true) },
];
/** The name parts the rules read: the catalog is asked for the names that contain one. */
export const NAMED = [...new Set(RULES.flatMap((rule) => rule.parts))];
/** Every line, from the names the server serves, in groups: the lines themselves, the lines worked out here, the percentiles, Bedrock's floors, then the rarity lines. @returns {Made[]} */
export function linesOf(/** @type {Set<string>} */ served) {
  /** @type {Groups} */ const into = { bases: new Set(), averages: [], percentiles: [], floors: [], rarity: [] }, taken = new Set();
  for (const id of served) for (const rule of RULES) {
    const m = rule.pattern.exec(id);
    if (!m) continue;
    // (A name this pattern takes that none of its parts are in: the catalog wouldn't have brought it.)
    if (!rule.parts.some((part) => id.includes(part))) throw new Error(`${id}: a rule takes it, but its name parts (${rule.parts.join(", ")}) don't ask the catalog for it`);
    if (rule.take(m, served, into)) { taken.add(rule); break; }
  }
  // (A rule that took nothing: its parts don't bring its names, or the server stopped serving them. Either way, lines are missing.)
  for (const rule of RULES) if (!taken.has(rule)) throw new Error(`no line taken by the rule for ${rule.pattern}: the catalog's names with ${rule.parts.join(" or ")} have none it takes`);
  return [...[...into.bases].map(alone), ...into.averages, ...Object.keys(WORKED).map(alone), ...into.percentiles, ...into.floors, ...into.rarity];
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
/** Every line held to each bottom, in the ranking's order. @returns {Ranked[]} */
export function ranking(/** @type {Float64Array} */ close, /** @type {Bottom[]} */ lows, /** @type {Line[]} */ all) {
  return rank(all.map(({ line, ...made }) => /** @type {Row} */ ({ ...made, at: lows.map((l) => miss(close, line, l)) })), lows.map((_, k) => k));
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
  cvdd: "CVDD", balanced_price: "Balanced price", delta_price: "Delta price", power_law_floor: "Power-law floor",
  active_price: "Active price", vaulted_price: "Vaulted price", cointime_price: "Cointime price", true_market_mean: "True market mean",
  rarity_meter: "Rarity meter", local_rarity_meter: "Local rarity meter", cycle_rarity_meter: "Cycle rarity meter",
});
const HOLDER = /** @type {Record<string, string>} */ ({ sth_: "short-term holders' ", lth_: "long-term holders' " });
/** A base's name, from its id's parts; a Bedrock floor's with its level, as Bedrock names its floors. */
function baseName(/** @type {string} */ id, /** @type {number | undefined} */ level) {
  if (NAMES[id]) return NAMES[id];
  let m = /^price_(sma|ema)_(\d+)(w|y)$/.exec(id);
  if (m) return `${m[2]}-${m[3] === "w" ? "week" : "year"} ${m[1] === "ema" ? "exponential average" : "average"}`;
  if ((m = /^bedrock_(\w+)_floor$/.exec(id))) return `Bedrock ${m[1]} ${level}% floor`;
  // (A cost basis by coin is a realized price, by dollar a capitalized one: the median by coin is the median realized price.)
  if ((m = /^(sth_|lth_)?(awake_|mobile_)?(?:(median_)?(realized|capitalized)_price|cost_basis_by_(coin|dollar)_p(\d+))$/.exec(id))) {
    const said = `${HOLDER[m[1]] ?? ""}${m[2]?.replace("_", " ") ?? ""}${m[3] ? "median " : m[6] ? `${m[6]}th percentile ` : ""}${m[4] ?? (m[5] === "coin" ? "realized" : "capitalized")} price`;
    return said[0].toUpperCase() + said.slice(1);
  }
  throw new Error(`a line with no name: ${id}`);
}
/** A line's name, from its definition: its base's, then for a rarity line its level, as the share of days it is rarer than (the meters are rarity lines by their own name). */
export function fullName(/** @type {Made} */ m) {
  const base = baseName(m.base, m.level);
  return m.kind === "rarity" ? `${base}, ${+(100 - /** @type {number} */ (m.level)).toFixed(1)}% ${m.base.endsWith("rarity_meter") ? "" : "rarity "}line` : base;
}

/** Where each line Bitview doesn't make comes from, as its source says (the study's formulas.md and origins.md): who published it and when. The year matters: a model of 2019 was made with the 2018 bottom known. */
const COINTIME = "ARK Invest & Glassnode, 2023", SPLIT = "after Glassnode, 2020";
const CREDIT = /** @type {Record<string, string>} */ ({
  cvdd: "Willy Woo, 2019", balanced_price: "David Puell, 2019", delta_price: "David Puell, 2019", power_law_floor: "Burger, 2019",
  realized_price: "Coin Metrics, 2018", sth_realized_price: SPLIT, lth_realized_price: SPLIT,
  active_price: "Dor Shahar, 2021", vaulted_price: COINTIME, cointime_price: COINTIME, true_market_mean: COINTIME,
});
/** The bases Bitview constructed (no earlier outside source found): every price weighted awake or mobile, and the capitalized prices (the cost basis by dollar); and those others publish too with no first author established. */
const OWN = /^(sth_|lth_)?((awake|mobile)_\w+|(median_)?capitalized_price|cost_basis_by_dollar_p\d+)$/, UNKNOWN = /^(sth_|lth_)?(median_realized_price|cost_basis_by_coin_p\d+)$/;
/** Who made a line: "Bitview" (its floors, its rarity lines, the bases it constructed), a published line's credit, or "" where no first author could be established. */
export function maker(/** @type {Made} */ m) {
  if (m.kind !== "line" || OWN.test(m.id)) return "Bitview";
  if (CREDIT[m.id]) return CREDIT[m.id];
  if (m.id.startsWith("price_")) return "moving average";
  if (UNKNOWN.test(m.id)) return "";
  throw new Error(`a line with no maker: ${m.id}`);
}
