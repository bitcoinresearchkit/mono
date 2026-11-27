// @ts-check
import { Fill, zoneSpans } from "./color.js";
import { NOISE, firstFrom } from "./timeline.js";
import { measure } from "./recorder.js";
import { TextureFill } from "./texture.js";

const FIT = 64;
// Zoomed out, each column takes each edge's last value in it: bands sharing an edge meet exactly at any zoom.

/** Grown, never made again: one for every band, drawn one after another. */
const outline = {
  g: /** @type {CanvasRenderingContext2D | null} */ (null),
  xs: new Float64Array(0),
  ups: new Float64Array(0),
  downs: new Float64Array(0),
  bends: new Float64Array(0),
  bent: new Uint8Array(0),
  count: 0,
  /** @param {CanvasRenderingContext2D} g @param {number} room */
  start(g, room) {
    if (this.xs.length < room) (this.xs = new Float64Array(room * 2)), (this.ups = new Float64Array(room * 2)), (this.downs = new Float64Array(room * 2)), (this.bends = new Float64Array(room * 16)), (this.bent = new Uint8Array(room * 2));
    this.g = g;
    this.count = 0;
  },
  /** @param {number} x @param {number} up @param {number} down */
  add(x, up, down) {
    (this.xs[this.count] = x), (this.ups[this.count] = up), (this.downs[this.count] = down), (this.bent[this.count] = 0);
    this.count++;
  },
  close() {
    const g = /** @type {CanvasRenderingContext2D} */ (this.g), { xs, ups, downs, bends, bent, count } = this;
    if (count > 1) {
      g.moveTo(xs[0], ups[0]);
      for (let k = 1; k < count; k++) bent[k] & 1 ? g.bezierCurveTo(bends[8 * k], bends[8 * k + 1], bends[8 * k + 2], bends[8 * k + 3], xs[k], ups[k]) : g.lineTo(xs[k], ups[k]);
      g.lineTo(xs[count - 1], downs[count - 1]);
      for (let k = count - 1; k > 0; k--) bent[k] & 2 ? g.bezierCurveTo(bends[8 * k + 6], bends[8 * k + 7], bends[8 * k + 4], bends[8 * k + 5], xs[k - 1], downs[k - 1]) : g.lineTo(xs[k - 1], downs[k - 1]);
      g.closePath();
    }
    this.count = 0;
  },
};

/** @typedef {{ value: number, color: string, fade?: string | null }} BandZone */
/** @typedef {import("./texture.js").Texture} Texture */
/** @typedef {{ color: string, fade?: string | null, texture?: Texture | null }} BandFill */

export class Band {
  /**
   * Its fades are gradients fixed to the pane, so a pan moves them with its picture. A textured fill doesn't fade, and a
   * baseline, zones and a gradient take no texture.
   * @param {{ lower?: Float64Array, upper: Float64Array, base?: number, color?: string, fade?: string | null, below?: string | null, belowFade?: string | null, to?: "base" | "top" | "foot", steps?: boolean, curve?: boolean, colors?: Uint8Array | Uint16Array | null, palette?: BandFill[], zones?: readonly BandZone[] | null, gradient?: import("./color.js").Gradient | null, gradientFade?: import("./color.js").Gradient | null, texture?: Texture | null, hold?: boolean }} options
   */
  constructor({ lower, upper, base = 0, color = "rgba(247, 248, 248, 0.2)", fade = null, below = null, belowFade = null, to = "base", steps = false, curve = false, colors = null, palette = [], zones = null, gradient = null, gradientFade = null, texture = null, hold = false }) {
    /** @type {Float64Array} */
    this.lower = lower ?? new Float64Array(upper.length).fill(base);
    this.base = base;
    this.area = !lower;
    this.below = below;
    this.fade = fade;
    this.belowFade = belowFade;
    // (Its gradients, made again only as their colors or ends change.)
    this.overFill = new Fill();
    this.underFill = new Fill();
    this.to = to;
    this.steps = steps;
    this.curve = curve;
    this.colors = colors;
    this.palette = palette;
    /** @type {Fill[]} */
    this.fills = [];
    /** @type {readonly BandZone[] | null} */
    this.zones = zones && zones.length ? zones : null;
    // (Its zones' fills, its own first, and where they lie, a draw each.)
    /** @type {Fill[]} */
    this.zoneFills = [];
    /** @type {number[]} */
    this.spans = [];
    (this.gradient = gradient), (this.gradientFade = gradientFade);
    // (Its gradient's fill, made again only as it, its fade or its scale change.)
    this.gradedFill = new Fill();
    /** @type {Texture | null} */
    this.texture = texture;
    this.textureFill = new TextureFill();
    /** @type {TextureFill[]} */
    this.textureFills = [];
    this.hold = hold;
    /** @type {((a: number, b: number, positive: boolean, out: Float64Array) => void) | null} */
    this.fit = null;
    /** @type {Float64Array} */
    this.upper = upper;
    this.color = color;
    /** Its pane drawn whole after a change: its chart's invalidate(true). */
    this.hidden = false;
    this.faint = false;
    /** @type {import("./scale.js").Scale | null} */
    this.overlay = null;
    this.version = 0;
    // (Each point's next gap, made at its first draw of a version: a pixel column of points with no gap reads its last alone.)
    /** @type {Int32Array | null} */
    this.gaps = null;
    /** @type {Int32Array | null} */
    this.logGaps = null;
    this.gapsVersion = -1;
    // (Its edges' lines for fitting, any scale's and a log one's, and the version they're of.)
    /** @type {[{ lows: Float64Array, highs: Float64Array, low: Float64Array, high: Float64Array } | null, { lows: Float64Array, highs: Float64Array, low: Float64Array, high: Float64Array } | null]} */
    this.fits = [null, null];
    this.fitVersion = -1;
  }

