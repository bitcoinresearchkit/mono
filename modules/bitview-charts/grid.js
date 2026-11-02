// @ts-check
// A grid: a heatmap's cells, its columns the page's (a day each, say: `columns`, each point's; -1, none there), its rows
// the page's too (integers, a row's floor on the pane's scale `edge(row)`, its top the next row's; even on a log scale,
// as a cost basis is cut). Zoomed out, a column a block of columns (a power of two of them, `width` CSS pixels at least;
// a block drawn as its last: the same blocks at any pan, as the page fetches them). A column's cells (`column(owner,
// step)`: its rows, rising, their values and a key; null: none, or not here yet) drawn as one strip, made once and
// stretched into place, never cell by cell (a view holds hundreds of thousands): on a log scale a pixel a row; on a
// linear one a pixel a step of values (a power of two near a CSS pixel's worth, `TALL` pixels at most), each pixel its
// strongest cell's (by its size, its sign aside), colored by the page (`paint`: into its red, green, blue, opacity; none,
// left clear), its look's key `look` (strips made again as it changes). Each column on device pixels, its strip
// smoothed only where it's squeezed (its rows' edges sharp where they're taller than a pixel). Before each draw the page
// is told the blocks in view (`viewed`): what it fetches and keeps. A backdrop, never hit; fitted as the page says
// (`extent`), else not.

/** A strip's pixels at most, on a linear scale. */
const TALL = 4096;
/** Strips kept at least (twice the columns in view where that's more): their canvases reused past it (a canvas a column
 * made anew each time would leave WebKit past its canvas memory, every canvas on the page blank). */
const KEPT = 768;

/**
 * @typedef {{ rows: Int32Array, values: ArrayLike<number>, key: string }} Column
 * @typedef {{ canvas: HTMLCanvasElement, bottom: number, top: number, height: number }} Strip
 */

export class Grid {
  /**
   * @param {{
   *   columns: Int32Array, column: (owner: number, step: number) => Column | null, edge: (row: number) => number,
   *   paint: (value: number, pixels: Uint8ClampedArray, at: number) => void, look?: string, width?: number,
   *   viewed?: ((blocks: { step: number, owners: number[] }) => void) | null,
   *   extent?: ((a: number, b: number, positive: boolean, out: Float64Array) => void) | null,
   * }} options
   */
  constructor({ columns, column, edge, paint, look = "", width = 2, viewed = null, extent = null }) {
    /** Each point's column (-1: none). */
    this.columns = columns;
    /** The last column (the newest): the blocks' last. */
    this.lastColumn = lastOf(columns);
    this.column = column;
    this.edge = edge;
    this.paint = paint;
    this.look = look;
    this.width = width;
    this.onViewed = viewed;
    this.fit = extent;
    /** The blocks' size as last found (a power of two of columns). */
    this.step = 1;
    /** Strips by column, look and step, the most recently drawn last. @type {Map<string, Strip>} */
    this.strips = new Map();
    /** Strips kept at most, as its last view needed. */
    this.kept = KEPT;
    /** Not read on its side's scale unless it says so (`extent`). */
    this.scaled = Boolean(extent);
    this.hidden = false;
    this.faint = false;
    this.version = 0;
  }

  /** Its columns anew (the timeline's, a live point grown): drawn again. @param {Int32Array} columns */
  setColumns(columns) {
    (this.columns = columns), (this.lastColumn = lastOf(columns));
    this.version++;
  }

  /** Its look's key: strips made again where it's another (its colors, its reference). @param {string} look */
  setLook(look) {
    if (look === this.look) return;
    this.look = look;
    this.version++;
  }

  /** What it draws changed (a column come, a coarser one replaced): drawn again, strips kept by their keys. */
  changed() {
    this.version++;
  }

  /** A column's strips let go (its cells anew under the same key: the newest day refreshed), drawn again. @param {string} key */
  forget(key) {
    for (const [other, strip] of this.strips) if (other.startsWith(`${key}|`)) this.strips.delete(other), spare.push(strip.canvas);
    this.version++;
  }

  /** @param {number} a @param {number} b @param {boolean} positive @param {Float64Array} out */
  extent(a, b, positive, out) {
    this.fit?.(a, b, positive, out);
  }

  /** Any base. */
  first() {
    return 1;
  }

  /** Never hit: a backdrop. */
  distance() {
    return Infinity;
  }

  /** The block a column's in: its last (the newest at most). @param {number} column @param {number} step */
  owner(column, step) {
    return Math.min(Math.ceil(column / step) * step, this.lastColumn);
  }

  /**
   * Before each draw: its blocks' size for the view (each `width` pixels at least) and the blocks in view (and one past
   * each edge), told the page.
   * @param {import("./timeline.js").Timeline} time
   */
  viewed(time) {
    const columns = this.columns, a = Math.max(0, Math.floor(time.from - 1)), b = Math.min(columns.length - 1, Math.ceil(time.to + 1));
    let first = -1, last = -1, count = 0;
    for (let i = a; i <= b; i++) if (columns[i] >= 0) (first < 0 && (first = columns[i])), (last = columns[i]), count++;
    let step = 1;
    if (count) {
      const each = (time.spacing * count) / Math.max(1, last - first + 1);
      while (step * each < this.width && step < 1 << 14) step *= 2;
    }
    // (Its blocks another size: drawn whole, none of its picture moved.)
    if (step !== this.step) (this.step = step), this.version++;
    if (!this.onViewed) return;
    const owners = [];
    for (let i = a; i <= b; i++) if (columns[i] >= 0) ((owner) => owner !== owners.at(-1) && owners.push(owner))(this.owner(columns[i], step));
    this.onViewed({ step, owners });
  }

