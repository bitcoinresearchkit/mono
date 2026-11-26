// @ts-check
// Heatmaps (the URPD, a day a column) and bubbles.
import { channels, columnWidth, Custom, placeIndex, press, Profile } from "/modules/bitview-charts/index.js";
import {
  boxing, capturing, chart, chooseDrawings, chosen, client, cohortColor, cohortName, cohortShort, commit, css, draft,
  edits, engine, exporting, gesturePointer, HEAT_AGES, HEAT_COHORTS, HEAT_HOLDERS, heatCohort, heatKnown, keepDrawing,
  keyTime, lensAt, light, moving, notice, picks, placing, profileColor, profileLit, renderBelt, reshape, reshaped,
  resolved, startLabel, startTyping, syncUndo, takeUp, toolKept, undoToast, writing, keepFocus, logRows, placingPath,
} from "/studio/app.js";
import { $, escapeHtml, TOUCH, isOpen, onClosed } from "/studio/dom.js";
import { clamp, lowerBound, uid } from "/studio/math.js";
import { until } from "/studio/timing.js";
import { format, plural } from "/studio/format.js";
import { CHEVRON, ICONS } from "/studio/icons.js";
import {
  belowOf, DEFAULT_MEASURE, heatGradient, DEFAULT_ROWS, HEAT_MEASURES, HEAT_ROWS, HEAT_WEIGHTS, heatLike, measureOf, measureUnit,
  PROFILE_VIEWS,
} from "/studio/vocabulary.js";
import { BOXES, LINES, MAX_DRAWINGS, POINTED, pointOf, TEXT_SIZE, THROUGH, TOOLS } from "/studio/tools.js";
import {
  alpha, cssColor, drawn, fingerCross, glide, Heat, ink, lastShown, paneBox, PROFILE_DIM, seriesPane, seriesValue,
  seriesY, shownSpan, valuesSoon,
} from "/studio/chart.js";
import { bothOf, choicePage } from "/studio/command.js";
import {
  drawingById, drawingNames, plotPoint,
  pointFrom, shareAlong, timeOfBar, } from "/studio/drawing.js";
import { editById, } from "/studio/inspector.js";
import { menuOf } from "/studio/menus.js";
import { refocus } from "/studio/presets-edit.js";
import { measureName, weightName, layerById, modeOf, gradientTable } from "/studio/looks.js";
import { barOf, boundsOf, finite, labelPlace, lineThrough, onScreen, ownLayer, plotBox, repaintDrawings, sketches, textPlace, timeLine, toSegment, wordsColor } from "/studio/sketch.js";
/** @import { ChartStep } from "/studio/app.js" */
/** @import { Chart, Drawing, DrawingKind, Layer } from "/studio/vocabulary.js" */
/** @import { ChartPoint, ReadPoint, Resolved } from "/studio/chart.js" */
/** @import { Action, Page } from "/studio/command.js" */
/** @import { Vector } from "/studio/formulas.js" */

/**
 * @param {Chart} doc
 * @param {Resolved} data
 * @param {Layer} layer
 * @param {Map<string, { values: Vector }>} stacked
 * @returns {[Float64Array, Float64Array]}
 */
function bubbleColumns(doc, data, layer, stacked) {
  const own = /** @type {Vector} */ (data.results.get(layer.id)?.values), on = doc.layers.find((other) => other.id === layer.ref);
  const at = on && (stacked.get(on.id)?.values ?? data.results.get(on.id)?.values), log = Boolean(on) && modeOf(doc, /** @type {Layer} */ (on)) === 1;
  const n = data.times.length, v = new Float64Array(n), y = new Float64Array(n);
  for (let i = 0; i < n; i++) (v[i] = own?.[i] ?? NaN), (y[i] = !on ? 0.5 : ((value) => (Number.isFinite(value) && (!log || value > 0) ? value : NaN))(at?.[i] ?? NaN));
  return [v, y];
}
/** @param {Vector} values @param {{ from: number, to: number } | null} [range] */
const mostIn = (values, range = { from: 0, to: values.length - 1 }) => {
  let most = 0;
  // An infinite value is none, as NaN is: else every other one would read as none beside it.
  if (range) for (let i = Math.max(0, Math.ceil(range.from)); i <= Math.min(values.length - 1, Math.floor(range.to)); i++) if (Math.abs(values[i]) > most && Math.abs(values[i]) < Infinity) most = Math.abs(values[i]);
  return most;
};

