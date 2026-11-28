// @ts-check
// The page's reading and measuring, off the main thread, in the order the eye waits on: the ranking stored with the
// page (shown at once), the price (the chart starts on it; beside it the server's catalog, which says what the lines
// are), the first three lines whole and any the page asks for (the chart draws them), every line by week (the chart's crowd, drawn as it lands),
// every line on the days it's held to (today's ranking, which only confirms or replaces the stored one; with the
// crowd's weeks it says which lines are ceilings), and last the price's candles. Every ask is tried twice. The page
// fails only without the price or today's ranking: what fails past that is a layer, left out and said, and read
// again when the page asks.

import { PRICE, EPOCH, SHOWN, NAMED, weekOf, lowsOf, linesOf, spans, stale, ranking, ceiling } from "./study.js";

const WAIT = 30000;
/** What the server's bulk route takes in one ask: this many series, and this many of their values in all (it weighs a line's value at 16, and takes 320,000). */
const MOST = 32, VALUES = 20000;
const why = (/** @type {unknown} */ error) => (error instanceof Error ? error.message : String(error));
/** An ask's JSON, asked once more when it fails (a slow link drops one now and then); a failure says what was asked. */
async function json(/** @type {string | URL} */ url, /** @type {string} */ what) {
  const once = async () => {
    const response = await fetch(url, { signal: AbortSignal.timeout(WAIT) });
    if (!response.ok) throw new Error(`the server answered ${response.status}`);
    return response.json();
  };
  return once().catch(once).catch((error) => { throw new Error(`${what}: ${why(error)}`); });
}
/**
 * Series by day, or by week, from `from` up to `to` (to the last when not said), in `ids`' order: a few asks for many
 * lines, not one a line, each ask's handed to `landed` as it lands.
 * @returns {Promise<{ start: number, data: any[] }[]>}
 */
