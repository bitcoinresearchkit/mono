// @ts-check
// A pane: a band of the chart, its own canvas, its scales (fitted to their series in view unless set), its series (lines,
// bands: drawn in the order added, the last on top), each on its right scale or its left one, its axis at the right (and
// the left one's, in a canvas of its own at the chart's left, once a series is on it). Drawn whole when a scale or its
// size changed; else, the view moved by whole pixels, its picture is moved and only the strip come into view drawn
// (drawn whole again once the motion stops: no seams left).

import { Scale } from "./scale.js";
import { Line } from "./line.js";
import { Band } from "./band.js";
import { Bars } from "./bars.js";
import { Candles } from "./candles.js";
import { Custom } from "./custom.js";
import { compact, kept } from "./format.js";
import { inkOn } from "./color.js";
import { Tags } from "./tags.js";

/**
 * A chart's look: its ground, its words' color, its grid's, its font; the crosshair's lines, and its badges' ground and
 * words; the line between panes.
 * @typedef {{ background: string, text: string, grid: string, font: string, crosshair: string, across: string, badge: string, separator: string }} Theme
 */

/** Drawing a pane whole slower than this (ms): a zoom stretches its picture instead (see draw). */
const SLOW = 8;
/** A price line's dots; none (made once). */
const DOTS = [2, 3], SOLID = /** @type {number[]} */ ([]);
/** A series' room at its scale's ends, as its margins write it (made once). */
const ROOM = new Float64Array(2);
/** A tag's height, and the least room between two (CSS pixels). */
const TAG = 18;
/**
 * What a pane draws: its lowest and highest over a range of indices (widening `out`), and its picture over part of the
 * plot; `version`, changed at each change of its values or look; `fixed`, what it draws holds its place as the view
 * moves (its pane drawn whole each frame, its picture never moved); `over`, drawn over everything, the crosshair too,
 * on a layer of its own drawn whole each frame (its pane's picture moved without it); `scaled: false`, not read on its
 * side's scale (its grid by the other's); `fit`, how it widens its scale's range in its place (a page's own reading:
 * none of it, a fixed range, its own and 0; none given, its extent); `margins`, room it keeps clear at its scale's ends
 * (CSS pixels over and under the range, by the points' spacing: its marks over bars). How far a point is from it: at
 * index `i` (and `x` CSS pixels into the plot), `y` from the pane's top.
 * @typedef {{
 *   version: number,
 *   hidden?: boolean,
 *   fixed?: boolean,
 *   over?: boolean,
 *   scaled?: boolean,
 *   fit?: ((a: number, b: number, positive: boolean, out: Float64Array) => void) | null,
 *   margins?(spacing: number, out: Float64Array): void,
 *   axisTags?(side: number, add: (y: number, text: string, color: string, ink?: string) => void): void,
 *   timeTags?(add: (x: number, text: string, color: string, ink?: string) => void): void,
 *   first(i: number): number,
 *   distance(i: number, y: number, scale: Scale, x: number, time: import("./timeline.js").Timeline): number,
 *   extent(a: number, b: number, positive: boolean, out: Float64Array): void,
 *   draw(g: CanvasRenderingContext2D, time: import("./timeline.js").Timeline, scale: Scale, x0: number, x1: number, dpr: number): void,
 * }} Series
 */

