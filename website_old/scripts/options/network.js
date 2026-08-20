/** Network section - On-chain activity and health */

import { colors } from "../utils/colors.js";
import { brk } from "../utils/client.js";
import { Unit } from "../utils/units.js";
import {
  line,
  baseline,
  fromSupplyPattern,
  chartsFromFull,
  chartsFromCount,
  chartsFromPercentCumulative,
  chartsFromPercentCumulativeEntries,
  chartsFromAggregatedPerBlock,
  distributionWindowsTree,
  averagesArray,
  simpleDeltaTree,
  ROLLING_WINDOWS,
  multiSeriesTree,
  percentRatio,
  percentRatioDots,
} from "./series.js";
import {
  satsBtcUsd,
  satsBtcUsdFrom,
  formatCohortTitle,
  groupedWindowsCumulative,
  avgHoldingsSubtree,
  exposedSubtree,
  reusedCountTree,
  reusedSubtree,
} from "./shared.js";
import { createOpReturnSection } from "./network/op-return.js";
import { createTransactionsSection } from "./network/transactions.js";

/**
 * Create Network section
 * @returns {PartialOptionsGroup}
 */
export function createNetworkSection() {
  const { blocks, transactions, inputs, outputs, supply, addrs, cohorts } =
    brk.series;

  const st = colors.scriptType;

  // Addressable types - newest to oldest (for addresses/counts that only support addressable types)
  const addressTypes = /** @type {const} */ ([
    { key: "p2a", name: "P2A", color: st.p2a, defaultActive: false },
    { key: "p2tr", name: "P2TR", color: st.p2tr, defaultActive: true },
    { key: "p2wsh", name: "P2WSH", color: st.p2wsh, defaultActive: true },
    { key: "p2wpkh", name: "P2WPKH", color: st.p2wpkh, defaultActive: true },
    { key: "p2sh", name: "P2SH", color: st.p2sh, defaultActive: true },
    { key: "p2pkh", name: "P2PKH", color: st.p2pkh, defaultActive: true },
    { key: "p2pk33", name: "P2PK33", color: st.p2pk33, defaultActive: false },
    { key: "p2pk65", name: "P2PK65", color: st.p2pk65, defaultActive: false },
  ]);

  // Non-addressable script types, reverse creation with catch-alls at tail
  const nonAddressableTypes = /** @type {const} */ ([
    {
      key: "opReturn",
      name: "OP_RETURN",
      color: st.opReturn,
      defaultActive: true,
    },
    { key: "p2ms", name: "P2MS", color: st.p2ms, defaultActive: false },
    {
      key: "empty",
      name: "Empty",
      color: st.empty,
      defaultActive: false,
    },
    {
      key: "unknown",
      name: "Unknown",
      color: st.unknown,
      defaultActive: false,
    },
  ]);

  // All output types = addressable + non-addressable (12 total)
  const outputTypes = [...addressTypes, ...nonAddressableTypes];
  // Spendable input types: every output type can fund an input *except* OP_RETURN
  const inputTypes = [
    ...addressTypes,
    ...nonAddressableTypes.filter((t) => t.key !== "opReturn"),
  ];

  // Transacting types (transaction participation)
  const activityTypes = /** @type {const} */ ([
    { key: "active", name: "Active" },
    { key: "sending", name: "Sending" },
    { key: "receiving", name: "Receiving" },
    { key: "bidirectional", name: "Bidirectional" },
    { key: "reactivated", name: "Reactivated" },
  ]);

  const countMetrics = /** @type {const} */ ([
    { key: "funded", name: "Funded", color: undefined },
    { key: "empty", name: "Empty", color: colors.gray },
    { key: "total", name: "Total", color: colors.default },
  ]);

  const reusedSubtreeForAll =
    /** @param {(name: string) => string} title */
    (title) => {
      const reused = addrs.reused;
      const respent = addrs.respent;
      const key = /** @type {const} */ ("all");

      return {
        name: "Reused",
        tree: [
          {
            name: "Funded",
            title: title("Funded Reused Addresses"),
            bottom: [
              line({
                series: reused.count.funded[key],
                name: "2+ Funded",
                unit: Unit.count,
              }),
              line({
                series: respent.count.funded[key],
                name: "2+ Spent",
                color: colors.gray,
                unit: Unit.count,
              }),
            ],
          },
          {
            name: "Total",
            title: title("Total Reused Addresses"),
            bottom: [
              line({
                series: reused.count.total[key],
                name: "2+ Funded",
                unit: Unit.count,
              }),
              line({
                series: respent.count.total[key],
                name: "2+ Spent",
                color: colors.gray,
                unit: Unit.count,
              }),
            ],
          },
          {
            name: "Active",
            tree: [
              {
                name: "Count",
                tree: ROLLING_WINDOWS.map((w) => ({
                  name: w.name,
                  title: title(`${w.title} Active Reused Addresses`),
                  bottom: [
                    line({
                      series: reused.events.activeReusedAddrCount[w.key],
                      name: "2+ Funded",
                      unit: Unit.count,
                    }),
                    line({
                      series: respent.events.activeReusedAddrCount[w.key],
                      name: "2+ Spent",
                      color: colors.gray,
                      unit: Unit.count,
                    }),
                  ],
                })),
              },
              {
                name: "Share",
                tree: ROLLING_WINDOWS.map((w) => ({
                  name: w.name,
                  title: title(`${w.title} Active Reused Address Share`),
                  bottom: [
                    line({
                      series: reused.events.activeReusedAddrShare[w.key],
                      name: "2+ Funded",
                      unit: Unit.percentage,
                    }),
                    line({
                      series: respent.events.activeReusedAddrShare[w.key],
                      name: "2+ Spent",
                      color: colors.gray,
                      unit: Unit.percentage,
                    }),
                  ],
                })),
              },
            ],
          },
          {
            name: "Outputs",
            tree: [
              {
                name: "Count",
                tree: reusedCountTree(
                  reused.events.outputToReusedAddrCount[key],
                  respent.events.outputToReusedAddrCount[key],
                  title,
                  "Transaction Outputs to Reused Addresses",
                ),
              },
              {
                name: "Share",
                tree: [
                  {
                    name: "All",
                    tree: chartsFromPercentCumulativeEntries({
                      entries: [
                        {
                          name: "2+ Funded",
                          pattern: reused.events.outputToReusedAddrShare[key],
                        },
                        {
                          name: "2+ Spent",
                          pattern: respent.events.outputToReusedAddrShare[key],
                          color: colors.gray,
                        },
                      ],
                      title,
                      metric: "Share of Transaction Outputs to Reused Addresses",
                    }),
                  },
                  {
                    name: "Spendable",
                    tree: chartsFromPercentCumulativeEntries({
                      entries: [
                        {
                          name: "2+ Funded",
                          pattern: reused.events.spendableOutputToReusedAddrShare,
                        },
                        {
                          name: "2+ Spent",
                          pattern: respent.events.spendableOutputToReusedAddrShare,
                          color: colors.gray,
                        },
                      ],
                      title,
                      metric: "Share of Spendable Transaction Outputs to Reused Addresses",
                    }),
                  },
                ],
              },
            ],
          },
          {
            name: "Inputs",
            tree: [
              {
                name: "Count",
                tree: reusedCountTree(
                  reused.events.inputFromReusedAddrCount[key],
                  respent.events.inputFromReusedAddrCount[key],
                  title,
                  "Transaction Inputs from Reused Addresses",
                ),
              },
              {
                name: "Share",
                tree: chartsFromPercentCumulativeEntries({
                  entries: [
                    {
                      name: "2+ Funded",
                      pattern: reused.events.inputFromReusedAddrShare[key],
                    },
                    {
                      name: "2+ Spent",
                      pattern: respent.events.inputFromReusedAddrShare[key],
                      color: colors.gray,
                    },
                  ],
                  title,
                  metric: "Share of Transaction Inputs from Reused Addresses",
                }),
              },
            ],
          },
          {
            name: "Supply",
            title: title("Supply in Reused Addresses"),
            bottom: [
              ...satsBtcUsd({ pattern: reused.supply[key], name: "2+ Funded" }),
              ...satsBtcUsd({
                pattern: respent.supply[key],
                name: "2+ Spent",
                color: colors.gray,
              }),
            ],
          },
          {
            name: "Share",
            title: title("Share of Supply in Reused Addresses"),
            bottom: [
              ...percentRatio({
                pattern: reused.supply.share[key],
                name: "2+ Funded",
              }),
              ...percentRatio({
                pattern: respent.supply.share[key],
                name: "2+ Spent",
                color: colors.gray,
              }),
            ],
          },
        ],
      };
    };


  const countSubtree =
    /**
     * @param {AddressableType | "all"} key
     * @param {(name: string) => string} title
     */
    (key, title) => ({
      name: "Count",
      tree: [
        {
          name: "Compare",
          title: title("Address Count"),
          bottom: countMetrics.map((m) =>
            line({
              series: addrs[m.key][key],
              name: m.name,
              color: m.color,
              unit: Unit.count,
            }),
          ),
        },
        ...countMetrics.map((m) => ({
          name: m.name,
          title: title(`${m.name} Addresses`),
          bottom: [
            line({
              series: addrs[m.key][key],
              name: m.name,
              unit: Unit.count,
            }),
          ],
        })),
      ],
    });


  const activityPerTypeEntries =
    /**
     * @param {AddressableType | "all"} key
     * @param {(name: string) => string} title
     */
    (key, title) =>
      activityTypes.map((t) => ({
        name: t.name,
        tree: averagesArray({
          windows: addrs.activity[key][t.key],
          title,
          metric: `${t.name} Addresses`,
          unit: Unit.count,
        }),
      }));

  const activitySubtreeForAll =
    /** @param {(name: string) => string} title */
    (title) => ({
      name: "Activity",
      tree: [
        {
          name: "Compare",
          tree: ROLLING_WINDOWS.map((w) => ({
            name: w.name,
            title: title(`${w.title} Active Addresses`),
            bottom: [
              ...activityTypes.map((t, i) =>
                line({
                  series: addrs.activity.all[t.key][w.key],
                  name: t.name,
                  color: colors.at(i, activityTypes.length),
                  unit: Unit.count,
                }),
              ),
              line({
                series: addrs.reused.events.activeReusedAddrShare[w.key],
                name: "Reused Share",
                unit: Unit.percentage,
              }),
              line({
                series: addrs.respent.events.activeReusedAddrShare[w.key],
                name: "Respent Share",
                color: colors.gray,
                unit: Unit.percentage,
              }),
            ],
          })),
        },
        ...activityPerTypeEntries("all", title),
      ],
    });

  const activitySubtreeForType =
    /**
     * @param {AddressableType} key
     * @param {(name: string) => string} title
     */
    (key, title) => ({
      name: "Activity",
      tree: [
        {
          name: "Compare",
          tree: ROLLING_WINDOWS.map((w) => ({
            name: w.name,
            title: title(`${w.title} Active Addresses`),
            bottom: activityTypes.map((t, i) =>
              line({
                series: addrs.activity[key][t.key][w.key],
                name: t.name,
                color: colors.at(i, activityTypes.length),
                unit: Unit.count,
              }),
            ),
          })),
        },
        ...activityPerTypeEntries(key, title),
      ],
    });

  const createAddressSeriesTreeForAll = () => {
    const title = formatCohortTitle();
    return [
      countSubtree("all", title),
      {
        name: "New",
        tree: chartsFromCount({
          pattern: addrs.new.all,
          title,
          metric: "New Addresses",
          unit: Unit.count,
        }),
      },
      ...simpleDeltaTree({
        delta: addrs.delta.all,
        title,
        metric: "Address Count",
        unit: Unit.count,
      }),
      activitySubtreeForAll(title),
      reusedSubtreeForAll(title),
      exposedSubtree(addrs.exposed, "all", title),
      avgHoldingsSubtree(addrs.avgAmount.all, title),
    ];
  };

  const createAddressSeriesTreeForType =
    /**
     * @param {AddressableType} addrType
     * @param {string} typeName
     */
    (addrType, typeName) => {
      const title = formatCohortTitle(typeName);
      return [
        countSubtree(addrType, title),
        {
          name: "New",
          tree: chartsFromCount({
            pattern: addrs.new[addrType],
            title,
            metric: "New Addresses",
            unit: Unit.count,
          }),
        },
        ...simpleDeltaTree({
          delta: addrs.delta[addrType],
          title,
          metric: "Address Count",
          unit: Unit.count,
        }),
        activitySubtreeForType(addrType, title),
        reusedSubtree(addrs.reused, addrs.respent, addrType, title),
        exposedSubtree(addrs.exposed, addrType, title),
        avgHoldingsSubtree(addrs.avgAmount[addrType], title),
      ];
    };

  /**
   * Mirror of the per-type singles tree, but every leaf is a cross-type
   * comparison chart (same metric, same unit, one line per addr type).
   * Structure parallels `createAddressSeriesTreeForType` section-by-section
   * so users can compare anything they can view on a single type.
   */
  const createAddressByTypeCompare = () => {
    const typeLines =
      /**
       * @param {(t: (typeof addressTypes)[number]) => AnySeriesPattern} getSeries
       * @param {Unit} [unit]
       */
      (getSeries, unit = Unit.count) =>
        addressTypes.map((t) =>
          line({
            series: getSeries(t),
            name: t.name,
            color: t.color,
            unit,
            defaultActive: t.defaultActive,
          }),
        );

    const typeBaselines =
      /**
       * @param {(t: (typeof addressTypes)[number]) => AnySeriesPattern} getSeries
       * @param {Unit} [unit]
       */
      (getSeries, unit = Unit.count) =>
        addressTypes.map((t) =>
          baseline({
            series: getSeries(t),
            name: t.name,
            color: t.color,
            unit,
            defaultActive: t.defaultActive,
          }),
        );

    const reuseCompareByTypeFolder =
      /**
       * @param {string} label - "Reused" or "Respent"
       * @param {ReusedTree | RespentTree} tree
       * @returns {PartialOptionsGroup}
       */
      (label, tree) => ({
        name: label,
        tree: [
          {
            name: "Funded",
            title: `Funded ${label} Address Count by Type`,
            bottom: typeLines((t) => tree.count.funded[t.key]),
          },
          {
            name: "Total",
            title: `Total ${label} Address Count by Type`,
            bottom: typeLines((t) => tree.count.total[t.key]),
          },
          {
            name: "Outputs",
            tree: [
              {
                name: "Count",
                tree: groupedWindowsCumulative({
                  list: addressTypes,
                  title: (s) => s,
                  metricTitle: `Transaction Outputs to ${label} Addresses by Type`,
                  getWindowSeries: (t, key) =>
                    tree.events.outputToReusedAddrCount[t.key].sum[key],
                  getCumulativeSeries: (t) =>
                    tree.events.outputToReusedAddrCount[t.key].cumulative,
                  seriesFn: line,
                  unit: Unit.count,
                }),
              },
              {
                name: "Share",
                tree: [
                  ...ROLLING_WINDOWS.map((w) => ({
                    name: w.name,
                    title: `${w.title} Share of Transaction Outputs to ${label} Addresses by Type`,
                    bottom: typeLines(
                      (t) =>
                        tree.events.outputToReusedAddrShare[t.key][w.key]
                          .percent,
                      Unit.percentage,
                    ),
                  })),
                  {
                    name: "Cumulative",
                    title: `Cumulative Share of Transaction Outputs to ${label} Addresses by Type`,
                    bottom: typeLines(
                      (t) =>
                        tree.events.outputToReusedAddrShare[t.key].percent,
                      Unit.percentage,
                    ),
                  },
                ],
              },
            ],
          },
          {
            name: "Inputs",
            tree: [
              {
                name: "Count",
                tree: groupedWindowsCumulative({
                  list: addressTypes,
                  title: (s) => s,
                  metricTitle: `Transaction Inputs from ${label} Addresses by Type`,
                  getWindowSeries: (t, key) =>
                    tree.events.inputFromReusedAddrCount[t.key].sum[key],
                  getCumulativeSeries: (t) =>
                    tree.events.inputFromReusedAddrCount[t.key].cumulative,
                  seriesFn: line,
                  unit: Unit.count,
                }),
              },
              {
                name: "Share",
                tree: [
                  ...ROLLING_WINDOWS.map((w) => ({
                    name: w.name,
                    title: `${w.title} Share of Transaction Inputs from ${label} Addresses by Type`,
                    bottom: typeLines(
                      (t) =>
                        tree.events.inputFromReusedAddrShare[t.key][w.key]
                          .percent,
                      Unit.percentage,
                    ),
                  })),
                  {
                    name: "Cumulative",
                    title: `Cumulative Share of Transaction Inputs from ${label} Addresses by Type`,
                    bottom: typeLines(
                      (t) => tree.events.inputFromReusedAddrShare[t.key].percent,
                      Unit.percentage,
                    ),
                  },
                ],
              },
            ],
          },
        ],
      });

    return {
      name: "Compare",
      tree: [
        // Count (lifetime Funded/Empty/Total)
        {
          name: "Count",
          tree: countMetrics.map((m) => ({
            name: m.name,
            title: `${m.name} Address Count by Type`,
            bottom: typeLines((t) => addrs[m.key][t.key]),
          })),
        },

        // New (rolling sums + cumulative)
        {
          name: "New",
          tree: groupedWindowsCumulative({
            list: addressTypes,
            title: (s) => s,
            metricTitle: "New Addresses by Type",
            getWindowSeries: (t, key) => addrs.new[t.key].sum[key],
            getCumulativeSeries: (t) => addrs.new[t.key].cumulative,
            seriesFn: line,
            unit: Unit.count,
          }),
        },

        // Change (rolling deltas, signed, baseline)
        {
          name: "Change",
          tree: ROLLING_WINDOWS.map((w) => ({
            name: w.name,
            title: `${w.title} Address Count Change by Type`,
            bottom: typeBaselines(
              (t) => addrs.delta[t.key].absolute[w.key],
            ),
          })),
        },

        // Growth Rate (rolling percent rates)
        {
          name: "Growth Rate",
          tree: ROLLING_WINDOWS.map((w) => ({
            name: w.name,
            title: `${w.title} Address Growth Rate by Type`,
            bottom: typeLines(
              (t) => addrs.delta[t.key].rate[w.key].percent,
              Unit.percentage,
            ),
          })),
        },

        // Activity (per activity type, per window)
        {
          name: "Activity",
          tree: activityTypes.map((a) => ({
            name: a.name,
            tree: ROLLING_WINDOWS.map((w) => ({
              name: w.name,
              title: `${w.title} ${a.name} Addresses by Type`,
              bottom: typeLines(
                (t) => addrs.activity[t.key][a.key][w.key],
              ),
            })),
          })),
        },

        // Address Reuse: receive-side (Reused) + spend-side (Respent)
        {
          name: "Address Reuse",
          tree: [
            reuseCompareByTypeFolder("Reused", addrs.reused),
            reuseCompareByTypeFolder("Respent", addrs.respent),
          ],
        },

        // Exposed
        {
          name: "Exposed",
          tree: [
            {
              name: "Funded",
              title: "Funded Exposed Address Count by Type",
              bottom: typeLines((t) => addrs.exposed.count.funded[t.key]),
            },
            {
              name: "Total",
              title: "Total Exposed Address Count by Type",
              bottom: typeLines((t) => addrs.exposed.count.total[t.key]),
            },
            {
              name: "Supply",
              title: "Supply in Exposed Addresses by Type",
              bottom: addressTypes.flatMap((t) =>
                satsBtcUsd({
                  pattern: addrs.exposed.supply[t.key],
                  name: t.name,
                  color: t.color,
                  defaultActive: t.defaultActive,
                }),
              ),
            },
            {
              name: "Share",
              title: "Share of Supply in Exposed Addresses by Type",
              bottom: addressTypes.flatMap((t) =>
                percentRatio({
                  pattern: addrs.exposed.supply.share[t.key],
                  name: t.name,
                  color: t.color,
                  defaultActive: t.defaultActive,
                }),
              ),
            },
          ],
        },

        // Average Holdings
        {
          name: "Average Holdings",
          tree: [
            {
              name: "Per UTXO",
              title: "Average Holdings per UTXO by Type",
              bottom: addressTypes.flatMap((t) =>
                satsBtcUsd({
                  pattern: addrs.avgAmount[t.key].utxo,
                  name: t.name,
                  color: t.color,
                  defaultActive: t.defaultActive,
                }),
              ),
            },
            {
              name: "Per Address",
              title: "Average Holdings per Funded Address by Type",
              bottom: addressTypes.flatMap((t) =>
                satsBtcUsd({
                  pattern: addrs.avgAmount[t.key].addr,
                  name: t.name,
                  color: t.color,
                  defaultActive: t.defaultActive,
                }),
              ),
            },
          ],
        },
      ],
    };
  };

  /**
   * Build a "By Type" subtree: Compare (count / tx count / tx %) plus a
   * per-type drill-down with the same three metrics.
   *
   * @template {string} K
   * @param {Object} args
   * @param {string} args.label - Singular noun for count/tree labels ("Output" / "Prev-Out")
   * @param {Readonly<Record<K | "all", CountPattern<number>>>} args.count
   * @param {Readonly<Record<K | "all", CountPattern<number>>>} args.txCount
   * @param {Readonly<Record<K, PercentRatioCumulativePattern>>} args.share
   * @param {Readonly<Record<K, PercentRatioCumulativePattern>>} args.txShare
   * @param {ReadonlyArray<{key: K, name: string, color: Color, defaultActive: boolean}>} args.types
   * @returns {PartialOptionsTree}
   */
  const createByTypeTree = ({
    label,
    count,
    share,
    txCount,
    txShare,
    types,
  }) => {
    const lowerLabel = label.toLowerCase();
    const countTypes = /** @type {const} */ ([
      ...types,
      {
        key: "all",
        name: "All",
        color: colors.default,
        defaultActive: false,
      },
    ]);

    /**
     * @param {Object} args
     * @param {Readonly<Record<string, CountPattern<number>>>} args.patterns
     * @param {(window: (typeof ROLLING_WINDOWS)[number], average: boolean) => string} args.windowTitle
     * @param {string} args.cumulativeTitle
     * @returns {PartialOptionsTree}
     */
    const countComparisonTree = ({
      patterns,
      windowTitle,
      cumulativeTitle,
    }) => [
      ...ROLLING_WINDOWS.map((window) => ({
        name: window.name,
        title: windowTitle(window, false),
        bottom: countTypes.map((type) =>
          line({
            series: patterns[type.key].sum[window.key],
            name: type.name,
            color: type.color,
            unit: Unit.count,
            defaultActive: type.defaultActive,
          }),
        ),
      })),
      {
        name: "Average",
        tree: ROLLING_WINDOWS.map((window) => ({
          name: window.name,
          title: windowTitle(window, true),
          bottom: countTypes.map((type) =>
            line({
              series: patterns[type.key].average[window.key],
              name: type.name,
              color: type.color,
              unit: Unit.count,
              defaultActive: type.defaultActive,
            }),
          ),
        })),
      },
      {
        name: "Cumulative",
        title: cumulativeTitle,
        bottom: countTypes.map((type) =>
          line({
            series: patterns[type.key].cumulative,
            name: type.name,
            color: type.color,
            unit: Unit.count,
            defaultActive: type.defaultActive,
          }),
        ),
      },
    ];

    return [
      {
        name: "Compare",
        tree: [
          {
            name: "Count",
            tree: countComparisonTree({
              patterns: count,
              windowTitle: (window, average) =>
                `${window.title}${average ? " Average" : ""} ${label} Count by Type`,
              cumulativeTitle: `Cumulative ${label} Count by Type`,
            }),
          },
          {
            name: "Share",
            tree: groupedWindowsCumulative({
              list: types,
              title: (n) => n,
              metricTitle: `Share of ${label}s by Type`,
              getWindowSeries: (t, key) => share[t.key][key].percent,
              getCumulativeSeries: (t) => share[t.key].percent,
              seriesFn: line,
              unit: Unit.percentage,
            }),
          },
          {
            name: "Transaction Count",
            tree: countComparisonTree({
              patterns: txCount,
              windowTitle: (window, average) =>
                `${window.title}${average ? " Average" : ""} Transactions by ${label} Type`,
              cumulativeTitle: `Cumulative Transactions by ${label} Type`,
            }),
          },
          {
            name: "Transaction Share",
            tree: [
              ...ROLLING_WINDOWS.map((w) => ({
                name: w.name,
                title: `${w.title} Share of Transactions by ${label} Type`,
                bottom: types.map((t) =>
                  line({
                    series: txShare[t.key][w.key].percent,
                    name: t.name,
                    color: t.color,
                    unit: Unit.percentage,
                    defaultActive: t.defaultActive,
                  }),
                ),
              })),
              {
                name: "Cumulative",
                title: `Cumulative Share of Transactions by ${label} Type`,
                bottom: types.map((t) =>
                  line({
                    series: txShare[t.key].percent,
                    name: t.name,
                    color: t.color,
                    unit: Unit.percentage,
                    defaultActive: t.defaultActive,
                  }),
                ),
              },
            ],
          },
        ],
      },
      ...types.map((t) => ({
        name: t.name,
        tree: [
          {
            name: "Count",
            tree: chartsFromCount({
              pattern: count[t.key],
              metric: `${t.name} ${label} Count`,
              unit: Unit.count,
              color: t.color,
            }),
          },
          {
            name: "Share",
            tree: chartsFromPercentCumulative({
              pattern: share[t.key],
              metric: `Share of ${label}s that are ${t.name}`,
              color: t.color,
            }),
          },
          {
            name: "Transaction Count",
            tree: chartsFromCount({
              pattern: txCount[t.key],
              metric: `Transactions with ${t.name} ${lowerLabel}`,
              unit: Unit.count,
              color: t.color,
            }),
          },
          {
            name: "Transaction Share",
            tree: chartsFromPercentCumulative({
              pattern: txShare[t.key],
              metric: `Share of Transactions with ${t.name} ${lowerLabel}`,
              color: t.color,
            }),
          },
        ],
      })),
    ];
  };

  return {
    name: "Network",
    tree: [
      // Supply
      {
        name: "Supply",
        tree: [
          {
            name: "Circulating",
            title: "Circulating Supply",
            bottom: fromSupplyPattern({
              pattern: supply.circulating,
              title: "Supply",
            }),
          },
          {
            name: "Inflation",
            title: "Inflation Rate",
            bottom: percentRatioDots({
              pattern: supply.inflationRate,
              name: "Rate",
            }),
          },
          {
            name: "Hodled or Lost",
            title: "Hodled or Lost Supply",
            bottom: satsBtcUsd({
              pattern: supply.hodledOrLost,
              name: "Supply",
            }),
          },
          {
            name: "Unspendable",
            tree: [
              {
                name: "All",
                title: "Unspendable Supply",
                bottom: satsBtcUsdFrom({
                  source: supply.burned,
                  key: "cumulative",
                  name: "All Time",
                }),
              },
              {
                name: "OP_RETURN",
                title: "OP_RETURN Burned",
                bottom: satsBtcUsd({
                  pattern: outputs.value.opReturn.cumulative,
                  name: "All Time",
                }),
              },
            ],
          },
        ],
      },

      // Blocks
      {
        name: "Blocks",
        tree: [
          {
            name: "Count",
            tree: [
              {
                name: "Compare",
                title: "Block Count",
                bottom: ROLLING_WINDOWS.map((w) =>
                  line({
                    series: blocks.count.total.sum[w.key],
                    name: w.name,
                    color: w.color,
                    unit: Unit.count,
                  }),
                ),
              },
              ...ROLLING_WINDOWS.map((w) => ({
                name: w.name,
                title: `${w.title} Block Count`,
                bottom: [
                  line({
                    series: blocks.count.total.sum[w.key],
                    name: "Actual",
                    unit: Unit.count,
                  }),
                  line({
                    series: blocks.count.target[w.key],
                    name: "Target",
                    color: colors.gray,
                    unit: Unit.count,
                    options: { lineStyle: 4 },
                  }),
                ],
              })),
              {
                name: "Cumulative",
                title: "Cumulative Block Count",
                bottom: [
                  {
                    series: blocks.count.total.cumulative,
                    title: "All Time",
                    unit: Unit.count,
                  },
                ],
              },
            ],
          },
          {
            name: "Interval",
            tree: averagesArray({
              windows: blocks.interval,
              metric: "Block Interval",
              unit: Unit.secs,
            }),
          },
          {
            name: "Size",
            tree: chartsFromFull({
              pattern: blocks.size,
              metric: "Block Size",
              unit: Unit.bytes,
            }),
          },
          {
            name: "Weight",
            tree: chartsFromFull({
              pattern: blocks.weight,
              metric: "Block Weight",
              unit: Unit.wu,
            }),
          },
          {
            name: "vBytes",
            tree: chartsFromFull({
              pattern: blocks.vbytes,
              metric: "Block vBytes",
              unit: Unit.vb,
            }),
          },
        ],
      },

      createTransactionsSection(),

      createOpReturnSection(),

      // UTXOs
      {
        name: "UTXOs",
        tree: [
          {
            name: "Count",
            title: "UTXO Count",
            bottom: [
              line({
                series: outputs.unspent.count,
                name: "Count",
                unit: Unit.count,
              }),
            ],
          },
          ...simpleDeltaTree({
            delta: cohorts.utxo.all.outputs.unspentCount.delta,
            metric: "UTXO Count",
            unit: Unit.count,
          }),
          {
            name: "Flow",
            tree: multiSeriesTree({
              entries: [
                {
                  name: "Created",
                  color: colors.entity.output,
                  average: outputs.count.total.rolling.average,
                  sum: outputs.count.total.rolling.sum,
                  cumulative: outputs.count.total.cumulative,
                },
                {
                  name: "Spent",
                  color: colors.entity.input,
                  average: inputs.count.rolling.average,
                  sum: inputs.count.rolling.sum,
                  cumulative: inputs.count.cumulative,
                },
              ],
              metric: "UTXO Flow",
              unit: Unit.count,
            }),
          },
        ],
      },
      {
        name: "Outputs",
        tree: [
          {
            name: "Count",
            tree: [
              {
                name: "Compare",
                title: "Output Count",
                bottom: ROLLING_WINDOWS.map((w) =>
                  line({
                    series: outputs.count.total.rolling.average[w.key],
                    name: w.name,
                    color: w.color,
                    unit: Unit.count,
                  }),
                ),
              },
              ...ROLLING_WINDOWS.map((w) => ({
                name: w.name,
                title: `${w.title} Output Count`,
                bottom: [
                  line({
                    series: outputs.count.total.rolling.sum[w.key],
                    name: "Total (Sum)",
                    color: w.color,
                    unit: Unit.count,
                  }),
                  line({
                    series: outputs.byType.spendableOutputCount.sum[w.key],
                    name: "Spendable (Sum)",
                    color: colors.gray,
                    unit: Unit.count,
                  }),
                  line({
                    series: outputs.count.total.rolling.average[w.key],
                    name: "Total (Avg)",
                    color: w.color,
                    unit: Unit.count,
                    defaultActive: false,
                  }),
                  line({
                    series: outputs.byType.spendableOutputCount.average[w.key],
                    name: "Spendable (Avg)",
                    color: colors.gray,
                    unit: Unit.count,
                    defaultActive: false,
                  }),
                ],
              })),
              {
                name: "Cumulative",
                title: "Cumulative Output Count",
                bottom: [
                  line({
                    series: outputs.count.total.cumulative,
                    name: "Total",
                    unit: Unit.count,
                  }),
                  line({
                    series: outputs.byType.spendableOutputCount.cumulative,
                    name: "Spendable",
                    color: colors.gray,
                    unit: Unit.count,
                  }),
                ],
              },
              distributionWindowsTree({
                pattern: outputs.count.total.rolling,
                metric: "Output Count per Block",
                unit: Unit.count,
              }),
            ],
          },
          {
            name: "Per Second",
            tree: averagesArray({
              windows: outputs.perSec,
              metric: "Outputs per Second",
              unit: Unit.perSec,
            }),
          },
          {
            name: "By Type",
            tree: createByTypeTree({
              label: "Output",
              count: outputs.byType.outputCount,
              share: outputs.byType.outputShare,
              txCount: outputs.byType.txCount,
              txShare: outputs.byType.txShare,
              types: outputTypes,
            }),
          },
        ],
      },
      {
        name: "Inputs",
        tree: [
          {
            name: "Count",
            tree: chartsFromAggregatedPerBlock({
              pattern: inputs.count,
              metric: "Input Count",
              unit: Unit.count,
            }),
          },
          {
            name: "Per Second",
            tree: averagesArray({
              windows: inputs.perSec,
              metric: "Inputs per Second",
              unit: Unit.perSec,
            }),
          },
          {
            name: "By Type",
            tree: createByTypeTree({
              label: "Prev-Out",
              count: inputs.byType.inputCount,
              share: inputs.byType.inputShare,
              txCount: inputs.byType.txCount,
              txShare: inputs.byType.txShare,
              types: inputTypes,
            }),
          },
        ],
      },
      {
        name: "Throughput",
        tree: ROLLING_WINDOWS.map((w) => ({
          name: w.name,
          title: `${w.title} Throughput`,
          bottom: [
            line({
              series: transactions.volume.txPerSec[w.key],
              name: "TX/sec",
              color: colors.entity.tx,
              unit: Unit.perSec,
            }),
            line({
              series: inputs.perSec[w.key],
              name: "Inputs/sec",
              color: colors.entity.input,
              unit: Unit.perSec,
            }),
            line({
              series: outputs.perSec[w.key],
              name: "Outputs/sec",
              color: colors.entity.output,
              unit: Unit.perSec,
            }),
          ],
        })),
      },

      // Addresses
      {
        name: "Addresses",
        tree: [
          ...createAddressSeriesTreeForAll(),
          {
            name: "By Type",
            tree: [
              createAddressByTypeCompare(),
              ...addressTypes.map((t) => ({
                name: t.name,
                tree: createAddressSeriesTreeForType(t.key, t.name),
              })),
            ],
          },
        ],
      },
    ],
  };
}
