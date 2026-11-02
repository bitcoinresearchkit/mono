// @ts-check
// A profile: groups' amounts by value rows (a day's cost basis, each holder group's), drawn as bars from the plot's right
// edge, the groups stacked (the first at the edge), each bar as long as its bin's total against the longest in view
// (`width(plot)` pixels at most). Its rows the page's (integers, a row's floor on the pane's scale `edge(row)`, the row a
// value's in `cell(value)`; even on a log scale, as a cost basis is cut), gathered into bins `bin` pixels tall at least,
// on whole multiples of a row (on a linear scale a power of two of them, rows taller as values grow): the same bins
// wherever the view's panned, whatever groups are shown; a row that tall its own bin. Each bin's bar from its top to a
// pixel over its floor, that pixel the `backdrop` (none seen through but by its opacity); split at `at` (a value: a
// close), each group's `gain` under it, `loss` over it (in profit and not); one group `lit`, the others in their dim
// ones. A close not the latest's dashed across it (`closeLine`, its color). Drawn whole each frame, on device pixels; the
// page told before each draw (`viewed`, its groups' day then). Fitted as the page says (`fit`); a bar under a point
// found as drawn (`hit`).

/**
 * @typedef {{ id: string, rows: Int32Array, values: ArrayLike<number>, gain: string, loss: string, dimGain: string, dimLoss: string }} ProfileGroup
 * @typedef {{ edges: number[], starts: number[], sums: Float64Array, count: number, most: number, width: number, right: number, ids: string[] }} ProfileBars
 */

export class Profile {
  /**
   * @param {{
   *   groups?: ProfileGroup[], edge: (row: number) => number, cell: (value: number) => number, at?: number, lit?: string,
   *   width?: (plot: number) => number, bin?: number, backdrop?: string, closeLine?: string | null,
   *   viewed?: ((time: import("./timeline.js").Timeline) => void) | null,
   *   fit?: ((a: number, b: number, positive: boolean, out: Float64Array) => void) | null,
   * }} options
   */
  constructor({ groups = [], edge, cell, at = NaN, lit = "", width = (plot) => Math.round(Math.min(200, Math.max(72, plot * 0.2))), bin = 3, backdrop = "transparent", closeLine = null, viewed = null, fit = null }) {
    /** @type {ProfileGroup[]} */
    this.groups = groups;
    (this.edge = edge), (this.cell = cell);
    (this.at = at), (this.lit = lit), (this.width = width), (this.bin = bin), (this.backdrop = backdrop), (this.closeLine = closeLine);
    this.onViewed = viewed;
    /** How it widens its scale's range, the page's (see the pane's Series); none, not fitted. */
    this.fit = fit;
    /** Its bars as last drawn (for a point on them: see hit); none drawn, null. @type {ProfileBars | null} */
    this.bars = null;
    /** Drawn whole each frame: its bars are the view's. */
    this.fixed = true;
    this.scaled = Boolean(fit);
    this.hidden = false;
    this.faint = false;
    this.version = 0;
  }

