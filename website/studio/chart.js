// @ts-check
import { command, heatmaps } from "/studio/later.js";
import {
  Band, Bars, Candles, channels, columnWidth, Custom, DARK, Gradient, Grid, lastIn, Bubbles as LibraryBubbles, Line, Marks,
  placeIndex, Profile, rightBlank, Rows, Scale, Shade,
} from "/modules/bitview-charts/index.js";
import { OKLCH } from "/studio/palette.js";
import { hexToOklch, ofPalette, oklchToHex, pageInk } from "/studio/color.js";
import { capitalized } from "/studio/words.js";
import {
  averageFor, bandLit, bubbleRing, buildAsked, buildChart, byBlock, capturing, catalogIndex, catalogIntervals,
  changesOf, chart, chartHeight, chartSans, chartSize, CHART_TEXT, client, CLOUD, cohortName, commit, crossMoved, css, dataRevision,
  chartShown, displayed, draft, EMPTY, engine, exporting, fitMargins, footPinned, fromShort, fromUnfit, goneLayer, grow,
  heatKnown, home, hoverTime, indexAt, indexCatalog, keepStretch, keyAway, keyTime, keyValue, layerPicks, light,
  liveHeight, lookDoc, lookShown, morphFrom, mostBars, MOTION, motionsSized, navBar, ontoData, openingView, paintNavigator,
  panelShown, pending, placing, pointedLayer, pointerOnChart, profileColor, profileLit, reaching, reduced, resolved,
  rowSizes, selected, shownOklch, signalShown, sizeChart, sizeHeld, SPAN, spanKey, spanParts, spotlight, submenus,
  subSetups, tagLit, timeAt, timesAt, tokensNow, tool, updateRowChanges, viewMoving, viewTimes, widest, windowTooLong,
  within, keepFocus, heatAt,
} from "/studio/app.js";
import { $, all$, closest, escapeHtml, keyText, focusKey, restoreFocus } from "/studio/dom.js";
import { motionTime } from "/modules/motion/clock.js";
import { clamp, lowerBound } from "/studio/math.js";
import {
  amount, AXIS_CHARS, axisTicks, dateLabel, format, growth, MONTHS, precise, PRECISE, SI_UNITS, unitOf, WIDEST_VALUES,
} from "/studio/format.js";
import { ICONS } from "/studio/icons.js";
import {
  belowOf, cloudLayers, edgesOf, failing, heatLike, MARK_PLACES, MARKER, measureOf, measureUnit, MODES, opacityOf, ownGradient,
  asRows, drawnInto, paneGradient, placedOn, resultText, rowLayers, rowsLevel, signedMeasure, summed, stretchResult, TWO_KEPT,
} from "/studio/vocabulary.js";
import {
  CADENCE, CHUNK, FINE, FINE_STEP, FIRST_CHUNKS, INTERVAL_DAYS, INTERVAL_LABEL, INTERVAL_ORDER, KEPT_CHUNKS,
  MOST_CHUNKS, outreaches, rangeDays, spanDays,
} from "/studio/intervals.js";
import {
  breaksAt, evaluate, fromStart, parse, refsOf, restarts, rolling, seriesNamed, seriesUnnamed, unitOfNode, unstarted,
} from "/studio/formulas.js";
import { profileName, modeOf, stackOf, layerById, labelOf, bandOf, paneMode, named, gradientTable, profileMark, seriesMark, titleOf, rangeOf, upToDatePlaces, levelSteps, fadingOf } from "/studio/looks.js";
import { renderRanges } from "/studio/toolbar.js";
import { plotBox, PULL } from "/studio/sketch.js";
/** @import { Levels } from "/modules/bitview-charts/index.js" */
/** @import { Chart, Layer, Pane } from "/studio/vocabulary.js" */
/** @import { Vector } from "/studio/formulas.js" */
/** @import { HeatLook } from "/studio/heatmaps.js" */


/** @type {Map<string, Promise<{ description?: string, indexes: string[], type: string, accumulation: string }>>} */
const infos = new Map();
const known = new Map();
const failedInfos = new Set();
const gone = new Set();
/** Fetched once; a failure stays until Retry clears it, so edits don't refetch it. */
function info(name, signal = /** @type {AbortSignal | undefined} */ (undefined)) {
  return cachedLoad(infos, name, false, Infinity, async (_, stop) => {
    const meta = (await command?.aboutOf(name)) ?? (await client.getSeriesInfo(name, { signal: stop }));
    known.set(name, meta);
    return meta;
  }, (error) => {
    failedInfos.add(name);
    if (error?.status === 404) gone.add(name);
  }, signal);
}
/** @param {string} name @returns {readonly string[] | undefined} */
const intervalsOf = (name) => catalogIntervals?.get(name) ?? known.get(name)?.indexes;
function retryInfo(name) {
  if (!failedInfos.delete(name)) return;
  infos.delete(name);
}
const units = new Map();

/** @template T @param {Map<string, T>} cache @param {string} key @param {T} value */
const lastUsed = (cache, key, value) => (cache.delete(key), cache.set(key, value), value);
function holder() {
  const names = new Set(chart ? drawSeries(chart) : []), span = chart ? spans.get(spanKey(chart.id, chart.index)) : undefined;
  return (/** @type {string} */ key) => {
    const [name, rest = ""] = key.split("@"), [index, k] = rest.split("#");
    if (!chart || index !== chart.index || !names.has(name)) return false;
    // Its span's chunks and the next out on either side: prefetched, they're kept till the view gets there.
    return k === undefined || (Boolean(span) && Number(k) >= /** @type {Span} */ (span).from - 1 && Number(k) <= /** @type {Span} */ (span).to + 1);
  };
}
/** @param {Map<string, unknown>} cache */
function prune(cache, most, held = /** @type {((key: string) => boolean) | undefined} */ (undefined)) {
  if (cache.size <= most) return;
  const holds = held ?? holder(), loose = [...cache.keys()].filter((key) => !holds(key));
  for (const key of loose.slice(0, Math.max(0, loose.length - most))) cache.delete(key);
}
/**
 * The block a series' latest points were last asked at: asked, not answered, so a server that's down isn't asked again
 * on each edit.
 */
const askedAt = new Map();
/**
 * The first poll that answers asks again, as a first load's failure is; not each edit, which a server that's down would
 * meet again and again.
 * @type {Set<string>}
 */
const behind = new Set();
/** @param {Chart} doc */
function refreshFailed(doc) {
  const names = new Set(drawSeries(doc));
  return [...behind].some((key) => {
    const [name, rest] = key.split("@");
    return names.has(name) && rest.split("#")[0] === doc.index;
  });
}
/** @param {Chart} doc */
function stale(doc) {
  if (!liveHeight) return false;
  const span = spanOf(doc), latest = latestChunk(doc.index);
  const ends = span ? [latest - 1, latest].filter((k) => k >= span.from && k <= span.to).map((k) => `#${k}`) : [""];
  return drawSeries(doc).some((name) => ends.some((end) => (askedAt.get(`${name}@${doc.index}${end}`) ?? liveHeight) < liveHeight));
}
/** Series kept in memory at the coarse intervals past those the open chart shows (one at 1D is about 115 KB). */
const KEPT_LOADS = 100;
/** @type {Map<string, Promise<{ start: number, data: readonly any[], times: number[], version: number }>>} */
const loads = new Map();
/**
 * The points at a series' end a new block can still change, by interval (the server's margins: `cache_class` in
 * crates/bitview_primitives/src/index.rs); one at the others.
 */
const MUTABLE = { minute10: 8, minute30: 3, hour1: 2, hour4: 2, hour12: 2, day1: 2, day3: 2, week1: 2 };
/**
 * Null without one to join to, or once the server has computed the series anew (its version moved on): then it comes
 * whole.
 * @param {any} endpoint
 * @param {string} index
 * @param {{ start: number, data: readonly any[], version: number } | null} held
 * @param {number} [to]
 */
async function refetchTail(endpoint, index, held, to = undefined) {
  if (!held?.data.length) return null;
  const tail = await endpoint.slice(Math.max(held.start, held.start + held.data.length - (MUTABLE[index] ?? 1)), to).fetch(), keep = tail.start - held.start;
  return tail.version === held.version && keep >= 0 && keep <= held.data.length ? { tail, keep, start: held.start, data: held.data.slice(0, keep).concat(tail.data), version: tail.version } : null;
}
function load(name, index, fresh = false, span = /** @type {Span | null} */ (null), signal = /** @type {AbortSignal | undefined} */ (undefined)) {
  if (span) return loadSpan(name, index, span, fresh, signal);
  const key = `${name}@${index}`;
  const times = (/** @type {number} */ start, /** @type {number} */ count) => timesAt(index, start, count);
  return cachedLoad(loads, key, fresh, KEPT_LOADS, async (cached, stop) => {
    askedAt.set(key, liveHeight);
    const endpoint = client.seriesEndpoint(name, /** @type {"day1"} */ (index));
    const held = fresh && cached ? await cached.catch(() => null) : null;
    const joined = await refetchTail(endpoint, index, held);
    if (joined && held) return { start: joined.start, data: joined.data, times: held.times.slice(0, joined.keep).concat(times(joined.tail.start, joined.tail.data.length)), version: joined.version };
    const response = await endpoint.fetch({ signal: stop });
    return { start: response.start, data: response.data, times: times(response.start, response.data.length), version: response.version };
  }, undefined, signal);
}
/** @type {WeakMap<Promise<unknown>, { stop: AbortController, wanting: Set<AbortSignal> }>} */
const underWay = new WeakMap();
/**
 * A first load that fails stays (`failed` notes it), so each edit doesn't ask again. A first one asked with a `signal`
 * is given up, its fetch aborted and out of the cache, once every asker's signal has; never once one without a signal
 * has asked for it.
 * @template T
 * @param {Map<string, Promise<T>>} cache
 * @param {(cached: Promise<T> | undefined, stop?: AbortSignal) => Promise<T>} make
 */
function cachedLoad(cache, /** @type {string} */ key, /** @type {boolean} */ fresh, /** @type {number} */ most, make, failed = (/** @type {any} */ _error) => {}, signal = /** @type {AbortSignal | undefined} */ (undefined)) {
  const cached = cache.get(key);
  if (cached && !fresh) {
    const going = underWay.get(cached);
    if (going) signal ? want(going, signal) : keep(cached, going);
    return lastUsed(cache, key, cached);
  }
  if (!cached && signal?.aborted) return Promise.reject(signal.reason);
  const stop = signal && !cached ? new AbortController() : null;
  const request = make(cached, stop?.signal), result = cached ? request.then((value) => (behind.delete(key), value), () => (behind.add(key), cached)) : request;
  if (stop) {
    const going = { stop, wanting: new Set() };
    underWay.set(result, going);
    // Out of the cache first, so no one joins it while it fails.
    stop.signal.addEventListener("abort", () => cache.get(key) === result && cache.delete(key), { once: true });
    const done = () => keep(result, going);
    result.then(done, done);
    want(going, /** @type {AbortSignal} */ (signal));
  }
  if (cached) lastUsed(cache, key, result);
  else (behind.delete(key), request.catch((error) => stop?.signal.aborted || failed(error)), cache.set(key, result));
  prune(cache, most);
  return result;
}
/** @param {Promise<unknown>} load @param {{ wanting: Set<AbortSignal> }} going */
const keep = (load, going) => (underWay.delete(load), going.wanting.clear());
/** @param {{ stop: AbortController, wanting: Set<AbortSignal> }} going @param {AbortSignal} signal */
function want(going, signal) {
  if (signal.aborted) return;
  going.wanting.add(signal);
  signal.addEventListener("abort", () => going.wanting.delete(signal) && !going.wanting.size && going.stop.abort(), { once: true });
}
// Fine intervals, by chunk: each chart holds a span (`spans`), the latest two at first; the view reaching either end
// loads the next out. Older chunks are final, the latest two fetched again as blocks come.

/** @typedef {{ from: number, to: number, live: boolean, rest?: number, floor?: { at: number, of: string } }} Span the chunks a chart holds, `live` while they reach the latest; `rest`: no reaching out before then (nothing came from there); `floor`: the oldest with points for the series it was found with (those before it came empty: they start later) */
const chunkAt = (index, time) => Math.floor(indexAt(index, time) / CHUNK);
const latestChunk = (index) => chunkAt(index, Date.now() / 1000);
/** @type {Map<string, Promise<{ start: number, data: readonly any[], version: number }>>} */
const chunks = new Map();
const failedChunks = new Set();
function loadChunk(name, index, k, fresh, signal = /** @type {AbortSignal | undefined} */ (undefined)) {
  const key = `${name}@${index}#${k}`;
  return cachedLoad(chunks, key, fresh, KEPT_CHUNKS, async (cached, stop) => {
    if (k >= latestChunk(index) - 1) askedAt.set(key, liveHeight);
    const endpoint = client.seriesEndpoint(name, /** @type {"minute10"} */ (index));
    const joined = await refetchTail(endpoint, index, fresh && cached ? await cached.catch(() => null) : null, (k + 1) * CHUNK);
    const response = joined ?? (await endpoint.slice(k * CHUNK, (k + 1) * CHUNK).fetch({ signal: stop }));
    return { start: response.start, data: response.data, version: response.version };
  }, () => failedChunks.add(key), signal);
}
/** @type {Map<string, Span>} */
const spans = new Map();
/** @param {string} id */
const forgetSpans = (id) => {
  for (const key of spans.keys()) if (key.startsWith(`${id}@`)) spans.delete(key);
};
/** @returns {Span} */
function presetSpan(index, range) {
  const latest = latestChunk(index), told = rangeDays(range), days = told / INTERVAL_DAYS[index] > mostBars() ? 0 : told;
  return { from: clamp(days ? chunkAt(index, Date.now() / 1000 - days * 86400) : latest - 1, Math.max(0, latest - FIRST_CHUNKS + 1), Math.max(0, latest - 1)), to: latest, live: true };
}
/** @param {Chart} doc */
function spanOf(doc) {
  if (!FINE.has(doc.index)) return null;
  const key = spanKey(doc.id, doc.index), latest = latestChunk(doc.index);
  let span = spans.get(key);
  if (!span) {
    const left = openingView(doc);
    if (left) {
      const { start, end } = viewTimes(left), most = left.index === doc.index ? MOST_CHUNKS : FIRST_CHUNKS;
      const to = left.latest ? latest : clamp(chunkAt(doc.index, end), 0, latest), since = left.latest ? Math.max(0, Date.now() / 1000 - end) : 0;
      const from = clamp(chunkAt(doc.index, start + since), Math.max(0, to - most + 1), to);
      span = { from, to, live: to >= latest };
    } else span = presetSpan(doc.index, doc.range);
    spans.set(key, span);
  }
  if (span.live && span.to < latest) {
    span.to = latest;
    span.from = Math.max(span.from, latest - MOST_CHUNKS + 1);
  }
  return span;
}
const spanned = new Map();
function loadSpan(name, index, { from, to, live }, fresh, signal = /** @type {AbortSignal | undefined} */ (undefined), again = false) {
  const latest = latestChunk(index);
  const parts = Array.from({ length: to - from + 1 }, (_, i) => loadChunk(name, index, from + i, fresh && from + i >= latest - 1, signal));
  const key = `${name}@${index}`, kept = spanned.get(key);
  if (kept && kept.parts.length === parts.length && kept.parts.every((part, i) => part === parts[i])) return kept.value;
  // Chunks that failed are left out at either end (a live span's newest must load); the newest run between is kept.
  const value = Promise.allSettled(parts).then((settled) => {
    let end = settled.length - 1;
    while (!live && end > 0 && settled[end].status === "rejected") end--;
    const newest = settled[end];
    if (newest.status === "rejected") throw newest.reason;
    let first = end;
    while (first > 0 && settled[first - 1].status === "fulfilled") first--;
    const loaded = settled.slice(first, end + 1).map((part) => /** @type {PromiseFulfilledResult<{ start: number, data: any[], version: number }>} */ (part).value).filter((part) => part.data.length);
    // Chunks of two computations of the series: all of it again, once; still two (a cache can go on serving the older
        // for a while), an error to retry, never one series of both.
    if (new Set(loaded.map((part) => part.version)).size > 1) {
      if (again) throw Object.assign(new Error("Being recomputed"), { recomputing: true });
      forget(name, index);
      return loadSpan(name, index, { from, to, live }, false, signal, true);
    }
    const start = loaded[0]?.start ?? (from + first) * CHUNK;
    const data = [];
    for (const part of loaded) {
      while (data.length < part.start - start) data.push(null);
      for (const value of part.data) data.push(value);
    }
    const begin = timeAt(index, start), step = FINE_STEP[/** @type {keyof typeof FINE_STEP} */ (index)], times = new Array(data.length);
    for (let i = 0; i < data.length; i++) times[i] = begin + i * step;
    return { start, data, times };
  });
  const held = holder();
  if (held(key)) spanned.set(key, { parts, value });
  for (const other of spanned.keys()) if (!held(other)) spanned.delete(other);
  return value;
}
/** @param {Chart} doc */
function asked(name, doc) {
  const span = spanOf(doc);
  if (!span) return loads.has(`${name}@${doc.index}`);
  for (let k = span.from; k <= span.to; k++) if (!chunks.has(`${name}@${doc.index}#${k}`)) return false;
  return true;
}
function forget(name, index) {
  loads.delete(`${name}@${index}`);
  spanned.delete(`${name}@${index}`);
  for (const key of [...chunks.keys()]) if (key.startsWith(`${name}@${index}#`)) chunks.delete(key);
}
const failure = (error, index) =>
  error?.unavailable ? `No ${INTERVAL_LABEL[index] ?? index} data`
  : error?.status === 404 ? "Not on Bitview"
  : error?.name === "TimeoutError" || error?.name === "AbortError" ? "Timed out"
  : error?.recomputing ? "Bitview is recomputing it"
  : "Couldn't load";
const datedIntervals = (meta) => INTERVAL_ORDER.filter((id) => meta?.indexes?.includes(id));
const DAILY = (/** @type {string} */ name) => ((unit) => ["24h", "sum_24h", "avg_24h"].flatMap((end) => [`${name}_${end}`, ...(unit ? [`${unit[1]}_${end}_${unit[2]}`] : [])]))(/^(.*)_(usd|btc)$/.exec(name));
const BY_BLOCK = "Only by block (Studio charts by date)";
const TEXT_VALUES = "Its values are text (a date, a hash, an address), not numbers to chart";
/** @param {{ data: any[] }} data */
const isText = (data) => ((value) => typeof value === "string" || (typeof value === "object" && value !== null && !Array.isArray(value)))(data.data.find((value) => value !== null && value !== undefined));
const dailyOf = (/** @type {string} */ name) => (indexCatalog(), DAILY(name).find((other) => catalogIndex?.has(other) && !byBlock.has(other)) ?? null);

/**
 * @typedef {{ values?: Vector, measurement?: Vector, opens?: Vector, highs?: Vector, lows?: Vector, low?: Vector, members?: Vector[], memberIds?: string[], highest?: Int8Array, breaks?: Uint8Array, sides?: Uint8Array, unit: string, error?: string, detail?: string, info?: any, unavailable?: boolean, missing?: boolean, blank?: Vector, heat?: true, closes?: Vector }} Result
 *   `opens`, `highs`, `lows`: an OHLC series' candles, column by column (their closes its values); `low`: a cloud's lows (its highs, its values); `members`: its layers' values (its bands), `memberIds` their layers, `highest` which is highest; `breaks`: a trend's
 *   flips, a formula's restarts (each the point before: see restarts); `sides`: which it's on, a trend's (see trendOf), or which of a cloud's two is over (see cloudOf).
 * `error` as short as says what to do (the legend, a row), `detail` the sentence why (the editor, a preview, a tip)
 * @typedef {{ times: number[], results: Map<string, Result>, drawn: Set<string>, failed: string[], key?: string, version?: number, live?: boolean }} Resolved `failed`: the series whose fetch failed (to ask again); a draw's (see draw): `key`, the `dataKey` it was resolved for, `version`, the draw, `live`, whether it reaches the latest bar (a fine interval's chunks may end in the past)
 */

/** @param {Chart} doc */
const formulaSeries = (doc) => [...new Set(doc.layers.flatMap((layer) => (layer.formula !== undefined ? [...seriesNamed(layer.formula, "reads"), ...seriesUnnamed(layer.formula)] : [])))];
/** @param {Chart} doc @returns {string[]} */
const seriesIn = (doc) => [...new Set([...doc.layers.flatMap((layer) => (layer.series ? [layer.series] : heatLike(layer) ? ["price"] : [])), ...formulaSeries(doc)])];

/**
 * Hidden inputs still feed visible results.
 * @param {Chart} doc
 */
