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
  createUtxoProfitabilitySection,
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
import { createInvestingSection } from "./investing.js";
import {
  oracleOutputsHeatmapOption,
  oraclePaymentsHeatmapOption,
} from "../../src/heatmap/oracle.js";
import {
  coinflowWeightedUrpdHeatmapTree,
  cointimeWeightedUrpdHeatmapTree,
  rawUrpdHeatmapTree,
} from "../../src/heatmap/urpd.js";

// Re-export types for external consumers
export * from "./types.js";

/**
 * Create partial options tree
 * @returns {PartialOptionsTree}
 */
export function createPartialOptions() {
  // Build cohort data
  const {
    cohortAll,
    termShort,
    termLong,
    underAge,
    overAge,
    ageRange,
    epoch,
    entry,
    utxosOverAmount,
    addressesOverAmount,
    utxosUnderAmount,
    addressesUnderAmount,
    utxosAmountRange,
    addressesAmountRange,
    typeAddressable,
    typeOther,
    class: class_,
    profitabilityRange,
    profitabilityProfit,
    profitabilityLoss,
  } = buildCohortData();

  return [
    {
      name: "Explorer",
      kind: "explorer",
      title: "Explorer",
    },

    {
      name: "Charts",
      tree: [
        createMarketSection(),

        createNetworkSection(),

        createMiningSection(),

        {
          name: "Distribution",
          tree: [
            createCohortFolderAll({ ...cohortAll, name: "Overview" }),

            createGroupedCohortFolderWithNupl({
              name: "STH vs LTH",
              title: "STH vs LTH",
              list: [termShort, termLong],
              all: cohortAll,
            }),

            createCohortFolderFull(termShort),

            createCohortFolderLongTerm(termLong),

            {
              name: "Entry",
              tree: [
                createGroupedCohortFolderCore({
                  name: "Compare",
                  title: "Veteran vs Rookie",
                  list: entry,
                  all: cohortAll,
                }),
                ...entry.map(createCohortFolderCore),
              ],
            },

            {
              name: "UTXO Age",
              tree: [
                {
                  name: "Under",
                  tree: [
                    createGroupedCohortFolderCore({
                      name: "Compare",
                      title: "Under Age",
                      list: underAge,
                      all: cohortAll,
                    }),
                    ...underAge.map(createCohortFolderCore),
                  ],
                },
                {
                  name: "Over",
                  tree: [
                    createGroupedCohortFolderCore({
                      name: "Compare",
                      title: "Over Age",
                      list: overAge,
                      all: cohortAll,
                    }),
                    ...overAge.map(createCohortFolderCore),
                  ],
                },
                {
                  name: "Range",
                  tree: [
                    createGroupedCohortFolderAgeRangeWithMatured({
                      name: "Compare",
                      title: "Age Ranges",
                      list: ageRange,
                      all: cohortAll,
                    }),
                    ...ageRange.map(createCohortFolderAgeRangeWithMatured),
                  ],
                },
              ],
            },

            {
              name: "UTXO Size",
              tree: [
                {
                  name: "Under",
                  tree: [
                    createGroupedCohortFolderBasicWithMarketCap({
                      name: "Compare",
                      title: "Under Amount",
                      list: utxosUnderAmount,
                      all: cohortAll,
                    }),
                    ...utxosUnderAmount.map(
                      createCohortFolderBasicWithMarketCap,
                    ),
                  ],
                },
                {
                  name: "Over",
                  tree: [
                    createGroupedCohortFolderBasicWithMarketCap({
                      name: "Compare",
                      title: "Over Amount",
                      list: utxosOverAmount,
                      all: cohortAll,
                    }),
                    ...utxosOverAmount.map(
                      createCohortFolderBasicWithMarketCap,
                    ),
                  ],
                },
                {
                  name: "Range",
                  tree: [
                    createGroupedCohortFolderBasicWithMarketCap({
                      name: "Compare",
                      title: "Amount Ranges",
                      list: utxosAmountRange,
                      all: cohortAll,
                    }),
                    ...utxosAmountRange.map(
                      createCohortFolderBasicWithMarketCap,
                    ),
                  ],
                },
              ],
            },

            createUtxoProfitabilitySection({
              range: profitabilityRange,
              profit: profitabilityProfit,
              loss: profitabilityLoss,
            }),

            {
              name: "Address Balance",
              tree: [
                {
                  name: "Under",
                  tree: [
                    createGroupedAddressCohortFolder({
                      name: "Compare",
                      title: "Under Balance",
                      list: addressesUnderAmount,
                      all: cohortAll,
                    }),
                    ...addressesUnderAmount.map(createAddressCohortFolder),
                  ],
                },
                {
                  name: "Over",
                  tree: [
                    createGroupedAddressCohortFolder({
                      name: "Compare",
                      title: "Over Balance",
                      list: addressesOverAmount,
                      all: cohortAll,
                    }),
                    ...addressesOverAmount.map(createAddressCohortFolder),
                  ],
                },
                {
                  name: "Range",
                  tree: [
                    createGroupedAddressCohortFolder({
                      name: "Compare",
                      title: "Balance Ranges",
                      list: addressesAmountRange,
                      all: cohortAll,
                    }),
                    ...addressesAmountRange.map(createAddressCohortFolder),
                  ],
                },
                createAddressBalanceGiniLeaf(),
              ],
            },

            {
              name: "Script Type",
              tree: [
                createGroupedCohortFolderAddress({
                  name: "Compare",
                  title: "Script Type",
                  list: typeAddressable,
                  all: cohortAll,
                }),
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
                  if (addr) return [createCohortFolderAddress(addr)];
                  const other = typeOther.find((t) => t.key === key);
                  if (other) return [createCohortFolderWithoutRelative(other)];
                  return [];
                }),
              ],
            },

            {
              name: "Epoch",
              tree: [
                createGroupedCohortFolderCore({
                  name: "Compare",
                  title: "Epoch",
                  list: epoch,
                  all: cohortAll,
                }),
                ...epoch.map(createCohortFolderCore),
              ],
            },

            {
              name: "Class",
              tree: [
                createGroupedCohortFolderCore({
                  name: "Compare",
                  title: "Class",
                  list: class_,
                  all: cohortAll,
                }),
                ...class_.map(createCohortFolderCore),
              ],
            },
          ],
        },

        createInvestingSection(),

        {
          name: "Frameworks",
          tree: [createCointimeSection(), createCoinflowSection()],
        },

        {
          name: "Models",
          tree: [
            createBedrockSection(),
            createRarityMeterSection(),
            createCapitalSentimentSection(),
          ],
        },
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
