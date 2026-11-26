// @ts-check
// The crosshair: over the whole chart, its own canvas (the panes' pictures never drawn again for it): a line down every
// pane at a point (the nearest to where it's pointed), a line across the pane pointed at, at the pointer's height; the
// point's date on the time axis and the height's value on that pane's axis, each in a badge. And a separator between
// panes pointed at or dragged, along it in the theme's active color. Lines first, dots over them: its lines, then a
// page's own that go with it (a Custom's `crosshair`: a line down a day), then each series' own dots its line down
// crosses, drawn again, and its dots on the lines; a page's layers over the panes (words, a dot of its own) over it all.

import { kept } from "./format.js";
import { FAINT } from "./line.js";
import { lineOf } from "./candles.js";
import { inkOn, painted } from "./color.js";
import { pill, PILL } from "./tags.js";
import { layer } from "./layer.js";

const DAY = 86400, MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

export class Crosshair {
  /** Its canvas' ends, from its plot's left (CSS pixels): where its badges are kept within (see draw), set each draw. */
  #within = /** @type {[number, number]} */ ([0, 0]);

  constructor() {
    this.canvas = layer("position: absolute; left: 0; top: 0; z-index: 1; display: block; pointer-events: none");
    const g = this.canvas.getContext("2d");
    if (!g) throw new Error("bitview-charts: no 2D canvas");
    this.g = g;
    /** The point it's on (a logical index), or NaN: none shown. */
    this.index = NaN;
    /** Its height in the chart (CSS pixels), and the pane it's in (-1: none, on the time axis). */
    this.y = NaN;
    this.pane = -1;
    /**
     * The series pointed at, its dot drawn larger and ringed (over the others): a page's (null: none), else (undefined)
     * the one a pointer's on (`hit`).
     * @type {any}
     */
    this.pointed = undefined;
    /** The series a pointer's on at it (found once by the chart as it moves: see setCrosshair), or null. @type {any} */
    this.hit = null;
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
    // A supplied formatter may depend on state beyond the time (such as the interval shown).
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
    const from = Math.max(0, time.firstVisible), on = x >= 0 && x <= plot;
    // (Each pane's own lines with it (a page's), then the dots its line down crosses: each series' own (a mark's, a
    // line's), in the picture under it, drawn again in a strip about it as wide as it is, with its smoothed edges. Not
    // along its line across: dashed, faint and moving with the pointer, where a line's dots would be drawn again the
    // plot's width each frame.)
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
    // (Its dot on each line at its point (a line's `marker` its radius), ringed in the ground, within its pane's plot; the
    // line pointed at's larger, in a ring of its color, over the others: which one the crosshair reads.)
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
          // (None where its color there paints nothing: an unseen line's ring would cover what's under it.)
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
          // (A faint line's as faint.)
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
    // (Its badges kept on its canvas (drawn from its plot's left): a long one slid in, not cut at an edge.)
    const within = this.#within;
    (within[0] = -left), (within[1] = this.width - left);
    // (Its value on its pane's axis, over its tags (none hidden by it: each still read beside it), the labels it covers
    // gone as under a line's tag (see Pane's cover). Written as nothing (a page's writer for a pane showing nothing): no
    // badge.)
    if (pane && this.y >= pane.top && this.y <= pane.top + pane.height) {
      const value = pane.scale.value(this.y - pane.top), words = pane.axisLabel(value, pane.scale, pane.stepAt(value));
      if (words) pane.cover(g, 0, this.y - pane.top, pane.top, plot, pane.width - pane.plot, theme), pill(g, words, theme.badge, inkOn(theme.badge), plot, this.y, "left", PILL, "", within);
      // (Its value on the left axis too, where the pane has a left scale: against the plot, as the right one is.)
      const leftValue = pane.left && left ? pane.left.value(this.y - pane.top) : NaN, leftWords = pane.left && left ? pane.axisLabel(leftValue, pane.left, pane.stepAt(leftValue, pane.left)) : "";
      if (leftWords) pane.cover(g, 1, this.y - pane.top, pane.top, -left, left, theme), pill(g, leftWords, theme.badge, inkOn(theme.badge), 0, this.y, "right", PILL, "", within);
    }
    // (Its date on the time axis, its point's; past the last, as the points' step goes on. Points less than a day apart
    // (the last one's from the one before it): its time too. Written as nothing (a page saying the date elsewhere): no
    // badge, as a value's.)
    const times = time.times, k = Math.round(i), at = time.timeAt(k), m = times.length;
    const words = at === at && x >= 0 && x <= plot ? (m > 1 && (k < m - 1 ? times[k + 1] - times[k] : times[m - 1] - times[m - 2]) < DAY ? this.fine : this.day)(at) : "";
    if (words) {
      // (Its middle where it's drawn: at the point, slid in from an edge.)
      const half = g.measureText(words).width / 2 + 6, middle = Math.max(within[0] + half, Math.min(x, within[1] - half));
      // (The time axis' labels it touches covered with the ground: none shows cut beside it, as none shows under a tag.)
      g.fillStyle = theme.background;
      for (let n = 0; n < axis.drawnCount; n++) {
        const at = axis.drawnX[n], reach = axis.drawnHalf[n] + 2;
        if (Math.abs(at - middle) < half + reach) g.fillRect(at - reach, room, 2 * reach, axis.height);
      }
      pill(g, words, theme.badge, inkOn(theme.badge), middle, room + 13, "center");
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
