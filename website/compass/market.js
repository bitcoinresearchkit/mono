// @ts-check
// Where Bitcoin's market stands, each day: its close against two clouds of holders' cost basis (the long-term and
// the short-term holders'), nine states from the two, each in one of three trends. All computed here
// from the server's daily series. One module, for the Compass page and anything else that tells the day's mood.

import { WeightedRank } from "/modules/weighted-rank/index.js";

/** Where price is against a cloud: 1 above, 0 inside, -1 below. @typedef {1 | 0 | -1} Side */
/** The nine, by the cycle's side (the long-term cloud) then the moment's (the short-term): north the cycle up. */
/** Each a mood (how the holders feel, by who's in profit) and the move it is (what price is doing), and its trend
 * (see TRENDS): the two sides added, up bull, even mixed, down bear (a grid's diagonals). */
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
/** A state's index from its two sides. */
export const stateOf = (/** @type {number} */ cycle, /** @type {number} */ local) => (1 - cycle) * 3 + (1 - local);
/** The moods' three trends, by STATES' trend: bull, mixed, bear; each in its corner mood's colour. */
export const TRENDS = [
  { name: "Bull", color: "optimism", arrow: "M8 13V3M3.5 7.5 8 3l4.5 4.5" },
  { name: "Mixed", color: "doubt", arrow: "M3 8h10M8.5 3.5 13 8l-4.5 4.5" },
  { name: "Bear", color: "fear", arrow: "M8 3v10M3.5 8.5 8 13l4.5-4.5" },
];

/** A day in milliseconds, and the day days are counted from. */
export const DAY = 86400e3, EPOCH = Date.UTC(2009, 0, 1);

export const SERIES = { close: "price_close" };
/** Which coins the clouds count (see the page's foot). @typedef {"mobile" | "awake"} Coins */
/**
 * What the clouds are made from: each of its three groups' 40th and 60th percentiles of the cost basis, by dollar,
 * with their averages or alone, their medians, their averages, or both; each basis' stats.
 * @typedef {"percentiles-and-average" | "percentiles" | "medians" | "averages" | "medians-and-averages"} Basis
 */
export const STATS = {
  "percentiles-and-average": ["p40", "p60", "avg"],
  percentiles: ["p40", "p60"],
  "medians-and-averages": ["median", "avg"],
  medians: ["median"],
  averages: ["avg"],
};
/** The basis a page opens on (a link names another): the steadiest of them, its nine moods about as even as any. */
export const BASIS = /** @type {Basis} */ ("percentiles-and-average");
/** A cloud's lines, of some coins and a basis: each of its three groups' lines, by dollar. */
export const cloudIds = (/** @type {"long" | "short"} */ cloud, /** @type {Coins} */ which, /** @type {Basis} */ by) => (cloud === "long" ? ["age_over_4m", "lth", "age_over_6m"] : ["age_under_4m", "sth", "age_under_6m"]).flatMap((k) => STATS[by].map((stat) => `${k}_${which}_cost_basis_by_dollar_${stat}`));
/** Every series read, by id: a switch of coins reads only its clouds'. @type {Map<string, Promise<Answer>>} */
export const answers = new Map();
/**
 * The series the page needs for some coins and a basis, by id, current to the chain's tip: one not read is read
 * whole, one read at an earlier block read again from its last day on (every block moves the last day of each,
 * the price's and the clouds' alike, so they are read together or not at all).
 */
export async function readAll(/** @type {Coins} */ which, /** @type {Basis} */ by) {
  // (The tip asked before the series: a block between only has the next ask read them again.)
  const tip = await (await ask("/api/blocks/tip/hash")).text();
  const ids = [SERIES.close, ...cloudIds("long", which, by), ...cloudIds("short", which, by)];
  for (const id of ids) {
    const held = answers.get(id);
    answers.set(id, (held ? held.then((answer) => (answer.tip === tip ? answer : renew(id, answer, tip))) : read(id).then((answer) => ({ ...answer, tip }))).catch((error) => (answers.delete(id), Promise.reject(error))));
  }
  return Object.fromEntries(await Promise.all(ids.map(async (id) => [id, await answers.get(id)])));
}
/** A series read at an earlier block, joined to its last day onward as the server has it now. */
async function renew(/** @type {string} */ id, /** @type {Answer} */ held, /** @type {string} */ tip) {
  const last = held.start + held.data.length - 1, tail = await read(id, last);
  return { start: held.start, data: held.data.slice(0, last - held.start).concat(tail.data), tip };
}