const PROFILE_ABOUT = "Holder groups' supply, capital or unrealized P&L by the price it last moved at, on the view's last day, as bars at the chart's right edge: each group its color, under the price in profit.";
const PROFILE_SHORT = "Holder groups' cost basis, at the right edge";
const HEAT_ABOUT = "A holder group's supply, capital or unrealized P&L by the price it last moved at, day by day, behind the price (the URPD).";
const HEAT_SHORT = "Holder groups' cost basis, day by day";
/** @param {Chart} doc @param {Layer} layer */
const heatPath = (doc, layer) => `/api/urpd/${encodeURIComponent(layer.heatmap ?? "all")}?agg=${heatGrid(doc, layer)?.agg ?? `log${rowsOf(layer)}`}${layer.weight ? `&weight=${layer.weight}` : ""}`;
const HEAT_SAYS = /** @type {Record<string, string>} */ ({ supply: "Bitcoin held, by the price it last moved at", capital: "What it cost, by that price", pnl: "Its gain or loss now, by that price", "": "Each coin counted as it is", wakefulness: "Each age band by how much of its coin days are spent (Cointime)", mobility: "Each age band by how likely its coins are to move (Coinflow)", bars: "Each price's amount", cumulative: "Summed from the close outward: at each price, all that last moved between it and the close" });
/** @param {string} id @param {string} cohort */
function toggleCohort(id, cohort) {
  if (layerById(id)?.type !== "profile") return;
  editById(id, (item) => (item.profile = item.profile?.includes(cohort) ? item.profile.filter((other) => other !== cohort) : [...(item.profile ?? []), cohort]));
  if (!isOpen($("palette"))) refocus(`#inspector [data-cohort-of="${CSS.escape(cohort)}"]`);
}
/** @param {Layer} layer */
const profileLists = (layer) => ({ holders: [...HEAT_HOLDERS, ...(layer.profile ?? []).filter((id) => !heatCohort(id))], ages: HEAT_AGES });
/** @param {string} id @param {"holders" | "ages"} list @param {boolean} on */
function setCohorts(id, list, on) {
  const layer = layerById(id);
  if (layer?.type !== "profile") return;
  const ids = profileLists(layer)[list].filter((cohort) => !on || heatCohort(cohort));
  editById(id, (item) => (item.profile = on ? [...(item.profile ?? []), ...ids.filter((cohort) => !item.profile?.includes(cohort))] : (item.profile ?? []).filter((cohort) => !ids.includes(cohort))));
  refocus(`#inspector [data-cohorts-${on ? "in" : "out"}="${list}"]`);
}
/** @param {string} id @returns {Page} */
const cohortsPage = (id) => ({ title: "Holders", placeholder: "Search the holder groups", rows: () => {
  const layer = layerById(id), groups = layer?.profile ?? [];
  const rows = [...HEAT_COHORTS.map((c) => [c.id, c.shown, c.age ? "Age bands" : "Holder groups"]), ...groups.filter((cohort) => !heatCohort(cohort)).map((cohort) => [cohort, `${cohort}: not on Bitview`, "Not on Bitview"])];
  return rows.map(([cohort, name, group]) => {
    const on = groups.includes(cohort);
    return { kind: "row", key: cohort, icon: `<i class="swatch" style="--c: ${escapeHtml(cssColor(layer ? profileColor(layer, cohort) : cohortColor(cohort)))}"></i>`, label: name, aside: on ? "In it" : "", current: on, verb: on ? "Deselect" : "Select", stay: true, group, run: () => toggleCohort(id, cohort) };
  });
} });
/** @param {string} id */
function heatRows(id) {
  const both = bothOf(id), layer = /** @type {Layer} */ (layerById(id)), kind = layer.type === "profile" ? "Profile" : "Heatmap", now = () => layerById(id) ?? layer, grid = heatGrid(chart, layer);
  const choices = (/** @type {string} */ title, /** @type {[any, string, ...any[]][]} */ list, /** @type {(item: Layer) => any} */ value, /** @type {(item: Layer, choice: any) => void} */ apply, /** @type {(choice: any) => string | undefined} */ says = () => undefined) => () => choicePage(title, () => list.map(([choice, text]) => ({ key: String(choice), label: text, aside: says(choice), current: value(now()) === choice, ...both((item) => apply(item, choice)) })));
  return [
    layer.type === "profile"
      ? { key: "holders", icon: ICONS.people, label: "Holders", aside: ((groups) => (groups.length > 2 ? plural(groups.length, "group") : groups.map(cohortName).join(", ") || "None"))(layer.profile ?? []), group: "Profile", leads: true, page: () => cohortsPage(id) }
      : { key: "holders", icon: ICONS.people, label: "Holders", aside: cohortName(layer.heatmap ?? "all"), group: "Heatmap", leads: true, page: choices("Holders", HEAT_COHORTS.map((c) => /** @type {[string, string]} */ ([c.id, c.shown])), (item) => item.heatmap ?? "all", (item, cohort) => (item.heatmap = cohort)) },
    ...(layer.type === "profile" ? [{ key: "view", icon: ICONS.layers, label: "View", aside: layer.cumulative ? "Cumulative" : "Bars", group: kind, page: choices("View", PROFILE_VIEWS, (item) => (item.cumulative ? "cumulative" : "bars"), (item, view) => (item.cumulative = view === "cumulative" || undefined), (view) => HEAT_SAYS[view]) }] : []),
    { key: "measure", icon: ICONS.coins, label: "Measure", aside: measureName(layer), group: kind, page: choices("Measure", HEAT_MEASURES, measureOf, (item, measure) => reshape(item, (one) => (one.measure = measure === DEFAULT_MEASURE ? undefined : measure)), (measure) => HEAT_SAYS[measure]) },
    { key: "weight", icon: ICONS.hourglass, label: "Weight", aside: weightName(layer), group: kind, page: choices("Weight", HEAT_WEIGHTS, (item) => item.weight ?? "", (item, weight) => (item.weight = weight || undefined), (weight) => HEAT_SAYS[weight]) },
    grid
      ? { key: "density", icon: ICONS.layers, label: "Density", aside: `${rowsOf(layer)} rows per ×10`, group: kind, page: choices("Density", HEAT_ROWS.map((n) => /** @type {[number, string]} */ ([n, `${n} rows per ×10`])), (item) => rowsOf(item), (item, n) => (item.rows = n === DEFAULT_ROWS ? undefined : n)) }
      : { key: "density", icon: ICONS.layers, label: "Density", aside: "Not on this scale", disabled: "On a log or linear scale", group: kind },
  ];
}
/** @param {Layer} layer */
function heatSection(layer) {
  const rows = heatRows(layer.id);
  const menu = (/** @type {string} */ key) => ((row) => `<div class="field"><span>${escapeHtml(row.label)}</span><button type="button" class="preset" data-action="heat-menu" data-heat="${key}" aria-haspopup="menu" aria-label="${escapeHtml(`${row.label}: ${row.aside ?? ""}`)}"${row.disabled ? ` aria-disabled="true" data-tip="${escapeHtml(row.disabled)}"` : ""}><span class="name">${escapeHtml(row.aside ?? "")}</span>${CHEVRON}</button></div>`)(/** @type {any} */ (rows.find((row) => row.key === key)));
  const tabs = (/** @type {string} */ key, /** @type {string} */ label, /** @type {string[][]} */ list, /** @type {string} */ now) => `<div class="field"><span>${label}</span><div class="tabs" role="group" aria-label="${label}">${list.map(([value, text]) => `<button type="button" data-heat-set="${key}" data-value="${value}" aria-pressed="${now === value}" data-tip="${escapeHtml(HEAT_SAYS[value])}">${escapeHtml(text)}</button>`).join("")}</div></div>`;
  const gone = !heatKnown(layer) ? '<div class="error-box gone"><p>Bitview has no holder group by this name: renamed, removed, or mistyped. Pick another above.</p></div>' : "";
  const groups = layer.profile ?? [], lists = profileLists(layer);
  const chips = (/** @type {"holders" | "ages"} */ list, /** @type {string} */ label) => `<div class="from-of ${list}" role="group" aria-label="${label}">${lists[list].map((id) => ((known) => `<button type="button" data-cohort-of="${escapeHtml(id)}" aria-pressed="${groups.includes(id)}"${known ? ` aria-label="${escapeHtml(cohortName(id))}" data-tip="${escapeHtml(cohortName(id))}"` : ""}><i class="swatch" style="--c: ${escapeHtml(cssColor(profileColor(layer, id)))}"></i><span class="name">${escapeHtml(known ? cohortShort(id) : `${id}: not on Bitview`)}</span></button>`)(Boolean(heatCohort(id)))).join("")}</div>`;
  const both = (/** @type {"holders" | "ages"} */ list, /** @type {string} */ name) => ((ids) => `<span class="list-actions"><button type="button" class="cap small words" data-cohorts-in="${list}" aria-label="All ${name}"${ids.every((id) => groups.includes(id) || !heatCohort(id)) ? ' aria-disabled="true"' : ""}>All</button><button type="button" class="cap small words" data-cohorts-out="${list}" aria-label="No ${name}"${ids.some((id) => groups.includes(id)) ? "" : ' aria-disabled="true"'}>None</button></span>`)(lists[list]);
  return `<section>
              <h3>${layer.type === "profile" ? "Profile" : "Heatmap"}</h3>
              ${layer.type === "profile" ? `<div class="field"><span>Holders<span class="color-now">${groups.length ? plural(groups.length, "group") : "None"}</span>${both("holders", "holder groups")}</span>${chips("holders", "Its holder groups")}</div>
              <div class="field"><span>Age bands${both("ages", "age bands")}</span>${chips("ages", "Its age bands")}</div>` : menu("holders")}${gone}
              ${layer.type === "profile" ? tabs("view", "View", PROFILE_VIEWS, layer.cumulative ? "cumulative" : "bars") : ""}
              ${tabs("measure", "Measure", HEAT_MEASURES.map(([id, , short]) => [id, short]), measureOf(layer))}
              ${tabs("weight", "Weight", HEAT_WEIGHTS, layer.weight ?? "")}
              ${menu("density")}
            </section>`;
}
const HEAT_COLUMN = 2;
/** @param {Layer} layer */
const rowsOf = (layer) => layer.rows ?? DEFAULT_ROWS;

/** @param {Chart} doc @param {Layer} layer */
function heatGrid(doc, layer) {
  const mode = modeOf(doc, layer), n = rowsOf(layer);
  if (mode > 1) return null;
  return { agg: `log${n}`, even: mode === 1, ...logRows(n) };
}
/** @param {HeatDay & { rows: Int32Array }} day @param {number} row @param {string} measure */
function dayCell(day, row, measure) {
  const k = lowerBound(day.rows, row);
  return { row, amount: day.rows[k] === row ? /** @type {Float32Array} */ (amountsOf(day, measure))[k] : 0 };
}
/** @param {HeatDay & { rows: Int32Array }} day @param {ReturnType<typeof logRows>} grid @param {string} measure */
function closeCell(day, grid, measure) {
  const close = Number(day.close);
  return close > 0 ? dayCell(day, grid.cell(close), measure) : null;
}
/** @param {number} change */
const fromClose = (change) => ((text) => (text.startsWith("−") ? text : `+${text}`))(format(change, "percent"));
/** @param {{ row: number, amount: number }} cell @param {ReturnType<typeof logRows>} grid @param {string} measure */
const cellText = (cell, grid, measure) => ({ amount: format(cell.amount, measureUnit(measure)), at: format(grid.price(cell.row), "usd") });
/** @param {number} from @param {number} to */
const priceSpan = (from, to) => `${format(from, "usd")}–${format(to, "usd")}`;
/** @param {string} at */
const placeSaid = (at) => `at ${at}`;
/** @param {{ amount: string, at: string }} reading */
const readingLine = (reading) => (reading.at ? `${reading.amount} ${placeSaid(reading.at)}` : reading.amount);
/**
 * The most recently drawn are kept, up to a cell budget (never those in view).
 * @typedef {{ pending?: true, failed?: true, retryAt?: number, rows?: Int32Array, supply?: Float32Array, capital?: Float32Array, pnl?: Float32Array, close?: number, used: number }} HeatDay
 * @type {Map<string, HeatDay>}
 */
