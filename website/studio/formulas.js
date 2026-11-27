// @ts-check
import { menus } from "/studio/later.js";
// What a formula is: its words, parsed, checked and computed over aligned layers.
import { bandNames, inBands } from "/studio/bands.js";
import { chart, engine, EXAMPLE, flowsAt, indexAt, timeAt, tree, windowTooLong } from "/studio/app.js";
import { isSeriesName } from "/studio/catalog-walk.js";
import { escapeHtml } from "/studio/dom.js";
import { grouped } from "/studio/format.js";
import { failing } from "/studio/vocabulary.js";
import {
  CHUNK, FINE, fitsFrom, flowFits, INTERVAL_DAYS, INTERVAL_LABEL, INTERVAL_ORDER, outreaches, SPAN_DAYS, SPAN_LETTERS,
  windowPoints,
} from "/studio/intervals.js";
import { idOf, lastUsed, paneBox } from "/studio/chart.js";
/** @import { Layer } from "/studio/vocabulary.js" */

// A small, safe expression language over aligned layers: no eval, vectors in, vectors out.

/** @typedef {Float64Array} Vector */
/**
 * A formula's parts: numbers (a date written 2020-05-11, its time), layers by letter (`ref`), series by name, each
 * point's date (`date`, its time), and what combines them.
 * @typedef {{ k: "num", v: number, unit?: string, text?: string, date?: true } | { k: "ref", v: string } | { k: "series", v: string } | { k: "date" } | { k: "year" } | { k: "neg", a: Node } | { k: "bin", op: string, a: Node, b: Node } | { k: "call", f: string, args: Node[] }} Node
 */

/** Functions whose window takes two points at least. */
const TWO_POINTS = new Set(["stdev", "zscore", "volatility", "chop", "rsi", "stochrsi", "stochrsi_k", "stochrsi_d", "macd", "macd_signal", "macd_hist"]);
/** The language's own values, each point's: its date and its year (year(…) a function besides). */
const VALUES = new Set(["date", "year"]);

/** The comparisons, each 1 where true, 0 where false. */
const COMPARES = new Set(["<", ">", "<=", ">=", "==", "!="]);
const KEEPS_UNIT = new Set(["sma", "ema", "sum", "stdev", "median", "percentile", "highest", "lowest", "shift", "diff", "flow", "perday", "abs", "macd", "macd_signal", "macd_hist", "when"]);
/** Functions whose result is a percent, whatever their input's unit. */
const IN_PERCENT = new Set(["change", "cagr", "drawdown", "volatility"]);
/** Functions reading more than their arguments (halving cycles' epochs) or how they're written (rebase of a shift): computed each time. */
const UNSHARED = new Set(["halving", "shift", "rebase"]);

/** @typedef {{ mean: Vector, variance: Vector, count: Vector, total: Vector }} WindowStats */
/** Each vector's window stats, by window: made once, whoever asks (sma, sum, zscore and stdev of the same input). @type {WeakMap<Vector, Map<string, WindowStats>>} */
const statsOf = new WeakMap();
/** Each vector's start, where its first value that isn't 0 is, with what follows it (none: it starts there, or holds no such value). @type {WeakMap<Vector, { start: number, tail: Vector } | null>} */
const starts = new WeakMap();
/**
 * `run` over a series from where it exists: the API gives 0 before one does, no data, so a window holds none of it (a
 * mean, a lowest, an EMA's seed), and its output is none there. The tail is made once a vector, so what `run` keeps by
 * its input (rolling's stats) is shared by every call. The trade: a count truly 0 until its event begins (P2SH inputs
 * before 2012) reads as starting there too, so its first window-1 points average a filling window, not the true 0s.
 * @param {Vector} x @param {(x: Vector) => Vector} run @returns {Vector}
 */
function fromStart(x, run) {
  if (!starts.has(x)) {
    const start = x.findIndex((value) => value !== 0 && Number.isFinite(value));
    starts.set(x, start > 0 ? { start, tail: x.subarray(start) } : null);
  }
  const from = starts.get(x);
  if (!from) return run(x);
  const out = new Float64Array(x.length).fill(NaN);
  out.set(run(from.tail), from.start);
  return out;
}
/**
 * Each window of the last `n` points (a span of time: the values in it, points without one left out): its mean,
 * variance and count, at the points with a value; from the first value on where the data is all of history
 * (`partial`: a window holds what there is so far), else from `n` points after it (a full window). The sums run over
 * differences from an origin in the window, made again from scratch every `n` points and once the window's spread
 * falls far below what passed through it: a small spread on a large level (a hash rate, a difficulty between
 * adjustments) keeps its digits.
 * @param {Vector} x @param {number} n @returns {WindowStats}
 */
function rolling(x, n, partial = false) {
  const byWindow = statsOf.get(x) ?? new Map(), key = `${n} ${partial}`, kept = byWindow.get(key);
  if (kept) {
    // (Used last, last out: the few a chart's formulas share stay, those typed through go.)
    byWindow.delete(key);
    byWindow.set(key, kept);
    return kept;
  }
  const mean = new Float64Array(x.length).fill(NaN), variance = new Float64Array(x.length).fill(NaN), counts = new Float64Array(x.length).fill(NaN), totals = new Float64Array(x.length).fill(NaN);
  let origin = NaN, sum = 0, squares = 0, count = 0, peak = 0, first = -1;
  // (Finite as `v - v === 0`, NaN as `origin !== origin`: no call a value, which a phone without a JIT (Lockdown
  // Mode) pays for: a fifth of the time this takes there, its results the same to the bit.)
  const add = (/** @type {number} */ value, /** @type {number} */ sign) => {
    if (origin !== origin) origin = value;
    const d = value - origin;
    sum += sign * d;
    squares += sign * d * d;
    count += sign;
    if (d * d > peak) peak = d * d;
  };
  for (let i = 0; i < x.length; i++) {
    const value = x[i];
    if (value - value === 0) {
      add(value, 1);
      if (first < 0) first = i;
    }
    if (i >= n) {
      const old = x[i - n];
      if (old - old === 0) add(old, -1);
    }
    if ((i + 1) % n === 0 || (count && peak > 1e6 * (squares - (sum * sum) / count))) {
      origin = NaN;
      sum = squares = count = peak = 0;
      for (let j = i - n + 1 > 0 ? i - n + 1 : 0; j <= i; j++) {
        const again = x[j];
        if (again - again === 0) add(again, 1);
      }
    }
    if (count && (partial || i - first + 1 >= n) && value - value === 0) {
      const shift = sum / count, spread = squares / count - shift * shift;
      mean[i] = origin + shift;
      variance[i] = spread < 0 ? 0 : spread;
      counts[i] = count;
      totals[i] = origin * count + sum;
    }
  }
  const stats = { mean, variance, count: counts, total: totals };
  statsOf.set(x, byWindow.set(key, stats));
  if (byWindow.size > 12) byWindow.delete(/** @type {string} */ (byWindow.keys().next().value));
  return stats;
}
/**
 * Each window's `p`th percentile (0 to 100, between the two nearest values), from the first value on (`partial`)
 * or `n` points after it, as `rolling`: the window's values kept in order by a tree of counts over their ranks.
 * @param {Vector} x @param {number} n @param {number} p
 */
