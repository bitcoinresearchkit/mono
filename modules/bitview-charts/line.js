// @ts-check
// A line: its values as one column on the chart's timeline (a Float64Array, an index a time; NaN: none there, the line
// broken), its color and width. Its lowest and highest kept a block of 64 points (a view's extremes read block by block),
// the lowest above zero apart (a log scale's). Drawn point to point where a point has a pixel or more; zoomed out, each
// pixel column as its first, low, high and last values (as many points as the column holds, drawn as the eye sees them).
// `breaks` (a Uint8Array, 1 at a point): the line stops after that point, nothing drawn to the next (a trend's flip, a
// stretch begun anew). `colors` (a Uint8Array of indices into `palette`): each segment in its first point's color.

const BLOCK = 64;

/**
 * A line's path as it's written: each y on a device pixel, and a run at one y one segment (a slow line, zoomed out, is
 * mostly runs: each a segment a pixel column, else). One for every line, drawn one after another.
 */
/**
 * A line's path as it's written, for one of its colors (`pass`; -1: it has one): each point with its color and whether
 * the line stops after it, a segment drawn from a point in this pass's color to the next; each y on a device pixel, and
 * a run at one y one segment (a slow line, zoomed out, is mostly runs: else a segment a pixel column). One for every
 * line, drawn one after another.
 */
const pen = {
  g: /** @type {CanvasRenderingContext2D | null} */ (null),
  dpr: 1,
  pass: -1,
  // (The path drawn: on it, its last y, a run's end not written yet (NaN: none).)
  on: false,
  y: NaN,
  runX: NaN,
  // (The last point: whether there's one, where, its color, whether the line stops after it.)
  has: false,
  px: 0,
  py: 0,
  color: 0,
  cut: false,
  /** @param {CanvasRenderingContext2D} g @param {number} dpr @param {number} pass */
  start(g, dpr, pass) {
    (this.g = g), (this.dpr = dpr), (this.pass = pass);
    (this.on = false), (this.has = false), (this.runX = NaN);
  },
  /** @param {number} x @param {number} y @param {number} color @param {boolean} cut */
  to(x, y, color, cut) {
    const g = /** @type {CanvasRenderingContext2D} */ (this.g);
    y = Math.round(y * this.dpr) / this.dpr;
    if (this.has && !this.cut && (this.pass < 0 || this.color === this.pass)) {
      if (!this.on) {
        g.moveTo(this.px, this.py);
        (this.on = true), (this.y = this.py), (this.runX = NaN);
      }
      if (y === this.y) this.runX = x;
      else {
        if (this.runX === this.runX) g.lineTo(this.runX, this.y);
        g.lineTo(x, y);
        (this.y = y), (this.runX = NaN);
      }
    } else this.close();
    (this.has = true), (this.px = x), (this.py = y), (this.color = color), (this.cut = cut);
  },
  /** The line lifted (a gap, its end): nothing drawn to the next point. */
  lift() {
    this.close();
    this.has = false;
  },
  /** The path so far written, its run's end too. */
  close() {
    if (this.on && this.runX === this.runX) /** @type {CanvasRenderingContext2D} */ (this.g).lineTo(this.runX, this.y);
    (this.on = false), (this.runX = NaN);
  },
};

/**
 * A pixel column's run of points (between gaps and breaks): its first, last, low and high, written to the pen as they
 * came, its low and high only past its first and last (two points a column: two segments); in its first point's
 * color, its last's on to the next.
 */
const run = {
  first: NaN,
  last: NaN,
  lo: Infinity,
  hi: -Infinity,
  firstAt: 0,
  lastAt: 0,
  loAt: 0,
  hiAt: 0,
  start() {
    (this.first = NaN), (this.lo = Infinity), (this.hi = -Infinity);
  },
  /** @param {number} v @param {number} i */
  add(v, i) {
    if (this.first !== this.first) (this.first = v), (this.firstAt = i);
    (this.last = v), (this.lastAt = i);
    if (v < this.lo) (this.lo = v), (this.loAt = i);
    if (v > this.hi) (this.hi = v), (this.hiAt = i);
  },
  /**
   * Written at `x`, if it has points; then begun again.
   * @param {number} x @param {import("./scale.js").Scale} scale @param {Uint8Array | null} colors @param {Uint8Array | null} breaks
   */
  end(x, scale, colors, breaks) {
    const { first, last, lo, hi } = this;
    if (first !== first) return;
    const color = colors ? colors[this.firstAt] : 0;
    pen.to(x, scale.y(first), color, false);
    const low = lo < first && lo < last, high = hi > first && hi > last;
    if (low && high && this.hiAt < this.loAt) pen.to(x, scale.y(hi), color, false), pen.to(x, scale.y(lo), color, false);
    else {
      if (low) pen.to(x, scale.y(lo), color, false);
      if (high) pen.to(x, scale.y(hi), color, false);
    }
    pen.to(x, scale.y(last), colors ? colors[this.lastAt] : 0, breaks ? breaks[this.lastAt] === 1 : false);
    this.start();
  },
};

export class Line {
  /**
   * @param {{ values?: Float64Array, color?: string, width?: number, breaks?: Uint8Array | null, colors?: Uint8Array | null, palette?: string[] }} [options]
   */
  constructor({ values = new Float64Array(0), color = "#f7f8f8", width = 1.5, breaks = null, colors = null, palette = [] } = {}) {
    this.color = color;
    this.width = width;
    this.breaks = breaks;
    this.colors = colors;
    this.palette = palette;
    /** Changed at each change of its values or look: its pane's picture drawn whole then. */
    this.version = 0;
    /** @type {Float64Array} */
    this.values = values;
    // (Each block's lowest, highest and lowest above zero.)
    /** @type {Float64Array} */
    this.blockLo = new Float64Array(0);
    /** @type {Float64Array} */
    this.blockHi = new Float64Array(0);
    /** @type {Float64Array} */
    this.blockPos = new Float64Array(0);
    this.setValues(values);
  }

