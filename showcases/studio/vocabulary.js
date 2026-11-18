// @ts-check
// A chart's document and its words: its shape, layer kinds and looks, marks, measures, panes, how layers are made from
// layers, a new layer, a blank chart.
import { precise } from "/studio/format.js";
import { DEFAULT_RANGE } from "/studio/intervals.js";
import { uid } from "/studio/math.js";
import { PALETTE } from "/studio/color.js";
/** @import { Result } from "/studio/chart.js" */
/** @import { End } from "/studio/tools.js" */

/** The types whose second color is kept as picked, one color or two: a trend's (on a support), a cloud's (its second over). */
const TWO_KEPT = new Set(["trend", "cloud"]);
/** The types in two colors, rising and falling (above and below zero), that can be in one (`colors: 1`): candles, bubbles. */
const TWO_OR_ONE = new Set(["candles", "bubbles"]);
/** Whether a layer's candles are hollow, as usual: hollow when rising, filled when falling (else all filled). @param {Layer} layer */
const hollowOf = (layer) => layer.hollow !== "none";
/** A layer's second color: a baseline's or bars' under their base (red, unset; bars of one color, theirs); a trend's on a support, a cloud's where its second is over (its own, unset). */
const belowOf = (layer) => ((layer.type === "histogram" || TWO_OR_ONE.has(layer.type) || TWO_KEPT.has(layer.type)) && layer.colors === 1 ? layer.color : layer.colorBelow ?? (TWO_KEPT.has(layer.type) ? layer.color : "red"));
/**
 * A cloud colored by its layers, its colors theirs: its first one's where that's over, its second's where that is
 * (to an edge, its one layer's; in its edit, and as every edit's chart is normalized: they follow).
 * @param {Layer} layer @param {Chart | null} [doc]
 */
const layersColors = (layer, doc) => {
  const [first, second] = (layer.from ?? []).flatMap((id) => ((item) => (item ? [item] : []))(doc?.layers.find((one) => one.id === id)));
  if (first) layer.color = first.color;
  if (second) layer.colorBelow = second.color;
  else delete layer.colorBelow;
};
const LOOK_KEYS = /** @type {const} */ (["color", "colorBelow", "colorFrom", "byLevels", "byGradient"]);
/** A holder group's URPD: a heatmap (its days as columns) or a profile (a day's, at the chart's right edge). @param {{ type?: string }} layer */
const heatLike = (layer) => layer.type === "heatmap" || layer.type === "profile";
/**
 * Whether a layer reads two ways, over and under (a baseline over its base, a heatmap's unrealized P&L in profit or
 * in loss): green over red by default (see kindLook). @param {{ type?: string, measure?: string }} layer
 */
const twoSided = (layer) => layer.type === "baseline" || (layer.type === "heatmap" && signedMeasure(layer));
/** A binned layer's measure: its coins (the default), their worth, or their unrealized P&L. @param {{ measure?: string }} layer */
const measureOf = (layer) => layer.measure ?? DEFAULT_MEASURE;
/** Whether a binned layer's amounts can be negative: its unrealized P&L's. @param {{ measure?: string }} layer */
const signedMeasure = (layer) => measureOf(layer) === "pnl";
/** A binned layer's amounts' unit, by its measure (see measureOf): coins, else dollars. @param {string} measure */
const measureUnit = (measure) => (measure === DEFAULT_MEASURE ? "btc" : "usd");
/**
 * A heatmap of an amount's steps, as the website drew heat: inferno, its 4 colors that mixed (see gradientTable)
 * come nearest it (0.023 apart in OKLab at most, at 64 places: about what an eye tells apart).
 */
const HEAT_GRADIENT = /** @type {[number, string][]} */ ([[0, "#000004"], [20, "#420a68"], [81, "#fcaa15"], [100, "#fcffa4"]]);
/** A layer's kind, as its first look goes (see kindLook): two-sided, a heatmap of an amount, or neither. @param {{ type?: string, measure?: string }} layer */
const kindOf = (layer) => (twoSided(layer) ? "sided" : layer.type === "heatmap" ? "heat" : "");
/**
 * A layer's own first color, where its kind has one (else the next free, see nextColor): two-sided, green (red its
 * own below); a heatmap's, orange (its gradient's off). @param {{ type?: string, measure?: string }} layer
 */