/** A series' values in order, each once, and each one's rank: what a rolling percentile counts by, made once a series (a band's two ask the same). */
const rankings = new WeakMap();
function rankingOf(/** @type {any} */ x) {
  let ranking = rankings.get(x);
  if (!ranking) {
    const sorted = x.filter(Number.isFinite).sort();
    const values = sorted.filter((/** @type {number} */ value, /** @type {number} */ i) => i === 0 || value !== sorted[i - 1]);
    rankings.set(x, (ranking = { values, rankOf: new Map(Array.from(values, (value, i) => [value, i + 1])) }));
  }
  return /** @type {{ values: ArrayLike<number>, rankOf: Map<number, number> }} */ (ranking);
}
function rollingPercentile(x, n, p, partial = false) {
  const out = new Float64Array(x.length).fill(NaN);
  const { values, rankOf } = rankingOf(x);
  const tree = new Int32Array(values.length + 1);
  const add = (/** @type {number} */ rank, /** @type {number} */ by) => {
    for (; rank < tree.length; rank += rank & -rank) tree[rank] += by;
  };
  let top = 1;
  while (top * 2 < tree.length) top *= 2;
  /** The `k`th smallest value in the window. */
  const nth = (/** @type {number} */ k) => {
    let at = 0;
    for (let step = top; step; step >>= 1) if (at + step < tree.length && tree[at + step] < k) (at += step), (k -= tree[at]);
    return values[at];
  };
  let count = 0, first = -1;
  for (let i = 0; i < x.length; i++) {
    if (Number.isFinite(x[i])) {
      add(/** @type {number} */ (rankOf.get(x[i])), 1), count++;
      if (first < 0) first = i;
    }
    if (i >= n && Number.isFinite(x[i - n])) add(/** @type {number} */ (rankOf.get(x[i - n])), -1), count--;
    if (count && (partial || i - first + 1 >= n) && Number.isFinite(x[i])) {
      const at = ((count - 1) * p) / 100, below = Math.floor(at), t = at - below, a = nth(below + 1);
      out[i] = t ? a * (1 - t) + nth(Math.min(count, below + 2)) * t : a;
    }
  }
  return out;
}
/** Rolling extreme over the last `n` points in one pass (from where `rolling` starts): a queue of candidates, each beating those after it. */
function extreme(x, n, beats, partial = false) {
  const out = new Float64Array(x.length).fill(NaN);
  const queue = new Int32Array(x.length);
  let head = 0, tail = 0, first = -1;
  for (let i = 0; i < x.length; i++) {
    if (Number.isFinite(x[i])) {
      while (tail > head && !beats(x[queue[tail - 1]], x[i])) tail--;
      queue[tail++] = i;
      if (first < 0) first = i;
    }
    while (tail > head && queue[head] <= i - n) head++;
    if (Number.isFinite(x[i]) && (partial || i - first + 1 >= n)) out[i] = x[queue[head]];
  }
  return out;
}
/** An exponential moving average over `n` points, decaying by points (one without a value counting as time passed). @param {Vector} x */
function emaOf(x, n, partial = false) {
  const out = new Float64Array(x.length).fill(NaN), alpha = 2 / (n + 1);
  let value = NaN, first = -1, last = -1;
  for (let i = 0; i < x.length; i++) {
    if (!Number.isFinite(x[i])) continue;
    if (first < 0) (first = i), (value = x[i]);
    else value += (i - last === 1 ? alpha : 1 - (1 - alpha) ** (i - last)) * (x[i] - value);
    last = i;
    if (partial || i - first + 1 >= n) out[i] = value;
  }
  return out;
}
/** A rolling standard deviation (two values at least). @param {Vector} x */
const deviationOf = (x, n, partial = false) => ((stats) => stats.variance.map((value, i) => (stats.count[i] >= 2 ? Math.sqrt(value) : NaN)))(rolling(x, n, partial));
/** Wilder's RSI over `n` points: its average gain against its average loss, each smoothed by 1/n. @param {Vector} x */
function rsiOf(x, n) {
  const out = new Float64Array(x.length).fill(NaN);
  let gain = 0, loss = 0, prev = NaN, seen = 0;
  for (let i = 0; i < x.length; i++) {
    if (!Number.isFinite(x[i])) continue;
    if (Number.isFinite(prev)) {
      const d = x[i] - prev, up = Math.max(d, 0), down = Math.max(-d, 0);
      seen++;
      if (seen <= n) (gain += up / n), (loss += down / n);
      else (gain = (gain * (n - 1) + up) / n), (loss = (loss * (n - 1) + down) / n);
      if (seen >= n) out[i] = loss ? 100 - 100 / (1 + gain / loss) : gain ? 100 : 50;
    }
    prev = x[i];
  }
  return out;
}
/** Raw stochastic RSI, shared by its raw, %K and %D formula functions. @param {Vector} x */
function stochrsiOf(x, n) {
  const r = rsiOf(x, n), lo = extreme(r, n, (a, b) => a < b), hi = extreme(r, n, (a, b) => a > b);
  // (Its RSI flat over the window: where it was.)
  let was = NaN;
  return r.map((value, i) => (Number.isFinite(value) ? (was = hi[i] > lo[i] ? ((value - lo[i]) / (hi[i] - lo[i])) * 100 : was) : NaN));
}
/** Three-point smoothing for each stochastic line, independent of the chart's interval. @param {Vector} x */
const stochrsiKOf = (x, n) => rolling(stochrsiOf(x, n), 3).mean;
/** MACD's line: an EMA over 12/26 of `n` points less one over `n`. @param {Vector} x */
const macdOf = (x, n, partial = false) => ((fast, slow) => fast.map((value, i) => value - slow[i]))(emaOf(x, Math.max(1, Math.round((n * 12) / 26)), partial), emaOf(x, n, partial));
/** Annualized volatility: the deviation of log returns over `n` points, a year's worth of points' (`step`: a point's days), in %. @param {Vector} x */
function volatilityOf(x, n, partial, step) {
  // Each log return with the points it spans (a gap's, more than one): over a window, its drift and its variance
  // each per point, as time measures them (a return across a gap counts as long as it is, its drift its own share):
  // variance = (Σ r²/k − (Σ r)² / Σ k) / (returns − 1). A steady growth reads as none, gaps or not.
  const len = x.length, r = new Float64Array(len).fill(NaN), k = new Float64Array(len);
  let prev = NaN, at = -1, first = -1;
  for (let i = 0; i < len; i++) {
    if (!Number.isFinite(x[i])) continue;
    if (first < 0) first = i;
    if (prev > 0 && x[i] > 0) (r[i] = Math.log(x[i] / prev)), (k[i] = i - at);
    prev = x[i];
    at = i;
  }
  const sums = (/** @type {(i: number) => number} */ of) => {
    const out = new Float64Array(len + 1);
    for (let i = 0; i < len; i++) out[i + 1] = out[i] + (Number.isFinite(r[i]) ? of(i) : 0);
    return out;
  };
  const sr = sums((i) => r[i]), sk = sums((i) => k[i]), sq = sums((i) => (r[i] * r[i]) / k[i]), sc = sums(() => 1);
  const scale = Math.sqrt(365 / step) * 100, out = new Float64Array(len).fill(NaN);
  for (let i = 0; i < len; i++) {
    // (Only where its own return is: none from a price of 0, or the first.)
    if (!Number.isFinite(r[i]) || !(partial || i - first + 1 >= n)) continue;
    const lo = Math.max(0, i - n + 1), count = sc[i + 1] - sc[lo];
    if (count < 2) continue;
    const total = sr[i + 1] - sr[lo], span = sk[i + 1] - sk[lo];
    out[i] = Math.sqrt(Math.max(0, (sq[i + 1] - sq[lo] - (total * total) / span) / (count - 1))) * scale;
  }
  return out;
}
/**
 * The choppiness index over `n` moves, of one series: their sum against the range of the prices they span (one more:
 * the first's start), log-scaled to 0–100 (a straight run, 0). @param {Vector} x
 */
function chopOf(x, n) {
  const moves = new Float64Array(x.length).fill(NaN);
  // (A move from the point just before only: one across a gap would start outside the prices it's measured against.)
  for (let i = 1; i < x.length; i++) if (Number.isFinite(x[i]) && Number.isFinite(x[i - 1])) moves[i] = Math.abs(x[i] - x[i - 1]);
  // (Only where all `n` are: fewer would sum to less than the range, below 0.)
  const { total, count } = rolling(moves, n), hi = extreme(x, n + 1, (a, b) => a > b), lo = extreme(x, n + 1, (a, b) => a < b), log = Math.log10(n);
  return x.map((_, i) => (count[i] === n && hi[i] > lo[i] && total[i] > 0 && log > 0 ? (100 * Math.log10(total[i] / (hi[i] - lo[i]))) / log : NaN));
}
/** Growth a year, compounded, in %: over `n` points (`step`: a point's days), or (`n` Infinity) since its first value. @param {Vector} x */
function cagrOf(x, n, step) {
  const out = new Float64Array(x.length).fill(NaN), positive = (/** @type {number} */ value) => Number.isFinite(value) && value > 0;
  // (Over the real time between its two values: a span back, the latest at or before it; else its first.)
  let before = -1, first = -1;
  for (let i = 0; i < x.length; i++) {
    if (n !== Infinity && i >= n && positive(x[i - n])) before = i - n;
    if (first < 0 && positive(x[i])) first = i;
    const from = n === Infinity ? first : before;
    if (from >= 0 && from < i && positive(x[i])) out[i] = ((x[i] / x[from]) ** (365 / ((i - from) * step)) - 1) * 100;
  }
  return out;
}
/** `f(value, the one a span earlier)`: the latest value `n` points back or before (a point may have none). */
/**
 * x over period `n` (its points where `periods` is n: halving_epoch's cycles, each point's year) from 1× at its first
 * value (not 0: one before there's any, none), the whole period from that one (a gap in it, as a Feb 29 a year's shift
 * leaves, doesn't start it again); moved by as many points as there are between period `n`'s first and period
 * `at`'s, where `at` is given (its first past the data: all of it past it too), and then none where x has no value
 * at the period's first point (laid over another, its 1× would be a later one's: the price's first cycle, from Jul
 * 2010, over the second).
 * @param {Vector} x @param {Vector} periods @param {number} n @param {number | undefined} at
 */
