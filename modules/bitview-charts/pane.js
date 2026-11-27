// @ts-check
// A pane: drawn whole when a scale or its size changed; else its picture shifts by whole pixels, new strip drawn.

import { Scale, EVERY_POWER } from "./scale.js";
import { Line, FAINT } from "./line.js";
import { Band } from "./band.js";
import { Bars } from "./bars.js";
import { Candles, lineOf } from "./candles.js";
import { Custom } from "./custom.js";
import { compact, kept } from "./format.js";
import { solid, tint, painted } from "./color.js";
import { Tags, pill, PILL } from "./tags.js";
import { Legend } from "./legend.js";
import { layer } from "./layer.js";
import { alike } from "./timeline.js";

/**
 * @typedef {{ background: string, text: string, grid: string, font: string, crosshair: string, across: string, badge: string, separator: string, active: string }} Theme
 */

/** ms: a pane drawing whole slower than this stretches its picture on a zoom instead. */
const SLOW = 8;
const DOTS = [2, 3], SOLID = /** @type {number[]} */ ([]);
const ROOM = new Float64Array(2);
/**
 * @param {Series} series @param {number} a @param {number} b @param {boolean} positive @param {Float64Array} out @param {import("./timeline.js").Timeline} time
 */
function widen(series, a, b, positive, out, time) {
  series.fit ? series.fit(a, b, positive, out) : series.extent(a, b, positive, out, time);
}
const TAG = PILL;
/** Nearer a tag than this, an axis label is dropped. */
const COVER = TAG / 2 + 10;
/**
 * More than two tags burying more than this share of the labels that would show yield: only the price's and the focused
 * series' stay. Two, the price and one line, always fit. (Swept over 632 presets at 382 and 1440: sides of 3 or 4 tags
 * bury a quarter or a third, one or two labels in six to eight; the many-line ones 0.43 and up, as SMA and EMA compare
 * at 1440. Set in the gap, so a pan's label more or less rarely tips a side.)
 */
const BURIED = 0.4;
/**
 * `fixed`: never moved; `dashed`: never stretched; `immovable`: nor panned; `over`: own layer, drawn whole each frame;
 * `scaled: false`: not on its side's scale; `viewed`: brought to the view before each draw; `crosshair`: never pictured.
 * @typedef {{
 *   version: number,
 *   hidden?: boolean,
 *   faint?: boolean,
 *   fixed?: boolean,
 *   dashed?: boolean,
 *   immovable?: boolean,
 *   over?: boolean,
 *   crosshair?: boolean,
 *   scaled?: boolean,
 *   overlay?: Scale | null,
 *   fit?: ((a: number, b: number, positive: boolean, out: Float64Array) => void) | null,
 *   margins?(spacing: number, out: Float64Array, time: import("./timeline.js").Timeline, positive: boolean): void,
 *   axisTags?(side: number, add: (y: number, text: string, color: string, ink?: string) => void, scale: Scale, stepAt: (v: number) => number): void,
 *   behind?: boolean,
 *   timeTags?(add: (x: number, text: string, color: string, ink?: string) => void): void,
 *   first(i: number): number,
 *   distance(i: number, y: number, scale: Scale, x: number, time: import("./timeline.js").Timeline): number,
 *   extent(a: number, b: number, positive: boolean, out: Float64Array, time?: import("./timeline.js").Timeline): void,
 *   viewed?(time: import("./timeline.js").Timeline): void,
 *   draw(g: CanvasRenderingContext2D, time: import("./timeline.js").Timeline, scale: Scale, x0: number, x1: number, dpr: number, theme: Theme): void,
 *   drawDots?(g: CanvasRenderingContext2D, time: import("./timeline.js").Timeline, scale: Scale, x0: number, x1: number, dpr: number, theme: Theme): void,
 *   drawOver?(g: CanvasRenderingContext2D, time: import("./timeline.js").Timeline, scale: Scale, x0: number, x1: number, dpr: number, theme: Theme, drew: (x: number, y: number, width: number, height: number) => void): void,
 * }} Series
 */

/** By original pane indices. @typedef {{ scale: Scale, side: number, indices: number[] }} ScaleGroup */

