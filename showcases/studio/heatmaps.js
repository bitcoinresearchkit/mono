// @ts-check
// Heatmaps (the URPD, a day a column) and bubbles.
import { channels, columnWidth, Custom, placeIndex, press, Profile } from "/modules/bitview-charts/index.js";
import { capitalized, inSentence } from "/studio/words.js";
import {
  boxing, capturing, chart, chooseDrawings, chosen, client, cohortColor, cohortName, cohortShort, cohortWho, commit,
  css, draft, edits, engine, exporting, gesturePointer, HEAT_AGES, HEAT_COHORTS, HEAT_HOLDERS, heatAt, heatCohort,
  heatKnown, keepDrawing, keyTime, lensAt, light, moving, notice, picks, placing, profileColor, profileLit, renderBelt,
  reshape, reshaped, resolved, selected, startLabel, startTyping, syncUndo, takeUp, tool, toolKept, undoToast, writing,
} from "/studio/app.js";
import { $, escapeHtml, TOUCH } from "/studio/dom.js";
import { clamp, lowerBound, uid } from "/studio/math.js";
import { format, plural } from "/studio/format.js";
import { CHEVRON, ICONS } from "/studio/icons.js";
import {
  belowOf, DEFAULT_MEASURE, DEFAULT_ROWS, HEAT_MEASURES, HEAT_ROWS, HEAT_WEIGHTS, heatLike, measureOf, measureUnit,
  PROFILE_VIEWS,
} from "/studio/vocabulary.js";
import { BOXES, LINES, MAX_DRAWINGS, POINTED, pointOf, TEXT_SIZE, THROUGH, TOOLS } from "/studio/tools.js";
import {
  alpha, crossSet, cssColor, drawn, fingerCross, glide, Heat, ink, lastShown, paneBox, PROFILE_DIM, seriesPane,
  seriesValue, seriesY, shownSpan, valuesSoon,
} from "/studio/chart.js";
import { bothOf, choicePage } from "/studio/command.js";
import {
  barOf, boundsOf, drawingById, drawingNames, finite, labelPlace, lineThrough, onScreen, ownLayer, plotBox, plotPoint,
  pointFrom, repaintDrawings, shareAlong, sketches, textPlace, timeLine, timeOfBar, toSegment, wordsColor,
} from "/studio/drawing.js";
import { isOpen, onClosed } from "/studio/grid.js";
import { editById, gradientTable, keepFocus, layerById, modeOf, showScale } from "/studio/inspector.js";
import { menuOf } from "/studio/menus.js";
import { refocus } from "/studio/presets-edit.js";
/** @import { ChartStep } from "/studio/app.js" */
/** @import { Chart, Drawing, DrawingKind, Layer } from "/studio/vocabulary.js" */
/** @import { ChartPoint, ReadPoint, Resolved } from "/studio/chart.js" */
/** @import { Action, Page } from "/studio/command.js" */
/** @import { Vector } from "/studio/formulas.js" */


/**
 * Bubbles' sizes, each date's own value, and places, the value there of the layer they're on as drawn (a stack's,
 * its total; on a log scale, none at 0 or under, as it draws none); on none, its pane's middle. On the timeline.
 * @param {Chart} doc @param {Resolved} data @param {Layer} layer @param {Map<string, { values: Vector }>} stacked
 * @returns {[Float64Array, Float64Array]}
 */
function bubbleColumns(doc, data, layer, stacked) {
  const own = /** @type {Vector} */ (data.results.get(layer.id)?.values), on = doc.layers.find((other) => other.id === layer.ref);
  const at = on && (stacked.get(on.id)?.values ?? data.results.get(on.id)?.values), log = Boolean(on) && modeOf(doc, /** @type {Layer} */ (on)) === 1;
  const n = data.times.length, v = new Float64Array(n), y = new Float64Array(n);
  for (let i = 0; i < n; i++) (v[i] = own?.[i] ?? NaN), (y[i] = !on ? 0.5 : ((value) => (Number.isFinite(value) && (!log || value > 0) ? value : NaN))(at?.[i] ?? NaN));
  return [v, y];
}
/** Values' largest size (in all of them, without a range), 0 for none. @param {Vector} values @param {{ from: number, to: number } | null} [range] */
const mostIn = (values, range = { from: 0, to: values.length - 1 }) => {
  let most = 0;
  // (An infinite value none, as NaN is: else every other one as none beside it.)
  if (range) for (let i = Math.max(0, Math.ceil(range.from)); i <= Math.min(values.length - 1, Math.floor(range.to)); i++) if (Math.abs(values[i]) > most && Math.abs(values[i]) < Infinity) most = Math.abs(values[i]);
  return most;
};

/** A heatmap's or a profile's measure by its name ("Supply"), its weight by its ("Every coin", "Awake"). @param {Layer} layer */
const measureName = (layer) => HEAT_MEASURES.find(([id]) => id === measureOf(layer))?.[1];
const weightName = (/** @type {Layer} */ layer) => HEAT_WEIGHTS.find(([id]) => id === (layer.weight ?? ""))?.[1];
/** Its weight as its name ends with it (", awake"; every coin: nothing). @param {Layer} layer */
const weightSaid = (layer) => (layer.weight ? `, ${/** @type {string} */ (weightName(layer)).toLowerCase()}` : "");
/** A heatmap's name: its measure by cost basis, its weight's, then whose ("Supply by cost basis, awake: Short-term holders"). @param {Layer} layer */
const heatName = (layer) => layer.type === "profile" ? profileName(layer) : `${measureName(layer)} by cost basis${weightSaid(layer)}: ${cohortWho(layer.heatmap ?? "all")}`;
/**
 * A profile's name, as a heatmap's (see heatName): its measure's, its weight's, then whose, its holder groups (two
 * named, more counted: "Supply profile, awake: Long-term holders and short-term holders"). @param {Layer} layer
 */
const profileName = (layer, grouped = true) => ((groups) => `${measureName(layer)} profile${weightSaid(layer)}${!grouped || !groups.length ? "" : `: ${groups.length > 2 ? capitalized(groups.every((cohort) => HEAT_AGES.includes(cohort)) ? (groups.length === HEAT_AGES.length ? "every age band" : `${groups.length} age bands`) : `${groups.length} holder groups`) : groups.map((cohort, k) => (k ? inSentence(cohortWho(cohort)) : cohortWho(cohort))).join(" and ")}`}`)(layer.profile ?? []);
/** What a profile is, said where one's added, and in a row's few words. */
const PROFILE_ABOUT = "Holder groups' supply, capital or unrealized P&L by the price it last moved at, on the view's last day, as bars at the chart's right edge: each group its color, under the price in profit.";
const PROFILE_SHORT = "Holder groups' cost basis, at the right edge";
/** What a heatmap is, said where one's added. */
const HEAT_ABOUT = "A holder group's supply, capital or unrealized P&L by the price it last moved at, day by day, behind the price (the URPD).";
/** The same, in a row's few words. */
const HEAT_SHORT = "Holder groups' cost basis, day by day";
/** A heatmap's code, where a series shows its name: its endpoint's, `urpd/` and its holders. @param {Layer} layer */
const heatCode = (layer) => ["urpd", layer.type === "profile" ? (layer.profile ?? []).join("+") : layer.heatmap ?? "all"].filter(Boolean).join("/");
/** A heatmap's latest day, as the API's path (its weight and density as it asks for them). @param {Chart} doc @param {Layer} layer */
const heatPath = (doc, layer) => `/api/urpd/${encodeURIComponent(layer.heatmap ?? "all")}?agg=${heatGrid(doc, layer)?.agg ?? `log${rowsOf(layer)}`}${layer.weight ? `&weight=${layer.weight}` : ""}`;
/** What each measure and weight is, said beside it. */
const HEAT_SAYS = /** @type {Record<string, string>} */ ({ supply: "Bitcoin held, by the price it last moved at", capital: "What it cost, by that price", pnl: "Its gain or loss now, by that price", "": "Each coin counted as it is", wakefulness: "Each age band by how much of its coin days are spent (Cointime)", mobility: "Each age band by how likely its coins are to move (Coinflow)", bars: "Each price's amount", cumulative: "Summed from the close outward: at each price, all that last moved between it and the close" });
/**
 * A profile's holder group in or out (in, after the others: further from the edge; out, its last too, none drawn till
 * one's picked). Its chip, or its row in ⌘K, keeps the focus. @param {string} id @param {string} cohort
 */
