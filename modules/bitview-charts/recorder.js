// @ts-check
// A canvas stand-in that keeps what a series draws instead of painting it: its strokes' center lines, its fills'
// outlines and its dots, as segments and circles, so a point can be measured against the drawing itself (a hit: the
// same geometry as the picture, whatever the zoom, gaps, joints, steps or curves). One for every series, its buffers
// grown, never made again.

/** A curve's segments, as kept. */
const CURVE = 8;
/** A real canvas' context, for the gradients a series makes (a fill keeps them for its real draws too). @type {CanvasRenderingContext2D | null} */
let real = null;
/** Whether a series is being drawn into the recorder (its geometry wanted: a line's colors in one pass, as their union). */
export let measuring = false;

const recorder = {
  // (The path being written: its segments (x0, y0, x1, y1), each subpath's start; the strokes' and fills' kept.)
  path: new Float64Array(1024),
  pathCount: 0,
  startX: NaN,
  startY: NaN,
  penX: NaN,
  penY: NaN,
  strokes: new Float64Array(1024),
  strokeCount: 0,
  fills: new Float64Array(1024),
  fillCount: 0,
  // (Where each fill's segments end in fills: each one inside or not on its own, as each was painted.)
  fillEnds: new Int32Array(64),
  fillsMade: 0,
  dots: new Float64Array(96),
  dotCount: 0,
  // (What a series sets on a canvas, kept and unread.)
  fillStyle: /** @type {any} */ (""),
  strokeStyle: /** @type {any} */ (""),
  lineWidth: 1,
  lineJoin: "",
  lineCap: "",
  lineDashOffset: 0,
  /** Its strokes' dashes, as set (none: solid). @type {number[]} */
  dash: [],
  globalAlpha: 1,
  font: "",
  textAlign: "",
  textBaseline: "",
  reset() {
    (this.pathCount = 0), (this.strokeCount = 0), (this.fillCount = 0), (this.fillsMade = 0), (this.dotCount = 0), (this.penX = NaN);
  },
  /** @param {number} x0 @param {number} y0 @param {number} x1 @param {number} y1 */
  segment(x0, y0, x1, y1) {
    // (Nothing for what the canvas wouldn't paint: a point a log scale can't show.)
    if (!(Number.isFinite(x0) && Number.isFinite(y0) && Number.isFinite(x1) && Number.isFinite(y1))) return;
    if (this.pathCount * 4 + 4 > this.path.length) this.path = grown(this.path);
    const k = this.pathCount++ * 4;
    (this.path[k] = x0), (this.path[k + 1] = y0), (this.path[k + 2] = x1), (this.path[k + 3] = y1);
  },
  beginPath() {
    (this.pathCount = 0), (this.penX = NaN);
  },
  /** @param {number} x @param {number} y */
  moveTo(x, y) {
    this.closeSub(false);
    (this.startX = this.penX = x), (this.startY = this.penY = y);
  },
  /** @param {number} x @param {number} y */
  lineTo(x, y) {
    if (this.penX !== this.penX) return this.moveTo(x, y);
    this.segment(this.penX, this.penY, x, y);
    (this.penX = x), (this.penY = y);
  },
  /** @param {number} c1x @param {number} c1y @param {number} c2x @param {number} c2y @param {number} x @param {number} y */
  bezierCurveTo(c1x, c1y, c2x, c2y, x, y) {
    if (this.penX !== this.penX) this.moveTo(c1x, c1y);
    const ax = this.penX, ay = this.penY;
    for (let k = 1; k <= CURVE; k++) {
      const s = k / CURVE, r = 1 - s;
      this.lineTo(r * r * r * ax + 3 * r * r * s * c1x + 3 * r * s * s * c2x + s * s * s * x, r * r * r * ay + 3 * r * r * s * c1y + 3 * r * s * s * c2y + s * s * s * y);
    }
  },
  /** Its subpath closed: back to its start (`force`: as closePath; else only where a fill would). @param {boolean} force */
  closeSub(force) {
    if (force && this.penX === this.penX && (this.penX !== this.startX || this.penY !== this.startY)) this.segment(this.penX, this.penY, this.startX, this.startY);
  },
  closePath() {
    this.closeSub(true);
    (this.penX = this.startX), (this.penY = this.startY);
  },
  /** @param {number} x @param {number} y @param {number} w @param {number} h */
  rect(x, y, w, h) {
    this.moveTo(x, y), this.lineTo(x + w, y), this.lineTo(x + w, y + h), this.lineTo(x, y + h), this.closePath();
  },
  /**
   * Its corners rounded as drawn (a radius, or one each: top left, top right, bottom right, bottom left), each a few
   * segments; wound as drawn (a negative width, the other way: a hole).
   * @param {number} x @param {number} y @param {number} w @param {number} h @param {number | number[]} [radii]
   */
  roundRect(x, y, w, h, radii = 0) {
    const all = typeof radii === "number" ? [radii] : radii, sx = Math.sign(w) || 1, sy = Math.sign(h) || 1;
    const r = (/** @type {number} */ k) => Math.max(0, Math.min(all[k] ?? all[0] ?? 0, Math.abs(w) / 2, Math.abs(h) / 2));
    // (Round the box from its top left, each corner a quarter turn about its center: top right, bottom right, bottom
    // left, top left, in the box's own directions.)
    const cx = [x + w - sx * r(1), x + w - sx * r(2), x + sx * r(3), x + sx * r(0)], cy = [y + sy * r(1), y + h - sy * r(2), y + h - sy * r(3), y + sy * r(0)];
    const ux = [0, 1, 0, -1], uy = [-1, 0, 1, 0];
    this.moveTo(x + sx * r(0), y);
    for (let k = 0; k < 4; k++) {
      const radius = r((k + 1) % 4);
      for (let s = 0; s <= 4; s++) {
        const a = (s / 4) * (Math.PI / 2), c = Math.cos(a), n = Math.sin(a);
        // (From the side's normal outward (ux, uy) turning to the next side's, k + 1.)
        const vx = ux[k] * c + ux[(k + 1) % 4] * n, vy = uy[k] * c + uy[(k + 1) % 4] * n;
        this.lineTo(cx[k] + vx * radius * sx, cy[k] + vy * radius * sy);
      }
    }
    this.closePath();
  },
  /** (A dot: kept as a circle.) @param {number} x @param {number} y @param {number} r */
  arc(x, y, r) {
    if (!(Number.isFinite(x) && Number.isFinite(y) && Number.isFinite(r))) return void (this.penX = NaN);
    if (this.dotCount * 3 + 3 > this.dots.length) this.dots = grown(this.dots);
    const k = this.dotCount++ * 3;
    (this.dots[k] = x), (this.dots[k + 1] = y), (this.dots[k + 2] = r);
    this.penX = NaN;
  },
  stroke() {
    // (Nothing where its paint shows nothing; dashed, its dashes alone.)
    if (!painted(this.strokeStyle)) return;
    if (this.dash.length) this.dashed();
    this.strokeCount = this.keep("strokes", this.strokeCount);
  },
  /**
   * The path cut to its dashes, as the canvas strokes it: the pattern from each subpath's start (a segment not
   * beginning where the last one ended starts one), shifted by lineDashOffset.
   */
  dashed() {
    const path = this.path, count = this.pathCount, dash = this.dash, period = dash.reduce((sum, d) => sum + d, 0);
    if (!(period > 0)) return;
    let n = 0, along = 0, endX = NaN, endY = NaN;
    const kept = this.cut.length >= count * 8 + 64 ? this.cut : (this.cut = new Float64Array(count * 16 + 64));
    for (let k = 0; k < count * 4; k += 4) {
      const x0 = path[k], y0 = path[k + 1], x1 = path[k + 2], y1 = path[k + 3], length = Math.hypot(x1 - x0, y1 - y0);
      if (x0 !== endX || y0 !== endY) along = ((this.lineDashOffset % period) + period) % period;
      // (Through the pattern along this segment, each "on" stretch kept.)
      let at = 0;
      while (at < length && n + 4 <= kept.length) {
        let phase = along % period, d = 0;
        while (phase >= dash[d]) (phase -= dash[d]), (d = (d + 1) % dash.length);
        const step = Math.min(dash[d] - phase, length - at);
        if (d % 2 === 0) {
          const t0 = at / length, t1 = (at + step) / length;
          (kept[n++] = x0 + (x1 - x0) * t0), (kept[n++] = y0 + (y1 - y0) * t0), (kept[n++] = x0 + (x1 - x0) * t1), (kept[n++] = y0 + (y1 - y0) * t1);
        }
        (at += step), (along += step);
      }
      (endX = x1), (endY = y1);
    }
    if (n > path.length) this.path = new Float64Array(n * 2);
    this.path.set(kept.subarray(0, n));
    this.pathCount = n / 4;
  },
  /** Its dashed path, as cut (grown, never made again). */
  cut: new Float64Array(0),
  fill() {
    if (!painted(this.fillStyle)) return void (this.pathCount = 0);
    // (A fill closes each subpath: their segments back to their starts are in it already where it was closed (rects,
    // outlines, bodies always are).)
    this.fillCount = this.keep("fills", this.fillCount);
    if (this.fillsMade >= this.fillEnds.length) {
      const next = new Int32Array(this.fillEnds.length * 2);
      next.set(this.fillEnds);
      this.fillEnds = next;
    }
    this.fillEnds[this.fillsMade++] = this.fillCount;
  },
  /** The path's segments added to `into` (its count `count`), the new count returned. @param {"strokes" | "fills"} into @param {number} count */
  keep(into, count) {
    let buffer = this[into];
    while ((count + this.pathCount) * 4 > buffer.length) buffer = grown(buffer);
    buffer.set(this.path.subarray(0, this.pathCount * 4), count * 4);
    this[into] = buffer;
    return count + this.pathCount;
  },
  save() {},
  restore() {},
  clip() {},
  /** (Kept as the canvas keeps it: an odd count, twice over.) @param {number[]} dash */
  setLineDash(dash) {
    this.dash = dash.length % 2 ? [...dash, ...dash] : dash;
  },
  setTransform() {},
  /** A real gradient (a fill keeps it for its real draws), its stops unread here. @param {number} x0 @param {number} y0 @param {number} x1 @param {number} y1 */
  createLinearGradient(x0, y0, x1, y1) {
    real ??= /** @type {CanvasRenderingContext2D} */ (document.createElement("canvas").getContext("2d"));
    return real.createLinearGradient(x0, y0, x1, y1);
  },
};

