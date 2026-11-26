// @ts-check
// A chart's document and its words: its shape, layer kinds and looks, marks, measures, panes, how layers are made from
// layers, a new layer, a blank chart.
import { precise } from "/studio/format.js";
import { DEFAULT_RANGE, FINE } from "/studio/intervals.js";
import { uid } from "/studio/math.js";
import { PALETTE } from "/studio/color.js";
/** @import { Result } from "/studio/chart.js" */
/** @import { End } from "/studio/tools.js" */

const TWO_KEPT = new Set(["trend", "cloud"]);
const TWO_OR_ONE = new Set(["candles", "bubbles"]);
/** @param {Layer} layer */
const hollowOf = (layer) => layer.hollow !== "none";
const belowOf = (layer) => ((layer.type === "histogram" || TWO_OR_ONE.has(layer.type) || TWO_KEPT.has(layer.type)) && layer.colors === 1 ? layer.color : layer.colorBelow ?? (TWO_KEPT.has(layer.type) ? layer.color : "red"));
/** @param {Layer} layer @param {Chart | null} [doc] */
const layersColors = (layer, doc) => {
  const [first, second] = (layer.from ?? []).flatMap((id) => ((item) => (item ? [item] : []))(doc?.layers.find((one) => one.id === id)));
  if (first) layer.color = first.color;
  if (second) layer.colorBelow = second.color;
  else delete layer.colorBelow;
};
const LOOK_KEYS = /** @type {const} */ (["color", "colorBelow", "colorFrom", "byLevels", "byGradient"]);
/** @param {{ type?: string }} layer */
const heatLike = (layer) => layer.type === "heatmap" || layer.type === "profile";
/** @param {{ type?: string, measure?: string }} layer */
const twoSided = (layer) => layer.type === "baseline" || (layer.type === "heatmap" && signedMeasure(layer));
/** @param {{ measure?: string }} layer */
const measureOf = (layer) => layer.measure ?? DEFAULT_MEASURE;
/** @param {{ measure?: string }} layer */
const signedMeasure = (layer) => measureOf(layer) === "pnl";
/** @param {string} measure */
const measureUnit = (measure) => (measure === DEFAULT_MEASURE ? "btc" : "usd");
const HEAT_GRADIENT = /** @type {[number, string][]} */ ([[0, "#000004"], [20, "#420a68"], [81, "#fcaa15"], [100, "#fcffa4"]]);
/** @type {[string, string, [number, string][]][]} */
const HEAT_GRADIENTS = [
  ["inferno", "Inferno", HEAT_GRADIENT],
  ["magma", "Magma", [[0, "#000004"], [18, "#331064"], [71, "#f8755e"], [100, "#fcfdbf"]]],
  ["viridis", "Viridis", [[0, "#440154"], [39, "#2c768e"], [69, "#41bd72"], [100, "#fde725"]]],
  ["plasma", "Plasma", [[0, "#0d0887"], [33, "#991a9e"], [72, "#f48b48"], [100, "#f0f921"]]],
];
/** @param {{ type?: string, measure?: string }} layer */
const kindColor = (layer) => (twoSided(layer) ? "green" : null);
/** @param {Layer} layer @param {{ type?: string, measure?: string }} next */
function kindLook(layer, next) {
  const color = kindColor(next);
  if (color) layer.color = color;
  if (twoSided(next)) delete layer.colorBelow, delete layer.byGradient;
}
/** @param {Layer} layer */
const shapeKey = (layer) => (layer.type === "heatmap" && layer.measure === "pnl" ? "heatmap:pnl" : layer.type);
/** @type {[string, string][]} */
const FROM_TYPES = [["cloud", "Cloud"], ["trend", "Trend"]];
const TREND_SAYS = "Follows the lowest until the layer that flips it touches it, then the highest until that layer touches it, and so on";
/** @param {Chart} doc */
const markable = (doc) => doc.layers.filter((other) => other.type !== "cloud" && !placedOn(other) && !heatLike(other));
const MARK_PLACES = /** @type {[string, string, string][]} */ ([["above", "Above", "above"], ["below", "Below", "below"], ["on", "On it", "on"], ["shade", "Shaded", ""]]);
/** @param {string} place */
const placeName = (place) => ((text) => (place === "shade" ? "Shaded while up or down" : place === "on" ? "On the layer" : `${text} the layer`))(MARK_PLACES.find(([id]) => id === place)?.[1] ?? "");
/**
 * That layer's change over it, up; the opposite, down (the state was right: more); none for one of neither, one begun before a change, or a value not above 0 (or not finite: a formula's division by 0).
 * @param {{ at: number, was: number, since: number }} change @param {Result | null | undefined} reference
 */