function drawLayers(doc) {
  if (doc.layers.every((layer) => !layer.hidden)) return doc.layers;
  const byId = new Map(doc.layers.map((layer) => [layer.id, layer])), byLetter = new Map(doc.layers.map((layer) => [layer.letter, layer]));
  const needed = new Set();
  let timeline = false;
  const visit = (/** @type {Layer | undefined} */ layer) => {
    if (!layer || needed.has(layer)) return;
    needed.add(layer);
    if (layer.series || heatLike(layer)) timeline = true;
    if (layer.formula !== undefined) {
      try {
        const node = parse(layer.formula);
        for (const letter of refsOf(node)) visit(byLetter.get(letter));
        if (!timeline && (refsOf(node, "series").size || seriesUnnamed(layer.formula).length)) timeline = true;
      } catch {
        // The formula evaluator reports invalid expressions without stopping other visible layers.
      }
    }
    for (const id of layer.from ?? []) visit(byId.get(id));
    if (layer.ref) visit(byId.get(layer.ref));
    if (layer.colorFrom) visit(byId.get(layer.colorFrom));
  };
  doc.layers.filter((layer) => !layer.hidden).forEach(visit);
  // A constant/date-only formula still needs the chart's clock, even when its source is hidden.
  if (needed.size && !timeline) visit(doc.layers.find((layer) => layer.series || heatLike(layer)) ?? doc.layers.find((layer) => layer.formula !== undefined && (seriesNamed(layer.formula, "reads").length || seriesUnnamed(layer.formula).length)));
  return doc.layers.filter((layer) => needed.has(layer));
}
/** @param {Chart} doc */
const drawSeries = (doc) => seriesIn({ ...doc, layers: drawLayers(doc) });

/** @type {Map<string, any>} */
const frames = new Map();
let frameCount = 0;
/** @type {WeakMap<object, { first: number, out: Result }>} */
const trims = new WeakMap();
const vectorIds = new WeakMap();
let vectorCount = 0;
const idOf = (/** @type {object} */ vector) => vectorIds.get(vector) ?? (vectorIds.set(vector, ++vectorCount), vectorCount);
/**
 * Charts on the same points share them, and so what's computed from them.
 * @type {Map<string, WeakMap<object, any>>}
 */
const alignedSets = new Map();
const alignedOn = (/** @type {number} */ lo, /** @type {number[]} */ times) => {
  const key = `${lo}|${times}`, aligned = lastUsed(alignedSets, key, alignedSets.get(key) ?? new WeakMap());
  if (alignedSets.size > 16) alignedSets.delete(/** @type {string} */ (alignedSets.keys().next().value));
  return aligned;
};
/** @param {Chart} doc @returns {Promise<Resolved>} */
async function resolve(doc, fresh = false, signal = /** @type {AbortSignal | undefined} */ (undefined)) {
  const layers = drawLayers(doc), drawing = { ...doc, layers };
  const sources = /** @type {(Layer & { series: string })[]} */ (layers.filter((layer) => layer.series));
  const named = formulaSeries(drawing);
  if (fresh) for (const name of seriesIn(drawing)) retryInfo(name);
  /** @param {string} name @returns {Promise<any>} */
  const fetchSeries = async (name) => {
    // Not in the client's catalog: not on Bitview, said here (a name half typed in a formula is never asked for). Until
        // the catalog is indexed, Bitview is asked.
    if (catalogIndex && !catalogIndex.has(name)) return { error: { status: 404 } };
    const lacking = intervalsOf(name)?.includes(doc.index) === false;
    const [about, series] = await Promise.allSettled([info(name, signal), lacking ? Promise.resolve(null) : load(name, doc.index, fresh, spanOf(doc), signal)]);
    if (about.status === "rejected") return { error: about.reason };
    const meta = about.value;
    units.set(name, unitOf(meta.type));
    if (!meta.indexes.includes(doc.index)) return { meta, error: { unavailable: true } };
    if (series.status === "rejected") return { meta, error: series.reason };
    return { meta, data: series.value };
  };
  const fetched = new Map(await Promise.all(seriesIn(drawing).map(async (name) => /** @type {[string, any]} */ ([name, await fetchSeries(name)]))));
  // Kept from the last resolve of this chart that read the same data: its series keep their identity, and with them
    // what's computed from them, so an edit to one layer redoes that layer only.
  const datas = [...fetched.values()].flatMap(({ data }) => (data ? [data] : []));
  const frameKey = `${doc.id}@${doc.index}`;
  let frame = frames.has(frameKey) ? lastUsed(frames, frameKey, frames.get(frameKey)) : undefined;
  // A series added whose every point is on the timeline already keeps the timeline, and with it the others' series:
    // only the new one is set.
  if (frame && frame.datas.every((data) => datas.includes(data))) {
    const { lo, position } = frame;
    const fits = (data) => data.start >= lo && data.start + data.times.length <= lo + position.length && position.subarray(data.start - lo, data.start - lo + data.times.length).every((at) => at >= 0);
    const added = datas.filter((data) => !frame?.datas.includes(data));
    if (added.length && added.every(fits)) {
      frame.datas = [...frame.datas, ...added];
    }
  }
  if (!frame || frame.datas.length !== datas.length || !datas.every((data) => frame?.datas.includes(data))) {
    // In flat arrays over the span of indexes: a Map per point is many times slower.
    let lo = Infinity, hi = -Infinity;
    for (const data of datas) {
      lo = Math.min(lo, data.start);
      hi = Math.max(hi, data.start + data.times.length);
    }
    const timeOf = new Float64Array(Math.max(0, hi - lo)).fill(NaN), position = new Int32Array(timeOf.length).fill(-1);
    for (const data of datas) for (let k = 0; k < data.times.length; k++) timeOf[data.start - lo + k] = data.times[k];
    /** @type {number[]} */
    const times = [];
    for (let i = 0; i < timeOf.length; i++) if (!Number.isNaN(timeOf[i])) (position[i] = times.length), times.push(timeOf[i]);
    frame = { id: ++frameCount, datas, lo, times, position, aligned: FINE.has(doc.index) ? new WeakMap() : alignedOn(lo, times), series: new Map(), computed: new Map(), trimmed: null };
    frames.set(frameKey, frame);
    prune(frames, 8, (key) => Boolean(chart) && key.startsWith(`${chart.id}@`));
  }
  const { lo, times, position } = frame, length = times.length;
  for (const id of frame.computed.keys()) if (!doc.layers.some((layer) => layer.id === id)) frame.computed.delete(id);
  /** @type {Map<string, Result>} */
  const results = new Map();
  const aligned = frame.aligned;
  const align = (data) => {
    if (aligned.has(data)) return aligned.get(data);
    const values = new Float64Array(length).fill(NaN);
    /** @type {{ opens: Float64Array, highs: Float64Array, lows: Float64Array } | undefined} */
    let candles;
    for (let k = 0; k < data.data.length; k++) {
      const value = data.data[k], p = position[data.start - lo + k];
      // A number, as nearly all are, taken as it is: no call per value, thousands a series.
      if (typeof value === "number") values[p] = value;
      else if (Array.isArray(value)) {
        candles ??= { opens: new Float64Array(length).fill(NaN), highs: new Float64Array(length).fill(NaN), lows: new Float64Array(length).fill(NaN) };
        candles.opens[p] = Number(value[0]);
        candles.highs[p] = Number(value[1]);
        candles.lows[p] = Number(value[2]);
        values[p] = Number(value[3]);
      } else if (value !== null && value !== undefined) values[p] = Number(value);
    }
    const columns = { values, ...candles };
    aligned.set(data, columns);
    return columns;
  };
  for (const layer of sources) {
    const { meta, data, error } = fetched.get(layer.series);
    if (error) {
      const blocked = Boolean(error?.unavailable && meta && !datedIntervals(meta).length);
      results.set(layer.id, { unit: units.get(layer.series) ?? "number", error: blocked ? "By block only" : failure(error, doc.index), ...(blocked ? { detail: BY_BLOCK } : {}), info: meta, unavailable: Boolean(error?.unavailable), missing: error?.status === 404 });
      continue;
    }
    if (isText(data)) {
      results.set(layer.id, { unit: "number", error: "Text, not numbers", detail: TEXT_VALUES, info: meta });
      continue;
    }
    const before = frame.series.get(layer.series);
    if (before?.info === meta) results.set(layer.id, before);
    else {
      const result = { ...align(data), unit: unitOf(meta.type), info: meta };
      frame.series.set(layer.series, result);
      results.set(layer.id, result);
    }
  }
  const closes = ((price) => (price?.data && !isText(price.data) ? align(price.data).values : undefined))(fetched.get("price"));
  // Waited for after the data, not before: the groups come with the whole client.
  if (layers.some(heatLike)) await import("/studio/client.js");
  for (const layer of layers) if (heatLike(layer)) results.set(layer.id, { unit: measureUnit(measureOf(layer)), heat: true, closes, ...(heatKnown(layer) ? {} : { error: "Not on Bitview", detail: "Bitview has no holder group by this name: renamed, removed, or mistyped" }) });
  /** @type {Map<string, Result>} */
  const bySeries = new Map();
  for (const name of named) {
    const { meta, data, error } = fetched.get(name);
    if (error) bySeries.set(name, { unit: units.get(name) ?? "number", error: error?.unavailable && meta && !datedIntervals(meta).length ? "By block only" : failure(error, doc.index) });
    else if (isText(data)) bySeries.set(name, { unit: "number", error: "Text, not numbers" });
    else {
      const { values } = align(data);
      bySeries.set(name, { values, unit: unitOf(meta.type) });
    }
  }
  const quiet = (text) => (/^([A-Z]{1,2}\b|Bitview)/.test(text) ? text : `${text.charAt(0).toLowerCase()}${text.slice(1)}`);
  // Formulas, in dependency order; a loop is an error, not a hang.
  const byLetter = new Map(doc.layers.map((layer) => [layer.letter, layer]));
  const visiting = new Set(), LOOP = "Formulas refer to each other in a loop";
  /** @param {Layer} layer @param {() => Result} make */
  const once = (layer, make) => {
    if (results.has(layer.id)) return results.get(layer.id);
    if (visiting.has(layer.id)) throw new Error(LOOP);
    visiting.add(layer.id);
    /** @type {Result} */
    let result;
    try {
      result = make();
    } catch (thrown) {
      const error = /** @type {Error & { short?: string }} */ (thrown);
      result = error.short ? { unit: "number", error: error.short, detail: error.message } : { unit: "number", error: error.message };
    } finally {
      visiting.delete(layer.id);
    }
    results.set(layer.id, result);
    return result;
  };
  /** @param {Layer} layer @param {string} key @param {() => Result} make */
  const kept = (layer, key, make) => {
    const cached = frame.computed.get(layer.id);
    if (cached?.key === key) return cached.result;
    const result = make();
    frame.computed.set(layer.id, { key, result });
    return result;
  };
  const compute = (/** @type {Layer} */ layer) => once(layer, () => {
    const node = parse(layer.formula ?? "");
    const inputs = [...refsOf(node)].map((letter) => {
      const target = byLetter.get(letter);
      const result = !target || target === layer || target.type === "cloud" ? null : computed(target);
      return result?.values ? idOf(result.values) : result?.blank ? idOf(result.blank) : result?.error ?? "-";
    });
    const key = [frame.id, layer.formula, doc.index, ...inputs, ...[...refsOf(node, "reads")].map((name) => (bySeries.get(name)?.values ? idOf(bySeries.get(name)?.values) : "-"))].join("|");
    return kept(layer, key, () => {
      const value = evaluate(node, length, (name, kind) => {
        if (kind === "date") return timelineOf(times);
        if (kind === "series") {
          const input = bySeries.get(name);
          if (!input?.values) throw new Error(`${name}: ${input?.error ? quiet(input.error) : "no data"}`);
          return input.values;
        }
        const target = byLetter.get(name);
        if (!target) throw new Error(goneLayer(doc, name) ?? `There is no layer ${name}`);
        if (target === layer) throw new Error(`${name} can't refer to itself`);
        if (target.type === "cloud") throw new Error(`${name} is a cloud: a range, not one value`);
        if (heatLike(target)) throw new Error(`${name} is a heatmap: many values a date, not one`);
        const result = computed(target);
        if (result?.blank) return result.blank;
        if (!result?.values) throw result?.error ? failing(`${name} has no data (${quiet(result.detail ?? result.error)})`, `${name} has no data`) : new Error(`${name} has no data`);
        return result.values;
      }, doc.index);
      if (!length) throw new Error("Name a series, or add one, to give the formula a timeline");
      const values = typeof value === "number" ? new Float64Array(length).fill(value) : value;
      const unit = unitOfNode(node, (name, kind) => {
        if (kind === "series") return bySeries.get(name)?.unit ?? null;
        const target = byLetter.get(name);
        return !target || target.type === "cloud" ? null : computed(target)?.unit ?? null;
      }) ?? "number";
      // No value anywhere with all of history loaded: said rather than a blank layer; its none-at-each-date is kept
            // apart (`blank`) for a formula reading it alone (an "or" of it decides by its other side).
      const blanks = [...refsOf(node)].filter((letter) => ((target) => target && target !== layer && target.type !== "cloud" && computed(target)?.blank)(byLetter.get(letter)));
      if (!FINE.has(doc.index) && !values.some(Number.isFinite)) return { blank: new Float64Array(values.length).fill(NaN), unit, ...(blanks.length ? { error: `${blanks[0]} has no data`, detail: `${blanks.join(", ")} ${blanks.length > 1 ? "have" : "has"} no value at any date` } : unstarted.has(values) ? { error: "Not from its start", detail: "What it reads begins later in this period, so it can't be read from 1× at the period's start" } : { error: "No values", detail: "It has no value at any date: what it reads has none where it's computed (a log of 0 or less, a root of a negative, a division by 0), or a when() holds nowhere" }) };
      const breaks = ((marks) => marks && breaksAt(values, marks))(restarts.get(values));
      return { values, unit, ...(breaks ? { breaks } : {}) };
    });
  });
  const computed = (/** @type {Layer} */ layer) => (layer.formula !== undefined ? compute(layer) : layer.from ? made(layer) : results.get(layer.id));
  const made = (/** @type {Layer} */ layer) => once(layer, () => {
    const all = (layer.from ?? []).flatMap((id) => ((member) => (member ? [computed(member)] : []))(doc.layers.find((item) => item.id === id)));
    if (all.some((result) => result?.error === LOOP)) throw new Error(LOOP);
    const inputs = all.filter((result) => result?.values), edges = layer.type === "cloud" ? edgesOf(layer).length : 0;
    const inputIds = (layer.from ?? []).filter((id) => doc.layers.some((item) => item.id === id)).filter((_, k) => all[k]?.values);
    if (inputs.length + edges < 2) return { unit: inputs[0]?.unit ?? "number", error: fromShort(layer, doc) || (edges ? "Needs a layer with data" : "Needs two layers with data") };
    const ref = layer.type === "trend" ? doc.layers.find((item) => item.id === layer.ref) : null, by = ref ? computed(ref) : null;
    if (by?.error === LOOP) throw new Error(LOOP);
    if (layer.type === "trend" && !by?.values) return { unit: inputs[0].unit, error: !ref ? "Pick what flips it" : `${ref.letter} has no data` };
    const key = [frame.id, layer.type, edgesOf(layer).join(), inputIds.join(), cloudLayers(layer).length, ...inputs.map((input) => idOf(/** @type {Vector} */ (input.values))), by?.values ? idOf(by.values) : "", by?.lows ? idOf(by.lows) : ""].join("|");
    return kept(layer, key, () => {
      if (layer.type === "cloud" && edges && inputs.length < 2) {
        const values = inputs[0]?.values ?? new Float64Array(length).fill(NaN);
        return { values, low: values, ...(inputs.length ? { members: [values], memberIds: inputIds } : {}), unit: inputs[0]?.unit ?? "number" };
      }
      if (layer.type === "cloud") return { ...cloudOf(inputs.map((input) => /** @type {Vector} */ (input.values)), edgesOf(layer).includes("top"), edgesOf(layer).includes("bottom"), cloudLayers(layer).length === 2), memberIds: inputIds, unit: inputs[0].unit };
      const low = new Float64Array(length).fill(NaN), high = new Float64Array(length).fill(NaN), count = new Uint8Array(length);
      for (const { values } of inputs) {
        for (let i = 0; i < length; i++) {
          const value = /** @type {Vector} */ (values)[i];
          if (!Number.isFinite(value)) continue;
          count[i]++;
          // NaN, where none was yet, compares false.
          if (!(value >= low[i])) low[i] = value;
          if (!(value <= high[i])) high[i] = value;
        }
      }
      // Where fewer than two have a value, none: a range of one value isn't one.
      for (let i = 0; i < length; i++) if (count[i] < 2) low[i] = high[i] = NaN;
      const trend = trendOf(low, high, /** @type {Result} */ (by));
      return { values: trend.values, breaks: trend.breaks, sides: trend.sides, unit: inputs[0].unit };
    });
  });
  layers.filter((layer) => layer.formula !== undefined || layer.from).forEach(computed);
  // Hidden rows keep their metadata and catalog errors without asking for their values.
  for (const layer of doc.layers) if (!results.has(layer.id) && layer.series) {
    const name = layer.series;
    results.set(layer.id, { unit: units.get(name) ?? "number", info: known.get(name), ...(catalogIndex && !catalogIndex.has(name) ? { error: "Not on Bitview", missing: true } : {}) });
  }
  // Formulas read raw measurements; a pane's average changes only the resolved view, never its inputs.
  for (const pane of doc.panes) {
    if (!pane.average) continue;
    const average = averageFor(doc, pane);
    for (const layer of layers.filter((layer) => layer.pane === pane.id)) {
      const result = results.get(layer.id);
      if (result?.values) results.set(layer.id, averageResult(result, average, doc.index, layer));
    }
  }
  let first = length;
  for (const { values } of results.values()) {
    if (!values) continue;
    for (let i = 0; i < first; i++) if (Number.isFinite(values[i]) && values[i] !== 0) {
      first = i;
      break;
    }
  }
  const failed = [...fetched].filter(([, { error }]) => error && !error.unavailable && error.status !== 404).map(([name]) => name);
  if (first === 0 || first === length) return { times, results, drawn: new Set(layers.map(layer => layer.id)), failed };
  if (frame.trimmed?.first !== first) frame.trimmed = { first, times: times.slice(first) };
  for (const [id, result] of results) {
    const trimmed = trims.get(result);
    if (trimmed && trimmed.first === first) {
      results.set(id, trimmed.out);
      continue;
    }
    const out = mapResult(result, (values) => values.slice(first));
    trims.set(result, { first, out });
    results.set(id, out);
  }
  return { times: frame.trimmed.times, results, drawn: new Set(layers.map(layer => layer.id)), failed };
}

/** @param {Result} result @param {(values: any, field: string) => any} map @returns {Result} */
function mapResult(result, map) {
  const out = { ...result };
  for (const [field, values] of Object.entries(result)) if (ArrayBuffer.isView(values)) out[field] = map(values, field);
  if (result.members) out.members = result.members.map((values) => map(values, "members"));
  return out;
}

const averagedResults = new WeakMap();
/**
 * @param {Result} result
 * @param {ReturnType<typeof averageFor>} average
 * @param {string} index
 * @param {Layer} layer
 * @returns {Result}
 */
function averageResult(result, average, index, layer) {
  const { points, span } = average, partial = !FINE.has(index);
  if (!average.active || !result.values || result.error) return result;
  const key = `${span}:${index}:${result.members ? `${edgesOf(layer).join()}:${cloudLayers(layer).length}` : ""}`, kept = averagedResults.get(result) ?? new Map();
  if (kept.has(key)) return kept.get(key);
  /** @type {Result} */
  let out;
  try {
    const mean = (/** @type {Vector} */ values) => fromStart(values, (from) => rolling(from, points, partial).mean);
    const members = result.members?.map(mean);
    const shape = members ? cloudOf(members, edgesOf(layer).includes("top"), edgesOf(layer).includes("bottom"), cloudLayers(layer).length === 2)
      : { values: mean(result.values), ...(result.low ? { low: mean(result.low) } : {}), ...(result.opens ? { opens: mean(result.opens), highs: mean(/** @type {Vector} */ (result.highs)), lows: mean(/** @type {Vector} */ (result.lows)) } : {}) };
    if (!partial && !shape.values.some(Number.isFinite) && outreaches(result.values, points)) throw windowTooLong("Average", span, spanDays(span), index, points > result.values.length, false);
    // Rolling outputs may be shared with formula results: never write to them.
    out = { ...result, ...shape, measurement: result.values };
  } catch (thrown) {
    const problem = /** @type {Error & { short?: string }} */ (thrown);
    out = { unit: result.unit, info: result.info, measurement: result.values, error: problem.short ?? problem.message, detail: problem.message };
  }
  kept.set(key, out);
  averagedResults.set(result, kept);
  return out;
}

