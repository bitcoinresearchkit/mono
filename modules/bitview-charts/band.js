// @ts-check
import { Fill } from "./color.js";
import { NOISE } from "./timeline.js";
// A band: a fill between two columns on the timeline (`lower` and `upper`, Float64Arrays; NaN in either: none there,
// the band broken), in one color. A cloud between two lines is one; a stack is one between each two edges (see
// stack.js). Without `lower`, an area: filled down to `base` (0 if not given; on a log scale, a base it can't show (0
// or under) is its foot: the area down to the pane's foot), or `to` the pane's "top" or its "foot"; fitted to its
// values alone; with `below` too, a baseline: under its base, in that color. With `steps`, its edges steps (each value
// held to the next point). Zoomed out, each pixel column takes each edge's last value in it: bands sharing an edge meet
// exactly, no gap or overlap between them, whatever the zoom.

/**
 * A band's outline as it's written: each point's x and its edges' ys kept for the way back along the lower edge (grown,
 * never made again), and where it's curved, the control points of the segments to it (`bends`: the upper edge's two,
 * then the lower's; `bent`: 1, the upper edge's segment curved, 2, the lower's); a stretch closed at a gap. One for every
 * band, drawn one after another.
 */
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
  /** The stretch so far: up its upper edge, back along its lower (each segment curved where it's bent). */
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

export class Band {
  /**
   * `fade`, its fill's color at its far end (a baseline's base, else the pane's foot) where it fades from `color` (at the
   * pane's top), a baseline's `belowFade` from `below` (at its base) to the pane's foot: gradients fixed to the pane,
   * so a pan moves them with its picture. None: one color.
   * `curve`: its edges bowed through their points (zoomed in), as a curved line is (its edge's line alike). `colors`
   * (a Uint8Array of indices into `palette`, its fills: a color and what it fades to): each segment in its first point's
   * (a band colored by another series), as a line's.
   * @param {{ lower?: Float64Array, upper: Float64Array, base?: number, color?: string, fade?: string | null, below?: string | null, belowFade?: string | null, to?: "base" | "top" | "foot", steps?: boolean, curve?: boolean, colors?: Uint8Array | null, palette?: { color: string, fade?: string | null }[] }} options
   */
  constructor({ lower, upper, base = 0, color = "rgba(247, 248, 248, 0.2)", fade = null, below = null, belowFade = null, to = "base", steps = false, curve = false, colors = null, palette = [] }) {
    /** @type {Float64Array} */
    this.lower = lower ?? new Float64Array(upper.length).fill(base);
    this.base = base;
    /** An area: its lower edge its base, made to its values' length. */
    this.area = !lower;
    /** Its color under its base (a baseline), or none: one color; each fading to another, or none. */
    this.below = below;
    this.fade = fade;
    this.belowFade = belowFade;
    // (Its gradients, made again only as their colors or ends change: nothing made a frame.)
    this.overFill = new Fill();
    this.underFill = new Fill();
    /** An area filled up from its values to the pane's top (else down to its base). */
    this.to = to;
    this.steps = steps;
    this.curve = curve;
    this.colors = colors;
    this.palette = palette;
    // (Its palette's fills, made once each.)
    /** @type {Fill[]} */
    this.fills = [];
    /** How it widens its scale's range in its place (a page's own reading of it), none: by its extent. @type {((a: number, b: number, positive: boolean, out: Float64Array) => void) | null} */
    this.fit = null;
    /** @type {Float64Array} */
    this.upper = upper;
    this.color = color;
    /** Not drawn, fitted, tagged or hit (its pane drawn whole after a change: its chart's invalidate(true)). */
    this.hidden = false;
    /** Changed at each change of its edges or color: its pane's picture drawn whole then. */
    this.version = 0;
    // (Each point's next without both edges (on a log scale, without both above zero), made at its first draw of a
    // version: a pixel column of points with no gap reads its last alone.)
    /** @type {Int32Array | null} */
    this.gaps = null;
    /** @type {Int32Array | null} */
    this.logGaps = null;
    this.gapsVersion = -1;
  }

