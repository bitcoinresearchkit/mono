// @ts-check
// Candles: four columns on the timeline (`open`, `high`, `low`, `close`, Float64Arrays; NaN in the close: none there),
// each candle its wick from low to high and its body from open to close, in `up` where it closed at or over its open,
// else in `down`. Zoomed out (under 3 CSS pixels a point), each pixel column one candle of its points (its first open,
// its highest high and lowest low, its last close) drawn as its wick alone.

import { Line } from "./line.js";

/** A candle's parts, as #shapes draws them. */
const WICKS = 1, BODIES = 2, WHOLE = 3;

export class Candles {
  /**
   * `up` and `down`, the bodies' colors of those that rose and fell; their wicks' and borders' (`upWick`, `downWick`,
   * `upBorder`, `downBorder`), the same unless given (a hollow candle: its body the ground's, its border and wick its).
   * @param {{ open: Float64Array, high: Float64Array, low: Float64Array, close: Float64Array, up?: string, down?: string, upWick?: string | null, downWick?: string | null, upBorder?: string | null, downBorder?: string | null }} options
   */
  constructor({ open, high, low, close, up = "#26a69a", down = "#ef5350", upWick = null, downWick = null, upBorder = null, downBorder = null }) {
    this.up = up;
    this.down = down;
    (this.upWick = upWick), (this.downWick = downWick), (this.upBorder = upBorder), (this.downBorder = downBorder);
    /** A color it's edged in: each candle's body and wick a pixel and a half wider all round, under them. @type {string | null} */
    this.outline = null;
    /** Its bodies' corners' radius (a number, or one by the spacing of its points). @type {number | ((spacing: number) => number)} */
    this.radius = 0;
    /** How it widens its scale's range in its place (a page's own reading of it), none: by its extent. @type {((a: number, b: number, positive: boolean, out: Float64Array) => void) | null} */
    this.fit = null;
    // (Its lows' and highs' extremes kept as a line's are: a view's read block by block.)
    this.lows = new Line();
    this.highs = new Line();
    /** @type {Float64Array} */
    this.open = open;
    /** @type {Float64Array} */
    this.close = close;
    /** Not drawn, fitted, tagged or hit (its pane drawn whole after a change: its chart's invalidate(true)). */
    this.hidden = false;
    this.version = 0;
    this.setValues({ open, high, low, close });
  }

  get high() {
    return this.highs.values;
  }

  get low() {
    return this.lows.values;
  }

  /**
   * Its columns; its lows and highs kept only where it has a close (a candle there): a missing one never widens the scale.
   * @param {{ open: Float64Array, high: Float64Array, low: Float64Array, close: Float64Array }} columns
   */
  setValues({ open, high, low, close }) {
    this.version++;
    (this.open = open), (this.close = close);
    this.lows.setValues(Float64Array.from(low, (v, i) => (close[i] === close[i] ? v : NaN)));
    this.highs.setValues(Float64Array.from(high, (v, i) => (close[i] === close[i] ? v : NaN)));
  }

  /** One candle set in place (a live one). @param {number} i @param {number} open @param {number} high @param {number} low @param {number} close */
  set(i, open, high, low, close) {
    this.version++;
    (this.open[i] = open), (this.close[i] = close);
    this.lows.set(i, close === close ? low : NaN);
    this.highs.set(i, close === close ? high : NaN);
  }

  /** @param {{ up?: string, down?: string, upWick?: string | null, downWick?: string | null, upBorder?: string | null, downBorder?: string | null, outline?: string | null, radius?: number | ((spacing: number) => number) }} look */
  setStyle({ up = this.up, down = this.down, upWick = this.upWick, downWick = this.downWick, upBorder = this.upBorder, downBorder = this.downBorder, outline = this.outline, radius = this.radius }) {
    this.version++;
    (this.up = up), (this.down = down), (this.upWick = upWick), (this.downWick = downWick), (this.upBorder = upBorder), (this.downBorder = downBorder), (this.outline = outline), (this.radius = radius);
  }

  /** Its first value at or after index `i` (a relative scale's base: its first in view), or NaN: none. @param {number} i */
  first(i) {
    const values = this.close;
    for (let k = Math.max(0, i); k < values.length; k++) if (values[k] === values[k]) return values[k];
    return NaN;
  }

  /**
   * How far a point (`y`, CSS pixels in its pane) is from it at index `i` (Infinity: it has nothing there), on `scale`
   * (its base set already, where it's relative).
   * @param {number} i @param {number} y @param {import("./scale.js").Scale} scale
   */
  distance(i, y, scale) {
    // (As it's drawn: a candle there, its close and low above zero on a log scale.)
    if (!(i >= 0 && i < this.close.length)) return Infinity;
    const h = this.highs.values[i], l = this.lows.values[i], c = this.close[i];
    if (!(scale.mode === "log" ? c > 0 && l > 0 && h > 0 : c === c && h === h && l === l)) return Infinity;
    const top = scale.y(h), bottom = scale.y(l);
    return y < top ? top - y : y > bottom ? y - bottom : 0;
  }