  /**
   * Its columns from x `x0` to `x1` of the plot, each its block's strip stretched over its points (half a point's room
   * each side), on device pixels.
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr
   */
  draw(g, time, scale, x0, x1, dpr) {
    const columns = this.columns, step = this.step, log = scale.mode === "log";
    // (On a linear scale, its strips' step of values: a power of two near a CSS pixel's worth, the same while the zoom
    // stays within it.)
    const span = log ? 0 : Math.abs(scale.y(scale.lo) - scale.y(scale.hi)) / Math.abs(scale.hi - scale.lo);
    const by = span > 0 ? 2 ** Math.ceil(Math.log2(1 / span)) : 0;
    if (!log && !by) return;
    const a = Math.max(0, Math.floor(time.from - 1)), b = Math.min(columns.length - 1, Math.ceil(time.to + 1)), half = time.spacing / 2, height = scale.height;
    const smooth = g.imageSmoothingEnabled;
    g.setTransform(1, 0, 0, 1, 0, 0);
    let viewed = 0;
    const draw = (/** @type {number} */ owner, /** @type {number} */ left, /** @type {number} */ right) => {
      viewed++;
      if (right < x0 || left > x1) return;
      const strip = this.#strip(owner, step, log ? 0 : by);
      if (!strip) return;
      const top = scale.y(strip.top), bottom = scale.y(strip.bottom);
      if (!(bottom >= 0) || !(top <= height)) return;
      // (Its ends where they fall, not rounded: each column's rows span their own values, so rounded ends scaled each
      // column a hair apart, their rows' edges a pixel off from one to the next, a comb along them.)
      const x = Math.round(left * dpr), w = Math.max(1, Math.round(right * dpr) - x), y0 = top * dpr, y1 = bottom * dpr;
      g.imageSmoothingEnabled = y1 - y0 < strip.height;
      g.drawImage(strip.canvas, x, y0, w, Math.max(1, y1 - y0));
    };
    let owner = -1, left = 0, right = 0;
    for (let i = a; i <= b; i++) {
      if (columns[i] < 0) continue;
      const x = time.x(i), here = this.owner(columns[i], step);
      if (here !== owner) {
        if (owner >= 0) draw(owner, left, right);
        (owner = here), (left = x - half);
      }
      right = x + half;
    }
    if (owner >= 0) draw(owner, left, right);
    g.imageSmoothingEnabled = smooth;
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    this.kept = Math.max(KEPT, 2 * viewed);
  }

  /**
   * Block `owner`'s strip (`step` columns), a pixel a row (`by` 0: a log scale's) or a step of `by` values: made once,
   * kept (the most recently drawn last); none where the page has no cells there (yet).
   * @param {number} owner @param {number} step @param {number} by
   * @returns {Strip | null}
   */
  #strip(owner, step, by) {
    const column = this.column(owner, step);
    if (!column || !column.rows.length) return null;
    const key = `${column.key}|${this.look}|${by}`, had = this.strips.get(key);
    if (had) return this.strips.delete(key), this.strips.set(key, had), had;
    const { rows, values } = column, edge = this.edge, n = rows.length, lowest = edge(rows[0]), highest = edge(rows[n - 1] + 1);
    // (A step no finer than its height's most, TALL: zoomed far in on a linear scale, its pixels stretched, finer than
    // its rows anyway.)
    if (by) by = Math.max(by, 2 ** Math.ceil(Math.log2(highest / TALL)));
    const bottom = by ? Math.floor(lowest / by) * by : lowest, top = by ? Math.ceil(highest / by) * by : highest;
    const height = Math.max(1, by ? Math.round((top - bottom) / by) : rows[n - 1] - rows[0] + 1);
    if (strongest.length < height) strongest = new Float64Array(height * 2);
    strongest.fill(0, 0, height);
    for (let k = 0; k < n; k++) {
      const value = values[k];
      if (!value) continue;
      // (Its pixels: its own on a log scale; on a linear one, every step it reaches, a part of it too.)
      const from = by ? Math.max(0, Math.floor((top - edge(rows[k] + 1)) / by)) : rows[n - 1] - rows[k];
      const to = by ? Math.min(height - 1, Math.ceil((top - edge(rows[k])) / by) - 1) : from;
      for (let p = from; p <= to; p++) if (Math.abs(value) > Math.abs(strongest[p])) strongest[p] = value;
    }
    if (this.strips.size >= this.kept) for (const [other, strip] of this.strips) {
      this.strips.delete(other), spare.push(strip.canvas);
      break;
    }
    const canvas = spare.pop() ?? document.createElement("canvas");
    (canvas.width = 1), (canvas.height = height);
    const context = /** @type {CanvasRenderingContext2D} */ (canvas.getContext("2d")), image = context.createImageData(1, height);
    for (let p = 0; p < height; p++) if (strongest[p]) this.paint(strongest[p], image.data, p * 4);
    context.putImageData(image, 0, 0);
    const strip = { canvas, bottom, top, height };
    this.strips.set(key, strip);
    return strip;
  }
}

/** @param {Int32Array} columns */
const lastOf = (columns) => columns.reduce((most, column) => Math.max(most, column), -1);
/** Each pixel's strongest value, as a strip's made (made once, grown). */
let strongest = new Float64Array(256);
/** Canvases let go of, to be painted anew (shared by every grid). @type {HTMLCanvasElement[]} */
const spare = [];