  /** Its first value at or after index `i` (a relative scale's base: its first in view), or NaN: none. @param {number} i */
  first(i) {
    const values = this.upper;
    for (let k = Math.max(0, i); k < values.length; k++) if (values[k] === values[k]) return values[k];
    return NaN;
  }

  /**
   * How far a point (`y`, CSS pixels in its pane; `x` in the plot, past the data's ends) is from it at index `i`
   * (Infinity: it has nothing there), on `scale` (its base set already, where it's relative).
   * @param {number} i @param {number} y @param {import("./scale.js").Scale} scale @param {number} [x] @param {import("./timeline.js").Timeline} [time]
   */
  distance(i, y, scale, x, time) {
    // (As it's drawn: both edges there, above zero on a log scale.)
    if (!(i >= 0 && i < this.lower.length && i < this.upper.length)) return Infinity;
    const log = scale.mode === "log", edge = this.#edge(log), hi = this.upper[i], lo = edge ? hi : this.lower[i];
    if (!(log ? lo > 0 && hi > 0 : lo === lo && hi === hi)) return Infinity;
    const up = this.area && this.to === "top", top = up ? 0 : scale.y(edge ? hi : Math.max(lo, hi)), bottom = up ? scale.y(hi) : edge ? scale.height : scale.y(Math.min(lo, hi));
    return Math.hypot(x === undefined || !time ? 0 : time.beyond(i, x), y < top ? top - y : y > bottom ? y - bottom : 0);
  }