export class Pane {
  /**
   * @param {{ share?: number, scale?: ConstructorParameters<typeof Scale>[0], left?: ConstructorParameters<typeof Scale>[0], format?: (v: number) => string }} [options]
   */
  constructor({ share = 1, scale, left, format = compact } = {}) {
    // (Plot and right axis on canvases of their own: a page can move or stretch the plot alone.)
    this.canvas = layer("position: absolute; left: 0; display: block");
    /** @type {Legend | null} */
    this.legend = null;
    this.frame = document.createElement("div");
    this.frame.style.cssText = "position: absolute; left: 0; top: 0; pointer-events: none";
    const g = this.canvas.getContext("2d", { alpha: false });
    if (!g) throw new Error("bitview-charts: no 2D canvas");
    this.g = g;
    this.axisCanvas = layer("position: absolute; left: 0; display: block");
    this.axisG = /** @type {CanvasRenderingContext2D} */ (this.axisCanvas.getContext("2d", { alpha: false }));
    this.share = share;
    this.scale = new Scale(scale);
    this.auto = true;
    /** @type {Scale | null} */
    this.left = null;
    this.leftAuto = true;
    this.leftOptions = left;
    /** @type {[number, number] | null} */
    this.leftRange = null;
    /** @type {HTMLCanvasElement | null} */
    this.leftCanvas = null;
    /** @type {CanvasRenderingContext2D | null} */
    this.leftG = null;
    this.leftWidth = 0;
    /** @type {HTMLCanvasElement | null} */
    this.overCanvas = null;
    /** @type {CanvasRenderingContext2D | null} */
    this.overG = null;
    /** Boxes drawn over last frame, cleared alone: in WebKit clearing the layer whole was most of a pan frame. */
    this.overBoxes = new Float64Array(64);
    this.overCount = 0;
    this.overWhole = true;
    /** Made once. @param {number} x @param {number} y @param {number} width @param {number} height */
    this.drewOver = (x, y, width, height) => {
      if (this.overCount * 4 >= this.overBoxes.length) ((grown) => (grown.set(this.overBoxes), (this.overBoxes = grown)))(new Float64Array(this.overBoxes.length * 2));
      const k = this.overCount++ * 4;
      (this.overBoxes[k] = x), (this.overBoxes[k + 1] = y), (this.overBoxes[k + 2] = width), (this.overBoxes[k + 3] = height);
    };
    /** @type {() => void} */
    this.onLayout = () => {};
    /** @type {Series[]} */
    this.series = [];
    /** 0: the right scale; 1: the left. @type {number[]} */
    this.sides = [];
    // (Writer slots: 0 right, 1 left, 2 and 3 theirs as a percent.)
    const percent = kept((v, step) => `${compact(v, step)}%`);
    this.labels = [kept(format), kept(format), percent, percent];
    /** @type {(((values: number[]) => string[]) | null)[]} */
    this.tickWriters = [null, null, null, null];
    this.tickValues = [new Float64Array(0), new Float64Array(0), new Float64Array(0), new Float64Array(0)];
    /** @type {string[][]} */
    this.tickTexts = [[], [], [], []];
    this.drawnTicks = [new Float64Array(0), new Float64Array(0), new Float64Array(0), new Float64Array(0)];
    this.drawnTickCounts = [0, 0, 0, 0];
    // (Relative bases as fitted and as drawn: its picture is moved only where each is the same.)
    this.bases = new Float64Array(4);
    this.drawnBases = new Float64Array(4);
    this.basesSame = false;
    this.own = new Float64Array(2);
    this.top = 0;
    this.height = 0;
    this.width = 0;
    this.plot = 0;
    this.separated = false;
    this.drawnFrom = NaN;
    this.drawnSpacing = NaN;
    this.drawnPlot = NaN;
    this.drawnVersions = NaN;
    this.drawnOvers = NaN;
    this.drawnHidden = new Uint8Array(0);
    /** Bumped as its scale or series change, so a still pointer's hits are read again. */
    this.changes = 0;
    this.sharp = false;
    /** @type {HTMLCanvasElement | null} */
    this.snapshot = null;
    this.snapFrom = NaN;
    this.snapSpacing = NaN;
    this.tickCount = 0;
    this.tagLines = /** @type {Line[]} */ ([]);
    /** A candle series' tag is its close line's. */
    this.tagSeries = /** @type {Series[]} */ ([]);
    this.tagAt = new Int32Array(8);
    this.tagIndex = new Int32Array(8);
    this.pageTags = [new Tags(), new Tags()];
    /** @type {{ lines: Series[], at: Float64Array, step: number }[]} */
    this.drawnTags = [{ lines: [], at: new Float64Array(8), step: 0 }, { lines: [], at: new Float64Array(8), step: 0 }];
    this.drawnLabels = [{ at: new Float64Array(16), count: 0 }, { at: new Float64Array(16), count: 0 }];
    this.axisNeed = [0, 0];
    this.axesEnd = NaN;
    this.tagsFrom = Infinity;
    this.tagsOut = -1;
    this.tagY = new Float64Array(8);
    this.tagPlace = new Float64Array(8);
    this.tagStep = 0;
    /** The series pointed at (the legend's lit one), set by the chart each frame. @type {Series | null} */
    this.focus = null;
    this.drawnFocus = /** @type {Series | null} */ (null);
    /** Per side: its tags yielded (only the price's and the focus' drawn). */
    this.yields = [false, false];
    this.wholeCost = 0;
    this.still = false;
    /** @type {Theme | null} */
    this.drawnTheme = null;
    this.ticks = new Float64Array(24);
    this.axisTicks = new Float64Array(24);
    this.extent = new Float64Array(2);
    /** @type {{
     *   groups: ScaleGroup[],
     *   bindings: { series: Series, overlay: Scale | null | undefined, side: number }[],
     *   right: Scale | null, left: Scale | null,
     * }} */
    this.fitting = { groups: [], bindings: [], right: null, left: null };
  }

