// @ts-check
// The chain's facts Studio's presets go by (presets.js), read from its server: apart, so the page reads them, and keys
// its presets' cache by them, without presets.js.

/**
 * @typedef {{ pools: Set<string>, recent: Set<string>, height: number, year: number, topPools: string[], names: Map<string, string> }} Facts the chain's, from its server:
 * the pools that have found a block, those that have in the last 3 years (the server's longest window short of all
 * time), its height, the year, the five pools with the most blocks lately (see TOP_POOLS_WINDOW) and every pool's name
 */

const TOP_POOLS_WINDOW = 2016 * 2;

/** @param {string} origin @returns {Promise<Facts>} */
export async function chainFacts(origin) {
  // Timed: a stalled server holds up nothing, the presets are then made without its facts.
  const read = (/** @type {string} */ path) => fetch(`${origin}${path}`, { signal: AbortSignal.timeout(15_000) }).then((response) => (response.ok ? response : Promise.reject(new Error(`${path}: ${response.status}`))));
  const [pools, recent, height, lately] = await Promise.all([read("/api/v1/mining/pools/all").then((r) => r.json()), read("/api/v1/mining/pools/3y").then((r) => r.json()), read("/api/blocks/tip/height").then((r) => r.text()).then(Number), read(`/api/series/block_pool/height?start=-${TOP_POOLS_WINDOW}`).then((r) => r.json())]);
  if (!Number.isFinite(height)) throw new Error("The chain's height isn't a number");
  const found = (/** @type {any} */ list) => new Set(list.pools.filter((/** @type {any} */ pool) => pool.blockCount > 0).map((/** @type {any} */ pool) => pool.slug));
  const blocksOf = new Map(/** @type {[string, number][]} */ ([]));
  for (const slug of lately.data) blocksOf.set(slug, (blocksOf.get(slug) ?? 0) + 1);
  const topPools = [...blocksOf].filter(([slug]) => slug !== "unknown").sort((a, b) => b[1] - a[1]).slice(0, 5).map(([slug]) => slug);
  const names = new Map(pools.pools.map((/** @type {any} */ pool) => [pool.slug, pool.name]));
  return { pools: found(pools), recent: found(recent), height, year: new Date().getUTCFullYear(), topPools, names };
}

/** @param {Facts | undefined} facts */
export const factsKey = (facts) => (facts ? JSON.stringify([[...facts.pools].sort(), [...facts.recent].sort(), Math.floor(facts.height / 210_000), facts.year, facts.topPools.map((slug) => [slug, facts.names.get(slug)])]) : "");

/** @param {string} key @returns {Facts} */
export function factsOfKey(key) {
  const [pools, recent, epoch, year, top] = JSON.parse(key);
  return { pools: new Set(pools), recent: new Set(recent), height: epoch * 210_000, year, topPools: top.map((/** @type {[string, string]} */ [slug]) => slug), names: new Map(top) };
}
