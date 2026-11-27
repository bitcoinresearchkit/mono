// @ts-check

import { Line } from "./line.js";
import { NOISE } from "./timeline.js";
import { barStyle } from "./bar-style.js";

const ROUND_UP = [0, 0, 0, 0], ROUND_DOWN = [0, 0, 0, 0];
export const PARTIAL = 0.4;
const OWN = new Float64Array(2);
/** Found once a draw, every color's pass reading it. */
let FAR = new Int32Array(0), FAR_FROM = 0;

/**
 * @param {CanvasRenderingContext2D} g @param {number} x @param {number} width @param {number} y @param {number} baseY
 * @param {number} sign @param {number} dpr @param {number} pad @param {number} radius
 */
function barShape(g, x, width, y, baseY, sign, dpr, pad, radius) {
  if (!sign) return;
  const up = sign > 0, height = Math.max(1 / dpr, up ? baseY - y : y - baseY);
  const top = (up ? baseY - height : baseY) - pad;
  if (radius) g.roundRect(x, top, width, height + 2 * pad, up ? ROUND_UP : ROUND_DOWN);
  else g.rect(x, top, width, height + 2 * pad);
}

export class Bars extends Line {
  /**
   * @param {{ values?: Float64Array, base?: number, color?: string, colors?: Uint8Array | Uint16Array | null, palette?: string[], gradient?: import("./color.js").Gradient | null, radius?: number | ((spacing: number) => number), outline?: string | null, partial?: boolean }} [options]
   */
  constructor({ values, base = 0, color = "#f7f8f8", colors = null, palette = [], gradient = null, radius = 2, outline = null, partial = false } = {}) {
    super({ values, color, colors, palette, gradient, marker: 0, outline, partial });
    this.base = base;
    this.radius = radius;
  }

  /** @param {Parameters<Line["setStyle"]>[0] & { radius?: number | ((spacing: number) => number), base?: number }} look */
  setStyle({ radius = this.radius, base = this.base, ...look }) {
    super.setStyle(look);
    (this.radius = radius), (this.base = base);
  }

  /** @param {number} a @param {number} b @param {boolean} positive @param {Float64Array} out */
  extent(a, b, positive, out) {
    (OWN[0] = Infinity), (OWN[1] = -Infinity);
    super.extent(a, b, positive, OWN);
    if (!(OWN[0] <= OWN[1])) return;
    if (!positive || this.base > 0) (OWN[0] = Math.min(OWN[0], this.base)), (OWN[1] = Math.max(OWN[1], this.base));
    (out[0] = Math.min(out[0], OWN[0])), (out[1] = Math.max(out[1], OWN[1]));
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr
   */
  draw(g, time, scale, x0, x1, dpr) {
    if (!this.values.length) return;
    const colors = this.pointColors(), palette = this.pointPalette(), partial = time.spacing >= 3 ? this.partialAt() : -1;
    if (time.spacing < 3) this.#farthest(time, scale, Math.floor(x0) - 3, Math.ceil(x1) + 3);
    if (this.outline) (g.fillStyle = this.outline), g.beginPath(), this.#shapes(g, time, scale, x0, x1, dpr, null, -1, Math.max(1.5, 1 / dpr), partial), g.fill();
    for (let pass = colors ? 0 : -1; pass < (colors ? palette.length : 0); pass++) {
      g.fillStyle = pass < 0 ? this.color : palette[pass];
      g.beginPath();
      this.#shapes(g, time, scale, x0, x1, dpr, colors, pass, 0, partial);
      g.fill();
    }
    if (partial < 0 || time.x(partial) < x0 - time.spacing || time.x(partial) > x1 + time.spacing) return;
    const alpha = g.globalAlpha;
    g.globalAlpha = alpha * PARTIAL;
    if (this.outline) (g.fillStyle = this.outline), g.beginPath(), this.#shapes(g, time, scale, x0, x1, dpr, null, -1, Math.max(1.5, 1 / dpr), -1, partial), g.fill();
    g.fillStyle = this.colorAt(partial);
    g.beginPath();
    this.#shapes(g, time, scale, x0, x1, dpr, null, -1, 0, -1, partial);
    g.fill();
    g.globalAlpha = alpha;
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr @param {Uint8Array | Uint16Array | null} colors @param {number} pass @param {number} pad
   * @param {number} [skip] none: -1 @param {number} [only] all: -1
   */
  #shapes(g, time, scale, x0, x1, dpr, colors, pass, pad, skip = -1, only = -1) {
    const values = this.values, n = values.length, base = this.base, log = scale.mode === "log";
    const spacing = time.spacing, wide = spacing >= 3, baseY = scale.y(log && !(base > 0) ? scale.lo : base);
    const style = barStyle(spacing, typeof this.radius === "function" ? this.radius(spacing) : this.radius);
    const half = style.width / 2, radius = style.radius > 0 ? style.radius + pad : 0;
    ROUND_UP[0] = ROUND_UP[1] = ROUND_DOWN[2] = ROUND_DOWN[3] = radius;
    const a = Math.max(0, Math.floor(time.index(x0)) - 1), b = Math.min(n - 1, Math.ceil(time.index(x1)) + 1);
    if (wide) {
      for (let i = only < 0 ? a : Math.max(a, only); i <= (only < 0 ? b : Math.min(b, only)); i++) {
        const v = values[i];
        if (!(log ? v > 0 : v === v) || i === skip || (colors && colors[i] !== pass)) continue;
        const x = time.x(i), y = scale.y(v);
        const left = time.snap(x - half, dpr) - pad, right = time.snap(x + half, dpr) + pad;
        barShape(g, left, right - left, y, baseY, Math.sign(v - base), dpr, pad, radius);
      }
      return;
    }
    for (let x = Math.floor(x0 - pad) - 1; x <= Math.ceil(x1 + pad) + 1; x++) {
      const far = FAR[x - FAR_FROM];
      if (far < 0 || (colors && colors[far] !== pass)) continue;
      const y = scale.y(values[far]);
      barShape(g, x - pad, 1 + 2 * pad, y, baseY, Math.sign(values[far] - base), dpr, pad, 0);
    }
  }

  /** @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale @param {number} from @param {number} to */
  #farthest(time, scale, from, to) {
    const values = this.values, n = values.length, base = this.base, log = scale.mode === "log";
    if (FAR.length < to - from + 1) FAR = new Int32Array(to - from + 1);
    FAR_FROM = from;
    for (let x = from; x <= to; x++) {
      const a = Math.max(0, Math.ceil(time.index(x) - NOISE)), b = Math.min(n - 1, Math.ceil(time.index(x + 1) - NOISE) - 1);
      let far = -1, most = -1;
      for (let i = a; i <= b; i++) {
        const v = values[i];
        if (!(log ? v > 0 : v === v)) continue;
        const off = Math.abs(v - base);
        if (off > most) (most = off), (far = i);
      }
      FAR[x - from] = far;
    }
  }
}
