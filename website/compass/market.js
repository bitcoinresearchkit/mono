// @ts-check

import { WeightedRank } from "/modules/weighted-rank/index.js";

/** Where price is against a cloud: 1 above, 0 inside, -1 below. @typedef {1 | 0 | -1} Side */
export const STATES = /** @type {{ id: string, name: string, move: string, cycle: number, local: number }[]} */ ([
  { id: "optimism", name: "Optimism", move: "Advance", cycle: 1, local: 1 },
  { id: "calm", name: "Calm", move: "Pullback", cycle: 1, local: 0 },
  { id: "anxiety", name: "Anxiety", move: "Correction", cycle: 1, local: -1 },
  { id: "hope", name: "Hope", move: "Recovery", cycle: 0, local: 1 },
  { id: "doubt", name: "Doubt", move: "Crossroads", cycle: 0, local: 0 },
  { id: "worry", name: "Worry", move: "Breakdown", cycle: 0, local: -1 },
  { id: "relief", name: "Relief", move: "Bounce", cycle: -1, local: 1 },
  { id: "apathy", name: "Apathy", move: "Stalling", cycle: -1, local: 0 },
  { id: "fear", name: "Fear", move: "Decline", cycle: -1, local: -1 },
]).map((s) => ({ ...s, trend: 1 - Math.sign(s.cycle + s.local) }));
export const stateOf = (/** @type {number} */ cycle, /** @type {number} */ local) => (1 - cycle) * 3 + (1 - local);
export const TRENDS = [
  { name: "Bull", color: "optimism", arrow: "M7.75 13.25v-10.5M3.75 6.75l4-4 4 4" },
  { name: "Mixed", color: "doubt", arrow: "M2.75 7.75h10.5M9.25 3.75l4 4-4 4" },
  { name: "Bear", color: "fear", arrow: "M7.75 2.75v10.5M3.75 9.25l4 4 4-4" },
];

export const DAY = 86400e3, EPOCH = Date.UTC(2009, 0, 1);

export const SERIES = { close: "price_close" };
/** @typedef {"mobile" | "awake"} Coins */
/**
 * @typedef {"percentiles-and-average" | "percentiles" | "medians" | "averages" | "medians-and-averages"} Basis
 */
export const STATS = {
  "percentiles-and-average": ["p40", "p60", "avg"],
  percentiles: ["p40", "p60"],
  "medians-and-averages": ["median", "avg"],
  medians: ["median"],
  averages: ["avg"],
};
/** The steadiest basis: its nine moods about as even as any. */
export const BASIS = /** @type {Basis} */ ("percentiles-and-average");
export const cloudIds = (/** @type {"long" | "short"} */ cloud, /** @type {Coins} */ which, /** @type {Basis} */ by) => (cloud === "long" ? ["age_over_4m", "lth", "age_over_6m"] : ["age_under_4m", "sth", "age_under_6m"]).flatMap((k) => STATS[by].map((stat) => `${k}_${which}_cost_basis_by_dollar_${stat}`));
/** @type {Map<string, Promise<Answer>>} */
export const answers = new Map();
/** Every block moves each series' last day, so they are read together or not at all. */
export async function readAll(/** @type {Coins} */ which, /** @type {Basis} */ by) {
  // The tip is asked first: a block landing between only makes the next ask read them again.
  const tip = await (await ask("/api/blocks/tip/hash")).text();
  const ids = [SERIES.close, ...cloudIds("long", which, by), ...cloudIds("short", which, by)];
  for (const id of ids) {
    const held = answers.get(id);
    answers.set(id, (held ? held.then((answer) => (answer.tip === tip ? answer : renew(id, answer, tip))) : read(id).then((answer) => ({ ...answer, tip }))).catch((error) => (answers.delete(id), Promise.reject(error))));
  }
  return Object.fromEntries(await Promise.all(ids.map(async (id) => [id, await answers.get(id)])));
}
async function renew(/** @type {string} */ id, /** @type {Answer} */ held, /** @type {string} */ tip) {
  const last = held.start + held.data.length - 1, tail = await read(id, last);
  return { start: held.start, data: held.data.slice(0, last - held.start).concat(tail.data), tip };
}

