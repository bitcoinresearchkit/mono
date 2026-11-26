// @ts-check
// Pans move the view by whole CSS pixels (the rest kept), so a drawn picture can be moved.

/** Under it an index counts as whole (1190.0000000000002 put point 1190 in the next column). */
export const NOISE = 1e-9;
/** @param {number} x */
const snap = (x) => (Math.abs(x - Math.round(x)) < NOISE ? Math.round(x) : x);
/** @param {number} a @param {number} b */
export const alike = (a, b) => Math.abs(a - b) <= Math.abs(a) * NOISE;

/** @param {Float64Array} values @param {number} i */
export function firstFrom(values, i) {
  for (let k = Math.max(0, i); k < values.length; k++) if (values[k] === values[k]) return values[k];
  return NaN;
}

/**
 * -1: none in bounds.
 * @param {ArrayLike<number>} values @param {number} from @param {number} to
 */
export function lastIn(values, from, to) {
  const first = Math.max(0, Math.ceil(from));
  for (let i = Math.min(Math.floor(to), values.length - 1); i >= first; i--) if (values[i] === values[i]) return i;
  return -1;
}

/** @param {number} width */
export const rightBlank = (width) => Math.round(Math.min(48, Math.max(16, width * 0.06)));

export class Timeline {
  /** @param {{ minSpacing?: number, maxSpacing?: number, minDataShare?: number }} [options] */
  constructor({ minSpacing = 0, maxSpacing = 60, minDataShare = 1 / 6 } = {}) {
    /** @type {Float64Array} */
    this.times = new Float64Array(0);
    this.from = 0;
    this.to = 1;
    this.width = 1;
    this.reserve = 0;
    this.minSpacing = minSpacing;
    this.maxSpacing = maxSpacing;
    this.minDataShare = minDataShare;
    /** @type {number | undefined} */
    this.minEndShare = undefined;
    this.first = 0;
    this.rest = 0;
    /** A view a resize kept past `maxSpacing`: no zooming in past it. Infinity: none. */
    this.kept = Infinity;
  }

  // Only float noise is allowed past an edge: a value beyond it never becomes the resting readout.
  get firstVisible() {
    return Math.ceil(this.from - NOISE);
  }

  get lastVisible() {
    return Math.floor(this.to + NOISE);
  }

  get lastSlot() {
    return Math.max(this.length - 1, this.lastVisible);
  }

  /** NaN: none. @param {number} i */
  timeAt(i) {
    const times = this.times, n = times.length;
    if (i >= 0 && i < n) return times[i];
    if (i < n || n < 2) return NaN;
    return times[n - 1] + (i - n + 1) * (times[n - 1] - times[n - 2]);
  }

  get length() {
    return this.times.length;
  }

  get rightOffset() {
    return rightBlank(this.width) + this.reserve;
  }

  get dataWidth() {
    return this.width > this.rightOffset ? this.width - this.rightOffset : this.width;
  }

  get spacing() {
    return this.width / (this.to - this.from);
  }

  /** @param {number} i */
  x(i) {
    return ((i - this.from) * this.width) / (this.to - this.from);
  }

  /** @param {number} x */
  index(x) {
    return this.from + (x * (this.to - this.from)) / this.width;
  }

  /** @param {Float64Array} times */
  setTimes(times) {
    const length = this.length, end = length > 0 && this.x(length - 1) <= this.width;
    this.times = times;
    if (end) {
      const added = times.length - length;
      this.from += added;
      this.to += added;
    }
    this.limit();
  }

  /** @param {number} span */
  show(span) {
    const { from, to } = this.shown(span);
    this.setRange(from, to);
  }

  /** @param {number} span */
  shown(span) {
    // (Up to the zoom's bound, or as close as a resize kept it: see kept.)
    const spacing = Math.min(Math.max(this.dataWidth / Math.max(span - 1, 0), this.minSpacing), Math.max(this.maxSpacing, this.width / this.kept));
    const to = this.length - 1 + this.rightOffset / spacing;
    return { from: to - this.width / spacing, to };
  }

  /** @param {number} from @param {number} to */
  setRange(from, to) {
    this.from = from;
    this.to = to;
    this.rest = 0;
    this.limit();
  }

  /** @param {number} from @param {number} to */
  slide(from, to) {
    const span = to - from;
    if (this.length && Math.abs(span - (this.to - this.from)) <= span * 1e-9) this.pan((this.from - from) * this.spacing - this.rest);
    else this.setRange(from, to);
  }

  /**
   * @param {number} width
   */
  resize(width) {
    if (width === this.width) return;
    this.width = width;
    const span = this.to - this.from;
    if (this.length && span < width / this.maxSpacing) this.kept = span;
    this.limit();
  }

  /** False: held at an end. @param {number} px */
  pan(px) {
    const whole = Math.round(px + this.rest);
    this.rest += px - whole;
    if (!whole) return true;
    const by = whole / this.spacing;
    const was = this.from;
    this.from -= by;
    this.to -= by;
    this.limit();
    return this.from !== was;
  }

  /**
   * @param {number} x @param {number} factor
   */
  zoom(x, factor) {
    const at = this.index(x), next = this.within((this.to - this.from) * factor);
    this.from = at - (x / this.width) * next;
    this.to = this.from + next;
    this.limit();
  }

  /**
   * Ties break a millionth of a pixel early so a pan's strip and a whole draw round alike.
   * @param {number} x @param {number} dpr
   */
  snap(x, dpr) {
    return Math.round(x * dpr + 1e-6) / dpr;
  }

  /** @param {number} span */
  within(span) {
    const least = Math.min(this.width / this.maxSpacing, this.kept);
    const most = this.minSpacing > 0 ? this.width / this.minSpacing : (Math.max(this.length - this.first, 2) * 1.1 * this.width) / this.dataWidth;
    return Math.min(Math.max(span, least), Math.max(most, least));
  }

  /** @param {number} from @param {number} to */
  held(from, to) {
    const { last, start, end } = this.holds(to - from);
    return from > last - end ? last - end - from : to < start ? start - to : 0;
  }

  /** Shares are of the room its points have (the view less the right offset), so 1 holds all the data shown. @param {number} span */
  holds(span) {
    const last = Math.max(this.length - 1, 0), data = last - this.first, blank = (span * (this.width - this.dataWidth)) / this.width, room = span - blank;
    return { last, start: this.first + Math.min(room * this.minDataShare, data) + blank, end: Math.min(room * (this.minEndShare ?? this.minDataShare), data) };
  }

  /** @param {number} px */
  room(px) {
    const { last, start, end } = this.holds(this.to - this.from);
    return Math.max(0, (px > 0 ? this.to - start : last - end - this.from) * this.spacing);
  }

  limit() {
    let span = this.to - this.from;
    if (span >= this.width / this.maxSpacing) this.kept = Infinity;
    const fits = this.within(span);
    if (fits !== span) {
      const middle = (this.from + this.to) / 2;
      (span = fits), (this.from = middle - span / 2), (this.to = middle + span / 2);
    }
    const by = this.held(this.from, this.to);
    (this.from += by), (this.to += by);
    // Layout and pan arithmetic can leave an endpoint a fraction past its bar.
    const from = snap(this.from), to = snap(this.to);
    if (to > from) (this.from = from), (this.to = to);
  }
}