/** A period's values left out for want of one at its start (see periodAlone): its layer says so. @type {WeakSet<Vector>} */
const unstarted = new WeakSet();
function periodAlone(x, periods, n, at) {
  const own = new Float64Array(x.length).fill(NaN);
  let base = NaN;
  // (Moved onto another, a period begun late none: its 1× would be a later one's; where it is, from its first.)
  const start = periods.indexOf(n), moved = at !== undefined && at !== n;
  if (moved && (start < 0 || !Number.isFinite(x[start]) || !x[start])) return (start >= 0 && unstarted.add(own), own);
  for (let i = Math.max(0, start); i < x.length; i++) {
    if (periods[i] !== n || !Number.isFinite(x[i])) continue;
    if (!base) base = x[i];
    own[i] = base ? x[i] / base : NaN;
  }
  if (at === undefined || at === n) return own;
  if (unstarted.has(own)) return ((out) => (unstarted.add(out), out))(new Float64Array(x.length).fill(NaN));
  const firstOf = (/** @type {number} */ period) => ((i) => (i < 0 ? Infinity : i))(periods.findIndex((p) => p >= period));
  const by = firstOf(at) - firstOf(n), out = new Float64Array(x.length).fill(NaN);
  if (!Number.isFinite(by)) return out;
  for (let i = Math.max(0, by); i < x.length && i - by < x.length; i++) out[i] = own[i - by];
  return out;
}
/** Each time's year (UTC; times in seconds, in order): a date made once a year. @param {Vector} times */
function yearsOf(times) {
  const out = new Float64Array(times.length);
  let year = NaN, next = -Infinity;
  for (let i = 0; i < times.length; i++) {
    if (times[i] >= next) (year = new Date(times[i] * 1000).getUTCFullYear()), (next = Date.UTC(year + 1, 0, 1) / 1000);
    out[i] = year;
  }
  return out;
}
/**
 * A shift's span as written (shift(A, -30d)): its count, signed, its unit (none written: days) and its text; null
 * for anything else (a series, a sum): a year's and a halving's own lengths vary, so it's read as written, not as a
 * window's days. @param {any} node @returns {{ v: number, unit: keyof typeof SPAN_DAYS, text: string } | null}
 */
const shiftSpan = (node) =>
  node?.k === "num" && !node.date ? { v: node.v, unit: node.unit ?? "day", text: node.text ?? String(node.v) } : node?.k === "neg" ? ((span) => span && { ...span, v: -span.v, text: `-${span.text}` })(shiftSpan(node.a)) : null;
/** What's wrong with a shift's span as written, or "". @param {ReturnType<typeof shiftSpan>} span */
const shiftSpanError = (span) => {
  if (!span || !span.v) return "shift(x, by): by is a span, as 30d, -2w, 1y or 1halving";
  return span.unit === "month" || span.unit === "year" || span.unit === "halving" ? (Number.isInteger(span.v) ? "" : `shift(x, by): ${span.text} isn't whole: months, years and halvings count whole ones (18mo, not 1.5y)`) : "";
};
/**
 * x moved by a span (shiftSpan's), later where it's positive: by points (hours, days, weeks: `step`, a point's days);
 * to the same date (months, years: one with none there, as Feb 29 a year on, dropped), placed at the point nearest
 * it, within half a step; or as far into the halving cycle `v` on as into its own (`epochs`, halving_epoch: day for
 * day, what runs past that cycle's end cut). Nothing past the data.
 * @param {Vector} x @param {{ v: number, unit: keyof typeof SPAN_DAYS }} span @param {Vector | null} times @param {Vector | null} epochs @param {number} step
 */
function shiftOf(x, span, times, epochs, step) {
  const out = new Float64Array(x.length).fill(NaN);
  if (epochs) {
    // (Each cycle's first point: the first at it or past it.)
    const starts = /** @type {number[]} */ ([]);
    for (let i = 0; i < epochs.length; i++) while (epochs[i] >= starts.length) starts.push(i);
    for (let i = 0; i < x.length; i++) {
      const own = epochs[i], to = own + span.v;
      if (!(to >= 0 && to < starts.length)) continue;
      const j = starts[to] + i - starts[own];
      if (j < (starts[to + 1] ?? x.length)) out[j] = x[i];
    }
  } else if (times) {
    const months = span.unit === "year" ? span.v * 12 : span.v, half = (step * 86400) / 2, at = new Date(0);
    // (Where each lands only grows, so its point is looked for from the last one's on.)
    for (let i = 0, j = 0; i < x.length; i++) {
      if (!Number.isFinite(x[i])) continue;
      at.setTime(times[i] * 1000);
      const date = at.getUTCDate();
      at.setUTCMonth(at.getUTCMonth() + months);
      if (at.getUTCDate() !== date) continue;
      const time = at.getTime() / 1000;
      while (j < times.length - 1 && times[j + 1] <= time) j++;
      const k = j + 1 < times.length && times[j + 1] - time < time - times[j] ? j + 1 : j;
      if (Math.abs(times[k] - time) <= half) out[k] = x[i];
    }
  } else {
    const n = Math.round((span.v * SPAN_DAYS[span.unit]) / step);
    for (let i = Math.max(0, n); i < Math.min(x.length, x.length + n); i++) out[i] = x[i - n];
  }
  return out;
}
/**
 * Where a formula's values start a stretch anew with no gap before it (a rebase's, by a change of its `by`), by the
 * values: the point before each marked, as a trend's breaks are (nothing drawn down to the new start). Kept through
 * what's worked out point by point (signs, arithmetic, comparisons, when, min and max, log and the like), not through
 * a window, a shift or a drawdown.
 * @type {WeakMap<Vector, Uint8Array>}
 */
const restarts = new WeakMap();
/**
 * A formula's restarts as its line's breaks: each on the last point it has before one (a when() may have taken the
 * marked one out), none where nothing follows. @param {Vector} values @param {Uint8Array} marks
 */
function breaksAt(values, marks) {
  let breaks = /** @type {Uint8Array | null} */ (null), last = -1, owed = false;
  for (let i = 0; i < values.length; i++) {
    if (Number.isFinite(values[i])) {
      if (owed) (breaks ??= new Uint8Array(values.length))[last] = 1;
      (owed = false), (last = i);
    }
    if (marks[i] && last >= 0) owed = true;
  }
  return breaks;
}
/** `out`, worked out point by point from `from`, with their restarts. @param {Vector} out @param {(Vector | number)[]} from */
function restarted(out, ...from) {
  let marks = /** @type {Uint8Array | null} */ (null);
  for (const vector of from) {
    const own = typeof vector === "number" ? undefined : restarts.get(vector);
    if (own) marks = marks ? marks.map((mark, i) => mark | own[i]) : own;
  }
  if (marks) restarts.set(out, marks);
  return out;
}
/**
 * Each stretch of `x` over its first value (from 1): a gap in `x` starts another, and with `by`, a change of by's
 * value (none of by's, no change); each after the first marked (see restarts: the chart's line drawn across a gap
 * too). Values of 0 first (a price before there was one), none: it starts at the next. With `by` and `moved` (x a
 * period shifted onto another), a period with no value at its first point, none: laid over another, its 1× would
 * be a later one's (the price's first cycle, from Jul 2010, over the second); where it is, it's drawn from its first.
 * @param {Vector} x @param {Vector | null} by
 */
function rebaseOf(x, by, moved = false) {
  const out = new Float64Array(x.length).fill(NaN);
  let base = NaN, was = NaN, last = -1, marks = /** @type {Uint8Array | null} */ (null), whole = true;
  for (let i = 0; i < x.length; i++) {
    // (By's changes followed through x's gaps too: one there starts the stretch after it, once.)
    const key = by ? by[i] : NaN, changed = key === key && was === was && key !== was;
    // (A period begun (by's first, or a change of it): whole only with a value at its start.)
    if (key === key && (changed || was !== was)) whole = Number.isFinite(x[i]) && x[i] !== 0;
    if (key === key) was = key;
    if (moved && by && !whole) continue;
    if (!Number.isFinite(x[i])) {
      base = NaN;
      continue;
    }
    if (!base || changed) {
      if (last >= 0) (marks ??= new Uint8Array(x.length))[last] = 1;
      base = x[i];
    }
    out[i] = base ? x[i] / base : NaN;
    if (base) last = i;
  }
  if (marks) restarts.set(out, marks);
  return out;
}
/**
 * A running total's change over each point's period, per day: by the days that period holds, so a day, a week or a
 * month each give the same rate (a total sampled at each period's end); the point still open (its period reaches past
 * now) its change so far over the time elapsed. The first value's change is from nothing where the data is all of
 * history (`partial`), else none: a chunk starts from a total it doesn't hold.
 */