const kindColor = (layer) => ({ sided: "green", heat: "orange" })[kindOf(layer)] ?? null;
/**
 * The look a layer's kind starts it with, onto `layer` (as `next`, its kind, reads): its color (kindColor); two-sided,
 * its own red below, no gradient; a heatmap of an amount, colored by its gradient (its own kept, else HEAT_GRADIENT's).
 * @param {Layer} layer @param {{ type?: string, measure?: string }} next
 */
function kindLook(layer, next) {
  const color = kindColor(next), kind = kindOf(next);
  if (color) layer.color = color;
  if (kind === "sided") delete layer.colorBelow, delete layer.byGradient;
  if (kind === "heat") (layer.byGradient = true), (layer.gradient ??= structuredClone(HEAT_GRADIENT));
}
/** A layer's shape, as its look follows it: its type, a heatmap's P&L its own (see reshape). @param {Layer} layer */
const shapeKey = (layer) => (layer.type === "heatmap" && layer.measure === "pnl" ? "heatmap:pnl" : layer.type);
/** The types of a layer made from layers. @type {[string, string][]} */
const FROM_TYPES = [["cloud", "Cloud"], ["trend", "Trend"]];
/** What a trend makes of its layers, said. */
const TREND_SAYS = "Follows the lowest until the layer that flips it touches it, then the highest until that layer touches it, and so on";
/** What a signal's marks can be on: a layer of one value drawn as itself (not a cloud, not a signal). @param {Chart} doc */
const markable = (doc) => doc.layers.filter((other) => other.type !== "cloud" && !placedOn(other) && !heatLike(other));
/**
 * How a signal shows: marks where it turns up (a condition: true), over its layer, under it or on it, a down one
 * mirrored; or, shaded, its stretches up or down, its layer's pane's height.
 */
const MARK_PLACES = /** @type {[string, string, string][]} */ ([["above", "Above", "above"], ["below", "Below", "below"], ["on", "On it", "on"], ["shade", "Shaded", ""]]);
/** A place's name as a choice says it: "Above the layer", "Shaded". @param {string} place */
const placeName = (place) => ((text) => (place === "shade" ? "Shaded while up or down" : place === "on" ? "On the layer" : `${text} the layer`))(MARK_PLACES.find(([id]) => id === place)?.[1] ?? "");
/**
 * What a stretch a change ends made, by the layer it's marked on: that layer's change over it, up; the opposite, down
 * (the state was right: more); none for one of neither, one begun before a change, or a value there not above 0 (or
 * not finite: a formula's division by 0).
 * @param {{ at: number, was: number, since: number }} change @param {Result | null | undefined} reference
 */
const stretchResult = (change, reference) => {
  const values = reference && measured(reference);
  if (!values) return undefined;
  const from = values[change.since], to = values[change.at];
  return change.was && change.since >= 0 && from > 0 && to > 0 && Number.isFinite(from) && Number.isFinite(to) ? change.was * (to / from - 1) : undefined;
};
/** Measurements belong to the result, even if its view shares a vector with another layer's formula. */
const measured = (/** @type {Result} */ result) => result.measurement ?? result.values;
/** A stretch's result, said: +10.2%, −4.1%; ten times or more, as a growth reads: 840×. */
const resultText = (/** @type {number} */ result) => (result >= 9 ? `${precise(1 + result)}×` : `${result < 0 ? "−" : "+"}${precise(Math.abs(result) * 100)}%`);
/** A layer's type, by its name: every type's (those drawn from a series, from layers, a signal, bubbles, a heatmap). */
const typeName = (/** @type {string} */ type) => [...TYPES, ...FROM_TYPES, ["signal", "Signal"], ["bubbles", "Bubbles"], ["heatmap", "Heatmap"], ["profile", "Profile"]].find(([id]) => id === type)?.[1] ?? type;
/** A heatmap's measures: what its cells say (each day's URPD has all three: one switched, nothing's asked again), named, then short (its settings' tabs); the first, unsaid, the default. */
const HEAT_MEASURES = /** @type {[string, string, string][]} */ ([["supply", "Supply", "Supply"], ["capital", "Capital", "Capital"], ["pnl", "Unrealized P&L", "P&L"]]), DEFAULT_MEASURE = HEAT_MEASURES[0][0];
/** A heatmap's weights, the URPD's: every coin as it is, or each age band's by its wakefulness (Cointime's) or mobility (Coinflow's). */
const HEAT_WEIGHTS = /** @type {[string, string][]} */ ([["", "Every coin"], ["wakefulness", "Awake"], ["mobility", "Mobile"]]);
/** A profile's views: its bars, or its amounts summed from the close outward. */
const PROFILE_VIEWS = /** @type {[string, string][]} */ ([["bars", "Bars"], ["cumulative", "Cumulative"]]);
/** A heatmap's densities, the URPD's: rows per ×10 in price, on a log scale or a linear one; one unsaid, the default's. */
const HEAT_ROWS = [100, 200, 500], DEFAULT_ROWS = 200;
/** A gradient's colors at most, at least (read back with a chart, at the start: see gradientOf). */
const GRADIENT_MOST = 4, GRADIENT_LEAST = 2;
/**
 * A series' looks, its Type's choices. Those made of other layers (a cloud, a trend, a signal, bubbles on a layer) are
 * kinds of their own, added as such (Add a cloud, As bubbles on…), not a look to switch to.
 * @type {[string, string][]}
 */
