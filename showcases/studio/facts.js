// @ts-check
// The chain's facts Studio's presets go by (presets.js), read from its server: apart, so the page reads them, and keys
// its presets' cache by them, without presets.js.

/**
 * @typedef {{ pools: Set<string>, recent: Set<string>, height: number, year: number, topPools: string[], names: Map<string, string> }} Facts the chain's, from its server:
 * the pools that have found a block, those that have in the last 3 years (the server's longest window short of all
 * time), its height, the year, the five pools with the most blocks lately (see TOP_POOLS_WINDOW) and every pool's name
 */

/** The blocks "today's top pools" are counted over: two difficulty periods. */
const TOP_POOLS_WINDOW = 2016 * 2;

/**
 * The chain's facts the presets go by, from its server (`origin`): the pools that have found a block (see notYet) and
 * those that have lately (see poolCharts), its height, the year (see notYet), today's top pools and the pools' names (see
 * chainStatistics).
 * @param {string} origin @returns {Promise<Facts>}
 */
export async function chainFacts(origin) {
  // (In a time: a stalled server holds up nothing; its facts are then missing, and the presets made without them.)
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

/**
 * What of the chain's facts the presets read, as text, the same facts the same text: the pools found and lately, the
 * halving epoch (not the height: a block more changes nothing), the year, the top pools and their names. None, "".
 * @param {Facts | undefined} facts
 */
export const factsKey = (facts) => (facts ? JSON.stringify([[...facts.pools].sort(), [...facts.recent].sort(), Math.floor(facts.height / 210_000), facts.year, facts.topPools.map((slug) => [slug, facts.names.get(slug)])]) : "");
