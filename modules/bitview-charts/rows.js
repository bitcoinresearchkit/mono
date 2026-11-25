// @ts-check
// Rows: a set's members one under another, time across (a heatmap of members: age bands, kinds). Each row the room its
// scale keeps for values shared evenly (the first at the top; over it, its legend's), each point a cell colored by the
// page (`paint(value, row, pixels, at)`:
// into its red, green, blue, opacity; none, left clear), so a row draws nothing before its data starts. Painted once
// into an image (a pixel a point, a row three pixels alike so a row's smoothing never reads its neighbour's), stretched
// onto the plot a row a draw (one pass of the pane's cells), smoothed only where its points are squeezed. `below`: a
// value under the page's level, its region's edges outlined in `outline` (see #outline). Its rows' names on its side's
// axis, each in its row's color (`axisTags`). Read on no scale (its own, unseen): its side's scale stays the page's.
// Hit anywhere on a row with a value there (`rowAt(y)`: which one).
import { PILL } from "./tags.js";

/**
 * A row: its values on the timeline, its name and color (its axis tag), and the page's `id` for it (what a hit on it is).
 * @typedef {{ values: ArrayLike<number>, name: string, color: string, id: string }} Row
 */

export class Rows {
  /**
   * @param {{
   *   paint: (value: number, row: number, pixels: Uint8ClampedArray, at: number) => void,
   *   below?: ((value: number) => boolean) | null, outline?: string, side?: "right" | "left",
   * }} options
   */
  constructor({ paint, below = null, outline = "", side = "right" }) {
    this.paint = paint;
    this.below = below;
    this.outline = outline;
    /** Its names' axis (0 the right, 1 the left). */
    this.side = side === "left" ? 1 : 0;
    /** @type {Row[]} */
    this.rows = [];
    this.hidden = false;
    this.faint = false;
    this.scaled = false;
    this.version = 0;
    /** Its cells as painted (`painted`: as it paints now, else painted again at the next draw). */
    this.image = document.createElement("canvas");
    this.painted = false;
    /** Each row's points under the level (1), as `below` reads them. @type {Uint8Array[]} */
    this.under = [];
  }

  /** Its rows (each on the timeline, the first at the top), painted anew. @param {Row[]} rows */
  set(rows) {
    this.rows = rows;
    this.changed();
  }

  /** What it paints changed (its colors, its level): painted anew at the next draw. */
  changed() {
    this.painted = false;
    this.version++;
  }

  /** The row at `y` (CSS pixels from its pane's top), -1 none. @param {number} y @param {import("./scale.js").Scale} scale */
  rowAt(y, scale) {
    const [top, h] = this.#band(scale), k = Math.floor((y - top) / h);
    return k >= 0 && k < this.rows.length ? k : -1;
  }