const TYPES = [["line", "Line"], ["dots", "Dots"], ["area", "Area"], ["histogram", "Bars"], ["baseline", "Baseline"], ["candles", "Candles"]];
/** The types drawn as a line: a dash and a shape apply. */
const LINED = new Set(["line", "area", "baseline", "trend"]);
/** The types an outline edges (the chart's background about what's drawn: the library's `outline`): what a series is drawn as, and a trend's line. */
const OUTLINED = new Set(["line", "dots", "area", "histogram", "baseline", "candles", "trend", "bubbles"]);
/** The crosshair's dot on a line: its radius and its edge's width (a lit bubble's ring clears it, see lightBubble). */
const MARKER = { radius: 3, border: 2 };
/**
 * Bubbles' largest, across (8 to 48 pixels by 4, 24 if none), and the share of it under which none is drawn (none
 * to 25% by 1%, 5% if none): their sliders' stops.
 */
const BUBBLE_SIZES = Array.from({ length: 11 }, (_, i) => 8 + 4 * i), BUBBLE_SIZE = 24, BUBBLE_HIDES = Array.from({ length: 26 }, (_, i) => i / 100), BUBBLE_HIDE = 0.05;
/** A share under which bubbles aren't drawn, in words. */
const hideText = (/** @type {number} */ share) => (share ? `${Math.round(share * 100)}%` : "None");
/** Whether a layer's values aren't drawn at their own height on its scale: a signal's (marks on its layer), bubbles' (sized on theirs). @param {Layer} layer */
const placedOn = (layer) => layer.type === "signal" || layer.type === "bubbles";
/**
 * Whether a layer can stay out of its axis' fit (`fit` false): one whose values widen its scale (not a heatmap's
 * rows, marks or bubbles on another layer, nor a cloud on a scale of its own, from its pane's edge to edge).
 * @param {Layer} layer
 */
const fittable = (layer) => !placedOn(layer) && !heatLike(layer) && !(layer.type === "cloud" && edgesOf(layer).length === 2 && !cloudLayers(layer).length);
/** A visible layer whose values participate in its pane's framing. @param {Layer} layer */
const framesAxis = (layer) => !layer.hidden && layer.fit !== false && (heatLike(layer) || fittable(layer));
/**
 * Whether a layer's linear axis is framed automatically, by its series' role: its axis' only framing layer, fitted,
 * not stacked (explicit fitting and stacks already state their rule).
 * @param {Chart} doc @param {Layer} layer @param {boolean} stacked
 */
const automaticZero = (doc, layer, stacked) => {
  if (stacked || layer.fit === "zero" || !framesAxis(layer) || !(layer.series || layer.formula)) return false;
  return doc.layers.every((other) => other.id === layer.id || !framesAxis(other) || other.pane !== layer.pane || other.side !== layer.side);
};
/**
 * Whether a layer's out of its axis' fit as drawn (the chart's, the previews'): its `fit` false, where another shown
 * on its pane's side frames that axis (one fitted, or a heatmap: alone, on the price's closes). None: it is.
 * @param {Chart} doc @param {Layer} layer
 */
