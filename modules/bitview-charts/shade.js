// @ts-check
// A shade: stretches of the pane's whole height behind what's drawn (a signal's: where it holds), a point's slot from
// halfway to the point before to halfway to the next, in its color (`color`, or `colors`, indices into `palette`, point
// by point: its color changing halfway between two points); a point whose value is NaN (`values`) none, the stretch
// broken there. Zoomed out, a pixel column in its last shaded point's color. A backdrop: not fitted, never hit.

import { NOISE } from "./timeline.js";

/** A slot's edges, shared by two neighbors and on device pixels (no seam between two colors' fills); zoomed out, each
 * column's last shaded point (-1: none): grown, never made again. */
let edges = new Float64Array(64), lasts = new Int32Array(64);

export class Shade {
  /** @param {{ values: Float64Array, color?: string, colors?: Uint8Array | Uint16Array | null, palette?: string[] }} options */
  constructor({ values, color = "rgba(38, 166, 154, 0.12)", colors = null, palette = [] }) {
    /** @type {Float64Array} */
    this.values = values;
    this.color = color;
    /** @type {Uint8Array | Uint16Array | null} */
    this.colors = colors;
    /** @type {string[]} */
    this.palette = palette;
    /** Not drawn. */
    this.hidden = false;
    /** Not read on its side's scale: it has none. */
    this.scaled = false;
    this.version = 0;
  }

  /** @param {{ values: Float64Array, colors?: Uint8Array | Uint16Array | null }} columns */
  setValues({ values, colors = this.colors }) {
    this.version++;
    (this.values = values), (this.colors = colors);
  }

  /** @param {{ color?: string, colors?: Uint8Array | Uint16Array | null, palette?: string[] }} look */
  setStyle({ color = this.color, colors = this.colors, palette = this.palette }) {
    this.version++;
    (this.color = color), (this.colors = colors), (this.palette = palette);
  }

  /** Nothing to fit. */
  extent() {}

  /** Any base: it has no values to read against one (a relative scale draws it all the same). */
  first() {
    return 1;
  }

  /** Never hit: a backdrop. */
  distance() {
    return Infinity;
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr
   */
  draw(g, time, scale, x0, x1, dpr) {
    const values = this.values, n = values.length, colors = this.colors && this.palette.length ? this.colors : null, passes = colors ? this.palette.length : 1;
    if (!n) return;
    // (Zoomed in, a point's slot; zoomed out, a pixel column each, in its last shaded point's color.)
    if (time.spacing >= 1) {
      const a = Math.max(0, Math.floor(time.index(x0)) - 1), b = Math.min(n - 1, Math.ceil(time.index(x1)) + 1);
      if (edges.length < b - a + 2) edges = new Float64Array((b - a + 2) * 2);
      for (let i = a; i <= b + 1; i++) edges[i - a] = time.snap(time.x(i - 0.5), dpr);
      for (let pass = 0; pass < passes; pass++) {
        g.fillStyle = colors ? this.palette[pass] : this.color;
        g.beginPath();
        for (let i = a; i <= b; i++) if (values[i] === values[i] && (!colors || colors[i] === pass)) g.rect(edges[i - a], 0, edges[i - a + 1] - edges[i - a], scale.height);
        g.fill();
      }
      return;
    }
    // (Each column's last shaded point found once, then its columns filled a color at a time.)
    const from = Math.floor(x0) - 1, columns = Math.ceil(x1) + 1 - from + 1;
    if (lasts.length < columns) lasts = new Int32Array(columns * 2);
    for (let c = 0; c < columns; c++) {
      const x = from + c;
      let last = -1;
      for (let i = Math.max(0, Math.ceil(time.index(x) - NOISE)), end = Math.min(n - 1, Math.ceil(time.index(x + 1) - NOISE) - 1); i <= end; i++) if (values[i] === values[i]) last = i;
      lasts[c] = last;
    }
    for (let pass = 0; pass < passes; pass++) {
      g.fillStyle = colors ? this.palette[pass] : this.color;
      g.beginPath();
      for (let c = 0; c < columns; c++) {
        const last = lasts[c], left = time.snap(from + c, dpr);
        if (last >= 0 && (!colors || colors[last] === pass)) g.rect(left, 0, time.snap(from + c + 1, dpr) - left, scale.height);
      }
      g.fill();
    }
  }
}