const hexCache = new Map();
const ink = (color) => {
  const page = pageInk(color);
  if (page) return css(page);
  if (!ofPalette(color)) return color;
  if (hexCache.size > 500) hexCache.clear();
  if (!hexCache.has(color)) hexCache.set(color, oklchToHex(...shownOklch(color)));
  return hexCache.get(color);
};
/** @returns {[number, number, number]} */
const oklchOf = (color) => OKLCH[color] ?? (color.startsWith("oklch(") ? /** @type {[number, number, number]} */ (color.slice(6, -1).split(" ").map(Number)) : hexToOklch(pageInk(color) ? ink(color) : color));
const cssColor = (color) => ((page) => (page ? `var(${page})` : ink(color)))(pageInk(color));
const alpha = (color, a) => `${color}${Math.round(a * 255).toString(16).padStart(2, "0")}`;
const unitFor = (layer, result) => (layer.type === "signal" ? "signal" : layer.format ? layer.format : result?.unit ?? (layer.series ? units.get(layer.series) : "number") ?? "number");
/** @param {Layer} layer */
const baseOf = (layer) => layer.base ?? 0;

const PROFILE_BIN = 3;
const PROFILE_DIM = 0.3;
/** @param {Chart} doc */
const profiled = (doc) => doc.layers.some((layer) => layer.type === "profile" && !layer.hidden);
const reserveOf = (/** @type {number} */ width, /** @type {Chart | null | undefined} */ doc) => (doc && profiled(doc) ? columnWidth(width) : 0);
const rightGap = (width = engine.time.width, doc = displayed?.doc ?? chart) => rightBlank(width) + reserveOf(width, doc);
function setReserve(/** @type {number} */ width = engine.time.width) {
  const reserve = reserveOf(width, displayed?.doc ?? chart);
  if (reserve !== engine.time.reserve) (engine.time.reserve = reserve), engine.invalidate(true);
}
const MIN_BAR_SPACING = 0.25;
function fitSpacing(/** @type {Chart} */ doc, /** @type {number} */ bars, /** @type {{ from: number, to: number } | null} */ held = displayed?.doc.id === doc.id && displayed.doc.index === doc.index ? shownSpan() : null) {
  // By the chart's own width, its price scales at their widest: not the plot's, which varies with a scale's labels and
    // would set it again as the view moves; a spacing set again is applied a frame later and undoes a zoom made since.
  const sides = (engine.axis ? 1 : 0) + (engine.panes.some((pane) => pane.left) ? 1 : 0);
  const width = engine.time.width ? chartWidth() - sides * axisWidth() : 0;
  const fitted = FINE.has(doc.index) || width <= 30 + rightGap(width) || bars < 2 ? MIN_BAR_SPACING : Math.max(0.005, Math.min(MIN_BAR_SPACING, (0.995 * (width - 30 - rightGap(width))) / (bars - 1)));
  // Never closer than the view's own bars: one kept at a narrower width (a layout's motion) isn't squeezed. `held`: a
    // view about to be shown, its own.
  const range = held, spacing = Math.min(fitted, range ? engine.time.width / (range.to - range.from) : Infinity);
  if (spacing !== engine.time.minSpacing) (engine.time.minSpacing = spacing), engine.invalidate(true);
}
/** Over several years the time axis names years only: a lone month at the edge ("Sep") reads as noise. */
const yearsOnly = (/** @type {number} */ _time, /** @type {string} */ kind) => (kind === "year" ? null : "");
let axisNamed = "";
function nameAxis() {
  const span = shownDates();
  const by = span && Number(span.to) - Number(span.from) > 3 * 365.25 * 86400 ? "years" : "dates";
  if (by === axisNamed) return;
  axisNamed = by;
  engine.timeAxis.format = by === "years" ? yearsOnly : null;
  engine.invalidate(true);
}
function axisWidth() {
  const context = /** @type {CanvasRenderingContext2D} */ (document.createElement("canvas").getContext("2d"));
  context.font = `${CHART_TEXT}px ${chartSans}`;
  return Math.ceil((Math.max(context.measureText("0".repeat(AXIS_CHARS)).width, context.measureText(`−${PRECISE}%`).width) + 16) / 4) * 4;
}
function chartTheme() {
  return { background: chartGround(), text: css("--chart-text"), grid: css("--chart-grid"), crosshair: css("--crosshair"), across: css("--crosshair-faint"), badge: css("--crosshair-label"), separator: css("--line"), active: css("--accent"), font: `${CHART_TEXT}px ${chartSans}` };
}
/**
 * The ground under the chart: its box's and its ancestors' backgrounds up to the first that isn't see-through, laid
 * over each other. Worked out, not drawn and read back: a browser resisting fingerprinting (Mullvad) gives a read pixel
 * noise.
 */
function chartGround() {
  /** @type {string[]} */
  const layers = [];
  for (let element = /** @type {HTMLElement | null} */ ($("chart")); element; element = element.parentElement) {
    const color = getComputedStyle(element).backgroundColor, alpha = channels(color)[3] / 255;
    if (!color || !alpha) continue;
    layers.push(color);
    if (alpha >= 1) break;
  }
  if (!layers.length) return DARK.background;
  const ground = channels(layers[layers.length - 1])[3] >= 255;
  if (layers.length === 1 && ground) return layers[0];
  const made = channels(ground ? layers[layers.length - 1] : DARK.background);
  for (let k = layers.length - (ground ? 2 : 1); k >= 0; k--) {
    const layer = channels(layers[k]), a = layer[3] / 255;
    for (let c = 0; c < 3; c++) made[c] = layer[c] * a + made[c] * (1 - a);
  }
  return `rgb(${Math.round(made[0])}, ${Math.round(made[1])}, ${Math.round(made[2])})`;
}
const VALUE_FORMAT = { value: (/** @type {number} */ value) => precise(value), ticks: (/** @type {number[]} */ values) => axisTicks(values) };
const SI_FORMATS = Object.fromEntries([...SI_UNITS].map((unit) => [unit, { value: (/** @type {number} */ value) => precise(value, "", unit), ticks: (/** @type {number[]} */ values) => axisTicks(values, "", unit) }]));
/** @param {string[]} units */
const valueFormatOf = (units) => (units.length && units.every((unit) => SI_UNITS.has(unit)) ? SI_FORMATS[units[0]] : VALUE_FORMAT);
const SHARE_FORMAT = { value: (/** @type {number} */ value) => (value > 100.5 || value < -0.5 ? "" : precise(value, "%")), ticks: (/** @type {number[]} */ values) => axisTicks(values, "%").map((label, i) => (values[i] > 100.5 || values[i] < -0.5 ? "" : label)) };
const PERCENT_FORMAT = { value: (/** @type {number} */ value) => precise(value, "%"), ticks: (/** @type {number[]} */ values) => axisTicks(values, "%") };
const NO_FORMAT = { value: () => "", ticks: (/** @type {number[]} */ values) => values.map(() => "") };
/** @type {import("/modules/bitview-charts/scale.js").ScaleMode[]} */
const SCALE_MODES = ["linear", "log", "percent", "indexed"];
const timelines = /** @type {WeakMap<number[], Float64Array>} */ (new WeakMap());
const timelineOf = (/** @type {number[]} */ times) => timelines.get(times) ?? ((line) => (timelines.set(times, line), line))(new Float64Array(times));
/** @param {number[]} times */
const showTimes = (times) => ((line) => engine.time.times !== line && engine.setTimes(line))(timelineOf(times));
const shownSpan = () => engine.controller.range;
const EXACT_RANGE = { exact: true };
/** @param {{ from: number, to: number }} span @param {{ exact?: boolean }} [options] */
const showSpan = (span, options) => engine.setRange(span.from, span.to, options);
const shownDates = () => {
  const times = engine.time.times, n = times.length, at = (/** @type {number} */ i) => times[clamp(i, 0, n - 1)];
  return n && engine.time.firstVisible <= engine.time.lastVisible ? { from: at(engine.time.firstVisible), to: at(engine.time.lastVisible) } : null;
};
/** @param {{ from: number, to: number }} span */
const showDates = (span) => {
  const times = engine.time.times, from = lowerBound(times, span.from), after = lowerBound(times, span.to);
  engine.setRange(from, times[after] === span.to ? after : after - 1, EXACT_RANGE);
};
/** @param {number} time */
/** @param {number} i */
const paneBox = (i) => {
  const pane = engine.panes[i];
  if (!pane) return null;
  const box = engine.element.getBoundingClientRect();
  return new DOMRect(box.left, box.top + pane.top, engine.left + pane.width, pane.height);
};
/** @type {readonly ["left", "right"]} */
const SIDES = ["left", "right"];
/** @param {number} i @param {"left" | "right"} side @param {{ from: number, to: number } | null} range */
const holdScale = (i, side, range) => (engine.panes[i]?.setRange(range?.from, range?.to, side), engine.invalidate(true));
/** @param {any} series @param {number} value */
const seriesY = (series, value) => ((y) => (Number.isFinite(y) ? y : null))(series ? engine.y(series.engine, value) : NaN);
/** @param {any} series @param {number} y */
const seriesValue = (series, y) => (series ? engine.value(series.engine, y) : NaN);
/** @param {any} series */
const seriesPane = (series) => engine.panes.findIndex((pane) => pane.series.includes(series?.engine));
/** @param {any} series */
const seriesScale = (series) => ((pane) => pane?.scaleOf(pane.series.indexOf(series.engine)))(engine.panes[seriesPane(series)]);
/** @type {Map<string, (value: number) => string>} */
const layerFormats = new Map();
/** @param {number} value @param {number} time @param {any} series */
const crossOn = (value, time, series) => {
  const times = engine.time.times, i = lowerBound(times, time), y = seriesY(series, value), pane = engine.panes.find((pane) => pane.series.includes(series?.engine));
  engine.setCrosshair(i < times.length ? i : -1, y === null || !pane ? NaN : pane.top + y);
};
const xOfTime = (time) => ((times, i) => (times[i] === time ? engine.time.x(i) : null))(engine.time.times, lowerBound(engine.time.times, time));
const fillBox = () => Object.assign(engine.element.style, { width: "100%", height: "100%" });
const sizeKeeping = (/** @type {number} */ width, /** @type {number} */ height) => {
  const was = (engine.render(true), shownSpan());
  sizeChart(width, height);
  keepStretch(was);
};
/** @type {{ legends: [HTMLElement, number, number][], to: number, sides: number, left: number, leftAxis: boolean, clipped: [HTMLElement, string][], room: number, lead: number, ranged: boolean } | null} */
let drawnStretch = null;
function stretchAcross(/** @type {number} */ run, /** @type {number} */ shown, /** @type {number} */ width) {
  const box = $("chart");
  const left = engine.left, right = engine.axisShown, element = engine.element;
  const clipped = /** @type {[HTMLElement, string][]} */ ([[element, element.style.overflow]]);
  for (const [item] of clipped) item.style.overflow = "visible";
  const ranged = presetShown && !engine.controller.target && Boolean(resolved?.times.length), at = (/** @type {number | undefined} */ time) => (ranged && time !== undefined ? xOfTime(time) : null);
  const first = at(resolved && chart ? resolved.times[Math.min(firstShown(chart, resolved), resolved.times.length - 1)] : undefined), last = at(resolved?.times.at(-1));
  drawnStretch = { legends: all$(".legend", element).map((legend) => /** @type {[HTMLElement, number, number]} */ ([legend, legend.offsetLeft, legend.offsetWidth])), to: width - left - right, sides: left + right, left, leftAxis: left > 0 && !(right > 0), clipped, room: last === null ? 0 : Math.max(0, width - left - right - last), lead: first !== null && first > 0.5 ? first : 0, ranged };
  stretchDrawn(shown);
  const frame = () => run === motionsSized && drawnStretch && (stretchDrawn(box.getBoundingClientRect().width), requestAnimationFrame(frame));
  requestAnimationFrame(frame);
}
const stretchNow = () => drawnStretch && stretchDrawn($("chart").getBoundingClientRect().width);
/**
 * By `translate`, apart from a stretch's `transform` (see stretchDrawn). It moves with what's under it (the card's
 * foot) rather than hiding under it or waiting at its end, set as the box is laid out (see resized): after layout, not
 * a frame behind.
 * @param {number} height
 */
function placeFoot(height) {
  const by = sizeHeld && footPinned ? height - chartSize.height : 0, moved = Math.abs(by) >= 0.5;
  for (const item of [engine.timeAxis.canvas, engine.corner]) Object.assign(item.style, { translate: moved ? `0 ${by}px` : "", background: moved ? engine.theme.background : "" });
}
/**
 * Scaled on the box's layout this frame: the browser's own animations of a transform can run a frame or two off the
 * box's.
 * @param {number | null} shown
 */
function stretchDrawn(shown) {
  const element = engine.element, canvases = [...engine.panes.flatMap((pane) => [pane.canvas, pane.overCanvas ?? []]).flat(), engine.timeAxis.canvas];
  const edged = [...all$(".legend", element), ...engine.panes.flatMap((pane) => (pane.leftCanvas ? [pane.leftCanvas] : []))];
  if (shown === null || !drawnStretch) {
    for (const [item, overflow] of drawnStretch?.clipped ?? []) item.style.overflow = overflow;
    drawnStretch = null;
    for (const item of [...canvases, ...edged]) Object.assign(item.style, { transform: "", transformOrigin: "" });
    for (const legend of all$(".legend", element)) legend.style.clipPath = "";
    return;
  }
  const { to, sides, left, leftAxis, room, lead, ranged } = drawnStretch, from = Math.max(1, shown - sides);
  const held = ranged && to - lead - room > 1, gap = held ? rightGap(from) : 0, k = held ? (from - lead - gap) / (to - lead - room) : from / to;
  for (const canvas of canvases) Object.assign(canvas.style, { transformOrigin: leftAxis ? "left" : "right", transform: leftAxis ? `translateX(${held ? lead * (1 - k) : 0}px) scaleX(${k})` : `translateX(${held ? k * room - gap : 0}px) scaleX(${k})` });
  if (!leftAxis) for (const item of left ? edged : all$(".legend", element)) item.style.transform = `translateX(${to - from}px)`;
  if (!leftAxis) for (const [legend, at, wide] of drawnStretch.legends) legend.style.clipPath = at + wide > from ? `inset(0 ${at + wide - from}px 0 0)` : "";
}
/** @type {Map<string, any>} */
const drawn = new Map();
/** A series' options without its format and scale (a restyle keeps both, and applying them again costs a pass over every point). */
/** @type {Map<string, { shape: string, shown?: Vector, opens?: Vector, highs?: Vector, lows?: Vector, stacked?: boolean, painted?: any[] | null, low?: Vector }>} */
const built = new Map();
/** @type {Map<string, { levels: Levels, pane: any }>} */
const levelsKept = new Map();
const PANE = { scale: { top: 0.17, bottom: 0.08 }, left: { top: 0.17, bottom: 0.08 } };
const levelTags = /** @type {{ line: import("/modules/bitview-charts/levels.js").Level, layers: Set<string> }[]} */ ([]);
/** Each set again only as that changes: from one lit layer to the next, two, not all. */
const fadedNow = new Set();
/** @param {string} layerId @param {boolean} faint */
function fadeLayer(layerId, faint, series = drawn) {
  const own = series.get(layerId);
  if (own) for (const line of [own.engine, own.edge]) if (line) line.faint = faint;
}
/** @type {Map<any, { marks: Marks, pane: any }>} */
const markers = new Map();
/** @type {Map<string, { series: any, width: number, radius: number }>} */
const dotted = new Map();
let dotSpacing = 1;
function dotRadius(width) {
  const base = dotSpacing >= 12 ? 3 : dotSpacing >= 6 ? 2 : dotSpacing >= 3 ? 1.5 : 1;
  const radius = clamp(base + (width - 1.5) / 2, 0.75, Math.max(base, dotSpacing / 2 - 1));
  return Math.round(radius * 4) / 4;
}
function fitDots() {
  const range = shownSpan();
  if (!range) return;
  dotSpacing = engine.time.width / Math.max(1, range.to - range.from);
  for (const dots of dotted.values()) {
    const radius = dotRadius(dots.width);
    if (radius !== dots.radius) dots.series.engine.setStyle({ dots: (dots.radius = radius) }), engine.invalidate(true);
  }
}
const layerOfSeries = (series) => [...drawn].find(([, other]) => other === series)?.[0] ?? null;
const respot = () => spotlight(pointedLayer ?? (selected && !pointerOnChart && panelShown() ? selected : null));
const cardsDrawn = new Map();
const noValues = (/** @type {string} */ id) => ((result) => (result?.values ? "" : result?.error ?? "Its data is still loading"))(resolved?.results.get(id));
/** The interval of the data on screen: during a switch the old one, its data being drawn till the new one's in. */
const shownIndex = () => displayed?.doc.index || chart?.index || "day1";
const fineDrawn = () => FINE.has(shownIndex());
/** @param {number} time @param {string} [index] */
const pointLabel = (time, index = shownIndex()) => ((at) => (index.startsWith("year") ? String(at.getUTCFullYear()) : index.startsWith("month") ? `${MONTHS[at.getUTCMonth()]} ${at.getUTCFullYear()}` : dateLabel(time, FINE.has(index))))(new Date(time * 1000));
/** @type {Set<() => void>} */
const drawnWaiters = new Set();
let presetShown = true;
const setPresetShown = (/** @type {boolean} */ shown) => {
  presetShown = shown;
  renderRanges();
};

const settled = () => chartShown() && !pending() && !buildAsked;
const dataKey = (/** @type {Chart} */ doc) => JSON.stringify([doc.id, doc.index, doc.panes.map(({ id, average }) => [id, average ?? ""]).sort(), doc.layers.map((layer) => [layer.id, layer.pane, layer.letter, layer.series, layer.formula, layer.from ? layer.type : null, layer.from ?? null, layer.ref ?? null, heatLike(layer) ? [layer.heatmap ?? "all", layer.profile ?? null, measureOf(layer)] : null]).sort()]);
const lookKey = (/** @type {Chart} */ doc) => JSON.stringify([tokensNow(), { ...doc, title: 0, range: 0, drawings: 0, updated: 0, folder: 0, preset: 0, base: 0, blank: 0, panes: doc.panes.map(({ size, ...pane }) => pane), layers: doc.layers.map(({ label, underName, levels, ...layer }) => ({ ...layer, levels: levels?.map(({ name, ...level }) => level) })) }]);

/** @param {Chart} doc @param {Resolved} data @param {Map<string, any>} drawn @param {any[]} old */
function markSignals(doc, data, drawn, old) {
  const marked = new Map();
  for (const layer of doc.layers) {
    const shade = layer.place === "shade", on = layer.type === "signal" && !layer.hidden && (!shade || layer.results) && layer.ref ? drawn.get(layer.ref) : null, values = data.results.get(layer.id)?.values;
    // Only where its layer has a value drawn: a mark in a gap would sit on another date.
    const target = doc.layers.find((item) => item.id === layer.ref), reference = data.results.get(layer.ref ?? ""), own = reference?.values, log = target ? modeOf(doc, target) === 1 : false;
    if (!on || !values || !own) continue;
    const place = layer.place ?? "above", [, , position] = MARK_PLACES.find(([id]) => id === place) ?? MARK_PLACES[0], op = opacityOf(layer);
    const [, , downPosition] = MARK_PLACES.find(([id]) => id === (place === "above" ? "below" : place === "below" ? "above" : place)) ?? MARK_PLACES[0];
    const color = op < 1 ? alpha(ink(layer.color), op) : ink(layer.color), down = alpha(ink(belowOf(layer)), op);
    const list = marked.get(on)?.list ?? marked.set(on, { list: [], pane: doc.panes.findIndex((pane) => pane.id === target?.pane), side: target?.side === "left" ? "left" : "right", room: doc.panes.find((pane) => pane.id === target?.pane)?.stack !== "percent" }).get(on).list;
    for (const change of changesOf(values)) {
      const i = change.at, result = layer.results ? stretchResult(change, reference) : undefined, text = result === undefined ? {} : { text: resultText(result), textColor: result >= 0 ? color : down, priority: Math.abs(Math.log1p(result * change.was)) };
      if (!Number.isFinite(own[i]) || (log && !(own[i] > 0))) continue;
      if (change.to > 0 && !shade) list.push({ at: i, position, color, ...text });
      else if (change.to < 0 && !shade) list.push({ at: i, position: downPosition, color: down, ...text });
      else if (result !== undefined) list.push({ at: i, position: "on", color: change.was > 0 ? color : down, ...text });
    }
  }
  for (const [series, made] of markers) {
    if (marked.has(series) && !old.includes(series)) continue;
    made.pane.remove(made.marks);
    markers.delete(series);
  }
  for (const [series, { list, pane, side, room }] of marked) {
    list.sort((a, b) => a.at - b.at);
    const made = markers.get(series) ?? ((on) => ((made) => (markers.set(series, made), on.add(made.marks, side), made))({ marks: new Marks({ series: series.engine, room }), pane: on }))(engine.panes[pane]);
    made.marks.room = room;
    made.marks.set(list);
  }
  engine.invalidate(true);
}
function build(doc, data, refit) {
  crossHeld = true;
  try {
    buildChart(doc, data, refit);
  } finally {
    crossHeld = false;
  }
  if (crossSet) holdCross();
}