const unfitted = (doc, layer) => layer.fit === false && doc.layers.some((other) => other !== layer && framesAxis(other) && other.pane === layer.pane && other.side === layer.side);
/** Whether a layer has a series' look, its Type one of TYPES: none made from layers, placed on one, or a heatmap. @param {Layer} layer */
const seriesLooked = (layer) => !layer.from && !placedOn(layer) && !heatLike(layer);
/** A line's dashes (a layer's `dash`, a drawing's): solid, dotted, dashed, long dashes, sparse dots. */
const LINE_STYLE = { Solid: 0, Dotted: 1, Dashed: 2, LargeDashed: 3, SparseDotted: 4 };
/** How a line runs between its points (a layer's `shape`): straight, in steps, curved. */
const LINE_TYPE = { Simple: 0, WithSteps: 1, Curved: 2 };
/** How a line runs between its points: straight, in steps (a value that holds until it changes), or curved. @type {[string, string, number][]} */
const SHAPES = [["straight", "Straight", LINE_TYPE.Simple], ["steps", "Steps", LINE_TYPE.WithSteps], ["curved", "Curved", LINE_TYPE.Curved]];
const lineTypeOf = (layer) => SHAPES.find(([id]) => id === layer.shape)?.[2] ?? LINE_TYPE.Simple;
/** A type's two colors, where it has two: what each is called. */
const PAIRS = { baseline: ["Above", "Below"], histogram: ["Above", "Below"], candles: ["Rising", "Falling"], trend: ["Resistance", "Support"], signal: ["Up", "Down"], bubbles: ["Above zero", "Below zero"] };
/**
 * How a layer's colors can be chosen, by its type (see colorModeOf): one color; two ("sides": a baseline's or bars'
 * above and below their base, a trend's on resistance and on support, a cloud's by which of its two is over); a
 * cloud's its layers' own ("layers", see layersColors); four (bars', MACD's: each side's own, paler
 * where a bar falls back toward the base); by its levels; from another layer, date by date. A baseline's are its two,
 * candles' two, rising and falling (their one, a choice).
 * @type {Record<string, string[]>}
 */
const COLOR_MODES = { line: ["one", "levels", "gradient", "from"], dots: ["one", "levels", "gradient", "from"], area: ["one", "levels", "gradient", "from"], histogram: ["one", "sides", "momentum", "levels", "gradient", "from"], baseline: ["sides"], trend: ["one", "sides", "gradient", "from"], cloud: ["layers", "one", "sides", "gradient", "from"], candles: ["one", "sides"], bubbles: ["sides", "one"], signal: ["one"], heatmap: ["one", "gradient"] };
/** The types that can take their colors from another layer, and those that can be colored by their levels. */
const COLORED_FROM = new Set(Object.keys(COLOR_MODES).filter((type) => COLOR_MODES[type].includes("from")));
const BY_LEVELS = new Set(Object.keys(COLOR_MODES).filter((type) => COLOR_MODES[type].includes("levels")));
/**
 * A layer's ways of coloring: its type's, but a cloud's two colors (which of its two is over) only between two layers
 * alone (an edge, or a third: no "the other one"), and its layers' only where it has some (its pane's two edges alone
 * have none). @param {Layer} layer
 */
const modesOf = (layer) => (COLOR_MODES[layer.type] ?? ["one"]).filter((mode) => layer.type !== "cloud" || (mode === "sides" ? cloudLayers(layer).length === 2 && !edgesOf(layer).length : mode !== "layers" || cloudLayers(layer).length > 0));
/**
 * The chart's line styles, lightest to most broken up: each with its name, and its dash and gap as drawn on whole
 * pixels at 16×12 (in the proportions the chart dashes them).
 * @type {[number, string, number, number][]}
 */
const DASHES = [
  [LINE_STYLE.Solid, "Solid", 14, 0],
  [LINE_STYLE.LargeDashed, "Long dashes", 6, 2],
  [LINE_STYLE.Dashed, "Dashed", 3, 2],
  [LINE_STYLE.Dotted, "Dotted", 2, 2],
  [LINE_STYLE.SparseDotted, "Sparse dots", 2, 4],
];
const dashOf = (/** @type {number | undefined} */ style) => DASHES.find(([dash]) => dash === style) ?? DASHES[0];
/** A pane's stack: none, its layers one on another, or as shares of their sum. @type {[("" | "stack" | "percent"), string, string][]} */
const STACKS = [["", "Off", "Each at its own value"], ["stack", "Stacked", "Each on the ones before"], ["percent", "100%", "As shares of them all"]];
/** A pane's scales, by mode: its short name (the pane's chip, the inspector's tab), the menu's name, and the tab's tip. */
const MODES = [
  { chip: "Linear", name: "Linear", tip: "Linear" },
  { chip: "Log", name: "Log", tip: "Logarithmic" },
  { chip: "Percent", name: "Percent", tip: "Change since the first value in view" },
  { chip: "Indexed", name: "Indexed", tip: "Indexed to 100 at the first value in view" },
];
/** Line widths in pixels, by halves up to 5: the width slider's stops (the chart draws halves crisply on dense screens). */
const WIDTHS = Array.from({ length: 10 }, (_, i) => (i + 1) / 2);
/** Opacities, by 5%: the opacity sliders' stops, a layer's, a drawing's and an area's fill's (each kind's default among them, see fillOf). */
const OPACITIES = Array.from({ length: 20 }, (_, i) => (i + 1) / 20);
/** A layer's opacity: its own, else its type's (a cloud's fill a fifth, under what it spans; solid at 100%). */
const defaultOpacity = (/** @type {{ type?: string }} */ layer) => (layer.type === "cloud" ? 0.2 : 1), opacityOf = (/** @type {Layer} */ layer) => layer.opacity ?? defaultOpacity(layer);
/** A distribution has many values per date, rather than a temporal measurement to average.
 * @param {Chart} doc @param {Pane} pane
 */
