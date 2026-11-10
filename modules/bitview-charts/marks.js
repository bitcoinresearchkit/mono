// @ts-check
// A series' marks (a signal's turns), drawn over its pane's series added before them: each at a point (`at`, its index),
// above it (a candle's high), below it (its low) or on it; an arrow (`up`, `down`), a `circle` or a `square`, as tall as a
// point is wide (12 to 30 pixels; times its `size`, 0: none), several at one point stacked away from it, in its `color`.
// Its words (`text`) past it, in a pill of the chart's ground edged in its color (or `textColor`), only where they have
// room at the view's zoom: from the latest back, one whose words would run into the next one's goes without them (zoomed
// in, they're back; a pan changes nothing). Added to its pane on its series' side, its marks in their points' order. The
// room they take kept at its scale's ends while its series has values in view (`room` false: none, the page's range
// alone). Its series hidden, none drawn.

import { pill } from "./tags.js";

/** A number rounded up, then down to an odd one, or an even one (as Lightweight Charts sized its markers). @param {number} x */
const odd = (x) => ((n) => (n % 2 ? n : n - 1))(Math.ceil(x)), even = (/** @type {number} */ x) => ((n) => (n % 2 ? n - 1 : n))(Math.ceil(x));
/** A mark's shape's height drawn at `size` (a small one no smaller than at 12 pixels). @param {string} shape @param {number} size */
const shapeHeight = (shape, size) => (shape === "circle" ? odd(Math.max(size, 12) * 0.8) - 1 : shape === "square" ? odd(Math.max(size, 12) * 0.7) : odd(Math.max(size, 12)) - 1);
/** Its words' pill: as tall as a tag's, 2 pixels from its shape. */
const TALL = 16, GAP = 2;
/** Words as wide as drawn in a font, each measured once (cleared when large). @type {Map<string, number>} */
const widths = new Map();
/** A series' range over the view, read for whether it has values there (made once). */
const SPAN = new Float64Array(2);

/**
 * @typedef {{ at: number, position: "above" | "below" | "on", shape: "up" | "down" | "circle" | "square", color: string, text?: string, textColor?: string, size?: number }} Mark
 * @typedef {{ values?: Float64Array, upper?: Float64Array, high?: Float64Array, low?: Float64Array, close?: Float64Array, hidden?: boolean, overlay?: import("./scale.js").Scale | null, first(i: number): number, extent(a: number, b: number, positive: boolean, out: Float64Array): void }} Host
 */

export class Marks {
  /** @param {{ series: Host, marks?: Mark[], room?: boolean }} options */
  constructor({ series, marks = [], room = true }) {
    this.series = series;
    /** @type {Mark[]} */
    this.marks = marks;
    this.room = room;
    this.hidden = false;
    /** Not read on its side's scale: it has no range of its own. */
    this.scaled = false;
    this.version = 0;
    /** Which marks show their words (1), at the spacing and font they were found for. */
    this.said = new Uint8Array(0);
    this.saidAt = NaN;
    this.saidFont = "";
    /** How far past its point a mark reaches (its words' half, a shape's): drawn into the strips it reaches. */
    this.reach = 18;
    /** The stacks' room at their clamped mark size; set() clears it. */
    this.roomAt = NaN;
    this.roomAbove = 0;
    this.roomBelow = 0;
    this.set(marks);
  }

  /** Its marks (in their points' order). @param {Mark[]} marks */
  set(marks) {
    this.version++;
    this.marks = marks;
    this.saidAt = this.roomAt = NaN;
  }

  /** Never fitted to: its room is its margins. */
  extent() {}

  /** Read on its host's overlay, where it has one, through the pane's usual scale selection. */
  get overlay() {
    return this.series.overlay ?? null;
  }

  /** Its series' base (a relative scale read as its series is). @param {number} i */
  first(i) {
    return this.series.first(i);
  }

  /** Never hit. */
  distance() {
    return Infinity;
  }

  /**
   * The room its marks keep clear at its scale's ends, by the points' `spacing`: a mark's height and a margin, and its
   * words' pill where it has words, over the range where it has marks above points, under it where below, half where
   * only on them; none while its series is hidden or has no values in view (`positive`: a log scale's, over zero).
   * @param {number} spacing @param {Float64Array} out @param {import("./timeline.js").Timeline} time @param {boolean} positive
   */
  margins(spacing, out, time, positive) {
    if (!this.room || !this.marks.length || this.series.hidden) return;
    const a = Math.ceil(time.from), b = Math.floor(time.to);
    if (a > b) return;
    (SPAN[0] = Infinity), (SPAN[1] = -Infinity);
    // (Only presence matters here: the first visible point usually answers without finding the whole range.)
    this.series.extent(a, a, positive, SPAN);
    if (!(SPAN[0] <= SPAN[1]) && a < b) this.series.extent(a + 1, b, positive, SPAN);
    if (!(SPAN[0] <= SPAN[1])) return;
    const unit = Math.min(Math.max(spacing, 12), 30);
    if (unit !== this.roomAt) {
      // (The tallest stack at a point, as drawn: each mark as tall as its shape is, a margin, its words' pill where it has
      // words.)
      const height = even(odd(unit)), margin = Math.max(odd(unit * 0.1), 3), marks = this.marks;
      let point = -1, above = 0, below = 0, top = 0, foot = 0;
      for (let k = 0; k < marks.length; k++) {
        const mark = marks[k], size = height * (mark.size ?? 1), shape = Math.max(size, size && shapeHeight(mark.shape, size)) + margin, words = mark.text ? TALL + GAP : 0;
        if (mark.at !== point) (point = mark.at), (above = below = 0);
        if (mark.position === "above") top = Math.max(top, (above += shape + words));
        else if (mark.position === "below") foot = Math.max(foot, (below += shape + words));
        // (On its point: half its shape each way, its words under it.)
        else (top = Math.max(top, shape / 2)), (foot = Math.max(foot, shape / 2 + words));
      }
      (this.roomAt = unit), (this.roomAbove = Math.ceil(top)), (this.roomBelow = Math.ceil(foot));
    }
    out[0] = Math.max(out[0], this.roomAbove);
    out[1] = Math.max(out[1], this.roomBelow);
  }

