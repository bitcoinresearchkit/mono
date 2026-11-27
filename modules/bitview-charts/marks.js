// @ts-check
import { legible, outlined, over, tint } from "./color.js";
import { pill, PILL } from "./tags.js";
import { alike } from "./timeline.js";

/** A pixel past GAP: words just clear of another dot still sit nearer their own. */
const DOT = 3.5, RING = 1.5, GAP = 2, APART = GAP + 1;
const PAD = 12, SEP = 2, CLEAR = 6;
const BOX = 8;
/** @type {string[]} */
const MORE = [];
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
    /** Per mark: 0 not shown, else 1 plus how many others' words it holds. */
    this.said = new Uint32Array(0);
    this.shown = new Int32Array(0);
    this.shownFrom = 0;
    /** Which shown mark holds each mark's words (itself when shown; -1: none). */
    this.holder = new Int32Array(0);
    this.order = new Int32Array(0);
    this.rank = new Int32Array(0);
    this.saidAt = NaN;
    this.saidEnd = NaN;
    this.saidFont = "";
    this.saidVersion = NaN;
    this.widths = new Float64Array(0);
    /** @type {number[]} */
    this.moreWidths = [];
    this.measuredFont = "";
    this.offsets = new Float64Array(0);
    /** @type {({ fill: string, ink: string } | null)[]} */
    this.looks = [];
    this.dots = 0;
    /** @type {(HTMLCanvasElement | undefined)[]} */
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
    /** Per mark, its words' middle from its point, nearest then furthest (y down). */
    this.rows = new Float64Array(0);
    this.saidMode = "";
    /** @type {Function | null} */
    this.saidReach = null;
    this.saidReached = NaN;
    this.saidPixels = NaN;
    this.set(marks);
  }

  /** Marks in their points' order. @param {Mark[]} marks */
  set(marks) {
    this.version++;
    const was = this.marks, wasLooks = this.looks, wasSprites = this.sprites, wasOutlined = this.outlinedSprites;
    this.marks = marks;
    this.saidAt = NaN;
    this.measuredFont = "";
    // (No priority sorts first (an event's name); two without: Infinity - Infinity is NaN, so the latest first.)
    const order = [];
    for (let k = 0; k < marks.length; k++) if (marks[k].text) order.push(k);
    order.sort((a, b) => (marks[b].priority ?? Infinity) - (marks[a].priority ?? Infinity) || b - a);
    (this.order = Int32Array.from(order)), (this.rank = new Int32Array(marks.length));
    for (let r = 0; r < order.length; r++) this.rank[order[r]] = r;
    this.looks = marks.map((mark) => (mark.text ? tint(mark.textColor ?? mark.color) : null));
    // (Sprites kept where words and look stay: apps set their marks again at every rebuild.)
    const kept = marks.map((mark, k) => mark.text === was[k]?.text && this.looks[k] === wasLooks[k]);
    (this.sprites = kept.map((keep, k) => (keep ? wasSprites[k] : undefined))), (this.outlinedSprites = kept.map((keep, k) => (keep ? wasOutlined[k] : undefined)));
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

  get overlay() {
    return this.series.overlay ?? null;
  }

  /** Read as its series is on a relative scale. @param {number} i */
  first(i) {
    return this.series.first(i);
  }

  distance() {
    return Infinity;
  }

  /** @param {number} _spacing @param {Float64Array} out @param {import("./timeline.js").Timeline} time @param {boolean} positive */
  margins(_spacing, out, time, positive) {
    if (!this.room || !this.marks.length || this.series.hidden) return;
    const a = Math.ceil(time.from), b = Math.floor(time.to);
    if (a > b) return;
    (SPAN[0] = Infinity), (SPAN[1] = -Infinity);
    // (Presence only: the first visible point usually answers.)
    this.series.extent(a, a, positive, SPAN);
    if (!(SPAN[0] <= SPAN[1]) && a < b) this.series.extent(a + 1, b, positive, SPAN);
    if (!(SPAN[0] <= SPAN[1])) return;
    out[0] = Math.max(out[0], this.roomAbove);
    out[1] = Math.max(out[1], this.roomBelow);
  }

  /** @param {CanvasRenderingContext2D} g @param {number} n */
  #moreWidth(g, n) {
    return (this.moreWidths[n] ??= g.measureText((MORE[n] ??= `${n} more`)).width);
  }

  /**
     * Read from values, the zoom and the scale's reach, never from drawn words: a pan or a refit regroups none.
     * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
     * @param {Float64Array} highs @param {Float64Array} lows @param {Float64Array} values
     */
  #say(g, time, scale, highs, lows, values) {
    const spacing = time.spacing, end = (time.length - 1) * spacing + time.rightOffset, version = this.series.version, mode = scale.mode, reach = scale.reach, reached = scale.reached;
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
    let from = n;
    for (let k = n - 1; k >= 0; k--) if (said[k]) this.shown[--from] = k;
    this.shownFrom = from;
  }

  /** @param {number} k */
  #into(k) {
    const marks = this.marks, n = marks.length, spacing = this.saidAt, half = (this.widths[k] + PAD) / 2;
    const from = MID[k] - half - ROOM[k], to = MID[k] + half;
    let into = -1, near = Infinity;
    for (let s = 0; s < 2; s++) {
      for (let j = before(SHOWN[s], k); j >= 0 && marks[j].at * spacing + FAR > from; j = before(SHOWN[s], j))
        if (Math.abs(MID[j] - MID[k]) < near && this.#runs(k, from, to, j, ROOM[j])) (into = j), (near = Math.abs(MID[j] - MID[k]));
      for (let j = after(SHOWN[s], k, n); j < n && marks[j].at * spacing - FAR - ROOMS < to; j = after(SHOWN[s], j, n))
        if (Math.abs(MID[j] - MID[k]) < near && this.#runs(k, from, to, j, ROOM[j])) (into = j), (near = Math.abs(MID[j] - MID[k]));
    }
    return into;
  }

  /** @param {number} k */
  #crowds(k) {
    const marks = this.marks, n = marks.length, rank = this.rank, spacing = this.saidAt, half = (this.widths[k] + PAD) / 2;
    const from = MID[k] - half, to = MID[k] + half;
    for (let j = k - 1; j >= 0 && marks[j].at * spacing + FAR > from; j--) if (marks[j].text && rank[j] > rank[k] && this.#runs(k, from, to, j, 0)) return true;
    for (let j = k + 1; j < n && marks[j].at * spacing - FAR < to; j++) if (marks[j].text && rank[j] > rank[k] && this.#runs(k, from, to, j, 0)) return true;
    return false;
  }

  /**
   * Never where only one has a priority: neither ranks against the other, so neither is counted in the other.
   * @param {number} k @param {number} from @param {number} to @param {number} j @param {number} room
   */
  #runs(k, from, to, j, room) {
    const marks = this.marks, half = (this.widths[j] + PAD) / 2;
    if (marks[j].at === marks[k].at || (marks[j].priority === undefined) !== (marks[k].priority === undefined)) return false;
    if (from >= MID[j] + half + CLEAR || to + CLEAR <= MID[j] - half - room) return false;
    return this.#meet(k, j);
  }

  /**
     * Whether k's and j's rows can meet at some pan at this zoom.
     * @param {number} k @param {number} j
     */
  #meet(k, j) {
    const d = HEIGHT[k] - HEIGHT[j], by = (PIXELS / Math.min(VIEWS[k], VIEWS[j])) * d || 0, rows = this.rows, near = PILL + CLEAR;
    return (d < 0 || rows[2 * k + 1] - rows[2 * j] - by > -near) && (d > 0 || rows[2 * k] - rows[2 * j + 1] - by < near);
  }

  /**
   * Into AT: its dot's middle and its words' (y).
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
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} _dpr @param {{ background: string, font: string }} theme
   */
  draw(g, time, scale, x0, x1, _dpr, theme) {
    const series = this.series, marks = this.marks, n = marks.length;
    if (series.hidden || !n) return;
    const values = /** @type {Float64Array} */ (series.close ?? series.upper ?? series.values), highs = series.tops?.(time) ?? values, lows = series.feet?.(time) ?? values;
    g.font = theme.font;
    this.#say(g, time, scale, highs, lows, values);
    // (A pixel past the ring, for its smoothed edge.)
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

  /** Its dots again, over the crosshair's lines. @param {Parameters<Marks["draw"]>} args */
  drawDots(...args) {
    this.draw(...args);
  }

  /**
     * Drawn whole each frame over the pane, so words keep to the view; each pill a sprite on whole device pixels.
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
      const past = (BOXES[b + 5] - BOXES[b + 7]) * (marks[k].position === "above" ? 1 : -1) > 0, sprites = past ? this.outlinedSprites : this.sprites;
      const count = alone ? words : left < words ? left : end < right ? end + SEP : NaN;
      if (!alone) place(g, (sprites[k] ??= sprite(/** @type {string} */ (marks[k].text), past ? outlined(look.fill, theme.background) : look, widths[k] + PAD, theme.font, dpr)), words, top, dpr);
      if (count === count) place(g, (this.moreSprites[held] ??= sprite(MORE[held], { fill: countFill, ink: countInk }, this.#moreWidth(g, held) + PAD, theme.font, dpr)), count, top, dpr);
      drew(left, top, right - left, PILL);
    }
  }

  /**
   * Into DOTS, in their points' order.
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

/** @param {string} words @param {{ fill: string, ink: string, edge?: string }} look @param {number} width @param {string} font @param {number} dpr */
function sprite(words, look, width, font, dpr) {
  const c = document.createElement("canvas");
  (c.width = Math.ceil(width * dpr)), (c.height = Math.ceil(PILL * dpr));
  const g = /** @type {CanvasRenderingContext2D} */ (c.getContext("2d"));
  g.scale(dpr, dpr);
  (g.font = font), (g.textBaseline = "middle");
  pill(g, words, look.fill, look.ink, 0, PILL / 2, "left", PILL, look.edge);
  return c;
}

/** On whole device pixels, so it's copied as drawn, never resampled.
 * @param {CanvasRenderingContext2D} g @param {HTMLCanvasElement} image @param {number} x @param {number} y @param {number} dpr */
function place(g, image, x, y, dpr) {
  g.drawImage(image, Math.round(x * dpr) / dpr, Math.round(y * dpr) / dpr, image.width / dpr, image.height / dpr);
}

/** @param {Mark} mark */
function dotOf(mark) {
  const r = DOT * (mark.size ?? 1);
  return r ? 2 * (r + RING) : 0;
}

/** BOXES per words: mark (-1: left out), left, pill's left, pill's right, right, middle's y, dot's x and y.
 * @param {number} s @param {number} k @param {number} left @param {number} words @param {number} end @param {number} right
 * @param {number} y @param {number} x @param {number} dot */
function box(s, k, left, words, end, right, y, x, dot) {
  const b = s * BOX;
  (BOXES[b] = k), (BOXES[b + 1] = left), (BOXES[b + 2] = words), (BOXES[b + 3] = end), (BOXES[b + 4] = right), (BOXES[b + 5] = y), (BOXES[b + 6] = x), (BOXES[b + 7] = dot);
}

/** @param {number} s @param {number} placed @param {number} plot @param {number} height @param {number} out */
function free(s, placed, plot, height, out) {
  const b = s * BOX, was = BOXES[b + 2], pw = BOXES[b + 3] - was, y = BOXES[b + 5];
  for (let cw = was - BOXES[b + 1]; ; cw = 0) {
    const width = cw + pw, most = plot - CLEAR - width;
    let a = Math.min(was - cw, most), c = Math.max(was, CLEAR), flip = a < CLEAR, left = flip ? c : a, to = y;
    let least = left <= most ? 0 : Infinity;
    if (!least && hit(s, left, width, y, placed)) {
      // (Each walk stops once past the nearest spot found before it: that one wins anyway.)
      while (a >= CLEAR && hit(s, a, width, y, placed)) a = HIT[0];
      const byA = a >= CLEAR && ours(s, a + cw, y) ? was - cw - a : Infinity;
      while (c <= most && c - was < byA && hit(s, c, width, y, placed)) c = HIT[1];
      const byC = c <= most && c - was < byA && ours(s, c, y) ? c - was : Infinity, best = Math.min(byA, byC);
      const start = flip ? left : left + cw, off = Math.abs(start - was);
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
 * Into HIT. Each bound tested as found: rounded once more, a step to one could land on its edge again and again.
 * @param {number} s @param {number} left @param {number} width @param {number} y @param {number} placed
 */
function hit(s, left, width, y, placed) {
  // (Each bound worked out only once the ones before it hold: faster without a JIT.)
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
 * Nearer its own dot than any other, so moved words never read as another's.
 * @param {number} s @param {number} left @param {number} y
 */
function ours(s, left, y) {
  const b = s * BOX, right = left + BOXES[b + 3] - BOXES[b + 2], near = apart(BOXES[b + 6], BOXES[b + 7], left, right, y), k = BOXES[b];
  for (let d = firstDot(left - near); d < DOTN && DOTS[d * 4] <= right + near; d++)
    if (DOTS[d * 4 + 3] !== k && apart(DOTS[d * 4], DOTS[d * 4 + 1], left, right, y) <= near) return false;
  return true;
}

/** @param {number} px @param {number} py @param {number} left @param {number} right @param {number} y */
function apart(px, py, left, right, y) {
  return Math.hypot(Math.max(left - px, 0, px - right), Math.max(Math.abs(py - y) - PILL / 2, 0));
}

/** @param {number} x */
function firstDot(x) {
  let d = 0;
  for (let hi = DOTN; d < hi; ) {
    const m = (d + hi) >> 1;
    if (DOTS[m * 4] < x) d = m + 1;
    else hi = m;
  }
  return d;
}

/** -1: none. @param {Int32Array} bits @param {number} i */
function before(bits, i) {
  if (i <= 0) return -1;
  let w = (i - 1) >> 5, word = bits[w] & (-1 >>> (31 - ((i - 1) & 31)));
  while (!word) {
    if (--w < 0) return -1;
    word = bits[w];
  }
  return (w << 5) + 31 - Math.clz32(word);
}

/** `n`: none. @param {Int32Array} bits @param {number} i @param {number} n */
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

const AT = new Float64Array(2), HIT = new Float64Array(4);
let BOXES = new Float64Array(BOX * 16), ORDER = new Int32Array(16), DOTS = new Float64Array(64), DOTN = 0, REACH = 0;
let MID = new Float64Array(64), HEIGHT = new Float64Array(64), ROOM = new Float64Array(64), VIEWS = new Float64Array(64), PIXELS = 0, SHOWN = [new Int32Array(2), new Int32Array(2)], FAR = 0, ROOMS = 0;
