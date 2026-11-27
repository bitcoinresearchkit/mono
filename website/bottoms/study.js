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

// ── The lines: each one a price by day, served as it is or worked out from a published formula ──

/** @typedef {"line" | "floor" | "rarity"} Kind how a line is made from its base: the base itself, one of Bedrock's floors, or one of the server's rarity levels */
/** @typedef {{ id: string, base: string, kind: Kind, level?: number }} Made */
/** @typedef {Made & { line: Float64Array }} Line */

const HOLDERS = ["", "sth_", "lth_"], MEASURES = ["realized_price", "capitalized_price", "median_realized_price", "median_capitalized_price"];
const ALONE = [...HOLDERS.flatMap((h) => MEASURES.map((m) => h + m)), "active_price", "vaulted_price", "cointime_price", "true_market_mean", "awake_realized_price", "mobile_realized_price", "price_sma_200w", "price_ema_200w", ...["1y", "2y", "4y"].flatMap((w) => [`price_sma_${w}`, `price_ema_${w}`])];
/** A level as the server's ids write it: 99.5 is `p99_5`. */
const tag = (/** @type {number} */ level) => `p${String(level).replace(".", "_")}`;
/** Bedrock's floors: a base a weight, its lines the five floors. */
const FLOORS = ["awake", "mobile", "unweighted"].flatMap((m) => [95, 98, 99, 99.5, 99.9].map((level) => /** @type {Made} */ ({ id: `bedrock_${m}_floor_${tag(level)}`, base: `bedrock_${m}_floor`, kind: "floor", level })));
/** The server's own rarity lines: a base a line, its lines the low levels the server publishes. (The meters are rarity lines by their own name; every other base's carry the word.) */
const RARITY = [
  ...["rarity_meter", "local_rarity_meter", "cycle_rarity_meter"],
  ...HOLDERS.flatMap((h) => ["", "awake_", "mobile_"].flatMap((mode) => MEASURES.map((m) => `${h}${mode}${m}`))),
  "vaulted_price", "true_market_mean",
].flatMap((base) => [0.1, 0.5, 1, 2, 5].map((level) => /** @type {Made} */ ({ id: `${base}${base.endsWith("rarity_meter") ? "" : "_rarity"}_${tag(level)}`, base, kind: "rarity", level })));

/** The published lines the server works out from the chain (crates/bitview_plugin_cointime). */
const PUBLISHED = ["cvdd", "balanced_price", "delta_price"];
/** The one line worked out here, from the date alone. */
const FORMULAS = ["power_law_floor"];
const alone = (/** @type {string} */ id) => /** @type {Made} */ ({ id, base: id, kind: "line" });
/** Every line, in the study's order. */
export const LINES = [...ALONE.map(alone), ...FLOORS, ...[...PUBLISHED, "power_law_floor"].map(alone), ...RARITY];

/** The series read whole, from their first day: the price and the halving epochs, which the bottoms come from. */
export const WHOLE = [PRICE, EPOCH];
/** The served lines read only on the days they are measured on (see `windows`). */
export const WINDOWED = LINES.map((l) => l.id).filter((id) => !FORMULAS.includes(id) && !WHOLE.includes(id));

// ── The bottoms ──

/** @typedef {import("./score.js").Bottom} Bottom */
/** The bottoms the lines are ranked on: those of the judged period. @returns {Bottom[]} */
export const lowsOf = (/** @type {Float64Array} */ close, /** @type {Float64Array} */ epoch) => bottoms(close, epoch).filter((l) => l.low >= FROM);
/**
 * The days a line is held to, so the only ones a served line is read on: each bottom's fall from its high to its
 * low, through the last day for the one still open, and the last day itself.
 * @returns {[number, number][]} each a first and a last day
 */
export function windows(/** @type {Bottom[]} */ lows, /** @type {number} */ last) {
  const falls = lows.map((l) => /** @type {[number, number]} */ ([l.peak, l.open ? last : l.low]));
  return falls.some(([, to]) => to === last) ? falls : [...falls, [last, last]];
}

// ── The lines worked out here ──

/** @typedef {(id: string) => Float64Array} Get a series by day, NaN where it has none, as long as the price */

/** Every line worked out here, by its id. */
function formulas(/** @type {Get} */ get) {
  const n = get(PRICE).length, { intercept, slope, support } = POWER_LAW, floor = new Float64Array(n).fill(NaN);
  for (let d = GENESIS + 1; d < n; d++) floor[d] = 10 ** (intercept - support + slope * Math.log10(d - GENESIS));
  return { power_law_floor: floor };
}
/** Every line with its values: the served ones as `get` reads them, the others from their formulas. */
export function lines(/** @type {Get} */ get) {
  const made = formulas(get);
  return LINES.map((l) => /** @type {Line} */ ({ ...l, line: made[l.id] ?? get(l.id) }));
}

// ── The ranking ──

/** @typedef {Made & { at: (import("./score.js").At | null)[], today: number }} Row (`at`: one per bottom; `today`: the line's last value) */
/** @typedef {{ row: Row, rank: number, worst: number }} Ranked (`rank`: shared by lines equal at every bottom; `worst`: its largest miss, Infinity with no value at a bottom) */
/** Every line held to each bottom, in the ranking's order. @returns {Ranked[]} */
export function ranking(/** @type {Float64Array} */ close, /** @type {Bottom[]} */ lows, /** @type {Line[]} */ all) {
  const last = close.length - 1;
  return rank(all.map(({ line, ...made }) => /** @type {Row} */ ({ ...made, at: lows.map((l) => miss(close, line, l)), today: Number.isFinite(line[last]) ? line[last] : NaN })), lows.map((_, k) => k));
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
  if ((m = /^(sth_|lth_)?(awake_|mobile_)?(median_)?(realized|capitalized)_price$/.exec(id))) { const said = `${HOLDER[m[1]] ?? ""}${m[2]?.replace("_", " ") ?? ""}${m[3] ? "median " : ""}${m[4]} price`; return said[0].toUpperCase() + said.slice(1); }
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
/** The bases Bitview constructed (no earlier outside source found), and the one others publish too with no first author established. */
const OWN = /^(sth_|lth_)?(median_)?capitalized_price$|^(awake|mobile)_realized_price$/, UNKNOWN = /^(sth_|lth_)?median_realized_price$/;
/** Who made a line: "Bitview" (its floors, its rarity lines, the bases it constructed), a published line's credit, or "" where no first author could be established. */
export function maker(/** @type {Made} */ m) {
  if (m.kind !== "line" || OWN.test(m.id)) return "Bitview";
  if (CREDIT[m.id]) return CREDIT[m.id];
  if (m.id.startsWith("price_")) return "moving average";
  if (UNKNOWN.test(m.id)) return "";
  throw new Error(`a line with no maker: ${m.id}`);
}