export class Pane {
  /**
   * `format` writes its values (a relative scale's are written as they are: a percent, an index); `left`, its left
   * scale's options (linear, else), for when a series is put on it.
   * @param {{ share?: number, scale?: ConstructorParameters<typeof Scale>[0], left?: ConstructorParameters<typeof Scale>[0], format?: (v: number) => string }} [options]
   */
  constructor({ share = 1, scale, left, format = compact } = {}) {
    // (Its plot's canvas, and its right axis' beside it (as its left axis' is, see placeLeft): a page can move or stretch
    // the plot alone.)
    this.canvas = document.createElement("canvas");
    this.canvas.style.cssText = "position: absolute; left: 0; display: block";
    const g = this.canvas.getContext("2d", { alpha: false });
    if (!g) throw new Error("bitview-charts: no 2D canvas");
    this.g = g;
    this.axisCanvas = document.createElement("canvas");
    this.axisCanvas.style.cssText = "position: absolute; left: 0; display: block";
    this.axisG = /** @type {CanvasRenderingContext2D} */ (this.axisCanvas.getContext("2d", { alpha: false }));
    /** Its part of the chart's height, against the other panes'. */
    this.share = share;
    this.scale = new Scale(scale);
    /** Fitted to its series in view at each frame (else as set: see setRange). */
    this.auto = true;
    /** Its left scale, once a series is on it (fitted or set as the right one is), and its axis' canvas. @type {Scale | null} */
    this.left = null;
    this.leftAuto = true;
    this.leftOptions = left;
    /** Its left scale's range set before it came (its first left series' coming applies it). @type {[number, number] | null} */
    this.leftRange = null;
    /** @type {HTMLCanvasElement | null} */
    this.leftCanvas = null;
    /** @type {CanvasRenderingContext2D | null} */
    this.leftG = null;
    this.leftWidth = 0;
    /** Its layer over everything, the crosshair too (its series `over`: see draw), made as the first comes. @type {HTMLCanvasElement | null} */
    this.overCanvas = null;
    /** @type {CanvasRenderingContext2D | null} */
    this.overG = null;
    /** Told when its layout changes (its left scale come or gone, its layer over it come): the chart lays itself out again. @type {() => void} */
    this.onLayout = () => {};
    /** @type {Series[]} */
    this.series = [];
    /** Each series' side: 0, the right scale; 1, the left. @type {number[]} */
    this.sides = [];
    // (How each side writes a value (its tags', but a line's own, and the crosshair's), and its ticks, all of them at
    // once (their digits alike; none, each as a value): 0, the right one's; 1, the left one's; 2 and 3, theirs as a
    // percent (an indexed scale's as its side's values are). Each slot's ticks as last written, and their words:
    // written anew only when they change.)
    const percent = kept((v) => `${compact(v)}%`);
    this.labels = [kept(format), kept(format), percent, percent];
    /** @type {(((values: number[]) => string[]) | null)[]} */
    this.tickWriters = [null, null, null, null];
    this.tickValues = [new Float64Array(0), new Float64Array(0), new Float64Array(0), new Float64Array(0)];
    /** @type {string[][]} */
    this.tickTexts = [[], [], [], []];
    // (A relative scale's: each series' base, as fitted and as drawn: its picture moved only where each is the same.)
    this.bases = new Float64Array(4);
    this.drawnBases = new Float64Array(4);
    this.basesSame = false;
    this.own = new Float64Array(2);
    /** Where it lies in the chart (CSS pixels), and its plot's width (the axis past it). */
    this.top = 0;
    this.height = 0;
    /** Where it shows (see stretch): where it lies, but while a separator's dragged. */
    this.shownTop = 0;
    this.shownHeight = 0;
    this.width = 0;
    this.plot = 0;
    /** A pane under it: its foot a separator's line (see Chart's layout). */
    this.separated = false;
    // (Its picture: the view it shows, for a pan to move it.)
    this.drawnFrom = NaN;
    this.drawnSpacing = NaN;
    this.drawnPlot = NaN;
    // (Its series' versions summed, and the theme, as drawn: either changed, its picture's stale.)
    this.drawnVersions = NaN;
    // (Its last picture drawn whole (not moved or stretched since), and that picture kept as a zoom began: where it
    // started, its spacing (NaN: none kept).)
    this.sharp = false;
    /** @type {HTMLCanvasElement | null} */
    this.snapshot = null;
    this.snapFrom = NaN;
    this.snapSpacing = NaN;
    this.tickCount = 0;
    // (Its tags as they're laid out: each one's line, its height, kept and grown, never made a frame.)
    this.tagLines = /** @type {Line[]} */ ([]);
    this.tagAt = new Int32Array(8);
    this.tagIndex = new Int32Array(8);
    /** Its series' own tags on the axis being drawn (a drawing's value). */
    this.pageTags = new Tags();
    /** Each side's lines' tags as last drawn: each one's line and middle, and their height. @type {{ lines: Line[], at: Float64Array, step: number }[]} */
    this.drawnTags = [{ lines: [], at: new Float64Array(8), step: 0 }, { lines: [], at: new Float64Array(8), step: 0 }];
    /** Each side's tags kept under this height (CSS pixels: a page's legend over the pane), as setTagsTop gives it. */
    this.tagsTop = [0, 0];
    /**
     * The view's last point as its axes were drawn (their tags read it); and, as the view's first moves, where its tags
     * change (a line's last point in view dropping out, one left out back in): the first point a tag was drawn at, the
     * last one a tag was left out at.
     */
    this.axesEnd = NaN;
    this.tagsFrom = Infinity;
    this.tagsOut = -1;
    this.tagY = new Float64Array(8);
    this.tagPlace = new Float64Array(8);
    /** The least room between two tags as last placed. */
    this.tagStep = 0;
    /** What drawing it whole takes (ms, the last few weighed): a zoom stretches its picture only where it's slow. */
    this.wholeCost = 0;
    /** @type {Theme | null} */
    this.drawnTheme = null;
    this.ticks = new Float64Array(24);
    this.axisTicks = new Float64Array(24);
    this.extent = new Float64Array(2);
  }

  /**
   * A series on its right scale, or its left one (made, with its axis, as the first comes).
   * @template {Series} S @param {S} series @param {"right" | "left"} [side]
   */
  add(series, side = "right") {
    this.series.push(series);
    this.sides.push(side === "left" ? 1 : 0);
    if (side === "left" && !this.left) {
      this.left = new Scale(this.leftOptions);
      this.left.height = this.height;
      if (this.leftRange) this.left.fit(this.leftRange[0], this.leftRange[1]);
      this.onLayout();
    }
    if (series.over && !this.overCanvas) {
      this.overCanvas = document.createElement("canvas");
      // (Over the crosshair's, wherever a page puts them.)
      this.overCanvas.style.cssText = "position: absolute; left: 0; z-index: 2; display: block; pointer-events: none";
      this.overG = this.overCanvas.getContext("2d");
      this.onLayout();
    }
    this.drawnFrom = NaN;
    return series;
  }

  /** @param {ConstructorParameters<typeof Line>[0]} [options] @param {"right" | "left"} [side] */
  addLine(options, side) {
    return this.add(new Line(options), side);
  }

  /** @param {ConstructorParameters<typeof Band>[0]} options @param {"right" | "left"} [side] */
  addBand(options, side) {
    return this.add(new Band(options), side);
  }

