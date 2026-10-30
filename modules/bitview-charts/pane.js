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
import { Custom } from "./custom.js";
import { compact, kept } from "./format.js";

/**
 * A chart's look: its ground, its words' color, its grid's, its font; the crosshair's lines, and its badges' ground and
 * words; the line between panes.
 * @typedef {{ background: string, text: string, grid: string, font: string, crosshair: string, badge: string, badgeText: string, separator: string }} Theme
 */

/** Drawing a pane whole slower than this (ms): a zoom stretches its picture instead (see draw). */
const SLOW = 8;
/** A price line's dots; none (made once). */
const DOTS = [2, 3], SOLID = /** @type {number[]} */ ([]);
/** A tag's height, and the least room between two (CSS pixels). */
const TAG = 18;
/**
 * What a pane draws: its lowest and highest over a range of indices (widening `out`), and its picture over part of the
 * plot; `version`, changed at each change of its values or look.
 * @typedef {{
 *   version: number,
 *   first(i: number): number,
 *   extent(a: number, b: number, positive: boolean, out: Float64Array): void,
 *   draw(g: CanvasRenderingContext2D, time: import("./timeline.js").Timeline, scale: Scale, x0: number, x1: number, dpr: number): void,
 * }} Series
 */

export class Pane {
  /**
   * `format` writes its values (a relative scale's are written as they are: a percent, an index).
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
    // (A relative scale's labels: a percent, an index.)
    this.percentLabel = kept((v) => `${compact(v)}%`);
    this.indexLabel = kept(compact);
    // (A relative scale's: each series' base, as fitted and as drawn: its picture moved only where each is the same.)
    this.bases = new Float64Array(4);
    this.drawnBases = new Float64Array(4);
    this.basesSame = false;
    this.own = new Float64Array(2);
    /** Where it lies in the chart (CSS pixels), and its plot's width (the axis past it). */
    this.top = 0;
    this.height = 0;
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
    this.tagY = new Float64Array(8);
    this.tagPlace = new Float64Array(8);
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

  /** @param {ConstructorParameters<typeof Custom>[0]} hooks */
  addCustom(hooks) {
    return this.add(new Custom(hooks));
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

  /** A value as its axis writes it. @param {number} v */
  axisLabel(v) {
    return this.scale.mode === "percent" ? this.percentLabel(v) : this.scale.mode === "indexed" ? this.indexLabel(v) : this.label(v);
  }

  /** The first point a relative scale reads its series from: its first in view. @param {import("./timeline.js").Timeline} time */
  #from(time) {
    return Math.max(0, Math.floor(time.from));
  }

