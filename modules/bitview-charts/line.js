// @ts-check
// A line: its values as one column on the chart's timeline (a Float64Array, an index a time; NaN: none there, the line
// broken), its color and width. Its values summed up by blocks of 4, 8, 16… points (its summary: a view's extremes,
// and the lowest above zero a log scale's, read by its largest whole blocks). Drawn point to point where a point has a pixel or more; zoomed
// out, each pixel column as its first, low, high and last values (as many points as the column holds, drawn as the eye
// sees them). `breaks` (a Uint8Array, 1 at a point): the line stops after that point, nothing drawn to the next (a
// trend's flip, a stretch begun anew). `colors` (a Uint8Array or Uint16Array of indices into `palette`): each segment
// in its first point's color.

import { Summary, FIRST } from "./summary.js";
import { kept } from "./format.js";
import { Fill } from "./color.js";
import { NOISE } from "./timeline.js";
import { measure, measuring, painted } from "./recorder.js";

/** A curved segment's control points, as #bend writes them (made once). */
const BEND = new Float64Array(4);
/** No dashes (made once). */
const SOLID = /** @type {number[]} */ ([]);
/** CSS pixels along a dashed line between the places its subpath may be split (see the pen's to()). */
const SPLIT = 128;
/** A dashed subpath longer than this never split (no point of it in a gap where it could be: its segments each about
 * a multiple of its dashes' period): its line drawn whole as the view moves (see Line's drifts). Found by a whole
 * draw: a pan's strips see short subpaths alone, so the first pan into such a stretch ends in one shift of its dashes
 * there (its whole draw at rest), the pans after it drawn whole. */
const DRIFT = 16 * SPLIT;

