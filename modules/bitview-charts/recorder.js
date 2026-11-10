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
    if (this.penX !== this.penX) return;
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
 * The parts of a fill's edge (at `k` in fills) not retraced by its other edges the other way (where it encloses
 * nothing: a bar at its base, a band its edges together, a curved area flat on its base, a stepped stack's transitions
 * across the gap between bands): written to PIECES as from, to (0 to 1 along it), their count returned. Retraced: the
 * reversed edges on its line (within a millionth of a pixel) covering it there.
 * @param {Float64Array} fills @param {number} from @param {number} to @param {number} k
 */
function uncovered(fills, from, to, k) {
  const x0 = fills[k], y0 = fills[k + 1], dx = fills[k + 2] - x0, dy = fills[k + 3] - y0, length = Math.hypot(dx, dy);
  if (!(length > 1e-9)) return 0;
  const ux = dx / length, uy = dy / length;
  let n = 0;
  for (let j = from * 4; j < to * 4; j += 4) {
    if (j === k) continue;
    const ax = fills[j] - x0, ay = fills[j + 1] - y0, bx = fills[j + 2] - x0, by = fills[j + 3] - y0;
    // (On its line (both ends within a millionth of a pixel of it), the other way.)
    if (Math.abs(ax * uy - ay * ux) > 1e-6 || Math.abs(bx * uy - by * ux) > 1e-6) continue;
    const a = (ax * ux + ay * uy) / length, b = (bx * ux + by * uy) / length;
    if (!(b < a && b < 1 && a > 0)) continue;
    if (n + 2 > covers.length) covers = grown(covers);
    (covers[n++] = Math.max(0, b)), (covers[n++] = Math.min(1, a));
  }
  // (The covered stretches in order; what's between them its pieces.)
  for (let p = 2; p < n; p += 2)
    for (let q = p; q > 0 && covers[q] < covers[q - 2]; q -= 2) {
      const a = covers[q], b = covers[q + 1];
      (covers[q] = covers[q - 2]), (covers[q + 1] = covers[q - 1]), (covers[q - 2] = a), (covers[q - 1] = b);
    }
  while (pieces.length < n + 2) pieces = grown(pieces);
  let count = 0, at = 0;
  for (let p = 0; p < n; p += 2) {
    if (covers[p] > at + 1e-9) (pieces[count++] = at), (pieces[count++] = covers[p]);
    at = Math.max(at, covers[p + 1]);
  }
  if (at < 1 - 1e-9) (pieces[count++] = at), (pieces[count++] = 1);
  return count / 2;
}

/**
 * How far a point is from a piece of a fill's edge on its clip's border (`a` to `b` along it, `at` across it: a
 * horizontal border's y (`flat`), else a vertical one's x), its parts the fill paints just inside the clip (`inward`, the
 * way in) alone: a baseline's base, where its over color paints over it and not where its edge has crossed under. Its
 * paint there changes only where its other edges cross that line.
 * @param {Float64Array} fills @param {number} from @param {number} to @param {number} x @param {number} y @param {boolean} flat
 * @param {number} at @param {number} inward @param {number} a @param {number} b
 * @param {number} bx0 @param {number} by0 @param {number} bx1 @param {number} by1
 */