function perDayOf(x, times, index, partial) {
  const out = new Float64Array(x.length).fill(NaN), now = Date.now() / 1000;
  let before = -1;
  for (let i = 0; i < x.length; i++) {
    if (!Number.isFinite(x[i])) continue;
    // (A period ends where the next begins, the last where the client's next one does.)
    const end = i + 1 < times.length ? times[i + 1] : timeAt(index, indexAt(index, times[i]) + 1);
    if (before >= 0 || partial) {
      const days = (Math.min(end, now) - (before >= 0 ? times[before + 1] : times[i])) / 86400;
      if (days > 0) out[i] = (x[i] - (before >= 0 ? x[before] : 0)) / days;
    }
    before = i;
  }
  return out;
}
function shifted(x, n, f) {
  const out = new Float64Array(x.length).fill(NaN);
  let before = -1;
  for (let i = 0; i < x.length; i++) {
    if (i >= n && Number.isFinite(x[i - n])) before = i - n;
    if (before >= 0 && Number.isFinite(x[i])) out[i] = f(x[i], x[before]);
  }
  return out;
}
/**
 * name → [arity, signature, what it does, run, the span may be left out, its usual span (days)]; a window is a span
 * of time (30d, 4w, 6mo, 2y, 12h; a bare number is days), always the last argument, run as the interval's points
 * (`step`: the days a point is). Where the data is all of history (1D and up), a window at its start holds what there
 * is (`partial`), and one left out is all of it so far (or its usual span, where it has one, in points, as these
 * indicators count: RSI's 14); on a finer
 * interval, only part of history loads: a window waits to be full.
 * @type {Record<string, [number, string, string, ((x: Vector, n: number, p: number, partial: boolean, step: number) => Vector) | null, boolean?, number?]>}
 */
const FUNCTIONS = {
  sma: [2, "sma(x, span?)", "moving average (no span: of all so far)", (x, n, _p, partial) => rolling(x, n, partial).mean, true],
  ema: [2, "ema(x, span)", "exponential moving average", (x, n, _p, partial) => emaOf(x, n, partial)],
  sum: [2, "sum(x, span?)", "rolling sum (no span: of all so far)", (x, n, _p, partial) => {
    // All so far: a running total, exact.
    if (n === Infinity) {
      let total = 0;
      return x.map((value) => (Number.isFinite(value) ? (total += value) : NaN));
    }
    return rolling(x, n, partial).total;
  }, true],
  // A deviation takes two values at least.
  stdev: [2, "stdev(x, span?)", "rolling standard deviation (no span: of all so far)", (x, n, _p, partial) => deviationOf(x, n, partial), true],
  zscore: [2, "zscore(x, span?)", "deviations from the rolling mean (no span: of all so far)", (x, n, _p, partial) => {
    const { mean, variance, count } = rolling(x, n, partial);
    return x.map((value, i) => {
      const sd = Math.sqrt(variance[i]);
      return count[i] >= 2 ? (sd ? (value - mean[i]) / sd : 0) : NaN;
    });
  }, true],
  median: [2, "median(x, span?)", "rolling median (no span: of all so far)", (x, n, _p, partial) => rollingPercentile(x, n, 50, partial), true],
  percentile: [3, "percentile(x, p, span?)", "rolling p-th percentile (p from 0 to 100; the median at 50; no span: of all so far)", (x, n, p, partial) => rollingPercentile(x, n, p, partial), true],
  change: [2, "change(x, span)", "% change over the span", (x, n) => shifted(x, n, (a, b) => (a / b - 1) * 100)],
  diff: [2, "diff(x, span)", "difference over the span", (x, n) => shifted(x, n, (a, b) => a - b)],
  flow: [2, "flow(x, span)", "calendar-year change in a cumulative total; zero before the total's first value, missing samples after it stay missing", (x, n, _p, partial) => ((first) => x.map((value, i) => value - (i - n < first && partial ? 0 : x[i - n])))(x.findIndex(Number.isFinite))],
  perday: [1, "perday(x)", "a running total's change per day: over each point, by the days it spans (a week's or a month's own); the latest, still open, its change so far over the time elapsed", null],
  shift: [2, "shift(x, by)", "x moved later by a span, earlier by a negative one (1y to the same date a year on, 1halving as far into the next halving cycle as into its own)", null],
  cagr: [2, "cagr(x, span?)", "% growth a year, compounded, over the span (no span: since its first value)", (x, n, _p, _partial, step) => cagrOf(x, n, step), true],
  drawdown: [1, "drawdown(x)", "% below its highest so far", (x) => {
    let top = -Infinity;
    // (A share of its highest: none where that's not above zero.)
    return x.map((value) => (Number.isFinite(value) ? ((top = Math.max(top, value)), top > 0 ? (value / top - 1) * 100 : NaN) : NaN));
  }],
  rsi: [2, "rsi(x, span?)", "relative strength index, 0 to 100 (span: 14 points if left out)", (x, n) => rsiOf(x, n), true, 14],
  stochrsi: [2, "stochrsi(x, span?)", "stochastic RSI: the RSI's place between its own low and high, 0 to 100 (span: 14 points if left out)", (x, n) => stochrsiOf(x, n), true, 14],
  stochrsi_k: [2, "stochrsi_k(x, span?)", "stochastic RSI %K: its three-point average (span: 14 points if left out)", (x, n) => stochrsiKOf(x, n), true, 14],
  stochrsi_d: [2, "stochrsi_d(x, span?)", "stochastic RSI %D: %K's three-point average (span: 14 points if left out)", (x, n) => rolling(stochrsiKOf(x, n), 3).mean, true, 14],
  macd: [2, "macd(x, span?)", "MACD line: a fast EMA (12/26 of the span) less a slow one (span: 26 points if left out)", (x, n, _p, partial) => macdOf(x, n, partial), true, 26],
  macd_signal: [2, "macd_signal(x, span?)", "MACD's signal: its line's EMA over 9/26 of the span (span: 26 points if left out)", (x, n, _p, partial) => emaOf(macdOf(x, n, partial), Math.max(1, Math.round((n * 9) / 26)), partial), true, 26],
  macd_hist: [2, "macd_hist(x, span?)", "MACD's histogram: its line less its signal (span: 26 points if left out)", (x, n, _p, partial) => ((line) => ((signal) => line.map((value, i) => value - signal[i]))(emaOf(line, Math.max(1, Math.round((n * 9) / 26)), partial)))(macdOf(x, n, partial)), true, 26],
  volatility: [2, "volatility(x, span?)", "annualized volatility: the deviation of its log returns over the span, in % (span: 30 points if left out)", (x, n, _p, partial, step) => volatilityOf(x, n, partial, step), true, 30],
  chop: [2, "chop(x, span?)", "choppiness index, 0 to 100: high, moving sideways; low, trending (span: 14 points if left out)", (x, n) => chopOf(x, n), true, 14],
  highest: [2, "highest(x, span?)", "rolling maximum (no span: of all so far)", (x, n, _p, partial) => extreme(x, n, (a, b) => a > b, partial), true],
  lowest: [2, "lowest(x, span?)", "rolling minimum (no span: of all so far)", (x, n, _p, partial) => extreme(x, n, (a, b) => a < b, partial), true],
  log: [1, "log(x)", "logarithm, base 10", (x) => x.map(Math.log10)],
  ln: [1, "ln(x)", "natural logarithm", (x) => x.map(Math.log)],
  exp: [1, "exp(x)", "the exponential: e to the power x", (x) => x.map(Math.exp)],
  sqrt: [1, "sqrt(x)", "square root", (x) => x.map(Math.sqrt)],
  abs: [1, "abs(x)", "absolute value", (x) => x.map(Math.abs)],
  over: [1, "over(x)", "x, an amount or count of an age, amount, balance, profit or loss band, summed with the same series of every band above: over(age_1y_to_18m_supply) is the coins over 1 year old", null],
  under: [1, "under(x)", "the same series of every band below x's, summed: under(age_1y_to_18m_supply) is the coins under 1 year old, under(loss_0_to_10_percent_supply) those over 10% in loss", null],
  min: [2, "min(a, b)", "smaller of two", null],
  max: [2, "max(a, b)", "larger of two", null],
  when: [2, "when(x, condition)", "x where the condition holds (not 0), none elsewhere: when(A, date >= 2020-05-11)", null],
  halving: [3, "halving(x, n, at?)", "x over halving cycle n (0 the first, from the genesis block; 4 since 2024) from 1× at its start; with at, moved to start where cycle at starts (halving(A, 0, 2): the first cycle over the third's dates, day for day)", null, true],
  year: [3, "year(x, y, at?)", "x over year y (2021) from 1× at its start; with at, moved to start where year at starts (year(A, 2021, 2024): 2021 over 2024's dates, day for day)", null, true],
  rebase: [2, "rebase(x, by?)", "each stretch of x over its first value, from 1×: a gap starts another, and with by, so does a change of by (rebase(A, halving_epoch): each cycle from its start; rebase(A, year), each year)", null, true],
};
const hasFunction = (name) => Object.hasOwn(FUNCTIONS, name);
/** halving()'s and year()'s periods as they take them: the least, and what's said of one that isn't. */
const PERIODS_TAKEN = /** @type {Record<string, [number, string]>} */ ({ halving: [0, "n and at are cycles, 0, 1, 2 and on"], year: [2009, "y and at are years, 2009 (the genesis block's) and on"] });
/** A function's arguments' names, as its signature writes them. */
const argNames = (/** @type {string} */ f) => ((signature) => signature.slice(signature.indexOf("(") + 1, -1).split(", "))(FUNCTIONS[f][1]);

