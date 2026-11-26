// @ts-check
/** A Fenwick tree of weights per slot: a value's weighted rank among those added before it. */
export class WeightedRank {
  #tree;
  #total = 0;
  constructor(/** @type {number} */ slots) {
    this.#tree = new Float64Array(slots + 1);
  }
  get total() {
    return this.#total;
  }
  add(/** @type {number} */ slot, /** @type {number} */ w) {
    const tree = this.#tree;
    for (let j = slot + 1; j < tree.length; j += j & -j) tree[j] += w;
    this.#total += w;
  }
  below(/** @type {number} */ slot) {
    const tree = this.#tree;
    let sum = 0;
    for (let j = slot; j > 0; j -= j & -j) sum += tree[j];
    return sum;
  }
}
