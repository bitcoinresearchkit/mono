// @ts-check
// What it takes to get into a coming block, and how a fee rate is written: one rule and one wording for every page
// that says it.

/** @typedef {{ blockVSize: number, feeRange: number[] }} Projected A block from `/api/v1/fees/mempool-blocks`. */
/** @typedef {{ fee_histogram: [number, number][] }} Mempool `/api/mempool`. */

/** Said after the rate. */
export const GETS_IN = "sat/vB gets you in";

const written = [0, 1, 2, 3].map((digits) => new Intl.NumberFormat("en-US", { maximumFractionDigits: digits }));
export const rateNum = (/** @type {number} */ r) => written[r < 1 ? 3 : r < 10 ? 2 : r < 100 ? 1 : 0].format(r);

/** Up to the digits `rateNum` shows: a rate said to get in must be enough. */
export const ceilRate = (/** @type {number} */ r) => {
  const k = r < 1 ? 1000 : r < 10 ? 100 : r < 100 ? 10 : 1;
  return Math.ceil(r * k - 1e-9) / k;
};

/** The lowest rate waiting in the mempool, else the relay floor. */
const floorRate = (/** @type {Mempool} */ mempool) => {
  const rates = mempool.fee_histogram.map(([r]) => r).filter((r) => r > 0);
  return rates.length ? Math.min(...rates) : 0.1;
};

/** A full block's cheapest, else the mempool's lowest. */
export const entryRate = (/** @type {Projected | undefined} */ block, /** @type {Mempool} */ mempool) => {
  const floor = floorRate(mempool);
  return block && block.blockVSize >= 995000 ? Math.max(block.feeRange[0], floor) : floor;
};

/** The rate that gets into one of the next `n` blocks, rounded up. */
export const rateWithin = (/** @type {Projected[]} */ projected, /** @type {Mempool} */ mempool, /** @type {number} */ n) =>
  ceilRate(Math.min(...Array.from({ length: n }, (_, i) => entryRate(projected[i], mempool))));
