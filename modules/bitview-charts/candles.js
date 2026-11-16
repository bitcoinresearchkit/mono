// @ts-check
// Candles: four columns on the timeline (`open`, `high`, `low`, `close`, Float64Arrays; NaN in the close: none there),
// each candle its wick from low to high and its body from open to close, in `up` where it closed at or over its open,
// else in `down`. Zoomed out (under 3 CSS pixels a point), no candles: the close as a line, as a Line draws it. Its last
// value's tag on its axis and the line across at it are the close line's own (`tag`, `priceLine`, `format`; see lineOf),
// in the color of the candle it reads while candles show (its border's, rising or falling), the line's else.

import { Line } from "./line.js";
import { firstFrom } from "./timeline.js";
import { measure } from "./recorder.js";

/** The least spacing (CSS pixels a point) candles are drawn at: under it, the close as a line. */
export const BODIES_FROM = 3;

/**
 * A body's width in device pixels at `spacing` CSS pixels a point (as the chart library's: most of it zoomed out, a
 * fifth less zoomed in; three CSS pixels' worth from 2.5 to 4), whole, odd or even as its wick is (centered on it).
 * @param {number} spacing @param {number} dpr
 */
function bodyWidth(spacing, dpr) {
  const tight = 1 - (0.2 * Math.atan(Math.max(4, spacing) - 4)) / (Math.PI / 2), wick = Math.max(1, Math.round(dpr));
  const px = spacing >= 2.5 && spacing <= 4 ? Math.floor(3 * dpr) : Math.max(Math.floor(dpr), Math.min(Math.floor(spacing * tight * dpr), Math.floor(spacing * dpr)));
  return Math.max(1, px >= 2 && px % 2 !== wick % 2 ? px - 1 : px);
}

/** The Line a series' tag, price line and crosshair dot are read from: itself, a candle series' close line; none, any other. @param {any} series @returns {Line | null} */
export const lineOf = (series) => (series instanceof Line ? series : series instanceof Candles ? series.line : null);

/** A candle's parts, as #shapes draws them. */
const WICKS = 1, BODIES = 2, RINGS = 4, INSIDES = 8;

export class Candles {
  /**
   * `up` and `down`, the bodies' colors of those that rose and fell; their wicks' and borders' (`upWick`, `downWick`,
   * `upBorder`, `downBorder`), the same unless given (a hollow candle: its body the ground's, its border and wick its).
   * `color` and `width`: the line the close is drawn as zoomed out, under BODIES_FROM pixels a point; `tag`, `priceLine`,
   * `format`: its last value's badge on the axis and its dotted line across, as a Line's (candles shown, in their own
   * color); `marker`: the crosshair's dot on that line (none while candles show).
   * @param {{ open: Float64Array, high: Float64Array, low: Float64Array, close: Float64Array, up?: string, down?: string, upWick?: string | null, downWick?: string | null, upBorder?: string | null, downBorder?: string | null, outline?: string | null, radius?: number | ((spacing: number) => number), color?: string, width?: number, tag?: boolean, priceLine?: boolean, format?: ((v: number, step?: number) => string) | null, marker?: number }} options
   */
  constructor({ open, high, low, close, up = "#26a69a", down = "#ef5350", upWick = null, downWick = null, upBorder = null, downBorder = null, outline = null, radius = 0, color = "#f7f8f8", width = 1.5, tag = false, priceLine = false, format = null, marker = 3 }) {
    this.up = up;
    this.down = down;
    (this.upWick = upWick), (this.downWick = downWick), (this.upBorder = upBorder), (this.downBorder = downBorder);
    /** A color it's edged in: each candle's body and wick a pixel and a half wider all round, under them. @type {string | null} */
    this.outline = outline;
    /** Its bodies' corners' radius (a number, or one by the spacing of its points). @type {number | ((spacing: number) => number)} */
    this.radius = radius;
    /** How it widens its scale's range in its place (a page's own reading of it), none: by its extent. @type {((a: number, b: number, positive: boolean, out: Float64Array) => void) | null} */
    this.fit = null;
    // (Its lows' and highs' extremes kept as a line's are: a view's read block by block.)
    this.lows = new Line();
    this.highs = new Line();
    /** Its close drawn as a line, zoomed out; also what its tag, price line and crosshair dot are read from (see lineOf, #look). */
    this.line = new Line({ color, width, outline, tag, priceLine, format, marker });
    /** The line's crosshair dot's radius, while the line shows. */
    this.marker = marker;
    /** Whether candles are drawn at the zoom it was last brought to (see viewed). */
    this.bodies = false;
    /** Each candle's index into its tag's colors: 1 where it closed under its open (falling), else 0. @type {Uint8Array} */
    this.fell = new Uint8Array(0);
    /** @type {Float64Array} */
    this.open = open;
    /** Not drawn, fitted, tagged or hit (its pane drawn whole after a change: its chart's invalidate(true)). */
    this.hidden = false;
    /** Drawn at a fifth of its opacity, its tags gone (another lit over it: see the pane's Series). */
    this.faint = false;
    /** A scale of its own (see the pane's Series), none: its side's. @type {import("./scale.js").Scale | null} */
    this.overlay = null;
    this.version = 0;
    this.setValues({ open, high, low, close });
  }

