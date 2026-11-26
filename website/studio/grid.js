// @ts-check
// The page's grid: tiles, mini charts, the presets' gallery, picking charts and moving them.
import { initialsOf, initialsWord } from "/studio/initials.js";
import { QuickMatch, QuickMatchConfig } from "/modules/quickmatch-js/src/index.js";
import { columnWidth, indexed, PARTIAL as PARTIAL_SHARE, placeIndex, press } from "/modules/bitview-charts/index.js";
import { capitalized, shownStep } from "/studio/words.js";
import {
  acrossUnfit, applyTheme, asThen, cardsSeen, chart, chartById, chartNow, css, deleteChart, deleteFolder, dialogsShown,
  discard, dismissPalette, drafts, drawCards, drawGallery, drawPresetMarks, drawYours, emptyYours, engine, FOLDER_ICONS,
  folderAgain, folderMark, folderNow, fromUnfit, galleryAhead, galleryPlace, galleryShown, galleryTilesAhead,
  galleryTilesSeen, goTo, GROUP_OF, GROUPS, HEAT_PICTURE_AGG, HEAT_PICTURE_GRID, HEAT_PICTURE_ROWS, HEAT_TILE_COLUMNS,
  heatKnown, heldUpright, home, homeAt, homeScroll, homeScroller, importCharts, inLibrary, isPreview, isYours,
  keepSidePresets, keepsLeaving, keptAsEdited, keysNow, layoutCurve, layoutOwed, leaveHome, leftView, leftWith,
  letterOf, library, libraryEdits, libraryStep, light, listedPreviews, listOwed, livePrice, loadPresets, minedHalvings,
  mostBars, MOTION, newDraft, notice, openChart, openView, paletteMarksSeen, phone, pickPreset, placeGallery, prefs,
  presetLabels, presetMarks, presets, presetsFailed, presetsMatching, pressedTile, previewCharts, previews,
  previousChart, profileColor, railMenuKept, recentCharts, redoing, reduced, relayout, relist, renderCharts, save,
  savePrefs, savePreviewsSoon, saveView, sectionMark, shelfRows, shelfScrolls, sideLight, sidePresets, started, storage,
  THEMES, thumbs, toast, touched, undoLibrary, viewOn, within, writeSideList, yourTilesSeen, zeroFit,
} from "/studio/app.js";
import {
  $, all$, closest, composing, escapeHtml, holdPage, KEY_STEPS, scrollsY, SLOP, stepIn, TOUCH, typing,
} from "/studio/dom.js";
import { inSlices, until, whenIdle } from "/studio/timing.js";
import { clamp, hashOf, lowerBound, uid } from "/studio/math.js";
import { format, grouped, plural, precise } from "/studio/format.js";
import { ICONS, markBody } from "/studio/icons.js";
import {
  belowOf, blankChart, BUBBLE_HIDE, BUBBLE_SIZE, cloudLayers, DASHES, dashOf, edgesOf, gradientOfLayer, heatGradient, heatLike, LINED, measureOf,
  opacityOf, OUTLINED, PAIRS, placedOn, summed, unfitted,
} from "/studio/vocabulary.js";
import { FINE, INTERVAL_DAYS, INTERVAL_LABEL, INTERVAL_ORDER, rangeDays } from "/studio/intervals.js";
import { isColor } from "/studio/color.js";
import { stopCarrying, swallowClick } from "/studio/gestures.js";
import {
  asked, baseOf, byColor, colorChain, colorNow, cutsOf, defaultSize, drawSeries, edges, fitStrip, foldOf, hiddenPanes,
  forget, gapped, info, ink, intervalsOf, load, loads, MIN_BAR_SPACING, paintOf, periodAmount, presetWindow, profiled, resolve,
  retryInfo, rightGap, seriesIn, SIDES, unitFor, viewName,
} from "/studio/chart.js";
import {
  chartActions, choicePage, inputPage, openPage, renameChart, renameChartPage, searchCase, searchText, typedWords,
} from "/studio/command.js";
import { tickOn } from "/studio/drawing.js";
import { FUNCTIONS, parse } from "/studio/formulas.js";
import {
  amountsOf, heatCalendar, heatColor, heatCome, heatDayHeld, heatDayKey, heatDays, heatExtras, heatGrid, heatLook,
  heatMost, heatPump, mostIn, rowsOf,
} from "/studio/heatmaps.js";
import {
  fadingOf, fillOf, focusKey, gradientCss, gradientTable, labelOf, layerById, levelSteps, lineLook, modeOf, rangeOf,
  restoreFocus, STACK_FILL, stackOf,
} from "/studio/inspector.js";
import {
  anythingKept, chartsFile, closedOwner, cutFade, download, downloadAll, downloadChart, downloadFolder, fadeOut, fading,
  hideMenu, keepCarriedView, menuOf, onRail, ownerNow, railed, setMenuOwner, supportedIntervals,
} from "/studio/menus.js";
import {
  editedPreset, fromPreset, keptPreset, presetById, presetDoc, presetDocs, presetEdited, presetIdOf, presetIndex,
  presetKey, pristine, RECOMMENDED, refocus, showAddress, vocabNow,
} from "/studio/presets-edit.js";
/** @import { Group, Thumb } from "/studio/app.js" */
/** @import { Chart, Folder, Layer } from "/studio/vocabulary.js" */
/** @import { Resolved } from "/studio/chart.js" */
/** @import { Action } from "/studio/command.js" */
/** @import { Node, Vector } from "/studio/formulas.js" */
/** @import { heatWants } from "/studio/heatmaps.js" */
/** @import { Preset } from "/studio/presets-edit.js" */

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
/** A drawing's key, what it's of, under the drawings' version (see MARK_VERSION). @param {unknown[]} parts */
const drawingKey = (parts) => MARK_VERSION + JSON.stringify(parts);
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
/** Whether a formula runs a window (a call with its span: on a coarser interval, a short one would be one point). */
const runsWindow = (/** @type {string} */ formula) => {
  /** @param {Node} node @returns {boolean} */
  const spans = (node) => node.k === "call" ? (FUNCTIONS[node.f][0] >= 2 && FUNCTIONS[node.f][3] !== null && (node.args.length === FUNCTIONS[node.f][0] || FUNCTIONS[node.f][5] !== undefined)) || node.args.some(spans)
    : node.k === "bin" ? spans(node.a) || spans(node.b) : node.k === "neg" ? spans(node.a) : false;
  try {
    return spans(parse(formula));
  } catch {
    return false;
  }
};
/** What a mini chart depends on: its layers, its panes, and the day. @param {Chart} doc */
const thumbKey = (doc) => drawingKey([new Date().toISOString().slice(0, 10), doc.index, doc.range, doc.panes.map((pane) => [pane.mode, pane.size, pane.stack, pane.reverse, pane.average, pane.outline, pane.gradient]), doc.layers.map((layer) => [layer.id, layer.series, layer.formula, layer.hidden, layer.color, layer.side, doc.panes.findIndex((pane) => pane.id === layer.pane), layer.letter, layer.type, layer.base, layer.colorBelow, layer.colors, layer.fill, layer.from, layer.ref, layer.colorFrom, layer.fit, layer.byLevels ? layer.levels : null, layer.byGradient ? [layer.gradient, layer.upToDate] : null, heatLike(layer) ? [layer.heatmap, layer.profile, layer.cohortColors, layer.weight, measureOf(layer), rowsOf(layer), layer.cumulative] : null, layer.outline, ...fadingKey(layer)])]);
/** Four-color bars' fading colors as drawn (their default the theme's): what a picture of them depends on. */
const fadingKey = (/** @type {Layer} */ layer) => (layer.type === "histogram" && layer.colors === 4 ? [ink(fadingOf(layer, false)), ink(fadingOf(layer, true))] : []);
/**
 * A layer's shape: its values as its pane scales them (log, or from the first value), averaged down to the mini
 * charts' points and lightly smoothed (at this size, the trend is what reads), with the range of the full values;
 * bars, each the value farthest from their base.
 */
function shapeOf(values, mode, bars = false, base = 0) {
  const from = values.find((value) => Number.isFinite(value) && value !== 0) ?? NaN, scale = onScale(mode, from);
  const scaled = onScaleAll(values, mode, from);
  const [lo, hi] = rangeOf(scaled);
  const step = scaled.length / THUMB_POINTS, level = scale(base);
  const averaged = Array.from({ length: THUMB_POINTS }, (_, k) => {
    let sum = 0, count = 0, far = NaN;
    for (let i = Math.floor(k * step), end = Math.min(scaled.length, Math.max(Math.floor(k * step) + 1, Math.floor((k + 1) * step))); i < end; i++) {
      const value = scaled[i];
      if (value - value !== 0) continue;
      sum += value;
      // Bars keep the one farthest from their base (an average of them would sit on it).
      if (!count++ || Math.abs(value - level) > Math.abs(far - level)) far = value;
    }
    return !count ? null : bars ? far : sum / count;
  });
  const points = averaged.map((value, k) => {
    if (value === null || bars) return value;
    const near = [averaged[k - 1], value, value, averaged[k + 1]].filter((other) => other !== null && other !== undefined);
    return near.reduce((sum, other) => sum + other, 0) / near.length;
  });
  return { points, lo, hi, scale };
}
/**
 * A value as its pane's scale draws it, for its shape: a log scale's, its log; a percent or indexed one's, its change
 * from where it starts (`from`) over that one's size, as the chart's are (from a negative one, a rise still rises).
 */
const onScale = (/** @type {number} */ mode, /** @type {number} */ from) => (/** @type {number} */ value) => (!Number.isFinite(value) ? NaN : mode === 1 ? (value > 0 ? Math.log10(value) : NaN) : mode >= 2 ? (from ? (value - from) / Math.abs(from) : NaN) : value);
/**
 * Values as onScale puts them, all at once (from `first` to `last`, every one by default; NaN past them): by index,
 * finite by a subtraction, no call a value (thousands a picture, a phone's engine without a JIT).
 */
function onScaleAll(/** @type {ArrayLike<number>} */ values, /** @type {number} */ mode, /** @type {number} */ from, first = 0, last = values.length - 1) {
  const out = new Float64Array(values.length).fill(NaN), size = Math.abs(from);
  for (let i = Math.max(0, first), end = Math.min(last, values.length - 1); i <= end; i++) {
    const value = values[i];
    out[i] = value - value !== 0 ? NaN : mode === 1 ? (value > 0 ? Math.log10(value) : NaN) : mode >= 2 ? (from ? (value - from) / size : NaN) : value;
  }
  return out;
}
/** Points placed between a low and a high, in hundredths of the height. */
const place = (points, lo, hi, bottom = 0, height = 1) => points.map((value) => (value === null || !Number.isFinite(value) ? null : clamp(Math.round((bottom + ((value - lo) / (hi - lo || 1)) * height) * 100) / 100, 0, 1)));
/** Work done one pass at a time: asked for again while one runs, it runs once more after. @param {() => Promise<void>} pass */
function serial(pass) {
  let running = false, again = false;
  const run = async () => {
    if (running) return void (again = true);
    running = true;
    try {
      await pass();
    } finally {
      running = false;
      if (again) {
        again = false;
        void run();
      }
    }
  };
  return run;
}
/**
 * Whether a series of a chart failed to come, each forgotten to be fetched again next time: a failed fetch leaves
 * nothing behind, no blank drawing, no cached failure. (A series the server doesn't have, or not at this interval,
 * isn't a failure to retry: it's drawn without.)
 * @param {Resolved} data @param {string} index
 */
function retryFailed(data, index) {
  for (const name of data.failed) {
    forget(name, index);
    retryInfo(name);
  }
  return data.failed.length > 0;
}
/** @param {Chart} doc A copy, so edits made meanwhile don't mix in. @returns {Promise<Thumb | null>} null when a fetch failed */
async function drawThumb(doc) {
  // (The open chart's own data, or any chart's all in memory: asked for once, not again at another interval. Not a finer
  // interval's: only part of its history loads.)
  const held = !FINE.has(doc.index) && (doc.id === chart?.id || drawSeries(doc).every((name) => loads.has(`${name}@${doc.index}`)));
  const index = held ? doc.index : await sampleable(doc, sampleIndex(doc, "month1", THUMB_POINTS));
  // (Its own frame, the open chart's not changed by a copy of it: the arrays are shared all the same, see parts.)
  const data = await resolve({ ...doc, index, layers: doc.layers.map(layer => ({ ...layer, hidden: false })), ...(held ? { id: `${doc.id}:mark` } : {}) });
  if (retryFailed(data, index)) return null;
  const bubbles = await bubblesAt(doc, data, index);
  if (!bubbles) return null;
  /** @type {Map<string, ReturnType<typeof shapeOf>>} */
  // (The chart's lines as it draws them, stacked; each layer's own mark, its own values.)
  const shapes = new Map(), own = new Map(), stacked = stackOf(doc, data, false);
  // (Its range, as the chart opens on it: the last 8 years, or all of them.)
  const start = rangeStart(doc, data.times);
  for (const layer of doc.layers) {
    const all = gapped(data.results.get(layer.id)?.values, data.results.get(layer.id)), values = all && start ? Array.prototype.slice.call(all, start) : all, highAll = stacked.get(layer.id)?.values, high = highAll && start ? Array.prototype.slice.call(highAll, start) : highAll, bars = layer.type === "histogram" && modeOf(doc, layer) === 0;
    // (A chart's mark is its lines: a cloud, a range, isn't one; a signal's marks neither.)
    if (!values || layer.type === "cloud" || placedOn(layer)) continue;
    const base = baseOf(layer);
    own.set(layer.id, shapeOf(values, modeOf(doc, layer), bars, base));
    shapes.set(layer.id, high ? shapeOf(high, modeOf(doc, layer), bars, base) : own.get(layer.id));
  }
  // Up to three lines, first pane first; each pane with one gets a band as tall as on the chart.
  const groups = [];
  let budget = 3;
  doc.panes.forEach((pane, i) => {
    const layers = doc.layers.filter((layer) => layer.pane === pane.id && !layer.hidden && shapes.has(layer.id)).slice(0, budget);
    if (!layers.length) return;
    budget -= layers.length;
    groups.push({ pane, size: pane.size ?? defaultSize(i), layers });
  });
  const total = groups.reduce((sum, group) => sum + group.size, 0);
  const lines = /** @type {Thumb["lines"]} */ ([]);
  /** Each layer's line as drawn, and its pane's band. @type {Map<string, { index: number, band: { bottom: number, height: number } }>} */
  const lineAt = new Map();
  let top = 1;
  for (const { pane, size, layers } of groups) {
    const height = size / total, gap = groups.length > 1 ? Math.min(0.08, height * 0.25) : 0;
    const band = { bottom: top - height + gap / 2, height: height - gap };
    top -= height;
    // Each axis side has its own scale, as on the chart.
    for (const side of SIDES) {
      const on = layers.filter((layer) => layer.side === side).map((layer) => ({ layer, shape: shapes.get(layer.id) }));
      // Bars' base is in their scale, as on the chart.
      // (A stack's, its foot too, on a linear scale: one layer alone, a 100% one, is a line at its top.)
      // (Those out of the fit drawn on the others' scale: see unfitted.)
      const framing = on.filter(({ layer }) => !unfitted(doc, layer));
      const bases = framing.flatMap(({ layer, shape }) => [
        ...(layer.type === "histogram" && !summed(pane) && modeOf(doc, layer) === 0 ? [shape.scale(baseOf(layer))] : []),
        ...(zeroFit(doc, layer, modeOf(doc, layer), summed(pane)) ? [shape.scale(0)] : []),
      ]).filter(Number.isFinite);
      const lo = Math.min(...framing.map(({ shape }) => shape.lo), ...bases), hi = Math.max(...framing.map(({ shape }) => shape.hi), ...bases);
      if (!Number.isFinite(lo)) continue;
      for (const { layer, shape } of on) {
        const ys = place(shape.points, lo, hi, band.bottom, band.height);
        // Drawn as its type: a baseline's colors and bars part at its base (where the scale has none, under it all).
        const kind = { area: "area", baseline: "baseline", histogram: modeOf(doc, layer) === 0 ? "bars" : undefined, dots: "dots" }[layer.type];
        const level = place([shape.scale(baseOf(layer))], lo, hi, band.bottom, band.height)[0] ?? band.bottom;
        // An area fills down to its band's foot, or up to its top where it's filled above.
        const edge = layer.fill === "above" ? Math.min(1, band.bottom + band.height + gap / 2) : Math.max(0, band.bottom - gap / 2);
        // (Painted date by date: each point's color, its bucket's last date's.)
        // (Of the dates it shows: its range's, from `start`.)
        const dateColors = paintOf(doc, layer, data), step = (data.times.length - start || 1) / THUMB_POINTS;
        const colors = dateColors && ys.map((value, k) => (value === null ? null : dateColors(start + Math.min(data.times.length - start - 1, Math.max(Math.floor(k * step), Math.floor((k + 1) * step) - 1))) ?? layer.color));
        if (colors) lines.push([layer.color, ys, kind ?? null, kind ? belowOf(layer) : null, kind ? (kind === "area" ? edge : level) : null, colors]);
        else lines.push(kind ? [layer.color, ys, kind, belowOf(layer), kind === "area" ? edge : level] : [layer.color, ys]);
        lineAt.set(layer.id, { index: lines.length - 1, band });
      }
    }
  }
  // A heatmap, a wash about the line it's behind (its pane's first drawn, the price as a rule): an unrealized P&L's,
  // its loss's color over it, its profit's under it, as its cells are; another's, its color on both sides. Bubbles,
  // their two largest (apart), dots on their layer's line, each in its side's color.
  const washes = /** @type {Thumb["washes"]} */ ([]), dots = /** @type {Thumb["dots"]} */ ([]);
  for (const layer of doc.layers) {
    if (layer.hidden) continue;
    // (Where the chart draws it: a holder group it knows, on a scale it can be, see heatGrid. In its colors: its
    // pane's gradient's middle; an unrealized P&L's, red for a loss and green for a profit, see heatColor.)
    if (layer.type === "heatmap" && !washes.length && heatKnown(layer) && heatGrid(doc, layer)) {
      const behind = doc.layers.find((other) => other.pane === layer.pane && lineAt.has(other.id)), at = behind && lineAt.get(behind.id);
      if (!at) continue;
      const stops = heatGradient(doc, layer), one = stops ? stops[Math.floor(stops.length / 2)][1] : "";
      washes.push(stops ? [at.index, one, one, at.band.bottom, at.band.bottom + at.band.height] : [at.index, belowOf(layer), layer.color, at.band.bottom, at.band.bottom + at.band.height]);
    } else if (layer.type === "bubbles" && layer.ref && lineAt.has(layer.ref) && dots.length < 4) {
      const all = bubbles.get(layer.id) ?? data.results.get(layer.id)?.values, values = all && start ? Array.prototype.slice.call(all, start) : all, ys = lines[/** @type {{ index: number }} */ (lineAt.get(layer.ref)).index][1];
      if (!values) continue;
      // (Bucketed as its line's points are: see shapeOf.)
      const step = values.length / ys.length, peaks = [];
      for (let k = 0; k < ys.length; k++) {
        let far = NaN;
        for (let i = Math.floor(k * step); i < Math.max(Math.floor(k * step) + 1, Math.floor((k + 1) * step)); i++) if (Number.isFinite(values[i]) && !(Math.abs(values[i]) <= Math.abs(far))) far = values[i];
        if (Number.isFinite(far) && far !== 0 && ys[k] !== null) peaks.push({ k, far });
      }
      peaks.sort((a, b) => Math.abs(b.far) - Math.abs(a.far));
      const chosen = [];
      for (const peak of peaks) if (chosen.length < 2 && chosen.every((other) => Math.abs(other.k - peak.k) > 2)) chosen.push(peak);
      for (const { k, far } of chosen) dots.push([far < 0 ? belowOf(layer) : layer.color, k / (ys.length - 1), /** @type {number} */ (ys[k])]);
    }
  }
  // Each layer alone fills its own mark; a baseline's and bars' with their base.
  const marks = /** @type {Record<string, (number | null)[]>} */ ({}), bases = /** @type {Record<string, number>} */ ({});
  for (const [id, { points, scale }] of own) {
    const there = points.filter((value) => value !== null);
    if (there.length < 2) continue;
    const layer = doc.layers.find((item) => item.id === id), level = layer && PAIRS[layer.type] ? scale(baseOf(layer)) : NaN;
    // Bars' range takes in their base: each one's height is its distance from it.
    const range = layer?.type === "histogram" && modeOf(doc, layer) === 0 && Number.isFinite(level) ? [...there, level] : there;
    marks[id] = place(points, Math.min(...range), Math.max(...range));
    if (layer && PAIRS[layer.type]) bases[id] = place([level], Math.min(...range), Math.max(...range))[0] ?? 0;
  }
  return { lines, washes, dots: dots.slice(0, 4), marks, bases };
}
const orphan = (id) => !chartById(id);
/** Writes our drawings over the stored ones (another tab's stay), without charts that are gone. */
function saveThumbs() {
  const merged = { ...(storage.get(THUMBS, {})), ...Object.fromEntries(thumbs) };
  for (const id of Object.keys(merged)) if (orphan(id)) delete merged[id];
  storage.set(THUMBS, merged);
}
/** A chart's row, and its legend while it's open, show its new drawings in place: focus, menus and tooltips stay. @param {Chart} doc */
function updateMark(doc) {
  for (const mark of all$(`:is([data-open-chart="${doc.id}"], [data-open-recent="${doc.id}"]) .chart-mark`)) mark.outerHTML = chartMark(doc);
  if (doc !== chart) return;
  // (Its legends' and its panel rows' marks, drawn from it.)
  for (const element of all$(".legend .series-mark, .layer-row .row-mark > .series-mark")) {
    const layer = layerById(/** @type {HTMLElement} */ (element.closest("[data-select]")).dataset.select);
    if (layer) element.outerHTML = seriesMark(layer);
  }
}
/** A chart whose mark failed (a fetch), and when: asked for again after a while, or once it's edited. @type {Map<string, { key: string, at: number }>} */
const thumbsFailed = new Map();
/**
 * A chart's mini chart drawn again where it's out of date, its marks with it (its row's, its legends'): whether it was.
 * `picture`: for a picture of it being made, whose legends need their marks now, its hidden layers' data loaded for it.
 * @param {Chart} doc @param {boolean} [picture]
 */
async function freshThumb(doc, picture = false) {
  // A background card must not load hidden data while its chart is open. The library draws it on return.
  if (!picture && doc.id === chart?.id && !home && seriesIn(doc).some(name => !asked(name, doc))) return false;
  const key = thumbKey(doc), failed = thumbsFailed.get(doc.id);
  if (thumbs.get(doc.id)?.key === key || (failed?.key === key && Date.now() - failed.at < 30_000)) return false;
  const drawing = doc.layers.length ? await drawThumb(structuredClone(doc)).catch(() => null) : { lines: [], washes: [], dots: [], marks: {} };
  if (!drawing) thumbsFailed.set(doc.id, { key, at: Date.now() });
  // Edited or deleted meanwhile: the next run draws what it has become.
  if (!drawing || thumbKey(doc) !== key || chartById(doc.id) !== doc) return false;
  thumbs.set(doc.id, { key, ...drawing });
  updateMark(doc);
  return true;
}
// A card is the chart's plot, small: every pane in its band (with the chart's scale margins), every layer shown in
// it as its type (see previewSvg), over the view the chart was left at (else its range), a point per few pixels. Drawn
// from the chart's own data, which is then at hand when it opens.

const PREVIEWS = "bitview-studio:previews";
const PREVIEW = { width: 320, height: 200, points: 120 };
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
/** What a card depends on past a mini chart's: its layers' looks the mark leaves out, the view it was left at and how much the chart shows. @param {Chart} doc */
// (A heatmap's picture in its colors as the theme shows them: drawn again in the other. A profile shown, its bars and
// its room after the latest (see previewShown): one kept from before them drawn again, others' as they were.)
const previewKey = (doc) => drawingKey([thumbKey(doc), doc.layers.map((layer) => [layer.colors, layer.dash, layer.opacity, layer.levels, layer.baseLine, layer.underName, layer.fillOpacity, layer.solid, layer.hollow, layer.size, layer.hide, layer.scaleTo, layer.byLayers]), openView(doc), FINE.has(doc.index) ? Math.round(engine.time.width / 50) : 0, doc.layers.some((layer) => heatLike(layer) && !layer.hidden) ? light : null, ...(profiled(doc) ? ["profile"] : [])]);
/**
 * How many bars a chart's preview shows at most (its data at `index`): its range's, as the chart opens on it (Max, at
 * a day or coarser, the whole history); finer, no more than fit the chart's width at four a pixel.
 */
function previewBars(/** @type {Chart} */ doc, index = doc.index) {
  const width = engine.time.width, days = rangeDays(doc.range);
  // (A short range at a coarse interval, 3M of months: a dozen bars still, not a preview of none.)
  return Math.min(FINE.has(doc.index) && width ? ((width / MIN_BAR_SPACING) * INTERVAL_DAYS[doc.index]) / INTERVAL_DAYS[index] : Infinity, days ? Math.max(12, days / INTERVAL_DAYS[index]) : Infinity);
}
/** Where a chart's range starts in its data: its first point within the range's days of the last one (Max, the first). */
const rangeStart = (/** @type {Chart} */ doc, /** @type {ArrayLike<number>} */ times) => {
  const days = rangeDays(doc.range);
  return days && times.length ? lowerBound(times, times[times.length - 1] - days * 86400) : 0;
};
/**
 * The drawings kept: the recommended charts', then those of the charts edited last, then the presets' last drawn,
 * within a budget (they're a cache: drawn again when missing, they never crowd out the charts themselves; a day old,
 * see thumbKey). A launch's Charts page shows those it kept, not each drawn again from its data. Written oldest
 * first, as they're read back: the last drawn last again.
 */
function savePreviews() {
  const drawn = new Map([...previews.keys()].map((id, k) => [id, k]));
  const kept = /** @type {[string, Preview & { key: string }][]} */ ([]), when = (/** @type {string} */ id) => (id.startsWith("recommended:") ? Infinity : chartById(id)?.updated ?? (id.startsWith("preset:") ? /** @type {number} */ (drawn.get(id)) : -Infinity));
  let size = 0;
  for (const [id, item] of [...previews].filter(([id]) => when(id) > -Infinity).sort(([a], [b]) => when(b) - when(a))) {
    size += JSON.stringify(item).length;
    if (size > 1_000_000) break;
    kept.push([id, item]);
  }
  storage.set(PREVIEWS, Object.fromEntries(kept.reverse()));
}
/**
 * What a chart's picture shows of its data (`most` bars at most): the view it opens on, its own; the open one's,
 * linked, every chart's (as its chunks are, see spanOf); else its range. (Another chart's: the room any chart has
 * after its latest point, not the open one's tags'.) A shown profile's room after the latest, as the chart leaves it
 * (see rightGap): a quarter of the picture, where it ends at the latest. As the picture spans it, its edges: a view's
 * as the chart shows it (the library's range is its screen's edges); a range's from its first bar's left edge to its
 * last's right edge (its window by their middles: half a bar more each side).
 * @param {Chart} doc @param {{ times: number[] }} data @param {number} most
 */
const previewShown = (doc, data, most) => {
  const left = openView(doc), shown = left ? within(viewOn(left, doc.index, data.times, rightGap()), "to") : presetWindow(doc, data, most, 0, 0), last = data.times.length - 1, ROOM = 0.25;
  const range = profiled(doc) && shown.to >= last - 0.5 && shown.to - last < (shown.to - shown.from) * ROOM ? { from: shown.from, to: last + ((last - shown.from) * ROOM) / (1 - ROOM) } : shown;
  return left ? range : { from: range.from - 0.5, to: range.to + 0.5 };
};
/**
 * Bubbles' values for a picture drawn at a coarser interval than its chart's (weeks for years, months for a mark): at
 * each of its points, the largest by size (its sign kept) of the chart's own over its span, as the chart's buckets
 * keep theirs. A day's sum read once a week is one day in seven: its largest days, the bubbles that read, missed.
 * By layer, aligned with the picture's points; outside what its own series reaches at the chart's interval (a fine
 * interval's: its view's chunks), the picture's own. None for a picture as fine as its chart; null when a fetch failed: the picture not
 * kept, asked again (see retryFailed).
 * @param {Chart} doc its own interval's @param {{ times: number[], results: Map<string, any> }} data the picture's @param {string} index the picture's
 * @returns {Promise<Map<string, Float64Array> | null>}
 */
async function bubblesAt(doc, data, index, signal = /** @type {AbortSignal | undefined} */ (undefined)) {
  const layers = doc.layers.filter((layer) => layer.type === "bubbles" && !layer.hidden), times = data.times, n = times.length;
  if (!layers.length || !(INTERVAL_DAYS[index] > INTERVAL_DAYS[doc.index])) return new Map();
  // (The chart as it's drawn, a copy: its frame and its view's span shared with it, see resolve and spanOf.)
  const fine = await resolve(structuredClone(doc), false, signal).catch(() => null);
  if (!fine || signal?.aborted || retryFailed(fine, doc.index)) return null;
  return new Map(layers.flatMap((layer) => {
    const values = fine.results.get(layer.id)?.values, own = data.results.get(layer.id)?.values;
    if (!values || !own) return [];
    // (Its own reach, not the timeline's: another layer's series may reach where its own didn't load.)
    let from = 0, to = values.length - 1;
    while (from <= to && !Number.isFinite(values[from])) from++;
    while (to >= from && !Number.isFinite(values[to])) to--;
    const out = new Float64Array(n).fill(NaN), first = fine.times[from], last = fine.times[to];
    for (let k = 0; k < n; k++) {
      if (!(times[k] >= first && times[k] <= last)) {
        out[k] = own[k];
        continue;
      }
      const end = k + 1 < n ? times[k + 1] : Infinity;
      let far = NaN;
      for (let j = lowerBound(fine.times, times[k]); j < fine.times.length && fine.times[j] < end; j++) if (Number.isFinite(values[j]) && !(Math.abs(values[j]) <= Math.abs(far))) far = values[j];
      out[k] = far;
    }
    return [[layer.id, out]];
  }));
}
/**
 * Each heatmap's days a picture shows, by its layer's id: a day for each of its `columns`, evenly across what it
 * shows (see previewShown), the open day, and the open day at the layer's own density (`fine`: the chart's
 * reference, see heatPicture); each profile's, its groups' days on the last it shows (`days`, in its groups' order,
 * as the chart's, see profilePoint); asked for as the chart's are (a want
 * of its own, see heatExtras, after the chart's), waited for (20 s at most: those not in then, none). None shown
 * (hidden, a holder group Bitview doesn't have, a scale it can't be drawn on), none.
 * Each layer's, and whether all its days came (`whole`: failed for good, as they are).
 * @typedef {Map<string, { days: (any)[], open: any, fine: any }> & { whole?: boolean }} HeatSamples
 * @param {Chart} doc @param {{ times: number[] }} data @param {number} most @param {number} columns @returns {Promise<HeatSamples>}
 */
async function heatSamples(doc, data, most, columns) {
  const layers = doc.layers.filter((layer) => heatLike(layer) && !layer.hidden && (layer.type === "heatmap" || layer.profile?.length) && heatKnown(layer) && heatGrid(doc, layer)), n = data.times.length;
  const shown = n > 1 ? previewShown(doc, data, most) : null, span = shown ? shown.to - shown.from : 0;
  if (!layers.length || !shown || !(span >= 1)) return Object.assign(/** @type {HeatSamples} */ (new Map()), { whole: true });
  // (A column past the data (a view's room after its latest), no day: left empty.)
  const { g, days } = heatCalendar(data), at = Array.from({ length: columns }, (_, c) => ((i) => (i >= 0 && i < n ? days[g[i]] : null))(Math.round(shown.from + ((c + 0.5) / columns) * span)));
  /** @type {{ wants: NonNullable<ReturnType<typeof heatWants.get>>, keys: Set<string> }} */
  const extra = { wants: [], keys: new Set() };
  const want = (/** @type {string} */ cohort, /** @type {string} */ weight, /** @type {string} */ agg, /** @type {string} */ day) => {
    const key = heatDayKey(cohort, weight, agg, day);
    if (extra.keys.has(key)) return key;
    extra.keys.add(key);
    if (!heatDayHeld(key)) extra.wants.push({ key, cohort, weight, agg, day, rank: 0 });
    return key;
  };
  // (A profile's day, the last it shows, as the chart's is the view's last.)
  const end = days[g[clamp(Math.floor(shown.to), 0, n - 1)]];
  const keyed = layers.map((layer) => {
    if (layer.type === "profile") return /** @type {[string, (string | null)[], string, string]} */ ([layer.id, (layer.profile ?? []).map((cohort) => want(cohort, layer.weight ?? "", HEAT_PICTURE_AGG, end)), "", ""]);
    const cohort = layer.heatmap ?? "all", weight = layer.weight ?? "", own = /** @type {NonNullable<ReturnType<typeof heatGrid>>} */ (heatGrid(doc, layer)).agg;
    // (The open day at its own density too: the chart's, its key the chart's, as often there already.)
    return /** @type {[string, (string | null)[], string, string]} */ ([layer.id, at.map((day) => (day === null ? null : want(cohort, weight, HEAT_PICTURE_AGG, day))), want(cohort, weight, HEAT_PICTURE_AGG, "latest"), want(cohort, weight, own, "latest")]);
  });
  heatExtras.add(extra);
  try {
    heatPump();
    await heatCome([...extra.keys], 20_000, () => true);
    // (Some not in yet, or failed for now: drawn, but not kept as up to date, see drawPreview.)
    const samples = /** @type {HeatSamples} */ (new Map(keyed.map(([id, keys, open, fine]) => [id, { days: keys.map((key) => (key === null ? null : heatDays.get(key))), open: heatDays.get(open), fine: heatDays.get(fine) }])));
    samples.whole = [...extra.keys].every((key) => heatDays.get(key)?.rows || (heatDays.get(key)?.failed && !heatDays.get(key)?.retryAt));
    return samples;
  } finally {
    heatExtras.delete(extra);
  }
}
/**
 * A heatmap drawn small (a tile's, Quick Look's), as a picture (a PNG's data URL): each of its days a column, each
 * pixel row from `top` to `bottom` (twice as many, sharp) the cell at the price `priceAt` reads there, colored as
 * the chart colors it (heatColor): as much a row of the chart's as it holds on average (a cell holds several of
 * them), against the chart's reference (its open day's largest row, at its density; not come, the open day's
 * largest cell here: brighter, the gaps between its rows filled). None to draw, none.
 * @param {Chart} doc @param {Layer} layer @param {Resolved | { times: number[], results: Map<string, any> }} data
 * @param {{ days: any[], open: any, fine: any } | undefined} sample @param {(y: number) => number} priceAt @param {number} top @param {number} bottom
 */
function heatPicture(doc, layer, data, sample, priceAt, top, bottom) {
  const look = heatLook(doc, layer, /** @type {Resolved} */ (data), opacityOf(layer)), k = HEAT_PICTURE_ROWS, grid = HEAT_PICTURE_GRID, rows = modeOf(doc, layer) <= 1 ? rowsOf(layer) : k;
  // (A cell as so many of the chart's rows: the reference as large as they'd all be at it.)
  const most = sample?.fine?.rows ? heatMost(sample.fine, look.measure) * (rows / k) : sample?.open?.rows ? heatMost(sample.open, look.measure) : 0;
  if (!sample || !most || !(bottom > top)) return null;
  const columns = sample.days.length, height = Math.round((bottom - top) * 2), canvas = document.createElement("canvas");
  canvas.width = columns;
  canvas.height = height;
  const context = /** @type {CanvasRenderingContext2D} */ (canvas.getContext("2d")), image = context.createImageData(columns, height), pixels = image.data;
  // (Each pixel row's cell, once; a cell's color once a day, copied down the pixel rows it covers. None where the
  // price isn't over zero: NaN.)
  const cells = new Float64Array(height);
  for (let p = 0; p < height; p++) {
    const price = priceAt(top + (p + 0.5) / 2);
    cells[p] = price > 0 ? grid.cell(price) : NaN;
  }
  for (let c = 0; c < columns; c++) {
    const day = sample.days[c];
    if (!day?.rows) continue;
    const values = amountsOf(day, look.measure);
    for (let p = 0, was = NaN; p < height; p++) {
      const row = cells[p], at = (p * columns + c) * 4;
      if (row === was) pixels.copyWithin(at, at - columns * 4, at - columns * 4 + 4);
      else if (row === row) {
        const found = lowerBound(day.rows, row);
        if (day.rows[found] === row) heatColor(values[found], most, look, pixels, at);
      }
      was = row;
    }
  }
  context.putImageData(image, 0, 0);
  return canvas.toDataURL("image/png");
}
/**
 * Each picture's panes as last drawn, by the data they're drawn from (see previewOf): the picture's look (the chart's,
 * its size, its window) and each pane's marks with what they're drawn from. The next picture of the same look from
 * the same data keeps each pane drawn from the same: a live price, which gives its own layer a new result alone, draws
 * its pane again, not a stack's 23 bands.
 * @type {WeakMap<object, { look: string, times: ArrayLike<number>, panes: { from: any[], marks: any[], scales: any[] }[] }>}
 */