  /**
   * @template {Series} S @param {S} series @param {"right" | "left"} [side]
   */
  add(series, side = "right") {
    const at = series.behind ? this.series.filter((other) => other.behind).length : this.series.length;
    this.series.splice(at, 0, series);
    this.sides.splice(at, 0, side === "left" ? 1 : 0);
    if (side === "left" && !this.left) {
      this.left = new Scale(this.leftOptions);
      this.left.height = this.height;
      if (this.leftRange) this.left.fit(this.leftRange[0], this.leftRange[1]);
      this.onLayout();
    }
    if ((series.over || series.drawOver) && !this.overCanvas) {
      // (z-index: over the crosshair's, wherever a page puts them.)
      this.overCanvas = layer("position: absolute; left: 0; z-index: 2; display: block; pointer-events: none");
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

  /** Those left out follow, as they were. @param {Series[]} list */
  order(list) {
    const given = list.length === this.series.length ? list : [...list, ...this.series.filter((series) => !list.includes(series))];
    const all = given.some((series) => series.behind) ? [...given.filter((series) => series.behind), ...given.filter((series) => !series.behind)] : given;
    let same = true;
    for (let k = 0; same && k < all.length; k++) same = all[k] === this.series[k];
    if (same) return;
    const sides = all.map((series) => this.sides[this.series.indexOf(series)]);
    console.assert(sides.every((side) => side !== undefined), "bitview-charts: order() takes the pane's own series");
    this.series.splice(0, this.series.length, ...all);
    this.sides.splice(0, this.sides.length, ...sides);
    // (What's on top under a still pointer changed too.)
    this.drawnFrom = NaN;
    this.changes++;
  }

  /** @param {Series} series */
  remove(series) {
    const k = this.series.indexOf(series);
    if (k < 0) return;
    this.series.splice(k, 1);
    this.sides.splice(k, 1);
    // (What's under a still pointer changed too.)
    this.drawnFrom = NaN;
    this.changes++;
    if (this.left && !this.sides.includes(1)) (this.left = null), this.onLayout();
  }

  /**
   * The one on top wins a tie; -1: none.
   * @param {import("./timeline.js").Timeline} time @param {number} i @param {number} x @param {number} y @param {number} [within]
   */
  hit(time, i, x, y, within = 6) {
    let best = -1, nearest = within;
    for (let k = this.series.length - 1; k >= 0; k--) {
      const d = this.#distance(k, time, i, x, y);
      if (d < nearest) (nearest = d), (best = k);
    }
    return best;
  }

  /**
     * Infinity: hidden, not its, or no reading there.
     * @param {Series} series @param {import("./timeline.js").Timeline} time @param {number} i @param {number} x @param {number} y
     */
  distanceOf(series, time, i, x, y) {
    return this.#distance(this.series.indexOf(series), time, i, x, y);
  }

  /** @param {number} k @param {import("./timeline.js").Timeline} time @param {number} i @param {number} x @param {number} y */
  #distance(k, time, i, x, y) {
    if (k < 0 || this.series[k].hidden) return Infinity;
    const scale = this.#reading(k, time);
    return scale ? this.series[k].distance(i, y, scale, x, time) : Infinity;
  }

  /** @returns {Legend} */
  setLegend() {
    return (this.legend ??= new Legend(this.frame));
  }

  /**
     * As its tag reads it (log: above zero); -1: none.
     * @param {any} series @param {import("./timeline.js").Timeline} time
     */
  restOf(series, time) {
    const k = this.series.indexOf(series), i = k < 0 ? -1 : series.lastBy(time.lastVisible, this.scaleOf(k).mode === "log");
    return i >= Math.max(0, time.firstVisible) ? i : -1;
  }

  /**
   * @param {any} series @param {number} v @param {import("./timeline.js").Timeline} time
   */
  words(series, v, time) {
    const k = this.series.indexOf(series), scale = k >= 0 ? this.scaleOf(k) : this.scale;
    if (scale.relative) {
      const base = series.first(this.#from(time));
      if (!base || base !== base) return "";
      scale.setBase(base);
      const read = v * scale.m + scale.c;
      return this.axisLabel(read, scale, this.stepAt(read, scale));
    }
    return (series.format ?? this.labels[k >= 0 && this.sides[k] ? 1 : 0])(v, this.stepAt(v, scale));
  }

  /** @param {number} k */
  scaleOf(k) {
    const overlay = this.series[k].overlay;
    if (!overlay) return this.sides[k] && this.left ? this.left : this.scale;
    if (overlay.height !== this.height) overlay.height = this.height;
    return overlay;
  }

  /**
   * @param {number} k @param {import("./timeline.js").Timeline} time
   */
  #reading(k, time) {
    if (k < 0) return null;
    const scale = this.scaleOf(k);
    if (!scale.relative) return scale;
    const base = this.series[k].first(this.#from(time));
    if (!base || base !== base) return null;
    scale.setBase(base);
    return scale;
  }

  /**
     * NaN where it can't.
     * @param {Series} series @param {number} value @param {import("./timeline.js").Timeline} time
     */
  y(series, value, time) {
    const scale = this.#reading(this.series.indexOf(series), time);
    return scale ? scale.y(value) : NaN;
  }

  /**
   * NaN where it has none.
   * @param {Series} series @param {number} y @param {import("./timeline.js").Timeline} time
   */
  value(series, y, time) {
    const scale = this.#reading(this.series.indexOf(series), time);
    if (!scale) return NaN;
    const v = scale.value(y);
    return scale.relative ? (v - scale.c) / scale.m : v;
  }

  /** For a change in a series' values or look. */
  changed() {
    this.drawnFrom = NaN;
  }

  /**
   * Kept for a left scale made later.
   * @param {"right" | "left"} side @param {{ mode?: import("./scale.js").ScaleMode, top?: number, bottom?: number }} look
   */
  setScale(side, look) {
    if (side === "left") this.leftOptions = { ...this.leftOptions, ...look };
    const scale = side === "left" ? this.left : this.scale;
    if (!scale) return;
    if (look.mode !== undefined) scale.mode = look.mode;
    if (look.top !== undefined) scale.top = look.top;
    if (look.bottom !== undefined) scale.bottom = look.bottom;
  }

  /**
   * No range: fitted to its series again.
   * @param {number} [lo] @param {number} [hi] @param {"right" | "left"} [side]
   */
  setRange(lo, hi, side = "right") {
    const auto = lo === undefined || hi === undefined, scale = side === "left" ? this.left : this.scale;
    side === "left" ? ((this.leftAuto = auto), (this.leftRange = auto ? null : [/** @type {number} */ (lo), /** @type {number} */ (hi)])) : (this.auto = auto);
    if (!auto && scale) scale.fit(/** @type {number} */ (lo), /** @type {number} */ (hi));
  }

  /**
   * @param {number} top @param {number} width @param {number} height @param {number} axis @param {number} dpr
   */
  place(top, width, height, axis, dpr) {
    this.top = top;
    this.width = width;
    this.height = height;
    this.plot = Math.max(1, width - axis);
    const w = Math.round(this.plot * dpr), h = Math.round(height * dpr), aw = Math.round((width - this.plot) * dpr);
    if (this.canvas.width !== w || this.canvas.height !== h) (this.canvas.width = w), (this.canvas.height = h), (this.drawnFrom = NaN);
    Object.assign(this.canvas.style, { top: `${top}px`, width: `${this.plot}px`, height: `${height}px`, transform: "" });
    Object.assign(this.frame.style, { top: `${top}px`, width: `${this.plot}px`, height: `${height}px` });
    if (this.axisCanvas.width !== aw || this.axisCanvas.height !== h) (this.axisCanvas.width = aw), (this.axisCanvas.height = h), (this.drawnFrom = NaN);
    Object.assign(this.axisCanvas.style, { top: `${top}px`, width: `${width - this.plot}px`, height: `${height}px`, transform: "" });
    this.scale.height = height;
    if (this.left) this.left.height = height;
    const ticks = Math.ceil(height / EVERY_POWER) + 4;
    if (this.ticks.length < ticks) (this.ticks = new Float64Array(ticks)), (this.axisTicks = new Float64Array(ticks));
    const over = this.overCanvas;
    if (over) {
      const ow = Math.round(this.plot * dpr);
      if (over.width !== ow || over.height !== h) (over.width = ow), (over.height = h), (this.drawnFrom = NaN);
      Object.assign(over.style, { top: `${top}px`, width: `${this.plot}px`, height: `${height}px`, transform: "" });
    }
  }

  /**
   * @param {import("./timeline.js").Timeline} time @param {number} dpr
   */
  drawOver(time, dpr) {
    const g = this.overG, over = this.overCanvas;
    if (!g || !over || this.still) return;
    g.setTransform(1, 0, 0, 1, 0, 0);
    if (this.overWhole) g.clearRect(0, 0, over.width, over.height);
    else {
      // (Each out to whole device pixels and one past, for its smoothed edges.)
      const boxes = this.overBoxes;
      for (let k = 0; k < this.overCount * 4; k += 4) {
        const x = Math.floor(boxes[k] * dpr) - 1, y = Math.floor(boxes[k + 1] * dpr) - 1;
        g.clearRect(x, y, Math.ceil((boxes[k] + boxes[k + 2]) * dpr) + 1 - x, Math.ceil((boxes[k + 1] + boxes[k + 3]) * dpr) + 1 - y);
      }
    }
    (this.overCount = 0), (this.overWhole = false);
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    const from = this.#from(time);
    for (let k = 0; k < this.series.length; k++) {
      const series = this.series[k];
      if (series.hidden || !(series.over || series.drawOver)) continue;
      const scale = this.scaleOf(k);
      if (scale.relative) {
        const base = series.first(from);
        if (!base || base !== base) continue;
        scale.setBase(base);
      }
      if (series.faint) g.globalAlpha = FAINT;
      if (series.over) (this.overWhole = true), series.draw(g, time, scale, 0, this.plot, dpr, /** @type {Theme} */ (this.drawnTheme));
      else series.drawOver?.(g, time, scale, 0, this.plot, dpr, /** @type {Theme} */ (this.drawnTheme), this.drewOver);
      g.globalAlpha = 1;
    }
  }

  /**
     * @param {number} top @param {number} width @param {number} dpr
     */
  placeLeft(top, width, dpr) {
    this.leftWidth = width;
    if (!width) return;
    if (!this.leftCanvas) {
      this.leftCanvas = layer("position: absolute; left: 0; display: block");
      this.leftG = this.leftCanvas.getContext("2d", { alpha: false });
    }
    const canvas = this.leftCanvas;
    const w = Math.round(width * dpr), h = Math.round(this.height * dpr);
    if (canvas.width !== w || canvas.height !== h) (canvas.width = w), (canvas.height = h), (this.drawnFrom = NaN);
    Object.assign(canvas.style, { top: `${top}px`, width: `${width}px`, height: `${this.height}px`, transform: "" });
  }

  /** @param {Scale} scale */
  #slot(scale) {
    return (scale.mode === "percent" ? 2 : 0) + (scale === this.left ? 1 : 0);
  }

  /**
   * @param {number} v @param {Scale} [scale] @param {number} [step]
   */
  axisLabel(v, scale = this.scale, step = NaN) {
    return this.labels[this.#slot(scale)](v, step);
  }

  /**
   * As last drawn; fewer than two ticks, NaN.
   * @param {number} v @param {Scale} [scale]
   */
  stepAt(v, scale = this.scale) {
    const slot = this.#slot(scale), ticks = this.drawnTicks[slot], n = this.drawnTickCounts[slot];
    if (n < 2) return NaN;
    let k = 1;
    while (k < n - 1 && ticks[k] < v) k++;
    return Math.abs(ticks[k] - ticks[k - 1]);
  }

  /**
     * `ticks` writes them all at once, so their digits agree.
     * @param {(v: number, step?: number) => string} format @param {((values: number[]) => string[]) | null} [ticks] @param {"right" | "left"} [side]
     */
  setFormat(format, ticks = null, side = "right") {
    this.#write(side === "left" ? 1 : 0, format, ticks);
  }

  /** @param {(v: number, step?: number) => string} format @param {((values: number[]) => string[]) | null} [ticks] */
  setPercentFormat(format, ticks = null) {
    this.#write(2, format, ticks);
    this.#write(3, format, ticks);
  }

  /** @param {number} slot @param {(v: number, step?: number) => string} format @param {((values: number[]) => string[]) | null} ticks */
  #write(slot, format, ticks) {
    this.labels[slot] = kept(format);
    this.tickWriters[slot] = ticks;
    this.tickValues[slot] = new Float64Array(0);
    this.drawnFrom = NaN;
  }

  /** 0: the right; 1: the left. @param {number} side @param {number} y */
  lineAt(side, y) {
    const drawn = this.drawnTags[side];
    for (let k = drawn.lines.length - 1; k >= 0; k--) if (Math.abs(drawn.at[k] - y) <= drawn.step / 2) return drawn.lines[k];
    return null;
  }

  /** @param {number} slot @param {number} count */
  #tickTexts(slot, count) {
    const was = this.tickValues[slot], ticks = this.axisTicks;
    let same = was.length === count;
    for (let k = 0; same && k < count; k++) same = was[k] === ticks[k];
    if (same) return this.tickTexts[slot];
    const values = new Array(count);
    for (let k = 0; k < count; k++) values[k] = ticks[k];
    this.tickValues[slot] = new Float64Array(values);
    return (this.tickTexts[slot] = /** @type {(values: number[]) => string[]} */ (this.tickWriters[slot])(values));
  }