  /** @param {ConstructorParameters<typeof Bars>[0]} [options] @param {"right" | "left"} [side] */
  addBars(options, side) {
    return this.add(new Bars(options), side);
  }

  /** @param {ConstructorParameters<typeof Candles>[0]} options @param {"right" | "left"} [side] */
  addCandles(options, side) {
    return this.add(new Candles(options), side);
  }

  /** @param {ConstructorParameters<typeof Custom>[0]} hooks @param {"right" | "left"} [side] */
  addCustom(hooks, side) {
    return this.add(new Custom(hooks), side);
  }

  /** Its series drawn in `list`'s order (the same series, rearranged: the last on top), each on its side. @param {Series[]} list */
  order(list) {
    let same = list.length === this.series.length;
    for (let k = 0; same && k < list.length; k++) same = list[k] === this.series[k];
    if (same) return;
    const sides = list.map((series) => this.sides[this.series.indexOf(series)]);
    console.assert(sides.every((side) => side !== undefined), "bitview-charts: order() takes the pane's own series");
    this.series.splice(0, this.series.length, ...list);
    this.sides.splice(0, this.sides.length, ...sides);
    this.drawnFrom = NaN;
  }

  /** @param {Series} series */
  remove(series) {
    const k = this.series.indexOf(series);
    if (k < 0) return;
    this.series.splice(k, 1);
    this.sides.splice(k, 1);
    this.drawnFrom = NaN;
    // (Its last left series gone: its left scale with it, and its axis' room.)
    if (this.left && !this.sides.includes(1)) (this.left = null), this.onLayout();
  }

  /**
   * The series nearest a point (index `i`, `x` CSS pixels into the plot, `y` from its top) within `within` pixels (the
   * one on top, where two are as near), or -1: none.
   * @param {import("./timeline.js").Timeline} time @param {number} i @param {number} x @param {number} y @param {number} [within]
   */
  hit(time, i, x, y, within = 6) {
    const from = this.#from(time);
    let best = -1, nearest = within;
    for (let k = this.series.length - 1; k >= 0; k--) {
      if (this.series[k].hidden) continue;
      const scale = this.scaleOf(k);
      if (scale.relative) {
        const base = this.series[k].first(from);
        if (!base || base !== base) continue;
        scale.setBase(base);
      }
      const d = this.series[k].distance(i, y, scale, x, time);
      if (d < nearest) (nearest = d), (best = k);
    }
    return best;
  }

  /** A series' scale: its side's. @param {number} k */
  scaleOf(k) {
    return this.sides[k] && this.left ? this.left : this.scale;
  }

  /** Drawn whole at the next frame: a series' values or look changed. */
  changed() {
    this.drawnFrom = NaN;
  }

  /**
   * A scale set by hand (`lo`, `hi`), or fitted to its series again (no range): its right one, or its left.
   * @param {number} [lo] @param {number} [hi] @param {"right" | "left"} [side]
   */
  setRange(lo, hi, side = "right") {
    const auto = lo === undefined || hi === undefined, scale = side === "left" ? this.left : this.scale;
    side === "left" ? ((this.leftAuto = auto), (this.leftRange = auto ? null : [/** @type {number} */ (lo), /** @type {number} */ (hi)])) : (this.auto = auto);
    if (!auto && scale) scale.fit(/** @type {number} */ (lo), /** @type {number} */ (hi));
  }

  /**
   * Laid out at `top`, `width` by `height` CSS pixels (its plot, then its axis `axis` wide), at `dpr` device pixels a CSS
   * pixel.
   * @param {number} top @param {number} width @param {number} height @param {number} axis @param {number} dpr
   */
  place(top, width, height, axis, dpr) {
    this.top = top;
    this.width = width;
    this.height = height;
    this.plot = Math.max(1, width - axis);
    (this.shownTop = top), (this.shownHeight = height);
    const w = Math.round(this.plot * dpr), h = Math.round(height * dpr), aw = Math.round((width - this.plot) * dpr);
    if (this.canvas.width !== w || this.canvas.height !== h) (this.canvas.width = w), (this.canvas.height = h), (this.drawnFrom = NaN);
    Object.assign(this.canvas.style, { top: `${top}px`, width: `${this.plot}px`, height: `${height}px`, transform: "" });
    if (this.axisCanvas.width !== aw || this.axisCanvas.height !== h) (this.axisCanvas.width = aw), (this.axisCanvas.height = h), (this.drawnFrom = NaN);
    Object.assign(this.axisCanvas.style, { top: `${top}px`, width: `${width - this.plot}px`, height: `${height}px`, transform: "" });
    this.scale.height = height;
    if (this.left) this.left.height = height;
    // (Its ticks' room: at most one each 28 pixels (see Scale.ticks), a few more at the ends.)
    const ticks = Math.ceil(height / 28) + 4;
    if (this.ticks.length < ticks) (this.ticks = new Float64Array(ticks)), (this.axisTicks = new Float64Array(ticks));
    // (Its layer over everything: over its plot.)
    const over = this.overCanvas;
    if (over) {
      const ow = Math.round(this.plot * dpr);
      if (over.width !== ow || over.height !== h) (over.width = ow), (over.height = h);
      Object.assign(over.style, { top: `${top}px`, width: `${this.plot}px`, height: `${height}px`, transform: "" });
    }
  }