let revealing = 0;
const maskPlots = (/** @type {string} */ mask) => engine.panes.forEach((pane) => [pane.canvas, pane.overCanvas].forEach((canvas) => canvas && (canvas.style.maskImage = canvas.style.webkitMaskImage = mask)));
let inReveal = /** @type {{ join: number, outward: -1 | 1 } | null} */ (null);
const endReveal = () => (revealing++, (inReveal = null), maskPlots(""));
function reveal(/** @type {number} */ join, /** @type {-1 | 1} */ outward) {
  const id = ++revealing, since = motionTime(), own = (inReveal = { join, outward });
  let duration = 0;
  const step = () => {
    if (id !== revealing) return;
    // Placed by the view's own window: the chart's coordinates lag a frame behind a new one.
    const range = shownSpan(), plot = engine.time.width;
    const at = range ? ((own.join - range.from) / (range.to - range.from)) * plot : NaN;
    if (!(at > 0 && at < plot)) return endReveal();
    const way = outward < 0 ? at : plot - at, soft = Math.max(24, way * 0.4);
    duration ||= MOTION.panel + ((MOTION.page - MOTION.panel) * way) / plot;
    const t = Math.min(1, since() / duration);
    if (t >= 1) return endReveal();
    const front = at + outward * ((way + soft) * wiped(t) - soft), shown = Math.min(1, (outward * (front - at) + soft) / soft);
    if (outward > 0) maskPlots(`linear-gradient(to right, #000 ${Math.max(at, front)}px, rgb(0 0 0 / ${shown}) ${Math.max(at, front)}px, transparent ${front + soft}px)`);
    else maskPlots(`linear-gradient(to right, transparent ${front - soft}px, rgb(0 0 0 / ${shown}) ${Math.min(at, front)}px, #000 ${Math.min(at, front)}px)`);
    requestAnimationFrame(step);
  };
  requestAnimationFrame(step);
}

const defaultSize = (i) => (i === 0 ? 3 : 1.4);
const layerShows = (/** @type {Chart} */ doc, /** @type {Layer} */ layer) => (layer.type === "signal" ? signalShown(layer, doc) : !layer.hidden);
const paneShows = (/** @type {Chart} */ doc, /** @type {string} */ id) => doc.layers.some((layer) => layer.pane === id && layerShows(doc, layer));
function hiddenPanes(/** @type {Chart} */ doc, empty = false) {
  const showing = doc.panes.filter((pane) => paneShows(doc, pane.id));
  return new Set(showing.length || empty ? doc.panes.filter((pane) => !showing.includes(pane)).map((pane) => pane.id) : []);
}
function paneFactors(/** @type {Chart} */ doc, picture = exporting) {
  const hidden = hiddenPanes(doc, picture);
  return doc.panes.map((pane, i) => (hidden.has(pane.id) ? 0 : pane.size ?? defaultSize(i)));
}
/**
 * The panes' sizes as last laid on them, laid on again only as what's meant changes or is asked again: sizes the panes
 * took otherwise (a separator dragged) stay. Put back each layout, a drag jumped back a frame at a time and was lost if
 * let go just then.
 */
let panesLaid = { doc: /** @type {Chart | null} */ (null), meant: "", panes: /** @type {any[]} */ ([]), factors: /** @type {number[]} */ ([]) };
function sizePanes(doc = lookDoc ?? chart, again = false, picture = exporting) {
  const panes = engine.panes;
  if (!doc || panes.length !== doc.panes.length) return;
  const factors = paneFactors(doc, picture), meant = `${doc.panes.map((pane) => pane.id).join()}|${factors.join()}`;
  if (doc === panesLaid.doc && meant === panesLaid.meant && panes.every((pane, i) => pane === panesLaid.panes[i]) && !again) return;
  panesLaid = { doc, meant, panes: [...panes], factors };
  if (!factors.some((factor, i) => Math.abs(panes[i].share - factor) > 1e-6)) return;
  factors.forEach((factor, i) => (panes[i].share = factor));
  engine.layout();
}
let firstOf = { data: /** @type {Resolved | null} */ (null), key: "", first: 0 };
function firstShown(doc, data) {
  const key = doc.layers.map((layer) => (layer.hidden ? "" : layer.id)).join();
  if (firstOf.data === data && firstOf.key === key) return firstOf.first;
  let first = data.times.length;
  for (const layer of doc.layers) {
    const values = layer.hidden ? null : data.results.get(layer.id)?.values;
    if (!values) continue;
    for (let i = 0; i < first; i++) if (Number.isFinite(values[i])) {
      first = i;
      break;
    }
  }
  firstOf = { data, key, first: first < data.times.length ? first : 0 };
  return firstOf.first;
}
function presetWindow(doc, data, most = mostBars(), room = 30, gap = rightGap()) {
  const n = data.times.length, days = rangeDays(doc.range);
  const first = days ? lowerBound(data.times, data.times[n - 1] - days * 86400) : firstShown(doc, data);
  const width = engine.time.width, lead = days ? 0 : room, inner = width - lead - gap;
  if (!(inner > 0)) return within({ from: first, to: n - 1 }, "to", most);
  const spacing = Math.max(inner / Math.max(1, n - 1 - first), width / most), to = n - 1 + gap / spacing;
  return { from: to - width / spacing, to };
}
function applyRange(doc, data, animate = false) {
  if (data.times.length < 2 || !engine.time.width) return engine.show(engine.time.length);
  const target = presetWindow(doc, data);
  if (animate) glide(target, doc.range);
  else showSpan(target);
}
/** @type {{ to: { from: number, to: number }, preset: string | null } | null} */
let glided = null;
function glide(target, preset = null) {
  target = within(target);
  target = ontoData(target) ?? target;
  glided = { to: target, preset };
  engine.controller.viewTo(target, reduced() ? 0 : MOTION.page);
}
function showsPreset() {
  if (glided?.preset != null && engine.controller.target === glided.to) return glided.preset === chart.range;
  // In bars, against the preset's own window: empty room past the data counts (a time range stops at the data,
  // and zoomed out past both ends it would read as "Max").
  const range = shownSpan();
  if (!range || !resolved || resolved.times.length < 2) return true;
  const target = presetWindow(chart, resolved), slack = (target.to - target.from) * 0.015 + 1;
  return Math.abs(range.from - target.from) <= slack && Math.abs(range.to - target.to) <= slack;
}
function zoom(factor) {
  const range = shownSpan();
  if (!range) return;
  const width = Math.min(mostBars(), Math.max(10, (range.to - range.from) * factor)), center = (range.from + range.to) / 2;
  const last = (resolved?.times.length ?? 0) - 1, to = last + (rightGap() * width) / Math.max(1, engine.time.width);
  glide(range.to >= last ? { from: to - width, to } : within({ from: center - width / 2, to: center + width / 2 }));
}
const autoScale = () => engine.panes.forEach((_, i) => SIDES.forEach((side) => holdScale(i, side, null)));
const presetFits = (/** @type {string} */ id) => id === "max" || rangeDays(id) / INTERVAL_DAYS[chart.index] <= mostBars();
const shownRange = () => (presetFits(chart.range) ? chart.range : "max");
function indexOfTime(time) {
  const times = resolved?.times ?? [], i = lowerBound(times, time);
  return times[i] === time ? i : -1;
}
function lastShown() {
  const n = resolved?.times.length ?? 0;
  const range = n ? shownSpan() : null;
  return range ? clamp(engine.time.lastVisible, 0, n - 1) : n - 1;
}
function firstInView() {
  const n = resolved?.times.length ?? 0;
  return n && engine.time.length ? clamp(engine.time.firstVisible, 0, n - 1) : 0;
}
/** @typedef {{ at: number, rest: boolean, cross?: { pane: number, y: number } | null }} ReadPoint */
/** @returns {ReadPoint} */
function readPoint() {
  const cross = exporting || capturing ? null : crossAt();
  return hoverTime === null ? { at: lastShown(), rest: true, cross } : { at: indexOfTime(hoverTime), rest: false, cross };
}
function valuePointOf(/** @type {Layer} */ layer, /** @type {{ at: number, rest: boolean }} */ { at, rest }) {
  const values = resolved?.results.get(layer.id)?.values;
  if (!values || at < 0) return -1;
  if (!rest) return at;
  const kept = restPoints.get(values);
  if (kept?.at === at && kept.revision === dataRevision) return kept.i;
  const i = lastIn(values, 0, at);
  restPoints.set(values, { at, revision: dataRevision, i });
  return i;
}
/** @type {WeakMap<object, { at: number, revision: number, i: number }>} */
const restPoints = new WeakMap();
function valueAt(layer, point) {
  const at = valuePointOf(layer, point);
  return at < 0 ? NaN : /** @type {Float64Array} */ (resolved?.results.get(layer.id)?.values)[at];
}
function asOf(/** @type {Layer} */ layer, /** @type {{ at: number, rest: boolean }} */ point) {
  const at = valuePointOf(layer, point), times = resolved?.times, values = resolved?.results.get(layer.id)?.values;
  if (at >= 0 && values && periodAmount(layer) && values.findLastIndex(Number.isFinite) === at) return "so far";
  return point.rest && at >= 0 && times && (at < point.at - 1 || at < firstInView()) ? pointLabel(times[at]) : "";
}
/**
 * Only an amount over each calendar period (a flow() of a running total) whose last period is still running: drawn
 * partial, read "so far". Not a level, a trailing window's sum (the server's day, week or month is a whole window
 * ending at its last block), or a running total.
 * @param {Layer} layer
 */
function periodAmount(layer) {
  if (typeof layer.formula !== "string") return false;
  try {
    const node = parse(layer.formula);
    return node.k === "call" && node.f === "flow";
  } catch {
    return false;
  }
}
function rangeText(/** @type {Layer} */ layer, /** @type {Vector} */ high, /** @type {Vector} */ low, /** @type {number} */ at, /** @type {string} */ unit, lines = false) {
  if (edgesOf(layer).length > 1 || !Number.isFinite(low[at]) || !Number.isFinite(high[at])) return "—";
  return layer.from?.includes("top") ? `≥ ${format(low[at], unit)}` : layer.from?.includes("bottom") ? `≤ ${format(high[at], unit)}` : lines ? `${format(high[at], unit)}\n${format(low[at], unit)}` : `${format(low[at], unit)} – ${format(high[at], unit)}`;
}
function rangeAt(/** @type {Layer} */ layer, /** @type {{ at: number, rest: boolean }} */ point, /** @type {string} */ unit, lines = false) {
  const result = resolved?.results.get(layer.id), high = result?.values, low = result?.low, at = valuePointOf(layer, point);
  return !high || !low || at < 0 ? "—" : rangeText(layer, high, low, at, unit, lines);
}
const legends = () => /** @type {HTMLElement[]} */ ([...$("chart").querySelectorAll(".legend:not([hidden])")]);
/** @type {WeakMap<HTMLElement, string>} */
const legendShown = new WeakMap();
/**
 * What an element was last given to hold, by its HTML as written (the page reads it back as another string: an SVG's
 * `<path/>` reads `<path></path>`), so the same again writes nothing.
 */
const htmlWritten = new WeakMap();
const writeHtml = (/** @type {Element} */ element, /** @type {string} */ html) => void (htmlWritten.get(element) !== html && (htmlWritten.set(element, html), (element.innerHTML = html)));

function placeFull() {
  const corner = engine.corner.offsetWidth >= 24 ? engine.corner : null, button = $("full");
  if (corner && button.parentElement !== corner) corner.append(button);
  button.hidden = !corner;
}
let roomPlaced = { id: "", on: false };
/** @param {{ from: number, to: number }} range @param {number} was */
function atLatest(range, was) {
  const width = engine.time.width, last = (resolved?.times.length ?? 0) - 1, span = range.to - range.from;
  if (!width || !was || last <= 0 || Math.abs(range.to - (last + (rightGap(was) * span) / was)) > 0.5) return null;
  const to = last + (rightGap(width) * span) / width;
  return { from: to - span, to };
}
function setGap() {
  const width = engine.time.width, gap = rightGap(width), was = engine.time.rightOffset;
  if (width) {
    const range = shownSpan(), last = (resolved?.times.length ?? 0) - 1, base = rightBlank(width), room = gap - base;
    setReserve(width);
    const span = range ? range.to - range.from : 0, end = (/** @type {number} */ px) => last + (px * span) / width;
    const on = room > 0, moves = !lookShown && roomPlaced.id === chart.id && on !== roomPlaced.on && range && last > 0 && range.to >= last - 0.5 && (Math.abs(range.to - end(base + (roomPlaced.on ? columnWidth(width) : 0))) <= 0.5 || range.to < end(gap));
    if (moves) showSpan({ from: end(gap) - span, to: end(gap) });
    else if (range && gap !== was) showSpan(range);
    if (!lookShown) roomPlaced = { id: chart.id, on };
  }
}
function bandsOf(/** @type {Pane} */ pane) {
  const layers = chart.layers.filter((layer) => layer.pane === pane.id && !layer.hidden && (!layer.from || !summed(pane)) && !placedOn(layer) && !heatLike(layer));
  return pane.reverse ? layers.reverse() : layers;
}
/**
 * The library names a series only near its line, not across its fill.
 * @param {ChartPoint} param
 */
function bandAt(param) {
  const point = param?.point, pane = point && param.pane !== undefined ? chart.panes[param.pane] : undefined;
  if (!point || !pane || !resolved || param.logical === undefined) return null;
  const at = Math.round(param.logical);
  if (!summed(pane)) {
    let near = null, off = PULL;
    const bands = bandsOf(pane), on = engine.panes[/** @type {number} */ (param.pane)];
    for (const layer of bands) {
      const line = layer.type === "cloud" ? null : dotLineOf(layer.id), stroke = line instanceof Line && !(line instanceof Bars);
      const d = stroke && on ? on.distanceOf(line, engine.time, at, point.x, point.y) : Infinity;
      if (d <= off) (near = layer.id), (off = d);
    }
    const under = param.layer;
    return near ?? (under && bands.some((layer) => layer.id === under) ? under : null);
  }
  const stacked = stackOf(chart, resolved);
  for (const layer of bandsOf(pane)) {
    // Each on its own scale: a stack's left and right sides are stacked apart.
    const price = seriesValue(drawn.get(layer.id), point.y), made = stacked.get(layer.id), top = made ? made.values[at] : NaN, foot = made ? made.foot[at] : NaN;
    if (typeof price === "number" && Number.isFinite(top) && Number.isFinite(foot) && price >= Math.min(foot, top) && price <= Math.max(foot, top)) return layer.id;
  }
  return null;
}
function shareAt(/** @type {Layer} */ layer, /** @type {{ at: number, rest: boolean }} */ point, stacked = resolved ? stackOf(chart, resolved) : null) {
  const made = stacked?.get(layer.id), at = valuePointOf(layer, point);
  if (!made?.scale || at < 0) return NaN;
  return made.values[at] - made.foot[at];
}
/** @param {Layer} layer */
function profileKeyHtml(layer) {
  const groups = layer.profile ?? [], color = (/** @type {string} */ cohort) => `<i style="--c:${escapeHtml(ink(profileColor(layer, cohort)))}"></i>`;
  return `<span class="legend-key profile-key">${groups.length <= 4 ? groups.map((cohort) => `<span>${color(cohort)}<span>${escapeHtml(cohortName(cohort))}</span></span>`).join("") : `<span>${escapeHtml(cohortName(groups[0]))}</span><span class="key-strip">${groups.map(color).join("")}</span><span>${escapeHtml(cohortName(/** @type {string} */ (groups.at(-1))))}</span>`}</span>`;
}
function paintProfileItems() {
  for (const button of all$(".legend button[data-select]", $("chart"))) {
    const layer = layerById(button.dataset.select ?? ""), named = button.querySelector(".legend-name");
    if (layer?.type !== "profile" || !named) continue;
    const lit = heatmaps.litCohort(layer.id);
    if ((button.dataset.lit ?? "") === lit) continue;
    lit ? (button.dataset.lit = lit) : delete button.dataset.lit;
    writeHtml(named, lit ? legendNameHtml(layer, `<svg class="series-mark" viewBox="0 0 16 12" aria-hidden="true">${profileMark([ink(profileColor(layer, lit))])}</svg>`, cohortName(lit)) : legendNameHtml(layer));
  }
}
/** @param {Chart} doc @param {Layer} layer @param {string} [name] */
const viewName = (doc, layer, name = labelOf(layer)) => ((average) => average.name ? `${name} · ${average.name}` : name)(averageFor(doc, doc.panes.find((pane) => pane.id === layer.pane)));
/** @param {Layer} layer */
const legendNameHtml = (layer, mark = seriesMark(layer), name = layer.type === "profile" && !layer.label ? profileName(layer, false) : labelOf(layer)) => `<span class="legend-eye" data-toggle="${layer.id}" data-tip="Hide${layer.type === "signal" ? "" : `\n${keyText("⌥-click: alone in its pane")}`}">${mark}<span class="eye">${ICONS.eye}</span></span><span class="name">${escapeHtml(viewName(chart, layer, name))}</span>`;
/** @param {Chart} doc @returns {Map<string, string>} */
function foldedInto(doc) {
  const folded = new Map();
  for (const cloud of doc.layers) {
    if (cloud.type !== "cloud" || !cloud.byLayers) continue;
    for (const id of cloudLayers(cloud)) ((member) => member && member.pane === cloud.pane && !member.from && !folded.has(member.id) && (!cloud.hidden || member.hidden) && folded.set(member.id, cloud.id))(doc.layers.find((layer) => layer.id === id));
  }
  return folded;
}
/** @param {Layer} layer */
function cloudKeyHtml(layer) {
  const folded = foldedInto(chart), colors = [...new Set(chart.layers.filter((other) => folded.get(other.id) === layer.id && !other.hidden).map((other) => ink(other.color)))];
  return colors.length ? `<span class="legend-key cloud-key">${colors.map((color) => `<i style="--c:${escapeHtml(color)}"></i>`).join("")}</span>` : "";
}
function updateLegends() {
  if (!chart) return;
  const said = !$("unreached").hidden;
  const hidden = hiddenPanes(chart), all = [...$("chart").querySelectorAll(".legend")];
  for (const [at, element] of all.entries()) {
    const pane = chart.panes.find((item) => item.id === element.dataset.pane), focused = element.contains(document.activeElement);
    element.hidden = chart.legend === "off" || Boolean(pane && hidden.has(pane.id));
    if (element.hidden && focused) keepFocus(element, ...[...all.slice(at + 1), ...all.slice(0, at)].map((legend) => () => legend.hidden ? null : legend.querySelector(":scope > button:not([hidden])")), () => document.querySelector(`[data-pane-eye="${element.dataset.pane}"]`));
    if (element.hidden) continue;
    /** @param {Layer} layer */
    const item = (layer) => {
      const result = resolved?.results.get(layer.id);
      const value = result?.error && !said ? `<span class="legend-error"${result.detail ? ` data-tip="${escapeHtml(result.detail)}"` : ""}>${escapeHtml(result.error)}</span>`
        : fromUnfit(chart, layer) ? `<span class="legend-error">${fromUnfit(chart, layer)}</span>`
        : !layerShows(chart, layer) || !(result?.values || result?.heat) ? ""
        : pane?.stack === "percent" && bandsOf(pane).includes(layer) ? `<span class="legend-value" data-band-share="${layer.id}" style="min-width: ${widest(["100.0%"], 510, 12)}px"></span>`
        : `<span class="legend-value" data-value="${layer.id}" style="min-width: ${valueSlot(layer, 12)}px"></span>`;
      return `<button type="button" class="${layerShows(chart, layer) ? "" : "hidden"}" data-select="${layer.id}" style="--c:${escapeHtml(ink(colorNow(layer) ?? layer.color))}">
              <span class="legend-name">${legendNameHtml(layer)}</span>${value}
            </button>${layer.type === "profile" && !layer.hidden && layer.profile?.length ? profileKeyHtml(layer) : ""}${!layer.hidden ? cloudKeyHtml(layer) : ""}`;
    };
    const cloudMembers = foldedInto(chart);
    const mine = chart.layers.filter((layer) => layer.pane === element.dataset.pane && !cloudMembers.has(layer.id));
    const html = (drawnInto(chart, pane) && pane?.reverse ? mine.reverse() : mine).map(item).join("");
    let focus = null;
    if (legendShown.get(element) !== html) {
      legendShown.set(element, html);
      focus = focusKey(element);
      element.innerHTML = html;
    }
    const legend = engine.panes[chart.panes.indexOf(/** @type {Pane} */ (pane))]?.legend;
    legend?.set(/** @type {HTMLElement[]} */ ([...element.querySelectorAll(":scope > button[data-select]")]).map((button) => {
      const id = /** @type {string} */ (button.dataset.select);
      const readOn = (/** @type {string} */ layer) => ((series) => [series?.engine, series?.edge])(drawn.get(layer));
      const members = [...cloudMembers].filter(([, cloud]) => cloud === id).flatMap(([member]) => readOn(member));
      return { id, elements: [button, ...(button.nextElementSibling?.classList.contains("legend-key") ? [/** @type {HTMLElement} */ (button.nextElementSibling)] : [])], series: [...readOn(id), ...members].filter(Boolean), hidden: button.classList.contains("hidden") };
    }), (pane && asRows(chart, pane) && rowPanes.get(pane.id)?.scale) || "");
    element.setAttribute("role", "group");
    element.setAttribute("aria-label", `Legend for pane ${chart.panes.indexOf(/** @type {Pane} */ (pane)) + 1}`);
    // Put back once the library has laid its items: it moves them into place, which drops a focus.
    restoreFocus(element, focus);
    fitSoon(element);
  }
  markLegends();
  paintProfileItems();
  lightLegend(litLayer, true);
  updateValues();
}
function markLegends() {
  for (const item of all$(".legend [data-select]", $("chart"))) item.classList.toggle("selected", item.dataset.select === selected || layerPicks.has(/** @type {string} */ (item.dataset.select)));
}
function updateValues() {
  updateReadout();
  updateRowChanges();
  for (const caption of $("palette").querySelectorAll("[data-latest]")) {
    const layer = layerById(/** @type {HTMLElement} */ (caption).dataset.latest ?? ""), result = layer && resolved?.results.get(layer.id);
    if (layer) caption.textContent = command.latestSaid(layer, result, unitFor(layer, result));
  }
}
function strip(element) {
  element.addEventListener("scroll", () => fitSoon(element), { passive: true });
  element.addEventListener("wheel", (event) => {
    if (!element.classList.contains("overflows")) return;
    event.stopPropagation();
    if (Math.abs(event.deltaX) >= Math.abs(event.deltaY)) return;
    event.preventDefault();
    // In pixels: a wheel can count in lines, or pages.
    element.scrollLeft += event.deltaY * (event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? element.clientWidth : 1);
  }, { passive: false });
}
const fitting = new WeakSet();
function fitSoon(/** @type {HTMLElement} */ element, down = false) {
  if (fitting.has(element)) return;
  fitting.add(element);
  requestAnimationFrame(() => (fitting.delete(element), fitStrip(element, down)));
}
function fitStrip(element, down = false) {
  const [at, size, full] = down ? [element.scrollTop, element.clientHeight, element.scrollHeight] : [element.scrollLeft, element.clientWidth, element.scrollWidth];
  edges(element, at, size, full, down ? undefined : cutsOf(element));
}
/**
 * A faded edge says there's more where something's cut, not where the edge falls between two (a shelf stepped to its
 * next tiles, each whole: a fade there read as the last one cut off). Read before what's written.
 * @param {HTMLElement} element
 */
