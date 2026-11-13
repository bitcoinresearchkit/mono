// @ts-check
// A profile: groups' amounts by value rows (a day's cost basis, each holder group's), drawn as bars from the plot's right
// edge, the groups stacked (the first at the edge), each bar as long as its bin's total against the longest in view
// (`columnWidth(plot)` pixels at most). Its rows the page's (integers, a row's floor on the pane's scale `edge(row)`, the row a
// value's in `cell(value)`; even on a log scale, as a cost basis is cut), gathered into bins `bin` pixels tall at least,
// on whole multiples of a row (on a linear scale a power of two of them, rows taller as values grow): the same bins
// wherever the view's panned, whatever groups are shown; a row that tall its own bin. Each bin's bar from its top to a
// pixel over its floor, that pixel the `backdrop` (none seen through but by its opacity); split at `at` (a value: a
// close), each group's `gain` under it, `loss` over it (in profit and not); one group `lit`, the others in their dim
// ones. A close not the latest's dashed across it (`closeLine`, its color). `cumulative`: in place of the bars, the
// amounts summed from the close outward (upward over it, downward under it), each group's stacked on the one before as
// its bars are, filled in its colors to the edge, as long as the longer side's whole; read at a height (`sumsAt`).
// Held at the plot's edge, drawn whole when its view or data changes; the page told before each draw (`viewed`, its groups' day then). Fitted as
// the page says (`fit`); its column, what a bin holds at a height and a bar under a point found as drawn (`covers`,
// `readAt`, `hit`).

/**
 * @typedef {{ id: string, rows: Int32Array, values: ArrayLike<number>, gain: string, loss: string, dimGain: string, dimLoss: string }} ProfileGroup
 * @typedef {{ edges: number[], starts: number[], sums: Float64Array, signed: Float64Array, count: number, most: number, width: number, right: number, dpr: number, ids: string[] }} ProfileBars
 */

/** A cumulative profile's curves' width (CSS pixels). */
const STROKE = 1.5;

/** A profile's column at its widest, by the plot's width (CSS pixels): a fifth of it, 72 to 200. @param {number} plot */
export const columnWidth = (plot) => Math.round(Math.min(200, Math.max(72, plot * 0.2)));

/**
 * A bar's right end as painted (device pixels: the column past its last), its left on `left` and its right end at `x`
 * (CSS pixels): a pixel past its left at least, a bar thinner than one over the edge of the one before it (nearer the
 * plot's edge, drawn first). The painter's and hit's both. @param {number} left @param {number} x @param {number} dpr
 */
const barEnd = (left, x, dpr) => Math.max(left + 1, Math.round(x * dpr));

export class Profile {
  /**
   * @param {{
   *   groups?: ProfileGroup[], edge: (row: number) => number, cell: (value: number) => number, at?: number, lit?: string,
   *   bin?: number, backdrop?: string, closeLine?: string | null, cumulative?: boolean,
   *   viewed?: ((time: import("./timeline.js").Timeline) => void) | null,
   *   fit?: ((a: number, b: number, positive: boolean, out: Float64Array) => void) | null,
   * }} options
   */
  constructor({ groups = [], edge, cell, at = NaN, lit = "", bin = 3, backdrop = "transparent", closeLine = null, cumulative = false, viewed = null, fit = null }) {
    /** @type {ProfileGroup[]} */
    this.groups = groups;
    (this.edge = edge), (this.cell = cell);
    (this.at = at), (this.lit = lit), (this.bin = bin), (this.backdrop = backdrop), (this.closeLine = closeLine), (this.cumulative = cumulative);
    this.onViewed = viewed;
    /** How it widens its scale's range, the page's (see the pane's Series); none, not fitted. */
    this.fit = fit;
    /** Its bars as last drawn (for a point on them: see hit); none drawn, or cumulative, null. @type {ProfileBars | null} */
    this.bars = null;
    /** Its column as last drawn, bars or cumulative (see covers): CSS pixels into the plot; none drawn, null. @type {{ left: number, right: number } | null} */
    this.column = null;
    /** Its bins and the close as last drawn, cumulative (see sumsAt): none, null. @type {{ edges: number[], sums: Float64Array, count: number, line: number } | null} */
    this.summed = null;
    /** Held at the plot's edge: a pan redraws it, an unchanged view keeps its picture and bins. */
    this.fixed = true;
    this.scaled = Boolean(fit);
    this.hidden = false;
    this.faint = false;
    this.version = 0;
  }