/** A date in a formula: a year, a month, a day (2020-05-11). */
const DATE_TOKEN = /\s*(\d{4})-(\d{1,2})-(\d{1,2})(?![\w.])/y;
const TOKEN = /\s*(?:((?:\d+\.?\d*|\.\d+)(?:e[+-]?\d+)?)([a-z]\w*)?|([A-Z]{1,2})(?![A-Za-z0-9_])|([a-z_][a-z0-9_]*)|([<>]=?|[=!]=|[-+*/^(),&|]))/y;
/** A formula that only isn't finished yet (more to type, not a mistake). */
const unfinished = (/** @type {string} */ message) => Object.assign(new Error(message), { unfinished: true });
/**
 * A formula's word: a number (a date's, a span's: its unit's name), a layer's letter, a name, an operator.
 * @typedef {{ t: "num", v: number, text: string, unit?: string, date?: true } | { t: "ref" | "id" | "op", v: string }} Token
 */
/** @returns {Token[]} */
function tokenize(/** @type {string} */ source) {
  const tokens = /** @type {Token[]} */ ([]);
  let index = 0;
  while (index < source.length) {
    if (/^\s*$/.test(source.slice(index))) break;
    // (A date, 2020-05-11: its time, UTC, as each point's `date` is.)
    DATE_TOKEN.lastIndex = index;
    const day = DATE_TOKEN.exec(source);
    if (day) {
      // (Its year as written: Date.UTC reads 0 to 99 as 1900 on.)
      const [year, month, date] = day.slice(1).map(Number), at = new Date(0);
      at.setUTCFullYear(year, month - 1, date);
      const time = at.getTime();
      if (at.getUTCFullYear() !== year || at.getUTCMonth() !== month - 1 || at.getUTCDate() !== date) throw new Error(`${day[0].trim()} isn't a date: a year, a month, a day (2020-05-11)`);
      tokens.push({ t: "num", v: time / 1000, text: day[0].trim(), date: true });
      index = DATE_TOKEN.lastIndex;
      continue;
    }
    TOKEN.lastIndex = index;
    const match = TOKEN.exec(source);
    if (!match) throw new Error(((char) => (char === "=" ? 'Unexpected "=": to compare, write ==' : `Unexpected "${char}"`))(source.slice(index).trim()[0]));
    index = TOKEN.lastIndex;
    if (match[1]) {
      if (match[2] && !Object.hasOwn(SPAN_LETTERS, match[2])) throw new Error(`Unknown unit "${match[2]}" in ${match[0].trim()}: h, d, w, mo, y or halving`);
      tokens.push({ t: "num", v: Number(match[1]), unit: match[2] ? SPAN_LETTERS[/** @type {keyof typeof SPAN_LETTERS} */ (match[2])] : undefined, text: match[0].trim() });
    } else if (match[3]) tokens.push({ t: "ref", v: match[3] });
    else if (match[4]) tokens.push({ t: "id", v: match[4] });
    else tokens.push({ t: "op", v: match[5] });
  }
  return tokens;
}
/** Each operator's precedence: or, and, a comparison (true 1, false 0), then the arithmetic. */
const BINARY = { "|": 2, "&": 4, "<": 6, ">": 6, "<=": 6, ">=": 6, "==": 6, "!=": 6, "+": 10, "-": 10, "*": 20, "/": 20, "^": 30 };

const plainOperators = (source) => source.replace(/[×·]/g, "*").replace(/÷/g, "/").replace(/[−–]/g, "-").replace(/≥/g, ">=").replace(/≤/g, "<=").replace(/≠/g, "!=");
const isBandSum = (/** @type {string} */ f) => f === "over" || f === "under";
/**
 * over(x) or under(x) as the sum it is: x's band and those above it, or those below it (a band without the series, as
 * 0-sat outputs have no supply, adds nothing), added from the far end in: each cutoff is the one beyond it plus a
 * band, a part they share (see parts). What can't be added (a price, a percent) isn't.
 * @param {Extract<Node, { k: "call" }>} node @returns {Node}
 */
function bandSum(node) {
  const signature = FUNCTIONS[node.f][1], arg = node.args[0], found = arg.k === "series" ? inBands(tree, arg.v) : undefined;
  if (!found) throw new Error(`${signature}: x is a series of an age, amount, balance, profit or loss band, by its name, like age_1y_to_18m_supply`);
  if (found.path[0] === "costBasis" || found.path.includes("percent")) throw new Error(`${signature}: a price or a percent doesn't add up across bands: sum what it's made of, then divide`);
  const names = bandNames(found, node.f);
  if (!names.length) throw new Error(`${signature}: no band below this one has this series`);
  return names.map((v) => /** @type {Node} */ ({ k: "series", v })).reduce((a, b) => ({ k: "bin", op: "+", a, b }));
}

/** Formulas as parsed, by their text (each edit normalizes every layer: its formula's read once, not again). */
const parsed = new Map();
/** @returns {Node} */
function parse(/** @type {string} */ source) {
  let node = parsed.get(source);
  if (node) return node;
  node = parseAnew(source);
  if (parsed.size > 500) parsed.clear();
  parsed.set(source, node);
  return node;
}
/** @returns {Node} */
function parseAnew(source) {
  const tokens = tokenize(plainOperators(source));
  if (!tokens.length) throw new Error(`Write a formula, like A / B or ${EXAMPLE}`);
  let at = 0;
  const peek = () => tokens[at];
  const expect = (value) => {
    const token = tokens[at++];
    if (!token || token.v !== value) throw token ? new Error(`Expected "${value}" before "${token.v}"`) : unfinished(`Missing "${value}"`);
  };
  /** @returns {Node} */
  function primary() {
    const token = tokens[at++];
    if (!token) throw unfinished("The formula ends too early");
    if (token.t === "num") return token.unit ? { k: "num", v: token.v, unit: token.unit, text: token.text } : token.date ? { k: "num", v: token.v, date: true } : { k: "num", v: token.v };
    if (token.t === "ref") return { k: "ref", v: token.v };
    if (token.t === "id") {
      // A name with arguments is a function; alone, it's a series, by its name on Bitview.
      if (peek()?.v !== "(") {
        if (VALUES.has(token.v)) return { k: /** @type {"date" | "year"} */ (token.v) };
        if (hasFunction(token.v)) throw new Error(`${token.v} needs its arguments: ${FUNCTIONS[token.v][1]}`);
        if (/^[a-z]{1,2}$/.test(token.v)) throw new Error(`Unknown name "${token.v}": layers are capital letters (A, B…)`);
        return { k: "series", v: token.v };
      }
      if (!hasFunction(token.v)) throw new Error(`Unknown function "${token.v}"`);
      const [arity, signature, , , spanless] = FUNCTIONS[token.v];
      expect("(");
      const args = [];
      if (peek()?.v !== ")") {
        do args.push(expression(0));
        while (peek()?.v === "," && at++);
      }
      expect(")");
      if (args.length !== arity && !(spanless && args.length === arity - 1)) throw new Error(`${token.v} takes ${spanless ? `${arity - 1} or ${arity}` : arity} argument${arity > 1 ? "s" : ""}: ${signature}`);
      return { k: "call", f: token.v, args };
    }
    if (token.v === "(") {
      const inner = expression(0);
      expect(")");
      return inner;
    }
    if (token.v === "-") return { k: "neg", a: expression(25) };
    if (token.v === "+") return expression(25);
    throw new Error(`Unexpected "${token.v}"`);
  }
  /** @returns {Node} */
  function expression(min) {
    let left = primary(), compared = false;
    for (;;) {
      const token = peek();
      if (token?.t !== "op") return left;
      const precedence = BINARY[token.v];
      if (precedence === undefined || precedence < min) return left;
      at++;
      // (Two comparisons in a row say nothing: 1 < A < 3 is (1 < A) < 3, a 0 or 1 under 3. Joined, they do; one in
      // parentheses, compared, is meant: (A > 1) > 0.)
      if (COMPARES.has(token.v) && compared) throw new Error("Join comparisons with & or |: A > 1 & A < 3");
      compared = COMPARES.has(token.v);
      const a = left, b = expression(token.v === "^" ? precedence : precedence + 1);
      // (A date compared with a bare number (date >= 2020: seconds since 1970) is a date written short.)
      if (COMPARES.has(token.v) && [[a, b], [b, a]].some(([one, other]) => one.k === "date" && other.k === "num" && !other.date && !other.unit)) throw unfinished("Compare date with a date, written 2020-05-01");
      left = { k: "bin", op: token.v, a, b };
    }
  }
  const node = expression(0);
  if (at < tokens.length) throw new Error(`Unexpected "${tokens[at].v}"`);
  return node;
}

