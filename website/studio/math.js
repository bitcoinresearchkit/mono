// @ts-check
// Plain number work any module may need, browser or not: a value held in bounds, a search of sorted times, a
// text's fingerprint, a fresh id.

const clamp = (value, lo, hi) => Math.max(lo, Math.min(hi, value));
const uid = () => Math.random().toString(36).slice(2, 10);
/** A short fingerprint of a text (two FNV-1a passes, as letters and digits): the same text, the same one. */
const hashOf = (/** @type {string} */ text) => {
  let a = 0x811c9dc5, b = 0x050c5d1f;
  for (let k = 0; k < text.length; k++) {
    const c = text.charCodeAt(k);
    a = Math.imul(a ^ c, 0x01000193);
    b = Math.imul(b ^ c, 0x5bd1e995);
  }
  return (a >>> 0).toString(36) + (b >>> 0).toString(36);
};
/** The first of sorted times at or after a time (their count, past the last). @param {ArrayLike<number>} times @param {number} time */
function lowerBound(times, time) {
  let lo = 0, hi = times.length;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (times[mid] < time) lo = mid + 1;
    else hi = mid;
  }
  return lo;
}

/**
 * A value as its pane's scale draws it, for its shape: a log scale's, its log; a percent or indexed one's, its change
 * from where it starts (`from`) over that one's size, as the chart's are (from a negative one, a rise still rises).
 */
const onScale = (/** @type {number} */ mode, /** @type {number} */ from) => (/** @type {number} */ value) => (!Number.isFinite(value) ? NaN : mode === 1 ? (value > 0 ? Math.log10(value) : NaN) : mode >= 2 ? (from ? (value - from) / Math.abs(from) : NaN) : value);
/**
 * Values as onScale puts them, all at once (from `first` to `last`, every one by default; NaN past them): by index,
 * finite by a subtraction, no call a value (thousands a picture, a phone's engine without a JIT).
 */
function onScaleAll(/** @type {ArrayLike<number>} */ values, /** @type {number} */ mode, /** @type {number} */ from, first = 0, last = values.length - 1) {
  const out = new Float64Array(values.length).fill(NaN), size = Math.abs(from);
  for (let i = Math.max(0, first), end = Math.min(last, values.length - 1); i <= end; i++) {
    const value = values[i];
    out[i] = value - value !== 0 ? NaN : mode === 1 ? (value > 0 ? Math.log10(value) : NaN) : mode >= 2 ? (from ? (value - from) / size : NaN) : value;
  }
  return out;
}
/**
 * A long series drawn small and smooth: the mean of each of `columns` columns, at its middle; a short one whole.
 * Finite points only, as [index, value]. @param {ArrayLike<number>} values @param {number} columns @returns {[number, number][]}
 */
function meansOf(values, columns) {
  /** @type {[number, number][]} */
  const points = [];
  const per = Math.max(1, values.length / columns);
  for (let start = 0; start < values.length; start += per) {
    let sum = 0, count = 0, at = 0;
    // (Finite by a subtraction: no call each value.)
    for (let i = Math.floor(start), end = Math.min(values.length, Math.floor(start + per)); i < end; i++) {
      const value = values[i];
      if (value - value !== 0) continue;
      sum += value;
      at += i;
      count++;
    }
    if (count) points.push([at / count, sum / count]);
  }
  return points;
}
export { clamp, hashOf, lowerBound, uid, onScale, onScaleAll, meansOf };
