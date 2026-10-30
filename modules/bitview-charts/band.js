// @ts-check
// A band: a fill between two columns on the timeline (`lower` and `upper`, Float64Arrays; NaN in either: none there, the
// band broken), in one color. A cloud between two lines is one; a stack is one between each two edges (see stack.js).
// Without `lower`, an area: filled down to `base` (0 if not given; on a log scale, a base it can't show (0 or under)
// is its foot: the area down to the pane's foot); with `below` too, a baseline: under its base, in that color. Zoomed out, each pixel column takes each edge's last value in it: bands
// sharing an edge meet exactly, no gap or overlap between them, whatever the zoom.

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
  /** @param {{ lower?: Float64Array, upper: Float64Array, base?: number, color?: string, below?: string | null }} options */
  constructor({ lower, upper, base = 0, color = "rgba(247, 248, 248, 0.2)", below = null }) {
    /** @type {Float64Array} */
    this.lower = lower ?? new Float64Array(upper.length).fill(base);
    this.base = base;
    /** An area: its lower edge its base, made to its values' length. */
    this.area = !lower;
    /** Its color under its base (a baseline), or none: one color. */
    this.below = below;
    /** @type {Float64Array} */
    this.upper = upper;
    this.color = color;
    /** Not drawn, fitted, tagged or hit (its pane drawn whole after a change: its chart's invalidate(true)). */
    this.hidden = false;
    /** Changed at each change of its edges or color: its pane's picture drawn whole then. */
    this.version = 0;
    // (Each point's next without both edges (on a log scale, without both above zero), made at its first draw of a
    // version: a pixel column of points with no gap reads its last alone.)
    /** @type {Int32Array | null} */
    this.gaps = null;
    /** @type {Int32Array | null} */
    this.logGaps = null;
    this.gapsVersion = -1;
  }

  /** Its first value at or after index `i` (a relative scale's base: its first in view), or NaN: none. @param {number} i */
  first(i) {
    const values = this.upper;
    for (let k = Math.max(0, i); k < values.length; k++) if (values[k] === values[k]) return values[k];
    return NaN;
  }

  /**
   * How far a point (`y`, CSS pixels in its pane) is from it at index `i` (Infinity: it has nothing there), on `scale`
   * (its base set already, where it's relative).
   * @param {number} i @param {number} y @param {import("./scale.js").Scale} scale
   */
  distance(i, y, scale) {
    // (As it's drawn: both edges there, above zero on a log scale.)
    if (!(i >= 0 && i < this.lower.length && i < this.upper.length)) return Infinity;
    const log = scale.mode === "log", floored = this.#floored(log), hi = this.upper[i], lo = floored ? hi : this.lower[i];
    if (!(log ? lo > 0 && hi > 0 : lo === lo && hi === hi)) return Infinity;
    const top = scale.y(floored ? hi : Math.max(lo, hi)), bottom = floored ? scale.height : scale.y(Math.min(lo, hi));
    return y < top ? top - y : y > bottom ? y - bottom : 0;
  }

  /** Its next gap from each point (the length: none). @param {boolean} log */
  #gaps(log) {
    if (this.gapsVersion !== this.version) (this.gaps = this.logGaps = null), (this.gapsVersion = this.version);
    // (An area to the foot: its values alone count; its log gaps those, kept as the log ones, its version theirs.)
    const kept = log ? this.logGaps : this.gaps, floored = this.#floored(log);
    if (kept) return kept;
    const lower = this.lower, upper = this.upper, n = Math.min(lower.length, upper.length), next = new Int32Array(n + 1);
    next[n] = n;
    for (let i = n - 1; i >= 0; i--) {
      const lo = floored ? upper[i] : lower[i], hi = upper[i];
      next[i] = (log ? lo > 0 && hi > 0 : lo === lo && hi === hi) ? next[i + 1] : i;
    }
    return log ? (this.logGaps = next) : (this.gaps = next);
  }

  /**
   * Its edges, color or base changed: a lower edge given, a band; `lower: null`, an area again (filled to its base).
   * @param {{ lower?: Float64Array | null, upper?: Float64Array, color?: string, below?: string | null, base?: number }} change
   */
  set({ lower, upper = this.upper, color = this.color, below = this.below, base = this.base }) {
    this.version++;
    const wasArea = this.area;
    if (lower === null) this.area = true;
    else if (lower) this.area = false;
    // (An area's base made again to its values' length, or a new base, or once it was a band's lower edge.)
    const kept = wasArea && this.lower.length === upper.length && base === this.base;
    this.base = base;
    this.lower = lower ?? (this.area ? (kept ? this.lower : new Float64Array(upper.length).fill(base)) : this.lower);
    (this.upper = upper), (this.color = color), (this.below = below);
  }

  /** On a log scale, an area whose base it can't show: filled to the pane's foot, its lower edge none. @param {boolean} log */
  #floored(log) {
    return log && this.area && !(this.base > 0);
  }

  /**
   * Its lowest and highest from index `a` to `b`, written into `out` widened ([lo, hi]): its lower edge's lowest, its
   * upper's highest (their lines, drawn with it, are usually there to keep them fitted fast).
   * @param {number} a @param {number} b @param {boolean} positive @param {Float64Array} out
   */
  extent(a, b, positive, out) {
    a = Math.max(0, a);
    b = Math.min(this.lower.length - 1, this.upper.length - 1, b);
    const floored = this.#floored(positive);
    // (Only where it's drawn: both edges there, above zero on a log scale; either edge the lower (an area or a
    // baseline under its base); an area to the foot, its values alone.)
    for (let i = a; i <= b; i++) {
      const l = floored ? this.upper[i] : this.lower[i], u = this.upper[i];
      if (!(positive ? l > 0 && u > 0 : l === l && u === u)) continue;
      const lo = l < u ? l : u, hi = l < u ? u : l;
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
    if (!this.below) return this.#fill(g, time, scale, x0, x1, this.color);
    // (A baseline: over its base in its color, under it in the other; the same outline filled twice, clipped at it (at
    // the foot where its scale can't show it: a log scale's base of zero).)
    let at = scale.y(this.base);
    at = at < 0 ? 0 : at < scale.height ? at : scale.height;
    g.save();
    g.beginPath();
    g.rect(x0, 0, x1 - x0, at);
    g.clip();
    this.#fill(g, time, scale, x0, x1, this.color);
    g.restore();
    g.save();
    g.beginPath();
    g.rect(x0, at, x1 - x0, scale.height - at);
    g.clip();
    this.#fill(g, time, scale, x0, x1, this.below);
    g.restore();
  }

  /**
   * Its outline from x `x0` to `x1` filled in `color`.
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {string} color
   */
  #fill(g, time, scale, x0, x1, color) {
    const lower = this.lower, upper = this.upper, n = Math.min(lower.length, upper.length), log = scale.mode === "log";
    if (!n) return;
    const per = 1 / time.spacing, zoomedIn = per <= 1, viewFrom = time.from, span = time.to - time.from, width = time.width, gaps = zoomedIn ? null : this.#gaps(log);
    // (Zoomed out, its y as bottom - k * (t - ta) (t the value, or its log, from the scale's low end), inline: without
    // a JIT a call costs more than its work. Its columns' points as Timeline.index has them, to the bit.)
    const ta = log ? Math.log10(scale.lo) : scale.lo, tb = log ? Math.log10(scale.hi) : scale.hi, top = scale.top * scale.height, room = scale.height - top - scale.bottom * scale.height;
    const kk = room / (tb - ta), bottom = top + room, m = scale.m, c = scale.c, floored = this.#floored(log), foot = scale.height;
    const a = Math.max(0, Math.floor(time.index(x0)) - 1), b = Math.min(n - 1, Math.ceil(time.index(x1)) + 1);
    // (A point each, zoomed in; a pixel column each, its last, zoomed out.)
    const from = zoomedIn ? a : Math.floor(x0) - 1, to = zoomedIn ? b : Math.ceil(x1) + 1;
    outline.start(g, to - from + 2);
    g.fillStyle = color;
    g.beginPath();
    for (let k = from; k <= to; k++) {
      if (zoomedIn) {
        const lo = floored ? upper[k] : lower[k], hi = upper[k];
        if (log ? lo > 0 && hi > 0 : lo === lo && hi === hi) outline.add(time.x(k), scale.y(hi), floored ? foot : scale.y(lo));
        else outline.close();
        continue;
      }
      // (A pixel column: its last point with both edges, the band broken at any point without, in it too; no gap in
      // it (most), its last read alone.)
      const first = Math.max(0, Math.ceil(viewFrom + (k * span) / width)), last = Math.min(n - 1, Math.ceil(viewFrom + ((k + 1) * span) / width) - 1);
      if (last < first) continue;
      if (/** @type {Int32Array} */ (gaps)[first] > last) {
        const hi = upper[last], lo = lower[last];
        outline.add(k, bottom - kk * ((log ? Math.log10(hi) : hi * m + c) - ta), floored ? foot : bottom - kk * ((log ? Math.log10(lo) : lo * m + c) - ta));
        continue;
      }
      let at = -1;
      for (let i = first; i <= last; i++) {
        const lo = floored ? upper[i] : lower[i], hi = upper[i];
        if (log ? lo > 0 && hi > 0 : lo === lo && hi === hi) at = i;
        else {
          if (at >= 0) outline.add(k, scale.y(upper[at]), floored ? foot : scale.y(lower[at]));
          outline.close();
          at = -1;
        }
      }
      if (at >= 0) outline.add(k, scale.y(upper[at]), floored ? foot : scale.y(lower[at]));
    }
    outline.close();
    g.fill();
  }

}