/**
 * Whether a style paints something (a gradient: yes; a color: not fully transparent), each color read once.
 * @param {any} style
 */
function painted(style) {
  if (typeof style !== "string") return true;
  let seen = paints.get(style);
  if (seen === undefined) {
    real ??= /** @type {CanvasRenderingContext2D} */ (document.createElement("canvas").getContext("2d"));
    real.fillStyle = "#000";
    real.fillStyle = style;
    const read = String(real.fillStyle);
    seen = !(/,\s*0\)$/.test(read) || /\/\s*0\)$/.test(read));
    if (paints.size > 512) paints.clear();
    paints.set(style, seen);
  }
  return seen;
}
/** Each color's painted() (see it). @type {Map<string, boolean>} */
const paints = new Map();

/** @param {Float64Array} buffer */
function grown(buffer) {
  const next = new Float64Array(buffer.length * 2);
  next.set(buffer);
  return next;
}

/** How far a point is from a segment. @param {number} x @param {number} y @param {number} x0 @param {number} y0 @param {number} x1 @param {number} y1 */
function toSegment(x, y, x0, y0, x1, y1) {
  const dx = x1 - x0, dy = y1 - y0, length = dx * dx + dy * dy, t = length ? Math.min(1, Math.max(0, ((x - x0) * dx + (y - y0) * dy) / length)) : 0;
  return Math.hypot(x - x0 - t * dx, y - y0 - t * dy);
}

