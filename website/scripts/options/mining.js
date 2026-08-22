/** Mining section - Network security and miner economics */

import { Unit } from "../utils/units.js";
import { entries, includes } from "../utils/array.js";
import { colors } from "../utils/colors.js";
import {
  line,
  dots,
  dotted,
  distributionBtcSatsUsd,
  statsAtWindow,
  ROLLING_WINDOWS,
  percentRatio,
  percentRatioBaseline,
  chartsFromSumCumulative,
} from "./series.js";
import {
  satsBtcUsdFrom,
  satsBtcUsdFullTree,
  revenueBtcSatsUsd,
  revenueRollingBtcSatsUsd,
  formatCohortTitle,
} from "./shared.js";
import { brk } from "../utils/client.js";

/** Major pools to show in Compare section (by current hashrate dominance) */
const MAJOR_POOL_IDS = /** @type {const} */ ([
  "foundryusa",
  "antpool",
  "viabtc",
  "f2pool",
  "marapool",
  "braiinspool",
  "spiderpool",
  "ocean",
]);

/**
 * AntPool & friends - pools sharing AntPool's block templates
 * Based on b10c's research: https://b10c.me/blog/015-bitcoin-mining-centralization/
 */
const ANTPOOL_AND_FRIENDS_IDS = /** @type {const} */ ([
  "antpool",
  "poolin",
  "btccom",
  "braiinspool",
  "ultimuspool",
  "binancepool",
  "secpool",
  "sigmapoolcom",
  "rawpool",
  "luxor",
]);

/**
 * Create Mining section
 * @returns {PartialOptionsGroup}
 */
