// @ts-check
// A line: its values as one column on the chart's timeline (a Float64Array, an index a time; NaN: none there, the line
// broken), its color and width. Its lowest and highest kept a block of 64 points (a view's extremes read block by
// block), the lowest above zero apart (a log scale's). Drawn point to point where a point has a pixel or more; zoomed
// out, each pixel column as its first, low, high and last values (as many points as the column holds, drawn as the eye
// sees them). `breaks` (a Uint8Array, 1 at a point): the line stops after that point, nothing drawn to the next (a
// trend's flip, a stretch begun anew). `colors` (a Uint8Array or Uint16Array of indices into `palette`): each segment
// in its first point's color.

import { Summary, FIRST } from "./summary.js";
import { kept } from "./format.js";
import { Fill } from "./color.js";
import { NOISE } from "./timeline.js";
import { measure, measuring } from "./recorder.js";

const BLOCK = 64;
/** A curved segment's control points, as #bend writes them (made once). */
const BEND = new Float64Array(4);
/** No dashes (made once). */
const SOLID = /** @type {number[]} */ ([]);

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
  /**
   * As to(), its segment from the last point a curve through two control points (CSS pixels).
   * @param {number} c1x @param {number} c1y @param {number} c2x @param {number} c2y @param {number} x @param {number} y @param {number} color @param {boolean} cut
   */
  curve(c1x, c1y, c2x, c2y, x, y, color, cut) {
    const g = /** @type {CanvasRenderingContext2D} */ (this.g);
    y = Math.round(y * this.dpr) / this.dpr;
    if (this.has && !this.cut && (this.pass < 0 || this.color === this.pass)) {
      if (!this.on) {
        g.moveTo(this.px, this.py);
        (this.on = true), (this.y = this.py), (this.runX = NaN);
      }
      if (this.runX === this.runX) g.lineTo(this.runX, this.y);
      g.bezierCurveTo(c1x, c1y, c2x, c2y, x, y);
      (this.y = y), (this.runX = NaN);
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
   * @param {number} x @param {import("./scale.js").Scale} scale @param {Uint8Array | Uint16Array | null} colors @param {Uint8Array | null} breaks
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
   * `tag`: its last value in a badge on its pane's axis, in its color (written by `format`, else as its side writes
   * values); `priceLine`: a dotted line across at it; `stroke: false`, no line drawn (its tag and price line still);
   * `outline`, a color it's edged in (under it, its width each side: over a fill or a line, it stands off it); `dots`, a
   * dot that radius at each point (in its color there; ringed in its outline); `marker`, the crosshair's dot on it that
   * radius (ringed in the ground, 2 pixels; 0: none); `steps`, each value held to the next point (zoomed in: zoomed out,
   * a pixel column holds several points either way); `dash`, its stroke's dashes (CSS pixels, as setLineDash takes
   * them; none, solid): dashes run along its path from where it starts, so its pane's drawn whole as the view moves
   * (its picture kept as it is otherwise); `curve`, bowed through its points (zoomed in), as the chart library's
   * curves (each segment's bend from its points' neighbors); `under`, its stroke in another color below a value (a
   * baseline's line: its color changing where it crosses its base).
   * @param {{ values?: Float64Array, color?: string, width?: number, breaks?: Uint8Array | null, colors?: Uint8Array | Uint16Array | null, palette?: string[], tag?: boolean, priceLine?: boolean, stroke?: boolean, format?: ((v: number) => string) | null, outline?: string | null, dots?: number, marker?: number, steps?: boolean, dash?: number[] | null, curve?: boolean, under?: { value: number, color: string } | null }} [options]
   */
  constructor({ values = new Float64Array(0), color = "#f7f8f8", width = 1.5, breaks = null, colors = null, palette = [], tag = false, priceLine = false, stroke = true, format = null, outline = null, dots = 0, marker = 4, steps = false, dash = null, curve = false, under = null } = {}) {
    this.color = color;
    this.width = width;
    this.breaks = breaks;
    this.colors = colors;
    this.palette = palette;
    this.tag = tag;
    this.priceLine = priceLine;
    this.stroke = stroke;
    /** How its tag's written (each value once: see kept); none, as its side writes values. @type {((v: number) => string) | null} */
    this.format = format && kept(format);
    /** @type {string | null} */
    this.outline = outline;
    this.dots = dots;
    this.marker = marker;
    this.steps = steps;
    /** @type {number[] | null} */
    this.dash = dash && dash.length ? dash : null;
    this.curve = curve;
    /** @type {{ value: number, color: string } | null} */
    this.under = under;
    // (Its stroke's style split at its under value, a color pass each, kept till it changes.)
    /** @type {Fill[]} */
    this.splits = [];
    /** How it widens its scale's range in its place (a page's own reading of it), none: by its extent. @type {((a: number, b: number, positive: boolean, out: Float64Array) => void) | null} */
    this.fit = null;
    // (Its last point with a value, found once a version: NaN, none.)
    this.lastAt = NaN;
    this.lastVersion = -1;
    // (lastBy's last ask and answer, and the version and scale they were for.)
    this.byAsk = -1;
    this.byAnswer = -1;
    this.byVersion = -1;
    this.byLog = false;
    /** Not drawn, fitted, tagged or hit (its pane drawn whole after a change: its chart's invalidate(true)). */
    this.hidden = false;
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

  /** @param {{ color?: string, width?: number, breaks?: Uint8Array | null, colors?: Uint8Array | Uint16Array | null, palette?: string[], tag?: boolean, priceLine?: boolean, stroke?: boolean, format?: ((v: number) => string) | null, outline?: string | null, dots?: number, marker?: number, steps?: boolean, dash?: number[] | null, curve?: boolean, under?: { value: number, color: string } | null }} look */
  setStyle({ color = this.color, width = this.width, breaks = this.breaks, colors = this.colors, palette = this.palette, tag = this.tag, priceLine = this.priceLine, stroke = this.stroke, format, outline = this.outline, dots = this.dots, marker = this.marker, steps = this.steps, dash = this.dash, curve = this.curve, under = this.under }) {
    this.version++;
    (this.color = color), (this.width = width), (this.breaks = breaks), (this.colors = colors), (this.palette = palette), (this.tag = tag), (this.priceLine = priceLine), (this.stroke = stroke), (this.outline = outline), (this.dots = dots), (this.marker = marker), (this.steps = steps), (this.dash = dash && dash.length ? dash : null), (this.curve = curve), (this.under = under);
    if (format !== undefined) this.format = format && kept(format);
  }

  /** What it draws can't move with a pan's picture (see Pane's draw): its dashes, from where its path starts. */
  get dashed() {
    return this.dash !== null && this.stroke;
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

  /** Its last point with a value at or before index `i` (on a log scale, above zero: a tag's, the last in view), or -1. @param {number} i @param {boolean} log */
  lastBy(i, log) {
    const last = this.last(log), values = this.values;
    if (i >= last) return last;
    if (i < 0) return -1;
    // (Its last ask and answer kept: a pan's asks are near each other, and between the two there's none. Past the ask,
    // only the points since read; else from `i` back.)
    const kept = this.byVersion === this.version && this.byLog === log;
    if (kept && i >= this.byAnswer && i <= this.byAsk) return this.byAnswer;
    const stop = kept && i > this.byAsk ? this.byAsk : -1;
    let k = i;
    while (k > stop && !(log ? values[k] > 0 : values[k] === values[k])) k--;
    (this.byVersion = this.version), (this.byLog = log), (this.byAsk = i), (this.byAnswer = k > stop ? k : stop >= 0 ? this.byAnswer : -1);
    return this.byAnswer;
  }

  /** Its first value at or after index `i` (a relative scale's base: its first in view), or NaN: none. @param {number} i */
  first(i) {
    const values = this.values;
    for (let k = Math.max(0, i); k < values.length; k++) if (values[k] === values[k]) return values[k];
    return NaN;
  }

  /**
   * How far a point (`x` into the plot, `y` in its pane: CSS pixels) is from it, measured against its drawing there
   * (see recorder.js: whatever it draws, as it draws it, at any zoom), on `scale` (its base set already, where it's
   * relative). At index `i` the pane's nearest point (unread: its drawing has it).
   * @param {number} i @param {number} y @param {import("./scale.js").Scale} scale @param {number} x @param {import("./timeline.js").Timeline} time
   */
  distance(i, y, scale, x, time) {
    return measure(this, time, scale, x, y);
  }



  /** Its color at a point: its own, or its palette's there. @param {number} i */
  colorAt(i) {
    // (Below its under value, that color: a baseline's tag, its dots and its crosshair's dot under its base.)
    if (this.under && this.values[i] < this.under.value) return this.under.color;
    return this.colors && this.palette.length ? this.palette[this.colors[i]] ?? this.color : this.color;
  }

  /**
   * The control points of its curved segment to point `i` from `i - 1` (CSS pixels, into BEND), as the chart library
   * bent them: each a sixth of the way along its neighbors' chord, those by index within its continuous run (a gap or a
   * break, the point itself), so a strip's and a whole draw's alike.
   * @param {number} i @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x1 @param {number} y1 @param {number} x2 @param {number} y2
   */
  #bend(i, time, scale, x1, y1, x2, y2) {
    const values = this.values, breaks = this.breaks, log = scale.mode === "log", w = values[i - 2], u = values[i + 1];
    const before = i >= 2 && (log ? w > 0 : w === w) && !(breaks && breaks[i - 2] === 1), after = i + 1 < values.length && (log ? u > 0 : u === u) && !(breaks && breaks[i] === 1);
    const x0 = before ? time.x(i - 2) : x1, y0 = before ? scale.y(w) : y1, x3 = after ? time.x(i + 1) : x2, y3 = after ? scale.y(u) : y2;
    (BEND[0] = x1 + (x2 - x0) / 6), (BEND[1] = y1 + (y2 - y0) / 6), (BEND[2] = x2 - (x3 - x1) / 6), (BEND[3] = y2 - (y3 - y1) / 6);
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
    const ta = log ? Math.log10(scale.lo) : scale.lo, tb = log ? Math.log10(scale.hi) : scale.hi, top = scale.top * scale.height + scale.above, room = scale.height - top - scale.bottom * scale.height - scale.below;
    const k = room / (tb - ta), bottom = top + room, m = scale.m, c = scale.c;
    // (Each column from where the last one ended: its start computed once.)
    let start = Math.ceil(from + ((Math.floor(x0) - 1) * span) / width - NOISE);
    for (let x = Math.floor(x0) - 1; x <= Math.ceil(x1) + 1; x++) {
      const end = Math.ceil(from + ((x + 1) * span) / width - NOISE), a = start < 0 ? 0 : start, b = end - 1 < n - 1 ? end - 1 : n - 1;
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
    if (this.stroke) this.#stroke(g, time, scale, x0, x1, dpr);
    // (Its dots over its line.)
    if (this.dots) this.#dots(g, time, scale, x0, x1);
  }

  /**
   * Its line stroked from x `x0` to `x1`: its outline first, then a stroke a color.
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr
   */
  #stroke(g, time, scale, x0, x1, dpr) {
    const colors = this.colors && this.palette.length ? this.colors : null;
    // (Its points as far past the strip as its stroke reaches (its outline's, three times as wide), so a strip draws
    // what reaches in from beside it; dashed as asked.)
    const reach = Math.ceil((this.width * (this.outline ? 3 : 1)) / 2) + 1;
    (x0 -= reach), (x1 += reach);
    if (this.dash) g.setLineDash(this.dash);
    // (Its outline first: its whole path in it, three times as wide, round at its joins (bevelled, its corners show);
    // dashed and colored, a color's segments at a time, as its strokes are, so their dashes start where theirs do.)
    if (this.outline) {
      g.lineWidth = this.width * 3;
      g.lineJoin = "round";
      g.strokeStyle = this.outline;
      for (let pass = this.dash && colors ? 0 : -1; pass < (this.dash && colors ? this.palette.length : 0); pass++) {
        g.beginPath();
        pen.start(g, dpr, pass);
        this.#trace(time, scale, x0, x1, colors);
        pen.lift();
        g.stroke();
      }
    }
    g.lineWidth = this.width;
    // (Bevelled: as round at a line's width, half the cost to stroke in Firefox, and no miter's spikes.)
    g.lineJoin = "bevel";
    // (One stroke a color: its segments in it. Measured (see recorder.js), all of them in one: their union, as drawn.)
    const passes = colors && !measuring ? this.palette.length : 0;
    for (let pass = passes ? 0 : -1; pass < passes; pass++) {
      // (Split at its under value where the scale shows it (a log scale can't show 0 or under: all above it).)
      const under = this.under, color = pass < 0 ? this.color : this.palette[pass], at = under ? scale.y(under.value) : NaN;
      g.strokeStyle = under && at === at ? (this.splits[pass + 1] ??= new Fill()).splitAt(g, color, under.color, at, scale.height) : color;
      g.beginPath();
      pen.start(g, dpr, pass);
      this.#trace(time, scale, x0, x1, colors);
      pen.lift();
      g.stroke();
    }
    if (this.dash) g.setLineDash(SOLID);
  }

  /**
   * Its dots from x `x0` to `x1` (and those reaching in), each color's in one fill, ringed in its outline first.
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1
   */
  #dots(g, time, scale, x0, x1) {
    const values = this.values, r = this.dots, ring = this.outline ? r + Math.max(1, this.width) : 0, reach = Math.max(r, ring);
    const a = Math.max(0, Math.floor(time.index(x0 - reach))), b = Math.min(values.length - 1, Math.ceil(time.index(x1 + reach)));
    const colors = this.colors && this.palette.length ? this.colors : null, under = this.under;
    // (Its rings, then a fill a color; those below its under value last, in that color.)
    for (let pass = ring ? -2 : colors ? 0 : -1; pass < (colors ? this.palette.length : 0) + (under ? 1 : 0); pass++) {
      if (pass === -1 && colors) continue;
      const below = Boolean(under) && pass === (colors ? this.palette.length : 0);
      g.fillStyle = pass === -2 ? /** @type {string} */ (this.outline) : below ? /** @type {{ color: string }} */ (under).color : pass < 0 ? this.color : this.palette[pass];
      g.beginPath();
      for (let i = a; i <= b; i++) {
        const v = values[i];
        if (v !== v || (pass !== -2 && (under ? v < under.value : false) !== below) || (colors && pass >= 0 && !below && colors[i] !== pass)) continue;
        const x = time.x(i), y = scale.y(v), size = pass === -2 ? ring : r;
        g.moveTo(x + size, y);
        g.arc(x, y, size, 0, 2 * Math.PI);
      }
      g.fill();
    }
  }

  /**
   * Its points from `x0` to `x1`, written to the pen.
   * @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale @param {number} x0 @param {number} x1
   * @param {Uint8Array | Uint16Array | null} colors
   */
  #trace(time, scale, x0, x1, colors) {
    const values = this.values, breaks = this.breaks, n = values.length, log = scale.mode === "log";
    if (time.spacing >= 1) {
      // (A pixel or more a point: each point, and one past each end, so the line reaches the edges.)
      const a = Math.max(0, Math.floor(time.index(x0)) - 1), b = Math.min(n - 1, Math.ceil(time.index(x1)) + 1);
      // (Stepped: each value held to the next point, then up or down to its value there.)
      for (let i = a; i <= b; i++) {
        const v = values[i];
        if (!(log ? v > 0 : v === v)) {
          pen.lift();
          continue;
        }
        const x = time.x(i), y = scale.y(v), color = colors ? colors[i] : 0, cut = breaks ? breaks[i] === 1 : false;
        if (this.steps && pen.has && !pen.cut) pen.to(x, pen.py, pen.color, false);
        // (Curved, not stepped: steps take it.)
        if (this.curve && !this.steps && pen.has && !pen.cut) this.#bend(i, time, scale, pen.px, pen.py, x, y), pen.curve(BEND[0], BEND[1], BEND[2], BEND[3], x, y, color, cut);
        else pen.to(x, y, color, cut);
      }
      return;
    }
    if (!colors && !breaks) return this.#plain(time, scale, x0, x1);
    // (Each pixel column's points as runs: its first, low, high and last; a point with no value ends a run and breaks
    // the line there, in its column too; a break ends a run after its point. Columns at whole pixels of the view, their
    // points at whole indices: a view moved by whole pixels holds the same columns.)
    for (let x = Math.floor(x0) - 1; x <= Math.ceil(x1) + 1; x++) {
      const a = Math.max(0, Math.ceil(time.index(x) - NOISE)), b = Math.min(n - 1, Math.ceil(time.index(x + 1) - NOISE) - 1);
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