/**
 * The layers a formula reads, by letter, or with `kind` "series", the series it names, "reads", those it reads (a band
 * sum's bands), "written", those as it writes them (a band sum whole, its over(x)), "call", the functions it calls.
 * @param {Node} node @returns {Set<string>}
 */
function refsOf(node, kind = "ref", into = new Set()) {
  if (node.k === "call" && isBandSum(node.f) && kind === "written") into.add(`${node.f}(${/** @type {{ v: string }} */ (node.args[0]).v})`);
  else if (node.k === "call" && isBandSum(node.f) && kind === "reads") {
    try {
      refsOf(bandSum(node), "series", into);
    } catch {
      // (Not a band's series: computing it says so.)
    }
  } else if (node.k === "ref" || node.k === "series") {
    if (node.k === (kind === "written" || kind === "reads" ? "series" : kind)) into.add(node.v);
  }
  else if (node.k === "neg") refsOf(node.a, kind, into);
  else if (node.k === "bin") (refsOf(node.a, kind, into), refsOf(node.b, kind, into));
  else if (node.k === "call") (kind === "call" && into.add(node.f), node.args.forEach((arg) => refsOf(arg, kind, into)));
  return into;
}
/** Whether a formula shifts by halvings somewhere (shift(A, 1halving)). @param {any} node @returns {boolean} */
/** Whether a formula shifts anywhere in it (rebase(shift(A, 1halving) * 2, halving_epoch): a moved series). @param {any} node @returns {boolean} */
const shifts = (node) =>
  node?.k === "call" ? node.f === "shift" || node.args.some(shifts) : node?.k === "neg" ? shifts(node.a) : node?.k === "bin" ? shifts(node.a) || shifts(node.b) : false;
const shiftsByHalvings = (node) =>
  node?.k === "call" ? (node.f === "shift" && shiftSpan(node.args[1])?.unit === "halving") || node.args.some(shiftsByHalvings) : node?.k === "neg" ? shiftsByHalvings(node.a) : node?.k === "bin" ? shiftsByHalvings(node.a) || shiftsByHalvings(node.b) : false;
/** The series a formula reads without naming them: halving()'s cycles, a shift's by halvings. @returns {string[]} */
function seriesUnnamed(/** @type {string} */ formula) {
  try {
    const node = parse(formula);
    return refsOf(node, "call").has("halving") || shiftsByHalvings(node) ? ["halving_epoch"] : [];
  } catch {
    return [];
  }
}
/** The series a formula layer can turn into: those it names. @param {Layer} layer */
const turnsInto = (layer) => (layer.formula !== undefined ? seriesNamed(layer.formula, "written").filter(isSeriesName) : []);
/** The series a formula names (`kind`: see refsOf), if it parses. @returns {string[]} */
function seriesNamed(formula, kind = "series") {
  try {
    return [...refsOf(parse(formula), kind)];
  } catch {
    return [];
  }
}
/** A formula with its series renamed (`names`: old to new), all at once (as series: a function of the same name stays). */
const renamed = (/** @type {string} */ formula, /** @type {Map<string, string>} */ names) => (names.size ? formula.replace(new RegExp(`(?<![A-Za-z0-9_])(${[...names.keys()].join("|")})(?![A-Za-z0-9_])(?!\\s*\\()`, "g"), (name) => /** @type {string} */ (names.get(name))) : formula);
/** The unit a function's result carries, from its input's. */
const unitOfCall = (/** @type {string} */ f, /** @type {string | null} */ unit) => (KEEPS_UNIT.has(f) ? unit : IN_PERCENT.has(f) ? "percent" : f === "rebase" || f === "halving" || f === "year" ? "ratio" : "number");
/**
 * The unit a formula's result carries, from its parts (`unitOfRef(name, kind)`); null for plain numbers.
 * @param {Node} node @param {(name: string, kind: "ref" | "series") => string | null} unitOfRef @returns {string | null}
 */
function unitOfNode(node, unitOfRef) {
  switch (node.k) {
    case "num":
    case "date":
    case "year":
      return null;
    case "ref":
    case "series":
      return unitOfRef(node.v, node.k);
    case "neg":
      return unitOfNode(node.a, unitOfRef);
    case "bin": {
      const a = unitOfNode(node.a, unitOfRef), b = unitOfNode(node.b, unitOfRef);
      if (node.op === "+" || node.op === "-") return a && b && a !== b ? "number" : a ?? b;
      if (node.op === "*") return a && b ? "number" : a ?? b;
      if (node.op === "/") return !b ? a : a === b ? "ratio" : "number";
      return "number";
    }
    case "call":
      if (isBandSum(node.f)) return unitOfNode(bandSum(node), unitOfRef);
      if (node.f === "min" || node.f === "max") {
        const a = unitOfNode(node.args[0], unitOfRef), b = unitOfNode(node.args[1], unitOfRef);
        return a && b && a !== b ? "number" : a ?? b;
      }
      return unitOfCall(node.f, KEEPS_UNIT.has(node.f) ? unitOfNode(node.args[0], unitOfRef) : null);
  }
}

/**
 * Every part of a formula as computed, whichever formula, layer or chart needed it first, by its operation and its
 * inputs by identity (see idOf: a series on a timeline is one array, a part another): the same inputs, the same
 * array, never made twice and never out of date (an input that changes is another array, so another key). Its arrays
 * are shared, so they're never written to: what changes one copies it first (see liveCopy). The most recently used.
 * (Under it, the series on each chart's points, alignedSets; over it, each layer's finished result, frame.computed.)
 * @type {Map<string, Vector | number>}
 */
const parts = new Map();
/** How many `parts` holds (a daily series is about 52 KB). Not at the fine intervals: arrays of up to 96,000 points, made anew as the view moves. */
const KEPT_PARTS = 500;
const keepsParts = (/** @type {boolean} */ inWindow, /** @type {string} */ index) => !inWindow && !FINE.has(index);
/** The part at `key`, made once. @param {string} key @param {() => Vector | number} make @returns {Vector | number} */
function part(key, make) {
  if (parts.has(key)) return lastUsed(parts, key, /** @type {Vector | number} */ (parts.get(key)));
  const value = make();
  parts.set(key, value);
  if (parts.size > KEPT_PARTS) parts.delete(/** @type {string} */ (parts.keys().next().value));
  return value;
}
/** An input's place in a key: a series or part by its identity, a number as itself (-0 apart: a division tells them apart). @param {Vector | number} value */
const inputKey = (value) => (typeof value !== "number" ? `#${idOf(value)}` : Object.is(value, -0) ? "-0" : String(value));
/**
 * Evaluates to a vector of `length` points, or a scalar; windows (spans of time, in days, where `inWindow`) run as
 * the points of `index`. A part already computed, in this formula or any other, is that one (see parts).
 * @returns {Vector | number}
 */