  /** Every relative reading starts at the first point whose center is in view. @param {import("./timeline.js").Timeline} time */
  #from(time) {
    return Math.max(0, time.firstVisible);
  }

  /**
     * A held scale stays held.
     * @param {import("./timeline.js").Timeline} time @param {"left" | "right"} side
     * @returns {{ from: number, to: number } | null} none without a scale on that side
     */
  fittedRange(time, side) {
    const left = side === "left", scale = left ? this.left : this.scale;
    if (!scale) return null;
    const auto = left ? this.leftAuto : this.auto, was = [scale.lo, scale.hi];
    left ? (this.leftAuto = true) : (this.auto = true);
    this.fit(time);
    const range = { from: scale.lo, to: scale.hi };
    left ? (this.leftAuto = auto) : (this.auto = auto);
    if (!auto) scale.fit(was[0], was[1]);
    return range;
  }

  /** @param {import("./timeline.js").Timeline} time */
  fit(time) {
    const n = this.series.length, a = Math.floor(time.from), b = Math.ceil(time.to);
    if (this.bases.length < n) (this.bases = new Float64Array(n * 2)), (this.drawnBases = new Float64Array(n * 2));
    this.basesSame = true;
    const state = this.fitting, bindings = state.bindings;
    let groups = state.groups;
    let same = bindings.length === n && state.right === this.scale && state.left === this.left;
    for (let k = 0; same && k < n; k++) {
      const binding = bindings[k], series = this.series[k];
      same = binding.series === series && binding.overlay === series.overlay && binding.side === this.sides[k];
    }
    if (!same) {
      const scales = new Map();
      scales.set(this.scale, { scale: this.scale, side: 0, indices: [] });
      if (this.left) scales.set(this.left, { scale: this.left, side: 1, indices: [] });
      bindings.length = n;
      for (let k = 0; k < n; k++) {
        const series = this.series[k], overlay = series.overlay, side = this.sides[k], binding = bindings[k];
        if (binding) (binding.series = series), (binding.overlay = overlay), (binding.side = side);
        else bindings[k] = { series, overlay, side };
        const scale = this.scaleOf(k);
        let group = scales.get(scale);
        if (!group) scales.set(scale, (group = { scale, side: -1, indices: [] }));
        group.indices.push(k);
      }
      (state.right = this.scale), (state.left = this.left);
      state.groups = groups = Array.from(scales.values());
      for (const group of groups) group.scale.reach = (time, a, b, out) => this.#reach(time, group, a, b, out);
    }
    for (let k = 0; k < groups.length; k++) {
      const group = groups[k];
      if (group.scale.height !== this.height) group.scale.height = this.height;
      this.#fit(time, group, a, b);
    }
  }

  /**
   * Indices are the pane's, so relative bases keep their slots.
   * @param {import("./timeline.js").Timeline} time
   * @param {ScaleGroup} group @param {number} a @param {number} b
   */
  #fit(time, group, a, b) {
    const scale = group.scale, indices = group.indices, side = group.side, auto = side < 0 || (side === 0 ? this.auto : this.leftAuto);
    const out = this.extent, log = scale.mode === "log", relative = scale.relative, from = relative ? this.#from(time) : 0, own = this.own, n = indices.length;
    if (relative && side < 0) throw new Error("bitview-charts: an overlay reads its values as they are (linear or log)");
    out[0] = Infinity;
    out[1] = -Infinity;
    let same = true, above = 0, below = 0, versions = 0;
    for (let at = 0; at < n; at++) {
      const k = indices[at], series = this.series[k];
      versions += series.version;
      if (series.hidden) continue;
      if (series.margins) {
        (ROOM[0] = 0), (ROOM[1] = 0), series.margins(time.spacing, ROOM, time, log);
        if (ROOM[0] > above) above = ROOM[0];
        if (ROOM[1] > below) below = ROOM[1];
      }
      if (!relative) {
        if (auto) widen(series, a, b, log, out, time);
        continue;
      }
      // (A series with no base (none in view, or 0) has no reading: neither fitted nor drawn.)
      const base = series.first(from);
      this.bases[k] = base;
      if (!Object.is(base, this.drawnBases[k])) same = false;
      if (!auto || !base || base !== base) continue;
      (own[0] = Infinity), (own[1] = -Infinity);
      widen(series, a, b, false, own, time);
      if (!(own[0] <= own[1])) continue;
      scale.setBase(base);
      const lo = own[0] * scale.m + scale.c, hi = own[1] * scale.m + scale.c;
      if (lo < out[0]) out[0] = lo;
      if (hi > out[1]) out[1] = hi;
    }
    if (!same) this.basesSame = false;
    scale.reached = auto ? versions : -1;
    if (auto) (scale.askedAbove = above), (scale.askedBelow = below);
    const asked = scale.askedAbove + scale.askedBelow;
    if (asked) {
      const free = (scale.height * (1 - scale.top - scale.bottom)) / 2, k = asked > free ? Math.max(0, free) / asked : 1;
      (scale.above = scale.askedAbove * k), (scale.below = scale.askedBelow * k);
    } else if (scale.above || scale.below) (scale.above = 0), (scale.below = 0);
    if (auto) scale.fit(out[0], out[1]);
    scale.empty = auto && !(log ? out[0] > 0 && out[0] <= out[1] : out[0] <= out[1]);
  }

  /**
     * Hidden series too (one may be shown); none where values alone don't say (held, or relative).
     * @param {import("./timeline.js").Timeline} time @param {ScaleGroup} group @param {number} a @param {number} b @param {Float64Array} out
     */
  #reach(time, group, a, b, out) {
    const scale = group.scale, auto = group.side < 0 || (group.side === 0 ? this.auto : this.leftAuto);
    if (!auto || scale.relative) return;
    for (const k of group.indices) widen(this.series[k], a, b, scale.mode === "log", out, time);
  }

  /**
     * Whether its picture was moved or stretched rather than drawn whole.
     * @param {import("./timeline.js").Timeline} time @param {Theme} theme @param {number} dpr @param {boolean} whole
     */
  draw(time, theme, dpr, whole) {
    const g = this.g, plot = this.plot, spacing = time.spacing, shift = (this.drawnFrom - time.from) * spacing, by = Math.round(shift);
    // (Its series over it drawn apart: theirs never stale its picture, only its axes.)
    let versions = 0, overs = 0, fixed = false, dashed = false, immovable = false, overlaid = true, shown = this.drawnHidden.length === this.series.length, own = false;
    for (let k = 0; k < this.series.length; k++) {
      const series = this.series[k];
      if (shown && this.drawnHidden[k] !== (series.hidden ? 1 : 0)) shown = false;
      // (Brought up to the view first: what it draws may hang on it, as bubbles sized to the largest in view.)
      if (!series.hidden) series.viewed?.(time);
      if (series.over) (overs += series.version), (own ||= !series.hidden);
      else if (!series.crosshair) (versions += series.version), (fixed ||= Boolean(series.fixed && !series.hidden)), (dashed ||= Boolean((series.dashed || series.immovable) && !series.hidden)), (immovable ||= Boolean(series.immovable && !series.hidden));
      if (series.overlay && !series.overlay.unchanged) overlaid = false;
    }
    if (versions !== this.drawnVersions || overs !== this.drawnOvers || !shown || !overlaid || !this.basesSame || !this.scale.unchanged || (this.left && !this.left.unchanged)) this.changes++;
    const same = !whole && overlaid && this.drawnFrom === this.drawnFrom && (!(this.scale.relative || this.left?.relative) || this.basesSame) && versions === this.drawnVersions && theme === this.drawnTheme && this.scale.unchanged && (!this.left || this.left.unchanged) && this.drawnPlot === plot;
    const moves = same && (!fixed || !shift) && alike(spacing, this.drawnSpacing) && Math.abs(shift - by) < 1e-3 && Math.abs(by) < plot / 2 && Math.abs(by * dpr - Math.round(by * dpr)) < 1e-3;
    // (A zoom stretches its last sharp picture only where drawing whole is slow (CPU canvas at a large size).)
    const slow = !fixed && this.wholeCost > SLOW;
    if (slow && same && !moves && !dashed && this.snapSpacing !== this.snapSpacing && this.sharp) this.#snap(dpr);
    const stretch = spacing / this.snapSpacing, stretches = slow && same && !moves && !dashed && stretch >= 0.5 && stretch <= 2;
    // (Yielded tags follow the focus: its tag drawn, the last one's gone.)
    const refocus = this.focus !== this.drawnFocus && (this.yields[0] || this.yields[1]);
    if (moves && (!by || !immovable)) {
      if (!by) {
        if (overs !== this.drawnOvers || refocus) (this.drawnOvers = overs), this.#axes(time, theme, dpr);
        // (A series `over` may follow the pointer: drawn whole each frame, so not still.)
        this.still = !own;
        return false;
      }
      const keep = (plot - Math.abs(by)) * dpr, h = this.canvas.height, from = by < 0 ? -by * dpr : 0, to = by > 0 ? by * dpr : 0;
      g.setTransform(1, 0, 0, 1, 0, 0);
      g.drawImage(this.canvas, from, 0, keep, h, to, 0, keep, h);
      by > 0 ? this.#strip(time, theme, dpr, 0, by + 1) : this.#strip(time, theme, dpr, plot + by - 1, plot);
    } else if (stretches) {
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
      this.sharp = true;
      this.snapSpacing = NaN;
      this.wholeCost = this.wholeCost * 0.5 + (performance.now() - began) * 0.5;
    }
    if (moves || stretches) {
      this.sharp = false;
      // (Its tags read the view's end: its axes drawn again where that's another point.)
      const first = Math.max(0, time.firstVisible);
      if (overs !== this.drawnOvers || refocus || ((time.lastVisible !== this.axesEnd || first > this.tagsFrom || first <= this.tagsOut) && this.#tagged())) this.#axes(time, theme, dpr);
    }
    this.scale.drawnNow();
    this.left?.drawnNow();
    for (const series of this.series) series.overlay?.drawnNow();
    this.drawnFrom = time.from;
    this.drawnSpacing = spacing;
    this.drawnPlot = plot;
    this.drawnVersions = versions;
    this.drawnOvers = overs;
    if (!shown) this.drawnHidden = Uint8Array.from(this.series, (series) => (series.hidden ? 1 : 0));
    this.drawnTheme = theme;
    this.still = false;
    for (let k = 0; k < this.series.length; k++) this.drawnBases[k] = this.bases[k];
    return moves || stretches;
  }


  /** @param {import("./timeline.js").Timeline} time @param {Theme} theme @param {number} dpr */
  #axes(time, theme, dpr) {
    (this.tagsFrom = Infinity), (this.tagsOut = -1), (this.drawnFocus = this.focus);
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
        (this.drawnTags[1].lines.length = 0), (this.yields[1] = false);
      }
    }
    this.axesEnd = time.lastVisible;
  }

  #tagged() {
    for (let k = 0; k < this.series.length; k++) {
      const series = this.series[k];
      if (series.hidden || series.faint) continue;
      const line = lineOf(series);
      if (line ? line.tag : series instanceof Custom && series.hooks.axisTags) return true;
    }
    return false;
  }

  /**
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
    let need = 0;
    const count = scale.ticks(this.axisTicks), at = side ? x + width - 8 : x + 8, slot = this.#slot(scale), texts = this.tickWriters[slot] ? this.#tickTexts(slot, count) : null;
    if (this.drawnTicks[slot].length < count) this.drawnTicks[slot] = new Float64Array(this.axisTicks.length);
    this.drawnTicks[slot].set(this.axisTicks.subarray(0, count));
    this.drawnTickCounts[slot] = count;
    const stepAt = (/** @type {number} */ v) => this.stepAt(v, scale);
    // (Tags placed first: a label within COVER of one is dropped.)
    const page = this.pageTags[side], labels = this.drawnLabels[side], place = this.tagPlace;
    (page.count = 0), (page.length = this.height);
    for (let k = 0; k < this.series.length; k++) if (!this.series[k].hidden && !this.series[k].faint) this.series[k].axisTags?.(side, page.add, scale, stepAt);
    for (let k = 0; k < page.count; k++) page.at[k] = Math.min(Math.max(page.at[k], TAG / 2), this.height - TAG / 2);
    const fits = (/** @type {number} */ y) => y > 6 && y < this.height - 6 && !page.covers(y, COVER);
    const tagged = (/** @type {number} */ y, /** @type {number} */ n) => {
      for (let t = 0; t < n; t++) if (Math.abs(place[t] - y) < COVER) return true;
      return false;
    };
    let tags = this.#placeTags(scale, time, side, false), fit = 0, buried = 0;
    for (let k = 0; k < count; k++) {
      const y = scale.own(this.axisTicks[k]);
      if (fits(y)) fit++, tagged(y, tags) && buried++;
    }
    // (Too many to read beside their labels: the legend, name and value, holds the rest.)
    this.yields[side] = tags > 2 && buried > BURIED * fit;
    if (this.yields[side]) tags = this.#placeTags(scale, time, side, true);
    const free = (/** @type {number} */ y) => fits(y) && !tagged(y, tags);
    // (Each label on its gridline, or none: never moved off the line it names.)
    if (labels.at.length < count) labels.at = new Float64Array(count * 2);
    labels.count = 0;
    for (let k = 0; k < count; k++) {
      const y = scale.own(this.axisTicks[k]);
      if (!free(y)) continue;
      labels.at[labels.count++] = y;
      const words = texts ? (texts[k] ?? "") : this.axisLabel(this.axisTicks[k], scale);
      g.fillText(words, at, y);
      need = Math.max(need, g.measureText(words).width + 16);
    }
    this.axisNeed[side] = need;
    this.#tags(g, scale, time, theme, x, width, side, tags);
    for (let k = 0; k < page.count; k++) this.axisNeed[side] = Math.max(this.axisNeed[side], pill(g, page.texts[k], page.colors[k], page.inks[k], side ? x + width : x, page.at[k], side ? "right" : "left") + 4);
    if (this.separated) this.#separator(g, theme, dpr, x, x + width);
  }

  /**
     * Labels under the crosshair's tag at `y` painted over, never past the pane.
     * @param {CanvasRenderingContext2D} g @param {number} side @param {number} y @param {number} top @param {number} x @param {number} width @param {Theme} theme
     */
  cover(g, side, y, top, x, width, theme) {
    const labels = this.drawnLabels[side];
    g.fillStyle = theme.background;
    for (let k = 0; k < labels.count; k++) {
      if (Math.abs(labels.at[k] - y) >= COVER) continue;
      const from = Math.max(0, labels.at[k] - COVER / 2), to = Math.min(this.height, labels.at[k] + COVER / 2);
      g.fillRect(x, top + from, width, to - from);
    }
  }

  /**
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
    let right = !this.left;
    for (let k = 0; !right && k < this.series.length; k++) right = this.sides[k] === 0 && !this.series[k].hidden && this.series[k].scaled !== false && !this.series[k].overlay;
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
      const series = this.series[k];
      if (series.hidden || series.over || series.crosshair) continue;
      const scale = this.scaleOf(k);
      if (scale.relative) {
        const base = series.first(from);
        if (!base || base !== base) continue;
        scale.setBase(base);
      }
      if (series.faint) g.globalAlpha = FAINT;
      series.draw(g, time, scale, x0, x1, dpr, theme);
      g.globalAlpha = 1;
      // (Dots phased by the timeline, so a panned picture has them where a whole draw does.)
      const line = lineOf(series), i = line?.priceLine && !series.faint ? line.last(scale.mode === "log") : -1;
      if (!line || i < 0) continue;
      const y = Math.round(scale.y(line.values[i]) * dpr) / dpr + 0.5 / dpr;
      g.setLineDash(DOTS);
      g.lineDashOffset = (x0 + time.from * time.spacing) % (DOTS[0] + DOTS[1]);
      g.strokeStyle = line.colorAt(i);
      g.lineWidth = 1;
      g.beginPath();
      g.moveTo(x0, y);
      g.lineTo(x1, y);
      g.stroke();
      g.setLineDash(SOLID);
      g.lineDashOffset = 0;
    }
    if (this.separated) this.#separator(g, theme, dpr, x0, x1);
    g.restore();
  }

  /**
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
     * Into tagLines, tagIndex, tagAt and tagPlace; returns their count. `yielded`: the price's and the focus' alone.
     * @param {Scale} scale @param {import("./timeline.js").Timeline} time @param {number} side @param {boolean} yielded
     */
  #placeTags(scale, time, side, yielded) {
    const log = scale.mode === "log", relative = scale.relative, from = this.#from(time), lines = this.tagLines, owners = this.tagSeries, end = time.lastVisible;
    lines.length = 0;
    owners.length = 0;
    for (let k = 0; k < this.series.length; k++) {
      if (this.sides[k] !== side) continue;
      const series = this.series[k], line = lineOf(series);
      if (!line || !line.tag || series.hidden || series.faint || series.overlay || (yielded && !line.priceLine && series !== this.focus && line !== this.focus)) continue;
      // (A price line's at its line, whatever the view's end: never the line without its tag.)
      const base = relative ? line.first(from) : 1, i = line.priceLine ? line.last(log) : line.lastBy(end, log);
      if (i >= 0 && i < from && i > this.tagsOut) this.tagsOut = i;
      if (i < from || !base || base !== base) continue;
      if (i < this.tagsFrom) this.tagsFrom = i;
      if (!painted(line.colorAt(i))) continue;
      if (this.tagIndex.length <= lines.length) ((grown) => (grown.set(this.tagIndex), (this.tagIndex = grown)))(new Int32Array(lines.length * 2 + 8));
      this.tagIndex[lines.length] = i;
      lines.push(line);
      owners.push(series);
    }
    const n = lines.length;
    if (!n) return 0;
    if (this.tagY.length < n) (this.tagY = new Float64Array(n * 2)), (this.tagAt = new Int32Array(n * 2)), (this.tagPlace = new Float64Array(n * 2));
    const ys = this.tagY, order = this.tagAt, at = this.tagPlace, index = this.tagIndex;
    for (let k = 0; k < n; k++) {
      if (relative) scale.setBase(lines[k].first(from));
      (ys[k] = scale.y(lines[k].values[index[k]])), (order[k] = k);
    }
    for (let k = 1; k < n; k++)
      for (let j = k; j > 0 && ys[order[j]] < ys[order[j - 1]]; j--) {
        const swap = order[j];
        (order[j] = order[j - 1]), (order[j - 1] = swap);
      }
    // (Each at its line or just under the one above; past the foot the lowest go; only equal readings share one.)
    const top = TAG / 2, foot = this.height - TAG / 2;
    this.tagStep = TAG;
    if (foot < top) return 0;
    let m = 0, last = NaN;
    for (let k = 0; k < n; k++) {
      const y0 = ys[order[k]];
      if (!(y0 > -0.5 && y0 < this.height + 0.5)) continue;
      if (Math.abs(y0 - last) < 1e-6) continue;
      const line = Math.min(Math.max(y0, top), foot), y = m ? Math.max(line, at[m - 1] + TAG) : line;
      if (y > foot) break;
      (order[m] = order[k]), (at[m++] = y), (last = y0);
    }
    return m;
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {Scale} scale @param {import("./timeline.js").Timeline} time @param {Theme} theme
   * @param {number} x @param {number} width @param {number} side @param {number} n
   */
  #tags(g, scale, time, theme, x, width, side, n) {
    const relative = scale.relative, from = this.#from(time), lines = this.tagLines, order = this.tagAt, at = this.tagPlace, index = this.tagIndex, step = this.tagStep;
    g.font = theme.font;
    g.textBaseline = "middle";
    const drawn = this.drawnTags[side];
    drawn.lines.length = 0;
    if (drawn.at.length < n) drawn.at = new Float64Array(n * 2);
    drawn.step = step;
    for (let k = 0; k < n; k++) {
      const line = lines[order[k]], i = index[order[k]], y = at[k];
      (drawn.lines[k] = this.tagSeries[order[k]]), (drawn.at[k] = y);
      // (Its line's color without its alpha: a faint line's tag still reads.)
      const color = solid(line.colorAt(i));
      if (relative) scale.setBase(line.first(from));
      const v = relative ? line.values[i] * scale.m + scale.c : line.values[i], by = this.stepAt(v, scale);
      const words = relative ? this.axisLabel(v, scale, by) : (line.format ?? this.labels[side])(v, by);
      const look = tint(color);
      this.axisNeed[side] = Math.max(this.axisNeed[side], pill(g, words, look.fill, look.ink, side ? x + width : x, y, side ? "right" : "left", Math.max(1, step)) + 4);
    }
  }

  /** @param {number} dpr */
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