/** @typedef {{ start: number, data: (number | null)[], tip?: string }} Answer `tip`: the block (its hash) it was read at */
/** The first day read: the price begins in July 2010, nothing before it counts. */
const START = (Date.UTC(2010, 6, 1) - EPOCH) / DAY;
/** A series by id, from START (or a later day). */
async function read(/** @type {string} */ id, from = START) {
  const body = await (await ask(`/api/series/${id}/day1?start=${from}`)).json().catch(() => null);
  if (!Array.isArray(body?.data)) throw new Error("the server's answer couldn't be read");
  return /** @type {Answer} */ (body);
}
/** How long the server's given to answer before it's taken as not answering. */
const WAIT = 20_000;
/** This server's answer, or an error saying why there's none: it couldn't be reached, didn't answer in time, or
 * answered with an error. (Without a limit a request can hang for good, and the page with it.) */
export async function ask(/** @type {string} */ url) {
  const signal = AbortSignal.timeout(WAIT);
  const response = await fetch(url, { signal }).catch((error) => {
    // (Asked of the signal, not the error: WebKit rejects a timed-out fetch as an AbortError.)
    if (signal.aborted) throw new Error(`the server didn't answer in ${WAIT / 1000} seconds`);
    throw error instanceof TypeError ? new Error("this server couldn't be reached") : error;
  });
  if (!response.ok) throw new Error((await response.json().catch(() => null))?.error?.message ?? `the server answered ${response.status}`);
  return response;
}
/** A series by day from day 0, its missing days NaN. */
function byDay(/** @type {Answer} */ answer, /** @type {number} */ n) {
  const out = new Float64Array(n).fill(NaN);
  answer.data.forEach((v, i) => {
    if (v !== null && v > 0 && answer.start + i < n) out[answer.start + i] = v;
  });
  return out;
}

/**
 * Every day's: close, each cloud's bottom and top, the state, and where on
 * the compass (x the moment's, y the cycle's, each -1 to 1), and whether it's extreme (see extremeOf).
 * @typedef {{ first: number, last: number, close: Float64Array, cloud: { long: [Float64Array, Float64Array], short: [Float64Array, Float64Array] }, state: Int8Array, x: Float32Array, y: Float32Array, index: Float64Array, extreme: Uint8Array }} Days
 */

/** A halving cycle in days, four years: the half-life of every comparison with history on the page (a market is read
 * against its recent cycles more than its first ones). */
const HALF_LIFE = 1461;
/**
 * A day's weight in history: half as much a halving cycle further back from the day being read. Only weights' ratios
 * are read, so it's counted from day 0 instead: the same ratios, and a sum of them stays good as later days join it.
 */
export const weight = (/** @type {number} */ day) => 2 ** (day / HALF_LIFE);
/** The weights of the days from `a` to `b`, summed (a geometric series, in closed form). */
const weightOf = (/** @type {number} */ a, /** @type {number} */ b) => (weight(b + 1) - weight(a)) / (weight(1) - 1);
/**
 * Where each day from `first` to `last` is against a cloud, -1 to 1. Inside it, the middle third, by where the
 * price is between its edges (in log terms). Outside, the outer third on its side, by how far out it is (in log
 * terms) against the days before that were out on that side, as the backend's rarity meter ranks its values:
 * each by its weight, none after it. As far out as any lately is at the edge, in any cycle; a cycle whose swings
 * were wider doesn't press the ones after it against the cloud.
 */
