// @ts-check
// A floor refit each day on the closes before it: the q-quantile regression of log price on log days since genesis
// (Sminston With's method). Fitted exactly: a two-parameter quantile regression's best line passes through two of its
// points, so the fit turns the line about one point to its best slope, then about the point that slope reaches, until
// the loss stops falling (Wesolowsky's descent, its weights made asymmetric). Each day starts from the day before's point.

/** The days worked out between two lets-go of the thread. */
const SLICE = 256;

/**
 * The smallest slope whose weight, with every smaller slope's, reaches `target`: a weighted quickselect over the first
 * `m` of `s` (moved in place, with their weights `w` and points `k`). @returns {number} its index
 */
function quantile(/** @type {Float64Array} */ s, /** @type {Float64Array} */ w, /** @type {Int32Array} */ k, /** @type {number} */ m, /** @type {number} */ target) {
  const swap = (/** @type {number} */ i, /** @type {number} */ j) => {
    let t = s[i]; (s[i] = s[j]), (s[j] = t);
    (t = w[i]), (w[i] = w[j]), (w[j] = t);
    (t = k[i]), (k[i] = k[j]), (k[j] = t);
  };
  for (let lo = 0, hi = m; ; ) {
    // (Three ways around the middle one: under it, equal to it, over it; the weights under and equal summed.)
    const at = s[(lo + hi) >> 1];
    let lt = lo, i = lo, gt = hi, under = 0, equal = 0;
    while (i < gt) {
      if (s[i] < at) (under += w[i]), swap(lt++, i++);
      else if (s[i] > at) swap(i, --gt);
      else (equal += w[i]), i++;
    }
    if (target <= under) hi = lt;
    else if (target <= under + equal) return lt;
    else (target -= under + equal), (lo = gt);
  }
}

/** The q-quantile line of the first `m` points, from the point `pivot`: its intercept, slope, and the point it turns on. */
function fit(/** @type {Float64Array} */ x, /** @type {Float64Array} */ y, /** @type {number} */ m, /** @type {number} */ q, /** @type {number} */ pivot, /** @type {[Float64Array, Float64Array, Int32Array]} */ [s, w, k]) {
  let best = Infinity, a = NaN, b = NaN, on = pivot;
  for (let p = pivot; ; ) {
    let j = 0, target = 0;
    for (let i = 0; i < m; i++) {
      const dx = x[i] - x[p];
      if (dx !== 0) (s[j] = (y[i] - y[p]) / dx), (w[j] = Math.abs(dx)), (k[j] = i), (target += w[j] * (dx > 0 ? q : 1 - q)), j++;
    }
    const t = quantile(s, w, k, j, target), slope = s[t], intercept = y[p] - slope * x[p];
    let loss = 0;
    for (let i = 0; i < m; i++) { const e = y[i] - intercept - slope * x[i]; loss += e >= 0 ? q * e : (q - 1) * e; }
    // (A turn that doesn't lower the loss: the line before it was the best.)
    if (!(loss < best - 1e-12)) return { a, b, pivot: on };
    (best = loss), (a = intercept), (b = slope), (on = p), (p = k[t]);
  }
}

/**
 * The floor by day, as long as `close`: each day's value the q-quantile line of every close before that day (days
 * counted from `genesis`), from the day `least` closes are in; NaN before. Worked out in slices of `SLICE` days, the
 * thread let go between them (a phone takes about a second in all).
 */
export async function refitFloor(/** @type {Float64Array} */ close, /** @type {number} */ genesis, /** @type {number} */ q, /** @type {number} */ least) {
  const n = close.length, out = new Float64Array(n).fill(NaN), x = new Float64Array(n), y = new Float64Array(n);
  /** @type {[Float64Array, Float64Array, Int32Array]} */ const room = [new Float64Array(n), new Float64Array(n), new Int32Array(n)];
  let m = 0, fitted = 0, line = { a: NaN, b: NaN, pivot: 0 };
  for (let d = genesis + 1; d < n; d++) {
    if (d % SLICE === 0) await new Promise((done) => setTimeout(done));
    if (m >= least) {
      if (fitted !== m) (line = fit(x, y, m, q, line.pivot, room)), (fitted = m);
      out[d] = 10 ** (line.a + line.b * Math.log10(d - genesis));
    }
    if (close[d] > 0) (x[m] = Math.log10(d - genesis)), (y[m] = Math.log10(close[d])), m++;
  }
  return out;
}