async function read(/** @type {string[]} */ ids, /** @type {number} */ from, to = Infinity, index = "day1", /** @type {(bodies: { start: number, data: any[] }[], some: string[]) => void} */ landed = () => {}) {
  // (How many days isn't known until the price is read: then, a series an ask.)
  const each = Math.max(1, Math.min(MOST, Math.floor(VALUES / (to - from)))), asks = Array.from({ length: Math.ceil(ids.length / each) }, (_, k) => ids.slice(k * each, (k + 1) * each));
  const answers = asks.map(async (some) => {
    const what = some.length > 1 ? `${some[0]} and ${some.length - 1} more` : some[0];
    const bodies = await json(`/api/series/bulk?series=${some.join(",")}&index=${index}&start=${from}${to < Infinity ? `&end=${to}` : ""}`, what);
    if (!Array.isArray(bodies) || bodies.length !== some.length || !bodies.every((body) => Array.isArray(body?.data))) throw new Error(`${what}: the server's answer couldn't be read`);
    return landed(bodies, some), bodies;
  });
  // (Failing only once every ask is in: a second read asks for what failed, never for what's still on its way.)
  await Promise.allSettled(answers);
  return (await Promise.all(answers)).flat();
}
/** The series the server serves whose names contain one of `NAMED`: its catalog list, read a page an ask for each (the first says how many there are). */
async function catalog() {
  const read = async (/** @type {string} */ q) => {
    const page = async (/** @type {number} */ k) => {
      const body = await json(`/api/series/list?q=${encodeURIComponent(q)}&page=${k}`, `the series list of "${q}", page ${k}`);
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
/** The ranking stored with the page at its release (scripts/home.mjs writes it from the release's server): shown at once, and where its reads start. @returns {Promise<import("./study.js").Board>} */
async function stored() {
  const board = await json(new URL("./ranking.json", import.meta.url), "ranking.json");
  if (!Array.isArray(board?.lows) || !Array.isArray(board?.ranked) || !Array.isArray(board?.ceilings)) throw new Error("it isn't a ranking");
  // (JSON has no Infinity: a line with no value at a bottom has its worst written as null.)
  for (const r of board.ranked) r.worst ??= Infinity;
  return board;
}
const post = (/** @type {any} */ message, /** @type {Transferable[]} */ handed = []) => /** @type {any} */ (self).postMessage(message, handed);
/** A line the page asks to see whole (a row tapped), by id: kept until the price is read, then read. */
const asked = /** @type {string[]} */ ([]);
let ask = (/** @type {string} */ id) => void asked.push(id);
/** @typedef {"lines" | "crowd" | "ceilings" | "candles"} Layer a part of the page past today's ranking */
/** A layer the page asks for again, once said to be missed. @type {(name: Layer) => void} */
let again;
self.onmessage = (/** @type {MessageEvent<{ id: string, again?: Layer }>} */ { data }) => (data.again ? again(data.again) : ask(data.id));
/** A layer's read, said either way: got, or left out where it fails. @template T @returns {Promise<T | undefined>} */
const layer = (/** @type {Layer} */ name, /** @type {Promise<T>} */ made) => made.then((value) => (post({ type: "got", layer: name }), value), (error) => void post({ type: "missed", layer: name, why: why(error) }));

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
  // (The chart draws the first three over every day it shows, and a line the page asks for: each read whole once, the
  // stored ranking's first three not waiting for the catalog, as the stored ranking is of the same release as the server
  // it's served with; ahead of the ranking's reads. One that fails can be asked again.)
  /** Each line's whole read, by id, kept unless it fails: whoever asks for one in flight waits on it too. @type {Map<string, Promise<void>>} */
  const whole = new Map();
  const draw = (/** @type {string[]} */ ids) => {
    const more = ids.filter((id) => !whole.has(id));
    if (more.length) {
      const read = fill(more, SHOWN, n).then(() => {
        const top = more.map((id) => ({ id, line: /** @type {Float64Array} */ (got.get(id)).slice() }));
        post({ type: "lines", lines: top }, top.map((l) => l.line.buffer));
      }, (error) => { throw (more.forEach((id) => whole.delete(id)), error); });
      more.forEach((id) => whole.set(id, read));
    }
    return Promise.all(ids.map((id) => whole.get(id)));
  };
  const firstOf = (/** @type {import("./study.js").Ranked[]} */ ranked) => ranked.slice(0, 3).map((r) => r.row.id);
  ask = (id) => void draw([id]).catch((error) => post({ type: "unread", id, why: why(error) }));
  asked.forEach(ask);
  // (A stored line the server stopped serving since the release it was written before is read again, as today's first
  // three, once ranked: the stored ranking only starts the reads, the catalog says what is served.)
  const drawn = known.then((board) => draw(firstOf(board?.ranked ?? []))).catch(() => {});
  const made = linesOf(await named), ids = made.map((l) => l.id), board = await known;
  // (Every line by week, the chart's crowd, each ask's lines sent as they land: asked
  // once the first three are drawn, so they don't share a slow link with them, and before the ranking's spans, as the
  // stored ranking already shows. A week's value is its last day's.)
  const from = weekOf(SHOWN), weeks = weekOf(last) - from + 1;
  /** Every line's weeks, by id, as they land. @type {Map<string, { start: number, data: (number | null)[] }>} */
  const weekly = new Map();
  const send = (/** @type {{ start: number, data: (number | null)[] }[]} */ bodies, /** @type {string[]} */ some) => {
    some.forEach((id, k) => weekly.set(id, bodies[k]));
    const values = new Float32Array(bodies.length * weeks).fill(NaN);
    bodies.forEach((body, k) => values.set(body.data.map((v) => v ?? NaN), k * weeks + body.start - from));
    post({ type: "crowd", from, weeks, values }, [values.buffer]);
  };
  /** The weeks not read yet: all of them first, what failed when asked again. */
  const crowd = () => read(ids.filter((id) => !weekly.has(id)), from, from + weeks, "week1", send);
  const first = drawn.then(crowd);
  // (Awaited where it's needed: a failure is said there, once.)
  first.catch(() => {});
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
  const closest = () => layer("lines", drawn.then(() => draw(firstOf(ranked))));
  /** Which lines are ceilings, by the crowd's weeks: "settled" says the ones now ranked without are today's. */
  const settle = async () => {
    const today = new Set(ids.filter((id) => { const { data, start } = /** @type {{ start: number, data: (number | null)[] }} */ (weekly.get(id)); return ceiling(close, data, start); }));
    if (today.size === ceilings.size && [...today].every((id) => ceilings.has(id))) return void post({ type: "settled" });
    const again = await rank(today);
    (ceilings = today), (ranked = again), said(ceilings, ranked), post({ type: "settled" });
    await closest();
  };
  /** The crowd, then today's ceilings once it's whole. */
  const judge = async (reading = crowd()) => { if (await layer("crowd", reading)) await layer("ceilings", settle()); };
  /** The price as candles, for a view zoomed in: a column each. */
  const candled = () => layer("candles", read(["price_ohlc"], SHOWN).then(([ohlc]) => {
    const candles = [0, 1, 2].map(() => new Float64Array(n).fill(NaN));
    ohlc.data.forEach((day, i) => { if (day) for (let k = 0; k < 3; k++) candles[k][ohlc.start + i] = day[k]; });
    post({ type: "candles", open: candles[0], high: candles[1], low: candles[2] }, candles.map((c) => c.buffer));
  }));
  again = (name) => void { lines: closest, crowd: judge, ceilings: judge, candles: candled }[name]();
  const redrawn = closest();
  await judge(first);
  await redrawn;
  await candled();
}
main().catch((error) => post({ type: "failed", why: why(error) }));