export function placeOf(/** @type {Float64Array} */ close, /** @type {[Float64Array, Float64Array]} */ [bottom, top], /** @type {number} */ first, /** @type {number} */ last) {
  const place = new Float32Array(close.length);
  // (Each side's days so far, weighted, by how far out in buckets of 0.002; farther than the last bucket, in it.)
  const BUCKETS = 4096, WIDTH = 0.002, sides = [new WeightedRank(BUCKETS), new WeightedRank(BUCKETS)];
  for (let i = first; i <= last; i++) {
    const p = close[i], lo = bottom[i], hi = top[i];
    if (p >= lo && p <= hi) {
      place[i] = hi > lo ? -1 / 3 + ((2 / 3) * Math.log(p / lo)) / Math.log(hi / lo) : 0;
      continue;
    }
    const k = p > hi ? 0 : 1, side = sides[k], bucket = Math.min(BUCKETS - 1, Math.floor(Math.log(k ? lo / p : p / hi) / WIDTH));
    // (Its share of the days before as far out or less; the first out on its side, halfway.)
    const rank = side.total ? side.below(bucket + 1) / side.total : 0.5;
    side.add(bucket, weight(i));
    place[i] = (k ? -1 : 1) * (1 / 3 + (2 / 3) * rank);
  }
  return place;
}

/** The days the index has behind it before it's said: a year (fewer, its place outside a cloud would be ranked
 * against next to nothing). */
const INDEXED_AFTER = 365;
export const indexedFrom = (/** @type {Days} */ d) => d.first + INDEXED_AFTER;

/** How far into a side's tails a day's place is extreme: its lowest or highest 1% of the side's days before it. */
const EXTREME_PERCENT = 1;
/**
 * Whether each day from `first` to `last` is extreme (1): either side's place in its own 1% tail, low or high, ranked
 * against that side's earlier days only (the day itself not among them), each by its weight, and said, as the index
 * is, only with a year of them behind it. In a tail: the weight of the days before strictly past it, low or high,
 * under 1% of theirs. (Each side's places ranked once, sorted; the days so far, weighted, by rank.)
 */
export function extremeOf(/** @type {Float32Array[]} */ sides, /** @type {number} */ first, /** @type {number} */ last) {
  const extreme = new Uint8Array(sides[0].length), sorted = new Float64Array(last - first + 1);
  /** How many of the sorted places are under `v` (or, `atMost`, not over it). */
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
  /** A cloud from its lines: their lowest and highest each day, where every one has a value. */
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
  // (A day missing between the first and the last: the day before's, so every one of them has its own.)
  for (let i = first + 1; i < last; i++) {
    if (state[i] >= 0) continue;
    close[i] = close[i - 1];
    for (const edge of [...long, ...short]) edge[i] = edge[i - 1];
    state[i] = state[i - 1];
  }
  // (Where on the compass: the moment's place across, the cycle's up.)
  const x = placeOf(close, short, first, last), y = placeOf(close, long, first, last);
  // (The index, 0 to 100: the compass's place along its diagonal, x and y added (each -1 to 1), 0 Fear's corner and
  // 100 Optimism's, its levels the trends' diagonals. Not ranked again: x and y already rank how far out the price is.)
  const index = new Float64Array(n).fill(NaN);
  for (let i = first + INDEXED_AFTER; i <= last; i++) index[i] = 25 * (x[i] + y[i] + 2);
  return { first, last, close, cloud: { long, short }, state, x, y, index, extreme: extremeOf([x, y], first, last) };
}

// ── Runs: a mood's days in a row, each told only from the days up to it ──

/**
 * A run of a mood's days: its first and last day, its length, the mood it turned into (-1: still going at the last
 * day), and the weight of the mood's days up to its end (its runs' so far, its own included).
 * @typedef {{ first: number, last: number, length: number, next: number, weightTo: number }} Run
 */