function toggleCohort(id, cohort) {
  if (layerById(id)?.type !== "profile") return;
  editById(id, (item) => (item.profile = item.profile?.includes(cohort) ? item.profile.filter((other) => other !== cohort) : [...(item.profile ?? []), cohort]));
  if (!isOpen($("palette"))) refocus(`#inspector [data-cohort-of="${CSS.escape(cohort)}"]`);
}
/**
 * A profile's settings' two lists: its holder groups (all coins, the terms, the age cutoffs; then its groups Bitview
 * doesn't have, to take out) and the age bands, the youngest first. @param {Layer} layer
 */
const profileLists = (layer) => ({ holders: [...HEAT_HOLDERS, ...(layer.profile ?? []).filter((id) => !heatCohort(id))], ages: HEAT_AGES });
/**
 * A profile's list (see profileLists) all in (those not in yet, after the others, in its order) or all out. The button
 * pressed keeps the focus. @param {string} id @param {"holders" | "ages"} list @param {boolean} on
 */
function setCohorts(id, list, on) {
  const layer = layerById(id);
  if (layer?.type !== "profile") return;
  const ids = profileLists(layer)[list].filter((cohort) => !on || heatCohort(cohort));
  editById(id, (item) => (item.profile = on ? [...(item.profile ?? []), ...ids.filter((cohort) => !item.profile?.includes(cohort))] : (item.profile ?? []).filter((cohort) => !ids.includes(cohort))));
  refocus(`#inspector [data-cohorts-${on ? "in" : "out"}="${list}"]`);
}
/** ⌘K's page of a profile's holder groups: each pressed in or out, as a trend's layers are (see fromPage). @param {string} id @returns {Page} */
const cohortsPage = (id) => ({ title: "Holders", placeholder: "Search the holder groups", rows: () => {
  const layer = layerById(id), groups = layer?.profile ?? [];
  // (Its groups Bitview doesn't have (a link's typo) last, to take out.)
  const rows = [...HEAT_COHORTS.map((c) => [c.id, c.shown, c.age ? "Age bands" : "Holder groups"]), ...groups.filter((cohort) => !heatCohort(cohort)).map((cohort) => [cohort, `${cohort}: not on Bitview`, "Not on Bitview"])];
  return rows.map(([cohort, name, group]) => {
    const on = groups.includes(cohort);
    return { kind: "row", key: cohort, icon: `<i class="swatch" style="--c: ${escapeHtml(cssColor(layer ? profileColor(layer, cohort) : cohortColor(cohort)))}"></i>`, label: name, aside: on ? "In it" : "", current: on, verb: on ? "Deselect" : "Select", stay: true, group, run: () => toggleCohort(id, cohort) };
  });
} });
/**
 * A heatmap's rows in its page (see layerPage) and its inspector's menus: its holders, measure, weight and density,
 * each a choice shown on the chart while highlighted. @param {string} id
 */
