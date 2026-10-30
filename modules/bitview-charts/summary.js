// @ts-check
// A column of values summed up by blocks of 4, 8, 16 points and so on (each level's blocks twice the last's): each
// block's first and last value, its low and high and where they are, and whether it has a point a line can't go
// through (none; on a log scale, none above zero). Zoomed out, a pixel column of many points reads its few whole blocks
// that are clean, its ends point by point: a draw's work by its columns, not by its points.

/** The first level's blocks: 4 points (2^2). */
const FIRST = 2;

export class Summary {
  /** @param {Float64Array} values */
  constructor(values) {
    /** @type {{ first: Float64Array, last: Float64Array, lo: Float64Array, hi: Float64Array, loAt: Int32Array, hiAt: Int32Array, nan: Uint8Array, low: Uint8Array }[]} */
    this.levels = [];
    this.values = values;
    for (let size = 1 << FIRST; size <= values.length; size *= 2) {
      const count = Math.floor(values.length / size);
      this.levels.push({ first: new Float64Array(count), last: new Float64Array(count), lo: new Float64Array(count), hi: new Float64Array(count), loAt: new Int32Array(count), hiAt: new Int32Array(count), nan: new Uint8Array(count), low: new Uint8Array(count) });
    }
    for (let k = 0; k < (this.levels[0]?.first.length ?? 0); k++) this.#update(0, k);
    for (let level = 1; level < this.levels.length; level++) for (let k = 0; k < this.levels[level].first.length; k++) this.#update(level, k);
  }

  /** Its blocks over index `i` made again (the value there changed). @param {number} i */
  set(i) {
    for (let level = 0; level < this.levels.length; level++) {
      const k = i >> (level + FIRST);
      if (k < this.levels[level].first.length) this.#update(level, k);
    }
  }

  /** A block made: the first level's from its points, the others' from their two halves. @param {number} level @param {number} k */
  #update(level, k) {
    const at = this.levels[level];
    if (level === 0) {
      const values = this.values, from = k << FIRST, to = from + (1 << FIRST);
      let lo = Infinity, hi = -Infinity, loAt = from, hiAt = from, nan = 0, low = 0;
      for (let i = from; i < to; i++) {
        const v = values[i];
        if (v !== v) (nan = 1), (low = 1);
        else {
          if (!(v > 0)) low = 1;
          if (v < lo) (lo = v), (loAt = i);
          if (v > hi) (hi = v), (hiAt = i);
        }
      }
      (at.first[k] = values[from]), (at.last[k] = values[to - 1]), (at.lo[k] = lo), (at.hi[k] = hi), (at.loAt[k] = loAt), (at.hiAt[k] = hiAt), (at.nan[k] = nan), (at.low[k] = low);
      return;
    }
    const below = this.levels[level - 1], a = 2 * k, b = a + 1;
    at.first[k] = below.first[a];
    at.last[k] = below.last[b];
    below.lo[b] < below.lo[a] ? ((at.lo[k] = below.lo[b]), (at.loAt[k] = below.loAt[b])) : ((at.lo[k] = below.lo[a]), (at.loAt[k] = below.loAt[a]));
    below.hi[b] > below.hi[a] ? ((at.hi[k] = below.hi[b]), (at.hiAt[k] = below.hiAt[b])) : ((at.hi[k] = below.hi[a]), (at.hiAt[k] = below.hiAt[a]));
    at.nan[k] = below.nan[a] | below.nan[b];
    at.low[k] = below.low[a] | below.low[b];
  }

}

export { FIRST };
