// @ts-check
// A grid: a heatmap's cells, a column a point (zoomed out, a column a block of points, by powers of two: the same blocks
// at any pan), each column's rows the page's (`column(i, step)`: the rows' bins and their values at point `i`, a block
// of `step` ending there; null: not here yet, the page to call invalidate() when it comes), its bins' bounds `edges`
// (values on the pane's scale, rising). A cell's color: `place(value)`, its place along `gradient` (0 to 1; NaN, none),
// the gradient's stops blended in OKLab. Each column drawn as a strip made once (a pixel a step of the scale, its
// strongest cell's color) and stretched into place, never cell by cell (a view holds hundreds of thousands); a strip
// kept while its column, its step and the scale's resolution stay (a pan, a zoom within a power of two). A backdrop:
// not fitted, never hit (a page reads a cell where it's asked).

import { gradientTable } from "./color.js";

/** Columns at least this many CSS pixels wide, zoomed out. */
const COLUMN = 2;
/** A strip's pixels at most. */
const TALL = 4096;
/** Strips kept, at most (their canvases reused past that: a canvas a column made anew each time would leave WebKit
 * past its canvas memory, every canvas on the page blank). */
const KEPT = 768;

export class Grid {
  /**
   * @param {{ edges: Float64Array, column: (i: number, step: number) => { rows: Int32Array, values: Float64Array } | null, place: (value: number) => number, gradient: string[] }} options
   */
  constructor({ edges, column, place, gradient }) {
    /** @type {Float64Array} */
    this.edges = edges;
    this.column = column;
    this.place = place;
    /** @type {string[]} */
    this.gradient = gradient;
    this.table = gradientTable(gradient);
    /** Strips by column, step and resolution, the latest last. @type {Map<string, { canvas: HTMLCanvasElement, t0: number, t1: number }>} */
    this.strips = new Map();
    this.hidden = false;
    this.version = 0;
  }

  /** What it draws changed (a column come, its values anew): its strips made again, its pane drawn whole. */
  invalidate() {
    this.version++;
    this.#forget();
  }

  /** @param {{ edges?: Float64Array, place?: (value: number) => number, gradient?: string[] }} look */
  setStyle({ edges = this.edges, place = this.place, gradient = this.gradient }) {
    (this.edges = edges), (this.place = place);
    if (gradient !== this.gradient) (this.gradient = gradient), (this.table = gradientTable(gradient));
    this.invalidate();
  }