const panesDrawn = new WeakMap();
/**
 * @param {Chart} doc @param {{ times: number[], results: Map<string, any> }} data
 * @param {number} [most] the most bars the chart shows, in this data's interval @returns {Preview}
 */
function previewOf(doc, data, most = mostBars(), size = PREVIEW, heat = /** @type {HeatSamples} */ (new Map()), bubbles = /** @type {Map<string, Float64Array>} */ (new Map())) {
  const { width: W, height: H } = size;
  const times = data.times, n = times.length;
  /** @type {Preview} */
  const drawn = { seps: [], marks: [] };
  if (n < 2) return drawn;
  // (The view it opens on: its own; the open one's, linked, every chart's (as its chunks are, see spanOf).)
  const shown = previewShown(doc, data, most), span = shown.to - shown.from;
  if (!(span >= 1)) return drawn;
  // No more points than bars in view: each one then holds a bar at least, so only missing data breaks a line.
  const B = Math.min(size.points, Math.floor(span));
  // (Quick Look's: its dates along its foot, its scales, the last bar it shows.)
  const big = size !== PREVIEW;
  if (big) {
    drawn.end = Math.min(n - 1, Math.floor(shown.to));
    drawn.ticks = dateTicks(times, shown.from, shown.to, W);
    drawn.scales = [];
  }
  // (Its bars: those whose middles are in it, as the chart's, see firstInView and lastShown.)
  const first = clamp(Math.ceil(shown.from), 0, n - 1), last = clamp(Math.floor(shown.to), 0, n - 1);
  /** A bucket's middle, as a date's place. @param {number} b */
  const middleOf = (b) => shown.from + ((b + 0.5) * span) / B;
  /** A bucket's first and last points in view (the last bucket's to the view's end). @param {number} b */
  const bucketOf = (b) => ((edge) => [Math.max(first, Math.ceil(shown.from + (b * span) / B)), Math.min(last, b === B - 1 ? Math.max(edge, Math.floor(shown.to)) : edge)])(Math.ceil(shown.from + ((b + 1) * span) / B) - 1);
  const round = (value) => Math.round(value * 10) / 10;
  /** A date's place across; written in a path, once a picture (its layers' points all at its dates, by index). */
  const x = (/** @type {number} */ i) => round(((i - shown.from) / span) * W), xTexts = new Array(n), xText = (/** @type {number} */ i) => xTexts[i] ?? (xTexts[i] = String(x(i)));
  // (A hidden pane takes no band: it shows nothing.)
  const hidden = hiddenPanes(doc), sizeOf = (pane, k) => (hidden.has(pane.id) ? 0 : pane.size ?? defaultSize(k));
  const total = doc.panes.reduce((sum, pane, i) => sum + sizeOf(pane, i), 0), stacked = stackOf(doc, data);
  // (The layers bubbles shown are on.)
  const carried = new Set(doc.layers.filter((layer) => layer.type === "bubbles" && !layer.hidden && layer.ref).map((layer) => layer.ref));
  // (The panes last drawn from this data, if it looked as it does now: see panesDrawn.)
  const look = JSON.stringify([previewKey(doc), W, H, size.points, big, shown.from, shown.to, light]), was = panesDrawn.get(data), before = was?.look === look && was.times === times ? was.panes : null;
  /** @type {{ from: any[], marks: any[], scales: any[] }[]} */
  const panes = [];
  let top = 0;
  doc.panes.forEach((pane, k) => {
    const height = (sizeOf(pane, k) / total) * H;
    if (top > 0 && height) drawn.seps.push(round(top));
    const paneTop = round(top), paneBottom = round(top + height);
    const band = { top: top + height * 0.17, height: height * 0.75 };
    top += height;
    // The pane's marks in the order the chart draws its series, whichever side each is on (a stack from its top, at
    // the back, to its foot); their lines across behind them all.
    const from = drawn.marks.length, ranks = new Map(), inPaneOrder = doc.layers.filter((layer) => layer.pane === pane.id);
    // What they're drawn from: the pane's place, and its layers' results (a stack's sums are of its own), heatmap days
    // and bubbles, and those of each layer their colors come from (see colorsAt). The same as last time: its marks kept.
    const sources = /** @type {any[]} */ ([band.top, band.height]), scalesFrom = drawn.scales?.length ?? 0;
    for (const layer of inPaneOrder) for (const at of colorChain(doc, layer)) sources.push(data.results.get(at.id), heat.get(at.id), bubbles.get(at.id));
    const kept = before?.[k];
    if (kept && kept.from.length === sources.length && sources.every((item, i) => item === kept.from[i])) return void (drawn.marks.push(...kept.marks), drawn.scales?.push(...kept.scales), panes.push(kept));
    const zOf = new Map((pane.reverse ? inPaneOrder : [...inPaneOrder].reverse()).map((layer, z) => [layer.id, z]));
    /**
     * A bubbles layer as the chart draws it, each at `place` (its date's height, NaN for none): each bucket's largest
     * (none past the view's end), as wide across as its size of the largest (in view, or of all time) has it, the
     * largest first, by sizes a pixel apart (a size's bubbles one mark), ringed in the ground's color if outlined.
     * @param {Layer} layer @param {(i: number) => number} place
     */
    const drawBubbles = (layer, place) => {
      // (At the chart's own interval, each point its span's largest: see bubblesAt.)
      const own = bubbles.get(layer.id) ?? data.results.get(layer.id)?.values;
      if (layer.hidden || !own) return;
      const most = layer.scaleTo === "all" ? mostIn(own) : mostIn(own, shown), least = (layer.hide ?? BUBBLE_HIDE) * most, across = (layer.size ?? BUBBLE_SIZE) * (big ? 1 : 0.6);
      if (!(most > 0)) return;
      // (Each bucket's largest with its middle in its pane, and the largest just out of it, larger: clipped to its pane,
      // as the chart's, what of it shows. Just out: within twice its radius, its size in pixels and its place in the
      // picture's own units, a tile drawn as small as half of them. One wholly out (a layer off its scale: a hidden one
      // in a 100% stack), none.)
      const inside = new Map(), edge = new Map(), larger = (/** @type {Map<number, { i: number }>} */ kept, /** @type {number} */ b, /** @type {number} */ size) => !(Math.abs(own[kept.get(b)?.i ?? NaN]) >= size);
      for (let i = first; i <= last; i++) {
        const size = Math.abs(own[i]), b = Math.min(Math.floor(((i - shown.from) / span) * B), i <= shown.to ? B - 1 : B);
        if (!(size > 0 && size < Infinity) || size < least || b < 0 || b >= B) continue;
        const at = place(i), radius = (across / 2) * Math.sqrt(Math.min(1, size / most));
        if (!Number.isFinite(at)) continue;
        if (at >= paneTop && at <= paneBottom) larger(inside, b, size) && inside.set(b, { i, at });
        else if (at >= paneTop - 2 * radius && at <= paneBottom + 2 * radius) larger(edge, b, size) && edge.set(b, { i, at });
      }
      for (const [b, out] of edge) if (larger(inside, b, Math.abs(own[out.i]))) inside.set(-1 - b, out);
      const sizes = new Map();
      for (const { i, at } of inside.values()) {
        const w = Math.max(1, Math.round(across * Math.sqrt(Math.min(1, Math.abs(own[i]) / most)))), dot = `M${x(i)} ${at}h0`;
        const into = sizes.get(w) ?? sizes.set(w, { d: "", d2: "" }).get(w);
        if (own[i] >= 0) into.d += dot;
        else into.d2 += dot;
      }
      for (const [w, { d, d2 }] of [...sizes].sort(([a], [b]) => b - a)) {
        const mark = { k: "bubble", c: layer.color, c2: belowOf(layer), d, ...(d2 ? { d2 } : {}), w, top: paneTop, bottom: paneBottom, ...(opacityOf(layer) < 1 ? { op: opacityOf(layer) } : {}), ...(layer.outline ? { o: doc.layers.indexOf(layer) + 1 } : {}) };
        drawn.marks.push(mark), ranks.set(mark, zOf.get(layer.id) ?? 0);
      }
    };
    // Each side of a pane has its own scale, as on the chart.
    for (const side of SIDES) {
      // (A signal's marks aren't drawn this small. A layer bubbles are on, hidden or not drawn here (a trend in a
      // stacked pane), unseen but there, as on the chart: their scale's and their places'.)
      const shaped = doc.layers.filter((layer) => layer.pane === pane.id && layer.side === side && (!layer.hidden || carried.has(layer.id)) && !placedOn(layer) && data.results.get(layer.id)?.values && (!fromUnfit(doc, layer) || carried.has(layer.id))).map((layer) => {
        const values = /** @type {Vector} */ (gapped(stacked.get(layer.id)?.values ?? data.results.get(layer.id)?.values, data.results.get(layer.id))), mode = modeOf(doc, layer);
        // Percent and indexed scales start from the first value in view.
        let base = NaN;
        for (let i = first; mode >= 2 && i <= last && !Number.isFinite(base); i++) if (Number.isFinite(values[i]) && values[i] !== 0) base = values[i];
        const scaled = onScale(mode, base), inView = onScaleAll(values, mode, base, first, last);
        // Each bucket's low and high, in the order they came (one point where they're one), and the latest value last:
        // at up to four bars a pixel the chart shows every bar, so its peaks and troughs show; an average would round
        // them off.
        const lows = new Float64Array(B).fill(Infinity), highs = new Float64Array(B).fill(-Infinity), lowAt = new Float64Array(B), highAt = new Float64Array(B);
        let latest = -1, latestIn = -1;
        const breaks = data.results.get(layer.id)?.breaks, cut = breaks ? new Uint8Array(B) : null;
        for (let i = first; i <= last; i++) {
          // (The point at the view's very end is its last bucket's, not past it. Its bucket by a truncation, no call:
          // a point before the view's start, its place under 0 but never under -1, none.)
          const value = inView[i], at = ((i - shown.from) / span) * B, edge = i <= shown.to ? B - 1 : B, b = at < 0 ? -1 : (at | 0) < edge ? at | 0 : edge;
          if (cut && breaks?.[i] && b >= 0 && b < B) cut[b] = 1;
          if (value - value !== 0 || b < 0 || b >= B) continue;
          if (value < lows[b]) (lows[b] = value), (lowAt[b] = i);
          if (value > highs[b]) (highs[b] = value), (highAt[b] = i);
          (latest = i), (latestIn = b);
        }
        /** @type {([number, number][] | null)[]} */
        const points = Array.from({ length: B }, (_, b) => {
          if (highs[b] === -Infinity || cut?.[b]) return null;
          const low = /** @type {[number, number]} */ ([lowAt[b], lows[b]]), high = /** @type {[number, number]} */ ([highAt[b], highs[b]]);
          return lowAt[b] === highAt[b] ? [low] : lowAt[b] < highAt[b] ? [low, high] : [high, low];
        });
        // (In its own bucket: one a break cuts, none, the gap kept.)
        const end = latestIn >= 0 ? points[latestIn] : null;
        if (end && latest > end[end.length - 1][0]) end.push([latest, scaled(values[latest])]);
        // A cloud: each bucket from its lowest low to its highest high (to an edge, its values: the band's drawn to the
        // pane's edge, and its scale needn't read it; of its two edges, each bucket with a date, its scale none).
        const result = data.results.get(layer.id), low = result?.low, whole = edgesOf(layer).length > 1 && !cloudLayers(layer).length;
        const band = low ? Array.from({ length: B }, (_, b) => {
          let under = Infinity, over = -Infinity;
          for (let [i, to] = bucketOf(b); i <= to; i++) {
            if (whole) return /** @type {[number, number]} */ ([0, 0]);
            const down = scaled(low[i]), up = scaled(/** @type {Vector} */ (values)[i]);
            if (Number.isFinite(down) && Number.isFinite(up)) (under = Math.min(under, down)), (over = Math.max(over, up));
          }
          return under <= over ? /** @type {[number, number]} */ ([under, over]) : null;
        }) : null;
        // By its layers, its clouds as the chart's (see drawBands): each bucket at its last date with a value, one between
        // each two of its layers next to each other in its order (its pane's top with its first, its bottom with its
        // last: ±Infinity), in the color of the one over.
        const members = result?.members, ids = result?.memberIds;
        // (Its scale's points, by its pairs: each bucket's lowest and highest of their values (not its pane's edges),
        // as the chart's own (on a log scale, a pair over zero though another isn't).)
        const hull = (/** @type {{ band: ([number, number] | null)[] }[]} */ pairs) => Array.from({ length: B }, (_, b) => {
          const values = pairs.flatMap(({ band: ranges }) => ranges[b] ?? []).filter((value) => Number.isFinite(value));
          return values.length ? /** @type {[number, number]} */ ([Math.min(...values), Math.max(...values)]) : null;
        });
        const bands = layer.type === "cloud" && layer.byLayers && members?.length && ids && low ? ((top, bottom) => {
          const colors = ids.map((id) => doc.layers.find((item) => item.id === id)?.color ?? layer.color), n = members.length;
          const ats = Array.from({ length: B }, (_, b) => {
            let at = -1;
            for (let [i, to] = bucketOf(b); i <= to; i++) if (Number.isFinite(low[i])) at = i;
            return at;
          });
          const valueOf = (/** @type {number} */ m, /** @type {number} */ at) => (m === -1 ? Infinity : m === -2 ? -Infinity : scaled(members[m][at]));
          const pairs = [...Array.from({ length: n - 1 }, (_, m) => [m, m + 1]), ...(top ? [[-1, 0]] : []), ...(bottom ? [[n - 1, -2]] : [])];
          return pairs.map(([a, z]) => {
            const ranges = ats.map((at) => ((p, q) => (at >= 0 && !Number.isNaN(p) && !Number.isNaN(q) && (Number.isFinite(p) || Number.isFinite(q)) ? /** @type {[number, number]} */ ([Math.min(p, q), Math.max(p, q)]) : null))(at >= 0 ? valueOf(a, at) : NaN, at >= 0 ? valueOf(z, at) : NaN));
            const tints = ats.map((at) => (at < 0 ? layer.color : ((p, q) => colors[p >= q ? (a < 0 ? z : a) : z < 0 ? a : z])(valueOf(a, at), valueOf(z, at)) ?? layer.color));
            return { band: ranges, tint: (/** @type {number} */ b) => tints[b] };
          });
        })(Boolean(layer.from?.includes("top")), Boolean(layer.from?.includes("bottom"))) : null;
        const ranged = bands ? hull(bands) : band;
        // (Where a baseline's colors and bars part, as opposed to the base a percent or indexed scale counts from.)
        const parts = baseOf(layer);
        return { layer, unseen: Boolean(layer.hidden || fromUnfit(doc, layer)), mode, from: base, scaled, values, inView, bars: layer.type === "histogram" && mode === 0, base: parts, level: scaled(parts), points: whole ? band?.map(() => null) ?? points : ranged ? ranged.map((range, b) => (range ? range.map((value) => /** @type {[number, number]} */ ([middleOf(b), value])) : null)) : points, band, bands, stack: stacked.has(layer.id) };
      });
      // A stack's layers, sampled at the same points, every one's highs and lows: each is the sum of those under it,
      // so their edges never cross, and none loses its peaks (a 100% stack's top is flat).
      const stack = shaped.filter((item) => item.stack);
      if (stack.length > 1) {
        // (Each bucket's dates, in order, once each: in Quick Look, any of its layers' lows and highs; on a card,
        // where the stack's top is lowest and highest (flat, a 100% stack's: its first and last), and the latest,
        // its line's end. Each layer's values there, finite by a subtraction. By loops: no call a point.)
        const shared = new Array(B);
        if (big) for (let b = 0; b < B; b++) {
          const all = [], indices = [];
          for (const item of stack) for (const point of item.points[b] ?? []) all.push(point[0]);
          all.sort((a, z) => a - z);
          for (let k = 0; k < all.length; k++) if (k === 0 || all[k] !== all[k - 1]) indices.push(all[k]);
          shared[b] = indices;
        }
        else {
          // (Its top: the highest band with a value there. A 100% stack's is flat, whatever its sums' rounding.)
          const flat = pane.stack === "percent", down = [...stack].sort((a, z) => (stacked.get(z.layer.id)?.rank ?? 0) - (stacked.get(a.layer.id)?.rank ?? 0));
          let lastIn = -1, latest = -1;
          for (let b = 0; b < B; b++) {
            const [from, to] = bucketOf(b);
            let start = -1, end = -1, low = -1, high = -1, least = Infinity, most = -Infinity;
            for (let i = from; i <= to; i++) {
              let top = NaN;
              for (const item of down) if (item.inView[i] - item.inView[i] === 0) {
                top = item.inView[i];
                break;
              }
              if (top - top !== 0) continue;
              if (start < 0) start = i;
              end = i;
              if (top < least) (least = top), (low = i);
              if (top > most) (most = top), (high = i);
            }
            shared[b] = start < 0 ? [] : flat || least === most ? (start === end ? [start] : [start, end]) : low < high ? [low, high] : [high, low];
            // (A band with values in the bucket but not at each place its top's taken, one starting, ending or there
            // for a while within it: its first and last there too, else it'd be a point there or gone. One there at
            // each place, as most are, adds none: two points a column. Each told against where the top's taken
            // alone: another's added there could hide it again.)
            const taken = shared[b], more = [];
            if (start >= 0) for (const item of stack) {
              if (taken.every((i) => item.inView[i] - item.inView[i] === 0)) continue;
              let first = -1, last = -1;
              for (let i = from; i <= to; i++) if (item.inView[i] - item.inView[i] === 0) (first < 0 && (first = i)), (last = i);
              if (first >= 0) more.push(first, last);
            }
            if (more.length) shared[b] = [...new Set([...taken, ...more])].sort((a, z) => a - z);
            if (end >= 0) (lastIn = b), (latest = end);
          }
          if (lastIn >= 0 && shared[lastIn].at(-1) !== latest) shared[lastIn].push(latest);
        }
        for (const item of stack) {
          const points = new Array(B);
          for (let b = 0; b < B; b++) {
            const at = [];
            for (const i of shared[b]) if (item.inView[i] - item.inView[i] === 0) at.push(/** @type {[number, number]} */ ([i, item.inView[i]]));
            points[b] = at.length ? at : null;
          }
          item.points = points;
        }
      }
      let lo = Infinity, hi = -Infinity;
      for (const { layer, mode, bars, base, points, stack, unseen } of shaped) {
        // (One out of the fit: drawn on the others' scale, see unfitted.)
        if (unfitted(doc, layer)) continue;
        // (An unseen layer's values alone, where bubbles sit: its bars', its stack's floor, not drawn.)
        if (bars && !unseen) (lo = Math.min(lo, base)), (hi = Math.max(hi, base));
        if (zeroFit(doc, layer, mode, Boolean(stack)) && !unseen) (lo = Math.min(lo, 0)), (hi = Math.max(hi, 0));
        for (const point of points) for (const [, value] of point ?? []) (lo = Math.min(lo, value)), (hi = Math.max(hi, value));
      }
      // (A layer bubbles ride, its values as they ride it, its line's breaks not theirs: the chart's scale reads them, see
      // Bubbles.)
      for (const item of shaped) {
        const values = carried.has(item.layer.id) ? stacked.get(item.layer.id)?.values ?? data.results.get(item.layer.id)?.values : null;
        for (let i = first; values && i <= last; i++) ((value) => Number.isFinite(value) && ((lo = Math.min(lo, value)), (hi = Math.max(hi, value))))(item.scaled(values[i]));
      }
      // (A heatmap's days, behind its side's lines; alone there (or beside lines that give its scale no height), on
      // the price's closes too, as the chart scales it.)
      const heats = doc.layers.filter((layer) => layer.pane === pane.id && layer.side === side && heat.has(layer.id));
      if (!Number.isFinite(lo) || lo === hi) for (const layer of heats) {
        const closes = data.results.get(layer.id)?.closes;
        if (!closes) continue;
        const [low, high] = rangeOf(onScaleAll(closes, modeOf(doc, layer), NaN, first, last));
        (lo = Math.min(lo, low)), (hi = Math.max(hi, high));
      }
      // (A side of a pane's shade alone: drawn, from the pane's edges.)
      if (!Number.isFinite(lo) && !shaped.some((item) => item.layer.type === "cloud" && edgesOf(item.layer).length > 1)) continue;
      if (!Number.isFinite(lo)) (lo = 0), (hi = 1);
      // (One value all along (a flat line, closes): half a unit either side of it, it in the middle as before.)
      if (lo === hi) (lo -= 0.5), (hi += 0.5);
      if (pane.stack === "percent") (lo = 0), (hi = 100);
      const range = hi - lo;
      const y = (value) => round(band.top + band.height * (range ? 1 - (value - lo) / range : 0.5));
      const inPane = (value) => clamp(value, paneTop, paneBottom);
      // (Each heatmap's, at the back of its pane: its scale's, its price at each place where the side's lines read it.)
      for (const layer of heats) {
        if (layer.type !== "heatmap") continue;
        const img = range ? heatPicture(doc, layer, data, /** @type {any} */ (heat.get(layer.id)), (place) => ((value) => (modeOf(doc, layer) === 1 ? 10 ** value : value))(lo + (1 - (place - band.top) / band.height) * range), paneTop, paneBottom) : null;
        // (Behind every other, in the pane's own order among themselves, as the chart draws them: see the sort below.)
        if (!img) continue;
        const mark = { k: "heat", c: layer.color, d: "", img, top: paneTop, bottom: paneBottom };
        drawn.marks.push(mark), ranks.set(mark, (zOf.get(layer.id) ?? 0) - doc.layers.length - 1);
      }
      // (Each profile's day (the last shown, as the chart's), its groups' bars from the right edge as the chart draws
      // them (see the library's columnWidth): that wide at the most, binned by a few units, strong under the close, faint
      // over it, a group a mark of each (see previewSvg); in the pane's order, as a series. Cumulative, each side's
      // amounts summed from the close outward, the groups stacked, an area each (see the library's Profile).)
      for (const layer of heats) {
        const sample = heat.get(layer.id), scaled = onScale(modeOf(doc, layer), NaN), grid = HEAT_PICTURE_GRID, measure = measureOf(layer);
        const groups = layer.type === "profile" && range ? (layer.profile ?? []).map((cohort, c) => ({ cohort, day: sample?.days[c] })).filter((item) => item.day?.rows) : [];
        if (!groups.length) continue;
        const BIN = 4, reach = columnWidth(W), bins = Math.max(1, Math.ceil((paneBottom - paneTop) / BIN)), count = groups.length, sums = new Float64Array(bins * count), totals = new Float64Array(bins);
        // (Cumulative: the amounts between the close and the pane's edge, out of view, each side's start.)
        const split = y(scaled(Number(groups[0].day.close))), over = new Float64Array(count), under = new Float64Array(count);
        groups.forEach(({ day }, c) => {
          const values = amountsOf(day, measure);
          for (let k = 0; k < day.rows.length; k++) {
            const at = y(scaled(grid.price(day.rows[k] + 0.5)));
            if (at < paneTop && split < at) under[c] += Math.abs(values[k]);
            else if (at >= paneBottom && at < split) over[c] += Math.abs(values[k]);
            if (!(at >= paneTop && at < paneBottom)) continue;
            const b = Math.floor((at - paneTop) / BIN), amount = Math.abs(values[k]);
            sums[b * count + c] += amount;
            totals[b] += amount;
          }
        });
        const most = totals.reduce((a, b) => Math.max(a, b), 0), paths = groups.map(() => ["", ""]), op = opacityOf(layer);
        if (!(most > 0) && !layer.cumulative) continue;
        if (layer.cumulative) {
          // (Each side's heights from the close (or the pane's edge) outward, a bin's far edge each, by its part on
          // that side; then each group's area, along its curve out and back along the one under it.)
          const line = clamp(split, paneTop, paneBottom), sides = [true, false].map((down) => {
            const run = Float64Array.from(down ? under : over), ys = [line], runs = [run.slice()];
            for (let i = 0; i < bins; i++) {
              const b = down ? i : bins - 1 - i, top = paneTop + b * BIN, bottom = Math.min(paneBottom, top + BIN), from = down ? Math.max(top, line) : top, to = down ? bottom : Math.min(bottom, line);
              if (!(to > from)) continue;
              for (let c = 0; c < count; c++) run[c] += (sums[b * count + c] * (to - from)) / (bottom - top);
              ys.push(down ? to : from), runs.push(run.slice());
            }
            return { down, ys, runs };
          });
          const whole = Math.max(...sides.map(({ runs }) => runs[runs.length - 1].reduce((a, b) => a + b, 0)));
          if (!(whole > 0)) continue;
          for (const { down, ys, runs } of sides) {
            const x = (v) => round(W - (v / whole) * reach), stacked = new Float64Array(ys.length);
            for (let c = 0; c < count; c++) {
              const out = ys.map((at, k) => `${x(stacked[k] + runs[k][c])} ${round(at)}`), back = ys.map((at, k) => `${x(stacked[k])} ${round(at)}`).reverse();
              if (runs[runs.length - 1][c]) paths[c][down ? 0 : 1] += `M${out.join("L")}L${back.join("L")}Z`;
              ys.forEach((_, k) => (stacked[k] += runs[k][c]));
            }
          }
        } else for (let b = 0; b < bins; b++) {
          if (!totals[b]) continue;
          const top = round(paneTop + b * BIN), bottom = round(Math.min(paneBottom, paneTop + (b + 1) * BIN - 1)), faint = (top + bottom) / 2 > split ? 0 : 1;
          for (let c = 0, x = W; c < count; c++) {
            const length = (sums[b * count + c] / most) * reach;
            if (length) paths[c][faint] += `M${round(x - length)} ${top}H${round(x)}V${bottom}H${round(x - length)}Z`;
            x -= length;
          }
        }
        groups.forEach(({ cohort }, c) => paths[c].forEach((d, faint) => d && ((mark) => (drawn.marks.push(mark), ranks.set(mark, zOf.get(layer.id) ?? 0)))({ k: "profile", c: profileColor(layer, cohort), d, f: [faint ? 0.32 : 0.72], top: paneTop, bottom: paneBottom, ...(op < 1 ? { op } : {}) })));
      }
      // (Quick Look's: the side's highest and lowest drawn, where they are, as the scale reads them: in their unit;
      // a change from the first value in view, as a percent; an index, from 100; a 100% stack's, its share.)
      if (big && drawn.scales) {
        let most = null, least = null;
        for (const item of shaped) {
          if (item.unseen) continue;
          for (const point of item.points) {
            // (A bar's drawn to its far end from its base: that end alone.)
            const ends = item.bars && point ? [point.reduce((far, next) => (Math.abs(next[1] - item.base) > Math.abs(far[1] - item.base) ? next : far))] : point ?? [];
            for (const [, at] of ends) {
              if (!Number.isFinite(at) || !Number.isFinite(y(at))) continue;
              if (!most || at > most.at) most = { at, item };
              if (!least || at < least.at) least = { at, item };
            }
          }
        }
        const text = (/** @type {{ at: number, item: any }} */ end) => (pane.stack === "percent" ? format(end.at, "percent") : end.item.mode === 2 ? format(end.at * 100, "percent") : end.item.mode >= 3 ? precise(indexed(end.at * 100)) : format(end.item.mode === 1 ? 10 ** end.at : end.at, unitFor(end.item.layer, data.results.get(end.item.layer.id))));
        if (most) drawn.scales.push({ y: y(most.at), text: text(most), side });
        if (least && least.at !== most?.at) drawn.scales.push({ y: y(least.at), text: text(least), side });
      }
      const drawLayer = ({ layer, unseen, scaled, bars, base, level, points, band, bands }) => {
        if (unseen) return;
        // (A period's amount's last point, its period still running (see periodAmount): drawn partial, as on the chart,
        // where it's drawn: its bar faint, a line's segment to it dashed. Of its data, before it's put in buckets.)
        const partial = periodAmount(layer) ? data.results.get(layer.id)?.values?.findLastIndex(Number.isFinite) ?? -1 : -1;
        // (None stacked, nor on a % or indexed scale: see acrossUnfit.)
        const across = acrossUnfit(doc, layer) ? [] : [...(layer.levels ?? []).map((item, i) => [String(i), item.value]), ...(layer.type === "baseline" ? [["base", base]] : [])];
        for (const [key, value] of across) {
          const yAt = y(scaled(value)), { color, dash, line } = lineLook(layer, key), [, , length, gap] = dashOf(dash);
          if (line !== false && yAt > paneTop && yAt < paneBottom) drawn.marks.push({ k: "across", c: color, d: `M0 ${yAt}H${W}`, ...(gap ? { dash: `${length} ${gap}` } : {}), ...(opacityOf(layer) < 1 ? { op: opacityOf(layer) } : {}) });
        }
        const at = (/** @type {[number, number]} */ point) => `${xText(point[0])} ${y(point[1])}`;
        const look = { c: layer.color, ...(opacityOf(layer) < 1 ? { op: opacityOf(layer) } : {}), ...(layer.outline && OUTLINED.has(layer.type) ? { o: doc.layers.indexOf(layer) + 1 } : {}) };
        // (A stacked area's band, as the chart draws it (see stackBands): down to the nearest band or line under it with
        // a value there, else the stack's floor: zero, the pane's foot on a log scale.)
        const own = layer.type === "area" && layer.fill !== "above" ? stacked.get(layer.id) : undefined;
        const under = own ? [...stacked].filter(([id, item]) => item.group === own.group && item.rank < own.rank && ((band) => (band?.type === "area" && band.fill !== "above") || band?.type === "line")(doc.layers.find((item) => item.id === id))).map(([, item]) => item).sort((a, b) => b.rank - a.rank) : [];
        // (A loop, finite by a subtraction: no call a band, its every point. See stackOf.)
        const floorAt = own && ((/** @type {number} */ i) => {
          let value = 0;
          for (const item of under) if (item.values[i] - item.values[i] === 0) {
            value = item.values[i];
            break;
          }
          const at = y(scaled(value));
          return at - at === 0 && at < paneBottom ? at : paneBottom;
        });
        /** Its floor, back along a run of its points from its last. @param {[number, number][]} part */
        const backAlong = (part) => {
          let back = "";
          for (let p = part.length - 1; p >= 0; p--) back += `L${xText(part[p][0])} ${/** @type {(i: number) => number} */ (floorAt)(part[p][0])}`;
          return back;
        };
        // (A line or an area colored by its levels, not stacked: by value, as on the chart, each level's height and color
        // (low to high: its color from there up, see previewSvg). Else painted date by date, as on the chart: a mark for
        // each of its colors, its pieces in that color.)
        const zones = layer.byLevels && layer.levels?.length && (layer.type === "line" || layer.type === "area") && !stacked.has(layer.id) ? levelSteps(layer).map((step) => /** @type {[number, string]} */ ([y(scaled(step.value)), step.color])) : null;
        const dateColors = zones ? null : paintOf(doc, layer, data), tintAt = (/** @type {number} */ i) => (dateColors && dateColors(Math.round(i))) ?? layer.color;
        /** @template T @param {{ color: string, points: T[] }[]} pieces @returns {Map<string, T[][]>} */
        const perColor = (pieces) => {
          const by = new Map();
          for (const piece of pieces) (by.get(piece.color) ?? by.set(piece.color, []).get(piece.color))?.push(piece.points);
          return by;
        };
        /** The buckets with something, run by run (each unbroken). @param {any[]} buckets */
        const runsOf = (buckets) => {
          /** @type {number[][]} */
          const runs = [];
          let run = [];
          buckets.forEach((item, b) => (item ? run.push(b) : run.length && (runs.push(run), (run = []))));
          if (run.length) runs.push(run);
          return runs;
        };
        /** Runs one after another, a gap between each (for byColor). @template T @param {T[][]} parts @returns {(T | null)[]} */
        const joined = (parts) => parts.flatMap((part, k) => (k ? [null, ...part] : part));
        /**
         * A band, run by run, its highs along, its lows back (±Infinity, or its pane's edges (`edged`), there), each
         * bucket in its color (`tint`). @param {([number, number] | null)[]} ranges @param {(b: number) => string} tint
         */
        const drawBand = (ranges, tint, edged = true) => {
          const above = (b) => ((edged && layer.from?.includes("top")) || /** @type {[number, number]} */ (ranges[b])[1] === Infinity ? paneTop : y(/** @type {[number, number]} */ (ranges[b])[1])), below = (b) => ((edged && layer.from?.includes("bottom")) || /** @type {[number, number]} */ (ranges[b])[0] === -Infinity ? paneBottom : y(/** @type {[number, number]} */ (ranges[b])[0]));
          const middle = (/** @type {number} */ b) => x(middleOf(b));
          for (const [c, parts] of perColor(byColor(joined(runsOf(ranges)), tint))) {
            const d = parts.map((part) => `M${part.map((b) => `${middle(b)} ${above(b)}`).join("L")}L${[...part].reverse().map((b) => `${middle(b)} ${below(b)}`).join("L")}Z`).join("");
            if (d) drawn.marks.push({ ...look, c, k: "band", d });
          }
        };
        if (bands) return void bands.forEach(({ band: ranges, tint }) => drawBand(ranges, tint, false));
        if (band) return void drawBand(band, (b) => tintAt(middleOf(b)));
        if (bars) {
          let d = "", d2 = "";
          const painted = new Map();
          // A bar a bucket, at its middle, to the one of its extremes farthest from the base (painted: in its date's
          // color, whichever side). A period's amount's last, its period still running, faint apart, as the chart's.
          points.forEach((point, b) => {
            if (!point) return;
            const [i, value] = point.reduce((far, next) => (Math.abs(next[1] - base) > Math.abs(far[1] - base) ? next : far));
            const bar = `M${x(middleOf(b))} ${y(base)}V${y(value)}`;
            if (i === partial) drawn.marks.push({ ...look, c: dateColors ? tintAt(i) : value >= base ? layer.color : belowOf(layer), k: "bars", d: bar, op: (look.op ?? 1) * PARTIAL_SHARE });
            else if (dateColors) painted.set(tintAt(i), (painted.get(tintAt(i)) ?? "") + bar);
            else if (value >= base) d += bar;
            else d2 += bar;
          });
          for (const [c, path] of painted) drawn.marks.push({ ...look, c, k: "bars", d: path });
          if (d || d2) drawn.marks.push({ ...look, k: "bars", c2: belowOf(layer), d, ...(d2 ? { d2 } : {}) });
          return;
        }
        // Its runs: the buckets with a value, unbroken.
        const runs = runsOf(points);
        if (!runs.length) return;
        const whole = (/** @type {number[]} */ part) => part.flatMap((b) => /** @type {[number, number][]} */ (points[b]));
        // (A line's partial point (see `partial`), as the chart draws it: the segment to it from the point before it
        // (no gap between) dashed apart, in its color there, the rest without it.)
        const final = whole(runs[runs.length - 1]).slice(-2), tail = layer.type === "line" && final.length === 2 && final[1][0] === partial && final[0][0] === partial - 1 ? final : null;
        const along = (/** @type {number[]} */ part) => (tail ? whole(part).filter(([i]) => i !== partial) : whole(part));
        // (Its latest value's dot, in the color it's drawn in there.)
        // (Painted date by date, its pieces by color; else one a run.)
        const pieces = (dateColors ? byColor(joined(runs.map(along)), ([i]) => tintAt(i), layer.type !== "dots") : runs.map((run) => ({ color: layer.color, points: along(run) }))).filter((piece) => piece.points.length), last = pieces.at(-1);
        if (!last) return;
        const [endAt, endValue] = /** @type {[number, number]} */ (tail ? tail[1] : last.points.at(-1));
        const [, , length, gap] = LINED.has(layer.type) ? dashOf(layer.dash) : DASHES[0];
        // (The latest value's color last: its dot over the rest.)
        for (const [c, parts] of [...perColor(pieces)].sort(([a], [b]) => Number(a === last.color) - Number(b === last.color))) {
          if (layer.type === "dots") {
            drawn.marks.push({ ...look, c, k: "dots", d: parts.flat().map((point) => `M${at(point)}h0`).join("") });
            continue;
          }
          // (Each part's points written once: its line's, and a band's way along them.)
          const written = parts.map((part) => part.map(at).join("L"));
          const line = { ...look, c, d: written.map((points) => `M${points}`).join(""), ...(c === last.color && !tail ? { end: [x(endAt), y(endValue)] } : {}), ...(gap ? { dash: `${length} ${gap}` } : {}), ...(zones ? { z: zones } : {}) };
          if (layer.type === "area") drawn.marks.push({ ...line, k: "area", top: paneTop, bottom: paneBottom, f: fillOf(layer, Boolean(floorAt)), ...(layer.fill === "above" ? { up: true } : {}), ...(floorAt ? { band: parts.map((part, k) => `M${written[k]}${backAlong(part)}Z`).join("") } : {}) });
          else if (layer.type === "baseline") drawn.marks.push({ ...line, k: "baseline", c2: belowOf(layer), y0: Number.isFinite(level) ? inPane(y(level)) : paneBottom, top: paneTop, bottom: paneBottom });
          else drawn.marks.push({ ...line, k: "line" });
        }
        if (tail) drawn.marks.push({ ...look, c: dateColors ? tintAt(partial) : layer.color, k: "line", d: `M${at(tail[0])}L${at(tail[1])}`, dash: "3 3", end: [x(endAt), y(endValue)], ...(zones ? { z: zones } : {}) });
      };
      for (const item of shaped) {
        const start = drawn.marks.length;
        drawLayer(item);
        for (let m = start; m < drawn.marks.length; m++) ranks.set(drawn.marks[m], drawn.marks[m].k === "across" ? -1 : zOf.get(item.layer.id) ?? 0);
      }
      // (Bubbles on a layer drawn here, at its value there on this side's scale: as the chart reads it, its line's
      // breaks not theirs.)
      for (const layer of doc.layers) {
        const on = layer.type === "bubbles" && layer.pane === pane.id && shaped.find((item) => item.layer.id === layer.ref), values = on && (stacked.get(on.layer.id)?.values ?? data.results.get(on.layer.id)?.values);
        if (on && values) drawBubbles(layer, (i) => y(on.scaled(values[i])));
      }
    }
    // (Bubbles on none: along the pane's middle, whatever its sides draw.)
    for (const layer of doc.layers) if (layer.type === "bubbles" && layer.pane === pane.id && !doc.layers.some((other) => other.id === layer.ref)) drawBubbles(layer, () => round((paneTop + paneBottom) / 2));
    drawn.marks.splice(from, Infinity, ...drawn.marks.slice(from).sort((a, b) => ranks.get(a) - ranks.get(b)));
    panes.push({ from: sources, marks: drawn.marks.slice(from), scales: drawn.scales?.slice(scalesFrom) ?? [] });
  });
  panesDrawn.set(data, { look, times, panes });
  return drawn;
}
/**
 * A preview's levels (`z`: each its height and color, low to high, a color from its height up) as the parts of a
 * picture `height` tall each holds, top down; under them all, its own (`c`, the last). A level whose height isn't a
 * number (on a log scale, at 0 or under) holds all from there up.
 * @param {[number, string][]} z @param {number} height @param {string} own @returns {{ y: number, h: number, c: string }[]}
 */
