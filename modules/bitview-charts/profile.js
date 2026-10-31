// @ts-check
// A profile: groups' amounts by value bins (a day's cost basis, each holder group's), drawn as bars from the plot's right
// edge, the groups stacked (the first at the edge), each bar as long as its bin's total against the largest in view
// (`width`, its share of the plot at most). The bins: the groups' rows (bounds `edges`, values on the pane's scale; each
// group's rows rising) gathered by a power of two, a bin `bin` CSS pixels tall at least (the same bins wherever the view
// is panned); only the rows in view read. Under `at` (a value: a close) in each group's color, over it in its `dim` one (in profit and not). On its
// pane's own layer over everything, drawn whole each frame (one day's bars: cheap), its picture never stale under a pan.
// Never fitted or hit.

/** Each bin's total a group, as a draw gathers them (made once, grown). */
let totals = new Float64Array(256);

export class Profile {
  /**
   * @param {{ edges: Float64Array, groups: { rows: Int32Array, values: Float64Array, color: string, dim: string }[], at?: number, width?: number, bin?: number }} options
   */
  constructor({ edges, groups, at = NaN, width = 0.25, bin = 6 }) {
    /** @type {Float64Array} */
    this.edges = edges;
    /** @type {{ rows: Int32Array, values: Float64Array, color: string, dim: string }[]} */
    this.groups = groups;
    this.at = at;
    this.width = width;
    this.bin = bin;
    /** Drawn over everything, on its pane's own layer. */
    this.over = true;
    /** Not read on its side's scale: it says nothing of it. */
    this.scaled = false;
    this.hidden = false;
    this.version = 0;
  }

  /** @param {{ edges?: Float64Array, groups?: { rows: Int32Array, values: Float64Array, color: string, dim: string }[], at?: number, width?: number, bin?: number }} change */
  set({ edges = this.edges, groups = this.groups, at = this.at, width = this.width, bin = this.bin }) {
    this.version++;
    (this.edges = edges), (this.groups = groups), (this.at = at), (this.width = width), (this.bin = bin);
  }

  /** Nothing to fit. */
  extent() {}

  /** Any base. */
  first() {
    return 1;
  }

  /** Never hit. */
  distance() {
    return Infinity;
  }

  /**
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr
   */
  draw(g, time, scale, x0, x1, dpr) {
    const edges = this.edges, groups = this.groups, log = scale.mode === "log", count = edges.length - 1;
    if (count < 1 || !groups.length) return;
    // (A row's foot's y (a bound a log scale can't show: below the plot); the rows rising as their ys fall.)
    const y = (/** @type {number} */ row) => (log && !(edges[row] > 0) ? Infinity : scale.y(edges[row]));
    // (The rows in view, found by halves (not a scan of all of them): the first whose top is over the plot's foot, the
    // last whose foot is under its top.)
    let lo = 0, hi = count - 1;
    for (let a = 0, b = count - 1; a <= b; ) {
      const m = (a + b) >> 1;
      if (y(m + 1) < scale.height) (lo = m), (b = m - 1);
      else a = m + 1;
    }
    for (let a = lo, b = count - 1; a <= b; ) {
      const m = (a + b) >> 1;
      if (y(m) > 0) (hi = m), (a = m + 1);
      else b = m - 1;
    }
    if (!(y(lo + 1) < scale.height && y(hi) > 0)) return;
    // (The rows a bin: a power of two making the shortest row in view `bin` pixels tall at least.)
    let shortest = Infinity;
    for (let r = lo; r <= hi; r++) {
      const tall = y(r) - y(r + 1);
      if (tall > 0 && tall < Infinity) shortest = Math.min(shortest, tall);
    }
    if (!(shortest < Infinity)) return;
    // (On a log scale, the rows from the first it can show: a bin's foot there at the lowest.)
    let shown = 0;
    while (log && shown < count && !(edges[shown] > 0)) shown++;
    const rows = 2 ** Math.max(0, Math.ceil(Math.log2(this.bin / shortest))), firstBin = Math.floor(lo / rows), bins = Math.floor(hi / rows) - firstBin + 1, n = groups.length, gap = 1 / dpr;
    const topOf = (/** @type {number} */ b) => y(Math.min(count, (b + 1) * rows)), footOf = (/** @type {number} */ b) => y(Math.max(shown, b * rows));
    // (A bin drawn: within the plot (not only touching it), taller than the hair between two.)
    const drawn = (/** @type {number} */ top, /** @type {number} */ bottom) => bottom > 0 && top < scale.height && bottom - top > gap;
    // (Each group's rows in those bins alone (its rows rising: the first found by halves), totalled a bin.)
    if (totals.length < bins * n) totals = new Float64Array(bins * n * 2);
    totals.fill(0, 0, bins * n);
    const from = Math.max(shown, firstBin * rows), to = (firstBin + bins) * rows;
    for (let k = 0; k < n; k++) {
      const { rows: at, values } = groups[k];
      let j = 0;
      for (let a = 0, b = at.length - 1; a <= b; ) {
        const m = (a + b) >> 1;
        if (at[m] < from) a = m + 1;
        else (j = m), (b = m - 1);
      }
      if (!(at.length && at[at.length - 1] >= from)) continue;
      for (; j < at.length && at[j] < to; j++) if (values[j] > 0) totals[(Math.floor(at[j] / rows) - firstBin) * n + k] += values[j];
    }
    // (The largest bin in view, all groups: the longest bar.)
    let most = 0;
    for (let b = 0; b < bins; b++) {
      if (!drawn(topOf(firstBin + b), footOf(firstBin + b))) continue;
      let sum = 0;
      for (let k = 0; k < n; k++) sum += totals[b * n + k];
      most = Math.max(most, sum);
    }
    if (!(most > 0)) return;
    // (Its line: none, all of it in color; one a log scale can't show (at or under 0), under every bin: all of it dim.)
    const right = time.width - 2, longest = time.width * this.width, line = this.at !== this.at ? -Infinity : log && !(this.at > 0) ? Infinity : scale.y(this.at);
    for (let b = 0; b < bins; b++) {
      const top = topOf(firstBin + b), bottom = footOf(firstBin + b);
      if (!drawn(top, bottom)) continue;
      // (Its groups from the edge leftwards; its part under the line (in profit) in each one's color, over it dim.)
      let x = right;
      for (let k = 0; k < n; k++) {
        const length = (totals[b * n + k] / most) * longest;
        if (!(length > 0)) continue;
        // (A hair between two bins, at each one's foot.)
        const group = groups[k], foot = bottom - gap, under = Math.max(top, Math.min(foot, line));
        if (under > top) (g.fillStyle = group.dim), g.fillRect(x - length, top, length, under - top);
        if (foot > under) (g.fillStyle = group.color), g.fillRect(x - length, under, length, foot - under);
        x -= length;
      }
    }
  }
}