function cutsOf(element) {
  const box = element.getBoundingClientRect(), cut = { start: false, end: false };
  for (const child of element.children) {
    const r = child.getBoundingClientRect();
    if (r.left < box.left - 1 && r.right > box.left + 1) cut.start = true;
    if (r.left < box.right - 1 && r.right > box.right + 1) cut.end = true;
  }
  return cut;
}
/** @param {HTMLElement} element */
function edges(element, /** @type {number} */ at, /** @type {number} */ size, /** @type {number} */ full, cut = { start: true, end: true }) {
  const overflows = full > size + 1;
  element.classList.toggle("overflows", overflows);
  element.style.setProperty("--edge-start", `${overflows && cut.start ? clamp(at, 0, 28) : 0}px`);
  element.style.setProperty("--edge-end", `${overflows && cut.end ? clamp(full - size - at, 0, 28) : 0}px`);
}
/**
 * Come from none, it shows where it lands (no slide from where it last was). None lit, it fades where it is, then takes
 * no room: an empty box still counts in what the list scrolls. A row written anew is found by `again`, so the highlight
 * is there at once, shown as it was.
 * @param {HTMLElement} list
 * @param {{ again?: (gone: HTMLElement) => HTMLElement | null, can?: (row: HTMLElement) => boolean }} [how]
 */
function glides(list, { again = () => null, can = () => true } = {}) {
  list.classList.add("glides");
  let lit = /** @type {HTMLElement | null} */ (null), keyedNow = false, shown = false, dropped = false;
  const set = (/** @type {Record<string, string>} */ values) => Object.entries(values).forEach(([name, value]) => list.style.setProperty(`--glide-${name}`, value));
  const collapse = () => ((shown = false), set({ x: "0px", y: "0px", w: "0px", h: "0px" }));
  const place = (/** @type {HTMLElement} */ row, /** @type {boolean} */ still) => {
    let x = 0, y = 0;
    for (let at = /** @type {HTMLElement | null} */ (row); at && at !== list; at = /** @type {HTMLElement | null} */ (at.offsetParent)) (x += at.offsetLeft), (y += at.offsetTop);
    if (still) list.dataset.still = "";
    set({ x: `${x}px`, y: `${y}px`, w: `${row.offsetWidth}px`, h: `${row.offsetHeight}px`, r: getComputedStyle(row).borderRadius });
    if (still) (getComputedStyle(list, "::before").transform, delete list.dataset.still);
  };
  list.addEventListener("transitionend", (event) => event.target === list && event.pseudoElement === "::before" && event.propertyName === "opacity" && list.dataset.glide === "off" && collapse());
  new MutationObserver(() => {
    if (!lit) return shown && list.dataset.glide === "off" && ((list.dataset.glide = "gone"), collapse());
    if (lit.isConnected) return can(lit) ? place(lit, true) : light(null);
    const anew = again(lit);
    if (anew && can(anew)) return place((lit = anew), true);
    lit = null;
    dropped = true;
    queueMicrotask(() => dropped && ((dropped = false), (list.dataset.glide = "gone"), collapse()));
  }).observe(list, { childList: true, subtree: true });
  new ResizeObserver(() => lit && place(lit, true)).observe(list);
  const light = (/** @type {HTMLElement | null} */ row, keyed = false) => {
    if (row && !can(row)) row = null;
    if (row === lit && keyed === keyedNow) return;
    if (!row) {
      const unseen = reduced() || getComputedStyle(list, "::before").opacity === "0";
      if (lit) list.dataset.glide = "off";
      lit = null;
      if (unseen) collapse();
      return;
    }
    // Reached by keys, it's there at once, as a step is: gliding, it'd cross the rows between (a pointer's moves glide,
        // as the pointer does).
    place(row, !shown || dropped || keyed);
    list.dataset.glide = keyed ? "keyed" : "on";
    (lit = row), (keyedNow = keyed), (shown = true), (dropped = false);
  };
  return Object.assign(light, { fit: () => lit?.isConnected && (can(lit) ? place(lit, true) : light(null)) });
}
function whenSubmenu(/** @type {(menu: HTMLElement) => void} */ setup) {
  subSetups.push(setup);
  for (const sub of submenus) setup(sub);
}
function subAt(/** @type {number} */ level) {
  while (submenus.length < level) {
    const sub = document.createElement("div");
    Object.assign(sub, { id: `submenu-${submenus.length + 1}`, className: "menu submenu", hidden: true });
    sub.setAttribute("role", "menu");
    /** @type {HTMLElement} */ (submenus.at(-1)).after(sub);
    submenus.push(sub);
    for (const setup of subSetups) setup(sub);
  }
  return submenus[level - 1];
}
const levelOf = (/** @type {Element} */ menu) => submenus.indexOf(/** @type {HTMLElement} */ (menu)) + 1;
const inSubmenu = (/** @type {Element | null} */ element) => submenus.some((sub) => sub.contains(element));
/** @type {WeakMap<HTMLElement, (data: DOMStringMap) => void>} */
const menuPicks = new WeakMap();
const lightRows = (/** @type {HTMLElement} */ menu) => {
  const light = glides(menu), rowOf = (/** @type {Event} */ event) => /** @type {HTMLElement | null} */ (closest(event, ".menu > button"));
  const control = (/** @type {Event} */ event) => (closest(event, "button, input, select, textarea, a, [tabindex]") ?? menu) !== menu;
  menu.addEventListener("pointerover", (event) => {
    if (event.pointerType !== "mouse") return;
    const row = rowOf(event);
    if (row) light(row.matches(':disabled, [aria-disabled="true"]') ? null : row);
    else if (control(event)) light(null);
  });
  const focused = () => /** @type {HTMLElement | null} */ (menu.querySelector(":scope > button:focus-visible"));
  menu.addEventListener("pointerleave", () => light(focused()));
  menu.addEventListener("focusin", (event) => {
    const row = rowOf(event);
    if (!row) light(null);
    else if (row.matches(":focus-visible")) light(row, true);
  });
  menu.addEventListener("keydown", (event) => ["ArrowUp", "ArrowDown", "Home", "End"].includes(event.key) && setTimeout(() => focused() && light(focused(), true)));
  menu.addEventListener("focusout", (event) => !menu.contains(/** @type {Element | null} */ (event.relatedTarget)) && !menu.matches(":hover") && light(null));
};
/**
 * No attribute, so a row written again in place keeps it.
 * @type {WeakMap<Element, Animation>}
 */
const foldOf = new WeakMap();
/** @param {HTMLElement} group @param {string} rows */
function glideRows(group, rows) {
  const light = glides(group, {
    again: (gone) => {
      const key = gone.id ? `#${CSS.escape(gone.id)}` : ((name) => name && `[${name.name}="${CSS.escape(name.value)}"]`)([...gone.attributes].find(({ name }) => name.startsWith("data-") && !["data-tip", "data-kbd", "data-icon"].includes(name)));
      const row = key ? /** @type {HTMLElement | null} */ (group.querySelector(key)) : null;
      return row?.matches(rows) ? row : null;
    },
  });
  group.addEventListener("pointerover", (event) => {
    if (event.pointerType !== "mouse") return;
    const row = closest(event, rows);
    if (row && group.contains(row)) light(row.matches(':disabled, [aria-disabled="true"]') ? null : row);
    else if (((control) => control && control !== group && group.contains(control))(closest(event, "button, a, input, select, [tabindex], [data-tip]"))) light(null);
  });
  group.addEventListener("pointerleave", () => light(null));
}
let litLayer = null;
let litItems = /** @type {Map<string, HTMLElement[]>} */ (new Map());
function findLitItems() {
  const items = all$(".legend [data-select], .layer-row[data-layer]");
  litItems = new Map();
  for (const item of items) {
    const of = /** @type {string} */ (item.dataset.select ?? item.dataset.layer);
    litItems.set(of, [...(litItems.get(of) ?? []), item]);
  }
  return items;
}
function litItemsOf(/** @type {string} */ layer) {
  const kept = litItems.get(layer);
  if (kept?.length && kept.every((item) => item.isConnected)) return kept;
  findLitItems();
  return litItems.get(layer) ?? [];
}
function lightLegend(id, again = false, follow = false) {
  if (id === litLayer && !again) return;
  const of = (/** @type {string | null} */ layer) => (layer ? litItemsOf(layer) : []);
  const items = again ? findLitItems() : [...of(litLayer), ...of(id)];
  litLayer = id;
  for (const item of items) item.classList.toggle("lit", (item.dataset.select ?? item.dataset.layer) === id);
  // Lit from the chart (`follow`), its row is kept in sight in the panel: it glides there, each glide taken over by the
    // next as a sweep crosses many; not while the view moves (bands sweep under a still pointer).
  const row = id && follow && !viewMoving && panelShown() ? of(id).find((item) => item.matches(".layer-row")) : undefined, list = row && $("layer-list");
  if (!row || !list) return;
  const box = list.getBoundingClientRect(), at = row.getBoundingClientRect(), bar = /** @type {HTMLElement | null} */ (document.querySelector(".mobile-bar"));
  const tabs = bar?.getClientRects().length ? bar.getBoundingClientRect() : null;
  const bottom = Math.min(box.bottom, tabs && tabs.width > tabs.height ? tabs.top : Infinity);
  if (at.top < box.top || at.bottom > bottom) list.scrollBy({ top: at.top < box.top ? at.top - box.top - 8 : at.bottom - bottom + 8, behavior: reduced() ? "auto" : "smooth" });
}
function updateReadout() {
  const point = readPoint(), shown = panelShown();
  // The panel's rows only while it shows: hidden, each written value can lay the page out again.
  const reading = exporting || !shown ? legends() : [];
  for (const element of [...reading.flatMap((legend) => all$("[data-value]", legend)), ...(shown ? all$(".row-value[data-value]", $("layer-list")) : [])]) {
    const id = /** @type {string} */ (element.dataset.value), layer = layerById(id), result = resolved?.results.get(id);
    if (!layer || !result || result.error) continue;
    const inRow = element.classList.contains("row-value"), heat = result.heat ? heatmaps.heatReading(layer, point) : null;
    const value = valueAt(layer, point), band = result.low ? null : bandOf(layer, value), text = result.heat ? (heat ? (inRow ? heat.amount : heatmaps.readingLine(heat)) : "—") : band ? band.name : result.low ? rangeAt(layer, point, unitFor(layer, result), "lines" in element.dataset) : Number.isFinite(value) ? format(value, unitFor(layer, result)) : "—";
    writeText(element, text);
    const since = Number.isFinite(value) || result.low ? asOf(layer, point) : "";
    if (inRow) {
      const holder = /** @type {HTMLElement} */ (element.parentElement), note = /** @type {HTMLElement} */ (holder.querySelector(":scope > .row-note")), said = result.heat ? (heat?.at ? heatmaps.placeSaid(heat.at) : "") : since;
      if (writeText(note, said)) holder.classList.toggle("noted", Boolean(said));
    } else if ((element.dataset.asOf ?? "") !== since) {
      since ? (element.dataset.asOf = since) : delete element.dataset.asOf;
      // A legend's slot as wide as it gets, its date too: read at another date, without it, nothing moves.
      const room = since && element.classList.contains("legend-value") ? valueSlot(layer, 12) + widest([` · ${since}`], 400, 12) : 0;
      if (room > (parseFloat(element.style.minWidth) || 0)) element.style.minWidth = `${room}px`;
    }
    const color = band ? cssColor(band.color) : "";
    if ((element.dataset.band ?? "") !== color) {
      element.dataset.band = color;
      element.classList.toggle("named", Boolean(band));
      if (band) element.style.setProperty("--c", color);
      else element.style.removeProperty("--c");
    }
  }
  // The stacks read once, not once a band.
  const shares = reading.flatMap((legend) => all$("[data-band-share]", legend)), stacked = shares.length && resolved ? stackOf(chart, resolved) : null;
  for (const element of shares) {
    const layer = layerById(element.dataset.bandShare ?? ""), text = layer ? format(shareAt(layer, point, stacked), "percent") : "";
    writeText(element, text);
  }
}
/**
 * Its one text node's, where it has one: its children are kept, so nothing that reads them (:empty, :has) matches
 * again.
 * @param {Element} element @param {string} text
 */
function writeText(element, text) {
  const node = element.firstChild;
  if (node instanceof Text && !node.nextSibling) return node.data !== text && ((node.data = text), true);
  return element.textContent !== text && ((element.textContent = text), true);
}
/**
 * A touch: of an OHLC series a bar's wick (its low on a support, its high on a resistance), else its value at or past
 * the line: from that date on, the other side. `breaks`: each touch, the old side's last point, so nothing is drawn up
 * or down between.
 * @param {Vector} low @param {Vector} high @param {Result} ref
 */
function trendOf(low, high, ref) {
  const n = low.length, values = new Float64Array(n).fill(NaN), breaks = new Uint8Array(n), onSide = new Uint8Array(n);
  const downs = /** @type {Vector} */ (ref.lows ?? ref.values), ups = /** @type {Vector} */ (ref.highs ?? ref.values);
  let side = 0, last = -1;
  for (let i = 0; i < n; i++) {
    if (!Number.isFinite(low[i]) || !Number.isFinite(high[i])) continue;
    const down = downs[i], up = ups[i];
    if (!side) {
      if (!Number.isFinite(down) || !Number.isFinite(up)) continue;
      side = down + up >= low[i] + high[i] ? 1 : 2;
    } else if (side === 1 ? down <= low[i] : up >= high[i]) {
      side = side === 1 ? 2 : 1;
      if (last >= 0) breaks[last] = 1;
    }
    values[i] = side === 1 ? low[i] : high[i];
    onSide[i] = side;
    last = i;
  }
  return { values, breaks, sides: onSide };
}
/** @param {Vector[]} members */
function cloudOf(members, top = false, bottom = false, two = members.length === 2) {
  const n = members[0].length, m = members.length, low = new Float64Array(n).fill(NaN), high = new Float64Array(n).fill(NaN), highest = new Int8Array(n).fill(-1), sides = two && m === 2 && !top && !bottom ? new Uint8Array(n) : null;
  let side = 2;
  for (let i = 0; i < n; i++) {
    let lo = Infinity, hi = -Infinity, best = -1;
    for (let k = 0; k < m; k++) {
      const value = members[k][i];
      if (!Number.isFinite(value) || !((k > 0 && Number.isFinite(members[k - 1][i])) || (k < m - 1 && Number.isFinite(members[k + 1][i])) || (k === 0 && top) || (k === m - 1 && bottom))) continue;
      if (value < lo) lo = value;
      if (value > hi) (hi = value), (best = k);
    }
    if (best < 0) continue;
    low[i] = lo;
    high[i] = hi;
    highest[i] = best;
    if (sides) {
      const a = members[0][i], b = members[1][i];
      side = a > b ? 2 : a < b ? 1 : side;
      sides[i] = side;
    }
  }
  return { values: high, low, members, highest, ...(sides ? { sides } : {}) };
}
const gapped = (/** @type {Vector | undefined} */ values, /** @type {Result | undefined} */ result) => {
  const breaks = result?.breaks;
  if (!values || !breaks) return values;
  // In a loop: without a JIT, from() with a mapping costs several times as much.
  const out = new Float64Array(values.length);
  for (let i = 0; i < out.length; i++) out[i] = breaks[i] ? NaN : values[i];
  return out;
};
/** @param {Chart} doc @param {Layer} layer @param {Resolved} data @returns {((i: number) => string | null) | null} */
function colorsAt(doc, layer, data, seen = new Set()) {
  if (layer.colorFrom && !seen.has(layer.id)) {
    const source = doc.layers.find((item) => item.id === layer.colorFrom);
    const values = source && data.results.get(source.id)?.values;
    if (source) {
      const one = source.color, its = colorsAt(doc, source, data, seen.add(layer.id));
      return values ? (i) => (Number.isFinite(values[i]) ? its?.(i) ?? one : null) : null;
    }
  }
  const result = data.results.get(layer.id), values = result?.values, top = layer.color, under = belowOf(layer);
  if (layer.byGradient && layer.gradient && values) {
    const table = gradientTable(layer.gradient);
    if (!layer.upToDate) {
      const gradient = new Gradient(table, ...rangeOf(values));
      return (i) => (Number.isFinite(values[i]) ? gradient.color(values[i]) : null);
    }
    const places = upToDatePlaces(values);
    return (i) => (Number.isNaN(places[i]) ? null : table[placeIndex(places[i], table.length)]);
  }
  if (layer.byLevels && values && layer.levels?.length) {
    const steps = levelSteps(layer);
    return (i) => {
      if (!Number.isFinite(values[i])) return null;
      let color = top;
      for (const step of steps) {
        if (values[i] < step.value) break;
        color = step.color;
      }
      return color;
    };
  }
  if (layer.type === "histogram" && layer.colors === 4 && values) {
    const base = baseOf(layer), fallsAbove = fadingOf(layer, false), fallsBelow = fadingOf(layer, true);
    return (i) => {
      const value = values[i];
      if (!Number.isFinite(value)) return null;
      let before = i - 1;
      while (before >= 0 && !Number.isFinite(values[before])) before--;
      const back = before >= 0 && values[before] < base === value < base && Math.abs(value - base) < Math.abs(values[before] - base);
      return value < base ? (back ? fallsBelow : under) : back ? fallsAbove : top;
    };
  }
  if (layer.type === "cloud" && layer.byLayers && result?.highest && result.memberIds) {
    const highest = result.highest, colors = result.memberIds.map((id) => doc.layers.find((item) => item.id === id)?.color ?? top);
    return (i) => (highest[i] < 0 ? null : colors[highest[i]]);
  }
  if (!values || ink(top) === ink(under)) return null;
  const base = baseOf(layer);
  if (layer.type === "baseline" || layer.type === "histogram") return (i) => (!Number.isFinite(values[i]) ? null : values[i] < base ? under : top);
  const opens = result?.opens;
  if (layer.type === "candles" && opens) return (i) => (!Number.isFinite(values[i]) ? null : values[i] < opens[i] ? under : top);
  if (TWO_KEPT.has(layer.type) && result?.sides) return (i) => (/** @type {Uint8Array} */ (result.sides)[i] === 1 ? under : result.sides?.[i] ? top : null);
  return null;
}
/** @param {Chart} doc @param {Layer} layer */
function colorChain(doc, layer) {
  /** @type {Layer[]} */
  const chain = [];
  for (let at = /** @type {Layer | undefined} */ (layer); at && !chain.includes(at); at = at.colorFrom ? doc.layers.find((item) => item.id === at?.colorFrom) : undefined) chain.push(at);
  return chain;
}
function paintSig(/** @type {Chart} */ doc, /** @type {Layer} */ layer, /** @type {Resolved} */ data, /** @type {number} */ op, /** @type {string} */ base) {
  /** @type {any[]} */
  const sig = [layer.type, layer.fill, layer.fillOpacity, layer.solid, op, base];
  for (const at of colorChain(doc, layer)) {
    const result = data.results.get(at.id);
    sig.push(ink(at.color), ink(belowOf(at)), at.base, at.type, result?.values, result?.sides, at.byLevels ? JSON.stringify(at.levels) : "", at.byGradient ? `${JSON.stringify(at.gradient)}${at.upToDate ? "|to date" : `|${rangeOf(result?.values ?? []).join()}`}|${light}` : "", at.colors === 4 ? `${ink(fadingOf(at, false))} ${ink(fadingOf(at, true))}` : "", at.type === "cloud" && at.byLayers ? (result?.memberIds ?? []).map((id) => ink(doc.layers.find((item) => item.id === id)?.color ?? "")).join() : "", at.type === "cloud" && at.byLayers ? result?.members : undefined, at.type === "candles" ? result?.opens : undefined);
  }
  return sig;
}
const samePaint = (/** @type {any[] | null | undefined} */ a, /** @type {any[]} */ b) => Boolean(a) && a?.length === b.length && b.every((value, i) => a?.[i] === value);
const paintOf = (/** @type {Chart} */ doc, /** @type {Layer} */ layer, /** @type {{ times: number[], results: Map<string, any> }} */ data) => (layer.colorFrom || layer.byLevels || ownGradient(doc, layer) || TWO_KEPT.has(layer.type) || (layer.type === "histogram" && layer.colors === 4) ? colorsAt(doc, layer, /** @type {Resolved} */ (data)) : null);
/** @param {Layer} layer */
function colorNow(layer) {
  const at = resolved ? paintOf(chart, layer, resolved) : null;
  const values = at && resolved?.results.get(layer.id)?.values;
  if (!at || !values) return null;
  for (let i = values.length - 1; i >= 0; i--) if (Number.isFinite(values[i])) return at(i);
  return null;
}
/** @template T @param {(T | null)[]} points @param {(point: T) => string} tint @returns {{ color: string, points: T[] }[]} */
function byColor(points, tint, share = true) {
  /** @type {{ color: string, points: T[] }[]} */
  const pieces = [];
  /** @type {{ color: string, points: T[] } | null} */
  let piece = null;
  for (const point of points) {
    if (point === null) {
      piece = null;
      continue;
    }
    const color = tint(point);
    if (piece && piece.color !== color) {
      if (share) piece.points.push(point);
      piece = null;
    }
    if (!piece) pieces.push((piece = { color, points: [] }));
    piece.points.push(point);
  }
  return pieces;
}
/**
 * @typedef {{ series: any, colors: Uint16Array, palette: string[], look: (color: string) => Record<string, string> }} Painted
 * @type {Map<string, Painted>}
 */