  /** Its strips let go (their canvases kept to be painted anew). */
  #forget() {
    for (const strip of this.strips.values()) spare.push(strip.canvas);
    this.strips.clear();
  }

  /** Nothing to fit: its pane's other series say where it is. */
  extent() {}

  /** Any base. */
  first() {
    return 1;
  }

  /** Never hit: a backdrop. */
  distance() {
    return Infinity;
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr
   */
  draw(g, time, scale, x0, x1, dpr) {
    const n = time.length, edges = this.edges, log = scale.mode === "log";
    if (!n || edges.length < 2) return;
    // (Its columns: a block of `step` points each, at least COLUMN pixels wide; their strips at a resolution a power of
    // two near the screen's, in the scale's own terms (a log scale's decades, a linear one's values).)
    let step = 1;
    while (step * time.spacing < COLUMN && step < 1 << 20) step *= 2;
    const t = (/** @type {number} */ v) => (log ? Math.log10(v) : v), lo = t(scale.lo), hi = t(scale.hi), pixels = Math.abs(scale.y(scale.lo) - scale.y(scale.hi)) * dpr;
    if (!(hi > lo) || !(pixels > 0)) return;
    const resolution = 2 ** Math.ceil(Math.log2(pixels / (hi - lo)));
    // (Its strips a little taller than their place (up to twice): smoothed, so no cell's dropped.)
    const first = Math.max(0, Math.floor(time.index(x0) / step) - 1), last = Math.min(Math.ceil(n / step) - 1, Math.ceil(time.index(x1) / step) + 1), smooth = g.imageSmoothingEnabled;
    g.imageSmoothingEnabled = true;
    for (let block = first; block <= last; block++) {
      const start = block * step, end = Math.min(n - 1, start + step - 1), strip = this.#strip(end, step, log, resolution);
      if (!strip) continue;
      const left = time.snap(time.x(start - 0.5), dpr), right = time.snap(time.x(end + 0.5), dpr), top = scale.y(log ? 10 ** strip.t1 : strip.t1), bottom = scale.y(log ? 10 ** strip.t0 : strip.t0);
      g.drawImage(strip.canvas, 0, 0, 1, strip.canvas.height, left, top, right - left, bottom - top);
    }
    g.imageSmoothingEnabled = smooth;
  }

  /**
   * Column `owner`'s strip (a block of `step` ending there) at `resolution` pixels a unit of the scale's terms: made
   * once, kept (the latest last); none where the page has no column there (yet).
   * @param {number} owner @param {number} step @param {boolean} log @param {number} resolution
   */
  #strip(owner, step, log, resolution) {
    const key = `${owner}|${step}|${log ? 1 : 0}|${resolution}`, had = this.strips.get(key);
    if (had) return this.strips.delete(key), this.strips.set(key, had), had;
    const column = this.column(owner, step);
    if (!column || !column.rows.length) return null;
    const { rows, values } = column, edges = this.edges, t = (/** @type {number} */ v) => (log ? Math.log10(v) : v);
    let res = resolution;
    const tLow = t(edges[rows[0]]), tHigh = t(edges[rows[rows.length - 1] + 1]);
    if (!(tHigh > tLow)) return null;
    while ((tHigh - tLow) * res > TALL) res /= 2;
    const t0 = Math.floor(tLow * res) / res, t1 = Math.ceil(tHigh * res) / res, height = Math.max(1, Math.round((t1 - t0) * res));
    // (Each pixel's strongest cell, by its size (its sign aside), its color from it.)
    if (strongest.length < height) strongest = new Float64Array(height * 2);
    strongest.fill(0, 0, height);
    for (let k = 0; k < rows.length; k++) {
      const value = values[k];
      if (!value || value !== value) continue;
      const from = Math.max(0, Math.floor((t1 - t(edges[rows[k] + 1])) * res)), to = Math.min(height - 1, Math.ceil((t1 - t(edges[rows[k]])) * res) - 1);
      for (let p = from; p <= Math.max(from, to); p++) if (Math.abs(value) > Math.abs(strongest[p])) strongest[p] = value;
    }
    if (this.strips.size >= KEPT) for (const [other, strip] of this.strips) {
      this.strips.delete(other), spare.push(strip.canvas);
      break;
    }
    const canvas = spare.pop() ?? document.createElement("canvas");
    (canvas.width = 1), (canvas.height = height);
    const context = /** @type {CanvasRenderingContext2D} */ (canvas.getContext("2d")), image = context.createImageData(1, height), pixels = image.data, table = this.table;
    for (let p = 0; p < height; p++) {
      if (!strongest[p]) continue;
      const place = this.place(strongest[p]);
      if (!(place >= 0)) continue;
      const c = Math.round(Math.min(1, place) * 255) * 4;
      (pixels[p * 4] = table[c]), (pixels[p * 4 + 1] = table[c + 1]), (pixels[p * 4 + 2] = table[c + 2]), (pixels[p * 4 + 3] = table[c + 3]);
    }
    context.putImageData(image, 0, 0);
    const strip = { canvas, t0, t1 };
    this.strips.set(key, strip);
    return strip;
  }
}

/** Each pixel's strongest value, as a strip's made (made once, grown). */
let strongest = new Float64Array(256);
/** Canvases let go of, to be painted anew (shared by every grid). @type {HTMLCanvasElement[]} */
const spare = [];