const stretchResult = (change, reference) => {
  const values = reference && measured(reference);
  if (!values) return undefined;
  const from = values[change.since], to = values[change.at];
  return change.was && change.since >= 0 && from > 0 && to > 0 && Number.isFinite(from) && Number.isFinite(to) ? change.was * (to / from - 1) : undefined;
};
// Measurements belong to the result, even if its view shares a vector with another layer's formula.
const measured = (/** @type {Result} */ result) => result.measurement ?? result.values;
const resultText = (/** @type {number} */ result) => (result >= 9 ? `${precise(1 + result)}×` : `${result < 0 ? "−" : "+"}${precise(Math.abs(result) * 100)}%`);
const typeName = (/** @type {string} */ type) => [...TYPES, ...FROM_TYPES, ["signal", "Signal"], ["bubbles", "Bubbles"], ["heatmap", "Heatmap"], ["profile", "Profile"]].find(([id]) => id === type)?.[1] ?? type;
// Each day's URPD has all three: switching measure asks for nothing again.
const HEAT_MEASURES = /** @type {[string, string, string][]} */ ([["supply", "Supply", "Supply"], ["capital", "Capital", "Capital"], ["pnl", "Unrealized P&L", "P&L"]]), DEFAULT_MEASURE = HEAT_MEASURES[0][0];
const HEAT_WEIGHTS = /** @type {[string, string][]} */ ([["", "Every coin"], ["wakefulness", "Awake"], ["mobility", "Mobile"]]);
const PROFILE_VIEWS = /** @type {[string, string][]} */ ([["bars", "Bars"], ["cumulative", "Cumulative"]]);
const HEAT_ROWS = [100, 200, 500], DEFAULT_ROWS = 200;
const GRADIENT_MOST = 4, GRADIENT_LEAST = 2;
/**
 * Made-of-layers kinds (a cloud, a trend, a signal, bubbles) are kinds of their own, added as such, not a look to switch to.
 * @type {[string, string][]}
 */
const TYPES = [["line", "Line"], ["dots", "Dots"], ["area", "Area"], ["histogram", "Bars"], ["baseline", "Baseline"], ["candles", "Candles"]];
const LINED = new Set(["line", "area", "baseline", "trend"]);
const OUTLINED = new Set(["line", "dots", "area", "histogram", "baseline", "candles", "trend", "bubbles"]);
const MARKER = { radius: 3, border: 2 };
const BUBBLE_SIZES = Array.from({ length: 11 }, (_, i) => 8 + 4 * i), BUBBLE_SIZE = 24, BUBBLE_HIDES = Array.from({ length: 26 }, (_, i) => i / 100), BUBBLE_HIDE = 0.05;
const hideText = (/** @type {number} */ share) => (share ? `${Math.round(share * 100)}%` : "None");
/** @param {Layer} layer */
const placedOn = (layer) => layer.type === "signal" || layer.type === "bubbles";
/** @param {Layer} layer */
const fittable = (layer) => !placedOn(layer) && !heatLike(layer) && !(layer.type === "cloud" && edgesOf(layer).length === 2 && !cloudLayers(layer).length);
/** @param {Layer} layer */
const framesAxis = (layer) => !layer.hidden && layer.fit !== false && (heatLike(layer) || fittable(layer));
/** @param {Chart} doc @param {Layer} layer @param {boolean} stacked */
const automaticZero = (doc, layer, stacked) => {
  if (stacked || layer.fit === "zero" || !framesAxis(layer) || !(layer.series || layer.formula)) return false;
  return doc.layers.every((other) => other.id === layer.id || !framesAxis(other) || other.pane !== layer.pane || other.side !== layer.side);
};
/** @param {Chart} doc @param {Layer} layer */
const unfitted = (doc, layer) => layer.fit === false && doc.layers.some((other) => other !== layer && framesAxis(other) && other.pane === layer.pane && other.side === layer.side);
/** @param {Layer} layer */
const seriesLooked = (layer) => !layer.from && !placedOn(layer) && !heatLike(layer);
const LINE_STYLE = { Solid: 0, Dotted: 1, Dashed: 2, LargeDashed: 3, SparseDotted: 4 };
const LINE_TYPE = { Simple: 0, WithSteps: 1, Curved: 2 };
/** @type {[string, string, number][]} */
const SHAPES = [["straight", "Straight", LINE_TYPE.Simple], ["steps", "Steps", LINE_TYPE.WithSteps], ["curved", "Curved", LINE_TYPE.Curved]];
const lineTypeOf = (layer) => SHAPES.find(([id]) => id === layer.shape)?.[2] ?? LINE_TYPE.Simple;
const PAIRS = { baseline: ["Above", "Below"], histogram: ["Above", "Below"], candles: ["Rising", "Falling"], trend: ["Resistance", "Support"], signal: ["Up", "Down"], bubbles: ["Above zero", "Below zero"] };
/**
 * A baseline's are its two; candles' two, rising and falling (their one, a choice); four: bars', MACD's own per side, paler where a bar falls back toward the base.
 * @type {Record<string, string[]>}
 */