const paintedLayers = new Map();
/** @param {number} n @param {(i: number) => string} colorAt */
function paletteOf(n, colorAt) {
  const colors = new Uint16Array(n), palette = /** @type {string[]} */ ([]), index = new Map();
  for (let i = 0; i < n; i++) {
    const color = colorAt(i);
    let k = index.get(color);
    if (k === undefined) index.set(color, (k = palette.length)), palette.push(color);
    colors[i] = k;
  }
  return { colors, palette };
}
/**
 * By a loop, not from(): its iterator is 4-6x slower without a JIT.
 * @param {number} n @param {(i: number) => number} at
 */
const columnBy = (n, at) => {
  const out = new Float64Array(n);
  for (let i = 0; i < n; i++) out[i] = at(i);
  return out;
};
class Standard {
  /** @param {"line" | "band" | "bars" | "candles"} kind @param {number} pane @param {"left" | "right"} side */
  constructor(kind, pane, side) {
    const on = engine.panes[pane];
    this.side = side;
    this.engine = on.add(kind === "band" ? new Band({ upper: EMPTY }) : kind === "bars" ? new Bars({}) : kind === "candles" ? new Candles({ open: EMPTY, high: EMPTY, low: EMPTY, close: EMPTY }) : new Line(), side);
    this.edge = kind === "band" ? on.add(new Line(), side) : null;
  }
}
class Bubbles {
  /** @param {number} pane @param {"left" | "right"} side @param {boolean} own */
  constructor(pane, side, own) {
    this.side = side;
    (this.v = EMPTY), (this.y = EMPTY);
    this.engine = new LibraryBubbles({ values: EMPTY, sizes: EMPTY });
    if (own) (this.engine.overlay = new Scale({ top: 0.1, bottom: 0.1 })), (this.engine.fit = fitMiddle);
    const near = this.engine.distance.bind(this.engine);
    this.engine.distance = (i, y, scale, x, time) => (near(i, y, scale, x, time) === 0 ? 0 : Infinity);
    engine.panes[pane].add(this.engine, side);
  }

  /** @param {Float64Array} v @param {Float64Array} y */
  set(v, y) {
    (this.v = v), (this.y = y);
    this.engine.setValues({ values: y, sizes: v });
    engine.invalidate();
  }

  /** @param {number} at @param {number} v @param {number} y */
  place(at, v, y) {
    if (this.v.length > at && this.y.length > at) return void (this.engine.set(at, y, v), engine.invalidate());
    const grow = (/** @type {Float64Array} */ from) => ((out) => (out.set(from), out))(new Float64Array(at + 1).fill(NaN));
    (this.v = grow(this.v)), (this.y = grow(this.y));
    (this.v[at] = v), (this.y[at] = y);
    this.set(this.v, this.y);
  }

  /** @param {Record<string, any>} look */
  style(look) {
    this.engine.setStyle(look);
    engine.invalidate();
  }
}
/** @param {number} _a @param {number} _b @param {boolean} _positive @param {Float64Array} out */
const fitMiddle = (_a, _b, _positive, out) => ((out[0] = Math.min(out[0], 0)), (out[1] = Math.max(out[1], 1)));
class Shading {
  /** @param {number} pane @param {"left" | "right"} side */
  constructor(pane, side) {
    this.side = side;
    this.engine = new Shade({ values: EMPTY });
    engine.panes[pane].add(this.engine, side);
  }

  /** @param {Float64Array} values @param {Uint16Array | null} [colors] @param {string[]} [palette] */
  set(values, colors = null, palette = []) {
    this.engine.setValues({ values, colors });
    this.engine.setStyle({ palette });
    engine.invalidate();
  }

  /** @param {string} color */
  style(color) {
    this.engine.setStyle({ color });
    engine.invalidate();
  }
}
/**
 * Run by run: a gap in its data is a gap in it, a point alone a sliver. Where its two cross it's pinched to the
 * crossing point, a color of its own from there. By its layers (`banded`) it is several clouds of two, each always
 * between its own two, so a color is never on what its layer doesn't bound.
 */
class Cloud {
  /** @param {number} pane @param {"left" | "right"} side @param {boolean} own */
  constructor(pane, side, own) {
    this.side = side;
    (this.low = EMPTY), (this.high = EMPTY);
    /** @type {Uint8Array | null} */
    this.sides = null;
    /** @type {Float64Array[]} */
    this.members = [];
    /** @type {Uint16Array | null} */
    this.colors = null;
    /** @type {string[]} */
    this.palette = [];
    this.look = { color: "transparent", top: false, bottom: false, banded: false, /** @type {string[]} */ bands: [] };
    (this.lows = new Line()), (this.highs = new Line());
    this.engine = new Custom({
      draw: (g, at) => this.#draw(g, at.time, at.scale, at.x0, at.x1, at.dpr),
      extent: (a, b, positive, out) => void (!this.engine.overlay && (this.lows.extent(a, b, positive, out), this.highs.extent(a, b, positive, out))),
      first: (i) => (this.engine.overlay ? 1 : this.highs.first(i)),
    });
    if (own) ((this.engine.overlay = new Scale({ top: 0.2, bottom: 0.1 })), this.engine.overlay.fit(0, 1));
    engine.panes[pane].add(this.engine, side);
  }

  /** @param {{ low: Float64Array, high: Float64Array, sides?: Uint8Array | null, members?: Float64Array[] }} columns */
  set({ low, high, sides = null, members = [] }) {
    (this.low = low), (this.high = high), (this.sides = sides), (this.members = members);
    this.lows.setValues(low);
    this.highs.setValues(high);
    this.#changed();
  }

  /** @param {Uint16Array | null} colors @param {string[]} palette */
  paint(colors, palette) {
    (this.colors = colors), (this.palette = palette);
    this.#changed();
  }

  /** @param {Partial<Cloud["look"]>} look */
  style(look) {
    Object.assign(this.look, look);
    this.#changed();
  }

  #changed() {
    this.engine.changed();
    engine.invalidate();
  }

  /**
   * @param {CanvasRenderingContext2D} g
   * @param {import("/modules/bitview-charts/timeline.js").Timeline} time
   * @param {import("/modules/bitview-charts/scale.js").Scale} scale
   * @param {number} x0
   * @param {number} x1
   * @param {number} dpr
   */
  #draw(g, time, scale, x0, x1, dpr) {
    const n = this.high.length, from = Math.max(0, Math.floor(time.index(x0 - 4) - 1)), to = Math.min(n, Math.ceil(time.index(x1 + 4) + 1) + 1);
    if (from >= to) return;
    if (CLOUD.xs.length < (to - from) * 2 + 2) for (const key of /** @type {const} */ (["xs", "highs", "lows", "ats"])) CLOUD[key] = new Float64Array((to - from) * 4 + 2);
    g.setTransform(1, 0, 0, 1, 0, 0);
    if (this.look.banded) this.#bands(g, time, scale, dpr, from, to);
    else this.#runs(g, time, scale, dpr, from, to);
  }

  /**
   * @param {CanvasRenderingContext2D} g
   * @param {import("/modules/bitview-charts/timeline.js").Timeline} time
   * @param {import("/modules/bitview-charts/scale.js").Scale} scale
   * @param {number} h
   * @param {number} from
   * @param {number} to
   */
  #runs(g, time, scale, h, from, to) {
    const look = this.look, { xs, highs, lows, ats } = CLOUD, v = h, height = scale.height, colors = this.colors, palette = this.palette;
    let color = look.color;
    const topAt = (/** @type {number} */ k) => (look.top ? 0 : Number.isNaN(ats[k]) ? scale.y(highs[k]) : ats[k]) * v, bottomAt = (/** @type {number} */ k) => (look.bottom ? height : Number.isNaN(ats[k]) ? scale.y(lows[k]) : ats[k]) * v;
    let k = 0, shared = false;
    const fill = () => {
      g.fillStyle = color;
      if (k === 1 && !shared) g.fillRect(xs[0] - h / 2, topAt(0), h, bottomAt(0) - topAt(0));
      else if (k > 1) {
        g.beginPath();
        for (let j = 0; j < k; j++) j ? g.lineTo(xs[j], topAt(j)) : g.moveTo(xs[j], topAt(j));
        for (let j = k - 1; j >= 0; j--) g.lineTo(xs[j], bottomAt(j));
        g.closePath();
        g.fill();
      }
      (k = 0), (shared = false);
    };
    const restart = () => {
      const [x0, high0, low0, at0] = [xs[k - 1], highs[k - 1], lows[k - 1], ats[k - 1]], drawn = k > 1;
      if (drawn) fill();
      (xs[0] = x0), (highs[0] = high0), (lows[0] = low0), (ats[0] = at0), (k = 1), (shared ||= drawn);
    };
    let last = NaN, side = 0;
    for (let i = from; i < to; i++) {
      const high = this.high[i], low = this.low[i];
      if (!(high === high && low === low)) continue;
      if (i !== last + 1) fill();
      last = i;
      const now = (colors && palette[colors[i]]) || look.color, column = time.snap(time.x(i), h) * h, there = this.sides ? this.sides[i] : 0;
      if (k && xs[k - 1] !== column && there !== side) {
        const top = scale.y(highs[k - 1]), bottom = scale.y(lows[k - 1]), nextTop = scale.y(high), nextBottom = scale.y(low);
        const before = Math.abs(bottom - top), after = Math.abs(nextBottom - nextTop), t = before + after > 0 ? before / (before + after) : 0.5;
        const first = side === 2 ? top : bottom, nextFirst = there === 2 ? nextTop : nextBottom;
        (xs[k] = xs[k - 1] + t * (column - xs[k - 1])), (ats[k++] = first + t * (nextFirst - first));
        if (now !== color) restart();
      } else if (now !== color && k) restart();
      color = now;
      side = there;
      if (k && xs[k - 1] === column && Number.isNaN(ats[k - 1])) {
        if (high > highs[k - 1]) highs[k - 1] = high;
        if (low < lows[k - 1]) lows[k - 1] = low;
      } else {
        (xs[k] = column), (highs[k] = high), (lows[k] = low), (ats[k++] = NaN);
      }
    }
    fill();
  }

  /**
   * @param {CanvasRenderingContext2D} g
   * @param {import("/modules/bitview-charts/timeline.js").Timeline} time
   * @param {import("/modules/bitview-charts/scale.js").Scale} scale
   * @param {number} h
   * @param {number} from
   * @param {number} to
   */
  #bands(g, time, scale, h, from, to) {
    const look = this.look, members = this.members, { xs, highs, lows } = CLOUD, v = h, TOP = -1, BOTTOM = -2, floor = scale.height * v;
    const n = members.length;
    const pairs = /** @type {[number, number][]} */ ([...Array.from({ length: Math.max(0, n - 1) }, (_, m) => [m, m + 1]), ...(look.top && n ? [[TOP, 0]] : []), ...(look.bottom && n ? [[n - 1, BOTTOM]] : [])]);
    for (const [a, b] of pairs) {
      let k = 0, color = "", last = NaN, over = 0, column = 0, before0 = NaN, before1 = NaN, shared = false, split = false, splitPaint = "";
      const fill = () => {
        g.fillStyle = color;
        if (k === 1 && !shared) g.fillRect(xs[0] - h / 2, highs[0], h, lows[0] - highs[0]);
        else if (k > 1) {
          g.beginPath();
          for (let q = 0; q < k; q++) q ? g.lineTo(xs[q], highs[q]) : g.moveTo(xs[q], highs[q]);
          for (let q = k - 1; q >= 0; q--) g.lineTo(xs[q], lows[q]);
          g.closePath();
          g.fill();
        }
        (k = 0), (shared = false);
      };
      const yOf = (/** @type {number} */ who, /** @type {number} */ i) => (who === TOP ? 0 : who === BOTTOM ? floor : ((value) => (Number.isFinite(value) ? scale.y(value) * v : NaN))(members[who][i]));
      const settle = () => {
        if (!split) return;
        split = false;
        if (k === 1) return void (color = splitPaint);
        const [x0, high0, low0] = [xs[k - 1], highs[k - 1], lows[k - 1]];
        fill();
        (xs[0] = x0), (highs[0] = high0), (lows[0] = low0), (k = 1), (color = splitPaint), (shared = true);
      };
      for (let i = from; i < to; i++) {
        if (!(this.high[i] === this.high[i] && this.low[i] === this.low[i])) continue;
        const ya = yOf(a, i), yb = yOf(b, i);
        if (i !== last + 1) settle(), fill();
        last = i;
        if (!Number.isFinite(ya) || !Number.isFinite(yb)) {
          settle(), fill();
          continue;
        }
        const now = ya < yb || (ya === yb && over === a) ? a : b, here = time.snap(time.x(i), h) * h, paint = look.bands[now < 0 ? (now === a ? b : a) : now] ?? look.color;
        if (k && xs[k - 1] === here) {
          // Bars narrower than a pixel: one column, as high and as low as any of them. Kept in the run, the run took
                    // the new color whole from wherever it began: a stretch's color changing as the zoom put its bars in columns.
          highs[k - 1] = Math.min(highs[k - 1], ya, yb);
          lows[k - 1] = Math.max(lows[k - 1], ya, yb);
          if (now !== over) split = true;
          if (split) splitPaint = paint;
          (over = now), (before0 = ya), (before1 = yb);
          continue;
        }
        settle();
        if (k && now !== over) {
          const d0 = before0 - before1, d1 = ya - yb, t0 = d0 / (d0 - d1), t = Number.isFinite(t0) && t0 >= 0 && t0 <= 1 ? t0 : 1, y = before0 + t * (ya - before0), at = column + t * (here - column);
          (xs[k] = at), (highs[k] = y), (lows[k++] = y);
          fill();
          (xs[0] = at), (highs[0] = y), (lows[0] = y), (k = 1);
        }
        (xs[k] = here), (highs[k] = Math.min(ya, yb)), (lows[k++] = Math.max(ya, yb));
        (color = paint), (over = now), (column = here), (before0 = ya), (before1 = yb);
      }
      settle();
      fill();
    }
  }
}
/**
 * Zoomed out, a heatmap draws a day in so many (a column `HEAT_COLUMN` pixels wide at least), each column its block's
 * last day, the days between never asked for; a day not come yet shows the coarser one over it meanwhile: the map comes
 * coarse, then sharpens (see viewHeat, heatShown).
 */
class Heat {
  /** @param {number} pane @param {"left" | "right"} side @param {string} id @param {boolean} profile */
  constructor(pane, side, id, profile) {
    this.side = side;
    this.id = id;
    this.alone = false;
    this.closes = EMPTY;
    this.line = new Line();
    const look = () => /** @type {HeatLook} */ (heatmaps.heatLooks.get(id)), fit = (/** @type {number} */ a, /** @type {number} */ b, /** @type {boolean} */ positive, /** @type {Float64Array} */ out) => void (this.alone && this.line.extent(a, b, positive, out));
    this.grid = profile ? null : new Grid({
      columns: new Int32Array(0),
      width: heatmaps.HEAT_COLUMN,
      column: (owner, step) => ((heat, shown) => (shown?.day.rows.length ? { rows: shown.day.rows, values: heatmaps.amountsOf(shown.day, heat.measure), key: shown.key } : null))(look(), look() && heatmaps.heatShown(look(), owner, step)),
      edge: (row) => /** @type {NonNullable<HeatLook["grid"]>} */ (look().grid).price(row),
      paint: (value, pixels, at) => heatmaps.heatColor(value, look().reference ?? 0, look(), pixels, at),
      viewed: (blocks) => void (look() && this.grid && heatmaps.viewHeat(look(), blocks, this.grid)),
      fit,
    });
    this.engine = this.grid ?? new Profile({
      edge: (row) => /** @type {NonNullable<HeatLook["grid"]>} */ (look().grid).price(row),
      cell: (value) => /** @type {NonNullable<HeatLook["grid"]>} */ (look().grid).cell(value),
      bin: PROFILE_BIN,
      viewed: () => void (this.engine instanceof Profile && (look() ? heatmaps.viewProfile(look(), this.engine) : this.engine.groups.length && this.engine.set({ groups: [] }))),
      fit,
    });
    engine.panes[pane].add(this.engine, side);
  }

  /** @param {Float64Array} closes */
  set(closes) {
    this.closes = closes;
    this.line.setValues(closes);
    if (this.grid) {
      const g = resolved ? heatmaps.heatCalendar(resolved).g : new Int32Array(0);
      this.grid.setColumns(Int32Array.from(closes, (close, i) => (close === close && i < g.length ? g[i] : -1)));
    }
    this.redraw();
  }

  /** @param {number} at @param {number} close */
  place(at, close) {
    const closes = this.closes.length > at ? this.closes : ((out) => (out.set(this.closes), out))(new Float64Array(at + 1).fill(NaN));
    closes[at] = close;
    this.set(closes);
  }

  redraw() {
    this.engine.changed();
    engine.invalidate();
  }
}
/**
 * @template {{ engine: Standard["engine"], edge?: Standard["edge"] }} T
 * @param {T} series
 * @param {"shares" | "zero" | null} stack
 * @param {boolean} out
 */