function zoneRects(z, height, own) {
  const parts = [];
  let foot = height, under = own;
  for (const [at, color] of z) {
    const y = Number.isFinite(at) ? Math.min(Math.max(at, 0), height) : height;
    if (y >= foot) {
      under = color;
      continue;
    }
    parts.push({ y, h: foot - y, c: under });
    (foot = y), (under = color);
  }
  parts.push({ y: 0, h: foot, c: under });
  return parts.reverse().filter((part) => part.h > 0);
}
/** Each subpath of a line closed down (or up) to `edge`: the fill under it. */
const closedTo = (d, edge) => d.split("M").filter(Boolean).map((part) => `M${part.split(" ")[0]} ${edge}L${part}L${part.slice(part.lastIndexOf("L") + 1).split(" ")[0]} ${edge}Z`).join("");
/**
 * Dates along a picture's foot, `width` wide, over `from`–`to` (bars of `times`): years for years (every few, past
 * eight), months for months, days for days, hours for hours; where each falls.
 */
function dateTicks(/** @type {ArrayLike<number>} */ times, /** @type {number} */ from, /** @type {number} */ to, /** @type {number} */ width) {
  const n = times.length, start = Math.max(0, Math.ceil(from)), end = Math.min(n - 1, Math.floor(to)), span = to - from;
  if (end <= start || !(span > 0)) return [];
  const days = (times[end] - times[start]) / 86400, unit = days > 730 ? "year" : days > 60 ? "month" : days > 2 ? "day" : "hour";
  const keyOf = (/** @type {number} */ time) => ((date) => (unit === "year" ? date.getUTCFullYear() : unit === "month" ? date.getUTCFullYear() * 12 + date.getUTCMonth() : unit === "day" ? Math.floor(time / 86400) : Math.floor(time / 3600)))(new Date(time * 1000));
  const all = [];
  for (let i = start + 1, was = keyOf(times[start]); i <= end; i++) {
    const key = keyOf(times[i]);
    if (key !== was) all.push(i);
    was = key;
  }
  const step = Math.max(1, Math.ceil(all.length / 8));
  return all.filter((_, k) => k % step === 0).map((i) => {
    const date = new Date(times[i] * 1000);
    const text = unit === "year" ? String(date.getUTCFullYear()) : unit === "hour" ? date.toLocaleTimeString("en", { hour: "numeric", minute: "2-digit", timeZone: "UTC" }) : date.toLocaleDateString("en", unit === "month" ? { month: "short", ...(date.getUTCMonth() === 0 ? { year: "numeric" } : {}), timeZone: "UTC" } : { month: "short", day: "numeric", timeZone: "UTC" });
    return { x: Math.round(((i - from) / span) * width * 10) / 10, text };
  });
}
/**
 * A drawing as an SVG: panes parted by a hairline, each layer as its type (an area fading down from its line, a
 * baseline in its two colors about its base, a stacked band filled to the one under it, bars, dots), dashed as it is
 * (in Quick Look, a dot on its latest value). Its ids are its place's (`place`: a tile's, a card's, ⌘K's, Quick Look's), the same each time:
 * the same chart can show twice at once, and a tile drawn again is unchanged.
 * @param {Preview} drawn @param {string} place (`size`: a picture's, as previewOf drew it)
 */
function previewSvg(drawn, place, size = PREVIEW) {
  const { width: W, height: H } = size, id = place.replace(/[^\w-]/g, "-"), defs = [];
  const body = drawn.seps.map((y) => `<path class="sep" d="M0 ${y}H${W}"/>`);
  // (Each mark's fill, its band if it's outlined (see svg.plot .halo), its line; a layer's (one `o`) together, as the
  // chart draws them: every fill, every band, then every line, a band of one of its colors never over another's
  // line; a band whole, as the chart's ground is, its layer's opacity on its fill and line alone.)
  const parts = drawn.marks.map((mark, n) => {
    const c = escapeHtml(ink(mark.c)), c2 = escapeHtml(ink(mark.c2 ?? mark.c)), g = `${id}-${n}`, dash = mark.dash ? ` stroke-dasharray="${mark.dash}"` : "";
    const halo = (/** @type {string | undefined} */ d, kind = "") => (mark.o && d ? `<path class="halo${kind}"${dash} d="${d}"/>` : "");
    const fade = (key, color, from, to, a, b) => `<linearGradient id="${g}${key}" gradientUnits="userSpaceOnUse" x1="0" x2="0" y1="${from}" y2="${to}"><stop offset="0" stop-color="${color}" stop-opacity="${a}"/><stop offset="1" stop-color="${color}" stop-opacity="${b}"/></linearGradient>`;
    // (`ring`: a mark's own band, right under its line, never faded: an outlined bubble's.)
    let under = "", band = "", over = "", ring = "";
    // (A heatmap's picture, its pixels as they are: smoothed, a cell bled into the empty ones by it, a dark fringe.)
    if (mark.k === "heat") under = `<image href="${mark.img}" x="0" y="${mark.top}" width="${W}" height="${/** @type {number} */ (mark.bottom) - /** @type {number} */ (mark.top)}" preserveAspectRatio="none" style="image-rendering: pixelated"/>`;
    else if (mark.k === "across") over = `<path class="across" stroke="${c}"${dash} d="${mark.d}"/>`;
    // (A profile's group's bars, strong or faint (`f`), mixed over the ground, solid as the chart's.)
    else if (mark.k === "profile") under = `<path class="fill" style="fill:color-mix(in srgb, ${c} ${Math.round((mark.f?.[0] ?? 1) * 100)}%, var(--plot-ground, var(--card)))" d="${mark.d}"/>`;
    else if (mark.k === "band") under = `<path class="fill" style="fill:${c}" d="${mark.d}"/>`;
    else if (mark.k === "bars") (band = `${halo(mark.d, " of-bars")}${halo(mark.d2, " of-bars")}`), (over = `<path class="bars" stroke="${c}" d="${mark.d}"/>${mark.d2 ? `<path class="bars" stroke="${c2}" d="${mark.d2}"/>` : ""}`);
    else if (mark.k === "dots") (band = halo(mark.d, " of-dots")), (over = `<path class="dots" stroke="${c}" d="${mark.d}"/>`);
    // (Bubbles of a size: each its edge's dot, its fill's dot a pixel in over it, both mixed with the ground (the chart's
    // are see-through); a dot keeps its size as the frame stretches, so each is round. Outlined: a ring of the ground
    // under each size's, over the larger ones', as the chart rings each in turn. Clipped to its pane, as the chart's.)
    else if (mark.k === "bubble") (defs.push(`<clipPath id="${g}c"><rect y="${mark.top}" width="${W}" height="${/** @type {number} */ (mark.bottom) - /** @type {number} */ (mark.top)}"/></clipPath>`), (ring = mark.o ? `<g clip-path="url(#${g}c)">${[mark.d, mark.d2].map((d) => (d ? `<path class="bubble" style="stroke-width:${(mark.w ?? 0) + 2}px;stroke:var(--plot-ground, var(--card))" d="${d}"/>` : "")).join("")}</g>` : ""), (over = `<g clip-path="url(#${g}c)">${[[c, mark.d], [c2, mark.d2]].map(([color, d]) => (d ? `<path class="bubble" style="stroke-width:${mark.w}px;stroke:color-mix(in srgb, ${color} 80%, var(--plot-ground, var(--card)))" d="${d}"/>${(mark.w ?? 0) > 3 ? `<path class="bubble" style="stroke-width:${(mark.w ?? 0) - 2}px;stroke:color-mix(in srgb, ${color} 40%, var(--plot-ground, var(--card)))" d="${d}"/>` : ""}` : "")).join("")}</g>`));
    else if (mark.k === "baseline") {
      const y0 = /** @type {number} */ (mark.y0), f = closedTo(mark.d, y0);
      defs.push(`<clipPath id="${g}a"><rect width="${W}" height="${y0}"/></clipPath><clipPath id="${g}b"><rect y="${y0}" width="${W}" height="${H - y0}"/></clipPath>`, fade("u", c, mark.top, y0, 0.24, 0.01), fade("v", c2, y0, mark.bottom, 0.01, 0.24));
      under = `<g clip-path="url(#${g}a)"><path class="fill" style="fill:url(#${g}u)" d="${f}"/></g><g clip-path="url(#${g}b)"><path class="fill" style="fill:url(#${g}v)" d="${f}"/></g>`;
      band = halo(mark.d);
      over = `<g clip-path="url(#${g}a)"><path stroke="${c}"${dash} d="${mark.d}"/></g><g clip-path="url(#${g}b)"><path stroke="${c2}"${dash} d="${mark.d}"/></g>`;
    } else {
      // (Its fill as the chart's: see fillOf. A preview kept from before, the defaults then.)
      const [top, foot] = mark.f ?? (mark.up ? [0.12, 0.12] : mark.band ? [STACK_FILL, STACK_FILL] : [0.2, 0]);
      // (Colored by its levels (`z`, low to high, each its height and its color from there up): its line and its fill
      // parted at each, each part clipped to its zone, as the chart's; else one color.)
      const zones = mark.z ? zoneRects(mark.z, H, mark.c) : [{ y: 0, h: H, c: mark.c }];
      zones.forEach((zone, k) => {
        const zc = escapeHtml(ink(zone.c)), clip = mark.z ? ` clip-path="url(#${g}z${k})"` : "";
        if (mark.z) defs.push(`<clipPath id="${g}z${k}"><rect y="${zone.y}" width="${W}" height="${zone.h}"/></clipPath>`);
        if (mark.k === "area" && mark.up) under += `<path class="fill"${clip} style="fill:color-mix(in srgb, ${zc} ${top * 100}%, transparent)" d="${closedTo(mark.d, mark.top)}"/>`;
        else if (mark.k === "area" && mark.band) under += `<path class="fill"${clip} style="fill:color-mix(in srgb, ${zc} ${top * 100}%, transparent)" d="${mark.band}"/>`;
        else if (mark.k === "area") {
          defs.push(fade(`u${k}`, zc, mark.top, mark.bottom, top, foot));
          under += `<path class="fill"${clip} style="fill:url(#${g}u${k})" d="${closedTo(mark.d, mark.bottom)}"/>`;
        }
        over += `<path stroke="${zc}"${clip}${dash} d="${mark.d}"/>`;
      });
      band = halo(mark.d);
    }
    // (Its end marked in Quick Look's picture alone, by its value beside it; a card's line just ends.)
    if (mark.end && size !== PREVIEW) {
      const [x, y] = mark.end, color = mark.k === "baseline" && y > /** @type {number} */ (mark.y0) ? c2 : mark.z ? escapeHtml(ink(zoneRects(mark.z, H, mark.c).find((zone) => y >= zone.y && y <= zone.y + zone.h)?.c ?? mark.c)) : c;
      over += `<path class="ring" d="M${x} ${y}h0"/><path class="end" stroke="${color}" d="M${x} ${y}h0"/>`;
    }
    const faded = (/** @type {string} */ html) => (html && mark.op ? `<g opacity="${mark.op}">${html}</g>` : html);
    return { o: mark.o, under: faded(under), band, over: ring + faded(over) };
  });
  for (let n = 0; n < parts.length; ) {
    let end = n + 1;
    while (parts[n].o && end < parts.length && parts[end].o === parts[n].o) end++;
    const run = parts.slice(n, end);
    body.push(...run.map((part) => part.under), ...run.map((part) => part.band), ...run.map((part) => part.over));
    n = end;
  }
  return `<svg class="plot" viewBox="0 0 ${W} ${H}" preserveAspectRatio="none" aria-hidden="true">${defs.length ? `<defs>${defs.join("")}</defs>` : ""}${body.join("")}</svg>`;
}
/**
 * Each drawing's own number, as a frame records the one it shows (`data-drawn`): two of one chart, at one key (a
 * tile's from weekly data, the chart's as it's left), told apart. @type {WeakMap<object, number>}
 */
const drawingIds = new WeakMap();
let drawingsNumbered = 0;
const drawingId = (/** @type {object} */ drawn) => drawingIds.get(drawn) ?? (drawingIds.set(drawn, ++drawingsNumbered), drawingsNumbered);
/**
 * The mini charts being drawn (a chart's tile, a card's, ⌘K's), by what they're drawn of: one ask at a time each, where
 * it's framed (its frames, and a preset's slot: its tile, written once near sight, see slotHtml), and what gives it up
 * (see giveUpUnframed).
 * @type {Map<string, { framed: string, drawn: Promise<(Preview & { key: string }) | null>, stop: AbortController }>}
 */
const plotsDrawing = new Map();
/**
 * Every drawing under way with no frame of its chart left in the page, nor a slot for its tile (its tile dropped by a key
 * typed, ⌘K's preview moved on, a page of tiles put aside) given up, its fetches no other asker wants with it (see cachedLoad). Called
 * once a page of frames is written anew. Out of the map before it's aborted: an ask meanwhile (the tile back) draws
 * anew rather than joining one that will paint nothing.
 */
function giveUpUnframed() {
  for (const [ask, { framed, stop }] of plotsDrawing) if (!document.querySelector(framed)) plotsDrawing.delete(ask), stop.abort();
}
/**
 * A chart's drawing, shown wherever it's framed (its tile, a template's card, ⌘K's preview): in the frames still
 * waiting, or (`all`: a new drawing) in all of them; none (a failed fetch) ends their wait.
 * @param {string} id @param {Preview | null} drawn
 */
function paint(id, drawn, all = false) {
  if (!drawn) drawnTiles.delete(id);
  for (const frame of all$(`[data-plot="${CSS.escape(id)}"]`)) {
    if (!all && !frame.classList.contains("pending")) continue;
    frame.classList.remove("pending");
    // (Which drawing it shows (see drawingId): a frame put back is painted again where it's another's, see renderGallery.)
    if (drawn) (frame.innerHTML = previewSvg(drawn, `${frame.classList[0]}-${id}`)), (frame.dataset.drawn = String(drawingId(drawn)));
    // (None, and no picture before it: said, not a blank that reads as no data; asked for again, see retryPlots.
    // One from before stays: a sketch of the same chart.)
    frame.classList.toggle("failed", !drawn && !frame.querySelector("svg"));
  }
}
/**
 * A chart's drawing, made if it's out of date, kept (and stored, a moment later), and shown wherever it's framed:
 * from `index`'s data (a template's weekly: enough at this size, over as much time as its chart shows). null when a
 * fetch failed, or the drawing did (reported): nothing is kept, the next time tries again. Given up once no frame of
 * its chart is left (see giveUpUnframed): nothing painted, null.
 * @param {Chart} doc @returns {Promise<(Preview & { key: string }) | null>}
 */
function drawPreview(doc, index = doc.index) {
  const key = previewKey(doc), have = previews.get(doc.id);
  if (have?.key === key) return paint(doc.id, have), Promise.resolve(have);
  // Asked again while it's being drawn (⌘K's highlight, the grid), the same drawing answers.
  const ask = JSON.stringify([doc.id, key, index]), going = plotsDrawing.get(ask);
  if (going) return going.drawn;
  const stop = new AbortController(), signal = stop.signal, preset = doc.id.startsWith("preset:") ? Number(doc.id.slice(7)) : -1, at = preset >= 0 ? preset : presetIndex(doc);
  const framed = `[data-plot="${CSS.escape(doc.id)}"]${at >= 0 ? `, .tile-slot[data-slot="${at}"]` : ""}`;
  const drawing = (async () => {
    index = await sampleable(doc, index, signal);
    const copy = { ...structuredClone(doc), index };
    const data = await resolve(copy, false, signal).catch(() => null);
    if (!data || signal?.aborted || retryFailed(data, index) || previewKey(doc) !== key) return null;
    // (Its heatmaps' days, a column of the tile each few pixels: see heatSamples.)
    const [heat, bubbles] = await Promise.all([heatSamples(copy, data, previewBars(doc, index), HEAT_TILE_COLUMNS), bubblesAt(doc, data, index, signal)]);
    if (!bubbles || previewKey(doc) !== key) return null;
    // Replaced meanwhile (another tab's edit): its successor is drawn on its own.
    if (!/^(recommended|preset):/.test(doc.id) && chartById(doc.id) !== doc) return null;
    // (A preset let go meanwhile, out of the last looked at: its drawing isn't kept either.)
    if (preset >= 0 && !presetDocs.has(preset)) return null;
    // (A preset's: which one, by its place, see templateDoc.)
    // (Its heatmaps' days not all in: shown, kept unkeyed, drawn again the next time it's asked for.)
    const drawn = { key: heat.whole === false ? "" : key, ...previewOf(copy, data, previewBars(doc, index), PREVIEW, heat, bubbles), ...(preset >= 0 ? { of: presetKey(preset) } : {}) };
    // (The last drawn last: kept first, see savePreviews.)
    previews.delete(doc.id);
    previews.set(doc.id, drawn);
    savePreviewsSoon();
    return drawn;
  })().catch((error) => (reportError(new Error(`${doc.id}: its drawing failed`, { cause: error })), null))
    .then((made) => (signal.aborted || paint(doc.id, made, true), made)).finally(() => plotsDrawing.get(ask)?.drawn === drawing && plotsDrawing.delete(ask));
  plotsDrawing.set(ask, { framed, drawn: drawing, stop });
  return drawing;
}
/**
 * A picture's interval as asked for, else the chart's own (a series without it: one only by the day): its series'
 * intervals the catalog's (indexed just after load), else their infos' asked for first (see intervalsOf).
 * @param {Chart} doc @param {string} index
 */
async function sampleable(doc, index, signal = /** @type {AbortSignal | undefined} */ (undefined)) {
  if (index !== doc.index && !seriesIn(doc).every(intervalsOf)) await Promise.allSettled(seriesIn(doc).map((name) => info(name, signal)));
  return supportedIntervals(doc)(index) ? index : doc.index;
}
/** A preset's drawing is made from weekly data (its own, where a window runs: it'd read otherwise). @param {Chart} doc */
const previewIndex = (/** @type {Chart} */ doc) => sampleIndex(doc, "week1", 100);
/**
 * The interval a small picture of a chart reads (a mini chart's, from months, for its few points; a preview's, from
 * weeks): the coarsest its range still gives `points` at (3M, days; 1Y, weeks), its own where that's coarser or a
 * formula runs a window over it. @param {Chart} doc @param {string} coarsest @param {number} points
 */
function sampleIndex(doc, coarsest, points) {
  if (INTERVAL_ORDER.indexOf(doc.index) >= INTERVAL_ORDER.indexOf(coarsest) || doc.layers.some((layer) => runsWindow(layer.formula ?? ""))) return doc.index;
  const days = rangeDays(doc.range);
  return [coarsest, "week1", "day1"].find((id) => !days || days / INTERVAL_DAYS[id] >= points) ?? "day1";
}
/** A tile's plot: the chart drawn, a blank box for an empty one, a shimmer until it's drawn. @param {Chart} doc */
function plotHtml(doc, current = false, frame = "tile-plot") {
  const drawn = previews.get(doc.id);
  // (An empty one's box, blank: its legend's line says so.)
  if (!doc.layers.length) return `<span class="tile-plot blank"></span>`;
  // (`current`: one drawn before it changed is drawn again, as it is now, once in view.)
  // (`frame`: its classes, the first naming its drawing's ids: two frames of one chart, a hidden one's ids not the
  // shown one's.)
  return `<span class="${frame} ${drawn && (!current || drawn.key === previewKey(doc)) ? "" : "pending"}" data-plot="${doc.id}"${drawn ? ` data-drawn="${drawingId(drawn)}"` : ""}>${drawn ? previewSvg(drawn, `${frame.split(" ")[0]}-${doc.id}`) : ""}</span>`;
}
/** A layer in a tile's legend (Quick Look's: `after`, its value's place): its color's dot (a gradient's, its colors), its name. @param {Layer} layer */
const legendEntry = (doc, layer, after = "") => `<span style="--c:${escapeHtml(((stops) => (stops ? gradientCss(stops) : ink(layer.color)))(gradientOfLayer(doc, layer)))}">${escapeHtml(viewName(doc, layer))}${after}</span>`;
/** A tile's interval, where it isn't a day's. @param {Chart} doc */
const tileIndexHtml = (doc) => (doc.index === "day1" ? "" : `<span class="tile-index">${INTERVAL_LABEL[doc.index]}</span>`);
/** A card's legend: its shown layers' entries. @param {Chart} doc */
const cardLegend = (doc) => doc.layers.filter((layer) => !layer.hidden).map((layer) => legendEntry(doc, layer)).join("");
/** A chart in the grid, as its own card: title (and its interval, where it isn't a day's), its layers' legend, its mini chart; a tick to pick it. @param {Chart} doc */
function tileHtml(doc) {
  const title = escapeHtml(titleOf(doc)), on = picked.has(doc.id), preview = isPreview(doc);
  const legend = cardLegend(doc);
  return `<div class="tile ${on ? "picked" : ""} ${preview ? "preview" : ""} ${preview && touched.has(doc) ? "edited" : ""}" data-home-card="${doc.id}" data-chart="${doc.id}">
            <button type="button" class="tile-open" data-open-chart="${doc.id}" aria-label="${title}" data-tip="${title}"><span class="tile-head"><span class="tile-name">${title}</span>${tileIndexHtml(doc)}</span><span class="tile-legend">${legend || (doc.layers.length ? "Every layer is hidden" : "No layers yet")}</span>${plotHtml(doc)}</button>
            ${moreCap("tile-more", "data-tile-menu", doc.id, titleOf(doc))}
            ${preview ? "" : `<input type="checkbox" class="pick-check tile-check" style="--tick:${escapeHtml(tickOn("ink"))}" aria-label="Select ${title}" data-tip="${on ? "Deselect" : "Select"}" ${on ? "checked" : ""} />`}
          </div>`;
}
/**
 * The previews the page shows (a draft, the empty chart in place of none, isn't one), as the search finds them (as it
 * finds charts).
 */
const previewsShown = () => foundOf(listedPreviews());
/** Charts as the page's search finds them (as it finds yours), the best first; nothing searched, all of them. @param {Chart[]} docs */
function foundOf(docs) {
  const query = $("home-filter").value.trim().toLowerCase();
  if (!query || !docs.length) return docs;
  const found = new QuickMatch(docs.map(wordsOf)).matchesBestWithIdsAndMatchedWords(query, new QuickMatchConfig().withLimit(docs.length).withUnionFallback(false));
  return found.map(([id]) => docs[id]);
}
/** A chart's words for the grid's filter: its title, its layers' names, series and formulas. @param {Chart} doc */
const wordsOf = (doc) => [titleOf(doc), doc.folder ? folderPath(doc.folder) : "", ...doc.layers.flatMap((layer) => [labelOf(layer), layer.series ?? "", layer.formula ?? ""])].join(" ").toLowerCase().replace(/_/g, " ");
/** Your charts the page shows, in its order (its grid's, then its folders' shelves'), as last drawn; how many charts it shows (theirs, the previews', Recent's). */
let shownYours = /** @type {Chart[]} */ ([]), yoursFound = 0;
/** Recent's presets its page shows, as last drawn, each its preset (-1: the presets not read yet): their tiles theirs, drawn as theirs are (see drawYours). @type {Map<Chart, number>} */
let presetTiles = new Map();
const shownCharts = () => shownYours.filter((doc) => library.charts.includes(doc));
/** The filter's index of the charts (their words, their folders' names), kept till a chart or a folder changes. */
let filterIndex = { key: "", charts: /** @type {Chart[]} */ ([]), index: /** @type {any} */ (null) };
/**
 * The charts the filter finds, as ⌘K finds series (quickmatch: prefixes, joined words, typos), those matching the
 * most of its words, best first; without a filter, all of them in their order.
 */
function filtered() {
  const query = $("home-filter").value.trim().toLowerCase();
  if (!query) return library.charts;
  const key = JSON.stringify([library.charts.map((doc) => [doc.id, doc.updated, doc.folder ?? ""]), library.folders.map((folder) => [folder.id, folder.name, folder.parent ?? ""])]);
  if (filterIndex.key !== key) filterIndex = { key, charts: library.charts.slice(), index: new QuickMatch(library.charts.map(wordsOf)) };
  const found = filterIndex.index.matchesBestWithIdsAndMatchedWords(query, new QuickMatchConfig().withLimit(filterIndex.charts.length).withUnionFallback(false));
  return found.map((/** @type {[number]} */ [id]) => filterIndex.charts[id]);
}
/** Each tile as last drawn, to redraw only those that changed; your part of the page as last laid out. @type {Map<string, string>} */
const drawnTiles = new Map();
let drawnLayout = "";
/** The page's head: its name (a folder of yours, its menu beside it), the way up (each level a step to), its search, what's there to do. */
function renderHomeHead() {
  const yours = "folder" in homeAt, folder = "folder" in homeAt ? folderById(homeAt.folder) : undefined, name = homeName(), top = "top" in homeAt;
  // (Home's shown by its question, what it's for, see .home-question.)
  $("home-title").textContent = name;
  // (Home: its question, its search and its doors, in the middle of the page, nothing else.)
  homeScroll().classList.toggle("at-home", top);
  $("home-tools").hidden = top;
  $("home-start").hidden = !top;
  // (Undo and Redo where every page has them, at its head's end; Home's, in its top corner, beside the theme.)
  const undo = /** @type {HTMLElement} */ (document.querySelector(".home-undo"));
  if (top && undo.parentElement !== $("home-top")) $("home-top").prepend(undo);
  else if (!top && undo.parentElement !== $("home-title").parentElement) /** @type {HTMLElement} */ ($("home-title").parentElement).append(undo);
  // (Home's own, around its page (its horizon, its corners): going and coming with it, as it fades (see goTo); back
  // as its copy's still going, that copy gone first.)
  for (const own of [$("home-horizon"), $("home-help"), $("home-top")]) {
    if (own.hidden === !top) continue;
    if (!top && started) fadeOut(own, [{ opacity: 1 }, { opacity: 0 }], MOTION.small, true, MOTION.ease);
    else cutFade(own);
    own.hidden = !top;
    if (top && started && !reduced()) own.animate({ opacity: [0, 1] }, { duration: MOTION.small, easing: MOTION.ease });
  }
  const more = $("home-title-more");
  // (A folder's menu on its page; your Library's, its group's, on its own: as their ⋯ in the sidebar.)
  more.hidden = !folder && !yours;
  delete more.dataset.folderMenu, delete more.dataset.groupMenu;
  if (folder) Object.assign(more.dataset, { folderMenu: folder.id }), more.setAttribute("aria-label", `More for ${folder.name}`);
  else if (yours) Object.assign(more.dataset, { groupMenu: "library" }), more.setAttribute("aria-label", "More for your Library");
  const ups = top ? [] : [{ name: "Home", at: /** @type {Place} */ ({ top: true }) }, ...stepsTo(homeAt).slice(0, -1)];
  const where = $("gallery-path");
  where.hidden = top;
  where.innerHTML = top ? "" : `<button type="button" class="cap small" data-place="${escapeHtml(JSON.stringify(/** @type {{ name: string, at: Place }} */ (ups[ups.length - 1]).at))}" aria-label="Up a level" data-tip="Up a level">${ICONS.back}</button>${crumbsHtml(ups)}`;
  const field = /** @type {HTMLInputElement} */ ($("home-filter"));
  field.placeholder = `Search in ${name}`;
  field.setAttribute("aria-label", field.placeholder);
  // (Where your folders are: their page, a folder's; the way to have them all elsewhere, on their page.)
  $("home-new-folder").hidden = !yours;
  $("home-foot").hidden = !yours || Boolean(folder) || !anythingKept();
  renderSideNav();
}
/** The page's name, where it is: a folder's, the Library's, the previews', Recent's, a section's, the presets'; at the top, Home. */
const homeName = () => ("folder" in homeAt ? folderById(homeAt.folder)?.name ?? "Library" : "previews" in homeAt ? "Previews" : "recent" in homeAt ? "Recent" : "path" in homeAt ? placeNames(homeAt.path).at(-1) ?? "Presets" : "Home");
/** @typedef {{ name: string, at: Place }} Step */
/** A path's steps as crumbs (see .crumbs): each a button to its place, "›" between. @param {Step[]} steps */
function crumbsHtml(steps) {
  return steps.map((step) => `<button type="button" dir="auto" data-place="${escapeHtml(JSON.stringify(step.at))}"><span>${escapeHtml(step.name)}</span></button>`).join('<span class="sep" aria-hidden="true">›</span>');
}
/**
 * The way to a place, the place last, from its group (none for the top): a section's, the Presets and the sections
 * it's in; a folder's, the Library and the folders it's in (one gone, the Library's).
 * @param {Place} place @returns {Step[]}
 */
function stepsTo(place) {
  const group = (/** @type {Group} */ name) => ({ name: GROUP_OF[name][0], at: GROUP_OF[name][1] });
  if ("previews" in place) return [group("previews")];
  if ("recent" in place) return [group("recent")];
  if ("path" in place) return [group("presets"), ...placeNames(place.path).map((name, k) => ({ name, at: { path: place.path.slice(0, k + 1) } }))];
  if (!("folder" in place)) return [];
  const folder = folderById(place.folder);
  return [group("library"), ...(folder ? [...foldersAbove(folder), folder] : []).map((step) => ({ name: step.name, at: { folder: step.id } }))];
}
/** A preset chart's sections, by its own key (read as stored: anything but a list of names, none). @param {Chart} doc */
function presetPathOf(doc) {
  try {
    const path = doc.preset === undefined ? null : JSON.parse(doc.preset);
    return Array.isArray(path) && path.length && path.every((name) => typeof name === "string") ? /** @type {string[]} */ (path).slice(0, -1) : null;
  } catch {
    return null;
  }
}
/** Where a chart is (the open one's, else): a preset's section, your folder (or the Library's top), the previews; a new chart, not yet anywhere, nowhere. @returns {Place | null} */
function chartPlace(doc = chart) {
  const i = presetIndex(doc);
  // (A preset's, before the presets are read: its own key says where it is (its sections, then its name).)
  return i >= 0 ? { path: /** @type {Preset[]} */ (presets)[i].p.slice(0, -1) }
    : !presets && presetPathOf(doc) ? { path: /** @type {string[]} */ (presetPathOf(doc)) }
    : isPreview(doc) && !drafts.has(doc) ? { previews: true }
    : library.charts.includes(doc) ? { folder: doc.folder ?? "" }
    : null;
}
/** The step last drawn before the open chart's name (drawn again only changed: it keeps its focus). */
let chartPathDrawn = "";
/** The chart it was drawn for (itself, whatever its id: a preview saved again takes a new one). @type {Chart | null} */
let chartPathOf = null;
/** Where the open chart is, before its name (see .chart-path): the section, folder or group it's in, its menu's anchor (see chartPathRows). */
function renderChartPath() {
  const place = chartPlace(), steps = place ? stepsTo(place) : [], parent = steps.at(-1), path = $("chart-path");
  const html = parent ? `<button type="button" id="chart-place" dir="auto" aria-haspopup="menu" aria-label="In ${escapeHtml(steps.map((step) => step.name).join(", "))}" data-tip="In ${escapeHtml(steps.map((step) => step.name).join(" › "))}"><span>${escapeHtml(parent.name)}</span></button><span class="sep" aria-hidden="true">›</span>` : "";
  // (The same chart somewhere else (a preview saved, and back): its path as wide as it's been at least, its name kept
  // where it is; another chart's, as wide as its own.)
  if (chart !== chartPathOf) path.style.minWidth = "";
  else if (html !== chartPathDrawn && chartPathDrawn) path.style.minWidth = `${path.offsetWidth}px`;
  chartPathOf = chart;
  path.hidden = !html;
  if (html !== chartPathDrawn) path.innerHTML = chartPathDrawn = html;
}
/**
 * The open chart's place as a menu: the way there, each a step to its page; then, apart, what's in it as the sidebar
 * has it, a section or folder a step to its page, a chart opened, the open one ticked.
 * @returns {Action[]}
 */
/** A place's mark, as its row's: a folder's, a preset section's (`open`: one the open chart's in); else a folder's. */
function placeMark(/** @type {any} */ at, open = false) {
  const folder = at && "folder" in at && at.folder ? folderById(at.folder) : undefined;
  return folder ? folderMark(folder) : at && "path" in at ? sectionMark(presetNodeAt(at.path), open) : open ? ICONS.folderOpen : ICONS.folder;
}
function chartPathRows() {
  const place = chartPlace();
  if (!place) return [];
  const steps = stepsTo(place), at = /** @type {Step} */ (steps.at(-1)).name;
  // (Every row in the ticked one's column: the open chart's tick, the others' room for it.)
  const page = (/** @type {Step} */ step, open = false) => ({ icon: placeMark(step.at, open), label: step.name, current: false, run: () => goTo(step.at) });
  // (Each chart by its mini chart, as the sidebar has it.)
  const doc = (/** @type {Chart} */ item) => ({ group: at, icon: chartMark(item), label: titleOf(item), current: item === chart, run: () => go(chartById(item.id)) });
  /** @type {Action[]} */
  const inside = "path" in place
    ? (presetNodeAt(place.path)?.items ?? []).map((item) => (typeof item === "number"
      ? { group: at, icon: presetMark(item), label: ((edited) => (edited ? titleOf(edited) : /** @type {Preset[]} */ (presets)[item].t))(keptPreset(item)), current: item === presetIndex(chart), run: () => pickPreset(item) }
      : { group: at, ...page({ name: item.name, at: { path: item.path } }) }))
    : "folder" in place ? childrenOf(place.folder || undefined).map((item) => ("layers" in item ? doc(item) : { group: at, ...page({ name: item.name, at: { folder: item.id } }) }))
    : listedPreviews().map(doc);
  // (Its presets' marks not drawn yet, drawn now, as the sidebar's rows in sight are.)
  if ("path" in place) for (const item of presetNodeAt(place.path)?.items ?? []) if (typeof item === "number" && !presetMarks.has(presetMarkKey(item))) presetMarksAsked.add(item);
  drawPresetMarks();
  return [...steps.map((step) => page(step, true)), ...inside];
}
/** The folders a folder's in, the outermost first. @param {Folder} folder */
function foldersAbove(folder) {
  const above = [];
  for (let at = folderById(folder.parent), steps = 0; at && steps <= library.folders.length; at = folderById(at.parent), steps++) above.unshift(at);
  return above;
}
/**
 * A part of the page's side of your charts: a shelf's `place`, where its name leads; its `folder`, what's dropped on
 * it goes into (yours, "": the Library's top; none: nothing, the previews'); a note `flat`, its html as it is (a
 * search's none, under its heading).
 * @typedef {{ kind: "grid", docs: Chart[], previews?: boolean } | { kind: "shelf", name: string, place: Place, folder?: string, docs: Chart[], count: number } | { kind: "note", html: string, card?: boolean, flat?: boolean }} YoursPart
 */
// Your charts' and the previews' tiles, each drawn (if it's out of date) once it's in view, one chart at a time while
// the page shows, as the presets' are (see galleryTilesSeen); let go of if it's left before its turn.
const yourQueue = /** @type {Set<string>} */ (new Set()), yoursSeen = /** @type {Set<Element>} */ (new Set());
/** The tiles in view, waited for again (a frame written anew, a fetch that failed). */
function queueYours() {
  yourQueue.clear();
  for (const frame of yoursSeen) if (frame.isConnected) yourQueue.add(/** @type {string} */ (/** @type {HTMLElement} */ (frame).dataset.plot));
  void drawYours();
}
/** Home's horizon as last drawn: its points, its last, the theme (drawn again only changed); its history's last ask failed (asked again, see updateStatus); one under way. */
let horizonDrawn = "", horizonFailed = false, horizonAsking = false;
/**
 * Home's horizon: the price's whole history, week by week, under the page, faint, on a log scale as the chart has
 * prices; drawn as a line is (its color, its width, its area's fade), coming in from the left; each halving a rule
 * under it; its last point the live price (said over it, in the live line). Read once, its end moved as the price
 * does.
 */
