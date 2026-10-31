// @ts-check
// A canvas stand-in that keeps what a series draws instead of painting it: its strokes' center lines, its fills'
// outlines and its dots, as segments and circles, so a point can be measured against the drawing itself (a hit: the
// same geometry as the picture, whatever the zoom, gaps, joints, steps or curves). One for every series, its buffers
// grown, never made again.

/** A curve's segments, as kept. */
const CURVE = 8;

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
  globalAlpha: 1,
  font: "",
  textAlign: "",
  textBaseline: "",
  reset() {
    (this.pathCount = 0), (this.strokeCount = 0), (this.fillCount = 0), (this.fillsMade = 0), (this.dotCount = 0), (this.penX = NaN);
  },
  /** @param {number} x0 @param {number} y0 @param {number} x1 @param {number} y1 */
  segment(x0, y0, x1, y1) {
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
  /** (Its corners as square: a hit's measure within them.) @param {number} x @param {number} y @param {number} w @param {number} h */
  roundRect(x, y, w, h) {
    this.rect(Math.min(x, x + w), y, Math.abs(w), h);
  },
  /** (A dot: kept as a circle.) @param {number} x @param {number} y @param {number} r */
  arc(x, y, r) {
    if (this.dotCount * 3 + 3 > this.dots.length) this.dots = grown(this.dots);
    const k = this.dotCount++ * 3;
    (this.dots[k] = x), (this.dots[k + 1] = y), (this.dots[k + 2] = r);
    this.penX = NaN;
  },
  stroke() {
    this.strokeCount = this.keep("strokes", this.strokeCount);
  },
  fill() {
    // (A fill closes each subpath: their segments back to their starts are in it already where it was closed; an
    // outline (a band's) always is.)
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
  setLineDash() {},
  setTransform() {},
  /** (A gradient's place: its stops unread.) */
  createLinearGradient() {
    return { addColorStop() {} };
  },
};

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
  series.draw(/** @type {any} */ (recorder), time, scale, x - reach, x + reach, 1);
  let best = Infinity, inside = false;
  const { strokes, strokeCount, fills, fillEnds, fillsMade, dots, dotCount } = recorder;
  for (let k = 0; k < strokeCount * 4; k += 4) best = Math.min(best, toSegment(x, y, strokes[k], strokes[k + 1], strokes[k + 2], strokes[k + 3]));
  for (let f = 0, from = 0; f < fillsMade; from = fillEnds[f++]) {
    // (Inside a fill: an odd number of its edges crossed by a ray to the right.)
    let odd = false;
    for (let k = from * 4; k < fillEnds[f] * 4; k += 4) {
      const x0 = fills[k], y0 = fills[k + 1], x1 = fills[k + 2], y1 = fills[k + 3];
      best = Math.min(best, toSegment(x, y, x0, y0, x1, y1));
      if (y0 > y !== y1 > y && x < x0 + ((y - y0) * (x1 - x0)) / (y1 - y0)) odd = !odd;
    }
    inside ||= odd;
  }
  for (let k = 0; k < dotCount * 3; k += 3) best = Math.min(best, Math.max(0, Math.hypot(x - dots[k], y - dots[k + 1]) - dots[k + 2]));
  return inside ? 0 : best;
}
