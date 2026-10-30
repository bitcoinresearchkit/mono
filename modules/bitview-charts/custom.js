// @ts-check
// A series of a page's own: what it draws (bubbles, a heatmap, a drawing, marks at dates), drawn by its hooks as part of
// its pane's picture, in the order added (the last on top): a pan moves it with the rest, a zoom draws it anew. `draw`
// is given the canvas (in CSS pixels, clipped to the strip being drawn) and the pane's view: its timeline (`x(i)`,
// `index(x)`, `spacing`, `from`, `to`, `times`), its scale (`y(v)`, `value(y)`, its mode, its range), the strip (`x0`,
// `x1`) and the device pixels a CSS pixel (`dpr`). Optional: `extent` (its lowest and highest from index `a` to `b`,
// widening `out`: the pane fitted to it too) and `first` (its first value at or after an index: a relative scale's
// base). After a change of what it draws, `changed()`, then the chart's invalidate(true).

/**
 * @typedef {{ time: import("./timeline.js").Timeline, scale: import("./scale.js").Scale, x0: number, x1: number, dpr: number }} View
 * @typedef {{
 *   draw: (g: CanvasRenderingContext2D, view: View) => void,
 *   extent?: (a: number, b: number, positive: boolean, out: Float64Array) => void,
 *   first?: (i: number) => number,
 *   distance?: (i: number, y: number, scale: import("./scale.js").Scale) => number,
 * }} Hooks
 */

export class Custom {
  /** @param {Hooks} hooks */
  constructor(hooks) {
    this.hooks = hooks;
    /** Not drawn, fitted, tagged or hit (its pane drawn whole after a change: its chart's invalidate(true)). */
    this.hidden = false;
    this.version = 0;
    // (Its view, filled at each draw: none made a frame.)
    /** @type {View} */
    this.view = { time: /** @type {any} */ (null), scale: /** @type {any} */ (null), x0: 0, x1: 0, dpr: 1 };
  }

  /** What it draws changed: its pane's picture drawn whole at the next frame. */
  changed() {
    this.version++;
  }

  /** @param {number} a @param {number} b @param {boolean} positive @param {Float64Array} out */
  extent(a, b, positive, out) {
    this.hooks.extent?.(a, b, positive, out);
  }

  /** How far a point is from it (its hook's; none: it's never hit). @param {number} i @param {number} y @param {import("./scale.js").Scale} scale */
  distance(i, y, scale) {
    return this.hooks.distance ? this.hooks.distance(i, y, scale) : Infinity;
  }

  /** @param {number} i */
  first(i) {
    return this.hooks.first ? this.hooks.first(i) : NaN;
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr
   */
  draw(g, time, scale, x0, x1, dpr) {
    const view = this.view;
    (view.time = time), (view.scale = scale), (view.x0 = x0), (view.x1 = x1), (view.dpr = dpr);
    g.save();
    this.hooks.draw(g, view);
    g.restore();
  }
}