  /** Fitted to its series in the timeline's view (a relative scale's, each read against its first value in view). @param {import("./timeline.js").Timeline} time */
  fit(time) {
    const out = this.extent, scale = this.scale, log = scale.mode === "log", relative = scale.relative, from = this.#from(time), own = this.own, n = this.series.length;
    out[0] = Infinity;
    out[1] = -Infinity;
    // (Not relative: each read as it is (a relative mode left, its reading gone too).)
    if (!relative) scale.setBase(NaN);
    if (this.bases.length < n) (this.bases = new Float64Array(n * 2)), (this.drawnBases = new Float64Array(n * 2));
    let same = true;
    const a = Math.ceil(time.from), b = Math.floor(time.to);
    for (let k = 0; k < n; k++) {
      const series = this.series[k];
      if (!relative) {
        if (this.auto) series.extent(a, b, log, out);
        continue;
      }
      // (A series with no base (none in view, or 0) has no reading: neither fitted nor drawn.)
      const base = series.first(from);
      this.bases[k] = base;
      if (!Object.is(base, this.drawnBases[k])) same = false;
      if (!this.auto || !base || base !== base) continue;
      (own[0] = Infinity), (own[1] = -Infinity);
      series.extent(a, b, false, own);
      if (!(own[0] <= own[1])) continue;
      scale.setBase(base);
      const lo = own[0] * scale.m + scale.c, hi = own[1] * scale.m + scale.c;
      if (lo < out[0]) out[0] = lo;
      if (hi > out[1]) out[1] = hi;
    }
    this.basesSame = same;
    if (this.auto) scale.fit(out[0], out[1]);
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
    const same = !whole && this.drawnFrom === this.drawnFrom && (!this.scale.relative || this.basesSame) && versions === this.drawnVersions && theme === this.drawnTheme && this.scale.unchanged && this.drawnPlot === plot;
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
        const y = this.scale.own(this.ticks[k]);
        if (y > 6 && y < this.height - 6) g.fillText(this.axisLabel(this.ticks[k]), plot + 8, y);
      }
      this.#tags(time, theme);
      if (this.separated) this.#separator(theme, dpr, plot, this.width);
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
    for (let k = 0; k < this.series.length; k++) this.drawnBases[k] = this.bases[k];
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
      const y = Math.round(this.scale.own(this.ticks[k]) * dpr) / dpr + 0.5 / dpr;
      g.moveTo(x0, y);
      g.lineTo(x1, y);
    }
    g.stroke();
    const relative = this.scale.relative, from = this.#from(time);
    for (let k = 0; k < this.series.length; k++) {
      if (relative) {
        const base = this.series[k].first(from);
        if (!base || base !== base) continue;
        this.scale.setBase(base);
      }
      this.series[k].draw(g, time, this.scale, x0, x1, dpr);
    }
    // (Price lines: across at a line's last value, dotted, in its color there.)
    const log = this.scale.mode === "log";
    g.setLineDash(DOTS);
    for (let k = 0; k < this.series.length; k++) {
      const line = this.series[k];
      if (!(line instanceof Line) || !line.priceLine) continue;
      const i = line.last(log);
      if (i < 0) continue;
      if (relative) {
        const base = line.first(from);
        if (!base || base !== base) continue;
        this.scale.setBase(base);
      }
      const y = Math.round(this.scale.y(line.values[i]) * dpr) / dpr + 0.5 / dpr;
      g.strokeStyle = line.colorAt(i);
      g.lineWidth = 1;
      g.beginPath();
      g.moveTo(x0, y);
      g.lineTo(x1, y);
      g.stroke();
    }
    g.setLineDash(SOLID);
    if (this.separated) this.#separator(theme, dpr, x0, x1);
    g.restore();
  }

  /** The separator's line along its foot from `x0` to `x1`. @param {Theme} theme @param {number} dpr @param {number} x0 @param {number} x1 */
  #separator(theme, dpr, x0, x1) {
    const g = this.g, y = this.height - 0.5 / dpr;
    g.strokeStyle = theme.separator;
    g.lineWidth = 1 / dpr;
    g.beginPath();
    g.moveTo(x0, y);
    g.lineTo(x1, y);
    g.stroke();
  }

  /**
   * Its lines' tags on its axis: each one's last value in a badge in its color, laid top to bottom where they'd
   * overlap (none pushed past its axis' ends); on a relative scale, as it reads them.
   * @param {import("./timeline.js").Timeline} time @param {Theme} theme
   */
  #tags(time, theme) {
    const g = this.g, scale = this.scale, log = scale.mode === "log", relative = scale.relative, from = this.#from(time), lines = this.tagLines;
    lines.length = 0;
    for (let k = 0; k < this.series.length; k++) {
      const line = this.series[k];
      if (!(line instanceof Line) || !line.tag || line.last(log) < 0) continue;
      // (On a relative scale, one with no base has no reading: no tag.)
      const base = relative ? line.first(from) : 1;
      if (base && base === base) lines.push(line);
    }
    const n = lines.length;
    if (!n) return;
    if (this.tagY.length < n) (this.tagY = new Float64Array(n * 2)), (this.tagAt = new Int32Array(n * 2)), (this.tagPlace = new Float64Array(n * 2));
    const ys = this.tagY, order = this.tagAt, at = this.tagPlace;
    for (let k = 0; k < n; k++) {
      if (relative) scale.setBase(lines[k].first(from));
      (ys[k] = scale.y(lines[k].values[lines[k].last(log)])), (order[k] = k);
    }
    // (Top to bottom, each at least a tag below the one over it (less, where they'd not all fit: the axis shared); then,
    // past the foot, each back up.)
    const step = Math.min(TAG, this.height / n);
    for (let k = 1; k < n; k++)
      for (let j = k; j > 0 && ys[order[j]] < ys[order[j - 1]]; j--) {
        const swap = order[j];
        (order[j] = order[j - 1]), (order[j - 1] = swap);
      }
    for (let k = 0; k < n; k++) at[k] = Math.max(ys[order[k]], k ? at[k - 1] + step : step / 2);
    for (let k = n - 1; k >= 0; k--) at[k] = Math.min(at[k], k < n - 1 ? at[k + 1] - step : this.height - step / 2);
    g.font = theme.font;
    g.textAlign = "left";
    g.textBaseline = "middle";
    for (let k = 0; k < n; k++) {
      const line = lines[order[k]], i = line.last(log), y = at[k];
      g.fillStyle = line.colorAt(i);
      g.beginPath();
      g.roundRect(this.plot + 2, y - step / 2 + 1, this.width - this.plot - 4, Math.max(1, step - 2), Math.max(0, Math.min(4, step / 2 - 1)));
      g.fill();
      // (Too tight to read: its color alone.)
      if (step < 10) continue;
      g.fillStyle = theme.background;
      if (relative) scale.setBase(line.first(from));
      g.fillText(relative ? this.axisLabel(line.values[i] * scale.m + scale.c) : this.label(line.values[i]), this.plot + 8, y);
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