const heatDays = new Map();
let heatCells = 0, heatUse = 0;
const HEAT_MOST_CELLS = 3_000_000, HEAT_AT_ONCE = 6, HEAT_RETRY = 30_000;
/** @type {Map<string, { key: string, cohort: string, weight: string, agg: string, day: string, rank: number }[]>} */
const heatWants = new Map();
let heatAsking = 0, heatPumping = false;
/** @type {Set<Heat>} */
const heatSeries = new Set();
let heatFrame = 0;
const heatSoon = () => void (heatFrame ||= requestAnimationFrame(() => {
  heatFrame = 0;
  for (const series of heatSeries) series.redraw();
  valuesSoon();
}));
/** @type {Set<() => void>} */
const heatWaiters = new Set();
// A tab drawing no frames draws none, so asks for none: as it is. One with no price to draw on, or `undrawable` (the library had nothing on its scale to draw it on), is as it is.
const heatIn = () => document.hidden || shownHeats().every((look) => (look.undrawable && !look.drawn) || (look.drawn && !heatWants.has(look.id) && ![...(look.keep ?? [])].some((key) => heatDays.get(key)?.pending)));
/** @type {Set<{ wants: NonNullable<ReturnType<typeof heatWants.get>>, keys: Set<string> }>} */
const heatExtras = new Set();
const heatCome = (/** @type {string[]} */ keys, /** @type {number} */ ms, /** @type {() => boolean} */ still) => until(() => keys.every((key) => heatDays.get(key) && !heatDays.get(key)?.pending) || !still(), ms, 100);
const shownHeats = () => chart.layers.flatMap((layer) => ((look) => (heatLike(layer) && !layer.hidden && look && resolved?.results.get(layer.id)?.closes?.some(Number.isFinite) ? [look] : []))(heatLooks.get(layer.id)));
function heatPump() {
  heatPumping = false;
  const wanted = [...heatWants.values(), ...[...heatExtras].map((extra) => extra.wants)].flat().filter((want) => !heatDays.has(want.key)).sort((a, b) => b.rank - a.rank || b.day.localeCompare(a.day));
  for (const want of wanted) {
    if (heatAsking >= HEAT_AT_ONCE) break;
    if (heatDays.has(want.key)) continue;
    const asked = { pending: /** @type {const} */ (true), used: ++heatUse };
    heatDays.set(want.key, asked);
    heatAsking++;
    const weight = /** @type {any} */ (want.weight || undefined), agg = /** @type {any} */ (want.agg);
    (want.day === "latest" ? client.getUrpdLatest(/** @type {any} */ (want.cohort), { agg, weight }) : client.getUrpd(/** @type {any} */ (want.cohort), want.day, { agg, weight }))
      .then((urpd) => {
        if (heatDays.get(want.key) !== asked) return;
        const buckets = /** @type {any} */ (urpd).buckets, n = buckets.price.length;
        const rows = Int32Array.from(buckets.price, logRows(Number(want.agg.slice(3))).floor);
        heatDays.set(want.key, { rows, supply: Float32Array.from(buckets.supply), capital: Float32Array.from(buckets.realizedCap), pnl: Float32Array.from(buckets.unrealizedPnl), close: /** @type {any} */ (urpd).close, used: ++heatUse });
        heatCells += n;
        heatTrim();
      }, (/** @type {any} */ error) => {
        if (heatDays.get(want.key) !== asked) return;
        heatDays.set(want.key, { failed: true, ...(error?.status === 404 ? {} : { retryAt: Date.now() + HEAT_RETRY }), used: ++heatUse });
        if (error?.status !== 404) setTimeout(heatSoon, HEAT_RETRY);
      })
      .finally(() => {
        heatAsking--;
        heatSoon();
        heatPump();
      });
  }
}
function heatFresh() {
  const today = new Date().toISOString().slice(0, 10);
  for (const [key, day] of heatDays) {
    const date = key.slice(key.lastIndexOf("|") + 1);
    if (date !== "latest" && date < today) continue;
    heatDays.delete(key);
    dropStrips(key);
    heatCells -= day.rows?.length ?? 0;
  }
  heatSoon();
}
/** @param {string} key */
function dropStrips(key) {
  for (const series of drawn.values()) if (series instanceof Heat) series.grid?.forget(key);
}
function heatTrim() {
  if (heatCells <= HEAT_MOST_CELLS) return;
  const shown = new Set([...[...heatLooks.values()].flatMap((look) => [...(look.keep ?? [])]), ...[...heatExtras].flatMap((extra) => [...extra.keys])]);
  const done = [...heatDays].filter(([key, day]) => day.rows && !shown.has(key)).sort((a, b) => a[1].used - b[1].used);
  for (const [key, day] of done) {
    if (heatCells <= HEAT_MOST_CELLS * 0.8) break;
    heatDays.delete(key);
    dropStrips(key);
    heatCells -= /** @type {Int32Array} */ (day.rows).length;
  }
}
const HEAT_DECADES = 4;
// Cells in view are mostly within a decade or two of the strongest: a straight scale lit nearly all of them (nothing standing out); this leaves the weaker in the dark and the clusters lit (measured: 2 left the bed muddy, 3 lost it).
const HEAT_CURVE = 2.5;
const HEAT_FADE = 0.3;
/** @param {number} value @param {number} most */
const heatFelt = (value, most) => (value && most ? clamp(1 + Math.log10(Math.abs(value) / most) / HEAT_DECADES, 0, 1) ** HEAT_CURVE : 0);
/** @param {[number, string][]} stops */
const gradientChannels = (stops) => gradientTable(stops).map(channels);
/** @param {number[][]} table @param {number} felt @param {number} opacity @param {Uint8ClampedArray} pixels @param {number} at */
function gradientCell(table, felt, opacity, pixels, at) {
  const [r, g, b] = table[placeIndex(felt, table.length)];
  pixels[at] = r;
  pixels[at + 1] = g;
  pixels[at + 2] = b;
  pixels[at + 3] = Math.round(255 * opacity * Math.min(1, felt / HEAT_FADE));
}
/** @param {any} day @param {string} measure @returns {Float32Array} */
const amountsOf = (day, measure) => (measure === "capital" ? day.capital : measure === "pnl" ? day.pnl : day.supply);
/** @param {any} day @param {string} measure */
const heatMost = (day, measure) => (day.most ??= {})[measure] ??= amountsOf(day, measure).reduce((most, value) => Math.max(most, Math.abs(value)), 0);
/**
 * Strength on a log scale over `HEAT_DECADES`, felt as `HEAT_CURVE` says, against the map's reference (the open day's largest cell, the same for every day and view), not each day's own: a day with one big cluster dimmed its whole column, and earlier eras' smaller amounts must still read.
 * @param {number} value
 * @param {number} most
 * @param {{ up: number[], down: number[], opacity: number, gradient: number[][] | null }} look
 * @param {Uint8ClampedArray} pixels
 * @param {number} at
 */