export function createMiningSection() {
  const { blocks, pools, mining } = brk.series;

  const majorPoolData = entries(pools.major).map(([id, pool]) => ({
    id,
    name: brk.POOL_ID_TO_POOL_NAME[id],
    pool,
  }));
  const minorPoolData = entries(pools.minor).map(([id, pool]) => ({
    id,
    name: brk.POOL_ID_TO_POOL_NAME[id],
    pool,
  }));

  const featuredPools = majorPoolData.filter((p) =>
    includes(MAJOR_POOL_IDS, p.id),
  );
  const antpoolFriends = majorPoolData.filter((p) =>
    includes(ANTPOOL_AND_FRIENDS_IDS, p.id),
  );

  /**
   * @param {(metric: string) => string} title
   * @param {string} metric
   * @param {DominancePattern} dominance
   */
  const dominanceTree = (title, metric, dominance) => ({
    name: "Dominance",
    tree: [
      {
        name: "Compare",
        title: title(metric),
        bottom: [
          ...ROLLING_WINDOWS.flatMap((w) =>
            percentRatio({
              pattern: dominance[w.key],
              name: w.name,
              color: w.color,
              defaultActive: w.key !== "_24h",
            }),
          ),
          ...percentRatio({
            pattern: dominance,
            name: "All Time",
            color: colors.time.all,
          }),
        ],
      },
      ...ROLLING_WINDOWS.map((w) => ({
        name: w.name,
        title: title(`${w.title} ${metric}`),
        bottom: percentRatio({
          pattern: dominance[w.key],
          name: "Dominance",
          color: w.color,
        }),
      })),
      {
        name: "All Time",
        title: title(`All Time ${metric}`),
        bottom: percentRatio({
          pattern: dominance,
          name: "Dominance",
          color: colors.time.all,
        }),
      },
    ],
  });

  /**
   * @param {typeof majorPoolData} poolList
   */
  const createPoolTree = (poolList) =>
    poolList.map(({ name, pool }) => {
      const title = formatCohortTitle(name);
      return {
        name,
        tree: [
          dominanceTree(title, "Dominance", pool.dominance),
          {
            name: "Blocks Mined",
            tree: chartsFromSumCumulative({
              pattern: pool.blocksMined,
              title,
              metric: "Blocks Mined",
              unit: Unit.count,
            }),
          },
          {
            name: "Rewards",
            tree: satsBtcUsdFullTree({
              pattern: pool.rewards,
              title,
              metric: "Rewards",
            }),
          },
        ],
      };
    });

  /**
   * @param {typeof minorPoolData} poolList
   */
  const createMinorPoolTree = (poolList) =>
    poolList.map(({ name, pool }) => {
      const title = formatCohortTitle(name);
      return {
        name,
        tree: [
          {
            name: "Dominance",
            title: title("Dominance"),
            bottom: percentRatio({
              pattern: pool.dominance,
              name: "All Time",
              color: colors.time.all,
            }),
          },
          {
            name: "Blocks Mined",
            tree: chartsFromSumCumulative({
              pattern: pool.blocksMined,
              title,
              metric: "Blocks Mined",
              unit: Unit.count,
            }),
          },
        ],
      };
    });

  /**
   * @param {string} groupTitle
   * @param {typeof majorPoolData} poolList
   * @param {string} [name]
   */
  const createPoolCompare = (groupTitle, poolList, name = "Compare") => ({
    name,
    tree: [
      {
        name: "Dominance",
        tree: ROLLING_WINDOWS.map((w) => ({
          name: w.name,
          title: formatCohortTitle(groupTitle)(`${w.title} Dominance`),
          bottom: poolList.flatMap((p, i) =>
            percentRatio({
              pattern: p.pool.dominance[w.key],
              name: p.name,
              color: colors.at(i, poolList.length),
            }),
          ),
        })),
      },
      {
        name: "Blocks Mined",
        tree: ROLLING_WINDOWS.map((w) => ({
          name: w.name,
          title: formatCohortTitle(groupTitle)(`${w.title} Blocks Mined`),
          bottom: poolList.map((p, i) =>
            line({
              series: p.pool.blocksMined.sum[w.key],
              name: p.name,
              color: colors.at(i, poolList.length),
              unit: Unit.count,
            }),
          ),
        })),
      },
    ],
  });

  return {
    name: "Mining",
    tree: [
      {
        name: "Hashrate",
        tree: [
          {
            name: "Current",
            title: "Network Hashrate",
            bottom: [
              dots({
                series: mining.hashrate.rate.base,
                name: "Hashrate",
                unit: Unit.hashRate,
              }),
              line({
                series: mining.hashrate.rate.sma._1w,
                name: "1w SMA",
                color: colors.time._1w,
                unit: Unit.hashRate,
                defaultActive: false,
              }),
              line({
                series: mining.hashrate.rate.sma._1m,
                name: "1m SMA",
                color: colors.time._1m,
                unit: Unit.hashRate,
                defaultActive: false,
              }),
              line({
                series: mining.hashrate.rate.sma._2m,
                name: "2m SMA",
                color: colors.indicator.main,
                unit: Unit.hashRate,
                defaultActive: false,
              }),
              line({
                series: mining.hashrate.rate.sma._1y,
                name: "1y SMA",
                color: colors.time._1y,
                unit: Unit.hashRate,
                defaultActive: false,
              }),
              dotted({
                series: blocks.difficulty.hashrate,
                name: "From Difficulty",
                color: colors.default,
                unit: Unit.hashRate,
              }),
              line({
                series: mining.hashrate.rate.ath,
                name: "ATH",
                color: colors.loss,
                unit: Unit.hashRate,
                defaultActive: false,
              }),
            ],
          },
          {
            name: "ATH",
            title: "Network Hashrate ATH",
            bottom: [
              line({
                series: mining.hashrate.rate.ath,
                name: "ATH",
                color: colors.loss,
                unit: Unit.hashRate,
              }),
              dots({
                series: mining.hashrate.rate.base,
                name: "Hashrate",
                color: colors.bitcoin,
                unit: Unit.hashRate,
              }),
            ],
          },
          {
            name: "Drawdown",
            title: "Network Hashrate Drawdown",
            bottom: percentRatio({
              pattern: mining.hashrate.rate.drawdown,
              name: "Drawdown",
              color: colors.loss,
            }),
          },
        ],
      },

      {
        name: "Revenue",
        tree: [
          ...ROLLING_WINDOWS.map((w) => ({
            name: w.name,
            title: `${w.title} Mining Revenue`,
            bottom: revenueRollingBtcSatsUsd({
              coinbase: mining.rewards.coinbase.average[w.key],
              subsidy: mining.rewards.subsidy.average[w.key],
              fee: mining.rewards.fees.average[w.key],
            }),
          })),
          {
            name: "Cumulative",
            title: "Cumulative Mining Revenue",
            bottom: revenueBtcSatsUsd({
              coinbase: mining.rewards.coinbase,
              subsidy: mining.rewards.subsidy,
              fee: mining.rewards.fees,
              key: "cumulative",
            }),
          },
          {
            name: "Coinbase",
            tree: satsBtcUsdFullTree({
              pattern: mining.rewards.coinbase,
              metric: "Coinbase Rewards",
            }),
          },
          {
            name: "Subsidy",
            tree: satsBtcUsdFullTree({
              pattern: mining.rewards.subsidy,
              metric: "Block Subsidy",
            }),
          },
          {
            name: "Fees",
            tree: [
              ...satsBtcUsdFullTree({
                pattern: mining.rewards.fees,
                metric: "Transaction Fee Revenue",
              }),
              {
                name: "Distribution",
                tree: ROLLING_WINDOWS.map((w) => ({
                  name: w.name,
                  title: `${w.title} Fee Revenue per Block Distribution`,
                  bottom: distributionBtcSatsUsd(
                    statsAtWindow(mining.rewards.fees, w.key),
                  ),
                })),
              },
            ],
          },
          {
            name: "Dominance",
            tree: [
              ...ROLLING_WINDOWS.map((w) => ({
                name: w.name,
                title: `${w.title} Mining Revenue Dominance`,
                bottom: [
                  ...percentRatio({
                    pattern: mining.rewards.subsidy.dominance[w.key],
                    name: "Subsidy",
                    color: colors.mining.subsidy,
                  }),
                  ...percentRatio({
                    pattern: mining.rewards.fees.dominance[w.key],
                    name: "Fees",
                    color: colors.mining.fee,
                  }),
                ],
              })),
              {
                name: "All Time",
                title: "All Time Mining Revenue Dominance",
                bottom: [
                  ...percentRatio({
                    pattern: mining.rewards.subsidy.dominance,
                    name: "Subsidy",
                    color: colors.mining.subsidy,
                  }),
                  ...percentRatio({
                    pattern: mining.rewards.fees.dominance,
                    name: "Fees",
                    color: colors.mining.fee,
                  }),
                ],
              },
            ],
          },
          {
            name: "Fee-to-Subsidy",
            tree: ROLLING_WINDOWS.map((w) => ({
              name: w.name,
              title: `${w.title} Fee-to-Subsidy Ratio`,
              bottom: percentRatio({
                pattern: mining.rewards.fees.toSubsidy[w.key],
                name: "Fees / Subsidy",
                color: colors.mining.fee,
              }),
            })),
          },
          {
            name: "Unclaimed",
            title: "Unclaimed Rewards",
            bottom: satsBtcUsdFrom({
              source: mining.rewards.unclaimed,
              key: "cumulative",
              name: "All Time",
            }),
          },
        ],
      },

      {
        name: "Economics",
        tree: [
          {
            name: "Hash Price",
            title: "Hash Price",
            bottom: [
              line({
                series: mining.hashrate.price.ths,
                name: "per TH/s",
                color: colors.usd,
                unit: Unit.usdPerThsPerDay,
              }),
              line({
                series: mining.hashrate.price.phs,
                name: "per PH/s",
                color: colors.usd,
                unit: Unit.usdPerPhsPerDay,
              }),
              dotted({
                series: mining.hashrate.price.thsMin,
                name: "per TH/s ATL",
                color: colors.stat.min,
                unit: Unit.usdPerThsPerDay,
              }),
              dotted({
                series: mining.hashrate.price.phsMin,
                name: "per PH/s ATL",
                color: colors.stat.min,
                unit: Unit.usdPerPhsPerDay,
              }),
            ],
          },
          {
            name: "Hash Value",
            title: "Hash Value",
            bottom: [
              line({
                series: mining.hashrate.value.ths,
                name: "per TH/s",
                color: colors.bitcoin,
                unit: Unit.satsPerThsPerDay,
              }),
              line({
                series: mining.hashrate.value.phs,
                name: "per PH/s",
                color: colors.bitcoin,
                unit: Unit.satsPerPhsPerDay,
              }),
              dotted({
                series: mining.hashrate.value.thsMin,
                name: "per TH/s ATL",
                color: colors.stat.min,
                unit: Unit.satsPerThsPerDay,
              }),
              dotted({
                series: mining.hashrate.value.phsMin,
                name: "per PH/s ATL",
                color: colors.stat.min,
                unit: Unit.satsPerPhsPerDay,
              }),
            ],
          },
          {
            name: "Recovery",
            title: "Hash Price & Value Recovery",
            bottom: [
              ...percentRatio({
                pattern: mining.hashrate.price.rebound,
                name: "Hash Price",
                color: colors.usd,
              }),
              ...percentRatio({
                pattern: mining.hashrate.value.rebound,
                name: "Hash Value",
                color: colors.bitcoin,
              }),
            ],
          },
        ],
      },

      {
        name: "Halving",
        tree: [
          {
            name: "Countdown",
            title: "Next Halving",
            bottom: [
              line({
                series: blocks.halving.blocksToHalving,
                name: "Blocks",
                unit: Unit.blocks,
              }),
              line({
                series: blocks.halving.daysToHalving,
                name: "Days",
                unit: Unit.days,
              }),
            ],
          },
          {
            name: "Epoch",
            title: "Halving Epoch",
            bottom: [
              line({
                series: blocks.halving.epoch,
                name: "Epoch",
                unit: Unit.epoch,
              }),
            ],
          },
        ],
      },

      {
        name: "Difficulty",
        tree: [
          {
            name: "Current",
            title: "Mining Difficulty",
            bottom: [
              line({
                series: blocks.difficulty.value,
                name: "Difficulty",
                unit: Unit.difficulty,
              }),
            ],
          },
          {
            name: "Adjustment",
            title: "Difficulty Adjustment",
            bottom: percentRatioBaseline({
              pattern: blocks.difficulty.adjustment,
              name: "Change",
            }),
          },
          {
            name: "Countdown",
            title: "Next Difficulty Adjustment",
            bottom: [
              line({
                series: blocks.difficulty.blocksToRetarget,
                name: "Blocks",
                unit: Unit.blocks,
              }),
              line({
                series: blocks.difficulty.daysToRetarget,
                name: "Days",
                unit: Unit.days,
              }),
            ],
          },
          {
            name: "Epoch",
            title: "Difficulty Epoch",
            bottom: [
              line({
                series: blocks.difficulty.epoch,
                name: "Epoch",
                unit: Unit.epoch,
              }),
            ],
          },
        ],
      },
      {
        name: "Pools",
        tree: [
          createPoolCompare("Major Pools", featuredPools, "Featured"),
          {
            name: "AntPool & Friends",
            tree: [
              createPoolCompare("AntPool & Friends", antpoolFriends),
              ...createPoolTree(antpoolFriends),
            ],
          },
          {
            name: "Major",
            tree: createPoolTree(majorPoolData),
          },
          {
            name: "Minor",
            tree: createMinorPoolTree(minorPoolData),
          },
        ],
      },
    ],
  };
}