function bordering(fills, from, to, x, y, flat, at, inward, a, b, bx0, by0, bx1, by1) {
  const lo = Math.min(a, b), hi = Math.max(a, b), line = at + inward;
  let n = 0;
  splits[n++] = lo;
  for (let j = from * 4; j < to * 4; j += 4) {
    const p0 = flat ? fills[j + 1] : fills[j], p1 = flat ? fills[j + 3] : fills[j + 2];
    if (p0 > line === p1 > line) continue;
    const q0 = flat ? fills[j] : fills[j + 1], q = q0 + ((line - p0) * ((flat ? fills[j + 2] : fills[j + 3]) - q0)) / (p1 - p0);
    if (!(q > lo && q < hi)) continue;
    if (n + 2 > splits.length) splits = grown(splits);
    splits[n++] = q;
  }
  splits[n++] = hi;
  for (let p = 1; p < n; p++)
    for (let q = p; q > 0 && splits[q] < splits[q - 1]; q--) {
      const v = splits[q];
      (splits[q] = splits[q - 1]), (splits[q - 1] = v);
    }
  let best = Infinity;
  for (let p = 0; p + 1 < n; p++) {
    const s0 = splits[p], s1 = splits[p + 1], m = (s0 + s1) / 2;
    if (!(s1 - s0 > 1e-9) || winding(fills, from, to, flat ? m : line, flat ? line : m) === 0) continue;
    best = Math.min(best, flat ? clipped(x, y, s0, at, s1, at, bx0, by0, bx1, by1) : clipped(x, y, at, s0, at, s1, bx0, by0, bx1, by1));
  }
  return best;
}

/** An edge's covered stretches, its uncovered pieces (uncovered() writes them), a border piece's splits (bordering()): grown, never made again. */
let covers = new Float64Array(64), pieces = new Float64Array(64), splits = new Float64Array(64);

/**
 * How far a point is from a segment's part within a box (Liang and Barsky's clipping; none in it, or a point of it
 * alone (an edge touching the box: what it bounds is outside it): Infinity).
 * @param {number} x @param {number} y @param {number} x0 @param {number} y0 @param {number} x1 @param {number} y1
 * @param {number} bx0 @param {number} by0 @param {number} bx1 @param {number} by1
 */
function clipped(x, y, x0, y0, x1, y1, bx0, by0, bx1, by1) {
  const dx = x1 - x0, dy = y1 - y0;
  span[0] = 0;
  span[1] = 1;
  if (!within(-dx, x0 - bx0) || !within(dx, bx1 - x0) || !within(-dy, y0 - by0) || !within(dy, by1 - y0) || !(span[0] < span[1])) return Infinity;
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
    // it's painted just inside (not its outline past the box); else, as inside, to its edges within the box.)
    const qx = Math.min(Math.max(x, bx0), bx1), qy = Math.min(Math.max(y, by0), by1);
    if ((qx !== x || qy !== y) && winding(fills, from, to, qx + (x < bx0 ? 1e-3 : x > bx1 ? -1e-3 : 0), qy + (y < by0 ? 1e-3 : y > by1 ? -1e-3 : 0)) !== 0) {
      best = Math.min(best, Math.hypot(x - qx, y - qy));
      continue;
    }
    // (Its edges' parts it doesn't retrace (there it encloses nothing, paints nothing); on its clip's border, those
    // it paints within the clip.)
    for (let k = from * 4; k < to * 4; k += 4) {
      const x0 = fills[k], y0 = fills[k + 1], dx = fills[k + 2] - x0, dy = fills[k + 3] - y0, count = uncovered(fills, from, to, k);
      for (let p = 0; p < count * 2; p += 2) {
        const ax = x0 + dx * pieces[p], ay = y0 + dy * pieces[p], bx = x0 + dx * pieces[p + 1], by = y0 + dy * pieces[p + 1];
        const d =
          ay === by && (ay === by0 || ay === by1) ? bordering(fills, from, to, x, y, true, ay, ay === by0 ? 1e-3 : -1e-3, ax, bx, bx0, by0, bx1, by1)
          : ax === bx && (ax === bx0 || ax === bx1) ? bordering(fills, from, to, x, y, false, ax, ax === bx0 ? 1e-3 : -1e-3, ay, by, bx0, by0, bx1, by1)
          : clipped(x, y, ax, ay, bx, by, bx0, by0, bx1, by1);
        best = Math.min(best, d);
      }
    }
    inside ||= qx === x && qy === y && winding(fills, from, to, x, y) !== 0;
  }
  for (let k = 0; k < dotCount * 3; k += 3) best = Math.min(best, Math.max(0, Math.hypot(x - dots[k], y - dots[k + 1]) - dots[k + 2]));
  return inside ? 0 : best;
}
