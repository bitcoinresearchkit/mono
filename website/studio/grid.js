// @ts-check
// The page's grid: tiles, mini charts, the presets' gallery, picking charts and moving them.
import { initialsOf, initialsWord } from "/studio/initials.js";
import { QuickMatch, QuickMatchConfig } from "/modules/quickmatch-js/src/index.js";
import { columnWidth, indexed, PARTIAL as PARTIAL_SHARE, press } from "/modules/bitview-charts/index.js";
import { capitalized } from "/studio/words.js";
import {
  acrossUnfit, applyTheme, asThen, cardsSeen, chart, chartById, chartNow, css, deleteChart, deleteFolder, dialogsShown,
  discard, dismissPalette, drawCards, drawGallery, drawPresetMarks, drawYours, emptyYours, engine, FOLDER_ICONS,
  folderAgain, folderMark, folderNow, fromUnfit, galleryAhead, galleryPlace, galleryShown, galleryTilesAhead,
  galleryTilesSeen, goTo, GROUP_OF, GROUPS, HEAT_PICTURE_AGG, HEAT_PICTURE_GRID, HEAT_PICTURE_ROWS, HEAT_TILE_COLUMNS,
  heatKnown, heldUpright, home, homeAt, homeScroll, importCharts, inLibrary, isPreview, isYours, keepSidePresets,
  keepsLeaving, keptAsEdited, keysNow, layoutCurve, layoutOwed, leaveHome, leftView, leftWith, letterOf, library,
  libraryEdits, libraryStep, light, listedPreviews, listOwed, livePrice, loadPresets, minedHalvings, mostBars, MOTION,
  newDraft, notice, onRail, openChart, openView, paletteMarksSeen, phone, pickPreset, placeGallery, prefs, presetLabels,
  presetMarks, presets, presetsFailed, presetsMatching, pressedTile, previewCharts, previews, previousChart,
  profileColor, railed, railMenuKept, recentCharts, redoing, reduced, relayout, relist, renderCharts, save, savePrefs,
  savePreviewsSoon, saveView, sectionMark, shelfRows, shelfScrolls, sideLight, sidePresets, started, storage, THEMES,
  thumbs, toast, touched, undoLibrary, viewOn, within, writeSideList, yourTilesSeen, zeroFit, THUMBS, PREVIEWS, SHELF,
  GALLERY_PAGE, MARK_VERSION, THUMB_POINTS,
} from "/studio/app.js";
import {
  $, all$, closest, composing, escapeHtml, holdPage, KEY_STEPS, scrollsY, SLOP, stepIn, TOUCH, typing, focusKey, restoreFocus, isOpen, cutFade, fadeOut, fading,
} from "/studio/dom.js";
import { inSlices, until, whenIdle } from "/studio/timing.js";
import { clamp, hashOf, lowerBound, uid, onScale, onScaleAll } from "/studio/math.js";
import { format, grouped, plural, precise } from "/studio/format.js";
import { ICONS } from "/studio/icons.js";
import {
  belowOf, blankChart, BUBBLE_HIDE, BUBBLE_SIZE, cloudLayers, DASHES, dashOf, edgesOf, heatGradient, heatLike, LINED,
  measureOf, opacityOf, OUTLINED, PAIRS, placedOn, summed, unfitted,
} from "/studio/vocabulary.js";
import { FINE, INTERVAL_DAYS, INTERVAL_LABEL, INTERVAL_ORDER, rangeDays } from "/studio/intervals.js";
import { stopCarrying, swallowClick } from "/studio/gestures.js";
import {
  asked, baseOf, byColor, colorChain, cutsOf, defaultSize, drawSeries, edges, fitStrip, foldOf, hiddenPanes, forget,
  gapped, info, ink, intervalsOf, load, loads, MIN_BAR_SPACING, paintOf, periodAmount, presetWindow, profiled, resolve,
  retryInfo, rightGap, seriesIn, SIDES, unitFor,
} from "/studio/chart.js";
import {
  chartActions, choicePage, inputPage, openPage, renameChart, renameChartPage, searchCase, searchText, typedWords,
} from "/studio/command.js";
import { FUNCTIONS, parse } from "/studio/formulas.js";
import {
  amountsOf, heatCalendar, heatColor, heatCome, heatDayHeld, heatDayKey, heatDays, heatExtras, heatGrid, heatLook,
  heatMost, heatPump, mostIn, rowsOf,
} from "/studio/heatmaps.js";
import {
  anythingKept, chartsFile, closedOwner, download, downloadAll, downloadChart, downloadFolder, hideMenu, menuOf, ownerNow, setMenuOwner, supportedIntervals,
} from "/studio/menus.js";
import {
  RECOMMENDED, refocus, vocabNow,
} from "/studio/presets-edit.js";
import { fillOf, labelOf, layerById, lineLook, modeOf, STACK_FILL, stackOf, derivedTitle, titleOf, seriesMark, barsOf, runs, sides, areaOf, legendEntry, fadingOf, levelSteps, rangeOf } from "/studio/looks.js";
import { presetTree, presetRoot, presetName, presetNodeAt, placeNames, homeName, folderById, asPlace, editedPreset, fromPreset, keptPreset, presetById, presetDoc, presetDocs, presetEdited, presetIdOf, presetIndex, presetKey, pristine, showAddress, chartPlace, stepsTo } from "/studio/preset-charts.js";
import { tickOn } from "/studio/sketch.js";
import { keepCarriedView } from "/studio/charts-read.js";
/** @import { Group, Preview, Thumb } from "/studio/app.js" */
/** @import { Chart, Folder, Layer } from "/studio/vocabulary.js" */
/** @import { Resolved } from "/studio/chart.js" */
/** @import { Action } from "/studio/command.js" */
/** @import { Node, Vector } from "/studio/formulas.js" */
/** @import { heatWants } from "/studio/heatmaps.js" */
/** @import { Preset } from "/studio/presets-edit.js" */
/** @import { Place, PresetNode, Step } from "/studio/preset-charts.js" */

/** @param {unknown[]} parts */
const drawingKey = (parts) => MARK_VERSION + JSON.stringify(parts);

/** Whether a formula runs a window: on a coarser interval a short one would be one point. */
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
/** @param {Chart} doc */
const thumbKey = (doc) => drawingKey([new Date().toISOString().slice(0, 10), doc.index, doc.range, doc.panes.map((pane) => [pane.mode, pane.size, pane.stack, pane.reverse, pane.average, pane.outline, pane.gradient]), doc.layers.map((layer) => [layer.id, layer.series, layer.formula, layer.hidden, layer.color, layer.side, doc.panes.findIndex((pane) => pane.id === layer.pane), layer.letter, layer.type, layer.base, layer.colorBelow, layer.colors, layer.fill, layer.from, layer.ref, layer.colorFrom, layer.fit, layer.byLevels ? layer.levels : null, layer.byGradient ? [layer.gradient, layer.upToDate] : null, heatLike(layer) ? [layer.heatmap, layer.profile, layer.cohortColors, layer.weight, measureOf(layer), rowsOf(layer), layer.cumulative] : null, layer.outline, ...fadingKey(layer)])]);
const fadingKey = (/** @type {Layer} */ layer) => (layer.type === "histogram" && layer.colors === 4 ? [ink(fadingOf(layer, false)), ink(fadingOf(layer, true))] : []);
/** Lightly smoothed: at this size, the trend is what reads. Bars keep the value farthest from their base. */
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

const place = (points, lo, hi, bottom = 0, height = 1) => points.map((value) => (value === null || !Number.isFinite(value) ? null : clamp(Math.round((bottom + ((value - lo) / (hi - lo || 1)) * height) * 100) / 100, 0, 1)));

/**
 * A failed fetch leaves nothing behind, no blank drawing and no cached failure. A series the server doesn't have, or
 * not at this interval, isn't a failure to retry: it's drawn without.
 * @param {Resolved} data @param {string} index
 */