/** @typedef {{ start: number, data: (number | null)[], tip?: string }} Answer */
const START = (Date.UTC(2010, 6, 1) - EPOCH) / DAY;
async function read(/** @type {string} */ id, from = START) {
  const body = await (await ask(`/api/series/${id}/day1?start=${from}`)).json().catch(() => null);
  if (!Array.isArray(body?.data)) throw new Error("the server's answer couldn't be read");
  return /** @type {Answer} */ (body);
}
const WAIT = 20_000;
/** Without a limit a request can hang for good, and the page with it. */
export async function ask(/** @type {string} */ url) {
  const signal = AbortSignal.timeout(WAIT);
  const response = await fetch(url, { signal }).catch((error) => {
    // Asked of the signal, not the error: WebKit rejects a timed-out fetch as an AbortError.
    if (signal.aborted) throw new Error(`the server didn't answer in ${WAIT / 1000} seconds`);
    throw error instanceof TypeError ? new Error("this server couldn't be reached") : error;
  });
  if (!response.ok) throw new Error((await response.json().catch(() => null))?.error?.message ?? `the server answered ${response.status}`);
  return response;
}
function byDay(/** @type {Answer} */ answer, /** @type {number} */ n) {
  const out = new Float64Array(n).fill(NaN);
  answer.data.forEach((v, i) => {
    if (v !== null && v > 0 && answer.start + i < n) out[answer.start + i] = v;
  });
  return out;
}

/**
 * `x` the moment's, `y` the cycle's, each -1 to 1.
 * @typedef {{ first: number, last: number, close: Float64Array, cloud: { long: [Float64Array, Float64Array], short: [Float64Array, Float64Array] }, state: Int8Array, x: Float32Array, y: Float32Array, index: Float64Array, extreme: Uint8Array }} Days
 */

/** A halving cycle: history's weight halves each, so recent cycles count more than the first. */
const HALF_LIFE = 1461;
/** From day 0, not the day read: only the ratios matter, and a sum stays good as later days join it. */
export const weight = (/** @type {number} */ day) => 2 ** (day / HALF_LIFE);
const weightOf = (/** @type {number} */ a, /** @type {number} */ b) => (weight(b + 1) - weight(a)) / (weight(1) - 1);
/** Inside a cloud the middle third by log place between its edges; outside, the outer third by weighted rank. */
export function placeOf(/** @type {Float64Array} */ close, /** @type {[Float64Array, Float64Array]} */ [bottom, top], /** @type {number} */ first, /** @type {number} */ last) {
  const place = new Float32Array(close.length);
  const BUCKETS = 4096, WIDTH = 0.002, sides = [new WeightedRank(BUCKETS), new WeightedRank(BUCKETS)];
  for (let i = first; i <= last; i++) {
    const p = close[i], lo = bottom[i], hi = top[i];
    if (p >= lo && p <= hi) {
      place[i] = hi > lo ? -1 / 3 + ((2 / 3) * Math.log(p / lo)) / Math.log(hi / lo) : 0;
      continue;
    }
    const k = p > hi ? 0 : 1, side = sides[k], bucket = Math.min(BUCKETS - 1, Math.floor(Math.log(k ? lo / p : p / hi) / WIDTH));
    const rank = side.total ? side.below(bucket + 1) / side.total : 0.5;
    side.add(bucket, weight(i));
    place[i] = (k ? -1 : 1) * (1 / 3 + (2 / 3) * rank);
  }
  return place;
}

/** With fewer days behind it, a place outside a cloud would be ranked against next to nothing. */
const INDEXED_AFTER = 365;
export const indexedFrom = (/** @type {Days} */ d) => d.first + INDEXED_AFTER;