function renderHorizon() {
  const box = $("home-horizon");
  if (box.hidden) return;
  // (One ask at a time: the price moving meanwhile waits for it.)
  if (horizonAsking) return;
  horizonAsking = true;
  void load("price_close", "week1", horizonFailed).finally(() => (horizonAsking = false)).then(({ data, times }) => {
    horizonFailed = false;
    if (box.hidden) return;
    const points = data.flatMap((value, i) => (value > 0 ? [[times[i], Math.log10(value)]] : []));
    if (livePrice > 0) points.push([Date.now() / 1000, Math.log10(livePrice)]);
    const key = `${points.length}:${livePrice}:${light}`;
    if (points.length < 2 || key === horizonDrawn) return;
    horizonDrawn = key;
    const W = 1000, H = 300, from = points[0][0], to = points[points.length - 1][0], values = points.map(([, value]) => value), lo = Math.min(...values), hi = Math.max(...values);
    const x = (/** @type {number} */ time) => ((time - from) / (to - from)) * W, y = (/** @type {number} */ value) => H * 0.12 + (1 - (value - lo) / (hi - lo)) * H * 0.88;
    const line = points.map(([time, value]) => `${x(time).toFixed(1)},${y(value).toFixed(1)}`), color = ink("orange");
    // (Each halving a rule from the price that week down to the floor.)
    const halvings = minedHalvings().filter((time) => time > from && time < to).map((time) => ((at) => `<line x1="${x(time).toFixed(1)}" x2="${x(time).toFixed(1)}" y1="${y(at).toFixed(1)}" y2="${H}" class="halving"/>`)((points.find(([when]) => when >= time) ?? points[points.length - 1])[1])).join("");
    box.innerHTML = `<svg viewBox="0 0 ${W} ${H}" preserveAspectRatio="none"><defs>${fadeDef("horizon-fill", color, ".12")}<linearGradient id="horizon-in" x1="0" x2="1"><stop offset="0" stop-color="#fff" stop-opacity="0"/><stop offset=".35" stop-color="#fff" stop-opacity=".55"/><stop offset="1" stop-color="#fff"/></linearGradient><mask id="horizon-mask"><rect width="${W}" height="${H}" fill="url(#horizon-in)"/></mask></defs><g mask="url(#horizon-mask)">${halvings}${areaUnder("horizon-fill", line, W, H)}${lineOver(line, color, 0.6)}</g></svg>`;
  }).catch(() => (horizonFailed = true));
}
/** Home's doors as last drawn (drawn again only changed: focus and hover stay). */
let doorsDrawn = "";
/** A group's mark, before its name (on the rail, its head): the previews', what's looked at; your Library's, what's kept; the presets'. */
const GROUP_MARK = /** @type {Record<Group, string>} */ ({ recent: "recent", previews: "eye", library: "keep", presets: "presets" });
/**
 * Home's doors, under its search: a door each to the groups, as the sidebar has them (its mark, its name, how many,
 * its way): Recent and the previews while there are some, your Library, the presets; nothing in them opened,
 * whatever was come for; New chart at the other end.
 */
function renderDoors() {
  const box = $("home-doors");
  box.hidden = !home || !("top" in homeAt);
  if (box.hidden) return;
  const counts = /** @type {Record<Group, number | null>} */ ({ recent: recentCharts().length, previews: listedPreviews().length, library: library.charts.length, presets: presets?.length ?? null });
  // (How many after its name, as a shelf's head says it; none for an empty Library, nor before the presets are in.)
  const door = (/** @type {Group} */ group) => {
    const [name, place] = GROUP_OF[group], count = counts[group];
    return `<button type="button" class="btn ghost" data-place="${escapeHtml(JSON.stringify(place))}" aria-label="${count === null ? name : `${name}, ${plural(count, "chart")}`}">${ICONS[GROUP_MARK[group]]}${name}${count ? `<span class="count">${grouped.format(count)}</span>` : ""}</button>`;
  };
  // (Recent and the previews only while they hold some, as in the sidebar.)
  const html = `<nav class="doors" aria-label="Go to">${GROUPS.filter((group) => counts[group] !== 0 || group === "library" || group === "presets").map(door).join("")}</nav><button type="button" class="btn ghost home-new" data-action="new-chart" data-tip="New chart" data-kbd="N">${ICONS.plus}New chart</button>`;
  if (html === doorsDrawn) return;
  const focus = focusKey(box);
  box.innerHTML = doorsDrawn = html;
  restoreFocus(box, focus);
}
/**
 * Your charts' side of the page, where it is (none in a section of the presets, at Home): the previews' page and
 * Recent's, a grid (Recent's presets as their own tiles); your Library's or a folder's, its charts as a grid, then
 * each folder in it as a shelf. Its tiles drawn again only where they changed.
 */
function renderYours() {
  const grid = $("home-grid"), folderId = "folder" in homeAt ? homeAt.folder || undefined : undefined, onPreviews = "previews" in homeAt;
  const query = $("home-filter").value.trim();
  grid.hidden = !yoursShown();
  /** @type {YoursPart[]} */
  const parts = [];
  presetTiles = new Map();
  if (!grid.hidden && "recent" in homeAt) {
    const items = recentCharts(), found = query ? foundOf(items.map((item) => item.doc)) : null, shown = found ? items.filter((item) => found.includes(item.doc)) : items;
    // (A preset's tile is the presets': till they're read, one waits in its place (-1); not read, said so, with a way
    // to try again, as the presets' page says it.)
    for (const item of shown) if (item.preset >= 0 || item.doc.preset !== undefined) presetTiles.set(item.doc, item.preset);
    const failed = presetsFailed && shown.some((item) => presetTiles.get(item.doc) === -1), listed = failed ? shown.filter((item) => presetTiles.get(item.doc) !== -1) : shown;
    if (listed.length) parts.push({ kind: "grid", docs: listed.map((item) => item.doc) });
    if (failed) parts.push({ kind: "note", html: `<p class="gallery-note">The presets couldn't be read. <button type="button" class="btn ghost small" data-gallery-retry>Try again</button></p>`, flat: true });
    else if (!listed.length) parts.push({ kind: "note", html: query ? `No recent charts match “${escapeHtml(query)}”` : "<p>Nothing opened yet. The charts you open show here, the latest first.</p>" });
  } else if (!grid.hidden && onPreviews) {
    const previewed = query ? previewsShown() : listedPreviews();
    parts.push(previewed.length ? { kind: "grid", docs: previewed, previews: true } : { kind: "note", html: query ? `No previews match “${escapeHtml(query)}”` : "<p>No previews. Charts opened from a link appear here until you save or close them.</p>" });
  } else if (!grid.hidden && query) {
    const found = filtered().filter((doc) => !folderId || isIn(doc.folder, folderId));
    parts.push(found.length ? { kind: "grid", docs: found } : { kind: "note", html: `<p class="gallery-note">No charts match “${escapeHtml(query)}”</p>`, flat: true });
  } else if (!grid.hidden) {
    const items = childrenOf(folderId), counts = chartCounts();
    const own = items.flatMap((item) => ("layers" in item ? [item] : []));
    // (Its own charts first, under its name, as a section of the presets has its own: theirs; then each folder in it,
    // named over its own row. After its folders, they read as the last one's.)
    if (own.length) parts.push({ kind: "grid", docs: own });
    for (const folder of items.flatMap((item) => ("layers" in item ? [] : [item]))) parts.push({ kind: "shelf", name: folder.name, place: { folder: folder.id }, folder: folder.id, docs: treeOf(folder.id).flatMap(({ item }) => ("layers" in item ? [item] : [])).slice(0, SHELF), count: counts.get(folder.id) ?? 0 });
    if (!items.length) parts.push({ kind: "note", html: folderId ? `<p>${TOUCH ? "Nothing in it yet. Start a chart in it, or hold a chart, then tap Move." : "Nothing in it yet. Start a chart in it, or drag charts onto it in the sidebar."}</p><div class="home-none-actions"><button class="btn small" type="button" data-action="new-chart">${ICONS.plus}New chart</button><button class="btn ghost small" type="button" data-action="import" data-into="${folderId}">${ICONS.upload}Import charts</button></div>` : `<p>No charts yet. Start a new chart, or open a preset and save a copy.</p><div class="home-none-actions"><button class="btn small" type="button" data-action="new-chart">${ICONS.plus}New chart</button><button class="btn ghost small" type="button" data-action="import">${ICONS.upload}Import charts</button></div>`, card: true });
  }
  const visible = parts.flatMap((part) => (part.kind === "grid" || part.kind === "shelf" ? part.docs : []));
  yoursFound = visible.length;
  shownYours = visible.filter((doc) => !isPreview(doc) && !presetTiles.has(doc));
  // Only what's shown stays picked: the bar acts on what can be seen (of your charts: no preview's picked).
  const ids = new Set(shownYours.map((doc) => doc.id));
  for (const id of [...picked]) if (!ids.has(id)) picked.delete(id);
  if (!picked.has(pickedLast)) pickedLast = [...picked].at(-1) ?? "";
  const focused = /** @type {HTMLElement | null} */ (document.activeElement?.closest(".tile-open")?.closest(".tile") ?? null)?.dataset.chart;
  // The same charts in the same order (and parts): only the tiles that changed are drawn again (a click on another
  // one, under way, isn't lost; focus and hover stay). Otherwise, all of them.
  // (A tile by the chart it opens: a preset's, the preset's chart's, whether its copy is kept here or not.)
  const html = visible.map((doc) => ((i) => (i === undefined ? tileHtml(doc) : i < 0 ? WAITING_TILE : presetTileHtml(i, doc)))(presetTiles.get(doc))), tiles = all$(".tile", grid);
  const idOf = (/** @type {Chart} */ doc) => ((i) => (i === undefined || i < 0 ? doc.id : presetIdOf(presetKey(i))))(presetTiles.get(doc));
  const layout = JSON.stringify(parts.map((part) => (part.kind === "shelf" ? [part.kind, JSON.stringify(part.place), part.name, part.count, part.docs.length, ((folder) => (folder ? folderMark(folder) : ""))(folderById(part.folder ?? ""))] : part.kind === "grid" ? [part.kind, Boolean(part.previews), part.docs.length] : [part.kind, part.html])));
  if (layout === drawnLayout && tiles.length === visible.length && tiles.every((tile, i) => tile.dataset.chart === idOf(visible[i]))) {
    tiles.forEach((tile, i) => drawnTiles.get(idOf(visible[i])) !== html[i] && swapTile(tile, html[i]));
  } else {
    // (The focus comes back to its like. A tile drawn as it was is moved, not written again: a search typed, its
    // charts' mini charts aren't made anew.)
    const focus = focusKey(grid), old = new Map(tiles.map((tile) => [/** @type {string} */ (tile.dataset.chart), tile]));
    const htmlOf = new Map(visible.map((doc, i) => ((id) => [id, old.has(id) && drawnTiles.get(id) === html[i] ? `<i data-slot="${id}"></i>` : html[i]])(idOf(doc))));
    const tilesOf = (/** @type {Chart[]} */ docs) => docs.map((doc) => htmlOf.get(idOf(doc))).join("");
    grid.innerHTML = parts.map((part) => (part.kind === "grid" ? `<div class="grid-tiles${part.previews ? " previews" : ""}">${tilesOf(part.docs)}</div>`
      : part.kind === "shelf" ? folderShelfHtml(part, tilesOf(part.docs))
      // (A search finding none says so under its heading, as the presets' does; a page with nothing, in its middle.)
      : part.flat ? part.html : `<div class="${part.card ? "shelf-empty library-empty" : "home-none"}">${part.html}</div>`)).join("");
    for (const slot of all$("[data-slot]", grid)) slot.replaceWith(/** @type {HTMLElement} */ (old.get(/** @type {string} */ (slot.dataset.slot))));
    restoreFocus(grid, focus);
    shelvesSoon();
  }
  drawnLayout = layout;
  drawnTiles.clear();
  visible.forEach((doc, i) => drawnTiles.set(idOf(doc), html[i]));
  if (focused) tileOpen(focused)?.focus({ preventScroll: true });
  renderPicked();
  for (const frame of all$("[data-plot]", grid)) yourTilesSeen.observe(frame);
  keepNaming();
}
/** A shelf's steps, a page of tiles back and on (see stepShelf). */
const SHELF_STEPS = `<button type="button" class="cap small shelf-step" data-shelf-step="-1" aria-label="Scroll back" data-tip="Scroll back">${ICONS.back}</button><button type="button" class="cap small shelf-step" data-shelf-step="1" aria-label="Scroll on" data-tip="Scroll on">${ICONS.next}</button>`;
/**
 * Your charts as a shelf: a folder's (its name, how many it holds, leading into it; its menu; its charts, the first
 * of them at any depth), a row; its name leads to its page. None in it, it says so.
 * @param {{ name: string, place: Place, folder?: string, count: number }} shelf @param {string} tiles
 */
const folderShelfHtml = ({ name, place, folder, count }, tiles) => `<section class="shelf"><div class="shelf-head"${folder === undefined ? "" : ` data-grid-folder="${folder}"`}><button type="button" class="shelf-title" data-place="${escapeHtml(JSON.stringify(place))}" aria-label="${escapeHtml(name)}, ${grouped.format(count)}">${((own) => (own ? folderMark(own) : ""))(folder ? folderById(folder) : undefined)}${escapeHtml(name)}<span class="count">${grouped.format(count)}</span>${ICONS.next}</button>${folder ? moreCap("shelf-more", "data-folder-menu", folder, name) : ""}<span class="spacer"></span>${SHELF_STEPS}</div><div class="shelf-row">${tiles ? `${tiles}${seeAll(place, count)}` : `<p class="shelf-empty">No charts in it yet</p>`}</div></section>`;
/** A chart's tile: what the chart grows out of and shrinks into. */
// (A preset's written as a slot yet: written now, for what wants it (the way back from it shrinks into it).)
const cardOf = (id) => /** @type {HTMLElement | null} */ (pressedTile?.isConnected && pressedTile.dataset.chart === id ? pressedTile : $("home").querySelector(`.tile[data-chart="${CSS.escape(id)}"]`) ?? ((slot) => slot && fillSlot(slot))(/** @type {HTMLElement | null} */ ($("home").querySelector(`.tile-slot[data-slot="${presetById(id)}"]`))));
const tileOpen = (id) => /** @type {HTMLElement | null} */ (cardOf(id)?.querySelector(".tile-open") ?? null);

/**
 * Where the page is: at the top, Charts (`top`); the presets (`path` [], their page) or a section of them (`path`,
 * as their tree names it); your Library (`folder` "") or one of its folders (`folder`); the previews (`previews`); the
 * charts last opened (`recent`). Each a step of the history (Back goes back up).
 * @typedef {{ top: true } | { path: string[] } | { folder: string } | { previews: true } | { recent: true }} Place
 */
/** A place as kept in a step of the history, as it can be (none: the top). @returns {Place} */
const asPlace = (/** @type {any} */ at) => (typeof at?.folder === "string" ? { folder: at.folder } : Array.isArray(at?.path) ? { path: at.path.filter((/** @type {any} */ name) => typeof name === "string") } : at?.previews === true ? { previews: true } : at?.recent === true ? { recent: true } : { top: true });
/**
 * Each page's place as it was left, by its step's key: Back and Forward come back there. A step left by them can't
 * be written any more (its place, in it, only when left otherwise): kept here, it's where it was last.
 */
const leftPlaces = /** @type {Map<string, { top: number, rows: Record<string, number>, shown: number }>} */ (new Map());
/** Whether the page shows charts of yours (or previews) as tiles: your Library, a folder of it, the previews. */
const yoursShown = () => home && ("folder" in homeAt || "previews" in homeAt || "recent" in homeAt);
/**
 * The page at a place (a shelf's name, a step up, the sidebar's, ⌘K's), nothing searched, at its top; where it is
 * already, back at its top. A step of the history, where it was kept in the one it leaves (Back comes back there).
 */
/**
 * The page left fading out over the one coming in (see pageIn), as it was, scrolled where it was: never a frame of
 * neither. (Left again before its own way in began, held: whole as it's left, its copy no blank.)
 */
function pageOut() {
  if (reduced()) return;
  for (const animation of /** @type {HTMLElement} */ ($("home").querySelector(".home-inner")).getAnimations()) animation.finish();
  fadeOut(homeScroll(), [{ opacity: 1 }, { opacity: 0 }], MOTION.small, true, MOTION.ease, true);
}
/**
 * Another page of Charts come to (a folder, a section, back): it comes in, up a little as it shows, not swapped at
 * once. Its way in and the page left's way out (see goTo) begun once frames come on time: on a phone the new page's
 * first frames (laid out, its tiles painted) took a tenth of a second or more, the fade all but over when it showed.
 */
function pageIn() {
  if (reduced()) return;
  const into = /** @type {HTMLElement} */ ($("home").querySelector(".home-inner")).animate({ opacity: [0, 1], translate: ["0 6px", "0 0"] }, { duration: MOTION.small, easing: MOTION.ease });
  beginOnTime(into, ...(fading.get(homeScroll())?.getAnimations() ?? []));
}
/**
 * Where the page is, lit in the sidebar: Charts at its top, the heads' pages, a folder of yours, a section of the
 * presets; closed over it (a head over a page of its own, a folder over one in it, a section over one under it),
 * barred, as one closed over the open chart is.
 */
function renderSideNav() {
  const lit = (/** @type {Element} */ element, /** @type {boolean} */ on) => on !== element.hasAttribute("aria-current") && (on ? element.setAttribute("aria-current", "page") : element.removeAttribute("aria-current"));
  const folder = home && "folder" in homeAt ? homeAt.folder : null, path = home && "path" in homeAt ? JSON.stringify(homeAt.path) : null, at = home ? JSON.stringify(homeAt) : "";
  // (Charts' row in the list: none where the list is owed, a phone's, see renderCharts.)
  const nav = document.getElementById("charts-nav");
  if (nav) lit(nav, home && "top" in homeAt);
  for (const head of all$("[data-at]", $("chart-list"))) lit(head, head.dataset.at === at);
  for (const button of all$("[data-folder-item]", $("chart-list"))) lit(button, button.dataset.folderItem === folder);
  for (const button of all$("[data-section-item]", $("chart-list"))) lit(button, button.dataset.sectionItem === path);
  const over = (/** @type {Element | null} */ row, /** @type {boolean} */ on) => row?.classList.toggle("holds-page", on && row.classList.contains("closed"));
  for (const head of all$(".list-head", $("chart-list"))) over(head, home && at !== /** @type {HTMLElement} */ (head.querySelector("[data-at]"))?.dataset.at && (head.getAttribute("data-group") === "library" ? folder !== null : head.getAttribute("data-group") === "presets" ? path !== null : false));
  for (const row of all$("[data-folder-row]", $("chart-list"))) over(row, Boolean(folder) && folder !== row.dataset.folderRow && isIn(folder ?? undefined, /** @type {string} */ (row.dataset.folderRow)));
  for (const row of all$("[data-section-row]", $("chart-list"))) over(row, path !== null && path !== row.dataset.sectionRow && path.startsWith(/** @type {string} */ (row.dataset.sectionRow).slice(0, -1) + ","));
  showAddress();
}
/** The most a shelf shows (its section's first), and a grid at once (more, a button away). */
const SHELF = 12, GALLERY_PAGE = 60;
/** A shelf's last tile, holding more than it shows: to its page, as its name is. @param {Place} place @param {number} count */
const seeAll = (place, count) => (count > SHELF ? `<button type="button" class="shelf-all" data-see-all="" data-place="${escapeHtml(JSON.stringify(place))}">See all ${grouped.format(count)}${ICONS.next}</button>` : "");
/** The recommended presets, by their place in the tree. */
/** The recommended places' presets as last found, and the list they were found in. */
let recommendedFound = /** @type {{ for: Preset[] | null, at: number[] }} */ ({ for: null, at: [] });
/** Each recommended place's preset (-1: none there), found in one walk of the list, once a list. @returns {number[]} */
const recommendedAt = () => {
  if (recommendedFound.for !== presets) {
    const wanted = new Map(RECOMMENDED.map((place, k) => [place, k])), at = RECOMMENDED.map(() => -1);
    (presets ?? []).forEach((preset, i) => ((k) => k !== undefined && at[k] < 0 && (at[k] = i))(wanted.get(preset.p.join(" › "))));
    recommendedFound = { for: presets, at };
  }
  return recommendedFound.at;
};
const recommendedPresets = () => recommendedAt().filter((i) => i >= 0);
/**
 * A preset's tile, as a chart's in the grid: its title, its layers' legend, its mini chart (drawn once in view). New
 * chart's (`card`, its place among the recommended): the same, its drawing framed apart (its own ids).
 */
function presetTileHtml(/** @type {number} */ i, doc = presetDoc(i), card = -1) {
  const title = escapeHtml(titleOf(doc));
  const legend = cardLegend(doc);
  return `<div class="tile preset-tile ${doc.preset !== undefined && presetEdited(doc) ? "edited" : ""}" data-preset="${i}"${card < 0 ? ` data-chart="${presetIdOf(presetKey(i))}"` : ""}><button type="button" class="tile-open" ${card < 0 ? `data-open-preset="${i}"` : `data-template="${card}"`} aria-label="${title}" data-tip="${title}"><span class="tile-head"><span class="tile-name">${title}</span>${tileIndexHtml(doc)}</span><span class="tile-legend">${legend}</span>${plotHtml(doc, true, card < 0 ? "tile-plot" : "template-plot tile-plot")}</button>${moreCap("tile-more", "data-preset-menu", i, titleOf(doc))}</div>`;
}
/** The gallery's lists as drawn, each by its place there (its grid's Show more finds its own by it). */
let listsDrawn = 0;
const listAt = () => listsDrawn++;
/**
 * A grid of presets, as many as shown, more a button away: its first tile written (every row as tall as it), the
 * others a slot each, written once near sight (see nearTiles): a search typed writes what shows, not sixty.
 * `total`: how many there are in all (`list` may be only the first of them, as many as show and one).
 * @param {number[]} list @param {number} [total]
 */
const presetGridHtml = (list, total = list.length) => `<div class="grid-tiles" data-list="${listAt()}">${list.slice(0, galleryShown).map((i, k) => (k ? slotHtml(i) : presetTileHtml(i))).join("")}</div>${total > galleryShown ? `<button type="button" class="btn ghost small gallery-more" data-gallery-more>Show ${grouped.format(Math.min(GALLERY_PAGE, total - galleryShown))} more</button>` : ""}`;
/** A tile while the presets are read: its place held, a light sweeping across it. */
const WAITING_TILE = '<div class="tile waiting" aria-hidden="true"><span class="tile-open"><span class="tile-head"><span class="tile-name">&nbsp;</span></span><span class="tile-legend"></span><span class="tile-plot pending" data-seen></span></span></div>';
/**
 * The tiles a shelf writes as it's drawn (as many as a screen's widest row shows); the rest a slot each, as wide, its
 * tile written as it comes near (see tilesNear), or a keyboard's focus comes into its row: a page of shelves (the
 * presets' fourteen) writes what can show, not every shelf's twelve.
 */
const SHELF_WRITTEN = 3;
const shelfTilesHtml = (/** @type {number[]} */ list) => list.slice(0, SHELF).map((i, k) => (k < SHELF_WRITTEN ? presetTileHtml(i) : slotHtml(i))).join("");
/** A preset's tile's place, held till it's written (see fillSlot). */
const slotHtml = (/** @type {number} */ i) => `<div class="tile-slot" data-slot="${i}"></div>`;
/** A shelf's slot, its tile written in its place, its mini chart drawn once in view. @param {HTMLElement} slot */
function fillSlot(/** @type {HTMLElement} */ slot) {
  return writeTile(slot, Number(slot.dataset.slot));
}
/**
 * Preset `i`'s tile written in `place`'s (a slot's, a tile's of it as it was: the focus in it kept on its like), its
 * mini chart drawn once in view. @param {HTMLElement} place
 */
function writeTile(place, /** @type {number} */ i) {
  const made = document.createElement("template"), focus = focusKey(place);
  made.innerHTML = presetTileHtml(i);
  const tile = /** @type {HTMLElement} */ (made.content.firstElementChild);
  place.replaceWith(tile);
  restoreFocus(tile, focus);
  for (const frame of all$(".pending[data-plot]", tile)) watchTile(frame);
  return tile;
}
/** Every slot of a shelf's row filled (a keyboard's focus come into it: Tab reaches each tile). @param {HTMLElement} row */
const fillRow = (row) => all$(":scope > .tile-slot", row).forEach(fillSlot);
/**
 * A shelf: a section's first charts in a row, its name leading to it (the recommended's, nowhere).
 * @param {string} name @param {number[]} list @param {string[] | null} to
 */
const shelfHtml = (name, list, to) => `<section class="shelf" data-list="${listAt()}"><div class="shelf-head">${to ? `<button type="button" class="shelf-title" data-place="${escapeHtml(JSON.stringify({ path: to }))}" aria-label="${escapeHtml(name)}, ${grouped.format(list.length)}">${escapeHtml(name)}<span class="count">${grouped.format(list.length)}</span>${ICONS.next}</button>` : `<h2 class="shelf-title">${escapeHtml(name)}</h2>`}<span class="spacer"></span>${SHELF_STEPS}</div><div class="shelf-row">${shelfTilesHtml(list)}${to ? seeAll({ path: to }, list.length) : ""}</div></section>`;
/**
 * The presets' page, where it is: theirs, the recommended, then each section as a shelf; a section's, its own as a
 * grid, then each section in it. Elsewhere, hidden as drawn (its plain page: a search's is found again), to come back
 * to as it is, not written again.
 */
function renderGallery() {
  const field = /** @type {HTMLInputElement} */ ($("home-filter")), words = typedWords(field.value);
  const shown = "path" in homeAt, path = "path" in homeAt ? homeAt.path : [], back = shown && $("gallery").hidden;
  $("gallery").hidden = !shown;
  if (!shown) {
    // (The focus in it let go, as a page gone lets it go (see goTo).)
    if ($("gallery").contains(document.activeElement)) /** @type {HTMLElement} */ (document.activeElement).blur();
    if (!galleryPlain) galleryDrawn = gallerySpot = "";
    galleryFound = 0;
    return;
  }
  if (!presets) void loadPresets().catch(() => {});
  // (Drawn as it is already, it stays: what's scrolled on it, focused, drawn, as it was.)
  // (A preset edited here shows as it is: drawn anew once one is; one only opened, kept as it was, shows as the preset
  // does, nothing to draw again for.)
  const spot = JSON.stringify([path, words, galleryShown]), drawn = JSON.stringify([spot, presets?.length ?? 0, presetsFailed, light, library.presets.filter((doc) => !pristine(doc)).map((doc) => `${doc.preset}@${doc.updated}`)]);
  if (drawn === galleryDrawn && !back) return;
  // (Come back to as it was drawn (hidden meanwhile, see above), nothing it shows changed since.)
  const again = back && drawn === galleryDrawn, lists = listsDrawn;
  // (The same place drawn anew (the theme switched): its scroll and its shelves' kept. Come back to, a page come to.)
  const kept = spot === gallerySpot && !back ? galleryPlace() : null;
  // (A search over the page as it was: that page kept aside, as drawn, to come back once the search is cleared.)
  if (words.length && galleryPlain && presets) galleryAside = { drawn: galleryDrawn, lists: listsDrawn, nodes: [...$("gallery").childNodes] };
  // (Any grid but a search's next: the loop's wait on a tile of the one it replaces ended, see galleryWait. It goes on
  // once this render is written, never on what it replaced.)
  if (galleryPlain || !words.length) endGalleryWait();
  galleryPlain = !words.length;
  galleryDrawn = drawn;
  gallerySpot = spot;
  listsDrawn = 0;
  // (A grid drawn anew is a move, as a scroll is: what a key typed shows may be gone at the next, see stillness.)
  galleryMoves++;
  galleryQueue.clear();
  gallerySeen.clear();
  galleryTilesSeen.disconnect();
  galleryAhead.clear();
  galleryTilesAhead.disconnect();
  galleryFound = 0;
  // (The focus in it, back on its like once drawn. A shelf's arrows are alike: one focused is let go, not put on
  // another shelf's.)
  const focus = document.activeElement?.closest?.("[data-shelf-step]") ? null : focusKey($("gallery"));
  if (!presets) {
    // (Read, the top's shelf as it will be, its tiles waiting: what comes takes their place, nothing under it moves.)
    $("gallery").innerHTML = presetsFailed ? `<p class="gallery-note">The presets couldn't be read. <button type="button" class="btn ghost small" data-gallery-retry>Try again</button></p>`
      : `<p class="gallery-note">Loading the presets…</p>`;
    return restoreFocus($("gallery"), focus);
  }
  if (words.length) {
    // (The best of them as many as show, ranked; how many, all of them.)
    const { found, count, partial } = presetsMatching(words, galleryShown + 1, path.length ? (i) => path.every((name, depth) => /** @type {Preset[]} */ (presets)[i].p[depth] === name) : null);
    galleryFound = count;
    // (None in a section: said where, and the presets as a whole offered, where they hold some: no dead end.)
    const elsewhere = count || !path.length ? 0 : presetsMatching(words, 0).count;
    $("gallery").innerHTML = (count ? `${partial ? `<p class="gallery-note">No preset holds every word; these hold the most.</p>` : ""}${presetGridHtml(found, count)}` : `<p class="gallery-note">No presets ${path.length ? `in ${escapeHtml(/** @type {string} */ (placeNames(path).at(-1)))} ` : ""}match “${escapeHtml(field.value.trim())}”${elsewhere ? ` <button type="button" class="btn small" data-search-all>Search all presets<span class="count">${grouped.format(elsewhere)}</span></button>` : ""}</p>`);
  } else if (again) {
    // (Kept, not written again: at its shelves' starts, as a page come to is (Back puts them where they were, see
    // popstate).)
    listsDrawn = lists;
    for (const row of all$(".shelf-row", $("gallery"))) if (shelfScrolls.get(row)) (row.scrollLeft = 0), shelfScrolls.set(row, 0);
    freshenTiles();
  } else if (galleryAside?.drawn === drawn) {
    // (The page as it was before the search, nothing it shows changed since: put back, not written again.)
    $("gallery").replaceChildren(...galleryAside.nodes);
    listsDrawn = galleryAside.lists;
    galleryAside = null;
    freshenTiles();
  } else {
    galleryAside = null;
    // (Under a section, its charts: those right under it, then each section under it with all of its own.)
    const items = presetNodeAt(path)?.items ?? [], own = /** @type {number[]} */ (items.filter((item) => typeof item === "number"));
    $("gallery").innerHTML = [
      ...(path.length ? [] : [shelfHtml("Recommended", recommendedPresets(), null)]),
      ...(own.length ? [presetGridHtml(own)] : []),
      ...items.flatMap((item) => (typeof item === "number" ? [] : [shelfHtml(item.name, item.under, item.path)])),
    ].join("");
  }
  if (kept) placeGallery(kept);
  restoreFocus($("gallery"), focus);
  for (const frame of all$(".pending[data-plot]", $("gallery"))) watchTile(frame);
  shelvesSoon();
}
/**
 * A page of the presets kept as drawn, put back (see renderGallery): each tile's plot as a written tile's would be
 * (see plotHtml): its preset's drawing as it is now, shown if it's another (a chart drawn anew as it's left,
 * `data-drawn`); none, or out of date (a day on), waiting in sight to be drawn; a failed one waits in sight too, for
 * a retry.
 */
function freshenTiles() {
  for (const frame of all$(".tile[data-preset] > .tile-open [data-plot]", $("gallery"))) {
    // (A tile of its preset as it was (opened since: its copy's id now): written again, as a page written anew has it.)
    const i = presetOf(frame), doc = presetDoc(i), now = previews.get(doc.id);
    if (frame.dataset.plot !== doc.id) writeTile(/** @type {HTMLElement} */ (frame.closest(".tile")), i);
    else if (!now || now.key !== previewKey(doc)) frame.classList.contains("failed") || frame.classList.add("pending");
    else if (frame.dataset.drawn !== String(drawingId(now))) paint(doc.id, now, true);
  }
  for (const frame of all$(".failed[data-plot]", $("gallery"))) galleryTilesSeen.observe(frame);
}
/** The gallery as last drawn (where, the search, how much of it, the presets as read, the theme), its place, and how many it found. */
let galleryDrawn = "", gallerySpot = "", galleryFound = 0;
/** Whether the gallery shows no search; the page it showed before a search, kept aside as drawn (see renderGallery). */
let galleryPlain = false, galleryAside = /** @type {{ drawn: string, lists: number, nodes: Node[] } | null} */ (null);
/** What ends the drawing loop's wait on a tile early (see galleryWait). @type {(value?: unknown) => void} */
let endGalleryWait = () => {};
/**
 * The drawing loop's wait on a tile's drawing, ended early by any grid drawn but a search's next (see renderGallery):
 * the grid shown then is asked for as any is, the drawing going on. Between a search's grids it isn't, so a key typed
 * meanwhile asks for no tile of the grids it passes. Only drawGallery waits, one tile at a time (it's serial): each
 * wait takes the one slot.
 * @param {Promise<unknown>} drawn
 */
const galleryWait = (drawn) => Promise.race([drawn, new Promise((done) => (endGalleryWait = done))]);
/** A shelf's steps: there only while its row scrolls, each off at its end. @param {HTMLElement} row */
function shelfSteps(row) {
  shelfScrolls.set(row, row.scrollLeft);
  fitShelf(row, row.scrollLeft, row.clientWidth, row.scrollWidth, cutsOf(row));
}
/** A shelf a page on (`dir` 1) or back (-1), by whole tiles: the first one there whole at its start. */
function stepShelf(/** @type {HTMLElement} */ row, /** @type {number} */ dir) {
  const tile = /** @type {HTMLElement | null} */ (row.firstElementChild);
  if (!tile) return;
  // (Its tiles' width as laid out, a fraction of a pixel too: whole tiles counted, three where three show.)
  const gap = parseFloat(getComputedStyle(row).columnGap) || 0, unit = tile.getBoundingClientRect().width + gap, page = Math.max(1, Math.floor((row.clientWidth - 8 + gap) / unit + 0.01));
  row.scrollTo({ left: (Math.round(row.scrollLeft / unit) + dir * page) * unit, behavior: reduced() ? "auto" : "smooth" });
}
/** Every shelf's steps and edges, once the page's writes this frame are in (one layout, not one a write), before it's shown. */
let shelvesFrame = 0;
const shelvesSoon = () => (shelvesFrame ||= requestAnimationFrame(() => ((shelvesFrame = 0), allShelfSteps())));
/** Every shelf's steps and edges, its slots watched, a grid's too (all read first, then written: one layout, not one a shelf). */
function allShelfSteps() {
  const rows = shelfRows(), sizes = rows.map((row) => ({ at: row.scrollLeft, size: row.clientWidth, full: row.scrollWidth, cut: cutsOf(row) }));
  nearTiles(rows);
  rows.forEach((row, k) => fitShelf(row, sizes[k].at, sizes[k].size, sizes[k].full, sizes[k].cut));
}
/** Each shelf whose slots are watched, and what watches them. @type {WeakMap<HTMLElement, IntersectionObserver>} */
const rowsNear = new WeakMap();
/**
 * Shelves two screens' height from sight in the page's own scroller (the window's would see them only once in it,
 * clipped by it), made again as that height changes (a margin's percentage is of its width): their slots watched
 * as they scroll (see nearTiles).
 */
let shelvesNear = /** @type {IntersectionObserver | null} */ (null), nearReach = 0;
/** A grid's slots as near (see nearTiles). @type {IntersectionObserver | null} */
let slotsNear = null;
/**
 * The shelves' slots, their tiles written once they're near sight: the shelf two screens' height from it, each slot
 * two rows' width from it as the row scrolls; a grid's, each two screens' height from it, those in sight at once (an
 * observer's first word comes a frame on: blank till then). In time for a step or a fling, never one shown blank.
 * The shelves and slots on the page now watched, not those it had (written anew, gone).
 * @param {HTMLElement[]} rows
 */