function evaluate(node, length, ref, index, inWindow = false) {
  if (node.k === "call" && isBandSum(node.f)) return evaluate(bandSum(node), length, ref, index, inWindow);
  if (node.k !== "bin" && node.k !== "neg") return evaluateAnew(node, length, ref, index, inWindow);
  // (Each side once, a side that fails failing the whole: a sum of bands is as deep as its bands.)
  const a = evaluate(node.a, length, ref, index, inWindow), b = node.k === "bin" ? evaluate(node.b, length, ref, index, inWindow) : undefined;
  const make = () => (node.k === "bin" ? pointwise(node.op, a, /** @type {Vector | number} */ (b)) : typeof a === "number" ? -a : restarted(a.map((v) => -v), a));
  return !keepsParts(inWindow, index) || (typeof a === "number" && typeof b !== "object") ? make() : part(`${node.k === "bin" ? node.op : "neg"}(${inputKey(a)},${b === undefined ? "" : inputKey(b)})`, make);
}
/** The four operations a formula is mostly made of, over at least one vector, each its own loop (no call per point: without a JIT, it's most of the cost). @param {string} op @param {Vector | number} a @param {Vector | number} b @returns {Vector} */
function arithmetic(op, a, b) {
  const n = /** @type {Vector} */ (typeof a === "number" ? b : a).length, out = new Float64Array(n);
  const x = typeof a === "number" ? new Float64Array(n).fill(a) : a, y = typeof b === "number" ? new Float64Array(n).fill(b) : b;
  if (op === "+") for (let i = 0; i < n; i++) out[i] = x[i] + y[i];
  else if (op === "-") for (let i = 0; i < n; i++) out[i] = x[i] - y[i];
  else if (op === "*") for (let i = 0; i < n; i++) out[i] = x[i] * y[i];
  else for (let i = 0; i < n; i++) out[i] = x[i] / y[i];
  return restarted(out, a, b);
}
/** Two sides point by point, by `f`, with their restarts. @param {Vector | number} a @param {Vector | number} b @param {(x: number, y: number) => number} f @returns {Vector | number} */
function lift(a, b, f) {
  if (typeof a === "number") return typeof b === "number" ? f(a, b) : restarted(b.map((v) => f(a, v)), b);
  return restarted(typeof b === "number" ? a.map((v) => f(v, b)) : a.map((v, i) => f(v, b[i])), a, b);
}
// (A comparison, or two joined, is 1 where true, 0 where false; where either has no value, none: an infinity (a
// log of 0, a division by 0) is none either. Joined, a side with none decides nothing the other decides
// alone: true | none is true, false & none false.)
const either = (/** @type {(x: number, y: number) => boolean} */ test) => (/** @type {number} */ x, /** @type {number} */ y) => (!Number.isFinite(x) || !Number.isFinite(y) ? NaN : Number(test(x, y)));
const or = (/** @type {number} */ x, /** @type {number} */ y) => ((Number.isFinite(x) && x !== 0) || (Number.isFinite(y) && y !== 0) ? 1 : Number.isFinite(x) && Number.isFinite(y) ? 0 : NaN);
const and = (/** @type {number} */ x, /** @type {number} */ y) => (x === 0 || y === 0 ? 0 : Number.isFinite(x) && Number.isFinite(y) ? 1 : NaN);
/** Each operator on two numbers. @type {Record<string, (x: number, y: number) => number>} */
const POINTWISE = { "+": (x, y) => x + y, "-": (x, y) => x - y, "*": (x, y) => x * y, "/": (x, y) => x / y, "^": (x, y) => x ** y, "<": either((x, y) => x < y), ">": either((x, y) => x > y), "<=": either((x, y) => x <= y), ">=": either((x, y) => x >= y), "==": either((x, y) => x === y), "!=": either((x, y) => x !== y), "&": and, "|": or };
/** An operator over two sides. @param {string} op @param {Vector | number} a @param {Vector | number} b */
const pointwise = (op, a, b) => ("+-*/".includes(op) && (typeof a !== "number" || typeof b !== "number") ? arithmetic(op, a, b) : lift(a, b, POINTWISE[op]));
/** @param {Exclude<Node, { k: "bin" | "neg" }>} node @returns {Vector | number} */
function evaluateAnew(node, length, ref, index, inWindow) {
  const sub = (/** @type {Node} */ child, within = inWindow) => evaluate(child, length, ref, index, within);
  const vec = (value) => (typeof value === "number" ? new Float64Array(length).fill(value) : value);
  switch (node.k) {
    case "num":
      if (node.unit && !inWindow) throw new Error(`${node.text} is a span of time, for a window: sma(x, ${node.text})`);
      return node.unit ? node.v * SPAN_DAYS[node.unit] : node.v;
    case "ref":
    case "series":
      return ref(node.v, node.k);
    case "date":
      return ref("date", "date");
    case "year":
      return yearsOf(vec(ref("date", "date")));
    case "call": {
      const [arity, signature, , run, , usual] = FUNCTIONS[node.f];
      const windowed = arity >= 2 && run !== null, spanned = windowed && node.args.length === arity;
      if (node.f === "percentile" && spanned && node.args[1].k === "num" && node.args[1].unit) throw new Error(`${signature}: p first, the span last`);
      // (Its span as written, not a window's days: see shiftSpan.)
      if (node.f === "shift") {
        const span = shiftSpan(node.args[1]), problem = shiftSpanError(span), step = INTERVAL_DAYS[index];
        if (!span || problem) throw new Error(problem);
        const days = Math.abs(span.v * SPAN_DAYS[span.unit]), halvings = span.unit === "halving";
        if (halvings && FINE.has(index)) throw failing(`shift by halvings: each cycle from its first point, in all of history: more than ${INTERVAL_LABEL[index]} loads; ${INTERVAL_LABEL[fitsFrom(Infinity, index)]} and up hold it`, `Needs ${INTERVAL_LABEL[fitsFrom(Infinity, index)]}+`);
        if (days < 0.9 * step) throw failing(`shift: ${span.text} is less than a point at ${INTERVAL_LABEL[index]}: a longer span, or a finer interval`, "Span too short");
        const x = vec(sub(node.args[0])), calendar = span.unit === "month" || span.unit === "year";
        const out = shiftOf(x, span, calendar ? vec(ref("date", "date")) : null, halvings ? vec(ref("halving_epoch", "series")) : null, step);
        // (Nothing where there was something: moved past all the data there is.)
        if (!out.some(Number.isFinite) && x.some(Number.isFinite)) throw windowTooLong("shift", span.text, days, index, days / step > length, true);
        return out;
      }
      const args = node.args.map((arg, i) => sub(arg, spanned ? i === arity - 1 : inWindow));
      const compute = () => {
        if (node.f === "when") return lift(args[0], args[1], (x, holds) => (holds && Number.isFinite(holds) ? x : NaN));
        // (Each stretch from its first value: all of history's, which a finer interval only loads part of.)
        if (node.f === "rebase" && FINE.has(index)) throw failing(`rebase: from each stretch's first value, in all of history: more than ${INTERVAL_LABEL[index]} loads; ${INTERVAL_LABEL[fitsFrom(Infinity, index)]} and up hold it`, `Needs ${INTERVAL_LABEL[fitsFrom(Infinity, index)]}+`);
        // (A shifted series rebased by period (one period laid over another): a period begun late left out there.)
        if (node.f === "rebase") return rebaseOf(vec(args[0]), args[1] === undefined ? null : vec(args[1]), shifts(node.args[0]));
        // (A halving cycle or a year alone, from 1× at its start: where it is, or moved to start where another does.)
        if (node.f === "halving" || node.f === "year") {
          const cycles = node.f === "halving", [least, says] = PERIODS_TAKEN[node.f];
          if (FINE.has(index)) throw failing(`${node.f}: from its ${cycles ? "cycle" : "year"}'s first value, in all of history: more than ${INTERVAL_LABEL[index]} loads; ${INTERVAL_LABEL[fitsFrom(Infinity, index)]} and up hold it`, `Needs ${INTERVAL_LABEL[fitsFrom(Infinity, index)]}+`);
          const [n, at] = args.slice(1);
          if ([n, at ?? least].some((period) => typeof period !== "number" || !Number.isInteger(period) || period < least)) throw new Error(`${signature}: ${says}`);
          return periodAlone(vec(args[0]), cycles ? vec(ref("halving_epoch", "series")) : yearsOf(vec(ref("date", "date"))), /** @type {number} */ (n), /** @type {number | undefined} */ (at));
        }
        if (node.f === "perday") return restarted(perDayOf(vec(args[0]), vec(ref("date", "date")), index, !FINE.has(index)), args[0]);
        // (Where one has none, none: an infinity too, a division by 0.)
        if (!run) return lift(args[0], args[1], ((most) => (/** @type {number} */ x, /** @type {number} */ y) => (Number.isFinite(x) && Number.isFinite(y) ? most(x, y) : NaN))(node.f === "min" ? Math.min : Math.max));
        if (windowed) {
          // (A usual span, in points at any interval: 14 RSI's, 26 MACD's; its days for what's said.)
          const days = spanned ? args[arity - 1] : usual ? usual * INTERVAL_DAYS[index] : Infinity, p = args[1];
          if (days === Infinity && FINE.has(index)) throw failing(`${node.f} over all of history: more than ${INTERVAL_LABEL[index]} loads; ${INTERVAL_LABEL[fitsFrom(Infinity, index)]} and up hold it, or give it a span (like 2y)`, `Needs ${INTERVAL_LABEL[fitsFrom(Infinity, index)]}+`);
          if (typeof days !== "number" || !(days > 0)) throw new Error(`${signature}: the window is a span of time, like 30d, 4w, 6mo or 2y`);
          if (node.f === "flow" && !flowFits(days, index)) throw failing("flow: calendar-year differences need a whole number of yearly samples", "Needs yearly samples");
          if (arity === 3 && (typeof p !== "number" || !(p >= 0 && p <= 100))) throw new Error(`${signature}: p is a percent, from 0 to 100`);
          const x = vec(args[0]), points = days === Infinity ? Infinity : !spanned && usual ? usual : windowPoints(/** @type {number} */ (days), index);
          if (TWO_POINTS.has(node.f) && points < 2) throw failing(`${node.f}: its window is one point at ${INTERVAL_LABEL[index]}, and it needs two at least (a longer span, or a finer interval)`, "Span too short");
          const out = fromStart(x, (from) => run(from, points, /** @type {number} */ (p), !FINE.has(index), INTERVAL_DAYS[index]));
          // (A growth over all of history, or a span, with nothing to grow from: said so.)
          if (node.f === "cagr" && !out.some(Number.isFinite) && !outreaches(x, points === Infinity ? 1 : points)) throw new Error("cagr: growth needs positive values, two at least");
          // (Where a window must be full: on a finer interval, and for a value a span back. Those warming up twice, an
          // RSI's stochastic, MACD's signal, reach twice as far.)
          const reach = node.f.startsWith("stochrsi") ? points * 2 + (node.f === "stochrsi_d" ? 3 : node.f === "stochrsi_k" ? 1 : -1) : ["macd_signal", "macd_hist"].includes(node.f) ? points * 2 : points;
          if (!out.some(Number.isFinite) && points !== Infinity && (FINE.has(index) || ["change", "diff", "cagr", "rsi", "stochrsi", "stochrsi_k", "stochrsi_d", "chop"].includes(node.f)) && outreaches(x, reach)) {
            const written = spanned ? node.args[arity - 1] : null;
            throw windowTooLong(node.f, !written ? `${grouped.format(points)} points` : written.k === "num" ? (written.unit ? written.text ?? "" : `${written.v}d`) : `${grouped.format(points)} points`, /** @type {number} */ (days), index, reach > length, ["change", "diff"].includes(node.f));
          }
          return out;
        }
        // (A drawdown is from the highest of all history: on a finer interval, only part of it loads.)
        if (node.f === "drawdown" && FINE.has(index)) throw failing(`drawdown: from the highest of all history, more than ${INTERVAL_LABEL[index]} loads; ${INTERVAL_LABEL[fitsFrom(Infinity, index)]} and up hold it`, `Needs ${INTERVAL_LABEL[fitsFrom(Infinity, index)]}+`);
        const x = vec(args[0]), out = run(x, 0, 0, false, INTERVAL_DAYS[index]);
        // (Point by point, a drawdown aside: from the highest so far, past any restart.)
        return node.f === "drawdown" ? out : restarted(out, x);
      };
      return keepsParts(inWindow, index) && !UNSHARED.has(node.f) && args.some((arg) => typeof arg !== "number") ? part(`${node.f}@${index}(${args.map(inputKey)})`, compute) : compute();
    }
  }
}

