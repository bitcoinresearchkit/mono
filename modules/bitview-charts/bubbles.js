// @ts-check
// Bubbles: a dot a point on the timeline, where its `values` put it on its scale (NaN: none there), its area its size's
// share of the largest (`sizes`, a column: `size` CSS pixels across at the largest, of every size, placed or not; those
// under `hide` of the largest left out), above zero in `up` and edged in `upEdge`, below in `down` and `downEdge`; the largest drawn first, the
// smallest on top (none hidden behind a big one). The largest: of those in view (`scaleTo` "view", kept up as the view
// moves: its pane drawn whole where it changes), or of all of them ("all"). With `halo`, each ringed in it first (the
// ground's color: overlapping bubbles stand apart).

import { Line } from "./line.js";
import { measure, measuring, painted } from "./recorder.js";

/** The bubbles a draw puts down: their x, y, radius (CSS pixels), whether above zero, and their order (made once, grown). */
let xs = new Float64Array(64), ys = new Float64Array(64), radii = new Float64Array(64), ups = new Uint8Array(64), order = new Int32Array(64);
/** A range's lowest and highest, as a line's extent reads them (made once). */
const EXTENT = new Float64Array(2);
/** A size's magnitude, none (NaN) where it's 0 or not finite. @param {number} size */
const magnitude = (size) => ((m) => (m > 0 && m < Infinity ? m : NaN))(Math.abs(size));
/** The largest first, ties by index (a strip's and a whole draw's alike). @param {number} a @param {number} b */
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
    // (Its positions as a line's (its scale fitted to them block by block), and its sizes' magnitudes (the largest read
    // block by block).)
    this.positions = new Line();
    this.magnitudes = new Line();
    /** @type {Float64Array} */
    this.sizes = sizes;
    /** The largest size it's drawn against (0: none drawn). */
    this.most = 0;
    /** Not drawn, fitted or hit. */
    this.hidden = false;
    this.version = 0;
    this.setValues({ values, sizes });
  }

  /** Its positions. */
  get values() {
    return this.positions.values;
  }

  /**
   * Its positions (its scale fitted to them, a relative one based on them, sized or not: they follow what they're on)
   * and sizes (the largest of them all, placed or not: a bubble's size the same wherever the others are).
   * @param {{ values: Float64Array, sizes: Float64Array }} columns
   */
  setValues({ values, sizes }) {
    this.version++;
    this.sizes = sizes;
    // (In a loop: without a JIT, from() with a mapping costs several times as much.)
    const magnitudes = new Float64Array(sizes.length);
    for (let i = 0; i < magnitudes.length; i++) magnitudes[i] = magnitude(sizes[i]);
    this.positions.setValues(values);
    this.magnitudes.setValues(magnitudes);
    if (this.scaleTo === "all") this.most = this.#largest(0, sizes.length - 1);
  }

  /** One point's position and size set in place (a live point), its blocks' extremes made again. @param {number} i @param {number} value @param {number} size */
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

  /** Brought up to the view before its pane draws: sized to the largest in it, its version changed where that changes. @param {import("./timeline.js").Timeline} time */
  viewed(time) {
    if (this.scaleTo !== "view") return;
    const most = this.#largest(Math.ceil(time.from), Math.floor(time.to));
    if (most !== this.most) (this.most = most), this.version++;
  }

  /** The largest size from index `a` to `b` (0: none). @param {number} a @param {number} b */
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

  /**
   * How far a point is from it, measured against its drawing there (see recorder.js).
   * @param {number} i @param {number} y @param {import("./scale.js").Scale} scale @param {number} x @param {import("./timeline.js").Timeline} time
   */
  distance(i, y, scale, x, time) {
    return measure(this, time, scale, x, y);
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr
   */
  draw(g, time, scale, x0, x1, dpr) {
    const most = this.most, values = this.positions.values, sizes = this.sizes, R = this.size / 2, log = scale.mode === "log";
    if (!(most > 0)) return;
    // (Its bubbles reaching into the strip: the largest's radius, its halo and its edge past each end.)
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
      // (Its halo the ground's: not what's hit.)
      if (this.halo && !measuring) (g.fillStyle = this.halo), g.beginPath(), g.arc(xs[j], ys[j], radii[j] + ring, 0, Math.PI * 2), g.fill();
      // (Measured, a see-through bubble with an edge that shows hit by its circle all the same.)
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
