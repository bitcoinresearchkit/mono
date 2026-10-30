// @ts-check
// Candles: four columns on the timeline (`open`, `high`, `low`, `close`, Float64Arrays; NaN in the close: none there),
// each candle its wick from low to high and its body from open to close, in `up` where it closed at or over its open,
// else in `down`. Zoomed out (under 3 CSS pixels a point), each pixel column one candle of its points (its first open,
// its highest high and lowest low, its last close) drawn as its wick alone.

import { Line } from "./line.js";

export class Candles {
  /**
   * @param {{ open: Float64Array, high: Float64Array, low: Float64Array, close: Float64Array, up?: string, down?: string }} options
   */
  constructor({ open, high, low, close, up = "#26a69a", down = "#ef5350" }) {
    this.up = up;
    this.down = down;
    // (Its lows' and highs' extremes kept as a line's are: a view's read block by block.)
    this.lows = new Line();
    this.highs = new Line();
    /** @type {Float64Array} */
    this.open = open;
    /** @type {Float64Array} */
    this.close = close;
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

  /** @param {{ up?: string, down?: string }} look */
  setStyle({ up = this.up, down = this.down }) {
    this.version++;
    (this.up = up), (this.down = down);
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
    const { open, close } = this, high = this.highs.values, low = this.lows.values, n = close.length, log = scale.mode === "log";
    if (!n) return;
    const spacing = time.spacing, wide = spacing >= 3, half = Math.max(0.5, (spacing * 0.7) / 2), wick = Math.max(1, Math.round(dpr)) / dpr;
    const a = Math.max(0, Math.floor(time.index(x0)) - 1), b = Math.min(n - 1, Math.ceil(time.index(x1)) + 1);
    // (Two passes: the candles that rose, then those that fell, each one fill.)
    for (let pass = 0; pass < 2; pass++) {
      g.fillStyle = pass ? this.down : this.up;
      g.beginPath();
      if (wide) {
        for (let i = a; i <= b; i++) {
          const o = open[i], c = close[i], h = high[i], l = low[i];
          if (!(log ? c > 0 && l > 0 : c === c && l === l && h === h && o === o) || (c >= o ? pass : !pass)) continue;
          const x = Math.round(time.x(i) * dpr) / dpr, top = scale.y(Math.max(o, c)), bottom = scale.y(Math.min(o, c));
          g.rect(x - wick / 2, scale.y(h), wick, scale.y(l) - scale.y(h));
          g.rect(Math.round((x - half) * dpr) / dpr, top, Math.round(2 * half * dpr) / dpr, Math.max(wick, bottom - top));
        }
      } else {
        for (let x = Math.floor(x0) - 1; x <= Math.ceil(x1) + 1; x++) {
          const from = Math.max(0, Math.ceil(time.index(x))), to = Math.min(n - 1, Math.ceil(time.index(x + 1)) - 1);
          let o = NaN, c = NaN, h = -Infinity, l = Infinity;
          for (let i = from; i <= to; i++) {
            if (!(log ? close[i] > 0 && low[i] > 0 : close[i] === close[i])) continue;
            if (o !== o) o = open[i];
            c = close[i];
            if (high[i] > h) h = high[i];
            if (low[i] < l) l = low[i];
          }
          if (c !== c || (c >= o ? pass : !pass)) continue;
          g.rect(x, scale.y(h), 1, Math.max(wick, scale.y(l) - scale.y(h)));
        }
      }
      g.fill();
    }
  }
}