const COLOR_MODES = { line: ["one", "levels", "gradient", "from"], dots: ["one", "levels", "gradient", "from"], area: ["one", "levels", "gradient", "from"], histogram: ["one", "sides", "momentum", "levels", "gradient", "from"], baseline: ["sides"], trend: ["one", "sides", "gradient", "from"], cloud: ["layers", "one", "sides", "gradient", "from"], candles: ["one", "sides"], bubbles: ["sides", "one"], signal: ["one"] };
const COLORED_FROM = new Set(Object.keys(COLOR_MODES).filter((type) => COLOR_MODES[type].includes("from")));
const BY_LEVELS = new Set(Object.keys(COLOR_MODES).filter((type) => COLOR_MODES[type].includes("levels")));
/**
 * A cloud's two colors (which of its two is over) only between two layers alone (no "the other one" with a third); its layers' only where it has some. A heatmap row has its one color; a heatmap layer's cells take its pane's gradient.
 * @param {Layer} layer @param {Chart} doc
 */
const modesOf = (layer, doc) => (layer.type === "heatmap" ? [] : heatmapRow(doc, layer) ? ["one"] : COLOR_MODES[layer.type] ?? ["one"]).filter((mode) => layer.type !== "cloud" || (mode === "sides" ? cloudLayers(layer).length === 2 && !edgesOf(layer).length : mode !== "layers" || cloudLayers(layer).length > 0));
/** @type {[number, string, number, number][]} */
const DASHES = [
  [LINE_STYLE.Solid, "Solid", 14, 0],
  [LINE_STYLE.LargeDashed, "Long dashes", 6, 2],
  [LINE_STYLE.Dashed, "Dashed", 3, 2],
  [LINE_STYLE.Dotted, "Dotted", 2, 2],
  [LINE_STYLE.SparseDotted, "Sparse dots", 2, 4],
];
const dashOf = (/** @type {number | undefined} */ style) => DASHES.find(([dash]) => dash === style) ?? DASHES[0];
/** @type {[("" | "stack" | "percent" | "rows"), string, string][]} */
const STACKS = [["", "Off", "Each at its own value"], ["stack", "Stacked", "Each on the ones before"], ["percent", "100%", "As shares of them all"], ["rows", "Heatmap", "Each a row, colored by its value"]];
/** @param {Pane | undefined} pane */
const summed = (pane) => pane?.stack === "stack" || pane?.stack === "percent";
/** @param {Chart} doc @param {Pane} pane */
const rowLayers = (doc, pane, all = false) => ((rows) => (pane.reverse ? rows.reverse() : rows))(doc.layers.filter((layer) => layer.pane === pane.id && (all || !layer.hidden) && layer.type !== "candles" && layer.type !== "cloud" && !layer.from && !placedOn(layer) && !heatLike(layer)));
/**
 * Finer than a day is a point a pixel of the image: too many. Needs two layers of its own on one scale (two units don't fit).
 * @param {Chart} doc @param {Pane} pane
 */
