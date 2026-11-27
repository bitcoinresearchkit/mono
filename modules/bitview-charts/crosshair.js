// @ts-check

import { kept } from "./format.js";
import { FAINT } from "./line.js";
import { lineOf } from "./candles.js";
import { inkOn, painted } from "./color.js";
import { pill, PILL } from "./tags.js";
import { layer } from "./layer.js";

const DAY = 86400, MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

export class Crosshair {
  #within = /** @type {[number, number]} */ ([0, 0]);

  constructor() {
    this.canvas = layer("position: absolute; left: 0; top: 0; z-index: 1; display: block; pointer-events: none");
    const g = this.canvas.getContext("2d");
    if (!g) throw new Error("bitview-charts: no 2D canvas");
    this.g = g;
    this.index = NaN;
    this.y = NaN;
    this.pane = -1;
    /** null: none; undefined: the one a pointer's on. @type {any} */
    this.pointed = undefined;
    /** @type {any} */
    this.hit = null;
    this.width = 0;
    this.height = 0;
    this.shown = false;
    this.day = kept((t) => dateOf(t, false));
    this.fine = kept((t) => dateOf(t, true));
  }

  /** @param {(t: number) => string} format */
  setFormat(format) {
    // A supplied formatter may depend on more than the time (such as the interval shown).
    this.day = this.fine = format;
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
   * @param {import("./timeline.js").Timeline} time @param {import("./pane.js").Pane[]} panes @param {number} room
   * @param {number} left @param {import("./pane.js").Theme} theme @param {number} dpr
   * @param {import("./axis.js").TimeAxis} axis
   * @param {import("./pane.js").Pane | null} [lit] the separator pointed at or dragged (the pane over it)
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
    if (over) {
      const y = Math.round((over.top + over.height) * dpr) / dpr - 0.5;
      (g.strokeStyle = theme.active), (g.lineWidth = 1), g.beginPath(), g.moveTo(0, y), g.lineTo(this.width, y), g.stroke();
    }
    if (i !== i) return;
    g.translate(left, 0);
    const plot = time.width, x = Math.round(time.x(i) * dpr) / dpr + 0.5 / dpr;
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
    const from = Math.max(0, time.firstVisible), on = x >= 0 && x <= plot;
    // Dots redraw in a strip about the vertical line only; the horizontal one would redraw every dot each frame.
    for (const each of panes) {
      g.save();
      g.beginPath();
      g.rect(0, each.top, plot, each.height);
      g.clip();
      g.translate(0, each.top);
      for (let s = 0; s < each.series.length; s++) {
        const series = each.series[s];
        if (series.crosshair && !series.hidden) series.draw(g, time, each.scaleOf(s), 0, plot, dpr, theme);
      }
      if (on && painted(theme.crosshair)) {
        g.beginPath();
        g.rect(x - 1, 0, 2, each.height);
        g.clip();
        for (let s = 0; s < each.series.length; s++) {
          const series = each.series[s], scale = each.scaleOf(s);
          if (!series.drawDots || series.hidden) continue;
          if (scale.relative) {
            const base = series.first(from);
            if (!base || base !== base) continue;
            scale.setBase(base);
          }
          g.globalAlpha = series.faint ? FAINT : 1;
          series.drawDots(g, time, scale, x - 1, x + 1, dpr, theme);
        }
      }
      g.restore();
    }
    if (on) {
      const pointed = this.pointed !== undefined ? this.pointed : this.hit;
      let focus = NaN, focusY = NaN, focusColor = "", focusMarker = 0;
      for (let p = 0; p < panes.length; p++) {
        const each = panes[p];
        g.save();
        g.beginPath();
        g.rect(0, each.top, plot, each.height);
        g.clip();
        for (let s = 0; s < each.series.length; s++) {
          const series = each.series[s], line = lineOf(series);
          if (!line || !line.marker || series.hidden) continue;
          const v = line.values[i], scale = each.scaleOf(s);
          // Nothing where its color paints nothing: an unseen line's ring would cover what's under it.
          if (v !== v || !painted(line.colorAt(i))) continue;
          if (scale.relative) {
            const base = line.first(from);
            if (!base || base !== base) continue;
            scale.setBase(base);
          }
          const y = each.top + scale.y(v);
          if (!(y >= each.top && y <= each.top + each.height)) continue;
          if (series === pointed && !series.faint) {
            (focus = p), (focusY = y), (focusColor = line.colorAt(i)), (focusMarker = line.marker);
            continue;
          }
          g.globalAlpha = series.faint ? FAINT : 1;
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
        if (focus === p) {
          g.fillStyle = focusColor;
          g.globalAlpha = 0.3;
          g.beginPath();
          g.arc(x, focusY, focusMarker + 7, 0, 2 * Math.PI);
          g.fill();
          g.globalAlpha = 1;
          g.fillStyle = theme.background;
          g.beginPath();
          g.arc(x, focusY, focusMarker + 3.5, 0, 2 * Math.PI);
          g.fill();
          g.fillStyle = focusColor;
          g.beginPath();
          g.arc(x, focusY, focusMarker + 1.5, 0, 2 * Math.PI);
          g.fill();
        }
        g.restore();
      }
    }
    g.font = theme.font;
    g.textBaseline = "middle";
    const within = this.#within;
    (within[0] = -left), (within[1] = this.width - left);
    if (pane && this.y >= pane.top && this.y <= pane.top + pane.height) {
      const value = pane.scale.value(this.y - pane.top), words = pane.axisLabel(value, pane.scale, pane.stepAt(value));
      if (words) pane.cover(g, 0, this.y - pane.top, pane.top, plot, pane.width - pane.plot, theme), pill(g, words, theme.badge, inkOn(theme.badge), plot, this.y, "left", PILL, "", within);
      const leftValue = pane.left && left ? pane.left.value(this.y - pane.top) : NaN, leftWords = pane.left && left ? pane.axisLabel(leftValue, pane.left, pane.stepAt(leftValue, pane.left)) : "";
      if (leftWords) pane.cover(g, 1, this.y - pane.top, pane.top, -left, left, theme), pill(g, leftWords, theme.badge, inkOn(theme.badge), 0, this.y, "right", PILL, "", within);
    }
    const times = time.times, k = Math.round(i), at = time.timeAt(k), m = times.length;
    const words = at === at && x >= 0 && x <= plot ? (m > 1 && (k < m - 1 ? times[k + 1] - times[k] : times[m - 1] - times[m - 2]) < DAY ? this.fine : this.day)(at) : "";
    if (words) {
      const half = g.measureText(words).width / 2 + 6, middle = Math.max(within[0] + half, Math.min(x, within[1] - half));
      g.fillStyle = theme.background;
      for (let n = 0; n < axis.drawnCount; n++) {
        const at = axis.drawnX[n], reach = axis.drawnHalf[n] + 2;
        if (Math.abs(at - middle) < half + reach) g.fillRect(at - reach, room, 2 * reach, axis.height);
      }
      pill(g, words, theme.badge, inkOn(theme.badge), middle, room + 13, "center");
    }
  }
}

const DASH = [6, 6], NONE = /** @type {number[]} */ ([]);

/** @param {number} t @param {boolean} fine */
function dateOf(t, fine) {
  const d = new Date(t * 1000), day = `${d.getUTCDate()} ${MONTHS[d.getUTCMonth()]} ${d.getUTCFullYear()}`;
  return fine ? `${day} ${String(d.getUTCHours()).padStart(2, "0")}:${String(d.getUTCMinutes()).padStart(2, "0")}` : day;
}
