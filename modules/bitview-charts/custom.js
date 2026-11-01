// @ts-check
// A series of a page's own: what it draws (bubbles, a heatmap, a drawing, marks at dates), drawn by its hooks as part
// of its pane's picture, in the order added (the last on top): a pan moves it with the rest, a zoom draws it anew.
// `draw` is given the canvas (in CSS pixels, clipped to the strip being drawn) and the pane's view: its timeline
// (`x(i)`, `index(x)`, `spacing`, `from`, `to`, `times`), its scale (`y(v)`, `value(y)`, its mode, its range), the
// strip (`x0`, `x1`), the device pixels a CSS pixel (`dpr`) and the chart's theme (`theme`: its ground, its font). Optional: `extent` (its lowest and highest from index
// `a` to `b`, widening `out`: the pane fitted to it too), `first` (its first value at or after an index: a relative
// scale's base) and `distance` (how far a point is from it, in CSS pixels: at index `i`, `x` into the plot, `y` from
// the pane's top; none, it's never hit), `axisTags` (its own tags on a price axis: its side's, 0 the right, 1 the left;
// each at a height in its pane, its words, its color; asked as the axis is drawn: whole, or its tags' view's end moved,
// so a change of them is a change of it, see changed()), `timeTags` (on the time axis, at an x in the plot: asked each
// frame), `margins` (room it keeps clear at its scale's ends, by the points' spacing: CSS pixels over its highest and
// under its lowest, into `out`: its marks over and under bars) and `viewed` (told before each draw of its pane, the
// view as it is: what it draws found again from it, once). Its options: `fixed` (what it draws holds its place
// as the view moves, a mark at the plot's edge: its pane drawn whole each frame), `over` (drawn over everything, the
// crosshair too, on its pane's layer over it, whole each frame: labels and handles placed in the plot, its pane's
// picture moved without them) or `behind` (drawn behind every series, whatever their order: a backdrop). After a
// change of what it draws, `changed()`, then the chart's invalidate().

/**
 * @typedef {{ time: import("./timeline.js").Timeline, scale: import("./scale.js").Scale, x0: number, x1: number, dpr: number, theme: import("./pane.js").Theme }} View
 * @typedef {{
 *   draw: (g: CanvasRenderingContext2D, view: View) => void,
 *   extent?: (a: number, b: number, positive: boolean, out: Float64Array) => void,
 *   first?: (i: number) => number,
 *   distance?: (i: number, y: number, scale: import("./scale.js").Scale, x: number) => number,
 *   axisTags?: (side: number, add: (y: number, text: string, color: string, ink?: string) => void, scale: import("./scale.js").Scale, stepAt: (v: number) => number) => void,
 *   timeTags?: (add: (x: number, text: string, color: string, ink?: string) => void) => void,
 *   margins?: (spacing: number, out: Float64Array, time: import("./timeline.js").Timeline, positive: boolean) => void,
 *   viewed?: (time: import("./timeline.js").Timeline) => void,
 * }} Hooks
 */

export class Custom {
  /** @param {Hooks} hooks @param {{ fixed?: boolean, over?: boolean, behind?: boolean }} [options] */
  constructor(hooks, { fixed = false, over = false, behind = false } = {}) {
    this.hooks = hooks;
    /** Not drawn, fitted, tagged or hit (its pane drawn whole after a change: its chart's invalidate(true)). */
    this.hidden = false;
    /** Drawn at a fifth of its opacity, its tags gone (another lit over it: see the pane's Series). */
    this.faint = false;
    /** A scale of its own (see the pane's Series), none: its side's. @type {import("./scale.js").Scale | null} */
    this.overlay = null;
    /** What it draws holds its place as the view moves: its pane drawn whole each frame. */
    this.fixed = fixed;
    /** Drawn over everything, on its pane's layer over it, whole each frame. */
    this.over = over;
    /** Drawn behind every series, whatever their order. */
    this.behind = behind;
    /** How it widens its scale's range in its place (a page's own reading of it), none: by its extent. @type {((a: number, b: number, positive: boolean, out: Float64Array) => void) | null} */
    this.fit = null;
    /** Read on its side's scale: it has an extent (else, drawn as it likes, its pane's grid isn't its side's for it). */
    this.scaled = Boolean(hooks.extent);
    this.version = 0;
    // (Its view, filled at each draw: none made a frame.)
    /** @type {View} */
    this.view = { time: /** @type {any} */ (null), scale: /** @type {any} */ (null), x0: 0, x1: 0, dpr: 1, theme: /** @type {any} */ (null) };
  }

  /** What it draws changed: its pane's picture drawn whole at the next frame. */
  changed() {
    this.version++;
  }

  /** @param {number} a @param {number} b @param {boolean} positive @param {Float64Array} out */
  extent(a, b, positive, out) {
    this.hooks.extent?.(a, b, positive, out);
  }

  /**
   * The room it keeps clear at its scale's ends (its hook's: none, none).
   * @param {number} spacing @param {Float64Array} out @param {import("./timeline.js").Timeline} time @param {boolean} positive
   */
  margins(spacing, out, time, positive) {
    this.hooks.margins?.(spacing, out, time, positive);
  }

  /** How far a point is from it (its hook's; none: it's never hit). @param {number} i @param {number} y @param {import("./scale.js").Scale} scale @param {number} x */
  distance(i, y, scale, x) {
    return this.hooks.distance ? this.hooks.distance(i, y, scale, x) : Infinity;
  }

  /**
   * Its own tags on its pane's axis on `side` (0, the right; 1, the left), that side's scale given, and the step between
   * its ticks around a value (a value written by them, as every one on an axis).
   * @param {number} side @param {(y: number, text: string, color: string, ink?: string) => void} add @param {import("./scale.js").Scale} scale
   * @param {(v: number) => number} stepAt
   */
  axisTags(side, add, scale, stepAt) {
    this.hooks.axisTags?.(side, add, scale, stepAt);
  }

  /** Told before each draw of its pane (its hook's). @param {import("./timeline.js").Timeline} time */
  viewed(time) {
    this.hooks.viewed?.(time);
  }

  /** Its own tags on the time axis. @param {(x: number, text: string, color: string, ink?: string) => void} add */
  timeTags(add) {
    this.hooks.timeTags?.(add);
  }

  /** Its first value at or after `i` (a relative scale's base), its hook's; none, any (its drawing reads no values against one). @param {number} i */
  first(i) {
    return this.hooks.first ? this.hooks.first(i) : 1;
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr @param {import("./pane.js").Theme} theme
   */
  draw(g, time, scale, x0, x1, dpr, theme) {
    const view = this.view;
    (view.time = time), (view.scale = scale), (view.x0 = x0), (view.x1 = x1), (view.dpr = dpr), (view.theme = theme);
    g.save();
    this.hooks.draw(g, view);
    g.restore();
  }
}