const EXTREME_PERCENT = 1;
/** Ranked against that side's earlier days only (not the day itself), and only with a year behind it. */
export function extremeOf(/** @type {Float32Array[]} */ sides, /** @type {number} */ first, /** @type {number} */ last) {
  const extreme = new Uint8Array(sides[0].length), sorted = new Float64Array(last - first + 1);
  const rankOf = (/** @type {number} */ v, atMost = false) => {
    let lo = 0, hi = sorted.length;
    while (lo < hi) {
      const mid = (lo + hi) >> 1;
      if (sorted[mid] < v || (atMost && sorted[mid] === v)) lo = mid + 1;
      else hi = mid;
    }
    return lo;
  };
  for (const place of sides) {
    sorted.set(place.subarray(first, last + 1));
    sorted.sort();
    const before = new WeightedRank(sorted.length);
    for (let i = first; i <= last; i++) {
      const v = place[i], rank = rankOf(v), mass = before.total;
      if (i >= first + INDEXED_AFTER && (before.below(rank) / mass < EXTREME_PERCENT / 100 || (mass - before.below(rankOf(v, true))) / mass < EXTREME_PERCENT / 100)) extreme[i] = 1;
      before.add(rank, weight(i));
    }
  }
  return extreme;
}

export function compute(/** @type {Record<string, Answer>} */ answered, /** @type {Coins} */ which, /** @type {Basis} */ by) {
  const n = Math.max(...Object.values(answered).map((a) => a.start + a.data.length));
  const get = (/** @type {string} */ id) => byDay(answered[id], n);
  const close = get(SERIES.close);
  const lines = (/** @type {string[]} */ ids) => ids.map(get);
  const cloudOf = (/** @type {Float64Array[]} */ members) => {
    const bottom = new Float64Array(n).fill(NaN), top = new Float64Array(n).fill(NaN);
    for (let i = 0; i < n; i++) {
      let low = Infinity, high = -Infinity;
      for (const line of members) {
        const value = line[i];
        if (!(value > 0)) (low = NaN), (high = NaN);
        else (low = Math.min(low, value)), (high = Math.max(high, value));
      }
      if (low > 0) (bottom[i] = low), (top[i] = high);
    }
    return /** @type {[Float64Array, Float64Array]} */ ([bottom, top]);
  };
  const long = cloudOf(lines(cloudIds("long", which, by))), short = cloudOf(lines(cloudIds("short", which, by)));
  const state = new Int8Array(n).fill(-1);
  let first = -1, last = -1;
  for (let i = 0; i < n; i++) {
    const p = close[i];
    if (!(p > 0) || !(long[0][i] > 0) || !(short[0][i] > 0)) continue;
    const side = (/** @type {[Float64Array, Float64Array]} */ c) => (p > c[1][i] ? 1 : p < c[0][i] ? -1 : 0);
    state[i] = stateOf(side(long), side(short));
    if (first < 0) first = i;
    last = i;
  }
  for (let i = first + 1; i < last; i++) {
    if (state[i] >= 0) continue;
    close[i] = close[i - 1];
    for (const edge of [...long, ...short]) edge[i] = edge[i - 1];
    state[i] = state[i - 1];
  }
  const x = placeOf(close, short, first, last), y = placeOf(close, long, first, last);
  // Not ranked again: x and y already rank how far out the price is.
  const index = new Float64Array(n).fill(NaN);
  for (let i = first + INDEXED_AFTER; i <= last; i++) index[i] = 25 * (x[i] + y[i] + 2);
  return { first, last, close, cloud: { long, short }, state, x, y, index, extreme: extremeOf([x, y], first, last) };
}

/**
 * `next`: -1 while the mood runs at the last day; `weightTo`: the mood's runs' weight so far, its own included.
 * @typedef {{ first: number, last: number, length: number, next: number, weightTo: number }} Run
 */
