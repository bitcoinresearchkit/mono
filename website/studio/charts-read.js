// @ts-check
// A chart read in, from a link, a file or storage: checked and made whole (validate), its content as a link compares it,
// the view a link carries.
/** @import { Chart, Level } from "/studio/vocabulary.js" */
import { MAX_DRAWINGS, TOOLS, MAX_STROKE, pointOf, PAIRED, lookOfDrawing, LINES, labelText } from "/studio/tools.js";
import { FORMATS } from "/studio/format.js";
import { isSeriesName, isCatalogPath } from "/studio/catalog-walk.js";
import { clamp, uid } from "/studio/math.js";
import { isColor, PALETTE } from "/studio/color.js";
import { INTERVAL_LABEL, RANGES, DEFAULT_RANGE, AVERAGES, INTERVAL_ORDER } from "/studio/intervals.js";
import { EDGES, DASHES, GRADIENT_LEAST, GRADIENT_MOST, FORMULA_MAX, blankChart, MAX_PANES, STACKS, HEAT_GRADIENTS, HEAT_WEIGHTS, HEAT_MEASURES, DEFAULT_MEASURE, HEAT_ROWS, TYPES, WIDTHS, OPACITIES, defaultOpacity, SHAPES, BUBBLE_SIZES, BUBBLE_SIZE, BUBBLE_HIDES, BUBBLE_HIDE, MARK_PLACES, makeLayer, markable, letterAt, kindColor, placedOn } from "/studio/vocabulary.js";
import { windowShort } from "/studio/words.js";
import { rightGap, forgetSpans } from "/studio/chart.js";
import { chart, chartShown, shownView, openingView, openView, timeAt, indexAt, engine, chartWindow, firstLook, firstForm, isView, viewsNow, putViews, library, takeView, normalize } from "/studio/app.js";
/** @import { Layer } from "/studio/vocabulary.js" */
import { refsOf, parse } from "/studio/formulas.js";

