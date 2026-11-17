// @ts-check
// A series' marks (a signal's turns), their dots drawn over its pane's series added before them: each at a point (`at`,
// its index), above it (a candle's high), below it (its low; a line's close where candles are drawn as one) or on it: a
// dot in its `color` (radius 3.5, times its `size`; 0: none), ringed in the chart's ground, resting on the point from
// its side (on it: centered), several at one point stacked away from it. Its words (`text`) just past its dot (on it:
// under it; at the timeline's ends, moved along it to stay a little within them), in the chart's one pill (a tag's):
// its color (or `textColor`), its words dark on it, the color lifted just as far as they need to read (see lightTint),
// its hue kept and as vivid as sRGB allows there (each hue as near itself as its words let it be). None lost where
// words crowd at the view's zoom: from the latest back, one whose words would run into the next ones shown on its side
// of its series (above it; under it), or on the other side where the series crosses between them (a foot under it
// higher than a top over it), is counted in those: "2 more" on their left, quieter than they are: in the chart's text
// color, kept legible, on a quiet tag of its separator color (zoomed in, they part; a pan regroups none). Its words
// drawn over everything, each frame (on its pane's layer over it), whole within the plot: those its edge would cut
// moved just within it while their dot's in view, else (where that runs them into other words) left out; a count that
// doesn't fit between the plot's edge and its pill's dot left out, the pill kept on its dot; one whose pill's dot is
// past the plot's right edge shown alone, just within it, while a mark it holds is in view. Added to its pane on its
// series' side, its marks in their points' order. The room they take kept at its scale's ends while its series has
// values in view (`room` false: none, the page's range alone). Its series hidden, none drawn.

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
    /** Those marks in their points' order: `shown` from `shownFrom` to its marks' count. */
    this.shown = new Int32Array(0);
    this.shownFrom = 0;
    /** Each shown mark's first mark held (the earliest), and the first one shown after it on the other side (-1: none). */
    this.first = new Int32Array(0);
    this.after = new Int32Array(0);
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
    /** Its dots' widest reach past their points (drawn into the strips they reach), and the stacks' room at its scale's ends (see set). */
    this.dots = 0;
    /** Each mark's words on their pill, and each count's ("n more"'s, at n), drawn once as a sprite, for the theme and
     * pixel ratio below (made again for another): see drawOver. @type {(HTMLCanvasElement | undefined)[]} */
    this.sprites = [];
    /** @type {(HTMLCanvasElement | undefined)[]} */
    this.moreSprites = [];
    /** @type {object | null} */
    this.spritesTheme = null;
    this.spritesDpr = 0;
    this.roomAbove = 0;
    this.roomBelow = 0;
    this.set(marks);
  }

  /** Its marks (in their points' order). @param {Mark[]} marks */
  set(marks) {
    this.version++;
    const was = this.marks, wasLooks = this.looks, wasSprites = this.sprites;
    this.marks = marks;
    this.saidAt = NaN;
    this.measuredFont = "";
    // (Its words' looks: a tag's in its color, lifted where dark words wouldn't read on it.)
    this.looks = marks.map((mark) => (mark.text ? lightTint(mark.textColor ?? mark.color) : null));
    // (Each one's sprite kept where its words and look stay: a chart app sets its marks again at every rebuild.)
    this.sprites = marks.map((mark, k) => (mark.text === was[k]?.text && this.looks[k] === wasLooks[k] ? wasSprites[k] : undefined));
    // (The tallest stack at a point, as drawn: each dot with its ring, each one's words with their gap, the next CLEAR
    // past them. On its point: half its dot each way, its words under it.)
    let point = -1, above = 0, below = 0, top = 0, foot = 0, dots = 0;
    for (const mark of marks) {
      const dot = dotOf(mark), words = mark.text ? GAP + PILL : 0;
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
    if (this.said.length < n) {
      (this.said = new Uint32Array(n * 2)), (this.offsets = new Float64Array(n * 2)), (this.shown = new Int32Array(n * 2));
      (this.first = new Int32Array(n * 2)), (this.after = new Int32Array(n * 2));
    }
    if (HEIGHT.length < n) HEIGHT = new Float64Array(n * 2);
    if (g.font !== this.measuredFont) {
      if (this.widths.length < n) this.widths = new Float64Array(n * 2);
      for (let k = 0; k < n; k++) this.widths[k] = marks[k].text ? g.measureText(/** @type {string} */ (marks[k].text)).width : 0;
      (this.measuredFont = g.font), (this.moreWidths.length = 0);
    }
    const said = this.said, widths = this.widths, offsets = this.offsets;
    // (Each side's left edge of the words shown nearest before, and whose they are.)
    (LEFT[0] = LEFT[1] = Infinity), (SHOWN[0] = SHOWN[1] = -1), (this.shownFrom = n);
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
        this.first[shown] = k;
        continue;
      }
      (said[k] = 1), (offsets[k] = mid - at), (this.after[k] = SHOWN[other]), (SHOWN[side] = k), (LEFT[side] = Math.min(LEFT[side], mid - half)), (this.shown[--this.shownFrom] = k);
    }
  }

  /**
   * Where mark `k` sits, into AT: its dot's middle and its words' (y). Past those before it at its point on its side
   * (above and below apart), each one further, CLEAR between one's words and the next one's dot (its words nearer its
   * own); on its point, centered, its words under it.
   * @param {number} k @param {import("./scale.js").Scale} scale @param {Float64Array} highs @param {Float64Array} lows @param {Float64Array} values
   */
  #at(k, scale, highs, lows, values) {
    const marks = this.marks, mark = marks[k], i = mark.at, position = mark.position, dot = dotOf(mark);
    if (position === "on") {
      const y = scale.y(values[i]);
      (AT[0] = y), (AT[1] = y + dot / 2 + GAP + PILL / 2);
      return;
    }
    let from = 0;
    for (let j = k - 1; j >= 0 && marks[j].at === i; j--) if (marks[j].position === position) from += dotOf(marks[j]) + (this.said[j] > 0 ? GAP + PILL : 0) + CLEAR;
    const up = position === "above" ? -1 : 1, at = up < 0 ? scale.y(highs[i]) - from : scale.y(lows[i]) + from;
    (AT[0] = at + (up * dot) / 2), (AT[1] = at + up * (dot + GAP + PILL / 2));
  }

  /**
   * Its dots, in its pane's picture (their words over everything: see drawOver).
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} _dpr @param {{ background: string, font: string }} theme
   */
  draw(g, time, scale, x0, x1, _dpr, theme) {
    const series = this.series, marks = this.marks, n = marks.length;
    if (series.hidden || !n) return;
    // (Its points' heights: a candle's high and low, else its value; an area's, its top.)
    const values = /** @type {Float64Array} */ (series.close ?? series.upper ?? series.values), highs = series.tops?.(time) ?? values, lows = series.feet?.(time) ?? values;
    g.font = theme.font;
    this.#say(g, time, highs, lows, values);
    // (Those reaching into the strip: their dot and ring, and a pixel past for their smoothed edge; the first found by
    // halves.)
    const from = x0 - this.dots - 1, to = x1 + this.dots + 1;
    let k = 0;
    for (let hi = n; k < hi; ) {
      const m = (k + hi) >> 1;
      if (time.x(marks[m].at) < from) k = m + 1;
      else hi = m;
    }
    for (; k < n; k++) {
      const mark = marks[k], x = time.x(mark.at), r = DOT * (mark.size ?? 1);
      if (x > to) break;
      if (mark.at < 0 || !r) continue;
      this.#at(k, scale, highs, lows, values);
      const y = AT[0];
      if (y !== y) continue;
      g.fillStyle = theme.background;
      g.beginPath();
      g.arc(x, y, r + RING, 0, Math.PI * 2);
      g.fill();
      g.fillStyle = mark.color;
      g.beginPath();
      g.arc(x, y, r, 0, Math.PI * 2);
      g.fill();
    }
  }

  /**
   * Its words, over everything (its pane's layer over it, drawn whole each frame: a picture moved by a pan never holds
   * them, so they can keep to the view): each in the chart's one pill, the marks it holds counted just left of it,
   * quieter than the words they're counted with: in the chart's text color (its axis labels'), kept legible, on a quiet
   * tag of its separator color as laid on its ground (it may hold any kind of mark): a line it crosses covered, not cut
   * off. Only those with their dot in the plot, each whole within it: where its edge would cut them, moved just within
   * it, unless that runs them into other words (then left out); a count that doesn't fit between the plot's left edge
   * and its pill's dot left out, so the pill stays on its dot; one whose pill's dot is past the plot's right edge, while
   * a mark it holds has its dot in the plot, shown alone just within the edge. Each pill a sprite drawn once (see
   * sprites), copied each frame onto whole device pixels: drawn anew each frame, a dense chart's words took twice as long
   * or more (0.065 ms a frame against 0.025 in WebKit, 0.17 against 0.09 in Chromium).
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} _x0 @param {number} plot @param {number} dpr @param {{ background: string, font: string, separator: string, text: string }} theme
   * @param {(x: number, y: number, width: number, height: number) => void} drew Told each one's box (count and pill).
   */
  drawOver(g, time, scale, _x0, plot, dpr, theme, drew) {
    const series = this.series, marks = this.marks, n = marks.length;
    if (series.hidden || !n) return;
    const values = /** @type {Float64Array} */ (series.close ?? series.upper ?? series.values), highs = series.tops?.(time) ?? values, lows = series.feet?.(time) ?? values;
    g.font = theme.font;
    this.#say(g, time, highs, lows, values);
    // (Each shown one with its dot in the plot (the first found by halves).)
    const said = this.said, widths = this.widths, shown = this.shown;
    if (GROUPS.length < (n - this.shownFrom + 2) * 5) GROUPS = new Float64Array((n - this.shownFrom + 2) * 5);
    let j = this.shownFrom, groups = 0;
    for (let hi = n; j < hi; ) {
      const m = (j + hi) >> 1;
      if (time.x(marks[shown[m]].at) < 0) j = m + 1;
      else hi = m;
    }
    for (; j < n; j++) {
      const k = shown[j], x = time.x(marks[k].at), held = said[k] - 1;
      if (x > plot) break;
      this.#at(k, scale, highs, lows, values);
      if (AT[1] !== AT[1]) continue;
      const right = x + this.offsets[k] + (widths[k] + PAD) / 2, words = right - widths[k] - PAD;
      group(groups++, k, held ? words - SEP - this.#moreWidth(g, held) - PAD : words, words, right, AT[1]);
    }
    for (let s = 0; s < groups; s++) {
      const k = GROUPS[s * 5], words = GROUPS[s * 5 + 2], right = GROUPS[s * 5 + 3], y = GROUPS[s * 5 + 4];
      // (Its count, on its pill's left, left out where it doesn't fit between the plot's edge and the pill's dot: moved
      // within the plot, it would take the pill off its dot.)
      if (words - GROUPS[s * 5 + 1] > time.x(marks[k].at)) GROUPS[s * 5 + 1] = words;
      const left = GROUPS[s * 5 + 1], by = left < CLEAR ? CLEAR - left : right > plot - CLEAR ? plot - CLEAR - right : 0;
      if (!by) continue;
      // (Moved within the plot only where no other words come within CLEAR of them there.)
      if (clearOf(s, left + by, right + by, y, groups)) group(s, k, left + by, words + by, right + by, y);
      else GROUPS[s * 5] = -1;
    }
    // (A count whose pill's dot is past the plot's right edge, while a mark it holds has its dot in the plot: alone, where
    // it would be or just within the edge, unless other words come within CLEAR of it there. Only the first shown past
    // the edge on each side of the series can hold one in view: one shown later holds none from before that one.)
    for (let t = 0, h = j < n ? shown[j] : -1; t < 2 && h >= 0; t++, h = this.after[h]) {
      if (said[h] < 2 || time.x(marks[this.first[h]].at) > plot) continue;
      this.#at(h, scale, highs, lows, values);
      const right = Math.min(time.x(marks[h].at) + this.offsets[h] - (widths[h] + PAD) / 2 - SEP, plot - CLEAR), left = right - this.#moreWidth(g, said[h] - 1) - PAD;
      if (AT[1] === AT[1] && clearOf(-1, left, right, AT[1], groups)) group(groups++, h, left, right, right, AT[1]);
    }
    if (theme !== this.spritesTheme || dpr !== this.spritesDpr) (this.spritesTheme = theme), (this.spritesDpr = dpr), (this.sprites = []), (this.moreSprites = []);
    const countFill = over(theme.separator, theme.background), countInk = legible(theme.text, countFill);
    for (let s = 0; s < groups; s++) {
      const k = GROUPS[s * 5], left = GROUPS[s * 5 + 1], words = GROUPS[s * 5 + 2], right = GROUPS[s * 5 + 3], top = GROUPS[s * 5 + 4] - PILL / 2;
      if (k < 0) continue;
      const look = /** @type {{ fill: string, ink: string }} */ (this.looks[k]), held = said[k] - 1;
      if (words < right) place(g, (this.sprites[k] ??= sprite(/** @type {string} */ (marks[k].text), look.fill, look.ink, widths[k] + PAD, theme.font, dpr)), words, top, dpr);
      if (left < words) place(g, (this.moreSprites[held] ??= sprite(MORE[held], countFill, countInk, this.#moreWidth(g, held) + PAD, theme.font, dpr)), left, top, dpr);
      drew(left, top, right - left, PILL);
    }
  }
}

/** `words` on a pill (see tags.js), `width` wide (its words' and PAD), drawn once as a sprite at `dpr`.
 * @param {string} words @param {string} fill @param {string} ink @param {number} width @param {string} font @param {number} dpr */
function sprite(words, fill, ink, width, font, dpr) {
  const c = document.createElement("canvas");
  (c.width = Math.ceil(width * dpr)), (c.height = Math.ceil(PILL * dpr));
  const g = /** @type {CanvasRenderingContext2D} */ (c.getContext("2d"));
  g.scale(dpr, dpr);
  (g.font = font), (g.textBaseline = "middle");
  pill(g, words, fill, ink, 0, PILL / 2, "left");
  return c;
}

/** A sprite (`image`) with its top left at x, y (CSS pixels), on whole device pixels: copied as drawn, never resampled.
 * @param {CanvasRenderingContext2D} g @param {HTMLCanvasElement} image @param {number} x @param {number} y @param {number} dpr */
function place(g, image, x, y, dpr) {
  g.drawImage(image, Math.round(x * dpr) / dpr, Math.round(y * dpr) / dpr, image.width / dpr, image.height / dpr);
}

/** A mark's dot with its ring, across (0: none). @param {Mark} mark */
function dotOf(mark) {
  const r = DOT * (mark.size ?? 1);
  return r ? 2 * (r + RING) : 0;
}

/** The `s`-th words a frame draws: whose (-1: left out), its count's left, its pill's left and right (a count alone: its
 * own right, twice), and their middle's y.
 * @param {number} s @param {number} k @param {number} left @param {number} words @param {number} right @param {number} y */
function group(s, k, left, words, right, y) {
  (GROUPS[s * 5] = k), (GROUPS[s * 5 + 1] = left), (GROUPS[s * 5 + 2] = words), (GROUPS[s * 5 + 3] = right), (GROUPS[s * 5 + 4] = y);
}

/** Whether words from `left` to `right`, their middle at `y`, come within CLEAR of none of the first `groups` drawn (but
 * the `s`-th), in a row of their height. @param {number} s @param {number} left @param {number} right @param {number} y @param {number} groups */
function clearOf(s, left, right, y, groups) {
  for (let t = 0; t < groups; t++) if (t !== s && GROUPS[t * 5] >= 0 && Math.abs(GROUPS[t * 5 + 4] - y) < PILL && left < GROUPS[t * 5 + 3] + CLEAR && right + CLEAR > GROUPS[t * 5 + 1]) return false;
  return true;
}

/** Where a mark sits (see #at), and the words a frame draws (see group): made once (grown). */
const AT = new Float64Array(2);
let GROUPS = new Float64Array(80);
/** #say's state on each side of a series (made once). */
const LEFT = new Float64Array(2), SHOWN = new Int32Array(2);
/** The heights #say reads its marks at, each its own (made once, grown). */
let HEIGHT = new Float64Array(64);
