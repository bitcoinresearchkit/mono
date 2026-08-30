import { createCohortSeries } from "./cohort-series.js";
import { createRollingWindowSeries } from "./rolling-windows.js";
import { colors } from "../../utils/colors.js";
import { bitview } from "../../utils/client.js";

const poolNames = bitview.POOL_ID_TO_POOL_NAME;

/**
 * @template {keyof typeof poolNames} Key
 * @template Pool
 * @param {Record<Key, Pool>} pools
 */
function createPools(pools) {
  const entries = [];

  for (const key in pools) {
    entries.push({
      name: poolNames[key],
      pool: pools[key],
    });
  }

  return entries;
}

/**
 * @param {(window: RollingWindowKey) => TimeframeMetric} createMetric
 */
function createWindowSeries(createMetric) {
  return createRollingWindowSeries((window) => () => createMetric(window));
}

export const majorPools = createPools(bitview.series.pools.major);

export const minorPools = createPools(bitview.series.pools.minor);

export const majorPoolDominanceSeries = createCohortSeries(
  majorPools.map(({ name, pool }) => ({
    label: name,
    metric: () => pool.dominance._1m.percent,
  })),
);

export const majorPoolBlocksMinedSeries = createCohortSeries(
  majorPools.map(({ name, pool }) => ({
    label: name,
    metric: () => pool.blocksMined.sum._1m,
  })),
);

export const majorPoolRewardsSeries = createCohortSeries(
  majorPools.map(({ name, pool }) => ({
    label: name,
    metric: () => pool.rewards.sum._1m.btc,
  })),
);

/** @param {MajorPool} pool */
export function createMajorPoolDominanceSeries(pool) {
  const series = createWindowSeries(
    (window) => pool.dominance[window].percent,
  );

  series.push({
    label: "All time",
    color: colors.orange,
    metric: () => pool.dominance.percent,
  });

  return series;
}

/** @param {MajorPool} pool */
export function createMajorPoolBlocksMinedSeries(pool) {
  return createWindowSeries(
    (window) => pool.blocksMined.sum[window],
  );
}

/** @param {MajorPool} pool */
export function createMajorPoolRewardsSeries(pool) {
  return createWindowSeries(
    (window) => pool.rewards.sum[window].btc,
  );
}

/** @param {MinorPool} pool */
export function createMinorPoolDominanceSeries(pool) {
  return createCohortSeries([
    {
      label: "All time",
      color: colors.orange,
      metric: () => pool.dominance.percent,
    },
  ]);
}

/** @param {MinorPool} pool */
export function createMinorPoolBlocksMinedSeries(pool) {
  return createWindowSeries(
    (window) => pool.blocksMined.sum[window],
  );
}

/** @typedef {typeof bitview.series.pools.major.unknown} MajorPool */
/** @typedef {typeof bitview.series.pools.minor.blockfills} MinorPool */