/** @type {WeakMap<Days, Run[][]>} */
const runsMade = new WeakMap();
export function runsOf(/** @type {Days} */ d) {
  let made = runsMade.get(d);
  if (made) return made;
  const of = d.state;
  made = STATES.map(() => /** @type {Run[]} */ ([]));
  for (let i = d.first, start = d.first; i <= d.last; i++) {
    if (i < d.last && of[i + 1] === of[i]) continue;
    const runs = made[of[i]], prior = runs.at(-1);
    runs.push({ first: start, last: i, length: i - start + 1, next: i < d.last ? of[i + 1] : -1, weightTo: (prior?.weightTo ?? 0) + weightOf(start, i) });
    start = i + 1;
  }
  runsMade.set(d, made);
  return made;
}
/** What was known that day: none after it is looked at. */
export function endedBy(/** @type {Run[]} */ runs, /** @type {number} */ day) {
  let lo = 0, hi = runs.length;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (runs[mid].last < day) lo = mid + 1;
    else hi = mid;
  }
  return lo;
}
export const runAt = (/** @type {Days} */ d, /** @type {number} */ day) => {
  const runs = runsOf(d)[d.state[day]];
  return runs[endedBy(runs, day)];
};

export function spellsOf(/** @type {Days} */ d, upTo = d.last) {
  const of = d.state, now = runAt(d, upTo);
  const all = runsOf(d).map((runs, k) => {
    const n = endedBy(runs, upTo), spent = (n ? runs[n - 1].weightTo : 0) + (of[upTo] === k ? weightOf(now.first, upTo) : 0);
    return { share: spent / weightOf(d.first, upTo) };
  });
  return { moods: all, now: { mood: of[upTo], first: now.first } };
}
/** Each by its last day's weight; NaN with none. All pass: exactly 1, both sums the same additions. */
export function shareOf(/** @type {Run[]} */ runs, /** @type {number} */ n, /** @type {(run: Run) => boolean} */ test) {
  let passed = 0, all = 0;
  for (let r = 0; r < n; r++) {
    const w = weight(runs[r].last);
    all += w;
    if (test(runs[r])) passed += w;
  }
  return passed / all;
}
/** Each by its last day's weight; 0 with none. */
export function medianOf(/** @type {Run[]} */ runs, /** @type {number} */ n) {
  if (!n) return 0;
  const order = Array.from({ length: n }, (_, r) => r).sort((p, q) => runs[p].length - runs[q].length);
  let all = 0, held = 0, k = 0;
  for (let r = 0; r < n; r++) all += weight(runs[r].last);
  while ((held += weight(runs[order[k]].last)) * 2 < all) k++;
  return runs[order[k]].length;
}
export const tenths = (/** @type {number} */ share) => Math.min(9, Math.max(1, Math.round(share * 10)));
export const lengthWords = (/** @type {number} */ n, months = Math.round(n / 30.44)) => (n >= 30 ? `${months}\u00a0month${months === 1 ? "" : "s"}` : `${n}\u00a0day${n === 1 ? "" : "s"}`);
/** Largest remainders, so the sides read as the sums of the cells they're shown beside. */
function percents(/** @type {{ share: number }[]} */ of) {
  const whole = of.map(({ share }) => Math.floor(share * 100));
  const rest = 100 - whole.reduce((sum, n) => sum + n, 0);
  for (const i of of.map((_, i) => i).sort((p, q) => (of[q].share * 100 - whole[q]) - (of[p].share * 100 - whole[p])).slice(0, rest)) whole[i]++;
  return whole;
}
export function cellWords(/** @type {Days} */ d, upTo = d.last) {
  const spells = spellsOf(d, upTo);
  return { spells, shares: percents(spells.moods) };
}

export const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
export const dateOf = (/** @type {number} */ day) => new Date(EPOCH + day * DAY);
export const dated = (/** @type {number} */ day, year = true) => {
  const d = dateOf(day);
  return `${d.getUTCDate()} ${MONTHS[d.getUTCMonth()]}${year ? ` ${d.getUTCFullYear()}` : ""}`;
};
