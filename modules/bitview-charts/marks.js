// @ts-check
// A series' marks (a signal's turns), their dots drawn over its pane's series added before them: each at a point (`at`,
// its index), above it (a candle's high), below it (its low; a line's close where candles are drawn as one) or on it: a
// dot in its `color` (radius 3.5, times its `size`; 0: none), ringed in the chart's ground, resting on the point from
// its side (on it: centered), several at one point stacked away from it. Its words (`text`) just past its dot (on it:
// under it; at the timeline's ends, moved along it to stay a little within them), in the chart's one pill (a tag's):
// its color (or `textColor`) as it is, its words on it as a tag's (see tint); placed past its dot, over its series,
// outlined: on the ground, edged and worded in that color (see outlined). Where words crowd at the view's zoom, in
// the order of their `priority` (a move's size, say: the biggest first; at one, the latest first; those without, an
// event's name say, first, each counted only in another without), one whose words would run into those of one shown
// before it, in a row theirs can meet at some pan (see #runs), is counted in the nearest of those: "2 more" (room kept
// for it on its left where words later in that order would run into its own), quieter than they are: in the chart's
// text color, kept legible, on a quiet tag of its separator color (zoomed in, they part; a pan regroups none: the order
// is the data's). Its words drawn over everything, each frame (on its pane's layer over it), whole within the plot and
// clear of its other dots and of the words placed before them (in that order, the counts alone last): where its edge
// would cut them or they'd cover a dot, moved to the nearest spot that's clear, along their row, out from the series or
// past their dot (on its other side), where they're still nearer their own dot than any other (none: a pill tried again
// without its count, else left out), a count on the side of its pill away from its dot (right of it where the pill
// moves right or the count has no room on its left); one whose pill's dot is past an edge of the plot, while a mark it
// holds is in view, shown alone where the words of the one nearest that edge would be.
// Added to its pane on its series' side, its marks in their points' order. The room they take kept at its scale's ends
// while its series has values in view (`room` false: none, the page's range alone). Its series hidden, none drawn.
import { legible, outlined, over, tint } from "./color.js";
import { pill, PILL } from "./tags.js";
import { alike } from "./timeline.js";

/** A dot's radius (times its mark's `size`), the ring of the ground about it, its words' gap from it, and the room
 * words keep from another mark's dot: a pixel more, so words just clear of it still sit nearer their own (see ours). */
const DOT = 3.5, RING = 1.5, GAP = 2, APART = GAP + 1;
/** A pill's width past its words', a count's gap from the pill holding it, and the least room between two marks' (side by
 * side, two pills; at one point, one's words and the next one's dot). */
const PAD = 12, SEP = 2, CLEAR = 6;
/** The numbers a frame's words keep in BOXES (see box). */
const BOX = 8;
/** The counts' words, each made once ("n more" at n). @type {string[]} */
const MORE = [];
/** A series' range over the view, read for whether it has values there (made once). */
const SPAN = new Float64Array(2);