/**
 * Formula source as colored HTML: letters in their layer's color, functions, series by name, numbers, operators; a
 * letter with no layer, an unknown function or a series Bitview doesn't have (`missing`) marked.
 */
function highlightFormula(source, colorOf, missing = (_name) => false) {
  const pattern = /(\d{4}-\d{1,2}-\d{1,2}(?![\w.])|(?:\d+\.?\d*|\.\d+)(?:e[+-]?\d+)?(?<unit>[a-z]\w*)?)|([A-Z]{1,2})(?![A-Za-z0-9_])|([a-z_][a-z0-9_]*)|([<>≥≤]=?|[=!]=|[-+*/^(),×·÷−–&|≠])/g;
  let html = "", last = 0;
  for (const match of source.matchAll(pattern)) {
    html += escapeHtml(source.slice(last, match.index));
    // (A number's unit, its group by name, a slot of its own too: skipped.)
    const [text, number, , ref, id] = match;
    if (number) html += `<span class="${match.groups?.unit && !Object.hasOwn(SPAN_LETTERS, match.groups.unit) ? "t-bad" : "t-num"}">${escapeHtml(text)}</span>`;
    else if (ref) {
      const color = colorOf(ref);
      html += color ? `<span class="t-ref" style="--c:${escapeHtml(color)}">${text}</span>` : `<span class="t-bad">${text}</span>`;
    } else if (id) {
      const call = /^\s*\(/.test(source.slice(match.index + text.length));
      // (`date` and `year`, each point's: the language's own, as a function's name is.)
      html += `<span class="${call ? (hasFunction(id) ? "t-fn" : "t-bad") : VALUES.has(id) ? "t-fn" : missing(id) ? "t-bad" : "t-series"}">${escapeHtml(text)}</span>`;
    }
    else html += `<span class="t-op">${escapeHtml(text)}</span>`;
    last = match.index + text.length;
  }
  return `${html}${escapeHtml(source.slice(last))}\n`;
}
/** Bitcoin's first block (3 January 2009): all of history's start, for a range over it. */
const GENESIS = 1230940800;
/** A span written as a sum (6h + 6h), its days; one that names a series, all of history. @param {any} node @returns {number} */
const spanDaysOf = (node) => {
  if (node.k === "num") return node.unit ? node.v * SPAN_DAYS[node.unit] : node.v;
  if (node.k === "neg") return -spanDaysOf(node.a);
  // (The less or the more of two: max(6h, 12h).)
  if (node.k === "call" && (node.f === "min" || node.f === "max") && node.args.length === 2) return Math[/** @type {"min" | "max"} */ (node.f)](spanDaysOf(node.args[0]), spanDaysOf(node.args[1]));
  if (node.k !== "bin") return Infinity;
  const a = spanDaysOf(node.a), b = spanDaysOf(node.b);
  return { "+": a + b, "-": a - b, "*": a * b, "/": a / b, "^": a ** b }[/** @type {string} */ (node.op)] ?? Infinity;
};
/** Each formula's windows (its spans, in days), as an interval's fit weighs them: read once a formula. @type {Map<string, { days: number, two: boolean, flow?: boolean }[]>} */
const windowsSeen = new Map();
function windowsOf(/** @type {string} */ formula) {
  let windows = windowsSeen.get(formula);
  if (windows) return windows;
  const found = /** @type {{ days: number, two: boolean, flow?: boolean }[]} */ ([]);
  /** @param {any} node */
  const walk = (node) => {
    if (!node || typeof node !== "object") return;
    if (node.k === "call") {
      const [arity, , , run, , usual] = FUNCTIONS[node.f] ?? [];
      // (A usual span is in points, at any interval; one written, days; none, all of history.)
      if (arity >= 2 && run !== null) {
        const span = node.args.length === arity ? node.args[arity - 1] : null;
        found.push({ days: span ? spanDaysOf(span) : usual ? 0 : Infinity, two: TWO_POINTS.has(node.f) && Boolean(span), flow: node.f === "flow" });
      }
      // (A drawdown's from the highest of all history; a rebase, from each stretch's first value in it; a shift, its
      // span back (or on), by halvings each cycle from its first point.)
      if (node.f === "drawdown" || node.f === "rebase" || node.f === "halving" || node.f === "year") found.push({ days: Infinity, two: false });
      if (node.f === "shift") found.push({ days: shiftSpan(node.args[1])?.unit === "halving" ? Infinity : Math.abs(spanDaysOf(node.args[1])), two: false });
      node.args.forEach(walk);
    } else (walk(node.a), walk(node.b));
  };
  try {
    walk(parse(formula));
  } catch {}
  if (windowsSeen.size > 500) windowsSeen.clear();
  windowsSeen.set(formula, (windows = found));
  return windows;
}
/** Whether a formula can be computed at an interval: its windows (all of history among them) within what a fine one holds, two points at least where it needs them. */
const computableAt = (/** @type {string} */ formula, /** @type {string} */ id) => flowsAt(formula, id) && windowsOf(formula).every(({ days, two }) => (!FINE.has(id) || (Number.isFinite(days) && windowPoints(days, id) < CHUNK)) && (!two || windowPoints(days, id) >= 2));
/**
 * Why formulas a click adds can't be computed at the chart's interval, if they can't: the intervals they can be
 * (the chart stays where it's put). @param {string[]} formulas
 */
const unfitHere = (formulas) => {
  if (formulas.every((formula) => computableAt(formula, chart.index))) return "";
  // (Those they can be, one run of them: a fine one's too short, a coarse one's too few points.)
  const supported = menus.supportedIntervals(chart), fit = INTERVAL_ORDER.filter((id) => supported(id) && formulas.every((formula) => computableAt(formula, id)));
  return `Not at ${INTERVAL_LABEL[chart.index]}${fit.length ? `: ${INTERVAL_LABEL[fit[0]]}${fit.length > 1 ? ` to ${INTERVAL_LABEL[/** @type {string} */ (fit.at(-1))]}` : ""}` : ""}`;
};
/** The pane under a point of the page (its index, its box), or none. */
function paneAt(clientY) {
  for (let i = 0; i < engine.panes.length; i++) {
    const rect = paneBox(i);
    if (rect && clientY >= rect.top && clientY <= rect.bottom) return { index: i, rect };
  }
  return null;
}
/** The point with a value nearest a point (itself, if it has one), or -1. */
function nearestFinite(values, i) {
  for (let d = 0; d < values.length; d++) {
    if (Number.isFinite(values[i - d])) return i - d;
    if (Number.isFinite(values[i + d])) return i + d;
  }
  return -1;
}
export {
  argNames, bandSum, breaksAt, COMPARES, evaluate, FUNCTIONS, GENESIS, hasFunction, highlightFormula,
  fromStart, isBandSum, nearestFinite, paneAt, parse, PERIODS_TAKEN, plainOperators, refsOf, renamed, restarts, rolling,
  seriesNamed, seriesUnnamed, shiftSpan, shiftSpanError, turnsInto, unfitHere, unitOfNode, unstarted, VALUES, windowsOf,
};