const rowsUnfit = (doc, pane) => ((rows) => (FINE.has(doc.index) ? "A heatmap needs a daily or coarser interval" : rows.length < 2 ? "A heatmap needs two layers of their own in the pane" : rows.some((layer) => layer.side !== rows[0].side) ? "A heatmap needs the pane's layers on one scale" : ""))(rowLayers(doc, pane, true));
/** @param {Chart} doc @param {Pane} pane */
const asRows = (doc, pane) => pane.stack === "rows" && !rowsUnfit(doc, pane);
/** @param {Chart} doc @param {Pane | undefined} pane */
const drawnInto = (doc, pane) => (pane && asRows(doc, pane) ? "a heatmap" : summed(pane) ? "a stack" : "");
/** @param {Chart} doc @param {Pane} pane */
const rowsLevel = (doc, pane) => ((rows) => rows[0]?.levels?.map((line) => line.value).find((value) => rows.every((layer) => layer.levels?.some((line) => line.value === value))))(rowLayers(doc, pane));
/** @param {Layer} layer @param {string | undefined} label */
const nameLayer = (layer, label) => {
  if (label) layer.label = label;
  else delete layer.label;
  delete layer.short;
};
/** @type {[boolean, string][]} */
const OUTLINES = [[false, "None"], [true, "Below the level"]];
/** @param {Chart} doc @param {Pane} pane */
const outlinable = (doc, pane) => asRows(doc, pane) && rowsLevel(doc, pane) !== undefined;
/** @param {Chart} doc @param {Pane} pane */
const gradable = (doc, pane) => (asRows(doc, pane) && rowsLevel(doc, pane) === undefined) || doc.layers.some((layer) => layer.pane === pane.id && layer.type === "heatmap" && !signedMeasure(layer));
/** @param {Pane | undefined} pane */
const paneGradient = (pane) => HEAT_GRADIENTS.find(([id]) => id === pane?.gradient) ?? HEAT_GRADIENTS[0];
/** @param {Chart} doc @param {Layer} layer */
const heatGradient = (doc, layer) => (signedMeasure(layer) ? null : paneGradient(doc.panes.find((pane) => pane.id === layer.pane))[2]);
/** @param {Chart} doc @param {Layer} layer */
const ownGradient = (doc, layer) => (layer.byGradient && layer.gradient && modesOf(layer, doc).includes("gradient") ? layer.gradient : null);
/** @param {Chart} doc @param {Layer} layer */
const gradientOfLayer = (doc, layer) => (layer.type === "heatmap" ? heatGradient(doc, layer) : ownGradient(doc, layer));
/** @param {Chart} doc @param {Layer} layer */
const heatmapRow = (doc, layer) => ((pane) => Boolean(pane && asRows(doc, pane) && rowLayers(doc, pane).some((row) => row.id === layer.id)))(doc.panes.find((pane) => pane.id === layer.pane));
const MODES = [
  { chip: "Linear", name: "Linear", tip: "Linear" },
  { chip: "Log", name: "Log", tip: "Logarithmic" },
  { chip: "Percent", name: "Percent", tip: "Change since the first value in view" },
  { chip: "Indexed", name: "Indexed", tip: "Indexed to 100 at the first value in view" },
];
const WIDTHS = Array.from({ length: 10 }, (_, i) => (i + 1) / 2);
const OPACITIES = Array.from({ length: 20 }, (_, i) => (i + 1) / 20);
const defaultOpacity = (/** @type {{ type?: string }} */ layer) => (layer.type === "cloud" ? 0.2 : 1), opacityOf = (/** @type {Layer} */ layer) => layer.opacity ?? defaultOpacity(layer);
/**
 * A distribution has many values per date, rather than a temporal measurement to average.
 * @param {Chart} doc @param {Pane} pane
 */
