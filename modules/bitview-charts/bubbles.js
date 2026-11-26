// @ts-check

import { Line } from "./line.js";
import { measure, measuring } from "./recorder.js";
import { painted } from "./color.js";

let xs = new Float64Array(64), ys = new Float64Array(64), radii = new Float64Array(64), ups = new Uint8Array(64), order = new Int32Array(64);
const EXTENT = new Float64Array(2);
/** @param {number} size */
const magnitude = (size) => ((m) => (m > 0 && m < Infinity ? m : NaN))(Math.abs(size));
/** Ties by index so a strip and a whole draw order alike. @param {number} a @param {number} b */
const largest = (a, b) => radii[b] - radii[a] || a - b;

export class Bubbles {
  /**
   * @param {{ values: Float64Array, sizes: Float64Array, up?: string, upEdge?: string, down?: string, downEdge?: string, size?: number, hide?: number, scaleTo?: "view" | "all", halo?: string | null }} options
   */
  constructor({ values, sizes, up = "rgba(38, 166, 154, 0.4)", upEdge = "rgba(38, 166, 154, 0.8)", down = "rgba(239, 83, 80, 0.4)", downEdge = "rgba(239, 83, 80, 0.8)", size = 48, hide = 0, scaleTo = "view", halo = null }) {
    (this.up = up), (this.upEdge = upEdge), (this.down = down), (this.downEdge = downEdge);
    this.size = size;
    this.hide = hide;
    /** @type {"view" | "all"} */
    this.scaleTo = scaleTo;
    /** @type {string | null} */
    this.halo = halo;
    this.positions = new Line();
    this.magnitudes = new Line();
    /** @type {Float64Array} */
    this.sizes = sizes;
    this.most = 0;
    this.hidden = false;
    this.faint = false;
    /** @type {import("./scale.js").Scale | null} */
    this.overlay = null;
    /** @type {((a: number, b: number, positive: boolean, out: Float64Array) => void) | null} */
    this.fit = null;
    this.version = 0;
    this.setValues({ values, sizes });
  }

  get values() {
    return this.positions.values;
  }

  /** @param {{ values: Float64Array, sizes: Float64Array }} columns */
  setValues({ values, sizes }) {
    this.version++;
    this.sizes = sizes;
    // In a loop: from() with a mapping is several times slower without a JIT.
    const magnitudes = new Float64Array(sizes.length);
    for (let i = 0; i < magnitudes.length; i++) magnitudes[i] = magnitude(sizes[i]);
    this.positions.setValues(values);
    this.magnitudes.setValues(magnitudes);
    if (this.scaleTo === "all") this.most = this.#largest(0, sizes.length - 1);
  }

  /** @param {number} i @param {number} value @param {number} size */
  set(i, value, size) {
    this.version++;
    this.sizes[i] = size;
    this.positions.set(i, value);
    this.magnitudes.set(i, magnitude(size));
    if (this.scaleTo === "all") this.most = this.#largest(0, this.sizes.length - 1);
  }

  /** @param {{ up?: string, upEdge?: string, down?: string, downEdge?: string, size?: number, hide?: number, scaleTo?: "view" | "all", halo?: string | null }} look */
  setStyle({ up = this.up, upEdge = this.upEdge, down = this.down, downEdge = this.downEdge, size = this.size, hide = this.hide, scaleTo = this.scaleTo, halo = this.halo }) {
    this.version++;
    (this.up = up), (this.upEdge = upEdge), (this.down = down), (this.downEdge = downEdge), (this.size = size), (this.hide = hide), (this.scaleTo = scaleTo), (this.halo = halo);
    if (scaleTo === "all") this.most = this.#largest(0, this.sizes.length - 1);
  }

  /** @param {import("./timeline.js").Timeline} time */
  viewed(time) {
    if (this.scaleTo !== "view") return;
    const most = this.#largest(Math.ceil(time.from), Math.floor(time.to));
    if (most !== this.most) (this.most = most), this.version++;
  }

  /** @param {number} a @param {number} b */
  #largest(a, b) {
    (EXTENT[0] = Infinity), (EXTENT[1] = -Infinity);
    this.magnitudes.extent(a, b, false, EXTENT);
    return EXTENT[1] > 0 ? EXTENT[1] : 0;
  }

  /** @param {number} a @param {number} b @param {boolean} positive @param {Float64Array} out */
  extent(a, b, positive, out) {
    this.positions.extent(a, b, positive, out);
  }

  /** @param {number} i */
  first(i) {
    return this.positions.first(i);
  }

  /** @param {number} _i @param {number} y @param {import("./scale.js").Scale} scale @param {number} x @param {import("./timeline.js").Timeline} time */
  distance(_i, y, scale, x, time) {
    return measure(this, time, scale, x, y);
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr
   */
  draw(g, time, scale, x0, x1, dpr) {
    const most = this.most, values = this.positions.values, sizes = this.sizes, R = this.size / 2, log = scale.mode === "log";
    if (!(most > 0)) return;
    const ring = Math.max(1, Math.round(1.5 * dpr)) / dpr, reach = R + ring + 1, least = this.hide * most;
    const a = Math.max(0, Math.floor(time.index(x0 - reach)) - 1), b = Math.min(values.length - 1, Math.ceil(time.index(x1 + reach)) + 1);
    if (radii.length < b - a + 1) (xs = new Float64Array((b - a + 1) * 2)), (ys = new Float64Array((b - a + 1) * 2)), (radii = new Float64Array((b - a + 1) * 2)), (ups = new Uint8Array((b - a + 1) * 2)), (order = new Int32Array((b - a + 1) * 2));
    let n = 0;
    for (let i = a; i <= b; i++) {
      const v = values[i], s = sizes[i], m = Math.abs(s);
      if (!(log ? v > 0 : v === v) || !(m > 0 && m < Infinity && m >= least)) continue;
      (xs[n] = time.x(i)), (ys[n] = scale.y(v)), (radii[n] = R * Math.sqrt(Math.min(1, m / most))), (ups[n] = s >= 0 ? 1 : 0), (order[n] = n), n++;
    }
    const sorted = order.subarray(0, n).sort(largest);
    g.lineWidth = Math.max(1, Math.floor(dpr)) / dpr;
    for (let k = 0; k < n; k++) {
      const j = sorted[k], up = ups[j] === 1;
      if (this.halo && !measuring) (g.fillStyle = this.halo), g.beginPath(), g.arc(xs[j], ys[j], radii[j] + ring, 0, Math.PI * 2), g.fill();
      // A see-through bubble with a visible edge is hit by its circle all the same.
      const fill = up ? this.up : this.down, edge = up ? this.upEdge : this.downEdge;
      g.fillStyle = measuring && !painted(fill) ? edge : fill;
      g.beginPath();
      g.arc(xs[j], ys[j], radii[j], 0, Math.PI * 2);
      g.fill();
      g.strokeStyle = edge;
      g.stroke();
    }
  }
}