function heatColor(value, most, look, pixels, at) {
  const felt = heatFelt(value, most);
  if (!felt) return;
  if (look.gradient) return gradientCell(look.gradient, felt, look.opacity, pixels, at);
  const [r, g, b] = value < 0 ? look.down : look.up;
  pixels[at] = r;
  pixels[at + 1] = g;
  pixels[at + 2] = b;
  pixels[at + 3] = Math.round(255 * look.opacity * felt);
}
/** @typedef {{ id: string, cohort: string, weight: string, grid: ReturnType<typeof heatGrid>, g: Int32Array, days: string[], measure: string, up: number[], down: number[], opacity: number, gradient: number[][] | null, paint: string, kind: string, cumulative: boolean, cohorts: ReturnType<typeof profileShades>[], last: Map<string, { key: string, day: HeatDay & { rows: Int32Array } }>, gain: string, lose: string, halo: string, backdrop: string, mark: string, profit?: { grid: ReturnType<typeof heatGrid>, close: number, days: (HeatDay | undefined)[], share: number | null }, shares?: { close: number, share: number }, drawn?: boolean, undrawable?: boolean, step?: number, keep?: Set<string>, reference?: number }} HeatLook */
/** @type {Map<string, HeatLook>} */
const heatLooks = new Map();
/** @type {WeakMap<number[], { g: Int32Array, days: string[] }>} */
const heatCalendars = new WeakMap();
/**
 * The last bar, the open one, is "latest" where the timeline reaches the latest (`live`), else its own date, a step on from its start (a chunk of history ends in the past).
 * @param {{ times: number[], live?: boolean }} data
 */
function heatCalendar({ times, live = true }) {
  const had = heatCalendars.get(times);
  if (had) return had;
  const g = new Int32Array(times.length), days = /** @type {string[]} */ ([]);
  for (let i = 0; i < times.length; i++) {
    const end = times[i + 1] ?? (live ? undefined : 2 * times[i] - (times[i - 1] ?? times[i]));
    const day = end === undefined ? "latest" : new Date((end - 1) * 1000).toISOString().slice(0, 10);
    if (day !== days.at(-1)) days.push(day);
    g[i] = days.length - 1;
  }
  const made = { g, days };
  heatCalendars.set(times, made);
  return made;
}
/** @param {Chart} doc @param {Layer} layer @param {Resolved} data @param {number} opacity */
/** @param {string} id @param {string} color @param {number} opacity */
const profileShades = (id, color, opacity) => {
  const card = channels(css("--card")), own = channels(color), over = (/** @type {number} */ strength) => `rgb(${own.slice(0, 3).map((value, i) => Math.round(card[i] + (value - card[i]) * strength)).join(" ")} / ${opacity})`;
  return { id, gain: over(0.72), loss: over(0.32), dimGain: over(0.72 * PROFILE_DIM), dimLoss: over(0.32 * PROFILE_DIM) };
};
const heatLook = (doc, layer, data, opacity) => ({ id: layer.id, cohort: layer.type === "profile" ? layer.profile?.[0] ?? "all" : layer.heatmap ?? "all", weight: layer.weight ?? "", grid: heatGrid(doc, layer), ...heatCalendar(data), measure: measureOf(layer), up: channels(ink(layer.color)), down: channels(ink(belowOf(layer))), opacity, kind: layer.type, cumulative: layer.type === "profile" && Boolean(layer.cumulative), cohorts: layer.type === "profile" ? (layer.profile ?? []).map((id) => profileShades(id, ink(profileColor(layer, id)), opacity)) : [], last: new Map(), gain: ink("green"), lose: ink("red"), halo: css("--card"), backdrop: alpha(css("--card"), opacity), mark: css("--ink"), gradient: ((stops) => (stops ? gradientChannels(stops) : null))(layer.type === "heatmap" && heatGradient(doc, layer)), paint: `${measureOf(layer)}|${opacity}|${layer.type === "heatmap" ? JSON.stringify(heatGradient(doc, layer)) : ""}|${light}` });
/** @param {HeatLook} heat @param {number} g */
const heatOwner = (heat, g, /** @type {number} */ by) => Math.min(Math.ceil(g / by) * by, heat.days.length - 1);
/** @param {HeatLook} heat @param {number} g */
/** @param {string} cohort @param {string} weight @param {string | undefined} agg @param {string | number} day */
const heatDayKey = (cohort, weight, agg, day) => `${cohort}|${weight}|${agg}|${day}`;
const heatKeyOf = (heat, g) => heatDayKey(heat.cohort, heat.weight, heat.grid?.agg, heat.days[g]);
/** @param {string} key */
const heatDayHeld = (key) => {
  if ((heatDays.get(key)?.retryAt ?? Infinity) < Date.now()) heatDays.delete(key);
  return heatDays.has(key);
};
/**
 * Its own day once it's come, else the coarser blocks' over it meanwhile; none (or its own failed), null. Drawn and read alike.
 * @param {HeatLook} heat
 * @param {number} owner
 * @param {number} step
 * @returns {{ key: string, day: HeatDay & { rows: Int32Array } } | null}
 */
function heatShown(heat, owner, step) {
  if (heatDays.get(heatKeyOf(heat, owner))?.failed) return null;
  for (let by = step; by <= step * 16; by *= 2) {
    const key = heatKeyOf(heat, by === step ? owner : heatOwner(heat, owner, by)), day = heatDays.get(key);
    if (day?.rows) return (day.used = ++heatUse), { key, day: /** @type {HeatDay & { rows: Int32Array }} */ (day) };
  }
  return null;
}
/** @param {HeatLook} look @param {Layer} layer @param {ReadPoint} point @returns {{ amount: string, at: string } | null} */
function profileReading(look, layer, point) {
  Object.assign(look, heatCalendar(/** @type {Resolved} */ (resolved)));
  const day = profilePoint().at;
  if (day < 0 || day >= look.g.length || !look.grid) return null;
  const shown = profileDays(look, look.g[day]), close = profileClose(shown), share = close > 0 ? profileShare(look, shown, close) : null;
  if (share === null) return null;
  const at = hoveredIn(layer, point, look.grid);
  return at ? profileAt(look, at) : { amount: `≈${format(share, "percent")} in profit`, at: "" };
}
/** @param {HeatLook} look @param {NonNullable<ReturnType<typeof hoveredIn>>} at */
function profileAt(look, { y, price, row }) {
  const engine = drawn.get(look.id)?.engine, grid = look.grid, unit = measureUnit(look.measure);
  if (!(engine instanceof Profile) || !engine.column || !grid || row < 0) return null;
  const sums = engine.sumsAt(y);
  if (sums) return { amount: `${fromClose((price / engine.at - 1) * 100)} ${format(sums.reduce((sum, value) => sum + value, 0), unit)}`, at: "" };
  const read = engine.readAt(y, litCohort(look.id) || undefined) ?? { from: row, to: row + 1, sum: 0 };
  return { amount: format(read.sum, unit), at: priceSpan(grid.price(read.from), grid.price(read.to)) };
}
/** @param {ChartPoint} param @returns {Layer[]} */
function profilesUnder(param) {
  if (keyTime !== null || !param.point || param.pane === undefined) return [];
  // Left of the widest column a profile draws (a pixel's slack: its bars' edge on the device's pixels, its curves' stroke), none asked.
  const x = param.point.x, plot = engine.time.width;
  if (x < plot - columnWidth(plot) - 1) return [];
  const mine = chart.layers.filter((layer) => {
    if (layer.type !== "profile" || layer.hidden) return false;
    const series = drawn.get(layer.id);
    return series?.engine instanceof Profile && series.engine.covers(x) && seriesPane(series) === param.pane;
  });
  return chart.panes[param.pane]?.reverse ? mine.reverse() : mine;
}
/** @param {Layer} layer @param {{ x: number, y: number }} point */
function profileHit(layer, point) {
  const engine = drawn.get(layer.id)?.engine, hit = engine instanceof Profile ? engine.hit(point.x, point.y) : null;
  return hit && { layer: layer.id, cohort: hit.group };
}
/** @param {string} id */
const litCohort = (id) => ((layer) => (profileLit?.layer === id && layer && !layer.hidden && layer.profile?.includes(profileLit.cohort) ? profileLit.cohort : ""))(layerById(id));