/**
 * @typedef {{ at: number, position: "above" | "below" | "on", color: string, text?: string, textColor?: string, size?: number, priority?: number }} Mark
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
    /** Which shown mark holds each mark's words (a shown one: itself; none: -1). */
    this.holder = new Int32Array(0);
    /** Its marks with words in the order they keep them where they crowd (see set), and each one's place in it. */
    this.order = new Int32Array(0);
    this.rank = new Int32Array(0);
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
    /** Each mark's words on their pill, outlined (past its dot, over its series: see drawOver), and each count's ("n
     * more"'s, at n), drawn once as a sprite, for the theme and pixel ratio below (made again for another).
     * @type {(HTMLCanvasElement | undefined)[]} */
    this.sprites = [];
    /** @type {(HTMLCanvasElement | undefined)[]} */
    this.outlinedSprites = [];
    /** @type {(HTMLCanvasElement | undefined)[]} */
    this.moreSprites = [];
    /** @type {object | null} */
    this.spritesTheme = null;
    this.spritesDpr = 0;
    this.roomAbove = 0;
    this.roomBelow = 0;
    /** Each mark's words' middle from its point (CSS pixels, y down), nearest and furthest, by twos (see set). */
    this.rows = new Float64Array(0);
    /** The scale the grouping was found for: its mode, how it's fitted (see Scale's reach) and its fewest pixels. */
    this.saidMode = "";
    /** @type {Function | null} */
    this.saidReach = null;
    this.saidReached = NaN;
    this.saidPixels = NaN;
    this.set(marks);
  }

  /** Its marks (in their points' order). @param {Mark[]} marks */
  set(marks) {
    this.version++;
    const was = this.marks, wasLooks = this.looks, wasSprites = this.sprites, wasOutlined = this.outlinedSprites;
    this.marks = marks;
    this.saidAt = NaN;
    this.measuredFont = "";
    // (Where words crowd, the order they keep theirs in: those without a priority first (an event's name, say: placed
    // before the rest, as neither is counted in the other: see #runs), then the highest, at one the latest (two without:
    // Infinity less Infinity, NaN, falls through to it).)
    const order = [];
    for (let k = 0; k < marks.length; k++) if (marks[k].text) order.push(k);
    order.sort((a, b) => (marks[b].priority ?? Infinity) - (marks[a].priority ?? Infinity) || b - a);
    (this.order = Int32Array.from(order)), (this.rank = new Int32Array(marks.length));
    for (let r = 0; r < order.length; r++) this.rank[order[r]] = r;
    // (Its words' looks: a tag's in its color, as it is (see tint).)
    this.looks = marks.map((mark) => (mark.text ? tint(mark.textColor ?? mark.color) : null));
    // (Each one's sprites kept where its words and look stay: a chart app sets its marks again at every rebuild.)
    const kept = marks.map((mark, k) => mark.text === was[k]?.text && this.looks[k] === wasLooks[k]);
    (this.sprites = kept.map((keep, k) => (keep ? wasSprites[k] : undefined))), (this.outlinedSprites = kept.map((keep, k) => (keep ? wasOutlined[k] : undefined)));
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
    // (Each one's words' middle from its point, y down, nearest and furthest: past the dots stacked before it at its
    // point on its side, and the words of those shown (see #at).)
    if (this.rows.length < 2 * marks.length) this.rows = new Float64Array(4 * marks.length);
    for (let k = 0; k < marks.length; k++) {
      const mark = marks[k], dot = dotOf(mark), up = mark.position === "above" ? -1 : 1;
      let least = (mark.position === "on" ? dot / 2 : dot) + GAP + PILL / 2, most = least;
      for (let j = k - 1; mark.position !== "on" && j >= 0 && marks[j].at === mark.at; j--)
        if (marks[j].position === mark.position) (least += dotOf(marks[j]) + CLEAR), (most += dotOf(marks[j]) + CLEAR + (marks[j].text ? GAP + PILL : 0));
      (this.rows[2 * k] = up < 0 ? -most : least), (this.rows[2 * k + 1] = up < 0 ? -least : most);
    }
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
   * Which marks show their words at `spacing`, and how many others' each holds: in their order (see set), each clear
   * of those shown before it (see #runs), else counted in the nearest of those it runs into (room kept for its count
   * just left of its pill where words later in the order would run into its own; drawn where free puts it); marks at
   * one point stack, each its own. Each one's words kept CLEAR within the timeline's ends as the chart frames them
   * whole (its first point at the plot's left, the right offset past its last), moved along their dot. Read from its
   * points' values, the zoom and how its scale can be fitted at it (see Scale's reach), not where they're drawn: a pan
   * or a scale's fit changes none.
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {Float64Array} highs @param {Float64Array} lows @param {Float64Array} values
   */
  #say(g, time, scale, highs, lows, values) {
    const spacing = time.spacing, end = (time.length - 1) * spacing + time.rightOffset, version = this.series.version, mode = scale.mode, reach = scale.reach, reached = scale.reached;
    // (The fewest pixels its scale's range takes: its height but the shares kept at its ends and the most its marks'
    // room can take there (see Pane's fit; another's marks keeping more there, a little less).)
    const share = scale.height * (1 - scale.top - scale.bottom), pixels = Math.max(0, share - Math.min(share / 2, this.roomAbove + this.roomBelow));
    if (alike(spacing, this.saidAt) && alike(end, this.saidEnd) && g.font === this.saidFont && version === this.saidVersion && mode === this.saidMode && reach === this.saidReach && reached === this.saidReached && pixels === this.saidPixels) return;
    (this.saidAt = spacing), (this.saidEnd = end), (this.saidFont = g.font), (this.saidVersion = version), (this.saidMode = mode), (this.saidReach = reach), (this.saidReached = reached), (this.saidPixels = pixels);
    const marks = this.marks, n = marks.length, order = this.order;
    if (this.said.length < n) {
      (this.said = new Uint32Array(n * 2)), (this.offsets = new Float64Array(n * 2)), (this.shown = new Int32Array(n * 2));
      this.holder = new Int32Array(n * 2);
    }
    if (MID.length < n) (MID = new Float64Array(n * 2)), (HEIGHT = new Float64Array(n * 2)), (ROOM = new Float64Array(n * 2)), (VIEWS = new Float64Array(n * 2));
    if (SHOWN[0].length * 32 <= n) SHOWN = [new Int32Array((n >> 4) + 1), new Int32Array((n >> 4) + 1)];
    if (g.font !== this.measuredFont) {
      if (this.widths.length < n) this.widths = new Float64Array(n * 2);
      for (let k = 0; k < n; k++) this.widths[k] = marks[k].text ? g.measureText(/** @type {string} */ (marks[k].text)).width : 0;
      (this.measuredFont = g.font), (this.moreWidths.length = 0);
    }
    const said = this.said, widths = this.widths, holder = this.holder;
    said.fill(0, 0, n), holder.fill(-1, 0, n), SHOWN[0].fill(0), SHOWN[1].fill(0);
    // (Each one's words' middle, CLEAR within the timeline's ends (off the plot's edge at its first, clear of the axis'
    // tags at its last); the height it sits at, as its scale reads it (a log scale, its log); and the widest range its
    // scale takes in a view with it at this zoom, as read too (none known: Infinity).)
    const log = mode === "log", width = time.to - time.from, last = time.length - 1;
    let most = 0;
    for (let r = 0; r < order.length; r++) {
      const k = order[r], i = marks[k].at, half = (widths[k] + PAD) / 2, position = marks[k].position, at = position === "above" ? highs[i] : position === "below" ? lows[i] : values[i];
      MID[k] = Math.min(Math.max(i * spacing, CLEAR + half), end - CLEAR - half);
      HEIGHT[k] = log ? Math.log10(at) : at;
      (SPAN[0] = Infinity), (SPAN[1] = -Infinity), reach(time, Math.max(0, Math.floor(i - width)), Math.min(last, Math.ceil(i + width)), SPAN);
      VIEWS[k] = !(SPAN[0] <= SPAN[1]) ? Infinity : log ? Math.log10(SPAN[1] / SPAN[0]) : SPAN[1] - SPAN[0];
      most = Math.max(most, half);
    }
    PIXELS = pixels;
    // (A count's room: as wide as the widest it could be, every other one held.)
    const room = order.length > 1 ? SEP + this.#moreWidth(g, order.length - 1) + PAD : 0;
    (FAR = 2 * most + 2 * CLEAR), (ROOMS = room);
    for (let r = 0; r < order.length; r++) {
      const k = order[r];
      ROOM[k] = 0;
      let into = this.#into(k);
      if (into < 0 && this.#crowds(k)) (ROOM[k] = room), (into = this.#into(k));
      if (into >= 0) {
        said[into]++, (holder[k] = into);
        continue;
      }
      (said[k] = 1), (holder[k] = k), (this.offsets[k] = MID[k] - marks[k].at * spacing);
      SHOWN[marks[k].position === "above" ? 0 : 1][k >> 5] |= 1 << (k & 31);
    }
    // (Those shown, in their points' order.)
    let from = n;
    for (let k = n - 1; k >= 0; k--) if (said[k]) this.shown[--from] = k;
    this.shownFrom = from;
  }

  /**
   * The nearest mark shown (see #say) whose words (and their count's room) mark k's (and theirs, ROOM[k]) would run
   * into, on either side (-1: none).
   * @param {number} k
   */
  #into(k) {
    const marks = this.marks, n = marks.length, spacing = this.saidAt, half = (this.widths[k] + PAD) / 2;
    const from = MID[k] - half - ROOM[k], to = MID[k] + half;
    let into = -1, near = Infinity;
    // (Each way from it, on each side, while words there could reach it (see FAR).)
    for (let s = 0; s < 2; s++) {
      for (let j = before(SHOWN[s], k); j >= 0 && marks[j].at * spacing + FAR > from; j = before(SHOWN[s], j))
        if (Math.abs(MID[j] - MID[k]) < near && this.#runs(k, from, to, j, ROOM[j])) (into = j), (near = Math.abs(MID[j] - MID[k]));
      for (let j = after(SHOWN[s], k, n); j < n && marks[j].at * spacing - FAR - ROOMS < to; j = after(SHOWN[s], j, n))
        if (Math.abs(MID[j] - MID[k]) < near && this.#runs(k, from, to, j, ROOM[j])) (into = j), (near = Math.abs(MID[j] - MID[k]));
    }
    return into;
  }

  /** Whether words later in their order (see set) would run into mark k's alone (see #runs). @param {number} k */
  #crowds(k) {
    const marks = this.marks, n = marks.length, rank = this.rank, spacing = this.saidAt, half = (this.widths[k] + PAD) / 2;
    const from = MID[k] - half, to = MID[k] + half;
    for (let j = k - 1; j >= 0 && marks[j].at * spacing + FAR > from; j--) if (marks[j].text && rank[j] > rank[k] && this.#runs(k, from, to, j, 0)) return true;
    for (let j = k + 1; j < n && marks[j].at * spacing - FAR < to; j++) if (marks[j].text && rank[j] > rank[k] && this.#runs(k, from, to, j, 0)) return true;
    return false;
  }

  /**
   * Whether mark k's words from `from` to `to` run into mark j's (`room` kept left of them): within CLEAR of them side
   * by side, in a row theirs can come within CLEAR of at some pan (see #meet); at one point, never (they stack); nor
   * where only one of them has a priority (an event's name against a move's size: neither ranks against the other, so
   * neither is counted in the other).
   * @param {number} k @param {number} from @param {number} to @param {number} j @param {number} room
   */
  #runs(k, from, to, j, room) {
    const marks = this.marks, half = (this.widths[j] + PAD) / 2;
    if (marks[j].at === marks[k].at || (marks[j].priority === undefined) !== (marks[k].priority === undefined)) return false;
    if (from >= MID[j] + half + CLEAR || to + CLEAR <= MID[j] - half - room) return false;
    return this.#meet(k, j);
  }

  /**
   * Whether marks k's and j's words can come within CLEAR of each other's row (a pill and CLEAR apart, or nearer) at
   * some pan at this zoom. In a view, k's row lies from j's by their offsets from their points (see set) less their
   * heights' difference (d, as their scale reads them) times its pixels a unit (s): s no fewer than its fewest pixels
   * over the widest range a view with both takes (no wider than either one's: see #say), and with no most (a narrow
   * range, a steep stretch). So the gap runs from where it lies at that fewest on, the way d leans: they meet where
   * it's near there, or once it's past. None known (a relative or held scale): s from 0, rows on one side always
   * meeting, on two only where the series crosses between them (a foot under it higher than a top over it).
   * @param {number} k @param {number} j
   */
  #meet(k, j) {
    const d = HEIGHT[k] - HEIGHT[j], by = (PIXELS / Math.min(VIEWS[k], VIEWS[j])) * d || 0, rows = this.rows, near = PILL + CLEAR;
    return (d < 0 || rows[2 * k + 1] - rows[2 * j] - by > -near) && (d > 0 || rows[2 * k] - rows[2 * j + 1] - by < near);
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
    this.#say(g, time, scale, highs, lows, values);
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
   * them, so they can keep to the view): each in the chart's one pill, the marks it holds counted beside it, quieter
   * than the words they're counted with: in the chart's text color (its axis labels'), kept legible, on a quiet tag of
   * its separator color as laid on its ground (it may hold any kind of mark): a line it crosses covered, not cut off.
   * Only those with their dot in the plot, each whole within it and clear of its other dots (another mark's, of another
   * color maybe) and of the words placed before them (in their order: see set; the counts alone last): where its edge
   * would cut them or they'd cover a dot, moved to the nearest spot that's clear, along their row (off their dot only
   * as far as that takes), out from the series or past their dot, where they're still nearer their own dot than any
   * other, so they never read as another's (none: a pill tried again without its count, else left out), a count on the
   * side of its pill away from its dot (see free); one whose pill's dot is past an edge of the plot, while a mark it
   * holds has its dot in the plot, shown alone where the words of the one nearest that edge would be. One placed past
   * its dot, over its series, outlined: words over the data (see outlined). Each pill a sprite drawn once (see
   * sprites), copied each frame onto whole device pixels: drawn anew each frame, a dense chart's words took twice as
   * long or more (0.065 ms a frame against 0.025 in WebKit, 0.17 against 0.09 in Chromium).
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} _x0 @param {number} plot @param {number} dpr @param {{ background: string, font: string, separator: string, text: string }} theme
   * @param {(x: number, y: number, width: number, height: number) => void} drew Told each one's box (count and pill).
   */
  drawOver(g, time, scale, _x0, plot, dpr, theme, drew) {
    const series = this.series, marks = this.marks, n = marks.length;
    if (series.hidden || !n) return;
    const values = /** @type {Float64Array} */ (series.close ?? series.upper ?? series.values), highs = series.tops?.(time) ?? values, lows = series.feet?.(time) ?? values;
    g.font = theme.font;
    this.#say(g, time, scale, highs, lows, values);
    const said = this.said, widths = this.widths, shown = this.shown, from = this.shownFrom;
    if (BOXES.length < (n - from) * BOX) BOXES = new Float64Array((n - from) * BOX * 2);
    // (Each shown one with its dot in the plot (the first found by halves), its count just left of its pill till free
    // places it.)
    let j = from, boxes = 0;
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
      const right = x + this.offsets[k] + (widths[k] + PAD) / 2, words = right - widths[k] - PAD, count = held ? SEP + this.#moreWidth(g, held) + PAD : 0;
      box(boxes++, k, words - count, words, right, right, AT[1], x, AT[0]);
    }
    // (A count whose pill's dot is out of the plot, while a mark it holds has its dot in it: alone, where the words of
    // the one nearest the edge its pill's dot is past would be.)
    const holder = this.holder, lone = boxes;
    let k = 0;
    for (let hi = n; k < hi; ) {
      const m = (k + hi) >> 1;
      if (time.x(marks[m].at) < 0) k = m + 1;
      else hi = m;
    }
    for (; k < n; k++) {
      const x = time.x(marks[k].at), h = holder[k];
      if (x > plot) break;
      if (h < 0 || h === k) continue;
      const past = time.x(marks[h].at);
      if (past >= 0 && past <= plot) continue;
      // (Past the left edge, the first it holds here; past the right, the last.)
      let t = lone;
      while (t < boxes && holder[BOXES[t * BOX]] !== h) t++;
      if (t < boxes && past < 0) continue;
      this.#at(k, scale, highs, lows, values);
      if (AT[1] !== AT[1]) continue;
      const half = (this.#moreWidth(g, said[h] - 1) + PAD) / 2;
      box(t, k, x - half, x - half, x + half, x + half, AT[1], x, AT[0]);
      if (t === boxes) boxes++;
    }
    if (!boxes) return;
    // (Its dots by the plot, as drawn, then each one's words where they're clear of them and of those placed before
    // them: the shown ones in their order (the biggest first: see set), then the counts alone.)
    this.#dots(time, scale, plot, highs, lows, values);
    if (ORDER.length < boxes) ORDER = new Int32Array(boxes * 2);
    const rank = this.rank;
    for (let p = 0, q = 0; p < boxes; p++) {
      for (q = p; p < lone && q > 0 && rank[BOXES[ORDER[q - 1] * BOX]] > rank[BOXES[p * BOX]]; q--) ORDER[q] = ORDER[q - 1];
      ORDER[q] = p;
    }
    for (let p = 0; p < boxes; p++) free(ORDER[p], p, plot, scale.height, marks[BOXES[ORDER[p] * BOX]].position === "above" ? -1 : 1);
    if (theme !== this.spritesTheme || dpr !== this.spritesDpr) (this.spritesTheme = theme), (this.spritesDpr = dpr), (this.sprites = []), (this.outlinedSprites = []), (this.moreSprites = []);
    const countFill = over(theme.separator, theme.background), countInk = legible(theme.text, countFill);
    for (let s = 0; s < boxes; s++) {
      const b = s * BOX, k = BOXES[b], left = BOXES[b + 1], words = BOXES[b + 2], end = BOXES[b + 3], right = BOXES[b + 4], top = BOXES[b + 5] - PILL / 2;
      if (k < 0) continue;
      const look = /** @type {{ fill: string, ink: string }} */ (this.looks[k]), alone = holder[k] !== k, held = said[holder[k]] - 1;
      // (Past its dot (on its series' side of it), over its series: outlined.)
      const past = (BOXES[b + 5] - BOXES[b + 7]) * (marks[k].position === "above" ? 1 : -1) > 0, sprites = past ? this.outlinedSprites : this.sprites;
      // (Its count where free put it: left of its pill, right of it, or alone in a pill's place; NaN: none.)
      const count = alone ? words : left < words ? left : end < right ? end + SEP : NaN;
      if (!alone) place(g, (sprites[k] ??= sprite(/** @type {string} */ (marks[k].text), past ? outlined(look.fill, theme.background) : look, widths[k] + PAD, theme.font, dpr)), words, top, dpr);
      if (count === count) place(g, (this.moreSprites[held] ??= sprite(MORE[held], { fill: countFill, ink: countInk }, this.#moreWidth(g, held) + PAD, theme.font, dpr)), count, top, dpr);
      drew(left, top, right - left, PILL);
    }
  }

  /**
   * Its dots by the plot (within REACH of it), each where it's drawn (its middle, its reach: its ring and APART, and
   * its mark), into DOTS, in their points' order.
   * @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale @param {number} plot
   * @param {Float64Array} highs @param {Float64Array} lows @param {Float64Array} values
   */
  #dots(time, scale, plot, highs, lows, values) {
    const marks = this.marks, n = marks.length;
    if (DOTS.length < n * 4) DOTS = new Float64Array(n * 8);
    REACH = this.dots + APART;
    let k = 0, d = 0;
    for (let hi = n; k < hi; ) {
      const m = (k + hi) >> 1;
      if (time.x(marks[m].at) < -REACH) k = m + 1;
      else hi = m;
    }
    for (; k < n; k++) {
      const mark = marks[k], x = time.x(mark.at), r = DOT * (mark.size ?? 1);
      if (x > plot + REACH) break;
      if (mark.at < 0 || !r) continue;
      this.#at(k, scale, highs, lows, values);
      if (AT[0] === AT[0]) (DOTS[d * 4] = x), (DOTS[d * 4 + 1] = AT[0]), (DOTS[d * 4 + 2] = r + RING + APART), (DOTS[d * 4 + 3] = k), d++;
    }
    DOTN = d;
  }
}

/** `words` on a pill (see tags.js) in `look` (its fill, its words' ink, its edge where it has one), `width` wide (its
 * words' and PAD), drawn once as a sprite at `dpr`.
 * @param {string} words @param {{ fill: string, ink: string, edge?: string }} look @param {number} width @param {string} font @param {number} dpr */
function sprite(words, look, width, font, dpr) {
  const c = document.createElement("canvas");
  (c.width = Math.ceil(width * dpr)), (c.height = Math.ceil(PILL * dpr));
  const g = /** @type {CanvasRenderingContext2D} */ (c.getContext("2d"));
  g.scale(dpr, dpr);
  (g.font = font), (g.textBaseline = "middle");
  pill(g, words, look.fill, look.ink, 0, PILL / 2, "left", PILL, look.edge);
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

/** The `s`-th words a frame draws, BOX numbers in BOXES: whose (-1: left out); their left, their pill's left and
 * right, and their right (its count SEP from the pill, left or right of it: see free; a count alone in a pill's
 * place); their middle's y; and their dot's middle (x, y).
 * @param {number} s @param {number} k @param {number} left @param {number} words @param {number} end @param {number} right
 * @param {number} y @param {number} x @param {number} dot */
function box(s, k, left, words, end, right, y, x, dot) {
  const b = s * BOX;
  (BOXES[b] = k), (BOXES[b + 1] = left), (BOXES[b + 2] = words), (BOXES[b + 3] = end), (BOXES[b + 4] = right), (BOXES[b + 5] = y), (BOXES[b + 6] = x), (BOXES[b + 7] = dot);
}

/**
 * The `s`-th words (see box) moved where they're clear (see hit) and whole within the plot (`plot` wide, `height`
 * tall), their count on the side of their pill away from its dot: where they are (just within its edge, where that cuts
 * them; the count right of the pill where it has no room on its left), else the nearest spot along their row, either
 * way (the count left of the pill as it moves left, right of it as it moves right), out from their series (`out`: up,
 * -1; down, 1), or past their dot on its other side (over their series), from as far from it as their row is on, where
 * they still read as their own dot's (see ours); none, a pill tried again without its count; else left out.
 * @param {number} s @param {number} placed @param {number} plot @param {number} height @param {number} out
 */
function free(s, placed, plot, height, out) {
  const b = s * BOX, was = BOXES[b + 2], pw = BOXES[b + 3] - was, y = BOXES[b + 5];
  for (let cw = was - BOXES[b + 1]; ; cw = 0) {
    const width = cw + pw, most = plot - CLEAR - width;
    // (Their left: from `a` on left, the count left of the pill (on its dot, or moved left within the plot's right
    // edge); from `c` on right, the count right of it (moved right within the left edge). They start at the first
    // that's in the plot (`flip`: the count right of the pill).)
    let a = Math.min(was - cw, most), c = Math.max(was, CLEAR), flip = a < CLEAR, left = flip ? c : a, to = y;
    let least = left <= most ? 0 : Infinity;
    if (!least && hit(s, left, width, y, placed)) {
      // (Just past what they run into, again and again till clear: left, right, and away from the series from where
      // they start; each walk stopped once it's further than the nearest spot found before it: it couldn't be taken.)
      while (a >= CLEAR && hit(s, a, width, y, placed)) a = HIT[0];
      const byA = a >= CLEAR && ours(s, a + cw, y) ? was - cw - a : Infinity;
      while (c <= most && c - was < byA && hit(s, c, width, y, placed)) c = HIT[1];
      const byC = c <= most && c - was < byA && ours(s, c, y) ? c - was : Infinity, best = Math.min(byA, byC);
      const start = flip ? left : left + cw, off = Math.abs(start - was);
      // (Out from the series from their row; then past their dot, from as far on its other side, where that's nearer.)
      let byAway = Infinity;
      for (let w = 0, dir = out, row = y; w < 2; w++, dir = -out, row = 2 * BOXES[b + 7] - y) {
        const limit = Math.min(best, byAway);
        while (row >= PILL / 2 && row <= height - PILL / 2 && off + Math.abs(row - y) <= limit && hit(s, left, width, row, placed)) row = dir < 0 ? HIT[2] : HIT[3];
        if (row >= PILL / 2 && row <= height - PILL / 2 && off + Math.abs(row - y) <= limit && ours(s, start, row)) (byAway = off + Math.abs(row - y)), (to = row);
      }
      least = Math.min(byA, byC, byAway);
      if (least !== byAway) (flip = least !== byA), (left = flip ? c : a), (to = y);
    }
    if (least < Infinity) {
      const words = flip ? left : left + cw;
      return void ((BOXES[b + 1] = left), (BOXES[b + 2] = words), (BOXES[b + 3] = words + pw), (BOXES[b + 4] = left + width), (BOXES[b + 5] = to));
    }
    if (!cw) return void (BOXES[b] = -1);
  }
}

/**
 * Whether the s-th words, `width` wide from `left`, their middle at `y`, would run into others' (the first `placed` in
 * ORDER: CLEAR apart side by side, GAP over and under) or come near a dot (any but their mark's: APART from its ring);
 * if so, the nearest spots clear of what they'd run into, into HIT: their left just before it and just past it, their
 * middle just over it and just under it. Each tested here as found: rounded once more (`left + width`), a step to one
 * could land back on its edge again and again.
 * @param {number} s @param {number} left @param {number} width @param {number} y @param {number} placed
 */
function hit(s, left, width, y, placed) {
  // (Each bound worked out only once the ones before it hold: without a JIT, a pan frame's placing took a sixth longer
  // working them all out first.)
  for (let p = 0; p < placed; p++) {
    const t = ORDER[p] * BOX;
    if (BOXES[t] < 0) continue;
    const lo = BOXES[t + 1] - CLEAR - width;
    if (lo >= left) continue;
    const hi = BOXES[t + 4] + CLEAR;
    if (left >= hi) continue;
    const up = BOXES[t + 5] - PILL - GAP, down = BOXES[t + 5] + PILL + GAP;
    if (up < y && y < down) return (HIT[0] = lo), (HIT[1] = hi), (HIT[2] = up), (HIT[3] = down), true;
  }
  const k = BOXES[s * BOX];
  for (let d = firstDot(left - REACH); d < DOTN && DOTS[d * 4] - REACH < left + width; d++) {
    const dy = DOTS[d * 4 + 1], reach = DOTS[d * 4 + 2], up = dy - reach - PILL / 2;
    if (up >= y) continue;
    const down = dy + reach + PILL / 2;
    if (y >= down || DOTS[d * 4 + 3] === k) continue;
    const x = DOTS[d * 4], lo = x - reach - width, hi = x + reach;
    if (lo < left && left < hi) return (HIT[0] = lo), (HIT[1] = hi), (HIT[2] = up), (HIT[3] = down), true;
  }
  return false;
}

/**
 * Whether the `s`-th words' pill (see box: a count alone, in a pill's place), its left at `left` and its middle at `y`,
 * is nearer its own dot than any other: moved off it, it never reads as another's.
 * @param {number} s @param {number} left @param {number} y
 */
function ours(s, left, y) {
  const b = s * BOX, right = left + BOXES[b + 3] - BOXES[b + 2], near = apart(BOXES[b + 6], BOXES[b + 7], left, right, y), k = BOXES[b];
  for (let d = firstDot(left - near); d < DOTN && DOTS[d * 4] <= right + near; d++)
    if (DOTS[d * 4 + 3] !== k && apart(DOTS[d * 4], DOTS[d * 4 + 1], left, right, y) <= near) return false;
  return true;
}

/** How far a point is from words from `left` to `right`, their middle at `y` (0: on them).
 * @param {number} px @param {number} py @param {number} left @param {number} right @param {number} y */
function apart(px, py, left, right, y) {
  return Math.hypot(Math.max(left - px, 0, px - right), Math.max(Math.abs(py - y) - PILL / 2, 0));
}

/** The first dot (see #dots) at `x` or past it, found by halves. @param {number} x */
function firstDot(x) {
  let d = 0;
  for (let hi = DOTN; d < hi; ) {
    const m = (d + hi) >> 1;
    if (DOTS[m * 4] < x) d = m + 1;
    else hi = m;
  }
  return d;
}

/** The nearest mark before `i` marked in `bits` (a bit a mark; -1: none). @param {Int32Array} bits @param {number} i */
function before(bits, i) {
  if (i <= 0) return -1;
  let w = (i - 1) >> 5, word = bits[w] & (-1 >>> (31 - ((i - 1) & 31)));
  while (!word) {
    if (--w < 0) return -1;
    word = bits[w];
  }
  return (w << 5) + 31 - Math.clz32(word);
}

/** The nearest mark after `i` marked in `bits` (`n`: none).
 * @param {Int32Array} bits @param {number} i @param {number} n */
function after(bits, i, n) {
  let w = (i + 1) >> 5;
  if (w >= bits.length) return n;
  let word = bits[w] & (-1 << ((i + 1) & 31));
  while (!word) {
    if (++w >= bits.length) return n;
    word = bits[w];
  }
  return Math.min(n, (w << 5) + 31 - Math.clz32(word & -word));
}

/** Where a mark sits (see #at), and what words would run into (see hit): made once. */
const AT = new Float64Array(2), HIT = new Float64Array(4);
/** A frame's words (see box), the order it places them in (see drawOver), and its dots: each one's middle's x and y,
 * its reach and its mark, DOTN of them, none reaching further than REACH (see #dots): made once (grown). */
let BOXES = new Float64Array(BOX * 16), ORDER = new Int32Array(16), DOTS = new Float64Array(64), DOTN = 0, REACH = 0;
/** #say's state: each mark's words' middle, the height it sits at and its count's room; the widest range its scale
 * takes in a view with it, and that scale's fewest pixels (see #meet); which marks are shown on each side (above its
 * series, under it); how far from their point words can run into others (the widest's width and CLEAR twice: their
 * middle within half of them of it, or pinned CLEAR within the timeline's start, and CLEAR apart), and a count's room
 * (made once, grown). */
let MID = new Float64Array(64), HEIGHT = new Float64Array(64), ROOM = new Float64Array(64), VIEWS = new Float64Array(64), PIXELS = 0, SHOWN = [new Int32Array(2), new Int32Array(2)], FAR = 0, ROOMS = 0;
