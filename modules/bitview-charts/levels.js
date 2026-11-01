// @ts-check
// Lines across a pane at values, on its side's scale (a page's levels: a series' thresholds, a baseline's base): each
// `{ value, color, width, dash, tag, hidden, faint }` (`dash`, its dashes: empty, solid), dashed where its dashes would be
// on the timeline (a pan's moved picture has them where a whole draw does); its value tagged on its side's axis (`side`,
// as it's added) in its color where it asks (`tag`), written by `format` (none: as a scale writes values, by its ticks); a faint one (a spotlight's)
// at FAINT, its tag gone. Behind every series (`behind`), never fitted (its scale reads what it marks) or hit. After a
// change of its lines, `set` (or `changed()`), then the chart's invalidate(true).

import { FAINT, periodOf } from "./line.js";
import { compact } from "./format.js";

/** @typedef {{ value: number, color: string, width?: number, dash?: number[], tag?: boolean, hidden?: boolean, faint?: boolean }} Level */

export class Levels {
  /** @param {{ lines?: Level[], side?: "right" | "left", format?: ((v: number, step?: number) => string) | null }} [options] */
  constructor({ lines = [], side = "right", format = null } = {}) {
    /** @type {Level[]} */
    this.lines = lines;
    /** The axis its tags are on: its side's, as it's added. */
    this.side = side;
    this.format = format;
    this.hidden = false;
    /** Drawn behind every series, whatever their order. */
    this.behind = true;
    /** Not read on its side's scale: it has no range of its own. */
    this.scaled = false;
    this.version = 0;
  }

  /** @param {{ lines?: Level[], format?: ((v: number, step?: number) => string) | null }} change */
  set({ lines = this.lines, format = this.format }) {
    this.version++;
    (this.lines = lines), (this.format = format);
  }

  /** Whether a line shown is dashed: its picture never stretched by a zoom (its dashes would be). */
  get dashed() {
    return this.lines.some((line) => !line.hidden && line.dash?.length);
  }

  /** Its lines' looks changed in place (one faint, one hidden). */
  changed() {
    this.version++;
  }

  /** Never fitted to. */
  extent() {}

  /** Any base. */
  first() {
    return 1;
  }

  /** Never hit. */
  distance() {
    return Infinity;
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr
   */
  draw(g, time, scale, x0, x1, dpr) {
    // (Faint as a whole (a spotlight's on it), each line's own fading under that.)
    const alpha = g.globalAlpha;
    for (const line of this.lines) {
      const y = Math.round(scale.y(line.value) * dpr) / dpr + 0.5 / dpr, dash = line.dash ?? [];
      if (line.hidden || y !== y) continue;
      g.globalAlpha = line.faint ? alpha * FAINT : alpha;
      g.strokeStyle = line.color;
      g.lineWidth = line.width ?? 1;
      g.setLineDash(dash);
      g.lineDashOffset = dash.length ? (x0 + time.from * time.spacing) % periodOf(dash) : 0;
      g.beginPath();
      g.moveTo(x0, y);
      g.lineTo(x1, y);
      g.stroke();
    }
    g.globalAlpha = alpha;
    g.setLineDash([]);
    g.lineDashOffset = 0;
  }

  /**
   * Its lines' values on its axis (`side`: 0, the right; 1, the left), each in its color, where they ask: written by its
   * axis's ticks around them (`stepAt`), as every value on an axis is.
   * @param {number} side @param {(y: number, text: string, color: string) => void} add @param {import("./scale.js").Scale} scale
   * @param {(v: number) => number} [stepAt]
   */
  axisTags(side, add, scale, stepAt) {
    if ((this.side === "left" ? 1 : 0) !== side) return;
    for (const line of this.lines) if (line.tag && !line.hidden && !line.faint) add(scale.y(line.value), (this.format ?? compact)(line.value, stepAt ? stepAt(line.value) : NaN), line.color);
  }
}