function nearTiles(rows) {
  // (Out of sight (another page's), none: watched once it's back, see showHome.)
  const scroller = homeScroll(), reach = scroller.clientHeight;
  if (!reach) return;
  if (!shelvesNear || reach !== nearReach) {
    shelvesNear?.disconnect();
    slotsNear?.disconnect();
    nearReach = reach;
    slotsNear = new IntersectionObserver((entries, own) => {
      for (const entry of entries) if (entry.isIntersecting) (own.unobserve(entry.target), fillSlot(/** @type {HTMLElement} */ (entry.target)));
    }, { root: scroller, rootMargin: `${2 * reach}px 0px` });
    shelvesNear = new IntersectionObserver((entries, own) => {
      // (Every shelf come near read first, then each written: one layout, not one a shelf.)
      const rows = entries.filter((entry) => entry.isIntersecting && !rowsNear.has(/** @type {HTMLElement} */ (entry.target))).map((entry) => /** @type {HTMLElement} */ (entry.target));
      const sizes = rows.map((row) => ({ at: row.scrollLeft, room: row.clientWidth, full: row.scrollWidth, unit: row.children.length > 1 ? row.children[1].getBoundingClientRect().left - row.children[0].getBoundingClientRect().left : 0, cut: cutsOf(row) }));
      rows.forEach((row, k) => (own.unobserve(row), fitShelf(row, sizes[k].at, sizes[k].room, sizes[k].full, sizes[k].cut), tilesNear(row, sizes[k])));
    }, { root: scroller, rootMargin: `${2 * reach}px 0px` });
  }
  const shelves = /** @type {IntersectionObserver} */ (shelvesNear), slots = /** @type {IntersectionObserver} */ (slotsNear);
  shelves.disconnect();
  for (const row of rows) if (!rowsNear.has(row)) shelves.observe(row);
  // (All read first, then written: one layout, not one a slot.)
  const view = scroller.getBoundingClientRect(), held = all$(".grid-tiles > .tile-slot", $("home")), boxes = held.map((slot) => slot.getBoundingClientRect());
  slots.disconnect();
  held.forEach((slot, k) => (boxes[k].bottom > view.top && boxes[k].top < view.bottom ? fillSlot(slot) : slots.observe(slot)));
}
/** A shelf's tile written anew where it was (a chart's changed). @param {HTMLElement} tile @param {string} html */
function swapTile(tile, html) {
  const made = document.createElement("template");
  made.innerHTML = html;
  tile.replaceWith(/** @type {HTMLElement} */ (made.content.firstElementChild));
}
/**
 * A shelf's slots near sight, their tiles written: those two rows' widths from sight now, at once (an observer's
 * first word comes a frame on, and a page scrolled fast is past a screen's height by then); the others as the row
 * scrolls, as far ahead (the browser scrolls on its own, a slow frame or two of the page's meanwhile). By the row
 * as it was read (`at` its scroll, `room` its width, `unit` a tile's step), before anything was written.
 * @param {HTMLElement} row @param {{ at: number, room: number, unit: number }} size
 */
function tilesNear(row, { at, room, unit }) {
  const near = new IntersectionObserver((entries) => {
    for (const entry of entries) if (entry.isIntersecting) (near.unobserve(entry.target), fillSlot(/** @type {HTMLElement} */ (entry.target)));
  }, { root: row, rootMargin: "0px 200%" });
  rowsNear.set(row, near);
  [...row.children].forEach((tile, i) => tile.matches(".tile-slot") && (i * unit < at + 3 * room && (i + 1) * unit > at - 2 * room ? fillSlot(/** @type {HTMLElement} */ (tile)) : near.observe(tile)));
}
/**
 * A shelf as it's scrolled: its steps there while it scrolls, each off at its end; its edges faded where there's
 * more and a tile's cut, as a strip's (see cutsOf).
 * @param {HTMLElement} row @param {number} at @param {number} size @param {number} full @param {{ start: boolean, end: boolean }} cut
 */
function fitShelf(row, at, size, full, cut) {
  const over = full - size;
  for (const step of all$("[data-shelf-step]", /** @type {HTMLElement} */ (row.closest(".shelf")))) {
    step.hidden = over <= 1;
    step.setAttribute("aria-disabled", String(Number(step.dataset.shelfStep) < 0 ? at <= 1 : at >= over - 1));
  }
  edges(row, at, size, full, cut);
}
// Each tile drawn once it's in view (on a shelf, scrolled to), one at a time; let go of if it's left before its turn
// (a preset on two shelves, the recommended's and its section's: while either's in view).
const galleryQueue = /** @type {Set<number>} */ (new Set()), gallerySeen = /** @type {Set<Element>} */ (new Set());
const presetOf = (/** @type {Element} */ frame) => Number(/** @type {HTMLElement | null} */ (frame.closest("[data-preset]"))?.dataset.preset);
// (Its root the grid's own scroller: from the window's, what's past the scroller's edge is cut off, margin or not. Its
// margin a screen, the scroller's height in pixels (a percentage there is of its width), made again as it's resized.)
const aheadOf = (/** @type {number} */ height) => new IntersectionObserver((entries) => {
  for (const entry of entries) entry.isIntersecting ? galleryAhead.add(entry.target) : galleryAhead.delete(entry.target);
  drawGallery();
}, { root: homeScroller, rootMargin: `0px 0px ${Math.round(height)}px 0px` });
/** A tile waiting to be drawn, watched in view and a screen ahead. @param {Element} frame */
const watchTile = (frame) => (galleryTilesSeen.observe(frame), galleryTilesAhead.observe(frame));
// (At rest: no scroll moving in Home (its own, a shelf's: caught on the way down, as scrollend doesn't bubble) and no
// finger down on it; a pan's pointer is cancelled, not let go: its scroll's end says it's over.)
let galleryResting = true;
let galleryScrolling = false;
/** Scrolls in Home so far, and its grid's drawings anew (see renderGallery). */
let galleryMoves = 0;
/**
 * A drawing loop's test of whether Home is still: at rest, and not scrolled since the loop last asked (a drawing trip
 * ago, at least). A wheel's ticks each end a scroll, so between them it's at rest; a trip always sees one of them. Its
 * first ask, at rest is enough (a landing's scroll to a section isn't a wheel's).
 */
function stillness() {
  let seen = -1;
  return () => {
    const still = galleryResting && (seen < 0 || galleryMoves === seen);
    seen = galleryMoves;
    return still;
  };
}
const galleryRests = () => {
  galleryResting = true;
  drawGallery();
};
/** Home's listeners that tell whether the grid's at rest, added where app.js's start comes to them. */
function startGalleryRest() {
  $("home").addEventListener("scroll", () => (galleryResting = false, galleryScrolling = true, galleryMoves++), { capture: true, passive: true });
  $("home").addEventListener("scrollend", () => ((galleryScrolling = false), galleryRests()), { capture: true });
  // (A press keeps the grid from rest till its release; a press cancelled or lost, till the next frame unless a
  // scroll's begun by then, whose end then says.)
  $("home").addEventListener("pointerdown", (event) => {
    galleryResting = false;
    press(event, {
      end: (release) => {
        if (release) galleryRests();
        else requestAnimationFrame(() => galleryScrolling || galleryRests());
      },
    });
  });
}
/** The next tile to draw: one in view, else, at rest, one ahead not tried yet in this pass (-1: none). @param {Set<number>} tried */
const nextGallery = (tried) => {
  for (const i of galleryQueue) return galleryQueue.delete(i), i;
  if (galleryResting) for (const frame of galleryAhead) if (frame.isConnected && frame.classList.contains("pending") && !tried.has(presetOf(frame))) return presetOf(frame);
  return -1;
};

/** The charts picked, and the last one picked (where a ⇧-click's run starts). */
const picked = new Set();
let pickedLast = "";
/** A pick's checkbox (a card's, a layer row's) as it is: ticked or not, its tip what a click does. @param {HTMLInputElement} check @param {boolean} on */
const showPick = (check, on) => ((check.checked = on), (check.dataset.tip = on ? "Deselect" : "Select"));
/** The picks shown: ticks, the bar and its count. While some are picked, a click picks rather than opens. */
function renderPicked() {
  const n = picked.size;
  $("home").classList.toggle("picking", n > 0);
  for (const tile of all$(".tile[data-home-card]:not(.preview)", $("home-grid"))) {
    const on = picked.has(/** @type {string} */ (tile.dataset.homeCard));
    tile.classList.toggle("picked", on);
    showPick(/** @type {HTMLInputElement} */ (tile.querySelector(".tile-check")), on);
  }
  $("home-bar").hidden = n === 0;
  document.body.toggleAttribute("data-charts-picked", n > 0);
  $("home-picked").textContent = plural(n, "chart");
  // (Rename, one at a time: dimmed with several, in its place, nothing in the bar moving as picks come and go.)
  const rename = /** @type {HTMLButtonElement} */ ($("home-bar").querySelector('[data-batch="rename"]'));
  rename.setAttribute("aria-disabled", String(n !== 1));
  rename.dataset.tip = n === 1 ? "Rename" : "Rename: one chart at a time";
}
/** Picks a chart: added or taken off; with `range`, every chart from the last one picked to it. */
function pickChart(id, range = false) {
  hideMenu();
  // (Your charts the page shows alone are what the bar acts on: not a preview, nor a preset's tile.)
  const ids = shownCharts().map((doc) => doc.id), from = ids.indexOf(pickedLast), to = ids.indexOf(id);
  if (to < 0) return;
  if (range && picked.size && from >= 0 && to >= 0) for (const other of ids.slice(Math.min(from, to), Math.max(from, to) + 1)) picked.add(other);
  else if (picked.has(id)) {
    // (One taken off isn't where a run starts: the last one picked that's still there is.)
    picked.delete(id);
    if (!picked.has(pickedLast)) pickedLast = [...picked].at(-1) ?? "";
    return renderPicked();
  } else picked.add(id);
  pickedLast = id;
  renderPicked();
}
/** No more picks; `refocus`: the focus goes back to the tile last picked (from the bar's ×, it'd be lost). */
function clearPicks(refocus = false) {
  picked.clear();
  renderPicked();
  if (refocus) tileOpen(pickedLast)?.focus();
}
/**
 * What the bar (or a tile's right-click) does to the charts picked: rename (one, in its tile), duplicate (each
 * under its original, the copies then picked), export (one file), a folder (its menu), delete (at once, one Undo for
 * all). @param {string} action @param {HTMLElement | null} [button] the bar's button, a folder's menu beside it
 */
function batch(action, button = null, /** @type {string[] | null} */ ids = null) {
  // (`ids`: a chart's own menu, on one not picked: the picks stay as they are.)
  const docs = library.charts.filter((doc) => (ids ? ids.includes(doc.id) : picked.has(doc.id)));
  if (action === "clear" || !docs.length) return clearPicks(action === "clear");
  if (action === "rename") return void (docs.length === 1 && renameTile(docs[0]));
  if (action === "folder") return folderMenu(/** @type {HTMLElement} */ (button ?? tileOpen(docs[0].id)), docs.map(sideKey));
  if (action === "export") {
    // One, its chart file (as from its menu); several, one file of them (imported, they come back to the grid, picked).
    if (docs.length === 1) return downloadChart(docs[0]);
    download(`${docs.length}-charts.json`, chartsFile(docs));
    return toast(`Downloaded ${docs.length} charts, in one file`);
  }
  if (action === "duplicate") {
    // (The picks' copies picked; one not picked, its copy alone, the picks as they are.)
    // (A ⇧-click's run from the copy of the one it ran from, as it is before the copies redraw the page: the
    // original isn't picked.)
    const from = docs.findIndex((doc) => doc.id === pickedLast), copies = duplicateCharts(docs).map((doc) => doc.id);
    if (!ids) {
      picked.clear();
      // (Those shown: one past a shelf's end isn't.)
      for (const id of copies) if (shownYours.some((doc) => doc.id === id)) picked.add(id);
      const anchor = copies[from];
      pickedLast = anchor && picked.has(anchor) ? anchor : [...picked].at(-1) ?? "";
      renderPicked();
    }
    tileOpen(copies[0])?.focus();
    return;
  }
  if (action === "delete") {
    // The focus then goes to the tile that takes the first one's place; none left, to the toast's Undo (never a
    // field: there ⌘Z would undo its typing, not this).
    const at = shownCharts().findIndex((doc) => docs.includes(doc));
    if (!ids) clearPicks();
    deleteCharts(docs);
    /** @type {HTMLElement | undefined} */ (all$(".tile:not(.preview) .tile-open", $("home-grid"))[at] ?? all$(".tile:not(.preview) .tile-open", $("home-grid")).at(-1) ?? $("toast").querySelector("button") ?? undefined)?.focus();
  }
}
/** Deletes charts, with one Undo that brings them all back where they were. @param {Chart[]} docs */
function deleteCharts(docs) {
  if (docs.length === 1) return deleteChart(docs[0]);
  const places = docs.map((doc) => /** @type {[number, Chart]} */ ([library.charts.indexOf(doc), doc])).filter(([at]) => at >= 0).sort((a, b) => a[0] - b[0]);
  for (const [, doc] of places) discard(doc);
  // All of them gone (the open one with them), a preview there is stands in, else a blank one.
  const open = chart, wasOpen = places.some(([, doc]) => doc === open), standIn = wasOpen && !library.charts.length ? previewCharts[0] ?? newDraft() : null;
  if (wasOpen) openChart(standIn ?? library.charts[Math.min(places[0][0], library.charts.length - 1)]);
  save();
  renderCharts();
  toast(`Deleted ${places.length} charts`, {
    label: "Undo",
    run: () => {
      if (places.every(([, doc]) => inLibrary(doc))) return;
      for (const [at, doc] of places) {
        if (!inLibrary(doc)) relist(doc, at);
      }
      if (wasOpen) openChart(chartById(open.id) ?? open);
      save();
      renderCharts();
      return () => ((left) => left.length && deleteCharts(left))(places.flatMap(([, doc]) => library.charts.find((other) => other.id === doc.id) ?? []));
    },
    undone: `${places.length} charts are back`,
  });
}
/** A chart's title, typed in its tile: ↵ or leaving keeps it, esc doesn't. @param {Chart} doc */
function renameTile(doc) {
  const tile = cardOf(doc.id);
  if (!tile) return;
  const name = /** @type {HTMLElement} */ (tile.querySelector(".tile-name"));
  nameInPlace(tile, { className: "tile-rename", value: doc.title, placeholder: derivedTitle(doc), max: 120, label: "Chart title", top: name.getBoundingClientRect().top - tile.getBoundingClientRect().top - 4, apply: (title) => renameById(doc.id, title), back: () => tileOpen(doc.id)?.focus(), find: () => cardOf(doc.id) });
}
/** A chart's title, typed in its sidebar row: ↵ or leaving keeps it, esc doesn't. @param {Chart} doc @param {HTMLElement} row */
function renameRow(doc, row) {
  nameInPlace(row, { className: "row-rename", value: doc.title, placeholder: derivedTitle(doc), max: 120, label: "Chart title", apply: (title) => renameById(doc.id, title), back: () => /** @type {HTMLElement | null} */ ($("chart-list").querySelector(`[data-open-chart="${doc.id}"]`))?.focus(), find: () => $("chart-list").querySelector(`[data-chart-row="${doc.id}"], [data-preview-row="${doc.id}"]`) });
}
/** A name being typed in place, and where it goes again when what it's in is drawn anew. */
let naming = /** @type {{ input: HTMLInputElement, find: () => Element | null, finish: () => void } | null} */ (null);
/** The name being typed, back in its place drawn anew (its text, its selection, the focus); its place gone, it ends. */
function keepNaming() {
  if (!naming || (naming.input.isConnected && document.activeElement === naming.input)) return;
  const { input } = naming, { selectionStart, selectionEnd } = input;
  if (!input.isConnected) {
    const host = naming.find();
    if (!host) return void (naming = null);
    host.append(input);
  }
  input.focus({ preventScroll: true });
  input.setSelectionRange(selectionStart, selectionEnd);
}
/**
 * A name typed where it shows (a tile's, a sidebar row's, a folder's head): ↵ or leaving it keeps it, esc doesn't;
 * ended by a key, `back` puts the focus back on what it names. `apply` takes the name; `find`, where it shows when
 * that's drawn again (another tab's change, a mark redrawn): the name being typed goes on there; `close`, once it's
 * ended, what it's in let go of; `cancel`, esc's (a folder just made, unmade).
 * @param {HTMLElement} host @param {{ className: string, value: string, placeholder: string, max: number, label: string, top?: number, apply: (name: string) => void, back: () => void, find: () => Element | null, close?: () => void, cancel?: () => void }} options
 */
function nameInPlace(host, { className, value, placeholder, max, label, top, apply, back, find, close = () => {}, cancel = () => {} }) {
  const input = document.createElement("input");
  input.enterKeyHint = "done";
  Object.assign(input, { className, value, placeholder, maxLength: max, spellcheck: false, autocomplete: "off" });
  input.setAttribute("aria-label", label);
  if (top !== undefined) input.style.top = `${top}px`;
  host.append(input);
  input.focus();
  input.select();
  let done = false;
  const finish = (/** @type {boolean} */ keep, /** @type {boolean} */ refocus) => {
    if (done) return;
    done = true;
    if (naming?.input === input) naming = null;
    const name = input.value.trim().slice(0, max);
    input.remove();
    close();
    if (keep) apply(name);
    else cancel();
    if (refocus) back();
  };
  // (Kept at once where the page leaves it: a save, another chart opened.)
  naming = { input, find, finish: () => finish(true, false) };
  input.addEventListener("keydown", (event) => {
    if (composing(event) || (event.key !== "Enter" && event.key !== "Escape")) return;
    event.preventDefault();
    event.stopPropagation();
    finish(event.key === "Enter", true);
  });
  // (Left, it's kept; taken out with what it's in, drawn anew, it's put back there: see keepNaming. The window left (a
  // phone's app switched), it stays as it is till the window's back: see leftOpen.)
  const left = () => queueMicrotask(() => input.isConnected && document.activeElement !== input && finish(true, false));
  input.addEventListener("blur", () => leftWith(input, left) || left());
}
/** A chart renamed by its id, as it is now (not as it was when the name began). */
function renameById(/** @type {string} */ id, /** @type {string} */ title) {
  const doc = chartById(id);
  if (doc && title !== doc.title) renameChart(doc, title);
}
/** What the picked charts do together (a picked card's ⋯ or right-click). @returns {Action[]} */
function gridActions() {
  const ids = [...picked], n = ids.length, leave = outOf(chartKeys(ids)), one = n === 1 ? chartById(ids[0]) : null;
  return [
    ...(one ? [{ key: "open", icon: ICONS.arrowRight, label: "Open", run: () => go(chartById(one.id)) }, { key: "rename", icon: ICONS.pencil, label: "Rename", run: () => batch("rename") }] : []),
    { key: "duplicate", icon: ICONS.duplicate, label: "Duplicate", run: () => batch("duplicate") },
    { key: "folder", icon: ICONS.folder, label: "Move to folder", page: () => choicePage("Move to folder", () => folderChoices(chartKeys(ids), false)) },
    ...(leave ? [{ key: "out", icon: ICONS.folderOut, label: leave, run: () => moveToFolder(chartKeys(ids), null) }] : []),
    { key: "download", icon: ICONS.file, label: "Download chart file", aside: "JSON", run: () => batch("export") },
    { key: "delete", icon: ICONS.trash, label: n > 1 ? `Delete ${n} charts` : "Delete", danger: true, run: () => batch("delete") },
  ];
}
/**
 * A change of place between the Charts page and a chart, both live (no pictures of them, every browser alike): the
 * chart's card grows out of a tile over the page (`opening`), or shrinks back into it, the page under it all along
 * (data-morph: both shown); with no tile, it fades in or out over the page. `update` changes the place and returns
 * the tile (none: null). The card keeps its shape: at the tile's end, the tile's width, its top in the tile,
 * clipped to the tile's height; shown over the tile only near its size (in over the first 8% as it grows: the ease
 * is quick at first, a third of the way; out over the last 30% as it shrinks, its last 1.4%), the live tile under
 * it meanwhile. Moved and clipped only (its compositor's work), its chart drawing as it goes. Resolves once over.
 * @param {boolean} opening @param {() => HTMLElement | null} update
 */
let morphing = /** @type {Animation[]} */ ([]);
/**
 * The library laid out at its element's own size. The library measures its element as it is drawn (its box scaled by
 * the card's motion), and lays out at every pane, price axis or share it changes: the first build of a page that
 * started on the grid (its data at hand, or arriving as the card grows) lies in that, a card's width or so, its
 * canvases at that size in the card's corner. The card's own size never changed (a motion moves it, nothing resizes
 * it), so nothing measures the chart again: once the motion's over, it's laid out against its box.
 */
function unscaled() {
  const { offsetWidth: width, offsetHeight: height } = engine.element;
  if (Math.abs(engine.width - width) >= 1 || Math.abs(engine.height - height) >= 1) engine.layout();
}
function morph(opening, update) {
  for (const animation of morphing) animation.cancel();
  // (Out of sight, no motion: a hidden page draws no frames, so it would never end, and a draw waits on it.)
  if (reduced() || document.hidden) return (update(), Promise.resolve());
  document.body.dataset.morph = "";
  const tile = update(), card = $("main"), at = card.getBoundingClientRect(), box = tile?.getBoundingClientRect();
  const ends = (/** @type {Keyframe[]} */ frames) => (opening ? frames : [...frames].reverse());
  const timing = { duration: box?.width ? MOTION.page : MOTION.panel, easing: MOTION.ease };
  const animations = [];
  if (box?.width) {
    const scale = box.width / at.width, radius = parseFloat(getComputedStyle(/** @type {HTMLElement} */ (tile)).borderTopLeftRadius) / scale;
    animations.push(card.animate(ends([
      { transform: `translate(${box.left - at.left}px, ${box.top - at.top}px) scale(${scale})`, clipPath: `inset(0 0 ${at.height - box.height / scale}px 0 round ${radius}px)` },
      { transform: "none", clipPath: "inset(-48px round 0px)" },
    ]), timing));
    animations.push(card.animate(opening ? [{ opacity: 0 }, { opacity: 1, offset: 0.08 }, { opacity: 1 }] : [{ opacity: 1 }, { opacity: 1, offset: 0.7 }, { opacity: 0 }], { duration: timing.duration }));
  } else animations.push(card.animate(ends([{ opacity: 0 }, { opacity: 1 }]), timing));
  morphing = animations;
  // (Over, the chart laid out as it shows, at its own size, and anything asked meanwhile (see relayout): measured
  // while it moved, it'd be measured scaled. The library's too: see unscaled.)
  return Promise.all(animations.map((animation) => animation.finished)).then(() => {}, () => {}).finally(() => morphing === animations && ((morphing = []), delete document.body.dataset.morph, unscaled(), (opening || layoutOwed) && relayout()));
}
/** Whether the focus is on the open chart (its page, not the grid over it), or in a menu (or its submenu) it opened. */
const onChart = () => {
  const active = document.activeElement, at = active?.closest("#menu, .submenu") ? ownerNow() ?? closedOwner : active;
  return Boolean(at?.closest("#main") && !at.closest("#home"));
};
/**
 * Opens a chart someone picked; from the grid, its card grows into it (`push`: a new step, for Back).
 * @param {Chart | undefined} doc
 */
let goes = 0, goingCard = /** @type {HTMLElement | null} */ (null);
function go(doc, push = true) {
  if (!doc) return;
  if (!home) return void (doc !== chart && openChart(doc));
  // (Its card pressed first, drawn before the chart's made, a while on a slow phone: the tap answered at once. Two
  // asked for meanwhile, the last one.)
  const mine = ++goes;
  goingCard?.classList.remove("opening");
  const card = (goingCard = cardOf(doc.id));
  card?.classList.add("opening");
  requestAnimationFrame(() => setTimeout(() => mine === goes && ((goingCard = null), leaveHome(() => doc !== chart && openChart(doc), card, push))));
}
/** A new chart, untouched, opened from the grid: leaving it goes back there. */
const backToHome = () => (isPreview(chart) || emptyYours(chart)) && history.state?.fromHome === true && history.state.chart === chart.id;
// Back and Forward, between the grid and the charts opened from it.
/**
 * Whether a dialog (⌘K, the guide, the shortcuts) is open. Each is a popover in the top layer, not a modal dialog
 * (Safari's AutoFill measures a modal's whole page as ⌘K's field takes the focus); what a modal does, done here: the
 * page behind held inert (holdPage), the keys kept in it, esc, a click beside it, Back on a phone.
 * @param {Element} element
 */
function isOpen(element) {
  return element.matches(":popover-open");
}
/** `then` once a dialog has closed (by then, maybe open again: it says so); `once`, the first time alone. @param {HTMLElement} element @param {() => void} then */
function onClosed(element, then, once = false) {
  const heard = (/** @type {Event} */ event) => {
    if (/** @type {ToggleEvent} */ (event).oldState !== "open") return;
    if (once) element.removeEventListener("toggle", heard);
    then();
  };
  element.addEventListener("toggle", heard);
}
/** What each dialog was opened from: the focus back there as it closes, as a modal's goes. */
const dialogOpeners = /** @type {WeakMap<HTMLElement, Element | null>} */ (new WeakMap());
/**
 * Each dialog's scrim, under it as it's open, taking a click beside it (a popover's backdrop takes none): its own,
 * as a modal's backdrop is, over whatever it's opened over (another dialog too).
 * @type {WeakMap<HTMLElement, HTMLElement>}
 */
const dialogScrims = new WeakMap();
/**
 * A dialog opened (see isOpen) on top, over its scrim, what had the focus kept to go back to.
 * @param {HTMLElement} dialog
 */
function showDialog(dialog) {
  if (isOpen(dialog)) return;
  dialogOpeners.set(dialog, document.activeElement);
  dialogsShown.push(dialog);
  holdPage(dialog, true);
  dialogScrims.get(dialog)?.showPopover();
  dialog.showPopover();
}
/** Back, Forward or a link pasted in: whatever dialog is open closes first. */
const closeDialogs = () => all$("dialog:popover-open").forEach((dialog) => dialog.hidePopover());
let leavings = 0;
/**
 * A dialog or a popover put away by the person (its ×, esc, a click beside it), as it leaves. Where the browser
 * keeps it drawn meanwhile, closed now, the style playing its way out; elsewhere (Safari: closed, gone at once), its
 * closed look played first (`data-leaving`, the style's own, as it'd go: a phone's sheet down, a dialog's settle
 * back out), out of reach, then closed. Opened again meanwhile (arrive), it stays. Closed by what it ran, it closes
 * at once, there (hidden): what follows is the page's (a dialog open, the keys stay in it), and
 * the result's what's looked at, as Spotlight goes.
 * @param {HTMLElement} element
 */
function leave(element) {
  if (!isOpen(element)) return;
  const shut = () => ((element.inert = false), delete element.dataset.leaving, element.hidePopover());
  if (keepsLeaving || reduced()) return void shut();
  if (element.dataset.leaving) return;
  const mine = String(++leavings);
  element.dataset.leaving = mine;
  element.inert = true;
  const motions = element.getAnimations({ subtree: true }).filter((motion) => motion instanceof CSSTransition && /** @type {KeyframeEffect} */ (motion.effect).target === element);
  Promise.allSettled(motions.map((motion) => motion.finished)).then(() => element.dataset.leaving === mine && shut());
}
/** A dialog or a popover opened again as it leaves (see leave): it stays. @param {HTMLElement} element */
const arrive = (element) => ((element.inert = false), delete element.dataset.leaving);
/**
 * A sheet dragged by its top, one pointer at a time: `grab`, a press there starts it (or not); `move`, how far it's
 * gone down (up, less than 0); `release`, let go there (its speed as it's let go, in px/ms); `back`, its look as it was, first thing
 * on a release and on its own when the system takes the pointer mid-way (a cancel isn't a release where it was).
 * `pulls`: whether its content pulls it too, as a phone's sheets: a finger on it going down past a tap's move (SLOP),
 * every list it's on at its top (or on what doesn't scroll), takes the sheet, not the list (`grab` then sees no
 * PointerEvent; the sheet follows from there); up, or a list scrolled, it scrolls as ever. The tap a pull ends on
 * isn't one.
 * @param {HTMLElement} element
 * @param {{ grab: (event: PointerEvent | null) => boolean, move: (by: number) => void, release: (by: number, speed: number) => void, back: () => void, pulls: () => boolean }} how
 */
function dragSheet(element, { grab, move, release, back, pulls }) {
  // (`pointer` -1: a finger's pull, by touch events.)
  let pull = /** @type {{ y: number, pointer: number, trail: { y: number, at: number }[] } | null} */ (null);
  // (Held, its handle lit, from the press to the release: what's held shows it.)
  const letGo = () => (element.classList.remove("pulled"), back());
  const follow = (/** @type {number} */ y) => {
    if (!pull) return;
    const now = performance.now();
    pull.trail = [...pull.trail.filter((point) => now - point.at <= 100), { y, at: now }];
    move(y - pull.y);
  };
  const end = (/** @type {number} */ y) => {
    if (!pull) return;
    // (Its speed as it's let go, the release its trail's last point.)
    const by = y - pull.y, speed = speedOf([...pull.trail, { y, at: performance.now() }], (point) => point.y);
    pull = null;
    letGo();
    release(by, speed);
  };
  element.addEventListener("pointerdown", (event) => {
    if (pull || !grab(event)) return;
    pull = { y: event.clientY, pointer: event.pointerId, trail: [{ y: event.clientY, at: performance.now() }] };
    element.classList.add("pulled");
    // (Where it shows from now, still or not: taken as it slides in, there, not at its stop till a move.)
    move(0);
    // (Held by what was pressed, the handle or the head: the click after a drag is its, not the sheet's own, which in
    // ⌘K is its backdrop's and would close it.)
    const held = /** @type {Element} */ (event.target), pointer = event.pointerId;
    held.setPointerCapture(pointer);
    const mine = () => pull?.pointer === pointer;
    press(event, { captor: held, move: (moved) => mine() && follow(moved.clientY), end: (release) => mine() && (release ? end(release.clientY) : ((pull = null), letGo())) });
  });
  // (A finger's, by touch events: one that turns out a pull stops the list's scroll, as a pointer's can't. Its events
  // go to what it touched, out of the sheet or not: a list drawn anew under it (a search's answer, a layer's data)
  // leaves it out, heard there then. A finger more, while one pulls, changes nothing.)
  let touch = /** @type {{ id: number, x: number, y: number, at: number, target: Element, taken: boolean } | null} */ (null);
  const own = (/** @type {TouchEvent} */ event) => (touch ? [...event.changedTouches].find((point) => point.identifier === touch?.id) : undefined);
  const moved = (/** @type {TouchEvent} */ event) => {
    // (While one pulls, no finger scrolls the sheet's lists.)
    if (touch?.taken && event.cancelable) event.preventDefault();
    const point = own(event);
    if (!touch || !point) return;
    if (!touch.taken) {
      const dx = point.clientX - touch.x, dy = point.clientY - touch.y;
      if (Math.hypot(dx, dy) <= SLOP) return;
      // (One finger on the page; down, mostly, every list it's on at its top (the sheet itself, a menu's); not a
      // press held first (a row's, carried or its actions); what it touched still on the sheet, where its lists
      // are read.)
      let top = element.contains(touch.target);
      for (let at = /** @type {Element | null} */ (touch.target); top && at; at = at === element ? null : at.parentElement) top = !scrollsY(at) || at.scrollTop <= 0;
      if (!(event.touches.length === 1 && dy > Math.abs(dx) && top && event.timeStamp - touch.at < 300 && !document.body.classList.contains("carrying") && event.cancelable && grab(null))) return forget();
      touch.taken = true;
      // (The gesture's now: a row's press waiting to be held goes.)
      stopCarrying();
      pull = { y: point.clientY, pointer: -1, trail: [{ y: point.clientY, at: performance.now() }] };
      element.classList.add("pulled");
    }
    event.preventDefault();
    follow(point.clientY);
  };
  const ended = (/** @type {TouchEvent} */ event) => {
    const point = own(event), taken = touch?.taken;
    if (!point) return;
    forget();
    if (taken && pull?.pointer === -1) (swallowClick(), end(point.clientY));
  };
  const cancelled = (/** @type {TouchEvent} */ event) => {
    const taken = touch?.taken;
    if (!own(event)) return;
    forget();
    if (taken && pull?.pointer === -1) (pull = null), letGo();
  };
  // (Each event once: on the sheet while what it touched is on it, else on what it touched.)
  const heard = (/** @type {(event: TouchEvent) => void} */ handle, /** @type {boolean} */ on) => (/** @type {Event} */ event) => (event.target instanceof Node && element.contains(event.target)) === on && handle(/** @type {TouchEvent} */ (event));
  const away = { move: heard(moved, false), end: heard(ended, false), cancel: heard(cancelled, false) };
  const forget = () => {
    if (touch && touch.target !== element) {
      touch.target.removeEventListener("touchmove", away.move);
      touch.target.removeEventListener("touchend", away.end);
      touch.target.removeEventListener("touchcancel", away.cancel);
    }
    touch = null;
  };
  element.addEventListener("touchstart", (event) => {
    if (touch?.taken) {
      const id = touch.id;
      if ([...event.touches].some((point) => point.identifier === id) && ![...event.changedTouches].some((point) => point.identifier === id)) return;
      // (Its finger's end never heard (none should go unsent): let go where it was.)
      forget();
      if (pull?.pointer === -1) (pull = null), letGo();
    }
    forget();
    const point = event.touches[0], target = event.target, box = element.getBoundingClientRect();
    // (On the sheet itself, not on what's around it: ⌘K's dim is its dialog's own.)
    if (pull || event.touches.length !== 1 || !(target instanceof Element) || point.clientY < box.top || point.clientX < box.left || point.clientX > box.right || !pulls()) return;
    touch = { id: point.identifier, x: point.clientX, y: point.clientY, at: event.timeStamp, target, taken: false };
    if (target === element) return;
    target.addEventListener("touchmove", away.move, { passive: false });
    target.addEventListener("touchend", away.end);
    target.addEventListener("touchcancel", away.cancel);
  }, { passive: true });
  element.addEventListener("touchmove", heard(moved, true), { passive: false });
  element.addEventListener("touchend", heard(ended, true));
  element.addEventListener("touchcancel", heard(cancelled, true));
}
/**
 * Animations begun together once frames come on time again (held at their start till then): begun in a frame the
 * page's work stalls (a layout under it changed with it), WebKit shows one already well on in the first frame it
 * draws (iOS 26.5: a sheet's first 80% at once, or all of it). 300 ms of frames on, they begin anyway (a tab out of
 * sight draws none: they wait for it). One taken over meanwhile (played, cancelled) is left as it is.
 * @param {Animation[]} animations
 */
function beginOnTime(...animations) {
  for (const animation of animations) animation.pause();
  let first = -1, last = -1;
  const wait = (/** @type {number} */ now) => {
    const held = animations.filter((animation) => animation.playState === "paused");
    if (!held.length) return;
    if (first < 0) first = now;
    else if (now - last < 40 || now - first > 300) return void held.forEach((animation) => animation.play());
    last = now;
    requestAnimationFrame(wait);
  };
  requestAnimationFrame(wait);
}
const slides = new WeakMap();
/** A sheet's transitions of its own transform (a drag let go, given back). @param {Element} sheet */
const ownSlides = (sheet) => sheet.getAnimations().filter((animation) => animation instanceof CSSTransition && animation.transitionProperty === "transform");
/**
 * A sheet's height changed by `change` (its foot the screen's), moving from where it showed (`from`, its top; where
 * it is, else) by its transform alone, which the screen moves on its own, the page busy or not (its height, laid out
 * each frame, stalled with it). Taller, it's so at once, slid up from there; shorter, as tall as it showed, slid down,
 * then so. Never as the screen's own height changes (a keyboard, a turn). It starts at `speed` (a finger's as it let
 * go, in px/ms, down above 0; still, none), so a drag goes on as it went: from still, gently, never the start's
 * frames skipped as a jump (iOS draws a motion's first frame some 40 ms into it: a curve fast at its start loses half
 * its way there).
 * @param {HTMLElement} sheet @param {() => void} change @param {number} [from] @param {number} [speed]
 */
function sizeSheet(sheet, change, from = sheet.getBoundingClientRect().top, speed = 0) {
  slides.get(sheet)?.cancel();
  sheet.style.height = sheet.style.maxHeight = "";
  change();
  // (Where its top comes to rest: its foot the screen's, whatever transform it has meanwhile; its height as laid out,
  // to the fraction.)
  const height = sheet.getBoundingClientRect().height, by = document.documentElement.clientHeight - height - from;
  // (A transition of its own transform, begun as a drag let go gave it back, gives way: run with the slide, iOS
  // showed neither, the sheet still where it was let go till both were over, then at its stop.)
  for (const animation of ownSlides(sheet)) animation.cancel();
  if (reduced() || Math.abs(by) < 1) return;
  // (The layout's curve, as a sheet opens and closes, its start's slope the finger's speed in shares of the way a
  // duration: none still, at most 2.5.)
  const timing = { duration: MOTION.layout, easing: `cubic-bezier(0.4, ${clamp((0.4 * speed * MOTION.layout) / by, 0, 1).toFixed(3)}, 0.2, 1)` };
  // (Its height as it showed, a stretched one's too: past its tallest.)
  if (by > 0) (sheet.style.height = `${height + by}px`), (sheet.style.maxHeight = "none");
  const slide = sheet.animate({ transform: by < 0 ? [`translateY(${-by}px)`, "none"] : ["none", `translateY(${by}px)`] }, timing);
  slides.set(sheet, slide);
  beginOnTime(slide);
  if (by > 0) slide.finished.then(() => (sheet.style.height = sheet.style.maxHeight = ""), () => {});
}
let keysWaiting = /** @type {(() => void) | null} */ (null), keysTimer = 0;
/**
 * `run` once the keyboard's up (see data-keys), or 400 ms on (none came: a keyboard of keys): a sheet grown as a field
 * in it is tapped. As the keyboard comes, iOS's screen stalls (150 ms on iOS 26.5), a motion's start lost to it.
 * @param {() => void} run
 */
