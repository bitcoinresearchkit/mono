// @ts-check
// A profile: groups' amounts by value bins (a day's cost basis, each holder group's), drawn as bars from the plot's right
// edge, the groups stacked (the first at the edge), each bar as long as its bin's total against the largest in view
// (`width`, its share of the plot at most). The bins: the groups' rows (bounds `edges`, values on the pane's scale)
// gathered `rows` at a time, a power of two making a bin `bin` CSS pixels tall at least (the same bins wherever the view
// is panned). Under `at` (a value: a close) in each group's color, over it in its `dim` one (in profit and not). On its
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
    // (A row's y; the rows a bin, a power of two making the shortest row in view `bin` pixels tall at least.)
    const y = (/** @type {number} */ row) => (log && !(edges[row] > 0) ? NaN : scale.y(edges[row]));
    let shortest = Infinity;
    for (let r = 0; r < count; r++) {
      const top = y(r + 1), bottom = y(r);
      if (bottom >= 0 && top <= scale.height && bottom - top > 0) shortest = Math.min(shortest, bottom - top);
    }
    if (!(shortest < Infinity)) return;
    const rows = 2 ** Math.max(0, Math.ceil(Math.log2(this.bin / shortest))), bins = Math.ceil(count / rows), n = groups.length;
    if (totals.length < bins * n) totals = new Float64Array(bins * n * 2);
    totals.fill(0, 0, bins * n);
    for (let k = 0; k < n; k++) {
      const group = groups[k];
      for (let j = 0; j < group.rows.length; j++) {
        const v = group.values[j];
        if (v > 0) totals[Math.floor(group.rows[j] / rows) * n + k] += v;
      }
    }
    // (The largest bin in view, all groups: the longest bar.)
    let most = 0;
    for (let b = 0; b < bins; b++) {
      const top = y(Math.min(count, (b + 1) * rows)), bottom = y(b * rows);
      if (!(bottom >= 0 && top <= scale.height)) continue;
      let sum = 0;
      for (let k = 0; k < n; k++) sum += totals[b * n + k];
      most = Math.max(most, sum);
    }
    if (!(most > 0)) return;
    const right = time.width - 2, longest = time.width * this.width, line = this.at === this.at ? scale.y(this.at) : -Infinity, gap = 1 / dpr;
    for (let b = 0; b < bins; b++) {
      const top = y(Math.min(count, (b + 1) * rows)), bottom = y(b * rows);
      if (!(bottom >= 0 && top <= scale.height) || !(bottom - top > gap)) continue;
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