/** @param {Layer} layer @param {ReadPoint} point @param {ReturnType<typeof logRows>} grid */
function hoveredIn(layer, point, grid) {
  const at = point.cross, series = drawn.get(layer.id);
  if (!at || at.pane !== seriesPane(series)) return null;
  const price = Number(seriesValue(series, at.y)), row = price > 0 ? grid.cell(price) : -1;
  return { y: at.y, price, row: Math.max(-1, row) };
}
/** @param {Layer} layer @param {ReadPoint} point @returns {{ amount: string, at: string } | null} */
function heatReading(layer, point) {
  const look = heatLooks.get(layer.id), grid = look?.grid;
  if (!look || !grid || !resolved) return null;
  if (look.kind === "profile") return profileReading(look, layer, point);
  Object.assign(look, heatCalendar(resolved));
  if (point.at < 0 || point.at >= look.g.length) return null;
  const shown = heatShown(look, heatOwner(look, look.g[point.at], look.step ?? 1), look.step ?? 1), day = shown?.day;
  if (!day?.rows) return null;
  const at = hoveredIn(layer, point, grid), cell = !at ? closeCell(day, grid, look.measure) : at.row >= 0 ? dayCell(day, at.row, look.measure) : null;
  return cell && cellText(cell, grid, look.measure);
}
/**
 * @param {HeatLook} heat
 * @param {{ step: number, owners: number[] }} blocks
 * @param {import("/modules/bitview-charts/grid.js").Grid} grid
 */
function viewHeat(heat, { step, owners }, grid) {
  heat.drawn = true;
  if (heatWaiters.size) queueMicrotask(() => { for (const wake of [...heatWaiters]) wake(); });
  if (!heat.grid || !resolved || !owners.length) return void (heatWants.delete(heat.id), (heat.keep = undefined));
  Object.assign(heat, heatCalendar(resolved));
  heat.step = step;
  const ownerAt = (/** @type {number} */ g, /** @type {number} */ by) => heatOwner(heat, g, by), keyOf = (/** @type {number} */ g) => heatKeyOf(heat, g), agg = heat.grid.agg;
  /** @type {Map<string, NonNullable<ReturnType<typeof heatWants.get>>[number]>} */
  const wanted = new Map();
  const keep = new Set();
  const want = (/** @type {number} */ g, /** @type {number} */ rank) => ((key) => {
    keep.add(key);
    if (!heatDayHeld(key) && !wanted.has(key)) wanted.set(key, { key, cohort: heat.cohort, weight: heat.weight, agg, day: heat.days[g], rank });
  })(keyOf(g));
  const open = heatDays.get(keyOf(heat.days.length - 1));
  keep.add(keyOf(heat.days.length - 1));
  if (!open?.rows) want(heat.days.length - 1, Number.MAX_SAFE_INTEGER);
  if (open?.rows) heat.reference = heatMost(open, heat.measure);
  grid.setLook(`${heat.paint}|${heat.reference ?? 0}`);
  for (const owner of owners) {
    const shown = heatShown(heat, owner, step);
    if (shown) keep.add(shown.key);
    if (!heatDays.get(keyOf(owner))?.rows) want(owner, step);
    if (!shown) want(ownerAt(owner, step * 16), step * 16);
  }
  heat.keep = keep;
  if (!wanted.size) return void heatWants.delete(heat.id);
  heatWants.set(heat.id, [...wanted.values()]);
  if (!heatPumping) (heatPumping = true), queueMicrotask(heatPump);
}
const profilePoint = () => ({ at: lastShown(), rest: true });
/** @param {HeatLook} look @param {number} g */
function profileDays(look, g) {
  return look.cohorts.map((cohort) => {
    const id = cohort.id, key = `${id}|${look.weight}|${look.grid?.agg}|${look.days[g]}`, day = heatDays.get(key);
    if (day?.rows) (day.used = ++heatUse), look.last.set(id, { key, day: /** @type {HeatDay & { rows: Int32Array }} */ (day) });
    const had = day?.failed ? null : look.last.get(id);
    return had ? { day: had.day, cohort, key: had.key, now: had.key === key } : null;
  });
}
/** @param {ReturnType<typeof profileDays>} shown */
const profileClose = (shown) => Number((shown.find((item) => item?.now) ?? shown.find(Boolean))?.day.close);
/**
 * A row the close falls in counts as far as the close is into it (its coins taken as even across it): a row doesn't say which of its coins are under the close, and one side by its middle moved the share by points with the density (2024-07-08: 9.3% at 100 rows a ×10, 12.9% at 200, 11.7% at 500; this: 12.3, 12.4, 12.2, the published 11.9). Kept while its days, grid and close are the same.
 * @param {HeatLook} heat @param {ReturnType<typeof profileDays>} shown @param {number} close
 */