  /** Which marks show their words at `spacing`: from the latest back, each clear of the one after it (3 pixels apart). @param {CanvasRenderingContext2D} g @param {number} spacing */
  #say(g, spacing) {
    if (spacing === this.saidAt && g.font === this.saidFont) return;
    (this.saidAt = spacing), (this.saidFont = g.font);
    const marks = this.marks, n = marks.length;
    if (this.said.length < n) this.said = new Uint8Array(n * 2);
    if (widths.size > 2000) widths.clear();
    let start = Infinity;
    for (let k = n - 1; k >= 0; k--) {
      const mark = marks[k];
      this.said[k] = 0;
      if (!mark.text) continue;
      const key = `${g.font}|${mark.text}`;
      let width = widths.get(key);
      if (width === undefined) widths.set(key, (width = g.measureText(mark.text).width));
      const at = mark.at * spacing, half = width / 2 + 6 + 3;
      if (at + half > start) continue;
      (start = at - half), (this.said[k] = 1);
      this.reach = Math.max(this.reach, 18 + width / 2);
    }
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} _dpr @param {{ background: string, font: string }} theme
   */
  draw(g, time, scale, x0, x1, _dpr, theme) {
    const series = this.series, marks = this.marks, n = marks.length;
    if (series.hidden || !n) return;
    // (Its points' heights: a candle's high and low, else its value; an area's, its top.)
    const values = /** @type {Float64Array} */ (series.close ?? series.upper ?? series.values), highs = series.high ?? values, lows = series.low ?? values;
    const unit = Math.min(Math.max(time.spacing, 12), 30), height = even(odd(unit)), margin = Math.max(odd(unit * 0.1), 3);
    g.font = theme.font;
    g.textBaseline = "middle";
    this.#say(g, time.spacing);
    // (Several at one point: each one further from it, above and below apart. Their words after all their shapes: each
    // readable where marks crowd, a neighbor's arrow under a pill rather than over its words.)
    let point = -1, above = 0, below = 0, words = 0;
    const said = WORDS.length < n * 2 ? (WORDS = new Float64Array(n * 2)) : WORDS;
    for (let k = 0; k < n; k++) {
      const mark = marks[k], i = mark.at;
      if (i < 0) continue;
      if (i !== point) (point = i), (above = margin), (below = margin);
      const size = height * (mark.size ?? 1), half = size / 2, text = this.said[k] === 1;
      let y, textY;
      if (mark.position === "above") {
        y = scale.y(highs[i]) - half - above;
        textY = y - half - GAP - TALL / 2;
        above += size + margin + (text ? TALL + GAP : 0);
      } else if (mark.position === "below") {
        y = scale.y(lows[i]) + half + below;
        textY = y + half + GAP + TALL / 2;
        below += size + margin + (text ? TALL + GAP : 0);
      } else (y = scale.y(values[i])), (textY = y + half + GAP + TALL / 2);
      const x = time.x(i);
      if (y !== y || x < x0 - this.reach || x > x1 + this.reach) continue;
      if (text) (said[words * 2] = k), (said[words * 2 + 1] = textY), words++;
      if (!size) continue;
      g.fillStyle = mark.color;
      g.beginPath();
      if (mark.shape === "circle") g.arc(x, y, shapeHeight("circle", size) / 2, 0, Math.PI * 2);
      else if (mark.shape === "square") ((side) => g.rect(x - side / 2, y - side / 2, side, side))(shapeHeight("square", size));
      else {
        // (An arrow: its head toward its point, its stem half as wide behind it.)
        const up = mark.shape === "up" ? 1 : -1, head = shapeHeight(mark.shape, size) / 2, stem = (odd(size / 2) - 1) / 2;
        g.moveTo(x - head, y);
        g.lineTo(x, y - up * head);
        g.lineTo(x + head, y);
        g.lineTo(x + stem, y);
        g.lineTo(x + stem, y + up * head);
        g.lineTo(x - stem, y + up * head);
        g.lineTo(x - stem, y);
        g.closePath();
      }
      g.fill();
    }
    // (Its words in a pill of the ground, edged in its color: read over any line, the chart's one pill shape.)
    for (let s = 0; s < words; s++) {
      const mark = marks[said[s * 2]], color = mark.textColor ?? mark.color;
      pill(g, /** @type {string} */ (mark.text), theme.background, color, time.x(mark.at), said[s * 2 + 1], "center", TALL, color);
    }
  }
}

/** The marks with words a draw found (each its index and its words' y), drawn after all their shapes (made once, grown). */
let WORDS = new Float64Array(64);
