// @ts-check
// The crosshair: over the whole chart, its own canvas (the panes' pictures never drawn again for it): a line down every
// pane at a point (the nearest to where it's pointed), a line across the pane pointed at, at the pointer's height; the
// point's date on the time axis and the height's value on that pane's axis, each in a badge.

import { kept } from "./format.js";
import { inkOn } from "./color.js";

const DAY = 86400, MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

export class Crosshair {
  constructor() {
    this.canvas = document.createElement("canvas");
    this.canvas.style.cssText = "position: absolute; left: 0; top: 0; display: block; pointer-events: none";
    const g = this.canvas.getContext("2d");
    if (!g) throw new Error("bitview-charts: no 2D canvas");
    this.g = g;
    /** The point it's on (a logical index), or NaN: none shown. */
    this.index = NaN;
    /** Its height in the chart (CSS pixels), and the pane it's in (-1: none, on the time axis). */
    this.y = NaN;
    this.pane = -1;
    this.width = 0;
    this.height = 0;
    // (As drawn: nothing to clear when it's gone already.)
    this.shown = false;
    // (Each date written once: a day's, or a day's and its time's where points are less than a day apart.)
    this.day = kept((t) => dateOf(t, false));
    this.fine = kept((t) => dateOf(t, true));
  }

  /** @param {number} width @param {number} height @param {number} dpr */
  place(width, height, dpr) {
    const w = Math.round(width * dpr), h = Math.round(height * dpr);
    if (this.canvas.width !== w || this.canvas.height !== h) (this.canvas.width = w), (this.canvas.height = h);
    Object.assign(this.canvas.style, { width: `${width}px`, height: `${height}px` });
    this.width = width;
    this.height = height;
  }

  /**
   * Drawn for where it is now (nothing, if it's not shown).
   * @param {import("./timeline.js").Timeline} time @param {import("./pane.js").Pane[]} panes @param {number} room the panes' height
   * @param {number} left the left axis' width (the plot after it) @param {import("./pane.js").Theme} theme @param {number} dpr
   */
  draw(time, panes, room, left, theme, dpr) {
    const g = this.g, i = this.index;
    if (i !== i) {
      if (this.shown) g.setTransform(1, 0, 0, 1, 0, 0), g.clearRect(0, 0, this.canvas.width, this.canvas.height), (this.shown = false);
      return;
    }
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    g.clearRect(0, 0, this.width, this.height);
    g.translate(left, 0);
    this.shown = true;
    const plot = time.width, x = Math.round(time.x(i) * dpr) / dpr + 0.5 / dpr;
    g.strokeStyle = theme.crosshair;
    g.lineWidth = 1;
    g.setLineDash(DASH);
    g.beginPath();
    if (x >= 0 && x <= plot) g.moveTo(x, 0), g.lineTo(x, room);
    const pane = panes[this.pane];
    if (pane && this.y >= pane.top && this.y <= pane.top + pane.height) {
      const y = Math.round(this.y * dpr) / dpr + 0.5 / dpr;
      g.moveTo(0, y), g.lineTo(plot, y);
    }
    g.stroke();
    g.setLineDash(NONE);
    g.font = theme.font;
    g.textBaseline = "middle";
    // (Its value on its pane's axis.)
    if (pane && this.y >= pane.top && this.y <= pane.top + pane.height) {
      badge(g, theme, pane.axisLabel(round(pane.scale.value(this.y - pane.top))), plot + 2, this.y, "left");
      // (Its value on the left axis too, where the pane has a left scale.)
      if (pane.left && left) badge(g, theme, pane.axisLabel(round(pane.left.value(this.y - pane.top)), pane.left), -left + 2, this.y, "left");
    }
    // (Its date on the time axis, its point's.)
    const times = time.times, k = Math.round(i);
    if (k >= 0 && k < times.length && x >= 0 && x <= plot) {
      // (Points less than a day apart (the last one's from the one before it): its time too.)
      const fine = times.length > 1 && (k < times.length - 1 ? times[k + 1] - times[k] : times[k] - times[k - 1]) < DAY;
      badge(g, theme, (fine ? this.fine : this.day)(times[k]), x, room + 13, "center");
    }
  }
}

/** A crosshair's dash, and none (made once). */
const DASH = [4, 4], NONE = /** @type {number[]} */ ([]);

/** A value as a badge shows it: four significant digits, no more. @param {number} v */
function round(v) {
  return v === 0 ? 0 : Number(v.toPrecision(4));
}

/**
 * Text in a badge at `x`, `y` (CSS pixels): its left there, or its middle.
 * @param {CanvasRenderingContext2D} g @param {import("./pane.js").Theme} theme @param {string} text @param {number} x @param {number} y
 * @param {"left" | "center"} align
 */
function badge(g, theme, text, x, y, align) {
  const width = g.measureText(text).width + 12, left = align === "center" ? x - width / 2 : x;
  g.fillStyle = theme.badge;
  g.beginPath();
  g.roundRect(left, y - 10, width, 20, 4);
  g.fill();
  g.fillStyle = inkOn(theme.badge);
  g.textAlign = "center";
  g.fillText(text, left + width / 2, y);
}

/** A point's date (UTC): its day, and its time with `fine`. @param {number} t @param {boolean} fine */
function dateOf(t, fine) {
  const d = new Date(t * 1000), day = `${d.getUTCDate()} ${MONTHS[d.getUTCMonth()]} ${d.getUTCFullYear()}`;
  return fine ? `${day} ${String(d.getUTCHours()).padStart(2, "0")}:${String(d.getUTCMinutes()).padStart(2, "0")}` : day;
}
