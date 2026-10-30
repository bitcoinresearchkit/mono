// @ts-check
// A pane's value scale: linear or logarithmic, fitted to what its series show (or set by hand), with room kept at its
// top and bottom; values to y and back, and its ticks: round numbers about every 50 CSS pixels.

/** @typedef {"linear" | "log"} ScaleMode */

// (A log scale's ticks in a power of ten: all three with room, else the power alone.)
const THIRDS = [1, 2, 5], ONE = [1];

export class Scale {
  /** @param {{ mode?: ScaleMode, top?: number, bottom?: number }} [options] */
  constructor({ mode = "linear", top = 0.1, bottom = 0.08 } = {}) {
    this.mode = mode;
    /** Room kept at the top and the bottom, as shares of the height. */
    this.top = top;
    this.bottom = bottom;
    /** The range shown, as values. */
    this.lo = 0;
    this.hi = 1;
    /** The height in CSS pixels. */
    this.height = 1;
    // (As drawn: where its picture can be moved, it's unchanged.)
    this.drawnLo = NaN;
    this.drawnHi = NaN;
    this.drawnHeight = NaN;
    this.drawnTop = NaN;
    this.drawnBottom = NaN;
    this.drawnMode = "";
  }

  /** Fitted to values from `lo` to `hi` (none: as it was). @param {number} lo @param {number} hi */
  fit(lo, hi) {
    if (!(lo <= hi)) return;
    if (this.mode === "log" && !(lo > 0)) return;
    if (lo === hi) (this.mode === "log" ? ((lo /= 1.1), (hi *= 1.1)) : ((lo -= Math.abs(lo) * 0.1 || 1), (hi += Math.abs(hi) * 0.1 || 1)));
    this.lo = lo;
    this.hi = hi;
  }

  /** @param {number} v */
  #t(v) {
    return this.mode === "log" ? Math.log10(v) : v;
  }

  /** The y of a value (NaN for none, or one a log scale can't show). @param {number} v */
  y(v) {
    const a = this.#t(this.lo), b = this.#t(this.hi), top = this.top * this.height, room = this.height - top - this.bottom * this.height;
    return top + (1 - (this.#t(v) - a) / (b - a)) * room;
  }

  /** The value at a y. @param {number} y */
  value(y) {
    const a = this.#t(this.lo), b = this.#t(this.hi), top = this.top * this.height, room = this.height - top - this.bottom * this.height;
    const t = a + (1 - (y - top) / room) * (b - a);
    return this.mode === "log" ? 10 ** t : t;
  }

  /** Whether it shows what it showed when last drawn. */
  get unchanged() {
    return this.drawnLo === this.lo && this.drawnHi === this.hi && this.drawnHeight === this.height && this.drawnTop === this.top && this.drawnBottom === this.bottom && this.drawnMode === this.mode;
  }

  drawnNow() {
    (this.drawnLo = this.lo), (this.drawnHi = this.hi), (this.drawnHeight = this.height), (this.drawnTop = this.top), (this.drawnBottom = this.bottom), (this.drawnMode = this.mode);
  }

  /**
   * Its ticks written into `out`, their count returned: round numbers (1, 2, 2.5 or 5 a power of ten) about `every`
   * CSS pixels apart; on a log scale over a decade or more, each power of ten, and its 2 and 5 when there's room.
   * @param {Float64Array} out @param {number} [every]
   */
  ticks(out, every = 50) {
    const lo = this.value(this.height), hi = this.value(0), want = Math.max(2, this.height / every);
    let count = 0;
    // (A log scale over a decade or more: its powers of ten, and their 2s and 5s with room; within one, as a linear
    // scale's: round numbers, else it'd show one or none.)
    const decades = this.mode === "log" ? Math.log10(hi / lo) : 0;
    if (decades >= 1) {
      const steps = decades * 3 <= want ? THIRDS : ONE, every10 = decades <= want ? 1 : Math.ceil(decades / want);
      for (let p = Math.floor(Math.log10(lo)); p <= Math.ceil(Math.log10(hi)) && count < out.length; p += every10)
        for (let k = 0; k < steps.length; k++) {
          const v = steps[k] * 10 ** p;
          if (v >= lo && v <= hi && count < out.length) out[count++] = v;
        }
      return count;
    }
    const raw = (hi - lo) / want, power = 10 ** Math.floor(Math.log10(raw)), unit = raw / power;
    const step = (unit <= 1 ? 1 : unit <= 2 ? 2 : unit <= 2.5 ? 2.5 : unit <= 5 ? 5 : 10) * power;
    for (let v = Math.ceil(lo / step) * step; v <= hi && count < out.length; v += step) out[count++] = Math.abs(v) < step * 1e-9 ? 0 : v;
    return count;
  }
}