const averageBlocked = (doc, pane) => doc.layers.some((layer) => layer.pane === pane.id && heatLike(layer)) ? "This pane contains a heatmap or profile (including hidden layers): it needs raw values" : "";
/** An error said short where it's glanced at (the legend, a row: what to do), and in full where there's room. */
const failing = (/** @type {string} */ detail, /** @type {string} */ short) => Object.assign(new Error(detail), { short });
/**
 * @typedef {{ id: string, letter: string, series?: string, formula?: string, label?: string, maxInterval?: string, color: string,
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
 *   (`upToDate`) of its history up to each date (no hindsight). A heatmap's: by its cells' strength (see
 *   heatColor). Its gradient kept while it's colored otherwise. Never with `colorFrom`, `byLevels`, `byLayers`.
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
 * @typedef {{ id: string, mode: number, size?: number, stack?: "stack" | "percent", reverse?: boolean, average?: string }} Pane `reverse`:
 *   its last layer in front (in a stack, at its foot); else its first, as the panel lists them
 * @typedef {{ id: string, name: string, parent?: string, closed?: true, icon?: string }} Folder A folder of charts and folders, in
 *   `parent` (none: at the top), as deep as wanted; `closed`, what's in it out of sight.
 * @typedef {{ id: string, title: string, index: string, range: string, panes: Pane[], layers: Layer[], made?: string, drawings?: Drawing[], halvings?: true, legend?: "off", folder?: string, preset?: string, base?: string, blank?: true, updated: number }} Chart
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
  // Letters still mentioned by a formula stay taken: a new layer never silently becomes what a formula reads.
  const used = new Set(doc.layers.map((layer) => layer.letter));
  for (const layer of doc.layers) for (const [ref] of (layer.formula ?? "").matchAll(/(?<![A-Za-z0-9_])[A-Z]{1,2}(?![A-Za-z0-9_])/g)) used.add(ref);
  for (let n = 0; ; n++) if (!used.has(letterAt(n))) return letterAt(n);
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
/** A cloud's edges, either of its two or both: its pane's top, its bottom (from the other up to there, or down to it). */
const EDGES = ["top", "bottom"];
const edgesOf = (/** @type {{ from?: string[] }} */ layer) => EDGES.filter((edge) => layer.from?.includes(edge));
/** Members as given that can be: layers there, none made from layers (a cloud's edges too), each once. @param {Chart} doc @param {Iterable<string>} members */
const membersOf = (doc, members, cloud = false) => [...new Set(members)].filter((id) => (cloud && EDGES.includes(id)) || doc.layers.some((item) => item.id === id && !item.from && !placedOn(item) && !heatLike(item)));
/**
 * What it's made from, as kept (membersOf): a cloud's all, as given (two, its colors say which one's over), a
 * trend's in the chart's order. @param {Chart} doc @param {Iterable<string>} members
 */
const fromOf = (doc, members, type = "") => ((ids) => (type === "cloud" ? ids : doc.layers.filter((item) => ids.includes(item.id)).map((item) => item.id)))(membersOf(doc, members, type === "cloud"));
/** A cloud's layers, its pane's edges aside. @param {{ from?: string[] }} layer */
const cloudLayers = (layer) => (layer.from ?? []).filter((id) => !EDGES.includes(id));
/**
 * Members with one ticked on or off, in the chart's order (a cloud's pane's edges after its layers): picked, they
 * read as the chart does. @param {Chart} doc @param {Iterable<string>} members @param {string} member
 */