function fitSeries(series, stack, out) {
  const own = series.engine;
  own.fit = out ? NOTHING : stack === "shares" ? fitShares : stack === "zero" ? (/** @type {number} */ a, /** @type {number} */ b, /** @type {boolean} */ positive, /** @type {Float64Array} */ out) => {
    (SPAN[0] = Infinity), (SPAN[1] = -Infinity);
    own.extent(a, b, positive, SPAN);
    if (!(SPAN[0] <= SPAN[1])) return;
    (out[0] = Math.min(out[0], SPAN[0], 0)), (out[1] = Math.max(out[1], SPAN[1], 0));
  } : null;
  if (series.edge) series.edge.fit = own.fit ? NOTHING : null;
  return series;
}
const fitShares = (/** @type {number} */ _a, /** @type {number} */ _b, /** @type {boolean} */ _positive, /** @type {Float64Array} */ out) => ((out[0] = Math.min(out[0], 0)), (out[1] = Math.max(out[1], 100)));
const NOTHING = () => {};
/** @param {any} series @param {ArrayLike<number>} values @param {boolean} stacked */
function setColumn(series, values, stacked) {
  const column = values instanceof Float64Array ? values : new Float64Array(values), own = series.engine;
  if (series.edge) own.set({ upper: column, ...(stacked ? {} : { lower: null }) }), series.edge.setValues(column);
  else own.setValues(column);
  engine.invalidate(true);
}
/** @param {any} series @param {Float64Array} values @param {number} at @param {number} value */
function liveColumn(series, values, at, value) {
  const line = series.edge ?? series.engine;
  if (line.values !== values) return void setColumn(series, values, false);
  line.set(at, value);
  if (series.edge) series.engine.set({});
  engine.invalidate();
}
/** @param {Result} candles @param {ArrayLike<number>} shown @param {boolean} log @param {(value: number, i: number) => number} at */
function candleColumns(candles, shown, log, at) {
  const closes = /** @type {Vector} */ (candles.values), opens = /** @type {Vector} */ (candles.opens), highs = /** @type {Vector} */ (candles.highs), lows = /** @type {Vector} */ (candles.lows);
  const n = closes.length, open = new Float64Array(n).fill(NaN), high = new Float64Array(n).fill(NaN), low = new Float64Array(n).fill(NaN), close = new Float64Array(n).fill(NaN);
  for (let i = 0; i < n; i++) {
    if (!Number.isFinite(closes[i]) || !Number.isFinite(shown[i]) || (log && !(at(lows[i], i) > 0))) continue;
    (open[i] = at(opens[i], i)), (high[i] = at(highs[i], i)), (low[i] = at(lows[i], i)), (close[i] = at(closes[i], i));
  }
  return { open, high, low, close };
}
/** A pane of rows' cells as strong as its strongest drawn whole: the rest of the way past them, the same (a rare spike never pales the rest). */
const ROWS_WHOLE = 0.98;
const ROWS_FAINT = 0.08;
const ROWS_CURVE = 0.6;
/** @param {number} strength @param {number} curve */
const rowsOpacity = (strength, curve) => ROWS_FAINT + (1 - ROWS_FAINT) * strength ** curve;
/** @type {Map<string, { on: import("/modules/bitview-charts/pane.js").Pane, side: string, rows: Rows, key: string, scale: HTMLElement }>} */
const rowPanes = new Map();
/** @param {Float64Array} sorted @param {number} share */
const quantile = (sorted, share) => (sorted.length ? sorted[Math.min(sorted.length - 1, Math.floor(share * sorted.length))] : NaN);
/** @param {Chart} doc @param {Resolved} data */
function drawRows(doc, data) {
  const gone = new Set(rowPanes.keys());
  doc.panes.forEach((pane, i) => {
    const on = engine.panes[i];
    if (!asRows(doc, pane) || !on) return;
    gone.delete(pane.id);
    for (const layer of doc.layers) if (layer.pane === pane.id) ((own) => own && [own.engine, own.edge].forEach((series) => series && (series.hidden = true)))(drawn.get(layer.id));
    const layers = rowLayers(doc, pane).filter((layer) => data.results.get(layer.id)?.values), side = layers[0]?.side === "left" ? "left" : "right";
    let kept = rowPanes.get(pane.id);
    if (kept && (kept.on !== on || kept.side !== side)) kept.on.remove(kept.rows), (kept = undefined);
    if (!kept) {
      kept = { on, side, rows: new Rows({ paint: NOTHING, side }), key: "", scale: document.createElement("span") };
      on.add(kept.rows, side);
      rowPanes.set(pane.id, kept);
    }
    const log = paneMode(pane) === 1, level = rowsLevel(doc, pane), outlined = level !== undefined && pane.outline === true;
    const key = JSON.stringify([tokensNow(), log, level, outlined, layers.map((layer) => [layer.id, idOf(/** @type {Vector} */ (data.results.get(layer.id)?.values)), layer.color, layer.short ?? labelOf(layer)]), pane.gradient]);
    if (key === kept.key) return;
    kept.key = key;
    const rows = layers.map((layer) => ({ values: /** @type {Vector} */ (data.results.get(layer.id)?.values), name: layer.short ?? labelOf(layer), color: ink(layer.color), id: layer.id }));
    // (A value's place: about a ratio's level (on a log scale, a level over 0), its log ratio; about an amount's, its
    // signed log, sign × ln(1 + |v − level| / typical), `typical` the median distance of the pane's cells from the level
    // (none at it): a typical cell half way up the log's first step, a year of billions not blacking out a decade of
    // millions; with no level, its own (its log).)
    const ratio = log && level !== undefined && level > 0, signed = level !== undefined && !ratio;
    const places = new Float64Array(rows.reduce((sum, row) => sum + row.values.length, 0));
    const placed = (/** @type {(value: number) => number} */ of) => {
      let count = 0;
      for (const { values } of rows) for (let i = 0; i < values.length; i++) ((at) => Number.isFinite(at) && (places[count++] = at))(of(values[i]));
      return places.subarray(0, count).sort();
    };
    const typical = signed ? quantile(placed((value) => Math.abs(value - /** @type {number} */ (level)) || NaN), 0.5) || 1 : 1;
    const place = level === undefined ? (log ? Math.log10 : (/** @type {number} */ value) => value) : ratio ? (/** @type {number} */ value) => Math.log(value / level) : (/** @type {number} */ value) => ((d) => Math.sign(d) * Math.log1p(Math.abs(d) / typical))(value - level);
    // (Its ends: about a ratio's level, each side's own, so the side that strays less still reads (MVRV's 0.6 as red as
    // its 4 is green); about an amount's, one for both, the log already evening them.)
    const whole = signed ? quantile(placed((value) => Math.abs(place(value)) || NaN), ROWS_WHOLE) : NaN, sorted = signed ? null : placed(place);
    const lo = sorted ? quantile(sorted, 1 - ROWS_WHOLE) : -whole, hi = sorted ? quantile(sorted, ROWS_WHOLE) : whole;
    const up = channels(ink("green")), down = channels(ink("red")), table = level === undefined ? heatmaps.gradientChannels(paneGradient(pane)[2]) : [];
    // (About a ratio, the middle lifted (see ROWS_CURVE); a signed log, as it is: lifted again, most cells read alike.)
    const curve = ratio ? ROWS_CURVE : 1;
    kept.rows.paint = (value, _, pixels, at) => {
      const p = place(value);
      if (!Number.isFinite(p) || (level === undefined ? value === 0 : p === 0)) return;
      if (level === undefined) return heatmaps.gradientCell(table, clamp((p - lo) / (hi - lo || 1), 0, 1), 1, pixels, at);
      const strength = clamp(p < 0 ? p / (lo < 0 ? lo : -1) : p / (hi > 0 ? hi : 1), 0, 1), [r, g, b] = p < 0 ? down : up;
      (pixels[at] = r), (pixels[at + 1] = g), (pixels[at + 2] = b), (pixels[at + 3] = 255 * rowsOpacity(strength, curve));
    };
    kept.rows.below = outlined ? (value) => place(value) < 0 : null;
    kept.rows.outline = css("--ink");
    const words = valueFormatOf(layers.slice(0, 1).map((layer) => unitFor(layer, data.results.get(layer.id)))).value, at = (/** @type {number} */ p) => (ratio ? /** @type {number} */ (level) * Math.exp(p) : level === undefined ? (log ? 10 ** p : p) : level + Math.sign(p) * typical * Math.expm1(Math.abs(p)));
    const strip = (/** @type {(strength: number) => string} */ shade, /** @type {boolean} */ rising) => `<span class="key-strip">${[0, 1, 2, 3, 4].map((k) => `<i style="--c:${shade(((rising ? k : 4 - k) + 0.5) / 5)}"></i>`).join("")}</span>`;
    const toward = (/** @type {number[]} */ c) => (/** @type {number} */ strength) => `rgb(${c.slice(0, 3).join(" ")} / ${rowsOpacity(strength, curve)})`;
    const along = (/** @type {number[][]} */ table) => (/** @type {number} */ strength) => `rgb(${table[placeIndex(strength, table.length)].slice(0, 3).join(" ")} / ${Math.min(1, strength / heatmaps.HEAT_FADE)})`;
    const ends = (/** @type {number} */ p) => `<span>${escapeHtml(Number.isFinite(p) ? words(at(p)) : "")}</span>`;
    kept.scale.className = "legend-key rows-key";
    kept.scale.innerHTML = level === undefined ? `${ends(lo)}${strip(along(table), true)}${ends(hi)}` : `${ends(Math.min(lo, 0))}${strip(toward(down), false)}<span>${escapeHtml(words(level))}</span>${strip(toward(up), true)}${ends(Math.max(hi, 0))}`;
    kept.rows.set(rows);
  });
  for (const id of gone) ((kept) => kept && (kept.on.remove(kept.rows), rowPanes.delete(id)))(rowPanes.get(id));
}
/** @param {Chart} doc @param {ReturnType<typeof stackOf>} stacked */
function stackBands(doc, stacked) {
  const bands = doc.layers.flatMap((layer) => ((stack, series) => (stack && series?.edge && (layer.type === "line" || (layer.type === "area" && layer.fill !== "above")) ? [{ stack, series }] : []))(stacked.get(layer.id), drawn.get(layer.id))).sort((a, b) => a.stack.rank - b.stack.rank);
  bands.forEach(({ stack, series }, k) => {
    const below = bands.slice(0, k).findLast((other) => other.stack.group === stack.group);
    series.engine.set({ lower: below ? below.series.engine.upper : null, base: 0 });
  });
  if (bands.length) engine.invalidate(true);
}
/**
 * @param {any} series
 * @param {{ hidden: boolean, tag: boolean, priceLine: boolean, format: (value: number) => string, outline: string | null, line?: Record<string, any>, band?: Record<string, any>, bars?: Record<string, any>, candles?: Record<string, any> }} look
 */
function styleSeries(series, look) {
  const own = series.engine, edge = series.edge, shows = !look.hidden, tags = { tag: look.tag && shows, priceLine: look.priceLine && shows, format: look.format, outline: look.outline };
  own.hidden = look.hidden;
  if (edge) edge.hidden = look.hidden;
  if (look.band) own.set(look.band), edge.setStyle({ ...tags, ...look.line });
  else if (look.bars) own.setStyle({ ...tags, ...look.bars });
  else if (look.candles) own.setStyle({ ...tags, ...look.candles });
  else own.setStyle({ ...tags, ...look.line });
  engine.invalidate(true);
}
/** @param {number | undefined} width @param {number | undefined} style */
const dashFor = (width = 1, style = 0) => [[], [width, width], [2 * width, 2 * width], [6 * width, 6 * width], [width, 4 * width]][style] ?? [];
/** @param {any} series @param {Painted | null} painted */
function paintSeries(series, painted) {
  const colors = painted?.colors ?? null, looks = painted ? painted.palette.map(painted.look) : [];
  if (series instanceof Cloud) return void series.paint(colors, looks.map((look) => look.color));
  if (series.edge) series.engine.set({ colors, palette: looks.map((look) => ({ color: look.fill, fade: look.fade })) }), series.edge.setStyle({ colors, palette: looks.map((look) => look.line) });
  else series.engine.setStyle({ colors, palette: looks.map((look) => look.color) });
  engine.invalidate(true);
}
/** @param {Painted} painted @param {number} at @param {string} color */
function paintPoint(painted, at, color) {
  let k = painted.palette.indexOf(color);
  if (k < 0) (k = painted.palette.length), painted.palette.push(color);
  if (painted.colors.length <= at) painted.colors = ((out) => (out.set(painted.colors), out))(new Uint16Array(at + 1));
  painted.colors[at] = k;
  paintSeries(painted.series, painted);
}
/** @type {WeakMap<object, Map<string, number>>} */
const slots = new WeakMap();
const slotsNow = () => {
  const measured = slots.get(/** @type {object} */ (resolved)) ?? new Map();
  if (measured.size > 400) measured.clear();
  slots.set(/** @type {object} */ (resolved), measured);
  return measured;
};
/** @type {Map<string, number>} */
const binnedSlots = new Map();
function slotOf(/** @type {Vector} */ values, /** @type {string} */ unit, /** @type {number} */ size) {
  const texts = new Set();
  const stride = Math.max(1, Math.floor(values.length / 240));
  const [lo, hi] = rangeOf(values);
  let last = NaN;
  for (let i = values.length - 1; i >= 0; i--) if (values[i] - values[i] === 0) {
    last = values[i];
    break;
  }
  for (const value of [lo, hi, last]) if (Number.isFinite(value)) texts.add(format(value, unit));
  for (let i = 0; i < values.length; i += stride) if (Number.isFinite(values[i])) texts.add(format(values[i], unit));
  return widest([...texts].sort((x, y) => y.length - x.length).slice(0, 5), 510, size);
}
/** @param {Layer} layer */
const binnedAt = (layer) => [0.01234, 888_888].map((value) => (layer.type === "profile" ? heatmaps.priceSpan(value, value) : format(value, "usd")));
/**
 * The room the legend and the panel keep for a value, so values change on hover without moving anything: its lowest,
 * highest, latest and a sample across its history written out, the longest measured at the size in semibold tabular
 * figures, in whole pixels.
 */
function valueSlot(layer, size, lines = false) {
  if (heatLike(layer)) {
    const key = `${layer.type}:${measureOf(layer)}:${Boolean(layer.cumulative)}:${size}:${lines}`, measured = binnedSlots;
    if (measured.has(key)) return /** @type {number} */ (measured.get(key));
    const unit = measureUnit(measureOf(layer)), amounts = WIDEST_VALUES.flatMap((value) => (signedMeasure(layer) ? [value, -value] : [value])).map((value) => format(value, unit));
    const changes = [...WIDEST_VALUES, -WIDEST_VALUES[0], -99.99].map(heatmaps.fromClose);
    const hovered = layer.cumulative ? changes.flatMap((change) => amounts.map((amount) => `${change} ${amount}`)) : lines ? amounts : amounts.flatMap((amount) => binnedAt(layer).map((at) => heatmaps.readingLine({ amount, at })));
    const shares = [100, 88.88, WIDEST_VALUES[0]].map((share) => `≈${format(share, "percent")} in profit`);
    const width = widest([...hovered, ...(layer.type === "profile" ? shares : [])], 510, size);
    measured.set(key, width);
    return width;
  }
  const result = resolved?.results.get(layer.id), values = result?.values;
  if (!values) return 0;
  const unit = unitFor(layer, result), key = `${layer.id}:${unit}:${size}:${layer.from ? edgesOf(layer).join() : ""}:${lines ? "lines" : ""}:${named(layer) ? JSON.stringify([layer.underName, layer.levels?.map((level) => level.name)]) : ""}`;
  const measured = slotsNow();
  if (measured.has(key)) return measured.get(key);
  if (result?.low) {
    const width = edgesOf(layer).length > 1 ? widest(["—"], 510, size) : edgesOf(layer).length ? widest([layer.from?.includes("top") ? "≥ " : "≤ "], 510, size) + slotOf(values, unit, size)
      : lines ? Math.max(slotOf(result.low, unit, size), slotOf(values, unit, size))
      : slotOf(result.low, unit, size) + widest([" – "], 510, size) + slotOf(values, unit, size);
    measured.set(key, width);
    return width;
  }
  const width = Math.max(slotOf(values, unit, size), named(layer) ? Math.min(152, widest([...(layer.levels ?? []).flatMap((level) => (level.name ? [level.name] : [])), layer.underName ?? ""], 510, size)) : 0);
  measured.set(key, width);
  return width;
}
/**
 * Relative when the series stays positive across the view, else a difference, the same way for every point of a view.
 */
function changeOver(layer, point) {
  if (layer.type === "signal") return null;
  const span = changeWindow(layer);
  if (!span) return null;
  const { values, times, a, base, relative, unit } = span;
  const b = point.rest ? valuePointOf(layer, point) : point.at;
  if (b < a || !Number.isFinite(values[b])) return null;
  return change(values[relative && b >= base ? base : a], values[b], relative && b >= base, unit, times[relative && b >= base ? base : a], times[b]);
}
function change(from, to, relative, unit, fromTime = 0, toTime = 0) {
  const ratio = relative ? growth(from, to) : null;
  const delta = to - from;
  return { up: to === from ? null : ratio ? ratio.up : delta > 0, text: ratio ? ratio.text : amount(delta, unit), from: fromTime, to: toTime };
}
const chipText = (change) => `${change.up === null ? "" : change.up ? "↑ " : "↓ "}${change.text}`;
let windows = { data: /** @type {Resolved | null} */ (null), view: "", by: new Map() };
/** @type {WeakMap<object, { first: number, nonZero: number, below: Int32Array }>} */
const scans = new WeakMap();
function scanOf(/** @type {ArrayLike<number>} */ values) {
  let scan = scans.get(values);
  if (scan) return scan;
  const n = values.length, below = new Int32Array(n + 1);
  let first = n, nonZero = n;
  for (let i = 0; i < n; i++) {
    // Finite by a subtraction: no call per value.
    const value = values[i], finite = value - value === 0;
    if (finite && first === n) first = i;
    if (finite && value && nonZero === n) nonZero = i;
    below[i + 1] = below[i] + (finite && value < 0 ? 1 : 0);
  }
  scans.set(values, (scan = { first, nonZero, below }));
  return scan;
}
function rescan(/** @type {ArrayLike<number>} */ values, /** @type {number} */ at) {
  const scan = scans.get(values);
  if (!scan) return;
  const value = values[at], finite = Number.isFinite(value);
  if (scan.below.length !== values.length + 1 || (!finite && at === scan.first) || (!(finite && value) && at === scan.nonZero)) return void scans.delete(values);
  if (finite && at < scan.first) scan.first = at;
  if (finite && value && at < scan.nonZero) scan.nonZero = at;
  for (let i = at; i < values.length; i++) scan.below[i + 1] = scan.below[i] + (Number.isFinite(values[i]) && values[i] < 0 ? 1 : 0);
}
/**
 * From the view's first value; relatively from its first non-zero one when the series stays positive across the view
 * (the price before 2010 starts at zero).
 */
function changeWindow(layer) {
  const range = shownSpan();
  if (!resolved || !range || !resolved.times.length) return null;
  const result = resolved.results.get(layer.id), values = result?.values;
  if (!values) return null;
  const last = lastShown(), first = firstInView(), unit = unitFor(layer, result);
  if (windows.data !== resolved || windows.view !== `${first}:${last}`) windows = { data: resolved, view: `${first}:${last}`, by: new Map() };
  const key = `${layer.id}:${unit}`;
  if (windows.by.has(key)) return windows.by.get(key);
  const scan = scanOf(values);
  if (first <= scan.first) return (windows.by.set(key, null), null);
  let a = Math.min(Math.max(first, scan.first), last);
  while (a < last && !Number.isFinite(values[a])) a++;
  let found = null;
  if (Number.isFinite(values[a])) {
    let base = Math.min(Math.max(a, scan.nonZero), last);
    while (base < last && !values[base]) base++;
    const negative = scan.below[last + 1] > scan.below[a];
    found = { values, times: resolved.times, a, base, last, lo: NaN, hi: NaN, unit, relative: unit !== "percent" && !negative && values[base] > 0 };
  }
  windows.by.set(key, found);
  return found;
}

const PERIODS = { "3m": "past 3 months", "1y": "past year", "4y": "past 4 years", "8y": "past 8 years", max: "whole history" };
const periodOf = (/** @type {string} */ id) => (id === "max" && FINE.has(chart.index) ? "most that fits" : PERIODS[id]);

/**
 * A finger's crosshair (held, then dragged): lifted, it stays till the next touch on the plot (a tap: gone, taking no
 * line or drawing; a drag pans). Not while drawing: a tool's points are the finger's.
 */
const fingerCross = { at: /** @type {{ price: number, time: number, series: any } | null} */ (null), kept: /** @type {number | null} */ (null), dismissing: false };
/**
 * The crosshair set here (a finger's kept, the keys'): when the view, the data or the size changes under it, the chart
 * draws it again at its place, not its date and price, reading whatever is there. It's put back on its date and price
 * (see holdCross).
 */
let crossSet = /** @type {{ price: number, time: number, layer: string } | null} */ (null);
/**
 * While the chart's built, what it says of the crosshair set here is left mid-way (its date may be out of view a
 * moment, its layer not drawn yet): put back once that's done (see build).
 */
let crossHeld = false;
/** @typedef {{ point?: { x: number, y: number }, time?: number, logical?: number, pane?: number, input: boolean, layer: string | null }} ChartPoint */
/** @param {number} index @param {number} pane @param {number} x @param {number} y @param {boolean} input @param {number} hit @returns {ChartPoint} */
function chartPoint(index, pane, x, y, input, hit) {
  if (index !== index) return { input, layer: null };
  const line = hit >= 0 ? engine.panes[pane]?.series[hit] : null;
  const row = line instanceof Rows ? line.rows[line.rowAt(y, engine.panes[pane].scaleOf(hit))] : null;
  const layer = row ? row.id : !line ? null : [...drawn].find(([, series]) => series.engine === line || series.edge === line)?.[0] ?? null;
  return { point: { x, y }, time: engine.time.times[index], logical: index, pane: pane >= 0 ? pane : undefined, input, layer };
}
/** @type {ChartPoint} */
const NO_CROSS = { input: false, layer: null };
let crossLater = 0;
/** @param {typeof crossSet} pin */
const pinCross = (pin) => ((crossSet = pin), cancelAnimationFrame(crossLater), (crossLater = 0));
/**
 * Its date out of view may be a moment's (a box resized, then its view kept; a range set): it's judged a frame on
 * (`settled`), none shown meanwhile. Where it can't be put back (its layer gone or hidden, nothing drawn), let go.
 */