  /** @param {{ groups?: ProfileGroup[], at?: number, lit?: string, closeLine?: string | null, backdrop?: string, cumulative?: boolean }} change */
  set({ groups = this.groups, at = this.at, lit = this.lit, closeLine = this.closeLine, backdrop = this.backdrop, cumulative = this.cumulative }) {
    this.version++;
    (this.groups = groups), (this.at = at), (this.lit = lit), (this.closeLine = closeLine), (this.backdrop = backdrop), (this.cumulative = cumulative);
  }

  /**
   * Cumulative, as last drawn: each group's amount from the close to a height (CSS pixels in its pane; over the close,
   * upward; under it, downward), a bin it's in by the part of it on the way; none drawn, null.
   * @param {number} y @returns {number[] | null}
   */
  sumsAt(y) {
    const summed = this.summed;
    if (!summed) return null;
    const { edges, sums, count, line } = summed, out = new Array(count).fill(0);
    for (let b = 0; b < edges.length - 1; b++) {
      // (Its part between the close and the height, as a share of it.)
      const top = edges[b + 1], bottom = edges[b], from = y < line ? Math.max(top, y) : Math.max(top, line), to = y < line ? Math.min(bottom, line) : Math.min(bottom, y);
      if (!(to > from)) continue;
      const share = (to - from) / (bottom - top);
      for (let c = 0; c < count; c++) out[c] += sums[b * count + c] * share;
    }
    return out;
  }

  /** What it draws changed in place: drawn again. */
  changed() {
    this.version++;
  }

  /** Nothing of its own to fit to (the page's `fit`, where it has one). */
  extent() {}

  /** Any base. */
  first() {
    return 1;
  }

  /** Not hit as a series: a page finds its bars (see hit). */
  distance() {
    return Infinity;
  }

  /** Told before each draw. @param {import("./timeline.js").Timeline} time */
  viewed(time) {
    this.onViewed?.(time);
  }

  /**
   * Whether `x` (CSS pixels into the plot) is in its column, as last drawn: from its left edge, as painted, to the
   * plot's; hidden (not drawn), never. @param {number} x
   */
  covers(x) {
    const column = this.column;
    return !this.hidden && column !== null && x >= column.left && x <= column.right;
  }

  /**
   * What its bars hold at a height (CSS pixels in its pane), as last drawn: the bin there (its prices span it, the pixel
   * under its bar its own), its rows (`from` to `to`, that one not in it) and its sum, signed (a loss less, where its bar
   * is as long as its amounts unsigned): its groups', or `group`'s alone; an empty bin's, 0. No bin there, hidden, or
   * drawn cumulative (see sumsAt), null.
   * @param {number} y @param {string} [group] @returns {{ from: number, to: number, sum: number } | null}
   */
  readAt(y, group) {
    const bars = this.bars;
    if (!bars || this.hidden) return null;
    const bin = this.#binAt(y);
    if (bin < 0) return null;
    const { starts, signed, count, ids } = bars, at = bin * count;
    let sum = 0;
    for (let c = 0; c < count; c++) if (group === undefined || ids[c] === group) sum += signed[at + c];
    return { from: starts[bin], to: starts[bin + 1], sum };
  }