const averageBlocked = (doc, pane) => doc.layers.some((layer) => layer.pane === pane.id && heatLike(layer)) ? "This pane contains a heatmap or profile (including hidden layers): it needs raw values" : "";
const failing = (/** @type {string} */ detail, /** @type {string} */ short) => Object.assign(new Error(detail), { short });
/**
 * @typedef {{ id: string, letter: string, series?: string, formula?: string, label?: string, short?: string, maxInterval?: string, color: string,
 *   type: string, width: number, dash: number, pane: string, side: string, hidden?: boolean, priceLine?: boolean, outline?: true, fit?: false | "zero", size?: number, hide?: number, scaleTo?: "all", colorBelow?: string, colors?: 1 | 4, colorFading?: string, colorBelowFading?: string, opacity?: number,
 *   format?: string, base?: number, path?: string, paths?: Record<string, string>,
 *   shape?: string, levels?: Level[], fill?: string, fillOpacity?: number, solid?: true, hollow?: "none", place?: string, results?: boolean, baseLine?: { color?: string, dash?: number, line?: false, tag?: boolean }, underName?: string, from?: string[], ref?: string, colorFrom?: string, byLevels?: boolean, byLayers?: boolean,
 *   heatmap?: string, weight?: string, measure?: string, rows?: number, profile?: string[], cumulative?: true, cohortColors?: Record<string, string>, gradient?: [number, string][], upToDate?: true, byGradient?: true }} Layer
 *   Made from layers (`from`: the ids of the layers it's made of, none itself made from layers), by its type: a cloud
 *   ("cloud") is filled between its two, at each date (in two colors, `color` where the first is over the second,
 *   `colorBelow` where the second is, parting where they cross); a trend ("trend") is on the lowest of two or more
 *   or the highest, flipping when its reference (`ref`: a layer's id, one of one value, not one of them) touches it
 *   (see trendOf). Short of what it needs, it says so.
 *   A signal ("signal"): a formula read as a state, up (above 0), down (below) or off (0; a condition is up where
 *   true), marked on another layer (`ref`, the first that can carry marks if none) where it turns up or down, `place`
 *   (above it, unset; below; on it: an up mark's, a down one's mirrored), or shaded while up or down ("shade"); up
 *   in its color, down in its second (`colorBelow`, red unset). `results`: each stretch's result on the mark that ends it (see
 *   stretchResult).
 *   Bubbles ("bubbles"): its values as bubbles on another layer (`ref`, the first that can carry them if none;
 *   none, "": along its pane's middle), each date's at that layer's value there, its area its own value's size (see
 *   Bubbles): `size` the largest's diameter, `hide` the share of it under which none is drawn, `scaleTo` "all":
 *   sized against all of its history (unset: what's in view); above zero in its color, below in its second
 *   (`colorBelow`, red unset; `colors` 1, its one).
 *   `short`: its name where room is tight (a heatmap's row), as its set says it (an age band's "1w-1m"); none, its label.
 *   `fit` "zero": a linear axis includes zero; false: its axis framed without it (see fittable), drawn where its values are, past the plot's edge if
 *   they're far: a spike, a far tail.
 *   `colorFrom`: another layer's id, whose colors it takes date by date (see colorsAt): a line's, dots', an area's,
 *   bars', a cloud's or a trend's (not a baseline's, nor candles').
 *   `byLevels`: colored by its levels instead: above each, that level's color; under them all, its own (a line's,
 *   dots', an area's or bars'; never with `colorFrom`). A line or an area by height, parted where it crosses a level
 *   (stacked, at its total, date by date); dots and bars each by their own value.
 *   `byGradient`: colored by its value instead, date by date, along its `gradient`: its stops (2 to 4, each where it
 *   is from 0% to 100% of the way from its lowest value to its highest, and its color), mixed between them; past its
 *   outer ones, theirs. Its lowest and highest of all its history (at a fine interval, of what it holds), or
 *   (`upToDate`) of its history up to each date (no hindsight). Its gradient kept while it's colored otherwise (a
 *   heatmap's is its pane's: see heatGradient). Never with `colorFrom`, `byLevels`, `byLayers`.
 *   `byLayers`: a cloud's, colored by its layers: each one's own where it's over, following them (its `color` and
 *   `colorBelow` theirs, see layersColors; never with `colorFrom`).
 *   `colors`: bars' (their two, above and below their base, unset): 1, their one; 4, MACD's, each side's own where a
 *   bar grows from the base, fainter where it fades back toward it (`colorFading` above, `colorBelowFading` below;
 *   unset, each side's color fainter: see fadingOf). A trend's or a cloud's 1: its one, its second (`colorBelow`) kept. Colors
 *   picked stay, whichever way it's colored.
 * @typedef {{ value: number, color?: string, dash?: number, line?: false, tag?: boolean, name?: string }} Level A line
 *   across at a value: its color (else its layer's) and its line style (else long dashes; `line` false, none: its band
 *   alone); its value on the axis only with `tag`; `name`, what a value from it up (to the next level) is called, read
 *   in its color in place of the number (see bandOf; under them all, the layer's `underName`). A baseline's base draws
 *   one too (`baseLine`: gray, in sparse dots).
 * @typedef {{ id: string, mode: number, size?: number, stack?: "stack" | "percent" | "rows", reverse?: boolean, average?: string, outline?: true, gradient?: string, lens?: Lens }} Pane `reverse`:
 *   its last layer in front (in a stack, at its foot); else its first, as the panel lists them. `outline`: a heatmap's
 *   region under its rows' level outlined (see rowsLevel), where that boundary is the question. `gradient`: its
 *   heatmap's (its rows without a level, a heatmap layer of an amount), one of HEAT_GRADIENTS' ids (none: the first)
 *   `lens`: its two views ("Show as"), see Lens: what it shows, one of them; the other kept aside in it.
 * @typedef {{ as: [string, string], at: 0 | 1, other: Omit<Pane, "id" | "size" | "lens"> & { layers: Layer[], drawings?: Drawing[] } }} Lens
 *   A pane's two ways of showing its subject (a pair and its net, an amount and its dollars), each a view: its layers
 *   and its own scale, stack, order and average. `as`: their names, in order; `at`: the one shown, whose layers are the
 *   chart's own and whose fields the pane's; `other`: the other, whole, out of the chart (nothing draws, lists or
 *   fetches it) till it's shown (see showView). Its ids and letters are none of the chart's.
 * @typedef {{ id: string, name: string, parent?: string, closed?: true, icon?: string }} Folder A folder of charts and folders, in
 *   `parent` (none: at the top), as deep as wanted; `closed`, what's in it out of sight.
 * @typedef {{ id: string, title: string, index: string, range: string, panes: Pane[], layers: Layer[], drawings?: Drawing[], halvings?: true, legend?: "off", folder?: string, preset?: string, base?: string, blank?: true, updated: number }} Chart
 * @typedef {"trend" | "path" | "hline" | "vline" | "rect" | "ellipse" | "text" | "brush" | "ruler"} DrawingKind
 * @typedef {{ id: string, kind: DrawingKind, layer: string, points: [number, number][], color: string, width: number, dash: number,
 *   opacity?: number, start?: End, end?: End, text?: string, label?: string, at?: number, size?: number, back?: string }} Drawing
 *   Drawn on a layer,
 *   on its scale: each point a time (unix
 *   seconds) and a value as the layer reads; it goes with its layer. A trend line's or a path's `start` and `end` (its
 *   first point's, its last's): an arrowhead there, or on past it to the plot's edge (see ENDS). A path: a line through
 *   two points or more, in their order. A text is a tag at its point (its middle), as a line's label is:
 *   one size whatever the zoom (`size`, a preset; a line's label's too). A line's `label` sits `at` a share of the
 *   way along it: a trend line's, from its first point to its second; a line across's, across the plot; down, down it.
 */