  /** @param {number} i */
  first(i) {
    return firstFrom(this.upper, i);
  }

  /**
     * Measured against its drawing there.
     * @param {number} _i @param {number} y @param {import("./scale.js").Scale} scale @param {number} x @param {import("./timeline.js").Timeline} time
     */
  distance(_i, y, scale, x, time) {
    return measure(this, time, scale, x, y);
  }

  /**
   * @param {number} x @param {number} up @param {number} down @param {boolean} mine
   */
  #put(x, up, down, mine) {
    if (!mine && !outline.count) return;
    outline.add(x, up, down);
    if (!mine) outline.close();
  }

  /**
     * A sixth of the way along its neighbors' chord, by index, so a strip's and a whole draw's alike.
     * @param {Float64Array} edge @param {number} i @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
     * @param {boolean} log @param {boolean} toEdge @param {number} x1 @param {number} y1 @param {number} x2 @param {number} y2 @param {number} out
     */
  #bend(edge, i, time, scale, log, toEdge, x1, y1, x2, y2, out) {
    const lows = toEdge ? this.upper : this.lower, highs = this.upper, n = Math.min(lows.length, highs.length), w = i - 2, u = i + 1, b = outline.bends;
    const before = w >= 0 && (log ? lows[w] > 0 && highs[w] > 0 : lows[w] === lows[w] && highs[w] === highs[w]);
    const after = u < n && (log ? lows[u] > 0 && highs[u] > 0 : lows[u] === lows[u] && highs[u] === highs[u]);
    const x0 = before ? time.x(i - 2) : x1, y0 = before ? scale.y(edge[i - 2]) : y1, x3 = after ? time.x(i + 1) : x2, y3 = after ? scale.y(edge[i + 1]) : y2;
    // (Written out, not a shared helper: without a JIT its call costs more than the segment.)
    b[out] = x1 + (x2 - x0) / 6;
    b[out + 1] = y1 + (y2 - y0) / 6;
    b[out + 2] = x2 - (x3 - x1) / 6;
    b[out + 3] = y2 - (y3 - y1) / 6;
  }

  /** @param {boolean} log */
  #gaps(log) {
    if (this.gapsVersion !== this.version) (this.gaps = this.logGaps = null), (this.gapsVersion = this.version);
    // (An area to an edge of the pane: its values alone count; its log gaps those, kept as the log ones, its version theirs.)
    const kept = log ? this.logGaps : this.gaps, edge = this.#edge(log);
    if (kept) return kept;
    const lower = this.lower, upper = this.upper, n = Math.min(lower.length, upper.length), next = new Int32Array(n + 1);
    next[n] = n;
    for (let i = n - 1; i >= 0; i--) {
      const lo = edge ? upper[i] : lower[i], hi = upper[i];
      next[i] = (log ? lo > 0 && hi > 0 : lo === lo && hi === hi) ? next[i + 1] : i;
    }
    return log ? (this.logGaps = next) : (this.gaps = next);
  }

  /**
   * `lower: null`, an area again.
   * @param {{ lower?: Float64Array | null, upper?: Float64Array, color?: string, fade?: string | null, below?: string | null, belowFade?: string | null, base?: number, to?: "base" | "top" | "foot", steps?: boolean, curve?: boolean, colors?: Uint8Array | Uint16Array | null, palette?: BandFill[], zones?: readonly BandZone[] | null, gradient?: import("./color.js").Gradient | null, gradientFade?: import("./color.js").Gradient | null, texture?: Texture | null }} change
   */
  set({ lower, upper = this.upper, color = this.color, fade = this.fade, below = this.below, belowFade = this.belowFade, base = this.base, to = this.to, steps = this.steps, curve = this.curve, colors = this.colors, palette = this.palette, zones = this.zones, gradient = this.gradient, gradientFade = this.gradientFade, texture = this.texture }) {
    this.version++;
    const wasArea = this.area;
    if (lower === null) this.area = true;
    else if (lower) this.area = false;
    const kept = wasArea && this.lower.length === upper.length && base === this.base;
    this.base = base;
    this.lower = lower ?? (this.area ? (kept ? this.lower : new Float64Array(upper.length).fill(base)) : this.lower);
    (this.upper = upper), (this.color = color), (this.below = below), (this.fade = fade), (this.belowFade = belowFade), (this.to = to), (this.steps = steps), (this.curve = curve), (this.colors = colors), (this.palette = palette);
    this.zones = zones && zones.length ? zones : null;
    (this.gradient = gradient), (this.gradientFade = gradientFade), (this.texture = texture);
  }

  /**
   * On a log scale, an area whose base can't show is filled down to its foot too. @param {boolean} log
   */
  #edge(log) {
    return this.area && (this.to !== "base" || (log && !(this.base > 0)));
  }

  /**
   * An area's base not fitted to: its values alone, as an area's are read.
   * @param {number} a @param {number} b @param {boolean} positive @param {Float64Array} out
   */
  extent(a, b, positive, out) {
    const { lows, highs, low, high } = this.#fitted(positive), n = lows.length;
    a = Math.max(0, a);
    b = Math.min(n - 1, b);
    let lo = out[0], hi = out[1], i = a;
    while (i <= b) {
      if (i % FIT === 0 && i + FIT - 1 <= b) {
        const k = i / FIT;
        if (low[k] < lo) lo = low[k];
        if (high[k] > hi) hi = high[k];
        i += FIT;
        continue;
      }
      if (lows[i] < lo) lo = lows[i];
      if (highs[i] > hi) hi = highs[i];
      i++;
    }
    (out[0] = lo), (out[1] = hi);
  }

  /**
     * Read block by block, no scan of every point; set() tells it of edges changed in place.
     * @param {boolean} positive
     */
  #fitted(positive) {
    if (this.fitVersion !== this.version) (this.fitVersion = this.version), (this.fits = [null, null]);
    const k = positive ? 1 : 0, had = this.fits[k];
    if (had) return had;
    const upper = this.upper, lower = this.area ? upper : this.lower, n = Math.min(this.lower.length, upper.length), lows = new Float64Array(n), highs = new Float64Array(n), blocks = Math.ceil(n / FIT), low = new Float64Array(blocks), high = new Float64Array(blocks);
    // (Block by block: without a JIT, an index divided at each point costs more than the point's own work.)
    for (let k = 0, from = 0; k < blocks; k++, from += FIT) {
      let lo = Infinity, hi = -Infinity;
      for (let i = from, to = from + FIT < n ? from + FIT : n; i < to; i++) {
        const l = lower[i], u = upper[i];
        if (positive ? l > 0 && u > 0 : l === l && u === u) {
          const a = l < u ? l : u, b = l < u ? u : l;
          (lows[i] = a), (highs[i] = b);
          if (a < lo) lo = a;
          if (b > hi) hi = b;
        } else (lows[i] = NaN), (highs[i] = NaN);
      }
      (low[k] = lo), (high[k] = hi);
    }
    const made = { lows, highs, low, high };
    this.fits[k] = made;
    return made;
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr
   */
  draw(g, time, scale, x0, x1, dpr) {
    if (this.zones && !this.below) {
      // (By its zones: each zone's part clipped, fading over the pane as a whole.)
      const zones = this.zones, spans = zoneSpans(zones, scale, scale.height, this.spans);
      for (let s = 0; s < spans.length; s += 3) {
        const k = spans[s], top = spans[s + 1], foot = spans[s + 2], color = k < 0 ? this.color : zones[k].color, fade = k < 0 ? this.fade : zones[k].fade ?? null;
        g.save();
        g.beginPath();
        g.rect(x0, top, x1 - x0, foot - top);
        g.clip();
        this.#fill(g, time, scale, x0, x1, (this.zoneFills[k + 1] ??= new Fill()).style(g, color, fade, 0, scale.height), -1);
        g.restore();
      }
      return;
    }
    if (this.gradient && !this.below) return this.#fill(g, time, scale, x0, x1, this.gradedFill.graded(g, this.gradient, scale, scale.height, this.gradientFade), -1);
    if (!this.below) {
      const palette = this.colors ? this.palette : [];
      for (let k = 0; k < palette.length; k++) {
        const { color, fade, texture } = palette[k];
        this.#fill(g, time, scale, x0, x1, texture ? (this.textureFills[k] ??= new TextureFill()).style(g, texture, color, dpr, time, scale) : (this.fills[k] ??= new Fill()).style(g, color, fade ?? null, 0, scale.height), k);
      }
      if (!palette.length) this.#fill(g, time, scale, x0, x1, this.texture ? this.textureFill.style(g, this.texture, this.color, dpr, time, scale) : this.overFill.style(g, this.color, this.fade, 0, scale.height), -1);
      return;
    }
    // (A baseline: one outline filled twice, clipped at its base (the foot where a log scale can't show it).)
    let at = scale.y(this.base);
    at = at < 0 ? 0 : at < scale.height ? at : scale.height;
    const over = this.overFill.style(g, this.color, this.fade, 0, at), under = this.underFill.style(g, this.below, this.belowFade, at, scale.height);
    g.save();
    g.beginPath();
    g.rect(x0, 0, x1 - x0, at);
    g.clip();
    this.#fill(g, time, scale, x0, x1, over, -1);
    g.restore();
    g.save();
    g.beginPath();
    g.rect(x0, at, x1 - x0, scale.height - at);
    g.clip();
    this.#fill(g, time, scale, x0, x1, under, -1);
    g.restore();
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {string | CanvasGradient | CanvasPattern} color @param {number} pass
   */
  #fill(g, time, scale, x0, x1, color, pass) {
    const lower = this.lower, upper = this.upper, n = Math.min(lower.length, upper.length), log = scale.mode === "log", colors = pass >= 0 ? this.colors : null;
    if (!n) return;
    const per = 1 / time.spacing, zoomedIn = per <= 1, viewFrom = time.from, span = time.to - time.from, width = time.width, gaps = zoomedIn ? null : this.#gaps(log);
    // (Zoomed out, its y inline: without a JIT a call costs more than its work; points as Timeline.index has them.)
    const ta = log ? Math.log10(scale.lo) : scale.lo, tb = log ? Math.log10(scale.hi) : scale.hi, top = scale.top * scale.height + scale.above, room = scale.height - top - scale.bottom * scale.height - scale.below;
    const kk = room / (tb - ta), bottom = top + room, m = scale.m, c = scale.c, toEdge = this.#edge(log), edgeY = this.to === "top" ? 0 : scale.height;
    const fast = kk < Infinity && kk > -Infinity;
    const a = Math.max(0, Math.floor(time.index(x0)) - 1), b = Math.min(n - 1, Math.ceil(time.index(x1)) + 1);
    const from = zoomedIn ? a : Math.floor(x0) - 1, to = zoomedIn ? b : Math.ceil(x1) + 1, steps = zoomedIn && this.steps, curve = zoomedIn && this.curve && !steps;
    outline.start(g, (to - from + 2) * 2);
    g.fillStyle = color;
    g.beginPath();
    let px = 0, pu = 0, pd = 0, pc = -1, start = zoomedIn ? 0 : Math.ceil(viewFrom + (from * span) / width - NOISE);
    const xs = outline.xs, ups = outline.ups, downs = outline.downs, bent = outline.bent;
    for (let k = from; k <= to; k++) {
      if (zoomedIn) {
        const lo = toEdge ? upper[k] : lower[k], hi = upper[k];
        if (!(log ? lo > 0 && hi > 0 : lo === lo && hi === hi)) {
          outline.close();
          pc = -1;
          continue;
        }
        const x = time.x(k), up = scale.y(hi), down = toEdge ? edgeY : scale.y(lo), color = colors ? colors[k] : 0;
        // (Colored, its color changes where its edges cross between two colors' points.)
        const d0 = pu - pd, d1 = up - down, crossing = colors !== null && pc >= 0 && pc !== color && !steps && !curve && d0 * d1 < 0, last = pc;
        const t = crossing ? d0 / (d0 - d1) : 0, cx = px + t * (x - px), cy = pu + t * (up - pu);
        (px = x), (pu = up), (pd = down), (pc = color);
        if (crossing && last === pass) {
          outline.add(cx, cy, cy), outline.close();
          continue;
        }
        if (crossing && color === pass) outline.close(), outline.add(cx, cy, cy);
        const mine = !colors || color === pass;
        if (!mine && !outline.count) continue;
        const at = outline.count - 1;
        if (steps && at >= 0) outline.add(x, outline.ups[at], outline.downs[at]);
        outline.add(x, up, down);
        if (curve && at >= 0) {
          const ox = outline.xs[at];
          this.#bend(upper, k, time, scale, log, toEdge, ox, outline.ups[at], x, up, 8 * (at + 1));
          outline.bent[at + 1] = 1;
          if (!toEdge && !this.area) this.#bend(lower, k, time, scale, log, toEdge, ox, outline.downs[at], x, down, 8 * (at + 1) + 4), (outline.bent[at + 1] |= 2);
        }
        if (!mine) outline.close();
        continue;
      }
      // (A pixel column: its last point with both edges, the band broken at any point without in it.)
      // (Each column starts where the last ended; outline.add written out: without a JIT, a call a column costs more than its work.)
      const end = Math.ceil(viewFrom + ((k + 1) * span) / width - NOISE), first = start < 0 ? 0 : start, last = end - 1 < n - 1 ? end - 1 : n - 1;
      start = end;
      if (last < first) continue;
      if (fast && /** @type {Int32Array} */ (gaps)[first] > last) {
        const mine = !colors || colors[last] === pass;
        if (!mine && !outline.count) continue;
        const hi = upper[last], lo = lower[last], at = outline.count++;
        (xs[at] = ((last - viewFrom) * width) / span), (ups[at] = bottom - kk * ((log ? Math.log10(hi) : hi * m + c) - ta)), (downs[at] = toEdge ? edgeY : bottom - kk * ((log ? Math.log10(lo) : lo * m + c) - ta)), (bent[at] = 0);
        if (!mine) outline.close();
        continue;
      }
      let at = -1;
      for (let i = first; i <= last; i++) {
        const lo = toEdge ? upper[i] : lower[i], hi = upper[i];
        if (log ? lo > 0 && hi > 0 : lo === lo && hi === hi) at = i;
        else {
          if (at >= 0) this.#put(time.x(at), scale.y(upper[at]), toEdge ? edgeY : scale.y(lower[at]), !colors || colors[at] === pass);
          outline.close();
          at = -1;
        }
      }
      if (at >= 0) this.#put(time.x(at), scale.y(upper[at]), toEdge ? edgeY : scale.y(lower[at]), !colors || colors[at] === pass);
    }
    // Held to x1: from the strip's left where the last point is out of it.
    if (this.hold) {
      const l = n - 1, ll = toEdge ? upper[l] : lower[l], lu = upper[l], lx = time.x(l);
      if (lx < x1 && (log ? ll > 0 && lu > 0 : ll === ll && lu === lu) && (!colors || colors[l] === pass)) {
        const up = scale.y(lu), down = toEdge ? edgeY : scale.y(ll);
        if (!outline.count) outline.add(Math.max(lx, x0 - 1), up, down);
        outline.add(x1 + 1, up, down);
      }
    }
    outline.close();
    g.fill();
  }
}
