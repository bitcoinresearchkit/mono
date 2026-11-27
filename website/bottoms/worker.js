// @ts-check
// The page's reading and measuring, off the main thread, in the order the page shows things: the price (the chart
// starts on it; beside it the server's catalog, which says what the lines are), then every line on the days it's held
// to (the ranking), then the first three lines whole (the chart draws them), the price's candles, and last every
// line by week (the chart's crowd).

import { PRICE, EPOCH, SHOWN, WEEK, NAMED, weekOf, lowsOf, windows, linesOf, servedOf, lines, ranking } from "./study.js";

const WAIT = 30000;
/** What the server's bulk route takes in one ask: this many series, and this many of their values in all (it weighs a line's value at 16, and takes 320,000). */
const MOST = 32, VALUES = 20000;
/**
 * Series by day, or by week, from `from` up to `to` (to the last when not said), in `ids`' order: a few asks for many
 * lines, not one a line.
 * @returns {Promise<{ start: number, data: any[] }[]>}
 */
async function read(/** @type {string[]} */ ids, /** @type {number} */ from, to = Infinity, index = "day1") {
  // (How many days isn't known until the price is read: then, a series an ask.)
  const each = Math.max(1, Math.min(MOST, Math.floor(VALUES / (to - from)))), asks = Array.from({ length: Math.ceil(ids.length / each) }, (_, k) => ids.slice(k * each, (k + 1) * each));
  return (await Promise.all(asks.map(async (some) => {
    const what = some.length > 1 ? `${some[0]} and ${some.length - 1} more` : some[0];
    const response = await fetch(`/api/series/bulk?series=${some.join(",")}&index=${index}&start=${from}${to < Infinity ? `&end=${to}` : ""}`, { signal: AbortSignal.timeout(WAIT) });
    if (!response.ok) throw new Error(`${what}: the server answered ${response.status}`);
    const bodies = await response.json();
    if (!Array.isArray(bodies) || bodies.length !== some.length || !bodies.every((body) => Array.isArray(body?.data))) throw new Error(`${what}: the server's answer couldn't be read`);
    return bodies;
  }))).flat();
}
/** The series the server serves whose names contain one of `NAMED`: its catalog list, read a page an ask for each (the first says how many there are). */
async function catalog() {
  const read = async (/** @type {string} */ q) => {
    const page = async (/** @type {number} */ k) => {
      const response = await fetch(`/api/series/list?q=${encodeURIComponent(q)}&page=${k}`, { signal: AbortSignal.timeout(WAIT) });
      if (!response.ok) throw new Error(`the series list of "${q}", page ${k}: the server answered ${response.status}`);
      const body = await response.json();
      if (!Array.isArray(body?.series)) throw new Error(`the series list of "${q}", page ${k}: the server's answer couldn't be read`);
      return body;
    };
    const first = await page(0), names = [first, ...(await Promise.all(Array.from({ length: first.max_page }, (_, k) => page(k + 1))))].flatMap((body) => body.series);
    // (A server updated between two pages would shift them: what was read then isn't its list.)
    if (new Set(names).size !== first.total_count) throw new Error(`the series list of "${q}" changed while it was read: ${names.length} names of ${first.total_count}`);
    return names;
  };
  return new Set((await Promise.all(NAMED.map(read))).flat());
}
const post = (/** @type {any} */ message, /** @type {Transferable[]} */ handed = []) => /** @type {any} */ (self).postMessage(message, handed);

async function main() {
  const named = catalog();
  // (Read beside the price but awaited after it: its failure is said once, where it's awaited, not first as an unhandled rejection.)
  named.catch(() => {});
  const [price, epochs] = await read([PRICE, EPOCH], 0), n = price.start + price.data.length, last = n - 1;
  /** What was read of each series, on the price's days: NaN where nothing was. @type {Map<string, Float64Array>} */ const got = new Map();
  const put = (/** @type {string} */ id, /** @type {{ start: number, data: (number | null)[] }} */ body) => {
    const out = got.get(id) ?? new Float64Array(n).fill(NaN);
    for (let i = 0, end = Math.min(body.data.length, n - body.start); i < end; i++) out[body.start + i] = body.data[i] ?? NaN;
    return got.set(id, out), out;
  };
  const fill = async (/** @type {string[]} */ ids, /** @type {number} */ a, /** @type {number} */ b) => (await read(ids, a, b)).forEach((body, i) => put(ids[i], body));
  const close = put(PRICE, price), lows = lowsOf(close, put(EPOCH, epochs));
  post({ type: "price", close: close.slice(), lows });
  const made = linesOf(await named), ids = servedOf(made), served = new Set(ids);
  await Promise.all(windows(lows, last).map(([from, to]) => fill(ids, from, to + 1)));
  const get = (/** @type {string} */ id) => /** @type {Float64Array} */ (got.get(id)), all = lines(made, get), ranked = ranking(close, lows, all);
  post({ type: "ranked", ranked });
  // (The chart draws the first three over every day it shows: the ones read only in part are read whole now.)
  const first = ranked.slice(0, 3).map((r) => r.row.id);
  await fill(first.filter((id) => served.has(id)), SHOWN, n);
  const top = first.map((id) => ({ id, line: (got.get(id) ?? /** @type {import("./study.js").Line} */ (all.find((l) => l.id === id)).line).slice() }));
  post({ type: "lines", lines: top }, top.map((l) => l.line.buffer));
  // (The price as candles, for a view zoomed in: a column each.)
  const [ohlc] = await read(["price_ohlc"], SHOWN), candles = [0, 1, 2].map(() => new Float64Array(n).fill(NaN));
  ohlc.data.forEach((day, i) => { if (day) for (let k = 0; k < 3; k++) candles[k][ohlc.start + i] = day[k]; });
  post({ type: "candles", open: candles[0], high: candles[1], low: candles[2] }, candles.map((c) => c.buffer));
  // (Every line by week, a row each in the ranking's order: what the chart's crowd is drawn from. A week's value is its last day's.)
  const from = weekOf(SHOWN), weeks = weekOf(last) - from + 1, crowd = new Float32Array(ranked.length * weeks).fill(NaN), place = new Map(ranked.map((r, k) => [r.row.id, k]));
  (await read(ids, from, from + weeks, "week1")).forEach((body, i) => crowd.set(body.data.map((v) => v ?? NaN), /** @type {number} */ (place.get(ids[i])) * weeks + body.start - from));
  for (const { id, line } of all) if (!served.has(id)) for (let w = 0, row = /** @type {number} */ (place.get(id)) * weeks; w < weeks; w++) crowd[row + w] = line[Math.min(last, (from + w) * WEEK.days + WEEK.day)];
  post({ type: "crowd", from, weeks, values: crowd }, [crowd.buffer]);
}
main().catch((error) => post({ type: "failed", why: error instanceof Error ? error.message : String(error) }));
