// @ts-check
// A series' marks (a signal's turns), drawn over its pane's series added before them: each at a point (`at`, its
// index), above it (a candle's high), below it (its low; a line's close where candles are drawn as one) or on it: a dot
// in its `color` (radius 3.5, times its `size`; 0: none), ringed in the chart's ground, resting on the point from its
// side (on it: centered), several at one point stacked away from it. Its words (`text`) just past its dot (on it: under
// it; at the timeline's ends, moved along it to stay a little within them), in the chart's one pill (a tag's): its
// color (or `textColor`), lifted to a light one where it isn't, its hue kept and as vivid as sRGB allows there (a light
// pill on either ground, never a dark, dull one), the words on it dark, as a tag's on a light fill. None lost where
// words crowd at the view's zoom: from the latest back, one whose words would run into the next ones shown on its side
// of its series (above it; under it), or on the other side where the series crosses between them (a foot under it
// higher than a top over it), is counted in those: "2 more" on their left, quieter than they are: in the chart's text
// color, kept legible, on a quiet tag of its separator color (zoomed in, they part; a pan changes nothing). Added to
// its pane on its series' side, its marks in their points' order. The room they take kept at its scale's ends while its
// series has values in view (`room` false: none, the page's range alone). Its series hidden, none drawn.

import { legible, lightTint, over } from "./color.js";
import { pill, PILL } from "./tags.js";

/** A dot's radius (times its mark's `size`), the ring of the ground about it, and its words' gap from it. */
const DOT = 3.5, RING = 1.5, GAP = 2;
/** A pill's width past its words', a count's gap from the pill holding it, and the least room between two marks' (side by
 * side, two pills; at one point, one's words and the next one's dot). */
const PAD = 12, SEP = 2, CLEAR = 6;
/** The counts' words, each made once ("n more" at n). @type {string[]} */
const MORE = [];
/** A series' range over the view, read for whether it has values there (made once). */
const SPAN = new Float64Array(2);

/**
 * @typedef {{ at: number, position: "above" | "below" | "on", color: string, text?: string, textColor?: string, size?: number }} Mark
 * @typedef {{ values?: Float64Array, upper?: Float64Array, close?: Float64Array, tops?(time: import("./timeline.js").Timeline): Float64Array, feet?(time: import("./timeline.js").Timeline): Float64Array, hidden?: boolean, overlay?: import("./scale.js").Scale | null, first(i: number): number, extent(a: number, b: number, positive: boolean, out: Float64Array): void }} Host
 */

export class Marks {
  /** @param {{ series: Host & { version: number }, marks?: Mark[], room?: boolean }} options */
  constructor({ series, marks = [], room = true }) {
    this.series = series;
    /** @type {Mark[]} */
    this.marks = marks;
    this.room = room;
    this.hidden = false;
    /** Not read on its side's scale: it has no range of its own. */
    this.scaled = false;
    this.version = 0;
    /** Which marks show their words, each 1 and how many others' it holds (0: none), at the spacing and font found for. */
    this.said = new Uint32Array(0);
    this.saidAt = NaN;
    this.saidEnd = NaN;
    this.saidFont = "";
    this.saidVersion = NaN;
    /** Each mark's words' width, and each count's ("n more"'s, at n), in the font they were measured in. */
    this.widths = new Float64Array(0);
    /** @type {number[]} */
    this.moreWidths = [];
    this.measuredFont = "";
    /** Each shown mark's words' middle past its dot (CSS pixels): none but at the timeline's ends, kept within them. */
    this.offsets = new Float64Array(0);
    /** Each mark's words' look (their pill's fill and ink). @type {({ fill: string, ink: string } | null)[]} */
    this.looks = [];
    /** How far past its point a mark reaches (its dot, its words' half, its count): drawn into the strips it reaches. */
    this.reach = 0;
    /** Its dots' widest reach, and the stacks' room at its scale's ends (see set). */
    this.dots = 0;
    this.roomAbove = 0;
    this.roomBelow = 0;
    this.set(marks);
  }

