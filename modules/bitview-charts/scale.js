// @ts-check
// A pane's value scale: linear or logarithmic, fitted to what its series show (or set by hand), with room kept at its
// top and bottom; values to y and back, and its ticks: round numbers about every 50 CSS pixels. Or relative: each series
// read against its first value in view (set by its pane before it's fitted or drawn: setBase), as its percent change
// from it ("percent"), or with it at 100 ("indexed"); the scale's range, ticks and values then in those terms.

/** @typedef {"linear" | "log" | "percent" | "indexed"} ScaleMode */

// (A log scale's ticks in a decade, the sparsest first, and each one's least gap (in decades): the densest with room.)
const DECADE = [[1], [1, 3], [1, 2, 5], [1, 2, 3, 4, 5, 6, 8], [1, 2, 3, 4, 5, 6, 7, 8, 9]];
const DECADE_GAP = DECADE.map((set) => Math.min(...set.map((v, i) => Math.log10((set[i + 1] ?? 10) / v))));

export class Scale {
  /** @param {{ mode?: ScaleMode, top?: number, bottom?: number }} [options] */
  constructor({ mode = "linear", top = 0.1, bottom = 0.08 } = {}) {
    this.mode = mode;
    // (A relative scale's reading of the series drawn now: t = v * m + c.)
    this.m = 1;
    this.c = 0;
    /** Room kept at the top and the bottom, as shares of the height. */
    this.top = top;
    this.bottom = bottom;
    /** The range shown, as values. */
    this.lo = 0;
    this.hi = 1;
    /** The height in CSS pixels. */
    this.height = 1;
    /** Fitted to nothing (its pane's series show nothing in view): its range kept, no ticks. */
    this.empty = false;
    // (As drawn: where its picture can be moved, it's unchanged.)
    this.drawnLo = NaN;
    this.drawnHi = NaN;
    this.drawnHeight = NaN;
    this.drawnTop = NaN;
    this.drawnBottom = NaN;
    this.drawnMode = "";
    this.drawnEmpty = false;
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
    return this.mode === "percent" || this.mode === "indexed";
  }

  /**
   * A relative scale's series read against `base` (its first value in view): its percent change from it, or 100 at it
   * (a negative base read the other way, as the chart library did). Else, as they are.
   * @param {number} base
   */
  setBase(base) {
    if (!this.relative || !base || base !== base) return void ((this.m = 1), (this.c = 0));
    const size = Math.abs(base);
    this.m = 100 / size;
    this.c = (-100 * base) / size + (this.mode === "indexed" ? (base < 0 ? -100 : 100) : 0);
  }

  /** An end of its range as it maps (its range is in its own terms: a relative one's already read). @param {number} v */
  #end(v) {
    return this.mode === "log" ? Math.log10(v) : v;
  }

  /** @param {number} v */
  #t(v) {
    return this.mode === "log" ? Math.log10(v) : v * this.m + this.c;
  }

  /** The y of a value (NaN for none, or one a log scale can't show). @param {number} v */
  y(v) {
    const a = this.#end(this.lo), b = this.#end(this.hi), top = this.top * this.height, room = this.height - top - this.bottom * this.height;
    return top + (1 - (this.#t(v) - a) / (b - a)) * room;
  }

  /** The y of a value in its own terms (a tick's, a relative scale's percent or index: not read against a base). @param {number} t */
  own(t) {
    const a = this.#end(this.lo), b = this.#end(this.hi), top = this.top * this.height, room = this.height - top - this.bottom * this.height;
    return top + (1 - (this.#end(t) - a) / (b - a)) * room;
  }

  /** The value at a y, in its own terms. @param {number} y */
  value(y) {
    const a = this.#end(this.lo), b = this.#end(this.hi), top = this.top * this.height, room = this.height - top - this.bottom * this.height;
    const t = a + (1 - (y - top) / room) * (b - a);
    return this.mode === "log" ? 10 ** t : t;
  }

  /** Whether it shows what it showed when last drawn. */
  get unchanged() {
    return this.drawnLo === this.lo && this.drawnHi === this.hi && this.drawnHeight === this.height && this.drawnTop === this.top && this.drawnBottom === this.bottom && this.drawnMode === this.mode && this.drawnEmpty === this.empty;
  }

  drawnNow() {
    (this.drawnLo = this.lo), (this.drawnHi = this.hi), (this.drawnHeight = this.height), (this.drawnTop = this.top), (this.drawnBottom = this.bottom), (this.drawnMode = this.mode), (this.drawnEmpty = this.empty);
  }

  /**
   * Its ticks written into `out`, their count returned: round numbers (1, 2, 2.5 or 5 a power of ten) `every` CSS
   * pixels apart at least (two and a half lines of text: as the chart library marked them); on a log scale over a decade
   * or more, its powers of ten (every few, where they'd be closer), each with as many of its 2 to 9 as have room (3; 2
   * and 5; 2 to 6 and 8; all). None where it's empty.
   * @param {Float64Array} out @param {number} [every]
   */
  ticks(out, every = 28) {
    if (this.empty) return 0;
    const lo = this.value(this.height), hi = this.value(0), want = Math.max(2, this.height / every);
    let count = 0;
    // (A log scale over a decade or more: its powers of ten, and their 2s and 5s with room; within one, as a linear
    // scale's: round numbers, else it'd show one or none.)
    const decades = this.mode === "log" ? Math.log10(hi / lo) : 0;
    if (decades >= 1) {
      const perDecade = this.height / decades;
      let set = 0;
      while (set + 1 < DECADE.length && DECADE_GAP[set + 1] * perDecade >= every) set++;
      // (Every few decades: from the first power of ten in view (a range of a decade or more has one), else from the one
      // under it, its 2s and 5s in view.)
      const steps = DECADE[set], every10 = perDecade >= every ? 1 : Math.ceil(every / perDecade);
      for (let p = every10 > 1 ? Math.ceil(Math.log10(lo)) : Math.floor(Math.log10(lo)); p <= Math.ceil(Math.log10(hi)) && count < out.length; p += every10)
        for (let k = 0; k < steps.length; k++) {
          const v = steps[k] * 10 ** p;
          if (v >= lo && v <= hi && count < out.length) out[count++] = v;
        }
      return count;
    }
    // (Round numbers a step apart: a log scale's (under a decade) closest at its top, where a value takes fewest pixels.)
    const raw = this.mode === "log" ? (every * hi * Math.LN10 * decades) / this.height : (hi - lo) / want, power = 10 ** Math.floor(Math.log10(raw)), unit = raw / power;
    const step = (unit <= 1 ? 1 : unit <= 2 ? 2 : unit <= 2.5 ? 2.5 : unit <= 5 ? 5 : 10) * power;
    for (let v = Math.ceil(lo / step) * step; v <= hi && count < out.length; v += step) out[count++] = Math.abs(v) < step * 1e-9 ? 0 : v;
    // (Too short for two steps of it (a log scale's few pixels): one round number in view at least.)
    if (!count && out.length) {
      const one = 10 ** Math.floor(Math.log10(hi - lo || Math.abs(hi) || 1)), v = Math.ceil(lo / one) * one;
      if (v <= hi) out[count++] = v;
    }
    return count;
  }
}
