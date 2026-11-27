// @ts-check
// The Bottoms ranking's rules, one function each: which days are cycle bottoms (`bottoms`), how far a line was from
// one (`miss`), and the order (`rank`). `price` is a day's price throughout, one series for every rule: the page's is
// the daily close. No DOM, no state. A change here changes the ranking.

/** @typedef {{ peak: number, low: number, open: boolean }} Bottom a fall's days: its all-time high's and its lowest price's; `open`: its cycle is still running, so it can move */

/**
 * The cycle bottoms. A fall runs from an all-time-high price to the lowest price before the next all-time high; each
 * halving cycle's bottom is the low of its deepest fall, if that fall is of half or more. A fall belongs to the cycle
 * its low is in, so the days a cycle opens on, still under an old high, are never its bottom.
 */
export function bottoms(/** @type {Float64Array} */ price, /** @type {Float64Array} */ epoch) {
  /** Each cycle's deepest fall. @type {{ peak: number, low: number }[]} */ const deepest = [];
  const left = (/** @type {{ peak: number, low: number }} */ f) => price[f.low] / price[f.peak];
  /** @type {{ peak: number, low: number } | undefined} */ let fall;
  const done = () => { if (!fall) return; const e = epoch[fall.low]; if (!deepest[e] || left(fall) < left(deepest[e])) deepest[e] = fall; };
  for (let d = 0; d < price.length; d++) {
    if (!(price[d] > 0) || !(epoch[d] === epoch[d])) continue;
    if (!fall || price[d] > price[fall.peak]) done(), (fall = { peak: d, low: d });
    else if (price[d] < price[fall.low]) fall.low = d;
  }
  done();
  return deepest.flatMap((f, e) => (left(f) <= 0.5 ? [/** @type {Bottom} */ ({ ...f, open: e === deepest.length - 1 })] : []));
}

/** @typedef {{ met: number, price: number, miss: number }} At (`met`: the day the falling price met the line, -1 never; `price`: the line's price there, or on the bottom's day; `miss`: percent) */
/**
 * How far a line's price was from a bottom's, in percent of the higher of the two.
 *
 * A line's value for a day is known only once the day is over, so each day's price is held against the line as it
 * stood the evening before. The price meets the line on the first day of the fall it is at or under it, at the
 * line's value. That meeting price is capped at the price of the day before: a line that had risen over the price was
 * met where the price was, not above it. A line never met is taken on the bottom's day.
 * @returns {At | null} null: the line has no value there
 */
export function miss(/** @type {Float64Array} */ price, /** @type {Float64Array} */ line, /** @type {{ peak: number, low: number }} */ { peak, low }) {
  const bottom = price[low], at = (/** @type {number} */ met, /** @type {number} */ price) => ({ met, price, miss: (1 - Math.min(price, bottom) / Math.max(price, bottom)) * 100 });
  for (let t = peak + 1; t <= low; t++) if (Math.min(price[t - 1], price[t]) <= line[t - 1]) return at(t, Math.min(line[t - 1], price[t - 1]));
  const stood = line[low - 1];
  return stood === stood ? at(-1, Math.max(0, stood)) : null;
}

/** @typedef {{ at: (At | null)[] }} Judged a line with its misses, one per bottom */
/**
 * The order: by each line's worst miss over the bottoms `counted` (indices into `at`), ties by its next worst, and so
 * on. Lines equal on every bottom share a rank; a line with no value at a bottom is last.
 * @template {Judged} R
 * @returns {{ row: R, rank: number, worst: number }[]}
 */
export function rank(/** @type {R[]} */ rows, /** @type {number[]} */ counted) {
  const keyed = rows.map((row) => ({ row, rank: 0, worst: 0, key: counted.map((k) => row.at[k]?.miss ?? Infinity).sort((a, b) => b - a) }));
  const order = (/** @type {number[]} */ a, /** @type {number[]} */ b) => { for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) return a[i] - b[i]; return 0; };
  keyed.sort((a, b) => order(a.key, b.key));
  keyed.forEach((r, i) => { (r.worst = r.key[0]), (r.rank = i && !order(r.key, keyed[i - 1].key) ? keyed[i - 1].rank : i + 1); });
  return keyed.map(({ row, rank, worst }) => ({ row, rank, worst }));
}
