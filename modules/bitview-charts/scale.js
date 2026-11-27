// @ts-check

/** @typedef {"linear" | "log" | "percent" | "indexed"} ScaleMode */

/** From 100 whatever the base's sign. @param {number} percent */
export const indexed = (percent) => 100 + percent;

export const EVERY = 28;
/** Half the other ticks': a short pane still names each decade it spans, not every other. */
export const EVERY_POWER = EVERY / 2;
const DECADE = [[1], [1, 3], [1, 2, 5], [1, 2, 3, 5], [1, 2, 3, 4, 5, 6, 8], [1, 2, 3, 4, 5, 6, 7, 8, 9]];
const DECADE_GAP = DECADE.map((set) => Math.min(...set.map((v, i) => Math.log10((set[i + 1] ?? 10) / v))));
let ROUND = new Float64Array(24);

export class Scale {
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
  // The offset is divided by the span first: a slope room / span overflows over the narrowest ranges.
  #stale = true;
  #log = false;
  #ta = 0;
  #span = 1;
  #room = 1;
  #foot = 1;

  /** @param {{ mode?: ScaleMode, top?: number, bottom?: number }} [options] */
  constructor({ mode = "linear", top = 0.1, bottom = 0.08 } = {}) {
    (this.#mode = mode), (this.#top = top), (this.#bottom = bottom);
    this.askedAbove = 0;
    this.askedBelow = 0;
    this.empty = false;
    /** @type {(time: import("./timeline.js").Timeline, a: number, b: number, out: Float64Array) => void} */
    this.reach = () => {};
    this.reached = 0;
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
    if (!this.relative) (this.#m = 1), (this.#c = 0);
  }
  get m() {
    return this.#m;
  }
  get c() {
    return this.#c;
  }
  /** top, bottom: shares of the height; above, below: CSS pixels (see Pane's fit). */
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
  get height() {
    return this.#height;
  }
  set height(height) {
    (this.#height = height), (this.#stale = true);
  }

  /** @param {number} lo @param {number} hi */
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

  /** @param {number} base */
  setBase(base) {
    if (!this.relative || !base || base !== base) return void ((this.#m = 1), (this.#c = 0));
    const size = Math.abs(base);
    this.#m = 100 / size;
    const offset = (-100 * base) / size;
    this.#c = this.mode === "indexed" ? indexed(offset) : offset;
  }

  #remap() {
    const log = this.#mode === "log", ta = log ? Math.log10(this.#lo) : this.#lo, tb = log ? Math.log10(this.#hi) : this.#hi;
    const top = this.#top * this.#height + this.#above, room = this.#height - top - this.#bottom * this.#height - this.#below;
    (this.#log = log), (this.#ta = ta), (this.#span = tb - ta), (this.#room = room), (this.#foot = top + room), (this.#stale = false);
  }

  /** NaN: none. @param {number} v */
  y(v) {
    if (this.#stale) this.#remap();
    return this.#foot - (((this.#log ? Math.log10(v) : v * this.#m + this.#c) - this.#ta) / this.#span) * this.#room;
  }

  /** @param {number} t */
  own(t) {
    if (this.#stale) this.#remap();
    return this.#foot - (((this.#log ? Math.log10(t) : t) - this.#ta) / this.#span) * this.#room;
  }

  /** @param {number} y */
  value(y) {
    if (this.#stale) this.#remap();
    const t = this.#ta + ((this.#foot - y) / this.#room) * this.#span;
    return this.#log ? 10 ** t : t;
  }

  get unchanged() {
    return this.drawnLo === this.lo && this.drawnHi === this.hi && this.drawnHeight === this.height && this.drawnTop === this.top && this.drawnBottom === this.bottom && this.drawnAbove === this.above && this.drawnBelow === this.below && this.drawnMode === this.mode && this.drawnEmpty === this.empty;
  }

  drawnNow() {
    (this.drawnLo = this.lo), (this.drawnHi = this.hi), (this.drawnHeight = this.height), (this.drawnTop = this.top), (this.drawnBottom = this.bottom), (this.drawnAbove = this.above), (this.drawnBelow = this.below), (this.drawnMode = this.mode), (this.drawnEmpty = this.empty);
  }

  /**
   * @param {Float64Array} out
   */
  ticks(out) {
    if (this.empty) return 0;
    const lo = this.value(this.height), hi = this.value(0), want = Math.max(2, this.height / EVERY);
    let count = 0;
    const decades = this.mode === "log" ? Math.log10(hi) - Math.log10(lo) : 0;
    if (decades > 0) {
      const perDecade = this.height / decades;
      let set = 0;
      while (set + 1 < DECADE.length && DECADE_GAP[set + 1] * perDecade >= EVERY) set++;
      const steps = DECADE[set], every10 = perDecade >= EVERY_POWER ? 1 : Math.ceil(EVERY_POWER / perDecade);
      for (let p = Math.ceil(Math.floor(Math.log10(lo)) / every10) * every10; p <= Math.ceil(Math.log10(hi)) && count < out.length; p += every10)
        for (let k = 0; k < steps.length; k++) {
          const v = steps[k] * 10 ** p;
          if (v >= lo && v <= hi && count < out.length) out[count++] = v;
        }
      let filled = 0;
      {
        const a = count ? out[count - 1] : lo, b = hi, fromA = count ? EVERY : EVERY / 2, fromB = EVERY / 2;
        const raw = (EVERY * b * Math.LN10) / perDecade, power = 10 ** Math.floor(Math.log10(raw)), unit = raw / power;
        const step = (unit <= 1 ? 1 : unit <= 2 ? 2 : unit <= 2.5 ? 2.5 : unit <= 5 ? 5 : 10) * power;
        // Bounded by tries: a range too narrow for a step to move would never end.
        for (let k = 0, v = Math.ceil(a / step) * step; k < out.length && v < b && count + filled < out.length; k++, v += step)
          if (Math.log10(v / a) * perDecade >= fromA && Math.log10(b / v) * perDecade >= fromB) out[count + filled++] = v;
      }
      count += filled;
      if (decades >= 1) return count;
    }
    const raw = this.mode === "log" ? (EVERY * hi * Math.LN10 * decades) / this.height : (hi - lo) / want, power = 10 ** Math.floor(Math.log10(raw)), unit = raw / power;
    const step = (unit <= 1 ? 1 : unit <= 2 ? 2 : unit <= 2.5 ? 2.5 : unit <= 5 ? 5 : 10) * power, round = ROUND.length < out.length ? (ROUND = new Float64Array(out.length)) : ROUND;
    let rounds = 0;
    for (let v = Math.ceil(lo / step) * step; v <= hi && rounds < out.length; v += step) round[rounds++] = Math.abs(v) < step * 1e-9 ? 0 : v;
    if (!rounds && out.length) {
      const one = 10 ** Math.floor(Math.log10(hi - lo || Math.abs(hi) || 1)), v = Math.ceil(lo / one) * one;
      if (v <= hi) round[rounds++] = v;
    }
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
