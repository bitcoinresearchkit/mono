// @ts-check
// A line: its values as one column on the chart's timeline (a Float64Array, an index a time; NaN: none there, the line
// broken), its color and width. Its lowest and highest kept a block of 64 points (a view's extremes read block by block),
// the lowest above zero apart (a log scale's). Drawn point to point where a point has a pixel or more; zoomed out, each
// pixel column as its first, low, high and last values (as many points as the column holds, drawn as the eye sees them).
// `breaks` (a Uint8Array, 1 at a point): the line stops after that point, nothing drawn to the next (a trend's flip, a
// stretch begun anew). `colors` (a Uint8Array of indices into `palette`): each segment in its first point's color.

import { Summary, FIRST } from "./summary.js";

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
   * `tag`: its last value in a badge on its pane's axis, in its color; `priceLine`: a dotted line across at it.
   * @param {{ values?: Float64Array, color?: string, width?: number, breaks?: Uint8Array | null, colors?: Uint8Array | null, palette?: string[], tag?: boolean, priceLine?: boolean }} [options]
   */
  constructor({ values = new Float64Array(0), color = "#f7f8f8", width = 1.5, breaks = null, colors = null, palette = [], tag = false, priceLine = false } = {}) {
    this.color = color;
    this.width = width;
    this.breaks = breaks;
    this.colors = colors;
    this.palette = palette;
    this.tag = tag;
    this.priceLine = priceLine;
    // (Its last point with a value, found once a version: NaN, none.)
    this.lastAt = NaN;
    this.lastVersion = -1;
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
    /** Its values summed up by blocks, for drawing zoomed out (see summary.js). @type {Summary | null} */
    this.summary = null;
    this.setValues(values);
  }

  /** @param {Float64Array} values */
  setValues(values) {
    this.version++;
    this.values = values;
    this.summary = values.length >= 1 << FIRST ? new Summary(values) : null;
    const blocks = Math.ceil(values.length / BLOCK);
    if (this.blockLo.length !== blocks) (this.blockLo = new Float64Array(blocks)), (this.blockHi = new Float64Array(blocks)), (this.blockPos = new Float64Array(blocks));
    for (let b = 0; b < blocks; b++) this.#block(b);
  }

  /** One value set in place (a live point), its block's extremes made again. @param {number} i @param {number} value */
  set(i, value) {
    this.version++;
    this.values[i] = value;
    this.summary?.set(i);
    this.#block(Math.floor(i / BLOCK));
  }

  /** @param {{ color?: string, width?: number, breaks?: Uint8Array | null, colors?: Uint8Array | null, palette?: string[], tag?: boolean, priceLine?: boolean }} look */
  setStyle({ color = this.color, width = this.width, breaks = this.breaks, colors = this.colors, palette = this.palette, tag = this.tag, priceLine = this.priceLine }) {
    this.version++;
    (this.color = color), (this.width = width), (this.breaks = breaks), (this.colors = colors), (this.palette = palette), (this.tag = tag), (this.priceLine = priceLine);
  }

  /** Its last point with a value (on a log scale, above zero), or -1: none. @param {boolean} log */
  last(log) {
    if (this.lastVersion !== this.version) {
      this.lastVersion = this.version;
      this.lastAt = NaN;
      const values = this.values;
      for (let i = values.length - 1; i >= 0; i--) if (values[i] === values[i]) {
        this.lastAt = i;
        break;
      }
    }
    // (On a log scale, one above zero: the last with a value usually is.)
    let i = this.lastAt === this.lastAt ? this.lastAt : -1;
    if (log) while (i >= 0 && !(this.values[i] > 0)) i--;
    return i;
  }

  /** Its first value at or after index `i` (a relative scale's base: its first in view), or NaN: none. @param {number} i */
  first(i) {
    const values = this.values;
    for (let k = Math.max(0, i); k < values.length; k++) if (values[k] === values[k]) return values[k];
    return NaN;
  }

  /**
   * How far a point (`y`, CSS pixels in its pane) is from it at index `i` (Infinity: it has nothing there), on `scale`
   * (its base set already, where it's relative).
   * @param {number} i @param {number} y @param {import("./scale.js").Scale} scale
   */
  distance(i, y, scale) {
    // (Where it's drawn alone: in its data, a value there, above zero on a log scale.)
    if (!(i >= 0 && i < this.values.length)) return Infinity;
    const v = this.values[i];
    return v === v && (scale.mode !== "log" || v > 0) ? Math.abs(scale.y(v) - y) : Infinity;
  }

  /** Its color at a point: its own, or its palette's there. @param {number} i */
  colorAt(i) {
    return this.colors && this.palette.length ? this.palette[this.colors[i]] ?? this.color : this.color;
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
   * A plain line (no colors, no breaks) zoomed out: each pixel column's run as #trace's, read from its summary's clean
   * blocks whole and its other points one by one, in one loop of its own (without a JIT, each call costs more than the
   * work in it: a run, a scale's y, a block's search, each inline here).
   * @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale @param {number} x0 @param {number} x1
   */
  #plain(time, scale, x0, x1) {
    const values = this.values, n = values.length, levels = this.summary ? this.summary.levels : null, log = scale.mode === "log";
    // (Each column's points as Timeline.index has them, its sums in the same order: the same columns to the bit.)
    const from = time.from, span = time.to - time.from, width = time.width;
    // (Its y: bottom - k * (t - ta), t the value as its scale reads it (v * m + c: as it is, or relative), or its log,
    // from the scale's low end: no large terms cancelling.)
    const ta = log ? Math.log10(scale.lo) : scale.lo, tb = log ? Math.log10(scale.hi) : scale.hi, top = scale.top * scale.height, room = scale.height - top - scale.bottom * scale.height;
    const k = room / (tb - ta), bottom = top + room, m = scale.m, c = scale.c;
    // (Each column from where the last one ended: its start computed once.)
    let start = Math.ceil(from + ((Math.floor(x0) - 1) * span) / width);
    for (let x = Math.floor(x0) - 1; x <= Math.ceil(x1) + 1; x++) {
      const end = Math.ceil(from + ((x + 1) * span) / width), a = start < 0 ? 0 : start, b = end - 1 < n - 1 ? end - 1 : n - 1;
      start = end;
      let first = NaN, last = NaN, lo = Infinity, hi = -Infinity, loAt = 0, hiAt = 0, i = a;
      while (i <= b) {
        // (The largest clean block starting at i and ending by b: as large as i's alignment and what's left allow.)
        if (levels && !(i & ((1 << FIRST) - 1)) && b - i + 1 >= 1 << FIRST) {
          let size = Math.min(i ? i & -i : 1 << 30, 1 << (31 - Math.clz32(b - i + 1)));
          for (; size >= 1 << FIRST; size >>= 1) {
            const level = 31 - Math.clz32(size) - FIRST;
            if (level >= levels.length) continue;
            const at = levels[level], k = i >> (level + FIRST);
            if (k >= at.first.length || (log ? at.low[k] : at.nan[k])) continue;
            if (first !== first) first = at.first[k];
            last = at.last[k];
            if (at.lo[k] < lo) (lo = at.lo[k]), (loAt = at.loAt[k]);
            if (at.hi[k] > hi) (hi = at.hi[k]), (hiAt = at.hiAt[k]);
            break;
          }
          if (size >= 1 << FIRST) {
            i += size;
            continue;
          }
        }
        const v = values[i];
        if (log ? v > 0 : v === v) {
          if (first !== first) first = v;
          last = v;
          if (v < lo) (lo = v), (loAt = i);
          if (v > hi) (hi = v), (hiAt = i);
        } else {
          // (No value: the run so far written, the line broken.)
          if (first === first) this.#run(x, first, last, lo, hi, loAt, hiAt, bottom, k, ta, m, c, log), (first = NaN), (lo = Infinity), (hi = -Infinity);
          pen.lift();
        }
        i++;
      }
      if (first === first) this.#run(x, first, last, lo, hi, loAt, hiAt, bottom, k, ta, m, c, log);
    }
  }

  /**
   * A run written at `x`: its first, its low and high past its first and last in the order they came, its last.
   * @param {number} x @param {number} first @param {number} last @param {number} lo @param {number} hi @param {number} loAt
   * @param {number} hiAt @param {number} bottom @param {number} k @param {number} ta @param {number} m @param {number} c
   * @param {boolean} log
   */
  #run(x, first, last, lo, hi, loAt, hiAt, bottom, k, ta, m, c, log) {
    pen.to(x, bottom - k * ((log ? Math.log10(first) : first * m + c) - ta), 0, false);
    const low = lo < first && lo < last, high = hi > first && hi > last;
    // (Flat: its one point written.)
    if (last === first && !low && !high) return;
    if (low && high && hiAt < loAt) pen.to(x, bottom - k * ((log ? Math.log10(hi) : hi * m + c) - ta), 0, false), pen.to(x, bottom - k * ((log ? Math.log10(lo) : lo * m + c) - ta), 0, false);
    else {
      if (low) pen.to(x, bottom - k * ((log ? Math.log10(lo) : lo * m + c) - ta), 0, false);
      if (high) pen.to(x, bottom - k * ((log ? Math.log10(hi) : hi * m + c) - ta), 0, false);
    }
    pen.to(x, bottom - k * ((log ? Math.log10(last) : last * m + c) - ta), 0, false);
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
    if (!colors && !breaks) return this.#plain(time, scale, x0, x1);
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
