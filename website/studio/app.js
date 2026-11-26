// @ts-check
import { askRest, command, gestures, grid, heatmaps, inspector, loadRest, menus, pen, presetsEdit, withRest, xkcd } from "/studio/later.js";
import { BitviewClientBase, _endpoint, dateToIndex, indexToDate } from "/modules/bitview-client/core.js";
// (Studio's chart library: bitview-charts.)
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
// (Studio's functions, by what they do, in modules of their own; what runs at its start runs from here, in order. The
// state written from several of its sections, and its own sections' state (library, toast, live, ...), is kept here.)
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

// Responses are kept here alone, as the page uses them (the client's copies, in memory and on disk, would be more).
// (The server that serves the page; the page built on its own names its server.)
export const CLIENT = { baseUrl: location.origin, timeout: 30_000, memCache: false, browserCache: false };
/**
 * The client: core's alone (all a first chart asks) till the rest brings the whole one (see whole).
 * @type {import("/modules/bitview-client/index.js").BitviewClient}
 */
let client = /** @type {any} */ (Object.assign(new BitviewClientBase(CLIENT), {
  /** @param {string} name @param {any} index */
  seriesEndpoint(name, index) { return _endpoint(/** @type {any} */ (this), name, index); },
  /** (The whole client's getSeriesInfo.) @param {string} name @param {any} [options] */
  getSeriesInfo(name, options) { return /** @type {any} */ (this).getJson(`/api/series/${encodeURIComponent(name)}`, options); },
}));
/** The server the page draws from, as its words name it: its host, as written (a prefix too, if it has one). */
const SERVER_HOST = new URL(client.baseUrl).host;
for (const element of document.querySelectorAll(".server-host")) element.textContent = SERVER_HOST;
/** The catalog: the whole client's tree, none till it comes (see whole). @type {any} */
let tree;
/**
 * The whole client, from client.js, in before the rest's other modules run. @param {import("/modules/bitview-client/index.js").BitviewClient} full
 */
export function whole(full) {
  client = full;
  tree = full.series;
  EXAMPLE = `sma(${tree.price.close.name}, 200d)`;
  LIVE = new Map([[tree.price.ohlc.name, usd], [tree.price.close.name, usd], [tree.price.spot.name, usd], [tree.price.satsPerDollar.name, sats]]);
  knowCohorts(full.AGE_RANGE_NAMES);
}
/** The catalog's node at a path (its own keys only: no "constructor" off a prototype), or undefined. @param {string[]} path */
function nodeAt(path) {
  let node = /** @type {any} */ (tree);
  for (const key of path) {
    if (!node || typeof node !== "object" || !Object.hasOwn(node, key)) return undefined;
    node = node[key];
  }
  return node;
}
/**
 * The series at a path, by its current name (charts keep a series' dotted catalog path with its name, so a rename
 * still finds it); nothing if the path is gone. @param {string} path
 */
function nameAt(path) {
  const node = nodeAt(path.split("."));
  return isLeaf(node) ? node.name : null;
}
/**
 * Every name the client knows, to its path: built once, in a worker (else here, in idle slices), at once when needed
 * (see indexCatalog).
 */
let catalogIndex = /** @type {Map<string, string> | null} */ (null);
/** How many series Studio can chart (by date), as the catalog here has them: known once it's indexed. */
let seriesCount = 0;
/** Each name the client knows, to the intervals (by date) it's charted at: indexed with catalogIndex. @type {Map<string, readonly string[]> | null} */
let catalogIntervals = null;
/** The series Bitview has only by block, none by date (Studio charts by date): known once the catalog's indexed. */
const byBlock = new Set();
/** Series the catalog lists under more than one top folder (a folder's place among them isn't theirs: see catalogTops). @type {Set<string>} */
const sharedTops = new Set();
/**
 * The tokens, kept for what they change with (the theme, the device's light or dark, a phone's layout): one asked
 * for after any of those changed, every one the page sets is read again, in one go (the page's style resolved once,
 * not once a token between its writes); light-dark() colors resolved on probes, in hex for the chart and canvases.
 */
const tokens = new Map(), probe = document.head.appendChild(document.createElement("i"));
/** What they were read for: the theme, the device's light or dark, a phone's layout. */
let tokensFor = "";
// (Read on lists of their own, which no one listens to: a list's matches read takes a change for itself
// (Chromium), its listeners never hearing of it.)
const tokensMedia = [matchMedia(MEDIA.light), matchMedia(MEDIA.phone), matchMedia(MEDIA.moreContrast), matchMedia("(pointer: coarse)")];
// (The theme's own colors for the chart, not a high-contrast mode's: the page around it takes those.)
probe.style.forcedColorAdjust = "none";
/** Tokens read in one go: those named, else every one the page's root lists. @param {Iterable<string>} [names] */
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
/** What the page's tokens are read for: its theme and the media they answer to. */
const tokensNow = () => [document.documentElement.dataset.theme, ...tokensMedia.map((media) => media.matches)].join();
/** A token's value (none set: ""). @param {string} name */
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
/** Motion reduced: as asked for (the system's setting, motion's one reading), or for a step shown already (see stepShown). */
const reduced = () => motionReduced() || stepShown;
let stillSteps = 0;
/** A step from the history begun, the system's motion of it played or not (see stepShown). @param {boolean} shown */
function stillStep(shown) {
  const at = ++stillSteps;
  stepShown = shown;
  document.documentElement.toggleAttribute("data-still", shown);
  if (!shown) return;
  // (A motion under way as it comes, or held for its frames (beginOnTime), at its end at once: a sheet let go as the
  // swipe ended, its slide begun over the page the step showed. One that never ends, as it is.)
  for (const motion of document.getAnimations()) if (motion.timeline === document.timeline && Number.isFinite(Number(motion.effect?.getComputedTiming().endTime))) motion.finish();
  // (Over once its work's begun; another step's since, that one's.)
  requestAnimationFrame(() => setTimeout(() => requestAnimationFrame(() => at === stillSteps && ((stepShown = false), document.documentElement.removeAttribute("data-still")))));
}
/** The stylesheet's motion, for what moves from here: its lengths in milliseconds, and its curve. */
const MOTION = { micro: parseFloat(css("--dur-micro")), small: parseFloat(css("--dur-small")), panel: parseFloat(css("--dur-panel")), page: parseFloat(css("--dur-page")), ease: css("--ease"), layout: parseFloat(css("--dur-layout")), easeLayout: css("--ease-layout"), wordsWait: parseFloat(css("--words-wait")), wordsIn: parseFloat(css("--words-in")) };
const phone = matchMedia(MEDIA.phone);
/** Whether the sidebar's list is owed: not written while the layout's a phone's, where it never shows (see renderCharts; here, as charts are listed at the start). */
let listOwed = false;
/** A layer row's value and change sizes, as the stylesheet sets them now (a phone's are larger): their room measured at them. */
const readRowSizes = () => ({ value: parseFloat(css("--row-value-size")), change: parseFloat(css("--row-change-size")) });
let rowSizes = readRowSizes();
// (Changed, the rows written again at them: a cloud's room is set as it's written.)
phone.addEventListener("change", () => ((rowSizes = readRowSizes()), chart && renderLayers()));
// (The sidebar's list as the rail, or whole in a phone's sheet: see onRail.)
phone.addEventListener("change", () => renderCharts());
// (Never scrolled above its top. WebKit can leave it there, an installed app turned on an iPhone Duo's inner screen
// keeping the upright status bar's room (iOS 27.2): what's drawn and what's touched then part by that much, a tap
// landing on what's above it. Put back, the page is what WebKit leaves it, every tap where it lands.)
addEventListener("scroll", () => scrollY < 0 && scrollTo(0, 0));
/** A phone held upright (its sheet at half height first). */
const heldUpright = matchMedia("not all and (max-height: 500px) and (min-width: 568px)");
/** Whether a phone's sheet is whole (the page behind it dimmed, out of reach): upright, dragged up; on its side, a column beside the chart, never. */
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
/** Studio's version: the Bitview server's that serves it (it comes with it), as it says it (a release's, `v0.12.2`; a build of a commit, next's, `a1b2c3d`), once it's said (else ""). */
let studioVersion = "";
withRest(() => client.getVersion().then((said) => {
  studioVersion = typeof said === "string" ? said : "";
  $("home-version").textContent = studioVersion;
}).catch(() => {}));
for (const element of all$("[data-icon]")) element.insertAdjacentHTML("afterbegin", ICONS[element.dataset.icon] ?? "");

// ── Vocabulary, as the page's state has it (the rest, stateless, in the leaf modules) ─

/** An index's time (unix seconds, where its bucket starts) and the index a time falls in, by the client (a time before the first is in the first). */
const timeAt = (/** @type {string} */ index, /** @type {number} */ i) => indexToDate(/** @type {"day1"} */ (index), i).getTime() / 1000;
/** Each index's times from its first, as far as they've been asked for: the same for every series at it, made once. @type {Map<string, number[]>} */
const indexTimes = new Map();
/** An index's times from `start`, `count` of them (a series' as loaded). */
function timesAt(/** @type {string} */ index, /** @type {number} */ start, /** @type {number} */ count) {
  const times = indexTimes.get(index) ?? [];
  indexTimes.set(index, times);
  for (let i = times.length; i < start + count; i++) times.push(timeAt(index, i));
  return times.slice(start, start + count);
}
const indexAt = (/** @type {string} */ index, /** @type {number} */ time) => dateToIndex(/** @type {"day1"} */ (index), new Date(Math.max(time, timeAt(index, 0)) * 1000));
/**
 * A color's lightness, chroma and hue as the theme shows it. The palette's (see ofPalette) on the dark as they are; on
 * the light, their lightness turned over along one curve: white as black and black as white, Bitview's band (0.5 to
 * 0.75) narrowed and raised (0.64 to 0.58), so yellow, amber and lime deepen enough to read on white and indigo, violet
 * lighten a little, each still itself; the warm hues a little less deep than the curve (see warmLift), else they'd go
 * brown. Any other color as given, in either theme. (The page's ink is its own: white or black.)
 */
let light = false;
function shownOklch(/** @type {string} */ color) {
  const [l, c, h] = oklchOf(color);
  return /** @type {[number, number, number]} */ (light && !pageInk(color) && ofPalette(color) ? [onCurve(TONE, l) + warmLift(c, h), c, h] : [l, c, h]);
}
/**
 * Color names, each at its usual color: the xkcd survey's, the names people use (filtered: see its module). A color
 * takes the nearest one's name.
 * @type {{ name: string, l: number, c: number, h: number }[] | null}
 */
let names = null;
const namesAt = () => (names ??= xkcd.default.map(([name, hex]) => { const [l, c, h] = hexToOklch(hex); return { name, l, c, h }; }));
/** Bitview's colors by their own names, as links, files and the guide write them (the page's ink: white or black). */
const ownName = (/** @type {string} */ key) => (key === "ink" ? (light ? "Black" : "White") : capitalized(key));
/** Each color's name as found (by the theme it's shown in: see colorName), a few hundred kept. */
const colorNames = new Map();
/**
 * A color's name: Bitview's own as they're named, any other by the nearest familiar name to it as shown (each found
 * once: a familiar name's every one weighed, a sheet's colors each time it's written, a phone's engine without a JIT).
 */
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
/** A second color, given one the first time (a trend's, a cloud's): a cloud's second layer's, else one apart from its first. @param {Layer} layer */
const secondColor = (layer) => ((other) => (other && other !== layer.color ? other : layer.color === "red" ? "green" : "red"))(layer.type === "cloud" ? ownerOf(layer)?.layers.find((item) => item.id === layer.from?.[1])?.color : undefined);
/**
 * A layer's look before its last shape change and as that shape left it (its colors, whose colors it takes, colored
 * by its levels or its gradient), whether it was two-sided then (see twoSided), this session (not kept with it), by its chart's
 * id and its own (a chart's copy has its layers' ids).
 * @type {Map<string, { to: string, kind: boolean, before: Record<string, any>, after: Record<string, any> }>}
 */
const looksLeft = new Map();
/**
 * A layer's shape changed (its type, a heatmap's measure, by `edit`): what the shape it leaves took or set (two
 * sides' green and red, colors from another layer or by its levels it drops), unchanged since, back as it was
 * before it (a color picked since, kept); one of another kind starts as its kind does (kindLook: two-sided green
 * over red, as the website draws a baseline), unless what came back is its
 * own look of that kind. @param {Layer} layer @param {(layer: Layer) => void} edit
 */
const reshape = (layer, edit) => {
  const from = shapeKey(layer), kind = twoSided(layer), next = /** @type {Layer} */ ({ ...layer });
  edit(next);
  if (shapeKey(next) === from) return void edit(layer);
  // (Its memory the shape it's left's own: one of another (an undo since) holds nothing.)
  const look = /** @type {Record<string, any>} */ (layer), key = `${chart?.id}:${layer.id}`, was = looksLeft.get(key), back = was?.to === from;
  if (back) for (const key of LOOK_KEYS) if (look[key] === was.after[key]) was.before[key] === undefined ? delete look[key] : (look[key] = was.before[key]);
  const before = Object.fromEntries(LOOK_KEYS.map((key) => [key, look[key]]));
  if (twoSided(next) !== kind && !(back && was.kind === twoSided(next))) kindLook(layer, next);
  edit(layer);
  const type = layer.type;
  // (As it'll be once normalized: what this type can't have, gone.)
  const after = Object.fromEntries(LOOK_KEYS.map((key) => [key, look[key]]));
  if (!COLORED_FROM.has(type)) delete after.colorFrom;
  if (!BY_LEVELS.has(type)) delete after.byLevels;
  // (Kept for the chart's own layer alone: a copy's, ⌘K's look at it, keeps nothing.)
  // (The kind its look before was: the one it came back to, else the one it left.)
  if (chart?.layers.includes(layer)) looksLeft.set(key, { to: shapeKey(layer), kind: back ? was.kind : kind, before, after });
  // (Colored by its levels again: one added meanwhile, its color.)
  if (layer.byLevels && BY_LEVELS.has(type)) inspector.colorLevels(layer);
};
/** Whether a signal shows: itself shown, on a layer there and shown (hidden with it, else). @param {Layer} layer @param {Chart} [doc] */
const signalShown = (layer, doc = chart) => layer.type === "signal" && !layer.hidden && ((on) => Boolean(on && !on.hidden))(doc.layers.find((item) => item.id === layer.ref));
/**
 * The layer a new signal is marked on: the first its formula reads that can be (by its letter, else by the series it
 * names, in its order), else the first that can be; of those, a shown one before a hidden one (its marks seen).
 * @param {Chart} doc @param {string} formula @returns {Layer | undefined}
 */
const markFor = (doc, formula) => {
  const can = markable(doc).sort((a, b) => Number(Boolean(a.hidden)) - Number(Boolean(b.hidden)));
  try {
    const node = parse(formula), by = (/** @type {Set<string>} */ read, /** @type {"letter" | "series"} */ key) => ((found) => found.find((layer) => !layer.hidden) ?? found[0])([...read].flatMap((name) => can.filter((layer) => layer[key] === name)));
    return by(refsOf(node), "letter") ?? by(refsOf(node, "series"), "series") ?? can[0];
  } catch {
    return can[0];
  }
};
/** What can flip a trend: a layer of one value (not made from layers), not one of those it's made from. @param {Chart} doc @param {Layer} trend */
const refsFor = (doc, trend) => doc.layers.filter((layer) => !layer.from && !placedOn(layer) && !heatLike(layer) && !trend.from?.includes(layer.id) && !readsOf(layer, doc).has(trend.letter));
/** A trend's reference by default: a price on its scale (candles, a series named so), else the first of those there; none there, none. @param {Chart} doc @param {Layer} trend */
const refFor = (doc, trend) => ((refs) => (refs.find((layer) => layer.type === "candles" || /^price/.test(layer.series ?? "")) ?? refs[0])?.id)(refsFor(doc, trend).filter((layer) => oneScale(layer, trend)));
/** An unnamed formula's line (its name is its formula): what it is, in what once known. @param {Layer} layer */
const formulaLine = (layer) => (layer.type === "signal" ? signalLine(layer) : ((result) => (result?.values ? `Formula · ${UNIT_NAMES[unitFor(layer, result)] ?? "Number"}` : "Formula"))(resolved?.results.get(layer.id)));
/**
 * Where a signal changes state, from one known (gaps passed over): above 0 it's up (1), below down (-1), at 0 neither.
 * Each change says the state it turns to (`to`), the one it ends (`was`), and where that stretch began with a change of
 * its own (`since`; -1: it was so from the first value known). @param {ArrayLike<number> | undefined} values
 * @returns {{ at: number, to: number, was: number, since: number }[]}
 */
const changesOf = (values) => {
  // (Read once a series: a signal's marks, its results and its times ask the same. Not one the live price writes in.)
  const kept = values && changesKept.get(values);
  if (kept) return kept;
  const out = [];
  let was = NaN, since = -1;
  for (let i = 0; i < (values?.length ?? 0); i++) {
    const value = /** @type {ArrayLike<number>} */ (values)[i];
    // (None, or an infinity (a division by 0): a gap, nothing turned.)
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
/**
 * Where a signal's marks are: each change to up or down (a condition's: where it turns true), and with its results,
 * each stretch ending off with one (its dot): counted, listed and stepped through as they're drawn. Its layer's
 * values, the chart's as drawn unless given (⌘K's preview, its own).
 * @param {Layer} layer @param {ArrayLike<number> | undefined} values @param {Result | null | undefined} [reference] (null: none)
 */
const marksOf = (layer, values, reference = resolved?.results.get(layer.ref ?? "")) => {
  const own = reference?.values;
  // (As markSignals draws them: only where its layer has a value (above 0 on a log scale).)
  const target = chart?.layers.find((item) => item.id === layer.ref), log = target ? modeOf(chart, target) === 1 : false;
  const drawn = (/** @type {number} */ i) => own != null && Number.isFinite(own[i]) && (!log || own[i] > 0);
  // (Shaded, its turns; with its results, their dots on its layer where each stretch ends.)
  if (layer.place === "shade") return changesOf(values).flatMap((change) => (change.to || (layer.results && own && drawn(change.at) && stretchResult(change, reference) !== undefined) ? [change.at] : []));
  // (Its layer without values (not loaded, failed): none drawn.)
  if (!own) return [];
  return changesOf(values).flatMap((change) => (drawn(change.at) && (change.to || (layer.results && stretchResult(change, reference) !== undefined)) ? [change.at] : []));
};
/** A signal's stretches' results, compounded: what following it made, by its layer (none, none). @param {Layer} layer */
const resultsOf = (layer) => {
  const values = resolved?.results.get(layer.id)?.values, own = resolved?.results.get(layer.ref ?? "");
  if (!values || !own?.values) return undefined;
  const results = changesOf(values).flatMap((change) => ((result) => (result === undefined ? [] : [result]))(stretchResult(change, own)));
  // (A stretch losing all, or more (down, its layer more than doubled), leaves nothing: none to go on from.)
  return results.length ? results.reduce((total, result) => total * Math.max(0, 1 + result), 1) - 1 : undefined;
};
/** A signal, said: the layer its marks are on, how many there are (with its results, what they made). @param {Layer} layer */
const signalLine = (layer) => ((on, values) => (!on ? "Pick a layer to mark it on" : on.hidden ? `Hidden while ${labelOf(on)} is hidden` : `Signal on ${labelOf(on)}${values ? ` · ${plural(marksOf(layer, values).length, "time")}${((total) => (total === undefined ? "" : ` · ${resultText(total)} in all`))(layer.results ? resultsOf(layer) : undefined)}${fineDrawn() ? " in the loaded range" : ""}${layer.results && resolved?.results.get(on.id)?.measurement ? " · Returns from raw values" : ""}` : ""}`))(chart.layers.find((item) => item.id === layer.ref), resolved?.results.get(layer.id)?.values);
/**
 * (Here, before the library's read: its profiles normalized as it is, see normalize.)
 * The URPD's cohorts (`GET /api/urpd`), named as the presets name them (see words.js): each its id, its name as
 * shown, its short name (where its list says what it is) and its name in titles. All coins, then each complementary
 * pair (the terms, under 4 months and at least 4, under 6 and at least 6), as holderName names them; then the age
 * bands from the youngest, each read from its key (see dimensionBand: "1 day to under 1 week old"), short as the
 * client's (an abbreviation its key's reader doesn't make: "1d–1w"); one it can't read, left out (the presets
 * report it: the same bands, see ageBand in presets.js).
 */
/** @typedef {{ id: string, shown: string, brief: string, who: string, age: boolean }} HeatCohort */
/** (None till the whole client names the age bands: see whole, and knowCohorts.) @type {HeatCohort[]} */
let HEAT_COHORTS = [];
/** @type {Map<string, HeatCohort>} */
let HEAT_BY_ID = new Map();
/** A URPD cohort, by its id (one Bitview doesn't have, a link's typo or one renamed: undefined). */
const heatCohort = (/** @type {string} */ id) => HEAT_BY_ID.get(id);
/** Whether Bitview has a heatmap's holder group (see heatCohort). @param {Layer} layer */
const heatKnown = (layer) => (layer.type === "profile" ? (layer.profile ?? []).every((id) => heatCohort(id)) : Boolean(heatCohort(layer.heatmap ?? "all")));
/** The URPD's holder groups (all coins and the age cutoffs), then its age bands, the youngest first. */
let HEAT_HOLDERS = /** @type {string[]} */ ([]), HEAT_AGES = /** @type {string[]} */ ([]);
/** The cohorts, by the client's age bands (see whole). @param {Record<string, { id: string, short: string }>} ageNames */
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
/**
 * Each holder group's color, one palette wherever it's drawn (see palette.js, the presets' too): its own (all coins
 * green, short-term holders orange, long-term blue, the age cutoffs around them; the URPD's age_under_4m is the
 * tree's under4m); the age bands around the wheel from the youngest, as the presets compare them. @param {string} id
 */
const cohortColor = (id) => holderColor(id.replace(/^age_(under|over)_/, "$1")) ?? ((at) => (at < 0 ? "gray" : wheelColor(at, HEAT_AGES.length)))(HEAT_AGES.indexOf(id));
/** A profile's color for one of its holder groups: its own, else the group's (see cohortColor). @param {Layer} layer @param {string} cohort */
const profileColor = (layer, cohort) => (layer.cohortColors && Object.hasOwn(layer.cohortColors, cohort) ? layer.cohortColors[cohort] : cohortColor(cohort));
/** A cohort's name (one not listed: its id). */
const cohortName = (/** @type {string} */ id) => heatCohort(id)?.shown ?? id;
/** A cohort's name in titles, whose coins they are ("Short-term holders", "Coins 1 day to under 1 week old"; one not listed: its id). */
const cohortWho = (/** @type {string} */ id) => heatCohort(id)?.who ?? id;
/** A holder group's short name, where its list says what it is (a profile's, under Holders or Age bands: "Short-term", "1h–1d"). @param {string} id */
const cohortShort = (id) => heatCohort(id)?.brief ?? id;
/**
 * Whether a layer is a running total: a series that is one (its info's accumulation, `cumulative`: read before
 * it's drawn, see resolve), or a formula of them through affine arithmetic (their sums and differences, by
 * constants), each layer it reads followed once.
 * @returns {boolean}
 */
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
      // (One that can't be read has no role: its own error says why, as it's evaluated.)
    }
    visiting.delete(item.id);
    roles.set(item.id, value);
    return value;
  };
  return role(layer) === "total";
}
/** One visible running total includes its origin; explicit fitting and stacks keep their own rule. */
const zeroFit = (/** @type {Chart} */ doc, /** @type {Layer} */ layer, /** @type {number} */ mode, /** @type {boolean} */ stacked) => mode === 0 && (stacked || layer.fit === "zero" || (automaticZero(doc, layer, stacked) && cumulativeLayer(doc, layer)));
/** Whether a signal has a down state (below 0) in what's drawn: its second color, the down one's, then offered. @param {Layer} layer */
const signedSignal = (layer) => layer.type === "signal" && Boolean(resolved?.results.get(layer.id)?.values?.some((/** @type {number} */ value) => value < 0));
/** A layer's letter, its badge wherever it shows (the panel, ⌘K's rows, its previews): in its color. @param {Layer} layer */
/** `tip`: its name as its tip, where the letter stands alone for it (several layers' heads). */
/** Each letter's tone (see letterChip), by its color, the theme and the tokens. @type {Map<string, string>} */
const letterTones = new Map();
function letterChip(layer, tip = false) {
  const colored = (layer.colorFrom && ownerOf(layer)?.layers.find((other) => other.id === layer.colorFrom)) || layer;
  const color = ink(colored.color), key = `${color}|${light}|${tokensNow()}`;
  // (Its letter in a tone of it that reads on its own tint, 4.5:1, on each ground it's on: the card, a raised
  // surface, a lit row on it; the tint and edge its color. Found once a color and theme: a layer's row and its
  // settings write it each time.)
  let tone = letterTones.get(key);
  if (tone === undefined) {
    const grounds = [css("--card"), css("--raise"), laidOver(css("--raise"), light ? "#000000" : "#ffffff", 0.07)];
    tone = /** @type {string} */ (grounds.reduce((each, ground) => wordsOn(each, laidOver(ground, color, 0.13)), color));
    if (letterTones.size > 500) letterTones.clear();
    letterTones.set(key, tone);
  }
  return `<span class="letter" style="--c:${escapeHtml(color)};--t:${escapeHtml(tone)}"${tip ? ` data-tip="${escapeHtml(labelOf(layer))}"` : ""}>${escapeHtml(layer.letter)}</span>`;
}
/** A layer's format by name: the one chosen, or "Automatic" with its unit's. */
const formatName = (layer, result) => (layer.format ? FORMATS.find(([format]) => format === layer.format)?.[1] ?? "Number" : `Automatic (${UNIT_NAMES[unitFor({ ...layer, format: undefined }, result)] ?? "Number"})`);
/** Whether a formula's calendar-total differences (flow) can be read at an interval (see flowFits). @param {string} formula @param {string} index */
const flowsAt = (formula, index) => windowsOf(formula).every(({ days, flow }) => !flow || flowFits(days, index));
/**
 * Why a window gives nothing: more than a fine interval loads (`beyond` the timeline), naming where it fits ("Needs
 * 4h+"); else longer than the data (a value a span back, `shift`, further back than all of history is, anywhere).
 * `span` as written ("4y"), or in points.
 */
const windowTooLong = (/** @type {string} */ f, /** @type {string} */ span, /** @type {number} */ days, /** @type {string} */ index, /** @type {boolean} */ beyond, /** @type {boolean} */ shift) =>
  beyond && FINE.has(index) && !(shift && days > (Date.now() / 1000 - timeAt("day1", 0)) / 86400)
    ? failing(`${f} over ${span}: more than ${INTERVAL_LABEL[index]} loads; ${INTERVAL_LABEL[fitsFrom(days, index)]} and up hold it`, `Needs ${INTERVAL_LABEL[fitsFrom(days, index)]}+`)
    : new Error(`${f}: ${span} is longer than the data`);
/** One pane's average at an interval: shared by choices, drawing, labels and live updates.
 * @param {Chart} doc @param {Pane | undefined} pane @param {string} [span]
 */
function averageFor(doc, pane, span = pane?.average ?? "") {
  if (!pane || !Object.hasOwn(AVERAGES, span)) return { span: "", active: "", name: "", points: 1, disabled: "" };
  const days = spanDays(span), points = windowPoints(days, doc.index), blocked = averageBlocked(doc, pane);
  const disabled = blocked || (points <= 1 ? `At ${INTERVAL_LABEL[doc.index]}, ${AVERAGES[span]} changes nothing: choose a longer span`
    : FINE.has(doc.index) && points >= CHUNK ? windowTooLong("Average", span, days, doc.index, true, false).message : "");
  return { span, active: disabled ? "" : span, name: disabled ? "" : AVERAGES[span], points, disabled };
}

// ── Units and formatting ──────────────────────────────────────────────────

/**
 * A window over time, as tight as it reads: years shortened (’23), times only within two days, a year both ends
 * share said once ("1 Mar", "12 Jun ’26"; each end stands at its own edge of the navigator's window); its points as
 * the interval drawn (else `index`) has them: a month's, months ("Sep ’18"), a year's, years ("2018").
 * @returns {[string, string] | [string]} the two ends, or one date when they're the same
 */
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

// ── Formulas ──────────────────────────────────────────────────────────────

/** How a new formula on an empty chart starts (a letter till the whole client names the price). */
let EXAMPLE = "sma(A, 200d)";

// ── Charts: the document ──────────────────────────────────────────────────

/** The halvings mined so far: those known, and any since, placed from the tip's time back (a block each ten minutes). */
const minedHalvings = () => [...KNOWN_HALVINGS, ...Array.from({ length: Math.max(0, Math.floor(liveHeight / HALVING_BLOCKS) - KNOWN_HALVINGS.length) }, (_, k) => (liveAt || Date.now() / 1000) - (liveHeight - (KNOWN_HALVINGS.length + k + 1) * HALVING_BLOCKS) * 600)];
/** The halvings to come, the next `count`, estimated (a block each ten minutes from the tip): each its number and time. */
const nextHalvings = (count = 3) => (liveHeight ? Array.from({ length: count }, (_, k) => ((n) => ({ n, time: Date.now() / 1000 + (n * HALVING_BLOCKS - liveHeight) * 600 }))(Math.floor(liveHeight / HALVING_BLOCKS) + 1 + k)) : []);
/**
 * A series' first look: as the presets draw it most (its color, its type: see looksOf); else
 * by what Bitview says it is (its type, from what's known of it: an added one's, asked for; a link's, if it's been
 * asked for already): candles where its type is open-high-low-close; the price itself (under its catalog's price,
 * in any unit) in ink; a value in dollars green, in bitcoin (BTC, sats) orange. A color taken on the chart (`used`)
 * isn't asked for: the next free one is had instead. `asked`, a type given (a link's): the presets' color and base
 * only where it's theirs.
 * @param {string} name @param {Set<string>} used @param {string} [asked] @returns {Partial<Layer>}
 */
function firstLook(name, used, asked) {
  const type = known.get(name)?.type ?? "", unit = type ? unitOf(type) : units.get(name), { look, told } = seriesLook(name, asked);
  // (A baseline as every one starts: green above its base, red below, the presets' shades of them said so.)
  // (The catalog, indexed the rest of the way only when it's asked: the presets' own looks come first.)
  const color = told?.color ?? (look === "baseline" ? "green" : (indexCatalog(), catalogIndex?.get(name))?.startsWith("price.") ? "ink" : unit === "usd" ? "green" : unit === "btc" || unit === "sats" ? "orange" : "");
  // (A baseline's green is every baseline's: taken already, still its own.)
  return { ...firstForm(name, asked), ...(color && (!used.has(color) || (look === "baseline" && color === "green")) ? { color } : {}) };
}
/** A series' first type, and the presets' look of it where that type's theirs. @param {string} name @param {string} [asked] */
const seriesLook = (name, asked) => {
  const own = presetLooks.get(name), ownType = own?.type ?? (isOhlc(name) ? "candles" : "line"), look = asked ?? ownType;
  return { look, told: look === ownType ? own : undefined };
};
/** A series' first type, base and colors (the price's candles, one): its first look's, its color aside. @param {string} name @param {string} [asked] @returns {Partial<Layer>} */
const firstForm = (name, asked) => (({ look, told }) => ({ ...(look !== "line" ? { type: look } : {}), ...(told?.base ? { base: told.base } : {}), ...(told?.colors ? { colors: told.colors } : {}) }))(seriesLook(name, asked));
/** Whether Bitview says a series is open-high-low-close (its type, as known: asked for already). */
const isOhlc = (/** @type {string} */ name) => /^OHLC/.test(known.get(name)?.type ?? "");
/** Layers removed this session, by chart and letter: a formula still naming one says what it was. */
const removedNames = new Map();
/** Why a letter names no layer, where one was removed: "Realized price (B) was removed". */
const goneLayer = (/** @type {Chart} */ doc, /** @type {string} */ letter) => ((name) => (name ? `${name} (${letter}) was removed` : null))(removedNames.get(`${doc.id}:${letter}`));
/** The chart a layer is in: the open one, else the one holding it (a tile's, a preview's; a copy, by its id). @param {Layer} layer */
const ownerOf = (layer) => ((docs) => docs.find((doc) => doc?.layers.includes(layer)) ?? docs.find((doc) => doc?.layers.some((item) => item.id === layer.id)))([chart, ...previewCharts, ...library.charts, ...library.presets]);
/** The layers it's made from, by their letters, in its order. @param {{ from?: string[] }} layer @param {Chart | null} [doc] */
const fromLetters = (layer, doc = ownerOf(/** @type {Layer} */ (layer))) => (layer.from ?? []).flatMap((id) => ((item) => (item ? [item.letter] : []))(doc?.layers.find((one) => one.id === id)));
/** What it's made from, as written: its layers' letters, a cloud's edges as said ("the top"). @param {Layer} layer @param {Chart | null} [doc] */
const fromSpan = (layer, doc = ownerOf(layer)) => (layer.from ?? []).flatMap((id) => (EDGES.includes(id) ? [`the ${id}`] : ((item) => (item ? [item.letter] : []))(doc?.layers.find((one) => one.id === id)))).join(", ");
/**
 * What a layer made from layers lacks, said (none: nothing): a cloud, its two (layers, edges); a trend, two layers,
 * and a third on the chart, outside them, to flip it (one rule: where it's offered, added, drawn).
 * @param {{ type?: string, from?: string[], id?: string, ref?: string }} layer @param {Chart} [doc]
 */
const fromShort = (layer, doc = chart) => {
  const ids = layer.from ?? [], members = (doc?.layers ?? []).filter((other) => ids.includes(other.id)), first = members[0];
  // (Their values read as one: on one scale, what flips a trend too.)
  return ids.length < 2 ? (layer.type === "cloud" ? "Pick what it's between" : "Select two layers") : members.some((other) => !oneScale(other, first)) ? ONE_SCALE : layer.type !== "trend" || doc?.layers.some((other) => !other.from && !placedOn(other) && !ids.includes(other.id) && (!first || oneScale(other, first))) ? "" : layer.ref && doc?.layers.some((other) => other.id === layer.ref) ? FLIP_APART : TREND_FLIP;
};
/**
 * Whether a layer can be one a cloud or a trend (`self`, made; none: to be made) is between: not made from layers, not
 * a signal, not one reading it (a loop), on one scale with `on` (one it'd be with: its first, the other of a cloud's
 * two; none, any). @param {Layer} item
 */
const memberable = (item, /** @type {Layer | null} */ self = null, /** @type {Layer | null} */ on = null) => !item.from && !placedOn(item) && !heatLike(item) && item !== self && !(self && readsOf(item).has(self.letter)) && (!on || oneScale(item, on));
/** A trend's room on the chart: three layers not made from layers on one scale (two it rides, one to flip it), else why not. */
const trendShort = () => {
  const bases = chart.layers.filter((layer) => !layer.from && !placedOn(layer));
  return bases.length < 3 ? "Needs three layers: two to follow and a third that flips it" : bases.some((layer) => bases.filter((other) => oneScale(other, layer)).length >= 3) ? "" : "Needs three layers in one pane, on one side";
};
/**
 * Why a layer made from layers isn't drawn in its pane, if it isn't: it's its layers' values side by side, not a
 * stack's running totals; a cloud's, not their changes from where each starts (a percent or indexed scale: a trend
 * line's one value reads there as any line's); its layers' values one's, on one scale.
 */
const fromUnfit = (/** @type {Chart} */ doc, /** @type {Layer} */ layer) => {
  const pane = doc.panes.find((item) => item.id === layer.pane);
  // (A cloud of its pane's two edges alone reads no values: anywhere.)
  return !layer.from || (edgesOf(layer).length > 1 && !cloudLayers(layer).length) ? "" : offScale(doc, layer) ? ONE_SCALE : drawnInto(doc, pane) ? `Not in ${drawnInto(doc, pane)}` : layer.type === "cloud" && paneMode(pane) >= 2 ? "Not on this scale" : "";
};

/**
 * Why a layer's lines across (its levels, a baseline's base line) aren't drawn, if they aren't: stacked, it's drawn
 * at its running total, not at its values (its price line neither); on a percent or indexed scale, as its changes.
 * @param {Chart} doc @param {Layer} layer
 */
const acrossUnfit = (doc, layer) => ((pane) => (drawnInto(doc, pane) ? `Not drawn in ${drawnInto(doc, pane)}` : paneMode(pane) >= 2 ? "Not drawn on a percent or indexed scale" : ""))(doc.panes.find((item) => item.id === layer.pane));
/**
 * Where a new layer of `unit` goes when no pane is asked for: a pane already showing that unit (not a stack: one
 * added there would be summed into it), else none (a new one). A series added and a formula typed in ⌘K alike.
 * @param {Chart} doc @param {string} unit
 */
const paneShowing = (doc, unit) => doc.panes.find((candidate) => !candidate.stack && doc.layers.some((layer) => layer.pane === candidate.id && unitFor(layer, resolved?.results.get(layer.id)) === unit));
/** Charts started empty in their stead, nothing else there (a first visit, the last one deleted): the person's own from their first edit (a link's preview is kept only by Keep). */
const drafts = new WeakSet();
/**
 * A chart of yours New chart made that's never had anything in it (`blank`, kept till its first content, a reload
 * through): left so, it goes, as a note left empty does. (One emptied since is a chart still: it stays.)
 * @param {Chart} doc
 */
const emptyYours = (doc) => isYours(doc) && doc.blank === true && !doc.layers.length && !doc.title && !doc.drawings?.length;
/** A chart started here and kept by an edit: the undo steps it had once kept (that step undone, a draft again). @type {WeakMap<Chart, number>} */
const keptByEdit = new WeakMap();
const draftChart = () => ((doc) => (drafts.add(doc), doc))(blankChart());
/** Previews changed since they were opened (an edit, a rename): Keep marked, kept aside, offered back once closed. */
const touched = new WeakSet();
/** Previews changed in this tab since it read them (only they're written aside: another tab's, as it left them). */
const asideDirty = new WeakSet();
/** A preview changed: marked edited, kept aside. @param {Chart} doc */
const touchPreview = (doc) => void (touched.add(doc), asideDirty.add(doc));
/**
 * A preview a link opened, kept aside as it is (marked not edited): its link's gone from the address bar once read,
 * so a reload, or a tab the phone put away, finds it again (till it's saved or closed). @param {Chart} doc
 */
const keepAsideOpened = (doc) => void (isPreview(doc) && !drafts.has(doc) && (asideDirty.add(doc), save()));
/** A chart saved or linked under a pool's old name, on its new one. @param {Layer} layer */
function followPool(layer) {
  if (layer.series) layer.series = poolSeries(layer.series);
  if (layer.path) layer.path = poolPath(layer.path);
  if (layer.formula === undefined) return;
  const names = new Map([...seriesNamed(layer.formula)].flatMap((name) => (poolSeries(name) === name ? [] : [[name, poolSeries(name)]])));
  layer.formula = renamed(layer.formula, names);
  if (layer.paths) layer.paths = Object.fromEntries(Object.entries(layer.paths).map(([name, path]) => [poolSeries(name), poolPath(path)]));
}

// ── Windows ───────────────────────────────────────────────────────────────
/**
 * A series' trailing window, by its place in the catalog (its last window step: never by its name, which puts it
 * anywhere, `market_cap_delta_1y_percent`), and the series at each window beside it, in the catalog's order
 * (shortest first); none, null.
 * @param {string} name @returns {{ window: string, at: Map<string, string> } | null}
 */
function seriesWindow(name) {
  const steps = catalogIndex?.get(name)?.split("."), i = steps ? steps.findLastIndex(isWindowKey) : -1;
  if (!steps || i < 0) return null;
  const at = new Map(Object.keys(nodeAt(steps.slice(0, i)) ?? {}).filter(isWindowKey).flatMap((key) => ((other) => (other ? [[key, other]] : []))(nameAt([...steps.slice(0, i), key, ...steps.slice(i + 1)].join(".")))));
  return { window: steps[i], at };
}
/** The series a chart's layers read (their own, their formulas'). @param {Chart} doc */
const seriesOf = (doc) => doc.layers.flatMap((layer) => [...(layer.series ? [layer.series] : []), ...(layer.formula ? seriesNamed(layer.formula) : [])]);
/**
 * @typedef {{ window: string, offered: string[], at: Map<string, Map<string, string>> }} ChartWindow a chart's
 * window, the windows it offers, shortest first, and each of its windowed series at each of them
 */
/** Each set of series' window (see chartWindow), by their names: found once (a switch asks it of a chart several times). @type {Map<string, ChartWindow | null>} */
const windowsFound = new Map();
/**
 * A chart's trailing window: the one every windowed series in it is over, and the windows they all have; none (no
 * windowed series, two windows, one alone), null. The catalog read first (indexed at once, if it isn't yet: a chart
 * opens over the right series), but for a chart no name of which holds a window's key: each windowed name the client
 * has holds its window's, `_<key>` before a `_` or at its end, so no walk to learn a chart has none.
 * @param {Chart} doc @returns {ChartWindow | null}
 */
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
/**
 * A chart over another window (see chartWindow): each series at it, by its place (its path learned again), and the
 * window as its title and labels say it (see said) said anew. A chart with no window, or one it doesn't offer, as it
 * is. @param {Chart} doc @param {string} to
 */
function atWindow(doc, to) {
  const now = chartWindow(doc);
  if (!now || now.window === to || !now.offered.includes(to)) return doc;
  // (A view kept aside (see Lens) at it too: its series, by their own windows, where they're over the chart's.)
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

/** Drops empty panes, keeps every layer on a pane, keeps letters unique, draws layers pane by pane. @param {Chart} doc */
function normalize(doc) {
  // A pane's view kept aside (see Lens), as the one shown is, on its own; none left in it, no lens. The one shown
  // emptied (its last layer gone), the other in its place, and no lens: nothing lost unsaid.
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
  // A series follows its path: renamed in a newer client, it takes the new name; its path is learned (or
  // relearned) from the catalog once that is indexed.
  for (const layer of doc.layers) {
    if (!layer.series) continue;
    const name = layer.path ? nameAt(layer.path) : null;
    if (name) layer.series = name;
    else if (catalogIndex?.has(layer.series)) layer.path = catalogIndex.get(layer.series);
  }
  // So do the series a formula names, their paths kept beside it (as they are till the tree's in: the catalog's take
  // normalizes again).
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
  // Made from layers still there, none made from layers, each once (a cloud short of two, a trend short of its two,
  // say so, as a formula does).
  for (const layer of doc.layers) if (layer.from) layer.from = fromOf(doc, layer.from, layer.type);
  // (A trend's reference one that can flip it, none else (it says so); no shape: a step would draw its flips.)
  for (const layer of doc.layers) if (layer.type === "trend") delete layer.shape;
  // (A profile: its holder groups each once (none: it draws nothing till one's picked), its color its first one's (its
  // letter's, its legend's), each group's own kept only where it isn't the group's, and only for those in it.)
  for (const layer of doc.layers) {
    // (Its groups' colors come with the whole client: a profile draws once it's in.)
    if (layer.type !== "profile" || !tree) continue;
    layer.profile = [...new Set(layer.profile ?? [])];
    // (A group taken out takes its color with it, as a link keeps them.)
    for (const [cohort, color] of Object.entries(layer.cohortColors ?? {})) if (color === cohortColor(cohort) || !layer.profile.includes(cohort)) delete /** @type {Record<string, string>} */ (layer.cohortColors)[cohort];
    if (layer.cohortColors && !Object.keys(layer.cohortColors).length) delete layer.cohortColors;
    if (layer.profile.length) layer.color = profileColor(layer, layer.profile[0]);
  }
  // (Colors from another layer: one there, not itself, for a type that takes them.)
  // (Nor one taking its colors from this one: a pair, each the other's, has nothing to start from.)
  for (const layer of doc.layers) if (layer.colorFrom && (!COLORED_FROM.has(layer.type) || layer.colorFrom === layer.id || !doc.layers.some((other) => other.id === layer.colorFrom) || doc.layers.find((other) => other.id === layer.colorFrom)?.colorFrom === layer.id)) delete layer.colorFrom;
  // (Each layer in a pane there: one whose pane is gone, in the first; before any reads its pane.)
  if (!doc.panes.length) doc.panes.push({ id: uid(), mode: 1 });
  const ids = new Set(doc.panes.map((pane) => pane.id));
  for (const layer of doc.layers) if (!ids.has(layer.pane)) layer.pane = doc.panes[0].id;
  // (A heatmap's gradient its pane's: one kept on the layer, as older charts and links held it, moved there if it's
  // one ready made (the first, none kept; several heatmaps on one pane, the first's other than the first); its own,
  // made by hand, gone (its pane's first, said in the changes).)
  // (A P&L heatmap's colors fixed, its kind's: green over, red under (none stored that could differ from what's drawn).)
  for (const layer of doc.layers) if (layer.type === "heatmap" && signedMeasure(layer)) (layer.color = /** @type {string} */ (kindColor(layer))), delete layer.colorBelow;
  for (const layer of doc.layers) {
    const pane = layer.type === "heatmap" && layer.byGradient && !signedMeasure(layer) && doc.panes.find((item) => item.id === layer.pane), ready = pane && !pane.gradient && HEAT_GRADIENTS.find(([, , stops]) => JSON.stringify(stops) === JSON.stringify(layer.gradient));
    if (pane && ready && ready !== HEAT_GRADIENTS[0]) pane.gradient = ready[0];
  }
  // (A gradient where its type takes one, its way of coloring alone; up to each date, a gradient's.)
  for (const layer of doc.layers) if (layer.gradient && !(COLOR_MODES[layer.type] ?? []).includes("gradient")) delete layer.gradient;
  for (const layer of doc.layers) if (!layer.gradient) delete layer.byGradient, delete layer.upToDate;
  for (const layer of doc.layers) if (layer.byGradient) delete layer.colorFrom, delete layer.byLevels, delete layer.byLayers, delete layer.colors;
  for (const layer of doc.layers) if (layer.byLevels && (!BY_LEVELS.has(layer.type) || !layer.levels?.length || layer.colorFrom)) delete layer.byLevels;
  // (By its layers: a cloud's (of its pane's two edges, none), its colors theirs as they are now. A cloud to an edge
  // of two colors shows one (nothing crosses an edge): kept so, its second kept.)
  for (const layer of doc.layers) if (layer.byLayers && (layer.type !== "cloud" || layer.colorFrom || !cloudLayers(layer).length)) delete layer.byLayers;
  for (const layer of doc.layers) if (layer.byLayers) layersColors(layer, doc);
  for (const layer of doc.layers) if (layer.type === "cloud" && edgesOf(layer).length && !layer.byLayers && !layer.colorFrom && layer.colorBelow !== undefined && layer.colors === undefined) layer.colors = 1;
  // (Candles, unless their two colors are said: one, their own, rising hollow and falling filled (as the price's).)
  for (const layer of doc.layers) if (layer.type === "candles" && layer.colors === undefined && layer.colorBelow === undefined && !layer.colorFrom && !layer.byLevels && !layer.byGradient) layer.colors = 1;
  // (How many colors: bars' one or four, a trend's or a cloud's one with a second kept; the colors picked kept,
  // whatever's used.)
  for (const layer of doc.layers) if (layer.colors !== undefined && (!(layer.type === "histogram" ? layer.colors === 1 || layer.colors === 4 : TWO_OR_ONE.has(layer.type) ? layer.colors === 1 : TWO_KEPT.has(layer.type) && layer.colors === 1 && layer.colorBelow !== undefined) || layer.colorFrom || layer.byLevels || layer.byLayers)) delete layer.colors;
  // (Bubbles made so (another layer's turned into them): on the pane's first that can carry them, else the chart's.)
  for (const layer of doc.layers) if (layer.type === "bubbles" && layer.ref === undefined) layer.ref = (markable(doc).find((other) => other.pane === layer.pane) ?? markable(doc)[0])?.id ?? "";
  // (A signal's layer gone, or bubbles': none, said so (""), so a reload doesn't put them on another; a trend's, none.)
  for (const layer of doc.layers) if (layer.ref && !(layer.type === "trend" ? refsFor(doc, layer) : placedOn(layer) ? markable(doc) : []).some((other) => other.id === layer.ref)) placedOn(layer) ? (layer.ref = "") : delete layer.ref;
  // A signal: marks on its layer (in its pane, on its side: below; its layer gone, none till another's picked: never
  // moved to another on its own); nothing of a line's look.
  // Bubbles: nothing of a line's look, their sizes' fields theirs alone.
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
  // (Out of its axis' fit where it can be.)
  for (const layer of doc.layers) if ((layer.fit !== false && layer.fit !== "zero") || !fittable(layer)) delete layer.fit;
  // A cloud or a trend where its layers are (it reads their values as one), in another pane behind them (a trend in
  // front, the other way round where a pane's last is in front), as one added; theirs apart, where it was (it says
  // so). A signal on its layer's.
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
  // (A pane's order with fewer than two layers is none: nothing then says it, nothing could turn it back.)
  for (const pane of doc.panes) if (pane.reverse && doc.layers.filter((layer) => layer.pane === pane.id).length < 2) delete pane.reverse;
  // Unknown saved settings are raw; known ones stay kept while an interval or a distribution blocks them.
  for (const pane of doc.panes) if (pane.average && !Object.hasOwn(AVERAGES, pane.average)) delete pane.average;
  const used = new Set(doc.layers.map((layer) => layer.pane));
  doc.panes = doc.layers.length ? doc.panes.filter((pane) => used.has(pane.id)) : doc.panes.slice(0, 1);
  const seen = new Set();
  for (const layer of doc.layers) {
    if (seen.has(layer.letter)) layer.letter = nextLetter({ ...doc, layers: doc.layers.filter((other) => other !== layer) });
    seen.add(layer.letter);
  }
  // (A view kept aside's letters none of the shown ones': shown, a formula reads what it read.)
  for (const layer of asideLayers(doc)) {
    if (seen.has(layer.letter)) layer.letter = nextLetter(doc);
    seen.add(layer.letter);
  }
  const order = new Map(doc.panes.map((pane, i) => [pane.id, i]));
  doc.layers.sort((a, b) => (order.get(a.pane) ?? 0) - (order.get(b.pane) ?? 0));
  // A drawing goes with its layer.
  if (doc.drawings) {
    const layers = new Set(doc.layers.map((layer) => layer.id));
    doc.drawings = doc.drawings.filter((drawing) => layers.has(drawing.layer));
    if (!doc.drawings.length) delete doc.drawings;
  }
}

// ── Library, saving, undo ─────────────────────────────────────────────────

const STORE = "bitview-studio:v2";
const RECENT = "bitview-studio:recent";
const PREFS = "bitview-studio:prefs";
/** The previews edited and not yet kept or closed (every tab's): previews again at the next start. */
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
/** The library as stored when this tab read it. */
const startText = (() => {
  try {
    return localStorage.getItem(STORE);
  } catch {
    return null;
  }
})();
/**
 * The library as storage held it when this tab last wrote, read or took it in: storage still holding it, no other tab
 * has saved since (see storedSince). (Read at start, not "": else its first save takes its own start in as another
 * tab's, and what it let go of since, an unedited preset's copy whose place is gone, comes back.)
 */
let storedAs = startText ?? "";
/** The library written, as text; whether it was. @param {string} text */
function writeStored(text) {
  try {
    localStorage.setItem(STORE, text);
  } catch {
    return false;
  }
  storedAs = text;
  return true;
}
/**
 * The charts, their folders, and the sidebar's own order: every item's among its folder's, at any depth, as arranged
 * ("f:" a folder's id, "c:" a chart's; see childrenOf); the presets as edited here (each a chart of its own, by the
 * preset it is: see presetKey); and the charts deleted, by id, when (see `discard`).
 * @type {{ charts: Chart[], active: string, folders: Folder[], order: string[], presets: Chart[], deleted: Record<string, number> }}
 */
let library = (() => {
  try {
    return JSON.parse(startText ?? "");
  } catch {
    return null;
  }
})();
/**
 * The library's shape, as this Studio writes it (raised with what `validate` learns to read, and with what's kept:
 * 5, the deletions, so a Studio that doesn't record them can't save over them; 6, folders in folders, so one that
 * reads them flat can't save them so; 7, the presets as edited, so one that doesn't keep them can't drop them; 8, a chart's interval Auto, so one that
 * doesn't know it can't save it as one interval for good; 9, signals, so one that doesn't know them can't save them
 * as lines; 10, a signal's down color and results, so one that doesn't know them can't drop them; 11, an area's fill
 * opacity and solid fill, candles' one color and hollow bodies, so one that doesn't know them can't drop them; 12, a
 * path (a drawing of many points) and a folder's icon, so one that doesn't know them can't drop them; 13, heatmaps
 * and gradients, so one that doesn't know them can't turn a heatmap into a line or drop a gradient; 14, profiles, so
 * one that doesn't know them can't turn a profile into a line or drop its holder groups; 15, a layer's interval ceiling
 * (`maxInterval`) and its zero framing (`fit: "zero"`), so one that doesn't know them can't drop them; 16, a pane drawn as a
 * heatmap, its outline and its layers' short names, so one that doesn't know them can't drop them or draw it as a stack; 17, a heatmap's gradient
 * on its pane, so one that keeps it on the layer can't drop it): one saved by a newer
 * Studio (a later number), or stored in a shape it can't read at all (damaged, another's), is opened read-only here,
 * said so, and kept as it is: it may hold what this one would drop, or lose entirely, if it saved over it.
 */
const SCHEMA = 17;
let readOnly = Number(/** @type {any} */ (library)?.schema) > SCHEMA || (!Array.isArray(library?.charts) && startText !== null);
/** Why nothing's saved here, if it isn't: a newer Studio's library ("newer"), one this can't read ("unreadable"), storage full ("full"). */
let unsavedWhy = !readOnly ? "" : Number(/** @type {any} */ (library)?.schema) > SCHEMA ? "newer" : "unreadable";
if (!Array.isArray(library?.charts)) library = { charts: [], active: "", folders: [], order: [], presets: [], deleted: {} };
/** The sidebar's order as stored, read: its keys, once each. */
const orderOf = (/** @type {any} */ stored) => [...new Set((Array.isArray(stored) ? stored : []).filter((key) => typeof key === "string" && /^[fc]:[a-z0-9]{1,16}$/.test(key)))];
/** Deletions as stored, read: a chart's or a folder's id, when; each kept a month (every other tab has heard long before). */
const deletedOf = (/** @type {any} */ stored) => {
  const since = Date.now() - 30 * 86400000;
  return Object.assign(/** @type {Record<string, number>} */ (Object.create(null)), Object.fromEntries(Object.entries(stored && typeof stored === "object" ? stored : {}).filter(([id, at]) => /^[a-z0-9]{1,16}$/.test(id) && Number.isFinite(at) && at > since)));
};
library.deleted = deletedOf(library.deleted);
/** Saved charts this version couldn't read, kept aside as they were: never lost to the next save. */
const UNREAD = "bitview-studio:unread";
/** A stored chart, validated like any import but keeping its identity; throws if it isn't one. @returns {Chart} */
function read(stored) {
  const doc = validate(stored, true);
  if (typeof stored.id === "string" && /^[a-z0-9]{1,16}$/.test(stored.id)) doc.id = stored.id;
  // (Its folder is the library's, not the chart's: kept here, never in a file or a link.)
  if (typeof stored.folder === "string" && /^[a-z0-9]{1,16}$/.test(stored.folder)) doc.folder = stored.folder;
  // (A preset's, which one it is: the library's too.)
  if (typeof stored.preset === "string" && stored.preset.length <= 2000) doc.preset = stored.preset;
  if (typeof stored.base === "string" && /^[a-z0-9]{1,24}$/.test(stored.base)) doc.base = stored.base;
  if (stored.blank === true) doc.blank = true;
  doc.updated = Number.isFinite(stored.updated) ? stored.updated : 0;
  // (A preset's copy kept as it was stored, to be told as made or edited from that, before it's normalized: a newer
  // client moving its fields isn't an edit (see pristine); one as made is made anew once the presets are read, see prunePresets.)
  keptAsStored(doc);
  normalize(doc);
  return doc;
}
/** A stored chart, read; one that can't be is set aside (never dropped: if it can't be, nothing's saved). @returns {Chart[]} */
function restore(stored) {
  try {
    return [read(stored)];
  } catch (error) {
    // Never silently: the chart goes aside, untouched, and it's said (see start). One per chart, as it was last: a
    // chart edited again elsewhere replaces its older copy.
    const aside = storage.get(UNREAD, []);
    const list = (Array.isArray(aside) ? aside : []).filter((item) => (stored?.id ? item?.id !== stored.id : JSON.stringify(item) !== JSON.stringify(stored)));
    // (Not set aside: the library stays as stored, read-only, rather than lose it at the next save.)
    if (!storage.set(UNREAD, [...list, stored])) (readOnly = true), (unsavedWhy = "full");
    console.error(`A saved chart couldn't be read; it's kept under "${UNREAD}" in this browser's storage.`, error);
    return [];
  }
}
library.charts = library.charts.flatMap(restore).filter((doc) => !(library.deleted[doc.id] >= doc.updated));
/** The presets as edited, as stored, read: one a preset (the latest), not deleted since. @returns {Chart[]} */
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
/**
 * Folders as stored, read: each named (at most 60 characters), once, in a folder that's there (none in itself, by
 * any way round: at the top then); a chart's folder that's gone, none.
 */
/** The icons a folder can have (its first letter if none), by name: a preset section's, and a few more. */
const FOLDER_ICONS = ["candles", "scale", "coins", "people", "hourglass", "percent", "cube", "key", "pickaxe", "nodes", "clock", "swap", "layers", "star", "flag", "bolt", "target", "calendar"];
/** A name's first letter, as a folder's mark shows it. */
const letterOf = (/** @type {string} */ name) => ([...name.trim()][0] ?? "?").toUpperCase();
/** A folder's mark, wherever it's listed (the sidebar, the rail, menus): its icon if it has one, else its first letter. */
const folderMark = (/** @type {Folder} */ folder) => (folder.icon && ICONS[folder.icon] ? `<span class="chart-mark section-mark" aria-hidden="true">${ICONS[folder.icon]}</span>` : `<span class="chart-mark folder-mark" aria-hidden="true">${escapeHtml(letterOf(folder.name))}</span>`);
/** A preset section's mark: a top one's own icon; else a folder's, open or closed. */
const sectionMark = (/** @type {PresetNode | null | undefined} */ node, open = false) => (node?.icon ? `<span class="chart-mark section-mark">${ICONS[node.icon]}</span>` : `<span class="chart-mark folder-glyph">${open ? ICONS.folderOpen : ICONS.folder}</span>`);
function foldersOf(/** @type {any} */ stored, /** @type {Chart[]} */ charts) {
  const seen = new Set(), folders = /** @type {Folder[]} */ ((Array.isArray(stored) ? stored : []).flatMap((folder) => {
    if (typeof folder?.id !== "string" || !/^[a-z0-9]{1,16}$/.test(folder.id) || seen.has(folder.id)) return [];
    seen.add(folder.id);
    return [{ id: folder.id, name: String(folder.name ?? "").trim().slice(0, 60) || "Folder", ...(typeof folder.parent === "string" && folder.parent ? { parent: folder.parent } : {}), ...(folder.closed === true ? { closed: true } : {}), ...(FOLDER_ICONS.includes(folder.icon) ? { icon: folder.icon } : {}) }];
  }));
  const byId = new Map(folders.map((folder) => [folder.id, folder]));
  // (Up from each: one in a folder not here, at the top; a loop, cut where it closes.)
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
// Charts set aside by a version that couldn't read them come back, first in the list, once this one can; the others
// stay aside. One here already (the same chart, or a copy alike in all but its id) isn't brought back twice.
{
  const aside = storage.get(UNREAD, []);
  if (Array.isArray(aside) && aside.length) {
    const alike = (doc) => JSON.stringify({ ...doc, id: "", updated: 0 });
    const ids = new Set(library.charts.map((doc) => doc.id)), here = new Set(library.charts.map(alike)), back = [], still = [], newer = new Map();
    for (const stored of aside) {
      try {
        const doc = read(stored);
        if (here.has(alike(doc)) || library.deleted[doc.id] >= doc.updated) continue;
        // (The same chart here, edited before this one was set aside: this one, in its place.)
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
    // Off the side only once they're saved in the library (a preset's with the presets, the latest of each).
    const charts = back.filter((doc) => doc.preset === undefined), edited = presetsOf([...library.presets, ...back.filter((doc) => doc.preset !== undefined)]);
    const replaced = library.charts.map((doc) => {
      const later = newer.get(doc.id);
      if (!later) return doc;
      // (Where it is, the library's: a move doesn't make a chart later.)
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
/**
 * The previews, newest first: charts a link (or a script) opened, not in your Library. Each stays, edited too, till
 * kept (Keep, ⌘S) or closed (its ×), listed over your charts (in the sidebar, the page, ⌘K); kept aside
 * (PREVIEW_ASIDE), edited or as a link opened it, and back after a reload. The same chart opened again, the one there
 * is. (The empty chart in place of none, a first visit's, the last one deleted's, is one too: a draft, yours from
 * its first edit; an empty one, left, simply goes.)
 * @type {Chart[]}
 */
let previewCharts = [];
const isPreview = (/** @type {Chart | null | undefined} */ doc) => Boolean(doc && previewCharts.includes(doc));
/** Whether a chart is one of yours (not a preview, not a preset). @param {Chart} doc */
const isYours = (doc) => !isPreview(doc) && doc.preset === undefined;
/** The most previews kept as they were opened: older ones go as newer come (an edited one stays till it's saved or closed). */
const MOST_PREVIEWS = 20;
/**
 * A preview added, the newest (once); past the most, the oldest not edited let go of (as a closed one is: no edit
 * lost; never the one open, nor a draft).
 * @param {Chart} doc
 */
function addPreview(doc) {
  if (isPreview(doc)) return;
  previewCharts.unshift(doc);
  const plain = previewCharts.filter((item) => !touched.has(item) && !drafts.has(item) && item !== chart && item !== doc);
  for (const old of plain.slice(MOST_PREVIEWS - 1)) dropPreview(old), forgetPreview(old), previews.delete(old.id);
}
/** A preview let go of (kept, closed, left empty). @param {Chart} doc */
const dropPreview = (doc) => void (previewCharts = previewCharts.filter((item) => item !== doc));
/** A new chart, empty, as a preview. */
const newDraft = () => ((draft) => (addPreview(draft), draft))(draftChart());
/** Quick Look, while it's on (see quickShow): what it shows, by its chart and content (none: the row isn't a chart). @type {{ key: string } | null} */
let quick = null;
/** The previews listed as such (a draft isn't: the empty chart in place of none, yours once something's in it). */
const listedPreviews = () => previewCharts.filter((doc) => !drafts.has(doc));
/** The preview already there that's the same chart (its range aside: a look, not what it is), if one is. @param {Chart} doc */
const previewLike = (doc) => (doc.layers.length ? previewCharts.find((item) => item !== doc && sameChart(item, doc)) : undefined);
/** The preview alike (previewLike) shown as `doc` came: its interval, range and view, its chunks at hand (for its views before) fetched anew. @param {Chart} like @param {Chart} doc */
function lookAs(like, doc) {
  like.index = doc.index;
  like.range = doc.range;
  takeView(doc.id, like.id);
  forgetSpans(like.id);
}
/** A chart's view (where it's zoomed or panned to) given to another id. @param {string} from @param {string} to */
const moveView = (from, to) => {
  const all = viewsNow();
  if (!all[from] || from === to) return;
  all[to] = all[from];
  delete all[from];
  putViews(all);
};
/** A chart's view forgotten (as made, it opens at its range). @param {string} id */
const forgetView = (id) => {
  const all = viewsNow();
  if (all[id]) (delete all[id], putViews(all));
};
/** The view a chart came with (a link's, a file's) given to the one it's alike: none, that one's forgotten. @param {string} from @param {string} to */
const takeView = (from, to) => (viewsNow()[from] ? moveView(from, to) : forgetView(to));
/** A chart given a new id (its own taken): its view, this tab's place in the history and the chart open follow. @param {Chart} doc @param {string} id */
const rekey = (doc, id) => {
  const old = doc.id;
  doc.id = id;
  moveView(old, id);
  for (const map of /** @type {Map<string, any>[]} */ ([histories, previews, thumbs, inspector.formulaDrafts])) if (map.has(old)) map.set(id, map.get(old)), map.delete(old);
  if (chart === doc) library.active = id;
  if (displayed?.doc.id === old) (displayed.doc.id = id), (displayed.look = lookKey(displayed.doc));
  if (history.state?.chart === old) history.replaceState({ ...history.state, chart: id }, "");
  // (Edited, kept aside under its new id, and its copy under the old one let go of once that's written (another
  // tab's later one stays).)
  if (touched.has(doc) || (isPreview(doc) && !drafts.has(doc))) asideDirty.add(doc);
  if (isPreview(doc) && !drafts.has(doc)) keptPreviews.set(old, doc.updated);
};
/** A link that couldn't be read, left in the address bar to be put right (until another chart's opened). */
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
/** A new chart Cancel or esc is taking back (see leaveEmpty): where it was. @type {{ doc: Chart, at: number, folder: string | undefined } | null} */
let cancelledNew = null;
/**
 * An empty chart of yours let go of (see emptyYours): out of the list, deleted as of its last edit only (another tab's
 * work on it since, not saved yet here, wins).
 */
const dropEmpty = (/** @type {Chart} */ doc) => {
  const at = library.charts.indexOf(doc);
  if (at < 0) return;
  library.charts.splice(at, 1);
  // (Taken back by Cancel or esc: said, with an Undo that brings it back where it was (its folder, its place) and
  // opens it, still new: its Cancel with it, a reload through.)
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
  // (Within the month deletions are kept, or it'd be pruned before another tab hears.)
  library.deleted[doc.id] = Math.max(doc.updated, Date.now() - 29 * 86400000, library.deleted[doc.id] ?? 0);
};
/** A deleted chart back (its Undo): edited now, after its deletion, wherever that's known; its folder made again since, in that one; gone, at the top. @param {Chart} doc */
const recover = (doc) => {
  stampEdit(doc);
  if (doc.folder) doc.folder = folderNow(doc.folder);
  if (doc.folder && !library.folders.some((folder) => folder.id === doc.folder)) delete doc.folder;
  delete library.deleted[doc.id];
};
/** A deleted chart back in your charts (an Undo, a Redo: the same chart, its id and steps its own), at `at`. @param {Chart} doc @param {number} at */
const relist = (doc, at) => {
  recover(doc);
  library.charts.splice(Math.min(at, library.charts.length), 0, doc);
};
/** A chart edited, stamped: now, and past its last stamp and any deletion of it (a clock set back). @param {Chart} doc */
const stampEdit = (doc) => (doc.updated = Math.max(Date.now(), doc.updated + 1, (library.deleted[doc.id] ?? 0) + 1));
/** Whether a chart was deleted after its last edit (here or in another tab). @param {{ id: string, updated: number }} doc */
const isDeleted = (doc) => library.deleted[doc.id] >= doc.updated;
/** Whether a chart stays in the library: not deleted since its last edit, or open (it goes once another is). @param {{ id: string, updated: number }} doc */
const alive = (doc) => doc.id === chart?.id || !isDeleted(doc);
/** @returns {Chart | undefined} */
const chartById = (id) => library.charts.find((doc) => doc.id === id) ?? library.presets.find((doc) => doc.id === id) ?? previewCharts.find((doc) => doc.id === id);
/** Whether a chart is one of yours (the Library's). @param {Chart} doc */
function inLibrary(doc) {
  return library.charts.some((other) => other.id === doc.id);
}
/**
 * The catalog indexed (`catalogIndex`, `catalogIntervals`, `byBlock`, `sharedTops`): in a worker, off the page's thread (long on a
 * phone without its JIT, in a launch's first seconds), begun with the rest, as what asks for it is (its own copy of
 * the client, fetched beside a first chart's data, held that data up); without one (or it failing), here, a slice at a time while the
 * page is idle. Then charts saved without paths get theirs (and survive renames). What needs it before (a search, a
 * series added) has the rest done here at once, never waiting on the worker or an idle moment: a hidden tab's are
 * throttled (to one a minute, after five minutes hidden).
 */
const indexCatalog = (() => {
  /** @type {ReturnType<typeof catalogWalk> | undefined} */
  let walk;
  /** What it found, the catalog's: the first to come, the worker's or the page's own. */
  const take = (/** @type {{ index: Map<string, string>, intervals: Map<string, readonly string[]>, sharedTops: Set<string> }} */ found) => {
    if (catalogIndex) return;
    catalogIndex = found.index;
    catalogIntervals = found.intervals;
    for (const [name, ids] of found.intervals) if (!ids.length) byBlock.add(name);
    for (const name of found.sharedTops) sharedTops.add(name);
    seriesCount = catalogIndex.size - byBlock.size;
    $("cta-label").textContent = `Search ${grouped.format(seriesCount)} series`;
    withRest(() => whenIdle(command.ranksSoon));
    // (After what asked for it: its work isn't held up, nor failed, by this.)
    queueMicrotask(() => {
      // (Saved if that learned anything: a series' path, a formula's.)
      const docs = [...library.charts, ...library.presets], before = docs.map((doc) => JSON.stringify(doc));
      for (const doc of docs) normalize(doc);
      if (docs.some((doc, i) => JSON.stringify(doc) !== before[i])) save();
      // (The chart drawn before the tree came, by a series' old name (renamed since): drawn again by its new one.)
      if (docs.includes(chart) && JSON.stringify(chart) !== before[docs.indexOf(chart)]) draw();
    });
  };
  /** Walked here while `more` says (it's told how many nodes so far). */
  const run = (/** @type {(done: number) => boolean} */ more) => {
    // (By the whole client's tree: see whole.)
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
    // (Its answer unread (an error, one that can't be delivered or read): walked here, as without one.)
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
/** The layer open in the panel (its settings). */
let selected = "";
/**
 * The layers picked, as charts are in the grid: by ⌘- or ⇧-click (in the list or a legend), space or X on a row, ⇧↑↓,
 * ⌘A in the list, a long press by touch; while some are, a click picks too. The last one picked is where a run
 * starts (`pickHead`: where ⇧↑↓ has taken it).
 */
const layerPicks = new Set();
let layerPickLast = "", pickHead = "";
/** The layer last open (its settings left): where a ⇧-click's run starts, nothing being picked or open (once). */
let lastOpen = "";
/**
 * A step of a chart's history: its state on the other side (as JSON), and when it last moved (done, undone, redone:
 * see stepCount): ⌘Z and ⇧⌘Z take the last moved of those in reach (see historyFor). @typedef {{ state: string, at: number }} ChartStep
 */
const edits = { past: /** @type {ChartStep[]} */ ([]), future: /** @type {ChartStep[]} */ ([]), key: "", at: 0, held: 0 };
/** Steps moved so far (done, undone, redone), in any history: each one's `at`. */
let stepCount = 0;
/** The drag under way (a slider's, a color surface's: see track), counted; none, 0. Its edits undo as one. */
let holding = 0, holds = 0;
/** Each chart's undo and redo while it isn't open (the last few charts' kept: see keepHistory), for as long as the page is. */
const histories = /** @type {Map<string, { past: ChartStep[], future: ChartStep[] }>} */ (new Map());
/**
 * Undo budgets count JSON characters (two bytes each); kept charts get less, or a large chart's history could
 * hold hundreds of MB.
 */
const UNDO_CHARS = 8e6, KEPT_CHARS = 2e6;
/** Steps kept: the open chart's, each other chart's; how many other charts keep theirs. */
const UNDO_STEPS = 200, KEPT_STEPS = 50, KEPT_HISTORIES = 8;
/** Steps within a budget: the farthest go first (the oldest undo, then the farthest redo), the nearest of each stays. */
function trimSteps(/** @type {{ past: ChartStep[], future: ChartStep[] }} */ steps, /** @type {number} */ budget) {
  let size = 0;
  for (const step of steps.past) size += step.state.length;
  for (const step of steps.future) size += step.state.length;
  while (size > budget && steps.past.length > 1) size -= /** @type {ChartStep} */ (steps.past.shift()).state.length;
  while (size > budget && steps.future.length > 1) size -= /** @type {ChartStep} */ (steps.future.shift()).state.length;
}
/** Bumped by every change of state: edits, undo, redo, opening a chart. */
let revision = 0;
/** Saves and toasts so far (a Library undo that saved nothing did nothing; one that said why, said it: see undoLibrary). */
let saves = 0, toasts = 0;
/** Whether the open chart changed in this task (a toast now says that change; see toastOf). */
let chartChanged = false;
/** The sidebar's groups, in its order (Home's doors too), each its name and its page. @typedef {"recent" | "previews" | "library" | "presets"} Group */
const GROUPS = /** @type {Group[]} */ (["recent", "previews", "library", "presets"]);
const GROUP_OF = /** @type {Record<Group, [string, Place]>} */ ({ recent: ["Recent", { recent: true }], previews: ["Previews", { previews: true }], library: ["Library", { folder: "" }], presets: ["Presets", { path: [] }] });
/** Preferences as stored, read (each its default where it's missing or isn't one). */
function readPrefs(/** @type {any} */ stored) {
  return {
    sidebar: stored.sidebar === "collapsed" || stored.sidebar === "expanded" ? stored.sidebar : tablet.matches ? "collapsed" : "expanded",
    panel: stored.panel === "open" || stored.panel === "closed" ? stored.panel : tablet.matches ? "closed" : "open",
    home: stored.home !== false,
    /** The sidebar's and the panel's widths, the person's (null: as they come; see setWidth). */
    sideWidth: /** @type {number | null} */ (Number.isFinite(stored.sideWidth) ? stored.sideWidth : null),
    panelWidth: /** @type {number | null} */ (Number.isFinite(stored.panelWidth) ? stored.panelWidth : null),
    /** The presets' sections open in the sidebar (their paths, as kept). */
    /** Where the Charts page was last (see Place): Charts, from a chart, after a reload too, comes back there. */
    at: /** @type {any} */ (stored.at && typeof stored.at === "object" ? stored.at : null),
    presetsOpen: /** @type {string[]} */ (Array.isArray(stored.presetsOpen) ? stored.presetsOpen.filter((/** @type {any} */ key) => typeof key === "string").slice(-500) : []),
    /** The sidebar's groups open there (see GROUPS): each closed till it's opened, as left after. */
    groupsOpen: /** @type {Group[]} */ (Array.isArray(stored.groupsOpen) ? GROUPS.filter((group) => stored.groupsOpen.includes(group)) : []),
    /** Single-key shortcuts (a letter, a digit, a sign alone): on unless turned off. */
    keys: stored.keys !== false,
    /** The view linked: every chart on the same dates, in every tab (see linkView); off unless turned on. */
    linkView: stored.linkView === true,
    theme: /** @type {"system" | "light" | "dark"} */ (["system", "light", "dark"].includes(stored.theme) ? stored.theme : "system"),
    belt: { on: stored.belt?.on === true, dock: ["top", "bottom", "left", "right"].includes(stored.belt?.dock) ? stored.belt.dock : "bottom", at: Number.isFinite(stored.belt?.at) ? clamp(stored.belt.at, 0, 1) : 0.5 },
    pen: /** @type {{ color: string, width: number, dash: number, opacity?: number, start?: End, end?: End, size: number, back?: string }} */ ({ ...lookOfDrawing("trend", stored.pen), size: Number.isFinite(stored.pen?.size) ? presetSize(stored.pen.size) : TEXT_SIZE }),
  };
}
/**
 * Per-browser view preferences: whether the sidebar is collapsed, whether the layers panel shows (at first, closed
 * on a narrow screen), whether the page opens on the grid; the drawing belt (on, where), and the pen: the look the
 * next drawing takes, whatever its tool (the last one set, on a tool or on a drawing).
 */
const prefs = readPrefs(storage.get(PREFS, {}));
// (A phone's drawing tools put away at each start: there, they take the place of the chart's foot, the interval and
// the ranges.)
if (phone.matches) prefs.belt.on = false;
/** Each preference as this tab last wrote or took it: only what it changed since is written (another tab's kept). */
const prefsSeen = Object.fromEntries(Object.entries(prefs).map(([key, value]) => [key, JSON.stringify(value)]));
function savePrefs() {
  const changed = Object.fromEntries(Object.entries(prefs).filter(([key, value]) => JSON.stringify(value) !== prefsSeen[key]));
  // (Counted written once they are: a write refused, the next tries them again.)
  const written = storage.set(PREFS, { ...storage.get(PREFS, {}), ...changed });
  if (written) for (const [key, value] of Object.entries(changed)) prefsSeen[key] = JSON.stringify(value);
  return written;
}
/**
 * The theme, as chosen, System the device's (the default). The page takes it
 * from its tokens; the chart, the charts' marks and their drawings draw again with them (and the palette's tone: see
 * shownOklch), crossfaded where the browser can. Set before the first paint (see the page's first script), and here.
 */
const THEMES = /** @type {const} */ ([
  ["system", "System", "computer", "As the device is set, and as it changes."],
  ["light", "Light", "sun", "Light surfaces; the palette's colors deepened to read on white, any other as given."],
  ["dark", "Dark", "moon", "Dark surfaces; the data's colors as they are."],
]);
const deviceLight = matchMedia(MEDIA.light);
/** The bar over the page (a browser's, an installed app's, a phone's status bar) in the page's color under it: the grid's ground, a chart's card on a phone. */
// (One a scheme in the page, for its first frame; both the theme's, whichever the device's is: the chosen one wins.)
const themeColors = /** @type {NodeListOf<HTMLMetaElement>} */ (document.querySelectorAll('meta[name="theme-color"]'));
// (The theme's, kept till it changes: the page coming or going reads no style for it, mid-change.)
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
/** The Charts page is on screen, where `homeAt` says (the open chart stays laid out under it). */
let home = false;

/** (`unsaved`: a write that failed, tried again on leaving: storage may take it by then.) */
let saveDue = false, savedFade = /** @type {Animation | null} */ (null), unsaved = false;
/** Whether the write coming says "Saved" (an edit asked for it meanwhile). */
let announcing = false;
/**
 * The library as stored now, if another tab saved since this one: read, and taken in by the caller (so not again by
 * the next save, which would bring back what this tab let go of since), or `null` (none since, or none).
 */
function storedSince() {
  let text = null;
  try {
    text = localStorage.getItem(STORE);
  } catch {}
  if (text === null || text === storedAs) return null;
  try {
    const stored = JSON.parse(text);
    // (Another tab's, newer or unreadable here: kept, and none of this one's saved over it.)
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
/**
 * The library written, its shape's number with it; storage full, the cached previews and marks (drawn again) give
 * way to it. Whether it was. Read-only (see SCHEMA), it isn't, and that's said once.
 */
/** The sidebar's arrangement (its order, its folders, each chart's folder), as a key; as last written or taken in. */
const arrangedNow = () => JSON.stringify([library.order, library.folders, library.charts.map((doc) => [doc.id, doc.folder ?? ""])]);
let arrangedAs = "";
function persist() {
  // Another tab's save since this one's, not heard yet (its storage event to come, or never: two tabs closed at once,
  // one back from the browser's history), taken in first: what's written holds both.
  const stored = storedSince();
  // (The arrangement this one's only if it changed it since it last wrote or took one in: else theirs, as it's
  // taken in when heard (an edit's save here doesn't roll back a folder renamed or a chart moved there).)
  if (stored) reconcile(stored, arrangedNow() !== arrangedAs, true);
  if (readOnly) return false;
  // (A deletion the chart open here still stands under kept past its month: else it'd be saved back.)
  const pruned = deletedOf(library.deleted);
  for (const doc of [...library.charts, ...library.presets]) if (isDeleted(doc)) pruned[doc.id] = library.deleted[doc.id];
  library.deleted = pruned;
  // (The open chart, deleted in another tab, stays here unsaved: edited, it's saved again.)
  const written = JSON.stringify({ ...library, charts: library.charts.filter((doc) => !isDeleted(doc)), presets: library.presets.filter((doc) => !isDeleted(doc)), schema: SCHEMA });
  // (Edited previews, with it: never in the list, never lost. Only those changed here written, each over its copy
  // there if that's older: another tab's kept aside as they are.)
  // (Storage full: the caches let go of first, then again; not written, not saved, and said.)
  const mine = previewCharts.filter((doc) => asideDirty.has(doc) && !drafts.has(doc));
  let aside = true;
  if (mine.length) {
    const merged = new Map(previewsAside().map((doc) => [doc.id, doc]));
    // (One not edited (a link's, as it came): marked so, a preview again as it was, not an edited one.)
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
  // (Previews kept: let go of aside only once they're saved with your charts, never before; one rekeyed, once its
  // copy under its new id is written.)
  if (ok && aside && keptPreviews.size) {
    const left = previewsAside().filter((item) => !keptPreviews.has(item.id) || laterAside(item, /** @type {number} */ (keptPreviews.get(item.id))));
    if (left.length) storage.set(PREVIEW_ASIDE, left);
    else storage.drop(PREVIEW_ASIDE);
    keptPreviews.clear();
  }
  return ok && aside;
}
/** When what isn't saved was last said (see tellUnsaved). */
let unsavedToldAt = -Infinity;
/**
 * What isn't saved, said with a way to keep it (a notice, at most once a minute while it lasts: an edit's, where the
 * header's word isn't seen too): read-only, its edits a download away (to import once reloaded); one this can't
 * read, it as stored; storage full, the charts.
 */
function tellUnsaved() {
  if (Date.now() - unsavedToldAt < 60_000) return;
  unsavedToldAt = Date.now();
  const why = readOnly ? unsavedWhy : "full";
  if (why === "unreadable") return notice("Your saved charts can't be read in this browser. They're left untouched, and changes here won't be saved", { label: "Download", run: () => (downloadStored(), menus.downloadAll(true)) });
  notice(why === "newer" ? "A newer Studio saved your charts: changes here can't be saved. Download them, then reload" : "Not saved: storage is full or blocked. Download your charts to keep them", { label: "Download", run: () => menus.downloadAll(true) });
}
/** The library as stored, as it is (one this can't read): a file of it. */
function downloadStored() {
  let stored = "";
  try {
    stored = localStorage.getItem(STORE) ?? "";
  } catch {}
  if (!stored) return notice("Nothing saved to download");
  menus.download(`bitview-charts-as-stored-${new Date().toLocaleDateString("en-CA")}.json`, new Blob([stored], { type: "application/json" }));
}
/**
 * What's being typed where it shows (a drawing's text or label, a name), kept now: before a save, a page left. `open`:
 * the page only hidden, a name or a drawing's text left open as it is, to go on with (esc taking it back still).
 */
function flushEdits(open = false) {
  // (Those left with the window ended too, as the window's return would: a title trimmed.)
  if (!open) (endTyping(), ((ends) => (leftOpen.clear(), ends.forEach((end) => end())))([...leftOpen.values()]));
  // (A name being given, a formula being typed: the rest's, none before it's in.)
  withRest(() => {
    if (!open) grid.naming?.finish();
    inspector.applyFormulaLeft();
    const formula = /** @type {HTMLTextAreaElement | null} */ (document.getElementById("formula"));
    if (formula && document.activeElement === formula && selected) inspector.applyFormula(selected, formula.value);
  });
}
/**
 * Writes the library at the page's next idle moment (a second at most), once for all asked for till then: never put
 * off by more; `announce` shows a brief "Saved" after an edit. A failure stays on screen.
 */
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
/** A write's outcome, said: a failure stays on screen (and is tried again on leaving), until one goes through. */
function written(/** @type {boolean} */ ok, /** @type {boolean} */ announced) {
  const saved = $("saved");
  unsaved = !ok && !readOnly;
  if (ok && !announced && !saved.classList.contains("error")) return;
  // (Not saved, read-only too: said in the header for as long as it lasts, and in a notice with a way to keep it.)
  saved.textContent = ok ? "Saved" : readOnly ? "Read-only: not saved" : "Not saved";
  saved.classList.toggle("error", !ok);
  if (ok) saved.removeAttribute("role");
  else saved.setAttribute("role", "alert");
  if (!ok) tellUnsaved();
  // (Said a while, then faded: said again meanwhile, from the start.)
  savedFade?.cancel();
  if (!ok) return;
  (savedFade = saved.animate([{ opacity: 1, offset: 0.55 }, { opacity: 0 }], { duration: 2600, easing: "ease", fill: "forwards" })).finished.then(() => (saved.textContent = ""), () => {});
}

/**
 * What a library holds, as one key: its charts as last edited, and its folders. (Not its deletions: a save takes in
 * storage's first, and one lost there brings back what it deleted, a difference here.)
 * @param {any[]} charts @param {Folder[]} folders @param {Record<string, number>} deleted
 */
const holdings = (charts, folders, deleted, presets = /** @type {any[]} */ ([])) => JSON.stringify([[...charts, ...presets].filter((doc) => !(deleted[doc?.id] >= doc?.updated)).map((doc) => `${doc?.id}@${doc?.updated}`).sort(), folders.map((folder) => folder.id).sort()]);
/**
 * Another tab's save, taken in. Its charts, ours kept where ours were edited later or it hasn't them; its folders, ours
 * it hasn't kept; every tab's deletions, what was deleted after its last edit gone (but the chart open here). The
 * arrangement (the charts' order, which is in which folder, the sidebar's order, a folder's name) as that tab saved
 * it; `saving`, as this one has it (changed here since it last wrote one: it's what's being saved). `writing`: taken
 * in by this tab's save, which writes it at once (nothing left to save). Storage then without
 * what's here (two tabs saving at once: the last one's library alone), the two, merged, saved over it.
 */
function reconcile(stored = storedSince(), saving = false, writing = false) {
  if (!stored) return;
  const deleted = deletedOf(stored.deleted), incoming = stored.charts.flatMap(restore), folders = foldersOf(stored.folders, []);
  // (A preview another tab kept: the same, that one it is now (open, that one's open); else one of its own, a new
  // id, its history and all with it.)
  for (const doc of [...previewCharts]) {
    const saved = incoming.find((item) => item.id === doc.id);
    if (saved && sameChart(saved, doc)) (dropPreview(doc), forgetPreview(doc));
    else if (saved) rekey(doc, uid());
  }
  for (const [id, at] of Object.entries(deleted)) if (!(library.deleted[id] >= at)) library.deleted[id] = at;
  const ours = new Map(library.charts.map((doc) => [doc.id, doc])), theirs = new Map(incoming.map((doc) => [doc.id, doc]));
  // (Saving here: the arrangement as this tab last wrote or took one in, each thing in it this tab's only where it
  // changed it since (a chart moved there, a folder closed there: not saved back as it was here).)
  const arranged = saving && arrangedAs ? /** @type {[string[], Folder[], [string, string][]]} */ (JSON.parse(arrangedAs)) : null, placedThen = new Map(arranged?.[2] ?? []);
  // (Theirs, as written, before any copy of theirs is placed below: what this tab has in line with them from now on,
  // its own changes still its own, see the end.)
  const theirArrangement = saving ? JSON.stringify([orderOf(stored.order), folders, incoming.map((doc) => [doc.id, doc.folder ?? ""])]) : "";
  /** A chart's later copy, in its folder as the arrangement has it. @param {Chart | undefined} own @param {Chart | undefined} other */
  const later = (own, other) => {
    if (!own || !other) return /** @type {Chart} */ (own ?? other);
    const doc = own.updated >= other.updated ? own : other, by = saving && !(placedThen.has(own.id) && placedThen.get(own.id) === (own.folder ?? "")) ? own : other;
    if (by.folder) doc.folder = by.folder;
    else delete doc.folder;
    return doc;
  };
  // In the arrangement's order; the other's alone first (new there: like every new chart, they start the list).
  const [lead, others] = saving ? [library.charts, incoming] : [incoming, library.charts], leading = new Set(lead.map((doc) => doc.id));
  const merged = [...others.filter((doc) => !leading.has(doc.id)), ...lead.map((doc) => later(ours.get(doc.id), theirs.get(doc.id)))];
  for (const doc of merged) if (ours.get(doc.id) !== doc) histories.delete(doc.id);
  // (The open chart, whichever copy, stays.)
  library.charts = merged.filter(alive);
  // (Saving here: this tab's folders, but each one's name, icon, place and closing theirs unless changed here since
  // this tab last wrote or took one in: a folder renamed or moved there isn't saved back as it was.)
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
  // (The order, theirs too unless changed here since.)
  if (!saving || (arranged && JSON.stringify(library.order) === JSON.stringify(arranged[0]))) library.order = orderOf(stored.order);
  // The presets as edited: each preset's latest copy, here or there (one deleted, gone; the open one stays).
  const theirPresets = presetsOf(stored.presets), ourPresets = new Set(library.presets), byPreset = /** @type {Map<string, Chart>} */ (new Map());
  for (const doc of [...library.presets, ...theirPresets]) {
    const have = byPreset.get(/** @type {string} */ (doc.preset));
    if (!have || doc.updated > have.updated) byPreset.set(/** @type {string} */ (doc.preset), doc);
  }
  library.presets = [...byPreset.values()].filter(alive);
  for (const doc of library.presets) if (!ourPresets.has(doc)) histories.delete(doc.id);
  // (Its charts this tab can't read, set aside: not counted, or each tab would save over the other's.)
  if (!saving && !writing && holdings(library.charts, library.folders, library.deleted, library.presets) !== holdings(incoming, folders, deleted, theirPresets)) save();
  // (Taken in, theirs is the arrangement as last written; saving here, its changes kept over theirs, the baseline
  // theirs all the same: its changes still told apart from it, what it took of theirs not its own at a later take-in.)
  arrangedAs = saving ? theirArrangement : arrangedNow();
  // (Still starting, a link being read: the library's taken, and start shows what's open once it's ready.)
  if (!chart) return;
  const current = isPreview(chart) ? chart : library.charts.find((doc) => doc.id === chart.id) ?? library.presets.find((doc) => doc.id === chart.id || (chart.preset !== undefined && doc.preset === chart.preset)) ?? chart;
  drawThumbs();
  if (isOpen($("palette"))) showPalette();
  // (A formula's draft is the inspector's, none before the rest is in.)
  if (current === chart) return (withRest(() => inspector.pruneDrafts()), renderCharts());
  chart = /** @type {Chart} */ (current);
  withRest(() => inspector.pruneDrafts());
  // (Its steps were of the chart as it was here: let go of, said (they'd take back the other tab's edit too).)
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
 * `run` as the page is left; by touch, as it's hidden too (`hidden`): a phone closes an app put away when it likes,
 * with no word to the page (a computer's tab put away runs on, a field being typed in still open as it's back).
 * @param {(hidden: boolean) => void} run
 */
function onLeave(run) {
  addEventListener("pagehide", () => run(false));
  if (TOUCH) document.addEventListener("visibilitychange", () => document.visibilityState === "hidden" && run(true));
}
/**
 * Fields left as the window was (a phone's app switched, another app in front), each with how it ends as it's left:
 * the window back, ended so then unless the focus came back to it (as it does, usually), a frame on.
 * @type {Map<Element, () => void>}
 */
const leftOpen = new Map();
/** Whether `field`'s focus went with the window: if so, `end` waits for its return (see leftOpen). */
const leftWith = (/** @type {Element} */ field, /** @type {() => void} */ end) => !document.hasFocus() && Boolean(leftOpen.set(field, end));
addEventListener("focus", (event) => event.target === window && requestAnimationFrame(() => {
  const ends = [...leftOpen].filter(([field]) => document.activeElement !== field);
  leftOpen.clear();
  for (const [, end] of ends) end();
}));
/**
 * A write done at the page's next idle moment (`timeout`, ms, at most; 0, whenever it is), once for all asked for
 * till then, never put off by more; the page left (or hidden) first, written then. @param {() => void} write
 * @returns {(() => void) & { cancel: () => void }} asking for it
 */
function writeSoon(write, timeout = 0) {
  let due = false;
  const now = () => due && ((due = false), write());
  onLeave(now);
  // (`cancel`: written another way meanwhile, the one waiting isn't.)
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

/**
 * Every edit goes through here: undoable, normalized, saved, redrawn. Edits sharing a `coalesce` key within a second
 * (typing) undo as one; `quiet`, a change of view (range, pane sizes), is saved without the "Saved" note; `look`, a
 * way of looking at what's there (a scale, an interval, the legend, layers hidden), keeps no chart started here
 * (only making something does: a layer, a style, a drawing, a title; another's preview is kept by Keep alone);
 * `keepMenu`, made from a menu that stays (its slider), the menu stays open. `view`, a view's step (a range): undone
 * as a step, but no edit (nothing stamped, a preview untouched).
 * @param {(doc: Chart) => void} edit @param {EditOptions} [options]
 */
function commit(edit, /** @type {EditOptions} */ options) {
  return editChart(chart, edit, options);
}
/**
 * An edit of any chart, as its own step (see commit): the open one's, whichever page shows; another's (from its row,
 * its tile, ⌘K) kept with its steps (see keepHistory), shown in the lists.
 * @typedef {{ coalesce?: string, quiet?: boolean, look?: boolean, keepMenu?: boolean, view?: boolean }} EditOptions
 * @param {Chart} doc @param {(doc: Chart) => void} edit @param {EditOptions} [options] @returns {boolean} whether it changed anything
 */
function editChart(doc, edit, { coalesce = "", quiet = false, look = false, keepMenu = false, view = false } = {}) {
  const open = doc === chart;
  // (The open one's from the Charts page in reach there, where the page's own is the Library's.)
  if (!open || home) reached(doc);
  const before = JSON.stringify(doc), sources = new Map(doc.layers.map((layer) => [layer.id, layer.series ?? layer.formula]));
  edit(doc);
  // (A layer's interval ceiling is its source's (see maxInterval): its source changed, let go of.)
  for (const layer of doc.layers) if (layer.maxInterval && sources.get(layer.id) !== (layer.series ?? layer.formula)) delete layer.maxInterval;
  normalize(doc);
  // (Something in it at last: a chart, not a blank one New chart made, for good.)
  if (doc.blank && (doc.layers.length || doc.title || doc.drawings?.length)) delete doc.blank;
  if (JSON.stringify(doc) === before) return false;
  const now = Date.now(), steps = open ? edits : histories.get(doc.id) ?? { past: [], future: [] };
  // One step: a drag's edits, however long it lasts (the next drag, another); typed ones a second apart. (Joining
  // only a step there's still: the history let go meanwhile (another tab's edit taken in), it starts one anew.)
  // (Nor across another step done since, in any history: one undo takes back one thing.)
  const joins = open && coalesce && edits.past.at(-1)?.at === stepCount && coalesce === edits.key && (holding ? edits.held === holding : !edits.held && now - edits.at < 1000);
  // (Joined, it's done now all the same: the newest, from where it began.)
  recordStep(steps, joins ? /** @type {ChartStep} */ (edits.past.pop()).state : before);
  // (A view's step (a range) is a step, not an edit: nothing stamped, a preview untouched, as a pan.)
  if (!view) stampEdit(doc);
  if (!view && isPreview(doc)) touchPreview(doc);
  if (!open) return keepHistory(doc.id, steps), offPage(), true;
  const kept = edits.past.length;
  if (edits.past.length > UNDO_STEPS) edits.past.shift();
  trimSteps(edits, UNDO_CHARS);
  // (Its oldest steps let go: the one that kept a chart started here may be among them, kept for good then.)
  if (edits.past.length < kept) keptByEdit.delete(chart);
  edits.key = coalesce;
  edits.at = now;
  edits.held = holding;
  if (isPreview(chart) && drafts.has(chart) && !quiet && !look) (keepPreview(false), keptByEdit.set(chart, edits.past.length));
  changed(!quiet, keepMenu);
  return true;
}
/**
 * The charts not open whose steps were taken from this page (renamed from a row, reset from a menu; the open one's,
 * from the Charts page), each with when it came into reach: its steps done since are in reach of ⌘Z here, as the
 * page's own history is (see historyFor), till another page is shown (`reachAt`, see navigations, placeHome).
 * @type {Map<Chart, number>}
 */
const inReach = new Map();
let reachAt = -1;
/** A chart stepped in from this page, as it's about to be (see inReach). @param {Chart} doc */
function reached(doc) {
  if (reachAt !== navigations) inReach.clear(), (reachAt = navigations);
  if (!inReach.has(doc)) inReach.set(doc, stepCount);
}
/**
 * The histories in reach (see historyFor), each with the step it's in reach from: the page's own (a chart's; the
 * Charts page's, the Library's) and the Library's from the first, the charts stepped in from this page (see inReach)
 * from then. (A chart that isn't here any more, deleted or taken by another tab's copy, has no steps to take.)
 */
function reachable() {
  const all = new Map([[home ? null : chart, 0], [null, 0]]);
  if (reachAt === navigations) for (const [doc, from] of inReach) if (!all.has(doc) && (doc === chart || chartById(doc.id) === doc)) all.set(doc, from);
  return all;
}
/** A history's steps: the Library's (null), the open chart's, another's kept (see histories). */
const stepsOf = (/** @type {Chart | null} */ doc) => (!doc ? libraryEdits : doc === chart ? edits : histories.get(doc.id));
/**
 * A step done in a history (`state`, as JSON, the chart's before it), now: the newest. One line of steps across what's
 * in reach (see reachable), as in one document: what's done anew lets go of every redo in reach.
 * @param {{ past: ChartStep[], future: ChartStep[] }} steps @param {string} state
 */
function recordStep(steps, state) {
  steps.past.push({ state, at: ++stepCount });
  steps.future = [];
  dropRedos();
}
/**
 * Every redo in reach let go of (something new done: see recordStep, libraryStep); a chart's undone before it came
 * into reach kept (a redo's undone later the higher up it is: see historyFor).
 */
function dropRedos() {
  for (const [doc, from] of reachable()) {
    const future = stepsOf(doc)?.future;
    while (future && (future.at(-1)?.at ?? 0) > from) future.pop();
  }
}
/**
 * The history ⌘Z (`back`) or ⇧⌘Z takes a step in: of those in reach (see reachable), the one whose next step moved
 * last (done or redone; undone, for ⇧⌘Z). Never another chart's: ⌘Z in one chart doesn't change another. null, the
 * Library's; undefined, none.
 */
function historyFor(/** @type {boolean} */ back) {
  let found, at = 0;
  for (const [doc, from] of reachable()) {
    const step = stepsOf(doc)?.[back ? "past" : "future"].at(-1);
    if (step && step.at > from && step.at > at) (found = doc), (at = step.at);
  }
  return found;
}
/** Undo, as ⌘Z and ⌘K do it (see historyFor). */
function undoLast() {
  // (A path being placed: its last point taken back first, as ⌫ does.)
  if (!home && unplacePath()) return;
  takeStep(true);
}
/** Redo, as ⇧⌘Z does it (see historyFor). */
function redoLast() {
  // (A path being placed: the last point taken back from it put back first; none, nothing (the chart's redo would
  // let it go).)
  if (!home && (replacePath() || placingPath())) return;
  takeStep(false);
}
/** ⌘Z's step (`back`) or ⇧⌘Z's: what's being typed, and a color picked, kept first, a step of their own read with the rest. */
function takeStep(/** @type {boolean} */ back) {
  leaveField();
  inspector.flushPick();
  travelIn(historyFor(back), back);
}
/** A step back (`back`) or forth in a history: the Library's (null), the open chart's, another's as it lies. */
function travelIn(/** @type {Chart | null | undefined} */ doc, /** @type {boolean} */ back) {
  if (doc === undefined) return;
  if (!doc) return back ? undoLibrary() : redoLibrary();
  if (doc === chart) return back ? undo() : redo();
  const steps = /** @type {{ past: ChartStep[], future: ChartStep[] }} */ (stepsOf(doc)), now = JSON.stringify(doc);
  if (!putState(doc, moveStep(steps, back, now).state, now) && isPreview(doc)) asideDirty.add(doc);
  keepHistory(doc.id, steps);
  offPage();
  // (Said where it shows, in place of a toast of its step: the chart it changed isn't on screen.)
  toast(`${back ? "Undone" : "Redone"} in “${titleOf(doc)}”`);
}
/** A chart not open changed by its own step (an edit, an undo): saved, shown in the lists and the buttons. */
function offPage() {
  save();
  renderCharts();
  syncUndo();
}
/**
 * A chart's step taken back (`back`) or forth: off its side, the state it leaves (`now`) onto the other, moved now
 * (see historyFor), the tool its drawing was made with going with it (see drawnSteps).
 * @returns {{ state: string, drawn: { kind: DrawingKind, picks: number } | undefined }}
 */
function moveStep(/** @type {{ past: ChartStep[], future: ChartStep[] }} */ steps, /** @type {boolean} */ back, /** @type {string} */ now) {
  const { state } = /** @type {ChartStep} */ (steps[back ? "past" : "future"].pop());
  steps[back ? "future" : "past"].push({ state: now, at: ++stepCount });
  const drawn = heatmaps.drawnSteps.get(state);
  heatmaps.drawnSteps.delete(state);
  if (drawn) heatmaps.noteDrawn(now, drawn);
  return { state, drawn };
}
/**
 * A chart's steps kept while it isn't open: the last few charts', newest last, each within its budget; those in reach
 * (see inReach) all kept, as many as they are.
 */
function keepHistory(/** @type {string} */ id, /** @type {{ past: ChartStep[], future: ChartStep[] }} */ steps) {
  steps.past = steps.past.slice(-KEPT_STEPS);
  steps.future = steps.future.slice(-KEPT_STEPS);
  trimSteps(steps, KEPT_CHARS);
  histories.delete(id);
  histories.set(id, steps);
  const held = new Set([...reachable().keys()].map((doc) => doc?.id));
  for (const old of histories.keys()) if (histories.size > KEPT_HISTORIES && !held.has(old)) histories.delete(old);
}
/**
 * A chart put back to one of its states (`state`, an undo's or a redo's; `now`, as it is): all of it as it was then,
 * but its id, folder and blank (the chart's own now), and its range, a view as a pan is, unless the step was the
 * range alone (see setRange); stamped as an edit unless so. @returns {boolean} whether the step was the range alone
 */
function putState(/** @type {Chart} */ doc, /** @type {string} */ state, /** @type {string} */ now) {
  const was = JSON.parse(state), folder = doc.folder, stamped = doc.updated, blank = doc.blank;
  const bare = (/** @type {Record<string, any>} */ item) => JSON.stringify(Object.keys(item).filter((key) => !["range", "updated", "folder", "id", "blank"].includes(key)).sort().map((key) => [key, item[key]]));
  const rangeOnly = was.range !== doc.range && bare(was) === bare(JSON.parse(now)), shapes = new Map(doc.layers.map((layer) => [layer.id, shapeKey(layer)]));
  for (const key of Object.keys(doc)) if (!(key in was)) delete /** @type {any} */ (doc)[key];
  Object.assign(doc, was, { range: rangeOnly ? was.range : doc.range, id: doc.id });
  if (folder) doc.folder = folder;
  else delete doc.folder;
  // (Once something's been in it, a chart for good: not blank again, whichever step.)
  if (!blank) delete doc.blank;
  // (Stamped past the copy as it was: this step is its latest edit; a range's, a view's, nothing stamped, as when set.)
  doc.updated = stamped;
  if (!rangeOnly) stampEdit(doc);
  // (The looks remembered by the layers whose shape this changed (a type, a heatmap's P&L) let go of: they were the
  // steps' since, not this one's.)
  for (const layer of doc.layers) if (shapes.get(layer.id) !== shapeKey(layer)) looksLeft.delete(`${doc.id}:${layer.id}`);
  return rangeOnly;
}
/** The open chart's step taken back (`back`) or forth, whichever page shows. */
function travel(/** @type {boolean} */ back) {
  // A field being typed into is kept first (a step of its own: the steps are read after it), a color picked and
  // waiting for its frame too; what's being drawn or moved, of the chart as it is, goes.
  leaveField();
  inspector.flushPick();
  // (Nothing to go to: what's being drawn stays as it is.)
  if (!edits[back ? "past" : "future"].length) return;
  stopDrawing();
  pathResume = null;
  // The chart as it was, all of it (what it didn't have then, it hasn't now; see putState).
  const now = JSON.stringify(chart), { state, drawn } = moveStep(edits, back, now);
  const ranged = putState(chart, state, now);
  // (A preview's step, kept aside as its edits are: reloaded, it's as it is now.)
  if (isPreview(chart) && !ranged) asideDirty.add(chart);
  edits.key = "";
  // (A toast of the chart's step said what's now taken back or done again, or something before it: no longer true.)
  if (toastOf === chart) hideToast();
  // The step that kept a chart started here undone: a draft again, not one of your charts (redone, kept again).
  const keptAt = keptByEdit.get(chart);
  if (keptAt !== undefined && back && edits.past.length === keptAt - 1 && library.charts.includes(chart)) {
    // (Its row gone with it (a draft isn't listed): the focus on it, to New chart, then the plot.)
    keepFocus(null, () => $("new-chart"), () => $("plot-keys"));
    discard(chart);
    addPreview(chart);
    save(false);
    renderCharts();
    clearLink();
  } else if (keptAt !== undefined && !back && edits.past.length === keptAt && isPreview(chart)) keepPreview(false);
  changed(true);
  // (A drawing a tool made once (it was put down then), undone: that tool in hand again, to try again, unless one's
  // been picked since; redone, down again if an undo took it up. Under the Charts page, no tool's in hand.)
  if (drawn && !home && back && (undoTook || picks === drawn.picks)) (takeUp(drawn.kind, false, true), (undoTook = drawn.kind));
  else if (drawn && !home && !back && undoTook === drawn.kind && tool === drawn.kind) takeUp("", false, true);
  // (Said, for a screen reader: nothing else on screen says it.)
  say(back ? "Undone" : "Redone");
}
/** The open chart changed (an edit, an undo): its menu closes, a selection gone with it too; saved, shown. */
function changed(announce, keepMenu = false) {
  // (A preview's changes aren't saved: no "Saved" for them.)
  announce &&= !isPreview(chart);
  revision++;
  if (!chartChanged) (chartChanged = true), queueMicrotask(() => (chartChanged = false));
  if (!keepMenu) menus.hideMenu();
  if (!layerById(selected)) selected = "";
  // (A formula left unfinished over one that's since changed (⌘K, an undo, the layer gone): let go for good.)
  inspector.pruneDrafts();
  chosen = new Set([...chosen].filter((id) => pen.drawingById(id)));
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
/** Edits made while a drag held: the page drawn again when it ends. */
let heldEdits = false;
const undo = () => travel(true);
const redo = () => travel(false);

/** The chart to go back to from this one: the last one opened before it. */
const previousChart = () => viewed().map(chartById).find((doc) => doc && doc !== chart) ?? library.charts.find((doc) => doc !== chart) ?? previewCharts.find((doc) => doc !== chart) ?? null;
/** The one opened last before this one, if it's a preset left as it was (let go of: its index, opened as it's made). */
const previousPreset = () => {
  const id = viewed().find((other) => other !== chart?.id);
  return id && !chartById(id) ? presetById(id) : -1;
};
/**
 * Where each chart was left, zoomed or panned off its preset, kept across reloads: each edge of the view as a point's
 * time and how many bars it lies from that point, so it holds as new points come in, and past either end of the data.
 * On its preset, a chart keeps nothing: the preset follows the newest data.
 */
const VIEWS = "bitview-studio:views";
/**
 * The views left, as stored: read once (a grid of tiles asks for each, at each key typed), written through; another
 * tab's write, read again. `viewsBase`: as read, to tell another tab's writes since from this one's. `viewsUnkept`: one
 * not written, said (once till one is).
 */
let viewsLeft = /** @type {Record<string, any> | null} */ (null), viewsBase = /** @type {Record<string, any>} */ ({}), viewsUnkept = false;
const viewsNow = () => viewsLeft ?? ((viewsBase = storage.get(VIEWS, {})), (viewsLeft = { ...viewsBase }));
/**
 * The views written. (Charts this tab doesn't know may be another's, its previews and presets: their views kept;
 * only past a hundred, the oldest left go.)
 * @param {Record<string, any>} all
 */
const putViews = (all) => {
  // (Another tab's writes since this one read them, taken in a chart's view at a time: changed there alone, theirs
  // (gone there, gone); here alone, this one's; in both, the later (a view left over one let go of).)
  const stored = storage.get(VIEWS, {}), at = (/** @type {any} */ view) => (view === undefined ? -Infinity : view?.at ?? 0);
  for (const id of new Set([...Object.keys(stored), ...Object.keys(viewsBase)])) {
    const theirs = Object.hasOwn(stored, id) ? stored[id] : undefined, base = Object.hasOwn(viewsBase, id) ? viewsBase[id] : undefined;
    if ((theirs === undefined) === (base === undefined) && at(theirs) === at(base)) continue;
    const ours = Object.hasOwn(all, id) ? all[id] : undefined;
    if (ours === base || at(theirs) > at(ours)) theirs === undefined ? delete all[id] : (all[id] = theirs);
    // (The base as storage is now, whichever won: written or not, what's theirs isn't this one's own.)
    theirs === undefined ? delete viewsBase[id] : (viewsBase[id] = theirs);
  }
  const ids = Object.keys(all);
  if (ids.length > 100) for (const id of ids.sort((a, b) => (all[a]?.at ?? 0) - (all[b]?.at ?? 0)).slice(0, ids.length - 100)) delete all[id];
  // (Storage full: the caches let go of first, then again; not written, said once till one is.)
  if (storage.set(VIEWS, all) || (storage.drop(PREVIEWS, THUMBS, PRESET_MARKS, SIDE_PRESETS), storage.set(VIEWS, all))) (viewsBase = { ...all }), (viewsUnkept = false);
  else if (!viewsUnkept) (viewsUnkept = true), notice("Can't remember where each chart was left: this browser's storage is full or blocked");
};
// (Another tab's linked view, taken here at once out of sight: shown already as it's turned to (see followLinked).)
addEventListener("storage", (event) => event.key === VIEWS && ((viewsLeft = null), document.visibilityState !== "visible" && followLinked()));
// (Back from the browser's history, others' writes since unheard: read again.)
addEventListener("pageshow", (event) => event.persisted && (viewsLeft = null));
const isEdge = (edge) => Array.isArray(edge) && edge.length === 2 && edge.every(Number.isFinite) && Math.abs(edge[0]) < 1e11;
/** A view that reads as dates: at a known interval, its edges (their bars counted) within the calendar. */
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
/**
 * Left at the latest, `latest` is its edges by the bars from the last (its right edge on it or past it).
 * @returns {{ index: string, from: [number, number], to: [number, number], latest?: [number, number] } | null}
 */
function leftView(id) {
  const left = viewsNow()[id];
  // (Its time kept aside: the view alone, as a link or a picture's key takes it.)
  return isView(left) ? (({ at, ...view }) => view)(left) : null;
}
/** Where the linked view is kept, among the charts' own (no chart's id). */
const LINKED = "*";
/** A chart's span of chunks at an interval, by the view it's made for: its own, or the linked one's (never mixed). */
const spanKey = (/** @type {string} */ id, /** @type {string} */ index, linked = viewKey(id) === LINKED) => `${id}@${index}${linked ? "@linked" : ""}`;
/**
 * Where the open chart was last in line with the linked view (opened on it, following it, or saying it), by its edges'
 * dates (as kept: chunks loaded before it move its points, not them): it says the linked view again only once it's
 * moved from there (see storeView), never what it showed meanwhile (a chart at minutes or hours left where it was, a
 * window it shows wider). @type {any}
 */
let linkedAt = null;
/** The linked view this page last took or said, by when it was said (its `at`): another said since, it takes it (see followLinked). */
let linkedTook = 0;
/**
 * A hand on the page (a pointer, the wheel, a key) since it last took or said the linked view: only then does it say
 * it (see storeView). A chart laid out, resized or taking it moves its view too, as no one's move.
 */
let handled = false;
/** A hand on the page since it started, ever (nothing done by its hand yet: what it shows can be put right unseen). */
let handledYet = false;
/** The pointers held down (fingers, by their ids): a hand on the page till the last is let go (said halfway, a drag held on is still one). */
const handsHeld = new Set();
/** What waits for every hand to be let go (see afterPress). @type {(() => void)[]} */
const afterHands = [];
for (const type of ["pointerdown", "wheel", "keydown"]) addEventListener(type, (event) => {
  handled = handledYet = true;
  if (!(event instanceof PointerEvent)) return;
  const id = event.pointerId;
  handsHeld.add(id);
  press(event, { end: () => handsHeld.delete(id) && !handsHeld.size && afterHands.splice(0).forEach((run) => setTimeout(run)) });
}, { capture: true, passive: true });
/**
 * The linked view the open chart is drawn again to take (see followLinked), as it was when taken: its chunks and its
 * view both for it, one said meanwhile taken next; its chart's, while it's linked (by date). @type {{ view: any, at: number, id: string } | null}
 */
let retaking = null;
/** Two views at the same dates: their edges where they fall in time (however each is written), within a hundredth of a bar. @param {any} a @param {any} b */
const sameEdges = (a, b) => Boolean(a && b && a.index === b.index) && ((x, y, bar) => Math.abs(x.start - y.start) < bar && Math.abs(x.end - y.end) < bar)(viewTimes(a), viewTimes(b), (INTERVAL_DAYS[a.index] * 86400) / 100);
/** The open chart's span moved to where it's kept now (its view the same through a link's change, see linkView): one span, one key. */
function carrySpan() {
  if (!chart) return;
  const from = spanKey(chart.id, chart.index, !prefs.linkView), span = spans.get(from);
  if (!span) return;
  spans.delete(from);
  spans.set(spanKey(chart.id, chart.index), span);
}
/** Where a chart's view is kept: its own, but the open chart's every chart's while the view is linked (see linkView). */
const viewKey = (/** @type {string} */ id) => (prefs.linkView && id === chart?.id ? LINKED : id);
/** The view a chart opens on (see viewKey). */
const openView = (/** @type {Chart} */ doc) => leftView(viewKey(doc.id));
/**
 * The chart a link (a script, a file; read at the start too) just brought, to open at its own window: the view it came with, else its
 * range, whatever view is linked (you: "a link's own range should win"); the linked one follows it once it's moved.
 */
let linkWindow = "";
/** The view the open chart opens on now: a link's own (see linkWindow); the linked one it's taking, as taken (see followLinked); else its own. */
const openingView = (/** @type {Chart} */ doc) => (linkWindow === doc.id ? leftView(doc.id) : retaking && doc.id === retaking.id && viewKey(doc.id) === LINKED ? retaking.view : openView(doc));
/** An edge of the view as the time of the point nearest it, and how many bars it lies from that point. */
function edgeOf(logical, times) {
  const i = clamp(Math.round(logical), 0, times.length - 1);
  return [times[i], logical - i];
}
/** The most bars the chart shows at once: four a pixel (its least bar spacing), however many there are. */
const mostBars = () => (engine.time.width ? engine.time.width / engine.time.minSpacing : Infinity);
/**
 * A window of the view, no wider than the chart shows (wider, it would fold the view to a sliver): a wider one keeps
 * its middle, or the end `keep` names ("to", its latest; "from").
 * @param {{ from: number, to: number }} range @param {"middle" | "from" | "to"} [keep]
 */
function within({ from, to }, keep = "middle", most = mostBars()) {
  const over = to - from - most;
  if (over <= 0) return { from, to };
  return keep === "to" ? { from: from + over, to } : keep === "from" ? { from, to: to - over } : { from: from + over / 2, to: to - over / 2 };
}
/**
 * A view as the chart holds it on its data (the library's rule: see Timeline.held): moved back on the side it's
 * past; null, on it already. For where a move is going (a glide, a jump): the chart holds the steps there, not past.
 * @param {{ from: number, to: number }} range
 */
function ontoData(range) {
  const shift = engine.time.length ? engine.time.held(range.from, range.to) : 0;
  return shift ? { from: range.from + shift, to: range.to + shift } : null;
}
/** An edge back on the chart: the bar holding its time (past the data, counted by the calendar), and the bars from there. */
const logicalOf = (/** @type {[number, number]} */ [time, bars], /** @type {number[]} */ times, /** @type {string} */ index) => (time > /** @type {number} */ (times.at(-1)) ? times.length - 1 + indexAt(index, time) - indexAt(index, /** @type {number} */ (times.at(-1))) : Math.max(0, lowerBound(times, time + 1) - 1)) + bars;
/**
 * A view kept for a chart, on its points at `index`: as kept, at the interval it was kept at; at another, by the
 * moments its edges stand at (see viewTimes), so it shows the same stretch of history, each moment where it was on
 * screen: a bar at its own time (as the time axis labels it), a moment between two bars as far between them (a
 * date both intervals have, at the same place: its label doesn't move).
 */
function viewOn(left, index, times, gap = rightGap()) {
  // (Left at the latest (its right edge on the last bar, or past it): at the latest again, bars come since or not,
  // as wide, as far past it. At its interval, by its bars from the last (its times may be before the chunks loaded
  // now); at another, placed by the calendar: up to the last.)
  if (left.latest && left.index === index) return { from: times.length - 1 + left.latest[0], to: times.length - 1 + left.latest[1] };
  const view = viewAsLeft(left, index, times, gap), shift = left.latest ? Math.max(0, times.length - 1 - view.to) : 0;
  return shift > 0 ? { from: view.from + shift, to: view.to + shift } : view;
}
/** A kept view as it was left, on the points at `index` (see viewOn), `gap` its room after the latest point. */
function viewAsLeft(left, index, times, gap = rightGap()) {
  if (left.index === index) return { from: logicalOf(left.from, times, index), to: logicalOf(left.to, times, index) };
  const { start, end } = viewTimes(left), last = /** @type {number} */ (times.at(-1)), lastIndex = indexAt(index, last);
  // A moment's place: on the data, between the point at or before it and the next; past it, counted by the calendar
  // (before it, by its bars' length: the room before the data kept, as after it).
  const placeOf = (/** @type {number} */ time) => {
    if (time < times[0]) return (time - times[0]) / (INTERVAL_DAYS[index] * 86400);
    if (time >= last) {
      const k = indexAt(index, time), from = timeAt(index, k);
      return times.length - 1 + k - lastIndex + (time - from) / (timeAt(index, k + 1) - from);
    }
    // (The last point at or before it, a moment a fraction of a second before the last's included: one before it.)
    const at = lowerBound(times, time), i = times[at] === time ? at : at - 1;
    return i + (time - times[i]) / (times[i + 1] - times[i]);
  };
  // No further past the data than the chart's own room on the right (at the view's own spacing); only a few bars
  // here (two days, at a week): at least one bar, ending where it did.
  // (Over as many bars as the chart shows at most; no chart to measure, none past the data.)
  const plot = engine.time.width, bars = Math.min(mostBars(), Math.max(1, placeOf(end) - placeOf(start)));
  const room = plot > gap ? (gap * bars) / (plot - gap) : 0;
  const to = Math.min(placeOf(end), times.length - 1 + room), from = Math.min(placeOf(start), to - 1);
  return { from, to };
}
/**
 * A kept view's stretch of time, at its own interval: the moments its edges stand at (an edge at a bar, the bar's
 * own time, as the time axis labels it; between two, as far between their times).
 */
function viewTimes(left) {
  // (A bar before the calendar's first, by its length from there.)
  const timeOf = (/** @type {number} */ i) => (i >= 0 ? timeAt(left.index, i) : timeAt(left.index, 0) + i * INTERVAL_DAYS[left.index] * 86400);
  const at = (/** @type {[number, number]} */ [time, bars]) => {
    const place = indexAt(left.index, time) + bars, i = Math.floor(place);
    return timeOf(i) + (place - i) * (timeOf(i + 1) - timeOf(i));
  };
  return { start: at(left.from), end: at(left.to) };
}
/**
 * The view linked or not: on, every chart opens on the same dates, and moving the view (a pan, a zoom, a range, the
 * navigator) moves it for all: each takes it as it opens, a tab as it comes in sight (never one in sight as it's
 * looked at); only the dates (each its own interval). It starts as the open chart shows. Off, the open chart stays
 * where it is (its own now), the others on their own.
 */
function linkView(/** @type {boolean} */ on) {
  if (on === prefs.linkView) return;
  prefs.linkView = on;
  savePrefs();
  carrySpan();
  // (Kept where it is now: as every chart's, or, off, as its own.)
  linkedAt = null;
  storeView(false);
  renderLinkView();
  toast(on ? "Every chart shows these dates, in every tab" : "Each chart keeps its own view");
}
function renderLinkView() {
  $("link-view").setAttribute("aria-pressed", String(prefs.linkView));
}
/**
 * The linked view said by another tab: the open chart takes it, drawn again as it opens (its chunks for it, at minutes
 * or hours), as every interval does; out of sight at once (it says nothing back: see storeView), else as it comes in
 * sight. Never as it's looked at: what's on screen moves only by a hand on it.
 */
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
// (A window beside another, both in sight: as it's turned to.)
addEventListener("focus", followLinked);
/** Keeps where the open chart is, at the page's next idle moment (a second at most) as it moves. */
const keepView = writeSoon(() => saveView(), 1000);
/** The view kept, as it shows: the chart on screen, at the interval it's drawn at (see storeView). */
function saveView() {
  storeView(false);
  // (Mid-gesture, the bar waits for the view to rest: Safari allows only so many writes.)
  if (!viewMoving) showAddress();
}
/**
 * The view kept, as it shows. Mid-switch, it still shows the interval before (the new one's data on its way): kept
 * then only `asDrawn`, by the switch's draw as it lands, so the view moved meanwhile is the one it shows.
 * @param {boolean} asDrawn
 */
/**
 * A view as it's kept, from a range on the points `times`: its edges each the point nearest it and the bars from it,
 * at the latest its bars from the last (at the latest only where its data is: a fine interval's chunks loaded may
 * end in the past). @param {{ from: number, to: number }} range @param {number[]} times
 */
const viewRecord = (range, times, index = shownIndex()) => ({ index, from: edgeOf(range.from, times), to: edgeOf(range.to, times), ...(range.to > times.length - 1.5 && resolved?.live ? { latest: [range.from - (times.length - 1), range.to - (times.length - 1)] } : {}) });
/** The open chart's view as it shows (none while it shows its range, the range said instead), as a link carries it. */
function shownView() {
  const range = shownSpan(), times = resolved?.times;
  return chartShown() && range && times?.length && !presetShown ? viewRecord(range, times) : null;
}
/** Whether the page has been in sight (see storeView). */
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
  // (By the switch's draw, mid-glide (a zoom asked for meanwhile), where it's going: the draw ends the glide.)
  const range = (asDrawn && engine.controller.target) || shownSpan();
  // (Linked, every chart's, a range's too: the one each opens on; said only once it's moved since it was in line with
  // it (see linkedAt).)
  const all = viewsNow(), key = viewKey(chart.id), shown = range ? viewRecord(range, times) : null;
  if (key === LINKED && (!handled || sameEdges(shown, linkedAt))) return;
  if (key === LINKED) (linkedAt = shown), (handled = handsHeld.size > 0);
  if ((presetShown && key !== LINKED) || !shown) delete all[key];
  else all[key] = { ...shown, at: Date.now() };
  if (key === LINKED && all[key]) linkedTook = all[key].at;
  putViews(all);
}
onLeave(() => storeView(false, true));
/**
 * Charts opened, and the grid (a link, a script, a click): a picture made meanwhile, of the one before, dropped. The
 * chart open opened again (a preset made anew from the presets as they are now, its id the same) is no other.
 */
let navigations = 0;
/** A block came while the Charts page was over the chart (see checkLive). */
let blockAtHome = false;
/** The chart under the grid not drawn yet (a start onto the grid, see redrawChart): drawn as it's opened, as any other is (see leaveHome). */
let undrawn = false;
/** @param {Chart} doc */
function openChart(doc) {
  if (doc.id !== chart?.id) navigations++;
  // (Another opened before a link's is drawn: that one at its own window no more.)
  if (linkWindow !== doc.id) linkWindow = "";
  // (Opened afresh: a block come meanwhile is in its data already.)
  blockAtHome = false;
  lastOpen = "";
  // (Another chart opened over Quick Look (⌘K's pick, a link, a script): the look's put away first.)
  quickHide();
  // (Left, a chart started here is kept for good: undoing its first edit makes it a draft only while it's still on screen.
  // Left a draft again, it keeps that step's place: brought back to redo it (see redoLibrary), kept again.)
  if (chart && chart !== doc && !isPreview(chart)) keptByEdit.delete(chart);
  gestures.stopCarrying();
  // What's being typed is kept, on the chart it was typed on.
  leaveField();
  saveView();
  // This tab's place, for a reload and for Back: a chart opened from another, a step of its own (Back, that one again,
  // where it was left; Forward, this one). Come back to by Back or Forward, its step is the one there.
  if (history.state?.chart && history.state.chart !== doc.id) history.pushState({ chart: doc.id }, "");
  // (An empty preview left has nothing to come back to (nor kept aside); a link that couldn't be read, left behind.)
  for (const item of previewCharts) if (item !== doc && !item.layers.length) (dropPreview(item), forgetPreview(item));
  brokenLink = "";
  const previous = chart;
  chart = doc;
  // (One deleted in another tab while it was open: gone now it isn't.)
  library.charts = library.charts.filter(alive);
  library.presets = library.presets.filter(alive);
  linkedAt = null;
  // (A preset left as it was: nothing of it kept but for Recent; a chart New chart made, left as it came, let go of.)
  if (previous?.preset !== undefined && previous !== doc) prunePresets(doc);
  if (previous && previous !== doc && emptyYours(previous) && library.charts.includes(previous)) dropEmpty(previous);
  library.active = doc.id;
  rememberViewed(doc.id);
  selected = "";
  chosen = new Set();
  layerPicks.clear();
  drawingsHidden = false;
  // (Its undo and redo, as left; the one before's kept for when it's back, if it has any.)
  // (The same chart under its id again (a copy of it taken in): its steps go on.)
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
/** Leaves a new chart, empty, for the one before it: it's simply taken back (nothing is lost). */
function leaveEmpty() {
  // (A new chart of yours taken back so: said, with an Undo, once it's gone; see dropEmpty.)
  const leaving = emptyYours(chart) ? { doc: chart, at: library.charts.indexOf(chart), folder: chart.folder } : null;
  if (backToHome()) {
    cancelling = chart.id;
    cancelledNew = leaving;
    return history.back();
  }
  const back = previousChart(), preset = previousPreset();
  if (!(isPreview(chart) || emptyYours(chart)) || chart.layers.length) return;
  cancelledNew = leaving;
  // (A preset left as it was: itself again, as opened.)
  if (preset >= 0) {
    keepFocus(null, () => $("plot-keys"));
    return openChart(presetChart(preset));
  }
  // (No chart to go back to (a preset opened, let go of as it was): the Charts page, where it was left.)
  if (!back) {
    cancelling = chart.id;
    return showHome(true, leftPlace);
  }
  // (The focus on the chart gone back to: its row, else its title.)
  keepFocus(null, () => $("chart-list").querySelector(`[data-open-chart="${CSS.escape(back.id)}"]`), () => $("plot-keys"), () => $("new-chart"), () => $("charts-nav"));
  // (An empty preview left simply goes.)
  openChart(back);
}
/** @param {Chart} doc */
function adopt(doc) {
  library.charts.unshift(doc);
  leaveHome(() => openChart(doc));
}
/**
 * A chart opened as a preview: looked at, not in your charts yet (see `previewCharts`); the same chart a preview already,
 * that one (`push`: none pushed, Back or Forward bringing it). @param {Chart} doc
 */
function showPreview(doc, push = true) {
  // (What's being typed in the chart before, kept on it while it's still the preview it is. Back, the toast saying
  // it was closed goes. The same chart there, it, at the window this one came with.)
  flushEdits();
  const like = previewLike(doc), shown = like ?? doc;
  if (like) {
    // (Its own window kept first, then this one's over it, or none: the interval and range this one came with show.)
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
/** A preview closed (the open one, else `doc`): open, the chart before it takes its place (none: no ×). */
function closePreview(doc = chart) {
  if (!isPreview(doc)) return;
  const back = chart === doc ? previousChart() : chart;
  if (!back) return;
  // (What's being typed, in first: its step before this one's start.)
  flushEdits();
  // (The focus on a row: the next one's, else the one before's, else the open chart's.)
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
  // Back, its edits with it, where it was, at its window; open again only if it was (and still on a chart). Edited:
  // offered at once; else a step all the same, as ⌘Z finds it.
  const reopen = () => {
    if (chartById(doc.id)) return;
    closedPreviews.delete(doc.id);
    if (view) (viewsNow()[doc.id] = view), putViews(viewsNow());
    previewCharts.splice(Math.min(at, previewCharts.length), 0, doc);
    asideDirty.add(doc);
    // (Redone: closed again, if it's still a preview.)
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
/** The previews closed in this tab, by id, with their titles: a step back to one (Back) says it was closed. */
const closedPreviews = new Map();
/** The preview the toast up says was closed (its Undo brings it back). @type {Chart | null} */
let closedToast = null;
/** The previews kept aside, this tab's and others' (migration, remove after the release following 2026-11-26 (the next past v0.13.0): a preset as made, once kept beside its edited copy, is read as the ordinary preview it is now). @returns {Chart[]} */
const previewsAside = () => ((stored) => (Array.isArray(stored) ? stored.filter((doc) => typeof doc?.id === "string" && !(doc.made !== undefined && doc.untouched === true)) : []))(storage.get(PREVIEW_ASIDE, null));
/** Previews kept, by their ids aside (and when last edited): let go of there once saved with your charts (see persist). @type {Map<string, number>} */
const keptPreviews = new Map();
/** An aside copy that's another tab's later one than `updated` (edited there since): kept, as theirs. */
const laterAside = (/** @type {any} */ item, /** @type {number} */ updated) => (Number(item?.updated) || 0) > updated;
/** A preview let go of (closed): no longer kept aside (another tab's later copy of it stays). */
const forgetPreview = (/** @type {Chart} */ doc) => {
  asideDirty.delete(doc);
  const left = previewsAside().filter((item) => item.id !== doc.id || laterAside(item, doc.updated));
  if (left.length) storage.set(PREVIEW_ASIDE, left);
  else storage.drop(PREVIEW_ASIDE);
};
/**
 * A preview, the user's (the open one, else `doc`): first in your charts, saved (`announce`: kept by the button, the
 * "Saved" note says so).
 */
function keepPreview(announce = true, doc = chart) {
  if (!isPreview(doc)) return;
  // (Its copy aside stays till it's saved with your charts: see persist.)
  asideDirty.delete(doc);
  keptPreviews.set(doc.id, doc.updated);
  // (A draft's own id, deleted as its first edit was undone: its own again, kept again by a redo or a new edit. Any
  // other id another's, listed or deleted (a link's): a new one, the steps that knew the old one finding it, see
  // chartAgain.)
  const listed = inLibrary(doc), old = doc.id;
  if (!listed && drafts.has(doc) && Object.hasOwn(library.deleted, doc.id)) recover(doc);
  if (listed || Object.hasOwn(library.deleted, doc.id)) rekey(doc, uid());
  if (!listed && old !== doc.id) chartAgain.set(old, doc.id);
  // (Kept by its button, which goes: the focus on the chart's ⋯ beside it, not into a field where the next key would type.)
  if (document.activeElement === $("keep")) keepFocus(null, () => $("chart-more"), () => $("plot-keys"));
  library.charts.unshift(doc);
  dropPreview(doc);
  save(announce);
  renderCharts();
  clearLink();
}

/** A preview kept by asking (its button, ⌘S, ⌘K): the Library's step, undone as a preview again. */
function keepChart(doc = chart) {
  if (!isPreview(doc)) return;
  keepPreview(true, doc);
  libraryStep(`Saved “${titleOf(doc)}”`, () => {
    const now = library.charts.find((other) => other.id === doc.id);
    if (!now) return;
    discard(now);
    // (A preview again, kept aside as previews are: a reload keeps it.)
    keptPreviews.delete(now.id);
    asideDirty.add(now);
    addPreview(now);
    save(false);
    renderCharts();
    renderHome();
    // (Kept again: the same chart, its deletion taken back, as a deleted chart's Undo does.)
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
/** What adding a series offers first, after those added lately: the series most charts start from. */
const SUGGESTED = ["price", "realized_price", "mvrv", "market_cap", "supply_in_profit", "hashrate"];
/** The charts last opened, most recent first: kept on their own, so viewing reorders and edits nothing. */
const VIEWED = "bitview-studio:viewed";
const viewed = () => {
  const ids = storage.get(VIEWED, []);
  return Array.isArray(ids) ? ids.filter((id) => typeof id === "string" && /^[a-z0-9]{1,16}$/.test(id)) : [];
};
const rememberViewed = (id) => storage.set(VIEWED, [id, ...viewed().filter((other) => other !== id)].slice(0, 12));
/** How many charts Recent lists. */
const RECENT_SHOWN = 5;
/**
 * Recent: the charts last opened, the latest first (one gone or emptied since, left out): yours, previews, presets
 * (as edited here, else as made; `preset`, which, else -1).
 * @returns {{ id: string, doc: Chart, preset: number }[]}
 */
const recentCharts = () => viewed().flatMap((id) => {
  const kept = chartById(id), preset = kept ? presetIndex(kept) : presetById(id);
  // (A preset's as the presets have it once they're read: a copy kept here (another tab's too) of one changed since,
  // as made now; of one gone, left out. Before, as its copy's kept.)
  const doc = preset >= 0 ? presetDoc(preset) : kept?.preset !== undefined && presets ? undefined : kept;
  return doc?.layers.length ? [{ id, doc, preset }] : [];
}).slice(0, RECENT_SHOWN);
/**
 * A chart of Recent's opened, by its id: a preset's as a preset is (its edits, if any). Its row, picked, is in sight:
 * no sections opened, nothing scrolled to.
 * @param {string} id
 */
function openRecent(id) {
  const doc = chartById(id), preset = doc ? presetIndex(doc) : presetById(id);
  if (preset >= 0) return presetsEdit.openPreset(preset);
  delete document.body.dataset.sheet;
  grid.go(doc);
}
/** A chart of Recent's mark: a preset's as the sidebar draws it (asked for, see presetMarksAsked), else its own. @param {{ doc: Chart, preset: number }} item */
// (An edited preset's, before the presets are read: its mark as kept, by its copy, as the presets' rows will have it.)
const recentMark = ({ doc, preset }) => (preset >= 0 ? (presetMarks.has(grid.presetMarkKey(preset)) || grid.presetMarksAsked.add(preset), grid.presetMark(preset)) : doc.preset !== undefined && presetMarks.has(grid.keptMarkKey(doc)) ? grid.markOf(presetMarks.get(grid.keptMarkKey(doc)) ?? grid.NO_MARK) : grid.chartMark(doc));

/**
 * The focus, where what had it goes (`away`: put away, the panel closed, the sidebar slid off; else removed, once
 * the page is drawn again), on the first of `next` that shows (a row's successor, the button that brings it back):
 * never lost to the page. A focus nowhere (after a click on the chart) stays nowhere.
 * @param {Element | null} away @param {...(() => Element | null | undefined)} next
 */
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
  // (Gone: once the page is drawn again; what takes its place still coming in, once it's in.)
  const lost = () => !had.isConnected || document.activeElement === document.body || !had.checkVisibility();
  requestAnimationFrame(() => lost() && !go() && setTimeout(() => lost() && go(), MOTION.page + 40));
}
// ── Sidebar ───────────────────────────────────────────────────────────────

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
/** The panel open or closed: as kept, or (full screen's, for the while) as given. */
function applyPanel(state = prefs.panel) {
  if (state === "closed") keepFocus($("panel-grip"), () => $("panel-toggle"), () => $("plot-keys"));
  document.body.dataset.panel = state;
  $("panel-toggle").setAttribute("aria-pressed", String(state === "open"));
}
const panelShown = () => (phone.matches ? document.body.dataset.sheet === "panel" : document.body.dataset.panel === "open" || document.body.hasAttribute("data-peek"));
// (Shown or not, the panes' room over their lines again: their legends there only while it's not, see .legend.)
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
  // (In full screen, for the while: leaving it, the panel's as it was.)
  if (fullOn()) return applyPanel(open ? "open" : "closed");
  prefs.panel = open ? "open" : "closed";
  savePrefs();
  applyPanel();
}
applyPanel();

// ── The sidebar's and the panel's widths ─────────────────────────────────

/**
 * Each one's width: at least as it comes (the page's own, --side and --panel-width: the first script too), at most a
 * share of the window (never wider than `cap`); its property, its pref, its element and its grip.
 */
const SIDE_WIDTHS = /** @type {const} */ ({
  side: { prop: "--side", key: "sideWidth", min: 264, share: 0.34, cap: 480, element: "sidebar", grip: "side-grip" },
  panel: { prop: "--panel-kept", key: "panelWidth", min: 300, share: 0.4, cap: 560, element: "panel", grip: "panel-grip" },
});
/** @typedef {keyof typeof SIDE_WIDTHS} Widthed */
/** The chart's least room beside both (its foot's ranges and navigator, its plot). */
const CHART_ROOM = 500;
/** Whether the panel sits beside the chart (narrower, it floats over it or is a sheet: its width is its own). */
const besideChart = matchMedia("(min-width: 1101px) and (min-height: 501px)");
/**
 * The most one can be at this window's width, the other as kept (the sidebar's by the panel's kept or least, when it
 * sits beside the chart; the panel's by the sidebar's: the first script's the same).
 * @param {Widthed} which
 */
const widthMost = (which) => {
  const { min, share, cap } = SIDE_WIDTHS[which], other = which === "side" ? (besideChart.matches ? Math.max(prefs.panelWidth ?? 0, SIDE_WIDTHS.panel.min) : 0) : widthOf("side");
  return Math.max(min, Math.min(cap, Math.round(innerWidth * share), innerWidth - CHART_ROOM - other));
};
const widthOf = (/** @type {Widthed} */ which) => ((kept) => (kept === null ? SIDE_WIDTHS[which].min : clamp(kept, SIDE_WIDTHS[which].min, widthMost(which))))(prefs[SIDE_WIDTHS[which].key]);
/** The widths as kept, within their bounds at this window's width; their grips say them. */
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
/**
 * The view kept while a width is changed by hand (its stretch of time, its bars narrower or wider; see holdView):
 * the one before. Let go of once the chart has its last size (two frames after).
 * @type {{ from: number, to: number } | null}
 */
let resizeKeeps = null;
let resizes = 0;
/** A width changed by hand: to `px` (null: as it comes), at once, the view kept. @param {Widthed} which @param {number | null} px */
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
    // (The chart stretched as it's dragged, drawn once it's let go: see holdChart.)
    holdChart();
    // (At most once a frame, where the hand is then; held again if let go meanwhile (the window resized, a motion
    // over).)
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
  // (Its keys as a separator's: ← → by a step (⇧, four), Home and End to its least and its most.)
  grip.addEventListener("keydown", (event) => {
    const now = element().getBoundingClientRect().width, step = event.shiftKey ? 64 : 16;
    const to = event.key === "ArrowRight" ? now + sign * step : event.key === "ArrowLeft" ? now - sign * step : event.key === "Home" ? 0 : event.key === "End" ? Infinity : null;
    if (to === null || event.metaKey || event.ctrlKey || event.altKey) return;
    // (Its own: not the page's too, a drawing nudged.)
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
    // (Closed, at half again, once out of sight: the height's wait held with its slide's, below.)
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
/** The open list's first row in sight as it narrowed to the rail (its key, where it was), and the rail's scroll then: widened again unscrolled, back there. */
let sideKept = /** @type {{ key: string, top: number, rail: number } | null} */ (null);
function toggleSidebar() {
  // (The rail's folders never open: the rows in them gone as it narrows, back as it widens, fading in; the others
  // gliding to their places as its width eases, not standing in them before it's moved. Where they are, read
  // first: as the last frame laid them out, nothing to lay out again.)
  const before = listRows($("chart-list")), was = grid.placesOf([...before.keys()], (row) => before.get(row)), scrolled = $("chart-list").scrollTop;
  prefs.sidebar = prefs.sidebar === "collapsed" ? "expanded" : "collapsed";
  savePrefs();
  // (While it slides, its words clipped by it, not cut short anew each frame: till its width's slide ends.)
  if (!reduced()) document.body.toggleAttribute("data-sliding", true);
  // (Collapsed, the rail keeps only its icons (a group's head's too): a focus on what it puts away, a row's ⋯, a
  // folder's, on its toggle.)
  const putAway = prefs.sidebar === "collapsed" && document.activeElement?.closest(".sidebar") && !document.activeElement.matches(".nav-item, .list-head-name, .search, .brand a, #sidebar-toggle") ? document.activeElement : null;
  // (A row in a folder, folding away (the rail's folders never open): its folder at the top takes the focus first.)
  const inFolder = prefs.sidebar === "collapsed" ? document.activeElement?.closest("#chart-list .in-folder") : null;
  let top = inFolder;
  while (top?.classList.contains("in-folder")) top = top.previousElementSibling;
  if (inFolder) /** @type {HTMLElement | null | undefined} */ (top?.querySelector(".folder-item"))?.focus({ preventScroll: true });
  applySidebar();
  grid.movingSide(renderCharts);
  // (The list where it was: the first row in sight (or, gone, the nearest before it there still, its folder's) where it
  // was, not the list scrolled to where its other rows put it. Widened again, the rail not scrolled since: the open
  // list's first row in sight as it was before it narrowed (the rail, shorter, couldn't hold its place).)
  const list = $("chart-list"), sight = list.getBoundingClientRect(), after = listRows(list), keys = [...before.values()], first = keys.findIndex((key) => (was.get(key) ?? -Infinity) >= sight.top - 1);
  const rowOf = new Map([...after].map(([row, key]) => [key, row]));
  const back = prefs.sidebar !== "collapsed" && sideKept && sideKept.rail === scrolled && rowOf.has(sideKept.key) ? sideKept : null;
  const anchor = back?.key ?? (first < 0 ? null : keys.slice(0, first + 1).reverse().find((key) => rowOf.has(key)));
  const kept = anchor ? rowOf.get(anchor) : undefined;
  if (kept) list.scrollTop += kept.getBoundingClientRect().top - /** @type {number} */ (back ? back.top : was.get(anchor));
  // (The row with the focus in sight, as little moved as that takes: the keys go on from it.)
  const focused = list.contains(document.activeElement) ? /** @type {Element} */ (document.activeElement).getBoundingClientRect() : null;
  if (focused) list.scrollTop += Math.min(0, focused.top - sight.top) || Math.max(0, focused.bottom - sight.bottom);
  sideKept = prefs.sidebar === "collapsed" && first >= 0 ? { key: keys[first], top: /** @type {number} */ (was.get(keys[first])), rail: list.scrollTop } : null;
  glideList(list, was);
  // (No motion to it (transitions held off meanwhile: a width dragged): no slide to wait on.)
  if (!widthMoving()) document.body.removeAttribute("data-sliding");
  // (The list's one stop for Tab on a row in sight.)
  grid.roveList();
  keepFocus(putAway, () => $("sidebar-toggle"));
}
// (Slid, once its width's done moving: one turned back midway is cancelled, the one taking its place still going
// (held a frame as the chart's sized for it: paused, see sizeOnce).)
const widthMoving = () => $("sidebar").getAnimations().some((motion) => motion instanceof CSSTransition && motion.transitionProperty === "width" && (motion.playState === "running" || motion.playState === "paused"));
const slid = (/** @type {TransitionEvent} */ event) => event.target === $("sidebar") && event.propertyName === "width" && !widthMoving() && document.body.removeAttribute("data-sliding");
$("sidebar").addEventListener("transitionend", slid);
$("sidebar").addEventListener("transitioncancel", slid);
applySidebar();

// ── Bitview's mark ────────────────────────────────────────────────────────

/**
 * A glass cube that empties and fills; pointed at (by a mouse), it spins up, over a few seconds, to a blur (a still
 * cylinder), and holds there; let go of, it winds down along the same curve and lands square. It's a box seen from
 * above: a square with eased corners, raised. At rest it's drawn flat (the rim as thick all around); turning, as the
 * box.
 */
function animateMark() {
  const svg = /** @type {SVGSVGElement} */ (document.querySelector(".logo svg")), first = /** @type {SVGPathElement} */ (svg.querySelector("path"));
  const SIN = Math.tan(Math.PI / 6), COS = Math.sqrt(1 - SIN * SIN), C30 = Math.cos(Math.PI / 6), SQ3 = Math.sqrt(3);
  const REST = Math.PI / 4;
  // The box: a square of side 1 seen from 35° up, each corner easing in over 0.27 of a side; the opening, the same
  // square at 0.72. Flat, the rim is RIM thick and the opening's sides round off over CORNER along their edges.
  const CUT = 0.2718, EASE = 0.798, HALF_HEIGHT = 0.3674 / COS, OPENING = 0.7231, RIM = 0.0979, CORNER = 0.104;
  /** The square at a scale, around its eased corners. @returns {number[][]} */
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
  /** The square at a height, turned by `yaw`, as seen. */
  const seen = (points, height, yaw) => points.map(([x, z]) => [x * Math.cos(yaw) - z * Math.sin(yaw), -height * COS + (x * Math.sin(yaw) + z * Math.cos(yaw)) * SIN]);
  /** @returns {number[][]} */
  const hull = (points) => {
    const sorted = [...points].sort((p, q) => p[0] - q[0] || p[1] - q[1]), lower = [], upper = [];
    const turn = (o, a, b) => (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0]);
    for (const p of sorted) { while (lower.length > 1 && turn(lower.at(-2), lower.at(-1), p) <= 0) lower.pop(); lower.push(p); }
    for (const p of sorted.reverse()) { while (upper.length > 1 && turn(upper.at(-2), upper.at(-1), p) <= 0) upper.pop(); upper.push(p); }
    return [...lower.slice(0, -1), ...upper.slice(0, -1)];
  };

  // Flat, the opening keeps the box's own tips (as seen at rest), and its sides run down at the rim to the level.
  const middle = -HALF_HEIGHT * COS, reach = (OPENING / 2) * Math.SQRT2 * SIN, top = middle - reach, width = 0.63 - RIM;
  const atRest = seen(OPEN, HALF_HEIGHT, REST);
  /** The opening's tip on one side of the middle, around its sharp point. */
  const tip = (above) => atRest.filter(([x, y]) => Math.abs(x) <= 0.14 && y < middle === above).map(([x, y]) => [x, y - (above ? top : middle + reach)]);
  const TOP = tip(true), FRONT = tip(false), sideTop = top + SIN * width, EMPTY = -2 * sideTop;
  const arc = (cx, cy, r, from, to) => Array.from({ length: 9 }, (_, i) => { const q = ((from + ((to - from) * i) / 8) * Math.PI) / 180; return [cx + r * Math.cos(q), cy + r * Math.sin(q)]; });
  /** The flat opening when `fill` of the cube is full (1: only the opening shows). */
  const flat = (fill) => {
    const h = (1 - fill) * EMPTY, sideBottom = sideTop + h, bottom = sideBottom + SIN * width;
    // The sides' corners keep rounding off over CORNER: one corner while the side is short, then two.
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
  /** (1 − k)·A + k·B for convex shapes, their edges merged by direction: a blend with no bumps. */
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
  /**
   * The mark at a fill, turned by `yaw` (REST: as it rests); `spinning` (0 to 1) draws it as the box at any angle.
   * (`opened`: the flat opening at that fill, made once for a blur's copies.)
   */
  const draw = (fill, yaw, spinning = 0, opened = flat(fill)) => {
    const box = hull([...seen(BOX, HALF_HEIGHT, yaw), ...seen(BOX, -HALF_HEIGHT, yaw)]);
    const k = Math.max(spinning, Math.abs(Math.sin(2 * (yaw - REST)))), level = HALF_HEIGHT - (1 - fill) * 2 * HALF_HEIGHT;
    const opening = k < 1e-6 ? opened : blend(opened, hull([...seen(OPEN, HALF_HEIGHT, yaw), ...seen(OPEN, level, yaw)]), k);
    return ring(box) + ring(opening);
  };
  first.setAttribute("d", draw(1, REST));

  // It waits full, empties easing out, waits, fills easing out (seconds).
  const WAIT = 0.5, MOVE = 1.6, LOOP = 2 * (WAIT + MOVE);
  const easeOut = (x) => 1 - (1 - x) ** 3;
  const fillAt = (t) => {
    t %= LOOP;
    if (t < WAIT) return 1;
    if (t < WAIT + MOVE) return 1 - easeOut((t - WAIT) / MOVE);
    if (t < 2 * WAIT + MOVE) return 0;
    return easeOut((t - 2 * WAIT - MOVE) / MOVE);
  };
  // Pointed at: the turn eases in to TURNS turns a second over FULL seconds (by the POWER of the time), and holds there.
  // Let go of: the same curve backwards, stretched to land square (pointed at too briefly to wind down that way, it
  // eases on to the next rest instead).
  const FULL = 5, TURNS = 8, POWER = 1.6, QUARTER = Math.PI / 2, VMAX = 2 * Math.PI * TURNS;
  const along = (t) => Math.min(1, Math.max(0, t / FULL)); // (a frame's time can fall a hair before the pointer's)
  /** The speed after t seconds pointed at, the time it takes to reach a speed, and the turn covered by then. */
  const speedAt = (t) => VMAX * along(t) ** POWER, timeTo = (v) => FULL * Math.min(1, v / VMAX) ** (1 / POWER);
  const turnBy = (t) => (VMAX * FULL * along(t) ** (POWER + 1)) / (POWER + 1);
  // Turning fast, a blur: what an EXPOSURE sees, as COPIES faint copies (always as many, so it never pulses). From a
  // quarter turn up, its copies cover all its ways round alike: drawn on a fixed grid, a still cylinder.
  const COPIES = 24, EXPOSURE = 1 / 30;
  const paths = [first, ...Array.from({ length: COPIES - 1 }, () => svg.appendChild(/** @type {SVGPathElement} */ (first.cloneNode())))];
  const brand = /** @type {HTMLElement} */ (document.querySelector(".brand"));
  let pointedAt = /** @type {number | null} */ (null), angle = 0, speed = 0, last = 0, drawn = "";
  /** While it slows: the curve backwards (from h seconds in, stretched), or an ease on to the next rest. */
  let landing = /** @type {{ back: boolean, from: number, to: number, h: number, stretch: number, speed: number, at: number, length: number } | null} */ (null);
  /** The mark at rest, by its fill (in 240ths): drawn once each, the idle loop then costs nothing. */
  const rest = new Map();
  /** The frames asked for while it moves (its fill's cycle from `cycleFrom`); none at rest. */
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
    // (Pointed at again while slowing: on from the speed it has; still, a new cycle of its fill.)
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
    // (Out of sight, a phone's, or motion reduced: at rest, full.)
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
        // Hermite: it leaves at its speed and lands still.
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
    // Moving, or its cycle under way: the next frame; still and full again, a cycle over: none.
    if (!still || cycle < LOOP || cycle % LOOP >= WAIT) requestAnimationFrame(frame);
    else looping = false;
    const from = sweep >= QUARTER ? Math.floor(angle / step) * step : angle;
    // What's drawn, by what it depends on (the still cylinder, whatever its angle, by its fill alone).
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

// ── Data ──────────────────────────────────────────────────────────────────

// ── The chart ─────────────────────────────────────────────────────────────

/**
 * The chart's Inter, a tad heavier than the page's (its labels read better, on the lines and on their colored tags):
 * the page's own font files (no second copy of them), as "Inter Chart" from 500 up (a variable font: a canvas's
 * regular text comes at 500, bold stays bold), its digits tabular (a canvas has no font-variant-numeric: the face's
 * own setting), every digit as wide: a price scale doesn't widen and narrow as its labels' digits change. Begun with the
 * page (see the first script), its last face only (a first screen's characters); the core's (é, ≥, °: a name, a
 * drawing's words) with the rest. Till one's in, the page's font: a canvas has no font-display, and asks for none.
 */
const chartFaces = [...document.fonts].filter((face) => face.family.replace(/"/g, "") === "Inter Chart");
const chartFont = (chartFaces.at(-1)?.load() ?? Promise.reject()).then(() => true, () => false);
let chartSans = css("--sans");
/** The size of every word on the chart (CSS pixels): its axes', its tags', its marks', its navigator's; its legends' the same (see .legend). */
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
// The chart (bitview-charts): its timeline (`engine.time`), its panes, its drawing. On an element of its own in the
// page's box, sized by the page (Studio sizes and pins it while a motion stretches its picture: see `resized`).
const engine = createChart($("chart").appendChild(Object.assign(document.createElement("div"), { style: `width: ${$("chart").clientWidth}px; height: ${$("chart").clientHeight}px` })));
engine.theme = chartTheme();
/** The right axis' width till it's drawn (then as its labels need: the library's), where a layer's on that side. */
const RIGHT_AXIS = engine.axis;
engine.time.minSpacing = MIN_BAR_SPACING;
engine.crosshair.setFormat((time) => pointLabel(time));
/** The size the chart's drawn at, as last given it (its element's is its box's, whatever it's drawn at). */
let chartSize = { width: $("chart").clientWidth, height: $("chart").clientHeight };
// (From the start, as after every resize.)
fillBox();
/** The chart drawn at a size, at once (as the library's own watch did: no frame of what it was). */
// (Its element filling its box after, whatever the library makes it (a pixel short, rounded): no strip under it.)
const sizeChart = (/** @type {number} */ width, /** @type {number} */ height) => ((chartSize = { width, height }), (Object.assign(engine.element.style, { width: `${width}px`, height: `${height}px` }), engine.layout()), !sizeHeld && !exporting && fillBox());
/**
 * A layout's motion (the sidebar's, the panel's, full screen's, a peek's): the chart sized once for it, not at each
 * of its frames (each a new size: its canvases made anew, every series drawn again; Firefox shows a third of them).
 * It takes its size at the motion's end at once, as the motion starts (the press that made it: it follows at once,
 * nothing snaps at the end), pinned to the edges of its box that move least (the sidebar's motion moves the left
 * one: pinned right), the moving edges sliding over it or off it. Its box narrowing or widening, it keeps the view's
 * stretch of time instead: drawn once at its end's width, pinned to its price axis's side, its plot stretched to its
 * box's width each frame (see stretchDrawn: nothing drawn again, the drawing's own end the motion's). At the end, it
 * follows its box again.
 */
let motionsSized = 0, sizeHeld = false;
/** The chart a motion only stretched, to draw at its end's width once it's over (see sizeOnce). @type {string | null} */
let stretchKept = null;
function sizeOnce() {
  const box = $("chart"), inMotion = /** @type {CSSTransition[]} */ (document.querySelector(".app")?.getAnimations({ subtree: true }).filter((motion) => motion instanceof CSSTransition && /^(width|height|margin|padding|border|inset|left|right|top|bottom)/.test(motion.transitionProperty)) ?? []);
  // (No motion (reduced motion): the chart at its box's new size at once, its view's stretch kept as well.)
  if (!inMotion.length) {
    const { width, height } = box.getBoundingClientRect();
    if (sizeHeld || Math.abs(width - chartWidth()) < 1 || !resolved?.times.length || !chartShown()) return;
    sizeKeeping(width, height);
    fillBox();
    return;
  }
  // (Its box now and the chart's own size: the same, unless a motion before this one's still holding it.)
  const run = ++motionsSized, at = inMotion.map((motion) => motion.currentTime), now = box.getBoundingClientRect(), own = engine.element.getBoundingClientRect();
  // (Its box at their end: each motion that moves it there for a moment, read, and back where it was; nothing's drawn
  // between. Those inside the sidebar or the panel (their rows' own, many) don't: only the boxes beside the chart's.)
  // (A hair before their end, not at it: WebKit takes a transition sought to its very end as done, and starts a copy
  // of it beside the first, whose second motion lays the chart out again once the first lands. WebKit bug.)
  const moving = inMotion.filter((motion) => ((target) => !target || !target.parentElement?.closest(".sidebar, #panel"))(/** @type {KeyframeEffect | null} */ (motion.effect)?.target));
  for (const motion of moving) motion.currentTime = Number(/** @type {AnimationEffect} */ (motion.effect).getComputedTiming().endTime) - 0.01;
  const end = box.getBoundingClientRect();
  moving.forEach((motion) => (motion.currentTime = at[inMotion.indexOf(motion)]));
  // (Across, its stretch kept: the view as drawn (a motion's in flight, as it'll end), and the plot's width shown now
  // (a motion's in flight, as stretched).)
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
  // (Its own clipping let go of meanwhile: its foot moves with the box's (placeFoot), past its end size; the card's
  // own clips at the box.)
  Object.assign(engine.element.style, { position: "absolute", left: "", right: "", top: "", bottom: "", [side]: "0", [edge]: "0", width: `${width}px`, height: `${height}px`, overflow: "visible" });
  if (!later && (width !== own.width || height !== own.height)) {
    // (Drawn at its new size now, the motions held at their start meanwhile: they start on the next frame, from
    // their start, not a long frame in.)
    for (const motion of inMotion) motion.pause();
    sizeChart(width, height);
    // (Its panes' room for their legends, a share of their height, fitted to it now too: nothing moves once it lands.)
    fitMargins();
    if (across) keepStretch(was);
    placeFoot(now.height);
    // (Those another toggle has replaced meanwhile left be: started again, they'd come back.)
    // (Which are still paused, all read first, then each started: a read between writes would lay the page out again
    // for each of them, hundreds with a long Library's rows.)
    requestAnimationFrame((time) => inMotion.filter((motion) => motion.playState === "paused").forEach((motion) => (motion.startTime = time - Number(at[inMotion.indexOf(motion)] ?? 0))));
  }
  // (Stretched, by its scales as drawn (at its end's width, or as it was); a motion started mid-motion, as drawn
  // too.)
  if (across) stretchAcross(run, now.width, width);
  // (Drawn at its end's size once the page is idle, not in the frames that end the motion (what fades in as it ends:
  // the sidebar's words, New folder): a heavy chart's draw is a long frame.)
  Promise.allSettled(inMotion.map((motion) => motion.finished)).then(() => {
    if (run !== motionsSized) return;
    if (later) return whenIdle(() => run === motionsSized && releaseSize(), { timeout: 200 });
    releaseSize();
  });
}
/**
 * The chart held as it's drawn while its width's changed by hand (a grip dragged): its drawing stretched to its box
 * each frame, as a width-only motion's (see sizeOnce), drawn at its width once let go (releaseSize): not drawn anew
 * each frame, a heavy chart's long ones.
 */
function holdChart() {
  if (sizeHeld || !resolved?.times.length || !chartShown()) return;
  const run = ++motionsSized, leftAxis = engine.left > 0 && !(engine.axisShown > 0);
  stretchKept = chart.id;
  sizeHeld = true;
  chartHeight = chartSize.height;
  Object.assign(engine.element.style, { position: "absolute", left: "", right: "", top: "0", bottom: "", [leftAxis ? "left" : "right"]: "0", width: `${chartSize.width}px`, height: `${chartSize.height}px` });
  stretchAcross(run, $("chart").getBoundingClientRect().width, chartSize.width);
}
/** Whether the chart, its size held, is pinned at its box's top (see sizeOnce): its foot then at the box's (placeFoot). */
let footPinned = false;
/**
 * A view kept at the chart's width now: its stretch of time (a preset showing, its window at this width), its bars
 * as close as that needs. Drawn at once. @param {{ from: number, to: number } | null} was
 */
function keepStretch(was) {
  if (!resolved || !chartShown()) return;
  fitSpacing(chart, resolved.times.length, was ?? undefined);
  if (was) showSpan(was, EXACT_RANGE);
  engine.render(true);
  // (Its box's new width and view, for the next resize: see holdView.)
  const range = shownSpan();
  lastBox = chartWidth();
  (heldView = range ? { box: lastBox, at: range } : null), (madeView = null);
}
/** The chart following its box again (a window resized meanwhile, at once): what's laid out by its size, again. */
function releaseSize() {
  if (!sizeHeld) return;
  sizeHeld = false;
  stretchDrawn(null);
  placeFoot(0);
  // (Back in its box, the whole of it, as the library made it: drawn there now, if the motion only stretched it,
  // its stretch of time kept.)
  Object.assign(engine.element.style, { position: "", left: "", right: "", top: "", bottom: "", width: "100%", height: "100%", overflow: "" });
  // (Its view as it is now, a pan or a zoom meanwhile kept; another chart opened meanwhile drew itself.)
  const { width, height } = $("chart").getBoundingClientRect();
  if (stretchKept && stretchKept === chart?.id && chartShown()) {
    sizeKeeping(width, height);
  } else if (width !== chartSize.width || height !== chartSize.height) sizeChart(width, height);
  stretchKept = null;
  chartHeight = $("chart").getBoundingClientRect().height;
  relayout();
}
// Whatever lays the page out anew (a button, a key, ⌘K, a peek, another tab's settings): its motion, sized once.
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
// Once a face is in (the first before the first frame when it already is), the chart's words in it, measured again (a
// new name: the axes' kept widths are by the font's).
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
/** The layers' order as last put in order (see build). */
let drawnOrder = "";
/** The layer in the spotlight while its row or caption is hovered: the others fade back. */
let spotlit = null;
function spotlight(id) {
  if (id === spotlit) return;
  spotlit = id;
  const lighted = id && drawn.has(id) && drawn.size > 1 && !layerById(id)?.hidden ? id : null;
  // (Among its own pane's: the others' panes as they are (a band lit, the price over it still read).)
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
/** The layer whose row or legend item the pointer's on (a finger's, none), and whether it's on the chart itself. */
let pointedLayer = /** @type {string | null} */ (null), pointerOnChart = false;
document.addEventListener("pointerover", (event) => {
  // (Not while something's carried: the rows passed over aren't pointed at. A finger points at nothing: what it
  // touched, a row it scrolled the list by, would stay lit once it's lifted.)
  if (document.body.classList.contains("carrying")) return;
  pointedLayer = event.pointerType === "touch" ? null : (closest(event, ".layer-row")?.dataset.layer ?? closest(event, ".legend [data-select]")?.dataset.select ?? null);
  // (A finger points at nothing: a tap on the chart's legend opens its layer, in the spotlight while it's open.)
  pointerOnChart = event.pointerType !== "touch" && !pointedLayer && Boolean(closest(event, "#chart"));
  respot();
});
// (Out of the window: on nothing.)
document.addEventListener("pointerout", (event) => event.relatedTarget || ((pointedLayer = null), (pointerOnChart = false), respot()));
/** The data on screen, written by the build that lands (see buildChart) and grown by the live price. @type {Resolved | null} */
let resolved = null;
/** How many times the open chart's data has changed here (read anew, grown, a live price moved in it). */
let dataRevision = 0;
/** The draw asked for last (ahead of `resolved.version` while one is loading). */
let drawVersion = 0;
/** What that draw asked for: what it resolves (its `dataKey`), its look (its `lookKey`) and its interval. */
let asking = { key: "", look: "", index: "" };
/**
 * What's on screen, written only by the build that lands (see buildChart): the chart as built (a ⌘K look's, while one
 * shows; the live price paints its colors from it), its range the one its view was laid out from (the build's, or
 * showRange's once it has moved the view there; none while one asked for waits for its draw), and its `lookKey`. Null: nothing, or the open chart to be drawn as if
 * opened anew. Its data is `resolved`. @type {{ doc: Chart, look: string } | null}
 */
let displayed = null;
/** Whether the open chart is the one on screen. */
const chartShown = () => Boolean(chart && displayed && displayed.doc.id === chart.id);
/** Whether a draw is on its way (the data on screen another's than the one asked for last). */
const pending = () => drawVersion > (resolved?.version ?? 0);
/** A build asked for (see buildSoon): its frame's request (-1: run at once, a hidden tab's), and whether it fits the view anew. */
let buildAsked = 0, buildRefit = false;

/** Layers enough that a chart's build takes more than a frame or two on a phone without its JIT (six daily series, some 36,000 points: ~100 ms): see draw. */
const MANY_LAYERS = 6;
/**
 * Cached data keeps each edit's rebuild cheap (missing data fetched first).
 * @returns {Promise<boolean>} whether it drew (a newer draw can take its place)
 */
async function draw(refit = false, fresh = false) {
  // (A build asked for is this draw's, which builds as its data's in: see buildSoon.)
  if (buildAsked) {
    if (buildAsked > 0) cancelAnimationFrame(buildAsked);
    (buildAsked = 0), (refit ||= buildRefit), (buildRefit = false);
  }
  const version = ++drawVersion;
  undrawn = false;
  // Drawn on the way out of the grid: the chart grows out of its card instead of wiping in.
  const fromHome = document.body.hasAttribute("data-home");
  const doc = structuredClone(chart), was = displayed?.doc, askedIndex = asking.index;
  asking = { key: dataKey(doc), look: lookKey(doc), index: doc.index };
  const first = was?.id !== doc.id;
  // (Grown out of its card, a chart of many layers is built once it's grown, as one whose data loads is: its build
  // mid-motion (23 bands' on a phone without its JIT, a third of a second) held the motion still. Fewer build in a
  // frame or two, drawn as they grow.)
  const later = first && fromHome && doc.layers.filter((layer) => !layer.hidden).length >= MANY_LAYERS;
  // Each chart keeps its span at each fine interval while the view it backs is the one the chart opens on (another
  // chart opened, its view left at this interval): back at it, it's as it was left, its chunks at hand (unless the
  // memory's budget let them go). Opening on its preset instead (an interval switch, a view left at another
  // interval or none), the preset's span is made again, once a switch: a draw at the interval already asked for (a
  // range picked mid-switch) keeps the span showRange aimed. The runs joined go (made again from their chunks).
  if (first || (was?.index !== doc.index && askedIndex !== doc.index)) {
    if (!first || viewKey(doc.id) === LINKED || openView(doc)?.index !== doc.index) spans.delete(spanKey(doc.id, doc.index));
    spanned.clear();
  }
  // (Counted against the span it'll draw: see above.)
  const unasked = drawSeries(doc).filter((name) => !asked(name, doc) && intervalsOf(name)?.includes(doc.index) !== false).length;
  const showLoading = unasked > 0 && (first || !resolved);
  $("progress").hidden = !(unasked > 0 && !showLoading);
  if (showLoading) {
    // (How many, not from where: the live pill's tip says where, and an error, if it can't be reached.)
    $("loading-text").textContent = `Loading ${grouped.format(unasked)} series`;
    $("loading").hidden = false;
    $("navigator").classList.add("waiting");
  }
  // (Another chart's, loading: the one before out of sight meanwhile, its lines and legend not under the words.)
  $("chart").classList.toggle("waiting", (showLoading || later) && first);
  if (showLoading) $("unreached").hidden = true;
  // Data from memory, asked before the latest block: its tails come again. (Its span as asked: what it reaches.)
  const live = !FINE.has(doc.index) || spanOf(doc)?.live !== false;
  // (A heatmap, a profile, bubbles: drawn by the rest, asked for with its data, see later.js.)
  const rest = doc.layers.some((layer) => heatLike(layer) || layer.type === "bubbles") ? loadRest() : undefined;
  // (A sum of bands (over, under) finds its bands by the tree.)
  if (doc.layers.some((layer) => /\b(over|under)\(/.test(layer.formula ?? ""))) await import("/studio/client.js");
  const result = await resolve(doc, fresh || stale(doc));
  await rest;
  // (Its data come while the chart grows out of its tile: drawn once it's grown, not built mid-motion (a phone's
  // build of a big one, a second, held the motion still).)
  if (showLoading || later) await Promise.all((grid?.morphing ?? []).map((animation) => animation.finished.catch(() => {})));
  if (version !== drawVersion) return false;
  // (Back to the grid while it grew (its growth cancelled): built when it's opened again, as it grows (see
  // leaveHome's undrawn), not during the shrink.)
  if ((showLoading || later) && home) {
    // (Its loading words gone with it: another opened next, its data at hand, says nothing of loading.)
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
  // (At another interval, the view kept as it shows now, at the one before: moved while its data came, it's that one.
  // Only the switch's own, kept at the one before: none (a range asked for shows instead), or one kept at the new
  // interval (a date gone to), stays as it is.)
  if (displayed?.doc.id === doc.id && displayed.doc.index !== doc.index && openView(doc)?.index === displayed.doc.index) storeView(true);
  // (Nothing at all, bitview.space not answering: one word of it over the plot, not a red line per layer.)
  const unreached = result.failed.length > 0 && ![...result.results.values()].some((item) => item?.values);
  if (unreached) sayUnreached();
  $("unreached").hidden = !unreached;
  $("navigator").classList.toggle("waiting", unreached);
  dataRevision++;
  const reveal = first && doc.layers.length > 0 && !reduced() && !fromHome;
  build(withLook(structuredClone(chart)), Object.assign(result, { key: asking.key, version, live }), refit);
  for (const wake of [...drawnWaiters]) wake();
  // (A first chart drawn, on screen: what it didn't need, asked for now, see later.js.)
  requestAnimationFrame(() => setTimeout(askRest));
  withRest(layersPanels);
  updateStatus();
  // (What it shows said once drawn, not only on focus.)
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
/** The layers' list and the inspector, as a draw leaves them. */
function layersPanels() {
  renderLayers();
  inspector.renderInspector();
}
/** The kinds the library colors by a gradient itself (see buildChart's `graded`): lines, areas and trends by height, dots and bars each by its value. */
const GRADED = new Set(["line", "area", "trend", "dots", "histogram"]);
function buildChart(doc, data, refit) {
  // (One asked for, a frame on, is this one: see buildSoon.)
  if (buildAsked) {
    if (buildAsked > 0) cancelAnimationFrame(buildAsked);
    (buildAsked = 0), (refit ||= buildRefit), (buildRefit = false);
    // (Who waited for it, woken once this one's done: see settle.)
    queueMicrotask(() => { for (const wake of [...drawnWaiters]) wake(); });
  }
  // (What's on screen from now: this, whatever builds it, a restyle and a ⌘K look too; what was, to keep what can be.
  // A range asked for since this chart's view was laid out (a pick, an undo, at this interval or across a switch) is
  // owed: this build lays it out, whichever draw it's from.)
  const prior = displayed, priorTimes = resolved?.times ?? [], same = prior?.doc.id === doc.id && prior?.doc.index === doc.index, sameView = !refit && same;
  const owed = prior?.doc.id === doc.id && prior?.doc.range !== doc.range;
  (displayed = { doc, look: lookKey(doc) }), (resolved = data);
  // (Another chart, another interval: the crosshair set here, or the keys', lets go of a date it may not have.)
  if (!same) crossSet ? dropCross() : keyAway();
  const visible = sameView ? shownDates() : null;
  const logical = sameView ? shownSpan() : null;
  const showedAll = visible && priorTimes.length > 0 && Number(visible.from) <= priorTimes[0] && Number(visible.to) >= /** @type {number} */ (priorTimes.at(-1));
  const sameTimeline = priorTimes.length > 0 && data.times.length === priorTimes.length && data.times[0] === priorTimes[0] && data.times.at(-1) === priorTimes.at(-1);
  // Points added (or dropped) at the start, at a fine interval (the view reaching out): the view holds still, its
  // bars shifted by as many.
  const shift = sameView && !sameTimeline && FINE.has(doc.index) && priorTimes.length && data.times.length ? Math.round((priorTimes[0] - data.times[0]) / FINE_STEP[doc.index]) : 0;
  // Price scales stay as the person left them (a range set by hand, in any mode) while the panes and their modes
  // are the same: new data, a restyle. Otherwise they fit their data again.
  const shapeOf = (/** @type {Chart} */ of) => of.panes.map((pane) => `${pane.id}:${pane.mode}:${pane.stack ?? ""}:${pane.reverse ? 1 : ""}:${pane.average ?? ""}`).join();
  const panesShape = shapeOf(doc), priorShape = prior ? shapeOf(prior.doc) : "";
  const keep = sameView && panesShape === priorShape;
  // (A pane added or taken away, the chart's own: the chart as it was fades out over it laid out anew (its panes
  // cut to their new heights), as everything here changes, never a cut.)
  const paneIds = (/** @type {string} */ shape) => shape.split(",").map((pane) => pane.split(":")[0]).join();
  if (prior?.doc.id === doc.id && paneIds(priorShape) !== paneIds(panesShape) && !reduced() && !document.body.hasAttribute("data-picture")) fadeOut($("chart"), [{ opacity: 1 }, { opacity: 0 }], MOTION.panel, true, MOTION.ease);
  const glidingTo = engine.controller.target;
  engine.controller.stop();
  const was = new Map(drawn);
  drawn.clear();
  dotted.clear();
  const wasPainted = new Map(paintedLayers);
  paintedLayers.clear();
  // (The layer lit, lit again once its series are drawn anew.)
  const relit = spotlit;
  spotlit = null;
  for (const layerId of fadedNow) fadeLayer(layerId, false, was);
  fadedNow.clear();
  levelTags.length = 0;
  // (Its timeline now: what's below reads and moves the view on its dates (the view as it was read above).)
  showTimes(data.times);
  const paneIndex = new Map(doc.panes.map((pane, i) => [pane.id, i]));
  // (A pane on the library for each of the chart's, made as it comes: those past them go once their series have.)
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
  // The others' series go once the new ones are in (their panes kept till then, see the panes trimmed below).
  const old = [...was].filter(([id]) => !kept.has(id)).map(([, series]) => series);
  let added = false;
  /** @type {{ layer: Layer, value: number, color: string, dash: number, tag: boolean, mode: number, pane: number, format: any }[]} */
  const across = [];
  /** A layer's lines across, drawn once all are known: its levels, and a baseline's base (not as a band). */
  const linesAcross = (layer, pane, format, op) => {
    // (None stacked, nor on a % or indexed scale: see acrossUnfit.)
    if (acrossUnfit(doc, layer)) return;
    const values = [...(layer.levels ?? []).map((level, i) => ({ key: String(i), value: level.value })), ...(layer.type === "baseline" ? [{ key: "base", value: baseOf(layer) }] : [])];
    for (const { key, value } of values) {
      const { color, dash, tag, line } = lineLook(layer, key);
      // (No line, its band alone: nothing across.)
      if (line === false) continue;
      across.push({ layer, mode: modeOf(doc, layer), pane, format, value, color: op < 1 ? alpha(ink(color), op) : ink(color), dash, tag: Boolean(tag) });
    }
  };
  for (const layer of doc.layers) {
    const result = data.results.get(layer.id);
    // Its opacity on every color it paints (fills keep their own share of it).
    const op = opacityOf(layer), base = ink(layer.color), under = ink(belowOf(layer));
    const color = op < 1 ? alpha(base, op) : base, below = op < 1 ? alpha(under, op) : under;
    // Stacked: drawn as it is, at its running total (its tag and price line would say that total, not its value: the
    // legend says that); the scale from zero (the stack's foot), a 100% one's from 0 to 100.
    const stack = stacked.get(layer.id);
    // (A stacked line: an area of no fill, its line where a line's is, the edge the band over it stops at (see
    // stackBands): a band shows its own layer's share alone.)
    const stackedLine = layer.type === "line" && Boolean(stack);
    // (A line or an area colored by its levels, not stacked: by its value, its color changing exactly where it crosses
    // a level (the library's zones), not date by date. Stacked, drawn at its total: date by date, by its own value.)
    const zoned = layer.byLevels && layer.levels?.length && (layer.type === "line" || layer.type === "area") && !stack ? levelSteps(layer) : null;
    // (One colored by a gradient, by its value alone (its place between its lowest and highest of all its history, not
    // stacked): the library's gradient, its colors in a look (see Gradient) each, a line's or an area's by height in one
    // stroke or fill, dots and bars each by its own. Date by date (`upToDate`, stacked, a cloud), painted as below.)
    // (Its colors from another or by its levels first, as its readings take them: see colorsAt.)
    const graded = layer.byGradient && layer.gradient && !layer.upToDate && !stack && !layer.colorFrom && !zoned && GRADED.has(layer.type) && result?.values ? layer.gradient : null;
    // (A signal's own series is never seen: its marks are on its layer's, see below. It's there as any layer's is, for
    // its values: the legend's, the crosshair's.)
    const signal = layer.type === "signal";
    // (Its values written as its scale writes them: a 100% stack's in percent.)
    const format = shares.has(layer.pane) ? SHARE_FORMAT : valueFormatOf([unitFor(layer, result)]);
    layerFormats.set(layer.id, format.value);
    const lineWidth = /** @type {any} */ (layer.width ?? (stack ? STACK_WIDTH : undefined));
    // (A standard series' look, the library's: shown or not; its value's tag reading the axis where it sits (a
    // stacked band's, its sum there, at its top: its own amount the legend's and the panel's); its line across; its
    // values' writer; edged in the card's color where it's outlined (over a heatmap, a cloud, another line). One
    // folded into a cloud (see foldedInto): no tag of its own, the cloud's range its legend's.)
    const shown = !layer.hidden && !signal;
    const tags = { hidden: !shown, tag: shown && !foldedLayers.has(layer.id), priceLine: Boolean(layer.priceLine) && !stack && !signal, format: format.value, outline: Boolean(layer.outline) && OUTLINED.has(layer.type) ? css("--card") : null };
    // (Its line's: width, dashes (by its width as set), steps or curved, its crosshair dot.)
    // (Its last period still running, a period's amount (see periodAmount): drawn partial, the library's way.)
    const partial = periodAmount(layer);
    const stroke = { width: lineWidth ?? 1.5, dash: dashFor(lineWidth, layer.dash), steps: lineTypeOf(layer) === LINE_TYPE.WithSteps, curve: lineTypeOf(layer) === LINE_TYPE.Curved, marker: MARKER.radius, stroke: true, dots: 0, partial };
    const pane = paneIndex.get(layer.pane) ?? 0;
    const candles = layer.type === "candles" && result?.opens;
    // Its colors date by date, where they vary (or come from another layer). (A cloud by its layers: each band's its
    // own, theirs (see Cloud), not one a date.)
    const banded = layer.type === "cloud" && Boolean(layer.byLayers && result?.members?.length && result.memberIds);
    const fromSource = candles || banded || zoned || graded ? null : paintOf(doc, layer, data);
    // (A date it has no color for: its own.)
    const dated = fromSource && ((/** @type {number} */ i) => ((name) => (name ? ink(name) : base))(fromSource(i)));
    // (A shaded signal's down stretches, in its second color: painted in its rows, as dated layers' colors are, so a new
    // color (a theme, its opacity) paints them again.)
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
    /** A look's line (an area's edge; else its one color). @param {Record<string, string>} look */
    const lineOf = (look) => look.line ?? look.color;
    // (Its gradient's colors each in its look, once; a part of that (a line's, a fill's, a fade's) a Gradient over all
    // its history, as colorsAt reads it.)
    const lookTable = graded ? gradientTable(graded).map(looks) : null;
    const [lo, hi] = lookTable && result?.values ? rangeOf(result.values) : [0, 0];
    // Its colors, as the library takes them (a line's; a band's fill and fade, a baseline's under its base too; candles'
    // parts; a cloud's, its bands').
    /** @type {Record<string, any>} */
    const paint = layer.type === "cloud" ? { color: alpha(base, op), bands: banded ? /** @type {string[]} */ (result?.memberIds).map((id) => alpha(ink(doc.layers.find((item) => item.id === id)?.color ?? layer.color), op)) : [] } : signal && layer.place === "shade" ? { color: alpha(base, 0.14 * op) } : candles ? ((up, down) => ({ up: hollowOf(layer) ? css("--card") : up, down, upWick: up, downWick: down, upBorder: up, downBorder: down }))(color, layer.colors === 1 ? color : below)
      : layer.type === "baseline" ? { line: color, under: below, fill: alpha(base, 0.24 * op), fade: alpha(base, 0.01 * op), below: alpha(under, 0.01 * op), belowFade: alpha(under, 0.24 * op) }
      : looks(base);
    let series;
    // (Drawn as before, the series as it was.)
    const again = kept.has(layer.id) ? was.get(layer.id) : undefined;
    if (!again) added = true;
    // (A standard one's: kept, or made, fitted as its stack is.)
    const fitting = stack && shares.has(layer.pane) ? "shares" : zeroFit(doc, layer, modeOf(doc, layer), Boolean(stack)) ? "zero" : null;
    const standard = (/** @type {"line" | "band" | "bars" | "candles"} */ kind, /** @type {Standard | undefined} */ kept) => fitSeries(kept ?? new Standard(kind, pane, layer.side), fitting, unfitted(doc, layer));
    if (signal && layer.place === "shade") {
      // Shaded: its stretches, its pane's whole height, faint, behind what's drawn (the library's Shade).
      series = again ?? new Shading(pane, layer.side);
      series.engine.hidden = !signalShown(layer, doc);
      series.style(paint.color);
    } else if (layer.type === "cloud") {
      // Its band alone: no label on the axis, no line across, no marker (its legend says its range); to its pane's edges
      // if it has them (both: on a scale of its own, unseen, its rows none of the pane's scales' business: drawn
      // whatever's on its side, or hidden, or how it's scaled).
      const [top, bottom] = EDGES.map((edge) => Boolean(layer.from?.includes(edge)));
      series = again ?? new Cloud(pane, layer.side, top && bottom && !cloudLayers(layer).length);
      series.engine.hidden = !shown;
      fitSeries(series, fitting, unfitted(doc, layer));
      series.style({ color: paint.color, top, bottom, banded, bands: paint.bands });
    } else if (heatLike(layer)) {
      // A heatmap: each day's URPD behind the lines (see Heat): no tag, no line across; nothing to its scale but
      // alone on it (nothing else drawn on its pane's side: a signal's marks, or one with no value its scale draws,
      // don't count), the price's then (its closes, see Heat).
      // (Its reference kept while it's the same map: a rebuild during a block's refresh draws on, see viewHeat.)
      const look = /** @type {HeatLook} */ (heatmaps.heatLook(doc, layer, data, op)), was = heatmaps.heatLooks.get(layer.id), known = heatKnown(layer);
      if (was && was.cohort === look.cohort && was.weight === look.weight && was.grid?.agg === look.grid?.agg && was.measure === look.measure) look.reference = was.reference;
      // (A profile's days last shown kept while they're read the same way: a group added draws the others at once.)
      if (was?.kind === "profile" && look.kind === "profile" && was.weight === look.weight && was.grid?.agg === look.grid?.agg) look.last = was.last;
      // (Of a holder group Bitview doesn't have (as drawn: a valid one highlighted in ⌘K previews): nothing drawn,
      // nothing asked for, no rows.)
      if (known) heatmaps.heatLooks.set(layer.id, look);
      else heatmaps.heatLooks.delete(layer.id), heatmaps.heatWants.delete(layer.id);
      // (A value its scale draws: a number, and above 0 on a log one.)
      const log = modeOf(doc, layer) === 1, drawable = (/** @type {number} */ value) => Number.isFinite(value) && (!log || value > 0);
      const alone = !doc.layers.some((other) => other !== layer && other.pane === layer.pane && other.side === layer.side && !other.hidden && !heatLike(other) && other.type !== "signal" && !unfitted(doc, other) && data.results.get(other.id)?.values?.some(drawable));
      // (Hidden, not drawn: nothing to ask for.)
      if (layer.hidden) heatmaps.heatWants.delete(layer.id);
      series = again ?? new Heat(pane, layer.side, layer.id, layer.type === "profile");
      series.engine.hidden = !shown;
      series.alone = alone;
      series.redraw();
      if (layer.type === "profile") heatmaps.profileShares(layer.id, series);
      // (Kept up with live only where drawn: see movePrice.)
      if (known) heatmaps.heatSeries.add(series);
      else heatmaps.heatSeries.delete(series);
    } else if (layer.type === "bubbles") {
      // (On the layer they're on, at its scale (see normalize: its pane, its side); on none, along their pane's middle,
      // on a scale of their own. Sized against the largest in view, or of all their history: see Bubbles.)
      const on = doc.layers.some((other) => other.id === layer.ref);
      series = again ?? new Bubbles(pane, layer.side, !on);
      series.engine.hidden = !shown;
      series.style({ up: alpha(base, 0.4 * op), upEdge: alpha(base, 0.8 * op), down: alpha(under, 0.4 * op), downEdge: alpha(under, 0.8 * op), size: layer.size ?? BUBBLE_SIZE, hide: layer.hide ?? BUBBLE_HIDE, scaleTo: layer.scaleTo === "all" ? "all" : "view", halo: layer.outline ? css("--card") : null });
    } else if (candles) {
      // (Zoomed out, the library draws its close as a line, in the layer's color, and carries its tag and price line.)
      series = standard("candles", again);
      styleSeries(series, { ...tags, candles: { ...paint, color, width: stroke.width, radius: BAR_CORNER } });
    } else {
      const radius = dotRadius(layer.width), base = baseOf(layer);
      // (Stacked, its fill a band: from its line down to its stack's fills under it, a stack a side (see
      // stackBands). An area's fill to the pane's foot, or its top where it's filled above its line.)
      // (Its levels' zones and its gradient, in their colors as its own are painted (see looks): a line's, an area's fill
      // and fade, bars'; none, its own color.)
      const zones = (/** @type {(look: Record<string, string>) => Record<string, string>} */ part) => (zoned ? zoned.map((step) => ({ value: step.value, ...part(looks(ink(step.color))) })) : null);
      const gradient = (/** @type {(look: Record<string, string>) => string} */ part) => (lookTable ? new Gradient(lookTable.map(part), lo, hi) : null);
      const lineZones = zones((look) => ({ color: lineOf(look) })), lineGradient = gradient(lineOf);
      const [kind, look] = layer.type === "dots" ? ["line", { line: { color: paint.color, width: 1.5, dash: [], steps: false, curve: false, marker: MARKER.radius, stroke: false, dots: radius, zones: null, gradient: lineGradient, partial: false } }]
        : layer.type === "area" || stackedLine ? ["band", { line: { ...stroke, color: paint.line, zones: lineZones, gradient: lineGradient }, band: { gradient: gradient((look) => look.fill), gradientFade: gradient((look) => look.fade), color: paint.fill, fade: paint.fade, below: null, belowFade: null, base: 0, to: layer.fill === "above" && !stackedLine ? "top" : "foot", steps: stroke.steps, curve: stroke.curve, zones: zones((look) => ({ color: look.fill, fade: look.fade })) } }]
        : layer.type === "histogram" ? ["bars", { bars: { color: paint.color, base, radius: stack ? 0 : BAR_CORNER, partial, gradient: lineGradient } }]
        // (A baseline's line by its base: its second color under it, its own from it up.)
        : layer.type === "baseline" ? ["band", { line: { ...stroke, color: paint.under, zones: [{ value: base, color: paint.line }], gradient: null }, band: { gradient: null, gradientFade: null, color: paint.fill, fade: paint.fade, below: paint.below, belowFade: paint.belowFade, base, to: "base", steps: stroke.steps, curve: stroke.curve, zones: null } }]
        : ["line", { line: { ...stroke, color: paint.color, zones: lineZones, gradient: lineGradient } }];
      series = standard(/** @type {"line" | "band" | "bars"} */ (kind), again);
      styleSeries(series, { ...tags, ...look });
      if (layer.type === "dots") dotted.set(layer.id, { series, width: layer.width, radius });
    }
    built.set(layer.id, { shape: layerShape(layer), shown: stack?.values ?? result?.values, stacked: Boolean(stack), painted: painting, ...(result?.opens ? { opens: result.opens, highs: result.highs, lows: result.lows } : {}), ...(result?.low ? { low: result.low } : {}) });
    // The same values (and not bars, their colors in them): nothing to set again. None now (an error, a formula
    // being written), a kept series is emptied.
    const before = kept.get(layer.id);
    // (A heatmap's rows: its timeline's dates and the price's closes there (its scale's), its days drawn from its
    // look, see viewHeat.)
    if (heatLike(layer)) series.set(heatKnown(layer) ? columnBy(data.times.length, (i) => result?.closes?.[i] ?? NaN) : EMPTY);
    // (Bubbles' sizes, each date's own value, and places, the value there of the layer they're on, as drawn (a
    // stack's, its total); on none, its pane's middle.)
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
      // (Its stretches: where it's up or down, to both its pane's edges; down, in its second color.)
      const shown = result.values, n = data.times.length, values = new Float64Array(n), colors = new Uint16Array(n);
      for (let i = 0; i < n; i++) (values[i] = Number.isFinite(shown[i]) && shown[i] !== 0 ? 0 : NaN), (colors[i] = shown[i] < 0 ? 1 : 0);
      series.set(values, colors, [paint.color, shadeDown]);
    } else if (result?.values && layer.type === "cloud") {
      // (Its scale's: log, none at or under zero; on one it doesn't fit, none. Which of its two is over: where that
      // changes, they cross. Of its pane's two edges alone, every date, its values none. By its layers: each one's
      // value (on a log scale, those over zero), its own lowest and highest drawn, where enough of them with its
      // edges are: its bands, see Cloud.)
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
      // (Its colors date by date, as a painted layer's: the live price paints its own point, see paintPoint.)
      const painted = dated && n ? { series, ...paletteOf(n, dated), look: (/** @type {string} */ c) => ({ color: alpha(c, op) }) } : null;
      if (painted) paintedLayers.set(layer.id, painted);
      paintSeries(series, painted);
    } else if (result?.values) {
      const shown = stack?.values ?? result.values;
      // A stacked candle: its open, high, low and close on the ones before it too.
      const at = (/** @type {number} */ value, /** @type {number} */ i) => (stack ? stack.foot[i] + value * (stack.scale?.[i] ?? 1) : value);
      // Log scales can't show zero or negative values: they become gaps.
      const log = modeOf(doc, layer) === 1;
      // (Painted date by date: a line's, dots' and bars' color, an area's line and fill, as its paint has them.)
      // (Its colors another layer's, they say something: its fill stronger, as a regime's.) (A baseline's two its own.)
      const look = !dated || layer.type === "baseline" ? null : looks;
      // (Its values as a column on the timeline, the library's (none where not finite (a formula's division by zero), or
      // not over zero on a log scale: copied only then), its colors and a trend's breaks as columns too; candles' four.)
      const times = layer.from && fromUnfit(doc, layer) ? [] : data.times;
      const drawable = (/** @type {number} */ value) => (value - value === 0 && (!log || value > 0) ? value : NaN);
      const values = times.length ? (log || shown.some((/** @type {number} */ value) => value === Infinity || value === -Infinity) ? shown.map(drawable) : shown) : [];
      if (candles) series.engine.setValues(candleColumns(result, shown, log, at));
      else setColumn(series, values, Boolean(stack) && (stackedLine || (layer.type === "area" && layer.fill !== "above")));
      const traced = candles ? null : series;
      (traced?.edge ?? traced?.engine)?.setStyle({ breaks: result.breaks ?? null });
      // (Bars below their base each in its second color, the rest in its own: two colors.)
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
  // Lines across, behind every series, labeled on the axis: one per value on a scale (a pane's side), in its color
  // and style; shared by several layers, gray, in the first one's style; shown while any of its layers is. None in
  // % and indexed panes: they show changes, not values. Each pane's side's the library's Levels.
  const lines = new Map();
  for (const line of across) {
    if (!Number.isFinite(line.value) || (line.mode === 1 && line.value <= 0)) continue;
    const key = `${line.pane}|${line.layer.side}|${line.value}`;
    (lines.get(key) ?? lines.set(key, []).get(key))?.push(line);
  }
  const levelsBefore = new Map(levelsKept), levelLines = new Map();
  levelsKept.clear();
  for (const [first, ...rest] of lines.values()) {
    // (Its value on the axis where one of them asks: a level's is fixed, the axis reads it already.)
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
    // (Its tags written as its layer's values are.)
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
  // (Each new series' sketch: its drawings, halving marks, a trend's ends.)
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
  // (A pane's shade is on a scale of its own: no side's.)
  const sides = new Set(doc.layers.flatMap((layer) => (edgesOf(layer).length > 1 ? [] : [layer.side])));
  doc.panes.forEach((pane, i) => {
    if (!panes[i]) return;
    // A side shows in every pane once any layer uses it: the plots line up.
    // A stack sits on its floor (a band's height is its value): no margin under it.
    // (Its values written as its layers' are, on a side with a series of its scale (none on a scale of its own): a
    // 100% stack's in percent; a side with none, the library's own. A % or indexed scale's changes in percent.)
    const fixed = shares.has(pane.id) ? SHARE_FORMAT : null;
    panes[i].setPercentFormat(PERCENT_FORMAT.value, PERCENT_FORMAT.ticks);
    for (const side of SIDES) {
      // (A side's ticks in the letters of its layers' unit, SI's for bytes and hash rates.)
      // (A heatmap: its rows' names on their axis (see drawRows), no values written.)
      const format = asRows(doc, pane) ? NO_FORMAT : fixed ?? valueFormatOf(doc.layers.filter((layer) => layer.pane === pane.id && layer.side === side && !layer.hidden && layer.type !== "signal").map((layer) => unitFor(layer, data.results.get(layer.id))));
      panes[i].setScale(side, { ...marginsOf(doc, pane, i), ...(keep ? {} : { mode: SCALE_MODES[paneMode(pane)] }) });
      if (!keep) panes[i].setRange(undefined, undefined, side);
      const used = panes[i].series.some((series, k) => (panes[i].sides[k] ? "left" : "right") === side && !series.overlay);
      panes[i].setFormat(used || format === NO_FORMAT ? format.value : libraryWords, used || format === NO_FORMAT ? format.ticks : null, side);
    }
  });
  // (Its right axis where a layer is on that side; none, the plots to the chart's right edge.)
  const axis = sides.has("right") ? RIGHT_AXIS : 0;
  if (engine.axis !== axis) (engine.axis = axis), engine.layout();
  engine.invalidate(true);
  // (Mid-drag, panes of the same shape keep their sizes: no need to measure them again. Laid from the chart, or the
  // look on it, whose copy this is: a copy is no other chart, see panesLaid.)
  if (!(holding && priorShape === panesShape)) sizePanes();
  // (Its axes as narrow as their labels let them again where its scales fit their data again, or its layers changed:
  // else they only widen, see the library's fitAxes.)
  const layerIds = (/** @type {Chart} */ of) => of.layers.map((layer) => layer.id).join();
  if (!keep || layerIds(doc) !== (prior ? layerIds(prior.doc) : "")) engine.fitAxes();
  nameAxis();
  // A chart opening comes back where it was left; at another interval, on the same time (on its preset, if it
  // showed one: it kept no view then). (Unless the view has since fallen off the data: an old one of a fine
  // interval, past its latest points.)
  fitSpacing(doc, data.times.length, same ? shownSpan() : null);
  // (Or the linked view it's taking, still its own: see openingView.)
  const opening = openingView(doc), taking = Boolean(retaking) && opening === retaking?.view;
  const left = !same || taking ? opening : null;
  // (A link's window, shown (its range, no view come with it): from now on, the chart's views as any chart's, the
  // linked one in line with it till it's moved.)
  const linkOpen = prior?.doc.id !== doc.id && linkWindow === doc.id;
  if (linkOpen) linkWindow = "";
  if ((left || linkOpen) && viewKey(doc.id) === LINKED) (linkedTook = taking ? /** @type {any} */ (retaking).at : viewsNow()[LINKED]?.at ?? 0), (handled = false);
  retaking = null;
  // (A view at the latest from where its range starts, its room after the latest that range's (a quarter either way:
  // that room is a width's, another tab's may be another): that range itself, as a range shows, at the latest as it
  // comes.)
  const onRange = Boolean(left?.latest && left.index === doc.index && data.times.length > 1) && ((preset, last, latest) => Math.abs(latest[0] - (preset.from - last)) < 1 && Math.abs(latest[1] - (preset.to - last)) <= Math.max(1, (preset.to - last) / 4))(presetWindow(doc, data), data.times.length - 1, /** @type {[number, number]} */ (left?.latest));
  const back = left && !onRange && !owed && data.times.length > 1 ? viewOn(left, doc.index, data.times) : null;
  // (Shown, drawn, and shown again: its price scales laid out for it first, else for the view before it (the
  // library's own, on a first draw), and the change of their width held against it: the library holds the bars where
  // they are on screen as an axis widens.)
  if (back && back.to >= back.from) {
    const target = within(back, "to");
    showSpan(target, EXACT_RANGE);
    engine.render(true);
    showSpan(target, EXACT_RANGE);
    setPresetShown(false);
    if (viewKey(doc.id) === LINKED) linkedAt = ((range) => range && viewRecord(range, data.times, doc.index))(shownSpan());
  } else if (owed && visible) {
    // (A range picked while this draw was on its way, at the same interval: glided to from the view as it was, as a
    // pick with its data at hand glides, not cut to once the data's in.)
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
  // Grown or shrunk at an end (a chunk reached for, a block, the oldest let go): the overview moves there from where
  // the data was, and the chart shows what came from the join out, at the same pace. Any other change lands both.
  if (!sameView) {
    morphFrom = null;
    endReveal();
  }
  if (sameView && !sameTimeline && priorTimes.length > 1 && data.times.length > 1 && !reduced()) {
    // Where the old first point sits in the new data (before its start, negative: the oldest let go).
    const offset = data.times[0] > priorTimes[0] ? -lowerBound(priorTimes, data.times[0]) : lowerBound(data.times, priorTimes[0]);
    morphFrom = { from: offset, to: offset + priorTimes.length - 1, ...overviewScale, at: performance.now() };
    // (More come the way one's going (a second chunk as the first shows): it goes on, its join where its data is
    // now, all past it what it shows; the other way, a reveal of its own.)
    if (inReveal) inReveal.join += offset;
    const outward = offset > 0 ? -1 : offset + priorTimes.length < data.times.length ? 1 : 0;
    if (outward && inReveal?.outward !== outward) reveal(outward < 0 ? offset - 0.5 : offset + priorTimes.length - 0.5, outward);
  }
  // (The build before let go of: the chart's own closures, made here, keep this scope, and so these, alive.)
  was.clear();
  kept.clear();
  showDrawings(doc);
  if (relit) spotlight(relit);
  legendsDue = true;
  requestAnimationFrame(layOutLegends);
  relayout();
}
/** Whether the chart's legends wait for their frame, the chart laid out (a picture made before it places them). */
let legendsDue = false;
function layOutLegends() {
  if (!legendsDue) return;
  legendsDue = false;
  // (Mid-drag, the panes as they were: their legends kept, their contents written again.)
  if (holding && legends().length === chart.panes.length - hiddenPanes(chart).size) updateLegends();
  else placeLegends();
  setGap();
}
/** The chart as ⌘K shows a look on it, while it does (sized as it shows). @type {Chart | null} */
let lookDoc = null;
// The person takes over: a press stops any glide (a wheel does by itself).
$("chart").addEventListener("pointerdown", () => engine.controller.stop(), { capture: true, passive: true });

// Legends at the top of each pane, and the values beside each layer, follow the crosshair.
let hoverTime = null;
/** A legend's scroll state (whether it scrolls, which edges fade) follows its width, as the chart's lays out. */
const legendWidth = new ResizeObserver((entries) => entries.forEach(({ target }) => fitSoon(/** @type {HTMLElement} */ (target))));
/**
 * Each pane's legend in its plot's frame: the one there kept (what it shows, its focus, written again only where it
 * changes: see updateLegends), one made for a pane new there, the others gone.
 */
function placeLegends() {
  // Until this chart is drawn, its panes don't exist yet: no legends to place (none at all, turned off).
  if (chartShown()) placeFull();
  if (chartShown()) {
    chart.panes.forEach((pane, i) => {
      // The library's, over its plot: a row that scrolls where its items don't fit.
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
/** A switch's width keeps its legend's row clear of it (none hidden). @param {HTMLElement} lens */
const leadFor = (lens) => /** @type {HTMLElement | null} */ (lens.nextElementSibling)?.style.setProperty("--lead", `${lens.offsetWidth ? lens.offsetWidth + 6 : 0}px`);
// (Its width changed later (a font come, a finger's wider buttons): on the next frame, not in the observer's round,
// where the row it moves, watched too (see legendWidth), would loop it.)
const lensWidth = new ResizeObserver((entries) => requestAnimationFrame(() => entries.forEach(({ target }) => leadFor(/** @type {HTMLElement} */ (target)))));
/**
 * A pane's two views' switch, at the start of its legend's row, which scrolls beside it (only a pane with two views):
 * named as its preset names them, shown and hidden with its legend (see .lens-switch). @param {HTMLElement} legend @param {Pane} pane
 */
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
/** The band the pointer's on, in a stacked pane (see bandAt): kept while the pointer's on its legend. */
let bandLit = /** @type {string | null} */ (null);
// The menus: the row pointed at (a mouse's; between rows, on a rule, it stays), or reached by keys (its edge its own:
// the row keys act on, wherever the pointer is); on another of their controls (the colors' swatches), none.
/**
 * The menu's submenus, one a level (a row's own menu beside the one it's in: 1 beside the menu, 2 beside that, as
 * deep as they go), each made as it's first needed, set up as the others (see whenSubmenu).
 */
const submenus = [$("submenu")], subSetups = /** @type {((menu: HTMLElement) => void)[]} */ ([]);
lightRows($("menu"));
whenSubmenu(lightRows);
/** Whether ⌘K's row was last moved to by the pointer (see move): its highlight glides there; by keys, at once. */
let pointedOption = false;
// ⌘K: the row ↵ runs, however it's come to (the arrows, the pointer, the list written anew); a finger's ⌘K has none
// (its keyboard in use, it has).
{
  const list = $("palette-list"), light = glides(list), coarse = matchMedia("(hover: none) and (pointer: coarse)");
  new MutationObserver(() => (!coarse.matches || command.keyboard()) && light(list.querySelector('[aria-selected="true"]'), !pointedOption)).observe(list, { childList: true, subtree: true, attributeFilter: ["aria-selected"] });
}
// The sidebar's rows, a chart's, a folder's, a group's head (a row too: it goes to its page; on the rail, its mark's own
// square lights it): the one pointed at (folding, none: the highlight would show whole over a row half shown); a row
// written anew (a folder opened, a chart), its own, by its key.
const sideLight = glides($("chart-list"), {
  again: (gone) => {
    const key = [.../** @type {HTMLElement} */ (gone.parentElement).attributes].find(({ name }) => /^data-(?:.+-row|group)$/.test(name));
    return key ? $("chart-list").querySelector(`[${key.name}="${CSS.escape(key.value)}"] > :is(.chart-item, .folder-item, .list-head-name)`) : null;
  },
  can: (item) => !foldOf.has(/** @type {Element} */ (item.closest("#chart-list > *"))),
});
/**
 * Where the mouse was last over the list, and whether its highlight waits for it to move: rows reordered under a still
 * pointer (Recent's, a chart opened from it) put it away, none lit as it lands (a row only slid there).
 */
let sidePointer = "", sideHeld = false;
const sidePoint = (/** @type {PointerEvent} */ event) => {
  if (event.pointerType !== "mouse") return;
  const at = `${event.clientX},${event.clientY}`;
  if (sideHeld && at === sidePointer) return;
  (sidePointer = at), (sideHeld = false);
  const row = closest(event, menus.railed() ? ".chart-row, .folder-row" : ".chart-row, .folder-row, .list-head");
  // (Rows meet, each reaching half the gap to the next (see .chart-list): off them, by a group's rule or past the
  // list's ends, none lit, not one left lit without its row's own (its count, its way, its ⋯).)
  sideLight(row ? row.querySelector(":scope > :is(.chart-item, .folder-item, .list-head-name)") : null);
};
$("chart-list").addEventListener("pointerover", sidePoint);
$("chart-list").addEventListener("pointermove", sidePoint);
$("chart-list").addEventListener("pointerleave", () => sideLight(null));
// Every group of rows a pointer goes along, as the sidebar's list, the menus and ⌘K: the sidebar's foot, the chart's
// tools and the Charts page's Undo and Redo, its foot's interval and ranges, ⌘K's foot, the layers, a layer's
// actions, the drawing tools.
glideRows(/** @type {HTMLElement} */ (document.querySelector(".side-foot")), ".nav-item");
glideRows(/** @type {HTMLElement} */ ($("undo").parentElement), ".head-actions > .cap");
glideRows(/** @type {HTMLElement} */ (document.querySelector(".home-undo")), ".cap");
glideRows($("inspector"), ".layer-actions > .nav-item");
glideRows($("stage-foot"), "#interval, #window, #ranges > button");
glideRows(/** @type {HTMLElement} */ (document.querySelector(".palette-foot")), ".foot-action");
glideRows($("layer-list"), ".layer-row");
glideRows($("belt"), ".belt :is(.cap, .btn)");
/** No values (made once). */
const EMPTY = /** @type {Float64Array} */ (new Float64Array(0));
/** A cloud's columns as a draw puts them in pixels (each a column's place, top and bottom, a crossing's height), kept and grown. */
const CLOUD = { xs: new Float64Array(64), highs: new Float64Array(64), lows: new Float64Array(64), ats: new Float64Array(64) };
/** A stack's range from zero: its own (`out` widened by it and zero), none in view nothing. (Made once.) */
const SPAN = new Float64Array(2);
const measure = /** @type {CanvasRenderingContext2D} */ (document.createElement("canvas").getContext("2d"));
let figuresFont = "";
/** The widest of some texts as the page sets them, at a weight and size, in whole pixels, figures as Inter's tabular ones (each a zero's width). */
function widest(texts, weight, size) {
  measure.font = `${weight} ${size}px ${(figuresFont ||= css("--sans"))}`;
  return Math.ceil(Math.max(0, ...texts.map((text) => measure.measureText(text.replace(/[0-9]/g, "0")).width)));
}
/** @param {ChartPoint} param */
function crossMoved(param) {
  if (param.input || !param.point) pinCross(null);
  // (Set by keys on a layer: its dot marked.)
  else if (crossSet) return void ((engine.crosshair.pointed = dotLineOf(crossSet.layer)), !crossHeld && !exporting && holdCross());
  const time = param.time === undefined ? null : /** @type {number} */ (param.time);
  // (The profiles whose bars it's among: not a picture's, see presentLegends.)
  const profiles = heatmaps?.heatLooks.size && !exporting ? heatmaps.profilesUnder(param) : [];
  // (A finger's kept where it is: on a profile's bars, or past the data (the slots after its last point), on the
  // view's last day, the one they show.)
  const kept = time ?? ((profiles.length || (param.logical ?? -1) >= engine.time.length) && resolved ? resolved.times[lastShown()] ?? null : null);
  if (fingers.size && param.point && kept !== null && param.pane !== undefined && !tool && !placing) {
    const series = [...drawn.values()].find((item) => !item.engine.hidden && seriesPane(item) === param.pane), price = seriesValue(series, param.point.y);
    fingerCross.at = series && price !== null && price !== undefined ? { price: Number(price), time: kept, series } : null;
  } else if (!param.point && fingerCross.at && !fingers.size) {
    const { price, time: at, series } = fingerCross.at, layer = layerOfSeries(series);
    pinCross(layer === null ? null : { price, time: at, layer });
    fingerCross.at = null;
    fingerCross.kept = at;
    // (Set here, said by the chart to no one: its bubble lit as a moved one's is.)
    lightBubble(at, seriesPane(series));
    // (Read on its day: on a profile's bars past the data, the view's last as it was, not the one it moves to.)
    if (hoverTime !== at) (hoverTime = at), valuesSoon();
    return crossOn(price, at, series);
  } else {
    fingerCross.at = null;
    // (Kept while it's the one shown (drawn again on its date, see crossSet): a pointer's, or none, ends it. One set or
    // cleared by Studio says nothing: see keyAt.)
    if (!param.point || time !== fingerCross.kept) fingerCross.kept = null;
  }
  // (The layer read: in a stacked pane the band under it, else a line near it (see bandAt), a finger's held too; its
  // legend names it (its pane's others fade only as its item, or its row, is pointed at: see spotlight). On the
  // legend over the plot, the one it was.)
  crossIn = Boolean(param.point);
  // (Where it is in its pane: a binned layer reads the price there, see hoveredIn; gone, read again, back at rest.)
  const wasAt = heatAt;
  heatAt = param.point && param.pane !== undefined ? { pane: param.pane, y: param.point.y } : null;
  if (heatmaps?.heatLooks.size && (heatAt || wasAt)) valuesSoon();
  // (The bar under it lit, the frontmost profile's.)
  const point = param.point;
  if (!exporting) pointProfile(point ? profiles.reduce((/** @type {ReturnType<typeof heatmaps.profileHit>} */ hit, layer) => hit ?? heatmaps.profileHit(layer, point), null) : null);
  // (Kept, nothing else pointed at: in its own pane's empty space (its legend's lit item never flickers as the pointer
  // crosses gaps); off its pane or the chart, none.)
  bandLit = bandAt(param) ?? ((param.point && !profileLit && bandLit && seriesPane(drawn.get(bandLit)) === param.pane) ? bandLit : null);
  // (In the plot, a tag pointed at before is left.)
  if (param.point) tagLit = null;
  namePointed();
  lightLegend(bandLit ?? profileLit?.layer ?? (param.point ? param.layer : null), false, true);
  lightBubble(param.point ? time : null, param.pane);
  if (time === hoverTime) return;
  hoverTime = time;
  valuesSoon();
}
engine.onCrosshair((index, pane, y, input, x, series) => crossMoved(chartPoint(index, pane, x, y, input, series)));
/** The ring about the bubble under the crosshair (see lightBubble), and where that was. */
const bubbleRing = Object.assign(document.createElement("span"), { className: "bubble-lit" });
/** Whether the pointer's in the plot (the crosshair's): off it, on an axis, its tags are read (see tagAt). */
let crossIn = false;
/** A tag pointed at (or none): its layer's legend item lit, its line's dot ringed. */
const lightTag = (/** @type {string | null} */ tag) => ((tagLit = tag), namePointed(), lightLegend(tag, false, true));
/** The layer whose value's tag the pointer's on, off the plot (none: null). */
let tagLit = /** @type {string | null} */ (null);
// (Off the plot, on the axis, the crosshair says nothing: the tag under it here. A finger points at nothing.)
engine.element.addEventListener("pointermove", (event) => !crossIn && event.pointerType !== "touch" && lightTag(tagAt(event.clientX, event.clientY)));
engine.element.addEventListener("pointerleave", () => !crossIn && lightTag(null));

// ── The chart by keys: what it shows, said; a crosshair taken bar by bar ──────

/** The date the keys' crosshair is on (a time in the data), or null: none. */
let keyTime = /** @type {number | null} */ (null);
/** With a tool in hand, the value ↑ and ↓ moved the keys' point to at its date (off its layer's line); another date, on the line again. */
let keyValue = /** @type {{ layer: string, value: number } | null} */ (null);
/** The keys' crosshair on a bar: in view (the view moves to it), on the chart, its values in the legends, said. */
function keyAt(/** @type {number} */ i) {
  if (!resolved) return;
  const time = resolved.times[i], range = shownSpan();
  keyTime = time;
  // (A profile's bar pointed at, let go: the keys' day read.)
  pointProfile(null);
  // (The one it was, drawn again as the view moves, is no one's: see crossSet.)
  pinCross(null);
  // (Out of sight, the view moves just enough to show it.)
  const first = range ? firstInView() : i, last = lastShown(), shift = i < first ? i - first : i > last ? i - last : 0;
  if (range && shift) showSpan({ from: range.from + shift, to: range.to + shift });
  // Its line where its point is (none drawn there, none); a point being placed click by click goes there too.
  const point = keyPoint(i);
  // (The chart says nothing of a crosshair set or cleared here: a finger's kept one, ended now, see fingerCross.)
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
/** The keys' crosshair let go: the readout back at the view's end. */
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
/** The keys' point moved up or down a step (a hundredth of its pane; with shift, a tenth), off its line. */
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
/** With a tool in hand, ↵: a point placed where the keys' crosshair is (a two-point kind's first, then its other: one clicked first too). */
function placeByKeys(/** @type {number} */ i) {
  const kind = tool, point = keyPoint(i), said = $("chart-said");
  if (!kind || !resolved) return;
  if (kind === "brush") return void (said.textContent = "The brush draws only with a pointer");
  if (!point) return void (said.textContent = "Nothing drawn at this date to place it on");
  // (As a press on the chart does: a measure shown goes.)
  clearMeasure();
  const time = resolved.times[i], spot = pointOf(time, point.value), part = BOXES.has(kind) ? "corner" : "end";
  const at = (/** @type {[number, number]} */ [when, value]) => `${pointLabel(when)}, ${amountOn(point.layer, value)}`;
  // (A path's next point, placed (on its last, it's done): the one after follows the keys.)
  if (kind === "path" && placing && draft) {
    draft = { ...draft, points: [...draft.points.slice(0, -1), spot] };
    return placePathPoint();
  }
  // A two-point kind's first (a path's): its other follows the keys (a click places it too).
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
  // (Both on one spot: the other's still to place.)
  if (PAIRED.has(kind) && !heatmaps.pointsApart(drawing)) return void (said.textContent = `Its other ${part} is on its first: move it first`);
  keyValue = null;
  // (Kept, the tool's in hand again for the next one.)
  takeUp(toolKept ? kind : "", toolKept, true);
  if (kind === "text") return startTyping(drawing, $("plot-keys"));
  keepDrawing(drawing);
  // (Said once it's there: a chart full, its notice says why not.)
  const reading = kind === "ruler" ? rulerReading(drawing) : null;
  if (kind === "ruler") said.textContent = reading ? `${reading.name}: ${reading.figures.join(", ")} over ${reading.span}` : "Nothing measured";
  else if (pen.drawingById(drawing.id)) said.textContent = `${TOOLS[kind].name} ${kind === "hline" ? `at ${amountOn(point.layer, point.value)}` : kind === "vline" ? `at ${pointLabel(time)}` : `from ${at(drawing.points[0])} to ${at(spot)}`}`;
}
$("plot-keys").addEventListener("focus", () => (describePlot(), tool && renderBelt()));
// (Left for another window, it's where it was when the window's back.)
$("plot-keys").addEventListener("blur", () => (document.hasFocus() && keyAway(), tool && renderBelt()));
$("plot-keys").addEventListener("keydown", (event) => {
  const n = resolved?.times.length ?? 0, range = shownSpan();
  // (Another chart loading, nothing to read yet; drawings selected, the arrows are theirs: see nudgeDrawings.)
  if (!n || !range || !chartShown() || $("chart").classList.contains("waiting") || event.metaKey || event.ctrlKey || event.altKey || (chosen.size && event.key.startsWith("Arrow"))) return;
  const on = keyTime === null ? -1 : indexOfTime(keyTime), first = firstInView(), last = lastShown(), from = on >= 0 ? on : last + 1;
  const step = event.shiftKey ? 10 : 1;
  const to = event.key === "ArrowLeft" ? from - step : event.key === "ArrowRight" ? (on < 0 ? first : from + step) : event.key === "Home" ? first : event.key === "End" ? last : null;
  // (A point being placed, a tool in hand, a measure shown: esc's for them first.)
  if (event.key === "Escape" && keyTime !== null && !tool && !placing && !measuring) {
    event.preventDefault();
    event.stopPropagation();
    return keyAway();
  }
  // With a tool in hand: ↵ places a point, ↑ and ↓ move it (none shown yet, it shows at the view's end first).
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
/**
 * The overview, the drawing belt and a preset's window, laid out again once a frame however many changes ask: the
 * plot's size (the window, the panel, a price axis grown with its labels), the navigator's, a new build.
 */
function relayout() {
  if (layoutFrame) return;
  layoutFrame = requestAnimationFrame(() => {
    layoutFrame = 0;
    // (A chart's card growing out of the grid or shrinking into it: nothing laid out against it as it moves, where
    // it'd be measured scaled; laid out once it's over, see morph.)
    if (document.body.hasAttribute("data-morph")) return void (layoutOwed = true);
    layoutOwed = false;
    // (A layout's motion, the chart's size held: only the belt follows its box; the navigator stretches as the chart
    // does (its brush in shares), the rest laid out once it ends (releaseSize).)
    if (sizeHeld) return void pen?.placeBelt();
    // (The lens shows what the chart draws next: after this frame's changes.)
    if (lensAt) lookAt(lensAt);
    // The chart's width, for spacing and the range controls.
    const width = engine.time.width;
    // (A drawing placed by taps, its first made: it starts over, its pane moved from under it (the screen turned,
    // the sidebar went); its second, else, would land as if it hadn't. Said once the belt's placed: its toast by it.)
    // (A path's points kept, its pane where it is now: as many as it has, starting over would lose them.)
    const restarted = width !== laidWidth && Boolean(placing?.clicks && draft) && !placingPath();
    if (restarted) {
      placing = draft = null;
      repaintDrawings();
      renderBelt();
    } else if (placing) {
      // (Its pane where it is now, its height changed alone too: a point placed lands where it's pressed.)
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
/** The chart's height, as last laid out. */
let chartHeight = 0;
/** The overview's box, as last laid out (see resized): none till it's first laid out. */
let overviewBox = { width: 0, height: 0 };
/** The panes' margins again, where a size has changed (theirs, a legend's): only those that changed. */
let marginsSet = "";
function fitMargins() {
  if (!chartShown()) return;
  const all = chart.panes.map((pane, i) => marginsOf(chart, pane, i)), key = JSON.stringify(all.map(({ top, bottom }) => [top.toFixed(3), bottom.toFixed(3)]));
  if (key === marginsSet) return;
  marginsSet = key;
  all.forEach((margins, i) => SIDES.forEach((side) => engine.panes[i]?.setScale(side, margins)));
  engine.invalidate(true);
}
// (A separator dragged, the panes laid out at each move: their room over their lines with them, as tall as their
// legends, not a share of the height they had (the lines moving when let go).)
engine.onLayout(() => engine.resizing && fitMargins());
addEventListener("pointerdown", (event) => press(event, { end: keepPaneSizes }), true);
/**
 * Whether the view moves: from a move (a pan, a fling, a zoom, a glide, the navigator, the wheel) till it's at rest,
 * a frame drawn with it where it was, nothing held down on it (the chart, the navigator), no glide on its way: the
 * moment a fling stops, a finger lifts. At rest, what waited for it: the rows' changes in full (see updateRowChanges),
 * more data reached for (see reachOut).
 */
let viewMoving = false, movedSince = false, restFrame = 0, wheelAt = -Infinity;
addEventListener("wheel", () => (wheelAt = performance.now()), { capture: true, passive: true });
const restSoon = () => void (restFrame ||= requestAnimationFrame(atRest));
function viewMoved() {
  viewMoving = movedSince = true;
  restSoon();
}
/** Let go of (a press, the navigator's brush): a frame more, for a fling to start in. */
const released = () => ((movedSince = true), restSoon());
function atRest() {
  restFrame = 0;
  if (movedSince || engine.controller.target || performance.now() - wheelAt < WHEEL_QUIET) return void ((movedSince = false), restSoon());
  // (Held, still or not: at rest once it's let go, see the release.)
  if (pressed.size || navBar.dragging) return;
  viewMoving = false;
  valuesSoon();
  reachOut();
  showAddress();
}
/** Row widths as last laid on: independent of the crosshair, forgotten when rows are written again. @type {{ revision: number, from: number, to: number, sizes: typeof rowSizes } | null} */
let roomsAt = null;
/**
 * Each row's change from the view's start to the readout's point. Its figures keep the room of their widest
 * value and change, so nothing beside them moves while the crosshair runs.
 */
function updateRowChanges() {
  if (!resolved || !panelShown()) return;
  const point = readPoint(), moving = viewMoving, since = /** @type {Set<number>} */ (new Set());
  for (const element of all$(".row-change[data-change]")) {
    const id = /** @type {string} */ (element.dataset.change), layer = layerById(id), result = resolved.results.get(id);
    if (!layer || !result) continue;
    // (A named value's change says nothing: its name says where it is.)
    const change = !layer.hidden && !result.error && !(named(layer) && bandOf(layer, valueAt(layer, point))) ? changeOver(layer, point) : null;
    // (Each written only where it changed: a crosshair's move restyles nothing else.)
    const name = `row-change ${change?.up === true ? "up" : change?.up === false ? "down" : ""}`, text = change ? chipText(change) : "";
    if (change) since.add(change.from);
    if (element.className !== name) element.className = name;
    // (Whether there's one marked on its figures (see .row-figures), as it comes or goes: its text read by no rule.)
    if (writeText(element, text)) /** @type {HTMLElement} */ (element.parentElement).classList.toggle("changed", Boolean(text));
    // While the view moves (a pan, a zoom), its change only: its tip and its room, read and measured from the view,
    // are set once the view rests (each frame, they'd cost the move its smoothness).
    if (moving) continue;
    const tip = change ? `${viewName(chart, layer)}\nChange from ${pointLabel(change.from)} to ${pointLabel(change.to)}` : undefined;
    if (element.dataset.tip !== tip) {
      if (tip) element.dataset.tip = tip;
      else delete element.dataset.tip;
      // (Its tip open: it says the new dates, or goes.)
      if (tipTarget === element && !$("tip").hidden) tip ? showTip(element) : hideTip();
    }
  }
  // (Over them, what they're since, once the view rests: the date they share, else each its own first in view.)
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

// ── Navigator: the whole history, and the window being shown ──────────────
/**
 * Where the overview was, just after the data grew or shrank at an end (a chunk reached for, a block, the oldest let
 * go): its span of points (in the new data's own) and its scale, which it moves from over the page's step, the brush
 * with it; the chart shows what came at the same pace (`reveal`). Null when it shows all the data as it is.
 */
let morphFrom = /** @type {{ from: number, to: number, lo: number, hi: number, at: number } | null} */ (null);
/** The overview's scale as last drawn, for a move to start from. */
let overviewScale = { lo: NaN, hi: NaN };
/** A chart has been revealed on this page: the next ones switched to wipe in at a page's pace. */
let revealed = false;
/** The overview as drawn (its shape): the navigator is painted from it. */
const overviewBase = document.createElement("canvas");
/** The screen's density the overview was drawn at (the navigator is painted at it; a new one draws it again). */
let overviewRatio = 1;
/** Live prices laid on the data so far (the overview's shape is drawn again for each). */
let liveMoves = 0;
/**
 * The overview's shape, kept while its series, scale, width and the live price are the same: its points (each
 * column's mean) and their range. A morph's frames, a relayout, only place them. @type {{ key: string, n: number, lo: number, hi: number, points: [number, number][] }}
 */
let overviewShape = { key: "", n: 0, lo: NaN, hi: NaN, points: [] };
function drawOverview() {
  const canvas = overviewBase, rect = overviewBox;
  // (Not shown, or no room for it (a phone on its side, the layers beside): nothing to draw.)
  if (!rect.width) return;
  const ratio = (overviewRatio = devicePixelRatio || 1);
  // (Its size set only when it changes: set at all, the canvas starts over.)
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
      // The shape: smooth (each pixel's column its mean: a sketch of the series, not its noise). (A few points in
      // one column, a mean is one point, no line: their lowest and highest then.)
      let points = meansOf(scaled, canvas.width);
      if (points.length < 2) points = command.columnsOf(scaled, canvas.width);
      overviewShape = { key, n: scaled.length, lo, hi, points };
    }
    let { lo, hi } = overviewShape;
    const points = overviewShape.points;
    if (Number.isFinite(lo)) {
      // The shape over the strip's whole height (the window's dates drawn over it, haloed, while it's pointed at).
      const span = overviewSpan(overviewShape.n), w = canvas.width, h = canvas.height, top = 4 * ratio, band = 3 * ratio;
      overviewScale = { lo, hi };
      // On its way: from the old scale too.
      if (morphFrom && span.t < 1 && Number.isFinite(morphFrom.lo)) (lo = morphFrom.lo + (lo - morphFrom.lo) * span.t), (hi = morphFrom.hi + (hi - morphFrom.hi) * span.t);
      const x = (i) => ((i - span.from) / Math.max(1, span.to - span.from)) * w;
      const y = (value) => h - band - ((value - lo) / (hi - lo || 1)) * (h - band - top);
      const color = ink(source.layer.color);
      // The shape: a line over a filled body.
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
/** The navigator's window at its least, in pixels: wide enough to take hold of, however short the view. */
// (By touch, a window a finger holds, and its edges grips one takes: wider.)
const BRUSH_MIN = TOUCH ? 36 : 24;
$("navigator").style.setProperty("--brush-min", `${BRUSH_MIN}px`);
/** The navigator's font as its canvas last took it (see paintNavigator). */
let navigatorFont = "";
/** Whether the pointer's over the navigator (its window's dates show then, and while it's dragged). */
let navigatorNear = false;
/**
 * The navigator as shown, on each move, back to front: the strip dimmed outside the window (no time marks: the
 * time axis right above says them); the overview, dimmed outside the window; the window's dates over all (while it's pointed at or dragged: at rest, the time axis
 * above says them), outlined in the strip's dark to read on any line. They
 * stand at the window's edges, each just outside it (just inside, clear of its grip, where the strip ends); where
 * they don't both fit so, as one ("7 Oct ’25 – 7 Oct ’26") beside the window or in it; else none.
 * @param {{ left: number, right: number } | null} lit the window, in shares of the navigator's width
 */
function paintNavigator(lit) {
  const canvas = $("overview"), context = canvas.getContext("2d"), ratio = overviewRatio;
  // (Moved to a screen of another density: drawn again for it, this frame painted as it was.)
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
  /** Outside the window (as the brush stands: no narrower than its least width), to dim. */
  const outside = new Path2D();
  const shown = shownDates();
  // (Said: what it's over (a finer interval loads a stretch, not all of history), what it shows, its keys.)
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
  // (The overview, dimmed outside the window: half faded out, the strip's own ground through it (a ground painted over
  // it, the card's, made the strip two greys where the field is translucent, the light's).)
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
/**
 * The view as left: what each resize of the box keeps (see
 * holdView), however many come and go (it isn't the view a resize made); and the view a resize made last (by the
 * plot), to tell it from one moved since.
 * @type {{ box: number, at: { from: number, to: number } } | null}
 */
let heldView = null, madeView = /** @type {{ from: number, to: number } | null} */ (null);
/** The chart's own width as last seen: a size change at it is a price scale's, not the box's. */
let lastBox = 0;
/** An image being made of the chart (its resizes the picture's: see chartImage). */
let exporting = false;
/**
 * The box resized: keep the view as it was before the change, including a range preset's endpoints.
 */
function holdView() {
  const width = engine.time.width, box = chartWidth(), was = heldView, resized = box !== lastBox;
  // (Only the chart's box resized: a price scale's labels can change without it.)
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
  // (The view as left: one moved in the box it was left in, or moved since a resize of the box made it; not one the
  // library or a resize made, see holdView.)
  ((range, width, box) => {
    if (!range || !width || exporting) return;
    lastBox ||= box;
    const made = madeView && Math.abs(range.from - madeView.from) < 0.05 && Math.abs(range.to - madeView.to) < 0.05;
    if (!heldView || heldView.box === box || (madeView && !made)) (heldView = { box, at: range }), (madeView = null);
  })(shownSpan(), engine.time.width, chartWidth());
  // (The lens shows the view as it's drawn next, moved by a key, a range, the navigator.)
  if (lensAt) lookAt(lensAt);
  nameAxis();
  if (writing) heatmaps.placeField();
  lightBubble(bubbleAt.time, bubbleAt.pane);
  viewMoved();
  placeBrush();
  fitDots();
  valuesSoon();
  // (Mid-resize, keep the preset selected until the new width's layout settles.)
  const shown = engine.time.width !== laidWidth && layoutFrame ? presetShown : showsPreset();
  if (shown !== presetShown) setPresetShown(shown);
  prefetch();
  keepView();
});
// (Reaching out waits for the view to rest (see viewMoving): mid-gesture, the chart keeps a pan's start by its last
// bar, so points loaded under it would make the view leap.)
let reaching = false;
// (Counted as it goes down, before the chart's own handlers take it.)
$("chart").addEventListener("pointerdown", (event) => {
  const id = event.pointerId;
  pressed.add(id);
  if (event.pointerType === "touch") fingers.add(id);
  press(event, { end: () => (fingers.delete(id), pressed.delete(id) && !pressed.size && released()) });
}, true);
// (A finger's crosshair kept where it was lifted: let go at the next touch on the plot, before a drawing's press
// reads it, see fingerCross. A mouse's or the keys' crosshair takes its place.)
$("chart").addEventListener("pointerdown", (event) => {
  fingerCross.dismissing = event.pointerType === "touch" && fingerCross.kept !== null && engine.element.contains(/** @type {Element} */ (event.target));
  if (fingerCross.dismissing) (fingerCross.kept = null), pinCross(null), engine.setCrosshair(NaN), lightBubble(null, undefined);
}, true);
// (A mouse on the same date takes it over too.)
engine.element.addEventListener("pointermove", (event) => event.pointerType !== "touch" && (fingerCross.kept = null));
// (A press on the chart lets a ruler's measure go: the next one, or none.)
$("chart").addEventListener("pointerdown", () => clearMeasure(), true);

// ── Formulas' windows, and the price scales glided ─────────────────────────────

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
  // Chunks that failed are asked for again (anywhere: a series can lack one the others have).
  for (const key of [...chunks.keys()]) {
    if (failedChunks.has(key) && key.includes(`@${index}#`)) (chunks.delete(key), failedChunks.delete(key));
  }
  let drew = false;
  try {
    drew = await draw();
  } finally {
    reaching = false;
  }
  // Another draw took its place: nothing to judge (that one shows what is).
  if (!drew) return;
  // Nothing more on that side: back as it was (drawn again, the far end it let go back too), and a rest before trying
  // that side again.
  const now = by < 0 ? resolved?.times[0] : resolved?.times.at(-1);
  if (reached !== undefined && now !== undefined && (by < 0 ? now >= reached : now <= reached)) {
    const dropped = by < 0 ? span.to < was.to : span.from > was.from, k = by < 0 ? span.from : span.to, names = drawSeries(chart);
    Object.assign(span, was, { rest: Date.now() + 5000 });
    if (dropped) void draw();
    // An older chunk that came empty for every series the chart reads (none failed; not judged by what's drawn: a
    // formula's result is trimmed of its leading zeros): none older asked for again, its series starting later.
    if (by < 0) {
      // (One not held any more, the cache's oldest let go, counts as not: only a series without chunks has none.)
      const empty = await Promise.all(names.map((name) => chunks.get(`${name}@${index}#${k}`)?.then((got) => got.data.every((value) => value == null), () => false) ?? !chunked(name, index)));
      if (empty.every(Boolean)) span.floor = { at: was.from, of: names.sort().join() };
    }
    return;
  }
  // Once the chart has moved the view onto its new points, at rest again: judged before, the view it had would ask
  // for the same chunk out again.
  restSoon();
}

// (Said as a finger does it, where it's one.)
$("navigator").dataset.tip = `Drag to pan, drag an edge to zoom, ${CLICK} to jump`;
/** The navigator's drags, the library's (see Navigator), on the overview as it's painted (its morph, too). */
const navBar = new Navigator($("navigator"), engine, {
  overview: () => ((n) => (n < 2 ? null : overviewSpan(n)))(resolved?.times.length ?? 0),
  length: () => (resolved?.times.length ?? 0) - 1,
  data: () => resolved?.times,
  within,
  glide: (range) => glide(range),
  window: $("brush"),
  least: BRUSH_MIN,
  // (Each visible grip ends 5px from the edge; touch gets 2px more without taking the window's body.)
  grip: TOUCH ? 7 : 5,
  // (Taken: the overview lands where it was going.)
  pressed: () => (morphFrom = null),
  dropped: () => (placeBrush(), released()),
});
for (const [type, near] of /** @type {const} */ ([["pointerenter", true], ["pointerleave", false]])) $("navigator").addEventListener(type, () => ((navigatorNear = near), placeBrush()));
// Focused (tabbed to, pressed or dragged, after Go to a date), its keys move the view as its drags do: ← → a tenth of
// it along (⇧, all of it), ↑ ↓ in and out (⇧, twice as far), Home and End to the start and to the latest.
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

// ── Drawing: lines, boxes, text and strokes, on the chart ─────────────────
const SANS = css("--sans");
const measurer = /** @type {CanvasRenderingContext2D} */ (document.createElement("canvas").getContext("2d"));
// (A face loaded late, as Inter's rest for a label in another script: the chart's words measured and drawn again in it,
// as its tags and panes measure theirs at each frame.)
document.fonts.addEventListener("loadingdone", () => {
  tagShapes.clear();
  repaintDrawings();
  engine.invalidate(true);
});
/** The drawings hidden from view: a view, not an edit (back on another chart, with a tool taken, or any selected). */
let drawingsHidden = false;
/** The drawings hidden or shown again (`tell`: by a key or ⌘K, said where the drawing tools don't show it). */
function hideDrawings(/** @type {boolean} */ hidden, tell = false) {
  drawingsHidden = hidden;
  if (hidden) chooseDrawings([]);
  renderBelt();
  repaintDrawings();
  renderLayers();
  if (hidden && tell && $("belt").hidden) toast(`${plural(chart.drawings?.length ?? 0, "drawing")} hidden`, { label: "Show", run: () => hideDrawings(false) });
}
/** The drawings shown again (a tool taken, one selected): the panel's row says so too. */
function revealDrawings() {
  if (!drawingsHidden) return;
  drawingsHidden = false;
  renderLayers();
}
/**
 * A measure just taken with the ruler: a glance, not a drawing (never kept, nor in a link), shown until the next press
 * on the chart, esc, or another chart. @type {Drawing | null}
 */
let measuring = null;
/** The chart whose drawings were shown last (its measure kept while it's the one built again). */
let drawingsOf = "";
/** The measure gone, if there was one (and said so): at once from what a press finds too (the one clearing it goes on). */
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
/** A chart's drawings, on the series just built (each given its sketch once). @param {Chart} doc */
function showDrawings(doc) {
  if (doc.id !== drawingsOf || (measuring && !drawn.has(measuring.layer))) measuring = null;
  drawingsOf = doc.id;
  // (The layer a drawing was being placed on, gone: it goes.)
  if (placing && !drawn.has(placing.host.id)) stopDrawing();
  shownDrawings = doc.drawings ?? [];
  repaintDrawings();
}

/** The tool a press on the chart draws with, or none. @type {DrawingKind | ""} */
let tool = "";
/** A drawing being placed: its pane, its layer, where its press was; `clicks`, placed click by click. */
let placing = null;
/** Drawings taken hold of: as they were (those selected, moved together; or one, by a handle), from where. */
let moving = null;
/** The drawings selected: one, or several (⇧-click, a box around them, ⌘A). Apart from the selected layer. @type {Set<string>} */
let chosen = new Set();
/** Drawings being moved or reshaped, as they show until let go (then they're the chart's), by id. @type {Map<string, Drawing>} */
let reshaped = new Map();
/** A drawing being placed, as it shows until it's let go (then it's the chart's). @type {Drawing | null} */
let draft = null;
/** A text being typed in its field (not drawn under it): the drawing, whether it's new; a line's label being typed (its line still drawn); `cancel`: let go by esc, as it was. */
let writing = null, typingIn = "", labelTyping = "";
/** Drawn for a picture: nothing selected or pointed at. */
let capturing = false;
/** The drawings as last built. @type {Drawing[]} */
let shownDrawings = [];

/**
 * The tool in hand kept after each drawing (`kept`, by a double-click on it): drawn again and again, till esc;
 * else one drawing, and the chart pans again. Text is one at a time (its typing takes the keys).
 */
let toolKept = false;
/**
 * The magnifier: a round lens over the chart, what's under the pointer twice as large (the chart's own pixels: crisp
 * on a screen of two pixels a point), the pointer itself at its center (its cursor hidden); by touch, above the
 * finger (which would cover what it shows), and the chart held still while pressed. Shown, never kept: a tool as
 * Select is (picking another, or the cursor, puts it down).
 */
let magnifying = false, lensAt = /** @type {{ x: number, y: number, touch: boolean } | null} */ (null), lensFrame = 0;
/** Where the mouse was last (the magnifier shows there at once). */
let pointerAt = /** @type {{ x: number, y: number } | null} */ (null);
/** The lens on a point of the page (none: away), drawn on the next frame: after the chart's own, which it shows. */
function lookAt(/** @type {{ x: number, y: number, touch: boolean } | null} */ at) {
  lensAt = magnifying ? at : null;
  // (Asked again, it's drawn after the frames asked for meanwhile: the chart's, which it shows.)
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
  // Each of the chart's layers (its panes', its axes', the drawings'), in order, the part under the lens (not the
  // lens itself: it would show itself again, and again).
  for (const source of $("canvas").querySelectorAll("canvas")) {
    if (source === canvas) continue;
    const box = source.getBoundingClientRect();
    const x0 = Math.max(box.left, at.x - span), y0 = Math.max(box.top, at.y - span), x1 = Math.min(box.right, at.x + span), y1 = Math.min(box.bottom, at.y + span);
    if (!box.width || !box.height || x1 <= x0 || y1 <= y0) continue;
    const sx = source.width / box.width, sy = source.height / box.height;
    context.drawImage(source, (x0 - box.left) * sx, (y0 - box.top) * sy, (x1 - x0) * sx, (y1 - y0) * sy, (x0 - at.x + span) * k, (y0 - at.y + span) * k, (x1 - x0) * k, (y1 - y0) * k);
  }
  // Its center marked, a ring haloed to read on anything (the pointer's there).
  const mid = size / 2, r = 3.5 * ratio;
  context.lineWidth = 3 * ratio;
  context.strokeStyle = alpha(css("--card"), 0.85);
  context.beginPath();
  context.arc(mid, mid, r, 0, 2 * Math.PI);
  context.stroke();
  context.lineWidth = 1.25 * ratio;
  context.strokeStyle = css("--ink");
  context.stroke();
  // On the pointer; a finger's above it, else beside it, else below it: the first on screen and clear of it.
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
/** The Select tool: a box dragged around drawings selects them (once; as ⌘ and a drag does, anytime). */
let selectingArea = false;
/**
 * What's in hand, one thing at a time: a drawing tool (`kept`: drawn with again and again, see toolKept), Select,
 * the magnifier, or none (the pointer the pointer again). Whatever was under way with the one before let go of.
 */
function takeUp(/** @type {DrawingKind | "select" | "magnifier" | ""} */ what, kept = false, auto = false) {
  undoTook = "";
  // (Picked by you, counted: put down after a drawing, or by an undo or redo, not.)
  if (!auto) picks++;
  pathTaken = [];
  pathResume = null;
  const kind = what === "select" || what === "magnifier" ? "" : what, lens = what === "magnifier";
  toolKept = Boolean(kind) && kept && kind !== "text";
  // A tool in hand, nothing's selected: the belt shows what it'll draw (and the drawings, if hidden).
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
    // (On, at once where the pointer rests over the chart: no move needed to show it.)
    lookAt(lens && pointerAt && !TOUCH && $("chart").matches(":hover") ? { ...pointerAt, touch: false } : null);
  }
  hover(null);
  renderBelt();
  repaintDrawings();
}
/**
 * The drawings selected (none: the selection goes). A drawing selected puts down what was in hand and lets the selected
 * layer go: one thing's in focus at a time.
 */
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

/**
 * The belt, over the chart: the cursor, Select and the tools, always the same; then what's in hand, which adapts:
 * with a tool, the look its next drawing takes (the pen's, one for every tool); with drawings selected, their look and
 * what to do with them. On, it stays; off, it shows only while a tool's in hand or a drawing is selected. Its grip moves it to any side of the plot, anywhere along it: along the top or the foot it lies, along
 * the left or the right it stands, turning as it's carried there.
 */
let beltShown = "", beltKeys = false;
/** The belt's tools, in order: the cursor (none in hand), Select (a box around drawings), each drawing's (the ruler last), the magnifier. */
const BELT_TOOLS = ["cursor", "select", ...KINDS, "magnifier"];
function renderBelt() {
  // (Its state as the belt's: a path being placed, its points are steps.)
  syncUndo();
  const belt = $("belt"), drawings = tool ? [] : /** @type {Drawing[]} */ ([...chosen].map(pen.drawingById).filter(Boolean)), drawing = drawings.length === 1 ? drawings[0] : undefined;
  const was = belt.hidden || belt.classList.contains("held") || reduced() ? null : belt.getBoundingClientRect();
  // (Its parts where they were, by their ids: kept ones move from there, see reshapeBelt.)
  const partsWere = was ? new Map(all$("[id]", belt).map((part) => [part.id, part.getBoundingClientRect()])) : null;
  const hide = home || !chart.layers.length || !(prefs.belt.on || tool || selectingArea || magnifying || drawings.length || beltKeys);
  // (Put away, it goes as everything put away goes (see fadeOut); what it stood in for (a phone's foot) back once it's
  // gone, not under it as it goes.)
  // (Shown again as it goes, its copy's gone: one, never two.)
  if (!hide) cutFade(belt);
  const going = hide && !belt.hidden ? fadeOut(belt, [{ opacity: 1, transform: "none" }, { opacity: 0, transform: "translateY(4px)" }]) : undefined;
  belt.hidden = hide;
  const stage = /** @type {HTMLElement} */ (belt.closest(".stage"));
  stage.toggleAttribute("data-belt-shown", !hide || fading.has(belt));
  going?.then(() => belt.hidden && !fading.has(belt) && stage.removeAttribute("data-belt-shown"));
  // (Its button gone, the magnifier with it.)
  if (belt.hidden && magnifying) return takeUp("");
  $("draw-toggle").setAttribute("aria-pressed", String(prefs.belt.on));
  if (belt.hidden) {
    // (What had the focus in it lets go: hidden, it can't keep it; the chart's keys take it, not the page.)
    if (belt.contains(document.activeElement)) /** @type {HTMLElement} */ (document.activeElement).blur(), $("plot-keys").hidden || $("plot-keys").focus({ preventScroll: true });
    return;
  }
  // (Carried, it turns as it goes: its side is set on release.)
  if (!belt.classList.contains("held")) belt.classList.toggle("upright", pen.upright(prefs.belt.dock));
  const kinds = new Set(tool ? [tool] : drawings.map((item) => item.kind)), look = (/** @type {string} */ part) => pen.lookFor(part, drawings);
  const pressed = (/** @type {string} */ each) => (each === "cursor" ? !tool && !selectingArea && !magnifying : each === "select" ? selectingArea : each === "magnifier" ? magnifying : tool === each);
  const tools = BELT_TOOLS.map((each) => {
    const { name, tip = name, key, kbd, icon } = pen.beltTool(each);
    // (The ways of looking, the magnifier and the drawings' eye, apart from the ways of drawing.)
    return `${each === "magnifier" ? '<i class="belt-rule"></i>' : ""}<button type="button" class="cap ${each === tool && toolKept ? "kept" : ""}" id="belt-${each}" data-tool="${each}" aria-pressed="${pressed(each)}" aria-label="${name}${each === tool && toolKept ? ", kept" : ""}" data-tip="${escapeHtml(tip)}" data-kbd="${kbd}">${icon}${key ? `<kbd>${key}</kbd>` : ""}</button>`;
  }).join("") + (chart.drawings?.length ? `<button type="button" class="cap" id="belt-hide" data-belt="hide" aria-pressed="${drawingsHidden}" aria-label="${drawingsHidden ? "Show the drawings" : "Hide the drawings"}" data-tip="${drawingsHidden ? "Show the drawings" : "Hide the drawings (they're kept)"}" data-kbd="⇧ H">${drawingsHidden ? ICONS.eyeOff : ICONS.eye}</button>` : "");
  const button = (/** @type {string} */ part, /** @type {string} */ label, /** @type {string} */ body, menu = true, kbd = "") => `<button type="button" class="cap" id="belt-${part}" data-belt="${part}" ${menu ? 'aria-haspopup="menu"' : ""} aria-label="${escapeHtml(label)}" data-tip="${escapeHtml(label)}" ${kbd ? `data-kbd="${kbd}"` : ""}>${body}</button>`;
  // Each look shown as the first drawing that has it has it; set, it's set on each that has it.
  const has = (/** @type {string} */ part) => [...kinds].some((each) => pen.hasLook(each, part));
  const context = !kinds.size ? "" : [
    ...(has("color") ? [
      button("color", `Color: ${colorName(look("color").color)}`, `<i class="swatch" style="--c:${escapeHtml(cssColor(look("color").color))}"></i>`),
      button("opacity", `Opacity: ${Math.round((look("opacity").opacity ?? 1) * 100)}%`, ICONS.opacity),
    ] : []),
    ...(has("width") ? [button("width", `Width: ${look("width").width}px`, ICONS.weight), button("dash", `Line style: ${dashOf(look("dash").dash)[1]}`, dashGlyph(dashOf(look("dash").dash)[2], dashOf(look("dash").dash)[3]))] : []),
    // (Words' size and background: a text's, a labelled line's; with a tool, the Text tool's.)
    ...((tool ? tool === "text" : Boolean(pen.wordsIn(drawings))) ? ((words) => [
      button("size", `Size: ${SIZE_NAMES[words.size ?? TEXT_SIZE]}`, `<span class="belt-size" style="--s: ${({ 11: 10, 13: 12, 16: 14, 22: 17 })[words.size ?? TEXT_SIZE] ?? 12}px">A</span>`),
      button("back", `Background: ${pen.backOf(words.back)[1]}`, pen.backChip(words.back ?? "", words.color)),
    ])(tool ? prefs.pen : /** @type {Drawing} */ (pen.wordsIn(drawings))) : []),
    ...([...kinds].some((each) => THROUGH.has(each)) ? [button("ends", `Ends: ${pen.endName(look("ends").start)}, ${pen.endName(look("ends").end).toLowerCase()}`, pen.endsGlyph(look("ends").start, look("ends").end))] : []),
    ...(drawings.length && [...kinds].some((each) => pen.hasLook(each, "color")) ? ['<i class="belt-rule"></i>'] : []),
    // (A phone's: what's done to them (its text, its label, duplicate, remove) behind one ⋯, as a layer's are, the
    // looks in view: nothing past the strip's edge at any width. See selectionActions.)
    ...(drawings.length && phone.matches ? [
      button("actions", drawings.length > 1 ? `More for ${drawings.length} drawings` : "More for the drawing", ICONS.more),
    ] : drawings.length ? [
      ...(drawing?.kind === "text" ? [button("text", "Edit the text", ICONS.pencil, false)] : []),
      ...(drawing && LINES.has(drawing.kind) ? [button("label", TOUCH ? (drawing.label ? "Edit the label" : "Add a label") : drawing.label ? "Edit the label (or double-click it)" : "Add a label (or double-click the line)", ICONS.label, false)] : []),
      // (Each flip that changes how they show, as their menu offers it.)
      ...heatmaps.flipsOf(drawings.map((item) => item.id)).map(([across, way, icon]) => button(`flip-${across ? "x" : "y"}`, `Flip ${way}`, ICONS[icon], false)),
      button("duplicate", drawings.length > 1 ? `Duplicate ${drawings.length} drawings` : "Duplicate", ICONS.duplicate, false, keyText("⌘D")),
      button("remove", drawings.length > 1 ? `Remove ${drawings.length} drawings` : "Remove", ICONS.trash, false, "⌫"),
    ] : []),
  ].join("");
  // (The plot in focus, as the keys place it.)
  const byKeys = document.activeElement === $("plot-keys"), other = BOXES.has(tool) ? "other corner" : "other end";
  // (A path's next point, then the last again (or esc) to finish it.)
  const next = tool === "path" ? (byKeys ? "Arrows to its next point, ↵ to place it, ↵ again to finish" : `${capitalized(CLICK)} its next point, the last again to finish`) : "";
  const how = selectingArea ? "Drag a box around drawings to select them" : tool && (placing?.clicks ? next || (byKeys ? `Arrows to its ${other}, ↵ to place it` : `${capitalized(CLICK)} its ${other}`) : `${byKeys ? TOOLS[tool].keys : TOOLS[tool].tip}${toolKept ? ", as many as you like" : ""}`);
  // The tools, then what's in hand; drawings selected, only theirs (esc, or a click on the chart, and the tools are back).
  const html = `<button type="button" class="grip" id="belt-grip" aria-haspopup="menu" aria-label="Move the drawing tools" data-tip="Drag it to any side of the chart">${ICONS.grip}</button>
          <div class="belt-row ${drawings.length || context ? "" : "spread"}">${drawings.length ? "" : `<div class="belt-tools" role="group" aria-label="Tools">${tools}</div>`}
          ${context ? `${drawings.length ? "" : '<i class="belt-rule"></i>'}<div class="belt-context" role="group" aria-label="${escapeHtml(tool ? `Next ${TOOLS[tool].noun}` : drawing ? pen.drawingNames().get(drawing.id) ?? "" : `${drawings.length} drawings`)}">${context}</div>` : ""}</div>
          ${how ? '<p class="belt-caption"><span></span><kbd>esc</kbd></p>' : ""}`;
  // (Its words set in place: the plot taking the focus or giving it up, as a belt's button is pressed, changes only
  // them, and the button pressed stays the one clicked.)
  const writeHow = () => {
    const caption = belt.querySelector(".belt-caption span");
    if (caption && caption.textContent !== how) caption.textContent = how || "";
  };
  if (html !== beltShown) {
    const focus = focusKey(belt), slid = belt.querySelector(".belt-row")?.scrollLeft ?? 0;
    belt.innerHTML = beltShown = html;
    // (On a phone, where its row slid to: still there.)
    const row = belt.querySelector(".belt-row");
    if (row && slid) row.scrollLeft = slid;
    writeHow();
    drawKeys(belt);
    restoreFocus(belt, focus);
    // (Its button in focus gone with what it was for (a drawing removed): the chart's keys take it, not the page.)
    if (focus && document.activeElement === document.body && !$("plot-keys").hidden) $("plot-keys").focus({ preventScroll: true });
    pen.rovingBelt();
    // The button in focus gone with what it was for (a drawing removed from it): D's keys end with it.
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
/** The belt's one stop for Tab (a toolbar's: the arrows within): the button last in focus, else the tool in hand. */
let beltStop = "";
withRest(() => {
  $("belt").addEventListener("focusin", (event) => {
    beltStop = /** @type {HTMLElement} */ (event.target).id;
    pen.rovingBelt();
  });
  phone.addEventListener("change", () => (renderBelt(), pen.rovingBelt()));
});
/**
 * The drawing tools put away, now: off (shown again only while drawing, or with a drawing selected), and whatever's in
 * hand with them, else they'd stay for it (a path being placed kept as it is, a tool, the magnifier, a selection,
 * their keys).
 */
function hideBelt() {
  heatmaps.finishPath();
  pen.putDown();
  chooseDrawings([]);
  beltKeys = false;
  $("belt").classList.remove("keys");
  if (prefs.belt.on) (prefs.belt.on = false), savePrefs();
  renderBelt();
}
/**
 * D: the belt, as its button: on (it stays) with its keys at hand (a tool's key picks it, arrows go along it, esc
 * lets go); D again, while they are, puts it away (whatever's in hand with it).
 */
function focusBelt() {
  if ($("belt").classList.contains("keys")) {
    hideBelt();
    return leaveBelt();
  }
  // (To draw: what's selected is let go, the tools come back.)
  chooseDrawings([]);
  if (!prefs.belt.on) (prefs.belt.on = true), savePrefs();
  beltFrom = document.activeElement instanceof HTMLElement && !$("belt").contains(document.activeElement) ? document.activeElement : null;
  beltKeys = true;
  renderBelt();
  $("belt").classList.add("keys");
  /** @type {HTMLElement | null} */ ($("belt").querySelector('[data-tool][aria-pressed="true"]'))?.focus();
}
/** Where D was pressed: the belt's keys done, the focus goes back there (if it's still there to have it). */
let beltFrom = /** @type {HTMLElement | null} */ (null);
function leaveBelt() {
  const back = beltFrom;
  beltFrom = null;
  // (Nothing in focus as D was pressed, a pointer's way: nothing again.)
  if (back === document.body) return void (/** @type {HTMLElement | null} */ ($("belt").querySelector(":focus"))?.blur());
  // (Gone since: the chart, its keys, else Draw: never the page.)
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
  // A tool's key picks it only once D was pressed (its keys show then); else keys are the chart's, as anywhere.
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
// An end picked: set where the popover is, which stays as it is (the other end a click away; the focus where it was).
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
  // (The grip by keys, ↵ or space: the sides it docks on, as dragging it does; by a pointer, it's dragged.)
  if (button.id === "belt-grip") return void (event.detail === 0 && menus.menuOf(button, pen.dockChoices(), null, "", { title: "Dock the drawing tools" }));
  const { tool: each, belt: part = "" } = button.dataset, drawings = [...chosen], drawing = chosen.size === 1 ? pen.drawingById(drawings[0]) : undefined, look = pen.lookFor(part);
  // (Its second click, a double-click: the tool kept in hand, for several; text, one at a time.)
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
// A grid of swatches: the arrows go across it and up and down it (in a menu, before its own, which go down a list).
document.addEventListener("keydown", (event) => {
  const target = /** @type {HTMLElement} */ (event.target), grid = /** @type {HTMLElement | null} */ (target.closest?.(".swatch-grid"));
  const step = grid && { ArrowLeft: -1, ArrowRight: 1, ArrowUp: -1, ArrowDown: 1 }[event.key];
  if (!grid || !step) return;
  event.preventDefault();
  event.stopPropagation();
  // (At its edge, an arrow stays.)
  const swatches = all$("button", grid), at = swatches.indexOf(target), columns = getComputedStyle(grid).gridTemplateColumns.split(" ").length;
  const next = swatches[at + step * (upDown(event) ? columns : 1)];
  // (Past its foot, in a menu: the menu's next row, its Custom.)
  if (next) next.focus();
  else if (event.key === "ArrowDown") /** @type {HTMLElement | undefined} */ (all$("#menu button").find((button) => grid.compareDocumentPosition(button) & Node.DOCUMENT_POSITION_FOLLOWING && !grid.contains(button)))?.focus();
}, true);
withRest(() => {
  inspector.sliders($("menu"), { valueOf: (id) => ((look) => (look ? SLIDERS[id].of(look) : undefined))(pen.lookFor(id === "draw-width" ? "width" : "opacity")), set: (id, value) => pen.setDrawSlider(/** @type {"draw-width" | "draw-opacity"} */ (id), value) });
});
// Its grip carries it, turning as it nears a side; let go, it docks on the side it'd come to rest by, thrown (restOf),
// where along it it would.
$("belt").addEventListener("pointerdown", (event) => {
  const belt = $("belt");
  if (!closest(event, ".grip") || event.button !== 0) return;
  event.preventDefault();
  const box = belt.getBoundingClientRect(), canvas = $("canvas").getBoundingClientRect(), offset = { x: event.clientX - box.left, y: event.clientY - box.top };
  belt.classList.add("held");
  // (The belt holds the pointer, not the grip: a redraw mid-way replaces the grip.)
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
    // Let go: on the side it's on, where along it its middle is. (Taken away, by a touch that scrolled: back.)
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

// ── Bubbles: a series' sizes on another layer ───────────────────────────────

// ── Heatmaps: the URPD, a day a column ──────────────────────────────────────
withRest(() => {
  document.addEventListener("visibilitychange", () => document.hidden && [...heatmaps.heatWaiters].forEach((wake) => wake()));
});
/** The price the pointer's at in a pane (its index, its height there), while it's over the chart. @type {{ pane: number, y: number } | null} */
let heatAt = null;
/**
 * The profile bar pointed at (see profileHit): its layer and its group; its group's bars drawn full, the others
 * faint, its legend item the group's (see paintProfileItems).
 * @type {{ layer: string, cohort: string } | null}
 */
let profileLit = null;
/** A bar pointed at (or none), as its group changes: lit (its profile drawn again), its legend item and value. @param {{ layer: string, cohort: string } | null} hit */
function pointProfile(hit) {
  const was = profileLit;
  if (was?.layer === hit?.layer && was?.cohort === hit?.cohort) return;
  profileLit = hit;
  for (const id of new Set([was?.layer, hit?.layer])) if (id) drawn.get(id)?.redraw();
  paintProfileItems();
  valuesSoon();
}

/** A press the chart doesn't also take (it would pan): its mousedown or touchstart, swallowed. */
let swallow = false;
/** The pointer a gesture under way is (a drawing placed, moved, a box drawn): another finger's moves nothing. */
let gesturePointer = -1, gestureTouch = false;
function take(event) {
  event.preventDefault();
  event.stopPropagation();
  // (Its press's own focus move held back: what had the focus outside the chart lets go of it, as a press anywhere
  // else makes it: the keys the chart's (the navigator, pressed before, would take the arrows).)
  if (document.activeElement instanceof HTMLElement && !$("chart").contains(document.activeElement)) document.activeElement.blur();
  swallow = true;
  gesturePointer = event.pointerId;
  gestureTouch = event.pointerType === "touch";
  $("chart").setPointerCapture(event.pointerId);
  // (One gesture's press at a time: one abandoned (stopDrawing) before its pointer's up ends nothing that came after.)
  stopGesture();
  stopGesture = press(event, { captor: $("chart"), end: gestureEnded });
}
/** The press of the gesture under way let go of, unheard. */
let stopGesture = () => {};
/**
 * A gesture's press over (see take): the lens a finger held gone; by its release, what it drew, moved or boxed
 * placed; else what it was drawing or moving let go of, as it was.
 * @param {PointerEvent | null} release
 */
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
// Capture phase, before the chart: a press that draws, or takes a drawing, doesn't pan.
// The magnifier's lens on the pointer (a mouse's, over the chart; a finger's, while it presses); gone as it leaves.
// (A mouse's by its mousemove: the chart's own comes before it, and asks for its frame first, so the lens drawn
// after shows that frame, not the one before.)
addEventListener("mousemove", (event) => (pointerAt = { x: event.clientX, y: event.clientY }), { passive: true });
addEventListener("mousemove", (event) => magnifying && !lensAt?.touch && event.target instanceof Node && $("chart").contains(event.target) && lookAt(closest(event, ".legend, button") ? null : { x: event.clientX, y: event.clientY, touch: false }));
// (A finger's: the one pressing, another's moves and lifts nothing.)
$("chart").addEventListener("pointermove", (event) => lensAt?.touch && event.pointerId === gesturePointer && lookAt({ x: event.clientX, y: event.clientY, touch: true }));
// (A mouse's gone as it leaves; a finger's with its press, see gestureEnded.)
$("chart").addEventListener("pointerleave", (event) => magnifying && event.pointerType === "mouse" && !lensAt?.touch && lookAt(null));
// (A scale reset by a double-click on its axis, the pointer still: the lens after it.)
$("chart").addEventListener("dblclick", () => lensAt && lookAt(lensAt));
$("chart").addEventListener("wheel", () => lensAt && lookAt(lensAt), { passive: true });
withRest(() => {
  $("chart").addEventListener("pointerdown", (event) => {
    // (Any press on the chart let go of with no release (cancelled, its touch lost, the window left) while no gesture's
    // held: what's placed click by click goes with it.)
    press(event, {
      end: (release) => {
        if (!release && !heatmaps.gesturing()) stopDrawing();
      },
    });
    // (A second finger, while one draws or moves: neither a drawing's nor the chart's, which would pinch or pan. A
    // finger's move, another finger down too: the move let go, the drawing as it was, nothing pinched.)
    if (heatmaps.otherPointer(event)) {
      event.preventDefault();
      event.stopPropagation();
      swallow = true;
      if (moving && gestureTouch && event.pointerType === "touch") stopDrawing();
      return;
    }
    // (A finger come down while another is on the chart (panning it): the chart's, its pinch, whatever's under it. One
    // letting a finger's crosshair go, the chart's too (it pans), unless a tool's in hand: see fingerCross.)
    if (event.pointerType === "touch" && [...fingers].some((id) => id !== event.pointerId)) return;
    if (fingerCross.dismissing && !tool) return;
    if (writing) $("draw-text").blur();
    // (A legend takes its own presses: a click on a name, ⌘ or ⇧ too, is the layer's, not a drawing's.)
    if (event.button !== 0 || !resolved?.times.length || closest(event, ".legend, #full")) return;
    // (The magnifier by touch, on the axes too: the lens above the finger as it moves; the chart doesn't pan meanwhile.)
    if (magnifying && event.pointerType !== "mouse") return void (take(event), lookAt({ x: event.clientX, y: event.clientY, touch: true }));
    const pane = paneAt(event.clientY);
    if (!pane) return;
    const at = pen.plotPoint(event.clientX, event.clientY, pane.rect);
    // A press on an axis scales it, whatever's drawing.
    if (at.x < 0 || at.x > engine.time.width) return;
    // A box around drawings: the Select tool's, or ⌘ and a drag (a chord: the keys held don't come up).
    if (!tool && (selectingArea || pen.boxKey(event))) {
      take(event);
      hideHints();
      boxing = { pane: pane.index, rect: pane.rect, start: at, now: at };
      return heatmaps.showBox();
    }
    if (tool) {
      // Placed click by click: this press ends it, at its release; its end where it's pressed (a finger still till
      // it lifts moves nothing: a mouse's hovered there first).
      if (placing) {
        take(event);
        // (Its pane read as it is now: laid out anew since (a height changed), the press lands where it's pressed.)
        Object.assign(placing, { rect: paneRect(placing.host) ?? placing.rect, down: at, held: true, keys: false });
        return placeTo(event);
      }
      // (Drawn to a value within reach: a line's ends, a box's corners, a level; not a ruler's: it reads its dates.
      // Nowhere to place it, the press is the chart's: it pans.)
      const host = pen.hostIn(pane.index), point = host && pen.pointFrom(at, host, { free: tool === "brush" || tool === "text", pull: (POINTED.has(tool) && tool !== "ruler") || tool === "hline" ? !event.altKey : false });
      if (!host || !point) return;
      take(event);
      placing = { rect: pane.rect, host, down: at, held: true, clicks: false, stroke: [{ ...at, point }] };
      pathResume = null;
      if (tool !== "text") draft = pen.newDrawing(tool, host.id, POINTED.has(tool) ? [point, [...point]] : [point]);
      return repaintDrawings();
    }
    const hit = pen.drawingAt(event.clientX, event.clientY, pen.reachOf(event));
    // A press on nothing pans; a click there lets the selected drawings go (at its release, unmoved; a ⇧-click that
    // missed one changes nothing).
    if (!hit) return void (chosen.size && !event.shiftKey && heatmaps.pressAt(event, ""));
    // By touch, one not selected is the chart's still: a drag over it pans (a finger's wide reach on it moves nothing);
    // a tap selects it, then it's taken as a pointer takes it. (A pen points as a mouse does.)
    if (event.pointerType === "touch" && !chosen.has(hit.drawing.id)) return void heatmaps.pressAt(event, hit.drawing.id);
    take(event);
    const id = hit.drawing.id;
    // ⇧: in or out of the selection.
    if (event.shiftKey) return chooseDrawings(chosen.has(id) ? [...chosen].filter((other) => other !== id) : [...chosen, id]);
    // Taken by its body, it moves with every other selected; by one of its points, it alone is reshaped.
    if (!chosen.has(id)) chooseDrawings([id]);
    // (A line across's dot is its body: it has nothing to reshape. A label taken slides along its line, alone.)
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
  // Double-click a text: typed into, where it is; a line: its label, typed where the click was (on it: typed into).
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
  // A right-click on a drawing: its menu, as a layer's row has.
  $("chart").addEventListener("contextmenu", (event) => {
    const hit = tool || placing ? null : pen.drawingAt(event.clientX, event.clientY);
    // (Else a layer's, as its row's: its legend item, or its line or tag under the pointer (the one its legend lights).)
    const item = !hit && !tool && !placing ? /** @type {HTMLElement | null} */ (closest(event, "#chart .legend [data-select]")) : null;
    const layer = item?.dataset.select ?? (!hit && !tool && !placing && litLayer && layerById(litLayer) ? litLayer : null);
    if (layer && !TOUCH) return void (event.preventDefault(), inspector.layerMenu(item ?? $("plot-keys"), layer, { x: event.clientX, y: event.clientY }));
    if (!hit) return;
    event.preventDefault();
    if (!chosen.has(hit.drawing.id)) chooseDrawings([hit.drawing.id]);
    heatmaps.drawingMenu({ x: event.clientX, y: event.clientY });
  });
});
/** What's being drawn or moved, let go of as it was. */
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
/** The point being placed follows the pointer (a stroke goes on, steadied). */
function placeTo(event) {
  const host = placing.host;
  if (tool === "brush" && draft) {
    const coalesced = event.getCoalescedEvents?.() ?? [];
    for (const each of coalesced.length ? coalesced : [event]) {
      // Each point half the way from the last to the pointer: the hand's tremor taken out.
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
/** Let go: a click on a line's or a box's first point leaves its second to the next click; else it's placed. */
function placed(event) {
  const at = pen.plotPoint(event.clientX, event.clientY, placing.rect), still = Math.hypot(at.x - placing.down.x, at.y - placing.down.y) < 4;
  // A path's points, one a click (its first stretch dragged, or not), each the next's start; the last clicked again
  // (a double-click's second), it's done.
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
  // (Kept, the tool's in hand again for the next one.)
  takeUp(again ? kind : "", again, true);
  if (kind === "text") {
    // (Its middle where the click was.)
    const point = pen.pointFrom(at, host, { free: true });
    return void (point && startTyping(pen.newDrawing("text", host.id, [point])));
  }
  if (kind === "brush" && drawing) {
    stroke.push({ ...at, point: pen.pointFrom(at, host, { free: true }) });
    let kept = heatmaps.simplify(stroke, 0.75);
    // (Tolerance grows until it fits: only a long scribble gets there.)
    for (let tolerance = 1.5; kept.length > MAX_STROKE; tolerance *= 1.5) kept = heatmaps.simplify(stroke, tolerance);
    drawing = { ...drawing, points: /** @type {[number, number][]} */ (kept.map((each) => each.point).filter(Boolean)) };
  }
  if (!drawing) return repaintDrawings();
  keepDrawing(drawing);
}
/**
 * A path's point being placed (its last, the pointer's or the keys'), placed: where the one before it is (within
 * `reach`: a finger's wider), the path's done; else the next follows from it.
 */
function placePathPoint(reach = 4) {
  if (!draft) return;
  const placed = /** @type {[number, number]} */ (draft.points.at(-1)), before = draft.points.at(-2);
  if (before && !heatmaps.pointsApart({ ...draft, points: [before, placed] }, reach)) return void heatmaps.finishPath();
  pathTaken = [];
  draft = { ...draft, points: [...draft.points, [...placed]] };
  // (As many points as a link keeps: at the most, it's done.)
  if (draft.points.length > MAX_STROKE) return void (heatmaps.finishPath(), notice(`A path holds up to ${MAX_STROKE} points`));
  heatmaps.pathMoved();
}
/** A path being placed: the points taken back from it (⌫, ⌘Z), the last last, for ⇧⌘Z to put back. @type {[number, number][]} */
let pathTaken = [];
/** A path whose first point was taken back too: for ⇧⌘Z to place it again from there, till anything else is done. @type {{ placing: any, draft: Drawing, taken: [number, number][] } | null} */
let pathResume = null;
/** A path being placed, its last point placed taken back (its first, it's let go). Whether one was. */
function unplacePath() {
  if (!placingPath() || !draft) return false;
  // (Its first: let go, ⇧⌘Z bringing it back with the rest taken back.)
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
/** A path being placed, the last point taken back from it put back. Whether one was. */
function replacePath() {
  // (Its first taken back, it's placed again from there.)
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
/** A drawing placed, into the chart: a ruler's measure shown (it passes); its two points on one spot, nothing to draw. */
function keepDrawing(/** @type {Drawing} */ drawing) {
  if (!drawing.points.length || (PAIRED.has(drawing.kind) && !heatmaps.pointsApart(drawing)) || (drawing.kind === "path" && drawing.points.length < 2)) return repaintDrawings();
  if (drawing.kind === "ruler") return void ((measuring = drawing), repaintDrawings());
  heatmaps.addDrawn(drawing);
}
let undoTook = "", picks = 0;
/** Let go of drawings: moved, they're the chart's where they are (one undo); not moved, it was a click: they're selected. */
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
/** A box being dragged around drawings: its pane, where it started, where it is now. */
let boxing = null;
/** Let go of a box: the drawings of its pane it touches are selected (a click, no box: none); the Select tool's put down. */
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
/**
 * Typing a text where it's drawn: ↵ (or a click away) keeps it, ⇧↵ starts a line, esc leaves it as it was (a new one
 * not made), as esc does to what's typed elsewhere; emptied, it goes. ↵ or esc gives the focus `back` (what opened it
 * by keys: the plot, the belt's button).
 * @param {Drawing} drawing @param {HTMLElement | null} back
 */
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
/**
 * Typing a line's label where it sits (`at`: where it goes, a share of the way along): ↵ keeps it, esc leaves it as
 * it was (the focus `back`, as a text's); emptied, it goes.
 * @param {Drawing} drawing @param {HTMLElement | null} back
 */
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
// (The window left (a phone's app switched), it stays as it is till the window's back: see leftOpen.)
$("draw-text").addEventListener("blur", () => leftWith($("draw-text"), endTyping) || endTyping());
$("draw-text").addEventListener("keydown", (event) => {
  // (Enter mid-composition is the input method's. A label is one line: ⇧Enter puts it down too.)
  if (composing(event) || !((event.key === "Enter" && (!event.shiftKey || writing?.label)) || event.key === "Escape")) return;
  event.preventDefault();
  const back = writing?.back;
  if (event.key === "Escape" && writing) writing.cancel = true;
  $("draw-text").blur();
  // (The belt's button drawn again as it's kept: the one in its place.)
  const to = back?.isConnected ? back : back?.id ? document.getElementById(back.id) : null;
  if (to?.checkVisibility()) to.focus({ preventScroll: true });
});

// ── Layers ────────────────────────────────────────────────────────────────
// ── A group of choices, one stop for Tab: in it, the arrows ───────────────────
/** A segmented control (its pressed one is its value) or a set of radios: one place in the tab order, the arrows within. */
const GROUP = '.tabs[role="group"], .color-slots[role="group"], [role="radiogroup"]';
const groupButtons = (/** @type {Element} */ group) => /** @type {HTMLButtonElement[]} */ (all$(":scope > button", group)).filter((button) => !button.disabled && !button.hidden);
// Entered from outside, the one in effect takes the focus; whichever has it is the group's one stop.
document.addEventListener("focusin", (event) => {
  const button = /** @type {HTMLElement} */ (event.target), group = button.matches?.("button") ? button.parentElement?.closest(GROUP) : null;
  if (!group || button.parentElement !== group) return;
  const from = /** @type {HTMLElement | null} */ (event.relatedTarget), buttons = groupButtons(group);
  const chosen = buttons.find((other) => other.getAttribute("aria-pressed") === "true" || other.getAttribute("aria-checked") === "true");
  // (By Tab, onto the one in effect; a click or a tap lands where it's made.)
  if (from && !group.contains(from) && chosen && chosen !== button && button.matches(":focus-visible")) return chosen.focus();
  for (const other of buttons) other.tabIndex = other === button ? 0 : -1;
});
// The arrows move along it (round past its ends), what's chosen waiting for ↵ or space (the swatches, a grid, have
// their own arrows: see below).
document.addEventListener("keydown", (event) => {
  const button = /** @type {HTMLElement} */ (document.activeElement), group = button?.parentElement?.closest('.tabs[role="group"], .color-slots');
  if (!group || button.parentElement !== group || event.metaKey || event.ctrlKey || event.altKey) return;
  const step = KEY_STEPS[event.key];
  // (Down and up the four: two along.)
  const along = group.matches(".color-slots.four") && upDown(event) ? 2 : 1;
  if (!step) return;
  const buttons = groupButtons(group), at = buttons.indexOf(/** @type {HTMLButtonElement} */ (button));
  if (at < 0) return;
  event.preventDefault();
  event.stopPropagation();
  buttons[stepIn(at, step * along, buttons.length, true)].focus();
}, true);

/**
 * The dialogs open, in the order they opened: the last on top, as the top layer lays them (the page lays them in
 * its own order). See showDialog.
 * @type {HTMLElement[]}
 */
const dialogsShown = [];
/** The dialog on top, if one's open. */
const topDialog = () => dialogsShown.at(-1) ?? null;
/** Words for a screen reader alone, on the status line where the focus is (the page's, or an open dialog's). */
function say(/** @type {string} */ text) {
  const line = /** @type {HTMLElement | null} */ ((topDialog() ?? document.body).querySelector(":scope > [data-said]"));
  if (!line) return;
  line.textContent = "";
  requestAnimationFrame(() => (line.textContent = text));
}
// The panel's views: a rule under their head once something has scrolled under it.
/** How far a panel's scrolled, for the rule under its head: 0 at its top, 1 from 16px. */
const scrolledBy = (/** @type {HTMLElement} */ view, /** @type {number} */ top) => view.style.setProperty("--scrolled", String(Math.min(1, Math.max(0, top) / 16)));
for (const view of all$(".panel-view")) view.addEventListener("scroll", () => scrolledBy(view, view.scrollTop), { passive: true });
// (The list of layers scrolls inside its view: its head is ruled by its scroll.)
$("layer-list").addEventListener("scroll", () => scrolledBy($("layers-view"), $("layer-list").scrollTop), { passive: true });
// A status line in the page and in each dialog, there from the start: what a toast says is read from it.
for (const host of [document.body, ...all$("dialog")]) {
  const said = document.createElement("p");
  Object.assign(said, { className: "vh" });
  said.setAttribute("role", "status");
  said.dataset.said = "";
  host.append(said);
}
/** What each keyed list last wrote: its parts, [key, html], each one element. */
const listWritten = new WeakMap();
/** A list's rows as last written, each with its key (see writeList). @param {HTMLElement} container */
const listRows = (container) => new Map((listWritten.get(container) ?? []).map(([key], i) => [/** @type {HTMLElement} */ (container.children[i]), key]));
/** Where a written list's rows are now, by key (see slide). @param {HTMLElement} container */
const listPlaces = (container) => ((rows) => grid.placesOf([...rows.keys()], (row) => rows.get(row)))(listRows(container));
/**
 * A written list's rows gliding from where they were (`was`, see listPlaces) to where they are: those in sight, before
 * or after (none other's seen moving), each over the others on its ground, one come fading in as they uncover it.
 * @param {HTMLElement} list @param {Map<string | undefined, number>} was
 */
function glideList(list, was) {
  const sight = list.getBoundingClientRect(), after = listRows(list);
  const seen = (/** @type {number | undefined} */ top) => top !== undefined && top < sight.bottom && top > sight.top - 60;
  grid.slide([...after.keys()].filter((row) => seen(row.getBoundingClientRect().top) || seen(was.get(after.get(row)))), (row) => after.get(row), was, { fade: true, layout: true });
}
/**
 * A list written again only where it changed: each part kept where its key is (its element, its hover, what it's
 * doing: a row lit stays lit, a folder opened above doesn't draw it again, a press on it still clicks), brought to its
 * new html in place where that changed (see rewriteInPlace), the others written, all in the parts' order; the focus put back
 * on its like where its own was written. Whether anything was written.
 * @param {HTMLElement} container @param {[string, string][]} parts
 */
function writeList(container, parts) {
  const was = listWritten.get(container);
  listWritten.set(container, parts);
  const focus = focusKey(container);
  // (Written as it was last, its elements one to a part, each key once: each found by its key; else, none to keep.)
  const unique = (/** @type {[string, string][]} */ list) => new Set(list.map(([key]) => key)).size === list.length;
  const kept = new Map(was && container.children.length === was.length && unique(was) && unique(parts) ? was.map(([key, html], i) => [key, { html, element: /** @type {Element} */ (container.children[i]) }]) : []);
  if (!kept.size) {
    container.innerHTML = parts.map(([, html]) => html).join("");
    restoreFocus(container, focus);
    return true;
  }
  const make = document.createElement("template");
  let wrote = false;
  // (Those gone, gone first: the others then where they are, not moved past them (a move loses a press on it).)
  const keys = new Set(parts.map(([key]) => key));
  for (const [key, { element }] of kept) if (!keys.has(key)) element.remove(), kept.delete(key), (wrote = true);
  parts.forEach(([key, html], i) => {
    const old = kept.get(key);
    kept.delete(key);
    // (One folded away (inert, its fold's end held), shown again by another way than its fold: written anew. One
    // still folding, as it is: its fold goes on.)
    let element = old && !(/** @type {HTMLElement} */ (old.element).inert && foldOf.get(old.element)?.playState !== "running") && (old.html === html || rewriteInPlace(old.element, old.html, html)) ? old.element : null;
    if (old && element && old.html !== html) wrote = true;
    if (!element) {
      make.innerHTML = html;
      element = /** @type {Element} */ (make.content.firstElementChild);
      if (old) old.element.remove();
      wrote = true;
    }
    // (One kept moved as it is, where the browser can: its focus, its hover kept.)
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

/**
 * An element brought from the html it was written from (`was`) to new html, itself kept, and what's in it where its
 * shape is the same: a piece the html didn't change kept as it is (what the page set on it since too), one it changed
 * given the new html's attributes alone (as written anew: what the page set on it, set again by the page), one whose
 * shape changed, or that the page wrote again, written anew. Whether it could be (its root the same tag).
 * @param {Element} element @param {string} was @param {string} html
 */
function rewriteInPlace(element, was, html) {
  const read = (/** @type {string} */ markup) => ((make) => ((make.innerHTML = markup), make.content.firstElementChild))(document.createElement("template"));
  return patchNode(element, read(was), read(html));
}
/** @param {ChildNode} live @param {ChildNode | null} a what it was written from @param {ChildNode | null} b what it's now @returns {boolean} */
function patchNode(live, a, b) {
  if (!a || !b || live.nodeName !== a.nodeName || a.nodeName !== b.nodeName) return false;
  if (a.isEqualNode(b)) return true;
  if (!(a instanceof Element && b instanceof Element && live instanceof Element)) return ((live.nodeValue = b.nodeValue), true);
  const from = [...a.childNodes], to = [...b.childNodes], now = [...live.childNodes];
  // (Its children not as written (the page's own added, a field typed in): written anew, as a whole.)
  if (now.length !== from.length) return false;
  for (const { name } of [...live.attributes]) if (!b.hasAttribute(name)) live.removeAttribute(name);
  for (const { name, value } of [...b.attributes]) if (live.getAttribute(name) !== value) live.setAttribute(name, value);
  to.forEach((node, i) => (i < now.length ? patchNode(now[i], from[i], node) || live.replaceChild(node, now[i]) : live.append(node)));
  for (const node of now.slice(to.length)) node.remove();
  return true;
}

// ── Mini charts: each chart's own shape, and each layer's, from their data ───────────────────────
/**
 * A chart's mark: up to three lines, each with its kind (none: a line), its second color and its base; a heatmap's
 * wash, bubbles' dots (see validMark); and each layer's own, with its base (a baseline's, bars'), in hundredths of
 * the height.
 * @typedef {{ washes: [number, string, string, number, number][], dots: [string, number, number][], lines: ([string, (number | null)[]] | [string, (number | null)[], string, string, number] | [string, (number | null)[], string | null, string | null, number | null, (string | null)[]])[], marks: Record<string, (number | null)[]>, bases?: Record<string, number> }} Thumb
 */
// One pass per chart squeezes every layer, with its formulas and its pane's scale, to a few dozen points: up to
// three of them make the chart's mark, each one alone makes its layer's mark in the legend. Monthly data is
// enough, except where windows run (sma, stdev…: a short one would be a point there): those use the chart's
// own interval. Only the drawings are kept, by chart, with what they were drawn from and the day; a change, or a
// new day, redraws them.

const THUMBS = "bitview-studio:thumbs";
/** The presets' marks kept in this browser, the last drawn: a visit shows them at once, not drawn anew first (as a chart's, see THUMBS); let go of first, as the thumbs are, when storage is full. */
const PRESET_MARKS = "bitview-studio:preset-marks";
const THUMB_POINTS = 28;
/** The drawings' version: one kept from before it (drawn by older rules) isn't reused. Drawings only, never saved charts or preferences. */
const MARK_VERSION = "4:";
const validDrawingKey = (key) => typeof key === "string" && key.startsWith(MARK_VERSION);
const validYs = (ys) => Array.isArray(ys) && ys.length >= 2 && ys.length <= THUMB_POINTS && ys.every((y) => y === null || (typeof y === "number" && y >= 0 && y <= 1));
const validLevel = (y) => typeof y === "number" && y >= 0 && y <= 1;
const MARK_KINDS = new Set(["area", "baseline", "bars", "dots"]);
/** A mark's lines, as kept: each its color, its points, its kind (and a two-sided one's other color, its level), its dates' colors. */
const validLines = (/** @type {any} */ lines) => Array.isArray(lines) && lines.length <= 3
  && lines.every((line) => Array.isArray(line) && isColor(line[0]) && validYs(line[1]) && (line[2] == null || (MARK_KINDS.has(line[2]) && isColor(line[3]) && validLevel(line[4])))
    && (line[5] === undefined || (Array.isArray(line[5]) && line[5].length === line[1].length && line[5].every((color) => color === null || isColor(color)))));
/**
 * A mark, as kept: its lines (see validLines); a heatmap's wash about one of them (that line, the colors over it and
 * under it, its band's foot and top); a few of the largest bubbles, dots (each its color, across, up).
 */
const validChartMark = (/** @type {any} */ mark) => typeof mark === "object" && mark !== null && validLines(mark.lines)
  && Array.isArray(mark.washes) && mark.washes.length <= 1 && mark.washes.every((wash) => Array.isArray(wash) && Number.isInteger(wash[0]) && wash[0] >= 0 && wash[0] < mark.lines.length && isColor(wash[1]) && isColor(wash[2]) && validLevel(wash[3]) && validLevel(wash[4]))
  && Array.isArray(mark.dots) && mark.dots.length <= 4 && mark.dots.every((dot) => Array.isArray(dot) && isColor(dot[0]) && validLevel(dot[1]) && validLevel(dot[2]));
const validThumb = (thumb) => validDrawingKey(thumb?.key) && validChartMark(thumb)
  && typeof thumb.marks === "object" && thumb.marks !== null && Object.values(thumb.marks).every(validYs)
  && (thumb.bases === undefined || (typeof thumb.bases === "object" && thumb.bases !== null && Object.values(thumb.bases).every(validLevel)));
// A card is the chart's plot, small: every pane in its band (with the chart's scale margins), every layer shown in
// it as its type (see previewSvg), over the view the chart was left at (else its range), a point per few pixels. Drawn
// from the chart's own data, which is then at hand when it opens.

const PREVIEWS = "bitview-studio:previews";
/**
 * A layer drawn small, as its type draws on the chart: `k` its kind (a line, an area, a baseline, bars, dots, one of
 * its lines across, or bubbles of one size, `w` across), `c` its color (`c2` under its base), `d` its line (or its
 * bars, or bubbles, above the base, `d2` those below), `y0` a baseline's base, `top`/`bottom` its pane's extent (`up`: an area filled above; `band`: a stacked one's band, its path), `dash`
 * its line style, `op` its opacity, `end` its latest point, `f` an area's fill (its top's opacity and its foot's: see fillOf),
 * `o` outlined (see OUTLINED): its layer's place, from 1 (its marks drawn together, see previewSvg).
 * @typedef {{ k: string, c: string, c2?: string, d: string, d2?: string, y0?: number, top?: number, bottom?: number, up?: boolean, band?: string, f?: number[], dash?: string, op?: number, end?: number[], img?: string, o?: number, w?: number, z?: [number, string][] }} PlotMark
 * Quick Look's (a big one's) too: its dates along its foot (`ticks`, where), each pane side's highest and lowest
 * value (`scales`, where). A preset's: which preset it is, by its place in their tree (`of`, see templateDoc).
 * @typedef {{ seps: number[], marks: PlotMark[], end?: number, ticks?: { x: number, text: string }[], scales?: { y: number, text: string, side: string }[], of?: string }} Preview
 */
/** Each kind, with what it can't be drawn without. */
const PLOT_NEEDS = { line: [], dots: [], bars: [], across: [], band: [], bubble: ["w", "top", "bottom"], area: ["top", "bottom"], baseline: ["y0", "top", "bottom"], heat: ["img", "top", "bottom"], profile: ["f", "top", "bottom"] };
const isPath = (d) => typeof d === "string" && /^[MLHVZh\d. -]*$/.test(d);
const validMark = (mark) => Object.hasOwn(PLOT_NEEDS, mark?.k) && PLOT_NEEDS[mark.k].every((field) => mark[field] !== undefined) && isColor(mark.c) && (mark.c2 === undefined || isColor(mark.c2)) && isPath(mark.d)
  && (mark.d2 === undefined || isPath(mark.d2)) && [mark.y0, mark.top, mark.bottom, mark.op, mark.w].every((n) => n === undefined || Number.isFinite(n))
  && (mark.up === undefined || typeof mark.up === "boolean") && (mark.f === undefined || (Array.isArray(mark.f) && mark.f.every(Number.isFinite))) && (mark.dash === undefined || (typeof mark.dash === "string" && /^[\d. ]+$/.test(mark.dash)))
  && (mark.end === undefined || (Array.isArray(mark.end) && mark.end.length === 2 && mark.end.every(Number.isFinite)))
  && (mark.img === undefined || (typeof mark.img === "string" && /^data:image\/png;base64,[A-Za-z0-9+/=]+$/.test(mark.img)));
const validPreview = (item) => validDrawingKey(item?.key) && Array.isArray(item.seps) && item.seps.every(Number.isFinite) && Array.isArray(item.marks) && item.marks.every(validMark);
/** The most a shelf shows (its section's first), and a grid at once (more, a button away). */
const SHELF = 12, GALLERY_PAGE = 60;
/**
 * The presets' rows as the sidebar last drew them, open, and the sections open then (see presetParts): kept in this
 * browser, the first to go when it's full, as a picture's are.
 */
const SIDE_PRESETS = "bitview-studio:side-presets";
/** What's picked from ⌘K's search, how often and when: what's used most, lately, comes first. */
const USED = "bitview-studio:used";
/** @type {Map<string, Thumb & { key: string }>} */
const thumbs = new Map(Object.entries(storage.get(THUMBS, {})).filter(([, thumb]) => validThumb(thumb)));
const saveThumbsSoon = writeSoon(() => grid.saveThumbs());
/** Brings every chart's mini chart up to date, one chart at a time, in the background. */
const drawThumbs = serial(async () => {
  // (They're the grid's: drawn as it comes.)
  if (!grid) return void withRest(drawThumbs);
  for (const id of [...thumbs.keys()]) if (grid.orphan(id)) thumbs.delete(id);
  let drew = false;
  for (const doc of [...library.charts, ...previewCharts, ...(chart?.preset !== undefined ? [chart] : [])]) drew = (await grid.freshThumb(doc)) || drew;
  // Stored a moment after (passes follow each edit: one write for many).
  if (drew) saveThumbsSoon();
});
// Drawings another tab made.
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
// ── The page's tiles: each chart drawn as it shows; a chart grows out of its card ──────────────────
/** @type {Map<string, Preview & { key: string }>} */
const previews = new Map(Object.entries(storage.get(PREVIEWS, {})).filter(([, item]) => validPreview(item)));
const savePreviewsSoon = writeSoon(() => grid.savePreviews());
/**
 * Rows `n` a ×10 in price, each the same share of its price: where a price falls among them (`at`, the row it's in
 * and how far into it), the row it's in (`cell`), a row's floor price (`price`), and the row a floor price is
 * (`floor`: as the URPD writes it, rounded). @param {number} n
 */
const logRows = (n) => ({ at: (/** @type {number} */ price) => n * Math.log10(price * 100), cell: (/** @type {number} */ price) => Math.floor(n * Math.log10(price * 100)), price: (/** @type {number} */ row) => 10 ** (row / n) / 100, floor: (/** @type {number} */ price) => Math.round(n * Math.log10(price * 100)) });
/**
 * The density a picture's heatmaps are sampled at: 50 rows a ×10 (a tile's few hundred pixels tall); their days, a
 * tile's (32: about 10 pixels a day) and Quick Look's (96).
 */
const HEAT_PICTURE_ROWS = 50, HEAT_PICTURE_AGG = `log${HEAT_PICTURE_ROWS}`, HEAT_PICTURE_GRID = logRows(HEAT_PICTURE_ROWS), HEAT_TILE_COLUMNS = 32, HEAT_QUICK_COLUMNS = 96;
/**
 * The page, where it is (see homeAt): Home, its question, its search (⌘K's) and a door to each group; your Library's
 * page or a folder's, its charts as a grid, then each folder in it as a shelf; the presets' or a section's, the same;
 * the previews' and Recent's, a grid. Its search searches where it is.
 */
function renderHome() {
  if (!home) return;
  // (One of your folders gone meanwhile (deleted in another tab): your charts' page.)
  if ("folder" in homeAt && homeAt.folder && !folderById(homeAt.folder)) {
    placeHome({ folder: "" });
    leftPlace = null;
    history.replaceState(homeState(), "");
  }
  // (A section of the presets gone (renamed in a newer Studio): the nearest one still there, else their page.)
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
  // (Searched: how many found, of both.)
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
// (Home's tests of being still, the grid's once it's in: Home waits for it.)
let yoursStill = () => true, galleryStill = yoursStill;
withRest(() => ((yoursStill = grid.stillness()), (galleryStill = grid.stillness()), grid.startGalleryRest()));
const drawYours = serial(async () => {
  for (const id of grid.yourQueue) {
    grid.yourQueue.delete(id);
    const doc = chartById(id) ?? [...grid.presetTiles.keys()].find((tile) => tile.id === id);
    if (!home || !doc?.layers.length) continue;
    // Drawn already (maybe from ⌘K), it's shown where it's still waited for (a preset's, from weekly data, as the
    // gallery's). A failed fetch isn't drawn (nor kept): it says so, and it's asked for again as the server answers
    // (see paint, retryPlots). Home still, each waiting tile's drawing starts at once (its fetch with it); else one
    // at a time, so a scroll doesn't ask for the tiles it passes.
    const drawn = grid.drawPreview(doc, grid.presetTiles.has(doc) ? grid.previewIndex(doc) : doc.index);
    if (!yoursStill()) await drawn;
  }
});
/**
 * The tile last pressed on the page: of the tiles showing a chart (a preset on two shelves, the recommended's and its
 * section's), the one it grows out of and shrinks back into. Every tile says the chart it opens (`data-chart`): one
 * of yours, a preset's.
 * @type {HTMLElement | null}
 */
let pressedTile = null;
$("home").addEventListener("click", (event) => (pressedTile = closest(event, ".tile[data-chart]") ?? pressedTile), true);

// ── The presets, as a gallery ─────────────────────────────────────────────
let homeAt = asPlace(history.state?.at ?? prefs.at);
/** The page somewhere else: kept (with the preferences: a reload, from a chart, comes back there). @param {Place} place */
const placeHome = (place) => {
  homeAt = place;
  // (Another page shown: what was in reach on the one left isn't, see inReach.)
  reachAt = -1;
  prefs.at = place;
  savePrefs();
};
/** The page's step of the history as it is. */
/**
 * The page's step of the history: where it is, and its key (a new one, else this step's own): the step shown from
 * then (see leftPlaces).
 */
const homeState = (/** @type {string | undefined} */ key = history.state?.home ? history.state.key : undefined) => ({ home: true, at: homeAt, key: (shownKey = key ?? uid()) });
/** The key of the step shown (a page's, see homeState). */
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
  // (Gone with the page it was on, the focus starts the new one: its name.)
  if (fromPage && !$("home").contains(document.activeElement)) $("home-title").focus({ preventScroll: true });
}
let galleryShown = GALLERY_PAGE;
$("home").addEventListener("scroll", (event) => /** @type {HTMLElement} */ (event.target).matches?.(".shelf-row") && grid.shelfSteps(/** @type {HTMLElement} */ (event.target)), { capture: true, passive: true });
// (A keyboard's focus come into it (Tab, the arrows, a menu's way back to its tile), for the keys to reach each; not
// one put there by a click's or a tap's way back from its chart: that wrote its row whole, 15 ms on a desktop.)
$("home").addEventListener("focusin", (event) => ((row) => row && /** @type {HTMLElement} */ (event.target).matches(":focus-visible") && grid.fillRow(row))(closest(event, ".shelf-row, .grid-tiles")));
// Every shelf measured again as the page's size changes (the window's, the sidebar's, a phone turned), shown at
// another size than it was left at too; not as it's simply shown again: hidden, its shelves stayed as they were.
{
  let size = "";
  new ResizeObserver(([entry]) => {
    const { inlineSize, blockSize } = entry.contentBoxSize[0], now = `${inlineSize} ${blockSize}`;
    // (Hidden, none: its size as it was left kept.)
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
// (Ahead: the tiles a screen below the view, drawn while the grid's at rest (`galleryResting`, grid.js), after
// those in view, one at a time: a scroll finds them drawn.)
const galleryAhead = /** @type {Set<Element>} */ (new Set()), homeScroller = /** @type {HTMLElement} */ ($("home").querySelector(".home-scroll"));
// (Its root the grid's own scroller: from the window's, what's past the scroller's edge is cut off, margin or not. Its
// margin a screen, the scroller's height in pixels (a percentage there is of its width), made again as it's resized.)
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
    // (A tile written of it as it was, kept since as opened or edited (in another tab): written again, as it is.)
    for (const tile of all$(`.tile[data-preset="${i}"]:has(> [data-open-preset])`, $("gallery"))) if (tile.querySelector("[data-plot]")?.getAttribute("data-plot") !== doc.id) grid.writeTile(tile, i);
    if (have?.key === grid.previewKey(doc)) grid.paint(doc.id, have, true);
    else {
      // (In view and Home still: started without waiting on it, so every tile in view is asked for at once, each
      // drawn as its data comes. Moving, or a screen ahead, one at a time: a scroll doesn't ask for the tiles it
      // passes, nor those it may never reach. Each given up if the grid's drawn anew without it, see giveUpUnframed.)
      const drawn = grid.drawPreview(doc, grid.previewIndex(doc));
      if (![...grid.gallerySeen].some((frame) => grid.presetOf(frame) === i) || !galleryStill()) await grid.galleryWait(drawn);
    }
  }
});
$("home").addEventListener("click", (event) => {
  const to = closest(event, "[data-place]"), step = closest(event, "[data-shelf-step]"), preset = closest(event, "[data-open-preset]");
  if (to) goTo(asPlace(JSON.parse(/** @type {string} */ (to.dataset.place))));
  else if (closest(event, "[data-gallery-more]")) {
    // (Where it was, the focus on the first one more.)
    const place = galleryPlace(), from = galleryShown, list = /** @type {HTMLElement} */ (closest(event, "[data-gallery-more]")).previousElementSibling?.getAttribute("data-list");
    galleryShown += GALLERY_PAGE;
    renderHome();
    placeGallery(place);
    const tile = $("gallery").querySelector(`[data-list="${list}"]`)?.children[from];
    /** @type {HTMLElement | null | undefined} */ ((tile?.matches(".tile-slot") ? grid.fillSlot(/** @type {HTMLElement} */ (tile)) : tile)?.querySelector(".tile-open"))?.focus({ preventScroll: true });
  }
  else if (closest(event, "[data-gallery-retry]")) retryPresets();
  else if (closest(event, "[data-search-all]")) {
    // (The same words, over every preset: the field as it was, its focus kept by keys, not a keyboard by touch.)
    const words = /** @type {HTMLInputElement} */ ($("home-filter")).value;
    goTo({ path: [] });
    /** @type {HTMLInputElement} */ ($("home-filter")).value = words;
    renderHome();
    if (!byTouch) $("home-filter").focus({ preventScroll: true });
  }
  else if (step && step.getAttribute("aria-disabled") !== "true") grid.stepShelf(/** @type {HTMLElement} */ (step.closest(".shelf")?.querySelector(".shelf-row")), Number(step.dataset.shelfStep));
  else if (preset) pickPreset(Number(preset.dataset.openPreset));
});

// ── Picking charts in the grid, and what the bar does to them ──
/** A tile's right-click: the bar's actions at the pointer, on the picks if it's one of them, else on it alone. */
function tileMenu(id, point = /** @type {{ x: number, y: number } | undefined} */ (undefined), anchor = /** @type {HTMLElement} */ (grid.tileOpen(id))) {
  // (On a picked card, it acts on every one picked; on another, on it alone: the picks stay as they are, it outlined.)
  const target = grid.picked.size && !grid.picked.has(id) ? grid.cardOf(id) : null;
  queueMicrotask(() => target && !$("menu").hidden && menus.outlineMenuTarget(target));
  // (Picked, on a phone, its sheet named for all it acts on: the cards out of sight under it (one alone: named as any is).)
  const title = phone.matches && grid.picked.has(id) ? plural(grid.picked.size, "chart") : "";
  menus.menuOf(anchor, grid.picked.has(id) ? grid.gridActions() : command.chartActions(id, "tile"), point ?? null, "", { title });
}
$("home-bar").addEventListener("click", (event) => {
  const button = closest(event, "[data-batch]");
  if (button?.getAttribute("aria-disabled") === "true") notice(/** @type {string} */ (button.dataset.tip));
  else if (button) grid.batch(/** @type {string} */ (button.dataset.batch), button);
});
// A click on a tile picks it with ⌘ or ⇧, on its tick, or while others are picked; else (or by a key) it opens.
$("home-grid").addEventListener("click", (event) => {
  const more = closest(event, "[data-tile-menu]");
  if (more) return tileMenu(/** @type {string} */ (more.dataset.tileMenu), undefined, more);
  const tile = closest(event, "[data-home-card]:not(.preview)"), check = closest(event, ".tile-check");
  if (!tile || closest(event, ".tile-rename")) return;
  if (!(check || event.metaKey || event.ctrlKey || event.shiftKey || (grid.picked.size && event.detail > 0))) return;
  event.stopPropagation();
  grid.pickChart(/** @type {string} */ (tile.dataset.homeCard), event.shiftKey);
});
// The filter: as typed; esc empties it, then leaves it.
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
    // Down, or ↵, to the first chart through (yours first, then the presets').
    const tiles = all$(".tile:not(.waiting) > .tile-open", $("home")).filter((tile) => tile.checkVisibility()), first = tiles[0];
    if (!first) return;
    event.preventDefault();
    if (event.key === "Enter" && tiles.length === 1) first.click();
    else first.focus();
  }
});
/**
 * The Charts page, where `homeAt` is; the open one shrinks into its card (one of yours that's never had anything in
 * it, left so, goes: see emptyYours).
 * `push`: a step of the browser's history (not when going back to it).
 */
/** The open chart, deleted on a phone: the Charts page comes back without it, another under it (see deleteChart). */
let deletedOpen = "";
function showHome(push = true, /** @type {{ top: number, rows: Record<string, number> } | null} */ place = null) {
  /** The open chart's card drawn again once it's in (see below). @type {(() => void) | null} */
  let redrawCard = null;
  gestures.stopCarrying();
  delete document.body.dataset.sheet;
  if (home) return;
  navigations++;
  // (A focus on the chart or a menu over it, before leaving lets it go: see below.)
  const focusLeaves = grid.onChart();
  // (A toast of the chart's edit stays with it: its Undo isn't the page's.)
  if (toastOf === chart) hideToast();
  // (Full screen is the chart's: the grid, as ever.)
  presetsEdit.setFull(false);
  leaveField();
  saveView();
  // The open chart as it is now: its card shows it before it shrinks into it. (Its data failed: none, its card's
  // own fetch tries again, see paint.)
  if (settled() && chart.layers.length && resolved && !resolved.failed.length) {
    // (A heatmap's days aren't at hand at a card's density: its card kept till it's drawn with them, see drawPreview.
    // One drawn here as it is now already, of the same data (left as it was), kept; one as it is, its data since
    // moved on (a live price, a block: its last points), shown as it shrinks and drawn again in an idle moment once
    // it's in (not in the motion's frames).)
    const key = chart.layers.some((layer) => heatLike(layer) && !layer.hidden) ? "" : grid.previewKey(chart);
    const drawCard = (/** @type {Chart} */ doc, /** @type {Resolved} */ data, /** @type {number} */ revision) => {
      const drawn = { key, ...grid.previewOf(doc, data) };
      previews.set(doc.id, drawn), grid.savePreviews(), cardsDrawn.set(doc.id, revision);
      return drawn;
    };
    if (!key || previews.get(chart.id)?.key !== key) drawCard(chart, resolved, dataRevision);
    else if (cardsDrawn.get(chart.id) !== dataRevision) {
      const doc = chart, data = resolved, revision = dataRevision, kept = previews.get(doc.id);
      // (Only while it's still the chart under the grid: another opened since, what a picture reads of the open one
      // (its bars' spacing, its room) is that one's; this card kept as it is till the next time.)
      redrawCard = () => home && chart === doc && previews.get(doc.id) === kept && grid.previewKey(doc) === key && grid.paint(doc.id, drawCard(doc, data, revision), true);
    }
  }
  // (A draft taken back only when Cancel (or esc) is the way there; one of yours never filled, whichever way; one
  // deleted, gone (nothing to shrink into).)
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
  /** Where the focus goes once the page is there (see below), and where it was as the way back began. */
  let focusTo = /** @type {HTMLElement | null} */ (null), focusWas = /** @type {Element | null} */ (null);
  void grid.morph(false, () => {
    document.body.toggleAttribute("data-home", true);
    // (What's in hand let go of once the page is over the chart: the chart not drawn again for it, see
    // repaintDrawings.)
    pen.putDown();
    clearMeasure();
    // (A toast up stays, where it shows on this page: over its foot, not the chart's.)
    if ($("toast").classList.contains("visible")) placeToast();
    refresh();
    // (Back to the page from a chart opened from it: where it was, else where the chart's card is.)
    const card = grid.cardOf(chart.id);
    if (place) placeGallery(place);
    else card?.scrollIntoView({ block: "nearest" });
    // (Its card in sight where the page came back to: the chart shrinks into it, the focus on it; else neither,
    // the page as it was left, a focus gone with the chart (or a menu over it) starting it: its name.)
    const box = card?.getBoundingClientRect(), view = homeScroll().getBoundingClientRect(), seen = Boolean(box && box.bottom > view.top && box.top < view.bottom);
    focusTo = seen ? grid.tileOpen(chart.id) : focusLeaves && (document.activeElement === document.body || grid.onChart()) ? $("home-title") : null;
    focusWas = document.activeElement;
    return draft || !seen ? null : card;
  }).then(() => {
    if (!home) return;
    if (redrawCard) whenIdle(redrawCard);
    // (In reach again once it's there, the focus on it then: inert is style, inherited, and lifting it has the whole
    // page styled again, not in the motion's frames. Moved meanwhile, the focus is where it was put.)
    $("home").inert = false;
    // (Its tile gone meanwhile (stepped on, its page written anew): the page's name, as where it's not in sight.)
    if (document.activeElement === focusWas || document.activeElement === document.body) (focusTo?.isConnected ? focusTo : focusTo && $("home-title"))?.focus({ preventScroll: true });
    // The new chart, empty, taken back once it's faded out over the page, the one before it under the grid then
    // (else that one would show, fading); none opened meanwhile. (An empty preview left simply goes.)
    if (draft && chart === draft) openChart(previousChart() ?? newDraft());
  });
}
/**
 * Leaves the grid: `show` opens the chart to go to (or leaves the one under the grid), and it grows out of
 * `before`, its card. Back returns to the grid. Out of
 * the grid, `show` simply runs.
 */
function leaveHome(show = () => {}, /** @type {HTMLElement | null} */ before = null, push = true) {
  if (!home) return (before?.classList.remove("opening"), show(), Promise.resolve());
  navigations++;
  home = false;
  prefs.home = false;
  savePrefs();
  grid.picked.clear();
  // (Where the page was, kept in its step: Back comes back there. Read before anything changes on it: after, it'd
  // be laid out anew just to read it.)
  leftPlace = galleryPlace();
  show();
  // (A step from before steps had keys: given one.)
  if (push && history.state?.home) (history.replaceState({ ...history.state, ...homeState(), place: leftPlace }, ""), grid.leftPlaces.set(history.state.key, leftPlace));
  if (push) history.pushState({ chart: chart.id, fromHome: true, place: leftPlace }, "");
  const focused = $("home").contains(document.activeElement);
  // (Out of reach once the chart's over it: see .home, and showHome.)
  return grid.morph(true, () => {
    // (Pressed as it was tapped, see go: as it grows, as it is.)
    before?.classList.remove("opening");
    document.body.removeAttribute("data-home");
    if ($("toast").classList.contains("visible")) placeToast();
    // (Never drawn (a start onto the grid): drawn now, as it opens. A block came while it was away: its data comes
    // again, as it would have. Asked first: what's around it, shown as that draw lands, see redrawChart.)
    if (undrawn) (blockAtHome = false), void draw(true);
    else if (blockAtHome) (blockAtHome = false), void draw(false, true);
    refresh();
    const row = /** @type {HTMLElement | null} */ ($("chart-list").querySelector(`[data-open-chart="${chart.id}"]`));
    if (focused && row?.checkVisibility()) row.focus({ preventScroll: true });
    // (Drawn as it grows, live: its data is usually at hand, from its tile.)
    return before;
  }).then(() => void (home || ($("home").inert = true)));
}
/** Where the page was when a chart was opened from it (its scroll, its shelves'): Back comes back there. @type {{ top: number, rows: Record<string, number>, shown: number } | null} */
let leftPlace = ((place) => (Number.isFinite(place?.top) && place.rows && typeof place.rows === "object" ? place : null))(history.state?.chart ? history.state.place : null);
/** A new chart being cancelled on the way back to the grid (it's taken back once there). */
let cancelling = "";
/** Whether this browser keeps a closing dialog or popover drawn while it leaves (`overlay`; Safari doesn't). */
const keepsLeaving = CSS.supports("overlay: auto");
// Back (Android's) closes a phone's sheet, the layers' or a menu's, or a dialog: a close watcher for each open one
// (where the browser has them), the last opened closed first, ⌘K among them. Closing the layers'
// sheet, its list next time (a layer's settings were in it).
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
  // (On a layer's settings (or several layers', together), Back is the list's way back, as ‹ is; on the list, the
  // sheet goes.)
  const closeSheet = () => {
    if ((selected || inspector.editingMany()) && document.body.dataset.sheet === "panel") {
      deselect();
      watch("sheet", true, closeSheet);
    } else delete document.body.dataset.sheet;
  };
  const sync = () => {
    if (panelOpen && document.body.dataset.sheet !== "panel" && (selected || inspector.editingMany())) deselect();
    panelOpen = document.body.dataset.sheet === "panel";
    // (The tab bar's Layers says its sheet's open, as it shows: lit.)
    $("mobile-layers").setAttribute("aria-expanded", String(panelOpen));
    watch("sheet", phone.matches && Boolean(document.body.dataset.sheet), closeSheet);
    watch("menu", phone.matches && !$("menu").hidden && $("menu").classList.contains("as-sheet"), menus.menuGoBack);
  };
  new MutationObserver(sync).observe(document.body, { attributes: true, attributeFilter: ["data-sheet"] });
  new MutationObserver(sync).observe($("menu"), { attributes: true, attributeFilter: ["hidden", "class"] });
  // (A dialog, as a modal's was: put away as esc puts it away.)
  for (const dialog of all$("dialog")) dialog.addEventListener("toggle", () => watch(dialog.id, phone.matches && isOpen(dialog), () => dialog.dispatchEvent(new Event("cancel", { cancelable: true })) && dialog.hidePopover()));
}
addEventListener("popstate", (event) => {
  stillStep(/** @type {PopStateEvent & { hasUAVisualTransition?: boolean }} */ (event).hasUAVisualTransition === true);
  // (The page left, where it was, by its step: its own can't be written now. Still coming back (a second step
  // taken at once), it isn't shown yet: nothing to keep.)
  if (home && shownKey && document.body.hasAttribute("data-home")) grid.leftPlaces.set(shownKey, galleryPlace());
  shownKey = event.state?.key;
  // (A step whose address holds a link, one put in or a picture's: its hashchange reads it; one that couldn't be
  // read, as it is.)
  if (location.hash !== brokenLink && isLinkHash(location.hash)) return;
  grid.closeDialogs();
  if (event.state?.home) {
    const to = asPlace(event.state.at), place = grid.leftPlaces.get(event.state.key) ?? event.state.place ?? null;
    // (Back to where it was (from a chart), as much of it shown; elsewhere, as much as it showed when left, else a
    // page, nothing searched; its scroll as it was left, else its top.)
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
    // (Gone with the page it was on, the focus starts the one come back to: its name.)
    if (fromPage && !$("home").contains(document.activeElement)) $("home-title").focus({ preventScroll: true });
    if (place) placeGallery(place);
    else homeScroll().scrollTop = 0;
    return;
  }
  // (A preset's step, its chart let go of since (left as it was): made again. Another's gone (an empty one let go
  // of): nowhere, the step where it is now.)
  // (A preset's address, a step back to it or its fields changed by hand: its preset as the address says, as a
  // visit to it reads it (see presetByAddress), whatever it's shown since; as shown already, nothing to do.)
  const route = ROUTED ? routeOf(location.pathname) : null, routed = route && "preset" in route ? route.preset : -1;
  if (routed >= 0 && !home && presetIndex(chart) === routed && `${location.pathname}${location.hash}` === `${BASE}${presetLink(chart)}`) return void showAddress();
  const preset = typeof event.state?.chart === "string" ? presetById(event.state.chart) : -1;
  const doc = routed >= 0 ? presetByAddress(routed, route && "window" in route ? route.window : undefined) : chartById(event.state?.chart) ?? (preset >= 0 ? presetChart(preset) : undefined);
  if (!doc) {
    // (A preview closed since: said, and how it comes back.)
    const closed = closedPreviews.get(event.state?.chart);
    if (closed) notice(`“${closed}” was closed${libraryEdits.past.at(-1)?.text === `Closed the preview “${closed}”` ? `. Press ${keyText("⌘Z")} on Home to bring it back` : ""}`);
    return void history.replaceState(home ? homeState() : { chart: chart.id }, "");
  }
  // (Its chunks at hand are for the view it's left: those for this one fetched anew; the open one drawn again.)
  const again = routed >= 0 && doc === chart;
  if (routed >= 0) forgetSpans(doc.id);
  // (Its step names what it opens (a preset edited since: its preview), so opening it pushes no step: Forward kept.)
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
// The layers' sheet, by its handle or its head, or its list at its top (a phone's; upright, it opens at half).
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
// In the grid, the arrows walk the cards as they're laid out.
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
    // (Charts alone picked: a shelf's last tile, to them all, isn't one.)
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
// (The presets' gallery's cards too: the arrows, Home and End, as the grid's.)
$("gallery").addEventListener("keydown", (event) => {
  if (event.metaKey || event.ctrlKey || event.altKey || event.shiftKey) return;
  grid.walkCards($("gallery"), event);
});
/**
 * The sidebar's charts, in the order you give them (new ones on top), ⌘1–⌘9 opening the first nine of its topmost open
 * group's (see numbered): written
 * again only where it changed (an edit that changes nothing in it, a formula typed, leaves it as it is).
 */
function renderCharts() {
  // (The sidebar's list is the grid's: written as it comes, see later.js.)
  if (!grid) return void withRest(renderCharts);
  const current = home ? null : chart;
  // (On a phone the sidebar never shows: its list left unwritten, owed, till the layout turns wider or something
  // reads it: see sideList.)
  if (phone.matches) listOwed = true;
  else writeSideList(current);
  grid.renderSideNav();
  // (A new chart, empty, has nothing to keep, nor anything ⋯ does: it's yours once something's in it; its Cancel takes it back.)
  $("preview-note").hidden = !isPreview(chart) || !chart.layers.length;
  // (A preset's: what can be done with it beside it, Reset once it's edited.)
  $("preset-note").hidden = chart.preset === undefined || presetIndex(chart) < 0;
  renderChartPath();
  $("preset-reset").hidden = $("preset-note").hidden || !presetEdited(chart);
  $("chart-more").hidden = !home && isPreview(chart) && !chart.layers.length;
  // The phone bar's Add, in the grid, makes a chart (there's none open to add to).
  /** @type {HTMLElement} */ ($("mobile-add").querySelector(".label")).textContent = home ? "New chart" : "Add";
  // (Layers, the open chart's: none in the grid, nor on an empty chart.)
  $("mobile-layers").disabled = home || !chart.layers.length;
  if (home) $("mobile-charts").setAttribute("aria-current", "page");
  else $("mobile-charts").removeAttribute("aria-current");
  if (home) $("mobile-add").removeAttribute("aria-haspopup");
  else $("mobile-add").setAttribute("aria-haspopup", "dialog");
  grid.keepNaming();
  renderHome();
  // (The empty chart's cards, where they show, one of them edited since: as it is now.)
  if (presets && !$("empty").hidden && grid.recommendedKey() !== grid.recommendedDrawn) grid.renderRecommended();
}
/** The sidebar's list written (see renderCharts): its rows, ⌘1–⌘9's keys, the presets' marks, the open one lit. @param {Chart | null} current */
function writeSideList(current) {
  listOwed = false;
  /** A preview's row, its × (but the open one with nothing to go back to). */
  const previewRow = (/** @type {Chart} */ doc) => `<div class="chart-row preview" data-preview-row="${doc.id}">${`<button type="button" class="nav-item chart-item${touched.has(doc) ? " edited" : ""}" data-open-chart="${doc.id}" data-tip="${escapeHtml(titleOf(doc))}: a preview, not in your Library yet (Save${doc === chart ? " or ⌘S" : ""} to keep it${touched.has(doc) ? " with your edits" : ""})">${grid.chartMark(doc)}<span class="label">${escapeHtml(titleOf(doc))}</span></button>${doc !== chart || previousChart() ? `<button type="button" class="cap small chart-more" data-action="close-preview" data-preview="${doc.id}" aria-label="Close the preview ${escapeHtml(titleOf(doc))}" data-tip="Close the preview">${ICONS.close}</button>` : ""}`}</div>`;
  // (A draft, the empty chart in place of none (the open one deleted), isn't listed: it's the page to start one, not
  // a chart, till something's in it.)
  // (On the rail, folders never open: each a door to what's in it, see railMenu.)
  const listed = listedPreviews(), counts = grid.chartCounts(), rail = menus.onRail(), recents = recentCharts();
  // (Recent reordered (a chart opened): the rows slid under the pointer, none of them pointed at: its highlight put
  // away till the pointer moves, see sidePoint.)
  const recentOrder = recents.map((item) => item.id).join();
  if (recentDrawn && recentOrder !== recentDrawn) (sideHeld = true), sideLight(null);
  recentDrawn = recentOrder;
  const row = (/** @type {Chart} */ doc, depth = 0) => grid.leafRow({ kind: "chart", id: doc.id, depth, mark: grid.chartMark(doc), label: titleOf(doc), labelCls: doc.title ? "" : "derived", name: titleOf(doc) });
  /** @type {[string, string][]} */
  const parts = [
    // Home first (Studio's start), a rule under it, scrolled with the rest: only the brand, the search and the
    // foot stay put.
    /** @type {[string, string]} */ (["charts", `<div class="chart-row action-row" data-action-row="charts" style="--depth: 0"><button id="charts-nav" class="nav-item side-action" type="button" data-action="charts" data-tip="Home"${home && "top" in homeAt ? ' aria-current="page"' : ""}><span data-icon="home">${ICONS.home}</span><span class="label">Home</span></button></div>`]),
    /** @type {[string, string]} */ (["rule", `<div class="side-rule" aria-hidden="true"></div>`]),
    // The charts last opened first (the open one at their top, lit), under their head; then the previews under
    // theirs, then your charts under theirs (told apart; none, no heads), a new chart first among yours (where it'll
    // be once something's in it).
    ...(recents.length ? [grid.sideHead("recent", recents.length, false), ...(grid.groupShut("recent") ? [] : recents.map((item) => /** @type {[string, string]} */ ([`recent:${item.id}`, `<div class="chart-row" data-recent-row="${item.id}" style="--depth: 0"><button type="button" class="nav-item chart-item" data-open-recent="${item.id}" data-tip="${escapeHtml(titleOf(item.doc))}">${recentMark(item)}<span class="label">${escapeHtml(titleOf(item.doc))}</span></button></div>`])))] : []),
    ...(listed.length ? [grid.sideHead("previews", listed.length, Boolean(current && listed.includes(current))), ...(grid.groupShut("previews") ? [] : listed.map((doc) => /** @type {[string, string]} */ ([`preview:${doc.id}`, previewRow(doc)])))] : []),
    // (Yours under their name, always: closed, it's the way to open them again.)
    grid.sideHead("library", library.charts.length, Boolean(current && !isPreview(current) && current.preset === undefined)),
    ...(grid.groupShut("library") ? [] : [
    // (New chart first: what it makes is yours, here; folded away with them.)
    /** @type {[string, string]} */ (["new", `<div class="chart-row action-row" data-action-row="new" style="--depth: 0"><button type="button" id="new-chart" class="nav-item chart-item side-action" data-action="new-chart" data-tip="New chart" data-kbd="N"><span data-icon="plus">${ICONS.plus}</span><span class="label">New chart</span><kbd>N</kbd></button></div>`]),
    // (Your charts as a tree: each folder's under it, as deep as they go; a closed one's out of sight.)
    ...grid.sideTree().map(({ item, depth }) => {
      if ("layers" in item) return /** @type {[string, string]} */ ([item.id, row(item, depth)]);
      return /** @type {[string, string]} */ ([`folder:${item.id}`, grid.folderRow({ kind: "folder", id: item.id, depth, closed: item.closed || rail, holds: Boolean(current?.folder && grid.isIn(current.folder, item.id)), rail, mark: folderMark(item), name: item.name, count: counts.get(item.id) ?? 0, place: { folder: item.id } })]);
    })]),
    ...grid.presetParts(),
  ];
  // (What it lists changed (a chart opened, saved, closed or deleted, a group come): its rows glide to their places,
  // none jumping there, unless what changed it moves them itself (see movingSide). Where they were, read only then.)
  const list = $("chart-list"), drawn = listWritten.get(list), keys = (/** @type {[string, string][]} */ items) => items.map(([key]) => key).join("\n");
  const was = drawn && !grid.sideMoved && !reduced() && keys(drawn) !== keys(parts) ? listPlaces(list) : null;
  // (Its scroll edges measured on the next frame, once, as the page lays out anyway: not here, mid-change.)
  if (writeList(list, parts)) fitSoon(list, true);
  if (was) glideList(list, was);
  // (⌘1–⌘9 shown on the rows they open, those alone: their key, and their tip's.)
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
  // (The presets' rows in sight: their marks, drawn as they come; those drawn already, nothing to watch; those
  // gone, nothing to draw.)
  // (Not before the presets are read: their rows as kept till then, see presetParts.)
  presetRowsSeen.disconnect();
  grid.presetMarkQueue.clear();
  if (presets) for (const row of all$("[data-preset-row]", $("chart-list"))) if (!presetMarks.has(grid.presetMarkKey(Number(row.dataset.presetRow)))) presetRowsSeen.observe(row);
  // (Recent's presets, their marks asked for as written.)
  if (grid.presetMarksAsked.size) drawPresetMarks();
  // The open one marked in place: a switch writes no row again (the focus on one stays where it is). (A preset's,
  // by which it is.)
  const openedPreset = current?.preset !== undefined ? presetIndex(current) : -1;
  for (const button of all$("[data-open-chart], [data-open-preset], [data-open-recent]", $("chart-list"))) {
    const on = button.dataset.openPreset !== undefined ? Number(button.dataset.openPreset) === openedPreset : (button.dataset.openChart ?? button.dataset.openRecent) === current?.id;
    // (Each, its row's mark and its own, set apart: a row written again in place keeps its button as it is.)
    button.closest(".chart-row")?.classList.toggle("active", on);
    if (on === button.hasAttribute("aria-current")) continue;
    if (on) button.setAttribute("aria-current", "page");
    else button.removeAttribute("aria-current");
  }
  // (A preset picked: once its own row's drawn, its rows (Recent's too) shown.)
  if (presetToReveal >= 0 && presetToReveal === openedPreset && $("chart-list").querySelector(`[data-open-preset="${openedPreset}"]`)) (presetToReveal = -1), presetsEdit.revealNearest(openedPreset, /** @type {Chart} */ (current).id);
  // (Its Tab stop, the open one marked first.)
  grid.roveList();
}
/** The presets' marks, as drawn (made as their rows come into sight; those of earlier visits, kept), by what each is of. @type {Map<string, Pick<Thumb, "lines" | "washes" | "dots">>} */
const presetMarks = new Map(((kept) => (kept && typeof kept === "object" ? Object.entries(kept).filter(([key, mark]) => validDrawingKey(key) && validChartMark(mark)) : []))(storage.get(PRESET_MARKS, {})));
const savePresetMarks = writeSoon(() => storage.set(PRESET_MARKS, Object.fromEntries([...presetMarks].slice(-grid.KEPT_PRESET_MARKS))));
/** ⌘K's preset rows in sight, their marks to draw; out of sight, or written again (see renderPalette), let go of. */
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
  // (One at a time, as the queue is now: a row scrolled away meanwhile, out of it.)
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
    // (Wherever it shows: its row, a menu of its section's.)
    for (const mark of all$(`[data-preset-mark="${i}"]`)) mark.outerHTML = grid.presetMark(i);
  }
});
let sidePresets = ((kept) => (typeof kept?.open === "string" && Array.isArray(kept.parts) && kept.parts.every((part) => Array.isArray(part) && part.length === 2 && part.every((text) => typeof text === "string")) ? /** @type {{ open: string, parts: [string, string][] }} */ (kept) : null))(storage.get(SIDE_PRESETS, null));
const saveSidePresets = writeSoon(() => storage.set(SIDE_PRESETS, sidePresets));
/** The presets' rows drawn, kept if they're new. @param {[string, string][]} parts */
function keepSidePresets(parts) {
  const open = JSON.stringify(prefs.presetsOpen);
  if (sidePresets?.open === open && sidePresets.parts.length === parts.length && sidePresets.parts.every(([key, html], i) => parts[i][0] === key && parts[i][1] === html)) return;
  sidePresets = { open, parts };
  saveSidePresets();
}
/** Recent's charts as the sidebar last listed them, in order (see renderCharts). */
let recentDrawn = "";
/**
 * The page in the theme now chosen (the device's, with System), if it changed: drawn again, crossfaded (the chart
 * paints in the frame the crossfade ends on: nothing's waited for, the page is still while it's taken).
 */
function applyTheme(/** @type {boolean} */ anyway = false) {
  if (!anyway && lightChosen() === light) return void (document.documentElement.dataset.theme = prefs.theme);
  const swap = () => {
    // (As chosen when it runs: two changes queued are one. Where colors were picked, on the other surface.)
    light = lightChosen();
    withRest(() => inspector.pickedAt.clear());
    document.documentElement.dataset.theme = prefs.theme;
    hexCache.clear();
    engine.theme = chartTheme();
    refresh();
    // (Its chart recolored inside the crossfade, not a frame after it: see buildSoon.)
    buildNow();
    if (home) renderHome();
    if (presets && !$("empty").hidden) grid.renderRecommended();
    // (Every chart's plot on screen in its tones: the tiles, the cards of charts to start from, ⌘K's preview, Quick
    // Look's.)
    for (const [id, drawn] of previews) grid.paint(id, drawn, true);
    if (quick) quickShow();
    if (isOpen($("palette"))) renderPalette();
    // (Words being typed on the chart: in their tone on this surface.)
    if (writing) heatmaps.dressField(writing.drawing);
  };
  if (document.startViewTransition && !reduced()) document.startViewTransition(swap);
  else swap();
}
deviceLight.addEventListener("change", () => applyTheme());
// (More contrast asked for, or no longer: the same theme's tones changed, everything drawn in them drawn again.)
matchMedia(MEDIA.moreContrast).addEventListener("change", () => applyTheme(true));
// Another tab's choices, taken here too as they're shown (the theme, the sidebar, the panel, the belt, the pen).
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
  // (Unlinked there: the chart here stays where it is, as its own.)
  if (took.includes("linkView")) (carrySpan(), renderLinkView(), !prefs.linkView && storeView(false));
  if (took.includes("presetsOpen") || took.includes("groupsOpen")) renderCharts();
});
withRest(() => {
  grid.renderTheme();
});
renderLinkView();
/** A CSS cubic-bezier's progress at a time (both 0 to 1), as the browser eases by it. @param {string} css */
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
/**
 * A folder deleted: what's in it kept (its charts, its folders), a level up where it was, with an Undo; the focus,
 * gone with its row, on what's in its place (the first of what was in it, else the one after it, before it, the
 * folder it was in).
 */
function deleteFolder(/** @type {string} */ id) {
  const at = library.folders.findIndex((folder) => folder.id === id), folder = library.folders[at];
  if (!folder) return;
  const members = grid.childrenOf(id), charts = library.charts.filter((doc) => grid.isIn(doc.folder, id)).length, siblings = grid.childrenOf(folder.parent), place = siblings.indexOf(folder);
  library.folders.splice(at, 1);
  // (Kept with the library, as a chart's: another tab's copy of it doesn't bring it back.)
  library.deleted[id] = Date.now();
  // (What was in it where it was, in its order; its page open, the one it was in.)
  for (const item of members) grid.setParent(item, folder.parent);
  // (Its page where the page is, or was left: the one it was in, from its top.)
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
    // (Back as a folder anew, its deletion standing: wherever that's been heard, this one isn't it.)
    // (What was in it, still where the delete put it; the folder that was around it gone since, wherever that took
    // them: the folder back with them there.)
    // (Its folder made again since under another id: that one.)
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
/** Deletes a chart, with an Undo; if it was the open one, its neighbor opens. @param {Chart} doc */
function deleteChart(doc) {
  const index = library.charts.indexOf(doc);
  if (index < 0) return;
  const wasOpen = doc === chart;
  // Its history kept as any chart's left is (see keepHistory: the last few charts', fifty steps each), there when it
  // comes back: undone, the chart can still be undone further. (Its redos let go of: the delete is a step done since.)
  if (wasOpen) edits.future = [];
  // (Where the focus goes, asked before what has it goes: the row after, else the one before, else New chart; from
  // the Charts page (its tile, its ⋯'s menu), that one's tile there.)
  const neighbor = library.charts[index + 1] ?? library.charts[index - 1], kept = () => Boolean(neighbor && library.charts.includes(neighbor));
  const onPage = home && [document.activeElement, menus.ownerNow(), performance.now() - menus.closedAt < 1000 ? menus.closedOwner : null].some((element) => element?.closest("#home"));
  // (There, the tile shown after it in its row, else the one before: a shelf shows some, a folder's elsewhere.)
  const tile = onPage ? grid.cardOf(doc.id) : null, beside = /** @type {HTMLElement | null | undefined} */ (tile?.nextElementSibling?.matches(".tile[data-home-card]") ? tile.nextElementSibling : tile?.previousElementSibling?.matches(".tile[data-home-card]") ? tile.previousElementSibling : null)?.dataset.homeCard;
  keepFocus(null, () => (beside ? grid.tileOpen(beside) : null), () => (kept() ? $("chart-list").querySelector(`[data-open-chart="${CSS.escape(/** @type {Chart} */ (neighbor).id)}"]`) : null), () => $("new-chart"), () => $("charts-nav"));
  discard(doc);
  // On a phone, the open one deleted: back to the Charts page, its list (as Back, where it was opened from there),
  // another chart under it (see showHome); elsewhere, the next one open in its place.
  const away = wasOpen && phone.matches && !home;
  if (away) {
    deletedOpen = doc.id;
    // (Else its step in the history the Charts page's, not one more: Back from there goes where it went before.)
    if (history.state?.fromHome === true && history.state.chart === doc.id) history.back();
    else (showHome(false, leftPlace), history.replaceState(homeState(), ""));
  }
  // The last chart gone (the open one), the preview there is stands in, else a blank one.
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
      // (Gone back to the Charts page for it: back into it.)
      else if (away && home) void leaveHome(reopen);
      else reopen();
      // (Again, only as it came back: edited since, here or in another tab, it's kept, said.)
      const same = asThen([doc]);
      return () => (same() ? ((now) => now && deleteChart(now))(library.charts.find((other) => other.id === doc.id)) : keptAsEdited([doc]));
    },
    undone: `“${titleOf(doc)}” is back`,
  };
  if (!away) return toast(text, action);
  // (Gone back to the Charts page: its step done now, in its place among the others; said once another chart's
  // under it.)
  const step = libraryStep(text, action.run, action.undone);
  void until(() => chart !== doc, 4000).then(() => toast(text, { ...action, step }));
}
// ── Presses on every list and control, lists reordered by dragging, the sidebar's list by keys ──
withRest(() => {
  gestures.startGestures();
  gestures.dragToReorder($("chart-list"), {
    item: "[data-chart-row], [data-folder-row]",
    // (A row's → and ⋯ pressed: theirs, not a drag.)
    refuse: (event) => Boolean(closest(event, ".chart-go, .chart-more")),
    idOf: grid.sideId,
    label: (id) => (id.startsWith("f:") ? ((folder) => (folder ? `${folderMark(folder)}<span class="name">${escapeHtml(folder.name)}</span>` : ""))(folderById(id.slice(2))) : grid.chartLabel(id.slice(2))),
    landing: (x, y, ids) => {
      const places = grid.placesFor(ids), under = /** @type {HTMLElement | null | undefined} */ (document.elementFromPoint(x, y)?.closest("[data-folder-row]"));
      const onto = under && places.includes(under) ? under : null;
      /** (At its row's level; after the last row of a folder, as far out as the pointer is, to the next row's level.) @param {Landing | null} at */
      const level = (at) => {
        if (!at || !("item" in at) || menus.railed()) return at;
        const depth = grid.depthOf(at.item), next = places[places.indexOf(at.item) + 1], low = next ? grid.depthOf(next) : 0;
        if (!at.after || low >= depth) return { ...at, depth };
        return { ...at, depth: clamp(Math.floor((x - at.item.getBoundingClientRect().left - 13) / 16), low, depth) };
      };
      if (onto) {
        const box = onto.getBoundingClientRect(), share = (y - box.top) / box.height, folder = folderById(onto.dataset.folderRow);
        const first = places[places.indexOf(onto) + 1], inside = first && folder && grid.parentOf(/** @type {Chart | Folder} */ (grid.itemOf(grid.sideId(first)))) === folder.id ? first : null;
        if (share < 0.25) return level({ item: onto, after: false });
        if (share > 0.75) return folder?.closed || menus.onRail() ? level({ item: onto, after: true }) : inside ? level({ item: inside, after: false }) : { zone: onto };
        return { zone: onto };
      }
      return level(grid.landingIn(places, y, grid.sideId, ids));
    },
    land: (ids, at, y) => {
      if ("zone" in at) return grid.placeItems(ids, at.zone.dataset.folderRow, null, { dropY: y });
      // (After a row, levels out from it: after the folder it's in at that level.)
      let key = grid.sideId(at.item);
      for (let up = grid.depthOf(at.item) - (at.depth ?? grid.depthOf(at.item)); up > 0; up--) {
        const item = grid.itemOf(key), parent = item && grid.parentOf(item);
        if (parent) key = `f:${parent}`;
      }
      const { into, before } = grid.besideItem(key, at.after, ids);
      grid.placeItems(ids, into, before, { dropY: y });
    },
    // (Held, its menu showed (see below): carried on from there, it goes, as iOS's does.)
    lift: () => menus.hideMenu(),
  });
  // A row's menu by touch: held, the one its ⋯ shows (by touch out of sight, the list kept quiet: see the rows' ⋯).
  // (On the rail, a press shows it already.)
  gestures.onHold($("chart-list"), ".nav-item, .list-head-name", (row) => ((more) => (more && !menus.railed() ? () => (more.click(), true) : null))(/** @type {HTMLElement | null} */ (row.parentElement?.querySelector(':scope > .chart-more[aria-haspopup="menu"]') ?? null)));
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
      // (Not among the previews: they're none of your charts' order.)
      if (under?.closest(".grid-tiles.previews")) return undefined;
      const tile = /** @type {HTMLElement | null | undefined} */ (under?.closest("[data-home-card]"));
      // Off the tiles: on a folder's shelf, into it; elsewhere on the page's charts, into the folder it's in.
      if (!tile) {
        const head = under?.closest(".shelf")?.querySelector("[data-grid-folder]");
        return head ? { zone: /** @type {HTMLElement} */ (head) } : under?.closest("#home-grid") ? { zone: $("home-grid") } : undefined;
      }
      if (ids.includes(/** @type {string} */ (tile.dataset.homeCard))) return null;
      // (Beside it, in a grid of columns or on a shelf; above or below it, in a grid of one: a phone's, upright.)
      const box = tile.getBoundingClientRect(), row = /** @type {HTMLElement} */ (tile.parentElement), down = row.matches(".grid-tiles") && getComputedStyle(row).gridTemplateColumns.split(" ").length < 2;
      return down ? { item: tile, after: y > box.top + box.height / 2, down } : { item: tile, after: x > box.left + box.width / 2 };
    },
    land: (ids, at) => {
      const keys = ids.map((id) => `c:${id}`);
      if ("zone" in at) grid.placeItems(keys, at.zone.dataset.gridFolder || ("folder" in homeAt ? homeAt.folder : undefined));
      else {
        // (Beside a tile: in its folder, before or after it.)
        const { into, before } = grid.besideItem(`c:${at.item.dataset.homeCard}`, at.after, keys);
        grid.placeItems(keys, into, before);
      }
      grid.tileOpen(ids[0])?.focus({ preventScroll: true });
    },
  });
  grid.startListKeys();
});

// ── Buttons (the open chart's, Home's new folder), the chart list's fit, the cards to start from ──
$("keep").addEventListener("click", () => keepChart());
$("preset-copy").addEventListener("click", () => presetsEdit.copyPreset(presetIndex(chart)));
$("preset-reset").addEventListener("click", () => presetsEdit.resetPreset(chart));
$("chart-more").addEventListener("click", (event) => {
  const button = /** @type {HTMLElement} */ (event.currentTarget);
  // (On a phone, headed by the chart's whole name: its title may be cut short there.)
  menus.menuOf(button, command.headMenu());
});
$("home-new-folder").addEventListener("click", (event) => {
  event.preventDefault();
  grid.newFolder([], "folder" in homeAt ? homeAt.folder || undefined : undefined);
});
$("chart-list").addEventListener("scroll", () => fitSoon($("chart-list"), true), { passive: true });
new ResizeObserver(() => fitStrip($("chart-list"), true)).observe($("chart-list"));
// The empty chart offers the recommended presets, each with its mini chart, name and place, in a row that scrolls
// (waiting cards, the room they take, until the presets are read).
strip($("templates"));
new ResizeObserver(() => fitStrip($("templates"))).observe($("templates"));
/** The cards waiting, each one's drawing started: their fetches go together, each drawn as its data comes. */
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

// ── The presets: every one, a chart to start from ─────────────────────────
/** See looksOf: empty till the presets are read. */
let presetLooks = /** @type {Map<string, Partial<Layer>>} */ (new Map());
/** Their places' steps shown otherwise than their places say, by key (see shownStep): none till they're read. */
let presetLabels = /** @type {Record<string, string>} */ ({});
/** The presets, once read. */
let presets = /** @type {Preset[] | null} */ (null), presetsFailed = false;
/** @type {Promise<Preset[]> | null} */
let presetsLoad = null, presetsRead = /** @type {Promise<Preset[]> | null} */ (null);
/**
 * The presets' list, read once (made from presets.js and the client, see presetsKept), and the looks they give series
 * (see looksOf). Nothing else: nothing opens or shows (that's loadPresets').
 */
function readPresetList() {
  return (presetsRead ??= (async () => {
    const { presets: list, labels } = await presetsKept();
    presetLabels = labels;
    presetLooks = looksOf(list);
    return list;
  })());
}
/**
 * The presets, read once, when first wanted (readPresetList), then shown: read, ⌘K, open, shows them.
 */
function loadPresets() {
  return (presetsLoad ??= (async () => {
    presetsFailed = false;
    const list = await readPresetList();
    // (A frame between their reading and their showing: neither one long one, Firefox's a tenth of a second.)
    await aFrame();
    presets = list;
    // (Its tree: see presetTree.)
    presetTree();
    // (What a search reads (see searchNow), then their words each once, for a slip in a search (see slipsOf), and their
    // charts' ids (see presetById), gathered meanwhile: no keystroke, no way back to the grid, waits for them.)
    withRest(() => whenIdle(grid.searchNow));
    whenIdle(idsNow);
    // (A search narrowed from the last one's found: of the list before, let go of.)
    presetsFound = { query: "", found: [] };
    // (Those opened and left as they were, before they could be told: let go of now; the open one, as the presets
    // have it now, if that's changed.)
    // (Adopted first: copies left as they were, stored, are still here for an adoption's stamp to pass.)
    adoptPresets();
    prunePresets(chart ?? null);
    // (The open one opened and left as it was, its place gone (renamed since): the preset that shows it now, as made,
    // in its place, Recent's too; it let go of as it's left. None does, it stays as it is, in none of their sections.
    // A link in the address bar being read (it waits for them): its chart opens instead, nothing opened before it.
    // Only before a hand's been on the page: one may be editing it (a path half drawn, a title typed in ⌘K); left to
    // it then, an edit moved with the presets on the next start, see adoptPresets.)
    // (Asked first: making the one to open takes the open one's place.)
    const at = chart?.preset === undefined || (reading.hash && currentRead(reading.run, reading.at)) ? -1 : presetIndex(chart) >= 0 ? presetIndex(chart) : pristine(chart) && !handledYet ? presetShowing(chart) : -1, now = at >= 0 ? presetChart(at) : chart;
    if (now && now !== chart) openChart(now);
    withRest(() => (grid.renderRecommended(), presetsEdit.presetsArrived(), renderCharts()));
    // (Their addresses made while the page is idle: not by the first page change that names one, mid-change.)
    whenIdle(addressesSoon);
    withRest(() => whenIdle(command.ranksSoon));
    // (An address that waited for them, opened now, as the rest is in: its page may be the grid.)
    if (pendingRoute && started) withRest(() => {
      // (Still on the step it was asked from, there; gone on since (a retry later, elsewhere), where it is now: a
      // missed one only said.)
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
      // (The sidebar's presets as kept, let go of: its head alone.)
      renderCharts();
    });
    throw error;
  }));
}
/** The presets read again, asked for (after a failure). */
function retryPresets() {
  presetsLoad = presetsRead = null;
  presetsFailed = false;
  grid.renderRecommended();
  void loadPresets().catch(() => {});
  if (isOpen($("palette"))) showPalette();
  renderHome();
}
// ── Addresses: the presets' page, each section, each preset, to be found ──────────────────
/** Whether the page is Studio's own, at BASE (its server serving the page at every address under it): an address of the presets' read as it starts (an old /chart/ one its server can't place, too: said missing); elsewhere (a file, another server's), none. */
const ROUTED = /^\/studio\/(?:$|presets\/|chart\/)/.test(location.pathname);
/** An address of the presets' naming none of them (renamed, or gone): their page (its step), saying so. @param {string} address */
function missedRoute(address) {
  // (Still on the page it opened on, there in its step, as goTo leaves a page; a chart opened meanwhile, said alone.)
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
/** Whether start() has made the page what it shows first (till then, the address is the one asked for). */
let started = false;
// (Nothing acted on before then (no chart open yet, a link still being read): a key, a press or a click, let go of,
// the browser's own keys aside; a click's own doing too (a link followed: a hash of its own drops the link read).
// The page's Reload clicked, said while it's slow to start, taken.)
for (const type of ["keydown", "pointerdown", "click"]) window.addEventListener(type, (event) => !started && !(type === "click" && event.target instanceof Element && event.target.closest(".booting")) && (event.stopImmediatePropagation(), type === "click" && event.preventDefault()), true);
/** An address asked for that waits for the presets (they failed to load at first): opened once they're read. */
let pendingRoute = "";

// ── The presets, edited where they are ─────────────────────────────────────
/**
 * A preset opened at its address (a visit, not a step back): what the address says of how to look at it, else as it
 * opens, whatever view was left on it here (whoever follows a link sees what it says); `window`, an old address's
 * (see routeOf), unless its link says one. Edited here: that copy, as it is, its look only what the link says.
 * @param {number} i @param {string} [window]
 */
function presetByAddress(i, window = undefined) {
  const doc = presetChart(i);
  takeLinkLook(doc, linkLookOf(new URLSearchParams(location.hash.slice(1)), window), presetEdited(doc) ? undefined : presetLook(i));
  linkWindow = doc.id;
  return doc;
}
/**
 * A preset picked (its tile, its row, ⌘K's, the empty chart's cards): opened, as itself, whatever's open, and where
 * it is in the sidebar in sight: the presets opened (the sections it's in open as it's the open one: see revealed),
 * its rows (marked, as the open one's are) in sight, the nearest scrolled to (see revealNearest).
 */
function pickPreset(/** @type {number} */ i) {
  if (!presets?.[i]) return;
  // (The group opened before it opens, so its own drawing of the list shows it, scrolled to as it's marked. A phone has
  // no sidebar to show it in.)
  if (!phone.matches) {
    if (grid.groupShut("presets")) (prefs.groupsOpen = [...prefs.groupsOpen, "presets"]), savePrefs();
    presetToReveal = i;
  }
  presetsEdit.openPreset(i);
  // (The open one again, off the grid: nothing drawn the list, so it's drawn here.)
  if (!home && presetToReveal === i) renderCharts();
}
/** The preset whose row is scrolled to the next time the list marks it open (see pickPreset); -1, none. */
let presetToReveal = -1;
/** The last search over the presets (all of them, no `under`): typed on, it narrows what it found. */
let presetsFound = { query: "", found: /** @type {number[]} */ ([]) };
/**
 * The presets whose words hold every word typed (`words`, see typedWords), the best first (what's typed its name, or
 * its title's subject, before ": "; then a whole name's initials; then word prefixes, initials, and substrings,
 * each preferring every word in the title, then its title starting with the first; at each,
 * the nearest the top of the tree, then the shortest title: the measure itself before its families' members), as
 * many as `most`; none holding them all, those holding the most of them (see below). With how many it found, all of
 * them (`count`), and whether they hold only some of the words (`partial`).
 * @returns {{ found: number[], count: number, partial: boolean }}
 */
function presetsMatching(/** @type {string[]} */ words, /** @type {number} */ most, under = /** @type {((i: number) => boolean) | null} */ (null), slipped = false) {
  const list = /** @type {Preset[]} */ (presets), query = words.join(" "), { texts: made, initials: madeInitials, titles, rank, order, exact } = grid.presetSearch();
  // (An edited one's words, its title as it is now too. The copies kept here by their place, found once, not looked for
  // among them all at each match: thousands, a word or two typed.)
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
  // (One scan: each one's words held, counted, and kept those holding the most, every word where any does. Else, of
  // the presets as a whole, at least half of them and two (one word of two is a search for that word alone, not a
  // name half remembered: an older release's, "reused outputs share spendable"), said to be partial where they're
  // counted, ⌘K's rows marking the words they hold; in a section, none: its note offers the presets as a whole. One's
  // let go once it can't hold as many: holding every word, at its first missing. Index loops, not iterators (all of
  // them, `among` none), the first word tested inline, a word alone a loop of its own (a search's first letter: every
  // preset found): thousands each keystroke, most let go at the first word, a phone's engine without a JIT.)
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
  // (Typed on from the last search, those holding every word now are among those that held every word then: looked
  // through first; none, the presets as a whole for the most held. In a section, only its own.)
  const among = !under && !renamed.size && presetsFound.query && query.startsWith(presetsFound.query) ? presetsFound.found : null;
  const all = under ? order.filter(under) : null;
  let { found, top } = scan(among ?? all, among ? wordCount : least);
  if (among && !found.length && least < wordCount) ({ found, top } = scan(all, least));
  const partial = found.length > 0 && top < wordCount;
  // (Typed on, a search narrows only what held every word.)
  if (!under) presetsFound = { query, found: partial ? [] : found };
  // (None holding every word, and a word in no preset's words a slip off one that is (as ⌘K's rows forgive one, see
  // fitScore): found as that word, as series and settings are found through a slip; of the words each could be (the
  // nearest first, then those more presets have), the first together found (where it's searched: a few tried, a
  // dozen at most), else the nearest of each. A slip only: the words as typed first.)
  if ((!found.length || partial) && !slipped) {
    const extra = [...renamed.values()].map(command.searchCase), choices = words.map((word) => ((slips) => (slips.length ? slips : [word]))(presetsEdit.slipsOf(word, texts, extra, initials)));
    if (choices.some((options, k) => options[0] !== words[k])) {
      const anyHolds = (/** @type {string[]} */ some) => {
        for (let i = 0; i < texts.length; i++) if ((!under || under(i)) && some.every((word) => texts[i].includes(word) || (initialsWord(word) && initials[i].includes(word)))) return true;
        return false;
      };
      // (Each combination in turn, the last word's choices turning fastest, as an odometer's.)
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
  // (Each one's title as read, but an edited one's as it is now.)
  const holdsAll = (/** @type {string} */ text, /** @type {string} */ initial) => {
    for (let k = 0; k < wordCount; k++) if (!text.includes(words[k]) && !initial.includes(initialWords[k])) return false;
    return true;
  };
  // Whole words and prefixes lead initials; initials lead incidental substrings of other words.
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
  // (A renamed one ranked by its title as it is now: see presetSearch.)
  const rankOf = (/** @type {number} */ i) => ((title) => (title === undefined ? rank[i] : grid.presetRankOf(list[i].p.length, title)))(renamed.get(i));
  const byRank = (/** @type {number} */ a, /** @type {number} */ b) => rankOf(a) - rankOf(b) || a - b;
  // (Exactly what's typed: looked up, and the copies kept, whose title may say it now.)
  const best = [...new Set([...(exact.get(query) ?? []), ...copies.keys()])].filter((i) => inFound[i] && fitOf(i) >= 14).sort((a, b) => fitOf(b) - fitOf(a) || byRank(a, b)).slice(0, most);
  // (Those found in rank order: a few, sorted; many, read along the presets' order, the renamed ones (few) in their
  // own places.)
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
// Read once the page has settled: the recommended cards fill in, and ⌘K finds them at once.
withRest(() => {
  grid.renderRecommended();
});
void loadPresets().catch(() => {});
withRest(() => {
  document.addEventListener("fullscreenchange", () => !document.fullscreenElement && fullOn() && presetsEdit.setFull(false));
});
// (Full screen slides the header up by its own height, as it is: a phone's, a desktop's.)
new ResizeObserver(([entry]) => /** @type {HTMLElement} */ (document.querySelector(".app")).style.setProperty("--head-height", `${entry.borderBoxSize[0].blockSize}px`)).observe(/** @type {HTMLElement} */ (document.querySelector(".card-head")));
// Escape is the page's: it does here what it does, and goes no further, so a browser's own full screen (macOS's,
// ⌃⌘F) doesn't leave with it. (One a page asked for is the browser's to leave, see setFull.)
window.addEventListener("keydown", (event) => event.key === "Escape" && !document.querySelector("dialog:popover-open") && event.preventDefault(), true);
// (A dialog's too: nothing in it having taken the key, its own way out, as the browser's would be (its cancel, then
// closed unless that's refused), the key going no further: left to the browser, Firefox's full screen left with it.)
window.addEventListener("keydown", (event) => {
  // (The one on top.)
  const dialog = event.key === "Escape" && !event.defaultPrevented && !event.isComposing ? topDialog() : undefined;
  if (!dialog) return;
  event.preventDefault();
  if (dialog.dispatchEvent(new Event("cancel", { cancelable: true }))) dialog.hidePopover();
});
// (Tab kept in the dialog on top, as a modal's is (see isOpen): past its last stop, its first; ⇧Tab before its
// first, its last; from outside it, in. The focus in what's over it (a menu, Quick Look), theirs.)
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
/** The layer list's parts as last written, by key (its panes', then what's under them). */
let layersWritten = { keys: /** @type {string[]} */ ([]), tail: /** @type {[string, string][]} */ ([]) };
/** The layers' list behind the chart (written while hidden: not yet). */
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
  // (Its rows are the rest's: written as it comes.)
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
  // Each pane's head and rows, and what's under them, written again only where they changed (a layer selected:
  // two rows; a value typed: none).
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
        // (Its second line: what's wrong, what it's between or its formula; a series or a heatmap, nothing (its id is the inspector's, not the list's); said whole in
        // its tip, under its name, when either's cut short.)
        const [subClass, subText] = result?.error ? ["sub error", result.error] : layer.from ? ["sub", fromUnfit(chart, layer) || `Between ${fromSpan(layer)}${((silent) => (silent.length ? ` · ${silent.join(", ")} ${silent.length === 1 ? "has" : "have"} no data` : ""))(resolved ? chart.layers.filter((item) => layer.from?.includes(item.id) && !resolved?.results.get(item.id)?.values).map((item) => item.letter) : [])}`] : layer.formula !== undefined ? (layer.label ? ["sub code", `ƒ\u00a0${layer.formula}`] : ["sub", formulaLine(layer)]) : ["sub", ""];
        // (Its letter heading its second line, its mark leading the row: see .row-mark.)
        const sub = `<span class="${subClass}"${result?.error && result.detail ? ` data-tip="${escapeHtml(result.detail)}"` : ""}><span class="sub-letter">${letterChip(layer)}</span>${escapeHtml(subText)}</span>`;
        // (Shown as the chart has it (a signal, by its layer too); one shown only but for its layer: that layer's eye, dimmed,
        // a press showing it.)
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

// Layers dragged: before a layer (or after, on its lower half), in its pane; or below, into a new pane.
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

// ── Picking layers, and what the bar does to them ─────────────────────────

/** The panel shown with the list (a layer open in it closes; a phone's sheet opens, a closed panel peeks). */
function showList() {
  // (Let go of, as deselect lets go: its row, its legend's, its settings drawn so.)
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
/** Picks a layer: added or taken off; with `range`, every one from the last picked (or the one open) to it. */
function pickLayer(/** @type {string} */ id, range = false) {
  menus.hideMenu();
  // (Nothing picked, a run starts from the layer open, or the one last open: Back leaves it where a run starts.)
  const ids = chart.layers.map((layer) => layer.id), start = layerPicks.size ? layerPickLast : selected || lastOpen, from = ids.indexOf(start), to = ids.indexOf(id);
  if (range && from >= 0 && to >= 0) for (const other of ids.slice(Math.min(from, to), Math.max(from, to) + 1)) layerPicks.add(other);
  else if (layerPicks.has(id)) {
    // (One taken off isn't where a run starts: the last one picked that's still there is.)
    layerPicks.delete(id);
    if (!layerPicks.has(layerPickLast)) layerPickLast = [...layerPicks].at(-1) ?? "";
    pickHead = layerPickLast;
    return inspector.showPicks();
  } else {
    // (The layer open, as a run from it takes it: picked with the one added.)
    if (!layerPicks.size && selected && selected !== id) layerPicks.add(selected);
    layerPicks.add(id);
  }
  layerPickLast = range && from >= 0 ? start : id;
  pickHead = id;
  lastOpen = "";
  inspector.showPicks();
}
/** ⇧↓ or ⇧↑: the run from where picking started to the next or previous layer (from the one open, or in focus, at first). */
function extendPicks(/** @type {number} */ delta) {
  const ids = chart.layers.map((layer) => layer.id), focused = /** @type {HTMLElement | null} */ (document.activeElement?.closest?.(".layer-row[data-layer]") ?? null);
  if (!ids.length) return;
  const anchor = layerPicks.size ? layerPickLast : selected || focused?.dataset.layer || "", start = anchor || ids[delta > 0 ? 0 : ids.length - 1], from = ids.indexOf(start);
  if (from < 0) return;
  // (From nothing, the first one alone, as ↓ takes it.)
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
  // (Marked here, not as last written: the next writes write them whole.)
  layersWritten = { keys: [], tail: [] };
  inspector.renderLayersBar();
  if (inspector.multiEdit) inspector.renderInspector();
}
/** No more picks; `refocus`: the focus goes back to the row last picked (from the bar's ×, it'd be lost). */
function clearLayerPicks(refocus = false) {
  layerPicks.clear();
  lastOpen = "";
  markPicks();
  if (refocus) /** @type {HTMLElement | null} */ ($("layer-list").querySelector(`[data-select="${layerPickLast}"]`))?.focus();
}
/** What the bar (or a picked row's right-click, ⌘K, a key) does to the layers picked. @param {string} action @param {HTMLElement} [anchor] */
function layersBatch(action, anchor) {
  const ids = inspector.pickedLayers().map((layer) => layer.id);
  if (action === "clear" || !ids.length) return clearLayerPicks(action === "clear");
  if (action === "edit") {
    // (One picked: its own settings, as selecting it opens them.)
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

// ── Inspector ─────────────────────────────────────────────────────────────
/** A layer selected, its settings shown (on a phone, in its sheet; `sheet` false: one just added, left as it is there). */
function select(id, sheet = true) {
  // (On a phone, one just added stays as it is, unselected: the chart in sight, the list the list; Edit opens it.)
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
  // (Drawings selected, let go: painted so.)
  if (unchosen) repaintDrawings();
  if (fromList && selected) /** @type {HTMLElement | null} */ ($("inspector").querySelector('[data-action="deselect"]'))?.focus();
}
function deselect() {
  // (Several opened together: back to the list, still picked.)
  if (inspector.editingMany()) {
    inspector.editPicked(false);
    return void /** @type {HTMLElement | null} */ ($("layers-bar").querySelector('[data-layers-batch="edit"]'))?.focus();
  }
  const was = selected, fromSettings = $("inspector").contains(document.activeElement), peeking = document.body.hasAttribute("data-peek");
  selected = "";
  lastOpen = was;
  // (Open for the while, the panel goes: a key's focus in it to its button, else the chart.)
  if (peeking) keepFocus($("panel"), () => $("panel-toggle"), () => $("plot-keys"));
  document.body.removeAttribute("data-peek");
  inspector.showSelection();
  if (fromSettings && !peeking) /** @type {HTMLElement | null} */ ($("layer-list").querySelector(`[data-select="${was}"]`))?.focus();
}

/** A reset appears only away from the default (tabs need none: their default is one click away). */
const DEFAULTS = {
  opacity: { is: (layer) => layer.opacity === undefined, set: (layer) => (layer.opacity = undefined), text: (layer) => `${Math.round(defaultOpacity(layer) * 100)}%`, focus: "#opacity" },
  width: { is: (layer) => layer.width === 1.5, set: (layer) => (layer.width = 1.5), text: "1.5px", focus: "#width" },
  "fill-opacity": { is: (layer) => layer.fillOpacity === undefined, set: (layer) => (layer.fillOpacity = undefined), text: (layer) => `${Math.round(fillOf({ ...layer, fillOpacity: undefined }, inspector.stackedIn(layer))[0] * 100)}%`, focus: "#fill-opacity" },
  base: { is: (layer) => baseOf(layer) === 0, set: (layer) => (layer.base = undefined), text: "0", focus: "#base" },
  format: { is: (layer) => !layer.format, set: (layer) => (layer.format = undefined), text: "Automatic", focus: "#format" },
  "bubble-size": { is: (layer) => layer.size === undefined, set: (layer) => (layer.size = undefined), text: `${BUBBLE_SIZE}px`, focus: "#bubble-size" },
  "bubble-hide": { is: (layer) => layer.hide === undefined, set: (layer) => (layer.hide = undefined), text: hideText(BUBBLE_HIDE), focus: "#bubble-hide" },
};
/** The inspector's sliders, each a few values in order: what it steps through, how its value reads, what it sets. */
const SLIDERS = {
  width: { label: "Width", stops: WIDTHS, of: (layer) => layer.width, text: (v) => `${v}px`, say: (v) => `${v} pixels`, set: (layer, v) => (layer.width = v) },
  // A drawing's, on the belt: the same stops, read the same (its look set by setDrawSlider).
  "draw-width": { label: "Width", stops: WIDTHS, of: (look) => look.width, text: (v) => `${v}px`, say: (v) => `${v} pixels` },
  "draw-opacity": { label: "Opacity", stops: OPACITIES, of: (look) => look.opacity ?? 1, text: (v) => `${Math.round(v * 100)}%`, say: (v) => `${Math.round(v * 100)} percent` },
  "fill-opacity": { label: "Fill opacity", stops: OPACITIES, of: (layer) => fillOf(layer, inspector.stackedIn(layer))[0], text: (v) => `${Math.round(v * 100)}%`, say: (v) => `${Math.round(v * 100)} percent`, set: (layer, v) => (layer.fillOpacity = v === fillOf({ ...layer, fillOpacity: undefined }, inspector.stackedIn(layer))[0] ? undefined : v) },
  "bubble-size": { label: "Size", stops: BUBBLE_SIZES, of: (layer) => layer.size ?? BUBBLE_SIZE, text: (v) => `${v}px`, say: (v) => `${v} pixels`, set: (layer, v) => (layer.size = v === BUBBLE_SIZE ? undefined : v) },
  "bubble-hide": { label: "Hide below", stops: BUBBLE_HIDES, of: (layer) => layer.hide ?? BUBBLE_HIDE, text: hideText, say: (v) => (v ? `${Math.round(v * 100)} percent of the largest` : "None"), set: (layer, v) => (layer.hide = v === BUBBLE_HIDE ? undefined : v) },
  opacity: { label: "Opacity", stops: OPACITIES, of: opacityOf, text: (v) => `${Math.round(v * 100)}%`, say: (v) => `${Math.round(v * 100)} percent`, set: (layer, v) => (layer.opacity = v === defaultOpacity(layer) ? undefined : v) },
};
/**
 * A press that drags: `move` runs at the press and at every move of its pointer until it ends (see press), wherever
 * it goes, the body's `cursor` class holding the cursor meanwhile. One at a time: while one's held, another press
 * (a second finger) does nothing.
 */
function track(event, element, cursor, move) {
  event.preventDefault();
  if (holding) return;
  element.focus();
  // (Its edits one undo step, `holding` while it's held: see commit.)
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
// A cursor by the keys: ← → a step (⇧, ten), ⌫ gone (two left at least).
$("inspector").addEventListener("keydown", (event) => {
  const cursor = /** @type {HTMLElement} */ (event.target);
  if (!cursor.classList?.contains("gradient-stop")) return;
  const k = Number(cursor.dataset.stop), by = event.key === "ArrowRight" ? 1 : event.key === "ArrowLeft" ? -1 : 0;
  if (by) {
    event.preventDefault();
    event.stopPropagation();
    // (The one moved, the one in hand: its row, its palette.)
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
// A place typed (or stepped by ↑ ↓): its color there as it's typed, between its neighbors, its keys within a second
// one undo step (coalesced), as every number's field's; the field kept as typed till it's left, then showing where
// the color is.
$("inspector").addEventListener("input", (event) => {
  const field = /** @type {HTMLInputElement} */ (event.target);
  if (!field.dataset?.stopAt) return;
  const k = Number(field.dataset.stopAt), at = inspector.readNumber(field.value);
  if (!field.value.trim() || !Number.isFinite(at)) return;
  inspector.editLayer((item) => inspector.moveStop(item, k, at), { coalesce: `stop:${selected}:${k}` });
  // (Stepped, not typed (an InputEvent): the field at where it went, never past its neighbors.)
  inspector.showStop(k, !(event instanceof InputEvent));
});
// (Left, not changed: a place stepped by ↑ ↓ is set by script, no `change` to follow.)
$("inspector").addEventListener("focusout", (event) => {
  const field = /** @type {HTMLInputElement} */ (event.target);
  if (field.dataset?.stopAt) inspector.showStop(Number(field.dataset.stopAt));
});
// (Several at different stops ("Mixed"): a key steps from the first's, and sets them all there.)
withRest(() => {
  inspector.sliders($("inspector"), { valueOf: (id) => ((layers) => (layers.length ? inspector.sharedBy(layers, SLIDERS[id].of) ?? SLIDERS[id].of(layers[0]) : undefined))(inspector.inHand()), set: inspector.setSlider });
});
// ── Gradients: colors by value ───────────────────────────────────────────────
/** Custom: the surface alone, its color picked live (the page stays open), Bitview's colors a step back. */
function colorPane(/** @type {HTMLElement} */ anchor, /** @type {string} */ color, /** @type {string} */ target, /** @type {() => void} */ back) {
  menus.showMenu(anchor, `<div class="field belt-pop color-pop color-pane"><span><button type="button" class="cap small" data-pane-back="1" aria-label="Bitview's colors" data-tip="Bitview's colors">${ICONS.back}</button>Custom</span><div class="color-box">${inspector.planeHtml(color, target)}${inspector.hexField(target, color)}</div></div>`, ({ paneBack }) => paneBack && back(), null, anchor.closest("#belt") ? pen.beltSide() : "");
  menus.setMenuBack(back);
  inspector.drawPlanes();
  inspector.planeOf(target)?.focus({ preventScroll: true });
}
// The panel's hex: ↵ takes what reads (a #fff, an OKLCH, a name too) and leaves it, esc leaves it; left, the panel's
// drawn again (it shows the color as it is).
document.addEventListener("keydown", (event) => {
  const field = /** @type {HTMLInputElement} */ (event.target);
  if (!field.matches?.("#inspector .color-editor .hex") || composing(event) || (event.key !== "Enter" && event.key !== "Escape")) return;
  event.preventDefault();
  event.stopPropagation();
  const color = event.key === "Enter" ? command.readColor(field.value) : null, key = field.closest("[data-palette]")?.getAttribute("data-palette");
  if (color && key) inspector.editLayer((item) => inspector.setColor(item, key, color));
  menus.letGo(field);
}, true);
// (Another's hex (a level's, a popover's): ↵ takes what reads there too, before its menu closes.)
document.addEventListener("keydown", (event) => {
  const field = event.target;
  if (event.key !== "Enter" || composing(event) || !(field instanceof HTMLInputElement) || !field.dataset.hex || field.matches("#inspector .color-editor .hex")) return;
  const color = command.readColor(field.value);
  if (color) inspector.pick(field.dataset.hex, color), inspector.flushPick();
}, true);
withRest(() => {
  document.addEventListener("focusout", (event) => /** @type {HTMLElement} */ (event.target).matches?.("#inspector .color-editor .hex") && inspector.inspectorDue && inspector.afterPress(() => setTimeout(inspector.renderInspector)));
  // A color typed in hex (#3b82f6, or without its #): picked as the surface picks it, its ring moved there.
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
// Keys move over a surface: ← → a degree of hue (on into the grays), ↑ ↓ a step of lightness; ⇧ for ten. (Before a
// menu's own keys, when it's in one.)
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
// A surface picks under the pointer, wherever it is (the inspector, a popover); by touch, as pickPress has it.
withRest(() => {
  document.addEventListener("pointerdown", (event) => {
    const plane = closest(event, ".plane");
    if (event.button !== 0 || !plane) return;
    const name = /** @type {string} */ (plane.dataset.plane);
    inspector.pickPress(event, plane, "picking", (/** @type {PointerEvent} */ point) => inspector.pickAt(name, point.clientX, point.clientY));
  });
});
/** Mirror colors, status line, name placeholder: everything that follows the formula as it's typed. */
/** The formula editor as tall as its lines, its width changed (a phone turned, the panel resized). */
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
      // Its Reset put in or taken out (only it: the panel drawn again would lose the click that left the field).
      const layer = layerById(selected), label = field.closest(".field")?.firstElementChild, reset = label?.querySelector(".reset");
      if (layer && label && !DEFAULTS.base.is(layer) && !reset) label.insertAdjacentHTML("beforeend", inspector.resetFor("base", layer));
      else if (layer && DEFAULTS.base.is(layer)) reset?.remove();
      return;
    }
    if (field.id === "formula") inspector.formulaFieldLeft(/** @type {HTMLTextAreaElement} */ (event.target));
  });
});
// A number's field stepped by its arrows, as a number field is: a unit (with ⇧, ten).
$("inspector").addEventListener("keydown", (event) => {
  const field = /** @type {HTMLInputElement} */ (event.target);
  if (!field.matches?.(".number-input") || (event.key !== "ArrowUp" && event.key !== "ArrowDown") || event.metaKey || event.ctrlKey || event.altKey) return;
  const now = inspector.readNumber(field.value);
  if (!Number.isFinite(now)) return;
  event.preventDefault();
  // (Rounded to the places it has, not to a precision: 1234567890123 and up step as they are.)
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
    // (The band under its levels: what its values read as.)
    const name = target.value.trim().slice(0, 40);
    inspector.editLayer((layer) => (name ? (layer.underName = name) : delete layer.underName), { coalesce: `under-name:${selected}` });
  } else if (target.id.startsWith("level-name-")) {
    // (A level's name: what the values from it up read as, in its color.)
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
    // (Coloring it: the band under them follows in place, the field kept as it's typed in.)
    const layer = layerById(selected), under = $("inspector").querySelector(".under-label");
    if (under && layer?.levels?.length) under.innerHTML = `<span class="sign" aria-hidden="true">&lt;</span><span class="vh">Under </span>${Math.min(...layer.levels.map((item) => item.value))}`;
  }
});
// (A control pressed in the panel, not a field typed in: what it edits draws the whole panel again. See renderInspector.)
$("inspector").addEventListener("pointerdown", (event) => {
  if (!closest(event, "input, textarea")) $("inspector").dataset.pressed = "1";
  else delete $("inspector").dataset.pressed;
}, true);
$("inspector").addEventListener("input", () => delete $("inspector").dataset.pressed, true);
// A level's field left: the levels drawn again as they are now (in their order, the band under them shown once one's named).
$("inspector").addEventListener("focusout", (event) => {
  if (!/** @type {HTMLElement} */ (event.target).closest?.(".levels")) return;
  inspector.afterPress(() => requestAnimationFrame(() => !$("inspector").querySelector(".levels")?.contains(document.activeElement) && inspector.renderInspector()));
});
$("inspector").addEventListener("keydown", (event) => {
  // A picker (pane, format, color) opens its menu on ↑↓, as a select does: the keys don't go on to the layers.
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
    // ↵ is done, as in every field (⇧↵ a line), once it reads; esc too, as typed, or as it last read (⌘Z takes it back).
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
  // (A gradient's color pressed by a pointer: the one nearest the press, taken as it went down (see the bar's
  // pointerdown), not the one whose reach is on top.)
  else if (slot) (button.matches(".gradient-stop") && event.detail) || inspector.colorSlot.set(`${chart.id}:${selected}`, slot), inspector.renderInspector();
  else if (source && paletteKey) {
    // (Auto is a color: its own, set now; Palette and Custom, where to pick one: shown. Several in hand, theirs.)
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
    // (Removed: the focus on the row in its place, or the way to add one.)
    if (row.key === "remove") (all$(".layer-row .row-main", $("layer-list"))[Math.min(at, chart.layers.length - 1)] ?? /** @type {HTMLElement | null} */ (document.querySelector("#empty [data-action='add-series']")))?.focus();
  }
  else if (action === "pane-menu") inspector.paneMenu(button, inspector.inHand().map((layer) => layer.id), () => presetsEdit.refocus("#pane"));
  // (A gradient's: a color added, one removed, one ready made (its menu), reversed, relative to all of history or up to each date.)
  else if ((action === "gradient-add" || button.dataset.stopRemove) && button.getAttribute("aria-disabled") === "true") notice(/** @type {string} */ (button.dataset.tip));
  else if (action === "gradient-add") inspector.gradientEdit((item) => inspector.addStop(item), true);
  else if (button.dataset.stopRemove) inspector.gradientEdit((item) => inspector.removeStop(item, Number(button.dataset.stopRemove)), true);
  else if (action === "gradient-ready") menus.menuOf(button, /** @type {any} */ (inspector.readyPage(selected).options?.() ?? []));
  else if (action === "gradient-reverse") inspector.reverseGradient(selected), presetsEdit.refocus('#inspector [data-action="gradient-reverse"]');
  else if (button.dataset.gradientTo !== undefined) inspector.editLayer((item) => (item.upToDate = button.dataset.gradientTo ? true : undefined));
  // (A heatmap's holders or density: its page's choices, see heatRows; its measure or weight, one of its tabs.)
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
    // (No line, no value on the axis: nothing to turn on.)
    if (!layer || !lineAt(layer, lineTag)) return;
    // (Not to be had: why, said, as every control that can't be pressed says it (a finger has no tip).)
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
    // (By touch, a sidebar row's long press: its hold shows its menu, see onHold. Not a key's (⇧F10), a mouse's, nor
    // a field's own.)
    if (byTouch && closest(event, "#chart-list") && !typing(event.target)) return void event.preventDefault();
    // (The browser's own, after the menu key opened a row's menu: that one's open.)
    if (event.isTrusted && performance.now() - grid.keyMenuAt < 500 && closest(event, "#chart-list")) return void event.preventDefault();
    // (A row on the rail: its one menu, as pointing at it or pressing it shows (see railMenu).)
    const railItem = menus.onRail() && !closest(event, ".row-rename, .folder-rename") ? /** @type {HTMLElement | null} */ (closest(event, "#chart-list > *")?.querySelector(":scope > :is([data-open-chart], [data-open-preset], [data-folder-item], [data-section-item], [data-group-item])") ?? null) : null;
    if (railItem) {
      event.preventDefault();
      return railMenuKept(railItem);
    }
    const tile = closest(event, "[data-home-card]");
    if (tile && !closest(event, ".tile-rename")) {
      event.preventDefault();
      // By touch, a long press picks the tile instead (above).
      if (TOUCH) return;
      return tileMenu(tile.dataset.homeCard, { x: event.clientX, y: event.clientY });
    }
    // (A preset's, as its ⋯.)
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
      return menus.menuOf(/** @type {HTMLElement} */ (groupAt.querySelector(menus.railed() ? ".list-head-name" : "[data-group-menu]")), grid.groupRows(/** @type {Group} */ (groupAt.dataset.group)), { x: event.clientX, y: event.clientY });
    }
    const sectionRow = closest(event, "[data-section-row]");
    if (sectionRow) {
      event.preventDefault();
      return menus.menuOf(/** @type {HTMLElement} */ (sectionRow.querySelector(menus.railed() ? ".folder-item" : "[data-section-menu]")), grid.sectionRows(/** @type {string} */ (sectionRow.dataset.sectionRow)), { x: event.clientX, y: event.clientY });
    }
    const folderRow = closest(event, "[data-folder-row], [data-grid-folder]");
    if (folderRow && !closest(event, ".folder-rename")) {
      event.preventDefault();
      // (Yours, all of them: no menu of its own.)
      // (Its ⋯, out of sight on the rail: the row it's on there.)
      const more = /** @type {HTMLElement | null} */ (folderRow.querySelector("[data-folder-menu]"));
      return more ? grid.folderActions(menus.railed() && folderRow.matches("[data-folder-row]") ? /** @type {HTMLElement} */ (folderRow.querySelector(".folder-item")) : more, /** @type {string} */ (more.dataset.folderMenu), { x: event.clientX, y: event.clientY }) : undefined;
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
    // (By touch, a long press picks the row instead: see dragToReorder.)
    else if (TOUCH || !layerRow) return;
    // (On a picked row, it acts on every one picked; on another, on it alone: the picks stay as they are.)
    else if (layerPicks.has(/** @type {string} */ (layerRow.dataset.layer))) inspector.layersMenu(/** @type {HTMLElement} */ (layerRow.querySelector("[data-select]")), point);
    else {
      inspector.layerMenu(/** @type {HTMLElement} */ (layerRow.querySelector("[data-select]")), /** @type {string} */ (layerRow.dataset.layer), point);
      // (On it alone, beside picks: outlined, so it's clear which.)
      if (layerPicks.size && !$("menu").hidden) menus.outlineMenuTarget(layerRow);
    }
  });
});

// ── Adding layers ─────────────────────────────────────────────────────────
const STACKED_ONLY = `A chart holds up to ${MAX_PANES} panes, and the last is a stack: this needs a pane of its own`;
/**
 * A layer's halving cycles or years side by side, or `over` each other (periodsOf's): a layer each, colored in time
 * (the oldest indigo, the newest orange: recencyColor; over each other, this one the widest), in a pane of their own
 * on a log scale, all of history in view. Formulas, there to read and change; not at an interval they can't be at.
 * @param {string} id @param {"cycles" | "years"} which
 */
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
  // (Once they're drawn, all of history in view, the first cycle from its start: the view moved there, not its
  // range. An edit since (an undo), not.)
  const go = ++dateGoes, at = revision;
  const drawn = () => revision === at && (!settled() ? command.afterDrawn(go, drawn) : applyRange({ ...chart, range: "max" }, /** @type {Resolved} */ (resolved), true));
  command.afterDrawn(go, drawn);
  toast(over ? `Added ${labelOf(layer)}'s ${what} over each other: each over the ${periods.length - 1} before it` : `Added ${labelOf(layer)}'s ${periods.length} ${what}, each from its start`, inspector.editAdded(first));
}

// ── Series search: bitview.space's, on this machine ───────────────────────

// ── Palette: search and browse the catalog ────────────────────────────────
/**
 * Where ⌘K was left: put away (esc, its close buttons, a click beside it), or closed by a series it added or a preset
 * it opened. Opened again, it's right there, on the same page, search and row (that series or preset lit: the next a
 * ↓ away, the search selected: typed over, it starts over): on the chart it was left on, a preset's wherever the next
 * opens. Closed by anything else (a command run: no ↵ away from running it again), it starts over at the search.
 * Whenever: no clock decides it.
 */
let putAway = /** @type {any} */ (null);
const putAside = (anywhere = false) => (putAway = { chart: chart.id, home, browsing: command.palette.op.kind === "browse", anywhere, state: { path: command.palette.path.slice(), pages: command.palette.pages, root: command.palette.root, query: command.palette.query, landOn: command.palette.items[command.palette.active]?.key ?? null } });
/** Whether ⌘K is being put away (esc, its close buttons, a click beside it), not closed by what it ran. */
let dismissing = false;
/** ⌘K put away: at once (for what follows), or `leaving`, by the person (see leave). */
function dismissPalette(leaving = false) {
  dismissing = true;
  putAside();
  // (Quick Look in it put away with it, as it starts.)
  if (leaving) quickHide(), grid.leave($("palette"));
  else $("palette").hidePopover();
}
$("palette").addEventListener("cancel", () => ((dismissing = true), putAside()));
// (Closed by a series it added, or a preset it opened (for wherever the next opens): where it was kept.)
onClosed($("palette"), () => {
  if (!dismissing && !isOpen($("palette")) && (command.presetPicked() || command.seriesPicked())) putAside(command.presetPicked());
  dismissing = false;
  // (A page left as it was when closed: what it began taken back, and gone from where ⌘K comes back to. Open again by
  // then (a row that closes it and opens a page, New folder's), the pages are the new ones: see openPalette.)
  const left = isOpen($("palette")) ? -1 : command.palette.pages.findIndex((page) => page.left);
  if (left >= 0) command.leavePages(left);
});
/** Whether ⌘K was opened by Home's search, to be on it (see placeOnHome); its arrivals there, counted (see openPalette). */
let paletteOnField = false, paletteArrivals = 0;
/**
 * ⌘K kept on Home's field as Home's box, or what of the page shows, changes: where it always opens while there's no
 * room there, back on it once there is. Moved or resized, a menu it opened goes (left where it was, it'd be by
 * nothing), its path's crumbs fit to its width again.
 */
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
// (Opened or closed, said as it is, in the same frame.)
$("palette").addEventListener("beforetoggle", (event) => command.underHome(/** @type {ToggleEvent} */ (event).newState === "open"));
viewport.addEventListener("resize", keepOnHome);
viewport.addEventListener("scroll", keepOnHome);
/**
 * `add`: only series (to add to this chart, `into` a pane); `replace`: the id of a layer whose series is being
 * swapped; `formula`: a formula to add (`into` a pane) or, `editing`, a formula layer's own; `query` starts the
 * search, `path` the folder it opens on and `landOn` the row; `resume`: back where it was put away, if it can be.
 */
function openPalette({ add = false, replace = /** @type {string | null} */ (null), query = "", formula = false, editing = /** @type {string | null} */ (null), into = /** @type {string | null} */ (null), path = /** @type {string[] | null} */ (null), landOn = /** @type {string | null} */ (null), resume = false, typing = false, signal = false, side = /** @type {string | null} */ (null), bubbles = false, onField = false } = {}) {
  paletteOpenings++;
  // (Resumed only where it was browsing: adding, replacing or a formula start again.)
  const spot = resume && putAway?.browsing && (putAway.anywhere || (putAway.chart === chart.id && putAway.home === home)) ? putAway.state : null;
  putAway = null;
  countSaidFor = "";
  menus.hideMenu();
  // Opened afresh, it starts at the search (or where adding a series last added from, on this chart).
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
  // (Opened again as it leaves: it stays, put away no more.)
  if (dialog.dataset.leaving) (dismissing = false), grid.arrive(dialog);
  // (On Home, on its search, see placeOnHome; its motion from its second frame, the first the opening's.)
  if (!isOpen(dialog)) (paletteOnField = onField), command.placeOnHome(), dialog.toggleAttribute("data-arriving", dialog.hasAttribute("data-on-home") && !reduced());
  // (Its second frame; frames not running (a tab put away as it opened), a moment on all the same: never left unseen.)
  // (This arrival's alone: ⌘K closed and opened again meanwhile, the next one's its own.)
  if (dialog.hasAttribute("data-arriving")) ((arrived) => (requestAnimationFrame(() => requestAnimationFrame(arrived)), setTimeout(arrived, 100)))(((arrival) => () => arrival === paletteArrivals && dialog.removeAttribute("data-arriving"))(++paletteArrivals));
  if (!isOpen(dialog)) (input.inert = !keys), grid.showDialog(dialog), (input.inert = false), !keys && dialog.focus();
  if (typing && TOUCH) input.focus();
  // (Its series, those by block only said so, and how many there are: the catalog indexed the rest of the way first.)
  indexCatalog();
  command.applyScope();
  input.value = command.palette.query;
  command.paletteKeys(true);
  showPalette();
}
// (By touch, ⌘K itself can have the focus, not its field: no keyboard till it's tapped.)
if (TOUCH) $("palette").tabIndex = -1;
/** Whether the last thing done was by keys, not a pointer (a touch device's keyboard: ⌘K by keys, its field takes them). */
let byKeys = false;
/** Whether the last thing done was a finger's (a mouse on a touch device isn't). */
let byTouch = false;
addEventListener("keydown", (event) => ((byKeys = true), (byTouch = false), TOUCH && command.keyboardKey(event) && command.keyed(true)), true);
// (A touch heard at all, as WebKit needs to show a control pressed, :active, under a finger.)
addEventListener("touchstart", () => {}, { passive: true });
// (A press isn't held for the rest: till it's in, there's no ⌘K to tell.)
addEventListener("pointerdown", (event) => ((byKeys = false), (byTouch = event.pointerType === "touch"), command?.keyed(false)), true);
addEventListener("resize", () => isOpen($("palette")) && ((trailDrawn = ""), command.renderPath()));
$("palette-scope").addEventListener("click", () => {
  if (command.palette.path.length) {
    command.goUp(0);
    command.paletteKeys();
  } else if (command.palette.adding || command.palette.formula) command.leaveScope();
});
/** The trail as last drawn (drawn again only when it changes, or the window does). */
let trailDrawn = "";
/**
 * A trail of places ([name, depth], the last `current`), as many as fit: past that, "… ›" stands for the ones
 * dropped on the left (and goes up to the deepest of them).
 */
function drawTrail(crumbs, current) {
  const bar = $("palette-path"), key = JSON.stringify([crumbs, current, bar.hidden]);
  if (key === trailDrawn) return;
  trailDrawn = key;
  // (Hidden, nothing to fit.)
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
  // (Its list written anew (typed into, opened, a page turned): its row lit as keys light it, till the pointer moves.)
  pointedOption = false;
  const query = command.palette.query.trim();
  command.palette.pending = null;
  command.palette.controller?.abort();
  $("palette-glow").hidden = true;
  command.renderPath();
  if (command.palette.formula) return command.showFormula();
  if (command.palette.pages.length) return command.showPage();
  // In a folder, a search searches it first, at once (the crumbs say which), then every series, as the search answers:
  // one list, one ↵.
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
    // (Searching, said by the glow, as the whole search says it, till every series is in; failing, said too.)
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
    // ⌘K opens on where you were (the charts you last opened, not this one) and what to do; adding a series opens
    // on the series you last added, then the catalog.
    const lastCharts = root ? recentCharts().filter((item) => home || item.id !== chart.id).slice(0, 3).map((item) => ({ ...command.recentRow(item), group: "Recent" })) : [];
    const recents = root && browsing ? recent().map((name) => ({ kind: "series", key: name, name, group: "Recent series" })) : [];
    const suggestions = root && browsing ? SUGGESTED.filter((name) => !gone.has(name) && !recents.some((item) => item.name === name)).map((name) => ({ kind: "series", key: name, name, group: "Suggested" })) : [];
    const suggested = root ? command.commands().filter((command) => command.suggested).map(command.commandItem) : [];
    // What you pick most from here, lately: three, a step closer. (Named for what orders them: "Recently used" read as
    // "Recent" again, above it.)
    const usual = root && !browsing ? command.byUse([...command.commands().filter((command) => command.run).map((command) => ({ kind: "command", key: command.title, command })), ...command.browseRows(), ...(home ? [] : [...command.settingsNow().rows, ...chart.layers.map(command.layerRow), ...chart.panes.map((pane, i) => command.paneRow(pane.id, i))])].filter((item) => command.useScore(item.key) > 0 && !command.dangerous(item) && !/^Download/.test(item.command?.title ?? item.label ?? ""))).slice(0, 3).map((item) => ({ ...item, group: "Frequently used" })) : [];
    const fresh = (item) => !usual.some((other) => other.key === item.key);
    const others = root && command.palette.adding ? command.otherLayerRows() : [];
    // (Adding: what else a layer can be, first, a few rows in sight on any screen; then the series, what's added most,
    // then the catalog.)
    command.palette.items = browsing ? [...others, ...recents, ...suggestions, ...folder] : [...lastCharts, ...usual, ...command.browseRows().filter(fresh), ...suggested.filter(fresh)];
    // Coming back up, the row you came from stays highlighted; going in, the first.
    command.palette.active = command.palette.items.findIndex((item) => (item.kind !== "series" || command.palette.path.length > 0) && item.key === command.palette.landOn);
    // (Adding with what else a layer can be first: it opens on the first, its list at its top, the kinds in sight;
    // else on the first series not in the chart yet: ↵ adds something new.)
    if (command.palette.active < 0) command.palette.active = Math.max(0, command.palette.adding && !home && !others.length ? command.palette.items.findIndex((item) => item.kind === "series" && command.actionable(item) && !chart.layers.some((layer) => layer.series === item.name)) : 0);
    command.palette.landOn = null;
    renderPalette();
    return;
  }
  // Charts and commands match here; series in the browser's catalog index.
  const words = command.typedWords(query);
  // Adding or replacing a series, only series are found: nothing else is built, to be dropped later.
  const everything = !command.palette.adding && !command.palette.replacing;
  // A command leads only when the first word typed starts a word of its name ("download", "share"); otherwise (a word
  // of its description alone) it follows the series, which is what people search for. One of many of a kind (an
  // interval) leads only by its own name ("weekly", "1w"); by the kind ("interval"), the page that has them all, the
  // one in effect checked, does.
  const leads = (said) => ((own) => [...own.words, ...own.joins].some((word) => word.startsWith(words[0])))(command.textWords((said.follows ?? said.title).toLowerCase()));
  const found = !everything ? [] : command.rank(command.commands().map(command.commandItem), (item) => `${item.command.title} ${item.command.text ?? ""}`, words);
  // (Of those leading, a title starting with it first, "delete" Delete this chart; an interval's by its own name
  // after; one that destroys, so, last.)
  const titled = (/** @type {Command} */ command) => command.title.toLowerCase().startsWith(words[0]), ahead = found.filter((item) => leads(item.command));
  const strong = [...ahead.filter((item) => titled(item.command)), ...ahead.filter((item) => !titled(item.command) && !item.command.danger), ...ahead.filter((item) => !titled(item.command) && item.command.danger)].slice(0, 6), weak = found.filter((item) => !leads(item.command)).slice(0, 6);
  // Charts that match come before the series: a chart's title is a few words, matched, it's what's looked for.
  // (A few; more, all of them a row away, on their page, searched as here: as many as it lists, `count`.)
  const allOf = (/** @type {any[]} */ list, /** @type {string} */ group, /** @type {string} */ what, /** @type {() => Page} */ page, count = list.length, most = Infinity) => [...list.slice(0, 4), ...(count > 4 ? [{ kind: "row", key: `all:${group}`, icon: ICONS.browse, label: `All ${grouped.format(count)}${count >= most ? "+" : ""} ${what}`, verb: "Open", group, page: () => Object.assign(page(), { query }) }] : [])];
  const chartsFound = !everything ? [] : command.rank([...listedPreviews(), ...library.charts].map((doc) => ({ kind: "chart", key: doc.id, group: "Charts" })), command.rowText, words);
  const charts = allOf(chartsFound.filter((item) => home || item.key !== chart.id), "Charts", "charts found", command.chartsPage, chartsFound.length);
  // The presets (once read), after yours: a few that match; all of them on their own page.
  const matched = everything && presets ? presetsMatching(words, 60) : null;
  const presetRows = matched ? allOf(matched.found.map((i) => ({ ...presetsEdit.presetRow(i, true), group: presetsEdit.presetsGroup(matched.partial) })), presetsEdit.presetsGroup(matched.partial), "presets found", () => presetsEdit.presetsPage([]), undefined, 60) : [];
  // This chart's own: the ways in, its layers and panes, by name, series or formula; then any of their settings and
  // actions, by what they're of and what they are ("price color", "mvrv base").
  // (The chart's settings, as kept while ⌘K's open: see settingsNow.)
  const settings = everything && !home ? command.settingsNow().rows : [];
  const own = !everything ? [] : home ? command.rank(command.browseRows(), command.rowText, words) : [
    ...command.rank(command.browseRows(), command.rowText, words),
    ...command.rank(chart.layers.map((layer) => ({ ...command.layerRow(layer), group: "Layers" })), command.rowText, words),
    ...command.rank(chart.panes.map((pane, i) => ({ ...command.paneRow(pane.id, i), group: "Panes" })), command.rowText, words),
    // (A setting by a word of its own, not its owner's alone: "mvrv" is the layer's row, "mvrv color" its color; or
    // of one of its choices: "gradient", its Colors, opened on Gradient.)
    // (A slip in a choice's name finds it too: "dashde".)
    ...((choices) => ((said) => command.rank(settings.filter((row) => words.some((typed) => typed.length > 1 && (command.fitScore(row.itself, [typed]) || command.fitScore(choices(row).said, [typed])))), said, words, said))((/** @type {any} */ row) => `${command.wayText(row)} ${choices(row).said}`).slice(0, 4).map((row) => {
      // (Landed on by the words that don't name the setting nor its owner: "price line dashed", on Dashed; the
      // one they fit best: "dashed", Dashed, not Long dashes. Said at its end, where its value is: found by it,
      // "histogram" on a layer's As bubbles on…, it says why it's there.)
      const asked = words.filter((typed) => typed.length > 1 && !command.fitScore(command.wayText(row), [typed]));
      const chosen = asked.length ? choices(row).choices.map((choice) => /** @type {[any, number]} */ ([choice, command.fitScore(choice.label ?? "", asked)])).reduce((best, next) => (next[1] > best[1] ? next : best), [null, 0])[0] : null;
      return chosen?.key == null ? row : { ...row, aside: chosen.label ?? row.aside, page: () => ((next) => ((command.palette.landOn = String(chosen.key)), next))(row.page()) };
    }))(command.settingChoices()),
  ];
  // A date ("go to 2021", or just "nov 2021"): going there, on top.
  const span = everything && !home && chart.layers.length ? command.readDate(query.replace(/^go ?to\s+/i, "")) : null;
  // (A formula typed here, not a date: one that reads, of this chart's layers, all there; added as a layer, first,
  // as ⌘K's formula row adds it. A search with a dash in it stays a search.)
  const asFormula = everything && !home && !span && command.isOwnFormula(query.trim()) ? [{ kind: "formula", key: "formula", group: "Formula" }] : [];
  const dated = span && !command.outOfData(span) ? [{ kind: "row", key: "go-to-date", icon: ICONS.calendar, label: `Go to ${span.label}`, group: "Go to", verb: "Go", run: () => goToDate(span), preview: (/** @type {HTMLElement} */ panel) => command.previewDate(panel, span) }] : [];
  // (The add page's other layers: first where what's typed starts their name, "clo"; else after the series, "layers".)
  const others = command.palette.adding && !command.palette.path.length ? command.rank(command.otherLayerRows(), command.rowText, words) : [];
  const leading = others.filter((row) => row.label.toLowerCase().startsWith(words[0])), trailing = others.filter((row) => !leading.includes(row));
  const local = [...leading, ...asFormula, ...dated, ...strong, ...own, ...charts, ...presetRows, ...trailing], more = weak.map((item) => ({ ...item, group: "More commands" }));
  command.palette.moved = false;
  // What matched here, while the series are looked for: at once when bitview.space answers; on this machine with
  // the series, a frame or two on (or first, for a key on the list: see above).
  // Back from a page to this search, on the row that led there.
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
  // (This machine's search answering within a frame or two: drawn once, with its series, then; meanwhile a key on the
  // list (↵ too) draws these first and goes on with them, a click or a hover on the last query's rows only draws
  // them: nothing's done to a list no longer typed. The search still building: these now, its series after.)
  if (command.finder.ready) command.palette.pending = searching;
  else searching();
  // ("add price" is a search for price: ↵ adds it. The top result leads, or the row come back to.)
  command.searchSeries(query.replace(/^add\s+(?=\S)/i, ""), (names) => {
    if (!names) return [...local, ...more];
    // (Series named as typed, by the starts of their names' words: this chart's own rows, its three best, before them,
    // not a screen of them; unless the person's moved to a row already.)
    const typedParts = words.flatMap((typed) => typed.split("_")).filter(Boolean);
    const seriesByName = !command.palette.moved && names.some((name) => typedParts.every((typed) => name.split("_").some((part) => part.startsWith(typed))));
    // (A series typed whole, by its id as pasted from the API or by its name as listed, "thermo cap": that series
    // first, ↵ adds it; the rest where they'd be.)
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
  // (A menu by one of its rows (its actions, on Home; a held row's), gone with them: another may take its place.)
  if (menus.ownerNow()?.closest("#palette-list")) command.paletteMenuGone();
  // (Nothing found: the list alone, across.)
  $("palette").toggleAttribute("data-none", !command.palette.items.length && !pending);
  if (!command.palette.items.length) {
    list.innerHTML = tail || `<li class="note no-results">${searching ? "Nothing matches. Try other words, or describe what a series measures." : "Nothing here."}</li>`;
    $("palette-input").removeAttribute("aria-activedescendant");
    $("palette-preview").innerHTML = "";
    $("palette-look").hidden = true;
    // (Nothing highlighted: the chart as it is.)
    clearLook();
    if (quick) quickShow();
    updatePaletteKeys();
    return;
  }
  command.palette.active = Math.min(command.palette.active, Math.max(0, command.palette.items.length - 1));
  // (A list lit at its top, that row doing nothing: the first that does lit instead; one that destroys, it stays where
  // it is, unlit, for an arrow to light.)
  // (A page asking for a value keeps its row lit as it's typed: its preview stays.)
  if (!command.palette.moved && command.palette.active === 0 && !command.palette.pages.at(-1)?.input && !command.litFirst(command.palette.items[0])) command.palette.active = command.actionable(command.palette.items[0]) ? -1 : command.palette.items.findIndex(command.litFirst);
  // A new list starts at its top (set before it's written: after, it would lay the list out at once); the same list
  // drawn again (a row acted on that stays, ⌘↵, results arriving) stays where it was scrolled, as many rows drawn.
  const same = paletteShown !== null && paletteShown.page === (command.palette.pages.at(-1) ?? null) && paletteShown.query === command.palette.query && paletteShown.path === command.palette.path.join("/");
  const kept = same ? list.scrollTop : 0;
  if (!same) list.scrollTop = 0;
  paletteShown = { page: command.palette.pages.at(-1) ?? null, query: command.palette.query, path: command.palette.path.join("/") };
  // (Its first rows, as many as a page shows and more, the one highlighted among them: the rest as it's scrolled
  // or moved through.)
  command.palette.drawn = Math.min(command.palette.items.length, Math.max(command.PALETTE_PAGE, command.palette.active + command.PALETTE_PAGE / 2, same ? command.palette.drawn ?? 0 : 0));
  list.innerHTML = command.optionsHtml(0, command.palette.drawn) + (tail ? tail.replace("<li ", "<li data-tail ") : "");
  grid.forgetPaletteMarks();
  grid.seePaletteMarks(list);
  if (same) list.scrollTop = kept;
  // (A list taller than its rows, as many more as fill it: seen, laid out, on the next frame.)
  if (command.palette.drawn < command.palette.items.length) requestAnimationFrame(() => list.scrollHeight <= list.clientHeight + 400 && command.drawOptions(command.palette.drawn + command.PALETTE_PAGE));
  drawKeys(list);
  // A row landed on (coming back up) is brought into view, with its neighbors, where it's out of sight (in sight, the
  // list stays at its top: a short one's first heading cut by a scroll to its middle, for nothing).
  ((row) => row && ((box, seen) => box.bottom > seen.bottom || box.top < seen.top)(row.getBoundingClientRect(), list.getBoundingClientRect()) && row.scrollIntoView({ block: same ? "nearest" : "center" }))(command.palette.active > 0 ? $(`option-${command.palette.active}`) : null);
  command.activeOption();
  command.preview();
}
/** The list ⌘K shows (its page, its search, its place in the catalog), to tell it drawn again from a new one; none when it opens. @type {{ page: any, query: string, path: string } | null} */
let paletteShown = null;
/** The search and count ⌘K last said (see updatePaletteKeys). */
let countSaidFor = "";
/** The footer names what Enter does to the highlighted row. */
function updatePaletteKeys() {
  const item = command.palette.items[command.palette.active];
  // (A series this chart has already: another of it, said so.)
  const twin = !home && item?.kind === "series" && chart.layers.some((layer) => layer.series === item.name);
  const adds = home ? "New chart with it" : twin ? "Add again" : command.palette.into === "new" && chart.layers.length ? "Add to a new pane" : "Add to chart";
  $("palette-primary").textContent = !item ? adds : item.kind === "command" ? (item.command.run ? "Run" : "Close") : item.kind === "chart" ? "Open chart" : item.kind === "folder" ? "Open folder" : command.palette.replacing ? "Use instead" : adds;
  if (item?.kind === "insert") $("palette-primary").textContent = item.recipe ? adds : "Insert";
  if (item?.kind === "row") $("palette-primary").textContent = item.verb ?? (item.page ? "Open" : "Run");
  if (item?.kind === "formula" && command.palette.editing) $("palette-primary").textContent = "Apply";
  $("palette-more").hidden = command.actionsFor(item).length < 2;
  // (A row saying how things are, or what's still wanting, has nothing for ↵ to do: esc says that.)
  $("palette-enter").hidden = $("palette-close-sep").hidden = !item || (item.kind !== "formula" && item.kind !== "insert" && item.kind !== "folder" && item.kind !== "chart" && !command.actionable(item));
  // (How many it found, while searching: the series' search stops at its 80, then "80+"; else how many series there are,
  // where they're searched. Not on a page, nor writing a formula.)
  $("palette-count").hidden = command.palette.pages.length > 0 || command.palette.formula;
  if (!$("palette-glow").hidden) $("palette-count").textContent = "";
  else if (command.palette.query.trim()) {
    // (A row to look elsewhere (out of this folder, or to browse instead) isn't one found.)
    const found = command.palette.items.filter((row) => !row.elsewhere).length, more = command.palette.items.filter((row) => row.kind === "series").length >= 80;
    $("palette-count").textContent = found ? `${grouped.format(found)}${more ? "+" : ""} ${found === 1 && !more ? "result" : "results"}` : "No results";
    // (Said for a screen reader as the search answers: once a search, not again as the highlight moves.)
    const said = `${command.palette.query}|${$("palette-count").textContent}`;
    if (said !== countSaidFor && isOpen($("palette")) && !$("palette-count").hidden) (countSaidFor = said), say($("palette-count").textContent);
  } else $("palette-count").textContent = seriesCount ? `${grouped.format(seriesCount)} series on Bitview` : "Bitview";
}

/** The highlight, one row on (round past the ends), or several (⇧: five, stopping at the ends). */
function move(delta, scroll = true) {
  if (!command.palette.items.length) return;
  command.palette.moved = true;
  const n = command.palette.items.length;
  // (From none lit: as from before the first, ↑ from past the last.)
  command.palette.active = stepIn(command.palette.active, delta, n, Math.abs(delta) === 1);
  // (Moved by the pointer (no scroll asked), lit as a pointer lights, gliding; else by keys, at once.)
  pointedOption = !scroll;
  command.drawOptions(command.palette.active + command.PALETTE_PAGE / 2);
  $("palette-list").querySelector('[aria-selected="true"]')?.setAttribute("aria-selected", "false");
  const row = $(`option-${command.palette.active}`);
  row?.setAttribute("aria-selected", "true");
  // (The first of its group with its group's name: it says what the row is.)
  const head = row?.previousElementSibling;
  if (scroll && head?.classList.contains("group")) head.scrollIntoView({ block: "nearest" });
  if (scroll) row?.scrollIntoView({ block: "nearest" });
  command.activeOption();
  command.preview();
}
// (Out of a phone's size with ⌘K open, its pane beside the list shows: what it says, drawn.)
withRest(() => {
  phone.addEventListener("change", () => isOpen($("palette")) && !phone.matches && command.preview());
});

// ── A color looked at in ⌘K, on the chart itself ─────────────────────────

/**
 * While a row with a `look` is highlighted, the chart shows it (a doc's edit: a color), drawn from the data at hand;
 * nothing is kept until it's chosen. Moving off it, going back or closing, the chart is as it was.
 */
let looking = false;
/** The look shown, applied again to whatever the chart's drawn from next (a block's new data, a restyle). @type {((doc: Chart) => void) | null} */
let lookShown = null;
/** The chart to build: as it is, or with the look shown on it. @param {Chart} doc */
function withLook(doc) {
  if (!lookShown) return doc;
  lookShown(doc);
  // (As an edit leaves it: a cloud colored by its layers follows one's color looked at.)
  normalize(doc);
  lookDoc = doc;
  return doc;
}
function showLook(look) {
  // (Not shown (the chart's own still drawing): none, not the one before kept.)
  if (!resolved || !chartShown() || !isOpen($("palette"))) return clearLook();
  lookShown = look;
  // (At the interval of the data on screen: a switch still loading has its own; see shownIndex. A look is a
  // restyle: each series kept, its look applied.)
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
// ── ⌘K remembers what you use ────────────────────────────────────────────
let used = /** @type {Record<string, { n: number, at: number }>} */ (Object.fromEntries(Object.entries(storage.get(USED, {})).filter(([, use]) => Number.isFinite(use?.n) && Number.isFinite(use?.at))));
function noteUse(key) {
  used[key] = { n: (used[key]?.n ?? 0) + 1, at: Date.now() };
  // The sixty that weigh most are kept.
  used = Object.fromEntries(Object.entries(used).sort(([a], [b]) => command.useScore(b) - command.useScore(a)).slice(0, 60));
  storage.set(USED, used);
}

// ── ⌘K's pages: each part of the studio, a list to search and act on ─────────────
/** A bar's time, gone to: centered, among no more than AROUND bars (closer, at the zoom the chart has). @param {number} time */
function goToTime(time, go = ++dateGoes) {
  // (Drawn first: a view set while it loads is the one it lands on undone. For this chart only, the latest Go: see
  // afterDrawn.)
  if (!settled()) return command.afterDrawn(go, () => goToTime(time, go));
  const range = shownSpan(), at = resolved ? lowerBound(resolved.times, time) : -1;
  if (!range || !resolved) return;
  // (Not in what's loaded now (an interval since, the oldest let go): there by its date, loaded first.)
  if (resolved.times[at] !== time) return goToDate({ from: time, to: time + INTERVAL_DAYS[chart.index] * 86400 - 1, day: true, label: pointLabel(time, chart.index) }, 0, go);
  const half = Math.min(command.AROUND, range.to - range.from) / 2;
  autoScale();
  glide({ from: at - half, to: at + half });
}
let paletteOpenings = 0;
/** A tablet's header (as the stylesheet's .card-head on tablets): the drawing tools and Add in its ⋯. */
const tabletHead = matchMedia("(760px < width <= 1100px) and (height > 500px)");
// (One of them focused as the window comes to a tablet's size: ⋯, where they are now.)
tabletHead.addEventListener("change", () => tabletHead.matches && document.activeElement?.matches("#add, #draw-toggle") && $("chart-more").focus({ preventScroll: true }));

// ── Go to a date ──────────────────────────────────────────────────────────
/** The latest Go to a date: one waiting on a draw (see goToDate) goes on only if none came since. */
let dateGoes = 0;
/**
 * The view on a date: a year or a month shown whole, a day (or a span of a few points) centered at the zoom it has.
 * (`go`: which Go it is, kept through its waits; `by`, the draw that loaded its chunks: loaded while it's the one
 * drawn.)
 */
function goToDate(span, by = 0, go = ++dateGoes) {
  if (!span || command.outOfData(span)) return;
  // (Any other draw under way (another Go's, a block come in) would take the view back as it lands: once it has,
  // from what it drew.)
  if (!settled()) return void command.afterDrawn(go, () => goToDate(span, by, go));
  const loaded = by > 0 && by === resolved?.version;
  // Past what a fine interval has loaded: the chunks around it load first (joined to those held when they meet,
  // the most held at once kept on its side), then the view goes there.
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
    // (Loaded while this draw's the one drawn; another in its place (another interval), looked at again.)
    const mine = drawVersion;
    return void command.afterDrawn(go, () => goToDate(span, mine, go));
  }
  const { first, last } = command.pointsOf(span), range = shownSpan();
  autoScale();
  if (span.day || last - first < 8) {
    const half = range ? (range.to - range.from) / 2 : 60, center = (first + last) / 2;
    glide({ from: center - half, to: center + half });
  } else {
    // Its middle when it's wider than the chart shows (as loaded for it), else all of it.
    const times = /** @type {Resolved} */ (resolved).times, middle = command.stretchOf(span);
    // (Its first and last bars whole at the edges: the library puts bar i at (i - from) bars in, its middle, so half a
    // bar more each side shows each whole, none of the days around it.)
    glide({ from: Math.max(first, lowerBound(times, middle.from)) - 0.5, to: Math.min(last, lowerBound(times, middle.to) - 1) + 0.5 });
  }
}

$("palette-input").addEventListener("input", (event) => {
  command.palette.query = event.target.value;
  showPalette();
});
$("palette-input").addEventListener("keydown", (event) => {
  if (composing(event)) return;
  // (The list's keys as they're said (↑↓, ⇧ by five; ↵, ⇧↵, ⌘↵): another chord's the field's, ⌘↑ to its start. A
  // page left or entered by a key's press, not its repeat: ⌫ held empties the field and stops there.)
  const plain = !event.metaKey && !event.ctrlKey && !event.altKey;
  // (⌘Z, ⇧⌘Z: the chart's undo and redo, as everywhere, ⌘K open on it as it is now; while a value's written (a
  // formula, a name, a date), the writing's own.)
  if ((event.metaKey || event.ctrlKey) && !event.altKey && event.key.toLowerCase() === "z" && !command.palette.formula && !command.palette.pages.at(-1)?.input) {
    event.preventDefault();
    event.shiftKey ? redoLast() : undoLast();
    // (Its field kept, the undo having let the page's go: the next ⌘Z, or ⇧⌘Z, is ⌘K's too.)
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
// Scrolled near its end, the next rows are written.
$("palette-list").addEventListener("scroll", (event) => {
  const list = /** @type {HTMLElement} */ (event.currentTarget);
  if (list.scrollTop + list.clientHeight > list.scrollHeight - 400) command.drawOptions((command.palette.drawn ?? 0) + command.PALETTE_PAGE);
}, { passive: true });
$("palette-list").addEventListener("click", (event) => {
  const option = closest(event, "[data-option]"), more = closest(event, "[data-row-more]");
  if (!option) return;
  if (command.palette.pending) return command.palette.pending();
  // (Its ⋯: its row lit (as the arrows light it: kept as results come), its actions by it.)
  if (more) return void (move(Number(more.dataset.rowMore) - command.palette.active, false), command.showActions(more));
  command.palette.active = Number(option.dataset.option);
  command.choose(event.shiftKey);
});
$("palette-list").addEventListener("mousemove", (event) => {
  if (command.palette.pending) return command.palette.pending();
  const option = closest(event, "[data-option]");
  if (option && Number(option.dataset.option) !== command.palette.active) move(Number(option.dataset.option) - command.palette.active, false);
});
// (The foot's ↵ and Actions as their keys: rows waiting, drawn first.)
$("palette-enter").addEventListener("click", () => {
  command.palette.pending?.();
  command.choose(false);
  if (isOpen($("palette"))) command.paletteKeys();
});
$("palette-actions").addEventListener("click", () => command.showActions());
// By a finger, a row held: its other actions (there's no foot's Actions by touch); moved, it's a scroll.
// (The result held, as it was: one replaced under the finger meanwhile (a search's later answer) is let go.)
withRest(() => {
  gestures.onHold($("palette-list"), "[data-option]", (option) => ((held) => () => {
    if (command.palette.items[Number(option.dataset.option)] !== held) return false;
    command.palette.active = Number(option.dataset.option);
    if (command.actionsFor(command.palette.items[command.palette.active]).length < 2) return false;
    command.showActions(option);
    return true;
  })(command.palette.items[Number(option.dataset.option)]));
  // A preset's tile held (the gallery's, Recent's): its menu, as its ⋯ (a hold never opens it: a tap does).
  gestures.onHold($("home"), ".tile[data-preset] > [data-open-preset]", (open) => ((more) => (more ? () => (presetsEdit.openPresetMenu(more), true) : null))(/** @type {HTMLElement | null} */ (open.parentElement?.querySelector("[data-preset-menu]") ?? null)));
});
$("palette-path").addEventListener("click", (event) => {
  const crumb = closest(event, "[data-depth]");
  if (!crumb || crumb.hasAttribute("aria-current")) return;
  if (command.palette.pages.length) command.popPage(Number(crumb.dataset.depth));
  else command.goUp(Number(crumb.dataset.depth));
  command.paletteKeys();
});

// ── Menus and toolbar ─────────────────────────────────────────────────────

withRest(() => {
  menus.menuScrim.addEventListener("click", (event) => {
    event.stopPropagation();
    menus.dismissMenu();
  });
});
// A menu placed for the screen as it was (a phone's sheet, a popover at its opener) isn't carried to another: the
// screen turned, or a window made wider or narrower, it's put away. (Not as a keyboard makes it shorter.)
{
  let wide = innerWidth;
  addEventListener("resize", () => innerWidth !== wide && ((wide = innerWidth), !$("menu").hidden && menus.dismissMenu()));
}
// ⌘K on a phone, a sheet of two heights as the layers', by its handle (and its list at its top, pulled down): whole as
// it opens, and as its field's tapped; made half, the keyboard goes (the field and a row or two above it would be all
// there was).
withRest(() => {
  {
    const sheet = $("palette");
    const size = (/** @type {boolean} */ whole) => {
      sheet.toggleAttribute("data-half", !whole);
      if (!whole && document.activeElement === $("palette-input")) sheet.focus();
    };
    // (A menu open in it (a result's actions) is dragged on its own: ⌘K under it stays.)
    const menuIn = () => !$("menu").hidden && sheet.contains($("menu"));
    grid.heightsSheet(sheet, { pulls: () => phone.matches && isOpen(sheet) && !menuIn(), grab: (event) => phone.matches && isOpen(sheet) && !menuIn() && Boolean(closest(event, ".menu-handle")) && "handle", whole: () => !heldUpright.matches || !sheet.hasAttribute("data-half"), size, close: dismissPalette });
    $("palette-input").addEventListener("focus", () => sheet.hasAttribute("data-half") && grid.onceKeys(() => isOpen(sheet) && sheet.hasAttribute("data-half") && document.activeElement === $("palette-input") && grid.sizeSheet(sheet, () => size(true))));
  }
  // A phone's menu sheet, dragged down by its top (its handle), or its rows at their top: as the layers' sheet goes, it
  // goes; else it comes back.
  {
    const menu = $("menu");
    // (How far down it showed as it was taken: sliding in or away, from there; its top as it shows; its height as taken,
    // and whether it's pulled up past it.)
    let base = 0, shown = 0, height = 0, pulled = false;
    grid.dragSheet(menu, {
      pulls: () => menu.classList.contains("as-sheet") && !menu.hidden,
      grab: (event) => {
        if (!menu.classList.contains("as-sheet") || (event && !closest(event, ".menu-handle"))) return false;
        // (Taken as it goes back down from a stretch: still as stretched as it shows, the finger's from there.)
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
      // (Up past its top, it stretches, less and less, as the layers' sheet does; down, it follows the finger.)
      move: (moved) => {
        const by = moved + base;
        pulled = by < 0;
        menu.style.maxHeight = pulled ? "none" : "";
        menu.style.height = pulled ? `${height + grid.stretched(-by)}px` : "";
        menu.style.transform = pulled ? "" : `translateY(${by}px)`;
        shown = menu.getBoundingClientRect().top;
      },
      back: () => (menu.style.transition = menu.style.transform = menu.style.height = menu.style.maxHeight = ""),
      // (Let go stretched, back to its height from where it showed; else gone if it'd come to rest low enough.)
      release: (by, speed) => (pulled ? grid.sizeSheet(menu, () => {}, shown, speed) : Math.abs(by) >= 4 && grid.sheetGoes(Math.max(0, by + base), speed, menu.offsetHeight) && grid.slideAway(menu, menus.hideMenu, shown, speed)),
    });
  }
  // The shortcuts' and the guide's sheets, a phone's: dragged down by their handle, or their lines at their top, they
  // go as sheetGoes says (slid away, then closed: slideAway); else they come back.
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
  // Tab goes on from what opened the menu (on a color's Custom page, first through it: the way back, the surface,
  // its hex). In its hex, the field's own keys; ↵ or esc, done.
  const target = /** @type {HTMLElement} */ (event.target), stops = all$(".color-pane :is([data-pane-back], .plane, .hex)", $("menu"));
  const next = event.key === "Tab" && stops.includes(target) ? stops[stops.indexOf(target) + (event.shiftKey ? -1 : 1)] : null;
  if (next) return void (event.preventDefault(), next.focus());
  // (Share's, a dialog of buttons: Tab goes through them, round; esc closes it.)
  if (event.key === "Tab" && $("menu").getAttribute("role") === "dialog") {
    const buttons = all$("button:not(:disabled)", $("menu")), at = buttons.indexOf(target);
    event.preventDefault();
    return void buttons[(at + (event.shiftKey ? -1 : 1) + buttons.length) % buttons.length]?.focus();
  }
  const done = target.matches("input") && !composing(event) && (event.key === "Enter" || event.key === "Escape");
  if (event.key === "Tab" || done) {
    // (Tab, the browser's: on from the opener.)
    if (done) event.preventDefault(), event.stopPropagation();
    return menus.dismissMenu(true);
  }
  if (target.matches("input")) return;
  // A key shown beside an item does it, to what the menu is of: a layer's menu acts on that layer, selected or not.
  // (With its modifiers, as it's written: ⌥↑, ⇧↵, ⌘↵. A bare ↵ or arrow is the menu's own: the focused item, the next.)
  const glyph = { Backspace: "⌫", Enter: "↵", ArrowUp: "↑", ArrowDown: "↓", ArrowLeft: "←", ArrowRight: "→" }[event.key] ?? event.key.toUpperCase();
  const modified = event.metaKey || event.ctrlKey || event.altKey || event.shiftKey;
  const key = keyText(`${event.metaKey || event.ctrlKey ? "⌘" : ""}${event.altKey ? "⌥" : ""}${event.shiftKey && (event.key.length > 1 || /^[a-z]$/i.test(event.key)) ? "⇧" : ""}${glyph}`);
  const hinted = (modified || !["↵", "↑", "↓", "←", "→"].includes(glyph)) && all$("button:not(:disabled)", $("menu")).find((button) => button.dataset.key === key);
  if (hinted) {
    event.preventDefault();
    event.stopPropagation();
    return hinted.click();
  }
  // In ⌘K, esc closes the actions, not ⌘K: back to the search.
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
// A press away closes the menu. (Inside it by its path as the press began: what it pressed may be redrawn meanwhile, as
// a slider is at each step.)
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
/**
 * The chart's range shown (picked, or put back: see redrawChart): the view glides there, once the chunks it needs are
 * in (by the draw asked for them, not one in its place). Not drawn yet, or mid-switch, the build that lands lays its
 * view out from it.
 */
function showRange() {
  autoScale();
  if (!resolved || !chartShown()) return;
  // At a fine interval, a preset shows the latest, over the chunks it covers and those held when they join them: those
  // it lacks drawn first (mid-switch too, the new interval's), the range then laid out by the build that lands.
  const span = spanOf(chart);
  if (span) {
    const wanted = presetSpan(chart.index, chart.range);
    const from = Math.max(span.to + 1 >= wanted.from ? Math.min(span.from, wanted.from) : wanted.from, wanted.to - MOST_CHUNKS + 1);
    if (from !== span.from || !span.live) Object.assign(span, { ...wanted, from }), draw();
  }
  // (A draw on its way (its chunks', a switch's, a block's): its build lays it out, over the data it brings, not a glide
  // now over the data it replaces. Owed till then, the same range asked again too: the view laid out from none.)
  if (pending()) return void (/** @type {NonNullable<typeof displayed>} */ (displayed).doc.range = "");
  applyRange(chart, resolved, true);
  /** @type {NonNullable<typeof displayed>} */ (displayed).doc.range = chart.range;
}
// (A double-click or a double tap on the time axis: the chart's range shown again, as its button does: the view back
// to it, at the latest.)
engine.onTimeReset(() => !home && chartShown() && menus.setRange(chart.range));
$("ranges").addEventListener("click", (event) => {
  const button = closest(event, "[data-range]");
  // (One that doesn't fit says why, as its key does: a tap, a finger's only way to ask.)
  if (button?.getAttribute("aria-disabled") === "true") notice(`${button.textContent} is more than fits at ${INTERVAL_LABEL[chart.index]}`);
  else if (button) menus.setRange(/** @type {string} */ (button.dataset.range));
});
$("title").addEventListener("input", (event) => (fitTitle(), commit((doc) => (doc.title = event.target.value), { coalesce: "title" })));
// (Its font come late, measured again.)
document.fonts.addEventListener("loadingdone", fitTitle);
// (The window left (a phone's app switched), it's as it was till the window's back: see leftOpen.)
withRest(() => {
  $("title").addEventListener("blur", () => (leftWith($("title"), menus.leaveTitle) || menus.leaveTitle(), titleReads()));
});
titleReads();
phone.addEventListener("change", () => document.activeElement !== $("title") && titleReads());
// (Left with the window (a phone's app switched), back in it: what it was typed from stays, esc's to take back. See
// leftOpen: #title's own blur says so.)
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
  // The edit may have redrawn the field: leave whichever one has the focus now (for what opened the one typed in).
  const now = /** @type {HTMLElement | null} */ (document.activeElement);
  if (now && now !== field && menus.fieldOpener.has(field)) menus.fieldOpener.set(now, /** @type {HTMLElement} */ (menus.fieldOpener.get(field)));
  if (now) menus.letGo(now);
}, true);
for (const id of ["undo", "home-undo"]) $(id).addEventListener("click", () => ($(id).getAttribute("aria-disabled") === "true" ? say("Nothing to undo") : undoLast()));
for (const id of ["redo", "home-redo"]) $(id).addEventListener("click", () => ($(id).getAttribute("aria-disabled") === "true" ? say("Nothing to redo") : redoLast()));
// The panel's adds, from the header (and the phone's bar): those made from layers too, once there are layers for one.
// Adding: one page, the series, and the other kinds of layer first (in the grid, a new chart).
for (const id of ["add", "mobile-add"]) $(id).addEventListener("click", () => (home ? grid.newChart() : openPalette({ add: true })));
// ── Share, import, export ─────────────────────────────────────────────────

/** Where a device has a share sheet (a phone's), the link goes through it (copying, one of its ways); else it's copied. */
const SHEET_SHARE = TOUCH && typeof navigator.share === "function", SHARE = SHEET_SHARE ? "Share a link" : "Copy a share link";
/** Whether its share sheet takes a picture too. */
const SHARES_IMAGES = SHEET_SHARE && Boolean(navigator.canShare?.({ files: [new File([""], "chart.png", { type: "image/png" })] }));
$("share").setAttribute("aria-label", ($("share").dataset.tip = SHARE));
/** Each press of Share, numbered: one asked again (or moved on from) shows nothing of the one before. */
let shareAsk = 0;
$("share").addEventListener("click", () => {
  const button = $("share");
  // (Open: a press closes it, as a menu's button does.)
  if (menus.ownerNow() === button && !$("menu").hidden) return menus.hideMenu();
  const doc = chart, ask = ++shareAsk, shown = menus.menusShown, { copy, short } = menus.linksOf(), copying = SHEET_SHARE ? Promise.resolve(false) : menus.copyLink(copy);
  // (Anything pressed while it's made, a key or the page (Escape, elsewhere, ⌘K): moved on from it.)
  const movedOn = () => ask === shareAsk && ++shareAsk;
  for (const type of ["pointerdown", "keydown"]) addEventListener(type, movedOn, { capture: true, once: true });
  void (async () => {
    // (Its code as large as the popover takes: a phone's sheet, the screen's width.)
    const [copied, url, qr] = await Promise.all([copying, copy.catch(() => ""), short.then(menus.qrCode)]);
    for (const type of ["pointerdown", "keydown"]) removeEventListener(type, movedOn, true);
    if (!url) return notice("Couldn't make the link");
    // (Pressed again, moved on from, another chart or the grid shown, another menu opened meanwhile (closed since
    // too): nothing shown, what was done said.)
    if (ask !== shareAsk || shown !== menus.menusShown || chart !== doc || document.body.hasAttribute("data-home") || !$("menu").hidden) return void (copied && toast("Copied a link: it opens this chart for anyone"));
    const done = SHEET_SHARE ? "" : copied ? "Link copied" : "Couldn't copy the link";
    const code = qr ? (SHEET_SHARE && phone.matches ? "Scan to open it on another phone" : "Scan to open it on a phone") : "Too long for a QR code";
    say([done, code].filter(Boolean).join(". "));
    // (The chart it opens and what was done, the link; under them its code, to scan; then what else.)
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
    // (Its top first, in sight: the row taking the focus may be under the code, as on a phone on its side.)
    $("menu").scrollTop = 0;
  })();
});


/** The folder an import goes into (none: your Library's top), as asked from its menu. */
let importInto = /** @type {string | undefined} */ (undefined);
/** Charts from a file, into folder `into` (none: your Library's top). @param {string} [into] */
const importCharts = (into = undefined) => ((importInto = into), $("import-file").click());
$("import-file").addEventListener("change", async (event) => {
  const file = event.target.files?.[0], target = importInto && folderById(importInto) ? importInto : undefined;
  event.target.value = "";
  importInto = undefined;
  if (!file) return;
  // (Charts, even all of them, are a few megabytes at most: a file far past that isn't, and would hang the page read.)
  if (file.size > 32e6) return notice(`That file is ${Math.round(file.size / 1e6)} MB: too large to be charts`);
  // (Loaded, not only read: its edited presets with no place here are adopted with it, under its Undo.)
  await loadPresets().catch(() => {});
  try {
    const parsed = JSON.parse(await file.text());
    // Several, downloaded together (picked, or Download all): they join the top of your Library, shown picked on its
    // page. All of them (Download all), their folders too, made anew here, and their order: on top, arranged as they
    // were; the presets as edited there too.
    if (Array.isArray(parsed?.charts)) {
      const folders = new Map(foldersOf(parsed.folders, []).map((folder) => [folder.id, { ...folder, id: uid() }]));
      /** @type {Map<string, Chart>} */
      const byId = new Map();
      let already = 0;
      const docs = parsed.charts.flatMap((item) => {
        try {
          const doc = readChart(item);
          // (One here already isn't taken: neither is the view it came with, a backup's maybe, older than this one's.)
          if (library.charts.some((other) => sameChart(other, doc))) return (forgetView(doc.id), already++, []);
          const folder = folders.get(item.folder);
          if (folder) doc.folder = folder.id;
          if (typeof item.id === "string" && !byId.has(item.id)) byId.set(item.id, doc);
          return [doc];
        } catch {
          return [];
        }
      });
      // The presets as edited there, with them (Download all's): each as it was there, but one edited here too, or open
      // here (here's kept).
      let editsHere = 0;
      const edits = /** @type {Chart[]} */ ((Array.isArray(parsed.presets) ? parsed.presets : []).flatMap((/** @type {any} */ item) => {
        try {
          if (typeof item?.preset !== "string" || item.preset.length > 2000 || chart?.preset === item.preset) return [];
          if (library.presets.some((other) => other.preset === item.preset && presetEdited(other))) return [];
          const doc = readChart(item), carried = doc.id;
          Object.assign(doc, { id: presetIdOf(item.preset), preset: item.preset, updated: Date.now() });
          // (One with no place here, the same as one of yours already (adopted from a file imported before): here, the
          // view it carried let go of.)
          if (presets && presetIndex(doc) < 0 && [...library.charts, ...library.presets].some((other) => sameChart(other, doc))) return (forgetView(carried), editsHere++, []);
          moveView(carried, doc.id);
          if (typeof item.base === "string" && /^[a-z0-9]{1,24}$/.test(item.base)) doc.base = item.base;
          return [doc];
        } catch {
          return [];
        }
      }));
      // (In the folders made for theirs; then those with charts in them, and empty ones not here by name: none made for
      // charts left out, what was in one of those a level up.)
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
      // (One here already (a backup imported again, a folder with more in it since): that one, what's in it put in
      // yours, not made again; by its id (renamed since, the same), else by its path of names where one alone here has
      // it (two of one name, neither: a new one).)
      const namesOf = (/** @type {Folder} */ folder, /** @type {(id: string) => Folder | undefined} */ up, /** @type {number} */ most) => {
        const names = [];
        for (let at = /** @type {Folder | undefined} */ (folder), steps = 0; at && steps <= most; at = at.parent ? up(at.parent) : undefined, steps++) names.unshift(at.name);
        return JSON.stringify(names);
      };
      // (Into a folder: matched within it alone, by id or by their paths from it.)
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
      // (Nothing new, charts nor folders: said, nothing changed.)
      if (!docs.length && !edits.length && !folders.size && editsHere) return notice("You already have all of it");
      if (!docs.length && !edits.length && !folders.size && already) return toast(`${already === 1 ? "You already have that chart" : `You already have all ${grouped.format(already)} charts`}`);
      // (A file of charts none of which reads: said so, not that it's no file of charts.)
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
      // (Edited where this Studio's presets have no place: moved to the one showing the same, else kept as charts.)
      const keptAsCharts = presets ? adoptPresets(edits, false) : 0, keptIds = edits.filter((doc) => library.charts.includes(doc)).map((doc) => doc.id);
      // (The edited presets by their ids as placed (an adopted one's, new): another tab's edit replaces the object.)
      const editIds = edits.filter((doc) => library.presets.includes(doc)).map((doc) => doc.id);
      save();
      // (On the page they went to, your Library's or the folder's; picked to tell them from the charts already here
      // (into none, they're all there is).)
      goTo({ folder: target ?? "" });
      grid.picked.clear();
      if (among) for (const doc of docs) grid.picked.add(doc.id);
      drawThumbs();
      renderCharts();
      const skipped = parsed.charts.length - docs.length - already, same = asThen([...docs, ...edits]), arranged = arrangedNow();
      const what = [docs.length ? plural(docs.length, "chart") : "", folders.size ? plural(folders.size, "folder") : "", edits.length ? `${plural(edits.length, "edited preset")}${keptAsCharts ? ` (${keptAsCharts} with no place here, kept as ${keptAsCharts === 1 ? "a chart" : "charts"})` : ""}` : ""].filter(Boolean);
      return toast(`Imported ${new Intl.ListFormat("en-US").format(what)}${already ? `; ${grouped.format(already)} already here` : ""}${skipped === 1 ? "; 1 entry wasn't a chart" : skipped ? `; ${grouped.format(skipped)} entries weren't charts` : ""}`, { label: "Undo", run: () => {
        // (Any edited since: all kept, as they are. Arranged since, the order as it is.)
        if (!same()) return keptAsEdited([...docs, ...edits]);
        const unarranged = arrangedNow() === arranged;
        // (As a delete takes them, by their ids: another tab may have saved one anew since; the open one among them,
        // another opens.)
        // (Edited presets kept as charts, with them.)
        const ids = new Set([...docs.map((doc) => doc.id), ...keptIds]);
        for (const doc of library.charts.filter((doc) => ids.has(doc.id))) (discard(doc), grid.picked.delete(doc.id));
        for (const id of editIds) {
          const doc = library.presets.find((other) => other.id === id);
          if (!doc) continue;
          library.presets = library.presets.filter((other) => other !== doc);
          library.deleted[doc.id] = Math.max(Date.now(), doc.updated);
          // (Open: as made, there.)
          if (doc === chart && presetIndex(doc) >= 0) openChart(presetChart(presetIndex(doc)));
        }
        const gone = new Set([...folders.values()].map((folder) => folder.id));
        library.folders = library.folders.filter((folder) => !gone.has(folder.id));
        for (const id of gone) library.deleted[id] = Date.now();
        for (const doc of library.charts) if (doc.folder && gone.has(doc.folder)) delete doc.folder;
        for (const folder of library.folders) if (folder.parent && gone.has(folder.parent)) delete folder.parent;
        // (The order as it was, unless it's been arranged since: then as it is.)
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
    // One here already (the same chart, as a link opened again finds it): that one shown, at the file's view.
    const twin = twinOf(doc);
    if (twin) return void (showLinked(twin), toast(`Opened “${titleOf(twin)}”, which you already have`));
    // (Onto a new chart still empty, the file takes its place: an empty preview left simply goes.)
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
/**
 * A chart written into a link by hand: its JSON, URL-encoded or not, once or twice (a "%" of its own, as typed, kept;
 * encoded as a form, its spaces "+"); what came before its "{" or after its last "}" left out.
 */
function fromWritten(/** @type {string} */ raw) {
  let text = raw, parsed, failure;
  for (let round = 0; round < 3 && parsed === undefined; round++) {
    try {
      parsed = JSON.parse(text.slice(text.indexOf("{"), text.lastIndexOf("}") + 1));
    } catch (error) {
      // (The last one's: of the text as decoded as it gets, what its writer can fix.)
      failure = error;
      // (Encoded as a form, a "+" is a space, a plus "%2B": it's told by its ":" encoded too, where a link's own
      // encoding, a chat's, Studio's, keeps it, and a "+" with it. Every chart has one, after its first key.)
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
/**
 * The picture of the chart, as it shows: its title; each pane's legend where it stands; the source, the interval,
 * the view's dates and the date of the values the legends show. None where the browser would give noise for it.
 * `wait` (a script's), a heatmap's days at its size waited for first.
 * @param {{ width?: number, height?: number, scale?: number }} [size] @returns {Promise<HTMLCanvasElement>}
 */
async function chartImage(size = {}, wait = false, grown = false) {
  if (!menus.canvasReadable()) throw new Error(menus.UNREADABLE);
  const PAD = 28, FOOT = 40, title = titleOf(chart);
  // Laid out as its picture while it's read: at its size (a script's, whatever the window; else as shown), its panes
  // sized to it, its legends as the picture writes them (see presentLegends) and room made over the lines for them,
  // drawn at once (no frame waited for: a tab out of sight has none); then exactly as it was (its zoom, its size, its
  // margins, its legends).
  const element = $("chart"), wrapper = /** @type {HTMLElement} */ (element.firstElementChild), sized = Boolean(size.width && size.height);
  const owner = menus.shownAs();
  const width = sized ? /** @type {number} */ (size.width) - PAD * 2 : element.clientWidth;
  // (Its title on two lines where one doesn't hold it, its head taller only then: a picture has no hover, no tap, to
  // read the rest.)
  const titled = pictureTitle(title, width), HEAD = titled.length > 1 ? 86 : 60;
  const height = sized ? /** @type {number} */ (size.height) - HEAD - FOOT : element.clientHeight;
  if (!width || !height) throw new Error("The chart has no room: the window is of no size");
  /**
   * The chart laid out as its picture, `read` run on it at once (what it reads, given back), then put back exactly
   * as it was. @template T @param {(layout: ReturnType<typeof menus.apartLayout>) => T} read @returns {T}
   */
  const asPicture = (read) => {
    // (As it is now: a pan between two of them (a probe's and the picture's), the one put back.)
    const style = element.getAttribute("style"), wrapped = [wrapper.style.width, wrapper.style.height], logical = shownSpan(), heightWas = chartHeight, shown = presetShown, screen = engine.time.width;
    let restoreLegends = () => {};
    // (Its resizes aren't the chart's: the view held as it is, see holdView. Its tags at rest, as its legends read:
    // a crosshair on it, a pinned one's too, read nowhere (put back after).)
    exporting = true;
    const crossIndex = engine.crosshair.index, hovered = hoverTime;
    engine.crosshair.index = NaN;
    hoverTime = null;
    try {
      if (sized) Object.assign(element.style, { right: "auto", bottom: "auto", width: `${width}px`, height: `${height}px` });
      // (A tab drawing no frames: its legends still to place, the chart's, not the one before.)
      layOutLegends();
      const presented = presentLegends();
      restoreLegends = presented;
      // (The panes at its width first, its legends wrapped there: then those past two rows set apart, and the plot's
      // height less their block, at a size asked for.)
      sizeChart(width, height);
      // (Its view as the screen's: its spacing at this width, its range, again once its margins are set (an axis
      // grown over the plot moves it).)
      const view = () => {
        if (!sized || !resolved || !chartShown()) return;
        fitSpacing(chart, resolved.times.length);
        if (presetShown) applyRange(chart, resolved);
        else if (logical) showSpan(atLatest(logical, screen) ?? logical, EXACT_RANGE);
      };
      view();
      // (Read as drawn at this size: a heatmap's days by its blocks there, see viewHeat.)
      engine.render(true);
      updateReadout();
      // (Their block under the plot, out of its height at a size asked for: see chartImage for one too small.)
      const together = menus.setApart();
      restoreLegends = () => (together(), presented());
      const layout = menus.apartLayout(width);
      // (Its legends set apart, under the plot: their block's height, out of the plot's at a size asked for.)
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
      // (The chart library's own frame back to its container's size, and following it again.)
      sizeChart(element.clientWidth, element.clientHeight);
      [wrapper.style.width, wrapper.style.height] = wrapped;
      sizePanes(undefined, false, false);
      if (sized && resolved && chartShown()) fitSpacing(chart, resolved.times.length);
      // (The room after the latest point the screen's again (a picture's, at its own width, may have had another).)
      // The blank changes with width even when the profile reserve does not.
      if (sized) setReserve(), engine.invalidate(true);
      if (sized && logical) showSpan(logical, EXACT_RANGE);
      // (Its range preset showing, shown again (as a relayout would, a frame on: none between a probe and the
      // picture).)
      if (sized && shown && resolved && chartShown()) {
        applyRange(chart, resolved);
        if (!presetShown) setPresetShown(true);
      }
      marginsSet = "";
      fitMargins();
      // (Its view put back now, not as the library's next frame would: another picture right after reads it.)
      engine.render(true);
      exporting = false;
      hoverTime = hovered;
      updateReadout();
      // (A profile's bar pointed at, lit again: the picture's drew none.)
      if (profileLit) drawn.get(profileLit.layer)?.redraw();
      lastBox = chartWidth();
      // (The crosshair set here, where it was: see crossHeld.)
      if (crossSet) holdCross();
    }
  };
  /** @type {{ shot: HTMLCanvasElement, scale: number, pieces: ReturnType<typeof menus.measureLegends>, layout: ReturnType<typeof menus.apartLayout>, dates: ReturnType<typeof menus.viewDates>, axisHeight: number, leftWidth: number }} */
  let taken;
  /** @type {{ wants: NonNullable<ReturnType<typeof heatmaps.heatWants.get>>, keys: Set<string> }} */
  const extra = { wants: [], keys: new Set() };
  heatmaps.heatExtras.add(extra);
  try {
    // (Too small a size to hold its legends under a plot of 160 pixels: laid out larger by the least of 1.25, 1.25²…
    // that holds them (up to about twice: drawn down further, its words would be too small to read), its shape the
    // same, and drawn down to the size asked for (see the resampling at its end): every name whole, nothing over the
    // lines. Once: a larger one is the picture, checked as laid out. None that holds them, no picture: why.)
    const grow = !sized ? 1 : asPicture((layout) => {
      const holds = (/** @type {number} */ k) => height * k + (k - 1) * (HEAD + FOOT) - (k === 1 ? layout : menus.apartLayout(width * k + (k - 1) * PAD * 2)).height >= 160;
      let k = 1;
      while (!grown && !holds(k) && k * 1.25 <= 2) k *= 1.25;
      if (!holds(k)) throw new Error(menus.TOO_SMALL);
      return k;
    });
    if (grow > 1) return chartImage({ width: /** @type {number} */ (size.width) * grow, height: /** @type {number} */ (size.height) * grow, scale: (size.scale ?? devicePixelRatio) / grow }, wait, true);
    // (`wait`, a script's at a size not the screen's: a heatmap's days there, asked for by a draw at it (the chart
    // laid out so only while it's drawn), waited for with the page as it is, a few rounds while a draw asks for more
    // (a coarser day's columns, then their own); still missing after them, no picture: why. Not a tap's: a share
    // then would have lost the tap that allows it.)
    if (sized && wait) for (let round = 0; heatmaps.shownHeats().length; round++) {
      const asked = asPicture(() => {
        chartPicture(engine);
        Object.assign(extra, { wants: [...heatmaps.heatWants.values()].flat(), keys: new Set(heatmaps.shownHeats().flatMap((look) => [...(look.keep ?? [])])) });
        // (Those not in, nor failed: a failed one won't come.)
        return [...extra.keys].filter((key) => !heatmaps.heatDays.get(key)?.rows && !heatmaps.heatDays.get(key)?.failed);
      });
      if (!asked.length) break;
      if (round === 3) throw new Error("The heatmap's days are still loading");
      heatmaps.heatPump();
      await heatmaps.heatCome(asked, 20_000, () => menus.shownAs() === owner);
      // (The chart changed meanwhile (another opened, edited, another interval): none of it taken.)
      if (menus.shownAs() !== owner) throw new Error("The chart changed meanwhile");
    }
    // All of it read now: while the marks load, another chart can open. (The drawings as they are, none selected.)
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
      // (Its dates and its axes' size, as the shot has them.)
      return { shot, scale, pieces, layout, dates: menus.viewDates(), axisHeight: engine.timeHeight, leftWidth: engine.left };
    });
  } finally {
    heatmaps.heatExtras.delete(extra);
  }
  const { shot, scale, pieces, layout, dates, axisHeight, leftWidth } = taken;
  // (The theme's colors too, as the shot has them: all read before the marks load, while another chart, theme or
  // size may come.)
  const cadence = CADENCE[shownIndex()], [sans, cardColor, inkColor, softColor, mutedColor, lineColor] = ["--sans", "--card", "--ink", "--soft", "--muted", "--line"].map(css);
  // (The legends set apart, under the plot, in columns, read across: each its mark, its full name,
  // its value at its column's end; pane after pane.)
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
  // (A date cut by the plot's left edge fades into it, as on the page.)
  const axis = Math.round(axisHeight * scale), left = pad + Math.round(leftWidth * scale), fade = context.createLinearGradient(left, 0, left + px(14), 0);
  fade.addColorStop(0, cardColor);
  fade.addColorStop(1, alpha(cardColor, 0));
  context.fillStyle = fade;
  context.fillRect(left, head + shot.height - axis, px(14), axis);
  // (The time axis's rule, as the page draws it over the chart.)
  context.fillStyle = lineColor;
  context.fillRect(pad, head + shot.height - axis - px(1), shot.width, Math.max(1, px(1)));
  for (const piece of pieces) {
    const x = pad + piece.x, y = head + piece.y;
    context.globalAlpha = piece.faded ? 0.4 : 1;
    // (Cut at its plot's edge, as the page cuts it: see measureLegends.)
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
    // (Edged as the page edges it: a hairline of the card's color under the letters.)
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
  // The source, set like the sidebar's brand (its mark, in its orange, then its name); across from it the interval,
  // the view's dates, the values' date.
  const baseline = head + shot.height + under + px(28);
  let at = pad;
  ((mark) => {
    const path = mark?.querySelector("path"), box = mark?.viewBox.baseVal;
    if (!path || !box?.height) return;
    // (As tall as the name's capitals and a little more, its foot on their baseline.)
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
  // (Where it was made: this page's own domain, as it's served; one only this machine or its
  // network knows (a file, localhost, a private address), the data's.)
  const local = !location.hostname || location.hostname === "localhost" || /^(?:127\.|10\.|192\.168\.|172\.(?:1[6-9]|2\d|3[01])\.|\[::1\]|\[f[cd])/i.test(location.hostname) || location.hostname.endsWith(".local");
  const where = ` · ${local ? SERVER_HOST : location.host}`;
  // Too narrow for all of it: the view's dates go first, then where it was made, then the interval; the values' date
  // stays (cut, if even that's too long).
  const last = dates.values || dates.span, full = [cadence, dates.span, dates.values].filter(Boolean).join(" · "), short = [cadence, last].filter(Boolean).join(" · ");
  const room = (/** @type {boolean} */ placed) => canvas.width - pad - at - (placed ? context.measureText(where).width : 0) - px(24);
  const fits = (/** @type {string} */ text, /** @type {boolean} */ placed) => context.measureText(text).width < room(placed);
  const [said, placed] = fits(full, true) ? [full, true] : fits(short, true) ? [short, true] : fits(short, false) ? [short, false] : [fitted(context, last, room(false)), false];
  context.fillStyle = mutedColor;
  if (placed) context.fillText(where, at, baseline);
  context.textAlign = "right";
  context.fillText(said, canvas.width - pad, baseline);
  // At the scale asked for (a script's): the screen's density made that, smoothly; none asked, the screen's as it is.
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
/**
 * A picture's title as it's set (590, 20px), in `width` CSS pixels: one line, else two, broken between words (the
 * second cut with an ellipsis, if even that's too long; see fitted).
 */
function pictureTitle(/** @type {string} */ text, /** @type {number} */ width) {
  measure.font = `590 20px ${(figuresFont ||= css("--sans"))}`;
  // (Its room less what the drawn size's rounding to whole pixels can add: 20 at a scale of 4/3 is drawn at 27.)
  const words = text.split(" "), room = width * 0.97;
  if (words.length < 2 || measure.measureText(text).width <= room) return [text];
  let k = 1;
  while (k < words.length - 1 && measure.measureText(words.slice(0, k + 1).join(" ")).width <= room) k++;
  return [words.slice(0, k).join(" "), words.slice(k).join(" ")];
}
/**
 * Each pane's legend as a picture writes it, till what it returns puts them back as they were: every layer named (a
 * long legend wraps onto more lines within its pane: a picture can't be pointed at; past two, set apart under the
 * plot), a hidden layer and a hidden pane's legend left out,
 * none lit or selected.
 * @returns {() => void}
 */
function presentLegends() {
  const strips = legends().map((legend) => /** @type {[HTMLElement, string]} */ ([legend, legend.style.cssText]));
  const buttons = all$("#chart .legend button").map((button) => /** @type {[HTMLElement, string, string]} */ ([button, button.className, button.style.cssText]));
  const keys = all$("#chart .legend .legend-key").map((key) => /** @type {[HTMLElement, string]} */ ([key, key.style.cssText]));
  const hidden = hiddenPanes(chart, true), lit = profileLit;
  // (A profile's item its own, its share read: no bar pointed at, see paintProfileItems.)
  if (lit) (profileLit = null), paintProfileItems();
  // (Every layer as it is, none faded by the spotlight: lit again after.)
  const spot = spotlit;
  spotlight(null);
  for (const [legend] of strips) Object.assign(legend.style, { display: hidden.has(legend.dataset.pane ?? "") ? "none" : "flex", flexWrap: "wrap", overflow: "visible", maskImage: "none", rowGap: "2px", height: "auto" });
  // (A profile's key onto more lines where it's longer than its pane is wide.)
  for (const [key] of keys) Object.assign(key.style, { flexWrap: "wrap", flexShrink: "1", minWidth: "0" });
  for (const [button] of buttons) {
    // (Its rows wrapped, none scrolling: an item no wider than its line, its name cut short past that, never its value.)
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

// ── Tooltips: one floating label for anything with data-tip, its shortcut as keys ──

const tip = $("tip");
let tipTarget = null, tipTimer = 0, tipClosed = 0;
/** What the pointer's over: in the open sidebar, an item's icon has its tip, its name shown whole doesn't. */
let tipPointed = /** @type {Element | null} */ (null);
/** The item a shown tip describes (as its description, to a screen reader, where its label doesn't say it already). */
let described = /** @type {HTMLElement | null} */ (null);
function tipWanted(/** @type {HTMLElement} */ target) {
  // (On the rail, a row its menu names as the pointer rests on it: no tip before it. Reached by a key, its tip.)
  if (menus?.railed() && tipPointed && target.matches("#chart-list :is(.folder-item, .list-head-name, [data-open-chart], [data-open-preset])")) return false;
  // (A tip of two lines, a row's name and its second line: repeating only while both are shown whole.)
  const name = target.querySelector(".name, .label, .head-name, .tile-name"), sub = target.querySelector(".sub"), [head, ...more] = (target.dataset.tip ?? "").split("\n");
  const whole = (/** @type {Element | null} */ words) => Boolean(words) && /** @type {Element} */ (words).scrollWidth <= /** @type {Element} */ (words).clientWidth;
  // (Its second line's words as its tip has them: a layer row's letter, heading it, not among them.)
  const subText = sub ? [...sub.childNodes].filter((node) => !(node instanceof Element && node.classList.contains("sub-letter"))).map((node) => node.textContent).join("").trim() : "";
  const repeats = whole(name) && name?.textContent?.trim() === head && (!more.length || (whole(sub) && subText === more.join("\n")));
  const keys = Boolean(target.dataset.kbd && answers(target.dataset.kbd));
  return !repeats || (target.closest(".sidebar") ? menus?.railed() || Boolean(tipPointed && target.contains(tipPointed) && tipPointed.closest("[data-icon], .chart-mark, .head-mark")) : keys);
}
function showTip(target) {
  if (!target.isConnected || (menus?.ownerNow() === target && !$("menu").hidden)) return;
  // A tip names what only an icon or a value shows, or gives its keys: words shown whole, the same, nothing more
  // to say. The open sidebar names its items: there, an item's icon pointed at says it (as the rail does), its
  // name pointed at doesn't, unless it's cut short.
  if (!tipWanted(target)) return;
  const keys = target.dataset.kbd && answers(target.dataset.kbd) ? keycaps(target.dataset.kbd) : "";
  // Inside a dialog, the tip must live in the dialog (in the top layer) to be seen.
  const host = target.closest("dialog") ?? document.body;
  if (tip.parentElement !== host) host.append(tip);
  // (A tip of several lines: its first, what it is; the others, quieter, more about it.)
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
  // (Set only when it changes: set again, the page's style is looked at again. Going, it fades.)
  if (!tip.hidden) (fadeOut(tip, [{ opacity: 1 }, { opacity: 0 }]), (tipClosed = performance.now()), (tip.hidden = true));
  tipTarget = null;
  described?.removeAttribute("aria-describedby");
  described = null;
}
/** Whether the focus is moved by the keys that move it (Tab, the arrows): only then does a control focused say its tip, not where the page puts the focus (back on a menu's button, on a dropdown's choice). */
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
  // (None by touch, nor while a press drags: the navigator's, a slider's.)
  if (event.pointerType === "touch" || event.buttons) return;
  const target = closest(event, "[data-tip]");
  tipPointed = /** @type {Element} */ (event.target);
  // (The same one, its tip shown or coming: from its icon to its name, gone where it's not wanted there; onto its
  // icon from its name, its tip, if it hadn't one.)
  if (target === tipTarget && (!tip.hidden || tipTimer)) return void (target && !tipWanted(target) && hideTip());
  hideTip();
  if (!target?.dataset.tip) return;
  tipTarget = target;
  // Once one tip has shown, neighbors show at once.
  const warm = performance.now() - tipClosed < 500;
  tip.classList.toggle("warm", warm);
  tipTimer = setTimeout(() => ((tipTimer = 0), showTip(target)), warm ? 0 : 550);
});
document.addEventListener("pointerout", (event) => !event.relatedTarget && hideTip());
for (const type of ["pointerdown", "keydown", "wheel", "scroll"]) document.addEventListener(type, hideTip, { capture: true, passive: true });

// ── Toast and status ──────────────────────────────────────────────────────

/** A toast that tells, as a refusal or a failure: nothing was done, so no check. */
function notice(text, action) {
  toast(text, action, false);
}
/** An Undo asked for once another change came after its own: told how to step back to it. */
const steppedPast = () => notice(`Things have changed since then. Press ${keyText("⌘Z")} to step back`);
/** A toast of a chart's last step (the open one's by default), whose Undo takes it back while it's still that chart's last. */
function undoToast(text, doc = chart) {
  // (The step just done; nothing changed, none.)
  const step = ((last) => (last?.at === stepCount ? last : null))(stepsOf(doc)?.past.at(-1));
  // (Its chart still the one here (not a preset let go of and made anew since), its step still its last.)
  toast(text, { label: "Undo", of: doc, run: () => (step && (doc === chart || chartById(doc.id) === doc) && stepsOf(doc)?.past.at(-1) === step ? travelIn(doc, true) : steppedPast()) });
}
let toastTimer = 0;
/**
 * Undo and Redo, each pressable or not (Redo beside it always: an undo, even long after, taken back at a glance;
 * nothing moves under a second press); a preview's Keep marked once it's edited (the edits go with it, if kept).
 */
function syncUndo() {
  // (Dimmed, not disabled: a key's focus on one stays there as it runs out. The chart's and the Charts page's alike:
  // what ⌘Z and ⇧⌘Z take where they show.)
  const back = canTravel(true), forth = canTravel(false);
  for (const id of ["undo", "home-undo"]) $(id).setAttribute("aria-disabled", String(!back));
  for (const id of ["redo", "home-redo"]) $(id).setAttribute("aria-disabled", String(!forth));
  const edited = isPreview(chart) && touched.has(chart), keep = $("keep");
  keep.classList.toggle("edited", edited);
  keep.dataset.tip = edited ? "Save this chart and your edits to your Library" : "Save this chart to your Library";
  keep.setAttribute("aria-label", edited ? "Save, edited" : "Save");
}
/** Whether a path is being placed (its first point placed). */
const placingPath = () => tool === "path" && Boolean(draft && placing);
/**
 * Whether ⌘Z (`back`) or ⇧⌘Z has a step to take, as undoLast and redoLast take them: a path's point being placed
 * (taken back, put back), a history's (see historyFor).
 */
function canTravel(/** @type {boolean} */ back) {
  if (placingPath() && draft) return back ? draft.points.length > 1 : pathTaken.length > 0;
  return historyFor(back) !== undefined || (!back && !home && Boolean(pathResume && tool === "path"));
}
/**
 * A step of the Library's own (a chart or a folder deleted, moved, copied; one a toast offered to undo): what it
 * said, how it's undone (giving how it's done again, if it can be), and when it was done (see ChartStep).
 * (`folder`: the folder it made, whose first name is its own: see nameFolder. An undone one's `open`, the chart open
 * before its undo, and `left`, the one open after it: see redoLibrary.)
 * @typedef {{ text: string, run: () => (void | false | (() => void)), undone?: string, at: number, folder?: string, open?: Chart, left?: string }} LibraryStep
 */
/**
 * The Library's steps, kept however long ago they were (the toast that said one gone or not): ⌘Z and ⇧⌘Z take them
 * wherever they're the newest in reach (see historyFor). Fifty back.
 */
const libraryEdits = { past: /** @type {LibraryStep[]} */ ([]), future: /** @type {LibraryStep[]} */ ([]) };
/** A Library step being done again (its Redo): the steps after it, still to redo, stay; being undone: it makes none. */
let redoing = false, undoing = false;
/**
 * For an undo that lets charts go (a copy, a duplicate, an import): whether each is still as it was then (edited
 * since, here or in another tab, it isn't; edited and undone back, it is: its content and interval, not its stamp;
 * its range a view).
 * @param {Chart[]} docs @returns {() => boolean}
 */
const asThen = (docs) => ((contents) => () => docs.every((doc, i) => ((now) => !now || contentOf({ ...now, range: "" }) === contents[i])(chartById(doc.id))))(docs.map((doc) => contentOf({ ...doc, range: "" })));
/**
 * Folders made again under a new id (a delete undone, a folder made again: its deletion stands where it's been
 * heard): the old id → the new, so the steps that knew the old one find it.
 * @type {Map<string, string>}
 */
const folderAgain = new Map();
/** A folder's id as it is now (made again since, the new one's). */
const folderNow = (/** @type {string} */ id) => {
  for (let hops = 0; folderAgain.has(id) && hops < 50; hops++) id = /** @type {string} */ (folderAgain.get(id));
  return id;
};
/** Charts given a new id since, their old one let go of (a link's preview kept, its id deleted here; an edited preset moved): the old → the new. @type {Map<string, string>} */
const chartAgain = new Map();
const chartNow = (/** @type {string} */ id) => {
  for (let hops = 0; chartAgain.has(id) && hops < 50; hops++) id = /** @type {string} */ (chartAgain.get(id));
  return id;
};
/** Sidebar keys as they are now (a folder made again, a chart given a new id: their new ones). */
const keysNow = (/** @type {string[]} */ keys) => keys.map((key) => (key[0] === "f" ? `f:${folderNow(key.slice(2))}` : `c:${chartNow(key.slice(2))}`));
/** Its refusal: the charts it'd let go of edited since, kept. @param {Chart[]} docs */
const keptAsEdited = (docs) => notice(docs.length === 1 ? `“${titleOf(docs[0])}” has been edited since then, so it was left unchanged` : "They've been edited since then, so they were left unchanged");
/** A Library step kept (see libraryEdits): said by a toast's Undo, or by nothing (a rename, a reorder). @returns {LibraryStep | null} */
function libraryStep(/** @type {string} */ text, /** @type {LibraryStep["run"]} */ run, /** @type {string | undefined} */ undone = undefined) {
  if (undoing) return null;
  const step = { text, run, undone, at: ++stepCount };
  libraryEdits.past.push(step);
  if (libraryEdits.past.length > 50) libraryEdits.past.shift();
  // (Something new done: nothing in reach to redo, see recordStep.)
  if (!redoing) dropRedos();
  syncUndo();
  return step;
}
/** The Library's last step undone: said, with its Redo where it can be done again. */
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
  // (Nothing done (what it'd undo changed since, gone, back already): not said as done; its own word, if it had one.)
  if (saves === saved) return void (syncUndo(), toasts === said && notice(`Nothing to undo: things have changed since then (${step.text})`));
  if (typeof again !== "function") return void (syncUndo(), toast(step.undone ?? "Undone"));
  const back = { text: step.text, run: again, undone: step.undone, at: ++stepCount, open: was, left: chart.id };
  libraryEdits.future.push(back);
  // (Its Redo, while it's still the next to redo.)
  toast(step.undone ?? "Undone", { label: "Redo", run: () => (libraryEdits.future.at(-1) === back ? redoLibrary() : notice("Nothing to redo: things have changed since then")) });
}
/**
 * The Library's last step undone, done again (it says so itself, its Undo a step again). Redone where its undo left
 * (nothing opened since), the chart open before that undo is open again, whatever either opened (a duplicate's copy,
 * a deleted chart's neighbor, a preset's copy): its own steps in reach as they were then. Another opened since, it's
 * left open: the redo doesn't take you back.
 */
function redoLibrary() {
  const step = libraryEdits.future.pop();
  if (!step) return notice("Nothing to redo");
  // (Its "Undone" put away: what it does says itself, or nothing.)
  hideToast();
  const saved = saves, said = toasts, from = chart;
  redoing = true;
  try {
    step.run();
  } finally {
    redoing = false;
  }
  // (Only once it did something: a refused redo moves nothing, nor the chart open.)
  // (From the Charts page, none: the chart under it isn't the one being looked at, nor swapped unseen.)
  if (step.open && saves !== saved && !home && from.id === step.left && chart.id !== step.open.id) {
    const doc = step.open, preset = doc.preset !== undefined ? presetIndex(doc) : -1;
    // (Let go of as it was left, as it came (see openChart): a draft one again, a preset made anew; its steps, kept by
    // its id, with it. Deleted since (another tab), none: the one the redo opened stays.)
    if (!chartById(doc.id) && drafts.has(doc)) addPreview(doc);
    const now = chartById(doc.id) ?? (preset >= 0 ? presetChart(preset) : undefined);
    if (now?.id === doc.id) {
      // (The redo opened one itself: this one in its place in the tab's history, Back never to a chart not seen.)
      if (chart !== from && history.state?.chart === chart.id) history.replaceState({ ...history.state, chart: now.id }, "");
      openChart(now);
    }
  }
  syncUndo();
  if (saves === saved && toasts === said) notice(`Nothing to redo: things have changed since then (${step.text})`);
}
/** The chart whose edit the toast up says (its Undo that chart's: put away as its steps move, or as it's left). */
let toastOf = /** @type {Chart | null} */ (null);
/**
 * Something done, with its check; or (`done` false) something to know, without. An Undo said with a Library step
 * (not `of` a chart, a chart's own step's: see undoToast) keeps that step (see libraryEdits): the toast's button undoes it
 * while it's still the last, as ⌘Z does (see historyFor); one whose `run` gives back how to do it again is followed
 * by a Redo (`undone` saying it): a folder or a chart deleted, moved, copied, back and gone again as a chart's edits are.
 * @param {{ label: string, run: () => (void | (() => void)), undone?: string, of?: Chart, folder?: string, step?: LibraryStep | null }} [action]
 */
function toast(text, action, done = true) {
  toasts++;
  unsay();
  // (Another said: not the closed preview's any more.)
  closedToast = null;
  const element = $("toast");
  const host = topDialog() ?? document.body;
  // (Moved, its copy fading where it was gone first: its id is found where it is, never in a copy before it.)
  if (element.parentElement !== host) (cutFade(element), host.append(element));
  // Said where it shows, by the status line that's always there (one moved in, or made, is often not read).
  const said = /** @type {HTMLElement | null} */ (host.querySelector(":scope > [data-said]"));
  if (said) {
    said.textContent = "";
    cancelAnimationFrame(sayingFrame);
    sayingFrame = requestAnimationFrame(() => ((saidOn = said), (said.textContent = toastSaid = action ? `${text}. ${action.label}${action.label === "Undo" ? `, ${keyText("⌘Z")}` : action.label === "Redo" ? `, ${keyText("⇧⌘Z")}` : ""}` : text)));
  }
  const undoes = action?.label === "Undo";
  // (A Library step: kept, as long as the Library's are.)
  // (Or its step done already, said late: see deleteChart.)
  const step = undoes && action && !action.of ? ("step" in action ? action.step ?? null : libraryStep(text, action.run, action.undone)) : null;
  if (step && action?.folder) step.folder = action.folder;
  // (Said with an edit of the open chart, it's that edit's step; else, a deleted chart's, a copied link's, no step's.)
  toastOf = action?.of ?? (chartChanged ? chart : null);
  // (No ✓ where there's an Undo: a removal, a move, isn't a success to tick; ✓ elsewhere says what's chosen.)
  const redoes = action?.label === "Redo";
  // (One said as another shows: that one fading out as this one comes, each as wide as its words, never one cut to
  // the other.)
  const replacing = element.classList.contains("visible") && !reduced();
  if (replacing) fadeOut(element, [{ opacity: 1 }, { opacity: 0 }]);
  else cutFade(element);
  element.innerHTML = `${done && !undoes && !redoes ? `<i aria-hidden="true">${ICONS.check}</i>` : ""}<span>${escapeHtml(text)}</span>${action ? `<button type="button" class="btn small" ${undoes ? 'data-kbd="⌘ Z"' : redoes ? 'data-kbd="⇧ ⌘ Z"' : ""}>${escapeHtml(action.label)}</button>` : ""}`;
  element.querySelector("button")?.addEventListener("click", () => {
    element.classList.remove("visible");
    // (A Library step's Undo: it, while it's the last of them.)
    if (step) return libraryEdits.past.at(-1) === step ? undoLibrary() : steppedPast();
    action?.run();
  });
  syncUndo();
  placeToast(host);
  element.classList.add("visible");
  if (replacing) element.animate({ opacity: [0, 1] }, { duration: MOTION.micro, easing: MOTION.easeLayout });
  // Up long enough to read (longer for more words, an Undo or a notice the longest), and while it's pointed at or
  // focused: then its time starts again.
  toastFor = Math.max(action ? 6000 : done ? 2600 : 4500, 1500 + text.length * 55);
  hideToastLater();
}
/**
 * Where the toast shows: above the chart's foot; in the grid, above its bar (or its foot), centered on it; the
 * layers' bar showing on a phone's sheet, above it; above the keyboard. Again as the screen changes while it's up (a
 * turn, the keyboard): measured once, it stayed where the old layout had it, off the new one.
 */
function placeToast(/** @type {Element} */ host = /** @type {HTMLElement} */ ($("toast").parentElement)) {
  const element = $("toast");
  const layersBar = $("layers-bar"), sheetBar = document.body.dataset.sheet === "panel" && !layersBar.hidden ? layersBar.getBoundingClientRect() : null;
  const frame = sheetBar ?? (home ? $("home") : $("stage-foot")).getBoundingClientRect();
  const belt = $("belt"), beltTop = !home && !belt.hidden && belt.dataset.dock === "bottom" ? belt.getBoundingClientRect().top : Infinity;
  const tabs = /** @type {HTMLElement} */ (document.querySelector(".mobile-bar")).getBoundingClientRect(), tabsTop = tabs.width > tabs.height ? tabs.top : Infinity;
  // (A phone's menu sheet up: above it, not over its rows; where it settles, sliding in as it may be.)
  const menuSheet = $("menu").classList.contains("as-sheet") && !$("menu").hidden ? $("menu").offsetTop : Infinity;
  // (In the page's own measure: iOS's window is as short as its keyboard leaves, the page isn't. The chart's foot
  // hidden under the keyboard, above the keyboard.)
  const page = document.documentElement.clientHeight, shown = viewport.offsetTop + viewport.height, keys = shown < page - 1;
  const above = Math.min(shown, menuSheet, sheetBar ? sheetBar.top : Math.min(tabsTop, frame.height ? (!home ? Math.min(frame.top, beltTop) : $("home-bar").hidden ? frame.bottom - 8 : $("home-bar").getBoundingClientRect().top) : Infinity));
  element.style.bottom = host === document.body && (frame.height || keys) ? `${page - above + 12}px` : "";
  const center = host === document.body && frame.width ? frame.left + frame.width / 2 : 0;
  element.style.left = center ? `${center}px` : "";
  // (Centered off the window's middle, as wide as fits on both sides of it.)
  element.style.maxWidth = center ? `${2 * Math.min(center, innerWidth - center) - 24}px` : "";
}
let toastFor = 0;
// (Placed again while it shows: the screen changed (a turn, the keyboard), a sheet or a phone's menu sheet come up
// under it or gone; again where a sheet's motion leaves it.)
const replaceToast = () => {
  if (!$("toast").classList.contains("visible")) return;
  placeToast();
  const moving = [$("panel"), $("menu")].flatMap((sheet) => sheet.getAnimations());
  if (moving.length) Promise.allSettled(moving.map((motion) => motion.finished)).then(() => $("toast").classList.contains("visible") && placeToast());
};
for (const target of [window, viewport]) target.addEventListener("resize", replaceToast);
new MutationObserver(replaceToast).observe(document.body, { attributes: true, attributeFilter: ["data-sheet", "data-sheet-size"] });
new MutationObserver(replaceToast).observe($("menu"), { attributes: true, attributeFilter: ["hidden", "class"] });
/** The toast up put away at once: what it said no longer true (an undo, a redo, a keep). */
function hideToast() {
  closedToast = null;
  clearTimeout(toastTimer);
  $("toast").classList.remove("visible");
  unsay();
}
/** What the last toast put on a status line, and that line (the page's, or a dialog's, the toast moved out since). */
let toastSaid = "", saidOn = /** @type {HTMLElement | null} */ (null);
/** The frame a toast's words wait for (another toast, or the toast gone, before it: not said). */
let sayingFrame = 0;
/** The toast's words off the status line it said them on: gone with it (words said there since, kept); not yet said, never. */
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

/**
 * The guide's examples in its lines, set as code: a script's call, a chart's JSON (a list of it too), a link's start,
 * a formula's function called (a block's, as it is).
 */
const CODE_IN = new RegExp(String.raw`(?:await )?studio\.\w+\((?:[^()<]|\([^()<]*\))*\)|\[(?:\{(?:[^{}<]|\{[^{}<]*\})*\}(?:, )?)+\]|\{(?:[^{}<]|\{[^{}<]*\})*\}|https:\/\/bitview\.space\/studio\/#[a-z]+=|#(?:chart|image|c)=|\b(?:${Object.keys(FUNCTIONS).join("|")})\([^()<]*\)`, "g");
const codeIn = (/** @type {string} */ html) => html.split(/(<pre[\s\S]*?<\/pre>)/).map((part, i) => (i % 2 ? part : part.replace(CODE_IN, (match) => `<code>${match}</code>`))).join("");
/**
 * The page's links guide: for people, a few words and this chart's link at hand, and its reference in short; the
 * rest of the reference, for scripts and assistants, folded under it (llms.txt has all of it: see seo.mjs). The
 * dialog opens on itself.
 */
function showGuide() {
  const open = !home && chart.layers.length > 0;
  // (The reference from its first section, "In short", to the next, then the rest: its title and its line for a
  // page without JavaScript left out.)
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
/** The shortcuts sheet opens on itself, not on a ringed close button. */
function showShortcuts() {
  keyGroupsNow();
  renderSingleKeys();
  grid.arrive($("shortcuts"));
  grid.showDialog($("shortcuts"));
  $("shortcuts").focus();
}
/** A key a hint says the page answers to: any, but a single one (a letter, a digit, a sign) once those are off. */
const answers = (/** @type {string} */ kbd) => prefs.keys || NAMED_KEY.test(kbd);
/** Single-key shortcuts on or off: kept, said where they're set. */
function setSingleKeys(/** @type {boolean} */ on) {
  prefs.keys = on;
  savePrefs();
  renderSingleKeys();
  toast(on ? "Single-key shortcuts on" : `Single-key shortcuts off: only shortcuts with ${keyText("⌘")} or ${keyText("⌥")} work`);
}
function renderSingleKeys() {
  for (const button of all$("[data-single-keys]")) button.setAttribute("aria-pressed", String((button.dataset.singleKeys === "on") === prefs.keys));
  $("single-keys-text").textContent = `Shortcuts that are a single letter, digit or symbol. When off, only shortcuts with ${keyText("⌘")} or ${keyText("⌥")} work`;
  // (Off, a single key's cap goes wherever it's shown: it would promise what it doesn't do.)
  document.body.toggleAttribute("data-single-off", !prefs.keys);
}
renderSingleKeys();
$("shortcuts").addEventListener("click", (event) => {
  const button = closest(event, "[data-single-keys]");
  if (button && (button.dataset.singleKeys === "on") !== prefs.keys) setSingleKeys(button.dataset.singleKeys === "on");
});

/** The block the server's series are computed up to, when it was indexed (unix seconds), and the live price, as last seen. */
let liveHeight = 0, liveAt = 0, livePrice = 0;
/** When bitview.space last answered (its health): a minute without, or the device itself offline, and the live element says it's offline. */
let answeredAt = Date.now();
const offline = () => !navigator.onLine || Date.now() - answeredAt > 60_000;
/** Live, with the last block the server's series include and when it came. */
/** Nothing drawn for want of the server: what's wrong said as the live dot says it, this device offline or the server away. */
// (Written only as it changes: it's a live region, and the same words written again at each price are said again.)
const sayUnreached = () => ((text) => $("unreached-text").textContent !== text && ($("unreached-text").textContent = text))(navigator.onLine ? `Couldn't reach ${SERVER_HOST}` : "This device is offline");
/** A live figure (the last block, the price, when it was last heard): its mark before it, a screen reader's word for it. @param {string} icon @param {string} name @param {string} text */
const liveFigure = (icon, name, text) => `<span class="live-figure"><span role="img" aria-label="${name}">${icon}</span>${escapeHtml(text)}</span>`;
/** Whether a screen reader was last told it's offline, or live (none yet: connecting; see updateStatus). */
let saidLost = /** @type {boolean | null} */ (null);
function updateStatus() {
  sayUnreached();
  const at = liveAt ? new Date(liveAt * 1000) : null;
  // The block, then the price once it's known (till then, when the block came: the price comes in its place, nothing
  // before it moving); how long ago the block came in the tip.
  const lost = offline();
  // (Nothing heard from the server yet, and not given up on: still connecting, as the page began (back online
  // after "Offline", too).)
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
  // (Live said only when it isn't. Home's apart: the chain at its left, the price (or when it was last heard) at its
  // right; each said in full by its tip, as the live dot's.)
  $("home-pulse").replaceChildren(...[[[said.status, said.block].filter(Boolean).join(" "), block], [said.price, lost ? "Last heard" : "Price, as it trades"]].map(([html, tip]) => {
    const half = document.createElement("span");
    half.innerHTML = html;
    if (tip) half.dataset.tip = tip;
    return half;
  }));
  liveTip(lost ? "offline" : "live", state, ...(lost ? [block, "Trying again every 15 seconds"] : [livePrice ? `Price ${dollarsAndCents(livePrice)}, as it trades` : "", block, "A new block shows within seconds"]));
}
/**
 * The live state on the logo (its dot the badge, the logo the target, a finger's or a pointer's): its name Bitview
 * Studio and the state in a word, its tip that name and the state's lines (where from, the price, the last block):
 * the one open, said again. @param {string} state @param {...string} lines
 */
function liveTip(state, ...lines) {
  const brand = $("brand"), said = ["Bitview Studio", ...lines].filter(Boolean).join("\n");
  if (brand.dataset.tip === said) return;
  brand.dataset.tip = said;
  brand.setAttribute("aria-label", `Bitview Studio, ${state}`);
  if (tipTarget === brand && !$("tip").hidden) showTip(brand);
}

// ── Rendering the page ────────────────────────────────────────────────────

/** Shows the open chart as it is now, the chart itself drawn again as little as that takes (see redrawChart); `refit`, its view laid out anew, as it opens. */
function refresh(refit = false) {
  // (What waits for a draw asks again: the grid shown, layers gone, it says so at once.)
  for (const wake of [...drawnWaiters]) wake();
  clearLink();
  // (Focused but not typed in, it follows the chart too: another tab's rename, taken in.)
  if (document.activeElement !== $("title") || $("title").value === typedFrom.get($("title"))) typedFrom.set($("title"), ($("title").value = chart.title));
  renderChartPath();
  // (Untitled, what it's called meanwhile: in the field, the one it'd be named.)
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
  // A new chart, empty, can be left with Escape (or Cancel), for the chart before it: taken back.
  const back = chart.layers.length ? null : ((i) => (i >= 0 ? presetDoc(i) : previousChart()))(previousPreset());
  $("leave-empty").hidden = !(isPreview(chart) || emptyYours(chart));
  if (backToHome()) $("leave-empty").dataset.tip = `Back to ${homeName()}`;
  else $("leave-empty").dataset.tip = back ? `Back to ${titleOf(back)}` : "Back to Home";
  // An empty chart is the empty state's to fill: no panel beside it, nothing to export or share yet.
  document.body.toggleAttribute("data-empty", !chart.layers.length);
  $("share").disabled = $("draw-toggle").disabled = $("panel-toggle").disabled = !chart.layers.length;
  if (!chart.layers.length && document.body.dataset.sheet === "panel") delete document.body.dataset.sheet;
  $("stage-foot").hidden = !chart.layers.length;
  // (Placed once the chart's laid out: see placePlotKeys. Drawn under the page, it's placed once the page has gone.)
  if (home || !chart.layers.length) $("plot-keys").hidden = true;
  else if ($("plot-keys").hidden) relayout();
  withRest(refreshPanels);
  updateStatus();
  redrawChart(refit);
}
/** What shows beside the chart (the lists, the layers, the inspector, the belt) and its mini chart, as it is now: once the rest is in, as a first chart's drawn. */
function refreshPanels() {
  renderCharts();
  renderLayers();
  inspector.renderInspector();
  drawThumbs();
  renderBelt();
}
/**
 * The open chart drawn again as it is now, as little as that takes. Built from all it was built from, or asked of a
 * draw still on its way (a title, a name, a range, a drawing, a pane's size changed: see lookKey), its legends, panes
 * and drawings only; its data the same, restyled on it (`now`, a drag's: at once; else see buildSoon); else resolved
 * anew. Under the Charts page, one not on screen, none on its way for it, waits till it's opened (see leaveHome's undrawn).
 */
function redrawChart(refit = false, now = false) {
  const asked = pending() ? asking.look : displayed?.look;
  if (home && !chartShown() && !pending()) undrawn = true;
  if ((home && undrawn) || (!refit && asked === lookKey(chart))) {
    updateLegends();
    if (chartShown()) sizePanes();
    // (Under the Charts page, its drawings as they were, nothing painted unseen: painted as it's left, see leaveHome.)
    if (!home) showDrawings(chart);
  } else if (builds()) now ? build(withLook(structuredClone(chart)), /** @type {Resolved} */ (resolved), refit) : buildSoon(refit);
  else draw(refit);
  // (Its range another since its view was laid out from one (an undo, another tab's): shown, as a pick of it is.)
  if (chartShown() && chart.range !== displayed?.doc.range) showRange();
}
/** Whether the open chart can be drawn again from the data at hand (its data the same, none other on its way). */
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
/** The build asked for (see buildSoon), now: from the data at hand if it's still the chart's, else drawn anew (its id changed meanwhile, say). */
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
// (A tab put away draws no frames: what was asked, built as it goes.)
document.addEventListener("visibilitychange", () => document.hidden && buildNow());

/**
 * A row's menu on the rail (there, it's its icon: no ⋯, no name), its name over it: a chart's, a preset's, as its ⋯
 * (pressed, a chart opens: its menu shows as the pointer rests on it); a folder's, a section's of the presets, a
 * group's, its page and its opening or closing here (your Library's, what can be made in it), then a folder's
 * charts (at any depth, one in a folder in it that folder named beside it). As the pointer rests on it, or pressed
 * (a container; a finger's way).
 */
function railMenu(/** @type {HTMLElement} */ item, hover = false) {
  const folder = item.dataset.folderItem ? folderById(item.dataset.folderItem) : null, key = item.dataset.sectionItem, group = /** @type {Group | undefined} */ (item.dataset.groupItem);
  const doc = item.dataset.openChart ? chartById(item.dataset.openChart) : undefined, preset = item.dataset.openPreset;
  if (doc) return menus.menuOf(item, command.chartActions(doc.id, "row"), null, "right", { hover });
  if (preset !== undefined) return menus.menuOf(item, presetsEdit.presetActions(Number(preset), "row"), null, "right", { hover });
  if (group) return menus.menuOf(item, grid.groupRows(group), null, "right", { hover });
  if (key) return menus.menuOf(item, sectionFlyout(key), null, "right", { hover });
  if (folder) menus.menuOf(item, folderFlyout(folder.id), null, "right", { hover });
}
/** A rail row's menu kept open, the focus in it: shown already (pointed at), kept as it is; else shown, as a press does. */
function railMenuKept(/** @type {HTMLElement} */ item) {
  if ($("menu").hidden || menus.ownerNow() !== item) return railMenu(item);
  menus.pinFlyout();
  menus.focusMenu();
}
/**
 * The name of what a menu's for, as its menu says it where it's out of sight (see menuOf): by what opened it (its
 * item, its ⋯, a right-click on it): a sidebar row's (a chart, a preset whole, a folder, a section, a group), a
 * tile's, a folder's on the Charts page, a layer's, a pane's, the open chart's; else none.
 * @param {HTMLElement} anchor
 */
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
/**
 * A folder's menu on the rail (its folders never open there), however it's opened (pointed at, pressed,
 * right-clicked): what's in it, in its order, a chart opened by a click, a folder's own beside it (a level at a
 * time, each the same); then its page, and its own actions one row away (More: as its ⋯ has them, opening and
 * closing it aside, not there), what's in it the menu's own.
 * @param {string} id @returns {Action[]}
 */
function folderFlyout(id) {
  const counts = grid.chartCounts(), own = () => grid.folderRows(id, false, true).filter((row) => row.key !== "toggle" && row.key !== "view");
  return [
    // (The open chart, and the folder it's in at each level, marked: its way down, as the rail's icon is.)
    ...grid.childrenOf(id).map((each) => ("layers" in each
      ? { key: each.id, icon: grid.chartMark(each), label: titleOf(each), marked: !home && chart.id === each.id, run: () => grid.go(chartById(each.id)) }
      : { key: `f:${each.id}`, icon: folderMark(each), label: each.name, aside: grid.shortCount(counts.get(each.id) ?? 0), marked: !home && Boolean(chart.folder && grid.isIn(chart.folder, each.id)), page: () => ({ title: each.name, heading: true, rows: () => folderFlyout(each.id) }) })),
    // (Empty, said so: not its actions under its name as if what's in it were missing.)
    ...(grid.childrenOf(id).length ? [] : [{ key: "empty", label: "No charts yet", disabled: "Move charts here, or make one in it (New chart inside)" }]),
    ...grid.folderRows(id, false, true).filter((row) => row.key === "view").map((row) => ({ ...row, apart: true })),
    { key: "more", icon: ICONS.more, label: "More", page: () => ({ title: folderById(id)?.name ?? "", heading: true, rows: own }) },
  ];
}
/** A section of the presets' menu on the rail, as a folder's (see folderFlyout): its presets, its sections; its page. @param {string} key @returns {Action[]} */
function sectionFlyout(key) {
  const node = presetNodeAt(JSON.parse(key)), all = /** @type {Preset[]} */ (presets);
  if (!node) return [];
  // (The open preset, and the section it's in at each level, marked, as a folder's are.)
  const open = !home && chart.preset !== undefined ? presetIndex(chart) : -1, holding = open < 0 ? [] : grid.sectionKeys(open);
  // (Their marks drawn as they show, as a page of them has them.)
  for (const each of node.items) if (typeof each === "number" && !presetMarks.has(grid.presetMarkKey(each))) grid.presetMarksAsked.add(each);
  drawPresetMarks();
  return [
    ...node.items.map((each) => (typeof each === "number"
      // (By its own name, as its row: retitled here, by that.)
      ? { key: `p:${each}`, icon: grid.presetMark(each), marked: each === open, label: ((doc) => (doc && titleOf(doc) !== all[each].t ? titleOf(doc) : presetName(all[each])))(keptPreset(each)), run: () => pickPreset(each) }
      : { key: each.key, icon: sectionMark(each), label: each.name, aside: grid.shortCount(each.under.length), marked: holding.includes(each.key), page: () => ({ title: each.name, heading: true, rows: () => sectionFlyout(each.key) }) })),
    { key: "view", icon: ICONS.arrowRight, label: "View", aside: "Its page", apart: true, run: () => goTo(asPlace({ path: JSON.parse(key) })) },
  ];
}

// ── Global clicks and keys ────────────────────────────────────────────────

document.addEventListener("click", (event) => {
  // (A preset's ⋯, its row's in the sidebar, its tile's.)
  const presetMenu = closest(event, "[data-preset-menu]");
  if (presetMenu) return presetsEdit.openPresetMenu(presetMenu);
  const menuFor = closest(event, "[data-chart-menu], [data-scale-menu], [data-pane-menu], [data-folder-menu]");
  const { chartMenu: chartId, paneMenu: paneId, folderMenu: folderId, scaleMenu: scaleId } = menuFor?.dataset ?? {};
  if (menuFor) return chartId ? grid.chartMenu(menuFor, chartId) : paneId ? presetsEdit.paneActions(menuFor, paneId) : folderId ? grid.folderActions(menuFor, folderId) : presetsEdit.scaleMenu(menuFor, /** @type {string} */ (scaleId));
  // A folder in the sidebar: each click opens or closes it there (one under way turns back from where it is: clicks
  // as fast as they come); renamed from its ⋯. A section of the presets and a group's head the same. On the rail, its
  // menu (see railMenu).
  // (A row's →: its page.)
  const page = closest(event, "#chart-list .chart-go");
  if (page) return goTo(asPlace(JSON.parse(/** @type {string} */ (page.dataset.place))));
  const folderItem = closest(event, "[data-folder-item]");
  if (folderItem) return menus.railed() ? railMenu(folderItem) : grid.toggleFolder(/** @type {string} */ (folderItem.dataset.folderItem));
  const sectionMenu = closest(event, "[data-section-menu]");
  if (sectionMenu) return menus.menuOf(sectionMenu, grid.sectionRows(/** @type {string} */ (sectionMenu.dataset.sectionMenu)));
  const groupItem = closest(event, "[data-group-item]");
  if (groupItem) return menus.railed() ? railMenu(groupItem) : grid.toggleGroup(/** @type {Group} */ (groupItem.dataset.groupItem));
  const groupMenu = closest(event, "[data-group-menu]");
  if (groupMenu) return menus.menuOf(groupMenu, grid.groupRows(/** @type {Group} */ (groupMenu.dataset.groupMenu)));
  const sectionItem = closest(event, "[data-section-item]");
  if (sectionItem) return menus.railed() ? railMenu(sectionItem) : grid.toggleSection(/** @type {string} */ (sectionItem.dataset.sectionItem));
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
  // (A legend item's mark is its switch only under a mouse, where its eye shows as it's pointed at: tapped, the item.)
  const target = byTouch && found?.matches(".legend [data-toggle]") ? /** @type {HTMLElement | null} */ (found.closest("[data-select]")) : found;
  if (!target) return;
  const { select: id, toggle, paneMode, paneStack, paneReverse, paneOutline, paneGradient, gradient, mode, action, sheet, openChart } = target.dataset;
  if (openChart) {
    delete document.body.dataset.sheet;
    grid.go(chartById(openChart));
  }
  // ⌘ or ⇧, or a click while some are picked (not a key: ↵ opens it): picked with the others; else opened. (↵ on
  // one picked among several: they're opened together, as the bar's Edit does.)
  else if (id && (event.metaKey || event.ctrlKey || event.shiftKey || (layerPicks.size && event.detail > 0))) pickLayer(id, event.shiftKey);
  else if (id && event.detail === 0 && layerPicks.size > 1 && layerPicks.has(id) && target.closest("#layer-list")) layersBatch("edit");
  else if (id) select(id);
  else if (toggle) event.altKey ? inspector.soloLayer(toggle) : inspector.toggleHidden(toggle);
  else if (paneMode) target.getAttribute("aria-disabled") === "true" ? notice(/** @type {string} */ (target.dataset.tip)) : presetsEdit.setPaneMode(paneMode, Number(mode));
  else if (paneStack) target.getAttribute("aria-disabled") === "true" ? notice(/** @type {string} */ (target.dataset.tip)) : presetsEdit.setPaneStack(paneStack, target.dataset.stack ?? "");
  else if (paneReverse) presetsEdit.setPaneReverse(paneReverse, Boolean(target.dataset.reverse));
  else if (paneOutline) presetsEdit.setPaneOutline(paneOutline, Boolean(target.dataset.outline));
  else if (paneGradient) presetsEdit.setPaneGradient(paneGradient, gradient ?? "");
  // (The Search tab opens ⌘K, not the keyboard: that comes with a tap in its field, see openPalette.)
  else if (action === "palette") openPalette({ resume: true });
  // (Home's search, drawn as a field: ⌘K afresh, its keyboard with it by touch, as a tap on a field asks.)
  else if (action === "search") openPalette({ typing: true, onField: target.matches(".home-search") });
  else if (action === "new-chart") grid.newChart();
  else if (action === "close-preview") closePreview(chartById(target.dataset.preview ?? "") ?? chart);
  else if (action === "add-series") openPalette({ add: true, typing: target.hasAttribute("data-typing") });
  else if (action === "leave-empty") leaveEmpty();
  else if (action === "import") importCharts(target.dataset.into);
  else if (action === "download-all") menus.downloadAll();
  // (Home, from anywhere, a chart too: a page of its own, not the one a chart was opened from (its path, Back, go
  // there). The sidebar's.)
  else if (action === "charts") goTo({ top: true });
  // (The phone's Home tab, as iOS's tabs are: from a chart, Home where it was left (its page, scrolled as it was);
  // on a page of Home's, up to its top; at its top, back to the top of it.)
  else if (action === "home-tab") {
    if (home) goTo({ top: true });
    // (As many of its cards as it showed, as Back brings it: past a reload too.)
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
// A click beside a dialog (on its scrim, see showDialog) puts it away; a toast shown inside it moves back to the page.
withRest(() => {
  for (const dialog of all$("dialog")) {
    const scrim = Object.assign(document.createElement("div"), { className: "dialog-scrim", popover: "manual" });
    scrim.setAttribute("aria-hidden", "true");
    scrim.addEventListener("click", () => (dialog.id === "palette" ? dismissPalette(true) : grid.leave(dialog)));
    dialog.before(scrim);
    grid.dialogScrims.set(dialog, scrim);
    // (Closing: off the stack, its scrim gone with it, and the focus back where it was, as a modal's: there and then,
    // before what follows moves it on.)
    dialog.addEventListener("beforetoggle", (event) => {
      if (/** @type {ToggleEvent} */ (event).newState !== "closed") return;
      const to = grid.dialogOpeners.get(dialog), at = document.activeElement;
      if (dialogsShown.includes(dialog)) dialogsShown.splice(dialogsShown.indexOf(dialog), 1);
      holdPage(dialog, false);
      if (isOpen(scrim)) scrim.hidePopover();
      if (to instanceof HTMLElement && to.isConnected && (!at || at === document.body || dialog.contains(at))) to.focus({ preventScroll: true });
    });
    // (esc: as it leaves, where the browser wouldn't play it: see leave. Unless something of it took the key: Quick Look.)
    dialog.addEventListener("cancel", (event) => {
      if (keepsLeaving || event.defaultPrevented) return;
      event.preventDefault();
      grid.leave(dialog);
    });
    onClosed(dialog, () => $("toast").parentElement === dialog && (cutFade($("toast")), (topDialog() ?? document.body).append($("toast"))));
    // (Closed: no series being added or replaced any more. Open again by then (a row that closes it and opens it another
    // way: Bubbles, adding), its way the new one.)
    if (dialog.id === "palette") onClosed(dialog, () => !isOpen(dialog) && command.palette.op.kind !== "formula" && (command.palette.op = { kind: "browse" }));
  }
});

/**
 * Out of the field being typed into: what it holds kept (or, a text emptied, gone), and a formula left by a press
 * applied, there and then: on the chart it was typed on (another's opening).
 */
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
  /** The page's height with no keyboard, at each width it's had (a screen turned with the keyboard up, the other's). */
  const tallest = /** @type {Map<number, number>} */ (new Map());
  /** How much of the page's foot the keyboard covers, as --keys says (none, as the stylesheet starts it). */
  let covered = 0;
  /** The field typed into, scrolled into what shows (in the list that holds it, under its head), its sheet sized first. */
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
      // (Taller than what shows, its top.)
      const by = box.bottom > bottom ? Math.min(box.bottom - bottom, box.top - top) : Math.min(0, box.top - top);
      if (Math.abs(by) > 1) list.scrollBy({ top: by, behavior: reduced() ? "auto" : "smooth" });
    });
  };
  const read = () => {
    const layout = root.clientHeight, most = Math.max(tallest.get(innerWidth) ?? 0, layout);
    tallest.set(innerWidth, most);
    const under = Math.max(0, Math.round(layout - viewport.height));
    // (By touch: a window made shorter by a mouse isn't a keyboard.)
    // (Covering by what's under it; else by the page made shorter: a pixel's rounding there is none.)
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
  // (Moved past a tap's move, it's none, back where it began or not: a sheet's pull, a scroll.)
  addEventListener("touchmove", (event) => press && Math.hypot(event.touches[0].clientX - press.x, event.touches[0].clientY - press.y) > SLOP && (press = null), { capture: true, passive: true });
  addEventListener(
    "touchend",
    (event) => {
      const target = /** @type {HTMLElement} */ (event.target), was = press, touch = event.changedTouches[0];
      press = null;
      // (Its own, or its label's: a hex's box around it.)
      const field = /** @type {HTMLElement | null | undefined} */ (typing(target) ? target : target.closest("label")?.control);
      // (A tap: not a scroll, nor a press held for the caret's magnifier or a paste.)
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
  // (A key composing a character, an IME's (⌃K among them), is the composition's: none here.)
  if (composing(event)) return;
  const mod = event.metaKey || event.ctrlKey;
  const key = event.key.toLowerCase();
  // (A modifier pressed alone (⇧, ⌘ held for the keys' hints): nothing pressed yet; a menu shown stays.)
  if (menus.flyout && event.getModifierState(event.key)) return;
  if (menus.flyout) {
    // (↑ ↓ into it only from nowhere, or from what opened it: from a field, a list, they're theirs, and it goes.)
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
  // An open menu has the keys (its own, and its arrows): the chart's don't act behind it. Esc closes it.
  if (!$("menu").hidden && event.key !== "Escape") return;
  // A search's button with the focus (an empty chart's "Search N series", the phone's Search): what's typed there
  // searches, as in its field, rather than acting as a shortcut.
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
  // Everything saves as you go: ⌘S just says so, instead of the browser saving the page; a preview, it keeps it.
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
  // A chart focused in the sidebar or the grid: ⌘D and ⌫ are its own (in the grid, the picks' too), not the open chart's.
  const inList = typing(event.target) || !document.activeElement?.matches?.(":focus-visible") ? null : /** @type {HTMLElement | null} */ (document.activeElement.closest("#chart-list, #home-grid"));
  if (inList && !event.altKey && ((mod && !event.shiftKey && key === "d") || (!mod && erasing(event)))) {
    event.preventDefault();
    const item = /** @type {HTMLElement | null} */ (document.activeElement?.closest?.("[data-chart-row], [data-home-card]") ?? null);
    const id = item?.dataset.chartRow ?? item?.dataset.homeCard ?? "", doc = chartById(id), inGrid = inList.id === "home-grid";
    if (!doc || isPreview(doc)) return;
    // (Once a press: held, the key repeats, and each copy's picks would be copied again for as long as it's held.)
    if (key === "d") return void (event.repeat || (inGrid ? grid.batch("duplicate", null, grid.picked.has(id) ? null : [id]) : grid.duplicateCharts([doc])));
    // (One of the picks: they all go; one alone, alone. An Undo brings them back.)
    return inGrid ? grid.batch("delete", null, grid.picked.has(id) ? null : [id]) : deleteChart(doc);
  }
  // A layer's row: ⌥↑ ⌥↓ move it among its pane's, as a chart's row moves in the list.
  const layerRow = typing(event.target) ? null : /** @type {HTMLElement | null} */ (document.activeElement?.closest?.(".layer-row[data-layer]") ?? null);
  if (layerRow && event.altKey && !mod && upDown(event)) {
    event.preventDefault();
    const id = /** @type {string} */ (layerRow.dataset.layer);
    inspector.moveLayer(id, event.key === "ArrowUp" ? -1 : 1);
    return void presetsEdit.refocus(`.layer-row[data-layer="${CSS.escape(id)}"] .row-main`);
  }
  // Drawings selected: the arrows move them (⇧ further), ⌘ the arrows resize them; not from a list or the panel.
  if (chosen.size && !home && !event.altKey && ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(event.key) && !typing(event.target) && !document.activeElement?.closest?.("#chart-list, #panel, #belt")) {
    event.preventDefault();
    return heatmaps.nudgeDrawings(event.key, event.shiftKey, mod, event.repeat);
  }
  // (⌘D alone: ⌘⇧D, ⌘⌥D are no one's here.)
  if (mod && !event.shiftKey && !event.altKey && key === "d" && !home && (chosen.size || layerPicks.size || selected) && !typing(event.target)) {
    event.preventDefault();
    // (Once a press, as a chart's above.)
    if (event.repeat) return;
    return chosen.size ? heatmaps.duplicateDrawings([...chosen]) : layerPicks.size ? layersBatch("duplicate") : inspector.duplicateLayer(selected);
  }
  // ⌘A picks every layer while some are picked, or from the panel (on the chart, every drawing).
  if (mod && key === "a" && !home && chart.layers.length && !typing(event.target) && (layerPicks.size || $("layers-view").contains(document.activeElement))) {
    event.preventDefault();
    return pickEvery();
  }
  if (mod && key === "a" && !home && chart.drawings?.length && !typing(event.target)) {
    event.preventDefault();
    return chooseDrawings(chart.drawings.map((drawing) => drawing.id));
  }
  if (mod || event.altKey || typing(event.target)) return;
  // Space or X on a layer's row picks it, as on a chart's card in the grid (a row's own key: whatever the setting).
  const row = /** @type {HTMLElement | null} */ (document.activeElement?.matches?.(".layer-row .row-main") ? document.activeElement.closest(".layer-row[data-layer]") : null);
  if ((event.key === " " || key === "x") && row && !event.shiftKey && !home) {
    event.preventDefault();
    return pickLayer(/** @type {string} */ (row.dataset.layer));
  }
  // (Turned off, a letter, a digit or a sign alone does nothing here: only named keys, ⌫, esc, the arrows.)
  if (!prefs.keys && event.key.length === 1) return;
  if (grid.yoursShown() && erasing(event) && grid.picked.size) {
    event.preventDefault();
    return grid.batch("delete");
  }
  // In the grid, only what isn't of a chart: the open one is out of sight.
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
    // (On the Charts page, the chart under it, out of sight, is left as it is: its tool, measure, picks.)
    if (!home && $("menu").hidden && clearMeasure()) return;
    if (home && grid.picked.size && $("menu").hidden) grid.clearPicks();
    else if (!$("menu").hidden) menus.dismissMenu(true);
    // (A path being placed, kept as it is: esc finishes it.)
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
    // A new chart left empty: Escape takes it back, as its Cancel button does.
    else if (!home && !chart.layers.length && (isPreview(chart) || emptyYours(chart))) leaveEmpty();
    // (Full screen's panel, opened for the while (P), put away first; nothing else to put away: out of full screen,
    // as anywhere (a press, not its repeat: one held unwinds the rest, not this too).)
    else if (!home && fullOn() && panelShown()) togglePanel(false);
    else if (fullOn() && !event.repeat) presetsEdit.setFull(false);
  } else if (upDown(event) && event.shiftKey) extendPicks(event.key === "ArrowDown" ? 1 : -1);
  else if (upDown(event)) selectBy(event.key === "ArrowDown" ? 1 : -1);
  // (A path being placed: its last point taken back.)
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
  // (A signal's marks stepped through, as a player's frames: before, after the middle of the view.)
  else if ((event.key === "," || event.key === ".") && chart.layers.length) chart.layers.some((layer) => signalShown(layer)) ? command.stepSignal(event.key === "." ? 1 : -1) : notice("No signal shown to step through");
  else return;
  event.preventDefault();
});

// ── Keys on screen: hold ⌘ and every visible control with a ⌘ key shows it ──

const HOLD = MAC ? "Meta" : "Control";
// The page's own keys as this platform writes them (the shortcuts sheet, "Hold ⌘", the search's ⌘K).
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
    // Controls that carry their key show it in place (on the rail, in place of their icon); the rest float one.
    if (element.querySelector(":scope > kbd.mod")) continue;
    // (Out of reach (inert: the Charts page under a chart, kept laid out), none either.)
    if (/** @type {HTMLButtonElement} */ (element).disabled || element.closest("[inert]") || !element.checkVisibility({ checkVisibilityCSS: true })) continue;
    const rect = element.getBoundingClientRect();
    if (!rect.width || rect.bottom < 0 || rect.top > innerHeight) continue;
    const badge = document.createElement("kbd");
    badge.className = "key-hint";
    badge.textContent = keysOf(element.dataset.kbd);
    layer.append(badge);
    const width = badge.offsetWidth, height = badge.offsetHeight;
    // Just below the control, or above it near the bottom of the window.
    const below = rect.bottom + 6 + height <= innerHeight - 4;
    badges.push({ badge, width, x: rect.left + rect.width / 2, top: below ? rect.bottom + 6 : rect.top - 6 - height });
  }
  // Each badge centered under its control; neighbours that would touch part evenly, 4px apart.
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
  // Once the keys are up, ⇧ keeps them (a ⌘⇧ key is among them); before, it's a chord like any other key.
  if (event.key === "Shift" && hinting()) return;
  // Any other key is a chord: the keys don't come up. Once up, they follow what the key changed (⌘1, ⌘2…).
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

// ── Live: each new block refreshes the chart's series ─────────────────────
// The server's health says which block its series are computed up to: a light, uncached read, every 15 seconds
// while the page is seen and at once when it's seen again. A new block refetches the chart, and the dot flashes.

$("live").addEventListener("animationend", (event) => event.animationName === "block" && $("live").classList.remove("block"));
let checking = false;
async function checkLive() {
  if (document.visibilityState !== "visible" || checking) return;
  // (The rest lost as it loaded, a chart drawn: asked again, as the device is back and each 15 seconds; till it's in,
  // said as offline (the server unanswered, as it was). In, this check is its own: it retries the rest's plots.)
  if (!command) return void loadRest().then(checkLive, () => ((answeredAt = 0), updateStatus()));
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
  // (A refresh that failed, its series' older points held: asked again at once too, not at the next block. A draw
  // for it asks the tails afresh.)
  const behind = idle && !retry && refreshFailed(chart);
  // (Each answer: back from offline, and how long ago the last block came, said anew.)
  updateStatus();
  const height = health.computedHeight;
  if (!Number.isInteger(height) || height === liveHeight) return void (retry ? draw() : behind && draw(false, true));
  const first = !liveHeight;
  liveHeight = height;
  liveAt = Number.isFinite(health.lastIndexedAtUnix) ? health.lastIndexedAtUnix : 0;
  updateStatus();
  // (The next halving, estimated from it: its row's words, and its line, once known.)
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
  // (Under the Charts page, the chart isn't drawn again for it: once it's back, see leaveHome.)
  if (chart && drawSeries(chart).length) home ? (blockAtHome = true) : draw(false, true);
}
setInterval(checkLive, 15_000);
// (The device's network gone: said at once, not a minute on; back, asked at once. Nothing said before the first
// answer: it's still connecting.)
addEventListener("offline", () => (sayUnreached(), !$("live").classList.contains("waiting") && updateStatus()));
addEventListener("online", () => (sayUnreached(), checkLive()));
// (Hidden, nothing is asked: back in view, the minute starts over.)
document.addEventListener("visibilitychange", () => document.visibilityState === "visible" && (answeredAt = Date.now()));
document.addEventListener("visibilitychange", checkLive);
withRest(checkLive);

// ── Live price: between blocks, the price's own layers follow it ──────────
// bitview.space's live price (round-dollar outputs of the last blocks and the mempool), read each second while the
// page is seen, as its own chart does: the live element says it, and each layer drawing the price itself (its candles, its close or its spot,
// in dollars, or sats per dollar; no stack) has its point for the interval the clock is in moved there, a
// candle's high or low pushed out. Between blocks, the server's latest interval (its latest block's) can be over:
// a live bar then opens for the current one, from the last close. A formula of the price waits for
// the next block, whose data takes over.

/** The price's series, each with what the live price (in dollars) is in its unit. */
const usd = (/** @type {number} */ price) => price, sats = (/** @type {number} */ price) => 1e8 / price;
/** (Named by the whole client: see whole.) @type {Map<string, (price: number) => number>} */
let LIVE = new Map();
/** The open chart's layers the live price moves: none on the grid, mid-draw, or with history alone loaded. */
function liveLayers() {
  if (home || !settled() || (FINE.has(chart.index) && !spans.get(spanKey(chart.id, chart.index))?.live)) return [];
  // Averaged measurements, like formula averages, update when a block supplies their source data.
  return chart.layers.filter((layer) => LIVE.has(layer.series ?? "") && !layer.hidden && layer.type !== "histogram" && ((pane) => !drawnInto(chart, pane) && !averageFor(chart, pane).active)(chart.panes.find((pane) => pane.id === layer.pane)));
}
/** Vectors the live price has its own copies of (written in place from then on: a tick copies nothing). */
const ownLive = new WeakSet();
/** A vector the live price can write: its own copy, made once. @template {Float64Array | any[]} T @param {T} vector @returns {T} */
const liveCopy = (vector) => {
  if (ownLive.has(vector)) return vector;
  const copy = /** @type {T} */ (vector.slice());
  ownLive.add(copy);
  return copy;
};
/** What the live price was last laid on (the data, the price, the layers): a poll bringing the same does nothing. */
let liveLast = "";
/** Data with intervals added at its end, empty for now: every layer's, as copies (the data drawn stays as it was). */
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
/**
 * The live price onto those layers' point for the interval the clock is in, laid on the timeline if the server
 * hasn't it yet (the intervals between left as gaps; far behind, more than a hundred, it waits for the server): on
 * the chart, and in the legends and changes reading it. Into copies (a formula naming the layer reads the same
 * values, and waits for the next block).
 */
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
    // (A heatmap's bars too, at the price now: its newest column drawn there, see viewHeat.)
    for (const series of heatmaps.heatSeries) for (let k = 0; k < added.length; k++) series.place(resolved.times.length - added.length + k, price);
  }
  const data = /** @type {Resolved} */ (resolved), at = data.times.length - 1;
  // The same price on the same data, for the same layers (as most polls bring): nothing to move.
  const key = `${idOf(data)}:${price}:${layers.map((layer) => layer.id).join()}`;
  if (key === liveLast) return;
  liveLast = key;
  dataRevision++;
  let moved = false, regrade = false;
  /** The painted layers whose latest point's color is found again (see paintPoint). @type {string[]} */
  const toPaint = [];
  for (const layer of layers) {
    const result = data.results.get(layer.id), value = /** @type {(price: number) => number} */ (LIVE.get(layer.series ?? ""))(price);
    if (!result?.values) continue;
    // A live bar opens at the last close (at an interval the timeline's just grown by, or one it has that this layer
    // hasn't yet: another layer's series has it already; far behind, it waits for the server); a point the server
    // has (or one opened before) moves.
    const opens = added.length > 0 || !Number.isFinite(result.values[at]);
    const last = opens ? lastIn(result.values, 0, at) : at;
    if (last < 0 || at - last > 100 || !Number.isFinite(result.values[last])) continue;
    const values = liveCopy(result.values), series = drawn.get(layer.id), own = series?.engine instanceof Candles ? series.engine : null;
    // (Its gradient's range of all its history moved by it, either way: every point's color another, see below.)
    const was = layer.byGradient && !layer.upToDate ? rangeOf(result.values).join() : "";
    values[at] = value;
    rescan(values, at);
    if (was && rangeOf(values).join() !== was) regrade = true;
    // Its candles move too, drawn as candles or not (a switch to candles shows them as they are).
    const candles = result.opens && Number.isFinite(result.opens[last]) ? { opens: liveCopy(result.opens), highs: liveCopy(/** @type {Vector} */ (result.highs)), lows: liveCopy(/** @type {Vector} */ (result.lows)) } : null;
    if (candles) {
      // (A bar opening, at the last close; else its own open, its wicks reaching the price.)
      const open = opens ? values[last] : candles.opens[at];
      candles.opens[at] = open;
      candles.highs[at] = Math.max(opens ? open : candles.highs[at], value);
      candles.lows[at] = Math.min(opens ? open : candles.lows[at], value);
      // (Its candle set in place on the library's columns (Studio's own, made at the build); a bar added, all again.)
      if (own && own.close.length > at) own.set(at, open, candles.highs[at], candles.lows[at], value), engine.invalidate();
      else if (own) own.setValues(candleColumns({ ...result, values, ...candles }, values, modeOf(displayed?.doc ?? chart, layer) === 1, (/** @type {number} */ v) => v)), engine.invalidate(true);
    }
    // (Bubbles' own below, with their places; a heatmap's closes above; a cloud's at the next build.)
    data.results.set(layer.id, { ...result, values, ...candles });
    // (What's drawn is read too: a drawing snaps to the price as it is now.)
    const made = built.get(layer.id);
    if (made && !made.stacked) built.set(layer.id, { ...made, shown: values, ...candles });
    if (series instanceof Standard && !own) liveColumn(series, values, at, value);
    // (Painted date by date: its point once every live value is in, from the colors as they are then.)
    if (paintedLayers.has(layer.id)) toPaint.push(layer.id);
    moved = true;
  }
  // (And each layer taking its colors from one that moved, its latest point (a value, a cloud's band), where it has
  // one; each as the chart is drawn, a preview's while one shows.)
  const doc = displayed?.doc ?? chart, inDoc = (/** @type {string | undefined} */ id) => doc.layers.find((item) => item.id === id);
  const movedIds = new Set(layers.map((layer) => layer.id)), painting = new Set(toPaint);
  // (Bubbles on a layer that moved, or moving themselves: their latest where it is now, at their size now (a live
  // value written as a point's, none of theirs, put right); the dates opened meanwhile, empty.)
  for (const layer of doc.layers) {
    const series = drawn.get(layer.id);
    if (layer.type !== "bubbles" || !series || !(movedIds.has(layer.id) || movedIds.has(layer.ref ?? ""))) continue;
    const on = inDoc(layer.ref), y = on ? /** @type {Vector | undefined} */ (built.get(on.id)?.shown)?.[at] ?? NaN : 0.5;
    series.place(at, data.results.get(layer.id)?.values?.[at] ?? NaN, on && !(Number.isFinite(y) && (modeOf(doc, on) !== 1 || y > 0)) ? NaN : y);
  }
  // (A bubble pointed at: its ring as it's drawn now, its size or the largest moved.)
  if (bubbleAt.time !== null) lightBubble(bubbleAt.time, bubbleAt.pane);
  for (const id of paintedLayers.keys()) {
    // (Its point there: a value, or a cloud's band (one to its pane's edges, every date).)
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
  // (A gradient's range moved by it: all of it colored again from the data in hand (the live value in it), see
  // colorsAt and paintSig.)
  if (regrade) buildSoon(false);
  // (The view's change windows were read from the old value; the navigator's shape from the data before it.)
  windows.data = null;
  drawOverview();
  // (The crosshair set here back on its date and price, on its scale as the new value fits it: drawn now, then put
  // back, as after a build; the library keeps one set by a page at its height.)
  if (crossSet) engine.render(false), holdCross();
  updateValues();
}
let pricePending = false, priceTimer = 0, priceDelay = 1000;
async function pollPrice() {
  // (Asked by the whole client: see whole.)
  if (!tree || pricePending || document.visibilityState !== "visible") return;
  clearTimeout(priceTimer);
  pricePending = true;
  try {
    // On a phone the live element isn't shown: only a chart drawing the price asks. An answer that comes after a
    // draw (another chart, interval or block) moves no point: the next one is for it.
    // (Nor in full screen, where the live line isn't shown; on a phone, its grid shows it.)
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
    // Unanswered, it asks less often (up to every half minute); answered, each second again.
    priceDelay = Math.min(priceDelay * 2, 30_000);
  } finally {
    pricePending = false;
    if (document.visibilityState === "visible") priceTimer = setTimeout(pollPrice, priceDelay);
  }
}
document.addEventListener("visibilitychange", pollPrice);
withRest(pollPrice);

// ── Start ─────────────────────────────────────────────────────────────────

/** Drawings a written link had that couldn't be read (no layer, a time not a date, too few points): told. */
let lostDrawings = 0;
/**
 * The chart a link carries in the URL's hash, if any (let go once read: see clearLink): as JSON as one writes it (see
 * the guide, the "About" section: "{" first, as typed or URL-encoded), else compressed, as Studio writes them ("#chart=",
 * base64; a QR code's, "#c=", base32).
 * A chart already kept is the one opened, at the link's window (or its range, the link having none).
 */
async function fromHash(run = linkRun, at = 0) {
  // (The field the link starts with: what follows is its chart, whatever it holds; else the first there is.)
  const hash = location.hash.slice(1), params = new URLSearchParams(hash), key = /^(?:chart|image|c)(?==)/.exec(hash)?.[0] ?? (params.has("chart") ? "chart" : params.has("image") ? "image" : params.has("c") ? "c" : "");
  const shared = key ? params.get(key) : null;
  if (!shared) return null;
  picturing = key === "image";
  if (picturing) pictureDrawing();
  Object.assign(reading, { hash: location.hash, run, at });
  // (Read, the link goes from the address bar (see clearLink); one that can't be read stays as it is, to be put
  // right there.)
  // (Written, it runs to the end, read as it is: a "+" in a formula a plus, an "&" in a name, part of it.)
  return linked(new RegExp(`(?:^|&)${key}=(.*)$`, "s").exec(hash)?.[1] ?? "", shared, run, at, key === "c");
}
/**
 * A link opened in the address bar goes once read, its chart open (a picture's link stays its own; one that couldn't
 * be read, as it is, to be put right, until another chart's opened).
 */
function clearLink() {
  if (picturing || !isLinkHash(location.hash) || location.hash === brokenLink || (location.hash === reading.hash && currentRead(reading.run, reading.at))) return;
  history.replaceState(history.state, "", `${location.pathname}${location.search}`);
}
/**
 * The link in the address bar being read, and which read (none: none is): it stays there till it's read, or known
 * broken, or another chart's opened or link read meanwhile.
 */
const reading = { hash: "", run: 0, at: 0 };
/** Whether a link's read (`run`, the navigation it is: `at`) is still the one: no other link read, nor chart opened, since. */
const currentRead = (/** @type {number} */ run, /** @type {number} */ at) => run === linkRun && !(at && navigations > at);
/**
 * A chart as a link's fields say to look at it, over what it holds; what they don't say, as `made` opens (none: as it
 * is). A range or dates said, or as made, no view left on it from before.
 * @param {Chart} doc @param {ReturnType<typeof linkLookOf>} look @param {{ index: string, range: string, window?: string, show?: string }} [made]
 */
function takeLinkLook(doc, look, made) {
  doc.range = look.range ?? made?.range ?? doc.range;
  ((show) => show !== undefined && showLinkView(doc, show))(look.show ?? made?.show);
  // (A window it doesn't offer, as if it weren't said.)
  const offered = chartWindow(doc)?.offered ?? [], window = (look.window && windowNamed(look.window, offered)) || made?.window;
  if (window) atWindow(doc, window);
  // (An interval the picker wouldn't offer (a layer's ceiling, a formula's needs, a series' intervals: see
  // supportedIntervals), as if it weren't said.)
  doc.index = (look.index && menus.supportedIntervals(doc)(look.index) ? look.index : undefined) ?? made?.index ?? doc.index;
  if (look.range || look.view || made) {
    const all = viewsNow();
    delete all[doc.id];
    putViews(all);
  }
  if (look.view) keepCarriedView(doc, { view: datedView(doc.index, look.view) });
}
/** Why the last chart written couldn't be read (for studio.open). */
let linkError = "";
/**
 * A chart from a link, written (its JSON) or compressed (`shared`): the one kept already if it's the same, else it;
 * said in a toast (not under a picture), or why it couldn't be read.
 * @param {string} written @param {string | null} shared @param {number} run @param {number} [at] the navigation it is
 *   (another after it, it's dropped, unsaid; its own may not be counted yet: a link written out is read at once)
 */
async function linked(written, shared, run, at = 0, base32 = false) {
  // (Written out, as JSON, however encoded: else compressed.)
  const plain = /^(?:%20|%0a|%0d|%09|\s)*(?:\{|%7b|%257b)/i.test(written);
  try {
    lostDrawings = 0;
    // (Its layers' looks, as the presets have them, known first: a series without a type starts as theirs.)
    await presetsSettled();
    // (Another field after it, "&image=" or "&chart=": the whole read first, a title's "&" its own; else up to there.)
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
    // (Written out, its fields are what follow its last "}": an "&" inside it, the chart's own.)
    if (shared !== null) takeLinkLook(doc, linkLookOf(new URLSearchParams(plain ? written.replace(/^.*(?:\}|%7d|%257d)/is, "") : location.hash.slice(1))));
    const twin = twinOf(doc);
    const lost = lostDrawings ? `; ${plural(lostDrawings, "drawing")} couldn't be read` : "";
    // (From the address bar: read, it goes, its step in the history this chart's (on the grid, the grid's: opening
    // the chart steps on from it).)
    if (shared !== null && !picturing) history.replaceState(home ? { ...(history.state?.home ? history.state : {}), ...homeState() } : { chart: (twin ?? doc).id }, "", `${location.pathname}${location.search}`);
    if (shared !== null) reading.hash = "";
    // (A preview says so in its header, Save beside it: shown again only what that can't say, that it's one you have,
    // or drawings of it lost; else said, for a screen reader, the header not being read out.)
    if (!picturing) twin || lost ? toast(`Opened a shared chart${twin ? " you already have" : ""}${lost}`) : say("Opened a shared chart as a preview. Save it to keep it in your Library");
    return twin ?? doc;
  } catch (error) {
    // (Written by hand, why: to tell whoever wrote it; a newer link meanwhile, or a chart opened, nothing.)
    if (!currentRead(run, at)) return null;
    // (Plain words first, the reader's own after: whoever wrote it can find the spot.)
    linkError = plain ? `That link's chart couldn't be read: ${error instanceof SyntaxError ? `its JSON is broken (${error.message})` : /** @type {Error} */ (error).message}` : "That link doesn't contain a chart";
    if (shared !== null) (brokenLink = reading.hash || location.hash), (reading.hash = "");
    notice(linkError);
    return null;
  }
}
/** A chart a link (or a script, a file) brought, shown: the one open drawn anew, at the link's window; one kept, opened; else a preview. */
/** @returns {Chart} the chart shown (a preview of it already there, that one) */
function showLinked(/** @type {Chart} */ doc) {
  // (At its own window, whatever view is linked: the one it opens as, a preview like it if one's there.)
  linkWindow = (doc === chart || library.charts.includes(doc) ? doc : previewLike(doc) ?? doc).id;
  if (doc === chart) leaveHome(() => ((displayed = null), refresh(true)));
  else if (library.charts.includes(doc)) leaveHome(() => openChart(doc));
  else return showPreview(doc);
  return doc;
}
// A link pasted into this tab while it's open.
/** Each link opened, counted: one still being read when another comes is dropped (see fromHash). */
let linkRun = 0;
addEventListener("hashchange", () => {
  const params = new URLSearchParams(location.hash.slice(1));
  // (Cleared, or another kind: a link still being read is dropped.)
  if (!params.has("chart") && !params.has("image") && !params.has("c")) return void (++linkRun, (picturing = picturing && document.body.hasAttribute("data-picture")), document.body.hasAttribute("data-picture") && leavePicture());
  readLink();
});
/** The address bar's link read, and its chart shown. */
function readLink() {
  grid.closeDialogs();
  // (The navigation this is, as navigate counts it: another chart opened meanwhile, the grid shown, this one's dropped.)
  const run = ++linkRun, at = navigations + 1;
  navigate((async () => {
    const doc = await fromHash(run, at);
    if (run !== linkRun) return;
    // (Dropped for a chart opened meanwhile: no picture, and the address that chart's.)
    if (navigations > at) return void ((picturing = false), clearLink());
    const shown = doc ? showLinked(doc) : null;
    // (Its picture once it's the chart open: not the one before's; any other link, out of a picture.)
    if (shown && picturing) await until(() => chart === shown && !home, 5000);
    if (shown && picturing && run === linkRun) showPicture();
    else if (picturing && run === linkRun) pictureFailed(linkError);
    else if (document.body.hasAttribute("data-picture")) leavePicture();
  })());
}

// ── A link's picture, and the page's script API ───────────────────────────

/** Whether the page is a link's picture (#image=): see showPicture. */
let picturing = false;
/**
 * The page's last way to a chart (its start, a link, a script's open), till it's the chart open: what studio.ready
 * waits for first, failing as it failed (counted, as every chart opened is: see navigations).
 */
let navigating = Promise.resolve();
function navigate(/** @type {Promise<void>} */ way) {
  navigations++;
  navigating = way;
  // (Its failure said where it happens: a toast, or a script's own catch.)
  way.catch(() => {});
  return way;
}
/** The picture being made: a newer one's, the older dropped. */
let pictureRun = 0;
/**
 * A link's picture alone, made out of sight as full screen lays it out (click for its chart; failure says why,
 * Studio a click away).
 */
async function showPicture() {
  const run = ++pictureRun;
  document.body.toggleAttribute("data-picture", true);
  presetsEdit.layOutAlone(true);
  const image = /** @type {HTMLImageElement} */ ($("picture-img"));
  pictureDrawing();
  try {
    // (The chart as the window lays it out: a link's picture, across it; its legends' marks its own lines, not the
    // faint ones they show till its mini chart's drawn.)
    await studio.ready();
    document.title = `${titleOf(chart)} · Bitview Studio`;
    const at = navigations;
    if (await grid.freshThumb(chart, true)) saveThumbsSoon();
    const url = (await chartImage()).toDataURL("image/png");
    if (at !== navigations) throw new Error("Another chart was opened meanwhile");
    if (run !== pictureRun || !picturing) return;
    image.src = url;
    image.alt = `${titleOf(chart)}. ${chartFacts()}`;
    // (Its link as Copy link writes it, compressed: shown once it's this picture's.)
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
/** The picture-note's showings, counted: one a newer has taken over shows nothing. */
let noteRun = 0;
/**
 * A picture being drawn: a picture or a failure before it put away, and its note said after a moment, once the page's
 * fonts are in (a picture made at once shows none, and the note's never in the font before Inter).
 */
function pictureDrawing() {
  const run = ++noteRun, note = $("picture-note");
  $("picture-open").hidden = true;
  $("picture-leave").hidden = true;
  note.hidden = false;
  delete note.dataset.shown;
  note.textContent = "Drawing the chart…";
  setTimeout(() => void document.fonts.ready.then(() => run === noteRun && (note.dataset.shown = "")), 800);
}
/** A picture's link that gives none: the page says why (its title too, what a screenshot or a tab shows), Studio a click away. */
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
/** Out of a picture: the chart, as its #chart= link opens it (the picture's link let go). */
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
  // (A plain click: here. A new tab's, as links open.)
  if (event.metaKey || event.ctrlKey || event.shiftKey || event.button !== 0) return;
  event.preventDefault();
  leavePicture();
});
// A picture's page takes no keys for the chart out of sight under it (⌘K, P, a layer removed), wherever the focus
// is: its link's and button's own (↵, Tab) as ever, the browser's.
window.addEventListener("keydown", (event) => document.body.hasAttribute("data-picture") && event.stopImmediatePropagation(), true);

/**
 * The chart open (or opening: failing as a script's open failed) drawn whole, with what data it has (its reveal and
 * fades at their end), and the fonts its picture writes in, all within `ms`; else why not: at once for what can't
 * change (no chart, no layers).
 */
async function settle(ms = 60_000) {
  // (The time given for all of it: the chart's way there and the fonts too.)
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
      // (The chart drawn, a heatmap of it not two frames on: the library had nothing on its scale to draw it on
      // (see heatIn).)
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
  // (Drawn on its next frame, where there are frames: out of sight there are none, and a picture's drawn when it's made.)
  if (!document.hidden) await new Promise((done) => (requestAnimationFrame(() => done(undefined)), setTimeout(done, Math.min(100, left()))));
  ended();
  // (That frame's draw asking a heatmap for days: waited for too.)
  if (!heatmaps.heatIn()) return settle(left());
}
/** The picture a script last had (see png): made for the chart as it was, read again in parts. @type {{ wanted: string, url: string } | null} */
let picture = null;
/** A script's number checked: from `low` to `high`, else it's told so. */
const bounded = (/** @type {unknown} */ value, /** @type {string} */ name, /** @type {number} */ low, /** @type {number} */ high) => {
  if (typeof value !== "number" || !(value >= low && value <= high)) throw new Error(`${name}: ${low} to ${high}`);
  return value;
};
/**
 * Studio for scripts run in the page (a console, an assistant driving a browser): a chart opened as its link opens
 * it, waited for, its picture as Export makes it; the guide says the same ("From a script").
 */
const studio = Object.freeze({
  /** A chart as a link writes it (an object, or its JSON): opened as its link would be; done once it's the one open. */
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
  /** Once the chart open (or opening) is drawn with its data in (see settle), and has some: else why not. */
  async ready(ms = 60_000) {
    await settle(ms);
    if (!resolved?.times.length) {
      const why = chart.layers.flatMap((layer) => ((error) => (error ? [`${layer.letter}: ${error}`] : []))(resolved?.results.get(layer.id)?.error));
      throw new Error(`The chart has no data${why.length ? ` (${why.join("; ")})` : ""}`);
    }
  },
  /**
   * Its picture as Export makes it (title, legends, dates), as a data URL: `width` and `height` in CSS pixels (1200 by
   * 760 if none, whatever the window), `scale` 1 or 2 (1 if none), a `type` (PNG, WebP or JPEG) and its
   * `quality`; with `part`, that part of its base64 (`partSize` characters, 45,000 if none: with the rest, under 50,000) as { part, parts, mime,
   * base64 }, part 0 made for the chart as it is, the others from that same picture (refused for another chart
   * open since, or other options, `partSize` too). None if another chart opens meanwhile.
   */
  async png(/** @type {{ width?: number, height?: number, scale?: number, type?: string, quality?: number, part?: number, partSize?: number }} */ options = {}) {
    const width = Math.round(bounded(options.width ?? 1200, "width", 320, 4000)), height = Math.round(bounded(options.height ?? 760, "height", 240, 4000));
    const scale = options.scale ?? 1;
    if (scale !== 1 && scale !== 2) throw new Error("scale: 1 or 2");
    const type = options.type ?? "image/png", quality = bounded(options.quality ?? 0.9, "quality", 0.01, 1), partSize = Math.round(bounded(options.partSize ?? 45_000, "partSize", 1000, 1e8));
    if (!["image/png", "image/webp", "image/jpeg"].includes(type)) throw new Error('type: "image/png", "image/webp" or "image/jpeg"');
    await studio.ready();
    // (Its legends' marks its own lines, as the picture page draws them: not the faint ones they show till its mini
    // chart's drawn, which a page just opened, by a script, has yet to.)
    if (await grid.freshThumb(chart, true)) saveThumbsSoon();
    // (Made for the chart as it is now, whole or its part 0; its parts after the first from that same picture, of the
    // same chart, cut alike, whatever's moved since (a live price each second); one asked for otherwise, none.)
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
  /** The chart open, as a link writes it: a copy (changing it changes nothing here; open it to). */
  chart: () => structuredClone(menus.toSketch(chart)),
  /** Once it's drawn, its layers that show nothing, and why: [{ layer, error }], a missing series' closest too. */
  async errors() {
    await settle();
    return Promise.all(chart.layers.flatMap((layer) => {
      const result = resolved?.results.get(layer.id), error = result?.error ? result.detail ?? result.error : fromUnfit(chart, layer);
      if (!error) return [];
      const closest = !layer.series ? null : result?.missing ? inspector.closestSeries(layer.series) : result?.detail === BY_BLOCK ? Promise.resolve(indexCatalog()).then(() => dailyOf(/** @type {string} */ (layer.series)) ?? inspector.closestSeries(/** @type {string} */ (layer.series))) : null;
      return [closest ? closest.then((name) => ({ layer: layer.letter, error, ...(name ? { closest: name } : {}) })) : { layer: layer.letter, error }];
    }));
  },
  /** Series names, best first, only those Studio can chart. A cold page waits for the browser's catalog index. */
  async search(/** @type {string} */ text, count = 10) {
    const most = Math.round(bounded(count, "count", 1, 50));
    indexCatalog();
    return (await command.findSeries(String(text), most + 20)).filter((name) => !gone.has(name) && !byBlock.has(name)).slice(0, most);
  },
});
Object.defineProperty(window, "studio", { value: studio });

// ── The page's place: its scroll, and its shelves' ───────────────────────────

/** The page's scroll. */
const homeScroll = () => /** @type {HTMLElement} */ (document.querySelector(".home-scroll"));
/**
 * Each shelf's scroll, as it last scrolled (one not in it, new: at its start). Where the page is, read and put back
 * by it: a shelf out of sight, its scroll read or written, is laid out for it alone, shelf by shelf (back from a
 * chart: 64 ms at a phone's 4× slower CPU, to write where they were already).
 * @type {WeakMap<HTMLElement, number>}
 */
const shelfScrolls = new WeakMap();
/** The shelves on the page: not the presets' kept hidden under another (see renderGallery). */
const shelfRows = () => all$(".shelf-row:not([hidden] *)", $("home"));
/** Where the page is: its scroll, and each shelf's. */
const galleryPlace = () => ({ top: homeScroll().scrollTop, rows: Object.fromEntries(shelfRows().map((row) => [shelfKey(row), shelfScrolls.get(row) ?? 0])), shown: galleryShown });
/** A shelf by what it is (where its name leads, else its name): kept where it was, whatever came before it since. */
const shelfKey = (/** @type {HTMLElement} */ row) => ((title) => title?.getAttribute("data-place") ?? title?.firstChild?.textContent ?? "")(row.closest(".shelf")?.querySelector(".shelf-title"));
/** The gallery back where it was (drawn with as much of it as it showed, see popstate). @param {{ top: number, rows: Record<string, number> }} place */
function placeGallery(place) {
  // (Only those not there already: the rest aren't laid out for it.)
  for (const row of shelfRows()) {
    const left = place.rows[shelfKey(row)] ?? 0;
    if (left !== (shelfScrolls.get(row) ?? 0)) (row.scrollLeft = left), shelfScrolls.set(row, row.scrollLeft);
  }
  homeScroll().scrollTop = place.top;
}

// ── Quick Look: ⌘K's highlighted chart, big ────────────────────────────────

/** Quick Look works by a keyboard: none by touch, nor on a phone. */
const quickWorks = () => command.keyboard() && !phone.matches;
phone.addEventListener("change", () => phone.matches && quickHide());
/** Whether a ⌘K row is a chart to look at. */
const canLook = (/** @type {any} */ item) => Boolean(lookableAt(item));
/** The chart a ⌘K row is, to look at: a chart's (yours, a preview), a preset's (as edited here, else as made). @returns {{ doc: Chart, i: number } | null} */
function lookableAt(/** @type {any} */ item) {
  // (A chart's row: ⌘K's search's, a folder's page's; a preset's own chart, as the preset it is.)
  const chartKey = item?.kind === "chart" ? item.key : /^chart:/.test(String(item?.key ?? "")) ? String(item.key).slice(6) : null;
  if (chartKey !== null) return ((doc) => (doc ? { doc, i: doc.preset !== undefined ? presetIndex(doc) : -1 } : null))(chartById(chartKey));
  const at = /^(?:preset|recommended):(\d+)$/.exec(String(item?.key ?? "")), i = at ? Number(at[1]) : -1;
  return presets?.[i] ? { doc: presetDoc(i), i } : null;
}
/** Quick Look's picture: as large as its frame, as many points as it has room for. */
const QUICK_PICTURE = { width: 1000, height: 560, points: 700 };
/** Each chart's picture for Quick Look as last made, by what it's of (its content, the theme): the last few; with each layer's latest value, as its legend says it. @type {Map<string, Promise<{ drawn: Preview, values: Map<string, string>, bands: Map<string, string> } | null>>} */
const quickPictures = new Map();
/** A chart's picture for Quick Look: drawn from its data on its own, at its interval, over its window (nothing opened). */
function quickPicture(/** @type {Chart} */ doc) {
  const key = `${doc.id}:${grid.previewKey(doc)}:${light}`;
  let made = quickPictures.get(key);
  if (made) quickPictures.delete(key);
  else made = (async () => {
    // (At an interval with as many points as it has room for, no finer: a week's for years, the chart's own for weeks.)
    const index = await grid.sampleable(doc, grid.sampleIndex(doc, "week1", QUICK_PICTURE.points)), copy = { ...structuredClone(doc), index };
    const data = await resolve(copy).catch(() => null);
    // (Not drawn, or some of it missing: not kept, asked again next time.)
    if (!data || grid.retryFailed(data, index)) return void quickPictures.delete(key), null;
    const [heat, bubbles] = await Promise.all([grid.heatSamples(copy, data, grid.previewBars(doc, index), HEAT_QUICK_COLUMNS), grid.bubblesAt(doc, data, index)]);
    // (Its bubbles' fetch failed, or the theme changed meanwhile: its heatmaps' colors another's, none; its days not all
    // in: shown, not kept.)
    if (!bubbles || key !== `${doc.id}:${grid.previewKey(doc)}:${light}`) return void quickPictures.delete(key), null;
    if (heat.whole === false) quickPictures.delete(key);
    const drawn = grid.previewOf(copy, data, grid.previewBars(doc, index), QUICK_PICTURE, heat, bubbles);
    // (Each layer's value at the picture's end, as the legend reads it there, a cloud's by the legend's own reading.)
    const values = new Map(copy.layers.flatMap((layer) => {
      const result = data.results.get(layer.id), high = result?.values, low = result?.low, unit = unitFor(layer, result);
      // (A heatmap's, as its legend reads it at rest: its open day's cell at that day's close, see heatReading; at the
      // chart's own density (the picture's coarser cells hold several), else the picture's. A profile's, none: its
      // share is of the days it draws, see profileReading.)
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
    // (Its named values, in their band's color.)
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
/**
 * Quick Look on ⌘K's highlighted row: a picture of its chart, big, over ⌘K (in it: the search keeps the keys), drawn
 * on its own (nothing opened, nothing behind it changes); ↑ ↓ the next row's, ⌘Y or esc put it away, ↵ does what
 * ⌘K's ↵ does.
 */
function quickShow() {
  // (Where it can't show (the window made a phone's), one showing goes: it would show another than the one lit.)
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
    // (Drawn where the highlight rests: rows passed by, held ↓, ask for nothing.)
    if (doc.layers.length) requestAnimationFrame(() => quick?.key === key && void quickPicture(doc).then((made) => {
      if (quick?.key !== key) return;
      const plot = /** @type {HTMLElement} */ (frame.querySelector(".quick-plot"));
      plot.classList.remove("pending");
      if (!made) return void (plot.textContent = "Couldn't be drawn");
      const { drawn, values, bands } = made, { width: W, height: H } = QUICK_PICTURE;
      plot.innerHTML = grid.previewSvg(drawn, `quick-${doc.id}`, QUICK_PICTURE);
      // (Each layer's latest value by its name; each pane's highest and lowest where they are; the dates along its foot.)
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
/** Quick Look put away. */
function quickHide() {
  if (!quick) return;
  quick = null;
  // (Its pictures not kept past it: the next look draws from the data as it is then.)
  quickPictures.clear();
  grid.leave($("quick"));
  if (isOpen($("palette"))) $("palette-input").focus({ preventScroll: true });
}
const quickToggle = () => (quick ? quickHide() : quickShow());
// (A click on it, on its chart too: the keys back to ⌘K's search.)
$("quick").addEventListener("pointerup", () => requestAnimationFrame(() => isOpen($("palette")) && $("palette-input").focus({ preventScroll: true })));
// (Its ×, or around it: put away.)
$("quick").addEventListener("click", (event) => {
  // (Rows waiting drawn first, and the look then theirs: what shows is what opens.)
  if (closest(event, "[data-quick-open]")) return void (command.palette.pending ? command.palette.pending() : quick?.key && command.choose());
  if (event.target === $("quick") || closest(event, "[data-quick-close]")) quickHide();
});
$("palette-quick").addEventListener("click", () => (quickToggle(), $("palette-input").focus({ preventScroll: true })));
// (esc puts it away first; ⌘K's closing, with it.)
$("palette").addEventListener("cancel", (event) => {
  if (!quick) return;
  event.preventDefault();
  event.stopImmediatePropagation();
  quickHide();
}, { capture: true });
onClosed($("palette"), quickHide);

async function start() {
  // (The arrangement as loaded: one changed from it, not written yet, is this tab's own.)
  arrangedAs = arrangedNow();
  // (The page's first navigation, as navigate counts it: a chart opened while a link's read, the link's dropped.)
  const at = navigations + 1, shared = await fromHash(linkRun, at);
  // (A chart opened while its link was read: the link dropped, no picture, and gone from the address bar.)
  if (navigations > at) (picturing = false), clearLink();
  // Edited previews kept aside (left, reloaded, a tab closed): previews again; one opened, with nothing else to show.
  // (One kept since, in another tab, let go of.)
  // (One kept in your charts as late or later, let go of; a later one, its edits since in another tab, a preview of
  // its own, under a new id: its copy aside with it.)
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
  // (Charts another tab saved while this one was starting: the first of them, as a library's first is.)
  chart ??= chartById(library.active) ?? library.charts[0] ?? setAside[0];
  // First visit: the empty chart in place of none (a draft) under the page, the Charts page over it.
  const first = !chart;
  if (first) chart = newDraft();
  // Where this tab was, as its history keeps it (a reload), else where the studio was last: the page, or a chart. A
  // shared link opens its chart; a first visit, the Charts page. An address of the presets' (a visit, not a step
  // back), what it names: the presets read first (the page out of sight meanwhile); one they don't have, the Charts page.
  const there = shared ? null : history.state, resumed = there?.chart ? chartById(there.chart) : null;
  // (An address of the presets', a reload too: what it says, over what this tab kept (the address follows what's shown).)
  const asked = !shared && ROUTED && location.pathname !== `${BASE}/` ? location.pathname : "";
  if (asked) await loadPresets().catch(() => {});
  const routed = asked ? routeOf(asked) : null;
  chart = (routed && "preset" in routed ? presetByAddress(routed.preset, routed.window) : resumed) || chart;
  // (The empty chart a first visit makes in place of none, not wanted under a preset: let go of.)
  if (first && routed && "preset" in routed) {
    const draft = previewCharts.findIndex((doc) => drafts.has(doc));
    if (draft >= 0) previewCharts.splice(draft, 1);
  }
  // (An address of a section, there; one that's nothing, the presets' page, said; the presets unread, kept till they are.)
  if (asked) homeAt = routed && "path" in routed ? { path: routed.path } : presets ? { path: [] } : { top: true };
  if (asked && !routed && !presets) pendingRoute = asked;
  else if (asked && !routed) setTimeout(() => withRest(() => notice(presetsEdit.missedSaid(asked))));
  // (Blank charts left elsewhere (a tab closed on one): let go of, the one open here aside.)
  for (const doc of library.charts.filter((doc) => doc !== chart && emptyYours(doc))) dropEmpty(doc);
  home = !shared && Boolean(asked ? !(routed && "preset" in routed) : there ? there.home : first || prefs.home);
  started = true;
  document.body.toggleAttribute("data-home", home);
  $("home").inert = !home;
  // (Asked by an address, its step is where it led (one waiting for the presets says so: a reload waits again).)
  if (!there || asked) history.replaceState({ ...(home ? homeState() : { chart: chart.id }), ...(pendingRoute ? { route: true } : {}) }, "");
  library.active = chart.id;
  rememberViewed(chart.id);
  normalize(chart);
  save();
  // (Home is the grid, an empty chart its cards: shown with the rest, so it's in first (not loading: said as at load);
  // a chart, drawn first, then the rest: see later.js. Nothing drawn and the rest lost: offline, as at load.)
  if (home || !chart.layers.length) await loadRest().catch((error) => (document.body.toggleAttribute("data-offline", true), Promise.reject(error)));
  // (Onto the grid, its chart drawn once it's opened: the grid's first moments its own, not shared with a chart's
  // build out of sight: see redrawChart.)
  refresh(!home);
  // (Rendered: the page shown, settling in whole; opened from a home screen, there whole at once, under its launch's
  // orange, which drawing back is its way in: see /assets/launch/launch.js.)
  if ("launch" in document.documentElement.dataset) document.body.classList.remove("boot");
  document.documentElement.dataset.ready = "";
  if (picturing && shared) showPicture();
  else if (picturing) pictureFailed(linkError || "That link doesn't contain a chart");
  else if (document.body.hasAttribute("data-picture")) leavePicture();
  setTimeout(() => document.body.classList.remove("boot"), 1400);
  // Once the chart's asked for its data: the search's index builds, off the page, at once (search is this machine's
  // a second or so after a load, not after an idle moment a hidden tab may give late).
  withRest(() => command.startFinder());
  // Charts this Studio couldn't read: kept aside, said once (again only when they change), to be had as a file
  // (Download all has them too).
  const aside = storage.get(UNREAD, []), told = Array.isArray(aside) ? String(hashOf(JSON.stringify(aside))) : "";
  if (Array.isArray(aside) && aside.length && storage.get(`${UNREAD}:told`, "") !== told && storage.set(`${UNREAD}:told`, told)) notice(`${plural(aside.length, "saved chart")} couldn't be read here and ${aside.length === 1 ? "was" : "were"} set aside`, { label: "Download", run: () => menus.download("bitview-charts-kept-aside.json", new Blob([JSON.stringify({ v: 2, charts: aside }, null, 2)], { type: "application/json" })) });
}
// (Shown whatever start makes of it: one failing shows what it got to, not a page held out of sight, nor held from
// input (see started).)
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
  paletteMarksSeen, paletteOnField, paletteOpenings, panelShown, paneShowing, pathResume, pending, pendingRoute, phone,
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