  /**
   * The bin at a height, as last painted (see readAt, #bars): its index; none, or drawn cumulative, -1.
   * @param {number} y
   */
  #binAt(y) {
    const bars = this.bars;
    if (!bars) return -1;
    // (By the device pixel's row it's in, as painted: a bin's rows from its top edge's to its floor's, that one not in
    // it; the edges falling as prices rise.)
    const { edges, dpr } = bars, row = Math.floor(y * dpr);
    let lo = 0, hi = edges.length - 1;
    if (!(row < Math.round(edges[lo] * dpr) && row >= Math.round(edges[hi] * dpr))) return -1;
    while (hi - lo > 1) {
      const mid = (lo + hi) >> 1;
      if (Math.round(edges[mid] * dpr) > row) lo = mid;
      else hi = mid;
    }
    return lo;
  }

  /**
   * The bar under a point (CSS pixels: `x` into the plot, `y` in its pane), as last drawn (its ends on the device's
   * pixels, as painted): its group, its bin's rows (see readAt); none between them, nor out of its column (see covers),
   * nor drawn cumulative.
   * @param {number} x @param {number} y @returns {{ group: string, from: number, to: number } | null}
   */
  hit(x, y) {
    const bars = this.bars;
    if (!bars || !this.covers(x)) return null;
    const bin = this.#binAt(y);
    if (bin < 0) return null;
    const { starts, sums, count, most, width, right, dpr, ids } = bars;
    // (By the device pixel's column it's in (at the plot's edge, its last): each bar's columns as painted, the last
    // group's over it there the one seen.)
    const column = Math.min(Math.floor(x * dpr), Math.ceil(right * dpr) - 1);
    let seen = -1;
    for (let c = 0, at = right; c < count; c++) {
      const length = (sums[bin * count + c] / most) * width;
      if (!length) continue;
      const left = Math.round((at - length) * dpr);
      if (column >= left && column < barEnd(left, at, dpr)) seen = c;
      at -= length;
    }
    return seen >= 0 && ids[seen] ? { group: ids[seen], from: starts[bin], to: starts[bin + 1] } : null;
  }

  /**
   * Its bars: each bin's from its top to a pixel over its floor, the groups stacked from the right edge, split at the
   * close (see the class), on device pixels.
   * @param {CanvasRenderingContext2D} g @param {number[]} edges @param {Float64Array} sums @param {Float64Array} totals @param {number} count
   * @param {number} most @param {number} line @param {number} right @param {number} width @param {number} dpr
   * @returns {number} the left edge it painted (CSS pixels: its leftmost device pixel's)
   */
  #bars(g, edges, sums, totals, count, most, line, right, width, dpr) {
    const bins = totals.length, groups = this.groups;
    g.setTransform(1, 0, 0, 1, 0, 0);
    // (Each bin's bar from its top to a pixel over its floor (one too thin for that, to its floor); that pixel the
    // backdrop's, as long as its bar.)
    const gap = Math.max(1, Math.round(dpr)), tops = new Int32Array(bins), bottoms = new Int32Array(bins);
    let leftmost = Infinity;
    g.fillStyle = this.backdrop;
    for (let b = 0; b < bins; b++) {
      const top = (tops[b] = Math.round(edges[b + 1] * dpr)), floor = Math.round(edges[b] * dpr), bottom = (bottoms[b] = floor - gap > top ? floor - gap : floor);
      if (!totals[b] || bottom === floor) continue;
      const left = Math.round((right - (totals[b] / most) * width) * dpr);
      g.fillRect(left, bottom, Math.max(1, Math.round(right * dpr) - left), floor - bottom);
      leftmost = Math.min(leftmost, left);
    }
    // (The groups stacked from the right edge, the first at it: each group's bars in two passes, its gain's under the
    // close, its loss's over it, a bin it crosses split at it; a group lit, the others dimmed.)
    const split = Math.round(line * dpr), from = new Float64Array(bins).fill(right), lit = this.lit;
    groups.forEach((group, c) => {
      const dim = lit && group.id !== lit;
      for (const strong of [true, false]) {
        g.fillStyle = strong ? (dim ? group.dimGain : group.gain) : dim ? group.dimLoss : group.loss;
        for (let b = 0; b < bins; b++) {
          const length = (sums[b * count + c] / most) * width;
          if (!length) continue;
          const x = from[b], left = Math.round((x - length) * dpr), top = tops[b], bottom = bottoms[b];
          const y0 = strong ? Math.max(top, split) : top, y1 = strong ? bottom : Math.min(bottom, split);
          if (y1 > y0) {
            g.fillRect(left, y0, barEnd(left, x, dpr) - left, y1 - y0);
            leftmost = Math.min(leftmost, left);
          }
          if (!strong) from[b] = x - length;
        }
      }
    });
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    return leftmost / dpr;
  }

  /**
   * Cumulative: each side's amounts summed from the close outward, bin by bin (the close's own shared by its parts each
   * side), the groups stacked; each group's area from its stack's curve under it to its own, in its colors (its gain's
   * under the close, its loss's over it), to the edge as long as the longer side's whole; each curve along its top, on
   * to the pane's edge (flat past its last row).
   * @param {CanvasRenderingContext2D} g @param {number[]} edges @param {Float64Array} sums @param {Float64Array} signed
   * @param {number} count @param {number} line @param {number} right @param {number} width @param {number} height
   * @returns {number} the left edge it painted (CSS pixels: the longer side's whole, its stroke's half past it)
   */
  #cumulative(g, edges, sums, signed, count, line, right, width, height) {
    this.summed = { edges, sums: signed, count, line };
    const bins = edges.length - 1, groups = this.groups, lit = this.lit;
    // (Each side's heights, from the close outward: a bin's far edge each, its sums by its part on that side.)
    for (const up of [true, false]) {
      /** @type {number[]} */
      const ys = [line];
      /** @type {Float64Array[]} each group's running sum, at each height */
      const runs = groups.map(() => new Float64Array(bins + 2));
      const order = up ? [...Array(bins).keys()] : [...Array(bins).keys()].reverse();
      for (const b of order) {
        const top = edges[b + 1], bottom = edges[b], from = up ? top : Math.max(top, line), to = up ? Math.min(bottom, line) : bottom;
        if (!(to > from)) continue;
        const share = (to - from) / (bottom - top), k = ys.length;
        ys.push(up ? from : to);
        for (let c = 0; c < count; c++) runs[c][k] = runs[c][k - 1] + sums[b * count + c] * share;
      }
      // (Past its last row, its whole to the pane's edge.)
      const last = ys.length - 1, edge = up ? Math.min(0, ys[last]) : Math.max(height, ys[last]);
      if (edge !== ys[last]) {
        ys.push(edge);
        for (let c = 0; c < count; c++) runs[c][last + 1] = runs[c][last];
      }
      this.#sides[up ? 0 : 1] = { ys, runs };
    }
    const [over, under] = this.#sides, whole = (/** @type {{ ys: number[], runs: Float64Array[] }} */ side) => side.runs.reduce((sum, run) => sum + run[side.ys.length - 1], 0);
    const most = Math.max(whole(over), whole(under)), x = (/** @type {number} */ v) => right - (v / most) * width;
    for (const [side, up] of /** @type {const} */ ([[over, true], [under, false]])) {
      const { ys, runs } = side, n = ys.length, stacked = new Float64Array(n);
      groups.forEach((group, c) => {
        const dim = lit && group.id !== lit, color = up ? (dim ? group.dimLoss : group.loss) : dim ? group.dimGain : group.gain;
        // (Its area: along its own curve out, back along the one under it.)
        g.beginPath();
        for (let k = 0; k < n; k++) g.lineTo(x(stacked[k] + runs[c][k]), ys[k]);
        for (let k = n - 1; k >= 0; k--) g.lineTo(x(stacked[k]), ys[k]);
        g.closePath();
        g.fillStyle = color;
        g.fill();
        g.beginPath();
        for (let k = 0; k < n; k++) g.lineTo(x(stacked[k] + runs[c][k]), ys[k]);
        g.strokeStyle = dim ? group.dimGain : group.gain;
        g.lineWidth = STROKE;
        g.lineJoin = "round";
        g.stroke();
        for (let k = 0; k < n; k++) stacked[k] += runs[c][k];
      });
    }
    return right - width - STROKE / 2;
  }

  /** Each side's heights and running sums, as the cumulative draw makes them (made once a draw). @type {{ ys: number[], runs: Float64Array[] }[]} */
  #sides = [{ ys: [], runs: [] }, { ys: [], runs: [] }];

  /**
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} _x0 @param {number} _x1 @param {number} dpr
   */
  draw(g, time, scale, _x0, _x1, dpr) {
    (this.bars = null), (this.summed = null), (this.column = null);
    const groups = this.groups, edge = this.edge, plot = time.width, height = scale.height, width = columnWidth(plot), even = scale.mode === "log";
    let low = Infinity, high = -Infinity;
    for (const group of groups) if (group.rows.length) (low = Math.min(low, group.rows[0])), (high = Math.max(high, group.rows[group.rows.length - 1]));
    if (!(low <= high)) return;
    // (The rows in view, a row from its floor's y up to its next's: the first whose top is over the pane's bottom, the
    // last whose floor is under its top.)
    const yOf = (/** @type {number} */ row) => scale.y(edge(row));
    let lo = low, hi = high + 1;
    while (lo < hi) {
      const mid = (lo + hi) >> 1;
      if (yOf(mid + 1) < height) hi = mid;
      else lo = mid + 1;
    }
    for (let last = high; hi < last; ) {
      const mid = (hi + last + 1) >> 1;
      if (yOf(mid) > 0) hi = mid;
      else last = mid - 1;
    }
    // (Cumulative, from the close's row wherever it is: the amounts between it and the view counted, the bins there
    // whole (none skipped across rows without coins); a view past every row, its sums whole.)
    const summing = this.cumulative && this.at === this.at && this.at > 0;
    if (summing) ((at) => ((lo = Math.min(lo, at)), (hi = Math.max(hi, at))))(this.cell(this.at));
    if (hi < lo) return;
    // (Rows into bins `bin` pixels tall at least, as many rows as it takes, on whole multiples of a row (a power of two
    // on a linear scale, its rows taller as values grow): the same bins wherever the view's panned and whichever groups
    // are shown; a row that tall, its own bin, as tall as it is.)
    const bin = this.bin, rowsFor = (/** @type {number} */ tall) => (tall >= bin || !(tall > 0) ? 1 : even ? Math.ceil(bin / tall) : 2 ** Math.min(20, Math.ceil(Math.log2(bin / tall))));
    // (On a log scale, rows alike: as tall as a decade's over its rows, one where the scale's own offset at its foot is
    // nothing (100 to 1,000): only a zoom changes it.)
    const uniform = even ? rowsFor((yOf(this.cell(100)) - yOf(this.cell(1000))) / (this.cell(1000) - this.cell(100))) : 0;
    const binOf = (/** @type {number} */ row) => ((rows) => Math.floor(row / rows) * rows)(uniform || rowsFor(yOf(row) - yOf(row + 1)));
    // (A run, the rows binned alike, a bin; one cut short, where a linear scale's count of rows changes, part of the bin
    // over it: the same bins wherever the view is. The bins at its edges whole.)
    const runStart = (/** @type {number} */ row) => { while (binOf(row - 1) === binOf(row)) row--; return row; };
    const runEnd = (/** @type {number} */ row) => { while (binOf(row + 1) === binOf(row)) row++; return row; };
    const short = (/** @type {number} */ from, /** @type {number} */ to) => !uniform && yOf(from) - yOf(to + 1) < bin - 0.01;
    lo = runStart(lo);
    for (let under = runStart(lo - 1); lo > low && short(under, lo - 1); under = runStart(lo - 1)) lo = under;
    hi = runEnd(hi);
    while (hi < high && short(runStart(hi), hi)) hi = runEnd(hi + 1);
    // (Each row's bin, from the lowest in view; each bin's edges, its floor's y and its top's.)
    const span = hi + 1 - lo, binAt = new Int32Array(span), edges = [yOf(lo)], starts = [lo];
    for (let k = 0, at = binOf(lo), from = lo; k < span; k++) {
      const row = lo + k, b = binOf(row);
      if (b !== at) {
        if (!short(from, row - 1)) edges.push(yOf(row)), starts.push(row);
        (at = b), (from = row);
      }
      binAt[k] = edges.length - 1;
    }
    edges.push(yOf(hi + 1)), starts.push(hi + 1);
    // (Each group's amount a bin, by its size; its signed sum too (a loss less): a bin's reading (see readAt), and what the
    // cumulative one sums outward.)
    const bins = edges.length - 1, count = groups.length, sums = new Float64Array(bins * count), totals = new Float64Array(bins), signed = new Float64Array(bins * count);
    groups.forEach((group, c) => {
      const rows = group.rows, values = group.values;
      for (let k = lowerBound(rows, lo); k < rows.length && rows[k] <= hi; k++) {
        const b = binAt[rows[k] - lo], amount = Math.abs(values[k]);
        sums[b * count + c] += amount;
        signed[b * count + c] += values[k];
        totals[b] += amount;
      }
    });
    let most = 0;
    for (let b = 0; b < bins; b++) most = Math.max(most, totals[b]);
    if (!(most > 0)) return;
    const line = this.at === this.at ? scale.y(this.at) : -Infinity, cumulative = summing && line === line && line > -Infinity;
    // (Cumulative: in the bars' column, read by height (see sumsAt), no bars to point at.)
    const painted = cumulative ? this.#cumulative(g, edges, sums, signed, count, line, plot, width, height) : this.#bars(g, edges, sums, totals, count, most, line, plot, width, dpr);
    if (!cumulative) this.bars = { edges, starts, sums, signed, count, most, width, right: plot, dpr, ids: groups.map((group) => group.id) };
    // (Its column: from the left edge it painted, in the plot, to the plot's.)
    const left = Math.max(0, painted);
    this.column = { left, right: plot };
    // (A close not the latest's: dashed across its bars.)
    if (this.closeLine && line === line && line > -Infinity) {
      const y = Math.round(line) + 0.5;
      g.strokeStyle = this.closeLine;
      g.lineWidth = 1;
      g.setLineDash([3, 3]);
      g.beginPath();
      g.moveTo(left, y);
      g.lineTo(plot, y);
      g.stroke();
      g.setLineDash([]);
    }
  }
}

/** The first index whose row is at least `row` (rows rising). @param {Int32Array} rows @param {number} row */
function lowerBound(rows, row) {
  let a = 0, b = rows.length;
  while (a < b) {
    const m = (a + b) >> 1;
    if (rows[m] < row) a = m + 1;
    else b = m;
  }
  return a;
}
