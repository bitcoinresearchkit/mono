// @ts-check
// A pane: a band of the chart, its own canvas, its scale (fitted to its series in view unless set), its series (lines,
// bands: drawn in the order added, the last on top), its axis
// at the right. Drawn whole when its scale or size changed; else, the view moved by whole pixels, its picture is moved
// and only the strip come into view drawn (drawn whole again once the motion stops: no seams left).

import { Scale } from "./scale.js";
import { Line } from "./line.js";
import { Band } from "./band.js";
import { Bars } from "./bars.js";
import { Candles } from "./candles.js";
import { compact, kept } from "./format.js";

/** @typedef {{ background: string, text: string, grid: string, font: string }} Theme */

/** Drawing a pane whole slower than this (ms): a zoom stretches its picture instead (see draw). */
const SLOW = 8;
/**
 * What a pane draws: its lowest and highest over a range of indices (widening `out`), and its picture over part of the
 * plot; `version`, changed at each change of its values or look.
 * @typedef {{
 *   version: number,
 *   extent(a: number, b: number, positive: boolean, out: Float64Array): void,
 *   draw(g: CanvasRenderingContext2D, time: import("./timeline.js").Timeline, scale: Scale, x0: number, x1: number, dpr: number): void,
 * }} Series
 */

export class Pane {
  /**
   * @param {{ share?: number, scale?: ConstructorParameters<typeof Scale>[0], format?: (v: number) => string }} [options]
   */
  constructor({ share = 1, scale, format = compact } = {}) {
    this.canvas = document.createElement("canvas");
    this.canvas.style.cssText = "position: absolute; left: 0; display: block";
    const g = this.canvas.getContext("2d", { alpha: false });
    if (!g) throw new Error("bitview-charts: no 2D canvas");
    this.g = g;
    /** Its part of the chart's height, against the other panes'. */
    this.share = share;
    this.scale = new Scale(scale);
    /** Fitted to its series in view at each frame (else as set: see setRange). */
    this.auto = true;
    /** @type {Series[]} */
    this.series = [];
    this.label = kept(format);
    /** Where it lies in the chart (CSS pixels), and its plot's width (the axis past it). */
    this.top = 0;
    this.height = 0;
    this.width = 0;
    this.plot = 0;
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
    /** What drawing it whole takes (ms, the last few weighed): a zoom stretches its picture only where it's slow. */
    this.wholeCost = 0;
    /** @type {Theme | null} */
    this.drawnTheme = null;
    this.ticks = new Float64Array(24);
    this.extent = new Float64Array(2);
  }

  /** @template {Series} S @param {S} series */
  add(series) {
    this.series.push(series);
    this.drawnFrom = NaN;
    return series;
  }

  /** @param {ConstructorParameters<typeof Line>[0]} [options] */
  addLine(options) {
    return this.add(new Line(options));
  }

  /** @param {ConstructorParameters<typeof Band>[0]} options */
  addBand(options) {
    return this.add(new Band(options));
  }

  /** @param {ConstructorParameters<typeof Bars>[0]} [options] */
  addBars(options) {
    return this.add(new Bars(options));
  }

  /** @param {ConstructorParameters<typeof Candles>[0]} options */
  addCandles(options) {
    return this.add(new Candles(options));
  }

  /** @param {Series} series */
  remove(series) {
    this.series = this.series.filter((each) => each !== series);
    this.drawnFrom = NaN;
  }

  /** Drawn whole at the next frame: a series' values or look changed. */
  changed() {
    this.drawnFrom = NaN;
  }

  /** Its scale set by hand (`lo`, `hi`), or fitted to its series again (no arguments). @param {number} [lo] @param {number} [hi] */
  setRange(lo, hi) {
    this.auto = lo === undefined || hi === undefined;
    if (!this.auto) this.scale.fit(/** @type {number} */ (lo), /** @type {number} */ (hi));
  }

  /**
   * Laid out at `top`, `width` by `height` CSS pixels, the axis `axis` wide, at `dpr` device pixels a CSS pixel.
   * @param {number} top @param {number} width @param {number} height @param {number} axis @param {number} dpr
   */
  place(top, width, height, axis, dpr) {
    const w = Math.round(width * dpr), h = Math.round(height * dpr);
    if (this.canvas.width !== w || this.canvas.height !== h) (this.canvas.width = w), (this.canvas.height = h), (this.drawnFrom = NaN);
    Object.assign(this.canvas.style, { top: `${top}px`, width: `${width}px`, height: `${height}px` });
    this.top = top;
    this.width = width;
    this.height = height;
    this.plot = Math.max(1, width - axis);
    this.scale.height = height;
  }

  /** Fitted to its series in the timeline's view. @param {import("./timeline.js").Timeline} time */
  fit(time) {
    if (!this.auto) return;
    const out = this.extent, log = this.scale.mode === "log";
    out[0] = Infinity;
    out[1] = -Infinity;
    const a = Math.ceil(time.from), b = Math.floor(time.to);
    for (let k = 0; k < this.series.length; k++) this.series[k].extent(a, b, log, out);
    this.scale.fit(out[0], out[1]);
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
    let versions = 0;
    for (let k = 0; k < this.series.length; k++) versions += this.series[k].version;
    const same = !whole && this.drawnFrom === this.drawnFrom && versions === this.drawnVersions && theme === this.drawnTheme && this.scale.unchanged && this.drawnPlot === plot;
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
      // (The axis: its ticks' values, at their heights.)
      g.fillStyle = theme.background;
      g.fillRect(plot, 0, this.width - plot, this.height);
      g.fillStyle = theme.text;
      g.font = theme.font;
      g.textBaseline = "middle";
      g.textAlign = "left";
      for (let k = 0; k < this.tickCount; k++) {
        const y = this.scale.y(this.ticks[k]);
        if (y > 6 && y < this.height - 6) g.fillText(this.label(this.ticks[k]), plot + 8, y);
      }
      // (Sharp: a zoom from here keeps this picture as it begins.)
      this.sharp = true;
      this.snapSpacing = NaN;
      this.wholeCost = this.wholeCost * 0.5 + (performance.now() - began) * 0.5;
    }
    if (moves || stretches) this.sharp = false;
    this.scale.drawnNow();
    this.drawnFrom = time.from;
    this.drawnSpacing = spacing;
    this.drawnPlot = plot;
    this.drawnVersions = versions;
    this.drawnTheme = theme;
    return moves || stretches;
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
    this.tickCount = this.scale.ticks(this.ticks);
    g.strokeStyle = theme.grid;
    g.lineWidth = 1;
    g.beginPath();
    for (let k = 0; k < this.tickCount; k++) {
      const y = Math.round(this.scale.y(this.ticks[k]) * dpr) / dpr + 0.5 / dpr;
      g.moveTo(x0, y);
      g.lineTo(x1, y);
    }
    g.stroke();
    for (let k = 0; k < this.series.length; k++) this.series[k].draw(g, time, this.scale, x0, x1, dpr);
    g.restore();
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