function onceKeys(run) {
  clearTimeout(keysTimer);
  keysWaiting = run;
  keysTimer = setTimeout(keysCame, 400);
}
function keysCame() {
  const run = keysWaiting;
  keysWaiting = null;
  clearTimeout(keysTimer);
  run?.();
}
/**
 * Where something thrown, let go `at` (px, along one way) moving at `speed` (px/ms that way), would come to rest,
 * slowing as iOS's own lists do (0.998 of its speed kept each millisecond): how a native sheet or window reads a
 * flick, a short fast one as far as a long slow drag.
 */
const restOf = (/** @type {number} */ at, /** @type {number} */ speed) => at + (speed * 0.998) / (1 - 0.998);
/**
 * A pointer's speed as it's let go, along what `along` reads of each point (px/ms): over its last tenth of a second;
 * held still that long, none. @template P @param {(P & { at: number })[]} trail @param {(point: P) => number} along
 */
const speedOf = (trail, along) => {
  const now = performance.now(), last = trail.at(-1), past = trail.find((point) => now - point.at <= 100) ?? last;
  return last && past ? (along(last) - along(past)) / Math.max(1, now - past.at) : 0;
};
/** Whether a sheet dragged down goes: it'd come to rest past a third of its height (flicked up, it comes back). */
const sheetGoes = (/** @type {number} */ by, /** @type {number} */ speed, /** @type {number} */ height) => restOf(by, speed) > height / 3;
/** How far a sheet pulled `past` its tallest shows past it: less and less, as iOS's own edges give (55%, then less). */
const stretched = (/** @type {number} */ past) => (1 - 1 / ((past * 0.55) / innerHeight + 1)) * innerHeight;
/** How far a sheet's pulled for it to show `shown` past its tallest (stretched's own way back). */
const unstretched = (/** @type {number} */ shown) => ((1 / (1 - Math.min(shown, innerHeight - 1) / innerHeight) - 1) * innerHeight) / 0.55;
/**
 * A sheet let go to close: on down from where it showed (`from`, its top) past the screen's foot, at the finger's
 * speed (px/ms). Closed (`close`) at once, all that follows its closing now, its slide over its own way out (its dim's
 * its own); a dialog, though, once out of sight: closed in Safari, it's gone at once (no `overlay` there), so it's
 * slid first, its backdrop fading with it. Opened or closed meanwhile otherwise (esc, ×, Back, another menu), it's
 * that's: the slide stops, a dialog never closed later. A dialog pressed as it slides stays where it's caught, open
 * (see sheetTaken).
 * @param {HTMLElement} sheet @param {() => void} close @param {number} from @param {number} speed
 */
function slideAway(sheet, close, from, speed) {
  const later = sheet instanceof HTMLDialogElement;
  slides.get(sheet)?.cancel();
  // (Where it's laid out, its transform aside: the slide's from where it showed to its own height past its place,
  // past the foot it's on, however tall the screen gets meanwhile: a keyboard going down. As fast as the finger.)
  const away = () => {
    const top = sheet.getBoundingClientRect().top - new DOMMatrix(getComputedStyle(sheet).transform).m42, by = document.documentElement.clientHeight + 16 - from;
    return { frames: [{ transform: `translateY(${from - top}px)` }, { transform: "translateY(calc(100% + 16px))" }], duration: MOTION.layout, easing: `cubic-bezier(0.4, ${clamp((0.4 * Math.max(0, speed) * MOTION.layout) / by, 0, 1).toFixed(3)}, 0.2, 1)` };
  };
  // (A menu's sheet: closed at once, its copy sliding on (see fadeOut) from where it was let go, as fast.)
  if (sheet === $("menu") && !reduced()) return void hideMenu(away());
  if (!later) close();
  // (Typed into, let go of as it goes, the keyboard down with it: iOS draws a field's caret apart, left behind.)
  else if (sheet.contains(document.activeElement) && typing(document.activeElement)) /** @type {HTMLElement} */ (document.activeElement).blur();
  // (Its own way, back as the drag let go of it, or out as it closed, gives way: a transition outranks an animation.
  // The rest of its way out (its display, its visibility, kept till it's done) held to begin with the slide.)
  const own = sheet.getAnimations().filter((animation) => animation instanceof CSSTransition);
  for (const animation of own) if (/** @type {CSSTransition} */ (animation).transitionProperty === "transform") animation.cancel();
  if (reduced()) return void (later && close());
  const { frames, duration, easing } = away(), timing = /** @type {KeyframeAnimationOptions} */ ({ duration, easing, fill: "forwards" });
  const slide = sheet.animate(frames, timing);
  slide.id = "away";
  let fade = /** @type {Animation | null} */ (null);
  try {
    if (later) fade = sheet.animate({ opacity: [1, 0] }, { ...timing, pseudoElement: "::backdrop" });
  } catch {}
  slides.set(sheet, slide);
  beginOnTime(slide, ...(fade ? [fade] : []), ...own.filter((animation) => animation.playState !== "idle"));
  // (Opened or closed meanwhile, another way: let go of. A sheet's by its attributes, a dialog's as it toggles (a
  // popover's state is none of them).)
  let changed = false;
  const meanwhile = new MutationObserver(() => slide.cancel()), toggled = () => ((changed = true), slide.cancel());
  meanwhile.observe(sheet, { attributes: true, attributeFilter: ["open", "hidden"] });
  meanwhile.observe(document.body, { attributes: true, attributeFilter: ["data-sheet"] });
  sheet.addEventListener("beforetoggle", toggled);
  const unheard = () => (meanwhile.disconnect(), sheet.removeEventListener("beforetoggle", toggled));
  slide.finished.then(() => {
    // (Opened or closed in the very task it ended in: its change not heard yet, but there.)
    changed ||= meanwhile.takeRecords().length > 0;
    unheard();
    if (later && !changed) close();
    // (Closed, at its closed place at once, out of sight already: its own way there, from where it went, not taken.)
    for (const animation of ownSlides(sheet)) animation.finish();
    slide.cancel();
    fade?.cancel();
  }, () => (unheard(), fade?.cancel()));
}
/**
 * A sheet pressed as it slides away: stopped where it shows, open; how far down from its rest it shows then (as it
 * slides in, its own motion's). @param {HTMLElement} sheet
 */
function sheetTaken(sheet) {
  const slide = slides.get(sheet), away = slide?.id === "away" && slide.playState !== "finished", shown = sheet.getBoundingClientRect().top;
  if (away) slide.cancel();
  return new DOMMatrix(getComputedStyle(sheet).transform).m42 + (away ? shown - sheet.getBoundingClientRect().top : 0);
}
/**
 * A phone's sheet of two heights, whole (88% of the screen) and half (56%), dragged by its top: `grab` says where a
 * press takes it, by its handle or by its head (or not at all); `pulls`, whether its content pulls it too (see
 * dragSheet). Up from half it grows, down it follows the finger.
 * Up past whole, it stretches, less and less. Let go upright, it stops at whole, half or gone, the nearest to where
 * it'd come to rest (restOf), never back against a flick; its handle tapped, the other height. On its side (a
 * column), as sheetGoes says, it goes; else it comes back. `whole` and `size` read and set which height it has (its
 * motion is sizeSheet's, from where it was let go), `close` closes it (slid away first: slideAway); `scrim`, dimmed
 * behind it whole, lightens as it goes down.
 * @param {HTMLElement} sheet
 * @param {{ grab: (event: PointerEvent) => "handle" | "head" | false, whole: () => boolean, size: (whole: boolean) => void, close: () => void, scrim?: HTMLElement, pulls: () => boolean }} how
 */
function heightsSheet(sheet, { grab, whole, size, close, scrim, pulls }) {
  const [SHEET_WHOLE, SHEET_HALF] = ["--sheet-whole", "--sheet-half"].map((name) => Number(getComputedStyle(document.documentElement).getPropertyValue(name)));
  // (Its top as it shows: where a release moves it from. `held`: its height as it showed, taken mid-motion; `base`,
  // how far down it showed, taken as it slid in.)
  let height = 0, handle = false, shown = 0, held = "", base = 0;
  dragSheet(sheet, {
    pulls,
    grab: (event) => {
      // (Its content's pull, as its head's.)
      const by = event ? grab(event) : "head";
      if (!by) return false;
      shown = sheet.getBoundingClientRect().top;
      // (Taken as it moves to a height: stopped where it shows, as tall as it shows, the finger's from there.)
      const slide = slides.get(sheet);
      held = "";
      if (slide && slide.id !== "away" && slide.playState !== "finished") {
        slide.cancel();
        held = sheet.style.height = `${document.documentElement.clientHeight - shown}px`;
      }
      // (Taken as it slides in or away: from where it shows, not from its stop.)
      base = sheetTaken(sheet);
      (height = sheet.offsetHeight), (handle = by === "handle");
      sheet.style.transition = "none";
      if (scrim) scrim.style.transition = "none";
      return true;
    },
    move: (moved) => {
      // (A drag that turns back grows it or moves it as it is now, never both; past whole, stretched.)
      const by = moved + base, grows = by < 0 && heldUpright.matches, down = Math.max(0, by), tallest = innerHeight * SHEET_WHOLE;
      sheet.style.maxHeight = grows || held ? "none" : "";
      sheet.style.height = grows ? `${height - by <= tallest ? height - by : tallest + stretched(height - by - tallest)}px` : held;
      sheet.style.transform = grows ? "" : `translateY(${down}px)`;
      shown = sheet.getBoundingClientRect().top;
      if (scrim && whole()) scrim.style.opacity = String(1 - down / sheet.offsetHeight);
    },
    back: () => {
      sheet.style.transition = sheet.style.transform = sheet.style.height = sheet.style.maxHeight = "";
      if (scrim) scrim.style.transition = scrim.style.opacity = "";
    },
    release: (moved, speed) => {
      // (A press let go where it was taken, by the finger's move, not by where it showed (one as it slides in is
      // one too): its handle's switches the height; any other, it goes on where it was going: by its own motion as
      // it slid in, its slide to a height taken up again from where it shows.)
      const by = moved + base, still = Math.abs(moved) < 4;
      const away = () => slideAway(sheet, close, shown, speed);
      if (handle && still && heldUpright.matches) return sizeSheet(sheet, () => size(!whole()), shown);
      if (still) return void (held && sizeSheet(sheet, () => {}, shown));
      if (!heldUpright.matches) return void (sheetGoes(by, speed, height) && away());
      // (The stop nearest where it'd come to rest, of those on its way: a flick down never brings it back up.)
      const now = document.documentElement.clientHeight - shown, rest = height - restOf(by, speed), stops = [innerHeight * SHEET_WHOLE, innerHeight * SHEET_HALF, 0];
      const onWay = stops.filter((each) => (speed > 0.1 ? each <= now + 1 : speed < -0.1 ? each >= now - 1 : true));
      const near = (onWay.length ? onWay : stops).reduce((best, each) => (Math.abs(each - rest) < Math.abs(best - rest) ? each : best));
      if (near === 0) return away();
      sizeSheet(sheet, () => size(near === stops[0]), shown, speed);
    },
  });
}
/**
 * The layers' sheet whole or half (a phone's, upright). Closed meanwhile (mid-drag, by esc or Back), it isn't: it
 * opens again at half. @param {boolean} whole
 */
const panelSize = (whole) => document.body.dataset.sheet === "panel" && (whole ? (document.body.dataset.sheetSize = "full") : delete document.body.dataset.sheetSize);
/**
 * The arrows, Home and End among a list's cards (the grid's, the gallery's), as they're laid out.
 * @param {HTMLElement} list @param {KeyboardEvent} event @param {HTMLElement[]} cards
 */
function walkCards(list, event, cards = all$(CARDS, list)) {
  const at = cards.indexOf(/** @type {HTMLElement} */ (document.activeElement));
  if (at < 0 || !["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", "Home", "End"].includes(event.key)) return;
  event.preventDefault();
  if (event.key === "Home") return cards[0].focus();
  if (event.key === "End") return lastCard(list).focus();
  nextCard(cards[at], event.key, list)?.focus();
}
/** A list's last card, its last shelf's row (or grid) written whole first (its last tiles may be slots yet). @param {HTMLElement} list */
const lastCard = (list) => (((row) => row && fillRow(row))(all$(".shelf-row, .grid-tiles", list).at(-1)), /** @type {HTMLElement} */ (all$(CARDS, list).at(-1)));
/** What the arrows go along in the grid: the cards (one still waited for, not yet), and a shelf's last tile, to them all. */
const CARDS = ".tile:not(.waiting) > .tile-open, .shelf-all";
/**
 * The card an arrow goes to from another: ← → the one before or after; ↑ ↓ the nearest row above or below, as laid
 * out, its card nearest across (sections break the columns). @param {HTMLElement} from
 */
function nextCard(from, /** @type {string} */ key, within = $("home-grid")) {
  const cards = all$(CARDS, within), at = cards.indexOf(from);
  if (key === "ArrowLeft" || key === "ArrowRight") return cards[at + (key === "ArrowLeft" ? -1 : 1)] ?? null;
  const box = (/** @type {HTMLElement} */ card) => card.getBoundingClientRect(), here = box(from), down = key === "ArrowDown";
  const beyond = cards.filter((card) => (down ? box(card).top > here.bottom - 1 : box(card).bottom < here.top + 1));
  if (!beyond.length) return null;
  const row = down ? Math.min(...beyond.map((card) => box(card).top)) : Math.max(...beyond.map((card) => box(card).top));
  const inRow = beyond.filter((card) => Math.abs(box(card).top - row) < 2);
  return inRow.reduce((best, card) => (Math.abs(box(card).left - here.left) < Math.abs(box(best).left - here.left) ? card : best));
}
/** A drawing's points placed at `x(i)`, `y(value)`: the runs of "x,y" between its gaps, each of two points or more. */
function runs(ys, x, y) {
  const parts = /** @type {string[][]} */ ([[]]);
  ys.forEach((value, i) => (value === null ? parts.push([]) : parts[parts.length - 1].push(`${x(i).toFixed(1)},${y(value).toFixed(1)}`)));
  return parts.filter((part) => part.length > 1);
}
/**
 * A run's stretches above a level (`y0`, as drawn) and below it, cut where the run crosses it: a baseline's two
 * colors. @param {string[]} run its "x,y" points
 */
function sides(run, y0) {
  const out = /** @type {{ above: { part: string[], from: number }[], below: { part: string[], from: number }[] }} */ ({ above: [], below: [] });
  const points = run.map((point) => point.split(",").map(Number));
  // On the base at first, it's on the side it goes to.
  let part = /** @type {string[]} */ ([]), side = /** @type {"above" | "below"} */ ((points.find(([, y]) => y !== y0)?.[1] ?? y0) > y0 ? "below" : "above"), along = 0, from = 0;
  points.forEach(([x, y], i) => {
    const now = y < y0 ? "above" : y > y0 ? "below" : side;
    if (i) {
      const [px, py] = points[i - 1];
      if (now !== side) {
        const t = (y0 - py) / (y - py), cx = px + (x - px) * t;
        along += Math.hypot(cx - px, y0 - py);
        part.push(`${cx.toFixed(1)},${y0.toFixed(1)}`);
        if (part.length > 1) out[side].push({ part, from });
        part = [`${cx.toFixed(1)},${y0.toFixed(1)}`];
        from = along;
        along += Math.hypot(x - cx, y - y0);
      } else along += Math.hypot(x - px, y - py);
    }
    side = now;
    part.push(`${x},${y}`);
  });
  if (part.length > 1) out[side].push({ part, from });
  return out;
}
/** A few bars from a drawing's points (each its share's farthest from the base, and where that is), above the base in one color, under it in the other. */
const barsOf = (ys, n, base, place) => Array.from({ length: n }, (_, k) => {
  let far = -1;
  for (let i = Math.floor((k * ys.length) / n); i < Math.floor(((k + 1) * ys.length) / n); i++) if (ys[i] !== null && (far < 0 || Math.abs(ys[i] - base) > Math.abs(ys[far] - base))) far = i;
  return far < 0 ? "" : place(k, ys[far], far);
}).join("");
/** Marks as drawn, by the mini chart they're drawn from and the theme: the list written again, its marks aren't. */
const marksDrawn = new WeakMap();
/** A chart's mark: its mini chart once drawn, a faint line until then. @param {Chart} doc */
const chartMark = (doc) => {
  const thumb = thumbs.get(doc.id), kept = thumb && marksDrawn.get(thumb);
  if (kept?.light === light) return kept.svg;
  const svg = markOf(thumb ?? NO_MARK);
  if (thumb) marksDrawn.set(thumb, { light, svg });
  return svg;
};
/** The points of a run of a line's area: the run, closed down (or up) to `edge` under its ends. @param {string[]} part @param {number} edge */
const areaPoints = (part, edge) => `${part[0].split(",")[0]},${edge} ${part.join(" ")} ${part.at(-1)?.split(",")[0]},${edge}`;
/** Small charts' writers: a run's area (see areaPoints), a fill fading down a viewBox, the area under a line filled by it, a line. */
const areaOf = (/** @type {string[]} */ part, /** @type {number} */ edge, /** @type {string} */ color, opacity = ".2") => `<polygon points="${areaPoints(part, edge)}" fill="${color}" fill-opacity="${opacity}"/>`;
const fadeDef = (/** @type {string} */ id, /** @type {string} */ color, /** @type {number | string} */ top) => `<linearGradient id="${id}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="${escapeHtml(color)}" stop-opacity="${top}"/><stop offset="1" stop-color="${escapeHtml(color)}" stop-opacity="0"/></linearGradient>`;
const areaUnder = (/** @type {string} */ id, /** @type {string[]} */ points, /** @type {number} */ w, /** @type {number} */ h) => `<polygon points="0,${h} ${points.join(" ")} ${w},${h}" fill="url(#${id})"/>`;
const lineOver = (/** @type {string[]} */ points, /** @type {string} */ stroke, opacity = 1) => `<polyline points="${points.join(" ")}" fill="none" stroke="${escapeHtml(stroke)}"${opacity < 1 ? ` stroke-opacity="${opacity}"` : ""} stroke-width="1.5" stroke-linejoin="round" vector-effect="non-scaling-stroke"/>`;
/** A mark not drawn yet: a faint line. @type {Pick<Thumb, "lines" | "washes" | "dots">} */
const NO_MARK = { lines: [], washes: [], dots: [] };
/** A mark drawn from a mini chart: its lines, a heatmap's wash under them, bubbles' dots over them. @param {Pick<Thumb, "lines" | "washes" | "dots">} mark */
const markOf = ({ lines, washes, dots }) => {
  const y = (value) => 36 - value * 32, stroke = 'fill="none" stroke-width="1.4" vector-effect="non-scaling-stroke" stroke-linejoin="round" stroke-linecap="round"';
  const polyline = (part, color) => `<polyline points="${part.join(" ")}" stroke="${color}" ${stroke}/>`;
  // Each line as its type: an area over a light fill, a baseline in its two colors about its base, bars, dots.
  const paths = lines.map(([raw, ys, kind, under, level, dated]) => {
    level ??= 0;
    const color = escapeHtml(ink(raw)), below = escapeHtml(ink(under ?? raw)), x = (i) => 2 + (i / (ys.length - 1)) * 116, base = y(level);
    // (Painted date by date: a bar, a dot, a stretch of line in its points' color.)
    const tintAt = (/** @type {number} */ i) => escapeHtml(ink(dated?.[i] ?? raw));
    if (kind === "bars") return barsOf(ys, 10, level, (k, value, i) => `<rect x="${(4 + k * 11.6).toFixed(1)}" y="${Math.min(y(value), base).toFixed(1)}" width="7" height="${Math.max(1.5, Math.abs(y(value) - base)).toFixed(1)}" fill="${dated ? tintAt(i) : value >= level ? color : below}"/>`);
    if (kind === "dots") return byColor(ys.map((value, i) => (value === null || i % 2 ? null : { i, value })), ({ i }) => tintAt(i), false).map((piece) => `<path d="${piece.points.map(({ i, value }) => `M${x(i).toFixed(1)} ${y(value).toFixed(1)}h0`).join("")}" stroke="${piece.color}" stroke-width="2.2" stroke-linecap="round" vector-effect="non-scaling-stroke"/>`).join("");
    if (dated) return byColor(ys.map((value, i) => (value === null ? null : { i, value })), ({ i }) => tintAt(i)).filter((piece) => piece.points.length > 1).map(({ color: stroke, points }) => {
      const part = points.map(({ i, value }) => `${x(i).toFixed(1)},${y(value).toFixed(1)}`);
      return (kind === "area" ? areaOf(part, base, stroke) : "") + polyline(part, stroke);
    }).join("");
    return runs(ys, x, y).map((part) => {
      if (kind === "baseline") {
        const { above, below: under } = sides(part, base);
        return [...above.map((piece) => polyline(piece.part, color)), ...under.map((piece) => polyline(piece.part, below))].join("");
      }
      const fill = kind === "area" ? areaOf(part, base, color) : "";
      return fill + polyline(part, color);
    }).join("");
  }).join("");
  // (A heatmap's wash about its line, faint: its color over it, its other under it, to its band's top and foot.)
  const wash = washes.map(([k, over, under, bottom, top]) => ((ys) => runs(ys, (i) => 2 + (i / (ys.length - 1)) * 116, y).map((part) => areaOf(part, y(top), escapeHtml(ink(over))) + areaOf(part, y(bottom), escapeHtml(ink(under)))).join(""))(lines[k][1])).join("");
  // (Bubbles' largest, dots on their line, as a dots line's are.)
  const dotted = dots.map(([color, across, up]) => `<path d="M${(2 + across * 116).toFixed(1)} ${y(up).toFixed(1)}h0" stroke="${escapeHtml(ink(color))}" stroke-width="2.6" stroke-linecap="round" vector-effect="non-scaling-stroke"/>`).join("");
  return `<svg class="chart-mark" viewBox="0 0 120 40" preserveAspectRatio="none" aria-hidden="true">${wash}${paths || `<path d="M4 30H116" stroke="currentColor" stroke-opacity=".35" stroke-width="1.4" vector-effect="non-scaling-stroke" stroke-dasharray="3 3"/>`}${dotted}</svg>`;
};
/** A profile's mark's bars, from the edge, each split in its colors (its first two groups', or a group's), the first at the edge. @param {string[]} colors */
const profileMark = (colors) => [[7, 1.5], [12, 4.75], [5, 8]].map(([length, at]) => colors.map((color, c) => ((part) => `<rect x="${(15 - part * (c + 1)).toFixed(1)}" y="${at}" width="${part.toFixed(1)}" height="2.5" fill="${escapeHtml(color)}"/>`)(length / colors.length)).join("")).join("");
/**
 * A layer's own mark in the legend: its shape across its history, in its color and drawn as its type (bars for a
 * histogram, a filled line for an area), from its chart's drawing; a faint line until there's one. Kept from the
 * last drawing while a new one is made, so an edit doesn't blank it.
 */
function seriesMark(layer) {
  const thumb = thumbs.get(chart.id), ys = thumb && Object.hasOwn(thumb.marks, layer.id) ? thumb.marks[layer.id] : undefined;
  const level = thumb?.bases && Object.hasOwn(thumb.bases, layer.id) ? thumb.bases[layer.id] : 0;
  const now = layer.type === "histogram" && layer.colors === 4 ? null : colorNow(layer), color = escapeHtml(ink(now ?? layer.color)), below = now ? color : escapeHtml(ink(belowOf(layer)));
  const span = (ys?.length ?? 2) - 1, x = (i) => 0.8 + (i / span) * 14.4, y = (value) => 11 - value * 10;
  let body = `<path d="M1.5 6.5h13" stroke="${color}" stroke-opacity=".4" stroke-width="1.4" stroke-dasharray="2 2"/>`;
  // (A gradient's, a bar of its colors, low to high, whatever its type.)
  const stops = gradientOfLayer(chart, layer);
  if (stops) body = ((table) => Array.from({ length: 8 }, (_, k) => `<rect x="${(1 + k * 1.75).toFixed(2)}" y="3.5" width="1.85" height="6" fill="${escapeHtml(table[placeIndex(k / 7, table.length)])}"/>`).join(""))(gradientTable(stops));
  // A cloud's, a band in its color.
  else if (layer.type === "cloud") body = `<path d="M1 4.5c2.4-2 4.6-2.4 7-1.3s4.6 1 7-1.6v6.2c-2.4 2.6-4.6 2.6-7 1.5S3.4 8.1 1 10Z" fill="${color}" fill-opacity=".35"/>`;
  // (A signal's, its mark.)
  else if (layer.type === "signal") body = markBody(layer.place, color);
  // (Bubbles', two: one in each of their colors, below zero and above.)
  else if (layer.type === "bubbles") body = `<circle cx="4.5" cy="7.5" r="2.5" fill="${below}" fill-opacity=".4" stroke="${below}"/><circle cx="11" cy="5.5" r="4" fill="${color}" fill-opacity=".4" stroke="${color}"/>`;
  // (A profile's, bars from the edge, each split in its first two groups' colors, the first at the edge.)
  else if (layer.type === "profile") body = profileMark(layer.profile?.length ? layer.profile.slice(0, 2).map((cohort) => ink(profileColor(layer, cohort))) : [ink("gray")]);
  // (A heatmap's, cells of it: its unrealized P&L's, its loss over its profit.)
  else if (layer.type === "heatmap") body = [[0.3, 0.6, 1, 0.5], [0.6, 1, 0.45, 0.25], [1, 0.45, 0.2, 0.6]].map((row, r) => row.map((opacity, k) => `<rect x="${(1 + k * 3.6).toFixed(1)}" y="${(2 + r * 3.3).toFixed(1)}" width="3" height="2.7" rx=".5" fill="${layer.measure === "pnl" && !r ? below : color}" fill-opacity="${opacity}"/>`).join("")).join("");
  else if (ys && layer.type === "histogram" && modeOf(chart, layer) === 0) {
    // Six bars from its base: above it in its color, under it in the other.
    body = barsOf(ys, 6, level, (k, value) => `<rect x="${(0.5 + k * 2.6).toFixed(1)}" y="${Math.min(y(value), y(level)).toFixed(1)}" width="1.8" height="${Math.max(1.2, Math.abs(y(value) - y(level))).toFixed(1)}" rx=".4" fill="${value >= level ? color : below}"/>`);
  } else if (ys && layer.type === "dots") {
    body = ys.map((value, i) => (value === null || i % 4 !== 1 ? "" : `<circle cx="${x(i).toFixed(1)}" cy="${y(value).toFixed(1)}" r="1.2" fill="${color}"/>`)).join("");
  } else if (ys) {
    // Its line dashed as on the chart, in the mark's proportions (where the type has a line style).
    const [, , dash, gap] = LINED.has(layer.type) ? dashOf(layer.dash) : DASHES[0];
    const dashes = gap ? `stroke-dasharray="${+(dash * 0.8).toFixed(1)} ${+(gap * 0.8).toFixed(1)}" stroke-linecap="butt"` : 'stroke-linecap="round"';
    // In steps, each value held until the next.
    const stepped = (part) => (layer.shape === "steps" && LINED.has(layer.type) ? part.flatMap((point, i) => (i ? [`${point.split(",")[0]},${part[i - 1].split(",")[1]}`, point] : [point])) : part);
    const edge = layer.fill === "above" ? 0.5 : 11.5;
    const line = (part, stroke, from = 0) => `<polyline points="${part.join(" ")}" fill="none" stroke="${stroke}" stroke-width="1.4" stroke-linejoin="round" ${dashes}${gap && from ? ` stroke-dashoffset="${from.toFixed(1)}"` : ""}/>`;
    body = runs(ys, x, y).map(stepped).map((part) => {
      // A baseline in its two colors, parting at its base.
      if (layer.type === "baseline") {
        const { above, below: under } = sides(part, y(level));
        return [...above.map((piece) => line(piece.part, color, piece.from)), ...under.map((piece) => line(piece.part, below, piece.from))].join("");
      }
      return (layer.type === "area" ? areaOf(part, edge, color, ".22") : "") + line(part, color);
    }).join("");
  }
  return `<svg class="series-mark" viewBox="0 0 16 12" aria-hidden="true">${body}</svg>`;
}
/**
 * A chart's name: its own, else what it holds, untitled till named and never stored, so it follows its layers: its
 * first layer's, its first two's, or the first's and how many more.
 */
const titleOf = (doc) => doc.title || derivedTitle(doc);
const derivedTitle = (/** @type {Chart} */ doc) => {
  const names = (doc.layers ?? []).filter((layer) => layer.type !== "signal").map(labelOf);
  return !names.length ? "Untitled chart" : names.length === 1 ? names[0] : names.length === 2 ? `${names[0]} & ${names[1]}` : `${names[0]} & ${names.length - 1} more`;
};
/** The sidebar's list, written first where it's owed (⌘1–⌘9 read its rows on a phone's layout too). */
function sideList() {
  if (listOwed) writeSideList(home ? null : chart);
  return $("chart-list");
}
/** Set while what writes the sidebar's list moves its rows itself: a fold, a reorder, its width (see movingSide). */
let sideMoved = false;
/** The sidebar written by `change`, which moves its rows itself: they don't glide there again (see writeSideList). */
function movingSide(/** @type {() => void} */ change) {
  sideMoved = true;
  try {
    change();
  } finally {
    sideMoved = false;
  }
}
/** A count as the sidebar says it, short: whole thousands (1k, 11k, 111k), whole millions, never more than it is. */
const shortCount = (/** @type {number} */ n) => (n < 1000 ? String(n) : n < 1e6 ? `${Math.floor(n / 1000)}k` : `${Math.floor(n / 1e6)}m`);
/** Whether one of the sidebar's groups is closed there (on the rail too: its head's menu opens it). @param {Group} group */
const groupShut = (group) => !prefs.groupsOpen.includes(group);
/** A row's ⋯ cap: its menu found by `attr`, in its place's class (`chart-more`, `tile-more`, `shelf-more`). */
const moreCap = (/** @type {string} */ cls, /** @type {string} */ attr, /** @type {string | number} */ id, /** @type {string} */ name) => `<button type="button" class="cap small ${cls}" ${attr}="${escapeHtml(id)}" aria-haspopup="menu" aria-label="More for ${escapeHtml(name)}" data-tip="More">${ICONS.more}</button>`;
const viewCap = (/** @type {Place} */ place, /** @type {string} */ name) => `<button type="button" class="cap small chart-go" data-place="${escapeHtml(JSON.stringify(place))}" aria-label="View ${escapeHtml(name)}" data-tip="View">${ICONS.arrowRight}</button>`;
/** A chart's or a preset's (`kind`) row in the sidebar: its mark and `label` opening it (`tip`: its hover, else `name`), its ⋯. */
const leafRow = ({ kind, id, depth, cls = "", mark, label, labelCls = "", name, tip = name }) => `<div class="chart-row ${cls} ${depth ? "in-folder" : ""}" data-${kind}-row="${id}" style="--depth: ${depth}"><button type="button" class="nav-item chart-item" data-open-${kind}="${id}" data-tip="${escapeHtml(tip)}">${mark}<span class="label ${labelCls}">${escapeHtml(label)}</span></button>${moreCap("chart-more", `data-${kind}-menu`, id, name)}</div>`;
/** A folder's or a presets' section's (`kind`) row: its name opening or closing it (on the rail, its menu), its count, View, ⋯; closed holding the open chart (`holds`), it says so. */
const folderRow = ({ kind, id, depth, closed, holds, rail, mark, name, count, place }) => `<div class="folder-row ${kind === "section" ? "section-row" : ""} ${depth ? "in-folder" : ""} ${closed ? "closed" : ""} ${closed && holds ? "holds" : ""}" data-${kind}-row="${escapeHtml(id)}" style="--depth: ${depth}"><button type="button" class="nav-item folder-item" data-${kind}-item="${escapeHtml(id)}" ${rail ? 'aria-haspopup="menu"' : `aria-expanded="${!closed}"`} data-tip="${escapeHtml(name)}">${mark}<span class="label">${escapeHtml(name)}</span><span class="turn" aria-hidden="true">${ICONS.next}</span><span class="folder-count">${shortCount(count)}</span></button>${viewCap(place, name)}${moreCap("chart-more", `data-${kind}-menu`, id, name)}</div>`;
/**
 * A group's head in the sidebar, as a folder's row: a click opens or closes it there (kept); how many it holds, a
 * turn showing which, as it's pointed at; its ⋯ (a right-click), its page first. Closed, holding the open chart, it
 * says so (as a folder does; holding the page shown, see renderSideNav). Its count none till it's known (the presets
 * not read yet).
 * @param {Group} group @param {number | null} count @param {boolean} holds @returns {[string, string]}
 */
const sideHead = (group, count, holds) => {
  const [name, place] = GROUP_OF[group], shut = groupShut(group);
  return [`head:${group}`, `<div class="list-head ${shut ? "closed" : ""} ${shut && holds ? "holds" : ""}" data-group="${group}"><button type="button" class="list-head-name" data-group-item="${group}" data-at='${JSON.stringify(place)}' aria-expanded="${!shut}" aria-label="${count === null ? name : `${name}, ${count}`}" data-tip="${name}"><span class="head-mark">${ICONS[GROUP_MARK[group]]}</span><span class="head-name">${name}</span>${count === null ? "" : `<span class="count">${shortCount(count)}</span>`}<span class="turn" aria-hidden="true">${ICONS.next}</span></button>${viewCap(place, name)}${moreCap("chart-more", "data-group-menu", group, name)}</div>`];
};
/** A group's menu in the sidebar (its ⋯, a right-click): its page; open or close it there. @param {Group} group @returns {Action[]} */
const groupRows = (group) => [
  { key: "view", icon: ICONS.arrowRight, label: "View", aside: "Its page", run: () => goTo(asPlace(GROUP_OF[group][1])) },
  ((shut) => ({ key: "toggle", icon: shut ? ICONS.folderOpen : ICONS.folder, label: shut ? "Expand" : "Collapse", stay: true, run: () => toggleGroup(group) }))(groupShut(group)),
  // (Your Library's, as a folder's: what can be made in it.)
  // (And what comes in and goes out of it: a file of charts imported, all of it downloaded, as on its page.)
  ...(group === "library" ? [
    { key: "new-chart", icon: ICONS.plus, label: "New chart", kbd: "N", run: () => newChart() }, { key: "new-folder", icon: ICONS.folderAdd, label: "New folder", run: () => newFolder([], undefined, { side: true }) },
    { key: "import", icon: ICONS.upload, label: "Import charts", run: () => importCharts() },
    ...(anythingKept() ? [{ key: "download-all", icon: ICONS.download, label: "Download all charts", aside: "Folders and all", run: () => downloadAll() }] : []),
  ] : []),
];
/** One of the sidebar's groups opened or closed there, as a folder is (kept as left). @param {Group} group */
function toggleGroup(group) {
  const closing = !groupShut(group);
  prefs.groupsOpen = closing ? prefs.groupsOpen.filter((other) => other !== group) : [...prefs.groupsOpen, group];
  savePrefs();
  foldRow(() => /** @type {HTMLElement | null} */ ($("chart-list").querySelector(`[data-group="${group}"]`)), closing);
}
/**
 * The presets' sections as a tree, made once they're read: each with what's in it as made (its sections and its
 * own presets, in the presets' order), every preset it holds at any depth (`under`, in their order), its key (its path, as kept), its name as
 * shown (see shownStep) and its path as shown (`shown`); and, once a search needs them, each preset's words (see presetSearch).
 * @typedef {{ path: string[], key: string, name: string, shown: string[], items: (PresetNode | number)[], under: number[], icon?: string }} PresetNode
 */
let presetRoot = /** @type {{ for: Preset[] | null, node: PresetNode | null, byKey: Map<string, PresetNode>, search: PresetSearch | null, making: Generator<void, PresetSearch> | null }} */ ({ for: null, node: null, byKey: new Map(), search: null, making: null });
/** The keys of the sections preset `i` is in, outermost first (as `presetTree` keys them). */
function sectionKeys(/** @type {number} */ i) {
  const path = /** @type {Preset[]} */ (presets)[i].p;
  return path.slice(0, -1).map((_, depth) => JSON.stringify(path.slice(0, depth + 1)));
}
function presetTree() {
  if (!presets) return null;
  if (presetRoot.for === presets) return presetRoot.node;
  const root = /** @type {PresetNode} */ ({ path: [], key: "[]", name: "Presets", shown: [], items: [], under: [] }), byKey = new Map([[root.key, root]]);
  presets.forEach((preset, i) => {
    let node = root;
    node.under.push(i);
    for (let depth = 0; depth < preset.p.length - 1; depth++) {
      const path = preset.p.slice(0, depth + 1), key = JSON.stringify(path);
      let next = byKey.get(key);
      if (!next) {
        // (A top section's icon, on its first preset: see presets.js.)
        const name = shownStep(presetLabels, path);
        next = { path, key, name, shown: [...node.shown, name], items: [], under: [], ...(!depth && preset.s && ICONS[preset.s] ? { icon: preset.s } : {}) };
        byKey.set(key, next);
        node.items.push(next);
      }
      node = next;
      node.under.push(i);
    }
    node.items.push(i);
  });
  presetRoot = { for: presets, node: root, byKey, search: null, making: null };
  return root;
}
/**
 * What a search of the presets reads, made from the tree after they're read, a slice at a time in idle moments (none
 * of it at the start, nor a long task: see searchNow), the rest at once when a search comes first. Each preset's
 * words (its title, its place, its places' steps as shown where they're shown otherwise, and its own name as shown,
 * lowercased): as shown (`shown`: what a slip's taken for, see slipsOf), as a search reads them (`texts`, see
 * searchText), and their consecutive initials (`initials`); its title as a search reads it, its name and its subject (its title
 * before ": "), lowercased (`titles`); its rank among those alike in fit (see presetRankOf), and the presets in that
 * order (`order`); those whose name or subject is a search exactly, or whose whole initials it is, by it (`exact`).
 * @typedef {{ shown: string[], texts: string[], initials: string[], titles: { title: string, initials: string, name: string, subject: string }[], rank: Int32Array, order: Int32Array, exact: Map<string, number[]> }} PresetSearch
 * @param {Preset[]} list @param {PresetNode} root @returns {Generator<void, PresetSearch>}
 */
