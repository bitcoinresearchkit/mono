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

export { clamp, hashOf, lowerBound, uid };
