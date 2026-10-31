// @ts-check
// The timeline every pane shares: one time a logical index (the chart's dates, in seconds), the view (a logical range,
// from `from` to `to`, fractions allowed), and where each index is across the plot. A pan moves the view by whole CSS
// pixels (the rest kept for the next one), so a picture drawn before can be moved instead of drawn again. Its data
// always holds part of the view (`minDataShare` of its width, or all of the data where it's less), a pan, a flick or a
// zoom held there; zoomed in to `maxSpacing` CSS pixels a point at most, out to all of the data (or `minSpacing`). Its
// newest point in view, a zoom keeps it where it is: the latest data stays in sight at the right.

/**
 * The float noise a view's moves leave in an index, under which it counts as whole: a pixel column's first point the
 * same in a pan's strip as in a whole draw (an index 1190.0000000000002 had put point 1190 in the next column).
 */
export const NOISE = 1e-9;

export class Timeline {
  /** @param {{ rightOffset?: number, minSpacing?: number, maxSpacing?: number, minDataShare?: number }} [options] */
  constructor({ rightOffset = 24, minSpacing = 0, maxSpacing = 60, minDataShare = 1 / 6 } = {}) {
    /** One time a logical index, in seconds, rising. @type {Float64Array} */
    this.times = new Float64Array(0);
    this.from = 0;
    this.to = 1;
    /** The plot's width in CSS pixels. */
    this.width = 1;
    /** CSS pixels left empty past the last point when the view shows the end. */
    this.rightOffset = rightOffset;
    this.minSpacing = minSpacing;
    this.maxSpacing = maxSpacing;
    this.minDataShare = minDataShare;
    /** A pan's part of a pixel, kept for the next. */
    this.rest = 0;
  }

  get length() {
    return this.times.length;
  }

  /** CSS pixels a point. */
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
    const end = this.length > 0 && this.x(this.length - 1) <= this.width;
    this.times = times;
    // (Its end in view before, in view still: a new last point comes into view.)
    if (end) this.show(this.to - this.from);
    else this.limit();
  }

  /** The last `span` points in view, the right offset past them. @param {number} span */
  show(span) {
    const spacing = Math.min(Math.max(this.width / span, this.minSpacing), this.maxSpacing);
    this.to = this.length - 1 + this.rightOffset / spacing;
    this.from = this.to - this.width / spacing;
    this.rest = 0;
    this.limit();
  }

  /** @param {number} from @param {number} to */
  setRange(from, to) {
    this.from = from;
    this.to = to;
    this.rest = 0;
    this.limit();
  }

  /** The plot's width changed: the view's right end and its spacing kept. @param {number} width */
  resize(width) {
    if (width === this.width) return;
    const spacing = this.spacing;
    this.width = width;
    this.from = this.to - width / spacing;
    this.limit();
  }

  /**
   * Moved by `px` CSS pixels (to the right: earlier points come in), by whole pixels, the rest kept. Whether it could
   * move (false: held at an end).
   * @param {number} px
   */
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
   * Zoomed about `x` (CSS pixels across the plot) by `factor` (over 1: out); about the newest point instead, where it's
   * in view.
   * @param {number} x @param {number} factor
   */
  zoom(x, factor) {
    const last = this.length - 1, newest = last >= 0 && this.x(last) <= this.width;
    if (newest) x = this.x(last);
    const at = this.index(x), next = this.within((this.to - this.from) * factor);
    this.from = at - (x / this.width) * next;
    this.to = this.from + next;
    // (A pan's part of a pixel kept: a pinch pans as it zooms.)
    this.limit();
  }

  /**
   * `x` (CSS pixels) on a device pixel at `dpr`, the same whatever float noise the view's moves leave in it: a tie (a
   * point's x on a half pixel, as a spacing's halves put it) broken a millionth of a pixel early, so a pan's strip and a
   * whole draw round it alike (else a candle there drew a pixel off in a moved picture).
   * @param {number} x @param {number} dpr
   */
  snap(x, dpr) {
    return Math.round(x * dpr + 1e-6) / dpr;
  }

  /**
   * How far `x` (CSS pixels in the plot) is past point `i`'s column (half the spacing each side of it, a pixel at
   * least): 0 within it. A hit's measure across, where the point under a pointer is clamped to the data's ends.
   * @param {number} i @param {number} x
   */
  beyond(i, x) {
    return Math.max(0, Math.abs(x - this.x(i)) - Math.max(this.spacing / 2, 1));
  }

  /** A span (points) within the zoom's bounds: `maxSpacing` a point at most; at least all of the data, or `minSpacing`. @param {number} span */
  within(span) {
    const least = this.width / this.maxSpacing, most = this.minSpacing > 0 ? this.width / this.minSpacing : Math.max(this.length, 2) * 1.1 + this.rightOffset;
    return Math.min(Math.max(span, least), Math.max(most, least));
  }

  /** How far it can pan (CSS pixels) to `px`'s side (over 0: earlier points coming in) before an end holds it. @param {number} px */
  room(px) {
    const span = this.to - this.from, last = Math.max(this.length - 1, 0), hold = Math.min(span * this.minDataShare, last);
    return Math.max(0, (px > 0 ? this.to - hold : last - hold - this.from) * this.spacing);
  }

  /** Its spacing within bounds, and its data holding its share of the view (all of it where there's less). */
  limit() {
    let span = this.to - this.from;
    const fits = this.within(span);
    if (fits !== span) {
      const middle = (this.from + this.to) / 2;
      (span = fits), (this.from = middle - span / 2), (this.to = middle + span / 2);
    }
    const last = Math.max(this.length - 1, 0), hold = Math.min(span * this.minDataShare, last);
    // (Its last point at least `hold` points into the view; its first at most `hold` from the view's end.)
    if (this.from > last - hold) (this.from = last - hold), (this.to = this.from + span);
    if (this.to < hold) (this.to = hold), (this.from = this.to - span);
  }
}
