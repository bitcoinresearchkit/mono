// @ts-check
// `axisTags` are asked only as the axis is drawn, so a change of them is a change of it; `timeTags` each frame.
// After a change of what it draws, `changed()`, then the chart's invalidate().

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
  /** @param {Hooks} hooks @param {{ fixed?: boolean, over?: boolean, behind?: boolean, crosshair?: boolean }} [options] */
  constructor(hooks, { fixed = false, over = false, behind = false, crosshair = false } = {}) {
    this.hooks = hooks;
    this.hidden = false;
    this.faint = false;
    /** @type {import("./scale.js").Scale | null} */
    this.overlay = null;
    /** Holds its place as the view moves: its pane redraws on view or content changes. */
    this.fixed = fixed;
    /** On its pane's layer over it, whole each frame. */
    this.over = over;
    this.behind = behind;
    /** Never in its pane's picture. */
    this.crosshair = crosshair;
    /** @type {((a: number, b: number, positive: boolean, out: Float64Array) => void) | null} */
    this.fit = null;
    /** Only with an extent: else its pane's grid isn't its side's for it. */
    this.scaled = Boolean(hooks.extent);
    this.version = 0;
    /** @type {View} */
    this.view = { time: /** @type {any} */ (null), scale: /** @type {any} */ (null), x0: 0, x1: 0, dpr: 1, theme: /** @type {any} */ (null) };
  }

  changed() {
    this.version++;
  }

  /** @param {number} a @param {number} b @param {boolean} positive @param {Float64Array} out */
  extent(a, b, positive, out) {
    this.hooks.extent?.(a, b, positive, out);
  }

  /**
   * @param {number} spacing @param {Float64Array} out @param {import("./timeline.js").Timeline} time @param {boolean} positive
   */
  margins(spacing, out, time, positive) {
    this.hooks.margins?.(spacing, out, time, positive);
  }

  /** @param {number} i @param {number} y @param {import("./scale.js").Scale} scale @param {number} x */
  distance(i, y, scale, x) {
    return this.hooks.distance ? this.hooks.distance(i, y, scale, x) : Infinity;
  }

  /**
   * @param {number} side @param {(y: number, text: string, color: string, ink?: string) => void} add @param {import("./scale.js").Scale} scale
   * @param {(v: number) => number} stepAt
   */
  axisTags(side, add, scale, stepAt) {
    this.hooks.axisTags?.(side, add, scale, stepAt);
  }

  /** @param {import("./timeline.js").Timeline} time */
  viewed(time) {
    this.hooks.viewed?.(time);
  }

  /** @param {(x: number, text: string, color: string, ink?: string) => void} add */
  timeTags(add) {
    this.hooks.timeTags?.(add);
  }

  /** @param {number} i */
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
