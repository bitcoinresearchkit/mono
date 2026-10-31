// @ts-check
// A series of a page's own: what it draws (bubbles, a heatmap, a drawing, marks at dates), drawn by its hooks as part of
// its pane's picture, in the order added (the last on top): a pan moves it with the rest, a zoom draws it anew. `draw`
// is given the canvas (in CSS pixels, clipped to the strip being drawn) and the pane's view: its timeline (`x(i)`,
// `index(x)`, `spacing`, `from`, `to`, `times`), its scale (`y(v)`, `value(y)`, its mode, its range), the strip (`x0`,
// `x1`) and the device pixels a CSS pixel (`dpr`). Optional: `extent` (its lowest and highest from index `a` to `b`,
// widening `out`: the pane fitted to it too), `first` (its first value at or after an index: a relative scale's base)
// and `distance` (how far a point is from it, in CSS pixels: at index `i`, `x` into the plot, `y` from the pane's top;
// none, it's never hit), `axisTags` (its own tags on a price axis: its side's, 0 the right, 1 the left; each at a
// height in its pane, its words, its color; asked as the axis is drawn: whole, or its tags' view's end moved, so a
// change of them is a change of it, see changed()) and `timeTags` (on the time axis, at an x in the plot: asked each
// frame). Its options: `fixed` (what it draws holds its place as the view moves, a mark at the plot's
// edge: its pane drawn whole each frame) or `over` (drawn over everything, the crosshair too, on its pane's layer over
// it, whole each frame: labels and handles placed in the plot, its pane's picture moved without them). After a change
// of what it draws, `changed()`, then the chart's invalidate().

/**
 * @typedef {{ time: import("./timeline.js").Timeline, scale: import("./scale.js").Scale, x0: number, x1: number, dpr: number }} View
 * @typedef {{
 *   draw: (g: CanvasRenderingContext2D, view: View) => void,
 *   extent?: (a: number, b: number, positive: boolean, out: Float64Array) => void,
 *   first?: (i: number) => number,
 *   distance?: (i: number, y: number, scale: import("./scale.js").Scale, x: number) => number,
 *   axisTags?: (side: number, add: (y: number, text: string, color: string, ink?: string) => void) => void,
 *   timeTags?: (add: (x: number, text: string, color: string, ink?: string) => void) => void,
 * }} Hooks
 */

export class Custom {
  /** @param {Hooks} hooks @param {{ fixed?: boolean, over?: boolean }} [options] */
  constructor(hooks, { fixed = false, over = false } = {}) {
    this.hooks = hooks;
    /** Not drawn, fitted, tagged or hit (its pane drawn whole after a change: its chart's invalidate(true)). */
    this.hidden = false;
    /** What it draws holds its place as the view moves: its pane drawn whole each frame. */
    this.fixed = fixed;
    /** Drawn over everything, on its pane's layer over it, whole each frame. */
    this.over = over;
    /** Read on its side's scale: it has an extent (else, drawn as it likes, its pane's grid isn't its side's for it). */
    this.scaled = Boolean(hooks.extent);
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

  /** How far a point is from it (its hook's; none: it's never hit). @param {number} i @param {number} y @param {import("./scale.js").Scale} scale @param {number} x */
  distance(i, y, scale, x) {
    return this.hooks.distance ? this.hooks.distance(i, y, scale, x) : Infinity;
  }

  /** Its own tags on its pane's axis on `side` (0, the right; 1, the left). @param {number} side @param {(y: number, text: string, color: string, ink?: string) => void} add */
  axisTags(side, add) {
    this.hooks.axisTags?.(side, add);
  }

  /** Its own tags on the time axis. @param {(x: number, text: string, color: string, ink?: string) => void} add */
  timeTags(add) {
    this.hooks.timeTags?.(add);
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
