// @ts-check
// Bins sit on whole multiples of a row: the same wherever the view's panned and whatever groups show.

/**
 * @typedef {{ id: string, rows: Int32Array, values: ArrayLike<number>, gain: string, loss: string, dimGain: string, dimLoss: string }} ProfileGroup
 * @typedef {{ edges: number[], starts: number[], sums: Float64Array, signed: Float64Array, count: number, most: number, width: number, right: number, dpr: number, ids: string[] }} ProfileBars
 */

const STROKE = 1.5;

/** @param {number} plot */
export const columnWidth = (plot) => Math.round(Math.min(200, Math.max(72, plot * 0.2)));

/** @param {number} left @param {number} x @param {number} dpr */
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
    this.fit = fit;
    /** @type {ProfileBars | null} */
    this.bars = null;
    /** @type {{ left: number, right: number } | null} */
    this.column = null;
    /** @type {{ edges: number[], sums: Float64Array, count: number, line: number } | null} */
    this.summed = null;
    /** A pan redraws it; an unchanged view keeps its picture. */
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
   * null: none drawn.
   * @param {number} y @returns {number[] | null}
   */
  sumsAt(y) {
    const summed = this.summed;
    if (!summed) return null;
    const { edges, sums, count, line } = summed, out = new Array(count).fill(0);
    for (let b = 0; b < edges.length - 1; b++) {
      const top = edges[b + 1], bottom = edges[b], from = y < line ? Math.max(top, y) : Math.max(top, line), to = y < line ? Math.min(bottom, line) : Math.min(bottom, y);
      if (!(to > from)) continue;
      const share = (to - from) / (bottom - top);
      for (let c = 0; c < count; c++) out[c] += sums[b * count + c] * share;
    }
    return out;
  }

  changed() {
    this.version++;
  }

  extent() {}

  first() {
    return 1;
  }

  distance() {
    return Infinity;
  }

  /** @param {import("./timeline.js").Timeline} time */
  viewed(time) {
    this.onViewed?.(time);
  }

  /** @param {number} x */
  covers(x) {
    const column = this.column;
    return !this.hidden && column !== null && x >= column.left && x <= column.right;
  }

  /**
   * null: drawn cumulative.
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

  /** @param {number} y */
  #binAt(y) {
    const bars = this.bars;
    if (!bars) return -1;
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

  /** @param {number} x @param {number} y @returns {{ group: string, from: number, to: number } | null} */
  hit(x, y) {
    const bars = this.bars;
    if (!bars || !this.covers(x)) return null;
    const bin = this.#binAt(y);
    if (bin < 0) return null;
    const { starts, sums, count, most, width, right, dpr, ids } = bars;
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
   * @param {CanvasRenderingContext2D} g @param {number[]} edges @param {Float64Array} sums @param {Float64Array} totals @param {number} count
   * @param {number} most @param {number} line @param {number} right @param {number} width @param {number} dpr
   * @returns {number} the left edge it painted
   */
  #bars(g, edges, sums, totals, count, most, line, right, width, dpr) {
    const bins = totals.length, groups = this.groups;
    g.setTransform(1, 0, 0, 1, 0, 0);
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
   * @param {CanvasRenderingContext2D} g @param {number[]} edges @param {Float64Array} sums @param {Float64Array} signed
   * @param {number} count @param {number} line @param {number} right @param {number} width @param {number} height
   * @returns {number} the left edge it painted
   */
  #cumulative(g, edges, sums, signed, count, line, right, width, height) {
    this.summed = { edges, sums: signed, count, line };
    const bins = edges.length - 1, groups = this.groups, lit = this.lit;
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

  /** @type {{ ys: number[], runs: Float64Array[] }[]} */
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
    const summing = this.cumulative && this.at === this.at && this.at > 0;
    if (summing) ((at) => ((lo = Math.min(lo, at)), (hi = Math.max(hi, at))))(this.cell(this.at));
    if (hi < lo) return;
    const bin = this.bin, rowsFor = (/** @type {number} */ tall) => (tall >= bin || !(tall > 0) ? 1 : even ? Math.ceil(bin / tall) : 2 ** Math.min(20, Math.ceil(Math.log2(bin / tall))));
    const uniform = even ? rowsFor((yOf(this.cell(100)) - yOf(this.cell(1000))) / (this.cell(1000) - this.cell(100))) : 0;
    const binOf = (/** @type {number} */ row) => ((rows) => Math.floor(row / rows) * rows)(uniform || rowsFor(yOf(row) - yOf(row + 1)));
    // A run cut short by a linear scale's row count changing is part of the bin over it: same bins wherever the view is.
    const runStart = (/** @type {number} */ row) => { while (binOf(row - 1) === binOf(row)) row--; return row; };
    const runEnd = (/** @type {number} */ row) => { while (binOf(row + 1) === binOf(row)) row++; return row; };
    const short = (/** @type {number} */ from, /** @type {number} */ to) => !uniform && yOf(from) - yOf(to + 1) < bin - 0.01;
    lo = runStart(lo);
    for (let under = runStart(lo - 1); lo > low && short(under, lo - 1); under = runStart(lo - 1)) lo = under;
    hi = runEnd(hi);
    while (hi < high && short(runStart(hi), hi)) hi = runEnd(hi + 1);
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
    const painted = cumulative ? this.#cumulative(g, edges, sums, signed, count, line, plot, width, height) : this.#bars(g, edges, sums, totals, count, most, line, plot, width, dpr);
    if (!cumulative) this.bars = { edges, starts, sums, signed, count, most, width, right: plot, dpr, ids: groups.map((group) => group.id) };
    const left = Math.max(0, painted);
    this.column = { left, right: plot };
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

/** @param {Int32Array} rows @param {number} row */
function lowerBound(rows, row) {
  let a = 0, b = rows.length;
  while (a < b) {
    const m = (a + b) >> 1;
    if (rows[m] < row) a = m + 1;
    else b = m;
  }
  return a;
}