  /** @param {Float64Array} values */
  setValues(values) {
    this.version++;
    this.values = values;
    const blocks = Math.ceil(values.length / BLOCK);
    if (this.blockLo.length !== blocks) (this.blockLo = new Float64Array(blocks)), (this.blockHi = new Float64Array(blocks)), (this.blockPos = new Float64Array(blocks));
    for (let b = 0; b < blocks; b++) this.#block(b);
  }

  /** One value set in place (a live point), its block's extremes made again. @param {number} i @param {number} value */
  set(i, value) {
    this.version++;
    this.values[i] = value;
    this.#block(Math.floor(i / BLOCK));
  }

  /** @param {{ color?: string, width?: number, breaks?: Uint8Array | null, colors?: Uint8Array | null, palette?: string[] }} look */
  setStyle({ color = this.color, width = this.width, breaks = this.breaks, colors = this.colors, palette = this.palette }) {
    this.version++;
    (this.color = color), (this.width = width), (this.breaks = breaks), (this.colors = colors), (this.palette = palette);
  }

  /** @param {number} b */
  #block(b) {
    const values = this.values, end = Math.min(values.length, (b + 1) * BLOCK);
    let lo = Infinity, hi = -Infinity, pos = Infinity;
    for (let i = b * BLOCK; i < end; i++) {
      const v = values[i];
      if (v < lo) lo = v;
      if (v > hi) hi = v;
      if (v > 0 && v < pos) pos = v;
    }
    this.blockLo[b] = lo;
    this.blockHi[b] = hi;
    this.blockPos[b] = pos;
  }

  /**
   * Its lowest and highest from index `a` to `b` (whole blocks read as kept, the ends point by point), written into
   * `out` widened ([lo, hi]); `positive`: the lowest above zero.
   * @param {number} a @param {number} b @param {boolean} positive @param {Float64Array} out
   */
  extent(a, b, positive, out) {
    const values = this.values, lows = positive ? this.blockPos : this.blockLo;
    a = Math.max(0, a);
    b = Math.min(values.length - 1, b);
    let lo = out[0], hi = out[1], i = a;
    while (i <= b) {
      if (i % BLOCK === 0 && i + BLOCK - 1 <= b) {
        const k = i / BLOCK;
        if (lows[k] < lo) lo = lows[k];
        if (this.blockHi[k] > hi) hi = this.blockHi[k];
        i += BLOCK;
        continue;
      }
      const v = values[i++];
      if (positive ? v > 0 && v < lo : v < lo) lo = v;
      if (v > hi) hi = v;
    }
    out[0] = lo;
    out[1] = hi;
  }

  /**
   * Drawn on `g` (CSS pixels) from x `x0` to `x1` of the plot: the timeline's view, the scale's values, at `dpr`
   * device pixels a CSS pixel; a stroke a color.
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr
   */
  draw(g, time, scale, x0, x1, dpr) {
    if (!this.values.length) return;
    g.lineWidth = this.width;
    // (Bevelled: as round at a line's width, half the cost to stroke in Firefox, and no miter's spikes.)
    g.lineJoin = "bevel";
    // (One stroke a color: its segments in it.)
    const colors = this.colors && this.palette.length ? this.colors : null;
    for (let pass = colors ? 0 : -1; pass < (colors ? this.palette.length : 0); pass++) {
      g.strokeStyle = pass < 0 ? this.color : this.palette[pass];
      g.beginPath();
      pen.start(g, dpr, pass);
      this.#trace(time, scale, x0, x1, colors);
      pen.lift();
      g.stroke();
    }
  }

  /**
   * Its points from `x0` to `x1`, written to the pen.
   * @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale @param {number} x0 @param {number} x1
   * @param {Uint8Array | null} colors
   */
  #trace(time, scale, x0, x1, colors) {
    const values = this.values, breaks = this.breaks, n = values.length, log = scale.mode === "log";
    if (time.spacing >= 1) {
      // (A pixel or more a point: each point, and one past each end, so the line reaches the edges.)
      const a = Math.max(0, Math.floor(time.index(x0)) - 1), b = Math.min(n - 1, Math.ceil(time.index(x1)) + 1);
      for (let i = a; i <= b; i++) {
        const v = values[i];
        if (log ? v > 0 : v === v) pen.to(time.x(i), scale.y(v), colors ? colors[i] : 0, breaks ? breaks[i] === 1 : false);
        else pen.lift();
      }
      return;
    }
    // (Each pixel column's points as runs: its first, low, high and last; a point with no value ends a run and breaks
    // the line there, in its column too; a break ends a run after its point. Columns at whole pixels of the view, their
    // points at whole indices: a view moved by whole pixels holds the same columns.)
    for (let x = Math.floor(x0) - 1; x <= Math.ceil(x1) + 1; x++) {
      const a = Math.max(0, Math.ceil(time.index(x))), b = Math.min(n - 1, Math.ceil(time.index(x + 1)) - 1);
      run.start();
      for (let i = a; i <= b; i++) {
        const v = values[i];
        if (log ? v > 0 : v === v) {
          // (A color changing in the column: a run each, so each segment keeps its own.)
          if (colors && run.first === run.first && colors[i] !== colors[run.firstAt]) run.end(x, scale, colors, breaks);
          run.add(v, i);
          if (breaks && breaks[i] === 1) run.end(x, scale, colors, breaks);
        } else run.end(x, scale, colors, breaks), pen.lift();
      }
      run.end(x, scale, colors, breaks);
    }
  }


}
