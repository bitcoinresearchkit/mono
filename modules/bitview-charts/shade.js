// @ts-check
// A shade: stretches of the pane's whole height behind what's drawn (a signal's: where it holds), in its color (`color`,
// or `colors`, indices into `palette`, point by point); a point whose value is NaN (`values`) none, the stretch broken
// there. A stretch runs from half a point's room before its first point to half after its last (a point's room: the
// spacing, a pixel at least: a point alone, zoomed out, a sliver), its color changing halfway between two points, on
// device-pixel columns: the same at any zoom, a stretch however short shown. A backdrop: not fitted, never hit.

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
    /** Drawn at a fifth of its opacity (another lit over it: see the pane's Series). */
    this.faint = false;
    /** Not read on its side's scale: it has none. */
    this.scaled = false;
    /** Drawn behind every series, whatever their order: a backdrop. */
    this.behind = true;
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
   * Its stretches from x `x0` to `x1` (and a point past each), in device pixels: each one filled as it ends (a gap, a
   * color changing, the last point).
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr
   */
  draw(g, time, scale, x0, x1, dpr) {
    const values = this.values, n = values.length, colors = this.colors && this.palette.length ? this.colors : null;
    const from = Math.max(0, Math.floor(time.index(x0 - 4) - 1)), to = Math.min(n, Math.ceil(time.index(x1 + 4) + 1) + 1);
    if (from >= to) return;
    const half = (Math.max(time.spacing, 1) * dpr) / 2, height = scale.height * dpr;
    g.save();
    g.setTransform(1, 0, 0, 1, 0, 0);
    // (The stretch being made: where it starts (NaN: none), its color; the last point's index and column.)
    let start = NaN, color = "", last = NaN, column = 0;
    const fill = (/** @type {number} */ end) => {
      if (start === start) (g.fillStyle = color), g.fillRect(start, 0, end - start, height);
      start = NaN;
    };
    for (let i = from; i < to; i++) {
      if (values[i] !== values[i]) continue;
      const now = colors ? this.palette[colors[i]] ?? this.color : this.color, at = time.snap(time.x(i), dpr) * dpr;
      if (i !== last + 1) fill(column + half), (start = at - half);
      // (Its color changing: halfway between the two points' columns, or at the column both are in.)
      else if (now !== color) ((middle) => (fill(middle), (start = middle)))(at !== column ? (column + at) / 2 : at);
      (color = now), (last = i), (column = at);
    }
    fill(column + half);
    g.restore();
  }
}