  /** Its marks (in their points' order). @param {Mark[]} marks */
  set(marks) {
    this.version++;
    this.marks = marks;
    this.saidAt = NaN;
    this.measuredFont = "";
    // (Its words' looks: a tag's in its color, lifted where dark words wouldn't read on it.)
    this.looks = marks.map((mark) => (mark.text ? lightTint(mark.textColor ?? mark.color) : null));
    // (The tallest stack at a point, as drawn: each dot with its ring, each one's words with their gap, the next CLEAR
    // past them. On its point: half its dot each way, its words under it.)
    let point = -1, above = 0, below = 0, top = 0, foot = 0, dots = 0;
    for (const mark of marks) {
      const r = DOT * (mark.size ?? 1), dot = r ? 2 * (r + RING) : 0, words = mark.text ? GAP + PILL : 0;
      if (mark.at !== point) (point = mark.at), (above = below = 0);
      if (mark.position === "above") top = Math.max(top, (above += (above ? CLEAR : 0) + dot + words));
      else if (mark.position === "below") foot = Math.max(foot, (below += (below ? CLEAR : 0) + dot + words));
      else (top = Math.max(top, dot / 2)), (foot = Math.max(foot, dot / 2 + words));
      dots = Math.max(dots, dot / 2);
    }
    (this.roomAbove = Math.ceil(top)), (this.roomBelow = Math.ceil(foot)), (this.dots = dots);
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
   * The room its marks keep clear at its scale's ends (see set): over the range where it has marks above points, under
   * it where below, half a dot each way where only on them; none while its series is hidden or has no values in view
   * (`positive`: a log scale's, over zero).
   * @param {number} _spacing @param {Float64Array} out @param {import("./timeline.js").Timeline} time @param {boolean} positive
   */
  margins(_spacing, out, time, positive) {
    if (!this.room || !this.marks.length || this.series.hidden) return;
    const a = Math.ceil(time.from), b = Math.floor(time.to);
    if (a > b) return;
    (SPAN[0] = Infinity), (SPAN[1] = -Infinity);
    // (Only presence matters here: the first visible point usually answers without finding the whole range.)
    this.series.extent(a, a, positive, SPAN);
    if (!(SPAN[0] <= SPAN[1]) && a < b) this.series.extent(a + 1, b, positive, SPAN);
    if (!(SPAN[0] <= SPAN[1])) return;
    out[0] = Math.max(out[0], this.roomAbove);
    out[1] = Math.max(out[1], this.roomBelow);
  }

  /** A count's words' width ("n more"'s), measured once a font. @param {CanvasRenderingContext2D} g @param {number} n */
  #moreWidth(g, n) {
    return (this.moreWidths[n] ??= g.measureText((MORE[n] ??= `${n} more`)).width);
  }

  /**
   * Which marks show their words at `spacing`, and how many others' each holds: from the latest back, each clear
   * (`CLEAR` apart) of the one shown after it on its side (above its series; under it), and of the other side's where
   * the series crosses between them (by the heights its marks sit at: their tops over it, their feet under it, else its
   * values; at one point, never), else counted in that one (its count just left of its pill, toward the marks it
   * holds); marks at one point stack, each its own. Each one's words kept CLEAR within the timeline's ends as the chart
   * frames them whole (its first point at the plot's left, the right offset past its last), moved along their dot. Read
   * from its points' values and the zoom, not where they're drawn: a pan or a scale's fit changes none.
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {Float64Array} highs @param {Float64Array} lows @param {Float64Array} values
   */
  #say(g, time, highs, lows, values) {
    const spacing = time.spacing, end = (time.length - 1) * spacing + time.rightOffset, version = this.series.version;
    if (spacing === this.saidAt && end === this.saidEnd && g.font === this.saidFont && version === this.saidVersion) return;
    (this.saidAt = spacing), (this.saidEnd = end), (this.saidFont = g.font), (this.saidVersion = version);
    const marks = this.marks, n = marks.length;
    if (this.said.length < n) (this.said = new Uint32Array(n * 2)), (this.offsets = new Float64Array(n * 2));
    if (HEIGHT.length < n) HEIGHT = new Float64Array(n * 2);
    if (g.font !== this.measuredFont) {
      if (this.widths.length < n) this.widths = new Float64Array(n * 2);
      for (let k = 0; k < n; k++) this.widths[k] = marks[k].text ? g.measureText(/** @type {string} */ (marks[k].text)).width : 0;
      (this.measuredFont = g.font), (this.moreWidths.length = 0);
    }
    const said = this.said, widths = this.widths, offsets = this.offsets;
    this.reach = this.dots;
    // (Each side's left edge of the words shown nearest before, and whose they are.)
    (LEFT[0] = LEFT[1] = Infinity), (SHOWN[0] = SHOWN[1] = -1);
    for (let k = n - 1; k >= 0; k--) {
      const mark = marks[k];
      said[k] = 0;
      if (!mark.text) continue;
      const i = mark.at, side = mark.position === "above" ? 0 : 1, other = 1 - side, at = i * spacing, half = (widths[k] + PAD) / 2;
      // (CLEAR within the timeline's ends: off the plot's edge at its first, clear of the axis' tags at its last.)
      const mid = Math.min(Math.max(at, CLEAR + half), end - CLEAR - half);
      HEIGHT[k] = side ? (mark.position === "below" ? lows[i] : values[i]) : highs[i];
      const near = SHOWN[other], crossed = near >= 0 && mid + half + CLEAR > LEFT[other] && marks[near].at !== i && (side ? HEIGHT[k] > HEIGHT[near] : HEIGHT[k] < HEIGHT[near]);
      const into = mid + half + CLEAR > LEFT[side] && marks[SHOWN[side]].at !== i ? side : crossed ? other : -1;
      if (into >= 0) {
        const shown = SHOWN[into], count = said[shown]++, x = marks[shown].at * spacing + offsets[shown] - (widths[shown] + PAD) / 2;
        LEFT[into] = Math.min(LEFT[into], x - SEP - this.#moreWidth(g, count) - PAD);
        this.reach = Math.max(this.reach, marks[shown].at * spacing - LEFT[into]);
        continue;
      }
      (said[k] = 1), (offsets[k] = mid - at), (SHOWN[side] = k), (LEFT[side] = Math.min(LEFT[side], mid - half));
      this.reach = Math.max(this.reach, half + Math.abs(mid - at));
    }
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} _dpr @param {{ background: string, font: string, separator: string, text: string }} theme
   */
  draw(g, time, scale, x0, x1, _dpr, theme) {
    const series = this.series, marks = this.marks, n = marks.length;
    if (series.hidden || !n) return;
    // (Its points' heights: a candle's high and low, else its value; an area's, its top.)
    const values = /** @type {Float64Array} */ (series.close ?? series.upper ?? series.values), highs = series.tops?.(time) ?? values, lows = series.feet?.(time) ?? values;
    g.font = theme.font;
    g.textBaseline = "middle";
    this.#say(g, time, highs, lows, values);
    // (Several at one point: each one further from it, above and below apart, CLEAR between one's words and the next
    // one's dot (its words nearer its own). Their words after all their dots: each readable where marks crowd, a
    // neighbor's dot under a pill rather than over its words.)
    let point = -1, above = 0, below = 0, words = 0;
    const said = WORDS.length < n * 2 ? (WORDS = new Float64Array(n * 2)) : WORDS;
    for (let k = 0; k < n; k++) {
      const mark = marks[k], i = mark.at;
      if (i < 0) continue;
      if (i !== point) (point = i), (above = below = 0);
      const r = DOT * (mark.size ?? 1), dot = r ? 2 * (r + RING) : 0, text = this.said[k] > 0;
      let y, textY;
      if (mark.position === "above") {
        const at = scale.y(highs[i]) - above;
        (y = at - dot / 2), (textY = at - dot - GAP - PILL / 2), (above += dot + (text ? GAP + PILL : 0) + CLEAR);
      } else if (mark.position === "below") {
        const at = scale.y(lows[i]) + below;
        (y = at + dot / 2), (textY = at + dot + GAP + PILL / 2), (below += dot + (text ? GAP + PILL : 0) + CLEAR);
      } else (y = scale.y(values[i])), (textY = y + dot / 2 + GAP + PILL / 2);
      const x = time.x(i);
      if (y !== y || x < x0 - this.reach || x > x1 + this.reach) continue;
      if (text) (said[words * 2] = k), (said[words * 2 + 1] = textY), words++;
      if (!r) continue;
      g.fillStyle = theme.background;
      g.beginPath();
      g.arc(x, y, r + RING, 0, Math.PI * 2);
      g.fill();
      g.fillStyle = mark.color;
      g.beginPath();
      g.arc(x, y, r, 0, Math.PI * 2);
      g.fill();
    }
    // (Its words in the chart's one pill; the marks it holds counted just left of it, quieter than the words they're
    // counted with: in the chart's text color (its axis labels'), kept legible, on a quiet tag of its separator color as
    // laid on its ground (it may hold any kind of mark): a line it crosses covered, not cut off.)
    const countFill = over(theme.separator, theme.background), countInk = legible(theme.text, countFill);
    for (let s = 0; s < words; s++) {
      const k = said[s * 2], mark = marks[k], look = /** @type {{ fill: string, ink: string }} */ (this.looks[k]), x = time.x(mark.at) + this.offsets[k], y = said[s * 2 + 1], held = this.said[k] - 1;
      pill(g, /** @type {string} */ (mark.text), look.fill, look.ink, x, y, "center");
      if (held) pill(g, MORE[held], countFill, countInk, x - (this.widths[k] + PAD) / 2 - SEP, y, "right");
    }
  }
}

/** The marks with words a draw found (each its index and its words' y), drawn after all their dots (made once, grown). */
let WORDS = new Float64Array(64);
/** #say's state on each side of a series (made once). */
const LEFT = new Float64Array(2), SHOWN = new Int32Array(2);
/** The heights #say reads its marks at, each its own (made once, grown). */
let HEIGHT = new Float64Array(64);
