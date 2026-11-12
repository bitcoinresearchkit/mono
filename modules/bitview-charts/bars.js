// @ts-check
// Bars: a histogram from one column (a Float64Array on the timeline), each bar from `base` to its value, rounded at its
// value's end where it's wide enough. Colored as a line is: one color, or `colors` (a Uint8Array or Uint16Array of
// indices into `palette`), each bar its own. Zoomed out (under 3 CSS pixels a point), each pixel column one bar, to its
// point farthest from the base, in that point's color: bars side by side, as they'd show.

import { Line } from "./line.js";
import { NOISE } from "./timeline.js";

/** A rounded rect's corners, at its value's end (top or bottom), made once and set at each draw. */
const ROUND_UP = [0, 0, 0, 0], ROUND_DOWN = [0, 0, 0, 0];
/** Its own extent, read before it widens the pane's. */
const OWN = new Float64Array(2);

/**
 * A bar's footprint, from its base to its value: one with a value (`sign` its side of the base, -1 or 1) a device pixel
 * tall at least, even where its scale clips the base (a log scale's zero); none (0) on the base.
 * @param {CanvasRenderingContext2D} g @param {number} x @param {number} width @param {number} y @param {number} baseY
 * @param {number} sign @param {number} dpr @param {number} pad @param {number} radius
 */
function barShape(g, x, width, y, baseY, sign, dpr, pad, radius) {
  const up = y < baseY || (y === baseY && sign >= 0), height = Math.max(sign ? 1 / dpr : 0, Math.abs(baseY - y));
  const top = (up ? baseY - height : baseY) - pad;
  if (radius) g.roundRect(x, top, width, height + 2 * pad, up ? ROUND_UP : ROUND_DOWN);
  else g.rect(x, top, width, height + 2 * pad);
}

export class Bars extends Line {
  /**
   * `radius`, its corners' at their value's end (a number, or one by the spacing of its points).
   * @param {{ values?: Float64Array, base?: number, color?: string, colors?: Uint8Array | Uint16Array | null, palette?: string[], radius?: number | ((spacing: number) => number), outline?: string | null }} [options]
   */
  constructor({ values, base = 0, color = "#f7f8f8", colors = null, palette = [], radius = 2, outline = null } = {}) {
    // (No crosshair dot on bars.)
    super({ values, color, colors, palette, marker: 0, outline });
    this.base = base;
    this.radius = radius;
  }

  /** Its look; its corners (`radius`) and base too. @param {Parameters<Line["setStyle"]>[0] & { radius?: number | ((spacing: number) => number), base?: number }} look */
  setStyle({ radius = this.radius, base = this.base, ...look }) {
    super.setStyle(look);
    (this.radius = radius), (this.base = base);
  }

  /** @param {number} a @param {number} b @param {boolean} positive @param {Float64Array} out */
  extent(a, b, positive, out) {
    (OWN[0] = Infinity), (OWN[1] = -Infinity);
    super.extent(a, b, positive, OWN);
    if (!(OWN[0] <= OWN[1])) return;
    // (Its base in view too, where it has a bar: the bars stand on it.)
    if (!positive || this.base > 0) (OWN[0] = Math.min(OWN[0], this.base)), (OWN[1] = Math.max(OWN[1], this.base));
    (out[0] = Math.min(out[0], OWN[0])), (out[1] = Math.max(out[1], OWN[1]));
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr
   */
  draw(g, time, scale, x0, x1, dpr) {
    if (!this.values.length) return;
    const colors = this.colors && this.palette.length ? this.colors : null;
    // (Its outline first (each bar a pixel and a half wider all round), all of them one fill; then one fill a color.)
    if (this.outline) (g.fillStyle = this.outline), g.beginPath(), this.#shapes(g, time, scale, x0, x1, dpr, null, -1, Math.max(1.5, 1 / dpr)), g.fill();
    for (let pass = colors ? 0 : -1; pass < (colors ? this.palette.length : 0); pass++) {
      g.fillStyle = pass < 0 ? this.color : this.palette[pass];
      g.beginPath();
      this.#shapes(g, time, scale, x0, x1, dpr, colors, pass, 0);
      g.fill();
    }
  }

  /**
   * Its bars from x `x0` to `x1` added to the path: those of color `pass` (by `colors`; none, all), each `pad` pixels
   * wider all round (an outline's).
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr @param {Uint8Array | Uint16Array | null} colors @param {number} pass @param {number} pad
   */
  #shapes(g, time, scale, x0, x1, dpr, colors, pass, pad) {
    const values = this.values, n = values.length, base = this.base, log = scale.mode === "log", minimum = log ? (base > 0 ? Number.MIN_VALUE : scale.lo) : -Infinity;
    const spacing = time.spacing, wide = spacing >= 3, baseline = log && !(base > 0) ? scale.lo : base, baseY = scale.y(baseline);
    // (A bar's half width, a gap between bars (a pixel, two once there's room), its corners' radius.)
    const corner = typeof this.radius === "function" ? this.radius(spacing) : this.radius;
    const half = Math.max(0.5, (spacing - (spacing > 8 ? 2 : 1)) / 2), radius = wide && corner > 0 ? Math.min(corner, half) + pad : 0;
    ROUND_UP[0] = ROUND_UP[1] = ROUND_DOWN[2] = ROUND_DOWN[3] = radius;
    const a = Math.max(0, Math.floor(time.index(x0)) - 1), b = Math.min(n - 1, Math.ceil(time.index(x1)) + 1);
    if (wide) {
      for (let i = a; i <= b; i++) {
        const v = values[i];
        if (!(v >= minimum) || (colors && colors[i] !== pass)) continue;
        const x = time.x(i), y = scale.y(v);
        // (Its edges on device pixels: bars side by side keep even gaps.)
        const left = time.snap(x - half, dpr) - pad, right = time.snap(x + half, dpr) + pad;
        barShape(g, left, right - left, y, baseY, Math.sign(v - base), dpr, pad, radius);
      }
      return;
    }
    // (A pixel column each: its point farthest from the base.)
    for (let x = Math.floor(x0 - pad) - 1; x <= Math.ceil(x1 + pad) + 1; x++) {
      const from = Math.max(0, Math.ceil(time.index(x) - NOISE)), to = Math.min(n - 1, Math.ceil(time.index(x + 1) - NOISE) - 1);
      let far = -1, most = -1;
      for (let i = from; i <= to; i++) {
        const v = values[i];
        if (!(v >= minimum)) continue;
        const off = Math.abs(v - base);
        if (off > most) (most = off), (far = i);
      }
      if (far < 0 || (colors && colors[far] !== pass)) continue;
      const y = scale.y(values[far]);
      barShape(g, x - pad, 1 + 2 * pad, y, baseY, Math.sign(values[far] - base), dpr, pad, 0);
    }
  }
}
