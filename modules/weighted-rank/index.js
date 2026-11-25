// @ts-check
/**
 * Weights added to slots, and the weight in the slots below any one, each in log time (a Fenwick tree): a value's
 * weighted rank among those added before it, its slot a bucket of values or its sorted rank.
 */
export class WeightedRank {
  #tree;
  #total = 0;
  constructor(/** @type {number} */ slots) {
    this.#tree = new Float64Array(slots + 1);
  }
  /** The weight added so far, in every slot. */
  get total() {
    return this.#total;
  }
  /** Adds `w` to `slot` (0 to slots - 1). */
  add(/** @type {number} */ slot, /** @type {number} */ w) {
    const tree = this.#tree;
    for (let j = slot + 1; j < tree.length; j += j & -j) tree[j] += w;
    this.#total += w;
  }
  /** The weight in the slots below `slot` (the first `slot` of them). */
  below(/** @type {number} */ slot) {
    const tree = this.#tree;
    let sum = 0;
    for (let j = slot; j > 0; j -= j & -j) sum += tree[j];
    return sum;
  }
}
