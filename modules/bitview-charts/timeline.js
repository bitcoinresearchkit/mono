// @ts-check
// The timeline every pane shares: one time a logical index (the chart's dates, in seconds), the view (a logical range,
// from `from` to `to`, fractions allowed), and where each index is across the plot. A pan moves the view by whole CSS
// pixels (the rest kept for the next one), so a picture drawn before can be moved instead of drawn again. Its data
// always holds part of the view (`minDataShare` of its width, or all of the data where it's less), a pan, a flick or a
// zoom held there; zoomed in to `maxSpacing` CSS pixels a point at most, out to all of the data (or `minSpacing`). A
// zoom keeps the point under the pointer where it is (what's looked at stays put).

/**
 * The float noise a view's moves leave in an index, under which it counts as whole: a pixel column's first point the
 * same in a pan's strip as in a whole draw (an index 1190.0000000000002 had put point 1190 in the next column).
 */
export const NOISE = 1e-9;

/** A column's first value at or after index `i` (a relative scale's base: a series' first in view), or NaN: none. @param {Float64Array} values @param {number} i */
export function firstFrom(values, i) {
  for (let k = Math.max(0, i); k < values.length; k++) if (values[k] === values[k]) return values[k];
  return NaN;
}

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

  /**
   * The plot's width changed: the view's right end and its spacing kept; or (`moved`: its left edge moved that many CSS
   * pixels on the screen) every point kept where it is on the screen, its spacing too, unless that'd take its latest
   * point out of view (one in view stays: the view follows it as more come, see setTimes).
   * @param {number} width @param {number} [moved]
   */
  resize(width, moved = NaN) {
    if (width === this.width && !moved) return;
    const spacing = this.spacing, to = this.to, last = this.length - 1, following = last >= 0 && this.x(last) <= this.width;
    this.width = width;
    if (moved === moved) {
      (this.from += moved / spacing), (this.to = this.from + width / spacing), this.limit();
      if (!(following && this.x(last) > width)) return;
    }
    (this.to = to), (this.from = to - width / spacing), this.limit();
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
   * Zoomed about `x` (CSS pixels across the plot) by `factor` (over 1: out): the point there stays there.
   * @param {number} x @param {number} factor
   */
  zoom(x, factor) {
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

  /** A span (points) within the zoom's bounds: `maxSpacing` a point at most; at least all of the data, or `minSpacing`. @param {number} span */
  within(span) {
    const least = this.width / this.maxSpacing, most = this.minSpacing > 0 ? this.width / this.minSpacing : Math.max(this.length, 2) * 1.1 + this.rightOffset;
    return Math.min(Math.max(span, least), Math.max(most, least));
  }

  /**
   * How far (points) a view from `from` to `to` moves to hold its data's share (see limit): back toward the data on the
   * side it's past; 0, held already. A page's glide aimed with it ends where the chart holds it.
   * @param {number} from @param {number} to
   */
  held(from, to) {
    const span = to - from, last = Math.max(this.length - 1, 0), hold = Math.min(span * this.minDataShare, last);
    return from > last - hold ? last - hold - from : to < hold ? hold - to : 0;
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
    // (Its last point at least its share of the view into it; its first at most that from the view's end.)
    const by = this.held(this.from, this.to);
    (this.from += by), (this.to += by);
  }
}