  /** @param {number} a @param {number} b @param {boolean} positive @param {Float64Array} out */
  extent(a, b, positive, out) {
    this.lows.extent(a, b, positive, out);
    this.highs.extent(a, b, positive, out);
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr
   */
  draw(g, time, scale, x0, x1, dpr) {
    if (!this.close.length) return;
    // (Its outline first, all of them one fill; then the candles that rose, then those that fell: their wicks and
    // bodies one fill where they're one color (else the wicks, then the bodies), then where a border's another color
    // than its body, the body a device pixel in, in its own.)
    if (this.outline) (g.fillStyle = this.outline), g.beginPath(), this.#shapes(g, time, scale, x0, x1, dpr, -1, Math.max(1.5, 1 / dpr), WHOLE), g.fill();
    const inset = -Math.max(1, Math.round(dpr)) / dpr;
    for (let pass = 0; pass < 2; pass++) {
      const body = pass ? this.down : this.up, wick = (pass ? this.downWick : this.upWick) ?? body, border = (pass ? this.downBorder : this.upBorder) ?? body;
      if (wick === border) (g.fillStyle = border), g.beginPath(), this.#shapes(g, time, scale, x0, x1, dpr, pass, 0, WHOLE), g.fill();
      else {
        (g.fillStyle = wick), g.beginPath(), this.#shapes(g, time, scale, x0, x1, dpr, pass, 0, WICKS), g.fill();
        (g.fillStyle = border), g.beginPath(), this.#shapes(g, time, scale, x0, x1, dpr, pass, 0, BODIES), g.fill();
      }
      if (border !== body) (g.fillStyle = body), g.beginPath(), this.#shapes(g, time, scale, x0, x1, dpr, pass, inset, BODIES), g.fill();
    }
  }

  /**
   * Its candles from x `x0` to `x1` added to the path: those that rose (`pass` 0), fell (1) or all (-1), their wicks,
   * bodies or both (`part`), each `pad` pixels wider all round (an outline's; under 0, narrower: a border's inside).
   * Zoomed out, a pixel column's range its wick and its body.
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr @param {number} pass @param {number} pad @param {number} part
   */
  #shapes(g, time, scale, x0, x1, dpr, pass, pad, part) {
    const { open, close } = this, high = this.highs.values, low = this.lows.values, n = close.length, log = scale.mode === "log";
    const spacing = time.spacing, wide = spacing >= 3, half = Math.max(0.5, (spacing * 0.7) / 2), wick = Math.max(1, Math.round(dpr)) / dpr;
    const a = Math.max(0, Math.floor(time.index(x0)) - 1), b = Math.min(n - 1, Math.ceil(time.index(x1)) + 1), corner = typeof this.radius === "function" ? this.radius(spacing) : this.radius;
    if (wide) {
      for (let i = a; i <= b; i++) {
        const o = open[i], c = close[i], h = high[i], l = low[i];
        if (!(log ? c > 0 && l > 0 : c === c && l === l && h === h && o === o) || (pass >= 0 && (c >= o ? pass : !pass))) continue;
        const x = Math.round(time.x(i) * dpr) / dpr, top = scale.y(Math.max(o, c)), bottom = scale.y(Math.min(o, c));
        if (part & WICKS) g.rect(x - wick / 2 - pad, scale.y(h) - pad, wick + 2 * pad, scale.y(l) - scale.y(h) + 2 * pad);
        if (!(part & BODIES)) continue;
        const width = Math.round(2 * half * dpr) / dpr + 2 * pad, height = Math.max(wick, bottom - top) + 2 * pad;
        if (width <= 0 || height <= 0) continue;
        // (Its body's corners rounded (no more than half its width or its height), an outline's as much more.)
        if (corner > 0) g.roundRect(Math.round((x - half) * dpr) / dpr - pad, top - pad, width, height, Math.max(0, Math.min(corner + pad, width / 2, height / 2)));
        else g.rect(Math.round((x - half) * dpr) / dpr - pad, top - pad, width, height);
      }
      return;
    }
    for (let x = Math.floor(x0 - pad) - 1; x <= Math.ceil(x1 + pad) + 1; x++) {
      const from = Math.max(0, Math.ceil(time.index(x))), to = Math.min(n - 1, Math.ceil(time.index(x + 1)) - 1);
      let o = NaN, c = NaN, h = -Infinity, l = Infinity;
      for (let i = from; i <= to; i++) {
        if (!(log ? close[i] > 0 && low[i] > 0 : close[i] === close[i])) continue;
        if (o !== o) o = open[i];
        c = close[i];
        if (high[i] > h) h = high[i];
        if (low[i] < l) l = low[i];
      }
      // (A column's one rect, drawn with its wicks; its insides none.)
      if (c !== c || (pass >= 0 && (c >= o ? pass : !pass)) || !(part & WICKS) || pad < 0) continue;
      g.rect(x - pad, scale.y(h) - pad, 1 + 2 * pad, Math.max(wick, scale.y(l) - scale.y(h)) + 2 * pad);
    }
  }
}