  /** @returns {Float64Array} */
  get close() {
    return this.line.values;
  }

  /** The heights marks stand over (a candle's highs) and under (its lows) at this zoom: the close where the line's drawn. @param {import("./timeline.js").Timeline} time */
  tops(time) {
    return time.spacing < BODIES_FROM ? this.close : this.highs.values;
  }

  /** @param {import("./timeline.js").Timeline} time */
  feet(time) {
    return time.spacing < BODIES_FROM ? this.close : this.lows.values;
  }

  /**
   * Its columns; its lows and highs kept only where it has a close (a candle there): a missing one never widens the scale.
   * @param {{ open: Float64Array, high: Float64Array, low: Float64Array, close: Float64Array }} columns
   */
  setValues({ open, high, low, close }) {
    this.version++;
    this.open = open;
    this.line.setValues(close);
    // (In a loop: without a JIT, from() with a mapping costs several times as much.)
    const lows = new Float64Array(low.length), highs = new Float64Array(high.length), fell = new Uint8Array(close.length);
    for (let i = 0; i < lows.length; i++) lows[i] = close[i] === close[i] ? low[i] : NaN;
    for (let i = 0; i < highs.length; i++) highs[i] = close[i] === close[i] ? high[i] : NaN;
    for (let i = 0; i < fell.length; i++) fell[i] = close[i] < open[i] ? 1 : 0;
    this.lows.setValues(lows);
    this.highs.setValues(highs);
    this.fell = fell;
    this.#look();
  }

  /** One candle set in place (a live one). @param {number} i @param {number} open @param {number} high @param {number} low @param {number} close */
  set(i, open, high, low, close) {
    this.version++;
    this.open[i] = open;
    this.line.set(i, close);
    this.fell[i] = close < open ? 1 : 0;
    this.lows.set(i, close === close ? low : NaN);
    this.highs.set(i, close === close ? high : NaN);
  }

  /** @param {{ up?: string, down?: string, upWick?: string | null, downWick?: string | null, upBorder?: string | null, downBorder?: string | null, outline?: string | null, radius?: number | ((spacing: number) => number), color?: string, width?: number, tag?: boolean, priceLine?: boolean, format?: ((v: number, step?: number) => string) | null, marker?: number }} look */
  setStyle({ up = this.up, down = this.down, upWick = this.upWick, downWick = this.downWick, upBorder = this.upBorder, downBorder = this.downBorder, outline = this.outline, radius = this.radius, color = this.line.color, width = this.line.width, tag = this.line.tag, priceLine = this.line.priceLine, format, marker = this.marker }) {
    this.version++;
    (this.up = up), (this.down = down), (this.upWick = upWick), (this.downWick = downWick), (this.upBorder = upBorder), (this.downBorder = downBorder), (this.outline = outline), (this.radius = radius), (this.marker = marker);
    this.line.setStyle({ color, width, outline, tag, priceLine, format });
    this.#look();
  }