  /** Where its rows lie on its scale's height (CSS pixels): their top, and each row's height. @param {import("./scale.js").Scale} scale */
  #band(scale) {
    const top = scale.top * scale.height;
    return [top, Math.max(0, scale.height * (1 - scale.top - scale.bottom)) / Math.max(1, this.rows.length)];
  }

  /** Nothing to fit. */
  extent() {}

  /** Any base. */
  first() {
    return 1;
  }

  /** Hit on a row with a value at the point. @param {number} i @param {number} y @param {import("./scale.js").Scale} scale */
  distance(i, y, scale) {
    const k = this.rowAt(y, scale), at = Math.round(i);
    return k >= 0 && Number.isFinite(this.rows[k].values[at]) ? 0 : Infinity;
  }

  /**
   * Its rows' names on its side's axis, at their middles, each in its row's color: every row's where each is as tall
   * as a name (its pill and a gap), else as many as fit a pill apart, evenly spread, the first and the last always.
   * @param {number} side @param {(y: number, text: string, color: string) => void} add @param {import("./scale.js").Scale} scale
   */
  axisTags(side, add, scale) {
    const n = this.rows.length;
    if (side !== this.side || !n) return;
    // (Named every `every` rows at least, a pill and its gap apart: as many as that leaves, spread from the first to the last.)
    const [top, h] = this.#band(scale), every = Math.ceil((PILL + 2) / Math.max(h, 1e-9)), fit = n === 1 ? 1 : Math.min(n, Math.floor((n - 1) / every) + 1);
    for (let j = 0; j < fit; j++) {
      const k = fit === n ? j : fit === 1 ? 0 : Math.round((j * (n - 1)) / (fit - 1));
      add(top + (k + 0.5) * h, this.rows[k].name, this.rows[k].color);
    }
  }

  /** Its cells painted, a pixel a point, each row three pixels alike; and each row's points under the level. */
  #paintCells() {
    const rows = this.rows, n = rows.reduce((most, row) => Math.max(most, row.values.length), 0), image = this.image;
    (image.width = Math.max(1, n)), (image.height = Math.max(1, 3 * rows.length));
    const context = /** @type {CanvasRenderingContext2D} */ (image.getContext("2d")), cells = context.createImageData(image.width, image.height), pixels = cells.data, line = 4 * image.width;
    this.under = rows.map(() => new Uint8Array(n));
    rows.forEach((row, k) => {
      const values = row.values, under = this.under[k], top = 3 * k * line;
      for (let i = 0; i < values.length; i++) {
        const value = values[i];
        if (value !== value) continue;
        this.paint(value, k, pixels, top + 4 * i);
        if (this.below?.(value)) under[i] = 1;
      }
      pixels.copyWithin(top + line, top, top + line);
      pixels.copyWithin(top + 2 * line, top, top + line);
    });
    context.putImageData(cells, 0, 0);
    this.painted = true;
  }

  /**
   * Its cells from x `x0` to `x1` of the plot (a point past each edge), a row a draw, on device pixels; then the edges of
   * where they're under the level.
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {import("./scale.js").Scale} scale
   * @param {number} x0 @param {number} x1 @param {number} dpr
   */
  draw(g, time, scale, x0, x1, dpr) {
    const n = this.rows.length;
    if (!n) return;
    if (!this.painted) this.#paintCells();
    const image = this.image, last = image.width - 1;
    const a = Math.max(0, Math.floor(time.index(x0) - 1)), b = Math.min(last, Math.ceil(time.index(x1) + 1));
    if (a > b) return;
    const [band, size] = this.#band(scale), half = time.spacing / 2, left = (time.x(a) - half) * dpr, right = (time.x(b) + half) * dpr, top = band * dpr, h = size * dpr;
    const smooth = g.imageSmoothingEnabled, quality = g.imageSmoothingQuality;
    g.setTransform(1, 0, 0, 1, 0, 0);
    // (Squeezed, its points averaged into each pixel; else each a block of its own color, edges sharp.)
    g.imageSmoothingEnabled = time.spacing * dpr < 1;
    g.imageSmoothingQuality = "medium";
    for (let k = 0; k < n; k++) {
      const y = Math.round(top + k * h), foot = Math.round(top + (k + 1) * h);
      g.drawImage(image, a, 3 * k + 1, b - a + 1, 1, left, y, right - left, foot - y);
    }
    (g.imageSmoothingEnabled = smooth), (g.imageSmoothingQuality = quality);
    if (this.below && this.outline) this.#outline(g, time, a, b, top, h, dpr);
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
  }

  /**
   * The edges of the regions where its rows are under the level, from point `a` to `b`, in one stroke: down between two
   * blocks of a row, one under, the other not; across between two rows' blocks the same way (the top's and the foot's
   * against nothing). A row read by blocks (a power of two of points, a device pixel wide at least, each under where
   * most of its points with a value are: the same blocks at any pan), then its regions as the eye reads them: a gap in
   * one narrower than OUTLINE_RUN filled, then one narrower than it dropped (a closing, then an opening), so a day-to-day
   * flicker is one region or none, never a comb of edges; zoomed in until a point is that wide, every point its own.
   * Read from OUTLINE_RUN's width twice past each edge, so a region crossing one is judged whole, as panning shows it.
   * @param {CanvasRenderingContext2D} g @param {import("./timeline.js").Timeline} time @param {number} a @param {number} b
   * @param {number} top @param {number} h @param {number} dpr
   */
  #outline(g, time, a, b, top, h, dpr) {
    const n = this.rows.length, under = this.under, last = this.image.width - 1;
    let step = 1;
    while (step * time.spacing * dpr < 1 && step < 1 << 14) step *= 2;
    const run = Math.ceil(OUTLINE_RUN / (step * time.spacing)), from = Math.max(0, (Math.floor(a / step) - 2 * run) * step);
    const blocks = Math.ceil((Math.min(last, b + 2 * run * step) + 1 - from) / step);
    if (marks.length < blocks * (n + 2)) marks = new Uint8Array(blocks * (n + 2) * 2);
    // (A row's blocks, 1 where most of its points with a value are under; rows -1 and n, nothing.)
    marks.fill(0, 0, blocks * (n + 2));
    for (let k = 0; k < n; k++) {
      const values = this.rows[k].values, row = under[k], at = (k + 1) * blocks;
      for (let j = 0; j < blocks; j++) {
        let count = 0, seen = 0;
        for (let i = from + j * step, end = Math.min(i + step, row.length); i < end; i++) if (values[i] === values[i]) (seen++, (count += row[i]));
        marks[at + j] = seen && 2 * count > seen ? 1 : 0;
      }
      if (run > 1) merge(at, blocks, 0, run), merge(at, blocks, 1, run);
    }
    // (A line on whole device pixels: an odd width centered on one, an even one between two.)
    const width = Math.max(1, Math.round(dpr)), off = (width % 2) / 2;
    const xAt = (/** @type {number} */ j) => Math.round((time.x(from + j * step) - time.spacing / 2) * dpr), yAt = (/** @type {number} */ k) => Math.round(top + k * h);
    g.beginPath();
    for (let k = 0; k <= n; k++) {
      const y = yAt(k), over = k * blocks, at = (k + 1) * blocks;
      // (Down, at a block's left edge, in row k.)
      if (k < n) for (let j = 0; j <= blocks; j++) if ((j ? marks[at + j - 1] : 0) !== (j < blocks ? marks[at + j] : 0)) g.moveTo(xAt(j) + off, y), g.lineTo(xAt(j) + off, yAt(k + 1));
      // (Across, along row k's top, each run of blocks that differ from the row over it as one.)
      for (let j = 0; j < blocks; j++) {
        if (marks[over + j] === marks[at + j]) continue;
        let end = j;
        while (end + 1 < blocks && marks[over + end + 1] !== marks[at + end + 1]) end++;
        g.moveTo(xAt(j), y + off), g.lineTo(xAt(end + 1), y + off);
        j = end;
      }
    }
    g.lineWidth = width;
    g.strokeStyle = this.outline;
    g.stroke();
  }
}

/** The narrowest region (or gap) an outline draws, in CSS pixels: narrower, its two edges read as one mark, a comb of them as noise (at 2, the young bands at Max still combed; at 4, regions). */
const OUTLINE_RUN = 4;

/**
 * A row's marks (`count` of them from `at`) with each run of `value` shorter than `run` between two of the other
 * turned to the other (one at an end, against nothing, kept: it may go on past what's read).
 * @param {number} at @param {number} count @param {number} value @param {number} run
 */
function merge(at, count, value, run) {
  for (let j = 0; j < count; ) {
    if (marks[at + j] !== value) {
      j++;
      continue;
    }
    let end = j;
    while (end < count && marks[at + end] === value) end++;
    if (j > 0 && end < count && end - j < run) marks.fill(1 - value, at + j, at + end);
    j = end;
  }
}

/** Each row's blocks under the level, as an outline's found (made once, grown). */
let marks = new Uint8Array(1024);