const contentOf = (/** @type {Chart} */ doc) => JSON.stringify([doc.title, doc.index, doc.range, Boolean(doc.halvings), doc.legend ?? "", doc.panes.map(({ id, ...pane }) => pane), doc.layers.map(({ id, pane, path, paths, ...layer }) => ({ ...layer, pane: doc.panes.findIndex((item) => item.id === pane), ...(layer.from ? { from: layer.from.map((member) => (EDGES.includes(member) ? member : doc.layers.findIndex((item) => item.id === member))) } : {}), ...(layer.ref ? { ref: doc.layers.findIndex((item) => item.id === layer.ref) } : {}), ...(layer.colorFrom ? { colorFrom: doc.layers.findIndex((item) => item.id === layer.colorFrom) } : {}) })), (doc.drawings ?? []).map(({ id, layer, ...drawing }) => ({ ...drawing, layer: doc.layers.findIndex((item) => item.id === layer) }))], (_key, value) => (value && typeof value === "object" && !Array.isArray(value) ? Object.fromEntries(Object.entries(value).sort(([a], [b]) => (a < b ? -1 : 1))) : value));
const INTERVAL_OF_FIELD = Object.fromEntries(Object.entries(INTERVAL_LABEL).map(([id, label]) => [String(label).toLowerCase(), id]));
/** @param {URLSearchParams} params @param {string} [said] */
function linkLookOf(params, said = undefined) {
  const range = params.get("range")?.toLowerCase(), interval = params.get("interval")?.toLowerCase() ?? "", from = params.get("from"), to = params.get("to"), view = { from, ...(to === null ? {} : { to }) }, window = params.get("window") ?? said;
  return { ...(RANGES.some(([id]) => id === range) ? { range: /** @type {string} */ (range) } : {}), ...(Object.hasOwn(INTERVAL_OF_FIELD, interval) ? { index: INTERVAL_OF_FIELD[interval] } : {}), ...(datedView("day1", view) ? { view } : {}), ...(window ? { window } : {}) };
}
/** @param {string} index @param {any} view */
function datedView(index, view) {
  const from = timeOfText(view?.from), to = view?.to === undefined && from !== null ? Date.now() / 1000 : timeOfText(view?.to);
  return from !== null && to !== null && from <= to ? { index, from: [from, 0], to: [to, 0] } : null;
}
const timeOfText = (/** @type {any} */ value) => {
  // Without a zone, UTC, as every date here.
  const text = typeof value === "string" ? value.trim().replace(/^(\d{4}-\d\d-\d\d)[T ](\d\d:\d\d(?::\d\d(?:\.\d+)?)?)$/, "$1T$2Z") : null;
  const time = Number.isFinite(value) ? value : text !== null && Number.isFinite(Date.parse(text)) ? Date.parse(text) / 1000 : NaN;
  return Math.abs(time) < 1e11 ? Math.floor(time) : null;
};
function lineStyleOf(line) {
  return { ...(isColor(line?.color) ? { color: line.color } : {}), ...(DASHES.some(([style]) => style === line?.dash) ? { dash: line.dash } : {}), ...(line?.line === false ? { line: /** @type {false} */ (false) } : {}), ...(line?.tag === true ? { tag: true } : {}) };
}
/** @returns {Level[] | undefined} */
function levelsOf(input) {
  if (!Array.isArray(input)) return undefined;
  const levels = input.map((level) => (typeof level === "number" ? { value: level } : level)).filter((level) => Number.isFinite(level?.value)).slice(0, 12);
  return levels.length ? levels.map((level) => ({ value: level.value, ...lineStyleOf(level), ...(typeof level.name === "string" && level.name.trim() ? { name: level.name.trim().slice(0, 40) } : {}) })) : undefined;
}
/** @returns {[number, string][] | undefined} */
function gradientOf(/** @type {any} */ input) {
  if (!Array.isArray(input) || input.length < GRADIENT_LEAST || input.length > GRADIENT_MOST) return undefined;
  const stops = input.filter((stop) => Array.isArray(stop) && Number.isFinite(stop[0]) && isColor(stop[1])).map(([at, color]) => /** @type {[number, string]} */ ([Math.round(clamp(at, 0, 100)), color]));
  if (stops.length !== input.length) return undefined;
  // Each a point apart at least: one on another would hide it (see moveStop).
  stops.sort((a, b) => a[0] - b[0]);
  for (let k = 1; k < stops.length; k++) stops[k][0] = Math.max(stops[k][0], stops[k - 1][0] + 1);
  for (let k = stops.length - 1; k >= 0; k--) stops[k][0] = Math.min(stops[k][0], k < stops.length - 1 ? stops[k + 1][0] - 1 : 100);
  return stops;
}
// lag( becomes shift( where that still fits (a cut one would break). A function, not a const: charts are read before
// the script below runs.
function formulaNow(/** @type {string} */ formula) {
  const now = formula.replace(/(?<![A-Za-z0-9_])lag(?=\s*\()/g, "shift");
  return now.length <= FORMULA_MAX ? now : formula;
}
/**
 * Only `keepIds` preserves valid, unique ids; the caller normalizes after.
 * @returns {Chart}
 */
function validate(input, keepIds = false) {
  const taken = new Set();
  const idOf = (id) => (keepIds && typeof id === "string" && /^[a-z0-9]{1,16}$/.test(id) && !taken.has(id) ? (taken.add(id), id) : uid());
  if (!input || !Array.isArray(input.layers) || !Array.isArray(input.panes) || !input.panes.length) throw new Error("Not a studio chart");
  const doc = blankChart(String(input.title ?? "").slice(0, 120));
  doc.index = Object.hasOwn(INTERVAL_LABEL, input.index) ? input.index : "day1";
  doc.range = RANGES.some(([id]) => id === input.range) ? input.range : DEFAULT_RANGE;
  if (input.halvings === true) doc.halvings = true;
  if (input.legend === "off") doc.legend = "off";
  const panes = input.panes.slice(0, MAX_PANES);
  doc.panes = panes.map((pane) => ({ id: idOf(pane?.id), ...(Number.isFinite(pane?.size) && pane.size > 0 && pane.size < 100 ? { size: pane.size } : {}), mode: [0, 1, 2, 3].includes(pane?.mode) ? pane.mode : 0, ...(STACKS.some(([id]) => id && id === pane?.stack) ? { stack: pane.stack } : {}), ...(pane?.reverse === true ? { reverse: true } : {}), ...(pane?.outline === true ? { outline: /** @type {true} */ (true) } : {}), ...(HEAT_GRADIENTS.some(([id], k) => k && id === pane?.gradient) ? { gradient: pane.gradient } : {}), ...(Object.hasOwn(AVERAGES, pane?.average) ? { average: pane.average } : {}) }));
  const paneFor = (id) => doc.panes[Math.max(0, panes.findIndex((pane) => pane?.id === id))].id;
  const layerIds = new Map();
  /** @param {any} layer */
  const layerOf = (layer) => {
    if (!layer || typeof layer !== "object") return null;
    const made = Array.isArray(layer.trend) ? "trend" : Array.isArray(layer.cloud) ? "cloud" : Array.isArray(layer.from) && (layer.type === "cloud" || layer.type === "trend") ? layer.type : null;
    const madeOf = made ? layer.trend ?? layer.cloud ?? layer.from : null;
    const heat = !made && typeof layer.heatmap === "string" && /^[a-z0-9_]{1,40}$/.test(layer.heatmap) ? layer.heatmap : null;
    const cohort = (/** @type {any} */ id) => typeof id === "string" && /^[a-z0-9_]{1,40}$/.test(id);
    const groups = !made && !heat ? [...new Set((Array.isArray(layer.profile) ? layer.profile : [layer.profile]).filter(cohort))].slice(0, 40) : [];
    const profiled = !made && !heat && (Array.isArray(layer.profile) || cohort(layer.profile));
    /** @type {Record<string, any>} */
    const spec = {
      ...(heat || profiled ? { ...(heat ? { heatmap: heat } : { profile: groups, ...(layer.cumulative === true ? { cumulative: true } : {}), ...(layer.cohortColors && typeof layer.cohortColors === "object" ? ((colors) => (Object.keys(colors).length ? { cohortColors: colors } : {}))(Object.fromEntries(Object.entries(layer.cohortColors).filter(([id, color]) => groups.includes(id) && isColor(color)))) : {}) }), weight: HEAT_WEIGHTS.some(([id]) => id && id === layer.weight) ? layer.weight : undefined, measure: HEAT_MEASURES.some(([id]) => id !== DEFAULT_MEASURE && id === layer.measure) ? layer.measure : undefined, rows: HEAT_ROWS.includes(layer.rows) ? layer.rows : undefined } : madeOf ? { from: madeOf.filter((id) => typeof id === "string").map((id) => (EDGES.includes(id.toLowerCase()) ? id.toLowerCase() : id)) } : typeof layer.series === "string"
        ? { series: isSeriesName(layer.series) ? layer.series : "invalid_series_name", ...(isCatalogPath(layer.path) ? { path: layer.path } : {}) }
        : { formula: formulaNow(String(layer.formula ?? "").slice(0, FORMULA_MAX)), ...(layer.paths && typeof layer.paths === "object" ? { paths: Object.fromEntries(Object.entries(layer.paths).filter(([name, path]) => isSeriesName(name) && isCatalogPath(path))) } : {}) }),
      ...(typeof layer.label === "string" ? { label: layer.label.slice(0, 120) } : {}), ...(typeof layer.short === "string" ? { short: layer.short.slice(0, 24) } : {}),
      color: isColor(layer.color) ? layer.color : undefined,
      type: heat ? "heatmap" : profiled ? "profile" : made ?? (TYPES.some(([type]) => type === layer.type) || layer.type === "bubbles" || (layer.type === "signal" && typeof layer.series !== "string") ? layer.type : "line"),
      width: WIDTHS.includes(layer.width) ? layer.width : 1.5,
      dash: DASHES.some(([style]) => style === layer.dash) ? layer.dash : 0,
      pane: paneFor(layer.pane),
      side: layer.side === "left" ? "left" : "right",
      hidden: layer.hidden === true || undefined,
      priceLine: layer.priceLine === true || undefined,
      outline: layer.outline === true || undefined,
      fit: layer.fit === false || layer.fit === "zero" ? layer.fit : undefined,
      maxInterval: INTERVAL_ORDER.includes(layer.maxInterval) ? layer.maxInterval : undefined,
      colorBelow: isColor(layer.colorBelow) ? layer.colorBelow : undefined,
      colors: layer.colors === 1 || layer.colors === 4 ? layer.colors : undefined,
      colorFading: isColor(layer.colorFading) ? layer.colorFading : undefined,
      colorBelowFading: isColor(layer.colorBelowFading) ? layer.colorBelowFading : undefined,
      opacity: OPACITIES.includes(layer.opacity) && layer.opacity !== defaultOpacity({ type: made ?? "line" }) ? layer.opacity : undefined,
      letter: typeof layer.letter === "string" && /^[A-Z]{1,2}$/.test(layer.letter) ? layer.letter : undefined,
      ...(layer.format !== "auto" && FORMATS.some(([id]) => id === layer.format) ? { format: layer.format } : {}),
      ...(Number.isFinite(layer.base) ? { base: layer.base } : {}),
      ...(SHAPES.some(([id]) => id !== "straight" && id === layer.shape) ? { shape: layer.shape } : {}),
      ...(layer.fill === "above" ? { fill: "above" } : {}),
      ...(OPACITIES.includes(layer.fillOpacity) ? { fillOpacity: layer.fillOpacity } : {}),
      ...(layer.solid === true ? { solid: true } : {}),
      ...(layer.hollow === "none" ? { hollow: "none" } : {}),
      ...((made === "trend" || (!made && (layer.type === "signal" || layer.type === "bubbles"))) && typeof layer.ref === "string" ? { ref: layer.ref } : {}),
      ...(!made && layer.type === "bubbles" ? { size: BUBBLE_SIZES.includes(layer.size) && layer.size !== BUBBLE_SIZE ? layer.size : undefined, hide: BUBBLE_HIDES.includes(layer.hide) && layer.hide !== BUBBLE_HIDE ? layer.hide : undefined, scaleTo: layer.scaleTo === "all" ? "all" : undefined } : {}),
      ...(MARK_PLACES.some(([place]) => place !== "above" && place === layer.place) ? { place: layer.place } : {}),
      ...(!made && layer.type === "signal" && layer.results === true ? { results: true } : {}),
      ...(typeof layer.colorFrom === "string" ? { colorFrom: layer.colorFrom } : {}),
      ...(layer.byLevels === true ? { byLevels: true } : {}),
      ...(layer.byLayers === true ? { byLayers: true } : {}),
      ...((stops) => (stops ? { gradient: stops, ...(layer.byGradient === true ? { byGradient: true } : {}), ...(layer.upToDate === true ? { upToDate: true } : {}) } : {}))(gradientOf(layer.gradient)),
      levels: levelsOf(layer.levels),
      ...(typeof layer.underName === "string" && layer.underName.trim() && levelsOf(layer.levels) ? { underName: layer.underName.trim().slice(0, 40) } : {}),
      baseLine: Object.keys(lineStyleOf(layer.baseLine)).length ? lineStyleOf(layer.baseLine) : undefined,
    };
    const id = idOf(layer.id);
    if (typeof layer.id === "string") layerIds.set(layer.id, id);
    return makeLayer({ ...Object.fromEntries(Object.entries(spec).filter(([, value]) => value !== undefined)), id }, doc);
  };
  for (const layer of input.layers) ((read) => read && doc.layers.push(read))(layerOf(layer));
  for (const layer of doc.layers) if (layer.ref) layer.ref = layerIds.get(layer.ref) ?? "";
  for (const layer of doc.layers) if (layer.type === "signal" && layer.ref === undefined) layer.ref = markable(doc)[0]?.id;
  for (const layer of doc.layers) if (layer.colorFrom) layer.colorFrom = layerIds.get(layer.colorFrom);
  for (const layer of doc.layers) if (layer.from) layer.from = layer.from.flatMap((id) => (EDGES.includes(id) ? [id] : layerIds.has(id) ? [/** @type {string} */ (layerIds.get(id))] : []));
  doc.drawings = (Array.isArray(input.drawings) ? input.drawings : []).slice(0, MAX_DRAWINGS).flatMap((drawing) => {
    const kind = drawing?.kind, layer = layerIds.get(drawing?.layer);
    if (!Object.hasOwn(TOOLS, kind) || !layer || !Array.isArray(drawing.points)) return [];
    const points = drawing.points.slice(0, MAX_STROKE).filter((point) => Array.isArray(point) && point.length === 2 && point.every(Number.isFinite) && Math.abs(point[0]) < 1e11).map(([time, value]) => pointOf(time, value));
    const takes = kind === "brush" || kind === "path" ? MAX_STROKE : PAIRED.has(kind) ? 2 : 1, least = kind === "path" ? 2 : kind === "brush" ? 1 : takes, text = String(drawing.text ?? "").slice(0, 500);
    if (points.length < least || (kind === "text" && !text.trim())) return [];
    return [{
      id: idOf(drawing.id), kind, layer, points: points.slice(0, takes),
      ...lookOfDrawing(kind, drawing),
      ...(kind === "text" ? { text } : {}),
      ...(LINES.has(kind) && typeof drawing.label === "string" && labelText(drawing.label) ? { label: labelText(drawing.label), ...(Number.isFinite(drawing.at) ? { at: clamp(drawing.at, 0, 1) } : {}) } : {}),
    }];
  });
  return doc;
}
const dateText = (/** @type {number} */ time) => ((iso) => (iso.endsWith("T00:00:00Z") ? iso.slice(0, 10) : iso))(new Date(time * 1000).toISOString().replace(/\.\d+Z$/, "Z"));
/** @param {Chart} doc */
const carriedView = (doc) => (doc === chart ? (chartShown() ? shownView() : openingView(doc)) : openView(doc));
/** @param {Chart} doc */
function linkDates(doc) {
  const kept = carriedView(doc);
  return kept ? Object.fromEntries((/** @type {const} */ (["from", "to"])).map((edge) => [edge, dateText(timeAt(kept.index, indexAt(kept.index, kept[edge][0]) + Math.round(kept[edge][1])))])) : null;
}
/** @param {Chart} doc */
// Within half again the room after the latest (a drag's slack): a view let go of past it is a view of its own.
const endsAtLatest = (doc) => ((view) => doc === chart && Boolean(view?.latest) && (view.latest[1] * engine.time.width) / (view.latest[1] - view.latest[0]) <= rightGap() * 1.5)(carriedView(doc));
/** @param {Chart} doc @param {{ index: string, range: string, window?: string }} made */
function lookFields(doc, made) {
  const dates = linkDates(doc);
  const field = (/** @type {string} */ text) => text.replace(/:00Z$/, "");
  const look = [...(doc.range !== made.range ? [`range=${RANGES.find(([id]) => id === doc.range)?.[1] ?? doc.range}`] : []), ...(dates ? [`from=${field(dates.from)}`, ...(endsAtLatest(doc) ? [] : [`to=${field(dates.to)}`])] : [])];
  const window = made.window && chartWindow(doc)?.window;
  return [...look, ...(doc.index !== made.index ? [`interval=${INTERVAL_LABEL[doc.index]}`] : []), ...(window && window !== made.window ? [`window=${windowShort(window)}`] : [])].join("&");
}
function fromSketch(/** @type {any} */ input) {
  if (!input || typeof input !== "object" || !Array.isArray(input.layers)) throw new Error("it lists no layers");
  const layers = input.layers.filter((/** @type {any} */ layer) => layer && typeof layer === "object");
  const number = (/** @type {any} */ value) => (Number.isInteger(value) ? value : typeof value === "string" && /^\d{1,3}$/.test(value) ? Number(value) : null);
  const asked = Math.max(1, ...layers.map((/** @type {any} */ layer) => number(layer.pane) ?? 1)), listed = Array.isArray(input.panes) ? input.panes : [];
  const given = Array.from({ length: clamp(Math.max(asked, listed.length), 1, MAX_PANES) }, (_, i) => listed[i] ?? {});
  const free = (/** @type {Set<string>} */ taken, /** @type {(n: number) => string} */ make) => {
    let n = 0;
    return () => {
      while (taken.has(make(n))) n++;
      const next = make(n);
      taken.add(next);
      return next;
    };
  };
  const ownString = (/** @type {any} */ value, /** @type {RegExp} */ shape) => (typeof value === "string" && shape.test(value) ? value : null);
  const paneIds = new Set(given.map((/** @type {any} */ pane) => ownString(pane?.id, /^[a-z0-9]{1,16}$/)).filter((id) => id !== null)), nextPane = free(paneIds, (n) => `p${n}`);
  const panes = given.slice(0, MAX_PANES).map((/** @type {any} */ pane, /** @type {number} */ i) => ({ ...(i ? {} : { mode: 1 }), ...pane, id: ownString(pane?.id, /^[a-z0-9]{1,16}$/) ?? nextPane() }));
  const ownId = (/** @type {any} */ value) => ((id) => (id && !EDGES.includes(id) ? id : null))(ownString(value, /^[a-z0-9]{1,16}$/));
  const nextId = free(new Set(layers.map((/** @type {any} */ layer) => ownId(layer.id)).filter(Boolean)), (n) => `l${n}`);
  const ids = layers.map((/** @type {any} */ layer) => ownId(layer.id) ?? nextId());
  const nextLetter = free(new Set(layers.map((/** @type {any} */ layer) => ownString(layer.letter, /^[A-Z]{1,2}$/)).filter(Boolean)), letterAt);
  const letters = layers.map((/** @type {any} */ layer) => ownString(layer.letter, /^[A-Z]{1,2}$/) ?? nextLetter());
  const usedColors = new Set(layers.map((/** @type {any} */ layer) => (isColor(layer.color) ? layer.color : null)).filter(Boolean));
  let cycled = 0;
  const colors = layers.map((/** @type {any} */ layer) => {
    if (isColor(layer.color)) return layer.color;
    if (Array.isArray(layer.cloud)) return null;
    const look = typeof layer.series === "string" ? firstLook(seriesName(layer.series), usedColors, typeof layer.type === "string" ? layer.type : undefined) : {};
    const own = kindColor({ type: typeof layer.heatmap === "string" ? "heatmap" : typeof layer.type === "string" ? layer.type : look.type, measure: layer.measure });
    const color = look.color ?? own ?? PALETTE.find((name) => !usedColors.has(name)) ?? PALETTE[cycled++ % PALETTE.length];
    usedColors.add(color);
    return color;
  });
  const letterId = (/** @type {any} */ ref) => (typeof ref === "string" && ids.includes(ref) ? ref : ((key) => (letters.includes(key) ? ids[letters.indexOf(key)] : ref))(typeof ref === "string" ? ref.toUpperCase() : ref));
  const index = Object.hasOwn(INTERVAL_LABEL, input.index) ? input.index : "day1", dated = datedView(index, input.view);
  return { ...input,
    ...(Array.isArray(input.drawings) ? { drawings: input.drawings.slice(0, MAX_DRAWINGS).filter((/** @type {any} */ drawing) => drawing && typeof drawing === "object").map((/** @type {any} */ drawing) => ({
      ...drawing,
      layer: letterId(drawing.layer),
      points: Array.isArray(drawing.points) ? drawing.points.slice(0, MAX_STROKE).map((/** @type {any} */ point) => (Array.isArray(point) ? [timeOfText(point[0]), point[1]] : point)) : drawing.points,
    })) } : {}),
    ...(dated ? { view: dated } : {}),
    panes, layers: layers.map((/** @type {any} */ layer, /** @type {number} */ i) => {
    const kind = Array.isArray(layer.trend) ? "trend" : Array.isArray(layer.cloud) ? "cloud" : null, members = kind ? layer[kind].map(letterId) : null;
    return {
      ...layer,
      id: ids[i],
      letter: letters[i],
      color: colors[i] ?? colors[ids.indexOf(members?.find((member) => ids.includes(member)))] ?? PALETTE[0],
      ...(kind === "cloud" && !colors[i] && layer.colorBelow === undefined ? { byLayers: true } : {}),
      ...(Array.isArray(layer.gradient) && layer.byGradient === undefined ? { byGradient: true } : {}),
      ...(number(layer.pane) !== null ? { pane: panes[clamp(/** @type {number} */ (number(layer.pane)), 1, panes.length) - 1].id } : layer.pane === undefined ? { pane: panes[0].id } : {}),
      ...(typeof layer.series === "string" ? { series: seriesName(layer.series), ...(({ type, base }) => ({ ...(type && layer.type === undefined ? { type } : {}), ...(base !== undefined && layer.base === undefined ? { base } : {}) }))(firstForm(seriesName(layer.series), typeof layer.type === "string" ? layer.type : undefined)) } : {}),
      ...(kind ? { [kind]: members } : {}),
      ...(typeof layer.ref === "string" ? { ref: letterId(layer.ref) } : {}),
      ...(typeof layer.colorFrom === "string" ? { colorFrom: letterId(layer.colorFrom) } : {}),
    };
  }) };
}
const seriesName = (/** @type {string} */ text) => text.trim().toLowerCase().replace(/[\s-]+/g, "_");
/** @param {Chart} doc */
function keepCarriedView(doc, input) {
  const carried = input?.view;
  if (!isView(carried)) return;
  const all = viewsNow();
  all[doc.id] = { index: carried.index, from: carried.from, to: carried.to, ...(carried.latest ? { latest: carried.latest } : {}), at: Date.now() };
  putViews(all);
}
const sameChart = (/** @type {Chart} */ a, /** @type {Chart} */ b) => contentOf({ ...a, range: "", index: "" }) === contentOf({ ...b, range: "", index: "" });
/** @param {Chart} doc */
function twinOf(doc) {
  const twin = library.charts.find((other) => sameChart(other, doc));
  if (!twin) return null;
  takeView(doc.id, twin.id);
  forgetSpans(twin.id);
  return twin;
}
const BASE32 = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
/** @param {string} text */
function fromBase32(text) {
  const bytes = [];
  let bits = 0, value = 0;
  for (const char of text.toUpperCase()) {
    const digit = BASE32.indexOf(char);
    if (digit < 0) throw new Error("That link doesn't contain a chart");
    value = ((value << 5) | digit) & 0xffff;
    if ((bits += 5) >= 8) bytes.push((value >>> (bits -= 8)) & 255);
  }
  return Uint8Array.from(bytes);
}
// A "+" arrives as a space (a form's way) and is read as "+" again, as links always opened.
function fromBase64(/** @type {string} */ text) {
  const plain = text.replace(/[- ]/g, "+").replace(/_/g, "/");
  return /** @type {Uint8Array<ArrayBuffer>} */ (/** @type {any} */ (Uint8Array).fromBase64?.(plain) ?? Uint8Array.from(atob(plain), (char) => char.charCodeAt(0)));
}
/** @param {string} encoded */
async function fromLink(encoded, base32 = false) {
  const bytes = base32 ? fromBase32(encoded) : fromBase64(encoded);
  // Inflating stops at two megabytes whatever the link claims: a chart is a few kilobytes at most.
  const reader = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("deflate-raw")).getReader();
  const chunks = [];
  for (let size = 0; ;) {
    const { done, value } = await reader.read();
    if (done) break;
    if ((size += value.length) > 2_000_000) {
      reader.cancel();
      throw new Error("Too large to be a chart");
    }
    chunks.push(value);
  }
  const input = JSON.parse(await new Blob(chunks).text());
  if (input?.v === undefined) return readChart(input);
  const doc = validate(input);
  normalize(doc);
  keepCarriedView(doc, input);
  return doc;
}
function readChart(/** @type {any} */ input) {
  const sketch = fromSketch(input), doc = validate(sketch);
  normalize(doc);
  keepCarriedView(doc, sketch);
  return doc;
}
/** @param {Layer} layer @param {Chart} doc */
function lettersRead(layer, doc) {
  const letters = new Set();
  if (layer.formula !== undefined) try { for (const letter of refsOf(parse(layer.formula))) letters.add(letter); } catch {}
  // A signal's ref is the layer it's marked on, not one it reads; a trend's, the one that flips it: read.
  for (const id of [...(layer.from ?? []), ...(layer.ref && !placedOn(layer) ? [layer.ref] : [])]) {
    const made = doc.layers.find((item) => item.id === id);
    if (made) letters.add(made.letter);
  }
  return letters;
}
/** @param {Layer} layer */
function readsOf(layer, doc = chart) {
  const seen = new Set(), queue = [layer];
  while (queue.length) {
    for (const letter of lettersRead(/** @type {Layer} */ (queue.pop()), doc)) {
      if (seen.has(letter)) continue;
      seen.add(letter);
      const next = doc.layers.find((item) => item.letter === letter);
      if (next) queue.push(next);
    }
  }
  return seen;
}
export { validate, formulaNow, gradientOf, levelsOf, lineStyleOf, contentOf, datedView, timeOfText, linkLookOf, INTERVAL_OF_FIELD, fromSketch, seriesName, lookFields, linkDates, endsAtLatest, carriedView, dateText, fromLink, twinOf, sameChart, fromBase32, fromBase64, readChart, keepCarriedView, BASE32, readsOf, lettersRead
};