  /**
   * A pixel column's point added to the outline of the fill being drawn: its own (`mine`), or another color's, as the end
   * of this color's segment to it (if one comes to it), closing it.
   * @param {number} x @param {number} up @param {number} down @param {boolean} mine
   */
  #put(x, up, down, mine) {
    if (!mine && !outline.count) return;
    outline.add(x, up, down);
    if (!mine) outline.close();
  }

  /**
   * The control points of an edge's curved segment to point `i` from `i - 1` (CSS pixels, into the outline's bends from
   * `out`), as a curved line's: each a sixth of the way along its neighbors' chord, those by index where the band's drawn
   * (else the point itself), so a strip's and a whole draw's alike.
   * @param {Float64Array} edge @param {number} i @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {boolean} log @param {boolean} toEdge @param {number} x1 @param {number} y1 @param {number} x2 @param {number} y2 @param {number} out
   */
  #bend(edge, i, time, scale, log, toEdge, x1, y1, x2, y2, out) {
    const lows = toEdge ? this.upper : this.lower, highs = this.upper, n = Math.min(lows.length, highs.length), w = i - 2, u = i + 1, b = outline.bends;
    const before = w >= 0 && (log ? lows[w] > 0 && highs[w] > 0 : lows[w] === lows[w] && highs[w] === highs[w]);
    const after = u < n && (log ? lows[u] > 0 && highs[u] > 0 : lows[u] === lows[u] && highs[u] === highs[u]);
    const x0 = before ? time.x(i - 2) : x1, y0 = before ? scale.y(edge[i - 2]) : y1, x3 = after ? time.x(i + 1) : x2, y3 = after ? scale.y(edge[i + 1]) : y2;
    (b[out] = x1 + (x2 - x0) / 6), (b[out + 1] = y1 + (y2 - y0) / 6), (b[out + 2] = x2 - (x3 - x1) / 6), (b[out + 3] = y2 - (y3 - y1) / 6);
  }

  /** Its next gap from each point (the length: none). @param {boolean} log */
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
   * Its edges, color or base changed: a lower edge given, a band; `lower: null`, an area again (filled to its base, or
   * `to` the pane's "top" or "foot").
   * @param {{ lower?: Float64Array | null, upper?: Float64Array, color?: string, fade?: string | null, below?: string | null, belowFade?: string | null, base?: number, to?: "base" | "top" | "foot", steps?: boolean, curve?: boolean, colors?: Uint8Array | null, palette?: { color: string, fade?: string | null }[] }} change
   */
  set({ lower, upper = this.upper, color = this.color, fade = this.fade, below = this.below, belowFade = this.belowFade, base = this.base, to = this.to, steps = this.steps, curve = this.curve, colors = this.colors, palette = this.palette }) {
    this.version++;
    const wasArea = this.area;
    if (lower === null) this.area = true;
    else if (lower) this.area = false;
    // (An area's base made again to its values' length, or a new base, or once it was a band's lower edge.)
    const kept = wasArea && this.lower.length === upper.length && base === this.base;
    this.base = base;
    this.lower = lower ?? (this.area ? (kept ? this.lower : new Float64Array(upper.length).fill(base)) : this.lower);
    (this.upper = upper), (this.color = color), (this.below = below), (this.fade = fade), (this.belowFade = belowFade), (this.to = to), (this.steps = steps), (this.curve = curve), (this.colors = colors), (this.palette = palette);
  }

  /**
   * Whether it's an area filled to an edge of the pane, its lower edge none: `to` its top or its foot, or, on a log
   * scale, down to its foot where its base can't show. @param {boolean} log
   */
  #edge(log) {
    return this.area && (this.to !== "base" || (log && !(this.base > 0)));
  }

  /**
   * Its lowest and highest from index `a` to `b`, written into `out` widened ([lo, hi]): a band's, its lower edge's
   * lowest, its upper's highest; an area's, its values' alone (its fill reaching what's under them, its base not
   * fitted to, as an area's values are read).
   * @param {number} a @param {number} b @param {boolean} positive @param {Float64Array} out
   */
  extent(a, b, positive, out) {
    a = Math.max(0, a);
    b = Math.min(this.lower.length - 1, this.upper.length - 1, b);
    // (Only where it's drawn: both edges there, above zero on a log scale; either edge the lower (a band crossing).)
    for (let i = a; i <= b; i++) {
      const l = this.area ? this.upper[i] : this.lower[i], u = this.upper[i];
      if (!(positive ? l > 0 && u > 0 : l === l && u === u)) continue;
      const lo = l < u ? l : u, hi = l < u ? u : l;
      if (lo < out[0]) out[0] = lo;
      if (hi > out[1]) out[1] = hi;
    }
  }

  /**
   * Filled on `g` (CSS pixels) from x `x0` to `x1` of the plot.
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1
   */
  draw(g, time, scale, x0, x1) {
    if (!this.below) {
      // (Colored point by point: a fill a color, its segments in it.)
      const palette = this.colors ? this.palette : [];
      for (let k = 0; k < palette.length; k++) this.#fill(g, time, scale, x0, x1, (this.fills[k] ??= new Fill()).style(g, palette[k].color, palette[k].fade ?? null, 0, scale.height), k);
      if (!palette.length) this.#fill(g, time, scale, x0, x1, this.overFill.style(g, this.color, this.fade, 0, scale.height), -1);
      return;
    }
    // (A baseline: over its base in its color, under it in the other; the same outline filled twice, clipped at it (at
    // the foot where its scale can't show it: a log scale's base of zero), each fading towards its base.)
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
   * Its outline from x `x0` to `x1` filled in `color` (a gradient's too): all of it, or (`pass`, an index into its
   * palette) its segments colored so, each to the point after it.
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {string | CanvasGradient} color @param {number} pass
   */
  #fill(g, time, scale, x0, x1, color, pass) {
    const lower = this.lower, upper = this.upper, n = Math.min(lower.length, upper.length), log = scale.mode === "log", colors = pass >= 0 ? this.colors : null;
    if (!n) return;
    const per = 1 / time.spacing, zoomedIn = per <= 1, viewFrom = time.from, span = time.to - time.from, width = time.width, gaps = zoomedIn ? null : this.#gaps(log);
    // (Zoomed out, its y as bottom - k * (t - ta) (t the value, or its log, from the scale's low end), inline: without
    // a JIT a call costs more than its work. Its columns' points as Timeline.index has them, to the bit.)
    const ta = log ? Math.log10(scale.lo) : scale.lo, tb = log ? Math.log10(scale.hi) : scale.hi, top = scale.top * scale.height + scale.above, room = scale.height - top - scale.bottom * scale.height - scale.below;
    // (An area to an edge of the pane: its values against that edge, its top or its foot.)
    const kk = room / (tb - ta), bottom = top + room, m = scale.m, c = scale.c, toEdge = this.#edge(log), edgeY = this.to === "top" ? 0 : scale.height;
    const a = Math.max(0, Math.floor(time.index(x0)) - 1), b = Math.min(n - 1, Math.ceil(time.index(x1)) + 1);
    // (A point each, zoomed in; a pixel column each, its last, zoomed out.)
    const from = zoomedIn ? a : Math.floor(x0) - 1, to = zoomedIn ? b : Math.ceil(x1) + 1, steps = zoomedIn && this.steps, curve = zoomedIn && this.curve && !steps;
    outline.start(g, (to - from + 2) * (steps ? 2 : 1));
    g.fillStyle = color;
    g.beginPath();
    for (let k = from; k <= to; k++) {
      if (zoomedIn) {
        const lo = toEdge ? upper[k] : lower[k], hi = upper[k];
        if (!(log ? lo > 0 && hi > 0 : lo === lo && hi === hi)) {
          outline.close();
          continue;
        }
        // (Another color's point: the end of this color's segment to it, if one comes to it, else none.)
        const mine = !colors || colors[k] === pass;
        if (!mine && !outline.count) continue;
        const x = time.x(k), at = outline.count - 1, up = scale.y(hi), down = toEdge ? edgeY : scale.y(lo);
        // (A step: its edges held at the last point's to here, then straight to this one's.)
        if (steps && at >= 0) outline.add(x, outline.ups[at], outline.downs[at]);
        outline.add(x, up, down);
        // (Curved: its segment to here bowed as a curved line's, its upper edge's and (a band's) its lower's.)
        if (curve && at >= 0) {
          const ox = outline.xs[at];
          this.#bend(upper, k, time, scale, log, toEdge, ox, outline.ups[at], x, up, 8 * (at + 1));
          outline.bent[at + 1] = 1;
          if (!toEdge && !this.area) this.#bend(lower, k, time, scale, log, toEdge, ox, outline.downs[at], x, down, 8 * (at + 1) + 4), (outline.bent[at + 1] |= 2);
        }
        if (!mine) outline.close();
        continue;
      }
      // (A pixel column: its last point with both edges, the band broken at any point without, in it too; no gap in
      // it (most), its last read alone.)
      const first = Math.max(0, Math.ceil(viewFrom + (k * span) / width - NOISE)), last = Math.min(n - 1, Math.ceil(viewFrom + ((k + 1) * span) / width - NOISE) - 1);
      if (last < first) continue;
      if (/** @type {Int32Array} */ (gaps)[first] > last) {
        const hi = upper[last], lo = lower[last];
        this.#put(k, bottom - kk * ((log ? Math.log10(hi) : hi * m + c) - ta), toEdge ? edgeY : bottom - kk * ((log ? Math.log10(lo) : lo * m + c) - ta), !colors || colors[last] === pass);
        continue;
      }
      let at = -1;
      for (let i = first; i <= last; i++) {
        const lo = toEdge ? upper[i] : lower[i], hi = upper[i];
        if (log ? lo > 0 && hi > 0 : lo === lo && hi === hi) at = i;
        else {
          if (at >= 0) this.#put(k, scale.y(upper[at]), toEdge ? edgeY : scale.y(lower[at]), !colors || colors[at] === pass);
          outline.close();
          at = -1;
        }
      }
      if (at >= 0) this.#put(k, scale.y(upper[at]), toEdge ? edgeY : scale.y(lower[at]), !colors || colors[at] === pass);
    }
    outline.close();
    g.fill();
  }

}