/** Each mood's runs, in order, made once per days. @type {WeakMap<Days, Run[][]>} */
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
/** How many of a mood's runs ended before a day: what was known that day (none after it is looked at). */
export function endedBy(/** @type {Run[]} */ runs, /** @type {number} */ day) {
  let lo = 0, hi = runs.length;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (runs[mid].last < day) lo = mid + 1;
    else hi = mid;
  }
  return lo;
}
/** The run a day is in: its mood's first not ended by it. */
export const runAt = (/** @type {Days} */ d, /** @type {number} */ day) => {
  const runs = runsOf(d)[d.state[day]];
  return runs[endedBy(runs, day)];
};

/**
 * Each mood's days from the first to a day (the last, unsaid), as spells (its runs of days): its share of the days,
 * each by its weight. And the spell going on: its mood and its first day.
 */
export function spellsOf(/** @type {Days} */ d, upTo = d.last) {
  const of = d.state, now = runAt(d, upTo);
  const all = runsOf(d).map((runs, k) => {
    const n = endedBy(runs, upTo), spent = (n ? runs[n - 1].weightTo : 0) + (of[upTo] === k ? weightOf(now.first, upTo) : 0);
    return { share: spent / weightOf(d.first, upTo) };
  });
  return { moods: all, now: { mood: of[upTo], first: now.first } };
}
/**
 * The share of a mood's first `n` runs that pass a test, each by its last day's weight (NaN with none): where a run
 * stands among those before it, what they turned into. (All pass: exactly 1, both sums the same additions.)
 */
export function shareOf(/** @type {Run[]} */ runs, /** @type {number} */ n, /** @type {(run: Run) => boolean} */ test) {
  let passed = 0, all = 0;
  for (let r = 0; r < n; r++) {
    const w = weight(runs[r].last);
    all += w;
    if (test(runs[r])) passed += w;
  }
  return passed / all;
}
/** The median length of a mood's first `n` runs, each by its last day's weight (see shareOf): as much of them shorter
 * as longer (0: none). */
export function medianOf(/** @type {Run[]} */ runs, /** @type {number} */ n) {
  if (!n) return 0;
  const order = Array.from({ length: n }, (_, r) => r).sort((p, q) => runs[p].length - runs[q].length);
  let all = 0, held = 0, k = 0;
  for (let r = 0; r < n; r++) all += weight(runs[r].last);
  while ((held += weight(runs[order[k]].last)) * 2 < all) k++;
  return runs[order[k]].length;
}
/** A share in tenths, as said ("9" in 10): never 0 or 10 of them. */
export const tenths = (/** @type {number} */ share) => Math.min(9, Math.max(1, Math.round(share * 10)));
/** A spell's length in words: days, months from a month on. */
export const lengthWords = (/** @type {number} */ n, months = Math.round(n / 30.44)) => (n >= 30 ? `${months}\u00a0month${months === 1 ? "" : "s"}` : `${n}\u00a0day${n === 1 ? "" : "s"}`);
/** Whole percents of shares that add up to 100, as the shares do: each rounded down, the rest given to the largest
 * remainders. So the sides read as the sums of the cells they're shown beside. */
function percents(/** @type {{ share: number }[]} */ of) {
  const whole = of.map(({ share }) => Math.floor(share * 100));
  const rest = 100 - whole.reduce((sum, n) => sum + n, 0);
  for (const i of of.map((_, i) => i).sort((p, q) => (of[q].share * 100 - whole[q]) - (of[p].share * 100 - whole[p])).slice(0, rest)) whole[i]++;
  return whole;
}
/**
 * The moods' cells' words on a day, from their spells up to it (none after): each one's whole percent of the days
 * (all of them adding up to 100); and the spells they're from.
 */
export function cellWords(/** @type {Days} */ d, upTo = d.last) {
  const spells = spellsOf(d, upTo);
  return { spells, shares: percents(spells.moods) };
}

// ── Days in words ──

export const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
export const dateOf = (/** @type {number} */ day) => new Date(EPOCH + day * DAY);
export const dated = (/** @type {number} */ day, year = true) => {
  const d = dateOf(day);
  return `${d.getUTCDate()} ${MONTHS[d.getUTCMonth()]}${year ? ` ${d.getUTCFullYear()}` : ""}`;
};
