// @ts-check
// Bars: a histogram from one column (a Float64Array on the timeline), each bar from `base` to its value, rounded at its
// value's end where it's wide enough. Colored as a line is: one color, or `colors` (a Uint8Array of indices into
// `palette`), each bar its own. Zoomed out (under 3 CSS pixels a point), each pixel column one bar, to its point
// farthest from the base, in that point's color: bars side by side, as they'd show.

import { Line } from "./line.js";

/** A rounded rect's corners, at its value's end (top or bottom), made once and set at each draw. */
const ROUND_UP = [0, 0, 0, 0], ROUND_DOWN = [0, 0, 0, 0];
/** Its own extent, read before it widens the pane's. */
const OWN = new Float64Array(2);

export class Bars extends Line {
  /**
   * @param {{ values?: Float64Array, base?: number, color?: string, colors?: Uint8Array | null, palette?: string[], radius?: number }} [options]
   */
  constructor({ values, base = 0, color = "#f7f8f8", colors = null, palette = [], radius = 2 } = {}) {
    super({ values, color, colors, palette });
    this.base = base;
    this.radius = radius;
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
    const values = this.values, n = values.length, log = scale.mode === "log", colors = this.colors && this.palette.length ? this.colors : null;
    if (!n) return;
    const spacing = time.spacing, wide = spacing >= 3, baseY = scale.y(log && !(this.base > 0) ? scale.lo : this.base);
    // (A bar's half width, a gap between bars (a pixel, two once there's room), its corners' radius.)
    const half = Math.max(0.5, (spacing - (spacing > 8 ? 2 : 1)) / 2), radius = wide ? Math.min(this.radius, half) : 0;
    ROUND_UP[0] = ROUND_UP[1] = ROUND_DOWN[2] = ROUND_DOWN[3] = radius;
    const a = Math.max(0, Math.floor(time.index(x0)) - 1), b = Math.min(n - 1, Math.ceil(time.index(x1)) + 1);
    for (let pass = colors ? 0 : -1; pass < (colors ? this.palette.length : 0); pass++) {
      g.fillStyle = pass < 0 ? this.color : this.palette[pass];
      g.beginPath();
      if (wide) {
        for (let i = a; i <= b; i++) {
          const v = values[i];
          if (!(log ? v > 0 : v === v) || (colors && colors[i] !== pass)) continue;
          const x = time.x(i), y = scale.y(v), up = y < baseY;
          // (Its edges on device pixels: bars side by side keep even gaps.)
          const left = Math.round((x - half) * dpr) / dpr, right = Math.round((x + half) * dpr) / dpr;
          if (radius) g.roundRect(left, up ? y : baseY, right - left, Math.abs(baseY - y), up ? ROUND_UP : ROUND_DOWN);
          else g.rect(left, up ? y : baseY, right - left, Math.abs(baseY - y));
        }
      } else {
        // (A pixel column each: its point farthest from the base.)
        for (let x = Math.floor(x0) - 1; x <= Math.ceil(x1) + 1; x++) {
          const from = Math.max(0, Math.ceil(time.index(x))), to = Math.min(n - 1, Math.ceil(time.index(x + 1)) - 1);
          let far = -1, most = -1;
          for (let i = from; i <= to; i++) {
            const v = values[i];
            if (!(log ? v > 0 : v === v)) continue;
            const off = Math.abs(v - this.base);
            if (off > most) (most = off), (far = i);
          }
          if (far < 0 || (colors && colors[far] !== pass)) continue;
          const y = scale.y(values[far]);
          g.rect(x, Math.min(y, baseY), 1, Math.max(1 / dpr, Math.abs(baseY - y)));
        }
      }
      g.fill();
    }
  }
}