/**
 * How far a point (`x` into the plot, `y` in its pane: CSS pixels) is from what `series` draws there, drawn into the
 * recorder over `reach` pixels each side of it: 0 inside a fill or a dot, else the distance to the nearest stroke, fill
 * edge or dot (Infinity: nothing drawn there).
 * @param {{ draw(g: CanvasRenderingContext2D, time: import("./timeline.js").Timeline, scale: import("./scale.js").Scale, x0: number, x1: number, dpr: number): void }} series
 * @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale @param {number} x @param {number} y @param {number} [reach]
 */
export function measure(series, time, scale, x, y, reach = 8) {
  recorder.reset();
  recorder.dash = [];
  measuring = true;
  try {
    series.draw(/** @type {any} */ (recorder), time, scale, x - reach, x + reach, 1);
  } finally {
    measuring = false;
  }
  let best = Infinity, inside = false;
  const { strokes, strokeCount, fills, fillEnds, fillsMade, dots, dotCount } = recorder;
  for (let k = 0; k < strokeCount * 4; k += 4) best = Math.min(best, toSegment(x, y, strokes[k], strokes[k + 1], strokes[k + 2], strokes[k + 3]));
  for (let f = 0, from = 0; f < fillsMade; from = fillEnds[f++]) {
    // (Inside a fill as the canvas fills it, nonzero: its edges' windings round the point (those a ray to the right
    // crosses, up one way, down the other) not summing to none.)
    let winding = 0;
    for (let k = from * 4; k < fillEnds[f] * 4; k += 4) {
      const x0 = fills[k], y0 = fills[k + 1], x1 = fills[k + 2], y1 = fills[k + 3];
      best = Math.min(best, toSegment(x, y, x0, y0, x1, y1));
      if (y0 > y !== y1 > y && x < x0 + ((y - y0) * (x1 - x0)) / (y1 - y0)) winding += y1 > y0 ? 1 : -1;
    }
    inside ||= winding !== 0;
  }
  for (let k = 0; k < dotCount * 3; k += 3) best = Math.min(best, Math.max(0, Math.hypot(x - dots[k], y - dots[k + 1]) - dots[k + 2]));
  return inside ? 0 : best;
}