  /** Brought up to the view before its pane draws: its line's look made again where candles turn into it, or back (see #look). @param {import("./timeline.js").Timeline} time */
  viewed(time) {
    const bodies = time.spacing >= BODIES_FROM;
    if (bodies === this.bodies) return;
    (this.bodies = bodies), this.version++;
    this.#look();
  }

  /**
   * What its line is read as: while candles show, each candle's own color (its border's), no dot on it (so its tag
   * and its price line read in the color of the candle they're at); else its own color, its dot there.
   */
  #look() {
    if (this.bodies) this.line.setStyle({ colors: this.fell, palette: [this.upBorder ?? this.up, this.downBorder ?? this.down], marker: 0 });
    else this.line.setStyle({ colors: null, palette: [], marker: this.marker });
  }

  /** Its first value at or after index `i` (a relative scale's base: its first in view), or NaN: none. @param {number} i */
  first(i) {
    return firstFrom(this.close, i);
  }

  /**
   * How far a point (`x` into the plot, `y` in its pane: CSS pixels) is from it, measured against its drawing there
   * (see recorder.js: whatever it draws, as it draws it, at any zoom), on `scale` (its base set already, where it's
   * relative). At index `i` the pane's nearest point (unread: its drawing has it).
   * @param {number} _i @param {number} y @param {import("./scale.js").Scale} scale @param {number} x @param {import("./timeline.js").Timeline} time
   */
  distance(_i, y, scale, x, time) {
    return measure(this, time, scale, x, y);
  }

  /**
   * Its extent from `a` to `b`: its lows' and highs', or its closes' where the line's drawn (`time`: the view it's
   * drawn in; none, a candle's).
   * @param {number} a @param {number} b @param {boolean} positive @param {Float64Array} out @param {import("./timeline.js").Timeline} [time]
   */
  extent(a, b, positive, out, time) {
    if (time && time.spacing < BODIES_FROM) return this.line.extent(a, b, positive, out);
    this.lows.extent(a, b, positive, out);
    this.highs.extent(a, b, positive, out);
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr
   */
  draw(g, time, scale, x0, x1, dpr) {
    if (!this.close.length) return;
    if (time.spacing < BODIES_FROM) return this.line.draw(g, time, scale, x0, x1, dpr);
    // (Its outline first, all of them one fill; then the candles that rose, then those that fell: their wicks and
    // bodies one fill where they're one color (else the wicks, then the bodies); where a border's another color than its
    // body, the border a ring (its inside wound the other way: a hole), its inside a device pixel in, in the body's
    // color (see-through, where that is: the ground under it, not the border).)
    if (this.outline) (g.fillStyle = this.outline), g.beginPath(), this.#shapes(g, time, scale, x0, x1, dpr, -1, Math.max(1.5, 1 / dpr), WICKS | BODIES), g.fill();
    for (let pass = 0; pass < 2; pass++) {
      const body = pass ? this.down : this.up, wick = (pass ? this.downWick : this.upWick) ?? body, border = (pass ? this.downBorder : this.upBorder) ?? body;
      const hollow = border !== body, frame = hollow ? RINGS : BODIES;
      if (wick === border) (g.fillStyle = border), g.beginPath(), this.#shapes(g, time, scale, x0, x1, dpr, pass, 0, WICKS | frame), g.fill();
      else {
        (g.fillStyle = wick), g.beginPath(), this.#shapes(g, time, scale, x0, x1, dpr, pass, 0, WICKS), g.fill();
        (g.fillStyle = border), g.beginPath(), this.#shapes(g, time, scale, x0, x1, dpr, pass, 0, frame), g.fill();
      }
      if (hollow) (g.fillStyle = body), g.beginPath(), this.#shapes(g, time, scale, x0, x1, dpr, pass, 0, INSIDES), g.fill();
    }
  }

  /**
   * Its candles from x `x0` to `x1` added to the path: those that rose (`pass` 0), fell (1) or all (-1); their wicks
   * (above and below their bodies; an outline's, through them), bodies, bodies' rings (each body, its inside wound the
   * other way) or insides (a device pixel in, their top and foot on device pixels: no anti-aliasing where they meet
   * their rings) by `part`; each `pad` pixels wider all round (an outline's).
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr @param {number} pass @param {number} pad @param {number} part
   */
  #shapes(g, time, scale, x0, x1, dpr, pass, pad, part) {
    const { open, close } = this, high = this.highs.values, low = this.lows.values, n = close.length, log = scale.mode === "log";
    // (A wick and a body in whole device pixels, each its half (rounded down) left of its point's pixel: centered alike,
    // their widths odd or even alike.)
    const spacing = time.spacing, wicks = Math.max(1, Math.round(dpr)), bodies = bodyWidth(spacing, dpr);
    const wick = wicks / dpr, width = bodies / dpr, wickBy = Math.floor(wicks / 2) / dpr, bodyBy = Math.floor(bodies / 2) / dpr;
    const a = Math.max(0, Math.floor(time.index(x0)) - 1), b = Math.min(n - 1, Math.ceil(time.index(x1)) + 1), corner = typeof this.radius === "function" ? this.radius(spacing) : this.radius;
    for (let i = a; i <= b; i++) {
      const o = open[i], c = close[i], h = high[i], l = low[i];
      if (!(log ? c > 0 && l > 0 : c === c && l === l && h === h && o === o) || (pass >= 0 && (c >= o ? pass : !pass))) continue;
      // (Its body as drawn: a wick's width tall at least.)
      const x = time.snap(time.x(i), dpr), top = scale.y(Math.max(o, c)), foot = top + Math.max(wick, scale.y(Math.min(o, c)) - top), hy = scale.y(h), ly = scale.y(l);
      if (part & WICKS) {
        if (pad > 0) g.rect(x - wickBy - pad, hy - pad, wick + 2 * pad, ly - hy + 2 * pad);
        else {
          if (top > hy) g.rect(x - wickBy, hy, wick, top - hy);
          if (ly > foot) g.rect(x - wickBy, foot, wick, ly - foot);
        }
      }
      const left = x - bodyBy;
      if (part & (BODIES | RINGS)) this.#body(g, left - pad, top - pad, width + 2 * pad, foot - top + 2 * pad, corner > 0 ? corner + pad : 0, false);
      if (!(part & (RINGS | INSIDES))) continue;
      const inTop = Math.round((top + wick) * dpr) / dpr, inFoot = Math.round((foot - wick) * dpr) / dpr;
      this.#body(g, left + wick, inTop, width - 2 * wick, inFoot - inTop, corner > 0 ? corner - wick : 0, (part & RINGS) !== 0);
    }
  }

  /**
   * A body's shape added to the path (none where it has no room), its corners rounded by `radius` (no more than half
   * its width or its height); `back`, wound the other way (a hole in what it's within: a roundRect drawn from its right,
   * as both Chromium and WebKit wind it, where WebKit's rect isn't).
   * @param {CanvasRenderingContext2D} g @param {number} x @param {number} y @param {number} width @param {number} height @param {number} radius @param {boolean} back
   */
  #body(g, x, y, width, height, radius, back) {
    if (width <= 0 || height <= 0) return;
    const r = Math.max(0, Math.min(radius, width / 2, height / 2));
    if (back) g.roundRect(x + width, y, -width, height, r);
    else if (r > 0) g.roundRect(x, y, width, height, r);
    else g.rect(x, y, width, height);
  }
}