function retryFailed(data, index) {
  for (const name of data.failed) {
    forget(name, index);
    retryInfo(name);
  }
  return data.failed.length > 0;
}
/** @param {Chart} doc @returns {Promise<Thumb | null>} */
async function drawThumb(doc) {
  // The open chart's own data, or any chart's all in memory: asked once, not again at another interval. Not a finer
    // interval's: only part of its history loads.
  const held = !FINE.has(doc.index) && (doc.id === chart?.id || drawSeries(doc).every((name) => loads.has(`${name}@${doc.index}`)));
  const index = held ? doc.index : await sampleable(doc, sampleIndex(doc, "month1", THUMB_POINTS));
  const data = await resolve({ ...doc, index, layers: doc.layers.map(layer => ({ ...layer, hidden: false })), ...(held ? { id: `${doc.id}:mark` } : {}) });
  if (retryFailed(data, index)) return null;
  const bubbles = await bubblesAt(doc, data, index);
  if (!bubbles) return null;
  /** @type {Map<string, ReturnType<typeof shapeOf>>} */
  const shapes = new Map(), own = new Map(), stacked = stackOf(doc, data, false);
  const start = rangeStart(doc, data.times);
  for (const layer of doc.layers) {
    const all = gapped(data.results.get(layer.id)?.values, data.results.get(layer.id)), values = all && start ? Array.prototype.slice.call(all, start) : all, highAll = stacked.get(layer.id)?.values, high = highAll && start ? Array.prototype.slice.call(highAll, start) : highAll, bars = layer.type === "histogram" && modeOf(doc, layer) === 0;
    if (!values || layer.type === "cloud" || placedOn(layer)) continue;
    const base = baseOf(layer);
    own.set(layer.id, shapeOf(values, modeOf(doc, layer), bars, base));
    shapes.set(layer.id, high ? shapeOf(high, modeOf(doc, layer), bars, base) : own.get(layer.id));
  }
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
  /** @type {Map<string, { index: number, band: { bottom: number, height: number } }>} */
  const lineAt = new Map();
  let top = 1;
  for (const { pane, size, layers } of groups) {
    const height = size / total, gap = groups.length > 1 ? Math.min(0.08, height * 0.25) : 0;
    const band = { bottom: top - height + gap / 2, height: height - gap };
    top -= height;
    for (const side of SIDES) {
      const on = layers.filter((layer) => layer.side === side).map((layer) => ({ layer, shape: shapes.get(layer.id) }));
      // A stack's foot too, on a linear scale: one layer alone, a 100% one, is a line at its top.
      const framing = on.filter(({ layer }) => !unfitted(doc, layer));
      const bases = framing.flatMap(({ layer, shape }) => [
        ...(layer.type === "histogram" && !summed(pane) && modeOf(doc, layer) === 0 ? [shape.scale(baseOf(layer))] : []),
        ...(zeroFit(doc, layer, modeOf(doc, layer), summed(pane)) ? [shape.scale(0)] : []),
      ]).filter(Number.isFinite);
      const lo = Math.min(...framing.map(({ shape }) => shape.lo), ...bases), hi = Math.max(...framing.map(({ shape }) => shape.hi), ...bases);
      if (!Number.isFinite(lo)) continue;
      for (const { layer, shape } of on) {
        const ys = place(shape.points, lo, hi, band.bottom, band.height);
        const kind = { area: "area", baseline: "baseline", histogram: modeOf(doc, layer) === 0 ? "bars" : undefined, dots: "dots" }[layer.type];
        const level = place([shape.scale(baseOf(layer))], lo, hi, band.bottom, band.height)[0] ?? band.bottom;
        const edge = layer.fill === "above" ? Math.min(1, band.bottom + band.height + gap / 2) : Math.max(0, band.bottom - gap / 2);
        const dateColors = paintOf(doc, layer, data), step = (data.times.length - start || 1) / THUMB_POINTS;
        const colors = dateColors && ys.map((value, k) => (value === null ? null : dateColors(start + Math.min(data.times.length - start - 1, Math.max(Math.floor(k * step), Math.floor((k + 1) * step) - 1))) ?? layer.color));
        if (colors) lines.push([layer.color, ys, kind ?? null, kind ? belowOf(layer) : null, kind ? (kind === "area" ? edge : level) : null, colors]);
        else lines.push(kind ? [layer.color, ys, kind, belowOf(layer), kind === "area" ? edge : level] : [layer.color, ys]);
        lineAt.set(layer.id, { index: lines.length - 1, band });
      }
    }
  }
  const washes = /** @type {Thumb["washes"]} */ ([]), dots = /** @type {Thumb["dots"]} */ ([]);
  for (const layer of doc.layers) {
    if (layer.hidden) continue;
    if (layer.type === "heatmap" && !washes.length && heatKnown(layer) && heatGrid(doc, layer)) {
      const behind = doc.layers.find((other) => other.pane === layer.pane && lineAt.has(other.id)), at = behind && lineAt.get(behind.id);
      if (!at) continue;
      const stops = heatGradient(doc, layer), one = stops ? stops[Math.floor(stops.length / 2)][1] : "";
      washes.push(stops ? [at.index, one, one, at.band.bottom, at.band.bottom + at.band.height] : [at.index, belowOf(layer), layer.color, at.band.bottom, at.band.bottom + at.band.height]);
    } else if (layer.type === "bubbles" && layer.ref && lineAt.has(layer.ref) && dots.length < 4) {
      const all = bubbles.get(layer.id) ?? data.results.get(layer.id)?.values, values = all && start ? Array.prototype.slice.call(all, start) : all, ys = lines[/** @type {{ index: number }} */ (lineAt.get(layer.ref)).index][1];
      if (!values) continue;
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
  const marks = /** @type {Record<string, (number | null)[]>} */ ({}), bases = /** @type {Record<string, number>} */ ({});
  for (const [id, { points, scale }] of own) {
    const there = points.filter((value) => value !== null);
    if (there.length < 2) continue;
    const layer = doc.layers.find((item) => item.id === id), level = layer && PAIRS[layer.type] ? scale(baseOf(layer)) : NaN;
    const range = layer?.type === "histogram" && modeOf(doc, layer) === 0 && Number.isFinite(level) ? [...there, level] : there;
    marks[id] = place(points, Math.min(...range), Math.max(...range));
    if (layer && PAIRS[layer.type]) bases[id] = place([level], Math.min(...range), Math.max(...range))[0] ?? 0;
  }
  return { lines, washes, dots: dots.slice(0, 4), marks, bases };
}
const orphan = (id) => !chartById(id);
function saveThumbs() {
  const merged = { ...(storage.get(THUMBS, {})), ...Object.fromEntries(thumbs) };
  for (const id of Object.keys(merged)) if (orphan(id)) delete merged[id];
  storage.set(THUMBS, merged);
}
/** @param {Chart} doc */
function updateMark(doc) {
  for (const mark of all$(`:is([data-open-chart="${doc.id}"], [data-open-recent="${doc.id}"]) .chart-mark`)) mark.outerHTML = chartMark(doc);
  if (doc !== chart) return;
  for (const element of all$(".legend .series-mark, .layer-row .row-mark > .series-mark")) {
    const layer = layerById(/** @type {HTMLElement} */ (element.closest("[data-select]")).dataset.select);
    if (layer) element.outerHTML = seriesMark(layer);
  }
}
/** @type {Map<string, { key: string, at: number }>} */
const thumbsFailed = new Map();
/** @param {Chart} doc @param {boolean} [picture] */
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

const PREVIEW = { width: 320, height: 200, points: 120 };

/** @param {Chart} doc */
const previewKey = (doc) => drawingKey([thumbKey(doc), doc.layers.map((layer) => [layer.colors, layer.dash, layer.opacity, layer.levels, layer.baseLine, layer.underName, layer.fillOpacity, layer.solid, layer.hollow, layer.size, layer.hide, layer.scaleTo, layer.byLayers]), openView(doc), FINE.has(doc.index) ? Math.round(engine.time.width / 50) : 0, doc.layers.some((layer) => heatLike(layer) && !layer.hidden) ? light : null, ...(profiled(doc) ? ["profile"] : [])]);
function previewBars(/** @type {Chart} */ doc, index = doc.index) {
  const width = engine.time.width, days = rangeDays(doc.range);
  // A short range at a coarse interval (3M of months): a dozen bars still, not a preview of none.
  return Math.min(FINE.has(doc.index) && width ? ((width / MIN_BAR_SPACING) * INTERVAL_DAYS[doc.index]) / INTERVAL_DAYS[index] : Infinity, days ? Math.max(12, days / INTERVAL_DAYS[index]) : Infinity);
}
const rangeStart = (/** @type {Chart} */ doc, /** @type {ArrayLike<number>} */ times) => {
  const days = rangeDays(doc.range);
  return days && times.length ? lowerBound(times, times[times.length - 1] - days * 86400) : 0;
};
/**
 * Kept within a budget: they're a cache, drawn again when missing, so they never crowd out the charts themselves.
 * Written oldest first, as they're read back: the last drawn last again.
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
 * A shown profile's room after the latest as the chart leaves it (see rightGap): a quarter of the picture where it ends
 * at the latest. A view's edges as the chart shows it (the library's range is its screen's edges); a range's from its
 * first bar's left edge to its last's right edge (its window is by their middles: half a bar more each side).
 * @param {Chart} doc @param {{ times: number[] }} data @param {number} most
 */
const previewShown = (doc, data, most) => {
  const left = openView(doc), shown = left ? within(viewOn(left, doc.index, data.times, rightGap()), "to") : presetWindow(doc, data, most, 0, 0), last = data.times.length - 1, ROOM = 0.25;
  const range = profiled(doc) && shown.to >= last - 0.5 && shown.to - last < (shown.to - shown.from) * ROOM ? { from: shown.from, to: last + ((last - shown.from) * ROOM) / (1 - ROOM) } : shown;
  return left ? range : { from: range.from - 0.5, to: range.to + 0.5 };
};
/**
 * A day's sum read once a week is one day in seven: its largest days, the bubbles that read, would be missed, so each
 * point takes the largest by size (sign kept) of the chart's own over its span, as the chart's buckets keep theirs.
 * Null when a fetch failed: the picture isn't kept, asked again (see retryFailed).
 * @param {Chart} doc
 * @param {{ times: number[], results: Map<string, any> }} data
 * @param {string} index
 * @returns {Promise<Map<string, Float64Array> | null>}
 */
async function bubblesAt(doc, data, index, signal = /** @type {AbortSignal | undefined} */ (undefined)) {
  const layers = doc.layers.filter((layer) => layer.type === "bubbles" && !layer.hidden), times = data.times, n = times.length;
  if (!layers.length || !(INTERVAL_DAYS[index] > INTERVAL_DAYS[doc.index])) return new Map();
  const fine = await resolve(structuredClone(doc), false, signal).catch(() => null);
  if (!fine || signal?.aborted || retryFailed(fine, doc.index)) return null;
  return new Map(layers.flatMap((layer) => {
    const values = fine.results.get(layer.id)?.values, own = data.results.get(layer.id)?.values;
    if (!values || !own) return [];
    // Its own reach, not the timeline's: another layer's series may reach where its own didn't load.
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
 * @typedef {Map<string, { days: (any)[], open: any, fine: any }> & { whole?: boolean }} HeatSamples
 * @param {Chart} doc
 * @param {{ times: number[] }} data
 * @param {number} most
 * @param {number} columns
 * @returns {Promise<HeatSamples>}
 */
async function heatSamples(doc, data, most, columns) {
  const layers = doc.layers.filter((layer) => heatLike(layer) && !layer.hidden && (layer.type === "heatmap" || layer.profile?.length) && heatKnown(layer) && heatGrid(doc, layer)), n = data.times.length;
  const shown = n > 1 ? previewShown(doc, data, most) : null, span = shown ? shown.to - shown.from : 0;
  if (!layers.length || !shown || !(span >= 1)) return Object.assign(/** @type {HeatSamples} */ (new Map()), { whole: true });
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
  const end = days[g[clamp(Math.floor(shown.to), 0, n - 1)]];
  const keyed = layers.map((layer) => {
    if (layer.type === "profile") return /** @type {[string, (string | null)[], string, string]} */ ([layer.id, (layer.profile ?? []).map((cohort) => want(cohort, layer.weight ?? "", HEAT_PICTURE_AGG, end)), "", ""]);
    const cohort = layer.heatmap ?? "all", weight = layer.weight ?? "", own = /** @type {NonNullable<ReturnType<typeof heatGrid>>} */ (heatGrid(doc, layer)).agg;
    return /** @type {[string, (string | null)[], string, string]} */ ([layer.id, at.map((day) => (day === null ? null : want(cohort, weight, HEAT_PICTURE_AGG, day))), want(cohort, weight, HEAT_PICTURE_AGG, "latest"), want(cohort, weight, own, "latest")]);
  });
  heatExtras.add(extra);
  try {
    heatPump();
    await heatCome([...extra.keys], 20_000, () => true);
    // Some not in yet, or failed for now: drawn, but not kept as up to date (see drawPreview).
    const samples = /** @type {HeatSamples} */ (new Map(keyed.map(([id, keys, open, fine]) => [id, { days: keys.map((key) => (key === null ? null : heatDays.get(key))), open: heatDays.get(open), fine: heatDays.get(fine) }])));
    samples.whole = [...extra.keys].every((key) => heatDays.get(key)?.rows || (heatDays.get(key)?.failed && !heatDays.get(key)?.retryAt));
    return samples;
  } finally {
    heatExtras.delete(extra);
  }
}
/**
 * Against the chart's reference (its open day's largest row, at its density); not come yet, the open day's largest cell
 * here: brighter, the gaps between its rows filled.
 * @param {Chart} doc
 * @param {Layer} layer
 * @param {Resolved | { times: number[], results: Map<string, any> }} data
 * @param {{ days: any[], open: any, fine: any } | undefined} sample
 * @param {(y: number) => number} priceAt
 * @param {number} top
 * @param {number} bottom
 */
function heatPicture(doc, layer, data, sample, priceAt, top, bottom) {
  const look = heatLook(doc, layer, /** @type {Resolved} */ (data), opacityOf(layer)), k = HEAT_PICTURE_ROWS, grid = HEAT_PICTURE_GRID, rows = modeOf(doc, layer) <= 1 ? rowsOf(layer) : k;
  const most = sample?.fine?.rows ? heatMost(sample.fine, look.measure) * (rows / k) : sample?.open?.rows ? heatMost(sample.open, look.measure) : 0;
  if (!sample || !most || !(bottom > top)) return null;
  const columns = sample.days.length, height = Math.round((bottom - top) * 2), canvas = document.createElement("canvas");
  canvas.width = columns;
  canvas.height = height;
  const context = /** @type {CanvasRenderingContext2D} */ (canvas.getContext("2d")), image = context.createImageData(columns, height), pixels = image.data;
  // Each pixel row's cell once; a cell's color once a day, copied down the pixel rows it covers. None where the price
    // isn't over zero: NaN.
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
 * The next picture of the same look from the same data keeps each pane drawn from the same: a live price, which gives
 * its own layer a new result alone, redraws its pane, not a stack's 23 bands.
 * @type {WeakMap<object, { look: string, times: ArrayLike<number>, panes: { from: any[], marks: any[], scales: any[] }[] }>}
 */
const panesDrawn = new WeakMap();
/** @param {Chart} doc @param {{ times: number[], results: Map<string, any> }} data @param {number} [most] @returns {Preview} */
function previewOf(doc, data, most = mostBars(), size = PREVIEW, heat = /** @type {HeatSamples} */ (new Map()), bubbles = /** @type {Map<string, Float64Array>} */ (new Map())) {
  const { width: W, height: H } = size;
  const times = data.times, n = times.length;
  /** @type {Preview} */
  const drawn = { seps: [], marks: [] };
  if (n < 2) return drawn;
  const shown = previewShown(doc, data, most), span = shown.to - shown.from;
  if (!(span >= 1)) return drawn;
  // No more points than bars in view: each one then holds a bar at least, so only missing data breaks a line.
  const B = Math.min(size.points, Math.floor(span));
  const big = size !== PREVIEW;
  if (big) {
    drawn.end = Math.min(n - 1, Math.floor(shown.to));
    drawn.ticks = dateTicks(times, shown.from, shown.to, W);
    drawn.scales = [];
  }
  const first = clamp(Math.ceil(shown.from), 0, n - 1), last = clamp(Math.floor(shown.to), 0, n - 1);
  /** @param {number} b */
  const middleOf = (b) => shown.from + ((b + 0.5) * span) / B;
  /** @param {number} b */
  const bucketOf = (b) => ((edge) => [Math.max(first, Math.ceil(shown.from + (b * span) / B)), Math.min(last, b === B - 1 ? Math.max(edge, Math.floor(shown.to)) : edge)])(Math.ceil(shown.from + ((b + 1) * span) / B) - 1);
  const round = (value) => Math.round(value * 10) / 10;
  const x = (/** @type {number} */ i) => round(((i - shown.from) / span) * W), xTexts = new Array(n), xText = (/** @type {number} */ i) => xTexts[i] ?? (xTexts[i] = String(x(i)));
  const hidden = hiddenPanes(doc), sizeOf = (pane, k) => (hidden.has(pane.id) ? 0 : pane.size ?? defaultSize(k));
  const total = doc.panes.reduce((sum, pane, i) => sum + sizeOf(pane, i), 0), stacked = stackOf(doc, data);
  const carried = new Set(doc.layers.filter((layer) => layer.type === "bubbles" && !layer.hidden && layer.ref).map((layer) => layer.ref));
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
    const from = drawn.marks.length, ranks = new Map(), inPaneOrder = doc.layers.filter((layer) => layer.pane === pane.id);
    const sources = /** @type {any[]} */ ([band.top, band.height]), scalesFrom = drawn.scales?.length ?? 0;
    for (const layer of inPaneOrder) for (const at of colorChain(doc, layer)) sources.push(data.results.get(at.id), heat.get(at.id), bubbles.get(at.id));
    const kept = before?.[k];
    if (kept && kept.from.length === sources.length && sources.every((item, i) => item === kept.from[i])) return void (drawn.marks.push(...kept.marks), drawn.scales?.push(...kept.scales), panes.push(kept));
    const zOf = new Map((pane.reverse ? inPaneOrder : [...inPaneOrder].reverse()).map((layer, z) => [layer.id, z]));
    /** @param {Layer} layer @param {(i: number) => number} place */
    const drawBubbles = (layer, place) => {
      const own = bubbles.get(layer.id) ?? data.results.get(layer.id)?.values;
      if (layer.hidden || !own) return;
      const most = layer.scaleTo === "all" ? mostIn(own) : mostIn(own, shown), least = (layer.hide ?? BUBBLE_HIDE) * most, across = (layer.size ?? BUBBLE_SIZE) * (big ? 1 : 0.6);
      if (!(most > 0)) return;
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
    for (const side of SIDES) {
      const shaped = doc.layers.filter((layer) => layer.pane === pane.id && layer.side === side && (!layer.hidden || carried.has(layer.id)) && !placedOn(layer) && data.results.get(layer.id)?.values && (!fromUnfit(doc, layer) || carried.has(layer.id))).map((layer) => {
        const values = /** @type {Vector} */ (gapped(stacked.get(layer.id)?.values ?? data.results.get(layer.id)?.values, data.results.get(layer.id))), mode = modeOf(doc, layer);
        let base = NaN;
        for (let i = first; mode >= 2 && i <= last && !Number.isFinite(base); i++) if (Number.isFinite(values[i]) && values[i] !== 0) base = values[i];
        const scaled = onScale(mode, base), inView = onScaleAll(values, mode, base, first, last);
        // Each bucket's low and high, in the order they came, and the latest value last: at up to four bars a pixel the
                // chart shows every bar, so its peaks and troughs show; an average would round them off.
        const lows = new Float64Array(B).fill(Infinity), highs = new Float64Array(B).fill(-Infinity), lowAt = new Float64Array(B), highAt = new Float64Array(B);
        let latest = -1, latestIn = -1;
        const breaks = data.results.get(layer.id)?.breaks, cut = breaks ? new Uint8Array(B) : null;
        for (let i = first; i <= last; i++) {
          // The point at the view's very end is its last bucket's, not past it. Its bucket by a truncation, no call; a
                    // point before the view's start has its place under 0 but never under -1: none.
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
        const end = latestIn >= 0 ? points[latestIn] : null;
        if (end && latest > end[end.length - 1][0]) end.push([latest, scaled(values[latest])]);
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
        const members = result?.members, ids = result?.memberIds;
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
        const parts = baseOf(layer);
        return { layer, unseen: Boolean(layer.hidden || fromUnfit(doc, layer)), mode, from: base, scaled, values, inView, bars: layer.type === "histogram" && mode === 0, base: parts, level: scaled(parts), points: whole ? band?.map(() => null) ?? points : ranged ? ranged.map((range, b) => (range ? range.map((value) => /** @type {[number, number]} */ ([middleOf(b), value])) : null)) : points, band, bands, stack: stacked.has(layer.id) };
      });
      // A stack's layers, sampled at the same points, every one's highs and lows: each is the sum of those under it, so
            // their edges never cross, and none loses its peaks (a 100% stack's top is flat).
      const stack = shaped.filter((item) => item.stack);
      if (stack.length > 1) {
        // Finite by a subtraction, by loops: no call per point.
        const shared = new Array(B);
        if (big) for (let b = 0; b < B; b++) {
          const all = [], indices = [];
          for (const item of stack) for (const point of item.points[b] ?? []) all.push(point[0]);
          all.sort((a, z) => a - z);
          for (let k = 0; k < all.length; k++) if (k === 0 || all[k] !== all[k - 1]) indices.push(all[k]);
          shared[b] = indices;
        }
        else {
          // A 100% stack's top is flat, whatever its sums' rounding.
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
            // A band with values in the bucket but not at each place its top's taken (one starting, ending or there for
                        // a while within it) takes its first and last there too, else it would be a point there or gone. Told
                        // against where the top's taken alone: another's added there could hide it again.
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
        if (unfitted(doc, layer)) continue;
        if (bars && !unseen) (lo = Math.min(lo, base)), (hi = Math.max(hi, base));
        if (zeroFit(doc, layer, mode, Boolean(stack)) && !unseen) (lo = Math.min(lo, 0)), (hi = Math.max(hi, 0));
        for (const point of points) for (const [, value] of point ?? []) (lo = Math.min(lo, value)), (hi = Math.max(hi, value));
      }
      for (const item of shaped) {
        const values = carried.has(item.layer.id) ? stacked.get(item.layer.id)?.values ?? data.results.get(item.layer.id)?.values : null;
        for (let i = first; values && i <= last; i++) ((value) => Number.isFinite(value) && ((lo = Math.min(lo, value)), (hi = Math.max(hi, value))))(item.scaled(values[i]));
      }
      const heats = doc.layers.filter((layer) => layer.pane === pane.id && layer.side === side && heat.has(layer.id));
      if (!Number.isFinite(lo) || lo === hi) for (const layer of heats) {
        const closes = data.results.get(layer.id)?.closes;
        if (!closes) continue;
        const [low, high] = rangeOf(onScaleAll(closes, modeOf(doc, layer), NaN, first, last));
        (lo = Math.min(lo, low)), (hi = Math.max(hi, high));
      }
      if (!Number.isFinite(lo) && !shaped.some((item) => item.layer.type === "cloud" && edgesOf(item.layer).length > 1)) continue;
      if (!Number.isFinite(lo)) (lo = 0), (hi = 1);
      // One value all along: half a unit either side of it, it in the middle.
      if (lo === hi) (lo -= 0.5), (hi += 0.5);
      if (pane.stack === "percent") (lo = 0), (hi = 100);
      const range = hi - lo;
      const y = (value) => round(band.top + band.height * (range ? 1 - (value - lo) / range : 0.5));
      const inPane = (value) => clamp(value, paneTop, paneBottom);
      for (const layer of heats) {
        if (layer.type !== "heatmap") continue;
        const img = range ? heatPicture(doc, layer, data, /** @type {any} */ (heat.get(layer.id)), (place) => ((value) => (modeOf(doc, layer) === 1 ? 10 ** value : value))(lo + (1 - (place - band.top) / band.height) * range), paneTop, paneBottom) : null;
        if (!img) continue;
        const mark = { k: "heat", c: layer.color, d: "", img, top: paneTop, bottom: paneBottom };
        drawn.marks.push(mark), ranks.set(mark, (zOf.get(layer.id) ?? 0) - doc.layers.length - 1);
      }
      for (const layer of heats) {
        const sample = heat.get(layer.id), scaled = onScale(modeOf(doc, layer), NaN), grid = HEAT_PICTURE_GRID, measure = measureOf(layer);
        const groups = layer.type === "profile" && range ? (layer.profile ?? []).map((cohort, c) => ({ cohort, day: sample?.days[c] })).filter((item) => item.day?.rows) : [];
        if (!groups.length) continue;
        const BIN = 4, reach = columnWidth(W), bins = Math.max(1, Math.ceil((paneBottom - paneTop) / BIN)), count = groups.length, sums = new Float64Array(bins * count), totals = new Float64Array(bins);
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
      if (big && drawn.scales) {
        let most = null, least = null;
        for (const item of shaped) {
          if (item.unseen) continue;
          for (const point of item.points) {
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
        const partial = periodAmount(layer) ? data.results.get(layer.id)?.values?.findLastIndex(Number.isFinite) ?? -1 : -1;
        const across = acrossUnfit(doc, layer) ? [] : [...(layer.levels ?? []).map((item, i) => [String(i), item.value]), ...(layer.type === "baseline" ? [["base", base]] : [])];
        for (const [key, value] of across) {
          const yAt = y(scaled(value)), { color, dash, line } = lineLook(layer, key), [, , length, gap] = dashOf(dash);
          if (line !== false && yAt > paneTop && yAt < paneBottom) drawn.marks.push({ k: "across", c: color, d: `M0 ${yAt}H${W}`, ...(gap ? { dash: `${length} ${gap}` } : {}), ...(opacityOf(layer) < 1 ? { op: opacityOf(layer) } : {}) });
        }
        const at = (/** @type {[number, number]} */ point) => `${xText(point[0])} ${y(point[1])}`;
        const look = { c: layer.color, ...(opacityOf(layer) < 1 ? { op: opacityOf(layer) } : {}), ...(layer.outline && OUTLINED.has(layer.type) ? { o: doc.layers.indexOf(layer) + 1 } : {}) };
        const own = layer.type === "area" && layer.fill !== "above" ? stacked.get(layer.id) : undefined;
        const under = own ? [...stacked].filter(([id, item]) => item.group === own.group && item.rank < own.rank && ((band) => (band?.type === "area" && band.fill !== "above") || band?.type === "line")(doc.layers.find((item) => item.id === id))).map(([, item]) => item).sort((a, b) => b.rank - a.rank) : [];
        // A loop, finite by a subtraction: no call per band point (see stackOf).
        const floorAt = own && ((/** @type {number} */ i) => {
          let value = 0;
          for (const item of under) if (item.values[i] - item.values[i] === 0) {
            value = item.values[i];
            break;
          }
          const at = y(scaled(value));
          return at - at === 0 && at < paneBottom ? at : paneBottom;
        });
        /** @param {[number, number][]} part */
        const backAlong = (part) => {
          let back = "";
          for (let p = part.length - 1; p >= 0; p--) back += `L${xText(part[p][0])} ${/** @type {(i: number) => number} */ (floorAt)(part[p][0])}`;
          return back;
        };
        const zones = layer.byLevels && layer.levels?.length && (layer.type === "line" || layer.type === "area") && !stacked.has(layer.id) ? levelSteps(layer).map((step) => /** @type {[number, string]} */ ([y(scaled(step.value)), step.color])) : null;
        const dateColors = zones ? null : paintOf(doc, layer, data), tintAt = (/** @type {number} */ i) => (dateColors && dateColors(Math.round(i))) ?? layer.color;
        /** @template T @param {{ color: string, points: T[] }[]} pieces @returns {Map<string, T[][]>} */
        const perColor = (pieces) => {
          const by = new Map();
          for (const piece of pieces) (by.get(piece.color) ?? by.set(piece.color, []).get(piece.color))?.push(piece.points);
          return by;
        };
        /** @param {any[]} buckets */
        const runsOf = (buckets) => {
          /** @type {number[][]} */
          const runs = [];
          let run = [];
          buckets.forEach((item, b) => (item ? run.push(b) : run.length && (runs.push(run), (run = []))));
          if (run.length) runs.push(run);
          return runs;
        };
        /** @template T @param {T[][]} parts @returns {(T | null)[]} */
        const joined = (parts) => parts.flatMap((part, k) => (k ? [null, ...part] : part));
        /** @param {([number, number] | null)[]} ranges @param {(b: number) => string} tint */
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
        const runs = runsOf(points);
        if (!runs.length) return;
        const whole = (/** @type {number[]} */ part) => part.flatMap((b) => /** @type {[number, number][]} */ (points[b]));
        const final = whole(runs[runs.length - 1]).slice(-2), tail = layer.type === "line" && final.length === 2 && final[1][0] === partial && final[0][0] === partial - 1 ? final : null;
        const along = (/** @type {number[]} */ part) => (tail ? whole(part).filter(([i]) => i !== partial) : whole(part));
        const pieces = (dateColors ? byColor(joined(runs.map(along)), ([i]) => tintAt(i), layer.type !== "dots") : runs.map((run) => ({ color: layer.color, points: along(run) }))).filter((piece) => piece.points.length), last = pieces.at(-1);
        if (!last) return;
        const [endAt, endValue] = /** @type {[number, number]} */ (tail ? tail[1] : last.points.at(-1));
        const [, , length, gap] = LINED.has(layer.type) ? dashOf(layer.dash) : DASHES[0];
        for (const [c, parts] of [...perColor(pieces)].sort(([a], [b]) => Number(a === last.color) - Number(b === last.color))) {
          if (layer.type === "dots") {
            drawn.marks.push({ ...look, c, k: "dots", d: parts.flat().map((point) => `M${at(point)}h0`).join("") });
            continue;
          }
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
      for (const layer of doc.layers) {
        const on = layer.type === "bubbles" && layer.pane === pane.id && shaped.find((item) => item.layer.id === layer.ref), values = on && (stacked.get(on.layer.id)?.values ?? data.results.get(on.layer.id)?.values);
        if (on && values) drawBubbles(layer, (i) => y(on.scaled(values[i])));
      }
    }
    for (const layer of doc.layers) if (layer.type === "bubbles" && layer.pane === pane.id && !doc.layers.some((other) => other.id === layer.ref)) drawBubbles(layer, () => round((paneTop + paneBottom) / 2));
    drawn.marks.splice(from, Infinity, ...drawn.marks.slice(from).sort((a, b) => ranks.get(a) - ranks.get(b)));
    panes.push({ from: sources, marks: drawn.marks.slice(from), scales: drawn.scales?.slice(scalesFrom) ?? [] });
  });
  panesDrawn.set(data, { look, times, panes });
  return drawn;
}
/** @param {[number, string][]} z @param {number} height @param {string} own @returns {{ y: number, h: number, c: string }[]} */
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
const closedTo = (d, edge) => d.split("M").filter(Boolean).map((part) => `M${part.split(" ")[0]} ${edge}L${part}L${part.slice(part.lastIndexOf("L") + 1).split(" ")[0]} ${edge}Z`).join("");
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
 * Its ids are its place's (`place`: a tile's, a card's, ⌘K's, Quick Look's), the same each time: the same chart can
 * show twice at once, and a tile drawn again is unchanged.
 * @param {Preview} drawn @param {string} place
 */
function previewSvg(drawn, place, size = PREVIEW) {
  const { width: W, height: H } = size, id = place.replace(/[^\w-]/g, "-"), defs = [];
  const body = drawn.seps.map((y) => `<path class="sep" d="M0 ${y}H${W}"/>`);
  // Fills, then bands, then lines, as the chart draws them: a band of one of its colors is never over another's line; a
    // band whole, as the chart's ground is, its layer's opacity on its fill and line alone.
  const parts = drawn.marks.map((mark, n) => {
    const c = escapeHtml(ink(mark.c)), c2 = escapeHtml(ink(mark.c2 ?? mark.c)), g = `${id}-${n}`, dash = mark.dash ? ` stroke-dasharray="${mark.dash}"` : "";
    const halo = (/** @type {string | undefined} */ d, kind = "") => (mark.o && d ? `<path class="halo${kind}"${dash} d="${d}"/>` : "");
    const fade = (key, color, from, to, a, b) => `<linearGradient id="${g}${key}" gradientUnits="userSpaceOnUse" x1="0" x2="0" y1="${from}" y2="${to}"><stop offset="0" stop-color="${color}" stop-opacity="${a}"/><stop offset="1" stop-color="${color}" stop-opacity="${b}"/></linearGradient>`;
    let under = "", band = "", over = "", ring = "";
    // A heatmap's picture, its pixels as they are: smoothed, a cell would bleed into the empty ones, a dark fringe.
    if (mark.k === "heat") under = `<image href="${mark.img}" x="0" y="${mark.top}" width="${W}" height="${/** @type {number} */ (mark.bottom) - /** @type {number} */ (mark.top)}" preserveAspectRatio="none" style="image-rendering: pixelated"/>`;
    else if (mark.k === "across") over = `<path class="across" stroke="${c}"${dash} d="${mark.d}"/>`;
    else if (mark.k === "profile") under = `<path class="fill" style="fill:color-mix(in srgb, ${c} ${Math.round((mark.f?.[0] ?? 1) * 100)}%, var(--plot-ground, var(--card)))" d="${mark.d}"/>`;
    else if (mark.k === "band") under = `<path class="fill" style="fill:${c}" d="${mark.d}"/>`;
    else if (mark.k === "bars") (band = `${halo(mark.d, " of-bars")}${halo(mark.d2, " of-bars")}`), (over = `<path class="bars" stroke="${c}" d="${mark.d}"/>${mark.d2 ? `<path class="bars" stroke="${c2}" d="${mark.d2}"/>` : ""}`);
    else if (mark.k === "dots") (band = halo(mark.d, " of-dots")), (over = `<path class="dots" stroke="${c}" d="${mark.d}"/>`);
    // Bubbles of a size: each its edge's dot, its fill's dot a pixel in over it, both mixed with the ground (the
        // chart's are see-through); a dot keeps its size as the frame stretches, so each is round. Outlined: a ring of the
        // ground under each size's, over the larger ones', as the chart rings each in turn.
    else if (mark.k === "bubble") (defs.push(`<clipPath id="${g}c"><rect y="${mark.top}" width="${W}" height="${/** @type {number} */ (mark.bottom) - /** @type {number} */ (mark.top)}"/></clipPath>`), (ring = mark.o ? `<g clip-path="url(#${g}c)">${[mark.d, mark.d2].map((d) => (d ? `<path class="bubble" style="stroke-width:${(mark.w ?? 0) + 2}px;stroke:var(--plot-ground, var(--card))" d="${d}"/>` : "")).join("")}</g>` : ""), (over = `<g clip-path="url(#${g}c)">${[[c, mark.d], [c2, mark.d2]].map(([color, d]) => (d ? `<path class="bubble" style="stroke-width:${mark.w}px;stroke:color-mix(in srgb, ${color} 80%, var(--plot-ground, var(--card)))" d="${d}"/>${(mark.w ?? 0) > 3 ? `<path class="bubble" style="stroke-width:${(mark.w ?? 0) - 2}px;stroke:color-mix(in srgb, ${color} 40%, var(--plot-ground, var(--card)))" d="${d}"/>` : ""}` : "")).join("")}</g>`));
    else if (mark.k === "baseline") {
      const y0 = /** @type {number} */ (mark.y0), f = closedTo(mark.d, y0);
      defs.push(`<clipPath id="${g}a"><rect width="${W}" height="${y0}"/></clipPath><clipPath id="${g}b"><rect y="${y0}" width="${W}" height="${H - y0}"/></clipPath>`, fade("u", c, mark.top, y0, 0.24, 0.01), fade("v", c2, y0, mark.bottom, 0.01, 0.24));
      under = `<g clip-path="url(#${g}a)"><path class="fill" style="fill:url(#${g}u)" d="${f}"/></g><g clip-path="url(#${g}b)"><path class="fill" style="fill:url(#${g}v)" d="${f}"/></g>`;
      band = halo(mark.d);
      over = `<g clip-path="url(#${g}a)"><path stroke="${c}"${dash} d="${mark.d}"/></g><g clip-path="url(#${g}b)"><path stroke="${c2}"${dash} d="${mark.d}"/></g>`;
    } else {
      const [top, foot] = mark.f ?? (mark.up ? [0.12, 0.12] : mark.band ? [STACK_FILL, STACK_FILL] : [0.2, 0]);
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
/** @type {WeakMap<object, number>} */
const drawingIds = new WeakMap();
let drawingsNumbered = 0;
const drawingId = (/** @type {object} */ drawn) => drawingIds.get(drawn) ?? (drawingIds.set(drawn, ++drawingsNumbered), drawingsNumbered);
/** @type {Map<string, { framed: string, drawn: Promise<(Preview & { key: string }) | null>, stop: AbortController }>} */
const plotsDrawing = new Map();
/**
 * Drawings under way with no frame of their chart left in the page, nor a slot for their tile, are given up, with the
 * fetches no other asker wants (see cachedLoad). Out of the map before it's aborted: an ask meanwhile (the tile back)
 * draws anew rather than joining one that will paint nothing.
 */
function giveUpUnframed() {
  for (const [ask, { framed, stop }] of plotsDrawing) if (!document.querySelector(framed)) plotsDrawing.delete(ask), stop.abort();
}
/** @param {string} id @param {Preview | null} drawn */
function paint(id, drawn, all = false) {
  if (!drawn) drawnTiles.delete(id);
  for (const frame of all$(`[data-plot="${CSS.escape(id)}"]`)) {
    if (!all && !frame.classList.contains("pending")) continue;
    frame.classList.remove("pending");
    if (drawn) (frame.innerHTML = previewSvg(drawn, `${frame.classList[0]}-${id}`)), (frame.dataset.drawn = String(drawingId(drawn)));
    // None, with no picture before it: said, not a blank that reads as no data; asked for again (see retryPlots). One
        // from before stays: a sketch of the same chart.
    frame.classList.toggle("failed", !drawn && !frame.querySelector("svg"));
  }
}
/** @param {Chart} doc @returns {Promise<(Preview & { key: string }) | null>} */
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
    const [heat, bubbles] = await Promise.all([heatSamples(copy, data, previewBars(doc, index), HEAT_TILE_COLUMNS), bubblesAt(doc, data, index, signal)]);
    if (!bubbles || previewKey(doc) !== key) return null;
    // Replaced meanwhile (another tab's edit): its successor is drawn on its own.
    if (!/^(recommended|preset):/.test(doc.id) && chartById(doc.id) !== doc) return null;
    if (preset >= 0 && !presetDocs.has(preset)) return null;
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
/** @param {Chart} doc @param {string} index */
async function sampleable(doc, index, signal = /** @type {AbortSignal | undefined} */ (undefined)) {
  if (index !== doc.index && !seriesIn(doc).every(intervalsOf)) await Promise.allSettled(seriesIn(doc).map((name) => info(name, signal)));
  return supportedIntervals(doc)(index) ? index : doc.index;
}
/** @param {Chart} doc */
const previewIndex = (/** @type {Chart} */ doc) => sampleIndex(doc, "week1", 100);
/** @param {Chart} doc @param {string} coarsest @param {number} points */
function sampleIndex(doc, coarsest, points) {
  if (INTERVAL_ORDER.indexOf(doc.index) >= INTERVAL_ORDER.indexOf(coarsest) || doc.layers.some((layer) => runsWindow(layer.formula ?? ""))) return doc.index;
  const days = rangeDays(doc.range);
  return [coarsest, "week1", "day1"].find((id) => !days || days / INTERVAL_DAYS[id] >= points) ?? "day1";
}
/** @param {Chart} doc */
function plotHtml(doc, current = false, frame = "tile-plot") {
  const drawn = previews.get(doc.id);
  if (!doc.layers.length) return `<span class="tile-plot blank"></span>`;
  return `<span class="${frame} ${drawn && (!current || drawn.key === previewKey(doc)) ? "" : "pending"}" data-plot="${doc.id}"${drawn ? ` data-drawn="${drawingId(drawn)}"` : ""}>${drawn ? previewSvg(drawn, `${frame.split(" ")[0]}-${doc.id}`) : ""}</span>`;
}

/** @param {Chart} doc */
const tileIndexHtml = (doc) => (doc.index === "day1" ? "" : `<span class="tile-index">${INTERVAL_LABEL[doc.index]}</span>`);
/** @param {Chart} doc */
const cardLegend = (doc) => doc.layers.filter((layer) => !layer.hidden).map((layer) => legendEntry(doc, layer)).join("");
/** @param {Chart} doc */
function tileHtml(doc) {
  const title = escapeHtml(titleOf(doc)), on = picked.has(doc.id), preview = isPreview(doc);
  const legend = cardLegend(doc);
  return `<div class="tile ${on ? "picked" : ""} ${preview ? "preview" : ""} ${preview && touched.has(doc) ? "edited" : ""}" data-home-card="${doc.id}" data-chart="${doc.id}">
            <button type="button" class="tile-open" data-open-chart="${doc.id}" aria-label="${title}" data-tip="${title}"><span class="tile-head"><span class="tile-name">${title}</span>${tileIndexHtml(doc)}</span><span class="tile-legend">${legend || (doc.layers.length ? "Every layer is hidden" : "No layers yet")}</span>${plotHtml(doc)}</button>
            ${moreCap("tile-more", "data-tile-menu", doc.id, titleOf(doc))}
            ${preview ? "" : `<input type="checkbox" class="pick-check tile-check" style="--tick:${escapeHtml(tickOn("ink"))}" aria-label="Select ${title}" data-tip="${on ? "Deselect" : "Select"}" ${on ? "checked" : ""} />`}
          </div>`;
}
const previewsShown = () => foundOf(listedPreviews());
/** @param {Chart[]} docs */
function foundOf(docs) {
  const query = $("home-filter").value.trim().toLowerCase();
  if (!query || !docs.length) return docs;
  const found = new QuickMatch(docs.map(wordsOf)).matchesBestWithIdsAndMatchedWords(query, new QuickMatchConfig().withLimit(docs.length).withUnionFallback(false));
  return found.map(([id]) => docs[id]);
}
/** @param {Chart} doc */
const wordsOf = (doc) => [titleOf(doc), doc.folder ? folderPath(doc.folder) : "", ...doc.layers.flatMap((layer) => [labelOf(layer), layer.series ?? "", layer.formula ?? ""])].join(" ").toLowerCase().replace(/_/g, " ");
let shownYours = /** @type {Chart[]} */ ([]), yoursFound = 0;
/** @type {Map<Chart, number>} */
let presetTiles = new Map();
const shownCharts = () => shownYours.filter((doc) => library.charts.includes(doc));
let filterIndex = { key: "", charts: /** @type {Chart[]} */ ([]), index: /** @type {any} */ (null) };
function filtered() {
  const query = $("home-filter").value.trim().toLowerCase();
  if (!query) return library.charts;
  const key = JSON.stringify([library.charts.map((doc) => [doc.id, doc.updated, doc.folder ?? ""]), library.folders.map((folder) => [folder.id, folder.name, folder.parent ?? ""])]);
  if (filterIndex.key !== key) filterIndex = { key, charts: library.charts.slice(), index: new QuickMatch(library.charts.map(wordsOf)) };
  const found = filterIndex.index.matchesBestWithIdsAndMatchedWords(query, new QuickMatchConfig().withLimit(filterIndex.charts.length).withUnionFallback(false));
  return found.map((/** @type {[number]} */ [id]) => filterIndex.charts[id]);
}
/** @type {Map<string, string>} */
const drawnTiles = new Map();
let drawnLayout = "";
function renderHomeHead() {
  const yours = "folder" in homeAt, folder = "folder" in homeAt ? folderById(homeAt.folder) : undefined, name = homeName(), top = "top" in homeAt;
  $("home-title").textContent = name;
  homeScroll().classList.toggle("at-home", top);
  $("home-tools").hidden = top;
  $("home-start").hidden = !top;
  const undo = /** @type {HTMLElement} */ (document.querySelector(".home-undo"));
  if (top && undo.parentElement !== $("home-top")) $("home-top").prepend(undo);
  else if (!top && undo.parentElement !== $("home-title").parentElement) /** @type {HTMLElement} */ ($("home-title").parentElement).append(undo);
  for (const own of [$("home-horizon"), $("home-help"), $("home-top")]) {
    if (own.hidden === !top) continue;
    if (!top && started) fadeOut(own, [{ opacity: 1 }, { opacity: 0 }], MOTION.small, true, MOTION.ease);
    else cutFade(own);
    own.hidden = !top;
    if (top && started && !reduced()) own.animate({ opacity: [0, 1] }, { duration: MOTION.small, easing: MOTION.ease });
  }
  const more = $("home-title-more");
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
  $("home-new-folder").hidden = !yours;
  $("home-foot").hidden = !yours || Boolean(folder) || !anythingKept();
  renderSideNav();
}

/** @param {Step[]} steps */
function crumbsHtml(steps) {
  return steps.map((step) => `<button type="button" dir="auto" data-place="${escapeHtml(JSON.stringify(step.at))}"><span>${escapeHtml(step.name)}</span></button>`).join('<span class="sep" aria-hidden="true">›</span>');
}

/** @returns {Action[]} */
/** A place's mark, as its row's (`open`: one the open chart's in). */
function placeMark(/** @type {any} */ at, open = false) {
  const folder = at && "folder" in at && at.folder ? folderById(at.folder) : undefined;
  return folder ? folderMark(folder) : at && "path" in at ? sectionMark(presetNodeAt(at.path), open) : open ? ICONS.folderOpen : ICONS.folder;
}
function chartPathRows() {
  const place = chartPlace();
  if (!place) return [];
  const steps = stepsTo(place), at = /** @type {Step} */ (steps.at(-1)).name;
  const page = (/** @type {Step} */ step, open = false) => ({ icon: placeMark(step.at, open), label: step.name, current: false, run: () => goTo(step.at) });
  const doc = (/** @type {Chart} */ item) => ({ group: at, icon: chartMark(item), label: titleOf(item), current: item === chart, run: () => go(chartById(item.id)) });
  /** @type {Action[]} */
  const inside = "path" in place
    ? (presetNodeAt(place.path)?.items ?? []).map((item) => (typeof item === "number"
      ? { group: at, icon: presetMark(item), label: ((edited) => (edited ? titleOf(edited) : /** @type {Preset[]} */ (presets)[item].t))(keptPreset(item)), current: item === presetIndex(chart), run: () => pickPreset(item) }
      : { group: at, ...page({ name: item.name, at: { path: item.path } }) }))
    : "folder" in place ? childrenOf(place.folder || undefined).map((item) => ("layers" in item ? doc(item) : { group: at, ...page({ name: item.name, at: { folder: item.id } }) }))
    : listedPreviews().map(doc);
  if ("path" in place) for (const item of presetNodeAt(place.path)?.items ?? []) if (typeof item === "number" && !presetMarks.has(presetMarkKey(item))) presetMarksAsked.add(item);
  drawPresetMarks();
  return [...steps.map((step) => page(step, true)), ...inside];
}

/** @typedef {{ kind: "grid", docs: Chart[], previews?: boolean } | { kind: "shelf", name: string, place: Place, folder?: string, docs: Chart[], count: number } | { kind: "note", html: string, card?: boolean, flat?: boolean }} YoursPart */
const yourQueue = /** @type {Set<string>} */ (new Set()), yoursSeen = /** @type {Set<Element>} */ (new Set());
function queueYours() {
  yourQueue.clear();
  for (const frame of yoursSeen) if (frame.isConnected) yourQueue.add(/** @type {string} */ (/** @type {HTMLElement} */ (frame).dataset.plot));
  void drawYours();
}
let horizonDrawn = "", horizonFailed = false, horizonAsking = false;
function renderHorizon() {
  const box = $("home-horizon");
  if (box.hidden) return;
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
    const halvings = minedHalvings().filter((time) => time > from && time < to).map((time) => ((at) => `<line x1="${x(time).toFixed(1)}" x2="${x(time).toFixed(1)}" y1="${y(at).toFixed(1)}" y2="${H}" class="halving"/>`)((points.find(([when]) => when >= time) ?? points[points.length - 1])[1])).join("");
    box.innerHTML = `<svg viewBox="0 0 ${W} ${H}" preserveAspectRatio="none"><defs>${fadeDef("horizon-fill", color, ".12")}<linearGradient id="horizon-in" x1="0" x2="1"><stop offset="0" stop-color="#fff" stop-opacity="0"/><stop offset=".35" stop-color="#fff" stop-opacity=".55"/><stop offset="1" stop-color="#fff"/></linearGradient><mask id="horizon-mask"><rect width="${W}" height="${H}" fill="url(#horizon-in)"/></mask></defs><g mask="url(#horizon-mask)">${halvings}${areaUnder("horizon-fill", line, W, H)}${lineOver(line, color, 0.6)}</g></svg>`;
  }).catch(() => (horizonFailed = true));
}
let doorsDrawn = "";
const GROUP_MARK = /** @type {Record<Group, string>} */ ({ recent: "recent", previews: "eye", library: "keep", presets: "presets" });
function renderDoors() {
  const box = $("home-doors");
  box.hidden = !home || !("top" in homeAt);
  if (box.hidden) return;
  const counts = /** @type {Record<Group, number | null>} */ ({ recent: recentCharts().length, previews: listedPreviews().length, library: library.charts.length, presets: presets?.length ?? null });
  const door = (/** @type {Group} */ group) => {
    const [name, place] = GROUP_OF[group], count = counts[group];
    return `<button type="button" class="btn ghost" data-place="${escapeHtml(JSON.stringify(place))}" aria-label="${count === null ? name : `${name}, ${plural(count, "chart")}`}">${ICONS[GROUP_MARK[group]]}${name}${count ? `<span class="count">${grouped.format(count)}</span>` : ""}</button>`;
  };
  const html = `<nav class="doors" aria-label="Go to">${GROUPS.filter((group) => counts[group] !== 0 || group === "library" || group === "presets").map(door).join("")}</nav><button type="button" class="btn ghost home-new" data-action="new-chart" data-tip="New chart" data-kbd="N">${ICONS.plus}New chart</button>`;
  if (html === doorsDrawn) return;
  const focus = focusKey(box);
  box.innerHTML = doorsDrawn = html;
  restoreFocus(box, focus);
}
function renderYours() {
  const grid = $("home-grid"), folderId = "folder" in homeAt ? homeAt.folder || undefined : undefined, onPreviews = "previews" in homeAt;
  const query = $("home-filter").value.trim();
  grid.hidden = !yoursShown();
  /** @type {YoursPart[]} */
  const parts = [];
  presetTiles = new Map();
  if (!grid.hidden && "recent" in homeAt) {
    const items = recentCharts(), found = query ? foundOf(items.map((item) => item.doc)) : null, shown = found ? items.filter((item) => found.includes(item.doc)) : items;
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
    if (own.length) parts.push({ kind: "grid", docs: own });
    for (const folder of items.flatMap((item) => ("layers" in item ? [] : [item]))) parts.push({ kind: "shelf", name: folder.name, place: { folder: folder.id }, folder: folder.id, docs: treeOf(folder.id).flatMap(({ item }) => ("layers" in item ? [item] : [])).slice(0, SHELF), count: counts.get(folder.id) ?? 0 });
    if (!items.length) parts.push({ kind: "note", html: folderId ? `<p>${TOUCH ? "Nothing in it yet. Start a chart in it, or hold a chart, then tap Move." : "Nothing in it yet. Start a chart in it, or drag charts onto it in the sidebar."}</p><div class="home-none-actions"><button class="btn small" type="button" data-action="new-chart">${ICONS.plus}New chart</button><button class="btn ghost small" type="button" data-action="import" data-into="${folderId}">${ICONS.upload}Import charts</button></div>` : `<p>No charts yet. Start a new chart, or open a preset and save a copy.</p><div class="home-none-actions"><button class="btn small" type="button" data-action="new-chart">${ICONS.plus}New chart</button><button class="btn ghost small" type="button" data-action="import">${ICONS.upload}Import charts</button></div>`, card: true });
  }
  const visible = parts.flatMap((part) => (part.kind === "grid" || part.kind === "shelf" ? part.docs : []));
  yoursFound = visible.length;
  shownYours = visible.filter((doc) => !isPreview(doc) && !presetTiles.has(doc));
  const ids = new Set(shownYours.map((doc) => doc.id));
  for (const id of [...picked]) if (!ids.has(id)) picked.delete(id);
  if (!picked.has(pickedLast)) pickedLast = [...picked].at(-1) ?? "";
  const focused = /** @type {HTMLElement | null} */ (document.activeElement?.closest(".tile-open")?.closest(".tile") ?? null)?.dataset.chart;
  // Only the tiles that changed are drawn again: a click on another, under way, isn't lost; focus and hover stay.
  const html = visible.map((doc) => ((i) => (i === undefined ? tileHtml(doc) : i < 0 ? WAITING_TILE : presetTileHtml(i, doc)))(presetTiles.get(doc))), tiles = all$(".tile", grid);
  const idOf = (/** @type {Chart} */ doc) => ((i) => (i === undefined || i < 0 ? doc.id : presetIdOf(presetKey(i))))(presetTiles.get(doc));
  const layout = JSON.stringify(parts.map((part) => (part.kind === "shelf" ? [part.kind, JSON.stringify(part.place), part.name, part.count, part.docs.length, ((folder) => (folder ? folderMark(folder) : ""))(folderById(part.folder ?? ""))] : part.kind === "grid" ? [part.kind, Boolean(part.previews), part.docs.length] : [part.kind, part.html])));
  if (layout === drawnLayout && tiles.length === visible.length && tiles.every((tile, i) => tile.dataset.chart === idOf(visible[i]))) {
    tiles.forEach((tile, i) => drawnTiles.get(idOf(visible[i])) !== html[i] && swapTile(tile, html[i]));
  } else {
    const focus = focusKey(grid), old = new Map(tiles.map((tile) => [/** @type {string} */ (tile.dataset.chart), tile]));
    const htmlOf = new Map(visible.map((doc, i) => ((id) => [id, old.has(id) && drawnTiles.get(id) === html[i] ? `<i data-slot="${id}"></i>` : html[i]])(idOf(doc))));
    const tilesOf = (/** @type {Chart[]} */ docs) => docs.map((doc) => htmlOf.get(idOf(doc))).join("");
    grid.innerHTML = parts.map((part) => (part.kind === "grid" ? `<div class="grid-tiles${part.previews ? " previews" : ""}">${tilesOf(part.docs)}</div>`
      : part.kind === "shelf" ? folderShelfHtml(part, tilesOf(part.docs))
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
const SHELF_STEPS = `<button type="button" class="cap small shelf-step" data-shelf-step="-1" aria-label="Scroll back" data-tip="Scroll back">${ICONS.back}</button><button type="button" class="cap small shelf-step" data-shelf-step="1" aria-label="Scroll on" data-tip="Scroll on">${ICONS.next}</button>`;
/** @param {{ name: string, place: Place, folder?: string, count: number }} shelf @param {string} tiles */
const folderShelfHtml = ({ name, place, folder, count }, tiles) => `<section class="shelf"><div class="shelf-head"${folder === undefined ? "" : ` data-grid-folder="${folder}"`}><button type="button" class="shelf-title" data-place="${escapeHtml(JSON.stringify(place))}" aria-label="${escapeHtml(name)}, ${grouped.format(count)}">${((own) => (own ? folderMark(own) : ""))(folder ? folderById(folder) : undefined)}${escapeHtml(name)}<span class="count">${grouped.format(count)}</span>${ICONS.next}</button>${folder ? moreCap("shelf-more", "data-folder-menu", folder, name) : ""}<span class="spacer"></span>${SHELF_STEPS}</div><div class="shelf-row">${tiles ? `${tiles}${seeAll(place, count)}` : `<p class="shelf-empty">No charts in it yet</p>`}</div></section>`;
const cardOf = (id) => /** @type {HTMLElement | null} */ (pressedTile?.isConnected && pressedTile.dataset.chart === id ? pressedTile : $("home").querySelector(`.tile[data-chart="${CSS.escape(id)}"]`) ?? ((slot) => slot && fillSlot(slot))(/** @type {HTMLElement | null} */ ($("home").querySelector(`.tile-slot[data-slot="${presetById(id)}"]`))));
const tileOpen = (id) => /** @type {HTMLElement | null} */ (cardOf(id)?.querySelector(".tile-open") ?? null);

/**
 * Each page's place as it was left, by its step's key. A step left by Back or Forward can't be written any more (its
 * place, in it, only when left otherwise): kept here, it's where it was last.
 */
const leftPlaces = /** @type {Map<string, { top: number, rows: Record<string, number>, shown: number }>} */ (new Map());
const yoursShown = () => home && ("folder" in homeAt || "previews" in homeAt || "recent" in homeAt);
function pageOut() {
  if (reduced()) return;
  for (const animation of /** @type {HTMLElement} */ ($("home").querySelector(".home-inner")).getAnimations()) animation.finish();
  fadeOut(homeScroll(), [{ opacity: 1 }, { opacity: 0 }], MOTION.small, true, MOTION.ease, true);
}
/**
 * Its way in and the page left's way out (see goTo) begin once frames come on time: on a phone the new page's first
 * frames (laid out, its tiles painted) took a tenth of a second or more, the fade all but over when it showed.
 */
function pageIn() {
  if (reduced()) return;
  const into = /** @type {HTMLElement} */ ($("home").querySelector(".home-inner")).animate({ opacity: [0, 1], translate: ["0 6px", "0 0"] }, { duration: MOTION.small, easing: MOTION.ease });
  beginOnTime(into, ...(fading.get(homeScroll())?.getAnimations() ?? []));
}
function renderSideNav() {
  const lit = (/** @type {Element} */ element, /** @type {boolean} */ on) => on !== element.hasAttribute("aria-current") && (on ? element.setAttribute("aria-current", "page") : element.removeAttribute("aria-current"));
  const folder = home && "folder" in homeAt ? homeAt.folder : null, path = home && "path" in homeAt ? JSON.stringify(homeAt.path) : null, at = home ? JSON.stringify(homeAt) : "";
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

/** @param {Place} place @param {number} count */
const seeAll = (place, count) => (count > SHELF ? `<button type="button" class="shelf-all" data-see-all="" data-place="${escapeHtml(JSON.stringify(place))}">See all ${grouped.format(count)}${ICONS.next}</button>` : "");
let recommendedFound = /** @type {{ for: Preset[] | null, at: number[] }} */ ({ for: null, at: [] });
/** @returns {number[]} */
const recommendedAt = () => {
  if (recommendedFound.for !== presets) {
    const wanted = new Map(RECOMMENDED.map((place, k) => [place, k])), at = RECOMMENDED.map(() => -1);
    (presets ?? []).forEach((preset, i) => ((k) => k !== undefined && at[k] < 0 && (at[k] = i))(wanted.get(preset.p.join(" › "))));
    recommendedFound = { for: presets, at };
  }
  return recommendedFound.at;
};
const recommendedPresets = () => recommendedAt().filter((i) => i >= 0);
function presetTileHtml(/** @type {number} */ i, doc = presetDoc(i), card = -1) {
  const title = escapeHtml(titleOf(doc));
  const legend = cardLegend(doc);
  return `<div class="tile preset-tile ${doc.preset !== undefined && presetEdited(doc) ? "edited" : ""}" data-preset="${i}"${card < 0 ? ` data-chart="${presetIdOf(presetKey(i))}"` : ""}><button type="button" class="tile-open" ${card < 0 ? `data-open-preset="${i}"` : `data-template="${card}"`} aria-label="${title}" data-tip="${title}"><span class="tile-head"><span class="tile-name">${title}</span>${tileIndexHtml(doc)}</span><span class="tile-legend">${legend}</span>${plotHtml(doc, true, card < 0 ? "tile-plot" : "template-plot tile-plot")}</button>${moreCap("tile-more", "data-preset-menu", i, titleOf(doc))}</div>`;
}
let listsDrawn = 0;
const listAt = () => listsDrawn++;
/**
 * Its first tile written (every row as tall as it), the others a slot each, written once near sight (see nearTiles): a
 * search typed writes what shows, not sixty.
 * @param {number[]} list @param {number} [total]
 */
const presetGridHtml = (list, total = list.length) => `<div class="grid-tiles" data-list="${listAt()}">${list.slice(0, galleryShown).map((i, k) => (k ? slotHtml(i) : presetTileHtml(i))).join("")}</div>${total > galleryShown ? `<button type="button" class="btn ghost small gallery-more" data-gallery-more>Show ${grouped.format(Math.min(GALLERY_PAGE, total - galleryShown))} more</button>` : ""}`;
const WAITING_TILE = '<div class="tile waiting" aria-hidden="true"><span class="tile-open"><span class="tile-head"><span class="tile-name">&nbsp;</span></span><span class="tile-legend"></span><span class="tile-plot pending" data-seen></span></span></div>';
/**
 * A shelf writes this many tiles as it's drawn; the rest a slot each, written as it comes near (see tilesNear) or a
 * keyboard's focus comes into its row: a page of shelves (the presets' fourteen) writes what can show, not every
 * shelf's twelve.
 */
const SHELF_WRITTEN = 3;
const shelfTilesHtml = (/** @type {number[]} */ list) => list.slice(0, SHELF).map((i, k) => (k < SHELF_WRITTEN ? presetTileHtml(i) : slotHtml(i))).join("");
const slotHtml = (/** @type {number} */ i) => `<div class="tile-slot" data-slot="${i}"></div>`;
/** @param {HTMLElement} slot */
function fillSlot(/** @type {HTMLElement} */ slot) {
  return writeTile(slot, Number(slot.dataset.slot));
}
/** @param {HTMLElement} place */
function writeTile(place, /** @type {number} */ i) {
  const made = document.createElement("template"), focus = focusKey(place);
  made.innerHTML = presetTileHtml(i);
  const tile = /** @type {HTMLElement} */ (made.content.firstElementChild);
  place.replaceWith(tile);
  restoreFocus(tile, focus);
  for (const frame of all$(".pending[data-plot]", tile)) watchTile(frame);
  return tile;
}
/** @param {HTMLElement} row */
const fillRow = (row) => all$(":scope > .tile-slot", row).forEach(fillSlot);
/** @param {string} name @param {number[]} list @param {string[] | null} to */
const shelfHtml = (name, list, to) => `<section class="shelf" data-list="${listAt()}"><div class="shelf-head">${to ? `<button type="button" class="shelf-title" data-place="${escapeHtml(JSON.stringify({ path: to }))}" aria-label="${escapeHtml(name)}, ${grouped.format(list.length)}">${escapeHtml(name)}<span class="count">${grouped.format(list.length)}</span>${ICONS.next}</button>` : `<h2 class="shelf-title">${escapeHtml(name)}</h2>`}<span class="spacer"></span>${SHELF_STEPS}</div><div class="shelf-row">${shelfTilesHtml(list)}${to ? seeAll({ path: to }, list.length) : ""}</div></section>`;
function renderGallery() {
  const field = /** @type {HTMLInputElement} */ ($("home-filter")), words = typedWords(field.value);
  const shown = "path" in homeAt, path = "path" in homeAt ? homeAt.path : [], back = shown && $("gallery").hidden;
  $("gallery").hidden = !shown;
  if (!shown) {
    if ($("gallery").contains(document.activeElement)) /** @type {HTMLElement} */ (document.activeElement).blur();
    if (!galleryPlain) galleryDrawn = gallerySpot = "";
    galleryFound = 0;
    return;
  }
  if (!presets) void loadPresets().catch(() => {});
  const spot = JSON.stringify([path, words, galleryShown]), drawn = JSON.stringify([spot, presets?.length ?? 0, presetsFailed, light, library.presets.filter((doc) => !pristine(doc)).map((doc) => `${doc.preset}@${doc.updated}`)]);
  if (drawn === galleryDrawn && !back) return;
  const again = back && drawn === galleryDrawn, lists = listsDrawn;
  const kept = spot === gallerySpot && !back ? galleryPlace() : null;
  if (words.length && galleryPlain && presets) galleryAside = { drawn: galleryDrawn, lists: listsDrawn, nodes: [...$("gallery").childNodes] };
  if (galleryPlain || !words.length) endGalleryWait();
  galleryPlain = !words.length;
  galleryDrawn = drawn;
  gallerySpot = spot;
  listsDrawn = 0;
  galleryMoves++;
  galleryQueue.clear();
  gallerySeen.clear();
  galleryTilesSeen.disconnect();
  galleryAhead.clear();
  galleryTilesAhead.disconnect();
  galleryFound = 0;
  const focus = document.activeElement?.closest?.("[data-shelf-step]") ? null : focusKey($("gallery"));
  if (!presets) {
    $("gallery").innerHTML = presetsFailed ? `<p class="gallery-note">The presets couldn't be read. <button type="button" class="btn ghost small" data-gallery-retry>Try again</button></p>`
      : `<p class="gallery-note">Loading the presets…</p>`;
    return restoreFocus($("gallery"), focus);
  }
  if (words.length) {
    const { found, count, partial } = presetsMatching(words, galleryShown + 1, path.length ? (i) => path.every((name, depth) => /** @type {Preset[]} */ (presets)[i].p[depth] === name) : null);
    galleryFound = count;
    const elsewhere = count || !path.length ? 0 : presetsMatching(words, 0).count;
    $("gallery").innerHTML = (count ? `${partial ? `<p class="gallery-note">No preset holds every word; these hold the most.</p>` : ""}${presetGridHtml(found, count)}` : `<p class="gallery-note">No presets ${path.length ? `in ${escapeHtml(/** @type {string} */ (placeNames(path).at(-1)))} ` : ""}match “${escapeHtml(field.value.trim())}”${elsewhere ? ` <button type="button" class="btn small" data-search-all>Search all presets<span class="count">${grouped.format(elsewhere)}</span></button>` : ""}</p>`);
  } else if (again) {
    listsDrawn = lists;
    for (const row of all$(".shelf-row", $("gallery"))) if (shelfScrolls.get(row)) (row.scrollLeft = 0), shelfScrolls.set(row, 0);
    freshenTiles();
  } else if (galleryAside?.drawn === drawn) {
    $("gallery").replaceChildren(...galleryAside.nodes);
    listsDrawn = galleryAside.lists;
    galleryAside = null;
    freshenTiles();
  } else {
    galleryAside = null;
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
function freshenTiles() {
  for (const frame of all$(".tile[data-preset] > .tile-open [data-plot]", $("gallery"))) {
    const i = presetOf(frame), doc = presetDoc(i), now = previews.get(doc.id);
    if (frame.dataset.plot !== doc.id) writeTile(/** @type {HTMLElement} */ (frame.closest(".tile")), i);
    else if (!now || now.key !== previewKey(doc)) frame.classList.contains("failed") || frame.classList.add("pending");
    else if (frame.dataset.drawn !== String(drawingId(now))) paint(doc.id, now, true);
  }
  for (const frame of all$(".failed[data-plot]", $("gallery"))) galleryTilesSeen.observe(frame);
}
let galleryDrawn = "", gallerySpot = "", galleryFound = 0;
let galleryPlain = false, galleryAside = /** @type {{ drawn: string, lists: number, nodes: Node[] } | null} */ (null);
/** @type {(value?: unknown) => void} */
let endGalleryWait = () => {};
/**
 * Between a search's grids it isn't ended, so a key typed meanwhile asks for no tile of the grids it passes. Only
 * drawGallery waits, one tile at a time (it's serial): each wait takes the one slot.
 * @param {Promise<unknown>} drawn
 */
const galleryWait = (drawn) => Promise.race([drawn, new Promise((done) => (endGalleryWait = done))]);
/** @param {HTMLElement} row */
function shelfSteps(row) {
  shelfScrolls.set(row, row.scrollLeft);
  fitShelf(row, row.scrollLeft, row.clientWidth, row.scrollWidth, cutsOf(row));
}
function stepShelf(/** @type {HTMLElement} */ row, /** @type {number} */ dir) {
  const tile = /** @type {HTMLElement | null} */ (row.firstElementChild);
  if (!tile) return;
  const gap = parseFloat(getComputedStyle(row).columnGap) || 0, unit = tile.getBoundingClientRect().width + gap, page = Math.max(1, Math.floor((row.clientWidth - 8 + gap) / unit + 0.01));
  row.scrollTo({ left: (Math.round(row.scrollLeft / unit) + dir * page) * unit, behavior: reduced() ? "auto" : "smooth" });
}
/** Once the page's writes this frame are in: one layout, not one a write. */
let shelvesFrame = 0;
const shelvesSoon = () => (shelvesFrame ||= requestAnimationFrame(() => ((shelvesFrame = 0), allShelfSteps())));
/** All read first, then written: one layout, not one a shelf. */
function allShelfSteps() {
  const rows = shelfRows(), sizes = rows.map((row) => ({ at: row.scrollLeft, size: row.clientWidth, full: row.scrollWidth, cut: cutsOf(row) }));
  nearTiles(rows);
  rows.forEach((row, k) => fitShelf(row, sizes[k].at, sizes[k].size, sizes[k].full, sizes[k].cut));
}
/** @type {WeakMap<HTMLElement, IntersectionObserver>} */
const rowsNear = new WeakMap();
/**
 * Shelves two screens' height from sight in the page's own scroller (the window's would see them only once in it,
 * clipped by it), made again as that height changes (a margin's percentage is of its width).
 */
let shelvesNear = /** @type {IntersectionObserver | null} */ (null), nearReach = 0;
/** @type {IntersectionObserver | null} */
let slotsNear = null;
/**
 * In time for a step or a fling, never one shown blank: an observer's first word comes a frame on, so those in sight
 * are written at once.
 * @param {HTMLElement[]} rows
 */
function nearTiles(rows) {
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
      const rows = entries.filter((entry) => entry.isIntersecting && !rowsNear.has(/** @type {HTMLElement} */ (entry.target))).map((entry) => /** @type {HTMLElement} */ (entry.target));
      const sizes = rows.map((row) => ({ at: row.scrollLeft, room: row.clientWidth, full: row.scrollWidth, unit: row.children.length > 1 ? row.children[1].getBoundingClientRect().left - row.children[0].getBoundingClientRect().left : 0, cut: cutsOf(row) }));
      rows.forEach((row, k) => (own.unobserve(row), fitShelf(row, sizes[k].at, sizes[k].room, sizes[k].full, sizes[k].cut), tilesNear(row, sizes[k])));
    }, { root: scroller, rootMargin: `${2 * reach}px 0px` });
  }
  const shelves = /** @type {IntersectionObserver} */ (shelvesNear), slots = /** @type {IntersectionObserver} */ (slotsNear);
  shelves.disconnect();
  for (const row of rows) if (!rowsNear.has(row)) shelves.observe(row);
  const view = scroller.getBoundingClientRect(), held = all$(".grid-tiles > .tile-slot", $("home")), boxes = held.map((slot) => slot.getBoundingClientRect());
  slots.disconnect();
  held.forEach((slot, k) => (boxes[k].bottom > view.top && boxes[k].top < view.bottom ? fillSlot(slot) : slots.observe(slot)));
}
/** @param {HTMLElement} tile @param {string} html */
function swapTile(tile, html) {
  const made = document.createElement("template");
  made.innerHTML = html;
  tile.replaceWith(/** @type {HTMLElement} */ (made.content.firstElementChild));
}
/**
 * An observer's first word comes a frame on, and a page scrolled fast is past a screen's height by then, so the slots
 * two rows' widths from sight are written at once; the others as the row scrolls.
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
 * @param {HTMLElement} row
 * @param {number} at
 * @param {number} size
 * @param {number} full
 * @param {{ start: boolean, end: boolean }} cut
 */
function fitShelf(row, at, size, full, cut) {
  const over = full - size;
  for (const step of all$("[data-shelf-step]", /** @type {HTMLElement} */ (row.closest(".shelf")))) {
    step.hidden = over <= 1;
    step.setAttribute("aria-disabled", String(Number(step.dataset.shelfStep) < 0 ? at <= 1 : at >= over - 1));
  }
  edges(row, at, size, full, cut);
}
const galleryQueue = /** @type {Set<number>} */ (new Set()), gallerySeen = /** @type {Set<Element>} */ (new Set());
const presetOf = (/** @type {Element} */ frame) => Number(/** @type {HTMLElement | null} */ (frame.closest("[data-preset]"))?.dataset.preset);

/** @param {Element} frame */
const watchTile = (frame) => (galleryTilesSeen.observe(frame), galleryTilesAhead.observe(frame));
// At rest: no scroll moving in Home (its own, a shelf's: caught on the way down, as scrollend doesn't bubble) and no
// finger down on it; a pan's pointer is cancelled, not let go: its scroll's end says it's over.
let galleryResting = true;
let galleryScrolling = false;
let galleryMoves = 0;
/**
 * A wheel's ticks each end a scroll, so between them it's at rest; a trip always sees one of them. Its first ask, at
 * rest, is enough (a landing's scroll to a section isn't a wheel's).
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
function startGalleryRest() {
  $("home").addEventListener("scroll", () => (galleryResting = false, galleryScrolling = true, galleryMoves++), { capture: true, passive: true });
  $("home").addEventListener("scrollend", () => ((galleryScrolling = false), galleryRests()), { capture: true });
  // A press keeps the grid from rest till its release; a press cancelled or lost, till the next frame unless a scroll's
    // begun by then, whose end then says.
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
/** @param {Set<number>} tried */
const nextGallery = (tried) => {
  for (const i of galleryQueue) return galleryQueue.delete(i), i;
  if (galleryResting) for (const frame of galleryAhead) if (frame.isConnected && frame.classList.contains("pending") && !tried.has(presetOf(frame))) return presetOf(frame);
  return -1;
};

const picked = new Set();
let pickedLast = "";
/** @param {HTMLInputElement} check @param {boolean} on */
const showPick = (check, on) => ((check.checked = on), (check.dataset.tip = on ? "Deselect" : "Select"));
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
  const rename = /** @type {HTMLButtonElement} */ ($("home-bar").querySelector('[data-batch="rename"]'));
  rename.setAttribute("aria-disabled", String(n !== 1));
  rename.dataset.tip = n === 1 ? "Rename" : "Rename: one chart at a time";
}
function pickChart(id, range = false) {
  hideMenu();
  const ids = shownCharts().map((doc) => doc.id), from = ids.indexOf(pickedLast), to = ids.indexOf(id);
  if (to < 0) return;
  if (range && picked.size && from >= 0 && to >= 0) for (const other of ids.slice(Math.min(from, to), Math.max(from, to) + 1)) picked.add(other);
  else if (picked.has(id)) {
    // One taken off isn't where a run starts: the last one picked that's still there is.
    picked.delete(id);
    if (!picked.has(pickedLast)) pickedLast = [...picked].at(-1) ?? "";
    return renderPicked();
  } else picked.add(id);
  pickedLast = id;
  renderPicked();
}
function clearPicks(refocus = false) {
  picked.clear();
  renderPicked();
  if (refocus) tileOpen(pickedLast)?.focus();
}
/** @param {string} action @param {HTMLElement | null} [button] */
function batch(action, button = null, /** @type {string[] | null} */ ids = null) {
  const docs = library.charts.filter((doc) => (ids ? ids.includes(doc.id) : picked.has(doc.id)));
  if (action === "clear" || !docs.length) return clearPicks(action === "clear");
  if (action === "rename") return void (docs.length === 1 && renameTile(docs[0]));
  if (action === "folder") return folderMenu(/** @type {HTMLElement} */ (button ?? tileOpen(docs[0].id)), docs.map(sideKey));
  if (action === "export") {
    if (docs.length === 1) return downloadChart(docs[0]);
    download(`${docs.length}-charts.json`, chartsFile(docs));
    return toast(`Downloaded ${docs.length} charts, in one file`);
  }
  if (action === "duplicate") {
    const from = docs.findIndex((doc) => doc.id === pickedLast), copies = duplicateCharts(docs).map((doc) => doc.id);
    if (!ids) {
      picked.clear();
      for (const id of copies) if (shownYours.some((doc) => doc.id === id)) picked.add(id);
      const anchor = copies[from];
      pickedLast = anchor && picked.has(anchor) ? anchor : [...picked].at(-1) ?? "";
      renderPicked();
    }
    tileOpen(copies[0])?.focus();
    return;
  }
  if (action === "delete") {
    // The focus then goes to the tile that takes the first one's place; none left, to the toast's Undo (never a field:
        // there ⌘Z would undo its typing, not this).
    const at = shownCharts().findIndex((doc) => docs.includes(doc));
    if (!ids) clearPicks();
    deleteCharts(docs);
    /** @type {HTMLElement | undefined} */ (all$(".tile:not(.preview) .tile-open", $("home-grid"))[at] ?? all$(".tile:not(.preview) .tile-open", $("home-grid")).at(-1) ?? $("toast").querySelector("button") ?? undefined)?.focus();
  }
}
/** @param {Chart[]} docs */
function deleteCharts(docs) {
  if (docs.length === 1) return deleteChart(docs[0]);
  const places = docs.map((doc) => /** @type {[number, Chart]} */ ([library.charts.indexOf(doc), doc])).filter(([at]) => at >= 0).sort((a, b) => a[0] - b[0]);
  for (const [, doc] of places) discard(doc);
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
/** @param {Chart} doc */
function renameTile(doc) {
  const tile = cardOf(doc.id);
  if (!tile) return;
  const name = /** @type {HTMLElement} */ (tile.querySelector(".tile-name"));
  nameInPlace(tile, { className: "tile-rename", value: doc.title, placeholder: derivedTitle(doc), max: 120, label: "Chart title", top: name.getBoundingClientRect().top - tile.getBoundingClientRect().top - 4, apply: (title) => renameById(doc.id, title), back: () => tileOpen(doc.id)?.focus(), find: () => cardOf(doc.id) });
}
/** @param {Chart} doc @param {HTMLElement} row */
function renameRow(doc, row) {
  nameInPlace(row, { className: "row-rename", value: doc.title, placeholder: derivedTitle(doc), max: 120, label: "Chart title", apply: (title) => renameById(doc.id, title), back: () => /** @type {HTMLElement | null} */ ($("chart-list").querySelector(`[data-open-chart="${doc.id}"]`))?.focus(), find: () => $("chart-list").querySelector(`[data-chart-row="${doc.id}"], [data-preview-row="${doc.id}"]`) });
}
let naming = /** @type {{ input: HTMLInputElement, find: () => Element | null, finish: () => void } | null} */ (null);
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
 * @param {HTMLElement} host
 * @param {{ className: string, value: string, placeholder: string, max: number, label: string, top?: number, apply: (name: string) => void, back: () => void, find: () => Element | null, close?: () => void, cancel?: () => void }} options
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
  naming = { input, find, finish: () => finish(true, false) };
  input.addEventListener("keydown", (event) => {
    if (composing(event) || (event.key !== "Enter" && event.key !== "Escape")) return;
    event.preventDefault();
    event.stopPropagation();
    finish(event.key === "Enter", true);
  });
  // The window left (a phone's app switched): it stays as it is till the window's back (see leftOpen).
  const left = () => queueMicrotask(() => input.isConnected && document.activeElement !== input && finish(true, false));
  input.addEventListener("blur", () => leftWith(input, left) || left());
}
function renameById(/** @type {string} */ id, /** @type {string} */ title) {
  const doc = chartById(id);
  if (doc && title !== doc.title) renameChart(doc, title);
}
/** @returns {Action[]} */
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
 * The card grows out of a tile over the page (`opening`) or shrinks back into it, the page under it all along; with no
 * tile it fades. Shown over the tile only near its size (in over the first 8% as it grows: the ease is quick at first;
 * out over the last 30% as it shrinks), the live tile under it meanwhile. Moved and clipped only (the compositor's
 * work), its chart drawing as it goes.
 * @param {boolean} opening @param {() => HTMLElement | null} update
 */
let morphing = /** @type {Animation[]} */ ([]);
/**
 * The library measures its element as it is drawn (its box scaled by the card's motion) and lays out at every pane,
 * price axis or share it changes: the first build of a page that started on the grid lies in that. The card's own size
 * never changed, so nothing measures the chart again: once the motion's over, it's laid out against its box.
 */
function unscaled() {
  const { offsetWidth: width, offsetHeight: height } = engine.element;
  if (Math.abs(engine.width - width) >= 1 || Math.abs(engine.height - height) >= 1) engine.layout();
}
function morph(opening, update) {
  for (const animation of morphing) animation.cancel();
  // Out of sight, no motion: a hidden page draws no frames, so it would never end, and a draw waits on it.
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
  // Over, the chart laid out as it shows, at its own size: measured while it moved, it'd be measured scaled.
  return Promise.all(animations.map((animation) => animation.finished)).then(() => {}, () => {}).finally(() => morphing === animations && ((morphing = []), delete document.body.dataset.morph, unscaled(), (opening || layoutOwed) && relayout()));
}
const onChart = () => {
  const active = document.activeElement, at = active?.closest("#menu, .submenu") ? ownerNow() ?? closedOwner : active;
  return Boolean(at?.closest("#main") && !at.closest("#home"));
};
/** @param {Chart | undefined} doc */
let goes = 0, goingCard = /** @type {HTMLElement | null} */ (null);
function go(doc, push = true) {
  if (!doc) return;
  if (!home) return void (doc !== chart && openChart(doc));
  // Its card pressed first, drawn before the chart's made (a while on a slow phone): the tap answered at once. Two
    // asked for meanwhile, the last one.
  const mine = ++goes;
  goingCard?.classList.remove("opening");
  const card = (goingCard = cardOf(doc.id));
  card?.classList.add("opening");
  requestAnimationFrame(() => setTimeout(() => mine === goes && ((goingCard = null), leaveHome(() => doc !== chart && openChart(doc), card, push))));
}

const dialogOpeners = /** @type {WeakMap<HTMLElement, Element | null>} */ (new WeakMap());
/** @type {WeakMap<HTMLElement, HTMLElement>} */
const dialogScrims = new WeakMap();
/** @param {HTMLElement} dialog */
function showDialog(dialog) {
  if (isOpen(dialog)) return;
  dialogOpeners.set(dialog, document.activeElement);
  dialogsShown.push(dialog);
  holdPage(dialog, true);
  dialogScrims.get(dialog)?.showPopover();
  dialog.showPopover();
}
const closeDialogs = () => all$("dialog:popover-open").forEach((dialog) => dialog.hidePopover());
let leavings = 0;
/**
 * Where the browser keeps it drawn while closed, the style plays its way out; elsewhere (Safari: closed, gone at once),
 * its closed look is played first (`data-leaving`), out of reach, then closed. Opened again meanwhile (arrive), it
 * stays.
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
/** @param {HTMLElement} element */
const arrive = (element) => ((element.inert = false), delete element.dataset.leaving);
/**
 * `pulls`: whether its content pulls it too, as a phone's sheets: a finger on it going down past a tap's move (SLOP),
 * every list it's on at its top (or on what doesn't scroll), takes the sheet, not the list; up, or a list scrolled, it
 * scrolls as ever. The tap a pull ends on isn't one.
 * @param {HTMLElement} element
 * @param {{ grab: (event: PointerEvent | null) => boolean, move: (by: number) => void, release: (by: number, speed: number) => void, back: () => void, pulls: () => boolean }} how
 */
function dragSheet(element, { grab, move, release, back, pulls }) {
  let pull = /** @type {{ y: number, pointer: number, trail: { y: number, at: number }[] } | null} */ (null);
  const letGo = () => (element.classList.remove("pulled"), back());
  const follow = (/** @type {number} */ y) => {
    if (!pull) return;
    const now = performance.now();
    pull.trail = [...pull.trail.filter((point) => now - point.at <= 100), { y, at: now }];
    move(y - pull.y);
  };
  const end = (/** @type {number} */ y) => {
    if (!pull) return;
    const by = y - pull.y, speed = speedOf([...pull.trail, { y, at: performance.now() }], (point) => point.y);
    pull = null;
    letGo();
    release(by, speed);
  };
  element.addEventListener("pointerdown", (event) => {
    if (pull || !grab(event)) return;
    pull = { y: event.clientY, pointer: event.pointerId, trail: [{ y: event.clientY, at: performance.now() }] };
    element.classList.add("pulled");
    move(0);
    // Held by what was pressed, the handle or the head: the click after a drag is its, not the sheet's own, which in ⌘K
        // is its backdrop's and would close it.
    const held = /** @type {Element} */ (event.target), pointer = event.pointerId;
    held.setPointerCapture(pointer);
    const mine = () => pull?.pointer === pointer;
    press(event, { captor: held, move: (moved) => mine() && follow(moved.clientY), end: (release) => mine() && (release ? end(release.clientY) : ((pull = null), letGo())) });
  });
  // A finger's, by touch events: one that turns out a pull stops the list's scroll, as a pointer's can't. Its events go
    // to what it touched, out of the sheet or not: a list drawn anew under it (a search's answer, a layer's data) leaves
    // it out, heard there then.
  let touch = /** @type {{ id: number, x: number, y: number, at: number, target: Element, taken: boolean } | null} */ (null);
  const own = (/** @type {TouchEvent} */ event) => (touch ? [...event.changedTouches].find((point) => point.identifier === touch?.id) : undefined);
  const moved = (/** @type {TouchEvent} */ event) => {
    if (touch?.taken && event.cancelable) event.preventDefault();
    const point = own(event);
    if (!touch || !point) return;
    if (!touch.taken) {
      const dx = point.clientX - touch.x, dy = point.clientY - touch.y;
      if (Math.hypot(dx, dy) <= SLOP) return;
      let top = element.contains(touch.target);
      for (let at = /** @type {Element | null} */ (touch.target); top && at; at = at === element ? null : at.parentElement) top = !scrollsY(at) || at.scrollTop <= 0;
      if (!(event.touches.length === 1 && dy > Math.abs(dx) && top && event.timeStamp - touch.at < 300 && !document.body.classList.contains("carrying") && event.cancelable && grab(null))) return forget();
      touch.taken = true;
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
      forget();
      if (pull?.pointer === -1) (pull = null), letGo();
    }
    forget();
    const point = event.touches[0], target = event.target, box = element.getBoundingClientRect();
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
 * Held at their start till frames come on time: begun in a frame the page's work stalls, WebKit shows one already well
 * on in the first frame it draws (iOS 26.5: a sheet's first 80% at once, or all of it). 300 ms of frames on, they begin
 * anyway (a tab out of sight draws none: they wait for it). One taken over meanwhile is left as it is.
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
/** @param {Element} sheet */
const ownSlides = (sheet) => sheet.getAnimations().filter((animation) => animation instanceof CSSTransition && animation.transitionProperty === "transform");
/**
 * Moves by its transform alone, which the screen runs on its own, the page busy or not (its height, laid out each
 * frame, stalled with it). Taller, it's so at once, slid up from there; shorter, slid down, then so. Never as the
 * screen's own height changes (a keyboard, a turn). It starts at `speed` so a drag goes on as it went; from still,
 * gently: iOS draws a motion's first frame some 40 ms in, and a curve fast at its start loses half its way there.
 * @param {HTMLElement} sheet @param {() => void} change @param {number} [from] @param {number} [speed]
 */
function sizeSheet(sheet, change, from = sheet.getBoundingClientRect().top, speed = 0) {
  slides.get(sheet)?.cancel();
  sheet.style.height = sheet.style.maxHeight = "";
  change();
  const height = sheet.getBoundingClientRect().height, by = document.documentElement.clientHeight - height - from;
  // A transition of its own transform, begun as a drag let go gave it back, gives way: run with the slide, iOS showed
    // neither, the sheet still where it was let go till both were over.
  for (const animation of ownSlides(sheet)) animation.cancel();
  if (reduced() || Math.abs(by) < 1) return;
  const timing = { duration: MOTION.layout, easing: `cubic-bezier(0.4, ${clamp((0.4 * speed * MOTION.layout) / by, 0, 1).toFixed(3)}, 0.2, 1)` };
  if (by > 0) (sheet.style.height = `${height + by}px`), (sheet.style.maxHeight = "none");
  const slide = sheet.animate({ transform: by < 0 ? [`translateY(${-by}px)`, "none"] : ["none", `translateY(${by}px)`] }, timing);
  slides.set(sheet, slide);
  beginOnTime(slide);
  if (by > 0) slide.finished.then(() => (sheet.style.height = sheet.style.maxHeight = ""), () => {});
}
let keysWaiting = /** @type {(() => void) | null} */ (null), keysTimer = 0;
/**
 * As the keyboard comes, iOS's screen stalls (150 ms on iOS 26.5), a motion's start lost to it; or 400 ms on, none
 * having come (a keyboard of keys).
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
 * Where something thrown, let go `at` (px) moving at `speed` (px/ms), would come to rest, slowing as iOS's own lists do
 * (0.998 of its speed kept each millisecond): how a native sheet or window reads a flick, a short fast one as far as a
 * long slow drag.
 */
const restOf = (/** @type {number} */ at, /** @type {number} */ speed) => at + (speed * 0.998) / (1 - 0.998);
/** @template P @param {(P & { at: number })[]} trail @param {(point: P) => number} along */
const speedOf = (trail, along) => {
  const now = performance.now(), last = trail.at(-1), past = trail.find((point) => now - point.at <= 100) ?? last;
  return last && past ? (along(last) - along(past)) / Math.max(1, now - past.at) : 0;
};
const sheetGoes = (/** @type {number} */ by, /** @type {number} */ speed, /** @type {number} */ height) => restOf(by, speed) > height / 3;
const stretched = (/** @type {number} */ past) => (1 - 1 / ((past * 0.55) / innerHeight + 1)) * innerHeight;
const unstretched = (/** @type {number} */ shown) => ((1 / (1 - Math.min(shown, innerHeight - 1) / innerHeight) - 1) * innerHeight) / 0.55;
/**
 * Slid first, once out of sight: closed in Safari it's gone at once (no `overlay` there), so its backdrop fades with
 * the slide. Opened or closed meanwhile (esc, ×, Back, another menu), the slide stops, never closed later. A dialog
 * pressed as it slides stays where it's caught (see sheetTaken).
 * @param {HTMLElement} sheet @param {() => void} close @param {number} from @param {number} speed
 */
function slideAway(sheet, close, from, speed) {
  const later = sheet instanceof HTMLDialogElement;
  slides.get(sheet)?.cancel();
  const away = () => {
    const top = sheet.getBoundingClientRect().top - new DOMMatrix(getComputedStyle(sheet).transform).m42, by = document.documentElement.clientHeight + 16 - from;
    return { frames: [{ transform: `translateY(${from - top}px)` }, { transform: "translateY(calc(100% + 16px))" }], duration: MOTION.layout, easing: `cubic-bezier(0.4, ${clamp((0.4 * Math.max(0, speed) * MOTION.layout) / by, 0, 1).toFixed(3)}, 0.2, 1)` };
  };
  if (sheet === $("menu") && !reduced()) return void hideMenu(away());
  if (!later) close();
  // Typed into, let go of as it goes, the keyboard down with it: iOS draws a field's caret apart, left behind.
  else if (sheet.contains(document.activeElement) && typing(document.activeElement)) /** @type {HTMLElement} */ (document.activeElement).blur();
  // Its own way, back as the drag let go of it or out as it closed, gives way: a transition outranks an animation. The
    // rest of its way out (its display, its visibility, kept till it's done) is held to begin with the slide.
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
  let changed = false;
  const meanwhile = new MutationObserver(() => slide.cancel()), toggled = () => ((changed = true), slide.cancel());
  meanwhile.observe(sheet, { attributes: true, attributeFilter: ["open", "hidden"] });
  meanwhile.observe(document.body, { attributes: true, attributeFilter: ["data-sheet"] });
  sheet.addEventListener("beforetoggle", toggled);
  const unheard = () => (meanwhile.disconnect(), sheet.removeEventListener("beforetoggle", toggled));
  slide.finished.then(() => {
    // Opened or closed in the very task it ended in: its change not heard yet, but there.
    changed ||= meanwhile.takeRecords().length > 0;
    unheard();
    if (later && !changed) close();
    for (const animation of ownSlides(sheet)) animation.finish();
    slide.cancel();
    fade?.cancel();
  }, () => (unheard(), fade?.cancel()));
}
/** @param {HTMLElement} sheet */
function sheetTaken(sheet) {
  const slide = slides.get(sheet), away = slide?.id === "away" && slide.playState !== "finished", shown = sheet.getBoundingClientRect().top;
  if (away) slide.cancel();
  return new DOMMatrix(getComputedStyle(sheet).transform).m42 + (away ? shown - sheet.getBoundingClientRect().top : 0);
}
/**
 * Up past whole, it stretches, less and less. Let go upright, it stops at whole, half or gone, the nearest to where
 * it'd come to rest (restOf), never back against a flick.
 * @param {HTMLElement} sheet
 * @param {{ grab: (event: PointerEvent) => "handle" | "head" | false, whole: () => boolean, size: (whole: boolean) => void, close: () => void, scrim?: HTMLElement, pulls: () => boolean }} how
 */
function heightsSheet(sheet, { grab, whole, size, close, scrim, pulls }) {
  const [SHEET_WHOLE, SHEET_HALF] = ["--sheet-whole", "--sheet-half"].map((name) => Number(getComputedStyle(document.documentElement).getPropertyValue(name)));
  let height = 0, handle = false, shown = 0, held = "", base = 0;
  dragSheet(sheet, {
    pulls,
    grab: (event) => {
      const by = event ? grab(event) : "head";
      if (!by) return false;
      shown = sheet.getBoundingClientRect().top;
      const slide = slides.get(sheet);
      held = "";
      if (slide && slide.id !== "away" && slide.playState !== "finished") {
        slide.cancel();
        held = sheet.style.height = `${document.documentElement.clientHeight - shown}px`;
      }
      base = sheetTaken(sheet);
      (height = sheet.offsetHeight), (handle = by === "handle");
      sheet.style.transition = "none";
      if (scrim) scrim.style.transition = "none";
      return true;
    },
    move: (moved) => {
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
      const by = moved + base, still = Math.abs(moved) < 4;
      const away = () => slideAway(sheet, close, shown, speed);
      if (handle && still && heldUpright.matches) return sizeSheet(sheet, () => size(!whole()), shown);
      if (still) return void (held && sizeSheet(sheet, () => {}, shown));
      if (!heldUpright.matches) return void (sheetGoes(by, speed, height) && away());
      // The stop nearest where it'd come to rest, of those on its way: a flick down never brings it back up.
      const now = document.documentElement.clientHeight - shown, rest = height - restOf(by, speed), stops = [innerHeight * SHEET_WHOLE, innerHeight * SHEET_HALF, 0];
      const onWay = stops.filter((each) => (speed > 0.1 ? each <= now + 1 : speed < -0.1 ? each >= now - 1 : true));
      const near = (onWay.length ? onWay : stops).reduce((best, each) => (Math.abs(each - rest) < Math.abs(best - rest) ? each : best));
      if (near === 0) return away();
      sizeSheet(sheet, () => size(near === stops[0]), shown, speed);
    },
  });
}
/** @param {boolean} whole */
const panelSize = (whole) => document.body.dataset.sheet === "panel" && (whole ? (document.body.dataset.sheetSize = "full") : delete document.body.dataset.sheetSize);
/** @param {HTMLElement} list @param {KeyboardEvent} event @param {HTMLElement[]} cards */
function walkCards(list, event, cards = all$(CARDS, list)) {
  const at = cards.indexOf(/** @type {HTMLElement} */ (document.activeElement));
  if (at < 0 || !["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", "Home", "End"].includes(event.key)) return;
  event.preventDefault();
  if (event.key === "Home") return cards[0].focus();
  if (event.key === "End") return lastCard(list).focus();
  nextCard(cards[at], event.key, list)?.focus();
}
/** @param {HTMLElement} list */
const lastCard = (list) => (((row) => row && fillRow(row))(all$(".shelf-row, .grid-tiles", list).at(-1)), /** @type {HTMLElement} */ (all$(CARDS, list).at(-1)));
const CARDS = ".tile:not(.waiting) > .tile-open, .shelf-all";
/** @param {HTMLElement} from */
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

const marksDrawn = new WeakMap();
/** @param {Chart} doc */
const chartMark = (doc) => {
  const thumb = thumbs.get(doc.id), kept = thumb && marksDrawn.get(thumb);
  if (kept?.light === light) return kept.svg;
  const svg = markOf(thumb ?? NO_MARK);
  if (thumb) marksDrawn.set(thumb, { light, svg });
  return svg;
};

const fadeDef = (/** @type {string} */ id, /** @type {string} */ color, /** @type {number | string} */ top) => `<linearGradient id="${id}" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="${escapeHtml(color)}" stop-opacity="${top}"/><stop offset="1" stop-color="${escapeHtml(color)}" stop-opacity="0"/></linearGradient>`;
const areaUnder = (/** @type {string} */ id, /** @type {string[]} */ points, /** @type {number} */ w, /** @type {number} */ h) => `<polygon points="0,${h} ${points.join(" ")} ${w},${h}" fill="url(#${id})"/>`;
const lineOver = (/** @type {string[]} */ points, /** @type {string} */ stroke, opacity = 1) => `<polyline points="${points.join(" ")}" fill="none" stroke="${escapeHtml(stroke)}"${opacity < 1 ? ` stroke-opacity="${opacity}"` : ""} stroke-width="1.5" stroke-linejoin="round" vector-effect="non-scaling-stroke"/>`;
/** @type {Pick<Thumb, "lines" | "washes" | "dots">} */
const NO_MARK = { lines: [], washes: [], dots: [] };
/** @param {Pick<Thumb, "lines" | "washes" | "dots">} mark */
const markOf = ({ lines, washes, dots }) => {
  const y = (value) => 36 - value * 32, stroke = 'fill="none" stroke-width="1.4" vector-effect="non-scaling-stroke" stroke-linejoin="round" stroke-linecap="round"';
  const polyline = (part, color) => `<polyline points="${part.join(" ")}" stroke="${color}" ${stroke}/>`;
  const paths = lines.map(([raw, ys, kind, under, level, dated]) => {
    level ??= 0;
    const color = escapeHtml(ink(raw)), below = escapeHtml(ink(under ?? raw)), x = (i) => 2 + (i / (ys.length - 1)) * 116, base = y(level);
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
  const wash = washes.map(([k, over, under, bottom, top]) => ((ys) => runs(ys, (i) => 2 + (i / (ys.length - 1)) * 116, y).map((part) => areaOf(part, y(top), escapeHtml(ink(over))) + areaOf(part, y(bottom), escapeHtml(ink(under)))).join(""))(lines[k][1])).join("");
  const dotted = dots.map(([color, across, up]) => `<path d="M${(2 + across * 116).toFixed(1)} ${y(up).toFixed(1)}h0" stroke="${escapeHtml(ink(color))}" stroke-width="2.6" stroke-linecap="round" vector-effect="non-scaling-stroke"/>`).join("");
  return `<svg class="chart-mark" viewBox="0 0 120 40" preserveAspectRatio="none" aria-hidden="true">${wash}${paths || `<path d="M4 30H116" stroke="currentColor" stroke-opacity=".35" stroke-width="1.4" vector-effect="non-scaling-stroke" stroke-dasharray="3 3"/>`}${dotted}</svg>`;
};

function sideList() {
  if (listOwed) writeSideList(home ? null : chart);
  return $("chart-list");
}
let sideMoved = false;
function movingSide(/** @type {() => void} */ change) {
  sideMoved = true;
  try {
    change();
  } finally {
    sideMoved = false;
  }
}
const shortCount = (/** @type {number} */ n) => (n < 1000 ? String(n) : n < 1e6 ? `${Math.floor(n / 1000)}k` : `${Math.floor(n / 1e6)}m`);
/** @param {Group} group */
const groupShut = (group) => !prefs.groupsOpen.includes(group);
const moreCap = (/** @type {string} */ cls, /** @type {string} */ attr, /** @type {string | number} */ id, /** @type {string} */ name) => `<button type="button" class="cap small ${cls}" ${attr}="${escapeHtml(id)}" aria-haspopup="menu" aria-label="More for ${escapeHtml(name)}" data-tip="More">${ICONS.more}</button>`;
const viewCap = (/** @type {Place} */ place, /** @type {string} */ name) => `<button type="button" class="cap small chart-go" data-place="${escapeHtml(JSON.stringify(place))}" aria-label="View ${escapeHtml(name)}" data-tip="View">${ICONS.arrowRight}</button>`;
const leafRow = ({ kind, id, depth, cls = "", mark, label, labelCls = "", name, tip = name }) => `<div class="chart-row ${cls} ${depth ? "in-folder" : ""}" data-${kind}-row="${id}" style="--depth: ${depth}"><button type="button" class="nav-item chart-item" data-open-${kind}="${id}" data-tip="${escapeHtml(tip)}">${mark}<span class="label ${labelCls}">${escapeHtml(label)}</span></button>${moreCap("chart-more", `data-${kind}-menu`, id, name)}</div>`;
const folderRow = ({ kind, id, depth, closed, holds, rail, mark, name, count, place }) => `<div class="folder-row ${kind === "section" ? "section-row" : ""} ${depth ? "in-folder" : ""} ${closed ? "closed" : ""} ${closed && holds ? "holds" : ""}" data-${kind}-row="${escapeHtml(id)}" style="--depth: ${depth}"><button type="button" class="nav-item folder-item" data-${kind}-item="${escapeHtml(id)}" ${rail ? 'aria-haspopup="menu"' : `aria-expanded="${!closed}"`} data-tip="${escapeHtml(name)}">${mark}<span class="label">${escapeHtml(name)}</span><span class="turn" aria-hidden="true">${ICONS.next}</span><span class="folder-count">${shortCount(count)}</span></button>${viewCap(place, name)}${moreCap("chart-more", `data-${kind}-menu`, id, name)}</div>`;
/** @param {Group} group @param {number | null} count @param {boolean} holds @returns {[string, string]} */
const sideHead = (group, count, holds) => {
  const [name, place] = GROUP_OF[group], shut = groupShut(group);
  return [`head:${group}`, `<div class="list-head ${shut ? "closed" : ""} ${shut && holds ? "holds" : ""}" data-group="${group}"><button type="button" class="list-head-name" data-group-item="${group}" data-at='${JSON.stringify(place)}' aria-expanded="${!shut}" aria-label="${count === null ? name : `${name}, ${count}`}" data-tip="${name}"><span class="head-mark">${ICONS[GROUP_MARK[group]]}</span><span class="head-name">${name}</span>${count === null ? "" : `<span class="count">${shortCount(count)}</span>`}<span class="turn" aria-hidden="true">${ICONS.next}</span></button>${viewCap(place, name)}${moreCap("chart-more", "data-group-menu", group, name)}</div>`];
};
/** @param {Group} group @returns {Action[]} */
const groupRows = (group) => [
  { key: "view", icon: ICONS.arrowRight, label: "View", aside: "Its page", run: () => goTo(asPlace(GROUP_OF[group][1])) },
  ((shut) => ({ key: "toggle", icon: shut ? ICONS.folderOpen : ICONS.folder, label: shut ? "Expand" : "Collapse", stay: true, run: () => toggleGroup(group) }))(groupShut(group)),
  ...(group === "library" ? [
    { key: "new-chart", icon: ICONS.plus, label: "New chart", kbd: "N", run: () => newChart() }, { key: "new-folder", icon: ICONS.folderAdd, label: "New folder", run: () => newFolder([], undefined, { side: true }) },
    { key: "import", icon: ICONS.upload, label: "Import charts", run: () => importCharts() },
    ...(anythingKept() ? [{ key: "download-all", icon: ICONS.download, label: "Download all charts", aside: "Folders and all", run: () => downloadAll() }] : []),
  ] : []),
];
/** @param {Group} group */
function toggleGroup(group) {
  const closing = !groupShut(group);
  prefs.groupsOpen = closing ? prefs.groupsOpen.filter((other) => other !== group) : [...prefs.groupsOpen, group];
  savePrefs();
  foldRow(() => /** @type {HTMLElement | null} */ ($("chart-list").querySelector(`[data-group="${group}"]`)), closing);
}

function sectionKeys(/** @type {number} */ i) {
  const path = /** @type {Preset[]} */ (presets)[i].p;
  return path.slice(0, -1).map((_, depth) => JSON.stringify(path.slice(0, depth + 1)));
}

/**
 * Made from the tree after the presets are read, a slice at a time in idle moments (none of it at the start, nor a long
 * task: see searchNow), the rest at once when a search comes first.
 * @typedef {{ shown: string[], texts: string[], initials: string[], titles: { title: string, initials: string, name: string, subject: string }[], rank: Int32Array, order: Int32Array, exact: Map<string, number[]> }} PresetSearch
 * @param {Preset[]} list
 * @param {PresetNode} root
 * @returns {Generator<void, PresetSearch>}
 */
function* searchMaking(list, root) {
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
    const pieces = [preset.t, ...preset.p, ...named[i], ...(own ? [own] : [])].map(searchCase), title = pieces[0];
    shown[i] = pieces.join(" ");
    texts[i] = searchText(shown[i], pieces);
    initials[i] = initialsOf(pieces);
    titles[i] = { title: searchText(title), initials: initials[i].split(" ", 1)[0], name: searchCase(presetName(preset)), subject: title.split(": ")[0] };
    if (i % 250 === 249) yield;
  }
  // Counted into place, not sorted: no comparison called thousands of times, as a phone's engine without a JIT would.
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
const presetRankOf = (/** @type {number} */ depth, /** @type {string} */ title) => depth * 1000 + Math.min(title.length, 999);
function searchStep() {
  presetTree();
  if (presetRoot.search) return true;
  const step = (presetRoot.making ??= searchMaking(/** @type {Preset[]} */ (presets), /** @type {PresetNode} */ (presetRoot.node))).next();
  if (step.done) (presetRoot.search = step.value), (presetRoot.making = null);
  return Boolean(step.done);
}
function presetSearch() {
  while (!searchStep());
  return /** @type {PresetSearch} */ (presetRoot.search);
}
function searchNow(/** @type {{ timeRemaining: () => number } | undefined} */ deadline) {
  inSlices(deadline, searchStep, searchNow);
  if (presetRoot.search) whenIdle(vocabNow);
}

const KEPT_PRESET_MARKS = 200;
/** @type {WeakMap<Preset, string>} */
const presetTags = new WeakMap();
const keptMarkKey = (/** @type {Chart} */ doc) => drawingKey([doc.id, doc.updated]);
const presetMarkKey = (/** @type {number} */ i) => ((doc) => (doc ? keptMarkKey(doc) : ((preset) => drawingKey([presetKey(i), presetTags.get(preset) ?? (presetTags.set(preset, hashOf(JSON.stringify(preset))), presetTags.get(preset))]))(/** @type {Preset[]} */ (presets)[i])))(keptPreset(i));
const presetMark = (/** @type {number} */ i) => markOf(presetMarks.get(presetMarkKey(i)) ?? NO_MARK).replace("<svg ", `<svg data-preset-mark="${i}" `);
const presetMarkQueue = /** @type {Set<number>} */ (new Set());
const paletteMarksAsked = /** @type {Set<number>} */ (new Set());
const forgetPaletteMarks = () => (paletteMarksSeen.disconnect(), paletteMarksAsked.forEach((i) => presetMarksAsked.delete(i)), paletteMarksAsked.clear());
/** @param {ParentNode} rows */
const seePaletteMarks = (rows) => {
  for (const mark of rows.querySelectorAll("[data-preset-mark]")) {
    const i = Number(/** @type {HTMLElement} */ (mark).dataset.presetMark);
    if (!presetMarks.has(presetMarkKey(i))) paletteMarksSeen.observe(mark);
    else if (!mark.classList.contains("chart-mark")) mark.outerHTML = presetMark(i);
  }
};
const presetMarksAsked = /** @type {Set<number>} */ (new Set());
/**
 * The sections the open preset is in are opened in the sidebar while it's the one open (less any closed there): shown,
 * never kept (`presetsOpen`).
 */
let revealed = { at: -1, keys: /** @type {Set<string>} */ (new Set()) };
const sectionShown = (/** @type {string} */ key) => prefs.presetsOpen.includes(key) || revealed.keys.has(key);
/** @returns {[string, string][]} */
function presetParts() {
  const root = presetTree();
  // Not read yet: as they were last drawn, with the same sections open, from the first frame, not come in after the
    // rest.
  if (!root || !presets) return sidePresets && !presetsFailed && !onRail() && !groupShut("presets") && sidePresets.open === JSON.stringify(prefs.presetsOpen) ? sidePresets.parts.map(([key, html]) => [key, html.replace(/^<div /, "<div inert ")]) : [sideHead("presets", null, false)];
  const at = !home && chart.preset !== undefined ? presetIndex(chart) : -1, holding = new Set(at < 0 ? [] : sectionKeys(at));
  if (at !== revealed.at) revealed = { at, keys: new Set(holding) };
  const rail = onRail(), open = new Set(rail ? [] : [...prefs.presetsOpen, ...revealed.keys]);
  /** @type {[string, string][]} */
  const parts = [sideHead("presets", presets.length, at >= 0)];
  if (groupShut("presets")) return parts;
  const walk = (/** @type {PresetNode} */ node, /** @type {number} */ depth) => {
    for (const item of node.items) {
      if (typeof item === "number") {
        if (!depth) continue;
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
  // Kept as drawn only with what's kept open: a reveal restored in a first frame would be one of another view's.
  if (!rail && [...revealed.keys].every((key) => prefs.presetsOpen.includes(key))) keepSidePresets(parts);
  return parts;
}

/** @param {Chart | Folder} item */
const sideKey = (item) => ("layers" in item ? `c:${item.id}` : `f:${item.id}`);
/** @param {Chart | Folder} item */
const parentOf = (item) => ("layers" in item ? item.folder : item.parent);
/** @param {Chart | Folder} item @param {string | undefined} parent */
const setParent = (item, parent) => {
  if ("layers" in item) parent ? (item.folder = parent) : delete item.folder;
  else parent ? (item.parent = parent) : delete item.parent;
};
const itemOf = (/** @type {string} */ key) => (key[0] === "f" ? folderById(key.slice(2)) : chartById(key.slice(2)));
/** @returns {(Chart | Folder)[]} */
const itemsOf = (/** @type {string[]} */ keys) => keys.flatMap((key) => itemOf(key) ?? []);
/** @returns {(Chart | Folder)[]} */
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
/** @param {(Chart | Folder)[]} items */
const setChildren = (items) => ((keys) => (library.order = [...library.order.filter((key) => !keys.includes(key)), ...keys]))(items.map(sideKey));
/** @returns {{ item: Chart | Folder, depth: number }[]} */
function treeOf(parent = /** @type {string | undefined} */ (undefined), open = false, depth = 0, /** @type {{ item: Chart | Folder, depth: number }[]} */ out = []) {
  for (const item of childrenOf(parent)) {
    out.push({ item, depth });
    if (!("layers" in item) && !(open && item.closed)) treeOf(item.id, open, depth + 1, out);
  }
  return out;
}

const isIn = (/** @type {string | undefined} */ id, /** @type {string} */ outer) => {
  for (let at = id, steps = 0; at && steps <= library.folders.length; at = folderById(at)?.parent, steps++) if (at === outer) return true;
  return false;
};
function chartCounts() {
  const byId = new Map(library.folders.map((folder) => [folder.id, folder])), counts = /** @type {Map<string, number>} */ (new Map());
  for (const doc of library.charts) for (let at = doc.folder, steps = 0; at && steps <= byId.size; at = byId.get(at)?.parent, steps++) counts.set(at, (counts.get(at) ?? 0) + 1);
  return counts;
}
const folderPath = (/** @type {string} */ id) => {
  const names = [];
  for (let at = /** @type {string | undefined} */ (id), steps = 0; at && steps <= library.folders.length; at = folderById(at)?.parent, steps++) names.unshift(folderById(at)?.name ?? "");
  return names.join(" › ");
};
/** @returns {HTMLElement[]} */
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
const sideTree = () => (onRail() ? childrenOf(undefined).map((item) => ({ item, depth: 0 })) : treeOf(undefined, true));
function newFolder(/** @type {string[]} */ keys = [], /** @type {string | undefined} */ parent = undefined, { step = true, name = true, side = false } = {}) {
  const was = arrangement();
  const folder = /** @type {Folder} */ ({ id: uid(), name: "New folder", ...(parent && folderById(parent) ? { parent } : {}) });
  if (home) $("home-filter").value = "";
  library.folders.push(folder);
  setChildren([folder, ...childrenOf(folder.parent).filter((item) => item !== folder)]);
  if (groupShut("library")) (prefs.groupsOpen = [...prefs.groupsOpen, "library"]), savePrefs();
  for (let at = folder.parent; at; at = folderById(at)?.parent) delete /** @type {Folder} */ (folderById(at)).closed;
  moveToFolder(keys, folder.id, true);
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
  if (name) renameFolder(folder.id, side, true);
  return folder;
}
function toggleFolder(/** @type {string} */ id) {
  const folder = folderById(id);
  if (!folder) return;
  const closing = !folder.closed;
  if (closing) folder.closed = true;
  else delete folder.closed;
  save();
  foldRow(() => /** @type {HTMLElement | null} */ ($("chart-list").querySelector(`[data-folder-row="${id}"]`)), closing);
}
/** @returns {Action[]} */
const sectionRows = (/** @type {string} */ key) => [
  { key: "view", icon: ICONS.arrowRight, label: "View", aside: "Its page", run: () => goTo(asPlace({ path: JSON.parse(key) })) },
  ((open) => ({ key: "toggle", icon: open ? ICONS.folder : ICONS.folderOpen, label: open ? "Collapse" : "Expand", stay: true, run: () => toggleSection(key) }))(sectionShown(key)),
];
/** @param {string} key */
function toggleSection(key) {
  const closing = sectionShown(key);
  revealed.keys.delete(key);
  prefs.presetsOpen = closing ? prefs.presetsOpen.filter((other) => other !== key) : [...prefs.presetsOpen, key];
  savePrefs();
  foldRow(() => all$("[data-section-row]", $("chart-list")).find((row) => row.dataset.sectionRow === key) ?? null, closing);
}
function foldRow(/** @type {() => HTMLElement | null} */ find, /** @type {boolean} */ closing) {
  let head = find();
  if (reduced() || !head || onRail()) return renderCharts();
  const glyph = head.querySelector(".folder-glyph");
  if (glyph) glyph.innerHTML = closing ? ICONS.folder : ICONS.folderOpen;
  head.classList.toggle("closed", closing);
  head.querySelector("[aria-expanded]")?.setAttribute("aria-expanded", String(!closing));
  const rowKey = (/** @type {HTMLElement} */ row) => row.dataset.chartRow ?? row.dataset.folderRow ?? row.dataset.sectionRow ?? row.dataset.presetRow ?? row.dataset.previewRow ?? "";
  const shown = new Map(closing ? [] : rowsUnder(head).filter((row) => row.getAnimations().length).map((row) => [rowKey(row), row.getBoundingClientRect().height]));
  if (!closing) {
    movingSide(renderCharts);
    head = find();
    if (!head) return;
  }
  const folds = foldRows(rowsUnder(head), closing, (row) => shown.get(rowKey(row)));
  let sliding = true;
  Promise.allSettled(folds.map((animation) => animation.finished)).then(() => {
    sliding = false;
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
    movingSide(renderCharts);
  });
}
/** @param {"system" | "light" | "dark"} theme */
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
function themeMenu(/** @type {HTMLElement} */ anchor) {
  menuOf(anchor, THEMES.map(([id, name]) => ({ key: id, label: name, current: prefs.theme === id, run: () => setTheme(/** @type {"system" | "light" | "dark"} */ (id)) })));
}
function rowsUnder(/** @type {HTMLElement} */ head) {
  if (head.classList.contains("list-head")) {
    const under = [];
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
/**
 * Folders closing now: the list's drawn without their charts once the last is done (one before would cut the others
 * short).
 */
let closings = 0;

/**
 * One edge moving down from the one above (or back up to it) on the layout's curve: each row opened (its gap, then
 * itself) as the edge passes it, its words in place, fading in over its own height; what's below moves with the edge.
 * One under way turns back from where its edge is. All measured first, then all moved (a long folder: one layout, not
 * one a row).
 * @param {HTMLElement[]} elements @param {boolean} closing @param {(row: HTMLElement) => number | undefined} was
 */
function foldRows(elements, closing, was = () => undefined) {
  const gaps = new Map(), gapOf = (/** @type {HTMLElement} */ element) => ((list) => gaps.get(list) ?? (gaps.set(list, parseFloat(getComputedStyle(list).rowGap) || 0), gaps.get(list)))(/** @type {HTMLElement} */ (element.parentElement));
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
  // As far as it shows: past the list's visible end, rows open at the last frame (out of sight by then) and fold at the
    // first; the edge's time all spent in sight, so a long group's isn't run through in a blink.
  const list = elements[0]?.parentElement, room = list ? Math.max(0, list.getBoundingClientRect().bottom - first) : 0;
  const past = (/** @type {{ top: number, at: number }} */ row) => row.top >= room && row.at >= room, near = rows.filter((row) => !past(row));
  const reach = Math.min(y, Math.max(room, ...near.map((row) => row.top + row.gap + row.full))), steps = Math.max(2, Math.ceil(MOTION.layout / 16));
  const start = closing ? Math.max(0, ...near.map((row) => (row.now > 0 ? row.top + row.now : 0))) : Math.min(reach, ...near.map((row) => (row.now < row.gap + row.full ? row.top + row.now : reach)));
  const edges = Array.from({ length: steps + 1 }, (_, j) => start + ((closing ? 0 : reach) - start) * layoutCurve(j / steps));
  return rows.map((row) => {
    const { element, gap, full, top, now } = row;
    element.inert = closing;
    const frames = edges.map((edge, j) => {
      const passed = Math.min(Math.max(edge - top, 0), gap + full), shown = past(row) ? (j === 0 ? now : j === steps && !closing ? gap + full : 0) : closing ? Math.min(now, passed) : Math.max(now, passed), height = Math.max(shown - gap, 0);
      return { offset: j / steps, height: `${height}px`, marginTop: `${Math.min(shown, gap) - gap}px`, opacity: full ? height / full : 1, overflow: "clip", overflowClipMargin: "1px", alignItems: "flex-start" };
    });
    const animation = element.animate(frames, { duration: MOTION.layout, easing: "linear", fill: closing ? "forwards" : "none" });
    foldOf.set(element, animation);
    const still = () => foldOf.get(element) === animation && foldOf.delete(element);
    if (!closing) animation.finished.then(still, still);
    return animation;
  });
}
function renameFolder(/** @type {string} */ id, inSide = false, first = false) {
  const folder = folderById(id);
  if (!folder) return;
  if (inSide && onRail()) {
    let row = null;
    for (let up = /** @type {Folder | undefined} */ (folder); up && !row; up = up.parent ? folderById(up.parent) : undefined) row = /** @type {HTMLElement | null} */ (document.querySelector(`#chart-list [data-folder-item="${up.id}"]`));
    if (row) return nameBeside(row, id, first);
  }
  const side = !phone.matches && !railed(), own = home && "folder" in homeAt && homeAt.folder === id && !(inSide && side);
  const at = own ? ".home-head" : inSide && side ? `#chart-list [data-folder-row="${id}"]` : home && $("home-grid").querySelector(`[data-grid-folder="${id}"]`) ? `#home-grid [data-grid-folder="${id}"]` : side ? `#chart-list [data-folder-row="${id}"]` : "";
  const host = /** @type {HTMLElement | null} */ (at ? document.querySelector(at) : null);
  if (!host && onRail() && !inSide) return renameFolder(id, true, first);
  if (!host) return openPage(renameFolderPage(id, first));
  const back = own ? "#home-title-more" : at.startsWith("#home-grid") ? `${at} .shelf-title` : `#chart-list [data-folder-item="${id}"]`;
  nameInPlace(host, { className: "folder-rename", value: folder.name, placeholder: "Folder", max: 60, label: "Folder name", apply: (name) => nameFolder(id, name, first), back: () => /** @type {HTMLElement | null} */ (document.querySelector(back))?.focus(), find: () => document.querySelector(at), cancel: () => first && unmakeFolder(id) });
}
function nameBeside(/** @type {HTMLElement} */ anchor, /** @type {string} */ id, first = false) {
  const folder = folderById(id);
  if (!folder) return;
  const box = document.createElement("div"), stop = new AbortController();
  box.className = "name-beside";
  box.innerHTML = `<p class="menu-group">${first ? "New folder" : "Rename folder"}</p><div class="name-beside-field">${folderMark(folder)}</div><p class="name-beside-keys"><span><kbd>↵</kbd> ${first ? "Name" : "Rename"}</span><span><kbd>esc</kbd> Cancel</span></p>`;
  document.body.append(box);
  const field = /** @type {HTMLElement} */ (box.querySelector(".name-beside-field"));
  nameInPlace(field, { className: "name-beside-input", value: folder.name, placeholder: "Folder", max: 60, label: "Folder name", apply: (name) => nameFolder(id, name, first), back: () => anchor.isConnected && anchor.focus({ preventScroll: true }), find: () => (box.isConnected ? field : null), close: () => (fadeOut(box, [{ opacity: 1, transform: "none" }, { opacity: 0, transform: "scale(0.96)" }]), box.remove(), stop.abort()), cancel: () => first && unmakeFolder(id) });
  const input = /** @type {HTMLInputElement} */ (field.querySelector("input"));
  input.addEventListener("input", () => ((mark) => mark && (mark.outerHTML = folderMark({ ...(folderById(id) ?? folder), name: input.value.trim() || folder.name })))(field.querySelector(".folder-mark")));
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
const unmakeFolder = (/** @type {string} */ id) => libraryEdits.past.at(-1)?.folder === id && undoLibrary();
const renameFolderPage = (/** @type {string} */ id, first = false) => inputPage({ title: first ? "New folder" : "Rename", value: folderById(id)?.name ?? "", placeholder: "The folder's name", hint: "A folder needs a name", verb: first ? "Name" : "Rename", read: (text) => (text.trim() ? `${first ? "Name it" : "Rename to"} “${text.trim().slice(0, 60)}”` : null), apply: (text) => nameFolder(id, text.trim().slice(0, 60), first), cancel: first ? () => unmakeFolder(id) : undefined });
function setFolderIcon(/** @type {string} */ id, /** @type {string | undefined} */ icon) {
  const folder = folderById(id), was = folder?.icon;
  if (!folder || was === icon) return;
  libraryStep(`Changed “${folder.name}”'s icon`, () => ((now) => folderById(now)?.icon === icon && (setFolderIcon(now, was), () => folderById(folderNow(id))?.icon === was && setFolderIcon(folderNow(id), icon)))(folderNow(id)), "Icon put back");
  if (icon === undefined) delete folder.icon;
  else folder.icon = icon;
  save();
  renderCharts();
}
function nameFolder(/** @type {string} */ id, /** @type {string} */ name, first = false) {
  const folder = folderById(id), was = folder?.name;
  if (!folder || !name || name === was) return;
  // A new folder's first name (`first`: the field New folder opened), while its making is the last step: its making's,
    // undone and made again with it; once. Undone, only if it's still the name it was given.
  const making = libraryEdits.past.at(-1);
  if (first && !redoing && making?.folder && folderNow(making.folder) === id) delete making.folder;
  else libraryStep(`Renamed “${was}”`, () => ((now) => folderById(now)?.name === name && (nameFolder(now, /** @type {string} */ (was)), () => folderById(folderNow(id))?.name === was && nameFolder(folderNow(id), name)))(folderNow(id)), "Name put back");
  folder.name = name;
  save();
  renderCharts();
}
function stepFolder(/** @type {string} */ id, /** @type {number} */ by) {
  const folder = folderById(id);
  if (folder) stepSide(folder, by);
}
/** @param {Chart | Folder} item */
function stepSide(item, by) {
  const items = childrenOf(parentOf(item)), at = items.indexOf(item), to = at + by;
  if (at < 0 || to < 0 || to >= items.length) return;
  [items[at], items[to]] = [items[to], items[at]];
  arrange(() => setChildren(items), sideKey(item));
}
function arrange(/** @type {() => void} */ edit, /** @type {string} */ moved, /** @type {number | null} */ dropY = null, /** @type {string[]} */ all = [moved]) {
  const rows = () => all$("[data-chart-row], [data-folder-row]", $("chart-list")), was = placesOf(rows(), sideId);
  const row = rows().find((item) => sideId(item) === moved);
  if (dropY !== null && row) was.set(moved, dropY - row.offsetHeight / 2);
  const before = arrangement();
  edit();
  const after = arrangement();
  if (before.order.join() !== after.order.join() && [...before.folders].every(([key, folder]) => after.folders.get(key) === folder)) reorderStep(all, before, after);
  save();
  movingSide(renderCharts);
  slide(rows(), sideId, was, { moved });
}
function reorderStep(/** @type {string[]} */ moved, /** @type {ReturnType<typeof arrangement>} */ before, /** @type {ReturnType<typeof arrangement>} */ after) {
  const item = itemOf(moved[0]);
  libraryStep(moved.length > 1 ? `Moved ${moved.length} items` : `Moved “${!item ? "it" : "layers" in item ? titleOf(item) : item.name}”`, () => {
    if (!itemsOf(keysNow(moved)).length) return;
    placeAs(before, itemsOf(keysNow(moved)));
    return () => void (placeAs(after, itemsOf(keysNow(moved))) && reorderStep(keysNow(moved), before, after));
  }, "Order put back");
}
/** @template T @param {T[]} list @param {T} item @param {(T | undefined)[]} before */
function placedAfter(list, item, before) {
  const after = before.findLast((other) => other !== undefined && list.includes(other));
  const at = after === undefined ? 0 : list.indexOf(after) + 1;
  return [...list.slice(0, at), item, ...list.slice(at)];
}
const arrangement = () => ({ order: treeOf().map(({ item }) => sideKey(item)), folders: new Map([...library.charts, ...library.folders].map((item) => [sideKey(item), parentOf(item)])) });
/** @param {(Chart | Folder)[]} items */
function folderToast(was, items) {
  const moved = items.filter((item) => was.folders.get(sideKey(item)) !== parentOf(item));
  if (!moved.length) return;
  const what = moved.length === 1 ? `“${"layers" in moved[0] ? titleOf(moved[0]) : moved[0].name}”` : `${moved.length} ${moved.every((item) => "layers" in item) ? "charts" : "items"}`;
  const to = new Set(moved.map(parentOf)), from = new Set(moved.map((item) => was.folders.get(sideKey(item)))), into = [...to][0], left = [...from][0];
  const named = (/** @type {string | undefined} */ id) => `“${folderById(id)?.name ?? ""}”`;
  toast(to.size === 1 && into ? `Moved ${what} to ${named(into)}` : to.size === 1 && from.size === 1 && left ? `Moved ${what} out of ${named(left)}` : `Moved ${what}`, { label: "Undo", run: () => {
    const after = arrangement();
    placeAs(was, itemsOf(keysNow(moved.map(sideKey))));
    return () => void (placeAs(after, itemsOf(keysNow(moved.map(sideKey)))) && folderToast(arrangedNowOf(was), itemsOf(keysNow(moved.map(sideKey)))));
  }, undone: "Moved back" });
}
/** @param {ReturnType<typeof arrangement>} was */
const arrangedNowOf = (was) => ({ order: keysNow(was.order), folders: new Map([...was.folders].map(([key, folder]) => [keysNow([key])[0], folder && folderNow(folder)])) });
/** @param {ReturnType<typeof arrangement>} was @param {(Chart | Folder)[]} items @returns {boolean} */
function placeAs(was, items) {
  const tree = () => JSON.stringify(treeOf().map(({ item, depth }) => [sideKey(item), depth])), was0 = tree(), order0 = [...library.order];
  const { order, folders } = arrangedNowOf(was);
  for (const old of items) {
    const key = sideKey(old), now = itemOf(key);
    if (!now) continue;
    const folder = folders.get(key);
    if (folder && !folderById(folder)) continue;
    const into = folder && ("layers" in now || !isIn(folder, now.id)) ? folder : undefined;
    setParent(now, into);
    const at = order.lastIndexOf(key), before = order.slice(0, Math.max(0, at)).map(itemOf);
    setChildren(placedAfter(childrenOf(into).filter((item) => item !== now), now, before));
  }
  // Saved only if it moved anything (the sidebar's tree, not how its order's written): an undo that moved nothing did
    // nothing, see undoLibrary.
  const moved = tree() !== was0;
  if (moved) save();
  else library.order = order0;
  renderCharts();
  return moved;
}
function placeItems(/** @type {string[]} */ keys, /** @type {string | undefined} */ into, /** @type {string | null} */ before = null, { quiet = false, dropY = /** @type {number | null} */ (null) } = {}) {
  if (into && !folderById(into)) return void notice("That folder was deleted meanwhile");
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
function moveToFolder(/** @type {string[]} */ keys, /** @type {string | null} */ id, quiet = false) {
  const moving = itemsOf(keys).filter((item) => !("layers" in item && !isYours(item)) && (id ? parentOf(item) !== id : parentOf(item)));
  if (!moving.length) return;
  const hadFocus = Boolean(document.activeElement?.closest("#chart-list, #menu, .submenu"));
  if (id) placeItems(moving.map(sideKey), id, null, { quiet });
  else {
    // By the folder each was in before any moved: one moved up into another's isn't moved again.
    const was = arrangement(), from = new Map(moving.map((item) => [item, parentOf(item)]));
    for (const folder of new Set(from.values())) {
      const out = folderById(folder), siblings = out ? childrenOf(out.parent) : [], next = out ? siblings[siblings.indexOf(out) + 1] : undefined;
      placeItems(moving.filter((item) => from.get(item) === folder).map(sideKey), out?.parent, next ? sideKey(next) : null, { quiet: true });
    }
    if (!quiet) folderToast(was, moving);
  }
  let shown = /** @type {string | undefined} */ (undefined);
  for (let at = id ?? undefined, steps = 0; at && steps <= library.folders.length; at = folderById(at)?.parent, steps++) if (folderById(at)?.closed) shown = at;
  if (hadFocus && shown) refocus(`#chart-list [data-folder-item="${CSS.escape(shown)}"]`);
}
const chartKeys = (/** @type {string[]} */ ids) => ids.map((id) => `c:${id}`);
/** @param {string[]} keys */
function outOf(keys) {
  const from = new Set(itemsOf(keys).map(parentOf).filter(Boolean));
  return !from.size ? "" : from.size === 1 ? `Move out of “${folderById([...from][0])?.name ?? ""}”` : "Move out of their folders";
}
/** @param {HTMLElement} anchor @param {string[]} keys */
function folderMenu(anchor, keys) {
  menuOf(anchor, folderChoices(keys), undefined, "", { title: phone.matches ? "Move to folder" : "" });
}
/** @param {string[]} keys @returns {Action[]} */
function folderChoices(keys, out = true) {
  const items = itemsOf(keys), from = new Set(items.map(parentOf)), moved = items.filter((item) => !("layers" in item)).map((folder) => folder.id);
  const places = treeOf().flatMap(({ item }) => (!("layers" in item) && !moved.some((id) => isIn(item.id, id)) ? [item] : [])), counts = chartCounts();
  return [
    ...places.map((folder) => ({ key: folder.id, icon: folderMark(folder), label: folderPath(folder.id), aside: plural(counts.get(folder.id) ?? 0, "chart"), current: from.size === 1 && from.has(folder.id), run: () => moveToFolder(keys, folder.id) })),
    { key: "new", icon: ICONS.folderAdd, label: "New folder", apart: true, run: () => (isOpen($("palette")) && dismissPalette(), newFolder(keys, from.size === 1 ? [...from][0] : undefined)) },
    ...(out && outOf(keys) ? [{ key: "out", icon: ICONS.folderOut, label: outOf(keys), run: () => moveToFolder(keys, null) }] : []),
  ];
}
const folderActions = (/** @type {HTMLElement} */ anchor, /** @type {string} */ id, /** @type {{ x: number, y: number } | undefined} */ point = undefined) => menuOf(anchor, folderRows(id, false, Boolean(anchor.closest("#chart-list"))), point ?? null);
/** @returns {Action[]} */
function folderRows(/** @type {string} */ id, inPalette = false, inSide = false) {
  const folder = folderById(id);
  if (!folder) return [];
  const peers = childrenOf(folder.parent), at = peers.indexOf(folder), leave = outOf([`f:${id}`]);
  const view = { key: "view", icon: ICONS.arrowRight, label: "View", aside: "Its page", run: () => goTo({ folder: id }) };
  const toggle = { key: "toggle", icon: folder.closed ? ICONS.folderOpen : ICONS.folder, label: folder.closed ? "Expand" : "Collapse", ...(inPalette ? { aside: folder.closed ? "What's in it shown" : "What's in it out of sight" } : {}), verb: folder.closed ? "Expand" : "Collapse", stay: true, run: () => toggleFolder(id) };
  const rename = { key: "rename", icon: ICONS.pencil, label: "Rename", ...(inPalette ? { aside: folder.name, page: () => renameFolderPage(id) } : { run: () => renameFolder(id, inSide) }) };
  const now = () => folderById(id) ?? folder;
  const icon = { key: "icon", icon: folderMark(folder), label: "Icon", aside: folder.icon ? capitalized(folder.icon) : "Letter", page: () => choicePage("Icon", () => [
    { key: "letter", icon: folderMark({ ...now(), icon: undefined }), label: "Letter", aside: letterOf(now().name), current: !now().icon, run: () => setFolderIcon(id, undefined) },
    ...FOLDER_ICONS.map((name) => ({ key: name, icon: folderMark({ ...now(), icon: name }), label: capitalized(name), current: now().icon === name, run: () => setFolderIcon(id, name) })),
  ]) };
  return [
    ...(inPalette ? [view, toggle, rename] : [view, rename, toggle]),
    icon,
    ...(inPalette && at > 0 ? [{ key: "up", icon: ICONS.arrowUp, label: "Move up", kbd: "⌥↑", verb: "Move", stay: true, run: () => stepFolder(id, -1) }] : []),
    ...(inPalette && at < peers.length - 1 ? [{ key: "down", icon: ICONS.arrowDown, label: "Move down", kbd: "⌥↓", verb: "Move", stay: true, run: () => stepFolder(id, 1) }] : []),
    { key: "new-chart", icon: ICONS.plus, label: "New chart inside", run: () => (isOpen($("palette")) && dismissPalette(), newChart(id)) },
    { key: "new-folder", icon: ICONS.folderAdd, label: "New folder inside", run: () => (isOpen($("palette")) && dismissPalette(), newFolder([], id, { side: inSide })) },
    { key: "folder", icon: ICONS.folder, label: "Move to folder", ...(inPalette ? { aside: folder.parent ? folderPath(folder.parent) : "None" } : {}), page: () => choicePage("Move to folder", () => folderChoices([`f:${id}`], inPalette)) },
    ...(leave ? [{ key: "out", icon: ICONS.folderOut, label: leave, run: () => moveToFolder([`f:${id}`], null) }] : []),
    { key: "import", icon: ICONS.upload, label: "Import charts", aside: "Into it", run: () => (isOpen($("palette")) && dismissPalette(), importCharts(id)) },
    ...(library.charts.some((doc) => isIn(doc.folder, id)) || library.folders.some((other) => other.id !== id && isIn(other.id, id)) ? [{ key: "download", icon: ICONS.download, label: "Download all charts", aside: "Its folders too", run: () => downloadFolder(id) }] : []),
    { key: "delete", icon: ICONS.trash, label: "Delete the folder", aside: "Keeps its charts", verb: "Delete", danger: true, run: () => deleteFolder(id) },
  ];
}
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
/** @param {Chart[]} docs @param {Chart[] | null} [again] */
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
    for (const doc of library.charts.filter((doc) => ids.includes(doc.id))) discard(doc);
    if (open >= 0) openChart(chartById(docs[open].id) ?? previousChart() ?? library.charts[0] ?? newDraft());
    save();
    renderCharts();
    return () => {
      const left = docs.flatMap((doc, i) => ((now) => (now && !inLibrary(copies[i]) ? [{ doc: now, copy: copies[i] }] : []))(library.charts.find((other) => other.id === chartNow(doc.id))));
      if (left.length) duplicateCharts(left.map(({ doc }) => doc), left.map(({ copy }) => copy));
    };
  } });
  return copies;
}
/** @param {Chart} doc */
function duplicateChart(doc) {
  const base = `${titleOf(doc).replace(/ copy(?: \d+)?$/, "")} copy`, titles = new Set(library.charts.map(titleOf));
  let title = base;
  for (let n = 2; titles.has(title); n++) title = `${base} ${n}`;
  const copy = { ...structuredClone(doc), id: uid(), ...(doc.title ? { title } : {}), updated: Date.now() };
  library.charts.splice(library.charts.indexOf(doc) + 1, 0, copy);
  if (doc === chart) saveView();
  keepCarriedView(copy, { view: leftView(doc.id) });
  const drawn = previews.get(doc.id), mark = thumbs.get(doc.id);
  if (drawn) previews.set(copy.id, { ...drawn, key: "" });
  if (mark) {
    thumbs.set(copy.id, mark);
    saveThumbs();
  }
  return copy;
}
/** @param {string} [into] */
function newChart(into) {
  delete document.body.dataset.sheet;
  const folder = into ? folderById(into) : home && "folder" in homeAt ? folderById(homeAt.folder) : undefined;
  if (emptyYours(chart)) {
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

/** @param {HTMLElement[]} elements @param {(element: HTMLElement) => string | undefined} key */
const placesOf = (elements, key) => new Map(elements.map((element) => [key(element), element.getBoundingClientRect().top]));
/** @param {HTMLElement} element */
function groundOf(element) {
  for (let node = element.parentElement; node; node = node.parentElement) {
    const color = getComputedStyle(node).backgroundColor;
    if (color !== "transparent" && color !== "rgba(0, 0, 0, 0)") return color;
  }
  return css("--app");
}
/** @type {WeakMap<Element, object>} */
const lifts = new WeakMap();
/**
 * `moved` passes over the others, on its ground, so their words never cross; with `fade`, one that wasn't there fades
 * in. `layout`: at a layout's pace (with a box resizing: the sidebar's width), each moving one passing over the others
 * on its ground (its list's gaps too). Nothing moves under reduced motion.
 * @param {HTMLElement[]} elements
 * @param {(element: HTMLElement) => string | undefined} key
 * @param {Map<string | undefined, number>} was
 * @param {{ moved?: string | null, fade?: boolean, pseudo?: string, layout?: boolean }} [options]
 */
function slide(elements, key, was, { moved = null, fade = false, pseudo, layout = false } = {}) {
  if (reduced()) return;
  const timing = { duration: layout ? MOTION.layout : MOTION.panel, easing: layout ? MOTION.easeLayout : MOTION.ease, ...(pseudo ? { pseudoElement: pseudo } : {}) };
  const come = elements.filter((element) => was.get(key(element)) === undefined), going = elements.filter((element) => was.get(key(element)) !== undefined);
  // A glide still under way goes first: where it's laid out now is measured, not where it's being carried. All
    // cancelled, then all read, then all moved: a read between writes would lay a long list out again for each row.
  for (const element of going) for (const animation of element.getAnimations()) if (/** @type {KeyframeEffect | null} */ (animation.effect)?.getKeyframes().some((frame) => "transform" in frame)) animation.cancel();
  const tops = going.map((element) => element.getBoundingClientRect().top);
  const lifted = (/** @type {HTMLElement} */ element) => layout || (moved !== null && key(element) === moved);
  const first = going.find(lifted), ground = first ? groundOf(first) : "";
  if (fade) for (const element of come) element.animate({ opacity: [0, 1] }, layout ? { delay: MOTION.wordsWait, duration: MOTION.wordsIn, easing: MOTION.ease, fill: "backwards" } : timing);
  for (const [i, element] of going.entries()) {
    const by = /** @type {number} */ (was.get(key(element))) - tops[i];
    if (Math.abs(by) < 0.5) continue;
    const lift = lifted(element), glide = {};
    // Its ground over half its list's gap above and below too: what fades in under it never shows between them.
    if (lift) Object.assign(element.style, { zIndex: "1", background: ground, boxShadow: `0 calc(var(--row-gap, 0px) / -2) 0 ${ground}, 0 calc(var(--row-gap, 0px) / 2) 0 ${ground}` });
    if (lift) lifts.set(element, glide);
    // A glide cancelled by a newer one lands after it began: only the latest lift is let go, by its own end.
    const land = () => lifts.get(element) === glide && (lifts.delete(element), Object.assign(element.style, { zIndex: "", background: "", boxShadow: "" }));
    element.animate({ transform: [`translateY(${by}px)`, "none"] }, timing).finished.then(land, land);
  }
}

/** @param {HTMLElement[]} rows */
function landingIn(rows, /** @type {number} */ y, /** @type {(row: HTMLElement) => string} */ idOf, /** @type {string[]} */ ids) {
  const others = rows.filter((row) => !ids.includes(idOf(row))), row = others.find((each) => y < each.getBoundingClientRect().bottom) ?? others.at(-1);
  if (!row) return null;
  const box = row.getBoundingClientRect();
  return { item: row, after: y > box.top + box.height / 2 };
}

function stepChart(/** @type {string} */ id, /** @type {number} */ by) {
  const doc = chartById(id);
  if (doc && !isPreview(doc)) stepSide(doc, by);
}
/** @returns {{ into: string | undefined, before: string | null }} */
function besideItem(/** @type {string} */ key, /** @type {boolean} */ after, /** @type {string[]} */ keys) {
  const item = itemOf(key), into = item ? parentOf(item) : undefined;
  if (!after) return { into, before: key };
  const rest = childrenOf(into).filter((other) => !keys.includes(sideKey(other))), next = rest[rest.findIndex((other) => sideKey(other) === key) + 1];
  return { into, before: next ? sideKey(next) : null };
}
const chartLabel = (/** @type {string} */ id) => ((doc) => (doc ? `${chartMark(doc)}<span class="name">${escapeHtml(titleOf(doc))}</span>` : ""))(chartById(id));
const sideId = (/** @type {HTMLElement} */ row) => (row.dataset.chartRow ? `c:${row.dataset.chartRow}` : `f:${row.dataset.folderRow}`);
const depthOf = (/** @type {HTMLElement} */ row) => Number(row.style.getPropertyValue("--depth")) || 0;
/** @param {string[]} ids */
function placesFor(ids) {
  const carried = ids.filter((id) => id.startsWith("f:")).map((id) => id.slice(2));
  return all$("[data-chart-row]:not([inert]), [data-folder-row]:not([inert])", $("chart-list")).filter((row) => {
    const key = sideId(row), item = itemOf(key);
    return !ids.includes(key) && item !== undefined && !carried.some((id) => isIn(parentOf(item), id));
  });
}
// The grid's tiles move the charts in the list (the sidebar's order, and ⌘1–9's), those picked together. Found by the
// filter, they're ranked, not in their order: there's no place to drop them into (a hold still picks).
const reorderable = () => !$("home-filter").value.trim() && ("folder" in homeAt);
const rowKeyOf = (/** @type {Element | null} */ row) => (row instanceof HTMLElement ? Object.entries(row.dataset).map(([key, value]) => `${key}:${value}`).join(" ") || row.className : "");
function roveList() {
  const list = $("chart-list"), rows = all$(".chart-row:not([inert]), .folder-row:not([inert]), .list-head", list);
  for (const button of all$("button", list)) button.tabIndex = -1;
  const row = rows.find((each) => listStop && rowKeyOf(each) === listStop) ?? list.querySelector(".chart-row.active") ?? rows[0];
  const stop = /** @type {HTMLElement | null | undefined} */ (row?.querySelector(".nav-item, .list-head-name"));
  if (stop) stop.tabIndex = 0;
}
/**
 * The sidebar's list is one stop for Tab (not two a row: a long Library, a hundred): the row last on (by its key), else
 * the open chart's, else the first.
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
  $("chart-list").addEventListener("keydown", (event) => {
    if (composing(event) || typing(event.target)) return;
    const rows = all$(".chart-row:not([inert]), .folder-row:not([inert]), .list-head", $("chart-list")), row = /** @type {HTMLElement | null} */ (closest(event, ".chart-row, .folder-row, .list-head"));
    const first = (/** @type {HTMLElement | undefined} */ each) => /** @type {HTMLElement | null | undefined} */ (each?.querySelector(".nav-item, .list-head-name"));
    const level = (/** @type {HTMLElement} */ each) => (each.classList.contains("list-head") ? -1 : depthOf(each));
    if (!row) return;
    // The menu key and ⇧F10 open its menu as a right-click does, the focus back on the row as it closes; the browser's
        // own right-click from the same key, after it, isn't a second one.
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
      if (onRail() && (folder !== undefined || section !== undefined)) return void (event.key === "ArrowRight" && first(row) && railMenuKept(/** @type {HTMLElement} */ (first(row))));
      if (opens && open === (event.key === "ArrowLeft")) {
        if (folder) toggleFolder(folder);
        else if (group) toggleGroup(group);
        else toggleSection(/** @type {string} */ (section));
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
/** @type {{ preset: number, doc: Chart }[]} */
let recommendedCards = [];
function renderRecommended() {
  const list = presets;
  recommendedCards = !list ? [] : recommendedAt().flatMap((preset, k) => {
    if (preset < 0) return [];
    const edited = editedPreset(preset);
    if (edited && presetEdited(edited)) return [{ preset, doc: edited }];
    const doc = Object.assign(fromPreset(list[preset]), { id: `recommended:${k}` });
    return [{ preset, doc }];
  });
  recommendedDrawn = recommendedKey();
  const focus = focusKey($("templates"));
  $("templates").innerHTML = list ? recommendedCards.map(({ preset, doc }, i) => presetTileHtml(preset, doc, i)).join("")
    : presetsFailed ? `<p class="gallery-note">The presets couldn't be read. <button type="button" class="btn ghost small" data-gallery-retry>Try again</button></p>`
    : RECOMMENDED.map(() => WAITING_TILE).join("");
  cardsSeen.disconnect();
  for (const card of all$("[data-template]")) cardsSeen.observe(card);
  restoreFocus($("templates"), focus);
  fitStrip($("templates"));
}
let recommendedDrawn = "";
const recommendedKey = () => (presets ? JSON.stringify([light, ...recommendedAt().map((i) => ((doc) => (doc && presetEdited(doc) ? `${doc.id}@${doc.updated}` : ""))(i >= 0 ? editedPreset(i) : undefined))]) : "");
const cardQueue = new Set();
function retryPlots() {
  const failed = all$("[data-plot].failed");
  if (!failed.length) return;
  for (const frame of failed) frame.classList.replace("failed", "pending");
  for (const frame of failed) ((card) => card && cardQueue.add(Number(card.dataset.template)))(/** @type {HTMLElement | null} */ (frame.closest("[data-template]")));
  drawCards();
  if (!home) return;
  queueYours();
  for (const frame of gallerySeen) if (frame.classList.contains("pending")) galleryQueue.add(presetOf(frame));
  void drawGallery();
}
export {
  allShelfSteps, areaUnder, arrive, batch, beginOnTime, besideItem, bubblesAt, cardOf,
  cardQueue, CARDS, chartCounts, chartKeys, chartLabel, chartMark, chartMenu, chartPathRows, chartPick, childrenOf,
  clearPicks, closeDialogs, depthOf, dialogOpeners, dialogScrims, dragSheet, drawPreview, drawThumb, fadeDef,
  duplicateCharts, fillRow, fillSlot, folderActions, folderChoices, folderPath, folderRow, folderRows,
  forgetPaletteMarks, freshThumb, galleryFound, galleryQueue, gallerySeen, galleryWait, giveUpUnframed, go, gridActions,
  groupRows, groupShut, heatSamples, heightsSheet, isIn, itemOf, itemsOf, keepNaming,
  KEPT_PRESET_MARKS, keptMarkKey, keyMenuAt, keysCame, landingIn, leafRow, leave, leftPlaces, lineOver, markOf, morph,
  morphing, moveToFolder, movingSide, naming, newChart, newFolder, nextCard, nextGallery, NO_MARK, numbered, onceKeys,
  onChart, orphan, outOf, pageIn, pageOut, paint, paletteMarksAsked, panelSize, parentOf,
  pickChart, picked, placeItems, placesFor, placesOf, presetMark, presetMarkKey,
  presetMarkQueue, presetMarksAsked, presetOf, presetParts, presetRankOf, presetSearch,
  presetTiles, previewBars, previewIndex, previewKey, previewOf, previewSvg, 
  queueYours, recommendedCards, recommendedDrawn, recommendedKey, recommendedPresets, renderDoors,
  renderGallery, renderHomeHead, renderHorizon, renderPicked, renderRecommended, renderSideNav, renderTheme,
  renderYours, reorderable, restOf, retryFailed, retryPlots, roveList, sampleable, sampleIndex, savePreviews,
  saveThumbs, searchNow, sectionKeys, sectionRows, seePaletteMarks, setChildren, setParent,
  setTheme, sheetGoes, sheetTaken, shelfSteps, shortCount, showDialog, shownCharts, showPick, sideHead,
  sideId, sideKey, sideMoved, sideTree, sizeSheet, slide, slideAway, slides, speedOf, startGalleryRest, startListKeys,
  stepChart, stepShelf, stillness, stretched, themeMenu, thumbKey, tileOpen, toggleFolder, toggleGroup,
  toggleSection, treeOf, unstretched, updateMark, 
  walkCards, writeTile, yourQueue, yoursFound, yoursSeen, yoursShown,
};