const letterAt = (n) => (n < 26 ? String.fromCharCode(65 + n) : String.fromCharCode(64 + Math.floor(n / 26)) + String.fromCharCode(65 + (n % 26)));
/** @param {Chart} doc */
function nextLetter(doc) {
  // Letters a formula still mentions stay taken: a new layer never silently becomes what the formula reads (a view kept aside's too).
  const layers = [...doc.layers, ...asideLayers(doc)], used = new Set(layers.map((layer) => layer.letter));
  for (const layer of layers) for (const [ref] of (layer.formula ?? "").matchAll(/(?<![A-Za-z0-9_])[A-Z]{1,2}(?![A-Za-z0-9_])/g)) used.add(ref);
  for (let n = 0; ; n++) if (!used.has(letterAt(n))) return letterAt(n);
}
/** @param {Chart} doc @returns {Layer[]} */
const asideLayers = (doc) => doc.panes.flatMap((pane) => pane.lens?.other.layers ?? []);
/**
 * Its size is the slot's, kept: nothing moves.
 * @param {Chart} doc @param {string} id @param {0 | 1} at
 */
function showView(doc, id, at) {
  const pane = doc.panes.find((item) => item.id === id), lens = pane?.lens;
  if (!pane || !lens || lens.at === at) return;
  const { id: _id, size, lens: _lens, ...fields } = pane, { layers, drawings = [], ...other } = lens.other;
  const leaving = doc.layers.filter((layer) => layer.pane === id), gone = new Set(leaving.map((layer) => layer.id));
  const place = Math.max(0, doc.layers.findIndex((layer) => layer.pane === id)), carried = (doc.drawings ?? []).filter((drawing) => gone.has(drawing.layer));
  doc.layers = doc.layers.filter((layer) => layer.pane !== id);
  doc.layers.splice(place, 0, ...layers.map((layer) => ({ ...layer, pane: id })));
  doc.drawings = [...(doc.drawings ?? []).filter((drawing) => !gone.has(drawing.layer)), ...drawings];
  if (!doc.drawings.length) delete doc.drawings;
  for (const key of Object.keys(fields)) delete /** @type {Record<string, any>} */ (pane)[key];
  Object.assign(pane, other, { lens: { as: lens.as, at, other: { ...fields, layers: leaving, ...(carried.length ? { drawings: carried } : {}) } } });
}
/** @param {Chart} doc */
function nextColor(doc) {
  const used = doc.layers.map((layer) => layer.color);
  return PALETTE.find((color) => !used.includes(color)) ?? PALETTE[doc.layers.length % PALETTE.length];
}
/** @param {Partial<Layer>} spec @param {Chart} doc @returns {Layer} */
function makeLayer(spec, doc) {
  return { id: uid(), letter: nextLetter(doc), color: nextColor(doc), type: "line", width: 1.5, dash: 0, pane: doc.panes[0]?.id ?? "", side: "right", ...spec };
}
const EDGES = ["top", "bottom"];
const edgesOf = (/** @type {{ from?: string[] }} */ layer) => EDGES.filter((edge) => layer.from?.includes(edge));
/** @param {Chart} doc @param {Iterable<string>} members */
const membersOf = (doc, members, cloud = false) => [...new Set(members)].filter((id) => (cloud && EDGES.includes(id)) || doc.layers.some((item) => item.id === id && !item.from && !placedOn(item) && !heatLike(item)));
/** @param {Chart} doc @param {Iterable<string>} members */
const fromOf = (doc, members, type = "") => ((ids) => (type === "cloud" ? ids : doc.layers.filter((item) => ids.includes(item.id)).map((item) => item.id)))(membersOf(doc, members, type === "cloud"));
/** @param {{ from?: string[] }} layer */
const cloudLayers = (layer) => (layer.from ?? []).filter((id) => !EDGES.includes(id));
/** @param {Chart} doc @param {Iterable<string>} members @param {string} member */
const toggleMember = (doc, members, member) => ((set) => (set.has(member) ? set.delete(member) : set.add(member), [...doc.layers.map((layer) => layer.id), ...EDGES].filter((id) => set.has(id))))(new Set(members));
/** @param {Layer} a @param {Layer} b */
const oneScale = (a, b) => a.pane === b.pane && a.side === b.side;
/** @param {Chart} doc @param {{ from?: string[] }} layer */
const placedBy = (doc, layer) => doc.layers.find((other) => other.id === layer.from?.find((id) => !EDGES.includes(id))) ?? null;
/** @param {Layer | null | undefined} layer */
const goesWith = (layer) => (layer && placedOn(layer) ? (layer.ref ? [layer.ref] : []) : (layer?.from ?? []).filter((id) => !EDGES.includes(id)));
/** @param {Layer | null | undefined} layer */
const follows = (layer) => goesWith(layer).length > 0;
/** @param {Chart} doc @param {Layer} layer */
const offScale = (doc, layer) => [...(layer.from ?? []), ...(layer.type === "trend" && layer.ref ? [layer.ref] : [])].some((id) => ((other) => other && !oneScale(other, layer))(doc.layers.find((item) => item.id === id)));
const TREND_FLIP = "Needs a third layer, such as a price, to flip it", ONE_SCALE = "Not on one scale", FLIP_APART = "What flips it is on another scale";
const MAX_PANES = 12, MAX_LEVELS = 12;
const FORMULA_MAX = 2000;
/** @param {Chart} doc */
function newPaneIn(doc) {
  if (doc.panes.length >= MAX_PANES) return doc.panes[doc.panes.length - 1];
  const pane = { id: uid(), mode: 0 };
  doc.panes.push(pane);
  return pane;
}
/** @returns {Chart} */
function blankChart(title = "") {
  return { id: uid(), title, index: "day1", range: DEFAULT_RANGE, panes: [{ id: uid(), mode: 1 }], layers: [], updated: Date.now() };
}
// Pools mempool renamed (old slug, new): key and series ids both change, so neither finds the other (see app.js's followPool).
const POOLS_RENAMED = /** @type {[string, string][]} */ ([["btccom", "cloverpool"]]);
const poolSeries = (/** @type {string} */ name) => POOLS_RENAMED.reduce((n, [from, to]) => (n.startsWith(`pool_${from}_`) ? `pool_${to}_${n.slice(from.length + 6)}` : n), name);
const poolPath = (/** @type {string} */ path) => POOLS_RENAMED.reduce((p, [from, to]) => (p.startsWith(`pools.${from}.`) ? `pools.${to}.${p.slice(from.length + 7)}` : p), path);