function* searchMaking(list, root) {
  // (Each one's places' names shown otherwise than their steps, by a walk down the tree, at once: a step a section.)
  const named = /** @type {string[][]} */ (new Array(list.length));
  const walk = (/** @type {PresetNode} */ node, /** @type {string[]} */ names) => {
    for (const item of node.items) {
      if (typeof item === "number") named[item] = names;
      else walk(item, item.name === item.path.at(-1) ? names : [...names, item.name]);
    }
  };
  walk(root, []);
  const shown = /** @type {string[]} */ (new Array(list.length)), texts = /** @type {string[]} */ (new Array(list.length)), initials = /** @type {string[]} */ (new Array(list.length)), titles = /** @type {PresetSearch["titles"]} */ (new Array(list.length));
  for (let i = 0; i < list.length; i++) {
    const preset = list[i], own = presetLabels[JSON.stringify(preset.p)];
    // (Its pieces each a phrase: words paired within one, never across a place's steps.)
    const pieces = [preset.t, ...preset.p, ...named[i], ...(own ? [own] : [])].map(searchCase), title = pieces[0];
    shown[i] = pieces.join(" ");
    texts[i] = searchText(shown[i], pieces);
    initials[i] = initialsOf(pieces);
    titles[i] = { title: searchText(title), initials: initials[i].split(" ", 1)[0], name: searchCase(presetName(preset)), subject: title.split(": ")[0] };
    if (i % 250 === 249) yield;
  }
  // (Counted into place, not sorted: no comparison called thousands of times, a phone's engine without a JIT.)
  const rank = Int32Array.from(list, (preset) => presetRankOf(preset.p.length, preset.t)), starts = new Int32Array(Math.max(0, ...rank) + 2), order = new Int32Array(list.length);
  for (let i = 0; i < rank.length; i++) starts[rank[i] + 1]++;
  for (let r = 1; r < starts.length; r++) starts[r] += starts[r - 1];
  for (let i = 0; i < rank.length; i++) order[starts[rank[i]]++] = i;
  const exact = /** @type {Map<string, number[]>} */ (new Map());
  for (let i = 0; i < titles.length; i++) {
    for (const said of new Set([titles[i].name, titles[i].subject, ...[titles[i].name, titles[i].subject].map((text) => initialsOf([text])).filter(initialsWord)])) {
      const found = exact.get(said);
      if (found) found.push(i);
      else exact.set(said, [i]);
    }
    if (i % 250 === 249) yield;
  }
  return { shown, texts, initials, titles, rank, order, exact };
}
/** A preset's rank among those alike in a search's fit: the nearest the top of the tree, then the shortest title. */
const presetRankOf = (/** @type {number} */ depth, /** @type {string} */ title) => depth * 1000 + Math.min(title.length, 999);
/** A slice of the presets' search's making (see searchMaking): true once it's made. */
function searchStep() {
  presetTree();
  if (presetRoot.search) return true;
  const step = (presetRoot.making ??= searchMaking(/** @type {Preset[]} */ (presets), /** @type {PresetNode} */ (presetRoot.node))).next();
  if (step.done) (presetRoot.search = step.value), (presetRoot.making = null);
  return Boolean(step.done);
}
/** The presets' search (see searchMaking), the rest of its making at once. Once the presets are read. */
function presetSearch() {
  while (!searchStep());
  return /** @type {PresetSearch} */ (presetRoot.search);
}
/** The presets' search made in idle moments, a slice each (see inSlices); then the slips' words (see vocabNow). */
function searchNow(/** @type {{ timeRemaining: () => number } | undefined} */ deadline) {
  inSlices(deadline, searchStep, searchNow);
  if (presetRoot.search) whenIdle(vocabNow);
}
/** A preset's name as shown (see shownStep). @param {Preset} preset */
const presetName = (preset) => shownStep(presetLabels, preset.p);
/** A section of the presets as the sidebar has it (see presetTree), by its path; none, gone. */
function presetNodeAt(/** @type {string[]} */ path) {
  return presetTree() ? presetRoot.byKey.get(JSON.stringify(path)) : undefined;
}
/** A section's steps as shown (see shownStep); their names till the presets are read. @param {string[]} path */
function placeNames(path) {
  return presetNodeAt(path)?.shown ?? path;
}
/** How many of the presets' marks are kept in this browser (see PRESET_MARKS). */
const KEPT_PRESET_MARKS = 200;
/** Each preset's own tag, by what it is (its layers, its look): a mark kept of it as it was, not taken for it as it is now. @type {WeakMap<Preset, string>} */
const presetTags = new WeakMap();
// (By the preset, as made (as it is now); edited here, by its copy as it is: nothing else changes it.)
const keptMarkKey = (/** @type {Chart} */ doc) => drawingKey([doc.id, doc.updated]);
const presetMarkKey = (/** @type {number} */ i) => ((doc) => (doc ? keptMarkKey(doc) : ((preset) => drawingKey([presetKey(i), presetTags.get(preset) ?? (presetTags.set(preset, hashOf(JSON.stringify(preset))), presetTags.get(preset))]))(/** @type {Preset[]} */ (presets)[i])))(keptPreset(i));
/** A preset's mark, as its row shows it: drawn, else the empty one till it is. */
const presetMark = (/** @type {number} */ i) => markOf(presetMarks.get(presetMarkKey(i)) ?? NO_MARK).replace("<svg ", `<svg data-preset-mark="${i}" `);
/** The preset rows in sight, their marks drawn one at a time; one gone out of sight before its turn, let go of. */
const presetMarkQueue = /** @type {Set<number>} */ (new Set());
/** The marks ⌘K's rows asked for, not drawn yet: its rows written again, let go of. */
const paletteMarksAsked = /** @type {Set<number>} */ (new Set());
const forgetPaletteMarks = () => (paletteMarksSeen.disconnect(), paletteMarksAsked.forEach((i) => presetMarksAsked.delete(i)), paletteMarksAsked.clear());
/**
 * ⌘K's rows' preset marks as they're written: drawn since their rows were made (a row's icon as it was then), drawn;
 * not drawn yet, watched till they're in sight. @param {ParentNode} rows
 */
const seePaletteMarks = (rows) => {
  for (const mark of rows.querySelectorAll("[data-preset-mark]")) {
    const i = Number(/** @type {HTMLElement} */ (mark).dataset.presetMark);
    if (!presetMarks.has(presetMarkKey(i))) paletteMarksSeen.observe(mark);
    else if (!mark.classList.contains("chart-mark")) mark.outerHTML = presetMark(i);
  }
};
/** The presets a menu shows, their marks to draw (apart from the sidebar's rows in sight, which it clears). */
const presetMarksAsked = /** @type {Set<number>} */ (new Set());
/** The sections the open preset (`at`) is in, opened in the sidebar while it's the one open, less any closed there: shown, never kept (`presetsOpen`). */
let revealed = { at: -1, keys: /** @type {Set<string>} */ (new Set()) };
const sectionShown = (/** @type {string} */ key) => prefs.presetsOpen.includes(key) || revealed.keys.has(key);
/**
 * The presets in the sidebar, under your Library, under their name: their sections as folders are (a section's name
 * opens or closes it here, as kept), the presets in an open one as charts are (one opens it).
 * @returns {[string, string][]}
 */
function presetParts() {
  const root = presetTree();
  // (Not read yet: as they were last drawn, with the same sections open (the group open, not on the rail), from the
  // first frame, not come in after the rest, out of reach till the presets' own replace them (a moment: what they'd
  // run needs the presets); else, or they couldn't be read, its head alone, what it holds counted once it's read.)
  if (!root || !presets) return sidePresets && !presetsFailed && !onRail() && !groupShut("presets") && sidePresets.open === JSON.stringify(prefs.presetsOpen) ? sidePresets.parts.map(([key, html]) => [key, html.replace(/^<div /, "<div inert ")]) : [sideHead("presets", null, false)];
  // (The sections the open preset is in: opened for the view (see revealed); one closed holds it, marked as a folder
  // holding the open chart is; the head too, only so (a preset's chart whose place is gone is in none of them).)
  const at = !home && chart.preset !== undefined ? presetIndex(chart) : -1, holding = new Set(at < 0 ? [] : sectionKeys(at));
  if (at !== revealed.at) revealed = { at, keys: new Set(holding) };
  // (On the rail, none open: see renderCharts.)
  const rail = onRail(), open = new Set(rail ? [] : [...prefs.presetsOpen, ...revealed.keys]);
  /** @type {[string, string][]} */
  const parts = [sideHead("presets", presets.length, at >= 0)];
  if (groupShut("presets")) return parts;
  const walk = (/** @type {PresetNode} */ node, /** @type {number} */ depth) => {
    for (const item of node.items) {
      if (typeof item === "number") {
        if (!depth) continue;
        // (In its folders, by its own name, the folders saying the rest; its title on hover. Retitled here, by that.)
        const preset = /** @type {Preset[]} */ (presets)[item], doc = keptPreset(item), edited = Boolean(doc && presetEdited(doc)), title = doc ? titleOf(doc) : preset.t;
        parts.push([`preset:${item}`, leafRow({ kind: "preset", id: item, depth, cls: `preset-row ${edited ? "edited" : ""}`, mark: presetMark(item), label: title !== preset.t ? title : presetName(preset), name: title, tip: `${title}${edited ? " (edited)" : ""}` })]);
        continue;
      }
      const section = item, shown = open.has(section.key);
      parts.push([`section:${section.key}`, folderRow({ kind: "section", id: section.key, depth, closed: !shown, holds: holding.has(section.key), rail, mark: sectionMark(section, shown), name: section.name, count: section.under.length, place: { path: section.path } })]);
      if (shown) walk(section, depth + 1);
    }
  };
  walk(root, 0);
  // (Kept as drawn only with what's kept open: a reveal restored in a first frame would be one of another view's.)
  if (!rail && [...revealed.keys].every((key) => prefs.presetsOpen.includes(key))) keepSidePresets(parts);
  return parts;
}
/**
 * The presets' rows as the sidebar last drew them, open, and the sections open then (see presetParts): kept in this
 * browser, the first to go when it's full, as a picture's are.
 */
const SIDE_PRESETS = "bitview-studio:side-presets";
/** An item's key in the sidebar's order: a folder's ("f:…"), a chart's ("c:…"), at any depth. @param {Chart | Folder} item */
const sideKey = (item) => ("layers" in item ? `c:${item.id}` : `f:${item.id}`);
/** The folder an item's in (none: the top). @param {Chart | Folder} item */
const parentOf = (item) => ("layers" in item ? item.folder : item.parent);
/** An item put in a folder (none: the top). @param {Chart | Folder} item @param {string | undefined} parent */
const setParent = (item, parent) => {
  if ("layers" in item) parent ? (item.folder = parent) : delete item.folder;
  else parent ? (item.parent = parent) : delete item.parent;
};
/** A chart or a folder by its key ("c:…", "f:…"). */
const itemOf = (/** @type {string} */ key) => (key[0] === "f" ? folderById(key.slice(2)) : chartById(key.slice(2)));
/** Items by their keys, those still here. @returns {(Chart | Folder)[]} */
const itemsOf = (/** @type {string[]} */ keys) => keys.flatMap((key) => itemOf(key) ?? []);
/**
 * A folder's items (none given: the top's), in their order: its folders and its charts as arranged (one order for
 * all, each folder's among themselves). One new to it: a folder, first; a chart, after the one before it in the list
 * (first if none is: a new chart; a copy, after its own).
 * @returns {(Chart | Folder)[]}
 */
function childrenOf(/** @type {string | undefined} */ parent = undefined) {
  const folders = library.folders.filter((folder) => folder.parent === parent), charts = library.charts.filter((doc) => doc.folder === parent);
  const byKey = new Map([...folders, ...charts].map((item) => [sideKey(item), item]));
  const keys = library.order.filter((key) => byKey.has(key));
  let after = -1;
  for (const doc of charts) {
    const at = keys.indexOf(`c:${doc.id}`);
    if (at >= 0) after = at;
    else keys.splice(++after, 0, `c:${doc.id}`);
  }
  keys.unshift(...folders.map(sideKey).filter((key) => !keys.includes(key)));
  return keys.map((key) => /** @type {Chart | Folder} */ (byKey.get(key)));
}
/**
 * A folder's items in a new order (they're all one folder's): written last in the one order, each folder's own
 * among themselves as they were. @param {(Chart | Folder)[]} items
 */
const setChildren = (items) => ((keys) => (library.order = [...library.order.filter((key) => !keys.includes(key)), ...keys]))(items.map(sideKey));
/** Everything under a folder (none: the top), as the sidebar has it, each with how deep; `open`: not into a closed one. @returns {{ item: Chart | Folder, depth: number }[]} */
function treeOf(parent = /** @type {string | undefined} */ (undefined), open = false, depth = 0, /** @type {{ item: Chart | Folder, depth: number }[]} */ out = []) {
  for (const item of childrenOf(parent)) {
    out.push({ item, depth });
    if (!("layers" in item) && !(open && item.closed)) treeOf(item.id, open, depth + 1, out);
  }
  return out;
}
/** A folder of yours, by its id (none: gone, or the Library's top). */
const folderById = (/** @type {string | undefined} */ id) => library.folders.find((folder) => folder.id === id);
/** Whether a folder is `outer`, or in it, at any depth. */
const isIn = (/** @type {string | undefined} */ id, /** @type {string} */ outer) => {
  for (let at = id, steps = 0; at && steps <= library.folders.length; at = folderById(at)?.parent, steps++) if (at === outer) return true;
  return false;
};
/** How many charts each folder holds, at any depth: one pass, each chart counted up through the folders it's in. */
function chartCounts() {
  const byId = new Map(library.folders.map((folder) => [folder.id, folder])), counts = /** @type {Map<string, number>} */ (new Map());
  for (const doc of library.charts) for (let at = doc.folder, steps = 0; at && steps <= byId.size; at = byId.get(at)?.parent, steps++) counts.set(at, (counts.get(at) ?? 0) + 1);
  return counts;
}
/** A folder's name with those it's in ("Work › Cycles"). */
const folderPath = (/** @type {string} */ id) => {
  const names = [];
  for (let at = /** @type {string | undefined} */ (id), steps = 0; at && steps <= library.folders.length; at = folderById(at)?.parent, steps++) names.unshift(folderById(at)?.name ?? "");
  return names.join(" › ");
};
/** The charts ⌘1–9 open: the first nine the sidebar shows (one in a closed folder or a closed Library, out of sight, out of reach). */
/**
 * What ⌘1–⌘9 open: the first nine charts of the sidebar's topmost open group (Recent, the previews, yours, the
 * presets) with any, as it lists them (its open folders' and sections' too): their rows' buttons, pressed as a click
 * presses them. @returns {HTMLElement[]}
 */
const numbered = () => {
  const rows = [];
  let open = false;
  for (const row of sideList().children) {
    if (row.matches(".list-head")) {
      if (rows.length) break;
      open = !row.classList.contains("closed");
      continue;
    }
    const button = open ? row.querySelector("[data-open-recent], [data-open-chart], [data-open-preset]") : null;
    if (button && rows.push(/** @type {HTMLElement} */ (button)) === 9) break;
  }
  return rows;
};
/** Your charts and folders as the sidebar shows them: open, as a tree (a closed folder's out of sight); the rail, the top's alone. */
const sideTree = () => (onRail() ? childrenOf(undefined).map((item) => ({ item, depth: 0 })) : treeOf(undefined, true));
/** A folder made (in `parent`, first there; none: at the top), named at once where it shows (`keys`: what it starts with). */
function newFolder(/** @type {string[]} */ keys = [], /** @type {string | undefined} */ parent = undefined, { step = true, name = true, side = false } = {}) {
  const was = arrangement();
  const folder = /** @type {Folder} */ ({ id: uid(), name: "New folder", ...(parent && folderById(parent) ? { parent } : {}) });
  // (In the grid, its filter cleared: the folder shows, to be named.)
  if (home) $("home-filter").value = "";
  library.folders.push(folder);
  setChildren([folder, ...childrenOf(folder.parent).filter((item) => item !== folder)]);
  // (Your Library opened in the sidebar if it was closed there: named in place, where it is.)
  if (groupShut("library")) (prefs.groupsOpen = [...prefs.groupsOpen, "library"]), savePrefs();
  // (Where it is, in sight: the folders it's in opened.)
  for (let at = folder.parent; at; at = folderById(at)?.parent) delete /** @type {Folder} */ (folderById(at)).closed;
  moveToFolder(keys, folder.id, true);
  // (Undone: what went in back where each was, the folder gone; redone, made again, named as it was.)
  const made = step ? libraryStep("Made a folder", () => {
    const now = folderById(folderNow(folder.id));
    if (!now) return;
    const named = now.name;
    placeAs(was, itemsOf(keysNow(keys)));
    deleteFolder(now.id);
    return () => {
      const again = newFolder(keysNow(keys), parent && folderNow(parent), { name: false });
      folderAgain.set(now.id, again.id);
      again.name = named;
      save();
      renderCharts();
    };
  }, "Folder taken away") : null;
  if (made) made.folder = folder.id;
  save();
  renderCharts();
  // (Named where it was asked from: the sidebar's (on the rail, beside its row), else where it shows.)
  if (name) renameFolder(folder.id, side, true);
  return folder;
}
/**
 * A folder opened or closed in the sidebar (its icon, ← →, its menu): what's in it unfolds from under it, or folds
 * back into it, and what's under them follows. Its icon turns at once; closing, the list is drawn without its charts
 * once they've folded away.
 */
function toggleFolder(/** @type {string} */ id) {
  const folder = folderById(id);
  if (!folder) return;
  const closing = !folder.closed;
  if (closing) folder.closed = true;
  else delete folder.closed;
  save();
  foldRow(() => /** @type {HTMLElement | null} */ ($("chart-list").querySelector(`[data-folder-row="${id}"]`)), closing);
}
/** A section of the presets' menu in the sidebar (its ⋯, a right-click): its page; open or close it here. @returns {Action[]} */
const sectionRows = (/** @type {string} */ key) => [
  { key: "view", icon: ICONS.arrowRight, label: "View", aside: "Its page", run: () => goTo(asPlace({ path: JSON.parse(key) })) },
  ((open) => ({ key: "toggle", icon: open ? ICONS.folder : ICONS.folderOpen, label: open ? "Collapse" : "Expand", stay: true, run: () => toggleSection(key) }))(sectionShown(key)),
];
/** A section of the presets opened or closed in the sidebar, as a folder is (kept as left). @param {string} key */
function toggleSection(key) {
  const closing = sectionShown(key);
  revealed.keys.delete(key);
  prefs.presetsOpen = closing ? prefs.presetsOpen.filter((other) => other !== key) : [...prefs.presetsOpen, key];
  savePrefs();
  foldRow(() => all$("[data-section-row]", $("chart-list")).find((row) => row.dataset.sectionRow === key) ?? null, closing);
}
/**
 * A folder's row in the sidebar (yours, a section of the presets) opened or closed: what's under it unfolds from
 * under it, or folds back into it, and what's under them follows. Its icon turns at once; closing, the list is drawn
 * without them once they've folded away. (Its row found anew once drawn: opening draws the list again.)
 */
function foldRow(/** @type {() => HTMLElement | null} */ find, /** @type {boolean} */ closing) {
  let head = find();
  // (On the rail, nothing unfolds: kept as it's set, shown once the sidebar's open.)
  if (reduced() || !head || onRail()) return renderCharts();
  // (A group's head turns its arrow, as its class says.)
  const glyph = head.querySelector(".folder-glyph");
  if (glyph) glyph.innerHTML = closing ? ICONS.folder : ICONS.folderOpen;
  head.classList.toggle("closed", closing);
  head.querySelector("[aria-expanded]")?.setAttribute("aria-expanded", String(!closing));
  // (Opened while it closes: each row from the height it shows now, drawn anew as it may be.)
  const rowKey = (/** @type {HTMLElement} */ row) => row.dataset.chartRow ?? row.dataset.folderRow ?? row.dataset.sectionRow ?? row.dataset.presetRow ?? row.dataset.previewRow ?? "";
  const shown = new Map(closing ? [] : rowsUnder(head).filter((row) => row.getAnimations().length).map((row) => [rowKey(row), row.getBoundingClientRect().height]));
  if (!closing) {
    movingSide(renderCharts);
    head = find();
    if (!head) return;
  }
  const folds = foldRows(rowsUnder(head), closing, (row) => shown.get(rowKey(row)));
  // (The sidebar's highlight with its row as the rows slide, each frame till they're still.)
  let sliding = true;
  Promise.allSettled(folds.map((animation) => animation.finished)).then(() => {
    sliding = false;
    // (Still, the row under the pointer, the one it folded under lit at last.)
    const under = document.documentElement.classList.contains("can-hover") ? /** @type {HTMLElement | null} */ ($("chart-list").querySelector(":is(.chart-row, .folder-row):hover > :is(.chart-item, .folder-item)")) : null;
    if (under) sideLight(under);
    else sideLight.fit();
  });
  const follow = () => sliding && (sideLight.fit(), requestAnimationFrame(follow));
  requestAnimationFrame(follow);
  if (!closing) return;
  closings++;
  Promise.allSettled(folds.map((animation) => animation.finished)).then(() => {
    if (--closings) return;
    // (Its rows folded to nothing already: dropped with nothing to glide.)
    movingSide(renderCharts);
  });
}
/** A theme chosen: kept, named on its row, and on the page (see THEMES). @param {"system" | "light" | "dark"} theme */
function setTheme(theme) {
  prefs.theme = theme;
  savePrefs();
  renderTheme();
  applyTheme();
}
function renderTheme() {
  const [, name, icon] = THEMES.find(([id]) => id === prefs.theme) ?? THEMES[0];
  $("home-theme").innerHTML = ICONS[icon];
  $("home-theme").setAttribute("aria-label", ($("home-theme").dataset.tip = `Theme: ${name}`));
}
/** The themes, from their row: the one chosen ticked. */
function themeMenu(/** @type {HTMLElement} */ anchor) {
  menuOf(anchor, THEMES.map(([id, name]) => ({ key: id, label: name, current: prefs.theme === id, run: () => setTheme(/** @type {"system" | "light" | "dark"} */ (id)) })));
}
/** The sidebar's rows under a folder's (yours, a section's), at any depth (those after it, deeper than it). */
function rowsUnder(/** @type {HTMLElement} */ head) {
  // (A group's: all up to the next group's head.)
  if (head.classList.contains("list-head")) {
    const under = [];
    // (Your Library's New chart with them: what it makes is theirs.)
    for (let row = head.nextElementSibling; row && !row.classList.contains("list-head"); row = row.nextElementSibling) under.push(/** @type {HTMLElement} */ (row));
    return under;
  }
  const rows = all$(".chart-row, .folder-row", $("chart-list")), at = rows.indexOf(head);
  if (at < 0) return [];
  const depth = depthOf(rows[at]), under = [];
  for (const row of rows.slice(at + 1)) {
    if (depthOf(row) <= depth) break;
    under.push(row);
  }
  return under;
}
/** Folders closing now: the list's drawn without their charts once the last is done (one before would cut the others short). */
let closings = 0;
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
/**
 * Rows unfolding from nothing, or folding to it, as one edge moving down from the one above them (or back up to it)
 * on the layout's curve: each row opened (its gap, then itself) as the edge passes it, its words in place, fading in
 * over its own height; what's below moves with the edge. One under way turns back from where its edge is. All
 * measured first, then all moved (a long folder: one layout, not one a row). `was`: a row's height as shown before
 * it was drawn anew.
 * @param {HTMLElement[]} elements @param {boolean} closing @param {(row: HTMLElement) => number | undefined} was
 */
function foldRows(elements, closing, was = () => undefined) {
  // (Its gap from the one above, as wide as its list has it, goes and comes with it: a list's, read once.)
  const gaps = new Map(), gapOf = (/** @type {HTMLElement} */ element) => ((list) => gaps.get(list) ?? (gaps.set(list, parseFloat(getComputedStyle(list).rowGap) || 0), gaps.get(list)))(/** @type {HTMLElement} */ (element.parentElement));
  // (Under way: how much of it shows, its gap and height as they are.)
  const first = elements[0]?.getBoundingClientRect().top ?? 0, at = elements.map((element) => element.getBoundingClientRect().top - first);
  const showing = elements.map((element) => {
    const height = was(element) ?? (element.getAnimations().length ? element.getBoundingClientRect().height : undefined);
    if (height === undefined) return undefined;
    const margin = element.getAnimations().length ? parseFloat(getComputedStyle(element).marginTop) : height > 0 ? 0 : -gapOf(element);
    return height + gapOf(element) + margin;
  });
  for (const element of elements) for (const animation of element.getAnimations()) animation.cancel();
  let y = 0;
  const rows = elements.map((element, i) => {
    const gap = gapOf(element), full = element.offsetHeight, top = y;
    y += gap + full;
    return { element, gap, full, top, at: at[i], now: Math.min(showing[i] ?? (closing ? gap + full : 0), gap + full) };
  });
  // (As far as it shows: past the list's visible end (now, rows above it maybe part folded, and whole), rows open at the
  // last frame, out of sight by then, and fold at the first (as they are till then); the edge's time all spent in
  // sight, a long group's not run through in a blink.)
  const list = elements[0]?.parentElement, room = list ? Math.max(0, list.getBoundingClientRect().bottom - first) : 0;
  const past = (/** @type {{ top: number, at: number }} */ row) => row.top >= room && row.at >= room, near = rows.filter((row) => !past(row));
  const reach = Math.min(y, Math.max(room, ...near.map((row) => row.top + row.gap + row.full))), steps = Math.max(2, Math.ceil(MOTION.layout / 16));
  // (The edge from where it changes nothing at first: closing, the lowest shown's end; opening, the highest part-shown
  // one's. Each row then as the edge has it, never past what it shows now the other way: one folding inside, a
  // folder's, as it is till the edge reaches it.)
  const start = closing ? Math.max(0, ...near.map((row) => (row.now > 0 ? row.top + row.now : 0))) : Math.min(reach, ...near.map((row) => (row.now < row.gap + row.full ? row.top + row.now : reach)));
  const edges = Array.from({ length: steps + 1 }, (_, j) => start + ((closing ? 0 : reach) - start) * layoutCurve(j / steps));
  return rows.map((row) => {
    const { element, gap, full, top, now } = row;
    element.inert = closing;
    // (Clipped, its words from its top, as it goes: the animation's, nothing left on the row.)
    const frames = edges.map((edge, j) => {
      const passed = Math.min(Math.max(edge - top, 0), gap + full), shown = past(row) ? (j === 0 ? now : j === steps && !closing ? gap + full : 0) : closing ? Math.min(now, passed) : Math.max(now, passed), height = Math.max(shown - gap, 0);
      return { offset: j / steps, height: `${height}px`, marginTop: `${Math.min(shown, gap) - gap}px`, opacity: full ? height / full : 1, overflow: "clip", overflowClipMargin: "1px", alignItems: "flex-start" };
    });
    const animation = element.animate(frames, { duration: MOTION.layout, easing: "linear", fill: closing ? "forwards" : "none" });
    // (Folding, it's no row to light: see glides. Unmarked once this fold's done, unless another's begun meanwhile.)
    foldOf.set(element, animation);
    const still = () => foldOf.get(element) === animation && foldOf.delete(element);
    if (!closing) animation.finished.then(still, still);
    return animation;
  });
}
/**
 * A folder's name typed in place (on the page: its head, on its own page, else its shelf's; else the sidebar's row):
 * ↵ keeps it, esc doesn't. None in sight (a phone's sidebar, the rail): typed in ⌘K.
 */
function renameFolder(/** @type {string} */ id, inSide = false, first = false) {
  const folder = folderById(id);
  if (!folder) return;
  // (Asked from the rail, beside its row (one in a folder, its top folder's): its name's out of sight there.)
  if (inSide && onRail()) {
    let row = null;
    for (let up = /** @type {Folder | undefined} */ (folder); up && !row; up = up.parent ? folderById(up.parent) : undefined) row = /** @type {HTMLElement | null} */ (document.querySelector(`#chart-list [data-folder-item="${up.id}"]`));
    if (row) return nameBeside(row, id, first);
  }
  // (Asked from the sidebar's row (its ⋯): there, where it can be.)
  const side = !phone.matches && !railed(), own = home && "folder" in homeAt && homeAt.folder === id && !(inSide && side);
  const at = own ? ".home-head" : inSide && side ? `#chart-list [data-folder-row="${id}"]` : home && $("home-grid").querySelector(`[data-grid-folder="${id}"]`) ? `#home-grid [data-grid-folder="${id}"]` : side ? `#chart-list [data-folder-row="${id}"]` : "";
  const host = /** @type {HTMLElement | null} */ (at ? document.querySelector(at) : null);
  // (Nowhere else it shows, on the rail: beside its row, as asked from there.)
  if (!host && onRail() && !inSide) return renameFolder(id, true, first);
  if (!host) return openPage(renameFolderPage(id, first));
  // (Back to where it was asked from: the page's ⋯, the shelf's name, the sidebar's row.)
  const back = own ? "#home-title-more" : at.startsWith("#home-grid") ? `${at} .shelf-title` : `#chart-list [data-folder-item="${id}"]`;
  nameInPlace(host, { className: "folder-rename", value: folder.name, placeholder: "Folder", max: 60, label: "Folder name", apply: (name) => nameFolder(id, name, first), back: () => /** @type {HTMLElement | null} */ (document.querySelector(back))?.focus(), find: () => document.querySelector(at), cancel: () => first && unmakeFolder(id) });
}
/**
 * A folder named beside a row of the rail (its own, or its top folder's): a small box over the page, its mark as it'll
 * be as the name's typed (its first letter), ↵ keeping it, esc leaving it as it was; one just made (`first`), named.
 */
function nameBeside(/** @type {HTMLElement} */ anchor, /** @type {string} */ id, first = false) {
  const folder = folderById(id);
  if (!folder) return;
  const box = document.createElement("div"), stop = new AbortController();
  box.className = "name-beside";
  box.innerHTML = `<p class="menu-group">${first ? "New folder" : "Rename folder"}</p><div class="name-beside-field">${folderMark(folder)}</div><p class="name-beside-keys"><span><kbd>↵</kbd> ${first ? "Name" : "Rename"}</span><span><kbd>esc</kbd> Cancel</span></p>`;
  document.body.append(box);
  const field = /** @type {HTMLElement} */ (box.querySelector(".name-beside-field"));
  nameInPlace(field, { className: "name-beside-input", value: folder.name, placeholder: "Folder", max: 60, label: "Folder name", apply: (name) => nameFolder(id, name, first), back: () => anchor.isConnected && anchor.focus({ preventScroll: true }), find: () => (box.isConnected ? field : null), close: () => (fadeOut(box, [{ opacity: 1, transform: "none" }, { opacity: 0, transform: "scale(0.96)" }]), box.remove(), stop.abort()), cancel: () => first && unmakeFolder(id) });
  // (Its mark as it'll be: its first letter follows the name.)
  const input = /** @type {HTMLInputElement} */ (field.querySelector("input"));
  input.addEventListener("input", () => ((mark) => mark && (mark.outerHTML = folderMark({ ...(folderById(id) ?? folder), name: input.value.trim() || folder.name })))(field.querySelector(".folder-mark")));
  // (Beside the rail, as its rows' menus are, the field level with the row: again as the window or the list moves; its
  // row gone from the rail (widened), it's kept as it is.)
  const place = () => {
    if (!anchor.isConnected || !onRail()) return void input.blur();
    const strip = /** @type {HTMLElement} */ (document.querySelector(".sidebar .search")).getBoundingClientRect(), row = anchor.getBoundingClientRect();
    const top = clamp(row.top + row.height / 2 - (field.offsetTop + field.offsetHeight / 2), 8, innerHeight - box.offsetHeight - 8);
    Object.assign(box.style, { left: `${strip.right + 6}px`, top: `${Math.round(top)}px`, transformOrigin: `0 ${Math.round(row.top + row.height / 2 - top)}px` });
  };
  place();
  addEventListener("resize", place, { signal: stop.signal });
  $("chart-list").addEventListener("scroll", place, { passive: true, signal: stop.signal });
}
/** A folder just made, its naming cancelled (esc): unmade, as an undo of its making (redone, back), while that's the last step. */
const unmakeFolder = (/** @type {string} */ id) => libraryEdits.past.at(-1)?.folder === id && undoLibrary();
/** A folder's name, as ⌘K's page (read by its id when set: another tab may have changed it since); one just made (`first`), named. */
const renameFolderPage = (/** @type {string} */ id, first = false) => inputPage({ title: first ? "New folder" : "Rename", value: folderById(id)?.name ?? "", placeholder: "The folder's name", hint: "A folder needs a name", verb: first ? "Name" : "Rename", read: (text) => (text.trim() ? `${first ? "Name it" : "Rename to"} “${text.trim().slice(0, 60)}”` : null), apply: (text) => nameFolder(id, text.trim().slice(0, 60), first), cancel: first ? () => unmakeFolder(id) : undefined });
/** A folder's icon set (none: its first letter), by its id (as it is now), the Library's step. */
function setFolderIcon(/** @type {string} */ id, /** @type {string | undefined} */ icon) {
  const folder = folderById(id), was = folder?.icon;
  if (!folder || was === icon) return;
  libraryStep(`Changed “${folder.name}”'s icon`, () => ((now) => folderById(now)?.icon === icon && (setFolderIcon(now, was), () => folderById(folderNow(id))?.icon === was && setFolderIcon(folderNow(id), icon)))(folderNow(id)), "Icon put back");
  if (icon === undefined) delete folder.icon;
  else folder.icon = icon;
  save();
  renderCharts();
}
/** A folder named (by its id: as it is now), the Library's step. */
function nameFolder(/** @type {string} */ id, /** @type {string} */ name, first = false) {
  const folder = folderById(id), was = folder?.name;
  if (!folder || !name || name === was) return;
  // (A new folder's first name (`first`: the field New folder opened), while its making is the last step: its
  // making's, undone and made again with it; once. Undone, only if it's still the name it was given.)
  const making = libraryEdits.past.at(-1);
  if (first && !redoing && making?.folder && folderNow(making.folder) === id) delete making.folder;
  else libraryStep(`Renamed “${was}”`, () => ((now) => folderById(now)?.name === name && (nameFolder(now, /** @type {string} */ (was)), () => folderById(folderNow(id))?.name === was && nameFolder(folderNow(id), name)))(folderNow(id)), "Name put back");
  folder.name = name;
  save();
  renderCharts();
}
/** A folder a place up or down in the sidebar (past a chart, or another folder), gliding there. */
function stepFolder(/** @type {string} */ id, /** @type {number} */ by) {
  const folder = folderById(id);
  if (folder) stepSide(folder, by);
}
/** An item a place up or down among its folder's (the top's). @param {Chart | Folder} item */
function stepSide(item, by) {
  const items = childrenOf(parentOf(item)), at = items.indexOf(item), to = at + by;
  if (at < 0 || to < 0 || to >= items.length) return;
  [items[at], items[to]] = [items[to], items[at]];
  arrange(() => setChildren(items), sideKey(item));
}
/**
 * The sidebar rearranged (`edit`), gliding: its rows from where they were to where they are (`moved`, the one moved,
 * from `dropY`, where it was let go).
 */
