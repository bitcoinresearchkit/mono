// @ts-check

import { Line } from "./line.js";
import { firstFrom } from "./timeline.js";
import { measure } from "./recorder.js";

export const BODIES_FROM = 3;

/** @param {number} spacing @param {number} dpr */
function bodyWidth(spacing, dpr) {
  const tight = 1 - (0.2 * Math.atan(Math.max(4, spacing) - 4)) / (Math.PI / 2), wick = Math.max(1, Math.round(dpr));
  const px = spacing >= 2.5 && spacing <= 4 ? Math.floor(3 * dpr) : Math.max(Math.floor(dpr), Math.min(Math.floor(spacing * tight * dpr), Math.floor(spacing * dpr)));
  return Math.max(1, px >= 2 && px % 2 !== wick % 2 ? px - 1 : px);
}

/** @param {any} series @returns {Line | null} */
export const lineOf = (series) => (series instanceof Line ? series : series instanceof Candles ? series.line : null);

const WICKS = 1, BODIES = 2, RINGS = 4, INSIDES = 8;

export class Candles {
  /**
   * @param {{ open: Float64Array, high: Float64Array, low: Float64Array, close: Float64Array, up?: string, down?: string, upWick?: string | null, downWick?: string | null, upBorder?: string | null, downBorder?: string | null, outline?: string | null, radius?: number | ((spacing: number) => number), color?: string, width?: number, tag?: boolean, priceLine?: boolean, format?: ((v: number, step?: number) => string) | null, marker?: number }} options
   */
  constructor({ open, high, low, close, up = "#26a69a", down = "#ef5350", upWick = null, downWick = null, upBorder = null, downBorder = null, outline = null, radius = 0, color = "#f7f8f8", width = 1.5, tag = false, priceLine = false, format = null, marker = 3 }) {
    this.up = up;
    this.down = down;
    (this.upWick = upWick), (this.downWick = downWick), (this.upBorder = upBorder), (this.downBorder = downBorder);
    /** @type {string | null} */
    this.outline = outline;
    /** @type {number | ((spacing: number) => number)} */
    this.radius = radius;
    /** @type {((a: number, b: number, positive: boolean, out: Float64Array) => void) | null} */
    this.fit = null;
    this.lows = new Line();
    this.highs = new Line();
    this.line = new Line({ color, width, outline, tag, priceLine, format, marker });
    this.marker = marker;
    this.bodies = false;
    /** 1 where it fell. @type {Uint8Array} */
    this.fell = new Uint8Array(0);
    /** @type {Float64Array} */
    this.open = open;
    this.hidden = false;
    this.faint = false;
    /** @type {import("./scale.js").Scale | null} */
    this.overlay = null;
    this.version = 0;
    this.setValues({ open, high, low, close });
  }

  /** @returns {Float64Array} */
  get close() {
    return this.line.values;
  }

  /** @param {import("./timeline.js").Timeline} time */
  tops(time) {
    return time.spacing < BODIES_FROM ? this.close : this.highs.values;
  }

  /** @param {import("./timeline.js").Timeline} time */
  feet(time) {
    return time.spacing < BODIES_FROM ? this.close : this.lows.values;
  }

  /**
   * Only where there's a close: a missing one never widens the scale.
   * @param {{ open: Float64Array, high: Float64Array, low: Float64Array, close: Float64Array }} columns
   */
  setValues({ open, high, low, close }) {
    this.version++;
    this.open = open;
    this.line.setValues(close);
    // In a loop: from() with a mapping is several times slower without a JIT.
    const lows = new Float64Array(low.length), highs = new Float64Array(high.length), fell = new Uint8Array(close.length);
    for (let i = 0; i < lows.length; i++) lows[i] = close[i] === close[i] ? low[i] : NaN;
    for (let i = 0; i < highs.length; i++) highs[i] = close[i] === close[i] ? high[i] : NaN;
    for (let i = 0; i < fell.length; i++) fell[i] = close[i] < open[i] ? 1 : 0;
    this.lows.setValues(lows);
    this.highs.setValues(highs);
    this.fell = fell;
    this.#look();
  }

  /** @param {number} i @param {number} open @param {number} high @param {number} low @param {number} close */
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

  /** @param {import("./timeline.js").Timeline} time */
  viewed(time) {
    const bodies = time.spacing >= BODIES_FROM;
    if (bodies === this.bodies) return;
    (this.bodies = bodies), this.version++;
    this.#look();
  }

  #look() {
    if (this.bodies) this.line.setStyle({ colors: this.fell, palette: [this.upBorder ?? this.up, this.downBorder ?? this.down], marker: 0 });
    else this.line.setStyle({ colors: null, palette: [], marker: this.marker });
  }

  /** @param {number} i */
  first(i) {
    return firstFrom(this.close, i);
  }

  /** @param {number} _i @param {number} y @param {import("./scale.js").Scale} scale @param {number} x @param {import("./timeline.js").Timeline} time */
  distance(_i, y, scale, x, time) {
    return measure(this, time, scale, x, y);
  }

  /**
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
    // A border of another color is a ring (inside wound the other way), so a see-through body shows the ground.
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
   * No anti-aliasing where insides meet their rings.
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr @param {number} pass @param {number} pad @param {number} part
   */
  #shapes(g, time, scale, x0, x1, dpr, pass, pad, part) {
    const { open, close } = this, high = this.highs.values, low = this.lows.values, n = close.length, log = scale.mode === "log";
    const spacing = time.spacing, wicks = Math.max(1, Math.round(dpr)), bodies = bodyWidth(spacing, dpr);
    const wick = wicks / dpr, width = bodies / dpr, wickBy = Math.floor(wicks / 2) / dpr, bodyBy = Math.floor(bodies / 2) / dpr;
    const a = Math.max(0, Math.floor(time.index(x0)) - 1), b = Math.min(n - 1, Math.ceil(time.index(x1)) + 1), corner = typeof this.radius === "function" ? this.radius(spacing) : this.radius;
    for (let i = a; i <= b; i++) {
      const o = open[i], c = close[i], h = high[i], l = low[i];
      if (!(log ? c > 0 && l > 0 : c === c && l === l && h === h && o === o) || (pass >= 0 && (c >= o ? pass : !pass))) continue;
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
   * `back` winds the other way: roundRect starts from its right in Chromium and WebKit, WebKit's rect doesn't.
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