  /**
   * Shown from `top`, `height` tall, laid out as it was (a separator being dragged: its canvases scaled to it, drawn as
   * laid out; laid out at it once let go, as a canvas just resized is slow to draw into on iOS).
   * @param {number} top @param {number} height
   */
  stretch(top, height) {
    (this.shownTop = top), (this.shownHeight = height);
    const style = { top: `${top}px`, transform: height === this.height ? "" : `scaleY(${height / this.height})`, transformOrigin: "0 0" };
    Object.assign(this.canvas.style, style), Object.assign(this.axisCanvas.style, style);
    if (this.leftCanvas) Object.assign(this.leftCanvas.style, style);
    if (this.overCanvas) Object.assign(this.overCanvas.style, style);
  }

  /**
   * Its series `over` (marks over everything: the crosshair too) drawn on its layer over it, whole: at each frame,
   * wherever its picture's moved.
   * @param {import("./timeline.js").Timeline} time @param {number} dpr
   */
  drawOver(time, dpr) {
    const g = this.overG;
    if (!g || !this.overCanvas) return;
    g.setTransform(1, 0, 0, 1, 0, 0);
    g.clearRect(0, 0, this.overCanvas.width, this.overCanvas.height);
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    const from = this.#from(time);
    for (let k = 0; k < this.series.length; k++) {
      const series = this.series[k];
      if (!series.over || series.hidden) continue;
      const scale = this.scaleOf(k);
      if (scale.relative) {
        const base = series.first(from);
        if (!base || base !== base) continue;
        scale.setBase(base);
      }
      series.draw(g, time, scale, 0, this.plot, dpr);
    }
  }

  /**
   * Its left axis' canvas (every pane has one where any has a left scale: its ground and separator, at least) laid out
   * at `top`, `width` wide (0: none). Made the first time: the chart puts it in.
   * @param {number} top @param {number} width @param {number} dpr
   */
  placeLeft(top, width, dpr) {
    this.leftWidth = width;
    if (!width) return;
    if (!this.leftCanvas) {
      this.leftCanvas = document.createElement("canvas");
      this.leftCanvas.style.cssText = "position: absolute; left: 0; display: block";
      this.leftG = this.leftCanvas.getContext("2d", { alpha: false });
    }
    const canvas = this.leftCanvas;
    const w = Math.round(width * dpr), h = Math.round(this.height * dpr);
    if (canvas.width !== w || canvas.height !== h) (canvas.width = w), (canvas.height = h), (this.drawnFrom = NaN);
    Object.assign(canvas.style, { top: `${top}px`, width: `${width}px`, height: `${this.height}px`, transform: "" });
  }