function profileShare(heat, shown, close) {
  const grid = /** @type {NonNullable<HeatLook["grid"]>} */ (heat.grid), had = heat.profit;
  if (had && had.grid === grid && had.close === close && had.days.length === shown.length && shown.every((item, k) => item?.day === had.days[k])) return had.share;
  let under = 0, all = 0;
  const at = grid.at(close);
  for (const item of shown) {
    if (!item) continue;
    const rows = item.day.rows, supply = /** @type {Float32Array} */ (item.day.supply);
    for (let k = 0; k < rows.length; k++) (all += supply[k]), (under += supply[k] * Math.min(1, Math.max(0, at - rows[k])));
  }
  const share = all > 0 ? (100 * under) / all : null;
  heat.profit = { grid, close, days: shown.map((item) => item?.day), share };
  return share;
}
/** @param {HeatLook} heat @param {import("/modules/bitview-charts/profile.js").Profile} profile */
function viewProfile(heat, profile) {
  heat.drawn = true;
  if (heatWaiters.size) queueMicrotask(() => { for (const wake of [...heatWaiters]) wake(); });
  const grid = heat.grid;
  // Its shares as drawn, for its axis: where they change, drawn again (the chart paints its axes before its series: theirs read on the next frame).
  const shown = (/** @type {{ close: number, share: number } | undefined} */ next) => {
    const was = heat.shares;
    // As written: a live tick moving the share past what it says draws nothing again.
    const said = (/** @type {{ share: number } | undefined} */ shares) => shares && `${format(shares.share, "percent")} ${format(100 - shares.share, "percent")}`;
    if (was?.close === next?.close && said(was) === said(next)) return;
    heat.shares = next;
    queueMicrotask(() => drawn.get(heat.id)?.redraw());
  };
  /** @param {import("/modules/bitview-charts/profile.js").ProfileGroup[]} groups @param {number} at @param {string | null} closeLine */
  const show = (groups, at = NaN, closeLine = null) => {
    const lit = exporting || capturing ? "" : litCohort(heat.id), had = profile.groups;
    const same = had.length === groups.length && had.every((group, k) => group.rows === groups[k].rows && group.values === groups[k].values && group.gain === groups[k].gain && group.loss === groups[k].loss && group.dimGain === groups[k].dimGain);
    if (same && (profile.at === at || (profile.at !== profile.at && at !== at)) && profile.lit === lit && profile.closeLine === closeLine && profile.backdrop === heat.backdrop && profile.cumulative === heat.cumulative) return;
    profile.set({ groups, at, lit, closeLine, backdrop: heat.backdrop, cumulative: heat.cumulative });
  };
  const none = () => void (heatWants.delete(heat.id), (heat.keep = undefined), shown(undefined), show([]));
  if (!grid || !resolved || !heat.cohorts.length) return none();
  Object.assign(heat, heatCalendar(resolved));
  const point = profilePoint();
  if (point.at < 0 || point.at >= heat.g.length) return none();
  const g = heat.g[point.at], keep = new Set(), wanted = /** @type {NonNullable<ReturnType<typeof heatWants.get>>} */ ([]);
  for (const { id } of heat.cohorts) {
    const key = heatDayKey(id, heat.weight, grid.agg, heat.days[g]);
    keep.add(key);
    if (!heatDayHeld(key)) wanted.push({ key, cohort: id, weight: heat.weight, agg: grid.agg, day: heat.days[g], rank: Number.MAX_SAFE_INTEGER });
  }
  const days = profileDays(heat, g), close = profileClose(days);
  for (const item of days) if (item) keep.add(item.key);
  heat.keep = keep;
  if (wanted.length) (heatWants.set(heat.id, wanted), heatPumping || ((heatPumping = true), queueMicrotask(heatPump)));
  else heatWants.delete(heat.id);
  const share = close > 0 ? profileShare(heat, days, close) : null;
  if (share === null) return void (shown(undefined), show([]));
  shown({ close, share });
  const groups = days.flatMap((item) => (item ? [{ id: item.cohort.id, rows: item.day.rows, values: amountsOf(item.day, heat.measure), gain: item.cohort.gain, loss: item.cohort.loss, dimGain: item.cohort.dimGain, dimLoss: item.cohort.dimLoss }] : []));
  show(groups, close, g === heat.days.length - 1 ? null : alpha(heat.mark, 0.6));
}
const sharesShown = new WeakSet();
/** @param {string} id @param {any} series */
function profileShares(id, series) {
  if (sharesShown.has(series)) return;
  sharesShown.add(series);
  const side = series.side === "left" ? 1 : 0;
  // Over everything: its tags are asked again each time the axes are drawn.
  const tags = new Custom(
    {
      draw: () => {},
      axisTags: (on, add) => {
        const look = heatLooks.get(id), shares = look?.shares, y = shares && seriesY(series, shares.close);
        if (on !== side || series.engine.faint || !shares || y === null || y === undefined) return;
        add(y - 19, format(100 - shares.share, "percent"), look.halo ?? "transparent", look.lose ?? "transparent");
        add(y + 19, format(shares.share, "percent"), look.halo ?? "transparent", look.gain ?? "transparent");
      },
    },
    { over: true },
  );
  ownLayer(series, tags);
}
function simplify(points, tolerance) {
  if (points.length < 3) return points;
  const keep = new Uint8Array(points.length), stack = [[0, points.length - 1]];
  keep[0] = keep[points.length - 1] = 1;
  while (stack.length) {
    const [from, to] = /** @type {[number, number]} */ (stack.pop());
    let far = -1, most = tolerance;
    for (let i = from + 1; i < to; i++) {
      const off = toSegment(points[i].x, points[i].y, points[from], points[to]);
      if (off > most) [far, most] = [i, off];
    }
    if (far >= 0) {
      keep[far] = 1;
      stack.push([from, far], [far, to]);
    }
  }
  return points.filter((_, i) => keep[i]);
}
const gesturing = () => Boolean(placing?.held || moving || boxing || lensAt?.touch);
/** @param {PointerEvent} event */
const otherPointer = (event) => gesturing() && event.pointerId !== gesturePointer;
/** @param {PointerEvent} event @param {string} pick */
function pressAt(event, pick) {
  const { clientX, clientY } = event;
  press(event, { end: (release) => release && !otherPointer(release) && !fingerCross.dismissing && Math.hypot(release.clientX - clientX, release.clientY - clientY) < 4 && chooseDrawings(pick ? [pick] : []) });
}
const drawingMenu = (/** @type {{ x: number, y: number }} */ point) => ((ids) => menuOf($("plot-keys"), drawingRows(ids), point, "", { named: ids.length > 1 ? plural(ids.length, "drawing") : drawingNames().get(ids[0]) ?? "Drawing" }))([...chosen]);
/** @param {string[]} ids @returns {Action[]} */
function drawingRows(ids, /** @type {HTMLElement} */ anchor = $("plot-keys")) {
  const several = ids.length > 1, one = several ? null : drawingById(ids[0]);
  return [
    ...(one?.kind === "text" ? [{ key: "text", icon: ICONS.pencil, label: "Edit the text", run: () => ((now) => now && startTyping(now, anchor))(drawingById(one.id)) }] : []),
    ...(one && LINES.has(one.kind) ? [{ key: "label", icon: ICONS.label, label: one.label ? "Edit the label" : "Add a label", run: () => ((now) => now && startLabel(now, now.at, anchor))(drawingById(one.id)) }] : []),
    ...flipsOf(ids).map(([across, way, icon]) => ({ key: `flip-${across ? "x" : "y"}`, icon: ICONS[icon], label: `Flip ${way}`, verb: "Flip", run: () => flipDrawings(ids, across) })),
    { key: "duplicate", icon: ICONS.duplicate, label: several ? `Duplicate ${ids.length} drawings` : "Duplicate", kbd: "⌘D", verb: "Duplicate", run: () => duplicateDrawings(ids) },
    // What it was opened from gone with them, a ⋯'s: the focus on the plot, as the keys place it.
    { key: "remove", icon: ICONS.trash, label: several ? `Remove ${ids.length} drawings` : "Remove", kbd: "⌫", verb: "Remove", danger: true, run: () => (keepFocus(null, () => $("plot-keys")), removeDrawings(ids)) },
  ];
}
function pointsApart(/** @type {Drawing} */ drawing, apart = 3) {
  const series = drawn.get(drawing.layer), line = timeLine(), ends = series && line ? onScreen(drawing, series, line) : [];
  return ends.length > 1 && Math.hypot(ends[1].x - ends[0].x, ends[1].y - ends[0].y) >= apart;
}
function pathMoved() {
  renderBelt();
  repaintDrawings();
  syncUndo();
  $("chart-said").textContent = plural(/** @type {Drawing} */ (draft).points.length - 1, "point");
}

