/** Partial options - Main entry point */

import {
  buildCohortData,
  createCohortFolderAll,
  createCohortFolderFull,
  createCohortFolderCore,
  createCohortFolderLongTerm,
  createCohortFolderAgeRangeWithMatured,
  createCohortFolderBasicWithMarketCap,
  createCohortFolderWithoutRelative,
  createCohortFolderAddress,
  createAddressCohortFolder,
  createGroupedCohortFolderCore,
  createGroupedCohortFolderWithNupl,
  createGroupedCohortFolderAgeRangeWithMatured,
  createGroupedCohortFolderBasicWithMarketCap,
  createGroupedCohortFolderAddress,
  createGroupedAddressCohortFolder,
  createAddressBalanceGiniLeaf,
} from "./distribution/index.js";
import { createMarketSection } from "./market.js";
import { createNetworkSection } from "./network.js";
import { createMiningSection } from "./mining.js";
import { createCointimeSection } from "./frameworks/cointime/index.js";
import { createCoinflowSection } from "./frameworks/coinflow.js";
import { createCapitalSentimentSection } from "./models/capital-sentiment.js";
import { createBedrockSection } from "./models/bedrock.js";
import { createRarityMeterSection } from "./models/rarity-meter.js";
import {
  oracleOutputsHeatmapOption,
  oraclePaymentsHeatmapOption,
} from "../../src/heatmap/oracle.js";
import {
  coinflowWeightedUrpdHeatmapTree,
  cointimeWeightedUrpdHeatmapTree,
  rawUrpdHeatmapTree,
} from "../../src/heatmap/urpd.js";
import { lazy, lazyGroup } from "./lazy.js";

// Re-export types for external consumers
export * from "./types.js";

/**
 * Create partial options tree
 * @returns {PartialOptionsTree}
 */