const toggleMember = (doc, members, member) => ((set) => (set.has(member) ? set.delete(member) : set.add(member), [...doc.layers.map((layer) => layer.id), ...EDGES].filter((id) => set.has(id))))(new Set(members));
/** Whether two layers are on one scale (their pane, their side): a cloud or a trend reads its layers' values as one. @param {Layer} a @param {Layer} b */
const oneScale = (a, b) => a.pane === b.pane && a.side === b.side;
/** A cloud's or a trend's first layer (not an edge), none if it has none. @param {Chart} doc @param {{ from?: string[] }} layer */
const placedBy = (doc, layer) => doc.layers.find((other) => other.id === layer.from?.find((id) => !EDGES.includes(id))) ?? null;
/** The layers a layer goes with: a cloud's or a trend's (not its pane's edges), a signal's own; else none. @param {Layer | null | undefined} layer */
const goesWith = (layer) => (layer && placedOn(layer) ? (layer.ref ? [layer.ref] : []) : (layer?.from ?? []).filter((id) => !EDGES.includes(id)));
/** Whether a layer goes where its layers go: a cloud or a trend of any (one of its pane's edges alone, or of none, is put), a signal marked on one. @param {Layer | null | undefined} layer */
const follows = (layer) => goesWith(layer).length > 0;
/** Whether some of a cloud's or a trend's layers (what flips it too) are on another scale than it: their values not one's. @param {Chart} doc @param {Layer} layer */
const offScale = (doc, layer) => [...(layer.from ?? []), ...(layer.type === "trend" && layer.ref ? [layer.ref] : [])].some((id) => ((other) => other && !oneScale(other, layer))(doc.layers.find((item) => item.id === id)));
const TREND_FLIP = "Needs a third layer, such as a price, to flip it", ONE_SCALE = "Not on one scale", FLIP_APART = "What flips it is on another scale";
const MAX_PANES = 12, MAX_LEVELS = 12;
/** A formula's length at most: typed, as read from a link or a file. */
const FORMULA_MAX = 2000;
/** A new pane, or the last one once a chart has as many as it can hold. @param {Chart} doc */
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
/** Pools mempool renamed (old slug, new): their key and their series' ids both change, so neither finds the other (see app.js's followPool). */
const POOLS_RENAMED = /** @type {[string, string][]} */ ([["btccom", "cloverpool"]]);
const poolSeries = (/** @type {string} */ name) => POOLS_RENAMED.reduce((n, [from, to]) => (n.startsWith(`pool_${from}_`) ? `pool_${to}_${n.slice(from.length + 6)}` : n), name);
const poolPath = (/** @type {string} */ path) => POOLS_RENAMED.reduce((p, [from, to]) => (p.startsWith(`pools.${from}.`) ? `pools.${to}.${p.slice(from.length + 7)}` : p), path);

export {
  automaticZero, averageBlocked, belowOf, blankChart, BUBBLE_HIDE, BUBBLE_HIDES, BUBBLE_SIZE, BUBBLE_SIZES, BY_LEVELS,
  cloudLayers, COLOR_MODES, COLORED_FROM, DASHES, dashOf, DEFAULT_MEASURE, DEFAULT_ROWS, defaultOpacity, EDGES, edgesOf,
  failing, fittable, FLIP_APART, follows, FORMULA_MAX, fromOf, goesWith, GRADIENT_LEAST, GRADIENT_MOST, HEAT_GRADIENT,
  HEAT_MEASURES, HEAT_ROWS, HEAT_WEIGHTS, heatLike, hideText, hollowOf, kindColor, kindLook, kindOf, layersColors,
  letterAt, LINE_STYLE, LINE_TYPE, LINED, lineTypeOf, LOOK_KEYS, makeLayer, MARK_PLACES, markable, MARKER, MAX_LEVELS,
  MAX_PANES, measured, measureOf, measureUnit, membersOf, MODES, modesOf, newPaneIn, nextColor, nextLetter, offScale,
  ONE_SCALE, oneScale, OPACITIES, opacityOf, OUTLINED, PAIRS, placedBy, placedOn, placeName, poolPath, poolSeries,
  PROFILE_VIEWS, resultText, seriesLooked, shapeKey, SHAPES, signedMeasure, STACKS, stretchResult, toggleMember,
  TREND_FLIP, TREND_SAYS, TWO_KEPT, TWO_OR_ONE, typeName, TYPES, unfitted, WIDTHS,
};
