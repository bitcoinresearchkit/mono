// @ts-check
// The page's reading and measuring, off the main thread, in the order the page shows things: the ranking stored with
// the page (shown at once), the price (the chart starts on it; beside it the server's catalog, which says what the
// lines are), the first three lines whole (the chart draws them), every line on the days it's held to (today's
// ranking, which replaces the stored one), the price's candles, and last every line by week (the chart's crowd, and
// which lines are ceilings).

import { PRICE, EPOCH, SHOWN, WEEK, NAMED, WORKED, weekOf, lowsOf, linesOf, spans, stale, ranking, ceiling } from "./study.js";

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
/** The ranking stored with the page at its release (scripts/home.mjs writes it from bitview.space): shown at once, and where its reads start. @returns {Promise<import("./study.js").Board>} */
async function stored() {
  const response = await fetch(new URL("./ranking.json", import.meta.url), { signal: AbortSignal.timeout(WAIT) });
  if (!response.ok) throw new Error(`the server answered ${response.status}`);
  const board = await response.json();
  if (!Array.isArray(board?.lows) || !Array.isArray(board?.ranked) || !Array.isArray(board?.ceilings)) throw new Error("it isn't a ranking");
  // (JSON has no Infinity: a line with no value at a bottom has its worst written as null.)
  for (const r of board.ranked) r.worst ??= Infinity;
  return board;
}
const post = (/** @type {any} */ message, /** @type {Transferable[]} */ handed = []) => /** @type {any} */ (self).postMessage(message, handed);
const why = (/** @type {unknown} */ error) => (error instanceof Error ? error.message : String(error));

