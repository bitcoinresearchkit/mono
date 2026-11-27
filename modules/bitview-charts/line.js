// @ts-check
// A line (NaN: a gap): point to point; zoomed out, each pixel column's low and high in the order they came.

import { Summary, FIRST } from "./summary.js";
import { kept } from "./format.js";
import { Fill, painted, zoneSpans } from "./color.js";
import { NOISE, firstFrom } from "./timeline.js";
import { measure, measuring } from "./recorder.js";

const BEND = new Float64Array(4);
/** -1: under them all. @param {readonly import("./color.js").Zone[]} zones @param {number} v */
function zoneIndex(zones, v) {
  let k = -1;
  while (k + 1 < zones.length && v >= zones[k + 1].value) k++;
  return k;
}
/** @param {readonly import("./color.js").Zone[]} zones @param {number} v @param {string} own */
const zoneOf = (zones, v, own) => ((k) => (k < 0 ? own : zones[k].color))(zoneIndex(zones, v));
/** In widths of its line. */
const PARTIAL_DASH = [2, 2];
const SOLID = /** @type {number[]} */ ([]);
const SPLIT = 128;
/** A dashed subpath longer than this is never split, so its line is drawn whole as the view moves (only a whole draw finds one: a pan's strips see short subpaths alone). */
const DRIFT = 16 * SPLIT;

/** Twice its sum where odd: the canvas repeats an odd list. @param {number[]} dash */
export function periodOf(dash) {
  let sum = 0;
  for (let k = 0; k < dash.length; k++) sum += dash[k];
  return dash.length % 2 ? 2 * sum : sum;
}

/**
 * `pass` -1: all. A run at one y is one segment: a slow line zoomed out is mostly runs.
 */