  /** @param {{ groups?: ProfileGroup[], at?: number, lit?: string, closeLine?: string | null, backdrop?: string }} change */
  set({ groups = this.groups, at = this.at, lit = this.lit, closeLine = this.closeLine, backdrop = this.backdrop }) {
    this.version++;
    (this.groups = groups), (this.at = at), (this.lit = lit), (this.closeLine = closeLine), (this.backdrop = backdrop);
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
   * The bar under a point (CSS pixels: `x` into the plot, `y` in its pane), as last drawn: its group, its bin's rows
   * (`from` to `to`, that one not in it); none between them or left of them. (The first group's to the plot's edge.)
   * @param {number} x @param {number} y @returns {{ group: string, from: number, to: number } | null}
   */
  hit(x, y) {
    const bars = this.bars;
    if (!bars) return null;
    const { edges, starts, sums, count, most, width, right } = bars;
    let bin = 0;
    while (bin < edges.length - 1 && !(y <= edges[bin] && y > edges[bin + 1])) bin++;
    for (let c = 0, at = right; bin < edges.length - 1 && c < count; c++) {
      const length = (sums[bin * count + c] / most) * width, group = bars.ids[c];
      if (length && group && x >= at - length && (!c || x <= at)) return { group, from: starts[bin], to: starts[bin + 1] };
      at -= length;
    }
    return null;
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} _x0 @param {number} _x1 @param {number} dpr
   */
  draw(g, time, scale, _x0, _x1, dpr) {
    this.bars = null;
    const groups = this.groups, edge = this.edge, plot = time.width, height = scale.height, width = this.width(plot), right = plot - 2, even = scale.mode === "log";
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
    // (Each group's amount a bin; the groups', the longest bin's.)
    const bins = edges.length - 1, count = groups.length, sums = new Float64Array(bins * count), totals = new Float64Array(bins);
    groups.forEach((group, c) => {
      const rows = group.rows, values = group.values;
      for (let k = lowerBound(rows, lo); k < rows.length && rows[k] <= hi; k++) {
        const b = binAt[rows[k] - lo], amount = Math.abs(values[k]);
        sums[b * count + c] += amount;
        totals[b] += amount;
      }
    });
    let most = 0;
    for (let b = 0; b < bins; b++) most = Math.max(most, totals[b]);
    if (!(most > 0)) return;
    this.bars = { edges, starts, sums, count, most, width, right, ids: groups.map((group) => group.id) };
    g.setTransform(1, 0, 0, 1, 0, 0);
    // (Each bin's bar from its top to a pixel over its floor (one too thin for that, to its floor); that pixel the
    // backdrop's, as long as its bar.)
    const gap = Math.max(1, Math.round(dpr)), tops = new Int32Array(bins), bottoms = new Int32Array(bins);
    g.fillStyle = this.backdrop;
    for (let b = 0; b < bins; b++) {
      const top = (tops[b] = Math.round(edges[b + 1] * dpr)), floor = Math.round(edges[b] * dpr), bottom = (bottoms[b] = floor - gap > top ? floor - gap : floor);
      if (!totals[b] || bottom === floor) continue;
      const left = Math.round((right - (totals[b] / most) * width) * dpr);
      g.fillRect(left, bottom, Math.max(1, Math.round(right * dpr) - left), floor - bottom);
    }
    // (The groups stacked from the right edge, the first at it: each group's bars in two passes, its gain's under the
    // close, its loss's over it, a bin it crosses split at it; a group lit, the others dimmed.)
    const line = this.at === this.at ? scale.y(this.at) : -Infinity, split = Math.round(line * dpr), from = new Float64Array(bins).fill(right), lit = this.lit;
    groups.forEach((group, c) => {
      const dim = lit && group.id !== lit;
      for (const strong of [true, false]) {
        g.fillStyle = strong ? (dim ? group.dimGain : group.gain) : dim ? group.dimLoss : group.loss;
        for (let b = 0; b < bins; b++) {
          const length = (sums[b * count + c] / most) * width;
          if (!length) continue;
          const x = from[b], left = Math.round((x - length) * dpr), top = tops[b], bottom = bottoms[b];
          const y0 = strong ? Math.max(top, split) : top, y1 = strong ? bottom : Math.min(bottom, split);
          if (y1 > y0) g.fillRect(left, y0, Math.max(1, Math.round(x * dpr) - left), y1 - y0);
          if (!strong) from[b] = x - length;
        }
      }
    });
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    // (A close not the latest's: dashed across its bars.)
    if (this.closeLine && line === line && line > -Infinity) {
      const y = Math.round(line) + 0.5;
      g.strokeStyle = this.closeLine;
      g.lineWidth = 1;
      g.setLineDash([3, 3]);
      g.beginPath();
      g.moveTo(right - width, y);
      g.lineTo(right, y);
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