function finishPath() {
  if (!placingPath() || !draft) return false;
  const drawing = { ...draft, points: draft.points.slice(0, -1) }, again = toolKept;
  takeUp(again ? "path" : "", again, true);
  keepDrawing(drawing);
  if (drawingById(drawing.id)) $("chart-said").textContent = `Path of ${drawing.points.length} points`;
  return true;
}
function addDrawn(/** @type {Drawing} */ drawing) {
  addDrawing(drawing);
  if (!toolKept && drawingById(drawing.id) && edits.past.length) noteDrawn(/** @type {ChartStep} */ (edits.past.at(-1)).state, { kind: drawing.kind, picks });
}
function noteDrawn(/** @type {string} */ state, /** @type {{ kind: DrawingKind, picks: number }} */ drawn) {
  drawnSteps.set(state, drawn);
  for (const key of drawnSteps.keys()) if (drawnSteps.size > 200) drawnSteps.delete(key);
}
// By the step itself: its tool, and how many times you had picked one then (`picks`: a tool put down after a drawing, or taken up or down by an undo, isn't a pick). `undoTook`: the tool an undo took up again, none once one's picked otherwise.
const drawnSteps = /** @type {Map<string, { kind: DrawingKind, picks: number }>} */ (new Map());
// By its body, every one selected with it follows; by one of its points, it alone (a line by an end, a box by a corner, the opposite held; a stroke, line or path by a corner or side of its box, scaled from the opposite).
function moveTo(event) {
  const at = plotPoint(event.clientX, event.clientY, moving.rect), { originals, target, handle } = moving, line = timeLine();
  if (!moving.moved && Math.hypot(at.x - moving.start.x, at.y - moving.start.y) < 3) return;
  moving.moved = true;
  hold(moving.cursor);
  if (!line) return;
  if (moving.slide) {
    const original = /** @type {Drawing} */ (originals.get(target)), series = drawn.get(original.layer), size = sketches.get(series)?.size;
    if (series && size) reshaped.set(target, { ...original, at: shareAlong(original, onScreen(original, series, line), size, at.x, at.y) });
    return repaintDrawings();
  }
  if (handle < 0) {
    // One shift for all of them (whole bars, if any keeps to bars): they keep their places to each other.
    const free = [...originals.values()].every((each) => each.kind === "brush" || each.kind === "text"), dx = free ? at.x - moving.start.x : Math.round((at.x - moving.start.x) / line.step) * line.step;
    for (const [id, original] of originals) {
      const series = drawn.get(original.layer);
      if (series) reshaped.set(id, { ...original, points: movedBy(original, series, line, dx, at.y - moving.start.y) });
    }
    return repaintDrawings();
  }
  const original = /** @type {Drawing} */ (originals.get(target)), host = layerById(original.layer), series = drawn.get(original.layer);
  if (!host || !series) return;
  const point = () => pointFrom(at, host, { pull: original.kind !== "ruler" && !event.altKey });
  let points = original.points;
  if (THROUGH.has(original.kind) && handle >= original.points.length) points = scaled(original, series, line, handle - original.points.length, at);
  else if (THROUGH.has(original.kind)) points = original.points.map((each, i) => (i === handle ? point() ?? each : each));
  else if (BOXES.has(original.kind) && handle >= 4) {
    const [a, b] = onScreen(original, series, line), across = handle % 2 === 1, taken = point() ?? original.points[0];
    const i = across ? Number((handle === 5) !== (a.x >= b.x)) : Number((handle === 4) !== (a.y <= b.y));
    points = original.points.map((each, k) => (k === i ? (across ? [taken[0], each[1]] : [each[0], taken[1]]) : each));
  } else if (BOXES.has(original.kind)) {
    const [[t0, v0], [t1, v1]] = original.points, held = /** @type {[number, number]} */ ([[t0, v0], [t1, v0], [t1, v1], [t0, v1]][(handle + 2) % 4]);
    points = [held, point() ?? original.points[1]];
  } else points = scaled(original, series, line, handle, at);
  reshaped.set(target, { ...original, points });
  repaintDrawings();
}
function scaled(/** @type {Drawing} */ drawing, series, line, /** @type {number} */ corner, at) {
  const points = onScreen(drawing, series, line), box = boundsOf(drawing, points);
  if (corner >= 4) at = { x: corner === 5 ? at.x : corner === 7 ? at.x : box.right, y: corner === 4 || corner === 6 ? at.y : box.bottom };
  const west = corner === 0 || corner === 3 || corner === 7, north = corner < 2 || corner === 4;
  const held = { x: west ? box.right : box.left, y: north ? box.bottom : box.top }, from = { x: west ? box.left : box.right, y: north ? box.top : box.bottom };
  // A corner can be taken past the held one; a flat stroke (no height or no width) keeps it.
  const sx = from.x === held.x ? 1 : (at.x - held.x) / (from.x - held.x), sy = from.y === held.y ? 1 : (at.y - held.y) / (from.y - held.y);
  return points.flatMap((point) => {
    if (!finite(point)) return [];
    const value = seriesValue(series, held.y + (point.y - held.y) * sy);
    return value === null || !Number.isFinite(value) ? [] : [pointOf(timeOfBar((held.x + (point.x - held.x) * sx - line.at) / line.step), value)];
  });
}
/** @param {Drawing} drawing @returns {[number, number][]} */
function movedBy(drawing, series, line, dx, dy) {
  const bars = drawing.kind === "brush" || drawing.kind === "text" ? dx / line.step : Math.round(dx / line.step);
  return drawing.points.map(([time, value]) => {
    const y = dy ? seriesY(series, value) : null, to = y === null ? null : seriesValue(series, y + dy);
    return pointOf(timeOfBar(barOf(time) + bars), to !== null && Number.isFinite(to) ? to : value);
  });
}
function duplicateDrawings(/** @type {string[]} */ ids) {
  const line = timeLine(), copies = /** @type {Drawing[]} */ (ids.map(drawingById).filter(Boolean)).flatMap((drawing) => {
    const series = drawn.get(drawing.layer);
    return series && line ? [{ ...structuredClone(drawing), id: uid(), points: movedBy(drawing, series, line, Math.max(1, Math.round(12 / line.step)) * line.step, 12) }] : [];
  });
  addDrawings(copies);
}
// Across, by their places on the timeline; up and down, as they show on their pane (a log scale's too), each pane's apart. A line across keeps its value flipped left for right, a line down its date top for bottom.
function flipDrawings(/** @type {string[]} */ ids, /** @type {boolean} */ across) {
  const drawings = /** @type {Drawing[]} */ (ids.map(drawingById).filter(Boolean)), flipped = new Map();
  const moves = (/** @type {Drawing} */ drawing) => drawing.kind !== (across ? "hline" : "vline");
  if (across) {
    const bars = drawings.filter(moves).flatMap((drawing) => drawing.points.map(([time]) => barOf(time))), sum = Math.min(...bars) + Math.max(...bars);
    for (const drawing of drawings) if (moves(drawing)) flipped.set(drawing.id, drawing.points.map(([time, value]) => pointOf(timeOfBar(sum - barOf(time)), value)));
  } else {
    const paneOf = (/** @type {Drawing} */ drawing) => layerById(drawing.layer)?.pane;
    for (const pane of new Set(drawings.filter(moves).map(paneOf))) {
      const own = drawings.filter((drawing) => moves(drawing) && paneOf(drawing) === pane), ys = own.flatMap((drawing) => drawing.points.map(([, value]) => seriesY(drawn.get(drawing.layer), value) ?? NaN)).filter(Number.isFinite);
      if (!ys.length) continue;
      const sum = Math.min(...ys) + Math.max(...ys);
      for (const drawing of own) {
        const series = drawn.get(drawing.layer);
        if (series) flipped.set(drawing.id, drawing.points.map(([time, value]) => {
          const y = seriesY(series, value), to = y === null ? null : seriesValue(series, sum - y);
          return pointOf(time, to !== null && Number.isFinite(to) ? to : value);
        }));
      }
    }
  }
  if (flipped.size) commit((doc) => doc.drawings?.forEach((item) => flipped.has(item.id) && (item.points = flipped.get(item.id))));
}
const flips = (/** @type {Drawing[]} */ drawings, /** @type {boolean} */ across) => drawings.length > 1 ? drawings.some((drawing) => drawing.kind !== (across ? "hline" : "vline")) : ["trend", "path", "brush"].includes(drawings[0]?.kind ?? "");
const FLIPS = /** @type {const} */ ([[true, "horizontally", "flipX", "Left for right"], [false, "vertically", "flipY", "Top for bottom"]]);
const flipsOf = (/** @type {string[]} */ ids) => FLIPS.filter(([across]) => flips(/** @type {Drawing[]} */ (ids.map(drawingById).filter(Boolean)), across));
function selectInView(/** @type {string} */ id) {
  const drawing = drawingById(id), range = shownSpan();
  if (!drawing) return;
  chooseDrawings([id]);
  // ⌘K gives the focus back as it closes: taken from there.
  const keys = () => $("plot-keys").focus({ preventScroll: true });
  if (!TOUCH) isOpen($("palette")) ? onClosed($("palette"), keys, true) : keys();
  if (!range || drawing.kind === "hline") return;
  const bars = drawing.points.map(([time]) => barOf(time)), first = Math.min(...bars), last = Math.max(...bars);
  if (last >= range.from && first <= range.to) return;
  const half = Math.max((range.to - range.from) / 2, (last - first) / 2 + 2), middle = (first + last) / 2;
  glide({ from: middle - half, to: middle + half });
}
function nudgeDrawings(/** @type {string} */ key, far = false, end = false, repeat = false) {
  const line = timeLine(), ids = [...chosen];
  if (!line || !ids.length) return;
  const times = far ? 10 : 1, across = key === "ArrowLeft" ? -1 : key === "ArrowRight" ? 1 : 0, down = key === "ArrowDown" ? 1 : key === "ArrowUp" ? -1 : 0;
  const dx = across * Math.max(1, Math.round(1 / line.step)) * times * line.step, dy = down * times;
  if (end && !ids.some((id) => POINTED.has(drawingById(id)?.kind ?? ""))) return void (!repeat && notice("Only lines through points and boxes can be resized. The arrow keys move this one"));
  commit((doc) => doc.drawings?.forEach((item) => {
    const series = drawn.get(item.layer);
    if (!ids.includes(item.id) || !series || (end && !POINTED.has(item.kind))) return;
    item.points = end ? [...item.points.slice(0, -1), movedBy({ ...item, points: item.points.slice(-1) }, series, line, dx, dy)[0]] : movedBy(item, series, line, dx, dy);
  }), { coalesce: `${end ? "resize" : "nudge"}:${ids.join(",")}` });
}
/** @param {Drawing[]} drawings */
function addDrawings(drawings) {
  if (!drawings.length) return;
  if ((chart.drawings?.length ?? 0) + drawings.length > MAX_DRAWINGS) {
    repaintDrawings();
    return notice(`A chart holds up to ${MAX_DRAWINGS} drawings`);
  }
  commit((doc) => (doc.drawings ??= []).push(...drawings));
  if (!toolKept) chooseDrawings(drawings.map((drawing) => drawing.id));
}
const addDrawing = (/** @type {Drawing} */ drawing) => addDrawings([drawing]);
function removeDrawings(/** @type {string[]} */ ids) {
  const gone = /** @type {Drawing[]} */ (ids.map(drawingById).filter(Boolean));
  if (!gone.length) return;
  commit((doc) => (doc.drawings = doc.drawings?.filter((item) => !ids.includes(item.id))));
  undoToast(gone.length > 1 ? `Removed ${plural(gone.length, "drawing")}` : `Removed the ${TOOLS[gone[0].kind].noun}`);
}
const removeDrawing = (/** @type {string} */ id) => removeDrawings([id]);

