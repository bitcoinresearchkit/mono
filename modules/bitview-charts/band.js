// @ts-check
// A band: a fill between two columns on the timeline (`lower` and `upper`, Float64Arrays; NaN in either: none there, the
// band broken), in one color. A cloud between two lines is one; a stack is one between each two edges (see stack.js).
// Zoomed out, each pixel column takes each edge's last value in it: bands sharing an edge meet exactly, no gap or
// overlap between them, whatever the zoom.

/**
 * A band's outline as it's written: each point's x and its edges' ys kept for the way back along the lower edge (grown,
 * never made again); a stretch closed at a gap. One for every band, drawn one after another.
 */
const outline = {
  g: /** @type {CanvasRenderingContext2D | null} */ (null),
  xs: new Float64Array(0),
  ups: new Float64Array(0),
  downs: new Float64Array(0),
  count: 0,
  /** @param {CanvasRenderingContext2D} g @param {number} room */
  start(g, room) {
    if (this.xs.length < room) (this.xs = new Float64Array(room * 2)), (this.ups = new Float64Array(room * 2)), (this.downs = new Float64Array(room * 2));
    this.g = g;
    this.count = 0;
  },
  /** @param {number} x @param {number} up @param {number} down */
  add(x, up, down) {
    (this.xs[this.count] = x), (this.ups[this.count] = up), (this.downs[this.count] = down);
    this.count++;
  },
  /** The stretch so far: up its upper edge, back along its lower. */
  close() {
    const g = /** @type {CanvasRenderingContext2D} */ (this.g), { xs, ups, downs, count } = this;
    if (count > 1) {
      g.moveTo(xs[0], ups[0]);
      for (let k = 1; k < count; k++) g.lineTo(xs[k], ups[k]);
      for (let k = count - 1; k >= 0; k--) g.lineTo(xs[k], downs[k]);
      g.closePath();
    }
    this.count = 0;
  },
};

export class Band {
  /** @param {{ lower: Float64Array, upper: Float64Array, color?: string }} options */
  constructor({ lower, upper, color = "rgba(247, 248, 248, 0.2)" }) {
    /** @type {Float64Array} */
    this.lower = lower;
    /** @type {Float64Array} */
    this.upper = upper;
    this.color = color;
    /** Changed at each change of its edges or color: its pane's picture drawn whole then. */
    this.version = 0;
  }

  /** @param {{ lower?: Float64Array, upper?: Float64Array, color?: string }} change */
  set({ lower = this.lower, upper = this.upper, color = this.color }) {
    this.version++;
    (this.lower = lower), (this.upper = upper), (this.color = color);
  }

  /**
   * Its lowest and highest from index `a` to `b`, written into `out` widened ([lo, hi]): its lower edge's lowest, its
   * upper's highest (their lines, drawn with it, are usually there to keep them fitted fast).
   * @param {number} a @param {number} b @param {boolean} positive @param {Float64Array} out
   */
  extent(a, b, positive, out) {
    a = Math.max(0, a);
    b = Math.min(this.lower.length - 1, this.upper.length - 1, b);
    // (Only where it's drawn: both edges there, above zero on a log scale.)
    for (let i = a; i <= b; i++) {
      const lo = this.lower[i], hi = this.upper[i];
      if (!(positive ? lo > 0 && hi > 0 : lo === lo && hi === hi)) continue;
      if (lo < out[0]) out[0] = lo;
      if (hi > out[1]) out[1] = hi;
    }
  }

  /**
   * Filled on `g` (CSS pixels) from x `x0` to `x1` of the plot.
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1
   */
  draw(g, time, scale, x0, x1) {
    const lower = this.lower, upper = this.upper, n = Math.min(lower.length, upper.length), log = scale.mode === "log";
    if (!n) return;
    const per = 1 / time.spacing, zoomedIn = per <= 1;
    const a = Math.max(0, Math.floor(time.index(x0)) - 1), b = Math.min(n - 1, Math.ceil(time.index(x1)) + 1);
    // (A point each, zoomed in; a pixel column each, its last, zoomed out.)
    const from = zoomedIn ? a : Math.floor(x0) - 1, to = zoomedIn ? b : Math.ceil(x1) + 1;
    outline.start(g, to - from + 2);
    g.fillStyle = this.color;
    g.beginPath();
    for (let k = from; k <= to; k++) {
      if (zoomedIn) {
        const lo = lower[k], hi = upper[k];
        if (log ? lo > 0 && hi > 0 : lo === lo && hi === hi) outline.add(time.x(k), scale.y(hi), scale.y(lo));
        else outline.close();
        continue;
      }
      // (A pixel column: its last point with both edges, the band broken at any point without, in it too.)
      const first = Math.max(0, Math.ceil(time.index(k))), last = Math.min(n - 1, Math.ceil(time.index(k + 1)) - 1);
      let at = -1;
      for (let i = first; i <= last; i++) {
        const lo = lower[i], hi = upper[i];
        if (log ? lo > 0 && hi > 0 : lo === lo && hi === hi) at = i;
        else {
          if (at >= 0) outline.add(k, scale.y(upper[at]), scale.y(lower[at]));
          outline.close();
          at = -1;
        }
      }
      if (at >= 0) outline.add(k, scale.y(upper[at]), scale.y(lower[at]));
    }
    outline.close();
    g.fill();
  }
}