function holdCross(settled = false) {
  const { price, time, layer } = /** @type {NonNullable<typeof crossSet>} */ (crossSet), series = drawn.get(layer), item = layerById(layer), i = indexOfTime(time);
  if (!series || !item || !layerShows(chart, item) || seriesY(series, price) === null) return dropCross();
  if (i < 0 || i < firstInView() || i > lastShown()) {
    if (settled) return dropCross();
    engine.setCrosshair(NaN);
    lightBubble(null, undefined);
    crossLater ||= requestAnimationFrame(() => ((crossLater = 0), crossSet && holdCross(true)));
    return;
  }
  lightBubble(time, seriesPane(series));
  if (heatmaps?.heatLooks.size) valuesSoon();
  crossOn(price, time, series);
}
function dropCross() {
  pinCross(null), (fingerCross.kept = null);
  if (keyTime !== null) keyAway();
  else engine.setCrosshair(NaN);
  crossMoved(NO_CROSS);
}
function dotLineOf(/** @type {string} */ id) {
  const own = drawn.get(id);
  return own?.edge ?? own?.engine ?? null;
}
let bubbleAt = /** @type {{ time: number | null, pane: number | undefined }} */ ({ time: null, pane: undefined });
/** @param {number | null} time @param {number | undefined} pane */
function lightBubble(time, pane) {
  bubbleAt = { time, pane };
  let lit = null;
  const doc = displayed?.doc ?? chart;
  if (time !== null && pane !== undefined && resolved && doc.layers.some((layer) => layer.type === "bubbles")) {
    const x = xOfTime(time), i = lowerBound(resolved.times, time);
    for (const layer of doc.layers) {
      const series = drawn.get(layer.id), row = series instanceof Bubbles && !layer.hidden && doc.panes.findIndex((item) => item.id === layer.pane) === pane && resolved.times[i] === time ? { v: series.v[i], y: series.y[i] } : null;
      if (!row || x === null) continue;
      series.engine.viewed(engine.time);
      const { most, hide, size, upEdge, downEdge } = series.engine, worth = Math.abs(row.v), y = Number.isFinite(row.y) ? seriesY(series, row.y) : null;
      if (y === null || !(most > 0 && worth > 0 && worth < Infinity && worth >= hide * most) || (lit && !doc.panes[pane]?.reverse)) continue;
      lit = { x, y, d: Math.max(size * Math.sqrt(Math.min(1, worth / most)), 2 * (MARKER.radius + MARKER.border)) + 6, color: row.v >= 0 ? upEdge : downEdge };
    }
  }
  const plot = lit && engine.panes[/** @type {number} */ (pane)]?.frame;
  if (!lit || !plot) return void bubbleRing.remove();
  if (bubbleRing.parentElement !== plot) plot.append(bubbleRing);
  Object.assign(bubbleRing.style, { left: `${lit.x}px`, top: `${lit.y}px`, width: `${lit.d}px`, height: `${lit.d}px` });
  bubbleRing.style.setProperty("--c", lit.color);
}
function tagAt(/** @type {number} */ x, /** @type {number} */ y) {
  const box = engine.element.getBoundingClientRect(), line = engine.tagAt(x - box.left, y - box.top);
  if (line) for (const [id, series] of drawn) if (line === series.engine || line === series.edge) return id;
  return null;
}
function namePointed() {
  if (crossSet) return;
  const line = bandLit ? dotLineOf(bandLit) : profileLit ? drawn.get(profileLit.layer)?.engine : tagLit ? dotLineOf(tagLit) : undefined;
  if (engine.crosshair.pointed === (line ?? undefined)) return;
  engine.crosshair.pointed = line ?? undefined;
  engine.invalidate();
}
function placePlotKeys() {
  const keys = $("plot-keys"), plot = home || !chart?.layers.length || !chartShown() ? null : plotBox();
  if (!plot) keyAway();
  keys.hidden = !plot;
  if (!plot) return;
  Object.assign(keys.style, { left: `${plot.left}px`, top: `${plot.top}px`, width: `${plot.width}px`, height: `${plot.height}px` });
  if (document.activeElement === keys) describePlot();
}
function describePlot() {
  $("plot-keys").setAttribute("aria-label", `Chart: ${titleOf(chart)}`);
  $("chart-summary").textContent = chartSummary();
}
function sayValue(/** @type {Layer} */ layer, /** @type {{ at: number, rest: boolean }} */ point) {
  const result = resolved?.results.get(layer.id);
  if (!result || result.error) return `(${result?.error ?? "No data"})`;
  const value = valueAt(layer, point), since = asOf(layer, point), band = result.low ? null : bandOf(layer, value);
  const said = band ? band.name : result.low ? rangeAt(layer, point, unitFor(layer, result)) : Number.isFinite(value) ? format(value, unitFor(layer, result)) : "no value";
  const share = chart.panes.find((pane) => pane.id === layer.pane)?.stack === "percent" ? shareAt(layer, point) : NaN;
  return `${Number.isFinite(share) ? `${format(share, "percent")} (${said})` : said}${since ? `, as of ${since}` : ""}`;
}
function chartFacts() {
  if (!resolved?.times.length) return "No data yet.";
  const point = { at: lastShown(), rest: true }, shown = shownDates();
  const span = shown ? spanParts(Number(shown.from), Number(shown.to)).join(" to ").replace(/’(\d\d)/g, "20$1") : "";
  const panes = chart.panes.map((pane, i) => {
    const layers = chart.layers.filter((layer) => layer.pane === pane.id && !layer.hidden);
    return layers.length ? capitalized(`${chart.panes.length > 1 ? `Pane ${i + 1}, ` : ""}${pane.stack === "percent" ? "100% stacked" : `${MODES[paneMode(pane)].name.toLowerCase()} scale`}: ${layers.map((layer) => `${viewName(chart, layer)} ${sayValue(layer, point)}`).join(", ")}`) : "";
  }).filter(Boolean);
  return `${CADENCE[shownIndex()]}, ${span}. ${panes.join(". ")}.`;
}
const chartSummary = () => `${chartFacts()} Use the left and right arrows to read it date by date (Shift for ten at a time), and Home and End for the first and last dates shown. With a drawing tool selected, Enter places a point there and the up and down arrows move it off the line.`;
function keyPoint(/** @type {number} */ i) {
  // On a log scale, none at or under zero (a cloud's low too: it's drawn only above).
  const at = (/** @type {Layer} */ item) => ((made) => ((value) => (modeOf(chart, item) === 1 && (!(value > 0) || (made?.low && !(made.low[i] > 0))) ? NaN : value))(made?.shown?.[i] ?? NaN))(built.get(item.id));
  const layer = placing?.host ?? [layerById(selected), ...chart.layers].find((item) => item && !item.hidden && !placedOn(item) && drawn.get(item.id) && Number.isFinite(at(item)));
  const value = !layer ? NaN : tool && keyValue && keyValue.layer === layer.id ? keyValue.value : Number.isFinite(at(layer)) ? at(layer) : placing ? (draft?.points.at(-1)?.[1] ?? NaN) : NaN;
  return layer && Number.isFinite(value) ? { layer, value } : null;
}
const amountOn = (/** @type {Layer} */ layer, /** @type {number} */ value) => format(value, unitFor(layer, resolved?.results.get(layer.id)));
/** @param {Layer} layer */
const paneRect = (layer) => paneBox(chart.panes.findIndex((pane) => pane.id === layer.pane)) ?? undefined;
/**
 * The legend's height, not a share of the pane (on a log scale a span of decades would grow with it); a little under
 * the lines, none under a stack: it sits on its floor.
 * @param {Chart} doc @param {Pane} pane @param {number} i
 */
function marginsOf(doc, pane, i) {
  const height = engine.panes[i]?.height || chartHeight / Math.max(1, doc.panes.length) || 400;
  const legend = /** @type {HTMLElement | null} */ ($("chart").querySelector(`.legend[data-pane="${pane.id}"]`));
  // A pane new to this build has no legend yet, placed a frame later: the room one took, every legend being one row, so
    // its lines don't start under it and move down.
  if (legend) legendRoom = legend.offsetTop + legend.offsetHeight + 10;
  const room = doc.legend === "off" ? 14 : legendRoom;
  return { top: clamp(room / height, 0.03, exporting ? 0.6 : 0.35), bottom: drawnInto(doc, pane) ? 0 : clamp(14 / height, 0.02, 0.08) };
}
let legendRoom = 40;
// Pane separators resize panes (the library's own drag); their sizes stay with the chart, kept as a press ends (a
// moment later: the library's last move in). Only the open chart's, drawn, no look on it.
const keepPaneSizes = () => setTimeout(() => {
  const panes = engine.panes, laid = panesLaid.factors;
  if (lookDoc || !settled() || panesLaid.doc !== chart || panes.length !== chart.panes.length || panes.length !== laid.length || panes.some((pane, i) => pane !== panesLaid.panes[i])) return;
  if (panes.every((pane, i) => Math.abs(pane.share - laid[i]) <= 1e-6)) return;
  const hidden = hiddenPanes(chart);
  if (chart.panes.some((pane, i) => hidden.has(pane.id) && Math.abs(panes[i].share - laid[i]) > 1e-6)) return sizePanes(chart, true);
  // To a thousandth: laid on again, a rounder size would move the separator a pixel from where it was let go.
  const sizes = chart.panes.map((_, i) => Math.round(panes[i].share * 1000) / 1000);
  if (sizes.every((size, i) => hidden.has(chart.panes[i].id) || size === Math.round((chart.panes[i].size ?? defaultSize(i)) * 1000) / 1000)) return;
  commit((doc) => doc.panes.forEach((pane, i) => !hidden.has(pane.id) && (pane.size = sizes[i])), { quiet: true });
  // Laid on as kept: what an undo puts back is then another size, laid on in its turn.
  sizePanes(chart);
  fitMargins();
}, 0);

/** Once on the next frame however many moves ask in it: written per move, they'd lay the page out again for each. */
let valuesFrame = 0;
const valuesSoon = () => (valuesFrame ||= requestAnimationFrame(() => ((valuesFrame = 0), updateValues())));
/**
 * A wheel's (a trackpad's) end, unsaid by the browser: none for this long. Its momentum comes as more wheel events, a
 * frame or two apart; a mouse's notches, a little more.
 */
const WHEEL_QUIET = 150;
/** @returns {{ value: number, change: number, date: number, dated: boolean }} */
function figuresRoom(/** @type {Layer} */ layer, /** @type {Result} */ result) {
  if (layer.hidden || result.error) return { value: 0, change: 0, date: 0, dated: false };
  if (heatLike(layer)) {
    const value = valueSlot(layer, rowSizes.value, true) + 10;
    return { value, change: 0, date: layer.cumulative ? 0 : Math.max(0, widest(binnedAt(layer).map(heatmaps.placeSaid), 400, 11) + 10 - value), dated: !layer.cumulative };
  }
  const lines = layer.type === "cloud" && Boolean(result.low), value = valueSlot(layer, rowSizes.value, lines);
  const rest = { at: lastShown(), rest: true }, since = Number.isFinite(valueAt(layer, rest)) || result.low ? asOf(layer, rest) : "", measured = slotsNow();
  if (since && !measured.has(`asOf:${since}`)) measured.set(`asOf:${since}`, widest([since], 400, 11));
  const date = since ? /** @type {number} */ (measured.get(`asOf:${since}`)) : 0;
  const chip = layer.type === "cloud" || layer.type === "signal" ? 0 : chipSlot(layer), change = Math.max(0, chip - value);
  return { value: value + 10, change, date: !date ? 0 : chip || lines ? date + 6 : Math.max(0, date - value - change), dated: Boolean(date) };
}
function chipSlot(layer) {
  const span = changeWindow(layer);
  if (!span) return 0;
  const key = `chip:${layer.id}:${span.unit}:${span.a}:${span.last}:${rowSizes.change}`, measured = slotsNow();
  if (measured.has(key)) return /** @type {number} */ (measured.get(key));
  const { values, a, base, lo, hi, relative, unit } = extremes(span);
  const from = values[relative ? base : a];
  const texts = [lo, hi].map((to) => chipText(change(from, to, relative, unit)));
  if (relative) texts.push("↓ 00.0%", "↑ 00.0%", "↑ 000%");
  const width = widest(texts, 510, rowSizes.change) + 8;
  measured.set(key, width);
  return width;
}

/** @template {{ values: ArrayLike<number>, a: number, last: number, lo: number, hi: number }} T @param {T} span @returns {T} */
function extremes(span) {
  if (!Number.isNaN(span.lo)) return span;
  let lo = Infinity, hi = -Infinity;
  for (let i = span.a; i <= span.last; i++) {
    const value = span.values[i];
    if (!Number.isFinite(value)) continue;
    if (value < lo) lo = value;
    if (value > hi) hi = value;
  }
  span.lo = lo;
  span.hi = hi;
  return span;
}

function overviewSource() {
  if (!resolved || !chart) return null;
  const first = chart.panes[0];
  const valuesOf = (/** @type {Layer} */ item) => (item.type === "signal" ? undefined : resolved?.results.get(item.id)?.values);
  const layer = chart.layers.find((item) => item.pane === first?.id && !item.hidden && valuesOf(item)) ?? chart.layers.find(valuesOf), values = layer && valuesOf(layer);
  return layer && values ? { layer, values, log: modeOf(chart, layer) === 1 } : null;
}
const eased = (/** @type {number} */ t) => 1 - (1 - t) ** 3;
/** The page's wipe curve, cubic-bezier(0.32, 0.72, 0.2, 1), at a share of its time (its time solved by halving). */
function wiped(/** @type {number} */ x) {
  const at = (/** @type {number} */ a, /** @type {number} */ b, /** @type {number} */ t) => 3 * a * t * (1 - t) ** 2 + 3 * b * t * t * (1 - t) + t ** 3;
  let lo = 0, hi = 1, t = x;
  for (let i = 0; i < 20; i++) {
    t = (lo + hi) / 2;
    if (at(0.32, 0.2, t) < x) lo = t;
    else hi = t;
  }
  return at(0.72, 1, t);
}
function overviewSpan(/** @type {number} */ n) {
  const t = morphFrom ? Math.min(1, (performance.now() - morphFrom.at) / MOTION.page) : 1;
  if (!morphFrom || t >= 1) return { from: 0, to: n - 1, t: 1 };
  const e = eased(t);
  return { from: morphFrom.from * (1 - e), to: morphFrom.to + (n - 1 - morphFrom.to) * e, t: e };
}
function placeBrush() {
  const brush = $("brush");
  const n = resolved?.times.length ?? 0;
  const range = shownSpan();
  if (!range || n < 2 || !chart?.layers.length) {
    brush.hidden = true;
    paintNavigator(null);
    return;
  }
  brush.hidden = false;
  // In shares of the navigator's width, placed on each move without reading the layout.
  const span = overviewSpan(n), share = (/** @type {number} */ at) => clamp((at - span.from) / Math.max(1, span.to - span.from), 0, 1);
  const left = share(range.from), right = share(range.to);
  brush.style.setProperty("--from", `${left * 100}%`);
  brush.style.setProperty("--to", `${right * 100}%`);
  paintNavigator({ left, right });
}
const chartWidth = () => engine.time.width + engine.left + engine.axisShown;
const pressed = new Set(), fingers = new Set();
/** @param {Span} span @param {{ from: number, to: number }} range @param {number} n */
function nearEnd(span, range, n) {
  const near = Math.max(CHUNK, (range.to - range.from) / 2);
  const floor = span.floor && span.floor.of === drawSeries(chart).sort().join() ? span.floor.at : 0;
  return range.from < near && span.from > floor ? -1 : range.to > n - 1 - near && !span.live ? 1 : 0;
}
function prefetch() {
  // (The library frames from the page's start, an empty one included; the chart is the one the page opens, made as it starts: none yet, nothing to reach for.)
  const span = chart && FINE.has(chart.index) ? spans.get(spanKey(chart.id, chart.index)) : null, n = resolved?.times.length ?? 0;
  const range = shownSpan();
  if (!span || presetShown || !range || n < 2 || !chartShown() || shownIndex() !== chart.index) return;
  const side = nearEnd(span, range, n);
  if (!side) return;
  const k = side < 0 ? span.from - 1 : span.to + 1;
  for (const name of drawSeries(chart)) {
    if (chunked(name, chart.index) && !chunks.has(`${name}@${chart.index}#${k}`)) loadChunk(name, chart.index, k, false);
  }
}
const chunked = (name, index) => !(catalogIndex && !catalogIndex.has(name)) && intervalsOf(name)?.includes(index) !== false;
/**
 * At a fine interval, the view near either end of what's loaded, at rest: drawn mid-gesture, the view would leap. The
 * next chunk out; only where the view is taken (not while a preset shows or a glide runs).
 */
function reachOut() {
  const span = chart && FINE.has(chart.index) ? spans.get(spanKey(chart.id, chart.index)) : null, n = resolved?.times.length ?? 0;
  const range = shownSpan();
  if (!span || reaching || viewMoving || pressed.size || navBar.dragging || presetShown || engine.controller.target || !settled() || !range || n < 2 || Date.now() < (span.rest ?? 0)) return;
  const side = nearEnd(span, range, n), near = Math.max(CHUNK, (range.to - range.from) / 2);
  // Holding the most, growing at one end lets go of the other: not when the view would then be close to that one
  // (it would grow back there, and on, to and fro).
  if (span.to - span.from + 1 >= MOST_CHUNKS && (side < 0 ? range.to > n - 1 - near - CHUNK : range.from < near + CHUNK)) return;
  if (side) void grow(span, side);
}
function crossAt() {
  const pinned = crossSet && drawn.get(crossSet.layer), y = pinned && crossSet ? seriesY(pinned, crossSet.price) : null;
  return crossSet ? (pinned && y !== null ? { pane: seriesPane(pinned), y } : null) : heatAt;
}
export {
  alpha, amountOn, applyRange, asked, atLatest, autoScale, bandAt, baseOf, bubbleAt, Bubbles,
  build, built, BY_BLOCK, byColor, candleColumns, cardsDrawn, changeOver, chartFacts, chartPoint, chartTheme,
  chartWidth, chipText, chunkAt, chunked, chunks, Cloud, colorChain, colorNow, colorsAt, columnBy, crossHeld, crossOn,
  crossSet, cssColor, cutsOf, dailyOf, dashFor, dataKey, datedIntervals, defaultSize, describePlot, dotLineOf,
  dotRadius, dotted, drawLayers, drawn, drawnWaiters, drawRows, drawSeries, dropCross, edges, endReveal, EXACT_RANGE, fadedNow,
  fadeLayer, failedChunks, failure, figuresRoom, fillBox, fineDrawn, fingerCross, fingers, firstInView, fitDots,
  fitSeries, fitSoon, fitSpacing, fitStrip, foldedInto, foldOf, hiddenPanes, layerShows, paneShows, forget, forgetSpans, gapped, glide,
  glideRows, glides, gone, Heat, hexCache, holdCross, idOf, indexOfTime, info, ink, inReveal, inSubmenu, intervalsOf,
  keepPaneSizes, keyPoint, known, lastShown, lastUsed, latestChunk, layerFormats, layerOfSeries, legends,
  legendShown, levelOf, levelsKept, levelTags, lightBubble, lightLegend, lightRows, litLayer, liveColumn, load, loads, lookKey,
  mapResult, marginsOf, markLegends, markSignals, menuPicks, MIN_BAR_SPACING, nameAxis, namePointed, NO_FORMAT,
  noValues, oklchOf, overviewSource, overviewSpan, paintedLayers, paintOf, paintPoint, paintProfileItems, paintSeries,
  paintSig, paletteOf, PANE, paneBox, paneRect, PERCENT_FORMAT, periodAmount, periodOf, pinCross, placeBrush,
  placeFoot, placeFull, placePlotKeys, pointLabel, prefetch, presetFits, presetShown, presetSpan, presetWindow,
  pressed, PROFILE_DIM, profiled, rangeAt, rangeText, reachOut, readPoint, refreshFailed, rescan, resolve, respot, retryInfo, reveal, rightGap, setReserve,
  samePaint, sayValue, SCALE_MODES, seriesIn, seriesPane, seriesScale, seriesValue, seriesY, setColumn, setGap,
  setPresetShown, settled, Shading, SHARE_FORMAT, showDates, shownDates, shownIndex, shownRange, shownSpan, showSpan,
  showsPreset, showTimes, SIDES, sizeKeeping, sizePanes, spanned, spanOf, spans, stackBands, stale, Standard,
  stretchAcross, stretchDrawn, stretchNow, strip, styleSeries, subAt, tagAt, unitFor, units, updateLegends,
  updateReadout, updateValues, valueAt, valueFormatOf, valueSlot, valuesSoon, viewName, WHEEL_QUIET,
  whenSubmenu, windows, writeText, zoom, crossAt,
};