/** A dash list's period: its sum, twice where it's odd (the canvas repeats it). @param {number[]} dash */
export function periodOf(dash) {
  let sum = 0;
  for (let k = 0; k < dash.length; k++) sum += dash[k];
  return dash.length % 2 ? 2 * sum : sum;
}

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
  /** Which colors' segments it draws, where not all (a measure's one pass: those that paint), else null. @type {Uint8Array | null} */
  shown: null,
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
  // (A dashed line's (see Line's #stroke): its dashes and their period (0: not kept), its path's length so far from its
  // dash origin, and at the point before; whether a subpath's been begun (each its own stroke); whether it's a ruler's
  // (its length measured alone, nothing drawn: see Line's #anchor); the point whose vertex is written next (a point's
  // own, zoomed in; a column's first run's, zoomed out), else -1; a ruler's asks: the length at point `seek`'s vertex
  // (`found`), and at the first from point `from` on (`first`, its point `firstId`); the last written (`last`, its
  // point `lastId`).)
  dashes: SOLID,
  period: 0,
  length: 0,
  before: 0,
  begun: false,
  ruling: false,
  id: -1,
  seek: -1,
  found: NaN,
  from: 0,
  first: NaN,
  firstId: -1,
  last: NaN,
  lastId: -1,
  // (Where its dashed subpath began, along the line, and the longest it's written this stroke: see DRIFT.)
  subStart: 0,
  longest: 0,
  /**
   * Begun on `g`; dashed (`dashes`, their period `period`), its path `length` along from its dash origin where it
   * starts.
   * @param {CanvasRenderingContext2D} g @param {number} dpr @param {number} pass @param {number} [length] @param {number[]} [dashes] @param {number} [period]
   */
  start(g, dpr, pass, length = 0, dashes = SOLID, period = 0) {
    (this.g = g), (this.dpr = dpr), (this.pass = pass), (this.shown = null);
    (this.on = false), (this.has = false), (this.runX = NaN);
    (this.dashes = dashes), (this.period = period), (this.length = length), (this.before = length), (this.begun = false), (this.ruling = false), (this.id = -1), (this.seek = -1), (this.found = NaN), (this.first = NaN), (this.firstId = -1), (this.lastId = -1), (this.subStart = length), (this.longest = 0);
  },
  /** @param {number} x @param {number} y @param {number} color @param {boolean} cut */
  to(x, y, color, cut) {
    const g = /** @type {CanvasRenderingContext2D} */ (this.g);
    y = Math.round(y * this.dpr) / this.dpr;
    if (this.ruling) return void (this.measure(x, y), (this.has = true), (this.px = x), (this.py = y), (this.cut = cut));
    // (A dashed subpath ended at a point in a gap (nothing drawn there, so no join's missed), the next begun there: the
    // canvas measures a subpath in single precision, its dashes drifting off ours along a long one (a zoomed-out line's
    // runs, a pixel off its strips' over some thousands of pixels). Where, by the line's length alone (see splits): a
    // strip and a whole draw end their strokes at the same points.)
    if (this.on && this.period && this.splits()) this.close();
    if (this.has && !this.cut && (this.pass < 0 ? !this.shown || this.shown[this.color] === 1 : this.color === this.pass)) {
      if (!this.on) {
        if (this.period) this.dash(g);
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
    if (this.period) this.measure(x, y);
    (this.has = true), (this.px = x), (this.py = y), (this.color = color), (this.cut = cut);
  },
  /**
   * A dashed subpath begun at the last point: the one before stroked (each its own stroke), its dashes from its dash
   * origin, as far along the path as it starts.
   * @param {CanvasRenderingContext2D} g
   */
  dash(g) {
    if (this.begun) g.stroke(), g.beginPath();
    this.subStart = this.length;
    g.lineDashOffset = this.length % this.period;
    this.begun = true;
  },
  /**
   * Whether its subpath ends at the last point (see to()): the first point in the gap that ends its dashes' period (the
   * list's last, a gap however long the list: the canvas repeats an odd one twice), that gap the first to start at or
   * past a multiple of SPLIT along the line.
   */
  splits() {
    const period = this.period, gap = this.dashes[this.dashes.length - 1], start = period * Math.floor((this.length + gap) / period) - gap;
    return this.length - start < gap && this.before < start && Math.floor(start / SPLIT) > Math.floor((start - period) / SPLIT);
  },
  /** The path's length to the point at `x`, `y` (from the last, where it's joined to it), a ruler's asks noted there. @param {number} x @param {number} y */
  measure(x, y) {
    this.before = this.length;
    if (this.has && !this.cut) {
      const dx = x - this.px, dy = y - this.py;
      this.length += Math.sqrt(dx * dx + dy * dy);
    }
    const id = this.id;
    if (id < 0) return;
    if (id === this.seek) this.found = this.length;
    if (this.firstId < 0 && id >= this.from) (this.firstId = id), (this.first = this.length);
    (this.lastId = id), (this.last = this.length);
  },
  /**
   * As to(), its segment from the last point a curve through two control points (CSS pixels).
   * @param {number} c1x @param {number} c1y @param {number} c2x @param {number} c2y @param {number} x @param {number} y @param {number} color @param {boolean} cut
   */
  curve(c1x, c1y, c2x, c2y, x, y, color, cut) {
    const g = /** @type {CanvasRenderingContext2D} */ (this.g);
    y = Math.round(y * this.dpr) / this.dpr;
    if (this.has && !this.cut && (this.pass < 0 ? !this.shown || this.shown[this.color] === 1 : this.color === this.pass)) {
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
    // (A dashed subpath's length, the longest kept: see DRIFT.)
    if (this.on && this.period) this.longest = Math.max(this.longest, this.length - this.subStart);
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
   * Written, if it has points, each at its own point's x (see Line's #run); then begun again.
   * @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale @param {Uint8Array | Uint16Array | null} colors @param {Uint8Array | null} breaks
   */
  end(time, scale, colors, breaks) {
    const { first, last, lo, hi } = this;
    if (first !== first) return;
    const color = colors ? colors[this.firstAt] : 0;
    pen.to(time.x(this.firstAt), scale.y(first), color, false);
    pen.id = -1;
    const low = lo < first && lo < last, high = hi > first && hi > last;
    if (low && high && this.hiAt < this.loAt) pen.to(time.x(this.hiAt), scale.y(hi), color, false), pen.to(time.x(this.loAt), scale.y(lo), color, false);
    else {
      if (low) pen.to(time.x(this.loAt), scale.y(lo), color, false);
      if (high) pen.to(time.x(this.hiAt), scale.y(hi), color, false);
    }
    pen.to(time.x(this.lastAt), scale.y(last), colors ? colors[this.lastAt] : 0, breaks ? breaks[this.lastAt] === 1 : false);
    this.start();
  },
};

/** A faint series' opacity (`faint`: another lit over it): a fifth, its tag and price line gone. */
export const FAINT = 0.2;

/**
 * A line's pixel columns as #plain read them zoomed out (each by its points, `a` to `b`: its low and high and where
 * they are; its first and last, its points' own), kept while its values and its scale's kind stay: a pan by whole
 * pixels, or a refit of its scale alone, reads the same columns again, each in its slot by its place on the time axis
 * (one a pan doesn't change; `size` of them, a power of two past the plot's width) and found there by its points (a
 * column of other points read anew: a zoom). Only a column of one run (no point it can't go through).
 */
class Columns {
  /** @param {number} size */
  constructor(size) {
    this.size = size;
    (this.a = new Int32Array(size)), (this.b = new Int32Array(size)), (this.stamp = new Int32Array(size));
    (this.lo = new Float64Array(size)), (this.hi = new Float64Array(size)), (this.loAt = new Int32Array(size)), (this.hiAt = new Int32Array(size));
    /** Its slots' stamp now (one of another stamp, another version's: none); its line's version and scale's kind. */
    (this.now = 1), (this.version = -1), (this.log = false);
  }
}

export class Line {
  /**
   * `tag`: its last value in a badge on its pane's axis, in its color (written by `format`, else as its side writes
   * values); `priceLine`: a dotted line across at it; `stroke: false`, no line drawn (its tag and price line still);
   * `outline`, a color it's edged in (under it, its width each side: over a fill or a line, it stands off it); `dots`, a
   * dot that radius at each point (in its color there; ringed in its outline); `marker`, the crosshair's dot on it that
   * radius (ringed in the ground, 2 pixels; 0: none); `steps`, each value held to the next point (zoomed in: zoomed out,
   * a pixel column holds several points either way); `dash`, its stroke's dashes (CSS pixels, as setLineDash takes
   * them; none, solid): fixed to its path, so they move with it (curved, its pane's drawn whole as the view moves: its
   * dashes from where its path starts); `curve`, bowed through its points (zoomed in), as the chart library's
   * curves (each segment's bend from its points' neighbors); `under`, its stroke in another color below a value (a
   * baseline's line: its color changing where it crosses its base).
   * @param {{ values?: Float64Array, color?: string, width?: number, breaks?: Uint8Array | null, colors?: Uint8Array | Uint16Array | null, palette?: string[], tag?: boolean, priceLine?: boolean, stroke?: boolean, format?: ((v: number) => string) | null, outline?: string | null, dots?: number, marker?: number, steps?: boolean, dash?: number[] | null, curve?: boolean, under?: { value: number, color: string } | null }} [options]
   */
  constructor({ values = new Float64Array(0), color = "#f7f8f8", width = 1.5, breaks = null, colors = null, palette = [], tag = false, priceLine = false, stroke = true, format = null, outline = null, dots = 0, marker = 3, steps = false, dash = null, curve = false, under = null } = {}) {
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
    // (Its dashes' anchors: the first and last points its last draw drew (-1: none yet), its path's length at each from
    // its dash origin.)
    this.dashFirst = -1;
    this.dashFirstAt = 0;
    this.dashLast = -1;
    this.dashLastAt = 0;
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
    /** Drawn at a fifth of its opacity, its tag and price line gone (another lit over it: see the pane's Series). */
    this.faint = false;
    /** A scale of its own (see the pane's Series), none: its side's. @type {import("./scale.js").Scale | null} */
    this.overlay = null;
    /** Changed at each change of its values or look: its pane's picture drawn whole then. */
    this.version = 0;
    /** @type {Float64Array} */
    this.values = values;
    /** Its values summed up by blocks, for drawing zoomed out and fitting (see summary.js). @type {Summary | null} */
    this.summary = null;
    /** A dashed subpath of it written longer than DRIFT, never split: drawn whole as the view moves (till its values or look change). */
    this.drifts = false;
    /** Its pixel columns as #plain last read them (see Columns), made once it's drawn zoomed out. @type {Columns | null} */
    this.columns = null;
    this.setValues(values);
  }

  /** @param {Float64Array} values */
  setValues(values) {
    this.version++;
    this.values = values;
    this.drifts = false;
    this.summary = values.length >= 1 << FIRST ? new Summary(values) : null;
  }

  /** One value set in place (a live point), its block's extremes made again. @param {number} i @param {number} value */
  set(i, value) {
    this.version++;
    this.values[i] = value;
    this.summary?.set(i);
  }

  /** @param {{ color?: string, width?: number, breaks?: Uint8Array | null, colors?: Uint8Array | Uint16Array | null, palette?: string[], tag?: boolean, priceLine?: boolean, stroke?: boolean, format?: ((v: number) => string) | null, outline?: string | null, dots?: number, marker?: number, steps?: boolean, dash?: number[] | null, curve?: boolean, under?: { value: number, color: string } | null }} look */
  setStyle({ color = this.color, width = this.width, breaks = this.breaks, colors = this.colors, palette = this.palette, tag = this.tag, priceLine = this.priceLine, stroke = this.stroke, format, outline = this.outline, dots = this.dots, marker = this.marker, steps = this.steps, dash = this.dash, curve = this.curve, under = this.under }) {
    this.version++;
    this.drifts = false;
    (this.color = color), (this.width = width), (this.breaks = breaks), (this.colors = colors), (this.palette = palette), (this.tag = tag), (this.priceLine = priceLine), (this.stroke = stroke), (this.outline = outline), (this.dots = dots), (this.marker = marker), (this.steps = steps), (this.dash = dash && dash.length ? dash : null), (this.curve = curve), (this.under = under);
    if (format !== undefined) this.format = format && kept(format);
  }

  /** Its picture can't be stretched (see Pane's draw): its dashes would stretch with it. */
  get dashed() {
    return this.dash !== null && this.stroke;
  }

  /** What it draws can't move with a pan's picture (see Pane's draw): a curve's dashes, from where its path starts; a dashed line's that drift (see DRIFT). */
  get immovable() {
    return this.dashed && ((this.curve && !this.steps) || this.drifts);
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
    const k = room / (tb - ta), bottom = top + room, m = scale.m, c = scale.c, px = width / span;
    // (Its columns as last read, while its values and its scale's kind stay.)
    // (Made again, wider, for a plot wider than its slots.)
    const size = 2 ** Math.ceil(Math.log2(Math.max(64, width + 4)));
    if (!this.columns || this.columns.size < size) this.columns = new Columns(size);
    const columns = this.columns, mask = columns.size - 1;
    if (columns.version !== this.version || columns.log !== log) (columns.version = this.version), (columns.log = log), columns.now++;
    const now = columns.now, cA = columns.a, cB = columns.b, cStamp = columns.stamp, origin = Math.round(from * px);
    // (Each column from where the last one ended: its start computed once.)
    let start = Math.ceil(from + ((Math.floor(x0) - 1) * span) / width - NOISE);
    for (let x = Math.floor(x0) - 1; x <= Math.ceil(x1) + 1; x++) {
      const end = Math.ceil(from + ((x + 1) * span) / width - NOISE), a = start < 0 ? 0 : start, b = end - 1 < n - 1 ? end - 1 : n - 1;
      start = end;
      pen.id = a;
      // (The same points as a column read before: its run as it was.)
      const slot = (x + origin) & mask;
      if (cStamp[slot] === now && cA[slot] === a && cB[slot] === b) {
        this.#run(from, px, values[a], values[b], columns.lo[slot], columns.hi[slot], a, b, columns.loAt[slot], columns.hiAt[slot], bottom, k, ta, m, c, log);
        continue;
      }
      let first = NaN, last = NaN, lo = Infinity, hi = -Infinity, loAt = 0, hiAt = 0, firstAt = 0, lastAt = 0, i = a, clean = true;
      while (i <= b) {
        // (The largest clean block starting at i and ending by b: as large as i's alignment and what's left allow.)
        if (levels && !(i & ((1 << FIRST) - 1)) && b - i + 1 >= 1 << FIRST) {
          let size = Math.min(i ? i & -i : 1 << 30, 1 << (31 - Math.clz32(b - i + 1)));
          for (; size >= 1 << FIRST; size >>= 1) {
            const level = 31 - Math.clz32(size) - FIRST;
            if (level >= levels.length) continue;
            const at = levels[level], k = i >> (level + FIRST);
            if (k >= at.first.length || (log ? at.low[k] : at.nan[k])) continue;
            if (first !== first) (first = at.first[k]), (firstAt = i);
            (last = at.last[k]), (lastAt = i + size - 1);
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
          if (first !== first) (first = v), (firstAt = i);
          (last = v), (lastAt = i);
          if (v < lo) (lo = v), (loAt = i);
          if (v > hi) (hi = v), (hiAt = i);
        } else {
          // (No value: the run so far written, the line broken.)
          if (first === first) this.#run(from, px, first, last, lo, hi, firstAt, lastAt, loAt, hiAt, bottom, k, ta, m, c, log), (first = NaN), (lo = Infinity), (hi = -Infinity);
          pen.lift();
          clean = false;
        }
        i++;
      }
      if (first === first) {
        this.#run(from, px, first, last, lo, hi, firstAt, lastAt, loAt, hiAt, bottom, k, ta, m, c, log);
        // (Kept clean: its first and last its own end points.)
        if (clean) (cStamp[slot] = now), (cA[slot] = a), (cB[slot] = b), (columns.lo[slot] = lo), (columns.hi[slot] = hi), (columns.loAt[slot] = loAt), (columns.hiAt[slot] = hiAt);
      }
    }
  }

  /**
   * A run written: its first, its low and high past its first and last in the order they came, its last, each at its
   * own point's x (as Timeline.x: `from`, `px` pixels a point): a trend through a column drawn as one slope, not a stroke
   * down the column and a step to the next (stairs, on a screen of two or three pixels a CSS pixel).
   * @param {number} from @param {number} px @param {number} first @param {number} last @param {number} lo @param {number} hi
   * @param {number} firstAt @param {number} lastAt @param {number} loAt @param {number} hiAt @param {number} bottom
   * @param {number} k @param {number} ta @param {number} m @param {number} c @param {boolean} log
   */
  #run(from, px, first, last, lo, hi, firstAt, lastAt, loAt, hiAt, bottom, k, ta, m, c, log) {
    pen.to((firstAt - from) * px, bottom - k * ((log ? Math.log10(first) : first * m + c) - ta), 0, false);
    pen.id = -1;
    const low = lo < first && lo < last, high = hi > first && hi > last;
    // (Flat: its first and last written (the pen runs them as one segment), its plateau to its last point's x.)
    if (low && high && hiAt < loAt) pen.to((hiAt - from) * px, bottom - k * ((log ? Math.log10(hi) : hi * m + c) - ta), 0, false), pen.to((loAt - from) * px, bottom - k * ((log ? Math.log10(lo) : lo * m + c) - ta), 0, false);
    else {
      if (low) pen.to((loAt - from) * px, bottom - k * ((log ? Math.log10(lo) : lo * m + c) - ta), 0, false);
      if (high) pen.to((hiAt - from) * px, bottom - k * ((log ? Math.log10(hi) : hi * m + c) - ta), 0, false);
    }
    pen.to((lastAt - from) * px, bottom - k * ((log ? Math.log10(last) : last * m + c) - ta), 0, false);
  }

  /**
   * Its lowest and highest from index `a` to `b` (its summary's largest whole blocks, the ends point by point), written into
   * `out` widened ([lo, hi]); `positive`: the lowest above zero.
   * @param {number} a @param {number} b @param {boolean} positive @param {Float64Array} out
   */
  extent(a, b, positive, out) {
    const values = this.values, levels = this.summary ? this.summary.levels : null;
    a = Math.max(0, a);
    b = Math.min(values.length - 1, b);
    let lo = out[0], hi = out[1], i = a;
    while (i <= b) {
      // (The largest whole block starting at i and ending by b, as #plain reads them.)
      if (levels && !(i & ((1 << FIRST) - 1)) && b - i + 1 >= 1 << FIRST) {
        let size = Math.min(i ? i & -i : 1 << 30, 1 << (31 - Math.clz32(b - i + 1)));
        for (; size >= 1 << FIRST; size >>= 1) {
          const level = 31 - Math.clz32(size) - FIRST;
          if (level >= levels.length) continue;
          const at = levels[level], k = i >> (level + FIRST);
          if (k >= at.lo.length) continue;
          const low = positive ? at.pos[k] : at.lo[k];
          if (low < lo) lo = low;
          if (at.hi[k] > hi) hi = at.hi[k];
          break;
        }
        if (size >= 1 << FIRST) {
          i += size;
          continue;
        }
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
    // (Dashed, each subpath's dashes as far along from its origin as it starts: a pan's strip draws the dashes its moved
    // picture has (none of its points there, nothing to draw). Not a curve's: its length the canvas's own.)
    const dash = this.dash, period = dash && !measuring && !(this.curve && !this.steps) ? periodOf(dash) : 0;
    const phase = period > 0 && time.x(0) - 1 <= x1 && time.x(this.values.length - 1) + 1 >= x0 ? this.#anchor(g, time, scale, x0, x1, dpr, colors) : 0;
    if (dash) g.setLineDash(dash);
    // (Its outline first: its whole path in it, three times as wide, round at its joins (bevelled, its corners show);
    // dashed and colored, a color's segments at a time, as its strokes are, so their dashes start where theirs do.)
    if (this.outline) {
      g.lineWidth = this.width * 3;
      g.lineJoin = "round";
      g.strokeStyle = this.outline;
      const passes = this.dash && colors && !measuring ? this.palette.length : 0;
      for (let pass = passes ? 0 : -1; pass < passes; pass++) {
        g.beginPath();
        pen.start(g, dpr, pass, phase, dash ?? SOLID, period);
        this.#trace(time, scale, x0, x1, colors);
        pen.lift();
        g.stroke();
      }
    }
    g.lineWidth = this.width;
    // (Bevelled: as round at a line's width, half the cost to stroke in Firefox, and no miter's spikes.)
    g.lineJoin = "bevel";
    // (One stroke a color: its segments in it. Measured (see recorder.js), all of them in one pass, their union as
    // drawn: those of its colors that paint (a stroke of some paint for it, its segments chosen by `shown`; all, where
    // its under color paints: below its value they show in it).)
    const passes = colors && !measuring ? this.palette.length : 0, shown = colors && measuring && !this.palette.every(painted) && !(this.under && painted(this.under.color)) ? Uint8Array.from(this.palette, (c) => (painted(c) ? 1 : 0)) : null;
    for (let pass = passes ? 0 : -1; pass < passes; pass++) {
      // (Split at its under value where the scale shows it (a log scale can't show 0 or under: all above it).)
      const under = this.under, color = colors && measuring ? "#000" : pass < 0 ? this.color : this.palette[pass], at = under ? scale.y(under.value) : NaN;
      g.strokeStyle = under && at === at ? (this.splits[pass + 1] ??= new Fill()).splitAt(g, color, under.color, at, scale.height) : color;
      g.beginPath();
      pen.start(g, dpr, pass, phase, dash ?? SOLID, period);
      pen.shown = shown;
      this.#trace(time, scale, x0, x1, colors);
      pen.lift();
      g.stroke();
      if (period > 0 && pen.longest > DRIFT) this.drifts = true;
    }
    if (dash) g.setLineDash(SOLID), (g.lineDashOffset = 0);
    if (period > 0) (this.dashLast = pen.lastId), (this.dashLastAt = pen.last);
  }

  /**
   * Its path's length from its dash origin to the first point it draws from `x0` to `x1`: from the nearer of its last
   * draw's first and last points (their lengths known), measured along its path as drawn now (a pan's, as before; a
   * zoom's, carried on), that point its first from now. Both far (a jump, past two widths) or gone (a zoom out of its
   * columns): begun there anew.
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr @param {Uint8Array | Uint16Array | null} colors
   */
  #anchor(g, time, scale, x0, x1, dpr, colors) {
    // (Its first point drawn: the first of its own from where #trace begins.)
    const from = time.spacing >= 1 ? Math.max(0, Math.floor(time.index(x0)) - 1) : Math.max(0, Math.ceil(time.index(Math.floor(x0) - 1) - NOISE));
    const a = this.dashFirst, b = this.dashLast, ax = a >= 0 ? time.x(a) : Infinity, bx = b >= 0 ? time.x(b) : Infinity, nearer = Math.abs(ax - x0) <= Math.abs(bx - x0);
    let origin = nearer ? a : b, ox = nearer ? ax : bx;
    if (!(Math.abs(ox - x0) <= 2 * time.width)) (origin = -1), (ox = x0);
    // (Measured from it to that point, on to the draw's end where there's none till then; within its points (a view
    // far wider than its data has columns of none).)
    const start = Math.max(Math.min(ox, x0), time.x(0) - 1), last = time.x(this.values.length - 1) + 1;
    for (let k = 0; k < 2; k++) {
      pen.start(g, dpr, -1);
      (pen.ruling = true), (pen.seek = origin), (pen.from = from);
      this.#trace(time, scale, start, Math.min(Math.max(ox, k ? x1 : x0), last), colors);
      pen.lift();
      if (pen.firstId >= 0) break;
    }
    if (pen.firstId < 0) return 0;
    const at = origin >= 0 && pen.found === pen.found ? (nearer ? this.dashFirstAt : this.dashLastAt) + pen.first - pen.found : 0;
    (this.dashFirst = pen.firstId), (this.dashFirstAt = at);
    return at;
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
        pen.id = i;
        // (Curved, not stepped: steps take it.)
        if (this.curve && !this.steps && pen.has && !pen.cut) this.#bend(i, time, scale, pen.px, pen.py, x, y), pen.curve(BEND[0], BEND[1], BEND[2], BEND[3], x, y, color, cut);
        else pen.to(x, y, color, cut);
        pen.id = -1;
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
      pen.id = a;
      for (let i = a; i <= b; i++) {
        const v = values[i];
        if (log ? v > 0 : v === v) {
          // (A color changing in the column: a run each, so each segment keeps its own.)
          if (colors && run.first === run.first && colors[i] !== colors[run.firstAt]) run.end(time, scale, colors, breaks);
          run.add(v, i);
          if (breaks && breaks[i] === 1) run.end(time, scale, colors, breaks);
        } else run.end(time, scale, colors, breaks), pen.lift();
      }
      run.end(time, scale, colors, breaks);
    }
  }


}
