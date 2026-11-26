// @ts-check
import { askRest, command, gestures, grid, heatmaps, inspector, loadRest, menus, pen, presetsEdit, withRest, xkcd } from "/studio/later.js";
import { BitviewClientBase, _endpoint, dateToIndex, indexToDate } from "/modules/bitview-client/core.js";
import { Navigator, windowOn } from "/modules/bitview-charts/navigator.js";
import { createChart, press, picture as chartPicture, Candles, Gradient, Levels, compact as libraryWords, lastIn } from "/modules/bitview-charts/index.js";
import { barCorner as BAR_CORNER } from "/modules/bitview-charts/bar-style.js";
import { holderColor, recencyColor, wheelColor } from "/studio/palette.js";
import { hexToOklch, nameDistance, ofPalette, onCurve, pageInk, SWATCHES, TONE, warmLift, isColor } from "/studio/color.js";
import { AGES, HOLDERS, capitalized, dimensionBand, holderName, isLeaf, isWindowKey, windowAdjective, windowNamed, windowSaid, windowShort } from "/studio/words.js";
import { presetsKept } from "/studio/presets-kept.js";
import { initialsOf, initialsWord } from "/studio/initials.js";
import { aside, catalogWalk, isSeriesName } from "/studio/catalog-walk.js";
import { startWorker } from "/studio/workers.js";
import {
  $, all$, CLICK, closest, composing, drawKeys, erasing, escapeHtml, holdPage, KEY_STEPS, keycaps, keysOf, keyText,
  MAC, MEDIA, NAMED_KEY, scrollsY, SLOP, stepIn, TOUCH, typing, upDown, viewport, focusKey, restoreFocus, isOpen, onClosed, fadeOut, cutFade, fading, scrolled, fixedOrigin, placeFixed,
} from "/studio/dom.js";
import { motionClock } from "/modules/motion/clock.js";
import { reduced as motionReduced, whenReduced } from "/modules/motion/reduced.js";
import { until, whenIdle, serial } from "/studio/timing.js";
import { clamp, hashOf, lowerBound, uid, onScaleAll, meansOf } from "/studio/math.js";
import {
  ago, dollarsAndCents, format, FORMATS, grouped, humanize, humanizeKey, MONTHS, plural, showsTimes, toTheCent,
  UNIT_NAMES, unitOf,
} from "/studio/format.js";
import { CHEVRON, dashGlyph, ICONS } from "/studio/icons.js";
import {
  asideLayers, asRows, automaticZero, drawnInto, nameLayer, averageBlocked, belowOf, blankChart, BUBBLE_HIDE, BUBBLE_HIDES, BUBBLE_SIZE, BUBBLE_SIZES, BY_LEVELS,
  cloudLayers, COLOR_MODES, COLORED_FROM, DASHES, dashOf, DEFAULT_MEASURE, defaultOpacity, EDGES, edgesOf, failing,
  fittable, FLIP_APART, fromOf, HEAT_GRADIENTS, heatLike, hideText, hollowOf, kindColor, kindLook, layersColors, LINE_TYPE, lineTypeOf,
  LOOK_KEYS, makeLayer, MARK_PLACES, markable, MARKER, MAX_PANES, measureOf, newPaneIn, nextLetter, offScale, ONE_SCALE,
  oneScale, OPACITIES, opacityOf, OUTLINED, placedOn, poolPath, poolSeries, resultText, shapeKey, showView, signedMeasure, stretchResult,
  toggleMember, TREND_FLIP, TWO_KEPT, TWO_OR_ONE, twoSided, unfitted, WIDTHS,
} from "/studio/vocabulary.js";
import {
  BACKS, BOXES, KINDS, LABEL_MAX, labelText, LINES, lookOfDrawing, MAX_STROKE, PAIRED, PEN_COLOR, POINTED, pointOf,
  presetSize, SIZE_NAMES, SIZE_SHORT, SIZES, TEXT_SIZE, THROUGH, TOOLS,
} from "/studio/tools.js";
import {
  AVERAGES, CADENCE, CHUNK, FINE, FINE_STEP, fitsFrom, flowFits, HALVING_BLOCKS, INTERVAL_DAYS, INTERVAL_LABEL,
  INTERVAL_ORDER, intervalFits, INTERVALS, KNOWN_HALVINGS, MOST_CHUNKS, RANGES, spanDays, windowPoints,
} from "/studio/intervals.js";
import {
  alpha, amountOn, applyRange, asked, atLatest, autoScale, bandAt, baseOf, bubbleAt, Bubbles, build,
  built, BY_BLOCK, candleColumns, cardsDrawn, changeOver, chartFacts, chartPoint, chartTheme, chartWidth, chipText,
  chunkAt, chunked, chunks, Cloud, columnBy, crossHeld, crossOn, crossSet, cssColor, dailyOf, dashFor, dataKey,
  describePlot, dotLineOf, dotRadius, dotted, drawLayers, drawn, drawnWaiters, drawRows, drawSeries, dropCross, endReveal,
  EXACT_RANGE, fadedNow, fadeLayer, failedChunks, figuresRoom, fillBox, fineDrawn, fingerCross, fingers, firstInView,
  fitDots, fitSeries, fitSoon, fitSpacing, fitStrip, foldedInto, foldOf, hiddenPanes, layerShows, paneShows, forget, forgetSpans, glide,
  glideRows, glides, gone, Heat, hexCache, holdCross, idOf, indexOfTime, info, ink, inReveal,
  intervalsOf, keepPaneSizes, keyPoint, known, lastShown, latestChunk, layerFormats, layerOfSeries, legends,
  legendShown, levelOf, levelsKept, levelTags, lightBubble, lightLegend, lightRows, litLayer, liveColumn, lookKey,
  mapResult, marginsOf, markLegends, markSignals, MIN_BAR_SPACING, nameAxis, namePointed, NO_FORMAT, noValues, oklchOf,
  overviewSource, overviewSpan, paintedLayers, paintOf, paintPoint, paintProfileItems, paintSeries, paintSig, paletteOf,
  PANE, paneRect, PERCENT_FORMAT, periodAmount, pinCross, placeBrush, placeFoot, placeFull, placePlotKeys, pointLabel,
  prefetch, presetFits, presetShown, presetSpan, presetWindow, pressed, rangeText, reachOut, readPoint, refreshFailed, rescan, resolve, respot,
  retryInfo, reveal, rightGap, samePaint, sayValue, SCALE_MODES, seriesPane, seriesValue, seriesY, setColumn, setGap,
  setPresetShown, setReserve, settled, Shading, SHARE_FORMAT, showDates, shownDates, shownIndex, shownSpan, showSpan,
  showsPreset, showTimes, SIDES, sizeKeeping, sizePanes, spanned, spanOf, spans, stackBands, stale, Standard,
  stretchAcross, stretchDrawn, stretchNow, strip, styleSeries, tagAt, unitFor, units, updateLegends, updateReadout,
  updateValues, valueAt, valueFormatOf, valueSlot, valuesSoon, viewName, WHEEL_QUIET, whenSubmenu, windows,
  writeText, zoom,
} from "/studio/chart.js";
import {
  bandSum, FUNCTIONS, isBandSum, paneAt, parse, refsOf, renamed, seriesNamed, unfitHere, windowsOf,
} from "/studio/formulas.js";
import { modeOf, labelOf, paneMode, layerById, stackOf, lineLook, STACK_WIDTH, fillOf, named, bandOf, lineAt, gradientTable, titleOf, seriesMark, derivedTitle, legendEntry, levelSteps, rangeOf } from "/studio/looks.js";
import { renderRanges, fitTitle, titleReads, typedFrom, renderInterval, renderWindow, renderChartPath } from "/studio/toolbar.js";
import { validate, linkLookOf, contentOf, showLinkView, datedView, sameChart, readChart, twinOf, keepCarriedView, fromLink, readsOf } from "/studio/charts-read.js";
import { presetTree, presetNodeAt, presetName, placeNames, folderById, backToHome, asPlace, homeName, presetById, showAddress, prunePresets, presetChart, presetIndex, presetDoc, isLinkHash, routeOf, BASE, presetLink, presetEdited, looksOf, aFrame, idsNow, adoptPresets, pristine, keptAsStored, presetShowing, addressesSoon, presetLook, keptPreset, presetIdOf, presetsSettled } from "/studio/preset-charts.js";
import { fitted, laidOver, wordsOn, dropOwnLayers, sketches, Sketch, ownLayers, repaintDrawings, rulerReading, tagShapes, plotBox, timeLine, onScreen, tickOn, hover } from "/studio/sketch.js";
/** @import { ChartPoint, Resolved, Result, Span } from "/studio/chart.js" */
/** @import { Action, Command, Page } from "/studio/command.js" */
/** @import { Node, Vector } from "/studio/formulas.js" */
/** @import { Place, PresetNode } from "/studio/preset-charts.js" */
/** @import { HeatLook } from "/studio/heatmaps.js" */
/** @import { Landing } from "/studio/gestures.js" */
/** @import { Preset } from "/studio/presets-edit.js" */
/** @import { End } from "/studio/tools.js" */
/** @import { Chart, Drawing, DrawingKind, Folder, Layer, Pane } from "/studio/vocabulary.js" */

document.getElementById("page-info")?.remove();

export const CLIENT = { baseUrl: location.origin, timeout: 30_000, memCache: false, browserCache: false };
/** @type {import("/modules/bitview-client/index.js").BitviewClient} */
let client = /** @type {any} */ (Object.assign(new BitviewClientBase(CLIENT), {
  /** @param {string} name @param {any} index */
  seriesEndpoint(name, index) { return _endpoint(/** @type {any} */ (this), name, index); },
  /** @param {string} name @param {any} [options] */
  getSeriesInfo(name, options) { return /** @type {any} */ (this).getJson(`/api/series/${encodeURIComponent(name)}`, options); },
}));
const SERVER_HOST = new URL(client.baseUrl).host;
for (const element of document.querySelectorAll(".server-host")) element.textContent = SERVER_HOST;
/** @type {any} */
let tree;
/** @param {import("/modules/bitview-client/index.js").BitviewClient} full */
export function whole(full) {
  client = full;
  tree = full.series;
  EXAMPLE = `sma(${tree.price.close.name}, 200d)`;
  LIVE = new Map([[tree.price.ohlc.name, usd], [tree.price.close.name, usd], [tree.price.spot.name, usd], [tree.price.satsPerDollar.name, sats]]);
  knowCohorts(full.AGE_RANGE_NAMES);
}
/** @param {string[]} path */
function nodeAt(path) {
  let node = /** @type {any} */ (tree);
  for (const key of path) {
    if (!node || typeof node !== "object" || !Object.hasOwn(node, key)) return undefined;
    node = node[key];
  }
  return node;
}
/** @param {string} path */
function nameAt(path) {
  const node = nodeAt(path.split("."));
  return isLeaf(node) ? node.name : null;
}
let catalogIndex = /** @type {Map<string, string> | null} */ (null);
let seriesCount = 0;
/** @type {Map<string, readonly string[]> | null} */
let catalogIntervals = null;
const byBlock = new Set();
/** @type {Set<string>} */
const sharedTops = new Set();
const tokens = new Map(), probe = document.head.appendChild(document.createElement("i"));
let tokensFor = "";
// (Read on lists of their own, which no one listens to: a list's matches read takes a change for itself
// (Chromium), its listeners never hearing of it.)
const tokensMedia = [matchMedia(MEDIA.light), matchMedia(MEDIA.phone), matchMedia(MEDIA.moreContrast), matchMedia("(pointer: coarse)")];
probe.style.forcedColorAdjust = "none";
/** @param {Iterable<string>} [names] */
function readTokens(names) {
  const style = getComputedStyle(document.documentElement), colors = [];
  for (const name of names ?? style) {
    if (!name.startsWith("--")) continue;
    const written = style.getPropertyValue(name).trim();
    if (written.startsWith("light-dark(")) colors.push(name);
    else tokens.set(name, written);
  }
  probe.replaceChildren(...colors.map((name) => Object.assign(document.createElement("i"), { style: `color: var(${name})` })));
  colors.forEach((name, k) => {
    const [r, g, b, a = 1] = (getComputedStyle(probe.children[k]).color.match(/[\d.]+/g) ?? []).map(Number);
    tokens.set(name, `#${[r, g, b, ...(a < 1 ? [a * 255] : [])].map((n) => Math.round(n).toString(16).padStart(2, "0")).join("")}`);
  });
}
const tokensNow = () => [document.documentElement.dataset.theme, ...tokensMedia.map((media) => media.matches)].join();
/** @param {string} name */
const css = (name) => {
  const now = tokensNow();
  if (now !== tokensFor) (tokens.clear(), (tokensFor = now), readTokens());
  // (One the page's root doesn't list, read by its name: a browser that lists none (Chromium before 141), one unset.)
  if (!tokens.has(name)) readTokens([name]);
  return tokens.get(name);
};
/**
 * A Back or Forward whose motion the system played already (iOS's swipe from the screen's edge: the page before slid
 * in, as it was): Studio's own motion not played over it (its transitions neither: data-still), the step shown as it
 * lands. For the work the step starts (go's, a frame and a task on), then over.
 */
let stepShown = false;
const reduced = () => motionReduced() || stepShown;
let stillSteps = 0;
/** @param {boolean} shown */
function stillStep(shown) {
  const at = ++stillSteps;
  stepShown = shown;
  document.documentElement.toggleAttribute("data-still", shown);
  if (!shown) return;
  // (A motion under way as it comes, or held for its frames (beginOnTime), at its end at once: a sheet let go as the
  // swipe ended, its slide begun over the page the step showed. One that never ends, as it is.)
  for (const motion of document.getAnimations()) if (motion.timeline === document.timeline && Number.isFinite(Number(motion.effect?.getComputedTiming().endTime))) motion.finish();
  requestAnimationFrame(() => setTimeout(() => requestAnimationFrame(() => at === stillSteps && ((stepShown = false), document.documentElement.removeAttribute("data-still")))));
}
const MOTION = { micro: parseFloat(css("--dur-micro")), small: parseFloat(css("--dur-small")), panel: parseFloat(css("--dur-panel")), page: parseFloat(css("--dur-page")), ease: css("--ease"), layout: parseFloat(css("--dur-layout")), easeLayout: css("--ease-layout"), wordsWait: parseFloat(css("--words-wait")), wordsIn: parseFloat(css("--words-in")) };
const phone = matchMedia(MEDIA.phone);
const railed = () => document.body.dataset.sidebar === "collapsed";
const onRail = () => railed() && !phone.matches;
let listOwed = false;
const readRowSizes = () => ({ value: parseFloat(css("--row-value-size")), change: parseFloat(css("--row-change-size")) });
let rowSizes = readRowSizes();
phone.addEventListener("change", () => ((rowSizes = readRowSizes()), chart && renderLayers()));
phone.addEventListener("change", () => renderCharts());
// (Never scrolled above its top. WebKit can leave it there, an installed app turned on an iPhone Duo's inner screen
// keeping the upright status bar's room (iOS 27.2): what's drawn and what's touched then part by that much, a tap
// landing on what's above it. Put back, the page is what WebKit leaves it, every tap where it lands.)
addEventListener("scroll", () => scrollY < 0 && scrollTo(0, 0));
const heldUpright = matchMedia("not all and (max-height: 500px) and (min-width: 568px)");
const sheetWhole = () => heldUpright.matches && document.body.dataset.sheetSize === "full";
// Hover's looks, a pointer's that hovers only (every :hover in the style is so): iOS keeps what a finger tapped
// hovered till the next tap, a button lit as if still pressed, a swatch as if picked. As the device is now (a mouse
// put to a tablet), as the style's own touch rules are.
{
  const fingers = matchMedia(MEDIA.noHover), mark = () => document.documentElement.classList.toggle("can-hover", !fingers.matches);
  mark();
  fingers.addEventListener("change", mark);
}
const tablet = matchMedia("(max-width: 1100px)");
let studioVersion = "";
withRest(() => client.getVersion().then((said) => {
  studioVersion = typeof said === "string" ? said : "";
  $("home-version").textContent = studioVersion;
}).catch(() => {}));
for (const element of all$("[data-icon]")) element.insertAdjacentHTML("afterbegin", ICONS[element.dataset.icon] ?? "");

const timeAt = (/** @type {string} */ index, /** @type {number} */ i) => indexToDate(/** @type {"day1"} */ (index), i).getTime() / 1000;
/** @type {Map<string, number[]>} */
const indexTimes = new Map();
function timesAt(/** @type {string} */ index, /** @type {number} */ start, /** @type {number} */ count) {
  const times = indexTimes.get(index) ?? [];
  indexTimes.set(index, times);
  for (let i = times.length; i < start + count; i++) times.push(timeAt(index, i));
  return times.slice(start, start + count);
}
const indexAt = (/** @type {string} */ index, /** @type {number} */ time) => dateToIndex(/** @type {"day1"} */ (index), new Date(Math.max(time, timeAt(index, 0)) * 1000));
let light = false;
function shownOklch(/** @type {string} */ color) {
  const [l, c, h] = oklchOf(color);
  return /** @type {[number, number, number]} */ (light && !pageInk(color) && ofPalette(color) ? [onCurve(TONE, l) + warmLift(c, h), c, h] : [l, c, h]);
}
/** @type {{ name: string, l: number, c: number, h: number }[] | null} */
let names = null;
const namesAt = () => (names ??= xkcd.default.map(([name, hex]) => { const [l, c, h] = hexToOklch(hex); return { name, l, c, h }; }));
const ownName = (/** @type {string} */ key) => (key === "ink" ? (light ? "Black" : "White") : capitalized(key));
const colorNames = new Map();
/** A color's name: Bitview's own as they're named, any other by the nearest familiar name. Found once: weighing every familiar name each time a sheet's colors are written costs a phone's engine without a JIT. */
function colorName(color) {
  if (SWATCHES.includes(color)) return ownName(color);
  const key = `${light ? "light" : "dark"}:${color}`, had = colorNames.get(key);
  if (had) return had;
  const [l, c, h] = shownOklch(color);
  let best = namesAt()[0], nearest = Infinity;
  for (const name of namesAt()) {
    const distance = nameDistance(l, c, h, name.l, name.c, name.h);
    if (distance < nearest) (nearest = distance), (best = name);
  }
  if (colorNames.size > 500) colorNames.clear();
  colorNames.set(key, capitalized(best.name));
  return capitalized(best.name);
}
/** @param {Layer} layer */
const secondColor = (layer) => ((other) => (other && other !== layer.color ? other : layer.color === "red" ? "green" : "red"))(layer.type === "cloud" ? ownerOf(layer)?.layers.find((item) => item.id === layer.from?.[1])?.color : undefined);
/** @type {Map<string, { to: string, kind: boolean, before: Record<string, any>, after: Record<string, any> }>} */
const looksLeft = new Map();
/** @param {Layer} layer @param {(layer: Layer) => void} edit */
const reshape = (layer, edit) => {
  const from = shapeKey(layer), kind = twoSided(layer), next = /** @type {Layer} */ ({ ...layer });
  edit(next);
  if (shapeKey(next) === from) return void edit(layer);
  const look = /** @type {Record<string, any>} */ (layer), key = `${chart?.id}:${layer.id}`, was = looksLeft.get(key), back = was?.to === from;
  if (back) for (const key of LOOK_KEYS) if (look[key] === was.after[key]) was.before[key] === undefined ? delete look[key] : (look[key] = was.before[key]);
  const before = Object.fromEntries(LOOK_KEYS.map((key) => [key, look[key]]));
  if (twoSided(next) !== kind && !(back && was.kind === twoSided(next))) kindLook(layer, next);
  edit(layer);
  const type = layer.type;
  const after = Object.fromEntries(LOOK_KEYS.map((key) => [key, look[key]]));
  if (!COLORED_FROM.has(type)) delete after.colorFrom;
  if (!BY_LEVELS.has(type)) delete after.byLevels;
  if (chart?.layers.includes(layer)) looksLeft.set(key, { to: shapeKey(layer), kind: back ? was.kind : kind, before, after });
  if (layer.byLevels && BY_LEVELS.has(type)) inspector.colorLevels(layer);
};
/** @param {Layer} layer @param {Chart} [doc] */
const signalShown = (layer, doc = chart) => layer.type === "signal" && !layer.hidden && ((on) => Boolean(on && !on.hidden))(doc.layers.find((item) => item.id === layer.ref));
/** @param {Chart} doc @param {string} formula @returns {Layer | undefined} */
const markFor = (doc, formula) => {
  const can = markable(doc).sort((a, b) => Number(Boolean(a.hidden)) - Number(Boolean(b.hidden)));
  try {
    const node = parse(formula), by = (/** @type {Set<string>} */ read, /** @type {"letter" | "series"} */ key) => ((found) => found.find((layer) => !layer.hidden) ?? found[0])([...read].flatMap((name) => can.filter((layer) => layer[key] === name)));
    return by(refsOf(node), "letter") ?? by(refsOf(node, "series"), "series") ?? can[0];
  } catch {
    return can[0];
  }
};
/** @param {Chart} doc @param {Layer} trend */
const refsFor = (doc, trend) => doc.layers.filter((layer) => !layer.from && !placedOn(layer) && !heatLike(layer) && !trend.from?.includes(layer.id) && !readsOf(layer, doc).has(trend.letter));
/** @param {Chart} doc @param {Layer} trend */
const refFor = (doc, trend) => ((refs) => (refs.find((layer) => layer.type === "candles" || /^price/.test(layer.series ?? "")) ?? refs[0])?.id)(refsFor(doc, trend).filter((layer) => oneScale(layer, trend)));
/** @param {Layer} layer */
const formulaLine = (layer) => (layer.type === "signal" ? signalLine(layer) : ((result) => (result?.values ? `Formula · ${UNIT_NAMES[unitFor(layer, result)] ?? "Number"}` : "Formula"))(resolved?.results.get(layer.id)));
/** @param {ArrayLike<number> | undefined} values @returns {{ at: number, to: number, was: number, since: number }[]} */
const changesOf = (values) => {
  // (Read once a series: a signal's marks, its results and its times ask the same. Not one the live price writes in.)
  const kept = values && changesKept.get(values);
  if (kept) return kept;
  const out = [];
  let was = NaN, since = -1;
  for (let i = 0; i < (values?.length ?? 0); i++) {
    const value = /** @type {ArrayLike<number>} */ (values)[i];
    if (!Number.isFinite(value)) continue;
    const state = value > 0 ? 1 : value < 0 ? -1 : 0;
    if (state === was) continue;
    if (was === was) out.push({ at: i, to: state, was, since });
    since = was === was ? i : -1;
    was = state;
  }
  if (values && !ownLive.has(values)) changesKept.set(values, out);
  return out;
};
const changesKept = new WeakMap();
/** @param {Layer} layer @param {ArrayLike<number> | undefined} values @param {Result | null | undefined} [reference] */
const marksOf = (layer, values, reference = resolved?.results.get(layer.ref ?? "")) => {
  const own = reference?.values;
  const target = chart?.layers.find((item) => item.id === layer.ref), log = target ? modeOf(chart, target) === 1 : false;
  const drawn = (/** @type {number} */ i) => own != null && Number.isFinite(own[i]) && (!log || own[i] > 0);
  if (layer.place === "shade") return changesOf(values).flatMap((change) => (change.to || (layer.results && own && drawn(change.at) && stretchResult(change, reference) !== undefined) ? [change.at] : []));
  if (!own) return [];
  return changesOf(values).flatMap((change) => (drawn(change.at) && (change.to || (layer.results && stretchResult(change, reference) !== undefined)) ? [change.at] : []));
};
/** @param {Layer} layer */
const resultsOf = (layer) => {
  const values = resolved?.results.get(layer.id)?.values, own = resolved?.results.get(layer.ref ?? "");
  if (!values || !own?.values) return undefined;
  const results = changesOf(values).flatMap((change) => ((result) => (result === undefined ? [] : [result]))(stretchResult(change, own)));
  return results.length ? results.reduce((total, result) => total * Math.max(0, 1 + result), 1) - 1 : undefined;
};
/** @param {Layer} layer */
const signalLine = (layer) => ((on, values) => (!on ? "Pick a layer to mark it on" : on.hidden ? `Hidden while ${labelOf(on)} is hidden` : `Signal on ${labelOf(on)}${values ? ` · ${plural(marksOf(layer, values).length, "time")}${((total) => (total === undefined ? "" : ` · ${resultText(total)} in all`))(layer.results ? resultsOf(layer) : undefined)}${fineDrawn() ? " in the loaded range" : ""}${layer.results && resolved?.results.get(on.id)?.measurement ? " · Returns from raw values" : ""}` : ""}`))(chart.layers.find((item) => item.id === layer.ref), resolved?.results.get(layer.id)?.values);
/** @typedef {{ id: string, shown: string, brief: string, who: string, age: boolean }} HeatCohort */
/** @type {HeatCohort[]} */
let HEAT_COHORTS = [];
/** @type {Map<string, HeatCohort>} */
let HEAT_BY_ID = new Map();
const heatCohort = (/** @type {string} */ id) => HEAT_BY_ID.get(id);
/** @param {Layer} layer */
const heatKnown = (layer) => (layer.type === "profile" ? (layer.profile ?? []).every((id) => heatCohort(id)) : Boolean(heatCohort(layer.heatmap ?? "all")));
let HEAT_HOLDERS = /** @type {string[]} */ ([]), HEAT_AGES = /** @type {string[]} */ ([]);
/** @param {Record<string, { id: string, short: string }>} ageNames */
function knowCohorts(ageNames) {
  HEAT_COHORTS = [
    ...Object.keys(HOLDERS).map((key) => holderName(key)).flatMap((h) => (h.urpd ? [{ id: h.urpd, shown: h.shown, brief: h.brief, who: h.who, age: false }] : [])),
    ...Object.entries(ageNames).flatMap(([key, { id, short }]) => {
      const band = dimensionBand(key, AGES);
      return band ? [{ id: `age_${id}`, shown: capitalized(band.said), brief: short.replace("-", "–"), who: band.who, age: true }] : [];
    }),
  ];
  HEAT_BY_ID = new Map(HEAT_COHORTS.map((c) => [c.id, c]));
  HEAT_HOLDERS = HEAT_COHORTS.filter((c) => !c.age).map((c) => c.id);
  HEAT_AGES = HEAT_COHORTS.filter((c) => c.age).map((c) => c.id);
}
/** @param {string} id */
const cohortColor = (id) => holderColor(id.replace(/^age_(under|over)_/, "$1")) ?? ((at) => (at < 0 ? "gray" : wheelColor(at, HEAT_AGES.length)))(HEAT_AGES.indexOf(id));
/** @param {Layer} layer @param {string} cohort */
const profileColor = (layer, cohort) => (layer.cohortColors && Object.hasOwn(layer.cohortColors, cohort) ? layer.cohortColors[cohort] : cohortColor(cohort));
const cohortName = (/** @type {string} */ id) => heatCohort(id)?.shown ?? id;
const cohortWho = (/** @type {string} */ id) => heatCohort(id)?.who ?? id;
/** @param {string} id */
const cohortShort = (id) => heatCohort(id)?.brief ?? id;
/** @returns {boolean} */
function cumulativeLayer(/** @type {Chart} */ doc, /** @type {Layer} */ layer) {
  const runningTotal = (/** @type {string} */ name) => known.get(name)?.accumulation === "cumulative";
  if (layer.series) return runningTotal(layer.series);
  if (!layer.formula) return false;
  const roles = new Map(), visiting = new Set();
  /** @param {Node} node @returns {"constant" | "total" | null} */
  const kind = (node) => {
    if (node.k === "num") return "constant";
    if (node.k === "series") return runningTotal(node.v) ? "total" : null;
    if (node.k === "ref") {
      const source = doc.layers.find((item) => item.letter === node.v);
      return source ? role(source) : null;
    }
    if (node.k === "neg") return kind(node.a);
    if (node.k === "bin") {
      const a = kind(node.a), b = kind(node.b);
      if (a === "constant" && b === "constant") return "constant";
      if (node.op === "+" || node.op === "-") return a && b ? "total" : null;
      if (node.op === "*") return (a === "total" && b === "constant") || (a === "constant" && b === "total") ? "total" : null;
      return node.op === "/" && a === "total" && b === "constant" ? "total" : null;
    }
    if (node.k === "call") return isBandSum(node.f) ? kind(bandSum(node)) : node.f === "sum" && node.args.length === 1 ? "total" : null;
    return null;
  };
  /** @param {Layer} item @returns {"constant" | "total" | null} */
  const role = (item) => {
    if (item.series) return runningTotal(item.series) ? "total" : null;
    const had = roles.get(item.id);
    if (had !== undefined) return had;
    if (!item.formula || visiting.has(item.id)) return null;
    visiting.add(item.id);
    let value = null;
    try {
      value = kind(parse(item.formula));
    } catch {
    }
    visiting.delete(item.id);
    roles.set(item.id, value);
    return value;
  };
  return role(layer) === "total";
}
const zeroFit = (/** @type {Chart} */ doc, /** @type {Layer} */ layer, /** @type {number} */ mode, /** @type {boolean} */ stacked) => mode === 0 && (stacked || layer.fit === "zero" || (automaticZero(doc, layer, stacked) && cumulativeLayer(doc, layer)));
/** @param {Layer} layer */
const signedSignal = (layer) => layer.type === "signal" && Boolean(resolved?.results.get(layer.id)?.values?.some((/** @type {number} */ value) => value < 0));
/** @param {Layer} layer */
/** @type {Map<string, string>} */
const letterTones = new Map();
function letterChip(layer, tip = false) {
  const colored = (layer.colorFrom && ownerOf(layer)?.layers.find((other) => other.id === layer.colorFrom)) || layer;
  const color = ink(colored.color), key = `${color}|${light}|${tokensNow()}`;
  // A tone of its letter that reads on its own tint (4.5:1) on each ground it's on. Found once a color and theme: a layer's row and its settings write it each time.
  let tone = letterTones.get(key);
  if (tone === undefined) {
    const grounds = [css("--card"), css("--raise"), laidOver(css("--raise"), light ? "#000000" : "#ffffff", 0.07)];
    tone = /** @type {string} */ (grounds.reduce((each, ground) => wordsOn(each, laidOver(ground, color, 0.13)), color));
    if (letterTones.size > 500) letterTones.clear();
    letterTones.set(key, tone);
  }
  return `<span class="letter" style="--c:${escapeHtml(color)};--t:${escapeHtml(tone)}"${tip ? ` data-tip="${escapeHtml(labelOf(layer))}"` : ""}>${escapeHtml(layer.letter)}</span>`;
}
const formatName = (layer, result) => (layer.format ? FORMATS.find(([format]) => format === layer.format)?.[1] ?? "Number" : `Automatic (${UNIT_NAMES[unitFor({ ...layer, format: undefined }, result)] ?? "Number"})`);
/** @param {string} formula @param {string} index */
const flowsAt = (formula, index) => windowsOf(formula).every(({ days, flow }) => !flow || flowFits(days, index));
const windowTooLong = (/** @type {string} */ f, /** @type {string} */ span, /** @type {number} */ days, /** @type {string} */ index, /** @type {boolean} */ beyond, /** @type {boolean} */ shift) =>
  beyond && FINE.has(index) && !(shift && days > (Date.now() / 1000 - timeAt("day1", 0)) / 86400)
    ? failing(`${f} over ${span}: more than ${INTERVAL_LABEL[index]} loads; ${INTERVAL_LABEL[fitsFrom(days, index)]} and up hold it`, `Needs ${INTERVAL_LABEL[fitsFrom(days, index)]}+`)
    : new Error(`${f}: ${span} is longer than the data`);
/** @param {Chart} doc @param {Pane | undefined} pane @param {string} [span] */
function averageFor(doc, pane, span = pane?.average ?? "") {
  if (!pane || !Object.hasOwn(AVERAGES, span)) return { span: "", active: "", name: "", points: 1, disabled: "" };
  const days = spanDays(span), points = windowPoints(days, doc.index), blocked = averageBlocked(doc, pane);
  const disabled = blocked || (points <= 1 ? `At ${INTERVAL_LABEL[doc.index]}, ${AVERAGES[span]} changes nothing: choose a longer span`
    : FINE.has(doc.index) && points >= CHUNK ? windowTooLong("Average", span, days, doc.index, true, false).message : "");
  return { span, active: disabled ? "" : span, name: disabled ? "" : AVERAGES[span], points, disabled };
}

/** @returns {[string, string] | [string]} */
function spanParts(from, to, index = shownIndex()) {
  const a = new Date(from * 1000), b = new Date(to * 1000), fine = FINE.has(index);
  if (index.startsWith("year")) {
    const [first, last] = [String(a.getUTCFullYear()), String(b.getUTCFullYear())];
    return first === last ? [last] : [first, last];
  }
  const year = (date) => `’${String(date.getUTCFullYear()).slice(2)}`;
  const day = (date) => `${date.getUTCDate()} ${MONTHS[date.getUTCMonth()]}`;
  const clock = (date) => `${String(date.getUTCHours()).padStart(2, "0")}:${String(date.getUTCMinutes()).padStart(2, "0")}`;
  const sameYear = a.getUTCFullYear() === b.getUTCFullYear(), sameMonth = sameYear && a.getUTCMonth() === b.getUTCMonth();
  if (showsTimes(from, to, fine)) return sameMonth && a.getUTCDate() === b.getUTCDate() ? [`${day(a)} ${clock(a)}`, clock(b)] : [`${day(a)} ${clock(a)}`, `${day(b)} ${clock(b)}`];
  const month = (date) => MONTHS[date.getUTCMonth()];
  if (index.startsWith("month")) return !sameYear ? [`${month(a)} ${year(a)}`, `${month(b)} ${year(b)}`] : sameMonth ? [`${month(b)} ${year(b)}`] : [month(a), `${month(b)} ${year(b)}`];
  if (!sameYear) return [`${day(a)} ${year(a)}`, `${day(b)} ${year(b)}`];
  if (sameMonth && a.getUTCDate() === b.getUTCDate()) return [`${day(b)} ${year(b)}`];
  return [day(a), `${day(b)} ${year(b)}`];
}

let EXAMPLE = "sma(A, 200d)";

const minedHalvings = () => [...KNOWN_HALVINGS, ...Array.from({ length: Math.max(0, Math.floor(liveHeight / HALVING_BLOCKS) - KNOWN_HALVINGS.length) }, (_, k) => (liveAt || Date.now() / 1000) - (liveHeight - (KNOWN_HALVINGS.length + k + 1) * HALVING_BLOCKS) * 600)];
const nextHalvings = (count = 3) => (liveHeight ? Array.from({ length: count }, (_, k) => ((n) => ({ n, time: Date.now() / 1000 + (n * HALVING_BLOCKS - liveHeight) * 600 }))(Math.floor(liveHeight / HALVING_BLOCKS) + 1 + k)) : []);
/** @param {string} name @param {Set<string>} used @param {string} [asked] @returns {Partial<Layer>} */
function firstLook(name, used, asked) {
  const type = known.get(name)?.type ?? "", unit = type ? unitOf(type) : units.get(name), { look, told } = seriesLook(name, asked);
  const color = told?.color ?? (look === "baseline" ? "green" : (indexCatalog(), catalogIndex?.get(name))?.startsWith("price.") ? "ink" : unit === "usd" ? "green" : unit === "btc" || unit === "sats" ? "orange" : "");
  return { ...firstForm(name, asked), ...(color && (!used.has(color) || (look === "baseline" && color === "green")) ? { color } : {}) };
}
/** @param {string} name @param {string} [asked] */
const seriesLook = (name, asked) => {
  const own = presetLooks.get(name), ownType = own?.type ?? (isOhlc(name) ? "candles" : "line"), look = asked ?? ownType;
  return { look, told: look === ownType ? own : undefined };
};
/** @param {string} name @param {string} [asked] @returns {Partial<Layer>} */
const firstForm = (name, asked) => (({ look, told }) => ({ ...(look !== "line" ? { type: look } : {}), ...(told?.base ? { base: told.base } : {}), ...(told?.colors ? { colors: told.colors } : {}) }))(seriesLook(name, asked));
const isOhlc = (/** @type {string} */ name) => /^OHLC/.test(known.get(name)?.type ?? "");
const removedNames = new Map();
const goneLayer = (/** @type {Chart} */ doc, /** @type {string} */ letter) => ((name) => (name ? `${name} (${letter}) was removed` : null))(removedNames.get(`${doc.id}:${letter}`));
/** @param {Layer} layer */
const ownerOf = (layer) => ((docs) => docs.find((doc) => doc?.layers.includes(layer)) ?? docs.find((doc) => doc?.layers.some((item) => item.id === layer.id)))([chart, ...previewCharts, ...library.charts, ...library.presets]);
/** @param {{ from?: string[] }} layer @param {Chart | null} [doc] */
const fromLetters = (layer, doc = ownerOf(/** @type {Layer} */ (layer))) => (layer.from ?? []).flatMap((id) => ((item) => (item ? [item.letter] : []))(doc?.layers.find((one) => one.id === id)));
/** @param {Layer} layer @param {Chart | null} [doc] */
const fromSpan = (layer, doc = ownerOf(layer)) => (layer.from ?? []).flatMap((id) => (EDGES.includes(id) ? [`the ${id}`] : ((item) => (item ? [item.letter] : []))(doc?.layers.find((one) => one.id === id)))).join(", ");
/** @param {{ type?: string, from?: string[], id?: string, ref?: string }} layer @param {Chart} [doc] */
const fromShort = (layer, doc = chart) => {
  const ids = layer.from ?? [], members = (doc?.layers ?? []).filter((other) => ids.includes(other.id)), first = members[0];
  return ids.length < 2 ? (layer.type === "cloud" ? "Pick what it's between" : "Select two layers") : members.some((other) => !oneScale(other, first)) ? ONE_SCALE : layer.type !== "trend" || doc?.layers.some((other) => !other.from && !placedOn(other) && !ids.includes(other.id) && (!first || oneScale(other, first))) ? "" : layer.ref && doc?.layers.some((other) => other.id === layer.ref) ? FLIP_APART : TREND_FLIP;
};
/** @param {Layer} item */
const memberable = (item, /** @type {Layer | null} */ self = null, /** @type {Layer | null} */ on = null) => !item.from && !placedOn(item) && !heatLike(item) && item !== self && !(self && readsOf(item).has(self.letter)) && (!on || oneScale(item, on));
const trendShort = () => {
  const bases = chart.layers.filter((layer) => !layer.from && !placedOn(layer));
  return bases.length < 3 ? "Needs three layers: two to follow and a third that flips it" : bases.some((layer) => bases.filter((other) => oneScale(other, layer)).length >= 3) ? "" : "Needs three layers in one pane, on one side";
};
const fromUnfit = (/** @type {Chart} */ doc, /** @type {Layer} */ layer) => {
  const pane = doc.panes.find((item) => item.id === layer.pane);
  return !layer.from || (edgesOf(layer).length > 1 && !cloudLayers(layer).length) ? "" : offScale(doc, layer) ? ONE_SCALE : drawnInto(doc, pane) ? `Not in ${drawnInto(doc, pane)}` : layer.type === "cloud" && paneMode(pane) >= 2 ? "Not on this scale" : "";
};

/** @param {Chart} doc @param {Layer} layer */
const acrossUnfit = (doc, layer) => ((pane) => (drawnInto(doc, pane) ? `Not drawn in ${drawnInto(doc, pane)}` : paneMode(pane) >= 2 ? "Not drawn on a percent or indexed scale" : ""))(doc.panes.find((item) => item.id === layer.pane));
/**
 * Not a stack: one added there would be summed into it.
 * @param {Chart} doc @param {string} unit
 */
const paneShowing = (doc, unit) => doc.panes.find((candidate) => !candidate.stack && doc.layers.some((layer) => layer.pane === candidate.id && unitFor(layer, resolved?.results.get(layer.id)) === unit));
const drafts = new WeakSet();
/** @param {Chart} doc */
const emptyYours = (doc) => isYours(doc) && doc.blank === true && !doc.layers.length && !doc.title && !doc.drawings?.length;
/** @type {WeakMap<Chart, number>} */
const keptByEdit = new WeakMap();
const draftChart = () => ((doc) => (drafts.add(doc), doc))(blankChart());
const touched = new WeakSet();
const asideDirty = new WeakSet();
/** @param {Chart} doc */
const touchPreview = (doc) => void (touched.add(doc), asideDirty.add(doc));
/** @param {Chart} doc */
const keepAsideOpened = (doc) => void (isPreview(doc) && !drafts.has(doc) && (asideDirty.add(doc), save()));
/** @param {Layer} layer */
function followPool(layer) {
  if (layer.series) layer.series = poolSeries(layer.series);
  if (layer.path) layer.path = poolPath(layer.path);
  if (layer.formula === undefined) return;
  const names = new Map([...seriesNamed(layer.formula)].flatMap((name) => (poolSeries(name) === name ? [] : [[name, poolSeries(name)]])));
  layer.formula = renamed(layer.formula, names);
  if (layer.paths) layer.paths = Object.fromEntries(Object.entries(layer.paths).map(([name, path]) => [poolSeries(name), poolPath(path)]));
}

/**
 * By its place in the catalog (its last window step), never by its name, which puts it anywhere (`market_cap_delta_1y_percent`).
 * @param {string} name @returns {{ window: string, at: Map<string, string> } | null}
 */
function seriesWindow(name) {
  const steps = catalogIndex?.get(name)?.split("."), i = steps ? steps.findLastIndex(isWindowKey) : -1;
  if (!steps || i < 0) return null;
  const at = new Map(Object.keys(nodeAt(steps.slice(0, i)) ?? {}).filter(isWindowKey).flatMap((key) => ((other) => (other ? [[key, other]] : []))(nameAt([...steps.slice(0, i), key, ...steps.slice(i + 1)].join(".")))));
  return { window: steps[i], at };
}
/** @param {Chart} doc */
const seriesOf = (doc) => doc.layers.flatMap((layer) => [...(layer.series ? [layer.series] : []), ...(layer.formula ? seriesNamed(layer.formula) : [])]);
/**
 * @typedef {{ window: string, offered: string[], at: Map<string, Map<string, string>> }} ChartWindow a chart's
 * window, the windows it offers, shortest first, and each of its windowed series at each of them
 */
/** @type {Map<string, ChartWindow | null>} */
const windowsFound = new Map();
/** @param {Chart} doc @returns {ChartWindow | null} */
function chartWindow(doc) {
  const names = seriesOf(doc), key = names.join(" ");
  let found = windowsFound.get(key);
  if (found !== undefined) return found;
  found = null;
  if (names.some((name) => name.split("_").some((part) => isWindowKey(`_${part}`)))) {
    indexCatalog();
    let window = "", offered = /** @type {string[] | null} */ (null);
    const at = new Map();
    for (const name of names) {
      const its = seriesWindow(name);
      if (!its) continue;
      if (window && its.window !== window) {
        offered = null;
        break;
      }
      window = its.window;
      at.set(name, its.at);
      offered = (offered ?? [...its.at.keys()]).filter((other) => its.at.has(other));
    }
    if (offered && offered.length > 1) found = { window, offered, at };
  }
  windowsFound.set(key, found);
  return found;
}
/** @param {Chart} doc @param {string} to */
function atWindow(doc, to) {
  const now = chartWindow(doc);
  if (!now || now.window === to || !now.offered.includes(to)) return doc;
  const swap = (/** @type {string} */ name) => (now.at.get(name) ?? ((its) => (its?.window === now.window ? its.at : undefined))(seriesWindow(name)))?.get(to) ?? name;
  // (Only as the presets say a window: the whole of a label, or closing a name or what's before its colon, after a
  // comma or "per" (", 30 days", see appended), and as an adjective ("30-day"); never inside another name, as an age
  // band's "1 day to under 1 week old".)
  const said = (/** @type {string} */ text) => text.replace(new RegExp(`(?<=^|, |per )${windowSaid(now.window)}(?=:|$)`), windowSaid(to)).replace(new RegExp(`(?<![\\w-])${windowAdjective(now.window)}(?![\\w-])`), windowAdjective(to));
  for (const layer of [...doc.layers, ...asideLayers(doc)]) {
    if (layer.series) {
      layer.series = swap(layer.series);
      delete layer.path;
    }
    if (layer.formula) {
      layer.formula = renamed(layer.formula, new Map(seriesNamed(layer.formula).map((name) => [name, swap(name)])));
      delete layer.paths;
    }
    if (layer.label) layer.label = said(layer.label);
  }
  doc.title = said(doc.title);
  normalize(doc);
  return doc;
}

/** @param {Chart} doc */
function normalize(doc) {
  for (const pane of doc.panes) {
    if (pane.lens && !doc.layers.some((layer) => layer.pane === pane.id)) showView(doc, pane.id, pane.lens.at ? 0 : 1);
    if (!pane.lens) continue;
    const { layers, drawings, ...fields } = pane.lens.other, aside = { ...doc, panes: [{ ...fields, id: pane.id }], layers, drawings };
    normalize(aside);
    const { id: _id, ...kept } = aside.panes[0];
    if (aside.layers.length) pane.lens.other = { ...kept, layers: aside.layers, ...(aside.drawings ? { drawings: aside.drawings } : {}) };
    else delete pane.lens;
  }
  for (const layer of doc.layers) {
    if (layer.maxInterval && !INTERVAL_ORDER.includes(layer.maxInterval)) delete layer.maxInterval;
    if (!intervalFits(layer.maxInterval, doc.index)) doc.index = /** @type {string} */ (layer.maxInterval);
  }
  for (const layer of doc.layers) followPool(layer);
  for (const layer of doc.layers) {
    if (!layer.series) continue;
    const name = layer.path ? nameAt(layer.path) : null;
    if (name) layer.series = name;
    else if (catalogIndex?.has(layer.series)) layer.path = catalogIndex.get(layer.series);
  }
  for (const layer of doc.layers) {
    if (layer.formula === undefined || !tree) continue;
    /** @type {Record<string, string>} */
    const paths = {}, names = new Map();
    for (const name of seriesNamed(layer.formula)) {
      const path = layer.paths?.[name], current = path ? nameAt(path) : null;
      if (current && path) {
        if (current !== name) names.set(name, current);
        paths[current] = path;
      } else if (catalogIndex?.has(name)) paths[name] = /** @type {string} */ (catalogIndex.get(name));
    }
    // (All at once: one renamed to another's old name isn't renamed again.)
    layer.formula = renamed(layer.formula, names);
    if (Object.keys(paths).length) layer.paths = paths;
    else delete layer.paths;
  }
  for (const layer of doc.layers) if (layer.from) layer.from = fromOf(doc, layer.from, layer.type);
  for (const layer of doc.layers) if (layer.type === "trend") delete layer.shape;
  for (const layer of doc.layers) {
    if (layer.type !== "profile" || !tree) continue;
    layer.profile = [...new Set(layer.profile ?? [])];
    for (const [cohort, color] of Object.entries(layer.cohortColors ?? {})) if (color === cohortColor(cohort) || !layer.profile.includes(cohort)) delete /** @type {Record<string, string>} */ (layer.cohortColors)[cohort];
    if (layer.cohortColors && !Object.keys(layer.cohortColors).length) delete layer.cohortColors;
    if (layer.profile.length) layer.color = profileColor(layer, layer.profile[0]);
  }
  for (const layer of doc.layers) if (layer.colorFrom && (!COLORED_FROM.has(layer.type) || layer.colorFrom === layer.id || !doc.layers.some((other) => other.id === layer.colorFrom) || doc.layers.find((other) => other.id === layer.colorFrom)?.colorFrom === layer.id)) delete layer.colorFrom;
  if (!doc.panes.length) doc.panes.push({ id: uid(), mode: 1 });
  const ids = new Set(doc.panes.map((pane) => pane.id));
  for (const layer of doc.layers) if (!ids.has(layer.pane)) layer.pane = doc.panes[0].id;
  for (const layer of doc.layers) if (layer.type === "heatmap" && signedMeasure(layer)) (layer.color = /** @type {string} */ (kindColor(layer))), delete layer.colorBelow;
  for (const layer of doc.layers) {
    const pane = layer.type === "heatmap" && layer.byGradient && !signedMeasure(layer) && doc.panes.find((item) => item.id === layer.pane), ready = pane && !pane.gradient && HEAT_GRADIENTS.find(([, , stops]) => JSON.stringify(stops) === JSON.stringify(layer.gradient));
    if (pane && ready && ready !== HEAT_GRADIENTS[0]) pane.gradient = ready[0];
  }
  for (const layer of doc.layers) if (layer.gradient && !(COLOR_MODES[layer.type] ?? []).includes("gradient")) delete layer.gradient;
  for (const layer of doc.layers) if (!layer.gradient) delete layer.byGradient, delete layer.upToDate;
  for (const layer of doc.layers) if (layer.byGradient) delete layer.colorFrom, delete layer.byLevels, delete layer.byLayers, delete layer.colors;
  for (const layer of doc.layers) if (layer.byLevels && (!BY_LEVELS.has(layer.type) || !layer.levels?.length || layer.colorFrom)) delete layer.byLevels;
  for (const layer of doc.layers) if (layer.byLayers && (layer.type !== "cloud" || layer.colorFrom || !cloudLayers(layer).length)) delete layer.byLayers;
  for (const layer of doc.layers) if (layer.byLayers) layersColors(layer, doc);
  for (const layer of doc.layers) if (layer.type === "cloud" && edgesOf(layer).length && !layer.byLayers && !layer.colorFrom && layer.colorBelow !== undefined && layer.colors === undefined) layer.colors = 1;
  for (const layer of doc.layers) if (layer.type === "candles" && layer.colors === undefined && layer.colorBelow === undefined && !layer.colorFrom && !layer.byLevels && !layer.byGradient) layer.colors = 1;
  for (const layer of doc.layers) if (layer.colors !== undefined && (!(layer.type === "histogram" ? layer.colors === 1 || layer.colors === 4 : TWO_OR_ONE.has(layer.type) ? layer.colors === 1 : TWO_KEPT.has(layer.type) && layer.colors === 1 && layer.colorBelow !== undefined) || layer.colorFrom || layer.byLevels || layer.byLayers)) delete layer.colors;
  for (const layer of doc.layers) if (layer.type === "bubbles" && layer.ref === undefined) layer.ref = (markable(doc).find((other) => other.pane === layer.pane) ?? markable(doc)[0])?.id ?? "";
  for (const layer of doc.layers) if (layer.ref && !(layer.type === "trend" ? refsFor(doc, layer) : placedOn(layer) ? markable(doc) : []).some((other) => other.id === layer.ref)) placedOn(layer) ? (layer.ref = "") : delete layer.ref;
  for (const layer of doc.layers) {
    if (layer.type !== "bubbles") for (const key of ["size", "hide", "scaleTo"]) delete layer[key];
    else for (const key of ["shape", "levels", "colorFrom", "byLevels", "byGradient", "priceLine", "base", "fill", "fillOpacity", "solid", "hollow", "baseLine", "underName"]) delete layer[key];
  }
  for (const layer of doc.layers) {
    if (layer.type !== "signal") {
      delete layer.results;
      continue;
    }
    for (const key of ["shape", "levels", "colorFrom", "byLevels", "colors", "priceLine", "outline", "base", "format", "fill", "fillOpacity", "solid", "hollow", "baseLine", "underName"]) delete layer[key];
    if (layer.place === "above" || !MARK_PLACES.some(([place]) => place === layer.place)) delete layer.place;
  }
  for (const layer of doc.layers) if ((layer.fit !== false && layer.fit !== "zero") || !fittable(layer)) delete layer.fit;
  for (const layer of doc.layers.filter((item) => item.from)) {
    const members = doc.layers.filter((other) => layer.from?.includes(other.id)), first = members[0];
    if (!first || members.some((other) => !oneScale(other, first)) || oneScale(layer, first)) continue;
    const moved = layer.pane !== first.pane;
    Object.assign(layer, { pane: first.pane, side: first.side });
    if (!moved) continue;
    doc.layers.splice(doc.layers.indexOf(layer), 1);
    const behind = (layer.type === "cloud") !== Boolean(doc.panes.find((pane) => pane.id === first.pane)?.reverse);
    doc.layers.splice(behind ? doc.layers.indexOf(/** @type {Layer} */ (members.at(-1))) + 1 : doc.layers.indexOf(first), 0, layer);
  }
  for (const layer of doc.layers) if (placedOn(layer)) ((on) => on && Object.assign(layer, { pane: on.pane, side: on.side }))(doc.layers.find((other) => other.id === layer.ref));
  for (const pane of doc.panes) if (pane.reverse && doc.layers.filter((layer) => layer.pane === pane.id).length < 2) delete pane.reverse;
  for (const pane of doc.panes) if (pane.average && !Object.hasOwn(AVERAGES, pane.average)) delete pane.average;
  const used = new Set(doc.layers.map((layer) => layer.pane));
  doc.panes = doc.layers.length ? doc.panes.filter((pane) => used.has(pane.id)) : doc.panes.slice(0, 1);
  const seen = new Set();
  for (const layer of doc.layers) {
    if (seen.has(layer.letter)) layer.letter = nextLetter({ ...doc, layers: doc.layers.filter((other) => other !== layer) });
    seen.add(layer.letter);
  }
  for (const layer of asideLayers(doc)) {
    if (seen.has(layer.letter)) layer.letter = nextLetter(doc);
    seen.add(layer.letter);
  }
  const order = new Map(doc.panes.map((pane, i) => [pane.id, i]));
  doc.layers.sort((a, b) => (order.get(a.pane) ?? 0) - (order.get(b.pane) ?? 0));
  if (doc.drawings) {
    const layers = new Set(doc.layers.map((layer) => layer.id));
    doc.drawings = doc.drawings.filter((drawing) => layers.has(drawing.layer));
    if (!doc.drawings.length) delete doc.drawings;
  }
}

const STORE = "bitview-studio:v2";
const RECENT = "bitview-studio:recent";
const PREFS = "bitview-studio:prefs";
const PREVIEW_ASIDE = "bitview-studio:preview";
const storage = {
  get(key, fallback) {
    try {
      return JSON.parse(localStorage.getItem(key) ?? "") ?? fallback;
    } catch {
      return fallback;
    }
  },
  set(key, value) {
    try {
      localStorage.setItem(key, JSON.stringify(value));
      return true;
    } catch {
      return false;
    }
  },
  drop(...keys) {
    try {
      for (const key of keys) localStorage.removeItem(key);
    } catch {}
  },
};
const startText = (() => {
  try {
    return localStorage.getItem(STORE);
  } catch {
    return null;
  }
})();
let storedAs = startText ?? "";
/** @param {string} text */
function writeStored(text) {
  try {
    localStorage.setItem(STORE, text);
  } catch {
    return false;
  }
  storedAs = text;
  return true;
}
/** @type {{ charts: Chart[], active: string, folders: Folder[], order: string[], presets: Chart[], deleted: Record<string, number> }} */
let library = (() => {
  try {
    return JSON.parse(startText ?? "");
  } catch {
    return null;
  }
})();
/** The library's shape. Raised with what `validate` learns to read and with what's kept, so an older Studio that doesn't know a field can't save over it (5 deletions, 6 folders in folders, 7 edited presets, 8 interval Auto, 9 signals, 10 a signal's down color and results, 11 area fill opacity and solid fill, candles' one color and hollow bodies, 12 paths and a folder's icon, 13 heatmaps and gradients, 14 profiles, 15 a layer's `maxInterval` and `fit: "zero"`, 16 a pane drawn as a heatmap, its outline and short names, 17 a heatmap's gradient on its pane). One saved by a newer Studio, or stored in a shape this can't read, is opened read-only, said so, and kept as it is: saving over it could drop what this one doesn't know. */
const SCHEMA = 17;
let readOnly = Number(/** @type {any} */ (library)?.schema) > SCHEMA || (!Array.isArray(library?.charts) && startText !== null);
let unsavedWhy = !readOnly ? "" : Number(/** @type {any} */ (library)?.schema) > SCHEMA ? "newer" : "unreadable";
if (!Array.isArray(library?.charts)) library = { charts: [], active: "", folders: [], order: [], presets: [], deleted: {} };
const orderOf = (/** @type {any} */ stored) => [...new Set((Array.isArray(stored) ? stored : []).filter((key) => typeof key === "string" && /^[fc]:[a-z0-9]{1,16}$/.test(key)))];
const deletedOf = (/** @type {any} */ stored) => {
  const since = Date.now() - 30 * 86400000;
  return Object.assign(/** @type {Record<string, number>} */ (Object.create(null)), Object.fromEntries(Object.entries(stored && typeof stored === "object" ? stored : {}).filter(([id, at]) => /^[a-z0-9]{1,16}$/.test(id) && Number.isFinite(at) && at > since)));
};
library.deleted = deletedOf(library.deleted);
/** Saved charts this version couldn't read, kept aside as they were: never lost to the next save. */
const UNREAD = "bitview-studio:unread";
/** @returns {Chart} */
function read(stored) {
  const doc = validate(stored, true);
  if (typeof stored.id === "string" && /^[a-z0-9]{1,16}$/.test(stored.id)) doc.id = stored.id;
  if (typeof stored.folder === "string" && /^[a-z0-9]{1,16}$/.test(stored.folder)) doc.folder = stored.folder;
  if (typeof stored.preset === "string" && stored.preset.length <= 2000) doc.preset = stored.preset;
  if (typeof stored.base === "string" && /^[a-z0-9]{1,24}$/.test(stored.base)) doc.base = stored.base;
  if (stored.blank === true) doc.blank = true;
  doc.updated = Number.isFinite(stored.updated) ? stored.updated : 0;
  keptAsStored(doc);
  normalize(doc);
  return doc;
}
/** @returns {Chart[]} */
function restore(stored) {
  try {
    return [read(stored)];
  } catch (error) {
    // Never silently: the chart goes aside, untouched, and it's said (see start). One per chart, as it was last: a
    // chart edited again elsewhere replaces its older copy.
    const aside = storage.get(UNREAD, []);
    const list = (Array.isArray(aside) ? aside : []).filter((item) => (stored?.id ? item?.id !== stored.id : JSON.stringify(item) !== JSON.stringify(stored)));
    if (!storage.set(UNREAD, [...list, stored])) (readOnly = true), (unsavedWhy = "full");
    console.error(`A saved chart couldn't be read; it's kept under "${UNREAD}" in this browser's storage.`, error);
    return [];
  }
}
library.charts = library.charts.flatMap(restore).filter((doc) => !(library.deleted[doc.id] >= doc.updated));
/** @returns {Chart[]} */
function presetsOf(/** @type {any} */ stored) {
  const byKey = /** @type {Map<string, Chart>} */ (new Map());
  for (const doc of (Array.isArray(stored) ? stored : []).flatMap(restore)) {
    if (typeof doc.preset !== "string" || library.deleted[doc.id] >= doc.updated) continue;
    delete doc.folder;
    if (!((byKey.get(doc.preset)?.updated ?? -Infinity) > doc.updated)) byKey.set(doc.preset, doc);
  }
  return [...byKey.values()];
}
library.presets = presetsOf(library.presets);
const FOLDER_ICONS = ["candles", "scale", "coins", "people", "hourglass", "percent", "cube", "key", "pickaxe", "nodes", "clock", "swap", "layers", "star", "flag", "bolt", "target", "calendar"];
const letterOf = (/** @type {string} */ name) => ([...name.trim()][0] ?? "?").toUpperCase();
const folderMark = (/** @type {Folder} */ folder) => (folder.icon && ICONS[folder.icon] ? `<span class="chart-mark section-mark" aria-hidden="true">${ICONS[folder.icon]}</span>` : `<span class="chart-mark folder-mark" aria-hidden="true">${escapeHtml(letterOf(folder.name))}</span>`);
const sectionMark = (/** @type {PresetNode | null | undefined} */ node, open = false) => (node?.icon ? `<span class="chart-mark section-mark">${ICONS[node.icon]}</span>` : `<span class="chart-mark folder-glyph">${open ? ICONS.folderOpen : ICONS.folder}</span>`);
function foldersOf(/** @type {any} */ stored, /** @type {Chart[]} */ charts) {
  const seen = new Set(), folders = /** @type {Folder[]} */ ((Array.isArray(stored) ? stored : []).flatMap((folder) => {
    if (typeof folder?.id !== "string" || !/^[a-z0-9]{1,16}$/.test(folder.id) || seen.has(folder.id)) return [];
    seen.add(folder.id);
    return [{ id: folder.id, name: String(folder.name ?? "").trim().slice(0, 60) || "Folder", ...(typeof folder.parent === "string" && folder.parent ? { parent: folder.parent } : {}), ...(folder.closed === true ? { closed: true } : {}), ...(FOLDER_ICONS.includes(folder.icon) ? { icon: folder.icon } : {}) }];
  }));
  const byId = new Map(folders.map((folder) => [folder.id, folder]));
  // One in a folder not here goes to the top; a loop is cut where it closes.
  for (const folder of folders) {
    const path = new Set([folder.id]);
    for (let at = folder; at.parent; ) {
      const up = byId.get(at.parent);
      if (!up || path.has(up.id)) {
        delete at.parent;
        break;
      }
      path.add(up.id);
      at = up;
    }
  }
  for (const doc of charts) if (doc.folder && !seen.has(doc.folder)) delete doc.folder;
  return folders;
}
// Charts set aside by a version that couldn't read them come back, first in the list, once this one can. One here already (the same chart, or a copy alike in all but its id) isn't brought back twice.
{
  const aside = storage.get(UNREAD, []);
  if (Array.isArray(aside) && aside.length) {
    const alike = (doc) => JSON.stringify({ ...doc, id: "", updated: 0 });
    const ids = new Set(library.charts.map((doc) => doc.id)), here = new Set(library.charts.map(alike)), back = [], still = [], newer = new Map();
    for (const stored of aside) {
      try {
        const doc = read(stored);
        if (here.has(alike(doc)) || library.deleted[doc.id] >= doc.updated) continue;
        if (ids.has(doc.id)) {
          const have = library.charts.find((item) => item.id === doc.id);
          if (have && doc.updated > (newer.get(doc.id) ?? have).updated) newer.set(doc.id, doc);
          continue;
        }
        ids.add(doc.id);
        here.add(alike(doc));
        back.push(doc);
      } catch {
        still.push(stored);
      }
    }
    const charts = back.filter((doc) => doc.preset === undefined), edited = presetsOf([...library.presets, ...back.filter((doc) => doc.preset !== undefined)]);
    const replaced = library.charts.map((doc) => {
      const later = newer.get(doc.id);
      if (!later) return doc;
      if (doc.folder === undefined) delete later.folder;
      else later.folder = doc.folder;
      return later;
    });
    if (!readOnly && still.length !== aside.length && writeStored(JSON.stringify({ ...library, schema: SCHEMA, charts: [...charts, ...replaced], presets: edited }))) {
      library.charts = [...charts, ...replaced];
      library.presets = edited;
      storage.set(UNREAD, still);
    }
  }
}
library.folders = foldersOf(library.folders, library.charts);
library.order = orderOf(library.order);
/** @type {Chart[]} */
let previewCharts = [];
const isPreview = (/** @type {Chart | null | undefined} */ doc) => Boolean(doc && previewCharts.includes(doc));
/** @param {Chart} doc */
const isYours = (doc) => !isPreview(doc) && doc.preset === undefined;
const MOST_PREVIEWS = 20;
/** @param {Chart} doc */
function addPreview(doc) {
  if (isPreview(doc)) return;
  previewCharts.unshift(doc);
  const plain = previewCharts.filter((item) => !touched.has(item) && !drafts.has(item) && item !== chart && item !== doc);
  for (const old of plain.slice(MOST_PREVIEWS - 1)) dropPreview(old), forgetPreview(old), previews.delete(old.id);
}
/** @param {Chart} doc */
const dropPreview = (doc) => void (previewCharts = previewCharts.filter((item) => item !== doc));
const newDraft = () => ((draft) => (addPreview(draft), draft))(draftChart());
/** @type {{ key: string } | null} */
let quick = null;
const listedPreviews = () => previewCharts.filter((doc) => !drafts.has(doc));
/** @param {Chart} doc */
const previewLike = (doc) => (doc.layers.length ? previewCharts.find((item) => item !== doc && sameChart(item, doc)) : undefined);
/** @param {Chart} like @param {Chart} doc */
function lookAs(like, doc) {
  like.index = doc.index;
  like.range = doc.range;
  takeView(doc.id, like.id);
  forgetSpans(like.id);
}
/** @param {string} from @param {string} to */
const moveView = (from, to) => {
  const all = viewsNow();
  if (!all[from] || from === to) return;
  all[to] = all[from];
  delete all[from];
  putViews(all);
};
/** @param {string} id */
const forgetView = (id) => {
  const all = viewsNow();
  if (all[id]) (delete all[id], putViews(all));
};
/** @param {string} from @param {string} to */
const takeView = (from, to) => (viewsNow()[from] ? moveView(from, to) : forgetView(to));
/** @param {Chart} doc @param {string} id */
const rekey = (doc, id) => {
  const old = doc.id;
  doc.id = id;
  moveView(old, id);
  for (const map of /** @type {Map<string, any>[]} */ ([histories, previews, thumbs, inspector.formulaDrafts])) if (map.has(old)) map.set(id, map.get(old)), map.delete(old);
  if (chart === doc) library.active = id;
  if (displayed?.doc.id === old) (displayed.doc.id = id), (displayed.look = lookKey(displayed.doc));
  if (history.state?.chart === old) history.replaceState({ ...history.state, chart: id }, "");
  if (touched.has(doc) || (isPreview(doc) && !drafts.has(doc))) asideDirty.add(doc);
  if (isPreview(doc) && !drafts.has(doc)) keptPreviews.set(old, doc.updated);
};
let brokenLink = "";
/**
 * A chart deleted: out of the list, and when, kept with the library, so that another tab's copy of it (saved before
 * that tab heard) doesn't bring it back. Edited after that (an Undo, an edit in a tab where it's open), it's back.
 */
const discard = (/** @type {Chart} */ doc) => {
  const at = library.charts.indexOf(doc);
  if (at < 0) return;
  library.charts.splice(at, 1);
  library.deleted[doc.id] = Math.max(Date.now(), doc.updated, library.deleted[doc.id] ?? 0);
};
/** @type {{ doc: Chart, at: number, folder: string | undefined } | null} */
let cancelledNew = null;
const dropEmpty = (/** @type {Chart} */ doc) => {
  const at = library.charts.indexOf(doc);
  if (at < 0) return;
  library.charts.splice(at, 1);
  const left = cancelledNew?.doc === doc ? cancelledNew : null;
  cancelledNew = null;
  if (left) queueMicrotask(() => toast("Removed the new chart", { label: "Undo", run: () => {
    if (inLibrary(doc)) return;
    relist(doc, left.at);
    if (left.folder && folderById(left.folder)) doc.folder = left.folder;
    save();
    if (home) leaveHome(() => openChart(doc));
    else openChart(doc);
  } }));
  library.deleted[doc.id] = Math.max(doc.updated, Date.now() - 29 * 86400000, library.deleted[doc.id] ?? 0);
};
/** @param {Chart} doc */
const recover = (doc) => {
  stampEdit(doc);
  if (doc.folder) doc.folder = folderNow(doc.folder);
  if (doc.folder && !library.folders.some((folder) => folder.id === doc.folder)) delete doc.folder;
  delete library.deleted[doc.id];
};
/** @param {Chart} doc @param {number} at */
const relist = (doc, at) => {
  recover(doc);
  library.charts.splice(Math.min(at, library.charts.length), 0, doc);
};
/** @param {Chart} doc */
const stampEdit = (doc) => (doc.updated = Math.max(Date.now(), doc.updated + 1, (library.deleted[doc.id] ?? 0) + 1));
/** @param {{ id: string, updated: number }} doc */
const isDeleted = (doc) => library.deleted[doc.id] >= doc.updated;
/** @param {{ id: string, updated: number }} doc */
const alive = (doc) => doc.id === chart?.id || !isDeleted(doc);
/** @returns {Chart | undefined} */
const chartById = (id) => library.charts.find((doc) => doc.id === id) ?? library.presets.find((doc) => doc.id === id) ?? previewCharts.find((doc) => doc.id === id);
/** @param {Chart} doc */
function inLibrary(doc) {
  return library.charts.some((other) => other.id === doc.id);
}
/** The catalog indexed in a worker, off the page's thread (long on a phone without its JIT, in a launch's first seconds), begun with the rest as what asks for it is: its own copy of the client, fetched beside a first chart's data, held that data up. Without a worker, here, a slice at a time while the page is idle. What needs it before (a search, a series added) has the rest done here at once, never waiting on the worker or an idle moment: a hidden tab's are throttled (to one a minute, after five minutes hidden). */
const indexCatalog = (() => {
  /** @type {ReturnType<typeof catalogWalk> | undefined} */
  let walk;
  const take = (/** @type {{ index: Map<string, string>, intervals: Map<string, readonly string[]>, sharedTops: Set<string> }} */ found) => {
    if (catalogIndex) return;
    catalogIndex = found.index;
    catalogIntervals = found.intervals;
    for (const [name, ids] of found.intervals) if (!ids.length) byBlock.add(name);
    for (const name of found.sharedTops) sharedTops.add(name);
    seriesCount = catalogIndex.size - byBlock.size;
    $("cta-label").textContent = `Search ${grouped.format(seriesCount)} series`;
    withRest(() => whenIdle(command.ranksSoon));
    queueMicrotask(() => {
      const docs = [...library.charts, ...library.presets], before = docs.map((doc) => JSON.stringify(doc));
      for (const doc of docs) normalize(doc);
      if (docs.some((doc, i) => JSON.stringify(doc) !== before[i])) save();
      if (docs.includes(chart) && JSON.stringify(chart) !== before[docs.indexOf(chart)]) draw();
    });
  };
  const run = (/** @type {(done: number) => boolean} */ more) => {
    if (!tree) return;
    walk ??= catalogWalk(tree, INTERVAL_ORDER);
    for (let done = 0; !catalogIndex && more(done); done++) {
      const step = walk.next();
      if (step.done) return take(step.value);
    }
  };
  const here = () => {
    const step = (/** @type {{ timeRemaining: () => number }} */ deadline) => {
      // (At least a few hundred nodes a turn: an idle callback timed out has no time left, and would never go on.)
      run((done) => deadline.timeRemaining() > 1 || done < 400);
      if (!catalogIndex) whenIdle(step);
    };
    whenIdle(step, { timeout: 3000 });
  };
  withRest(() => { try {
    const worker = startWorker(new URL("./catalog-worker.js", import.meta.url)), done = () => worker.terminate();
    worker.onmessage = ({ data }) => {
      done();
      let found = null;
      try {
        const [index, lists, at, shared] = JSON.parse(data.text);
        found = { index: new Map(index), intervals: new Map(index.map((/** @type {[string, string]} */ [name], /** @type {number} */ i) => [name, lists[at[i]]])), sharedTops: new Set(shared) };
      } catch (error) {
        console.error("catalog-worker.js: its answer could not be read", data.error ?? error);
      }
      found ? take(found) : here();
    };
    worker.onerror = worker.onmessageerror = () => (done(), here());
    worker.postMessage(INTERVAL_ORDER);
  } catch {
    here();
  } });
  return () => run(() => true);
})();
/** @type {Chart} */
let chart = chartById(library.active) ?? library.charts[0];
let selected = "";
const layerPicks = new Set();
let layerPickLast = "", pickHead = "";
let lastOpen = "";
/** @typedef {{ state: string, at: number }} ChartStep */
const edits = { past: /** @type {ChartStep[]} */ ([]), future: /** @type {ChartStep[]} */ ([]), key: "", at: 0, held: 0 };
let stepCount = 0;
let holding = 0, holds = 0;
const histories = /** @type {Map<string, { past: ChartStep[], future: ChartStep[] }>} */ (new Map());
/**
 * Undo budgets count JSON characters (two bytes each); kept charts get less, or a large chart's history could
 * hold hundreds of MB.
 */
const UNDO_CHARS = 8e6, KEPT_CHARS = 2e6;
const UNDO_STEPS = 200, KEPT_STEPS = 50, KEPT_HISTORIES = 8;
function trimSteps(/** @type {{ past: ChartStep[], future: ChartStep[] }} */ steps, /** @type {number} */ budget) {
  let size = 0;
  for (const step of steps.past) size += step.state.length;
  for (const step of steps.future) size += step.state.length;
  while (size > budget && steps.past.length > 1) size -= /** @type {ChartStep} */ (steps.past.shift()).state.length;
  while (size > budget && steps.future.length > 1) size -= /** @type {ChartStep} */ (steps.future.shift()).state.length;
}
let revision = 0;
let saves = 0, toasts = 0;
let chartChanged = false;
/** @typedef {"recent" | "previews" | "library" | "presets"} Group */
const GROUPS = /** @type {Group[]} */ (["recent", "previews", "library", "presets"]);
const GROUP_OF = /** @type {Record<Group, [string, Place]>} */ ({ recent: ["Recent", { recent: true }], previews: ["Previews", { previews: true }], library: ["Library", { folder: "" }], presets: ["Presets", { path: [] }] });
function readPrefs(/** @type {any} */ stored) {
  return {
    sidebar: stored.sidebar === "collapsed" || stored.sidebar === "expanded" ? stored.sidebar : tablet.matches ? "collapsed" : "expanded",
    panel: stored.panel === "open" || stored.panel === "closed" ? stored.panel : tablet.matches ? "closed" : "open",
    home: stored.home !== false,
    sideWidth: /** @type {number | null} */ (Number.isFinite(stored.sideWidth) ? stored.sideWidth : null),
    panelWidth: /** @type {number | null} */ (Number.isFinite(stored.panelWidth) ? stored.panelWidth : null),
    at: /** @type {any} */ (stored.at && typeof stored.at === "object" ? stored.at : null),
    presetsOpen: /** @type {string[]} */ (Array.isArray(stored.presetsOpen) ? stored.presetsOpen.filter((/** @type {any} */ key) => typeof key === "string").slice(-500) : []),
    groupsOpen: /** @type {Group[]} */ (Array.isArray(stored.groupsOpen) ? GROUPS.filter((group) => stored.groupsOpen.includes(group)) : []),
    keys: stored.keys !== false,
    linkView: stored.linkView === true,
    theme: /** @type {"system" | "light" | "dark"} */ (["system", "light", "dark"].includes(stored.theme) ? stored.theme : "system"),
    belt: { on: stored.belt?.on === true, dock: ["top", "bottom", "left", "right"].includes(stored.belt?.dock) ? stored.belt.dock : "bottom", at: Number.isFinite(stored.belt?.at) ? clamp(stored.belt.at, 0, 1) : 0.5 },
    pen: /** @type {{ color: string, width: number, dash: number, opacity?: number, start?: End, end?: End, size: number, back?: string }} */ ({ ...lookOfDrawing("trend", stored.pen), size: Number.isFinite(stored.pen?.size) ? presetSize(stored.pen.size) : TEXT_SIZE }),
  };
}
const prefs = readPrefs(storage.get(PREFS, {}));
if (phone.matches) prefs.belt.on = false;
const prefsSeen = Object.fromEntries(Object.entries(prefs).map(([key, value]) => [key, JSON.stringify(value)]));
function savePrefs() {
  const changed = Object.fromEntries(Object.entries(prefs).filter(([key, value]) => JSON.stringify(value) !== prefsSeen[key]));
  const written = storage.set(PREFS, { ...storage.get(PREFS, {}), ...changed });
  if (written) for (const [key, value] of Object.entries(changed)) prefsSeen[key] = JSON.stringify(value);
  return written;
}
/** Set before the first paint (see the page's first script), and here. */
const THEMES = /** @type {const} */ ([
  ["system", "System", "computer", "As the device is set, and as it changes."],
  ["light", "Light", "sun", "Light surfaces; the palette's colors deepened to read on white, any other as given."],
  ["dark", "Dark", "moon", "Dark surfaces; the data's colors as they are."],
]);
const deviceLight = matchMedia(MEDIA.light);
const themeColors = /** @type {NodeListOf<HTMLMetaElement>} */ (document.querySelectorAll('meta[name="theme-color"]'));
const syncThemeColor = () => {
  const color = css(phone.matches && !document.body.hasAttribute("data-home") ? "--card" : "--app");
  for (const meta of themeColors) meta.content = color;
};
new MutationObserver(syncThemeColor).observe(document.body, { attributes: true, attributeFilter: ["data-home"] });
new MutationObserver(syncThemeColor).observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme"] });
deviceLight.addEventListener("change", syncThemeColor);
phone.addEventListener("change", syncThemeColor);
syncThemeColor();
const lightChosen = () => prefs.theme === "light" || (prefs.theme === "system" && deviceLight.matches);
light = lightChosen();
document.documentElement.dataset.theme = prefs.theme;
let home = false;

let saveDue = false, savedFade = /** @type {Animation | null} */ (null), unsaved = false;
let announcing = false;
/** The library as stored now, if another tab saved since this one: read and taken in by the caller, so the next save doesn't bring back what this tab let go of since; else null. */
function storedSince() {
  let text = null;
  try {
    text = localStorage.getItem(STORE);
  } catch {}
  if (text === null || text === storedAs) return null;
  try {
    const stored = JSON.parse(text);
    if (!(Number(stored?.schema) > SCHEMA) && Array.isArray(stored?.charts)) {
      storedAs = text;
      return stored;
    }
    unsavedWhy ||= Number(stored?.schema) > SCHEMA ? "newer" : "unreadable";
  } catch {
    unsavedWhy ||= "unreadable";
  }
  readOnly = true;
  return null;
}
const arrangedNow = () => JSON.stringify([library.order, library.folders, library.charts.map((doc) => [doc.id, doc.folder ?? ""])]);
let arrangedAs = "";
function persist() {
  // Another tab's save since this one's, not heard yet (its storage event to come, or never: two tabs closed at once,
  // one back from the browser's history), taken in first: what's written holds both.
  const stored = storedSince();
  if (stored) reconcile(stored, arrangedNow() !== arrangedAs, true);
  if (readOnly) return false;
  const pruned = deletedOf(library.deleted);
  for (const doc of [...library.charts, ...library.presets]) if (isDeleted(doc)) pruned[doc.id] = library.deleted[doc.id];
  library.deleted = pruned;
  const written = JSON.stringify({ ...library, charts: library.charts.filter((doc) => !isDeleted(doc)), presets: library.presets.filter((doc) => !isDeleted(doc)), schema: SCHEMA });
  const mine = previewCharts.filter((doc) => asideDirty.has(doc) && !drafts.has(doc));
  let aside = true;
  if (mine.length) {
    const merged = new Map(previewsAside().map((doc) => [doc.id, doc]));
    for (const doc of mine) if (!((merged.get(doc.id)?.updated ?? -Infinity) > doc.updated)) merged.set(doc.id, touched.has(doc) ? doc : /** @type {any} */ ({ ...doc, untouched: true }));
    aside = storage.set(PREVIEW_ASIDE, [...merged.values()]) || (storage.drop(PREVIEWS, THUMBS, PRESET_MARKS, SIDE_PRESETS), storage.set(PREVIEW_ASIDE, [...merged.values()]));
    if (aside) for (const doc of mine) asideDirty.delete(doc);
  }
  const write = () => {
    if (!writeStored(written)) return false;
    arrangedAs = arrangedNow();
    return true;
  };
  const ok = write() || (storage.drop(PREVIEWS, THUMBS, PRESET_MARKS, SIDE_PRESETS), write());
  if (ok && aside && keptPreviews.size) {
    const left = previewsAside().filter((item) => !keptPreviews.has(item.id) || laterAside(item, /** @type {number} */ (keptPreviews.get(item.id))));
    if (left.length) storage.set(PREVIEW_ASIDE, left);
    else storage.drop(PREVIEW_ASIDE);
    keptPreviews.clear();
  }
  return ok && aside;
}
let unsavedToldAt = -Infinity;
function tellUnsaved() {
  if (Date.now() - unsavedToldAt < 60_000) return;
  unsavedToldAt = Date.now();
  const why = readOnly ? unsavedWhy : "full";
  if (why === "unreadable") return notice("Your saved charts can't be read in this browser. They're left untouched, and changes here won't be saved", { label: "Download", run: () => (downloadStored(), menus.downloadAll(true)) });
  notice(why === "newer" ? "A newer Studio saved your charts: changes here can't be saved. Download them, then reload" : "Not saved: storage is full or blocked. Download your charts to keep them", { label: "Download", run: () => menus.downloadAll(true) });
}
function downloadStored() {
  let stored = "";
  try {
    stored = localStorage.getItem(STORE) ?? "";
  } catch {}
  if (!stored) return notice("Nothing saved to download");
  menus.download(`bitview-charts-as-stored-${new Date().toLocaleDateString("en-CA")}.json`, new Blob([stored], { type: "application/json" }));
}
function flushEdits(open = false) {
  if (!open) (endTyping(), ((ends) => (leftOpen.clear(), ends.forEach((end) => end())))([...leftOpen.values()]));
  withRest(() => {
    if (!open) grid.naming?.finish();
    inspector.applyFormulaLeft();
    const formula = /** @type {HTMLTextAreaElement | null} */ (document.getElementById("formula"));
    if (formula && document.activeElement === formula && selected) inspector.applyFormula(selected, formula.value);
  });
}
/** Written at the page's next idle moment (a second at most), once for all asked for till then: never put off by more. A failure stays on screen. */
function save(announce = false) {
  saves++;
  announcing ||= announce;
  if (saveDue) return;
  saveDue = true;
  whenIdle(() => {
    if (!saveDue) return;
    const announced = announcing;
    saveDue = announcing = false;
    written(persist(), announced);
  });
}
function written(/** @type {boolean} */ ok, /** @type {boolean} */ announced) {
  const saved = $("saved");
  unsaved = !ok && !readOnly;
  if (ok && !announced && !saved.classList.contains("error")) return;
  saved.textContent = ok ? "Saved" : readOnly ? "Read-only: not saved" : "Not saved";
  saved.classList.toggle("error", !ok);
  if (ok) saved.removeAttribute("role");
  else saved.setAttribute("role", "alert");
  if (!ok) tellUnsaved();
  savedFade?.cancel();
  if (!ok) return;
  (savedFade = saved.animate([{ opacity: 1, offset: 0.55 }, { opacity: 0 }], { duration: 2600, easing: "ease", fill: "forwards" })).finished.then(() => (saved.textContent = ""), () => {});
}

/** @param {any[]} charts @param {Folder[]} folders @param {Record<string, number>} deleted */
const holdings = (charts, folders, deleted, presets = /** @type {any[]} */ ([])) => JSON.stringify([[...charts, ...presets].filter((doc) => !(deleted[doc?.id] >= doc?.updated)).map((doc) => `${doc?.id}@${doc?.updated}`).sort(), folders.map((folder) => folder.id).sort()]);
function reconcile(stored = storedSince(), saving = false, writing = false) {
  if (!stored) return;
  const deleted = deletedOf(stored.deleted), incoming = stored.charts.flatMap(restore), folders = foldersOf(stored.folders, []);
  for (const doc of [...previewCharts]) {
    const saved = incoming.find((item) => item.id === doc.id);
    if (saved && sameChart(saved, doc)) (dropPreview(doc), forgetPreview(doc));
    else if (saved) rekey(doc, uid());
  }
  for (const [id, at] of Object.entries(deleted)) if (!(library.deleted[id] >= at)) library.deleted[id] = at;
  const ours = new Map(library.charts.map((doc) => [doc.id, doc])), theirs = new Map(incoming.map((doc) => [doc.id, doc]));
  const arranged = saving && arrangedAs ? /** @type {[string[], Folder[], [string, string][]]} */ (JSON.parse(arrangedAs)) : null, placedThen = new Map(arranged?.[2] ?? []);
  const theirArrangement = saving ? JSON.stringify([orderOf(stored.order), folders, incoming.map((doc) => [doc.id, doc.folder ?? ""])]) : "";
  /** @param {Chart | undefined} own @param {Chart | undefined} other */
  const later = (own, other) => {
    if (!own || !other) return /** @type {Chart} */ (own ?? other);
    const doc = own.updated >= other.updated ? own : other, by = saving && !(placedThen.has(own.id) && placedThen.get(own.id) === (own.folder ?? "")) ? own : other;
    if (by.folder) doc.folder = by.folder;
    else delete doc.folder;
    return doc;
  };
  const [lead, others] = saving ? [library.charts, incoming] : [incoming, library.charts], leading = new Set(lead.map((doc) => doc.id));
  const merged = [...others.filter((doc) => !leading.has(doc.id)), ...lead.map((doc) => later(ours.get(doc.id), theirs.get(doc.id)))];
  for (const doc of merged) if (ours.get(doc.id) !== doc) histories.delete(doc.id);
  library.charts = merged.filter(alive);
  const base = arranged ? new Map(arranged[1].map((folder) => [folder.id, folder])) : null, other = new Map(folders.map((folder) => [folder.id, folder]));
  const ourFolders = !base ? library.folders : library.folders.map((folder) => {
    const was = base.get(folder.id), theirs = other.get(folder.id);
    if (!was || !theirs) return folder;
    const out = /** @type {any} */ ({ ...folder });
    for (const key of /** @type {const} */ (["name", "icon", "parent", "closed"])) if (folder[key] === was[key]) theirs[key] === undefined ? delete out[key] : (out[key] = theirs[key]);
    return /** @type {Folder} */ (out);
  });
  const [first, second] = saving ? [ourFolders, folders] : [folders, library.folders], firstIds = new Set(first.map((folder) => folder.id));
  library.folders = foldersOf([...first, ...second.filter((folder) => !firstIds.has(folder.id))].filter((folder) => !Object.hasOwn(library.deleted, folder.id)), library.charts);
  if (!saving || (arranged && JSON.stringify(library.order) === JSON.stringify(arranged[0]))) library.order = orderOf(stored.order);
  const theirPresets = presetsOf(stored.presets), ourPresets = new Set(library.presets), byPreset = /** @type {Map<string, Chart>} */ (new Map());
  for (const doc of [...library.presets, ...theirPresets]) {
    const have = byPreset.get(/** @type {string} */ (doc.preset));
    if (!have || doc.updated > have.updated) byPreset.set(/** @type {string} */ (doc.preset), doc);
  }
  library.presets = [...byPreset.values()].filter(alive);
  for (const doc of library.presets) if (!ourPresets.has(doc)) histories.delete(doc.id);
  // (Its charts this tab can't read, set aside: not counted, or each tab would save over the other's.)
  if (!saving && !writing && holdings(library.charts, library.folders, library.deleted, library.presets) !== holdings(incoming, folders, deleted, theirPresets)) save();
  arrangedAs = saving ? theirArrangement : arrangedNow();
  if (!chart) return;
  const current = isPreview(chart) ? chart : library.charts.find((doc) => doc.id === chart.id) ?? library.presets.find((doc) => doc.id === chart.id || (chart.preset !== undefined && doc.preset === chart.preset)) ?? chart;
  drawThumbs();
  if (isOpen($("palette"))) showPalette();
  if (current === chart) return (withRest(() => inspector.pruneDrafts()), renderCharts());
  chart = /** @type {Chart} */ (current);
  withRest(() => inspector.pruneDrafts());
  if (edits.past.length || edits.future.length) notice("Edited in another tab. Undo starts over from here");
  edits.past = [];
  edits.future = [];
  revision++;
  refresh();
}
// (Another tab's save, or back from the browser's history as it was left (another tab's saves unheard), this one's
// arrangement changed since it last wrote it (a chart moved, a folder renamed, just now): this one's wins, as if it
// were saving.)
const takeIn = () => reconcile(undefined, arrangedNow() !== arrangedAs);
addEventListener("storage", (event) => event.key === STORE && takeIn());
addEventListener("pageshow", (event) => event.persisted && takeIn());

/**
 * By touch also as it's hidden: a phone closes an app put away when it likes, with no word to the page (a computer's tab put away runs on).
 * @param {(hidden: boolean) => void} run
 */
function onLeave(run) {
  addEventListener("pagehide", () => run(false));
  if (TOUCH) document.addEventListener("visibilitychange", () => document.visibilityState === "hidden" && run(true));
}
/** @type {Map<Element, () => void>} */
const leftOpen = new Map();
const leftWith = (/** @type {Element} */ field, /** @type {() => void} */ end) => !document.hasFocus() && Boolean(leftOpen.set(field, end));
addEventListener("focus", (event) => event.target === window && requestAnimationFrame(() => {
  const ends = [...leftOpen].filter(([field]) => document.activeElement !== field);
  leftOpen.clear();
  for (const [, end] of ends) end();
}));
/** @param {() => void} write @returns {(() => void) & { cancel: () => void }} */
function writeSoon(write, timeout = 0) {
  let due = false;
  const now = () => due && ((due = false), write());
  onLeave(now);
  return Object.assign(() => void (due || ((due = true), whenIdle(now, { timeout }))), { cancel: () => void (due = false) });
}
// Leaving (or hidden): what's being typed kept, and what's not yet written written now (as a save would, full storage
// too). Hidden (a phone's app switched), what's typed in place stays open as it is, to go on with (the focus comes
// back to it; the app closed meanwhile, that alone is lost).
onLeave((hidden) => {
  flushEdits(hidden);
  if (!saveDue && !unsaved) return;
  saveDue = announcing = false;
  written(persist(), false);
});

/** @param {(doc: Chart) => void} edit @param {EditOptions} [options] */
function commit(edit, /** @type {EditOptions} */ options) {
  return editChart(chart, edit, options);
}
/**
 * @typedef {{ coalesce?: string, quiet?: boolean, look?: boolean, keepMenu?: boolean, view?: boolean }} EditOptions
 * @param {Chart} doc
 * @param {(doc: Chart) => void} edit
 * @param {EditOptions} [options]
 * @returns {boolean}
 */
function editChart(doc, edit, { coalesce = "", quiet = false, look = false, keepMenu = false, view = false } = {}) {
  const open = doc === chart;
  if (!open || home) reached(doc);
  const before = JSON.stringify(doc), sources = new Map(doc.layers.map((layer) => [layer.id, layer.series ?? layer.formula]));
  edit(doc);
  for (const layer of doc.layers) if (layer.maxInterval && sources.get(layer.id) !== (layer.series ?? layer.formula)) delete layer.maxInterval;
  normalize(doc);
  if (doc.blank && (doc.layers.length || doc.title || doc.drawings?.length)) delete doc.blank;
  if (JSON.stringify(doc) === before) return false;
  const now = Date.now(), steps = open ? edits : histories.get(doc.id) ?? { past: [], future: [] };
  // One step: a drag's edits, however long it lasts (the next drag, another); typed ones a second apart. (Joining
  // only a step there's still: the history let go meanwhile (another tab's edit taken in), it starts one anew.)
  // (Nor across another step done since, in any history: one undo takes back one thing.)
  const joins = open && coalesce && edits.past.at(-1)?.at === stepCount && coalesce === edits.key && (holding ? edits.held === holding : !edits.held && now - edits.at < 1000);
  recordStep(steps, joins ? /** @type {ChartStep} */ (edits.past.pop()).state : before);
  if (!view) stampEdit(doc);
  if (!view && isPreview(doc)) touchPreview(doc);
  if (!open) return keepHistory(doc.id, steps), offPage(), true;
  const kept = edits.past.length;
  if (edits.past.length > UNDO_STEPS) edits.past.shift();
  trimSteps(edits, UNDO_CHARS);
  if (edits.past.length < kept) keptByEdit.delete(chart);
  edits.key = coalesce;
  edits.at = now;
  edits.held = holding;
  if (isPreview(chart) && drafts.has(chart) && !quiet && !look) (keepPreview(false), keptByEdit.set(chart, edits.past.length));
  changed(!quiet, keepMenu);
  return true;
}
/** @type {Map<Chart, number>} */
const inReach = new Map();
let reachAt = -1;
/** @param {Chart} doc */
function reached(doc) {
  if (reachAt !== navigations) inReach.clear(), (reachAt = navigations);
  if (!inReach.has(doc)) inReach.set(doc, stepCount);
}
function reachable() {
  const all = new Map([[home ? null : chart, 0], [null, 0]]);
  if (reachAt === navigations) for (const [doc, from] of inReach) if (!all.has(doc) && (doc === chart || chartById(doc.id) === doc)) all.set(doc, from);
  return all;
}
const stepsOf = (/** @type {Chart | null} */ doc) => (!doc ? libraryEdits : doc === chart ? edits : histories.get(doc.id));
/** @param {{ past: ChartStep[], future: ChartStep[] }} steps @param {string} state */
function recordStep(steps, state) {
  steps.past.push({ state, at: ++stepCount });
  steps.future = [];
  dropRedos();
}
function dropRedos() {
  for (const [doc, from] of reachable()) {
    const future = stepsOf(doc)?.future;
    while (future && (future.at(-1)?.at ?? 0) > from) future.pop();
  }
}
function historyFor(/** @type {boolean} */ back) {
  let found, at = 0;
  for (const [doc, from] of reachable()) {
    const step = stepsOf(doc)?.[back ? "past" : "future"].at(-1);
    if (step && step.at > from && step.at > at) (found = doc), (at = step.at);
  }
  return found;
}
function undoLast() {
  if (!home && unplacePath()) return;
  takeStep(true);
}
function redoLast() {
  if (!home && (replacePath() || placingPath())) return;
  takeStep(false);
}
function takeStep(/** @type {boolean} */ back) {
  leaveField();
  inspector.flushPick();
  travelIn(historyFor(back), back);
}
function travelIn(/** @type {Chart | null | undefined} */ doc, /** @type {boolean} */ back) {
  if (doc === undefined) return;
  if (!doc) return back ? undoLibrary() : redoLibrary();
  if (doc === chart) return back ? undo() : redo();
  const steps = /** @type {{ past: ChartStep[], future: ChartStep[] }} */ (stepsOf(doc)), now = JSON.stringify(doc);
  if (!putState(doc, moveStep(steps, back, now).state, now) && isPreview(doc)) asideDirty.add(doc);
  keepHistory(doc.id, steps);
  offPage();
  toast(`${back ? "Undone" : "Redone"} in “${titleOf(doc)}”`);
}
function offPage() {
  save();
  renderCharts();
  syncUndo();
}
/** @returns {{ state: string, drawn: { kind: DrawingKind, picks: number } | undefined }} */
function moveStep(/** @type {{ past: ChartStep[], future: ChartStep[] }} */ steps, /** @type {boolean} */ back, /** @type {string} */ now) {
  const { state } = /** @type {ChartStep} */ (steps[back ? "past" : "future"].pop());
  steps[back ? "future" : "past"].push({ state: now, at: ++stepCount });
  const drawn = heatmaps.drawnSteps.get(state);
  heatmaps.drawnSteps.delete(state);
  if (drawn) heatmaps.noteDrawn(now, drawn);
  return { state, drawn };
}
function keepHistory(/** @type {string} */ id, /** @type {{ past: ChartStep[], future: ChartStep[] }} */ steps) {
  steps.past = steps.past.slice(-KEPT_STEPS);
  steps.future = steps.future.slice(-KEPT_STEPS);
  trimSteps(steps, KEPT_CHARS);
  histories.delete(id);
  histories.set(id, steps);
  const held = new Set([...reachable().keys()].map((doc) => doc?.id));
  for (const old of histories.keys()) if (histories.size > KEPT_HISTORIES && !held.has(old)) histories.delete(old);
}
/** @returns {boolean} */
function putState(/** @type {Chart} */ doc, /** @type {string} */ state, /** @type {string} */ now) {
  const was = JSON.parse(state), folder = doc.folder, stamped = doc.updated, blank = doc.blank;
  const bare = (/** @type {Record<string, any>} */ item) => JSON.stringify(Object.keys(item).filter((key) => !["range", "updated", "folder", "id", "blank"].includes(key)).sort().map((key) => [key, item[key]]));
  const rangeOnly = was.range !== doc.range && bare(was) === bare(JSON.parse(now)), shapes = new Map(doc.layers.map((layer) => [layer.id, shapeKey(layer)]));
  for (const key of Object.keys(doc)) if (!(key in was)) delete /** @type {any} */ (doc)[key];
  Object.assign(doc, was, { range: rangeOnly ? was.range : doc.range, id: doc.id });
  if (folder) doc.folder = folder;
  else delete doc.folder;
  if (!blank) delete doc.blank;
  doc.updated = stamped;
  if (!rangeOnly) stampEdit(doc);
  for (const layer of doc.layers) if (shapes.get(layer.id) !== shapeKey(layer)) looksLeft.delete(`${doc.id}:${layer.id}`);
  return rangeOnly;
}
function travel(/** @type {boolean} */ back) {
  leaveField();
  inspector.flushPick();
  if (!edits[back ? "past" : "future"].length) return;
  stopDrawing();
  pathResume = null;
  const now = JSON.stringify(chart), { state, drawn } = moveStep(edits, back, now);
  const ranged = putState(chart, state, now);
  if (isPreview(chart) && !ranged) asideDirty.add(chart);
  edits.key = "";
  if (toastOf === chart) hideToast();
  const keptAt = keptByEdit.get(chart);
  if (keptAt !== undefined && back && edits.past.length === keptAt - 1 && library.charts.includes(chart)) {
    keepFocus(null, () => $("new-chart"), () => $("plot-keys"));
    discard(chart);
    addPreview(chart);
    save(false);
    renderCharts();
    clearLink();
  } else if (keptAt !== undefined && !back && edits.past.length === keptAt && isPreview(chart)) keepPreview(false);
  changed(true);
  if (drawn && !home && back && (undoTook || picks === drawn.picks)) (takeUp(drawn.kind, false, true), (undoTook = drawn.kind));
  else if (drawn && !home && !back && undoTook === drawn.kind && tool === drawn.kind) takeUp("", false, true);
  say(back ? "Undone" : "Redone");
}
function changed(announce, keepMenu = false) {
  announce &&= !isPreview(chart);
  revision++;
  if (!chartChanged) (chartChanged = true), queueMicrotask(() => (chartChanged = false));
  // (Before the rest, no menu is open, no formula unfinished, no drawing chosen.)
  withRest(() => {
    if (!keepMenu) menus.hideMenu();
    inspector.pruneDrafts();
    chosen = new Set([...chosen].filter((id) => pen.drawingById(id)));
  });
  if (!layerById(selected)) selected = "";
  for (const id of layerPicks) if (!layerById(id)) layerPicks.delete(id);
  if (!layerById(layerPickLast)) layerPickLast = [...layerPicks][0] ?? "";
  if (!layerById(pickHead)) pickHead = layerPickLast;
  save(announce);
  // While a drag holds (a slider's, a color surface's): the chart drawn at once, the rest of the page once it ends
  // (see track), not under the pointer at every step.
  if (holding) {
    heldEdits = true;
    return redrawChart(false, true);
  }
  refresh();
}
let heldEdits = false;
const undo = () => travel(true);
const redo = () => travel(false);

const previousChart = () => viewed().map(chartById).find((doc) => doc && doc !== chart) ?? library.charts.find((doc) => doc !== chart) ?? previewCharts.find((doc) => doc !== chart) ?? null;
const previousPreset = () => {
  const id = viewed().find((other) => other !== chart?.id);
  return id && !chartById(id) ? presetById(id) : -1;
};
/** Where each chart was left, zoomed or panned off its preset: each edge of the view as a point's time and how many bars it lies from that point, so it holds as new points come in, and past either end of the data. On its preset a chart keeps nothing: the preset follows the newest data. */
const VIEWS = "bitview-studio:views";
let viewsLeft = /** @type {Record<string, any> | null} */ (null), viewsBase = /** @type {Record<string, any>} */ ({}), viewsUnkept = false;
const viewsNow = () => viewsLeft ?? ((viewsBase = storage.get(VIEWS, {})), (viewsLeft = { ...viewsBase }));
/** @param {Record<string, any>} all */
const putViews = (all) => {
  const stored = storage.get(VIEWS, {}), at = (/** @type {any} */ view) => (view === undefined ? -Infinity : view?.at ?? 0);
  for (const id of new Set([...Object.keys(stored), ...Object.keys(viewsBase)])) {
    const theirs = Object.hasOwn(stored, id) ? stored[id] : undefined, base = Object.hasOwn(viewsBase, id) ? viewsBase[id] : undefined;
    if ((theirs === undefined) === (base === undefined) && at(theirs) === at(base)) continue;
    const ours = Object.hasOwn(all, id) ? all[id] : undefined;
    if (ours === base || at(theirs) > at(ours)) theirs === undefined ? delete all[id] : (all[id] = theirs);
    theirs === undefined ? delete viewsBase[id] : (viewsBase[id] = theirs);
  }
  const ids = Object.keys(all);
  if (ids.length > 100) for (const id of ids.sort((a, b) => (all[a]?.at ?? 0) - (all[b]?.at ?? 0)).slice(0, ids.length - 100)) delete all[id];
  if (storage.set(VIEWS, all) || (storage.drop(PREVIEWS, THUMBS, PRESET_MARKS, SIDE_PRESETS), storage.set(VIEWS, all))) (viewsBase = { ...all }), (viewsUnkept = false);
  else if (!viewsUnkept) (viewsUnkept = true), notice("Can't remember where each chart was left: this browser's storage is full or blocked");
};
addEventListener("storage", (event) => event.key === VIEWS && ((viewsLeft = null), document.visibilityState !== "visible" && followLinked()));
addEventListener("pageshow", (event) => event.persisted && (viewsLeft = null));
const isEdge = (edge) => Array.isArray(edge) && edge.length === 2 && edge.every(Number.isFinite) && Math.abs(edge[0]) < 1e11;
function isView(/** @type {any} */ view) {
  if (!view || !Object.hasOwn(INTERVAL_LABEL, view.index) || !isEdge(view.from) || !isEdge(view.to)) return false;
  if (view.latest !== undefined && !(Array.isArray(view.latest) && view.latest.length === 2 && view.latest.every(Number.isFinite))) return false;
  try {
    const { start, end } = viewTimes(view);
    return Math.abs(start) < 1e11 && Math.abs(end) < 1e11;
  } catch {
    return false;
  }
}
/** @returns {{ index: string, from: [number, number], to: [number, number], latest?: [number, number] } | null} */
function leftView(id) {
  const left = viewsNow()[id];
  return isView(left) ? (({ at, ...view }) => view)(left) : null;
}
const LINKED = "*";
const spanKey = (/** @type {string} */ id, /** @type {string} */ index, linked = viewKey(id) === LINKED) => `${id}@${index}${linked ? "@linked" : ""}`;
/**
 * Chunks loaded before it move its points, not its dates: the linked view is said again only once the chart has moved from here (see storeView), never for what it showed meanwhile.
 * @type {any}
 */
let linkedAt = null;
let linkedTook = 0;
let handled = false;
let handledYet = false;
const handsHeld = new Set();
/** @type {(() => void)[]} */
const afterHands = [];
for (const type of ["pointerdown", "wheel", "keydown"]) addEventListener(type, (event) => {
  handled = handledYet = true;
  if (!(event instanceof PointerEvent)) return;
  const id = event.pointerId;
  handsHeld.add(id);
  press(event, { end: () => handsHeld.delete(id) && !handsHeld.size && afterHands.splice(0).forEach((run) => setTimeout(run)) });
}, { capture: true, passive: true });
/** @type {{ view: any, at: number, id: string } | null} */
let retaking = null;
/** @param {any} a @param {any} b */
const sameEdges = (a, b) => Boolean(a && b && a.index === b.index) && ((x, y, bar) => Math.abs(x.start - y.start) < bar && Math.abs(x.end - y.end) < bar)(viewTimes(a), viewTimes(b), (INTERVAL_DAYS[a.index] * 86400) / 100);
function carrySpan() {
  if (!chart) return;
  const from = spanKey(chart.id, chart.index, !prefs.linkView), span = spans.get(from);
  if (!span) return;
  spans.delete(from);
  spans.set(spanKey(chart.id, chart.index), span);
}
const viewKey = (/** @type {string} */ id) => (prefs.linkView && id === chart?.id ? LINKED : id);
const openView = (/** @type {Chart} */ doc) => leftView(viewKey(doc.id));
let linkWindow = "";
const openingView = (/** @type {Chart} */ doc) => (linkWindow === doc.id ? leftView(doc.id) : retaking && doc.id === retaking.id && viewKey(doc.id) === LINKED ? retaking.view : openView(doc));
function edgeOf(logical, times) {
  const i = clamp(Math.round(logical), 0, times.length - 1);
  return [times[i], logical - i];
}
const mostBars = () => (engine.time.width ? engine.time.width / engine.time.minSpacing : Infinity);
/** @param {{ from: number, to: number }} range @param {"middle" | "from" | "to"} [keep] */
function within({ from, to }, keep = "middle", most = mostBars()) {
  const over = to - from - most;
  if (over <= 0) return { from, to };
  return keep === "to" ? { from: from + over, to } : keep === "from" ? { from, to: to - over } : { from: from + over / 2, to: to - over / 2 };
}
/** @param {{ from: number, to: number }} range */
function ontoData(range) {
  const shift = engine.time.length ? engine.time.held(range.from, range.to) : 0;
  return shift ? { from: range.from + shift, to: range.to + shift } : null;
}
const logicalOf = (/** @type {[number, number]} */ [time, bars], /** @type {number[]} */ times, /** @type {string} */ index) => (time > /** @type {number} */ (times.at(-1)) ? times.length - 1 + indexAt(index, time) - indexAt(index, /** @type {number} */ (times.at(-1))) : Math.max(0, lowerBound(times, time + 1) - 1)) + bars;
function viewOn(left, index, times, gap = rightGap()) {
  if (left.latest && left.index === index) return { from: times.length - 1 + left.latest[0], to: times.length - 1 + left.latest[1] };
  const view = viewAsLeft(left, index, times, gap), shift = left.latest ? Math.max(0, times.length - 1 - view.to) : 0;
  return shift > 0 ? { from: view.from + shift, to: view.to + shift } : view;
}
function viewAsLeft(left, index, times, gap = rightGap()) {
  if (left.index === index) return { from: logicalOf(left.from, times, index), to: logicalOf(left.to, times, index) };
  const { start, end } = viewTimes(left), last = /** @type {number} */ (times.at(-1)), lastIndex = indexAt(index, last);
  const placeOf = (/** @type {number} */ time) => {
    if (time < times[0]) return (time - times[0]) / (INTERVAL_DAYS[index] * 86400);
    if (time >= last) {
      const k = indexAt(index, time), from = timeAt(index, k);
      return times.length - 1 + k - lastIndex + (time - from) / (timeAt(index, k + 1) - from);
    }
    const at = lowerBound(times, time), i = times[at] === time ? at : at - 1;
    return i + (time - times[i]) / (times[i + 1] - times[i]);
  };
  const plot = engine.time.width, bars = Math.min(mostBars(), Math.max(1, placeOf(end) - placeOf(start)));
  const room = plot > gap ? (gap * bars) / (plot - gap) : 0;
  const to = Math.min(placeOf(end), times.length - 1 + room), from = Math.min(placeOf(start), to - 1);
  return { from, to };
}
function viewTimes(left) {
  const timeOf = (/** @type {number} */ i) => (i >= 0 ? timeAt(left.index, i) : timeAt(left.index, 0) + i * INTERVAL_DAYS[left.index] * 86400);
  const at = (/** @type {[number, number]} */ [time, bars]) => {
    const place = indexAt(left.index, time) + bars, i = Math.floor(place);
    return timeOf(i) + (place - i) * (timeOf(i + 1) - timeOf(i));
  };
  return { start: at(left.from), end: at(left.to) };
}
function linkView(/** @type {boolean} */ on) {
  if (on === prefs.linkView) return;
  prefs.linkView = on;
  savePrefs();
  carrySpan();
  linkedAt = null;
  storeView(false);
  renderLinkView();
  toast(on ? "Every chart shows these dates, in every tab" : "Each chart keeps its own view");
}
function renderLinkView() {
  $("link-view").setAttribute("aria-pressed", String(prefs.linkView));
}
function followLinked() {
  if (home || !chartShown() || viewKey(chart.id) !== LINKED) return;
  const linked = leftView(LINKED), at = viewsNow()[LINKED]?.at;
  if (!linked || !at || at === linkedTook) return;
  engine.controller.stop();
  retaking = { view: linked, at, id: chart.id };
  spans.delete(spanKey(chart.id, chart.index));
  void draw();
}
document.addEventListener("visibilitychange", () => document.visibilityState === "visible" && followLinked());
addEventListener("focus", followLinked);
const keepView = writeSoon(() => saveView(), 1000);
function saveView() {
  storeView(false);
  // (Mid-gesture, the bar waits for the view to rest: Safari allows only so many writes.)
  if (!viewMoving) showAddress();
}
/** @param {boolean} asDrawn */
/** @param {{ from: number, to: number }} range @param {number[]} times */
const viewRecord = (range, times, index = shownIndex()) => ({ index, from: edgeOf(range.from, times), to: edgeOf(range.to, times), ...(range.to > times.length - 1.5 && resolved?.live ? { latest: [range.from - (times.length - 1), range.to - (times.length - 1)] } : {}) });
function shownView() {
  const range = shownSpan(), times = resolved?.times;
  return chartShown() && range && times?.length && !presetShown ? viewRecord(range, times) : null;
}
let seen = document.visibilityState === "visible";
document.addEventListener("visibilitychange", () => (seen ||= document.visibilityState === "visible"));
function storeView(asDrawn, leaving = false) {
  keepView.cancel();
  // (Kept only from a page in sight, or as one that was is left: one out of sight isn't moved by anyone, what it shows
  // there is another's (the linked view followed), or as it was; said, it would take another tab's view back to it.
  // A page made ahead, unseen (a prerender), keeps nothing.)
  if (!(document.visibilityState === "visible" || (leaving && seen))) return;
  const times = resolved?.times;
  if (!chartShown() || (!asDrawn && shownIndex() !== chart.index) || !times?.length || !engine.time.width) return;
  const range = (asDrawn && engine.controller.target) || shownSpan();
  const all = viewsNow(), key = viewKey(chart.id), shown = range ? viewRecord(range, times) : null;
  if (key === LINKED && (!handled || sameEdges(shown, linkedAt))) return;
  if (key === LINKED) (linkedAt = shown), (handled = handsHeld.size > 0);
  if ((presetShown && key !== LINKED) || !shown) delete all[key];
  else all[key] = { ...shown, at: Date.now() };
  if (key === LINKED && all[key]) linkedTook = all[key].at;
  putViews(all);
}
onLeave(() => storeView(false, true));
let navigations = 0;
let blockAtHome = false;
let undrawn = false;
/** @param {Chart} doc */
function openChart(doc) {
  if (doc.id !== chart?.id) navigations++;
  if (linkWindow !== doc.id) linkWindow = "";
  blockAtHome = false;
  lastOpen = "";
  quickHide();
  if (chart && chart !== doc && !isPreview(chart)) keptByEdit.delete(chart);
  gestures.stopCarrying();
  leaveField();
  saveView();
  if (history.state?.chart && history.state.chart !== doc.id) history.pushState({ chart: doc.id }, "");
  for (const item of previewCharts) if (item !== doc && !item.layers.length) (dropPreview(item), forgetPreview(item));
  brokenLink = "";
  const previous = chart;
  chart = doc;
  library.charts = library.charts.filter(alive);
  library.presets = library.presets.filter(alive);
  linkedAt = null;
  if (previous?.preset !== undefined && previous !== doc) prunePresets(doc);
  if (previous && previous !== doc && emptyYours(previous) && library.charts.includes(previous)) dropEmpty(previous);
  library.active = doc.id;
  rememberViewed(doc.id);
  selected = "";
  chosen = new Set();
  layerPicks.clear();
  drawingsHidden = false;
  const left = { past: edits.past, future: edits.future }, kept = previous?.id === doc.id ? left : histories.get(doc.id);
  histories.delete(doc.id);
  if (previous && previous.id !== doc.id && (left.past.length || left.future.length)) keepHistory(previous.id, left);
  edits.past = kept?.past ?? [];
  edits.future = kept?.future ?? [];
  edits.key = "";
  edits.held = 0;
  revision++;
  menus.hideMenu();
  pen.putDown();
  save();
  refresh(true);
}
function leaveEmpty() {
  const leaving = emptyYours(chart) ? { doc: chart, at: library.charts.indexOf(chart), folder: chart.folder } : null;
  if (backToHome()) {
    cancelling = chart.id;
    cancelledNew = leaving;
    return history.back();
  }
  const back = previousChart(), preset = previousPreset();
  if (!(isPreview(chart) || emptyYours(chart)) || chart.layers.length) return;
  cancelledNew = leaving;
  if (preset >= 0) {
    keepFocus(null, () => $("plot-keys"));
    return openChart(presetChart(preset));
  }
  if (!back) {
    cancelling = chart.id;
    return showHome(true, leftPlace);
  }
  keepFocus(null, () => $("chart-list").querySelector(`[data-open-chart="${CSS.escape(back.id)}"]`), () => $("plot-keys"), () => $("new-chart"), () => $("charts-nav"));
  openChart(back);
}
/** @param {Chart} doc */
function adopt(doc) {
  library.charts.unshift(doc);
  leaveHome(() => openChart(doc));
}
/** @param {Chart} doc */
function showPreview(doc, push = true) {
  flushEdits();
  const like = previewLike(doc), shown = like ?? doc;
  if (like) {
    if (like === chart) saveView();
    lookAs(like, doc);
  }
  if (closedToast === shown) hideToast();
  leaveHome(() => {
    addPreview(shown);
    keepAsideOpened(shown);
    if (shown === chart) (displayed = null), refresh(true);
    else openChart(shown);
  }, null, push);
  return shown;
}
function closePreview(doc = chart) {
  if (!isPreview(doc)) return;
  const back = chart === doc ? previousChart() : chart;
  if (!back) return;
  flushEdits();
  const focused = Boolean(document.activeElement?.closest("[data-preview-row]"));
  if (chart === doc) saveView();
  const at = previewCharts.indexOf(doc), open = chart === doc, view = viewsNow()[doc.id], next = previewCharts[at + 1] ?? previewCharts[at - 1];
  dropPreview(doc);
  forgetPreview(doc);
  previews.delete(doc.id);
  if (open) openChart(back);
  save();
  renderCharts();
  renderHome();
  if (focused) /** @type {HTMLElement | null} */ ($("chart-list").querySelector(`[data-open-chart="${CSS.escape(next && next !== back ? next.id : chart.id)}"]`))?.focus();
  const reopen = () => {
    if (chartById(doc.id)) return;
    closedPreviews.delete(doc.id);
    if (view) (viewsNow()[doc.id] = view), putViews(viewsNow());
    previewCharts.splice(Math.min(at, previewCharts.length), 0, doc);
    asideDirty.add(doc);
    const again = () => void (isPreview(doc) && closePreview(doc));
    if (open && !home) return (showPreview(doc), save(), again);
    save();
    renderCharts();
    renderHome();
    return again;
  };
  closedPreviews.set(doc.id, titleOf(doc));
  if (touched.has(doc)) toast(`Closed the preview “${titleOf(doc)}”`, { label: "Undo", run: reopen, undone: "Preview back" }), (closedToast = doc);
  else libraryStep(`Closed the preview “${titleOf(doc)}”`, reopen, "Preview back");
}
const closedPreviews = new Map();
/** @type {Chart | null} */
let closedToast = null;
/** @returns {Chart[]} */
const previewsAside = () => ((stored) => (Array.isArray(stored) ? stored.filter((doc) => typeof doc?.id === "string" && !(doc.made !== undefined && doc.untouched === true)) : []))(storage.get(PREVIEW_ASIDE, null));
/** @type {Map<string, number>} */
const keptPreviews = new Map();
const laterAside = (/** @type {any} */ item, /** @type {number} */ updated) => (Number(item?.updated) || 0) > updated;
const forgetPreview = (/** @type {Chart} */ doc) => {
  asideDirty.delete(doc);
  const left = previewsAside().filter((item) => item.id !== doc.id || laterAside(item, doc.updated));
  if (left.length) storage.set(PREVIEW_ASIDE, left);
  else storage.drop(PREVIEW_ASIDE);
};
function keepPreview(announce = true, doc = chart) {
  if (!isPreview(doc)) return;
  asideDirty.delete(doc);
  keptPreviews.set(doc.id, doc.updated);
  const listed = inLibrary(doc), old = doc.id;
  if (!listed && drafts.has(doc) && Object.hasOwn(library.deleted, doc.id)) recover(doc);
  if (listed || Object.hasOwn(library.deleted, doc.id)) rekey(doc, uid());
  if (!listed && old !== doc.id) chartAgain.set(old, doc.id);
  if (document.activeElement === $("keep")) keepFocus(null, () => $("chart-more"), () => $("plot-keys"));
  library.charts.unshift(doc);
  dropPreview(doc);
  save(announce);
  renderCharts();
  clearLink();
}

function keepChart(doc = chart) {
  if (!isPreview(doc)) return;
  keepPreview(true, doc);
  libraryStep(`Saved “${titleOf(doc)}”`, () => {
    const now = library.charts.find((other) => other.id === doc.id);
    if (!now) return;
    discard(now);
    keptPreviews.delete(now.id);
    asideDirty.add(now);
    addPreview(now);
    save(false);
    renderCharts();
    renderHome();
    return () => {
      if (!isPreview(now)) return;
      recover(now);
      keepChart(now);
    };
  }, `“${titleOf(doc)}” a preview again`);
}
const recent = () => {
  const names = storage.get(RECENT, []);
  return Array.isArray(names) ? names.filter((name) => isSeriesName(name) && !gone.has(name)).slice(0, 8) : [];
};
const remember = (name) => storage.set(RECENT, [name, ...recent().filter((other) => other !== name)].slice(0, 8));
const SUGGESTED = ["price", "realized_price", "mvrv", "market_cap", "supply_in_profit", "hashrate"];
const VIEWED = "bitview-studio:viewed";
const viewed = () => {
  const ids = storage.get(VIEWED, []);
  return Array.isArray(ids) ? ids.filter((id) => typeof id === "string" && /^[a-z0-9]{1,16}$/.test(id)) : [];
};
const rememberViewed = (id) => storage.set(VIEWED, [id, ...viewed().filter((other) => other !== id)].slice(0, 12));
const RECENT_SHOWN = 5;
/** @returns {{ id: string, doc: Chart, preset: number }[]} */
const recentCharts = () => viewed().flatMap((id) => {
  const kept = chartById(id), preset = kept ? presetIndex(kept) : presetById(id);
  const doc = preset >= 0 ? presetDoc(preset) : kept?.preset !== undefined && presets ? undefined : kept;
  return doc?.layers.length ? [{ id, doc, preset }] : [];
}).slice(0, RECENT_SHOWN);
/** @param {string} id */
function openRecent(id) {
  const doc = chartById(id), preset = doc ? presetIndex(doc) : presetById(id);
  if (preset >= 0) return presetsEdit.openPreset(preset);
  delete document.body.dataset.sheet;
  grid.go(doc);
}
/** @param {{ doc: Chart, preset: number }} item */
const recentMark = ({ doc, preset }) => (preset >= 0 ? (presetMarks.has(grid.presetMarkKey(preset)) || grid.presetMarksAsked.add(preset), grid.presetMark(preset)) : doc.preset !== undefined && presetMarks.has(grid.keptMarkKey(doc)) ? grid.markOf(presetMarks.get(grid.keptMarkKey(doc)) ?? grid.NO_MARK) : grid.chartMark(doc));

/** @param {Element | null} away @param {...(() => Element | null | undefined)} next */
function keepFocus(away, ...next) {
  const had = document.activeElement;
  if (!had || had === document.body) return;
  const go = () => {
    for (const pick of next) {
      const element = pick();
      if (element instanceof HTMLElement && element.isConnected && !away?.contains(element) && element.checkVisibility({ checkVisibilityCSS: true })) return (element.focus({ preventScroll: true }), true);
    }
    return false;
  };
  if (away?.contains(had)) return void go();
  const lost = () => !had.isConnected || document.activeElement === document.body || !had.checkVisibility();
  requestAnimationFrame(() => lost() && !go() && setTimeout(() => lost() && go(), MOTION.page + 40));
}

function applySidebar() {
  const collapsed = prefs.sidebar === "collapsed";
  if (collapsed) keepFocus($("side-grip"), () => $("sidebar-toggle"));
  if (collapsed) document.body.dataset.sidebar = "collapsed";
  else delete document.body.dataset.sidebar;
  const toggle = $("sidebar-toggle");
  toggle.setAttribute("aria-expanded", String(!collapsed));
  toggle.dataset.tip = collapsed ? "Expand" : "Collapse";
  toggle.querySelector(".label").textContent = collapsed ? "Expand" : "Collapse";
}
function applyPanel(state = prefs.panel) {
  if (state === "closed") keepFocus($("panel-grip"), () => $("panel-toggle"), () => $("plot-keys"));
  document.body.dataset.panel = state;
  $("panel-toggle").setAttribute("aria-pressed", String(state === "open"));
}
const panelShown = () => (phone.matches ? document.body.dataset.sheet === "panel" : document.body.dataset.panel === "open" || document.body.hasAttribute("data-peek"));
new MutationObserver(() => (fitMargins(), respot(), panelShown() && layersBehind ? writeLayers() : valuesSoon())).observe(document.body, { attributes: true, attributeFilter: ["data-panel", "data-sheet", "data-peek"] });
function fullOn() {
  return /** @type {HTMLElement} */ (document.querySelector(".app")).hasAttribute("data-full");
}
function togglePanel(open = !panelShown()) {
  if (!chart.layers.length) return;
  if (!open) gestures.stopCarrying();
  if (phone.matches) {
    if (open) document.body.dataset.sheet = "panel";
    else if (document.body.dataset.sheet === "panel") delete document.body.dataset.sheet;
    return;
  }
  document.body.removeAttribute("data-peek");
  if (!open) keepFocus($("panel"), () => $("panel-toggle"), () => $("plot-keys"));
  if (fullOn()) return applyPanel(open ? "open" : "closed");
  prefs.panel = open ? "open" : "closed";
  savePrefs();
  applyPanel();
}
applyPanel();

const SIDE_WIDTHS = /** @type {const} */ ({
  side: { prop: "--side", key: "sideWidth", min: 264, share: 0.34, cap: 480, element: "sidebar", grip: "side-grip" },
  panel: { prop: "--panel-kept", key: "panelWidth", min: 300, share: 0.4, cap: 560, element: "panel", grip: "panel-grip" },
});
/** @typedef {keyof typeof SIDE_WIDTHS} Widthed */
const CHART_ROOM = 500;
const besideChart = matchMedia("(min-width: 1101px) and (min-height: 501px)");
/** @param {Widthed} which */
const widthMost = (which) => {
  const { min, share, cap } = SIDE_WIDTHS[which], other = which === "side" ? (besideChart.matches ? Math.max(prefs.panelWidth ?? 0, SIDE_WIDTHS.panel.min) : 0) : widthOf("side");
  return Math.max(min, Math.min(cap, Math.round(innerWidth * share), innerWidth - CHART_ROOM - other));
};
const widthOf = (/** @type {Widthed} */ which) => ((kept) => (kept === null ? SIDE_WIDTHS[which].min : clamp(kept, SIDE_WIDTHS[which].min, widthMost(which))))(prefs[SIDE_WIDTHS[which].key]);
function applyWidths() {
  for (const which of /** @type {Widthed[]} */ (["side", "panel"])) {
    const { prop, key, min, grip } = SIDE_WIDTHS[which], kept = prefs[key], width = widthOf(which);
    if (kept === null) document.documentElement.style.removeProperty(prop);
    else document.documentElement.style.setProperty(prop, `${width}px`);
    for (const [name, value] of [["aria-valuemin", min], ["aria-valuemax", widthMost(which)], ["aria-valuenow", width]]) $(grip).setAttribute(name, String(value));
  }
}
applyWidths();
addEventListener("resize", applyWidths);
/** @type {{ from: number, to: number } | null} */
let resizeKeeps = null;
let resizes = 0;
/** @param {Widthed} which @param {number | null} px */
function setWidth(which, px) {
  const run = ++resizes;
  resizeKeeps ??= chartShown() ? shownSpan() : null;
  document.body.dataset.resizing = "";
  /** @type {any} */ (prefs)[SIDE_WIDTHS[which].key] = px === null ? null : Math.round(clamp(px, SIDE_WIDTHS[which].min, widthMost(which)));
  applyWidths();
  requestAnimationFrame(() => requestAnimationFrame(() => run === resizes && ((resizeKeeps = null), delete document.body.dataset.resizing)));
}
for (const which of /** @type {Widthed[]} */ (["side", "panel"])) {
  const grip = $(SIDE_WIDTHS[which].grip), element = () => $(SIDE_WIDTHS[which].element), sign = which === "side" ? 1 : -1;
  grip.addEventListener("pointerdown", (event) => {
    if (event.button !== 0 || grip.classList.contains("dragging")) return;
    event.preventDefault();
    const from = element().getBoundingClientRect().width, start = event.clientX;
    let x = start, frame = 0;
    grip.setPointerCapture(event.pointerId);
    grip.classList.add("dragging");
    holdChart();
    press(event, {
      captor: grip,
      move: (moved) => {
        x = moved.clientX;
        frame ||= requestAnimationFrame(() => ((frame = 0), holdChart(), setWidth(which, from + sign * (x - start)), stretchNow()));
      },
      end: () => {
        if (frame) (cancelAnimationFrame(frame), setWidth(which, from + sign * (x - start)));
        grip.classList.remove("dragging");
        releaseSize();
        savePrefs();
      },
    });
  });
  grip.addEventListener("dblclick", () => (setWidth(which, null), savePrefs()));
  grip.addEventListener("keydown", (event) => {
    const now = element().getBoundingClientRect().width, step = event.shiftKey ? 64 : 16;
    const to = event.key === "ArrowRight" ? now + sign * step : event.key === "ArrowLeft" ? now - sign * step : event.key === "Home" ? 0 : event.key === "End" ? Infinity : null;
    if (to === null || event.metaKey || event.ctrlKey || event.altKey) return;
    event.preventDefault();
    event.stopPropagation();
    setWidth(which, to);
    savePrefs();
  });
}
// A phone's sheet takes the focus as it opens, and gives it back to what opened it as it closes. Whole (dragged up),
// it's modal: what's behind it can't be reached, by Tab or a screen reader (the tab bar isn't behind it: it floats
// over it, live); at half height, or a column on its side, the chart beside stays live.
{
  let opener = /** @type {Element | null} */ (null), shown = false, wasModal = false;
  const sheetModal = () => {
    const panel = $("panel"), was = shown, modalWas = wasModal;
    if (!document.body.dataset.sheet) delete document.body.dataset.sheetSize;
    shown = phone.matches && document.body.dataset.sheet === "panel";
    // (Its slide in or out, and the dim's, begun once frames come on time: the chart laid out again beside it stalls
    // the frame they'd begin in, iOS showing the sheet more than half in at once, or gone in one frame.)
    if (shown !== was) grid.beginOnTime(...[panel, $("scrim")].flatMap((element) => element.getAnimations().filter((animation) => animation instanceof CSSTransition)));
    const modal = (wasModal = shown && sheetWhole());
    for (const element of [$("sidebar"), ...(/** @type {HTMLElement} */ (document.querySelector(".card-inner"))).children, ...(/** @type {HTMLElement} */ (panel.parentElement)).children]) {
      if (!element.contains(panel) && element.id !== "scrim" && !element.classList.contains("mobile-bar")) /** @type {HTMLElement} */ (element).inert = modal;
    }
    if (shown && (!was || (modal && !modalWas)) && !panel.contains(document.activeElement)) {
      if (!was) opener = document.activeElement;
      panel.focus({ preventScroll: true });
    } else if (!shown && opener) {
      const back = opener;
      opener = null;
      if ((panel.contains(document.activeElement) || document.activeElement === document.body) && back instanceof HTMLElement && back.isConnected) back.focus({ preventScroll: true });
    }
  };
  new MutationObserver(sheetModal).observe(document.body, { attributes: true, attributeFilter: ["data-sheet", "data-sheet-size"] });
  phone.addEventListener("change", sheetModal);
  heldUpright.addEventListener("change", sheetModal);
}
let sideKept = /** @type {{ key: string, top: number, rail: number } | null} */ (null);
function toggleSidebar() {
  const before = listRows($("chart-list")), was = grid.placesOf([...before.keys()], (row) => before.get(row)), scrolled = $("chart-list").scrollTop;
  prefs.sidebar = prefs.sidebar === "collapsed" ? "expanded" : "collapsed";
  savePrefs();
  if (!reduced()) document.body.toggleAttribute("data-sliding", true);
  const putAway = prefs.sidebar === "collapsed" && document.activeElement?.closest(".sidebar") && !document.activeElement.matches(".nav-item, .list-head-name, .search, .brand a, #sidebar-toggle") ? document.activeElement : null;
  const inFolder = prefs.sidebar === "collapsed" ? document.activeElement?.closest("#chart-list .in-folder") : null;
  let top = inFolder;
  while (top?.classList.contains("in-folder")) top = top.previousElementSibling;
  if (inFolder) /** @type {HTMLElement | null | undefined} */ (top?.querySelector(".folder-item"))?.focus({ preventScroll: true });
  applySidebar();
  grid.movingSide(renderCharts);
  const list = $("chart-list"), sight = list.getBoundingClientRect(), after = listRows(list), keys = [...before.values()], first = keys.findIndex((key) => (was.get(key) ?? -Infinity) >= sight.top - 1);
  const rowOf = new Map([...after].map(([row, key]) => [key, row]));
  const back = prefs.sidebar !== "collapsed" && sideKept && sideKept.rail === scrolled && rowOf.has(sideKept.key) ? sideKept : null;
  const anchor = back?.key ?? (first < 0 ? null : keys.slice(0, first + 1).reverse().find((key) => rowOf.has(key)));
  const kept = anchor ? rowOf.get(anchor) : undefined;
  if (kept) list.scrollTop += kept.getBoundingClientRect().top - /** @type {number} */ (back ? back.top : was.get(anchor));
  const focused = list.contains(document.activeElement) ? /** @type {Element} */ (document.activeElement).getBoundingClientRect() : null;
  if (focused) list.scrollTop += Math.min(0, focused.top - sight.top) || Math.max(0, focused.bottom - sight.bottom);
  sideKept = prefs.sidebar === "collapsed" && first >= 0 ? { key: keys[first], top: /** @type {number} */ (was.get(keys[first])), rail: list.scrollTop } : null;
  glideList(list, was);
  if (!widthMoving()) document.body.removeAttribute("data-sliding");
  grid.roveList();
  keepFocus(putAway, () => $("sidebar-toggle"));
}
const widthMoving = () => $("sidebar").getAnimations().some((motion) => motion instanceof CSSTransition && motion.transitionProperty === "width" && (motion.playState === "running" || motion.playState === "paused"));
const slid = (/** @type {TransitionEvent} */ event) => event.target === $("sidebar") && event.propertyName === "width" && !widthMoving() && document.body.removeAttribute("data-sliding");
$("sidebar").addEventListener("transitionend", slid);
$("sidebar").addEventListener("transitioncancel", slid);
applySidebar();

function animateMark() {
  const svg = /** @type {SVGSVGElement} */ (document.querySelector(".logo svg")), first = /** @type {SVGPathElement} */ (svg.querySelector("path"));
  const SIN = Math.tan(Math.PI / 6), COS = Math.sqrt(1 - SIN * SIN), C30 = Math.cos(Math.PI / 6), SQ3 = Math.sqrt(3);
  const REST = Math.PI / 4;
  const CUT = 0.2718, EASE = 0.798, HALF_HEIGHT = 0.3674 / COS, OPENING = 0.7231, RIM = 0.0979, CORNER = 0.104;
  /** @returns {number[][]} */
  const square = (scale) => {
    const s = scale / 2, corners = [[s, s], [-s, s], [-s, -s], [s, -s]], out = [];
    corners.forEach((v, i) => {
      const toward = (p, d) => [v[0] + (p[0] - v[0]) * d, v[1] + (p[1] - v[1]) * d];
      const prev = corners[(i + 3) % 4], next = corners[(i + 1) % 4];
      const a = toward(prev, CUT), b = toward(next, CUT), ha = toward(prev, CUT * (1 - EASE)), hb = toward(next, CUT * (1 - EASE));
      for (let j = 0; j <= 12; j++) {
        const t = j / 12, u = 1 - t, w = [u * u * u, 3 * u * u * t, 3 * u * t * t, t * t * t];
        out.push([0, 1].map((n) => w[0] * a[n] + w[1] * ha[n] + w[2] * hb[n] + w[3] * b[n]));
      }
    });
    return out;
  };
  const BOX = square(1), OPEN = square(OPENING);
  const seen = (points, height, yaw) => points.map(([x, z]) => [x * Math.cos(yaw) - z * Math.sin(yaw), -height * COS + (x * Math.sin(yaw) + z * Math.cos(yaw)) * SIN]);
  /** @returns {number[][]} */
  const hull = (points) => {
    const sorted = [...points].sort((p, q) => p[0] - q[0] || p[1] - q[1]), lower = [], upper = [];
    const turn = (o, a, b) => (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0]);
    for (const p of sorted) { while (lower.length > 1 && turn(lower.at(-2), lower.at(-1), p) <= 0) lower.pop(); lower.push(p); }
    for (const p of sorted.reverse()) { while (upper.length > 1 && turn(upper.at(-2), upper.at(-1), p) <= 0) upper.pop(); upper.push(p); }
    return [...lower.slice(0, -1), ...upper.slice(0, -1)];
  };

  const middle = -HALF_HEIGHT * COS, reach = (OPENING / 2) * Math.SQRT2 * SIN, top = middle - reach, width = 0.63 - RIM;
  const atRest = seen(OPEN, HALF_HEIGHT, REST);
  const tip = (above) => atRest.filter(([x, y]) => Math.abs(x) <= 0.14 && y < middle === above).map(([x, y]) => [x, y - (above ? top : middle + reach)]);
  const TOP = tip(true), FRONT = tip(false), sideTop = top + SIN * width, EMPTY = -2 * sideTop;
  const arc = (cx, cy, r, from, to) => Array.from({ length: 9 }, (_, i) => { const q = ((from + ((to - from) * i) / 8) * Math.PI) / 180; return [cx + r * Math.cos(q), cy + r * Math.sin(q)]; });
  const flat = (fill) => {
    const h = (1 - fill) * EMPTY, sideBottom = sideTop + h, bottom = sideBottom + SIN * width;
    let side;
    if (h < 2 * CORNER) {
      const r = (CORNER + h) / SQ3, x = (bottom - top - (2 * r) / C30) / (2 * SIN);
      side = arc(x, top + SIN * x + r / C30, r, -60, 60);
    } else {
      const r = SQ3 * CORNER;
      side = [...arc(width - r, top + SIN * (width - r) + r / C30, r, -60, 0), ...arc(width - r, sideBottom + SIN * r - r / C30, r, 0, 60)];
    }
    const right = [
      ...TOP.filter(([x]) => x >= 0).map(([x, y]) => [x, top + y]).sort((p, q) => p[0] - q[0]),
      ...side,
      ...FRONT.filter(([x]) => x > 0).map(([x, y]) => [x, bottom + y]).sort((p, q) => q[0] - p[0]),
    ];
    const tipBottom = FRONT.reduce((p, q) => (Math.abs(q[0]) < Math.abs(p[0]) ? q : p));
    return [...right, [0, bottom + tipBottom[1]], ...right.slice(1).map(([x, y]) => [-x, y]).reverse()];
  };
  const blend = (A, B, k) => {
    const prep = (poly, scale) => {
      const p = poly.map(([x, y]) => [x * scale, -y * scale]);
      if (p.reduce((sum, a, i) => sum + a[0] * p[(i + 1) % p.length][1] - p[(i + 1) % p.length][0] * a[1], 0) < 0) p.reverse();
      const low = p.reduce((best, a, i) => (a[1] < p[best][1] || (a[1] === p[best][1] && a[0] < p[best][0]) ? i : best), 0);
      return [...p.slice(low), ...p.slice(0, low)];
    };
    const P = prep(A, 1 - k), Q = prep(B, k), n = P.length, m = Q.length, out = [];
    for (let i = 0, j = 0; i < n || j < m; ) {
      out.push([P[i % n][0] + Q[j % m][0], -(P[i % n][1] + Q[j % m][1])]);
      const e = [P[(i + 1) % n][0] - P[i % n][0], P[(i + 1) % n][1] - P[i % n][1]], f = [Q[(j + 1) % m][0] - Q[j % m][0], Q[(j + 1) % m][1] - Q[j % m][1]];
      const cross = e[0] * f[1] - e[1] * f[0];
      if (j >= m || (i < n && cross >= 0)) i++;
      if (i > n || (j < m && cross <= 0)) j++;
    }
    return out;
  };
  const ring = (points) => `M${points.map(([x, y]) => `${x.toFixed(4)} ${y.toFixed(4)}`).join("L")}Z`;
  const draw = (fill, yaw, spinning = 0, opened = flat(fill)) => {
    const box = hull([...seen(BOX, HALF_HEIGHT, yaw), ...seen(BOX, -HALF_HEIGHT, yaw)]);
    const k = Math.max(spinning, Math.abs(Math.sin(2 * (yaw - REST)))), level = HALF_HEIGHT - (1 - fill) * 2 * HALF_HEIGHT;
    const opening = k < 1e-6 ? opened : blend(opened, hull([...seen(OPEN, HALF_HEIGHT, yaw), ...seen(OPEN, level, yaw)]), k);
    return ring(box) + ring(opening);
  };
  first.setAttribute("d", draw(1, REST));

  const WAIT = 0.5, MOVE = 1.6, LOOP = 2 * (WAIT + MOVE);
  const easeOut = (x) => 1 - (1 - x) ** 3;
  const fillAt = (t) => {
    t %= LOOP;
    if (t < WAIT) return 1;
    if (t < WAIT + MOVE) return 1 - easeOut((t - WAIT) / MOVE);
    if (t < 2 * WAIT + MOVE) return 0;
    return easeOut((t - 2 * WAIT - MOVE) / MOVE);
  };
  const FULL = 5, TURNS = 8, POWER = 1.6, QUARTER = Math.PI / 2, VMAX = 2 * Math.PI * TURNS;
  const along = (t) => Math.min(1, Math.max(0, t / FULL));
  const speedAt = (t) => VMAX * along(t) ** POWER, timeTo = (v) => FULL * Math.min(1, v / VMAX) ** (1 / POWER);
  const turnBy = (t) => (VMAX * FULL * along(t) ** (POWER + 1)) / (POWER + 1);
  // Turning fast, a blur: a fixed number of faint copies (so it never pulses), covering all its ways round alike from a quarter turn up: drawn on a fixed grid, a still cylinder.
  const COPIES = 24, EXPOSURE = 1 / 30;
  const paths = [first, ...Array.from({ length: COPIES - 1 }, () => svg.appendChild(/** @type {SVGPathElement} */ (first.cloneNode())))];
  const brand = /** @type {HTMLElement} */ (document.querySelector(".brand"));
  let pointedAt = /** @type {number | null} */ (null), angle = 0, speed = 0, last = 0, drawn = "";
  let landing = /** @type {{ back: boolean, from: number, to: number, h: number, stretch: number, speed: number, at: number, length: number } | null} */ (null);
  const rest = new Map();
  let looping = false, cycleFrom = performance.now();
  const wake = () => {
    if (looping || phone.matches || reduced()) return;
    looping = true;
    last = 0;
    requestAnimationFrame(frame);
  };
  const written = paths.map(() => ({ d: /** @type {string | null} */ (null), opacity: "" }));
  brand.addEventListener("pointerenter", (event) => {
    if (event.pointerType !== "mouse" || pointedAt !== null) return;
    if (!looping) cycleFrom = performance.now();
    pointedAt = performance.now() - 1000 * timeTo(speed);
    landing = null;
    wake();
  });
  brand.addEventListener("pointerleave", () => {
    if (pointedAt === null) return;
    pointedAt = null;
    const h = timeTo(speed), natural = turnBy(h), to = Math.ceil((angle + natural) / QUARTER - 1e-9) * QUARTER;
    const span = to - angle, stretch = span / Math.max(natural, 1e-9);
    if (span < 1e-6) return void (speed = 0);
    landing = { back: stretch <= 3, from: angle, to, h, stretch, speed, at: performance.now(), length: Math.min(0.7, (2.7 * span) / Math.max(speed, 1e-3)) };
  });
  const clock = motionClock();
  const frame = (now) => {
    now = clock(now);
    if (phone.matches || reduced()) {
      looping = false;
      paths.forEach((path, i) => {
        const d = i ? "" : (rest.get(240) ?? (rest.set(240, draw(1, REST)), rest.get(240)));
        if (written[i].d !== d) path.setAttribute("d", (written[i].d = d));
        if (!i && written[i].opacity !== "1") path.setAttribute("fill-opacity", (written[i].opacity = "1"));
      });
      return;
    }
    const dt = Math.min(0.05, (now - (last || now)) / 1000);
    last = now;
    if (pointedAt !== null) {
      speed = speedAt((now - pointedAt) / 1000);
      angle += speed * dt;
    } else if (landing) {
      const t = (now - landing.at) / 1000;
      let done;
      if (landing.back) {
        const back = Math.max(0, landing.h - t / landing.stretch);
        angle = landing.from + landing.stretch * (turnBy(landing.h) - turnBy(back));
        speed = speedAt(back);
        done = back <= 0;
      } else {
        const T = landing.length, s = Math.min(1, t / T);
        angle = (2 * s ** 3 - 3 * s ** 2 + 1) * landing.from + (s ** 3 - 2 * s ** 2 + s) * T * landing.speed + (-2 * s ** 3 + 3 * s ** 2) * landing.to;
        speed = ((6 * s ** 2 - 6 * s) * landing.from + (3 * s ** 2 - 4 * s + 1) * T * landing.speed + (-6 * s ** 2 + 6 * s) * landing.to) / T;
        done = s >= 1;
      }
      if (done) {
        angle = landing.to % (2 * Math.PI);
        speed = 0;
        landing = null;
      }
    }
    const cycle = (now - cycleFrom) / 1000, fill = fillAt(cycle), sweep = Math.min(QUARTER, speed * EXPOSURE), spinning = Math.min(1, speed / 2);
    const still = !speed && !landing && pointedAt === null, step = sweep / COPIES;
    if (!still || cycle < LOOP || cycle % LOOP >= WAIT) requestAnimationFrame(frame);
    else looping = false;
    const from = sweep >= QUARTER ? Math.floor(angle / step) * step : angle;
    const key = still ? `r${Math.round(fill * 240)}` : sweep < 0.005 ? `s${fill} ${angle}` : `b${fill} ${sweep >= QUARTER ? "" : from} ${sweep}`;
    if (key === drawn) return;
    drawn = key;
    let shapes;
    if (still) {
      const n = Math.round(fill * 240);
      if (!rest.has(n)) rest.set(n, draw(n / 240, REST));
      shapes = [rest.get(n)];
    } else if (sweep < 0.005) shapes = [draw(fill, REST + angle, spinning)];
    else {
      const opened = flat(fill);
      shapes = Array.from({ length: COPIES }, (_, j) => draw(fill, REST + from - j * step, spinning, opened));
    }
    const opacity = shapes.length === 1 ? "1" : String(1 - 0.005 ** (1 / COPIES));
    paths.forEach((path, i) => {
      const d = shapes[i] ?? "", seen = written[i];
      if (seen.d !== d) path.setAttribute("d", (seen.d = d));
      if (d && seen.opacity !== opacity) path.setAttribute("fill-opacity", (seen.opacity = opacity));
    });
  };
  phone.addEventListener("change", wake);
  whenReduced(wake);
  wake();
}
animateMark();

const chartFaces = [...document.fonts].filter((face) => face.family.replace(/"/g, "") === "Inter Chart");
const chartFont = (chartFaces.at(-1)?.load() ?? Promise.reject()).then(() => true, () => false);
let chartSans = css("--sans");
const CHART_TEXT = 12;
// Register the page's resize work before the chart's observer, so it joins the chart's next draw.
const resized = new ResizeObserver((entries) => {
  for (const entry of entries) if (entry.target === $("overview")) overviewBox = entry.contentRect;
  for (const entry of entries) if (entry.target === $("chart") && !sizeHeld) {
    chartHeight = entry.contentRect.height;
    if (!exporting) chartSize = { width: entry.contentRect.width, height: entry.contentRect.height };
  }
  for (const entry of entries) if (entry.target === $("chart")) placeFoot(entry.contentRect.height);
  relayout();
});
resized.observe($("chart"));
resized.observe($("navigator"));
resized.observe($("overview"));
const engine = createChart($("chart").appendChild(Object.assign(document.createElement("div"), { style: `width: ${$("chart").clientWidth}px; height: ${$("chart").clientHeight}px` })));
engine.theme = chartTheme();
const RIGHT_AXIS = engine.axis;
engine.time.minSpacing = MIN_BAR_SPACING;
engine.crosshair.setFormat((time) => pointLabel(time));
let chartSize = { width: $("chart").clientWidth, height: $("chart").clientHeight };
fillBox();
const sizeChart = (/** @type {number} */ width, /** @type {number} */ height) => ((chartSize = { width, height }), (Object.assign(engine.element.style, { width: `${width}px`, height: `${height}px` }), engine.layout()), !sizeHeld && !exporting && fillBox());
/** A layout's motion (the sidebar's, the panel's, full screen's, a peek's): the chart sized once for it, not at each of its frames (each a new size: its canvases made anew, every series drawn again; Firefox shows a third of them). It takes its size at the motion's end at once, pinned to the edges of its box that move least, the moving edges sliding over it or off it. Its box narrowing or widening, it keeps the view's stretch of time: drawn once at its end's width, its plot stretched to its box's width each frame (see stretchDrawn). */
let motionsSized = 0, sizeHeld = false;
/** @type {string | null} */
let stretchKept = null;
function sizeOnce() {
  const box = $("chart"), inMotion = /** @type {CSSTransition[]} */ (document.querySelector(".app")?.getAnimations({ subtree: true }).filter((motion) => motion instanceof CSSTransition && /^(width|height|margin|padding|border|inset|left|right|top|bottom)/.test(motion.transitionProperty)) ?? []);
  if (!inMotion.length) {
    const { width, height } = box.getBoundingClientRect();
    if (sizeHeld || Math.abs(width - chartWidth()) < 1 || !resolved?.times.length || !chartShown()) return;
    sizeKeeping(width, height);
    fillBox();
    return;
  }
  const run = ++motionsSized, at = inMotion.map((motion) => motion.currentTime), now = box.getBoundingClientRect(), own = engine.element.getBoundingClientRect();
  // (Its box at their end: each motion that moves it there for a moment, read, and back where it was; nothing's drawn
  // between. Those inside the sidebar or the panel (their rows' own, many) don't: only the boxes beside the chart's.)
  // (A hair before their end, not at it: WebKit takes a transition sought to its very end as done, and starts a copy
  // of it beside the first, whose second motion lays the chart out again once the first lands. WebKit bug.)
  const moving = inMotion.filter((motion) => ((target) => !target || !target.parentElement?.closest(".sidebar, #panel"))(/** @type {KeyframeEffect | null} */ (motion.effect)?.target));
  for (const motion of moving) motion.currentTime = Number(/** @type {AnimationEffect} */ (motion.effect).getComputedTiming().endTime) - 0.01;
  const end = box.getBoundingClientRect();
  moving.forEach((motion) => (motion.currentTime = at[inMotion.indexOf(motion)]));
  const across = Math.abs(end.width - now.width) >= 1 && Boolean(resolved?.times.length) && chartShown();
  const leftAxis = engine.left > 0 && !(engine.axisShown > 0);
  // (Only its width changing (the sidebar's, the panel's): the drawing as it is, stretched from the first frame,
  // nothing held back, nothing drawn (a draw waiting, the live price's, flushed now, would hold the motion's first
  // frame as long as a heavy chart takes); drawn at its end's width once the motion's over (releaseSize). Its height
  // too (full screen, a phone's sheet): drawn at its end's size first, its scales fitted to it at once: one motion,
  // nothing gliding after it.)
  const later = across && Math.abs(end.height - chartSize.height) < 1;
  const was = across && !later ? (engine.render(true), shownSpan()) : null;
  stretchDrawn(null);
  const width = later ? chartSize.width : end.width, height = end.height;
  stretchKept = later ? (chart?.id ?? null) : null;
  const side = across ? (leftAxis ? "left" : "right") : Math.abs(end.left - now.left) <= Math.abs(end.right - now.right) ? "left" : "right", edge = Math.abs(end.top - now.top) <= Math.abs(end.bottom - now.bottom) ? "top" : "bottom";
  sizeHeld = true;
  footPinned = edge === "top";
  chartHeight = height;
  Object.assign(engine.element.style, { position: "absolute", left: "", right: "", top: "", bottom: "", [side]: "0", [edge]: "0", width: `${width}px`, height: `${height}px`, overflow: "visible" });
  if (!later && (width !== own.width || height !== own.height)) {
    // (Drawn at its new size now, the motions held at their start meanwhile: they start on the next frame, from
    // their start, not a long frame in.)
    for (const motion of inMotion) motion.pause();
    sizeChart(width, height);
    fitMargins();
    if (across) keepStretch(was);
    placeFoot(now.height);
    // (Those another toggle has replaced meanwhile left be: started again, they'd come back.)
    // (Which are still paused, all read first, then each started: a read between writes would lay the page out again
    // for each of them, hundreds with a long Library's rows.)
    requestAnimationFrame((time) => inMotion.filter((motion) => motion.playState === "paused").forEach((motion) => (motion.startTime = time - Number(at[inMotion.indexOf(motion)] ?? 0))));
  }
  if (across) stretchAcross(run, now.width, width);
  Promise.allSettled(inMotion.map((motion) => motion.finished)).then(() => {
    if (run !== motionsSized) return;
    if (later) return whenIdle(() => run === motionsSized && releaseSize(), { timeout: 200 });
    releaseSize();
  });
}
function holdChart() {
  if (sizeHeld || !resolved?.times.length || !chartShown()) return;
  const run = ++motionsSized, leftAxis = engine.left > 0 && !(engine.axisShown > 0);
  stretchKept = chart.id;
  sizeHeld = true;
  chartHeight = chartSize.height;
  Object.assign(engine.element.style, { position: "absolute", left: "", right: "", top: "0", bottom: "", [leftAxis ? "left" : "right"]: "0", width: `${chartSize.width}px`, height: `${chartSize.height}px` });
  stretchAcross(run, $("chart").getBoundingClientRect().width, chartSize.width);
}
let footPinned = false;
/** @param {{ from: number, to: number } | null} was */
function keepStretch(was) {
  if (!resolved || !chartShown()) return;
  fitSpacing(chart, resolved.times.length, was ?? undefined);
  if (was) showSpan(was, EXACT_RANGE);
  engine.render(true);
  const range = shownSpan();
  lastBox = chartWidth();
  (heldView = range ? { box: lastBox, at: range } : null), (madeView = null);
}
function releaseSize() {
  if (!sizeHeld) return;
  sizeHeld = false;
  stretchDrawn(null);
  placeFoot(0);
  Object.assign(engine.element.style, { position: "", left: "", right: "", top: "", bottom: "", width: "100%", height: "100%", overflow: "" });
  const { width, height } = $("chart").getBoundingClientRect();
  if (stretchKept && stretchKept === chart?.id && chartShown()) {
    sizeKeeping(width, height);
  } else if (width !== chartSize.width || height !== chartSize.height) sizeChart(width, height);
  stretchKept = null;
  chartHeight = $("chart").getBoundingClientRect().height;
  relayout();
}
{
  const laidOut = new MutationObserver(sizeOnce);
  laidOut.observe(document.body, { attributes: true, attributeFilter: ["data-sidebar", "data-panel", "data-peek", "data-sheet", "data-keys"] });
  laidOut.observe(/** @type {HTMLElement} */ (document.querySelector(".app")), { attributes: true, attributeFilter: ["data-full"] });
}
addEventListener("resize", () => {
  releaseSize();
  // A turn or a breakpoint lands at once: moving the card's margins and padding for it would resize the chart each
  // frame, nothing sizing it once. Those already started (WebKit and Chromium restyle a turned page before this event)
  // finish now; none start until the frame after next.
  const card = /** @type {HTMLElement} */ (document.querySelector(".card"));
  for (const motion of card.getAnimations({ subtree: true })) {
    if (motion instanceof CSSTransition && /^(margin|padding)-/.test(motion.transitionProperty)) motion.finish();
  }
  card.style.setProperty("--card-motion", "0s");
  requestAnimationFrame(() => requestAnimationFrame(() => card.style.removeProperty("--card-motion")));
});
// Once a face is in, the chart's words in it are measured again: the axes' kept widths are by the font's.
const chartFontIn = (/** @type {boolean} */ loaded) => {
  if (!loaded) return;
  chartSans = css("--chart-sans");
  navigatorFont = "";
  engine.theme = chartTheme();
  engine.invalidate(true);
  drawOverview();
};
chartFont.then(chartFontIn);
withRest(() => void chartFaces[0]?.load().then(() => chartFontIn(true), () => {}));
let drawnOrder = "";
let spotlit = null;
function spotlight(id) {
  if (id === spotlit) return;
  spotlit = id;
  const lighted = id && drawn.has(id) && drawn.size > 1 && !layerById(id)?.hidden ? id : null;
  const pane = lighted ? layerById(lighted)?.pane : undefined, among = (/** @type {string} */ layerId) => Boolean(lighted) && layerId !== lighted && layerById(layerId)?.pane === pane;
  for (const layerId of drawn.keys()) {
    const faint = among(layerId);
    if (fadedNow.has(layerId) === faint) continue;
    faint ? fadedNow.add(layerId) : fadedNow.delete(layerId);
    fadeLayer(layerId, faint);
  }
  let levelsFaded = false;
  for (const tag of levelTags) {
    const faint = Boolean(lighted) && !tag.layers.has(/** @type {string} */ (lighted)) && [...tag.layers].some((layerId) => layerById(layerId)?.pane === pane);
    if (Boolean(tag.line.faint) !== faint) (tag.line.faint = faint), (levelsFaded = true);
  }
  if (levelsFaded) for (const { levels } of levelsKept.values()) levels.changed();
  engine.invalidate(true);
}
let pointedLayer = /** @type {string | null} */ (null), pointerOnChart = false;
document.addEventListener("pointerover", (event) => {
  if (document.body.classList.contains("carrying")) return;
  pointedLayer = event.pointerType === "touch" ? null : (closest(event, ".layer-row")?.dataset.layer ?? closest(event, ".legend [data-select]")?.dataset.select ?? null);
  pointerOnChart = event.pointerType !== "touch" && !pointedLayer && Boolean(closest(event, "#chart"));
  respot();
});
document.addEventListener("pointerout", (event) => event.relatedTarget || ((pointedLayer = null), (pointerOnChart = false), respot()));
/** @type {Resolved | null} */
let resolved = null;
let dataRevision = 0;
let drawVersion = 0;
let asking = { key: "", look: "", index: "" };
/** @type {{ doc: Chart, look: string } | null} */
let displayed = null;
const chartShown = () => Boolean(chart && displayed && displayed.doc.id === chart.id);
const pending = () => drawVersion > (resolved?.version ?? 0);
let buildAsked = 0, buildRefit = false;

/** Layers enough that a chart's build takes more than a frame or two on a phone without its JIT (six daily series, some 36,000 points: ~100 ms): see draw. */
const MANY_LAYERS = 6;
/** @returns {Promise<boolean>} */
async function draw(refit = false, fresh = false) {
  if (buildAsked) {
    if (buildAsked > 0) cancelAnimationFrame(buildAsked);
    (buildAsked = 0), (refit ||= buildRefit), (buildRefit = false);
  }
  const version = ++drawVersion;
  undrawn = false;
  const fromHome = document.body.hasAttribute("data-home");
  const doc = structuredClone(chart), was = displayed?.doc, askedIndex = asking.index;
  asking = { key: dataKey(doc), look: lookKey(doc), index: doc.index };
  const first = was?.id !== doc.id;
  // (Grown out of its card, a chart of many layers is built once it's grown, as one whose data loads is: its build
  // mid-motion (23 bands' on a phone without its JIT, a third of a second) held the motion still. Fewer build in a
  // frame or two, drawn as they grow.)
  const later = first && fromHome && doc.layers.filter((layer) => !layer.hidden).length >= MANY_LAYERS;
  if (first || (was?.index !== doc.index && askedIndex !== doc.index)) {
    if (!first || viewKey(doc.id) === LINKED || openView(doc)?.index !== doc.index) spans.delete(spanKey(doc.id, doc.index));
    spanned.clear();
  }
  const unasked = drawSeries(doc).filter((name) => !asked(name, doc) && intervalsOf(name)?.includes(doc.index) !== false).length;
  const showLoading = unasked > 0 && (first || !resolved);
  $("progress").hidden = !(unasked > 0 && !showLoading);
  if (showLoading) {
    $("loading-text").textContent = `Loading ${grouped.format(unasked)} series`;
    $("loading").hidden = false;
    $("navigator").classList.add("waiting");
  }
  $("chart").classList.toggle("waiting", (showLoading || later) && first);
  if (showLoading) $("unreached").hidden = true;
  const live = !FINE.has(doc.index) || spanOf(doc)?.live !== false;
  const rest = doc.layers.some((layer) => heatLike(layer) || layer.type === "bubbles") ? loadRest() : undefined;
  if (doc.layers.some((layer) => /\b(over|under)\(/.test(layer.formula ?? ""))) await import("/studio/client.js");
  const result = await resolve(doc, fresh || stale(doc));
  await rest;
  if (showLoading || later) await Promise.all((grid?.morphing ?? []).map((animation) => animation.finished.catch(() => {})));
  if (version !== drawVersion) return false;
  if ((showLoading || later) && home) {
    $("loading").hidden = true;
    $("progress").hidden = true;
    $("navigator").classList.remove("waiting");
    undrawn = true;
    return false;
  }
  $("loading").hidden = true;
  $("progress").hidden = true;
  $("navigator").classList.remove("waiting");
  $("chart").classList.remove("waiting");
  if (displayed?.doc.id === doc.id && displayed.doc.index !== doc.index && openView(doc)?.index === displayed.doc.index) storeView(true);
  const unreached = result.failed.length > 0 && ![...result.results.values()].some((item) => item?.values);
  if (unreached) sayUnreached();
  $("unreached").hidden = !unreached;
  $("navigator").classList.toggle("waiting", unreached);
  dataRevision++;
  const reveal = first && doc.layers.length > 0 && !reduced() && !fromHome;
  build(withLook(structuredClone(chart)), Object.assign(result, { key: asking.key, version, live }), refit);
  for (const wake of [...drawnWaiters]) wake();
  requestAnimationFrame(() => setTimeout(askRest));
  withRest(layersPanels);
  updateStatus();
  describePlot();
  if (reveal) {
    const chartElement = $("chart");
    chartElement.classList.remove("reveal");
    void chartElement.offsetWidth;
    chartElement.classList.toggle("quick", revealed);
    chartElement.classList.add("reveal");
    revealed = true;
    chartElement.addEventListener("animationend", () => chartElement.classList.remove("reveal"), { once: true });
  }
  return true;
}
function layersPanels() {
  renderLayers();
  inspector.renderInspector();
}
const GRADED = new Set(["line", "area", "trend", "dots", "histogram"]);
function buildChart(doc, data, refit) {
  if (buildAsked) {
    if (buildAsked > 0) cancelAnimationFrame(buildAsked);
    (buildAsked = 0), (refit ||= buildRefit), (buildRefit = false);
    queueMicrotask(() => { for (const wake of [...drawnWaiters]) wake(); });
  }
  const prior = displayed, priorTimes = resolved?.times ?? [], same = prior?.doc.id === doc.id && prior?.doc.index === doc.index, sameView = !refit && same;
  const owed = prior?.doc.id === doc.id && prior?.doc.range !== doc.range;
  (displayed = { doc, look: lookKey(doc) }), (resolved = data);
  if (!same) crossSet ? dropCross() : keyAway();
  const visible = sameView ? shownDates() : null;
  const logical = sameView ? shownSpan() : null;
  const showedAll = visible && priorTimes.length > 0 && Number(visible.from) <= priorTimes[0] && Number(visible.to) >= /** @type {number} */ (priorTimes.at(-1));
  const sameTimeline = priorTimes.length > 0 && data.times.length === priorTimes.length && data.times[0] === priorTimes[0] && data.times.at(-1) === priorTimes.at(-1);
  const shift = sameView && !sameTimeline && FINE.has(doc.index) && priorTimes.length && data.times.length ? Math.round((priorTimes[0] - data.times[0]) / FINE_STEP[doc.index]) : 0;
  // Price scales stay as the person left them (a range set by hand, in any mode) while the panes and their modes
  // are the same: new data, a restyle. Otherwise they fit their data again.
  const shapeOf = (/** @type {Chart} */ of) => of.panes.map((pane) => `${pane.id}:${pane.mode}:${pane.stack ?? ""}:${pane.reverse ? 1 : ""}:${pane.average ?? ""}`).join();
  const panesShape = shapeOf(doc), priorShape = prior ? shapeOf(prior.doc) : "";
  const keep = sameView && panesShape === priorShape;
  const paneIds = (/** @type {string} */ shape) => shape.split(",").map((pane) => pane.split(":")[0]).join();
  if (prior?.doc.id === doc.id && paneIds(priorShape) !== paneIds(panesShape) && !reduced() && !document.body.hasAttribute("data-picture")) fadeOut($("chart"), [{ opacity: 1 }, { opacity: 0 }], MOTION.panel, true, MOTION.ease);
  const glidingTo = engine.controller.target;
  engine.controller.stop();
  const was = new Map(drawn);
  drawn.clear();
  dotted.clear();
  const wasPainted = new Map(paintedLayers);
  paintedLayers.clear();
  const relit = spotlit;
  spotlit = null;
  for (const layerId of fadedNow) fadeLayer(layerId, false, was);
  fadedNow.clear();
  levelTags.length = 0;
  showTimes(data.times);
  const paneIndex = new Map(doc.panes.map((pane, i) => [pane.id, i]));
  while (engine.panes.length < doc.panes.length) engine.addPane(PANE);
  const stacked = stackOf(doc, data), shares = new Set(doc.panes.filter((pane) => pane.stack === "percent").map((pane) => pane.id));
  const foldedLayers = foldedInto(doc);
  // Each layer drawn as before (its kind, place, scale and stacking the same) keeps its series, its look applied to
  // it; its data is set again only where it changed (its values new), so an edit to one layer redraws that layer
  // alone, and a color or a width scrubbed stays smooth however many points there are. (A histogram's set again
  // whole: its bars under its base in its second color, a palette made with its data.)
  // (Not where it is among them: a layer moved, or its pane reversed, keeps its series, put in order below.)
  const layerShape = (/** @type {Layer} */ layer) => JSON.stringify([layer.type, layer.type === "signal" && layer.place === "shade", layer.type === "bubbles" && !layer.ref, fromUnfit(doc, layer), paneIndex.get(layer.pane), layer.side, layer.fill ?? "", baseOf(layer), modeOf(doc, layer), Boolean(layer.type === "candles" && data.results.get(layer.id)?.opens), stacked.has(layer.id), shares.has(layer.pane), edgesOf(layer).length > 1 && !cloudLayers(layer).length, layer.type === "cloud" && Boolean(layer.byLayers)]);
  /** @type {Map<string, { shape: string, shown?: Vector, opens?: Vector, highs?: Vector, lows?: Vector, stacked?: boolean, painted?: any[] | null, low?: Vector }>} */
  const kept = new Map();
  for (const layer of doc.layers) {
    const before = built.get(layer.id);
    if (before && before.shape === layerShape(layer) && was.has(layer.id)) kept.set(layer.id, before);
  }
  built.clear();
  const old = [...was].filter(([id]) => !kept.has(id)).map(([, series]) => series);
  let added = false;
  /** @type {{ layer: Layer, value: number, color: string, dash: number, tag: boolean, mode: number, pane: number, format: any }[]} */
  const across = [];
  const linesAcross = (layer, pane, format, op) => {
    if (acrossUnfit(doc, layer)) return;
    const values = [...(layer.levels ?? []).map((level, i) => ({ key: String(i), value: level.value })), ...(layer.type === "baseline" ? [{ key: "base", value: baseOf(layer) }] : [])];
    for (const { key, value } of values) {
      const { color, dash, tag, line } = lineLook(layer, key);
      if (line === false) continue;
      across.push({ layer, mode: modeOf(doc, layer), pane, format, value, color: op < 1 ? alpha(ink(color), op) : ink(color), dash, tag: Boolean(tag) });
    }
  };
  for (const layer of doc.layers) {
    const result = data.results.get(layer.id);
    const op = opacityOf(layer), base = ink(layer.color), under = ink(belowOf(layer));
    const color = op < 1 ? alpha(base, op) : base, below = op < 1 ? alpha(under, op) : under;
    const stack = stacked.get(layer.id);
    const stackedLine = layer.type === "line" && Boolean(stack);
    const zoned = layer.byLevels && layer.levels?.length && (layer.type === "line" || layer.type === "area") && !stack ? levelSteps(layer) : null;
    const graded = layer.byGradient && layer.gradient && !layer.upToDate && !stack && !layer.colorFrom && !zoned && GRADED.has(layer.type) && result?.values ? layer.gradient : null;
    const signal = layer.type === "signal";
    const format = shares.has(layer.pane) ? SHARE_FORMAT : valueFormatOf([unitFor(layer, result)]);
    layerFormats.set(layer.id, format.value);
    const lineWidth = /** @type {any} */ (layer.width ?? (stack ? STACK_WIDTH : undefined));
    const shown = !layer.hidden && !signal;
    const tags = { hidden: !shown, tag: shown && !foldedLayers.has(layer.id), priceLine: Boolean(layer.priceLine) && !stack && !signal, format: format.value, outline: Boolean(layer.outline) && OUTLINED.has(layer.type) ? css("--card") : null };
    const partial = periodAmount(layer);
    const stroke = { width: lineWidth ?? 1.5, dash: dashFor(lineWidth, layer.dash), steps: lineTypeOf(layer) === LINE_TYPE.WithSteps, curve: lineTypeOf(layer) === LINE_TYPE.Curved, marker: MARKER.radius, stroke: true, dots: 0, partial };
    const pane = paneIndex.get(layer.pane) ?? 0;
    const candles = layer.type === "candles" && result?.opens;
    const banded = layer.type === "cloud" && Boolean(layer.byLayers && result?.members?.length && result.memberIds);
    const fromSource = candles || banded || zoned || graded ? null : paintOf(doc, layer, data);
    const dated = fromSource && ((/** @type {number} */ i) => ((name) => (name ? ink(name) : base))(fromSource(i)));
    const shadeDown = signal && layer.place === "shade" ? alpha(ink(belowOf(layer)), 0.14 * op) : "";
    const painting = dated ? paintSig(doc, layer, data, op, base) : shadeDown ? [shadeDown] : null;
    // (What one color of its own paints, once for all its colors (its own, its zones', its gradient's, each date's): a
    // line's, dots' or a trend's at its opacity; an area's line, fill and fade (a stacked line's fill none); bars' at
    // three quarters. An area filled above its line is evenly tinted: the fill's gradient runs by price, so above a line
    // that runs high it would all but vanish. Stacked, its band (from its line to the one under it, see stackBands)
    // evenly too: a band fading by price would read lighter the lower it sits.)
    const fills = stackedLine ? [0, 0] : fillOf(layer, Boolean(stack));
    const looks = (/** @type {string} */ c) => /** @type {Record<string, string>} */ (layer.type === "area" || stackedLine ? { line: op < 1 ? alpha(c, op) : c, fill: alpha(c, fills[0] * op), fade: alpha(c, fills[1] * op) }
      : layer.type === "histogram" ? { color: alpha(c, 0.75 * op) } : { color: op < 1 ? alpha(c, op) : c });
    /** @param {Record<string, string>} look */
    const lineOf = (look) => look.line ?? look.color;
    const lookTable = graded ? gradientTable(graded).map(looks) : null;
    const [lo, hi] = lookTable && result?.values ? rangeOf(result.values) : [0, 0];
    /** @type {Record<string, any>} */
    const paint = layer.type === "cloud" ? { color: alpha(base, op), bands: banded ? /** @type {string[]} */ (result?.memberIds).map((id) => alpha(ink(doc.layers.find((item) => item.id === id)?.color ?? layer.color), op)) : [] } : signal && layer.place === "shade" ? { color: alpha(base, 0.14 * op) } : candles ? ((up, down) => ({ up: hollowOf(layer) ? css("--card") : up, down, upWick: up, downWick: down, upBorder: up, downBorder: down }))(color, layer.colors === 1 ? color : below)
      : layer.type === "baseline" ? { line: color, under: below, fill: alpha(base, 0.24 * op), fade: alpha(base, 0.01 * op), below: alpha(under, 0.01 * op), belowFade: alpha(under, 0.24 * op) }
      : looks(base);
    let series;
    const again = kept.has(layer.id) ? was.get(layer.id) : undefined;
    if (!again) added = true;
    const fitting = stack && shares.has(layer.pane) ? "shares" : zeroFit(doc, layer, modeOf(doc, layer), Boolean(stack)) ? "zero" : null;
    const standard = (/** @type {"line" | "band" | "bars" | "candles"} */ kind, /** @type {Standard | undefined} */ kept) => fitSeries(kept ?? new Standard(kind, pane, layer.side), fitting, unfitted(doc, layer));
    if (signal && layer.place === "shade") {
      series = again ?? new Shading(pane, layer.side);
      series.engine.hidden = !signalShown(layer, doc);
      series.style(paint.color);
    } else if (layer.type === "cloud") {
      const [top, bottom] = EDGES.map((edge) => Boolean(layer.from?.includes(edge)));
      series = again ?? new Cloud(pane, layer.side, top && bottom && !cloudLayers(layer).length);
      series.engine.hidden = !shown;
      fitSeries(series, fitting, unfitted(doc, layer));
      series.style({ color: paint.color, top, bottom, banded, bands: paint.bands });
    } else if (heatLike(layer)) {
      const look = /** @type {HeatLook} */ (heatmaps.heatLook(doc, layer, data, op)), was = heatmaps.heatLooks.get(layer.id), known = heatKnown(layer);
      if (was && was.cohort === look.cohort && was.weight === look.weight && was.grid?.agg === look.grid?.agg && was.measure === look.measure) look.reference = was.reference;
      if (was?.kind === "profile" && look.kind === "profile" && was.weight === look.weight && was.grid?.agg === look.grid?.agg) look.last = was.last;
      if (known) heatmaps.heatLooks.set(layer.id, look);
      else heatmaps.heatLooks.delete(layer.id), heatmaps.heatWants.delete(layer.id);
      const log = modeOf(doc, layer) === 1, drawable = (/** @type {number} */ value) => Number.isFinite(value) && (!log || value > 0);
      const alone = !doc.layers.some((other) => other !== layer && other.pane === layer.pane && other.side === layer.side && !other.hidden && !heatLike(other) && other.type !== "signal" && !unfitted(doc, other) && data.results.get(other.id)?.values?.some(drawable));
      if (layer.hidden) heatmaps.heatWants.delete(layer.id);
      series = again ?? new Heat(pane, layer.side, layer.id, layer.type === "profile");
      series.engine.hidden = !shown;
      series.alone = alone;
      series.redraw();
      if (layer.type === "profile") heatmaps.profileShares(layer.id, series);
      if (known) heatmaps.heatSeries.add(series);
      else heatmaps.heatSeries.delete(series);
    } else if (layer.type === "bubbles") {
      const on = doc.layers.some((other) => other.id === layer.ref);
      series = again ?? new Bubbles(pane, layer.side, !on);
      series.engine.hidden = !shown;
      series.style({ up: alpha(base, 0.4 * op), upEdge: alpha(base, 0.8 * op), down: alpha(under, 0.4 * op), downEdge: alpha(under, 0.8 * op), size: layer.size ?? BUBBLE_SIZE, hide: layer.hide ?? BUBBLE_HIDE, scaleTo: layer.scaleTo === "all" ? "all" : "view", halo: layer.outline ? css("--card") : null });
    } else if (candles) {
      series = standard("candles", again);
      styleSeries(series, { ...tags, candles: { ...paint, color, width: stroke.width, radius: BAR_CORNER } });
    } else {
      const radius = dotRadius(layer.width), base = baseOf(layer);
      const zones = (/** @type {(look: Record<string, string>) => Record<string, string>} */ part) => (zoned ? zoned.map((step) => ({ value: step.value, ...part(looks(ink(step.color))) })) : null);
      const gradient = (/** @type {(look: Record<string, string>) => string} */ part) => (lookTable ? new Gradient(lookTable.map(part), lo, hi) : null);
      const lineZones = zones((look) => ({ color: lineOf(look) })), lineGradient = gradient(lineOf);
      const [kind, look] = layer.type === "dots" ? ["line", { line: { color: paint.color, width: 1.5, dash: [], steps: false, curve: false, marker: MARKER.radius, stroke: false, dots: radius, zones: null, gradient: lineGradient, partial: false } }]
        : layer.type === "area" || stackedLine ? ["band", { line: { ...stroke, color: paint.line, zones: lineZones, gradient: lineGradient }, band: { gradient: gradient((look) => look.fill), gradientFade: gradient((look) => look.fade), color: paint.fill, fade: paint.fade, below: null, belowFade: null, base: 0, to: layer.fill === "above" && !stackedLine ? "top" : "foot", steps: stroke.steps, curve: stroke.curve, zones: zones((look) => ({ color: look.fill, fade: look.fade })) } }]
        : layer.type === "histogram" ? ["bars", { bars: { color: paint.color, base, radius: stack ? 0 : BAR_CORNER, partial, gradient: lineGradient } }]
        : layer.type === "baseline" ? ["band", { line: { ...stroke, color: paint.under, zones: [{ value: base, color: paint.line }], gradient: null }, band: { gradient: null, gradientFade: null, color: paint.fill, fade: paint.fade, below: paint.below, belowFade: paint.belowFade, base, to: "base", steps: stroke.steps, curve: stroke.curve, zones: null } }]
        : ["line", { line: { ...stroke, color: paint.color, zones: lineZones, gradient: lineGradient } }];
      series = standard(/** @type {"line" | "band" | "bars"} */ (kind), again);
      styleSeries(series, { ...tags, ...look });
      if (layer.type === "dots") dotted.set(layer.id, { series, width: layer.width, radius });
    }
    built.set(layer.id, { shape: layerShape(layer), shown: stack?.values ?? result?.values, stacked: Boolean(stack), painted: painting, ...(result?.opens ? { opens: result.opens, highs: result.highs, lows: result.lows } : {}), ...(result?.low ? { low: result.low } : {}) });
    const before = kept.get(layer.id);
    if (heatLike(layer)) series.set(heatKnown(layer) ? columnBy(data.times.length, (i) => result?.closes?.[i] ?? NaN) : EMPTY);
    else if (layer.type === "bubbles") result?.values ? series.set(...heatmaps.bubbleColumns(doc, data, layer, stacked)) : series.set(EMPTY, EMPTY);
    else if (!result?.values && again && before?.shown) {
      series instanceof Standard ? setColumn(series, EMPTY, false) : series instanceof Shading ? series.set(EMPTY) : series.set({ low: EMPTY, high: EMPTY });
      if (series.engine instanceof Candles) series.engine.setValues({ open: EMPTY, high: EMPTY, low: EMPTY, close: EMPTY });
    } else if (result?.values && again && (painting ? samePaint(before?.painted, painting) && wasPainted.has(layer.id) : !before?.painted) && before?.shown === (stack?.values ?? result.values) && before?.opens === result.opens && layer.type !== "histogram" && before?.low === result.low) {
      const carried = painting ? wasPainted.get(layer.id) : undefined;
      if (carried) paintedLayers.set(layer.id, carried);
      linesAcross(layer, pane, format, op);
    }
    else if (result?.values && signal && layer.place === "shade") {
      const shown = result.values, n = data.times.length, values = new Float64Array(n), colors = new Uint16Array(n);
      for (let i = 0; i < n; i++) (values[i] = Number.isFinite(shown[i]) && shown[i] !== 0 ? 0 : NaN), (colors[i] = shown[i] < 0 ? 1 : 0);
      series.set(values, colors, [paint.color, shadeDown]);
    } else if (result?.values && layer.type === "cloud") {
      const log = modeOf(doc, layer) === 1, low = /** @type {Vector} */ (result.low), high = result.values, whole = edgesOf(layer).length > 1 && !cloudLayers(layer).length, n = fromUnfit(doc, layer) ? 0 : data.times.length;
      const lows = new Float64Array(n).fill(NaN), highs = new Float64Array(n).fill(NaN), members = banded ? (result.members ?? []).map((values) => columnBy(n, (i) => (log && !(values[i] > 0) ? NaN : values[i]))) : [];
      for (let i = 0; i < n; i++) {
        if (whole) (lows[i] = 0), (highs[i] = 0);
        else if (banded) {
          let count = 0, lo = Infinity, hi = -Infinity;
          for (const values of members) if (Number.isFinite(values[i])) count++, (lo = Math.min(lo, values[i])), (hi = Math.max(hi, values[i]));
          if (count && count + edgesOf(layer).length >= 2) (lows[i] = lo), (highs[i] = hi);
        } else if (Number.isFinite(low[i]) && Number.isFinite(high[i]) && (!log || low[i] > 0)) (lows[i] = low[i]), (highs[i] = high[i]);
      }
      series.set({ low: lows, high: highs, sides: banded || whole ? null : result.sides ?? null, members });
      const painted = dated && n ? { series, ...paletteOf(n, dated), look: (/** @type {string} */ c) => ({ color: alpha(c, op) }) } : null;
      if (painted) paintedLayers.set(layer.id, painted);
      paintSeries(series, painted);
    } else if (result?.values) {
      const shown = stack?.values ?? result.values;
      const at = (/** @type {number} */ value, /** @type {number} */ i) => (stack ? stack.foot[i] + value * (stack.scale?.[i] ?? 1) : value);
      const log = modeOf(doc, layer) === 1;
      const look = !dated || layer.type === "baseline" ? null : looks;
      const times = layer.from && fromUnfit(doc, layer) ? [] : data.times;
      const drawable = (/** @type {number} */ value) => (value - value === 0 && (!log || value > 0) ? value : NaN);
      const values = times.length ? (log || shown.some((/** @type {number} */ value) => value === Infinity || value === -Infinity) ? shown.map(drawable) : shown) : [];
      if (candles) series.engine.setValues(candleColumns(result, shown, log, at));
      else setColumn(series, values, Boolean(stack) && (stackedLine || (layer.type === "area" && layer.fill !== "above")));
      const traced = candles ? null : series;
      (traced?.edge ?? traced?.engine)?.setStyle({ breaks: result.breaks ?? null });
      const tint = alpha(under, 0.75 * op), from = baseOf(layer), belowBase = !look && layer.type === "histogram" && layer.colors !== 1 && values.some((/** @type {number} */ value) => value < from);
      const colored = look && dated && !candles ? paletteOf(times.length, dated) : belowBase ? paletteOf(times.length, (i) => (values[i] < from ? tint : paint.color)) : null;
      const painted = colored && { series, ...colored, look: look ?? ((/** @type {string} */ c) => ({ color: c })) };
      if (painted) paintedLayers.set(layer.id, painted);
      if (!candles) paintSeries(series, painted);
      linesAcross(layer, pane, format, op);
    }
    drawn.set(layer.id, series);
  }
  stackBands(doc, stacked);
  drawRows(doc, data);
  const lines = new Map();
  for (const line of across) {
    if (!Number.isFinite(line.value) || (line.mode === 1 && line.value <= 0)) continue;
    const key = `${line.pane}|${line.layer.side}|${line.value}`;
    (lines.get(key) ?? lines.set(key, []).get(key))?.push(line);
  }
  const levelsBefore = new Map(levelsKept), levelLines = new Map();
  levelsKept.clear();
  for (const [first, ...rest] of lines.values()) {
    const group = [first, ...rest], key = `${first.pane}|${first.layer.side}`, shared = group.some((line) => line.layer !== first.layer), shown = group.some((line) => !line.layer.hidden);
    const line = { value: first.value, color: shared ? ink("gray") : first.color, width: 1, dash: dashFor(1, first.dash), tag: shown && group.some((line) => line.tag), hidden: !shown };
    (levelLines.get(key) ?? levelLines.set(key, { first, lines: [] }).get(key)).lines.push(line);
    if (shown) levelTags.push({ line, layers: new Set(group.map((item) => item.layer.id)) });
  }
  for (const [key, { first, lines: list }] of levelLines) {
    const pane = engine.panes[first.pane], was = levelsBefore.get(key);
    if (!pane) continue;
    levelsBefore.delete(key);
    if (was && was.pane !== pane) was.pane.remove(was.levels);
    const levels = was && was.pane === pane ? was.levels : ((made) => (pane.add(made, first.layer.side), made))(new Levels({ side: first.layer.side }));
    levels.set({ lines: list, format: first.format.value });
    levelsKept.set(key, { levels, pane });
  }
  for (const { levels, pane } of levelsBefore.values()) pane.remove(levels);
  engine.invalidate(true);
  markSignals(doc, data, drawn, old);
  for (const series of old) {
    heatmaps?.heatSeries.delete(series), dropOwnLayers(series);
    ((pane) => [series.engine, series.edge].forEach((own) => own && pane?.remove(own)))(engine.panes[seriesPane(series)]);
  }
  while (engine.panes.length > Math.max(1, doc.panes.length)) engine.removePane(engine.panes[engine.panes.length - 1]);
  if (heatmaps) for (const id of heatmaps.heatLooks.keys()) if (!doc.layers.some((layer) => layer.id === id && heatLike(layer))) heatmaps.heatLooks.delete(id), heatmaps.heatWants.delete(id);
  for (const [id, series] of drawn) if (!sketches.has(series)) sketches.set(series, new Sketch(id, series));
  // Each pane's series front to back as the panel lists them (its first layer in front; reversed, its last), over its
  // lines across (in a stack, its foot in front of the taller ones: each band and bar shows whole). Drawn back first:
  // as a series is added, or the order's another (a layer moved, a pane reversed). Each series' edge and its own
  // layers just over it (see ownLayer): those behind every series under them all, those over everything over them,
  // each in the same order.
  const order = JSON.stringify([doc.layers.map((layer) => [layer.id, layer.pane]), doc.panes.map((pane) => Boolean(pane.reverse))]);
  if (added || order !== drawnOrder) {
    /** @type {Map<number, { under: any[], series: any[], over: any[] }>} */
    const lists = new Map();
    const reversed = new Set(doc.panes.filter((pane) => pane.reverse).map((pane) => pane.id));
    for (const layer of [...[...doc.layers].reverse().filter((item) => !reversed.has(item.pane)), ...doc.layers.filter((item) => reversed.has(item.pane))]) {
      const pane = paneIndex.get(layer.pane) ?? 0, list = lists.get(pane) ?? lists.set(pane, { under: [], series: [], over: [] }).get(pane);
      for (const series of [drawn.get(layer.id)]) {
        if (!series || !list) continue;
        list.series.push(series.engine);
        if (series.edge) list.series.push(series.edge);
        for (const custom of ownLayers.get(series) ?? []) (custom.behind ? list.under : custom.over ? list.over : list.series).push(custom);
      }
    }
    for (const [pane, { under, series, over }] of lists) engine.panes[pane]?.order([...under, ...series, ...over]);
  }
  drawnOrder = order;
  const panes = engine.panes;
  const sides = new Set(doc.layers.flatMap((layer) => (edgesOf(layer).length > 1 ? [] : [layer.side])));
  doc.panes.forEach((pane, i) => {
    if (!panes[i]) return;
    const fixed = shares.has(pane.id) ? SHARE_FORMAT : null;
    panes[i].setPercentFormat(PERCENT_FORMAT.value, PERCENT_FORMAT.ticks);
    for (const side of SIDES) {
      const format = asRows(doc, pane) ? NO_FORMAT : fixed ?? valueFormatOf(doc.layers.filter((layer) => layer.pane === pane.id && layer.side === side && !layer.hidden && layer.type !== "signal").map((layer) => unitFor(layer, data.results.get(layer.id))));
      panes[i].setScale(side, { ...marginsOf(doc, pane, i), ...(keep ? {} : { mode: SCALE_MODES[paneMode(pane)] }) });
      if (!keep) panes[i].setRange(undefined, undefined, side);
      const used = panes[i].series.some((series, k) => (panes[i].sides[k] ? "left" : "right") === side && !series.overlay);
      panes[i].setFormat(used || format === NO_FORMAT ? format.value : libraryWords, used || format === NO_FORMAT ? format.ticks : null, side);
    }
  });
  const axis = sides.has("right") ? RIGHT_AXIS : 0;
  if (engine.axis !== axis) (engine.axis = axis), engine.layout();
  engine.invalidate(true);
  if (!(holding && priorShape === panesShape)) sizePanes();
  const layerIds = (/** @type {Chart} */ of) => of.layers.map((layer) => layer.id).join();
  if (!keep || layerIds(doc) !== (prior ? layerIds(prior.doc) : "")) engine.fitAxes();
  nameAxis();
  fitSpacing(doc, data.times.length, same ? shownSpan() : null);
  const opening = openingView(doc), taking = Boolean(retaking) && opening === retaking?.view;
  const left = !same || taking ? opening : null;
  const linkOpen = prior?.doc.id !== doc.id && linkWindow === doc.id;
  if (linkOpen) linkWindow = "";
  if ((left || linkOpen) && viewKey(doc.id) === LINKED) (linkedTook = taking ? /** @type {any} */ (retaking).at : viewsNow()[LINKED]?.at ?? 0), (handled = false);
  retaking = null;
  const onRange = Boolean(left?.latest && left.index === doc.index && data.times.length > 1) && ((preset, last, latest) => Math.abs(latest[0] - (preset.from - last)) < 1 && Math.abs(latest[1] - (preset.to - last)) <= Math.max(1, (preset.to - last) / 4))(presetWindow(doc, data), data.times.length - 1, /** @type {[number, number]} */ (left?.latest));
  const back = left && !onRange && !owed && data.times.length > 1 ? viewOn(left, doc.index, data.times) : null;
  if (back && back.to >= back.from) {
    const target = within(back, "to");
    showSpan(target, EXACT_RANGE);
    engine.render(true);
    showSpan(target, EXACT_RANGE);
    setPresetShown(false);
    if (viewKey(doc.id) === LINKED) linkedAt = ((range) => range && viewRecord(range, data.times, doc.index))(shownSpan());
  } else if (owed && visible) {
    showDates(visible);
    engine.render(true);
    applyRange(doc, data, true);
    setPresetShown(true);
  } else if (!visible || presetShown || onRange || linkOpen || owed) {
    applyRange(doc, data);
    engine.render(true);
    applyRange(doc, data);
    setPresetShown(true);
    if (onRange || linkOpen) linkedAt = ((range) => range && viewRecord(range, data.times, doc.index))(shownSpan());
  }
  else if (shift && (glidingTo ?? logical)) {
    const range = /** @type {{ from: number, to: number }} */ (glidingTo ?? logical);
    showSpan(within({ from: range.from + shift, to: range.to + shift }));
  }
  else if (sameTimeline && (glidingTo ?? logical)) showSpan(within(/** @type {{ from: number, to: number }} */ (glidingTo ?? logical)));
  else if (showedAll) applyRange({ ...doc, range: "max" }, data);
  else showDates(visible);
  if (!sameView) {
    morphFrom = null;
    endReveal();
  }
  if (sameView && !sameTimeline && priorTimes.length > 1 && data.times.length > 1 && !reduced()) {
    const offset = data.times[0] > priorTimes[0] ? -lowerBound(priorTimes, data.times[0]) : lowerBound(data.times, priorTimes[0]);
    morphFrom = { from: offset, to: offset + priorTimes.length - 1, ...overviewScale, at: performance.now() };
    if (inReveal) inReveal.join += offset;
    const outward = offset > 0 ? -1 : offset + priorTimes.length < data.times.length ? 1 : 0;
    if (outward && inReveal?.outward !== outward) reveal(outward < 0 ? offset - 0.5 : offset + priorTimes.length - 0.5, outward);
  }
  was.clear();
  kept.clear();
  showDrawings(doc);
  if (relit) spotlight(relit);
  legendsDue = true;
  requestAnimationFrame(layOutLegends);
  relayout();
}
let legendsDue = false;
function layOutLegends() {
  if (!legendsDue) return;
  legendsDue = false;
  if (holding && legends().length === chart.panes.length - hiddenPanes(chart).size) updateLegends();
  else placeLegends();
  setGap();
}
/** @type {Chart | null} */
let lookDoc = null;
$("chart").addEventListener("pointerdown", () => engine.controller.stop(), { capture: true, passive: true });

let hoverTime = null;
const legendWidth = new ResizeObserver((entries) => entries.forEach(({ target }) => fitSoon(/** @type {HTMLElement} */ (target))));
function placeLegends() {
  if (chartShown()) placeFull();
  if (chartShown()) {
    chart.panes.forEach((pane, i) => {
      const legend = engine.panes[i]?.setLegend();
      if (!legend) return;
      const element = legend.element;
      if (!element.classList.contains("legend")) (element.classList.add("legend", "strip"), strip(element), legendWidth.observe(element));
      if (element.dataset.pane !== pane.id) (element.dataset.pane = pane.id), legendShown.delete(element);
      placeLensSwitch(element, pane);
    });
  }
  if (!chartShown()) return;
  updateLegends();
  sizePanes();
  marginsSet = "";
  relayout();
}
/** @param {HTMLElement} lens */
const leadFor = (lens) => /** @type {HTMLElement | null} */ (lens.nextElementSibling)?.style.setProperty("--lead", `${lens.offsetWidth ? lens.offsetWidth + 6 : 0}px`);
// (Its width changed later (a font come, a finger's wider buttons): on the next frame, not in the observer's round,
// where the row it moves, watched too (see legendWidth), would loop it.)
const lensWidth = new ResizeObserver((entries) => requestAnimationFrame(() => entries.forEach(({ target }) => leadFor(/** @type {HTMLElement} */ (target)))));
/** @param {HTMLElement} legend @param {Pane} pane */
function placeLensSwitch(legend, pane) {
  let lens = /** @type {HTMLElement | null} */ (legend.previousElementSibling?.matches(".lens-switch") ? legend.previousElementSibling : null);
  if (!pane.lens) return lens && (lens.remove(), legend.style.removeProperty("--lead"));
  if (!lens) {
    lens = Object.assign(document.createElement("div"), { className: "tabs lens-switch" });
    lens.setAttribute("role", "group");
    legend.before(lens);
    lensWidth.observe(lens);
  }
  const { as, at } = pane.lens, key = `${pane.id} ${at} ${as.join(" ")}`;
  if (lens.dataset.key !== key) {
    lens.dataset.key = key;
    lens.setAttribute("aria-label", `Pane ${chart.panes.indexOf(pane) + 1} shown as`);
    lens.innerHTML = as.map((name, k) => `<button type="button" data-show-view="${k}" data-pane="${pane.id}" aria-pressed="${at === k}">${escapeHtml(name)}</button>`).join("");
    leadFor(lens);
  }
}
let bandLit = /** @type {string | null} */ (null);
const submenus = [$("submenu")], subSetups = /** @type {((menu: HTMLElement) => void)[]} */ ([]);
lightRows($("menu"));
whenSubmenu(lightRows);
let pointedOption = false;
{
  const list = $("palette-list"), light = glides(list), coarse = matchMedia("(hover: none) and (pointer: coarse)");
  new MutationObserver(() => (!coarse.matches || command.keyboard()) && light(list.querySelector('[aria-selected="true"]'), !pointedOption)).observe(list, { childList: true, subtree: true, attributeFilter: ["aria-selected"] });
}
const sideLight = glides($("chart-list"), {
  again: (gone) => {
    const key = [.../** @type {HTMLElement} */ (gone.parentElement).attributes].find(({ name }) => /^data-(?:.+-row|group)$/.test(name));
    return key ? $("chart-list").querySelector(`[${key.name}="${CSS.escape(key.value)}"] > :is(.chart-item, .folder-item, .list-head-name)`) : null;
  },
  can: (item) => !foldOf.has(/** @type {Element} */ (item.closest("#chart-list > *"))),
});
let sidePointer = "", sideHeld = false;
const sidePoint = (/** @type {PointerEvent} */ event) => {
  if (event.pointerType !== "mouse") return;
  const at = `${event.clientX},${event.clientY}`;
  if (sideHeld && at === sidePointer) return;
  (sidePointer = at), (sideHeld = false);
  const row = closest(event, railed() ? ".chart-row, .folder-row" : ".chart-row, .folder-row, .list-head");
  sideLight(row ? row.querySelector(":scope > :is(.chart-item, .folder-item, .list-head-name)") : null);
};
$("chart-list").addEventListener("pointerover", sidePoint);
$("chart-list").addEventListener("pointermove", sidePoint);
$("chart-list").addEventListener("pointerleave", () => sideLight(null));
glideRows(/** @type {HTMLElement} */ (document.querySelector(".side-foot")), ".nav-item");
glideRows(/** @type {HTMLElement} */ ($("undo").parentElement), ".head-actions > .cap");
glideRows(/** @type {HTMLElement} */ (document.querySelector(".home-undo")), ".cap");
glideRows($("inspector"), ".layer-actions > .nav-item");
glideRows($("stage-foot"), "#interval, #window, #ranges > button");
glideRows(/** @type {HTMLElement} */ (document.querySelector(".palette-foot")), ".foot-action");
glideRows($("layer-list"), ".layer-row");
glideRows($("belt"), ".belt :is(.cap, .btn)");
const EMPTY = /** @type {Float64Array} */ (new Float64Array(0));
const CLOUD = { xs: new Float64Array(64), highs: new Float64Array(64), lows: new Float64Array(64), ats: new Float64Array(64) };
const SPAN = new Float64Array(2);
const measure = /** @type {CanvasRenderingContext2D} */ (document.createElement("canvas").getContext("2d"));
let figuresFont = "";
function widest(texts, weight, size) {
  measure.font = `${weight} ${size}px ${(figuresFont ||= css("--sans"))}`;
  return Math.ceil(Math.max(0, ...texts.map((text) => measure.measureText(text.replace(/[0-9]/g, "0")).width)));
}
/** @param {ChartPoint} param */
function crossMoved(param) {
  if (param.input || !param.point) pinCross(null);
  else if (crossSet) return void ((engine.crosshair.pointed = dotLineOf(crossSet.layer)), !crossHeld && !exporting && holdCross());
  const time = param.time === undefined ? null : /** @type {number} */ (param.time);
  const profiles = heatmaps?.heatLooks.size && !exporting ? heatmaps.profilesUnder(param) : [];
  const kept = time ?? ((profiles.length || (param.logical ?? -1) >= engine.time.length) && resolved ? resolved.times[lastShown()] ?? null : null);
  if (fingers.size && param.point && kept !== null && param.pane !== undefined && !tool && !placing) {
    const series = [...drawn.values()].find((item) => !item.engine.hidden && seriesPane(item) === param.pane), price = seriesValue(series, param.point.y);
    fingerCross.at = series && price !== null && price !== undefined ? { price: Number(price), time: kept, series } : null;
  } else if (!param.point && fingerCross.at && !fingers.size) {
    const { price, time: at, series } = fingerCross.at, layer = layerOfSeries(series);
    pinCross(layer === null ? null : { price, time: at, layer });
    fingerCross.at = null;
    fingerCross.kept = at;
    lightBubble(at, seriesPane(series));
    if (hoverTime !== at) (hoverTime = at), valuesSoon();
    return crossOn(price, at, series);
  } else {
    fingerCross.at = null;
    if (!param.point || time !== fingerCross.kept) fingerCross.kept = null;
  }
  crossIn = Boolean(param.point);
  const wasAt = heatAt;
  heatAt = param.point && param.pane !== undefined ? { pane: param.pane, y: param.point.y } : null;
  if (heatmaps?.heatLooks.size && (heatAt || wasAt)) valuesSoon();
  const point = param.point;
  if (!exporting) pointProfile(point ? profiles.reduce((/** @type {ReturnType<typeof heatmaps.profileHit>} */ hit, layer) => hit ?? heatmaps.profileHit(layer, point), null) : null);
  // (Kept, nothing else pointed at: in its own pane's empty space (its legend's lit item never flickers as the pointer
  // crosses gaps); off its pane or the chart, none.)
  bandLit = bandAt(param) ?? ((param.point && !profileLit && bandLit && seriesPane(drawn.get(bandLit)) === param.pane) ? bandLit : null);
  if (param.point) tagLit = null;
  namePointed();
  lightLegend(bandLit ?? profileLit?.layer ?? (param.point ? param.layer : null), false, true);
  lightBubble(param.point ? time : null, param.pane);
  if (time === hoverTime) return;
  hoverTime = time;
  valuesSoon();
}
engine.onCrosshair((index, pane, y, input, x, series) => crossMoved(chartPoint(index, pane, x, y, input, series)));
const bubbleRing = Object.assign(document.createElement("span"), { className: "bubble-lit" });
let crossIn = false;
const lightTag = (/** @type {string | null} */ tag) => ((tagLit = tag), namePointed(), lightLegend(tag, false, true));
let tagLit = /** @type {string | null} */ (null);
engine.element.addEventListener("pointermove", (event) => !crossIn && event.pointerType !== "touch" && lightTag(tagAt(event.clientX, event.clientY)));
engine.element.addEventListener("pointerleave", () => !crossIn && lightTag(null));

let keyTime = /** @type {number | null} */ (null);
let keyValue = /** @type {{ layer: string, value: number } | null} */ (null);
function keyAt(/** @type {number} */ i) {
  if (!resolved) return;
  const time = resolved.times[i], range = shownSpan();
  keyTime = time;
  pointProfile(null);
  pinCross(null);
  const first = range ? firstInView() : i, last = lastShown(), shift = i < first ? i - first : i > last ? i - last : 0;
  if (range && shift) showSpan({ from: range.from + shift, to: range.to + shift });
  const point = keyPoint(i);
  fingerCross.kept = null;
  pinCross(point ? { price: point.value, time, layer: point.layer.id } : null);
  if (point) crossOn(point.value, time, drawn.get(point.layer.id));
  else engine.setCrosshair(NaN);
  lightBubble(point ? time : null, point ? chart.panes.findIndex((pane) => pane.id === point.layer.pane) : undefined);
  if (point && draft && placing && !placing.held) {
    draft = { ...draft, points: [...draft.points.slice(0, -1), pointOf(time, point.value)] };
    repaintDrawings();
  }
  hoverTime = time;
  valuesSoon();
  const shown = chart.layers.filter((item) => !item.hidden);
  $("chart-said").textContent = `${pointLabel(time)}: ${shown.length ? shown.map((item) => `${labelOf(item)} ${sayValue(item, { at: i, rest: false })}`).join(", ") : "every layer is hidden"}`;
}
function keyAway() {
  keyValue = null;
  if (keyTime === null) return;
  keyTime = null;
  pinCross(null), (fingerCross.kept = null);
  engine.setCrosshair(NaN);
  lightBubble(null, undefined);
  hoverTime = null;
  valuesSoon();
}
function nudgeKey(/** @type {number} */ i, /** @type {boolean} */ up, /** @type {boolean} */ far) {
  const point = keyPoint(i), series = point && drawn.get(point.layer.id), y = point && series ? seriesY(series, point.value) : null;
  const height = point ? (engine.panes[chart.panes.findIndex((pane) => pane.id === point.layer.pane)]?.height ?? 0) : 0;
  if (!point || !series || y === null || y === undefined || !height) return;
  const value = seriesValue(series, clamp(y + ((up ? -1 : 1) * height) / (far ? 10 : 100), 0, height));
  if (value === null || !Number.isFinite(value)) return;
  keyValue = { layer: point.layer.id, value };
  keyAt(i);
  $("chart-said").textContent = amountOn(point.layer, value);
}
function placeByKeys(/** @type {number} */ i) {
  const kind = tool, point = keyPoint(i), said = $("chart-said");
  if (!kind || !resolved) return;
  if (kind === "brush") return void (said.textContent = "The brush draws only with a pointer");
  if (!point) return void (said.textContent = "Nothing drawn at this date to place it on");
  clearMeasure();
  const time = resolved.times[i], spot = pointOf(time, point.value), part = BOXES.has(kind) ? "corner" : "end";
  const at = (/** @type {[number, number]} */ [when, value]) => `${pointLabel(when)}, ${amountOn(point.layer, value)}`;
  if (kind === "path" && placing && draft) {
    draft = { ...draft, points: [...draft.points.slice(0, -1), spot] };
    return placePathPoint();
  }
  if (POINTED.has(kind) && !placing) {
    const rect = paneRect(point.layer);
    if (!rect) return;
    placing = { rect, host: point.layer, down: { x: NaN, y: NaN }, held: false, clicks: true, keys: true, stroke: [] };
    pathResume = null;
    draft = pen.newDrawing(kind, point.layer.id, [spot, [...spot]]);
    repaintDrawings();
    renderBelt();
    return void (said.textContent = `First ${kind === "path" ? "point" : part} at ${at(spot)}`);
  }
  const drawing = draft && placing ? { ...draft, points: [draft.points[0], spot] } : pen.newDrawing(kind, point.layer.id, [spot]);
  if (PAIRED.has(kind) && !heatmaps.pointsApart(drawing)) return void (said.textContent = `Its other ${part} is on its first: move it first`);
  keyValue = null;
  takeUp(toolKept ? kind : "", toolKept, true);
  if (kind === "text") return startTyping(drawing, $("plot-keys"));
  keepDrawing(drawing);
  const reading = kind === "ruler" ? rulerReading(drawing) : null;
  if (kind === "ruler") said.textContent = reading ? `${reading.name}: ${reading.figures.join(", ")} over ${reading.span}` : "Nothing measured";
  else if (pen.drawingById(drawing.id)) said.textContent = `${TOOLS[kind].name} ${kind === "hline" ? `at ${amountOn(point.layer, point.value)}` : kind === "vline" ? `at ${pointLabel(time)}` : `from ${at(drawing.points[0])} to ${at(spot)}`}`;
}
$("plot-keys").addEventListener("focus", () => (describePlot(), tool && renderBelt()));
$("plot-keys").addEventListener("blur", () => (document.hasFocus() && keyAway(), tool && renderBelt()));
$("plot-keys").addEventListener("keydown", (event) => {
  const n = resolved?.times.length ?? 0, range = shownSpan();
  if (!n || !range || !chartShown() || $("chart").classList.contains("waiting") || event.metaKey || event.ctrlKey || event.altKey || (chosen.size && event.key.startsWith("Arrow"))) return;
  const on = keyTime === null ? -1 : indexOfTime(keyTime), first = firstInView(), last = lastShown(), from = on >= 0 ? on : last + 1;
  const step = event.shiftKey ? 10 : 1;
  const to = event.key === "ArrowLeft" ? from - step : event.key === "ArrowRight" ? (on < 0 ? first : from + step) : event.key === "Home" ? first : event.key === "End" ? last : null;
  if (event.key === "Escape" && keyTime !== null && !tool && !placing && !measuring) {
    event.preventDefault();
    event.stopPropagation();
    return keyAway();
  }
  const keyed = tool && tool !== "brush" && upDown(event);
  if ((tool && event.key === "Enter") || keyed) {
    event.preventDefault();
    event.stopPropagation();
    if (on < 0) keyAt(last);
    return event.key === "Enter" ? placeByKeys(on < 0 ? last : on) : nudgeKey(on < 0 ? last : on, event.key === "ArrowUp", event.shiftKey);
  }
  if (to === null) return;
  event.preventDefault();
  event.stopPropagation();
  keyValue = null;
  keyAt(clamp(to, 0, n - 1));
});
let layoutFrame = 0, laidWidth = 0, layoutOwed = false;
function relayout() {
  if (layoutFrame) return;
  layoutFrame = requestAnimationFrame(() => {
    layoutFrame = 0;
    if (document.body.hasAttribute("data-morph")) return void (layoutOwed = true);
    layoutOwed = false;
    if (sizeHeld) return void pen?.placeBelt();
    if (lensAt) lookAt(lensAt);
    const width = engine.time.width;
    const restarted = width !== laidWidth && Boolean(placing?.clicks && draft) && !placingPath();
    if (restarted) {
      placing = draft = null;
      repaintDrawings();
      renderBelt();
    } else if (placing) {
      placing.rect = paneRect(placing.host) ?? placing.rect;
    }
    if (width !== laidWidth) {
      laidWidth = width;
      if (resolved && chartShown()) fitSpacing(chart, resolved.times.length);
      if (chart) renderRanges();
    }
    drawOverview();
    pen?.placeBelt();
    if (restarted) notice(`The chart moved: ${CLICK} the drawing's first point again`);
    placePlotKeys();
    if (chartShown()) sizePanes();
    fitMargins();
  });
}
let chartHeight = 0;
let overviewBox = { width: 0, height: 0 };
let marginsSet = "";
function fitMargins() {
  if (!chartShown()) return;
  const all = chart.panes.map((pane, i) => marginsOf(chart, pane, i)), key = JSON.stringify(all.map(({ top, bottom }) => [top.toFixed(3), bottom.toFixed(3)]));
  if (key === marginsSet) return;
  marginsSet = key;
  all.forEach((margins, i) => SIDES.forEach((side) => engine.panes[i]?.setScale(side, margins)));
  engine.invalidate(true);
}
engine.onLayout(() => engine.resizing && fitMargins());
addEventListener("pointerdown", (event) => press(event, { end: keepPaneSizes }), true);
/** Whether the view moves: from a move (a pan, a fling, a zoom, a glide, the navigator, the wheel) till it's at rest: a frame drawn with it where it was, nothing held down on it, no glide on its way. At rest, what waited for it: the rows' changes in full (see updateRowChanges), more data reached for (see reachOut). */
let viewMoving = false, movedSince = false, restFrame = 0, wheelAt = -Infinity;
addEventListener("wheel", () => (wheelAt = performance.now()), { capture: true, passive: true });
const restSoon = () => void (restFrame ||= requestAnimationFrame(atRest));
function viewMoved() {
  viewMoving = movedSince = true;
  restSoon();
}
const released = () => ((movedSince = true), restSoon());
function atRest() {
  restFrame = 0;
  if (movedSince || engine.controller.target || performance.now() - wheelAt < WHEEL_QUIET) return void ((movedSince = false), restSoon());
  if (pressed.size || navBar.dragging) return;
  viewMoving = false;
  valuesSoon();
  reachOut();
  showAddress();
}
/** @type {{ revision: number, from: number, to: number, sizes: typeof rowSizes } | null} */
let roomsAt = null;
function updateRowChanges() {
  if (!resolved || !panelShown()) return;
  const point = readPoint(), moving = viewMoving, since = /** @type {Set<number>} */ (new Set());
  for (const element of all$(".row-change[data-change]")) {
    const id = /** @type {string} */ (element.dataset.change), layer = layerById(id), result = resolved.results.get(id);
    if (!layer || !result) continue;
    const change = !layer.hidden && !result.error && !(named(layer) && bandOf(layer, valueAt(layer, point))) ? changeOver(layer, point) : null;
    const name = `row-change ${change?.up === true ? "up" : change?.up === false ? "down" : ""}`, text = change ? chipText(change) : "";
    if (change) since.add(change.from);
    if (element.className !== name) element.className = name;
    if (writeText(element, text)) /** @type {HTMLElement} */ (element.parentElement).classList.toggle("changed", Boolean(text));
    // While the view moves (a pan, a zoom), its change only: its tip and its room, read and measured from the view,
    // are set once the view rests (each frame, they'd cost the move its smoothness).
    if (moving) continue;
    const tip = change ? `${viewName(chart, layer)}\nChange from ${pointLabel(change.from)} to ${pointLabel(change.to)}` : undefined;
    if (element.dataset.tip !== tip) {
      if (tip) element.dataset.tip = tip;
      else delete element.dataset.tip;
      if (tipTarget === element && !$("tip").hidden) tip ? showTip(element) : hideTip();
    }
  }
  if (moving) return;
  const from = firstInView(), to = lastShown();
  if (!roomsAt || roomsAt.revision !== dataRevision || roomsAt.from !== from || roomsAt.to !== to || roomsAt.sizes !== rowSizes) {
    roomsAt = { revision: dataRevision, from, to, sizes: rowSizes };
    for (const figures of all$(".row-figures", $("layer-list"))) {
      const layer = layerById(/** @type {HTMLElement} */ (figures.closest("[data-layer]"))?.dataset.layer ?? ""), result = layer && resolved.results.get(layer.id);
      if (!layer || !result) continue;
      const room = figuresRoom(layer, result), main = /** @type {HTMLElement} */ (figures.parentElement);
      figures.classList.toggle("dated", room.dated);
      for (const [name, px] of /** @type {const} */ ([["--value-room", room.value], ["--change-room", room.change], ["--date-room", room.date]])) {
        if (main.style.getPropertyValue(name) !== (px ? `${px}px` : "")) px ? main.style.setProperty(name, `${px}px`) : main.style.removeProperty(name);
      }
    }
  }
  const head = $("changes-since"), text = since.size === 1 ? `Change since ${pointLabel([...since][0])}` : since.size ? "Change since the view's start" : "";
  if (head.textContent !== text) head.textContent = text;
  head.hidden = !text;
}

let morphFrom = /** @type {{ from: number, to: number, lo: number, hi: number, at: number } | null} */ (null);
let overviewScale = { lo: NaN, hi: NaN };
let revealed = false;
const overviewBase = document.createElement("canvas");
let overviewRatio = 1;
let liveMoves = 0;
/** @type {{ key: string, n: number, lo: number, hi: number, points: [number, number][] }} */
let overviewShape = { key: "", n: 0, lo: NaN, hi: NaN, points: [] };
function drawOverview() {
  const canvas = overviewBase, rect = overviewBox;
  if (!rect.width) return;
  const ratio = (overviewRatio = devicePixelRatio || 1);
  const width = Math.max(1, Math.round(rect.width * ratio)), height = Math.max(1, Math.round(rect.height * ratio));
  if (canvas.width !== width || canvas.height !== height) (canvas.width = width), (canvas.height = height);
  const context = /** @type {CanvasRenderingContext2D} */ (canvas.getContext("2d"));
  context.clearRect(0, 0, canvas.width, canvas.height);
  const source = overviewSource();
  if (source && source.values.length > 1) {
    const key = `${idOf(source.values)}:${source.log}:${canvas.width}:${liveMoves}`;
    if (overviewShape.key !== key) {
      const scaled = onScaleAll(source.values, source.log ? 1 : 0, NaN);
      let [lo, hi] = rangeOf(scaled);
      // Smooth: each pixel's column its mean, a sketch of the series, not its noise. A few points in one column, a mean is one point, no line: their lowest and highest then.
      let points = meansOf(scaled, canvas.width);
      if (points.length < 2) points = command.columnsOf(scaled, canvas.width);
      overviewShape = { key, n: scaled.length, lo, hi, points };
    }
    let { lo, hi } = overviewShape;
    const points = overviewShape.points;
    if (Number.isFinite(lo)) {
      const span = overviewSpan(overviewShape.n), w = canvas.width, h = canvas.height, top = 4 * ratio, band = 3 * ratio;
      overviewScale = { lo, hi };
      if (morphFrom && span.t < 1 && Number.isFinite(morphFrom.lo)) (lo = morphFrom.lo + (lo - morphFrom.lo) * span.t), (hi = morphFrom.hi + (hi - morphFrom.hi) * span.t);
      const x = (i) => ((i - span.from) / Math.max(1, span.to - span.from)) * w;
      const y = (value) => h - band - ((value - lo) / (hi - lo || 1)) * (h - band - top);
      const color = ink(source.layer.color);
      context.beginPath();
      for (const [i, value] of points) context.lineTo(x(i), y(value));
      context.strokeStyle = alpha(color, 0.85);
      context.lineWidth = 1.2 * ratio;
      context.stroke();
      context.lineTo(x(points[points.length - 1][0]), h);
      context.lineTo(x(points[0][0]), h);
      context.closePath();
      const fade = context.createLinearGradient(0, 0, 0, h);
      fade.addColorStop(0, alpha(color, 0.3));
      fade.addColorStop(1, alpha(color, 0.06));
      context.fillStyle = fade;
      context.fill();
      if (span.t < 1) relayout();
      else morphFrom = null;
    }
  }
  placeBrush();
}
const BRUSH_MIN = TOUCH ? 36 : 24;
$("navigator").style.setProperty("--brush-min", `${BRUSH_MIN}px`);
let navigatorFont = "";
let navigatorNear = false;
/** @param {{ left: number, right: number } | null} lit */
function paintNavigator(lit) {
  const canvas = $("overview"), context = canvas.getContext("2d"), ratio = overviewRatio;
  if ((devicePixelRatio || 1) !== overviewRatio) relayout();
  if (canvas.width !== overviewBase.width || canvas.height !== overviewBase.height) (canvas.width = overviewBase.width), (canvas.height = overviewBase.height);
  const w = canvas.width, h = canvas.height, edge = 4 * ratio, inner = 10 * ratio, gap = 8 * ratio, foot = h - 3.5 * ratio;
  context.clearRect(0, 0, w, h);
  // (Its font set once: kept by the canvas till it's resized, setting it again would work out the page's styles.)
  if (context.font !== navigatorFont) {
    context.font = `${CHART_TEXT * ratio}px ${chartSans}`;
    navigatorFont = context.font;
  }
  /** @type {{ text: string, width: number, x: number }[]} */
  const dates = [];
  const outside = new Path2D();
  const shown = shownDates();
  const over = fineDrawn() ? "the stretch loaded" : "all history";
  const keys = "← → to move, ↑ ↓ to zoom, ⇧ for bigger steps";
  const label = (/** @type {string} */ said) => $("navigator").getAttribute("aria-label") !== said && $("navigator").setAttribute("aria-label", said);
  if (!(lit && shown)) label(`Overview of ${over}: ${keys}`);
  else {
    const [left, right] = windowOn(lit.left * w, lit.right * w, w, BRUSH_MIN * ratio);
    outside.rect(0, 0, w, h);
    outside.roundRect(left, 0, right - left, h, 8 * ratio);
    // (Said, always; painted, and so measured, only while it's pointed at or dragged: at rest, the time axis above
    // says them, and a canvas measuring text works out the page's styles first.)
    const near = navigatorNear || $("brush").classList.contains("dragging");
    const ends = spanParts(Number(shown.from), Number(shown.to)).map((text) => ({ text, width: near ? context.measureText(text).width : 0 }));
    label(`Overview of ${over}, showing ${ends.map((end) => end.text).join(" – ")}: ${keys}`);
    if (near && ends.length === 2) {
      const [start, end] = ends;
      const from = left - edge - start.width >= edge ? left - edge - start.width : left + inner;
      const to = right + edge + end.width <= w - edge ? right + edge : right - inner - end.width;
      if (from + start.width + gap <= to && (from < left || from + start.width <= right - inner) && (to > right || to >= left + inner)) dates.push({ ...start, x: from }, { ...end, x: to });
    }
    if (near && !dates.length) {
      const text = ends.map((end) => end.text).join(" – "), width = context.measureText(text).width;
      const x = [left - edge - width, right + edge, (left + right - width) / 2].find((at, i) => at >= edge && at + width <= w - edge && (i < 2 || (at >= left + inner && at + width <= right - inner)));
      if (x !== undefined) dates.push({ text, width, x });
    }
  }
  context.drawImage(overviewBase, 0, 0);
  if (lit && shown) {
    context.globalCompositeOperation = "destination-out";
    context.fillStyle = "rgb(0 0 0 / 0.5)";
    context.fill(outside, "evenodd");
    context.globalCompositeOperation = "source-over";
  }
  context.lineWidth = 3 * ratio;
  context.lineJoin = "round";
  context.strokeStyle = css("--card");
  context.fillStyle = css("--text");
  for (const date of dates) {
    context.strokeText(date.text, date.x, foot);
    context.fillText(date.text, date.x, foot);
  }
}
new ResizeObserver(() => (holdView(), relayout())).observe(engine.element);
/** @type {{ box: number, at: { from: number, to: number } } | null} */
let heldView = null, madeView = /** @type {{ from: number, to: number } | null} */ (null);
let lastBox = 0;
let exporting = false;
function holdView() {
  const width = engine.time.width, box = chartWidth(), was = heldView, resized = box !== lastBox;
  if (exporting) return;
  if (width) lastBox = box;
  if (!was || !width || !resized) return;
  setReserve(width);
  if (!resolved || !chartShown()) return void (heldView = madeView = null);
  const view = resizeKeeps ?? was.at;
  fitSpacing(chart, resolved.times.length, view);
  madeView = view;
  showSpan(view, EXACT_RANGE);
}
engine.onRange(() => {
  ((range, width, box) => {
    if (!range || !width || exporting) return;
    lastBox ||= box;
    const made = madeView && Math.abs(range.from - madeView.from) < 0.05 && Math.abs(range.to - madeView.to) < 0.05;
    if (!heldView || heldView.box === box || (madeView && !made)) (heldView = { box, at: range }), (madeView = null);
  })(shownSpan(), engine.time.width, chartWidth());
  if (lensAt) lookAt(lensAt);
  nameAxis();
  if (writing) heatmaps.placeField();
  lightBubble(bubbleAt.time, bubbleAt.pane);
  viewMoved();
  placeBrush();
  fitDots();
  valuesSoon();
  const shown = engine.time.width !== laidWidth && layoutFrame ? presetShown : showsPreset();
  if (shown !== presetShown) setPresetShown(shown);
  prefetch();
  keepView();
});
// (Reaching out waits for the view to rest (see viewMoving): mid-gesture, the chart keeps a pan's start by its last
// bar, so points loaded under it would make the view leap.)
let reaching = false;
$("chart").addEventListener("pointerdown", (event) => {
  const id = event.pointerId;
  pressed.add(id);
  if (event.pointerType === "touch") fingers.add(id);
  press(event, { end: () => (fingers.delete(id), pressed.delete(id) && !pressed.size && released()) });
}, true);
$("chart").addEventListener("pointerdown", (event) => {
  fingerCross.dismissing = event.pointerType === "touch" && fingerCross.kept !== null && engine.element.contains(/** @type {Element} */ (event.target));
  if (fingerCross.dismissing) (fingerCross.kept = null), pinCross(null), engine.setCrosshair(NaN), lightBubble(null, undefined);
}, true);
engine.element.addEventListener("pointermove", (event) => event.pointerType !== "touch" && (fingerCross.kept = null));
$("chart").addEventListener("pointerdown", () => clearMeasure(), true);

/** @param {Span} span @param {number} by */
async function grow(span, by) {
  reaching = true;
  const was = { ...span }, index = chart.index, reached = by < 0 ? resolved?.times[0] : resolved?.times.at(-1);
  if (by < 0) span.from--;
  else {
    span.to++;
    span.live = span.to >= latestChunk(index);
  }
  if (span.to - span.from + 1 > MOST_CHUNKS) {
    if (by < 0) (span.to--, (span.live = false));
    else span.from++;
  }
  for (const key of [...chunks.keys()]) {
    if (failedChunks.has(key) && key.includes(`@${index}#`)) (chunks.delete(key), failedChunks.delete(key));
  }
  let drew = false;
  try {
    drew = await draw();
  } finally {
    reaching = false;
  }
  if (!drew) return;
  const now = by < 0 ? resolved?.times[0] : resolved?.times.at(-1);
  if (reached !== undefined && now !== undefined && (by < 0 ? now >= reached : now <= reached)) {
    const dropped = by < 0 ? span.to < was.to : span.from > was.from, k = by < 0 ? span.from : span.to, names = drawSeries(chart);
    Object.assign(span, was, { rest: Date.now() + 5000 });
    if (dropped) void draw();
    // An older chunk that came empty for every series the chart reads (none failed; not judged by what's drawn: a
    // formula's result is trimmed of its leading zeros): none older asked for again, its series starting later.
    if (by < 0) {
      const empty = await Promise.all(names.map((name) => chunks.get(`${name}@${index}#${k}`)?.then((got) => got.data.every((value) => value == null), () => false) ?? !chunked(name, index)));
      if (empty.every(Boolean)) span.floor = { at: was.from, of: names.sort().join() };
    }
    return;
  }
  // Once the chart has moved the view onto its new points, at rest again: judged before, the view it had would ask
  // for the same chunk out again.
  restSoon();
}

$("navigator").dataset.tip = `Drag to pan, drag an edge to zoom, ${CLICK} to jump`;
const navBar = new Navigator($("navigator"), engine, {
  overview: () => ((n) => (n < 2 ? null : overviewSpan(n)))(resolved?.times.length ?? 0),
  length: () => (resolved?.times.length ?? 0) - 1,
  data: () => resolved?.times,
  within,
  glide: (range) => glide(range),
  window: $("brush"),
  least: BRUSH_MIN,
  grip: TOUCH ? 7 : 5,
  pressed: () => (morphFrom = null),
  dropped: () => (placeBrush(), released()),
});
for (const [type, near] of /** @type {const} */ ([["pointerenter", true], ["pointerleave", false]])) $("navigator").addEventListener(type, () => ((navigatorNear = near), placeBrush()));
$("navigator").addEventListener("keydown", (event) => {
  const range = shownSpan(), n = resolved?.times.length ?? 0;
  if (!range || n < 2 || event.metaKey || event.ctrlKey || event.altKey || !["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", "Home", "End"].includes(event.key)) return;
  event.preventDefault();
  event.stopPropagation();
  const width = range.to - range.from, far = event.shiftKey;
  if (upDown(event)) return zoom(event.key === "ArrowUp" ? (far ? 0.5 : 0.8) : far ? 2 : 1.25);
  const by = (event.key === "ArrowLeft" ? -1 : 1) * width * (far ? 1 : 0.1);
  const end = n - 1 + (rightGap() * width) / Math.max(1, engine.time.width);
  glide(event.key === "Home" ? { from: -0.5, to: width - 0.5 } : event.key === "End" ? { from: end - width, to: end } : { from: range.from + by, to: range.to + by });
});

const SANS = css("--sans");
const measurer = /** @type {CanvasRenderingContext2D} */ (document.createElement("canvas").getContext("2d"));
document.fonts.addEventListener("loadingdone", () => {
  tagShapes.clear();
  repaintDrawings();
  engine.invalidate(true);
});
let drawingsHidden = false;
function hideDrawings(/** @type {boolean} */ hidden, tell = false) {
  drawingsHidden = hidden;
  if (hidden) chooseDrawings([]);
  renderBelt();
  repaintDrawings();
  renderLayers();
  if (hidden && tell && $("belt").hidden) toast(`${plural(chart.drawings?.length ?? 0, "drawing")} hidden`, { label: "Show", run: () => hideDrawings(false) });
}
function revealDrawings() {
  if (!drawingsHidden) return;
  drawingsHidden = false;
  renderLayers();
}
/** @type {Drawing | null} */
let measuring = null;
let drawingsOf = "";
function clearMeasure() {
  const was = measuring;
  if (!was) return false;
  measuring = null;
  for (const series of drawn.values()) {
    const sketch = sketches.get(series);
    if (sketch) sketch.shown = sketch.shown.filter((/** @type {{ drawing: Drawing }} */ item) => item.drawing !== was);
  }
  repaintDrawings();
  return true;
}
/** @param {Chart} doc */
function showDrawings(doc) {
  if (doc.id !== drawingsOf || (measuring && !drawn.has(measuring.layer))) measuring = null;
  drawingsOf = doc.id;
  if (placing && !drawn.has(placing.host.id)) stopDrawing();
  shownDrawings = doc.drawings ?? [];
  repaintDrawings();
}

/** @type {DrawingKind | ""} */
let tool = "";
let placing = null;
let moving = null;
/** @type {Set<string>} */
let chosen = new Set();
/** @type {Map<string, Drawing>} */
let reshaped = new Map();
/** @type {Drawing | null} */
let draft = null;
let writing = null, typingIn = "", labelTyping = "";
let capturing = false;
/** @type {Drawing[]} */
let shownDrawings = [];

let toolKept = false;
let magnifying = false, lensAt = /** @type {{ x: number, y: number, touch: boolean } | null} */ (null), lensFrame = 0;
let pointerAt = /** @type {{ x: number, y: number } | null} */ (null);
function lookAt(/** @type {{ x: number, y: number, touch: boolean } | null} */ at) {
  lensAt = magnifying ? at : null;
  cancelAnimationFrame(lensFrame);
  lensFrame = requestAnimationFrame(drawLens);
}
function drawLens() {
  lensFrame = 0;
  const loupe = $("loupe"), at = lensAt;
  if (!at) return void (loupe.hidden = true);
  const ratio = devicePixelRatio || 1, size = Math.round(pen.LENS * ratio), canvas = /** @type {HTMLCanvasElement} */ (loupe.firstElementChild);
  if (canvas.width !== size) canvas.width = canvas.height = size;
  const context = /** @type {CanvasRenderingContext2D} */ (canvas.getContext("2d")), span = pen.LENS / pen.LENS_ZOOM / 2, k = pen.LENS_ZOOM * ratio;
  context.clearRect(0, 0, size, size);
  context.imageSmoothingQuality = "high";
  // Each of the chart's layers, in order, the part under the lens: not the lens itself, which would show itself again, and again.
  for (const source of $("canvas").querySelectorAll("canvas")) {
    if (source === canvas) continue;
    const box = source.getBoundingClientRect();
    const x0 = Math.max(box.left, at.x - span), y0 = Math.max(box.top, at.y - span), x1 = Math.min(box.right, at.x + span), y1 = Math.min(box.bottom, at.y + span);
    if (!box.width || !box.height || x1 <= x0 || y1 <= y0) continue;
    const sx = source.width / box.width, sy = source.height / box.height;
    context.drawImage(source, (x0 - box.left) * sx, (y0 - box.top) * sy, (x1 - x0) * sx, (y1 - y0) * sy, (x0 - at.x + span) * k, (y0 - at.y + span) * k, (x1 - x0) * k, (y1 - y0) * k);
  }
  const mid = size / 2, r = 3.5 * ratio;
  context.lineWidth = 3 * ratio;
  context.strokeStyle = alpha(css("--card"), 0.85);
  context.beginPath();
  context.arc(mid, mid, r, 0, 2 * Math.PI);
  context.stroke();
  context.lineWidth = 1.25 * ratio;
  context.strokeStyle = css("--ink");
  context.stroke();
  const room = (/** @type {number} */ value, /** @type {number} */ most) => Math.min(Math.max(8, value), most - pen.LENS - 8);
  let left = at.x - pen.LENS / 2, top = at.y - pen.LENS / 2;
  if (at.touch) {
    const clear = (/** @type {number} */ x, /** @type {number} */ y) => !(at.x > x - 36 && at.x < x + pen.LENS + 36 && at.y > y - 36 && at.y < y + pen.LENS + 36);
    const near = at.x > innerWidth / 2 ? at.x - pen.LENS - 36 : at.x + 36;
    [left, top] = [[room(left, innerWidth), at.y - pen.LENS - 36], [room(near, innerWidth), room(top, innerHeight)], [room(left, innerWidth), at.y + 36]].find(([x, y]) => y >= 8 && y <= innerHeight - pen.LENS - 8 && clear(x, y)) ?? [room(left, innerWidth), room(at.y - pen.LENS - 36, innerHeight)];
  }
  loupe.style.transform = `translate(${Math.round(left)}px, ${Math.round(top)}px)`;
  loupe.hidden = false;
}
let selectingArea = false;
function takeUp(/** @type {DrawingKind | "select" | "magnifier" | ""} */ what, kept = false, auto = false) {
  undoTook = "";
  if (!auto) picks++;
  pathTaken = [];
  pathResume = null;
  const kind = what === "select" || what === "magnifier" ? "" : what, lens = what === "magnifier";
  toolKept = Boolean(kind) && kept && kind !== "text";
  if (kind) (chooseDrawings([]), revealDrawings());
  tool = kind;
  selectingArea = what === "select";
  placing = moving = draft = boxing = null;
  keyValue = null;
  reshaped = new Map();
  $("select-box").hidden = true;
  heatmaps.hold("");
  document.body.toggleAttribute("data-drawing", Boolean(kind) || selectingArea);
  if (lens !== magnifying) {
    magnifying = lens;
    document.body.toggleAttribute("data-magnifying", lens);
    lookAt(lens && pointerAt && !TOUCH && $("chart").matches(":hover") ? { ...pointerAt, touch: false } : null);
  }
  hover(null);
  renderBelt();
  repaintDrawings();
}
function chooseDrawings(/** @type {Iterable<string>} */ ids) {
  const next = new Set(ids);
  if (next.size) revealDrawings();
  if (next.size && (tool || selectingArea || magnifying)) takeUp("");
  if (next.size === chosen.size && [...next].every((id) => chosen.has(id))) return;
  chosen = next;
  if (next.size && layerPicks.size) clearLayerPicks();
  if (next.size && selected) deselect();
  renderBelt();
  repaintDrawings();
}

let beltShown = "", beltKeys = false;
const BELT_TOOLS = ["cursor", "select", ...KINDS, "magnifier"];
function renderBelt() {
  syncUndo();
  const belt = $("belt"), drawings = tool ? [] : /** @type {Drawing[]} */ ([...chosen].map(pen.drawingById).filter(Boolean)), drawing = drawings.length === 1 ? drawings[0] : undefined;
  const was = belt.hidden || belt.classList.contains("held") || reduced() ? null : belt.getBoundingClientRect();
  const partsWere = was ? new Map(all$("[id]", belt).map((part) => [part.id, part.getBoundingClientRect()])) : null;
  const hide = home || !chart.layers.length || !(prefs.belt.on || tool || selectingArea || magnifying || drawings.length || beltKeys);
  if (!hide) cutFade(belt);
  const going = hide && !belt.hidden ? fadeOut(belt, [{ opacity: 1, transform: "none" }, { opacity: 0, transform: "translateY(4px)" }]) : undefined;
  belt.hidden = hide;
  const stage = /** @type {HTMLElement} */ (belt.closest(".stage"));
  stage.toggleAttribute("data-belt-shown", !hide || fading.has(belt));
  going?.then(() => belt.hidden && !fading.has(belt) && stage.removeAttribute("data-belt-shown"));
  if (belt.hidden && magnifying) return takeUp("");
  $("draw-toggle").setAttribute("aria-pressed", String(prefs.belt.on));
  if (belt.hidden) {
    if (belt.contains(document.activeElement)) /** @type {HTMLElement} */ (document.activeElement).blur(), $("plot-keys").hidden || $("plot-keys").focus({ preventScroll: true });
    return;
  }
  if (!belt.classList.contains("held")) belt.classList.toggle("upright", pen.upright(prefs.belt.dock));
  const kinds = new Set(tool ? [tool] : drawings.map((item) => item.kind)), look = (/** @type {string} */ part) => pen.lookFor(part, drawings);
  const pressed = (/** @type {string} */ each) => (each === "cursor" ? !tool && !selectingArea && !magnifying : each === "select" ? selectingArea : each === "magnifier" ? magnifying : tool === each);
  const tools = BELT_TOOLS.map((each) => {
    const { name, tip = name, key, kbd, icon } = pen.beltTool(each);
    return `${each === "magnifier" ? '<i class="belt-rule"></i>' : ""}<button type="button" class="cap ${each === tool && toolKept ? "kept" : ""}" id="belt-${each}" data-tool="${each}" aria-pressed="${pressed(each)}" aria-label="${name}${each === tool && toolKept ? ", kept" : ""}" data-tip="${escapeHtml(tip)}" data-kbd="${kbd}">${icon}${key ? `<kbd>${key}</kbd>` : ""}</button>`;
  }).join("") + (chart.drawings?.length ? `<button type="button" class="cap" id="belt-hide" data-belt="hide" aria-pressed="${drawingsHidden}" aria-label="${drawingsHidden ? "Show the drawings" : "Hide the drawings"}" data-tip="${drawingsHidden ? "Show the drawings" : "Hide the drawings (they're kept)"}" data-kbd="⇧ H">${drawingsHidden ? ICONS.eyeOff : ICONS.eye}</button>` : "");
  const button = (/** @type {string} */ part, /** @type {string} */ label, /** @type {string} */ body, menu = true, kbd = "") => `<button type="button" class="cap" id="belt-${part}" data-belt="${part}" ${menu ? 'aria-haspopup="menu"' : ""} aria-label="${escapeHtml(label)}" data-tip="${escapeHtml(label)}" ${kbd ? `data-kbd="${kbd}"` : ""}>${body}</button>`;
  const has = (/** @type {string} */ part) => [...kinds].some((each) => pen.hasLook(each, part));
  const context = !kinds.size ? "" : [
    ...(has("color") ? [
      button("color", `Color: ${colorName(look("color").color)}`, `<i class="swatch" style="--c:${escapeHtml(cssColor(look("color").color))}"></i>`),
      button("opacity", `Opacity: ${Math.round((look("opacity").opacity ?? 1) * 100)}%`, ICONS.opacity),
    ] : []),
    ...(has("width") ? [button("width", `Width: ${look("width").width}px`, ICONS.weight), button("dash", `Line style: ${dashOf(look("dash").dash)[1]}`, dashGlyph(dashOf(look("dash").dash)[2], dashOf(look("dash").dash)[3]))] : []),
    ...((tool ? tool === "text" : Boolean(pen.wordsIn(drawings))) ? ((words) => [
      button("size", `Size: ${SIZE_NAMES[words.size ?? TEXT_SIZE]}`, `<span class="belt-size" style="--s: ${({ 11: 10, 13: 12, 16: 14, 22: 17 })[words.size ?? TEXT_SIZE] ?? 12}px">A</span>`),
      button("back", `Background: ${pen.backOf(words.back)[1]}`, pen.backChip(words.back ?? "", words.color)),
    ])(tool ? prefs.pen : /** @type {Drawing} */ (pen.wordsIn(drawings))) : []),
    ...([...kinds].some((each) => THROUGH.has(each)) ? [button("ends", `Ends: ${pen.endName(look("ends").start)}, ${pen.endName(look("ends").end).toLowerCase()}`, pen.endsGlyph(look("ends").start, look("ends").end))] : []),
    ...(drawings.length && [...kinds].some((each) => pen.hasLook(each, "color")) ? ['<i class="belt-rule"></i>'] : []),
    ...(drawings.length && phone.matches ? [
      button("actions", drawings.length > 1 ? `More for ${drawings.length} drawings` : "More for the drawing", ICONS.more),
    ] : drawings.length ? [
      ...(drawing?.kind === "text" ? [button("text", "Edit the text", ICONS.pencil, false)] : []),
      ...(drawing && LINES.has(drawing.kind) ? [button("label", TOUCH ? (drawing.label ? "Edit the label" : "Add a label") : drawing.label ? "Edit the label (or double-click it)" : "Add a label (or double-click the line)", ICONS.label, false)] : []),
      ...heatmaps.flipsOf(drawings.map((item) => item.id)).map(([across, way, icon]) => button(`flip-${across ? "x" : "y"}`, `Flip ${way}`, ICONS[icon], false)),
      button("duplicate", drawings.length > 1 ? `Duplicate ${drawings.length} drawings` : "Duplicate", ICONS.duplicate, false, keyText("⌘D")),
      button("remove", drawings.length > 1 ? `Remove ${drawings.length} drawings` : "Remove", ICONS.trash, false, "⌫"),
    ] : []),
  ].join("");
  const byKeys = document.activeElement === $("plot-keys"), other = BOXES.has(tool) ? "other corner" : "other end";
  const next = tool === "path" ? (byKeys ? "Arrows to its next point, ↵ to place it, ↵ again to finish" : `${capitalized(CLICK)} its next point, the last again to finish`) : "";
  const how = selectingArea ? "Drag a box around drawings to select them" : tool && (placing?.clicks ? next || (byKeys ? `Arrows to its ${other}, ↵ to place it` : `${capitalized(CLICK)} its ${other}`) : `${byKeys ? TOOLS[tool].keys : TOOLS[tool].tip}${toolKept ? ", as many as you like" : ""}`);
  const html = `<button type="button" class="grip" id="belt-grip" aria-haspopup="menu" aria-label="Move the drawing tools" data-tip="Drag it to any side of the chart">${ICONS.grip}</button>
          <div class="belt-row ${drawings.length || context ? "" : "spread"}">${drawings.length ? "" : `<div class="belt-tools" role="group" aria-label="Tools">${tools}</div>`}
          ${context ? `${drawings.length ? "" : '<i class="belt-rule"></i>'}<div class="belt-context" role="group" aria-label="${escapeHtml(tool ? `Next ${TOOLS[tool].noun}` : drawing ? pen.drawingNames().get(drawing.id) ?? "" : `${drawings.length} drawings`)}">${context}</div>` : ""}</div>
          ${how ? '<p class="belt-caption"><span></span><kbd>esc</kbd></p>' : ""}`;
  const writeHow = () => {
    const caption = belt.querySelector(".belt-caption span");
    if (caption && caption.textContent !== how) caption.textContent = how || "";
  };
  if (html !== beltShown) {
    const focus = focusKey(belt), slid = belt.querySelector(".belt-row")?.scrollLeft ?? 0;
    belt.innerHTML = beltShown = html;
    const row = belt.querySelector(".belt-row");
    if (row && slid) row.scrollLeft = slid;
    writeHow();
    drawKeys(belt);
    restoreFocus(belt, focus);
    if (focus && document.activeElement === document.body && !$("plot-keys").hidden) $("plot-keys").focus({ preventScroll: true });
    pen.rovingBelt();
    if (focus && !belt.contains(document.activeElement) && beltKeys) {
      beltKeys = false;
      belt.classList.remove("keys");
      return renderBelt();
    }
    pen.placeBelt(false, true);
    if (was) pen.reshapeBelt(was, /** @type {Map<string, DOMRect>} */ (partsWere));
    return;
  }
  writeHow();
  pen.placeBelt(false, true);
}
let beltStop = "";
withRest(() => {
  $("belt").addEventListener("focusin", (event) => {
    beltStop = /** @type {HTMLElement} */ (event.target).id;
    pen.rovingBelt();
  });
  phone.addEventListener("change", () => (renderBelt(), pen.rovingBelt()));
});
function hideBelt() {
  heatmaps.finishPath();
  pen.putDown();
  chooseDrawings([]);
  beltKeys = false;
  $("belt").classList.remove("keys");
  if (prefs.belt.on) (prefs.belt.on = false), savePrefs();
  renderBelt();
}
function focusBelt() {
  if ($("belt").classList.contains("keys")) {
    hideBelt();
    return leaveBelt();
  }
  chooseDrawings([]);
  if (!prefs.belt.on) (prefs.belt.on = true), savePrefs();
  beltFrom = document.activeElement instanceof HTMLElement && !$("belt").contains(document.activeElement) ? document.activeElement : null;
  beltKeys = true;
  renderBelt();
  $("belt").classList.add("keys");
  /** @type {HTMLElement | null} */ ($("belt").querySelector('[data-tool][aria-pressed="true"]'))?.focus();
}
let beltFrom = /** @type {HTMLElement | null} */ (null);
function leaveBelt() {
  const back = beltFrom;
  beltFrom = null;
  if (back === document.body) return void (/** @type {HTMLElement | null} */ ($("belt").querySelector(":focus"))?.blur());
  const to = back?.isConnected && back.checkVisibility() ? back : [$("plot-keys"), $("draw-toggle")].find((element) => element?.checkVisibility());
  to?.focus({ preventScroll: true });
}
$("belt").addEventListener("scroll", (event) => event.target instanceof HTMLElement && event.target.classList.contains("belt-row") && fitSoon(event.target), { capture: true, passive: true });
$("belt").addEventListener("focusout", (event) => {
  if ($("belt").contains(/** @type {Node | null} */ (event.relatedTarget))) return;
  $("belt").classList.remove("keys");
  if (!beltKeys) return;
  beltKeys = false;
  renderBelt();
});
$("belt").addEventListener("keydown", (event) => {
  if (event.metaKey || event.ctrlKey || event.altKey) return;
  const each = $("belt").classList.contains("keys") ? BELT_TOOLS.find((item) => pen.beltTool(item).key && pen.beltTool(item).key === event.key.toUpperCase()) : undefined;
  const buttons = all$("button", $("belt")), at = buttons.indexOf(/** @type {HTMLElement} */ (document.activeElement));
  const step = { ArrowLeft: -1, ArrowUp: -1, ArrowRight: 1, ArrowDown: 1 }[event.key];
  if (each) {
    pen.pickTool(each);
    leaveBelt();
  } else if (step && at >= 0) buttons[clamp(at + step, 0, buttons.length - 1)]?.focus();
  else if (event.key === "Escape" && $("belt").classList.contains("keys")) leaveBelt();
  else return;
  event.preventDefault();
  event.stopPropagation();
});
$("menu").addEventListener("click", (event) => {
  const pick = closest(event, "[data-pick-start], [data-pick-end]");
  if (!pick) return;
  event.stopPropagation();
  const side = pick.dataset.pickStart !== undefined ? "start" : "end";
  pen.setLook(side, (side === "start" ? pick.dataset.pickStart : pick.dataset.pickEnd) || undefined, "", true);
  for (const each of all$(`[data-pick-${side}]`, $("menu"))) each.setAttribute("aria-pressed", String(each === pick));
}, true);
$("belt").addEventListener("click", (event) => {
  const button = closest(event, "button");
  if (!button) return;
  if (button.id === "belt-grip") return void (event.detail === 0 && menus.menuOf(button, pen.dockChoices(), null, "", { title: "Dock the drawing tools" }));
  const { tool: each, belt: part = "" } = button.dataset, drawings = [...chosen], drawing = chosen.size === 1 ? pen.drawingById(drawings[0]) : undefined, look = pen.lookFor(part);
  if (each && event.detail === 2 && Object.hasOwn(TOOLS, each) && each !== "text") return takeUp(/** @type {DrawingKind} */ (each), true);
  if (each) return pen.pickTool(each);
  if (part === "hide") return hideDrawings(!drawingsHidden);
  if (!look) return;
  if (part === "ends") return pen.endsPopover(button);
  const words = tool ? prefs.pen : pen.wordsIn([...chosen].map(pen.drawingById));
  if (part === "size") {
    const now = words?.size ?? TEXT_SIZE;
    return pen.beltPopover(button, "Size", pen.beltTabs("Size", SIZES.map((size) => `<button type="button" data-pick-size="${size}" aria-pressed="${now === size}" aria-label="${SIZE_NAMES[size]}" data-tip="${SIZE_NAMES[size]}, ${size}px">${SIZE_SHORT[size]}</button>`)), ({ pickSize }) => pickSize && pen.setLook("size", Number(pickSize)));
  }
  if (part === "back") {
    const now = words?.back ?? "", color = words?.color ?? PEN_COLOR;
    return pen.beltPopover(button, "Background", pen.beltTabs("Background", BACKS.map(([back, text, tip]) => `<button type="button" data-pick-back="${back}" aria-pressed="${now === back}" aria-label="${text}" data-tip="${text}: ${tip.toLowerCase()}">${pen.backChip(back, color)}</button>`)), ({ pickBack }) => pickBack !== undefined && pen.setLook("back", pickBack || undefined));
  }
  if (part === "label" && drawing) return startLabel(drawing, drawing.at, button);
  if (part === "color") inspector.penColors(button);
  else if (part === "width" || part === "opacity") {
    pen.beltPopover(button, part === "width" ? "Width" : "Opacity", inspector.slider(`draw-${part}`, look), () => {});
    $(`draw-${part}`)?.focus();
  } else if (part === "dash") pen.beltPopover(button, "Line style", pen.beltTabs("Line style", DASHES.map(([dash, text, length, gap]) => `<button type="button" data-pick-dash="${dash}" aria-pressed="${look.dash === dash}" aria-label="${text}" data-tip="${text}">${dashGlyph(length, gap)}</button>`)), ({ pickDash }) => pickDash !== undefined && pen.setLook("dash", Number(pickDash)));
  else if (part === "text" && drawing) startTyping(drawing, button);
  else if (part === "flip-x" || part === "flip-y") heatmaps.flipDrawings(drawings, part === "flip-x");
  else if (part === "duplicate") heatmaps.duplicateDrawings(drawings);
  else if (part === "remove") heatmaps.removeDrawings(drawings);
  else if (part === "actions") menus.menuOf(button, heatmaps.drawingRows(drawings, button), null, pen.beltSide(), { title: drawing ? pen.drawingNames().get(drawing.id) ?? "" : `${drawings.length} drawings` });
});
document.addEventListener("keydown", (event) => {
  const target = /** @type {HTMLElement} */ (event.target), grid = /** @type {HTMLElement | null} */ (target.closest?.(".swatch-grid"));
  const step = grid && { ArrowLeft: -1, ArrowRight: 1, ArrowUp: -1, ArrowDown: 1 }[event.key];
  if (!grid || !step) return;
  event.preventDefault();
  event.stopPropagation();
  const swatches = all$("button", grid), at = swatches.indexOf(target), columns = getComputedStyle(grid).gridTemplateColumns.split(" ").length;
  const next = swatches[at + step * (upDown(event) ? columns : 1)];
  if (next) next.focus();
  else if (event.key === "ArrowDown") /** @type {HTMLElement | undefined} */ (all$("#menu button").find((button) => grid.compareDocumentPosition(button) & Node.DOCUMENT_POSITION_FOLLOWING && !grid.contains(button)))?.focus();
}, true);
withRest(() => {
  inspector.sliders($("menu"), { valueOf: (id) => ((look) => (look ? SLIDERS[id].of(look) : undefined))(pen.lookFor(id === "draw-width" ? "width" : "opacity")), set: (id, value) => pen.setDrawSlider(/** @type {"draw-width" | "draw-opacity"} */ (id), value) });
});
$("belt").addEventListener("pointerdown", (event) => {
  const belt = $("belt");
  if (!closest(event, ".grip") || event.button !== 0) return;
  event.preventDefault();
  const box = belt.getBoundingClientRect(), canvas = $("canvas").getBoundingClientRect(), offset = { x: event.clientX - box.left, y: event.clientY - box.top };
  belt.classList.add("held");
  belt.setPointerCapture(event.pointerId);
  let trail = [{ x: event.clientX, y: event.clientY, at: performance.now() }];
  const carry = (/** @type {PointerEvent} */ at) => {
    const now = performance.now();
    trail = [...trail.filter((point) => now - point.at <= 100), { x: at.clientX, y: at.clientY, at: now }];
    Object.assign(belt.style, { left: `${at.clientX - offset.x - canvas.left}px`, top: `${at.clientY - offset.y - canvas.top}px` });
    const plot = plotBox();
    if (plot) pen.turnBelt(pen.upright(pen.sideAt(at.clientX - canvas.left, at.clientY - canvas.top, plot)));
  };
  const end = (/** @type {PointerEvent | null} */ at) => {
    belt.classList.remove("held");
    belt.getAnimations({ subtree: true }).forEach((animation) => animation.finish());
    const plot = plotBox();
    if (plot && at) {
      const thrown = (/** @type {number} */ from, /** @type {(point: { x: number, y: number }) => number} */ along) => grid.restOf(from, grid.speedOf(trail, along)) - from;
      const by = { x: thrown(at.clientX, (point) => point.x), y: thrown(at.clientY, (point) => point.y) };
      const now = belt.getBoundingClientRect(), side = pen.sideAt(at.clientX + by.x - canvas.left, at.clientY + by.y - canvas.top, plot);
      const middle = pen.upright(side) ? (now.top + by.y + now.height / 2 - canvas.top - plot.top) / plot.height : (now.left + by.x + now.width / 2 - canvas.left - plot.left) / plot.width;
      Object.assign(prefs.belt, { dock: side, at: clamp(middle, 0, 1) });
      savePrefs();
    }
    pen.turnBelt(pen.upright(prefs.belt.dock));
    pen.placeBelt(true);
  };
  press(event, { captor: belt, move: carry, end });
});

withRest(() => {
  document.addEventListener("visibilitychange", () => document.hidden && [...heatmaps.heatWaiters].forEach((wake) => wake()));
});
/** @type {{ pane: number, y: number } | null} */
let heatAt = null;
/** @type {{ layer: string, cohort: string } | null} */
let profileLit = null;
/** @param {{ layer: string, cohort: string } | null} hit */
function pointProfile(hit) {
  const was = profileLit;
  if (was?.layer === hit?.layer && was?.cohort === hit?.cohort) return;
  profileLit = hit;
  for (const id of new Set([was?.layer, hit?.layer])) if (id) drawn.get(id)?.redraw();
  paintProfileItems();
  valuesSoon();
}

let swallow = false;
let gesturePointer = -1, gestureTouch = false;
function take(event) {
  event.preventDefault();
  event.stopPropagation();
  if (document.activeElement instanceof HTMLElement && !$("chart").contains(document.activeElement)) document.activeElement.blur();
  swallow = true;
  gesturePointer = event.pointerId;
  gestureTouch = event.pointerType === "touch";
  $("chart").setPointerCapture(event.pointerId);
  stopGesture();
  stopGesture = press(event, { captor: $("chart"), end: gestureEnded });
}
let stopGesture = () => {};
/** @param {PointerEvent | null} release */
function gestureEnded(release) {
  swallow = false;
  if (magnifying && lensAt?.touch) lookAt(null);
  if (!release) return stopDrawing();
  if (placing?.held) placed(release);
  else if (moving) moved();
  else if (boxing) boxed();
}
for (const type of ["mousedown", "touchstart"]) $("chart").addEventListener(type, (event) => {
  if (!swallow) return;
  swallow = false;
  event.preventDefault();
  event.stopPropagation();
}, { capture: true, passive: false });
// A mouse's by its mousemove: the chart's own comes before it and asks for its frame first, so the lens drawn after shows that frame, not the one before.
addEventListener("mousemove", (event) => (pointerAt = { x: event.clientX, y: event.clientY }), { passive: true });
addEventListener("mousemove", (event) => magnifying && !lensAt?.touch && event.target instanceof Node && $("chart").contains(event.target) && lookAt(closest(event, ".legend, button") ? null : { x: event.clientX, y: event.clientY, touch: false }));
$("chart").addEventListener("pointermove", (event) => lensAt?.touch && event.pointerId === gesturePointer && lookAt({ x: event.clientX, y: event.clientY, touch: true }));
$("chart").addEventListener("pointerleave", (event) => magnifying && event.pointerType === "mouse" && !lensAt?.touch && lookAt(null));
$("chart").addEventListener("dblclick", () => lensAt && lookAt(lensAt));
$("chart").addEventListener("wheel", () => lensAt && lookAt(lensAt), { passive: true });
withRest(() => {
  $("chart").addEventListener("pointerdown", (event) => {
    press(event, {
      end: (release) => {
        if (!release && !heatmaps.gesturing()) stopDrawing();
      },
    });
    if (heatmaps.otherPointer(event)) {
      event.preventDefault();
      event.stopPropagation();
      swallow = true;
      if (moving && gestureTouch && event.pointerType === "touch") stopDrawing();
      return;
    }
    if (event.pointerType === "touch" && [...fingers].some((id) => id !== event.pointerId)) return;
    if (fingerCross.dismissing && !tool) return;
    if (writing) $("draw-text").blur();
    if (event.button !== 0 || !resolved?.times.length || closest(event, ".legend, #full")) return;
    if (magnifying && event.pointerType !== "mouse") return void (take(event), lookAt({ x: event.clientX, y: event.clientY, touch: true }));
    const pane = paneAt(event.clientY);
    if (!pane) return;
    const at = pen.plotPoint(event.clientX, event.clientY, pane.rect);
    if (at.x < 0 || at.x > engine.time.width) return;
    if (!tool && (selectingArea || pen.boxKey(event))) {
      take(event);
      hideHints();
      boxing = { pane: pane.index, rect: pane.rect, start: at, now: at };
      return heatmaps.showBox();
    }
    if (tool) {
      if (placing) {
        take(event);
        Object.assign(placing, { rect: paneRect(placing.host) ?? placing.rect, down: at, held: true, keys: false });
        return placeTo(event);
      }
      const host = pen.hostIn(pane.index), point = host && pen.pointFrom(at, host, { free: tool === "brush" || tool === "text", pull: (POINTED.has(tool) && tool !== "ruler") || tool === "hline" ? !event.altKey : false });
      if (!host || !point) return;
      take(event);
      placing = { rect: pane.rect, host, down: at, held: true, clicks: false, stroke: [{ ...at, point }] };
      pathResume = null;
      if (tool !== "text") draft = pen.newDrawing(tool, host.id, POINTED.has(tool) ? [point, [...point]] : [point]);
      return repaintDrawings();
    }
    const hit = pen.drawingAt(event.clientX, event.clientY, pen.reachOf(event));
    if (!hit) return void (chosen.size && !event.shiftKey && heatmaps.pressAt(event, ""));
    // By touch, one not selected is the chart's still: a drag over it pans (a finger's wide reach on it moves nothing);
    // a tap selects it, then it's taken as a pointer takes it. (A pen points as a mouse does.)
    if (event.pointerType === "touch" && !chosen.has(hit.drawing.id)) return void heatmaps.pressAt(event, hit.drawing.id);
    take(event);
    const id = hit.drawing.id;
    if (event.shiftKey) return chooseDrawings(chosen.has(id) ? [...chosen].filter((other) => other !== id) : [...chosen, id]);
    if (!chosen.has(id)) chooseDrawings([id]);
    const reshape = hit.handle >= 0 && (POINTED.has(hit.drawing.kind) || hit.drawing.kind === "brush"), slide = hit.over === "label", group = reshape || slide ? [id] : [...chosen];
    moving = { originals: new Map(group.map((each) => [each, pen.drawingById(each) ?? hit.drawing])), target: id, handle: reshape ? hit.handle : -1, slide, rect: pane.rect, start: at, moved: false, cursor: ["nwse", "nesw", "ns", "ew"].includes(hit.over) ? `${hit.over}-resize` : "grabbing" };
  }, true);
  $("chart").addEventListener("pointermove", (event) => {
    if (heatmaps.otherPointer(event)) return;
    if (placing) return void (placing.keys || placeTo(event));
    if (moving) return heatmaps.moveTo(event);
    if (boxing) return heatmaps.boxTo(event);
    hover(event.buttons || tool || selectingArea ? null : pen.drawingAt(event.clientX, event.clientY));
  });
  $("chart").addEventListener("pointerleave", () => !placing && !moving && hover(null));
  $("chart").addEventListener("dblclick", (event) => {
    const hit = tool ? null : pen.drawingAt(event.clientX, event.clientY);
    if (!hit) return;
    event.stopPropagation();
    if (hit.drawing.kind === "text") return startTyping(hit.drawing);
    if (!LINES.has(hit.drawing.kind)) return;
    const pane = paneAt(event.clientY), series = drawn.get(hit.drawing.layer), line = timeLine(), size = sketches.get(series)?.size;
    if (hit.over === "label" || !pane || !series || !line || !size) return startLabel(hit.drawing);
    const at = pen.plotPoint(event.clientX, event.clientY, pane.rect);
    startLabel(hit.drawing, pen.shareAlong(hit.drawing, onScreen(hit.drawing, series, line), size, at.x, at.y));
  }, true);
  $("chart").addEventListener("contextmenu", (event) => {
    const hit = tool || placing ? null : pen.drawingAt(event.clientX, event.clientY);
    const item = !hit && !tool && !placing ? /** @type {HTMLElement | null} */ (closest(event, "#chart .legend [data-select]")) : null;
    const layer = item?.dataset.select ?? (!hit && !tool && !placing && litLayer && layerById(litLayer) ? litLayer : null);
    if (layer && !TOUCH) return void (event.preventDefault(), inspector.layerMenu(item ?? $("plot-keys"), layer, { x: event.clientX, y: event.clientY }));
    if (!hit) return;
    event.preventDefault();
    if (!chosen.has(hit.drawing.id)) chooseDrawings([hit.drawing.id]);
    heatmaps.drawingMenu({ x: event.clientX, y: event.clientY });
  });
});
function stopDrawing() {
  if (!placing && !moving && !boxing) return;
  placing = moving = draft = boxing = null;
  pathTaken = [];
  keyValue = null;
  reshaped = new Map();
  $("select-box").hidden = true;
  heatmaps.hold("");
  renderBelt();
  repaintDrawings();
}
function placeTo(event) {
  const host = placing.host;
  if (tool === "brush" && draft) {
    const coalesced = event.getCoalescedEvents?.() ?? [];
    for (const each of coalesced.length ? coalesced : [event]) {
      const raw = pen.plotPoint(each.clientX, each.clientY, placing.rect), last = placing.stroke[placing.stroke.length - 1];
      const x = (last.x + raw.x) / 2, y = (last.y + raw.y) / 2;
      if (Math.hypot(x - last.x, y - last.y) < 1) continue;
      const point = pen.pointFrom({ x, y }, host, { free: true });
      placing.stroke.push({ x, y, point });
      if (point) draft.points.push(point);
    }
    return repaintDrawings();
  }
  const point = draft && pen.pointFrom(pen.plotPoint(event.clientX, event.clientY, placing.rect), host, { pull: tool !== "vline" && tool !== "ruler" && !event.altKey });
  if (!point || !draft) return;
  draft = { ...draft, points: [...draft.points.slice(0, -1), point] };
  repaintDrawings();
}
function placed(event) {
  const at = pen.plotPoint(event.clientX, event.clientY, placing.rect), still = Math.hypot(at.x - placing.down.x, at.y - placing.down.y) < 4;
  if (tool === "path" && draft) {
    if (placing.clicks) return void (Object.assign(placing, { held: false }), placePathPoint(event.pointerType === "touch" ? 16 : 4));
    Object.assign(placing, { clicks: true, held: false });
    if (!still) draft = { ...draft, points: [...draft.points, [...(draft.points.at(-1) ?? [0, 0])]] };
    return heatmaps.pathMoved();
  }
  if (PAIRED.has(tool) && still && !placing.clicks) {
    Object.assign(placing, { clicks: true, held: false });
    return renderBelt();
  }
  const { host, stroke } = placing, kind = tool, again = toolKept;
  let drawing = draft;
  takeUp(again ? kind : "", again, true);
  if (kind === "text") {
    const point = pen.pointFrom(at, host, { free: true });
    return void (point && startTyping(pen.newDrawing("text", host.id, [point])));
  }
  if (kind === "brush" && drawing) {
    stroke.push({ ...at, point: pen.pointFrom(at, host, { free: true }) });
    let kept = heatmaps.simplify(stroke, 0.75);
    for (let tolerance = 1.5; kept.length > MAX_STROKE; tolerance *= 1.5) kept = heatmaps.simplify(stroke, tolerance);
    drawing = { ...drawing, points: /** @type {[number, number][]} */ (kept.map((each) => each.point).filter(Boolean)) };
  }
  if (!drawing) return repaintDrawings();
  keepDrawing(drawing);
}
function placePathPoint(reach = 4) {
  if (!draft) return;
  const placed = /** @type {[number, number]} */ (draft.points.at(-1)), before = draft.points.at(-2);
  if (before && !heatmaps.pointsApart({ ...draft, points: [before, placed] }, reach)) return void heatmaps.finishPath();
  pathTaken = [];
  draft = { ...draft, points: [...draft.points, [...placed]] };
  if (draft.points.length > MAX_STROKE) return void (heatmaps.finishPath(), notice(`A path holds up to ${MAX_STROKE} points`));
  heatmaps.pathMoved();
}
/** @type {[number, number][]} */
let pathTaken = [];
/** @type {{ placing: any, draft: Drawing, taken: [number, number][] } | null} */
let pathResume = null;
function unplacePath() {
  if (!placingPath() || !draft) return false;
  if (draft.points.length <= 2) {
    const resume = { placing, draft, taken: [...pathTaken, /** @type {[number, number]} */ (draft.points[0])] };
    stopDrawing();
    pathResume = resume;
    syncUndo();
    return void ($("chart-said").textContent = plural(0, "point")), true;
  }
  pathTaken.push(/** @type {[number, number]} */ (draft.points.at(-2)));
  draft = { ...draft, points: [...draft.points.slice(0, -2), /** @type {[number, number]} */ (draft.points.at(-1))] };
  heatmaps.pathMoved();
  return true;
}
function replacePath() {
  if (pathResume && tool === "path" && !placing) {
    const { draft: was, taken } = pathResume, first = /** @type {[number, number]} */ (taken.pop());
    ({ placing } = pathResume);
    draft = { ...was, points: [first, /** @type {[number, number]} */ (was.points.at(-1))] };
    pathTaken = taken;
    pathResume = null;
    return heatmaps.pathMoved(), true;
  }
  const point = placingPath() ? pathTaken.pop() : undefined;
  if (!point || !draft) return false;
  draft = { ...draft, points: [...draft.points.slice(0, -1), point, /** @type {[number, number]} */ (draft.points.at(-1))] };
  heatmaps.pathMoved();
  return true;
}
function keepDrawing(/** @type {Drawing} */ drawing) {
  if (!drawing.points.length || (PAIRED.has(drawing.kind) && !heatmaps.pointsApart(drawing)) || (drawing.kind === "path" && drawing.points.length < 2)) return repaintDrawings();
  if (drawing.kind === "ruler") return void ((measuring = drawing), repaintDrawings());
  heatmaps.addDrawn(drawing);
}
let undoTook = "", picks = 0;
function moved() {
  const { originals } = moving, now = [...reshaped.values()].filter((drawing) => JSON.stringify(drawing) !== JSON.stringify(originals.get(drawing.id)));
  moving = null;
  reshaped = new Map();
  heatmaps.hold("");
  if (!now.length) return repaintDrawings();
  commit((doc) => doc.drawings?.forEach((item) => {
    const moved = now.find((drawing) => drawing.id === item.id);
    if (moved) Object.assign(item, { points: moved.points, ...(moved.at === undefined ? {} : { at: moved.at }) });
  }));
}
let boxing = null;
function boxed() {
  const { start, now, pane } = boxing, id = chart.panes[pane]?.id, box = { left: Math.min(start.x, now.x), right: Math.max(start.x, now.x), top: Math.min(start.y, now.y), bottom: Math.max(start.y, now.y) };
  boxing = null;
  $("select-box").hidden = true;
  const found = [];
  for (const layer of chart.layers) {
    const sketch = layer.pane === id ? sketches.get(drawn.get(layer.id)) : null;
    if (sketch) for (const { drawing, points } of sketch.fresh().shown) if (heatmaps.inBox(drawing, points, box, sketch.size)) found.push(drawing.id);
  }
  if (selectingArea) takeUp("");
  chooseDrawings(found);
}
/** @param {Drawing} drawing @param {HTMLElement | null} back */
function startTyping(drawing, back = null) {
  const field = /** @type {HTMLTextAreaElement} */ ($("draw-text")), fresh = !pen.drawingById(drawing.id);
  writing = { drawing, fresh, back };
  field.maxLength = 500;
  field.placeholder = "Text";
  field.setAttribute("aria-label", "Text");
  typingIn = drawing.id;
  field.value = drawing.text ?? "";
  heatmaps.dressField(drawing);
  heatmaps.placeField();
  field.focus();
  field.setSelectionRange(field.value.length, field.value.length);
  repaintDrawings();
}
/** @param {Drawing} drawing @param {HTMLElement | null} back */
function startLabel(drawing, at = drawing.at, back = null) {
  if (!chosen.has(drawing.id)) chooseDrawings([drawing.id]);
  const field = /** @type {HTMLTextAreaElement} */ ($("draw-text"));
  writing = { drawing: { ...drawing, ...(at === undefined ? {} : { at }) }, fresh: false, label: true, back };
  field.maxLength = LABEL_MAX;
  field.placeholder = "Label";
  field.setAttribute("aria-label", "Label");
  labelTyping = drawing.id;
  field.value = drawing.label ?? "";
  heatmaps.dressField(drawing);
  heatmaps.placeField();
  field.focus();
  field.select();
  repaintDrawings();
}
function endTyping() {
  if (!writing) return;
  if (writing.cancel) {
    writing = null;
    typingIn = labelTyping = "";
    $("draw-text").hidden = true;
    return repaintDrawings();
  }
  if (writing.label) {
    const { drawing } = writing, label = labelText(/** @type {HTMLTextAreaElement} */ ($("draw-text")).value);
    writing = null;
    labelTyping = "";
    $("draw-text").hidden = true;
    if (label === (drawing.label ?? "") && drawing.at === pen.drawingById(drawing.id)?.at) return repaintDrawings();
    return inspector.editById(drawing.id, (item) => {
      if (label) Object.assign(item, { label, ...(drawing.at === undefined ? {} : { at: drawing.at }) });
      else {
        delete item.label;
        delete item.at;
      }
    });
  }
  const { drawing, fresh } = writing, text = /** @type {HTMLTextAreaElement} */ ($("draw-text")).value.replace(/\s+$/, "");
  writing = null;
  typingIn = "";
  $("draw-text").hidden = true;
  if (!text.trim()) return fresh ? repaintDrawings() : heatmaps.removeDrawing(drawing.id);
  if (text === drawing.text) return repaintDrawings();
  if (fresh) return heatmaps.addDrawn({ ...drawing, text });
  inspector.editById(drawing.id, (item) => (item.text = text));
}
withRest(() => {
  $("draw-text").addEventListener("input", heatmaps.placeField);
});
$("draw-text").addEventListener("blur", () => leftWith($("draw-text"), endTyping) || endTyping());
$("draw-text").addEventListener("keydown", (event) => {
  if (composing(event) || !((event.key === "Enter" && (!event.shiftKey || writing?.label)) || event.key === "Escape")) return;
  event.preventDefault();
  const back = writing?.back;
  if (event.key === "Escape" && writing) writing.cancel = true;
  $("draw-text").blur();
  const to = back?.isConnected ? back : back?.id ? document.getElementById(back.id) : null;
  if (to?.checkVisibility()) to.focus({ preventScroll: true });
});

const GROUP = '.tabs[role="group"], .color-slots[role="group"], [role="radiogroup"]';
const groupButtons = (/** @type {Element} */ group) => /** @type {HTMLButtonElement[]} */ (all$(":scope > button", group)).filter((button) => !button.disabled && !button.hidden);
document.addEventListener("focusin", (event) => {
  const button = /** @type {HTMLElement} */ (event.target), group = button.matches?.("button") ? button.parentElement?.closest(GROUP) : null;
  if (!group || button.parentElement !== group) return;
  const from = /** @type {HTMLElement | null} */ (event.relatedTarget), buttons = groupButtons(group);
  const chosen = buttons.find((other) => other.getAttribute("aria-pressed") === "true" || other.getAttribute("aria-checked") === "true");
  if (from && !group.contains(from) && chosen && chosen !== button && button.matches(":focus-visible")) return chosen.focus();
  for (const other of buttons) other.tabIndex = other === button ? 0 : -1;
});
document.addEventListener("keydown", (event) => {
  const button = /** @type {HTMLElement} */ (document.activeElement), group = button?.parentElement?.closest('.tabs[role="group"], .color-slots');
  if (!group || button.parentElement !== group || event.metaKey || event.ctrlKey || event.altKey) return;
  const step = KEY_STEPS[event.key];
  const along = group.matches(".color-slots.four") && upDown(event) ? 2 : 1;
  if (!step) return;
  const buttons = groupButtons(group), at = buttons.indexOf(/** @type {HTMLButtonElement} */ (button));
  if (at < 0) return;
  event.preventDefault();
  event.stopPropagation();
  buttons[stepIn(at, step * along, buttons.length, true)].focus();
}, true);

/** @type {HTMLElement[]} */
const dialogsShown = [];
const topDialog = () => dialogsShown.at(-1) ?? null;
function say(/** @type {string} */ text) {
  const line = /** @type {HTMLElement | null} */ ((topDialog() ?? document.body).querySelector(":scope > [data-said]"));
  if (!line) return;
  line.textContent = "";
  requestAnimationFrame(() => (line.textContent = text));
}
const scrolledBy = (/** @type {HTMLElement} */ view, /** @type {number} */ top) => view.style.setProperty("--scrolled", String(Math.min(1, Math.max(0, top) / 16)));
for (const view of all$(".panel-view")) view.addEventListener("scroll", () => scrolledBy(view, view.scrollTop), { passive: true });
$("layer-list").addEventListener("scroll", () => scrolledBy($("layers-view"), $("layer-list").scrollTop), { passive: true });
for (const host of [document.body, ...all$("dialog")]) {
  const said = document.createElement("p");
  Object.assign(said, { className: "vh" });
  said.setAttribute("role", "status");
  said.dataset.said = "";
  host.append(said);
}
const listWritten = new WeakMap();
/** @param {HTMLElement} container */
const listRows = (container) => new Map((listWritten.get(container) ?? []).map(([key], i) => [/** @type {HTMLElement} */ (container.children[i]), key]));
/** @param {HTMLElement} container */
const listPlaces = (container) => ((rows) => grid.placesOf([...rows.keys()], (row) => rows.get(row)))(listRows(container));
/** @param {HTMLElement} list @param {Map<string | undefined, number>} was */
function glideList(list, was) {
  const sight = list.getBoundingClientRect(), after = listRows(list);
  const seen = (/** @type {number | undefined} */ top) => top !== undefined && top < sight.bottom && top > sight.top - 60;
  grid.slide([...after.keys()].filter((row) => seen(row.getBoundingClientRect().top) || seen(was.get(after.get(row)))), (row) => after.get(row), was, { fade: true, layout: true });
}
/** @param {HTMLElement} container @param {[string, string][]} parts */
function writeList(container, parts) {
  const was = listWritten.get(container);
  listWritten.set(container, parts);
  const focus = focusKey(container);
  const unique = (/** @type {[string, string][]} */ list) => new Set(list.map(([key]) => key)).size === list.length;
  const kept = new Map(was && container.children.length === was.length && unique(was) && unique(parts) ? was.map(([key, html], i) => [key, { html, element: /** @type {Element} */ (container.children[i]) }]) : []);
  if (!kept.size) {
    container.innerHTML = parts.map(([, html]) => html).join("");
    restoreFocus(container, focus);
    return true;
  }
  const make = document.createElement("template");
  let wrote = false;
  const keys = new Set(parts.map(([key]) => key));
  for (const [key, { element }] of kept) if (!keys.has(key)) element.remove(), kept.delete(key), (wrote = true);
  parts.forEach(([key, html], i) => {
    const old = kept.get(key);
    kept.delete(key);
    let element = old && !(/** @type {HTMLElement} */ (old.element).inert && foldOf.get(old.element)?.playState !== "running") && (old.html === html || rewriteInPlace(old.element, old.html, html)) ? old.element : null;
    if (old && element && old.html !== html) wrote = true;
    if (!element) {
      make.innerHTML = html;
      element = /** @type {Element} */ (make.content.firstElementChild);
      if (old) old.element.remove();
      wrote = true;
    }
    if (container.children[i] !== element) {
      const to = container.children[i] ?? null, move = /** @type {any} */ (container).moveBefore;
      if (move && element.isConnected) move.call(container, element, to);
      else container.insertBefore(element, to);
      wrote = true;
    }
  });
  for (const { element } of kept.values()) element.remove(), (wrote = true);
  if (wrote) restoreFocus(container, focus);
  return wrote;
}

/** @param {Element} element @param {string} was @param {string} html */
function rewriteInPlace(element, was, html) {
  const read = (/** @type {string} */ markup) => ((make) => ((make.innerHTML = markup), make.content.firstElementChild))(document.createElement("template"));
  return patchNode(element, read(was), read(html));
}
/** @param {ChildNode} live @param {ChildNode | null} a @param {ChildNode | null} b @returns {boolean} */
function patchNode(live, a, b) {
  if (!a || !b || live.nodeName !== a.nodeName || a.nodeName !== b.nodeName) return false;
  if (a.isEqualNode(b)) return true;
  if (!(a instanceof Element && b instanceof Element && live instanceof Element)) return ((live.nodeValue = b.nodeValue), true);
  const from = [...a.childNodes], to = [...b.childNodes], now = [...live.childNodes];
  if (now.length !== from.length) return false;
  for (const { name } of [...live.attributes]) if (!b.hasAttribute(name)) live.removeAttribute(name);
  for (const { name, value } of [...b.attributes]) if (live.getAttribute(name) !== value) live.setAttribute(name, value);
  to.forEach((node, i) => (i < now.length ? patchNode(now[i], from[i], node) || live.replaceChild(node, now[i]) : live.append(node)));
  for (const node of now.slice(to.length)) node.remove();
  return true;
}

/** @typedef {{ washes: [number, string, string, number, number][], dots: [string, number, number][], lines: ([string, (number | null)[]] | [string, (number | null)[], string, string, number] | [string, (number | null)[], string | null, string | null, number | null, (string | null)[]])[], marks: Record<string, (number | null)[]>, bases?: Record<string, number> }} Thumb */

const THUMBS = "bitview-studio:thumbs";
/** The presets' marks kept in this browser, the last drawn: a visit shows them at once, not drawn anew first; let go of first, as the thumbs are, when storage is full. */
const PRESET_MARKS = "bitview-studio:preset-marks";
const THUMB_POINTS = 28;
/** The drawings' version: one kept from before it (drawn by older rules) isn't reused. Drawings only, never saved charts or preferences. */
const MARK_VERSION = "4:";
const validDrawingKey = (key) => typeof key === "string" && key.startsWith(MARK_VERSION);
const validYs = (ys) => Array.isArray(ys) && ys.length >= 2 && ys.length <= THUMB_POINTS && ys.every((y) => y === null || (typeof y === "number" && y >= 0 && y <= 1));
const validLevel = (y) => typeof y === "number" && y >= 0 && y <= 1;
const MARK_KINDS = new Set(["area", "baseline", "bars", "dots"]);
const validLines = (/** @type {any} */ lines) => Array.isArray(lines) && lines.length <= 3
  && lines.every((line) => Array.isArray(line) && isColor(line[0]) && validYs(line[1]) && (line[2] == null || (MARK_KINDS.has(line[2]) && isColor(line[3]) && validLevel(line[4])))
    && (line[5] === undefined || (Array.isArray(line[5]) && line[5].length === line[1].length && line[5].every((color) => color === null || isColor(color)))));
const validChartMark = (/** @type {any} */ mark) => typeof mark === "object" && mark !== null && validLines(mark.lines)
  && Array.isArray(mark.washes) && mark.washes.length <= 1 && mark.washes.every((wash) => Array.isArray(wash) && Number.isInteger(wash[0]) && wash[0] >= 0 && wash[0] < mark.lines.length && isColor(wash[1]) && isColor(wash[2]) && validLevel(wash[3]) && validLevel(wash[4]))
  && Array.isArray(mark.dots) && mark.dots.length <= 4 && mark.dots.every((dot) => Array.isArray(dot) && isColor(dot[0]) && validLevel(dot[1]) && validLevel(dot[2]));
const validThumb = (thumb) => validDrawingKey(thumb?.key) && validChartMark(thumb)
  && typeof thumb.marks === "object" && thumb.marks !== null && Object.values(thumb.marks).every(validYs)
  && (thumb.bases === undefined || (typeof thumb.bases === "object" && thumb.bases !== null && Object.values(thumb.bases).every(validLevel)));

const PREVIEWS = "bitview-studio:previews";
/**
 * @typedef {{ k: string, c: string, c2?: string, d: string, d2?: string, y0?: number, top?: number, bottom?: number, up?: boolean, band?: string, f?: number[], dash?: string, op?: number, end?: number[], img?: string, o?: number, w?: number, z?: [number, string][] }} PlotMark
 *   Quick Look's (a big one's) too: its dates along its foot (`ticks`, where), each pane side's highest and lowest
 *   value (`scales`, where). A preset's: which preset it is, by its place in their tree (`of`, see templateDoc).
 * @typedef {{ seps: number[], marks: PlotMark[], end?: number, ticks?: { x: number, text: string }[], scales?: { y: number, text: string, side: string }[], of?: string }} Preview
 */
const PLOT_NEEDS = { line: [], dots: [], bars: [], across: [], band: [], bubble: ["w", "top", "bottom"], area: ["top", "bottom"], baseline: ["y0", "top", "bottom"], heat: ["img", "top", "bottom"], profile: ["f", "top", "bottom"] };
const isPath = (d) => typeof d === "string" && /^[MLHVZh\d. -]*$/.test(d);
const validMark = (mark) => Object.hasOwn(PLOT_NEEDS, mark?.k) && PLOT_NEEDS[mark.k].every((field) => mark[field] !== undefined) && isColor(mark.c) && (mark.c2 === undefined || isColor(mark.c2)) && isPath(mark.d)
  && (mark.d2 === undefined || isPath(mark.d2)) && [mark.y0, mark.top, mark.bottom, mark.op, mark.w].every((n) => n === undefined || Number.isFinite(n))
  && (mark.up === undefined || typeof mark.up === "boolean") && (mark.f === undefined || (Array.isArray(mark.f) && mark.f.every(Number.isFinite))) && (mark.dash === undefined || (typeof mark.dash === "string" && /^[\d. ]+$/.test(mark.dash)))
  && (mark.end === undefined || (Array.isArray(mark.end) && mark.end.length === 2 && mark.end.every(Number.isFinite)))
  && (mark.img === undefined || (typeof mark.img === "string" && /^data:image\/png;base64,[A-Za-z0-9+/=]+$/.test(mark.img)));
const validPreview = (item) => validDrawingKey(item?.key) && Array.isArray(item.seps) && item.seps.every(Number.isFinite) && Array.isArray(item.marks) && item.marks.every(validMark);
const SHELF = 12, GALLERY_PAGE = 60;
/** The presets' rows as the sidebar last drew them, open, and the sections open then (see presetParts): the first to go when storage is full, as a picture's are. */
const SIDE_PRESETS = "bitview-studio:side-presets";
const USED = "bitview-studio:used";
/** @type {Map<string, Thumb & { key: string }>} */
const thumbs = new Map(Object.entries(storage.get(THUMBS, {})).filter(([, thumb]) => validThumb(thumb)));
const saveThumbsSoon = writeSoon(() => grid.saveThumbs());
const drawThumbs = serial(async () => {
  if (!grid) return void withRest(drawThumbs);
  for (const id of [...thumbs.keys()]) if (grid.orphan(id)) thumbs.delete(id);
  let drew = false;
  for (const doc of [...library.charts, ...previewCharts, ...(chart?.preset !== undefined ? [chart] : [])]) drew = (await grid.freshThumb(doc)) || drew;
  if (drew) saveThumbsSoon();
});
addEventListener("storage", (event) => {
  if (event.key !== THUMBS || !event.newValue) return;
  let stored;
  try {
    stored = JSON.parse(event.newValue);
  } catch {
    return;
  }
  for (const [id, thumb] of Object.entries(stored ?? {})) {
    const doc = chartById(id);
    if (!doc || !validThumb(thumb) || thumb.key !== grid.thumbKey(doc) || thumbs.get(id)?.key === thumb.key) continue;
    thumbs.set(id, thumb);
    grid.updateMark(doc);
  }
});
/** @type {Map<string, Preview & { key: string }>} */
const previews = new Map(Object.entries(storage.get(PREVIEWS, {})).filter(([, item]) => validPreview(item)));
const savePreviewsSoon = writeSoon(() => grid.savePreviews());
/** @param {number} n */
const logRows = (n) => ({ at: (/** @type {number} */ price) => n * Math.log10(price * 100), cell: (/** @type {number} */ price) => Math.floor(n * Math.log10(price * 100)), price: (/** @type {number} */ row) => 10 ** (row / n) / 100, floor: (/** @type {number} */ price) => Math.round(n * Math.log10(price * 100)) });
const HEAT_PICTURE_ROWS = 50, HEAT_PICTURE_AGG = `log${HEAT_PICTURE_ROWS}`, HEAT_PICTURE_GRID = logRows(HEAT_PICTURE_ROWS), HEAT_TILE_COLUMNS = 32, HEAT_QUICK_COLUMNS = 96;
function renderHome() {
  if (!home) return;
  if ("folder" in homeAt && homeAt.folder && !folderById(homeAt.folder)) {
    placeHome({ folder: "" });
    leftPlace = null;
    history.replaceState(homeState(), "");
  }
  if ("path" in homeAt && presetTree() && !presetNodeAt(homeAt.path)) {
    const path = [...homeAt.path];
    while (path.length && !presetNodeAt(path)) path.pop();
    placeHome({ path });
    leftPlace = null;
    history.replaceState(homeState(), "");
  }
  grid.renderHomeHead();
  grid.renderDoors();
  grid.renderHorizon();
  grid.renderYours();
  grid.renderGallery();
  grid.giveUpUnframed();
  const query = $("home-filter").value.trim();
  $("home-count").textContent = query ? `${grouped.format(grid.yoursFound + grid.galleryFound)} found` : "";
}
$("chart-path").addEventListener("click", (/** @type {MouseEvent} */ event) => {
  const anchor = closest(event, "button");
  if (anchor) menus.menuOf(anchor, grid.chartPathRows());
});
const yourTilesSeen = new IntersectionObserver((entries) => {
  for (const entry of entries) (entry.isIntersecting ? grid.yoursSeen.add(entry.target) : grid.yoursSeen.delete(entry.target)), entry.target.toggleAttribute("data-seen", entry.isIntersecting);
  grid.queueYours();
});
let yoursStill = () => true, galleryStill = yoursStill;
withRest(() => ((yoursStill = grid.stillness()), (galleryStill = grid.stillness()), grid.startGalleryRest()));
const drawYours = serial(async () => {
  for (const id of grid.yourQueue) {
    grid.yourQueue.delete(id);
    const doc = chartById(id) ?? [...grid.presetTiles.keys()].find((tile) => tile.id === id);
    if (!home || !doc?.layers.length) continue;
    const drawn = grid.drawPreview(doc, grid.presetTiles.has(doc) ? grid.previewIndex(doc) : doc.index);
    if (!yoursStill()) await drawn;
  }
});
/** @type {HTMLElement | null} */
let pressedTile = null;
$("home").addEventListener("click", (event) => (pressedTile = closest(event, ".tile[data-chart]") ?? pressedTile), true);

let homeAt = asPlace(history.state?.at ?? prefs.at);
/** @param {Place} place */
const placeHome = (place) => {
  homeAt = place;
  reachAt = -1;
  prefs.at = place;
  savePrefs();
};
const homeState = (/** @type {string | undefined} */ key = history.state?.home ? history.state.key : undefined) => ({ home: true, at: homeAt, key: (shownKey = key ?? uid()) });
let shownKey = /** @type {string | undefined} */ (history.state?.key);
function goTo(/** @type {Place} */ place) {
  quickHide();
  const same = JSON.stringify(place) === JSON.stringify(homeAt);
  // (Where the page was, read before its copy's put in (after, it'd be laid out at once for it); at its top already,
  // not put there again (a scroll written lays the page out first). Above it (Safari's bounce), not there.)
  const left = home && !same ? galleryPlace() : null, scrolledDown = !home || homeScroll().scrollTop !== 0;
  if (home && !same) grid.pageOut();
  /** @type {HTMLInputElement} */ ($("home-filter")).value = "";
  if (!same) {
    if (left) {
      history.replaceState({ ...homeState(), place: left }, "");
      grid.leftPlaces.set(history.state.key, left);
    }
    placeHome(place);
    galleryShown = GALLERY_PAGE;
    grid.clearPicks();
  }
  if (scrolledDown) homeScroll().scrollTop = 0;
  if (!home) return showHome();
  if (!same) history.pushState(homeState(uid()), "");
  const fromPage = $("home").contains(document.activeElement);
  renderHome();
  if (!same) grid.pageIn();
  if (scrolledDown) homeScroll().scrollTop = 0;
  if (fromPage && !$("home").contains(document.activeElement)) $("home-title").focus({ preventScroll: true });
}
let galleryShown = GALLERY_PAGE;
$("home").addEventListener("scroll", (event) => /** @type {HTMLElement} */ (event.target).matches?.(".shelf-row") && grid.shelfSteps(/** @type {HTMLElement} */ (event.target)), { capture: true, passive: true });
// (A keyboard's focus come into it (Tab, the arrows, a menu's way back to its tile), for the keys to reach each; not
// one put there by a click's or a tap's way back from its chart: that wrote its row whole, 15 ms on a desktop.)
$("home").addEventListener("focusin", (event) => ((row) => row && /** @type {HTMLElement} */ (event.target).matches(":focus-visible") && grid.fillRow(row))(closest(event, ".shelf-row, .grid-tiles")));
{
  let size = "";
  new ResizeObserver(([entry]) => {
    const { inlineSize, blockSize } = entry.contentBoxSize[0], now = `${inlineSize} ${blockSize}`;
    if (!inlineSize || now === size) return;
    size = now;
    grid?.allShelfSteps();
  }).observe(/** @type {HTMLElement} */ ($("home").querySelector(".home-scroll")));
}
const galleryTilesSeen = new IntersectionObserver((entries) => {
  for (const entry of entries) (entry.isIntersecting ? grid.gallerySeen.add(entry.target) : grid.gallerySeen.delete(entry.target)), entry.target.toggleAttribute("data-seen", entry.isIntersecting);
  grid.galleryQueue.clear();
  for (const frame of grid.gallerySeen) if (frame.classList.contains("pending")) grid.galleryQueue.add(grid.presetOf(frame));
  drawGallery();
});
const galleryAhead = /** @type {Set<Element>} */ (new Set()), homeScroller = /** @type {HTMLElement} */ ($("home").querySelector(".home-scroll"));
const aheadOf = (/** @type {number} */ height) => new IntersectionObserver((entries) => {
  for (const entry of entries) entry.isIntersecting ? galleryAhead.add(entry.target) : galleryAhead.delete(entry.target);
  drawGallery();
}, { root: homeScroller, rootMargin: `0px 0px ${Math.round(height)}px 0px` });
let galleryTilesAhead = aheadOf(homeScroller.clientHeight || innerHeight), aheadHeight = homeScroller.clientHeight;
new ResizeObserver(() => {
  const height = homeScroller.clientHeight;
  if (!height || height === aheadHeight) return;
  aheadHeight = height;
  galleryTilesAhead.disconnect();
  galleryAhead.clear();
  galleryTilesAhead = aheadOf(height);
  for (const frame of all$(".pending[data-plot]", $("gallery"))) galleryTilesAhead.observe(frame);
}).observe(homeScroller);
const drawGallery = serial(async () => {
  const tried = /** @type {Set<number>} */ (new Set());
  for (let i = grid.nextGallery(tried); i >= 0; i = grid.nextGallery(tried)) {
    tried.add(i);
    if (!presets?.[i] || !home || $("gallery").hidden || !$("gallery").querySelector(`[data-preset="${i}"] .pending`)) continue;
    const doc = presetDoc(i), have = previews.get(doc.id);
    for (const tile of all$(`.tile[data-preset="${i}"]:has(> [data-open-preset])`, $("gallery"))) if (tile.querySelector("[data-plot]")?.getAttribute("data-plot") !== doc.id) grid.writeTile(tile, i);
    if (have?.key === grid.previewKey(doc)) grid.paint(doc.id, have, true);
    else {
      const drawn = grid.drawPreview(doc, grid.previewIndex(doc));
      if (![...grid.gallerySeen].some((frame) => grid.presetOf(frame) === i) || !galleryStill()) await grid.galleryWait(drawn);
    }
  }
});
$("home").addEventListener("click", (event) => {
  const to = closest(event, "[data-place]"), step = closest(event, "[data-shelf-step]"), preset = closest(event, "[data-open-preset]");
  if (to) goTo(asPlace(JSON.parse(/** @type {string} */ (to.dataset.place))));
  else if (closest(event, "[data-gallery-more]")) {
    const place = galleryPlace(), from = galleryShown, list = /** @type {HTMLElement} */ (closest(event, "[data-gallery-more]")).previousElementSibling?.getAttribute("data-list");
    galleryShown += GALLERY_PAGE;
    renderHome();
    placeGallery(place);
    const tile = $("gallery").querySelector(`[data-list="${list}"]`)?.children[from];
    /** @type {HTMLElement | null | undefined} */ ((tile?.matches(".tile-slot") ? grid.fillSlot(/** @type {HTMLElement} */ (tile)) : tile)?.querySelector(".tile-open"))?.focus({ preventScroll: true });
  }
  else if (closest(event, "[data-gallery-retry]")) retryPresets();
  else if (closest(event, "[data-search-all]")) {
    const words = /** @type {HTMLInputElement} */ ($("home-filter")).value;
    goTo({ path: [] });
    /** @type {HTMLInputElement} */ ($("home-filter")).value = words;
    renderHome();
    if (!byTouch) $("home-filter").focus({ preventScroll: true });
  }
  else if (step && step.getAttribute("aria-disabled") !== "true") grid.stepShelf(/** @type {HTMLElement} */ (step.closest(".shelf")?.querySelector(".shelf-row")), Number(step.dataset.shelfStep));
  else if (preset) pickPreset(Number(preset.dataset.openPreset));
});

function tileMenu(id, point = /** @type {{ x: number, y: number } | undefined} */ (undefined), anchor = /** @type {HTMLElement} */ (grid.tileOpen(id))) {
  const target = grid.picked.size && !grid.picked.has(id) ? grid.cardOf(id) : null;
  queueMicrotask(() => target && !$("menu").hidden && menus.outlineMenuTarget(target));
  const title = phone.matches && grid.picked.has(id) ? plural(grid.picked.size, "chart") : "";
  menus.menuOf(anchor, grid.picked.has(id) ? grid.gridActions() : command.chartActions(id, "tile"), point ?? null, "", { title });
}
$("home-bar").addEventListener("click", (event) => {
  const button = closest(event, "[data-batch]");
  if (button?.getAttribute("aria-disabled") === "true") notice(/** @type {string} */ (button.dataset.tip));
  else if (button) grid.batch(/** @type {string} */ (button.dataset.batch), button);
});
$("home-grid").addEventListener("click", (event) => {
  const more = closest(event, "[data-tile-menu]");
  if (more) return tileMenu(/** @type {string} */ (more.dataset.tileMenu), undefined, more);
  const tile = closest(event, "[data-home-card]:not(.preview)"), check = closest(event, ".tile-check");
  if (!tile || closest(event, ".tile-rename")) return;
  if (!(check || event.metaKey || event.ctrlKey || event.shiftKey || (grid.picked.size && event.detail > 0))) return;
  event.stopPropagation();
  grid.pickChart(/** @type {string} */ (tile.dataset.homeCard), event.shiftKey);
});
$("home-filter").addEventListener("input", () => renderHome());
$("home-filter").addEventListener("keydown", (event) => {
  if (composing(event)) return;
  if (event.key === "Escape") {
    event.preventDefault();
    if ($("home-filter").value) {
      $("home-filter").value = "";
      renderHome();
    } else menus.letGo($("home-filter"));
  } else if (event.key === "ArrowDown" || event.key === "Enter") {
    const tiles = all$(".tile:not(.waiting) > .tile-open", $("home")).filter((tile) => tile.checkVisibility()), first = tiles[0];
    if (!first) return;
    event.preventDefault();
    if (event.key === "Enter" && tiles.length === 1) first.click();
    else first.focus();
  }
});
let deletedOpen = "";
function showHome(push = true, /** @type {{ top: number, rows: Record<string, number> } | null} */ place = null) {
  /** @type {(() => void) | null} */
  let redrawCard = null;
  gestures.stopCarrying();
  delete document.body.dataset.sheet;
  if (home) return;
  navigations++;
  const focusLeaves = grid.onChart();
  if (toastOf === chart) hideToast();
  presetsEdit.setFull(false);
  leaveField();
  saveView();
  if (settled() && chart.layers.length && resolved && !resolved.failed.length) {
    const key = chart.layers.some((layer) => heatLike(layer) && !layer.hidden) ? "" : grid.previewKey(chart);
    const drawCard = (/** @type {Chart} */ doc, /** @type {Resolved} */ data, /** @type {number} */ revision) => {
      const drawn = { key, ...grid.previewOf(doc, data) };
      previews.set(doc.id, drawn), grid.savePreviews(), cardsDrawn.set(doc.id, revision);
      return drawn;
    };
    if (!key || previews.get(chart.id)?.key !== key) drawCard(chart, resolved, dataRevision);
    else if (cardsDrawn.get(chart.id) !== dataRevision) {
      const doc = chart, data = resolved, revision = dataRevision, kept = previews.get(doc.id);
      redrawCard = () => home && chart === doc && previews.get(doc.id) === kept && grid.previewKey(doc) === key && grid.paint(doc.id, drawCard(doc, data, revision), true);
    }
  }
  const draft = deletedOpen === chart.id || (cancelling === chart.id && previousChart()) || emptyYours(chart) ? chart : null;
  cancelling = deletedOpen = "";
  menus.hideMenu();
  hideTip();
  if (selected) deselect();
  document.body.removeAttribute("data-peek");
  home = true;
  prefs.home = true;
  savePrefs();
  if (push) history.pushState(homeState(uid()), "");
  let focusTo = /** @type {HTMLElement | null} */ (null), focusWas = /** @type {Element | null} */ (null);
  void grid.morph(false, () => {
    document.body.toggleAttribute("data-home", true);
    pen.putDown();
    clearMeasure();
    if ($("toast").classList.contains("visible")) placeToast();
    refresh();
    const card = grid.cardOf(chart.id);
    if (place) placeGallery(place);
    else card?.scrollIntoView({ block: "nearest" });
    const box = card?.getBoundingClientRect(), view = homeScroll().getBoundingClientRect(), seen = Boolean(box && box.bottom > view.top && box.top < view.bottom);
    focusTo = seen ? grid.tileOpen(chart.id) : focusLeaves && (document.activeElement === document.body || grid.onChart()) ? $("home-title") : null;
    focusWas = document.activeElement;
    return draft || !seen ? null : card;
  }).then(() => {
    if (!home) return;
    if (redrawCard) whenIdle(redrawCard);
    $("home").inert = false;
    if (document.activeElement === focusWas || document.activeElement === document.body) (focusTo?.isConnected ? focusTo : focusTo && $("home-title"))?.focus({ preventScroll: true });
    if (draft && chart === draft) openChart(previousChart() ?? newDraft());
  });
}
function leaveHome(show = () => {}, /** @type {HTMLElement | null} */ before = null, push = true) {
  if (!home) return (before?.classList.remove("opening"), show(), Promise.resolve());
  navigations++;
  home = false;
  prefs.home = false;
  savePrefs();
  grid.picked.clear();
  leftPlace = galleryPlace();
  show();
  if (push && history.state?.home) (history.replaceState({ ...history.state, ...homeState(), place: leftPlace }, ""), grid.leftPlaces.set(history.state.key, leftPlace));
  if (push) history.pushState({ chart: chart.id, fromHome: true, place: leftPlace }, "");
  const focused = $("home").contains(document.activeElement);
  return grid.morph(true, () => {
    before?.classList.remove("opening");
    document.body.removeAttribute("data-home");
    if ($("toast").classList.contains("visible")) placeToast();
    if (undrawn) (blockAtHome = false), void draw(true);
    else if (blockAtHome) (blockAtHome = false), void draw(false, true);
    refresh();
    const row = /** @type {HTMLElement | null} */ ($("chart-list").querySelector(`[data-open-chart="${chart.id}"]`));
    if (focused && row?.checkVisibility()) row.focus({ preventScroll: true });
    return before;
  }).then(() => void (home || ($("home").inert = true)));
}
/** @type {{ top: number, rows: Record<string, number>, shown: number } | null} */
let leftPlace = ((place) => (Number.isFinite(place?.top) && place.rows && typeof place.rows === "object" ? place : null))(history.state?.chart ? history.state.place : null);
let cancelling = "";
/** Whether this browser keeps a closing dialog or popover drawn while it leaves (`overlay`; Safari doesn't). */
const keepsLeaving = CSS.supports("overlay: auto");
{
  const watchers = /** @type {Map<string, any>} */ (new Map());
  const watch = (/** @type {string} */ name, /** @type {boolean} */ open, /** @type {() => void} */ close) => {
    if (open && !watchers.has(name) && "CloseWatcher" in window) {
      const watcher = new /** @type {any} */ (window).CloseWatcher();
      watcher.onclose = () => (watchers.delete(name), close());
      watchers.set(name, watcher);
    } else if (!open && watchers.has(name)) {
      const watcher = watchers.get(name);
      watchers.delete(name);
      watcher.destroy();
    }
  };
  let panelOpen = false;
  const closeSheet = () => {
    if ((selected || inspector.editingMany()) && document.body.dataset.sheet === "panel") {
      deselect();
      watch("sheet", true, closeSheet);
    } else delete document.body.dataset.sheet;
  };
  const sync = () => {
    if (panelOpen && document.body.dataset.sheet !== "panel" && (selected || inspector.editingMany())) deselect();
    panelOpen = document.body.dataset.sheet === "panel";
    $("mobile-layers").setAttribute("aria-expanded", String(panelOpen));
    watch("sheet", phone.matches && Boolean(document.body.dataset.sheet), closeSheet);
    watch("menu", phone.matches && !$("menu").hidden && $("menu").classList.contains("as-sheet"), menus.menuGoBack);
  };
  new MutationObserver(sync).observe(document.body, { attributes: true, attributeFilter: ["data-sheet"] });
  new MutationObserver(sync).observe($("menu"), { attributes: true, attributeFilter: ["hidden", "class"] });
  for (const dialog of all$("dialog")) dialog.addEventListener("toggle", () => watch(dialog.id, phone.matches && isOpen(dialog), () => dialog.dispatchEvent(new Event("cancel", { cancelable: true })) && dialog.hidePopover()));
}
addEventListener("popstate", (event) => {
  stillStep(/** @type {PopStateEvent & { hasUAVisualTransition?: boolean }} */ (event).hasUAVisualTransition === true);
  if (home && shownKey && document.body.hasAttribute("data-home")) grid.leftPlaces.set(shownKey, galleryPlace());
  shownKey = event.state?.key;
  if (location.hash !== brokenLink && isLinkHash(location.hash)) return;
  grid.closeDialogs();
  if (event.state?.home) {
    const to = asPlace(event.state.at), place = grid.leftPlaces.get(event.state.key) ?? event.state.place ?? null;
    if (JSON.stringify(to) !== JSON.stringify(homeAt)) {
      galleryShown = Number.isInteger(place?.shown) ? place.shown : GALLERY_PAGE;
      /** @type {HTMLInputElement} */ ($("home-filter")).value = "";
      grid.clearPicks();
    }
    placeHome(to);
    if (!home) return showHome(false, place);
    const fromPage = $("home").contains(document.activeElement);
    grid.pageOut();
    renderHome();
    grid.pageIn();
    if (fromPage && !$("home").contains(document.activeElement)) $("home-title").focus({ preventScroll: true });
    if (place) placeGallery(place);
    else homeScroll().scrollTop = 0;
    return;
  }
  const route = ROUTED ? routeOf(location.pathname) : null, routed = route && "preset" in route ? route.preset : -1;
  if (routed >= 0 && !home && presetIndex(chart) === routed && `${location.pathname}${location.hash}` === `${BASE}${presetLink(chart)}`) return void showAddress();
  const preset = typeof event.state?.chart === "string" ? presetById(event.state.chart) : -1;
  const doc = routed >= 0 ? presetByAddress(routed, route && "window" in route ? route.window : undefined) : chartById(event.state?.chart) ?? (preset >= 0 ? presetChart(preset) : undefined);
  if (!doc) {
    const closed = closedPreviews.get(event.state?.chart);
    if (closed) notice(`“${closed}” was closed${libraryEdits.past.at(-1)?.text === `Closed the preview “${closed}”` ? `. Press ${keyText("⌘Z")} on Home to bring it back` : ""}`);
    return void history.replaceState(home ? homeState() : { chart: chart.id }, "");
  }
  const again = routed >= 0 && doc === chart;
  if (routed >= 0) forgetSpans(doc.id);
  if (history.state?.chart && history.state.chart !== doc.id) history.replaceState({ ...history.state, chart: doc.id }, "");
  if (again) displayed = null;
  grid.go(doc, false);
  if (again) refresh(true);
  showAddress();
});
// (The screen turned or resized mid-slide: each sheet's slide in flight is where it was going at once, laid out for
// the screen it's on, not slid on by the old one's way. One stopped already stays so: finished, it'd be back.)
withRest(() => addEventListener("resize", () => {
  for (const sheet of [$("panel"), $("menu"), ...all$("dialog")]) {
    const slide = grid.slides.get(sheet);
    if (slide?.playState === "running" || slide?.playState === "paused") slide.finish();
  }
}));
withRest(() => {
  grid.heightsSheet($("panel"), {
    pulls: () => phone.matches && document.body.dataset.sheet === "panel",
    grab: (event) => {
      const from = event.clientY - $("panel").getBoundingClientRect().top;
      return phone.matches && document.body.dataset.sheet === "panel" && from <= 56 && !closest(event, "button, input, textarea, a") && (from <= 22 ? "handle" : "head");
    },
    whole: sheetWhole,
    size: grid.panelSize,
    close: () => $("scrim").click(),
    scrim: $("scrim"),
  });
});
$("home-grid").addEventListener("keydown", (event) => {
  const arrow = ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(event.key), focused = /** @type {HTMLElement} */ (document.activeElement);
  if (arrow && event.altKey && !event.metaKey && !event.ctrlKey && focused.matches("[data-open-chart]") && grid.reorderable()) {
    event.preventDefault();
    const id = /** @type {string} */ (focused.dataset.openChart);
    grid.stepChart(id, event.key === "ArrowLeft" || event.key === "ArrowUp" ? -1 : 1);
    return void grid.tileOpen(id)?.focus({ preventScroll: true });
  }
  if (arrow && event.shiftKey && !event.altKey && !event.metaKey && !event.ctrlKey && focused.matches(".tile-open")) {
    event.preventDefault();
    const next = grid.nextCard(focused, event.key);
    if (!next?.matches(".tile-open")) return;
    if (!grid.picked.size) grid.pickChart(/** @type {string} */ (focused.dataset.openChart));
    grid.pickChart(/** @type {string} */ (next.dataset.openChart), true);
    return void next.focus();
  }
  if (event.metaKey || event.ctrlKey || event.altKey || event.shiftKey) return;
  const cards = all$(grid.CARDS, $("home-grid"));
  if ((event.key === " " || event.key.toLowerCase() === "x") && document.activeElement?.matches(".tile-open") && cards.includes(/** @type {HTMLElement} */ (document.activeElement))) {
    event.preventDefault();
    return grid.pickChart(/** @type {string} */ (/** @type {HTMLElement} */ (document.activeElement).dataset.openChart));
  }
  grid.walkCards($("home-grid"), event, cards);
});
$("gallery").addEventListener("keydown", (event) => {
  if (event.metaKey || event.ctrlKey || event.altKey || event.shiftKey) return;
  grid.walkCards($("gallery"), event);
});
function renderCharts() {
  if (!grid) return void withRest(renderCharts);
  const current = home ? null : chart;
  if (phone.matches) listOwed = true;
  else writeSideList(current);
  grid.renderSideNav();
  $("preview-note").hidden = !isPreview(chart) || !chart.layers.length;
  $("preset-note").hidden = chart.preset === undefined || presetIndex(chart) < 0;
  renderChartPath();
  $("preset-reset").hidden = $("preset-note").hidden || !presetEdited(chart);
  $("chart-more").hidden = !home && isPreview(chart) && !chart.layers.length;
  /** @type {HTMLElement} */ ($("mobile-add").querySelector(".label")).textContent = home ? "New chart" : "Add";
  $("mobile-layers").disabled = home || !chart.layers.length;
  if (home) $("mobile-charts").setAttribute("aria-current", "page");
  else $("mobile-charts").removeAttribute("aria-current");
  if (home) $("mobile-add").removeAttribute("aria-haspopup");
  else $("mobile-add").setAttribute("aria-haspopup", "dialog");
  grid.keepNaming();
  renderHome();
  if (presets && !$("empty").hidden && grid.recommendedKey() !== grid.recommendedDrawn) grid.renderRecommended();
}
/** @param {Chart | null} current */
function writeSideList(current) {
  listOwed = false;
  const previewRow = (/** @type {Chart} */ doc) => `<div class="chart-row preview" data-preview-row="${doc.id}">${`<button type="button" class="nav-item chart-item${touched.has(doc) ? " edited" : ""}" data-open-chart="${doc.id}" data-tip="${escapeHtml(titleOf(doc))}: a preview, not in your Library yet (Save${doc === chart ? " or ⌘S" : ""} to keep it${touched.has(doc) ? " with your edits" : ""})">${grid.chartMark(doc)}<span class="label">${escapeHtml(titleOf(doc))}</span></button>${doc !== chart || previousChart() ? `<button type="button" class="cap small chart-more" data-action="close-preview" data-preview="${doc.id}" aria-label="Close the preview ${escapeHtml(titleOf(doc))}" data-tip="Close the preview">${ICONS.close}</button>` : ""}`}</div>`;
  const listed = listedPreviews(), counts = grid.chartCounts(), rail = onRail(), recents = recentCharts();
  const recentOrder = recents.map((item) => item.id).join();
  if (recentDrawn && recentOrder !== recentDrawn) (sideHeld = true), sideLight(null);
  recentDrawn = recentOrder;
  const row = (/** @type {Chart} */ doc, depth = 0) => grid.leafRow({ kind: "chart", id: doc.id, depth, mark: grid.chartMark(doc), label: titleOf(doc), labelCls: doc.title ? "" : "derived", name: titleOf(doc) });
  /** @type {[string, string][]} */
  const parts = [
    /** @type {[string, string]} */ (["charts", `<div class="chart-row action-row" data-action-row="charts" style="--depth: 0"><button id="charts-nav" class="nav-item side-action" type="button" data-action="charts" data-tip="Home"${home && "top" in homeAt ? ' aria-current="page"' : ""}><span data-icon="home">${ICONS.home}</span><span class="label">Home</span></button></div>`]),
    /** @type {[string, string]} */ (["rule", `<div class="side-rule" aria-hidden="true"></div>`]),
    ...(recents.length ? [grid.sideHead("recent", recents.length, false), ...(grid.groupShut("recent") ? [] : recents.map((item) => /** @type {[string, string]} */ ([`recent:${item.id}`, `<div class="chart-row" data-recent-row="${item.id}" style="--depth: 0"><button type="button" class="nav-item chart-item" data-open-recent="${item.id}" data-tip="${escapeHtml(titleOf(item.doc))}">${recentMark(item)}<span class="label">${escapeHtml(titleOf(item.doc))}</span></button></div>`])))] : []),
    ...(listed.length ? [grid.sideHead("previews", listed.length, Boolean(current && listed.includes(current))), ...(grid.groupShut("previews") ? [] : listed.map((doc) => /** @type {[string, string]} */ ([`preview:${doc.id}`, previewRow(doc)])))] : []),
    grid.sideHead("library", library.charts.length, Boolean(current && !isPreview(current) && current.preset === undefined)),
    ...(grid.groupShut("library") ? [] : [
    /** @type {[string, string]} */ (["new", `<div class="chart-row action-row" data-action-row="new" style="--depth: 0"><button type="button" id="new-chart" class="nav-item chart-item side-action" data-action="new-chart" data-tip="New chart" data-kbd="N"><span data-icon="plus">${ICONS.plus}</span><span class="label">New chart</span><kbd>N</kbd></button></div>`]),
    ...grid.sideTree().map(({ item, depth }) => {
      if ("layers" in item) return /** @type {[string, string]} */ ([item.id, row(item, depth)]);
      return /** @type {[string, string]} */ ([`folder:${item.id}`, grid.folderRow({ kind: "folder", id: item.id, depth, closed: item.closed || rail, holds: Boolean(current?.folder && grid.isIn(current.folder, item.id)), rail, mark: folderMark(item), name: item.name, count: counts.get(item.id) ?? 0, place: { folder: item.id } })]);
    })]),
    ...grid.presetParts(),
  ];
  const list = $("chart-list"), drawn = listWritten.get(list), keys = (/** @type {[string, string][]} */ items) => items.map(([key]) => key).join("\n");
  const was = drawn && !grid.sideMoved && !reduced() && keys(drawn) !== keys(parts) ? listPlaces(list) : null;
  if (writeList(list, parts)) fitSoon(list, true);
  if (was) glideList(list, was);
  const keyed = grid.numbered();
  for (const button of all$("[data-kbd]", $("chart-list"))) {
    if (keyed.includes(button) || !/^⌘ [1-9]$/.test(button.dataset.kbd ?? "")) continue;
    button.querySelector(":scope > kbd.mod")?.remove();
    button.removeAttribute("data-kbd");
    button.removeAttribute("aria-keyshortcuts");
  }
  keyed.forEach((button, i) => {
    const key = keyText(`⌘${i + 1}`), shown = button.querySelector(":scope > kbd.mod");
    if (button.dataset.kbd === `⌘ ${i + 1}` && shown?.textContent === key) return;
    button.dataset.kbd = `⌘ ${i + 1}`;
    button.removeAttribute("aria-keyshortcuts");
    (shown ?? button.appendChild(Object.assign(document.createElement("kbd"), { className: "mod" }))).textContent = key;
    shown?.removeAttribute("aria-hidden");
  });
  drawKeys($("chart-list"));
  presetRowsSeen.disconnect();
  grid.presetMarkQueue.clear();
  if (presets) for (const row of all$("[data-preset-row]", $("chart-list"))) if (!presetMarks.has(grid.presetMarkKey(Number(row.dataset.presetRow)))) presetRowsSeen.observe(row);
  if (grid.presetMarksAsked.size) drawPresetMarks();
  const openedPreset = current?.preset !== undefined ? presetIndex(current) : -1;
  for (const button of all$("[data-open-chart], [data-open-preset], [data-open-recent]", $("chart-list"))) {
    const on = button.dataset.openPreset !== undefined ? Number(button.dataset.openPreset) === openedPreset : (button.dataset.openChart ?? button.dataset.openRecent) === current?.id;
    button.closest(".chart-row")?.classList.toggle("active", on);
    if (on === button.hasAttribute("aria-current")) continue;
    if (on) button.setAttribute("aria-current", "page");
    else button.removeAttribute("aria-current");
  }
  if (presetToReveal >= 0 && presetToReveal === openedPreset && $("chart-list").querySelector(`[data-open-preset="${openedPreset}"]`)) (presetToReveal = -1), presetsEdit.revealNearest(openedPreset, /** @type {Chart} */ (current).id);
  grid.roveList();
}
/** @type {Map<string, Pick<Thumb, "lines" | "washes" | "dots">>} */
const presetMarks = new Map(((kept) => (kept && typeof kept === "object" ? Object.entries(kept).filter(([key, mark]) => validDrawingKey(key) && validChartMark(mark)) : []))(storage.get(PRESET_MARKS, {})));
const savePresetMarks = writeSoon(() => storage.set(PRESET_MARKS, Object.fromEntries([...presetMarks].slice(-grid.KEPT_PRESET_MARKS))));
const paletteMarksSeen = new IntersectionObserver((entries) => {
  for (const { target, isIntersecting } of entries) {
    const i = Number(/** @type {HTMLElement} */ (target).dataset.presetMark);
    if (isIntersecting) grid.presetMarksAsked.add(i), grid.paletteMarksAsked.add(i);
    else grid.presetMarksAsked.delete(i), grid.paletteMarksAsked.delete(i);
  }
  drawPresetMarks();
}, { root: $("palette-list") });
const presetRowsSeen = new IntersectionObserver((entries) => {
  for (const { target, isIntersecting } of entries) {
    const i = Number(/** @type {HTMLElement} */ (target).dataset.presetRow);
    if (isIntersecting && !presetMarks.has(grid.presetMarkKey(i))) grid.presetMarkQueue.add(i);
    else grid.presetMarkQueue.delete(i);
  }
  drawPresetMarks();
}, { root: $("chart-list") });
const drawPresetMarks = serial(async () => {
  while (grid.presetMarkQueue.size || grid.presetMarksAsked.size) {
    const [i] = grid.presetMarkQueue.size ? grid.presetMarkQueue : grid.presetMarksAsked;
    grid.presetMarkQueue.delete(i);
    grid.presetMarksAsked.delete(i);
    if (!presets?.[i] || presetMarks.has(grid.presetMarkKey(i))) continue;
    const doc = presetDoc(i), key = grid.presetMarkKey(i);
    const drawing = doc.layers.length ? await grid.drawThumb(structuredClone(doc)).catch(() => null) : grid.NO_MARK;
    if (!drawing) continue;
    presetMarks.set(key, { lines: drawing.lines, washes: drawing.washes, dots: drawing.dots });
    for (const [old] of presetMarks) if (presetMarks.size > 400) presetMarks.delete(old);
    savePresetMarks();
    for (const mark of all$(`[data-preset-mark="${i}"]`)) mark.outerHTML = grid.presetMark(i);
  }
});
let sidePresets = ((kept) => (typeof kept?.open === "string" && Array.isArray(kept.parts) && kept.parts.every((part) => Array.isArray(part) && part.length === 2 && part.every((text) => typeof text === "string")) ? /** @type {{ open: string, parts: [string, string][] }} */ (kept) : null))(storage.get(SIDE_PRESETS, null));
const saveSidePresets = writeSoon(() => storage.set(SIDE_PRESETS, sidePresets));
/** @param {[string, string][]} parts */
function keepSidePresets(parts) {
  const open = JSON.stringify(prefs.presetsOpen);
  if (sidePresets?.open === open && sidePresets.parts.length === parts.length && sidePresets.parts.every(([key, html], i) => parts[i][0] === key && parts[i][1] === html)) return;
  sidePresets = { open, parts };
  saveSidePresets();
}
let recentDrawn = "";
function applyTheme(/** @type {boolean} */ anyway = false) {
  if (!anyway && lightChosen() === light) return void (document.documentElement.dataset.theme = prefs.theme);
  const swap = () => {
    light = lightChosen();
    withRest(() => inspector.pickedAt.clear());
    document.documentElement.dataset.theme = prefs.theme;
    hexCache.clear();
    engine.theme = chartTheme();
    refresh();
    buildNow();
    if (home) renderHome();
    if (presets && !$("empty").hidden) grid.renderRecommended();
    for (const [id, drawn] of previews) grid.paint(id, drawn, true);
    if (quick) quickShow();
    if (isOpen($("palette"))) renderPalette();
    if (writing) heatmaps.dressField(writing.drawing);
  };
  if (document.startViewTransition && !reduced()) document.startViewTransition(swap);
  else swap();
}
deviceLight.addEventListener("change", () => applyTheme());
matchMedia(MEDIA.moreContrast).addEventListener("change", () => applyTheme(true));
addEventListener("storage", ({ key, newValue }) => {
  if (key !== PREFS) return;
  let theirs;
  try {
    theirs = JSON.parse(newValue ?? "");
  } catch {
    return;
  }
  if (!theirs || typeof theirs !== "object") return;
  const read = readPrefs(theirs), took = Object.keys(prefs).filter((name) => name in theirs && JSON.stringify(/** @type {any} */ (read)[name]) !== prefsSeen[name]);
  for (const name of took) {
    /** @type {any} */ (prefs)[name] = /** @type {any} */ (read)[name];
    prefsSeen[name] = JSON.stringify(/** @type {any} */ (read)[name]);
  }
  if (took.includes("theme")) {
    grid.renderTheme();
    applyTheme();
  }
  if (took.includes("sidebar")) (applySidebar(), renderCharts());
  if (took.includes("panel") && !fullOn()) applyPanel();
  if (took.includes("sideWidth") || took.includes("panelWidth")) applyWidths();
  if (took.includes("belt") || took.includes("pen")) renderBelt();
  if (took.includes("keys")) renderSingleKeys();
  if (took.includes("linkView")) (carrySpan(), renderLinkView(), !prefs.linkView && storeView(false));
  if (took.includes("presetsOpen") || took.includes("groupsOpen")) renderCharts();
});
withRest(() => {
  grid.renderTheme();
});
renderLinkView();
/** @param {string} css */
function bezierOf(css) {
  const [a, b, c, d] = (css.match(/-?[\d.]+/g) ?? []).map(Number);
  const at = (/** @type {number} */ p1, /** @type {number} */ p2, /** @type {number} */ s) => 3 * p1 * s * (1 - s) ** 2 + 3 * p2 * s * s * (1 - s) + s ** 3;
  return (/** @type {number} */ t) => {
    let lo = 0, hi = 1;
    for (let k = 0; k < 24; k++) ((s) => (at(a, c, s) < t ? (lo = s) : (hi = s)))((lo + hi) / 2);
    return at(b, d, (lo + hi) / 2);
  };
}
const layoutCurve = bezierOf(MOTION.easeLayout);
function deleteFolder(/** @type {string} */ id) {
  const at = library.folders.findIndex((folder) => folder.id === id), folder = library.folders[at];
  if (!folder) return;
  const members = grid.childrenOf(id), charts = library.charts.filter((doc) => grid.isIn(doc.folder, id)).length, siblings = grid.childrenOf(folder.parent), place = siblings.indexOf(folder);
  library.folders.splice(at, 1);
  library.deleted[id] = Date.now();
  for (const item of members) grid.setParent(item, folder.parent);
  if ("folder" in homeAt && homeAt.folder === id) {
    placeHome({ folder: folder.parent ?? "" });
    leftPlace = null;
    if (home) history.replaceState(homeState(), "");
  }
  grid.setChildren([...siblings.slice(0, place), ...members, ...siblings.slice(place + 1)]);
  save();
  renderCharts();
  const lost = !document.activeElement || document.activeElement === document.body || !document.activeElement.isConnected;
  const next = members[0] ?? siblings[place + 1] ?? siblings[place - 1] ?? (folder.parent ? folderById(folder.parent) : undefined);
  if (lost && next) /** @type {HTMLElement | null} */ ($("chart-list").querySelector("layers" in next ? `[data-open-chart="${CSS.escape(next.id)}"]` : `[data-folder-item="${CSS.escape(next.id)}"]`))?.focus({ preventScroll: true });
  toast(`Deleted “${folder.name}”${members.length ? ` (${charts === 1 ? "its chart was" : charts ? `its ${plural(charts, "chart")} were` : "what was in it was"} kept)` : ""}`, { label: "Undo", run: () => {
    if (folderById(id)) return;
    const up = folder.parent ? folderNow(folder.parent) : undefined, again = /** @type {Folder} */ ({ ...folder, id: uid() }), gone = Boolean(up && !folderById(up));
    folderAgain.set(folder.id, again.id);
    const back = grid.itemsOf(keysNow(members.map(grid.sideKey))).filter((item) => gone || grid.parentOf(item) === up);
    const parent = gone ? (back[0] ? grid.parentOf(back[0]) : undefined) : up;
    parent ? (again.parent = parent) : delete again.parent;
    if (gone) back.splice(0, back.length, ...back.filter((item) => grid.parentOf(item) === parent));
    const now = grid.childrenOf(again.parent), first = now.findIndex((item) => back.includes(item));
    library.folders.splice(Math.min(at, library.folders.length), 0, again);
    for (const item of back) grid.setParent(item, again.id);
    grid.setChildren(back);
    const rest = now.filter((item) => !back.includes(item));
    rest.splice(first >= 0 ? first : Math.min(place, rest.length), 0, again);
    grid.setChildren(rest);
    save();
    renderCharts();
    return () => deleteFolder(folderNow(again.id));
  }, undone: `“${folder.name}” is back` });
}
/** @param {Chart} doc */
function deleteChart(doc) {
  const index = library.charts.indexOf(doc);
  if (index < 0) return;
  const wasOpen = doc === chart;
  if (wasOpen) edits.future = [];
  const neighbor = library.charts[index + 1] ?? library.charts[index - 1], kept = () => Boolean(neighbor && library.charts.includes(neighbor));
  const onPage = home && [document.activeElement, menus.ownerNow(), performance.now() - menus.closedAt < 1000 ? menus.closedOwner : null].some((element) => element?.closest("#home"));
  const tile = onPage ? grid.cardOf(doc.id) : null, beside = /** @type {HTMLElement | null | undefined} */ (tile?.nextElementSibling?.matches(".tile[data-home-card]") ? tile.nextElementSibling : tile?.previousElementSibling?.matches(".tile[data-home-card]") ? tile.previousElementSibling : null)?.dataset.homeCard;
  keepFocus(null, () => (beside ? grid.tileOpen(beside) : null), () => (kept() ? $("chart-list").querySelector(`[data-open-chart="${CSS.escape(/** @type {Chart} */ (neighbor).id)}"]`) : null), () => $("new-chart"), () => $("charts-nav"));
  discard(doc);
  const away = wasOpen && phone.matches && !home;
  if (away) {
    deletedOpen = doc.id;
    if (history.state?.fromHome === true && history.state.chart === doc.id) history.back();
    else (showHome(false, leftPlace), history.replaceState(homeState(), ""));
  }
  const standIn = wasOpen && !away && !library.charts.length ? previewCharts[0] ?? newDraft() : null;
  if (wasOpen && !away) openChart(standIn ?? library.charts[Math.min(index, library.charts.length - 1)]);
  save();
  renderCharts();
  const text = `Deleted “${titleOf(doc)}”`, action = {
    label: "Undo",
    run: () => {
      if (inLibrary(doc)) return;
      relist(doc, index);
      save();
      const reopen = () => openChart(doc);
      if (!wasOpen) renderCharts();
      else if (away && home) void leaveHome(reopen);
      else reopen();
      const same = asThen([doc]);
      return () => (same() ? ((now) => now && deleteChart(now))(library.charts.find((other) => other.id === doc.id)) : keptAsEdited([doc]));
    },
    undone: `“${titleOf(doc)}” is back`,
  };
  if (!away) return toast(text, action);
  const step = libraryStep(text, action.run, action.undone);
  void until(() => chart !== doc, 4000).then(() => toast(text, { ...action, step }));
}
withRest(() => {
  gestures.startGestures();
  gestures.dragToReorder($("chart-list"), {
    item: "[data-chart-row], [data-folder-row]",
    refuse: (event) => Boolean(closest(event, ".chart-go, .chart-more")),
    idOf: grid.sideId,
    label: (id) => (id.startsWith("f:") ? ((folder) => (folder ? `${folderMark(folder)}<span class="name">${escapeHtml(folder.name)}</span>` : ""))(folderById(id.slice(2))) : grid.chartLabel(id.slice(2))),
    landing: (x, y, ids) => {
      const places = grid.placesFor(ids), under = /** @type {HTMLElement | null | undefined} */ (document.elementFromPoint(x, y)?.closest("[data-folder-row]"));
      const onto = under && places.includes(under) ? under : null;
      /** @param {Landing | null} at */
      const level = (at) => {
        if (!at || !("item" in at) || railed()) return at;
        const depth = grid.depthOf(at.item), next = places[places.indexOf(at.item) + 1], low = next ? grid.depthOf(next) : 0;
        if (!at.after || low >= depth) return { ...at, depth };
        return { ...at, depth: clamp(Math.floor((x - at.item.getBoundingClientRect().left - 13) / 16), low, depth) };
      };
      if (onto) {
        const box = onto.getBoundingClientRect(), share = (y - box.top) / box.height, folder = folderById(onto.dataset.folderRow);
        const first = places[places.indexOf(onto) + 1], inside = first && folder && grid.parentOf(/** @type {Chart | Folder} */ (grid.itemOf(grid.sideId(first)))) === folder.id ? first : null;
        if (share < 0.25) return level({ item: onto, after: false });
        if (share > 0.75) return folder?.closed || onRail() ? level({ item: onto, after: true }) : inside ? level({ item: inside, after: false }) : { zone: onto };
        return { zone: onto };
      }
      return level(grid.landingIn(places, y, grid.sideId, ids));
    },
    land: (ids, at, y) => {
      if ("zone" in at) return grid.placeItems(ids, at.zone.dataset.folderRow, null, { dropY: y });
      let key = grid.sideId(at.item);
      for (let up = grid.depthOf(at.item) - (at.depth ?? grid.depthOf(at.item)); up > 0; up--) {
        const item = grid.itemOf(key), parent = item && grid.parentOf(item);
        if (parent) key = `f:${parent}`;
      }
      const { into, before } = grid.besideItem(key, at.after, ids);
      grid.placeItems(ids, into, before, { dropY: y });
    },
    lift: () => menus.hideMenu(),
  });
  gestures.onHold($("chart-list"), ".nav-item, .list-head-name", (row) => ((more) => (more && !railed() ? () => (more.click(), true) : null))(/** @type {HTMLElement | null} */ (row.parentElement?.querySelector(':scope > .chart-more[aria-haspopup="menu"]') ?? null)));
  gestures.dragToReorder($("home-grid"), {
    item: "[data-home-card]",
    label: grid.chartLabel,
    idOf: (tile) => /** @type {string} */ (tile.dataset.homeCard),
    carried: (id) => (grid.picked.has(id) ? grid.shownCharts().filter((doc) => grid.picked.has(doc.id)).map((doc) => doc.id) : [id]),
    pick: (id) => !grid.picked.has(id) && grid.pickChart(id),
    picking: () => grid.picked.size > 0,
    unpick: () => grid.clearPicks(),
    refuse: (event) => Boolean(closest(event, ".tile-check, .tile-more, .tile-rename, .tile.preview")),
    still: () => !grid.reorderable(),
    sticky: true,
    landing: (x, y, ids) => {
      const under = document.elementFromPoint(x, y), head = /** @type {HTMLElement | null | undefined} */ (under?.closest("[data-grid-folder]"));
      if (head) return { zone: head };
      if (under?.closest(".grid-tiles.previews")) return undefined;
      const tile = /** @type {HTMLElement | null | undefined} */ (under?.closest("[data-home-card]"));
      if (!tile) {
        const head = under?.closest(".shelf")?.querySelector("[data-grid-folder]");
        return head ? { zone: /** @type {HTMLElement} */ (head) } : under?.closest("#home-grid") ? { zone: $("home-grid") } : undefined;
      }
      if (ids.includes(/** @type {string} */ (tile.dataset.homeCard))) return null;
      const box = tile.getBoundingClientRect(), row = /** @type {HTMLElement} */ (tile.parentElement), down = row.matches(".grid-tiles") && getComputedStyle(row).gridTemplateColumns.split(" ").length < 2;
      return down ? { item: tile, after: y > box.top + box.height / 2, down } : { item: tile, after: x > box.left + box.width / 2 };
    },
    land: (ids, at) => {
      const keys = ids.map((id) => `c:${id}`);
      if ("zone" in at) grid.placeItems(keys, at.zone.dataset.gridFolder || ("folder" in homeAt ? homeAt.folder : undefined));
      else {
        const { into, before } = grid.besideItem(`c:${at.item.dataset.homeCard}`, at.after, keys);
        grid.placeItems(keys, into, before);
      }
      grid.tileOpen(ids[0])?.focus({ preventScroll: true });
    },
  });
  grid.startListKeys();
});

$("keep").addEventListener("click", () => keepChart());
$("preset-copy").addEventListener("click", () => presetsEdit.copyPreset(presetIndex(chart)));
$("preset-reset").addEventListener("click", () => presetsEdit.resetPreset(chart));
$("chart-more").addEventListener("click", (event) => {
  const button = /** @type {HTMLElement} */ (event.currentTarget);
  menus.menuOf(button, command.headMenu());
});
$("home-new-folder").addEventListener("click", (event) => {
  event.preventDefault();
  grid.newFolder([], "folder" in homeAt ? homeAt.folder || undefined : undefined);
});
$("chart-list").addEventListener("scroll", () => fitSoon($("chart-list"), true), { passive: true });
new ResizeObserver(() => fitStrip($("chart-list"), true)).observe($("chart-list"));
strip($("templates"));
new ResizeObserver(() => fitStrip($("templates"))).observe($("templates"));
function drawCards() {
  for (const i of grid.cardQueue) {
    grid.cardQueue.delete(i);
    const card = grid.recommendedCards[i];
    if (card && previews.get(card.doc.id)?.key !== grid.previewKey(card.doc)) void grid.drawPreview(card.doc, grid.previewIndex(card.doc));
  }
}
const cardsSeen = new IntersectionObserver((entries) => {
  for (const entry of entries) {
    if (!entry.isIntersecting) continue;
    grid.cardQueue.add(Number(/** @type {HTMLElement} */ (entry.target).dataset.template));
    cardsSeen.unobserve(entry.target);
  }
  drawCards();
});

let presetLooks = /** @type {Map<string, Partial<Layer>>} */ (new Map());
let presetLabels = /** @type {Record<string, string>} */ ({});
let presets = /** @type {Preset[] | null} */ (null), presetsFailed = false;
/** @type {Promise<Preset[]> | null} */
let presetsLoad = null, presetsRead = /** @type {Promise<Preset[]> | null} */ (null);
function readPresetList() {
  return (presetsRead ??= (async () => {
    const { presets: list, labels } = await presetsKept();
    presetLabels = labels;
    presetLooks = looksOf(list);
    return list;
  })());
}
function loadPresets() {
  return (presetsLoad ??= (async () => {
    presetsFailed = false;
    const list = await readPresetList();
    // (A frame between their reading and their showing: neither one long one, Firefox's a tenth of a second.)
    await aFrame();
    presets = list;
    presetTree();
    withRest(() => whenIdle(grid.searchNow));
    whenIdle(idsNow);
    presetsFound = { query: "", found: [] };
    adoptPresets();
    prunePresets(chart ?? null);
    const at = chart?.preset === undefined || (reading.hash && currentRead(reading.run, reading.at)) ? -1 : presetIndex(chart) >= 0 ? presetIndex(chart) : pristine(chart) && !handledYet ? presetShowing(chart) : -1, now = at >= 0 ? presetChart(at) : chart;
    if (now && now !== chart) openChart(now);
    withRest(() => (grid.renderRecommended(), presetsEdit.presetsArrived(), renderCharts()));
    whenIdle(addressesSoon);
    withRest(() => whenIdle(command.ranksSoon));
    if (pendingRoute && started) withRest(() => {
      const asked = pendingRoute, route = routeOf(asked), here = Boolean(history.state?.route);
      pendingRoute = "";
      const { route: _, ...state } = history.state ?? {};
      if (here) history.replaceState(state, "");
      if (here && route) "preset" in route ? grid.go(presetByAddress(route.preset, route.window)) : goTo({ path: route.path });
      else if (here) missedRoute(asked);
      else if (!route) notice(presetsEdit.missedSaid(asked));
    });
    return list;
  })().catch((error) => {
    // Not read: said where they'd be, with a way to try again (never on its own: no loop of attempts).
    presetsFailed = true;
    withRest(() => {
      grid.renderRecommended();
      presetsEdit.presetsArrived();
      renderHome();
      renderCharts();
    });
    throw error;
  }));
}
function retryPresets() {
  presetsLoad = presetsRead = null;
  presetsFailed = false;
  grid.renderRecommended();
  void loadPresets().catch(() => {});
  if (isOpen($("palette"))) showPalette();
  renderHome();
}
/** Whether the page is Studio's own, at BASE (its server serving the page at every address under it): an address of the presets' read as it starts (an old /chart/ one its server can't place, too: said missing); elsewhere (a file, another server's), none. */
const ROUTED = /^\/studio\/(?:$|presets\/|chart\/)/.test(location.pathname);
/** @param {string} address */
function missedRoute(address) {
  if (home) {
    /** @type {HTMLInputElement} */ ($("home-filter")).value = "";
    galleryShown = GALLERY_PAGE;
    grid.clearPicks();
    placeHome({ path: [] });
    history.replaceState(homeState(), "");
    renderCharts();
    homeScroll().scrollTop = 0;
  }
  notice(presetsEdit.missedSaid(address));
}
let started = false;
for (const type of ["keydown", "pointerdown", "click"]) window.addEventListener(type, (event) => !started && !(type === "click" && event.target instanceof Element && event.target.closest(".booting")) && (event.stopImmediatePropagation(), type === "click" && event.preventDefault()), true);
let pendingRoute = "";

/** @param {number} i @param {string} [window] */
function presetByAddress(i, window = undefined) {
  const doc = presetChart(i);
  takeLinkLook(doc, linkLookOf(new URLSearchParams(location.hash.slice(1)), window), presetEdited(doc) ? undefined : presetLook(i));
  linkWindow = doc.id;
  return doc;
}
function pickPreset(/** @type {number} */ i) {
  if (!presets?.[i]) return;
  if (!phone.matches) {
    if (grid.groupShut("presets")) (prefs.groupsOpen = [...prefs.groupsOpen, "presets"]), savePrefs();
    presetToReveal = i;
  }
  presetsEdit.openPreset(i);
  if (!home && presetToReveal === i) renderCharts();
}
let presetToReveal = -1;
let presetsFound = { query: "", found: /** @type {number[]} */ ([]) };
/** @returns {{ found: number[], count: number, partial: boolean }} */
function presetsMatching(/** @type {string[]} */ words, /** @type {number} */ most, under = /** @type {((i: number) => boolean) | null} */ (null), slipped = false) {
  const list = /** @type {Preset[]} */ (presets), query = words.join(" "), { texts: made, initials: madeInitials, titles, rank, order, exact } = grid.presetSearch();
  const copies = new Map();
  for (const doc of library.presets) ((i) => i >= 0 && !copies.has(i) && copies.set(i, doc))(presetIndex(doc));
  // (Only those renamed, by their title as it is now: a copy kept as it came (any preset opened) searched by its words
  // as made, the quick way. Their words, those as made and the title, in a copy of the list (a native copy, ~0.02 ms
  // without a JIT, against a lookup at each of thousands).)
  const renamed = /** @type {Map<number, string>} */ (new Map());
  for (const [i, doc] of copies) {
    const kept = keptPreset(i, doc);
    if (kept && titleOf(kept) !== list[i].t) renamed.set(i, titleOf(kept));
  }
  const texts = renamed.size ? made.slice() : made;
  const initials = renamed.size ? madeInitials.slice() : madeInitials;
  for (const [i, title] of renamed) {
    texts[i] = `${made[i]} ${command.searchText(command.searchCase(title))}`;
    initials[i] = `${madeInitials[i]} ${initialsOf([title])}`;
  }
  const initialWords = words.map((word) => initialsWord(word) ? word : "\0");
  const holds = (/** @type {number} */ i, /** @type {number} */ k) => texts[i].includes(words[k]) || initials[i].includes(initialWords[k]);
  // One scan: each one's words held, counted, and kept those holding the most (every word where any does). Else, of the presets as a whole, at least half of them and two (one word of two is a search for that word alone, not a name half remembered). One's let go once it can't hold as many. Index loops, not iterators, the first word tested inline, a word alone a loop of its own: thousands each keystroke, most let go at the first word, on a phone's engine without a JIT.
  const wordCount = words.length, first = words[0], least = under || wordCount < 2 ? wordCount : Math.max(2, Math.ceil(wordCount / 2));
  const scan = (/** @type {ArrayLike<number> | null} */ among, /** @type {number} */ fewest) => {
    const found = [];
    let top = fewest;
    const n = among === null ? texts.length : among.length;
    if (wordCount === 1) {
      for (let j = 0; j < n; j++) {
        const i = among === null ? j : among[j];
        if (texts[i].includes(first) || initials[i].includes(initialWords[0])) found.push(i);
      }
      return { found, top };
    }
    for (let j = 0; j < n; j++) {
      const i = among === null ? j : among[j];
      let held = 1;
      if (!holds(i, 0)) {
        if (wordCount - 1 < top) continue;
        held = 0;
      }
      for (let k = 1; k < wordCount && held + wordCount - k >= top; k++) if (holds(i, k)) held++;
      if (held < top) continue;
      if (held > top) (top = held), (found.length = 0);
      found.push(i);
    }
    return { found, top };
  };
  const among = !under && !renamed.size && presetsFound.query && query.startsWith(presetsFound.query) ? presetsFound.found : null;
  const all = under ? order.filter(under) : null;
  let { found, top } = scan(among ?? all, among ? wordCount : least);
  if (among && !found.length && least < wordCount) ({ found, top } = scan(all, least));
  const partial = found.length > 0 && top < wordCount;
  if (!under) presetsFound = { query, found: partial ? [] : found };
  if ((!found.length || partial) && !slipped) {
    const extra = [...renamed.values()].map(command.searchCase), choices = words.map((word) => ((slips) => (slips.length ? slips : [word]))(presetsEdit.slipsOf(word, texts, extra, initials)));
    if (choices.some((options, k) => options[0] !== words[k])) {
      const anyHolds = (/** @type {string[]} */ some) => {
        for (let i = 0; i < texts.length; i++) if ((!under || under(i)) && some.every((word) => texts[i].includes(word) || (initialsWord(word) && initials[i].includes(word)))) return true;
        return false;
      };
      const at = choices.map(() => 0), pick = () => choices.map((options, k) => options[at[k]]);
      let fixed = pick();
      for (let tries = 0; tries < 12; tries++) {
        if (anyHolds(pick())) {
          fixed = pick();
          break;
        }
        let k = at.length - 1;
        for (; k >= 0 && ++at[k] === choices[k].length; k--) at[k] = 0;
        if (k < 0) break;
      }
      return presetsMatching(fixed, most, under, true);
    }
  }
  // Exact names and their whole initials lead; then word prefixes, initials, and substrings. At each,
  // prefer all words in the title, then its start, then the nearest the top and shortest (`order`). Each one's fit found as it's reached, the fits in turn, till
  // `most` are: a word or two matches thousands, a few dozen show.
  const inFound = new Uint8Array(list.length), fits = new Int8Array(list.length).fill(-1);
  for (let k = 0; k < found.length; k++) inFound[found[k]] = 1;
  const holdsAll = (/** @type {string} */ text, /** @type {string} */ initial) => {
    for (let k = 0; k < wordCount; k++) if (!text.includes(words[k]) && !initial.includes(initialWords[k])) return false;
    return true;
  };
  const prefixes = words.map((word) => new RegExp(`(?:^|[^a-z0-9])${word.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}`));
  const fitOf = (/** @type {number} */ i) => {
    if (fits[i] < 0) {
      const kept = copies.has(i) ? keptPreset(i, copies.get(i)) : undefined, named = titles[i], lower = kept ? command.searchCase(titleOf(kept)) : null, title = lower === null ? named.title : command.searchText(lower);
      const isExact = named.name === query || (lower === null ? named.subject : lower.split(": ")[0]) === query;
      const initial = lower === null ? named.initials : initialsOf([lower]);
      const quality = words.every((_, k) => prefixes[k].test(texts[i])) ? 2 : words.every((_, k) => prefixes[k].test(texts[i]) || initials[i].includes(initialWords[k])) ? 1 : 0;
      const isInitialName = wordCount === 1 && initialsWord(query) && ((exact.get(query)?.includes(i) ?? false) || (lower !== null && initialsOf([lower.split(": ")[0]]) === query));
      fits[i] = isExact ? 15 : isInitialName ? 14 : quality * 5 + (holdsAll(title, initial) ? 2 : 0) + (title.startsWith(words[0]) ? 1 : 0);
    }
    return fits[i];
  };
  const rankOf = (/** @type {number} */ i) => ((title) => (title === undefined ? rank[i] : grid.presetRankOf(list[i].p.length, title)))(renamed.get(i));
  const byRank = (/** @type {number} */ a, /** @type {number} */ b) => rankOf(a) - rankOf(b) || a - b;
  const best = [...new Set([...(exact.get(query) ?? []), ...copies.keys()])].filter((i) => inFound[i] && fitOf(i) >= 14).sort((a, b) => fitOf(b) - fitOf(a) || byRank(a, b)).slice(0, most);
  const few = found.length * 16 < order.length ? found.slice().sort(byRank) : null;
  const along = few ?? (!renamed.size ? order : ((moved) => {
    const out = [];
    let m = 0;
    for (let k = 0; k < order.length; k++) {
      const i = order[k];
      if (renamed.has(i)) continue;
      for (; m < moved.length && byRank(moved[m], i) < 0; m++) out.push(moved[m]);
      out.push(i);
    }
    for (; m < moved.length; m++) out.push(moved[m]);
    return out;
  })([...renamed.keys()].sort(byRank)));
  for (let fit = 13; fit >= 0 && best.length < most; fit--) for (let k = 0; k < along.length && best.length < most; k++) {
    const i = along[k];
    if ((few || inFound[i]) && fitOf(i) === fit) best.push(i);
  }
  return { found: best, count: found.length, partial };
}
withRest(() => {
  grid.renderRecommended();
});
void loadPresets().catch(() => {});
withRest(() => {
  document.addEventListener("fullscreenchange", () => !document.fullscreenElement && fullOn() && presetsEdit.setFull(false));
});
new ResizeObserver(([entry]) => /** @type {HTMLElement} */ (document.querySelector(".app")).style.setProperty("--head-height", `${entry.borderBoxSize[0].blockSize}px`)).observe(/** @type {HTMLElement} */ (document.querySelector(".card-head")));
// Escape is the page's: it does here what it does, and goes no further, so a browser's own full screen (macOS's,
// ⌃⌘F) doesn't leave with it. (One a page asked for is the browser's to leave, see setFull.)
window.addEventListener("keydown", (event) => event.key === "Escape" && !document.querySelector("dialog:popover-open") && event.preventDefault(), true);
// (A dialog's too: nothing in it having taken the key, its own way out, as the browser's would be (its cancel, then
// closed unless that's refused), the key going no further: left to the browser, Firefox's full screen left with it.)
window.addEventListener("keydown", (event) => {
  const dialog = event.key === "Escape" && !event.defaultPrevented && !event.isComposing ? topDialog() : undefined;
  if (!dialog) return;
  event.preventDefault();
  if (dialog.dispatchEvent(new Event("cancel", { cancelable: true }))) dialog.hidePopover();
});
window.addEventListener("keydown", (event) => {
  const dialog = event.key === "Tab" && !event.defaultPrevented ? topDialog() : undefined, active = document.activeElement;
  if (!dialog || (active && !dialog.contains(active) && active.closest(":popover-open"))) return;
  const stops = all$("button:not(:disabled), a[href], input:not(:disabled), textarea:not(:disabled), select:not(:disabled), summary, [tabindex]", dialog).filter((stop) => stop.tabIndex >= 0 && stop.checkVisibility({ visibilityProperty: true }) && !stop.closest("[inert]"));
  const at = active instanceof HTMLElement ? stops.indexOf(active) : -1;
  const to = event.shiftKey ? (at <= 0 ? stops.at(-1) : undefined) : at < 0 || at === stops.length - 1 ? stops[0] : undefined;
  if (!to) return;
  event.preventDefault();
  to.focus();
});
let layersWritten = { keys: /** @type {string[]} */ ([]), tail: /** @type {[string, string][]} */ ([]) };
let layersBehind = false;
/**
 * The layers' list, as the chart is now: where it shows, at once; hidden (a phone's sheet put away, the panel shut),
 * while the page is idle after, or as it shows, whichever comes first, not in the change that asked (a chart's
 * opening: 31 ms of its tap at a phone's 4× slower CPU). What of it shows on the chart (a lit legend, the values)
 * at once either way.
 */
function renderLayers() {
  if (panelShown()) return writeLayers();
  lightLegend(litLayer, true);
  updateValues();
  if (layersBehind) return;
  layersBehind = true;
  whenIdle((deadline) => layersBehind && withRest(() => (writeLayers(), presetsEdit.roomsAhead(deadline))));
}
function writeLayers() {
  if (!presetsEdit) return void withRest(writeLayers);
  roomsAt = null;
  layersBehind = false;
  const list = $("layer-list");
  const focus = focusKey(list);
  $("layer-count").textContent = String(chart.layers.length);
  if (!chart.layers.length) {
    list.innerHTML = "";
    layersWritten = { keys: [], tail: [] };
    return;
  }
  const sections = chart.panes.map((pane, i) => {
    const layers = chart.layers.filter((layer) => layer.pane === pane.id);
    /** @type {[string, string][]} */
    const rows = [["head:pane", `<div class="pane-head">
              <span>Pane ${i + 1}</span><button type="button" class="pane-scale" data-scale-menu="${pane.id}" aria-haspopup="menu" aria-label="Pane ${i + 1} scale: ${presetsEdit.paneChip(pane)}" data-tip="${pane.lens ? "Show as, scale" : "Scale"}, stack, order and average">${presetsEdit.paneChip(pane)}${CHEVRON}</button><span class="spacer"></span>
              <button type="button" class="cap small" data-pane-menu="${pane.id}" aria-haspopup="menu" aria-label="More for pane ${i + 1}" data-tip="More">${ICONS.more}</button>
              ${((shown) => `<button type="button" class="cap small ${shown ? "" : "off"}" data-pane-eye="${pane.id}" aria-label="${shown ? "Hide" : "Show"} pane ${i + 1}" data-tip="${shown ? "Hide" : "Show"} pane">${shown ? ICONS.eye : ICONS.eyeOff}</button>`)(paneShows(chart, pane.id))}
            </div>`], ...layers.map((layer) => /** @type {[string, string]} */ ([layer.id, ((layer) => {
        const result = resolved?.results.get(layer.id);
        const loading = layer.series && !result;
        const [subClass, subText] = result?.error ? ["sub error", result.error] : layer.from ? ["sub", fromUnfit(chart, layer) || `Between ${fromSpan(layer)}${((silent) => (silent.length ? ` · ${silent.join(", ")} ${silent.length === 1 ? "has" : "have"} no data` : ""))(resolved ? chart.layers.filter((item) => layer.from?.includes(item.id) && !resolved?.results.get(item.id)?.values).map((item) => item.letter) : [])}`] : layer.formula !== undefined ? (layer.label ? ["sub code", `ƒ\u00a0${layer.formula}`] : ["sub", formulaLine(layer)]) : ["sub", ""];
        const sub = `<span class="${subClass}"${result?.error && result.detail ? ` data-tip="${escapeHtml(result.detail)}"` : ""}><span class="sub-letter">${letterChip(layer)}</span>${escapeHtml(subText)}</span>`;
        const shows = layerShows(chart, layer), by = !shows && !layer.hidden ? chart.layers.find((item) => item.id === layer.ref) : undefined;
        return `<div class="layer-row ${shows ? "" : "hidden"} ${layer.id === selected ? "selected" : ""} ${layerPicks.has(layer.id) ? "picked" : ""}" data-layer="${layer.id}">
                <input type="checkbox" class="pick-check row-check" data-pick-layer="${layer.id}" style="${escapeHtml(`--c:${ink(layer.color)};--tick:${tickOn(layer.color)}`)}" aria-label="Select ${escapeHtml(labelOf(layer))}" data-tip="${layerPicks.has(layer.id) ? "Deselect" : "Select"}" ${layerPicks.has(layer.id) ? "checked" : ""} />
                <button type="button" class="row-main" data-select="${layer.id}" data-tip="${escapeHtml(subText ? `${labelOf(layer)}\n${subText}` : labelOf(layer))}" ${layer.id === selected ? 'aria-current="true"' : ""}${layer.type === "cloud" && result?.low ? ` style="--value-room: ${valueSlot(layer, rowSizes.value, true) + 10}px"` : ""}>
                  <span class="row-mark" style="--c:${escapeHtml(ink(layer.color))}">${seriesMark(layer)}</span>
                  <span class="row-text"><span class="name">${escapeHtml(viewName(chart, layer))}</span>${sub}</span>
                  <span class="row-figures"><span class="row-value ${loading ? "loading" : ""}" ${layer.hidden || result?.error ? "" : `data-value="${layer.id}"${layer.type === "cloud" ? " data-lines" : ""}`}></span>${layer.type === "cloud" || layer.type === "signal" || heatLike(layer) ? "" : `<span class="row-change" data-change="${layer.id}"></span>`}<span class="row-note"></span></span>
                </button>
                ${by ? `<button type="button" class="cap small by" data-toggle="${by.id}" aria-label="Hidden with ${escapeHtml(labelOf(by))}: show ${escapeHtml(labelOf(by))}" data-tip="Hidden with its layer, ${escapeHtml(by.letter)}: show it">${ICONS.eye}</button>` : `<button type="button" class="cap small" data-toggle="${layer.id}" aria-label="${layer.hidden ? "Show" : "Hide"} ${escapeHtml(labelOf(layer))}" data-tip="${layer.hidden ? "Show" : "Hide"}${layer.type === "signal" ? "" : `\n${keyText("⌥-click: alone in its pane")}`}">${layer.hidden ? ICONS.eyeOff : ICONS.eye}</button>`}
              </div>`;
      })(layer)]))];
    return { key: pane.id, open: `<section class="pane-group" data-pane="${pane.id}">`, rows };
  });
  /** @type {[string, string][]} */
  const tail = [["drop-new", `<div class="drop-new" data-drop-new>Drop here for a new pane</div>`], ["markers", presetsEdit.markersHtml()], ["adds", `<div class="layer-adds"><button type="button" class="nav-item" data-action="add-series">${ICONS.plus}<span class="label">Add a layer</span><kbd>A</kbd></button></div>`]];
  const keys = [...sections.map((section) => section.key), ...tail.map(([key]) => key)], was = layersWritten;
  layersWritten = { keys, tail };
  if (was.keys.length === keys.length && list.children.length === keys.length && was.keys.every((key, i) => key === keys[i])) {
    sections.forEach((section, i) => writeList(/** @type {HTMLElement} */ (list.children[i]), section.rows));
    tail.forEach(([, html], i) => html !== was.tail[i][1] && (list.children[sections.length + i].outerHTML = html));
    restoreFocus(list, focus);
  } else {
    inspector.rewrite(list, [...sections.map(({ open, rows }) => `${open}${rows.map(([, html]) => html).join("")}</section>`), ...tail.map(([, html]) => html)].join(""));
    sections.forEach(({ rows }, i) => listWritten.set(/** @type {HTMLElement} */ (list.children[i]), rows));
    restoreFocus(list, focus);
  }
  drawKeys(list);
  updateValues();
  lightLegend(litLayer, true);
  list.classList.toggle("picking", layerPicks.size > 0);
  inspector.renderLayersBar();
}

withRest(() => {
  gestures.dragToReorder($("layer-list"), {
    item: ".layer-row[data-layer]",
    label: (id) => ((layer) => (layer ? `${letterChip(layer)}<span class="name">${escapeHtml(labelOf(layer))}</span>` : ""))(layerById(id)),
    idOf: (row) => /** @type {string} */ (row.dataset.layer),
    carried: (id) => (layerPicks.has(id) ? chart.layers.filter((layer) => layerPicks.has(layer.id)).map((layer) => layer.id) : [id]),
    pick: (id) => !layerPicks.has(id) && pickLayer(id),
    picking: () => layerPicks.size > 0,
    unpick: () => clearLayerPicks(),
    refuse: (event) => Boolean(closest(event, "[data-toggle]")),
    // (The new pane's place shows below; the panel keeps its height: a phone's sheet, as tall as what it holds, would
    // grow under the finger. Nearing its edge, it scrolls there.)
    lift: () => ($("panel").style.height = `${$("panel").offsetHeight}px`),
    settle: () => ($("panel").style.height = ""),
    landing: (x, y, ids) => {
      const under = document.elementFromPoint(x, y), zone = /** @type {HTMLElement | null | undefined} */ (under?.closest("[data-drop-new]"));
      if (zone) return { zone };
      const pane = /** @type {HTMLElement | null | undefined} */ (under?.closest(".pane-group[data-pane]"));
      return pane ? grid.landingIn(all$(".layer-row[data-layer]", pane), y, (row) => /** @type {string} */ (row.dataset.layer), ids) : null;
    },
    land: presetsEdit.dropLayers,
  });
});

function showList() {
  if (selected) {
    lastOpen = selected;
    selected = "";
    renderLayers();
    inspector.renderInspector();
    updateLegends();
    respot();
  }
  if (phone.matches) document.body.dataset.sheet = "panel";
  else if (document.body.dataset.panel === "closed") document.body.toggleAttribute("data-peek", true);
}
function pickLayer(/** @type {string} */ id, range = false) {
  menus.hideMenu();
  const ids = chart.layers.map((layer) => layer.id), start = layerPicks.size ? layerPickLast : selected || lastOpen, from = ids.indexOf(start), to = ids.indexOf(id);
  if (range && from >= 0 && to >= 0) for (const other of ids.slice(Math.min(from, to), Math.max(from, to) + 1)) layerPicks.add(other);
  else if (layerPicks.has(id)) {
    layerPicks.delete(id);
    if (!layerPicks.has(layerPickLast)) layerPickLast = [...layerPicks].at(-1) ?? "";
    pickHead = layerPickLast;
    return inspector.showPicks();
  } else {
    if (!layerPicks.size && selected && selected !== id) layerPicks.add(selected);
    layerPicks.add(id);
  }
  layerPickLast = range && from >= 0 ? start : id;
  pickHead = id;
  lastOpen = "";
  inspector.showPicks();
}
function extendPicks(/** @type {number} */ delta) {
  const ids = chart.layers.map((layer) => layer.id), focused = /** @type {HTMLElement | null} */ (document.activeElement?.closest?.(".layer-row[data-layer]") ?? null);
  if (!ids.length) return;
  const anchor = layerPicks.size ? layerPickLast : selected || focused?.dataset.layer || "", start = anchor || ids[delta > 0 ? 0 : ids.length - 1], from = ids.indexOf(start);
  if (from < 0) return;
  const head = anchor ? clamp((layerPicks.size && ids.includes(pickHead) ? ids.indexOf(pickHead) : from) + delta, 0, ids.length - 1) : from;
  layerPicks.clear();
  for (const id of ids.slice(Math.min(from, head), Math.max(from, head) + 1)) layerPicks.add(id);
  layerPickLast = start;
  pickHead = ids[head];
  inspector.showPicks();
  /** @type {HTMLElement | null} */ ($("layer-list").querySelector(`[data-select="${pickHead}"]`))?.focus();
}
function pickEvery() {
  for (const layer of chart.layers) layerPicks.add(layer.id);
  layerPickLast = pickHead = chart.layers[0]?.id ?? "";
  inspector.showPicks();
}
/** Mark picks in place (replacing a row held by touch would turn its release into a click). */
function markPicks() {
  $("layer-list").classList.toggle("picking", layerPicks.size > 0);
  for (const row of all$(".layer-row[data-layer]", $("layer-list"))) {
    const on = layerPicks.has(/** @type {string} */ (row.dataset.layer));
    row.classList.toggle("picked", on);
    row.classList.toggle("selected", row.dataset.layer === selected);
    row.querySelector(".row-main")?.toggleAttribute("aria-current", row.dataset.layer === selected);
    grid.showPick(/** @type {HTMLInputElement} */ (row.querySelector(".row-check")), on);
  }
  markLegends();
  layersWritten = { keys: [], tail: [] };
  inspector.renderLayersBar();
  if (inspector.multiEdit) inspector.renderInspector();
}
function clearLayerPicks(refocus = false) {
  layerPicks.clear();
  lastOpen = "";
  markPicks();
  if (refocus) /** @type {HTMLElement | null} */ ($("layer-list").querySelector(`[data-select="${layerPickLast}"]`))?.focus();
}
/** @param {string} action @param {HTMLElement} [anchor] */
function layersBatch(action, anchor) {
  const ids = inspector.pickedLayers().map((layer) => layer.id);
  if (action === "clear" || !ids.length) return clearLayerPicks(action === "clear");
  if (action === "edit") {
    if (ids.length === 1) return select(ids[0]);
    inspector.editPicked(true);
    /** @type {HTMLElement | null} */ ($("inspector").querySelector('[data-action="deselect"]'))?.focus();
  } else if (action === "hide") inspector.hideLayers(ids);
  else if (action === "duplicate") {
    const copies = inspector.duplicateLayers(ids);
    if (!copies.length) return;
    layerPicks.clear();
    for (const id of copies) layerPicks.add(id);
    layerPickLast = pickHead = copies[0];
    inspector.showPicks();
  } else if (action === "move" && anchor) inspector.paneMenu(anchor, ids, () => presetsEdit.refocus('[data-layers-batch="move"]'));
  else if (action === "cloud" || action === "trend") inspector.addFrom(action, ids);
  else if (action === "remove") inspector.removeLayers(ids);
}
$("layers-bar").addEventListener("click", (event) => {
  const button = closest(event, "[data-layers-batch]");
  if (button?.getAttribute("aria-disabled") === "true") notice(/** @type {string} */ (button.dataset.tip));
  else if (button) layersBatch(/** @type {string} */ (button.dataset.layersBatch), button);
});

function select(id, sheet = true) {
  if (phone.matches && !sheet) return;
  const fromList = $("layer-list").contains(document.activeElement), unchosen = Boolean(id) && chosen.size > 0;
  selected = id;
  if (id) {
    chosen = new Set();
    layerPicks.clear();
  }
  if (selected && phone.matches && sheet) document.body.dataset.sheet = "panel";
  else if (selected && document.body.dataset.panel === "closed") document.body.toggleAttribute("data-peek", true);
  inspector.showSelection();
  const layer = layerById(id), result = resolved?.results.get(id);
  if (layer?.series && result && !result.info && !result.missing) void info(layer.series).then(meta => {
    result.info = meta;
    result.unit = unitOf(meta.type);
    if (selected === id && resolved?.results.get(id) === result) inspector.renderInspector();
  }, () => {});
  if (unchosen) repaintDrawings();
  if (fromList && selected) /** @type {HTMLElement | null} */ ($("inspector").querySelector('[data-action="deselect"]'))?.focus();
}
function deselect() {
  if (inspector.editingMany()) {
    inspector.editPicked(false);
    return void /** @type {HTMLElement | null} */ ($("layers-bar").querySelector('[data-layers-batch="edit"]'))?.focus();
  }
  const was = selected, fromSettings = $("inspector").contains(document.activeElement), peeking = document.body.hasAttribute("data-peek");
  selected = "";
  lastOpen = was;
  if (peeking) keepFocus($("panel"), () => $("panel-toggle"), () => $("plot-keys"));
  document.body.removeAttribute("data-peek");
  inspector.showSelection();
  if (fromSettings && !peeking) /** @type {HTMLElement | null} */ ($("layer-list").querySelector(`[data-select="${was}"]`))?.focus();
}

const DEFAULTS = {
  opacity: { is: (layer) => layer.opacity === undefined, set: (layer) => (layer.opacity = undefined), text: (layer) => `${Math.round(defaultOpacity(layer) * 100)}%`, focus: "#opacity" },
  width: { is: (layer) => layer.width === 1.5, set: (layer) => (layer.width = 1.5), text: "1.5px", focus: "#width" },
  "fill-opacity": { is: (layer) => layer.fillOpacity === undefined, set: (layer) => (layer.fillOpacity = undefined), text: (layer) => `${Math.round(fillOf({ ...layer, fillOpacity: undefined }, inspector.stackedIn(layer))[0] * 100)}%`, focus: "#fill-opacity" },
  base: { is: (layer) => baseOf(layer) === 0, set: (layer) => (layer.base = undefined), text: "0", focus: "#base" },
  format: { is: (layer) => !layer.format, set: (layer) => (layer.format = undefined), text: "Automatic", focus: "#format" },
  "bubble-size": { is: (layer) => layer.size === undefined, set: (layer) => (layer.size = undefined), text: `${BUBBLE_SIZE}px`, focus: "#bubble-size" },
  "bubble-hide": { is: (layer) => layer.hide === undefined, set: (layer) => (layer.hide = undefined), text: hideText(BUBBLE_HIDE), focus: "#bubble-hide" },
};
const SLIDERS = {
  width: { label: "Width", stops: WIDTHS, of: (layer) => layer.width, text: (v) => `${v}px`, say: (v) => `${v} pixels`, set: (layer, v) => (layer.width = v) },
  "draw-width": { label: "Width", stops: WIDTHS, of: (look) => look.width, text: (v) => `${v}px`, say: (v) => `${v} pixels` },
  "draw-opacity": { label: "Opacity", stops: OPACITIES, of: (look) => look.opacity ?? 1, text: (v) => `${Math.round(v * 100)}%`, say: (v) => `${Math.round(v * 100)} percent` },
  "fill-opacity": { label: "Fill opacity", stops: OPACITIES, of: (layer) => fillOf(layer, inspector.stackedIn(layer))[0], text: (v) => `${Math.round(v * 100)}%`, say: (v) => `${Math.round(v * 100)} percent`, set: (layer, v) => (layer.fillOpacity = v === fillOf({ ...layer, fillOpacity: undefined }, inspector.stackedIn(layer))[0] ? undefined : v) },
  "bubble-size": { label: "Size", stops: BUBBLE_SIZES, of: (layer) => layer.size ?? BUBBLE_SIZE, text: (v) => `${v}px`, say: (v) => `${v} pixels`, set: (layer, v) => (layer.size = v === BUBBLE_SIZE ? undefined : v) },
  "bubble-hide": { label: "Hide below", stops: BUBBLE_HIDES, of: (layer) => layer.hide ?? BUBBLE_HIDE, text: hideText, say: (v) => (v ? `${Math.round(v * 100)} percent of the largest` : "None"), set: (layer, v) => (layer.hide = v === BUBBLE_HIDE ? undefined : v) },
  opacity: { label: "Opacity", stops: OPACITIES, of: opacityOf, text: (v) => `${Math.round(v * 100)}%`, say: (v) => `${Math.round(v * 100)} percent`, set: (layer, v) => (layer.opacity = v === defaultOpacity(layer) ? undefined : v) },
};
function track(event, element, cursor, move) {
  event.preventDefault();
  if (holding) return;
  element.focus();
  holding = ++holds;
  move(event);
  document.body.classList.add(cursor);
  press(event, {
    move,
    end: () => {
      try {
        inspector.flushPick();
      } finally {
        holding = 0;
        document.body.classList.remove(cursor);
      }
      if (heldEdits) {
        heldEdits = false;
        refresh();
      }
    },
  });
}
// A cursor dragged: its color between its neighbors, one undo step a drag; the bar pressed elsewhere, a color added
// there. (Taken by the nearest within a finger's reach (22px either side), not by whichever's reach is on top: two
// close together, each where it's drawn.)
$("inspector").addEventListener("pointerdown", (event) => {
  const bar = closest(event, "[data-gradient-bar]"), stops = layerById(selected)?.gradient;
  if (event.button !== 0 || !bar || !stops?.length) return;
  const at = inspector.barAt(bar, event.clientX), k = stops.reduce((near, [place], i) => (Math.abs(place - at) < Math.abs(stops[near][0] - at) ? i : near), 0);
  const cursor = (Math.abs(stops[k][0] - at) / 100) * bar.offsetWidth <= 22 ? /** @type {HTMLElement | null} */ (bar.querySelector(`.gradient-stop[data-stop="${k}"]`)) : null;
  if (!cursor) return void (event.preventDefault(), inspector.gradientEdit((item) => inspector.addStop(item, at), true));
  inspector.colorSlot.set(`${chart.id}:${selected}`, `stop${k}`);
  inspector.pickPress(event, cursor, "scrubbing", (/** @type {{ clientX: number }} */ point) => {
    const at = Math.round(inspector.barAt(bar, point.clientX)), now = layerById(selected)?.gradient?.[k]?.[0];
    if (now === undefined || Math.round(clamp(at, layerById(selected)?.gradient?.[k - 1]?.[0] ?? 0, layerById(selected)?.gradient?.[k + 1]?.[0] ?? 100)) === now) return;
    inspector.editLayer((item) => inspector.moveStop(item, k, at), { coalesce: `stop:${selected}:${k}` });
    inspector.showStop(k);
  });
});
$("inspector").addEventListener("keydown", (event) => {
  const cursor = /** @type {HTMLElement} */ (event.target);
  if (!cursor.classList?.contains("gradient-stop")) return;
  const k = Number(cursor.dataset.stop), by = event.key === "ArrowRight" ? 1 : event.key === "ArrowLeft" ? -1 : 0;
  if (by) {
    event.preventDefault();
    event.stopPropagation();
    inspector.colorSlot.set(`${chart.id}:${selected}`, `stop${k}`);
    inspector.editLayer((item) => inspector.moveStop(item, k, (item.gradient?.[k]?.[0] ?? 0) + by * (event.shiftKey ? 10 : 1)), { coalesce: `stop:${selected}:${k}` });
    return void (inspector.renderInspector(), presetsEdit.refocus(`#inspector .gradient-stop[data-stop="${k}"]`));
  }
  if (erasing(event)) {
    event.preventDefault();
    event.stopPropagation();
    inspector.gradientEdit((item) => inspector.removeStop(item, k), true);
  }
});
$("inspector").addEventListener("input", (event) => {
  const field = /** @type {HTMLInputElement} */ (event.target);
  if (!field.dataset?.stopAt) return;
  const k = Number(field.dataset.stopAt), at = inspector.readNumber(field.value);
  if (!field.value.trim() || !Number.isFinite(at)) return;
  inspector.editLayer((item) => inspector.moveStop(item, k, at), { coalesce: `stop:${selected}:${k}` });
  inspector.showStop(k, !(event instanceof InputEvent));
});
$("inspector").addEventListener("focusout", (event) => {
  const field = /** @type {HTMLInputElement} */ (event.target);
  if (field.dataset?.stopAt) inspector.showStop(Number(field.dataset.stopAt));
});
withRest(() => {
  inspector.sliders($("inspector"), { valueOf: (id) => ((layers) => (layers.length ? inspector.sharedBy(layers, SLIDERS[id].of) ?? SLIDERS[id].of(layers[0]) : undefined))(inspector.inHand()), set: inspector.setSlider });
});
function colorPane(/** @type {HTMLElement} */ anchor, /** @type {string} */ color, /** @type {string} */ target, /** @type {() => void} */ back) {
  menus.showMenu(anchor, `<div class="field belt-pop color-pop color-pane"><span><button type="button" class="cap small" data-pane-back="1" aria-label="Bitview's colors" data-tip="Bitview's colors">${ICONS.back}</button>Custom</span><div class="color-box">${inspector.planeHtml(color, target)}${inspector.hexField(target, color)}</div></div>`, ({ paneBack }) => paneBack && back(), null, anchor.closest("#belt") ? pen.beltSide() : "");
  menus.setMenuBack(back);
  inspector.drawPlanes();
  inspector.planeOf(target)?.focus({ preventScroll: true });
}
document.addEventListener("keydown", (event) => {
  const field = /** @type {HTMLInputElement} */ (event.target);
  if (!field.matches?.("#inspector .color-editor .hex") || composing(event) || (event.key !== "Enter" && event.key !== "Escape")) return;
  event.preventDefault();
  event.stopPropagation();
  const color = event.key === "Enter" ? command.readColor(field.value) : null, key = field.closest("[data-palette]")?.getAttribute("data-palette");
  if (color && key) inspector.editLayer((item) => inspector.setColor(item, key, color));
  menus.letGo(field);
}, true);
document.addEventListener("keydown", (event) => {
  const field = event.target;
  if (event.key !== "Enter" || composing(event) || !(field instanceof HTMLInputElement) || !field.dataset.hex || field.matches("#inspector .color-editor .hex")) return;
  const color = command.readColor(field.value);
  if (color) inspector.pick(field.dataset.hex, color), inspector.flushPick();
}, true);
withRest(() => {
  document.addEventListener("focusout", (event) => /** @type {HTMLElement} */ (event.target).matches?.("#inspector .color-editor .hex") && inspector.inspectorDue && inspector.afterPress(() => setTimeout(inspector.renderInspector)));
  document.addEventListener("input", (event) => {
    const field = /** @type {HTMLInputElement} */ (event.target), name = field.dataset?.hex;
    if (!name || !field.matches(".hex")) return;
    const color = `#${field.value.trim().replace(/^#/, "").toLowerCase()}`;
    if (!/^#[0-9a-f]{6}$/.test(color)) return;
    const plane = inspector.planeOf(name), { u, t } = inspector.placeOf(color), named = field.closest(".field, .color-editor")?.querySelector(".color-now");
    /** @type {HTMLElement | null | undefined} */ (field.closest(".color-editor")?.querySelector('.slot-chip[aria-pressed="true"]'))?.style.setProperty("--c", cssColor(color));
    plane?.style.setProperty("--x", String(u));
    plane?.style.setProperty("--y", String(t));
    plane?.querySelector(".ring")?.setAttribute("style", `--c:${cssColor(color)}`);
    if (named) named.textContent = colorName(color);
    inspector.pick(name, color);
  });
});
document.addEventListener("keydown", (event) => {
  const plane = /** @type {HTMLElement} */ (event.target);
  if (!plane.classList?.contains("plane") || !["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].includes(event.key)) return;
  const name = /** @type {string} */ (plane.dataset.plane);
  if (!inspector.planeTarget(name)) return;
  event.preventDefault();
  event.stopPropagation();
  const { u, t } = inspector.placeOf(inspector.pickedOf(name));
  const sign = event.key === "ArrowUp" || event.key === "ArrowRight" ? 1 : -1, steps = event.shiftKey ? 10 : 1;
  if (upDown(event)) inspector.pickPoint(name, u, t - sign * steps * 0.02);
  else inspector.pickPoint(name, u + (sign * steps * inspector.HUES) / 360, t);
}, true);
withRest(() => {
  document.addEventListener("pointerdown", (event) => {
    const plane = closest(event, ".plane");
    if (event.button !== 0 || !plane) return;
    const name = /** @type {string} */ (plane.dataset.plane);
    inspector.pickPress(event, plane, "picking", (/** @type {PointerEvent} */ point) => inspector.pickAt(name, point.clientX, point.clientY));
  });
});
const formulaFit = new ResizeObserver(([entry]) => {
  const textarea = /** @type {HTMLTextAreaElement} */ (entry.target);
  if (Math.round(entry.contentRect.width) === Number(textarea.dataset.fitWidth)) return;
  textarea.dataset.fitWidth = String(Math.round(entry.contentRect.width));
  textarea.style.height = "auto";
  textarea.style.height = `${textarea.scrollHeight}px`;
});
withRest(() => {
  $("inspector").addEventListener("scroll", inspector.placeAssist, true);
  addEventListener("resize", inspector.placeAssist);
  viewport.addEventListener("resize", inspector.placeAssist);
  viewport.addEventListener("scroll", inspector.placeAssist);
  document.addEventListener("selectionchange", inspector.updateCallHelp);
  document.addEventListener("input", inspector.updateCallHelp);
});
withRest(() => {
  document.addEventListener("focusout", () => requestAnimationFrame(inspector.updateCallHelp));
  document.addEventListener("focusout", (event) => {
    const field = /** @type {HTMLInputElement} */ (event.target);
    if (field.id === "base") {
      const layer = layerById(selected), label = field.closest(".field")?.firstElementChild, reset = label?.querySelector(".reset");
      if (layer && label && !DEFAULTS.base.is(layer) && !reset) label.insertAdjacentHTML("beforeend", inspector.resetFor("base", layer));
      else if (layer && DEFAULTS.base.is(layer)) reset?.remove();
      return;
    }
    if (field.id === "formula") inspector.formulaFieldLeft(/** @type {HTMLTextAreaElement} */ (event.target));
  });
});
$("inspector").addEventListener("keydown", (event) => {
  const field = /** @type {HTMLInputElement} */ (event.target);
  if (!field.matches?.(".number-input") || (event.key !== "ArrowUp" && event.key !== "ArrowDown") || event.metaKey || event.ctrlKey || event.altKey) return;
  const now = inspector.readNumber(field.value);
  if (!Number.isFinite(now)) return;
  event.preventDefault();
  const [, decimals = "", power = "0"] = /(?:\.(\d+))?(?:e([+-]\d+))?$/.exec(String(now)) ?? [];
  const places = Math.max(0, decimals.length - Number(power));
  field.value = String(Number((now + (event.key === "ArrowUp" ? 1 : -1) * (event.shiftKey ? 10 : 1)).toFixed(Math.min(places, 20))));
  field.dispatchEvent(new Event("input", { bubbles: true }));
});
$("inspector").addEventListener("input", (event) => {
  const target = event.target;
  if (target.id === "label") inspector.editLayer((layer) => nameLayer(layer, target.value.trim()), { coalesce: `label:${selected}` });
  else if (target.id === "formula") {
    inspector.markTyping();
    inspector.drawWhenReady(selected, target.value);
    const layer = layerById(selected);
    if (layer) inspector.updateFormula(layer);
    inspector.suggestHere();
  } else if (target.id === "base") {
    const base = inspector.readNumber(target.value);
    if (Number.isFinite(base)) inspector.editLayer((layer) => (layer.base = base || undefined), { coalesce: `base:${selected}` });
  } else if (target.id === "level-name-under") {
    const name = target.value.trim().slice(0, 40);
    inspector.editLayer((layer) => (name ? (layer.underName = name) : delete layer.underName), { coalesce: `under-name:${selected}` });
  } else if (target.id.startsWith("level-name-")) {
    const i = Number(target.id.slice(11)), name = target.value.trim().slice(0, 40);
    inspector.editLayer((layer) => {
      const level = layer.levels?.[i];
      if (!level) return;
      if (name) level.name = name;
      else delete level.name;
    }, { coalesce: `level-name:${i}:${selected}` });
  } else if (target.id.startsWith("level-")) {
    const i = Number(target.id.slice(6)), level = inspector.readNumber(target.value);
    if (Number.isFinite(level)) inspector.editLayer((layer) => layer.levels?.[i] && (layer.levels[i].value = level), { coalesce: `level:${i}:${selected}` });
    const layer = layerById(selected), under = $("inspector").querySelector(".under-label");
    if (under && layer?.levels?.length) under.innerHTML = `<span class="sign" aria-hidden="true">&lt;</span><span class="vh">Under </span>${Math.min(...layer.levels.map((item) => item.value))}`;
  }
});
$("inspector").addEventListener("pointerdown", (event) => {
  if (!closest(event, "input, textarea")) $("inspector").dataset.pressed = "1";
  else delete $("inspector").dataset.pressed;
}, true);
$("inspector").addEventListener("input", () => delete $("inspector").dataset.pressed, true);
$("inspector").addEventListener("focusout", (event) => {
  if (!/** @type {HTMLElement} */ (event.target).closest?.(".levels")) return;
  inspector.afterPress(() => requestAnimationFrame(() => !$("inspector").querySelector(".levels")?.contains(document.activeElement) && inspector.renderInspector()));
});
$("inspector").addEventListener("keydown", (event) => {
  if (upDown(event) && event.target.matches?.(".preset[aria-haspopup]")) {
    event.preventDefault();
    event.stopPropagation();
    return event.target.click();
  }
  if (event.target.id !== "formula" || composing(event)) return;
  if (event.key === "Escape" && (!$("suggest").hidden || !$("formula-call").hidden)) {
    inspector.suggest.controller?.abort();
    inspector.suggest.away = inspector.assistKey();
    inspector.showSuggestions([]);
    event.preventDefault();
    event.stopPropagation();
  } else if (!$("suggest").hidden) {
    const n = inspector.suggest.items.length, down = event.key === "ArrowDown";
    if (down || event.key === "ArrowUp") inspector.chooseSuggestion(inspector.suggest.active < 0 ? (down ? 0 : n - 1) : (inspector.suggest.active + (down ? 1 : -1) + n) % n);
    else if ((event.key === "Enter" || (event.key === "Tab" && !event.shiftKey)) && inspector.suggest.active >= 0) inspector.acceptSuggestion();
    else if (event.key === "Enter" && !event.shiftKey) inspector.leaveFormula(event.target, true);
    else return;
    event.preventDefault();
    event.stopPropagation();
  } else if ((event.key === "Enter" && !event.shiftKey) || event.key === "Escape") {
    event.preventDefault();
    inspector.leaveFormula(event.target, event.key === "Enter");
  }
});
$("inspector").addEventListener("focusin", (event) => {
  if (event.target.id !== "formula") return;
  inspector.suggestHere();
});
withRest(() => {
  document.addEventListener("selectionchange", () => document.activeElement?.id === "formula" && inspector.suggestHere());
});
$("inspector").addEventListener("focusout", (event) => event.target.id === "formula" && inspector.dropSuggestions());
$("inspector").addEventListener("scroll", (event) => {
  if (event.target.id === "formula") $("formula-mirror").scrollTop = event.target.scrollTop;
}, true);
$("inspector").addEventListener("mousedown", (event) => {
  if (closest(event, ".assist")) event.preventDefault();
  const option = closest(event, "[data-suggest]");
  if (!option) return;
  event.preventDefault();
  inspector.acceptSuggestion(Number(option.dataset.suggest));
});
$("inspector").addEventListener("click", (event) => {
  const button = closest(event, "button");
  if (!button) return;
  const { type, dash, shape, fill, solid, hollow, place, side, priceLine, outline, fit, results, action, insert, index, reset, colorSelect, replace, lineColor, lineDash, lineTag, levelMenu, act, colorSlot: slot, pickColor, colorSource: source } = button.dataset;
  const paletteKey = button.closest("[data-palette]")?.getAttribute("data-palette");
  if (index) menus.switchInterval(index);
  else if (slot) (button.matches(".gradient-stop") && event.detail) || inspector.colorSlot.set(`${chart.id}:${selected}`, slot), inspector.renderInspector();
  else if (source && paletteKey) {
    const layers = inspector.inHand();
    inspector.colorSource.set(`${chart.id}:${layers.map((item) => item.id).join()}:${paletteKey}`, { source, color: source === "auto" ? undefined : inspector.sharedBy(layers, (item) => inspector.ownColor(item, paletteKey)) });
    if (source === "auto") inspector.editLayer((item) => inspector.setColor(item, paletteKey, ""));
    inspector.renderInspector();
  } else if (pickColor && paletteKey) inspector.editLayer((item) => inspector.setColor(item, paletteKey, pickColor));
  else if (colorSelect) inspector.underColorMenu(button);
  else if (act !== undefined) {
    const row = inspector.layerActions(selected, "panel")[Number(act)], at = chart.layers.findIndex((layer) => layer.id === selected);
    if (!row) return;
    if (row.disabled) return notice(row.disabled);
    if (row.page) return void ((page) => menus.menuOf(button, page.options?.() ?? page.rows()))(row.page());
    row.run?.();
    if (row.key === "remove") (all$(".layer-row .row-main", $("layer-list"))[Math.min(at, chart.layers.length - 1)] ?? /** @type {HTMLElement | null} */ (document.querySelector("#empty [data-action='add-series']")))?.focus();
  }
  else if (action === "pane-menu") inspector.paneMenu(button, inspector.inHand().map((layer) => layer.id), () => presetsEdit.refocus("#pane"));
  else if ((action === "gradient-add" || button.dataset.stopRemove) && button.getAttribute("aria-disabled") === "true") notice(/** @type {string} */ (button.dataset.tip));
  else if (action === "gradient-add") inspector.gradientEdit((item) => inspector.addStop(item), true);
  else if (button.dataset.stopRemove) inspector.gradientEdit((item) => inspector.removeStop(item, Number(button.dataset.stopRemove)), true);
  else if (action === "gradient-ready") menus.menuOf(button, /** @type {any} */ (inspector.readyPage(selected).options?.() ?? []));
  else if (action === "gradient-reverse") inspector.reverseGradient(selected), presetsEdit.refocus('#inspector [data-action="gradient-reverse"]');
  else if (button.dataset.gradientTo !== undefined) inspector.editLayer((item) => (item.upToDate = button.dataset.gradientTo ? true : undefined));
  else if (action === "heat-menu") ((row) => (row?.disabled ? notice(row.disabled) : row?.page && ((page) => menus.menuOf(button, page.options?.() ?? page.rows()))(row.page())))(heatmaps.heatRows(selected).find((row) => row.key === button.dataset.heat));
  else if (button.dataset.heatSet) inspector.editLayer((item) => {
    const value = button.dataset.value ?? "";
    if (button.dataset.heatSet === "measure") reshape(item, (one) => (one.measure = value === DEFAULT_MEASURE ? undefined : value));
    else if (button.dataset.heatSet === "view") item.cumulative = value === "cumulative" || undefined;
    else item.weight = value || undefined;
  });
  else if (button.id === "trend-ref" || button.id === "signal-ref" || button.id === "bubble-ref") inspector.refMenu(button, selected);
  else if (button.dataset.bubblesScale !== undefined) inspector.editLayer((layer) => (layer.scaleTo = button.dataset.bubblesScale === "all" ? "all" : undefined));
  else if (button.id === "signal-times") noValues(selected) ? notice(noValues(selected)) : command.openPage(command.timesPage(selected));
  else if (place) inspector.editLayer((layer) => (layer.place = place === "above" ? undefined : place));
  else if (action === "format-menu") inspector.formatMenu(button);
  else if (action === "color-mode") menus.menuOf(button, inspector.colorModeChoices(selected));
  else if (action === "color-from") menus.menuOf(button, inspector.fromChoices(selected));
  else if (type && button.getAttribute("aria-disabled") === "true") notice(/** @type {string} */ (button.dataset.tip));
  else if (type) inspector.editLayer((layer) => reshape(layer, (item) => (item.type = /** @type {Layer["type"]} */ (type))));
  else if (button.dataset.cohortOf) heatmaps.toggleCohort(selected, button.dataset.cohortOf);
  else if (button.dataset.cohortsIn || button.dataset.cohortsOut) button.getAttribute("aria-disabled") !== "true" && heatmaps.setCohorts(selected, /** @type {"holders" | "ages"} */ (button.dataset.cohortsIn ?? button.dataset.cohortsOut), Boolean(button.dataset.cohortsIn));
  else if (button.dataset.memberOf) {
    const member = button.dataset.memberOf;
    inspector.editLayer((layer, doc) => (layer.from = fromOf(doc, toggleMember(doc, layer.from ?? [], member), layer.type)));
    presetsEdit.refocus(`#inspector [data-member-of="${CSS.escape(member)}"]`);
  }
  else if (dash) inspector.editLayer((layer) => (layer.dash = Number(dash)));
  else if (shape) inspector.editLayer((layer) => (layer.shape = shape === "straight" ? undefined : shape));
  else if (fill) inspector.editLayer((layer) => (layer.fill = fill === "above" ? "above" : undefined));
  else if (solid !== undefined) inspector.editLayer((layer) => (layer.solid = solid === "1" || undefined));
  else if (hollow !== undefined) inspector.editLayer((layer) => (layer.hollow = hollow === "1" ? undefined : "none"));
  else if (action === "add-level") inspector.addLevel();
  else if (lineColor) inspector.lineMenu(button, lineColor, "color");
  else if (lineDash) inspector.lineMenu(button, lineDash, "dash");
  else if (levelMenu) menus.menuOf(button, inspector.levelRows(selected, Number(levelMenu)));
  else if (lineTag) {
    const layer = layerById(selected);
    if (!layer || !lineAt(layer, lineTag)) return;
    if (lineLook(layer, lineTag).line === false) return notice(inspector.TAG_NEEDS_LINE);
    presetsEdit.refocus(`[data-line-tag="${lineTag}"]`);
    const on = !lineLook(layer, lineTag).tag;
    inspector.setLineLook(selected, lineTag, "tag", on || undefined);
    // (The panel isn't drawn again under a level's value being typed (Safari leaves the focus there): the button says it.)
    button.setAttribute("aria-pressed", String(on));
    button.dataset.tip = on ? "Its value on the axis" : "Show its value on the axis";
  }
  else if (side) inspector.editLayer((layer) => (layer.side = side));
  else if (priceLine) inspector.editLayer((layer) => (layer.priceLine = priceLine === "1" || undefined));
  else if (outline) inspector.editLayer((layer) => (layer.outline = outline === "1" || undefined));
  else if (fit) inspector.editLayer((layer) => inspector.setFit(layer, fit));
  else if (results) inspector.editLayer((layer) => (layer.results = results === "1" || undefined));
  else if (insert) inspector.insertAtCaret(insert);
  else if (reset) {
    inspector.editLayer((layer) => DEFAULTS[reset].set(layer));
    presetsEdit.refocus(DEFAULTS[reset].focus);
  } else if (action === "deselect") deselect();
  else if (action === "more") inspector.toggleDescription(button);
  else if (action === "change-series") inspector.changeSeries(selected);
  else if (action === "copy-name") inspector.copyName(layerById(selected)?.series ?? "");
  else if (action === "replace") {
    const layer = layerById(selected);
    if (layer?.series) openPalette({ replace: layer.id, query: layer.series.replace(/_/g, " ") });
  } else if (replace) inspector.replaceSeries(selected, replace);
  else if (action === "retry") {
    const layer = layerById(selected);
    forget(layer?.series, chart.index);
    retryInfo(layer?.series ?? "");
    draw();
  }
});
withRest(() => {
  document.addEventListener("contextmenu", (event) => {
    if (byTouch && closest(event, "#chart-list") && !typing(event.target)) return void event.preventDefault();
    // (The browser's own, after the menu key opened a row's menu: that one's open.)
    if (event.isTrusted && performance.now() - grid.keyMenuAt < 500 && closest(event, "#chart-list")) return void event.preventDefault();
    const railItem = onRail() && !closest(event, ".row-rename, .folder-rename") ? /** @type {HTMLElement | null} */ (closest(event, "#chart-list > *")?.querySelector(":scope > :is([data-open-chart], [data-open-preset], [data-folder-item], [data-section-item], [data-group-item])") ?? null) : null;
    if (railItem) {
      event.preventDefault();
      return railMenuKept(railItem);
    }
    const tile = closest(event, "[data-home-card]");
    if (tile && !closest(event, ".tile-rename")) {
      event.preventDefault();
      if (TOUCH) return;
      return tileMenu(tile.dataset.homeCard, { x: event.clientX, y: event.clientY });
    }
    const presetAt = closest(event, "[data-preset], [data-preset-row]");
    if (presetAt) {
      event.preventDefault();
      if (TOUCH && presetAt.matches("[data-preset]")) return;
      const more = /** @type {HTMLElement} */ (presetAt.querySelector("[data-preset-menu]"));
      return presetsEdit.openPresetMenu(more, { x: event.clientX, y: event.clientY });
    }
    const groupAt = closest(event, "#chart-list [data-group]");
    if (groupAt) {
      event.preventDefault();
      return menus.menuOf(/** @type {HTMLElement} */ (groupAt.querySelector(railed() ? ".list-head-name" : "[data-group-menu]")), grid.groupRows(/** @type {Group} */ (groupAt.dataset.group)), { x: event.clientX, y: event.clientY });
    }
    const sectionRow = closest(event, "[data-section-row]");
    if (sectionRow) {
      event.preventDefault();
      return menus.menuOf(/** @type {HTMLElement} */ (sectionRow.querySelector(railed() ? ".folder-item" : "[data-section-menu]")), grid.sectionRows(/** @type {string} */ (sectionRow.dataset.sectionRow)), { x: event.clientX, y: event.clientY });
    }
    const folderRow = closest(event, "[data-folder-row], [data-grid-folder]");
    if (folderRow && !closest(event, ".folder-rename")) {
      event.preventDefault();
      const more = /** @type {HTMLElement | null} */ (folderRow.querySelector("[data-folder-menu]"));
      return more ? grid.folderActions(railed() && folderRow.matches("[data-folder-row]") ? /** @type {HTMLElement} */ (folderRow.querySelector(".folder-item")) : more, /** @type {string} */ (more.dataset.folderMenu), { x: event.clientX, y: event.clientY }) : undefined;
    }
    const previewRow = closest(event, "[data-preview-row]:not(:has(.row-rename))");
    const previewed = previewRow && chartById(/** @type {string} */ (/** @type {HTMLElement} */ (previewRow).dataset.previewRow));
    if (previewRow && previewed?.layers.length) {
      event.preventDefault();
      return grid.chartMenu(/** @type {HTMLElement} */ (previewRow.querySelector("[data-open-chart]")), previewed.id, { x: event.clientX, y: event.clientY });
    }
    const chartRow = closest(event, "[data-chart-row]:not(:has(.row-rename))");
    const layerRow = closest(event, ".layer-row[data-layer]");
    const paneHead = closest(event, ".pane-head");
    if (paneHead) {
      event.preventDefault();
      const more = /** @type {HTMLElement} */ (paneHead.querySelector("[data-pane-menu]"));
      return presetsEdit.paneActions(more, /** @type {string} */ (more.dataset.paneMenu), { x: event.clientX, y: event.clientY });
    }
    if (!chartRow && !layerRow) return;
    event.preventDefault();
    const point = { x: event.clientX, y: event.clientY };
    if (chartRow) grid.chartMenu(/** @type {HTMLElement} */ (chartRow.querySelector("[data-open-chart]")), /** @type {string} */ (chartRow.dataset.chartRow), point);
    else if (TOUCH || !layerRow) return;
    else if (layerPicks.has(/** @type {string} */ (layerRow.dataset.layer))) inspector.layersMenu(/** @type {HTMLElement} */ (layerRow.querySelector("[data-select]")), point);
    else {
      inspector.layerMenu(/** @type {HTMLElement} */ (layerRow.querySelector("[data-select]")), /** @type {string} */ (layerRow.dataset.layer), point);
      if (layerPicks.size && !$("menu").hidden) menus.outlineMenuTarget(layerRow);
    }
  });
});

const STACKED_ONLY = `A chart holds up to ${MAX_PANES} panes, and the last is a stack: this needs a pane of its own`;
/** @param {string} id @param {"cycles" | "years"} which */
function comparePeriods(id, which, over = false) {
  const layer = layerById(id);
  if (!layer) return;
  const periods = inspector.periodsOf(layer, which, over), what = which === "cycles" ? "halving cycles" : "years";
  if (!periods.length) return void notice(inspector.overShort(which));
  if (chart.panes.length >= MAX_PANES) return void notice(`A chart holds up to ${MAX_PANES} panes: the ${what} need one of their own`);
  const unfit = unfitHere([periods[0].formula]);
  if (unfit) return void notice(unfit);
  /** @type {Layer | undefined} */
  let first;
  commit((doc) => {
    const pane = newPaneIn(doc);
    pane.mode = 1;
    periods.forEach(({ formula, label }, k) => {
      const added = makeLayer({ formula, label, color: recencyColor(k, periods.length), pane: pane.id, ...(over && k === periods.length - 1 ? { width: 2 } : {}) }, doc);
      doc.layers.push(added);
      first ??= added;
    });
  });
  const go = ++dateGoes, at = revision;
  const drawn = () => revision === at && (!settled() ? command.afterDrawn(go, drawn) : applyRange({ ...chart, range: "max" }, /** @type {Resolved} */ (resolved), true));
  command.afterDrawn(go, drawn);
  toast(over ? `Added ${labelOf(layer)}'s ${what} over each other: each over the ${periods.length - 1} before it` : `Added ${labelOf(layer)}'s ${periods.length} ${what}, each from its start`, inspector.editAdded(first));
}

let putAway = /** @type {any} */ (null);
const putAside = (anywhere = false) => (putAway = { chart: chart.id, home, browsing: command.palette.op.kind === "browse", anywhere, state: { path: command.palette.path.slice(), pages: command.palette.pages, root: command.palette.root, query: command.palette.query, landOn: command.palette.items[command.palette.active]?.key ?? null } });
let dismissing = false;
function dismissPalette(leaving = false) {
  dismissing = true;
  putAside();
  if (leaving) quickHide(), grid.leave($("palette"));
  else $("palette").hidePopover();
}
$("palette").addEventListener("cancel", () => ((dismissing = true), putAside()));
onClosed($("palette"), () => {
  if (!dismissing && !isOpen($("palette")) && (command.presetPicked() || command.seriesPicked())) putAside(command.presetPicked());
  dismissing = false;
  const left = isOpen($("palette")) ? -1 : command.palette.pages.findIndex((page) => page.left);
  if (left >= 0) command.leavePages(left);
});
let paletteOnField = false, paletteArrivals = 0;
const keepOnHome = () => {
  if (!isOpen($("palette"))) return;
  const was = $("palette").getBoundingClientRect();
  command.placeOnHome();
  const now = $("palette").getBoundingClientRect();
  if (now.x === was.x && now.y === was.y && now.width === was.width && now.height === was.height) return;
  if (menus.ownerNow()?.closest("#palette")) command.paletteMenuGone();
  trailDrawn = "";
  command.renderPath();
};
new ResizeObserver(keepOnHome).observe($("home"));
new MutationObserver(() => command.underHome()).observe($("palette"), { attributes: true, attributeFilter: ["data-on-home", "data-leaving", "data-arriving"] });
$("palette").addEventListener("beforetoggle", (event) => command.underHome(/** @type {ToggleEvent} */ (event).newState === "open"));
viewport.addEventListener("resize", keepOnHome);
viewport.addEventListener("scroll", keepOnHome);
function openPalette({ add = false, replace = /** @type {string | null} */ (null), query = "", formula = false, editing = /** @type {string | null} */ (null), into = /** @type {string | null} */ (null), path = /** @type {string[] | null} */ (null), landOn = /** @type {string | null} */ (null), resume = false, typing = false, signal = false, side = /** @type {string | null} */ (null), bubbles = false, onField = false } = {}) {
  paletteOpenings++;
  const spot = resume && putAway?.browsing && (putAway.anywhere || (putAway.chart === chart.id && putAway.home === home)) ? putAway.state : null;
  putAway = null;
  countSaidFor = "";
  menus.hideMenu();
  const back = add && !query && command.addedFrom?.chart === chart.id ? command.addedFrom : null;
  if (!spot) command.leavePages(0);
  Object.assign(command.palette, spot ?? { pages: [], root: {}, query, path: path ?? (back ? back.path.slice() : []), landOn: landOn ?? back?.key ?? null });
  paletteShown = null;
  command.palette.op = replace ? { kind: "replace", layer: replace } : formula ? { kind: "formula", editing, into, signal, side } : add ? { kind: "add", into, bubbles } : { kind: "browse" };
  const dialog = $("palette"), input = $("palette-input");
  if (!isOpen(dialog)) (trailDrawn = ""), dialog.removeAttribute("data-half");
  // (By touch, opened with the focus on ⌘K, not its field: no keyboard till it's tapped (the Search tab too: a keyboard
  // comes only when asked for). Opened by a tap on what's drawn as a field (an empty chart's search), its field at
  // once: the keyboard with it, as that tap asked, in it for iOS to allow it.)
  const keys = !TOUCH || byKeys || typing;
  if (dialog.dataset.leaving) (dismissing = false), grid.arrive(dialog);
  if (!isOpen(dialog)) (paletteOnField = onField), command.placeOnHome(), dialog.toggleAttribute("data-arriving", dialog.hasAttribute("data-on-home") && !reduced());
  if (dialog.hasAttribute("data-arriving")) ((arrived) => (requestAnimationFrame(() => requestAnimationFrame(arrived)), setTimeout(arrived, 100)))(((arrival) => () => arrival === paletteArrivals && dialog.removeAttribute("data-arriving"))(++paletteArrivals));
  if (!isOpen(dialog)) (input.inert = !keys), grid.showDialog(dialog), (input.inert = false), !keys && dialog.focus();
  if (typing && TOUCH) input.focus();
  indexCatalog();
  command.applyScope();
  input.value = command.palette.query;
  command.paletteKeys(true);
  showPalette();
}
if (TOUCH) $("palette").tabIndex = -1;
let byKeys = false;
let byTouch = false;
addEventListener("keydown", (event) => ((byKeys = true), (byTouch = false), TOUCH && command.keyboardKey(event) && command.keyed(true)), true);
// (A touch heard at all, as WebKit needs to show a control pressed, :active, under a finger.)
addEventListener("touchstart", () => {}, { passive: true });
addEventListener("pointerdown", (event) => ((byKeys = false), (byTouch = event.pointerType === "touch"), command?.keyed(false)), true);
addEventListener("resize", () => isOpen($("palette")) && ((trailDrawn = ""), command.renderPath()));
$("palette-scope").addEventListener("click", () => {
  if (command.palette.path.length) {
    command.goUp(0);
    command.paletteKeys();
  } else if (command.palette.adding || command.palette.formula) command.leaveScope();
});
let trailDrawn = "";
function drawTrail(crumbs, current) {
  const bar = $("palette-path"), key = JSON.stringify([crumbs, current, bar.hidden]);
  if (key === trailDrawn) return;
  trailDrawn = key;
  if (bar.hidden) return void (bar.innerHTML = "");
  const draw = (from) => {
    const shown = from ? [["…", crumbs[from - 1][1]], ...crumbs.slice(from)] : crumbs;
    bar.innerHTML = shown.map(([text, depth], i) => `${i ? '<i aria-hidden="true">›</i>' : ""}<button type="button" data-depth="${depth}" ${depth === current ? 'aria-current="location"' : ""} ${text === "…" ? 'aria-label="Up a level" data-tip="Up"' : ""}>${escapeHtml(String(text))}</button>`).join("");
  };
  let from = 0;
  bar.classList.remove("tight");
  draw(from);
  while (!bar.hidden && bar.scrollWidth > bar.clientWidth && from < crumbs.length - 1) draw(++from);
  bar.classList.toggle("tight", bar.scrollWidth > bar.clientWidth);
}
function showPalette() {
  pointedOption = false;
  const query = command.palette.query.trim();
  command.palette.pending = null;
  command.palette.controller?.abort();
  $("palette-glow").hidden = true;
  command.renderPath();
  if (command.palette.formula) return command.showFormula();
  if (command.palette.pages.length) return command.showPage();
  if (query && command.palette.path.length) {
    const words = query.toLowerCase().split(/[\s_]+/).filter(Boolean);
    const found = new Set();
    const stack = [[nodeAt(command.palette.path), ""]];
    while (stack.length && found.size < 200) {
      const [node, keys] = /** @type {[any, string]} */ (stack.pop());
      if (isLeaf(node)) {
        if (words.every((word) => `${node.name} ${keys}`.toLowerCase().includes(word))) found.add(node.name);
      } else if (node && typeof node === "object") for (const key of Object.keys(node).reverse()) stack.push([aside(node, key), `${keys} ${humanizeKey(key)}`]);
    }
    const here = [...found].map((name) => ({ kind: "series", key: name, name, group: "In this folder" }));
    command.palette.items = here;
    command.palette.active = 0;
    command.palette.moved = false;
    $("palette-glow").hidden = false;
    renderPalette(true);
    return command.searchSeries(query, (names) => [...here, ...(names ?? []).filter((name) => isSeriesName(name) && !found.has(name)).map((name) => ({ kind: "series", key: name, name, group: "Every series" }))]);
  }
  if (!query) {
    const node = nodeAt(command.palette.path);
    const folder = (command.palette.path.length ? Object.keys(node ?? {}) : command.catalogTops(node)).map((key) => {
      const child = node[key];
      const group = command.palette.path.length ? "" : "Catalog";
      return isLeaf(child) ? { kind: "series", key, name: child.name, group } : { kind: "folder", key, path: [...command.palette.path, key], group };
    }).filter((item) => item.kind === "series" || (node[item.key] && typeof node[item.key] === "object"));
    const root = !command.palette.path.length;
    const browsing = command.palette.adding || Boolean(command.palette.replacing) || !root;
    const lastCharts = root ? recentCharts().filter((item) => home || item.id !== chart.id).slice(0, 3).map((item) => ({ ...command.recentRow(item), group: "Recent" })) : [];
    const recents = root && browsing ? recent().map((name) => ({ kind: "series", key: name, name, group: "Recent series" })) : [];
    const suggestions = root && browsing ? SUGGESTED.filter((name) => !gone.has(name) && !recents.some((item) => item.name === name)).map((name) => ({ kind: "series", key: name, name, group: "Suggested" })) : [];
    const suggested = root ? command.commands().filter((command) => command.suggested).map(command.commandItem) : [];
    const usual = root && !browsing ? command.byUse([...command.commands().filter((command) => command.run).map((command) => ({ kind: "command", key: command.title, command })), ...command.browseRows(), ...(home ? [] : [...command.settingsNow().rows, ...chart.layers.map(command.layerRow), ...chart.panes.map((pane, i) => command.paneRow(pane.id, i))])].filter((item) => command.useScore(item.key) > 0 && !command.dangerous(item) && !/^Download/.test(item.command?.title ?? item.label ?? ""))).slice(0, 3).map((item) => ({ ...item, group: "Frequently used" })) : [];
    const fresh = (item) => !usual.some((other) => other.key === item.key);
    const others = root && command.palette.adding ? command.otherLayerRows() : [];
    command.palette.items = browsing ? [...others, ...recents, ...suggestions, ...folder] : [...lastCharts, ...usual, ...command.browseRows().filter(fresh), ...suggested.filter(fresh)];
    command.palette.active = command.palette.items.findIndex((item) => (item.kind !== "series" || command.palette.path.length > 0) && item.key === command.palette.landOn);
    if (command.palette.active < 0) command.palette.active = Math.max(0, command.palette.adding && !home && !others.length ? command.palette.items.findIndex((item) => item.kind === "series" && command.actionable(item) && !chart.layers.some((layer) => layer.series === item.name)) : 0);
    command.palette.landOn = null;
    renderPalette();
    return;
  }
  const words = command.typedWords(query);
  const everything = !command.palette.adding && !command.palette.replacing;
  // A command leads only when the first word typed starts a word of its name ("download", "share"); otherwise (a word
  // of its description alone) it follows the series, which is what people search for. One of many of a kind (an
  // interval) leads only by its own name ("weekly", "1w"); by the kind ("interval"), the page that has them all, the
  // one in effect checked, does.
  const leads = (said) => ((own) => [...own.words, ...own.joins].some((word) => word.startsWith(words[0])))(command.textWords((said.follows ?? said.title).toLowerCase()));
  const found = !everything ? [] : command.rank(command.commands().map(command.commandItem), (item) => `${item.command.title} ${item.command.text ?? ""}`, words);
  const titled = (/** @type {Command} */ command) => command.title.toLowerCase().startsWith(words[0]), ahead = found.filter((item) => leads(item.command));
  const strong = [...ahead.filter((item) => titled(item.command)), ...ahead.filter((item) => !titled(item.command) && !item.command.danger), ...ahead.filter((item) => !titled(item.command) && item.command.danger)].slice(0, 6), weak = found.filter((item) => !leads(item.command)).slice(0, 6);
  const allOf = (/** @type {any[]} */ list, /** @type {string} */ group, /** @type {string} */ what, /** @type {() => Page} */ page, count = list.length, most = Infinity) => [...list.slice(0, 4), ...(count > 4 ? [{ kind: "row", key: `all:${group}`, icon: ICONS.browse, label: `All ${grouped.format(count)}${count >= most ? "+" : ""} ${what}`, verb: "Open", group, page: () => Object.assign(page(), { query }) }] : [])];
  const chartsFound = !everything ? [] : command.rank([...listedPreviews(), ...library.charts].map((doc) => ({ kind: "chart", key: doc.id, group: "Charts" })), command.rowText, words);
  const charts = allOf(chartsFound.filter((item) => home || item.key !== chart.id), "Charts", "charts found", command.chartsPage, chartsFound.length);
  const matched = everything && presets ? presetsMatching(words, 60) : null;
  const presetRows = matched ? allOf(matched.found.map((i) => ({ ...presetsEdit.presetRow(i, true), group: presetsEdit.presetsGroup(matched.partial) })), presetsEdit.presetsGroup(matched.partial), "presets found", () => presetsEdit.presetsPage([]), undefined, 60) : [];
  const settings = everything && !home ? command.settingsNow().rows : [];
  const own = !everything ? [] : home ? command.rank(command.browseRows(), command.rowText, words) : [
    ...command.rank(command.browseRows(), command.rowText, words),
    ...command.rank(chart.layers.map((layer) => ({ ...command.layerRow(layer), group: "Layers" })), command.rowText, words),
    ...command.rank(chart.panes.map((pane, i) => ({ ...command.paneRow(pane.id, i), group: "Panes" })), command.rowText, words),
    ...((choices) => ((said) => command.rank(settings.filter((row) => words.some((typed) => typed.length > 1 && (command.fitScore(row.itself, [typed]) || command.fitScore(choices(row).said, [typed])))), said, words, said))((/** @type {any} */ row) => `${command.wayText(row)} ${choices(row).said}`).slice(0, 4).map((row) => {
      const asked = words.filter((typed) => typed.length > 1 && !command.fitScore(command.wayText(row), [typed]));
      const chosen = asked.length ? choices(row).choices.map((choice) => /** @type {[any, number]} */ ([choice, command.fitScore(choice.label ?? "", asked)])).reduce((best, next) => (next[1] > best[1] ? next : best), [null, 0])[0] : null;
      return chosen?.key == null ? row : { ...row, aside: chosen.label ?? row.aside, page: () => ((next) => ((command.palette.landOn = String(chosen.key)), next))(row.page()) };
    }))(command.settingChoices()),
  ];
  const span = everything && !home && chart.layers.length ? command.readDate(query.replace(/^go ?to\s+/i, "")) : null;
  const asFormula = everything && !home && !span && command.isOwnFormula(query.trim()) ? [{ kind: "formula", key: "formula", group: "Formula" }] : [];
  const dated = span && !command.outOfData(span) ? [{ kind: "row", key: "go-to-date", icon: ICONS.calendar, label: `Go to ${span.label}`, group: "Go to", verb: "Go", run: () => goToDate(span), preview: (/** @type {HTMLElement} */ panel) => command.previewDate(panel, span) }] : [];
  const others = command.palette.adding && !command.palette.path.length ? command.rank(command.otherLayerRows(), command.rowText, words) : [];
  const leading = others.filter((row) => row.label.toLowerCase().startsWith(words[0])), trailing = others.filter((row) => !leading.includes(row));
  const local = [...leading, ...asFormula, ...dated, ...strong, ...own, ...charts, ...presetRows, ...trailing], more = weak.map((item) => ({ ...item, group: "More commands" }));
  command.palette.moved = false;
  const landOn = command.palette.landOn;
  command.palette.landOn = null;
  const landing = (items) => Math.max(0, items.findIndex((item) => item.key === landOn));
  const searching = () => {
    command.palette.pending = null;
    command.palette.items = local;
    command.palette.active = landing(local);
    command.palette.moved = false;
    $("palette-glow").hidden = false;
    renderPalette(true);
  };
  if (command.finder.ready) command.palette.pending = searching;
  else searching();
  command.searchSeries(query.replace(/^add\s+(?=\S)/i, ""), (names) => {
    if (!names) return [...local, ...more];
    const typedParts = words.flatMap((typed) => typed.split("_")).filter(Boolean);
    const seriesByName = !command.palette.moved && names.some((name) => typedParts.every((typed) => name.split("_").some((part) => part.startsWith(typed))));
    const whole = query.trim().toLowerCase().replace(/^add\s+(?=\S)/, "").replace(/\s+/g, " ");
    const exact = names.find((name) => isSeriesName(name) && (name === whole || humanize(name).toLowerCase() === whole));
    const series = names.filter((name) => isSeriesName(name) && name !== exact).map((name) => ({ kind: "series", key: name, name, group: exact ? "More series" : "Series" }));
    const rows = [...(exact ? [{ kind: "series", key: exact, name: exact, group: "Series" }] : []), ...leading, ...asFormula, ...dated, ...strong, ...(seriesByName ? own.slice(0, 3) : own), ...charts, ...presetRows, ...series, ...trailing, ...more];
    return rows.length || !everything ? rows : command.browseInstead();
  }, landing);
}
withRest(() => {
  document.addEventListener("selectionchange", () => {
    const input = /** @type {HTMLInputElement} */ ($("palette-input"));
    if (document.activeElement !== input || !command.palette.formula || !isOpen($("palette"))) return;
    if (`${input.value} ${input.selectionStart} ${input.selectionEnd}` !== command.formulaAt) command.showFormula();
  });
});
function renderPalette(pending = false, note = "") {
  inspector.updateCallHelp();
  const searching = Boolean(command.palette.query.trim());
  const list = $("palette-list");
  const tail = pending ? `<li class="note"><span class="shimmer">Searching ${seriesCount ? grouped.format(seriesCount) : "every"} series…</span></li>` : note ? `<li class="note">${escapeHtml(note)}</li>` : "";
  command.palette.items = command.palette.items.filter((item) => ((command.palette.replacing || command.palette.adding) && !command.palette.pages.length ? item.kind === "series" || item.kind === "folder" || (command.palette.adding && item.other) : item.kind !== "chart" || Boolean(chartById(item.key))));
  if (menus.ownerNow()?.closest("#palette-list")) command.paletteMenuGone();
  $("palette").toggleAttribute("data-none", !command.palette.items.length && !pending);
  if (!command.palette.items.length) {
    list.innerHTML = tail || `<li class="note no-results">${searching ? "Nothing matches. Try other words, or describe what a series measures." : "Nothing here."}</li>`;
    $("palette-input").removeAttribute("aria-activedescendant");
    $("palette-preview").innerHTML = "";
    $("palette-look").hidden = true;
    clearLook();
    if (quick) quickShow();
    updatePaletteKeys();
    return;
  }
  command.palette.active = Math.min(command.palette.active, Math.max(0, command.palette.items.length - 1));
  if (!command.palette.moved && command.palette.active === 0 && !command.palette.pages.at(-1)?.input && !command.litFirst(command.palette.items[0])) command.palette.active = command.actionable(command.palette.items[0]) ? -1 : command.palette.items.findIndex(command.litFirst);
  // A new list starts at its top (set before it's written: after, it would lay the list out at once); the same list
  // drawn again (a row acted on that stays, ⌘↵, results arriving) stays where it was scrolled, as many rows drawn.
  const same = paletteShown !== null && paletteShown.page === (command.palette.pages.at(-1) ?? null) && paletteShown.query === command.palette.query && paletteShown.path === command.palette.path.join("/");
  const kept = same ? list.scrollTop : 0;
  if (!same) list.scrollTop = 0;
  paletteShown = { page: command.palette.pages.at(-1) ?? null, query: command.palette.query, path: command.palette.path.join("/") };
  command.palette.drawn = Math.min(command.palette.items.length, Math.max(command.PALETTE_PAGE, command.palette.active + command.PALETTE_PAGE / 2, same ? command.palette.drawn ?? 0 : 0));
  list.innerHTML = command.optionsHtml(0, command.palette.drawn) + (tail ? tail.replace("<li ", "<li data-tail ") : "");
  grid.forgetPaletteMarks();
  grid.seePaletteMarks(list);
  if (same) list.scrollTop = kept;
  if (command.palette.drawn < command.palette.items.length) requestAnimationFrame(() => list.scrollHeight <= list.clientHeight + 400 && command.drawOptions(command.palette.drawn + command.PALETTE_PAGE));
  drawKeys(list);
  // A row landed on (coming back up) is brought into view, with its neighbors, where it's out of sight (in sight, the
  // list stays at its top: a short one's first heading cut by a scroll to its middle, for nothing).
  ((row) => row && ((box, seen) => box.bottom > seen.bottom || box.top < seen.top)(row.getBoundingClientRect(), list.getBoundingClientRect()) && row.scrollIntoView({ block: same ? "nearest" : "center" }))(command.palette.active > 0 ? $(`option-${command.palette.active}`) : null);
  command.activeOption();
  command.preview();
}
/** @type {{ page: any, query: string, path: string } | null} */
let paletteShown = null;
let countSaidFor = "";
function updatePaletteKeys() {
  const item = command.palette.items[command.palette.active];
  const twin = !home && item?.kind === "series" && chart.layers.some((layer) => layer.series === item.name);
  const adds = home ? "New chart with it" : twin ? "Add again" : command.palette.into === "new" && chart.layers.length ? "Add to a new pane" : "Add to chart";
  $("palette-primary").textContent = !item ? adds : item.kind === "command" ? (item.command.run ? "Run" : "Close") : item.kind === "chart" ? "Open chart" : item.kind === "folder" ? "Open folder" : command.palette.replacing ? "Use instead" : adds;
  if (item?.kind === "insert") $("palette-primary").textContent = item.recipe ? adds : "Insert";
  if (item?.kind === "row") $("palette-primary").textContent = item.verb ?? (item.page ? "Open" : "Run");
  if (item?.kind === "formula" && command.palette.editing) $("palette-primary").textContent = "Apply";
  $("palette-more").hidden = command.actionsFor(item).length < 2;
  $("palette-enter").hidden = $("palette-close-sep").hidden = !item || (item.kind !== "formula" && item.kind !== "insert" && item.kind !== "folder" && item.kind !== "chart" && !command.actionable(item));
  $("palette-count").hidden = command.palette.pages.length > 0 || command.palette.formula;
  if (!$("palette-glow").hidden) $("palette-count").textContent = "";
  else if (command.palette.query.trim()) {
    const found = command.palette.items.filter((row) => !row.elsewhere).length, more = command.palette.items.filter((row) => row.kind === "series").length >= 80;
    $("palette-count").textContent = found ? `${grouped.format(found)}${more ? "+" : ""} ${found === 1 && !more ? "result" : "results"}` : "No results";
    const said = `${command.palette.query}|${$("palette-count").textContent}`;
    if (said !== countSaidFor && isOpen($("palette")) && !$("palette-count").hidden) (countSaidFor = said), say($("palette-count").textContent);
  } else $("palette-count").textContent = seriesCount ? `${grouped.format(seriesCount)} series on Bitview` : "Bitview";
}

function move(delta, scroll = true) {
  if (!command.palette.items.length) return;
  command.palette.moved = true;
  const n = command.palette.items.length;
  command.palette.active = stepIn(command.palette.active, delta, n, Math.abs(delta) === 1);
  pointedOption = !scroll;
  command.drawOptions(command.palette.active + command.PALETTE_PAGE / 2);
  $("palette-list").querySelector('[aria-selected="true"]')?.setAttribute("aria-selected", "false");
  const row = $(`option-${command.palette.active}`);
  row?.setAttribute("aria-selected", "true");
  const head = row?.previousElementSibling;
  if (scroll && head?.classList.contains("group")) head.scrollIntoView({ block: "nearest" });
  if (scroll) row?.scrollIntoView({ block: "nearest" });
  command.activeOption();
  command.preview();
}
withRest(() => {
  phone.addEventListener("change", () => isOpen($("palette")) && !phone.matches && command.preview());
});

let looking = false;
/** @type {((doc: Chart) => void) | null} */
let lookShown = null;
/** @param {Chart} doc */
function withLook(doc) {
  if (!lookShown) return doc;
  lookShown(doc);
  normalize(doc);
  lookDoc = doc;
  return doc;
}
function showLook(look) {
  if (!resolved || !chartShown() || !isOpen($("palette"))) return clearLook();
  lookShown = look;
  build(withLook({ ...structuredClone(chart), index: shownIndex() }), resolved, false);
  looking = true;
  $("palette").toggleAttribute("data-looking", true);
}
function clearLook() {
  $("palette").removeAttribute("data-looking");
  lookDoc = null;
  lookShown = null;
  if (!looking) return;
  looking = false;
  if (resolved && chartShown()) build({ ...structuredClone(chart), index: shownIndex() }, resolved, false);
}
onClosed($("palette"), clearLook);
let used = /** @type {Record<string, { n: number, at: number }>} */ (Object.fromEntries(Object.entries(storage.get(USED, {})).filter(([, use]) => Number.isFinite(use?.n) && Number.isFinite(use?.at))));
function noteUse(key) {
  used[key] = { n: (used[key]?.n ?? 0) + 1, at: Date.now() };
  used = Object.fromEntries(Object.entries(used).sort(([a], [b]) => command.useScore(b) - command.useScore(a)).slice(0, 60));
  storage.set(USED, used);
}

/** @param {number} time */
function goToTime(time, go = ++dateGoes) {
  if (!settled()) return command.afterDrawn(go, () => goToTime(time, go));
  const range = shownSpan(), at = resolved ? lowerBound(resolved.times, time) : -1;
  if (!range || !resolved) return;
  if (resolved.times[at] !== time) return goToDate({ from: time, to: time + INTERVAL_DAYS[chart.index] * 86400 - 1, day: true, label: pointLabel(time, chart.index) }, 0, go);
  const half = Math.min(command.AROUND, range.to - range.from) / 2;
  autoScale();
  glide({ from: at - half, to: at + half });
}
let paletteOpenings = 0;
const tabletHead = matchMedia("(760px < width <= 1100px) and (height > 500px)");
tabletHead.addEventListener("change", () => tabletHead.matches && document.activeElement?.matches("#add, #draw-toggle") && $("chart-more").focus({ preventScroll: true }));

let dateGoes = 0;
function goToDate(span, by = 0, go = ++dateGoes) {
  if (!span || command.outOfData(span)) return;
  // (Any other draw under way (another Go's, a block come in) would take the view back as it lands: once it has,
  // from what it drew.)
  if (!settled()) return void command.afterDrawn(go, () => goToDate(span, by, go));
  const loaded = by > 0 && by === resolved?.version;
  const held = spanOf(chart), stretch = command.stretchOf(span);
  if (held && !loaded && !stretch.loaded) {
    const latest = latestChunk(chart.index), to = clamp(chunkAt(chart.index, stretch.to), 0, latest), from = clamp(chunkAt(chart.index, stretch.from), 0, to);
    const meets = to + 1 >= held.from && from <= held.to + 1, older = to < held.from;
    Object.assign(held, meets ? { from: Math.min(from, held.from), to: Math.max(to, held.to) } : { from, to });
    if (held.to - held.from + 1 > MOST_CHUNKS) {
      if (older) held.to = held.from + MOST_CHUNKS - 1;
      else held.from = held.to - MOST_CHUNKS + 1;
    }
    held.live = held.to >= latest;
    void draw();
    const mine = drawVersion;
    return void command.afterDrawn(go, () => goToDate(span, mine, go));
  }
  const { first, last } = command.pointsOf(span), range = shownSpan();
  autoScale();
  if (span.day || last - first < 8) {
    const half = range ? (range.to - range.from) / 2 : 60, center = (first + last) / 2;
    glide({ from: center - half, to: center + half });
  } else {
    const times = /** @type {Resolved} */ (resolved).times, middle = command.stretchOf(span);
    glide({ from: Math.max(first, lowerBound(times, middle.from)) - 0.5, to: Math.min(last, lowerBound(times, middle.to) - 1) + 0.5 });
  }
}

$("palette-input").addEventListener("input", (event) => {
  command.palette.query = event.target.value;
  showPalette();
});
$("palette-input").addEventListener("keydown", (event) => {
  if (composing(event)) return;
  const plain = !event.metaKey && !event.ctrlKey && !event.altKey;
  if ((event.metaKey || event.ctrlKey) && !event.altKey && event.key.toLowerCase() === "z" && !command.palette.formula && !command.palette.pages.at(-1)?.input) {
    event.preventDefault();
    event.shiftKey ? redoLast() : undoLast();
    showPalette();
    return command.paletteKeys();
  }
  if (["ArrowDown", "ArrowUp", "ArrowRight", "Enter", "Tab"].includes(event.key)) command.palette.pending?.();
  if (event.key === "ArrowDown" && plain) move(event.shiftKey ? 5 : 1);
  else if (event.key === "ArrowUp" && plain) move(event.shiftKey ? -5 : -1);
  else if (event.key === "Enter" && !event.altKey) event.repeat || command.choose(event.shiftKey, event.metaKey || event.ctrlKey);
  else if (event.key === "ArrowRight" && plain && !event.shiftKey && command.palette.items[command.palette.active] && command.goesOn(command.palette.items[command.palette.active]) && event.target.selectionStart === event.target.value.length) event.repeat || command.choose();
  else if ((event.key === "ArrowLeft" || event.key === "Backspace") && !event.target.value && command.palette.pages.length) event.repeat || command.popPage();
  else if ((event.key === "ArrowLeft" || event.key === "Backspace") && !event.target.value && command.palette.path.length) event.repeat || command.goUp(command.palette.path.length - 1);
  else if (event.key === "Backspace" && !event.target.value && (command.palette.adding || command.palette.formula)) event.repeat || command.leaveScope();
  else if (event.key === "Tab" && !event.shiftKey && command.palette.items[command.palette.active]?.kind === "insert") command.insertPiece(command.palette.items[command.palette.active]);
  else if ((event.metaKey || event.ctrlKey) && !event.altKey && !event.shiftKey && event.key.toLowerCase() === "y" && quickWorks()) quickToggle();
  // (esc puts Quick Look away, ⌘K staying: here, as a key, not as the dialog's cancel, which a browser lets be
  // stopped only once in a row.)
  else if (event.key === "Escape" && quick) quickHide();
  else return;
  event.preventDefault();
});
$("palette-list").addEventListener("scroll", (event) => {
  const list = /** @type {HTMLElement} */ (event.currentTarget);
  if (list.scrollTop + list.clientHeight > list.scrollHeight - 400) command.drawOptions((command.palette.drawn ?? 0) + command.PALETTE_PAGE);
}, { passive: true });
$("palette-list").addEventListener("click", (event) => {
  const option = closest(event, "[data-option]"), more = closest(event, "[data-row-more]");
  if (!option) return;
  if (command.palette.pending) return command.palette.pending();
  if (more) return void (move(Number(more.dataset.rowMore) - command.palette.active, false), command.showActions(more));
  command.palette.active = Number(option.dataset.option);
  command.choose(event.shiftKey);
});
$("palette-list").addEventListener("mousemove", (event) => {
  if (command.palette.pending) return command.palette.pending();
  const option = closest(event, "[data-option]");
  if (option && Number(option.dataset.option) !== command.palette.active) move(Number(option.dataset.option) - command.palette.active, false);
});
$("palette-enter").addEventListener("click", () => {
  command.palette.pending?.();
  command.choose(false);
  if (isOpen($("palette"))) command.paletteKeys();
});
$("palette-actions").addEventListener("click", () => command.showActions());
withRest(() => {
  gestures.onHold($("palette-list"), "[data-option]", (option) => ((held) => () => {
    if (command.palette.items[Number(option.dataset.option)] !== held) return false;
    command.palette.active = Number(option.dataset.option);
    if (command.actionsFor(command.palette.items[command.palette.active]).length < 2) return false;
    command.showActions(option);
    return true;
  })(command.palette.items[Number(option.dataset.option)]));
  gestures.onHold($("home"), ".tile[data-preset] > [data-open-preset]", (open) => ((more) => (more ? () => (presetsEdit.openPresetMenu(more), true) : null))(/** @type {HTMLElement | null} */ (open.parentElement?.querySelector("[data-preset-menu]") ?? null)));
});
$("palette-path").addEventListener("click", (event) => {
  const crumb = closest(event, "[data-depth]");
  if (!crumb || crumb.hasAttribute("aria-current")) return;
  if (command.palette.pages.length) command.popPage(Number(crumb.dataset.depth));
  else command.goUp(Number(crumb.dataset.depth));
  command.paletteKeys();
});

withRest(() => {
  menus.menuScrim.addEventListener("click", (event) => {
    event.stopPropagation();
    menus.dismissMenu();
  });
});
{
  let wide = innerWidth;
  addEventListener("resize", () => innerWidth !== wide && ((wide = innerWidth), !$("menu").hidden && menus.dismissMenu()));
}
withRest(() => {
  {
    const sheet = $("palette");
    const size = (/** @type {boolean} */ whole) => {
      sheet.toggleAttribute("data-half", !whole);
      if (!whole && document.activeElement === $("palette-input")) sheet.focus();
    };
    const menuIn = () => !$("menu").hidden && sheet.contains($("menu"));
    grid.heightsSheet(sheet, { pulls: () => phone.matches && isOpen(sheet) && !menuIn(), grab: (event) => phone.matches && isOpen(sheet) && !menuIn() && Boolean(closest(event, ".menu-handle")) && "handle", whole: () => !heldUpright.matches || !sheet.hasAttribute("data-half"), size, close: dismissPalette });
    $("palette-input").addEventListener("focus", () => sheet.hasAttribute("data-half") && grid.onceKeys(() => isOpen(sheet) && sheet.hasAttribute("data-half") && document.activeElement === $("palette-input") && grid.sizeSheet(sheet, () => size(true))));
  }
  {
    const menu = $("menu");
    let base = 0, shown = 0, height = 0, pulled = false;
    grid.dragSheet(menu, {
      pulls: () => menu.classList.contains("as-sheet") && !menu.hidden,
      grab: (event) => {
        if (!menu.classList.contains("as-sheet") || (event && !closest(event, ".menu-handle"))) return false;
        const back = grid.slides.get(menu), top = menu.getBoundingClientRect().top;
        if (back && back.id !== "away" && (back.playState === "running" || back.playState === "paused")) {
          back.cancel();
          menu.style.height = menu.style.maxHeight = "";
        }
        const past = Math.max(0, menu.getBoundingClientRect().top - top);
        base = past ? -grid.unstretched(past) : grid.sheetTaken(menu);
        height = menu.offsetHeight;
        if (past) (menu.style.maxHeight = "none"), (menu.style.height = `${height + past}px`);
        menu.style.transition = "none";
        return true;
      },
      move: (moved) => {
        const by = moved + base;
        pulled = by < 0;
        menu.style.maxHeight = pulled ? "none" : "";
        menu.style.height = pulled ? `${height + grid.stretched(-by)}px` : "";
        menu.style.transform = pulled ? "" : `translateY(${by}px)`;
        shown = menu.getBoundingClientRect().top;
      },
      back: () => (menu.style.transition = menu.style.transform = menu.style.height = menu.style.maxHeight = ""),
      release: (by, speed) => (pulled ? grid.sizeSheet(menu, () => {}, shown, speed) : Math.abs(by) >= 4 && grid.sheetGoes(Math.max(0, by + base), speed, menu.offsetHeight) && grid.slideAway(menu, menus.hideMenu, shown, speed)),
    });
  }
  for (const sheet of [$("shortcuts"), $("guide")]) {
    let base = 0, shown = 0;
    grid.dragSheet(sheet, {
      pulls: () => phone.matches && isOpen(sheet),
      grab: (event) => {
        if (!phone.matches || !isOpen(sheet) || (event && !closest(event, ".menu-handle"))) return false;
        base = grid.sheetTaken(sheet);
        sheet.style.transition = "none";
        return true;
      },
      move: (by) => ((sheet.style.transform = `translateY(${Math.max(0, by + base)}px)`), (shown = sheet.getBoundingClientRect().top)),
      back: () => (sheet.style.transition = sheet.style.transform = ""),
      release: (by, speed) => Math.abs(by) >= 4 && grid.sheetGoes(Math.max(0, by + base), speed, sheet.offsetHeight) && grid.slideAway(sheet, () => sheet.hidePopover(), shown, speed),
    });
  }
  menus.startCascade();
});
whenSubmenu((sub) => sub.addEventListener("keydown", (event) => {
  if (event.key === "ArrowLeft" || event.key === "Escape") {
    event.preventDefault();
    event.stopPropagation();
    return menus.hideSubmenu(levelOf(sub));
  }
  if (event.key === "Tab") return menus.dismissMenu(true);
  if (event.key === "ArrowRight" && menus.cascades(/** @type {HTMLElement} */ (event.target))) {
    event.preventDefault();
    event.stopPropagation();
    return menus.openCascade(/** @type {HTMLElement} */ (event.target), true);
  }
  if (menus.walkMenu(event, sub)) menus.hideSubmenu(levelOf(sub) + 1);
}));
withRest(() => {
  menus.startFlyouts();
});
addEventListener("scroll", (event) => {
  const element = event.target instanceof Element ? event.target : /** @type {Element} */ (document.scrollingElement);
  if (scrolled.has(element)) return;
  for (const other of scrolled) if (!other.isConnected) scrolled.delete(other);
  scrolled.add(element);
}, { capture: true, passive: true });
withRest(() => {
  document.addEventListener("focusin", (event) => {
    const field = /** @type {HTMLElement} */ (event.target), from = /** @type {Element | null} */ (event.relatedTarget);
    if (!(field instanceof HTMLInputElement || field instanceof HTMLTextAreaElement)) return;
    if (menus.closedOwner && (from?.closest("#menu, .submenu") || performance.now() - menus.closedAt < 400)) menus.fieldOpener.set(field, /** @type {HTMLElement} */ (menus.closedOwner));
    else menus.fieldOpener.delete(field);
  });
});
$("menu").addEventListener("keydown", (event) => {
  const target = /** @type {HTMLElement} */ (event.target), stops = all$(".color-pane :is([data-pane-back], .plane, .hex)", $("menu"));
  const next = event.key === "Tab" && stops.includes(target) ? stops[stops.indexOf(target) + (event.shiftKey ? -1 : 1)] : null;
  if (next) return void (event.preventDefault(), next.focus());
  if (event.key === "Tab" && $("menu").getAttribute("role") === "dialog") {
    const buttons = all$("button:not(:disabled)", $("menu")), at = buttons.indexOf(target);
    event.preventDefault();
    return void buttons[(at + (event.shiftKey ? -1 : 1) + buttons.length) % buttons.length]?.focus();
  }
  const done = target.matches("input") && !composing(event) && (event.key === "Enter" || event.key === "Escape");
  if (event.key === "Tab" || done) {
    if (done) event.preventDefault(), event.stopPropagation();
    return menus.dismissMenu(true);
  }
  if (target.matches("input")) return;
  const glyph = { Backspace: "⌫", Enter: "↵", ArrowUp: "↑", ArrowDown: "↓", ArrowLeft: "←", ArrowRight: "→" }[event.key] ?? event.key.toUpperCase();
  const modified = event.metaKey || event.ctrlKey || event.altKey || event.shiftKey;
  const key = keyText(`${event.metaKey || event.ctrlKey ? "⌘" : ""}${event.altKey ? "⌥" : ""}${event.shiftKey && (event.key.length > 1 || /^[a-z]$/i.test(event.key)) ? "⇧" : ""}${glyph}`);
  const hinted = (modified || !["↵", "↑", "↓", "←", "→"].includes(glyph)) && all$("button:not(:disabled)", $("menu")).find((button) => button.dataset.key === key);
  if (hinted) {
    event.preventDefault();
    event.stopPropagation();
    return hinted.click();
  }
  if (event.key === "Escape" && $("menu").closest("#palette")) {
    event.preventDefault();
    event.stopPropagation();
    menus.hideMenu();
    return command.paletteKeys();
  }
  if (event.key === "ArrowRight" && menus.cascades(/** @type {HTMLElement} */ (event.target))) {
    event.preventDefault();
    event.stopPropagation();
    return menus.openCascade(/** @type {HTMLElement} */ (event.target), true);
  }
  if (menus.walkMenu(event, $("menu"))) menus.hideSubmenu();
});
withRest(() => {
  document.addEventListener("pointerdown", (event) => {
    if (!$("menu").hidden && event.target !== menus.menuScrim && !event.composedPath().includes($("menu")) && !submenus.some((sub) => event.composedPath().includes(sub)) && closest(event, "button") !== menus.ownerNow()) menus.hideMenu();
  });
});
$("window").addEventListener("click", (event) => {
  const found = chartWindow(chart);
  if (found) menus.showMenu(event.currentTarget, `<div class="menu-group">Window</div><div class="menu-row" style="--cols: ${Math.min(4, found.offered.length)}">${found.offered.map((key) => `<button type="button" data-window="${key}" aria-checked="${found.window === key}" aria-label="${windowSaid(key)}">${windowShort(key)}</button>`).join("")}</div><div class="menu-note">Each value is its series over the ${windowSaid(found.window).replace(/^1 /, "")} before it.</div>`, ({ window }) => menus.switchWindow(window));
});
$("interval").addEventListener("click", (event) => {
  const supported = menus.supportedIntervals();
  menus.showMenu(event.currentTarget, INTERVALS.map(([group, items]) => `<div class="menu-group">${group}</div><div class="menu-row">${items.map(([id, text]) => `<button type="button" data-index="${id}" aria-checked="${chart.index === id}" ${supported(id) ? "" : `aria-disabled="true" data-tip="${menus.NO_INTERVAL}"`}>${text}</button>`).join("")}</div>`).join("") + `<div class="menu-note">Minutes and hours load as far back as you look.</div>`, ({ index }) => (supported(index) ? menus.switchInterval(index) : notice(`${INTERVAL_LABEL[index]}: ${menus.NO_INTERVAL.toLowerCase()}`)));
});
function showRange() {
  autoScale();
  if (!resolved || !chartShown()) return;
  const span = spanOf(chart);
  if (span) {
    const wanted = presetSpan(chart.index, chart.range);
    const from = Math.max(span.to + 1 >= wanted.from ? Math.min(span.from, wanted.from) : wanted.from, wanted.to - MOST_CHUNKS + 1);
    if (from !== span.from || !span.live) Object.assign(span, { ...wanted, from }), draw();
  }
  if (pending()) return void (/** @type {NonNullable<typeof displayed>} */ (displayed).doc.range = "");
  applyRange(chart, resolved, true);
  /** @type {NonNullable<typeof displayed>} */ (displayed).doc.range = chart.range;
}
engine.onTimeReset(() => !home && chartShown() && menus.setRange(chart.range));
$("ranges").addEventListener("click", (event) => {
  const button = closest(event, "[data-range]");
  if (button?.getAttribute("aria-disabled") === "true") notice(`${button.textContent} is more than fits at ${INTERVAL_LABEL[chart.index]}`);
  else if (button) menus.setRange(/** @type {string} */ (button.dataset.range));
});
$("title").addEventListener("input", (event) => (fitTitle(), commit((doc) => (doc.title = event.target.value), { coalesce: "title" })));
document.fonts.addEventListener("loadingdone", fitTitle);
withRest(() => {
  $("title").addEventListener("blur", () => (leftWith($("title"), menus.leaveTitle) || menus.leaveTitle(), titleReads()));
});
titleReads();
phone.addEventListener("change", () => document.activeElement !== $("title") && titleReads());
withRest(() => {
  document.addEventListener("focusout", (event) => event.target instanceof HTMLInputElement && event.target.matches(menus.FIELDS) && event.target.id !== "title" && leftWith(event.target, () => {}));
  document.addEventListener("focusin", (event) => {
    if (!(event.target instanceof HTMLInputElement) || !event.target.matches(menus.FIELDS) || leftOpen.has(event.target)) return;
    typedFrom.set(event.target, event.target.value);
  });
});
document.addEventListener("keydown", (event) => {
  const field = event.target;
  if (!(field instanceof HTMLInputElement) || !field.matches(menus.FIELDS) || composing(event) || (event.key !== "Enter" && event.key !== "Escape")) return;
  event.preventDefault();
  if (event.key === "Escape" && typedFrom.has(field) && field.value !== typedFrom.get(field)) {
    field.value = typedFrom.get(field);
    field.dispatchEvent(new Event("input", { bubbles: true }));
  }
  const now = /** @type {HTMLElement | null} */ (document.activeElement);
  if (now && now !== field && menus.fieldOpener.has(field)) menus.fieldOpener.set(now, /** @type {HTMLElement} */ (menus.fieldOpener.get(field)));
  if (now) menus.letGo(now);
}, true);
for (const id of ["undo", "home-undo"]) $(id).addEventListener("click", () => ($(id).getAttribute("aria-disabled") === "true" ? say("Nothing to undo") : undoLast()));
for (const id of ["redo", "home-redo"]) $(id).addEventListener("click", () => ($(id).getAttribute("aria-disabled") === "true" ? say("Nothing to redo") : redoLast()));
for (const id of ["add", "mobile-add"]) $(id).addEventListener("click", () => (home ? grid.newChart() : openPalette({ add: true })));

const SHEET_SHARE = TOUCH && typeof navigator.share === "function", SHARE = SHEET_SHARE ? "Share a link" : "Copy a share link";
const SHARES_IMAGES = SHEET_SHARE && Boolean(navigator.canShare?.({ files: [new File([""], "chart.png", { type: "image/png" })] }));
$("share").setAttribute("aria-label", ($("share").dataset.tip = SHARE));
let shareAsk = 0;
$("share").addEventListener("click", () => {
  const button = $("share");
  if (menus.ownerNow() === button && !$("menu").hidden) return menus.hideMenu();
  const doc = chart, ask = ++shareAsk, shown = menus.menusShown, { copy, short } = menus.linksOf(), copying = SHEET_SHARE ? Promise.resolve(false) : menus.copyLink(copy);
  const movedOn = () => ask === shareAsk && ++shareAsk;
  for (const type of ["pointerdown", "keydown"]) addEventListener(type, movedOn, { capture: true, once: true });
  void (async () => {
    const [copied, url, qr] = await Promise.all([copying, copy.catch(() => ""), short.then(menus.qrCode)]);
    for (const type of ["pointerdown", "keydown"]) removeEventListener(type, movedOn, true);
    if (!url) return notice("Couldn't make the link");
    if (ask !== shareAsk || shown !== menus.menusShown || chart !== doc || document.body.hasAttribute("data-home") || !$("menu").hidden) return void (copied && toast("Copied a link: it opens this chart for anyone"));
    const done = SHEET_SHARE ? "" : copied ? "Link copied" : "Couldn't copy the link";
    const code = qr ? (SHEET_SHARE && phone.matches ? "Scan to open it on another phone" : "Scan to open it on a phone") : "Too long for a QR code";
    say([done, code].filter(Boolean).join(". "));
    menus.showMenu(button, `<div class="share-pop"><div class="share-text"><p class="share-title">${escapeHtml(titleOf(doc))}</p>${done ? `<p class="share-done${copied ? "" : " failed"}">${copied ? ICONS.check : ""}${done}</p>` : ""}</div>${qr ?? ""}<p class="share-said">${code}</p></div><div class="menu-rule"></div>${SHEET_SHARE ? `<button type="button" data-share="sheet">${ICONS.share}Share…</button>` : ""}${copied ? "" : `<button type="button" data-share="copy">${ICONS.copy}Copy link</button>`}<button type="button" data-share="open">${ICONS.link}Open in a new tab</button>`, async ({ share }) => {
      if (share === "open") window.open(url, "_blank", "noopener");
      else if (share === "copy") (await menus.copyLink(Promise.resolve(url))) ? toast("Copied a link: it opens this chart for anyone") : notice("Couldn't copy the link");
      else if (share === "sheet") {
        try {
          await navigator.share({ title: titleOf(doc), url });
        } catch (error) {
          if (/** @type {Error} */ (error).name !== "AbortError") notice("Couldn't share the link");
        }
      }
    });
    $("menu").scrollTop = 0;
  })();
});


let importInto = /** @type {string | undefined} */ (undefined);
/** @param {string} [into] */
const importCharts = (into = undefined) => ((importInto = into), $("import-file").click());
$("import-file").addEventListener("change", async (event) => {
  const file = event.target.files?.[0], target = importInto && folderById(importInto) ? importInto : undefined;
  event.target.value = "";
  importInto = undefined;
  if (!file) return;
  // (Charts, even all of them, are a few megabytes at most: a file far past that isn't, and would hang the page read.)
  if (file.size > 32e6) return notice(`That file is ${Math.round(file.size / 1e6)} MB: too large to be charts`);
  await loadPresets().catch(() => {});
  try {
    const parsed = JSON.parse(await file.text());
    if (Array.isArray(parsed?.charts)) {
      const folders = new Map(foldersOf(parsed.folders, []).map((folder) => [folder.id, { ...folder, id: uid() }]));
      /** @type {Map<string, Chart>} */
      const byId = new Map();
      let already = 0;
      const docs = parsed.charts.flatMap((item) => {
        try {
          const doc = readChart(item);
          if (library.charts.some((other) => sameChart(other, doc))) return (forgetView(doc.id), already++, []);
          const folder = folders.get(item.folder);
          if (folder) doc.folder = folder.id;
          if (typeof item.id === "string" && !byId.has(item.id)) byId.set(item.id, doc);
          return [doc];
        } catch {
          return [];
        }
      });
      let editsHere = 0;
      const edits = /** @type {Chart[]} */ ((Array.isArray(parsed.presets) ? parsed.presets : []).flatMap((/** @type {any} */ item) => {
        try {
          if (typeof item?.preset !== "string" || item.preset.length > 2000 || chart?.preset === item.preset) return [];
          if (library.presets.some((other) => other.preset === item.preset && presetEdited(other))) return [];
          const doc = readChart(item), carried = doc.id;
          Object.assign(doc, { id: presetIdOf(item.preset), preset: item.preset, updated: Date.now() });
          if (presets && presetIndex(doc) < 0 && [...library.charts, ...library.presets].some((other) => sameChart(other, doc))) return (forgetView(carried), editsHere++, []);
          moveView(carried, doc.id);
          if (typeof item.base === "string" && /^[a-z0-9]{1,24}$/.test(item.base)) doc.base = item.base;
          return [doc];
        } catch {
          return [];
        }
      }));
      const made = new Map([...folders.values()].map((folder) => [folder.id, folder]));
      for (const folder of made.values()) {
        const parent = folder.parent && folders.get(folder.parent)?.id;
        parent ? (folder.parent = parent) : delete folder.parent;
      }
      const held = (/** @type {(string | undefined)[]} */ ids) => new Set(ids.flatMap((id) => {
        const chain = [];
        for (let at = id, steps = 0; at && steps <= made.size; at = made.get(at)?.parent, steps++) chain.push(at);
        return chain;
      }));
      const holds = held(docs.map((doc) => doc.folder)), heldThere = held(parsed.charts.map((item) => folders.get(item?.folder)?.id));
      const namesOf = (/** @type {Folder} */ folder, /** @type {(id: string) => Folder | undefined} */ up, /** @type {number} */ most) => {
        const names = [];
        for (let at = /** @type {Folder | undefined} */ (folder), steps = 0; at && steps <= most; at = at.parent ? up(at.parent) : undefined, steps++) names.unshift(at.name);
        return JSON.stringify(names);
      };
      const fromTarget = (/** @type {Folder} */ folder) => {
        const names = [];
        for (let at = /** @type {Folder | undefined} */ (folder), steps = 0; at && steps <= library.folders.length; at = at.parent ? folderById(at.parent) : undefined, steps++) {
          if (at.id === target) return JSON.stringify(names);
          names.unshift(at.name);
        }
        return null;
      };
      const pathThere = (/** @type {Folder} */ folder) => namesOf(folder, (id) => made.get(id), made.size), here = new Map(), into = new Map();
      for (const folder of library.folders) ((key) => key !== null && here.set(key, here.has(key) ? null : folder.id))(target ? fromTarget(folder) : namesOf(folder, folderById, library.folders.length));
      for (const [from, folder] of folders) ((id) => id && into.set(folder.id, id))(folderById(from) && (!target || grid.isIn(from, target)) ? from : here.get(pathThere(folder)));
      for (const doc of docs) if (doc.folder && into.has(doc.folder)) doc.folder = into.get(doc.folder);
      for (const [from, folder] of folders) if (into.has(folder.id) || (!holds.has(folder.id) && heldThere.has(folder.id))) folders.delete(from);
      const kept = new Set([...folders.values()].map((folder) => folder.id));
      for (const folder of folders.values()) {
        let parent = folder.parent;
        while (parent && !kept.has(parent) && !into.has(parent)) parent = made.get(parent)?.parent;
        parent ? (folder.parent = into.get(parent) ?? parent) : target ? (folder.parent = target) : delete folder.parent;
      }
      if (target) for (const doc of docs) doc.folder ??= target;
      if (!docs.length && !edits.length && !folders.size && editsHere) return notice("You already have all of it");
      if (!docs.length && !edits.length && !folders.size && already) return toast(`${already === 1 ? "You already have that chart" : `You already have all ${grouped.format(already)} charts`}`);
      if (!docs.length && !edits.length && !folders.size && parsed.charts.length) return notice(parsed.charts.length === 1 ? "Its chart couldn't be read" : `None of its ${grouped.format(parsed.charts.length)} charts could be read`);
      if (!docs.length && !edits.length && !folders.size) throw new Error("No charts in it");
      library.presets = [...library.presets.filter((other) => !edits.some((doc) => doc.preset === other.preset)), ...edits];
      const before = grid.treeOf().map(({ item }) => grid.sideKey(item)), among = library.charts.length > 0;
      const order = orderOf(parsed.order).flatMap((key) => {
        const folder = key[0] === "f" ? folders.get(key.slice(2)) : undefined, doc = key[0] === "c" ? byId.get(key.slice(2)) : undefined;
        return folder ? [`f:${folder.id}`] : doc ? [`c:${doc.id}`] : [];
      });
      library.charts.unshift(...docs);
      library.folders.push(...folders.values());
      library.order = [...order, ...before];
      const keptAsCharts = presets ? adoptPresets(edits, false) : 0, keptIds = edits.filter((doc) => library.charts.includes(doc)).map((doc) => doc.id);
      const editIds = edits.filter((doc) => library.presets.includes(doc)).map((doc) => doc.id);
      save();
      goTo({ folder: target ?? "" });
      grid.picked.clear();
      if (among) for (const doc of docs) grid.picked.add(doc.id);
      drawThumbs();
      renderCharts();
      const skipped = parsed.charts.length - docs.length - already, same = asThen([...docs, ...edits]), arranged = arrangedNow();
      const what = [docs.length ? plural(docs.length, "chart") : "", folders.size ? plural(folders.size, "folder") : "", edits.length ? `${plural(edits.length, "edited preset")}${keptAsCharts ? ` (${keptAsCharts} with no place here, kept as ${keptAsCharts === 1 ? "a chart" : "charts"})` : ""}` : ""].filter(Boolean);
      return toast(`Imported ${new Intl.ListFormat("en-US").format(what)}${already ? `; ${grouped.format(already)} already here` : ""}${skipped === 1 ? "; 1 entry wasn't a chart" : skipped ? `; ${grouped.format(skipped)} entries weren't charts` : ""}`, { label: "Undo", run: () => {
        if (!same()) return keptAsEdited([...docs, ...edits]);
        const unarranged = arrangedNow() === arranged;
        const ids = new Set([...docs.map((doc) => doc.id), ...keptIds]);
        for (const doc of library.charts.filter((doc) => ids.has(doc.id))) (discard(doc), grid.picked.delete(doc.id));
        for (const id of editIds) {
          const doc = library.presets.find((other) => other.id === id);
          if (!doc) continue;
          library.presets = library.presets.filter((other) => other !== doc);
          library.deleted[doc.id] = Math.max(Date.now(), doc.updated);
          if (doc === chart && presetIndex(doc) >= 0) openChart(presetChart(presetIndex(doc)));
        }
        const gone = new Set([...folders.values()].map((folder) => folder.id));
        library.folders = library.folders.filter((folder) => !gone.has(folder.id));
        for (const id of gone) library.deleted[id] = Date.now();
        for (const doc of library.charts) if (doc.folder && gone.has(doc.folder)) delete doc.folder;
        for (const folder of library.folders) if (folder.parent && gone.has(folder.parent)) delete folder.parent;
        if (unarranged) library.order = before;
        if (ids.has(chart.id)) {
          const standIn = library.charts.length ? null : previewCharts[0] ?? newDraft();
          openChart(standIn ?? library.charts[0]);
        }
        save();
        renderCharts();
      } });
    }
    const doc = readChart(parsed);
    const twin = twinOf(doc);
    if (twin) return void (showLinked(twin), toast(`Opened “${titleOf(twin)}”, which you already have`));
    const before = chart;
    if (target) doc.folder = target;
    linkWindow = doc.id;
    adopt(doc);
    const same = asThen([doc]);
    toast(`Imported “${titleOf(doc)}”`, { label: "Undo", run: () => {
      if (!same()) return keptAsEdited([doc]);
      const now = chartById(doc.id);
      if (!now) return;
      discard(now);
      if (chart.id === doc.id) openChart(chartById(before.id) ?? previousChart() ?? library.charts[0] ?? newDraft());
      save();
      renderCharts();
    } });
  } catch {
    notice("That file isn't a Bitview Studio chart");
  }
});
function fromWritten(/** @type {string} */ raw) {
  let text = raw, parsed, failure;
  for (let round = 0; round < 3 && parsed === undefined; round++) {
    try {
      parsed = JSON.parse(text.slice(text.indexOf("{"), text.lastIndexOf("}") + 1));
    } catch (error) {
      failure = error;
      if (/^\s*(?:%[0-9a-f]{2})*%(?:25)*7b/i.test(text) && !text.includes(":")) text = text.replace(/\+/g, "%20");
      try {
        text = decodeURIComponent(text.replace(/%(?![0-9a-f]{2})/gi, "%25"));
      } catch {
        break;
      }
    }
  }
  if (parsed === undefined) throw failure;
  const doc = readChart(parsed);
  lostDrawings = (parsed.drawings?.length ?? 0) - (doc.drawings?.length ?? 0);
  return doc;
}
withRest(() => addEventListener("pagehide", () => menus.openedImages.splice(0).forEach((url) => URL.revokeObjectURL(url))));
/** @param {{ width?: number, height?: number, scale?: number }} [size] @returns {Promise<HTMLCanvasElement>} */
async function chartImage(size = {}, wait = false, grown = false) {
  if (!menus.canvasReadable()) throw new Error(menus.UNREADABLE);
  const PAD = 28, FOOT = 40, title = titleOf(chart);
  const element = $("chart"), wrapper = /** @type {HTMLElement} */ (element.firstElementChild), sized = Boolean(size.width && size.height);
  const owner = menus.shownAs();
  const width = sized ? /** @type {number} */ (size.width) - PAD * 2 : element.clientWidth;
  const titled = pictureTitle(title, width), HEAD = titled.length > 1 ? 86 : 60;
  const height = sized ? /** @type {number} */ (size.height) - HEAD - FOOT : element.clientHeight;
  if (!width || !height) throw new Error("The chart has no room: the window is of no size");
  /** @template T @param {(layout: ReturnType<typeof menus.apartLayout>) => T} read @returns {T} */
  const asPicture = (read) => {
    const style = element.getAttribute("style"), wrapped = [wrapper.style.width, wrapper.style.height], logical = shownSpan(), heightWas = chartHeight, shown = presetShown, screen = engine.time.width;
    let restoreLegends = () => {};
    exporting = true;
    const crossIndex = engine.crosshair.index, hovered = hoverTime;
    engine.crosshair.index = NaN;
    hoverTime = null;
    try {
      if (sized) Object.assign(element.style, { right: "auto", bottom: "auto", width: `${width}px`, height: `${height}px` });
      layOutLegends();
      const presented = presentLegends();
      restoreLegends = presented;
      sizeChart(width, height);
      const view = () => {
        if (!sized || !resolved || !chartShown()) return;
        fitSpacing(chart, resolved.times.length);
        if (presetShown) applyRange(chart, resolved);
        else if (logical) showSpan(atLatest(logical, screen) ?? logical, EXACT_RANGE);
      };
      view();
      engine.render(true);
      updateReadout();
      const together = menus.setApart();
      restoreLegends = () => (together(), presented());
      const layout = menus.apartLayout(width);
      const plot = sized ? height - layout.height : height;
      if (sized) element.style.height = `${plot}px`;
      sizeChart(width, plot);
      chartHeight = plot;
      sizePanes();
      marginsSet = "";
      fitMargins();
      sizeChart(width, plot);
      view();
      return read(layout);
    } finally {
      engine.crosshair.index = crossIndex;
      restoreLegends();
      if (style === null) element.removeAttribute("style");
      else element.setAttribute("style", style);
      chartHeight = heightWas;
      sizeChart(element.clientWidth, element.clientHeight);
      [wrapper.style.width, wrapper.style.height] = wrapped;
      sizePanes(undefined, false, false);
      if (sized && resolved && chartShown()) fitSpacing(chart, resolved.times.length);
      if (sized) setReserve(), engine.invalidate(true);
      if (sized && logical) showSpan(logical, EXACT_RANGE);
      if (sized && shown && resolved && chartShown()) {
        applyRange(chart, resolved);
        if (!presetShown) setPresetShown(true);
      }
      marginsSet = "";
      fitMargins();
      engine.render(true);
      exporting = false;
      hoverTime = hovered;
      updateReadout();
      if (profileLit) drawn.get(profileLit.layer)?.redraw();
      lastBox = chartWidth();
      if (crossSet) holdCross();
    }
  };
  /** @type {{ shot: HTMLCanvasElement, scale: number, pieces: ReturnType<typeof menus.measureLegends>, layout: ReturnType<typeof menus.apartLayout>, dates: ReturnType<typeof menus.viewDates>, axisHeight: number, leftWidth: number }} */
  let taken;
  /** @type {{ wants: NonNullable<ReturnType<typeof heatmaps.heatWants.get>>, keys: Set<string> }} */
  const extra = { wants: [], keys: new Set() };
  heatmaps.heatExtras.add(extra);
  try {
    const grow = !sized ? 1 : asPicture((layout) => {
      const holds = (/** @type {number} */ k) => height * k + (k - 1) * (HEAD + FOOT) - (k === 1 ? layout : menus.apartLayout(width * k + (k - 1) * PAD * 2)).height >= 160;
      let k = 1;
      while (!grown && !holds(k) && k * 1.25 <= 2) k *= 1.25;
      if (!holds(k)) throw new Error(menus.TOO_SMALL);
      return k;
    });
    if (grow > 1) return chartImage({ width: /** @type {number} */ (size.width) * grow, height: /** @type {number} */ (size.height) * grow, scale: (size.scale ?? devicePixelRatio) / grow }, wait, true);
    if (sized && wait) for (let round = 0; heatmaps.shownHeats().length; round++) {
      const asked = asPicture(() => {
        chartPicture(engine);
        Object.assign(extra, { wants: [...heatmaps.heatWants.values()].flat(), keys: new Set(heatmaps.shownHeats().flatMap((look) => [...(look.keep ?? [])])) });
        return [...extra.keys].filter((key) => !heatmaps.heatDays.get(key)?.rows && !heatmaps.heatDays.get(key)?.failed);
      });
      if (!asked.length) break;
      if (round === 3) throw new Error("The heatmap's days are still loading");
      heatmaps.heatPump();
      await heatmaps.heatCome(asked, 20_000, () => menus.shownAs() === owner);
      if (menus.shownAs() !== owner) throw new Error("The chart changed meanwhile");
    }
    taken = asPicture((layout) => {
      capturing = true;
      repaintDrawings();
      let shot;
      try {
        shot = chartPicture(engine);
      } finally {
        capturing = false;
        repaintDrawings();
      }
      const scale = shot.width / width;
      const pieces = menus.measureLegends(scale);
      return { shot, scale, pieces, layout, dates: menus.viewDates(), axisHeight: engine.timeHeight, leftWidth: engine.left };
    });
  } finally {
    heatmaps.heatExtras.delete(extra);
  }
  const { shot, scale, pieces, layout, dates, axisHeight, leftWidth } = taken;
  const cadence = CADENCE[shownIndex()], [sans, cardColor, inkColor, softColor, mutedColor, lineColor] = ["--sans", "--card", "--ink", "--soft", "--muted", "--line"].map(css);
  if (layout.items.length) {
    const row = menus.APART.row * scale, [markWidth, markHeight] = [menus.APART.mark * scale, menus.APART.markHeight * scale];
    const { name: nameFont, value: valueFont } = menus.apartFonts(scale, sans);
    for (const item of layout.items) {
      const x = item.x * scale, y = shot.height + item.y * scale, nameAt = x + (item.mark ? markWidth + menus.APART.markGap * scale : 0);
      if (item.mark) pieces.push({ x, y: y + (row - markHeight) / 2, width: markWidth, height: markHeight, svg: item.mark, faded: item.faded });
      item.lines.forEach((text, i) => pieces.push({ x: nameAt, y: y + i * row, width: item.nameWidth * scale, height: row, faded: item.faded, text, font: nameFont, color: softColor, align: "left", cut: true }));
      if (item.value) pieces.push({ x: (item.x + layout.column - menus.APART.columnGap - item.valueWidth) * scale, y, width: item.valueWidth * scale, height: row, faded: item.faded, text: item.value, font: valueFont, color: inkColor, align: "left", cut: true });
    }
  }
  await menus.drawMarks(pieces);
  const px = (n) => Math.round(n * scale);
  const font = (weight, size) => `${weight} ${px(size)}px ${sans}`;
  const pad = px(PAD), head = px(HEAD), foot = px(FOOT), under = layout.items.length ? px(layout.height) : 0;
  const canvas = document.createElement("canvas");
  canvas.width = shot.width + pad * 2;
  canvas.height = shot.height + head + under + foot;
  const context = /** @type {CanvasRenderingContext2D} */ (canvas.getContext("2d"));
  context.fillStyle = cardColor;
  context.fillRect(0, 0, canvas.width, canvas.height);
  context.fillStyle = inkColor;
  context.font = font(590, 20);
  titled.forEach((line, i) => context.fillText(fitted(context, line, canvas.width - pad * 2), pad, px(38 + 26 * i)));
  context.drawImage(shot, pad, head);
  const axis = Math.round(axisHeight * scale), left = pad + Math.round(leftWidth * scale), fade = context.createLinearGradient(left, 0, left + px(14), 0);
  fade.addColorStop(0, cardColor);
  fade.addColorStop(1, alpha(cardColor, 0));
  context.fillStyle = fade;
  context.fillRect(left, head + shot.height - axis, px(14), axis);
  context.fillStyle = lineColor;
  context.fillRect(pad, head + shot.height - axis - px(1), shot.width, Math.max(1, px(1)));
  for (const piece of pieces) {
    const x = pad + piece.x, y = head + piece.y;
    context.globalAlpha = piece.faded ? 0.4 : 1;
    context.save();
    if (piece.clip !== undefined) (context.beginPath(), context.rect(0, 0, pad + piece.clip, canvas.height), context.clip());
    if (piece.image) {
      context.drawImage(piece.image, x, y, piece.width, piece.height);
      context.restore();
      continue;
    }
    context.font = /** @type {string} */ (piece.font);
    context.fillStyle = /** @type {string} */ (piece.color);
    context.textAlign = piece.align ?? "left";
    context.textBaseline = "middle";
    const text = piece.cut ? fitted(context, /** @type {string} */ (piece.text), piece.width) : /** @type {string} */ (piece.text), at = piece.align === "right" ? x + piece.width : x;
    context.strokeStyle = cardColor;
    context.lineWidth = 2 * scale;
    context.lineJoin = "round";
    context.strokeText(text, at, y + piece.height / 2);
    context.fillText(text, at, y + piece.height / 2);
    context.restore();
  }
  context.globalAlpha = 1;
  context.textAlign = "left";
  context.textBaseline = "alphabetic";
  const baseline = head + shot.height + under + px(28);
  let at = pad;
  ((mark) => {
    const path = mark?.querySelector("path"), box = mark?.viewBox.baseVal;
    if (!path || !box?.height) return;
    const tall = px(13), k = tall / box.height;
    context.save();
    context.translate(at, baseline - tall + px(1.5));
    context.scale(k, k);
    context.translate(-box.x, -box.y);
    context.fillStyle = /** @type {string} */ (path.getAttribute("fill"));
    context.fill(new Path2D(/** @type {string} */ (path.getAttribute("d"))), "evenodd");
    context.restore();
    at += box.width * k + px(6);
  })(/** @type {SVGSVGElement | null} */ (document.querySelector(".brand .logo svg")));
  context.font = font(590, 12);
  context.fillStyle = inkColor;
  context.fillText("Bitview", at, baseline);
  at += context.measureText("Bitview ").width;
  context.font = font(510, 12);
  context.fillStyle = softColor;
  context.fillText("Studio", at, baseline);
  at += context.measureText("Studio").width;
  const local = !location.hostname || location.hostname === "localhost" || /^(?:127\.|10\.|192\.168\.|172\.(?:1[6-9]|2\d|3[01])\.|\[::1\]|\[f[cd])/i.test(location.hostname) || location.hostname.endsWith(".local");
  const where = ` · ${local ? SERVER_HOST : location.host}`;
  const last = dates.values || dates.span, full = [cadence, dates.span, dates.values].filter(Boolean).join(" · "), short = [cadence, last].filter(Boolean).join(" · ");
  const room = (/** @type {boolean} */ placed) => canvas.width - pad - at - (placed ? context.measureText(where).width : 0) - px(24);
  const fits = (/** @type {string} */ text, /** @type {boolean} */ placed) => context.measureText(text).width < room(placed);
  const [said, placed] = fits(full, true) ? [full, true] : fits(short, true) ? [short, true] : fits(short, false) ? [short, false] : [fitted(context, last, room(false)), false];
  context.fillStyle = mutedColor;
  if (placed) context.fillText(where, at, baseline);
  context.textAlign = "right";
  context.fillText(said, canvas.width - pad, baseline);
  if (!sized || !size.scale) return canvas;
  const outWidth = Math.round(/** @type {number} */ (size.width) * size.scale), outHeight = Math.round(/** @type {number} */ (size.height) * size.scale);
  if (canvas.width === outWidth && canvas.height === outHeight) return canvas;
  const out = document.createElement("canvas");
  out.width = outWidth;
  out.height = outHeight;
  const resampled = /** @type {CanvasRenderingContext2D} */ (out.getContext("2d"));
  resampled.imageSmoothingQuality = "high";
  resampled.drawImage(canvas, 0, 0, out.width, out.height);
  return out;
}
function pictureTitle(/** @type {string} */ text, /** @type {number} */ width) {
  measure.font = `590 20px ${(figuresFont ||= css("--sans"))}`;
  const words = text.split(" "), room = width * 0.97;
  if (words.length < 2 || measure.measureText(text).width <= room) return [text];
  let k = 1;
  while (k < words.length - 1 && measure.measureText(words.slice(0, k + 1).join(" ")).width <= room) k++;
  return [words.slice(0, k).join(" "), words.slice(k).join(" ")];
}
/** @returns {() => void} */
function presentLegends() {
  const strips = legends().map((legend) => /** @type {[HTMLElement, string]} */ ([legend, legend.style.cssText]));
  const buttons = all$("#chart .legend button").map((button) => /** @type {[HTMLElement, string, string]} */ ([button, button.className, button.style.cssText]));
  const keys = all$("#chart .legend .legend-key").map((key) => /** @type {[HTMLElement, string]} */ ([key, key.style.cssText]));
  const hidden = hiddenPanes(chart, true), lit = profileLit;
  if (lit) (profileLit = null), paintProfileItems();
  const spot = spotlit;
  spotlight(null);
  for (const [legend] of strips) Object.assign(legend.style, { display: hidden.has(legend.dataset.pane ?? "") ? "none" : "flex", flexWrap: "wrap", overflow: "visible", maskImage: "none", rowGap: "2px", height: "auto" });
  for (const [key] of keys) Object.assign(key.style, { flexWrap: "wrap", flexShrink: "1", minWidth: "0" });
  for (const [button] of buttons) {
    button.style.maxWidth = "100%";
    if (button.classList.contains("hidden")) button.style.display = "none";
    button.classList.remove("selected", "lit");
  }
  return () => {
    for (const [button, className, cssText] of buttons) (button.className = className), (button.style.cssText = cssText);
    for (const [legend, cssText] of strips) legend.style.cssText = cssText;
    for (const [key, cssText] of keys) key.style.cssText = cssText;
    if (lit && !profileLit) (profileLit = lit), paintProfileItems();
    spotlight(spot);
  };
}

const tip = $("tip");
let tipTarget = null, tipTimer = 0, tipClosed = 0;
let tipPointed = /** @type {Element | null} */ (null);
let described = /** @type {HTMLElement | null} */ (null);
function tipWanted(/** @type {HTMLElement} */ target) {
  if (railed() && tipPointed && target.matches("#chart-list :is(.folder-item, .list-head-name, [data-open-chart], [data-open-preset])")) return false;
  const name = target.querySelector(".name, .label, .head-name, .tile-name"), sub = target.querySelector(".sub"), [head, ...more] = (target.dataset.tip ?? "").split("\n");
  const whole = (/** @type {Element | null} */ words) => Boolean(words) && /** @type {Element} */ (words).scrollWidth <= /** @type {Element} */ (words).clientWidth;
  const subText = sub ? [...sub.childNodes].filter((node) => !(node instanceof Element && node.classList.contains("sub-letter"))).map((node) => node.textContent).join("").trim() : "";
  const repeats = whole(name) && name?.textContent?.trim() === head && (!more.length || (whole(sub) && subText === more.join("\n")));
  const keys = Boolean(target.dataset.kbd && answers(target.dataset.kbd));
  return !repeats || (target.closest(".sidebar") ? railed() || Boolean(tipPointed && target.contains(tipPointed) && tipPointed.closest("[data-icon], .chart-mark, .head-mark")) : keys);
}
function showTip(target) {
  if (!target.isConnected || (menus?.ownerNow() === target && !$("menu").hidden)) return;
  if (!tipWanted(target)) return;
  const keys = target.dataset.kbd && answers(target.dataset.kbd) ? keycaps(target.dataset.kbd) : "";
  // Inside a dialog, the tip must live in the dialog (in the top layer) to be seen.
  const host = target.closest("dialog") ?? document.body;
  if (tip.parentElement !== host) host.append(tip);
  const [head, ...more] = target.dataset.tip.split("\n");
  tip.innerHTML = `<span>${escapeHtml(head)}${more.length ? `<span class="tip-more">${more.map(escapeHtml).join("<br>")}</span>` : ""}</span>${keys}`;
  drawKeys(tip);
  cutFade(tip);
  tip.hidden = false;
  if (target.getAttribute("aria-label") !== target.dataset.tip) {
    described?.removeAttribute("aria-describedby");
    (described = target).setAttribute("aria-describedby", "tip");
  }
  const rect = target.getBoundingClientRect();
  const box = tip.getBoundingClientRect();
  let left = rect.left + rect.width / 2 - box.width / 2, top = rect.bottom + 8;
  const side = target.closest("#belt") ? pen.beltSide() : "";
  if (side === "above") top = rect.top - box.height - 8;
  else if (side === "left" || side === "right") {
    left = side === "right" ? rect.right + 8 : rect.left - box.width - 8;
    top = rect.top + rect.height / 2 - box.height / 2;
  } else if (target.closest(".sidebar")) {
    left = $("sidebar").getBoundingClientRect().right + 6;
    top = rect.top + rect.height / 2 - box.height / 2;
  } else if (top + box.height > innerHeight - 8) top = rect.top - box.height - 8;
  const origin = fixedOrigin(tip);
  placeFixed(tip, origin, clamp(left, 8, innerWidth - box.width - 8), clamp(top, 8, innerHeight - box.height - 8));
}
function hideTip() {
  clearTimeout(tipTimer);
  tipTimer = 0;
  if (!tip.hidden) (fadeOut(tip, [{ opacity: 1 }, { opacity: 0 }]), (tipClosed = performance.now()), (tip.hidden = true));
  tipTarget = null;
  described?.removeAttribute("aria-describedby");
  described = null;
}
let tabbing = false;
addEventListener("keydown", (event) => (tabbing = event.key === "Tab" || event.key.startsWith("Arrow")), true);
addEventListener("pointerdown", () => (tabbing = false), true);
document.addEventListener("focusin", (event) => {
  const target = /** @type {HTMLElement} */ (event.target);
  if (!(target instanceof HTMLElement) || !target.dataset.tip || target === tipTarget || !tabbing || !target.matches(":focus-visible") || typing(target)) return;
  hideTip();
  tipPointed = null;
  tipTarget = target;
  tip.classList.add("warm");
  showTip(target);
});
document.addEventListener("focusout", (event) => event.target === tipTarget && hideTip());
document.addEventListener("pointerover", (event) => {
  if (event.pointerType === "touch" || event.buttons) return;
  const target = closest(event, "[data-tip]");
  tipPointed = /** @type {Element} */ (event.target);
  if (target === tipTarget && (!tip.hidden || tipTimer)) return void (target && !tipWanted(target) && hideTip());
  hideTip();
  if (!target?.dataset.tip) return;
  tipTarget = target;
  const warm = performance.now() - tipClosed < 500;
  tip.classList.toggle("warm", warm);
  tipTimer = setTimeout(() => ((tipTimer = 0), showTip(target)), warm ? 0 : 550);
});
document.addEventListener("pointerout", (event) => !event.relatedTarget && hideTip());
for (const type of ["pointerdown", "keydown", "wheel", "scroll"]) document.addEventListener(type, hideTip, { capture: true, passive: true });

function notice(text, action) {
  toast(text, action, false);
}
const steppedPast = () => notice(`Things have changed since then. Press ${keyText("⌘Z")} to step back`);
function undoToast(text, doc = chart) {
  const step = ((last) => (last?.at === stepCount ? last : null))(stepsOf(doc)?.past.at(-1));
  toast(text, { label: "Undo", of: doc, run: () => (step && (doc === chart || chartById(doc.id) === doc) && stepsOf(doc)?.past.at(-1) === step ? travelIn(doc, true) : steppedPast()) });
}
let toastTimer = 0;
function syncUndo() {
  const back = canTravel(true), forth = canTravel(false);
  for (const id of ["undo", "home-undo"]) $(id).setAttribute("aria-disabled", String(!back));
  for (const id of ["redo", "home-redo"]) $(id).setAttribute("aria-disabled", String(!forth));
  const edited = isPreview(chart) && touched.has(chart), keep = $("keep");
  keep.classList.toggle("edited", edited);
  keep.dataset.tip = edited ? "Save this chart and your edits to your Library" : "Save this chart to your Library";
  keep.setAttribute("aria-label", edited ? "Save, edited" : "Save");
}
const placingPath = () => tool === "path" && Boolean(draft && placing);
function canTravel(/** @type {boolean} */ back) {
  if (placingPath() && draft) return back ? draft.points.length > 1 : pathTaken.length > 0;
  return historyFor(back) !== undefined || (!back && !home && Boolean(pathResume && tool === "path"));
}
/** @typedef {{ text: string, run: () => (void | false | (() => void)), undone?: string, at: number, folder?: string, open?: Chart, left?: string }} LibraryStep */
const libraryEdits = { past: /** @type {LibraryStep[]} */ ([]), future: /** @type {LibraryStep[]} */ ([]) };
let redoing = false, undoing = false;
/** @param {Chart[]} docs @returns {() => boolean} */
const asThen = (docs) => ((contents) => () => docs.every((doc, i) => ((now) => !now || contentOf({ ...now, range: "" }) === contents[i])(chartById(doc.id))))(docs.map((doc) => contentOf({ ...doc, range: "" })));
/** @type {Map<string, string>} */
const folderAgain = new Map();
const folderNow = (/** @type {string} */ id) => {
  for (let hops = 0; folderAgain.has(id) && hops < 50; hops++) id = /** @type {string} */ (folderAgain.get(id));
  return id;
};
/** @type {Map<string, string>} */
const chartAgain = new Map();
const chartNow = (/** @type {string} */ id) => {
  for (let hops = 0; chartAgain.has(id) && hops < 50; hops++) id = /** @type {string} */ (chartAgain.get(id));
  return id;
};
const keysNow = (/** @type {string[]} */ keys) => keys.map((key) => (key[0] === "f" ? `f:${folderNow(key.slice(2))}` : `c:${chartNow(key.slice(2))}`));
/** @param {Chart[]} docs */
const keptAsEdited = (docs) => notice(docs.length === 1 ? `“${titleOf(docs[0])}” has been edited since then, so it was left unchanged` : "They've been edited since then, so they were left unchanged");
/** @returns {LibraryStep | null} */
function libraryStep(/** @type {string} */ text, /** @type {LibraryStep["run"]} */ run, /** @type {string | undefined} */ undone = undefined) {
  if (undoing) return null;
  const step = { text, run, undone, at: ++stepCount };
  libraryEdits.past.push(step);
  if (libraryEdits.past.length > 50) libraryEdits.past.shift();
  if (!redoing) dropRedos();
  syncUndo();
  return step;
}
function undoLibrary() {
  const step = libraryEdits.past.pop();
  if (!step) return;
  const saved = saves, said = toasts, was = chart;
  undoing = true;
  let again;
  try {
    again = step.run();
  } finally {
    undoing = false;
  }
  if (saves === saved) return void (syncUndo(), toasts === said && notice(`Nothing to undo: things have changed since then (${step.text})`));
  if (typeof again !== "function") return void (syncUndo(), toast(step.undone ?? "Undone"));
  const back = { text: step.text, run: again, undone: step.undone, at: ++stepCount, open: was, left: chart.id };
  libraryEdits.future.push(back);
  toast(step.undone ?? "Undone", { label: "Redo", run: () => (libraryEdits.future.at(-1) === back ? redoLibrary() : notice("Nothing to redo: things have changed since then")) });
}
function redoLibrary() {
  const step = libraryEdits.future.pop();
  if (!step) return notice("Nothing to redo");
  hideToast();
  const saved = saves, said = toasts, from = chart;
  redoing = true;
  try {
    step.run();
  } finally {
    redoing = false;
  }
  if (step.open && saves !== saved && !home && from.id === step.left && chart.id !== step.open.id) {
    const doc = step.open, preset = doc.preset !== undefined ? presetIndex(doc) : -1;
    if (!chartById(doc.id) && drafts.has(doc)) addPreview(doc);
    const now = chartById(doc.id) ?? (preset >= 0 ? presetChart(preset) : undefined);
    if (now?.id === doc.id) {
      if (chart !== from && history.state?.chart === chart.id) history.replaceState({ ...history.state, chart: now.id }, "");
      openChart(now);
    }
  }
  syncUndo();
  if (saves === saved && toasts === said) notice(`Nothing to redo: things have changed since then (${step.text})`);
}
let toastOf = /** @type {Chart | null} */ (null);
/** @param {{ label: string, run: () => (void | (() => void)), undone?: string, of?: Chart, folder?: string, step?: LibraryStep | null }} [action] */
function toast(text, action, done = true) {
  toasts++;
  unsay();
  closedToast = null;
  const element = $("toast");
  const host = topDialog() ?? document.body;
  if (element.parentElement !== host) (cutFade(element), host.append(element));
  const said = /** @type {HTMLElement | null} */ (host.querySelector(":scope > [data-said]"));
  if (said) {
    said.textContent = "";
    cancelAnimationFrame(sayingFrame);
    sayingFrame = requestAnimationFrame(() => ((saidOn = said), (said.textContent = toastSaid = action ? `${text}. ${action.label}${action.label === "Undo" ? `, ${keyText("⌘Z")}` : action.label === "Redo" ? `, ${keyText("⇧⌘Z")}` : ""}` : text)));
  }
  const undoes = action?.label === "Undo";
  const step = undoes && action && !action.of ? ("step" in action ? action.step ?? null : libraryStep(text, action.run, action.undone)) : null;
  if (step && action?.folder) step.folder = action.folder;
  toastOf = action?.of ?? (chartChanged ? chart : null);
  const redoes = action?.label === "Redo";
  const replacing = element.classList.contains("visible") && !reduced();
  if (replacing) fadeOut(element, [{ opacity: 1 }, { opacity: 0 }]);
  else cutFade(element);
  element.innerHTML = `${done && !undoes && !redoes ? `<i aria-hidden="true">${ICONS.check}</i>` : ""}<span>${escapeHtml(text)}</span>${action ? `<button type="button" class="btn small" ${undoes ? 'data-kbd="⌘ Z"' : redoes ? 'data-kbd="⇧ ⌘ Z"' : ""}>${escapeHtml(action.label)}</button>` : ""}`;
  element.querySelector("button")?.addEventListener("click", () => {
    element.classList.remove("visible");
    if (step) return libraryEdits.past.at(-1) === step ? undoLibrary() : steppedPast();
    action?.run();
  });
  syncUndo();
  placeToast(host);
  element.classList.add("visible");
  if (replacing) element.animate({ opacity: [0, 1] }, { duration: MOTION.micro, easing: MOTION.easeLayout });
  toastFor = Math.max(action ? 6000 : done ? 2600 : 4500, 1500 + text.length * 55);
  hideToastLater();
}
/** Placed again as the screen changes while it's up (a turn, the keyboard): measured once, it stayed where the old layout had it, off the new one. */
function placeToast(/** @type {Element} */ host = /** @type {HTMLElement} */ ($("toast").parentElement)) {
  const element = $("toast");
  const layersBar = $("layers-bar"), sheetBar = document.body.dataset.sheet === "panel" && !layersBar.hidden ? layersBar.getBoundingClientRect() : null;
  const frame = sheetBar ?? (home ? $("home") : $("stage-foot")).getBoundingClientRect();
  const belt = $("belt"), beltTop = !home && !belt.hidden && belt.dataset.dock === "bottom" ? belt.getBoundingClientRect().top : Infinity;
  const tabs = /** @type {HTMLElement} */ (document.querySelector(".mobile-bar")).getBoundingClientRect(), tabsTop = tabs.width > tabs.height ? tabs.top : Infinity;
  const menuSheet = $("menu").classList.contains("as-sheet") && !$("menu").hidden ? $("menu").offsetTop : Infinity;
  // (In the page's own measure: iOS's window is as short as its keyboard leaves, the page isn't. The chart's foot
  // hidden under the keyboard, above the keyboard.)
  const page = document.documentElement.clientHeight, shown = viewport.offsetTop + viewport.height, keys = shown < page - 1;
  const above = Math.min(shown, menuSheet, sheetBar ? sheetBar.top : Math.min(tabsTop, frame.height ? (!home ? Math.min(frame.top, beltTop) : $("home-bar").hidden ? frame.bottom - 8 : $("home-bar").getBoundingClientRect().top) : Infinity));
  element.style.bottom = host === document.body && (frame.height || keys) ? `${page - above + 12}px` : "";
  const center = host === document.body && frame.width ? frame.left + frame.width / 2 : 0;
  element.style.left = center ? `${center}px` : "";
  element.style.maxWidth = center ? `${2 * Math.min(center, innerWidth - center) - 24}px` : "";
}
let toastFor = 0;
const replaceToast = () => {
  if (!$("toast").classList.contains("visible")) return;
  placeToast();
  const moving = [$("panel"), $("menu")].flatMap((sheet) => sheet.getAnimations());
  if (moving.length) Promise.allSettled(moving.map((motion) => motion.finished)).then(() => $("toast").classList.contains("visible") && placeToast());
};
for (const target of [window, viewport]) target.addEventListener("resize", replaceToast);
new MutationObserver(replaceToast).observe(document.body, { attributes: true, attributeFilter: ["data-sheet", "data-sheet-size"] });
new MutationObserver(replaceToast).observe($("menu"), { attributes: true, attributeFilter: ["hidden", "class"] });
function hideToast() {
  closedToast = null;
  clearTimeout(toastTimer);
  $("toast").classList.remove("visible");
  unsay();
}
let toastSaid = "", saidOn = /** @type {HTMLElement | null} */ (null);
let sayingFrame = 0;
const unsay = () => (cancelAnimationFrame(sayingFrame), saidOn?.textContent === toastSaid && (saidOn.textContent = ""));
/** A pointer that hovers (a mouse's; a tap's :hover sticks where it was, on iOS). */
const hoverable = matchMedia("(hover: hover)");
function hideToastLater() {
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    const element = $("toast");
    if ((hoverable.matches && element.matches(":hover")) || element.matches(":focus-within")) return hideToastLater();
    element.classList.remove("visible");
    unsay();
  }, toastFor);
}

const CODE_IN = new RegExp(String.raw`(?:await )?studio\.\w+\((?:[^()<]|\([^()<]*\))*\)|\[(?:\{(?:[^{}<]|\{[^{}<]*\})*\}(?:, )?)+\]|\{(?:[^{}<]|\{[^{}<]*\})*\}|https:\/\/bitview\.space\/studio\/#[a-z]+=|#(?:chart|image|c)=|\b(?:${Object.keys(FUNCTIONS).join("|")})\([^()<]*\)`, "g");
const codeIn = (/** @type {string} */ html) => html.split(/(<pre[\s\S]*?<\/pre>)/).map((part, i) => (i % 2 ? part : part.replace(CODE_IN, (match) => `<code>${match}</code>`))).join("");
function showGuide() {
  const open = !home && chart.layers.length > 0;
  const parts = [...$("about").children], [first, next] = parts.filter((part) => part.tagName === "H2").map((head) => parts.indexOf(head));
  const html = (/** @type {Element[]} */ list) => codeIn(list.map((part) => part.outerHTML).join("")), rest = next === undefined ? [] : parts.slice(next);
  $("guide-body").innerHTML = `<section class="guide-start"><p>A chart's link opens it for anyone as you see it: its layers, their look and the view. The link contains the whole chart (as JSON), so nothing is stored on a server. A preset you haven't edited shares its own address instead, with what you changed about how it's looked at. Your Library and your edited presets stay in this browser.</p>${open ? `<div class="guide-actions"><button type="button" class="btn small" data-guide="link">${ICONS.copy}Copy this chart's link</button><button type="button" class="btn small" data-guide="json">${ICONS.copy}Copy its JSON</button></div>` : ""}</section>${html(parts.slice(first, next))}${rest.length ? `<details class="help guide-more"><summary>For scripts and assistants</summary>${html(rest)}</details>` : ""}`;
  grid.arrive($("guide"));
  grid.showDialog($("guide"));
  $("guide").focus();
}
$("guide").addEventListener("click", async (event) => {
  const kind = /** @type {HTMLElement | null} */ (closest(event, "[data-guide]"))?.dataset.guide;
  if (!kind) return;
  const copied = await menus.copyLink(kind === "link" ? menus.linksOf().copy : Promise.resolve(JSON.stringify(menus.portable(chart), null, 2)));
  copied ? toast(kind === "link" ? "Copied the link" : "Copied the JSON") : notice("Couldn't copy");
});
/**
 * The shortcuts sheet's lists, made the first time it opens (600 elements a phone may never show, none of them
 * styled, laid out or read by the page meanwhile): their keys as this platform writes them and drawn, as the page's.
 */
function keyGroupsNow() {
  const template = /** @type {HTMLTemplateElement | null} */ (document.getElementById("key-groups"));
  if (!template) return;
  const groups = /** @type {DocumentFragment} */ (template.content.cloneNode(true));
  for (const element of groups.querySelectorAll(MAC ? "kbd" : "kbd, dt, h3")) element.textContent = keyText(element.textContent ?? "");
  drawKeys(groups);
  template.replaceWith(groups);
}
function showShortcuts() {
  keyGroupsNow();
  renderSingleKeys();
  grid.arrive($("shortcuts"));
  grid.showDialog($("shortcuts"));
  $("shortcuts").focus();
}
const answers = (/** @type {string} */ kbd) => prefs.keys || NAMED_KEY.test(kbd);
function setSingleKeys(/** @type {boolean} */ on) {
  prefs.keys = on;
  savePrefs();
  renderSingleKeys();
  toast(on ? "Single-key shortcuts on" : `Single-key shortcuts off: only shortcuts with ${keyText("⌘")} or ${keyText("⌥")} work`);
}
function renderSingleKeys() {
  for (const button of all$("[data-single-keys]")) button.setAttribute("aria-pressed", String((button.dataset.singleKeys === "on") === prefs.keys));
  $("single-keys-text").textContent = `Shortcuts that are a single letter, digit or symbol. When off, only shortcuts with ${keyText("⌘")} or ${keyText("⌥")} work`;
  document.body.toggleAttribute("data-single-off", !prefs.keys);
}
renderSingleKeys();
$("shortcuts").addEventListener("click", (event) => {
  const button = closest(event, "[data-single-keys]");
  if (button && (button.dataset.singleKeys === "on") !== prefs.keys) setSingleKeys(button.dataset.singleKeys === "on");
});

let liveHeight = 0, liveAt = 0, livePrice = 0;
let answeredAt = Date.now();
const offline = () => !navigator.onLine || Date.now() - answeredAt > 60_000;
const sayUnreached = () => ((text) => $("unreached-text").textContent !== text && ($("unreached-text").textContent = text))(navigator.onLine ? `Couldn't reach ${SERVER_HOST}` : "This device is offline");
/** @param {string} icon @param {string} name @param {string} text */
const liveFigure = (icon, name, text) => `<span class="live-figure"><span role="img" aria-label="${name}">${icon}</span>${escapeHtml(text)}</span>`;
let saidLost = /** @type {boolean | null} */ (null);
function updateStatus() {
  sayUnreached();
  const at = liveAt ? new Date(liveAt * 1000) : null;
  const lost = offline();
  if (!liveHeight && !lost) {
    $("live").classList.remove("offline");
    $("live").classList.add("waiting");
    for (const line of [$("home-live"), $("home-pulse")]) line.textContent = "Connecting";
    liveTip("connecting", `Connecting to ${SERVER_HOST}`);
    return;
  }
  $("live").classList.toggle("offline", lost);
  $("live").classList.remove("waiting");
  // (To a screen reader, as it connects, goes offline or comes back, in the live tip's words: not each price, as a
  // live line read whenever it changes would, each second; the logo's name says the state alone.)
  const state = lost ? (navigator.onLine ? `${SERVER_HOST} isn't answering` : "This device is offline") : `Live from ${SERVER_HOST}`;
  if (lost !== saidLost) (saidLost = lost), say(state);
  grid?.renderHorizon();
  const said = {
    status: lost ? '<span class="live-off">Offline</span>' : "",
    block: liveHeight ? liveFigure(ICONS.cube, "Block", grouped.format(liveHeight)) : "",
    price: !lost && livePrice ? liveFigure(ICONS.dollar, "Price in dollars", toTheCent.format(livePrice)) : at ? liveFigure(ICONS.clock, "Last heard at", at.toLocaleTimeString("en-GB", { hour: "2-digit", minute: "2-digit" })) : "",
  };
  $("home-live").innerHTML = [said.status, said.block, said.price].filter(Boolean).join(" ");
  const block = liveHeight ? `Last block ${grouped.format(liveHeight)}${at ? `, ${ago(at.getTime())}` : ""}` : "";
  $("home-pulse").replaceChildren(...[[[said.status, said.block].filter(Boolean).join(" "), block], [said.price, lost ? "Last heard" : "Price, as it trades"]].map(([html, tip]) => {
    const half = document.createElement("span");
    half.innerHTML = html;
    if (tip) half.dataset.tip = tip;
    return half;
  }));
  liveTip(lost ? "offline" : "live", state, ...(lost ? [block, "Trying again every 15 seconds"] : [livePrice ? `Price ${dollarsAndCents(livePrice)}, as it trades` : "", block, "A new block shows within seconds"]));
}
/** @param {string} state @param {...string} lines */
function liveTip(state, ...lines) {
  const brand = $("brand"), said = ["Bitview Studio", ...lines].filter(Boolean).join("\n");
  if (brand.dataset.tip === said) return;
  brand.dataset.tip = said;
  brand.setAttribute("aria-label", `Bitview Studio, ${state}`);
  if (tipTarget === brand && !$("tip").hidden) showTip(brand);
}

function refresh(refit = false) {
  for (const wake of [...drawnWaiters]) wake();
  clearLink();
  if (document.activeElement !== $("title") || $("title").value === typedFrom.get($("title"))) typedFrom.set($("title"), ($("title").value = chart.title));
  renderChartPath();
  $("title").placeholder = derivedTitle(chart);
  fitTitle();
  syncUndo();
  renderInterval();
  renderWindow();
  renderRanges();
  $("empty").hidden = chart.layers.length > 0;
  if (presets && !$("empty").hidden && grid.recommendedKey() !== grid.recommendedDrawn) grid.renderRecommended();
  const unseen = chart.layers.length > 0 && chart.layers.every((layer) => layer.hidden);
  $("all-hidden").hidden = !unseen;
  $("chart").classList.toggle("unseen", unseen);
  const back = chart.layers.length ? null : ((i) => (i >= 0 ? presetDoc(i) : previousChart()))(previousPreset());
  $("leave-empty").hidden = !(isPreview(chart) || emptyYours(chart));
  if (backToHome()) $("leave-empty").dataset.tip = `Back to ${homeName()}`;
  else $("leave-empty").dataset.tip = back ? `Back to ${titleOf(back)}` : "Back to Home";
  document.body.toggleAttribute("data-empty", !chart.layers.length);
  $("share").disabled = $("draw-toggle").disabled = $("panel-toggle").disabled = !chart.layers.length;
  if (!chart.layers.length && document.body.dataset.sheet === "panel") delete document.body.dataset.sheet;
  $("stage-foot").hidden = !chart.layers.length;
  if (home || !chart.layers.length) $("plot-keys").hidden = true;
  else if ($("plot-keys").hidden) relayout();
  withRest(refreshPanels);
  updateStatus();
  redrawChart(refit);
}
function refreshPanels() {
  renderCharts();
  renderLayers();
  inspector.renderInspector();
  drawThumbs();
  renderBelt();
}
function redrawChart(refit = false, now = false) {
  const asked = pending() ? asking.look : displayed?.look;
  if (home && !chartShown() && !pending()) undrawn = true;
  if ((home && undrawn) || (!refit && asked === lookKey(chart))) {
    updateLegends();
    if (chartShown()) sizePanes();
    if (!home) showDrawings(chart);
  } else if (builds()) now ? build(withLook(structuredClone(chart)), /** @type {Resolved} */ (resolved), refit) : buildSoon(refit);
  else draw(refit);
  if (chartShown() && chart.range !== displayed?.doc.range) showRange();
}
const builds = () => chartShown() && resolved?.key === dataKey(chart) && asking.key === resolved.key && drawLayers(chart).every((layer) => resolved?.drawn.has(layer.id));
/**
 * The open chart drawn again from the data at hand once what was pressed has shown: the row, the eye, the swatch answer
 * on the next frame, the chart on the one after (a stack of many built again is a third of a second on a phone);
 * however many edits ask by then, one build. A tab drawing no frames (hidden), at once. Any build meanwhile is it.
 */
function buildSoon(/** @type {boolean} */ refit) {
  buildRefit ||= refit;
  if (buildAsked) return void (document.hidden && buildNow());
  if (document.hidden) return void ((buildAsked = -1), buildNow());
  buildAsked = requestAnimationFrame(() => (buildAsked = requestAnimationFrame(buildNow)));
}
function buildNow() {
  if (!buildAsked) return;
  if (buildAsked > 0) cancelAnimationFrame(buildAsked);
  buildAsked = 0;
  const fit = buildRefit;
  buildRefit = false;
  if (builds()) build(withLook(structuredClone(chart)), /** @type {Resolved} */ (resolved), fit);
  else draw(fit);
  for (const wake of [...drawnWaiters]) wake();
}
document.addEventListener("visibilitychange", () => document.hidden && buildNow());

function railMenu(/** @type {HTMLElement} */ item, hover = false) {
  const folder = item.dataset.folderItem ? folderById(item.dataset.folderItem) : null, key = item.dataset.sectionItem, group = /** @type {Group | undefined} */ (item.dataset.groupItem);
  const doc = item.dataset.openChart ? chartById(item.dataset.openChart) : undefined, preset = item.dataset.openPreset;
  if (doc) return menus.menuOf(item, command.chartActions(doc.id, "row"), null, "right", { hover });
  if (preset !== undefined) return menus.menuOf(item, presetsEdit.presetActions(Number(preset), "row"), null, "right", { hover });
  if (group) return menus.menuOf(item, grid.groupRows(group), null, "right", { hover });
  if (key) return menus.menuOf(item, sectionFlyout(key), null, "right", { hover });
  if (folder) menus.menuOf(item, folderFlyout(folder.id), null, "right", { hover });
}
function railMenuKept(/** @type {HTMLElement} */ item) {
  if ($("menu").hidden || menus.ownerNow() !== item) return railMenu(item);
  menus.pinFlyout();
  menus.focusMenu();
}
/** @param {HTMLElement} anchor */
function menuNameOf(anchor) {
  const chartName = (/** @type {string | undefined} */ id) => ((doc) => (doc ? titleOf(doc) : ""))(chartById(id ?? ""));
  if (anchor.id === "chart-more") return titleOf(chart);
  const item = /** @type {HTMLElement | null} */ (anchor.closest("#chart-list") ? anchor.closest("#chart-list > *")?.querySelector(".nav-item, .list-head-name") ?? null : null);
  const at = (item ?? /** @type {HTMLElement | null} */ (anchor.closest("[data-home-card], [data-preset], [data-grid-folder], .layer-row[data-layer], [data-pane-menu]")))?.dataset ?? {};
  if (at.openChart || at.homeCard) return chartName(at.openChart ?? at.homeCard);
  if (at.openPreset !== undefined || at.preset !== undefined) return titleOf(presetDoc(Number(at.openPreset ?? at.preset)));
  if (at.folderItem || at.gridFolder) return folderById(at.folderItem ?? at.gridFolder)?.name ?? "";
  if (at.sectionItem) return /** @type {string} */ (JSON.parse(at.sectionItem).at(-1));
  if (at.groupItem) return GROUP_OF[/** @type {Group} */ (at.groupItem)][0];
  if (at.layer) return ((layer) => (layer ? labelOf(layer) : ""))(layerById(at.layer));
  return at.paneMenu ? `Pane ${chart.panes.findIndex((pane) => pane.id === at.paneMenu) + 1}` : "";
}
/** @param {string} id @returns {Action[]} */
function folderFlyout(id) {
  const counts = grid.chartCounts(), own = () => grid.folderRows(id, false, true).filter((row) => row.key !== "toggle" && row.key !== "view");
  return [
    ...grid.childrenOf(id).map((each) => ("layers" in each
      ? { key: each.id, icon: grid.chartMark(each), label: titleOf(each), marked: !home && chart.id === each.id, run: () => grid.go(chartById(each.id)) }
      : { key: `f:${each.id}`, icon: folderMark(each), label: each.name, aside: grid.shortCount(counts.get(each.id) ?? 0), marked: !home && Boolean(chart.folder && grid.isIn(chart.folder, each.id)), page: () => ({ title: each.name, heading: true, rows: () => folderFlyout(each.id) }) })),
    ...(grid.childrenOf(id).length ? [] : [{ key: "empty", label: "No charts yet", disabled: "Move charts here, or make one in it (New chart inside)" }]),
    ...grid.folderRows(id, false, true).filter((row) => row.key === "view").map((row) => ({ ...row, apart: true })),
    { key: "more", icon: ICONS.more, label: "More", page: () => ({ title: folderById(id)?.name ?? "", heading: true, rows: own }) },
  ];
}
/** @param {string} key @returns {Action[]} */
function sectionFlyout(key) {
  const node = presetNodeAt(JSON.parse(key)), all = /** @type {Preset[]} */ (presets);
  if (!node) return [];
  const open = !home && chart.preset !== undefined ? presetIndex(chart) : -1, holding = open < 0 ? [] : grid.sectionKeys(open);
  for (const each of node.items) if (typeof each === "number" && !presetMarks.has(grid.presetMarkKey(each))) grid.presetMarksAsked.add(each);
  drawPresetMarks();
  return [
    ...node.items.map((each) => (typeof each === "number"
      ? { key: `p:${each}`, icon: grid.presetMark(each), marked: each === open, label: ((doc) => (doc && titleOf(doc) !== all[each].t ? titleOf(doc) : presetName(all[each])))(keptPreset(each)), run: () => pickPreset(each) }
      : { key: each.key, icon: sectionMark(each), label: each.name, aside: grid.shortCount(each.under.length), marked: holding.includes(each.key), page: () => ({ title: each.name, heading: true, rows: () => sectionFlyout(each.key) }) })),
    { key: "view", icon: ICONS.arrowRight, label: "View", aside: "Its page", apart: true, run: () => goTo(asPlace({ path: JSON.parse(key) })) },
  ];
}

document.addEventListener("click", (event) => {
  const presetMenu = closest(event, "[data-preset-menu]");
  if (presetMenu) return presetsEdit.openPresetMenu(presetMenu);
  const menuFor = closest(event, "[data-chart-menu], [data-scale-menu], [data-pane-menu], [data-folder-menu]");
  const { chartMenu: chartId, paneMenu: paneId, folderMenu: folderId, scaleMenu: scaleId } = menuFor?.dataset ?? {};
  if (menuFor) return chartId ? grid.chartMenu(menuFor, chartId) : paneId ? presetsEdit.paneActions(menuFor, paneId) : folderId ? grid.folderActions(menuFor, folderId) : presetsEdit.scaleMenu(menuFor, /** @type {string} */ (scaleId));
  const page = closest(event, "#chart-list .chart-go");
  if (page) return goTo(asPlace(JSON.parse(/** @type {string} */ (page.dataset.place))));
  const folderItem = closest(event, "[data-folder-item]");
  if (folderItem) return railed() ? railMenu(folderItem) : grid.toggleFolder(/** @type {string} */ (folderItem.dataset.folderItem));
  const sectionMenu = closest(event, "[data-section-menu]");
  if (sectionMenu) return menus.menuOf(sectionMenu, grid.sectionRows(/** @type {string} */ (sectionMenu.dataset.sectionMenu)));
  const groupItem = closest(event, "[data-group-item]");
  if (groupItem) return railed() ? railMenu(groupItem) : grid.toggleGroup(/** @type {Group} */ (groupItem.dataset.groupItem));
  const groupMenu = closest(event, "[data-group-menu]");
  if (groupMenu) return menus.menuOf(groupMenu, grid.groupRows(/** @type {Group} */ (groupMenu.dataset.groupMenu)));
  const sectionItem = closest(event, "[data-section-item]");
  if (sectionItem) return railed() ? railMenu(sectionItem) : grid.toggleSection(/** @type {string} */ (sectionItem.dataset.sectionItem));
  const sidePreset = closest(event, "#chart-list [data-open-preset]");
  if (sidePreset) return pickPreset(Number(sidePreset.dataset.openPreset));
  const recentItem = closest(event, "[data-open-recent]");
  if (recentItem) return openRecent(/** @type {string} */ (recentItem.dataset.openRecent));
  const check = closest(event, "[data-pick-layer]");
  if (check) return pickLayer(/** @type {string} */ (check.dataset.pickLayer), event.shiftKey);
  const marker = closest(event, "[data-markers]");
  if (marker) return marker.dataset.markers?.startsWith("drawings") ? hideDrawings(!drawingsHidden) : presetsEdit.toggleHalvings();
  const view = closest(event, "[data-show-view]");
  if (view) return presetsEdit.setPaneView(/** @type {string} */ (view.dataset.pane), /** @type {0 | 1} */ (Number(view.dataset.showView)));
  const paneEye = closest(event, "[data-pane-eye]");
  if (paneEye) return inspector.hideLayers(chart.layers.filter((layer) => layer.pane === paneEye.dataset.paneEye).map((layer) => layer.id), paneShows(chart, /** @type {string} */ (paneEye.dataset.paneEye)));
  if (closest(event, "#templates [data-gallery-retry]")) return retryPresets();
  const template = closest(event, "[data-template]");
  const card = template && grid.recommendedCards[Number(template.dataset.template)];
  if (card) return pickPreset(card.preset);
  const found = closest(event, "[data-select], [data-toggle], [data-pane-mode], [data-pane-stack], [data-pane-reverse], [data-pane-outline], [data-pane-gradient], [data-action], .mobile-bar [data-sheet], [data-open-chart]");
  const target = byTouch && found?.matches(".legend [data-toggle]") ? /** @type {HTMLElement | null} */ (found.closest("[data-select]")) : found;
  if (!target) return;
  const { select: id, toggle, paneMode, paneStack, paneReverse, paneOutline, paneGradient, gradient, mode, action, sheet, openChart } = target.dataset;
  if (openChart) {
    delete document.body.dataset.sheet;
    grid.go(chartById(openChart));
  }
  else if (id && (event.metaKey || event.ctrlKey || event.shiftKey || (layerPicks.size && event.detail > 0))) pickLayer(id, event.shiftKey);
  else if (id && event.detail === 0 && layerPicks.size > 1 && layerPicks.has(id) && target.closest("#layer-list")) layersBatch("edit");
  else if (id) select(id);
  else if (toggle) event.altKey ? inspector.soloLayer(toggle) : inspector.toggleHidden(toggle);
  else if (paneMode) target.getAttribute("aria-disabled") === "true" ? notice(/** @type {string} */ (target.dataset.tip)) : presetsEdit.setPaneMode(paneMode, Number(mode));
  else if (paneStack) target.getAttribute("aria-disabled") === "true" ? notice(/** @type {string} */ (target.dataset.tip)) : presetsEdit.setPaneStack(paneStack, target.dataset.stack ?? "");
  else if (paneReverse) presetsEdit.setPaneReverse(paneReverse, Boolean(target.dataset.reverse));
  else if (paneOutline) presetsEdit.setPaneOutline(paneOutline, Boolean(target.dataset.outline));
  else if (paneGradient) presetsEdit.setPaneGradient(paneGradient, gradient ?? "");
  else if (action === "palette") openPalette({ resume: true });
  else if (action === "search") openPalette({ typing: true, onField: target.matches(".home-search") });
  else if (action === "new-chart") grid.newChart();
  else if (action === "close-preview") closePreview(chartById(target.dataset.preview ?? "") ?? chart);
  else if (action === "add-series") openPalette({ add: true, typing: target.hasAttribute("data-typing") });
  else if (action === "leave-empty") leaveEmpty();
  else if (action === "import") importCharts(target.dataset.into);
  else if (action === "download-all") menus.downloadAll();
  else if (action === "charts") goTo({ top: true });
  else if (action === "home-tab") {
    if (home) goTo({ top: true });
    else (galleryShown = Number.isInteger(leftPlace?.shown) ? /** @type {number} */ (leftPlace?.shown) : galleryShown), showHome(true, leftPlace);
  }
  else if (action === "toggle-sidebar") toggleSidebar();
  else if (action === "toggle-panel") togglePanel();
  else if (action === "link-view") linkView(!prefs.linkView);
  else if (action === "shortcuts") showShortcuts();
  else if (action === "theme") grid.themeMenu(target);
  else if (action === "show-every") inspector.showEvery();
  else if (action === "retry-all") {
    for (const name of drawSeries(chart)) {
      forget(name, chart.index);
      retryInfo(name);
    }
    draw();
  }
  else if (action === "full") presetsEdit.setFull();
  else if (action === "draw") pen.toggleBelt();
  else if (action === "close-palette") dismissPalette(true);
  else if (action === "close-shortcuts") grid.leave($("shortcuts"));
  else if (action === "close-guide") grid.leave($("guide"));
  else if (action === "guide") showGuide();
  else if (action === "close-sheet") {
    delete document.body.dataset.sheet;
    document.body.removeAttribute("data-peek");
    if (!phone.matches && panelShown()) togglePanel(false);
  }
  else if (sheet) {
    if (document.body.dataset.sheet === sheet) delete document.body.dataset.sheet;
    else document.body.dataset.sheet = sheet;
  }
});
withRest(() => {
  for (const dialog of all$("dialog")) {
    const scrim = Object.assign(document.createElement("div"), { className: "dialog-scrim", popover: "manual" });
    scrim.setAttribute("aria-hidden", "true");
    scrim.addEventListener("click", () => (dialog.id === "palette" ? dismissPalette(true) : grid.leave(dialog)));
    dialog.before(scrim);
    grid.dialogScrims.set(dialog, scrim);
    dialog.addEventListener("beforetoggle", (event) => {
      if (/** @type {ToggleEvent} */ (event).newState !== "closed") return;
      const to = grid.dialogOpeners.get(dialog), at = document.activeElement;
      if (dialogsShown.includes(dialog)) dialogsShown.splice(dialogsShown.indexOf(dialog), 1);
      holdPage(dialog, false);
      if (isOpen(scrim)) scrim.hidePopover();
      if (to instanceof HTMLElement && to.isConnected && (!at || at === document.body || dialog.contains(at))) to.focus({ preventScroll: true });
    });
    dialog.addEventListener("cancel", (event) => {
      if (keepsLeaving || event.defaultPrevented) return;
      event.preventDefault();
      grid.leave(dialog);
    });
    onClosed(dialog, () => $("toast").parentElement === dialog && (cutFade($("toast")), (topDialog() ?? document.body).append($("toast"))));
    if (dialog.id === "palette") onClosed(dialog, () => !isOpen(dialog) && command.palette.op.kind !== "formula" && (command.palette.op = { kind: "browse" }));
  }
});

const leaveField = () => (flushEdits(), typing(document.activeElement) && /** @type {HTMLElement} */ (document.activeElement).blur(), inspector.applyFormulaLeft());
// The keyboard, by touch. iOS lays it over the page (its viewport the same: what's under it, hidden) and, a field
// under it tapped, moves the whole page up to it (the header, a sheet's top, off the screen); Android makes the
// page as short as it leaves (resizes-content). While it's up (typing, and the page's foot under it, or the page
// a quarter shorter, not as the focus says: a field can keep it with the keyboard put away), `data-keys`
// ("covering" the page's foot, iOS's; "shortening" the page, Android's), the bars out of its way, and `--keys`,
// how much of the page's foot it covers (iOS's; none where the page is made shorter): a sheet's list goes on to
// its top. A field in a phone's sheet tapped (the layers', ⌘K), not focused yet, is focused where it is, its
// caret where it was tapped (the page left in place; a half sheet made whole once the keyboard's up, as ⌘K is:
// see onceKeys), and scrolled into what shows (seen on iOS 26.5: focused so, the page doesn't move, however low
// the field: its list scrolls). A tap in a field focused already is the browser's (its caret, a word selected). A
// menu's (a color's hex) is iOS's: no list there to scroll, iOS moves the page up to it, smoothly, on its own.
{
  const root = document.documentElement;
  const tallest = /** @type {Map<number, number>} */ (new Map());
  let covered = 0;
  const reveal = () => {
    const field = /** @type {HTMLElement} */ (document.activeElement), sheet = field.closest("#panel, #palette");
    if (!typing(field) || !sheet) return;
    Promise.allSettled(sheet.getAnimations().map((animation) => animation.finished)).then(() => {
      if (document.activeElement !== field || !document.body.hasAttribute("data-keys")) return;
      let list = field.parentElement;
      while (list && list !== sheet.parentElement && !scrollsY(list)) list = list.parentElement;
      if (!list || list === sheet.parentElement) return;
      const box = field.getBoundingClientRect(), view = list.getBoundingClientRect();
      const top = view.top + (parseFloat(getComputedStyle(list).scrollPaddingTop) || 0), bottom = Math.min(view.bottom, viewport.offsetTop + viewport.height) - 12;
      const by = box.bottom > bottom ? Math.min(box.bottom - bottom, box.top - top) : Math.min(0, box.top - top);
      if (Math.abs(by) > 1) list.scrollBy({ top: by, behavior: reduced() ? "auto" : "smooth" });
    });
  };
  const read = () => {
    const layout = root.clientHeight, most = Math.max(tallest.get(innerWidth) ?? 0, layout);
    tallest.set(innerWidth, most);
    const under = Math.max(0, Math.round(layout - viewport.height));
    const covering = under > 120, up = TOUCH && typing(document.activeElement) && (covering || most - layout > most / 4);
    if (up) document.body.dataset.keys = covering ? "covering" : "shortening";
    else delete document.body.dataset.keys;
    const keys = up && covering ? under : 0;
    if (keys !== covered) for (const sheet of [$("panel"), $("palette")]) sheet.style.setProperty("--keys", `${(covered = keys)}px`);
    if (up) grid.keysCame(), reveal();
  };
  viewport.addEventListener("resize", read);
  addEventListener("resize", read);
  addEventListener("focusin", read);
  let press = /** @type {{ x: number, y: number, at: number, target: EventTarget | null } | null} */ (null);
  addEventListener("touchstart", (event) => (press = event.touches.length === 1 ? { x: event.touches[0].clientX, y: event.touches[0].clientY, at: event.timeStamp, target: event.target } : null), { capture: true, passive: true });
  addEventListener("touchmove", (event) => press && Math.hypot(event.touches[0].clientX - press.x, event.touches[0].clientY - press.y) > SLOP && (press = null), { capture: true, passive: true });
  addEventListener(
    "touchend",
    (event) => {
      const target = /** @type {HTMLElement} */ (event.target), was = press, touch = event.changedTouches[0];
      press = null;
      const field = /** @type {HTMLElement | null | undefined} */ (typing(target) ? target : target.closest("label")?.control);
      if (!phone.matches || !field || !was || was.target !== target || document.activeElement === field || !typing(field) || !field.matches(":read-write") || !field.closest("#panel, #palette")) return;
      if (Math.hypot(touch.clientX - was.x, touch.clientY - was.y) > SLOP || event.timeStamp - was.at > 500) return;
      event.preventDefault();
      field.focus({ preventScroll: true });
      const at = document.caretPositionFromPoint?.(touch.clientX, touch.clientY);
      if (at?.offsetNode === field && (field instanceof HTMLInputElement || field instanceof HTMLTextAreaElement)) field.setSelectionRange(at.offset, at.offset);
      if (field.closest("#panel") && heldUpright.matches && !sheetWhole()) grid.onceKeys(() => document.activeElement === field && document.body.dataset.sheet === "panel" && !sheetWhole() && grid.sizeSheet($("panel"), () => grid.panelSize(true)));
    },
    { capture: true, passive: false },
  );
}
function selectBy(delta) {
  if (!chart.layers.length) return;
  const from = layerPicks.size ? pickHead : selected, index = chart.layers.findIndex((layer) => layer.id === from);
  select(chart.layers[clamp(index < 0 ? (delta > 0 ? 0 : chart.layers.length - 1) : index + delta, 0, chart.layers.length - 1)].id);
}
document.addEventListener("keydown", (event) => {
  if (composing(event)) return;
  const mod = event.metaKey || event.ctrlKey;
  const key = event.key.toLowerCase();
  if (menus.flyout && event.getModifierState(event.key)) return;
  if (menus.flyout) {
    const at = document.activeElement, free = !at || at === document.body || at === menus.ownerNow();
    if (!mod && free && upDown(event)) {
      event.preventDefault();
      menus.pinFlyout();
      const items = all$("button:not(:disabled)", $("menu"));
      return void items[event.key === "ArrowDown" ? 0 : items.length - 1]?.focus();
    }
    menus.hideMenu();
    if (event.key === "Escape") return void event.preventDefault();
  }
  if (mod && key === "k") {
    event.preventDefault();
    if (!isOpen($("palette")) || $("palette").dataset.leaving) return openPalette({ resume: true });
    if (!$("menu").hidden && $("menu").closest("#palette")) return void (menus.hideMenu(), command.paletteKeys());
    return command.showActions();
  }
  if (document.querySelector("dialog:popover-open")) return;
  if (!$("menu").hidden && event.key !== "Escape") return;
  const search = /** @type {HTMLElement | null} */ (closest(event, "[data-typing]"));
  if (search && event.key.length === 1 && event.key !== " " && !mod && !event.altKey) {
    event.preventDefault();
    openPalette({ add: search.dataset.action === "add-series", query: event.key, typing: true, onField: search.matches(".home-search") });
    const input = $("palette-input");
    return void input.setSelectionRange(input.value.length, input.value.length);
  }
  if (mod && /^[1-9]$/.test(event.key) && !event.shiftKey && !event.altKey) {
    event.preventDefault();
    grid.numbered()[Number(event.key) - 1]?.click();
    return;
  }
  if (mod && key === "s") {
    event.preventDefault();
    flushEdits();
    if (isPreview(chart) && !home) return chart.layers.length ? keepChart() : notice("Nothing to save yet: add a layer");
    return save(true);
  }
  if (mod && key === "z" && !typing(event.target)) {
    event.preventDefault();
    if (!event.shiftKey) return undoLast();
    return redoLast();
  }
  if (mod && key === "y" && !typing(event.target)) {
    event.preventDefault();
    return redoLast();
  }
  if (mod && key === "a" && grid.yoursShown() && !typing(event.target)) {
    event.preventDefault();
    for (const doc of grid.shownCharts()) grid.picked.add(doc.id);
    return grid.renderPicked();
  }
  const inList = typing(event.target) || !document.activeElement?.matches?.(":focus-visible") ? null : /** @type {HTMLElement | null} */ (document.activeElement.closest("#chart-list, #home-grid"));
  if (inList && !event.altKey && ((mod && !event.shiftKey && key === "d") || (!mod && erasing(event)))) {
    event.preventDefault();
    const item = /** @type {HTMLElement | null} */ (document.activeElement?.closest?.("[data-chart-row], [data-home-card]") ?? null);
    const id = item?.dataset.chartRow ?? item?.dataset.homeCard ?? "", doc = chartById(id), inGrid = inList.id === "home-grid";
    if (!doc || isPreview(doc)) return;
    // (Once a press: held, the key repeats, and each copy's picks would be copied again for as long as it's held.)
    if (key === "d") return void (event.repeat || (inGrid ? grid.batch("duplicate", null, grid.picked.has(id) ? null : [id]) : grid.duplicateCharts([doc])));
    return inGrid ? grid.batch("delete", null, grid.picked.has(id) ? null : [id]) : deleteChart(doc);
  }
  const layerRow = typing(event.target) ? null : /** @type {HTMLElement | null} */ (document.activeElement?.closest?.(".layer-row[data-layer]") ?? null);
  if (layerRow && event.altKey && !mod && upDown(event)) {
    event.preventDefault();
    const id = /** @type {string} */ (layerRow.dataset.layer);
    inspector.moveLayer(id, event.key === "ArrowUp" ? -1 : 1);
    return void presetsEdit.refocus(`.layer-row[data-layer="${CSS.escape(id)}"] .row-main`);
  }
  if (chosen.size && !home && !event.altKey && ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(event.key) && !typing(event.target) && !document.activeElement?.closest?.("#chart-list, #panel, #belt")) {
    event.preventDefault();
    return heatmaps.nudgeDrawings(event.key, event.shiftKey, mod, event.repeat);
  }
  if (mod && !event.shiftKey && !event.altKey && key === "d" && !home && (chosen.size || layerPicks.size || selected) && !typing(event.target)) {
    event.preventDefault();
    if (event.repeat) return;
    return chosen.size ? heatmaps.duplicateDrawings([...chosen]) : layerPicks.size ? layersBatch("duplicate") : inspector.duplicateLayer(selected);
  }
  if (mod && key === "a" && !home && chart.layers.length && !typing(event.target) && (layerPicks.size || $("layers-view").contains(document.activeElement))) {
    event.preventDefault();
    return pickEvery();
  }
  if (mod && key === "a" && !home && chart.drawings?.length && !typing(event.target)) {
    event.preventDefault();
    return chooseDrawings(chart.drawings.map((drawing) => drawing.id));
  }
  if (mod || event.altKey || typing(event.target)) return;
  const row = /** @type {HTMLElement | null} */ (document.activeElement?.matches?.(".layer-row .row-main") ? document.activeElement.closest(".layer-row[data-layer]") : null);
  if ((event.key === " " || key === "x") && row && !event.shiftKey && !home) {
    event.preventDefault();
    return pickLayer(/** @type {string} */ (row.dataset.layer));
  }
  if (!prefs.keys && event.key.length === 1) return;
  if (grid.yoursShown() && erasing(event) && grid.picked.size) {
    event.preventDefault();
    return grid.batch("delete");
  }
  if (home && !["/", "n", "?", "\\", "escape"].includes(key)) return;
  if (event.key === "/") openPalette({ resume: true });
  else if (key === "f" && event.shiftKey && !home) presetsEdit.setFull();
  else if (key === "f") inspector.addFormula();
  else if (key === "n") grid.newChart();
  else if (key === "a") openPalette({ add: true });
  else if (event.key === "?") showShortcuts();
  else if (event.key === "\\" && !phone.matches) toggleSidebar();
  else if (key === "p") togglePanel();
  else if (key === "v") linkView(!prefs.linkView);
  else if (event.key === "Escape") {
    if (!home && $("menu").hidden && clearMeasure()) return;
    if (home && grid.picked.size && $("menu").hidden) grid.clearPicks();
    else if (!$("menu").hidden) menus.dismissMenu(true);
    else if (!home && (placing || moving || boxing)) heatmaps.finishPath() || stopDrawing();
    else if (!home && (tool || selectingArea || magnifying)) takeUp("");
    else if (!home && chosen.size) chooseDrawings([]);
    else if (!home && inspector.editingMany()) deselect();
    else if (!home && layerPicks.size) clearLayerPicks(true);
    else if (!home && selected) deselect();
    else if (document.body.hasAttribute("data-peek")) {
      keepFocus($("panel"), () => $("panel-toggle"), () => $("plot-keys"));
      document.body.removeAttribute("data-peek");
    }
    else if (document.body.dataset.sheet) delete document.body.dataset.sheet;
    else if (!home && tablet.matches && panelShown()) togglePanel(false);
    else if (!home && !chart.layers.length && (isPreview(chart) || emptyYours(chart))) leaveEmpty();
    else if (!home && fullOn() && panelShown()) togglePanel(false);
    else if (fullOn() && !event.repeat) presetsEdit.setFull(false);
  } else if (upDown(event) && event.shiftKey) extendPicks(event.key === "ArrowDown" ? 1 : -1);
  else if (upDown(event)) selectBy(event.key === "ArrowDown" ? 1 : -1);
  else if (erasing(event) && !home && placing && tool === "path") unplacePath();
  else if (erasing(event) && (chosen.size || layerPicks.size || selected)) chosen.size ? heatmaps.removeDrawings([...chosen]) : layerPicks.size ? layersBatch("remove") : inspector.removeLayer(selected);
  else if (key === "h" && event.shiftKey && chart.drawings?.length) hideDrawings(!drawingsHidden, true);
  else if (key === "h" && !event.shiftKey && layerPicks.size) layersBatch("hide");
  else if (key === "h" && !event.shiftKey && layerById(selected)) inspector.toggleHidden(selected);
  else if (key === "l" && chart.layers.length) presetsEdit.toggleLog(command.scalePane().id);
  else if (key === "m" && chart.layers.length) takeUp(tool === "ruler" ? "" : "ruler");
  else if (key === "d" && chart.layers.length) focusBelt();
  else if (key === "g" && chart.layers.length) command.openPage(command.datePage());
  else if (event.key === "-" || event.key === "_") zoom(1.6);
  else if (event.key === "=" || event.key === "+") zoom(1 / 1.6);
  else if (["1", "2", "3", "4", "5"].includes(event.key)) {
    const [id, text] = RANGES[Number(event.key) - 1];
    if (presetFits(id)) menus.setRange(id);
    else notice(`${text} is more than fits at ${INTERVAL_LABEL[chart.index]}`);
  }
  else if (event.key === "[" || event.key === "]") menus.stepInterval(event.key === "]" ? 1 : -1);
  else if ((event.key === "," || event.key === ".") && chart.layers.length) chart.layers.some((layer) => signalShown(layer)) ? command.stepSignal(event.key === "." ? 1 : -1) : notice("No signal shown to step through");
  else return;
  event.preventDefault();
});

const HOLD = MAC ? "Meta" : "Control";
for (const element of all$(MAC ? "kbd" : "kbd, #shortcuts dt, #shortcuts h3")) element.textContent = keyText(element.textContent ?? "");
drawKeys(document.body);
if (!MAC) for (const element of all$("[data-tip*='⇧']")) element.dataset.tip = keyText(element.dataset.tip ?? "");
/** Long enough that a quick ⌘Z or ⌘C never flashes the keys, short enough to feel instant when held. */
const HINT_DELAY = 200;
let hintTimer = 0;
const hinting = () => document.body.hasAttribute("data-hints");
function placeHints() {
  const layer = $("hints");
  layer.replaceChildren();
  if (!hinting()) return;
  if (document.querySelector("dialog:popover-open")) return hideHints();
  const badges = [];
  // Only what ⌘ + a key runs: the single keys mean something else while ⌘ is down (⌘1 opens a chart, ⌘P prints).
  for (const element of all$("[data-kbd*='⌘']")) {
    if (element.querySelector(":scope > kbd.mod")) continue;
    if (/** @type {HTMLButtonElement} */ (element).disabled || element.closest("[inert]") || !element.checkVisibility({ checkVisibilityCSS: true })) continue;
    const rect = element.getBoundingClientRect();
    if (!rect.width || rect.bottom < 0 || rect.top > innerHeight) continue;
    const badge = document.createElement("kbd");
    badge.className = "key-hint";
    badge.textContent = keysOf(element.dataset.kbd);
    layer.append(badge);
    const width = badge.offsetWidth, height = badge.offsetHeight;
    const below = rect.bottom + 6 + height <= innerHeight - 4;
    badges.push({ badge, width, x: rect.left + rect.width / 2, top: below ? rect.bottom + 6 : rect.top - 6 - height });
  }
  badges.sort((a, b) => a.top - b.top || a.x - b.x);
  for (let i = 1; i < badges.length; i++) {
    const item = badges[i], last = badges[i - 1];
    const overlap = last.x + last.width / 2 + 4 - (item.x - item.width / 2);
    if (Math.abs(last.top - item.top) >= 4 || overlap <= 0) continue;
    item.x += overlap / 2;
    for (let j = i - 1; j >= 0 && Math.abs(badges[j].top - item.top) < 4; j--) badges[j].x -= overlap / 2;
  }
  for (const item of badges) {
    item.badge.style.left = `${Math.round(clamp(item.x - item.width / 2, 4, innerWidth - item.width - 4))}px`;
    item.badge.style.top = `${Math.round(item.top)}px`;
  }
}
function hideHints() {
  clearTimeout(hintTimer);
  hintTimer = 0;
  delete document.body.dataset.hints;
  $("hints").replaceChildren();
}
addEventListener("keydown", (event) => {
  if (event.key === HOLD) {
    // ⇧⌘ starts a chord, not a hold: often the system's own (⇧⌘3, ⇧⌘4: screenshots, which shouldn't show the keys).
    if (event.repeat || hinting() || hintTimer || event.altKey || event.shiftKey) return;
    hintTimer = setTimeout(() => {
      hintTimer = 0;
      if (document.querySelector("dialog:popover-open")) return;
      document.body.dataset.hints = "";
      placeHints();
    }, HINT_DELAY);
    return;
  }
  if (event.key === "Shift" && hinting()) return;
  clearTimeout(hintTimer);
  hintTimer = 0;
  if (hinting()) requestAnimationFrame(placeHints);
}, true);
addEventListener("keyup", (event) => event.key === HOLD && hideHints());
// The system can take a chord whole (⇧⌘4, then a drag for the screenshot) and the page never hears ⌘ come up:
// the first key, pointer or wheel event that says ⌘ isn't held puts the keys away.
const held = (event) => (HOLD === "Meta" ? event.metaKey : event.ctrlKey);
for (const type of ["keydown", "keyup", "pointermove", "pointerdown", "wheel"]) {
  addEventListener(type, (event) => (hinting() || hintTimer) && !held(event) && hideHints(), { capture: true, passive: true });
}
addEventListener("blur", hideHints);
addEventListener("resize", hideHints);
document.addEventListener("visibilitychange", hideHints);
addEventListener("scroll", () => hinting() && placeHints(), true);

// The server's health says which block its series are computed up to: a light, uncached read, every 15 seconds while the page is seen and at once when it's seen again. A new block refetches the chart, and the dot flashes.

$("live").addEventListener("animationend", (event) => event.animationName === "block" && $("live").classList.remove("block"));
let checking = false;
async function checkLive() {
  if (document.visibilityState !== "visible" || checking) return;
  if (!command) return void (displayed && loadRest().then(checkLive, () => ((answeredAt = 0), updateStatus())));
  checking = true;
  let health;
  try {
    health = await client.getHealth({ cache: false });
  } catch {
    return updateStatus();
  } finally {
    checking = false;
  }
  answeredAt = Date.now();
  grid?.retryPlots();
  // (A layer that couldn't load (bitview.space away a while): asked for again as it answers, not at the next block,
  // ten minutes on; drawn once, a new block's draw if one came. Not while it's still being drawn (a slow answer
  // never overtaken), nor under a menu (its button kept). Under the Charts page, once it's back: see leaveHome.)
  const idle = Boolean(chart && !home && settled() && $("menu").hidden);
  const retry = idle && Boolean(resolved?.failed.length) && grid.retryFailed(/** @type {Resolved} */ (resolved), chart.index);
  const behind = idle && !retry && refreshFailed(chart);
  updateStatus();
  const height = health.computedHeight;
  if (!Number.isInteger(height) || height === liveHeight) return void (retry ? draw() : behind && draw(false, true));
  const first = !liveHeight;
  liveHeight = height;
  liveAt = Number.isFinite(health.lastIndexedAtUnix) ? health.lastIndexedAtUnix : 0;
  updateStatus();
  if (first && chart?.layers.length) {
    renderLayers();
    repaintDrawings();
  }
  if (first) return void (retry && draw());
  heatmaps.heatFresh();
  const live = $("live");
  live.classList.remove("block");
  void live.offsetWidth;
  live.classList.add("block");
  if (chart && drawSeries(chart).length) home ? (blockAtHome = true) : draw(false, true);
}
setInterval(checkLive, 15_000);
addEventListener("offline", () => (sayUnreached(), !$("live").classList.contains("waiting") && updateStatus()));
addEventListener("online", () => (sayUnreached(), checkLive()));
document.addEventListener("visibilitychange", () => document.visibilityState === "visible" && (answeredAt = Date.now()));
document.addEventListener("visibilitychange", checkLive);
withRest(checkLive);

// bitview.space's live price (round-dollar outputs of the last blocks and the mempool), read each second while the page is seen. Layers drawing the price itself have its point for the interval the clock is in moved there, a candle's high or low pushed out. Between blocks, the server's latest interval can be over: a live bar then opens for the current one, from the last close. A formula of the price waits for the next block, whose data takes over.

const usd = (/** @type {number} */ price) => price, sats = (/** @type {number} */ price) => 1e8 / price;
/** @type {Map<string, (price: number) => number>} */
let LIVE = new Map();
function liveLayers() {
  if (home || !settled() || (FINE.has(chart.index) && !spans.get(spanKey(chart.id, chart.index))?.live)) return [];
  return chart.layers.filter((layer) => LIVE.has(layer.series ?? "") && !layer.hidden && layer.type !== "histogram" && ((pane) => !drawnInto(chart, pane) && !averageFor(chart, pane).active)(chart.panes.find((pane) => pane.id === layer.pane)));
}
const ownLive = new WeakSet();
/** @template {Float64Array | any[]} T @param {T} vector @returns {T} */
const liveCopy = (vector) => {
  if (ownLive.has(vector)) return vector;
  const copy = /** @type {T} */ (vector.slice());
  ownLive.add(copy);
  return copy;
};
let liveLast = "";
function widened(/** @type {Resolved} */ data, /** @type {number[]} */ added) {
  const grown = (/** @type {any} */ values, /** @type {string} */ field) => {
    const out = Array.isArray(values) ? values.concat(added.map(() => null)) : new values.constructor(values.length + added.length);
    if (!Array.isArray(values)) {
      out.set(values);
      out.fill(field === "closes" ? livePrice || NaN : values instanceof Float64Array ? NaN : values instanceof Int8Array ? -1 : 0, values.length);
    }
    ownLive.add(out);
    return out;
  };
  const results = new Map([...data.results].map(([id, result]) => [id, mapResult(result, grown)]));
  return { ...data, times: data.times.concat(added), results };
}
function movePrice(price) {
  const layers = liveLayers();
  if (!Number.isFinite(price) || price <= 0 || !resolved?.times.length || !layers.length) return;
  const index = chart.index, latest = indexAt(index, /** @type {number} */ (resolved.times.at(-1))), current = indexAt(index, Date.now() / 1000);
  if (current - latest > 100) return;
  /** @type {number[]} */
  const added = [];
  for (let i = latest + 1; i <= current; i++) added.push(timeAt(index, i));
  if (added.length) {
    resolved = widened(resolved, added);
    showTimes(resolved.times);
    dataRevision++;
    for (const series of heatmaps.heatSeries) for (let k = 0; k < added.length; k++) series.place(resolved.times.length - added.length + k, price);
  }
  const data = /** @type {Resolved} */ (resolved), at = data.times.length - 1;
  const key = `${idOf(data)}:${price}:${layers.map((layer) => layer.id).join()}`;
  if (key === liveLast) return;
  liveLast = key;
  dataRevision++;
  let moved = false, regrade = false;
  /** @type {string[]} */
  const toPaint = [];
  for (const layer of layers) {
    const result = data.results.get(layer.id), value = /** @type {(price: number) => number} */ (LIVE.get(layer.series ?? ""))(price);
    if (!result?.values) continue;
    const opens = added.length > 0 || !Number.isFinite(result.values[at]);
    const last = opens ? lastIn(result.values, 0, at) : at;
    if (last < 0 || at - last > 100 || !Number.isFinite(result.values[last])) continue;
    const values = liveCopy(result.values), series = drawn.get(layer.id), own = series?.engine instanceof Candles ? series.engine : null;
    const was = layer.byGradient && !layer.upToDate ? rangeOf(result.values).join() : "";
    values[at] = value;
    rescan(values, at);
    if (was && rangeOf(values).join() !== was) regrade = true;
    const candles = result.opens && Number.isFinite(result.opens[last]) ? { opens: liveCopy(result.opens), highs: liveCopy(/** @type {Vector} */ (result.highs)), lows: liveCopy(/** @type {Vector} */ (result.lows)) } : null;
    if (candles) {
      const open = opens ? values[last] : candles.opens[at];
      candles.opens[at] = open;
      candles.highs[at] = Math.max(opens ? open : candles.highs[at], value);
      candles.lows[at] = Math.min(opens ? open : candles.lows[at], value);
      if (own && own.close.length > at) own.set(at, open, candles.highs[at], candles.lows[at], value), engine.invalidate();
      else if (own) own.setValues(candleColumns({ ...result, values, ...candles }, values, modeOf(displayed?.doc ?? chart, layer) === 1, (/** @type {number} */ v) => v)), engine.invalidate(true);
    }
    data.results.set(layer.id, { ...result, values, ...candles });
    const made = built.get(layer.id);
    if (made && !made.stacked) built.set(layer.id, { ...made, shown: values, ...candles });
    if (series instanceof Standard && !own) liveColumn(series, values, at, value);
    if (paintedLayers.has(layer.id)) toPaint.push(layer.id);
    moved = true;
  }
  const doc = displayed?.doc ?? chart, inDoc = (/** @type {string | undefined} */ id) => doc.layers.find((item) => item.id === id);
  const movedIds = new Set(layers.map((layer) => layer.id)), painting = new Set(toPaint);
  for (const layer of doc.layers) {
    const series = drawn.get(layer.id);
    if (layer.type !== "bubbles" || !series || !(movedIds.has(layer.id) || movedIds.has(layer.ref ?? ""))) continue;
    const on = inDoc(layer.ref), y = on ? /** @type {Vector | undefined} */ (built.get(on.id)?.shown)?.[at] ?? NaN : 0.5;
    series.place(at, data.results.get(layer.id)?.values?.[at] ?? NaN, on && !(Number.isFinite(y) && (modeOf(doc, on) !== 1 || y > 0)) ? NaN : y);
  }
  if (bubbleAt.time !== null) lightBubble(bubbleAt.time, bubbleAt.pane);
  for (const id of paintedLayers.keys()) {
    const layer = inDoc(id), series = drawn.get(id), has = paintedLayers.get(id)?.colors.length === at + 1 && Number.isFinite(series instanceof Cloud ? series.high[at] : built.get(id)?.shown?.[at]);
    if (!layer || painting.has(id) || !has) continue;
    let source = inDoc(layer.colorFrom);
    for (const seen = new Set([id]); source && !seen.has(source.id) && !movedIds.has(source.id); source = inDoc(source.colorFrom)) seen.add(source.id);
    if (source && movedIds.has(source.id)) toPaint.push(id);
  }
  for (const id of toPaint) {
    const layer = inDoc(id), painted = paintedLayers.get(id);
    if (layer && painted) paintPoint(painted, at, ink(paintOf(doc, layer, data)?.(at) ?? layer.color));
  }
  if (lensAt) lookAt(lensAt);
  if (!moved) return;
  liveMoves++;
  if (regrade) buildSoon(false);
  windows.data = null;
  drawOverview();
  if (crossSet) engine.render(false), holdCross();
  updateValues();
}
let pricePending = false, priceTimer = 0, priceDelay = 1000;
async function pollPrice() {
  if (!tree || pricePending || document.visibilityState !== "visible") return;
  clearTimeout(priceTimer);
  pricePending = true;
  try {
    if ((phone.matches ? home : !fullOn()) || liveLayers().length) {
      const version = drawVersion, price = await client.getLivePrice();
      if (typeof price === "number" && Number.isFinite(price) && price !== livePrice) {
        livePrice = price;
        updateStatus();
      }
      if (version === drawVersion) movePrice(price);
    }
    priceDelay = 1000;
  } catch {
    priceDelay = Math.min(priceDelay * 2, 30_000);
  } finally {
    pricePending = false;
    if (document.visibilityState === "visible") priceTimer = setTimeout(pollPrice, priceDelay);
  }
}
document.addEventListener("visibilitychange", pollPrice);
withRest(pollPrice);

let lostDrawings = 0;
async function fromHash(run = linkRun, at = 0) {
  const hash = location.hash.slice(1), params = new URLSearchParams(hash), key = /^(?:chart|image|c)(?==)/.exec(hash)?.[0] ?? (params.has("chart") ? "chart" : params.has("image") ? "image" : params.has("c") ? "c" : "");
  const shared = key ? params.get(key) : null;
  if (!shared) return null;
  picturing = key === "image";
  if (picturing) pictureDrawing();
  Object.assign(reading, { hash: location.hash, run, at });
  return linked(new RegExp(`(?:^|&)${key}=(.*)$`, "s").exec(hash)?.[1] ?? "", shared, run, at, key === "c");
}
function clearLink() {
  if (picturing || !isLinkHash(location.hash) || location.hash === brokenLink || (location.hash === reading.hash && currentRead(reading.run, reading.at))) return;
  history.replaceState(history.state, "", `${location.pathname}${location.search}`);
}
const reading = { hash: "", run: 0, at: 0 };
const currentRead = (/** @type {number} */ run, /** @type {number} */ at) => run === linkRun && !(at && navigations > at);
/**
 * @param {Chart} doc
 * @param {ReturnType<typeof linkLookOf>} look
 * @param {{ index: string, range: string, window?: string, show?: string }} [made]
 */
function takeLinkLook(doc, look, made) {
  doc.range = look.range ?? made?.range ?? doc.range;
  ((show) => show !== undefined && showLinkView(doc, show))(look.show ?? made?.show);
  const offered = chartWindow(doc)?.offered ?? [], window = (look.window && windowNamed(look.window, offered)) || made?.window;
  if (window) atWindow(doc, window);
  doc.index = (look.index && menus.supportedIntervals(doc)(look.index) ? look.index : undefined) ?? made?.index ?? doc.index;
  if (look.range || look.view || made) {
    const all = viewsNow();
    delete all[doc.id];
    putViews(all);
  }
  if (look.view) keepCarriedView(doc, { view: datedView(doc.index, look.view) });
}
let linkError = "";
/** @param {string} written @param {string | null} shared @param {number} run @param {number} [at] */
async function linked(written, shared, run, at = 0, base32 = false) {
  const plain = /^(?:%20|%0a|%0d|%09|\s)*(?:\{|%7b|%257b)/i.test(written);
  try {
    lostDrawings = 0;
    await presetsSettled();
    const field = /&(?:chart|image)=/.exec(written), doc = plain ? (() => {
      try {
        return fromWritten(written);
      } catch (error) {
        if (!field) throw error;
        return fromWritten(written.slice(0, field.index));
      }
    })() : await fromLink(/** @type {string} */ (shared), base32);
    if (!doc.layers.length) throw new Error("it lists no layers");
    if (!currentRead(run, at)) return null;
    if (shared !== null) takeLinkLook(doc, linkLookOf(new URLSearchParams(plain ? written.replace(/^.*(?:\}|%7d|%257d)/is, "") : location.hash.slice(1))));
    const twin = twinOf(doc);
    const lost = lostDrawings ? `; ${plural(lostDrawings, "drawing")} couldn't be read` : "";
    if (shared !== null && !picturing) history.replaceState(home ? { ...(history.state?.home ? history.state : {}), ...homeState() } : { chart: (twin ?? doc).id }, "", `${location.pathname}${location.search}`);
    if (shared !== null) reading.hash = "";
    if (!picturing) twin || lost ? toast(`Opened a shared chart${twin ? " you already have" : ""}${lost}`) : say("Opened a shared chart as a preview. Save it to keep it in your Library");
    return twin ?? doc;
  } catch (error) {
    if (!currentRead(run, at)) return null;
    linkError = plain ? `That link's chart couldn't be read: ${error instanceof SyntaxError ? `its JSON is broken (${error.message})` : /** @type {Error} */ (error).message}` : "That link doesn't contain a chart";
    if (shared !== null) (brokenLink = reading.hash || location.hash), (reading.hash = "");
    notice(linkError);
    return null;
  }
}
/** @returns {Chart} */
function showLinked(/** @type {Chart} */ doc) {
  linkWindow = (doc === chart || library.charts.includes(doc) ? doc : previewLike(doc) ?? doc).id;
  if (doc === chart) leaveHome(() => ((displayed = null), refresh(true)));
  else if (library.charts.includes(doc)) leaveHome(() => openChart(doc));
  else return showPreview(doc);
  return doc;
}
let linkRun = 0;
addEventListener("hashchange", () => {
  const params = new URLSearchParams(location.hash.slice(1));
  if (!params.has("chart") && !params.has("image") && !params.has("c")) return void (++linkRun, (picturing = picturing && document.body.hasAttribute("data-picture")), document.body.hasAttribute("data-picture") && leavePicture());
  readLink();
});
function readLink() {
  grid.closeDialogs();
  const run = ++linkRun, at = navigations + 1;
  navigate((async () => {
    const doc = await fromHash(run, at);
    if (run !== linkRun) return;
    if (navigations > at) return void ((picturing = false), clearLink());
    const shown = doc ? showLinked(doc) : null;
    if (shown && picturing) await until(() => chart === shown && !home, 5000);
    if (shown && picturing && run === linkRun) showPicture();
    else if (picturing && run === linkRun) pictureFailed(linkError);
    else if (document.body.hasAttribute("data-picture")) leavePicture();
  })());
}

let picturing = false;
let navigating = Promise.resolve();
function navigate(/** @type {Promise<void>} */ way) {
  navigations++;
  navigating = way;
  way.catch(() => {});
  return way;
}
let pictureRun = 0;
async function showPicture() {
  const run = ++pictureRun;
  document.body.toggleAttribute("data-picture", true);
  presetsEdit.layOutAlone(true);
  const image = /** @type {HTMLImageElement} */ ($("picture-img"));
  pictureDrawing();
  try {
    await studio.ready();
    document.title = `${titleOf(chart)} · Bitview Studio`;
    const at = navigations;
    if (await grid.freshThumb(chart, true)) saveThumbsSoon();
    const url = (await chartImage()).toDataURL("image/png");
    if (at !== navigations) throw new Error("Another chart was opened meanwhile");
    if (run !== pictureRun || !picturing) return;
    image.src = url;
    image.alt = `${titleOf(chart)}. ${chartFacts()}`;
    const link = await menus.shareLink(chart);
    if (run !== pictureRun || !picturing) return;
    /** @type {HTMLAnchorElement} */ ($("picture-open")).href = link.slice(link.indexOf("#"));
    $("picture-open").hidden = false;
    $("picture-note").hidden = true;
  } catch (error) {
    if (run !== pictureRun) return;
    pictureFailed(`Couldn't make its picture: ${/** @type {Error} */ (error).message}`);
  }
}
let noteRun = 0;
function pictureDrawing() {
  const run = ++noteRun, note = $("picture-note");
  $("picture-open").hidden = true;
  $("picture-leave").hidden = true;
  note.hidden = false;
  delete note.dataset.shown;
  note.textContent = "Drawing the chart…";
  setTimeout(() => void document.fonts.ready.then(() => run === noteRun && (note.dataset.shown = "")), 800);
}
function pictureFailed(/** @type {string} */ why) {
  document.body.toggleAttribute("data-picture", true);
  $("picture-open").hidden = true;
  noteRun++;
  $("picture-note").hidden = false;
  $("picture-note").dataset.shown = "";
  $("picture-note").textContent = why;
  $("picture-leave").hidden = false;
  document.title = why;
}
function leavePicture() {
  picturing = false;
  pictureRun++;
  document.body.removeAttribute("data-picture");
  if (/[#&]image=/.test(location.hash)) history.replaceState(history.state, "", `${location.pathname}${location.search}`);
  presetsEdit.setFull(false);
  presetsEdit.layOutAlone(false);
  clearLink();
}
$("picture-leave").addEventListener("click", () => {
  leavePicture();
  document.title = "Bitview Studio";
  refresh();
});
$("picture-open").addEventListener("click", (event) => {
  if (event.metaKey || event.ctrlKey || event.shiftKey || event.button !== 0) return;
  event.preventDefault();
  leavePicture();
});
window.addEventListener("keydown", (event) => document.body.hasAttribute("data-picture") && event.stopImmediatePropagation(), true);

async function settle(ms = 60_000) {
  const end = performance.now() + ms, left = () => Math.max(0, end - performance.now()), LATE = Symbol(), late = () => new Promise((done) => setTimeout(() => done(LATE), left()));
  if ((await Promise.race([navigating, late()])) === LATE || (await Promise.race([document.fonts.ready, late()])) === LATE) throw new Error("The chart is still loading");
  while (home || !chart.layers.length || !settled() || !heatmaps.heatIn()) {
    if (home) throw new Error("No chart is open (Studio shows Home)");
    if (!chart.layers.length) throw new Error("The chart has no layers");
    if (!left()) throw new Error("The chart is still loading");
    // (Woken by the draw, else at the time given: no timer chain, which a hidden tab would slow to one a minute.)
    await new Promise((done) => {
      const wake = () => (clearTimeout(timer), drawnWaiters.delete(wake), heatmaps.heatWaiters.delete(wake), done(undefined));
      const timer = setTimeout(wake, left());
      drawnWaiters.add(wake);
      heatmaps.heatWaiters.add(wake);
      const undrawn = settled() && !document.hidden ? heatmaps.shownHeats().filter((look) => !look.drawn && !look.undrawable) : [];
      if (undrawn.length) requestAnimationFrame(() => requestAnimationFrame(() => (undrawn.forEach((look) => (look.undrawable = !look.drawn)), wake())));
    });
  }
  // (Its reveal, the page's crossfade to it, its legends' fades, the chart's own wipe: each at its end now. A script
  // asking wants the chart as it stays, and a tab drawing no frames (out of sight, a pane not shown) would never
  // move them. Those that don't end, a live dot's, go on. Again after a frame: what an end started.)
  const ended = () => {
    for (const motion of document.getAnimations()) if (motion.playState === "running" && Number.isFinite(Number(motion.effect?.getComputedTiming().endTime))) motion.finish();
    endReveal();
  };
  ended();
  if (!document.hidden) await new Promise((done) => (requestAnimationFrame(() => done(undefined)), setTimeout(done, Math.min(100, left()))));
  ended();
  if (!heatmaps.heatIn()) return settle(left());
}
/** @type {{ wanted: string, url: string } | null} */
let picture = null;
const bounded = (/** @type {unknown} */ value, /** @type {string} */ name, /** @type {number} */ low, /** @type {number} */ high) => {
  if (typeof value !== "number" || !(value >= low && value <= high)) throw new Error(`${name}: ${low} to ${high}`);
  return value;
};
const studio = Object.freeze({
  open(/** @type {object | string} */ sketch) {
    const run = ++linkRun, at = navigations + 1;
    grid.closeDialogs();
    if (document.body.hasAttribute("data-picture")) leavePicture();
    return navigate((async () => {
      const doc = await linked(typeof sketch === "string" ? sketch : JSON.stringify(sketch), null, run, at);
      if (!doc) throw new Error(linkError);
      if (run !== linkRun || navigations > at) throw new Error("Another chart was opened meanwhile");
      const shown = showLinked(doc);
      await until(() => chart === shown && !home, 5000);
    })());
  },
  async ready(ms = 60_000) {
    await settle(ms);
    if (!resolved?.times.length) {
      const why = chart.layers.flatMap((layer) => ((error) => (error ? [`${layer.letter}: ${error}`] : []))(resolved?.results.get(layer.id)?.error));
      throw new Error(`The chart has no data${why.length ? ` (${why.join("; ")})` : ""}`);
    }
  },
  async png(/** @type {{ width?: number, height?: number, scale?: number, type?: string, quality?: number, part?: number, partSize?: number }} */ options = {}) {
    const width = Math.round(bounded(options.width ?? 1200, "width", 320, 4000)), height = Math.round(bounded(options.height ?? 760, "height", 240, 4000));
    const scale = options.scale ?? 1;
    if (scale !== 1 && scale !== 2) throw new Error("scale: 1 or 2");
    const type = options.type ?? "image/png", quality = bounded(options.quality ?? 0.9, "quality", 0.01, 1), partSize = Math.round(bounded(options.partSize ?? 45_000, "partSize", 1000, 1e8));
    if (!["image/png", "image/webp", "image/jpeg"].includes(type)) throw new Error('type: "image/png", "image/webp" or "image/jpeg"');
    await studio.ready();
    if (await grid.freshThumb(chart, true)) saveThumbsSoon();
    const wanted = JSON.stringify([chart.id, width, height, scale, type, quality, partSize]);
    if (options.part !== undefined && options.part !== 0) {
      if (picture?.wanted !== wanted) throw new Error("Read part 0 first, of this chart, with the same options");
    } else {
      const at = navigations, url = (await chartImage({ width, height, scale }, true)).toDataURL(type, quality);
      if (at !== navigations) throw new Error("Another chart was opened meanwhile");
      picture = { wanted, url };
    }
    if (options.part === undefined) return picture.url;
    const { url } = picture, mime = url.slice(5, url.indexOf(";")), base64 = url.slice(url.indexOf(",") + 1), parts = Math.ceil(base64.length / partSize);
    if (!Number.isInteger(options.part) || /** @type {number} */ (options.part) < 0 || /** @type {number} */ (options.part) >= parts) throw new Error(`part: 0 to ${parts - 1}`);
    const part = /** @type {number} */ (options.part);
    return { part, parts, mime, base64: base64.slice(part * partSize, (part + 1) * partSize) };
  },
  chart: () => structuredClone(menus.toSketch(chart)),
  async errors() {
    await settle();
    return Promise.all(chart.layers.flatMap((layer) => {
      const result = resolved?.results.get(layer.id), error = result?.error ? result.detail ?? result.error : fromUnfit(chart, layer);
      if (!error) return [];
      const closest = !layer.series ? null : result?.missing ? inspector.closestSeries(layer.series) : result?.detail === BY_BLOCK ? Promise.resolve(indexCatalog()).then(() => dailyOf(/** @type {string} */ (layer.series)) ?? inspector.closestSeries(/** @type {string} */ (layer.series))) : null;
      return [closest ? closest.then((name) => ({ layer: layer.letter, error, ...(name ? { closest: name } : {}) })) : { layer: layer.letter, error }];
    }));
  },
  async search(/** @type {string} */ text, count = 10) {
    const most = Math.round(bounded(count, "count", 1, 50));
    indexCatalog();
    return (await command.findSeries(String(text), most + 20)).filter((name) => !gone.has(name) && !byBlock.has(name)).slice(0, most);
  },
});
Object.defineProperty(window, "studio", { value: studio });

const homeScroll = () => /** @type {HTMLElement} */ (document.querySelector(".home-scroll"));
/**
 * Each shelf's scroll, as it last scrolled (one not in it, new: at its start). Where the page is, read and put back
 * by it: a shelf out of sight, its scroll read or written, is laid out for it alone, shelf by shelf (back from a
 * chart: 64 ms at a phone's 4× slower CPU, to write where they were already).
 * @type {WeakMap<HTMLElement, number>}
 */
const shelfScrolls = new WeakMap();
const shelfRows = () => all$(".shelf-row:not([hidden] *)", $("home"));
const galleryPlace = () => ({ top: homeScroll().scrollTop, rows: Object.fromEntries(shelfRows().map((row) => [shelfKey(row), shelfScrolls.get(row) ?? 0])), shown: galleryShown });
const shelfKey = (/** @type {HTMLElement} */ row) => ((title) => title?.getAttribute("data-place") ?? title?.firstChild?.textContent ?? "")(row.closest(".shelf")?.querySelector(".shelf-title"));
/** @param {{ top: number, rows: Record<string, number> }} place */
function placeGallery(place) {
  for (const row of shelfRows()) {
    const left = place.rows[shelfKey(row)] ?? 0;
    if (left !== (shelfScrolls.get(row) ?? 0)) (row.scrollLeft = left), shelfScrolls.set(row, row.scrollLeft);
  }
  homeScroll().scrollTop = place.top;
}

const quickWorks = () => command.keyboard() && !phone.matches;
phone.addEventListener("change", () => phone.matches && quickHide());
const canLook = (/** @type {any} */ item) => Boolean(lookableAt(item));
/** @returns {{ doc: Chart, i: number } | null} */
function lookableAt(/** @type {any} */ item) {
  const chartKey = item?.kind === "chart" ? item.key : /^chart:/.test(String(item?.key ?? "")) ? String(item.key).slice(6) : null;
  if (chartKey !== null) return ((doc) => (doc ? { doc, i: doc.preset !== undefined ? presetIndex(doc) : -1 } : null))(chartById(chartKey));
  const at = /^(?:preset|recommended):(\d+)$/.exec(String(item?.key ?? "")), i = at ? Number(at[1]) : -1;
  return presets?.[i] ? { doc: presetDoc(i), i } : null;
}
const QUICK_PICTURE = { width: 1000, height: 560, points: 700 };
/** @type {Map<string, Promise<{ drawn: Preview, values: Map<string, string>, bands: Map<string, string> } | null>>} */
const quickPictures = new Map();
function quickPicture(/** @type {Chart} */ doc) {
  const key = `${doc.id}:${grid.previewKey(doc)}:${light}`;
  let made = quickPictures.get(key);
  if (made) quickPictures.delete(key);
  else made = (async () => {
    const index = await grid.sampleable(doc, grid.sampleIndex(doc, "week1", QUICK_PICTURE.points)), copy = { ...structuredClone(doc), index };
    const data = await resolve(copy).catch(() => null);
    if (!data || grid.retryFailed(data, index)) return void quickPictures.delete(key), null;
    const [heat, bubbles] = await Promise.all([grid.heatSamples(copy, data, grid.previewBars(doc, index), HEAT_QUICK_COLUMNS), grid.bubblesAt(doc, data, index)]);
    if (!bubbles || key !== `${doc.id}:${grid.previewKey(doc)}:${light}`) return void quickPictures.delete(key), null;
    if (heat.whole === false) quickPictures.delete(key);
    const drawn = grid.previewOf(copy, data, grid.previewBars(doc, index), QUICK_PICTURE, heat, bubbles);
    const values = new Map(copy.layers.flatMap((layer) => {
      const result = data.results.get(layer.id), high = result?.values, low = result?.low, unit = unitFor(layer, result);
      if (layer.type === "heatmap") return ((sample) => {
        const fine = Boolean(sample?.fine?.rows), day = fine ? sample?.fine : sample?.open, measure = measureOf(layer);
        const grid = fine ? logRows(heatmaps.rowsOf(layer)) : HEAT_PICTURE_GRID, cell = day?.rows ? heatmaps.closeCell(day, grid, measure) : null;
        return cell ? [[layer.id, heatmaps.readingLine(heatmaps.cellText(cell, grid, measure))]] : [];
      })(heat.get(layer.id));
      if (!high || result.error) return [];
      const at = lastIn(high, 0, drawn.end ?? high.length - 1);
      if (at < 0) return [];
      return [[layer.id, !low ? bandOf(layer, high[at])?.name ?? format(high[at], unit) : rangeText(layer, high, low, at, unit)]];
    }));
    const bands = new Map(copy.layers.flatMap((layer) => {
      const high = data.results.get(layer.id)?.values;
      if (!high || !named(layer)) return [];
      const at = lastIn(high, 0, drawn.end ?? high.length - 1);
      const band = at >= 0 ? bandOf(layer, high[at]) : null;
      return band ? [[layer.id, cssColor(band.color)]] : [];
    }));
    return { drawn, values, bands };
  })();
  quickPictures.set(key, made);
  for (const [old] of quickPictures) if (quickPictures.size > 12) quickPictures.delete(old);
  return made;
}
function quickShow() {
  if (!quickWorks() || !isOpen($("palette"))) return void quickHide();
  const look = lookableAt(command.palette.items[command.palette.active]), key = look ? JSON.stringify([look.doc.id, grid.previewKey(look.doc), titleOf(look.doc), look.doc.layers.map(labelOf), light]) : "";
  if (quick?.key === key && $("quick").matches(":popover-open")) return;
  quick = { key };
  $("quick-name").textContent = look ? titleOf(look.doc) : "";
  $("quick-where").textContent = !look ? "" : look.i >= 0 ? `Presets › ${placeNames(/** @type {Preset[]} */ (presets)[look.i].p.slice(0, -1)).join(" › ")}` : isPreview(look.doc) ? "A preview" : look.doc.folder ? `Library › ${grid.folderPath(look.doc.folder)}` : "Library";
  $("quick-none").hidden = Boolean(look);
  /** @type {HTMLElement} */ ($("quick").querySelector("[data-quick-open]")).hidden = !look;
  const frame = $("quick-picture");
  frame.hidden = !look;
  if (look) {
    const doc = look.doc, legend = doc.layers.filter((layer) => !layer.hidden).map((layer) => legendEntry(doc, layer, `<b data-quick-value="${escapeHtml(layer.id)}"></b>`)).join("");
    frame.innerHTML = `<div class="tile-legend quick-legend">${legend || (doc.layers.length ? "Every layer is hidden" : "No layers yet")}</div><div class="quick-chart"><span class="tile-plot quick-plot ${doc.layers.length ? "pending" : "blank"}"></span><div class="quick-scales"></div></div><div class="quick-ticks"></div>`;
    if (doc.layers.length) requestAnimationFrame(() => quick?.key === key && void quickPicture(doc).then((made) => {
      if (quick?.key !== key) return;
      const plot = /** @type {HTMLElement} */ (frame.querySelector(".quick-plot"));
      plot.classList.remove("pending");
      if (!made) return void (plot.textContent = "Couldn't be drawn");
      const { drawn, values, bands } = made, { width: W, height: H } = QUICK_PICTURE;
      plot.innerHTML = grid.previewSvg(drawn, `quick-${doc.id}`, QUICK_PICTURE);
      for (const value of all$("[data-quick-value]", frame)) {
        const id = /** @type {string} */ (value.dataset.quickValue), color = bands.get(id);
        value.textContent = values.get(id) ?? "";
        value.classList.toggle("named", Boolean(color));
        if (color) value.style.setProperty("--c", color);
      }
      /** @type {HTMLElement} */ (frame.querySelector(".quick-scales")).innerHTML = (drawn.scales ?? []).map((tag) => `<span class="${tag.side}" style="top: ${((tag.y / H) * 100).toFixed(2)}%">${escapeHtml(tag.text)}</span>`).join("");
      /** @type {HTMLElement} */ (frame.querySelector(".quick-ticks")).innerHTML = (drawn.ticks ?? []).map((tick) => `<span style="left: ${((tick.x / W) * 100).toFixed(2)}%">${escapeHtml(tick.text)}</span>`).join("");
    }));
  }
  grid.arrive($("quick"));
  if (!$("quick").matches(":popover-open")) $("quick").showPopover();
  $("palette-input").focus({ preventScroll: true });
}
function quickHide() {
  if (!quick) return;
  quick = null;
  quickPictures.clear();
  grid.leave($("quick"));
  if (isOpen($("palette"))) $("palette-input").focus({ preventScroll: true });
}
const quickToggle = () => (quick ? quickHide() : quickShow());
$("quick").addEventListener("pointerup", () => requestAnimationFrame(() => isOpen($("palette")) && $("palette-input").focus({ preventScroll: true })));
$("quick").addEventListener("click", (event) => {
  if (closest(event, "[data-quick-open]")) return void (command.palette.pending ? command.palette.pending() : quick?.key && command.choose());
  if (event.target === $("quick") || closest(event, "[data-quick-close]")) quickHide();
});
$("palette-quick").addEventListener("click", () => (quickToggle(), $("palette-input").focus({ preventScroll: true })));
$("palette").addEventListener("cancel", (event) => {
  if (!quick) return;
  event.preventDefault();
  event.stopImmediatePropagation();
  quickHide();
}, { capture: true });
onClosed($("palette"), quickHide);

async function start() {
  arrangedAs = arrangedNow();
  const at = navigations + 1, shared = await fromHash(linkRun, at);
  if (navigations > at) (picturing = false), clearLink();
  const stored = previewsAside(), kept = (/** @type {any} */ doc) => library.charts.find((item) => item.id === doc.id);
  const unsaved = stored.filter((doc) => !kept(doc) || laterAside(doc, /** @type {Chart} */ (kept(doc)).updated)).map((doc) => (kept(doc) ? { ...doc, id: uid() } : doc));
  if (unsaved.length < stored.length || unsaved.some((doc, i) => doc !== stored[i])) unsaved.length ? storage.set(PREVIEW_ASIDE, unsaved) : storage.drop(PREVIEW_ASIDE);
  const setAside = unsaved.flatMap((record) => restore(record).map((doc) => ((/** @type {any} */ (record)?.untouched === true || touched.add(doc)), doc)));
  for (const doc of setAside) previewCharts.push(doc);
  if (shared) {
    const like = !library.charts.includes(shared) && previewLike(shared);
    if (like) lookAs(like, shared);
    chart = like || shared;
    linkWindow = chart.id;
    if (!library.charts.includes(chart)) addPreview(chart), keepAsideOpened(chart);
  }
  chart ??= chartById(library.active) ?? library.charts[0] ?? setAside[0];
  const first = !chart;
  if (first) chart = newDraft();
  const there = shared ? null : history.state, resumed = there?.chart ? chartById(there.chart) : null;
  const asked = !shared && ROUTED && location.pathname !== `${BASE}/` ? location.pathname : "";
  if (asked) await loadPresets().catch(() => {});
  const routed = asked ? routeOf(asked) : null;
  chart = (routed && "preset" in routed ? presetByAddress(routed.preset, routed.window) : resumed) || chart;
  if (first && routed && "preset" in routed) {
    const draft = previewCharts.findIndex((doc) => drafts.has(doc));
    if (draft >= 0) previewCharts.splice(draft, 1);
  }
  if (asked) homeAt = routed && "path" in routed ? { path: routed.path } : presets ? { path: [] } : { top: true };
  if (asked && !routed && !presets) pendingRoute = asked;
  else if (asked && !routed) setTimeout(() => withRest(() => notice(presetsEdit.missedSaid(asked))));
  for (const doc of library.charts.filter((doc) => doc !== chart && emptyYours(doc))) dropEmpty(doc);
  home = !shared && Boolean(asked ? !(routed && "preset" in routed) : there ? there.home : first || prefs.home);
  started = true;
  document.body.toggleAttribute("data-home", home);
  $("home").inert = !home;
  if (!there || asked) history.replaceState({ ...(home ? homeState() : { chart: chart.id }), ...(pendingRoute ? { route: true } : {}) }, "");
  library.active = chart.id;
  rememberViewed(chart.id);
  normalize(chart);
  save();
  if (home || !chart.layers.length) await loadRest().catch((error) => (document.body.toggleAttribute("data-offline", true), Promise.reject(error)));
  refresh(!home);
  if ("launch" in document.documentElement.dataset) document.body.classList.remove("boot");
  document.documentElement.dataset.ready = "";
  if (picturing && shared) showPicture();
  else if (picturing) pictureFailed(linkError || "That link doesn't contain a chart");
  else if (document.body.hasAttribute("data-picture")) leavePicture();
  setTimeout(() => document.body.classList.remove("boot"), 1400);
  withRest(() => command.startFinder());
  const aside = storage.get(UNREAD, []), told = Array.isArray(aside) ? String(hashOf(JSON.stringify(aside))) : "";
  if (Array.isArray(aside) && aside.length && storage.get(`${UNREAD}:told`, "") !== told && storage.set(`${UNREAD}:told`, told)) notice(`${plural(aside.length, "saved chart")} couldn't be read here and ${aside.length === 1 ? "was" : "were"} set aside`, { label: "Download", run: () => menus.download("bitview-charts-kept-aside.json", new Blob([JSON.stringify({ v: 2, charts: aside }, null, 2)], { type: "application/json" })) });
}
navigate(start().finally(() => ((started = true), (document.documentElement.dataset.ready = ""), /** @type {any} */ (window).bitviewLaunch?.ready())));
export {
  acrossUnfit, afterHands, answers, applyPanel, applyTheme, asThen, atWindow, averageFor, bandLit, beltStop, boxing,
  bubbleRing, buildAsked, buildChart, byBlock, byKeys, byTouch, canLook, canTravel, capturing,
  cardsSeen, catalogIndex, catalogIntervals, changesOf, chart, chartAgain, chartById, chartHeight, chartImage, chartNow, chartSans,
  chartShown, chartSize, CHART_TEXT, chartWindow, chooseDrawings, chosen, clearLayerPicks, clearLook, client, closePreview, CLOUD,
  cohortColor, cohortName, cohortShort, cohortWho, colorName, colorPane, commit, comparePeriods, crossMoved, css,
  dataRevision, dateGoes, DEFAULTS, deleteChart, deleteFolder, dialogsShown, discard, dismissPalette, displayed, draft,
  drafts, drawCards, drawGallery, drawingsHidden, drawPresetMarks, drawTrail, drawVersion, drawYours, editChart, edits,
  EMPTY, emptyYours, engine, EXAMPLE, exporting, firstForm, firstLook, fitMargins, flowsAt, FOLDER_ICONS, folderAgain,
  folderMark, folderNow, footPinned, formatName, formulaFit, formulaLine, fromLetters, fromShort, fromSpan, fromUnfit,
  galleryAhead, galleryPlace, galleryShown, galleryTilesAhead, galleryTilesSeen, gesturePointer, goneLayer, goTo,
  goToDate, goToTime, GROUP_OF, GROUPS, grow, handsHeld, HEAT_AGES, HEAT_COHORTS, HEAT_HOLDERS, HEAT_PICTURE_AGG,
  HEAT_PICTURE_GRID, HEAT_PICTURE_ROWS, HEAT_TILE_COLUMNS, heatAt, heatCohort, heatKnown, heldUpright, hideBelt,
  hideDrawings, hideTip, histories, historyFor, holding, home, homeAt, homeScroll, homeScroller, hoverTime,
  importCharts, indexAt, indexCatalog, inLibrary, isDeleted, isOhlc, isPreview, isView, isYours, keepChart, keepDrawing,
  keepSidePresets, keepsLeaving, keepStretch, keptAsEdited, keyAway, keysNow, keyTime, keyValue, labelTyping,
  layerPicks, layersBatch, layoutCurve, layoutOwed, leaveHome, leftView, leftWith, lensAt, letterChip, letterOf,
  library, libraryEdits, libraryStep, light, linkView, listedPreviews, listOwed, LIVE, liveHeight, livePrice,
  loadPresets, lookDoc, lookShown, magnifying, markFor, markPicks, marksOf, measure, measurer, measuring, memberable,
  menuNameOf, minedHalvings, morphFrom, mostBars, MOTION, motionsSized, moving, navBar, navigations, newDraft, nextHalvings,
  normalize, noteUse, notice, ontoData, openChart, openingView, openPalette, openView, ownerOf, paintNavigator,
  onRail, railed, paletteMarksSeen, paletteOnField, paletteOpenings, panelShown, paneShowing, pathResume, pending, pendingRoute, phone,
  pickPreset, picks, placeGallery, placing, pointedLayer, pointerOnChart, prefs, presetLabels, presetMarks, presets,
  presetsFailed, presetsMatching, pressedTile, previewCharts, previews, previousChart, profileColor, profileLit,
  putViews, quick, quickHide, quickShow, quickWorks, railMenu, railMenuKept, reaching, readOnly, readPresetList, recent,
  recentCharts, redoing, redoLast, reduced, refFor, refresh, refsFor, rekey, relayout, relist, remember, removedNames,
  renderBelt, renderCharts, renderLayers, renderPalette, reshape, reshaped, resolved, retryPresets, revision, ROUTED,
  rowSizes, SANS, save, savePrefs, savePreviewsSoon, saveView, say, secondColor, sectionMark, select, selected,
  selectingArea, seriesCount, setSingleKeys, SHARE, sharedTops, SHARES_IMAGES, SHEET_SHARE, shelfRows, shelfScrolls,
  showGuide, showList, showLook, shownDrawings, shownOklch, shownView, showPalette, showRange, showShortcuts, sideLight,
  sidePresets, signalShown, signedSignal, sizeChart, sizeHeld, SLIDERS, SPAN, spanKey, spanParts, spotlight,
  STACKED_ONLY, started, startLabel, startTyping, storage, studioVersion, submenus, subSetups, syncUndo, tabletHead,
  tagLit, takeUp, takeView, THEMES, thumbs, timeAt, timesAt, toast, togglePanel, toggleSidebar, tokensMedia, tokensNow,
  tool, toolKept, touched, track, tree, trendShort, typingIn, undoLast, undoLibrary, undoToast, UNREAD,
  updatePaletteKeys, updateRowChanges, used, viewed, VIEWED, viewMoving, viewOn, viewsNow, viewTimes, widest,
  windowTooLong, within, writeSideList, writing, yourTilesSeen, zeroFit, THUMBS, PRESET_MARKS, validDrawingKey, validChartMark, validThumb, PREVIEWS, validPreview, SHELF, GALLERY_PAGE, SIDE_PRESETS, MARK_VERSION, validLevel, validLines, validMark, validYs, MARK_KINDS, PLOT_NEEDS, THUMB_POINTS, isPath, USED, keepFocus, nodeAt, aheadOf, bezierOf, logRows, placingPath, fullOn,
};
