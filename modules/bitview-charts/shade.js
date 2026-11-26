// @ts-check

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
    this.hidden = false;
    this.faint = false;
    this.scaled = false;
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

  extent() {}

  first() {
    return 1;
  }

  distance() {
    return Infinity;
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr
   */
  draw(g, time, scale, x0, x1, dpr) {
    const values = this.values, n = values.length, colors = this.colors && this.palette.length ? this.colors : null;
    const from = Math.max(0, Math.floor(time.index(x0 - 4) - 1)), to = Math.min(n, Math.ceil(time.index(x1 + 4) + 1) + 1);
    if (from >= to) return;
    const half = (Math.max(time.spacing, 1) * dpr) / 2, height = scale.height * dpr;
    g.save();
    const { e, f } = g.getTransform();
    g.setTransform(1, 0, 0, 1, e, f);
    let start = NaN, color = "", last = NaN, column = 0;
    const fill = (/** @type {number} */ end) => {
      if (start === start) (g.fillStyle = color), g.fillRect(start, 0, end - start, height);
      start = NaN;
    };
    for (let i = from; i < to; i++) {
      if (values[i] !== values[i]) continue;
      const now = colors ? this.palette[colors[i]] ?? this.color : this.color, at = time.snap(time.x(i), dpr) * dpr;
      if (i !== last + 1) fill(column + half), (start = at - half);
      else if (now !== color) ((middle) => (fill(middle), (start = middle)))(at !== column ? (column + at) / 2 : at);
      (color = now), (last = i), (column = at);
    }
    fill(column + half);
    g.restore();
  }
}