function arrange(/** @type {() => void} */ edit, /** @type {string} */ moved, /** @type {number | null} */ dropY = null, /** @type {string[]} */ all = [moved]) {
  const rows = () => all$("[data-chart-row], [data-folder-row]", $("chart-list")), was = placesOf(rows(), sideId);
  const row = rows().find((item) => sideId(item) === moved);
  if (dropY !== null && row) was.set(moved, dropY - row.offsetHeight / 2);
  const before = arrangement();
  edit();
  // (Its order alone changed, the Library's step: one into or out of a folder has its own, see folderToast.)
  const after = arrangement();
  if (before.order.join() !== after.order.join() && [...before.folders].every(([key, folder]) => after.folders.get(key) === folder)) reorderStep(all, before, after);
  save();
  movingSide(renderCharts);
  slide(rows(), sideId, was, { moved });
}
/** An item reordered, the Library's step: undone, back after what was before it (others as they are now); redone, again. */
function reorderStep(/** @type {string[]} */ moved, /** @type {ReturnType<typeof arrangement>} */ before, /** @type {ReturnType<typeof arrangement>} */ after) {
  const item = itemOf(moved[0]);
  libraryStep(moved.length > 1 ? `Moved ${moved.length} items` : `Moved “${!item ? "it" : "layers" in item ? titleOf(item) : item.name}”`, () => {
    if (!itemsOf(keysNow(moved)).length) return;
    placeAs(before, itemsOf(keysNow(moved)));
    return () => void (placeAs(after, itemsOf(keysNow(moved))) && reorderStep(keysNow(moved), before, after));
  }, "Order put back");
}
/** `item` in `list`, just after the last of `before` still in it (none: first). @template T @param {T[]} list @param {T} item @param {(T | undefined)[]} before */
function placedAfter(list, item, before) {
  const after = before.findLast((other) => other !== undefined && list.includes(other));
  const at = after === undefined ? 0 : list.indexOf(after) + 1;
  return [...list.slice(0, at), item, ...list.slice(at)];
}
/** How things are arranged (the one order, each item's folder), to put back. */
const arrangement = () => ({ order: treeOf().map(({ item }) => sideKey(item)), folders: new Map([...library.charts, ...library.folders].map((item) => [sideKey(item), parentOf(item)])) });
/** Items whose folder changed since `was` say so (into one, or out of theirs), with an Undo back to it. @param {(Chart | Folder)[]} items */
function folderToast(was, items) {
  const moved = items.filter((item) => was.folders.get(sideKey(item)) !== parentOf(item));
  if (!moved.length) return;
  const what = moved.length === 1 ? `“${"layers" in moved[0] ? titleOf(moved[0]) : moved[0].name}”` : `${moved.length} ${moved.every((item) => "layers" in item) ? "charts" : "items"}`;
  const to = new Set(moved.map(parentOf)), from = new Set(moved.map((item) => was.folders.get(sideKey(item)))), into = [...to][0], left = [...from][0];
  const named = (/** @type {string | undefined} */ id) => `“${folderById(id)?.name ?? ""}”`;
  toast(to.size === 1 && into ? `Moved ${what} to ${named(into)}` : to.size === 1 && from.size === 1 && left ? `Moved ${what} out of ${named(left)}` : `Moved ${what}`, { label: "Undo", run: () => {
    const after = arrangement();
    placeAs(was, itemsOf(keysNow(moved.map(sideKey))));
    // (Again, as they were after it: each where the move put it, however it was moved, a drop's place too.)
    return () => void (placeAs(after, itemsOf(keysNow(moved.map(sideKey)))) && folderToast(arrangedNowOf(was), itemsOf(keysNow(moved.map(sideKey)))));
  }, undone: "Moved back" });
}
/** An arrangement with folders made again since by their new ids (see folderAgain). @param {ReturnType<typeof arrangement>} was */
const arrangedNowOf = (was) => ({ order: keysNow(was.order), folders: new Map([...was.folders].map(([key, folder]) => [keysNow([key])[0], folder && folderNow(folder)])) });
/**
 * Items back where an arrangement had them (by key: another tab's moves since, and its copies, stay as they are):
 * each in its folder then (still there, and not in itself), after the nearest item before it there then.
 * @param {ReturnType<typeof arrangement>} was @param {(Chart | Folder)[]} items @returns {boolean} whether it moved any
 */
function placeAs(was, items) {
  const tree = () => JSON.stringify(treeOf().map(({ item, depth }) => [sideKey(item), depth])), was0 = tree(), order0 = [...library.order];
  // (Folders made again since under new ids, by those.)
  const { order, folders } = arrangedNowOf(was);
  for (const old of items) {
    const key = sideKey(old), now = itemOf(key);
    if (!now) continue;
    const folder = folders.get(key);
    // (Its folder then gone since (another tab's, a delete's): where it is.)
    if (folder && !folderById(folder)) continue;
    const into = folder && ("layers" in now || !isIn(folder, now.id)) ? folder : undefined;
    setParent(now, into);
    const at = order.lastIndexOf(key), before = order.slice(0, Math.max(0, at)).map(itemOf);
    setChildren(placedAfter(childrenOf(into).filter((item) => item !== now), now, before));
  }
  // (Saved only if it moved anything (the sidebar's tree, not how its order's written): an undo that moved nothing did
  // nothing, see undoLibrary.)
  const moved = tree() !== was0;
  if (moved) save();
  else library.order = order0;
  renderCharts();
  return moved;
}
/**
 * Items (keys) put in a folder (`into`; none: the top), before `before` (a key there; none: at its end; `first`: at
 * its start), gliding there (from `dropY`, where let go); said, with an Undo (`quiet`: not). A folder never into
 * itself, nor into one in it; nor into one gone (a menu open as another tab deleted it): said so.
 */
function placeItems(/** @type {string[]} */ keys, /** @type {string | undefined} */ into, /** @type {string | null} */ before = null, { quiet = false, dropY = /** @type {number | null} */ (null) } = {}) {
  if (into && !folderById(into)) return void notice("That folder was deleted meanwhile");
  // (Several, in the order the tree has them, however they were picked.)
  const order = keys.length > 1 ? treeOf().map(({ item }) => item) : [];
  const items = itemsOf(keys).sort((a, b) => order.indexOf(a) - order.indexOf(b)).filter((item) => (("layers" in item && isYours(item)) || (!("layers" in item) && !(into && isIn(into, item.id)))));
  if (!items.length || (before && keys.includes(before))) return;
  const was = arrangement();
  arrange(() => {
    for (const item of items) setParent(item, into);
    const rest = childrenOf(into).filter((item) => !items.includes(item)), at = before ? rest.findIndex((item) => sideKey(item) === before) : rest.length;
    rest.splice(at < 0 ? rest.length : at, 0, ...items);
    setChildren(rest);
  }, sideKey(items[0]), dropY, items.map(sideKey));
  if (!quiet) folderToast(was, items);
}
/**
 * Charts and folders (keys) moved into a folder (after what's there; a folder never into itself, nor into one in it),
 * or out of theirs (`null`: each just under the folder it was in, a level up); said, with an Undo (`quiet`: not, a new
 * folder's own, named first).
 */
function moveToFolder(/** @type {string[]} */ keys, /** @type {string | null} */ id, quiet = false) {
  const moving = itemsOf(keys).filter((item) => !("layers" in item && !isYours(item)) && (id ? parentOf(item) !== id : parentOf(item)));
  if (!moving.length) return;
  // (Into a closed folder, their rows out of sight: a focus that was on one, or in its menu, goes to the folder's.)
  const hadFocus = Boolean(document.activeElement?.closest("#chart-list, #menu, .submenu"));
  if (id) placeItems(moving.map(sideKey), id, null, { quiet });
  else {
    // (Out: each folder's under it, in their order, one Undo for all.)
    // (By the folder each was in before any moved: one moved up into another's isn't moved again.)
    const was = arrangement(), from = new Map(moving.map((item) => [item, parentOf(item)]));
    for (const folder of new Set(from.values())) {
      const out = folderById(folder), siblings = out ? childrenOf(out.parent) : [], next = out ? siblings[siblings.indexOf(out) + 1] : undefined;
      placeItems(moving.filter((item) => from.get(item) === folder).map(sideKey), out?.parent, next ? sideKey(next) : null, { quiet: true });
    }
    if (!quiet) folderToast(was, moving);
  }
  // (The folder in sight that holds them: the outermost closed one they're in.)
  let shown = /** @type {string | undefined} */ (undefined);
  for (let at = id ?? undefined, steps = 0; at && steps <= library.folders.length; at = folderById(at)?.parent, steps++) if (folderById(at)?.closed) shown = at;
  if (hadFocus && shown) refocus(`#chart-list [data-folder-item="${CSS.escape(shown)}"]`);
}
/** Chart ids as keys. */
const chartKeys = (/** @type {string[]} */ ids) => ids.map((id) => `c:${id}`);
/** "Move out of" the folder items (keys) are in (theirs, for items in several), or "" for items in none. @param {string[]} keys */
function outOf(keys) {
  const from = new Set(itemsOf(keys).map(parentOf).filter(Boolean));
  return !from.size ? "" : from.size === 1 ? `Move out of “${folderById([...from][0])?.name ?? ""}”` : "Move out of their folders";
}
/**
 * Where items (keys) can go: each folder (theirs checked), a new one, and out of theirs.
 * @param {HTMLElement} anchor @param {string[]} keys
 */
function folderMenu(anchor, keys) {
  menuOf(anchor, folderChoices(keys), undefined, "", { title: phone.matches ? "Move to folder" : "" });
}
/**
 * Where items (keys) go, as choices: each folder as the sidebar has them, named with those it's in (theirs ticked;
 * a folder moved, not itself nor one in it), a new one (where they are, if they're all in one), out of theirs.
 * @param {string[]} keys
 * @returns {Action[]}
 */
function folderChoices(keys, out = true) {
  const items = itemsOf(keys), from = new Set(items.map(parentOf)), moved = items.filter((item) => !("layers" in item)).map((folder) => folder.id);
  const places = treeOf().flatMap(({ item }) => (!("layers" in item) && !moved.some((id) => isIn(item.id, id)) ? [item] : [])), counts = chartCounts();
  return [
    ...places.map((folder) => ({ key: folder.id, icon: folderMark(folder), label: folderPath(folder.id), aside: plural(counts.get(folder.id) ?? 0, "chart"), current: from.size === 1 && from.has(folder.id), run: () => moveToFolder(keys, folder.id) })),
    { key: "new", icon: ICONS.folderAdd, label: "New folder", apart: true, run: () => (isOpen($("palette")) && dismissPalette(), newFolder(keys, from.size === 1 ? [...from][0] : undefined)) },
    ...(out && outOf(keys) ? [{ key: "out", icon: ICONS.folderOut, label: outOf(keys), run: () => moveToFolder(keys, null) }] : []),
  ];
}
/** A folder's menu (its ⋯, a right-click on it). */
const folderActions = (/** @type {HTMLElement} */ anchor, /** @type {string} */ id, /** @type {{ x: number, y: number } | undefined} */ point = undefined) => menuOf(anchor, folderRows(id, false, Boolean(anchor.closest("#chart-list"))), point ?? null);
/**
 * A folder's actions (its menu; ⌘K's page of it, before its charts: renamed there, moved with ⌥↑ and ⌥↓).
 * @returns {Action[]}
 */
function folderRows(/** @type {string} */ id, inPalette = false, inSide = false) {
  const folder = folderById(id);
  if (!folder) return [];
  const peers = childrenOf(folder.parent), at = peers.indexOf(folder), leave = outOf([`f:${id}`]);
  const view = { key: "view", icon: ICONS.arrowRight, label: "View", aside: "Its page", run: () => goTo({ folder: id }) };
  const toggle = { key: "toggle", icon: folder.closed ? ICONS.folderOpen : ICONS.folder, label: folder.closed ? "Expand" : "Collapse", ...(inPalette ? { aside: folder.closed ? "What's in it shown" : "What's in it out of sight" } : {}), verb: folder.closed ? "Expand" : "Collapse", stay: true, run: () => toggleFolder(id) };
  const rename = { key: "rename", icon: ICONS.pencil, label: "Rename", ...(inPalette ? { aside: folder.name, page: () => renameFolderPage(id) } : { run: () => renameFolder(id, inSide) }) };
  // (Its mark: its first letter, or an icon.)
  const now = () => folderById(id) ?? folder;
  const icon = { key: "icon", icon: folderMark(folder), label: "Icon", aside: folder.icon ? capitalized(folder.icon) : "Letter", page: () => choicePage("Icon", () => [
    { key: "letter", icon: folderMark({ ...now(), icon: undefined }), label: "Letter", aside: letterOf(now().name), current: !now().icon, run: () => setFolderIcon(id, undefined) },
    ...FOLDER_ICONS.map((name) => ({ key: name, icon: folderMark({ ...now(), icon: name }), label: capitalized(name), current: now().icon === name, run: () => setFolderIcon(id, name) })),
  ]) };
  return [
    // (In ⌘K, View first, as its pages lead with their own (see viewRow); in its menu, Rename first, as a chart's is.)
    ...(inPalette ? [view, toggle, rename] : [view, rename, toggle]),
    icon,
    ...(inPalette && at > 0 ? [{ key: "up", icon: ICONS.arrowUp, label: "Move up", kbd: "⌥↑", verb: "Move", stay: true, run: () => stepFolder(id, -1) }] : []),
    ...(inPalette && at < peers.length - 1 ? [{ key: "down", icon: ICONS.arrowDown, label: "Move down", kbd: "⌥↓", verb: "Move", stay: true, run: () => stepFolder(id, 1) }] : []),
    { key: "new-chart", icon: ICONS.plus, label: "New chart inside", run: () => (isOpen($("palette")) && dismissPalette(), newChart(id)) },
    { key: "new-folder", icon: ICONS.folderAdd, label: "New folder inside", run: () => (isOpen($("palette")) && dismissPalette(), newFolder([], id, { side: inSide })) },
    { key: "folder", icon: ICONS.folder, label: "Move to folder", ...(inPalette ? { aside: folder.parent ? folderPath(folder.parent) : "None" } : {}), page: () => choicePage("Move to folder", () => folderChoices([`f:${id}`], inPalette)) },
    ...(leave ? [{ key: "out", icon: ICONS.folderOut, label: leave, run: () => moveToFolder([`f:${id}`], null) }] : []),
    // (Its charts in and out, as your Library's: a file imported into it, all it holds downloaded with it.)
    { key: "import", icon: ICONS.upload, label: "Import charts", aside: "Into it", run: () => (isOpen($("palette")) && dismissPalette(), importCharts(id)) },
    ...(library.charts.some((doc) => isIn(doc.folder, id)) || library.folders.some((other) => other.id !== id && isIn(other.id, id)) ? [{ key: "download", icon: ICONS.download, label: "Download all charts", aside: "Its folders too", run: () => downloadFolder(id) }] : []),
    { key: "delete", icon: ICONS.trash, label: "Delete the folder", aside: "Keeps its charts", verb: "Delete", danger: true, run: () => deleteFolder(id) },
  ];
}
/** A chart's menu, from its sidebar row (its ⋯, a right-click). */
const chartMenu = (/** @type {HTMLElement} */ anchor, /** @type {string} */ id, /** @type {{ x: number, y: number } | null} */ point = null) => menuOf(anchor, chartActions(id, "row"), point);
function chartPick(id, pick) {
  const doc = chartById(id);
  if (!doc) return renderCharts();
  if (pick === "rename" && home && cardOf(id)) {
    if (!isPreview(doc)) (picked.clear(), pickChart(id));
    renameTile(doc);
  } else if (pick === "rename") {
    const row = !phone.matches && !railed() ? /** @type {HTMLElement | null} */ ($("chart-list").querySelector(`[data-chart-row="${id}"], [data-preview-row="${id}"]`)) : null;
    if (row) renameRow(doc, row);
    else openPage(renameChartPage(doc));
  } else if (pick === "duplicate") {
    if (!home) return void duplicateCharts([doc]);
    picked.clear();
    picked.add(id);
    batch("duplicate");
  }
  else if (pick === "download") downloadChart(doc);
  else if (pick === "delete") deleteChart(doc);
}
/**
 * Charts copied, each right under its original (nothing opens), with an Undo. `again`: the copies an Undo let go of,
 * listed again in their place (a Redo: the same charts, so their own steps and every Library step that knew them find
 * them).
 * @param {Chart[]} docs @param {Chart[] | null} [again]
 */
function duplicateCharts(docs, again = null) {
  const copies = again ?? docs.map(duplicateChart);
  again?.forEach((copy, i) => relist(copy, library.charts.indexOf(docs[i]) + 1));
  save();
  renderCharts();
  const same = asThen(copies);
  toast(copies.length === 1 ? `Duplicated “${titleOf(docs[0])}”` : `Duplicated ${copies.length} charts`, { label: "Undo", run: () => {
    if (!same()) return keptAsEdited(copies);
    const ids = copies.map((copy) => copy.id), open = ids.indexOf(chart.id);
    if (!library.charts.some((doc) => ids.includes(doc.id))) return;
    // (The copies gone first: the one opened in place of an open copy is one still listed, else a new one.)
    for (const doc of library.charts.filter((doc) => ids.includes(doc.id))) discard(doc);
    if (open >= 0) openChart(chartById(docs[open].id) ?? previousChart() ?? library.charts[0] ?? newDraft());
    save();
    renderCharts();
    // (Redone: each copy back under its original, those still there.)
    return () => {
      const left = docs.flatMap((doc, i) => ((now) => (now && !inLibrary(copies[i]) ? [{ doc: now, copy: copies[i] }] : []))(library.charts.find((other) => other.id === chartNow(doc.id))));
      if (left.length) duplicateCharts(left.map(({ doc }) => doc), left.map(({ copy }) => copy));
    };
  } });
  return copies;
}
/**
 * A copy of a chart. New charts start the list; a copy is a variant of its original and sits right under it (as
 * layers do), drawn as it until it's drawn itself. @param {Chart} doc
 */
function duplicateChart(doc) {
  const base = `${titleOf(doc).replace(/ copy(?: \d+)?$/, "")} copy`, titles = new Set(library.charts.map(titleOf));
  let title = base;
  for (let n = 2; titles.has(title); n++) title = `${base} ${n}`;
  // (Of one untitled, untitled too: named by what it holds, as it is.)
  const copy = { ...structuredClone(doc), id: uid(), ...(doc.title ? { title } : {}), updated: Date.now() };
  library.charts.splice(library.charts.indexOf(doc) + 1, 0, copy);
  // Where it's looked at, too.
  if (doc === chart) saveView();
  keepCarriedView(copy, { view: leftView(doc.id) });
  // (Its mark in the list doesn't depend on its id.)
  const drawn = previews.get(doc.id), mark = thumbs.get(doc.id);
  if (drawn) previews.set(copy.id, { ...drawn, key: "" });
  if (mark) {
    thumbs.set(copy.id, mark);
    saveThumbs();
  }
  return copy;
}
/**
 * A blank chart, where to start from (series, a preset, a file): yours at once, first in your charts (on a folder's
 * page, in it); left as it came, it goes (see emptyYours). One just made, still as it came, open: that one.
 */
/** A new chart, in `into` (a folder's ⋯), else in the folder whose page it's made from. @param {string} [into] */
function newChart(into) {
  delete document.body.dataset.sheet;
  const folder = into ? folderById(into) : home && "folder" in homeAt ? folderById(homeAt.folder) : undefined;
  if (emptyYours(chart)) {
    // (The empty one at hand taken, where it's asked for.)
    if (into && chart.folder !== into) (chart.folder = into), save(), renderCharts();
    leaveHome(() => {});
  } else {
    const doc = /** @type {Chart} */ ({ ...blankChart(), blank: true });
    if (folder) doc.folder = folder.id;
    library.charts.unshift(doc);
    save();
    leaveHome(() => openChart(doc));
  }
  void until(() => !document.body.hasAttribute("data-home"), 1000).then(() => /** @type {HTMLElement | null} */ ($("empty").querySelector(".search-cta"))?.focus());
}

/** Where elements are on screen now (mid-glide, where they show), by key. @param {HTMLElement[]} elements @param {(element: HTMLElement) => string | undefined} key */
const placesOf = (elements, key) => new Map(elements.map((element) => [key(element), element.getBoundingClientRect().top]));
/** The first color behind an element: the ground one passing over others is painted on. @param {HTMLElement} element */
function groundOf(element) {
  for (let node = element.parentElement; node; node = node.parentElement) {
    const color = getComputedStyle(node).backgroundColor;
    if (color !== "transparent" && color !== "rgba(0, 0, 0, 0)") return color;
  }
  return css("--app");
}
/** Each row lifted over its neighbors by a glide, by that glide (see slide). @type {WeakMap<Element, object>} */
const lifts = new WeakMap();
/**
 * Elements glide from where they were (`was`, by key) to where they are now (FLIP). `moved` passes over the others,
 * on its ground, so their words never cross; with `fade`, one that wasn't there fades in. `pseudo` moves an element's
 * ::before instead (the line between panes); `layout`, at a layout's pace (with a box resizing: the sidebar's
 * width), each moving passing over the others on its ground (its list's gaps too), one come fading in under them as
 * the words come: their words never cross. Nothing moves under reduced motion.
 * @param {HTMLElement[]} elements @param {(element: HTMLElement) => string | undefined} key @param {Map<string | undefined, number>} was
 * @param {{ moved?: string | null, fade?: boolean, pseudo?: string, layout?: boolean }} [options]
 */
function slide(elements, key, was, { moved = null, fade = false, pseudo, layout = false } = {}) {
  if (reduced()) return;
  const timing = { duration: layout ? MOTION.layout : MOTION.panel, easing: layout ? MOTION.easeLayout : MOTION.ease, ...(pseudo ? { pseudoElement: pseudo } : {}) };
  const come = elements.filter((element) => was.get(key(element)) === undefined), going = elements.filter((element) => was.get(key(element)) !== undefined);
  // (A glide still under way goes first: where it's laid out now is measured, not where it's being carried. All
  // cancelled, then all read, then all moved: a read between writes would lay a long list out again for each row.)
  for (const element of going) for (const animation of element.getAnimations()) if (/** @type {KeyframeEffect | null} */ (animation.effect)?.getKeyframes().some((frame) => "transform" in frame)) animation.cancel();
  const tops = going.map((element) => element.getBoundingClientRect().top);
  const lifted = (/** @type {HTMLElement} */ element) => layout || (moved !== null && key(element) === moved);
  // (Their ground, read once with the rest: they're one list's.)
  const first = going.find(lifted), ground = first ? groundOf(first) : "";
  // (At a layout's pace (the sidebar widening), with its words: once there's room for them, see data-sliding.)
  if (fade) for (const element of come) element.animate({ opacity: [0, 1] }, layout ? { delay: MOTION.wordsWait, duration: MOTION.wordsIn, easing: MOTION.ease, fill: "backwards" } : timing);
  for (const [i, element] of going.entries()) {
    const by = /** @type {number} */ (was.get(key(element))) - tops[i];
    if (Math.abs(by) < 0.5) continue;
    const lift = lifted(element), glide = {};
    // (Its ground over half its list's gap above and below too: what fades in under it never shows between them.)
    if (lift) Object.assign(element.style, { zIndex: "1", background: ground, boxShadow: `0 calc(var(--row-gap, 0px) / -2) 0 ${ground}, 0 calc(var(--row-gap, 0px) / 2) 0 ${ground}` });
    if (lift) lifts.set(element, glide);
    // (A glide cancelled by a newer one lands after it began: only the latest lift is let go, by its own end.)
    const land = () => lifts.get(element) === glide && (lifts.delete(element), Object.assign(element.style, { zIndex: "", background: "", boxShadow: "" }));
    element.animate({ transform: [`translateY(${by}px)`, "none"] }, timing).finished.then(land, land);
  }
}

/** Where a drag in a list (along y) lands: by the row the pointer's over, or the nearest (never the end). @param {HTMLElement[]} rows */
function landingIn(rows, /** @type {number} */ y, /** @type {(row: HTMLElement) => string} */ idOf, /** @type {string[]} */ ids) {
  // (Those carried aren't places: by the nearest of the others.)
  const others = rows.filter((row) => !ids.includes(idOf(row))), row = others.find((each) => y < each.getBoundingClientRect().bottom) ?? others.at(-1);
  if (!row) return null;
  const box = row.getBoundingClientRect();
  return { item: row, after: y > box.top + box.height / 2 };
}

/** A chart a place up or down among its folder's (past a folder too), gliding there. */
function stepChart(/** @type {string} */ id, /** @type {number} */ by) {
  const doc = chartById(id);
  if (doc && !isPreview(doc)) stepSide(doc, by);
}
/**
 * Where something lands beside an item (its key), before it or after it: in its folder, before what's there (none:
 * at its end), the carried (`keys`) aside. @returns {{ into: string | undefined, before: string | null }}
 */
function besideItem(/** @type {string} */ key, /** @type {boolean} */ after, /** @type {string[]} */ keys) {
  const item = itemOf(key), into = item ? parentOf(item) : undefined;
  if (!after) return { into, before: key };
  const rest = childrenOf(into).filter((other) => !keys.includes(sideKey(other))), next = rest[rest.findIndex((other) => sideKey(other) === key) + 1];
  return { into, before: next ? sideKey(next) : null };
}
const chartLabel = (/** @type {string} */ id) => ((doc) => (doc ? `${chartMark(doc)}<span class="name">${escapeHtml(titleOf(doc))}</span>` : ""))(chartById(id));
// The sidebar, as a tree: a chart or a folder onto a folder's row goes into it (its top edge: above it; its foot: into
// it, first, or under it, closed); beside a row, it takes its place, in its folder. A folder never into itself, nor
// into one in it: their rows aren't places while it's carried, its own are carried along.
const sideId = (/** @type {HTMLElement} */ row) => (row.dataset.chartRow ? `c:${row.dataset.chartRow}` : `f:${row.dataset.folderRow}`);
/** A sidebar row's level in the tree. */
const depthOf = (/** @type {HTMLElement} */ row) => Number(row.style.getPropertyValue("--depth")) || 0;
/** The rows a drag can land by: those in sight, but the carried and what's in them. @param {string[]} ids */
function placesFor(ids) {
  const carried = ids.filter((id) => id.startsWith("f:")).map((id) => id.slice(2));
  return all$("[data-chart-row]:not([inert]), [data-folder-row]:not([inert])", $("chart-list")).filter((row) => {
    const key = sideId(row), item = itemOf(key);
    return !ids.includes(key) && item !== undefined && !carried.some((id) => isIn(parentOf(item), id));
  });
}
// The grid's tiles move the charts in the list (the sidebar's order, and ⌘1–9's): those picked together. Found by the
// filter, they're ranked, not in their order: there's no place to drop them into (a hold still picks).
/** Whether the grid's charts can be moved in their order (dragged, ⌥-arrows): not searched, nor the previews' or Recent's (theirs, the order they were opened). */
const reorderable = () => !$("home-filter").value.trim() && ("folder" in homeAt);
/** A row by what it is (its data): the same row once it's drawn again. */
const rowKeyOf = (/** @type {Element | null} */ row) => (row instanceof HTMLElement ? Object.entries(row.dataset).map(([key, value]) => `${key}:${value}`).join(" ") || row.className : "");
function roveList() {
  const list = $("chart-list"), rows = all$(".chart-row:not([inert]), .folder-row:not([inert]), .list-head", list);
  for (const button of all$("button", list)) button.tabIndex = -1;
  const row = rows.find((each) => listStop && rowKeyOf(each) === listStop) ?? list.querySelector(".chart-row.active") ?? rows[0];
  const stop = /** @type {HTMLElement | null | undefined} */ (row?.querySelector(".nav-item, .list-head-name"));
  if (stop) stop.tabIndex = 0;
}
/**
 * The sidebar's list, one stop for Tab (not two a row: a long Library, a hundred): the row last on (by its key),
 * else the open chart's, else the first; ↑ ↓ and ← → walk it from there. A row's ⋯ (a head's buttons too) by its
 * menu key or ⇧F10, its press, a right-click.
 */
let listStop = "", keyMenuAt = 0;
function startListKeys() {
  $("chart-list").addEventListener("focusin", (event) => {
    const target = /** @type {HTMLElement} */ (event.target);
    if (!target.matches(".nav-item, .list-head-name")) return;
    for (const button of all$('button[tabindex="0"]', $("chart-list"))) if (button !== target) button.tabIndex = -1;
    target.tabIndex = 0;
    listStop = rowKeyOf(target.closest(".chart-row, .folder-row, .list-head"));
  });
  // In the list, ↑ ↓ (Home, End) walk its rows rather than the layers; ⌥↑ ⌥↓ move the focused one (yours); ← → as a
  // tree: → opens a folder (a group's head), then goes into it; ← closes it, then goes to the one it's in, as from a
  // chart. (The heads a level above the rows.)
  $("chart-list").addEventListener("keydown", (event) => {
    if (composing(event) || typing(event.target)) return;
    const rows = all$(".chart-row:not([inert]), .folder-row:not([inert]), .list-head", $("chart-list")), row = /** @type {HTMLElement | null} */ (closest(event, ".chart-row, .folder-row, .list-head"));
    const first = (/** @type {HTMLElement | undefined} */ each) => /** @type {HTMLElement | null | undefined} */ (each?.querySelector(".nav-item, .list-head-name"));
    const level = (/** @type {HTMLElement} */ each) => (each.classList.contains("list-head") ? -1 : depthOf(each));
    if (!row) return;
    // (Its menu, as a right-click on it is: the menu key, ⇧F10; the focus back on the row as it closes. The browser's
    // own right-click from the same key, after it, not a second one.)
    if (event.key === "ContextMenu" || (event.shiftKey && event.key === "F10")) {
      const button = /** @type {HTMLElement | null} */ (row.querySelector(".nav-item, .list-head-name"));
      if (!button) return;
      event.preventDefault();
      const box = button.getBoundingClientRect();
      keyMenuAt = performance.now();
      button.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, cancelable: true, clientX: box.left + 16, clientY: box.bottom }));
      if (!$("menu").hidden) setMenuOwner(button);
      return;
    }
    if (!event.altKey && !event.metaKey && !event.ctrlKey && ["ArrowUp", "ArrowDown", "Home", "End"].includes(event.key)) {
      const at = rows.indexOf(row);
      if (at < 0) return;
      event.preventDefault();
      event.stopPropagation();
      return void first(rows[stepIn(at, Number(KEY_STEPS[event.key]), rows.length)])?.focus();
    }
    const folder = row.dataset.folderRow, section = row.dataset.sectionRow, group = /** @type {Group | undefined} */ (row.dataset.group), id = row.dataset.chartRow;
    if (!event.altKey && !event.metaKey && !event.ctrlKey && !event.shiftKey && (event.key === "ArrowLeft" || event.key === "ArrowRight")) {
      const opens = folder !== undefined || section !== undefined || group !== undefined, open = opens && !row.classList.contains("closed"), depth = level(row), at = rows.indexOf(row);
      event.preventDefault();
      // (On the rail, a folder's or a section's → is its menu, the focus in it, as a press shows it: they never open
      // there.)
      if (onRail() && (folder !== undefined || section !== undefined)) return void (event.key === "ArrowRight" && first(row) && railMenuKept(/** @type {HTMLElement} */ (first(row))));
      if (opens && open === (event.key === "ArrowLeft")) {
        if (folder) toggleFolder(folder);
        else if (group) toggleGroup(group);
        else toggleSection(/** @type {string} */ (section));
        // (Drawn anew: its row's own again.)
        const again = all$(".folder-row, .list-head", $("chart-list")).find((each) => (folder ? each.dataset.folderRow === folder : group ? each.dataset.group === group : each.dataset.sectionRow === section));
        return void first(again)?.focus();
      }
      const toward = event.key === "ArrowRight" ? (open && rows[at + 1] && level(rows[at + 1]) > depth ? rows[at + 1] : undefined) : rows.slice(0, at).findLast((each) => level(each) < depth);
      return void first(toward)?.focus();
    }
    if (!event.altKey || (event.key !== "ArrowUp" && event.key !== "ArrowDown")) return;
    if (folder) {
      event.preventDefault();
      stepFolder(folder, event.key === "ArrowUp" ? -1 : 1);
      return void (/** @type {HTMLElement | null} */ ($("chart-list").querySelector(`[data-folder-item="${folder}"]`))?.focus());
    }
    if (!id) return;
    event.preventDefault();
    stepChart(id, event.key === "ArrowUp" ? -1 : 1);
    /** @type {HTMLElement | null} */ ($("chart-list").querySelector(`[data-open-chart="${id}"]`))?.focus();
  });
}
/** The recommended charts as the charts they make, their drawings kept under stable ids. @type {{ preset: number, doc: Chart }[]} */
let recommendedCards = [];
function renderRecommended() {
  const list = presets;
  // (Each as it shows: as edited here, else as made.)
  recommendedCards = !list ? [] : recommendedAt().flatMap((preset, k) => {
    if (preset < 0) return [];
    const edited = editedPreset(preset);
    if (edited && presetEdited(edited)) return [{ preset, doc: edited }];
    const doc = Object.assign(fromPreset(list[preset]), { id: `recommended:${k}` });
    return [{ preset, doc }];
  });
  recommendedDrawn = recommendedKey();
  // (The focus in them, back on its like once they're written anew: a theme's change, more contrast asked for.)
  const focus = focusKey($("templates"));
  // (Each a preset's tile, as the gallery's; not read, said, with a way to try again, as the presets' page says it.)
  $("templates").innerHTML = list ? recommendedCards.map(({ preset, doc }, i) => presetTileHtml(preset, doc, i)).join("")
    : presetsFailed ? `<p class="gallery-note">The presets couldn't be read. <button type="button" class="btn ghost small" data-gallery-retry>Try again</button></p>`
    : RECOMMENDED.map(() => WAITING_TILE).join("");
  // (Those drawn before, gone now, let go of first.)
  cardsSeen.disconnect();
  for (const card of all$("[data-template]")) cardsSeen.observe(card);
  restoreFocus($("templates"), focus);
  fitStrip($("templates"));
}
/** The recommended cards as last drawn: by the theme and the presets edited among them (one edited since, drawn anew, see renderCharts). */
let recommendedDrawn = "";
const recommendedKey = () => (presets ? JSON.stringify([light, ...recommendedAt().map((i) => ((doc) => (doc && presetEdited(doc) ? `${doc.id}@${doc.updated}` : ""))(i >= 0 ? editedPreset(i) : undefined))]) : "");
// A card's drawing, once it comes into view: one at a time, each asked for at the page's next idle moment (however
// long that takes). Kept for the day.
const cardQueue = new Set();
/**
 * The pictures that couldn't be fetched (see paint), asked for again, each as it's drawn (a tile's, the gallery's, a
 * recommended card's): as the server answers (see checkLive), they're waited for again.
 */
function retryPlots() {
  const failed = all$("[data-plot].failed");
  if (!failed.length) return;
  for (const frame of failed) frame.classList.replace("failed", "pending");
  // (The recommended cards, wherever they are: on the empty chart too.)
  for (const frame of failed) ((card) => card && cardQueue.add(Number(card.dataset.template)))(/** @type {HTMLElement | null} */ (frame.closest("[data-template]")));
  drawCards();
  if (!home) return;
  queueYours();
  for (const frame of gallerySeen) if (frame.classList.contains("pending")) galleryQueue.add(presetOf(frame));
  void drawGallery();
}
export {
  aheadOf, allShelfSteps, areaUnder, arrive, asPlace, backToHome, batch, beginOnTime, besideItem, bezierOf, bubblesAt, cardOf,
  cardQueue, CARDS, chartCounts, chartKeys, chartLabel, chartMark, chartMenu, chartPathRows, chartPick, childrenOf,
  clearPicks, closeDialogs, depthOf, derivedTitle, dialogOpeners, dialogScrims, dragSheet, drawPreview, drawThumb, fadeDef,
  duplicateCharts, fillRow, fillSlot, folderActions, folderById, folderChoices, folderPath, folderRow, folderRows,
  forgetPaletteMarks, freshThumb, GALLERY_PAGE, galleryFound, galleryQueue, gallerySeen, galleryWait, giveUpUnframed, go, gridActions,
  groupRows, groupShut, heatSamples, heightsSheet, homeName, isIn, isOpen, itemOf, itemsOf, keepNaming,
  KEPT_PRESET_MARKS, keptMarkKey, keyMenuAt, keysCame, landingIn, leafRow, leave, leftPlaces, legendEntry, lineOver, markOf, morph,
  morphing, moveToFolder, movingSide, naming, newChart, newFolder, nextCard, nextGallery, NO_MARK, numbered, onceKeys,
  onChart, onClosed, onScaleAll, orphan, outOf, pageIn, pageOut, paint, paletteMarksAsked, panelSize, parentOf,
  pickChart, picked, placeItems, placeNames, placesFor, placesOf, PRESET_MARKS, presetMark, presetMarkKey,
  presetMarkQueue, presetMarksAsked, presetName, presetNodeAt, presetOf, presetParts, presetRankOf, presetSearch,
  presetTiles, presetTree, previewBars, previewIndex, previewKey, previewOf, PREVIEWS, previewSvg, profileMark,
  queueYours, recommendedCards, recommendedDrawn, recommendedKey, recommendedPresets, renderChartPath, renderDoors,
  renderGallery, renderHomeHead, renderHorizon, renderPicked, renderRecommended, renderSideNav, renderTheme,
  renderYours, reorderable, restOf, retryFailed, retryPlots, roveList, sampleable, sampleIndex, savePreviews,
  saveThumbs, searchNow, sectionKeys, sectionRows, seePaletteMarks, serial, seriesMark, setChildren, setParent,
  setTheme, sheetGoes, sheetTaken, shelfSteps, shortCount, showDialog, shownCharts, showPick, SIDE_PRESETS, sideHead,
  sideId, sideKey, sideMoved, sideTree, sizeSheet, slide, slideAway, slides, speedOf, startGalleryRest, startListKeys,
  stepChart, stepShelf, stillness, stretched, themeMenu, thumbKey, THUMBS, tileOpen, titleOf, toggleFolder, toggleGroup,
  toggleSection, treeOf, unstretched, updateMark, validChartMark, validDrawingKey, validPreview, validThumb,
  walkCards, writeTile, yourQueue, yoursFound, yoursSeen, yoursShown,
};