async function main() {
  const named = catalog();
  // (Read beside the price but awaited after it: its failure is said once, where it's awaited, not first as an unhandled rejection.)
  named.catch(() => {});
  // (Without the stored ranking nothing is known ahead: that is said, and today's is read in full.)
  const known = stored().then((board) => (post({ type: "ranked", live: false, ...board }), board), (error) => (post({ type: "unstored", why: why(error) }), null));
  const [price, epochs] = await read([PRICE, EPOCH], 0), n = price.start + price.data.length, last = n - 1;
  /** What was read of each series, on the price's days: NaN where nothing was. @type {Map<string, Float64Array>} */ const got = new Map();
  const put = (/** @type {string} */ id, /** @type {{ start: number, data: (number | null)[] }} */ body) => {
    const out = got.get(id) ?? new Float64Array(n).fill(NaN);
    for (let i = 0, end = Math.min(body.data.length, n - body.start); i < end; i++) out[body.start + i] = body.data[i] ?? NaN;
    return got.set(id, out), out;
  };
  const fill = async (/** @type {string[]} */ ids, /** @type {number} */ a, /** @type {number} */ b) => (await read(ids, a, b)).forEach((body, i) => put(ids[i], body));
  /** Spans of many lines in a few asks: those of one fall together, by their last day, each ask as many as the server takes. */
  const spanned = async (/** @type {ReturnType<typeof spans>} */ plan) => {
    /** @type {ReturnType<typeof spans>[]} */ const asks = [];
    for (const s of plan.toSorted((a, b) => a.from - b.from || a.to - b.to)) {
      const ask = asks.at(-1);
      if (ask && ask[0].from === s.from && ask.length < MOST && (ask.length + 1) * (s.to - s.from + 1) <= VALUES) ask.push(s);
      else asks.push([s]);
    }
    await Promise.all(asks.map((ask) => fill(ask.map((s) => s.id), ask[0].from, /** @type {{ to: number }} */ (ask.at(-1)).to + 1)));
  };
  const close = put(PRICE, price), lows = lowsOf(close, put(EPOCH, epochs));
  post({ type: "price", close: close.slice(), lows });
  for (const id in WORKED) got.set(id, WORKED[id](n));
  // (The chart draws the first three over every day it shows: read whole, the stored ranking's first, not waiting for the
  // catalog, as the stored ranking is of the same release as the server it's served with; ahead of the ranking's reads.)
  const whole = new Set();
  const draw = async (/** @type {import("./study.js").Ranked[]} */ ranked) => {
    const more = ranked.slice(0, 3).map((r) => r.row.id).filter((id) => !whole.has(id));
    if (!more.length) return;
    more.forEach((id) => whole.add(id)), await fill(more.filter((id) => !(id in WORKED)), SHOWN, n);
    const top = more.map((id) => ({ id, line: /** @type {Float64Array} */ (got.get(id)).slice() }));
    post({ type: "lines", lines: top }, top.map((l) => l.line.buffer));
  };
  // (A stored line the server stopped serving since the release it was written before is read again, as today's first
  // three, once ranked: the stored ranking only starts the reads, the catalog says what is served.)
  const drawn = known.then((board) => draw(board?.ranked ?? [])).catch(() => whole.clear());
  const made = linesOf(await named), ids = made.map((l) => l.id).filter((id) => !(id in WORKED)), board = await known;
  const plan = spans(lows, ids, board);
  // (Beside the first three, not after them: held back, the ranking would wait on their bytes on a slow link.)
  await spanned(plan);
  const all = made.map((m) => ({ ...m, line: /** @type {Float64Array} */ (got.get(m.id)) }));
  /** Today's ranking without the lines taken as ceilings, its stale spans read through first. */
  const rank = async (/** @type {Set<string>} */ ceilings) => {
    const ranked = ranking(close, lows, all.filter((l) => !ceilings.has(l.id))), rest = stale(lows, plan, ranked);
    if (!rest.length) return ranked;
    await spanned(rest);
    return ranking(close, lows, all.filter((l) => !ceilings.has(l.id)));
  };
  const said = (/** @type {Set<string>} */ ceilings, /** @type {import("./study.js").Ranked[]} */ ranked) => post({ type: "ranked", live: true, day: last, lows, ranked, ceilings: made.filter((m) => ceilings.has(m.id)) });
  // (Ceilings as the stored ranking has them until the crowd's weeks say: they change only as weeks add up.)
  let ceilings = new Set(board?.ceilings.map((m) => m.id)), ranked = await rank(ceilings);
  said(ceilings, ranked);
  await drawn, await draw(ranked);
  // (The price as candles, for a view zoomed in: a column each.)
  const [ohlc] = await read(["price_ohlc"], SHOWN), candles = [0, 1, 2].map(() => new Float64Array(n).fill(NaN));
  ohlc.data.forEach((day, i) => { if (day) for (let k = 0; k < 3; k++) candles[k][ohlc.start + i] = day[k]; });
  post({ type: "candles", open: candles[0], high: candles[1], low: candles[2] }, candles.map((c) => c.buffer));
  // (Every line by week: which are ceilings, and the chart's crowd, a row each of the ranked lines in their order. A week's value is its last day's.)
  const from = weekOf(SHOWN), weeks = weekOf(last) - from + 1, read1 = await read(ids, from, from + weeks, "week1");
  const weekly = [...ids.map((id, i) => /** @type {const} */ ([id, read1[i]])), ...Object.keys(WORKED).map((id) => /** @type {const} */ ([id, { start: from, data: Array.from({ length: weeks }, (_, w) => /** @type {Float64Array} */ (got.get(id))[Math.min(last, (from + w) * WEEK.days + WEEK.day)]) }]))];
  const today = new Set(weekly.filter(([, body]) => ceiling(close, body.data, body.start)).map(([id]) => id));
  if (today.size !== ceilings.size || [...today].some((id) => !ceilings.has(id))) (ceilings = today), (ranked = await rank(ceilings)), said(ceilings, ranked), await draw(ranked);
  const crowd = new Float32Array(ranked.length * weeks).fill(NaN), place = new Map(ranked.map((r, k) => [r.row.id, k]));
  for (const [id, body] of weekly) { const k = place.get(id); if (k !== undefined) crowd.set(body.data.map((v) => v ?? NaN), k * weeks + body.start - from); }
  post({ type: "crowd", from, weeks, values: crowd }, [crowd.buffer]);
}
main().catch((error) => post({ type: "failed", why: why(error) }));