function hold(/** @type {string} */ cursor) {
  document.body.toggleAttribute("data-held", Boolean(cursor));
  document.body.style.setProperty("--held", cursor || "auto");
}
function showBox() {
  const box = $("select-box"), plot = plotBox(), canvas = $("canvas").getBoundingClientRect();
  if (!boxing || !plot) return;
  const top = boxing.rect.top - canvas.top, { start, now } = boxing;
  Object.assign(box.style, { left: `${plot.left + Math.min(start.x, now.x)}px`, top: `${top + Math.min(start.y, now.y)}px`, width: `${Math.abs(now.x - start.x)}px`, height: `${Math.abs(now.y - start.y)}px` });
  box.hidden = false;
}
function boxTo(event) {
  const at = plotPoint(event.clientX, event.clientY, boxing.rect);
  boxing.now = { x: clamp(at.x, 0, engine.time.width), y: clamp(at.y, 0, boxing.rect.height) };
  showBox();
}
function crossesBox(a, b, box) {
  if (!finite(a) || !finite(b)) return false;
  let from = 0, to = 1;
  const dx = b.x - a.x, dy = b.y - a.y;
  for (const [p, q] of [[-dx, a.x - box.left], [dx, box.right - a.x], [-dy, a.y - box.top], [dy, box.bottom - a.y]]) {
    if (p === 0) {
      if (q < 0) return false;
    } else if (p < 0) from = Math.max(from, q / p);
    else to = Math.min(to, q / p);
  }
  return from <= to;
}
function inBox(/** @type {Drawing} */ drawing, points, box, size) {
  const [a, b] = points, overlaps = (other) => other.left <= box.right && other.right >= box.left && other.top <= box.bottom && other.bottom >= box.top;
  if (THROUGH.has(drawing.kind)) return lineThrough(drawing, points, size).some((point, i, through) => i > 0 && crossesBox(through[i - 1], point, box));
  if (drawing.kind === "hline") return a.y >= box.top && a.y <= box.bottom;
  if (drawing.kind === "vline") return a.x >= box.left && a.x <= box.right;
  if (drawing.kind === "ellipse" && finite(a) && finite(b) && a.x !== b.x && a.y !== b.y) {
    const cx = (a.x + b.x) / 2, cy = (a.y + b.y) / 2, x = (clamp(cx, box.left, box.right) - cx) / Math.abs(b.x - cx), y = (clamp(cy, box.top, box.bottom) - cy) / Math.abs(b.y - cy);
    return x * x + y * y <= 1;
  }
  if (BOXES.has(drawing.kind)) return finite(a) && finite(b) && overlaps({ left: Math.min(a.x, b.x), right: Math.max(a.x, b.x), top: Math.min(a.y, b.y), bottom: Math.max(a.y, b.y) });
  if (drawing.kind === "text") return finite(a) && overlaps(boundsOf(drawing, points));
  const shown = points.filter(finite);
  return shown.some((point, i) => crossesBox(point, shown[i + 1] ?? point, box));
}
function clearDrawings() {
  const n = chart.drawings?.length ?? 0;
  if (!n) return;
  commit((doc) => delete doc.drawings);
  undoToast(`Removed ${plural(n, "drawing")}`);
}

function dressField(/** @type {Drawing} */ drawing) {
  const field = $("draw-text"), size = drawing.size ?? TEXT_SIZE;
  field.style.setProperty("--c", cssColor(drawing.color));
  field.style.setProperty("--on", wordsColor(drawing));
  field.style.setProperty("--size", `${size}px`);
  field.style.setProperty("--line", `${Math.round(size * 1.33)}px`);
  field.dataset.back = drawing.back ?? "";
  field.hidden = false;
}
function placeField() {
  const field = /** @type {HTMLTextAreaElement} */ ($("draw-text")), drawing = writing?.drawing, layer = drawing && layerById(drawing.layer);
  const series = layer && drawn.get(layer.id), line = timeLine(), pane = layer && paneBox(chart.panes.findIndex((item) => item.id === layer.pane));
  if (!drawing || !series || !line || !pane) return;
  const canvasBox = $("canvas").getBoundingClientRect(), plotLeft = engine.element.getBoundingClientRect().left + engine.left;
  if (writing.label) {
    const size = sketches.get(series)?.size, place = size && labelPlace(drawing, onScreen(drawing, series, line), size, field.value || field.placeholder);
    if (!place) return;
    return void Object.assign(field.style, { left: `${plotLeft - canvasBox.left + place.x - place.width / 2 - 6}px`, top: `${pane.top - canvasBox.top + place.y - place.height / 2}px`, width: `${Math.ceil(place.width) + 12}px`, height: `${place.height}px` });
  }
  const place = textPlace(drawing, onScreen(drawing, series, line)[0], field.value || field.placeholder);
  Object.assign(field.style, { left: `${plotLeft - canvasBox.left + place.x - place.width / 2 - 6}px`, top: `${pane.top - canvasBox.top + place.y - place.height / 2}px`, width: `${Math.ceil(place.width) + 12}px`, height: `${place.height}px` });
}
export {
  addDrawn, amountsOf, boxTo, bubbleColumns, cellText, clearDrawings, closeCell, drawingMenu, drawingRows,
  drawnSteps, dressField, duplicateDrawings, finishPath, flipDrawings, flipsOf, fromClose, gesturing, gradientCell, gradientChannels, HEAT_ABOUT,
  HEAT_COLUMN, HEAT_FADE, HEAT_SHORT, heatCalendar, heatColor, heatCome, heatDayHeld, heatDayKey,
  heatDays, heatExtras, heatFresh, heatGrid, heatIn, heatLook, heatLooks, heatMost, heatPath,
  heatPump, heatReading, heatRows, heatSection, heatSeries, heatShown, heatWaiters, heatWants, hold, inBox, litCohort,
  mostIn, moveTo, noteDrawn, nudgeDrawings, otherPointer, pathMoved, placeField, placeSaid, 
  pointsApart, pressAt, priceSpan, PROFILE_ABOUT, PROFILE_SHORT, profileHit, profileShares, profilesUnder,
  readingLine, removeDrawing, removeDrawings, rowsOf, selectInView, setCohorts, showBox, shownHeats, simplify,
  toggleCohort, viewHeat, viewProfile,
};
