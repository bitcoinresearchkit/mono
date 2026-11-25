// @ts-check
// A pane's value scale: linear or logarithmic, fitted to what its series show (or set by hand), with room kept at its
// top and bottom; values to y and back, and its ticks: round numbers 28 CSS pixels apart at least. Or relative: each series
// read against its first value in view (set by its pane before it's fitted or drawn: setBase), as its percent change
// from it ("percent"), or with it at 100 ("indexed"); the scale's range, ticks and values then in those terms.

/** @typedef {"linear" | "log" | "percent" | "indexed"} ScaleMode */

/** An indexed scale's reading of a change of `percent` from its base: from 100, whatever the base's sign (a rise still reads as one). The one rule every reading of an indexed value follows. @param {number} percent */
export const indexed = (percent) => 100 + percent;

/** Ticks' least room between them (CSS pixels): two and a half lines of text, as the chart library marked them. */
export const EVERY = 28;
/**
 * A log scale's powers of ten's least room between them (CSS pixels): half its other ticks', so a short pane still names
 * each decade it spans (0.1 to 1M on a phone's 120 pixels), not every other.
 */
export const EVERY_POWER = EVERY / 2;
// (A log scale's ticks in a decade, the sparsest first, and each one's least gap (in decades): the densest with room.)
const DECADE = [[1], [1, 3], [1, 2, 5], [1, 2, 3, 5], [1, 2, 3, 4, 5, 6, 8], [1, 2, 3, 4, 5, 6, 7, 8, 9]];
const DECADE_GAP = DECADE.map((set) => Math.min(...set.map((v, i) => Math.log10((set[i + 1] ?? 10) / v))));
/** Round numbers a log scale's ticks might be instead, made once (grown). */
let ROUND = new Float64Array(24);

export class Scale {
  // (What it shows, behind accessors: any change marks its mapping stale, made again at the next y().)
  #mode = /** @type {ScaleMode} */ ("linear");
  #m = 1;
  #c = 0;
  #top = 0;
  #bottom = 0;
  #above = 0;
  #below = 0;
  #lo = 0;
  #hi = 1;
  #height = 1;
  // (Its mapping, as y() reads it: y = foot - (t - ta) / span * room, t a value as it reads it (v * m + c, or its log),
  // ta its range's low end so read: no large terms cancelling; the offset divided by the span first (a slope
  // room / span overflows over the narrowest ranges).)
  #stale = true;
  #log = false;
  #ta = 0;
  #span = 1;
  #room = 1;
  #foot = 1;