export function createPartialOptions() {
  const data = lazy(buildCohortData);

  return [
    {
      name: "Explorer",
      kind: "explorer",
      title: "Explorer",
    },

    {
      name: "Charts",
      tree: [
        lazyGroup("Market", createMarketSection),

        lazyGroup("Network", createNetworkSection),

        lazyGroup("Mining", createMiningSection),

        {
          name: "Distribution",
          tree: [
            lazyGroup("Overview", () =>
              createCohortFolderAll({
                ...data().cohortAll,
                name: "Overview",
              }),
            ),

            lazyGroup("STH vs LTH", () =>
              createGroupedCohortFolderWithNupl({
                name: "STH vs LTH",
                title: "STH vs LTH",
                list: [data().termShort, data().termLong],
                all: data().cohortAll,
              }),
            ),

            lazyGroup("STH", () => createCohortFolderFull(data().termShort)),

            lazyGroup("LTH", () => createCohortFolderLongTerm(data().termLong)),

            lazyGroup("Age Cutoffs", () => ({
              name: "Age Cutoffs",
              tree: data().ageCutoff.map((cohort) =>
                lazyGroup(cohort.name, () => createCohortFolderFull(cohort)),
              ),
            })),

            lazyGroup("UTXO Age", () => {
              const { ageRange, cohortAll } = data();
              return {
                name: "UTXO Age",
                tree: [
                  lazyGroup("Compare", () =>
                    createGroupedCohortFolderAgeRangeWithMatured({
                      name: "Compare",
                      title: "Age Ranges",
                      list: ageRange,
                      all: cohortAll,
                    }),
                  ),
                  ...ageRange.map((cohort) =>
                    lazyGroup(cohort.name, () =>
                      createCohortFolderAgeRangeWithMatured(cohort),
                    ),
                  ),
                ],
              };
            }),

            lazyGroup("UTXO Size", () => {
              const { cohortAll, utxosAmountRange } = data();
              return {
                name: "UTXO Size",
                tree: [
                  lazyGroup("Compare", () =>
                    createGroupedCohortFolderBasicWithMarketCap({
                      name: "Compare",
                      title: "Amount Ranges",
                      list: utxosAmountRange,
                      all: cohortAll,
                    }),
                  ),
                  ...utxosAmountRange.map((cohort) =>
                    lazyGroup(cohort.name, () =>
                      createCohortFolderBasicWithMarketCap(cohort),
                    ),
                  ),
                ],
              };
            }),

            lazyGroup("Address Balance", () => {
              const { addressesAmountRange, cohortAll } = data();
              return {
                name: "Address Balance",
                tree: [
                  lazyGroup("Compare", () =>
                    createGroupedAddressCohortFolder({
                      name: "Compare",
                      title: "Balance Ranges",
                      list: addressesAmountRange,
                      all: cohortAll,
                    }),
                  ),
                  ...addressesAmountRange.map((cohort) =>
                    lazyGroup(cohort.name, () =>
                      createAddressCohortFolder(cohort),
                    ),
                  ),
                  createAddressBalanceGiniLeaf(),
                ],
              };
            }),

            lazyGroup("Script Type", () => {
              const { cohortAll, typeAddressable, typeOther } = data();
              return {
                name: "Script Type",
                tree: [
                  lazyGroup("Compare", () =>
                    createGroupedCohortFolderAddress({
                      name: "Compare",
                      title: "Script Type",
                      list: typeAddressable,
                      all: cohortAll,
                    }),
                  ),
                  .../** @satisfies {readonly SpendableType[]} */ ([
                    "p2a",
                    "p2tr",
                    "p2wsh",
                    "p2wpkh",
                    "p2sh",
                    "p2ms",
                    "p2pkh",
                    "p2pk33",
                    "p2pk65",
                    "empty",
                    "unknown",
                  ]).flatMap((key) => {
                    const addr = typeAddressable.find((t) => t.key === key);
                    if (addr)
                      return [
                        lazyGroup(addr.name, () =>
                          createCohortFolderAddress(addr),
                        ),
                      ];
                    const other = typeOther.find((t) => t.key === key);
                    if (other)
                      return [
                        lazyGroup(other.name, () =>
                          createCohortFolderWithoutRelative(other),
                        ),
                      ];
                    return [];
                  }),
                ],
              };
            }),

            lazyGroup("Epoch", () => {
              const { cohortAll, epoch } = data();
              return {
                name: "Epoch",
                tree: [
                  lazyGroup("Compare", () =>
                    createGroupedCohortFolderCore({
                      name: "Compare",
                      title: "Epoch",
                      list: epoch,
                      all: cohortAll,
                    }),
                  ),
                  ...epoch.map((cohort) =>
                    lazyGroup(cohort.name, () =>
                      createCohortFolderCore(cohort),
                    ),
                  ),
                ],
              };
            }),

            lazyGroup("Class", () => {
              const { class: class_, cohortAll } = data();
              return {
                name: "Class",
                tree: [
                  lazyGroup("Compare", () =>
                    createGroupedCohortFolderCore({
                      name: "Compare",
                      title: "Class",
                      list: class_,
                      all: cohortAll,
                    }),
                  ),
                  ...class_.map((cohort) =>
                    lazyGroup(cohort.name, () =>
                      createCohortFolderCore(cohort),
                    ),
                  ),
                ],
              };
            }),
          ],
        },

        lazyGroup("Frameworks", () => ({
          name: "Frameworks",
          tree: [createCointimeSection(), createCoinflowSection()],
        })),

        lazyGroup("Models", () => ({
          name: "Models",
          tree: [
            createBedrockSection(),
            createRarityMeterSection(),
            createCapitalSentimentSection(),
          ],
        })),
      ],
    },

    {
      name: "Heatmaps",
      tree: [
        {
          name: "Output Values",
          tree: [oracleOutputsHeatmapOption, oraclePaymentsHeatmapOption],
        },
        {
          name: "Price Distributions",
          tree: [
            ...rawUrpdHeatmapTree,
            {
              name: "Cointime Weighted",
              tree: cointimeWeightedUrpdHeatmapTree,
            },
            {
              name: "Coinflow Weighted",
              tree: coinflowWeightedUrpdHeatmapTree,
            },
          ],
        },
      ],
    },

    {
      name: "API",
      url: () => "/api",
      title: "API documentation",
    },

    {
      name: "Source",
      url: () => "https://bitcoinresearchkit.org",
      title: "Bitcoin Research Kit",
    },
  ];
}
