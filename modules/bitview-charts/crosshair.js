// @ts-check
// The crosshair: over the whole chart, its own canvas (the panes' pictures never drawn again for it): a line down every
// pane at a point (the nearest to where it's pointed), a line across the pane pointed at, at the pointer's height; the
// point's date on the time axis and the height's value on that pane's axis, each in a badge. And a separator between
// panes pointed at or dragged, along it in the theme's active color.

import { kept } from "./format.js";
import { Line, FAINT } from "./line.js";
import { inkOn } from "./color.js";
import { pill } from "./tags.js";

const DAY = 86400, MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

export class Crosshair {
  constructor() {
    this.canvas = document.createElement("canvas");
    this.canvas.style.cssText = "position: absolute; left: 0; top: 0; z-index: 1; display: block; pointer-events: none";
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
    // (Each date written once: a day's, or a day's and its time's where points are less than a day apart; or as a page
    // writes them, see setFormat.)
    this.day = kept((t) => dateOf(t, false));
    this.fine = kept((t) => dateOf(t, true));
  }

  /** Its dates written by `format` (a point's time, seconds), whatever their points' spacing. @param {(t: number) => string} format */
  setFormat(format) {
    this.day = this.fine = kept(format);
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
   * @param {import("./axis.js").TimeAxis} axis the time axis under it (its labels as drawn)
   * @param {import("./pane.js").Pane | null} [lit] the separator pointed at or dragged (the pane over it; a pane gone or the
   * last, none: nothing under it to separate)
   */
  draw(time, panes, room, left, theme, dpr, axis, lit = null) {
    const g = this.g, i = this.index, litAt = lit ? panes.indexOf(lit) : -1, over = litAt >= 0 && litAt < panes.length - 1 ? lit : null;
    if (i !== i && !over) {
      if (this.shown) g.setTransform(1, 0, 0, 1, 0, 0), g.clearRect(0, 0, this.canvas.width, this.canvas.height), (this.shown = false);
      return;
    }
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    g.clearRect(0, 0, this.width, this.height);
    this.shown = true;
    // (The separator lit: along it, the chart's whole width, as wide as a line, over its own (a device pixel at its
    // pane's foot).)
    if (over) {
      const y = Math.round((over.top + over.height) * dpr) / dpr - 0.5;
      (g.strokeStyle = theme.active), (g.lineWidth = 1), g.beginPath(), g.moveTo(0, y), g.lineTo(this.width, y), g.stroke();
    }
    if (i !== i) return;
    g.translate(left, 0);
    const plot = time.width, x = Math.round(time.x(i) * dpr) / dpr + 0.5 / dpr;
    // (Its line down, solid; its line across its pane, dashed, fainter.)
    g.lineWidth = 1;
    if (x >= 0 && x <= plot) (g.strokeStyle = theme.crosshair), g.beginPath(), g.moveTo(x, 0), g.lineTo(x, room), g.stroke();
    const pane = panes[this.pane];
    if (pane && this.y >= pane.top && this.y <= pane.top + pane.height) {
      const y = Math.round(this.y * dpr) / dpr + 0.5 / dpr;
      g.strokeStyle = theme.across;
      g.setLineDash(DASH);
      g.beginPath();
      g.moveTo(0, y), g.lineTo(plot, y);
      g.stroke();
      g.setLineDash(NONE);
    }
    // (Its dot on each line at its point (a line's `marker` its radius), ringed in the ground, within its pane's plot.)
    if (x >= 0 && x <= plot) {
      const from = Math.max(0, Math.floor(time.from));
      for (let p = 0; p < panes.length; p++) {
        const each = panes[p];
        g.save();
        g.beginPath();
        g.rect(0, each.top, plot, each.height);
        g.clip();
        for (let s = 0; s < each.series.length; s++) {
          const line = each.series[s];
          if (!(line instanceof Line) || !line.marker || line.hidden) continue;
          const v = line.values[i], scale = each.scaleOf(s);
          if (v !== v) continue;
          if (scale.relative) {
            const base = line.first(from);
            if (!base || base !== base) continue;
            scale.setBase(base);
          }
          const y = each.top + scale.y(v);
          if (!(y >= each.top && y <= each.top + each.height)) continue;
          // (A faint line's as faint.)
          g.globalAlpha = line.faint ? FAINT : 1;
          g.fillStyle = theme.background;
          g.beginPath();
          g.arc(x, y, line.marker + 2, 0, 2 * Math.PI);
          g.fill();
          g.fillStyle = line.colorAt(i);
          g.beginPath();
          g.arc(x, y, line.marker, 0, 2 * Math.PI);
          g.fill();
        }
        g.globalAlpha = 1;
        g.restore();
      }
    }
    g.font = theme.font;
    g.textBaseline = "middle";
    // (Its value on its pane's axis, over its tags (none hidden by it: each still read beside it). Written as nothing (a
    // page's writer for a pane showing nothing): no badge.)
    if (pane && this.y >= pane.top && this.y <= pane.top + pane.height) {
      const value = pane.scale.value(this.y - pane.top), words = pane.axisLabel(value, pane.scale, pane.stepAt(value));
      if (words) pill(g, words, theme.badge, inkOn(theme.badge), plot, this.y, "left");
      // (Its value on the left axis too, where the pane has a left scale: against the plot, as the right one is.)
      const leftValue = pane.left && left ? pane.left.value(this.y - pane.top) : NaN, leftWords = pane.left && left ? pane.axisLabel(leftValue, pane.left, pane.stepAt(leftValue, pane.left)) : "";
      if (leftWords) pill(g, leftWords, theme.badge, inkOn(theme.badge), 0, this.y, "right");
    }
    // (Its date on the time axis, its point's.)
    const times = time.times, k = Math.round(i);
    if (k >= 0 && k < times.length && x >= 0 && x <= plot) {
      // (Points less than a day apart (the last one's from the one before it): its time too.)
      const fine = times.length > 1 && (k < times.length - 1 ? times[k + 1] - times[k] : times[k] - times[k - 1]) < DAY;
      const words = (fine ? this.fine : this.day)(times[k]), half = g.measureText(words).width / 2 + 6;
      // (The time axis' labels it touches covered with the ground: none shows cut beside it, as none shows under a tag.)
      g.fillStyle = theme.background;
      for (let n = 0; n < axis.drawnCount; n++) {
        const at = axis.drawnX[n], reach = g.measureText(axis.drawnWords[n]).width / 2 + 2;
        if (Math.abs(at - x) < half + reach) g.fillRect(at - reach, room, 2 * reach, axis.height);
      }
      pill(g, words, theme.badge, inkOn(theme.badge), x, room + 13, "center");
    }
  }
}

/** A crosshair's dash (its line across), and none (made once). */
const DASH = [6, 6], NONE = /** @type {number[]} */ ([]);

/** A point's date (UTC): its day, and its time with `fine`. @param {number} t @param {boolean} fine */
function dateOf(t, fine) {
  const d = new Date(t * 1000), day = `${d.getUTCDate()} ${MONTHS[d.getUTCMonth()]} ${d.getUTCFullYear()}`;
  return fine ? `${day} ${String(d.getUTCHours()).padStart(2, "0")}:${String(d.getUTCMinutes()).padStart(2, "0")}` : day;
}