  /** @param {{ mode?: ScaleMode, top?: number, bottom?: number }} [options] */
  constructor({ mode = "linear", top = 0.1, bottom = 0.08 } = {}) {
    (this.#mode = mode), (this.#top = top), (this.#bottom = bottom);
    // (Room asked for above and below, in CSS pixels: see above and below.)
    this.askedAbove = 0;
    this.askedBelow = 0;
    /** Fitted to nothing (its pane's series show nothing in view): its range kept, no ticks. */
    this.empty = false;
    /**
     * What it's fitted to in any view within indices `a` to `b`, at the most (its series' range there, widening `out`;
     * none where that isn't known from their values alone), set by its pane (see Pane's reach); and a number that
     * changes whenever that could give another answer (its series or their values changed, its range held or let go).
     * @type {(time: import("./timeline.js").Timeline, a: number, b: number, out: Float64Array) => void}
     */
    this.reach = () => {};
    this.reached = 0;
    // (As drawn: where its picture can be moved, it's unchanged.)
    this.drawnLo = NaN;
    this.drawnHi = NaN;
    this.drawnHeight = NaN;
    this.drawnTop = NaN;
    this.drawnBottom = NaN;
    this.drawnAbove = NaN;
    this.drawnBelow = NaN;
    this.drawnMode = "";
    this.drawnEmpty = false;
  }

  get mode() {
    return this.#mode;
  }
  set mode(mode) {
    (this.#mode = mode), (this.#stale = true);
    // (A scale left relative reads raw values again, without a fitting pass clearing its base.)
    if (!this.relative) (this.#m = 1), (this.#c = 0);
  }
  /** A relative scale's reading of the series drawn now: t = v * m + c (see setBase). */
  get m() {
    return this.#m;
  }
  get c() {
    return this.#c;
  }
  /**
   * Room kept at the top and the bottom, as shares of the height (top, bottom); and more, in CSS pixels (above, below:
   * its series' marks over their points, as asked, and as kept: see Pane's fit).
   */
  get top() {
    return this.#top;
  }
  set top(top) {
    (this.#top = top), (this.#stale = true);
  }
  get bottom() {
    return this.#bottom;
  }
  set bottom(bottom) {
    (this.#bottom = bottom), (this.#stale = true);
  }
  get above() {
    return this.#above;
  }
  set above(above) {
    (this.#above = above), (this.#stale = true);
  }
  get below() {
    return this.#below;
  }
  set below(below) {
    (this.#below = below), (this.#stale = true);
  }
  /** The range shown, as values. */
  get lo() {
    return this.#lo;
  }
  set lo(lo) {
    (this.#lo = lo), (this.#stale = true);
  }
  get hi() {
    return this.#hi;
  }
  set hi(hi) {
    (this.#hi = hi), (this.#stale = true);
  }
  /** The height in CSS pixels. */
  get height() {
    return this.#height;
  }
  set height(height) {
    (this.#height = height), (this.#stale = true);
  }

  /** Fitted to values from `lo` to `hi` (none: as it was). @param {number} lo @param {number} hi */
  fit(lo, hi) {
    if (!(lo <= hi)) return;
    if (this.mode === "log" && !(lo > 0)) return;
    if (lo === hi) (this.mode === "log" ? ((lo /= 1.1), (hi *= 1.1)) : ((lo -= Math.abs(lo) * 0.1 || 1), (hi += Math.abs(hi) * 0.1 || 1)));
    this.lo = lo;
    this.hi = hi;
  }

  get relative() {
    return this.#mode === "percent" || this.#mode === "indexed";
  }

  /**
   * A relative scale's series read against `base` (its first value in view): its percent change from it, or 100 at it,
   * either way (a negative base: a rise still reads as one, over |base|). Else, as they are.
   * @param {number} base
   */
  setBase(base) {
    if (!this.relative || !base || base !== base) return void ((this.#m = 1), (this.#c = 0));
    const size = Math.abs(base);
    this.#m = 100 / size;
    const offset = (-100 * base) / size;
    this.#c = this.mode === "indexed" ? indexed(offset) : offset;
  }

  /** Its mapping made again (see #foot). */
  #remap() {
    const log = this.#mode === "log", ta = log ? Math.log10(this.#lo) : this.#lo, tb = log ? Math.log10(this.#hi) : this.#hi;
    const top = this.#top * this.#height + this.#above, room = this.#height - top - this.#bottom * this.#height - this.#below;
    (this.#log = log), (this.#ta = ta), (this.#span = tb - ta), (this.#room = room), (this.#foot = top + room), (this.#stale = false);
  }

  /** The y of a value (NaN for none, or one a log scale can't show). @param {number} v */
  y(v) {
    if (this.#stale) this.#remap();
    return this.#foot - (((this.#log ? Math.log10(v) : v * this.#m + this.#c) - this.#ta) / this.#span) * this.#room;
  }

  /** The y of a value in its own terms (a tick's, a relative scale's percent or index: not read against a base). @param {number} t */
  own(t) {
    if (this.#stale) this.#remap();
    return this.#foot - (((this.#log ? Math.log10(t) : t) - this.#ta) / this.#span) * this.#room;
  }

  /** The value at a y, in its own terms. @param {number} y */
  value(y) {
    if (this.#stale) this.#remap();
    const t = this.#ta + ((this.#foot - y) / this.#room) * this.#span;
    return this.#log ? 10 ** t : t;
  }

  /** Whether it shows what it showed when last drawn. */
  get unchanged() {
    return this.drawnLo === this.lo && this.drawnHi === this.hi && this.drawnHeight === this.height && this.drawnTop === this.top && this.drawnBottom === this.bottom && this.drawnAbove === this.above && this.drawnBelow === this.below && this.drawnMode === this.mode && this.drawnEmpty === this.empty;
  }

  drawnNow() {
    (this.drawnLo = this.lo), (this.drawnHi = this.hi), (this.drawnHeight = this.height), (this.drawnTop = this.top), (this.drawnBottom = this.bottom), (this.drawnAbove = this.above), (this.drawnBelow = this.below), (this.drawnMode = this.mode), (this.drawnEmpty = this.empty);
  }

  /**
   * Its ticks written into `out`, their count returned: round numbers (1, 2, 2.5 or 5 a power of ten) EVERY CSS
   * pixels apart at least (two and a half lines of text: as the chart library marked them); on a log scale, its powers
   * of ten (EVERY_POWER pixels apart at least; every few, where they'd be closer), each with as many of its 2 to 9 as
   * have room (3; 2 and 5; 2, 3 and 5; 2 to 6 and 8; all); within a decade, those or round numbers, whichever marks it
   * more evenly. None where it's empty.
   * @param {Float64Array} out
   */
  ticks(out) {
    if (this.empty) return 0;
    const lo = this.value(this.height), hi = this.value(0), want = Math.max(2, this.height / EVERY);
    let count = 0;
    // (A log scale: its powers of ten, and their 2s and 5s with room; within a decade, those or round numbers a step apart
    // as a linear scale's, whichever marks it more evenly (0.6 to 5: 1, 2, 3, 5; 60K to 80K: 62K, 64K… 80K).)
    const decades = this.mode === "log" ? Math.log10(hi) - Math.log10(lo) : 0;
    if (decades > 0) {
      const perDecade = this.height / decades;
      let set = 0;
      while (set + 1 < DECADE.length && DECADE_GAP[set + 1] * perDecade >= EVERY) set++;
      // (Each decade, its powers of ten EVERY_POWER pixels apart at least, from the one under the first in view, its 2s
      // and 5s in view. Where even that's too close, every few: those a multiple of their count (1, 1K, 1M), the same
      // whatever's in view, so they move with it.)
      const steps = DECADE[set], every10 = perDecade >= EVERY_POWER ? 1 : Math.ceil(EVERY_POWER / perDecade);
      for (let p = Math.ceil(Math.floor(Math.log10(lo)) / every10) * every10; p <= Math.ceil(Math.log10(hi)) && count < out.length; p += every10)
        for (let k = 0; k < steps.length; k++) {
          const v = steps[k] * 10 ** p;
          if (v >= lo && v <= hi && count < out.length) out[count++] = v;
        }
      // (Its top, where it's room for more past its last power or step (where the price often is: 100K to 150K), filled
      // with round numbers a step apart, as a linear scale's there, each `every` pixels from its ticks and half that from
      // its edge: 120K, 140K, as the chart library marked them. Elsewhere its decades' steps alone: even.)
      let filled = 0;
      {
        const a = count ? out[count - 1] : lo, b = hi, fromA = count ? EVERY : EVERY / 2, fromB = EVERY / 2;
        const raw = (EVERY * b * Math.LN10) / perDecade, power = 10 ** Math.floor(Math.log10(raw)), unit = raw / power;
        const step = (unit <= 1 ? 1 : unit <= 2 ? 2 : unit <= 2.5 ? 2.5 : unit <= 5 ? 5 : 10) * power;
        // (Its tries counted: a range too narrow for a step to move (a number's last digit) ends them.)
        for (let k = 0, v = Math.ceil(a / step) * step; k < out.length && v < b && count + filled < out.length; k++, v += step)
          if (Math.log10(v / a) * perDecade >= fromA && Math.log10(b / v) * perDecade >= fromB) out[count + filled++] = v;
      }
      count += filled;
      if (decades >= 1) return count;
    }
    // (Round numbers a step apart: a log scale's (under a decade) closest at its top, where a value takes fewest pixels.)
    const raw = this.mode === "log" ? (EVERY * hi * Math.LN10 * decades) / this.height : (hi - lo) / want, power = 10 ** Math.floor(Math.log10(raw)), unit = raw / power;
    const step = (unit <= 1 ? 1 : unit <= 2 ? 2 : unit <= 2.5 ? 2.5 : unit <= 5 ? 5 : 10) * power, round = ROUND.length < out.length ? (ROUND = new Float64Array(out.length)) : ROUND;
    let rounds = 0;
    for (let v = Math.ceil(lo / step) * step; v <= hi && rounds < out.length; v += step) round[rounds++] = Math.abs(v) < step * 1e-9 ? 0 : v;
    // (Too short for two steps of it (a log scale's few pixels): one round number in view at least.)
    if (!rounds && out.length) {
      const one = 10 ** Math.floor(Math.log10(hi - lo || Math.abs(hi) || 1)), v = Math.ceil(lo / one) * one;
      if (v <= hi) round[rounds++] = v;
    }
    // (A log scale's: whichever leaves the least of it unmarked (its widest gap, its ends' too), its decades' on a tie.)
    if (decades > 0) {
      const widest = (/** @type {Float64Array} */ list, /** @type {number} */ n) => {
        let most = 0, at = lo;
        for (let k = 0; k < n; k++) (most = Math.max(most, list[k] / at)), (at = list[k]);
        return Math.max(most, hi / at);
      };
      if (!rounds || widest(round, rounds) >= widest(out, count)) return count;
    }
    for (let k = 0; k < rounds; k++) out[k] = round[k];
    return rounds;
  }
}