function heatRows(id) {
  const both = bothOf(id), layer = /** @type {Layer} */ (layerById(id)), kind = layer.type === "profile" ? "Profile" : "Heatmap", now = () => layerById(id) ?? layer, grid = heatGrid(chart, layer);
  const choices = (/** @type {string} */ title, /** @type {[any, string, ...any[]][]} */ list, /** @type {(item: Layer) => any} */ value, /** @type {(item: Layer, choice: any) => void} */ apply, /** @type {(choice: any) => string | undefined} */ says = () => undefined) => () => choicePage(title, () => list.map(([choice, text]) => ({ key: String(choice), label: text, aside: says(choice), current: value(now()) === choice, ...both((item) => apply(item, choice)) })));
  return [
    // (A profile's, its groups pressed in or out; a heatmap's, its one.)
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
/**
 * A heatmap's section in the inspector: its holders and its density, each a menu of its page's choices (see
 * heatRows); its measure and its weight, each a row of three. A profile's: its holder groups as chips. @param {Layer} layer
 */
function heatSection(layer) {
  const rows = heatRows(layer.id);
  const menu = (/** @type {string} */ key) => ((row) => `<div class="field"><span>${escapeHtml(row.label)}</span><button type="button" class="preset" data-action="heat-menu" data-heat="${key}" aria-haspopup="menu" aria-label="${escapeHtml(`${row.label}: ${row.aside ?? ""}`)}"${row.disabled ? ` aria-disabled="true" data-tip="${escapeHtml(row.disabled)}"` : ""}><span class="name">${escapeHtml(row.aside ?? "")}</span>${CHEVRON}</button></div>`)(/** @type {any} */ (rows.find((row) => row.key === key)));
  const tabs = (/** @type {string} */ key, /** @type {string} */ label, /** @type {string[][]} */ list, /** @type {string} */ now) => `<div class="field"><span>${label}</span><div class="tabs" role="group" aria-label="${label}">${list.map(([value, text]) => `<button type="button" data-heat-set="${key}" data-value="${value}" aria-pressed="${now === value}" data-tip="${escapeHtml(HEAT_SAYS[value])}">${escapeHtml(text)}</button>`).join("")}</div></div>`;
  // (Of a holder group Bitview doesn't have: said under it, another one to pick there.)
  const gone = !heatKnown(layer) ? '<div class="error-box gone"><p>Bitview has no holder group by this name: renamed, removed, or mistyped. Pick another above.</p></div>' : "";
  // (A profile's: its holder groups and age bands, each pressed in or out, as a trend's layers are (see
  // betweenField), each in its color; the first pressed at the edge.)
  // (Each list in a grid, each by its short name (its whole one its tip), all of it in or out at once: All, None.)
  const groups = layer.profile ?? [], lists = profileLists(layer);
  const chips = (/** @type {"holders" | "ages"} */ list, /** @type {string} */ label) => `<div class="from-of ${list}" role="group" aria-label="${label}">${lists[list].map((id) => ((known) => `<button type="button" data-cohort-of="${escapeHtml(id)}" aria-pressed="${groups.includes(id)}"${known ? ` aria-label="${escapeHtml(cohortName(id))}" data-tip="${escapeHtml(cohortName(id))}"` : ""}><i class="swatch" style="--c: ${escapeHtml(cssColor(profileColor(layer, id)))}"></i><span class="name">${escapeHtml(known ? cohortShort(id) : `${id}: not on Bitview`)}</span></button>`)(Boolean(heatCohort(id)))).join("")}</div>`;
  const both = (/** @type {"holders" | "ages"} */ list, /** @type {string} */ name) => ((ids) => `<span class="list-actions"><button type="button" class="cap small words" data-cohorts-in="${list}" aria-label="All ${name}"${ids.every((id) => groups.includes(id) || !heatCohort(id)) ? ' aria-disabled="true"' : ""}>All</button><button type="button" class="cap small words" data-cohorts-out="${list}" aria-label="No ${name}"${ids.some((id) => groups.includes(id)) ? "" : ' aria-disabled="true"'}>None</button></span>`)(lists[list]);
  // (A profile's holder groups and age bands, a heatmap's holder group; then their measure, weight and density alike.)
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
/** A column at least this wide (CSS pixels): zoomed out, a day in so many drawn, the days between never asked for. */
const HEAT_COLUMN = 2;
/** A heatmap's or a profile's rows a ×10 in price, as it's drawn (see heatGrid). @param {Layer} layer */
const rowsOf = (layer) => layer.rows ?? DEFAULT_ROWS;
/**
 * Rows `n` a ×10 in price, each the same share of its price: where a price falls among them (`at`, the row it's in
 * and how far into it), the row it's in (`cell`), a row's floor price (`price`), and the row a floor price is
 * (`floor`: as the URPD writes it, rounded). @param {number} n
 */
const logRows = (n) => ({ at: (/** @type {number} */ price) => n * Math.log10(price * 100), cell: (/** @type {number} */ price) => Math.floor(n * Math.log10(price * 100)), price: (/** @type {number} */ row) => 10 ** (row / n) / 100, floor: (/** @type {number} */ price) => Math.round(n * Math.log10(price * 100)) });
/**
 * How a heatmap's rows are cut, as the URPD's `agg` says it: `rows` a ×10 in price (each the same share of its
 * price, as a cost basis is read), on a log scale or a linear one (`even`: on a log one, rows even on the scale;
 * on a linear one, binned by dollars, see the library's Grid: a day drawn as one strip, stretched, either way); none on a scale they can't be
 * (percent change, indexed). The row any price is in (`cell`) and a row's floor price (dollars).
 * @param {Chart} doc @param {Layer} layer
 */
function heatGrid(doc, layer) {
  const mode = modeOf(doc, layer), n = rowsOf(layer);
  if (mode > 1) return null;
  return { agg: `log${n}`, even: mode === 1, ...logRows(n) };
}
/**
 * A heatmap day's cell in a row (see logRows): its row and its amount, by a measure; nothing there, zero.
 * @param {HeatDay & { rows: Int32Array }} day @param {number} row @param {string} measure
 */
function dayCell(day, row, measure) {
  const k = lowerBound(day.rows, row);
  return { row, amount: day.rows[k] === row ? /** @type {Float32Array} */ (amountsOf(day, measure))[k] : 0 };
}
/** A heatmap day at rest: its close's cell (see dayCell); no close, none. @param {HeatDay & { rows: Int32Array }} day @param {ReturnType<typeof logRows>} grid @param {string} measure */
function closeCell(day, grid, measure) {
  const close = Number(day.close);
  return close > 0 ? dayCell(day, grid.cell(close), measure) : null;
}
/** How far a price is from the close, signed either way ("+5%", "−12%"): a cumulative profile's reading. @param {number} change a percent */
const fromClose = (change) => ((text) => (text.startsWith("−") ? text : `+${text}`))(format(change, "percent"));
/** A heatmap's cell as it's read: its amount ("₿412K") and the price it's at, its floor ("$62,034"). @param {{ row: number, amount: number }} cell @param {ReturnType<typeof logRows>} grid @param {string} measure */
const cellText = (cell, grid, measure) => ({ amount: format(cell.amount, measureUnit(measure)), at: format(grid.price(cell.row), "usd") });
/** A profile bin's prices, from its floor to the next one's ("$84,120–$85,130"). @param {number} from @param {number} to */
const priceSpan = (from, to) => `${format(from, "usd")}–${format(to, "usd")}`;
/** Where a binned layer's amount is, said ("at $62,034"). @param {string} at */
const placeSaid = (at) => `at ${at}`;
/** A binned layer's reading on one line: its amount, where it's at after it ("₿412K at $62,034"). @param {{ amount: string, at: string }} reading */
const readingLine = (reading) => (reading.at ? `${reading.amount} ${placeSaid(reading.at)}` : reading.amount);
/**
 * Each URPD day asked for, by cohort, weight, `agg` and day ("latest", the open one): its rows (each bucket's, low to
 * high) and their supply, capital and unrealized P&L, once come; `pending` while asked; `failed` (missing, for
 * good; else asked again from `retryAt`). The most recently drawn kept, up to so many cells (but never those in
 * view); its close (dollars).
 * @typedef {{ pending?: true, failed?: true, retryAt?: number, rows?: Int32Array, supply?: Float32Array, capital?: Float32Array, pnl?: Float32Array, close?: number, used: number }} HeatDay
 * @type {Map<string, HeatDay>}
 */
const heatDays = new Map();
let heatCells = 0, heatUse = 0;
const HEAT_MOST_CELLS = 3_000_000, HEAT_AT_ONCE = 6, HEAT_RETRY = 30_000;
/**
 * What each heatmap drawn wants and doesn't have, by its layer's id, as last drawn (see viewHeat): asked for the
 * coarsest first, the newest first among them, whichever heatmap's.
 * @type {Map<string, { key: string, cohort: string, weight: string, agg: string, day: string, rank: number }[]>}
 */
const heatWants = new Map();
let heatAsking = 0, heatPumping = false;
/** The heatmaps and profiles drawn, to draw again as days come (see heatSoon). @type {Set<Heat>} */
const heatSeries = new Set();
let heatFrame = 0;
/** Its series drawn again, once a frame, as days come. */
const heatSoon = () => void (heatFrame ||= requestAnimationFrame(() => {
  heatFrame = 0;
  for (const series of heatSeries) series.redraw();
  // (What it reads, the day come.)
  valuesSoon();
}));
/**
 * What waits for a heatmap's days (settle): woken as one's drawn, and as the tab's hidden (see heatIn).
 * @type {Set<() => void>}
 */
const heatWaiters = new Set();
/**
 * Whether the heatmaps shown have every day they show: each drawn since it was built, none wanted, none on its way
 * (one that failed, for good or for now, as it is). What a picture waits for (see settle); a tab drawing no frames
 * draws none, so asks for none: as it is; one with no price to draw on (its closes none), or `undrawable` (the
 * library had nothing on its scale to draw it on), as it is.
 */
const heatIn = () => document.hidden || shownHeats().every((look) => (look.undrawable && !look.drawn) || (look.drawn && !heatWants.has(look.id) && ![...(look.keep ?? [])].some((key) => heatDays.get(key)?.pending)));
/**
 * Each picture at a size of its own being made, its days (see chartImage): those it wants, asked for as the
 * chart's own are; all it shows (`keys`), kept as the chart's are, till it's taken.
 * @type {Set<{ wants: NonNullable<ReturnType<typeof heatWants.get>>, keys: Set<string> }>}
 */
const heatExtras = new Set();
/**
 * The days `keys` come (or failed), within `ms`, while `still` holds (what they're for): looked at every tenth of a
 * second (a tab out of sight too).
 */
const heatCome = (/** @type {string[]} */ keys, /** @type {number} */ ms, /** @type {() => boolean} */ still) => new Promise((done) => {
  const end = performance.now() + ms, look = () => (keys.every((key) => heatDays.get(key) && !heatDays.get(key)?.pending) || !still() || performance.now() > end ? done(undefined) : setTimeout(look, 100));
  look();
});
/** The looks of the heatmaps the chart shows (with a price to draw on). */
const shownHeats = () => chart.layers.flatMap((layer) => ((look) => (heatLike(layer) && !layer.hidden && look && resolved?.results.get(layer.id)?.closes?.some(Number.isFinite) ? [look] : []))(heatLooks.get(layer.id)));
/** The days wanted asked for, a few at once, the first wanted first. */
function heatPump() {
  heatPumping = false;
  const wanted = [...heatWants.values(), ...[...heatExtras].map((extra) => extra.wants)].flat().filter((want) => !heatDays.has(want.key)).sort((a, b) => b.rank - a.rank || b.day.localeCompare(a.day));
  for (const want of wanted) {
    if (heatAsking >= HEAT_AT_ONCE) break;
    if (heatDays.has(want.key)) continue;
    // (Its answer kept while it's still the one asked for: a block come meanwhile, the open day's is another.)
    const asked = { pending: /** @type {const} */ (true), used: ++heatUse };
    heatDays.set(want.key, asked);
    heatAsking++;
    const weight = /** @type {any} */ (want.weight || undefined), agg = /** @type {any} */ (want.agg);
    (want.day === "latest" ? client.getUrpdLatest(/** @type {any} */ (want.cohort), { agg, weight }) : client.getUrpd(/** @type {any} */ (want.cohort), want.day, { agg, weight }))
      .then((urpd) => {
        if (heatDays.get(want.key) !== asked) return;
        const buckets = /** @type {any} */ (urpd).buckets, n = buckets.price.length;
        // (Rows by the grid's own count: a bucket's floor price to its row, see heatGrid.)
        const rows = Int32Array.from(buckets.price, logRows(Number(want.agg.slice(3))).floor);
        heatDays.set(want.key, { rows, supply: Float32Array.from(buckets.supply), capital: Float32Array.from(buckets.realizedCap), pnl: Float32Array.from(buckets.unrealizedPnl), close: /** @type {any} */ (urpd).close, used: ++heatUse });
        heatCells += n;
        heatTrim();
      }, (/** @type {any} */ error) => {
        if (heatDays.get(want.key) !== asked) return;
        heatDays.set(want.key, { failed: true, ...(error?.status === 404 ? {} : { retryAt: Date.now() + HEAT_RETRY }), used: ++heatUse });
        // (Drawn again then, to ask again if it's still in view.)
        if (error?.status !== 404) setTimeout(heatSoon, HEAT_RETRY);
      })
      .finally(() => {
        heatAsking--;
        heatSoon();
        heatPump();
      });
  }
}
/** A block come: the open day's URPD another (and today's, a fine interval's), let go of: asked for again as it's drawn. */
function heatFresh() {
  const today = new Date().toISOString().slice(0, 10);
  for (const [key, day] of heatDays) {
    const date = key.slice(key.lastIndexOf("|") + 1);
    // (One still being asked for too: its answer, the old block's, isn't kept, see heatPump.)
    if (date !== "latest" && date < today) continue;
    heatDays.delete(key);
    dropStrips(key);
    heatCells -= day.rows?.length ?? 0;
  }
  heatSoon();
}
/** A day's strips let go of (its cells anew under the same key, or gone), by every heatmap's grid (drawn now or not). @param {string} key */
function dropStrips(key) {
  for (const series of drawn.values()) if (series instanceof Heat) series.grid?.forget(key);
}
/** The days drawn least recently let go of, past the most cells kept (those in view kept, however many). */
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
/** A heatmap's cells, by how many orders of magnitude under its reference they're seen (see heatColor). */
const HEAT_DECADES = 4;
/**
 * How strong a cell reads, its strength to this power: most cells in view within a decade or two of the strongest,
 * a straight scale lit nearly all of them (a map of one color, nothing standing out); this, the weaker in the dark
 * and the clusters lit (measured, 8 years to 1 year in view: 2 left the bed muddy, 3 lost it).
 */
const HEAT_CURVE = 2.5;
/** A gradient heatmap's weakest cells, up to this far into its gradient, fading into the chart (else opaque). */
const HEAT_FADE = 0.3;
/** How strong a cell's amount reads against the reference `most`, 0 to 1 (see heatColor). @param {number} value @param {number} most */
const heatFelt = (value, most) => (value && most ? clamp(1 + Math.log10(Math.abs(value) / most) / HEAT_DECADES, 0, 1) ** HEAT_CURVE : 0);
/** The amount a cell reads as strong as `felt` (0 to 1) against the reference `most` (heatFelt's inverse). @param {number} felt @param {number} most */
const heatAmount = (felt, most) => most * 10 ** (HEAT_DECADES * (felt ** (1 / HEAT_CURVE) - 1));
/** A gradient's colors as drawn in the theme now, each its channels (see gradientTable): a heatmap's, a heatmap pane's rows'. @param {[number, string][]} stops */
const gradientChannels = (stops) => gradientTable(stops).map(channels);
/**
 * A cell on a gradient (`table`, see gradientChannels): its color at its `place` in it (0 to 1), opaque but for the
 * weakest (as strong as `felt`, under HEAT_FADE: fading into the chart's own ground, a dark one's or a light one's), times `opacity`.
 * @param {number[][]} table @param {number} place @param {number} felt @param {number} opacity @param {Uint8ClampedArray} pixels @param {number} at
 */
function gradientCell(table, place, felt, opacity, pixels, at) {
  const [r, g, b] = table[placeIndex(place, table.length)];
  pixels[at] = r;
  pixels[at + 1] = g;
  pixels[at + 2] = b;
  pixels[at + 3] = Math.round(255 * opacity * Math.min(1, felt / HEAT_FADE));
}
/** A URPD day's amounts, a row each, as a measure reads them: its coins, their capital, their unrealized P&L. @param {any} day @param {string} measure @returns {Float32Array} */
const amountsOf = (day, measure) => (measure === "capital" ? day.capital : measure === "pnl" ? day.pnl : day.supply);
/** The largest of a day's cells, by measure. @param {any} day @param {string} measure */
const heatMost = (day, measure) => (day.most ??= {})[measure] ??= amountsOf(day, measure).reduce((most, value) => Math.max(most, Math.abs(value)), 0);
/**
 * A cell's color, by its amount against the reference `most` (the map's: the open day's largest cell, the same for
 * every day and every view, see viewHeat), as the heatmap's look paints it, into `pixels` at `at` (its red, green,
 * blue and opacity; the library's grid gives each pixel its strongest cell); none, left as it is: its strength on a
 * log scale over `HEAT_DECADES` (a ten-thousandth of it and under not seen), felt as `HEAT_CURVE` says: what changes
 * from one day to the next is what the coins did (each day against its own largest, a day with one big cluster
 * dimmed its whole column), and earlier eras' smaller amounts still read; with a gradient, its color there (unrealized P&L's from its loss's at 0%
 * through no gain at 50% to its profit's at 100%), opaque but for the weakest (`HEAT_FADE`, as a single color
 * fades: the chart's own ground under them, a dark one's or a light one's); else its color (a loss's, its color
 * below), as opaque as it's felt.
 * @param {number} value @param {number} most @param {{ measure: string, up: number[], down: number[], opacity: number, gradient: number[][] | null }} look
 * @param {Uint8ClampedArray} pixels @param {number} at
 */
function heatColor(value, most, look, pixels, at) {
  const felt = heatFelt(value, most);
  if (!felt) return;
  if (look.gradient) return gradientCell(look.gradient, look.measure === "pnl" ? 0.5 + (Math.sign(value) * felt) / 2 : felt, felt, look.opacity, pixels, at);
  const [r, g, b] = value < 0 ? look.down : look.up;
  pixels[at] = r;
  pixels[at + 1] = g;
  pixels[at + 2] = b;
  pixels[at + 3] = Math.round(255 * look.opacity * felt);
}
/**
 * A heatmap's look (see heatLook), and as last drawn: whether it's been (`drawn`; `undrawable`, not two frames after
 * its chart was, see settle), the days a column holds (`step`), the days it showed and wanted (`keep`), its
 * reference (`reference`: the open day's largest cell, see viewHeat); its colors (`gradient`, its gradient's as
 * shown, see gradientTable) and all it's painted by (`paint`, its strips' key).
 * A profile's (`kind`, see viewProfile): its holder groups and their colors, each one's day last shown (`last`), its
 * shares' colors (in profit, in loss), its close's line's (`mark`) and the card's (`halo`); its close and share in
 * profit (`shares`, see profileShares).
 * @typedef {{ id: string, cohort: string, weight: string, grid: ReturnType<typeof heatGrid>, g: Int32Array, days: string[], measure: string, up: number[], down: number[], opacity: number, gradient: number[][] | null, paint: string, kind: string, cumulative: boolean, cohorts: ReturnType<typeof profileShades>[], last: Map<string, { key: string, day: HeatDay & { rows: Int32Array } }>, gain: string, lose: string, halo: string, backdrop: string, mark: string, profit?: { grid: ReturnType<typeof heatGrid>, close: number, days: (HeatDay | undefined)[], share: number | null }, shares?: { close: number, share: number }, drawn?: boolean, undrawable?: boolean, step?: number, keep?: Set<string>, reference?: number }} HeatLook
 */
/**
 * Each heatmap's look, by its layer's id: what it draws and how, read by its layer's drawing (see Heat).
 * @type {Map<string, HeatLook>}
 */
const heatLooks = new Map();
/** Each timeline's days (see heatCalendar), made once a timeline. @type {WeakMap<number[], { g: Int32Array, days: string[] }>} */
const heatCalendars = new WeakMap();
/**
 * A timeline's days: each bar's (`g`), the day its stretch ends on (a fine interval's bars a day, one), and each
 * day's date (`days`); the last bar's, the open one, "latest" where the timeline reaches the latest (`live`, as it
 * does unless it says not), else its own date, a step on from its start as the bars before it step (a chunk of
 * history ends in the past). @param {{ times: number[], live?: boolean }} data
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
/** A heatmap's look, for its series' renderer (see heatLooks). @param {Chart} doc @param {Layer} layer @param {Resolved} data @param {number} opacity */
// (Its profile's colors: an unrealized P&L's own two, else the chart's green and red.)
/**
 * A profile's group's bars' colors: in profit and in loss, its color strong and faint over the card, solid (a profile
 * over other layers hides them), the layer's opacity over that. @param {string} id @param {string} color (hex) @param {number} opacity
 */
const profileShades = (id, color, opacity) => {
  const card = channels(css("--card")), own = channels(color), over = (/** @type {number} */ strength) => `rgb(${own.slice(0, 3).map((value, i) => Math.round(card[i] + (value - card[i]) * strength)).join(" ")} / ${opacity})`;
  return { id, gain: over(0.72), loss: over(0.32), dimGain: over(0.72 * PROFILE_DIM), dimLoss: over(0.32 * PROFILE_DIM) };
};
const heatLook = (doc, layer, data, opacity) => ({ id: layer.id, cohort: layer.type === "profile" ? layer.profile?.[0] ?? "all" : layer.heatmap ?? "all", weight: layer.weight ?? "", grid: heatGrid(doc, layer), ...heatCalendar(data), measure: measureOf(layer), up: channels(ink(layer.color)), down: channels(ink(belowOf(layer))), opacity, kind: layer.type, cumulative: layer.type === "profile" && Boolean(layer.cumulative), cohorts: layer.type === "profile" ? (layer.profile ?? []).map((id) => profileShades(id, ink(profileColor(layer, id)), opacity)) : [], last: new Map(), gain: ink("green"), lose: ink("red"), halo: css("--card"), backdrop: alpha(css("--card"), opacity), mark: css("--ink"), gradient: layer.byGradient && layer.gradient ? gradientChannels(layer.gradient) : null, paint: `${measureOf(layer)}|${ink(layer.color)}|${ink(belowOf(layer))}|${opacity}|${layer.byGradient ? JSON.stringify(layer.gradient) : ""}|${light}` });
/** The day a block of `by` days is drawn as: its last (the open one, at most). @param {HeatLook} heat @param {number} g */
const heatOwner = (heat, g, /** @type {number} */ by) => Math.min(Math.ceil(g / by) * by, heat.days.length - 1);
/** A day's key in heatDays, by its count (see heatCalendar). @param {HeatLook} heat @param {number} g */
/** A heatmap day's key (see heatDays): its cohort, weight, density and day. @param {string} cohort @param {string} weight @param {string | undefined} agg @param {string | number} day */
const heatDayKey = (cohort, weight, agg, day) => `${cohort}|${weight}|${agg}|${day}`;
const heatKeyOf = (heat, g) => heatDayKey(heat.cohort, heat.weight, heat.grid?.agg, heat.days[g]);
/** Whether a day's in hand, asked for, or failed for good: one failed for a while, its time come, let go of to be asked again. @param {string} key */
const heatDayHeld = (key) => {
  if ((heatDays.get(key)?.retryAt ?? Infinity) < Date.now()) heatDays.delete(key);
  return heatDays.has(key);
};
/**
 * The day a column of `step` days shows (`owner`, its last): its own once it's come, else the coarser blocks' over
 * it meanwhile (up to 16 times as many days); none (or its own failed), null. Drawn and read alike (see viewHeat,
 * heatReading).
 * @param {HeatLook} heat @param {number} owner @param {number} step @returns {{ key: string, day: HeatDay & { rows: Int32Array } } | null}
 */
function heatShown(heat, owner, step) {
  // (Its own day failed: none, not another day's in its place.)
  if (heatDays.get(heatKeyOf(heat, owner))?.failed) return null;
  for (let by = step; by <= step * 16; by *= 2) {
    const key = heatKeyOf(heat, by === step ? owner : heatOwner(heat, owner, by)), day = heatDays.get(key);
    if (day?.rows) return (day.used = ++heatUse), { key, day: /** @type {HeatDay & { rows: Int32Array }} */ (day) };
  }
  return null;
}
/**
 * A profile's reading, on the day it shows (see profilePoint), as it draws it (each group's day, else the one it
 * showed before, see viewProfile), by the binned layers' rule (see hoveredIn): hovered, what's at the pointer's
 * price (see profileAt); else the share of its holder groups' coins in profit, under that day's close ("65% in
 * profit"). Nothing drawn, none.
 * @param {HeatLook} look @param {Layer} layer @param {ReadPoint} point @returns {{ amount: string, at: string } | null}
 */
function profileReading(look, layer, point) {
  Object.assign(look, heatCalendar(/** @type {Resolved} */ (resolved)));
  const day = profilePoint().at;
  if (day < 0 || day >= look.g.length || !look.grid) return null;
  // (Nothing it draws (its days still asked for or failed, no close to split them at, no coins: see viewProfile):
  // none, hovered too.)
  const shown = profileDays(look, look.g[day]), close = profileClose(shown), share = close > 0 ? profileShare(look, shown, close) : null;
  if (share === null) return null;
  const at = hoveredIn(layer, point, look.grid);
  return at ? profileAt(look, at) : { amount: `≈${format(share, "percent")} in profit`, at: "" };
}
/**
 * What a profile reads at a height in its pane (see profileReading), as it's drawn: the bin there, its amount (its
 * groups' sums, a bar pointed at, its group's alone, as its legend item names it) and its prices ("₿412,345" at
 * "$84,120–$85,130"); no bin there, zero at that price's row (as a heatmap's, see heatReading). Drawn cumulative,
 * how far that price is from the close and the sum to it ("+5% ₿1,690,123"). Under the rows (a cent, see
 * hoveredIn), none, either way. @param {HeatLook} look @param {NonNullable<ReturnType<typeof hoveredIn>>} at
 */
function profileAt(look, { y, price, row }) {
  const engine = drawn.get(look.id)?.engine, grid = look.grid, unit = measureUnit(look.measure);
  // (Not painted yet (see the library's Profile.column), none.)
  if (!(engine instanceof Profile) || !engine.column || !grid || row < 0) return null;
  // (As it's drawn: summed from the close, see the library's Profile.sumsAt; else bars.)
  const sums = engine.sumsAt(y);
  if (sums) return { amount: `${fromClose((price / engine.at - 1) * 100)} ${format(sums.reduce((sum, value) => sum + value, 0), unit)}`, at: "" };
  const read = engine.readAt(y, litCohort(look.id) || undefined) ?? { from: row, to: row + 1, sum: 0 };
  return { amount: format(read.sum, unit), at: priceSpan(grid.price(read.from), grid.price(read.to)) };
}
/**
 * The profiles whose bars the pointer's among (on their pane, as drawn, see seriesPane; in the column each drew at
 * the plot's right edge, see the library's Profile.covers; one that drew nothing, none), the frontmost first (its
 * first layer, else its last: see Pane): a pointer's or a finger's, not the keys'. @param {ChartPoint} param
 * @returns {Layer[]}
 */
function profilesUnder(param) {
  if (keyTime !== null || !param.point || param.pane === undefined) return [];
  // (Left of the widest column a profile draws (a pixel's slack: its bars' edge on the device's pixels, its curves'
  // stroke), none asked.)
  const x = param.point.x, plot = engine.time.width;
  if (x < plot - columnWidth(plot) - 1) return [];
  const mine = chart.layers.filter((layer) => {
    if (layer.type !== "profile" || layer.hidden) return false;
    const series = drawn.get(layer.id);
    return series?.engine instanceof Profile && series.engine.covers(x) && seriesPane(series) === param.pane;
  });
  return chart.panes[param.pane]?.reverse ? mine.reverse() : mine;
}
/**
 * A profile's bar under a point (its pane's pixels), as last drawn: its group (the library's Profile.hit); none
 * between them or left of them. @param {Layer} layer @param {{ x: number, y: number }} point
 */
function profileHit(layer, point) {
  const engine = drawn.get(layer.id)?.engine, hit = engine instanceof Profile ? engine.hit(point.x, point.y) : null;
  return hit && { layer: layer.id, cohort: hit.group };
}
/** The group of a profile's bar pointed at, if it's still in it and shown (none: ""). @param {string} id */
const litCohort = (id) => ((layer) => (profileLit?.layer === id && layer && !layer.hidden && layer.profile?.includes(profileLit.cohort) ? profileLit.cohort : ""))(layerById(id));
/** Where the crosshair is: one set here at its price, on its scale as it's drawn now (see crossSet); else the pointer. */
function crossAt() {
  const pinned = crossSet && drawn.get(crossSet.layer), y = pinned && crossSet ? seriesY(pinned, crossSet.price) : null;
  return crossSet ? (pinned && y !== null ? { pane: seriesPane(pinned), y } : null) : heatAt;
}
/**
 * The binned layers' (by price: heatmaps, profiles) one rule for what they read: the crosshair's place, while a point
 * read follows it in a layer's pane (see readPoint), what's at its price read: its height, its price, and its row
 * of the layer's rows (see logRows; under them, a cent, -1); else none, their own at rest (a heatmap's day's close,
 * a profile's summary). @param {Layer} layer @param {ReadPoint} point @param {ReturnType<typeof logRows>} grid
 */
function hoveredIn(layer, point, grid) {
  const at = point.cross, series = drawn.get(layer.id);
  if (!at || at.pane !== seriesPane(series)) return null;
  const price = Number(seriesValue(series, at.y)), row = price > 0 ? grid.cell(price) : -1;
  return { y: at.y, price, row: Math.max(-1, row) };
}
/**
 * A heatmap's reading at a point, on the day drawn there (see heatShown): the cell at the price hovered (see
 * hoveredIn), else the day's close's; its amount ("₿412K") and the price it's at ("$62,034", its cell's floor),
 * the legend's on one line, a row's on two (a profile's, see profileReading). A row the day holds nothing in reads
 * zero; under the rows (a cent, see logRows), none. None drawn, none.
 * @param {Layer} layer @param {ReadPoint} point @returns {{ amount: string, at: string } | null}
 */
function heatReading(layer, point) {
  const look = heatLooks.get(layer.id), grid = look?.grid;
  if (!look || !grid || !resolved) return null;
  // (A profile's: its own day's, whatever day's pointed at.)
  if (look.kind === "profile") return profileReading(look, layer, point);
  Object.assign(look, heatCalendar(resolved));
  if (point.at < 0 || point.at >= look.g.length) return null;
  const shown = heatShown(look, heatOwner(look, look.g[point.at], look.step ?? 1), look.step ?? 1), day = shown?.day;
  if (!day?.rows) return null;
  const at = hoveredIn(layer, point, grid), cell = !at ? closeCell(day, grid, look.measure) : at.row >= 0 ? dayCell(day, at.row, look.measure) : null;
  return cell && cellText(cell, grid, look.measure);
}
/**
 * A heatmap's blocks in view, as its grid finds them before each draw (see Heat): each block of `step` days
 * drawn as its last day's (the coarser block's while it hasn't come); the days missing, wanted (the coarsest first);
 * those it shows and wants, kept (see heatTrim); its reference (the open day's largest cell, wanted first), its look
 * then. @param {HeatLook} heat @param {{ step: number, owners: number[] }} blocks @param {import("/modules/bitview-charts/grid.js").Grid} grid
 */
function viewHeat(heat, { step, owners }, grid) {
  // (Drawn: what waits for its days in (see heatIn) asks again, once this draw's done.)
  heat.drawn = true;
  if (heatWaiters.size) queueMicrotask(() => { for (const wake of [...heatWaiters]) wake(); });
  // (Nothing drawn: nothing wanted, nothing kept for it.)
  if (!heat.grid || !resolved || !owners.length) return void (heatWants.delete(heat.id), (heat.keep = undefined));
  // (The timeline's days as it is now: a live tick's new bar its own.)
  Object.assign(heat, heatCalendar(resolved));
  heat.step = step;
  const ownerAt = (/** @type {number} */ g, /** @type {number} */ by) => heatOwner(heat, g, by), keyOf = (/** @type {number} */ g) => heatKeyOf(heat, g), agg = heat.grid.agg;
  /** @type {Map<string, NonNullable<ReturnType<typeof heatWants.get>>[number]>} */
  const wanted = new Map();
  /** The days it shows and wants: kept however many cells (see heatTrim). */
  const keep = new Set();
  const want = (/** @type {number} */ g, /** @type {number} */ rank) => ((key) => {
    keep.add(key);
    if (!heatDayHeld(key) && !wanted.has(key)) wanted.set(key, { key, cohort: heat.cohort, weight: heat.weight, agg, day: heat.days[g], rank });
  })(keyOf(g));
  // (The map's reference, the open day's largest cell (see heatColor): wanted first, wherever the view is; until
  // it's come, none drawn.)
  const open = heatDays.get(keyOf(heat.days.length - 1));
  keep.add(keyOf(heat.days.length - 1));
  if (!open?.rows) want(heat.days.length - 1, Number.MAX_SAFE_INTEGER);
  // (Its last while a new block's is asked for.)
  const was = heat.reference;
  if (open?.rows) heat.reference = heatMost(open, heat.measure);
  // (Come or changed: what its gradient's places are worth, in its editor, after this frame's drawing.)
  if ((heat.reference ?? 0) !== (was ?? 0) && heat.id === selected) queueMicrotask(showScale);
  grid.setLook(`${heat.paint}|${heat.reference ?? 0}`);
  for (const owner of owners) {
    // (Its day, else the coarser ones' over it meanwhile; the missing ones wanted, the coarsest asked for first.)
    const shown = heatShown(heat, owner, step);
    if (shown) keep.add(shown.key);
    if (!heatDays.get(keyOf(owner))?.rows) want(owner, step);
    if (!shown) want(ownerAt(owner, step * 16), step * 16);
  }
  heat.keep = keep;
  // (Those in view now take the place of any it wanted before.)
  if (!wanted.size) return void heatWants.delete(heat.id);
  heatWants.set(heat.id, [...wanted.values()]);
  if (!heatPumping) (heatPumping = true), queueMicrotask(heatPump);
}
/**
 * The point a profile shows: the view's last day (the latest at its end), whatever's pointed at, as a visible range's
 * profile: panned or zoomed, another.
 */
const profilePoint = () => ({ at: lastShown(), rest: true });
/**
 * A profile's holder groups' days shown for a day (`g`, see heatCalendar): each one's own once it's in, else the one
 * it showed before (none for a day that failed), in their order; `now`, its own. Drawn and read alike (see
 * viewProfile, profileReading). @param {HeatLook} look @param {number} g
 */
function profileDays(look, g) {
  return look.cohorts.map((cohort) => {
    const id = cohort.id, key = `${id}|${look.weight}|${look.grid?.agg}|${look.days[g]}`, day = heatDays.get(key);
    if (day?.rows) (day.used = ++heatUse), look.last.set(id, { key, day: /** @type {HeatDay & { rows: Int32Array }} */ (day) });
    const had = day?.failed ? null : look.last.get(id);
    return had ? { day: had.day, cohort, key: had.key, now: had.key === key } : null;
  });
}
/** The close the days shown are split at: a group's own day's, else any's. @param {ReturnType<typeof profileDays>} shown */
const profileClose = (shown) => Number((shown.find((item) => item?.now) ?? shown.find(Boolean))?.day.close);
/**
 * The share of the days shown's coins in profit, last moved under the close, a percent; none, no coins. A row the
 * close falls in counts as far as the close is into it (its coins taken as even across it): a row doesn't say which
 * of its coins are under the close, and one side by its middle moved the share by points with the density (2024-07-08:
 * 9.3% at 100 rows a ×10, 12.9% at 200, 11.7% at 500; this: 12.3, 12.4, 12.2, the published 11.9). Kept while its days,
 * grid and close are the same: a crosshair reads the view's day too.
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
/**
 * A profile (see heatLike), as its library Profile is drawn (told before each draw): the view's last day (see
 * profilePoint), each holder group's, asked for as it's wanted (one not in yet, the one it showed before: no blank
 * between days); their amounts by price as bars from the plot's right edge, stacked, the first group's at the edge;
 * split at the close, an older day's dashed across it (the latest's, the price's own line marks), and the share of
 * their coins each side on the axis (see profileShares). In the room the chart leaves for it after its latest, see
 * rightGap. @param {HeatLook} heat @param {import("/modules/bitview-charts/profile.js").Profile} profile
 */
function viewProfile(heat, profile) {
  // (Drawn: what waits for its days in (see heatIn) asks again, once this draw's done.)
  heat.drawn = true;
  if (heatWaiters.size) queueMicrotask(() => { for (const wake of [...heatWaiters]) wake(); });
  const grid = heat.grid;
  // (Its shares as drawn, for its axis (see profileShares): where they change, drawn again (the chart paints its axes
  // before its series: theirs read on the next frame).)
  const shown = (/** @type {{ close: number, share: number } | undefined} */ next) => {
    const was = heat.shares;
    // (As written: a live tick moving the share past what it says, nothing drawn again.)
    const said = (/** @type {{ share: number } | undefined} */ shares) => shares && `${format(shares.share, "percent")} ${format(100 - shares.share, "percent")}`;
    if (was?.close === next?.close && said(was) === said(next)) return;
    heat.shares = next;
    queueMicrotask(() => drawn.get(heat.id)?.redraw());
  };
  /** Its bars as they are now (each told once: a change, drawn again). @param {import("/modules/bitview-charts/profile.js").ProfileGroup[]} groups @param {number} at @param {string | null} closeLine */
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
  // (An older day's: its close dashed across it.)
  show(groups, close, g === heat.days.length - 1 ? null : alpha(heat.mark, 0.6));
}
/** The profiles' series given their shares on the axis (see profileShares): each once. */
const sharesShown = new WeakSet();
/**
 * A profile's shares on its axis, where its plot has no room for them (over its bars or the price): in loss just
 * over its close's tag, in profit just under it, in their colors on the card's (as text beside the tag); none
 * where it isn't drawn, or while its profile's faint (a spotlight on another: its tags gone, as every faint series').
 * A layer of its series' own (see ownLayer).
 * @param {string} id @param {any} series
 */
function profileShares(id, series) {
  if (sharesShown.has(series)) return;
  sharesShown.add(series);
  const side = series.side === "left" ? 1 : 0;
  // (Over everything, as its tags are asked again each time the axes are drawn: its shares as they are.)
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
/** A stroke with what adds nothing taken out: no point strays further than `tolerance` pixels from what's kept. */
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
/** A press held that's drawing, moving drawings or drawing a box. */
const gesturing = () => Boolean(placing?.held || moving || boxing || lensAt?.touch);
/** A pointer other than the one a gesture's held by. @param {PointerEvent} event */
const otherPointer = (event) => gesturing() && event.pointerId !== gesturePointer;
/**
 * A press on nothing while a drawing is selected (`pick` ""), or a finger's on one not selected (see above): let go
 * where it was pressed, the drawings it picks chosen (none, on nothing). @param {PointerEvent} event @param {string} pick
 */
function pressAt(event, pick) {
  const { clientX, clientY } = event;
  press(event, { end: (release) => release && !otherPointer(release) && !fingerCross.dismissing && Math.hypot(release.clientX - clientX, release.clientY - clientY) < 4 && chooseDrawings(pick ? [pick] : []) });
}
/** A right-click on a drawing: what the chosen ones do. */
const drawingMenu = (/** @type {{ x: number, y: number }} */ point) => ((ids) => menuOf($("plot-keys"), drawingRows(ids), point, "", { named: ids.length > 1 ? plural(ids.length, "drawing") : drawingNames().get(ids[0]) ?? "Drawing" }))([...chosen]);
/** What drawings do (one: its text, its label too). @param {string[]} ids @returns {Action[]} */
function drawingRows(ids, /** @type {HTMLElement} */ anchor = $("plot-keys")) {
  const several = ids.length > 1, one = several ? null : drawingById(ids[0]);
  return [
    ...(one?.kind === "text" ? [{ key: "text", icon: ICONS.pencil, label: "Edit the text", run: () => ((now) => now && startTyping(now, anchor))(drawingById(one.id)) }] : []),
    ...(one && LINES.has(one.kind) ? [{ key: "label", icon: ICONS.label, label: one.label ? "Edit the label" : "Add a label", run: () => ((now) => now && startLabel(now, now.at, anchor))(drawingById(one.id)) }] : []),
    ...flipsOf(ids).map(([across, way, icon]) => ({ key: `flip-${across ? "x" : "y"}`, icon: ICONS[icon], label: `Flip ${way}`, verb: "Flip", run: () => flipDrawings(ids, across) })),
    { key: "duplicate", icon: ICONS.duplicate, label: several ? `Duplicate ${ids.length} drawings` : "Duplicate", kbd: "⌘D", verb: "Duplicate", run: () => duplicateDrawings(ids) },
    // (What it was opened from gone with them, a ⋯'s: the focus on the plot, as the keys place it.)
    { key: "remove", icon: ICONS.trash, label: several ? `Remove ${ids.length} drawings` : "Remove", kbd: "⌫", verb: "Remove", danger: true, run: () => (keepFocus(null, () => $("plot-keys")), removeDrawings(ids)) },
  ];
}
/** Whether a two-point drawing's points are apart on the plot (3px at least; `apart`, as far). */
function pointsApart(/** @type {Drawing} */ drawing, apart = 3) {
  const series = drawn.get(drawing.layer), line = timeLine(), ends = series && line ? onScreen(drawing, series, line) : [];
  return ends.length > 1 && Math.hypot(ends[1].x - ends[0].x, ends[1].y - ends[0].y) >= apart;
}
/** A path's points changed as it's placed: shown, said, its undo and redo as they are now. */
function pathMoved() {
  renderBelt();
  repaintDrawings();
  syncUndo();
  $("chart-said").textContent = plural(/** @type {Drawing} */ (draft).points.length - 1, "point");
}
/** Whether a path is being placed (its first point placed). */
const placingPath = () => tool === "path" && Boolean(draft && placing);
/** A path being placed, kept as it is (the point following the pointer left out): two points at least, else let go. Whether one was. */
function finishPath() {
  if (!placingPath() || !draft) return false;
  const drawing = { ...draft, points: draft.points.slice(0, -1) }, again = toolKept;
  takeUp(again ? "path" : "", again, true);
  keepDrawing(drawing);
  if (drawingById(drawing.id)) $("chart-said").textContent = `Path of ${drawing.points.length} points`;
  return true;
}
/** A drawing its tool just made, added: its tool put down with it (not kept), the step it made noted, to take it up again if it's undone. */
function addDrawn(/** @type {Drawing} */ drawing) {
  addDrawing(drawing);
  if (!toolKept && drawingById(drawing.id) && edits.past.length) noteDrawn(/** @type {ChartStep} */ (edits.past.at(-1)).state, { kind: drawing.kind, picks });
}
/** A drawn step noted by its state (moved, not copied, as its state is written anew: see travel); the oldest let go past as many as a chart's steps kept (see commit). */
function noteDrawn(/** @type {string} */ state, /** @type {{ kind: DrawingKind, picks: number }} */ drawn) {
  drawnSteps.set(state, drawn);
  for (const key of drawnSteps.keys()) if (drawnSteps.size > 200) drawnSteps.delete(key);
}
/**
 * The steps a drawing tool picked for one drawing made (it was put down then), by the step itself (the chart as it
 * was, as the history keeps it; undone, the chart as it was then: see travel): its tool, and how many times you had
 * picked one then (`picks`: a tool put down after a drawing, or taken up or down by an undo, isn't a pick). `undoTook`:
 * the tool an undo took up again, none once one's picked otherwise.
 */
const drawnSteps = /** @type {Map<string, { kind: DrawingKind, picks: number }>} */ (new Map());
/**
 * Drawings taken hold of follow the pointer: by its body, every one selected with it; by one of its points, it alone (a
 * line by an end, a box by a corner, the opposite one held; a stroke, a line or a path by a corner or a side of its
 * box, scaled from the opposite).
 */
function moveTo(event) {
  const at = plotPoint(event.clientX, event.clientY, moving.rect), { originals, target, handle } = moving, line = timeLine();
  // A click isn't a move.
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
    // A side: the point on that side takes the pointer's time (its left or right) or value (its top or foot).
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
/** A stroke stretched by a corner of its box to a point: every point as far across and up its box as it was, the opposite corner held. */
function scaled(/** @type {Drawing} */ drawing, series, line, /** @type {number} */ corner, at) {
  const points = onScreen(drawing, series, line), box = boundsOf(drawing, points);
  // A side (4 top, 5 right, 6 foot, 7 left) stretches it one way: the other way, as it is.
  if (corner >= 4) at = { x: corner === 5 ? at.x : corner === 7 ? at.x : box.right, y: corner === 4 || corner === 6 ? at.y : box.bottom };
  const west = corner === 0 || corner === 3 || corner === 7, north = corner < 2 || corner === 4;
  const held = { x: west ? box.right : box.left, y: north ? box.bottom : box.top }, from = { x: west ? box.left : box.right, y: north ? box.top : box.bottom };
  // (The corner under the pointer, its box's, on its points: it can be taken past the held one. A flat stroke, no
  // height or no width, keeps it.)
  const sx = from.x === held.x ? 1 : (at.x - held.x) / (from.x - held.x), sy = from.y === held.y ? 1 : (at.y - held.y) / (from.y - held.y);
  return points.flatMap((point) => {
    if (!finite(point)) return [];
    const value = seriesValue(series, held.y + (point.y - held.y) * sy);
    return value === null || !Number.isFinite(value) ? [] : [pointOf(timeOfBar((held.x + (point.x - held.x) * sx - line.at) / line.step), value)];
  });
}
/**
 * A drawing's points moved by pixels on its pane: across by whole bars (a stroke, a text: as far as it's moved), up or
 * down on its scale. @param {Drawing} drawing @returns {[number, number][]}
 */
function movedBy(drawing, series, line, dx, dy) {
  const bars = drawing.kind === "brush" || drawing.kind === "text" ? dx / line.step : Math.round(dx / line.step);
  return drawing.points.map(([time, value]) => {
    const y = dy ? seriesY(series, value) : null, to = y === null ? null : seriesValue(series, y + dy);
    return pointOf(timeOfBar(barOf(time) + bars), to !== null && Number.isFinite(to) ? to : value);
  });
}
/** A copy of each drawing, a little down and to the right of it (a bar at least), the copies selected. */
function duplicateDrawings(/** @type {string[]} */ ids) {
  const line = timeLine(), copies = /** @type {Drawing[]} */ (ids.map(drawingById).filter(Boolean)).flatMap((drawing) => {
    const series = drawn.get(drawing.layer);
    return series && line ? [{ ...structuredClone(drawing), id: uid(), points: movedBy(drawing, series, line, Math.max(1, Math.round(12 / line.step)) * line.step, 12) }] : [];
  });
  addDrawings(copies);
}
/**
 * Drawings flipped as one, in place: each point across the box they make together, left for right (`across`), else
 * top for bottom. Across, by their places on the timeline; up and down, as they show on their pane (a log scale's
 * too), each pane's apart. A line across keeps its value flipped left for right, a line down its date top for
 * bottom.
 */
function flipDrawings(/** @type {string[]} */ ids, /** @type {boolean} */ across) {
  const drawings = /** @type {Drawing[]} */ (ids.map(drawingById).filter(Boolean)), flipped = new Map();
  // (What a flip moves: across, the dates (a line across has none that shows); up and down, the values (a line down's
  // neither).)
  const moves = (/** @type {Drawing} */ drawing) => drawing.kind !== (across ? "hline" : "vline");
  if (across) {
    const bars = drawings.filter(moves).flatMap((drawing) => drawing.points.map(([time]) => barOf(time))), sum = Math.min(...bars) + Math.max(...bars);
    for (const drawing of drawings) if (moves(drawing)) flipped.set(drawing.id, drawing.points.map(([time, value]) => pointOf(timeOfBar(sum - barOf(time)), value)));
  } else {
    // (A pane's together, across its layers' scales: one line they flip about, in its pixels.)
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
/** Whether flipping drawings changes how they show (`across`, left for right): one line across, down, a box or a text, never. */
const flips = (/** @type {Drawing[]} */ drawings, /** @type {boolean} */ across) => drawings.length > 1 ? drawings.some((drawing) => drawing.kind !== (across ? "hline" : "vline")) : ["trend", "path", "brush"].includes(drawings[0]?.kind ?? "");
/** The two flips (across or not, its way, its icon, what it does), as they're offered. */
const FLIPS = /** @type {const} */ ([[true, "horizontally", "flipX", "Left for right"], [false, "vertically", "flipY", "Top for bottom"]]);
/** The flips that change how some drawings show, by their ids. */
const flipsOf = (/** @type {string[]} */ ids) => FLIPS.filter(([across]) => flips(/** @type {Drawing[]} */ (ids.map(drawingById).filter(Boolean)), across));
/** A drawing selected, from ⌘K: the view goes to it first where it's out of it (at the zoom it has). */
function selectInView(/** @type {string} */ id) {
  const drawing = drawingById(id), range = shownSpan();
  if (!drawing) return;
  chooseDrawings([id]);
  // (The chart then has the keys, ⌘K closed: its arrows move it; by touch, none.)
  // (⌘K gives the focus back as it closes: taken from there.)
  const keys = () => $("plot-keys").focus({ preventScroll: true });
  if (!TOUCH) isOpen($("palette")) ? onClosed($("palette"), keys, true) : keys();
  // (A line across is at every date: it's in view wherever the view is.)
  if (!range || drawing.kind === "hline") return;
  const bars = drawing.points.map(([time]) => barOf(time)), first = Math.min(...bars), last = Math.max(...bars);
  if (last >= range.from && first <= range.to) return;
  const half = Math.max((range.to - range.from) / 2, (last - first) / 2 + 2), middle = (first + last) / 2;
  glide({ from: middle - half, to: middle + half });
}
/**
 * The drawings selected moved by a key, as a drag moves them: across a bar (as many as a pixel holds, if more), up or
 * down a pixel; ⇧, ten times as far. `end`: only the last point (a line's end, a box's corner): it's resized. One undo
 * while the keys go on.
 */
function nudgeDrawings(/** @type {string} */ key, far = false, end = false, repeat = false) {
  const line = timeLine(), ids = [...chosen];
  if (!line || !ids.length) return;
  const times = far ? 10 : 1, across = key === "ArrowLeft" ? -1 : key === "ArrowRight" ? 1 : 0, down = key === "ArrowDown" ? 1 : key === "ArrowUp" ? -1 : 0;
  // (A step: a bar, or as many as a pixel holds; ⇧, ten of them.)
  const dx = across * Math.max(1, Math.round(1 / line.step)) * times * line.step, dy = down * times;
  if (end && !ids.some((id) => POINTED.has(drawingById(id)?.kind ?? ""))) return void (!repeat && notice("Only lines through points and boxes can be resized. The arrow keys move this one"));
  commit((doc) => doc.drawings?.forEach((item) => {
    const series = drawn.get(item.layer);
    if (!ids.includes(item.id) || !series || (end && !POINTED.has(item.kind))) return;
    item.points = end ? [...item.points.slice(0, -1), movedBy({ ...item, points: item.points.slice(-1) }, series, line, dx, dy)[0]] : movedBy(item, series, line, dx, dy);
  }), { coalesce: `${end ? "resize" : "nudge"}:${ids.join(",")}` });
}
/** Drawings into the chart (one undo), selected. @param {Drawing[]} drawings */
function addDrawings(drawings) {
  if (!drawings.length) return;
  if ((chart.drawings?.length ?? 0) + drawings.length > MAX_DRAWINGS) {
    repaintDrawings();
    return notice(`A chart holds up to ${MAX_DRAWINGS} drawings`);
  }
  commit((doc) => (doc.drawings ??= []).push(...drawings));
  // (Selected, unless a tool's kept in hand: the next one's drawn.)
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

/** The cursor a drag holds wherever the pointer goes (none: let go). */
function hold(/** @type {string} */ cursor) {
  document.body.toggleAttribute("data-held", Boolean(cursor));
  document.body.style.setProperty("--held", cursor || "auto");
}
/** The box, as dragged: over its pane, in the canvas. */
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
/** Whether a segment crosses a box (or lies in it): clipped to it, something's left. */
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
/** Whether a drawing is touched by a box: a line crossing it, a box or a text overlapping it, a stroke passing through it. */
function inBox(/** @type {Drawing} */ drawing, points, box, size) {
  const [a, b] = points, overlaps = (other) => other.left <= box.right && other.right >= box.left && other.top <= box.bottom && other.bottom >= box.top;
  if (THROUGH.has(drawing.kind)) return lineThrough(drawing, points, size).some((point, i, through) => i > 0 && crossesBox(through[i - 1], point, box));
  if (drawing.kind === "hline") return a.y >= box.top && a.y <= box.bottom;
  if (drawing.kind === "vline") return a.x >= box.left && a.x <= box.right;
  // (An ellipse where it is, not its corners: the box's point nearest its middle, inside it.)
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

/** The field typed in, dressed as the words will be: their color, size, background. */
function dressField(/** @type {Drawing} */ drawing) {
  const field = $("draw-text"), size = drawing.size ?? TEXT_SIZE;
  field.style.setProperty("--c", cssColor(drawing.color));
  field.style.setProperty("--on", wordsColor(drawing));
  field.style.setProperty("--size", `${size}px`);
  field.style.setProperty("--line", `${Math.round(size * 1.33)}px`);
  field.dataset.back = drawing.back ?? "";
  field.hidden = false;
}
/** The field over where its text is drawn, as wide and tall as what's typed (nothing yet: its placeholder, what it's for). */
function placeField() {
  const field = /** @type {HTMLTextAreaElement} */ ($("draw-text")), drawing = writing?.drawing, layer = drawing && layerById(drawing.layer);
  const series = layer && drawn.get(layer.id), line = timeLine(), pane = layer && paneBox(chart.panes.findIndex((item) => item.id === layer.pane));
  if (!drawing || !series || !line || !pane) return;
  const canvasBox = $("canvas").getBoundingClientRect(), plotLeft = engine.element.getBoundingClientRect().left + engine.left;
  if (writing.label) {
    // A label: centered where it sits (a trend line's at its angle; its field level).
    const size = sketches.get(series)?.size, place = size && labelPlace(drawing, onScreen(drawing, series, line), size, field.value || field.placeholder);
    if (!place) return;
    return void Object.assign(field.style, { left: `${plotLeft - canvasBox.left + place.x - place.width / 2 - 6}px`, top: `${pane.top - canvasBox.top + place.y - place.height / 2}px`, width: `${Math.ceil(place.width) + 12}px`, height: `${place.height}px` });
  }
  // A text: its tag around its middle, as wide and tall as what's typed.
  const place = textPlace(drawing, onScreen(drawing, series, line)[0], field.value || field.placeholder);
  Object.assign(field.style, { left: `${plotLeft - canvasBox.left + place.x - place.width / 2 - 6}px`, top: `${pane.top - canvasBox.top + place.y - place.height / 2}px`, width: `${Math.ceil(place.width) + 12}px`, height: `${place.height}px` });
}
export {
  addDrawn, amountsOf, boxTo, bubbleColumns, cellText, clearDrawings, closeCell, crossAt, drawingMenu, drawingRows,
  drawnSteps, dressField, duplicateDrawings, finishPath, flipDrawings, flipsOf, fromClose, gesturing, gradientCell, gradientChannels, HEAT_ABOUT,
  HEAT_COLUMN, HEAT_FADE, HEAT_SHORT, heatAmount, heatCalendar, heatCode, heatColor, heatCome, heatDayHeld, heatDayKey,
  heatDays, heatExtras, heatFelt, heatFresh, heatGrid, heatIn, heatLook, heatLooks, heatMost, heatName, heatPath,
  heatPump, heatReading, heatRows, heatSection, heatSeries, heatShown, heatWaiters, heatWants, hold, inBox, litCohort,
  logRows, mostIn, moveTo, noteDrawn, nudgeDrawings, otherPointer, pathMoved, placeField, placeSaid, placingPath,
  pointsApart, pressAt, priceSpan, PROFILE_ABOUT, PROFILE_SHORT, profileHit, profileName, profileShares, profilesUnder,
  readingLine, removeDrawing, removeDrawings, rowsOf, selectInView, setCohorts, showBox, shownHeats, simplify,
  toggleCohort, viewHeat, viewProfile,
};
