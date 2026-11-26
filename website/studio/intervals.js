// @ts-check
// Time as charts read it: intervals and their steps, the chunks a series loads in, ranges, spans and windows, each
// interval's cadence, the halvings.

/** @import { Vector } from "/studio/formulas.js" */

/** @type {[string, [string, string][]][]} */
const INTERVALS = [
  ["Minutes", [["minute10", "10m"], ["minute30", "30m"]]],
  ["Hours", [["hour1", "1h"], ["hour4", "4h"], ["hour12", "12h"]]],
  ["Days", [["day1", "1D"], ["day3", "3D"], ["week1", "1W"]]],
  ["Months", [["month1", "1M"], ["month3", "3M"], ["month6", "6M"]]],
  ["Years", [["year1", "1Y"], ["year10", "10Y"]]],
];
const INTERVAL_ORDER = INTERVALS.flatMap(([, items]) => items.map(([id]) => id));
const intervalFits = (/** @type {string | undefined} */ limit, /** @type {string} */ index) => !limit || INTERVAL_ORDER.indexOf(index) <= INTERVAL_ORDER.indexOf(limit);
const INTERVAL_LABEL = Object.fromEntries(INTERVALS.flatMap(([, items]) => items));
// Hundreds of thousands of points each: loaded in chunks, as far back as the view goes.
const FINE = new Set(["minute10", "minute30", "hour1", "hour4", "hour12"]);
const FINE_STEP = { minute10: 600, minute30: 1800, hour1: 3600, hour4: 14400, hour12: 43200 };
const CHUNK = 8000;
// The widest view (four points a pixel at most) and room each side to reach out from.
const FIRST_CHUNKS = 3;
// A draw redoes every held chunk, so reaching farther drops the far end.
const MOST_CHUNKS = 12;
// One budget across series and intervals, least recently used first.
const KEPT_CHUNKS = 150;
/** @type {[string, string, number][]} */
const RANGES = [["3m", "3M", 91], ["1y", "1Y", 365], ["4y", "4Y", 1461], ["8y", "8Y", 2922], ["max", "Max", 0]];
const rangeDays = (/** @type {string} */ id) => RANGES.find(([range]) => range === id)?.[2] ?? 0;
const DEFAULT_RANGE = "8y";
const SPAN_DAYS = { hour: 1 / 24, day: 1, week: 7, month: 365.25 / 12, year: 365.25, halving: (210_000 * 600) / 86400 };
const SPAN_LETTERS = { h: "hour", d: "day", w: "week", mo: "month", y: "year", halving: "halving" };
const INTERVAL_DAYS = { ...Object.fromEntries(Object.entries(FINE_STEP).map(([index, step]) => [index, step / 86400])), day1: 1, day3: 3, week1: 7, month1: SPAN_DAYS.month, month3: 3 * SPAN_DAYS.month, month6: 6 * SPAN_DAYS.month, year1: SPAN_DAYS.year, year10: 10 * SPAN_DAYS.year };
// Calendar-total differences need aligned whole-year samples.
const flowFits = (/** @type {number} */ days, /** @type {string} */ index) => {
  const periods = days / INTERVAL_DAYS[index], whole = Math.round(periods);
  return (index === "year1" || index === "year10") && whole > 0 && Math.abs(periods - whole) < 1e-9;
};
// One at least: a window shorter than the interval is the series as it is.
const windowPoints = (/** @type {number} */ days, /** @type {string} */ index) => Math.max(1, Math.round(days / INTERVAL_DAYS[index]));
const spanDays = (/** @type {string} */ span) => {
  const [, n, letter] = /** @type {RegExpExecArray} */ (/^(\d+)(mo|halving|[hdwy])$/.exec(span));
  return Number(n) * SPAN_DAYS[SPAN_LETTERS[/** @type {keyof typeof SPAN_LETTERS} */ (letter)]];
};
/** @type {[string, string, string?][]} */
const SPANS = [["12h", "half a day"], ["1d", "a day"], ["7d", "a week", "7-day average"], ["30d", "a month", "30-day average"], ["90d", "a quarter", "90-day average"], ["200d", "about six and a half months"], ["1y", "a year", "1-year average"], ["2y", "two years"], ["1halving", "a halving cycle"], ["4y", "four years"]];
const AVERAGES = Object.fromEntries(SPANS.flatMap(([span, , average]) => average ? [[span, average]] : []));
const outreaches = (/** @type {Vector} */ x, /** @type {number} */ points) => {
  const first = x.findIndex(Number.isFinite), last = x.findLastIndex(Number.isFinite);
  return first >= 0 && last - first < points;
};
const fitsFrom = (/** @type {number} */ days, /** @type {string} */ index) => /** @type {string} */ (INTERVAL_ORDER.slice(INTERVAL_ORDER.indexOf(index) + 1).find((other) => !FINE.has(other) || windowPoints(days, other) < CHUNK));
const CADENCE = { minute10: "Every 10 minutes", minute30: "Every 30 minutes", hour1: "Hourly", hour4: "Every 4 hours", hour12: "Every 12 hours", day1: "Daily", day3: "Every 3 days", week1: "Weekly", month1: "Monthly", month3: "Quarterly", month6: "Every 6 months", year1: "Yearly", year10: "By decade" };
// The next halving is an estimate (a block every ten minutes from the tip's), drawn as one.
const KNOWN_HALVINGS = [1354116278, 1468082773, 1589225023, 1713571767], HALVING_BLOCKS = 210_000;
export {
  AVERAGES, CADENCE, CHUNK, DEFAULT_RANGE, FINE, FINE_STEP, FIRST_CHUNKS, fitsFrom, flowFits, HALVING_BLOCKS,
  INTERVAL_DAYS, INTERVAL_LABEL, INTERVAL_ORDER, intervalFits, INTERVALS, KEPT_CHUNKS, KNOWN_HALVINGS, MOST_CHUNKS,
  outreaches, rangeDays, RANGES, SPAN_DAYS, SPAN_LETTERS, spanDays, SPANS, windowPoints,
};
