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
  // (Where each fill's segments end in fills: each one inside or not on its own, as each was painted; and the box it
  // was clipped to then (x0, y0, x1, y1; none: all of it).)
  fillEnds: new Int32Array(64),
  fillClips: new Float64Array(256),
  fillsMade: 0,
  // (The clip as set (a box: the path's bounds when clip() was called), and those saved.)
  clipBox: [-Infinity, -Infinity, Infinity, Infinity],
  /** @type {number[][]} */
  saved: [],
  dots: new Float64Array(96),
  dotCount: 0,
  // (What a series sets on a canvas, kept and unread.)
  fillStyle: /** @type {any} */ (""),
  strokeStyle: /** @type {any} */ (""),
  lineWidth: 1,
  lineJoin: "",
  lineCap: "",
  lineDashOffset: 0,
  globalAlpha: 1,
  font: "",
  textAlign: "",
  textBaseline: "",
  reset() {
    (this.pathCount = 0), (this.strokeCount = 0), (this.fillCount = 0), (this.fillsMade = 0), (this.dotCount = 0), (this.penX = NaN);
    (this.clipBox = [-Infinity, -Infinity, Infinity, Infinity]), (this.saved.length = 0);
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
    // (Its radii as the canvas takes them: all scaled down together where two on a side would pass its length.)
    const all = typeof radii === "number" ? [radii] : radii, sx = Math.sign(w) || 1, sy = Math.sign(h) || 1, aw = Math.abs(w), ah = Math.abs(h);
    const at = (/** @type {number} */ k) => Math.max(0, all.length === 1 ? all[0] : all.length === 2 ? all[k % 2] : all.length === 3 ? all[[0, 1, 2, 1][k]] : all[k] ?? 0);
    const fit = (/** @type {number} */ side, /** @type {number} */ a, /** @type {number} */ b) => (a + b > side ? side / (a + b) : 1);
    const f = Math.min(fit(aw, at(0), at(1)), fit(aw, at(3), at(2)), fit(ah, at(0), at(3)), fit(ah, at(1), at(2)));
    const r = (/** @type {number} */ k) => at(k) * f;
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
    // (None for what the canvas wouldn't paint: a point a log scale can't show, a dot in a fully transparent color.)
    if (!(Number.isFinite(x) && Number.isFinite(y) && Number.isFinite(r)) || !painted(this.fillStyle)) return void (this.penX = NaN);
    if (this.dotCount * 3 + 3 > this.dots.length) this.dots = grown(this.dots);
    const k = this.dotCount++ * 3;
    (this.dots[k] = x), (this.dots[k + 1] = y), (this.dots[k + 2] = r);
    this.penX = NaN;
  },
  stroke() {
    // (Nothing where its paint shows nothing. Dashed, its whole path: a dashed line's hit along it, its gaps too, as
    // the chart library measured it: no one aims at a dash.)
    if (!painted(this.strokeStyle)) return;
    this.strokeCount = this.keep("strokes", this.strokeCount);
  },
  fill() {
    if (!painted(this.fillStyle)) return void (this.pathCount = 0);
    // (A fill closes each subpath: their segments back to their starts are in it already where it was closed (rects,
    // outlines, bodies always are).)
    this.fillCount = this.keep("fills", this.fillCount);
    if (this.fillsMade >= this.fillEnds.length) {
      const ends = new Int32Array(this.fillEnds.length * 2), clips = new Float64Array(this.fillClips.length * 2);
      ends.set(this.fillEnds), clips.set(this.fillClips);
      (this.fillEnds = ends), (this.fillClips = clips);
    }
    this.fillClips.set(this.clipBox, this.fillsMade * 4);
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
  save() {
    this.saved.push(this.clipBox);
  },
  restore() {
    this.clipBox = this.saved.pop() ?? [-Infinity, -Infinity, Infinity, Infinity];
  },
  /** (The path's bounds, within the clip as it was: a series clips to a box, a baseline's halves.) */
  clip() {
    let x0 = Infinity, y0 = Infinity, x1 = -Infinity, y1 = -Infinity;
    for (let k = 0; k < this.pathCount * 4; k += 2) (x0 = Math.min(x0, this.path[k])), (x1 = Math.max(x1, this.path[k])), (y0 = Math.min(y0, this.path[k + 1])), (y1 = Math.max(y1, this.path[k + 1]));
    const [a, b, c, d] = this.clipBox;
    this.clipBox = [Math.max(a, x0), Math.max(b, y0), Math.min(c, x1), Math.min(d, y1)];
  },
  setLineDash() {},
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
export function painted(style) {
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

/**
 * A fill's winding round a point as the canvas fills it, nonzero: its edges a ray to the right crosses, up one way,
 * down the other (none: outside it).
 * @param {Float64Array} fills @param {number} from @param {number} to @param {number} x @param {number} y
 */
function winding(fills, from, to, x, y) {
  let sum = 0;
  for (let k = from * 4; k < to * 4; k += 4) {
    const x0 = fills[k], y0 = fills[k + 1], x1 = fills[k + 2], y1 = fills[k + 3];
    if (y0 > y !== y1 > y && x < x0 + ((y - y0) * (x1 - x0)) / (y1 - y0)) sum += y1 > y0 ? 1 : -1;
  }
  return sum;
}

/**
 * Whether a fill's edge (at `k` in fills) is retraced by another of its edges the other way (a shape that encloses
 * nothing there: a bar at its base, a band its edges together), so bounds no paint.
 * @param {Float64Array} fills @param {number} from @param {number} to @param {number} k
 */
function retraced(fills, from, to, k) {
  const x0 = fills[k], y0 = fills[k + 1], x1 = fills[k + 2], y1 = fills[k + 3];
  if (x0 === x1 && y0 === y1) return true;
  for (let j = from * 4; j < to * 4; j += 4) if (j !== k && fills[j] === x1 && fills[j + 1] === y1 && fills[j + 2] === x0 && fills[j + 3] === y0) return true;
  return false;
}

/**
 * How far a point is from a segment's part within a box (Liang and Barsky's clipping; none in it: Infinity).
 * @param {number} x @param {number} y @param {number} x0 @param {number} y0 @param {number} x1 @param {number} y1
 * @param {number} bx0 @param {number} by0 @param {number} bx1 @param {number} by1
 */
function clipped(x, y, x0, y0, x1, y1, bx0, by0, bx1, by1) {
  const dx = x1 - x0, dy = y1 - y0;
  span[0] = 0;
  span[1] = 1;
  if (!within(-dx, x0 - bx0) || !within(dx, bx1 - x0) || !within(-dy, y0 - by0) || !within(dy, by1 - y0)) return Infinity;
  return toSegment(x, y, x0 + span[0] * dx, y0 + span[0] * dy, x0 + span[1] * dx, y0 + span[1] * dy);
}

/** The part of a segment kept by the box's sides so far (from, to: 0 to 1), as clipped() narrows it (made once). */
const span = new Float64Array(2);

/** One of the box's sides narrowing `span` (p, q as Liang and Barsky write them): whether any of it is left. @param {number} p @param {number} q */
function within(p, q) {
  if (p === 0) return q >= 0;
  const r = q / p;
  if (p < 0) span[0] = Math.max(span[0], r);
  else span[1] = Math.min(span[1], r);
  return span[0] <= span[1];
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
  measuring = true;
  try {
    series.draw(/** @type {any} */ (recorder), time, scale, x - reach, x + reach, 1);
  } finally {
    measuring = false;
  }
  let best = Infinity, inside = false;
  const { strokes, strokeCount, fills, fillEnds, fillClips, fillsMade, dots, dotCount } = recorder;
  for (let k = 0; k < strokeCount * 4; k += 4) best = Math.min(best, toSegment(x, y, strokes[k], strokes[k + 1], strokes[k + 2], strokes[k + 3]));
  for (let f = 0, from = 0; f < fillsMade; from = fillEnds[f++]) {
    const to = fillEnds[f], c = f * 4, bx0 = fillClips[c], by0 = fillClips[c + 1], bx1 = fillClips[c + 2], by1 = fillClips[c + 3];
    // (Clipped to nothing: nothing painted.)
    if (!(bx1 > bx0 && by1 > by0)) continue;
    // (Clipped to a box (a baseline's half): the point outside it, measured to the nearest point of the box where
    // that's painted; else, as inside, to its edges within the box.)
    const qx = Math.min(Math.max(x, bx0), bx1), qy = Math.min(Math.max(y, by0), by1);
    if ((qx !== x || qy !== y) && winding(fills, from, to, qx, qy) !== 0) {
      best = Math.min(best, Math.hypot(x - qx, y - qy));
      continue;
    }
    // (Its edges, but those it retraces: there it encloses nothing, paints nothing.)
    for (let k = from * 4; k < to * 4; k += 4) if (!retraced(fills, from, to, k)) best = Math.min(best, clipped(x, y, fills[k], fills[k + 1], fills[k + 2], fills[k + 3], bx0, by0, bx1, by1));
    inside ||= qx === x && qy === y && winding(fills, from, to, x, y) !== 0;
  }
  for (let k = 0; k < dotCount * 3; k += 3) best = Math.min(best, Math.max(0, Math.hypot(x - dots[k], y - dots[k + 1]) - dots[k + 2]));
  return inside ? 0 : best;
}