  /** A scale's writers' slot: its side's, or its side's as a percent. @param {Scale} scale */
  #slot(scale) {
    return (scale.mode === "percent" ? 2 : 0) + (scale === this.left ? 1 : 0);
  }

  /** A value as an axis writes it (the right one's, or `scale`'s). @param {number} v @param {Scale} [scale] */
  axisLabel(v, scale = this.scale) {
    return this.labels[this.#slot(scale)](v);
  }

  /**
   * How a side writes its values (`format`: its tags', but a line's own, and the crosshair's; an indexed scale's too)
   * and its ticks (`ticks`: all of them at once, so their digits agree; none, each as a value).
   * @param {(v: number) => string} format @param {((values: number[]) => string[]) | null} [ticks] @param {"right" | "left"} [side]
   */
  setFormat(format, ticks = null, side = "right") {
    this.#write(side === "left" ? 1 : 0, format, ticks);
  }

  /** How a percent scale (either side) writes its values and its ticks, as setFormat. @param {(v: number) => string} format @param {((values: number[]) => string[]) | null} [ticks] */
  setPercentFormat(format, ticks = null) {
    this.#write(2, format, ticks);
    this.#write(3, format, ticks);
  }

  /** @param {number} slot @param {(v: number) => string} format @param {((values: number[]) => string[]) | null} ticks */
  #write(slot, format, ticks) {
    this.labels[slot] = kept(format);
    this.tickWriters[slot] = ticks;
    this.tickValues[slot] = new Float64Array(0);
    this.drawnFrom = NaN;
  }

  /** The line whose tag on its axis on `side` (0, the right; 1, the left) is at `y` (from its top), as last drawn; none, null. @param {number} side @param {number} y */
  lineAt(side, y) {
    const drawn = this.drawnTags[side];
    for (let k = drawn.lines.length - 1; k >= 0; k--) if (Math.abs(drawn.at[k] - y) <= drawn.step / 2) return drawn.lines[k];
    return null;
  }

  /** A side's tags kept under `top` (CSS pixels from its top: a page's legend over the pane). @param {number} top @param {"right" | "left"} [side] */
  setTagsTop(top, side = "right") {
    const k = side === "left" ? 1 : 0;
    if (this.tagsTop[k] !== top) (this.tagsTop[k] = top), (this.drawnFrom = NaN);
  }

  /** A slot's ticks (`count` of them, in axisTicks) as its writer writes them, written anew only when they change. @param {number} slot @param {number} count */
  #tickTexts(slot, count) {
    const was = this.tickValues[slot], ticks = this.axisTicks;
    let same = was.length === count;
    for (let k = 0; same && k < count; k++) same = was[k] === ticks[k];
    if (same) return this.tickTexts[slot];
    const values = new Array(count);
    for (let k = 0; k < count; k++) values[k] = ticks[k];
    this.tickValues[slot] = Float64Array.from(values);
    return (this.tickTexts[slot] = /** @type {(values: number[]) => string[]} */ (this.tickWriters[slot])(values));
  }

  /** The first point a relative scale reads its series from: its first in view. @param {import("./timeline.js").Timeline} time */
  #from(time) {
    return Math.max(0, Math.floor(time.from));
  }

  /** Fitted to its series in the timeline's view (a relative scale's, each read against its first value in view). @param {import("./timeline.js").Timeline} time */
  fit(time) {
    const n = this.series.length;
    if (this.bases.length < n) (this.bases = new Float64Array(n * 2)), (this.drawnBases = new Float64Array(n * 2));
    this.basesSame = true;
    this.#fit(time, this.scale, this.auto, 0);
    if (this.left) this.#fit(time, this.left, this.leftAuto, 1);
  }

  /**
   * A scale fitted to its side's series.
   * @param {import("./timeline.js").Timeline} time @param {Scale} scale @param {boolean} auto @param {number} side
   */
  #fit(time, scale, auto, side) {
    const out = this.extent, log = scale.mode === "log", relative = scale.relative, from = this.#from(time), own = this.own, n = this.series.length;
    out[0] = Infinity;
    out[1] = -Infinity;
    // (Not relative: each read as it is (a relative mode left, its reading gone too).)
    if (!relative) scale.setBase(NaN);
    let same = true, above = 0, below = 0;
    const a = Math.ceil(time.from), b = Math.floor(time.to);
    for (let k = 0; k < n; k++) {
      if (this.sides[k] !== side || this.series[k].hidden) continue;
      const series = this.series[k];
      // (The room its marks keep clear at the scale's ends: the most any asks, above and below, as the chart library's.)
      if (series.margins) {
        (ROOM[0] = 0), (ROOM[1] = 0), series.margins(time.spacing, ROOM);
        if (ROOM[0] > above) above = ROOM[0];
        if (ROOM[1] > below) below = ROOM[1];
      }
      if (!relative) {
        if (auto) series.fit ? series.fit(a, b, log, out) : series.extent(a, b, log, out);
        continue;
      }
      // (A series with no base (none in view, or 0) has no reading: neither fitted nor drawn.)
      const base = series.first(from);
      this.bases[k] = base;
      if (!Object.is(base, this.drawnBases[k])) same = false;
      if (!auto || !base || base !== base) continue;
      (own[0] = Infinity), (own[1] = -Infinity);
      series.fit ? series.fit(a, b, false, own) : series.extent(a, b, false, own);
      if (!(own[0] <= own[1])) continue;
      scale.setBase(base);
      const lo = own[0] * scale.m + scale.c, hi = own[1] * scale.m + scale.c;
      if (lo < out[0]) out[0] = lo;
      if (hi > out[1]) out[1] = hi;
    }
    if (!same) this.basesSame = false;
    // (No more than half the room its shares leave: a small pane's range kept the right way up.)
    const free = (scale.height * (1 - scale.top - scale.bottom)) / 2;
    if (above + below > free) {
      const k = Math.max(0, free) / (above + below);
      (above *= k), (below *= k);
    }
    if (auto) (scale.above = above), (scale.below = below), scale.fit(out[0], out[1]);
    scale.empty = auto && !(log ? out[0] > 0 && out[0] <= out[1] : out[0] <= out[1]);
  }

  /**
   * Drawn for the timeline's view. Its scale, size, series and theme as they were: a pan's picture moved by whole
   * pixels, a zoom's stretched from its last sharp picture (within half to twice its spacing), each with the strip come
   * into view drawn; else (or `whole`) drawn whole. Whether its picture was moved or stretched (to draw whole once
   * still: no seams, nothing blurred).
   * @param {import("./timeline.js").Timeline} time @param {Theme} theme @param {number} dpr @param {boolean} whole
   */
  draw(time, theme, dpr, whole) {
    const g = this.g, plot = this.plot, spacing = time.spacing, shift = (this.drawnFrom - time.from) * spacing, by = Math.round(shift);
    // (Its series over it drawn apart: theirs never stale its picture.)
    let versions = 0, fixed = false;
    for (let k = 0; k < this.series.length; k++) if (!this.series[k].over) (versions += this.series[k].version), (fixed ||= Boolean(this.series[k].fixed && !this.series[k].hidden));
    const same = !whole && !fixed && this.drawnFrom === this.drawnFrom && (!(this.scale.relative || this.left?.relative) || this.basesSame) && versions === this.drawnVersions && theme === this.drawnTheme && this.scale.unchanged && (!this.left || this.left.unchanged) && this.drawnPlot === plot;
    const moves = same && Math.abs(spacing - this.drawnSpacing) <= spacing * 1e-9 && Math.abs(shift - by) < 1e-3 && Math.abs(by) < plot / 2 && Math.abs(by * dpr - Math.round(by * dpr)) < 1e-3;
    // (A zoom, where drawing it whole takes over half a frame (a browser drawing on the CPU at a large size): its last
    // sharp picture kept as the zoom began, stretched to the new spacing while it's within half to twice it. Drawn whole
    // where that's quick: sharp all along.)
    const slow = this.wholeCost > SLOW;
    if (slow && same && !moves && this.snapSpacing !== this.snapSpacing && this.sharp) this.#snap(dpr);
    const stretch = spacing / this.snapSpacing, stretches = slow && same && !moves && stretch >= 0.5 && stretch <= 2;
    if (moves) {
      if (!by) return false;
      // (What stays, copied to where it goes, in device pixels; then the strip come in.)
      const keep = (plot - Math.abs(by)) * dpr, h = this.canvas.height, from = by < 0 ? -by * dpr : 0, to = by > 0 ? by * dpr : 0;
      g.setTransform(1, 0, 0, 1, 0, 0);
      g.drawImage(this.canvas, from, 0, keep, h, to, 0, keep, h);
      by > 0 ? this.#strip(time, theme, dpr, 0, by + 1) : this.#strip(time, theme, dpr, plot + by - 1, plot);
    } else if (stretches) {
      // (Where its picture's left edge lands now, and how wide it is: the plot left of and past it drawn.)
      const left = (this.snapFrom - time.from) * spacing, width = plot * stretch, right = left + width;
      g.setTransform(1, 0, 0, 1, 0, 0);
      g.save();
      g.beginPath();
      g.rect(0, 0, plot * dpr, this.canvas.height);
      g.clip();
      g.drawImage(/** @type {HTMLCanvasElement} */ (this.snapshot), 0, 0, plot * dpr, this.canvas.height, left * dpr, 0, width * dpr, this.canvas.height);
      g.restore();
      if (left > 0) this.#strip(time, theme, dpr, 0, Math.min(plot, Math.ceil(left) + 1));
      if (right < plot) this.#strip(time, theme, dpr, Math.max(0, Math.floor(right) - 1), plot);
    } else {
      const began = performance.now();
      this.#strip(time, theme, dpr, 0, plot);
      this.#axes(time, theme, dpr);
      // (Sharp: a zoom from here keeps this picture as it begins.)
      this.sharp = true;
      this.snapSpacing = NaN;
      this.wholeCost = this.wholeCost * 0.5 + (performance.now() - began) * 0.5;
    }
    if (moves || stretches) {
      this.sharp = false;
      // (Its tags read the view's end: its axes drawn again (the plot moved alone) where that's another point.)
      const from = this.#from(time);
      if ((Math.floor(time.to) !== this.axesEnd || from > this.tagsFrom || from <= this.tagsOut) && this.#tagged()) this.#axes(time, theme, dpr);
    }
    this.scale.drawnNow();
    this.left?.drawnNow();
    this.drawnFrom = time.from;
    this.drawnSpacing = spacing;
    this.drawnPlot = plot;
    this.drawnVersions = versions;
    this.drawnTheme = theme;
    for (let k = 0; k < this.series.length; k++) this.drawnBases[k] = this.bases[k];
    return moves || stretches;
  }

  /** Its axes drawn (each on its own canvas, beside its plot): ticks, tags, separators. @param {import("./timeline.js").Timeline} time @param {Theme} theme @param {number} dpr */
  #axes(time, theme, dpr) {
    (this.tagsFrom = Infinity), (this.tagsOut = -1);
    this.axisG.setTransform(dpr, 0, 0, dpr, 0, 0);
    this.#axis(this.axisG, this.scale, time, theme, dpr, 0, this.width - this.plot, 0);
    if (this.leftG && this.leftWidth) {
      const lg = this.leftG;
      lg.setTransform(dpr, 0, 0, dpr, 0, 0);
      if (this.left) this.#axis(lg, this.left, time, theme, dpr, 0, this.leftWidth, 1);
      else {
        lg.fillStyle = theme.background;
        lg.fillRect(0, 0, this.leftWidth, this.height);
        if (this.separated) this.#separator(lg, theme, dpr, 0, this.leftWidth);
        this.drawnTags[1].lines.length = 0;
      }
    }
    this.axesEnd = Math.floor(time.to);
  }

  /** Whether it has a tag shown (a line's, with one). */
  #tagged() {
    for (let k = 0; k < this.series.length; k++) {
      const line = this.series[k];
      if (line instanceof Line && line.tag && !line.hidden) return true;
    }
    return false;
  }

  /**
   * An axis drawn on `g` from `x` (CSS pixels), `width` wide: its ground, its scale's ticks' values at their heights, its
   * side's tags, the separator's line along its foot. The right one's words from its left edge, the left one's to its
   * right edge.
   * @param {CanvasRenderingContext2D} g @param {Scale} scale @param {import("./timeline.js").Timeline} time @param {Theme} theme
   * @param {number} dpr @param {number} x @param {number} width @param {number} side
   */
  #axis(g, scale, time, theme, dpr, x, width, side) {
    g.fillStyle = theme.background;
    g.fillRect(x, 0, width, this.height);
    g.fillStyle = theme.text;
    g.font = theme.font;
    g.textBaseline = "middle";
    g.textAlign = side ? "right" : "left";
    const count = scale.ticks(this.axisTicks), at = side ? x + width - 8 : x + 8, slot = this.#slot(scale), texts = this.tickWriters[slot] ? this.#tickTexts(slot, count) : null;
    // (Its tags placed first: a tick's label none where one covers it (within half a tag, half its words' height and
    // 4 pixels: one touching a tag reads cramped against it).)
    const tags = this.#placeTags(scale, time, side), place = this.tagPlace, near = this.tagStep / 2 + 10, page = this.pageTags;
    page.count = 0;
    for (let k = 0; k < this.series.length; k++) if (!this.series[k].hidden) this.series[k].axisTags?.(side, page.add);
    // (Each where it's drawn: within the pane.)
    for (let k = 0; k < page.count; k++) page.at[k] = Math.min(Math.max(page.at[k], TAG / 2), this.height - TAG / 2);
    for (let k = 0; k < count; k++) {
      const y = scale.own(this.axisTicks[k]);
      if (!(y > 6 && y < this.height - 6)) continue;
      let covered = page.covers(y, TAG / 2 + 10);
      for (let t = 0; !covered && t < tags; t++) covered = Math.abs(place[t] - y) < near;
      if (!covered) g.fillText(texts ? (texts[k] ?? "") : this.axisLabel(this.axisTicks[k], scale), at, y);
    }
    this.#tags(g, scale, time, theme, x, width, side, tags);
    // (Its series' own tags over its lines' (a drawing's value), each where it says, within the pane.)
    g.textAlign = side ? "right" : "left";
    for (let k = 0; k < page.count; k++) {
      const y = page.at[k];
      g.fillStyle = page.colors[k];
      g.beginPath();
      g.roundRect(x + 2, y - TAG / 2 + 1, width - 4, TAG - 2, 4);
      g.fill();
      g.fillStyle = page.inks[k];
      g.fillText(page.texts[k], side ? x + width - 8 : x + 8, y);
    }
    if (this.separated) this.#separator(g, theme, dpr, x, x + width);
  }

  /**
   * The plot from x `x0` to `x1` (CSS pixels) drawn anew: its ground, its grid, its series.
   * @param {import("./timeline.js").Timeline} time @param {Theme} theme @param {number} dpr @param {number} x0 @param {number} x1
   */
  #strip(time, theme, dpr, x0, x1) {
    const g = this.g;
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    g.save();
    g.beginPath();
    g.rect(x0, 0, x1 - x0, this.height);
    g.clip();
    g.fillStyle = theme.background;
    g.fillRect(x0, 0, x1 - x0, this.height);
    // (The grid by the right scale's ticks; by the left one's where only it has series read on a scale (a page's own
    // drawn as it likes, none).)
    let right = !this.left;
    for (let k = 0; !right && k < this.series.length; k++) right = this.sides[k] === 0 && !this.series[k].hidden && this.series[k].scaled !== false;
    const grid = right ? this.scale : /** @type {Scale} */ (this.left);
    this.tickCount = grid.ticks(this.ticks);
    g.strokeStyle = theme.grid;
    g.lineWidth = 1;
    g.beginPath();
    for (let k = 0; k < this.tickCount; k++) {
      const y = Math.round(grid.own(this.ticks[k]) * dpr) / dpr + 0.5 / dpr;
      g.moveTo(x0, y);
      g.lineTo(x1, y);
    }
    g.stroke();
    const from = this.#from(time);
    for (let k = 0; k < this.series.length; k++) {
      if (this.series[k].hidden || this.series[k].over) continue;
      const scale = this.scaleOf(k);
      if (scale.relative) {
        const base = this.series[k].first(from);
        if (!base || base !== base) continue;
        scale.setBase(base);
      }
      this.series[k].draw(g, time, scale, x0, x1, dpr);
    }
    // (Price lines: across at a line's last value, dotted, in its color there; its dots where they'd be on the timeline,
    // so a picture moved by a pan has them where a whole draw does.)
    g.setLineDash(DOTS);
    g.lineDashOffset = (x0 + time.from * time.spacing) % (DOTS[0] + DOTS[1]);
    for (let k = 0; k < this.series.length; k++) {
      const line = this.series[k], scale = this.scaleOf(k);
      if (!(line instanceof Line) || !line.priceLine || line.hidden) continue;
      const i = line.last(scale.mode === "log");
      if (i < 0) continue;
      if (scale.relative) {
        const base = line.first(from);
        if (!base || base !== base) continue;
        scale.setBase(base);
      }
      const y = Math.round(scale.y(line.values[i]) * dpr) / dpr + 0.5 / dpr;
      g.strokeStyle = line.colorAt(i);
      g.lineWidth = 1;
      g.beginPath();
      g.moveTo(x0, y);
      g.lineTo(x1, y);
      g.stroke();
    }
    g.setLineDash(SOLID);
    g.lineDashOffset = 0;
    if (this.separated) this.#separator(g, theme, dpr, x0, x1);
    g.restore();
  }

  /**
   * The separator's line along its foot, on `g` from `x0` to `x1`.
   * @param {CanvasRenderingContext2D} g @param {Theme} theme @param {number} dpr @param {number} x0 @param {number} x1
   */
  #separator(g, theme, dpr, x0, x1) {
    const y = this.height - 0.5 / dpr;
    g.strokeStyle = theme.separator;
    g.lineWidth = 1 / dpr;
    g.beginPath();
    g.moveTo(x0, y);
    g.lineTo(x1, y);
    g.stroke();
  }

  /**
   * A side's lines' tags placed (their count returned; their lines in tagLines, each one's point in tagIndex, by tagAt
   * top to bottom at tagPlace, tagStep apart at least): each one at its last value in view (as the chart library's
   * were: they read the view's end as a pan moves it; a price line, its last of all; one with nothing in view, none),
   * laid top to bottom where they'd overlap (none pushed past its axis' ends); on a relative scale, as it reads them.
   * @param {Scale} scale @param {import("./timeline.js").Timeline} time @param {number} side
   */
  #placeTags(scale, time, side) {
    const log = scale.mode === "log", relative = scale.relative, from = this.#from(time), lines = this.tagLines, end = Math.floor(time.to);
    lines.length = 0;
    for (let k = 0; k < this.series.length; k++) {
      if (this.sides[k] !== side) continue;
      const line = this.series[k];
      if (!(line instanceof Line) || !line.tag || line.hidden) continue;
      const i = line.lastBy(end, log);
      // (Its last point before the view: nothing of it in view. On a relative scale, one with no base has no reading.)
      const base = relative ? line.first(from) : 1;
      if (i >= 0 && i < from && i > this.tagsOut) this.tagsOut = i;
      if (i < from || !base || base !== base) continue;
      if (i < this.tagsFrom) this.tagsFrom = i;
      if (this.tagIndex.length <= lines.length) ((grown) => (grown.set(this.tagIndex), (this.tagIndex = grown)))(new Int32Array(lines.length * 2 + 8));
      this.tagIndex[lines.length] = i;
      lines.push(line);
    }
    const n = lines.length;
    if (!n) return 0;
    if (this.tagY.length < n) (this.tagY = new Float64Array(n * 2)), (this.tagAt = new Int32Array(n * 2)), (this.tagPlace = new Float64Array(n * 2));
    const ys = this.tagY, order = this.tagAt, at = this.tagPlace, index = this.tagIndex;
    for (let k = 0; k < n; k++) {
      if (relative) scale.setBase(lines[k].first(from));
      (ys[k] = scale.y(lines[k].values[index[k]])), (order[k] = k);
    }
    // (Top to bottom, each at least a tag below the one over it (less, where they'd not all fit: the axis shared); then,
    // past the foot, each back up.)
    const step = Math.min(TAG, (this.height - this.tagsTop[side]) / n);
    for (let k = 1; k < n; k++)
      for (let j = k; j > 0 && ys[order[j]] < ys[order[j - 1]]; j--) {
        const swap = order[j];
        (order[j] = order[j - 1]), (order[j - 1] = swap);
      }
    for (let k = 0; k < n; k++) at[k] = Math.max(ys[order[k]], k ? at[k - 1] + step : this.tagsTop[side] + step / 2);
    for (let k = n - 1; k >= 0; k--) at[k] = Math.min(at[k], k < n - 1 ? at[k + 1] - step : this.height - step / 2);
    this.tagStep = step;
    return n;
  }

  /**
   * A side's `n` tags (as #placeTags placed them) drawn on its axis (`g`, from `x`, `width` wide): each one's last value
   * in a badge in its color, its words black or white by its lightness.
   * @param {CanvasRenderingContext2D} g @param {Scale} scale @param {import("./timeline.js").Timeline} time @param {Theme} theme
   * @param {number} x @param {number} width @param {number} side @param {number} n
   */
  #tags(g, scale, time, theme, x, width, side, n) {
    const relative = scale.relative, from = this.#from(time), lines = this.tagLines, order = this.tagAt, at = this.tagPlace, index = this.tagIndex, step = this.tagStep;
    g.font = theme.font;
    g.textAlign = side ? "right" : "left";
    g.textBaseline = "middle";
    // (Where each is drawn kept, for a point on one: see lineAt.)
    const drawn = this.drawnTags[side];
    drawn.lines.length = 0;
    if (drawn.at.length < n) drawn.at = new Float64Array(n * 2);
    drawn.step = step;
    for (let k = 0; k < n; k++) {
      const line = lines[order[k]], i = index[order[k]], y = at[k];
      (drawn.lines[k] = line), (drawn.at[k] = y);
      const color = line.colorAt(i);
      g.fillStyle = color;
      g.beginPath();
      g.roundRect(x + 2, y - step / 2 + 1, width - 4, Math.max(1, step - 2), Math.max(0, Math.min(4, step / 2 - 1)));
      g.fill();
      // (Too tight to read: its color alone.)
      if (step < 10) continue;
      g.fillStyle = inkOn(color);
      if (relative) scale.setBase(line.first(from));
      g.fillText(relative ? this.axisLabel(line.values[i] * scale.m + scale.c, scale) : (line.format ?? this.labels[side])(line.values[i]), side ? x + width - 8 : x + 8, y);
    }
  }

  /** Its plot as drawn now (sharp) kept, for a zoom to stretch: the canvas it's kept in made once, grown with it. @param {number} dpr */
  #snap(dpr) {
    const w = this.canvas.width, h = this.canvas.height;
    this.snapshot ??= document.createElement("canvas");
    if (this.snapshot.width !== w || this.snapshot.height !== h) (this.snapshot.width = w), (this.snapshot.height = h);
    const sg = /** @type {CanvasRenderingContext2D} */ (this.snapshot.getContext("2d", { alpha: false }));
    sg.drawImage(this.canvas, 0, 0, this.plot * dpr, h, 0, 0, this.plot * dpr, h);
    this.snapFrom = this.drawnFrom;
    this.snapSpacing = this.drawnSpacing;
  }

}
