// @ts-check
// Dashes keep their place on the timeline: a pan's moved picture has them where a whole draw does.
// After a change of its lines, `set` (or `changed()`), then the chart's invalidate(true).

import { FAINT, periodOf } from "./line.js";
import { compact } from "./format.js";

/** @typedef {{ value: number, color: string, width?: number, dash?: number[], tag?: boolean, hidden?: boolean, faint?: boolean }} Level */

export class Levels {
  /** @param {{ lines?: Level[], side?: "right" | "left", format?: ((v: number, step?: number) => string) | null }} [options] */
  constructor({ lines = [], side = "right", format = null } = {}) {
    /** @type {Level[]} */
    this.lines = lines;
    this.side = side;
    this.format = format;
    this.hidden = false;
    this.behind = true;
    this.scaled = false;
    this.version = 0;
  }

  /** @param {{ lines?: Level[], format?: ((v: number, step?: number) => string) | null }} change */
  set({ lines = this.lines, format = this.format }) {
    this.version++;
    (this.lines = lines), (this.format = format);
  }

  /** A dashed line's picture is never stretched by a zoom: its dashes would be. */
  get dashed() {
    return this.lines.some((line) => !line.hidden && line.dash?.length);
  }

  changed() {
    this.version++;
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
   * @param {number} side @param {(y: number, text: string, color: string) => void} add @param {import("./scale.js").Scale} scale
   * @param {(v: number) => number} [stepAt]
   */
  axisTags(side, add, scale, stepAt) {
    if ((this.side === "left" ? 1 : 0) !== side) return;
    for (const line of this.lines) if (line.tag && !line.hidden && !line.faint) add(scale.y(line.value), (this.format ?? compact)(line.value, stepAt ? stepAt(line.value) : NaN), line.color);
  }
}