const pen = {
  g: /** @type {CanvasRenderingContext2D | null} */ (null),
  dpr: 1,
  pass: -1,
  on: false,
  y: NaN,
  runX: NaN,
  has: false,
  px: 0,
  py: 0,
  color: 0,
  cut: false,
  // (Dashed only; `ruling`: lengths measured alone, nothing drawn.)
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
  subStart: 0,
  longest: 0,
  /** @param {CanvasRenderingContext2D} g @param {number} dpr @param {number} pass @param {number} [length] @param {number[]} [dashes] @param {number} [period] */
  start(g, dpr, pass, length = 0, dashes = SOLID, period = 0) {
    (this.g = g), (this.dpr = dpr), (this.pass = pass);
    (this.on = false), (this.has = false), (this.runX = NaN);
    (this.dashes = dashes), (this.period = period), (this.length = length), (this.before = length), (this.begun = false), (this.ruling = false), (this.id = -1), (this.seek = -1), (this.found = NaN), (this.first = NaN), (this.firstId = -1), (this.lastId = -1), (this.subStart = length), (this.longest = 0);
  },
  /** @param {number} x @param {number} y @param {number} color @param {boolean} cut */
  to(x, y, color, cut) {
    const g = /** @type {CanvasRenderingContext2D} */ (this.g);
    y = Math.round(y * this.dpr) / this.dpr;
    if (this.ruling) return void (this.measure(x, y), (this.has = true), (this.px = x), (this.py = y), (this.cut = cut));
    // (Split at a point in a gap: the canvas measures a subpath in single precision, so its dashes drift off ours along
    // a long one. Split by length alone, so a strip and a whole draw split alike.)
    if (this.on && this.period && this.splits()) this.close();
    if (this.has && !this.cut && (this.pass < 0 || this.color === this.pass)) {
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
  /** @param {CanvasRenderingContext2D} g */
  dash(g) {
    if (this.begun) g.stroke(), g.beginPath();
    this.subStart = this.length;
    g.lineDashOffset = this.length % this.period;
    this.begun = true;
  },
  /** The list's last entry is always a gap: the canvas repeats an odd list twice. */
  splits() {
    const period = this.period, gap = this.dashes[this.dashes.length - 1], start = period * Math.floor((this.length + gap) / period) - gap;
    return this.length - start < gap && this.before < start && Math.floor(start / SPLIT) > Math.floor((start - period) / SPLIT);
  },
  /** @param {number} x @param {number} y */
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
  /** @param {number} c1x @param {number} c1y @param {number} c2x @param {number} c2y @param {number} x @param {number} y @param {number} color @param {boolean} cut */
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
  /**
   * #plain's vertices (x NaN: a lift), each as `to(x, y, 0, false)`; a solid line's in one loop with its state in
   * locals: without a JIT, a call and its property traffic a vertex cost more than the vertex.
   * @param {Float64Array} xs @param {Float64Array} ys @param {Int32Array} ids @param {number} count
   */
  trail(xs, ys, ids, count) {
    if (this.period || this.ruling || this.pass >= 0) {
      for (let v = 0; v < count; v++) xs[v] === xs[v] ? ((this.id = ids[v]), this.to(xs[v], ys[v], 0, false)) : this.lift();
      return;
    }
    const g = /** @type {CanvasRenderingContext2D} */ (this.g), dpr = this.dpr;
    let on = this.on, has = this.has, cut = this.cut, at = this.y, runX = this.runX, px = this.px, py = this.py, drew = false;
    for (let v = 0; v < count; v++) {
      const x = xs[v];
      if (x !== x || !has || cut) {
        if (on && runX === runX) g.lineTo(runX, at);
        (on = false), (runX = NaN);
        if (x !== x) {
          has = false;
          continue;
        }
      }
      const y = Math.round(ys[v] * dpr) / dpr;
      if (has && !cut) {
        if (!on) g.moveTo(px, py), (on = true), (at = py), (runX = NaN);
        if (y === at) runX = x;
        else {
          if (runX === runX) g.lineTo(runX, at);
          g.lineTo(x, y);
          (at = y), (runX = NaN);
        }
      }
      (has = true), (cut = false), (px = x), (py = y), (drew = true);
    }
    (this.on = on), (this.has = has), (this.cut = cut), (this.y = at), (this.runX = runX), (this.px = px), (this.py = py);
    if (drew) this.color = 0;
  },
  lift() {
    this.close();
    this.has = false;
  },
  close() {
    if (this.on && this.runX === this.runX) /** @type {CanvasRenderingContext2D} */ (this.g).lineTo(this.runX, this.y);
    if (this.on && this.period) this.longest = Math.max(this.longest, this.length - this.subStart);
    (this.on = false), (this.runX = NaN);
  },
};

const run = {
  first: NaN,
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
    this.lastAt = i;
    if (v < this.lo) (this.lo = v), (this.loAt = i);
    if (v > this.hi) (this.hi = v), (this.hiAt = i);
  },

  /** @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale @param {Uint8Array | Uint16Array | null} colors @param {Uint8Array | null} breaks */
  end(time, scale, colors, breaks) {
    if (this.first !== this.first) return;
    const color = colors ? colors[this.firstAt] : 0, cut = breaks ? breaks[this.lastAt] === 1 : false, loFirst = this.loAt <= this.hiAt;
    pen.to(time.x(loFirst ? this.loAt : this.hiAt), scale.y(loFirst ? this.lo : this.hi), color, false);
    pen.id = -1;
    pen.to(time.x(loFirst ? this.hiAt : this.loAt), scale.y(loFirst ? this.hi : this.lo), color, cut);
    this.start();
  },
};

export const FAINT = 0.2;

/**
 * #plain's columns, kept while values and scale kind stay: a whole-pixel pan or a refit reads them again.
 */
class Columns {
  /** @param {number} size */
  constructor(size) {
    this.size = size;
    (this.a = new Int32Array(size)), (this.b = new Int32Array(size)), (this.stamp = new Int32Array(size));
    (this.lo = new Float64Array(size)), (this.hi = new Float64Array(size)), (this.loAt = new Int32Array(size)), (this.hiAt = new Int32Array(size));
    /** A slot of another stamp is stale. */
    (this.now = 1), (this.version = -1), (this.log = false);
  }
}

/**
 * #plain's vertices for pen.trail, handed over each TRAIL: drawn as they're found (WebKit with its JIT drew slower given
 * a whole line's calls at once), at most three more each segment.
 */
const TRAIL = 128, XS = new Float64Array(TRAIL + 3), YS = new Float64Array(TRAIL + 3), IDS = new Int32Array(TRAIL + 3);
let ORDER = new Int32Array(0), STARTS = new Int32Array(0);
/**
 * A counting sort: one walk, not one a color.
 * @param {Uint8Array | Uint16Array} colors @param {number} count @param {number} a @param {number} b
 */
function byColor(colors, count, a, b) {
  if (ORDER.length < b - a + 1) ORDER = new Int32Array(b - a + 1);
  if (STARTS.length < count + 1) STARTS = new Int32Array(count + 1);
  STARTS.fill(0, 0, count + 1);
  for (let i = a; i <= b; i++) STARTS[colors[i] + 1]++;
  for (let k = 0; k < count; k++) STARTS[k + 1] += STARTS[k];
  for (let i = a; i <= b; i++) ORDER[STARTS[colors[i]]++] = i;
  for (let k = count; k > 0; k--) STARTS[k] = STARTS[k - 1];
  STARTS[0] = 0;
}

export class Line {
  /**
     * `dash` moves with its path; `partial`: its last point's period still running, the segment to it dashed.
     * @param {{ values?: Float64Array, color?: string, width?: number, breaks?: Uint8Array | null, colors?: Uint8Array | Uint16Array | null, palette?: string[], tag?: boolean, priceLine?: boolean, stroke?: boolean, format?: ((v: number, step?: number) => string) | null, outline?: string | null, dots?: number, marker?: number, steps?: boolean, dash?: number[] | null, curve?: boolean, zones?: readonly import("./color.js").Zone[] | null, gradient?: import("./color.js").Gradient | null, partial?: boolean }} [options]
     */
  constructor({ values = new Float64Array(0), color = "#f7f8f8", width = 1.5, breaks = null, colors = null, palette = [], tag = false, priceLine = false, stroke = true, format = null, outline = null, dots = 0, marker = 3, steps = false, dash = null, curve = false, zones = null, gradient = null, partial = false } = {}) {
    this.color = color;
    this.width = width;
    this.breaks = breaks;
    this.colors = colors;
    this.palette = palette;
    this.tag = tag;
    this.priceLine = priceLine;
    this.stroke = stroke;
    /** @type {((v: number) => string) | null} */
    this.format = format && kept(format);
    /** @type {string | null} */
    this.outline = outline;
    this.dots = dots;
    this.marker = marker;
    this.steps = steps;
    /** @type {number[] | null} */
    this.dash = dash && dash.length ? dash : null;
    this.curve = curve;
    this.used = new Uint8Array(0);
    /** @type {readonly import("./color.js").Zone[] | null} */
    this.zones = zones && zones.length ? zones : null;
    /** @type {import("./color.js").Gradient | null} */
    this.gradient = gradient;
    this.partial = partial;
    this.dashFirst = -1;
    this.dashFirstAt = 0;
    this.dashLast = -1;
    this.dashLastAt = 0;
    /** @type {Fill | null} */
    this.strokeFill = null;
    /** @type {number[]} */
    this.spans = [];
    /** @type {Uint16Array | null} */
    this.graded = null;
    this.gradedVersion = -1;
    /** A page's own reading of its range; none: its extent. @type {((a: number, b: number, positive: boolean, out: Float64Array) => void) | null} */
    this.fit = null;
    this.lastAt = NaN;
    this.lastVersion = -1;
    this.byAsk = -1;
    this.byAnswer = -1;
    this.byVersion = -1;
    this.byLog = false;
    /** Its pane needs a whole draw after a change (invalidate(true)). */
    this.hidden = false;
    this.faint = false;
    /** @type {import("./scale.js").Scale | null} */
    this.overlay = null;
    this.version = 0;
    /** @type {Float64Array} */
    this.values = values;
    /** @type {Summary | null} */
    this.summary = null;
    this.drifts = false;
    /** @type {Columns | null} */
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

  /** @param {number} i @param {number} value */
  set(i, value) {
    const graded = this.gradedVersion === this.version ? this.graded : null, gradient = this.gradient;
    this.version++;
    this.values[i] = value;
    if (graded && gradient && i < graded.length) (graded[i] = value === value ? gradient.index(value) : 0), (this.gradedVersion = this.version);
    this.summary?.set(i);
  }

  /** @param {{ color?: string, width?: number, breaks?: Uint8Array | null, colors?: Uint8Array | Uint16Array | null, palette?: string[], tag?: boolean, priceLine?: boolean, stroke?: boolean, format?: ((v: number, step?: number) => string) | null, outline?: string | null, dots?: number, marker?: number, steps?: boolean, dash?: number[] | null, curve?: boolean, zones?: readonly import("./color.js").Zone[] | null, gradient?: import("./color.js").Gradient | null, partial?: boolean }} look */
  setStyle({ color = this.color, width = this.width, breaks = this.breaks, colors = this.colors, palette = this.palette, tag = this.tag, priceLine = this.priceLine, stroke = this.stroke, format, outline = this.outline, dots = this.dots, marker = this.marker, steps = this.steps, dash = this.dash, curve = this.curve, zones = this.zones, gradient = this.gradient, partial = this.partial }) {
    this.version++;
    this.drifts = false;
    (this.color = color), (this.width = width), (this.breaks = breaks), (this.colors = colors), (this.palette = palette), (this.tag = tag), (this.priceLine = priceLine), (this.stroke = stroke), (this.outline = outline), (this.dots = dots), (this.marker = marker), (this.steps = steps), (this.dash = dash && dash.length ? dash : null), (this.curve = curve), (this.zones = zones && zones.length ? zones : null), (this.gradient = gradient), (this.partial = partial);
    if (format !== undefined) this.format = format && kept(format);
  }

  /** Its picture can't be stretched: its dashes would stretch with it. */
  get dashed() {
    return this.dash !== null && this.stroke;
  }

  /** Can't move with a pan's picture: a curve's dashes run from where its path starts, and drifting ones. */
  get immovable() {
    return this.dashed && ((this.curve && !this.steps) || this.drifts);
  }

  /** Its last point with a value (log: above zero), or -1. @param {boolean} log */
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
    let i = this.lastAt === this.lastAt ? this.lastAt : -1;
    if (log) while (i >= 0 && !(this.values[i] > 0)) i--;
    return i;
  }

  /** @param {number} i @param {boolean} log */
  lastBy(i, log) {
    const last = this.last(log), values = this.values;
    if (i >= last) return last;
    if (i < 0) return -1;
    // (Last ask and answer kept: a pan's asks are near each other.)
    const kept = this.byVersion === this.version && this.byLog === log;
    if (kept && i >= this.byAnswer && i <= this.byAsk) return this.byAnswer;
    const stop = kept && i > this.byAsk ? this.byAsk : -1;
    let k = i;
    while (k > stop && !(log ? values[k] > 0 : values[k] === values[k])) k--;
    (this.byVersion = this.version), (this.byLog = log), (this.byAsk = i), (this.byAnswer = k > stop ? k : stop >= 0 ? this.byAnswer : -1);
    return this.byAnswer;
  }

  /** NaN: none. @param {number} i */
  first(i) {
    return firstFrom(this.values, i);
  }

  /** @param {number} _i @param {number} y @param {import("./scale.js").Scale} scale @param {number} x @param {import("./timeline.js").Timeline} time */
  distance(_i, y, scale, x, time) {
    return measure(this, time, scale, x, y);
  }

  /** @param {number} i */
  colorAt(i) {
    if (this.zones) return zoneOf(this.zones, this.values[i], this.color);
    if (this.gradient) return this.gradient.color(this.values[i]) ?? this.color;
    return this.colors && this.palette.length ? this.palette[this.colors[i]] ?? this.color : this.color;
  }

  pointColors() {
    if (!this.gradient) return this.colors && this.palette.length ? this.colors : null;
    if (this.gradedVersion !== this.version) (this.graded = this.gradient.indexes(this.values, this.graded)), (this.gradedVersion = this.version);
    return this.graded;
  }

  /** @returns {readonly string[]} */
  pointPalette() {
    return this.gradient ? this.gradient.colors : this.palette;
  }

  /**
     * Into BEND; neighbors by index within their run, so a strip's and a whole draw's alike.
     * @param {number} i @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
     * @param {number} x1 @param {number} y1 @param {number} x2 @param {number} y2
     */
  #bend(i, time, scale, x1, y1, x2, y2) {
    const values = this.values, breaks = this.breaks, log = scale.mode === "log", w = values[i - 2], u = values[i + 1];
    const before = i >= 2 && (log ? w > 0 : w === w) && !(breaks && breaks[i - 2] === 1), after = i + 1 < values.length && (log ? u > 0 : u === u) && !(breaks && breaks[i] === 1);
    const x0 = before ? time.x(i - 2) : x1, y0 = before ? scale.y(w) : y1, x3 = after ? time.x(i + 1) : x2, y3 = after ? scale.y(u) : y2;
    // (Inlined: a shared helper's call costs too much a segment without a JIT.)
    BEND[0] = x1 + (x2 - x0) / 6;
    BEND[1] = y1 + (y2 - y0) / 6;
    BEND[2] = x2 - (x3 - x1) / 6;
    BEND[3] = y2 - (y3 - y1) / 6;
  }

  /**
     * One loop of its own: without a JIT, each call would cost more than its work.
     * @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale @param {number} x0 @param {number} x1
     */
  #plain(time, scale, x0, x1) {
    const values = this.values, n = values.length, levels = this.summary ? this.summary.levels : null, log = scale.mode === "log";
    // (Sums in the same order as Timeline.index: the same columns to the bit.)
    const from = time.from, span = time.to - time.from, width = time.width;
    // (y from the scale's low end, so no large terms cancel.)
    const ta = log ? Math.log10(scale.lo) : scale.lo, tb = log ? Math.log10(scale.hi) : scale.hi, top = scale.top * scale.height + scale.above, room = scale.height - top - scale.bottom * scale.height - scale.below;
    const k = room / (tb - ta), bottom = top + room, m = scale.m, c = scale.c, px = width / span;
    // (A slope too large to represent: the general column path, Scale.y's divide-first mapping.)
    if (!(k < Infinity && k > -Infinity)) return this.#columns(time, scale, x0, x1, null);
    const size = 2 ** Math.ceil(Math.log2(Math.max(64, width + 4)));
    if (!this.columns || this.columns.size < size) this.columns = new Columns(size);
    const columns = this.columns, mask = columns.size - 1;
    if (columns.version !== this.version || columns.log !== log) (columns.version = this.version), (columns.log = log), columns.now++;
    const now = columns.now, cA = columns.a, cB = columns.b, cStamp = columns.stamp, origin = Math.round(from * px);
    const first = Math.floor(x0) - 1, last = Math.ceil(x1) + 1;
    let start = Math.ceil(from + (first * span) / width - NOISE);
    const xs = XS, ys = YS, ids = IDS;
    let count = 0;
    for (let x = first; x <= last; x++) {
      const end = Math.ceil(from + ((x + 1) * span) / width - NOISE), a = start < 0 ? 0 : start, b = end - 1 < n - 1 ? end - 1 : n - 1;
      start = end;
      // (Only a column's first vertex carries its first point, for the dash ruler.)
      let id = a;
      const slot = (x + origin) & mask, hit = cStamp[slot] === now && cA[slot] === a && cB[slot] === b;
      let lo = hit ? columns.lo[slot] : Infinity, hi = hit ? columns.hi[slot] : -Infinity, loAt = hit ? columns.loAt[slot] : 0, hiAt = hit ? columns.hiAt[slot] : 0, i = hit ? b + 1 : a, clean = !hit;
      for (;;) {
        if (count >= TRAIL) pen.trail(xs, ys, ids, count), (count = 0);
        let gap = false;
        while (i <= b) {
          if (levels && !(i & ((1 << FIRST) - 1)) && b - i + 1 >= 1 << FIRST) {
            let size = Math.min(i ? i & -i : 1 << 30, 1 << (31 - Math.clz32(b - i + 1)));
            for (; size >= 1 << FIRST; size >>= 1) {
              const level = 31 - Math.clz32(size) - FIRST;
              if (level >= levels.length) continue;
              const at = levels[level], k = i >> (level + FIRST);
              if (k >= at.lo.length || (log ? at.low[k] : at.nan[k])) continue;
              if (at.lo[k] < lo) (lo = at.lo[k]), (loAt = at.loAt[k]);
              if (at.hi[k] > hi) (hi = at.hi[k]), (hiAt = at.hiAt[k]);
              break;
            }
            if (size >= 1 << FIRST) {
              i += size;
              continue;
            }
          }
          const v = values[i++];
          if (log ? v > 0 : v === v) {
            if (v < lo) (lo = v), (loAt = i - 1);
            if (v > hi) (hi = v), (hiAt = i - 1);
          } else {
            (gap = true), (clean = false);
            break;
          }
        }
        // (Low and high at their own points' x: a trend through a column draws as one slope, not stairs.)
        if (lo <= hi) {
          const loFirst = loAt <= hiAt, p = loFirst ? lo : hi, q = loFirst ? hi : lo;
          (xs[count] = ((loFirst ? loAt : hiAt) - from) * px), (ys[count] = bottom - k * ((log ? Math.log10(p) : p * m + c) - ta)), (ids[count++] = id);
          (xs[count] = ((loFirst ? hiAt : loAt) - from) * px), (ys[count] = bottom - k * ((log ? Math.log10(q) : q * m + c) - ta)), (ids[count++] = -1);
          id = -1;
          if (!gap && clean) (cStamp[slot] = now), (cA[slot] = a), (cB[slot] = b), (columns.lo[slot] = lo), (columns.hi[slot] = hi), (columns.loAt[slot] = loAt), (columns.hiAt[slot] = hiAt);
        }
        if (!gap) break;
        // (A lift after a lift does nothing.)
        if (!count || xs[count - 1] === xs[count - 1]) xs[count++] = NaN;
        (lo = Infinity), (hi = -Infinity);
      }
    }
    pen.trail(xs, ys, ids, count);
  }

  /**
   * Widens `out` ([lo, hi]); `positive`: the lowest above zero.
   * @param {number} a @param {number} b @param {boolean} positive @param {Float64Array} out
   */
  extent(a, b, positive, out) {
    const values = this.values, levels = this.summary ? this.summary.levels : null;
    a = Math.max(0, a);
    b = Math.min(values.length - 1, b);
    let lo = out[0], hi = out[1], i = a;
    while (i <= b) {
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
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr
   */
  draw(g, time, scale, x0, x1, dpr) {
    if (!this.values.length) return;
    if (this.stroke) this.#stroke(g, time, scale, x0, x1, dpr);
    this.drawDots(g, time, scale, x0, x1);
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr
   */
  #stroke(g, time, scale, x0, x1, dpr) {
    const zones = this.zones, gradient = zones ? null : this.gradient, colors = !zones && !gradient && this.colors && this.palette.length ? this.colors : null;
    // (Past the strip as far as the stroke reaches, so a strip draws what reaches in.)
    const reach = Math.ceil((this.width * (this.outline ? 3 : 1)) / 2) + 1;
    (x0 -= reach), (x1 += reach);
    // (Dashes from as far along as each subpath starts, so a strip matches its moved picture; a curve's are the canvas's.)
    const dash = this.dash, period = dash && !measuring && !(this.curve && !this.steps) ? periodOf(dash) : 0;
    const phase = period > 0 && time.x(0) - 1 <= x1 && time.x(this.values.length - 1) + 1 >= x0 ? this.#anchor(g, time, scale, x0, x1, dpr, colors) : 0;
    if (dash) g.setLineDash(dash);
    const held = time.spacing >= 1 ? this.#held(scale) : -1, partialDash = held < 0 ? SOLID : PARTIAL_DASH.map((part) => part * this.width);
    const tail = (/** @type {number} */ pass) => {
      if (held < 0) return;
      g.setLineDash(partialDash);
      g.beginPath();
      pen.start(g, dpr, pass, 0, partialDash, periodOf(partialDash));
      this.#trace(time, scale, x0, x1, colors, held);
      pen.lift();
      g.stroke();
      g.setLineDash(dash ?? SOLID);
    };
    // (Round joins, bevelled for an outline; passes as its strokes', so their dashes start alike.)
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
        tail(pass);
      }
    }
    g.lineWidth = this.width;
    // (Bevelled: as round at a line's width, half the cost to stroke in Firefox, and no miter spikes.)
    g.lineJoin = "bevel";
    // (Measured: an all-painted palette shares one pass, else a pass a color; a stroke a zone, clipped: what it paints is what's hit.)
    const spans = zones && measuring ? zoneSpans(zones, scale, scale.height, this.spans) : null;
    const passes = spans ? spans.length / 3 : colors && (!measuring || !this.palette.every(painted)) ? this.palette.length : 0;
    // (A color's pass traces every point in view: one for a color none has, skipped.)
    const used = colors && passes && !spans ? this.#used(time, x0, x1, colors) : null;
    for (let pass = passes ? 0 : -1; pass < passes; pass++) {
      if (used && !used[pass]) continue;
      const color = spans && zones ? ((k) => (k < 0 ? this.color : zones[k].color))(spans[pass * 3]) : colors && measuring && !passes ? "#000" : pass < 0 ? this.color : this.palette[pass];
      if (spans) g.save(), g.beginPath(), g.rect(x0, spans[pass * 3 + 1], x1 - x0, spans[pass * 3 + 2] - spans[pass * 3 + 1]), g.clip();
      g.strokeStyle = zones && !spans ? (this.strokeFill ??= new Fill()).zoned(g, color, zones, scale, scale.height) : gradient ? (this.strokeFill ??= new Fill()).graded(g, gradient, scale, scale.height) : color;
      g.beginPath();
      pen.start(g, dpr, spans ? -1 : pass, phase, dash ?? SOLID, period);
      this.#trace(time, scale, x0, x1, colors);
      pen.lift();
      g.stroke();
      tail(spans ? -1 : pass);
      if (spans) g.restore();
      if (period > 0 && pen.longest > DRIFT) this.drifts = true;
    }
    if (dash) g.setLineDash(SOLID), (g.lineDashOffset = 0);
    if (period > 0) (this.dashLast = pen.lastId), (this.dashLastAt = pen.last);
  }

  /**
   * @param {import("./timeline.js").Timeline} time @param {number} x0 @param {number} x1 @param {Uint8Array | Uint16Array} colors
   */
  #used(time, x0, x1, colors) {
    const used = this.used.length === this.palette.length ? this.used.fill(0) : (this.used = new Uint8Array(this.palette.length));
    const a = Math.max(0, Math.floor(time.index(Math.floor(x0) - 2)) - 1), b = Math.min(this.values.length - 1, Math.ceil(time.index(Math.ceil(x1) + 2)) + 1);
    for (let i = a; i <= b; i++) used[colors[i]] = 1;
    return used;
  }

  partialAt() {
    return this.partial ? this.last(false) : -1;
  }

  /** @param {import("./scale.js").Scale} scale */
  #held(scale) {
    const i = this.partialAt(), v = this.values;
    return i > 0 && (scale.mode === "log" ? v[i] > 0 && v[i - 1] > 0 : v[i - 1] === v[i - 1]) && !(this.breaks && this.breaks[i - 1] === 1) ? i : -1;
  }

  /**
     * Dash phase carried from the nearer end of its last draw, so dashes stay put as the view moves.
     * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
     * @param {number} x0 @param {number} x1 @param {number} dpr @param {Uint8Array | Uint16Array | null} colors
     */
  #anchor(g, time, scale, x0, x1, dpr, colors) {
    const from = time.spacing >= 1 ? Math.max(0, Math.floor(time.index(x0)) - 1) : Math.max(0, Math.ceil(time.index(Math.floor(x0) - 1) - NOISE));
    const a = this.dashFirst, b = this.dashLast, ax = a >= 0 ? time.x(a) : Infinity, bx = b >= 0 ? time.x(b) : Infinity, nearer = Math.abs(ax - x0) <= Math.abs(bx - x0);
    let origin = nearer ? a : b, ox = nearer ? ax : bx;
    if (!(Math.abs(ox - x0) <= 2 * time.width)) (origin = -1), (ox = x0);
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
   * Drawn again over the crosshair's lines too.
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1
   */
  drawDots(g, time, scale, x0, x1) {
    if (!this.dots || !this.values.length) return;
    const values = this.values, r = this.dots, ring = this.outline ? r + Math.max(1, this.width) : 0, reach = Math.max(r, ring);
    const a = Math.max(0, Math.floor(time.index(x0 - reach))), b = Math.min(values.length - 1, Math.ceil(time.index(x1 + reach)));
    const zones = this.zones, colors = zones ? null : this.pointColors(), palette = this.pointPalette();
    if (colors) byColor(colors, palette.length, a, b);
    for (let pass = ring ? -2 : colors || zones ? 0 : -1; pass < (colors ? palette.length : zones ? zones.length + 1 : 0); pass++) {
      if (pass === -1 && (colors || zones)) continue;
      g.fillStyle = pass === -2 ? /** @type {string} */ (this.outline) : zones ? (pass ? zones[pass - 1].color : this.color) : pass < 0 ? this.color : palette[pass];
      g.beginPath();
      const own = colors !== null && pass >= 0;
      for (let k = own ? STARTS[pass] : a, end = own ? STARTS[pass + 1] : b + 1; k < end; k++) {
        const i = own ? ORDER[k] : k, v = values[i];
        if (v !== v || (pass >= 0 && zones && zoneIndex(zones, v) + 1 !== pass)) continue;
        const x = time.x(i), y = scale.y(v), size = pass === -2 ? ring : r;
        g.moveTo(x + size, y);
        g.arc(x, y, size, 0, 2 * Math.PI);
      }
      g.fill();
    }
  }

  /**
   * @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale @param {number} x0 @param {number} x1
   * @param {Uint8Array | Uint16Array | null} colors @param {number} [only] its partial point: the segment to it alone;
   * none, all but it
   */
  #trace(time, scale, x0, x1, colors, only = -1) {
    const values = this.values, breaks = this.breaks, n = values.length, log = scale.mode === "log";
    if (time.spacing >= 1) {
      const held = only < 0 ? this.#held(scale) : -1, a = only < 0 ? Math.max(0, Math.floor(time.index(x0)) - 1) : only - 1, b = only < 0 ? Math.min(n - 1, Math.ceil(time.index(x1)) + 1) : only;
      for (let i = a; i <= b; i++) {
        const v = values[i];
        if (!(log ? v > 0 : v === v) || i === held) {
          pen.lift();
          continue;
        }
        const x = time.x(i), y = scale.y(v), color = colors ? colors[i] : 0, cut = breaks ? breaks[i] === 1 : false;
        if (this.steps && pen.has && !pen.cut) pen.to(x, pen.py, pen.color, false);
        pen.id = i;
        if (this.curve && !this.steps && pen.has && !pen.cut) this.#bend(i, time, scale, pen.px, pen.py, x, y), pen.curve(BEND[0], BEND[1], BEND[2], BEND[3], x, y, color, cut);
        else pen.to(x, y, color, cut);
        pen.id = -1;
      }
      return;
    }
    if (!colors && !breaks) return this.#plain(time, scale, x0, x1);
    this.#columns(time, scale, x0, x1, colors);
  }

  /** @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {Uint8Array | Uint16Array | null} colors */
  #columns(time, scale, x0, x1, colors) {
    const values = this.values, breaks = this.breaks, n = values.length, log = scale.mode === "log";
    // (Columns at whole pixels and points at whole indices: a whole-pixel pan holds the same columns.)
    for (let x = Math.floor(x0) - 1; x <= Math.ceil(x1) + 1; x++) {
      const a = Math.max(0, Math.ceil(time.index(x) - NOISE)), b = Math.min(n - 1, Math.ceil(time.index(x + 1) - NOISE) - 1);
      run.start();
      pen.id = a;
      for (let i = a; i <= b; i++) {
        const v = values[i];
        if (log ? v > 0 : v === v) {
          if (colors && run.first === run.first && colors[i] !== colors[run.firstAt]) run.end(time, scale, colors, breaks);
          run.add(v, i);
          if (breaks && breaks[i] === 1) run.end(time, scale, colors, breaks);
        } else run.end(time, scale, colors, breaks), pen.lift();
      }
      run.end(time, scale, colors, breaks);
    }
  }
}
