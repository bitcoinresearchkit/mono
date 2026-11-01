// @ts-check
// A column of values summed up by blocks of 4, 8, 16 points and so on (each level's blocks twice the last's): each
// block's low and high and where they are, its lowest above zero, and whether it has a point
// a line can't go through (none; on a log scale, none above zero). Zoomed out, a pixel column of many points reads its
// few whole blocks that are clean, its ends point by point: a draw's work by its columns, not by its points; a range's
// lowest and highest alike.

/** The first level's blocks: 4 points (2^2). */
const FIRST = 2;

export class Summary {
  /** @param {Float64Array} values */
  constructor(values) {
    /** @type {{ lo: Float64Array, hi: Float64Array, loAt: Int32Array, hiAt: Int32Array, pos: Float64Array, nan: Uint8Array, low: Uint8Array }[]} */
    this.levels = [];
    this.values = values;
    for (let size = 1 << FIRST; size <= values.length; size *= 2) {
      const count = Math.floor(values.length / size);
      this.levels.push({ lo: new Float64Array(count), hi: new Float64Array(count), loAt: new Int32Array(count), hiAt: new Int32Array(count), pos: new Float64Array(count), nan: new Uint8Array(count), low: new Uint8Array(count) });
    }
    // (Made in loops of their own, not a call a block: without a JIT, a call costs more than a block's work.)
    if (!this.levels.length) return;
    const { lo, hi, loAt, hiAt, pos, nan, low } = this.levels[0], size = 1 << FIRST;
    for (let k = 0, from = 0; k < lo.length; k++, from += size) {
      let l = Infinity, h = -Infinity, la = from, ha = from, p = Infinity, na = 0, lw = 0;
      for (let i = from; i < from + size; i++) {
        const v = values[i];
        if (v !== v) (na = 1), (lw = 1);
        else {
          if (!(v > 0)) lw = 1;
          else if (v < p) p = v;
          if (v < l) (l = v), (la = i);
          if (v > h) (h = v), (ha = i);
        }
      }
      (lo[k] = l), (hi[k] = h), (loAt[k] = la), (hiAt[k] = ha), (pos[k] = p), (nan[k] = na), (low[k] = lw);
    }
    for (let level = 1; level < this.levels.length; level++) {
      const at = this.levels[level], below = this.levels[level - 1], count = at.lo.length;
      const bLo = below.lo, bHi = below.hi, bLoAt = below.loAt, bHiAt = below.hiAt, bPos = below.pos, bNan = below.nan, bLow = below.low;
      for (let k = 0, a = 0; k < count; k++, a += 2) {
        const b = a + 1;
        bLo[b] < bLo[a] ? ((at.lo[k] = bLo[b]), (at.loAt[k] = bLoAt[b])) : ((at.lo[k] = bLo[a]), (at.loAt[k] = bLoAt[a]));
        bHi[b] > bHi[a] ? ((at.hi[k] = bHi[b]), (at.hiAt[k] = bHiAt[b])) : ((at.hi[k] = bHi[a]), (at.hiAt[k] = bHiAt[a]));
        at.pos[k] = bPos[b] < bPos[a] ? bPos[b] : bPos[a];
        at.nan[k] = bNan[a] | bNan[b];
        at.low[k] = bLow[a] | bLow[b];
      }
    }
  }

  /** Its blocks over index `i` made again (the value there changed). @param {number} i */
  set(i) {
    for (let level = 0; level < this.levels.length; level++) {
      const k = i >> (level + FIRST);
      if (k < this.levels[level].lo.length) this.#update(level, k);
    }
  }

  /** A block made: the first level's from its points, the others' from their two halves. @param {number} level @param {number} k */
  #update(level, k) {
    const at = this.levels[level];
    if (level === 0) {
      const values = this.values, from = k << FIRST, to = from + (1 << FIRST);
      let lo = Infinity, hi = -Infinity, loAt = from, hiAt = from, pos = Infinity, nan = 0, low = 0;
      for (let i = from; i < to; i++) {
        const v = values[i];
        if (v !== v) (nan = 1), (low = 1);
        else {
          if (!(v > 0)) low = 1;
          else if (v < pos) pos = v;
          if (v < lo) (lo = v), (loAt = i);
          if (v > hi) (hi = v), (hiAt = i);
        }
      }
      (at.lo[k] = lo), (at.hi[k] = hi), (at.loAt[k] = loAt), (at.hiAt[k] = hiAt), (at.pos[k] = pos), (at.nan[k] = nan), (at.low[k] = low);
      return;
    }
    const below = this.levels[level - 1], a = 2 * k, b = a + 1;
    below.lo[b] < below.lo[a] ? ((at.lo[k] = below.lo[b]), (at.loAt[k] = below.loAt[b])) : ((at.lo[k] = below.lo[a]), (at.loAt[k] = below.loAt[a]));
    below.hi[b] > below.hi[a] ? ((at.hi[k] = below.hi[b]), (at.hiAt[k] = below.hiAt[b])) : ((at.hi[k] = below.hi[a]), (at.hiAt[k] = below.hiAt[a]));
    at.pos[k] = below.pos[b] < below.pos[a] ? below.pos[b] : below.pos[a];
    at.nan[k] = below.nan[a] | below.nan[b];
    at.low[k] = below.low[a] | below.low[b];
  }

}

export { FIRST };