export {
  asideLayers, asRows, automaticZero, averageBlocked, belowOf, blankChart, BUBBLE_HIDE, BUBBLE_HIDES, BUBBLE_SIZE, BUBBLE_SIZES, BY_LEVELS,
  cloudLayers, COLOR_MODES, drawnInto, COLORED_FROM, DASHES, dashOf, DEFAULT_MEASURE, DEFAULT_ROWS, defaultOpacity, EDGES, edgesOf,
  failing, fittable, FLIP_APART, follows, FORMULA_MAX, fromOf, goesWith, GRADIENT_LEAST, GRADIENT_MOST, gradable, gradientOfLayer, heatGradient, HEAT_GRADIENTS,
  HEAT_MEASURES, HEAT_ROWS, HEAT_WEIGHTS, heatLike, hideText, hollowOf, kindColor, kindLook, layersColors,
  letterAt, LINE_STYLE, LINE_TYPE, LINED, lineTypeOf, LOOK_KEYS, makeLayer, MARK_PLACES, markable, MARKER, MAX_LEVELS,
  MAX_PANES, measured, measureOf, measureUnit, membersOf, MODES, modesOf, nameLayer, newPaneIn, nextColor, nextLetter, offScale,
  ONE_SCALE, oneScale, OPACITIES, ownGradient, opacityOf, OUTLINED, OUTLINES, outlinable, PAIRS, paneGradient, placedBy, placedOn, placeName, poolPath, poolSeries,
  PROFILE_VIEWS, resultText, rowLayers, rowsLevel, rowsUnfit, seriesLooked, shapeKey, SHAPES, showView, signedMeasure, STACKS, stretchResult, summed, toggleMember,
  TREND_FLIP, TREND_SAYS, TWO_KEPT, TWO_OR_ONE, twoSided, typeName, TYPES, unfitted, WIDTHS,
};
